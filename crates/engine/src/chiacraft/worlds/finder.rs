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
    /// This game's own worlds whose listing is pushed but not on the chain yet (their names):
    /// listed, pending. Another game can't see a listing before the chain has it.
    pub pending: Vec<String>,
}

/// Listings checked lately (`check_listing`), by coin id: whether they checked out, and when. A
/// listing's coin doesn't change, so it isn't checked again on every refresh (two DID reads each,
/// one listing after another: the finder took minutes, the join test 11-05).
static CHECKED: parking_lot::Mutex<Option<Checked>> = parking_lot::Mutex::new(None);
/// Each listing's coin id: whether it checked out, and when.
type Checked = std::collections::HashMap<[u8; 32], (bool, std::time::Instant)>;
const CHECKED_FOR: Duration = Duration::from_secs(600);

/// While the screen is open (each build pushes this on), the finder reads again every
/// `EVERY`, so a newly listed world shows up without pressing Refresh.
static WATCH_UNTIL: parking_lot::Mutex<Option<std::time::Instant>> = parking_lot::Mutex::new(None);
const EVERY: Duration = Duration::from_secs(60);
const WATCH: Duration = Duration::from_secs(180);

/// The screen is showing: keep reading again every minute for a while.
pub fn watch() {
    *WATCH_UNTIL.lock() = Some(std::time::Instant::now() + WATCH);
}

fn watched() -> bool {
    WATCH_UNTIL
        .lock()
        .is_some_and(|t| std::time::Instant::now() < t)
}

static FINDER: parking_lot::Mutex<Option<Finder>> = parking_lot::Mutex::new(None);
static BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Refresh pressed while the finder waits for its next read.
static KICK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
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
        // Reading, or waiting for its next read: that one comes now.
        KICK.store(true, std::sync::atomic::Ordering::SeqCst);
        return;
    }
    let settings = settings.clone();
    let _ = std::thread::Builder::new()
        .name("finder".into())
        .spawn(move || {
            loop {
                if let Err(e) = find(&settings) {
                    log::warn!("finder: {e}");
                    update(|f| f.phase = ("finder.failed".into(), vec![e]));
                }
                // Again in a minute while the screen is open, or at once on Refresh.
                let next = std::time::Instant::now() + EVERY;
                while std::time::Instant::now() < next && !KICK.swap(false, std::sync::atomic::Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(250));
                }
                if !watched() {
                    break;
                }
            }
            BUSY.store(false, std::sync::atomic::Ordering::SeqCst);
        });
}

/// How long a read waits for a chain peer to connect (a fresh reader has none for its first
/// seconds). Reading listings needs a peer, not the wallet synced: waiting for that (up to a
/// minute) only held the finder up.
const PEER_WAIT: Duration = Duration::from_secs(20);

fn find(settings: &crate::settings::Settings) -> Result<(), String> {
    let started = std::time::Instant::now();
    if snapshot().worlds.is_empty() {
        update(|f| f.phase = ("finder.reading".into(), Vec::new()));
    }
    let reader = crate::chiacraft::chain::reader(settings)?;
    let until = std::time::Instant::now() + PEER_WAIT;
    while !reader.reachable() && std::time::Instant::now() < until {
        std::thread::sleep(Duration::from_millis(250));
    }
    if !reader.reachable() {
        return Err("no chain peer answers".into());
    }
    let deposit = crate::settings::listing_deposit();
    let listings = reader
        .listings(deposit, MAX_READ, Duration::from_secs(60))
        .map_err(|e| e.to_string())?;
    let read_in = started.elapsed();
    // This game's own listings the chain doesn't show yet: pending, shown at once.
    let on_chain: std::collections::HashSet<[u8; 32]> = listings
        .iter()
        .map(|l| l.coin.coin_id().to_bytes())
        .collect();
    let pending: Vec<String> = crate::saves::list(&settings.saves_dir())
        .into_iter()
        .filter_map(|m| {
            let l = super::listing::load(&m.dir)?;
            let coin = l.coin()?;
            (!on_chain.contains(&coin.coin_id().to_bytes())).then_some(m.name)
        })
        .collect();
    update(|f| f.pending = pending.clone());
    let peak = reader.status().peak.unwrap_or(0);
    if snapshot().worlds.is_empty() {
        update(|f| f.phase = ("finder.checking".into(), vec![listings.len().to_string()]));
    }
    // Each listing checked (two DID reads), eight at a time; ones checked lately aren't again.
    let shown: Vec<_> = listings.iter().take(MAX_SHOWN).copied().collect();
    let mut ok = vec![false; shown.len()];
    let now = std::time::Instant::now();
    {
        let mut cache = CHECKED.lock();
        let cache = cache.get_or_insert_with(Default::default);
        cache.retain(|_, (_, at)| now.duration_since(*at) < CHECKED_FOR);
        for (i, l) in shown.iter().enumerate() {
            if let Some((good, _)) = cache.get(&l.coin.coin_id().to_bytes()) {
                ok[i] = *good;
            }
        }
    }
    let todo: Vec<usize> = (0..shown.len())
        .filter(|&i| {
            !CHECKED
                .lock()
                .as_ref()
                .is_some_and(|c| c.contains_key(&shown[i].coin.coin_id().to_bytes()))
        })
        .collect();
    for chunk in todo.chunks(8) {
        let answers: Vec<(usize, bool)> = std::thread::scope(|scope| {
            let handles: Vec<_> = chunk
                .iter()
                .map(|&i| {
                    let (reader, l) = (&reader, &shown[i]);
                    scope.spawn(move || {
                        (
                            i,
                            reader
                                .check_listing(l, Duration::from_secs(20))
                                .unwrap_or(false),
                        )
                    })
                })
                .collect();
            handles.into_iter().filter_map(|h| h.join().ok()).collect()
        });
        let mut cache = CHECKED.lock();
        let cache = cache.get_or_insert_with(Default::default);
        for (i, good) in answers {
            ok[i] = good;
            cache.insert(shown[i].coin.coin_id().to_bytes(), (good, std::time::Instant::now()));
        }
    }
    let checked_in = started.elapsed() - read_in;
    let mut worlds = Vec::new();
    let mut left_out = 0;
    for (i, l) in shown.iter().enumerate() {
        if !ok[i] {
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
    log::info!(
        "finder: {} listed worlds check out, {left_out} left out (read in {:.1} s, checked in {:.1} s, {} checked anew)",
        worlds.len(),
        read_in.as_secs_f32(),
        checked_in.as_secs_f32(),
        todo.len()
    );
    let n = worlds.len();
    // Keep what each world said last time until it answers again, so the list doesn't blink.
    let before = snapshot().worlds;
    for w in &mut worlds {
        if let Some(b) = before.iter().find(|b| b.key == w.key) {
            w.status = b.status.clone();
            w.asked = b.asked;
        }
    }
    update(|f| {
        f.worlds = worlds.clone();
        f.left_out = left_out;
        f.pending = pending;
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
    log::info!("finder: {online} of {n} listed worlds online ({:.1} s in all)", started.elapsed().as_secs_f32());
    update(|f| f.phase = ("finder.done".into(), vec![n.to_string(), online.to_string()]));
    Ok(())
}
