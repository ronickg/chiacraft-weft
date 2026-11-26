//! A mod's own data in one world (`storage` in the WIT world): small values by key, in a folder of
//! the world's save that holds nothing else, within quotas. Keys are file names the sandbox
//! checks, so a mod can't name a path.
use std::collections::BTreeMap;
use std::path::PathBuf;

/// At most this many keys, bytes in all, and bytes written per call.
pub const MAX_KEYS: usize = 1024;
pub const MAX_BYTES: u64 = 4 << 20;
pub const MAX_KEY_CHARS: usize = 64;
/// Bytes a mod may write in one call: fuel bounds its CPU, not the time a disk write takes.
pub const MAX_WRITE_PER_CALL: u64 = 1 << 20;

/// A key: 1 to 64 lowercase letters, digits, `_`, `-` and `.`, not starting with a dot.
pub fn key_ok(key: &str) -> bool {
    (1..=MAX_KEY_CHARS).contains(&key.len())
        && !key.starts_with('.')
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "_-.".contains(c))
}

#[derive(Debug, Default)]
pub struct Storage {
    /// None: kept in memory only (a mod loaded without a world).
    dir: Option<PathBuf>,
    values: BTreeMap<String, Vec<u8>>,
}

impl Storage {
    pub fn memory() -> Storage {
        Storage::default()
    }

    /// The storage kept in `dir` (made if missing). Files there that aren't keys are left alone
    /// and not read.
    pub fn open(dir: PathBuf) -> Result<Storage, String> {
        std::fs::create_dir_all(&dir).map_err(|e| format!("storage: {e}"))?;
        let mut s = Storage { dir: Some(dir.clone()), values: BTreeMap::new() };
        let mut total = 0u64;
        for e in std::fs::read_dir(&dir)
            .map_err(|e| format!("storage: {e}"))?
            .flatten()
        {
            let name = e.file_name().to_string_lossy().into_owned();
            let Ok(t) = e.file_type() else { continue };
            if !t.is_file() || !key_ok(&name) || s.values.len() >= MAX_KEYS {
                continue;
            }
            let Ok(bytes) = std::fs::read(e.path()) else { continue };
            total += bytes.len() as u64;
            if total > MAX_BYTES {
                break;
            }
            s.values.insert(name, bytes);
        }
        Ok(s)
    }

    fn total(&self) -> u64 {
        self.values.values().map(|v| v.len() as u64).sum()
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.values.get(key).cloned()
    }

    pub fn set(&mut self, key: &str, value: Vec<u8>) -> Result<(), String> {
        if !key_ok(key) {
            return Err("a key is 1 to 64 lowercase letters, digits, `_`, `-` and `.`".into());
        }
        let old = self.values.get(key).map_or(0, |v| v.len() as u64);
        if !self.values.contains_key(key) && self.values.len() >= MAX_KEYS {
            return Err(format!("at most {MAX_KEYS} keys"));
        }
        if self.total() - old + value.len() as u64 > MAX_BYTES {
            return Err(format!("at most {} MiB in all", MAX_BYTES >> 20));
        }
        if let Some(dir) = &self.dir {
            // Written beside and moved over, so a crash leaves the old value or the new.
            let tmp = dir.join(format!(".{key}.tmp"));
            std::fs::write(&tmp, &value)
                .and_then(|()| std::fs::rename(&tmp, dir.join(key)))
                .map_err(|e| format!("storage: {e}"))?;
        }
        self.values.insert(key.to_string(), value);
        Ok(())
    }

    pub fn remove(&mut self, key: &str) {
        if self.values.remove(key).is_some()
            && let Some(dir) = &self.dir
        {
            let _ = std::fs::remove_file(dir.join(key));
        }
    }

    pub fn keys(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_cannot_name_paths() {
        for bad in ["", ".hidden", "a/b", "..", "A", "a\\b", "a b", &"x".repeat(65)] {
            assert!(!key_ok(bad), "{bad:?}");
        }
        for good in ["members", "tribe.alice", "a-b_c.1"] {
            assert!(key_ok(good), "{good}");
        }
    }

    #[test]
    fn quotas_hold() {
        let mut s = Storage::memory();
        s.set("a", vec![0; (MAX_BYTES - 10) as usize]).unwrap();
        assert!(s.set("b", vec![0; 11]).is_err());
        s.set("b", vec![0; 10]).unwrap();
        // Replacing a value counts only the difference.
        s.set("a", vec![0; 5]).unwrap();
        s.set("b", vec![0; 1000]).unwrap();
        for i in 0..MAX_KEYS - 2 {
            s.set(&format!("k{i}"), Vec::new()).unwrap();
        }
        assert!(s.set("one-more", Vec::new()).is_err());
        s.set("a", vec![1]).unwrap();
    }

    #[test]
    fn kept_in_the_folder() {
        let dir = std::env::temp_dir().join(format!("chiacraft-sandbox-storage-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut s = Storage::open(dir.clone()).unwrap();
        s.set("members", b"alice bob".to_vec()).unwrap();
        s.set("gone", b"x".to_vec()).unwrap();
        s.remove("gone");
        std::fs::write(dir.join("Not A Key"), "ignored").unwrap();
        let again = Storage::open(dir.clone()).unwrap();
        assert_eq!(again.keys(), vec!["members".to_string()]);
        assert_eq!(again.get("members").unwrap(), b"alice bob");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
