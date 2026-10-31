//! Sage interop (identity row I8): a signature made the way Sage's `signMessageByAddress` makes
//! one verifies with our checks, and ours is the same signature.
//!
//! Sage (`xch-dev/sage`, `crates/sage/src/endpoints/wallet_connect.rs`, read 2026-10-30, not
//! run): it looks up the address's derivation (index, hardened or not), derives that wallet key
//! from the master secret key (`m/12381/8444/2/i`, hardened at every step or not), takes its
//! **synthetic** key, reads the message with `parse_signature_message` (an optional `0x`, then
//! hex if every character is a hex digit, else the UTF-8 bytes), signs the tree hash of
//! `("Chia Signed Message" . message)` with AugSchemeMPL and returns the synthetic public key and
//! the signature. Below that procedure is written again from that description, independently of
//! `chiacraft_wallet::sign`, and compared. No vector from Sage itself was available (Sage's file has
//! no tests); the fixed vector at the end pins our output for the public test words.
use chia_bls::{DerivableKey, SecretKey};
use chia_puzzle_types::{DeriveSynthetic, standard::StandardArgs};
use chiacraft_wallet::{Mnemonic, OwnerSigner, keys, sign};
use sha2::{Digest, Sha256};

const TEST_WORDS: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Sage's `parse_signature_message`.
fn sage_parse(input: &str) -> Vec<u8> {
    let stripped = input.strip_prefix("0x").unwrap_or(input);
    if !stripped.is_empty() && stripped.chars().all(|c| c.is_ascii_hexdigit()) {
        keys::from_hex(stripped).expect("Sage refuses odd hex")
    } else {
        input.as_bytes().to_vec()
    }
}

/// The CLVM tree hash of `("Chia Signed Message" . message)`, by hand.
fn sage_hash(message: &[u8]) -> [u8; 32] {
    let atom = |b: &[u8]| -> [u8; 32] {
        let mut h = Sha256::new();
        h.update([1u8]);
        h.update(b);
        h.finalize().into()
    };
    let mut h = Sha256::new();
    h.update([2u8]);
    h.update(atom(b"Chia Signed Message"));
    h.update(atom(message));
    h.finalize().into()
}

/// Sage's signing key for wallet address `index`.
fn sage_key(words: &str, index: u32, hardened: bool) -> SecretKey {
    let m = Mnemonic::parse(words).unwrap();
    let master = SecretKey::from_seed(m.seed().expose());
    let path = [12381u32, 8444, 2, index];
    let key = if hardened {
        path.iter().fold(master, |k, i| k.derive_hardened(*i))
    } else {
        path.iter().fold(master, |k, i| k.derive_unhardened(*i))
    };
    key.derive_synthetic()
}

fn sage_sign(words: &str, index: u32, hardened: bool, message: &str) -> ([u8; 48], [u8; 96]) {
    let sk = sage_key(words, index, hardened);
    let sig = chia_bls::sign(&sk, sage_hash(&sage_parse(message)));
    (sk.public_key().to_bytes(), sig.to_bytes())
}

#[test]
fn sage_style_signatures_verify_and_match_ours() {
    let m = Mnemonic::parse(TEST_WORDS).unwrap();
    let text = "chiacraft did proof v1\ndid: did:chia:1x\nsession: 00";
    for (index, hardened) in [(0, false), (3, false), (0, true), (5, true)] {
        let (public, signature) = sage_sign(TEST_WORDS, index, hardened, text);
        // Our checks accept Sage's signature …
        assert!(sign::verify_owner(&public, text.as_bytes(), &signature));
        // … and the address Sage signs for is the one our owner check compares.
        let ph = StandardArgs::curry_tree_hash(chia_bls::PublicKey::from_bytes(&public).unwrap())
            .to_bytes();
        assert_eq!(keys::p2_of_synthetic(&public), Some(ph));
        if !hardened {
            let facts = keys::facts_of(&m);
            assert_eq!(keys::wallet_puzzle_hash(&facts.master, index).unwrap(), ph);
        } else {
            assert_eq!(
                keys::hardened_puzzle_hashes(&m, index + 1)[index as usize],
                ph
            );
        }
        // Ours is the very same signature (BLS signing is deterministic).
        let ours = OwnerSigner::from_mnemonic(&m, index, hardened);
        assert_eq!(ours.public(), public);
        assert_eq!(ours.sign(text).unwrap(), signature);
        assert_eq!(OwnerSigner::find(&m, ph, 8).unwrap().hardened(), hardened);
    }
}

#[test]
fn our_texts_are_never_read_as_hex_by_sage() {
    // Every text we sign starts with "chiacraft ", which has a space and non-hex letters, so
    // Sage signs its UTF-8 bytes as we do. A hex-looking message would differ.
    for text in [
        "chiacraft sign-in v1\nsession: 00",
        "chiacraft sponsor v1 txch1abc 1700000000",
    ] {
        assert_eq!(sage_parse(text), text.as_bytes());
        assert_eq!(
            sage_hash(&sage_parse(text)),
            sign::message_hash(text.as_bytes())
        );
    }
    assert_ne!(sage_parse("0xcafe"), b"0xcafe".to_vec());
}

#[test]
fn a_fixed_vector_for_the_public_test_words() {
    // Pinned from this crate (no vector from Sage itself exists): address 0, unhardened.
    let text = "chiacraft interop v1";
    let (public, signature) = sage_sign(TEST_WORDS, 0, false, text);
    assert_eq!(keys::hex(&public), PUBLIC_0);
    assert_eq!(keys::hex(&signature), SIGNATURE_0);
}

const PUBLIC_0: &str = "8d9a5ed9c9b1a58476b07262007c636d775f2a33f0533737f3b3b0eaf99a8c0c51b3f2d87dc03a657e07f1828ab760fa";
const SIGNATURE_0: &str = "96b2871f2704e7d329ceaf5e94520661165c1751aa2c7b17507079863202de9d09c45724069160fff5335ee7b99e258a114926fb7a64f0e10e8548593237c3558cc52a02096a33285b8da34ad646cf650cccf0eaa960fe8d8e03348292a68a4f";
