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

use weft_sdk::*;

declare_mod!(deps: ["base"], init: init);

pub mod anim;
mod animal;
mod ankylosaur;
mod armour;
mod birds;
mod borkon;
mod boss;
mod breath;
mod breeding;
mod caiman;
mod capybara;
mod cat;
mod choose;
mod citipati;
mod coats;
mod comfort;
mod commands;
mod coyote;
mod cretaceous;
mod deer;
mod dog;
mod drenkaun;
mod drink;
mod eagle;
mod early_bird;
mod eggs;
mod evolution;
mod fish;
mod forage;
mod fox;
mod gateway;
mod group;
mod gull;
mod habitats;
mod hare;
mod hunt;
mod iguanodont;
mod jaguar;
pub mod journal;
mod journal_screen;
mod knockout;
mod kryptobaatar;
mod lizard;
mod meat;
mod memory;
mod migrate;
mod mouse;
mod owl;
mod prof;
mod protoceratops;
mod psittacosaurus;
mod raptor;
mod raven;
mod reindeer;
mod rest;
mod riding;
mod routines;
mod sauropod;
mod scars;
mod screen;
mod senses;
mod signatures;
mod small_bird;
mod species;
mod store;
mod taming;
mod utahraptor;
mod voices;
mod vole;
mod vulture;
mod wild;
mod wolf;
mod wyvern;
mod xp;

static HEART: Particle = Particle::named("heart");

// ------------------------------------------------------------------ the engine's bridge

chiacraft_bridge::install!();

/// A player's key: what their saves are keyed by, the same after a restart.
fn player_key(player: u64) -> String {
    chiacraft_bridge::ask("player_key", &player.to_string())
}

/// A folder in the world's save for this mod's own files (`evolution.rs`'s gene pools); empty
/// without one.
fn data_dir() -> String {
    chiacraft_bridge::ask("data_dir", "animals")
}

// ------------------------------------------------------------------ registration

fn init(r: &mut Registry) {
    // The goal timer first: its first row ("upkeep") is the one that logs.
    prof::init(r);
    armour::init(r);
    dog::init(r);
    for p in species::problems() {
        weft_sdk::error!("{p}");
    }
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
    ankylosaur::init(r);
    iguanodont::init(r);
    citipati::init(r);
    early_bird::init(r);
    sauropod::init(r);
    boss::init(r);
    drenkaun::init(r);
    borkon::init(r);
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
    voices::init(r);
    journal::init(r);
    journal_screen::register(r);
    breath::init(r);
    animal::init(r);
    comfort::init(r);
    forage::init(r);
    drink::init(r);
    hunt::init(r);
    anim::init(r);
    senses::init(r);
    memory::init(r);
    choose::init(r);
    group::init(r);
    migrate::init(r);
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
        // One that gives no meat leaves carcass pieces for the scavengers.
        let carcass = animals_core::meat::carcass(s.name).map(|(lo, hi)| Pool::new().items(animals_core::meat::CARCASS, lo, hi));
        let pools: Vec<Pool> = hide
            .into_iter()
            .chain(meat::pool(s.name))
            .chain(carcass)
            .collect();
        if !pools.is_empty() {
            pools
                .into_iter()
                .fold(r.loot(&EntityKind::named(s.name)), |l, p| l.pool(p));
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
pub(crate) fn dist2(a: impl Into<Vec3>, b: impl Into<Vec3>) -> f64 {
    let (a, b) = (a.into(), b.into());
    (a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)
}

/// Knocked out: lying down (the raptors, the wyvern and the jaguar as they rest, `rest.rs`).
fn sleep(w: &World, id: u64, _index: usize) {
    rest::sleep(w, id);
}

/// Up from resting, to follow or fight: whichever resting state its kind has (`rest.rs` for the
/// full creatures, `wild.rs` for the simple ones; the others have none).
fn wake(w: &World, id: u64, _index: usize) {
    rest::wake(w, id);
    wild::wake(w, id);
}

/// An entity kind's model from `models/<name>.geo.json` (Bedrock geometry, as Blockbench exports
/// it), as Weft's `model_file` did before it went: the kind's `MODEL` text component.
trait ModelFile {
    fn model_file(self, name: &str) -> Self;
}

impl ModelFile for EntityBuilder<'_> {
    fn model_file(self, name: &str) -> Self {
        let full = if name.contains(':') { name.to_string() } else { format!("{}:{name}", weft_sdk::mod_id()) };
        self.set_text(&weft_sdk::MODEL, &full)
    }
}
