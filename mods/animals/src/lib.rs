//! The animals mod: dogs by breed (`dog.rs`: the chihuahua), cats (`cat.rs`), capybaras
//! (`capybara.rs`), velociraptors of two species (`raptor.rs`), Utahraptors (`utahraptor.rs`),
//! wyverns (`wyvern.rs`, a fantasy animal, and their breath: `breath.rs`), and the living world's
//! simple wild creatures (`wild.rs`): hares of two species (`hare.rs`), wood mice (`mouse.rs`),
//! field voles (`vole.rs`), deer of two species (`deer.rs`) and foxes of two species, hunters
//! (`fox.rs`), ravens (`raven.rs`), Protoceratops in herds
//! (`protoceratops.rs`), Kryptobaatar, the Cretaceous's small mammal (`kryptobaatar.rs`), and its lizard,
//! Priscagama (`lizard.rs`); reindeer in herds (`reindeer.rs`), coyotes, hunters (`coyote.rs`); birds on the ground and in the air (`birds.rs`): small birds of two species (`small_bird.rs`), owls of two species (`owl.rs`), eagles and hawks (`eagle.rs`); and fish
//! of two species, spawned into the water and swimming (`fish.rs`),
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
mod capybara;
mod cat;
mod coats;
mod comfort;
mod commands;
mod coyote;
mod cretaceous;
mod deer;
mod eagle;
mod dog;
mod eggs;
mod evolution;
mod fish;
mod forage;
mod fox;
mod gateway;
mod habitats;
mod hare;
mod hunt;
mod knockout;
mod kryptobaatar;
mod lizard;
mod mouse;
mod owl;
mod protoceratops;
mod raptor;
mod raven;
mod reindeer;
mod riding;
mod scars;
mod screen;
mod signatures;
mod small_bird;
mod species;
mod taming;
mod utahraptor;
mod vole;
mod wild;
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
    kryptobaatar::init(r);
    lizard::init(r);
    reindeer::init(r);
    coyote::init(r);
    birds::init(r);
    small_bird::init(r);
    owl::init(r);
    eagle::init(r);
    breath::init(r);
    animal::init(r);
    comfort::init(r);
    forage::init(r);
    hunt::init(r);
    screen::init(r);
    taming::init(r);
    commands::init(r);
    signatures::init(r);
    knockout::init(r);
    riding::init(r);
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

/// Knocked out: lying down (`index` into `species::SPECIES`; the raptors and the wyvern).
fn sleep(w: &World, id: u64, index: usize) {
    match index {
        3 | 5 => raptor::sleep(w, id),
        4 => utahraptor::sleep(w, id),
        6 => wyvern::sleep(w, id),
        _ => {}
    }
}

/// Up from resting, to follow or fight (`index` into `species::SPECIES`).
fn wake(w: &World, id: u64, index: usize) {
    match index {
        1 => cat::wake(w, id),
        2 => capybara::wake(w, id),
        3 | 5 => raptor::wake(w, id),
        4 => utahraptor::wake(w, id),
        6 => wyvern::wake(w, id),
        7..=12 | 15 | 16 | 18..=22 => wild::wake(w, id),
        _ => {}
    }
}
