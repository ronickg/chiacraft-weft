//! The file store: `<dir>/<fingerprint>.key`, the entropy encrypted with AES-256-GCM under a key
//! that Argon2id derives from a passphrase and a random salt. That is the scheme of Sage's
//! `sage-keychain` (`encrypt.rs`; Apache-2.0, see the crate's NOTICE), in our own JSON container
//! rather than Sage's bincode `keys.bin`. Sage always encrypts with an empty password; here the
//! player may set a real one, and an empty one is shown as "not protected". Files are written
//! readable by their owner only.
use super::{SecretStore, StoreKind};
use crate::{
    Error, Result,
    keys::{from_hex, hex},
    secret::{Passphrase, Secret},
};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use std::path::{Path, PathBuf};

/// The KDF named in each file (argon2 0.5's defaults: Argon2id v19, 19 MiB, 2 passes, 1 lane).
const KDF: &str = "argon2id-v19-m19456-t2-p1";

pub struct FileStore {
    dir: PathBuf,
    passphrase: Passphrase,
}
impl FileStore {
    pub fn new(dir: impl Into<PathBuf>, passphrase: Passphrase) -> Self {
        Self {
            dir: dir.into(),
            passphrase,
        }
    }
    fn path(&self, account: u32) -> PathBuf {
        self.dir.join(format!("{account}.key"))
    }
    /// Whether the account's file has a real passphrase (`None`: no file).
    pub fn protected(dir: &Path, account: u32) -> Option<bool> {
        let text = std::fs::read_to_string(dir.join(format!("{account}.key"))).ok()?;
        let v: serde_json::Value = serde_json::from_str(&text).ok()?;
        v.get("protected").and_then(serde_json::Value::as_bool)
    }
}

fn key_of(passphrase: &Passphrase, salt: &[u8]) -> Result<Secret<[u8; 32]>> {
    let mut key = Secret::new([0u8; 32]);
    argon2::Argon2::default()
        .hash_password_into(passphrase.expose().as_bytes(), salt, key.expose_mut())
        .map_err(|e| Error::new(format!("argon2: {e}")))?;
    Ok(key)
}

impl SecretStore for FileStore {
    fn kind(&self) -> StoreKind {
        StoreKind::File
    }
    fn put(&self, account: u32, entropy: &Secret<Vec<u8>>) -> Result<()> {
        let mut salt = [0u8; 32];
        let mut nonce = [0u8; 12];
        getrandom::fill(&mut salt).map_err(|e| Error::new(format!("randomness: {e}")))?;
        getrandom::fill(&mut nonce).map_err(|e| Error::new(format!("randomness: {e}")))?;
        let key = key_of(&self.passphrase, &salt)?;
        let cipher = Aes256Gcm::new_from_slice(key.expose()).map_err(|_| Error::new("aes key"))?;
        let sealed = cipher
            .encrypt(Nonce::from_slice(&nonce), entropy.expose().as_slice())
            .map_err(|_| Error::new("encrypt"))?;
        let doc = serde_json::json!({
            "v": 1,
            "kdf": KDF,
            "salt": hex(&salt),
            "nonce": hex(&nonce),
            "ciphertext": hex(&sealed),
            "protected": !self.passphrase.is_empty(),
        });
        std::fs::create_dir_all(&self.dir)?;
        write_private(&self.path(account), doc.to_string().as_bytes())
    }
    fn get(&self, account: u32) -> Result<Option<Secret<Vec<u8>>>> {
        let text = match std::fs::read_to_string(self.path(account)) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let damaged = || Error::new(format!("{} is damaged", self.path(account).display()));
        let v: serde_json::Value = serde_json::from_str(&text).map_err(|_| damaged())?;
        let field = |k: &str| {
            v.get(k)
                .and_then(serde_json::Value::as_str)
                .and_then(from_hex)
                .ok_or_else(damaged)
        };
        if v.get("kdf").and_then(serde_json::Value::as_str) != Some(KDF) {
            return Err(Error::new("unknown key derivation in the wallet file"));
        }
        let (salt, nonce, sealed) = (field("salt")?, field("nonce")?, field("ciphertext")?);
        if nonce.len() != 12 {
            return Err(damaged());
        }
        let key = key_of(&self.passphrase, &salt)?;
        let cipher = Aes256Gcm::new_from_slice(key.expose()).map_err(|_| Error::new("aes key"))?;
        let plain = cipher
            .decrypt(Nonce::from_slice(&nonce), sealed.as_slice())
            .map_err(|_| Error::wrong_passphrase())?;
        Ok(Some(Secret::new(plain)))
    }
    fn delete(&self, account: u32) -> Result<()> {
        match std::fs::remove_file(self.path(account)) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
            _ => Ok(()),
        }
    }
}

/// Write a file readable only by its owner (through a temporary file, then a rename).
pub(crate) fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        use std::io::Write;
        let mut f = options.open(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_refuses_a_wrong_passphrase() {
        let dir = std::env::temp_dir().join(format!("cc-wallet-file-{}", std::process::id()));
        let store = FileStore::new(&dir, Secret::new("correct horse".to_owned()));
        let entropy = Secret::new(vec![7u8; 32]);
        store.put(42, &entropy).unwrap();
        assert_eq!(store.get(42).unwrap().unwrap().expose(), entropy.expose());
        assert_eq!(FileStore::protected(&dir, 42), Some(true));
        let text = std::fs::read_to_string(dir.join("42.key")).unwrap();
        assert!(!text.contains(&hex(entropy.expose())));
        let wrong = FileStore::new(&dir, Secret::new("wrong".to_owned()));
        assert!(wrong.get(42).unwrap_err().is_wrong_passphrase());
        assert!(store.get(43).unwrap().is_none());
        store.delete(42).unwrap();
        assert!(store.get(42).unwrap().is_none());
        let _ = std::fs::remove_dir_all(dir);
    }
}
