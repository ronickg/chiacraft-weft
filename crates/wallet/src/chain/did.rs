//! Reading a DID from the chain (identity-research.md 3.4's "DID check"): follow the singleton
//! from its launcher through each spend to the current coin, and read the owner from inside it.
//!
//! - A spent DID coin's spend reveals its puzzle: chia-sdk-driver's `Did::parse` reads the DID
//!   layer (launcher id, recovery list, metadata) and the **p2 puzzle hash** (the owner's inner
//!   puzzle: the standard puzzle of the owner's synthetic key for a wallet-held DID).
//! - The child is the odd-amount coin that spend created; `Did::parse_child` reads it from the
//!   parent's spend and the hint (the new owner's p2 puzzle hash), and is trusted only if it
//!   hashes to the child coin's puzzle hash.
//! - The current (unspent) coin's owner is the hint its parent's spend gave, so checked; an eve
//!   DID (never spent since launch: a sponsored one, I9) has no such hint and its owner shows as
//!   unknown, but [`DidRecord::owned_by`] still recognises the address it was launched to: the eve
//!   puzzle hash commits to it.
//!
//! A [`DidRecord`] is cached; a later resolve continues from its last coin instead of walking the
//! whole history again. Ported from `@chia/core` `did.ts` (`fetchDid`, `resolveDid`).
use super::{failed, source::ChainSource, sync::hex_bytes32, sync::hex_bytes32_opt};
use crate::{Error, Result};
use chia_protocol::{Bytes32, Coin, CoinState, Program};
use chia_puzzles::SINGLETON_LAUNCHER_HASH;
use chia_sdk_driver::{Did, Puzzle, SingletonInfo};
use clvm_traits::ToClvm;
use clvmr::{Allocator, NodePtr};
use serde::{Deserialize, Serialize};

/// The most spends one resolve follows (a DID spent more often than this resumes next time).
pub const MAX_STEPS: usize = 2_000;

/// One coin in a DID's life.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DidStep {
    #[serde(with = "hex_bytes32")]
    pub coin_id: Bytes32,
    #[serde(with = "hex_bytes32")]
    pub parent: Bytes32,
    #[serde(with = "hex_bytes32")]
    pub puzzle_hash: Bytes32,
    pub amount: u64,
    pub created: Option<u32>,
    pub spent: Option<u32>,
    /// The owner's p2 puzzle hash while this coin existed (None: an eve coin not yet spent).
    #[serde(with = "hex_bytes32_opt")]
    pub owner: Option<Bytes32>,
}

impl DidStep {
    fn of(state: &CoinState, owner: Option<Bytes32>) -> Self {
        Self {
            coin_id: state.coin.coin_id(),
            parent: state.coin.parent_coin_info,
            puzzle_hash: state.coin.puzzle_hash,
            amount: state.coin.amount,
            created: state.created_height,
            spent: state.spent_height,
            owner,
        }
    }
    pub fn coin(&self) -> Coin {
        Coin::new(self.parent, self.puzzle_hash, self.amount)
    }
}

/// What the chain says about a DID.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DidRecord {
    #[serde(with = "hex_bytes32")]
    pub launcher_id: Bytes32,
    /// Every coin from the eve coin to the current one, in order.
    pub history: Vec<DidStep>,
    /// The DID was melted (its last spend made no singleton child).
    pub melted: bool,
    /// The peak height when this was read.
    pub checked_height: u32,
}

impl DidRecord {
    /// `did:chia:1…`.
    pub fn did(&self) -> String {
        super::did_string(self.launcher_id)
    }
    /// The current coin (the last step; spent if the DID was melted).
    pub fn current(&self) -> Option<&DidStep> {
        self.history.last()
    }
    /// The current owner's p2 puzzle hash.
    pub fn owner(&self) -> Option<Bytes32> {
        if self.melted {
            return None;
        }
        self.current().and_then(|s| s.owner)
    }
    /// Whether the DID is now held by `p2_puzzle_hash` (for a wallet address: its standard puzzle
    /// hash; [`super::wallet::standard_puzzle_hash`] of a key).
    ///
    /// An eve DID (never spent since its launch, so no spend names its owner: a sponsored DID,
    /// identity row I9) counts as held by `p2_puzzle_hash` when its puzzle hash is the eve puzzle
    /// of a DID launched to it with no recovery list, one verification and no metadata
    /// ([`Self::eve_owned_by`]): the puzzle hash commits to the owner, so this is as sound as
    /// reading it from a spend.
    pub fn owned_by(&self, p2_puzzle_hash: Bytes32) -> bool {
        self.owner() == Some(p2_puzzle_hash) || self.eve_owned_by(p2_puzzle_hash)
    }
    /// Whether this is an unspent eve DID launched to `p2_puzzle_hash` (no recovery list, one
    /// verification, no metadata: how a sponsor launches one).
    pub fn eve_owned_by(&self, p2_puzzle_hash: Bytes32) -> bool {
        match self.history.as_slice() {
            [eve] if !self.melted && eve.spent.is_none() && eve.owner.is_none() => {
                eve.puzzle_hash == super::sponsor::eve_puzzle_hash(self.launcher_id, p2_puzzle_hash)
            }
            _ => false,
        }
    }
    /// Whether the DID is still in its eve state (no owner shown on chain yet).
    pub fn is_eve(&self) -> bool {
        matches!(self.history.as_slice(), [eve] if eve.owner.is_none() && eve.spent.is_none())
    }
    /// The distinct owners, oldest first (a transfer adds one; a metadata update doesn't).
    pub fn owners(&self) -> Vec<Bytes32> {
        let mut out: Vec<Bytes32> = Vec::new();
        for o in self.history.iter().filter_map(|s| s.owner) {
            if out.last() != Some(&o) {
                out.push(o);
            }
        }
        out
    }
    /// The height the DID was launched at.
    pub fn launched(&self) -> Option<u32> {
        self.history.first().and_then(|s| s.created)
    }
}

/// The puzzle hash of a plain DID's eve coin (no recovery list, one verification, empty
/// metadata) launched by `launcher_id` to the owner `p2_puzzle_hash`.
/// (The same as [`super::sponsor::eve_puzzle_hash`], which launches them.)
pub fn eve_puzzle_hash(launcher_id: Bytes32, p2_puzzle_hash: Bytes32) -> Bytes32 {
    super::sponsor::eve_puzzle_hash(launcher_id, p2_puzzle_hash)
}

fn node(allocator: &mut Allocator, program: &Program) -> Result<NodePtr> {
    program
        .to_clvm(allocator)
        .map_err(failed("reading a puzzle"))
}

/// Resolve a DID from its launcher id. `known` (a cached record) lets it continue from the last
/// coin it saw; `peak` is the height recorded as checked.
pub async fn resolve<S: ChainSource>(
    source: &S,
    launcher_id: Bytes32,
    known: Option<DidRecord>,
    peak: u32,
) -> Result<DidRecord> {
    let mut record = match known {
        Some(k) if k.launcher_id == launcher_id && !k.history.is_empty() => k,
        _ => start(source, launcher_id).await?,
    };
    record.melted = false;
    // Refresh the last coin's state, then follow spends. A cached coin the chain no longer has
    // (a reorg undid it) means starting over from the launcher.
    let mut state = None;
    for fresh in [false, true] {
        if fresh {
            record = start(source, launcher_id).await?;
        }
        let last = record.history.last().expect("non-empty").coin_id;
        state = source
            .coin_state(vec![last], false)
            .await?
            .into_iter()
            .next();
        if state.is_some() {
            break;
        }
    }
    let state = state.ok_or_else(|| Error::new("the DID's coin is unknown to the peer"))?;
    let owner = record.history.last().and_then(|s| s.owner);
    *record.history.last_mut().expect("non-empty") = DidStep::of(&state, owner);
    let mut current = state;
    for _ in 0..MAX_STEPS {
        let Some(spent_at) = current.spent_height else {
            break;
        };
        let coin = current.coin;
        let (puzzle, solution) = source.puzzle_and_solution(coin.coin_id(), spent_at).await?;
        let mut allocator = Allocator::new();
        let puzzle_ptr = node(&mut allocator, &puzzle)?;
        let solution_ptr = node(&mut allocator, &solution)?;
        let parsed = Puzzle::parse(&allocator, puzzle_ptr);
        // The spent coin's own owner, read from its revealed puzzle.
        let Some((did, _)) =
            Did::parse(&allocator, coin, parsed, solution_ptr).map_err(failed("DID spend"))?
        else {
            return Err(Error::new("a coin in the DID's line isn't a DID"));
        };
        if did.info.launcher_id != launcher_id {
            return Err(Error::new(
                "a coin in the DID's line belongs to another DID",
            ));
        }
        record.history.last_mut().expect("non-empty").owner = Some(did.info.p2_puzzle_hash);
        // Its child: the odd coin the spend made, as the parent's spend says it must be.
        let mut next = None;
        for child in source.children(coin.coin_id()).await? {
            if child.coin.amount % 2 != 1 {
                continue;
            }
            let parsed_child =
                Did::parse_child(&mut allocator, coin, parsed, solution_ptr, child.coin)
                    .map_err(failed("DID child"))?;
            if let Some(d) = parsed_child
                && Bytes32::from(d.info.puzzle_hash()) == child.coin.puzzle_hash
            {
                next = Some((child, d.info.p2_puzzle_hash));
                break;
            }
        }
        match next {
            Some((child, owner)) => {
                record.history.push(DidStep::of(&child, Some(owner)));
                current = child;
            }
            None => {
                record.melted = true;
                break;
            }
        }
    }
    record.checked_height = peak;
    Ok(record)
}

/// The launcher and the eve coin it created.
async fn start<S: ChainSource>(source: &S, launcher_id: Bytes32) -> Result<DidRecord> {
    let launcher = source
        .coin_state(vec![launcher_id], false)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| Error::new("no such DID: its launcher coin isn't on chain"))?;
    if launcher.coin.puzzle_hash != Bytes32::from(SINGLETON_LAUNCHER_HASH) {
        return Err(Error::new("not a DID: the coin isn't a singleton launcher"));
    }
    if launcher.spent_height.is_none() {
        return Err(Error::new("not a DID yet: its launcher hasn't been spent"));
    }
    let eve = source
        .children(launcher_id)
        .await?
        .into_iter()
        .find(|c| c.coin.amount % 2 == 1)
        .ok_or_else(|| Error::new("the launcher made no singleton"))?;
    Ok(DidRecord {
        launcher_id,
        history: vec![DidStep::of(&eve, None)],
        melted: false,
        checked_height: 0,
    })
}
