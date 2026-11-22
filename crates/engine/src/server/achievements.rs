//! chiacraft (MIT): the server signs the achievements mods grant (`crate::chiacraft::achievements`).

use super::*;
use chiacraft_wallet::Signer;
use chiacraft_wallet::achievement::{Achievement, Credential, Issuer, IssuerDid, new_nonce};

impl Server {
    /// Signs each achievement granted since the last tick and sends it to its player. A player
    /// without an account identity gets none; a world whose rule `sign_achievements` is off signs
    /// none.
    pub(super) fn sign_achievements(&mut self) {
        let grants = std::mem::take(&mut *self.shared.grants.lock());
        if grants.is_empty() {
            return;
        }
        let on = content()
            .game
            .config_names
            .get("core:sign_achievements")
            .is_none_or(|id| crate::config::get(*id) != 0.0);
        let Some(session) = chiacraft_account::current().filter(|_| on) else { return };
        let dir = self.storage.as_ref().map(|s| s.lock().meta.dir.clone());
        let name = self
            .storage
            .as_ref()
            .map(|s| s.lock().meta.name.clone())
            .unwrap_or_default();
        let issuer_key = session.signer.identity();
        let issuer = Issuer {
            key: issuer_key.hex(),
            name: name
                .chars()
                .take(chiacraft_wallet::achievement::MAX_NAME)
                .collect(),
            did: dir.as_deref().and_then(|d| issuer_did(d, &issuer_key.0)),
        };
        let status = self
            .cfg
            .identity
            .as_ref()
            .map(|i| format!("chiacraft://w/{}", i.key()));
        // What the world's DID was launched with, so a verifier can confirm a DID never spent.
        let data = dir
            .as_deref()
            .and_then(crate::chiacraft::worlds::world_did::load)
            .filter(|_| issuer.did.is_some())
            .map(|d| serde_json::json!({ "world": { "owner": d.owner, "key": d.key, "generation": d.generation } }).to_string());
        for g in grants {
            let Some(p) = self
                .players
                .iter()
                .find(|p| p.joined && p.entity == g.player)
            else {
                continue;
            };
            let Some(subject) = p
                .account
                .as_deref()
                .and_then(chiacraft_wallet::IdentityKey::from_hex)
            else {
                continue;
            };
            let cut = |s: &str, n: usize| s.chars().take(n).collect::<String>();
            let achievement = Achievement {
                id: cut(&g.id, chiacraft_wallet::achievement::MAX_ID),
                title: cut(&g.title, chiacraft_wallet::achievement::MAX_TITLE),
                description: cut(&g.description, chiacraft_wallet::achievement::MAX_DESCRIPTION),
                icon: g
                    .icon
                    .map(|i| cut(&i, chiacraft_wallet::achievement::MAX_ICON)),
            };
            let signed = new_nonce().and_then(|nonce| {
                Credential::unsigned(issuer.clone(), &subject.0, achievement, crate::saves::now_secs(), nonce, data.clone(), status.clone())
                    .sign(&*session.signer)
            });
            let c = match signed {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("signing an achievement: {e}");
                    continue;
                }
            };
            log::info!("{} earned '{}': signed", p.identity.name, c.achievement.id);
            if let Some(d) = &dir {
                record(d, &c);
            }
            let p = self
                .players
                .iter_mut()
                .find(|p| p.joined && p.entity == g.player)
                .expect("found above");
            p.conn.send(ToClient::Achievement(c.to_json()));
        }
    }
}

/// The world DID's proof for this issuer: the owner key of the address holding the world's DID
/// (the account's first) signing `issuer_did_text`, kept in `did.json` once made.
fn issuer_did(dir: &std::path::Path, issuer: &[u8; 48]) -> Option<IssuerDid> {
    let mut d = crate::chiacraft::worlds::world_did::load(dir).filter(|d| d.listed)?;
    let launcher: [u8; 32] = chiacraft_wallet::keys::from_hex(&d.launcher)?
        .try_into()
        .ok()?;
    let did = chiacraft_wallet::keys::encode_did(&launcher)?;
    let issuer_hex = chiacraft_wallet::keys::hex(issuer);
    if let Some((key, owner, sig)) = d.issuer.clone().filter(|(k, _, _)| *k == issuer_hex) {
        return (!key.is_empty()).then_some(IssuerDid { did, owner, sig });
    }
    let text = chiacraft_wallet::achievement::issuer_did_text(&launcher, issuer);
    match chiacraft_account::current()?.first_owner_sign(&text) {
        Ok((owner, sig)) => {
            d.issuer = Some((issuer_hex, owner.clone(), sig.clone()));
            if let Err(e) = crate::chiacraft::worlds::world_did::save(dir, &d) {
                log::warn!("{e}");
            }
            Some(IssuerDid { did, owner, sig })
        }
        Err(e) => {
            log::warn!("the world DID's proof for achievements: {e}");
            None
        }
    }
}

/// What the world signed, a line each in `achievements/issued.jsonl` (the credential's hash, its
/// UTC day, the player): what the daily anchor commits to.
fn record(dir: &std::path::Path, c: &Credential) {
    use std::io::Write;
    let line = serde_json::json!({
        "hash": chiacraft_wallet::keys::hex(&c.hash()),
        "day": chiacraft_wallet::achievement::day_of(c.issued),
        "player": c.subject.key,
        "id": c.achievement.id,
        "nonce": c.nonce,
    });
    let d = dir.join("achievements");
    let r = std::fs::create_dir_all(&d).and_then(|_| {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(d.join("issued.jsonl"))
            .and_then(|mut f| writeln!(f, "{line}"))
    });
    if let Err(e) = r {
        log::warn!("recording an achievement: {e}");
    }
}

/// A line of `achievements/issued.jsonl`.
struct Issued {
    hash: [u8; 32],
    day: u64,
    player: String,
    id: String,
    /// What a revocation names (lines from before it was recorded have none).
    nonce: String,
}

fn issued(dir: &std::path::Path) -> Vec<Issued> {
    let text = std::fs::read_to_string(dir.join("achievements").join("issued.jsonl")).unwrap_or_default();
    text.lines()
        .filter_map(|l| {
            let v: serde_json::Value = serde_json::from_str(l).ok()?;
            Some(Issued {
                hash: chiacraft_wallet::keys::from_hex(v.get("hash")?.as_str()?)?
                    .try_into()
                    .ok()?,
                day: v.get("day")?.as_u64()?,
                player: v.get("player")?.as_str()?.to_string(),
                id: v
                    .get("id")
                    .and_then(|i| i.as_str())
                    .unwrap_or("")
                    .to_string(),
                nonce: v
                    .get("nonce")
                    .and_then(|i| i.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
        })
        .collect()
}

/// A day the owner's game committed (`achievements/anchors.json`): its root, the coin carrying
/// it, the network.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Anchored {
    day: u64,
    root: String,
    coin: String,
    network: String,
}

fn anchors(dir: &std::path::Path) -> Vec<Anchored> {
    std::fs::read(dir.join("achievements").join("anchors.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

/// One anchoring at a time per game.
static ANCHORING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

impl Server {
    /// The daily anchor (opt-in, the world rule `anchor_achievements`): the earliest past day
    /// with achievements signed and not anchored is committed on chain, its Merkle root in a coin
    /// the owner's wallet spends to itself (fee 0), on a thread of its own.
    pub(super) fn anchor_achievements(&mut self) {
        let on = content()
            .game
            .config_names
            .get("core:anchor_achievements")
            .is_some_and(|id| crate::config::get(*id) != 0.0);
        let Some(dir) = self
            .storage
            .as_ref()
            .map(|s| s.lock().meta.dir.clone())
            .filter(|_| on)
        else {
            return;
        };
        let today = chiacraft_wallet::achievement::day_of(crate::saves::now_secs());
        let done: Vec<u64> = anchors(&dir).iter().map(|a| a.day).collect();
        let lines = issued(&dir);
        let Some(day) = lines
            .iter()
            .map(|l| l.day)
            .filter(|d| *d < today && !done.contains(d))
            .min()
        else {
            return;
        };
        if ANCHORING.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        let hashes: Vec<[u8; 32]> = lines
            .iter()
            .filter(|l| l.day == day)
            .map(|l| l.hash)
            .collect();
        let _ = std::thread::Builder::new()
            .name("achievement anchor".into())
            .spawn(move || {
                if let Err(e) = anchor_day(&dir, day, &hashes) {
                    log::warn!("anchoring the achievements of day {day}: {e}");
                }
                ANCHORING.store(false, std::sync::atomic::Ordering::SeqCst);
            });
    }

    /// Withdraws what this world signed for the player known as `identity` for achievement `id`
    /// (all of them: `*`): the issuer's list signed again and sent to everyone here. How many.
    pub(super) fn revoke_achievements(&mut self, identity: &str, id: &str) -> Result<usize, String> {
        let dir = self
            .storage
            .as_ref()
            .map(|s| s.lock().meta.dir.clone())
            .ok_or("this world isn't saved")?;
        let nonces: Vec<String> = issued(&dir)
            .into_iter()
            .filter(|l| l.player == identity && (id == "*" || l.id == id) && !l.nonce.is_empty())
            .map(|l| l.nonce)
            .collect();
        if nonces.is_empty() {
            return Ok(0);
        }
        let list = crate::chiacraft::achievements::revoke(&nonces)?;
        log::info!("{} achievement(s) withdrawn (revocation list version {})", nonces.len(), list.seq);
        let json = list.to_json();
        for p in self.players.iter_mut().filter(|p| p.joined) {
            p.conn.send(ToClient::Revocations(json.clone()));
        }
        Ok(nonces.len())
    }

    /// Sends player `pi` the proofs for their achievements from this world on the days anchored.
    pub(super) fn send_anchor_proofs(&mut self, pi: usize) {
        // And the issuer's list of withdrawn ones.
        if let Some(r) = crate::chiacraft::achievements::issuer_list() {
            self.players[pi]
                .conn
                .send(ToClient::Revocations(r.to_json()));
        }
        let Some(dir) = self.storage.as_ref().map(|s| s.lock().meta.dir.clone()) else { return };
        let Some(me) = self.players[pi].account.clone() else { return };
        let done = anchors(&dir);
        if done.is_empty() {
            return;
        }
        let lines = issued(&dir);
        for a in done {
            let day: Vec<[u8; 32]> = lines
                .iter()
                .filter(|l| l.day == a.day)
                .map(|l| l.hash)
                .collect();
            for (i, l) in lines.iter().filter(|l| l.day == a.day).enumerate() {
                if l.player != me {
                    continue;
                }
                let mut proof = chiacraft_wallet::achievement::Anchor::of(a.day, &day, i, &a.network);
                if proof.root != a.root {
                    continue;
                }
                proof.coin = Some(a.coin.clone());
                let anchor = serde_json::to_string(&proof).unwrap_or_default();
                self.players[pi]
                    .conn
                    .send(ToClient::AchievementAnchor { hash: chiacraft_wallet::keys::hex(&l.hash), anchor });
            }
        }
    }
}

/// Commits day `day`'s root on chain from the owner's wallet and records it.
fn anchor_day(dir: &std::path::Path, day: u64, hashes: &[[u8; 32]]) -> Result<(), String> {
    let session = chiacraft_account::current().ok_or("no account is open")?;
    let reader = crate::chiacraft::chain::get()?;
    let network = reader.network();
    crate::chiacraft::chain::may_spend(network)?;
    let root_hex = chiacraft_wallet::achievement::Anchor::of(day, hashes, 0, network.name()).root;
    let root: [u8; 32] = chiacraft_wallet::keys::from_hex(&root_hex)
        .and_then(|b| b.try_into().ok())
        .ok_or("a bad root")?;
    let words = session.words()?;
    let mnemonic = chiacraft_wallet::Mnemonic::parse(&words.expose().join(" ")).map_err(|e| e.to_string())?;
    let keys = chiacraft_wallet::chain::WalletKeys::with_hardened(&mnemonic, network, 20, 20);
    if !reader.wait_synced(std::time::Duration::from_secs(60)) {
        return Err("the wallet isn't synced with the chain".into());
    }
    let (push, coin) = reader
        .anchor(&keys, root, 0, std::time::Duration::from_secs(60))
        .map_err(|e| e.to_string())?;
    if matches!(push, chiacraft_wallet::chain::PushOutcome::Rejected { .. }) {
        return Err(format!("the chain refused it: {push:?}"));
    }
    let mut all = anchors(dir);
    all.push(Anchored { day, root: root_hex.clone(), coin: chiacraft_wallet::keys::hex(&coin.to_bytes()), network: network.name().into() });
    let d = dir.join("achievements");
    std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    let tmp = d.join("anchors.json.new");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&all).map_err(|e| e.to_string())?)
        .and_then(|_| std::fs::rename(&tmp, d.join("anchors.json")))
        .map_err(|e| e.to_string())?;
    log::info!("achievements of day {day} ({} of them) anchored on {network}: root {}", hashes.len(), &root_hex[..12]);
    Ok(())
}
