//! The map (`vitals::waystones`; `notes/PROGRESSION.md`: "a map that fills where you walk"), the
//! client's side. M opens it: an old painted chart of the land the player walked or saw (the
//! server's atlas, `waystones.rs`), the rest blank parchment; the player as a red arrow; woken
//! waystones as markers; the ley lines while the rod is held or with ley sight. Opened from a woken
//! waystone it lists the others to travel to, and a marker's click picks one.
//!
//! The land's colours come from the seed (`base_api::land`: each biome's colour, water, a slope's
//! light) a tile at a time on a thread of its own, so the frame never waits; the picture is painted
//! into a runtime GUI image (`Client::set_image`, "dyn:base:map") a few times a second while open.
//! Woken waystones nearby give off a slow rise of light, seen only by the players who woke them.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

use base_api::land as l;
use vitals::ley;
use vitals::waystones as n;
use weft_sdk::*;

use crate::waystones::{ASK, ATLAS, Atlas, Cell, OPEN, TRAVEL};

pub static SCREEN: Screen = Screen::named("map");
static OPEN_KEY: Key = Key::named("map");
const IMAGE: &str = "dyn:base:map";
/// The picture's side in GUI pixels (one image pixel each).
const VIEW: f32 = n::MAP_PX as f32;
const SIDE_W: f32 = 120.0;
/// Pixels of the tile colours a tile has on a side (at `TILE_PX_BLOCKS`).
const TPX: usize = (n::TILE / n::TILE_PX_BLOCKS) as usize;
/// Seconds between repaints while open, and between asking the server for the atlas.
const REPAINT_SECS: f32 = 0.25;
const ASK_SECS: f32 = 2.0;

// ------------------------------------------------------------------ the tile colours, on a thread

type Colours = Box<[[u8; 3]; TPX * TPX]>;

/// Tiles to colour (seed, tile) and those done.
struct Work {
    queue: VecDeque<(u64, i32, i32)>,
    queued: HashSet<(u64, i32, i32)>,
    done: HashMap<(i32, i32), Colours>,
    seed: u64,
}

/// The colouring thread's shared work and its wake-up.
type Shared = Arc<(Mutex<Work>, Condvar)>;

static WORK: Mutex<Option<Shared>> = Mutex::new(None);

fn work() -> Shared {
    let mut g = WORK.lock().unwrap();
    if let Some(w) = g.as_ref() {
        return w.clone();
    }
    let w = Arc::new((Mutex::new(Work { queue: VecDeque::new(), queued: HashSet::new(), done: HashMap::new(), seed: 0 }), Condvar::new()));
    let worker = w.clone();
    let _ = std::thread::Builder::new()
        .name("map tiles".into())
        .spawn(move || {
            loop {
                let job = {
                    let (m, cv) = &*worker;
                    let mut g = m.lock().unwrap();
                    while g.queue.is_empty() {
                        g = cv.wait(g).unwrap();
                    }
                    g.queue.pop_front()
                };
                let Some((seed, tx, tz)) = job else { continue };
                let colours = tile_colours(seed, tx, tz);
                let mut g = worker.0.lock().unwrap();
                g.queued.remove(&(seed, tx, tz));
                if g.seed == seed {
                    g.done.insert((tx, tz), colours);
                }
            }
        });
    *g = Some(w.clone());
    w
}

/// A tile's colours: each `TILE_PX_BLOCKS` square's biome or water, lit from the north-west by
/// the slope.
fn tile_colours(seed: u64, tx: i32, tz: i32) -> Colours {
    let f = l::fields(seed);
    let mut out: Colours = Box::new([[0; 3]; TPX * TPX]);
    let step = n::TILE_PX_BLOCKS;
    let h_at = |x: i32, z: i32| l::height(&f, x * 16, z * 16) >> 4;
    for pz in 0..TPX {
        for px in 0..TPX {
            let (x, z) = (tx * n::TILE + px as i32 * step, tz * n::TILE + pz as i32 * step);
            let h = h_at(x, z);
            let biome = l::overworld_biome(&f, x * 16, z * 16);
            let sea = matches!(biome, "ocean" | "coral_reef" | "lagoon");
            let c = if h < base_api::SEA && !sea { n::WATER } else { n::biome_colour(biome) };
            let shade = if h < base_api::SEA { 0 } else { ((h - h_at(x - step, z - step)) * 7).clamp(-28, 28) };
            out[pz * TPX + px] = c.map(|v| (v as i32 + shade).clamp(0, 255) as u8);
        }
    }
    out
}

/// Asks for the colours of the atlas's tiles not done yet (a ring at a time near `centre` first).
fn queue_tiles(seed: u64, atlas: &Atlas, centre: (i32, i32)) {
    let w = work();
    let (m, cv) = &*w;
    let mut g = m.lock().unwrap();
    if g.seed != seed {
        g.seed = seed;
        g.done.clear();
        g.queue.clear();
        g.queued.clear();
    }
    let mut want: Vec<(i32, i32)> = Vec::new();
    for (&(rx, rz), bits) in &atlas.regions {
        for (k, word) in bits.iter().enumerate() {
            let mut b = *word;
            while b != 0 {
                let bit = b.trailing_zeros() as i32;
                b &= b - 1;
                let at = k as i32 * 64 + bit;
                let (tx, tz) = (rx * n::REGION + at % n::REGION, rz * n::REGION + at / n::REGION);
                if !g.done.contains_key(&(tx, tz)) && !g.queued.contains(&(seed, tx, tz)) {
                    want.push((tx, tz));
                }
            }
        }
    }
    want.sort_by_key(|&(tx, tz)| (tx - centre.0).abs().max((tz - centre.1).abs()));
    for (tx, tz) in want {
        g.queued.insert((seed, tx, tz));
        g.queue.push_back((seed, tx, tz));
    }
    cv.notify_one();
}

// ------------------------------------------------------------------ what the client keeps

struct State {
    atlas: Atlas,
    /// The seed the atlas belongs to (another world's is dropped).
    seed: u64,
    zoom: usize,
    /// Blocks the view is moved from the player.
    pan: (i32, i32),
    /// The woken waystone the map was opened at, to travel from.
    from: Option<(i32, i32)>,
    asked: Option<Instant>,
    painted: Option<Instant>,
    /// What the screen was built from (its markers), to rebuild when it moves.
    built: u64,
    /// Glow of woken waystones nearby: when it last rose.
    glowed: Option<Instant>,
    rng: u64,
    /// Whether the map was open last frame: closing it forgets the travel and the pan.
    was_open: bool,
}

static STATE: Mutex<State> = Mutex::new(State {
    atlas: Atlas { regions: std::collections::BTreeMap::new(), woken: Vec::new() },
    seed: 0,
    zoom: n::ZOOM_START,
    pan: (0, 0),
    from: None,
    asked: None,
    painted: None,
    built: 0,
    glowed: None,
    rng: 0x9E37_79B9_7F4A_7C15,
    was_open: false,
});

fn change(c: &ClientView, f: impl FnOnce(&mut State)) {
    f(&mut STATE.lock().unwrap());
    c.client().rebuild_screen();
}

pub fn register(r: &mut Registry) {
    r.key(&OPEN_KEY, "M").on_press_client(|c| {
        STATE.lock().unwrap().from = None;
        open(c);
    });
    r.client_packet(&ATLAS, |c, bytes| {
        let Some(a) = Atlas::decode(&bytes) else { return };
        let Some(seed) = crate::ley::seed_here(&c.view()) else { return };
        let me = c
            .view()
            .player_state()
            .map_or((0, 0), |s| ((s.pos.x as i32).div_euclid(n::TILE), (s.pos.z as i32).div_euclid(n::TILE)));
        queue_tiles(seed, &a, me);
        let mut s = STATE.lock().unwrap();
        s.atlas = a;
        s.seed = seed;
        s.painted = None;
    });
    r.client_packet(&OPEN, |c, cell| {
        STATE.lock().unwrap().from = Some((cell.i, cell.j));
        open(c);
    });
    r.client_frame(frame);
    r.screen(&SCREEN, screen).on_key(|c, key| {
        let z = n::ZOOMS[STATE.lock().unwrap().zoom];
        let step = 32 * z;
        let pan = match key {
            "KeyW" | "ArrowUp" => Some((0, -step)),
            "KeyS" | "ArrowDown" => Some((0, step)),
            "KeyA" | "ArrowLeft" => Some((-step, 0)),
            "KeyD" | "ArrowRight" => Some((step, 0)),
            _ => None,
        };
        if let Some((dx, dz)) = pan {
            change(c, |s| s.pan = (s.pan.0 + dx, s.pan.1 + dz));
        }
        match key {
            "KeyZ" | "Minus" => change(c, |s| s.zoom = (s.zoom + 1).min(n::ZOOMS.len() - 1)),
            "KeyX" | "Equal" => change(c, |s| s.zoom = s.zoom.saturating_sub(1)),
            "KeyC" | "Home" => change(c, |s| s.pan = (0, 0)),
            "KeyM" => c.client().screen_close(),
            _ => {}
        }
    });
}

fn open(c: &Client) {
    if crate::ley::seed_here(&c.view()).is_none() {
        c.hint("hint.base.map.not_here", &[]);
        return;
    }
    c.send(&ASK, &());
    {
        let mut s = STATE.lock().unwrap();
        s.asked = Some(Instant::now());
        s.painted = None;
    }
    paint(c);
    c.screen_open(&SCREEN);
}

// ------------------------------------------------------------------ every frame

fn frame(c: &Client) {
    let view = c.view();
    let Some(seed) = crate::ley::seed_here(&view) else { return };
    let now = Instant::now();
    let open = c.screen() == "base:map";
    {
        let mut s = STATE.lock().unwrap();
        // A new world (or the first frame): ask for the atlas once, for the waystones' glow.
        let ask_every = if open { ASK_SECS } else { 30.0 };
        if s.seed != seed
            || s.asked
                .is_none_or(|t| now.duration_since(t).as_secs_f32() >= ask_every)
        {
            if s.seed != seed {
                s.atlas = Atlas::default();
                s.seed = seed;
            }
            s.asked = Some(now);
            c.send(&ASK, &());
        }
        if s.was_open && !open {
            s.from = None;
            s.pan = (0, 0);
        }
        s.was_open = open;
    }
    glow(c, seed, now);
    if !open {
        return;
    }
    let due = STATE
        .lock()
        .unwrap()
        .painted
        .is_none_or(|t| now.duration_since(t).as_secs_f32() >= REPAINT_SECS);
    if due {
        paint(c);
    }
    let sig = signature(&view);
    let mut s = STATE.lock().unwrap();
    if sig != s.built {
        s.built = sig;
        drop(s);
        c.rebuild_screen();
    }
}

/// What the screen's markers stand on: the view's middle by the pixel, the zoom, the woken.
fn signature(view: &ClientView) -> u64 {
    let s = STATE.lock().unwrap();
    let (cx, cz) = centre(view, &s);
    let z = n::ZOOMS[s.zoom];
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut mix = |v: i64| h = (h ^ v as u64).wrapping_mul(0x100_0000_01b3);
    mix(cx.div_euclid(z) as i64);
    mix(cz.div_euclid(z) as i64);
    mix(s.zoom as i64);
    mix(s.woken_len() as i64);
    mix(s.from.map_or(-1, |f| f.0 as i64 * 7919 + f.1 as i64));
    h
}

impl State {
    fn woken_len(&self) -> usize {
        self.atlas.woken.len()
    }
}

/// The block at the view's middle: the player, moved by the pan.
fn centre(view: &ClientView, s: &State) -> (i32, i32) {
    let p = view
        .player_state()
        .map_or((0, 0), |st| (st.pos.x.floor() as i32, st.pos.z.floor() as i32));
    (p.0 + s.pan.0, p.1 + s.pan.1)
}

/// Where block (`x`, `z`) lands in the picture (GUI pixels), at this view.
fn place(c: (i32, i32), z: i32, x: i32, zz: i32) -> (f32, f32) {
    ((x - c.0) as f32 / z as f32 + VIEW / 2.0, (zz - c.1) as f32 / z as f32 + VIEW / 2.0)
}

/// Woken waystones within reach give off a slow rise of light, for this player alone.
fn glow(c: &Client, seed: u64, now: Instant) {
    let mut s = STATE.lock().unwrap();
    if s.atlas.woken.is_empty()
        || s.glowed
            .is_some_and(|t| now.duration_since(t).as_secs_f32() < 0.3)
    {
        return;
    }
    s.glowed = Some(now);
    let Some(me) = c.view().player_state() else { return };
    let (x, y, z) = (me.pos.x, me.pos.y.floor() as i32, me.pos.z);
    let way = crate::ley::WAYSTONE.id();
    for &(i, j) in &s.atlas.woken.clone() {
        let Some(nd) = ley::node(seed, i, j) else { continue };
        if (nd.x as f64 - x).hypot(nd.z as f64 - z) > 64.0 {
            continue;
        }
        let Some(top) = (-24..=24)
            .map(|dy| y + dy)
            .find(|&yy| c.block([nd.x, yy, nd.z]) == Some(way))
        else {
            continue;
        };
        for _ in 0..2 {
            s.rng ^= s.rng << 13;
            s.rng ^= s.rng >> 7;
            s.rng ^= s.rng << 17;
            let (dx, dz) = (((s.rng >> 8) % 100) as f64 / 100.0 - 0.5, ((s.rng >> 24) % 100) as f64 / 100.0 - 0.5);
            drop(
                c.spawn_particles(&crate::waystones::FLARE, [nd.x as f64 + 0.5 + dx, top as f64 + 1.1, nd.z as f64 + 0.5 + dz])
                    .velocity([0.0, 0.6, 0.0]),
            );
        }
    }
}

// ------------------------------------------------------------------ the picture

/// Paints the map's picture for the view now.
fn paint(c: &Client) {
    let view = c.view();
    let Some(seed) = crate::ley::seed_here(&view) else { return };
    let (centre, zoom, atlas_woken, me) = {
        let mut s = STATE.lock().unwrap();
        s.painted = Some(Instant::now());
        (centre(&view, &s), n::ZOOMS[s.zoom], s.atlas.woken.clone(), view.player_state())
    };
    let side = n::MAP_PX as usize;
    let mut img = vec![255u8; side * side * 4];
    let half = side as i32 / 2;
    let w = work();
    {
        let s = STATE.lock().unwrap();
        let done = &w.0.lock().unwrap().done;
        for j in 0..side {
            let bz = centre.1 + (j as i32 - half) * zoom;
            let (tz, lz) = (bz.div_euclid(n::TILE), bz.rem_euclid(n::TILE));
            let mut last: Option<(i32, bool, Option<&Colours>)> = None;
            for i in 0..side {
                let bx = centre.0 + (i as i32 - half) * zoom;
                let (tx, lx) = (bx.div_euclid(n::TILE), bx.rem_euclid(n::TILE));
                let (seen, cols) = match last {
                    Some((t, seen, cols)) if t == tx => (seen, cols),
                    _ => {
                        let seen = s.atlas.seen(tx, tz);
                        let cols = if seen { done.get(&(tx, tz)) } else { None };
                        last = Some((tx, seen, cols));
                        (seen, cols)
                    }
                };
                let rgb = match (seen, cols) {
                    (true, Some(cols)) => cols[(lz / n::TILE_PX_BLOCKS) as usize * TPX + (lx / n::TILE_PX_BLOCKS) as usize],
                    (true, None) => n::PARCHMENT_DARK,
                    _ => parchment(bx.div_euclid(4 * zoom), bz.div_euclid(4 * zoom)),
                };
                let o = (j * side + i) * 4;
                img[o..o + 3].copy_from_slice(&rgb);
            }
        }
        // The explored land's edge, inked: a seen pixel next to an unseen one darkens.
        let seen_at = |i: i32, j: i32| {
            s.atlas
                .seen((centre.0 + (i - half) * zoom).div_euclid(n::TILE), (centre.1 + (j - half) * zoom).div_euclid(n::TILE))
        };
        for j in 1..side as i32 - 1 {
            for i in 1..side as i32 - 1 {
                if seen_at(i, j) && !(seen_at(i - 1, j) && seen_at(i + 1, j) && seen_at(i, j - 1) && seen_at(i, j + 1)) {
                    let o = (j as usize * side + i as usize) * 4;
                    for k in 0..3 {
                        img[o + k] = (img[o + k] as u16 * 3 / 5) as u8;
                    }
                }
            }
        }
        // The ley lines, over the explored land, with the rod or ley sight.
        if crate::ley::sees_lines(c) {
            let reach = half * zoom + ley::CELL;
            for li in ley::lines_near(seed, centre.0, centre.1, reach) {
                let (a, b) = (place(centre, zoom, li.from[0], li.from[1]), place(centre, zoom, li.to[0], li.to[1]));
                let steps = ((b.0 - a.0).abs().max((b.1 - a.1).abs()).ceil() as i32).max(1);
                for k in 0..=steps {
                    let t = k as f32 / steps as f32;
                    let (pi, pj) = ((a.0 + (b.0 - a.0) * t) as i32, (a.1 + (b.1 - a.1) * t) as i32);
                    if (0..side as i32).contains(&pi) && (0..side as i32).contains(&pj) && seen_at(pi, pj) {
                        let o = (pj as usize * side + pi as usize) * 4;
                        for (k, &v) in n::MAP_LEY.iter().enumerate() {
                            img[o + k] = ((img[o + k] as u16 * 2 + v as u16 * 3) / 5) as u8;
                        }
                    }
                }
            }
        }
    }
    // Woken waystones: a small diamond each (the markers over it are the buttons).
    for &(i, j) in &atlas_woken {
        let Some(nd) = ley::node(seed, i, j) else { continue };
        let (px, pz) = place(centre, zoom, nd.x, nd.z);
        for dz in -3i32..=3 {
            for dx in -3i32..=3 {
                let (x, y) = (px as i32 + dx, pz as i32 + dz);
                if dx.abs() + dz.abs() <= 3 && (0..side as i32).contains(&x) && (0..side as i32).contains(&y) {
                    let o = (y as usize * side + x as usize) * 4;
                    let edge = dx.abs() + dz.abs() == 3;
                    img[o..o + 3].copy_from_slice(&if edge { [0x1c, 0x34, 0x48] } else { n::MAP_WOKEN });
                }
            }
        }
    }
    // The player: an arrow the way they face.
    if let Some(st) = me {
        let (px, pz) = place(centre, zoom, st.pos.x.floor() as i32, st.pos.z.floor() as i32);
        let [fx, fz] = yaw_dir(st.yaw);
        for dz in -5i32..=5 {
            for dx in -5i32..=5 {
                // Along the facing (a) and across it (b).
                let (a, b) = (dx as f32 * fx + dz as f32 * fz, dx as f32 * -fz + dz as f32 * fx);
                let inside = (-2.5..=4.5).contains(&a) && b.abs() <= (4.5 - a) * 0.55;
                let rim = (-3.5..=5.5).contains(&a) && b.abs() <= (5.5 - a) * 0.6;
                let (x, y) = (px as i32 + dx, pz as i32 + dz);
                if !(0..side as i32).contains(&x) || !(0..side as i32).contains(&y) {
                    continue;
                }
                let o = (y as usize * side + x as usize) * 4;
                if inside {
                    img[o..o + 3].copy_from_slice(&n::MAP_PLAYER);
                } else if rim {
                    img[o..o + 3].copy_from_slice(&[0x3a, 0x14, 0x10]);
                }
            }
        }
    }
    c.set_image(IMAGE, n::MAP_PX, n::MAP_PX, &img);
}

/// The blank parchment's grain at a coarse cell.
fn parchment(x: i32, z: i32) -> [u8; 3] {
    let h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (z as u32).wrapping_mul(0x85EB_CA77);
    let h = (h ^ (h >> 15)).wrapping_mul(0x2C1B_3C6D);
    if h >> 28 < 3 { n::PARCHMENT_DARK } else { n::PARCHMENT }
}

// ------------------------------------------------------------------ the screen

fn screen(ui: &mut Ui) {
    let view = ui.view();
    let (centre, zoom, woken, from, seed) = {
        let s = STATE.lock().unwrap();
        (centre(&view, &s), n::ZOOMS[s.zoom], s.atlas.woken.clone(), s.from, s.seed)
    };
    let tiles = STATE.lock().unwrap().atlas.tiles();
    let way_item = Item::lookup("base:waystone").unwrap_or(0);
    let here = view
        .player_state()
        .map_or((0, 0), |st| (st.pos.x.floor() as i32, st.pos.z.floor() as i32));
    crate::window::window_titled(ui, S::row().pad(6.0).gap(6.0), "map.base.title", &[], |ui| {
        ui.column(S::column().gap(3.0), |ui| {
            ui.row(S::row().gap(3.0).center(), |ui| {
                ui.text_styled(S::default().class("dim").no_wrap(), "map.base.keys", &[]);
                ui.spacer();
                ui.button(
                    "map.base.zoom_out",
                    S::default()
                        .size(12.0, 12.0)
                        .tooltip("map.base.zoom_out_tip"),
                    |c| change(c, |s| s.zoom = (s.zoom + 1).min(n::ZOOMS.len() - 1)),
                );
                ui.button(
                    "map.base.zoom_in",
                    S::default()
                        .size(12.0, 12.0)
                        .tooltip("map.base.zoom_in_tip"),
                    |c| change(c, |s| s.zoom = s.zoom.saturating_sub(1)),
                );
                ui.button("map.base.centre", S::default().h(12.0).pad(2.0).tooltip("map.base.centre_tip"), |c| {
                    change(c, |s| s.pan = (0, 0))
                });
            });
            ui.panel("well", S::column().size(VIEW, VIEW), |ui| {
                ui.image(IMAGE, VIEW, VIEW);
                // The woken waystones: from one, the others are buttons to travel to.
                for &(i, j) in &woken {
                    let Some(nd) = ley::node(seed, i, j) else { continue };
                    let p = place(centre, zoom, nd.x, nd.z);
                    if !(4.0..=VIEW - 4.0).contains(&p.0) || !(4.0..=VIEW - 4.0).contains(&p.1) {
                        continue;
                    }
                    let dist = ((nd.x - here.0) as f64)
                        .hypot((nd.z - here.1) as f64)
                        .round() as i64;
                    let can_go = from.is_some_and(|f| f != (i, j));
                    let s = S::column()
                        .size(10.0, 10.0)
                        .at_pivot(p.0.round(), p.1.round(), [0.5, 0.5])
                        .tooltip_args(
                            if can_go { "map.base.travel_tip" } else { "map.base.waystone_tip" },
                            &[Arg::Int(nd.x as i64), Arg::Int(nd.z as i64), Arg::Int(dist)],
                        );
                    if can_go {
                        ui.button_with(
                            s,
                            move |c| go(c, (i, j)),
                            |ui| {
                                ui.item(way_item, 1, S::default().size(8.0, 8.0));
                            },
                        );
                    } else {
                        ui.column(s, |_| {});
                    }
                }
            });
        });
        ui.column(S::column().gap(3.0).w(SIDE_W), |ui| {
            ui.text_styled(S::default().no_wrap().mono(), "map.base.here", &[Arg::Int(here.0 as i64), Arg::Int(here.1 as i64)]);
            ui.text_styled(
                S::default().class("dim").w(SIDE_W),
                "map.base.known",
                &[Arg::Int(tiles as i64 * (n::TILE * n::TILE) as i64 / 10_000), Arg::Int(woken.len() as i64)],
            );
            if let Some(f) = from {
                ui.text_styled(S::default().class("title").w(SIDE_W), "map.base.travel_title", &[]);
                let mut others: Vec<(i64, (i32, i32), ley::Node)> = woken
                    .iter()
                    .filter(|&&c| c != f)
                    .filter_map(|&(i, j)| {
                        ley::node(seed, i, j).map(|nd| {
                            (
                                ((nd.x - here.0) as f64)
                                    .hypot((nd.z - here.1) as f64)
                                    .round() as i64,
                                (i, j),
                                nd,
                            )
                        })
                    })
                    .collect();
                others.sort_by_key(|o| o.0);
                if others.is_empty() {
                    ui.text_styled(S::default().class("dim").w(SIDE_W), "map.base.travel_none", &[]);
                }
                ui.column(S::column().gap(1.0).scroll().h(VIEW - 60.0), |ui| {
                    for (dist, cell, nd) in others {
                        ui.button_args(
                            "map.base.travel_to",
                            &[Arg::Int(nd.x as i64), Arg::Int(nd.z as i64), Arg::Int(dist)],
                            S::default().class("list_row").w(SIDE_W - 8.0).pad(2.0),
                            move |c| go(c, cell),
                        );
                    }
                });
            } else {
                ui.text_styled(S::default().class("dim").w(SIDE_W), "map.base.travel_how", &[]);
            }
        });
    });
}

/// Asks the server to travel to `cell`'s waystone and closes the map (the channel's hint follows).
fn go(c: &ClientView, cell: (i32, i32)) {
    let client = c.client();
    client.send(&TRAVEL, &Cell { i: cell.0, j: cell.1 });
    client.screen_close();
}
