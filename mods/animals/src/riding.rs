//! Riding (`notes/ANIMALS.md`, "Riding"): a tame of a rideable species, saddled with its saddle
//! (right click it with one), grown up, with a bond of 25 or more, is ridden by its owner or a
//! tribe member: an empty hand gets on. The engine seats the rider (`World::mount`) and reads
//! their movement keys; here they steer it: walking it faces where the rider looks, it runs while
//! sprinting and jumps higher. Sneaking gets off beside it.

use weft_sdk::*;

use crate::animal::ANIMAL;
use crate::species::SPECIES;

/// A tame's saddle (1: saddled), saved.
pub static SADDLED: Component<u32> = Component::named("saddled");
/// Being ridden (1), to undo the ride's jump once the rider is gone (not saved).
static RIDDEN: Component<u32> = Component::named("ridden");
static TOOLS: CreativeTab = CreativeTab::named("base:tools");
static CRAFTING: RecipeKind = RecipeKind::named("base:crafting");
static POOF: Particle = Particle::named("core:poof");
/// Each rideable species' saddle, by species name (made once).
static SADDLES: std::sync::OnceLock<Vec<(&'static str, &'static Item)>> = std::sync::OnceLock::new();

/// The bond riding needs.
const BOND: u8 = 25;
/// Running speed over walking, and how much higher it jumps ridden.
const SPRINT: f32 = 1.7;
const JUMP: f64 = 0.5;

pub fn init(r: &mut Registry) {
    r.entity_component(&SADDLED, 0u32, component_flags::SAVE | component_flags::SYNC_MENU);
    r.entity_component(&RIDDEN, 0u32, 0);
    let mut saddles = Vec::new();
    for s in SPECIES.iter().filter(|s| s.seat > 0.0) {
        r.attach(s.kind, &SADDLED);
        r.attach(s.kind, &RIDDEN);
        let name = format!("{}_saddle", s.kind_name());
        let saddle = Item::generated(name.clone());
        r.item(saddle).max_stack(1).tab(&TOOLS);
        r.shaped(&CRAFTING, &name, 1, &["lll", "s s"], &[('l', "base:leather"), ('s', "base:string")]);
        saddles.push((s.kind_name(), saddle));
    }
    let _ = SADDLES.set(saddles);
}

/// Its species' saddle (an item id), if it can be ridden.
fn saddle_of(w: &World, id: u64) -> Option<u32> {
    let s = w.component(id, &ANIMAL)?.species()?;
    SADDLES.get()?.iter().find(|(n, _)| *n == s.kind_name()).map(|(_, i)| i.id())
}

/// Takes its saddle off and drops it beside it (it is let go).
pub fn unsaddle(w: &World, id: u64) {
    let (Some(saddle), Some(st)) = (saddle_of(w, id), w.entity(id)) else { return };
    if let Some(s) = w.component_mut(id, &SADDLED)
        && *s != 0
    {
        *s = 0;
        w.spawn_item([st.pos[0], st.pos[1] + 0.5, st.pos[2]], ItemStack::new(saddle, 1));
    }
}

/// A player right clicks a tame: its saddle saddles it; an empty hand on a saddled one gets on.
/// True if the click was used.
pub fn interact(w: &World, id: u64, player: u64) -> bool {
    let Some(saddle) = saddle_of(w, id) else { return false };
    if !crate::animal::tamed(w, id) || !crate::commands::can_command(w, id, player) {
        return false;
    }
    let held = w.inventory(player).get(w.selected_slot(player));
    let saddled = w.component(id, &SADDLED).is_some_and(|&s| s != 0);
    if !saddled {
        if held.item != saddle {
            return false;
        }
        crate::taming::take_one(w, player);
        if let Some(s) = w.component_mut(id, &SADDLED) {
            *s = 1;
        }
        return true;
    }
    if !held.is_empty() || w.rider_of(id).is_some() {
        return false;
    }
    let Some(a) = w.component(id, &ANIMAL).copied() else { return false };
    let Some(st) = w.entity(id) else { return false };
    let out = w
        .component(id, &crate::knockout::KNOCKOUT)
        .is_some_and(|k| k.out > 0);
    if a.bond < BOND || a.stage < crate::animal::ADULT || out {
        // Not ready to carry anyone: it shakes them off.
        drop(
            w.spawn_particles(&POOF, [st.pos[0], st.pos[1] + 0.6, st.pos[2]])
                .count(4)
                .spread(0.2),
        );
        return true;
    }
    let seat = a.species().map_or(0.0, |s| s.seat);
    if w.mount(player, id, seat) {
        w.add_modifier(id, &attribute::JUMP_STRENGTH, "animals:ridden", attr_op::MUL_BASE, JUMP, 0);
        if let Some(r) = w.component_mut(id, &RIDDEN) {
            *r = 1;
        }
        weft_sdk::info!("{player} rides {id}");
    }
    true
}

/// A goal for rideable species, first: ridden, it goes where its rider steers (and nothing
/// else); a sneaking rider gets off.
pub fn ridden(index: usize) -> impl Fn(&World, u64, &EntityState) -> bool + Send + Sync + 'static {
    move |w, id, st| {
        let Some(rider) = w.rider_of(id) else {
            if w.component(id, &RIDDEN).is_some_and(|&r| r != 0) {
                end_ride(w, id);
            }
            return false;
        };
        let keys = w.keys(rider);
        if keys & held_keys::SNEAK != 0 {
            get_off(w, id, rider, st);
            return true;
        }
        let Some(r) = w.entity(rider) else { return false };
        crate::wake(w, id, index);
        let held = |k: u32| (keys & k != 0) as i32 as f32;
        let (forward, side) = (held(held_keys::FORWARD) - held(held_keys::BACK), held(held_keys::RIGHT) - held(held_keys::LEFT));
        let jump = keys & held_keys::JUMP != 0;
        // Yaw 0 looks along -z; right of it is +x.
        let (sy, cy) = r.yaw.sin_cos();
        let dir = [-sy * forward + cy * side, -cy * forward - sy * side];
        let len = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt();
        let speed = if keys & held_keys::SPRINT != 0 && forward > 0.0 { SPRINT } else { 1.0 };
        if len > 0.0 {
            w.walk(id, [dir[0] / len, dir[1] / len], speed, jump);
        } else if jump {
            w.walk(id, [0.0, 0.0], 0.0, true);
        }
        if w.navigating(id) {
            w.navigate(id, st.pos, 0.0);
        }
        // Walking, it faces where its rider looks (standing, the rider looks about freely).
        if len > 0.0 {
            let mut s = *st;
            s.yaw = r.yaw;
            w.set_entity(id, &s);
        }
        true
    }
}

/// The rider gets off beside it.
fn get_off(w: &World, id: u64, rider: u64, st: &EntityState) {
    w.dismount(rider);
    end_ride(w, id);
    let (sy, cy) = st.yaw.sin_cos();
    let side = [st.pos[0] + cy as f64 * 0.9, st.pos[1], st.pos[2] - sy as f64 * 0.9];
    w.teleport(rider, None, side);
}

fn end_ride(w: &World, id: u64) {
    w.remove_modifier(id, &attribute::JUMP_STRENGTH, "animals:ridden");
    if let Some(r) = w.component_mut(id, &RIDDEN) {
        *r = 0;
    }
}
