//! The animals mod: dogs by breed (`dog.rs`: the chihuahua), cats (`cat.rs`), capybaras
//! (`capybara.rs`), velociraptors of two species (`raptor.rs`), Utahraptors (`utahraptor.rs`) and
//! wyverns (`wyvern.rs`, a fantasy animal, and their breath: `breath.rs`),
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
mod breath;
mod breeding;
mod capybara;
mod cat;
mod coats;
mod comfort;
mod commands;
mod cretaceous;
mod dog;
mod eggs;
mod evolution;
mod forage;
mod gateway;
mod habitats;
mod hunt;
mod knockout;
mod raptor;
mod riding;
mod scars;
mod screen;
mod signatures;
mod species;
mod taming;
mod utahraptor;
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
        _ => {}
    }
}
