//! What the reader asks of the chain (identity-research.md 3.4): coin state by puzzle hash and by
//! coin id (each optionally a subscription), a spent coin's puzzle and solution, a coin's
//! children, and pushing a spend bundle. [`PeerSource`] answers it from a full-node peer over the
//! light wallet protocol; a coinset.org or own-node source would implement the same trait (not
//! built: section 1.5's fallback for networks that block port 8444).
use super::{ChainNetwork, failed};
use crate::{Error, Result};
use chia_protocol::{
    Bytes32, CoinState, CoinStateFilters, Program, RejectStateReason, SpendBundle,
};
use chia_sdk_client::Peer;
use std::future::Future;

/// One page of coin states for a set of puzzle hashes.
#[derive(Clone, Debug)]
pub struct PuzzleStatePage {
    pub coin_states: Vec<CoinState>,
    /// The height and header hash the page reaches: where the next page starts.
    pub height: u32,
    pub header_hash: Bytes32,
    pub finished: bool,
}

/// Why a state request was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The block the request started from is no longer on the peer's chain: start over.
    Reorg,
    /// The peer won't take more subscriptions from us.
    TooManySubscriptions,
}

/// What a peer said about a pushed spend bundle (`TransactionAck`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PushOutcome {
    /// In the mempool.
    Accepted {
        txid: Bytes32,
    },
    /// Waiting (for example on a coin the mempool doesn't have yet).
    Pending {
        txid: Bytes32,
    },
    Rejected {
        txid: Bytes32,
        error: String,
    },
}

/// The chain as the reader sees it.
pub trait ChainSource: Send + Sync {
    /// Coin states of `puzzle_hashes` since `from` (height and header hash; `None`: genesis),
    /// spent and unspent, hinted coins included; `subscribe` asks for updates once finished.
    fn puzzle_state(
        &self,
        puzzle_hashes: Vec<Bytes32>,
        from: Option<(u32, Bytes32)>,
        subscribe: bool,
    ) -> impl Future<Output = Result<std::result::Result<PuzzleStatePage, Refused>>> + Send;
    /// Current states of `coin_ids` (unknown coins are left out); `subscribe` asks for updates.
    fn coin_state(
        &self,
        coin_ids: Vec<Bytes32>,
        subscribe: bool,
    ) -> impl Future<Output = Result<Vec<CoinState>>> + Send;
    /// The puzzle reveal and solution a coin was spent with at `height`.
    fn puzzle_and_solution(
        &self,
        coin_id: Bytes32,
        height: u32,
    ) -> impl Future<Output = Result<(Program, Program)>> + Send;
    /// The coins a coin's spend created.
    fn children(&self, coin_id: Bytes32) -> impl Future<Output = Result<Vec<CoinState>>> + Send;
    /// Send a spend bundle to the mempool.
    fn push(&self, bundle: SpendBundle) -> impl Future<Output = Result<PushOutcome>> + Send;
}

/// A full-node peer (chia-sdk-client's [`Peer`]) on a network.
#[derive(Clone, Debug)]
pub struct PeerSource {
    pub peer: Peer,
    pub network: ChainNetwork,
}

impl PeerSource {
    pub fn new(peer: Peer, network: ChainNetwork) -> Self {
        Self { peer, network }
    }
}

fn refused(reason: RejectStateReason) -> Refused {
    match reason {
        RejectStateReason::Reorg => Refused::Reorg,
        RejectStateReason::ExceededSubscriptionLimit => Refused::TooManySubscriptions,
    }
}

impl ChainSource for PeerSource {
    async fn puzzle_state(
        &self,
        puzzle_hashes: Vec<Bytes32>,
        from: Option<(u32, Bytes32)>,
        subscribe: bool,
    ) -> Result<std::result::Result<PuzzleStatePage, Refused>> {
        let (height, header) = match from {
            Some((h, hash)) => (Some(h), hash),
            None => (None, self.network.genesis_challenge()),
        };
        let reply = self
            .peer
            .request_puzzle_state(
                puzzle_hashes,
                height,
                header,
                CoinStateFilters::new(true, true, true, 0),
                subscribe,
            )
            .await
            .map_err(failed("puzzle state"))?;
        Ok(match reply {
            Ok(r) => Ok(PuzzleStatePage {
                coin_states: r.coin_states,
                height: r.height,
                header_hash: r.header_hash,
                finished: r.is_finished,
            }),
            Err(rejected) => Err(refused(rejected.reason)),
        })
    }

    async fn coin_state(&self, coin_ids: Vec<Bytes32>, subscribe: bool) -> Result<Vec<CoinState>> {
        let reply = self
            .peer
            .request_coin_state(coin_ids, None, self.network.genesis_challenge(), subscribe)
            .await
            .map_err(failed("coin state"))?;
        match reply {
            Ok(r) => Ok(r.coin_states),
            Err(rejected) => Err(Error::new(format!(
                "coin state refused: {:?}",
                refused(rejected.reason)
            ))),
        }
    }

    async fn puzzle_and_solution(
        &self,
        coin_id: Bytes32,
        height: u32,
    ) -> Result<(Program, Program)> {
        let reply = self
            .peer
            .request_puzzle_and_solution(coin_id, height)
            .await
            .map_err(failed("puzzle and solution"))?;
        match reply {
            Ok(r) => Ok((r.puzzle, r.solution)),
            Err(_) => Err(Error::new(format!(
                "the peer has no spend of coin {} at height {height}",
                super::hex32(coin_id)
            ))),
        }
    }

    async fn children(&self, coin_id: Bytes32) -> Result<Vec<CoinState>> {
        Ok(self
            .peer
            .request_children(coin_id)
            .await
            .map_err(failed("children"))?
            .coin_states)
    }

    async fn push(&self, bundle: SpendBundle) -> Result<PushOutcome> {
        let ack = self
            .peer
            .send_transaction(bundle)
            .await
            .map_err(failed("push"))?;
        Ok(match (ack.status, ack.error) {
            (1, _) => PushOutcome::Accepted { txid: ack.txid },
            (2, _) => PushOutcome::Pending { txid: ack.txid },
            (_, error) => PushOutcome::Rejected {
                txid: ack.txid,
                error: error.unwrap_or_else(|| "rejected".into()),
            },
        })
    }
}
