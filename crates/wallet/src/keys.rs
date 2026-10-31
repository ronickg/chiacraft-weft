//! Keys from the mnemonic (identity-research.md 1.2): Chia's EIP-2333 tree
//! `m/12381/8444/<purpose>/<index>`. Purpose 2 holds the ordinary wallet keys (what Sage finds
//! when the player imports the words); purpose [`IDENTITY_PURPOSE`] holds the **identity keys**,
//! hardened all the way down, so a server that sees an identity key learns nothing about the
//! wallet's coins and can't derive any other key.
//!
//! Ported from Ronald's `@chia/core` `wallet.ts` (`deriveWallet`: seed with an empty passphrase,
//! `secretKeyFromSeed`, the unhardened `m/12381/8444/2` intermediate, `deriveSynthetic`, the
//! standard puzzle hash, bech32m addresses) onto the crates chia-wallet-sdk 0.37.1 is built from.
use crate::{Error, Result, secret::Secret};
use chia_bls::{DerivableKey, PublicKey, SecretKey};
use chia_puzzle_types::{DeriveSynthetic, standard::StandardArgs};

/// Chia's BLS key tree: `m/12381/8444/...` (EIP-2333 purpose 12381, Chia's coin type 8444).
pub const CHIA_ROOT: [u32; 2] = [12381, 8444];
/// The ordinary wallet keys (as Sage and Ronald's wallet).
pub const WALLET_PURPOSE: u32 = 2;
/// The identity keys' purpose number: 25443 is "cc" in ASCII (0x6363). Chia's reference wallet
/// uses 0 (farmer), 1 (pool), 2 (wallet), 3 (local), 4 (backup), 5 (singleton owner) and 6 (pool
/// authentication); nothing uses this one. Fixed forever: changing it orphans every identity.
pub const IDENTITY_PURPOSE: u32 = 25443;

/// A Chia network, for addresses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet11,
}
impl Network {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Mainnet => "xch",
            Self::Testnet11 => "txch",
        }
    }
}

/// The public half of an identity key (BLS12-381 G1, compressed): what servers see.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdentityKey(pub [u8; 48]);
impl IdentityKey {
    /// A key from bytes, checked to be a valid point.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let bytes: [u8; 48] = bytes.try_into().ok()?;
        PublicKey::from_bytes(&bytes).ok()?;
        Some(Self(bytes))
    }
    pub fn from_hex(text: &str) -> Option<Self> {
        Self::from_bytes(&from_hex(text)?)
    }
    pub fn hex(&self) -> String {
        hex(&self.0)
    }
}
impl std::fmt::Debug for IdentityKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IdentityKey({}…)", &self.hex()[..16])
    }
}

/// An account's public facts, computed once from the seed and kept in `accounts.json`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicFacts {
    /// The master public key: wallet addresses derive from it with no secret.
    pub master: [u8; 48],
    /// Chia's key fingerprint of the master key (what Sage shows for the same words).
    pub fingerprint: u32,
    /// Identity key 0, the player's default identity.
    pub identity: IdentityKey,
}

/// The master secret key of a seed.
pub(crate) fn master(seed: &Secret<[u8; 64]>) -> SecretKey {
    SecretKey::from_seed(seed.expose())
}

pub(crate) fn derive_hardened(key: &SecretKey, path: &[u32]) -> SecretKey {
    path.iter()
        .fold(key.clone(), |k, index| k.derive_hardened(*index))
}

/// Identity key `index`'s secret: `m/12381/8444/25443/index`, hardened at every step.
pub(crate) fn identity_secret(master: &SecretKey, index: u32) -> SecretKey {
    derive_hardened(
        master,
        &[CHIA_ROOT[0], CHIA_ROOT[1], IDENTITY_PURPOSE, index],
    )
}

pub(crate) fn facts(master: &SecretKey) -> PublicFacts {
    let public = master.public_key();
    PublicFacts {
        master: public.to_bytes(),
        fingerprint: public.get_fingerprint(),
        identity: IdentityKey(identity_secret(master, 0).public_key().to_bytes()),
    }
}

/// The public facts of a mnemonic (no secret survives the call).
pub fn facts_of(mnemonic: &crate::Mnemonic) -> PublicFacts {
    facts(&master(&mnemonic.seed()))
}

/// The standard puzzle hash of wallet address `index` (the unhardened
/// `m/12381/8444/2/index`, then the synthetic key), from the master public key alone.
pub fn wallet_puzzle_hash(master_public: &[u8; 48], index: u32) -> Result<[u8; 32]> {
    let master =
        PublicKey::from_bytes(master_public).map_err(|_| Error::new("bad master public key"))?;
    let key = [CHIA_ROOT[0], CHIA_ROOT[1], WALLET_PURPOSE, index]
        .iter()
        .fold(master, |k, i| k.derive_unhardened(*i));
    Ok(StandardArgs::curry_tree_hash(key.derive_synthetic()).to_bytes())
}

/// The standard puzzle hashes of wallet addresses `0..count` (unhardened), from the master
/// public key alone.
pub fn wallet_puzzle_hashes(master_public: &[u8; 48], count: u32) -> Result<Vec<[u8; 32]>> {
    (0..count)
        .map(|i| wallet_puzzle_hash(master_public, i))
        .collect()
}

/// How many hardened wallet addresses the game follows beside the unhardened ones (identity row
/// I8): Sage derives both branches, so words imported from Sage may hold coins and DIDs at
/// `m/12381'/8444'/2'/i'` addresses too.
pub const HARDENED_ADDRESSES: u32 = 20;

/// The standard puzzle hash of hardened wallet address `index` (`m/12381'/8444'/2'/index'`, then
/// the synthetic key). Hardened keys need the secret: there is no public derivation.
pub(crate) fn hardened_puzzle_hash(master: &SecretKey, index: u32) -> [u8; 32] {
    let key = derive_hardened(master, &[CHIA_ROOT[0], CHIA_ROOT[1], WALLET_PURPOSE, index])
        .derive_synthetic();
    StandardArgs::curry_tree_hash(key.public_key()).to_bytes()
}

/// The standard puzzle hashes of hardened wallet addresses `0..count` of `mnemonic` (public
/// facts: addresses), computed while the account is unlocked.
pub fn hardened_puzzle_hashes(mnemonic: &crate::Mnemonic, count: u32) -> Vec<[u8; 32]> {
    let master = master(&mnemonic.seed());
    (0..count)
        .map(|i| hardened_puzzle_hash(&master, i))
        .collect()
}

/// The standard puzzle hash of a key that is already synthetic (an owner proof's key): the
/// address it signs for. None if the bytes aren't a G1 point.
pub fn p2_of_synthetic(synthetic: &[u8; 48]) -> Option<[u8; 32]> {
    let key = PublicKey::from_bytes(synthetic).ok()?;
    Some(StandardArgs::curry_tree_hash(key).to_bytes())
}

/// A DID's launcher id from `did:chia:1…` (bech32m with the `did:chia:` prefix).
pub fn decode_did(text: &str) -> Option<[u8; 32]> {
    let a = chia_sdk_utils::Address::decode(text.trim()).ok()?;
    (a.prefix == "did:chia:").then(|| a.puzzle_hash.to_bytes())
}
/// `did:chia:1…` for a launcher id.
pub fn encode_did(launcher: &[u8; 32]) -> Option<String> {
    chia_sdk_utils::Address::new((*launcher).into(), "did:chia:".to_owned())
        .encode()
        .ok()
}

/// A puzzle hash as an address (`xch1…` or `txch1…`).
pub fn address(puzzle_hash: [u8; 32], network: Network) -> Result<String> {
    chia_sdk_utils::Address::new(puzzle_hash.into(), network.prefix().to_owned())
        .encode()
        .map_err(|e| Error::new(format!("address: {e}")))
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn from_hex(text: &str) -> Option<Vec<u8>> {
    let text = text.trim();
    if text.len() % 2 != 0 {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Mnemonic;

    const TEST_WORDS: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    #[test]
    fn matches_ronalds_wallet_vectors() {
        // chia-wallet-app/__tests__/keys.harness.ts: the master fingerprint, and the address of
        // the hardened m/12381/8444/2/0 key's synthetic key.
        let m = Mnemonic::parse(TEST_WORDS).unwrap();
        let master = master(&m.seed());
        assert_eq!(facts(&master).fingerprint, 3781984839);
        let sk = derive_hardened(&master, &[12381, 8444, 2, 0]).derive_synthetic();
        let ph = StandardArgs::curry_tree_hash(sk.public_key()).to_bytes();
        assert_eq!(
            address(ph, Network::Mainnet).unwrap(),
            "xch17m20cjztjwevl3rtql5j02rc0mgvmtup24lpq8esctctkaj3g54szqxuju"
        );
    }

    #[test]
    fn unhardened_addresses_need_no_secret() {
        let m = Mnemonic::parse(TEST_WORDS).unwrap();
        let master = master(&m.seed());
        let f = facts(&master);
        let from_secret = [12381, 8444, 2, 0]
            .iter()
            .fold(master.clone(), |k, i| k.derive_unhardened(*i))
            .derive_synthetic();
        assert_eq!(
            wallet_puzzle_hash(&f.master, 0).unwrap(),
            StandardArgs::curry_tree_hash(from_secret.public_key()).to_bytes()
        );
        assert!(
            address(
                wallet_puzzle_hash(&f.master, 0).unwrap(),
                Network::Testnet11
            )
            .unwrap()
            .starts_with("txch1")
        );
    }

    #[test]
    fn hardened_addresses_are_sages_second_branch() {
        // Ronald's vector above is the hardened m/12381'/8444'/2'/0' address.
        let m = Mnemonic::parse(TEST_WORDS).unwrap();
        let hardened = hardened_puzzle_hashes(&m, 2);
        assert_eq!(
            address(hardened[0], Network::Mainnet).unwrap(),
            "xch17m20cjztjwevl3rtql5j02rc0mgvmtup24lpq8esctctkaj3g54szqxuju"
        );
        let f = facts_of(&m);
        let plain = wallet_puzzle_hashes(&f.master, 2).unwrap();
        assert_eq!(plain[1], wallet_puzzle_hash(&f.master, 1).unwrap());
        assert!(hardened.iter().all(|h| !plain.contains(h)));
        assert_ne!(hardened[0], hardened[1]);
    }

    #[test]
    fn the_identity_key_is_its_own_hardened_branch() {
        let m = Mnemonic::parse(TEST_WORDS).unwrap();
        let master = master(&m.seed());
        let f = facts(&master);
        // Stable: the same words give the same identity on every machine.
        assert_eq!(f.identity, facts_of(&m).identity);
        assert_ne!(f.identity.0, f.master);
        let other = IdentityKey(identity_secret(&master, 1).public_key().to_bytes());
        assert_ne!(other, f.identity);
        // Not a key of the wallet branch.
        for i in 0..5 {
            let w = derive_hardened(&master, &[12381, 8444, 2, i]).public_key();
            assert_ne!(w.to_bytes(), f.identity.0);
        }
        assert_eq!(IdentityKey::from_hex(&f.identity.hex()), Some(f.identity));
        assert!(IdentityKey::from_bytes(&[0u8; 48]).is_none());
    }
}
