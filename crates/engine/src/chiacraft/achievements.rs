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

/// Whether the world that issued `c` is the owner's, on chain: its DID (the credential's issuer
/// DID) and the owner's DID name each other, and the key that vouched for the issuer holds the
/// owner's DID. None for a credential without an issuer DID. Blocks; never from an async task.
pub fn check_issuer(c: &Credential) -> Option<Result<bool, String>> {
    let d = c.issuer.did.as_ref()?;
    Some((|| {
        let data: serde_json::Value =
            serde_json::from_str(c.data.as_deref().ok_or("no world in the credential")?).map_err(|e| e.to_string())?;
        let w = data.get("world").ok_or("no world in the credential")?;
        let owner = w.get("owner").and_then(|v| v.as_str()).ok_or("no owner")?;
        let key = w.get("key").and_then(|v| v.as_str()).ok_or("no key")?;
        let generation = w
            .get("generation")
            .and_then(|v| v.as_u64())
            .ok_or("no generation")? as u32;
        let owner_bytes: [u8; 32] = chiacraft_wallet::keys::from_hex(owner)
            .and_then(|b| b.try_into().ok())
            .ok_or("a bad owner")?;
        let owner_did = chiacraft_wallet::keys::encode_did(&owner_bytes).ok_or("a bad owner")?;
        if !crate::chiacraft::worlds::world_did::check(&d.did, &owner_did, key, generation)? {
            return Ok(false);
        }
        let reader = crate::chiacraft::chain::get()?;
        let (record, _) = crate::chiacraft::chain::did(&reader, &owner_did)?;
        let signer: [u8; 48] = chiacraft_wallet::keys::from_hex(&d.owner)
            .and_then(|b| b.try_into().ok())
            .ok_or("a bad DID owner key")?;
        let p2 = chiacraft_wallet::keys::p2_of_synthetic(&signer).ok_or("a bad DID owner key")?;
        Ok(record.owned_by(p2.into()))
    })())
}

/// Issuer checks done in this game, by issuer DID: none yet (checking), or the answer.
type Check = std::sync::Arc<parking_lot::Mutex<Option<bool>>>;
static CHECKS: parking_lot::Mutex<Option<std::collections::HashMap<String, Check>>> = parking_lot::Mutex::new(None);

/// [`check_issuer`] in the background, once per issuer DID per game: None while it runs.
pub fn issuer_checked(c: &Credential) -> Option<bool> {
    let d = c.issuer.did.as_ref()?;
    let mut all = CHECKS.lock();
    let slot = all
        .get_or_insert_with(Default::default)
        .entry(d.did.clone())
        .or_insert_with(|| {
            let slot = Check::default();
            let (out, c) = (slot.clone(), c.clone());
            let _ = std::thread::Builder::new()
                .name("achievement check".into())
                .spawn(move || {
                    let r = check_issuer(&c).unwrap_or(Ok(false));
                    if let Err(e) = &r {
                        log::info!("the DID of '{}' (achievements): {e}", c.issuer.name);
                    }
                    *out.lock() = Some(r.unwrap_or(false));
                    GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                });
            slot
        })
        .clone();
    *slot.lock()
}

/// `2026-11-04` for unix seconds (UTC).
pub fn date(unix: u64) -> String {
    // Howard Hinnant's days-to-civil.
    let z = (unix / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (m <= 2) as i64;
    format!("{y:04}-{m:02}-{d:02}")
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
    fn dates() {
        assert_eq!(date(0), "1970-01-01");
        assert_eq!(date(1_793_764_725), "2026-11-04");
        assert_eq!(date(951_782_400), "2000-02-29");
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
