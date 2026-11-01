//! Chia offers (CHIP-0008 settlement payments), for section 3.6's onboarding: **the server makes
//! the offer** (it gives a 1-mojo coin or a membership NFT and requests the price, expiring), the
//! player's wallet takes it, and the server watches the coins it offered until they are spent.
//!
//! - [`summarize`]: parse an `offer1…` text and say what it gives, asks, costs and when it
//!   expires; [`check`] asks the chain that the maker's coins are still unspent and it hasn't
//!   expired.
//! - [`build_take`]: pay what the offer requests (XCH, with NFT royalties) from the local
//!   wallet's coins, receive what it offers, sign with [`WalletKeys`] and combine with the
//!   maker's half. Offers asking for CATs, NFTs or options are refused for now (the wallet tracks
//!   no CAT or NFT coins to pay with).
//! - [`build_offer`]: give XCH (a 1-mojo coin) or an NFT the wallet holds, request XCH paid to the
//!   wallet's primary address, optionally expiring at a time.
//!
//! Built on chia-sdk-driver's action system (its own offer tests are the pattern); ported from
//! `@chia/core` `offer.ts` (`summarizeOffer`, `takeOffer`, `createOffer`). Nothing here logs a
//! spend, a signature or a key.
use super::{failed, source::ChainSource, wallet::WalletKeys};
use crate::{Error, Result};
use chia_protocol::{Bytes32, Coin, SpendBundle};
use chia_puzzle_types::{
    Memos,
    offer::{NotarizedPayment, Payment},
};
use chia_puzzles::SETTLEMENT_PAYMENT_HASH;
use chia_sdk_driver::{
    Action, AssetInfo, Id, Nft, Offer, Puzzle, Relation, RequestedPayments, SpendContext, Spends, TransferNftById,
    calculate_royalty_payments, calculate_trade_price_amounts, calculate_trade_prices, decode_offer, encode_offer,
};
use chia_sdk_types::{Condition, run_puzzle};
use clvm_traits::{FromClvm, ToClvm};
use serde::{Deserialize, Serialize};

/// What an offer gives and asks.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfferSummary {
    /// Mojos the taker receives.
    pub offered_xch: u64,
    /// CATs the taker receives: asset id (hex) and amount.
    pub offered_cats: Vec<(String, u64)>,
    /// NFTs the taker receives (launcher ids, hex).
    pub offered_nfts: Vec<String>,
    /// Mojos the taker pays the maker.
    pub requested_xch: u64,
    pub requested_cats: Vec<(String, u64)>,
    pub requested_nfts: Vec<String>,
    /// Royalties the taker pays on the offered NFTs, in mojos.
    pub royalties_xch: u64,
    /// The fee the maker's half already pays.
    pub maker_fee: u64,
    /// The coins the maker spends (hex): the offer is void once any is spent elsewhere.
    pub maker_coins: Vec<String>,
    /// `ASSERT_BEFORE_SECONDS_ABSOLUTE`: the offer can't be taken at or after this time.
    pub expires_at_seconds: Option<u64>,
    /// `ASSERT_BEFORE_HEIGHT_ABSOLUTE`.
    pub expires_at_height: Option<u32>,
}

fn sum(amount: u128) -> u64 {
    u64::try_from(amount).unwrap_or(u64::MAX)
}

fn parse(ctx: &mut SpendContext, text: &str) -> Result<Offer> {
    let bundle = decode_offer(text.trim()).map_err(failed("not an offer"))?;
    Offer::from_spend_bundle(ctx, &bundle).map_err(failed("offer"))
}

/// Parse an offer and say what it does.
pub fn summarize(text: &str) -> Result<OfferSummary> {
    let mut ctx = SpendContext::new();
    let offer = parse(&mut ctx, text)?;
    summary_of(&mut ctx, &offer)
}

fn summary_of(ctx: &mut SpendContext, offer: &Offer) -> Result<OfferSummary> {
    let offered = offer.offered_coins().amounts();
    let requested = offer.requested_payments().amounts();
    let royalties = offer
        .requested_royalty_amounts()
        .map_err(failed("royalties"))?;
    let mut s = OfferSummary {
        offered_xch: sum(offered.xch),
        offered_cats: offered
            .cats
            .iter()
            .map(|(id, a)| (super::hex32(*id), sum(*a)))
            .collect(),
        offered_nfts: offer
            .offered_coins()
            .nfts
            .keys()
            .map(|id| super::hex32(*id))
            .collect(),
        requested_xch: sum(requested.xch),
        requested_cats: requested
            .cats
            .iter()
            .map(|(id, a)| (super::hex32(*id), sum(*a)))
            .collect(),
        requested_nfts: offer
            .requested_payments()
            .nfts
            .keys()
            .map(|id| super::hex32(*id))
            .collect(),
        royalties_xch: sum(royalties.xch),
        maker_fee: offer.offered_coins().fee,
        maker_coins: offer
            .cancellable_coin_spends()
            .map_err(failed("offer"))?
            .iter()
            .map(|cs| super::hex32(cs.coin.coin_id()))
            .collect(),
        ..OfferSummary::default()
    };
    // Expiry: the maker's spends' time conditions.
    for cs in &offer.spend_bundle().coin_spends {
        let puzzle = cs
            .puzzle_reveal
            .to_clvm(&mut **ctx)
            .map_err(failed("offer"))?;
        let solution = cs.solution.to_clvm(&mut **ctx).map_err(failed("offer"))?;
        let output = run_puzzle(ctx, puzzle, solution).map_err(failed("offer"))?;
        let conditions = Vec::<Condition>::from_clvm(ctx, output).map_err(failed("offer"))?;
        for c in conditions {
            match c {
                Condition::AssertBeforeSecondsAbsolute(c) => {
                    s.expires_at_seconds = Some(s.expires_at_seconds.map_or(c.seconds, |e| e.min(c.seconds)));
                }
                Condition::AssertBeforeHeightAbsolute(c) => {
                    s.expires_at_height = Some(s.expires_at_height.map_or(c.height, |e| e.min(c.height)));
                }
                _ => {}
            }
        }
    }
    Ok(s)
}

/// Unix seconds now.
pub fn now_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Whether the offer can still be taken: not expired at `now` / `peak`, and every coin the maker
/// spends exists unspent on the chain.
pub async fn check<S: ChainSource>(source: &S, summary: &OfferSummary, now: u64, peak: Option<u32>) -> Result<()> {
    if summary.expires_at_seconds.is_some_and(|t| now >= t) {
        return Err(Error::new("the offer has expired"));
    }
    if let (Some(h), Some(peak)) = (summary.expires_at_height, peak)
        && peak >= h
    {
        return Err(Error::new("the offer has expired"));
    }
    let ids: Vec<Bytes32> = summary
        .maker_coins
        .iter()
        .map(|h| super::parse_hex32(h))
        .collect::<Result<_>>()?;
    let states = source.coin_state(ids.clone(), false).await?;
    for id in ids {
        match states.iter().find(|s| s.coin.coin_id() == id) {
            None => return Err(Error::new("a coin the offer spends isn't on chain")),
            Some(s) if s.spent_height.is_some() => {
                return Err(Error::new("the offer was already taken or cancelled"));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// Coins for at least `need` mojos, largest first (Sage's simple selection for a few coins).
pub fn select(coins: &[Coin], need: u64) -> Result<Vec<Coin>> {
    let mut sorted: Vec<Coin> = coins.to_vec();
    sorted.sort_by(|a, b| b.amount.cmp(&a.amount).then(a.coin_id().cmp(&b.coin_id())));
    let mut picked = Vec::new();
    let mut total: u128 = 0;
    for c in sorted {
        if total >= u128::from(need) && !picked.is_empty() {
            break;
        }
        total += u128::from(c.amount);
        picked.push(c);
    }
    if total < u128::from(need) || picked.is_empty() {
        return Err(Error::new(format!("not enough XCH: {need} mojos needed, {total} spendable")));
    }
    Ok(picked)
}

/// A taken offer, ready to push.
#[derive(Clone, Debug)]
pub struct Taken {
    pub bundle: SpendBundle,
    pub summary: OfferSummary,
    /// The local coins it spends.
    pub spent: Vec<Bytes32>,
}

/// Take an offer with the local wallet: pay its requested XCH and royalties (and `fee`) from
/// `coins`, receive what it offers at the wallet's primary address.
pub fn build_take(text: &str, keys: &WalletKeys, coins: &[Coin], fee: u64) -> Result<Taken> {
    let mut ctx = SpendContext::new();
    let offer = parse(&mut ctx, text)?;
    let summary = summary_of(&mut ctx, &offer)?;
    let requested = offer.requested_payments();
    if !requested.cats.is_empty() || !requested.nfts.is_empty() || !requested.options.is_empty() {
        return Err(Error::new("this wallet takes only offers that ask for XCH (no CATs, NFTs or options yet)"));
    }
    let need = summary
        .requested_xch
        .checked_add(summary.royalties_xch)
        .and_then(|n| n.checked_add(fee))
        .ok_or_else(|| Error::new("the offer's amounts overflow"))?;
    let picked = if need > 0 { select(coins, need)? } else { Vec::new() };
    let mut spends = Spends::new(keys.primary());
    spends.add(offer.offered_coins().clone());
    for c in &picked {
        spends.add(*c);
    }
    let mut actions = requested.actions();
    let royalties = offer.requested_royalties();
    if !royalties.is_empty() {
        let prices = calculate_trade_price_amounts(&requested.amounts(), royalties.len());
        let payments = calculate_royalty_payments(&mut ctx, &prices, &royalties).map_err(failed("royalties"))?;
        actions.extend(payments.actions());
    }
    if fee > 0 {
        actions.push(Action::fee(fee));
    }
    let deltas = spends.apply(&mut ctx, &actions).map_err(failed("take"))?;
    spends
        .finish_with_keys(&mut ctx, &deltas, Relation::AssertConcurrent, &keys.synthetic_keys())
        .map_err(failed("take"))?;
    let coin_spends = ctx.take();
    let signature = keys.sign(&coin_spends)?;
    Ok(Taken { bundle: offer.take(SpendBundle::new(coin_spends, signature)), summary, spent: picked.iter().map(Coin::coin_id).collect() })
}

/// What the server gives in an offer it makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfferGive {
    /// This many mojos (1: section 3.6's admission coin).
    Xch(u64),
    /// An NFT the wallet holds, by launcher id.
    Nft(Bytes32),
}

/// An offer to make.
#[derive(Clone, Copy, Debug)]
pub struct MakeOffer {
    pub give: OfferGive,
    /// Mojos requested, paid to the wallet's primary address.
    pub price: u64,
    /// Unix seconds after which it can't be taken.
    pub expires_at: Option<u64>,
    /// A fee the maker's half pays (0: the taker pays one).
    pub fee: u64,
}

/// A made offer and what to watch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MadeOffer {
    /// The `offer1…` text to send the player.
    pub text: String,
    /// The coins the offer spends (hex): once one is spent on chain, the offer was taken (or
    /// cancelled by us), since the server signs no other spend of them.
    pub offered_coins: Vec<String>,
    /// Where the price is paid (hex puzzle hash) and how much.
    pub payment_puzzle_hash: String,
    pub price: u64,
    pub expires_at: Option<u64>,
}

/// Make an offer from the wallet: `coins` are its spendable XCH, `nft` the NFT to give (fetched
/// into the same `ctx` with [`fetch_nft`]) when `spec.give` is an NFT.
pub fn build_offer(ctx: &mut SpendContext, spec: MakeOffer, keys: &WalletKeys, coins: &[Coin], nft: Option<Nft>) -> Result<MadeOffer> {
    if spec.price == 0 {
        return Err(Error::new("an offer needs a price"));
    }
    let primary = keys.primary();
    let mut spends = Spends::new(primary);
    let mut inputs = Vec::new();
    let mut actions = Vec::new();
    let mut nft_input = None;
    let xch_needed = match spec.give {
        OfferGive::Xch(amount) => {
            if amount == 0 {
                return Err(Error::new("give at least 1 mojo"));
            }
            actions.push(Action::send(Id::Xch, SETTLEMENT_PAYMENT_HASH.into(), amount, Memos::None));
            amount
        }
        OfferGive::Nft(launcher) => {
            let nft = nft
                .filter(|n| n.info.launcher_id == launcher)
                .ok_or_else(|| Error::new("the wallet doesn't hold that NFT"))?;
            if !keys.puzzle_hashes().contains(&nft.info.p2_puzzle_hash) {
                return Err(Error::new("the NFT isn't held by this wallet's keys"));
            }
            inputs.push(nft.coin.coin_id());
            nft_input = Some(nft);
            0
        }
    };
    let need = xch_needed
        .checked_add(spec.fee)
        .ok_or_else(|| Error::new("the offer's amounts overflow"))?;
    let picked = if need > 0 { select(coins, need)? } else { Vec::new() };
    inputs.extend(picked.iter().map(Coin::coin_id));
    for c in &picked {
        spends.add(*c);
    }
    let hint = ctx.hint(primary).map_err(failed("offer"))?;
    let mut requested = RequestedPayments::new();
    requested
        .xch
        .push(NotarizedPayment::new(Offer::nonce(inputs.clone()), vec![Payment::new(primary, spec.price, hint)]));
    let asset_info = AssetInfo::new();
    if let Some(nft) = nft_input {
        let id = Id::Existing(nft.info.launcher_id);
        // A royalty NFT is told the trade price, so the taker's royalty is enforced.
        let trade_prices = if nft.info.royalty_basis_points > 0 {
            calculate_trade_prices(&calculate_trade_price_amounts(&requested.amounts(), 1), &asset_info).map_err(failed("offer"))?
        } else {
            Vec::new()
        };
        spends.add(nft);
        actions.push(Action::update_nft(id, vec![], Some(TransferNftById::new(None, trade_prices))));
        actions.push(Action::send(id, SETTLEMENT_PAYMENT_HASH.into(), 1, Memos::None));
    }
    if spec.fee > 0 {
        actions.push(Action::fee(spec.fee));
    }
    let assertions = requested
        .assertions(ctx, &asset_info)
        .map_err(failed("offer"))?;
    spends.conditions.required = spends.conditions.required.clone().extend(assertions);
    if let Some(t) = spec.expires_at {
        spends.conditions.required = spends
            .conditions
            .required
            .clone()
            .assert_before_seconds_absolute(t);
    }
    let deltas = spends.apply(ctx, &actions).map_err(failed("offer"))?;
    spends
        .finish_with_keys(ctx, &deltas, Relation::AssertConcurrent, &keys.synthetic_keys())
        .map_err(failed("offer"))?;
    let coin_spends = ctx.take();
    let signature = keys.sign(&coin_spends)?;
    let offer =
        Offer::from_input_spend_bundle(ctx, SpendBundle::new(coin_spends, signature), requested, asset_info).map_err(failed("offer"))?;
    let bundle = offer.to_spend_bundle(ctx).map_err(failed("offer"))?;
    Ok(MadeOffer {
        text: encode_offer(&bundle).map_err(failed("offer"))?,
        offered_coins: inputs.iter().map(|id| super::hex32(*id)).collect(),
        payment_puzzle_hash: super::hex32(primary),
        price: spec.price,
        expires_at: spec.expires_at,
    })
}

/// Read the NFT a hinted coin holds, into `ctx`, from its parent's spend (`fetchNft`). None if the
/// coin isn't an NFT.
pub async fn fetch_nft<S: ChainSource>(source: &S, ctx: &mut SpendContext, coin: Coin, created: u32) -> Result<Option<Nft>> {
    let parent = source
        .coin_state(vec![coin.parent_coin_info], false)
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| Error::new("the NFT's parent coin is unknown to the peer"))?;
    let (puzzle, solution) = source
        .puzzle_and_solution(coin.parent_coin_info, created)
        .await?;
    let puzzle = puzzle.to_clvm(&mut **ctx).map_err(failed("NFT"))?;
    let solution = solution.to_clvm(&mut **ctx).map_err(failed("NFT"))?;
    let parsed = Puzzle::parse(ctx, puzzle);
    let nft = Nft::parse_child(ctx, parent.coin, parsed, solution).map_err(failed("NFT"))?;
    Ok(nft.filter(|n| n.coin == coin))
}
