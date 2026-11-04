//! The animals mod: dogs by breed (`dog.rs`: the chihuahua), cats (`cat.rs`), grey wolves
//! (`wolf.rs`), jaguars (`jaguar.rs`), capybaras (`capybara.rs`), velociraptors of two species
//! (`raptor.rs`), Utahraptors (`utahraptor.rs`), wyverns (`wyvern.rs`, a fantasy animal, and their
//! breath: `breath.rs`), and the living world's simple wild creatures (`wild.rs`): hares of two
//! species (`hare.rs`), wood mice (`mouse.rs`), field voles (`vole.rs`), deer of two species
//! (`deer.rs`) and foxes of two species, hunters (`fox.rs`), ravens (`raven.rs`), Protoceratops in
//! herds (`protoceratops.rs`), Kryptobaatar, the Cretaceous's small mammal (`kryptobaatar.rs`), and
//! its lizard, Priscagama (`lizard.rs`); reindeer in herds (`reindeer.rs`), coyotes, hunters
//! (`coyote.rs`); birds on the ground and in the air (`birds.rs`): small birds of two species
//! (`small_bird.rs`), owls of two species (`owl.rs`), eagles and hawks (`eagle.rs`), turkey
//! vultures (`vulture.rs`) and herring gulls (`gull.rs`); caimans of two species, the water's
//! hunters (`caiman.rs`); and fish of two species, spawned into the water and swimming (`fish.rs`),
//! each placed in the tree of life (`species.rs`), and what they
//! share: species as data (`species.rs`), each animal's level and stats (`animal.rs`), taming
//! (`taming.rs`, `knockout.rs`), commands and tribes (`commands.rs`), signatures
//! (`signatures.rs`), riding (`riding.rs`), breeding and growing up (`breeding.rs`), eggs and nests (`eggs.rs`), coats from genes (`coats.rs`) that evolve where the
//! animals live (`evolution.rs`), and the screens and spyglass (`screen.rs`). The design
//! is `notes/ANIMALS.md`. An owner is kept by player key, which the engine's bridge gives
//! (`player_key`), so a tame knows its owner after a restart.

use std::sync::OnceLock;

use weft_sdk::abi::Str;
use weft_sdk::*;

declare_mod!(deps: ["base"], init: init);

mod animal;
mod birds;
mod breath;
mod breeding;
mod caiman;
mod capybara;
mod cat;
mod coats;
mod comfort;
mod commands;
mod coyote;
mod cretaceous;
mod deer;
mod dog;
mod drink;
mod eagle;
mod eggs;
mod evolution;
mod fish;
mod forage;
mod fox;
mod gateway;
mod gull;
mod habitats;
mod hare;
mod hunt;
mod jaguar;
mod knockout;
mod kryptobaatar;
mod lizard;
mod meat;
mod mouse;
mod owl;
mod protoceratops;
mod psittacosaurus;
mod raptor;
mod raven;
mod reindeer;
mod rest;
mod riding;
mod scars;
mod screen;
mod signatures;
mod small_bird;
mod species;
mod store;
mod taming;
mod utahraptor;
mod vole;
mod vulture;
mod wild;
mod wolf;
mod wyvern;
mod xp;

static HEART: Particle = Particle::named("heart");

// ------------------------------------------------------------------ the engine's bridge

type Query = unsafe extern "C" fn(query: Str, arg: Str) -> Str;
static BRIDGE: OnceLock<Query> = OnceLock::new();

/// The engine hands the mod its bridge when the mod loads.
///
/// # Safety
/// `q` must be the engine's `bridge::query`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chiacraft_bridge(q: Query) {
    let _ = BRIDGE.set(q);
}

/// A player's key: what their saves are keyed by, the same after a restart.
fn player_key(player: u64) -> String {
    BRIDGE
        .get()
        .map(|f| unsafe {
            f(Str::new("player_key"), Str::new(&player.to_string()))
                .as_str()
                .to_string()
        })
        .unwrap_or_default()
}

/// A folder in the world's save for this mod's own files (`evolution.rs`'s gene pools); empty
/// without one.
fn data_dir() -> String {
    BRIDGE
        .get()
        .map(|f| unsafe {
            f(Str::new("data_dir"), Str::new("animals"))
                .as_str()
                .to_string()
        })
        .unwrap_or_default()
}

// ------------------------------------------------------------------ registration

fn init(r: &mut Registry) {
    dog::init(r);
    // The full creatures' resting state, before they attach it.
    rest::init(r);
    cat::init(r);
    capybara::init(r);
    raptor::init(r);
    utahraptor::init(r);
    wyvern::init(r);
    wild::init(r);
    hare::init(r);
    mouse::init(r);
    vole::init(r);
    deer::init(r);
    fish::init(r);
    fox::init(r);
    raven::init(r);
    protoceratops::init(r);
    psittacosaurus::init(r);
    kryptobaatar::init(r);
    lizard::init(r);
    reindeer::init(r);
    coyote::init(r);
    birds::init(r);
    small_bird::init(r);
    owl::init(r);
    eagle::init(r);
    vulture::init(r);
    gull::init(r);
    caiman::init(r);
    wolf::init(r);
    jaguar::init(r);
    breath::init(r);
    animal::init(r);
    comfort::init(r);
    forage::init(r);
    drink::init(r);
    hunt::init(r);
    screen::init(r);
    taming::init(r);
    commands::init(r);
    signatures::init(r);
    knockout::init(r);
    riding::init(r);
    // What a carcass gives: pelts and hides for clothes (`animals_core::hides`) and meat
    // (`animals_core::meat`), one table a species.
    meat::init(r);
    for s in &species::SPECIES {
        let hide = animals_core::hides::HIDES
            .iter()
            .find(|h| h.0 == s.name)
            .map(|&(_, item, lo, hi)| Pool::new().items(item, lo, hi));
        let pools: Vec<Pool> = hide.into_iter().chain(meat::pool(s.name)).collect();
        if !pools.is_empty() {
            pools
                .into_iter()
                .fold(r.loot(&Entity::named(s.name)), |l, p| l.pool(p));
        }
    }
    coats::init(r);
    scars::init(r);
    breeding::init(r);
    xp::init(r);
    eggs::init(r);
    evolution::init(r);
    cretaceous::init(r);
    gateway::init(r);
    habitats::init(r);
    r.particle(&HEART)
        .textures(&["animals:heart"])
        .life(0.8, 1.2)
        .size(0.12, 0.16)
        .gravity(-0.4)
        .drag(0.9)
        .fullbright();
}

/// The squared distance between two points (blocks²).
pub(crate) fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

/// Knocked out: lying down (`index` into `species::SPECIES`; the raptors, the wyvern and the jaguar
/// as they rest, `rest.rs`).
fn sleep(w: &World, id: u64, _index: usize) {
    rest::sleep(w, id);
}

/// Up from resting, to follow or fight (`index` into `species::SPECIES`).
fn wake(w: &World, id: u64, index: usize) {
    match index {
        1..=6 | 33 | 34 => rest::wake(w, id),
        7..=12 | 15 | 16 | 18..=22 | 31 | 32 => wild::wake(w, id),
        _ => {}
    }
}
