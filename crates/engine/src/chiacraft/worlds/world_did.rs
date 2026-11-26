//! A world's DID, owned both ways (`notes/PLAN.md`, phase 4, slice 7).
//!
//! The owner asks a sponsor for a DID for their world: it is launched to the owner's first
//! address (the DID stays in the owner's wallet; the world holds only its key) with metadata
//! naming the owner's DID, the world's key and its generation. Then the owner's game spends the
//! owner's own DID once, listing the world in its metadata. A world counts as the DID's only when
//! both say so; anyone can check that on chain.
//!
//! The world keeps `did.json` in its folder ([`WorldDid`]): not in `world.txt`, which the running
//! server writes from memory. The world's status answer carries it, so visitors can check.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use chiacraft_wallet::chain::{self, Bytes32, ChainNetwork, DidMetadata, WalletKeys, WorldMeta};
use serde::{Deserialize, Serialize};

use crate::chiacraft::did::{short, short_did};

pub const FILE: &str = "did.json";

/// A world's DID as the world keeps it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldDid {
    /// The world DID's launcher id, hex.
    pub launcher: String,
    /// The owner's DID it names (launcher id, hex).
    pub owner: String,
    /// The world key and generation it was launched for.
    pub key: String,
    pub generation: u32,
    /// The owner's DID lists it (seen on chain).
    #[serde(default)]
    pub listed: bool,
    /// The proof the world's achievements carry (slice 8): the issuer key (hex), the DID owner
    /// key (hex) and its signature of `issuer_did_text`, made once.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<(String, String, String)>,
}

impl WorldDid {
    pub fn did(&self) -> String {
        hex32(&self.launcher)
            .and_then(|l| chiacraft_wallet::keys::encode_did(&l))
            .unwrap_or_default()
    }
    /// The metadata it was launched with.
    pub fn meta(&self) -> Option<WorldMeta> {
        Some(WorldMeta { owner: Bytes32::from(hex32(&self.owner)?), key: hex32(&self.key)?, generation: self.generation })
    }
}

fn hex32(h: &str) -> Option<[u8; 32]> {
    chiacraft_wallet::keys::from_hex(h)?.try_into().ok()
}

pub fn load(world: &Path) -> Option<WorldDid> {
    serde_json::from_slice(&std::fs::read(world.join(FILE)).ok()?).ok()
}

pub fn save(world: &Path, d: &WorldDid) -> Result<(), String> {
    let tmp = world.join(format!("{FILE}.new"));
    std::fs::write(&tmp, serde_json::to_vec_pretty(d).map_err(|e| e.to_string())?)
        .and_then(|_| std::fs::rename(&tmp, world.join(FILE)))
        .map_err(|e| format!("saving the world's DID: {e}"))
}

/// What the world's DID is doing, for the owner's screen: a lang key and its arguments.
static STATE: parking_lot::Mutex<(String, Vec<String>)> = parking_lot::Mutex::new((String::new(), Vec::new()));
static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn set(key: &str, args: &[String]) {
    *STATE.lock() = (key.to_string(), args.to_vec());
    crate::admission::GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

pub fn state() -> (String, Vec<String>) {
    STATE.lock().clone()
}

/// The world's key as 32 bytes, hex: what its DID's metadata names.
fn key_hex(identity: &crate::world_address::WorldIdentity) -> String {
    chiacraft_wallet::keys::hex(identity.secret.public().as_bytes())
}

/// Get the world in `dir` (with key `identity`) a DID from a sponsor, then list it in the owner's
/// DID. Needs the owner's own DID first.
pub fn get(settings: &crate::settings::Settings, dir: PathBuf, identity: Arc<crate::world_address::WorldIdentity>) {
    let settings = settings.clone();
    run(move || {
        let session = chiacraft_account::current().ok_or("no account is open")?;
        let owner = session
            .did()
            .ok_or("get your own DID first (Account screen)")?;
        let network = crate::chiacraft::chain::network(&crate::settings::chain())?;
        let address = session
            .first_puzzle_hash()
            .map(|ph| chain::address(Bytes32::from(ph), network))
            .ok_or("the account has no wallet address")?;
        let ask = chiacraft_wallet::sponsor::WorldAsk {
            owner_did: owner.launcher.clone(),
            key: key_hex(&identity),
            generation: identity.cert.generation,
        };
        let sponsors = crate::settings::sponsors();
        if sponsors.is_empty() {
            return Err("no DID sponsor is set (setting core:sponsors)".into());
        }
        let mut last = String::new();
        for sponsor in &sponsors {
            set("world_did.asking", &[short(sponsor)]);
            let bits = match crate::net::p2p::ask_sponsor(&settings, sponsor, None) {
                Ok((200, body)) => serde_json::from_str::<serde_json::Value>(&body)
                    .ok()
                    .and_then(|v| v.get("work_bits")?.as_u64())
                    .unwrap_or(0) as u32,
                Ok((status, body)) => {
                    last = format!("{}: {status} {body}", short(sponsor));
                    continue;
                }
                Err(e) => {
                    last = format!("{}: {e}", short(sponsor));
                    continue;
                }
            };
            let now = crate::saves::now_secs();
            let mut request = chiacraft_wallet::sponsor::Request::sign_world(&*session.signer, network.name(), &address, now, ask.clone())
                .map_err(|e| e.to_string())?;
            request.do_work(bits).map_err(|e| e.to_string())?;
            let answer = crate::net::p2p::ask_sponsor(&settings, sponsor, Some(&request))
                .and_then(|(status, body)| chiacraft_wallet::sponsor::read_answer(status, &body).map_err(|e| e.to_string()));
            match answer {
                Ok(reply) => {
                    let launcher = reply.launcher_id().map_err(|e| e.to_string())?;
                    let d = WorldDid {
                        launcher: chiacraft_wallet::keys::hex(&launcher),
                        owner: ask.owner_did.clone(),
                        key: ask.key.clone(),
                        generation: ask.generation,
                        listed: false,
                        issuer: None,
                    };
                    save(&dir, &d)?;
                    log::info!("world DID {} from sponsor {}", d.did(), short(sponsor));
                    return list_now(&settings, &dir, d);
                }
                Err(e) => last = format!("{}: {e}", short(sponsor)),
            }
        }
        Err(last)
    });
}

/// List the world's DID in the owner's DID, if it isn't yet (after a restart, a failed spend).
pub fn list(settings: &crate::settings::Settings, dir: PathBuf) {
    let settings = settings.clone();
    let Some(d) = load(&dir).filter(|d| !d.listed) else { return };
    run(move || list_now(&settings, &dir, d));
}

/// The owner's spend: their DID's metadata gains this world. Waits for the world's DID to be on
/// chain first, then for the spend to show.
fn list_now(settings: &crate::settings::Settings, dir: &Path, mut d: WorldDid) -> Result<(), String> {
    let session = chiacraft_account::current().ok_or("no account is open")?;
    let network = crate::chiacraft::chain::network(&crate::settings::chain())?;
    let world = Bytes32::from(hex32(&d.launcher).ok_or("a damaged did.json")?);
    let owner = Bytes32::from(hex32(&d.owner).ok_or("a damaged did.json")?);
    let reader = crate::chiacraft::chain::reader(settings)?;
    set("world_did.pending", &[short_did(&d.did())]);
    reader
        .wait_did(world, std::time::Duration::from_secs(10), std::time::Duration::from_secs(600))
        .map_err(|e| format!("the world's DID isn't on the chain yet: {e}"))?;
    let record = reader
        .wait_did(owner, std::time::Duration::from_secs(10), std::time::Duration::from_secs(60))
        .map_err(|e| format!("your DID can't be read: {e}"))?;
    let listed = record.owner_meta().unwrap_or_default();
    if !listed.lists(world) {
        set("world_did.listing", &[]);
        crate::chiacraft::chain::may_spend(network)?;
        let meta = DidMetadata::Owner(listed.with_world(world).map_err(|e| e.to_string())?);
        let keys = wallet_keys(&session, network)?;
        // testnet11's mempool may hold a spend with no fee for most of an hour: pay a little
        // there, from the sponsor's gift, when the wallet has it.
        let fee = if network == ChainNetwork::Testnet11 && reader.spendable_balance() >= TESTNET_FEE { TESTNET_FEE } else { 0 };
        reader
            .set_did_metadata(&keys, owner, &meta, fee, std::time::Duration::from_secs(60))
            .map_err(|e| format!("listing the world in your DID: {e}"))?;
        // The spend shows once the chain has it.
        let until = std::time::Instant::now() + std::time::Duration::from_secs(900);
        loop {
            let r = crate::chiacraft::chain::did(&reader, &chiacraft_wallet::keys::encode_did(&owner.to_bytes()).unwrap_or_default())?;
            if r.0.owner_meta().is_some_and(|m| m.lists(world)) {
                break;
            }
            if std::time::Instant::now() > until {
                return Err("the listing isn't on the chain after 15 minutes".into());
            }
            std::thread::sleep(std::time::Duration::from_secs(15));
        }
    }
    d.listed = true;
    save(dir, &d)?;
    log::info!("world DID {} is listed in its owner's DID", d.did());
    set("world_did.listed", &[short_did(&d.did())]);
    Ok(())
}

/// The fee the owner's listing spend pays on testnet11, in mojos.
const TESTNET_FEE: u64 = 5_000_000;

/// The account's wallet keys (first 20 addresses, both branches), for the owner's spend. The words
/// are read for this and not kept.
fn wallet_keys(session: &chiacraft_account::Session, network: ChainNetwork) -> Result<WalletKeys, String> {
    let words = session.words()?;
    let mnemonic = chiacraft_wallet::Mnemonic::parse(&words.expose().join(" ")).map_err(|e| e.to_string())?;
    Ok(WalletKeys::with_hardened(&mnemonic, network, 20, 20))
}

fn run(f: impl FnOnce() -> Result<(), String> + Send + 'static) {
    if BUSY.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("world did".into())
        .spawn(move || {
            if let Err(e) = f() {
                log::warn!("world DID: {e}");
                set("world_did.failed", &[e]);
            }
            BUSY.store(false, std::sync::atomic::Ordering::SeqCst);
        });
}

/// A visitor's check of a world's DID, both ways, on chain: whether the world DID names `owner`
/// and the key `key` (generation `generation`) and `owner` lists it. Blocks; never from an async
/// task.
pub fn check(world_did: &str, owner_did: &str, key: &str, generation: u32) -> Result<bool, String> {
    let reader = crate::chiacraft::chain::get()?;
    let (world, _) = crate::chiacraft::chain::did(&reader, world_did)?;
    let (owner, _) = crate::chiacraft::chain::did(&reader, owner_did)?;
    let expected = WorldMeta { owner: owner.launcher_id, key: hex32(key).ok_or("not a world key")?, generation };
    // A world DID not spent yet shows no metadata: its puzzle commits to it, launched to the
    // address that holds the owner's DID.
    let launched_to = owner
        .owner()
        .or_else(|| owner.current().map(|c| c.puzzle_hash))
        .unwrap_or_default();
    Ok(chain::world::world_link_expecting(&world, &owner, &expected, launched_to).holds())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_keeps_what_the_dids_metadata_names() {
        let dir = std::env::temp_dir().join(format!("chiacraft-world-did-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        assert_eq!(load(&dir), None);
        let d = WorldDid {
            launcher: "11".repeat(32),
            owner: "22".repeat(32),
            key: "33".repeat(32),
            generation: 2,
            listed: false,
            issuer: None,
        };
        save(&dir, &d).unwrap();
        assert_eq!(load(&dir), Some(d.clone()));
        let m = d.meta().unwrap();
        assert_eq!((m.owner, m.key, m.generation), (Bytes32::from([0x22; 32]), [0x33; 32], 2));
        assert!(d.did().starts_with("did:chia:1"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
