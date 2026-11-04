//! Peer-to-peer transport over iroh: players are identified by a key (not an address), so VPNs,
//! restarts and port changes do not matter. Connections go direct when NAT allows and through a
//! relay otherwise. The bridge threads convert between QUIC streams and the engine's channels.

use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

use crossbeam_channel::unbounded;
use iroh::endpoint::presets;
use iroh::{Endpoint, EndpointAddr, EndpointId, RelayMode, RelayUrl, SecretKey, TransportAddr};
use parking_lot::Mutex;

use super::codec;
use super::{ClientConn, Conn, ServerConn, ToClient, ToServer};
use crate::settings::Settings;

const ALPN: &[u8] = b"weft/1";
/// A world's status, for lists of worlds (chiacraft): answered without joining.
const STATUS_ALPN: &[u8] = b"chiacraft/status/1";
/// Link requests and answers between owners' account endpoints (chiacraft, `links`).
const LINKS_ALPN: &[u8] = b"chiacraft/links/1";
/// A player's worlds, for their friends, and where their worlds moved, for anyone (`friends`).
const WORLDS_ALPN: &[u8] = b"chiacraft/worlds/1";
/// Ban lists a world holds, its own and the ones it follows, for anyone (`admission`).
const BANS_ALPN: &[u8] = b"chiacraft/bans/1";
/// 3: `Join` carries the account's vouching (chiacraft). 4: the hello offers the world's data
/// packages, which the joining game downloads and loads before it is let in (chiacraft). 5: the
/// hello may carry a travel ticket from a linked world's portal (chiacraft, `travel`). 6: an offer
/// says whether the package carries sandboxed code (chiacraft, phase 6). 7: a world refuses with a
/// reason in the player's language (`ToClient::Refused`), and its status says what its admission
/// rules ask (chiacraft, `admission`). 8: an offer says which mod API its code needs instead of
/// only whether it has code (empty: no code; chiacraft, phase 6). 9: worlds send players their
/// signed achievements (`ToClient::Achievement`; chiacraft, phase 4). 10: a world asks a joining
/// game for an achievement its rules need (`AskAchievement`, `Proofs`; chiacraft, phase 4). 11: a
/// world sends its issuer's revocation list (`ToClient::Revocations`; chiacraft, phase 4). 12: a
/// time-of-day rate per dimension (`ToClient::Time`). 13: a moving structure leans
/// (`ToClient::BodyLean`; chiacraft, falling trees).
const PROTOCOL: u32 = 13;
const MAX_FRAME: usize = 64 << 20;

/// A data package the host's world plays with: id, version, hash (hex), size in bytes, its
/// signature envelope (empty if unsigned), and the mod API its sandboxed code
/// (`code/server.wasm`, `code/client.wasm`) needs, as `1.1` (empty: no code). The joining game refuses an API it
/// doesn't run before downloading, and checks there is code exactly when the manifest says so.
pub type Offer = (String, String, String, u64, String, String);

/// Requests on the extra streams a joining game opens to download packages.
const GET_MANIFEST: u8 = 0;
const GET_FILE: u8 = 1;
const MAX_MANIFEST: u64 = 1 << 20;

/// Limits on joining (chiacraft): the hello and the ready are small; a player has this long to
/// download the world's packages and load them; the host serves this many players' downloads at
/// once, this many requests of each at once.
const MAX_HELLO: usize = 2 << 10;
const MAX_OFFER: usize = 1 << 20;
const CONFIGURE_TIME: Duration = Duration::from_secs(15 * 60);
const MAX_CONFIGURING: usize = 8;
const PEER_STREAMS: usize = 4;
/// A world offers at most this many packages, this much in all.
const MAX_OFFERS: usize = 256;
const MAX_JOIN_BYTES: u64 = 1 << 30;
/// A download that sends nothing for this long has stalled.
const STALL: Duration = Duration::from_secs(30);

fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("net")
            .enable_all()
            .build()
            .expect("tokio runtime")
    })
}

/// This player's network identity: the key of the account's words (`chiacraft_account`), so the
/// same words give the same key, and the same player, on any computer. Servers key player saves
/// by it. The account is made quietly on the first start; one that cannot be read is an error and
/// is never replaced, because a new key would be a new player everywhere.
pub fn identity(run_dir: &Path) -> Result<SecretKey, String> {
    let session = chiacraft_account::open(run_dir)?;
    Ok(SecretKey::from_bytes(session.network_seed()))
}

/// The player's identity for servers, or empty (keyed by name) when there is none.
pub fn player_key(run_dir: &Path) -> String {
    identity(run_dir)
        .map(|k| k.public().to_string())
        .unwrap_or_else(|e| {
            log::error!("{e}; this world keys the player by name");
            String::new()
        })
}

/// Hash over the code mods (id and version): what a joining game must bring itself. Its data
/// packages it can download.
pub fn native_hash() -> u64 {
    let mut h = weft_noise::FNV;
    for m in crate::content::content()
        .mods
        .iter()
        .filter(|m| m.package.is_none())
    {
        for b in m.id.bytes().chain([0]).chain(m.version.bytes()).chain([0]) {
            h = weft_noise::fnv_step(h, b as u64);
        }
    }
    h
}

/// Hash over mods and every registered name: equal hash means equal runtime ids.
pub fn content_hash() -> u64 {
    let c = crate::content::content();
    let mut h = weft_noise::FNV;
    let mut eat = |s: &str| {
        for b in s.bytes().chain([0]) {
            h = weft_noise::fnv_step(h, b as u64);
        }
    };
    for m in &c.mods {
        eat(&m.id);
        eat(&m.version);
        eat(m.package.as_deref().unwrap_or(""));
    }
    for b in &c.blocks {
        eat(&b.name);
        eat(&b.states.to_string());
    }
    for i in &c.items {
        eat(&i.name);
    }
    for e in &c.game.entities {
        eat(&e.name);
    }
    for m in &c.game.menus {
        eat(&m.name);
    }
    for k in &c.game.components {
        eat(&k.name);
    }
    // Everything else that crosses the wire by id.
    for n in c
        .biomes
        .iter()
        .map(|b| &b.name)
        .chain(c.dims.iter().map(|d| &d.name))
    {
        eat(n);
    }
    for n in c
        .game
        .sounds
        .iter()
        .map(|s| &s.name)
        .chain(c.game.particles.iter().map(|p| &p.name))
        .chain(c.game.configs.iter().map(|k| &k.name))
    {
        eat(n);
    }
    for a in &c.game.attributes {
        eat(&a.name);
    }
    h
}

fn mod_list() -> String {
    crate::content::content()
        .mods
        .iter()
        .filter(|m| m.package.is_none())
        .map(|m| format!("{} {}", m.id, m.version))
        .collect::<Vec<_>>()
        .join(", ")
}

fn relay_mode(settings: &Settings) -> RelayMode {
    let custom: Vec<RelayUrl> = settings
        .relays()
        .iter()
        .filter_map(|r| r.parse().ok())
        .collect();
    if custom.is_empty() { RelayMode::Default } else { RelayMode::custom(custom) }
}

/// Whose endpoint (chiacraft): the player's (the account's network key), a hosted world's (its own
/// key), or nobody's (a throwaway key: asking a world for its status tells it nothing about who
/// asks, and never takes the relay from the player's own endpoint).
enum Who {
    Player,
    World(Box<iroh::SecretKey>),
    Anonymous,
}

/// An endpoint. Worlds are found by key through n0's DNS and the BitTorrent Mainline DHT, so
/// neither is needed alone; a hosted world publishes to both (its relay only, never its IPs).
async fn endpoint(settings: &Settings, who: Who) -> Result<Endpoint, String> {
    use iroh_mainline_address_lookup::DhtAddressLookup;
    let b = Endpoint::builder(presets::N0).relay_mode(relay_mode(settings));
    let b = match who {
        Who::Player => b
            .secret_key(identity(&settings.run_dir)?)
            .alpns(vec![LINKS_ALPN.to_vec(), WORLDS_ALPN.to_vec()])
            .address_lookup(DhtAddressLookup::builder()),
        Who::World(key) => b
            .secret_key(*key)
            .alpns(vec![ALPN.to_vec(), STATUS_ALPN.to_vec(), BANS_ALPN.to_vec()])
            .address_lookup(DhtAddressLookup::builder()),
        Who::Anonymous => b.secret_key(throwaway_key()),
    };
    b.bind().await.map_err(|e| format!("network: {e}"))
}

/// A key nobody keeps, from the system's randomness (each `RandomState` is seeded from it).
fn throwaway_key() -> iroh::SecretKey {
    use std::hash::{BuildHasher, Hasher};
    let mut bytes = [0u8; 32];
    for chunk in bytes.chunks_mut(8) {
        chunk.copy_from_slice(
            &std::collections::hash_map::RandomState::new()
                .build_hasher()
                .finish()
                .to_le_bytes(),
        );
    }
    iroh::SecretKey::from_bytes(&bytes)
}

async fn write_frame(send: &mut iroh::endpoint::SendStream, data: &[u8]) -> Result<(), String> {
    let packed = if data.len() > 256 { lz4_flex::compress_prepend_size(data) } else { data.to_vec() };
    let flag = (data.len() > 256) as u32;
    let header = (packed.len() as u32) << 1 | flag;
    send.write_all(&header.to_le_bytes())
        .await
        .map_err(|e| e.to_string())?;
    send.write_all(&packed).await.map_err(|e| e.to_string())
}

async fn read_frame(recv: &mut iroh::endpoint::RecvStream) -> Result<Vec<u8>, String> {
    read_frame_max(recv, MAX_FRAME).await
}

/// A frame of at most `max` bytes, unpacked: the size a packed frame claims is checked before
/// anything is allocated for it (chiacraft: a few bytes could otherwise claim gigabytes).
async fn read_frame_max(recv: &mut iroh::endpoint::RecvStream, max: usize) -> Result<Vec<u8>, String> {
    let mut h = [0u8; 4];
    recv.read_exact(&mut h).await.map_err(|e| e.to_string())?;
    let header = u32::from_le_bytes(h);
    let len = (header >> 1) as usize;
    if len > max {
        return Err("frame too large".into());
    }
    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf).await.map_err(|e| e.to_string())?;
    if header & 1 == 0 {
        return Ok(buf);
    }
    let unpacked = buf
        .get(..4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()) as usize)
        .ok_or("frame too short")?;
    if unpacked > max {
        return Err("frame too large".into());
    }
    lz4_flex::decompress_size_prepended(&buf).map_err(|e| e.to_string())
}

/// A world's address (`chiacraft://w/...`) or ticket: the key, and the hints that come with it.
fn parse_ticket(t: &str) -> Result<EndpointAddr, String> {
    let p = crate::world_address::parse(t).ok_or("invalid address")?;
    let id: EndpointId = p.key.parse().map_err(|_| "invalid address".to_string())?;
    let mut addrs = Vec::new();
    if let Ok(r) = p.relay.parse::<RelayUrl>() {
        addrs.push(TransportAddr::Relay(r));
    }
    for ip in p.addrs.iter().filter_map(|s| s.parse().ok()) {
        addrs.push(TransportAddr::Ip(ip));
    }
    Ok(EndpointAddr::from_parts(id, addrs))
}

/// The player's own endpoint (the account's network key, chiacraft): open while the game runs. It
/// joins other worlds and is the inbox for link requests to any of the player's worlds. One per
/// key: two endpoints with one key take the relay from each other.
static ACCOUNT: tokio::sync::OnceCell<Endpoint> = tokio::sync::OnceCell::const_new();
static OUTBOX: tokio::sync::Notify = tokio::sync::Notify::const_new();

async fn account(settings: &Settings) -> Result<Endpoint, String> {
    ACCOUNT
        .get_or_try_init(|| async {
            let ep = endpoint(settings, Who::Player).await?;
            let serving = ep.clone();
            tokio::spawn(async move {
                while let Some(incoming) = serving.accept().await {
                    tokio::spawn(async move {
                        let Ok(conn) = incoming.await else { return };
                        if conn.alpn() == WORLDS_ALPN {
                            serve_worlds(conn).await;
                        } else if conn.alpn() == LINKS_ALPN
                            && let Err(e) = serve_links(conn).await
                        {
                            log::warn!("link message: {e}");
                        }
                    });
                }
            });
            let sending = ep.clone();
            tokio::spawn(async move { outbox(sending).await });
            Ok::<Endpoint, String>(ep)
        })
        .await
        .cloned()
}

/// Worlds asked about ahead of a trip (`warm`), and what they answered.
static WARMED: Mutex<Option<std::collections::HashMap<String, (Status, std::time::Instant)>>> = Mutex::new(None);

/// Asks the world with key `key` for its status through the player's own endpoint, in the
/// background: the endpoint then knows the way there, and a trip that follows connects sooner.
pub fn warm(key: &str) {
    let (Some(ep), Ok(addr)) = (ACCOUNT.get().cloned(), parse_ticket(key)) else { return };
    let key = key.to_string();
    runtime().spawn(async move {
        match status_on(&ep, addr, Duration::from_secs(8)).await {
            Ok(st) => {
                log::info!("a portal leads to '{}' ({} playing): on the way", st.name, st.players);
                WARMED
                    .lock()
                    .get_or_insert_with(Default::default)
                    .insert(key, (st, std::time::Instant::now()));
            }
            Err(e) => {
                log::info!("a portal leads to a world that doesn't answer: {e}");
                if let Some(w) = WARMED.lock().as_mut() {
                    w.remove(&key);
                }
            }
        }
    });
}

/// What the world with key `key` answered when warmed in the last minute.
pub fn warmed(key: &str) -> Option<Status> {
    WARMED
        .lock()
        .as_ref()?
        .get(key)
        .filter(|(_, t)| t.elapsed().as_secs() < 60)
        .map(|(s, _)| s.clone())
}

/// Asks the DID sponsor `sponsor` (its endpoint id) through the player's own endpoint: `request`
/// as given, or its status when `None`. Blocks (call it from a thread of its own): the answer's
/// status and body, or why nobody answered.
pub fn ask_sponsor(
    settings: &Settings,
    sponsor: &str,
    request: Option<&chiacraft_wallet::sponsor::Request>,
) -> Result<(u16, String), String> {
    let id: EndpointId = sponsor
        .trim()
        .parse()
        .map_err(|_| format!("'{sponsor}' isn't a sponsor's address"))?;
    let settings = settings.clone();
    let request = request.cloned();
    runtime().block_on(async move {
        let ep = account(&settings).await?;
        match &request {
            Some(r) => chiacraft_sponsor::client::ask(&ep, id, r).await,
            None => chiacraft_sponsor::client::status(&ep, id).await,
        }
    })
}

/// Opens the player's endpoint in the background: link requests can reach this game from now on.
pub fn start_account(settings: &Settings) {
    let settings = settings.clone();
    runtime().spawn(async move {
        match account(&settings).await {
            Ok(ep) => log::info!("inbox open at {}", ep.id()),
            Err(e) => log::warn!("no inbox: {e}"),
        }
    });
}

/// The account endpoint's id: where this player's link answers come.
pub fn account_key(settings: &Settings) -> Result<iroh::SecretKey, String> {
    identity(&settings.run_dir)
}

/// Delivers what waits in the outbox soon (something was queued, or answered).
pub fn wake_outbox() {
    OUTBOX.notify_one();
}

/// A link message from another owner's account endpoint: one frame, answered with an empty frame
/// (taken) or an error.
async fn serve_links(conn: iroh::endpoint::Connection) -> Result<(), String> {
    let peer = conn.remote_id().to_string();
    let (mut send, mut recv) = tokio::time::timeout(Duration::from_secs(10), conn.accept_bi())
        .await
        .map_err(|_| "nothing sent")?
        .map_err(|e| e.to_string())?;
    let frame = tokio::time::timeout(Duration::from_secs(10), read_frame_max(&mut recv, 16 << 10))
        .await
        .map_err(|_| "nothing sent")??;
    let message: crate::links::Message = serde_json::from_slice(&frame).map_err(|e| format!("a message this game can't read: {e}"))?;
    let answer = crate::links::receive(&peer, message);
    if let Err(e) = &answer {
        log::info!("link message from {}: {e}", &peer[..12]);
    }
    write_frame(&mut send, answer.err().unwrap_or_default().as_bytes()).await?;
    let _ = send.finish();
    let _ = tokio::time::timeout(Duration::from_secs(5), conn.closed()).await;
    Ok(())
}

/// Answers `chiacraft/worlds/1`: this player's worlds if the asker is a friend, and where worlds
/// moved.
async fn serve_worlds(conn: iroh::endpoint::Connection) {
    let peer = conn.remote_id().to_string();
    let Ok(Ok((mut send, mut recv))) = tokio::time::timeout(Duration::from_secs(10), conn.accept_bi()).await else { return };
    let _ = tokio::time::timeout(Duration::from_secs(10), read_frame_max(&mut recv, 64)).await;
    let hosted = HOST_ENDPOINT.lock().as_ref().map(|e| e.id().to_string());
    let answer = tokio::task::spawn_blocking(move || crate::friends::answer_for(&peer, hosted))
        .await
        .unwrap_or_default();
    if let Ok(bytes) = serde_json::to_vec(&answer)
        && write_frame(&mut send, &bytes).await.is_ok()
    {
        let _ = send.finish();
        let _ = tokio::time::timeout(Duration::from_secs(5), conn.closed()).await;
    }
}

/// Asks the player at account endpoint `account` (whose identity is `identity`) for their worlds
/// and moves, in the background. Only worlds whose certificates that identity signed, and moves it
/// signed, are kept.
pub fn ask_worlds(
    account: &str,
    identity: &str,
    settings: &Settings,
) -> crossbeam_channel::Receiver<Result<crate::friends::Answer, String>> {
    let (tx, rx) = crossbeam_channel::bounded(1);
    let (account, identity, settings) = (account.to_string(), identity.to_string(), settings.clone());
    runtime().spawn(async move {
        let result = async {
            let ep = self::account(&settings).await?;
            let id: EndpointId = account.parse().map_err(|_| "a bad account")?;
            tokio::time::timeout(Duration::from_secs(10), async {
                let conn = ep
                    .connect(EndpointAddr::from_parts(id, Vec::<TransportAddr>::new()), WORLDS_ALPN)
                    .await
                    .map_err(|e| e.to_string())?;
                let (mut send, mut recv) = conn.open_bi().await.map_err(|e| e.to_string())?;
                write_frame(&mut send, &[]).await?;
                let frame = read_frame_max(&mut recv, 256 << 10).await?;
                conn.close(0u32.into(), b"thanks");
                let mut a: crate::friends::Answer = serde_json::from_slice(&frame).map_err(|e| e.to_string())?;
                a.worlds.retain(|w| {
                    crate::world_address::WorldCert::parse(&w.cert)
                        .ok()
                        .filter(|c| c.world == w.key)
                        .and_then(|c| c.verify())
                        .as_deref()
                        == Some(identity.as_str())
                });
                a.moved
                    .retain(|m| chiacraft_account::check_world_moved(&identity, &m.from, &m.to, m.generation, &m.sig));
                Ok::<_, String>(a)
            })
            .await
            .unwrap_or_else(|_| Err("no answer".into()))
        }
        .await;
        let _ = tx.send(result);
        crate::friends::GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    });
    rx
}

/// Delivers queued link messages, again and again until they arrive (or give up after two weeks).
async fn outbox(ep: Endpoint) {
    loop {
        for (_, q) in crate::links::due() {
            let mut inbox = q.inbox.clone();
            if inbox.is_empty() {
                // Learn the owner's inbox from the world itself.
                let addr = if q.address.is_empty() { q.world.clone() } else { q.address.clone() };
                if let Ok(a) = parse_ticket(&addr)
                    && let Ok(st) = status_on(&ep, a, Duration::from_secs(8)).await
                {
                    inbox = st.inbox;
                }
            }
            let delivered = !inbox.is_empty()
                && deliver(&ep, &inbox, &q.message)
                    .await
                    .inspect_err(|e| log::info!("link message not delivered yet: {e}"))
                    .is_ok();
            crate::links::tried(&q, &inbox, delivered);
        }
        let _ = tokio::time::timeout(Duration::from_secs(20), OUTBOX.notified()).await;
    }
}

async fn deliver(ep: &Endpoint, inbox: &str, message: &crate::links::Message) -> Result<(), String> {
    let id: EndpointId = inbox.parse().map_err(|_| "a bad inbox")?;
    tokio::time::timeout(Duration::from_secs(20), async {
        let conn = ep
            .connect(EndpointAddr::from_parts(id, Vec::<TransportAddr>::new()), LINKS_ALPN)
            .await
            .map_err(|e| e.to_string())?;
        let (mut send, mut recv) = conn.open_bi().await.map_err(|e| e.to_string())?;
        write_frame(&mut send, &serde_json::to_vec(message).map_err(|e| e.to_string())?).await?;
        let answer = read_frame_max(&mut recv, 1024).await?;
        conn.close(0u32.into(), b"thanks");
        if answer.is_empty() { Ok(()) } else { Err(format!("refused: {}", String::from_utf8_lossy(&answer))) }
    })
    .await
    .unwrap_or_else(|_| Err("no answer".into()))
}

pub struct Hosting {
    pub ticket: Option<String>,
    pub error: Option<String>,
}

pub static HOSTING: Mutex<Option<Hosting>> = Mutex::new(None);

/// The hosting endpoint (chiacraft), closed when hosting stops.
static HOST_ENDPOINT: Mutex<Option<Endpoint>> = Mutex::new(None);
/// The settings the hosted world was opened with (its relays), for fetching its ban lists.
static HOST_SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);

/// Starts accepting players for the running server. The ticket appears in `HOSTING` once the
/// endpoint is online.
/// `world` is the content the hosted world plays with: what joining games are checked against,
/// whatever the host's game shows meanwhile (chiacraft: it may be visiting another world).
/// What hosting a world needs besides its server: the world's own key and certificate, its name,
/// and the server's shared state (players, for the status answer).
pub struct HostWorld {
    pub identity: std::sync::Arc<crate::world_address::WorldIdentity>,
    pub name: String,
    pub shared: std::sync::Arc<crate::server::Shared>,
    /// The owner's account endpoint (their inbox for link requests) and their identity's vouching
    /// for it; empty without an account.
    pub inbox: (String, String),
    /// The world's folder: its admission rules (summed up in the status) and ban lists (served).
    pub dir: Option<std::path::PathBuf>,
}

/// Starts hosting under the world's own key: its address appears in `HOSTING` once the endpoint
/// is online. A world already hosted elsewhere (it answers its own address) is not hosted twice:
/// two endpoints with one key take the relay from each other.
pub fn host(connect: crossbeam_channel::Sender<ServerConn>, world: &'static crate::content::Content, hw: HostWorld, settings: &Settings) {
    if HOSTING.lock().is_some() {
        return;
    }
    *HOSTING.lock() = Some(Hosting { ticket: None, error: None });
    *HOST_SETTINGS.lock() = Some(settings.clone());
    let settings = settings.clone();
    runtime().spawn(async move {
        let key = hw.identity.key();
        let fail = |e: String| {
            log::error!("{e}");
            *HOSTING.lock() = Some(Hosting { ticket: None, error: Some(e) });
        };
        if let Ok(addr) = parse_ticket(&key)
            && let Ok(st) = status_of(&settings, addr, Duration::from_secs(4)).await
        {
            return fail(format!("'{}' is already open on another computer (players there: {})", st.name, st.players));
        }
        let ep = match endpoint(&settings, Who::World(Box::new(hw.identity.secret.clone()))).await {
            Ok(e) => e,
            Err(e) => return fail(e),
        };
        *HOST_ENDPOINT.lock() = Some(ep.clone());
        let _ = tokio::time::timeout(Duration::from_secs(8), ep.online()).await;
        let a = ep.addr();
        let ips: Vec<String> = a.ip_addrs().map(|a| a.to_string()).collect();
        let relay = a.relay_urls().next().map(|r| r.to_string());
        let t = crate::world_address::address(&key, &hw.name, relay.as_deref(), &ips);
        log::info!("hosting '{}' at {t}", hw.name);
        if let Some(h) = HOSTING.lock().as_mut() {
            h.ticket = Some(t);
        }
        let hw = std::sync::Arc::new(hw);
        // The ban lists it follows: now, then daily while it is open; a list not fetched yet
        // (its world offline, or not published yet) is asked for again every minute.
        if let Some(dir) = hw.dir.clone() {
            tokio::spawn(async move {
                while HOSTING.lock().is_some() {
                    fetch_lists(&dir).await;
                    let d = dir.clone();
                    let missing = tokio::task::spawn_blocking(move || crate::admission::lists_missing(&d))
                        .await
                        .unwrap_or(false);
                    tokio::time::sleep(Duration::from_secs(if missing { 60 } else { 24 * 3600 })).await;
                }
            });
        }
        while let Some(incoming) = ep.accept().await {
            let (connect, hw, key) = (connect.clone(), hw.clone(), key.clone());
            tokio::spawn(async move {
                let conn = match incoming.await {
                    Ok(c) => c,
                    Err(e) => return log::debug!("incoming connection: {e}"),
                };
                if conn.alpn() == STATUS_ALPN {
                    serve_status(conn, world, &hw).await;
                } else if conn.alpn() == BANS_ALPN {
                    serve_bans(conn, hw.dir.clone()).await;
                } else {
                    watch_path(conn.clone(), "player in");
                    if let Err(e) = accept(conn, connect, world, &key, hw.dir.clone()).await {
                        log::warn!("incoming player: {e}");
                    }
                }
            });
        }
    });
}

/// A world's status, as `chiacraft/status/1` answers: what lists of worlds show without joining.
#[derive(Clone, Debug)]
pub struct Status {
    pub protocol: u32,
    pub name: String,
    /// The owner's certificate for the world's key, checked: the owner's identity (hex), or
    /// empty if it doesn't check out.
    pub owner: String,
    pub cert: String,
    pub players: u32,
    pub packages: u32,
    pub package_bytes: u64,
    /// The owner's account endpoint, where link requests go: only if the owner vouches for it.
    pub inbox: String,
    /// What its admission rules ask, as lang keys with a number (`admission::summary`); empty:
    /// open to all, or a world from before protocol 6.
    pub admission: Vec<(String, i64)>,
    /// Its DID and the owner's DID it names (launcher ids, hex), as the world says: a visitor
    /// checks both ways on chain (`world_did::check`).
    pub did: Option<(String, String)>,
}

async fn serve_status(conn: iroh::endpoint::Connection, world: &'static crate::content::Content, hw: &HostWorld) {
    let Ok(Ok((mut send, _recv))) = tokio::time::timeout(Duration::from_secs(5), conn.accept_bi()).await else { return };
    let packages = crate::content::with(world, crate::modloader::current_packages);
    let players = hw.shared.views.read().len() as u32;
    let answer = (
        (PROTOCOL, hw.name.clone(), hw.identity.cert.text(), players, packages.len() as u32, packages.iter().map(|p| p.3).sum::<u64>()),
        hw.inbox.clone(),
    );
    let admission = hw
        .dir
        .as_deref()
        .map(|d| crate::admission::summary(&crate::admission::load_rules(d)))
        .unwrap_or_default();
    // Its DID and the owner's it names (phase 4), when it has one.
    let did = hw
        .dir
        .as_deref()
        .and_then(crate::chiacraft::worlds::world_did::load)
        .map(|d| (d.launcher, d.owner))
        .unwrap_or_default();
    let mut w = codec::W::default();
    codec::Wire::put(&answer, &mut w);
    // After the rest, so a game from before protocol 6 still reads the start.
    codec::Wire::put(&admission, &mut w);
    codec::Wire::put(&did, &mut w);
    if write_frame(&mut send, &w.0).await.is_ok() {
        let _ = send.finish();
        // Wait for the asker to read it and close.
        let _ = tokio::time::timeout(Duration::from_secs(5), conn.closed()).await;
    }
}

async fn status_of(settings: &Settings, addr: EndpointAddr, wait: Duration) -> Result<Status, String> {
    let ep = endpoint(settings, Who::Anonymous).await?;
    let result = status_on(&ep, addr, wait).await;
    ep.close().await;
    result
}

async fn status_on(ep: &Endpoint, addr: EndpointAddr, wait: Duration) -> Result<Status, String> {
    tokio::time::timeout(wait, async {
        let conn = ep
            .connect(addr.clone(), STATUS_ALPN)
            .await
            .map_err(|e| e.to_string())?;
        let (mut send, mut recv) = conn.open_bi().await.map_err(|e| e.to_string())?;
        // A stream is only seen by the other side once something is written on it.
        write_frame(&mut send, &[]).await?;
        let frame = read_frame_max(&mut recv, 8 << 10).await?;
        conn.close(0u32.into(), b"thanks");
        let mut r = codec::R(&frame);
        let ((protocol, name, cert, players, packages, package_bytes), (inbox, vouch)) =
            <((u32, String, String, u32, u32, u64), (String, String)) as codec::Wire>::get(&mut r)
                .ok_or("a status this game can't read")?;
        let mut admission = <Vec<(String, i64)> as codec::Wire>::get(&mut r).unwrap_or_default();
        // Only keys this game knows, so a world can't make a list say anything.
        admission.retain(|(k, _)| k.starts_with("admission.summary."));
        admission.truncate(8);
        let (did, owner_did) = <(String, String) as codec::Wire>::get(&mut r).unwrap_or_default();
        let hex64 = |s: &str| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit());
        let did = (hex64(&did) && hex64(&owner_did)).then_some((did, owner_did));
        // The certificate must be for the key that answered, and the inbox the owner's.
        let owner = crate::world_address::WorldCert::parse(&cert)
            .ok()
            .filter(|c| c.world == addr.id.to_string())
            .and_then(|c| c.verify())
            .unwrap_or_default();
        let inbox = if !owner.is_empty() && chiacraft_account::check_vouch(&vouch, &inbox).as_deref() == Some(owner.as_str()) {
            inbox
        } else {
            String::new()
        };
        Ok::<Status, String>(Status {
            protocol,
            name: name.chars().take(64).collect(),
            owner,
            cert,
            players,
            packages,
            package_bytes,
            inbox,
            admission,
            did,
        })
    })
    .await
    .unwrap_or_else(|_| Err("no answer".into()))
}

/// Asks the world at `address` for its status, in the background: the answer arrives on the
/// channel (within about five seconds).
pub fn ping(address: &str, settings: &Settings) -> crossbeam_channel::Receiver<Result<Status, String>> {
    let (tx, rx) = crossbeam_channel::bounded(1);
    let settings = settings.clone();
    match parse_ticket(address) {
        Ok(addr) => {
            runtime().spawn(async move {
                let _ = tx.send(status_of(&settings, addr, Duration::from_secs(5)).await);
            });
        }
        Err(e) => {
            let _ = tx.send(Err(e));
        }
    }
    rx
}

/// Answers `chiacraft/bans/1`: the asker names a maintainer (identity hex), the world answers
/// with the copy of their ban list it holds, or nothing. Lists are signed, so it serves any.
async fn serve_bans(conn: iroh::endpoint::Connection, dir: Option<std::path::PathBuf>) {
    let Ok(Ok((mut send, mut recv))) = tokio::time::timeout(Duration::from_secs(5), conn.accept_bi()).await else { return };
    let Ok(Ok(asked)) = tokio::time::timeout(Duration::from_secs(5), read_frame_max(&mut recv, 256)).await else { return };
    let maintainer = String::from_utf8_lossy(&asked).into_owned();
    let list = match dir {
        Some(d) => tokio::task::spawn_blocking(move || crate::admission::held_list(&d, &maintainer))
            .await
            .ok()
            .flatten()
            .unwrap_or_default(),
        None => Vec::new(),
    };
    if write_frame(&mut send, &list).await.is_ok() {
        let _ = send.finish();
        let _ = tokio::time::timeout(Duration::from_secs(5), conn.closed()).await;
    }
}

/// Asks the world at `address` for `maintainer`'s ban list.
async fn ask_list(ep: &Endpoint, address: &str, maintainer: &str) -> Result<crate::admission::BanList, String> {
    let addr = parse_ticket(address)?;
    tokio::time::timeout(Duration::from_secs(15), async {
        let conn = ep
            .connect(addr, BANS_ALPN)
            .await
            .map_err(|e| e.to_string())?;
        let (mut send, mut recv) = conn.open_bi().await.map_err(|e| e.to_string())?;
        write_frame(&mut send, maintainer.as_bytes()).await?;
        let frame = read_frame_max(&mut recv, crate::admission::MAX_LIST_BYTES).await?;
        conn.close(0u32.into(), b"thanks");
        if frame.is_empty() {
            return Err("it holds no such list".to_string());
        }
        let list = crate::admission::BanList::parse(&frame)?;
        if list.maintainer != maintainer {
            return Err("it sent someone else's list".into());
        }
        Ok(list)
    })
    .await
    .unwrap_or_else(|_| Err("no answer".into()))
}

/// Fetches the ban lists the world in `dir` follows from where each is served, keeping each
/// newer version that its maintainer signed.
async fn fetch_lists(dir: &std::path::Path) {
    let rules = crate::admission::load_rules(dir);
    if rules.lists.is_empty() {
        return;
    }
    let Some(settings) = HOST_SETTINGS.lock().clone() else { return };
    let Ok(ep) = endpoint(&settings, Who::Anonymous).await else { return };
    for f in rules.lists.iter().take(crate::admission::MAX_LISTS) {
        for from in f.from.iter().take(4) {
            let Some(key) = crate::admission::world_key(from) else { continue };
            // Whose list: the one named, else the owner of that world, as its certificate says.
            let maintainer = if f.maintainer.is_empty() {
                let Ok(addr) = parse_ticket(from) else { continue };
                match status_on(&ep, addr, Duration::from_secs(10)).await {
                    Ok(st) if !st.owner.is_empty() => {
                        let (d, k, o) = (dir.to_path_buf(), key.clone(), st.owner.clone());
                        if let Ok(Err(e)) = tokio::task::spawn_blocking(move || crate::admission::set_owner(&d, &k, &o)).await {
                            log::warn!("{e}");
                        }
                        st.owner
                    }
                    Ok(_) => {
                        log::info!(
                            "ban list from {}: that world's certificate doesn't check out",
                            key.chars().take(12).collect::<String>()
                        );
                        continue;
                    }
                    Err(e) => {
                        log::info!("ban list from {}: {e}", key.chars().take(12).collect::<String>());
                        continue;
                    }
                }
            } else {
                f.maintainer.clone()
            };
            match ask_list(&ep, from, &maintainer).await {
                Ok(list) => {
                    let (d, name, seq) = (dir.to_path_buf(), list.name.clone(), list.seq);
                    match tokio::task::spawn_blocking(move || crate::admission::store_list(&d, &list)).await {
                        Ok(Ok(true)) => log::info!("ban list '{name}' version {seq} fetched"),
                        Ok(Ok(false)) => log::info!("ban list '{name}' is up to date (version {seq})"),
                        Ok(Err(e)) => log::warn!("ban list '{name}': {e}"),
                        Err(_) => {}
                    }
                    break;
                }
                Err(e) => log::info!(
                    "ban list of {} from {}: {e}",
                    maintainer.chars().take(12).collect::<String>(),
                    key.chars().take(12).collect::<String>()
                ),
            }
        }
    }
    ep.close().await;
}

/// [`fetch_lists`] in the background.
pub fn fetch_ban_lists(dir: std::path::PathBuf) {
    runtime().spawn(async move { fetch_lists(&dir).await });
}

pub fn stop_hosting() {
    *HOSTING.lock() = None;
    // Closing it ends its accept loop.
    if let Some(ep) = HOST_ENDPOINT.lock().take() {
        runtime().spawn(async move { ep.close().await });
    }
}

async fn accept(
    conn: iroh::endpoint::Connection,
    connect: crossbeam_channel::Sender<ServerConn>,
    world: &'static crate::content::Content,
    me: &str,
    dir: Option<std::path::PathBuf>,
) -> Result<(), String> {
    use crate::content::with;
    let peer = conn.remote_id();
    // Banned (chiacraft, `admission`): turned away at the door, before anything is served.
    let banned = match dir {
        Some(d) => {
            let key = peer.to_string();
            tokio::task::spawn_blocking(move || crate::admission::at_the_door(&d, &key))
                .await
                .ok()
                .flatten()
        }
        None => None,
    };
    let (mut send, mut recv) = tokio::time::timeout(Duration::from_secs(10), conn.accept_bi())
        .await
        .map_err(|_| "no hello")?
        .map_err(|e| e.to_string())?;
    let hello = tokio::time::timeout(Duration::from_secs(10), read_frame_max(&mut recv, MAX_HELLO))
        .await
        .map_err(|_| "no hello")??;
    let mut r = codec::R(&hello);
    let (proto, native) = <(u32, u64) as codec::Wire>::get(&mut r).ok_or("bad hello")?;
    // Through a portal: a travel ticket from the linked world places the player at its portal.
    let ticket = <String as codec::Wire>::get(&mut r).unwrap_or_default();
    if !ticket.is_empty() {
        match crate::travel::admit(&ticket, &peer.to_string(), me) {
            Ok(()) => log::info!("{peer} comes through a portal"),
            Err(e) => log::info!("{peer}: {e}; placed as usual"),
        }
    }
    let offers = with(world, crate::modloader::current_packages);
    // Joins still configuring hold a place until this goes out of scope.
    let place = Configuring::take();
    let refusal = if let Some(why) = &banned {
        log::info!("{peer} is not let in at the door: {}", why.key);
        crate::admission::refusal_text(why)
    } else if place.is_none() {
        "the world is busy with other players joining; try again in a minute".to_string()
    } else if proto != PROTOCOL {
        format!("protocol mismatch: server {PROTOCOL}, client {proto}")
    } else if native != with(world, native_hash) {
        format!("different mods; server runs: {}", with(world, mod_list))
    } else {
        String::new()
    };
    let mut w = codec::W::default();
    codec::Wire::put(&(refusal.clone(), offers.clone()), &mut w);
    write_frame(&mut send, &w.0).await?;
    if !refusal.is_empty() {
        return Err(refusal);
    }
    // The joining game downloads the packages it lacks, then says what content it loaded.
    let serving = tokio::spawn(serve_packages(conn.clone(), offers));
    let ready = tokio::time::timeout(CONFIGURE_TIME, read_frame_max(&mut recv, MAX_HELLO)).await;
    serving.abort();
    drop(place);
    let ready = ready.map_err(|_| "took too long to download and load the world's packages")?;
    let loaded = <u64 as codec::Wire>::get(&mut codec::R(&ready?)).ok_or("bad ready")?;
    let refusal = if loaded != with(world, content_hash) {
        "the content does not match this world's after its packages".to_string()
    } else {
        String::new()
    };
    write_frame(&mut send, refusal.as_bytes()).await?;
    if !refusal.is_empty() {
        return Err(refusal);
    }
    log::info!("player {peer} connected");
    let (to_server_tx, to_server_rx) = unbounded::<ToServer>();
    let (to_client_tx, to_client_rx) = unbounded::<ToClient>();
    connect
        .send(Conn::new(to_client_tx, to_server_rx))
        .map_err(|_| "server stopped")?;
    let peer_key = peer.to_string();
    // Incoming: the authenticated key replaces whatever key the client claims.
    let reader = tokio::spawn(async move {
        while let Ok(frame) = read_frame(&mut recv).await {
            let Some(mut msg) = with(world, || codec::decode_server(&frame)) else { break };
            if let ToServer::Join { key, .. } = &mut msg {
                *key = peer_key.clone();
            }
            if to_server_tx.send(msg).is_err() {
                break;
            }
        }
        let _ = to_server_tx.send(ToServer::Leave);
    });
    pump_out(to_client_rx, send, codec::encode_client).await;
    reader.abort();
    conn.close(0u32.into(), b"bye");
    Ok(())
}

/// Answers a joining game's package downloads, each on a stream of its own: the manifests of the
/// offered packages and their files, from the store, and nothing else.
async fn serve_packages(conn: iroh::endpoint::Connection, offers: Vec<Offer>) {
    use chiacraft_packages::identity::Hash;
    let Some(store) = crate::modloader::store() else { return };
    let mut manifests = rustc_hash::FxHashSet::default();
    let mut files = rustc_hash::FxHashSet::default();
    let mut offered = 0u64;
    for (_, _, hex, ..) in &offers {
        let Some(h) = Hash::from_hex(hex) else { continue };
        if let Ok(m) = store.manifest(&h) {
            manifests.insert(h);
            files.extend(m.files.iter().map(|f| f.hash));
            offered += m.files.iter().map(|f| f.size).sum::<u64>() + MAX_MANIFEST;
        }
    }
    // Each player may fetch what is offered about twice over, a few requests at a time: enough
    // for a retry, never an endless drain of the host's upload.
    let budget = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(offered * 2));
    let requests = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new((manifests.len() + files.len()) * 2 + 16));
    let streams = std::sync::Arc::new(tokio::sync::Semaphore::new(PEER_STREAMS));
    let (manifests, files) = (std::sync::Arc::new(manifests), std::sync::Arc::new(files));
    while let Ok((mut send, mut recv)) = conn.accept_bi().await {
        use std::sync::atomic::Ordering::Relaxed;
        if requests
            .fetch_update(Relaxed, Relaxed, |n| n.checked_sub(1))
            .is_err()
        {
            log::warn!("{}: too many package requests", conn.remote_id());
            conn.close(1u32.into(), b"too many requests");
            return;
        }
        let Ok(permit) = streams.clone().acquire_owned().await else { return };
        let (store, manifests, files, budget) =
            (chiacraft_packages::store::Store::new(store.root()), manifests.clone(), files.clone(), budget.clone());
        tokio::spawn(async move {
            let _permit = permit;
            let Ok(Ok(req)) = tokio::time::timeout(STALL, read_frame_max(&mut recv, 256)).await else { return };
            let Some((kind, hex)) = <(u32, String) as codec::Wire>::get(&mut codec::R(&req)) else { return };
            let h = Hash::from_hex(&hex);
            let answer: Result<Body, String> = match (kind as u8, h) {
                (GET_MANIFEST, Some(h)) if manifests.contains(&h) => store.manifest_text(&h).map(|t| Body::Bytes(t.into_bytes())),
                (GET_FILE, Some(h)) if files.contains(&h) => store
                    .blob_path(&h)
                    .ok_or("not in the store".to_string())
                    .and_then(|p| Ok(Body::File(std::fs::metadata(&p).map_err(|e| e.to_string())?.len(), p))),
                _ => Err("not offered".into()),
            };
            let answer = answer.and_then(|a| match budget.fetch_update(Relaxed, Relaxed, |b| b.checked_sub(a.len())) {
                Ok(_) => Ok(a),
                Err(_) => Err("this player has downloaded too much".into()),
            });
            let mut w = codec::W::default();
            match answer {
                Err(e) => {
                    codec::Wire::put(&(e, 0u64), &mut w);
                    let _ = write_frame(&mut send, &w.0).await;
                }
                Ok(body) => {
                    codec::Wire::put(&(String::new(), body.len()), &mut w);
                    if write_frame(&mut send, &w.0).await.is_err() {
                        return;
                    }
                    let sent = match &body {
                        Body::Bytes(b) => send.write_all(b).await.is_ok(),
                        Body::File(len, p) => send_file(&mut send, p, *len).await.is_ok(),
                    };
                    if !sent {
                        return;
                    }
                }
            }
            let _ = send.finish();
        });
    }
}

/// What a package request is answered with: a manifest's text, or a stored file of this length.
enum Body {
    Bytes(Vec<u8>),
    File(u64, std::path::PathBuf),
}

impl Body {
    fn len(&self) -> u64 {
        match self {
            Body::Bytes(b) => b.len() as u64,
            Body::File(len, _) => *len,
        }
    }
}

/// Streams `len` bytes of the file at `path`, a piece at a time (never the whole file in memory).
async fn send_file(send: &mut iroh::endpoint::SendStream, path: &Path, len: u64) -> Result<(), String> {
    use tokio::io::AsyncReadExt;
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|e| e.to_string())?
        .take(len);
    let mut buf = vec![0u8; 256 << 10];
    loop {
        let n = file.read(&mut buf).await.map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(());
        }
        tokio::time::timeout(STALL, send.write_all(&buf[..n]))
            .await
            .map_err(|_| "stalled".to_string())?
            .map_err(|e| e.to_string())?;
    }
}

/// A place among the joins the host is configuring at once ([`MAX_CONFIGURING`]); given back on
/// drop.
struct Configuring;

static CONFIGURING: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

impl Configuring {
    fn take() -> Option<Configuring> {
        use std::sync::atomic::Ordering::SeqCst;
        CONFIGURING
            .fetch_update(SeqCst, SeqCst, |n| (n < MAX_CONFIGURING).then_some(n + 1))
            .ok()
            .map(|_| Configuring)
    }
}

impl Drop for Configuring {
    fn drop(&mut self) {
        CONFIGURING.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Fetches one manifest or file from the host on a stream of its own; at most `max` bytes.
async fn fetch(
    conn: &iroh::endpoint::Connection,
    kind: u8,
    hash: &chiacraft_packages::identity::Hash,
    max: u64,
) -> Result<Vec<u8>, String> {
    let (mut send, mut recv) = conn.open_bi().await.map_err(|e| e.to_string())?;
    let mut w = codec::W::default();
    codec::Wire::put(&(kind as u32, hash.hex()), &mut w);
    write_frame(&mut send, &w.0).await?;
    let head = tokio::time::timeout(STALL, read_frame_max(&mut recv, 1024))
        .await
        .map_err(|_| "the download stalled")??;
    let (err, len) = <(String, u64) as codec::Wire>::get(&mut codec::R(&head)).ok_or("bad answer")?;
    if !err.is_empty() {
        return Err(format!("the host: {err}"));
    }
    if len > max {
        return Err(format!("the host sent {len} bytes where {max} were expected"));
    }
    let mut buf = vec![0u8; len as usize];
    let mut got = 0;
    while got < buf.len() {
        match tokio::time::timeout(STALL, recv.read(&mut buf[got..])).await {
            Err(_) => return Err("the download stalled".into()),
            Ok(Err(e)) => return Err(e.to_string()),
            Ok(Ok(None)) => return Err("the host stopped sending".into()),
            Ok(Ok(Some(n))) => got += n,
        }
    }
    Ok(buf)
}

/// What a join in progress reports to the game, in order.
pub enum JoinEvent {
    /// The world plays with these packages; the second list is what this game lacks. Answer
    /// `Accept` to download them or `Decline`.
    Offer(Vec<Offer>, Vec<Offer>),
    /// Bytes downloaded of the total.
    Progress(u64, u64),
    /// Every package is in the store: load exactly these (hashes), then answer `Ready`.
    Packages(Vec<String>),
    Admitted(ClientConn),
    Failed(String),
}

pub enum JoinAnswer {
    Accept,
    Decline,
    /// The content hash after loading the world's packages.
    Ready(u64),
    Cancel(String),
}

/// A join in progress: its events, and where the game answers them.
pub struct Joining {
    pub events: crossbeam_channel::Receiver<JoinEvent>,
    pub answers: tokio::sync::mpsc::UnboundedSender<JoinAnswer>,
}

/// Downloads the offered packages this game lacks into the store, checking every manifest and
/// file against its hash and the sizes the host declared.
async fn download(
    conn: &iroh::endpoint::Connection,
    missing: &[Offer],
    events: &crossbeam_channel::Sender<JoinEvent>,
) -> Result<(), String> {
    use chiacraft_packages::identity::{FileManifest, Hash, MAX_TOTAL_BYTES};
    let store = crate::modloader::store().ok_or("mods are not loaded")?;
    let total: u64 = missing.iter().map(|o| o.3).sum();
    if missing.len() > MAX_OFFERS || total > MAX_JOIN_BYTES {
        return Err(format!("the world wants {} packages, {} MiB: more than this game downloads", missing.len(), total >> 20));
    }
    let mut done = 0u64;
    for (id, _, hex, size, sig, code) in missing {
        let h = Hash::from_hex(hex).ok_or_else(|| format!("package {id}: a bad hash"))?;
        if *size > MAX_TOTAL_BYTES {
            return Err(format!("package {id} is too big ({size} bytes)"));
        }
        let text = fetch(conn, GET_MANIFEST, &h, MAX_MANIFEST).await?;
        if Hash::of(&text) != h {
            return Err(format!("package {id}: the host sent a manifest that doesn't match its hash"));
        }
        let manifest =
            FileManifest::parse(std::str::from_utf8(&text).map_err(|e| e.to_string())?).map_err(|e| format!("package {id}: {e}"))?;
        if &manifest.id != id || manifest.files.iter().map(|f| f.size).sum::<u64>() != *size {
            return Err(format!("package {id}: the manifest doesn't match what the host offered"));
        }
        // The player was asked knowing whether it carries code: a host that hid it is refused.
        if manifest
            .files
            .iter()
            .any(|f| chiacraft_packages::identity::is_code(&f.path))
            != !code.is_empty()
        {
            return Err(format!("package {id}: the host didn't say truly whether it carries code"));
        }
        for f in &manifest.files {
            if !store.has_blob(&f.hash) {
                let bytes = fetch(conn, GET_FILE, &f.hash, f.size).await?;
                if bytes.len() as u64 != f.size {
                    return Err(format!("package {id}: {} came short", f.path));
                }
                store
                    .put_blob(&f.hash, &bytes)
                    .map_err(|e| format!("package {id}: {}: {e}", f.path))?;
            }
            done += f.size;
            let _ = events.send(JoinEvent::Progress(done, total));
        }
        store.lay_out(&manifest)?;
        // Who made it travels beside it, kept only when it checks out.
        if !sig.is_empty() && crate::package_trust::verify(sig, hex).is_ok() {
            store.put_signature(&h, sig)?;
        }
        log::info!("downloaded package {id} ({})", h.short());
    }
    Ok(())
}

/// Forwards a crossbeam channel into a stream. The channel is read on a blocking thread because
/// the engine side is synchronous.
async fn pump_out<T: Send + crate::net::Message + 'static>(
    rx: crossbeam_channel::Receiver<T>,
    mut send: iroh::endpoint::SendStream,
    encode: fn(&T) -> Vec<u8>,
) {
    let (tx, mut arx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    std::thread::Builder::new()
        .name("net-out".into())
        .spawn(move || {
            while let Ok(m) = rx.recv() {
                let bytes = encode(&m);
                crate::diag::net_message(T::TO_SERVER, m.kind(), 0, bytes.len());
                if tx.send(bytes).is_err() {
                    break;
                }
            }
        })
        .ok();
    while let Some(frame) = arx.recv().await {
        if write_frame(&mut send, &frame).await.is_err() {
            break;
        }
    }
}

/// Logs which way a connection to another game goes (Ronald, 11-04: do joins really go over the
/// network?), when it opens and whenever that changes, until it closes: `net: path to <id> relay`,
/// `direct` or `mixed` (both open), and the kind of address in use (loopback, LAN, VPN, public, or
/// the relay). `what`: a join, a portal arrival, or a player coming in (the host's side).
fn watch_path(conn: iroh::endpoint::Connection, what: &'static str) {
    runtime().spawn(async move {
        let id = conn.remote_id().to_string();
        let id = &id[..id.len().min(8)];
        let mut last = String::new();
        while conn.close_reason().is_none() {
            let line = path_line(&conn);
            if line != last {
                log::info!("net: path to {id} {line} ({what})");
                last = line;
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}

/// `relay`, `direct` or `mixed`, and the address the connection sends on now with its kind.
fn path_line(conn: &iroh::endpoint::Connection) -> String {
    let (mut ip, mut relay, mut used) = (false, false, None);
    for p in conn.paths().iter() {
        ip |= p.is_ip();
        relay |= p.is_relay();
        if p.is_selected() {
            used = Some(p.remote_addr().clone());
        }
    }
    let way = match (ip, relay) {
        (true, true) => "mixed",
        (true, false) => "direct",
        (false, true) => "relay",
        (false, false) => "none",
    };
    match used {
        Some(TransportAddr::Ip(a)) => format!("{way} {} {a}", ip_kind(a.ip())),
        Some(TransportAddr::Relay(url)) => format!("{way} relay {url}"),
        Some(other) => format!("{way} {other:?}"),
        None => format!("{way} (no path in use yet)"),
    }
}

/// Where an address is: this machine, the local network, a VPN or carrier NAT range (Tailscale's
/// 100.64/10), or the internet.
fn ip_kind(ip: std::net::IpAddr) -> &'static str {
    use std::net::IpAddr;
    match ip {
        IpAddr::V4(v) if v.is_loopback() => "loopback",
        IpAddr::V4(v) if v.is_private() || v.is_link_local() => "LAN",
        IpAddr::V4(v) if v.octets()[0] == 100 && (64..128).contains(&v.octets()[1]) => "VPN",
        IpAddr::V6(v) if v.is_loopback() => "loopback",
        IpAddr::V6(v) if (v.segments()[0] & 0xfe00) == 0xfc00 || (v.segments()[0] & 0xffc0) == 0xfe80 => "LAN",
        IpAddr::V6(v) if v.to_ipv4_mapped().is_some() => v
            .to_ipv4_mapped()
            .map_or("public", |v4| ip_kind(IpAddr::V4(v4))),
        _ => "public",
    }
}

/// Connects to `addr`: every address at once first; if the handshake does not finish soon, again
/// from a new endpoint through the relay alone. With several network adapters (VPNs, virtual
/// switches) the host may take the handshake's first packet from an address its answers never
/// get back to, and the attempt hangs on it whatever else the client tries (about one in six
/// on a machine with Radmin VPN and Hyper-V). The relay always gets through; iroh moves the
/// connection to a direct path afterwards when one works.
///
/// Both tries go out through the player's account endpoint (one endpoint per key).
async fn connect(settings: &Settings, addr: EndpointAddr) -> Result<iroh::endpoint::Connection, String> {
    let relayed = EndpointAddr::from_parts(
        addr.id,
        addr.relay_urls()
            .cloned()
            .map(TransportAddr::Relay)
            .collect::<Vec<_>>(),
    );
    // An address with no hints is looked up (the relay-only try would have nothing to go on).
    let attempts = if relayed.relay_urls().next().is_some() {
        vec![(addr, Duration::from_secs(3)), (relayed, Duration::from_secs(20))]
    } else {
        vec![(addr, Duration::from_secs(20))]
    };
    let mut last = String::new();
    let ep = account(settings).await?;
    let tries = attempts.len();
    for (i, (addr, wait)) in attempts.into_iter().enumerate() {
        match tokio::time::timeout(wait, ep.connect(addr, ALPN)).await {
            Ok(Ok(conn)) => return Ok(conn),
            Ok(Err(e)) => last = e.to_string(),
            Err(_) => last = "timed out".into(),
        }
        if i == 0 && tries > 1 {
            log::info!("connecting directly failed ({last}), trying through the relay");
        }
    }
    Err(last)
}

/// What a host offers is shown to the player and fetched: only well-formed offers, within limits.
fn check_offers(offers: &[Offer]) -> Result<(), String> {
    if offers.len() > MAX_OFFERS {
        return Err(format!("the world plays with {} packages: more than this game takes", offers.len()));
    }
    for (id, version, hash, _, sig, api) in offers {
        let fine = (1..=64).contains(&id.len())
            && id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            && (1..=64).contains(&version.len())
            && version
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || ".+-".contains(c))
            && chiacraft_packages::identity::Hash::from_hex(hash).is_some()
            && sig.len() <= 4096;
        if !fine {
            return Err("the host offered a package this game can't read".into());
        }
        // Code built for an API this game doesn't run would only fail once it loads.
        match chiacraft_packages::code::parse(api) {
            None => return Err("the host offered a package this game can't read".into()),
            Some(Some(v)) if !chiacraft_packages::code::runs(v) => {
                let (a, b) = chiacraft_packages::code::API;
                return Err(format!("package {id} runs code for mod API {api}; this game runs {a}.{b}: a newer game can join"));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Starts joining a host. The join runs in the background and reports through `events`: the
/// world's packages to download (with consent), progress, the packages to load, and finally the
/// connection, or why it failed.
pub fn start_join(ticket: &str, settings: &Settings) -> Result<Joining, String> {
    let addr = parse_ticket(ticket)?;
    let travel = crate::world_address::parse(ticket)
        .map(|p| p.ticket)
        .unwrap_or_default();
    let settings = settings.clone();
    let (events, events_rx) = unbounded::<JoinEvent>();
    let (answers, mut answers_rx) = tokio::sync::mpsc::unbounded_channel::<JoinAnswer>();
    runtime().spawn(async move {
        let result = async {
            let conn = connect(&settings, addr).await?;
            watch_path(conn.clone(), if travel.is_empty() { "join" } else { "portal arrival" });
            let (mut send, mut recv) = conn.open_bi().await.map_err(|e| e.to_string())?;
            let mut hello = codec::W::default();
            codec::Wire::put(&(PROTOCOL, native_hash(), travel.clone()), &mut hello);
            write_frame(&mut send, &hello.0).await?;
            let reply = tokio::time::timeout(Duration::from_secs(30), read_frame_max(&mut recv, MAX_OFFER))
                .await
                .map_err(|_| "the host didn't answer")??;
            let (refusal, offers) = <(String, Vec<Offer>) as codec::Wire>::get(&mut codec::R(&reply))
                .ok_or("the host answered in an unknown way (another version?)")?;
            if !refusal.is_empty() {
                return Err(refusal.chars().take(500).collect());
            }
            check_offers(&offers)?;
            let store = crate::modloader::store().ok_or("mods are not loaded")?;
            let missing: Vec<Offer> = offers
                .iter()
                .filter(|o| chiacraft_packages::identity::Hash::from_hex(&o.2).is_none_or(|h| store.verify(&h).is_err()))
                .cloned()
                .collect();
            if !missing.is_empty() {
                let _ = events.send(JoinEvent::Offer(offers.clone(), missing.clone()));
                match answers_rx.recv().await {
                    Some(JoinAnswer::Accept) => download(&conn, &missing, &events).await?,
                    Some(JoinAnswer::Cancel(e)) => return Err(e),
                    _ => return Err("declined the world's packages".into()),
                }
            }
            let _ = events.send(JoinEvent::Packages(offers.iter().map(|o| o.2.clone()).collect()));
            let loaded = match answers_rx.recv().await {
                Some(JoinAnswer::Ready(h)) => h,
                Some(JoinAnswer::Cancel(e)) => return Err(e),
                _ => return Err("cancelled".into()),
            };
            let mut w = codec::W::default();
            codec::Wire::put(&loaded, &mut w);
            write_frame(&mut send, &w.0).await?;
            let verdict = tokio::time::timeout(Duration::from_secs(30), read_frame_max(&mut recv, MAX_HELLO))
                .await
                .map_err(|_| "the host didn't answer")??;
            if !verdict.is_empty() {
                return Err(String::from_utf8_lossy(&verdict).into_owned());
            }
            Ok((conn, send, recv))
        }
        .await;
        match result {
            Ok((conn, send, mut recv)) => {
                let (to_server_tx, to_server_rx) = unbounded::<ToServer>();
                let (to_client_tx, to_client_rx) = unbounded::<ToClient>();
                let _ = events.send(JoinEvent::Admitted(Conn::new(to_server_tx, to_client_rx)));
                let reader = tokio::spawn(async move {
                    while let Ok(frame) = read_frame(&mut recv).await {
                        let Some(msg) = codec::decode_client(&frame) else { break };
                        if to_client_tx.send(msg).is_err() {
                            break;
                        }
                    }
                    let _ = to_client_tx.send(ToClient::Disconnect("connection closed".into()));
                });
                pump_out(to_server_rx, send, codec::encode_server).await;
                reader.abort();
                conn.close(0u32.into(), b"bye");
            }
            Err(e) => {
                let _ = events.send(JoinEvent::Failed(e));
            }
        }
    });
    Ok(Joining { events: events_rx, answers })
}

#[cfg(test)]
mod tests {

    /// Which kind of network a joined game's address is on (`net: path to`).
    #[test]
    fn address_kinds() {
        let kind = |a: &str| super::ip_kind(a.parse().unwrap());
        assert_eq!(kind("127.0.0.1"), "loopback");
        assert_eq!(kind("::1"), "loopback");
        assert_eq!(kind("192.168.1.13"), "LAN");
        assert_eq!(kind("10.0.0.5"), "LAN");
        assert_eq!(kind("fe80::1"), "LAN");
        assert_eq!(kind("100.127.39.114"), "VPN");
        assert_eq!(kind("::ffff:192.168.1.13"), "LAN");
        assert_eq!(kind("5.223.65.62"), "public");
        assert_eq!(kind("2a01:4ff:f0:b1b3::1"), "public");
    }
    /// The identity is the account's network key: made on the first call, the same afterwards.
    #[test]
    fn identity_is_the_accounts_network_key() {
        let dir = std::env::temp_dir().join(format!("weft-identity-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        chiacraft_account::use_file_store();
        let a = super::identity(&dir).expect("a new identity");
        assert_eq!(super::identity(&dir).expect("asked again").public(), a.public());
        let session = chiacraft_account::current().expect("the account is open");
        assert_eq!(a.to_bytes(), *session.network_seed());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
