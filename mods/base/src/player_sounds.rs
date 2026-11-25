//! The player's own sounds and the sounds of using things (Planner's sound audit, Ronald 11-25:
//! "Yes, top 4"; AI-made like the creatures' voices, `assets/sounds/SOURCES.md`). No engine
//! change: what the engine reports (hurt, death, an item used, a key), a look at each player
//! every tick (water, ground, breath, what is held, worn and being used), a call where a door
//! swings, a bow shoots or the grid crafts, and on the client the open menu and the fires near.
//!
//! - The player's kind takes `base:player.hurt|death` (the engine plays them); a fall's hurt adds
//!   a heavy landing, drowning's a burst of bubbles.
//! - Eating chews while the food is held to the mouth and ends in a small burp; stew is drunk.
//! - Into water a splash, swimming a stroke now and then, up from a long dive a gasp; a jump or
//!   a short drop lands with a thud (a long one hurts instead).
//! - A swing of the hand or a tool; a tool used up snaps; a bow creaks as it is drawn and twangs
//!   as it shoots; armour put on buckles.
//! - Doors open and close, chests and the furnace open (and chests close), the inventory opens
//!   and closes (the client's open menu); the crafting grid's result taken knocks wood.
//! - Fire crackles near (a campfire, a fire bed), torches flicker, a lit furnace roars low: loops
//!   on the client at the nearest of each, like ambience's water and lava (`ambience.rs`).

use std::collections::HashMap;
use std::sync::Mutex;

use weft_sdk::*;

/// Every set: its sound's name (`base:<name>`), the files `<dir>/<file><1..n>`, its category,
/// volume, pitch range and how far it is heard.
struct Set {
    sound: &'static Sound,
    files: &'static str,
    n: u32,
    players: bool,
    volume: f32,
    pitch: (f32, f32),
    range: f32,
}

macro_rules! sounds {
    ($($s:ident $name:literal),* $(,)?) => { $(static $s: Sound = Sound::named($name);)* };
}
sounds!(
    HURT "player.hurt", DEATH "player.death", EAT "player.eat", BURP "player.burp", DRINK "player.drink",
    SWIM "player.swim", SPLASH "player.splash", LAND "player.land", FALL "player.fall", DROWN "player.drown",
    BREATH "player.breath", SWING "player.swing", TOOL_BREAK "player.tool_break", BOW_DRAW "player.bow_draw",
    BOW_SHOOT "player.bow_shoot", ARMOUR "player.armour", DOOR_OPEN "use.door_open", DOOR_CLOSE "use.door_close",
    CHEST_OPEN "use.chest_open", CHEST_CLOSE "use.chest_close", CRAFT "use.craft", FURNACE_OPEN "use.furnace_open",
    UI_OPEN "use.inventory_open", UI_CLOSE "use.inventory_close", FIRE "use.fire", TORCH "use.torch",
    FURNACE_FIRE "use.furnace_fire",
);

const fn set(sound: &'static Sound, files: &'static str, n: u32, players: bool, volume: f32, pitch: (f32, f32), range: f32) -> Set {
    Set { sound, files, n, players, volume, pitch, range }
}

const SETS: [Set; 27] = [
    set(&HURT, "player/hurt", 4, true, 1.0, (0.9, 1.1), 16.0),
    set(&DEATH, "player/death", 2, true, 1.0, (0.95, 1.05), 24.0),
    set(&EAT, "player/eat", 3, true, 0.6, (0.85, 1.15), 12.0),
    set(&BURP, "player/burp", 1, true, 0.5, (0.9, 1.1), 12.0),
    set(&DRINK, "player/drink", 3, true, 0.6, (0.9, 1.1), 12.0),
    set(&SWIM, "player/swim", 4, true, 0.5, (0.85, 1.15), 16.0),
    set(&SPLASH, "player/splash", 3, true, 0.9, (0.9, 1.1), 24.0),
    set(&LAND, "player/land", 3, true, 0.5, (0.9, 1.1), 12.0),
    set(&FALL, "player/fall", 3, true, 1.0, (0.9, 1.1), 16.0),
    set(&DROWN, "player/drown", 3, true, 0.7, (0.9, 1.1), 12.0),
    set(&BREATH, "player/breath", 2, true, 0.8, (0.95, 1.05), 12.0),
    set(&SWING, "player/swing", 4, true, 0.35, (0.85, 1.2), 12.0),
    set(&TOOL_BREAK, "player/tool_break", 3, true, 0.9, (0.9, 1.1), 16.0),
    set(&BOW_DRAW, "player/bow_draw", 3, true, 0.6, (0.95, 1.05), 12.0),
    set(&BOW_SHOOT, "player/bow_shoot", 3, true, 0.8, (0.9, 1.1), 16.0),
    set(&ARMOUR, "player/armour", 3, true, 0.7, (0.9, 1.1), 12.0),
    set(&DOOR_OPEN, "use/door_open", 3, false, 0.8, (0.9, 1.1), 16.0),
    set(&DOOR_CLOSE, "use/door_close", 3, false, 0.8, (0.9, 1.1), 16.0),
    set(&CHEST_OPEN, "use/chest_open", 3, false, 0.7, (0.9, 1.1), 16.0),
    set(&CHEST_CLOSE, "use/chest_close", 3, false, 0.7, (0.9, 1.1), 16.0),
    set(&CRAFT, "use/craft", 3, false, 0.6, (0.9, 1.1), 12.0),
    set(&FURNACE_OPEN, "use/furnace_open", 3, false, 0.7, (0.9, 1.1), 16.0),
    set(&UI_OPEN, "use/inventory_open", 3, false, 0.4, (0.95, 1.05), 8.0),
    set(&UI_CLOSE, "use/inventory_close", 3, false, 0.4, (0.95, 1.05), 8.0),
    set(&FIRE, "use/fire", 4, false, 0.8, (0.95, 1.05), 16.0),
    set(&TORCH, "use/torch", 3, false, 0.35, (0.95, 1.05), 8.0),
    set(&FURNACE_FIRE, "use/furnace_fire", 3, false, 0.6, (0.95, 1.05), 12.0),
];

static PLAYER: EntityKind = EntityKind::named("core:player");
static BOW: Item = Item::named("base:bow");

/// A swim stroke every this many ticks while moving in water; chewing every this many while
/// eating, after the first bite.
const STROKE_TICKS: u32 = 16;
const CHEW_TICKS: u32 = 5;
/// A drop of this many blocks lands with a thud; from `HURTS` the fall's hurt sounds instead.
const LANDS: f32 = 1.2;
const HURTS: f32 = 3.5;
/// Down this fast (blocks a tick) into water splashes.
const SPLASHES: f64 = 0.15;
/// Ticks under water before coming up gasps.
const GASPS: u32 = 160;

pub fn register(r: &mut Registry) {
    for s in &SETS {
        let files: Vec<String> = (1..=s.n).map(|i| format!("{}{i}", s.files)).collect();
        let files: Vec<&str> = files.iter().map(String::as_str).collect();
        r.sound(s.sound)
            .files(&files)
            .category(if s.players { &sound_categories::PLAYERS } else { &sound_categories::BLOCKS })
            .volume(s.volume)
            .pitch(s.pitch.0, s.pitch.1)
            .range(s.range)
            .voices(4);
    }
    // The engine plays a kind's hurt and death (no step: that would replace the ground's).
    r.sounds_on_entities("core:player", "base:player");
    r.on_event(&actions::HURT, |b| {
        let w = b.world();
        for h in b.iter() {
            if !is_player(&w, h.entity) {
                continue;
            }
            let Some(st) = w.entity(h.entity) else { continue };
            if h.cause == damage_cause::FALL {
                w.play_sound(&FALL, st.pos, 1.0);
            } else if h.cause == damage_cause::DROWN {
                w.play_sound(&DROWN, st.eye(), 1.0);
            }
        }
    });
    // Eaten: a small burp, or the last gulp of stew.
    r.on_event(&actions::USE_ITEM, |b| {
        let w = b.world();
        for u in b.iter() {
            if !is_player(&w, u.entity) || item_info(u.stack.item).is_some_and(|i| i.durability > 0) {
                continue;
            }
            let Some(st) = w.entity(u.entity) else { continue };
            if u.stack.item == crate::cooking::STEW.id() {
                w.play_sound(&DRINK, st.eye(), 1.0);
            } else if base_api::components::FOOD.on_item(u.stack.item).is_some() {
                w.play_sound(&BURP, st.eye(), 1.0);
            }
        }
    });
    // A swing of the hand (louder ones, the hits, sound for themselves).
    r.on_event(&KEY, |b| {
        let w = b.world();
        for k in b.iter() {
            if k.down != 0 && k.key == keys::ATTACK.id()
                && let Some(st) = w.entity(k.player)
            {
                w.play_sound(&SWING, st.eye(), 1.0);
            }
        }
    });
    r.world_tick(tick);
    r.client_frame(frame);
}

fn is_player(w: &World, id: u64) -> bool {
    w.entity(id).is_some_and(|e| e.kind == PLAYER.id())
}

/// A door swung (`doors.rs`, by hand or power): open or close by what it is now.
pub fn door(w: &World, pos: impl Into<BlockPos>) {
    let pos = pos.into();
    let open = w.prop(pos, "open") == Some("true");
    let at = Vec3::new(pos.x as f64 + 0.5, pos.y as f64 + 1.0, pos.z as f64 + 0.5);
    w.play_sound(if open { &DOOR_OPEN } else { &DOOR_CLOSE }, at, 1.0);
}

/// The crafting grid's result taken (`player.rs`).
pub fn crafted(w: &World, player: u64) {
    if let Some(st) = w.entity(player) {
        w.play_sound(&CRAFT, st.pos, 1.0);
    }
}

/// A bow shot (`mobs.rs`).
pub fn bow_shot(w: &World, id: u64) {
    if let Some(st) = w.entity(id) {
        w.play_sound(&BOW_SHOOT, st.eye(), 1.0);
    }
}

/// What a player was doing last tick.
#[derive(Default)]
struct Was {
    in_liquid: bool,
    on_ground: bool,
    fall: f32,
    vy: f64,
    under: u32,
    stroke: u32,
    using: u32,
    held: (u32, ItemStack),
    worn: Vec<u32>,
}

static WAS: Mutex<Option<HashMap<u64, Was>>> = Mutex::new(None);

fn tick(w: &World) {
    let mut all = WAS.lock().unwrap();
    let all = all.get_or_insert_with(HashMap::new);
    let players = w.players();
    all.retain(|id, _| players.contains(id));
    for p in players {
        let Some(st) = w.entity(p) else { continue };
        let was = all.entry(p).or_insert_with(|| Was { on_ground: true, ..Was::default() });
        let (in_liquid, on_ground) = (st.in_liquid != 0, st.on_ground != 0);
        // Water: a splash going in fast, strokes while swimming, a gasp up from a long dive.
        if in_liquid && !was.in_liquid && was.vy < -SPLASHES {
            w.play_sound(&SPLASH, st.pos, 1.0);
        }
        let moving = st.vel.x * st.vel.x + st.vel.z * st.vel.z > 0.002;
        was.stroke = if in_liquid && !on_ground && moving { was.stroke + 1 } else { 0 };
        if was.stroke >= STROKE_TICKS {
            was.stroke = 0;
            w.play_sound(&SWIM, st.pos, 1.0);
        }
        let under = crate::survival::breath::underwater(w, &st);
        if !under && was.under >= GASPS {
            w.play_sound(&BREATH, st.eye(), 1.0);
        }
        was.under = if under { was.under + 1 } else { 0 };
        // Down on the ground from a jump or a short drop (a long one hurts: `FALL`).
        if on_ground && !was.on_ground && !in_liquid && (LANDS..HURTS).contains(&was.fall) && st.flying == 0 {
            w.play_sound(&LAND, st.pos, 1.0);
        }
        // In the hand: chewing or drinking, a bow drawn, a tool used up.
        let slot = w.selected_slot(p);
        let held = w.inventory(p).group("hotbar").get(slot);
        if st.using != 0 {
            let t = st.using_ticks;
            if held.item == BOW.id() {
                if was.using == 0 {
                    w.play_sound(&BOW_DRAW, st.eye(), 1.0);
                }
            } else if t > 2 && t % CHEW_TICKS == 0 {
                if held.item == crate::cooking::STEW.id() {
                    w.play_sound(&DRINK, st.eye(), 1.0);
                } else if base_api::components::FOOD.on_item(held.item).is_some() {
                    w.play_sound(&EAT, st.eye(), 1.0);
                }
            }
        }
        let (was_slot, was_held) = was.held;
        if was_slot == slot && was_held.item != 0 && used_up(was_held, held) {
            w.play_sound(&TOOL_BREAK, st.eye(), 1.0);
        }
        // Armour put on: a worn slot that holds something new.
        let worn: Vec<u32> = w
            .inventory(p)
            .find_group(EQUIPMENT_GROUP)
            .map(|g| (0..g.len()).map(|i| g.get(i).item).collect())
            .unwrap_or_default();
        if !w.creative(p) && worn.len() == was.worn.len() && worn.iter().zip(&was.worn).any(|(n, o)| *n != 0 && n != o) {
            w.play_sound(&ARMOUR, st.pos, 1.0);
        }
        *was = Was {
            in_liquid,
            on_ground,
            fall: if on_ground { 0.0 } else { st.fall_distance.max(was.fall) },
            vy: st.vel.y,
            under: was.under,
            stroke: was.stroke,
            using: st.using,
            held: (slot, held),
            worn,
        };
    }
}

/// A tool `was` that is used up now: gone from the hand, or kept broken (a tool of parts).
fn used_up(was: ItemStack, now: ItemStack) -> bool {
    let Some(most) = item_info(was.item).map(|i| i.durability).filter(|&d| d > 0) else { return false };
    let close = was.wear() + 8 >= most;
    (now.is_empty() && close) || (now.item == was.item && was.wear() < most && now.wear() >= most)
}

// ------------------------------------------------------------------------------- the client

/// The client's: the menu open last frame, and the fire loops.
#[derive(Default)]
struct Near {
    menu: Option<u32>,
    look: f32,
    loops: [SoundHandle; 3],
}

static NEAR: Mutex<Option<Near>> = Mutex::new(None);
/// Seconds between looks for fires; blocks round the head looked through.
const LOOK: f32 = 0.5;
const REACH: i32 = 8;
/// The player's eyes over its feet.
const EYE: f64 = 1.62;

fn frame(c: &Client) {
    let st = c.state();
    let mut s = NEAR.lock().unwrap();
    if st.flags & client::client_flags::JOINED == 0 {
        *s = None;
        return;
    }
    let s = s.get_or_insert_with(Near::default);
    let dt = st.dt.clamp(0.0, 0.5);
    // A menu opened or closed: a chest's lid, the furnace's door, the bag.
    let menu = c.menu().map(|m| m.kind);
    if menu != s.menu {
        if let Some(k) = s.menu {
            if k == base_api::menus::CHEST.id() {
                c.play_sound(&CHEST_CLOSE, None, 1.0);
            } else if k == screens::INVENTORY.id() {
                c.play_sound(&UI_CLOSE, None, 1.0);
            }
        }
        if let Some(k) = menu {
            if k == base_api::menus::CHEST.id() {
                c.play_sound(&CHEST_OPEN, None, 1.0);
            } else if k == base_api::menus::FURNACE.id() {
                c.play_sound(&FURNACE_OPEN, None, 1.0);
            } else if k == screens::INVENTORY.id() {
                c.play_sound(&UI_OPEN, None, 1.0);
            }
        }
        s.menu = menu;
    }
    s.look -= dt;
    if s.look > 0.0 {
        return;
    }
    s.look = LOOK;
    let eye = [st.pos.x, st.pos.y + EYE, st.pos.z];
    let head = [eye[0].floor() as i32, eye[1].floor() as i32, eye[2].floor() as i32];
    // The nearest fire, torch and lit furnace.
    let mut best: [Option<(f64, [f64; 3])>; 3] = [None; 3];
    for dx in -REACH..=REACH {
        for dy in -REACH / 2..=REACH / 2 {
            for dz in -REACH..=REACH {
                let p = [head[0] + dx, head[1] + dy, head[2] + dz];
                let Some(b) = c.block(p) else { continue };
                if b == 0 {
                    continue;
                }
                let Some(k) = burning(b) else { continue };
                let at = [p[0] as f64 + 0.5, p[1] as f64 + 0.5, p[2] as f64 + 0.5];
                let d = (0..3).map(|i| (at[i] - eye[i]).powi(2)).sum::<f64>().sqrt();
                if best[k].is_none_or(|(bd, _)| d < bd) {
                    best[k] = Some((d, at));
                }
            }
        }
    }
    for (k, sound) in [&FIRE, &TORCH, &FURNACE_FIRE].into_iter().enumerate() {
        let h = &mut s.loops[k];
        match best[k] {
            Some((_, at)) => {
                if h.playing(c) {
                    h.moved(c, at, LOOK);
                } else {
                    *h = c.start_sound(sound, Some(at), 1.0, true, 1.0);
                }
            }
            None if h.playing(c) => {
                h.stop(c, 1.0);
                *h = SoundHandle::default();
            }
            None => {}
        }
    }
}

/// Which loop a block state burns with: 0 a fire, 1 a torch, 2 a lit furnace.
fn burning(state: u32) -> Option<usize> {
    let name = block_name(state);
    let name = name.split('[').next().unwrap_or(name);
    match name {
        "base:campfire" | "base:fire_bed" => Some(0),
        "base:torch" | "base:wall_torch" => Some(1),
        "base:furnace" if state_prop(state, "lit") == Some("true") => Some(2),
        _ => None,
    }
}
