//! World DIDs from the sponsor (plan phase 4, slice 7) on chia-sdk-test's peer simulator: a world
//! request for an owner whose DID the sponsor launched gets a world DID whose eve puzzle hash
//! commits to the world's metadata, and a gift coin at the owner's address; an owner DID held
//! elsewhere (or no DID at all) is refused with 403; the ninth world DID of an identity's day is
//! 429; the same world key twice names its DID (403, read as the existing one), also after a
//! restart; a player request is still served after world requests. Throwaway keys, no network,
//! no real funds.
mod common;

use chiacraft_sponsor::{Config, Sponsor, service::now_seconds, store::Kind, store::WorldRecord};
use chiacraft_wallet::{
    Mnemonic,
    chain::{self, Bytes32, ChainConfig, ChainNetwork, ChainReader, WalletKeys, WorldMeta, sim::Sim, sponsor::eve_puzzle_hash_with},
    sponsor::{EXISTING, Request, WorldAsk, read_answer},
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

/// A sponsor funded with several coins (each bundle holds its coins until the chain confirms it),
/// synced.
fn funded(sim: &Sim, config: Config, words: &Mnemonic) -> Sponsor {
    let keys = WalletKeys::from_mnemonic(words, ChainNetwork::Testnet11, 1);
    for _ in 0..4 {
        sim.pay(keys.primary(), FUNDS).unwrap();
    }
    sponsor(sim, config, words, 0)
}

/// A fresh world key, as iroh writes it: its 32 bytes in hex.
fn world_key() -> String {
    chain::hex32(Bytes32::new(*iroh::SecretKey::generate().public().as_bytes()))
}

/// `player`'s request for a world DID: owned by `owner` (a DID launcher id), with `key`.
fn world_request(player: &Player, owner: Bytes32, key: &str, generation: u32) -> Request {
    let world = WorldAsk { owner_did: chain::hex32(owner), key: key.to_owned(), generation };
    Request::sign_world(&player.signer, ChainNetwork::Testnet11.name(), &player.address, now_seconds(), world).unwrap()
}

fn status_of(s: &Sponsor, r: &Request, n: u8) -> u16 {
    s.request(r, &client(n))
        .map(|_| 200)
        .unwrap_or_else(|e| e.status)
}

#[test]
fn a_world_did_is_launched_to_the_owner_with_its_metadata_and_a_gift() {
    let sim = Sim::start().unwrap();
    let dir = scratch("world-launch");
    let s = funded(&sim, config(&dir), &Mnemonic::generate().unwrap());
    let gift = s.config().gift_mojos;
    assert_eq!(gift, 2_000);
    let player = Player::new();
    // The owner's DID, from this sponsor.
    let owner = s.request(&player.request(), &client(1)).unwrap();
    let owner_id = chain::parse_hex32(&owner.launcher).unwrap();

    let key = world_key();
    let reply = s
        .request(&world_request(&player, owner_id, &key, 0), &client(1))
        .unwrap();
    assert_eq!(reply.status, "pending");
    assert!(reply.did.starts_with("did:chia:1"), "{reply:?}");
    assert_ne!(reply.launcher, owner.launcher);
    let launcher = chain::parse_hex32(&reply.launcher).unwrap();

    // On chain, read by someone else: an eve DID at the owner's address whose puzzle hash
    // commits to the world's metadata (the owner's DID, the key, the generation).
    let meta = WorldMeta { owner: owner_id, key: chain::parse_hex32(&key).unwrap().to_bytes(), generation: 0 };
    let other = reader(&sim);
    let record = other.resolve_did(launcher, WAIT).unwrap().record;
    let eve = record.current().unwrap();
    assert_eq!(chain::hex32(eve.coin_id), reply.coin);
    assert_eq!(eve.puzzle_hash, eve_puzzle_hash_with(launcher, player.p2(), meta.tree_hash()));
    assert!(record.eve_owned_by_with(player.p2(), meta.tree_hash()));
    assert!(!record.eve_owned_by(player.p2()), "not a plain DID");
    let rotated = meta.rotated([1; 32]);
    assert!(!record.eve_owned_by_with(player.p2(), rotated.tree_hash()));

    // The gift: a coin of gift_mojos at the owner's address.
    assert!(
        until(|| sim
            .unspent(player.p2())
            .iter()
            .any(|c| c.puzzle_hash == player.p2() && c.amount == gift)),
        "{:?}",
        sim.unspent(player.p2())
    );
    // Paid by the sponsor: two DIDs' mojos and fees, and the gift.
    assert!(until(|| s.status().balance == 4 * FUNDS - 2 * 1_001 - gift), "{:?}", s.status());

    // Kept as a world's launch; the identity's player DID is still its own.
    let l = s.world_launch(&key).unwrap();
    assert_eq!(l.kind, Kind::World);
    assert_eq!(l.world, Some(WorldRecord { key: key.clone(), owner_did: owner.launcher.clone(), generation: 0 }));
    assert_eq!(l.cost, s.config().cost_per_world());
    assert_eq!(s.launch_of(&player.identity()).unwrap().launcher, owner.launcher);
    let log = std::fs::read_to_string(dir.join("sponsor-log.jsonl")).unwrap();
    assert!(log.contains(r#""kind":"world""#), "{log}");
    assert!(log.contains(&key), "{log}");

    // The status: the world settings, and both launches against the budget.
    let st = s.status();
    assert_eq!((st.gift_mojos, st.worlds_per_identity_per_day), (gift, 8));
    assert_eq!(st.sponsored_today, 2);
    assert_eq!(st.budget_left_today, s.config().daily_budget_mojos - s.config().cost_per_did() - s.config().cost_per_world());
    s.check_pending();
    assert!(matches!(s.did(&reply.did).unwrap().status, chiacraft_sponsor::store::Status::Confirmed { .. }));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_world_request_naming_a_did_held_elsewhere_is_refused() {
    let sim = Sim::start().unwrap();
    let dir = scratch("world-elsewhere");
    let s = funded(&sim, config(&dir), &Mnemonic::generate().unwrap());
    let player = Player::new();
    let stranger = Player::new();

    // Another address's DID: 403.
    let theirs = sim.sponsor_did(stranger.p2()).unwrap();
    let e = s
        .request(&world_request(&player, theirs, &world_key(), 0), &client(1))
        .unwrap_err();
    assert_eq!(e.status, 403, "{e:?}");
    assert!(e.error.contains("isn't held"), "{e:?}");
    assert_eq!(e.did, None);
    // No DID at all: 403 too.
    let e = s
        .request(&world_request(&player, Bytes32::new([7; 32]), &world_key(), 0), &client(1))
        .unwrap_err();
    assert_eq!(e.status, 403, "{e:?}");
    assert!(e.error.contains("owner_did"), "{e:?}");

    // Malformed fields, though signed: 400; a world field changed after signing: 400.
    let mine = sim.sponsor_did(player.p2()).unwrap();
    assert_eq!(status_of(&s, &world_request(&player, mine, &"zz".repeat(32), 0), 1), 400);
    assert_eq!(status_of(&s, &world_request(&player, mine, &"ab".repeat(31), 0), 1), 400);
    let mut short = world_request(&player, mine, &world_key(), 0);
    short.world.as_mut().unwrap().owner_did = "ab".repeat(33);
    assert_eq!(status_of(&s, &short, 1), 400);
    let mut tampered = world_request(&player, mine, &world_key(), 0);
    tampered.world.as_mut().unwrap().generation = 1;
    assert_eq!(status_of(&s, &tampered, 1), 400);
    // A player's signature with a world grafted on: 400.
    let mut grafted = player.request();
    grafted.world = world_request(&player, mine, &world_key(), 0).world;
    assert_eq!(status_of(&s, &grafted, 1), 400);

    // None of it launched or spent anything.
    assert_eq!(s.bundles(), 0);
    assert_eq!(s.status().budget_left_today, s.config().daily_budget_mojos);
    // The player's own DID (not this sponsor's: any DID the address holds) is served.
    let reply = s
        .request(&world_request(&player, mine, &world_key(), 0), &client(1))
        .unwrap();
    assert_eq!(reply.status, "pending");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_ninth_world_did_of_the_day_is_429_and_a_player_did_still_comes() {
    let sim = Sim::start().unwrap();
    let dir = scratch("world-limit");
    let mut c = config(&dir);
    c.batch_window_ms = 1_500;
    let s = Arc::new(funded(&sim, c, &Mnemonic::generate().unwrap()));
    let player = Player::new();
    let owner = sim.sponsor_did(player.p2()).unwrap();
    let keys: Vec<String> = (0..8).map(|_| world_key()).collect();
    let replies: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = keys
            .iter()
            .enumerate()
            .map(|(i, key)| {
                let s = s.clone();
                let req = world_request(&player, owner, key, 0);
                scope.spawn(move || s.request(&req, &client(10 + i as u8)))
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let replies: Vec<_> = replies.into_iter().map(Result::unwrap).collect();
    let mut launchers: Vec<_> = replies.iter().map(|r| r.launcher.clone()).collect();
    launchers.sort();
    launchers.dedup();
    assert_eq!(launchers.len(), 8);

    // The ninth: 429.
    let ninth = s
        .request(&world_request(&player, owner, &world_key(), 0), &client(30))
        .unwrap_err();
    assert_eq!(ninth.status, 429, "{ninth:?}");
    assert!(ninth.error.contains("world DIDs per identity per day"), "{ninth:?}");
    // One of the eight asked again is still named (403), not limited.
    let again = s
        .request(&world_request(&player, owner, &keys[0], 0), &client(31))
        .unwrap_err();
    assert_eq!((again.status, again.did.map(|d| d.1)), (403, Some(replies[0].launcher.clone())));

    // The worlds didn't use up the identity's player DID.
    let mine = s.request(&player.request(), &client(32)).unwrap();
    assert_eq!(s.launch_of(&player.identity()).unwrap().launcher, mine.launcher);
    assert_eq!(s.status().sponsored_today, 9);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_same_world_key_twice_names_its_did_also_after_a_restart() {
    let sim = Sim::start().unwrap();
    let dir = scratch("world-twice");
    let words = Mnemonic::generate().unwrap();
    let player = Player::new();
    let owner = sim.sponsor_did(player.p2()).unwrap();
    let key = world_key();
    let first = {
        let served = Served::start(funded(&sim, config(&dir), &words));
        let ep = served.asker();
        let req = world_request(&player, owner, &key, 0);
        // The request's JSON carries the world as the contract says.
        let fields: serde_json::Value = serde_json::from_str(&req.to_json()).unwrap();
        assert_eq!(fields["world"]["owner_did"], chain::hex32(owner));
        assert_eq!(fields["world"]["key"], key.as_str());
        assert_eq!(fields["world"]["generation"], 0);

        let (status, reply) = served.ask(&ep, &req);
        assert_eq!(status, 200, "{reply}");
        let first = read_answer(status, &reply.to_string()).unwrap();
        // Again (another generation, the key in capitals): 403 naming the same DID, which the
        // game reads as the existing one.
        for again in [world_request(&player, owner, &key, 1), world_request(&player, owner, &key.to_uppercase(), 0)] {
            let (status, body) = served.ask(&ep, &again);
            assert_eq!(status, 403, "{body}");
            assert_eq!(body["launcher"], reply["launcher"]);
            assert!(body["error"].as_str().unwrap().contains("world"), "{body}");
            let existing = read_answer(status, &body.to_string()).unwrap();
            assert_eq!((existing.status.as_str(), existing.launcher.as_str()), (EXISTING, first.launcher.as_str()));
        }
        // The status names the world settings.
        let st = served.status(&ep);
        assert_eq!(st["worlds_per_identity_per_day"], 8);
        assert_eq!(st["gift_mojos"], 2_000);
        assert_eq!(st["sponsored_today"], 1);
        first
    };

    // A restart replays the log: the key is still taken.
    let s = sponsor(&sim, config(&dir), &words, 0);
    let e = s
        .request(&world_request(&player, owner, &key, 0), &client(1))
        .unwrap_err();
    assert_eq!((e.status, e.did.map(|d| d.1)), (403, Some(first.launcher.clone())));
    assert_eq!(s.world_launch(&key).unwrap().kind, Kind::World);
    // A player request after the world's is served.
    assert!(s.request(&player.request(), &client(1)).is_ok());
    assert_eq!(s.status().sponsored_today, 2);
    let _ = std::fs::remove_dir_all(dir);
}
