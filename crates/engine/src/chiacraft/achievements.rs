//! Achievements that outlive a world (`notes/PLAN.md`, phase 4, slices 8-10).
//!
//! When a world grants an advancement (the base mod's, or any mod through the bridge's
//! `achievement` question), its server signs a credential for the player
//! (`chiacraft_wallet::achievement::Credential`): the issuer is the account the world runs with
//! (its owner's), named after the world, with the world's DID and its owner key's proof when the
//! world has one, and the world's address as where to ask about it. The player's game checks it
//! and keeps it in the account home (`achievements.json`), so it follows the words to any
//! computer. Off chain and free; a world can turn it off (`/rule sign_achievements 0`).
//!
//! The server side is `server::achievements`; this is the shared part and the player's store.

use std::path::{Path, PathBuf};

use chiacraft_wallet::achievement::Credential;
use serde::{Deserialize, Serialize};

pub const FILE: &str = "achievements.json";

/// An achievement a mod granted, waiting for the server to sign it.
#[derive(Clone, Debug)]
pub struct Grant {
    /// The player's entity.
    pub player: u64,
    pub id: String,
    /// A lang key or text.
    pub title: String,
    pub description: String,
    pub icon: Option<String>,
}

/// The player's achievements, as kept in the account home.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Store {
    #[serde(default)]
    pub v: u32,
    #[serde(default)]
    pub credentials: Vec<Credential>,
}

fn path() -> Option<PathBuf> {
    chiacraft_account::current().map(|s| s.home.join(FILE))
}

/// The achievements this account holds (none without an account).
pub fn load() -> Store {
    path().map(|p| load_from(&p)).unwrap_or_default()
}

fn load_from(path: &Path) -> Store {
    match std::fs::read(path) {
        Ok(b) => serde_json::from_slice(&b).unwrap_or_else(|e| {
            // Never lost: moved aside, as admission's files are.
            let aside = path.with_extension(format!("json.bad-{}", crate::saves::now_secs()));
            let _ = std::fs::rename(path, &aside);
            log::error!("{} could not be read ({e}): kept as {}", path.display(), aside.display());
            Store::default()
        }),
        Err(_) => Store::default(),
    }
}

fn save_to(path: &Path, s: &Store) -> Result<(), String> {
    let tmp = path.with_extension("json.new");
    std::fs::write(&tmp, serde_json::to_vec_pretty(s).map_err(|e| e.to_string())?)
        .and_then(|_| std::fs::rename(&tmp, path))
        .map_err(|e| format!("saving achievements: {e}"))
}

/// Raised whenever an achievement arrives: the profile shows it.
pub static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// A credential a world sent: kept if it checks out and is this account's, once.
pub fn received(json: &str) {
    let Some(session) = chiacraft_account::current() else { return };
    let Some(path) = path() else { return };
    if let Err(e) = keep(&path, &session.identity_hex(), json) {
        log::warn!("an achievement from the world wasn't kept: {e}");
    }
}

/// Keeps `json` in the store at `path` if it is a valid credential for `me` (identity hex):
/// whether it was new.
pub fn keep(path: &Path, me: &str, json: &str) -> Result<bool, String> {
    if json.len() > 8 << 10 {
        return Err("too big".into());
    }
    let c = Credential::from_json(json).map_err(|e| e.to_string())?;
    c.verify().map_err(|e| e.to_string())?;
    if c.subject.key != me {
        return Err("it is someone else's".into());
    }
    let mut store = load_from(path);
    if store.credentials.iter().any(|k| k.hash() == c.hash()) {
        return Ok(false);
    }
    log::info!("achievement '{}' from '{}' kept", c.achievement.id, c.issuer.name);
    store.credentials.push(c);
    save_to(path, &store)?;
    GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chiacraft_wallet::achievement::{Achievement, Issuer, new_nonce};
    use chiacraft_wallet::{IdentitySigner, Mnemonic, Signer};

    fn credential(issuer: &IdentitySigner, subject: &IdentitySigner, id: &str) -> Credential {
        Credential::unsigned(
            Issuer { key: issuer.identity().hex(), name: "Ruby Hills".into(), did: None },
            &subject.identity().0,
            Achievement { id: id.into(), title: "advancement.base.first_log".into(), description: String::new(), icon: None },
            1_700_000_000,
            new_nonce().unwrap(),
            None,
            Some("chiacraft://w/abc".into()),
        )
        .sign(issuer)
        .unwrap()
    }

    #[test]
    fn a_players_store_keeps_their_own_checked_credentials_once() {
        let dir = std::env::temp_dir().join(format!("chiacraft-achievements-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(FILE);
        let _ = std::fs::remove_file(&path);
        let world = IdentitySigner::from_mnemonic(&Mnemonic::generate().unwrap(), 0);
        let me = IdentitySigner::from_mnemonic(&Mnemonic::generate().unwrap(), 0);
        let c = credential(&world, &me, "base:first_log");
        let json = c.to_json();
        assert_eq!(keep(&path, &me.identity().hex(), &json), Ok(true));
        assert_eq!(keep(&path, &me.identity().hex(), &json), Ok(false));
        // Someone else's, or tampered with: refused.
        assert!(keep(&path, &world.identity().hex(), &json).is_err());
        let tampered = json.replace("Ruby Hills", "Fake Hills");
        assert!(keep(&path, &me.identity().hex(), &tampered).is_err());
        assert_eq!(load_from(&path).credentials.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
