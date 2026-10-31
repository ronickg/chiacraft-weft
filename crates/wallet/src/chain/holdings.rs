//! What a player holds, checked on chain for admission policies (identity row I7,
//! identity-research.md 3.5): a DID, an NFT (by launcher id or by the DID that minted it), an
//! amount of a CAT. The player proves control of the holding address with an owner key
//! ([`crate::sign::OwnerSigner`], the address's synthetic key, as Sage signs by address); the
//! server reads the asset from the chain and compares its owner with that key's standard puzzle
//! hash. Every answer comes from a revealed parent spend, parsed with chia-sdk-driver and checked
//! against the coin the chain has, so a coin that merely carries the right puzzle hash (anyone
//! can make one) is never taken for the asset.
//!
//! The player's side finds which of its own coins to show: [`own_nfts`] and [`own_cat_coins`]
//! from the wallet's synced hinted coins.
use super::{
    ChainSource, failed,
    offer::fetch_nft,
    sync::{CoinRecord, WalletSync},
};
use crate::{Error, Result};
use chia_protocol::{Bytes32, Coin};
use chia_puzzles::SINGLETON_LAUNCHER_HASH;
use chia_sdk_driver::{Cat, CatInfo, Nft, Puzzle, SpendContext};
use clvm_traits::ToClvm;

/// The owner a proof key names: the standard puzzle hash of a synthetic key.
pub fn owner_of(key: &[u8; 48]) -> Option<Bytes32> {
    crate::keys::p2_of_synthetic(key).map(Bytes32::from)
}

/// Whether a resolved DID is held by the address of `key` (a DID proof's owner key).
pub fn did_owned_by_key(record: &super::DidRecord, key: &[u8; 48]) -> bool {
    owner_of(key).is_some_and(|p2| record.owned_by(p2))
}

/// An NFT coin as the chain shows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NftFacts {
    pub launcher_id: Bytes32,
    pub coin_id: Bytes32,
    /// The holder's p2 puzzle hash.
    pub owner: Bytes32,
    /// The DID the NFT is assigned to now, if any.
    pub owner_did: Option<Bytes32>,
}

/// The unspent NFT coin `coin_id`: read from its parent's spend and checked against the coin.
/// An error if the coin is spent (the NFT moved on), unknown, or not an NFT.
pub async fn nft_coin<S: ChainSource>(source: &S, coin_id: Bytes32) -> Result<NftFacts> {
    let state = source
        .coin_state(vec![coin_id], false)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| Error::new("the NFT's coin is unknown to the chain"))?;
    if state.spent_height.is_some() {
        return Err(Error::new("the NFT's coin is spent: it moved on"));
    }
    let created = state
        .created_height
        .ok_or_else(|| Error::new("the NFT's coin isn't confirmed yet"))?;
    let mut ctx = SpendContext::new();
    let nft = fetch_nft(source, &mut ctx, state.coin, created)
        .await?
        .ok_or_else(|| Error::new("that coin isn't an NFT"))?;
    Ok(NftFacts {
        launcher_id: nft.info.launcher_id,
        coin_id,
        owner: nft.info.p2_puzzle_hash,
        owner_did: nft.info.current_owner,
    })
}

/// The DID an NFT was minted with (its collection): the owner its first spend after the launch
/// assigned. None if it was minted without one (or its first coin was never spent).
pub async fn nft_minter<S: ChainSource>(
    source: &S,
    launcher_id: Bytes32,
) -> Result<Option<Bytes32>> {
    let launcher = source
        .coin_state(vec![launcher_id], false)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| Error::new("no such NFT: its launcher isn't on chain"))?;
    if launcher.coin.puzzle_hash != Bytes32::from(SINGLETON_LAUNCHER_HASH) {
        return Err(Error::new(
            "not an NFT: the coin isn't a singleton launcher",
        ));
    }
    let Some(eve) = source
        .children(launcher_id)
        .await?
        .into_iter()
        .find(|c| c.coin.amount % 2 == 1)
    else {
        return Ok(None);
    };
    let Some(spent) = eve.spent_height else {
        return Ok(None);
    };
    // The eve coin's child, as its spend made it: its ownership layer names the minter's DID.
    let (puzzle, solution) = source
        .puzzle_and_solution(eve.coin.coin_id(), spent)
        .await?;
    let mut ctx = SpendContext::new();
    let puzzle = puzzle.to_clvm(&mut *ctx).map_err(failed("NFT"))?;
    let solution = solution.to_clvm(&mut *ctx).map_err(failed("NFT"))?;
    let parsed = Puzzle::parse(&ctx, puzzle);
    let child = Nft::parse_child(&mut ctx, eve.coin, parsed, solution).map_err(failed("NFT"))?;
    Ok(child
        .filter(|n| n.info.launcher_id == launcher_id)
        .and_then(|n| n.info.current_owner))
}

/// A CAT's puzzle hash for an owner (no revocation layer: the ordinary CAT2).
pub fn cat_puzzle_hash(asset_id: Bytes32, owner: Bytes32) -> Bytes32 {
    CatInfo::new(asset_id, None, owner).puzzle_hash().into()
}

/// The amount of CAT `asset_id` in `coin_ids` held by `owner`: each coin unspent, at the CAT
/// puzzle hash of `owner`, and a real child of a CAT spend of the same asset (its parent's spend
/// parsed). Coins that fail any of that count nothing.
pub async fn cat_amount<S: ChainSource>(
    source: &S,
    asset_id: Bytes32,
    owner: Bytes32,
    coin_ids: Vec<Bytes32>,
) -> Result<u64> {
    let want = cat_puzzle_hash(asset_id, owner);
    let mut total: u64 = 0;
    for state in source.coin_state(coin_ids, false).await? {
        if state.spent_height.is_some() || state.coin.puzzle_hash != want {
            continue;
        }
        let Some(created) = state.created_height else {
            continue;
        };
        let Some(parent) = source
            .coin_state(vec![state.coin.parent_coin_info], false)
            .await?
            .into_iter()
            .next()
        else {
            continue;
        };
        let (puzzle, solution) = source
            .puzzle_and_solution(state.coin.parent_coin_info, created)
            .await?;
        let mut ctx = SpendContext::new();
        let puzzle = puzzle.to_clvm(&mut *ctx).map_err(failed("CAT"))?;
        let solution = solution.to_clvm(&mut *ctx).map_err(failed("CAT"))?;
        let parsed = Puzzle::parse(&ctx, puzzle);
        let children =
            Cat::parse_children(&mut ctx, parent.coin, parsed, solution).map_err(failed("CAT"))?;
        let real = children.unwrap_or_default().iter().any(|c| {
            c.coin == state.coin && c.info.asset_id == asset_id && c.info.p2_puzzle_hash == owner
        });
        if real {
            total = total.saturating_add(state.coin.amount);
        }
    }
    Ok(total)
}

/// The NFTs the wallet holds among its synced hinted coins: (launcher id, coin id, owner p2).
pub async fn own_nfts<S: ChainSource>(
    source: &S,
    wallet: &WalletSync,
) -> Result<Vec<(Bytes32, Bytes32, Bytes32)>> {
    let mut out = Vec::new();
    for c in wallet.unspent_hinted() {
        let Some(created) = c.created else { continue };
        let mut ctx = SpendContext::new();
        if let Some(n) = fetch_nft(source, &mut ctx, c.coin(), created).await?
            && wallet.is_ours(&n.info.p2_puzzle_hash)
        {
            out.push((n.info.launcher_id, c.id(), n.info.p2_puzzle_hash));
        }
    }
    Ok(out)
}

/// The wallet's coins of CAT `asset_id`, by owner: (owner p2, coins). No chain access: the CAT
/// puzzle hash of each of the wallet's addresses, matched against its synced hinted coins.
pub fn own_cat_coins(wallet: &WalletSync, asset_id: Bytes32) -> Vec<(Bytes32, Vec<Coin>)> {
    let hinted: Vec<CoinRecord> = wallet.unspent_hinted();
    wallet
        .puzzle_hashes
        .iter()
        .filter_map(|p2| {
            let ph = cat_puzzle_hash(asset_id, *p2);
            let coins: Vec<Coin> = hinted
                .iter()
                .filter(|c| c.puzzle_hash == ph)
                .map(CoinRecord::coin)
                .collect();
            (!coins.is_empty()).then_some((*p2, coins))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cat_puzzle_hashes_differ_by_owner_and_asset() {
        let a = Bytes32::new([1; 32]);
        let b = Bytes32::new([2; 32]);
        assert_ne!(cat_puzzle_hash(a, a), cat_puzzle_hash(a, b));
        assert_ne!(cat_puzzle_hash(a, a), cat_puzzle_hash(b, a));
        assert_eq!(owner_of(&[0; 48]), None);
    }
}
