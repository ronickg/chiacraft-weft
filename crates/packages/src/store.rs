//! The package store: every package file kept once by its BLAKE3 hash, and each package laid out
//! under its own hash from those files, checked again whenever it is used.
//!
//! ```text
//! <root>/blobs/<file hash>                 a file's bytes
//! <root>/packages/<package hash>/manifest.txt
//! <root>/packages/<package hash>/files/…   the package's files, as the manifest lists them
//! <root>/packages/<package hash>/.done
//! <root>/packages/<package hash>/package.sig   who made it, if signed (not covered by the hash)
//! ```
//!
//! Paths under `files/` come only from a checked manifest (`identity::valid_file_path`), never
//! straight from an archive or a download. A package arrives either from a folder on disk
//! ([`Store::import_dir`]) or, later, as a manifest and blobs fetched from other players.

use std::path::{Path, PathBuf};

use crate::identity::{FileManifest, Hash};

pub struct Store {
    root: PathBuf,
}

/// What [`Store::tidy`] removed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Tidied {
    pub packages: usize,
    pub files: usize,
    pub bytes: u64,
}

impl Store {
    pub fn new(root: impl Into<PathBuf>) -> Store {
        Store { root: root.into() }
    }

    fn blob(&self, h: &Hash) -> PathBuf {
        self.root.join("blobs").join(h.hex())
    }

    fn package(&self, h: &Hash) -> PathBuf {
        self.root.join("packages").join(h.hex())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// A blob's file, if the store has it.
    pub fn blob_path(&self, h: &Hash) -> Option<PathBuf> {
        let p = self.blob(h);
        p.is_file().then_some(p)
    }

    /// A laid-out package's manifest text, checked against its hash.
    pub fn manifest_text(&self, h: &Hash) -> Result<String, String> {
        let text = std::fs::read_to_string(self.package(h).join("manifest.txt")).map_err(|e| format!("package {h:?}: {e}"))?;
        if Hash::of(text.as_bytes()) != *h {
            return Err(format!("package {h:?}: its manifest doesn't match its hash"));
        }
        Ok(text)
    }

    /// Whether the store has a blob with these bytes.
    pub fn has_blob(&self, h: &Hash) -> bool {
        self.blob(h).is_file()
    }

    /// Keeps `bytes` as a blob, refusing bytes that don't match `expected`.
    pub fn put_blob(&self, expected: &Hash, bytes: &[u8]) -> Result<(), String> {
        if Hash::of(bytes) != *expected {
            return Err(format!("blob {expected:?}: the bytes don't match their hash"));
        }
        let path = self.blob(expected);
        if path.is_file() {
            return Ok(());
        }
        write_atomic(&path, bytes)
    }

    /// The folder of a package that is in the store and laid out.
    pub fn dir(&self, h: &Hash) -> Option<PathBuf> {
        let d = self.package(h);
        d.join(".done").is_file().then(|| d.join("files"))
    }

    pub fn manifest(&self, h: &Hash) -> Result<FileManifest, String> {
        let text = std::fs::read_to_string(self.package(h).join("manifest.txt")).map_err(|e| format!("package {h:?}: {e}"))?;
        if Hash::of(text.as_bytes()) != *h {
            return Err(format!("package {h:?}: its manifest doesn't match its hash"));
        }
        FileManifest::parse(&text).map_err(|e| format!("package {h:?}: {e}"))
    }

    /// Copies the package in folder `dir` into the store and lays it out; gives its hash.
    pub fn import_dir(&self, dir: &Path) -> Result<Hash, String> {
        let manifest = FileManifest::of_dir(dir)?;
        let h = manifest.hash();
        if self.dir(&h).is_some() {
            if let Ok(sig) = std::fs::read_to_string(dir.join(crate::signature::SIG_FILE)) {
                self.put_signature(&h, &sig)?;
            }
            return Ok(h);
        }
        for f in &manifest.files {
            if !self.has_blob(&f.hash) {
                let bytes = std::fs::read(dir.join(&f.path)).map_err(|e| format!("{}: {e}", f.path))?;
                // Changed while being read: the hash taken a moment ago no longer holds.
                self.put_blob(&f.hash, &bytes)
                    .map_err(|_| format!("{} changed while being read", f.path))?;
            }
        }
        self.lay_out(&manifest)?;
        if let Ok(sig) = std::fs::read_to_string(dir.join(crate::signature::SIG_FILE)) {
            self.put_signature(&h, &sig)?;
        }
        Ok(h)
    }

    /// Keeps a package's signature envelope beside it (checking it is the caller's).
    pub fn put_signature(&self, h: &Hash, text: &str) -> Result<(), String> {
        if self.dir(h).is_none() {
            return Err(format!("package {h:?} is not in the store"));
        }
        let e = crate::signature::Envelope::parse(text)?;
        if e.package != h.hex() {
            return Err("the signature is for another package".into());
        }
        write_atomic(&self.package(h).join(crate::signature::SIG_FILE), e.text().as_bytes())
    }

    /// A package's signature envelope, if it has one (not yet checked).
    pub fn signature(&self, h: &Hash) -> Option<String> {
        std::fs::read_to_string(self.package(h).join(crate::signature::SIG_FILE)).ok()
    }

    /// Lays out a package whose blobs are all in the store.
    pub fn lay_out(&self, manifest: &FileManifest) -> Result<PathBuf, String> {
        let h = manifest.hash();
        if let Some(d) = self.dir(&h) {
            return Ok(d);
        }
        let dest = self.package(&h);
        let tmp = self
            .root
            .join("packages")
            .join(format!(".{}-{}", h.hex(), std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let files = tmp.join("files");
        for f in &manifest.files {
            let to = files.join(&f.path);
            std::fs::create_dir_all(to.parent().unwrap_or(&files)).map_err(|e| e.to_string())?;
            let from = self.blob(&f.hash);
            if !from.is_file() {
                let _ = std::fs::remove_dir_all(&tmp);
                return Err(format!("package {h:?}: file {} is not in the store", f.path));
            }
            std::fs::copy(&from, &to).map_err(|e| format!("{}: {e}", f.path))?;
        }
        std::fs::write(tmp.join("manifest.txt"), manifest.text()).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_dir_all(&dest);
        std::fs::rename(&tmp, &dest).map_err(|e| e.to_string())?;
        std::fs::write(dest.join(".done"), "").map_err(|e| e.to_string())?;
        Ok(dest.join("files"))
    }

    /// Marks a package as just used: the packages used longest ago are the first to go when the
    /// store is tidied.
    pub fn touch(&self, h: &Hash) {
        if let Ok(f) = std::fs::File::options()
            .write(true)
            .open(self.package(h).join(".done"))
        {
            let _ = f.set_modified(std::time::SystemTime::now());
        }
    }

    /// Tidies the store: removes what crashed runs left half-done, then packages not in `keep`
    /// beyond `budget` bytes (those used longest ago first), then every file no package left
    /// needs. Packages in `keep` (those worlds on this computer pin, and the installed ones) are
    /// never removed.
    pub fn tidy(&self, keep: &[Hash], budget: u64) -> Tidied {
        let mut done = Tidied::default();
        let stale = |p: &Path| {
            p.metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.elapsed().ok())
                .is_some_and(|age| age.as_secs() > 3600)
        };
        let mut kept: Vec<FileManifest> = Vec::new();
        let mut spare: Vec<(std::time::SystemTime, u64, Hash, FileManifest)> = Vec::new();
        for e in std::fs::read_dir(self.root.join("packages"))
            .into_iter()
            .flatten()
            .flatten()
        {
            let path = e.path();
            let name = e.file_name().to_string_lossy().into_owned();
            let Some(h) = Hash::from_hex(&name) else {
                // Another run's half-laid-out package, or something that isn't the store's.
                if name.starts_with('.') && stale(&path) {
                    let _ = std::fs::remove_dir_all(&path);
                }
                continue;
            };
            let manifest = self
                .dir(&h)
                .ok_or(String::new())
                .and_then(|_| self.manifest(&h));
            match manifest {
                Ok(m) if keep.contains(&h) => kept.push(m),
                Ok(m) => {
                    let used = path
                        .join(".done")
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::UNIX_EPOCH);
                    spare.push((used, m.files.iter().map(|f| f.size).sum(), h, m));
                }
                Err(_) => {
                    let _ = std::fs::remove_dir_all(&path);
                    done.packages += 1;
                }
            }
        }
        spare.sort_by_key(|s| std::cmp::Reverse(s.0));
        let mut used = 0u64;
        for (_, size, h, m) in spare {
            used += size;
            if used <= budget {
                kept.push(m);
            } else if std::fs::remove_dir_all(self.package(&h)).is_ok() {
                done.packages += 1;
            }
        }
        let needed: std::collections::HashSet<Hash> = kept
            .iter()
            .flat_map(|m| m.files.iter().map(|f| f.hash))
            .collect();
        for e in std::fs::read_dir(self.root.join("blobs"))
            .into_iter()
            .flatten()
            .flatten()
        {
            let path = e.path();
            let name = e.file_name().to_string_lossy().into_owned();
            let remove = match Hash::from_hex(&name) {
                Some(h) => !needed.contains(&h),
                None => name.starts_with('.') && stale(&path),
            };
            if remove {
                let size = path.metadata().map_or(0, |m| m.len());
                if std::fs::remove_file(&path).is_ok() {
                    done.files += 1;
                    done.bytes += size;
                }
            }
        }
        done
    }

    /// Checks a laid-out package against its hash: the manifest, and every file's size and hash,
    /// with nothing extra. Gives its folder.
    pub fn verify(&self, h: &Hash) -> Result<PathBuf, String> {
        let dir = self
            .dir(h)
            .ok_or_else(|| format!("package {h:?} is not in the store"))?;
        let manifest = self.manifest(h)?;
        let mut found = Vec::new();
        list(&dir, &dir, &mut found);
        if found.len() != manifest.files.len() {
            return Err(format!("package {h:?}: files were added or removed"));
        }
        for f in &manifest.files {
            let bytes = std::fs::read(dir.join(&f.path)).map_err(|e| format!("package {h:?}: {}: {e}", f.path))?;
            if bytes.len() as u64 != f.size || Hash::of(&bytes) != f.hash {
                return Err(format!("package {h:?}: {} was changed", f.path));
            }
        }
        Ok(dir)
    }
}

fn list(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            list(root, &p, out);
        } else {
            out.push(p.strip_prefix(root).unwrap_or(&p).to_path_buf());
        }
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let dir = path.parent().ok_or("no folder")?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let tmp = dir.join(format!(".{}-{}", path.file_name().unwrap_or_default().to_string_lossy(), std::process::id()));
    std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("chiacraft-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    /// A package imported from a folder is laid out under its hash and checks out; a changed
    /// file is caught; importing again is a no-op.
    #[test]
    fn import_lay_out_verify() {
        let src = temp("src");
        std::fs::create_dir_all(src.join("defs/items")).unwrap();
        std::fs::write(src.join("package.toml"), "format = \"1.0\"\nid = \"gems\"\nversion = \"0.1.0\"\n").unwrap();
        std::fs::write(src.join("defs/items/ruby.toml"), "max_stack = 16\n").unwrap();
        let store = Store::new(temp("root"));
        let h = store.import_dir(&src).unwrap();
        assert_eq!(h, FileManifest::of_dir(&src).unwrap().hash());
        let dir = store.verify(&h).unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("defs/items/ruby.toml")).unwrap(), "max_stack = 16\n");
        assert_eq!(store.import_dir(&src).unwrap(), h);
        std::fs::write(dir.join("defs/items/ruby.toml"), "max_stack = 99\n").unwrap();
        assert!(store.verify(&h).unwrap_err().contains("was changed"));
        assert!(store.put_blob(&Hash::of(b"a"), b"b").is_err());
        let _ = std::fs::remove_dir_all(&src);
        let _ = std::fs::remove_dir_all(store.root);
    }

    /// Tidying keeps pinned packages whatever the budget, keeps the most recently used others
    /// within it, and removes files no package left needs.
    #[test]
    fn tidy_keeps_pins_and_recent() {
        let store = Store::new(temp("tidy"));
        let make = |n: u32| {
            let src = temp(&format!("tidy-src{n}"));
            std::fs::create_dir_all(src.join("defs/items")).unwrap();
            std::fs::write(src.join("package.toml"), format!("format = \"1.0\"\nid = \"p{n}\"\nversion = \"0.1.0\"\n")).unwrap();
            std::fs::write(src.join("defs/items/thing.toml"), format!("max_stack = {n}\n")).unwrap();
            let h = store.import_dir(&src).unwrap();
            let _ = std::fs::remove_dir_all(&src);
            h
        };
        let (pinned, old, recent) = (make(1), make(2), make(3));
        let size = store
            .manifest(&recent)
            .unwrap()
            .files
            .iter()
            .map(|f| f.size)
            .sum::<u64>();
        let past = std::time::SystemTime::now() - std::time::Duration::from_secs(1000);
        for h in [pinned, old] {
            std::fs::File::options()
                .write(true)
                .open(store.package(&h).join(".done"))
                .unwrap()
                .set_modified(past)
                .unwrap();
        }
        store.touch(&recent);
        std::fs::write(store.blob(&Hash::of(b"stray")), b"stray").unwrap();
        let done = store.tidy(&[pinned], size);
        assert_eq!(done.packages, 1);
        assert!(store.verify(&pinned).is_ok() && store.verify(&recent).is_ok() && store.dir(&old).is_none());
        assert!(!store.has_blob(&Hash::of(b"stray")));
        assert_eq!(store.tidy(&[pinned], size), Tidied::default());
        let _ = std::fs::remove_dir_all(store.root);
    }
}
