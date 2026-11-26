//! What a package is, byte for byte: a canonical manifest listing every file's path, size and
//! BLAKE3 hash, and the BLAKE3 hash of that manifest, which names the package. The same files give
//! the same hash on every computer however they were copied or zipped; worlds pin these hashes.
//!
//! The manifest is text, one line each, files sorted by path (bytewise):
//!
//! ```text
//! chiacraft package 1
//! id gems
//! version 0.1.0
//! file assets/lang/en_us.lang 70 6f1c…
//! file defs/blocks/ruby_block.toml 101 a93e…
//! ```
//!
//! Hidden files and the signature envelope (`package.sig`, see `signature`) are not part of it.

use std::fmt;
use std::path::{Path, PathBuf};

const HEADER: &str = "chiacraft package 1";

/// A package's files may not exceed these: it is data someone else wrote.
pub const MAX_FILES: usize = 4096;
pub const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: u64 = 256 * 1024 * 1024;

/// The files a package may hold, by where they are and what they end in, and how big each kind
/// may be: only formats the game reads, decoded by its own pure-Rust code. Anything else is
/// refused before it is downloaded or stored.
const ALLOWED: &[(&str, &[&str], u64)] = &[
    ("defs/blocks/", &["toml"], 256 << 10),
    ("defs/items/", &["toml"], 256 << 10),
    ("defs/recipes/", &["toml"], 256 << 10),
    ("defs/tags/", &["toml"], 256 << 10),
    ("assets/lang/", &["lang"], 1 << 20),
    ("assets/textures/", &["png"], 16 << 20),
    ("assets/textures/", &["anim", "ctm", "gui"], 64 << 10),
    ("assets/models/", &["json"], 1 << 20),
    ("assets/sounds/", &["ogg", "wav"], MAX_FILE_BYTES),
];

/// The sandboxed server half (`docs/sandboxed-mods.md`): one WebAssembly component, at exactly
/// this path, run only inside the game's sandbox.
pub const SERVER_CODE: &str = "code/server.wasm";
/// The sandboxed client half, run in each player's own game.
pub const CLIENT_CODE: &str = "code/client.wasm";

/// Whether `path` is one of a package's sandboxed halves.
pub fn is_code(path: &str) -> bool {
    path == SERVER_CODE || path == CLIENT_CODE
}
pub const MAX_CODE_BYTES: u64 = 16 << 20;

/// Whether a package may hold a file at `path` of `size` bytes; why not if it may not.
pub fn check_file(path: &str, size: u64) -> Result<(), String> {
    if !valid_file_path(path) {
        return Err(format!("{path}: name files and folders with letters, digits, `_`, `-` and `.`"));
    }
    let max = if path == "package.toml" {
        64 << 10
    } else if path == SERVER_CODE || path == CLIENT_CODE {
        MAX_CODE_BYTES
    } else {
        let ext = path.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
        let kind = ALLOWED
            .iter()
            .find(|(dir, exts, _)| path.starts_with(dir) && exts.contains(&ext));
        let Some((_, _, max)) = kind else {
            return Err(format!("{path}: not a file a package may hold (see docs/packages.md, \"What a package may hold\")"));
        };
        *max
    };
    if size > max {
        return Err(format!("{path}: bigger than {} KiB", max >> 10));
    }
    Ok(())
}

/// A BLAKE3 hash: of a file, or of a manifest (then it names a package).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn of(bytes: &[u8]) -> Hash {
        Hash(*blake3::hash(bytes).as_bytes())
    }

    pub fn hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// The first 12 hex digits, for people.
    pub fn short(&self) -> String {
        self.hex()[..12].to_string()
    }

    pub fn from_hex(text: &str) -> Option<Hash> {
        if text.len() != 64 || !text.is_ascii() {
            return None;
        }
        let mut out = [0u8; 32];
        for (i, o) in out.iter_mut().enumerate() {
            *o = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).ok()?;
        }
        Some(Hash(out))
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.hex())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// `/`-separated, relative to the package folder.
    pub path: String,
    pub size: u64,
    pub hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileManifest {
    pub id: String,
    pub version: String,
    /// Sorted by path.
    pub files: Vec<FileEntry>,
}

/// A path a package may hold: segments of ASCII letters, digits, `_`, `-` and `.`, none empty or
/// starting with a dot. Nothing else can become a file name on a player's disk.
pub fn valid_file_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 255
        && path.split('/').all(|s| {
            !s.is_empty()
                && !s.starts_with('.')
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_-.".contains(c))
        })
}

impl FileManifest {
    /// The manifest of the package in folder `dir`: every file in it, hidden ones (names
    /// starting with a dot, like `.DS_Store`) left out.
    pub fn of_dir(dir: &Path) -> Result<FileManifest, String> {
        let m = crate::read_manifest(dir)?;
        let mut paths = Vec::new();
        walk(dir, dir, &mut paths)?;
        let mut files = Vec::with_capacity(paths.len());
        let mut total = 0u64;
        for (rel, full) in paths {
            if files.len() >= MAX_FILES {
                return Err(format!("more than {MAX_FILES} files"));
            }
            check_file(
                &rel,
                std::fs::metadata(&full)
                    .map_err(|e| format!("{rel}: {e}"))?
                    .len(),
            )?;
            let bytes = std::fs::read(&full).map_err(|e| format!("{rel}: {e}"))?;
            let size = bytes.len() as u64;
            check_file(&rel, size)?;
            total += size;
            if total > MAX_TOTAL_BYTES {
                return Err(format!("more than {} MiB in all", MAX_TOTAL_BYTES >> 20));
            }
            files.push(FileEntry { path: rel, size, hash: Hash::of(&bytes) });
        }
        files.sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));
        Ok(FileManifest { id: m.id, version: m.version, files })
    }

    /// The canonical text, which the package's hash is taken of.
    pub fn text(&self) -> String {
        let mut s = format!("{HEADER}\nid {}\nversion {}\n", self.id, self.version);
        for f in &self.files {
            s.push_str(&format!("file {} {} {}\n", f.path, f.size, f.hash.hex()));
        }
        s
    }

    /// The package's name: the hash of the canonical text.
    pub fn hash(&self) -> Hash {
        Hash::of(self.text().as_bytes())
    }

    /// Reads a manifest, refusing anything that is not exactly canonical (so its hash is the one
    /// it was published under).
    pub fn parse(text: &str) -> Result<FileManifest, String> {
        let mut lines = text.lines();
        if lines.next() != Some(HEADER) {
            return Err("not a package manifest".into());
        }
        let id = lines
            .next()
            .and_then(|l| l.strip_prefix("id "))
            .ok_or("no id line")?
            .to_string();
        let version = lines
            .next()
            .and_then(|l| l.strip_prefix("version "))
            .ok_or("no version line")?
            .to_string();
        let mut files = Vec::new();
        for l in lines {
            let mut parts = l
                .strip_prefix("file ")
                .ok_or_else(|| format!("bad line `{l}`"))?
                .split(' ');
            let (Some(path), Some(size), Some(hash), None) = (parts.next(), parts.next(), parts.next(), parts.next()) else {
                return Err(format!("bad line `{l}`"));
            };
            let size = size.parse().map_err(|_| format!("bad size in `{l}`"))?;
            check_file(path, size)?;
            let hash = Hash::from_hex(hash).ok_or_else(|| format!("bad hash in `{l}`"))?;
            files.push(FileEntry { path: path.to_string(), size, hash });
        }
        if files.len() > MAX_FILES || files.iter().map(|f| f.size).sum::<u64>() > MAX_TOTAL_BYTES {
            return Err("over the size limits".into());
        }
        let m = FileManifest { id, version, files };
        if m.text() != text {
            return Err("not in canonical form".into());
        }
        Ok(m)
    }
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        // Hidden files, and the signature envelope (it names the package's hash, so it can't be
        // one of the files that hash covers).
        if name.starts_with('.') || (dir == root && name == crate::signature::SIG_FILE) {
            continue;
        }
        let p = e.path();
        let rel = p
            .strip_prefix(root)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        // Links could lead out of the package.
        let t = e.file_type().map_err(|e| format!("{rel}: {e}"))?;
        if t.is_dir() {
            walk(root, &p, out)?;
        } else if t.is_file() {
            if !valid_file_path(&rel) {
                return Err(format!("{rel}: name files and folders with letters, digits, `_`, `-` and `.`"));
            }
            out.push((rel, p));
        } else {
            return Err(format!("{rel}: links are not allowed in a package"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("chiacraft-identity-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("defs/items")).unwrap();
        std::fs::write(d.join("package.toml"), "format = \"1.0\"\nid = \"gems\"\nversion = \"0.1.0\"\n").unwrap();
        std::fs::write(d.join("defs/items/ruby.toml"), "max_stack = 16\n").unwrap();
        d
    }

    /// The same files give the same hash; hidden files don't count; any change gives another.
    #[test]
    fn same_files_same_hash() {
        let a = package("a");
        let b = package("b");
        std::fs::write(b.join(".DS_Store"), "junk").unwrap();
        let (ma, mb) = (FileManifest::of_dir(&a).unwrap(), FileManifest::of_dir(&b).unwrap());
        assert_eq!(ma.hash(), mb.hash());
        assert_eq!(FileManifest::parse(&ma.text()).unwrap(), ma);
        std::fs::write(b.join("defs/items/ruby.toml"), "max_stack = 17\n").unwrap();
        assert_ne!(FileManifest::of_dir(&b).unwrap().hash(), ma.hash());
        let _ = std::fs::remove_dir_all(&a);
        let _ = std::fs::remove_dir_all(&b);
    }

    #[test]
    fn refuses_bad_manifests() {
        let a = package("parse");
        let text = FileManifest::of_dir(&a).unwrap().text();
        assert!(FileManifest::parse(&text.replace("defs/items/ruby.toml", "../ruby.toml")).is_err());
        assert!(FileManifest::parse(&format!("{text}\n")).is_err());
        assert!(
            !valid_file_path("a/../b")
                && !valid_file_path("/etc/passwd")
                && !valid_file_path("a/.hidden")
                && valid_file_path("assets/lang/en_us.lang")
        );
        let _ = std::fs::remove_dir_all(&a);
    }

    /// Only the kinds of file the game reads, each within its size.
    #[test]
    fn allow_list() {
        assert!(check_file("package.toml", 100).is_ok());
        assert!(check_file("defs/blocks/ruby.toml", 100).is_ok());
        assert!(check_file("assets/textures/block/ruby.png", 4096).is_ok());
        assert!(check_file("assets/textures/block/water.anim", 100).is_ok());
        assert!(check_file("assets/sounds/music/calm.ogg", 10 << 20).is_ok());
        assert!(check_file("assets/textures/block/ruby.exe", 10).is_err());
        assert!(check_file("assets/lang/en_us.png", 10).is_err());
        assert!(check_file("defs/blocks/ruby.json", 10).is_err());
        assert!(check_file("README.md", 10).is_err());
        assert!(check_file("assets/textures/block/ruby.png", 17 << 20).is_err());
        assert!(check_file("defs/items/ruby.toml", 300 << 10).is_err());
        // Sandboxed code: one component at one path, nothing else in `code/`.
        assert!(check_file("code/server.wasm", 1 << 20).is_ok());
        assert!(check_file("code/server.wasm", 17 << 20).is_err());
        assert!(check_file("code/client.wasm", 10).is_ok());
        assert!(check_file("code/other.wasm", 10).is_err());
        assert!(check_file("code/server.so", 10).is_err());
        assert!(check_file("assets/server.wasm", 10).is_err());
        let a = package("allow");
        std::fs::write(a.join("notes.txt"), "hi").unwrap();
        assert!(FileManifest::of_dir(&a).unwrap_err().contains("notes.txt"));
        std::fs::remove_file(a.join("notes.txt")).unwrap();
        let text = FileManifest::of_dir(&a).unwrap().text();
        assert!(FileManifest::parse(&text.replace("defs/items/ruby.toml", "defs/items/ruby.dll")).is_err());
        let _ = std::fs::remove_dir_all(&a);
    }
}
