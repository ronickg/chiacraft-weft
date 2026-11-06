//! Ley lines in the game (`vitals::ley`, `notes/PROGRESSION.md` "Ley lines" and "Round 6": "seen
//! with the right tools. Invisible at first; a dowsing rod, then ley sight from magic, shows them as
//! faint flowing lines, so finding them is part of exploring").
//!
//! - **The dowsing rod** (Stone Age, the magic path's first step): two sticks, cordage and a
//!   tektite. Held, the hand holding it dips and swings toward the nearest line, harder near one and hardest
//!   at a node, ticking like a soft chime faster the stronger the pull, and a hint says how it
//!   pulls. It also shows the lines within `ley::SHOWN_ROD` blocks as faint motes flowing along
//!   them. All of it is the client's own: the server only hands it the world's seed (`SEED`), and
//!   the lines come from the seed alike on both sides.
//! - **Ley sight** (`base_api::LEY_SIGHT`, the magic path's perk) shows the lines without a rod,
//!   further out.
//! - **Waystones**: every crossing of `ley::WAYSTONE_LINES` lines or more carries a sleeping
//!   waystone (`WAYSTONE`, woken later) on a structure of ancient ley stone suited to its biome,
//!   built by a decoration from the node and the land's columns alone.
//!
//! `/ley` says what the rod would feel where you stand, and where the nearest line, node and
//! waystone are.

use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;
use std::time::Instant;

use vitals::ley::{self, Feel, Node};
use weft_sdk::*;

pub static DOWSING_ROD: Item = Item::named("dowsing_rod");
/// The ancients' carved stone the waystones stand on.
pub static LEY_STONE: Block = Block::named("ley_stone");
/// A sleeping waystone on a strong node: woken later (the map's travel points).
pub static WAYSTONE: Block = Block::named("waystone");
static MOSSY_COBBLESTONE: Block = Block::named("mossy_cobblestone");
static SANDSTONE: Block = Block::named("sandstone");
/// A faint speck of the line's light, flowing along it.
static MOTE: Particle = Particle::named("ley_mote");
/// The rod's tick: a soft glassy chime.
static TWITCH: Sound = Sound::named("ley.twitch");
/// The world's seed for the client's lines, 1 in `[2]` while the player is in the overworld
/// (the lines are the overworld's). `[lo, hi, here]`.
static SEED: Component<[u32; 3]> = Component::named("ley_seed");

pub fn register(r: &mut Registry) {
    // Held on use at a sleeping waystone it wakes it (`waystones.rs`).
    r.item(&DOWSING_ROD)
        .max_stack(1)
        .handheld()
        .use_ticks(vitals::waystones::WAKE_TICKS)
        .on_use(crate::waystones::rod_used)
        .tag("dowsing_rods")
        .tab(&crate::tabs::TOOLS);
    // A forked branch lashed with cordage, a tektite (star glass, the ancients' own) at its tip.
    r.shapeless(&base_api::CRAFTING, "dowsing_rod", 1, &["#stick", "#stick", "#string", "#tektite"], 0);
    r.block(&LEY_STONE)
        .sound_group("stone")
        .hardness(2.5)
        .mineable("pickaxe")
        .needs_tool_level(0)
        .on_use(crate::waystones::stone_used)
        .textures("ley_stone_top", "ley_stone_side", "ley_stone_top")
        .tag("ley_stone")
        .tab(&crate::tabs::BUILDING);
    // Not to be carried off: waking it (later) is what it is for.
    r.block(&WAYSTONE)
        .sound_group("stone")
        .unbreakable()
        .on_use(crate::waystones::stone_used)
        .textures("waystone_top", "waystone_side", "ley_stone_top")
        .tag("waystones")
        .tab(&crate::tabs::BUILDING);
    r.particle(&MOTE)
        .textures(&["core:glint_0", "core:glint_1", "core:glint_2", "core:glint_3"])
        .life(ley::MOTE_LIFE.0, ley::MOTE_LIFE.1)
        .size(0.16, 0.28)
        .gravity(0.0)
        .drag(1.0)
        .color(0xD0FAFF, 0x8C9CFF)
        .fullbright()
        .translucent()
        .shrinking();
    r.sound(&TWITCH)
        .files(&["random/ley_twitch"])
        .category(&sound_categories::PLAYERS)
        .pitch(0.95, 1.05);
    r.entity_component(&SEED, [0; 3], component_flags::SYNC_OWNER);
    r.attach(&PLAYER, &SEED);
    r.entity_component(&base_api::LEY_SIGHT, 0.0, component_flags::SAVE | component_flags::SYNC_OWNER | component_flags::KEEP_ON_RESPAWN);
    r.attach(&PLAYER, &base_api::LEY_SIGHT);
    r.world_tick(hand_seed);
    r.decoration(waystones).dimension(&crate::OVERWORLD);
    r.client_frame(frame);
    r.command("ley", "", |cmd| {
        let w = cmd.world();
        let Some(st) = w.entity(cmd.player()) else { return };
        let (x, z) = (st.pos.x, st.pos.z);
        let seed = w.seed();
        let r = ley::read(seed, x, z);
        let line = r
            .line
            .map_or((-1, 0, 0), |l| (l.dist.round() as i64, l.at[0].round() as i64, l.at[1].round() as i64));
        let node = ley::nodes_near(seed, x as i32, z as i32, 2 * ley::CELL)
            .into_iter()
            .min_by_key(|n| dist2(n, x, z));
        let stone = ley::waystones_near(seed, x as i32, z as i32, 4 * ley::CELL)
            .into_iter()
            .min_by_key(|n| dist2(n, x, z));
        let at = |n: Option<Node>| {
            n.map_or((0, 0, 0, -1), |n| (n.x as i64, n.z as i64, n.lines as i64, (dist2(&n, x, z) as f64).sqrt().round() as i64))
        };
        let (nx, nz, nl, nd) = at(node);
        let (sx, sz, _, sd) = at(stone);
        cmd.reply(
            "command.base.ley",
            &[
                Arg::Float(r.strength as f64, 2),
                Arg::Key(&format!("ley.base.feel.{}", Feel::of(&r).name())),
                Arg::Int(line.0),
                Arg::Int(line.1),
                Arg::Int(line.2),
                Arg::Int(nx),
                Arg::Int(nz),
                Arg::Int(nl),
                Arg::Int(nd),
                Arg::Int(sx),
                Arg::Int(sz),
                Arg::Int(sd),
            ],
        );
    });
}

fn dist2(n: &Node, x: f64, z: f64) -> i64 {
    let (dx, dz) = (n.x as i64 - x.floor() as i64, n.z as i64 - z.floor() as i64);
    dx * dx + dz * dz
}

/// Every second, each player's seed for its client's lines (none outside the overworld), and
/// its ley sight from the perk web.
fn hand_seed(w: &World) {
    if !w.game_time().is_multiple_of(20) {
        return;
    }
    let here = (w.dimension() == crate::OVERWORLD.id()) as u32;
    let seed = w.seed();
    let want = [seed as u32, (seed >> 32) as u32, here];
    for p in w.players() {
        if let Some(s) = w.component_mut(p, &SEED)
            && *s != want
        {
            *s = want;
        }
        // Ley sight from the perk web (Runecraft's node, or a socketed relic): the lines without
        // a rod. The web is the truth; the component is what the client reads.
        let sight = if base_api::skills::has(w, p, base_api::skills::Special::LeySight) { 1.0 } else { 0.0 };
        if let Some(v) = w.component_mut(p, &base_api::LEY_SIGHT)
            && *v != sight
        {
            *v = sight;
        }
    }
}

/// The world's seed on the client while its player is in the overworld (the lines and the map are
/// the overworld's), from the server (`SEED`).
pub(crate) fn seed_here(view: &ClientView) -> Option<u64> {
    view.component(&SEED)
        .filter(|s| s[2] != 0)
        .map(|[lo, hi, _]| lo as u64 | (hi as u64) << 32)
}

/// Whether the local player sees the ley lines now: the rod in a hand, or ley sight.
pub(crate) fn sees_lines(c: &Client) -> bool {
    let view = c.view();
    let me = view.player();
    let rod = DOWSING_ROD.id();
    let holding = rod != u32::MAX && (c.held_item(me, Hand::Main) == rod || c.held_item(me, Hand::Off) == rod);
    holding || view.component(&base_api::LEY_SIGHT).unwrap_or(0.0) > 0.0
}

// ------------------------------------------------------------------ the waystones

/// How a waystone's structure is built, by its biome.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Style {
    /// A pillar on a plinth with four small stones round it: the open lands and woods.
    Menhir { mossy: bool },
    /// A taller pillar on a stepped sandstone base: desert and savanna.
    Obelisk,
    /// A cairn of piled stones: the cold and the heights.
    Cairn,
    /// A ring of mossy stones round a low plinth, half sunk: swamp and jungle.
    Sunken,
    /// A plinth on the sea's bed.
    Seabed,
}

impl Style {
    fn of(biome: &str, under_water: bool) -> Style {
        if under_water {
            return Style::Seabed;
        }
        match biome.trim_start_matches("base:") {
            "desert" | "savanna" => Style::Obelisk,
            "mountains" | "snowy_plains" | "taiga" => Style::Cairn,
            "swamp" | "tropical_forest" => Style::Sunken,
            "ocean" | "coral_reef" | "lagoon" => Style::Seabed,
            "forest" | "birch_forest" | "sequoia_grove" | "old_growth_forest" => Style::Menhir { mossy: true },
            _ => Style::Menhir { mossy: false },
        }
    }
}

/// How far a waystone's structure reaches from its node (blocks, each axis).
const STONE_REACH: i32 = 3;

/// The blocks of `node`'s waystone and what stands round it: (position, block, whether it
/// replaces only what isn't solid). `ground(x, z)` is a column's top block.
fn waystone_blocks(node: &Node, ground: &dyn Fn(i32, i32) -> i32, biome: &str) -> Vec<([i32; 3], u32, bool)> {
    let (x, z) = (node.x, node.z);
    let gy = ground(x, z);
    let style = Style::of(biome, gy < base_api::SEA - 1);
    let (stone, way) = (LEY_STONE.id(), WAYSTONE.id());
    let mut out = Vec::new();
    let mut put = |p: [i32; 3], b: u32| out.push((p, b, false));
    let plinth = |put: &mut dyn FnMut([i32; 3], u32), y: i32, b: u32| {
        for dz in -1..=1 {
            for dx in -1..=1 {
                put([x + dx, y, z + dz], b);
            }
        }
    };
    match style {
        Style::Menhir { mossy } => {
            plinth(&mut put, gy, stone);
            put([x, gy + 1, z], stone);
            put([x, gy + 2, z], stone);
            put([x, gy + 3, z], way);
            let small = if mossy { MOSSY_COBBLESTONE.id() } else { stone };
            for (dx, dz) in [(STONE_REACH, 0), (-STONE_REACH, 0), (0, STONE_REACH), (0, -STONE_REACH)] {
                let h = ground(x + dx, z + dz);
                put([x + dx, h + 1, z + dz], small);
            }
        }
        Style::Obelisk => {
            let sand = if SANDSTONE.id() != u32::MAX { SANDSTONE.id() } else { stone };
            for dz in -2..=2 {
                for dx in -2..=2i32 {
                    put([x + dx, ground(x + dx, z + dz), z + dz], sand);
                }
            }
            plinth(&mut put, gy + 1, stone);
            for dy in 2..=4 {
                put([x, gy + dy, z], stone);
            }
            put([x, gy + 5, z], way);
        }
        Style::Cairn => {
            let rubble = crate::blocks::COBBLESTONE.id();
            for dz in -1..=1i32 {
                for dx in -1..=1i32 {
                    let h = ground(x + dx, z + dz);
                    put([x + dx, h + 1, z + dz], if dx.abs() + dz.abs() == 2 { rubble } else { stone });
                }
            }
            for (dx, dz) in [(0, 0), (1, 0), (0, 1), (-1, 0), (0, -1)] {
                put([x + dx, gy + 2, z + dz], if (dx, dz) == (0, 0) { stone } else { rubble });
            }
            put([x, gy + 3, z], way);
        }
        Style::Sunken => {
            let moss = MOSSY_COBBLESTONE.id();
            plinth(&mut put, gy, moss);
            put([x, gy, z], stone);
            put([x, gy + 1, z], way);
            for (dx, dz) in [(2, 0), (-2, 0), (0, 2), (0, -2), (2, 2), (-2, -2), (2, -2), (-2, 2)] {
                let h = ground(x + dx, z + dz);
                put([x + dx, h, z + dz], moss);
                if dx == 0 || dz == 0 {
                    put([x + dx, h + 1, z + dz], stone);
                }
            }
        }
        Style::Seabed => {
            plinth(&mut put, gy, stone);
            put([x, gy + 1, z], stone);
            put([x, gy + 2, z], way);
        }
    }
    // Clear plants off the plinth's top and the pillar's foot (not water: the seabed stays wet), at
    // each column's own ground and at the plinth's: tall grass on the plinth stood in the way of a
    // click on the stone (PC look wm_look2).
    if !matches!(style, Style::Seabed) {
        for dz in -2..=2 {
            for dx in -2..=2 {
                for y in [ground(x + dx, z + dz) + 1, gy + 1, gy + 2] {
                    out.push(([x + dx, y, z + dz], 0, true));
                }
            }
        }
    }
    out
}

/// The sleeping waystones on the strong nodes (`vitals::ley`), each cube its part.
fn waystones(g: &mut Gen) {
    let [ox, oy, oz] = g.origin().to_array();
    let seed = g.seed();
    let nodes = ley::waystones_near(seed, ox + 8, oz + 8, 8 + STONE_REACH);
    if nodes.is_empty() || WAYSTONE.id() == u32::MAX {
        return;
    }
    let inside = |p: [i32; 3]| (ox..ox + 16).contains(&p[0]) && (oy..oy + 16).contains(&p[1]) && (oz..oz + 16).contains(&p[2]);
    for n in nodes {
        // Columns asked once each: a structure is at most 7 by 7.
        let heights: HashMap<(i32, i32), i32> = (-STONE_REACH..=STONE_REACH)
            .flat_map(|dz| (-STONE_REACH..=STONE_REACH).map(move |dx| (n.x + dx, n.z + dz)))
            .map(|(x, z)| ((x, z), g.column(x, z).height))
            .collect();
        let ground = |x: i32, z: i32| heights.get(&(x, z)).copied().unwrap_or(i32::MIN / 2);
        let gy = ground(n.x, n.z);
        if gy - 4 > oy + 15 || gy + 8 < oy {
            continue;
        }
        let biome = name_of(abi::entry_kind::BIOME, g.column(n.x, n.z).biome);
        for (p, b, soft) in waystone_blocks(&n, &ground, biome) {
            if !inside(p) || b == u32::MAX {
                continue;
            }
            let (x, y, z) = ((p[0] - ox) as usize, (p[1] - oy) as usize, (p[2] - oz) as usize);
            if soft {
                let now = g.get(x, y, z);
                if now == 0 || liquid_of(now).is_some() || state_flags(now) & abi::block_flags::SOLID != 0 {
                    continue;
                }
            }
            g.set(x, y, z, b);
        }
    }
}

/// A ley node's heart opened: what waking a sleeping waystone does (`waystones.rs`, `notes/
/// PROGRESSION.md` "From the survival research": waystones, ancient ley nodes you wake), with the
/// age it happens in; the perk web's socket stone may drop at the stone (`StoneSource::LeyNode`: a
/// quarter of the time, depth 5). True when one dropped.
pub(crate) fn heart_opened(w: &World, node: &Node, tier: u8, top: i32) -> bool {
    let at = [node.x as f64 + 0.5, top as f64 + 1.2, node.z as f64 + 0.5];
    base_api::skills::drop_stone(w, base_api::skills::StoneSource::LeyNode, tier, at)
}

// ------------------------------------------------------------------ the rod and the lines, client

/// What the client keeps between frames.
struct Rod {
    last: Option<Instant>,
    rng: u64,
    /// Motes owed (fractions carried over).
    owed: f32,
    /// Seconds to the next tick of the chime.
    tick_in: f32,
    /// The stem's dip and swing now (degrees), eased toward the reading's.
    dip: f32,
    swing: f32,
    /// Seconds the stem has twitched, for its jitter.
    time: f32,
    /// The hint last shown and when.
    feel: Option<(Feel, &'static str)>,
    hinted: Option<Instant>,
    /// Ground heights the motes stand on, by column, and the seed they belong to.
    ground: BTreeMap<(i32, i32), i32>,
    ground_seed: u64,
}

static ROD: Mutex<Rod> = Mutex::new(Rod {
    last: None,
    rng: 0x2545_F491_4F6C_DD1D,
    owed: 0.0,
    tick_in: 0.0,
    dip: 0.0,
    swing: 0.0,
    time: 0.0,
    feel: None,
    hinted: None,
    ground: BTreeMap::new(),
    ground_seed: 0,
});

impl Rod {
    /// 0 to 1.
    fn unit(&mut self) -> f64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn frame(c: &Client) {
    let mut s = ROD.lock().unwrap();
    let now = Instant::now();
    let dt = s
        .last
        .map_or(0.0, |l| now.duration_since(l).as_secs_f32())
        .min(0.25);
    s.last = Some(now);
    let view = c.view();
    let Some(st) = view.player_state() else { return };
    let Some([lo, hi, here]) = view.component(&SEED) else { return };
    let me = view.player();
    let rod = DOWSING_ROD.id();
    let holding = rod != u32::MAX && (c.held_item(me, Hand::Main) == rod || c.held_item(me, Hand::Off) == rod);
    let sight = view.component(&base_api::LEY_SIGHT).unwrap_or(0.0);
    if here == 0 || (!holding && sight <= 0.0) {
        s.dip = 0.0;
        s.swing = 0.0;
        if s.feel.take().is_some() {
            c.show("", &[]).id("base:ley_rod");
        }
        return;
    }
    let seed = lo as u64 | (hi as u64) << 32;
    if s.ground_seed != seed {
        s.ground.clear();
        s.ground_seed = seed;
    }
    let (x, z) = (st.pos.x, st.pos.z);
    let reading = ley::read(seed, x, z);
    if holding {
        rod_feel(c, &mut s, me, &st, &reading, dt, now);
    }
    let shown = if sight > 0.0 { ley::SHOWN_ROD.max(ley::SHOWN_SIGHT * sight.min(1.0) as f64) } else { ley::SHOWN_ROD };
    motes(c, &mut s, seed, [x, st.pos.y, z], shown, dt);
}

/// The rod's twitch, chime and hint.
fn rod_feel(c: &Client, s: &mut Rod, me: u64, st: &abi::EntityState, r: &ley::Reading, dt: f32, now: Instant) {
    let feel = Feel::of(r);
    let pull = (r.strength / ley::STRONG).min(2.5);
    // Toward the nearest line: its angle off the way the player faces, -180 to 180 (right +).
    let off = r.line.map(|l| {
        let [fx, fz] = yaw_dir(st.yaw);
        let (dx, dz) = ((l.at[0] - st.pos.x) as f32, (l.at[1] - st.pos.z) as f32);
        // Right of facing is (-fz, fx).
        let right = dx * -fz + dz * fx;
        let ahead = dx * fx + dz * fz;
        right.atan2(ahead).to_degrees()
    });
    s.time += dt * (4.0 + 10.0 * pull);
    let jitter = if feel > Feel::Still { (s.time.sin() * 0.6 + (s.time * 2.3).sin() * 0.4) * (0.8 + 2.2 * pull.min(1.5)) } else { 0.0 };
    let (want_dip, want_swing) = match feel {
        Feel::Still => (0.0, 0.0),
        Feel::AtNode => (-16.0, (s.time * 0.9).sin() * 14.0),
        _ => (-3.0 - 8.0 * pull.min(1.5), off.unwrap_or(0.0).clamp(-40.0, 40.0) * 0.45 * pull.min(1.0)),
    };
    let ease = 1.0 - (-dt * 6.0).exp();
    s.dip += (want_dip - s.dip) * ease;
    s.swing += (want_swing - s.swing) * ease;
    // The arm that holds it (`leftArm`, Weft's main arm at +x): items are flat pictures, so the
    // hand dips and swings, not a bone of the rod. The bone's turn about its up axis is positive
    // to the left, so a swing to the right (positive) turns it negative (PC look wm_twitch2: "tugs
    // hard to the right" swung the rod left).
    c.bone_turn(me, "leftArm", [s.dip + jitter, -(s.swing + jitter * 0.5), 0.0]);
    // The chime: faster and louder the stronger the pull.
    if feel > Feel::Still {
        s.tick_in -= dt;
        if s.tick_in <= 0.0 {
            s.tick_in = (1.6 / (0.6 + 2.0 * pull)).clamp(0.16, 2.0);
            let pitch = if feel == Feel::AtNode { 1.5 } else { 0.85 + 0.25 * pull.min(1.6) };
            drop(
                c.play_sound(&TWITCH, None, 0.18 + 0.2 * pull.min(1.5))
                    .pitch(pitch),
            );
        }
    } else {
        s.tick_in = 0.0;
    }
    // The hint, when it changes and every few seconds.
    let dir = match off {
        _ if feel < Feel::Faint || feel >= Feel::OnLine => "",
        Some(a) if a.abs() <= 35.0 => "ley.base.dir.ahead",
        Some(a) if a.abs() >= 145.0 => "ley.base.dir.behind",
        Some(a) if a > 0.0 => "ley.base.dir.right",
        Some(_) => "ley.base.dir.left",
        None => "",
    };
    let waystone = feel == Feel::AtNode && r.node.is_some_and(|n| n.node.waystone());
    let key = match (feel, waystone) {
        (Feel::AtNode, true) => "hint.base.ley.at_waystone",
        (f, _) => match f {
            Feel::Still => "hint.base.ley.still",
            Feel::Faint => "hint.base.ley.faint",
            Feel::Pull => "hint.base.ley.pull",
            Feel::Strong => "hint.base.ley.strong",
            Feel::OnLine => "hint.base.ley.on_line",
            Feel::AtNode => "hint.base.ley.at_node",
        },
    };
    let changed = s.feel != Some((feel, dir));
    if changed
        || s.hinted
            .is_none_or(|t| now.duration_since(t).as_secs_f32() >= 3.0)
    {
        s.feel = Some((feel, dir));
        s.hinted = Some(now);
        c.hint(key, &[Arg::Key(dir)])
            .id("base:ley_rod")
            .times(0.15, 3.2, 0.4);
    }
}

/// The faint motes flowing along every line within `shown` blocks.
fn motes(c: &Client, s: &mut Rod, seed: u64, me: [f64; 3], shown: f64, dt: f32) {
    let (x, z) = (me[0], me[2]);
    // Each line's stretch within reach: (start, end along it, the line).
    let mut parts: Vec<(f64, f64, ley::Line)> = Vec::new();
    let mut total = 0.0;
    for l in ley::lines_near(seed, x.floor() as i32, z.floor() as i32, shown.ceil() as i32) {
        let (_, d) = l.nearest(x, z);
        if d >= shown {
            continue;
        }
        // Where the circle of reach cuts the line: t along it (blocks from `from`).
        let [ux, uz] = l.dir();
        let (fx, fz) = (l.from[0] as f64 - x, l.from[1] as f64 - z);
        let b = fx * ux + fz * uz;
        let cc = fx * fx + fz * fz - shown * shown;
        let disc = b * b - cc;
        if disc <= 0.0 {
            continue;
        }
        let (t0, t1) = ((-b - disc.sqrt()).max(0.0), (-b + disc.sqrt()).min(l.length()));
        if t1 > t0 {
            total += t1 - t0;
            parts.push((t0, t1, l));
        }
    }
    if parts.is_empty() {
        return;
    }
    // As many born a second as the stretches' length asks, to the budget.
    let rate = (total as f32 * ley::MOTES_PER_BLOCK).min(ley::MOTES_A_SECOND);
    s.owed = (s.owed + rate * dt).min(ley::MOTES_A_SECOND);
    if s.ground.len() > 8192 {
        s.ground.clear();
    }
    while s.owed >= 1.0 {
        s.owed -= 1.0;
        // A stretch by its length, then a point along it.
        let mut pick = s.unit() * total;
        let &(t0, t1, l) = parts
            .iter()
            .find(|p| {
                let len = p.1 - p.0;
                if pick < len {
                    true
                } else {
                    pick -= len;
                    false
                }
            })
            .unwrap_or(&parts[0]);
        let t = t0 + (t1 - t0) * s.unit();
        let [ux, uz] = l.dir();
        let side = (s.unit() - 0.5) * 0.8;
        let (px, pz) = (l.from[0] as f64 + 0.5 + ux * t - uz * side, l.from[1] as f64 + 0.5 + uz * t + ux * side);
        // None by the eyes: the line under the player's feet put one in the camera's face (PC look
        // ley_look2), a cross filling a fifth of the screen.
        if (px - x).hypot(pz - z) < ley::MOTE_CLEAR {
            continue;
        }
        let Some(top) = ground_at(c, s, px.floor() as i32, pz.floor() as i32, me[1]) else { continue };
        let y = top as f64 + 1.0 + ley::MOTE_RISE.0 + s.unit() * (ley::MOTE_RISE.1 - ley::MOTE_RISE.0);
        let speed = ley::FLOW_SPEED as f64 * (0.7 + 0.6 * l.flow as f64);
        let v = [ux * speed, (s.unit() - 0.5) * 0.15, uz * speed];
        drop(c.spawn_particles(&MOTE, [px, y, pz]).velocity(v));
    }
}

/// The top solid block of column (`x`, `z`) near height `y`, remembered; None where it isn't
/// loaded or no ground is within reach.
fn ground_at(c: &Client, s: &mut Rod, x: i32, z: i32, y: f64) -> Option<i32> {
    if let Some(&h) = s.ground.get(&(x, z))
        && (h - y as i32).abs() < 40
    {
        return Some(h);
    }
    let from = y.floor() as i32 + 16;
    let top = (0..48).map(|k| from - k).find(|&yy| {
        c.block([x, yy, z])
            .is_some_and(|b| b != 0 && liquid_of(b).is_none() && state_flags(b) & abi::block_flags::SOLID != 0)
            || c.block([x, yy, z]).is_some_and(|b| liquid_of(b).is_some())
    })?;
    s.ground.insert((x, z), top);
    Some(top)
}
