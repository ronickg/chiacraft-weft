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
        for g in grants {
            let Some(p) = self
                .players
                .iter()
                .find(|p| p.joined && p.entity == g.player)
            else {
                continue;
            };
            let Some(subject) = p
                .identity
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
                Credential::unsigned(issuer.clone(), &subject.0, achievement, crate::saves::now_secs(), nonce, None, status.clone())
                    .sign(&*session.signer)
            });
            let c = match signed {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("signing an achievement: {e}");
                    continue;
                }
            };
            log::info!("{} earned '{}': signed", p.name, c.achievement.id);
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
        return Some(IssuerDid { did, owner, sig }).filter(|_| !key.is_empty());
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
