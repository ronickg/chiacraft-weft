use std::sync::atomic::{AtomicBool, Ordering};
use weft_sdk::*;

static BLASTER: Item = Item::named("scoped_blaster");
static ARROW: Item = Item::named("base:arrow");
static FAST_ARROW: EntityKind = EntityKind::named("fast_arrow");
static SCOPE: Shader = Shader::named("blaster_scope");

const SPEED: f64 = 240.0;

pub fn register(r: &mut Registry) {
    r.item(&BLASTER)
        .texture("blaster")
        .model_file("item/blaster")
        .max_stack(1)
        .handheld()
        .use_until_released()
        .on_attack(shoot)
        .tab(&base_api::tabs::COMBAT);
    r.entity(&FAST_ARROW)
        .no_egg()
        .texture("base:arrow")
        .size(0.5, 0.5)
        .unpushable()
        .max_health(1.0)
        .walk_speed(0.0)
        .attribute(&attribute::GRAVITY, 0.625)
        .projectile(Projectile::new().damage(0.9).drag(0.999, 0.6).sticks(1200));
    r.set_model(&FAST_ARROW, "base:arrow");

    r.modify(&attribute::FOV, &phase::MULTIPLY, |m| {
        let w = m.world();
        let id = m.query().entity;
        if let Some(s) = w.entity(id)
            && s.using == BLASTER.id()
            && w.inventory(id).get(s.selected).item == BLASTER.id()
        {
            m.value *= 0.25;
        }
    });
    let scoped = AtomicBool::new(false);
    r.client_frame(move |c| {
        let st = c.state();
        let aiming = st.camera == cameras::FIRST_PERSON.id()
            && c.inventory_slot(st.selected).item == BLASTER.id()
            && c.key_held(&keys::USE)
            && c.view()
                .player_state()
                .is_some_and(|s| s.using == BLASTER.id());
        if scoped.swap(aiming, Ordering::Relaxed) != aiming {
            c.post_effect(&SCOPE, if aiming { 1.0 } else { 0.0 });
        }
    });
}

fn shoot(u: &mut ItemUse) -> bool {
    let (w, id) = (u.world(), u.entity());
    let Some(s) = w.entity(id) else { return false };
    let inv = w.inventory(id);
    let ammo = if w.creative(id) {
        None
    } else {
        let Some(slot) = (0..inv.len()).find(|&slot| inv.get(slot).item == ARROW.id()) else { return false };
        Some(slot)
    };
    let velocity = look_dir(s.yaw, s.pitch) * SPEED;
    if w.spawn_with(&FAST_ARROW, s.eye(), &Spawn::new().owner(id).velocity(velocity).yaw(s.yaw)) == 0 {
        return false;
    }
    if let Some(slot) = ammo {
        inv.set(slot, consume_one(inv.get(slot)));
    }
    true
}
