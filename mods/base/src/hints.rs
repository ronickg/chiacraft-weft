//! Hints that come from the game's state (Research round 4, section 3.4): a line under the
//! crosshair for a few seconds when something a new player should know about happens: getting
//! cold, an empty stomach, night coming with no fire, raw meat in hand, where the tombstone is
//! after dying. Each shows once per player (saved, kept through death), except freezing (each
//! time it starts) and the tombstone (each death). The server watches the player's state once a
//! second; the client shows one at a time and keeps them off when the player's `base:hints`
//! setting is.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Instant;

use base_api::{GRAVE, VITALS};
use vitals::effects::Effect;
use weft_sdk::*;

/// The hints, by their bit in `SEEN`: `hint.base.<name>` in the lang files.
const HINTS: [&str; 11] = ["logs", "table", "night", "cold", "freezing", "wet", "empty", "raw_meat", "grave", "hot", "rest"];
const LOGS: u32 = 0;
const TABLE: u32 = 1;
const NIGHT: u32 = 2;
const COLD: u32 = 3;
const FREEZING: u32 = 4;
const WET: u32 = 5;
const EMPTY: u32 = 6;
const RAW_MEAT: u32 = 7;
const GRAVE_AT: u32 = 8;
const HOT: u32 = 9;
const REST: u32 = 10;

/// Seconds played before the first hint about logs.
const LOGS_AFTER: u64 = 60;
/// The felt air under which being wet is worth a word, °C.
const WET_BELOW: f32 = 15.0;

/// Hints a player has had (a bit each), and whether they were freezing a second ago (not saved).
static SEEN: Component<u32> = Component::named("hints_seen");
static WAS_FREEZING: Component<u8> = Component::named("hints_was_freezing");
/// The player's own setting: hints on.
pub static SHOW: Config<bool> = Config::named("hints");

value! {
    /// Server to client: a hint, and two numbers for its text (the tombstone's x and z).
    pub struct Hint { pub hint: u32, pub a: i32, pub b: i32 }
}
static HINT: Channel<Hint> = Channel::named("hint");

static LOG_TAG: Tag = Tag::named("logs");
static TABLE_TAG: Tag = Tag::named("crafting_tables");
static FURNACE_TAG: Tag = Tag::named("furnaces");
static RAW_MEAT_TAG: Tag = Tag::named("raw_meat");
static CAMPFIRE_TAG: Tag = Tag::named("campfires");
/// The evening the night hint was last shown (the day count; not saved).
static NIGHT_SHOWN: Component<u32> = Component::named("hints_night_shown");
/// The hours the night hint shows in: from two game hours before dark (sunset is 18:00 in spring,
/// monsters need the dark after it), so a fresh player has time to build a fire.
const DUSK: std::ops::Range<f32> = 16.0..18.0;

pub fn register(r: &mut Registry) {
    r.config(&SHOW).bool(true);
    r.entity_component(&SEEN, 0, component_flags::SAVE | component_flags::KEEP);
    r.attach(&PLAYER, &SEEN);
    r.entity_component(&WAS_FREEZING, 0, 0);
    r.attach(&PLAYER, &WAS_FREEZING);
    r.entity_component(&NIGHT_SHOWN, u32::MAX, 0);
    r.attach(&PLAYER, &NIGHT_SHOWN);
    r.world_tick(watch);
    // A campfire built: the night hint has done its work (Ronald, 11-04: "Campfire keeps you
    // safe"; its light keeps monsters from spawning near it).
    r.on(&BLOCK_PLACED, |b| {
        let w = b.world();
        for c in b.iter() {
            if CAMPFIRE_TAG.on_block(c.state) {
                mark(&w, c.player, NIGHT);
            }
        }
    });
    // After dying: where the tombstone is, each time.
    r.on(&PLAYER_RESPAWNED, |b| {
        let w = b.world();
        for p in b.iter() {
            if let Some(g) = w.component(p.player, &GRAVE).filter(|g| g.set != 0) {
                w.send(p.player, &HINT, &Hint { hint: GRAVE_AT, a: g.x, b: g.z });
            }
        }
    });
    client(r);
}

fn seen(w: &World, player: u64, hint: u32) -> bool {
    w.component(player, &SEEN)
        .is_some_and(|s| s & (1 << hint) != 0)
}

/// Marks a hint seen without showing it (what it would say is already known).
fn mark(w: &World, player: u64, hint: u32) {
    if let Some(s) = w.component_mut(player, &SEEN) {
        *s |= 1 << hint;
    }
}

/// Shows a hint once.
fn once(w: &World, player: u64, hint: u32) {
    if !seen(w, player, hint) {
        mark(w, player, hint);
        w.send(player, &HINT, &Hint { hint, a: 0, b: 0 });
    }
}

/// Each player's state, once a second.
fn watch(w: &World) {
    let hour = ((w.day_time() % 24000) as f32 / 1000.0 + 6.0) % 24.0;
    for id in w.players() {
        let Some(st) = w.entity(id) else { continue };
        if st.age % 20 != 7 || w.creative(id) {
            continue;
        }
        let inv = w.inventory(id);
        let items: Vec<u32> = (0..inv.size())
            .map(|i| inv.get(i))
            .filter(|s| !s.is_empty())
            .map(|s| s.item)
            .collect();
        let has = |t: &Tag| items.iter().any(|&i| t.on_item(i));
        let (logs, table) = (has(&LOG_TAG), has(&TABLE_TAG));
        if logs {
            mark(w, id, LOGS);
        }
        if table {
            mark(w, id, TABLE);
        }
        if has(&FURNACE_TAG) {
            mark(w, id, RAW_MEAT);
        }
        if st.age >= LOGS_AFTER * 20 && !logs {
            once(w, id, LOGS);
        }
        if logs && !table {
            once(w, id, TABLE);
        }
        // Each evening at dusk until a campfire is built.
        if DUSK.contains(&hour) && !seen(w, id, NIGHT) {
            let day = (w.day_time() / 24000) as u32;
            if let Some(shown) = w.component_mut(id, &NIGHT_SHOWN)
                && *shown != day
            {
                *shown = day;
                w.send(id, &HINT, &Hint { hint: NIGHT, a: 0, b: 0 });
            }
        }
        if has(&RAW_MEAT_TAG) {
            once(w, id, RAW_MEAT);
        }
        let Some(v) = w.component(id, &VITALS) else { continue };
        let on = |e: Effect| v.effects & (1 << e.index()) != 0;
        let (outside, empty) = (v.outside, v.stomach().used() <= 0.0);
        if on(Effect::Chilly) || on(Effect::Cold) {
            once(w, id, COLD);
        }
        if on(Effect::Hot) || on(Effect::Heatstroke) {
            once(w, id, HOT);
        }
        if on(Effect::Wet) && outside < WET_BELOW {
            once(w, id, WET);
        }
        if empty {
            once(w, id, EMPTY);
        }
        let night = !(5.0..19.0).contains(&hour);
        if on(Effect::NearFire) && !on(Effect::Sheltered) && night {
            once(w, id, REST);
        }
        // Freezing: each time it starts.
        let freezing = on(Effect::Freezing);
        if let Some(was) = w.component_mut(id, &WAS_FREEZING) {
            let start = freezing && *was == 0;
            *was = freezing as u8;
            if start {
                mark(w, id, FREEZING);
                w.send(id, &HINT, &Hint { hint: FREEZING, a: 0, b: 0 });
            }
        }
    }
}

/// Seconds a hint stays, and fades at the end.
const SHOW_SECS: f32 = 6.0;
const FADE_SECS: f32 = 0.6;

/// Hints waiting (lang key and arguments), the first with when it appeared.
static QUEUE: Mutex<VecDeque<(String, Vec<i64>, Option<Instant>)>> = Mutex::new(VecDeque::new());

/// Queues a hint on the client, unless hints are off or the same one is waiting already.
fn queue(key: String, args: Vec<i64>) {
    if !SHOW.get() {
        return;
    }
    let mut q = QUEUE.lock().unwrap();
    if !q.iter().any(|(k, _, _)| *k == key) {
        q.push_back((key, args, None));
    }
}

fn client(r: &mut Registry) {
    r.client_packet(&HINT, |_, h| {
        queue(format!("hint.base.{}", HINTS[h.hint as usize % HINTS.len()]), vec![h.a as i64, h.b as i64]);
    });
    // Any mod's hint (`base_api::hint`).
    r.client_packet(&base_api::HINT_TEXT, |_, text| {
        let mut parts = text.split('\t');
        let key = parts.next().unwrap_or("").to_string();
        let args = parts.filter_map(|a| a.parse().ok()).collect();
        if !key.is_empty() {
            queue(key, args);
        }
    });
    r.client_frame(|c| {
        let mut q = QUEUE.lock().unwrap();
        let Some((key, args, at)) = q.front_mut() else { return };
        let since = at.get_or_insert_with(Instant::now).elapsed().as_secs_f32();
        if since > SHOW_SECS {
            q.pop_front();
            return;
        }
        let alpha = ((SHOW_SECS - since) / FADE_SECS).clamp(0.0, 1.0);
        let args: Vec<Arg> = args.iter().map(|&a| Arg::Int(a)).collect();
        c.screen_text(0.5, 0.78, key, &args, 0xffe8c000 | (alpha * 255.0) as u32, 1.0);
    });
}
