//! The player's account in the game (plan phase 1).
//!
//! The account is the wallet's 24 words (`chiacraft-wallet`). The first start makes one quietly,
//! so play never waits; the game asks the player to write the words down later. From the words
//! come the identity key (what worlds know the player by, and what links to a DID) and, one-way
//! from it, the **network key**: the key of the game's peer-to-peer endpoint. Weft made a random
//! key per install; this one follows the words, so restoring them on another computer brings
//! back the same address and the same player on every world.
//!
//! Where things are:
//! - `accounts.json` (public facts only) in the account home: `<run dir>/account`, or
//!   `CHIACRAFT_ACCOUNT_HOME`;
//! - the secret in the OS keystore (macOS Keychain, Windows Credential Manager, Linux Secret
//!   Service), or in the account home's encrypted file store for automated runs and machines
//!   without a keystore ([`use_file_store`], or `HORIZON_WALLET_STORE=file`).
//!
//! Nothing here touches the network or the chain.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use chiacraft_wallet::accounts::env_passphrase;
use chiacraft_wallet::sign::network_link_text;
use chiacraft_wallet::{AccountInfo, Accounts, IdentitySigner, Mnemonic, Secret, Signer, StoreChoice};

/// The OS keystore's service name for chiacraft's accounts.
pub const SERVICE: &str = "chiacraft";
/// The account a first start makes is called this until the player renames it.
pub const DEFAULT_NAME: &str = "Player";

/// The open account: its public facts and its identity signer.
pub struct Session {
    pub home: PathBuf,
    choice: StoreChoice,
    pub info: AccountInfo,
    pub signer: Arc<IdentitySigner>,
    network: Secret<[u8; 32]>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Session({}, {})", self.info.fingerprint, self.info.identity)
    }
}

impl Session {
    /// The seed of the network key (Ed25519). Only for building the endpoint; never log it.
    pub fn network_seed(&self) -> &[u8; 32] {
        self.network.expose()
    }

    /// The identity key, hex: the player's identity on every world.
    pub fn identity_hex(&self) -> String {
        self.signer.identity().hex()
    }

    /// The identity key vouching for this session's network key (`endpoint`, as the network
    /// writes the public key): sent to worlds so they learn the identity behind the endpoint.
    pub fn vouch(&self, endpoint: &str) -> Result<[u8; 96], String> {
        self.signer.sign(&network_link_text(endpoint)).map_err(|e| e.to_string())
    }

    /// What a joining game sends a world beside its network key: `<identity hex>:<signature hex>`,
    /// the identity key vouching for `endpoint` (the network key as the network writes it).
    pub fn vouch_line(&self, endpoint: &str) -> Option<String> {
        match self.vouch(endpoint) {
            Ok(sig) => Some(format!("{}:{}", self.identity_hex(), chiacraft_wallet::keys::hex(&sig))),
            Err(e) => {
                log::warn!("vouching for the network key: {e}");
                None
            }
        }
    }

    fn accounts(&self) -> Result<Accounts, String> {
        Accounts::load(&self.home, self.choice.clone()).map_err(|e| format!("reading the accounts: {e}"))
    }

    /// The open account's facts as `accounts.json` has them now (the backup state changes while
    /// the game runs).
    pub fn fresh_info(&self) -> AccountInfo {
        self.accounts().ok().and_then(|a| a.get(self.info.fingerprint).cloned()).unwrap_or_else(|| self.info.clone())
    }

    /// Whether the player still has to write the words down.
    pub fn needs_backup(&self) -> bool {
        !self.fresh_info().backed_up
    }

    /// The 24 words, for the backup screen only: shown on screen, never copied, logged or sent.
    pub fn words(&self) -> Result<Secret<Vec<String>>, String> {
        let mnemonic = self.accounts()?.reveal(self.info.fingerprint, env_passphrase()).map_err(|e| format!("reading the words: {e}"))?;
        Ok(mnemonic.words())
    }

    /// The player confirmed writing the words down.
    pub fn mark_backed_up(&self) -> Result<(), String> {
        self.accounts()?.mark_backed_up(self.info.fingerprint).map_err(|e| e.to_string())
    }

    /// A play session ended (the backup reminder waits for the first one).
    pub fn session_ended(&self) {
        if let Err(e) = self.accounts().and_then(|mut a| a.session_ended(self.info.fingerprint).map_err(|e| e.to_string())) {
            log::warn!("counting the play session: {e}");
        }
    }

    /// Whether to remind the player to write the words down (after their first session).
    pub fn wants_backup(&self) -> bool {
        self.fresh_info().wants_backup()
    }

    /// Restores an account from words typed by the player and makes it the default; it is the
    /// account from the next start on (this run keeps its network key). Returns its fingerprint.
    pub fn restore(&self, words: &str) -> Result<u32, String> {
        let mnemonic = Mnemonic::parse(words).map_err(|p| p.to_string())?;
        let mut accounts = self.accounts()?;
        let info = accounts.restore(&mnemonic, DEFAULT_NAME).map_err(|e| e.to_string())?;
        accounts.set_default(info.fingerprint).map_err(|e| e.to_string())?;
        log::info!("restored account {}; it is used from the next start", info.fingerprint);
        Ok(info.fingerprint)
    }
}

/// A world's check of a joining player's [`Session::vouch_line`] against the network key it
/// authenticated (`endpoint`): the identity key, hex, when the line is that key's valid vouching.
pub fn check_vouch(line: &str, endpoint: &str) -> Option<String> {
    let (identity, sig) = line.split_once(':')?;
    let key = chiacraft_wallet::IdentityKey::from_hex(identity)?;
    let sig = chiacraft_wallet::keys::from_hex(sig)?;
    chiacraft_wallet::sign::verify_network_link(&key, endpoint, &sig).then(|| key.hex())
}

static SESSION: OnceLock<Result<Session, String>> = OnceLock::new();
static OS_KEYSTORE: AtomicBool = AtomicBool::new(true);

/// Keep this process's secrets in the account home's file store, never the OS keystore. Automated
/// runs call it before anything opens the account, so tests never reach the real keystore.
pub fn use_file_store() {
    OS_KEYSTORE.store(false, Ordering::Relaxed);
}

/// The account home for a run directory.
pub fn home(run_dir: &Path) -> PathBuf {
    std::env::var_os("CHIACRAFT_ACCOUNT_HOME").map(PathBuf::from).unwrap_or_else(|| run_dir.join("account"))
}

/// Opens the account once per process (making it the first time) and keeps it open. Later calls
/// return the same session, or the same error: a key that cannot be read is never replaced by a
/// new one, because a new key would be a new player everywhere.
pub fn open(run_dir: &Path) -> Result<&'static Session, String> {
    SESSION.get_or_init(|| start(run_dir)).as_ref().map_err(Clone::clone)
}

/// The open account, if [`open`] succeeded.
pub fn current() -> Option<&'static Session> {
    SESSION.get().and_then(|s| s.as_ref().ok())
}

fn start(run_dir: &Path) -> Result<Session, String> {
    let home = home(run_dir);
    let file_only = !OS_KEYSTORE.load(Ordering::Relaxed);
    let choice = StoreChoice::for_home(&home, SERVICE, file_only);
    let mut accounts = Accounts::load(&home, choice.clone()).map_err(|e| format!("opening the account in {}: {e}", home.display()))?;
    let made = accounts.current().is_none();
    let info = accounts.ensure_default(DEFAULT_NAME).map_err(|e| format!("making an account: {e}"))?;
    if made {
        log::info!("made a new account ({}), kept in the {} store", info.fingerprint, info.store);
    }
    let unlocked = accounts.unlock(info.fingerprint, env_passphrase()).map_err(|e| {
        if e.is_wrong_passphrase() {
            "the account's file needs its passphrase (HORIZON_WALLET_PASSPHRASE)".to_string()
        } else {
            format!("unlocking the account: {e}")
        }
    })?;
    let network = unlocked.signer.network_seed();
    log::info!("account {} open, identity {}", unlocked.info.fingerprint, unlocked.info.identity);
    Ok(Session { home, choice, info: unlocked.info, signer: unlocked.signer, network })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh run directory: a first start makes an account in its file store, and a second
    /// opening of the same folder (the next start) finds the same account and network key.
    #[test]
    fn first_start_makes_an_account_and_the_next_start_finds_it() {
        let dir = std::env::temp_dir().join(format!("chiacraft-account-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let first = start_file(&dir).expect("a new account");
        let second = start_file(&dir).expect("the same account");
        assert_eq!(first.info.fingerprint, second.info.fingerprint);
        assert_eq!(first.network_seed(), second.network_seed());
        assert_eq!(first.identity_hex(), first.info.identity);
        assert!(!format!("{first:?}").contains(&hex(first.network_seed())));
        let sig = first.vouch("abc").unwrap();
        assert!(chiacraft_wallet::sign::verify_network_link(&first.signer.identity(), "abc", &sig));
        let line = first.vouch_line("abc").unwrap();
        assert_eq!(check_vouch(&line, "abc"), Some(first.identity_hex()));
        assert_eq!(check_vouch(&line, "abd"), None);
        assert_eq!(check_vouch("nonsense", "abc"), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn start_file(dir: &Path) -> Result<Session, String> {
        use_file_store();
        start(dir)
    }

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }
}
