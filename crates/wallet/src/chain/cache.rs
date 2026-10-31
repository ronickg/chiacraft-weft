//! What the reader learned, in one JSON file (`chain-cache.json` in a server's folder;
//! identity-research.md 9.3), so it survives restarts and answers while no peer is reachable
//! (3.7): the wallet's coins and sync height, resolved DIDs, watched coins, the offers made and
//! not yet settled, and peers that answered. Only public chain facts: no key, no secret.
use super::{
    ChainNetwork, MadeOffer,
    did::DidRecord,
    sync::{CoinRecord, WalletSync, hex_bytes32},
};
use crate::{Error, Result};
use chia_protocol::Bytes32;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

/// The cache format's version: another version is ignored (and rebuilt from the chain).
pub const VERSION: u32 = 1;

/// A coin the reader watches until it is spent (an offered coin, for instance).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchedCoin {
    #[serde(with = "hex_bytes32")]
    pub coin_id: Bytes32,
    /// Its state when last heard (None: not on chain, or not asked yet).
    pub state: Option<CoinRecord>,
}

impl WatchedCoin {
    pub fn spent(&self) -> Option<u32> {
        self.state.and_then(|s| s.spent)
    }
    /// Blocks on top of the spend (1 = in the peak block), given the peak.
    pub fn confirmations(&self, peak: Option<u32>) -> u32 {
        match (self.spent(), peak) {
            (Some(at), Some(peak)) if peak >= at => peak - at + 1,
            _ => 0,
        }
    }
}

/// The reader's cache.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainCache {
    pub version: u32,
    pub network: ChainNetwork,
    /// The highest peak a peer reported.
    pub peak: Option<u32>,
    /// The wallet followed (the server's own account first).
    pub wallet: WalletSync,
    /// Resolved DIDs, by launcher id (hex).
    pub dids: BTreeMap<String, DidRecord>,
    /// Watched coins, by coin id (hex).
    pub watched: BTreeMap<String, WatchedCoin>,
    /// Offers made and not yet taken or expired: their coins aren't spent for anything else.
    pub offers: Vec<MadeOffer>,
    /// Coins of ours a pushed spend is spending (hex → unix seconds when pushed), kept out of
    /// coin selection until the chain shows them spent or a while passes.
    pub pending: BTreeMap<String, u64>,
    /// Peers that answered lately (`host:port`), tried first next time.
    pub peers: Vec<String>,
}

impl ChainCache {
    pub fn new(network: ChainNetwork, puzzle_hashes: Vec<Bytes32>) -> Self {
        Self {
            version: VERSION,
            network,
            peak: None,
            wallet: WalletSync::new(puzzle_hashes),
            dids: BTreeMap::new(),
            watched: BTreeMap::new(),
            offers: Vec::new(),
            pending: BTreeMap::new(),
            peers: Vec::new(),
        }
    }

    /// The cache at `path` for `network` and these puzzle hashes; a missing, damaged or other
    /// network's file gives a fresh cache (the chain rebuilds it). A different wallet keeps the
    /// DIDs and watches but syncs its coins again.
    pub fn load(path: &Path, network: ChainNetwork, puzzle_hashes: Vec<Bytes32>) -> Self {
        let fresh = || Self::new(network, puzzle_hashes.clone());
        let Ok(text) = std::fs::read_to_string(path) else {
            return fresh();
        };
        let Ok(mut cache) = serde_json::from_str::<Self>(&text) else {
            log::warn!("chain cache {} unreadable: starting over", path.display());
            return fresh();
        };
        if cache.version != VERSION || cache.network != network {
            return fresh();
        }
        if cache.wallet.puzzle_hashes != puzzle_hashes {
            cache.wallet = WalletSync::new(puzzle_hashes);
            cache.offers.clear();
            cache.pending.clear();
        }
        cache
    }

    /// Write the cache (a temporary file, then a rename).
    pub fn save(&self, path: &Path) -> Result<()> {
        let text = serde_json::to_string_pretty(self).map_err(|e| Error::new(e.to_string()))?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, text)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Coins locked by open offers and pending spends.
    pub fn locked(&self) -> Vec<Bytes32> {
        self.offers
            .iter()
            .flat_map(|o| o.offered_coins.iter())
            .chain(self.pending.keys())
            .filter_map(|h| super::parse_hex32(h).ok())
            .collect()
    }

    /// Spendable XCH: unspent coins of the wallet that no offer or pending spend holds.
    pub fn spendable(&self) -> Vec<chia_protocol::Coin> {
        let locked = self.locked();
        self.wallet
            .unspent_xch()
            .into_iter()
            .filter(|c| !locked.contains(&c.coin_id()))
            .collect()
    }

    /// Forget pending spends the chain has settled, or older than `max_age` seconds.
    pub fn expire_pending(&mut self, now: u64, max_age: u64) {
        let wallet = &self.wallet;
        self.pending.retain(|id, at| {
            let spent = super::parse_hex32(id)
                .ok()
                .and_then(|id| wallet.coins.get(&id))
                .is_some_and(|c| c.spent.is_some());
            !spent && now.saturating_sub(*at) < max_age
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_loads_and_starts_over_on_another_network() {
        let dir = std::env::temp_dir().join(format!("hz-chain-cache-{}", std::process::id()));
        let path = dir.join("chain-cache.json");
        let ph = vec![Bytes32::new([3; 32])];
        let mut c = ChainCache::new(ChainNetwork::Testnet11, ph.clone());
        c.peak = Some(42);
        c.peers.push("127.0.0.1:58444".into());
        c.save(&path).unwrap();
        let back = ChainCache::load(&path, ChainNetwork::Testnet11, ph.clone());
        assert_eq!(back.peak, Some(42));
        assert_eq!(back.peers, c.peers);
        let other = ChainCache::load(&path, ChainNetwork::Mainnet, ph);
        assert_eq!(other.peak, None);
        let _ = std::fs::remove_dir_all(dir);
    }
}
