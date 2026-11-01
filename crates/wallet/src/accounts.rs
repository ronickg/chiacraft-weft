//! Accounts on this device (identity-research.md 1.4, 9.3): `accounts.json` in a home folder
//! holds each account's public facts (Chia fingerprint, master public key, identity key), its
//! name, where its secret is kept and how far its backup has gone; the secret itself is in a
//! [`SecretStore`]. Ported from `@chia/core` `state.ts` (several wallets, one active, the
//! mnemonic in the platform keystore and only metadata in plain storage); accounts are keyed by
//! the master key's fingerprint, as Sage keys them.
use crate::{
    Error, Mnemonic, Result,
    keys::{self, IdentityKey, PublicFacts},
    secret::{Passphrase, Secret},
    sign::IdentitySigner,
    store::{SecretStore, StoreKind, file::FileStore, memory::MemoryStore, os::OsStore},
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

/// The file in the home.
pub const FILE: &str = "accounts.json";
/// The file store's folder in the home.
pub const WALLET_DIR: &str = "wallet";

/// One account's public facts and state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountInfo {
    /// Chia's fingerprint of the master public key (what Sage shows for the same words).
    pub fingerprint: u32,
    pub name: String,
    /// The master public key, hex (wallet addresses derive from it with no secret).
    pub master: String,
    /// Identity key 0, hex: the player's identity on every server.
    pub identity: String,
    /// Unix seconds.
    pub created: i64,
    /// Where the secret is: `os`, `file` or `memory`.
    pub store: String,
    /// The player has seen the words and confirmed writing them down.
    #[serde(default)]
    pub backed_up: bool,
    /// Made from words the player typed (they have them already; import from Sage).
    #[serde(default)]
    pub restored: bool,
    /// Play sessions finished with this account (the backup prompt waits for the first).
    #[serde(default)]
    pub sessions: u32,
    /// A DID this account owns and shows servers (identity row I7: linked on the Account page;
    /// each sign-in then carries a proof by its owner key). Public facts only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub did: Option<DidLink>,
}

/// A linked DID: its launcher id and which wallet address owns it (the key that signs the proof).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DidLink {
    /// The launcher id, 64 hex digits.
    pub launcher: String,
    /// The owner's wallet address index (`m/12381/8444/2/index`).
    pub index: u32,
    /// Whether that address is on the hardened branch (Sage derives both).
    #[serde(default)]
    pub hardened: bool,
}
impl DidLink {
    /// The launcher id's bytes.
    pub fn launcher_id(&self) -> Option<[u8; 32]> {
        keys::from_hex(&self.launcher)?.try_into().ok()
    }
}
impl AccountInfo {
    pub fn identity_key(&self) -> Option<IdentityKey> {
        IdentityKey::from_hex(&self.identity)
    }
    pub fn master_key(&self) -> Option<[u8; 48]> {
        keys::from_hex(&self.master)?.try_into().ok()
    }
    pub fn store_kind(&self) -> StoreKind {
        StoreKind::parse(&self.store).unwrap_or(StoreKind::File)
    }
    /// The backup screen should be offered: after the first session, before the words were
    /// confirmed (identity-research.md 1.4: never at first start).
    pub fn wants_backup(&self) -> bool {
        !self.backed_up && !self.restored && self.sessions >= 1
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Doc {
    v: u32,
    default: Option<u32>,
    accounts: Vec<AccountInfo>,
}

/// Which store new accounts go to, and where the file store and the keystore items are.
#[derive(Clone, Debug)]
pub struct StoreChoice {
    pub kind: StoreKind,
    /// The OS keystore's service name.
    pub service: String,
    /// The file store's folder.
    pub file_dir: PathBuf,
}
impl StoreChoice {
    /// For a home folder: `HORIZON_WALLET_STORE` (`os`, `file`, `memory`) when set; else the OS
    /// keystore for the platform's default home, and the file store for a home that was given
    /// explicitly (`home_given`: checks run in `mktemp -d` homes and must never reach the real
    /// keystore).
    pub fn for_home(home: &Path, service: &str, home_given: bool) -> Self {
        let env = std::env::var("HORIZON_WALLET_STORE")
            .ok()
            .and_then(|s| StoreKind::parse(s.trim()));
        Self {
            kind: env.unwrap_or(if home_given { StoreKind::File } else { StoreKind::Os }),
            service: service.to_owned(),
            file_dir: home.join(WALLET_DIR),
        }
    }
    fn open(&self, kind: StoreKind, passphrase: Passphrase) -> Box<dyn SecretStore> {
        match kind {
            StoreKind::Os => Box::new(OsStore::new(&self.service)),
            StoreKind::File => Box::new(FileStore::new(&self.file_dir, passphrase)),
            StoreKind::Memory => Box::new(MemoryHandle),
        }
    }
}

/// The process's memory store (shared, so a memory account survives reloading the list).
fn memory() -> &'static MemoryStore {
    static STORE: OnceLock<MemoryStore> = OnceLock::new();
    STORE.get_or_init(MemoryStore::default)
}
struct MemoryHandle;
impl SecretStore for MemoryHandle {
    fn kind(&self) -> StoreKind {
        StoreKind::Memory
    }
    fn put(&self, account: u32, entropy: &Secret<Vec<u8>>) -> Result<()> {
        memory().put(account, entropy)
    }
    fn get(&self, account: u32) -> Result<Option<Secret<Vec<u8>>>> {
        memory().get(account)
    }
    fn delete(&self, account: u32) -> Result<()> {
        memory().delete(account)
    }
}

/// The passphrase checks give a file store (`HORIZON_WALLET_PASSPHRASE`), else none.
pub fn env_passphrase() -> Passphrase {
    Secret::new(std::env::var("HORIZON_WALLET_PASSPHRASE").unwrap_or_default())
}

/// An unlocked account: its facts and its identity signer, and the linked DID's owner signer.
pub struct Unlocked {
    pub info: AccountInfo,
    pub signer: Arc<IdentitySigner>,
    pub did: Option<Arc<crate::sign::OwnerSigner>>,
    /// The standard puzzle hashes of the first [`keys::HARDENED_ADDRESSES`] hardened wallet
    /// addresses (I8): public facts, but derivable only with the secret, so worked out here for
    /// the chain reader to follow beside the unhardened ones.
    pub hardened: Vec<[u8; 32]>,
}

/// The accounts of one home folder.
pub struct Accounts {
    path: PathBuf,
    pub choice: StoreChoice,
    pub default: Option<u32>,
    pub list: Vec<AccountInfo>,
}

impl Accounts {
    /// `accounts.json` in `home` (an empty list when there is none).
    pub fn load(home: &Path, choice: StoreChoice) -> Result<Self> {
        let path = home.join(FILE);
        let doc: Doc = match std::fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).map_err(|e| Error::new(format!("{}: {e}", path.display())))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Doc::default(),
            Err(e) => return Err(e.into()),
        };
        Ok(Self { path, choice, default: doc.default, list: doc.accounts })
    }
    pub fn save(&self) -> Result<()> {
        let doc = Doc { v: 1, default: self.default, accounts: self.list.clone() };
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = serde_json::to_string_pretty(&doc).map_err(|e| Error::new(e.to_string()))?;
        crate::store::file::write_private(&self.path, text.as_bytes())
    }
    pub fn get(&self, fingerprint: u32) -> Option<&AccountInfo> {
        self.list.iter().find(|a| a.fingerprint == fingerprint)
    }
    fn get_mut(&mut self, fingerprint: u32) -> Result<&mut AccountInfo> {
        self.list
            .iter_mut()
            .find(|a| a.fingerprint == fingerprint)
            .ok_or_else(|| Error::new("no such account"))
    }
    /// The default account (the first one when none is marked).
    pub fn current(&self) -> Option<&AccountInfo> {
        self.default
            .and_then(|f| self.get(f))
            .or_else(|| self.list.first())
    }

    /// Keep `mnemonic`'s entropy and list the account. The OS keystore falls back to the file
    /// store (no passphrase yet) when it isn't there.
    fn add(&mut self, mnemonic: &Mnemonic, name: &str, restored: bool) -> Result<AccountInfo> {
        let facts: PublicFacts = keys::facts_of(mnemonic);
        if self.get(facts.fingerprint).is_some() {
            return Err(Error::new("this account is already on this device"));
        }
        let entropy = mnemonic.entropy();
        let mut kind = self.choice.kind;
        if let Err(e) = self
            .choice
            .open(kind, env_passphrase())
            .put(facts.fingerprint, &entropy)
        {
            if kind != StoreKind::Os {
                return Err(e);
            }
            log::warn!("no OS keystore ({e}); keeping the account in the wallet folder");
            kind = StoreKind::File;
            self.choice
                .open(kind, env_passphrase())
                .put(facts.fingerprint, &entropy)?;
        }
        let info = AccountInfo {
            fingerprint: facts.fingerprint,
            name: name.to_owned(),
            master: keys::hex(&facts.master),
            identity: facts.identity.hex(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs() as i64),
            store: kind.name().to_owned(),
            backed_up: false,
            restored,
            sessions: 0,
            did: None,
        };
        self.list.push(info.clone());
        if self.default.is_none() {
            self.default = Some(info.fingerprint);
        }
        self.save()?;
        Ok(info)
    }
    /// A new account with fresh words. The words are returned for a create screen that shows
    /// them; the silent first-start account drops them (the backup screen reads them back later).
    pub fn create(&mut self, name: &str) -> Result<(AccountInfo, Mnemonic)> {
        let mnemonic = Mnemonic::generate()?;
        let info = self.add(&mnemonic, name, false)?;
        Ok((info, mnemonic))
    }
    /// A new account from words made elsewhere (checks' fixed test words): like [`Self::create`],
    /// so its backup is still offered.
    pub fn create_with(&mut self, mnemonic: &Mnemonic, name: &str) -> Result<AccountInfo> {
        self.add(mnemonic, name, false)
    }
    /// An account from words the player typed (restore, or import from Sage).
    pub fn restore(&mut self, mnemonic: &Mnemonic, name: &str) -> Result<AccountInfo> {
        self.add(mnemonic, name, true)
    }
    /// Make `mnemonic` this folder's account (a dedicated server given its words in the
    /// environment, I12): listed already, its secret is put back where it lives (a memory account
    /// loses it with each process) and it becomes the default; a folder holding only other
    /// accounts is refused (never two identities for one server); an empty folder lists it.
    pub fn provide(&mut self, mnemonic: &Mnemonic, name: &str) -> Result<AccountInfo> {
        let fingerprint = keys::facts_of(mnemonic).fingerprint;
        if let Some(info) = self.get(fingerprint).cloned() {
            if info.store_kind() == StoreKind::Memory {
                MemoryHandle.put(fingerprint, &mnemonic.entropy())?;
            }
            if self.default != Some(fingerprint) {
                self.set_default(fingerprint)?;
            }
            return Ok(info);
        }
        if let Some(other) = self.list.first() {
            return Err(Error::new(format!(
                "this folder already holds account {} and the words given are account {fingerprint}: use another folder, or move its accounts.json and wallet/ away first",
                other.fingerprint
            )));
        }
        self.add(mnemonic, name, true)
    }

    /// The default account, made silently when there is none (play never waits).
    pub fn ensure_default(&mut self, name: &str) -> Result<AccountInfo> {
        if let Some(a) = self.current() {
            return Ok(a.clone());
        }
        Ok(self.create(name)?.0)
    }

    /// Whether unlocking needs a passphrase (a protected file).
    pub fn needs_passphrase(&self, fingerprint: u32) -> bool {
        self.get(fingerprint)
            .is_some_and(|a| a.store_kind() == StoreKind::File)
            && FileStore::protected(&self.choice.file_dir, fingerprint) == Some(true)
    }
    /// Whether the account's secret is kept unencrypted on disk (a file with no passphrase).
    pub fn unprotected(&self, fingerprint: u32) -> bool {
        self.get(fingerprint)
            .is_some_and(|a| a.store_kind() == StoreKind::File)
            && FileStore::protected(&self.choice.file_dir, fingerprint) == Some(false)
    }

    fn mnemonic(&self, fingerprint: u32, passphrase: Passphrase) -> Result<Mnemonic> {
        let info = self
            .get(fingerprint)
            .ok_or_else(|| Error::new("no such account"))?;
        let entropy = self
            .choice
            .open(info.store_kind(), passphrase)
            .get(fingerprint)?
            .ok_or_else(|| Error::new("this account's secret is missing from its store"))?;
        let mnemonic = Mnemonic::from_entropy(entropy.expose())?;
        if keys::facts_of(&mnemonic).fingerprint != fingerprint {
            return Err(Error::new("the stored secret belongs to another account"));
        }
        Ok(mnemonic)
    }
    /// Read the secret and keep only identity key 0's signer.
    pub fn unlock(&self, fingerprint: u32, passphrase: Passphrase) -> Result<Unlocked> {
        let mnemonic = self.mnemonic(fingerprint, passphrase)?;
        let signer = Arc::new(IdentitySigner::from_mnemonic(&mnemonic, 0));
        let info = self.get(fingerprint).cloned().expect("checked");
        if info.identity_key() != Some(crate::sign::Signer::identity(&*signer)) {
            return Err(Error::new("accounts.json disagrees with the stored secret"));
        }
        let did = info
            .did
            .as_ref()
            .map(|d| Arc::new(crate::sign::OwnerSigner::from_mnemonic(&mnemonic, d.index, d.hardened)));
        let hardened = keys::hardened_puzzle_hashes(&mnemonic, keys::HARDENED_ADDRESSES);
        Ok(Unlocked { info, signer, did, hardened })
    }
    /// Link `did` to an account (None: unlink). Public facts only: the owner key is derived again
    /// when the account unlocks.
    pub fn set_did(&mut self, fingerprint: u32, did: Option<DidLink>) -> Result<()> {
        self.get_mut(fingerprint)?.did = did;
        self.save()
    }
    /// The words, for the backup screen.
    pub fn reveal(&self, fingerprint: u32, passphrase: Passphrase) -> Result<Mnemonic> {
        self.mnemonic(fingerprint, passphrase)
    }
    /// Re-encrypt a file account under a new passphrase (empty: unprotected).
    pub fn set_passphrase(&self, fingerprint: u32, old: Passphrase, new: Passphrase) -> Result<()> {
        let info = self
            .get(fingerprint)
            .ok_or_else(|| Error::new("no such account"))?;
        if info.store_kind() != StoreKind::File {
            return Err(Error::new("only an account kept in the wallet folder has a passphrase"));
        }
        let mnemonic = self.mnemonic(fingerprint, old)?;
        self.choice
            .open(StoreKind::File, new)
            .put(fingerprint, &mnemonic.entropy())
    }
    pub fn mark_backed_up(&mut self, fingerprint: u32) -> Result<()> {
        self.get_mut(fingerprint)?.backed_up = true;
        self.save()
    }
    /// A play session ended (the backup prompt may follow).
    pub fn session_ended(&mut self, fingerprint: u32) -> Result<()> {
        let a = self.get_mut(fingerprint)?;
        a.sessions = a.sessions.saturating_add(1);
        self.save()
    }
    pub fn set_default(&mut self, fingerprint: u32) -> Result<()> {
        self.get_mut(fingerprint)?;
        self.default = Some(fingerprint);
        self.save()
    }
    pub fn rename(&mut self, fingerprint: u32, name: &str) -> Result<()> {
        self.get_mut(fingerprint)?.name = name.to_owned();
        self.save()
    }
    /// Forget an account on this device: its secret is deleted from its store.
    pub fn remove(&mut self, fingerprint: u32) -> Result<()> {
        let kind = self
            .get(fingerprint)
            .ok_or_else(|| Error::new("no such account"))?
            .store_kind();
        self.choice
            .open(kind, Secret::new(String::new()))
            .delete(fingerprint)?;
        self.list.retain(|a| a.fingerprint != fingerprint);
        if self.default == Some(fingerprint) {
            self.default = self.list.first().map(|a| a.fingerprint);
        }
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("cc-accounts-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn a_silent_account_restores_to_the_same_identity_elsewhere() {
        let a = home("a");
        let choice = StoreChoice { kind: StoreKind::File, service: "chiacraft wallet test".into(), file_dir: a.join(WALLET_DIR) };
        let mut accounts = Accounts::load(&a, choice.clone()).unwrap();
        let first = accounts.ensure_default("Account 1").unwrap();
        assert_eq!(accounts.ensure_default("x").unwrap(), first);
        assert!(!first.wants_backup());
        assert!(accounts.unprotected(first.fingerprint));
        accounts.session_ended(first.fingerprint).unwrap();
        let reloaded = Accounts::load(&a, choice).unwrap();
        assert!(reloaded.current().unwrap().wants_backup());
        // The backup screen's words, typed on a second device.
        let words = reloaded
            .reveal(first.fingerprint, Secret::new(String::new()))
            .unwrap()
            .words();
        let b = home("b");
        let mut other = Accounts::load(
            &b,
            StoreChoice { kind: StoreKind::File, service: "chiacraft wallet test".into(), file_dir: b.join(WALLET_DIR) },
        )
        .unwrap();
        let typed = Mnemonic::parse(&words.expose().join(" ")).unwrap();
        let restored = other.restore(&typed, "Restored").unwrap();
        assert_eq!(restored.identity, first.identity);
        assert!(!restored.wants_backup());
        let unlocked = other
            .unlock(restored.fingerprint, Secret::new(String::new()))
            .unwrap();
        assert_eq!(crate::sign::Signer::identity(&*unlocked.signer).hex(), first.identity);
        // No secret in accounts.json.
        let text = std::fs::read_to_string(b.join(FILE)).unwrap();
        assert!(!text.contains(&words.expose()[..3].join(" ")));
        assert!(!text.contains(&keys::hex(typed.entropy().expose())));
        assert!(other.restore(&typed, "again").is_err());
        let _ = std::fs::remove_dir_all(a);
        let _ = std::fs::remove_dir_all(b);
    }

    #[test]
    fn a_passphrase_protects_the_file() {
        let a = home("p");
        let mut accounts = Accounts::load(
            &a,
            StoreChoice { kind: StoreKind::File, service: "chiacraft wallet test".into(), file_dir: a.join(WALLET_DIR) },
        )
        .unwrap();
        let info = accounts.create("A").unwrap().0;
        let f = info.fingerprint;
        accounts
            .set_passphrase(f, Secret::new(String::new()), Secret::new("pw".into()))
            .unwrap();
        assert!(accounts.needs_passphrase(f));
        assert!(
            accounts
                .unlock(f, Secret::new(String::new()))
                .err()
                .unwrap()
                .is_wrong_passphrase()
        );
        assert!(accounts.unlock(f, Secret::new("pw".into())).is_ok());
        accounts.remove(f).unwrap();
        assert!(accounts.list.is_empty());
        assert!(!a.join(WALLET_DIR).join(format!("{f}.key")).exists());
        let _ = std::fs::remove_dir_all(a);
    }

    #[test]
    fn memory_accounts_keep_nothing_on_disk_but_the_list() {
        let a = home("m");
        let mut accounts =
            Accounts::load(&a, StoreChoice { kind: StoreKind::Memory, service: String::new(), file_dir: a.join(WALLET_DIR) }).unwrap();
        let info = accounts.ensure_default("M").unwrap();
        assert!(!a.join(WALLET_DIR).exists());
        assert!(
            accounts
                .unlock(info.fingerprint, Secret::new(String::new()))
                .is_ok()
        );
        let _ = std::fs::remove_dir_all(a);
    }

    #[test]
    fn words_provided_make_the_folders_one_account() {
        let a = home("provide");
        let choice = StoreChoice { kind: StoreKind::Memory, service: String::new(), file_dir: a.join(WALLET_DIR) };
        let words = Mnemonic::generate().unwrap();
        let mut accounts = Accounts::load(&a, choice.clone()).unwrap();
        let info = accounts.provide(&words, "server").unwrap();
        assert!(info.restored);
        // A later start: the list is on disk, the secret only in the words given again.
        memory().delete(info.fingerprint).unwrap();
        let mut again = Accounts::load(&a, choice.clone()).unwrap();
        assert!(
            again
                .unlock(info.fingerprint, Secret::new(String::new()))
                .is_err()
        );
        again.provide(&words, "server").unwrap();
        assert!(
            again
                .unlock(info.fingerprint, Secret::new(String::new()))
                .is_ok()
        );
        assert_eq!(again.current().unwrap().fingerprint, info.fingerprint);
        // Other words for the same folder: refused, naming neither's words.
        let other = Mnemonic::generate().unwrap();
        let e = again.provide(&other, "server").unwrap_err().to_string();
        assert!(e.contains("already holds account"), "{e}");
        assert_eq!(again.list.len(), 1);
        let _ = std::fs::remove_dir_all(a);
    }
}
