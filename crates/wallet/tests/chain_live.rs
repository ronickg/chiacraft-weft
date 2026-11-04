//! A live, read-only testnet11 check of the chain reader, opt-in: `HORIZON_CHAIN_LIVE=1`.
//! It finds peers through testnet11's DNS introducer, connects over TLS, syncs an address and
//! reports the peak; with `HORIZON_CHAIN_LIVE_DID=did:chia:1…` it also resolves that DID. It
//! spends nothing and holds no key of value: the address is `HORIZON_CHAIN_LIVE_ADDRESS` (any
//! public `txch1…`) or a throwaway one. Never mainnet. Not part of run_checks.py's default jobs.
//!
//! `HORIZON_CHAIN_LIVE=1 cargo test -p horizon-wallet --features chain --test chain_live -- --nocapture`
#![cfg(feature = "chain")]

use chiacraft_wallet::{
    Mnemonic,
    chain::{self, ChainConfig, ChainNetwork, ChainReader, Phase, WalletKeys},
};
use std::time::Duration;

#[test]
fn live_testnet11_read() {
    if std::env::var("HORIZON_CHAIN_LIVE").as_deref() != Ok("1") {
        eprintln!("skipped: set HORIZON_CHAIN_LIVE=1 for the live testnet11 read");
        return;
    }
    let puzzle_hash = match std::env::var("HORIZON_CHAIN_LIVE_ADDRESS") {
        Ok(a) => chain::parse_address(&a).expect("HORIZON_CHAIN_LIVE_ADDRESS"),
        Err(_) => WalletKeys::from_mnemonic(&Mnemonic::generate().unwrap(), ChainNetwork::Testnet11, 1).primary(),
    };
    let dir = std::env::temp_dir().join(format!("hz-chain-live-{}", std::process::id()));
    let mut config = ChainConfig::new(ChainNetwork::Testnet11);
    config.puzzle_hashes = vec![puzzle_hash];
    config.cache_file = Some(dir.join("chain-cache.json"));
    let reader = ChainReader::start(config).expect("reader");
    let synced = reader.wait_synced(Duration::from_secs(120));
    let status = reader.status();
    eprintln!("{}", serde_json::to_string_pretty(&status).unwrap());
    assert!(synced, "no testnet11 peer synced in 2 minutes: {status:?}");
    assert_eq!(status.phase, Phase::Synced);
    // testnet11 passed height 4.3 million in July 2026.
    assert!(status.peak.unwrap_or(0) > 4_000_000, "peak {:?}", status.peak);
    if let Ok(did) = std::env::var("HORIZON_CHAIN_LIVE_DID") {
        let id = chain::parse_did(&did).expect("HORIZON_CHAIN_LIVE_DID");
        let found = reader
            .resolve_did(id, Duration::from_secs(60))
            .expect("resolve");
        eprintln!(
            "{}: {} coins in its history, owner {:?}, melted {}",
            found.record.did(),
            found.record.history.len(),
            found.record.owner().map(chain::hex32),
            found.record.melted
        );
        assert!(found.live);
    }
    reader.stop();
    let _ = std::fs::remove_dir_all(dir);
}
