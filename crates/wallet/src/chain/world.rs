//! World DIDs (plan phase 4, slice 7): the two DID metadata formats that tie a world to its
//! owner, and the check that both sides say so.
//!
//! A world gets a DID from a sponsor, launched to the owner's address
//! ([`super::sponsor::build_eve_dids_with`]), so the DID stays in the owner's wallet and the
//! world holds only its own key. The world DID's metadata names the owner's DID and the world's
//! current key and generation; the owner's DID lists its worlds in its metadata (one spend of
//! the owner's DID per world added: [`super::ChainReader::set_did_metadata`]). A world counts as
//! the DID's only when both say so ([`world_link`]).
//!
//! # The CLVM shapes
//!
//! Both are proper lists of pairs whose first pair is a tag, so neither can be read as the other
//! or as anyone else's DID metadata. Strings are plain atoms (their UTF-8 bytes), ids and keys
//! 32-byte atoms, the generation a canonical CLVM integer (0 is the empty atom).
//!
//! A world DID's ([`WorldMeta`]):
//!
//! ```text
//! (("chiacraft" . "world v1") ("owner" . <owner DID launcher id, 32 bytes>) ("key" . <world Ed25519 key, 32 bytes>) ("gen" . <generation>))
//! ```
//!
//! An owner DID's ([`OwnerMeta`]), at most [`MAX_WORLDS`] world DID launcher ids, in order:
//!
//! ```text
//! (("chiacraft" . "owner v1") ("worlds" <32 bytes> <32 bytes> ...))
//! ```
//!
//! Reading is strict: a node parses only when it is exactly one of these shapes **and** it is
//! the canonical encoding of the value read (re-encoding it gives the same tree hash: no padded
//! integers, nothing extra after the last entry). So one value has one metadata hash, which an
//! eve DID's puzzle hash commits to ([`super::did::DidRecord::eve_owned_by_with`]). Anything else
//! (nil, other wallets' metadata, a truncated or extended list) reads as `None`.
//!
//! # Revealing a new metadata on chain
//!
//! A DID's spend creates its next coin from a puzzle hash only: the new metadata is not on chain
//! until that coin is spent in turn. chia-sdk-driver's `Did::parse_child` assumes the parent's
//! metadata, so a child whose metadata changed doesn't parse. The metadata spends built here
//! ([`build_metadata_update`]) therefore add the serialized new metadata as the second memo of
//! the DID's `CREATE_COIN` (after the owner hint), and [`super::did::parse_child`] reads it from
//! there, trusting it only when the child's puzzle hash commits to it.
use super::{did::DidRecord, failed, wallet::WalletKeys};
use crate::{Error, Result};
use chia_protocol::{Bytes, Bytes32, Coin, SpendBundle};
use chia_puzzle_types::Memos;
use chia_sdk_driver::{Did, DidInfo, HashedPtr, SingletonInfo, SpendContext, StandardLayer};
use chia_sdk_types::Conditions;
use clvm_traits::{FromClvm, ToClvm};
use clvmr::{
    Allocator, NodePtr,
    serde::{node_from_bytes_backrefs, node_to_bytes},
};

/// The first pair's left side in both formats.
pub const APP: &str = "chiacraft";
/// A world DID's tag.
pub const WORLD_TAG: &str = "world v1";
/// An owner DID's tag.
pub const OWNER_TAG: &str = "owner v1";
/// The most worlds an owner DID lists.
pub const MAX_WORLDS: usize = 64;

/// A world DID's metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorldMeta {
    /// The owner's DID (its launcher id).
    pub owner: Bytes32,
    /// The world's current Ed25519 public key.
    pub key: [u8; 32],
    /// The key's generation (0 at launch, one more per rotation).
    pub generation: u32,
}

type WorldClvm<S> = ((S, S), ((S, Bytes32), ((S, Bytes32), ((S, u32), ()))));
type OwnerClvm<S> = ((S, S), ((S, Vec<Bytes32>), ()));

/// The canonical tree hash of `node`.
fn hash_of(allocator: &Allocator, node: NodePtr) -> Bytes32 {
    clvm_utils::tree_hash(allocator, node).into()
}

/// A value's node in a fresh allocator (the values here are small: it can't fail).
fn alone<T: ToClvm<Allocator>>(value: &T) -> (Allocator, NodePtr) {
    let mut allocator = Allocator::new();
    let node = value.to_clvm(&mut allocator).unwrap_or(NodePtr::NIL);
    (allocator, node)
}

fn bytes_of<T: ToClvm<Allocator>>(value: &T) -> Vec<u8> {
    let (allocator, node) = alone(value);
    node_to_bytes(&allocator, node).unwrap_or_default()
}

fn hash_alone<T: ToClvm<Allocator>>(value: &T) -> Bytes32 {
    let (allocator, node) = alone(value);
    hash_of(&allocator, node)
}

/// Parse serialized CLVM (with or without back references) in a fresh allocator.
fn parse_bytes<T>(bytes: &[u8], parse: impl FnOnce(&Allocator, NodePtr) -> Option<T>) -> Option<T> {
    let mut allocator = Allocator::new();
    let node = node_from_bytes_backrefs(&mut allocator, bytes).ok()?;
    parse(&allocator, node)
}

impl WorldMeta {
    fn clvm(&self) -> WorldClvm<&'static str> {
        ((APP, WORLD_TAG), (("owner", self.owner), (("key", Bytes32::new(self.key)), (("gen", self.generation), ()))))
    }
    /// The metadata in `ctx`, hashed (for a launch or a DID update).
    pub fn alloc(&self, ctx: &mut SpendContext) -> Result<HashedPtr> {
        ctx.alloc_hashed(&self.clvm())
            .map_err(failed("world metadata"))
    }
    /// The metadata's tree hash: what an eve world DID's puzzle hash commits to.
    pub fn tree_hash(&self) -> Bytes32 {
        hash_alone(&self.clvm())
    }
    /// The serialized CLVM.
    pub fn to_bytes(&self) -> Vec<u8> {
        bytes_of(&self.clvm())
    }
    /// A world DID's metadata from a CLVM node; None for anything else.
    pub fn from_node(allocator: &Allocator, node: NodePtr) -> Option<Self> {
        let ((app, tag), ((k_owner, owner), ((k_key, key), ((k_gen, generation), ())))) =
            <WorldClvm<String>>::from_clvm(allocator, node).ok()?;
        if app != APP || tag != WORLD_TAG || k_owner != "owner" || k_key != "key" || k_gen != "gen" {
            return None;
        }
        let meta = Self { owner, key: key.to_bytes(), generation };
        (meta.tree_hash() == hash_of(allocator, node)).then_some(meta)
    }
    /// A world DID's metadata from serialized CLVM; None for anything else.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        parse_bytes(bytes, Self::from_node)
    }
    /// The next generation with a new key (a key rotation).
    pub fn rotated(&self, key: [u8; 32]) -> Self {
        Self { key, generation: self.generation.saturating_add(1), ..*self }
    }
}

/// An owner DID's metadata: the worlds it holds (world DID launcher ids).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct OwnerMeta {
    pub worlds: Vec<Bytes32>,
}

impl OwnerMeta {
    pub fn new(worlds: Vec<Bytes32>) -> Result<Self> {
        let meta = Self { worlds };
        meta.check()?;
        Ok(meta)
    }
    fn check(&self) -> Result<()> {
        if self.worlds.len() > MAX_WORLDS {
            return Err(Error::new(format!("an owner DID lists at most {MAX_WORLDS} worlds")));
        }
        Ok(())
    }
    fn clvm(&self) -> OwnerClvm<&'static str> {
        ((APP, OWNER_TAG), (("worlds", self.worlds.clone()), ()))
    }
    /// Whether it lists `world`.
    pub fn lists(&self, world: Bytes32) -> bool {
        self.worlds.contains(&world)
    }
    /// With `world` added at the end (unchanged when already listed); refused past
    /// [`MAX_WORLDS`].
    pub fn with_world(&self, world: Bytes32) -> Result<Self> {
        let mut out = self.clone();
        if !out.lists(world) {
            out.worlds.push(world);
        }
        out.check()?;
        Ok(out)
    }
    /// Without `world`.
    pub fn without_world(&self, world: Bytes32) -> Self {
        Self {
            worlds: self
                .worlds
                .iter()
                .copied()
                .filter(|w| *w != world)
                .collect(),
        }
    }
    /// The metadata in `ctx`, hashed. Refused past [`MAX_WORLDS`].
    pub fn alloc(&self, ctx: &mut SpendContext) -> Result<HashedPtr> {
        self.check()?;
        ctx.alloc_hashed(&self.clvm())
            .map_err(failed("owner metadata"))
    }
    /// The metadata's tree hash.
    pub fn tree_hash(&self) -> Bytes32 {
        hash_alone(&self.clvm())
    }
    /// The serialized CLVM.
    pub fn to_bytes(&self) -> Vec<u8> {
        bytes_of(&self.clvm())
    }
    /// An owner DID's metadata from a CLVM node; None for anything else (more than
    /// [`MAX_WORLDS`] worlds included).
    pub fn from_node(allocator: &Allocator, node: NodePtr) -> Option<Self> {
        let ((app, tag), ((k_worlds, worlds), ())) = <OwnerClvm<String>>::from_clvm(allocator, node).ok()?;
        if app != APP || tag != OWNER_TAG || k_worlds != "worlds" || worlds.len() > MAX_WORLDS {
            return None;
        }
        let meta = Self { worlds };
        (meta.tree_hash() == hash_of(allocator, node)).then_some(meta)
    }
    /// An owner DID's metadata from serialized CLVM; None for anything else.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        parse_bytes(bytes, Self::from_node)
    }
}

/// A DID's metadata as this crate writes it: none (nil, a plain DID's), a world's or an owner's.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum DidMetadata {
    /// Nil: a plain DID (how `build_eve_dids` and `create_did` launch one).
    #[default]
    Empty,
    World(WorldMeta),
    Owner(OwnerMeta),
}

impl DidMetadata {
    /// The metadata in `ctx`, hashed.
    pub fn alloc(&self, ctx: &mut SpendContext) -> Result<HashedPtr> {
        match self {
            Self::Empty => Ok(HashedPtr::NIL),
            Self::World(m) => m.alloc(ctx),
            Self::Owner(m) => m.alloc(ctx),
        }
    }
    /// The metadata's tree hash.
    pub fn tree_hash(&self) -> Bytes32 {
        match self {
            Self::Empty => HashedPtr::NIL.tree_hash().into(),
            Self::World(m) => m.tree_hash(),
            Self::Owner(m) => m.tree_hash(),
        }
    }
    /// The serialized CLVM (`80` for nil).
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Empty => vec![0x80],
            Self::World(m) => m.to_bytes(),
            Self::Owner(m) => m.to_bytes(),
        }
    }
    /// One of the three from a CLVM node; None for any other metadata.
    pub fn from_node(allocator: &Allocator, node: NodePtr) -> Option<Self> {
        if matches!(allocator.sexp(node), clvmr::SExp::Atom) && allocator.atom_len(node) == 0 {
            return Some(Self::Empty);
        }
        WorldMeta::from_node(allocator, node)
            .map(Self::World)
            .or_else(|| OwnerMeta::from_node(allocator, node).map(Self::Owner))
    }
    /// One of the three from serialized CLVM; None for anything else.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        parse_bytes(bytes, Self::from_node)
    }
    pub fn world(&self) -> Option<&WorldMeta> {
        match self {
            Self::World(m) => Some(m),
            _ => None,
        }
    }
    pub fn owner(&self) -> Option<&OwnerMeta> {
        match self {
            Self::Owner(m) => Some(m),
            _ => None,
        }
    }
}

/// What a world DID and an owner DID say about each other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldLink {
    /// The world DID's metadata, as read (revealed on chain, or the expected one an eve world
    /// DID's puzzle hash was checked against).
    pub world: Option<WorldMeta>,
    /// The world DID's metadata names the owner DID.
    pub world_names_owner: bool,
    /// The owner DID's metadata lists the world DID.
    pub owner_lists_world: bool,
}

impl WorldLink {
    /// Both say so: the world is the owner DID's.
    pub fn holds(&self) -> bool {
        self.world_names_owner && self.owner_lists_world
    }
}

fn owner_lists(world: &DidRecord, owner: &DidRecord) -> bool {
    !owner.melted
        && owner
            .owner_meta()
            .is_some_and(|m| m.lists(world.launcher_id))
}

/// Whether `world` (a world DID's record) and `owner` (its owner DID's record) name each other,
/// from the metadata the chain revealed. A melted DID names nothing. An eve world DID (never
/// spent: its metadata isn't revealed yet) names nobody here: check it with
/// [`world_link_expecting`].
pub fn world_link(world: &DidRecord, owner: &DidRecord) -> WorldLink {
    let meta = (!world.melted).then(|| world.world_meta()).flatten();
    WorldLink {
        world: meta,
        world_names_owner: meta.is_some_and(|m| m.owner == owner.launcher_id),
        owner_lists_world: owner_lists(world, owner),
    }
}

/// [`world_link`], for a world DID that may still be eve: when the chain hasn't revealed its
/// metadata, it counts as `expected` (what the world says its metadata is) if the eve coin's
/// puzzle hash commits to `expected` and to `launched_to` (the owner's p2 puzzle hash it was
/// launched to). Revealed metadata always wins over `expected`.
pub fn world_link_expecting(world: &DidRecord, owner: &DidRecord, expected: &WorldMeta, launched_to: Bytes32) -> WorldLink {
    let mut link = world_link(world, owner);
    if link.world.is_none() && world.metadata.is_none() && world.eve_owned_by_with(launched_to, expected.tree_hash()) {
        link.world = Some(*expected);
        link.world_names_owner = expected.owner == owner.launcher_id;
    }
    link
}

/// The memos of a DID's new coin after a metadata update: the owner hint, then the serialized
/// new metadata (see the module's doc).
fn reveal_memos(ctx: &mut SpendContext, p2_puzzle_hash: Bytes32, metadata: &[u8]) -> Result<Memos<NodePtr>> {
    let memos = ctx
        .alloc(&(p2_puzzle_hash, (Bytes::new(metadata.to_vec()), ())))
        .map_err(failed("DID memos"))?;
    Ok(Memos::Some(memos))
}

/// A signed metadata update of `did` (its current coin as [`super::did::current_did`] rebuilt
/// it in `ctx`), held at one of `keys`' addresses: the DID is spent by its owner's key back to
/// the same owner with `metadata` (chia-sdk-driver's `Did::update_with_metadata`, plus the new
/// metadata revealed in the memos). `fee_coins` (the wallet's, at its addresses) pay `fee`, the
/// change back to the primary address. The bundle and the DID's new coin.
pub fn build_metadata_update(
    ctx: &mut SpendContext,
    keys: &WalletKeys,
    did: Did,
    metadata: &DidMetadata,
    fee_coins: &[Coin],
    fee: u64,
) -> Result<(SpendBundle, Did)> {
    let ours = keys.synthetic_keys();
    let owner = did.info.p2_puzzle_hash;
    let key = *ours
        .get(&owner)
        .ok_or_else(|| Error::new("the DID isn't held at one of this wallet's addresses"))?;
    let total: u64 = fee_coins.iter().map(|c| c.amount).sum();
    if total < fee {
        return Err(Error::new(format!("not enough XCH for the fee: {fee} mojos needed, {total} given")));
    }
    if fee == 0 && !fee_coins.is_empty() {
        return Err(Error::new("coins given for no fee"));
    }
    let new_metadata = metadata.alloc(ctx)?;
    let bytes = metadata.to_bytes();
    let new_info = DidInfo { metadata: new_metadata, ..did.info };
    let memos = reveal_memos(ctx, owner, &bytes)?;
    let mut conditions = Conditions::new().create_coin(new_info.inner_puzzle_hash().into(), did.coin.amount, memos);
    if let Some(first) = fee_coins.first() {
        conditions = conditions.assert_concurrent_spend(first.coin_id());
    }
    // The same spend as `update_with_metadata` (whose memos are the hint alone).
    did.spend_with(ctx, &StandardLayer::new(key), conditions)
        .map_err(failed("DID update"))?;
    let child = did.child(owner, new_metadata, did.coin.amount);
    for (i, coin) in fee_coins.iter().enumerate() {
        let layer = ours
            .get(&coin.puzzle_hash)
            .map(|k| StandardLayer::new(*k))
            .ok_or_else(|| Error::new("a fee coin not at one of this wallet's addresses"))?;
        let mut c = Conditions::new().assert_concurrent_spend(did.coin.coin_id());
        if i == 0 {
            c = c.reserve_fee(fee);
            if total > fee {
                c = c.create_coin(keys.primary(), total - fee, Memos::None);
            }
        }
        layer.spend(ctx, *coin, c).map_err(failed("fee spend"))?;
    }
    let coin_spends = ctx.take();
    let signature = keys.sign(&coin_spends)?;
    Ok((SpendBundle::new(coin_spends, signature), child))
}

/// The metadata `record` revealed, parsed (None while unrevealed or another format).
pub(crate) fn parse_hex(hex: &str) -> Option<DidMetadata> {
    DidMetadata::from_bytes(&crate::keys::from_hex(hex)?)
}

/// `did`'s metadata as hex of its serialized CLVM.
pub(crate) fn metadata_hex(allocator: &Allocator, metadata: HashedPtr) -> Option<String> {
    node_to_bytes(allocator, metadata.ptr())
        .ok()
        .map(|b| crate::keys::hex(&b))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> WorldMeta {
        WorldMeta { owner: Bytes32::new([1; 32]), key: [2; 32], generation: 3 }
    }

    #[test]
    fn world_metadata_round_trips() {
        let w = world();
        assert_eq!(WorldMeta::from_bytes(&w.to_bytes()), Some(w));
        let zero = WorldMeta { generation: 0, ..w };
        assert_eq!(WorldMeta::from_bytes(&zero.to_bytes()), Some(zero));
        let big = WorldMeta { generation: u32::MAX, ..w };
        assert_eq!(WorldMeta::from_bytes(&big.to_bytes()), Some(big));
        assert_eq!(w.rotated([9; 32]).generation, 4);
        // The tree hash without a context is the allocated one's.
        let mut ctx = SpendContext::new();
        let p = w.alloc(&mut ctx).unwrap();
        assert_eq!(Bytes32::from(p.tree_hash()), w.tree_hash());
        assert_eq!(WorldMeta::from_node(&ctx, p.ptr()), Some(w));
        assert_eq!(DidMetadata::from_node(&ctx, p.ptr()), Some(DidMetadata::World(w)));
        assert_ne!(w.tree_hash(), zero.tree_hash());
    }

    #[test]
    fn owner_metadata_round_trips_and_is_capped() {
        let none = OwnerMeta::default();
        assert_eq!(OwnerMeta::from_bytes(&none.to_bytes()), Some(none.clone()));
        let two = none
            .with_world(Bytes32::new([5; 32]))
            .unwrap()
            .with_world(Bytes32::new([6; 32]))
            .unwrap();
        assert_eq!(two.worlds.len(), 2);
        assert_eq!(two.with_world(Bytes32::new([5; 32])).unwrap(), two);
        assert_eq!(OwnerMeta::from_bytes(&two.to_bytes()), Some(two.clone()));
        assert!(two.lists(Bytes32::new([6; 32])));
        assert_eq!(two.without_world(Bytes32::new([5; 32])).worlds, vec![Bytes32::new([6; 32])]);
        let mut ctx = SpendContext::new();
        let p = two.alloc(&mut ctx).unwrap();
        assert_eq!(Bytes32::from(p.tree_hash()), two.tree_hash());
        let full = OwnerMeta::new(
            (0..MAX_WORLDS as u8)
                .map(|i| Bytes32::new([i; 32]))
                .collect(),
        )
        .unwrap();
        assert_eq!(OwnerMeta::from_bytes(&full.to_bytes()), Some(full.clone()));
        assert!(full.with_world(Bytes32::new([200; 32])).is_err());
        let over = OwnerMeta {
            worlds: (0..=MAX_WORLDS as u8)
                .map(|i| Bytes32::new([i; 32]))
                .collect(),
        };
        assert!(over.alloc(&mut ctx).is_err());
        assert_eq!(OwnerMeta::from_bytes(&over.to_bytes()), None, "more than 64 worlds isn't ours");
    }

    #[test]
    fn other_shapes_parse_as_none() {
        let w = world();
        let o = OwnerMeta { worlds: vec![Bytes32::new([7; 32])] };
        // Each format is not the other.
        assert_eq!(OwnerMeta::from_bytes(&w.to_bytes()), None);
        assert_eq!(WorldMeta::from_bytes(&o.to_bytes()), None);
        // Nil: a plain DID's.
        assert_eq!(WorldMeta::from_bytes(&[0x80]), None);
        assert_eq!(OwnerMeta::from_bytes(&[0x80]), None);
        assert_eq!(DidMetadata::from_bytes(&[0x80]), Some(DidMetadata::Empty));
        assert_eq!(DidMetadata::Empty.tree_hash(), Bytes32::from(HashedPtr::NIL.tree_hash()));
        // Garbage bytes, other wallets' metadata.
        assert_eq!(DidMetadata::from_bytes(&[0xff]), None);
        assert_eq!(DidMetadata::from_bytes(&[]), None);
        let other = bytes_of(&("Metadata", ("Example", ())));
        assert_eq!(DidMetadata::from_bytes(&other), None);
        let sage_like = bytes_of(&vec![("name", "Alice")]);
        assert_eq!(DidMetadata::from_bytes(&sage_like), None);
        // The right shape with a wrong tag or key name.
        let wrong_tag = bytes_of(&((APP, "world v2"), (("owner", w.owner), (("key", Bytes32::new(w.key)), (("gen", 3u32), ())))));
        assert_eq!(WorldMeta::from_bytes(&wrong_tag), None);
        let wrong_key = bytes_of(&((APP, WORLD_TAG), (("owner", w.owner), (("pk", Bytes32::new(w.key)), (("gen", 3u32), ())))));
        assert_eq!(WorldMeta::from_bytes(&wrong_key), None);
        // A short key, an extra entry, a padded (non-canonical) generation.
        let short = bytes_of(&((APP, WORLD_TAG), (("owner", w.owner), (("key", Bytes::new(vec![1; 31])), (("gen", 3u32), ())))));
        assert_eq!(WorldMeta::from_bytes(&short), None);
        let extra = bytes_of(&((APP, WORLD_TAG), (("owner", w.owner), (("key", Bytes32::new(w.key)), (("gen", 3u32), (("x", 1u8), ()))))));
        assert_eq!(WorldMeta::from_bytes(&extra), None);
        let padded =
            bytes_of(&((APP, WORLD_TAG), (("owner", w.owner), (("key", Bytes32::new(w.key)), (("gen", Bytes::new(vec![0, 3])), ())))));
        assert_eq!(WorldMeta::from_bytes(&padded), None);
        // A world id that isn't 32 bytes.
        let bad_world = bytes_of(&((APP, OWNER_TAG), (("worlds", (Bytes::new(vec![1; 5]), ())), ())));
        assert_eq!(OwnerMeta::from_bytes(&bad_world), None);
    }

    #[test]
    fn a_record_reads_its_revealed_metadata() {
        let w = world();
        let hex = crate::keys::hex(&w.to_bytes());
        assert_eq!(parse_hex(&hex), Some(DidMetadata::World(w)));
        assert_eq!(parse_hex("zz"), None);
    }
}
