//! Admission's chain checks (identity row I7) against chia-sdk-test's peer simulator: a DID
//! proof by its owner key (Sage's synthetic key) checked on chain, an NFT and CAT coins found by
//! the holder's wallet and checked by a server, a coin that only looks like a CAT refused, the
//! offer pool split and offers using its coins, and the simulator's `ws://` address reached by a
//! reader over a real socket. Throwaway keys, no network.
#![cfg(feature = "sim")]

use chiacraft_wallet::{
    Mnemonic, OwnerSigner,
    chain::{
        Bytes32, ChainConfig, ChainNetwork, ChainReader, OfferGive, OfferState, Phase, PushOutcome,
        WalletKeys, holdings,
        offer::{self, MakeOffer},
        sim::Sim,
    },
    sign,
};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(20);

fn account() -> (Mnemonic, WalletKeys) {
    let m = Mnemonic::generate().unwrap();
    let k = WalletKeys::from_mnemonic(&m, ChainNetwork::Testnet11, 3);
    (m, k)
}

fn reader(sim: &Sim, keys: &WalletKeys) -> ChainReader {
    let mut config = ChainConfig::new(ChainNetwork::Testnet11);
    config.target_peers = 1;
    config.puzzle_hashes = keys.puzzle_hashes();
    let r = ChainReader::start_with(config, sim.dialer()).unwrap();
    assert!(r.wait_synced(WAIT), "{:?}", r.status());
    r
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
fn a_did_proof_is_checked_against_its_owner_on_chain() {
    let sim = Sim::start().unwrap();
    let (words, keys) = account();
    sim.fund(keys.primary(), 10);
    let launcher = sim.create_did(&keys).unwrap();
    let server = reader(&sim, &account().1);
    let record = server.resolve_did(launcher, WAIT).unwrap().record;
    // The player's wallet finds which address owns it, and that key signs the proof.
    let owner = OwnerSigner::find(&words, record.owner().unwrap().to_bytes(), 20).unwrap();
    assert_eq!((owner.index(), owner.hardened()), (0, false));
    let text = "chiacraft did proof v1\ndid: x\nsession: 01";
    let sig = owner.sign(text).unwrap();
    assert!(sign::verify_owner(&owner.public(), text.as_bytes(), &sig));
    assert!(holdings::did_owned_by_key(&record, &owner.public()));
    // Another account's key doesn't own it, even with a good signature of its own.
    let (other_words, _) = account();
    let other = OwnerSigner::from_mnemonic(&other_words, 0, false);
    assert!(!holdings::did_owned_by_key(&record, &other.public()));
    assert!(OwnerSigner::find(&other_words, record.owner().unwrap().to_bytes(), 5).is_none());
}

#[test]
fn nft_and_cat_holdings_found_by_the_holder_and_checked_by_the_server() {
    let sim = Sim::start().unwrap();
    let (_, keys) = account();
    sim.fund(keys.primary(), 5_000);
    let launcher = sim.mint_nft(&keys).unwrap();
    let asset = sim.issue_cat(&keys, 3_000).unwrap();
    let player = reader(&sim, &keys);
    let wallet = player.cache().wallet;
    // The holder's side: its NFT and its CAT coins, by owner address.
    let nfts = player
        .read(WAIT, |src| async move {
            holdings::own_nfts(&src, &wallet).await
        })
        .unwrap();
    assert_eq!(nfts.len(), 1, "{nfts:?}");
    let (l, coin, owner) = nfts[0];
    assert_eq!(l, launcher);
    assert_eq!(owner, keys.primary());
    let cats = holdings::own_cat_coins(&player.cache().wallet, asset);
    assert_eq!(cats.len(), 1);
    assert_eq!(cats[0].0, keys.primary());
    let cat_coins: Vec<Bytes32> = cats[0].1.iter().map(|c| c.coin_id()).collect();
    let primary = keys.primary();
    // The server's side, from the coins named in a proof.
    let server = reader(&sim, &account().1);
    let seen = server
        .read(
            WAIT,
            |src| async move { holdings::nft_coin(&src, coin).await },
        )
        .unwrap();
    assert_eq!((seen.launcher_id, seen.owner), (launcher, keys.primary()));
    let minter = server
        .read(WAIT, |src| async move {
            holdings::nft_minter(&src, launcher).await
        })
        .unwrap();
    assert_eq!(minter, None, "minted without a DID");
    let ids = cat_coins.clone();
    let amount = server
        .read(WAIT, |src| async move {
            holdings::cat_amount(&src, asset, primary, ids).await
        })
        .unwrap();
    assert_eq!(amount, 3_000);
    // Not the owner's: nothing counts.
    let ids = cat_coins.clone();
    let stranger = Bytes32::new([3; 32]);
    let none = server
        .read(WAIT, |src| async move {
            holdings::cat_amount(&src, asset, stranger, ids).await
        })
        .unwrap();
    assert_eq!(none, 0);
    // A plain coin made at the CAT's puzzle hash (anyone can): not a CAT, counts nothing.
    let fake = sim.fund(holdings::cat_puzzle_hash(asset, keys.primary()), 9_000);
    let ids = vec![fake.coin_id()];
    let faked = server
        .read(WAIT, |src| async move {
            holdings::cat_amount(&src, asset, primary, ids).await
        })
        .unwrap();
    assert_eq!(faked, 0);
    // A coin that isn't an NFT.
    let plain = sim.fund(keys.primary(), 4).coin_id();
    assert!(
        server
            .read(
                WAIT,
                |src| async move { holdings::nft_coin(&src, plain).await }
            )
            .is_err()
    );
}

#[test]
fn the_offer_pool_is_split_and_used() {
    let sim = Sim::start().unwrap();
    let (_, keys) = account();
    sim.fund(keys.primary(), 1_000);
    let server = reader(&sim, &keys);
    let push = server.split(&keys, 4, 1, 0, WAIT).unwrap();
    assert!(matches!(push, PushOutcome::Accepted { .. }), "{push:?}");
    assert!(until(|| server.coins_of(1) == 4), "{:?}", server.coins());
    // Two offers at once, each on a pool coin; the big change coin stays spendable.
    let spec = |price| MakeOffer {
        give: OfferGive::Xch(1),
        price,
        expires_at: Some(offer::now_seconds() + 3_600),
        fee: 0,
    };
    let a = server.make_offer(spec(100), &keys, WAIT).unwrap();
    let b = server.make_offer(spec(100), &keys, WAIT).unwrap();
    assert_ne!(a.offered_coins, b.offered_coins);
    assert_eq!(server.coins_of(1), 2);
    // A player takes one; the server sees it paid.
    let (_, pk) = account();
    sim.fund(pk.primary(), 500);
    let player = reader(&sim, &pk);
    let taken = player.take_offer(&a.text, &pk, 0, WAIT).unwrap();
    assert!(matches!(taken.push, PushOutcome::Accepted { .. }));
    assert!(until(|| matches!(
        server.offer_state(&a),
        OfferState::Taken { paid: true, .. }
    )));
    assert_eq!(server.offer_state(&b), OfferState::Open);
}

#[test]
fn the_simulator_is_reached_over_its_ws_address() {
    let sim = Sim::start().unwrap();
    let address = sim.address().unwrap();
    assert!(address.starts_with("ws://127.0.0.1:"), "{address}");
    let (_, keys) = account();
    sim.fund(keys.primary(), 77);
    let mut config = ChainConfig::new(ChainNetwork::Testnet11);
    config.target_peers = 1;
    config.puzzle_hashes = keys.puzzle_hashes();
    config.peers = vec![address];
    let r = ChainReader::start(config).unwrap();
    assert!(r.wait_synced(WAIT), "{:?}", r.status());
    assert_eq!(r.status().balance, 77);
    // A plain ws:// peer that isn't on this computer is refused.
    let mut far = ChainConfig::new(ChainNetwork::Testnet11);
    far.peers = vec!["ws://192.0.2.1:58444".into()];
    far.puzzle_hashes = keys.puzzle_hashes();
    let r = ChainReader::start(far).unwrap();
    assert!(
        until(|| r.status().phase == Phase::Offline),
        "{:?}",
        r.status()
    );
    assert!(r.status().peers.is_empty());
}

#[test]
fn a_reader_resuming_from_its_cache_finds_an_nft_minted_since() {
    let sim = Sim::start().unwrap();
    let (_, keys) = account();
    sim.pay(keys.primary(), 5_000).unwrap();
    let dir = std::env::temp_dir().join(format!("hz-holdings-resume-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let start = || {
        let mut config = ChainConfig::new(ChainNetwork::Testnet11);
        config.target_peers = 1;
        config.puzzle_hashes = keys.puzzle_hashes();
        config.cache_file = Some(dir.join("cache.json"));
        let r = ChainReader::start_with(config, sim.dialer()).unwrap();
        assert!(r.wait_synced(WAIT), "{:?}", r.status());
        r
    };
    start().stop();
    let launcher = sim.mint_nft(&keys).unwrap();
    let again = start();
    let wallet = again.cache().wallet;
    let nfts = again
        .read(WAIT, |src| async move {
            holdings::own_nfts(&src, &wallet).await
        })
        .unwrap();
    assert_eq!(
        nfts.iter().map(|n| n.0).collect::<Vec<_>>(),
        vec![launcher],
        "{:?}",
        again.cache().wallet.unspent_hinted()
    );
    let _ = std::fs::remove_dir_all(&dir);
}
