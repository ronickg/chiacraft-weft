//! Listing a world (`notes/PLAN.md`, phase 5; `reports/Finding worlds.md`): a coin at the address
//! that holds the world's DID, locking a deposit, whose first memo (its hint) is [`tag`] and second
//! the world DID's launcher id. Any game asks any full node for the unspent coins hinted with the
//! tag that hold at least the deposit ([`listings`]), and keeps those whose DID checks out. The
//! owner delists by spending the coin back ([`build_delist`]), which returns the deposit.
//!
//! A 1-mojo listing would be free to spam (a full block of junk coins costs nothing while the
//! mempool isn't full), so the deposit is the floor the node filters by, and only a listing whose
//! world DID names the world and its owner both ways is shown. The tag is versioned: junk coins
//! never expire, so a later version can start clean.
use super::{ChainSource, failed, source::Refused, wallet::WalletKeys};
use crate::{Error, Result};
use chia_protocol::{Bytes, Bytes32, Coin, CoinSpend};
use chia_sdk_driver::{Action, Id, Relation, SpendContext, Spends};
use chia_sdk_types::{Condition, run_puzzle};
use clvm_traits::{FromClvm, ToClvm};
use sha2::{Digest, Sha256};

/// What the tag is the hash of.
pub const TAG_TEXT: &[u8] = b"chiacraft worlds v1";
/// The most pages [`listings`] reads (each up to 100,000 coins from an untrusted peer).
pub const MAX_PAGES: usize = 8;

/// The hint every listing carries: `sha256("chiacraft worlds v1")`.
pub fn tag() -> Bytes32 {
    Bytes32::new(Sha256::digest(TAG_TEXT).into())
}

/// The spends that list the world with DID `world_did` from `coin` (one of `keys`' coins, holding
/// at least `deposit + fee`): a coin of `deposit` mojos at `owner` (the address holding the world
/// DID) with the memos `[tag, world_did]`, the change back to the primary address. And the
/// listing coin they create.
pub fn build(
    keys: &WalletKeys,
    coin: Coin,
    owner: Bytes32,
    world_did: Bytes32,
    deposit: u64,
    fee: u64,
) -> Result<(Vec<CoinSpend>, chia_bls::Signature, Coin)> {
    if deposit == 0 {
        return Err(Error::new("a listing locks a deposit"));
    }
    let need = deposit
        .checked_add(fee)
        .ok_or_else(|| Error::new("the amounts overflow"))?;
    if coin.amount < need {
        return Err(Error::new("the coin doesn't cover the deposit and the fee"));
    }
    let mut ctx = SpendContext::new();
    let memos = ctx
        .memos(&vec![Bytes::from(tag().to_vec()), Bytes::from(world_did.to_vec())])
        .map_err(failed("listing"))?;
    let mut spends = Spends::new(keys.primary());
    spends.add(coin);
    let mut actions = vec![Action::send(Id::Xch, owner, deposit, memos)];
    if fee > 0 {
        actions.push(Action::fee(fee));
    }
    let deltas = spends
        .apply(&mut ctx, &actions)
        .map_err(failed("listing"))?;
    spends
        .finish_with_keys(&mut ctx, &deltas, Relation::AssertConcurrent, &keys.synthetic_keys())
        .map_err(failed("listing"))?;
    let coin_spends = ctx.take();
    let signature = keys.sign(&coin_spends)?;
    Ok((coin_spends, signature, Coin::new(coin.coin_id(), owner, deposit)))
}

/// The spends that delist: the listing coin spent back to the primary address, less `fee`.
pub fn build_delist(keys: &WalletKeys, listing: Coin, fee: u64) -> Result<(Vec<CoinSpend>, chia_bls::Signature)> {
    if listing.amount <= fee {
        return Err(Error::new("the deposit doesn't cover the fee"));
    }
    let mut ctx = SpendContext::new();
    let mut spends = Spends::new(keys.primary());
    spends.add(listing);
    let mut actions = vec![Action::send(Id::Xch, keys.primary(), listing.amount - fee, chia_puzzle_types::Memos::None)];
    if fee > 0 {
        actions.push(Action::fee(fee));
    }
    let deltas = spends
        .apply(&mut ctx, &actions)
        .map_err(failed("delisting"))?;
    spends
        .finish_with_keys(&mut ctx, &deltas, Relation::AssertConcurrent, &keys.synthetic_keys())
        .map_err(failed("delisting"))?;
    let coin_spends = ctx.take();
    let signature = keys.sign(&coin_spends)?;
    Ok((coin_spends, signature))
}

/// A listing as the chain shows it: its coin, the world DID it names, when it was made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Listing {
    pub coin: Coin,
    pub world_did: Bytes32,
    pub created: u32,
}

/// The listings on chain holding at least `min_amount` (the deposit): unspent coins hinted with
/// [`tag`], read page by page (at most [`MAX_PAGES`]), each one's memos read from the spend that
/// made it. The newest `max` only; coins whose memos don't name a DID are left out.
pub async fn listings<S: ChainSource>(source: &S, min_amount: u64, max: usize) -> Result<Vec<Listing>> {
    let mut states = Vec::new();
    let mut from = None;
    for _ in 0..MAX_PAGES {
        let page = match source.hinted_unspent(tag(), min_amount, from).await? {
            Ok(p) => p,
            Err(Refused::Reorg) => {
                states.clear();
                from = None;
                continue;
            }
            Err(Refused::TooManySubscriptions) => return Err(Error::new("the peer refused the request")),
        };
        states.extend(page.coin_states);
        if page.finished {
            break;
        }
        from = Some((page.height, page.header_hash));
    }
    states.retain(|s| s.spent_height.is_none() && s.created_height.is_some() && s.coin.amount >= min_amount);
    states.sort_by_key(|s| std::cmp::Reverse(s.created_height));
    states.truncate(max);
    let mut out = Vec::new();
    for s in states {
        let created = s.created_height.unwrap_or(0);
        if let Some(did) = second_memo(source, s.coin, created).await? {
            out.push(Listing { coin: s.coin, world_did: did, created });
        }
    }
    Ok(out)
}

/// The second memo of the CREATE_COIN that made `coin` (the world DID), if its first is [`tag`].
async fn second_memo<S: ChainSource>(source: &S, coin: Coin, created: u32) -> Result<Option<Bytes32>> {
    let (puzzle, solution) = source
        .puzzle_and_solution(coin.parent_coin_info, created)
        .await?;
    let mut ctx = SpendContext::new();
    let puzzle = puzzle.to_clvm(&mut *ctx).map_err(failed("listing"))?;
    let solution = solution.to_clvm(&mut *ctx).map_err(failed("listing"))?;
    let output = run_puzzle(&mut ctx, puzzle, solution).map_err(failed("listing"))?;
    let conditions = Vec::<Condition>::from_clvm(&*ctx, output).map_err(failed("listing"))?;
    for c in conditions {
        let Condition::CreateCoin(cc) = c else { continue };
        if cc.puzzle_hash != coin.puzzle_hash || cc.amount != coin.amount {
            continue;
        }
        let chia_puzzle_types::Memos::Some(node) = cc.memos else { continue };
        let Ok(memos) = Vec::<Bytes>::from_clvm(&*ctx, node) else { continue };
        if memos.first().map(|m| m.as_ref()) == Some(tag().as_ref())
            && let Some(did) = memos
                .get(1)
                .and_then(|d| <[u8; 32]>::try_from(d.as_ref()).ok())
        {
            return Ok(Some(Bytes32::new(did)));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tag_is_the_versioned_hash() {
        let want: [u8; 32] = Sha256::digest(b"chiacraft worlds v1").into();
        assert_eq!(tag(), Bytes32::new(want));
    }
}
