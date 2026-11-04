//! Sponsored DIDs (identity-research.md 2.2 (c), plan row I9): a sponsor's wallet launches DIDs
//! **straight to players' p2 puzzle hashes** with chia-sdk-driver's `Launcher::create_eve_did`,
//! several in one spend bundle. The player signs nothing; the sponsor pays the coin (1 mojo per
//! DID) and the fee, and never controls a DID: its eve coin's puzzle commits to the player's p2
//! puzzle hash from the launch on, so only the player's key can ever spend it.
//!
//! One of the sponsor's coins is the parent of every launcher in a batch: launchers of amounts
//! 0, 1, 2, … (distinct coin ids; the launcher amount doesn't matter, each eve DID holds 1 mojo,
//! as chia-sdk-driver's own `CreateDidAction` does it), the change back to the sponsor's primary
//! address, the fee reserved; further coins, when one isn't enough, assert that they are spent
//! together with it. Nothing here logs a spend, a signature or a key.
//!
//! [`build_eve_dids_with`] (plan phase 4, slice 7) also launches world DIDs: their metadata
//! ([`WorldMeta`]: the owner's DID, the world's key and generation) is set at launch, so the
//! eve puzzle hash commits to it ([`eve_puzzle_hash_with`]) and the player spends nothing. It can
//! send a few mojos with a DID (a gift coin at the owner's address, hinted to it) so the owner can
//! pay a small fee for the DID's first spend. [`build_eve_dids`] is the same launch with plain
//! asks.
use super::{failed, wallet::WalletKeys, world::WorldMeta};
use crate::{Error, Result};
use chia_protocol::{Bytes32, Coin, SpendBundle};
use chia_puzzle_types::Memos;
use chia_sdk_driver::{HashedPtr, Launcher, SingletonInfo, SpendContext, StandardLayer};
use chia_sdk_types::Conditions;
use clvmr::NodePtr;

/// The most DIDs one bundle launches (a block's cost limit allows far more; this keeps a bundle
/// small enough for any mempool).
pub const MAX_BATCH: usize = 100;

/// One DID a sponsored bundle launches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EveDid {
    /// The DID's id (`did:chia:1…` is its bech32m).
    pub launcher_id: Bytes32,
    /// The eve coin: the DID's coin until its owner first spends it.
    pub coin: Coin,
    /// The owner's p2 puzzle hash (the player's address).
    pub owner: Bytes32,
    /// The world metadata it was launched with ([`EveAsk::world`]; None: a plain DID, nil
    /// metadata).
    pub world: Option<WorldMeta>,
    /// The coin of mojos the owner was given with it ([`EveAsk::gift`]). Gifts to one owner in
    /// one bundle are one coin (their sum), shared by that owner's DIDs.
    pub gift: Option<Coin>,
}

/// One DID a sponsor is asked to launch ([`build_eve_dids_with`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EveAsk {
    /// The owner's p2 puzzle hash (the player's address): where the DID goes.
    pub owner: Bytes32,
    /// A world DID's metadata, set at launch (None: a plain DID with nil metadata).
    pub world: Option<WorldMeta>,
    /// Mojos sent with the DID to the owner's address (hinted to it), so the owner can pay a
    /// small fee for its first spend on testnet11. 0: none.
    pub gift: u64,
}

impl EveAsk {
    /// A plain DID for `owner`, nothing else: what [`build_eve_dids`] launches.
    pub fn plain(owner: Bytes32) -> Self {
        Self { owner, world: None, gift: 0 }
    }
}

/// A signed bundle launching DIDs.
#[derive(Clone, Debug)]
pub struct SponsorBundle {
    pub bundle: SpendBundle,
    /// In the order of the owners asked for.
    pub dids: Vec<EveDid>,
    /// The sponsor's coins it spends.
    pub spent: Vec<Bytes32>,
    /// Mojos back to the sponsor's primary address.
    pub change: u64,
    pub fee: u64,
    /// Mojos given to owners with their DIDs ([`EveAsk::gift`]), in all.
    pub gifts: u64,
}

impl SponsorBundle {
    /// What the batch costs the sponsor: a mojo per DID, the gifts and the fee.
    pub fn cost(&self) -> u64 {
        self.dids.len() as u64 + self.gifts + self.fee
    }
}

/// The eve puzzle hash of a DID launched by [`build_eve_dids`] (no recovery list, one
/// verification, no metadata) for `owner`: what the eve coin's puzzle hash must be.
pub fn eve_puzzle_hash(launcher_id: Bytes32, owner: Bytes32) -> Bytes32 {
    eve_puzzle_hash_with(launcher_id, owner, HashedPtr::NIL.tree_hash().into())
}

/// The eve puzzle hash of a DID launched by [`build_eve_dids_with`] for `owner` with metadata
/// whose tree hash is `metadata_hash` (a world DID's: [`WorldMeta::tree_hash`]; nil's for a plain
/// DID): no recovery list, one verification.
pub fn eve_puzzle_hash_with(launcher_id: Bytes32, owner: Bytes32, metadata_hash: Bytes32) -> Bytes32 {
    // Only the metadata's hash goes into a puzzle hash: a pointer to nothing is enough here.
    let metadata = HashedPtr::new(NodePtr::NIL, metadata_hash.into());
    chia_sdk_driver::DidInfo::new(launcher_id, None, 1, metadata, owner)
        .puzzle_hash()
        .into()
}

/// Launch one DID per owner from `coins` (the sponsor's spendable coins; the largest first are
/// used until they cover a mojo per DID plus `fee`), signed with `keys`.
pub fn build_eve_dids(keys: &WalletKeys, coins: &[Coin], owners: &[Bytes32], fee: u64) -> Result<SponsorBundle> {
    let asks: Vec<EveAsk> = owners.iter().copied().map(EveAsk::plain).collect();
    build_eve_dids_with(keys, coins, &asks, fee)
}

/// Launch one DID per ask from `coins` (the sponsor's spendable coins; the largest first are
/// used until they cover a mojo per DID, the gifts and `fee`), signed with `keys`: each to its
/// owner, with its world metadata when it has one, and the owner's gift (one coin per owner, the
/// sum of its asks' gifts, at its address and hinted to it).
pub fn build_eve_dids_with(keys: &WalletKeys, coins: &[Coin], asks: &[EveAsk], fee: u64) -> Result<SponsorBundle> {
    if asks.is_empty() {
        return Err(Error::new("no DID to launch"));
    }
    if asks.len() > MAX_BATCH {
        return Err(Error::new(format!("at most {MAX_BATCH} DIDs in one bundle")));
    }
    // The gifts, one per owner in the order first asked.
    let mut gifts: Vec<(Bytes32, u64)> = Vec::new();
    for ask in asks.iter().filter(|a| a.gift > 0) {
        match gifts.iter_mut().find(|(o, _)| *o == ask.owner) {
            Some((_, sum)) => {
                *sum = sum
                    .checked_add(ask.gift)
                    .ok_or_else(|| Error::new("the gifts overflow"))?;
            }
            None => gifts.push((ask.owner, ask.gift)),
        }
    }
    let gifts_total = gifts
        .iter()
        .try_fold(0u64, |t, (_, g)| t.checked_add(*g))
        .ok_or_else(|| Error::new("the gifts overflow"))?;
    let need = (asks.len() as u64)
        .checked_add(fee)
        .and_then(|n| n.checked_add(gifts_total))
        .ok_or_else(|| Error::new("the fee overflows"))?;
    let ours = keys.synthetic_keys();
    let mut sorted: Vec<Coin> = coins
        .iter()
        .filter(|c| ours.contains_key(&c.puzzle_hash))
        .copied()
        .collect();
    sorted.sort_by_key(|c| std::cmp::Reverse(c.amount));
    let mut selected = Vec::new();
    let mut total = 0u64;
    for c in sorted {
        if total >= need {
            break;
        }
        total = total.saturating_add(c.amount);
        selected.push(c);
    }
    if total < need {
        return Err(Error::new(format!("not enough spendable coins: {total} of {need} mojos")));
    }
    let change = total - need;
    let parent = selected[0];
    // Two coins of one parent with the same puzzle hash and amount would be one coin.
    if change > 0
        && gifts
            .iter()
            .any(|(o, g)| *o == keys.primary() && *g == change)
    {
        return Err(Error::new("a gift to the sponsor's own address equal to its change"));
    }
    let mut ctx = SpendContext::new();
    let mut conditions = Conditions::new();
    let mut gift_coins: Vec<(Bytes32, Coin)> = Vec::with_capacity(gifts.len());
    for (owner, amount) in &gifts {
        let memos = ctx.hint(*owner).map_err(failed("gift memos"))?;
        conditions = conditions.create_coin(*owner, *amount, memos);
        gift_coins.push((*owner, Coin::new(parent.coin_id(), *owner, *amount)));
    }
    let mut dids = Vec::with_capacity(asks.len());
    for (i, ask) in asks.iter().enumerate() {
        let (create, launcher) = Launcher::create_early(parent.coin_id(), i as u64);
        let launcher = launcher.with_singleton_amount(1);
        let launcher_id = launcher.coin().coin_id();
        let metadata = match &ask.world {
            Some(world) => world.alloc(&mut ctx)?,
            None => HashedPtr::NIL,
        };
        let (launch, did) = launcher
            .create_eve_did(&mut ctx, ask.owner, None, 1, metadata)
            .map_err(failed("DID launch"))?;
        conditions = conditions.with(create).extend(launch);
        let gift = gift_coins
            .iter()
            .find(|(o, _)| *o == ask.owner)
            .map(|(_, c)| *c)
            .filter(|_| ask.gift > 0);
        dids.push(EveDid { launcher_id, coin: did.coin, owner: ask.owner, world: ask.world, gift });
    }
    if change > 0 {
        conditions = conditions.create_coin(keys.primary(), change, Memos::None);
    }
    if fee > 0 {
        conditions = conditions.reserve_fee(fee);
    }
    for other in &selected[1..] {
        conditions = conditions.assert_concurrent_spend(other.coin_id());
    }
    let layer = |coin: &Coin| {
        ours.get(&coin.puzzle_hash)
            .map(|k| StandardLayer::new(*k))
            .ok_or_else(|| Error::new("a coin not at one of the sponsor's addresses"))
    };
    layer(&parent)?
        .spend(&mut ctx, parent, conditions)
        .map_err(failed("sponsor spend"))?;
    for other in &selected[1..] {
        layer(other)?
            .spend(&mut ctx, *other, Conditions::new().assert_concurrent_spend(parent.coin_id()))
            .map_err(failed("sponsor spend"))?;
    }
    let coin_spends = ctx.take();
    let signature = keys.sign(&coin_spends)?;
    Ok(SponsorBundle {
        bundle: SpendBundle::new(coin_spends, signature),
        dids,
        spent: selected.iter().map(Coin::coin_id).collect(),
        change,
        fee,
        gifts: gifts_total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sponsor() -> WalletKeys {
        WalletKeys::from_mnemonic(&crate::Mnemonic::generate().unwrap(), super::super::ChainNetwork::Testnet11, 1)
    }

    #[test]
    fn plain_asks_match_the_old_launch() {
        let keys = sponsor();
        let coin = Coin::new(Bytes32::new([1; 32]), keys.primary(), 1_000);
        let owners = [Bytes32::new([2; 32]), Bytes32::new([3; 32])];
        let old = build_eve_dids(&keys, &[coin], &owners, 7).unwrap();
        let asks: Vec<EveAsk> = owners.iter().copied().map(EveAsk::plain).collect();
        let new = build_eve_dids_with(&keys, &[coin], &asks, 7).unwrap();
        assert_eq!(old.dids, new.dids);
        assert_eq!(old.bundle, new.bundle);
        assert_eq!((old.change, old.gifts, old.cost()), (1_000 - 2 - 7, 0, 9));
        for d in &old.dids {
            assert_eq!(d.coin.puzzle_hash, eve_puzzle_hash(d.launcher_id, d.owner));
            assert_eq!(d.gift, None);
        }
    }

    #[test]
    fn world_dids_commit_to_their_metadata_and_gifts_are_paid() {
        let keys = sponsor();
        let coin = Coin::new(Bytes32::new([1; 32]), keys.primary(), 1_000);
        let player = Bytes32::new([2; 32]);
        let other = Bytes32::new([3; 32]);
        let world = WorldMeta { owner: Bytes32::new([9; 32]), key: [4; 32], generation: 0 };
        let asks = [
            EveAsk { owner: player, world: None, gift: 10 },
            EveAsk { owner: player, world: Some(world), gift: 5 },
            EveAsk { owner: other, world: None, gift: 0 },
        ];
        let built = build_eve_dids_with(&keys, &[coin], &asks, 2).unwrap();
        assert_eq!(built.gifts, 15);
        assert_eq!(built.cost(), 3 + 15 + 2);
        assert_eq!(built.change, 1_000 - 20);
        let plain = built.dids[0];
        assert_eq!(plain.coin.puzzle_hash, eve_puzzle_hash(plain.launcher_id, player));
        let w = built.dids[1];
        assert_eq!(w.world, Some(world));
        assert_eq!(w.coin.puzzle_hash, eve_puzzle_hash_with(w.launcher_id, player, world.tree_hash()));
        assert_ne!(w.coin.puzzle_hash, eve_puzzle_hash(w.launcher_id, player));
        // One gift coin for the player (the sum), none for the other owner.
        let gift = plain.gift.unwrap();
        assert_eq!(w.gift, Some(gift));
        assert_eq!((gift.puzzle_hash, gift.amount, gift.parent_coin_info), (player, 15, coin.coin_id()));
        assert_eq!(built.dids[2].gift, None);
        // Not enough coins for the gifts.
        let small = Coin::new(Bytes32::new([1; 32]), keys.primary(), 19);
        let e = build_eve_dids_with(&keys, &[small], &asks, 2).unwrap_err();
        assert!(e.to_string().contains("19 of 20"), "{e}");
    }
}
