//! The chain reader against chia-sdk-test's peer simulator (a full node's wallet protocol over a
//! local websocket, in memory): wallet sync and pushed updates, coin watches, the cache and
//! offline answers, DIDs and their history, offers made and taken (a 1-mojo coin and a royalty
//! NFT for XCH), expiry, and spent offers refused. Every key is a fresh throwaway mnemonic; no
//! network, no real funds.
#![cfg(feature = "chain")]

use chia_protocol::{Bytes32, Coin, CoinSpend, SpendBundle};
use chia_puzzle_types::Memos;
use chia_sdk_driver::{Action, HashedPtr, Id, Relation, SpendContext, Spends};
use chia_sdk_test::{PeerSimulator, to_program, to_puzzle};
use chiacraft_wallet::{
    Error, Mnemonic,
    chain::{
        ChainConfig, ChainNetwork, ChainReader, OfferGive, OfferState, Phase, PushOutcome, WalletKeys,
        did::{self, DidRecord},
        offer::{self, MakeOffer},
        reader::{BoxFuture, Connection, Dialer},
        source::PeerSource,
        wallet::standard_puzzle_hash,
    },
};
use std::{sync::Arc, time::Duration};

const WAIT: Duration = Duration::from_secs(20);

/// Dials the simulator (any address).
struct SimDialer(Arc<PeerSimulator>);
impl Dialer for SimDialer {
    fn candidates(&self) -> BoxFuture<'_, Vec<String>> {
        Box::pin(async { vec!["simulator:58444".to_owned()] })
    }
    fn dial(&self, _address: String) -> BoxFuture<'_, chiacraft_wallet::Result<Connection>> {
        let sim = self.0.clone();
        Box::pin(async move {
            sim.connect_raw()
                .await
                .map_err(|e| Error::new(e.to_string()))
        })
    }
}

/// Reaches nothing: the chain is down.
struct DeadDialer;
impl Dialer for DeadDialer {
    fn candidates(&self) -> BoxFuture<'_, Vec<String>> {
        Box::pin(async { vec!["192.0.2.1:58444".to_owned()] })
    }
    fn dial(&self, _address: String) -> BoxFuture<'_, chiacraft_wallet::Result<Connection>> {
        Box::pin(async { Err(Error::new("unreachable")) })
    }
}

struct World {
    rt: tokio::runtime::Runtime,
    sim: Arc<PeerSimulator>,
    dir: std::path::PathBuf,
}

impl World {
    fn new(name: &str) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        let sim = Arc::new(rt.block_on(PeerSimulator::new()).unwrap());
        let dir = std::env::temp_dir().join(format!("hz-chain-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self { rt, sim, dir }
    }
    fn fund(&self, puzzle_hash: Bytes32, amount: u64) -> Coin {
        self.rt
            .block_on(async { self.sim.lock().await.new_coin(puzzle_hash, amount) })
    }
    fn reader(&self, keys: &WalletKeys, cache: &str) -> ChainReader {
        let mut config = ChainConfig::new(ChainNetwork::Testnet11);
        config.target_peers = 1;
        config.puzzle_hashes = keys.puzzle_hashes();
        config.cache_file = Some(self.dir.join(cache));
        let reader = ChainReader::start_with(config, SimDialer(self.sim.clone())).unwrap();
        assert!(reader.wait_synced(WAIT), "synced: {:?}", reader.status());
        reader
    }
    /// Push a bundle straight to the simulator.
    fn push(&self, bundle: SpendBundle) -> PushOutcome {
        self.rt.block_on(async {
            let peer = self.sim.connect().await.unwrap();
            let ack = peer.send_transaction(bundle).await.unwrap();
            match ack.status {
                1 => PushOutcome::Accepted { txid: ack.txid },
                2 => PushOutcome::Pending { txid: ack.txid },
                _ => PushOutcome::Rejected { txid: ack.txid, error: ack.error.unwrap_or_default() },
            }
        })
    }
    fn source(&self) -> PeerSource {
        let peer = self.rt.block_on(self.sim.connect()).unwrap();
        PeerSource::new(peer, ChainNetwork::Testnet11)
    }
    fn balance(&self, puzzle_hash: Bytes32) -> u64 {
        self.rt.block_on(async {
            self.sim
                .lock()
                .await
                .unspent_coins(puzzle_hash, false)
                .iter()
                .map(|c| c.amount)
                .sum()
        })
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn throwaway(count: u32) -> WalletKeys {
    WalletKeys::from_mnemonic(&Mnemonic::generate().unwrap(), ChainNetwork::Testnet11, count)
}

/// Build, sign (with our own signer) and return a bundle from actions over `coins` of `keys`.
fn spend(
    ctx: &mut SpendContext,
    keys: &WalletKeys,
    add: impl FnOnce(&mut Spends),
    actions: &[Action],
) -> (chia_sdk_driver::Outputs, SpendBundle) {
    let mut spends = Spends::new(keys.primary());
    add(&mut spends);
    let deltas = spends.apply(ctx, actions).unwrap();
    let outputs = spends
        .finish_with_keys(ctx, &deltas, Relation::AssertConcurrent, &keys.synthetic_keys())
        .unwrap();
    let coin_spends = ctx.take();
    let signature = keys.sign(&coin_spends).unwrap();
    (outputs, SpendBundle::new(coin_spends, signature))
}

fn until(mut f: impl FnMut() -> bool) -> bool {
    let end = std::time::Instant::now() + WAIT;
    while std::time::Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    f()
}

#[test]
fn sync_watch_cache_and_offline_answers() {
    let w = World::new("sync");
    let keys = throwaway(3);
    let phs = keys.puzzle_hashes();
    w.fund(phs[0], 1_000);
    w.fund(phs[2], 500);
    let reader = w.reader(&keys, "cache.json");
    let status = reader.status();
    assert_eq!(status.phase, Phase::Synced);
    assert_eq!(status.balance, 1_500);
    assert_eq!(status.peers.len(), 1);
    assert!(status.primary_address.unwrap().starts_with("txch1"));

    // A coin that isn't ours, watched until spent: an anyone-can-spend puzzle (`1`).
    let (anyone, reveal) = to_puzzle(1u8).unwrap();
    let other = w.fund(anyone, 7);
    let watched = reader.watch_coin(other.coin_id(), WAIT).unwrap();
    assert_eq!(watched.spent(), None);
    assert!(watched.state.is_some());
    // Spending it pays 5 mojos to our second address: the subscription brings both.
    let pay = to_program(chia_sdk_types::Conditions::<clvmr::NodePtr>::new().create_coin(phs[1], 5, Memos::None)).unwrap();
    let outcome = w.push(SpendBundle::new(vec![CoinSpend::new(other, reveal, pay)], chia_bls::Signature::default()));
    assert!(matches!(outcome, PushOutcome::Accepted { .. }), "{outcome:?}");
    let spent_at = reader
        .wait_spent(other.coin_id(), WAIT)
        .expect("seen spent");
    assert!(until(|| reader.status().balance == 1_505), "{:?}", reader.status());
    assert!(
        reader
            .watched(other.coin_id())
            .unwrap()
            .confirmations(reader.status().peak)
            >= 1
    );
    assert!(spent_at >= other_created(&w, &other));

    // A DID resolved while online is answered from the cache once the chain is gone.
    let launcher = make_did(&w, &keys);
    let live = reader.resolve_did(launcher, WAIT).unwrap();
    assert!(live.live);
    assert!(live.record.owned_by(phs[0]));
    assert!(until(|| reader.status().balance == 1_504), "{:?}", reader.status());
    reader.stop();

    let mut config = ChainConfig::new(ChainNetwork::Testnet11);
    config.puzzle_hashes = keys.puzzle_hashes();
    config.cache_file = Some(w.dir.join("cache.json"));
    let offline = ChainReader::start_with(config, DeadDialer).unwrap();
    assert!(until(|| offline.status().phase == Phase::Offline));
    let status = offline.status();
    assert_eq!(status.balance, 1_505 - 1, "the cached coins (one mojo became the DID)");
    assert!(status.synced_height.is_some());
    let cached = offline
        .resolve_did(launcher, Duration::from_secs(1))
        .unwrap();
    assert!(!cached.live);
    assert_eq!(cached.record, live.record);
    assert!(
        offline
            .resolve_did(Bytes32::new([9; 32]), Duration::from_secs(1))
            .is_err()
    );
    assert!(offline.watched(other.coin_id()).unwrap().spent().is_some());
}

fn other_created(w: &World, coin: &Coin) -> u32 {
    w.rt.block_on(async { w.sim.lock().await.coin_state(coin.coin_id()) })
        .and_then(|s| s.created_height)
        .unwrap()
}

/// A DID made from `keys`' first coin, owned by its primary address.
fn make_did(w: &World, keys: &WalletKeys) -> Bytes32 {
    let coin =
        w.rt.block_on(async { w.sim.lock().await.unspent_coins(keys.primary(), false) })
            .into_iter()
            .max_by_key(|c| c.amount)
            .unwrap();
    let mut ctx = SpendContext::new();
    let (outputs, bundle) = spend(&mut ctx, keys, |s| s.add(coin), &[Action::create_empty_did()]);
    assert!(matches!(w.push(bundle), PushOutcome::Accepted { .. }));
    outputs.dids[&Id::New(0)].info.launcher_id
}

#[test]
fn dids_resolve_with_their_owner_and_history() {
    let w = World::new("did");
    let alice = throwaway(1);
    let bob = throwaway(1);
    let coin = w.fund(alice.primary(), 10);
    let mut ctx = SpendContext::new();
    let (outputs, bundle) = spend(&mut ctx, &alice, |s| s.add(coin), &[Action::create_empty_did()]);
    assert!(matches!(w.push(bundle), PushOutcome::Accepted { .. }));
    let did = outputs.dids[&Id::New(0)];
    let launcher = did.info.launcher_id;
    assert_eq!(did.info.p2_puzzle_hash, alice.primary());

    let source = w.source();
    let rec: DidRecord =
        w.rt.block_on(did::resolve(&source, launcher, None, 0))
            .unwrap();
    assert!(rec.owned_by(alice.primary()));
    assert!(!rec.melted);
    assert_eq!(rec.current().unwrap().coin_id, did.coin.coin_id());
    assert!(rec.did().starts_with("did:chia:1"));

    // Alice transfers it to Bob: the record follows, from the cached one.
    let hint = ctx.hint(bob.primary()).unwrap();
    let (_, bundle) = spend(&mut ctx, &alice, |s| s.add(did), &[Action::send(Id::Existing(launcher), bob.primary(), 1, hint)]);
    assert!(matches!(w.push(bundle), PushOutcome::Accepted { .. }));
    let after =
        w.rt.block_on(did::resolve(&source, launcher, Some(rec.clone()), 5))
            .unwrap();
    assert!(after.owned_by(bob.primary()), "{after:?}");
    assert!(!after.owned_by(alice.primary()));
    assert_eq!(after.owners(), vec![alice.primary(), bob.primary()]);
    assert_eq!(after.checked_height, 5);
    assert!(after.history.len() > rec.history.len());
    // From scratch gives the same.
    let fresh =
        w.rt.block_on(did::resolve(&source, launcher, None, 5))
            .unwrap();
    assert_eq!(fresh.owners(), after.owners());
    // The owner is the standard puzzle of a synthetic key (Sage's p2), here Bob's wallet key.
    assert_ne!(standard_puzzle_hash(&chia_bls::PublicKey::default()), bob.primary());

    // Not DIDs.
    let plain = w.fund(alice.primary(), 3);
    assert!(
        w.rt.block_on(did::resolve(&source, plain.coin_id(), None, 0))
            .is_err()
    );
    assert!(
        w.rt.block_on(did::resolve(&source, Bytes32::new([1; 32]), None, 0))
            .is_err()
    );
}

#[test]
fn offers_made_by_the_server_and_taken_by_a_player() {
    let w = World::new("offer");
    let server = throwaway(2);
    let player = throwaway(2);
    w.fund(server.primary(), 50);
    w.fund(player.primary(), 1_000);
    w.fund(player.puzzle_hashes()[1], 400);
    let server_reader = w.reader(&server, "server.json");
    let player_reader = w.reader(&player, "player.json");

    // Section 3.6, variant A: the server gives 1 mojo for 300, valid for an hour.
    let expires = offer::now_seconds() + 3_600;
    let made = server_reader
        .make_offer(MakeOffer { give: OfferGive::Xch(1), price: 300, expires_at: Some(expires), fee: 0 }, &server, WAIT)
        .unwrap();
    assert!(made.text.starts_with("offer1"));
    assert_eq!(server_reader.offer_state(&made), OfferState::Open);
    assert_eq!(server_reader.status().open_offers, 1);
    let summary = offer::summarize(&made.text).unwrap();
    assert_eq!(summary.offered_xch, 1);
    assert_eq!(summary.requested_xch, 300);
    assert_eq!(summary.expires_at_seconds, Some(expires));
    assert_eq!(summary.maker_coins, made.offered_coins);
    // Its coin is locked: a second offer finds no spendable XCH.
    assert!(
        server_reader
            .make_offer(MakeOffer { give: OfferGive::Xch(1), price: 5, expires_at: None, fee: 0 }, &server, WAIT)
            .is_err()
    );
    // A wallet the reader doesn't follow can't take it.
    assert!(
        player_reader
            .take_offer(&made.text, &server, 0, WAIT)
            .is_err()
    );

    let taken = player_reader
        .take_offer(&made.text, &player, 0, WAIT)
        .unwrap();
    assert!(matches!(taken.push, PushOutcome::Accepted { .. }), "{:?}", taken.push);
    let offered = chiacraft_wallet::chain::parse_hex32(&made.offered_coins[0]).unwrap();
    assert!(server_reader.wait_spent(offered, WAIT).is_some());
    assert!(
        until(|| matches!(server_reader.offer_state(&made), OfferState::Taken { paid: true, .. })),
        "{:?}",
        server_reader.offer_state(&made)
    );
    assert_eq!(server_reader.status().open_offers, 0);
    assert_eq!(w.balance(server.primary()), 50 - 1 + 300);
    assert!(until(|| player_reader.status().balance == 1_400 - 300 + 1));
    // Taken once: taking it again is refused before anything is pushed.
    let again = player_reader.take_offer(&made.text, &player, 0, WAIT);
    assert!(again.unwrap_err().to_string().contains("already taken"));

    // An expired offer is refused by the check, and by the chain if pushed anyway.
    let stale = server_reader
        .make_offer(MakeOffer { give: OfferGive::Xch(1), price: 10, expires_at: Some(100), fee: 0 }, &server, WAIT)
        .unwrap();
    let refused = player_reader.take_offer(&stale.text, &player, 0, WAIT);
    assert!(refused.unwrap_err().to_string().contains("expired"));
    w.rt.block_on(async { w.sim.lock().await.set_next_timestamp(1_000).unwrap() });
    let coins: Vec<Coin> = player_reader.cache().spendable();
    let bundle = offer::build_take(&stale.text, &player, &coins, 0)
        .unwrap()
        .bundle;
    assert!(matches!(w.push(bundle), PushOutcome::Rejected { .. }));
    assert_eq!(server_reader.offer_state(&stale), OfferState::Expired);
}

#[test]
fn an_nft_offer_with_a_royalty() {
    let w = World::new("nft");
    let server = throwaway(1);
    let player = throwaway(1);
    let artist = throwaway(1);
    let coin = w.fund(server.primary(), 10);
    w.fund(player.primary(), 2_000);
    // The server mints a membership NFT with a 3% royalty to the artist.
    let mut ctx = SpendContext::new();
    let (outputs, bundle) =
        spend(&mut ctx, &server, |s| s.add(coin), &[Action::mint_nft(HashedPtr::NIL, Bytes32::default(), artist.primary(), 300, 1)]);
    assert!(matches!(w.push(bundle), PushOutcome::Accepted { .. }));
    let launcher = outputs.nfts[&Id::New(0)].info.launcher_id;
    let server_reader = w.reader(&server, "server.json");
    let player_reader = w.reader(&player, "player.json");
    assert_eq!(server_reader.cache().wallet.unspent_hinted().len(), 1);

    let made = server_reader
        .make_offer(MakeOffer { give: OfferGive::Nft(launcher), price: 1_000, expires_at: None, fee: 0 }, &server, WAIT)
        .unwrap();
    let summary = offer::summarize(&made.text).unwrap();
    assert_eq!(summary.offered_nfts, vec![chiacraft_wallet::chain::hex32(launcher)]);
    assert_eq!(summary.requested_xch, 1_000);
    assert_eq!(summary.royalties_xch, 30);
    // Giving an NFT the wallet doesn't hold fails.
    assert!(
        server_reader
            .make_offer(MakeOffer { give: OfferGive::Nft(Bytes32::new([4; 32])), price: 1, expires_at: None, fee: 0 }, &server, WAIT)
            .is_err()
    );

    let taken = player_reader
        .take_offer(&made.text, &player, 0, WAIT)
        .unwrap();
    assert!(matches!(taken.push, PushOutcome::Accepted { .. }), "{:?}", taken.push);
    assert!(until(|| matches!(server_reader.offer_state(&made), OfferState::Taken { paid: true, .. })));
    assert_eq!(w.balance(artist.primary()), 30);
    assert_eq!(w.balance(server.primary()), 10 - 1 + 1_000);
    // The NFT now sits with the player (a hinted coin of the player's wallet).
    assert!(until(|| player_reader.cache().wallet.unspent_hinted().len() == 1));
}

#[test]
fn taking_refuses_what_it_cannot_pay() {
    // Not an offer at all.
    assert!(offer::summarize("offer1qqqq").is_err());
    assert!(offer::summarize("hello").is_err());
    // Coin selection.
    let c = |a| Coin::new(Bytes32::new([a as u8; 32]), Bytes32::default(), a);
    let picked = offer::select(&[c(5), c(20), c(7)], 21).unwrap();
    assert_eq!(picked.iter().map(|x| x.amount).sum::<u64>(), 27);
    assert!(offer::select(&[c(5)], 6).is_err());
}
