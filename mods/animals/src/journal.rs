//! The field journal (`animals_core::journal`): what each player has done with each species,
//! saved with them, and the entries a journal screen reads (`entries`).
//!
//! A species' entry opens when the player first sees one up close (within
//! `journal::SEEN_WITHIN`) or looks at one through the spyglass. Killing one, taming one, having
//! one born to them and riding one each fill in more. Creative reads everything.

use animals_core::journal::{self as core, BRED, ENTRIES, Fact, HUNTED, RIDDEN, SEEN, TAMED};
use weft_sdk::*;

use crate::animal::ANIMAL;
use crate::species::{SPECIES, Species};

value! {
    /// A player's deeds with each species (`core::note`: `(id << 8) | deeds`), saved.
    pub struct Journal { pub entries: [u32; 128] }
}
pub static JOURNAL: Component<Journal> = Component::named("journal");
static PLAYER: EntityKind = EntityKind::named("core:player");
/// Ticks between looks round each player (two seconds).
const EVERY: u64 = 40;

pub fn init(r: &mut Registry) {
    r.entity_component(
        &JOURNAL,
        Journal { entries: [0; ENTRIES] },
        component_flags::SAVE | component_flags::SYNC_OWNER | component_flags::KEEP_ON_RESPAWN,
    );
    r.attach(&PLAYER, &JOURNAL);
    // Seen up close.
    r.world_tick(|w| {
        if !w.game_time().is_multiple_of(EVERY) {
            return;
        }
        for player in w.players() {
            let Some(p) = w.entity(player) else { continue };
            for id in w.entities_near(p.pos, core::SEEN_WITHIN as f64, None) {
                if let Some(s) = species_of(w, id) {
                    note(w, player, s, SEEN);
                }
            }
        }
    });
    // Hunted: a player's kill.
    r.on(&actions::DIE, |b| {
        let w = b.world();
        for d in b.iter() {
            if w.entity(d.source).is_some_and(|e| e.kind == PLAYER.id())
                && let Some(s) = SPECIES.iter().find(|s| s.kind.id() == d.kind)
            {
                note(&w, d.source, s, HUNTED);
            }
        }
    });
    r.command("journal", "[action:word(list,all,clear)]", command)
        .permission(permission::GAME_MASTER);
}

/// The species of animal `id`, if it is one.
fn species_of(w: &World, id: u64) -> Option<&'static Species> {
    w.component(id, &ANIMAL)?.species()
}

/// `player` did `deed` with one of species `s` (seeing it, at least).
pub fn note(w: &World, player: u64, s: &Species, deed: u8) {
    let Some(j) = w.component_mut(player, &JOURNAL) else { return };
    if core::note(&mut j.entries, s.name, deed | SEEN) {
        weft_sdk::info!("journal: player {player} {} {}", deed_name(deed), s.name);
    }
}

/// `player` did `deed` with animal `id`.
pub fn note_animal(w: &World, player: u64, id: u64, deed: u8) {
    if let Some(s) = species_of(w, id) {
        note(w, player, s, deed);
    }
}

/// The owner of key `owner` (online) did `deed` with animal `id`.
pub fn note_owner(w: &World, owner: &[u8; 64], id: u64, deed: u8) {
    let key = String::from_utf8_lossy(owner).into_owned();
    if let Some(player) = w
        .players()
        .into_iter()
        .find(|&p| crate::player_key(p) == key)
    {
        note_animal(w, player, id, deed);
    }
}

fn deed_name(deed: u8) -> &'static str {
    core::DEEDS
        .iter()
        .find(|(d, _)| *d == deed)
        .map_or("?", |(_, n)| n)
}

/// One species' page, as the journal screen shows it.
pub struct Entry {
    pub species: &'static Species,
    /// Its entity kind (`animals:<kind>`): the model the picture draws.
    pub kind: String,
    /// What the reader has done with it (`SEEN`, `HUNTED`...); 0: not met, a silhouette.
    pub deeds: u8,
    /// Its facts, each with whether the reader knows it yet.
    pub facts: Vec<(Fact, bool)>,
}

/// The journal of the client's player: every species, by family then name. In creative every
/// page is full.
pub fn entries(view: &ClientView) -> Vec<Entry> {
    let saved = view.component(&JOURNAL).map(|j| j.entries);
    let all = SEEN | HUNTED | TAMED | BRED | RIDDEN;
    let mut out: Vec<Entry> = SPECIES
        .iter()
        .map(|s| {
            let deeds = if view.outside_survival() { all } else { saved.as_ref().map_or(0, |e| core::deeds(e, s.name)) };
            Entry {
                species: s,
                kind: format!("animals:{}", s.kind_name()),
                deeds,
                facts: core::facts(s)
                    .into_iter()
                    .map(|f| {
                        let k = core::known(deeds, &f);
                        (f, k)
                    })
                    .collect(),
            }
        })
        .collect();
    out.sort_by_key(|e| (e.species.family.name, e.species.name));
    out
}

/// `/journal list` (each species met and its deeds), `all` (every deed with every species),
/// `clear` (none): for checks.
fn command(cmd: &mut ServerCommand) {
    let (w, player) = (cmd.world(), cmd.player());
    match cmd.text("action") {
        Some("all") => {
            for s in SPECIES.iter() {
                note(&w, player, s, HUNTED | TAMED | BRED | RIDDEN);
            }
        }
        Some("clear") => {
            if let Some(j) = w.component_mut(player, &JOURNAL) {
                j.entries = [0; ENTRIES];
            }
        }
        _ => {}
    }
    let Some(j) = w.component(player, &JOURNAL) else { return };
    let mut met = 0;
    for s in SPECIES.iter() {
        let deeds = core::deeds(&j.entries, s.name);
        if deeds == 0 {
            continue;
        }
        met += 1;
        let names: Vec<&str> = core::DEEDS
            .iter()
            .filter(|(d, _)| deeds & d != 0)
            .map(|(_, n)| *n)
            .collect();
        let facts = core::facts(s);
        let known = facts.iter().filter(|f| core::known(deeds, f)).count();
        let line = format!("journal: {} {}: {known} of {} facts", s.name, names.join(","), facts.len());
        weft_sdk::info!("{line}");
        cmd.reply("command.animals.line", &[Arg::Text(&line)]);
    }
    let line = format!("journal: {met} of {} species met", SPECIES.len());
    weft_sdk::info!("{line}");
    cmd.reply("command.animals.line", &[Arg::Text(&line)]);
}
