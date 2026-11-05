//! Listing a world so others find it (`notes/PLAN.md`, phase 5, slice 1;
//! `reports/Finding worlds.md`): the owner's game makes a coin at the address holding the world's
//! DID that locks a deposit (`core:listing_deposit`, 0.0001 TXCH), hinted with the listing tag and
//! naming the world DID (the wallet's `chain::listing`). Only a world with a DID its owner's DID
//! lists can be listed. Delisting spends it back. testnet11 only (`chain::may_spend`).
//!
//! The world keeps `listing.json` in its folder: the coin, so the owner can delist it.

use std::path::{Path, PathBuf};

use chiacraft_wallet::chain::{Bytes32, Coin, WalletKeys};
use serde::{Deserialize, Serialize};

pub const FILE: &str = "listing.json";

/// A world's listing as the world keeps it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listed {
    /// The listing coin: parent, puzzle hash (hex) and amount.
    pub parent: String,
    pub puzzle_hash: String,
    pub amount: u64,
    /// The world DID it names (launcher id, hex).
    pub did: String,
    /// Seen among the listings on chain.
    #[serde(default)]
    pub confirmed: bool,
}

impl Listed {
    pub fn coin(&self) -> Option<Coin> {
        Some(Coin::new(Bytes32::from(hex32(&self.parent)?), Bytes32::from(hex32(&self.puzzle_hash)?), self.amount))
    }
}

fn hex32(h: &str) -> Option<[u8; 32]> {
    chiacraft_wallet::keys::from_hex(h)?.try_into().ok()
}

pub fn load(world: &Path) -> Option<Listed> {
    serde_json::from_slice(&std::fs::read(world.join(FILE)).ok()?).ok()
}

fn save(world: &Path, l: &Listed) -> Result<(), String> {
    let tmp = world.join(format!("{FILE}.new"));
    std::fs::write(&tmp, serde_json::to_vec_pretty(l).map_err(|e| e.to_string())?)
        .and_then(|_| std::fs::rename(&tmp, world.join(FILE)))
        .map_err(|e| format!("saving the listing: {e}"))
}

/// What the listing is doing, for the owner's screen: a lang key and its arguments.
static STATE: parking_lot::Mutex<(String, Vec<String>)> = parking_lot::Mutex::new((String::new(), Vec::new()));
static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Raised when a world is listed: the game hosts it now if it is open (`app`), so the key its
/// listing names answers.
pub static HOST_NOW: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What a full mempool answers a spend with no fee (testnet11 at busy times): the same spend,
/// pushed again a minute later, goes in once it empties.
const MEMPOOL_FULL: &str = "INVALID_FEE_TOO_CLOSE_TO_ZERO";
const MEMPOOL_TRIES: u32 = 10;

fn mempool_full(push: &chiacraft_wallet::chain::PushOutcome) -> bool {
    matches!(push, chiacraft_wallet::chain::PushOutcome::Rejected { error, .. } if error.contains(MEMPOOL_FULL))
}

/// Pushes with `push` until the mempool takes it or refuses it for another reason, a minute
/// apart while it is full (finder3, 11-04: the listing was refused once and the check went on
/// without it).
fn push_while_full<T>(
    mut push: impl FnMut() -> Result<(chiacraft_wallet::chain::PushOutcome, T), String>,
) -> Result<(chiacraft_wallet::chain::PushOutcome, T), String> {
    let mut tries = 1;
    loop {
        let (outcome, value) = push()?;
        if !mempool_full(&outcome) || tries >= MEMPOOL_TRIES {
            return Ok((outcome, value));
        }
        log::info!("listing: the mempool is full, pushing again in a minute ({tries} of {MEMPOOL_TRIES})");
        set("listing.mempool_full", &[]);
        tries += 1;
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

fn set(key: &str, args: &[String]) {
    *STATE.lock() = (key.to_string(), args.to_vec());
    crate::admission::GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

pub fn state() -> (String, Vec<String>) {
    STATE.lock().clone()
}

fn run(f: impl FnOnce() -> Result<(), String> + Send + 'static) {
    if BUSY.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("listing".into())
        .spawn(move || {
            if let Err(e) = f() {
                log::warn!("listing: {e}");
                set("listing.failed", &[e]);
            }
            BUSY.store(false, std::sync::atomic::Ordering::SeqCst);
        });
}

/// The deposit in TXCH, for showing.
pub fn txch(mojos: u64) -> String {
    let s = format!("{:.12}", mojos as f64 / 1e12);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// The account's wallet keys (its first 20 addresses, both branches): the words are read for it
/// and not kept.
fn wallet_keys(network: chiacraft_wallet::chain::ChainNetwork) -> Result<WalletKeys, String> {
    let session = chiacraft_account::current().ok_or("no account is open")?;
    let words = session.words()?;
    let mnemonic = chiacraft_wallet::Mnemonic::parse(&words.expose().join(" ")).map_err(|e| e.to_string())?;
    Ok(WalletKeys::with_hardened(&mnemonic, network, 20, 20))
}

/// List the world in `dir`: its DID must be named by its owner's DID both ways (`did.json`).
pub fn list(settings: &crate::settings::Settings, dir: PathBuf) {
    let settings = settings.clone();
    run(move || {
        if load(&dir).is_some() {
            return Err("this world is listed already".into());
        }
        let did = crate::chiacraft::worlds::world_did::load(&dir)
            .filter(|d| d.listed)
            .ok_or("get this world a DID first (and let your DID list it)")?;
        let world_did = Bytes32::from(hex32(&did.launcher).ok_or("a damaged did.json")?);
        let world = did.meta().ok_or("a damaged did.json")?;
        let session = chiacraft_account::current().ok_or("no account is open")?;
        let owner = Bytes32::from(
            session
                .first_puzzle_hash()
                .ok_or("the account has no wallet address")?,
        );
        let deposit = settings.listing_deposit();
        let reader = crate::chiacraft::chain::reader(&settings)?;
        let network = reader.network();
        crate::chiacraft::chain::may_spend(network)?;
        set("listing.listing", &[txch(deposit)]);
        if !reader.wait_synced(std::time::Duration::from_secs(60)) {
            return Err("the wallet isn't synced with the chain".into());
        }
        let keys = wallet_keys(network)?;
        let fee = settings.listing_fee();
        let pushed = std::time::Instant::now();
        let pushed_at = reader.status().peak.unwrap_or(0);
        let (push, coin) = push_while_full(|| {
            reader
                .list_world(&keys, owner, &world, world_did, deposit, fee, std::time::Duration::from_secs(60))
                .map_err(|e| e.to_string())
        })?;
        if matches!(push, chiacraft_wallet::chain::PushOutcome::Rejected { .. }) {
            return Err(format!("the chain refused it: {push:?}"));
        }
        let mut listed = Listed {
            parent: chiacraft_wallet::keys::hex(&coin.parent_coin_info.to_bytes()),
            puzzle_hash: chiacraft_wallet::keys::hex(&coin.puzzle_hash.to_bytes()),
            amount: coin.amount,
            did: did.launcher.clone(),
            confirmed: false,
        };
        save(&dir, &listed)?;
        HOST_NOW.store(true, std::sync::atomic::Ordering::Relaxed);
        log::info!(
            "listing world {} with a deposit of {} TXCH and a fee of {} TXCH (coin {}), at height {pushed_at}",
            did.did(),
            txch(deposit),
            txch(fee),
            coin.coin_id()
        );
        // On chain once the finders see it.
        let until = std::time::Instant::now() + std::time::Duration::from_secs(900);
        let created = loop {
            let seen = reader
                .listings(deposit, 1000, std::time::Duration::from_secs(30))
                .ok()
                .and_then(|l| l.iter().find(|x| x.coin == coin).map(|x| x.created));
            if let Some(h) = seen {
                break h;
            }
            if std::time::Instant::now() > until {
                return Err("the listing isn't on the chain after 15 minutes".into());
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        };
        listed.confirmed = true;
        save(&dir, &listed)?;
        // Where the time went: blocks until it was taken in, then seen.
        log::info!(
            "world {} is listed: in block {created}, {} blocks after the push, seen {:.0} s after it",
            did.did(),
            created.saturating_sub(pushed_at),
            pushed.elapsed().as_secs_f32()
        );
        set("listing.listed", &[txch(deposit)]);
        Ok(())
    });
}

/// Delist the world in `dir`: the deposit comes back.
pub fn delist(settings: &crate::settings::Settings, dir: PathBuf) {
    let settings = settings.clone();
    run(move || {
        let listed = load(&dir).ok_or("this world isn't listed")?;
        let coin = listed.coin().ok_or("a damaged listing.json")?;
        let reader = crate::chiacraft::chain::reader(&settings)?;
        let network = reader.network();
        crate::chiacraft::chain::may_spend(network)?;
        set("listing.delisting", &[]);
        if !reader.wait_synced(std::time::Duration::from_secs(60)) {
            return Err("the wallet isn't synced with the chain".into());
        }
        let keys = wallet_keys(network)?;
        let (push, ()) = push_while_full(|| {
            reader
                .delist_world(&keys, coin, 0, std::time::Duration::from_secs(60))
                .map(|p| (p, ()))
                .map_err(|e| e.to_string())
        })?;
        if matches!(push, chiacraft_wallet::chain::PushOutcome::Rejected { .. }) {
            return Err(format!("the chain refused it: {push:?}"));
        }
        let _ = std::fs::remove_file(dir.join(FILE));
        log::info!("world delisted; the deposit of {} TXCH comes back", txch(coin.amount));
        set("listing.delisted", &[txch(coin.amount)]);
        Ok(())
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposits_show_in_txch() {
        assert_eq!(txch(100_000_000), "0.0001");
        assert_eq!(txch(1_000_000_000_000), "1");
        assert_eq!(txch(1), "0.000000000001");
    }

    #[test]
    fn the_file_keeps_the_coin() {
        let l = Listed { parent: "11".repeat(32), puzzle_hash: "22".repeat(32), amount: 5, did: "33".repeat(32), confirmed: false };
        let c = l.coin().unwrap();
        assert_eq!((c.parent_coin_info, c.puzzle_hash, c.amount), (Bytes32::new([0x11; 32]), Bytes32::new([0x22; 32]), 5));
    }

    /// finder3 (11-04): the listing was refused for want of a fee while testnet11's mempool was
    /// full. That refusal is pushed again; any other is final, and so is a spend that went in.
    #[test]
    fn only_a_full_mempool_is_tried_again() {
        use chiacraft_wallet::chain::PushOutcome;
        let txid = Bytes32::new([0; 32]);
        let rejected = |e: &str| PushOutcome::Rejected { txid, error: e.into() };
        assert!(mempool_full(&rejected("INVALID_FEE_TOO_CLOSE_TO_ZERO")));
        assert!(!mempool_full(&rejected("DOUBLE_SPEND")));
        assert!(!mempool_full(&PushOutcome::Accepted { txid }));
        let mut pushes = 0;
        let out = push_while_full(|| {
            pushes += 1;
            Ok((rejected("DOUBLE_SPEND"), ()))
        });
        assert!(matches!(out, Ok((PushOutcome::Rejected { .. }, ()))));
        assert_eq!(pushes, 1, "a final refusal is not pushed again");
    }
}
