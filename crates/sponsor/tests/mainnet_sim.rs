//! The sponsor's mainnet opt-in on chia-sdk-test's peer simulator started
//! with mainnet's genesis: refused without `allow_mainnet`, its fee capped, `xch1` addresses
//! only, and a launch the gates let through signed for mainnet (this simulator checks spends with
//! testnet11's constants, so it refuses it: no real network, no real funds, throwaway keys).
mod common;

use chiacraft_sponsor::{Config, Sponsor, service::now_seconds};
use chiacraft_wallet::{
    IdentitySigner, Mnemonic,
    chain::{self, ChainNetwork, WalletKeys, sim::Sim},
    sponsor::Request,
};
use common::*;

fn mainnet_config(dir: &std::path::Path) -> Config {
    let mut c = Config::parse("network = \"mainnet\"\nallow_mainnet = true").unwrap();
    c.data_dir = dir.to_owned();
    c.batch_window_ms = 200;
    c.max_wait_seconds = 2;
    c.work_bits = 0;
    c.hourly_release = false;
    c
}

/// A throwaway player's request on mainnet (an `xch1` address).
fn mainnet_request() -> Request {
    let words = Mnemonic::generate().unwrap();
    let signer = IdentitySigner::from_mnemonic(&words, 0);
    let keys = WalletKeys::from_mnemonic(&words, ChainNetwork::Mainnet, 1);
    let address = chain::address(keys.primary(), ChainNetwork::Mainnet);
    Request::sign(&signer, ChainNetwork::Mainnet.name(), &address, now_seconds()).unwrap()
}

#[test]
fn mainnet_needs_allow_mainnet() {
    let sim = Sim::start_on(ChainNetwork::Mainnet).unwrap();
    let dir = scratch("mainnet-refused");
    let mut c = mainnet_config(&dir);
    c.allow_mainnet = false;
    let e = Sponsor::start_with(c, &Mnemonic::generate().unwrap(), sim.dialer()).unwrap_err();
    assert!(e.contains("allow_mainnet"), "{e}");
}

#[test]
fn a_mainnet_sponsor_serves_xch1_within_its_caps() {
    let sim = Sim::start_on(ChainNetwork::Mainnet).unwrap();
    let dir = scratch("mainnet");
    let words = Mnemonic::generate().unwrap();
    let funding = WalletKeys::from_mnemonic(&words, ChainNetwork::Mainnet, 1);
    sim.pay(funding.primary(), 100_000_000).unwrap();
    let s = Sponsor::start_with(mainnet_config(&dir), &words, sim.dialer()).unwrap();
    assert!(s.wait_synced(WAIT), "{:?}", s.status());
    assert!(s.address().starts_with("xch1"), "{}", s.address());
    let status = s.status();
    assert_eq!(status.network, "mainnet");
    assert_eq!(status.fee_per_did, 1_000_000);
    assert_eq!(status.per_endpoint_per_day, 1);
    // A testnet11 request (txch1, network testnet11) is refused; so is a txch1 address on mainnet.
    let player = Player::new();
    let r = s.request(&player.request(), &client(1)).unwrap_err();
    assert_eq!(r.status, 403, "{}", r.error);
    let mut wrong = mainnet_request();
    wrong.address = player.address.clone();
    let r = s.check(&wrong, now_seconds()).unwrap_err();
    assert_eq!(r.status, 400, "{}", r.error);
    // A good mainnet request passes the checks; its bundle is signed for mainnet, which this
    // simulator (testnet11's rules) refuses: the sponsor says so and spends nothing.
    let req = mainnet_request();
    assert!(s.check(&req, now_seconds()).is_ok());
    let r = s.request(&req, &client(2)).unwrap_err();
    assert_eq!(r.status, 503, "{}", r.error);
    assert!(r.error.contains("refused"), "{}", r.error);
    assert_eq!(sim.balance(funding.primary()), 100_000_000);
}

#[test]
fn a_fee_above_the_cap_launches_nothing() {
    let sim = Sim::start().unwrap();
    let dir = scratch("fee-cap");
    let mut c = config(&dir);
    c.fee_per_did = 1_000;
    c.max_fee_per_did = 999;
    let words = Mnemonic::generate().unwrap();
    // The config check refuses it at parse time; set after parsing, the chain reader's cap holds.
    let s = sponsor(&sim, c, &words, 1_000_000);
    let r = s.request(&Player::new().request(), &client(3)).unwrap_err();
    assert_eq!(r.status, 503, "{}", r.error);
    assert!(r.error.contains("max-did-fee"), "{}", r.error);
    assert_eq!(s.bundles(), 0);
}
