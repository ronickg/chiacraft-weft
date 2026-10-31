//! The OS keystore: one generic item per account, named `<service>` / `<fingerprint>`.
//!
//! - macOS: a generic password in the login Keychain (keyring's `apple-native`).
//! - Windows: a generic credential in Credential Manager with `CRED_PERSIST_LOCAL_MACHINE`, so it
//!   stays on this computer (keyring's `CRED_PERSIST_ENTERPRISE` would roam with the profile).
//! - Linux and FreeBSD: an item in the Secret Service (GNOME Keyring, KWallet) through oo7.
//! - Elsewhere: no keystore; [`OsStore::put`] fails and the caller falls back to the file store.
//!
//! Checks never reach this module: a game home given by `HORIZON_HOME` uses the file store unless
//! `HORIZON_WALLET_STORE=os` says otherwise (`crate::accounts::StoreChoice`).
use super::{SecretStore, StoreKind};
use crate::{Error, Result, secret::Secret};

pub struct OsStore {
    service: String,
}
impl OsStore {
    /// Items under `service` (the game uses "chiacraft wallet").
    pub fn new(service: &str) -> Self {
        Self {
            service: service.to_owned(),
        }
    }
}

impl SecretStore for OsStore {
    fn kind(&self) -> StoreKind {
        StoreKind::Os
    }
    fn put(&self, account: u32, entropy: &Secret<Vec<u8>>) -> Result<()> {
        imp::put(&self.service, &account.to_string(), entropy.expose())
    }
    fn get(&self, account: u32) -> Result<Option<Secret<Vec<u8>>>> {
        imp::get(&self.service, &account.to_string())
    }
    fn delete(&self, account: u32) -> Result<()> {
        imp::delete(&self.service, &account.to_string())
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::*;

    fn entry(service: &str, account: &str) -> Result<keyring::Entry> {
        keyring::Entry::new(service, account).map_err(|e| Error::new(format!("keychain: {e}")))
    }
    pub fn put(service: &str, account: &str, secret: &[u8]) -> Result<()> {
        entry(service, account)?
            .set_secret(secret)
            .map_err(|e| Error::new(format!("keychain: {e}")))
    }
    pub fn get(service: &str, account: &str) -> Result<Option<Secret<Vec<u8>>>> {
        match entry(service, account)?.get_secret() {
            Ok(s) => Ok(Some(Secret::new(s))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(Error::new(format!("keychain: {e}"))),
        }
    }
    pub fn delete(service: &str, account: &str) -> Result<()> {
        match entry(service, account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(Error::new(format!("keychain: {e}"))),
        }
    }
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
mod imp {
    use super::*;
    use std::ptr::null_mut;
    use windows_sys::Win32::{
        Foundation::{ERROR_NOT_FOUND, FILETIME, GetLastError},
        Security::Credentials::{
            CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CREDENTIALW, CredDeleteW, CredFree,
            CredReadW, CredWriteW,
        },
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }
    fn target(service: &str, account: &str) -> Vec<u16> {
        wide(&format!("{service}/{account}"))
    }
    fn last_error(what: &str) -> Error {
        // SAFETY: reads the calling thread's last-error value.
        Error::new(format!("Credential Manager {what}: error {}", unsafe {
            GetLastError()
        }))
    }
    pub fn put(service: &str, account: &str, secret: &[u8]) -> Result<()> {
        let target = target(service, account);
        let mut user = wide(account);
        let mut blob = secret.to_vec();
        let credential = CREDENTIALW {
            Flags: 0,
            Type: CRED_TYPE_GENERIC,
            TargetName: target.as_ptr() as *mut u16,
            Comment: null_mut(),
            LastWritten: FILETIME {
                dwLowDateTime: 0,
                dwHighDateTime: 0,
            },
            CredentialBlobSize: blob.len() as u32,
            CredentialBlob: blob.as_mut_ptr(),
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            AttributeCount: 0,
            Attributes: null_mut(),
            TargetAlias: null_mut(),
            UserName: user.as_mut_ptr(),
        };
        // SAFETY: every pointer in `credential` points into a live, NUL-terminated buffer (or is
        // null where allowed) for the duration of the call; CredWriteW copies what it keeps.
        let ok = unsafe { CredWriteW(&credential, 0) } != 0;
        zeroize::Zeroize::zeroize(&mut blob);
        if ok { Ok(()) } else { Err(last_error("write")) }
    }
    pub fn get(service: &str, account: &str) -> Result<Option<Secret<Vec<u8>>>> {
        let target = target(service, account);
        let mut found: *mut CREDENTIALW = null_mut();
        // SAFETY: `target` is NUL-terminated; on success `found` points to a credential the
        // system allocated, which is freed below with CredFree.
        if unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut found) } == 0 {
            // SAFETY: as in `last_error`.
            if unsafe { GetLastError() } == ERROR_NOT_FOUND {
                return Ok(None);
            }
            return Err(last_error("read"));
        }
        // SAFETY: `found` is valid until CredFree; the blob is `CredentialBlobSize` bytes.
        let secret = unsafe {
            let c = &*found;
            let bytes = std::slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize)
                .to_vec();
            CredFree(found as *const _);
            bytes
        };
        Ok(Some(Secret::new(secret)))
    }
    pub fn delete(service: &str, account: &str) -> Result<()> {
        let target = target(service, account);
        // SAFETY: `target` is NUL-terminated.
        if unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) } == 0 {
            // SAFETY: as in `last_error`.
            if unsafe { GetLastError() } == ERROR_NOT_FOUND {
                return Ok(());
            }
            return Err(last_error("delete"));
        }
        Ok(())
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod imp {
    use super::*;

    fn attributes<'a>(service: &'a str, account: &'a str) -> Vec<(&'a str, &'a str)> {
        vec![("service", service), ("account", account)]
    }
    fn keyring() -> Result<oo7::Keyring> {
        futures_lite::future::block_on(async {
            let keyring = oo7::Keyring::new().await?;
            keyring.unlock().await?;
            Ok::<_, oo7::Error>(keyring)
        })
        .map_err(|e| Error::new(format!("Secret Service: {e}")))
    }
    pub fn put(service: &str, account: &str, secret: &[u8]) -> Result<()> {
        let k = keyring()?;
        futures_lite::future::block_on(k.create_item(
            &format!("{service} {account}"),
            &attributes(service, account),
            secret,
            true,
        ))
        .map_err(|e| Error::new(format!("Secret Service: {e}")))
    }
    pub fn get(service: &str, account: &str) -> Result<Option<Secret<Vec<u8>>>> {
        let k = keyring()?;
        let items = futures_lite::future::block_on(k.search_items(&attributes(service, account)))
            .map_err(|e| Error::new(format!("Secret Service: {e}")))?;
        let Some(item) = items.into_iter().next() else {
            return Ok(None);
        };
        let secret = futures_lite::future::block_on(async {
            item.unlock().await?;
            item.secret().await
        })
        .map_err(|e| Error::new(format!("Secret Service: {e}")))?;
        Ok(Some(Secret::new(secret.to_vec())))
    }
    pub fn delete(service: &str, account: &str) -> Result<()> {
        let k = keyring()?;
        futures_lite::future::block_on(k.delete(&attributes(service, account)))
            .map_err(|e| Error::new(format!("Secret Service: {e}")))
    }
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
)))]
mod imp {
    use super::*;
    pub fn put(_: &str, _: &str, _: &[u8]) -> Result<()> {
        Err(Error::new("no OS keystore on this system"))
    }
    pub fn get(_: &str, _: &str) -> Result<Option<Secret<Vec<u8>>>> {
        Ok(None)
    }
    pub fn delete(_: &str, _: &str) -> Result<()> {
        Ok(())
    }
}
