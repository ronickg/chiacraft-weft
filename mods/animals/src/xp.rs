//! XP for players from animals (`base_api::give_xp`, `notes/SURVIVAL.md`): taming one (by its
//! level), a young one born or hatched to its owner, and killing one (by its level and how big its
//! kind is: a Utahraptor is worth far more than a chihuahua).
//!
//! A kill is known only once its animal is gone (the death event comes after), so who struck it,
//! and its level and kind, are noted when the blow lands (`STRUCK`).

use std::collections::HashMap;
use std::sync::Mutex;

use base_api::give_xp;
use vitals::levels::xp;
use weft_sdk::*;

use crate::animal::ANIMAL;
use crate::species::SPECIES;

/// Blows players landed on animals: the animal, then who struck it, its level, its species and
/// when; dropped when it dies or a minute after.
static STRUCK: Mutex<Option<HashMap<u64, (u64, u32, usize, u64)>>> = Mutex::new(None);
/// Ticks a blow is remembered.
const REMEMBER: u64 = 1200;

pub fn init(r: &mut Registry) {
    // Noted as the damage is worked out, while the animal still is.
    r.modify(&chain::DAMAGE, &phase::FINAL, |m| {
        let q = *m.query();
        let w = m.world();
        if q.other == 0 || !w.players().contains(&q.other) {
            return;
        }
        let Some(a) = w.component(q.entity, &ANIMAL) else { return };
        let Some(i) = (a.species as usize).checked_sub(1) else { return };
        let mut struck = STRUCK.lock().unwrap();
        struck
            .get_or_insert_with(HashMap::new)
            .insert(q.entity, (q.other, a.level(), i, w.game_time()));
    });
    r.on_event(&actions::DIE, |b| {
        let w = b.world();
        let mut struck = STRUCK.lock().unwrap();
        let struck = struck.get_or_insert_with(HashMap::new);
        for d in b.iter() {
            if let Some((player, level, i, _)) = struck.remove(&d.entity)
                && player == d.source
            {
                give_xp(&w, player, xp::kill(level, danger(i)));
            }
        }
        let now = w.game_time();
        struck.retain(|_, s| now.saturating_sub(s.3) < REMEMBER);
    });
}

/// How dangerous its kind is to kill, by its size: a quarter power of its mass against a 4 kg cat
/// (a chihuahua under 1, a capybara 2, a Utahraptor over 3).
pub fn danger(species: usize) -> f32 {
    (SPECIES[species].bio.mass / 4.0).powf(0.25).max(0.5)
}

/// `player` tamed an animal of that level.
pub fn tamed(w: &World, player: u64, level: u32) {
    give_xp(w, player, xp::tame(level));
}

/// A young one was born or hatched to the owner with that key, if they're here.
pub fn raised(w: &World, owner: &[u8; 64]) {
    let key = String::from_utf8_lossy(owner).into_owned();
    if let Some(player) = w
        .players()
        .into_iter()
        .find(|&p| crate::player_key(p) == key)
    {
        give_xp(w, player, xp::RAISE);
    }
}
