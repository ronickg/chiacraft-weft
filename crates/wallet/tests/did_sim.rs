//! Identity row I8 on chia-sdk-test's peer simulator: a DID made and paid by the wallet itself, a
//! DID launched by a sponsor straight to the player's address (still eve, checked by its puzzle
//! hash), coins at hardened addresses (as Sage derives them) found and spent, and an account kept
//! under a passphrase signing only once the passphrase is given. Throwaway keys, no network.
#![cfg(feature = "sim")]

use chiacraft_wallet::{
    Accounts, Mnemonic, OwnerSigner, Secret, StoreChoice, StoreKind,
    chain::{
        Bytes32, ChainConfig, ChainNetwork, ChainReader, PushOutcome, WalletKeys, did, holdings,
        sim::Sim,
    },
    keys,
};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(20);
const STEP: Duration = Duration::from_millis(100);

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
fn a_did_paid_by_the_wallet_is_made_watched_and_found_ours() {
    let sim = Sim::start().unwrap();
    let words = Mnemonic::generate().unwrap();
    let keys = WalletKeys::with_hardened(&words, ChainNetwork::Testnet11, 3, 3);
    let r = reader(&sim, keys.puzzle_hashes());
    // An empty wallet can't pay: the player is told how much is needed.
    let e = r.create_did(&keys, 5, WAIT).unwrap_err();
    assert!(e.to_string().contains("at least 6 mojos"), "{e}");
    sim.pay(keys.primary(), 1_000).unwrap();
    assert!(until(|| r.status().balance == 1_000), "{:?}", r.status());
    let made = r.create_did(&keys, 10, WAIT).unwrap();
    assert!(
        !matches!(made.push, PushOutcome::Rejected { .. }),
        "{:?}",
        made.push
    );
    let record = r.wait_did(made.launcher_id, STEP, WAIT).unwrap();
    assert_eq!(record.launcher_id, made.launcher_id);
    let owner = record.owner().expect("a wallet-made DID shows its owner");
    assert_eq!(owner, made.owner);
    assert!(keys.puzzle_hashes().contains(&owner));
    // Linking: which of the account's keys owns it, and that key's proof checks on chain.
    let signer = OwnerSigner::find(&words, owner.to_bytes(), 20).unwrap();
    assert!(holdings::did_owned_by_key(&record, &signer.public()));
    // The wallet paid 1 mojo and the fee.
    assert!(
        until(|| r.status().balance == 1_000 - 11),
        "{:?}",
        r.status()
    );
}

#[test]
fn a_sponsored_did_is_the_players_while_still_eve() {
    let sim = Sim::start().unwrap();
    let words = Mnemonic::generate().unwrap();
    let keys = WalletKeys::from_mnemonic(&words, ChainNetwork::Testnet11, 2);
    let r = reader(&sim, keys.puzzle_hashes());
    let launcher = sim.sponsor_did(keys.primary()).unwrap();
    let record = r.wait_did(launcher, STEP, WAIT).unwrap();
    assert!(record.is_eve());
    assert_eq!(record.owner(), None, "an eve DID shows no owner");
    assert!(record.owned_by(keys.primary()));
    assert_eq!(
        record.current().unwrap().puzzle_hash,
        did::eve_puzzle_hash(launcher, keys.primary())
    );
    let me = OwnerSigner::from_mnemonic(&words, 0, false);
    assert!(holdings::did_owned_by_key(&record, &me.public()));
    // Someone else's key doesn't own it.
    let other = OwnerSigner::from_mnemonic(&Mnemonic::generate().unwrap(), 0, false);
    assert!(!holdings::did_owned_by_key(&record, &other.public()));
    assert!(!record.owned_by(keys.puzzle_hashes()[1]));
    // A launcher that doesn't exist never confirms.
    assert!(r.wait_did(Bytes32::new([4; 32]), STEP, STEP * 3).is_err());
}

#[test]
fn coins_at_hardened_addresses_are_found_and_spent() {
    let sim = Sim::start().unwrap();
    let words = Mnemonic::generate().unwrap();
    let hardened = keys::hardened_puzzle_hashes(&words, 2);
    // Coins Sage would show at the hardened address 1, none at the unhardened ones.
    let at = Bytes32::from(hardened[1]);
    sim.pay(at, 700).unwrap();
    let plain = WalletKeys::from_mnemonic(&words, ChainNetwork::Testnet11, 3);
    let only_plain = reader(&sim, plain.puzzle_hashes());
    assert_eq!(only_plain.status().balance, 0, "unhardened only: nothing");
    only_plain.stop();
    let keys = WalletKeys::with_hardened(&words, ChainNetwork::Testnet11, 3, 2);
    let r = reader(&sim, keys.puzzle_hashes());
    assert!(until(|| r.status().balance == 700), "{:?}", r.status());
    // The hardened coin pays for a DID; the change goes to the primary address.
    let made = r.create_did(&keys, 0, WAIT).unwrap();
    let record = r.wait_did(made.launcher_id, STEP, WAIT).unwrap();
    let owner = record.owner().unwrap();
    assert!(keys.puzzle_hashes().contains(&owner));
    assert!(until(
        || sim.balance(keys.primary()) == 699 || sim.balance(owner) >= 699
    ));
    assert_eq!(sim.balance(at), 0);
    // A DID at a hardened address is found by the owner search too.
    let found = OwnerSigner::find(&words, hardened[1], 20).unwrap();
    assert!(found.hardened() && found.index() == 1);
}

#[test]
fn a_passphrase_protected_account_signs_only_with_its_passphrase() {
    let home = std::env::temp_dir().join(format!("cc-i8-pass-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    let choice = StoreChoice {
        kind: StoreKind::File,
        service: "chiacraft wallet test".into(),
        file_dir: home.join(chiacraft_wallet::accounts::WALLET_DIR),
    };
    let mut accounts = Accounts::load(&home, choice).unwrap();
    let info = accounts.create("A").unwrap().0;
    let f = info.fingerprint;
    accounts
        .set_passphrase(f, Secret::new(String::new()), Secret::new("pw 1".into()))
        .unwrap();
    assert!(accounts.needs_passphrase(f));
    assert!(
        accounts
            .reveal(f, Secret::new(String::new()))
            .unwrap_err()
            .is_wrong_passphrase()
    );
    assert!(
        accounts
            .reveal(f, Secret::new("pw 2".into()))
            .unwrap_err()
            .is_wrong_passphrase()
    );
    // With it: the spending keys for one call, and the hardened addresses for the reader.
    let words = accounts.reveal(f, Secret::new("pw 1".into())).unwrap();
    let unlocked = accounts.unlock(f, Secret::new("pw 1".into())).unwrap();
    assert_eq!(unlocked.hardened.len(), keys::HARDENED_ADDRESSES as usize);
    assert_eq!(unlocked.hardened, keys::hardened_puzzle_hashes(&words, 20));
    let sim = Sim::start().unwrap();
    let spending = WalletKeys::with_hardened(&words, ChainNetwork::Testnet11, 2, 2);
    drop(words);
    sim.pay(spending.primary(), 50).unwrap();
    let r = reader(&sim, spending.puzzle_hashes());
    assert!(until(|| r.status().balance == 50));
    let made = r.create_did(&spending, 0, WAIT).unwrap();
    assert!(r.wait_did(made.launcher_id, STEP, WAIT).is_ok());
    assert!(!format!("{spending:?}").contains("pw 1"));
    let _ = std::fs::remove_dir_all(home);
}
