//! The local wallet's spending keys, for taking and making offers: the standard puzzle's
//! synthetic keys of the first wallet addresses (unhardened `m/12381/8444/2/i`, as
//! [`crate::keys::wallet_puzzle_hash`] and Sage). Their secret halves live only in
//! [`Secret`]; [`WalletKeys`] formats as a count and the first address, and signs only the coin
//! spends the reader built itself (every signature a spend asks for must be one of these keys,
//! checked against the puzzle reveal, chia-sdk-signer's `RequiredSignature`).
//!
//! Ported from `@chia/core` `wallet.ts` (`deriveWallet`'s derivations) and `nft.ts`
//! (`signSpends`).
use super::{ChainNetwork, failed};
use crate::{Error, Mnemonic, Result, Secret};
use chia_bls::{DerivableKey, PublicKey, SecretKey, Signature};
use chia_protocol::{Bytes32, CoinSpend};
use chia_puzzle_types::{DeriveSynthetic, standard::StandardArgs};
use chia_sdk_signer::{AggSigConstants, RequiredSignature};
use clvmr::Allocator;
use indexmap::IndexMap;

/// How many wallet addresses the reader follows by default (Sage looks ahead further; a game
/// account uses its first few).
pub const DEFAULT_ADDRESSES: u32 = 20;

struct Key {
    puzzle_hash: Bytes32,
    public: PublicKey,
    secret: Secret<[u8; 32]>,
}

/// A wallet's spending keys on one network.
pub struct WalletKeys {
    network: ChainNetwork,
    keys: Vec<Key>,
}

impl std::fmt::Debug for WalletKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WalletKeys({} addresses from {}, [redacted])",
            self.keys.len(),
            self.keys
                .first()
                .map_or_else(String::new, |k| super::address(k.puzzle_hash, self.network))
        )
    }
}

impl WalletKeys {
    /// The first `count` addresses of an account.
    pub fn from_mnemonic(mnemonic: &Mnemonic, network: ChainNetwork, count: u32) -> Self {
        let master = crate::keys::master(&mnemonic.seed());
        let wallet = [
            crate::keys::CHIA_ROOT[0],
            crate::keys::CHIA_ROOT[1],
            crate::keys::WALLET_PURPOSE,
        ]
        .iter()
        .fold(master, |k, i| k.derive_unhardened(*i));
        let keys = (0..count.max(1))
            .map(|i| {
                let synthetic = wallet.derive_unhardened(i).derive_synthetic();
                let public = synthetic.public_key();
                Key {
                    puzzle_hash: StandardArgs::curry_tree_hash(public).into(),
                    public,
                    secret: Secret::new(synthetic.to_bytes()),
                }
            })
            .collect();
        Self { network, keys }
    }

    /// The first `count` unhardened addresses, then the first `hardened` hardened ones
    /// (`m/12381'/8444'/2'/i'`, as Sage also derives: identity row I8), so coins and DIDs that
    /// words imported from Sage hold at either kind of address can be spent. The primary
    /// (unhardened index 0) stays first.
    pub fn with_hardened(
        mnemonic: &Mnemonic,
        network: ChainNetwork,
        count: u32,
        hardened: u32,
    ) -> Self {
        let mut out = Self::from_mnemonic(mnemonic, network, count);
        let master = crate::keys::master(&mnemonic.seed());
        out.keys.extend((0..hardened).map(|i| {
            let synthetic = crate::keys::derive_hardened(
                &master,
                &[
                    crate::keys::CHIA_ROOT[0],
                    crate::keys::CHIA_ROOT[1],
                    crate::keys::WALLET_PURPOSE,
                    i,
                ],
            )
            .derive_synthetic();
            let public = synthetic.public_key();
            Key {
                puzzle_hash: StandardArgs::curry_tree_hash(public).into(),
                public,
                secret: Secret::new(synthetic.to_bytes()),
            }
        }));
        out
    }

    pub fn network(&self) -> ChainNetwork {
        self.network
    }

    /// The puzzle hashes of the addresses, the primary (index 0) first.
    pub fn puzzle_hashes(&self) -> Vec<Bytes32> {
        self.keys.iter().map(|k| k.puzzle_hash).collect()
    }

    /// The primary address's puzzle hash: change and received assets go there.
    pub fn primary(&self) -> Bytes32 {
        self.keys[0].puzzle_hash
    }

    /// Puzzle hash → synthetic public key, for chia-sdk-driver's `finish_with_keys`.
    pub fn synthetic_keys(&self) -> IndexMap<Bytes32, PublicKey> {
        self.keys
            .iter()
            .map(|k| (k.puzzle_hash, k.public))
            .collect()
    }

    /// The aggregate signature `coin_spends` need. Every signature they ask for must be one of
    /// these keys: a spend asking for another key's signature is refused (it isn't ours to
    /// build). Nothing about the keys or the messages is logged.
    pub fn sign(&self, coin_spends: &[CoinSpend]) -> Result<Signature> {
        let mut allocator = Allocator::new();
        let required = RequiredSignature::from_coin_spends(
            &mut allocator,
            coin_spends,
            &AggSigConstants::new(self.network.agg_sig_me()),
        )
        .map_err(failed("signatures needed"))?;
        let mut signature = Signature::default();
        for r in required {
            let RequiredSignature::Bls(r) = r else {
                return Err(Error::new("a spend asks for a secp signature"));
            };
            let key = self
                .keys
                .iter()
                .find(|k| k.public == r.public_key)
                .ok_or_else(|| Error::new("a spend asks for a key this wallet doesn't have"))?;
            let secret = SecretKey::from_bytes(key.secret.expose())
                .map_err(|_| Error::new("wallet key damaged in memory"))?;
            signature += &chia_bls::sign(&secret, r.message());
        }
        Ok(signature)
    }
}

/// The standard puzzle hash of a wallet key (unhardened or hardened `m/12381/8444/2/i`): its
/// synthetic key curried into `p2_delegated_puzzle_or_hidden_puzzle` with the default hidden
/// puzzle. The owner (p2 puzzle hash) a DID or coin held at that key's address has.
pub fn standard_puzzle_hash(public: &PublicKey) -> Bytes32 {
    p2_of_synthetic(&public.derive_synthetic())
}

/// The standard puzzle hash of a key that is already synthetic. Sage's `signMessageByAddress`
/// (sage `endpoints/wallet_connect.rs`) signs with the address's **synthetic** secret key, so a
/// DID proof made that way (I8) is checked as `record.owned_by(p2_of_synthetic(&signer))`.
pub fn p2_of_synthetic(synthetic: &PublicKey) -> Bytes32 {
    StandardArgs::curry_tree_hash(*synthetic).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_match_the_public_derivation_and_stay_redacted() {
        let m = Mnemonic::generate().unwrap();
        let keys = WalletKeys::from_mnemonic(&m, ChainNetwork::Testnet11, 3);
        let facts = crate::keys::facts_of(&m);
        for (i, ph) in keys.puzzle_hashes().iter().enumerate() {
            assert_eq!(
                ph.to_bytes(),
                crate::keys::wallet_puzzle_hash(&facts.master, i as u32).unwrap()
            );
        }
        let shown = format!("{keys:?}");
        assert!(
            shown.contains("txch1") && shown.contains("[redacted]"),
            "{shown}"
        );
        for k in &keys.keys {
            assert!(!shown.contains(&crate::keys::hex(k.secret.expose())));
        }
        assert!(keys.sign(&[]).is_ok());
        // A wallet key and its synthetic key name the same address.
        let wallet_key = [12381, 8444, 2, 0]
            .iter()
            .fold(crate::keys::master(&m.seed()), |k, i| {
                k.derive_unhardened(*i)
            })
            .public_key();
        assert_eq!(standard_puzzle_hash(&wallet_key), keys.primary());
        assert_eq!(p2_of_synthetic(&keys.keys[0].public), keys.primary());
    }

    #[test]
    fn hardened_keys_follow_the_unhardened_ones() {
        let m = Mnemonic::generate().unwrap();
        let both = WalletKeys::with_hardened(&m, ChainNetwork::Testnet11, 2, 3);
        let plain = WalletKeys::from_mnemonic(&m, ChainNetwork::Testnet11, 2);
        let hashes = both.puzzle_hashes();
        assert_eq!(hashes.len(), 5);
        assert_eq!(&hashes[..2], plain.puzzle_hashes().as_slice());
        assert_eq!(both.primary(), plain.primary());
        let hardened = crate::keys::hardened_puzzle_hashes(&m, 3);
        for (h, want) in hashes[2..].iter().zip(&hardened) {
            assert_eq!(h.to_bytes(), *want);
        }
        assert!(format!("{both:?}").contains("5 addresses"));
    }
}
