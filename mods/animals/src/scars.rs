//! Scars: what life leaves on an animal ("Looks, part 3" in the notes). A badly hurt animal may
//! keep a scar of what hurt it (claws rake three lines, teeth leave a ring of marks, a blade a
//! long line, fire a burn, and a winged one may have its wing skin torn), kept in its genome
//! (`Genome::scars`, up to four, the oldest going) and painted where it fell (`coats::render`).
//! Scars aren't inherited.

use weft_sdk::*;

use crate::animal::ANIMAL;
use crate::coats::{self, GENOME};
use crate::species;

pub fn init(r: &mut Registry) {
    r.on(&ENTITY_HURT, |b| {
        let w = b.world();
        for h in b.iter() {
            hurt(&w, h);
        }
    });
}

/// A blow: the bigger the share of its health it took, the likelier a scar (none under a
/// twelfth, at most six in ten).
fn hurt(w: &World, h: &abi::Hurt) {
    let Some(a) = w.component(h.entity, &ANIMAL).copied() else { return };
    let Some(s) = a.species() else { return };
    let max = a.stat(species::HEALTH);
    if max <= 0.0 {
        return;
    }
    let chance = ((h.amount / max - 0.08) * 2.5).clamp(0.0, 0.6);
    if (w.random_below(1000) as f32) >= chance * 1000.0 {
        return;
    }
    let kind = match h.cause {
        damage::FIRE | damage::LAVA | damage::EXPLOSION => coats::BURN,
        damage::ATTACK => by_attacker(w, h.source),
        damage::GENERIC => coats::SLASH,
        // Falls, crushing, drowning, hunger leave nothing to see.
        _ => return,
    };
    // A winged one's thin wing skin tears as often as it scars.
    let kind = if s.flight > 0.0 && kind != coats::BURN && w.random_below(3) == 0 { coats::TEAR } else { kind };
    let place = w.random_below(256) as u8;
    if let Some(g) = w.component_mut(h.entity, &GENOME)
        && g.species != 0
    {
        coats::scar(g, kind, place);
    }
}

/// What an attacker's blow leaves: an animal's one of its attacks (teeth, claws, a tail), anyone
/// else's a blade's line.
fn by_attacker(w: &World, source: u64) -> u8 {
    let attacks = w
        .component(source, &ANIMAL)
        .and_then(|a| a.species())
        .map_or(&[][..], |s| s.attacks);
    if attacks.is_empty() {
        return coats::SLASH;
    }
    match attacks[w.random_below(attacks.len() as u32) as usize] {
        "bite" => coats::BITE,
        "claw" | "swipe" | "kick" => coats::CLAW,
        _ => coats::SLASH,
    }
}
