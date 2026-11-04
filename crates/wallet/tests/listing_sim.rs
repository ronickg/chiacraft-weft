//! Listing a world (plan phase 5, slice 1) on chia-sdk-test's peer simulator: the owner locks a
//! deposit in a coin hinted with the listing tag naming the world DID; any reader finds it with
//! the node filtering by the deposit, a coin with the tag but less than the deposit is left out,
//! coin selection never spends the listing, and delisting returns the deposit and takes it off
//! the list. Throwaway keys, no network.
#![cfg(feature = "sim")]

use chiacraft_wallet::{
    Mnemonic,
    chain::{Bytes32, ChainConfig, ChainNetwork, ChainReader, PushOutcome, WalletKeys, sim::Sim},
};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(20);
const DEPOSIT: u64 = 100_000_000;

fn reader(sim: &Sim, puzzle_hashes: Vec<Bytes32>) -> ChainReader {
    let mut config = ChainConfig::new(ChainNetwork::Testnet11);
    config.target_peers = 1;
    config.puzzle_hashes = puzzle_hashes;
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
fn a_listing_is_found_by_its_deposit_and_delisting_takes_it_off() {
    let sim = Sim::start().unwrap();
    let owner = WalletKeys::from_mnemonic(&Mnemonic::generate().unwrap(), ChainNetwork::Testnet11, 2);
    sim.fund(owner.primary(), 3 * DEPOSIT);
    let r = reader(&sim, owner.puzzle_hashes());
    assert!(until(|| r.spendable_balance() == 3 * DEPOSIT));
    let world_did = Bytes32::new([5; 32]);

    let (push, coin) = r
        .list_world(&owner, owner.primary(), world_did, DEPOSIT, 0, WAIT)
        .unwrap();
    assert!(!matches!(push, PushOutcome::Rejected { .. }), "{push:?}");
    assert_eq!((coin.puzzle_hash, coin.amount), (owner.primary(), DEPOSIT));

    // Another game finds it: the node filters by the deposit, the memo names the DID.
    let other = reader(&sim, vec![Bytes32::new([1; 32])]);
    let mut found = Vec::new();
    assert!(until(|| {
        found = other.listings(DEPOSIT, 100, WAIT).unwrap_or_default();
        !found.is_empty()
    }));
    assert_eq!(found.len(), 1);
    assert_eq!((found[0].coin, found[0].world_did), (coin, world_did));

    // The listing is reserved: what the wallet may spend leaves it out.
    assert!(until(|| r.spendable_balance() == 2 * DEPOSIT));

    // A junk coin with the tag but less than the deposit isn't listed.
    let junk = WalletKeys::from_mnemonic(&Mnemonic::generate().unwrap(), ChainNetwork::Testnet11, 1);
    sim.fund(junk.primary(), DEPOSIT);
    let jr = reader(&sim, junk.puzzle_hashes());
    assert!(until(|| jr.spendable_balance() == DEPOSIT));
    jr.list_world(&junk, junk.primary(), Bytes32::new([6; 32]), DEPOSIT / 100, 0, WAIT)
        .unwrap();
    std::thread::sleep(Duration::from_secs(2));
    let again = other.listings(DEPOSIT, 100, WAIT).unwrap();
    assert_eq!(again.iter().map(|l| l.world_did).collect::<Vec<_>>(), vec![world_did]);
    // With a lower floor it shows, which is why the game asks for the deposit.
    assert!(until(|| other.listings(1, 100, WAIT).is_ok_and(|l| l.len() == 2)));

    // Delisting spends it back: the deposit returns and the world leaves the list.
    let push = r.delist_world(&owner, coin, 0, WAIT).unwrap();
    assert!(!matches!(push, PushOutcome::Rejected { .. }), "{push:?}");
    assert!(until(|| other
        .listings(DEPOSIT, 100, WAIT)
        .is_ok_and(|l| l.is_empty())));
    assert!(until(|| r.spendable_balance() == 3 * DEPOSIT));
}
