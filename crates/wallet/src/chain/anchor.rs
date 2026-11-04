//! The achievements' daily anchor (identity row I10; [`crate::achievement`]): a server commits one
//! Merkle root of a day's credentials on chain, the simplest way the chain reader supports: one of
//! its own coins spent back to its primary address with the memos `[ANCHOR_MEMO, root]`. The new
//! coin is the anchor; anyone who knows its id checks the root by reading the spend that created
//! it ([`anchor_root`]). No DataLayer store (not built), no extra coin kinds; one transaction per
//! day, which costs only the fee the server chooses (0 on testnet11).
use super::{ChainSource, failed, wallet::WalletKeys};
use crate::{Error, Result, achievement::ANCHOR_MEMO};
use chia_protocol::{Bytes, Bytes32, Coin, CoinSpend};
use chia_sdk_driver::{Action, Id, Relation, SpendContext, Spends};
use chia_sdk_types::{Condition, run_puzzle};
use clvm_traits::{FromClvm, ToClvm};

/// The spends that put `root` on chain from `coin` (one of `keys`' coins), paying `fee`; and the
/// anchor coin they create.
pub fn build(keys: &WalletKeys, coin: Coin, root: [u8; 32], fee: u64) -> Result<(Vec<CoinSpend>, chia_bls::Signature, Coin)> {
    if coin.amount <= fee {
        return Err(Error::new("the coin doesn't cover the anchor's fee"));
    }
    let mut ctx = SpendContext::new();
    let memos = ctx
        .memos(&vec![Bytes::from(ANCHOR_MEMO.to_vec()), Bytes::from(root.to_vec())])
        .map_err(failed("anchor"))?;
    let mut spends = Spends::new(keys.primary());
    spends.add(coin);
    let amount = coin.amount - fee;
    let mut actions = vec![Action::send(Id::Xch, keys.primary(), amount, memos)];
    if fee > 0 {
        actions.push(Action::fee(fee));
    }
    let deltas = spends.apply(&mut ctx, &actions).map_err(failed("anchor"))?;
    spends
        .finish_with_keys(&mut ctx, &deltas, Relation::AssertConcurrent, &keys.synthetic_keys())
        .map_err(failed("anchor"))?;
    let coin_spends = ctx.take();
    let signature = keys.sign(&coin_spends)?;
    let anchor = Coin::new(coin.coin_id(), keys.primary(), amount);
    Ok((coin_spends, signature, anchor))
}

/// The root the anchor coin `coin_id` carries, read from the spend that created it; none while the
/// coin isn't on chain or its creation carries no anchor memo.
pub async fn anchor_root<S: ChainSource>(source: &S, coin_id: Bytes32) -> Result<Option<[u8; 32]>> {
    let Some(state) = source
        .coin_state(vec![coin_id], false)
        .await?
        .into_iter()
        .next()
    else {
        return Ok(None);
    };
    let Some(created) = state.created_height else {
        return Ok(None);
    };
    let (puzzle, solution) = source
        .puzzle_and_solution(state.coin.parent_coin_info, created)
        .await?;
    let mut ctx = SpendContext::new();
    let puzzle = puzzle.to_clvm(&mut *ctx).map_err(failed("anchor"))?;
    let solution = solution.to_clvm(&mut *ctx).map_err(failed("anchor"))?;
    let output = run_puzzle(&mut ctx, puzzle, solution).map_err(failed("anchor"))?;
    let conditions = Vec::<Condition>::from_clvm(&*ctx, output).map_err(failed("anchor"))?;
    for c in conditions {
        let Condition::CreateCoin(cc) = c else {
            continue;
        };
        if cc.puzzle_hash != state.coin.puzzle_hash || cc.amount != state.coin.amount {
            continue;
        }
        let chia_puzzle_types::Memos::Some(node) = cc.memos else {
            continue;
        };
        let Ok(memos) = Vec::<Bytes>::from_clvm(&*ctx, node) else {
            continue;
        };
        if memos.first().map(|m| m.as_ref()) == Some(ANCHOR_MEMO)
            && let Some(root) = memos
                .get(1)
                .and_then(|r| <[u8; 32]>::try_from(r.as_ref()).ok())
        {
            return Ok(Some(root));
        }
    }
    Ok(None)
}
