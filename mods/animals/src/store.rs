//! The files the mod keeps in a world (the gene pools, `evolution.rs`): written whole and
//! atomically (a temporary file beside it, then a rename, so a crash mid-write leaves the old
//! file), and never lost to a file that fails to read: one that is there but can't be read or
//! parsed is moved aside as `<name>.bad-<seconds since 1970>` and logged as an error; if even that
//! fails, nothing is written over it.
//!
//! The same calls as the `chiacraft-store` crate (Phase 3's leftovers): this goes when that crate
//! is in `dev` and the mod depends on it.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Writes `bytes` to `path` whole: to a temporary file beside it, then renamed over it.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let name = path
        .file_name()
        .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidInput, "no file name"))?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut tmp = name.to_os_string();
    tmp.push(format!(".tmp-{}", std::process::id()));
    let tmp = path.with_file_name(tmp);
    let r = (|| {
        use std::io::Write;
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        std::fs::rename(&tmp, path)
    })();
    if r.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    r
}

/// Moves a file that couldn't be read aside, as `<name>.bad-<secs>`; where it went, or `None` if
/// it couldn't be moved (then nothing may be written over it).
pub fn keep_aside(path: &Path, why: &str) -> Option<PathBuf> {
    let mut name = path.file_name()?.to_os_string();
    name.push(format!(".bad-{}", now_secs()));
    let aside = path.with_file_name(name);
    match std::fs::rename(path, &aside) {
        Ok(()) => {
            weft_sdk::error!("{} could not be read ({why}): kept as {}, starting empty", path.display(), aside.display());
            Some(aside)
        }
        Err(e) => {
            weft_sdk::error!("{} could not be read ({why}) nor moved aside ({e}): it will not be written over", path.display());
            None
        }
    }
}

/// What reading a file gave.
pub enum Read<T> {
    /// It was there and read.
    Fine(T),
    /// It wasn't there.
    Missing,
    /// It was there but couldn't be read: moved aside (writing is safe again), or not.
    Bad { moved: bool },
}

/// Reads a JSON file; one that is there but can't be read or doesn't parse is kept aside.
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Read<T> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == ErrorKind::NotFound => return Read::Missing,
        Err(e) => return Read::Bad { moved: keep_aside(path, &e.to_string()).is_some() },
    };
    match serde_json::from_str(&text) {
        Ok(v) => Read::Fine(v),
        Err(e) => Read::Bad { moved: keep_aside(path, &e.to_string()).is_some() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn a_bad_file_is_kept_aside_not_lost() {
        let dir = std::env::temp_dir().join(format!("animals-store-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("pools").join("0_0.json");
        write_atomic(&path, b"{\"a\": 1}").unwrap();
        assert!(matches!(read_json::<HashMap<String, u32>>(&path), Read::Fine(m) if m["a"] == 1));
        std::fs::write(&path, "{ not json").unwrap();
        assert!(matches!(read_json::<HashMap<String, u32>>(&path), Read::Bad { moved: true }));
        assert!(!path.exists(), "moved aside");
        let kept: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .filter(|n| n.starts_with("0_0.json.bad-"))
            .collect();
        assert_eq!(kept.len(), 1);
        assert!(matches!(read_json::<HashMap<String, u32>>(&path), Read::Missing));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
