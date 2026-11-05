//! `weft-bots`: many headless players in one process, joining a world over the game's own network
//! protocol (iroh, `weft/1`), for load tests (Research round 18: how many players one server
//! holds). No window, no GPU: each bot is a few tokio tasks on one shared runtime with an iroh
//! endpoint of its own (its key is its identity), so hundreds fit in a process.
//!
//! A bot joins as the game does (hello, the world's packages, the content hash), sends `Join` and
//! `View`, then moves 20 times a second along a random walk about its home, now and then starts
//! and finishes digging the block under it, rarely chats, and every 10 s asks the server a command
//! whose answer comes back from the server's tick: the round trip shows how far behind the server
//! is. It reads everything the server sends and counts it; it decodes only what it needs.
//!
//! ```text
//! weft-bots --address <world address> --mods <server run dir>/mods --packages <dir> ...
//!           --run-dir <a folder of its own> [--ip 127.0.0.1] [--steps 10,50,100,250]
//!           [--hold 90] [--rate 10] [--spread 0] [--report 5] [--threads 8] [--chat 300]
//! ```
//! `--address-from <log>` reads the address from the server's log line `hosting '…' at …`.
//! `--spread R`: each bot's home is a random spot within R blocks of the spawn (0: all at the
//! spawn, bunched). `--steps`: how many bots to have, step by step, each held `--hold` seconds;
//! `--rate`: joins started per second at most.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering::Relaxed};
use std::time::{Duration, Instant};

use glam::{DVec3, IVec3};
use iroh::endpoint::presets;
use iroh::{Endpoint, EndpointAddr, RelayMode, SecretKey, TransportAddr};
use parking_lot::Mutex;
use weft_engine::net::codec::{self, Wire};
use weft_engine::net::p2p::{self, Offer};
use weft_engine::net::{ClientTag, Message, ToClient, ToServer};

struct Opts {
    address: String,
    ip: Option<std::net::IpAddr>,
    steps: Vec<usize>,
    hold: f64,
    rate: f64,
    spread: f64,
    report: f64,
    threads: usize,
    chat: f64,
    mod_dirs: Vec<PathBuf>,
    package_dirs: Vec<PathBuf>,
    run_dir: PathBuf,
    log: String,
}

fn opts() -> Result<Opts, String> {
    let mut o = Opts {
        address: String::new(),
        ip: None,
        steps: vec![10, 50, 100, 250],
        hold: 90.0,
        rate: 10.0,
        spread: 0.0,
        report: 5.0,
        threads: std::thread::available_parallelism().map_or(4, |n| (n.get() / 2).max(2)),
        chat: 300.0,
        mod_dirs: Vec::new(),
        package_dirs: Vec::new(),
        run_dir: PathBuf::from("run-bots"),
        log: String::new(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut v = || args.next().ok_or(format!("{a} needs a value"));
        match a.as_str() {
            "--address" => o.address = v()?,
            "--address-from" => {
                let path = v()?;
                let deadline = Instant::now() + Duration::from_secs(120);
                // The server may still be starting: wait for its line.
                loop {
                    let text = std::fs::read_to_string(&path).unwrap_or_default();
                    if let Some(at) = text
                        .lines()
                        .rev()
                        .find_map(|l| l.split_once("' at ").map(|(_, a)| a.trim().to_string()))
                    {
                        o.address = at;
                        break;
                    }
                    if Instant::now() > deadline {
                        return Err(format!("no `hosting ... at` line in {path}"));
                    }
                    std::thread::sleep(Duration::from_millis(500));
                }
            }
            "--ip" => o.ip = Some(v()?.parse().map_err(|_| "--ip: not an address")?),
            "--steps" => {
                o.steps = v()?
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect()
            }
            "--hold" => o.hold = v()?.parse().map_err(|_| "--hold")?,
            "--rate" => o.rate = v()?.parse().map_err(|_| "--rate")?,
            "--spread" => o.spread = v()?.parse().map_err(|_| "--spread")?,
            "--report" => o.report = v()?.parse().map_err(|_| "--report")?,
            "--threads" => o.threads = v()?.parse().map_err(|_| "--threads")?,
            "--chat" => o.chat = v()?.parse().map_err(|_| "--chat")?,
            "--mods" => o.mod_dirs.push(PathBuf::from(v()?)),
            "--packages" => o.package_dirs.push(PathBuf::from(v()?)),
            "--run-dir" => o.run_dir = PathBuf::from(v()?),
            "--log" => o.log = v()?,
            other => return Err(format!("unknown argument {other}")),
        }
    }
    if o.address.is_empty() {
        return Err("--address or --address-from is needed".into());
    }
    Ok(o)
}

/// A counter per message kind.
struct Counters([AtomicU64; 64]);

impl Default for Counters {
    fn default() -> Self {
        Counters(std::array::from_fn(|_| AtomicU64::new(0)))
    }
}

impl std::ops::Index<usize> for Counters {
    type Output = AtomicU64;
    fn index(&self, i: usize) -> &AtomicU64 {
        &self.0[i]
    }
}

/// Everything the bots counted, read and reset by the report.
#[derive(Default)]
struct Stats {
    live: AtomicUsize,
    joining: AtomicUsize,
    joined: AtomicU64,
    failed: Mutex<HashMap<String, u64>>,
    disconnects: Mutex<HashMap<String, u64>>,
    frames_in: Counters,
    bytes_in: Counters,
    frames_out: AtomicU64,
    bytes_out: AtomicU64,
    /// Seconds: connect to Welcome; Welcome to the view's cubes settled; command round trips.
    join_s: Mutex<Vec<f64>>,
    view_s: Mutex<Vec<f64>>,
    ping_ms: Mutex<Vec<f64>>,
    /// Bytes on the wire (QUIC, both ways) of connections that ended, plus live ones' at report.
    wire: Mutex<HashMap<usize, (u64, u64)>>,
    wire_done: Mutex<(u64, u64)>,
}

fn rng(seed: u64) -> impl FnMut() -> f64 {
    let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn random_key() -> SecretKey {
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
    SecretKey::from_bytes(&bytes)
}

/// What every bot needs to know about the world, found once.
struct World {
    addr: EndpointAddr,
    native: u64,
    /// The content hash after loading the world's packages (all bots load the same ones).
    content: Mutex<Option<u64>>,
}

fn main() {
    let o = match opts() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("weft-bots: {e}");
            std::process::exit(2);
        }
    };
    weft_engine::log::init(Some(&o.run_dir.join("logs").join("bots.log")), false, &o.log);
    weft_engine::init_workers();
    let cache = o.run_dir.join("cache");
    let off = o.run_dir.join("disabled_mods.txt");
    // Kept loaded for the whole run.
    let _mods = match weft_engine::modloader::load(&o.mod_dirs, &o.package_dirs, &cache, &off) {
        Ok(m) => m,
        Err(e) => {
            log::error!("loading the mods: {e}");
            std::process::exit(1);
        }
    };
    let Some(parsed) = weft_engine::world_address::parse(&o.address) else {
        log::error!("not a world address: {}", o.address);
        std::process::exit(1);
    };
    let id = match parsed.key.parse() {
        Ok(id) => id,
        Err(_) => {
            log::error!("the address has no key");
            std::process::exit(1);
        }
    };
    let ips: Vec<TransportAddr> = parsed
        .addrs
        .iter()
        .filter_map(|a| a.parse::<std::net::SocketAddr>().ok())
        .map(|mut a| {
            if let Some(ip) = o.ip {
                a.set_ip(ip);
            }
            TransportAddr::Ip(a)
        })
        .collect();
    log::info!("bots join {} at {} address(es), native hash {:016x}", parsed.name, ips.len(), p2p::native_hash());
    let world = Arc::new(World { addr: EndpointAddr::from_parts(id, ips), native: p2p::native_hash(), content: Mutex::new(None) });
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(o.threads)
        .thread_name("bots")
        .enable_all()
        .build()
        .expect("tokio runtime");
    let stats = Arc::new(Stats::default());
    rt.block_on(ramp(o, world, stats));
}

async fn ramp(o: Opts, world: Arc<World>, stats: Arc<Stats>) {
    let o = Arc::new(o);
    let start = Instant::now();
    let reporter = {
        let (stats, o) = (stats.clone(), o.clone());
        tokio::spawn(async move {
            let mut last = Instant::now();
            loop {
                tokio::time::sleep(Duration::from_secs_f64(o.report)).await;
                report(&stats, last.elapsed().as_secs_f64(), start);
                last = Instant::now();
            }
        })
    };
    let mut next = 0usize;
    for &target in &o.steps {
        log::info!("bots step: {target} bots (t {:.0} s)", start.elapsed().as_secs_f64());
        let gap = Duration::from_secs_f64(1.0 / o.rate.max(0.1));
        while next < target {
            let (world, stats, o) = (world.clone(), stats.clone(), o.clone());
            let n = next;
            tokio::spawn(async move { bot(n, &o, &world, stats).await });
            next += 1;
            tokio::time::sleep(gap).await;
        }
        tokio::time::sleep(Duration::from_secs_f64(o.hold)).await;
    }
    log::info!("bots: last step held; done");
    reporter.abort();
    report(&stats, o.report, start);
}

fn pct(v: &mut [f64], p: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * p).round() as usize]
}

fn report(s: &Stats, secs: f64, start: Instant) {
    let live = s.live.load(Relaxed);
    let mut kinds: Vec<(&str, u64, u64)> = (0..ToClient::KINDS.len())
        .map(|k| (ToClient::KINDS[k], s.frames_in[k].swap(0, Relaxed), s.bytes_in[k].swap(0, Relaxed)))
        .filter(|k| k.1 > 0)
        .collect();
    let (frames, bytes) = kinds.iter().fold((0, 0), |a, k| (a.0 + k.1, a.1 + k.2));
    kinds.sort_by_key(|k| std::cmp::Reverse(k.2));
    let top: Vec<String> = kinds
        .iter()
        .take(6)
        .map(|(k, c, b)| format!("{k} {:.0}/s {:.1} KB/s", *c as f64 / secs, *b as f64 / secs / 1024.0))
        .collect();
    let (out_f, out_b) = (s.frames_out.swap(0, Relaxed), s.bytes_out.swap(0, Relaxed));
    let wire = {
        let mut done = s.wire_done.lock();
        let live: (u64, u64) = s
            .wire
            .lock()
            .values()
            .fold((0, 0), |a, w| (a.0 + w.0, a.1 + w.1));
        let total = (done.0 + live.0, done.1 + live.1);
        *done = (0, 0);
        total
    };
    let (mut join, mut view, mut ping) =
        (std::mem::take(&mut *s.join_s.lock()), std::mem::take(&mut *s.view_s.lock()), std::mem::take(&mut *s.ping_ms.lock()));
    let failed: Vec<String> = s
        .failed
        .lock()
        .iter()
        .map(|(k, v)| format!("{v}x {k}"))
        .collect();
    let disc: Vec<String> = s
        .disconnects
        .lock()
        .iter()
        .map(|(k, v)| format!("{v}x {k}"))
        .collect();
    let per = |b: f64| b / secs / 1024.0 / live.max(1) as f64;
    log::info!(
        "bots stats t {:.0} s: live {live}, joining {}, joined {} | in {:.0} frames/s {:.1} KB/s ({:.2} KB/s a bot) ({}) | out {:.0} frames/s {:.1} KB/s | wire so far rx {:.1} MB tx {:.1} MB | join s p50 {:.2} p99 {:.2} (n {}) | view s p50 {:.2} p99 {:.2} (n {}) | ping ms p50 {:.1} p99 {:.1} max {:.1} (n {}) | failed [{}] | disconnects [{}]",
        start.elapsed().as_secs_f64(),
        s.joining.load(Relaxed),
        s.joined.load(Relaxed),
        frames as f64 / secs,
        bytes as f64 / secs / 1024.0,
        per(bytes as f64),
        top.join(", "),
        out_f as f64 / secs,
        out_b as f64 / secs / 1024.0,
        wire.0 as f64 / 1e6,
        wire.1 as f64 / 1e6,
        pct(&mut join, 0.5),
        pct(&mut join, 0.99),
        join.len(),
        pct(&mut view, 0.5),
        pct(&mut view, 0.99),
        view.len(),
        pct(&mut ping, 0.5),
        pct(&mut ping, 0.99),
        pct(&mut ping, 1.0),
        ping.len(),
        failed.join(", "),
        disc.join(", "),
    );
}

fn count(map: &Mutex<HashMap<String, u64>>, why: &str) {
    let key: String = why.chars().take(80).collect();
    *map.lock().entry(key).or_default() += 1;
}

async fn bot(n: usize, o: &Opts, world: &World, stats: Arc<Stats>) {
    stats.joining.fetch_add(1, Relaxed);
    // A busy world (joins configuring, `MAX_CONFIGURING`) asks to try again: do, a few times.
    let mut tries = 0;
    let joined = loop {
        match join(n, world).await {
            Ok(j) => break Some(j),
            Err(e) if e.contains("busy") && tries < 20 => {
                tries += 1;
                tokio::time::sleep(Duration::from_millis(500 + 250 * tries)).await;
            }
            Err(e) => {
                count(&stats.failed, &e);
                break None;
            }
        }
    };
    stats.joining.fetch_sub(1, Relaxed);
    let Some((ep, conn, send, recv, t0)) = joined else { return };
    stats.live.fetch_add(1, Relaxed);
    let why = play(n, o, stats.clone(), &conn, send, recv, t0).await;
    let st = conn.stats();
    stats.wire.lock().remove(&n);
    {
        let mut d = stats.wire_done.lock();
        d.0 += st.udp_rx.bytes;
        d.1 += st.udp_tx.bytes;
    }
    count(&stats.disconnects, &why);
    stats.live.fetch_sub(1, Relaxed);
    conn.close(0u32.into(), b"bye");
    ep.close().await;
}

type Joined = (Endpoint, iroh::endpoint::Connection, iroh::endpoint::SendStream, iroh::endpoint::RecvStream, Instant);

/// The join as `p2p::start_join` does it: the hello, the world's packages (the bots have them all
/// already; the first one loads them and keeps the content hash), the hash, the verdict.
async fn join(n: usize, world: &World) -> Result<Joined, String> {
    let t0 = Instant::now();
    let ep = Endpoint::builder(presets::Minimal)
        .relay_mode(RelayMode::Disabled)
        .portmapper_config(iroh::endpoint::PortmapperConfig::Disabled)
        .secret_key(random_key())
        .bind()
        .await
        .map_err(|e| format!("bind: {e}"))?;
    let conn = match tokio::time::timeout(Duration::from_secs(20), ep.connect(world.addr.clone(), p2p::ALPN)).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => return Err(format!("connect: {e}")),
        Err(_) => return Err("connect: timed out".into()),
    };
    let (mut send, mut recv) = conn.open_bi().await.map_err(|e| format!("open: {e}"))?;
    let mut hello = codec::W::default();
    (p2p::PROTOCOL, world.native, String::new()).put(&mut hello);
    p2p::write_frame(&mut send, &hello.0).await?;
    let reply = tokio::time::timeout(Duration::from_secs(30), p2p::read_frame_max(&mut recv, p2p::MAX_OFFER))
        .await
        .map_err(|_| "no answer to the hello".to_string())??;
    let (refusal, offers) = <(String, Vec<Offer>)>::get(&mut codec::R(&reply)).ok_or("bad answer")?;
    if !refusal.is_empty() {
        return Err(refusal);
    }
    let hash = {
        let known = *world.content.lock();
        match known {
            Some(h) => h,
            None => {
                let set: Result<Vec<_>, String> = offers
                    .iter()
                    .map(|o| {
                        chiacraft_packages::identity::Hash::from_hex(&o.2)
                            .ok_or_else(|| "a bad package hash".to_string())
                            .and_then(weft_engine::modloader::stored_package)
                    })
                    .collect();
                set.and_then(|s| weft_engine::modloader::use_packages(&s))
                    .map_err(|e| format!("the world's packages: {e}"))?;
                let h = p2p::content_hash();
                log::info!("bot {n}: the world plays with {} package(s); content hash {h:016x}", offers.len());
                *world.content.lock() = Some(h);
                h
            }
        }
    };
    let mut w = codec::W::default();
    hash.put(&mut w);
    p2p::write_frame(&mut send, &w.0).await?;
    let verdict = tokio::time::timeout(Duration::from_secs(30), p2p::read_frame_max(&mut recv, p2p::MAX_HELLO))
        .await
        .map_err(|_| "no verdict".to_string())??;
    if !verdict.is_empty() {
        return Err(String::from_utf8_lossy(&verdict).into_owned());
    }
    Ok((ep, conn, send, recv, t0))
}

/// What the reader saw, for the player loop.
#[derive(Default)]
struct Seen {
    welcome: Option<(DVec3, Instant)>,
    /// When the last cube, list of saved cubes, edits or block changes came: the view settles
    /// once these stop for a while.
    last_cube: Option<Instant>,
    pings: HashMap<u32, Instant>,
    died: bool,
    teleport: Option<DVec3>,
    ended: Option<String>,
}

async fn play(
    n: usize,
    o: &Opts,
    stats: Arc<Stats>,
    conn: &iroh::endpoint::Connection,
    mut send: iroh::endpoint::SendStream,
    mut recv: iroh::endpoint::RecvStream,
    t0: Instant,
) -> String {
    let seen = Arc::new(Mutex::new(Seen::default()));
    let reader = {
        let (seen, stats) = (seen.clone(), stats.clone());
        tokio::spawn(async move {
            loop {
                let frame = match p2p::read_frame_max(&mut recv, p2p::MAX_FRAME).await {
                    Ok(f) => f,
                    Err(e) => {
                        seen.lock().ended.get_or_insert(format!("read: {e}"));
                        break;
                    }
                };
                let Some(&tag) = frame.first() else { continue };
                let k = (tag as usize).min(63);
                stats.frames_in[k].fetch_add(1, Relaxed);
                stats.bytes_in[k].fetch_add(frame.len() as u64 + 4, Relaxed);
                let Some(&kind) = ClientTag::ALL.get(tag as usize) else { continue };
                let mut s = seen.lock();
                match kind {
                    ClientTag::Cube | ClientTag::Saved | ClientTag::Edits | ClientTag::Blocks => s.last_cube = Some(Instant::now()),
                    ClientTag::Welcome
                    | ClientTag::Teleport
                    | ClientTag::CommandReply
                    | ClientTag::Disconnect
                    | ClientTag::Refused
                    | ClientTag::Died => match codec::decode_client(&frame) {
                        Some(ToClient::Welcome { pos, .. }) => s.welcome = Some((pos, Instant::now())),
                        Some(ToClient::Teleport { pos, .. }) => s.teleport = Some(pos),
                        Some(ToClient::CommandReply { id, .. }) => {
                            if let Some(at) = s.pings.remove(&id) {
                                stats
                                    .ping_ms
                                    .lock()
                                    .push(at.elapsed().as_secs_f64() * 1000.0);
                            }
                        }
                        Some(ToClient::Disconnect(why)) => {
                            s.ended.get_or_insert(format!("disconnect: {why}"));
                        }
                        Some(ToClient::Refused(_)) => {
                            s.ended.get_or_insert("refused".into());
                        }
                        Some(ToClient::Died) => s.died = true,
                        _ => {}
                    },
                    _ => {}
                }
            }
        })
    };
    let out = |m: &ToServer| {
        let bytes = codec::encode_server(m);
        stats.frames_out.fetch_add(1, Relaxed);
        stats.bytes_out.fetch_add(bytes.len() as u64 + 4, Relaxed);
        bytes
    };
    let name = format!("bot{n}");
    for m in [ToServer::Join { name, key: String::new(), account: String::new(), did: String::new() }, ToServer::View(12)] {
        let b = out(&m);
        if let Err(e) = p2p::write_frame(&mut send, &b).await {
            reader.abort();
            return format!("write: {e}");
        }
    }
    let mut r = rng(n as u64 + 1);
    let mut tick = tokio::time::interval(Duration::from_millis(50));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let (mut pos, mut home) = (DVec3::ZERO, DVec3::ZERO);
    let mut heading = r() * std::f64::consts::TAU;
    let mut turn_in = 0u32;
    let mut ticks = 0u64;
    let mut ping_id = 0u32;
    let mut view_done = false;
    let mut mining: Option<(IVec3, u64)> = None;
    let why = loop {
        tick.tick().await;
        ticks += 1;
        let (welcome, last_cube, died, teleport, ended) = {
            let mut s = seen.lock();
            (s.welcome, s.last_cube, std::mem::take(&mut s.died), s.teleport.take(), s.ended.clone())
        };
        if let Some(e) = ended {
            break e;
        }
        let Some((spawn, welcomed)) = welcome else {
            if t0.elapsed() > Duration::from_secs(60) {
                break "no welcome in 60 s".into();
            }
            continue;
        };
        if home == DVec3::ZERO {
            stats.joined.fetch_add(1, Relaxed);
            stats.join_s.lock().push((welcomed - t0).as_secs_f64());
            // Spread: a home of its own within `spread` blocks; above the ground, flying.
            let (a, d) = (r() * std::f64::consts::TAU, o.spread * r().sqrt());
            home = spawn + DVec3::new(a.cos() * d, 40.0, a.sin() * d);
            pos = home;
        }
        if !view_done {
            let settled = last_cube.unwrap_or(welcomed);
            if settled.elapsed() > Duration::from_secs(3) || welcomed.elapsed() > Duration::from_secs(120) {
                view_done = true;
                stats.view_s.lock().push((settled - welcomed).as_secs_f64());
            }
        }
        if let Some(t) = teleport {
            pos = t;
        }
        let mut msgs: Vec<ToServer> = Vec::with_capacity(2);
        if died {
            msgs.push(ToServer::Respawn);
        }
        // A random walk at walking speed (4.3 blocks a second), turning every few seconds and back
        // toward home when it strays past 32 blocks from it.
        if turn_in == 0 {
            heading = if (pos - home).length() > 32.0 { (home.z - pos.z).atan2(home.x - pos.x) } else { heading + (r() - 0.5) * 2.0 };
            turn_in = 60 + (r() * 100.0) as u32;
        }
        turn_in -= 1;
        pos += DVec3::new(heading.cos(), 0.0, heading.sin()) * (4.3 / 20.0);
        let yaw = (-(heading.cos()).atan2(-(heading.sin()))) as f32;
        msgs.push(ToServer::Move {
            pos,
            yaw,
            pitch: 0.0,
            on_ground: true,
            pose: weft_engine::world::Pose::Stand,
            sprint: false,
            flying: true,
            frame: None,
            keys: 1,
        });
        // Now and then: dig the block under it (start, then finish a second later).
        match mining {
            Some((at, done)) if ticks >= done => {
                msgs.push(ToServer::Break { pos: at, face: 1, frame: 0 });
                mining = None;
            }
            None if r() < 1.0 / (30.0 * 20.0) => {
                let at = (pos - DVec3::new(0.0, 1.0, 0.0)).floor().as_ivec3();
                msgs.push(ToServer::StartBreak { pos: at, frame: 0 });
                mining = Some((at, ticks + 20));
            }
            _ => {}
        }
        if o.chat > 0.0 && r() < 1.0 / (o.chat * 20.0) {
            msgs.push(ToServer::Chat(format!("bot{n} says hello")));
        }
        if ticks % 200 == (n as u64 % 200) {
            ping_id += 1;
            seen.lock().pings.insert(ping_id, Instant::now());
            msgs.push(ToServer::Command { id: ping_id, line: "bot_ping".into() });
        }
        let mut failed = None;
        for m in &msgs {
            let b = out(m);
            if let Err(e) = p2p::write_frame(&mut send, &b).await {
                failed = Some(format!("write: {e}"));
                break;
            }
        }
        if let Some(e) = failed {
            break e;
        }
        if ticks.is_multiple_of(100) {
            let st = conn.stats();
            stats
                .wire
                .lock()
                .insert(n, (st.udp_rx.bytes, st.udp_tx.bytes));
        }
    };
    reader.abort();
    why
}
