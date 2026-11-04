//! The achievements' daily anchor (identity row I10) against chia-sdk-test's peer simulator: a
//! server's reader puts a day's Merkle root on chain as a coin's memo, anyone's reader reads it back
//! from the anchor coin, and a credential's inclusion proof leads to that root. Throwaway keys, no
//! network.
#![cfg(feature = "sim")]

use chiacraft_wallet::{
    IdentitySigner, Mnemonic, Signer,
    achievement::{self, Achievement, Anchor, Credential, Issuer},
    chain::{ChainConfig, ChainNetwork, ChainReader, PushOutcome, WalletKeys, sim::Sim},
};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(20);

fn reader(sim: &Sim, keys: &WalletKeys) -> ChainReader {
    let mut config = ChainConfig::new(ChainNetwork::Testnet11);
    config.target_peers = 1;
    config.puzzle_hashes = keys.puzzle_hashes();
    let r = ChainReader::start_with(config, sim.dialer()).unwrap();
    assert!(r.wait_synced(WAIT), "{:?}", r.status());
    r
}

#[test]
fn a_days_root_goes_on_chain_and_proofs_lead_to_it() {
    let sim = Sim::start().unwrap();
    let words = Mnemonic::generate().unwrap();
    let keys = WalletKeys::from_mnemonic(&words, ChainNetwork::Testnet11, 3);
    sim.fund(keys.primary(), 1_000);
    let server = reader(&sim, &keys);
    // Three credentials a server signed today.
    let issuer = IdentitySigner::from_mnemonic(&words, 0);
    let player = IdentitySigner::from_mnemonic(&Mnemonic::generate().unwrap(), 0);
    let creds: Vec<Credential> = (0..3)
        .map(|i| {
            Credential::unsigned(
                Issuer { key: issuer.identity().hex(), name: "Sim server".into(), did: None },
                &player.identity().0,
                Achievement { id: format!("t:a{i}"), title: format!("A{i}"), description: "d".into(), icon: None },
                1_790_000_000,
                achievement::new_nonce().unwrap(),
                None,
                None,
            )
            .sign(&issuer)
            .unwrap()
        })
        .collect();
    let hashes: Vec<[u8; 32]> = creds.iter().map(Credential::hash).collect();
    let leaves: Vec<[u8; 32]> = hashes.iter().copied().map(achievement::leaf).collect();
    let root = achievement::merkle_root(&leaves);
    let (push, coin) = server.anchor(&keys, root, 0, WAIT).unwrap();
    assert!(matches!(push, PushOutcome::Accepted { .. }), "{push:?}");
    // Anyone's reader (another account's) reads the root back from the anchor coin.
    let other = WalletKeys::from_mnemonic(&Mnemonic::generate().unwrap(), ChainNetwork::Testnet11, 1);
    let anyone = reader(&sim, &other);
    let mut seen = None;
    let end = std::time::Instant::now() + WAIT;
    while std::time::Instant::now() < end {
        seen = anyone.anchor_root(coin, WAIT).unwrap();
        if seen.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert_eq!(seen, Some(root));
    // Each credential's proof leads to it.
    for (i, c) in creds.iter().enumerate() {
        let mut a = Anchor::of(achievement::day_of(c.issued), &hashes, i, "sim");
        a.coin = Some(chiacraft_wallet::keys::hex(&coin.to_bytes()));
        assert_eq!(a.root, chiacraft_wallet::keys::hex(&root));
        let mut held = c.clone();
        held.anchor = Some(a);
        assert!(held.verify().unwrap().anchored.is_some());
    }
    // A coin with no anchor memo carries no root.
    let plain = sim.unspent(other.primary());
    assert!(
        plain.is_empty()
            || anyone
                .anchor_root(plain[0].coin_id(), WAIT)
                .unwrap()
                .is_none()
    );
}
