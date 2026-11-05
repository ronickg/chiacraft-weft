//! Dying and the way back (Research round 4 H1, the first-hour report): the death screen says what
//! killed one and where the tombstone is; after respawning a line over the hotbar points the way
//! to it; and monsters leave one be until one reaches it or three minutes pass
//! (`vitals::difficulty::GRACE_SECS`), Valheim's corpse run. The tombstone itself is `levels.rs`'s
//! (what one carried goes into it), the respawn kit `clothing.rs`'s and `survival`'s.

use std::sync::Mutex;

use base_api::GRAVE;
use vitals::difficulty::{GRACE_AT_GRAVE, GRACE_SECS, GRAVE_REACHED};
use weft_sdk::*;

/// 1 when a player's last death left no tombstone (nowhere to put one) and the engine dropped what
/// they carried where they died; for the death screen.
pub static DROPPED: Component<u8> = Component::named("base:dropped");
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
    /// No tombstone: what one carried fell where one died.
    dropped: bool,
}
static LAST: Mutex<Option<Death>> = Mutex::new(None);

pub fn register(r: &mut Registry) {
    r.entity_component(&GRACE, 0.0, component_flags::SAVE | component_flags::KEEP_ON_RESPAWN | component_flags::SYNC_OWNER);
    r.entity_component(&DROPPED, 0, component_flags::KEEP_ON_RESPAWN);
    r.attach(&PLAYER, &DROPPED);
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
    r.intercept(&actions::HURT, |q| {
        let w = q.world();
        let source = q.query().other;
        let in_grace = w.component(q.entity(), &GRACE).is_some_and(|g| g > 0.0);
        if in_grace && source != 0 && source != q.entity() && !w.players().contains(&source) { Verdict::Deny } else { Verdict::Abstain }
    });
    r.world_tick(watch);
    client(r);
}

/// Whether `player` is in grace now (monsters don't go after them, `mobs.rs`).
pub fn in_grace(w: &World, player: u64) -> bool {
    w.component(player, &GRACE).is_some_and(|g| g > 0.0)
}

/// Tells the one who died what killed them and where their things are (after the tombstone is
/// placed, `levels.rs`).
pub fn tell(w: &World, d: &abi::Death) {
    use crate::survival::body::{BLEED, FREEZE, HEAT, POISON};
    use abi::damage_cause as dm;
    let killer = (d.source != 0 && d.source != d.entity)
        .then(|| w.entity(d.source))
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
        (c, _) if c == FREEZE.id() => "freeze",
        (c, _) if c == HEAT.id() => "heat",
        (c, _) if c == POISON.id() => "poison",
        (c, _) if c == BLEED.id() => "bleed",
        _ => "generic",
    };
    let dropped = w.component(d.entity, &DROPPED).is_some_and(|x| x != 0);
    let grave = w
        .component(d.entity, &GRAVE)
        .filter(|g| g.set != 0)
        .map_or(if dropped { "dropped\t-\t-".to_string() } else { "-\t-\t-".to_string() }, |g| format!("{}\t{}\t{}", g.x, g.y, g.z));
    let text = format!("death.base.cause.{cause}\t{}\t{grave}", killer.unwrap_or_default());
    w.send(d.entity, &DIED, &text);
}

/// Once a second: the grace runs down; reaching the tombstone (or its being gone) ends the marker
/// and leaves the grace `GRACE_AT_GRAVE` seconds at most.
fn watch(w: &World) {
    if w.game_time() % 20 != 3 {
        return;
    }
    for id in w.players() {
        let Some(st) = w.entity(id) else { continue };
        if st.health <= 0.0 {
            continue;
        }
        // Only where the tombstone is (`set` is its dimension + 1): elsewhere the grace runs down and
        // the marker waits (integration review: a respawn in another dimension found none and cut
        // the grace).
        let Some(g) = w
            .component(id, &GRAVE)
            .filter(|g| g.set == w.dimension() + 1)
        else {
            if let Some(gr) = w.component_mut(id, &GRACE) {
                *gr = (*gr - 1.0).max(0.0);
            }
            continue;
        };
        let at = [g.x, g.y, g.z];
        let d2 = (0..3)
            .map(|i| (<[f64; 3]>::from(st.pos)[i] - (at[i] as f64 + 0.5)).powi(2))
            .sum::<f64>();
        let gone = !crate::levels::TOMBSTONE.is(w.block(at));
        if d2 <= GRAVE_REACHED * GRAVE_REACHED || gone {
            if let Some(g) = w.component_mut(id, &GRAVE) {
                g.set = 0;
            }
            if let Some(gr) = w.component_mut(id, &GRACE) {
                *gr = gr.min(GRACE_AT_GRAVE);
            }
        } else if let Some(gr) = w.component_mut(id, &GRACE) {
            *gr = (*gr - 1.0).max(0.0);
        }
    }
}

/// Eight ways round, from ahead turning left: `hud.base.way.<n>`.
pub(crate) fn way(yaw_to: f32, yaw: f32) -> usize {
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
            dropped: p.get(2) == Some(&"dropped"),
        });
    });
    // The death screen, the engine's with what happened.
    r.screen(&screens::DEATH, |ui| {
        // The UI redo: a dark red-brown vignette (`death`), and in its middle a window with the
        // title, what happened, where the tombstone is, and the two ways on.
        ui.panel("death", S::column().center().stretch(), |ui| {
            ui.panel(
                "panel",
                S::column()
                    .pad(10.0)
                    .gap(6.0)
                    .w(240.0)
                    .self_align(abi::ui_align::CENTER),
                |ui| {
                    ui.text_styled(
                        S::column()
                            .class("title")
                            .scale(2.0)
                            .no_wrap()
                            .self_align(abi::ui_align::CENTER),
                        "death.title",
                        &[],
                    );
                    ui.text_bind_styled(S::column().self_align(abi::ui_align::CENTER), |_, out| {
                        let last = LAST.lock().unwrap();
                        match last.as_ref() {
                            Some(d) if !d.killer.is_empty() => out.set(&d.cause, &[Arg::Key(&d.killer)]),
                            Some(d) => out.set(&d.cause, &[]),
                            None => out.set("death.base.cause.generic", &[]),
                        }
                    });
                    ui.panel("well", S::column().pad(5.0).stretch(), |ui| {
                        ui.text_bind_styled(S::column().max_w(216.0), |_, out| {
                            let (grave, dropped) = LAST
                                .lock()
                                .unwrap()
                                .as_ref()
                                .map_or((None, false), |d| (d.grave, d.dropped));
                            match (grave, dropped) {
                                (Some([x, y, z]), _) => {
                                    out.set("death.base.grave", &[Arg::Int(x as i64), Arg::Int(y as i64), Arg::Int(z as i64)])
                                }
                                (None, true) => out.set("death.base.dropped", &[]),
                                (None, false) => out.set("death.base.no_grave", &[]),
                            }
                        });
                    });
                    ui.row(S::row().gap(6.0).self_align(abi::ui_align::CENTER), |ui| {
                        ui.button("death.respawn", S::column().min_w(110.0), |c| c.client().respawn());
                        ui.button("menu.save_quit", S::column().class("button_secondary").min_w(110.0), |c| c.client().world_leave());
                    });
                },
            );
        });
    })
    .keep_on_escape()
    .closes_on_respawn();
    // Over the hotbar after respawning: the way to the tombstone, and the grace left.
    r.hud_element(&GRAVE_MARK, hud_anchor::ABOVE_HOTBAR, 2, |ui| {
        ui.text_bind(|c, out| {
            let (Some(g), Some(me)) = (c.component(&GRAVE).filter(|g| g.set != 0), c.player_state()) else {
                out.text("");
                return;
            };
            if c.outside_survival() || me.health <= 0.0 {
                out.text("");
                return;
            }
            let (dx, dz) = (g.x as f64 + 0.5 - me.pos.x, g.z as f64 + 0.5 - me.pos.z);
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
