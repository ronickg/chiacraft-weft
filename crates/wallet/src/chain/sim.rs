//! A local chain for tests and checks (feature `sim`): chia-sdk-test's peer simulator, a full
//! node's wallet protocol in memory, reachable as [`SimDialer`] in-process or as
//! `ws://127.0.0.1:<port>` (the reader dials plain websockets on loopback only). Never a real
//! network, never real funds: every key that uses it is a throwaway.
//!
//! Helpers make the assets admission checks need: coins at an address, a DID, an NFT, a CAT.
use super::{
    Bytes32, Coin, PushOutcome, SpendBundle, WalletKeys,
    reader::{BoxFuture, Connection, Dialer},
};
use crate::{Error, Result};
use chia_sdk_driver::{Action, HashedPtr, Id, Relation, SpendContext, Spends};
use chia_sdk_test::PeerSimulator;
use std::sync::Arc;

/// The simulator and its runtime.
pub struct Sim {
    pub rt: tokio::runtime::Runtime,
    pub sim: Arc<PeerSimulator>,
}

/// Dials the simulator in-process (any address).
pub struct SimDialer(pub Arc<PeerSimulator>);
impl Dialer for SimDialer {
    fn candidates(&self) -> BoxFuture<'_, Vec<String>> {
        Box::pin(async { vec!["simulator:58444".to_owned()] })
    }
    fn dial(&self, _address: String) -> BoxFuture<'_, Result<Connection>> {
        let sim = self.0.clone();
        Box::pin(async move {
            sim.connect_raw()
                .await
                .map_err(|e| Error::new(e.to_string()))
        })
    }
}

impl Sim {
    pub fn start() -> Result<Self> {
        Self::start_on(super::ChainNetwork::Testnet11)
    }

    /// A simulator whose sync starts from `network`'s genesis challenge, so a reader configured
    /// for that network follows it (the mainnet gates' tests, I12). Its spend checks stay
    /// testnet11's (chia-sdk-test validates with testnet11's constants): a spend signed for
    /// mainnet is pushed and then rejected here, which shows it committed to mainnet's data.
    pub fn start_on(network: super::ChainNetwork) -> Result<Self> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|e| Error::new(e.to_string()))?;
        let config = chia_sdk_test::PeerSimulatorConfig { genesis_challenge: network.genesis_challenge(), ..Default::default() };
        let sim = rt
            .block_on(PeerSimulator::with_config(config))
            .map_err(|e| Error::new(e.to_string()))?;
        Ok(Self { rt, sim: Arc::new(sim) })
    }

    /// `ws://127.0.0.1:<port>`: where a reader in another process reaches it. The simulator keeps
    /// its address private; its `Debug` names it.
    pub fn address(&self) -> Option<String> {
        let shown = format!("{:?}", self.sim);
        let at = shown.find("addr: ")? + 6;
        let rest = &shown[at..];
        let end = rest.find([',', ' ', '}']).unwrap_or(rest.len());
        Some(format!("ws://{}", &rest[..end]))
    }

    pub fn dialer(&self) -> SimDialer {
        SimDialer(self.sim.clone())
    }

    /// Pay `amount` mojos to `puzzle_hash` in a transaction (a block, which readers already
    /// subscribed are told about; [`Sim::fund`] only adds a coin, which a running reader doesn't
    /// see until it syncs again). Paid from a throwaway faucet key funded on the spot.
    pub fn pay(&self, puzzle_hash: Bytes32, amount: u64) -> Result<Coin> {
        let faucet = WalletKeys::from_mnemonic(&crate::Mnemonic::generate()?, super::ChainNetwork::Testnet11, 1);
        self.fund(faucet.primary(), amount);
        let out = self.spend(&faucet, &[Action::send(Id::Xch, puzzle_hash, amount, chia_puzzle_types::Memos::None)])?;
        out.xch
            .iter()
            .find(|c| c.puzzle_hash == puzzle_hash)
            .copied()
            .ok_or_else(|| Error::new("the payment made no coin"))
    }

    /// A new coin of `amount` mojos at `puzzle_hash`.
    pub fn fund(&self, puzzle_hash: Bytes32, amount: u64) -> Coin {
        self.rt
            .block_on(async { self.sim.lock().await.new_coin(puzzle_hash, amount) })
    }

    /// Unspent coins at a puzzle hash (hinted ones too).
    pub fn unspent(&self, puzzle_hash: Bytes32) -> Vec<Coin> {
        self.rt
            .block_on(async { self.sim.lock().await.unspent_coins(puzzle_hash, false) })
    }

    pub fn balance(&self, puzzle_hash: Bytes32) -> u64 {
        self.unspent(puzzle_hash).iter().map(|c| c.amount).sum()
    }

    /// The next block's time (offers expire by it).
    pub fn set_time(&self, unix_seconds: u64) -> Result<()> {
        self.rt.block_on(async {
            self.sim
                .lock()
                .await
                .set_next_timestamp(unix_seconds)
                .map_err(|e| Error::new(e.to_string()))
        })
    }

    /// Push a bundle straight to the simulator.
    pub fn push(&self, bundle: SpendBundle) -> Result<PushOutcome> {
        self.rt.block_on(async {
            let peer = self
                .sim
                .connect()
                .await
                .map_err(|e| Error::new(e.to_string()))?;
            let ack = peer
                .send_transaction(bundle)
                .await
                .map_err(|e| Error::new(e.to_string()))?;
            Ok(match ack.status {
                1 => PushOutcome::Accepted { txid: ack.txid },
                2 => PushOutcome::Pending { txid: ack.txid },
                _ => PushOutcome::Rejected { txid: ack.txid, error: ack.error.unwrap_or_default() },
            })
        })
    }

    /// Spend `keys`' largest coin at its primary address through `actions`; the outputs.
    fn spend(&self, keys: &WalletKeys, actions: &[Action]) -> Result<chia_sdk_driver::Outputs> {
        let coin = self
            .unspent(keys.primary())
            .into_iter()
            .max_by_key(|c| c.amount)
            .ok_or_else(|| Error::new("no coin to spend: fund the address first"))?;
        let mut ctx = SpendContext::new();
        let mut spends = Spends::new(keys.primary());
        spends.add(coin);
        let deltas = spends
            .apply(&mut ctx, actions)
            .map_err(|e| Error::new(e.to_string()))?;
        let outputs = spends
            .finish_with_keys(&mut ctx, &deltas, Relation::AssertConcurrent, &keys.synthetic_keys())
            .map_err(|e| Error::new(e.to_string()))?;
        let coin_spends = ctx.take();
        let signature = keys.sign(&coin_spends)?;
        // The simulator records a transaction's coins at its current height, which a reader that
        // last saw that height as its peak counts as seen: an empty block first keeps a reader
        // resuming from its cache in step (a real chain's blocks don't have this off-by-one).
        self.rt
            .block_on(async { self.sim.lock().await.create_block() });
        match self.push(SpendBundle::new(coin_spends, signature))? {
            PushOutcome::Rejected { error, .. } => Err(Error::new(error)),
            _ => Ok(outputs),
        }
    }

    /// A DID owned by `keys`' primary address (one mojo of its largest coin).
    pub fn create_did(&self, keys: &WalletKeys) -> Result<Bytes32> {
        let out = self.spend(keys, &[Action::create_empty_did()])?;
        Ok(out.dids[&Id::New(0)].info.launcher_id)
    }

    /// What a DID sponsor does (identity row I8's tests; I9 runs the real one): launch a plain
    /// eve DID straight to `p2_puzzle_hash` (`Launcher::create_eve_did`), paid by a throwaway
    /// faucet key, so the owner signs nothing and the sponsor never holds it. Its launcher id.
    pub fn sponsor_did(&self, p2_puzzle_hash: Bytes32) -> Result<Bytes32> {
        use chia_sdk_driver::{Launcher, StandardLayer};
        let faucet = WalletKeys::from_mnemonic(&crate::Mnemonic::generate()?, super::ChainNetwork::Testnet11, 1);
        let coin = self.fund(faucet.primary(), 1_000);
        let mut ctx = SpendContext::new();
        let launcher = Launcher::new(coin.coin_id(), 1);
        let launcher_id = launcher.coin().coin_id();
        let (conditions, _did) = launcher
            .create_eve_did(&mut ctx, p2_puzzle_hash, None, 1, HashedPtr::NIL)
            .map_err(|e| Error::new(e.to_string()))?;
        let conditions = conditions.create_coin(faucet.primary(), coin.amount - 1, chia_puzzle_types::Memos::None);
        let key = *faucet
            .synthetic_keys()
            .get(&faucet.primary())
            .ok_or_else(|| Error::new("no faucet key"))?;
        StandardLayer::new(key)
            .spend(&mut ctx, coin, conditions)
            .map_err(|e| Error::new(e.to_string()))?;
        let coin_spends = ctx.take();
        let signature = faucet.sign(&coin_spends)?;
        self.rt
            .block_on(async { self.sim.lock().await.create_block() });
        match self.push(SpendBundle::new(coin_spends, signature))? {
            PushOutcome::Rejected { error, .. } => Err(Error::new(error)),
            _ => Ok(launcher_id),
        }
    }

    /// What the DID sponsor does for world DIDs (plan phase 4, slice 7): launch DIDs with
    /// [`super::sponsor::build_eve_dids_with`] (world metadata, gifts of mojos to the owners),
    /// paid by a throwaway sponsor key funded on the spot, and push them in a block. The bundle.
    pub fn sponsor_dids(&self, asks: &[super::EveAsk], fee: u64) -> Result<super::SponsorBundle> {
        let sponsor = WalletKeys::from_mnemonic(&crate::Mnemonic::generate()?, super::ChainNetwork::Testnet11, 1);
        let gifts: u64 = asks.iter().map(|a| a.gift).sum();
        self.fund(sponsor.primary(), asks.len() as u64 + gifts + fee + 1_000);
        let built = super::sponsor::build_eve_dids_with(&sponsor, &self.unspent(sponsor.primary()), asks, fee)?;
        self.rt
            .block_on(async { self.sim.lock().await.create_block() });
        match self.push(built.bundle.clone())? {
            PushOutcome::Rejected { error, .. } => Err(Error::new(error)),
            _ => Ok(built),
        }
    }

    /// An NFT (no royalty, no DID) held by `keys`' primary address.
    pub fn mint_nft(&self, keys: &WalletKeys) -> Result<Bytes32> {
        let out = self.spend(keys, &[Action::mint_nft(HashedPtr::NIL, Bytes32::default(), keys.primary(), 0, 1)])?;
        Ok(out.nfts[&Id::New(0)].info.launcher_id)
    }

    /// A single-issuance CAT of `amount` mojos at `keys`' primary address: its asset id.
    pub fn issue_cat(&self, keys: &WalletKeys, amount: u64) -> Result<Bytes32> {
        let out = self.spend(keys, &[Action::single_issue_cat(None, amount)])?;
        out.cats
            .get(&Id::New(0))
            .and_then(|c| c.first())
            .map(|c| c.info.asset_id)
            .ok_or_else(|| Error::new("no CAT issued"))
    }
}
