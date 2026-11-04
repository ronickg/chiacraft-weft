//! The chain reader (identity-research.md 1.5, 3.4, 3.6, 3.7; plan row I6), behind the `chain`
//! feature: a light wallet on Chia's peer protocol, as Sage and Ronald's wallet do it, with no
//! full node and nothing of ours in between.
//!
//! - [`source`]: what the reader asks of the chain ([`ChainSource`]), answered by a full-node peer
//!   over the light wallet protocol (chia-sdk-client); a coinset or own-node source can implement
//!   the same trait later.
//! - [`sync`]: coin state for watched puzzle hashes (the server's own wallet first): a paginated
//!   catch-up with a subscription, then the peer's pushed updates, reorgs rolled back.
//! - [`did`]: a DID's current coin, its owner (the p2 puzzle hash inside it) and its history.
//! - [`offer`]: offers parsed and checked, taken and made (the server offering a 1-mojo coin or an
//!   NFT for a price, section 3.6's variant A), signed with the local wallet's keys.
//! - [`cache`]: what the reader learned, kept in one JSON file so it survives restarts and answers
//!   while no peer is reachable (3.7).
//! - [`sponsor`]: DIDs launched by a sponsor straight to players' addresses (I9).
//! - [`world`]: world DIDs (phase 4, slice 7): the world's and the owner's DID metadata, and
//!   whether a world and its owner DID name each other.
//! - [`anchor`]: the achievements' daily anchor (I10): a day's Merkle root put on chain as the
//!   memo of one of the server's own coins, and read back.
//! - [`tool`]: `chiacraft-wallet-test`, the live test tool an operator runs step by step (I12).
//! - [`reader`]: the background reader: its own tokio runtime, a pool of peers found through the
//!   DNS introducers, the subscription, coin watches, and a blocking API for the game.
//!
//! **testnet11 first** (PROJECT.md's wallet rules): [`ChainNetwork::Testnet11`] is the default.
//! Mainnet (I12) is an explicit opt-in: [`ChainReader::start`] refuses it unless the config
//! carries a [`crate::spending::MainnetConsent`], and every spend is held to the config's
//! [`crate::spending::SpendCaps`] (small on mainnet).
//!
//! Secrets: the reader itself needs none (it reads by puzzle hash). Signing a take or an offer
//! uses [`WalletKeys`], whose secret halves live only in [`crate::Secret`]; nothing here logs a
//! key, a coin spend's solution or a signature.
//!
//! Ported from Ronald's wallet (`ronickg/chia`, `packages/core`: `sync.ts`, `did.ts`, `offer.ts`)
//! onto chia-wallet-sdk 0.37.1's crates (chia-sdk-client, chia-sdk-driver, chia-sdk-signer); the
//! TLS connector follows chia-sdk-client's own. Both Apache-2.0 (see NOTICE).
pub mod anchor;
pub mod cache;
pub mod did;
pub mod holdings;
pub mod offer;
pub mod reader;
#[cfg(feature = "sim")]
pub mod sim;
pub mod source;
pub mod sponsor;
pub mod sync;
pub mod tls;
pub mod tool;
pub mod wallet;
pub mod world;

pub use cache::{ChainCache, WatchedCoin};
pub use did::{DidRecord, DidStep};
pub use offer::{MadeOffer, MakeOffer, OfferGive, OfferSummary};
pub use reader::{ChainConfig, ChainReader, ChainStatus, CreatedDid, DidLookup, OfferState, Phase, TakeOutcome, UpdatedDid};
pub use source::{ChainSource, PeerSource, PushOutcome};
pub use sponsor::{EveAsk, EveDid, SponsorBundle};
pub use sync::{CoinRecord, WalletSync};
pub use wallet::WalletKeys;
pub use world::{DidMetadata, OwnerMeta, WorldLink, WorldMeta};

pub use chia_protocol::{Bytes32, Coin, CoinState, SpendBundle};

use crate::{Error, Result};
use chia_sdk_types::{MAINNET_CONSTANTS, TESTNET11_CONSTANTS};

/// The Chia network a reader follows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChainNetwork {
    Testnet11,
    Mainnet,
}

impl ChainNetwork {
    /// `testnet11` or `mainnet` (the words of the `chain` setting).
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            "testnet11" => Some(Self::Testnet11),
            "mainnet" => Some(Self::Mainnet),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Testnet11 => "testnet11",
            Self::Mainnet => "mainnet",
        }
    }
    /// The network id a full node's handshake must carry.
    pub fn network_id(self) -> &'static str {
        self.name()
    }
    /// The genesis challenge: the header hash a sync from scratch starts from.
    pub fn genesis_challenge(self) -> Bytes32 {
        match self {
            Self::Testnet11 => TESTNET11_CONSTANTS.genesis_challenge,
            Self::Mainnet => MAINNET_CONSTANTS.genesis_challenge,
        }
    }
    /// What `AGG_SIG_ME` signatures commit to on this network.
    pub fn agg_sig_me(self) -> Bytes32 {
        match self {
            Self::Testnet11 => TESTNET11_CONSTANTS.agg_sig_me_additional_data,
            Self::Mainnet => MAINNET_CONSTANTS.agg_sig_me_additional_data,
        }
    }
    /// The DNS introducers and the full nodes' port (chia-sdk-client's lists, as Sage).
    pub fn introducers(self) -> chia_sdk_client::Network {
        match self {
            Self::Testnet11 => chia_sdk_client::Network::default_testnet11(),
            Self::Mainnet => chia_sdk_client::Network::default_mainnet(),
        }
    }
    /// The address network (`txch1…` or `xch1…`).
    pub fn address_network(self) -> crate::Network {
        match self {
            Self::Testnet11 => crate::Network::Testnet11,
            Self::Mainnet => crate::Network::Mainnet,
        }
    }
}

impl From<crate::Network> for ChainNetwork {
    fn from(n: crate::Network) -> Self {
        match n {
            crate::Network::Testnet11 => Self::Testnet11,
            crate::Network::Mainnet => Self::Mainnet,
        }
    }
}

impl std::fmt::Display for ChainNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// The DID prefix: `did:chia:1…` is the bech32m of the launcher id with this human part.
pub const DID_PREFIX: &str = "did:chia:";

/// A DID's launcher id from `did:chia:1…` or 64 hex digits.
pub fn parse_did(text: &str) -> Result<Bytes32> {
    let text = text.trim();
    if let Some(bytes) = crate::keys::from_hex(text.strip_prefix("0x").unwrap_or(text))
        && let Ok(id) = <[u8; 32]>::try_from(bytes.as_slice())
    {
        return Ok(id.into());
    }
    let address = chia_sdk_utils::Address::decode(text).map_err(|_| Error::new(format!("not a DID: {text:?}")))?;
    if address.prefix != DID_PREFIX {
        return Err(Error::new(format!("not a DID: {text:?}")));
    }
    Ok(address.puzzle_hash)
}

/// `did:chia:1…` for a launcher id.
pub fn did_string(launcher_id: Bytes32) -> String {
    chia_sdk_utils::Address::new(launcher_id, DID_PREFIX.to_owned())
        .encode()
        .unwrap_or_else(|_| hex32(launcher_id))
}

/// 64 hex digits.
pub fn hex32(bytes: Bytes32) -> String {
    crate::keys::hex(bytes.as_ref())
}

/// 32 bytes from 64 hex digits (an optional `0x`).
pub fn parse_hex32(text: &str) -> Result<Bytes32> {
    let text = text.trim();
    crate::keys::from_hex(text.strip_prefix("0x").unwrap_or(text))
        .and_then(|b| <[u8; 32]>::try_from(b.as_slice()).ok())
        .map(Bytes32::from)
        .ok_or_else(|| Error::new(format!("not 32 bytes of hex: {text:?}")))
}

/// An error of a dependency, with what was being done. Their texts name coins, puzzles and
/// peers, never a key.
pub(crate) fn failed<E: std::fmt::Display>(what: &'static str) -> impl Fn(E) -> Error {
    move |e| Error::new(format!("{what}: {e}"))
}

/// A puzzle hash as an address on `network`.
pub fn address(puzzle_hash: Bytes32, network: ChainNetwork) -> String {
    crate::keys::address(puzzle_hash.to_bytes(), network.address_network()).unwrap_or_else(|_| hex32(puzzle_hash))
}

/// The standard puzzle hash of an address (`xch1…`/`txch1…`) or 64 hex digits.
pub fn parse_address(text: &str) -> Result<Bytes32> {
    if let Ok(h) = parse_hex32(text) {
        return Ok(h);
    }
    chia_sdk_utils::Address::decode(text.trim())
        .map(|a| a.puzzle_hash)
        .map_err(|_| Error::new(format!("not an address: {text:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn did_strings_round_trip() {
        let id = Bytes32::new([7; 32]);
        let s = did_string(id);
        assert!(s.starts_with("did:chia:1"), "{s}");
        assert_eq!(parse_did(&s).unwrap(), id);
        assert_eq!(parse_did(&hex32(id)).unwrap(), id);
        assert!(parse_did("txch1qqqq").is_err());
        let addr = address(id, ChainNetwork::Testnet11);
        assert!(parse_did(&addr).is_err());
        assert_eq!(parse_address(&addr).unwrap(), id);
    }

    #[test]
    fn networks() {
        assert_eq!(ChainNetwork::parse("testnet11"), Some(ChainNetwork::Testnet11));
        assert_eq!(ChainNetwork::parse("off"), None);
        assert_ne!(ChainNetwork::Testnet11.genesis_challenge(), ChainNetwork::Mainnet.genesis_challenge());
        assert_eq!(ChainNetwork::Testnet11.introducers().default_port, 58444);
        // Mainnet: Chia's own introducers and port 8444, the network id `mainnet`, xch1 addresses.
        assert_eq!(ChainNetwork::Mainnet.introducers().default_port, 8444);
        assert_eq!(ChainNetwork::Mainnet.network_id(), "mainnet");
        assert!(address(Bytes32::new([1; 32]), ChainNetwork::Mainnet).starts_with("xch1"));
        assert_eq!(ChainNetwork::from(crate::Network::Mainnet), ChainNetwork::Mainnet);
    }
}
