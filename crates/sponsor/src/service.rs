//! The sponsor itself, without the network: checks a request, holds it in the queue, launches the
//! queue's DIDs together in one bundle, and watches them until the chain confirms them.
//!
//! Threads: callers (the iroh handlers in [`crate::net`], on tokio's blocking pool) wait in
//! [`Sponsor::request`] for their answer; one worker thread owns the chain work: it batches the
//! queue (requests arriving within `batch_window_ms` of the first go together, up to
//! `max_batch`), signs and pushes the bundle through the chain reader, writes the log, answers,
//! and checks pending launches for confirmation every couple of seconds.
use crate::config::Config;
use crate::store::{self, DAY, Event, Launch, Status, Store};
use chiacraft_wallet::{
    IdentityKey, Mnemonic,
    chain::{self, Bytes32, ChainConfig, ChainNetwork, ChainReader, Phase, PushOutcome, WalletKeys, reader::Dialer},
    sponsor::Request,
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

struct Job {
    identity: String,
    address: String,
    p2: Bytes32,
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
    /// Identities with a request in the queue or in a bundle being made.
    identities: HashSet<String>,
    /// Mojos and askers' launches those requests hold.
    reserved: u64,
    clients: HashMap<String, u32>,
}

impl Queue {
    fn release(&mut self, job: &Job) {
        self.identities.remove(&job.identity);
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
        Ok((p2, identity.hex()))
    }

    /// Ask for a DID: checked, queued, launched with the requests around it, and answered once
    /// pushed (blocking up to `max_wait_seconds` and a bit). `client` names the asker (its iroh
    /// endpoint id): only a daily hash of it is kept, to count its launches.
    pub fn request(&self, req: &Request, client: &str) -> Result<DidReply, Refusal> {
        let inner = &self.inner;
        let now = now_seconds();
        let (p2, identity) = self.check(req, now)?;
        let config = &inner.config;
        let day = now / DAY;
        let client = store::client_hash(day, client);
        let cost = config.cost_per_did();
        let (tx, rx) = mpsc::sync_channel(1);
        {
            let store = inner.store();
            let mut q = inner.queue();
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
            q.identities.insert(identity.clone());
            q.reserved += cost;
            *q.clients.entry(client.clone()).or_default() += 1;
            q.jobs.push_back(Job {
                identity,
                address: req.address.clone(),
                p2,
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
        }
    }

    /// A DID this sponsor launched, by `did:chia:1…` or launcher id.
    pub fn did(&self, text: &str) -> Option<DidState> {
        let launcher = chain::hex32(chain::parse_did(text).ok()?);
        let store = self.inner.store();
        let l = store.launch(&launcher)?;
        Some(DidState { did: did_of(&l.launcher), launcher: l.launcher.clone(), coin: l.coin.clone(), status: l.status })
    }

    /// The launch record of an identity (hex), if any and not dropped.
    pub fn launch_of(&self, identity: &str) -> Option<Launch> {
        self.inner.store().by_identity(identity).cloned()
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
    let cost = inner.config.cost_per_did().max(1);
    let affordable = usize::try_from(inner.reader.spendable_balance() / cost).unwrap_or(usize::MAX);
    let n = q.jobs.len().min(inner.config.max_batch).min(affordable);
    q.jobs.drain(..n).collect()
}

/// Launch a batch in one bundle and answer each request.
fn launch(inner: &Inner, batch: Vec<Job>, watching: &mut HashSet<String>) {
    let owners: Vec<Bytes32> = batch.iter().map(|j| j.p2).collect();
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
        .launch_dids(&inner.keys, &owners, fee, CHAIN_TIMEOUT)
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
    log::info!("pushed {count} DID(s) in one bundle (transaction {txid}, fee {fee} mojos)");
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
