//! The sponsor itself, without the network: checks a request, holds it in the queue, launches the
//! queue's DIDs together in one bundle, and watches them until the chain confirms them.
//!
//! Threads: callers (the iroh handlers in [`crate::net`], on tokio's blocking pool) wait in
//! [`Sponsor::request`] for their answer; one worker thread owns the chain work: it batches the
//! queue (requests arriving within `batch_window_ms` of the first go together, up to
//! `max_batch`), signs and pushes the bundle through the chain reader, writes the log, answers,
//! and checks pending launches for confirmation every couple of seconds.
//!
//! A request carrying `world` asks for a world DID (plan phase 4, slice 7): launched to the
//! owner's address with metadata naming the owner's DID and the world's key and generation, in
//! the same bundles as player DIDs, with a gift of `gift_mojos` to the owner's address. Before it
//! is queued the owner's DID is resolved on chain and must be held by the request's address.
use crate::config::Config;
use crate::store::{self, DAY, Event, Kind, Launch, Status, Store, WorldRecord};
use chiacraft_wallet::{
    IdentityKey, Mnemonic,
    chain::{self, Bytes32, ChainConfig, ChainNetwork, ChainReader, EveAsk, Phase, PushOutcome, WalletKeys, WorldMeta, reader::Dialer},
    sponsor::{Request, WorldAsk},
};
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, mpsc};
use std::time::{Duration, Instant};

/// A launched DID, as a DID request is answered (`200`): `did:chia:1…`, its launcher id and eve
/// coin id (64 hex digits each), and `status` `pending` (pushed, not yet confirmed). The wallet's
/// [`chiacraft_wallet::sponsor::Reply`], which the game reads it with.
pub use chiacraft_wallet::sponsor::Reply as DidReply;

/// A refused request: the status (HTTP's codes) and the reason (`{"error": …}`). An identity
/// already sponsored also names its DID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    pub status: u16,
    pub error: String,
    pub did: Option<(String, String)>,
}

impl Refusal {
    pub fn new(status: u16, error: impl Into<String>) -> Self {
        Self { status, error: error.into(), did: None }
    }

    /// The answer's body: `{"error": …}`, with `did` and `launcher` when it names one.
    pub fn body(&self) -> serde_json::Value {
        let mut body = serde_json::json!({ "error": self.error });
        if let Some((did, launcher)) = &self.did {
            body["did"] = did.as_str().into();
            body["launcher"] = launcher.as_str().into();
        }
        body
    }
}

/// The answer to `status`.
#[derive(Clone, Debug, Serialize)]
pub struct StatusReply {
    pub network: String,
    /// `connecting`, `syncing`, `synced`, `offline`.
    pub phase: Phase,
    /// The chain's peak height as last seen.
    pub height: Option<u32>,
    /// The funding wallet's address (send test coins here to fund it).
    pub address: String,
    /// The funding wallet's balance, and the part not held by unconfirmed spends, in mojos.
    pub balance: u64,
    pub spendable: u64,
    pub budget_left_today: u64,
    pub sponsored_today: usize,
    /// Launches waiting for the chain, and requests waiting in the queue.
    pub pending: usize,
    pub queued: usize,
    pub fee_per_did: u64,
    pub per_endpoint_per_day: u32,
    /// The proof of work a request must carry, in zero bits.
    pub work_bits: u32,
    /// World DIDs one identity may have launched per UTC day.
    pub worlds_per_identity_per_day: u32,
    /// Mojos sent with each world DID to the owner's address.
    pub gift_mojos: u64,
}

/// A DID this sponsor launched, and where it is.
#[derive(Clone, Debug, Serialize)]
pub struct DidState {
    pub did: String,
    pub launcher: String,
    pub coin: String,
    #[serde(flatten)]
    pub status: Status,
}

/// A world DID's ask, checked: its metadata and what the log keeps of it.
#[derive(Clone, Debug)]
struct WorldJob {
    meta: WorldMeta,
    record: WorldRecord,
}

struct Job {
    identity: String,
    address: String,
    p2: Bytes32,
    /// A world DID's (None: a player's).
    world: Option<WorldJob>,
    /// Mojos sent with the DID to the owner's address.
    gift: u64,
    /// [`store::client_hash`] of the asker.
    client: String,
    cost: u64,
    arrived: Instant,
    deadline: Instant,
    reply: mpsc::SyncSender<Result<DidReply, Refusal>>,
}

#[derive(Default)]
struct Queue {
    jobs: VecDeque<Job>,
    /// Identities with a player request in the queue or in a bundle being made.
    identities: HashSet<String>,
    /// World keys with a request in the queue or in a bundle being made.
    world_keys: HashSet<String>,
    /// Identity → its world requests there.
    worlds: HashMap<String, u32>,
    /// Mojos and askers' launches those requests hold.
    reserved: u64,
    clients: HashMap<String, u32>,
}

impl Queue {
    fn release(&mut self, job: &Job) {
        match &job.world {
            Some(w) => {
                self.world_keys.remove(&w.record.key);
                if let Some(n) = self.worlds.get_mut(&job.identity) {
                    *n = n.saturating_sub(1);
                }
            }
            None => {
                self.identities.remove(&job.identity);
            }
        }
        self.reserved = self.reserved.saturating_sub(job.cost);
        if let Some(n) = self.clients.get_mut(&job.client) {
            *n = n.saturating_sub(1);
        }
    }
}

struct Inner {
    config: Config,
    network: ChainNetwork,
    keys: WalletKeys,
    reader: ChainReader,
    store: Mutex<Store>,
    queue: Mutex<Queue>,
    wake: Condvar,
    stop: AtomicBool,
}

impl Inner {
    // Lock order: the store, then the queue.
    fn store(&self) -> MutexGuard<'_, Store> {
        self.store.lock().unwrap_or_else(|e| e.into_inner())
    }
    fn queue(&self) -> MutexGuard<'_, Queue> {
        self.queue.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// The running sponsor. Dropping it stops the worker (queued requests are refused) and the
/// chain reader (its cache is saved).
pub struct Sponsor {
    inner: Arc<Inner>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl std::fmt::Debug for Sponsor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sponsor({}, {:?})", self.inner.network, self.inner.keys)
    }
}

pub fn now_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

const CHAIN_TIMEOUT: Duration = Duration::from_secs(30);

impl Sponsor {
    /// Start on the network's peers (the DNS introducers, or `config.peers`).
    pub fn start(config: Config, words: &Mnemonic) -> Result<Self, String> {
        Self::start_inner(config, words, ChainReader::start)
    }

    /// Start with a given way to reach peers (the tests' simulator).
    pub fn start_with(config: Config, words: &Mnemonic, dialer: impl Dialer) -> Result<Self, String> {
        Self::start_inner(config, words, |c| ChainReader::start_with(c, dialer))
    }

    fn start_inner(
        config: Config,
        words: &Mnemonic,
        reader: impl FnOnce(ChainConfig) -> chiacraft_wallet::Result<ChainReader>,
    ) -> Result<Self, String> {
        let network = config.chain_network()?;
        let keys = WalletKeys::from_mnemonic(words, network, config.addresses);
        let store = Store::open(&config.data_dir.join("sponsor-log.jsonl"))?;
        let mut chain_config = ChainConfig::chosen(network.address_network(), config.consent());
        chain_config.caps = config.caps()?;
        if network == ChainNetwork::Mainnet {
            log::warn!(
                "{} (fee_per_did {} mojos, daily budget {} mojos, max_fee_per_did {} mojos)",
                chiacraft_wallet::spending::mainnet_warning("The sponsor"),
                config.fee_per_did,
                config.daily_budget_mojos,
                config.max_fee_per_did
            );
        }
        chain_config.peers = config.all_peers();
        chain_config.cache_file = Some(config.data_dir.join("chain-cache.json"));
        chain_config.puzzle_hashes = keys.puzzle_hashes();
        let reader = reader(chain_config).map_err(|e| format!("the chain reader: {e}"))?;
        let funding = chain::address(keys.primary(), network);
        let pending = store.pending().len();
        log::info!("sponsor on {network}: funding address {funding}, {pending} launches pending");
        let inner = Arc::new(Inner {
            config,
            network,
            keys,
            reader,
            store: Mutex::new(store),
            queue: Mutex::new(Queue::default()),
            wake: Condvar::new(),
            stop: AtomicBool::new(false),
        });
        let worker = {
            let inner = inner.clone();
            std::thread::Builder::new()
                .name("sponsor".into())
                .spawn(move || work(&inner))
                .map_err(|e| e.to_string())?
        };
        Ok(Self { inner, worker: Some(worker) })
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    /// The funding wallet's primary address.
    pub fn address(&self) -> String {
        chain::address(self.inner.keys.primary(), self.inner.network)
    }

    /// The funding wallet's primary puzzle hash.
    pub fn puzzle_hash(&self) -> Bytes32 {
        self.inner.keys.primary()
    }

    pub fn wait_synced(&self, timeout: Duration) -> bool {
        self.inner.reader.wait_synced(timeout)
    }

    /// Check a request without queueing it: the owner's puzzle hash and the identity (hex), or
    /// why it is refused (400 malformed, bad signature or time; 403 another network).
    pub fn check(&self, req: &Request, now: u64) -> Result<(Bytes32, String), Refusal> {
        if req.network != self.inner.network.name() {
            return Err(Refusal::new(403, format!("network {:?} not served: this sponsor serves {}", req.network, self.inner.network)));
        }
        let p2 = chain::parse_address(&req.address)
            .ok()
            .filter(|ph| chain::address(*ph, self.inner.network) == req.address)
            .ok_or_else(|| Refusal::new(400, format!("address: not a {} address", self.inner.network.address_network().prefix())))?;
        let identity = IdentityKey::from_hex(&req.identity)
            .filter(|_| req.identity.len() == 96)
            .ok_or_else(|| Refusal::new(400, "identity: not a public key (96 hex digits)"))?;
        let skew = self.inner.config.max_skew_seconds;
        if now.abs_diff(req.time) > skew {
            return Err(Refusal::new(400, format!("time: more than {skew} seconds from the sponsor's clock ({now})")));
        }
        // The wallet's check: the identity key's signature of `sponsor::text(address, time)`.
        if req.signature.len() != 192 || !req.verify() {
            return Err(Refusal::new(400, "signature: not the identity key's signature of the request"));
        }
        let bits = self.inner.config.work_bits;
        if req.work_bits() < bits {
            return Err(Refusal::new(400, format!("work: this sponsor asks for {bits} bits of proof of work")));
        }
        if let Some(world) = &req.world {
            world_job(world)?;
        }
        Ok((p2, identity.hex()))
    }

    /// A world request's owner check, on chain: the owner's DID resolves and is held by `p2` (the
    /// request's address). 403 when the chain says no (no such DID, not a DID, held by another
    /// address or melted); 503 when the chain can't answer, or when the owner's DID is one this
    /// sponsor launched that the chain hasn't confirmed yet.
    fn check_owner(&self, world: &WorldJob, p2: Bytes32) -> Result<(), Refusal> {
        let inner = &self.inner;
        if !inner.reader.synced() {
            return Err(Refusal::new(503, "the sponsor isn't synced with the chain yet"));
        }
        match inner.reader.resolve_did(world.meta.owner, CHAIN_TIMEOUT) {
            Ok(lookup) if !lookup.live => Err(Refusal::new(503, "the chain didn't answer: the owner's DID can't be checked now")),
            Ok(lookup) if lookup.record.owned_by(p2) => Ok(()),
            Ok(_) => Err(Refusal::new(403, "world.owner_did: that DID isn't held by this address")),
            // The chain's own answers that there is no such DID (the reader's words for them),
            // unless it is one this sponsor just launched and the chain hasn't confirmed yet.
            Err(e) if chain_said_no(&e.to_string()) => {
                let pending = inner
                    .store()
                    .launch(&world.record.owner_did)
                    .is_some_and(|l| l.status == Status::Pending);
                if pending {
                    Err(Refusal::new(503, "world.owner_did: that DID's launch isn't confirmed yet: try again once it is"))
                } else {
                    Err(Refusal::new(403, format!("world.owner_did: {e}")))
                }
            }
            Err(e) => Err(Refusal::new(503, format!("the chain didn't answer: the owner's DID can't be checked now ({e})"))),
        }
    }

    /// Ask for a DID: checked, queued, launched with the requests around it, and answered once
    /// pushed (blocking up to `max_wait_seconds` and a bit). `client` names the asker (its iroh
    /// endpoint id): only a daily hash of it is kept, to count its launches.
    ///
    /// A world request (`world`) is checked on chain first (the owner's DID held by the address),
    /// then limited to one DID per world key (a second answers 403 naming it, as a player's
    /// second does) and `worlds_per_identity_per_day` per identity; it doesn't use up the
    /// identity's player DID.
    pub fn request(&self, req: &Request, client: &str) -> Result<DidReply, Refusal> {
        let inner = &self.inner;
        let now = now_seconds();
        let (p2, identity) = self.check(req, now)?;
        let config = &inner.config;
        let day = now / DAY;
        let client = store::client_hash(day, client);
        let world = req.world.as_ref().map(world_job).transpose()?;
        let (cost, gift) = match world {
            Some(_) => (config.cost_per_world(), config.gift_mojos),
            None => (config.cost_per_did(), config.player_gift_mojos),
        };
        if let Some(w) = &world {
            // The day's limit before asking the chain (a world already sponsored is answered
            // whatever the count), then the owner on chain, outside the locks.
            {
                let store = inner.store();
                let q = inner.queue();
                if store.by_world_key(&w.record.key).is_none() {
                    world_limit(config, &store, &q, day, &identity)?;
                }
            }
            self.check_owner(w, p2)?;
        }
        let (tx, rx) = mpsc::sync_channel(1);
        {
            let store = inner.store();
            let mut q = inner.queue();
            match &world {
                Some(w) => {
                    if let Some(l) = store.by_world_key(&w.record.key) {
                        return Err(Refusal {
                            status: 403,
                            error: "this world already has a sponsored DID".into(),
                            did: Some((did_of(&l.launcher), l.launcher.clone())),
                        });
                    }
                    if q.world_keys.contains(&w.record.key) {
                        return Err(Refusal::new(429, "a request for this world is already in progress"));
                    }
                    world_limit(config, &store, &q, day, &identity)?;
                }
                None => {
                    if let Some(l) = store.by_identity(&identity) {
                        return Err(Refusal {
                            status: 403,
                            error: "this identity already has a sponsored DID".into(),
                            did: Some((did_of(&l.launcher), l.launcher.clone())),
                        });
                    }
                    if q.identities.contains(&identity) {
                        return Err(Refusal::new(429, "a request for this identity is already in progress"));
                    }
                }
            }
            let from_client = store.client_count(day, &client) + q.clients.get(&client).copied().unwrap_or(0);
            if from_client >= config.per_endpoint_per_day {
                return Err(Refusal::new(
                    429,
                    format!("{} DIDs per endpoint per day: try again tomorrow (UTC)", config.per_endpoint_per_day),
                ));
            }
            if store.spent_on(day) + q.reserved + cost > config.daily_budget_mojos {
                return Err(Refusal::new(429, "today's budget is spent: try again tomorrow (UTC)"));
            }
            if store.spent_on(day) + q.reserved + cost > released(config, now) {
                return Err(Refusal::new(429, "this hour's share of today's budget is spent: try again within the hour"));
            }
            if q.jobs.len() >= config.max_queue {
                return Err(Refusal::new(429, "too many requests waiting: try again soon"));
            }
            if !inner.reader.synced() {
                return Err(Refusal::new(503, "the sponsor isn't synced with the chain yet"));
            }
            let arrived = Instant::now();
            match &world {
                Some(w) => {
                    q.world_keys.insert(w.record.key.clone());
                    *q.worlds.entry(identity.clone()).or_default() += 1;
                }
                None => {
                    q.identities.insert(identity.clone());
                }
            }
            q.reserved += cost;
            *q.clients.entry(client.clone()).or_default() += 1;
            q.jobs.push_back(Job {
                identity,
                address: req.address.clone(),
                p2,
                world,
                gift,
                client,
                cost,
                arrived,
                deadline: arrived + Duration::from_secs(config.max_wait_seconds),
                reply: tx,
            });
        }
        inner.wake.notify_all();
        rx.recv_timeout(Duration::from_secs(config.max_wait_seconds) + CHAIN_TIMEOUT * 2)
            .unwrap_or_else(|_| Err(Refusal::new(503, "the sponsor didn't answer in time")))
    }

    pub fn status(&self) -> StatusReply {
        let inner = &self.inner;
        let chain = inner.reader.status();
        let day = now_seconds() / DAY;
        let store = inner.store();
        let q = inner.queue();
        StatusReply {
            network: inner.network.name().into(),
            phase: chain.phase,
            height: chain.peak,
            address: self.address(),
            balance: chain.balance,
            spendable: inner.reader.spendable_balance(),
            budget_left_today: inner
                .config
                .daily_budget_mojos
                .saturating_sub(store.spent_on(day) + q.reserved),
            sponsored_today: store.launched_on(day),
            pending: store.pending().len(),
            queued: q.jobs.len(),
            fee_per_did: inner.config.fee_per_did,
            per_endpoint_per_day: inner.config.per_endpoint_per_day,
            work_bits: inner.config.work_bits,
            worlds_per_identity_per_day: inner.config.worlds_per_identity_per_day,
            gift_mojos: inner.config.gift_mojos,
        }
    }

    /// A DID this sponsor launched, by `did:chia:1…` or launcher id.
    pub fn did(&self, text: &str) -> Option<DidState> {
        let launcher = chain::hex32(chain::parse_did(text).ok()?);
        let store = self.inner.store();
        let l = store.launch(&launcher)?;
        Some(DidState { did: did_of(&l.launcher), launcher: l.launcher.clone(), coin: l.coin.clone(), status: l.status })
    }

    /// The player DID launch record of an identity (hex), if any and not dropped.
    pub fn launch_of(&self, identity: &str) -> Option<Launch> {
        self.inner.store().by_identity(identity).cloned()
    }

    /// The world DID launch record of a world key (64 hex), if any and not dropped.
    pub fn world_launch(&self, key: &str) -> Option<Launch> {
        self.inner
            .store()
            .by_world_key(&key.to_ascii_lowercase())
            .cloned()
    }

    /// Bundles pushed since the log began.
    pub fn bundles(&self) -> usize {
        self.inner.store().bundles()
    }

    /// Check pending launches against the chain now (the worker does it every 2 seconds).
    pub fn check_pending(&self) {
        check_pending(&self.inner, &mut HashSet::new());
    }
}

impl Drop for Sponsor {
    fn drop(&mut self) {
        self.inner.stop.store(true, Ordering::SeqCst);
        self.inner.wake.notify_all();
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}

/// A world ask's fields checked (400 when malformed): 64 hex digits each (kept in lower case).
fn world_job(world: &WorldAsk) -> Result<WorldJob, Refusal> {
    let hex64 = |text: &str, what: &str| {
        (text.len() == 64 && text.bytes().all(|b| b.is_ascii_hexdigit()))
            .then(|| text.to_ascii_lowercase())
            .ok_or_else(|| Refusal::new(400, format!("world.{what}: not 64 hex digits")))
    };
    let owner_did = hex64(&world.owner_did, "owner_did")?;
    let key = hex64(&world.key, "key")?;
    let owner = chain::parse_hex32(&owner_did).map_err(|_| Refusal::new(400, "world.owner_did: not 64 hex digits"))?;
    let key_bytes = chain::parse_hex32(&key).map_err(|_| Refusal::new(400, "world.key: not 64 hex digits"))?;
    Ok(WorldJob {
        meta: WorldMeta { owner, key: key_bytes.to_bytes(), generation: world.generation },
        record: WorldRecord { key, owner_did, generation: world.generation },
    })
}

/// The identity's world DIDs today (launched and waiting) against `worlds_per_identity_per_day`.
fn world_limit(config: &Config, store: &Store, q: &Queue, day: u64, identity: &str) -> Result<(), Refusal> {
    let n = store.worlds_on(day, identity) + q.worlds.get(identity).copied().unwrap_or(0);
    if n >= config.worlds_per_identity_per_day {
        return Err(Refusal::new(
            429,
            format!("{} world DIDs per identity per day: try again tomorrow (UTC)", config.worlds_per_identity_per_day),
        ));
    }
    Ok(())
}

/// Whether a DID lookup's error is the chain's answer (no such DID, not a DID) rather than no
/// answer: the chain reader's words for those (`chiacraft_wallet::chain::did`).
fn chain_said_no(error: &str) -> bool {
    ["no such DID", "not a DID", "the launcher made no singleton", "a coin in the DID's line"]
        .iter()
        .any(|said| error.starts_with(said))
}

fn did_of(launcher: &str) -> String {
    chain::parse_hex32(launcher)
        .map(chain::did_string)
        .unwrap_or_default()
}

/// The worker: batches, launches, confirms.
fn work(inner: &Inner) {
    let mut watching = HashSet::new();
    let mut last_check = Instant::now() - Duration::from_secs(60);
    let window = Duration::from_millis(inner.config.batch_window_ms);
    loop {
        if inner.stop.load(Ordering::SeqCst) {
            let mut q = inner.queue();
            while let Some(job) = q.jobs.pop_front() {
                q.release(&job);
                let _ = job
                    .reply
                    .send(Err(Refusal::new(503, "the sponsor is stopping")));
            }
            return;
        }
        // Wait for work: the first request's window to close, or a second for the checks.
        let batch = {
            let mut q = inner.queue();
            let wait = match q.jobs.front() {
                Some(first) => window.saturating_sub(first.arrived.elapsed()),
                None => Duration::from_secs(1),
            };
            if !wait.is_zero() && q.jobs.len() < inner.config.max_batch {
                q = inner
                    .wake
                    .wait_timeout(q, wait.min(Duration::from_secs(1)))
                    .unwrap_or_else(|e| e.into_inner())
                    .0;
            }
            let ready = q
                .jobs
                .front()
                .is_some_and(|f| f.arrived.elapsed() >= window)
                || q.jobs.len() >= inner.config.max_batch;
            if ready {
                let batch = take_batch(inner, &mut q);
                if batch.is_empty() && !q.jobs.is_empty() {
                    // Nothing is affordable until a coin frees: look again in a moment.
                    drop(
                        inner
                            .wake
                            .wait_timeout(q, Duration::from_millis(500))
                            .unwrap_or_else(|e| e.into_inner()),
                    );
                }
                batch
            } else {
                Vec::new()
            }
        };
        if !batch.is_empty() {
            launch(inner, batch, &mut watching);
        }
        if last_check.elapsed() >= Duration::from_secs(2) {
            check_pending(inner, &mut watching);
            last_check = Instant::now();
        }
    }
}

/// Take as many queued requests as the spendable coins pay for (up to `max_batch`). Requests
/// that no coin can pay for yet stay queued (a coin frees when a pushed bundle confirms) until
/// their deadline.
fn take_batch(inner: &Inner, q: &mut Queue) -> Vec<Job> {
    let now = Instant::now();
    while let Some(i) = q.jobs.iter().position(|j| j.deadline <= now) {
        let job = q.jobs.remove(i).expect("found");
        q.release(&job);
        let _ = job
            .reply
            .send(Err(Refusal::new(503, "no coin of the sponsor is free (its coins are in unconfirmed spends or spent): try again later")));
    }
    // In order, as many as the coins cover: each a mojo, its fee and its gift (its cost).
    let spendable = inner.reader.spendable_balance();
    let mut total = 0u64;
    let mut n = 0;
    for job in q.jobs.iter().take(inner.config.max_batch) {
        total = total.saturating_add(job.cost.max(1));
        if total > spendable {
            break;
        }
        n += 1;
    }
    q.jobs.drain(..n).collect()
}

/// Launch a batch in one bundle and answer each request.
fn launch(inner: &Inner, batch: Vec<Job>, watching: &mut HashSet<String>) {
    let asks: Vec<EveAsk> = batch
        .iter()
        .map(|j| EveAsk { owner: j.p2, world: j.world.as_ref().map(|w| w.meta), gift: j.gift })
        .collect();
    let fee = inner.config.fee_per_did.saturating_mul(batch.len() as u64);
    let refuse_all = |batch: Vec<Job>, refusal: Refusal| {
        let mut q = inner.queue();
        for job in batch {
            q.release(&job);
            let _ = job.reply.send(Err(refusal.clone()));
        }
    };
    let (built, txid) = match inner
        .reader
        .launch_dids_with(&inner.keys, &asks, fee, CHAIN_TIMEOUT)
    {
        Ok((built, PushOutcome::Accepted { txid } | PushOutcome::Pending { txid })) => (built, txid),
        Ok((_, PushOutcome::Rejected { error, .. })) => {
            log::warn!("the chain refused a bundle of {} DIDs: {error}", batch.len());
            return refuse_all(batch, Refusal::new(503, format!("the chain refused the spend: {error}")));
        }
        Err(e) => {
            log::warn!("a bundle of {} DIDs failed: {e}", batch.len());
            return refuse_all(batch, Refusal::new(503, format!("the launch failed: {e}")));
        }
    };
    let now = now_seconds();
    let txid = chain::hex32(txid);
    let count = built.dids.len();
    let worlds = batch.iter().filter(|j| j.world.is_some()).count();
    let gifts = built.gifts;
    log::info!("pushed {count} DID(s) ({worlds} for worlds) in one bundle (transaction {txid}, fee {fee} mojos, gifts {gifts} mojos)");
    let mut store = inner.store();
    let logged = store.record(Event::Bundle {
        time: now,
        txid: txid.clone(),
        dids: built.dids.len(),
        fee,
        change: built.change,
        spent: built.spent.iter().map(|c| chain::hex32(*c)).collect(),
    });
    if let Err(e) = logged {
        log::error!("{e}");
    }
    let mut q = inner.queue();
    for (job, did) in batch.into_iter().zip(&built.dids) {
        let launcher = chain::hex32(did.launcher_id);
        let coin = chain::hex32(did.coin.coin_id());
        if let Err(e) = store.record(Event::Sponsored {
            time: now,
            identity: job.identity.clone(),
            address: job.address.clone(),
            launcher: launcher.clone(),
            coin: coin.clone(),
            cost: job.cost,
            client: job.client.clone(),
            txid: txid.clone(),
            kind: if job.world.is_some() { Kind::World } else { Kind::Player },
            world: job.world.as_ref().map(|w| w.record.clone()),
        }) {
            log::error!("{e}");
        }
        q.release(&job);
        let _ = job
            .reply
            .send(Ok(DidReply { did: chain::did_string(did.launcher_id), launcher, coin, status: "pending".into() }));
    }
    drop(q);
    drop(store);
    for did in &built.dids {
        let coin = did.coin.coin_id();
        if inner.reader.watch_coin(coin, CHAIN_TIMEOUT).is_ok() {
            watching.insert(chain::hex32(coin));
        }
    }
}

/// Mark pending launches confirmed once their eve coin is on chain, or dropped once they have
/// waited `drop_after_seconds` and still aren't.
fn check_pending(inner: &Inner, watching: &mut HashSet<String>) {
    let pending = inner.store().pending();
    let now = now_seconds();
    for l in pending {
        let Ok(coin) = chain::parse_hex32(&l.coin) else {
            continue;
        };
        let mut state = inner.reader.watched(coin).and_then(|w| w.state);
        let too_old = now.saturating_sub(l.time) > inner.config.drop_after_seconds;
        if !watching.contains(&l.coin) || (state.is_none() && too_old) {
            // Watch it (again after a restart, or a last live look before dropping it).
            if let Ok(w) = inner.reader.watch_coin(coin, CHAIN_TIMEOUT) {
                watching.insert(l.coin.clone());
                state = w.state;
            }
        }
        let event = match state.and_then(|s| s.created) {
            Some(height) => {
                log::info!("{} confirmed at height {height}", did_of(&l.launcher));
                Event::Confirmed { time: now, launcher: l.launcher.clone(), height }
            }
            None if too_old && inner.reader.synced() => {
                log::warn!("{} not confirmed after {} s: dropped", did_of(&l.launcher), inner.config.drop_after_seconds);
                Event::Dropped {
                    time: now,
                    launcher: l.launcher.clone(),
                    reason: format!("not on chain {} seconds after the push", inner.config.drop_after_seconds),
                }
            }
            None => continue,
        };
        if let Err(e) = inner.store().record(event) {
            log::error!("{e}");
        }
        inner.reader.unwatch(coin);
        watching.remove(&l.coin);
    }
}

/// What of today's budget may be spent by `now`: all of it, or with `hourly_release` the hours
/// begun so far (hour h of the UTC day releases (h + 1) / 24).
pub fn released(config: &Config, now: u64) -> u64 {
    if !config.hourly_release {
        return config.daily_budget_mojos;
    }
    let hour = (now % DAY) / 3600;
    config.daily_budget_mojos / 24 * (hour + 1) + if hour == 23 { config.daily_budget_mojos % 24 } else { 0 }
}
