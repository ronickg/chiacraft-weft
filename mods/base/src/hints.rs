//! Hints that come from the game's state (Research round 4, section 3.4): a line under the
//! crosshair for a few seconds when something a new player should know about happens: getting
//! cold, an empty stomach, night coming with no fire, raw meat in hand, where the tombstone is
//! after dying. Each shows once per player (saved, kept through death), except freezing (each
//! time it starts) and the tombstone (each death). The server watches the player's state once a
//! second; the client shows one at a time and keeps them off when the player's `base:hints`
//! setting is.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use base_api::{GRAVE, VITALS};
use vitals::effects::Effect;
use weft_sdk::*;

/// The hints, by their bit in `SEEN`: `hint.base.<name>` in the lang files.
const HINTS: [&str; 16] = [
    "logs", "table", "night", "cold", "freezing", "wet", "empty", "raw_meat", "grave", "hot", "rest", "knap", "flake", "lash", "tree",
    "tree_far",
];
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
/// The first hours (`first.rs`): a rock and flint in the bag, a sharp flake, an axe head.
const KNAP: u32 = 11;
const FLAKE: u32 = 12;
const LASH: u32 = 13;
/// The first axe: where the nearest tree foot is (`TREE_FAR` when none is near; one seen bit).
const TREE: u32 = 14;
const TREE_FAR: u32 = 15;

/// Seconds played before the first hint, that bare hands can't fell a tree (`first.rs`).
const LOGS_AFTER: u64 = 20;
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
static AXES_TAG: Tag = Tag::named("tools/axe");
static TABLE_TAG: Tag = Tag::named("crafting_tables");
static FURNACE_TAG: Tag = Tag::named("furnaces");
static RAW_MEAT_TAG: Tag = Tag::named("raw_meat");
static CAMPFIRE_TAG: Tag = Tag::named("campfires");
static ROCKS_TAG: Tag = Tag::named("rocks");
static FLINT_TAG: Tag = Tag::named("flint");
/// The evening the night hint was last shown (the day count; not saved).
static NIGHT_SHOWN: Component<u32> = Component::named("hints_night_shown");
/// Game hours before sunset the night hint comes (the sunset by the calendar's daylight: 18:00 at
/// the equinoxes, later in summer, earlier in winter), so a fresh player has time to build a fire.
const DUSK_BEFORE: f32 = 2.0;

pub fn register(r: &mut Registry) {
    r.config(&SHOW).bool(true);
    r.entity_component(&SEEN, 0, component_flags::SAVE | component_flags::KEEP_ON_RESPAWN);
    r.attach(&PLAYER, &SEEN);
    r.entity_component(&WAS_FREEZING, 0, 0);
    r.attach(&PLAYER, &WAS_FREEZING);
    r.entity_component(&NIGHT_SHOWN, u32::MAX, 0);
    r.attach(&PLAYER, &NIGHT_SHOWN);
    r.world_tick(watch);
    // A campfire built: the night hint has done its work (Ronald, 11-04: "Campfire keeps you
    // safe"; its light keeps monsters from spawning near it).
    r.on(&actions::PLACE_BLOCK, |b| {
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

/// The nearest tree foot to `from` (a log standing on something that isn't a log or air), ring by
/// ring out to `vitals::first::TREE_LOOK`, the nearest in height first within a ring.
fn nearest_tree(w: &World, from: BlockPos) -> Option<BlockPos> {
    use vitals::first::{TREE_LOOK, TREE_UP};
    let foot = |p: BlockPos| {
        LOG_TAG.on_block(w.block(p)) && {
            let below = w.block(p.below());
            below != 0 && !LOG_TAG.on_block(below)
        }
    };
    for r in 0..=TREE_LOOK {
        let mut best: Option<(i32, BlockPos)> = None;
        for dz in -r..=r {
            for dx in -r..=r {
                if dx.abs().max(dz.abs()) != r {
                    continue;
                }
                for dy in 0..=TREE_UP {
                    if best.is_some_and(|(b, _)| b <= dy) {
                        break;
                    }
                    for y in [dy, -dy] {
                        let p = from.offset(dx, y, dz);
                        if foot(p) {
                            best = Some((dy, p));
                        }
                    }
                }
            }
        }
        if let Some((_, p)) = best {
            return Some(p);
        }
    }
    None
}

/// Each player's state, once a second.
fn watch(w: &World) {
    let hour = ((w.day_time() % 24000) as f32 / 1000.0 + 6.0) % 24.0;
    for id in w.players() {
        let Some(st) = w.entity(id) else { continue };
        if st.age % 20 != 7 || w.outside_survival(id) {
            continue;
        }
        let inv = w.inventory(id);
        let items: Vec<u32> = (0..inv.len())
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
        // The way to the first axe, a step at a time.
        let flake = has(&crate::first::CUTTERS);
        let head = items
            .iter()
            .any(|&i| item_name(i) == "base:part_felling_axe_head");
        if has(&ROCKS_TAG) && has(&FLINT_TAG) && !flake {
            once(w, id, KNAP);
        }
        if flake && !logs {
            once(w, id, FLAKE);
        }
        if head && !logs {
            once(w, id, LASH);
        }
        if !logs && !seen(w, id, TREE) && has(&AXES_TAG) {
            mark(w, id, TREE);
            let hint = match nearest_tree(w, st.pos.block()) {
                Some(p) => Hint { hint: TREE, a: p.x, b: p.z },
                None => Hint { hint: TREE_FAR, a: 0, b: 0 },
            };
            weft_sdk::info!(
                "hints: first axe for {id} at {:?}: tree {}",
                st.pos.block(),
                if hint.hint == TREE { format!("at x {} z {}", hint.a, hint.b) } else { "none near".into() }
            );
            w.send(id, &HINT, &hint);
        }
        // Each evening at dusk until a campfire is built, by the season's sunset (as the HUD clock
        // and beds have it; integration review: fixed at 16-18 it came after dark in winter).
        let sunset = w
            .component(id, &VITALS)
            .filter(|v| v.date > 0)
            .map_or(18.0, |v| {
                let light = vitals::calendar::daylight_hours((v.date - 1) as f32, base_api::calendars::swing(v.calendar));
                12.0 + light / 2.0
            });
        // Tonight's sky event, foretold before sunset (`events.rs`).
        crate::events::dusk(w, id, hour, sunset);
        if (sunset - DUSK_BEFORE..sunset).contains(&hour) && !seen(w, id, NIGHT) {
            let day = (w.day_time() / 24000) as u32;
            if let Some(shown) = w.component_mut(id, &NIGHT_SHOWN)
                && *shown != day
            {
                *shown = day;
                w.send(id, &HINT, &Hint { hint: NIGHT, a: sunset.round() as i32, b: 0 });
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

/// Where the hint line sits, a share of the window's height down: over the crosshair's lower
/// half and clear of the chat in the bottom left (at 0.78 the two overlapped, Tester 2 11-04).
const HINT_Y: f32 = 0.64;
/// The widest a hint is (GUI px), and the room kept on each side for the HUD's top-left column
/// (the shortcuts, about 100 wide): a long hint wraps rather than running over it in a narrow
/// window (760 x 600, the audit, 11-04). It wraps no narrower than `HINT_MIN_W`.
const HINT_MAX_W: f32 = 360.0;
const HINT_SIDES: f32 = 108.0;
const HINT_MIN_W: f32 = 140.0;
/// Seconds a hint stays, and fades at the end.
const SHOW_SECS: f32 = 6.0;
const FADE_SECS: f32 = 0.6;

/// Hints waiting (lang key and arguments), the first with when it appeared.
static QUEUE: Mutex<VecDeque<(String, Vec<i64>, Option<Instant>)>> = Mutex::new(VecDeque::new());
/// The last frame's time, for pausing a hint while a screen is open.
static LAST_FRAME: Mutex<Option<Instant>> = Mutex::new(None);

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
        let now = Instant::now();
        let dt = LAST_FRAME
            .lock()
            .unwrap()
            .replace(now)
            .map_or(Duration::ZERO, |l| now - l);
        let Some((key, args, at)) = q.front_mut() else { return };
        // Not over a screen (it ran across the inventory's grid): it waits until the screen
        // closes, and one already showing pauses (its start moves on with the time), so a
        // hint shown once per player is never used up unseen.
        if !c.screen().is_empty() {
            if let Some(a) = at {
                *a += dt;
            }
            return;
        }
        let since = at.get_or_insert_with(Instant::now).elapsed().as_secs_f32();
        if since > SHOW_SECS {
            q.pop_front();
            return;
        }
        let alpha = ((SHOW_SECS - since) / FADE_SECS).clamp(0.0, 1.0);
        let args: Vec<Arg> = args.iter().map(|&a| Arg::Int(a)).collect();
        let st = c.state();
        let gw = st.window[0] as f32 / st.gui_scale.max(1.0);
        let width = (gw - 2.0 * HINT_SIDES).clamp(HINT_MIN_W, HINT_MAX_W);
        c.screen_text_wrapped(0.5, HINT_Y, key, &args, 0xe4dacf00 | (alpha * 255.0) as u32, 1.0, width);
    });
}

/// A fire lit by `player` (`firebed.rs`): the night hint has done its work, as for one placed.
pub fn fire_lit(w: &World, player: u64) {
    mark(w, player, NIGHT);
}
