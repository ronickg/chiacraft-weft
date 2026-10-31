//! Where an account's secret lives (identity-research.md 1.3): only the mnemonic's entropy is
//! stored, one item per account (by its Chia fingerprint), in the OS keystore ([`os`]) or a
//! passphrase-encrypted file ([`file`]). Public keys and names live in `accounts.json`.
pub mod file;
pub mod memory;
pub mod os;

use crate::{Result, secret::Secret};

/// Which store holds an account's secret.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreKind {
    /// The OS keystore: macOS Keychain, Windows Credential Manager (this machine only), the
    /// Secret Service on Linux.
    Os,
    /// `wallet/<fingerprint>.key` in the home, encrypted with a passphrase (empty: unprotected).
    File,
    /// Memory only (tests; a session that keeps nothing).
    Memory,
}
impl StoreKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Os => "os",
            Self::File => "file",
            Self::Memory => "memory",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "os" => Self::Os,
            "file" => Self::File,
            "memory" => Self::Memory,
            _ => return None,
        })
    }
}

/// A place that keeps accounts' entropy.
pub trait SecretStore: Send + Sync {
    fn kind(&self) -> StoreKind;
    fn put(&self, account: u32, entropy: &Secret<Vec<u8>>) -> Result<()>;
    /// `Ok(None)` when the store has no item for the account.
    fn get(&self, account: u32) -> Result<Option<Secret<Vec<u8>>>>;
    fn delete(&self, account: u32) -> Result<()>;
}
