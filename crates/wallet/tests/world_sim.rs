//! World DIDs (plan phase 4, slice 7) on chia-sdk-test's peer simulator: a sponsor launches a
//! world DID to the player's address with metadata naming the player's DID (checked while eve
//! by its puzzle hash), the player lists the world in their own DID's metadata (one spend, with
//! no fee and with a fee paid from the sponsor's gift), and resolving both then shows each
//! naming the other. Throwaway keys, no network.
#![cfg(feature = "sim")]

use chiacraft_wallet::{
    Mnemonic,
    chain::{
        Bytes32, ChainConfig, ChainNetwork, ChainReader, DidMetadata, DidRecord, EveAsk, OwnerMeta, PushOutcome, WalletKeys, WorldMeta,
        sim::Sim,
        world::{world_link, world_link_expecting},
    },
};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(20);
const STEP: Duration = Duration::from_millis(100);

fn reader(sim: &Sim, puzzle_hashes: Vec<Bytes32>) -> ChainReader {
    let mut config = ChainConfig::new(ChainNetwork::Testnet11);
    config.target_peers = 1;
    config.puzzle_hashes = puzzle_hashes;
    let r = ChainReader::start_with(config, sim.dialer()).unwrap();
    assert!(r.wait_synced(WAIT), "{:?}", r.status());
    r
}

fn until(mut f: impl FnMut() -> bool) -> bool {
    let end = std::time::Instant::now() + WAIT;
    while std::time::Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    f()
}

/// The DID's record once `f` holds for it (read live, again and again).
fn record_when(r: &ChainReader, launcher_id: Bytes32, f: impl Fn(&DidRecord) -> bool) -> DidRecord {
    let mut last = None;
    let ok = until(|| match r.resolve_did(launcher_id, WAIT) {
        Ok(found) => {
            let hit = found.live && f(&found.record);
            last = Some(found.record);
            hit
        }
        Err(_) => false,
    });
    assert!(ok, "the DID never got there: {last:?}");
    last.unwrap()
}

fn player() -> WalletKeys {
    WalletKeys::from_mnemonic(&Mnemonic::generate().unwrap(), ChainNetwork::Testnet11, 2)
}

#[test]
fn a_world_did_and_its_owner_did_name_each_other() {
    let sim = Sim::start().unwrap();
    let keys = player();
    let me = keys.primary();
    let r = reader(&sim, keys.puzzle_hashes());
    // The player's own DID, sponsored (plain, eve).
    let owner_did = sim.sponsor_dids(&[EveAsk::plain(me)], 0).unwrap().dids[0].launcher_id;
    // A world DID launched to the player's address, its metadata naming that DID.
    let world = WorldMeta { owner: owner_did, key: [7; 32], generation: 0 };
    let built = sim
        .sponsor_dids(&[EveAsk { owner: me, world: Some(world), gift: 0 }], 0)
        .unwrap();
    assert_eq!(built.dids[0].world, Some(world));
    let world_did = built.dids[0].launcher_id;

    // Still eve: the chain shows no metadata, but the puzzle hash commits to it and to the owner.
    let w = r.wait_did(world_did, STEP, WAIT).unwrap();
    assert!(w.is_eve());
    assert_eq!(w.metadata, None);
    assert_eq!(w.world_meta(), None);
    assert!(w.eve_owned_by_with(me, world.tree_hash()));
    assert!(!w.eve_owned_by(me), "not a plain eve DID");
    assert!(!w.eve_owned_by_with(keys.puzzle_hashes()[1], world.tree_hash()));
    assert!(!w.eve_owned_by_with(me, world.rotated([1; 32]).tree_hash()));
    let o = r.wait_did(owner_did, STEP, WAIT).unwrap();
    assert!(o.owned_by(me));
    let link = world_link_expecting(&w, &o, &world, me);
    assert!(link.world_names_owner && !link.owner_lists_world && !link.holds(), "{link:?}");
    assert!(!world_link(&w, &o).world_names_owner, "unrevealed metadata names nobody");

    // The player lists the world in their DID: its first spend (from eve), no fee.
    let listed = OwnerMeta::default().with_world(world_did).unwrap();
    let up = r
        .set_did_metadata(&keys, owner_did, &DidMetadata::Owner(listed.clone()), 0, WAIT)
        .unwrap();
    assert!(!matches!(up.push, PushOutcome::Rejected { .. }), "{:?}", up.push);
    assert_eq!(up.owner, me);
    let o = record_when(&r, owner_did, |rec| rec.owner_meta().is_some());
    assert_eq!(o.owner_meta(), Some(listed.clone()));
    assert_eq!(o.owner(), Some(me));
    assert!(o.owned_by(me));
    assert_eq!(o.current().unwrap().coin_id, up.coin_id);
    assert!(!o.melted);
    let link = world_link_expecting(&w, &o, &world, me);
    assert!(link.holds(), "{link:?}");
    // The other way round it doesn't: an owner DID isn't a world DID.
    assert!(!world_link(&o, &w).holds());

    // The world's key rotated: the world DID's first spend, from the metadata it was launched with.
    let rotated = world.rotated([8; 32]);
    let up = r
        .set_did_metadata_from(&keys, world_did, &DidMetadata::World(world), &DidMetadata::World(rotated), 0, WAIT)
        .unwrap();
    assert!(!matches!(up.push, PushOutcome::Rejected { .. }), "{:?}", up.push);
    let w = record_when(&r, world_did, |rec| rec.world_meta() == Some(rotated));
    assert_eq!(w.owner(), Some(me));
    let link = world_link(&w, &o);
    assert!(link.holds(), "{link:?}");
    assert_eq!(link.world, Some(rotated));
    // Revealed metadata wins over an expected one.
    assert_eq!(world_link_expecting(&w, &o, &world, me).world, Some(rotated));
    // A DID already spent ignores the eve metadata given: its own is read from the chain (here
    // the owner DID set to the same list again).
    let again = r
        .set_did_metadata_from(&keys, owner_did, &DidMetadata::World(world), &DidMetadata::Owner(listed.clone()), 0, WAIT)
        .unwrap();
    assert!(!matches!(again.push, PushOutcome::Rejected { .. }), "{:?}", again.push);

    // The world taken off the owner's list: a later spend (no longer eve) of the owner DID.
    let o = record_when(&r, owner_did, |rec| rec.current().is_some_and(|c| c.coin_id == again.coin_id));
    assert_eq!(o.history.len(), 3);
    assert_eq!(o.owner_meta(), Some(listed));
    let up = r
        .set_did_metadata(&keys, owner_did, &DidMetadata::Owner(OwnerMeta::default()), 0, WAIT)
        .unwrap();
    assert!(!matches!(up.push, PushOutcome::Rejected { .. }), "{:?}", up.push);
    let o2 = record_when(&r, owner_did, |rec| rec.current().is_some_and(|c| c.coin_id == up.coin_id));
    assert_eq!(o2.owner_meta(), Some(OwnerMeta::default()));
    assert_eq!(o2.history.len(), o.history.len() + 1);
    let link = world_link(&w, &o2);
    assert!(link.world_names_owner && !link.owner_lists_world && !link.holds(), "{link:?}");
}

#[test]
fn a_sponsors_gift_pays_the_owner_dids_fee() {
    let sim = Sim::start().unwrap();
    let keys = player();
    let me = keys.primary();
    let r = reader(&sim, keys.puzzle_hashes());
    let built = sim
        .sponsor_dids(&[EveAsk { owner: me, world: None, gift: 1_000 }], 0)
        .unwrap();
    let did = built.dids[0];
    assert_eq!(did.gift.map(|c| (c.puzzle_hash, c.amount)), Some((me, 1_000)));
    // The gift is the player's only coin.
    assert!(until(|| r.status().balance == 1_000), "{:?}", r.status());
    let record = r.wait_did(did.launcher_id, STEP, WAIT).unwrap();
    assert!(record.owned_by(me));
    // A fee the wallet can't pay is refused before anything is pushed.
    let e = r
        .set_did_metadata(&keys, did.launcher_id, &DidMetadata::Owner(OwnerMeta::default()), 5_000, WAIT)
        .unwrap_err();
    assert!(e.to_string().contains("at least 5000"), "{e}");
    // Listing a world (any id: the owner's side alone), paying 100 mojos of the gift.
    let listed = OwnerMeta::new(vec![Bytes32::new([3; 32])]).unwrap();
    let up = r
        .set_did_metadata(&keys, did.launcher_id, &DidMetadata::Owner(listed.clone()), 100, WAIT)
        .unwrap();
    assert!(!matches!(up.push, PushOutcome::Rejected { .. }), "{:?}", up.push);
    let rec = record_when(&r, did.launcher_id, |rec| rec.owner_meta().is_some());
    assert_eq!(rec.owner_meta(), Some(listed));
    assert_eq!(rec.owner(), Some(me));
    // The fee came out of the gift; the change is back at the primary address.
    assert!(until(|| r.status().balance == 900), "{:?}", r.status());
    assert_eq!(sim.balance(me), 900);
    // Someone else's wallet can't set it.
    let other = player();
    let r2 = reader(&sim, other.puzzle_hashes());
    assert!(
        r2.set_did_metadata(&other, did.launcher_id, &DidMetadata::Empty, 0, WAIT)
            .is_err()
    );
}
