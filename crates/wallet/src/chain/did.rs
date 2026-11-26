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
//! - The current coin's metadata is known once a spend revealed it: a spent coin's puzzle shows
//!   its own; a child's is its parent's unless the spend changed it, in which case it is read
//!   from the memos a chiacraft metadata update writes ([`super::world`]) and trusted only when
//!   the child's puzzle hash commits to it. An eve DID's metadata isn't revealed until it is
//!   spent: [`DidRecord::eve_owned_by_with`] checks an expected one against its puzzle hash.
//!
//! A [`DidRecord`] is cached; a later resolve continues from its last coin instead of walking the
//! whole history again. Ported from `@chia/core` `did.ts` (`fetchDid`, `resolveDid`).
use super::{
    failed,
    source::ChainSource,
    sync::hex_bytes32,
    sync::hex_bytes32_opt,
    world::{self, DidMetadata, OwnerMeta, WorldMeta},
};
use crate::{Error, Result};
use chia_protocol::{Bytes, Bytes32, Coin, CoinState, Program};
use chia_puzzle_types::{EveProof, Memos, Proof};
use chia_puzzles::SINGLETON_LAUNCHER_HASH;
use chia_sdk_driver::{Did, DidInfo, HashedPtr, Layer, Puzzle, SingletonInfo, SingletonLayer};
use chia_sdk_types::{Condition, run_puzzle};
use clvm_traits::{FromClvm, ToClvm};
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
    /// The current coin's metadata, hex of its serialized CLVM, when the chain revealed it (see
    /// the module's doc; None for an eve DID never spent, and in records cached before this was
    /// kept). A melted DID's is its last coin's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
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
        self.eve_owned_by_with(p2_puzzle_hash, HashedPtr::NIL.tree_hash().into())
    }
    /// Whether this is an unspent eve DID launched to `p2_puzzle_hash` with metadata whose tree
    /// hash is `metadata_hash` (no recovery list, one verification): a sponsored world DID, whose
    /// metadata the chain doesn't show until it is spent. A verifier told the world's expected
    /// [`WorldMeta`] checks it as `eve_owned_by_with(owner, expected.tree_hash())`: the eve
    /// puzzle hash commits to both.
    pub fn eve_owned_by_with(&self, p2_puzzle_hash: Bytes32, metadata_hash: Bytes32) -> bool {
        match self.history.as_slice() {
            [eve] if !self.melted && eve.spent.is_none() && eve.owner.is_none() => {
                eve.puzzle_hash == super::sponsor::eve_puzzle_hash_with(self.launcher_id, p2_puzzle_hash, metadata_hash)
            }
            _ => false,
        }
    }
    /// The revealed metadata's serialized CLVM.
    pub fn metadata_bytes(&self) -> Option<Vec<u8>> {
        self.metadata.as_deref().and_then(crate::keys::from_hex)
    }
    /// The revealed metadata, when it is nil, a world's or an owner's.
    pub fn did_metadata(&self) -> Option<DidMetadata> {
        self.metadata.as_deref().and_then(world::parse_hex)
    }
    /// The revealed metadata as a world DID's.
    pub fn world_meta(&self) -> Option<WorldMeta> {
        self.did_metadata().and_then(|m| m.world().copied())
    }
    /// The revealed metadata as an owner DID's.
    pub fn owner_meta(&self) -> Option<OwnerMeta> {
        self.did_metadata().and_then(|m| m.owner().cloned())
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
pub async fn resolve<S: ChainSource>(source: &S, launcher_id: Bytes32, known: Option<DidRecord>, peak: u32) -> Result<DidRecord> {
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
        let Some((did, _)) = Did::parse(&allocator, coin, parsed, solution_ptr).map_err(failed("DID spend"))? else {
            return Err(Error::new("a coin in the DID's line isn't a DID"));
        };
        if did.info.launcher_id != launcher_id {
            return Err(Error::new("a coin in the DID's line belongs to another DID"));
        }
        record.history.last_mut().expect("non-empty").owner = Some(did.info.p2_puzzle_hash);
        record.metadata = world::metadata_hex(&allocator, did.info.metadata);
        // Its child: the odd coin the spend made, as the parent's spend says it must be.
        let mut next = None;
        for child in source.children(coin.coin_id()).await? {
            if child.coin.amount % 2 != 1 {
                continue;
            }
            if let Some(d) = parse_child(&mut allocator, coin, parsed, solution_ptr, child.coin)? {
                next = Some((child, d.info.p2_puzzle_hash, world::metadata_hex(&allocator, d.info.metadata)));
                break;
            }
        }
        match next {
            Some((child, owner, metadata)) => {
                record.history.push(DidStep::of(&child, Some(owner)));
                record.metadata = metadata;
                current = child;
            }
            None => {
                record.melted = true;
                break;
            }
        }
    }
    // A record cached before metadata was kept: read the current coin's from its parent's spend.
    if !record.melted && record.metadata.is_none() && record.history.len() >= 2 {
        let parent = &record.history[record.history.len() - 2];
        let current = record.history.last().expect("non-empty").coin();
        if let Some(spent_at) = parent.spent {
            let mut allocator = Allocator::new();
            // Best effort: the record stands without it.
            if let Ok(Some(d)) = child_of(source, &mut allocator, parent.coin(), spent_at, current).await {
                record.metadata = world::metadata_hex(&allocator, d.info.metadata);
            }
        }
    }
    record.checked_height = peak;
    Ok(record)
}

/// The DID coin `child` as its parent's spend (at `spent_at`) made it, rebuilt in `allocator`.
async fn child_of<S: ChainSource>(source: &S, allocator: &mut Allocator, parent: Coin, spent_at: u32, child: Coin) -> Result<Option<Did>> {
    let (puzzle, solution) = source
        .puzzle_and_solution(parent.coin_id(), spent_at)
        .await?;
    let puzzle_ptr = node(allocator, &puzzle)?;
    let solution_ptr = node(allocator, &solution)?;
    let parsed = Puzzle::parse(allocator, puzzle_ptr);
    parse_child(allocator, parent, parsed, solution_ptr, child)
}

/// The DID coin `child` made by its parent's spend (`parent_puzzle`, `parent_solution`), with its
/// lineage proof, owner and metadata, trusted only when it hashes to `child`'s puzzle hash. The
/// metadata is the parent's (chia-sdk-driver's `Did::parse_child`) unless the spend changed it;
/// then it is the one the spend revealed in the memos (the hint, then the serialized metadata:
/// [`world::build_metadata_update`]). None when the child isn't the DID's next coin, or its new
/// metadata wasn't revealed.
pub fn parse_child(
    allocator: &mut Allocator,
    parent: Coin,
    parent_puzzle: Puzzle,
    parent_solution: NodePtr,
    child: Coin,
) -> Result<Option<Did>> {
    let Some(did) = Did::parse_child(allocator, parent, parent_puzzle, parent_solution, child).map_err(failed("DID child"))? else {
        return Ok(None);
    };
    if Bytes32::from(did.info.puzzle_hash()) == child.puzzle_hash {
        return Ok(Some(did));
    }
    let Some(metadata) = revealed_metadata(allocator, parent_puzzle, parent_solution) else {
        return Ok(None);
    };
    let info = DidInfo { metadata, ..did.info };
    Ok((Bytes32::from(info.puzzle_hash()) == child.puzzle_hash).then(|| Did::new(child, did.proof, info)))
}

/// The new metadata a DID spend revealed: the second memo of its singleton `CREATE_COIN`.
fn revealed_metadata(allocator: &mut Allocator, parent_puzzle: Puzzle, parent_solution: NodePtr) -> Option<HashedPtr> {
    let singleton = SingletonLayer::<Puzzle>::parse_puzzle(allocator, parent_puzzle).ok()??;
    let solution = SingletonLayer::<Puzzle>::parse_solution(allocator, parent_solution).ok()?;
    let output = run_puzzle(allocator, singleton.inner_puzzle.ptr(), solution.inner_solution).ok()?;
    let conditions = Vec::<Condition>::from_clvm(allocator, output).ok()?;
    let create = conditions
        .into_iter()
        .filter_map(Condition::into_create_coin)
        .find(|c| c.amount % 2 == 1)?;
    let Memos::Some(memos) = create.memos else {
        return None;
    };
    let (_hint, (bytes, _rest)) = <(Bytes32, (Bytes, NodePtr))>::from_clvm(allocator, memos).ok()?;
    let ptr = clvmr::serde::node_from_bytes_backrefs(allocator, bytes.as_ref()).ok()?;
    Some(HashedPtr::from_ptr(allocator, ptr))
}

/// The DID's current coin as a spendable `Did` (puzzle, lineage proof, owner, metadata), rebuilt
/// in `allocator`, for its owner's next spend (a metadata update).
///
/// - A coin made by a spend: its parent's spend gives it all, the lineage proof included
///   ([`parse_child`]).
/// - An eve coin (never spent: a sponsored DID): its parent is the launcher, so its proof is an
///   eve proof of the launcher's parent and amount (as `Launcher::create_eve_did` makes it); its
///   owner and metadata aren't on chain, so they are found among `owners` (p2 puzzle hashes) and
///   `metadata` (candidates, in `allocator`) as the pair the eve puzzle hash commits to (no
///   recovery list, one verification, as a sponsor launches it).
pub async fn current_did<S: ChainSource>(
    source: &S,
    allocator: &mut Allocator,
    record: &DidRecord,
    owners: &[Bytes32],
    metadata: &[HashedPtr],
) -> Result<Did> {
    if record.melted {
        return Err(Error::new("the DID was melted"));
    }
    let current = record
        .current()
        .ok_or_else(|| Error::new("the DID has no coin"))?;
    if current.spent.is_some() {
        return Err(Error::new("the DID's coin is already spent: resolve it again"));
    }
    if let [eve] = record.history.as_slice() {
        let launcher = source
            .coin_state(vec![record.launcher_id], false)
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| Error::new("the DID's launcher isn't on chain"))?
            .coin;
        if eve.parent != record.launcher_id {
            return Err(Error::new("the DID's eve coin isn't its launcher's child"));
        }
        let proof = Proof::Eve(EveProof { parent_parent_coin_info: launcher.parent_coin_info, parent_amount: launcher.amount });
        for owner in owners {
            for m in metadata {
                let info = DidInfo::new(record.launcher_id, None, 1, *m, *owner);
                if Bytes32::from(info.puzzle_hash()) == eve.puzzle_hash {
                    return Ok(Did::new(eve.coin(), proof, info));
                }
            }
        }
        return Err(Error::new("the DID is still eve and wasn't launched to these addresses with the metadata given"));
    }
    let parent = &record.history[record.history.len() - 2];
    let spent_at = parent
        .spent
        .ok_or_else(|| Error::new("the DID's history is broken"))?;
    child_of(source, allocator, parent.coin(), spent_at, current.coin())
        .await?
        .ok_or_else(|| Error::new("the DID's current coin couldn't be rebuilt from its parent's spend"))
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
    Ok(DidRecord { launcher_id, history: vec![DidStep::of(&eve, None)], melted: false, checked_height: 0, metadata: None })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_cached_before_metadata_still_load() {
        // A record as the cache wrote it before `metadata` existed.
        let old = r#"{
            "launcher_id": "0101010101010101010101010101010101010101010101010101010101010101",
            "history": [{
                "coin_id": "0202020202020202020202020202020202020202020202020202020202020202",
                "parent": "0101010101010101010101010101010101010101010101010101010101010101",
                "puzzle_hash": "0303030303030303030303030303030303030303030303030303030303030303",
                "amount": 1, "created": 5, "spent": null, "owner": null
            }],
            "melted": false,
            "checked_height": 9
        }"#;
        let record: DidRecord = serde_json::from_str(old).unwrap();
        assert_eq!(record.metadata, None);
        assert_eq!(record.world_meta(), None);
        assert_eq!(record.owner_meta(), None);
        assert!(record.is_eve());
        // Written again, it stays readable by the old format (no new key while unknown).
        let text = serde_json::to_string(&record).unwrap();
        assert!(!text.contains("metadata"), "{text}");
        // With metadata, it round-trips and parses.
        let world = WorldMeta { owner: Bytes32::new([4; 32]), key: [5; 32], generation: 1 };
        let with = DidRecord { metadata: Some(crate::keys::hex(&world.to_bytes())), ..record };
        let back: DidRecord = serde_json::from_str(&serde_json::to_string(&with).unwrap()).unwrap();
        assert_eq!(back, with);
        assert_eq!(back.world_meta(), Some(world));
        assert_eq!(back.owner_meta(), None);
        assert_eq!(back.did_metadata(), Some(DidMetadata::World(world)));
    }

    #[test]
    fn eve_checks_commit_to_the_metadata() {
        let launcher_id = Bytes32::new([1; 32]);
        let owner = Bytes32::new([2; 32]);
        let world = WorldMeta { owner: Bytes32::new([4; 32]), key: [5; 32], generation: 0 };
        let step = |puzzle_hash| DidStep {
            coin_id: Bytes32::new([6; 32]),
            parent: launcher_id,
            puzzle_hash,
            amount: 1,
            created: Some(1),
            spent: None,
            owner: None,
        };
        let record =
            |puzzle_hash| DidRecord { launcher_id, history: vec![step(puzzle_hash)], melted: false, checked_height: 1, metadata: None };
        let plain = record(eve_puzzle_hash(launcher_id, owner));
        assert!(plain.eve_owned_by(owner));
        assert!(plain.eve_owned_by_with(owner, DidMetadata::Empty.tree_hash()));
        assert!(!plain.eve_owned_by_with(owner, world.tree_hash()));
        let w = record(super::super::sponsor::eve_puzzle_hash_with(launcher_id, owner, world.tree_hash()));
        assert!(w.eve_owned_by_with(owner, world.tree_hash()));
        assert!(!w.eve_owned_by(owner), "a world DID isn't a plain eve DID");
        assert!(!w.owned_by(owner));
        assert!(!w.eve_owned_by_with(Bytes32::new([3; 32]), world.tree_hash()));
        let rotated = world.rotated([7; 32]);
        assert!(!w.eve_owned_by_with(owner, rotated.tree_hash()));
    }
}
