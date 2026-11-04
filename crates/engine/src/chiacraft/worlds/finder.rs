//! The finder (`notes/PLAN.md`, phase 5, slice 2): the worlds listed on chain, checked, with
//! what each says about itself now. On a thread of its own: the listings from any full node
//! (unspent, at least the deposit, `chain::listing`), each kept only if its world DID and its
//! owner's DID name each other the way the listing says (`ChainReader::check_listing`), then each
//! world asked for its status by the key the listing names.

use std::time::Duration;

/// A listed world as the finder shows it.
#[derive(Clone, Debug)]
pub struct Found {
    /// The world's key as the network writes it: its address.
    pub key: String,
    pub world_did: String,
    pub owner_did: String,
    pub deposit: u64,
    /// The height the listing was made at, and the chain's height when read.
    pub listed_at: u32,
    pub peak: u32,
    /// What the world answered, if it did.
    pub status: Option<crate::net::p2p::Status>,
    pub asked: bool,
}

impl Found {
    /// Days since it was listed.
    pub fn days(&self) -> u32 {
        self.peak.saturating_sub(self.listed_at) / crate::chiacraft::chain::BLOCKS_A_DAY
    }
}

#[derive(Clone, Debug, Default)]
pub struct Finder {
    /// What it is doing: a lang key and its arguments.
    pub phase: (String, Vec<String>),
    pub worlds: Vec<Found>,
    /// Listings left out because they didn't check out.
    pub left_out: usize,
}

static FINDER: parking_lot::Mutex<Option<Finder>> = parking_lot::Mutex::new(None);
static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Raised as the finder learns something: its screen shows it.
pub static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The most listings read, and the most checked and asked.
const MAX_READ: usize = 500;
const MAX_SHOWN: usize = 100;

fn update(f: impl FnOnce(&mut Finder)) {
    f(FINDER.lock().get_or_insert_with(Default::default));
    GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

pub fn snapshot() -> Finder {
    FINDER.lock().clone().unwrap_or_default()
}

/// Read the listings again, in the background (once at a time).
pub fn refresh(settings: &crate::settings::Settings) {
    if BUSY.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let settings = settings.clone();
    let _ = std::thread::Builder::new()
        .name("finder".into())
        .spawn(move || {
            if let Err(e) = find(&settings) {
                log::warn!("finder: {e}");
                update(|f| f.phase = ("finder.failed".into(), vec![e]));
            }
            BUSY.store(false, std::sync::atomic::Ordering::SeqCst);
        });
}

fn find(settings: &crate::settings::Settings) -> Result<(), String> {
    update(|f| f.phase = ("finder.reading".into(), Vec::new()));
    let reader = crate::chiacraft::chain::reader(settings)?;
    if !reader.wait_synced(Duration::from_secs(60)) && !reader.reachable() {
        return Err("no chain peer answers".into());
    }
    let deposit = settings.listing_deposit();
    let listings = reader
        .listings(deposit, MAX_READ, Duration::from_secs(60))
        .map_err(|e| e.to_string())?;
    let peak = reader.status().peak.unwrap_or(0);
    update(|f| f.phase = ("finder.checking".into(), vec![listings.len().to_string()]));
    let mut worlds = Vec::new();
    let mut left_out = 0;
    for l in listings.iter().take(MAX_SHOWN) {
        if !reader
            .check_listing(l, Duration::from_secs(20))
            .unwrap_or(false)
        {
            left_out += 1;
            continue;
        }
        let Ok(key) = iroh::PublicKey::from_bytes(&l.world.key) else {
            left_out += 1;
            continue;
        };
        let enc = |b: [u8; 32]| chiacraft_wallet::keys::encode_did(&b).unwrap_or_default();
        worlds.push(Found {
            key: key.to_string(),
            world_did: enc(l.world_did.to_bytes()),
            owner_did: enc(l.world.owner.to_bytes()),
            deposit: l.coin.amount,
            listed_at: l.created,
            peak,
            status: None,
            asked: false,
        });
    }
    // Several listings of one world (a world listed twice): the newest only.
    worlds.sort_by_key(|w| std::cmp::Reverse(w.listed_at));
    let mut seen = std::collections::HashSet::new();
    worlds.retain(|w| seen.insert(w.key.clone()));
    log::info!("finder: {} listed worlds check out, {left_out} left out", worlds.len());
    let n = worlds.len();
    update(|f| {
        f.worlds = worlds.clone();
        f.left_out = left_out;
        f.phase = ("finder.asking".into(), vec![n.to_string()]);
    });
    // Each world's status, a few at a time.
    for chunk in worlds.chunks(8) {
        let asks: Vec<_> = chunk
            .iter()
            .map(|w| (w.key.clone(), crate::net::p2p::ping(&w.key, settings)))
            .collect();
        for (key, rx) in asks {
            let st = rx
                .recv_timeout(Duration::from_secs(8))
                .ok()
                .and_then(Result::ok);
            update(|f| {
                if let Some(w) = f.worlds.iter_mut().find(|w| w.key == key) {
                    w.status = st;
                    w.asked = true;
                }
            });
        }
    }
    let online = snapshot()
        .worlds
        .iter()
        .filter(|w| w.status.is_some())
        .count();
    log::info!("finder: {online} of {n} listed worlds online");
    update(|f| f.phase = ("finder.done".into(), vec![n.to_string(), online.to_string()]));
    Ok(())
}
