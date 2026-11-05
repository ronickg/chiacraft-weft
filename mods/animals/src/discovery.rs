//! The field journal's discovery (`animals_core::journal`): what each player has done with each
//! species, saved with them, and the entries the journal screen (`journal.rs`, UI's) reads
//! (`entries`).
//!
//! A species' entry opens when the player first sees one up close (within
//! `journal::SEEN_WITHIN`, in their line of sight) or looks at one through the spyglass or its
//! screen; the client then gets `NEW` (the species' journal id) for the toast. Killing one, taming
//! or riding one, and having one born to them each fill in more. Creative reads everything. The
//! journal key (I) opens the screen.

use animals_core::journal::{self as core, ALL, ENTRIES, Fact, HUNTED, SEEN};
use weft_sdk::*;

use crate::animal::ANIMAL;
use crate::species::{SPECIES, Species};

value! {
    /// A player's deeds with each species (`core::note`: `(id << 8) | deeds`), saved.
    pub struct Journal { pub entries: [u32; 128] }
}
pub static JOURNAL: Component<Journal> = Component::named("journal");
static PLAYER: EntityKind = EntityKind::named("core:player");
/// Server to client: a species' entry opened (its `journal::id`), for the toast.
pub static NEW: Channel<u32> = Channel::named("journal_new");
/// The journal screen (`journal.rs`) and its key.
pub static SCREEN: Screen = Screen::named("journal");
static KEY: Key = Key::named("journal");
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
            let eye = [p.pos.x, p.pos.y + EYE, p.pos.z];
            for id in w.entities_near(p.pos, core::SEEN_WITHIN as f64, None) {
                let Some(s) = species_of(w, id) else { continue };
                if core::deeds(
                    &w.component(player, &JOURNAL)
                        .map_or([0; ENTRIES], |j| j.entries),
                    s.name,
                ) & SEEN
                    == 0
                    && w.entity(id)
                        .is_some_and(|e| in_sight(w, eye, [e.pos.x, e.pos.y + 0.5, e.pos.z]))
                {
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
    r.key(&KEY, "I").on_press_client(|c| {
        c.screen_open(&SCREEN);
    });
}

/// A player's eyes above their feet.
const EYE: f64 = 1.62;

/// Whether no opaque block stands between `eye` and `at` (tall grass, leaves' gaps and glass
/// don't hide an animal; the ray stops at the first block, so one behind a plant counts as seen).
fn in_sight(w: &World, eye: [f64; 3], at: [f64; 3]) -> bool {
    let d = [at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len < 1.0 {
        return true;
    }
    let dir = [d[0] / len, d[1] / len, d[2] / len];
    w.raycast(eye, dir, len - 0.5, false)
        .is_none_or(|hit| !weft_sdk::block_info(hit.state).is_some_and(|b| b.opaque()))
}

/// The species of animal `id`, if it is one.
fn species_of(w: &World, id: u64) -> Option<&'static Species> {
    w.component(id, &ANIMAL)?.species()
}

/// `player` did `deed` with one of species `s` (seeing it, at least).
pub fn note(w: &World, player: u64, s: &Species, deed: u8) {
    let Some(j) = w.component_mut(player, &JOURNAL) else { return };
    let opens = core::deeds(&j.entries, s.name) & SEEN == 0;
    if core::note(&mut j.entries, s.name, deed | SEEN) {
        weft_sdk::info!("journal: player {player} {} {}", deed_name(deed), s.name);
        if opens {
            w.send(player, &NEW, &core::id(s.name));
        }
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
    /// What the reader has done with it (`SEEN`, `HUNTED`, `TAMED`, `BRED`); 0: not met, a
    /// silhouette.
    pub deeds: u8,
    /// Its facts, each with whether the reader knows it yet.
    pub facts: Vec<(Fact, bool)>,
}

/// The journal of the client's player: every species, by family then name. In creative every
/// page is full.
pub fn entries(view: &ClientView) -> Vec<Entry> {
    let saved = view.component(&JOURNAL).map(|j| j.entries);
    let all = ALL;
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
                note(&w, player, s, ALL);
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
