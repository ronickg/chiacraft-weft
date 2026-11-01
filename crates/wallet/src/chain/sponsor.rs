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
use super::{failed, wallet::WalletKeys};
use crate::{Error, Result};
use chia_protocol::{Bytes32, Coin, SpendBundle};
use chia_puzzle_types::Memos;
use chia_sdk_driver::{HashedPtr, Launcher, SingletonInfo, SpendContext, StandardLayer};
use chia_sdk_types::Conditions;

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
}

impl SponsorBundle {
    /// What the batch costs the sponsor: a mojo per DID plus the fee.
    pub fn cost(&self) -> u64 {
        self.dids.len() as u64 + self.fee
    }
}

/// The eve puzzle hash of a DID launched by [`build_eve_dids`] (no recovery list, one
/// verification, no metadata) for `owner`: what the eve coin's puzzle hash must be.
pub fn eve_puzzle_hash(launcher_id: Bytes32, owner: Bytes32) -> Bytes32 {
    chia_sdk_driver::DidInfo::new(launcher_id, None, 1, HashedPtr::NIL, owner)
        .puzzle_hash()
        .into()
}

/// Launch one DID per owner from `coins` (the sponsor's spendable coins; the largest first are
/// used until they cover a mojo per DID plus `fee`), signed with `keys`.
pub fn build_eve_dids(keys: &WalletKeys, coins: &[Coin], owners: &[Bytes32], fee: u64) -> Result<SponsorBundle> {
    if owners.is_empty() {
        return Err(Error::new("no DID to launch"));
    }
    if owners.len() > MAX_BATCH {
        return Err(Error::new(format!("at most {MAX_BATCH} DIDs in one bundle")));
    }
    let need = (owners.len() as u64)
        .checked_add(fee)
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
    let mut ctx = SpendContext::new();
    let mut conditions = Conditions::new();
    let mut dids = Vec::with_capacity(owners.len());
    for (i, owner) in owners.iter().enumerate() {
        let (create, launcher) = Launcher::create_early(parent.coin_id(), i as u64);
        let launcher = launcher.with_singleton_amount(1);
        let launcher_id = launcher.coin().coin_id();
        let (launch, did) = launcher
            .create_eve_did(&mut ctx, *owner, None, 1, HashedPtr::NIL)
            .map_err(failed("DID launch"))?;
        conditions = conditions.with(create).extend(launch);
        dids.push(EveDid { launcher_id, coin: did.coin, owner: *owner });
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
    })
}
