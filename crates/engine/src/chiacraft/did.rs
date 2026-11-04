//! The account's DID in the game (`notes/PLAN.md`, phase 4, slice 5): getting one from a sponsor,
//! linking one the account's words already hold, and watching it until the chain has it. Each
//! runs on a thread of its own; the Account screen shows [`state`].
//!
//! Getting one: the sponsors in `core:sponsors` are asked in turn. Each first says how much proof
//! of work it wants (its status), then gets the request: the account's first wallet address on the
//! chain's network, signed by the identity key. The answer names the DID, which is linked at once
//! (the sponsor launched it to that address, so its key is the owner) and watched until the chain
//! shows it held there (about a minute).

use chiacraft_wallet::chain::{self, Bytes32};

/// What the DID is doing, for the Account screen: a lang key and its text arguments.
static STATE: parking_lot::Mutex<(String, Vec<String>)> = parking_lot::Mutex::new((String::new(), Vec::new()));
/// A thread is at work (no second one starts).
static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn set(key: &str, args: &[String]) {
    *STATE.lock() = (key.to_string(), args.to_vec());
    crate::admission::GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// The DID's state, as a lang key and arguments (empty key: nothing to say yet).
pub fn state() -> (String, Vec<String>) {
    STATE.lock().clone()
}

/// Runs `f` on a thread of its own unless one is at work already.
fn work(f: impl FnOnce() -> Result<(), String> + Send + 'static) {
    if BUSY.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("did".into())
        .spawn(move || {
            if let Err(e) = f() {
                log::warn!("DID: {e}");
                set("account.did.failed", &[e]);
            }
            BUSY.store(false, std::sync::atomic::Ordering::SeqCst);
        });
}

/// The account's first wallet address on the chain's network, and its puzzle hash.
fn first_address(settings: &crate::settings::Settings) -> Result<(String, [u8; 32]), String> {
    let session = chiacraft_account::current().ok_or("no account is open")?;
    let network = crate::chiacraft::chain::network(&settings.chain())?;
    let ph = session
        .first_puzzle_hash()
        .ok_or("the account has no wallet address")?;
    Ok((chain::address(Bytes32::from(ph), network), ph))
}

/// Ask the sponsors for a DID, link it, and watch it reach the chain.
pub fn get(settings: &crate::settings::Settings) {
    let settings = settings.clone();
    work(move || {
        let session = chiacraft_account::current().ok_or("no account is open")?;
        let (address, owner) = first_address(&settings)?;
        let network = crate::chiacraft::chain::network(&settings.chain())?;
        let sponsors = settings.sponsors();
        if sponsors.is_empty() {
            return Err("no DID sponsor is set (setting core:sponsors)".into());
        }
        let mut last = String::new();
        for sponsor in &sponsors {
            set("account.did.asking", &[short(sponsor)]);
            // How much work it wants, from its status (none if it doesn't say).
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
            set("account.did.working", &[]);
            let mut request = session.sponsor_request(network.name(), &address)?;
            request.do_work(bits).map_err(|e| e.to_string())?;
            set("account.did.asking", &[short(sponsor)]);
            let answer = crate::net::p2p::ask_sponsor(&settings, sponsor, Some(&request))
                .and_then(|(status, body)| chiacraft_wallet::sponsor::read_answer(status, &body).map_err(|e| e.to_string()));
            match answer {
                Ok(reply) => {
                    let launcher = reply.launcher_id().map_err(|e| e.to_string())?;
                    session.link_did(launcher, owner)?;
                    log::info!("DID {} from sponsor {} ({})", reply.did, short(sponsor), reply.status);
                    return watch_now(&settings, launcher, owner);
                }
                Err(e) => last = format!("{}: {e}", short(sponsor)),
            }
        }
        Err(last)
    });
}

/// Link a DID the account's words hold (`did:chia:1…`): the chain says which address holds it.
pub fn link(settings: &crate::settings::Settings, did: &str) {
    let (settings, did) = (settings.clone(), did.trim().to_string());
    work(move || {
        let session = chiacraft_account::current().ok_or("no account is open")?;
        let launcher = chain::parse_did(&did).map_err(|e| e.to_string())?;
        set("account.did.looking", &[]);
        let reader = crate::chiacraft::chain::reader(&settings)?;
        let (record, _) = crate::chiacraft::chain::did(&reader, &did)?;
        // Its owner: as the chain shows it, or for a DID never spent (a sponsored one), the
        // address of these words its launch commits to.
        let owner = record
            .owner()
            .map(|o| o.to_bytes())
            .or_else(|| {
                session
                    .puzzle_hashes(20)
                    .into_iter()
                    .find(|ph| record.eve_owned_by(Bytes32::from(*ph)))
            })
            .ok_or("that DID isn't held by an address of this account")?;
        session.link_did(launcher.to_bytes(), owner)?;
        watch_now(&settings, launcher.to_bytes(), owner)
    });
}

/// Watch the linked DID until the chain shows it held by its address (a few minutes at most).
pub fn watch(settings: &crate::settings::Settings) {
    let settings = settings.clone();
    let Some(link) = chiacraft_account::current().and_then(|s| s.did()) else { return };
    let Some(launcher) = link.launcher_id() else { return };
    work(move || {
        let session = chiacraft_account::current().ok_or("no account is open")?;
        let owner = session
            .puzzle_hashes(20)
            .into_iter()
            .nth(if link.hardened { 20 + link.index as usize } else { link.index as usize })
            .ok_or("the DID's address is beyond the first 20")?;
        watch_now(&settings, launcher, owner)
    });
}

fn watch_now(settings: &crate::settings::Settings, launcher: [u8; 32], owner: [u8; 32]) -> Result<(), String> {
    let did = chiacraft_wallet::keys::encode_did(&launcher).unwrap_or_default();
    set("account.did.pending", &[short_did(&did)]);
    let reader = crate::chiacraft::chain::reader(settings)?;
    let record = reader
        .wait_did(Bytes32::from(launcher), std::time::Duration::from_secs(10), std::time::Duration::from_secs(600))
        .map_err(|e| format!("{} isn't on the chain yet: {e}", short_did(&did)))?;
    if !record.owned_by(Bytes32::from(owner)) {
        return Err(format!("{} is on the chain but held by another address", short_did(&did)));
    }
    let peak = reader.status().peak.unwrap_or(0).max(record.checked_height);
    let days = record
        .launched()
        .map_or(0, |h| peak.saturating_sub(h) / crate::chiacraft::chain::BLOCKS_A_DAY);
    log::info!("DID {did} is on the chain, held by this account ({days} days old)");
    set("account.did.confirmed", &[short_did(&did), days.to_string()]);
    Ok(())
}

fn short(s: &str) -> String {
    s.chars().take(12).collect()
}

fn short_did(d: &str) -> String {
    let c: Vec<char> = d.chars().collect();
    if c.len() > 24 {
        format!("{}…{}", c[..16].iter().collect::<String>(), c[c.len() - 6..].iter().collect::<String>())
    } else {
        d.to_string()
    }
}
