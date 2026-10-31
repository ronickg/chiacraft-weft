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
/// 3: `Join` carries the account's vouching (chiacraft). 4: the hello offers the world's data
/// packages, which the joining game downloads and loads before it is let in (chiacraft).
const PROTOCOL: u32 = 4;
const MAX_FRAME: usize = 64 << 20;

/// A data package the host's world plays with: id, version, hash (hex), size in bytes.
pub type Offer = (String, String, String, u64);

/// Requests on the extra streams a joining game opens to download packages.
const GET_MANIFEST: u8 = 0;
const GET_FILE: u8 = 1;
const MAX_MANIFEST: u64 = 1 << 20;

fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread().worker_threads(2).thread_name("net").enable_all().build().expect("tokio runtime")
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
    identity(run_dir).map(|k| k.public().to_string()).unwrap_or_else(|e| {
        log::error!("{e}; this world keys the player by name");
        String::new()
    })
}

/// Hash over the code mods (id and version): what a joining game must bring itself. Its data
/// packages it can download.
pub fn native_hash() -> u64 {
    let mut h = weft_noise::FNV;
    for m in crate::content::content().mods.iter().filter(|m| m.package.is_none()) {
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
    for n in c.biomes.iter().map(|b| &b.name).chain(c.dims.iter().map(|d| &d.name)) {
        eat(n);
    }
    for n in c.game.sounds.iter().map(|s| &s.name).chain(c.game.particles.iter().map(|p| &p.name)).chain(c.game.configs.iter().map(|k| &k.name)) {
        eat(n);
    }
    for a in &c.game.attributes {
        eat(&a.name);
    }
    h
}

fn mod_list() -> String {
    crate::content::content().mods.iter().filter(|m| m.package.is_none()).map(|m| format!("{} {}", m.id, m.version)).collect::<Vec<_>>().join(", ")
}

fn relay_mode(settings: &Settings) -> RelayMode {
    let custom: Vec<RelayUrl> = settings.relays().iter().filter_map(|r| r.parse().ok()).collect();
    if custom.is_empty() { RelayMode::Default } else { RelayMode::custom(custom) }
}

async fn endpoint(settings: &Settings, accept: bool) -> Result<Endpoint, String> {
    let mut b = Endpoint::builder(presets::N0).secret_key(identity(&settings.run_dir)?).relay_mode(relay_mode(settings));
    if accept {
        b = b.alpns(vec![ALPN.to_vec()]);
    }
    b.bind().await.map_err(|e| format!("network: {e}"))
}

async fn write_frame(send: &mut iroh::endpoint::SendStream, data: &[u8]) -> Result<(), String> {
    let packed = if data.len() > 256 { lz4_flex::compress_prepend_size(data) } else { data.to_vec() };
    let flag = (data.len() > 256) as u32;
    let header = (packed.len() as u32) << 1 | flag;
    send.write_all(&header.to_le_bytes()).await.map_err(|e| e.to_string())?;
    send.write_all(&packed).await.map_err(|e| e.to_string())
}

async fn read_frame(recv: &mut iroh::endpoint::RecvStream) -> Result<Vec<u8>, String> {
    let mut h = [0u8; 4];
    recv.read_exact(&mut h).await.map_err(|e| e.to_string())?;
    let header = u32::from_le_bytes(h);
    let len = (header >> 1) as usize;
    if len > MAX_FRAME {
        return Err("frame too large".into());
    }
    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf).await.map_err(|e| e.to_string())?;
    if header & 1 == 1 { lz4_flex::decompress_size_prepended(&buf).map_err(|e| e.to_string()) } else { Ok(buf) }
}

/// Ticket: `id;relay;ip:port,ip:port`. The id alone is enough with n0 address lookup; the
/// relay and direct addresses make it work on a LAN or without DNS.
pub fn ticket(addr: &EndpointAddr) -> String {
    let relay = addr.relay_urls().next().map(|r| r.to_string()).unwrap_or_default();
    let ips: Vec<String> = addr.ip_addrs().map(|a| a.to_string()).collect();
    format!("{};{};{}", addr.id, relay, ips.join(","))
}

fn parse_ticket(t: &str) -> Result<EndpointAddr, String> {
    let mut parts = t.trim().split(';');
    let id: EndpointId = parts.next().unwrap_or("").parse().map_err(|_| "invalid ticket".to_string())?;
    let mut addrs = Vec::new();
    if let Some(r) = parts.next().filter(|s| !s.is_empty()).and_then(|r| r.parse::<RelayUrl>().ok()) {
        addrs.push(TransportAddr::Relay(r));
    }
    for ip in parts.next().unwrap_or("").split(',').filter_map(|s| s.parse().ok()) {
        addrs.push(TransportAddr::Ip(ip));
    }
    Ok(EndpointAddr::from_parts(id, addrs))
}

pub struct Hosting {
    pub ticket: Option<String>,
    pub error: Option<String>,
}

pub static HOSTING: Mutex<Option<Hosting>> = Mutex::new(None);

/// Starts accepting players for the running server. The ticket appears in `HOSTING` once the
/// endpoint is online.
pub fn host(connect: crossbeam_channel::Sender<ServerConn>, settings: &Settings) {
    if HOSTING.lock().is_some() {
        return;
    }
    *HOSTING.lock() = Some(Hosting { ticket: None, error: None });
    let settings = settings.clone();
    runtime().spawn(async move {
        let ep = match endpoint(&settings, true).await {
            Ok(e) => e,
            Err(e) => {
                log::error!("{e}");
                *HOSTING.lock() = Some(Hosting { ticket: None, error: Some(e) });
                return;
            }
        };
        let _ = tokio::time::timeout(Duration::from_secs(8), ep.online()).await;
        let t = ticket(&ep.addr());
        log::info!("hosting, ticket: {t}");
        if let Some(h) = HOSTING.lock().as_mut() {
            h.ticket = Some(t);
        }
        while let Some(incoming) = ep.accept().await {
            let connect = connect.clone();
            tokio::spawn(async move {
                if let Err(e) = accept(incoming, connect).await {
                    log::warn!("incoming player: {e}");
                }
            });
        }
    });
}

pub fn stop_hosting() {
    *HOSTING.lock() = None;
}

async fn accept(incoming: iroh::endpoint::Incoming, connect: crossbeam_channel::Sender<ServerConn>) -> Result<(), String> {
    let conn = incoming.await.map_err(|e| e.to_string())?;
    let peer = conn.remote_id();
    let (mut send, mut recv) = conn.accept_bi().await.map_err(|e| e.to_string())?;
    let hello = read_frame(&mut recv).await?;
    let (proto, native) = <(u32, u64) as codec::Wire>::get(&mut codec::R(&hello)).ok_or("bad hello")?;
    let offers = crate::modloader::current_packages();
    let refusal = if proto != PROTOCOL {
        format!("protocol mismatch: server {PROTOCOL}, client {proto}")
    } else if native != native_hash() {
        format!("different mods; server runs: {}", mod_list())
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
    let ready = read_frame(&mut recv).await;
    serving.abort();
    let loaded = <u64 as codec::Wire>::get(&mut codec::R(&ready?)).ok_or("bad ready")?;
    let refusal = if loaded != content_hash() { "the content does not match this world's after its packages".to_string() } else { String::new() };
    write_frame(&mut send, refusal.as_bytes()).await?;
    if !refusal.is_empty() {
        return Err(refusal);
    }
    log::info!("player {peer} connected");
    let (to_server_tx, to_server_rx) = unbounded::<ToServer>();
    let (to_client_tx, to_client_rx) = unbounded::<ToClient>();
    connect.send(Conn::new(to_client_tx, to_server_rx)).map_err(|_| "server stopped")?;
    let peer_key = peer.to_string();
    // Incoming: the authenticated key replaces whatever key the client claims.
    let reader = tokio::spawn(async move {
        while let Ok(frame) = read_frame(&mut recv).await {
            let Some(mut msg) = codec::decode_server(&frame) else { break };
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
    for (_, _, hex, _) in &offers {
        let Some(h) = Hash::from_hex(hex) else { continue };
        if let Ok(m) = store.manifest(&h) {
            manifests.insert(h);
            files.extend(m.files.iter().map(|f| f.hash));
        }
    }
    let (manifests, files) = (std::sync::Arc::new(manifests), std::sync::Arc::new(files));
    while let Ok((mut send, mut recv)) = conn.accept_bi().await {
        let (store, manifests, files) = (chiacraft_packages::store::Store::new(store.root()), manifests.clone(), files.clone());
        tokio::spawn(async move {
            let Ok(req) = read_frame(&mut recv).await else { return };
            let Some((kind, hex)) = <(u32, String) as codec::Wire>::get(&mut codec::R(&req)) else { return };
            let h = Hash::from_hex(&hex);
            let bytes: Result<Vec<u8>, String> = match (kind as u8, h) {
                (GET_MANIFEST, Some(h)) if manifests.contains(&h) => store.manifest_text(&h).map(String::into_bytes),
                (GET_FILE, Some(h)) if files.contains(&h) => store.blob_path(&h).ok_or("not in the store".to_string()).and_then(|p| std::fs::read(p).map_err(|e| e.to_string())),
                _ => Err("not offered".into()),
            };
            let (err, bytes) = match bytes {
                Ok(b) => (String::new(), b),
                Err(e) => (e, Vec::new()),
            };
            let mut w = codec::W::default();
            codec::Wire::put(&(err, bytes.len() as u64), &mut w);
            if write_frame(&mut send, &w.0).await.is_ok() && send.write_all(&bytes).await.is_ok() {
                let _ = send.finish();
            }
        });
    }
}

/// Fetches one manifest or file from the host on a stream of its own; at most `max` bytes.
async fn fetch(conn: &iroh::endpoint::Connection, kind: u8, hash: &chiacraft_packages::identity::Hash, max: u64) -> Result<Vec<u8>, String> {
    let (mut send, mut recv) = conn.open_bi().await.map_err(|e| e.to_string())?;
    let mut w = codec::W::default();
    codec::Wire::put(&(kind as u32, hash.hex()), &mut w);
    write_frame(&mut send, &w.0).await?;
    let head = read_frame(&mut recv).await?;
    let (err, len) = <(String, u64) as codec::Wire>::get(&mut codec::R(&head)).ok_or("bad answer")?;
    if !err.is_empty() {
        return Err(format!("the host: {err}"));
    }
    if len > max {
        return Err(format!("the host sent {len} bytes where {max} were expected"));
    }
    let mut buf = vec![0u8; len as usize];
    recv.read_exact(&mut buf).await.map_err(|e| e.to_string())?;
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
async fn download(conn: &iroh::endpoint::Connection, missing: &[Offer], events: &crossbeam_channel::Sender<JoinEvent>) -> Result<(), String> {
    use chiacraft_packages::identity::{FileManifest, Hash, MAX_TOTAL_BYTES};
    let store = crate::modloader::store().ok_or("mods are not loaded")?;
    let total: u64 = missing.iter().map(|o| o.3).sum();
    let mut done = 0u64;
    for (id, _, hex, size) in missing {
        let h = Hash::from_hex(hex).ok_or_else(|| format!("package {id}: a bad hash"))?;
        if *size > MAX_TOTAL_BYTES {
            return Err(format!("package {id} is too big ({size} bytes)"));
        }
        let text = fetch(conn, GET_MANIFEST, &h, MAX_MANIFEST).await?;
        if Hash::of(&text) != h {
            return Err(format!("package {id}: the host sent a manifest that doesn't match its hash"));
        }
        let manifest = FileManifest::parse(std::str::from_utf8(&text).map_err(|e| e.to_string())?).map_err(|e| format!("package {id}: {e}"))?;
        if &manifest.id != id || manifest.files.iter().map(|f| f.size).sum::<u64>() != *size {
            return Err(format!("package {id}: the manifest doesn't match what the host offered"));
        }
        for f in &manifest.files {
            if !store.has_blob(&f.hash) {
                let bytes = fetch(conn, GET_FILE, &f.hash, f.size).await?;
                if bytes.len() as u64 != f.size {
                    return Err(format!("package {id}: {} came short", f.path));
                }
                store.put_blob(&f.hash, &bytes).map_err(|e| format!("package {id}: {}: {e}", f.path))?;
            }
            done += f.size;
            let _ = events.send(JoinEvent::Progress(done, total));
        }
        store.lay_out(&manifest)?;
        log::info!("downloaded package {id} ({})", h.short());
    }
    Ok(())
}

/// Forwards a crossbeam channel into a stream. The channel is read on a blocking thread because
/// the engine side is synchronous.
async fn pump_out<T: Send + crate::net::Message + 'static>(rx: crossbeam_channel::Receiver<T>, mut send: iroh::endpoint::SendStream, encode: fn(&T) -> Vec<u8>) {
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

/// Connects to `addr`: every address at once first; if the handshake does not finish soon, again
/// from a new endpoint through the relay alone. With several network adapters (VPNs, virtual
/// switches) the host may take the handshake's first packet from an address its answers never
/// get back to, and the attempt hangs on it whatever else the client tries (about one in six
/// on a machine with Radmin VPN and Hyper-V). The relay always gets through; iroh moves the
/// connection to a direct path afterwards when one works.
async fn connect(settings: &Settings, addr: EndpointAddr) -> Result<(Endpoint, iroh::endpoint::Connection), String> {
    let relayed = EndpointAddr::from_parts(addr.id, addr.relay_urls().cloned().map(TransportAddr::Relay).collect::<Vec<_>>());
    let attempts = [(addr, Duration::from_secs(3)), (relayed, Duration::from_secs(20))];
    let mut last = String::new();
    for (i, (addr, wait)) in attempts.into_iter().enumerate() {
        let ep = endpoint(settings, false).await?;
        match tokio::time::timeout(wait, ep.connect(addr, ALPN)).await {
            Ok(Ok(conn)) => return Ok((ep, conn)),
            Ok(Err(e)) => last = e.to_string(),
            Err(_) => last = "timed out".into(),
        }
        if i == 0 {
            log::info!("connecting directly failed ({last}), trying through the relay");
        }
        ep.close().await;
    }
    Err(last)
}

/// Starts joining a host. The join runs in the background and reports through `events`: the
/// world's packages to download (with consent), progress, the packages to load, and finally the
/// connection, or why it failed.
pub fn start_join(ticket: &str, settings: &Settings) -> Result<Joining, String> {
    let addr = parse_ticket(ticket)?;
    let settings = settings.clone();
    let (events, events_rx) = unbounded::<JoinEvent>();
    let (answers, mut answers_rx) = tokio::sync::mpsc::unbounded_channel::<JoinAnswer>();
    runtime().spawn(async move {
        let result = async {
            let (ep, conn) = connect(&settings, addr).await?;
            let (mut send, mut recv) = conn.open_bi().await.map_err(|e| e.to_string())?;
            let mut hello = codec::W::default();
            codec::Wire::put(&(PROTOCOL, native_hash()), &mut hello);
            write_frame(&mut send, &hello.0).await?;
            let reply = read_frame(&mut recv).await?;
            let (refusal, offers) = <(String, Vec<Offer>) as codec::Wire>::get(&mut codec::R(&reply)).ok_or("the host answered in an unknown way (another version?)")?;
            if !refusal.is_empty() {
                return Err(refusal);
            }
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
            let verdict = read_frame(&mut recv).await?;
            if !verdict.is_empty() {
                return Err(String::from_utf8_lossy(&verdict).into_owned());
            }
            Ok((ep, conn, send, recv))
        }
        .await;
        match result {
            Ok((ep, conn, send, mut recv)) => {
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
                ep.close().await;
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
