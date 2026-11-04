//! The background chain reader: its own tokio runtime (the game has none), a small pool of
//! full-node peers, one of which carries the wallet's subscription, and a blocking API for the
//! game's threads (each call bounded by a timeout).
//!
//! The pool is Sage's (and `@chia/core` `sync.ts`'s `WalletSync`): peers come from the DNS
//! introducers (or the configured list), dialled a few at a time with failures banned for a
//! while; every peer's messages are pumped so each reports its peak; the highest one carries the
//! subscription and is replaced when it dies or falls three blocks behind the pool. While no
//! peer answers, everything is answered from the cache (identity-research.md 3.7): play never
//! waits on the chain.
use super::{
    ChainCache, ChainNetwork, ChainSource, PeerSource, PushOutcome,
    cache::WatchedCoin,
    did::{self, DidRecord},
    failed, hex32,
    offer::{self, MadeOffer, MakeOffer, OfferGive, OfferSummary},
    sponsor::EveAsk,
    sync::CoinRecord,
    wallet::WalletKeys,
    world::DidMetadata,
};
use crate::{
    Error, Result,
    spending::{MainnetConsent, SpendCaps},
};
use chia_protocol::{Bytes32, CoinStateUpdate, Message, NewPeakWallet, ProtocolMessageTypes, SpendBundle};
use chia_sdk_client::{Peer, PeerOptions};
use chia_traits::Streamable;
use std::{
    collections::HashMap,
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{Notify, mpsc};

/// A peer connection: the peer and the messages it pushes.
pub type Connection = (Peer, mpsc::Receiver<Message>);
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// How the reader finds and reaches peers. [`TlsDialer`] is the real one; tests dial
/// chia-sdk-test's peer simulator.
pub trait Dialer: Send + Sync + 'static {
    /// Addresses to try (`host:port`).
    fn candidates(&self) -> BoxFuture<'_, Vec<String>>;
    fn dial(&self, address: String) -> BoxFuture<'_, Result<Connection>>;
}

/// Full nodes over TLS, found through the DNS introducers unless a list is configured.
pub struct TlsDialer {
    pub network: ChainNetwork,
    pub peers: Vec<String>,
    connector: chia_sdk_client::Connector,
}

impl TlsDialer {
    pub fn new(network: ChainNetwork, peers: Vec<String>) -> Result<Self> {
        Ok(Self { network, peers, connector: super::tls::connector()? })
    }
}

impl Dialer for TlsDialer {
    fn candidates(&self) -> BoxFuture<'_, Vec<String>> {
        Box::pin(async move {
            if !self.peers.is_empty() {
                return self.peers.clone();
            }
            let mut found: Vec<String> = self
                .network
                .introducers()
                .lookup_all(Duration::from_secs(5), 2)
                .await
                .into_iter()
                .map(|a| a.to_string())
                .collect();
            found.sort();
            found.dedup();
            // Spread the load over the introducers' answers.
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.subsec_nanos() as usize);
            if !found.is_empty() {
                let n = found.len();
                found.rotate_left(seed % n);
            }
            found
        })
    }
    fn dial(&self, address: String) -> BoxFuture<'_, Result<Connection>> {
        Box::pin(async move {
            // `ws://127.0.0.1:port`: a local test node without TLS (chia-sdk-test's peer
            // simulator, the checks' `chain_sim`). Loopback only: real peers always use TLS.
            if let Some(rest) = address.strip_prefix("ws://") {
                let host: std::net::SocketAddr = rest
                    .trim_end_matches('/')
                    .parse()
                    .map_err(failed("simulator address"))?;
                if !host.ip().is_loopback() {
                    return Err(Error::new("plain ws:// peers must be on this computer"));
                }
                let (ws, _) = tokio_tungstenite::connect_async(address.as_str())
                    .await
                    .map_err(failed("connect"))?;
                return Peer::from_websocket(ws, PeerOptions::default()).map_err(failed("connect"));
            }
            let addr: std::net::SocketAddr = match address.parse() {
                Ok(a) => a,
                Err(_) => tokio::net::lookup_host(&address)
                    .await
                    .map_err(failed("peer address"))?
                    .next()
                    .ok_or_else(|| Error::new(format!("no address for {address}")))?,
            };
            chia_sdk_client::connect_peer(self.network.network_id().to_owned(), self.connector.clone(), addr, PeerOptions::default())
                .await
                .map_err(failed("connect"))
        })
    }
}

/// What the reader is given.
#[derive(Clone, Debug)]
pub struct ChainConfig {
    pub network: ChainNetwork,
    /// Peers to use instead of the DNS introducers (`host:port`): an own node, for instance.
    pub peers: Vec<String>,
    /// How many peers to keep connected.
    pub target_peers: usize,
    /// Where the cache lives (None: memory only).
    pub cache_file: Option<PathBuf>,
    /// The wallet followed: its puzzle hashes (the primary first).
    pub puzzle_hashes: Vec<Bytes32>,
    /// The most each spend may move (I12): the network's defaults unless the caller set its own.
    pub caps: SpendCaps,
    /// Mainnet needs the operator's confirmation ([`crate::spending::choose_network`]): the
    /// reader refuses to start on mainnet without it.
    pub mainnet: Option<MainnetConsent>,
}

impl ChainConfig {
    pub fn new(network: ChainNetwork) -> Self {
        Self {
            network,
            peers: Vec::new(),
            target_peers: 3,
            cache_file: None,
            puzzle_hashes: Vec::new(),
            caps: SpendCaps::for_network(network.address_network()),
            mainnet: None,
        }
    }

    /// The network a setting chose, with its consent and its network's default caps.
    pub fn chosen(network: crate::Network, mainnet: Option<MainnetConsent>) -> Self {
        let mut config = Self::new(network.into());
        config.mainnet = mainnet;
        config
    }
}

/// Where the reader is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Connecting,
    Syncing,
    Synced,
    /// No peer answers: answers come from the cache.
    Offline,
    Stopped,
}

/// A connected peer as the status shows it.
#[derive(Clone, Debug, serde::Serialize)]
pub struct PeerInfo {
    pub address: String,
    pub peak: Option<u32>,
    pub active: bool,
}

/// The reader's state, for `chain status`.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ChainStatus {
    pub network: ChainNetwork,
    pub phase: Phase,
    pub peers: Vec<PeerInfo>,
    pub peak: Option<u32>,
    pub synced_height: Option<u32>,
    pub puzzle_hashes: usize,
    pub primary_address: Option<String>,
    /// The unspent XCH coins `balance` adds up (the cache also keeps spent and hinted ones).
    pub coins: usize,
    pub balance: u64,
    pub dids: usize,
    pub watched: usize,
    pub open_offers: usize,
    pub last_error: Option<String>,
}

/// A DID read live or from the cache.
#[derive(Clone, Debug)]
pub struct DidLookup {
    pub record: DidRecord,
    /// False: the cache's answer (no peer reachable).
    pub live: bool,
}

/// What became of an offer the reader made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase", tag = "state")]
pub enum OfferState {
    Open,
    /// Its coins were spent at `height`; `paid` once the price is seen at the payment address.
    Taken {
        height: u32,
        paid: bool,
    },
    Expired,
}

/// A taken offer, pushed.
#[derive(Clone, Debug)]
pub struct TakeOutcome {
    pub summary: OfferSummary,
    pub push: PushOutcome,
}

/// A DID whose metadata the wallet set ([`ChainReader::set_did_metadata`]), pushed.
#[derive(Clone, Debug)]
pub struct UpdatedDid {
    pub launcher_id: Bytes32,
    /// The DID's coin the update spends.
    pub spent: Bytes32,
    /// The DID's coin after the update.
    pub coin_id: Bytes32,
    /// The owner's p2 puzzle hash (one of the wallet's addresses; unchanged).
    pub owner: Bytes32,
    pub push: PushOutcome,
}

/// A DID the wallet made ([`ChainReader::create_did`]), pushed.
#[derive(Clone, Debug)]
pub struct CreatedDid {
    pub launcher_id: Bytes32,
    /// The DID's coin after its creating spend.
    pub coin_id: Bytes32,
    /// The owner's p2 puzzle hash (one of the wallet's addresses).
    pub owner: Bytes32,
    pub push: PushOutcome,
}

struct PoolPeer {
    id: u64,
    address: String,
    peer: Peer,
    peak: Option<(u32, Bytes32)>,
}

struct State {
    config: ChainConfig,
    cache: ChainCache,
    pool: Vec<PoolPeer>,
    active: Option<u64>,
    bans: HashMap<String, Instant>,
    phase: Phase,
    stop: bool,
    dirty: bool,
    last_error: Option<String>,
}

impl State {
    fn active_source(&self) -> Option<PeerSource> {
        let id = self.active?;
        self.pool
            .iter()
            .find(|p| p.id == id)
            .map(|p| PeerSource::new(p.peer.clone(), self.config.network))
    }
    /// The active peer, else any peer (reads that need no subscription).
    fn any_source(&self) -> Option<PeerSource> {
        self.active_source().or_else(|| {
            self.pool
                .iter()
                .max_by_key(|p| p.peak.map(|(h, _)| h))
                .map(|p| PeerSource::new(p.peer.clone(), self.config.network))
        })
    }
    fn save(&mut self) {
        if let Some(path) = &self.config.cache_file {
            if let Err(e) = self.cache.save(path) {
                log::warn!("chain cache {}: {e}", path.display());
            }
        }
        self.dirty = false;
    }
}

enum Event {
    Peak(u64, NewPeakWallet),
    Update(u64, CoinStateUpdate),
    Closed(u64, String),
}

struct Shared {
    state: Mutex<State>,
    wake: Notify,
}

/// The running reader. Dropping it stops the reader and saves the cache.
pub struct ChainReader {
    shared: Arc<Shared>,
    runtime: Option<tokio::runtime::Runtime>,
}

impl std::fmt::Debug for ChainReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ChainReader")
    }
}

impl ChainReader {
    /// Start reading `config.network` through its DNS introducers (or `config.peers`).
    pub fn start(config: ChainConfig) -> Result<Self> {
        let dialer = TlsDialer::new(config.network, config.peers.clone())?;
        Self::start_with(config, dialer)
    }

    /// Start with a given way to reach peers.
    pub fn start_with(config: ChainConfig, dialer: impl Dialer) -> Result<Self> {
        if config.network == ChainNetwork::Mainnet && config.mainnet.is_none() {
            return Err(Error::new(format!(
                "the chain reader won't start on mainnet without the operator's confirmation (\"{}\")",
                crate::spending::MAINNET_CONFIRM
            )));
        }
        if config.caps.network != config.network.address_network() {
            return Err(Error::new("the spending caps are for another network"));
        }
        if config.network == ChainNetwork::Mainnet {
            log::warn!("{}; caps: {}", crate::spending::mainnet_warning("The chain reader"), config.caps.describe());
        }
        let cache = match &config.cache_file {
            Some(path) => ChainCache::load(path, config.network, config.puzzle_hashes.clone()),
            None => ChainCache::new(config.network, config.puzzle_hashes.clone()),
        };
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("chain reader")
            .enable_all()
            .build()
            .map_err(failed("chain runtime"))?;
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                config,
                cache,
                pool: Vec::new(),
                active: None,
                bans: HashMap::new(),
                phase: Phase::Connecting,
                stop: false,
                dirty: false,
                last_error: None,
            }),
            wake: Notify::new(),
        });
        runtime.spawn(supervise(shared.clone(), Arc::new(dialer)));
        Ok(Self { shared, runtime: Some(runtime) })
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.shared.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Run `f` on the reader's runtime, at most `timeout`.
    fn run<T>(&self, timeout: Duration, f: impl Future<Output = Result<T>> + Send) -> Result<T> {
        let runtime = self
            .runtime
            .as_ref()
            .ok_or_else(|| Error::new("chain reader stopped"))?;
        runtime.block_on(async {
            tokio::time::timeout(timeout, f)
                .await
                .unwrap_or_else(|_| Err(Error::new("the chain didn't answer in time")))
        })
    }

    pub fn network(&self) -> ChainNetwork {
        self.state().config.network
    }

    /// The caps every spend is held to.
    pub fn caps(&self) -> SpendCaps {
        self.state().config.caps
    }

    pub fn status(&self) -> ChainStatus {
        let s = self.state();
        let wallet = &s.cache.wallet;
        ChainStatus {
            network: s.config.network,
            phase: s.phase,
            peers: s
                .pool
                .iter()
                .map(|p| PeerInfo { address: p.address.clone(), peak: p.peak.map(|(h, _)| h), active: Some(p.id) == s.active })
                .collect(),
            peak: s.cache.peak,
            synced_height: wallet.height,
            puzzle_hashes: wallet.puzzle_hashes.len(),
            primary_address: wallet
                .puzzle_hashes
                .first()
                .map(|ph| super::address(*ph, s.config.network)),
            coins: wallet.unspent_xch().len(),
            balance: wallet.balance(),
            dids: s.cache.dids.len(),
            watched: s.cache.watched.len(),
            open_offers: s.cache.offers.len(),
            last_error: s.last_error.clone(),
        }
    }

    /// Whether the wallet's coins are synced with a live peer.
    pub fn synced(&self) -> bool {
        self.state().phase == Phase::Synced
    }

    /// Wait until synced (or `timeout`).
    pub fn wait_synced(&self, timeout: Duration) -> bool {
        let until = Instant::now() + timeout;
        while Instant::now() < until {
            if self.synced() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        self.synced()
    }

    /// The wallet's coins as cached.
    pub fn coins(&self) -> Vec<CoinRecord> {
        self.state().cache.wallet.coins.values().copied().collect()
    }

    /// A copy of the cache.
    pub fn cache(&self) -> ChainCache {
        self.state().cache.clone()
    }

    /// A DID's record: read live (continuing from the cached one) and subscribed, or the cached
    /// record while no peer answers.
    pub fn resolve_did(&self, launcher_id: Bytes32, timeout: Duration) -> Result<DidLookup> {
        let key = hex32(launcher_id);
        let (source, known, peak) = {
            let s = self.state();
            (s.any_source(), s.cache.dids.get(&key).cloned(), s.cache.peak.unwrap_or(0))
        };
        let Some(source) = source else {
            return known
                .map(|record| DidLookup { record, live: false })
                .ok_or_else(|| Error::new("no chain peer reachable and the DID isn't cached"));
        };
        let record = match self.run(timeout, did::resolve(&source, launcher_id, known.clone(), peak)) {
            Ok(r) => r,
            Err(e) => {
                return known
                    .map(|record| DidLookup { record, live: false })
                    .ok_or(e);
            }
        };
        // Follow its current coin on the subscribed peer: a transfer or melt marks the cached
        // record stale (each new subscribed peer subscribes it again).
        let active = self.state().active_source();
        if let (Some(current), Some(active)) = (record.current().filter(|c| c.spent.is_none()), active) {
            let id = current.coin_id;
            let _ = self.run(timeout, async { active.coin_state(vec![id], true).await });
        }
        let mut s = self.state();
        s.cache.dids.insert(key, record.clone());
        s.save();
        Ok(DidLookup { record, live: true })
    }

    /// A cached DID record (no chain access).
    pub fn cached_did(&self, launcher_id: Bytes32) -> Option<DidRecord> {
        self.state().cache.dids.get(&hex32(launcher_id)).cloned()
    }

    /// Watch a coin until it is spent: subscribed on the active peer (and on each later one).
    pub fn watch_coin(&self, coin_id: Bytes32, timeout: Duration) -> Result<WatchedCoin> {
        let source = {
            let mut s = self.state();
            s.cache
                .watched
                .entry(hex32(coin_id))
                .or_insert(WatchedCoin { coin_id, state: None });
            s.dirty = true;
            s.active_source()
        };
        if let Some(source) = source {
            let states = self.run(timeout, async { source.coin_state(vec![coin_id], true).await })?;
            let mut s = self.state();
            if let Some(w) = s.cache.watched.get_mut(&hex32(coin_id)) {
                w.state = states.first().map(CoinRecord::from);
            }
            s.save();
        }
        self.watched(coin_id)
            .ok_or_else(|| Error::new("watch vanished"))
    }

    /// A watched coin's last known state.
    pub fn watched(&self, coin_id: Bytes32) -> Option<WatchedCoin> {
        self.state().cache.watched.get(&hex32(coin_id)).cloned()
    }

    /// Stop watching a coin.
    pub fn unwatch(&self, coin_id: Bytes32) {
        let mut s = self.state();
        s.cache.watched.remove(&hex32(coin_id));
        s.dirty = true;
    }

    /// Wait until a watched coin is spent (or `timeout`): the spend height.
    pub fn wait_spent(&self, coin_id: Bytes32, timeout: Duration) -> Option<u32> {
        let until = Instant::now() + timeout;
        loop {
            if let Some(h) = self.watched(coin_id).and_then(|w| w.spent()) {
                return Some(h);
            }
            if Instant::now() >= until {
                return None;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Run a read of `f`'s own (an admission check: [`super::holdings`]) on a live peer, at most
    /// `timeout`. An error while no peer is reachable.
    pub fn read<T, F, Fut>(&self, timeout: Duration, f: F) -> Result<T>
    where
        F: FnOnce(PeerSource) -> Fut,
        Fut: Future<Output = Result<T>> + Send,
    {
        let source = self
            .state()
            .any_source()
            .ok_or_else(|| Error::new("no chain peer reachable"))?;
        self.run(timeout, f(source))
    }

    /// Whether a live peer answers now (synced or not).
    pub fn reachable(&self) -> bool {
        !self.state().pool.is_empty()
    }

    /// Split one of the wallet's coins into `count` coins of `amount` mojos at its primary
    /// address (the change too): the pool of 1-mojo coins a server gives in admission offers
    /// (identity-research.md 3.6, step 5), so each offer locks a coin of its own rather than the
    /// wallet's whole balance. Pushed; the spent coin is kept out of other spends until settled.
    pub fn split(&self, keys: &WalletKeys, count: u32, amount: u64, fee: u64, timeout: Duration) -> Result<PushOutcome> {
        use chia_puzzle_types::Memos;
        use chia_sdk_driver::{Action, Id, Relation, SpendContext, Spends};
        let caps = self.caps();
        caps.check_fee(fee)?;
        caps.check("max-split", "the split", u64::from(count).saturating_mul(amount))?;
        self.check_keys(keys)?;
        if count == 0 || amount == 0 {
            return Err(Error::new("split into at least one coin of at least 1 mojo"));
        }
        let need = u64::from(count)
            .checked_mul(amount)
            .and_then(|n| n.checked_add(fee))
            .ok_or_else(|| Error::new("the split's amounts overflow"))?;
        let (source, coins) = {
            let s = self.state();
            (
                s.active_source()
                    .ok_or_else(|| Error::new("no chain peer reachable"))?,
                s.cache.spendable(),
            )
        };
        // The largest coin above the need: the pool comes out of one coin, the rest is change.
        let coin = coins
            .iter()
            .filter(|c| c.amount > need)
            .max_by_key(|c| c.amount)
            .copied()
            .ok_or_else(|| Error::new(format!("no coin above {need} mojos to split")))?;
        let mut ctx = SpendContext::new();
        let mut spends = Spends::new(keys.primary());
        spends.add(coin);
        let mut actions: Vec<Action> = (0..count)
            .map(|_| Action::send(Id::Xch, keys.primary(), amount, Memos::None))
            .collect();
        if fee > 0 {
            actions.push(Action::fee(fee));
        }
        let deltas = spends.apply(&mut ctx, &actions).map_err(failed("split"))?;
        spends
            .finish_with_keys(&mut ctx, &deltas, Relation::AssertConcurrent, &keys.synthetic_keys())
            .map_err(failed("split"))?;
        let coin_spends = ctx.take();
        let signature = keys.sign(&coin_spends)?;
        let push = self.run(timeout, async { source.push(SpendBundle::new(coin_spends, signature)).await })?;
        if !matches!(push, PushOutcome::Rejected { .. }) {
            let mut s = self.state();
            s.cache
                .pending
                .insert(hex32(coin.coin_id()), offer::now_seconds());
            s.save();
        }
        Ok(push)
    }

    /// Send `amount` mojos from the wallet's spendable coins to `to` (a p2 puzzle hash), paying
    /// `fee`; the change comes back to the primary address. Pushed; the spent coins are kept out of
    /// other spends until settled. Capped by `max-send` and `max-fee`.
    pub fn send(&self, keys: &WalletKeys, to: Bytes32, amount: u64, fee: u64, timeout: Duration) -> Result<PushOutcome> {
        use chia_puzzle_types::Memos;
        use chia_sdk_driver::{Action, Id, Relation, SpendContext, Spends};
        let caps = self.caps();
        caps.check_fee(fee)?;
        caps.check("max-send", "the payment", amount)?;
        self.check_keys(keys)?;
        if amount == 0 {
            return Err(Error::new("send at least 1 mojo"));
        }
        let need = amount
            .checked_add(fee)
            .ok_or_else(|| Error::new("the payment's amounts overflow"))?;
        let (source, coins) = {
            let s = self.state();
            (
                s.active_source()
                    .ok_or_else(|| Error::new("no chain peer reachable"))?,
                s.cache.spendable(),
            )
        };
        let chosen = offer::select(&coins, need)
            .map_err(|_| Error::new(format!("the wallet needs at least {need} mojos to send ({amount} + {fee} fee)")))?;
        let mut ctx = SpendContext::new();
        let mut spends = Spends::new(keys.primary());
        for c in &chosen {
            spends.add(*c);
        }
        let mut actions = vec![Action::send(Id::Xch, to, amount, Memos::None)];
        if fee > 0 {
            actions.push(Action::fee(fee));
        }
        let deltas = spends.apply(&mut ctx, &actions).map_err(failed("send"))?;
        spends
            .finish_with_keys(&mut ctx, &deltas, Relation::AssertConcurrent, &keys.synthetic_keys())
            .map_err(failed("send"))?;
        let coin_spends = ctx.take();
        let signature = keys.sign(&coin_spends)?;
        let push = self.run(timeout, async { source.push(SpendBundle::new(coin_spends, signature)).await })?;
        if !matches!(push, PushOutcome::Rejected { .. }) {
            let now = offer::now_seconds();
            let mut s = self.state();
            for c in &chosen {
                s.cache.pending.insert(hex32(c.coin_id()), now);
            }
            s.save();
        }
        Ok(push)
    }

    /// Launch one DID per owner (p2 puzzle hash) from the wallet's spendable coins, paying `fee`
    /// (identity row I9: [`super::sponsor`]), and push the bundle. The coins it spends are kept
    /// out of other spends until the chain settles them; the bundle's DIDs come back with the
    /// push's outcome (a rejected push launched nothing).
    pub fn launch_dids(
        &self,
        keys: &WalletKeys,
        owners: &[Bytes32],
        fee: u64,
        timeout: Duration,
    ) -> Result<(super::sponsor::SponsorBundle, PushOutcome)> {
        let asks: Vec<EveAsk> = owners.iter().copied().map(EveAsk::plain).collect();
        self.launch_dids_with(keys, &asks, fee, timeout)
    }

    /// [`Self::launch_dids`] for asks that may carry a world's metadata and a gift of mojos to
    /// the owner ([`super::sponsor::build_eve_dids_with`]). Each gift is held to `max-send`.
    pub fn launch_dids_with(
        &self,
        keys: &WalletKeys,
        asks: &[EveAsk],
        fee: u64,
        timeout: Duration,
    ) -> Result<(super::sponsor::SponsorBundle, PushOutcome)> {
        let n = u64::try_from(asks.len().max(1)).unwrap_or(u64::MAX);
        let caps = self.caps();
        caps.check("max-did-fee", "the fee per DID", fee.div_ceil(n))?;
        for ask in asks {
            caps.check("max-send", "a gift with a DID", ask.gift)?;
        }
        self.check_keys(keys)?;
        let (source, coins) = {
            let s = self.state();
            (
                s.active_source()
                    .ok_or_else(|| Error::new("no chain peer reachable"))?,
                s.cache.spendable(),
            )
        };
        let built = super::sponsor::build_eve_dids_with(keys, &coins, asks, fee)?;
        let bundle = built.bundle.clone();
        let push = self.run(timeout, async { source.push(bundle).await })?;
        if !matches!(push, PushOutcome::Rejected { .. }) {
            let now = offer::now_seconds();
            let mut s = self.state();
            for id in &built.spent {
                s.cache.pending.insert(hex32(*id), now);
            }
            s.save();
        }
        Ok((built, push))
    }

    /// Make a DID paid by the wallet itself (identity row I8, research 2.2 (a)): a plain DID (no
    /// recovery list, empty metadata, 1 mojo) launched from the wallet's coins, `fee` paid, held
    /// at the address of the coin it came from (chia-sdk-driver's `create_empty_did`, as Sage's
    /// `create_did`). Pushed; the spent coins are kept out of other spends until settled. Watch
    /// it confirm with [`Self::wait_did`].
    pub fn create_did(&self, keys: &WalletKeys, fee: u64, timeout: Duration) -> Result<CreatedDid> {
        use chia_sdk_driver::{Action, Id, Relation, SpendContext, Spends};
        self.caps().check("max-did-fee", "the DID's fee", fee)?;
        self.check_keys(keys)?;
        let (source, coins) = {
            let s = self.state();
            (
                s.active_source()
                    .ok_or_else(|| Error::new("no chain peer reachable"))?,
                s.cache.spendable(),
            )
        };
        let need = fee
            .checked_add(1)
            .ok_or_else(|| Error::new("the fee overflows"))?;
        let chosen = offer::select(&coins, need)
            .map_err(|_| Error::new(format!("the wallet needs at least {need} mojos to make a DID (1 for the DID, {fee} fee)")))?;
        let mut ctx = SpendContext::new();
        let mut spends = Spends::new(keys.primary());
        for c in &chosen {
            spends.add(*c);
        }
        let mut actions = vec![Action::create_empty_did()];
        if fee > 0 {
            actions.push(Action::fee(fee));
        }
        let deltas = spends
            .apply(&mut ctx, &actions)
            .map_err(failed("making a DID"))?;
        let outputs = spends
            .finish_with_keys(&mut ctx, &deltas, Relation::AssertConcurrent, &keys.synthetic_keys())
            .map_err(failed("making a DID"))?;
        let did = outputs
            .dids
            .get(&Id::New(0))
            .ok_or_else(|| Error::new("the spend made no DID"))?;
        let created = CreatedDid {
            launcher_id: did.info.launcher_id,
            coin_id: did.coin.coin_id(),
            owner: did.info.p2_puzzle_hash,
            push: PushOutcome::Pending { txid: Bytes32::default() },
        };
        let coin_spends = ctx.take();
        let signature = keys.sign(&coin_spends)?;
        let push = self.run(timeout, async { source.push(SpendBundle::new(coin_spends, signature)).await })?;
        if !matches!(push, PushOutcome::Rejected { .. }) {
            let now = offer::now_seconds();
            let mut s = self.state();
            for c in &chosen {
                s.cache.pending.insert(hex32(c.coin_id()), now);
            }
            s.save();
        }
        Ok(CreatedDid { push, ..created })
    }

    /// Set the metadata of a DID the wallet holds (plan phase 4, slice 7: an owner DID listing
    /// its worlds, [`super::world`]): one spend of the DID by its owner's key, back to the same
    /// address with `metadata` (revealed in its memos, so a resolve reads it), `fee` paid from
    /// the wallet's spendable coins (refused when they can't pay it). Held to `max-fee` and
    /// `max-did-fee`. Pushed; the DID's coin and the fee's coins are kept out of other spends
    /// until settled.
    ///
    /// The DID is resolved live first and must be held at one of `keys`' addresses. An eve DID
    /// (a sponsored one never spent) is taken to have nil metadata, as a plain sponsored DID
    /// has; use [`Self::set_did_metadata_from`] for an eve world DID.
    pub fn set_did_metadata(
        &self,
        keys: &WalletKeys,
        launcher_id: Bytes32,
        metadata: &DidMetadata,
        fee: u64,
        timeout: Duration,
    ) -> Result<UpdatedDid> {
        self.set_did_metadata_from(keys, launcher_id, &DidMetadata::Empty, metadata, fee, timeout)
    }

    /// [`Self::set_did_metadata`] for a DID that may be eve with metadata `eve_metadata` (a
    /// sponsored world DID: the metadata it was launched with, which the chain doesn't show and
    /// its eve puzzle hash commits to). A DID already spent ignores `eve_metadata`: its metadata
    /// is read from the chain.
    pub fn set_did_metadata_from(
        &self,
        keys: &WalletKeys,
        launcher_id: Bytes32,
        eve_metadata: &DidMetadata,
        metadata: &DidMetadata,
        fee: u64,
        timeout: Duration,
    ) -> Result<UpdatedDid> {
        use chia_sdk_driver::SpendContext;
        let caps = self.caps();
        caps.check_fee(fee)?;
        caps.check("max-did-fee", "the DID update's fee", fee)?;
        self.check_keys(keys)?;
        let lookup = self.resolve_did(launcher_id, timeout)?;
        if !lookup.live {
            return Err(Error::new("no chain peer reachable"));
        }
        let record = lookup.record;
        let current = record
            .current()
            .filter(|c| c.spent.is_none() && !record.melted)
            .ok_or_else(|| Error::new("the DID has no unspent coin"))?
            .coin_id;
        let owners = keys.puzzle_hashes();
        if !owners.iter().any(|ph| record.owned_by(*ph)) && !record.is_eve() {
            return Err(Error::new("the DID isn't held at one of this wallet's addresses"));
        }
        let (source, coins) = {
            let s = self.state();
            if s.cache.pending.contains_key(&hex32(current)) {
                return Err(Error::new("a spend of this DID is already pending"));
            }
            (
                s.active_source()
                    .ok_or_else(|| Error::new("no chain peer reachable"))?,
                s.cache.spendable(),
            )
        };
        let chosen = if fee > 0 {
            offer::select(&coins, fee).map_err(|_| Error::new(format!("the wallet needs at least {fee} mojos for the DID update's fee")))?
        } else {
            Vec::new()
        };
        let (bundle, child) = self.run(timeout, async {
            let mut ctx = SpendContext::new();
            // Nil (a plain sponsored DID) first, then the eve metadata the caller knows.
            let candidates = [chia_sdk_driver::HashedPtr::NIL, eve_metadata.alloc(&mut ctx)?];
            let did = did::current_did(&source, &mut ctx, &record, &owners, &candidates).await?;
            super::world::build_metadata_update(&mut ctx, keys, did, metadata, &chosen, fee)
        })?;
        let push = self.run(timeout, async { source.push(bundle).await })?;
        if !matches!(push, PushOutcome::Rejected { .. }) {
            let now = offer::now_seconds();
            let mut s = self.state();
            s.cache.pending.insert(hex32(current), now);
            for c in &chosen {
                s.cache.pending.insert(hex32(c.coin_id()), now);
            }
            s.save();
        }
        Ok(UpdatedDid { launcher_id, spent: current, coin_id: child.coin.coin_id(), owner: child.info.p2_puzzle_hash, push })
    }

    /// Wait until a DID launched by `launcher_id` is on chain (its launcher spent, its singleton
    /// made), asking every `every` up to `timeout`: its record, read live. A sponsored DID is
    /// watched this way too (the sponsor answers with its launcher id before it confirms).
    pub fn wait_did(&self, launcher_id: Bytes32, every: Duration, timeout: Duration) -> Result<DidRecord> {
        let until = Instant::now() + timeout;
        loop {
            let left = until.saturating_duration_since(Instant::now());
            let last = match self.resolve_did(launcher_id, left.min(Duration::from_secs(20))) {
                Ok(found) if found.live => return Ok(found.record),
                Ok(_) => Error::new("no chain peer reachable"),
                Err(e) => e,
            };
            if Instant::now() + every >= until {
                return Err(last);
            }
            std::thread::sleep(every);
        }
    }

    /// The achievements' daily anchor (I10): one of the wallet's coins (the smallest above the
    /// fee) spent back to its primary address with `root` in the memos, pushed. The new coin is the
    /// anchor (its id returned); the spent coin is kept out of other spends until settled.
    pub fn anchor(&self, keys: &WalletKeys, root: [u8; 32], fee: u64, timeout: Duration) -> Result<(PushOutcome, Bytes32)> {
        self.caps()
            .check("max-anchor-fee", "the anchor's fee", fee)?;
        self.check_keys(keys)?;
        let (source, coins) = {
            let s = self.state();
            (
                s.active_source()
                    .ok_or_else(|| Error::new("no chain peer reachable"))?,
                s.cache.spendable(),
            )
        };
        let coin = coins
            .iter()
            .filter(|c| c.amount > fee)
            .min_by_key(|c| c.amount)
            .copied()
            .ok_or_else(|| Error::new(format!("no coin above {fee} mojos to anchor with")))?;
        let (coin_spends, signature, made) = super::anchor::build(keys, coin, root, fee)?;
        let push = self.run(timeout, async { source.push(SpendBundle::new(coin_spends, signature)).await })?;
        if !matches!(push, PushOutcome::Rejected { .. }) {
            let mut s = self.state();
            s.cache
                .pending
                .insert(hex32(coin.coin_id()), offer::now_seconds());
            s.save();
        }
        Ok((push, made.coin_id()))
    }

    /// The root an anchor coin carries on chain (none while it isn't there).
    pub fn anchor_root(&self, coin_id: Bytes32, timeout: Duration) -> Result<Option<[u8; 32]>> {
        self.read(timeout, move |source| async move { super::anchor::anchor_root(&source, coin_id).await })
    }

    /// List a world (phase 5, [`super::listing`]): a coin of `deposit` mojos at `owner` (one of
    /// `keys`' addresses: the one holding the world DID) hinted with the listing tag, naming
    /// `world_did`, made from the smallest spendable coin that covers it and `fee`. Pushed; the
    /// listing coin is reserved (no coin selection spends it) until [`Self::delist_world`].
    pub fn list_world(
        &self,
        keys: &WalletKeys,
        owner: Bytes32,
        world_did: Bytes32,
        deposit: u64,
        fee: u64,
        timeout: Duration,
    ) -> Result<(PushOutcome, chia_protocol::Coin)> {
        let caps = self.caps();
        caps.check_fee(fee)?;
        caps.check("max-send", "the listing's deposit", deposit)?;
        self.check_keys(keys)?;
        if !keys.puzzle_hashes().contains(&owner) {
            return Err(Error::new("the listing's address isn't one of this wallet's"));
        }
        let need = deposit
            .checked_add(fee)
            .ok_or_else(|| Error::new("the amounts overflow"))?;
        let (source, coins) = {
            let s = self.state();
            (
                s.active_source()
                    .ok_or_else(|| Error::new("no chain peer reachable"))?,
                s.cache.spendable(),
            )
        };
        let coin = coins
            .iter()
            .filter(|c| c.amount >= need)
            .min_by_key(|c| c.amount)
            .copied()
            .ok_or_else(|| Error::new(format!("no coin of at least {need} mojos to list the world with")))?;
        let (coin_spends, signature, listing) = super::listing::build(keys, coin, owner, world_did, deposit, fee)?;
        let push = self.run(timeout, async { source.push(SpendBundle::new(coin_spends, signature)).await })?;
        if !matches!(push, PushOutcome::Rejected { .. }) {
            let mut s = self.state();
            s.cache
                .pending
                .insert(hex32(coin.coin_id()), offer::now_seconds());
            s.cache.reserved.insert(hex32(listing.coin_id()));
            s.save();
        }
        Ok((push, listing))
    }

    /// Delist: spend the listing coin back to the primary address, less `fee`. Pushed.
    pub fn delist_world(&self, keys: &WalletKeys, listing: chia_protocol::Coin, fee: u64, timeout: Duration) -> Result<PushOutcome> {
        self.caps().check_fee(fee)?;
        self.check_keys(keys)?;
        let source = self
            .state()
            .active_source()
            .ok_or_else(|| Error::new("no chain peer reachable"))?;
        let (coin_spends, signature) = super::listing::build_delist(keys, listing, fee)?;
        let push = self.run(timeout, async { source.push(SpendBundle::new(coin_spends, signature)).await })?;
        if !matches!(push, PushOutcome::Rejected { .. }) {
            let mut s = self.state();
            let id = hex32(listing.coin_id());
            s.cache.reserved.remove(&id);
            s.cache.pending.insert(id, offer::now_seconds());
            s.save();
        }
        Ok(push)
    }

    /// Keep coin `coin_id` out of coin selection (a listing made before this cache).
    pub fn reserve(&self, coin_id: Bytes32) {
        let mut s = self.state();
        if s.cache.reserved.insert(hex32(coin_id)) {
            s.save();
        }
    }

    /// The listings on chain holding at least `min_amount`, the newest `max` (phase 5).
    pub fn listings(&self, min_amount: u64, max: usize, timeout: Duration) -> Result<Vec<super::listing::Listing>> {
        self.read(timeout, move |source| async move { super::listing::listings(&source, min_amount, max).await })
    }

    /// Spendable XCH of the wallet, in mojos (unspent coins no offer or pending spend holds).
    pub fn spendable_balance(&self) -> u64 {
        self.state()
            .cache
            .spendable()
            .iter()
            .map(|c| c.amount)
            .sum()
    }

    /// Spendable coins of exactly `amount` mojos (a split's pool).
    pub fn coins_of(&self, amount: u64) -> usize {
        self.state()
            .cache
            .spendable()
            .iter()
            .filter(|c| c.amount == amount)
            .count()
    }

    /// Push a spend bundle through a live peer. Not on mainnet: a bundle built elsewhere passed
    /// no cap, so only the reader's own spends (each checked) go out there.
    pub fn push(&self, bundle: SpendBundle, timeout: Duration) -> Result<PushOutcome> {
        if self.network() == ChainNetwork::Mainnet {
            return Err(Error::new(
                "refused: raw pushes are for testnet11 and the simulator; on mainnet only the reader's own capped spends go out",
            ));
        }
        let source = self
            .state()
            .any_source()
            .ok_or_else(|| Error::new("no chain peer reachable"))?;
        self.run(timeout, async { source.push(bundle).await })
    }

    fn check_keys(&self, keys: &WalletKeys) -> Result<()> {
        let s = self.state();
        if keys.network() != s.config.network {
            return Err(Error::new("the wallet keys are for another network"));
        }
        if keys
            .puzzle_hashes()
            .iter()
            .any(|ph| !s.cache.wallet.is_ours(ph))
        {
            return Err(Error::new("the reader doesn't follow this wallet's addresses"));
        }
        if s.phase != Phase::Synced {
            return Err(Error::new("the wallet isn't synced with a live peer yet"));
        }
        Ok(())
    }

    /// Check and take an offer with `keys` (the wallet the reader follows), paying `fee`, and
    /// push it. The coins it spends are kept out of other spends until the chain settles them.
    pub fn take_offer(&self, text: &str, keys: &WalletKeys, fee: u64, timeout: Duration) -> Result<TakeOutcome> {
        let summary = offer::summarize(text)?;
        let caps = self.caps();
        caps.check_fee(fee)?;
        caps.check("max-take", "taking the offer (its price and royalties)", summary.requested_xch.saturating_add(summary.royalties_xch))?;
        self.check_keys(keys)?;
        let (source, coins, peak) = {
            let s = self.state();
            (
                s.active_source()
                    .ok_or_else(|| Error::new("no chain peer reachable"))?,
                s.cache.spendable(),
                s.cache.peak,
            )
        };
        self.run(timeout, offer::check(&source, &summary, offer::now_seconds(), peak))?;
        let taken = offer::build_take(text, keys, &coins, fee)?;
        let push = self.run(timeout, async { source.push(taken.bundle).await })?;
        if !matches!(push, PushOutcome::Rejected { .. }) {
            let now = offer::now_seconds();
            let mut s = self.state();
            for id in &taken.spent {
                s.cache.pending.insert(hex32(*id), now);
            }
            s.save();
        }
        Ok(TakeOutcome { summary: taken.summary, push })
    }

    /// Make an offer with `keys`: give XCH or an NFT the wallet holds, request a price. Its coins
    /// are watched and kept out of other spends until it is taken or expires.
    pub fn make_offer(&self, spec: MakeOffer, keys: &WalletKeys, timeout: Duration) -> Result<MadeOffer> {
        let caps = self.caps();
        caps.check_fee(spec.fee)?;
        caps.check("max-offer-price", "the offer's price", spec.price)?;
        if let OfferGive::Xch(n) = spec.give {
            caps.check("max-offer-price", "what the offer gives", n)?;
        }
        self.check_keys(keys)?;
        let (source, coins, hinted) = {
            let s = self.state();
            (
                s.active_source()
                    .ok_or_else(|| Error::new("no chain peer reachable"))?,
                s.cache.spendable(),
                s.cache.wallet.unspent_hinted(),
            )
        };
        let locked = self.state().cache.locked();
        // Giving a few mojos with no fee: a coin of exactly that amount (the split pool) when
        // there is one, so the offer locks that coin and not the wallet's balance.
        let coins = match spec.give {
            OfferGive::Xch(n) if spec.fee == 0 => {
                match coins
                    .iter()
                    .filter(|c| c.amount == n)
                    .min_by_key(|c| c.coin_id())
                {
                    Some(exact) => vec![*exact],
                    None => coins,
                }
            }
            _ => coins,
        };
        let made = self.run(timeout, async {
            let mut ctx = chia_sdk_driver::SpendContext::new();
            let mut nft = None;
            if let OfferGive::Nft(launcher) = spec.give {
                for c in hinted.iter().filter(|c| !locked.contains(&c.id())) {
                    let Some(created) = c.created else { continue };
                    if let Some(n) = offer::fetch_nft(&source, &mut ctx, c.coin(), created).await?
                        && n.info.launcher_id == launcher
                    {
                        nft = Some(n);
                        break;
                    }
                }
            }
            offer::build_offer(&mut ctx, spec, keys, &coins, nft)
        })?;
        {
            let mut s = self.state();
            s.cache.offers.push(made.clone());
            s.dirty = true;
        }
        for id in &made.offered_coins {
            self.watch_coin(super::parse_hex32(id)?, timeout)?;
        }
        Ok(made)
    }

    /// What became of an offer this reader made. A settled or expired offer leaves the open list
    /// (its coins become spendable again if it expired untaken).
    pub fn offer_state(&self, made: &MadeOffer) -> OfferState {
        let mut s = self.state();
        let spent = made
            .offered_coins
            .iter()
            .filter_map(|h| s.cache.watched.get(h))
            .filter_map(WatchedCoin::spent)
            .min();
        let state = match spent {
            Some(height) => {
                let payment = super::parse_hex32(&made.payment_puzzle_hash).ok();
                let paid = s
                    .cache
                    .wallet
                    .coins
                    .values()
                    .any(|c| Some(c.puzzle_hash) == payment && c.amount == made.price && c.created == Some(height));
                OfferState::Taken { height, paid }
            }
            None if made.expires_at.is_some_and(|t| offer::now_seconds() >= t) => OfferState::Expired,
            None => OfferState::Open,
        };
        if matches!(state, OfferState::Expired | OfferState::Taken { paid: true, .. }) {
            s.cache.offers.retain(|o| o.text != made.text);
            s.dirty = true;
        }
        state
    }

    /// Stop the reader and save the cache.
    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        {
            let mut s = self.state();
            s.stop = true;
            s.phase = Phase::Stopped;
            s.save();
        }
        self.shared.wake.notify_waiters();
        if let Some(rt) = self.runtime.take() {
            rt.shutdown_timeout(Duration::from_secs(2));
        }
    }
}

impl Drop for ChainReader {
    fn drop(&mut self) {
        if self.runtime.is_some() {
            self.shutdown();
        }
    }
}

fn lock(shared: &Shared) -> std::sync::MutexGuard<'_, State> {
    shared.state.lock().unwrap_or_else(|e| e.into_inner())
}

/// A peer's pushed messages, as events.
async fn pump(id: u64, mut rx: mpsc::Receiver<Message>, events: mpsc::Sender<Event>) {
    loop {
        // Full nodes push a peak about every block (~18 s); two minutes of silence is a dead peer.
        let message = match tokio::time::timeout(Duration::from_secs(120), rx.recv()).await {
            Ok(Some(m)) => m,
            Ok(None) => {
                let _ = events
                    .send(Event::Closed(id, "connection closed".into()))
                    .await;
                return;
            }
            Err(_) => {
                let _ = events
                    .send(Event::Closed(id, "silent for 2 minutes".into()))
                    .await;
                return;
            }
        };
        let event = match message.msg_type {
            ProtocolMessageTypes::NewPeakWallet => NewPeakWallet::from_bytes(&message.data)
                .ok()
                .map(|p| Event::Peak(id, p)),
            ProtocolMessageTypes::CoinStateUpdate => CoinStateUpdate::from_bytes(&message.data)
                .ok()
                .map(|u| Event::Update(id, u)),
            _ => None,
        };
        if let Some(e) = event
            && events.send(e).await.is_err()
        {
            return;
        }
    }
}

/// Dial until the pool reaches its target (waves of 4, failures banned for 10 minutes).
async fn fill_pool(shared: &Arc<Shared>, dialer: &Arc<dyn Dialer>, events: &mpsc::Sender<Event>, next_id: &mut u64) {
    let (target, known) = {
        let s = lock(shared);
        (s.config.target_peers.max(1), s.cache.peers.clone())
    };
    if lock(shared).pool.len() >= target {
        return;
    }
    let mut candidates = known;
    for c in dialer.candidates().await {
        if !candidates.contains(&c) {
            candidates.push(c);
        }
    }
    {
        let mut s = lock(shared);
        let now = Instant::now();
        s.bans.retain(|_, until| *until > now);
        candidates.retain(|c| !s.bans.contains_key(c) && !s.pool.iter().any(|p| &p.address == c));
    }
    for wave in candidates.chunks(4).take(6) {
        let done = {
            let s = lock(shared);
            s.pool.len() >= target || s.stop
        };
        if done {
            break;
        }
        let dials = wave.iter().map(|addr| {
            let dialer = dialer.clone();
            let addr = addr.clone();
            async move {
                let r = tokio::time::timeout(Duration::from_secs(10), dialer.dial(addr.clone()))
                    .await
                    .unwrap_or_else(|_| Err(Error::new("timed out")));
                (addr, r)
            }
        });
        for (addr, result) in futures_util::future::join_all(dials).await {
            let mut s = lock(shared);
            match result {
                Ok((peer, rx)) if s.pool.len() < target => {
                    let id = *next_id;
                    *next_id += 1;
                    log::info!("chain: connected to {addr}");
                    tokio::spawn(pump(id, rx, events.clone()));
                    s.pool
                        .push(PoolPeer { id, address: addr.clone(), peer, peak: None });
                    if !s.cache.peers.contains(&addr) {
                        s.cache.peers.insert(0, addr);
                        s.cache.peers.truncate(16);
                    }
                }
                Ok((peer, _)) => {
                    tokio::spawn(async move {
                        let _ = peer.close().await;
                    });
                }
                Err(e) => {
                    log::debug!("chain: {addr}: {e}");
                    s.cache.peers.retain(|p| p != &addr);
                    s.bans
                        .insert(addr, Instant::now() + Duration::from_secs(600));
                }
            }
        }
    }
}

fn drop_peer(s: &mut State, id: u64, why: &str, ban: Option<Duration>) {
    if let Some(i) = s.pool.iter().position(|p| p.id == id) {
        let p = s.pool.remove(i);
        log::info!("chain: dropped {} ({why})", p.address);
        if let Some(ban) = ban {
            s.bans.insert(p.address.clone(), Instant::now() + ban);
        }
        tokio::spawn(async move {
            let _ = p.peer.close().await;
        });
    }
    if s.active == Some(id) {
        s.active = None;
        s.cache.wallet.subscribed.clear();
        if s.phase != Phase::Stopped {
            s.phase = Phase::Connecting;
        }
    }
}

/// Make the best peer carry the subscription: catch the wallet up, then subscribe the watched
/// coins and the cached DIDs' current coins on it.
async fn subscribe_best(shared: &Arc<Shared>) -> Result<()> {
    let (id, source, mut wallet, watch) = {
        let mut s = lock(shared);
        let best = s
            .pool
            .iter()
            .max_by_key(|p| p.peak.map(|(h, _)| h))
            .ok_or_else(|| Error::new("no peers"))?;
        let id = best.id;
        let source = PeerSource::new(best.peer.clone(), s.config.network);
        s.phase = Phase::Syncing;
        let mut watch: Vec<Bytes32> = s.cache.watched.values().map(|w| w.coin_id).collect();
        watch.extend(
            s.cache
                .dids
                .values()
                .filter_map(|d| d.current().filter(|c| c.spent.is_none()).map(|c| c.coin_id)),
        );
        let mut wallet = s.cache.wallet.clone();
        wallet.subscribed.clear();
        (id, source, wallet, watch)
    };
    let result = tokio::time::timeout(Duration::from_secs(120), async {
        wallet.catch_up(&source).await?;
        let mut states = Vec::new();
        for chunk in watch.chunks(1000) {
            states.extend(source.coin_state(chunk.to_vec(), true).await?);
        }
        Ok::<_, Error>(states)
    })
    .await
    .unwrap_or_else(|_| Err(Error::new("sync timed out")));
    let mut s = lock(shared);
    match result {
        Ok(states) => {
            if !s.pool.iter().any(|p| p.id == id) {
                return Err(Error::new("the peer left during the sync"));
            }
            s.cache.wallet = wallet;
            for st in &states {
                if let Some(w) = s.cache.watched.get_mut(&hex32(st.coin.coin_id())) {
                    w.state = Some(CoinRecord::from(st));
                }
            }
            if let Some(h) = s.cache.wallet.height {
                s.cache.peak = Some(s.cache.peak.map_or(h, |p| p.max(h)));
            }
            s.active = Some(id);
            s.phase = Phase::Synced;
            s.last_error = None;
            log::info!("chain: synced to height {} ({} coins)", s.cache.wallet.height.unwrap_or(0), s.cache.wallet.coins.len());
            s.save();
            Ok(())
        }
        Err(e) => {
            drop_peer(&mut s, id, "sync failed", Some(Duration::from_secs(600)));
            Err(e)
        }
    }
}

/// Apply a pushed update from the active peer: wallet coins, watched coins, DIDs.
async fn apply_update(shared: &Arc<Shared>, id: u64, update: CoinStateUpdate) {
    let (source, hinted) = {
        let mut s = lock(shared);
        if s.active != Some(id) {
            return;
        }
        // The peer pushes coins of our puzzle hashes, coins hinted to them (CATs, NFTs, DIDs
        // received) and the coins subscribed by id. Watched coins and DIDs' coins that are
        // nothing of the wallet's stay out of its coins.
        let tracked = |id: &Bytes32| {
            s.cache.watched.contains_key(&hex32(*id))
                || s.cache
                    .dids
                    .values()
                    .any(|d| d.current().is_some_and(|c| c.coin_id == *id))
        };
        let ours: Vec<_> = update
            .items
            .iter()
            .filter(|st| {
                let cid = st.coin.coin_id();
                !tracked(&cid)
                    || s.cache.wallet.is_ours(&st.coin.puzzle_hash)
                    || s.cache.wallet.coins.contains_key(&cid)
                    || s.cache.wallet.subscribed.contains(&cid)
            })
            .copied()
            .collect();
        let wallet_update = CoinStateUpdate::new(update.height, update.fork_height, update.peak_hash, ours);
        let hinted = s.cache.wallet.apply_update(&wallet_update);
        for st in &update.items {
            let key = hex32(st.coin.coin_id());
            if let Some(w) = s.cache.watched.get_mut(&key) {
                w.state = Some(CoinRecord::from(st));
            }
            // A cached DID's current coin was spent: the record is stale until resolved again.
            for d in s.cache.dids.values_mut() {
                if let Some(last) = d.history.last_mut()
                    && last.coin_id == st.coin.coin_id()
                {
                    last.spent = st.spent_height;
                }
            }
        }
        s.cache.peak = Some(s.cache.peak.map_or(update.height, |p| p.max(update.height)));
        s.cache.expire_pending(offer::now_seconds(), 600);
        s.dirty = true;
        (s.active_source(), hinted)
    };
    if !hinted.is_empty()
        && let Some(source) = source
    {
        let mut wallet = lock(shared).cache.wallet.clone();
        match wallet.subscribe_hinted(&source, hinted).await {
            Ok(()) => lock(shared).cache.wallet = wallet,
            Err(e) => log::debug!("chain: hinted coins: {e}"),
        }
    }
}

/// The reader's main loop.
async fn supervise(shared: Arc<Shared>, dialer: Arc<dyn Dialer>) {
    let (events_tx, mut events) = mpsc::channel::<Event>(256);
    let mut next_id = 0u64;
    let mut failures = 0u32;
    let mut last_save = Instant::now();
    let mut last_fill = Instant::now() - Duration::from_secs(3600);
    loop {
        if lock(&shared).stop {
            return;
        }
        let want_more = {
            let s = lock(&shared);
            s.pool.len() < s.config.target_peers.max(1)
        };
        if want_more && last_fill.elapsed() > Duration::from_secs(if failures == 0 { 5 } else { 15 }) {
            last_fill = Instant::now();
            fill_pool(&shared, &dialer, &events_tx, &mut next_id).await;
        }
        let (empty, needs_active) = {
            let mut s = lock(&shared);
            let empty = s.pool.is_empty();
            if empty {
                s.phase = Phase::Offline;
            }
            (empty, s.active.is_none() && !empty)
        };
        if empty {
            failures += 1;
            let wait = Duration::from_secs(u64::from(failures.min(15)) * 2);
            lock(&shared).last_error = Some("no chain peer reachable".into());
            tokio::select! {
                _ = tokio::time::sleep(wait) => {}
                _ = shared.wake.notified() => {}
            }
            continue;
        }
        if needs_active {
            // Let fresh peers report a first peak, so the choice is informed.
            let until = Instant::now() + Duration::from_millis(1500);
            while Instant::now() < until && lock(&shared).pool.iter().all(|p| p.peak.is_none()) {
                match tokio::time::timeout(Duration::from_millis(100), events.recv()).await {
                    Ok(Some(e)) => handle(&shared, e).await,
                    Ok(None) => return,
                    Err(_) => {}
                }
            }
            match subscribe_best(&shared).await {
                Ok(()) => failures = 0,
                Err(e) => {
                    failures += 1;
                    log::warn!("chain: sync failed: {e}");
                    lock(&shared).last_error = Some(e.to_string());
                    continue;
                }
            }
        }
        tokio::select! {
            e = events.recv() => match e {
                Some(e) => handle(&shared, e).await,
                None => return,
            },
            _ = tokio::time::sleep(Duration::from_secs(5)) => {}
            _ = shared.wake.notified() => {}
        }
        // The subscribed peer lagging the pool by 3 blocks is replaced.
        {
            let mut s = lock(&shared);
            let max = s.pool.iter().filter_map(|p| p.peak.map(|(h, _)| h)).max();
            if let (Some(max), Some(active)) = (max, s.active) {
                let at = s
                    .pool
                    .iter()
                    .find(|p| p.id == active)
                    .and_then(|p| p.peak.map(|(h, _)| h))
                    .unwrap_or(0);
                if max.saturating_sub(at) >= 3 {
                    drop_peer(&mut s, active, "lagging the pool", None);
                }
            }
            if s.dirty && last_save.elapsed() > Duration::from_secs(5) {
                s.save();
                last_save = Instant::now();
            }
        }
    }
}

async fn handle(shared: &Arc<Shared>, event: Event) {
    match event {
        Event::Peak(id, peak) => {
            let mut s = lock(shared);
            if let Some(p) = s.pool.iter_mut().find(|p| p.id == id) {
                p.peak = Some((peak.height, peak.header_hash));
            }
            s.cache.peak = Some(s.cache.peak.map_or(peak.height, |h| h.max(peak.height)));
            if s.active == Some(id) {
                if s.cache
                    .wallet
                    .height
                    .is_some_and(|h| peak.fork_point_with_previous_peak < h)
                {
                    // A reorg below what we synced: the update that follows rolls the coins back.
                    s.dirty = true;
                }
                s.cache.wallet.apply_peak(peak.height, peak.header_hash);
                s.dirty = true;
            }
        }
        Event::Update(id, update) => apply_update(shared, id, update).await,
        Event::Closed(id, why) => {
            let mut s = lock(shared);
            drop_peer(&mut s, id, &why, Some(Duration::from_secs(300)));
        }
    }
}
