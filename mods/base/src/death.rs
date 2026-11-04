//! Dying and the way back (Research round 4 H1, the first-hour report): the death screen says what
//! killed one and where the tombstone is; after respawning a line over the hotbar points the way
//! to it; and monsters leave one be until one reaches it or three minutes pass
//! (`vitals::difficulty::GRACE_SECS`), Valheim's corpse run. The tombstone itself is `levels.rs`'s
//! (what one carried goes into it), the respawn kit `clothing.rs`'s and `survival`'s.

use std::sync::Mutex;

use base_api::GRAVE;
use vitals::difficulty::{GRACE_SECS, GRAVE_REACHED};
use weft_sdk::*;

/// Seconds left of the grace after respawning (0: none), synced to show on the HUD.
pub static GRACE: Component<f32> = Component::named("base:grace");
/// Server to the one who died: what killed them (`death.base.cause.<x>`), the killer's name key if
/// a creature did, and the tombstone's x, y, z or "-", tab separated.
static DIED: Channel<String> = Channel::named("base:died");
static GRAVE_MARK: HudElement = HudElement::named("grave");

/// What the client was told of its last death.
struct Death {
    cause: String,
    killer: String,
    grave: Option<[i32; 3]>,
}
static LAST: Mutex<Option<Death>> = Mutex::new(None);

pub fn register(r: &mut Registry) {
    r.entity_component(&GRACE, 0.0, component_flags::SAVE | component_flags::KEEP | component_flags::SYNC_OWNER);
    r.attach(&PLAYER, &GRACE);
    // Back from the dead: monsters leave one be until the tombstone is reached.
    r.on(&PLAYER_RESPAWNED, |b| {
        let w = b.world();
        for p in b.iter() {
            let has_grave = w.component(p.player, &GRAVE).is_some_and(|g| g.set != 0);
            if let Some(g) = w.component_mut(p.player, &GRACE) {
                *g = if has_grave { GRACE_SECS } else { 0.0 };
            }
        }
    });
    // No creature hurts one in grace (monsters, their arrows, wild hunters); falls, fire and the
    // cold still do.
    r.intercept(&question::HURT, |q| {
        let w = q.world();
        let source = q.query().other;
        let in_grace = w.component(q.entity(), &GRACE).is_some_and(|g| *g > 0.0);
        if in_grace && source != 0 && source != q.entity() && !w.players().contains(&source) { Verdict::Deny } else { Verdict::Abstain }
    });
    r.world_tick(watch);
    client(r);
}

/// Whether `player` is in grace now (monsters don't go after them, `mobs.rs`).
pub fn in_grace(w: &World, player: u64) -> bool {
    w.component(player, &GRACE).is_some_and(|g| *g > 0.0)
}

/// Tells the one who died what killed them and where their things are (after the tombstone is
/// placed, `levels.rs`).
pub fn tell(w: &World, d: &abi::Death) {
    use abi::damage as dm;
    let killer = (d.killer != 0 && d.killer != d.entity)
        .then(|| w.entity(d.killer))
        .flatten()
        .map(|e| format!("entity.{}", name_of(abi::entry_kind::ENTITY, e.kind).replace(':', ".")));
    let cause = match (d.cause, &killer) {
        (dm::ATTACK, Some(_)) => "attack",
        (dm::FALL, _) => "fall",
        (dm::FIRE, _) => "fire",
        (dm::LAVA, _) => "lava",
        (dm::STARVE, _) => "starve",
        (dm::DROWN, _) => "drown",
        (dm::VOID, _) => "void",
        (dm::CRUSH, _) => "crush",
        (dm::EXPLOSION, _) => "explosion",
        (dm::SUFFOCATE, _) => "suffocate",
        (dm::FREEZE, _) => "freeze",
        (dm::HEAT, _) => "heat",
        (dm::POISON, _) => "poison",
        (dm::BLEED, _) => "bleed",
        _ => "generic",
    };
    let grave = w
        .component(d.entity, &GRAVE)
        .filter(|g| g.set != 0)
        .map_or("-\t-\t-".to_string(), |g| format!("{}\t{}\t{}", g.x, g.y, g.z));
    let text = format!("death.base.cause.{cause}\t{}\t{grave}", killer.unwrap_or_default());
    w.send(d.entity, &DIED, &text);
}

/// Once a second: the grace runs down, and reaching the tombstone (or its being gone) ends it and
/// the marker.
fn watch(w: &World) {
    if w.time() % 20 != 3 {
        return;
    }
    for id in w.players() {
        let Some(st) = w.entity(id) else { continue };
        if st.health <= 0.0 {
            continue;
        }
        let Some(g) = w.component(id, &GRAVE).filter(|g| g.set != 0) else {
            if let Some(gr) = w.component_mut(id, &GRACE) {
                *gr = (*gr - 1.0).max(0.0);
            }
            continue;
        };
        let at = [g.x, g.y, g.z];
        let d2 = (0..3)
            .map(|i| (st.pos[i] - (at[i] as f64 + 0.5)).powi(2))
            .sum::<f64>();
        let gone = !crate::levels::TOMBSTONE.is(w.get(at));
        if d2 <= GRAVE_REACHED * GRAVE_REACHED || gone {
            if let Some(g) = w.component_mut(id, &GRAVE) {
                g.set = 0;
            }
            if let Some(gr) = w.component_mut(id, &GRACE) {
                *gr = 0.0;
            }
        } else if let Some(gr) = w.component_mut(id, &GRACE) {
            *gr = (*gr - 1.0).max(0.0);
        }
    }
}

/// Eight ways round, from ahead turning left: `hud.base.way.<n>`.
fn way(yaw_to: f32, yaw: f32) -> usize {
    let tau = std::f32::consts::TAU;
    let rel = (yaw_to - yaw).rem_euclid(tau);
    ((rel / (tau / 8.0)).round() as usize) % 8
}

fn client(r: &mut Registry) {
    r.client_packet(&DIED, |_, text| {
        let p: Vec<&str> = text.split('\t').collect();
        let num = |i: usize| p.get(i).and_then(|v| v.parse::<i32>().ok());
        *LAST.lock().unwrap() = Some(Death {
            cause: p
                .first()
                .copied()
                .unwrap_or("death.base.cause.generic")
                .to_string(),
            killer: p.get(1).copied().unwrap_or("").to_string(),
            grave: num(2).zip(num(3)).zip(num(4)).map(|((x, y), z)| [x, y, z]),
        });
    });
    // The death screen, the engine's with what happened.
    r.screen(&screens::DEATH, |ui| {
        ui.panel("death", S::col().gap(10.0).center().stretch(), |ui| {
            ui.text_styled(
                S::col()
                    .class("title")
                    .scale(2.0)
                    .no_wrap()
                    .self_align(abi::ui_align::CENTER),
                "death.title",
                &[],
            );
            ui.text_bind_styled(S::col().self_align(abi::ui_align::CENTER), |_, out| {
                let last = LAST.lock().unwrap();
                match last.as_ref() {
                    Some(d) if !d.killer.is_empty() => out.set(&d.cause, &[Arg::Key(&d.killer)]),
                    Some(d) => out.set(&d.cause, &[]),
                    None => out.set("death.base.cause.generic", &[]),
                }
            });
            ui.text_bind_styled(S::col().self_align(abi::ui_align::CENTER), |_, out| {
                match LAST.lock().unwrap().as_ref().and_then(|d| d.grave) {
                    Some([x, y, z]) => out.set("death.base.grave", &[Arg::Int(x as i64), Arg::Int(y as i64), Arg::Int(z as i64)]),
                    None => out.set("death.base.no_grave", &[]),
                }
            });
            ui.button("death.respawn", S::col().min_w(200.0).self_align(abi::ui_align::CENTER), |c| c.client().respawn());
            ui.button("menu.save_quit", S::col().min_w(200.0).self_align(abi::ui_align::CENTER), |c| c.client().leave_world());
        });
    })
    .keep_on_escape()
    .closes_on_respawn();
    // Over the hotbar after respawning: the way to the tombstone, and the grace left.
    r.hud_element(&GRAVE_MARK, hud_anchor::ABOVE_HOTBAR, 2, |ui| {
        ui.text_bind(|c, out| {
            let (Some(g), Some(me)) = (c.component(&GRAVE).filter(|g| g.set != 0), c.me()) else {
                out.text("");
                return;
            };
            if c.creative() || me.health <= 0.0 {
                out.text("");
                return;
            }
            let (dx, dz) = (g.x as f64 + 0.5 - me.pos[0], g.z as f64 + 0.5 - me.pos[2]);
            let dist = (dx * dx + dz * dz).sqrt().round() as i64;
            let to = (-dx).atan2(-dz) as f32;
            let way = format!("hud.base.way.{}", way(to, me.yaw));
            let grace = c.component(&GRACE).unwrap_or(0.0).ceil() as i64;
            if grace > 0 {
                let secs = format!("{:02}", grace % 60);
                out.set("hud.base.grave_grace", &[Arg::Int(dist), Arg::Key(&way), Arg::Int(grace / 60), Arg::Text(&secs)]);
            } else {
                out.set("hud.base.grave", &[Arg::Int(dist), Arg::Key(&way)]);
            }
        });
    });
}
