//! Identity row I12 on chia-sdk-test's peer simulator: mainnet only with the operator's
//! confirmation, and every spend held to its caps. The simulator is started with mainnet's
//! genesis challenge, so a reader configured for mainnet syncs from it; its spend checks stay
//! testnet11's, so a spend the gates let through is signed for mainnet and rejected here (proof
//! that it committed to mainnet's data). Throwaway keys, no real network, no real funds.
#![cfg(feature = "sim")]

use chiacraft_wallet::{
    Mnemonic,
    chain::{
        Bytes32, ChainConfig, ChainNetwork, ChainReader, MakeOffer, OfferGive, PushOutcome,
        WalletKeys, sim::Sim,
    },
    spending::{self, MAINNET_CONFIRM, SpendCaps},
};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(20);

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

fn mainnet_config(puzzle_hashes: Vec<Bytes32>) -> ChainConfig {
    let (network, consent) =
        spending::choose_network("mainnet", MAINNET_CONFIRM, "chain", "chain-mainnet-confirm")
            .unwrap();
    let mut config = ChainConfig::chosen(network, consent);
    config.target_peers = 1;
    config.puzzle_hashes = puzzle_hashes;
    config
}

#[test]
fn mainnet_needs_the_confirmation() {
    let sim = Sim::start_on(ChainNetwork::Mainnet).unwrap();
    let mut config = ChainConfig::new(ChainNetwork::Mainnet);
    config.target_peers = 1;
    let e = ChainReader::start_with(config, sim.dialer()).unwrap_err();
    assert!(e.to_string().contains(MAINNET_CONFIRM), "{e}");
    // The settings refuse it too, naming the second setting.
    let e =
        spending::choose_network("mainnet", "yes", "chain", "chain-mainnet-confirm").unwrap_err();
    assert!(e.to_string().contains("chain-mainnet-confirm"), "{e}");
    // Caps made for another network are refused.
    let mut config = mainnet_config(Vec::new());
    config.caps = SpendCaps::testnet11();
    assert!(ChainReader::start_with(config, sim.dialer()).is_err());
    // Confirmed: it starts and syncs from mainnet's genesis.
    let r = ChainReader::start_with(mainnet_config(Vec::new()), sim.dialer()).unwrap();
    assert!(r.wait_synced(WAIT), "{:?}", r.status());
    assert_eq!(r.network(), ChainNetwork::Mainnet);
    assert_eq!(r.caps(), SpendCaps::mainnet());
}

#[test]
fn mainnet_caps_refuse_every_spend_above_them() {
    let sim = Sim::start_on(ChainNetwork::Mainnet).unwrap();
    let words = Mnemonic::generate().unwrap();
    let keys = WalletKeys::from_mnemonic(&words, ChainNetwork::Mainnet, 2);
    let r = ChainReader::start_with(mainnet_config(keys.puzzle_hashes()), sim.dialer()).unwrap();
    assert!(r.wait_synced(WAIT), "{:?}", r.status());
    let funds = 50_000_000_000;
    sim.pay(keys.primary(), funds).unwrap();
    assert!(until(|| r.status().balance == funds), "{:?}", r.status());
    let caps = SpendCaps::mainnet();
    let refused = |e: chiacraft_wallet::Error, cap: &str| {
        let t = e.to_string();
        assert!(t.starts_with("refused"), "{t}");
        assert!(t.contains(cap), "{t}");
        assert!(t.contains("mainnet"), "{t}");
    };
    refused(
        r.create_did(&keys, caps.max_did_fee + 1, WAIT).unwrap_err(),
        "max-did-fee",
    );
    refused(
        r.make_offer(
            MakeOffer {
                give: OfferGive::Xch(1),
                price: caps.max_offer_price + 1,
                expires_at: None,
                fee: 0,
            },
            &keys,
            WAIT,
        )
        .unwrap_err(),
        "max-offer-price",
    );
    refused(
        r.make_offer(
            MakeOffer {
                give: OfferGive::Xch(1),
                price: 1_000,
                expires_at: None,
                fee: caps.max_fee + 1,
            },
            &keys,
            WAIT,
        )
        .unwrap_err(),
        "max-fee",
    );
    refused(
        r.split(&keys, 4, 1, caps.max_fee + 1, WAIT).unwrap_err(),
        "max-fee",
    );
    refused(
        r.split(&keys, 2, caps.max_split, 0, WAIT).unwrap_err(),
        "max-split",
    );
    refused(
        r.anchor(&keys, [7; 32], caps.max_anchor_fee + 1, WAIT)
            .unwrap_err(),
        "max-anchor-fee",
    );
    refused(
        r.launch_dids(&keys, &[keys.primary()], caps.max_did_fee + 1, WAIT)
            .unwrap_err(),
        "max-did-fee",
    );
    // A raw bundle passed no cap: refused on mainnet.
    let e = r
        .push(
            chiacraft_wallet::chain::SpendBundle::new(Vec::new(), Default::default()),
            WAIT,
        )
        .unwrap_err();
    assert!(e.to_string().contains("raw pushes"), "{e}");
    // Nothing above moved a mojo.
    assert_eq!(r.spendable_balance(), funds);
    assert_eq!(sim.balance(keys.primary()), funds);

    // Under the caps the spend is built, signed for mainnet and pushed: this simulator checks
    // with testnet11's constants, so it rejects the mainnet signature.
    let made = r.create_did(&keys, caps.max_did_fee, WAIT).unwrap();
    match made.push {
        PushOutcome::Rejected { .. } => {}
        other => panic!("a mainnet-signed spend passed testnet11's check: {other:?}"),
    }
    assert_eq!(sim.balance(keys.primary()), funds);
}

#[test]
fn taking_an_offer_above_the_cap_is_refused_before_signing() {
    // A maker on testnet11 asks 1 XCH; the offer text is the same on any network.
    let maker_sim = Sim::start().unwrap();
    let maker =
        WalletKeys::from_mnemonic(&Mnemonic::generate().unwrap(), ChainNetwork::Testnet11, 1);
    let mut config = ChainConfig::new(ChainNetwork::Testnet11);
    config.target_peers = 1;
    config.puzzle_hashes = maker.puzzle_hashes();
    let m = ChainReader::start_with(config, maker_sim.dialer()).unwrap();
    assert!(m.wait_synced(WAIT));
    maker_sim.pay(maker.primary(), 10).unwrap();
    assert!(until(|| m.status().balance == 10));
    let offer = m
        .make_offer(
            MakeOffer {
                give: OfferGive::Xch(1),
                price: spending::MOJOS_PER_XCH,
                expires_at: None,
                fee: 0,
            },
            &maker,
            WAIT,
        )
        .unwrap();

    let sim = Sim::start_on(ChainNetwork::Mainnet).unwrap();
    let keys = WalletKeys::from_mnemonic(&Mnemonic::generate().unwrap(), ChainNetwork::Mainnet, 1);
    let r = ChainReader::start_with(mainnet_config(keys.puzzle_hashes()), sim.dialer()).unwrap();
    assert!(r.wait_synced(WAIT));
    sim.pay(keys.primary(), 2 * spending::MOJOS_PER_XCH)
        .unwrap();
    assert!(until(|| r.status().balance > 0));
    let e = r.take_offer(&offer.text, &keys, 0, WAIT).unwrap_err();
    assert!(e.to_string().contains("max-take"), "{e}");
    let e = r
        .take_offer(&offer.text, &keys, SpendCaps::mainnet().max_fee + 1, WAIT)
        .unwrap_err();
    assert!(e.to_string().contains("max-fee"), "{e}");
    assert_eq!(sim.balance(keys.primary()), 2 * spending::MOJOS_PER_XCH);
}

#[test]
fn testnet_caps_set_by_the_operator_hold_too() {
    let sim = Sim::start().unwrap();
    let keys =
        WalletKeys::from_mnemonic(&Mnemonic::generate().unwrap(), ChainNetwork::Testnet11, 2);
    let mut config = ChainConfig::new(ChainNetwork::Testnet11);
    config.target_peers = 1;
    config.puzzle_hashes = keys.puzzle_hashes();
    config.caps = SpendCaps::testnet11()
        .with_overrides("chain-", |n| (n == "max-did-fee").then(|| "10".to_owned()))
        .unwrap();
    let r = ChainReader::start_with(config, sim.dialer()).unwrap();
    assert!(r.wait_synced(WAIT));
    sim.pay(keys.primary(), 1_000).unwrap();
    assert!(until(|| r.status().balance == 1_000));
    let e = r.create_did(&keys, 11, WAIT).unwrap_err();
    assert!(e.to_string().contains("max-did-fee = 10 mojos"), "{e}");
    assert!(e.to_string().contains("TXCH"), "{e}");
    let made = r.create_did(&keys, 10, WAIT).unwrap();
    assert!(
        !matches!(made.push, PushOutcome::Rejected { .. }),
        "{:?}",
        made.push
    );
}
