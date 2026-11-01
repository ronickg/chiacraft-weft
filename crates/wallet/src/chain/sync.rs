//! Coin state for watched puzzle hashes, as Sage syncs (CHIP-0011 light wallet protocol): a
//! paginated `RequestPuzzleState` from the last height and header hash (genesis the first time)
//! that subscribes once finished, then the peer's `CoinStateUpdate` pushes; an update whose fork
//! height is below what we have rolls back coins created or spent above it. Hinted coins (CATs,
//! NFTs, DIDs sent to us) carry an outer puzzle hash that isn't ours, so their spends aren't
//! pushed by the puzzle-hash subscription: they get a coin-id subscription of their own.
//!
//! Ported from `@chia/core` `sync.ts` (`initialSync`, `applyCoinState`, `handleReorg`,
//! `subscribeHinted`).
use super::source::{ChainSource, Refused};
use crate::Result;
use chia_protocol::{Bytes32, Coin, CoinState, CoinStateUpdate};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A coin and when it was created and spent: what the cache keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoinRecord {
    #[serde(with = "hex_bytes32")]
    pub parent: Bytes32,
    #[serde(with = "hex_bytes32")]
    pub puzzle_hash: Bytes32,
    pub amount: u64,
    pub created: Option<u32>,
    pub spent: Option<u32>,
}

impl CoinRecord {
    pub fn coin(&self) -> Coin {
        Coin::new(self.parent, self.puzzle_hash, self.amount)
    }
    pub fn id(&self) -> Bytes32 {
        self.coin().coin_id()
    }
}

impl From<&CoinState> for CoinRecord {
    fn from(s: &CoinState) -> Self {
        Self {
            parent: s.coin.parent_coin_info,
            puzzle_hash: s.coin.puzzle_hash,
            amount: s.coin.amount,
            created: s.created_height,
            spent: s.spent_height,
        }
    }
}

/// The coins of a set of puzzle hashes and how far they are synced.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WalletSync {
    /// The puzzle hashes followed (the first is the primary address).
    #[serde(with = "hex_bytes32_vec")]
    pub puzzle_hashes: Vec<Bytes32>,
    /// Coin id → record.
    #[serde(with = "coin_map")]
    pub coins: BTreeMap<Bytes32, CoinRecord>,
    /// The height and header hash the coins are synced to.
    pub height: Option<u32>,
    #[serde(with = "hex_bytes32_opt")]
    pub header_hash: Option<Bytes32>,
    /// Hinted coins with a coin-id subscription (on the current peer; not kept).
    #[serde(skip)]
    pub subscribed: BTreeSet<Bytes32>,
}

impl WalletSync {
    pub fn new(puzzle_hashes: Vec<Bytes32>) -> Self {
        Self { puzzle_hashes, ..Self::default() }
    }

    /// Whether `puzzle_hash` is one of ours.
    pub fn is_ours(&self, puzzle_hash: &Bytes32) -> bool {
        self.puzzle_hashes.contains(puzzle_hash)
    }

    /// Forget everything learned (a sync from genesis follows).
    pub fn reset(&mut self) {
        self.coins.clear();
        self.height = None;
        self.header_hash = None;
        self.subscribed.clear();
    }

    /// Record a coin's state; true if it is a hinted coin (not at one of our puzzle hashes),
    /// unspent and not yet subscribed by id.
    pub fn apply(&mut self, state: &CoinState) -> bool {
        let record = CoinRecord::from(state);
        let id = state.coin.coin_id();
        self.coins.insert(id, record);
        !self.is_ours(&record.puzzle_hash) && record.spent.is_none() && !self.subscribed.contains(&id)
    }

    /// Roll back to `fork_height`: coins created above it never existed, spends above it are
    /// undone (`handleReorg`).
    pub fn rollback(&mut self, fork_height: u32) {
        self.coins
            .retain(|_, c| c.created.is_none_or(|h| h <= fork_height));
        for c in self.coins.values_mut() {
            if c.spent.is_some_and(|h| h > fork_height) {
                c.spent = None;
            }
        }
        if self.height.is_some_and(|h| h > fork_height) {
            self.height = Some(fork_height);
        }
    }

    /// A pushed update for our subscriptions: returns the hinted coin ids that want a coin-id
    /// subscription.
    pub fn apply_update(&mut self, update: &CoinStateUpdate) -> Vec<Bytes32> {
        if self.height.is_some_and(|h| update.fork_height < h) {
            self.rollback(update.fork_height);
        }
        let mut hinted = Vec::new();
        for s in &update.items {
            if self.apply(s) {
                hinted.push(s.coin.coin_id());
            }
        }
        self.height = Some(update.height);
        self.header_hash = Some(update.peak_hash);
        hinted
    }

    /// A new peak on the subscribed peer with no coin of ours in it: synced up to it.
    pub fn apply_peak(&mut self, height: u32, header_hash: Bytes32) {
        if self.height.is_none_or(|h| height >= h) {
            self.height = Some(height);
            self.header_hash = Some(header_hash);
        }
    }

    /// Catch up from where we are and subscribe (`initialSync`). A peer that no longer has our
    /// last block (a reorg past it) makes us start over from genesis.
    pub async fn catch_up<S: ChainSource>(&mut self, source: &S) -> Result<()> {
        if self.puzzle_hashes.is_empty() {
            return Ok(());
        }
        let mut restarted = false;
        loop {
            let from = self.height.zip(self.header_hash);
            match source
                .puzzle_state(self.puzzle_hashes.clone(), from, true)
                .await?
            {
                Ok(page) => {
                    for s in &page.coin_states {
                        self.apply(s);
                    }
                    self.height = Some(page.height);
                    self.header_hash = Some(page.header_hash);
                    if page.finished {
                        break;
                    }
                }
                Err(Refused::Reorg) if !restarted => {
                    restarted = true;
                    self.reset();
                }
                Err(why) => {
                    return Err(crate::Error::new(format!("puzzle state refused: {why:?}")));
                }
            }
        }
        // Hinted coins we hold: subscribe by id so their spends reach us too.
        let hinted: Vec<Bytes32> = self
            .coins
            .iter()
            .filter(|(id, c)| !self.is_ours(&c.puzzle_hash) && c.spent.is_none() && !self.subscribed.contains(id))
            .map(|(id, _)| *id)
            .collect();
        self.subscribe_hinted(source, hinted).await
    }

    /// Subscribe to hinted coins by id and record their current state (`subscribeHinted`).
    pub async fn subscribe_hinted<S: ChainSource>(&mut self, source: &S, coin_ids: Vec<Bytes32>) -> Result<()> {
        if coin_ids.is_empty() {
            return Ok(());
        }
        for chunk in coin_ids.chunks(1000) {
            let states = source.coin_state(chunk.to_vec(), true).await?;
            self.subscribed.extend(chunk.iter().copied());
            for s in &states {
                self.apply(s);
            }
        }
        Ok(())
    }

    /// Unspent coins at our own puzzle hashes (spendable XCH).
    pub fn unspent_xch(&self) -> Vec<Coin> {
        self.coins
            .values()
            .filter(|c| c.spent.is_none() && c.created.is_some() && self.is_ours(&c.puzzle_hash))
            .map(CoinRecord::coin)
            .collect()
    }

    /// Unspent hinted coins (CATs, NFTs, DIDs sent to us): not spendable as XCH.
    pub fn unspent_hinted(&self) -> Vec<CoinRecord> {
        self.coins
            .values()
            .filter(|c| c.spent.is_none() && c.created.is_some() && !self.is_ours(&c.puzzle_hash))
            .copied()
            .collect()
    }

    /// The XCH balance in mojos.
    pub fn balance(&self) -> u64 {
        self.unspent_xch().iter().map(|c| c.amount).sum()
    }
}

/// Serde helpers: 32-byte values as hex in the cache file.
pub(crate) mod hex_bytes32 {
    use chia_protocol::Bytes32;
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(v: &Bytes32, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&crate::keys::hex(v.as_ref()))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Bytes32, D::Error> {
        let text = String::deserialize(d)?;
        super::super::parse_hex32(&text).map_err(serde::de::Error::custom)
    }
}

pub(crate) mod hex_bytes32_opt {
    use chia_protocol::Bytes32;
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(v: &Option<Bytes32>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(v) => s.serialize_some(&crate::keys::hex(v.as_ref())),
            None => s.serialize_none(),
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Bytes32>, D::Error> {
        Option::<String>::deserialize(d)?
            .map(|t| super::super::parse_hex32(&t).map_err(serde::de::Error::custom))
            .transpose()
    }
}

pub(crate) mod hex_bytes32_vec {
    use chia_protocol::Bytes32;
    use serde::{Deserialize, Deserializer, Serializer, ser::SerializeSeq};
    pub fn serialize<S: Serializer>(v: &[Bytes32], s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(v.len()))?;
        for b in v {
            seq.serialize_element(&crate::keys::hex(b.as_ref()))?;
        }
        seq.end()
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Bytes32>, D::Error> {
        Vec::<String>::deserialize(d)?
            .iter()
            .map(|t| super::super::parse_hex32(t).map_err(serde::de::Error::custom))
            .collect()
    }
}

mod coin_map {
    use super::CoinRecord;
    use chia_protocol::Bytes32;
    use serde::{Deserialize, Deserializer, Serializer};
    use std::collections::BTreeMap;
    pub fn serialize<S: Serializer>(v: &BTreeMap<Bytes32, CoinRecord>, s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(v.values())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<Bytes32, CoinRecord>, D::Error> {
        Ok(Vec::<CoinRecord>::deserialize(d)?
            .into_iter()
            .map(|c| (c.id(), c))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(ph: u8, amount: u64, created: u32, spent: Option<u32>) -> CoinState {
        CoinState::new(Coin::new(Bytes32::new([amount as u8; 32]), Bytes32::new([ph; 32]), amount), spent, Some(created))
    }

    #[test]
    fn updates_reorgs_and_hinted_coins() {
        let mut s = WalletSync::new(vec![Bytes32::new([1; 32])]);
        let a = state(1, 10, 5, None);
        let b = state(1, 20, 7, None);
        let hinted = state(9, 1, 8, None);
        let up = CoinStateUpdate::new(8, 8, Bytes32::new([8; 32]), vec![a, b, hinted]);
        assert_eq!(s.apply_update(&up), vec![hinted.coin.coin_id()]);
        assert_eq!(s.balance(), 30);
        assert_eq!(s.unspent_hinted().len(), 1);
        // a spent at 9, then a reorg back to 6: b (created at 7) and the spend vanish.
        let spent = state(1, 10, 5, Some(9));
        s.apply_update(&CoinStateUpdate::new(9, 9, Bytes32::new([9; 32]), vec![spent]));
        assert_eq!(s.balance(), 20);
        s.apply_update(&CoinStateUpdate::new(10, 6, Bytes32::new([10; 32]), vec![]));
        assert_eq!(s.balance(), 10);
        assert_eq!(s.coins.get(&a.coin.coin_id()).unwrap().spent, None);
        assert_eq!(s.height, Some(10));
        // Round trip through the cache's JSON.
        let json = serde_json::to_string(&s).unwrap();
        let back: WalletSync = serde_json::from_str(&json).unwrap();
        assert_eq!(back.coins, s.coins);
        assert_eq!(back.header_hash, s.header_hash);
    }
}
