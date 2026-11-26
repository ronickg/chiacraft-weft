//! The dry season's migration (creature AI step 9, slice 5; `animals_core::groups`, "migration"):
//! a grazing herd's leader in its place's dry season (`vitals::climate::dry_season`, by its
//! biome's rain) looks round now and then for wetter ground and leads its herd there a leg at a
//! time; the members keep to their places round it (`group.rs`), and drink and graze with it on
//! the way. Only its wander gives way: water, food, rest and fear still go first (`choose.rs`).
//! What a thirsty herd does on the way, going to water it remembers, is slice 4 (`drink::trek`).

use std::sync::Mutex;

use animals_core::groups::{self, GROUP, MIGRATE_ARRIVED, MIGRATE_EVERY, MIGRATE_GIVE_UP, MIGRATE_LEG};
use weft_sdk::*;

use crate::senses::SENSES;

/// Where it leads its herd (x, z), the tick it gives up, and the tick it next looks round (0: not
/// migrating). Not saved: a reloaded herd looks again.
static MIGRATE: Component<[f64; 4]> = Component::named("animal_migrate");

/// Testing (`/animalprof wetter`): every place counts as in its dry season, rain on a tenth of
/// its days, but round (x, z) within the radius, where it rains on 70 %.
static WETTER: Mutex<Option<[f64; 3]>> = Mutex::new(None);

pub fn init(r: &mut Registry) {
    r.entity_component(&MIGRATE, [f64::NAN, 0.0, 0.0, 0.0], 0);
    for s in &crate::species::SPECIES {
        r.edit(s.kind).attach(&MIGRATE);
    }
}

/// Testing: wet ground round (x, z) within `r`, dry everywhere else (None: the real climate).
pub fn set_wetter(at: Option<[f64; 3]>) {
    *WETTER.lock().unwrap() = at;
}

/// Testing: `id` looks round at its next wander, and stops any migration under way.
pub fn look_now(w: &World, id: u64) {
    if let Some(c) = w.component_mut(id, &MIGRATE) {
        *c = [f64::NAN, 0.0, 0.0, 0.0];
    }
}

/// Its migration now.
fn set(w: &World, id: u64, m: [f64; 4]) {
    if let Some(c) = w.component_mut(id, &MIGRATE) {
        *c = m;
    }
}

/// The share of this season's days it rains at (x, z) near height `y`, and whether that is its
/// dry season there; None: no climate (another dimension's, or no seasons there).
fn rain_at(w: &World, x: f64, y: f64, z: f64) -> Option<(f32, bool)> {
    if let Some([cx, cz, r]) = *WETTER.lock().unwrap() {
        let wet = (x - cx).powi(2) + (z - cz).powi(2) <= r * r;
        return Some(if wet { (0.7, false) } else { (0.1, true) });
    }
    let biome = w.biome([x.floor() as i32, y.floor() as i32, z.floor() as i32]);
    let c = base_api::CLIMATE.on_biome(biome)?;
    let season = crate::breeding::season_bit(w).map(|bit| bit.trailing_zeros() as usize)?;
    Some((vitals::climate::rain_share(c.rain, Some(season)), vitals::climate::dry_season(c.rain, Some(season))))
}

/// Whether `id` is leading its herd on a migration now.
pub fn migrating(w: &World, id: u64) -> bool {
    w.component(id, &MIGRATE).is_some_and(|m| !m[0].is_nan())
}

/// The needs band's goal (`choose::Need::Migrate`): a leader on a migration walks it on, a member
/// whose leader is on one keeps to its place round it (and so goes along).
pub fn goal(speed: f32) -> impl weft_sdk::ai::Goal {
    move |w: &World, id: u64, st: &EntityState| {
        if migrating(w, id) {
            return lead(w, id, st, speed).is_some();
        }
        if crate::group::leader_doing(w, id) == Some(crate::choose::Need::Migrate as u8) {
            crate::group::follow(w, id, st, speed);
            return true;
        }
        false
    }
}

/// Whether `id` is a wild grazing herd's leader (a plant eater that lives in groups, with kin).
fn leads_a_herd(w: &World, id: u64) -> bool {
    let Some(s) = w.component(id, &SENSES) else { return false };
    s.kin >= GROUP
        && s.leader == id
        && !crate::animal::tamed(w, id)
        && w.component(id, &crate::animal::ANIMAL)
            .and_then(|a| a.species())
            .is_some_and(|sp| sp.forages_wild() && sp.bio.social != animals_core::biology::Social::Solitary)
}

/// For a herd's leader, in place of its wander (`group::or_follow`): whether it walks on toward
/// wetter ground now. None: it isn't migrating (it wanders).
pub fn lead(w: &World, id: u64, st: &EntityState, speed: f32) -> Option<bool> {
    let mut m = w.component(id, &MIGRATE)?;
    let now = w.game_time() as f64;
    let migrating = !m[0].is_nan();
    if !migrating && now < m[3] {
        return None;
    }
    // Not a herd's leader (now): it looks again in a while (a lone one, a member, a wolf).
    if !leads_a_herd(w, id) {
        set(w, id, [f64::NAN, 0.0, 0.0, now + MIGRATE_EVERY as f64]);
        return None;
    }
    let (x, z) = (st.pos.x, st.pos.z);
    if migrating {
        let (dx, dz) = (m[0] - x, m[1] - z);
        let d = (dx * dx + dz * dz).sqrt();
        let turned = rain_at(w, x, st.pos.y, z).is_none_or(|(_, dry)| !dry);
        if d <= MIGRATE_ARRIVED as f64 || turned || now >= m[2] {
            let why = if d <= MIGRATE_ARRIVED as f64 {
                "arrives"
            } else if turned {
                "is out of the dry"
            } else {
                "gives up"
            };
            weft_sdk::info!("animal {id} {why}: its herd's migration ends at {x:.0} {:.0} {z:.0} ({d:.0} blocks short)", st.pos.y);
            set(w, id, [f64::NAN, 0.0, 0.0, now + MIGRATE_EVERY as f64]);
            return None;
        }
        if w.navigating(id) {
            return Some(true);
        }
        // Its herd catches up before the next leg (itself among their middle).
        if let Some(k) = w.component(id, &SENSES).filter(|k| k.kin >= GROUP) {
            let behind = ((k.kin_mid[0] as f64 - x).powi(2) + (k.kin_mid[2] as f64 - z).powi(2)).sqrt();
            if behind > groups::MIGRATE_WAIT as f64 {
                return Some(true);
            }
        }
        weft_sdk::info!("animal {id} leads its herd on: {d:.0} blocks to go, from {x:.0} {z:.0}");
        // The next leg: up to `MIGRATE_LEG` blocks on, at the ground there.
        let step = (MIGRATE_LEG as f64).min(d) / d;
        let (tx, tz) = (x + dx * step, z + dz * step);
        let y = crate::comfort::ground(w, tx.floor() as i32, st.pos.y.floor() as i32, tz.floor() as i32).map_or(st.pos.y, f64::from);
        w.navigate(id, [tx, y, tz], speed);
        return Some(true);
    }
    // Looking round.
    m[3] = now + MIGRATE_EVERY as f64 + w.random_below(200) as f64;
    let found = rain_at(w, x, st.pos.y, z).and_then(|(here, dry)| {
        if !dry {
            return None;
        }
        // Only toward ground near water its herd knows (`groups::WATER_RANGE`): a dry season's
        // migration follows the water (ai1124_savanna3: led out past its last pool, a herd died
        // parched).
        let kind = w
            .component(id, &crate::animal::ANIMAL)
            .and_then(|a| a.species())
            .map(|s| s.kind)?;
        let pos = st.pos.to_array();
        let looks = groups::migrate_looks(x as f32, z as f32)
            .filter(|&[px, pz]| {
                crate::drink::herd_water_from(w, id, kind, pos, [px as f64, pz as f64]).is_some_and(|d| d <= groups::WATER_RANGE as f64)
            })
            .filter_map(|[px, pz]| rain_at(w, px as f64, st.pos.y, pz as f64).map(|(r, _)| ([px, pz], r)));
        groups::wetter(here, looks).map(|to| (to, here))
    });
    if let Some(([tx, tz], here)) = found {
        let there = rain_at(w, tx as f64, st.pos.y, tz as f64).map_or(0.0, |r| r.0);
        weft_sdk::info!(
            "animal {id} leads its herd out of the dry season: from {x:.0} {z:.0} (rain {:.0}% of days) toward {tx:.0} {tz:.0} ({:.0}%)",
            here * 100.0,
            there * 100.0
        );
        m = [tx as f64, tz as f64, now + MIGRATE_GIVE_UP as f64, m[3]];
    }
    set(w, id, m);
    found.map(|_| false)
}
