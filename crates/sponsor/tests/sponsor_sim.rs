//! The sponsor on chia-sdk-test's peer simulator: a request launches a DID owned by the player's
//! address (spendable by the player, not by the sponsor); a second request for the same identity
//! is refused; bad signatures, stale times, other networks and bad addresses are refused; the
//! daily budget and the per-endpoint limit answer 429; requests arriving together share one
//! bundle; a restart keeps everything; no free coin answers 503; over iroh (on loopback), the
//! protocol answers as the contract says and the address stays the same across restarts.
//! Throwaway keys, no network, no real funds.
mod common;

use chia_puzzle_types::{EveProof, Proof};
use chia_sdk_driver::{Did, DidInfo, HashedPtr, SpendContext, StandardLayer};
use chia_sdk_types::Conditions;
use chiacraft_sponsor::service::now_seconds;
use chiacraft_wallet::{
    Mnemonic,
    chain::{self, Bytes32, ChainConfig, ChainNetwork, ChainReader, PushOutcome, SpendBundle, WalletKeys, sim::Sim},
    sponsor::{EXISTING, Request, read_answer},
};
use common::*;
use std::sync::Arc;

const FUNDS: u64 = 1_000_000;

fn reader(sim: &Sim) -> ChainReader {
    let mut config = ChainConfig::new(ChainNetwork::Testnet11);
    config.target_peers = 1;
    config.puzzle_hashes = vec![Bytes32::new([9; 32])];
    let r = ChainReader::start_with(config, sim.dialer()).unwrap();
    assert!(r.wait_synced(WAIT));
    r
}

/// The DID as its owner sees it: the eve coin, its proof and its info.
fn eve_did(sim: &Sim, launcher_id: Bytes32, coin_id: Bytes32, owner: Bytes32) -> Did {
    let (launcher, eve) = sim.rt.block_on(async {
        let s = sim.sim.lock().await;
        (s.coin_state(launcher_id).unwrap().coin, s.coin_state(coin_id).unwrap().coin)
    });
    Did::new(
        eve,
        Proof::Eve(EveProof { parent_parent_coin_info: launcher.parent_coin_info, parent_amount: launcher.amount }),
        DidInfo::new(launcher_id, None, 1, HashedPtr::NIL, owner),
    )
}

/// Spend `did` with `keys`' primary key as its inner puzzle.
fn spend_did(sim: &Sim, did: Did, keys: &WalletKeys) -> PushOutcome {
    let mut ctx = SpendContext::new();
    let pk = keys.synthetic_keys()[&keys.primary()];
    let _child = did
        .update(&mut ctx, &StandardLayer::new(pk), Conditions::new())
        .unwrap();
    let spends = ctx.take();
    // A spend whose reveal isn't the coin's puzzle can't even be signed: push it unsigned, for
    // the chain to judge.
    let signature = keys.sign(&spends).unwrap_or_default();
    sim.push(SpendBundle::new(spends, signature))
        .unwrap_or_else(|e| PushOutcome::Rejected { txid: Bytes32::default(), error: e.to_string() })
}

#[test]
fn a_request_launches_a_did_owned_by_the_players_address() {
    let sim = Sim::start().unwrap();
    let dir = scratch("launch");
    let words = Mnemonic::generate().unwrap();
    let s = sponsor(&sim, config(&dir), &words, FUNDS);
    let player = Player::new();
    let reply = s.request(&player.request(), &client(1)).unwrap();
    assert!(reply.did.starts_with("did:chia:1"), "{reply:?}");
    assert_eq!(reply.status, "pending");
    let launcher = chain::parse_hex32(&reply.launcher).unwrap();
    assert_eq!(chain::parse_did(&reply.did).unwrap(), launcher);
    // Paid by the sponsor: a mojo and the fee.
    assert!(until(|| s.status().balance == FUNDS - 1 - 1_000), "{:?}", s.status());

    // On chain, read by someone else: an eve DID whose puzzle names the player's address.
    let other = reader(&sim);
    let record = other.resolve_did(launcher, WAIT).unwrap().record;
    assert_eq!(record.history.len(), 1);
    assert_eq!(chain::hex32(record.current().unwrap().coin_id), reply.coin);
    assert!(record.eve_owned_by(player.p2()));
    assert!(record.owned_by(player.p2()));
    assert!(!record.owned_by(s.puzzle_hash()));

    s.check_pending();
    let state = s.did(&reply.did).unwrap();
    assert!(matches!(state.status, chiacraft_sponsor::store::Status::Confirmed { .. }), "{state:?}");

    // The sponsor can't spend it (its key isn't the DID's inner puzzle) …
    let coin = chain::parse_hex32(&reply.coin).unwrap();
    let sponsor_keys = WalletKeys::from_mnemonic(&words, ChainNetwork::Testnet11, 1);
    let attempt = spend_did(&sim, eve_did(&sim, launcher, coin, player.p2()), &sponsor_keys);
    assert!(matches!(attempt, PushOutcome::Rejected { .. }), "{attempt:?}");
    // … the player can, with nothing from the sponsor.
    let done = spend_did(&sim, eve_did(&sim, launcher, coin, player.p2()), &player.keys);
    assert!(!matches!(done, PushOutcome::Rejected { .. }), "{done:?}");
    let record = other.resolve_did(launcher, WAIT).unwrap().record;
    assert_eq!(record.history.len(), 2);
    assert_eq!(record.owner(), Some(player.p2()));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_second_request_for_the_same_identity_is_refused() {
    let sim = Sim::start().unwrap();
    let dir = scratch("twice");
    let s = sponsor(&sim, config(&dir), &Mnemonic::generate().unwrap(), FUNDS);
    let player = Player::new();
    let first = s.request(&player.request(), &client(1)).unwrap();
    let again = s.request(&player.request(), &client(2)).unwrap_err();
    assert_eq!(again.status, 403, "{again:?}");
    assert_eq!(again.did, Some((first.did.clone(), first.launcher.clone())));
    // The same identity with another address is still the same identity.
    let mut moved = Player::new();
    moved.signer = chiacraft_wallet::IdentitySigner::from_mnemonic(&player.words, 0);
    assert_eq!(s.request(&moved.request(), &client(3)).unwrap_err().status, 403);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn bad_signatures_stale_times_and_other_networks_are_refused() {
    let sim = Sim::start().unwrap();
    let dir = scratch("refusals");
    let s = sponsor(&sim, config(&dir), &Mnemonic::generate().unwrap(), FUNDS);
    let player = Player::new();
    let now = now_seconds();
    let status = |r: &Request| {
        s.request(r, &client(1))
            .map(|_| 200)
            .unwrap_or_else(|e| e.status)
    };

    // Signed by another key.
    let mut forged = player.request();
    forged.signature = Player::new().request_at(forged.time).signature;
    assert_eq!(status(&forged), 400);
    // Signed for another time or address.
    let mut retimed = player.request_at(now);
    retimed.time = now + 1;
    assert_eq!(status(&retimed), 400);
    let mut readdressed = player.request();
    readdressed.address = Player::new().address;
    assert_eq!(status(&readdressed), 400);
    // Stale or from the future, though signed.
    assert_eq!(status(&player.request_at(now - 400)), 400);
    assert_eq!(status(&player.request_at(now + 400)), 400);
    // Not a testnet11 address; not an identity key; another network.
    let mut mainnet = player.request();
    mainnet.address = chain::address(player.p2(), ChainNetwork::Mainnet);
    assert_eq!(status(&mainnet), 400);
    let mut garbled = player.request();
    garbled.identity = "00".repeat(48);
    assert_eq!(status(&garbled), 400);
    let mut other = player.request();
    other.network = "mainnet".into();
    assert_eq!(status(&other), 403);
    // None of it launched anything; the honest request still works.
    assert_eq!(s.bundles(), 0);
    assert_eq!(status(&player.request()), 200);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_daily_budget_and_the_endpoint_limit_answer_429() {
    let sim = Sim::start().unwrap();
    let dir = scratch("budget");
    let mut c = config(&dir);
    c.daily_budget_mojos = 2 * c.cost_per_did();
    c.per_endpoint_per_day = 1;
    let s = sponsor(&sim, c, &Mnemonic::generate().unwrap(), FUNDS);
    assert!(s.request(&Player::new().request(), &client(1)).is_ok());
    // The same endpoint again: its daily limit.
    let limited = s.request(&Player::new().request(), &client(1)).unwrap_err();
    assert_eq!(limited.status, 429, "{limited:?}");
    assert!(limited.error.contains("per endpoint"), "{limited:?}");
    assert!(s.request(&Player::new().request(), &client(2)).is_ok());
    assert_eq!(s.status().budget_left_today, 0);
    let spent = s.request(&Player::new().request(), &client(3)).unwrap_err();
    assert_eq!(spent.status, 429, "{spent:?}");
    assert!(spent.error.contains("budget"), "{spent:?}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn requests_arriving_together_share_one_bundle() {
    let sim = Sim::start().unwrap();
    let dir = scratch("batch");
    let mut c = config(&dir);
    c.batch_window_ms = 1_500;
    let s = Arc::new(sponsor(&sim, c, &Mnemonic::generate().unwrap(), FUNDS));
    let players: Vec<Player> = (0..5).map(|_| Player::new()).collect();
    let replies: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = players
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let s = s.clone();
                scope.spawn(move || s.request(&p.request(), &client(10 + i as u8)))
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let replies: Vec<_> = replies.into_iter().map(Result::unwrap).collect();
    assert_eq!(s.bundles(), 1);
    let launches: Vec<_> = players
        .iter()
        .map(|p| s.launch_of(&p.identity()).unwrap())
        .collect();
    assert!(launches.iter().all(|l| l.txid == launches[0].txid));
    let other = reader(&sim);
    for (p, r) in players.iter().zip(&replies) {
        let record = other
            .resolve_did(chain::parse_hex32(&r.launcher).unwrap(), WAIT)
            .unwrap()
            .record;
        assert!(record.owned_by(p.p2()), "{r:?}");
    }
    // Five mojos and five fees from the sponsor, change back to it.
    assert!(until(|| s.status().balance == FUNDS - 5 * 1_001), "{:?}", s.status());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_restart_keeps_identities_budget_and_pending_launches() {
    let sim = Sim::start().unwrap();
    let dir = scratch("restart");
    let words = Mnemonic::generate().unwrap();
    let player = Player::new();
    let first = {
        let s = sponsor(&sim, config(&dir), &words, FUNDS);
        s.request(&player.request(), &client(1)).unwrap()
    };
    let s = sponsor(&sim, config(&dir), &words, 0);
    let again = s.request(&player.request(), &client(1)).unwrap_err();
    assert_eq!((again.status, again.did.map(|d| d.1)), (403, Some(first.launcher.clone())));
    let status = s.status();
    assert_eq!(status.sponsored_today, 1);
    assert_eq!(status.budget_left_today, s.config().daily_budget_mojos - s.config().cost_per_did());
    // The launch left pending before the restart is confirmed by the new worker.
    assert!(until(|| matches!(s.did(&first.launcher).map(|d| d.status), Some(chiacraft_sponsor::store::Status::Confirmed { .. }))));
    assert!(s.request(&Player::new().request(), &client(1)).is_ok());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn no_free_coin_answers_503() {
    let sim = Sim::start().unwrap();
    let dir = scratch("unfunded");
    let s = sponsor(&sim, config(&dir), &Mnemonic::generate().unwrap(), 0);
    let r = s.request(&Player::new().request(), &client(1)).unwrap_err();
    assert_eq!(r.status, 503, "{r:?}");
    // Nothing was held for it: the budget and the identity are free again.
    assert_eq!(s.status().queued, 0);
    assert_eq!(s.status().budget_left_today, s.config().daily_budget_mojos);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn over_iroh_the_protocol_answers_as_the_contract_says() {
    let sim = Sim::start().unwrap();
    let dir = scratch("iroh");
    let served = Served::start(sponsor(&sim, config(&dir), &Mnemonic::generate().unwrap(), FUNDS));
    let ep = served.asker();
    let player = Player::new();
    // The request's JSON is exactly the contract's.
    let body = player.request().to_json();
    let fields: serde_json::Value = serde_json::from_str(&body).unwrap();
    let mut keys: Vec<_> = fields.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    assert_eq!(keys, ["address", "identity", "network", "signature", "time", "work"]);

    let (status, reply) = served.ask(&ep, &player.request());
    assert_eq!(status, 200, "{reply}");
    assert!(reply["did"].as_str().unwrap().starts_with("did:chia:1"));
    assert_eq!(reply["launcher"].as_str().unwrap().len(), 64);
    assert_eq!(reply["coin"].as_str().unwrap().len(), 64);
    assert_eq!(reply["status"], "pending");
    // The game reads it with the wallet's reader.
    let read = read_answer(status, &reply.to_string()).unwrap();
    assert_eq!(read.did, reply["did"].as_str().unwrap());

    // Again: 403 naming the DID, which the game reads as the existing one.
    let (status, again) = served.ask(&ep, &player.request());
    assert_eq!(status, 403);
    assert!(again["error"].is_string());
    assert_eq!(again["launcher"], reply["launcher"]);
    let existing = read_answer(status, &again.to_string()).unwrap();
    assert_eq!((existing.status.as_str(), existing.launcher.as_str()), (EXISTING, reply["launcher"].as_str().unwrap()));

    // Not a request; stale; too large.
    let (status, bad) = served.send(&ep, b"{\"network\": 1}").unwrap();
    assert_eq!(status, 400);
    assert!(bad["error"].as_str().unwrap().contains("not a DID request"), "{bad}");
    let stale = Player::new().request_at(now_seconds() - 3_600);
    let (status, bad) = served.ask(&ep, &stale);
    assert_eq!(status, 400);
    assert!(bad["error"].as_str().unwrap().contains("time"), "{bad}");
    let huge = vec![b' '; chiacraft_sponsor::client::MAX_REQUEST + 1];
    let (status, bad) = served.send(&ep, &huge).unwrap();
    assert_eq!(status, 400);
    assert!(bad["error"].as_str().unwrap().contains("larger"), "{bad}");

    // The status.
    let st = served.status(&ep);
    assert_eq!(st["network"], "testnet11");
    assert_eq!(st["phase"], "synced");
    assert_eq!(st["sponsored_today"], 1);
    for k in ["balance", "budget_left_today", "pending", "height", "address", "per_endpoint_per_day"] {
        assert!(!st[k].is_null(), "{k}: {st}");
    }

    // Who asked is kept only as a daily hash of its endpoint id.
    let launch = served.sponsor.launch_of(&player.identity()).unwrap();
    assert_eq!(launch.client.len(), 32);
    assert!(!launch.client.contains(&ep.id().to_string()[..8]), "the id is kept hashed");
    let log = std::fs::read_to_string(dir.join("sponsor-log.jsonl")).unwrap();
    assert!(!log.contains(&ep.id().to_string()), "no endpoint id in the log");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_endpoint_limit_counts_the_asking_endpoint_over_iroh() {
    let sim = Sim::start().unwrap();
    let dir = scratch("iroh-limit");
    let mut c = config(&dir);
    c.per_endpoint_per_day = 1;
    let served = Served::start(sponsor(&sim, c, &Mnemonic::generate().unwrap(), FUNDS));
    let ep = served.asker();
    assert_eq!(served.ask(&ep, &Player::new().request()).0, 200);
    let (status, limited) = served.ask(&ep, &Player::new().request());
    assert_eq!(status, 429, "{limited}");
    assert!(limited["error"].as_str().unwrap().contains("per endpoint"), "{limited}");
    assert_eq!(served.ask(&served.asker(), &Player::new().request()).0, 200);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_sponsors_address_stays_the_same_across_restarts() {
    let dir = scratch("key");
    let first = chiacraft_sponsor::net::load_key(&dir).unwrap();
    let again = chiacraft_sponsor::net::load_key(&dir).unwrap();
    assert_eq!(first.public(), again.public());
    let path = dir.join(chiacraft_sponsor::net::KEY_FILE);
    assert_eq!(std::fs::read(&path).unwrap().len(), 32);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    }
    // Another folder, another sponsor.
    let other = scratch("key-other");
    assert_ne!(chiacraft_sponsor::net::load_key(&other).unwrap().public(), first.public());
    // A damaged key file refuses to start rather than quietly changing the address.
    std::fs::write(&path, b"short").unwrap();
    assert!(chiacraft_sponsor::net::load_key(&dir).is_err());
    let _ = std::fs::remove_dir_all(dir);
    let _ = std::fs::remove_dir_all(other);
}

/// Research round 4, P7: identities cost nothing, so each request costs some CPU, and the day's
/// budget opens hour by hour.
#[test]
fn work_is_asked_for_and_the_budget_opens_by_the_hour() {
    let dir = scratch("work");
    let sim = Sim::start().unwrap();
    let mut config = config(&dir);
    config.work_bits = 12;
    let s = sponsor(&sim, config.clone(), &Mnemonic::generate().unwrap(), FUNDS);
    let player = Player::new();
    let lazy = player.request();
    let e = s.request(&lazy, &client(1)).unwrap_err();
    assert_eq!(e.status, 400, "{}", e.error);
    assert!(e.error.contains("12 bits"), "{}", e.error);
    let mut worked = player.request();
    worked.do_work(12).unwrap();
    assert!(s.check(&worked, worked.time).is_ok());

    let mut c = config;
    c.daily_budget_mojos = 2_400;
    c.hourly_release = true;
    assert_eq!(chiacraft_sponsor::service::released(&c, 0), 100);
    assert_eq!(chiacraft_sponsor::service::released(&c, 3600 * 11 + 5), 1_200);
    assert_eq!(chiacraft_sponsor::service::released(&c, 86_399), 2_400);
    c.hourly_release = false;
    assert_eq!(chiacraft_sponsor::service::released(&c, 0), 2_400);
}
