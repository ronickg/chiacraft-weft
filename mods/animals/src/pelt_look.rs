//! A carried pelt shows on the player (Planner, 11-26: look only; no slowdown, no shoulder slot). A fur
//! anywhere in the inventory is a roll slung across the back; else a hide is folded over the left
//! shoulder (`source/pelt_carried/build.py`, drawn from the items' icons). The server sets what each
//! player carries on `CARRIED`, seen by everyone near; each client hides the models of who carries none.

use weft_sdk::*;

/// What a player carries, seen by everyone near: 0 nothing, `HIDE_SHOWN` or `FUR_SHOWN`.
static CARRIED: Component<u32> = Component::named("pelt_carried");
const HIDE_SHOWN: u32 = 1;
const FUR_SHOWN: u32 = 2;

static HIDE: Item = Item::named("base:hide");
static FUR: Item = Item::named("base:fur");

/// The two models on every player's body (`models/pelt_{hide,fur}.geo.json`), each its root bone's name.
const MODELS: [(&str, &str, u32); 2] = [("animals:pelt_hide", "pelt_hide", HIDE_SHOWN), ("animals:pelt_fur", "pelt_fur", FUR_SHOWN)];

/// How often the server looks in the players' inventories (ticks).
const EVERY: u64 = 10;

pub fn init(r: &mut Registry) {
    r.entity_component(&CARRIED, 0u32, component_flags::SYNC_WATCHERS);
    r.edit(&PLAYER).attach(&CARRIED);
    for (model, _, _) in MODELS {
        r.bone_model(&PLAYER, model, "body");
    }
    r.world_tick(tick);
    r.client_frame(|c| {
        let mut ids = c.entities();
        // One's own player too (not among the entities a client knows).
        let me = c.state().player;
        if !ids.contains(&me) {
            ids.push(me);
        }
        for id in ids {
            let Some(e) = c.entity(id) else { continue };
            if e.kind != PLAYER.id() {
                continue;
            }
            let shown = c.component_of(id, &CARRIED).unwrap_or(0);
            for (_, bone, which) in MODELS {
                if shown != which {
                    c.bone_scale(id, bone, [0.0; 3]);
                }
            }
        }
    });
}

/// The pelt `id` shows: a fur over a hide, none for an empty inventory of them.
fn carried(w: &World, id: u64) -> u32 {
    let inv = w.inventory(id);
    let mut shown = 0;
    for i in 0..inv.len() {
        let s = inv.get(i);
        if s.is_empty() {
            continue;
        }
        if s.item == FUR.id() {
            return FUR_SHOWN;
        }
        if s.item == HIDE.id() {
            shown = HIDE_SHOWN;
        }
    }
    shown
}

fn tick(w: &World) {
    if !w.game_time().is_multiple_of(EVERY) {
        return;
    }
    for id in w.players() {
        let now = carried(w, id);
        if w.component(id, &CARRIED) != Some(now)
            && let Some(c) = w.component_mut(id, &CARRIED)
        {
            *c = now;
        }
    }
}
