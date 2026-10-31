//! chiacraft's built-in Chia wallet core (identity-research.md, plan row I1).
//!
//! - The account is a **24-word mnemonic** ([`Mnemonic`]); from it come Chia's ordinary wallet
//!   keys (purpose 2, what Sage finds after importing the same words) and the player's
//!   **identity keys** on their own hardened branch ([`keys::IDENTITY_PURPOSE`]).
//! - Secrets stay on the device: the mnemonic's entropy is kept in the OS keystore or a
//!   passphrase-encrypted file ([`store`]); every secret travels in [`Secret`], which redacts
//!   itself and wipes its memory.
//! - The game signs through a narrow interface ([`sign::Signer`]): an identity key signs only
//!   chiacraft's own texts, as Chia signed messages (CHIP-0002).
//! - **No network** without the `chain` feature: nothing here opens a connection. With it,
//!   [`chain`] is the chain reader (I6): peer sync, DIDs, coin watches, offers, a cache; it
//!   starts only when asked.
//!
//! Ported from Ronald's wallet (`ronickg/chia`, `packages/core`: `wallet.ts`, `state.ts`, the
//! signing helper in `dapp.ts`) onto the crates chia-wallet-sdk 0.37.1 is built from; the file
//! store follows Sage's `sage-keychain`. Both are Apache-2.0 (see NOTICE); this crate is MIT.
//! - **Achievements** ([`achievement`], identity row I10): credentials a server signs with its
//!   identity key and the player keeps, verified offline by anyone.
//!
//! It has no dependency on the game, so a desktop wallet can use it as is.
#![deny(unsafe_code)]
pub mod accounts;
pub mod achievement;
#[cfg(feature = "chain")]
pub mod chain;
pub mod keys;
pub mod mnemonic;
pub mod secret;
pub mod sign;
pub mod spending;
pub mod sponsor;
pub mod store;

pub use accounts::{AccountInfo, Accounts, DidLink, StoreChoice, Unlocked};
pub use keys::{IdentityKey, Network, PublicFacts};
pub use mnemonic::{Mnemonic, WordsProblem};
pub use secret::{Passphrase, Secret};
pub use sign::{IdentitySigner, OwnerSigner, Signer};
pub use store::StoreKind;

/// What went wrong. Its text never holds a secret.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    message: String,
    wrong_passphrase: bool,
}
impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            wrong_passphrase: false,
        }
    }
    pub fn wrong_passphrase() -> Self {
        Self {
            message: "wrong passphrase".into(),
            wrong_passphrase: true,
        }
    }
    pub fn is_wrong_passphrase(&self) -> bool {
        self.wrong_passphrase
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::new(e.to_string())
    }
}
pub type Result<T> = std::result::Result<T, Error>;
