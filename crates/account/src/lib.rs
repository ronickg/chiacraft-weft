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
//! - `accounts.json` (public facts only) in the account home: `<run dir>/account` (automated runs:
//!   `<run dir>/test-account`, beside the player's), or `CHIACRAFT_ACCOUNT_HOME`;
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
use chiacraft_wallet::{AccountInfo, Accounts, DidLink, IdentitySigner, Mnemonic, OwnerSigner, Secret, Signer, StoreChoice};

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
    /// The key that holds the linked DID (phase 4), derived when the account opens or a DID is
    /// linked; it signs the join's DID proof and nothing else.
    owner: parking_lot::Mutex<Option<Arc<OwnerSigner>>>,
    /// The hardened wallet addresses' puzzle hashes (Sage derives both branches), for the chain.
    pub hardened: Vec<[u8; 32]>,
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

    /// The seed of the content key (Ed25519), which signs the packages this player makes. Only for
    /// signing; never log it.
    pub fn content_seed(&self) -> [u8; 32] {
        *self.signer.content_seed().expose()
    }

    /// The identity key vouching for the content key `key` (its public half, hex), as
    /// `<identity hex>:<signature hex>`.
    pub fn content_vouch_line(&self, key: &str) -> Result<String, String> {
        let sig = self
            .signer
            .sign(&chiacraft_wallet::sign::content_link_text(key))
            .map_err(|e| e.to_string())?;
        Ok(format!("{}:{}", self.identity_hex(), chiacraft_wallet::keys::hex(&sig)))
    }

    /// The seed of a world's key (Ed25519): see `IdentitySigner::world_seed`. Only for building the
    /// world's endpoint; never log it.
    pub fn world_seed(&self, index: u32, generation: u32) -> [u8; 32] {
        *self.signer.world_seed(index, generation).expose()
    }

    /// The identity key certifying the world `world` (its key as the network writes it), as
    /// `<identity hex>:<signature hex>`.
    pub fn world_cert_line(&self, world: &str, generation: u32, created: u64) -> Result<String, String> {
        let sig = self
            .signer
            .sign(&chiacraft_wallet::sign::world_cert_text(world, generation, created))
            .map_err(|e| e.to_string())?;
        Ok(format!("{}:{}", self.identity_hex(), chiacraft_wallet::keys::hex(&sig)))
    }

    /// The identity key saying its world `from` moved to `to` (generation `generation`), as a
    /// signature hex.
    pub fn world_moved_sig(&self, from: &str, to: &str, generation: u32) -> Result<String, String> {
        let sig = self
            .signer
            .sign(&chiacraft_wallet::sign::world_moved_text(from, to, generation))
            .map_err(|e| e.to_string())?;
        Ok(chiacraft_wallet::keys::hex(&sig))
    }

    /// The identity key's signature of one of chiacraft's own texts (`text` must start with
    /// `chiacraft `), hex: shared ban lists and other documents a player publishes.
    pub fn sign_text(&self, text: &str) -> Result<String, String> {
        let sig = self.signer.sign(text).map_err(|e| e.to_string())?;
        Ok(chiacraft_wallet::keys::hex(&sig))
    }

    /// The identity key, hex: the player's identity on every world.
    pub fn identity_hex(&self) -> String {
        self.signer.identity().hex()
    }

    /// The identity key vouching for this session's network key (`endpoint`, as the network
    /// writes the public key): sent to worlds so they learn the identity behind the endpoint.
    pub fn vouch(&self, endpoint: &str) -> Result<[u8; 96], String> {
        self.signer
            .sign(&network_link_text(endpoint))
            .map_err(|e| e.to_string())
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

    /// The linked DID, if any.
    pub fn did(&self) -> Option<DidLink> {
        self.fresh_info().did
    }

    /// The puzzle hash of the account's first wallet address: where a sponsor launches its DID
    /// (what Sage shows first after importing the words).
    pub fn first_puzzle_hash(&self) -> Option<[u8; 32]> {
        chiacraft_wallet::keys::wallet_puzzle_hash(&self.info.master_key()?, 0).ok()
    }

    /// The puzzle hashes of the account's first `count` wallet addresses, unhardened then
    /// hardened: what the chain reader follows.
    pub fn puzzle_hashes(&self, count: u32) -> Vec<[u8; 32]> {
        let mut out = self
            .info
            .master_key()
            .and_then(|m| chiacraft_wallet::keys::wallet_puzzle_hashes(&m, count).ok())
            .unwrap_or_default();
        out.extend(self.hardened.iter().take(count as usize));
        out
    }

    /// A request to a DID sponsor for a DID at `address` (the account's first, on `network`),
    /// signed by the identity key now.
    pub fn sponsor_request(&self, network: &str, address: &str) -> Result<chiacraft_wallet::sponsor::Request, String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        chiacraft_wallet::sponsor::Request::sign(&*self.signer, network, address, now).map_err(|e| e.to_string())
    }

    /// Link the DID `launcher` held at the wallet address whose puzzle hash is `owner` (one of
    /// this account's first 20, either branch): the account remembers it and signs join proofs
    /// with that address's key from now on.
    pub fn link_did(&self, launcher: [u8; 32], owner: [u8; 32]) -> Result<DidLink, String> {
        let mnemonic = self
            .accounts()?
            .reveal(self.info.fingerprint, env_passphrase())
            .map_err(|e| format!("reading the words: {e}"))?;
        let signer = OwnerSigner::find(&mnemonic, owner, 20).ok_or("that DID isn't held by one of this account's addresses")?;
        let link = DidLink { launcher: chiacraft_wallet::keys::hex(&launcher), index: signer.index(), hardened: signer.hardened() };
        self.accounts()?
            .set_did(self.info.fingerprint, Some(link.clone()))
            .map_err(|e| e.to_string())?;
        *self.owner.lock() = Some(Arc::new(signer));
        log::info!("DID {} linked to account {}", short_hex(&link.launcher), self.info.fingerprint);
        Ok(link)
    }

    /// Forget the linked DID (it stays on chain, held by the same address).
    pub fn unlink_did(&self) -> Result<(), String> {
        self.accounts()?
            .set_did(self.info.fingerprint, None)
            .map_err(|e| e.to_string())?;
        *self.owner.lock() = None;
        Ok(())
    }

    /// The join's DID proof: `<launcher hex>:<owner key hex>:<signature hex>`, the owner key
    /// signing [`chiacraft_wallet::sign::did_link_text`]. None without a linked DID.
    pub fn did_proof(&self) -> Option<String> {
        let did = self.did()?;
        let owner = self.owner.lock().clone()?;
        let sig = owner
            .sign(&chiacraft_wallet::sign::did_link_text(&did.launcher, &self.identity_hex()))
            .ok()?;
        Some(format!("{}:{}:{}", did.launcher, chiacraft_wallet::keys::hex(&owner.public()), chiacraft_wallet::keys::hex(&sig)))
    }

    fn accounts(&self) -> Result<Accounts, String> {
        Accounts::load(&self.home, self.choice.clone()).map_err(|e| format!("reading the accounts: {e}"))
    }

    /// The open account's facts as `accounts.json` has them now (the backup state changes while
    /// the game runs).
    pub fn fresh_info(&self) -> AccountInfo {
        self.accounts()
            .ok()
            .and_then(|a| a.get(self.info.fingerprint).cloned())
            .unwrap_or_else(|| self.info.clone())
    }

    /// Whether the player still has to write the words down.
    pub fn needs_backup(&self) -> bool {
        !self.fresh_info().backed_up
    }

    /// The 24 words, for the backup screen only: shown on screen, never copied, logged or sent.
    pub fn words(&self) -> Result<Secret<Vec<String>>, String> {
        let mnemonic = self
            .accounts()?
            .reveal(self.info.fingerprint, env_passphrase())
            .map_err(|e| format!("reading the words: {e}"))?;
        Ok(mnemonic.words())
    }

    /// The player confirmed writing the words down.
    pub fn mark_backed_up(&self) -> Result<(), String> {
        self.accounts()?
            .mark_backed_up(self.info.fingerprint)
            .map_err(|e| e.to_string())
    }

    /// A play session ended (the backup reminder waits for the first one).
    pub fn session_ended(&self) {
        if let Err(e) = self.accounts().and_then(|mut a| {
            a.session_ended(self.info.fingerprint)
                .map_err(|e| e.to_string())
        }) {
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
        // Words of an account this computer already lists (going back to an earlier one) switch to it.
        let mut fingerprint = chiacraft_wallet::keys::facts_of(&mnemonic).fingerprint;
        if accounts.get(fingerprint).is_none() {
            fingerprint = accounts
                .restore(&mnemonic, DEFAULT_NAME)
                .map_err(|e| e.to_string())?
                .fingerprint;
        }
        accounts
            .set_default(fingerprint)
            .map_err(|e| e.to_string())?;
        log::info!("restored account {fingerprint}; it is used from the next start");
        Ok(fingerprint)
    }
}

/// A world's check of a joining player's [`Session::vouch_line`] against the network key it
/// authenticated (`endpoint`): the identity key, hex, when the line is that key's valid vouching.
/// The identity behind a content key: `line` is `<identity hex>:<signature hex>` vouching for
/// `key` (hex). Gives the identity (hex) when it checks out.
pub fn check_content_vouch(line: &str, key: &str) -> Option<String> {
    let (identity, sig) = line.split_once(':')?;
    let id = chiacraft_wallet::IdentityKey::from_hex(identity)?;
    let sig = chiacraft_wallet::keys::from_hex(sig)?;
    chiacraft_wallet::sign::verify_content_link(&id, key, &sig).then(|| id.hex())
}

/// The owner (identity hex) if `line` (`<identity hex>:<signature hex>`) certifies the world.
pub fn check_world_cert(line: &str, world: &str, generation: u32, created: u64) -> Option<String> {
    let (identity, sig) = line.split_once(':')?;
    let id = chiacraft_wallet::IdentityKey::from_hex(identity)?;
    let sig = chiacraft_wallet::keys::from_hex(sig)?;
    chiacraft_wallet::sign::verify_world_cert(&id, world, generation, created, &sig).then(|| id.hex())
}

/// Whether `identity` (hex) signed (`sig`, hex) that its world `from` moved to `to`.
pub fn check_world_moved(identity: &str, from: &str, to: &str, generation: u32, sig: &str) -> bool {
    let (Some(id), Some(sig)) = (chiacraft_wallet::IdentityKey::from_hex(identity), chiacraft_wallet::keys::from_hex(sig)) else {
        return false;
    };
    chiacraft_wallet::sign::verify_world_moved(&id, from, to, generation, &sig)
}

/// Whether `sig` (hex) is the identity `identity`'s (hex) signature of `text` ([`Session::sign_text`]).
pub fn check_text(identity: &str, text: &str, sig: &str) -> bool {
    let (Some(id), Some(sig)) = (chiacraft_wallet::IdentityKey::from_hex(identity), chiacraft_wallet::keys::from_hex(sig)) else {
        return false;
    };
    chiacraft_wallet::sign::verify(&id, text.as_bytes(), &sig)
}

/// A world's check of a joining player's [`Session::did_proof`] for `identity` (hex): the DID's
/// launcher id and the puzzle hash of the address whose key signed, when the signature holds.
/// Whether that address holds the DID now is for the chain to say.
pub fn check_did_proof(line: &str, identity: &str) -> Option<([u8; 32], [u8; 32])> {
    let mut parts = line.split(':');
    let (launcher, owner, sig) = (parts.next()?, parts.next()?, parts.next()?);
    let launcher_bytes: [u8; 32] = chiacraft_wallet::keys::from_hex(launcher)?.try_into().ok()?;
    let owner_key: [u8; 48] = chiacraft_wallet::keys::from_hex(owner)?.try_into().ok()?;
    let sig = chiacraft_wallet::keys::from_hex(sig)?;
    let text = chiacraft_wallet::sign::did_link_text(&chiacraft_wallet::keys::hex(&launcher_bytes), identity);
    if !chiacraft_wallet::sign::verify_owner(&owner_key, text.as_bytes(), &sig) {
        return None;
    }
    Some((launcher_bytes, chiacraft_wallet::keys::p2_of_synthetic(&owner_key)?))
}

fn short_hex(h: &str) -> String {
    h.chars().take(12).collect()
}

pub fn check_vouch(line: &str, endpoint: &str) -> Option<String> {
    let (identity, sig) = line.split_once(':')?;
    let key = chiacraft_wallet::IdentityKey::from_hex(identity)?;
    let sig = chiacraft_wallet::keys::from_hex(sig)?;
    chiacraft_wallet::sign::verify_network_link(&key, endpoint, &sig).then(|| key.hex())
}

static SESSION: OnceLock<Result<Session, String>> = OnceLock::new();
static OS_KEYSTORE: AtomicBool = AtomicBool::new(true);

/// Keep this process's secrets in the account home's file store, never the OS keystore, and that
/// home apart from the player's. Automated runs call it before anything opens the account, so
/// tests never reach the real keystore or the player's account.
pub fn use_file_store() {
    OS_KEYSTORE.store(false, Ordering::Relaxed);
}

/// The account home for a run directory.
pub fn home(run_dir: &Path) -> PathBuf {
    let dir = if OS_KEYSTORE.load(Ordering::Relaxed) { "account" } else { "test-account" };
    std::env::var_os("CHIACRAFT_ACCOUNT_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| run_dir.join(dir))
}

/// Opens the account once per process (making it the first time) and keeps it open. Later calls
/// return the same session, or the same error: a key that cannot be read is never replaced by a
/// new one, because a new key would be a new player everywhere.
pub fn open(run_dir: &Path) -> Result<&'static Session, String> {
    SESSION
        .get_or_init(|| start(run_dir))
        .as_ref()
        .map_err(Clone::clone)
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
    let info = accounts
        .ensure_default(DEFAULT_NAME)
        .map_err(|e| format!("making an account: {e}"))?;
    if made {
        log::info!("made a new account ({}), kept in the {} store", info.fingerprint, info.store);
    }
    let unlocked = accounts
        .unlock(info.fingerprint, env_passphrase())
        .map_err(|e| {
            if e.is_wrong_passphrase() {
                "the account's file needs its passphrase (HORIZON_WALLET_PASSPHRASE)".to_string()
            } else {
                format!("unlocking the account: {e}")
            }
        })?;
    let network = unlocked.signer.network_seed();
    log::info!("account {} open, identity {}", unlocked.info.fingerprint, unlocked.info.identity);
    Ok(Session {
        home,
        choice,
        info: unlocked.info,
        signer: unlocked.signer,
        network,
        owner: parking_lot::Mutex::new(unlocked.did),
        hardened: unlocked.hardened,
    })
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
        let sig = first.sign_text("chiacraft ban list v1\nx").unwrap();
        assert!(check_text(&first.identity_hex(), "chiacraft ban list v1\nx", &sig));
        assert!(!check_text(&first.identity_hex(), "chiacraft ban list v1\ny", &sig));
        assert!(first.sign_text("not ours").is_err());
        // A DID held at the account's first address: linked, and the proof checks out for this
        // identity only.
        let owner = first.first_puzzle_hash().unwrap();
        let launcher = [7u8; 32];
        assert!(first.did_proof().is_none());
        first.link_did(launcher, owner).unwrap();
        let proof = first.did_proof().unwrap();
        assert_eq!(check_did_proof(&proof, &first.identity_hex()), Some((launcher, owner)));
        assert_eq!(check_did_proof(&proof, &"ab".repeat(48)), None);
        assert!(first.link_did(launcher, [1u8; 32]).is_err());
        first.unlink_did().unwrap();
        assert!(first.did_proof().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Restoring words makes their account the next start's, also when this computer lists it
    /// already (going back to an earlier account), and nonsense words change nothing.
    #[test]
    fn restore_switches_accounts_and_back() {
        let dir = std::env::temp_dir().join(format!("chiacraft-account-restore-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let first = start_file(&dir).expect("a new account");
        let words = first.words().unwrap().expose().join(" ");
        let other = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
        assert!(first.restore("these are not the words").is_err());
        assert_eq!(start_file(&dir).unwrap().info.fingerprint, first.info.fingerprint);
        let restored = first.restore(other).expect("restoring other words");
        assert_ne!(restored, first.info.fingerprint);
        assert_eq!(start_file(&dir).unwrap().info.fingerprint, restored);
        assert_eq!(
            first
                .restore(&words)
                .expect("going back to the first account"),
            first.info.fingerprint
        );
        let back = start_file(&dir).unwrap();
        assert_eq!(back.info.fingerprint, first.info.fingerprint);
        assert_eq!(back.network_seed(), first.network_seed());
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
