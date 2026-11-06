//! Wind life (`vitals::wind_life` has the numbers and the why): leaves, petals, pollen, spindrift,
//! blown sand and drips, put about round the player by the client and carried on the wind by the
//! GPU (`particle_sim.wgsl`: the kinds below are `carried`).
//!
//! The client looks at random blocks round the player each frame, as vanilla's display ticks do: a
//! leaf block with open air under it may drop a leaf, more as a gust passes (then off its downwind
//! side), more in autumn and in rain; a tree in blossom and a flower in bloom drop petals, and a
//! flower gives off pollen on dry days; snow and sand lying in the open blow in a strong wind; leaf
//! blocks drip in and after rain. Walking through flowers in bloom puffs petals up. The server
//! tells each player's client the season's leaf fall and how thick a dust storm is where they stand
//! (the client knows neither the biome nor its climate).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use vitals::{calendar, wind_life as wl};
use weft_sdk::*;

use crate::wind_life_colours as colours;

/// Leaves of six shapes (Art's, game-art's effects guide, 11-06); a leaf takes its tree's colour
/// and its season's tint.
static LEAVES: [Particle; 6] = [
    Particle::named("leaf_0"),
    Particle::named("leaf_1"),
    Particle::named("leaf_2"),
    Particle::named("leaf_3"),
    Particle::named("leaf_4"),
    Particle::named("leaf_5"),
];
static PETALS: [Particle; 4] =
    [Particle::named("petal_0"), Particle::named("petal_1"), Particle::named("petal_2"), Particle::named("petal_3")];
static POLLEN: Particle = Particle::named("pollen");
/// Snow off the block it blows from (block dust); sand grains (`sand_grain`, in the sand's shades).
static SPINDRIFT: Particle = Particle::named("spindrift");
static SAND_GRAIN: Particle = Particle::named("sand_grain");
/// A dust storm's streak of sand, drawn along its flight.
static SAND_STREAK: Particle = Particle::named("sand_streak");
static DRIP: Particle = Particle::named("core:drip");

/// The season's leaf fall where the player stands (`wl::fall`), the server's.
static FALL: Component<f32> = Component::named("wind_life_fall");
/// How thick a dust storm is where the player stands, 0..1 (`vitals::air::dust`), the server's.
static DUST: Component<f32> = Component::named("wind_life_dust");

static LEAVES_TAG: Tag = Tag::named("leaves");
static SAND_TAG: Tag = Tag::named("sand");

pub fn register(r: &mut Registry) {
    r.entity_component(&FALL, 1.0, component_flags::SYNC_OWNER);
    r.entity_component(&DUST, 0.0, component_flags::SYNC_OWNER);
    r.attach(&PLAYER, &FALL);
    r.attach(&PLAYER, &DUST);
    for (i, kind) in LEAVES.iter().enumerate() {
        r.particle(kind)
            .textures(&[&format!("leaf_{i}")])
            .life(wl::LEAF_LIFE.0, wl::LEAF_LIFE.1)
            .size(wl::LEAF_SIZE.0, wl::LEAF_SIZE.1)
            .carried(wl::LEAF_AIR, wl::LEAF_SINK)
            .flutter(1.0)
            .lifts_at(wl::LEAF_LIFT)
            .floats()
            .block_tinted()
            // Onto another tree's crown it is gone (out of sight, and it would hold its slot for
            // minutes); onto anything else it lies as litter.
            .on_land(When::tag("leaves"), Then::die())
            .on_land(When::any(), Then::stop());
    }
    for (i, kind) in PETALS.iter().enumerate() {
        r.particle(kind)
            .textures(&[&format!("petal_{i}")])
            .life(wl::PETAL_LIFE.0, wl::PETAL_LIFE.1)
            .size(wl::PETAL_SIZE.0, wl::PETAL_SIZE.1)
            .carried(wl::PETAL_AIR, wl::PETAL_SINK)
            .flutter(0.8)
            .lifts_at(wl::PETAL_LIFT)
            .floats()
            .on_land(When::tag("leaves"), Then::die())
            .on_land(When::any(), Then::stop());
    }
    r.particle(&POLLEN)
        .textures(&["pollen"])
        .life(wl::POLLEN_LIFE.0, wl::POLLEN_LIFE.1)
        .size(wl::POLLEN_SIZE.0, wl::POLLEN_SIZE.1)
        .carried(wl::POLLEN_AIR, wl::POLLEN_SINK)
        .on_land(When::any(), Then::die());
    r.particle(&SPINDRIFT)
        .life(wl::SPINDRIFT_LIFE.0, wl::SPINDRIFT_LIFE.1)
        .size(wl::SPINDRIFT_SIZE.0, wl::SPINDRIFT_SIZE.1)
        .carried(wl::SPINDRIFT_AIR, wl::SPINDRIFT_SINK)
        // Snow streaming low over the ground: drawn along its flight (specks alone barely showed,
        // wind_look1c).
        .streak(wl::SPINDRIFT_STREAK)
        .on_land(When::any(), Then::die());
    // A rounded grain of its own, coloured from the sand's shades (the sand block's dust drew hard
    // squares, pt4_dust).
    r.particle(&SAND_GRAIN)
        .textures(&["sand_grain"])
        .life(wl::SAND_LIFE.0, wl::SAND_LIFE.1)
        .size(wl::SAND_SIZE.0, wl::SAND_SIZE.1)
        .carried(wl::SAND_AIR, wl::SAND_SINK)
        .on_land(When::any(), Then::die());
    r.particle(&SAND_STREAK)
        .textures(&["sand_grain"])
        .life(wl::STREAK_LIFE.0, wl::STREAK_LIFE.1)
        .size(wl::STREAK_SIZE.0, wl::STREAK_SIZE.1)
        .carried(wl::SAND_AIR, wl::SAND_SINK * 0.3)
        .streak(wl::STREAK_SECS)
        .on_land(When::any(), Then::die());
    r.world_tick(tell);
    r.client_frame(frame);
    // One oak leaf a little ahead of you and above, to watch it fall (the frame strip; tuning).
    r.command("windleaf", "", |cmd| {
        let w = cmd.world();
        let Some(st) = w.entity(cmd.player()) else { return };
        let (sin, cos) = (st.yaw as f64).sin_cos();
        let at = [st.pos.x - sin * 2.5, st.pos.y + 3.2, st.pos.z - cos * 2.5];
        let oak = crate::blocks::OAK_LEAVES.id();
        drop(w.spawn_particles(&LEAVES[0], at).color(0xAEAEAE).block(oak));
        cmd.reply("command.base.windleaf", &[]);
    });
}

/// Every `REFRESH_SECS`, each player's season's leaf fall and dust storm.
fn tell(w: &World) {
    if !w
        .game_time()
        .is_multiple_of((wl::REFRESH_SECS * 20.0) as u64)
    {
        return;
    }
    let four = base_api::SEASONS.on_dimension(w.dimension()).unwrap_or(0) == base_api::calendars::FOUR;
    let date = calendar::date(w.day_time());
    for player in w.players() {
        let Some(st) = w.entity(player) else { continue };
        let block = st.pos.block().to_array();
        let fall = if four {
            base_api::CLIMATE
                .on_biome(w.biome(block))
                .map_or(1.0, |c| wl::fall_at(date.day as f32 + date.fraction, c.seasons))
        } else {
            1.0
        };
        let air = w.air_at(block);
        let dust = if air.flags & abi::air_flags::DUST != 0 { (air.fog / 0.95).clamp(0.0, 1.0) } else { 0.0 };
        if let Some(v) = w.component_mut(player, &FALL) {
            *v = fall;
        }
        if let Some(v) = w.component_mut(player, &DUST) {
            *v = dust;
        }
    }
}

/// What a block gives the wind, worked out once per state.
#[derive(Clone, Copy)]
enum Source {
    Nothing,
    /// Leaves: their colour, and whether they are needles or scales (fewer fall).
    Leaves {
        colour: u32,
        needles: bool,
    },
    /// A tree in blossom: its petals' colour.
    Blossom(u32),
    /// A flower in bloom: its petals' colour.
    Flower(u32),
    Snow,
    Sand,
}

fn source(state: u32) -> Source {
    let full = block_name(state);
    let name = full.split('[').next().unwrap_or(full);
    let bloom = state_prop(state, "bloom") == Some("1");
    if LEAVES_TAG.on_block(state) {
        if bloom && let Some(&(_, c)) = colours::BLOSSOMS.iter().find(|(n, _)| *n == name) {
            return Source::Blossom(c);
        }
        let colour = colours::LEAVES
            .iter()
            .find(|(n, _)| *n == name)
            .map_or(0xAEAEAE, |&(_, c)| c);
        let needles = vitals::wild::turn(name) == vitals::wild::Turn::Evergreen;
        return Source::Leaves { colour, needles };
    }
    if bloom && let Some(&(_, c)) = colours::FLOWERS.iter().find(|(n, _)| *n == name) {
        return Source::Flower(c);
    }
    if name.ends_with("snow_layer") || name == "base:snow" || name == "base:snowy_grass" {
        return Source::Snow;
    }
    if SAND_TAG.on_block(state) {
        return Source::Sand;
    }
    Source::Nothing
}

struct Life {
    last: Option<Instant>,
    start: Instant,
    seed: u64,
    /// Looks owed from the last frame (a fraction).
    owed: f32,
    sources: HashMap<u32, Source>,
    /// Seconds since it last rained here (drips go on a while after).
    dry_for: f32,
    /// Spawned since the last log, by kind (leaves, petals, pollen, snow, sand, drips), and when.
    sent: [u32; 6],
    logged: Option<Instant>,
}

static LIFE: Mutex<Option<Life>> = Mutex::new(None);

impl Life {
    /// 0 to 1.
    fn unit(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        (self.seed >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// The gust at `p` (about 1 its mean, 0.55 to 1.65), as the GPU's: bands of stronger wind sweeping
/// across the land the way it blows (`particle_sim.wgsl`'s `gust`; its own clock).
fn gust(p: [f64; 3], wind: [f32; 2], t: f32) -> f32 {
    let speed = wind[0].hypot(wind[1]);
    let dir = if speed > 0.01 { [wind[0] / speed, wind[1] / speed] } else { [1.0, 0.0] };
    let (x, z) = (p[0].rem_euclid(4096.0) as f32, p[2].rem_euclid(4096.0) as f32);
    let along = x * dir[0] + z * dir[1] - t * speed.max(1.0) * 0.9;
    let across = -x * dir[1] + z * dir[0];
    let band = noise3([along / 14.0, t * 0.05, across / 30.0]);
    0.55 + 1.1 * smooth(0.35, 0.85, band)
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn hash3(i: [i32; 3]) -> f32 {
    let mut h = (i[0] as u32)
        .wrapping_mul(374761393)
        .wrapping_add((i[1] as u32).wrapping_mul(668265263))
        .wrapping_add((i[2] as u32).wrapping_mul(2246822519));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^= h >> 16;
    (h & 0xffffff) as f32 / 16777215.0
}

fn noise3(p: [f32; 3]) -> f32 {
    let i = p.map(|v| v.floor() as i32);
    let f = [p[0] - i[0] as f32, p[1] - i[1] as f32, p[2] - i[2] as f32];
    let u = f.map(|v| v * v * (3.0 - 2.0 * v));
    let at = |dx, dy, dz| hash3([i[0] + dx, i[1] + dy, i[2] + dz]);
    let mix = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let a = mix(mix(at(0, 0, 0), at(1, 0, 0), u[0]), mix(at(0, 1, 0), at(1, 1, 0), u[0]), u[1]);
    let b = mix(mix(at(0, 0, 1), at(1, 0, 1), u[0]), mix(at(0, 1, 1), at(1, 1, 1), u[0]), u[1]);
    mix(a, b, u[2])
}

fn frame(c: &Client) {
    let density = wl::DENSITY[crate::video::FX_WIND_LIFE.get().clamp(0, 3) as usize];
    let st = c.state();
    let mut guard = LIFE.lock().unwrap();
    let life = guard.get_or_insert_with(|| Life {
        last: None,
        start: Instant::now(),
        seed: 0x2545_F491_4F6C_DD1D,
        owed: 0.0,
        sources: HashMap::new(),
        dry_for: 1e9,
        sent: [0; 6],
        logged: None,
    });
    let now = Instant::now();
    let dt = life
        .last
        .map_or(0.0, |l| now.duration_since(l).as_secs_f32())
        .min(0.25);
    life.last = Some(now);
    if density <= 0.0 || st.player == 0 {
        return;
    }
    life.dry_for = if st.rain > 0.2 { 0.0 } else { life.dry_for + dt };
    let view = c.view();
    let fall = view.component(&FALL).unwrap_or(1.0);
    let dust = view.component(&DUST).unwrap_or(0.0);
    // A dust storm needs its wind (`air::DUST`): the server's share goes stale round a spectator
    // (hidden from the world's players), and grains blew in calm air by the lake.
    let dust = dust * wl::blows(st.wind[0].hypot(st.wind[1]), (vitals::air::DUST.0 - 2.0, vitals::air::DUST.0));
    let t = now.duration_since(life.start).as_secs_f32();
    let wind = st.wind;
    let speed = wind[0].hypot(wind[1]);
    let dir = if speed > 0.01 { [wind[0] / speed, wind[1] / speed] } else { [0.0, 0.0] };
    let date = calendar::date(view.day_time());
    let hour = (date.fraction * 24.0 + 6.0) % 24.0;
    let pollen_time = (wl::POLLEN_HOURS.0..wl::POLLEN_HOURS.1).contains(&hour) && st.rain < 0.05;
    let drips = life.dry_for < wl::DRIP_AFTER;
    let me = [st.pos.x, st.pos.y, st.pos.z];
    let (reach, down) = (wl::REACH, wl::REACH_DOWN);
    let side = 2 * reach + 1;
    let volume = (side * side * (reach + down + 1)) as f32;
    // A block's rate (a second) is its chance at a look times this.
    let per_look = volume / wl::LOOKS_PER_SEC;
    life.owed += wl::LOOKS_PER_SEC * dt;
    let looks = life.owed as u32;
    life.owed -= looks as f32;
    let calm = wl::weather(speed, st.rain) * density;
    // Over the setting's most alive, none more this frame (`ALIVE_MAX`).
    let setting = crate::video::FX_WIND_LIFE.get().clamp(0, 3) as usize;
    if st.particles >= wl::ALIVE_MAX[setting] {
        return;
    }
    let base = [me[0].floor() as i32, me[1].floor() as i32, me[2].floor() as i32];
    for _ in 0..looks {
        let p = [
            base[0] + (life.unit() * side as f32) as i32 - reach,
            base[1] + (life.unit() * (reach + down + 1) as f32) as i32 - down,
            base[2] + (life.unit() * side as f32) as i32 - reach,
        ];
        let Some(state) = c.block(p) else { continue };
        if state == 0 {
            continue;
        }
        let src = *life.sources.entry(state).or_insert_with(|| source(state));
        let at = |dx: f64, dy: f64, dz: f64| [p[0] as f64 + dx, p[1] as f64 + dy, p[2] as f64 + dz];
        match src {
            Source::Nothing => {}
            Source::Leaves { colour, .. } | Source::Blossom(colour) => {
                let blossom = matches!(src, Source::Blossom(_));
                let rate = match src {
                    Source::Blossom(_) => wl::BLOSSOM_RATE,
                    Source::Leaves { needles: true, .. } => wl::NEEDLE_RATE,
                    _ => wl::LEAF_RATE * fall,
                } * calm;
                let open_below = c.block([p[0], p[1] - 1, p[2]]) == Some(0);
                let burst = wl::burst(gust(at(0.5, 0.5, 0.5), wind, t)) - 1.0;
                let mut from = Vec::new();
                // In calm air they come from under the crown, more as a gust passes ...
                if open_below {
                    for _ in 0..wl::draw(rate * (1.0 + burst) * per_look, life.unit()) {
                        from.push(at(life.unit() as f64, -0.25, life.unit() as f64));
                    }
                }
                // ... and a gust bursts them off its downwind side.
                let d = [(dir[0].abs() > 0.38) as i32 * dir[0].signum() as i32, (dir[1].abs() > 0.38) as i32 * dir[1].signum() as i32];
                if burst > 0.01 && d != [0, 0] && c.block([p[0] + d[0], p[1], p[2] + d[1]]) == Some(0) {
                    for _ in 0..wl::draw(rate * burst * per_look, life.unit()) {
                        let mut off = |d: i32| if d == 0 { life.unit() as f64 } else { 0.5 + d as f64 * 0.65 };
                        let (x, z) = (off(d[0]), off(d[1]));
                        from.push(at(x, life.unit() as f64 * 0.8, z));
                    }
                }
                for f in from {
                    let v = [wind[0] as f64 * 0.3, 0.0, wind[1] as f64 * 0.3];
                    if blossom {
                        let k = &PETALS[(life.unit() * 4.0) as usize % 4];
                        drop(c.spawn_particles(k, f).velocity(v).color(colour));
                        life.sent[1] += 1;
                    } else {
                        let k = &LEAVES[(life.unit() * 6.0) as usize % 6];
                        drop(
                            c.spawn_particles(k, f)
                                .velocity(v)
                                .color(colour)
                                .block(state),
                        );
                        life.sent[0] += 1;
                    }
                }
                // Drips off the leaves in rain and a while after.
                if drips && open_below && wl::draw(wl::DRIP_RATE * per_look * density, life.unit()) > 0 {
                    drop(
                        c.spawn_particles(&DRIP, at(0.5, -0.05, 0.5))
                            .spread_box([0.4, 0.0, 0.4]),
                    );
                    life.sent[5] += 1;
                }
            }
            Source::Flower(colour) => {
                let g = gust(at(0.5, 0.5, 0.5), wind, t);
                let petals = wl::PETAL_RATE * calm * (1.0 + (wl::PETAL_GUST - 1.0) * smooth(wl::GUST_FROM.0, wl::GUST_FROM.1, g));
                for _ in 0..wl::draw(petals * per_look, life.unit()) {
                    let k = &PETALS[(life.unit() * 4.0) as usize % 4];
                    let v = [wind[0] as f64 * 0.2, 0.25, wind[1] as f64 * 0.2];
                    drop(
                        c.spawn_particles(k, at(0.5, 0.6, 0.5))
                            .spread(0.2)
                            .velocity(v)
                            .color(colour),
                    );
                    life.sent[1] += 1;
                }
                if pollen_time && wl::draw(wl::POLLEN_RATE * density * per_look, life.unit()) > 0 {
                    drop(
                        c.spawn_particles(&POLLEN, at(0.5, 0.7, 0.5))
                            .spread(0.3)
                            .velocity([0.0, 0.15, 0.0]),
                    );
                    life.sent[2] += 1;
                }
            }
            Source::Snow | Source::Sand => {
                let snow = matches!(src, Source::Snow);
                let range = if snow { wl::SPINDRIFT } else { wl::SAND };
                let blows = wl::blows(speed * gust(at(0.5, 1.0, 0.5), wind, t), range);
                let storm = if snow { 0.0 } else { dust };
                if blows <= 0.0 && storm <= 0.0 {
                    continue;
                }
                // Only off a surface open to the sky: nothing solid over it (air, or grass and
                // flowers standing in it: on the dunes and the taiga most sand and snow has some).
                let open = c
                    .block([p[0], p[1] + 1, p[2]])
                    .is_some_and(|b| b == 0 || state_flags(b) & abi::block_flags::SOLID == 0 && liquid_of(b).is_none());
                if !open {
                    continue;
                }
                let top = c.light([p[0], p[1] + 1, p[2]]).map_or(0, |l| l & 0xFF);
                if top < 12 {
                    continue;
                }
                let rate = if snow { wl::SPINDRIFT_RATE } else { wl::SAND_RATE } * blows.max(storm);
                let n = wl::draw(rate * density * per_look, life.unit());
                if n > 0 {
                    let (kind, slot) = if snow { (&SPINDRIFT, 3) } else { (&SAND_GRAIN, 4) };
                    let height = if snow { snow_top(state) } else { 1.0 };
                    // Off along the ground with the wind, a little up.
                    let v = [wind[0] as f64 * 0.8, 0.3 + life.unit() as f64 * 0.6, wind[1] as f64 * 0.8];
                    let grains = c
                        .spawn_particles(kind, at(0.5, height + 0.05, 0.5))
                        .spread_box([0.5, 0.05, 0.5])
                        .velocity(v)
                        .speed(0.4)
                        .count(n);
                    drop(if snow { grains.block(state) } else { grains.color(sand_shade(life)) });
                    life.sent[slot] += n;
                }
            }
        }
    }
    // A dust storm fills the clear air round the player with sand, whatever the ground (the fog
    // hides farther), and draws streaks along the wind farther out.
    if dust > 0.0 {
        let v = [wind[0] as f64, 0.0, wind[1] as f64];
        let clear = wl::dust_clear(speed) as f64;
        let n = wl::draw(wl::DUST_STORM * dust * density * dt, life.unit());
        // Centred a little upwind, so the wind carries them through; in a few bursts, each its own
        // shade and layer (most skim the ground, the rest fill the air to 4 blocks).
        let up = [me[0] - dir[0] as f64 * clear * 0.4, me[2] - dir[1] as f64 * clear * 0.4];
        let bursts = n.min(4);
        for i in 0..bursts {
            let count = n / bursts + u32::from(i < n % bursts);
            let low = life.unit() < wl::DUST_LOW;
            let (y, half) = if low { (0.65, 0.55) } else { (2.6, 1.5) };
            drop(
                c.spawn_particles(&SAND_GRAIN, [up[0], me[1] + y, up[1]])
                    .spread_box([clear, half, clear])
                    .velocity(v)
                    .speed(1.0)
                    .count(count)
                    .color(sand_shade(life)),
            );
        }
        life.sent[4] += n;
        for _ in 0..wl::draw(wl::DUST_STREAKS * dust * density * dt, life.unit()) {
            let (a, r) =
                (life.unit() as f64 * std::f64::consts::TAU, wl::STREAK_REACH.0 + (wl::STREAK_REACH.1 - wl::STREAK_REACH.0) * life.unit());
            // Most along the ground, as the grains.
            let high = if life.unit() < wl::DUST_LOW { 1.2 } else { 4.0 };
            let at = [me[0] + a.cos() * r as f64, me[1] + 0.2 + life.unit() as f64 * high, me[2] + a.sin() * r as f64];
            drop(
                c.spawn_particles(&SAND_STREAK, at)
                    .velocity(v)
                    .speed(1.5)
                    .color(sand_shade(life)),
            );
            life.sent[4] += 1;
        }
    }
    stir(c, life, &st, density);
    if life
        .logged
        .is_none_or(|l| now.duration_since(l).as_secs_f32() >= 5.0)
    {
        weft_sdk::debug!(
            "wind life: wind {speed:.1} m/s, rain {:.2}, fall {fall:.2}, dust {dust:.2}, density {density}: sent leaves {}, petals {}, pollen {}, snow {}, sand {}, drips {} in 5 s; {} alive",
            st.rain,
            life.sent[0],
            life.sent[1],
            life.sent[2],
            life.sent[3],
            life.sent[4],
            life.sent[5],
            st.particles
        );
        life.sent = [0; 6];
        life.logged = Some(now);
    }
}

/// How high a snow block's top is (a layer's eighths; a full block 1).
fn snow_top(state: u32) -> f64 {
    state_prop(state, "layers")
        .and_then(|l| l.parse::<f64>().ok())
        .map_or(1.0, |l| l / 8.0)
}

/// One of blown sand's shades, at random (`wind_life_colours::SAND`, `vitals::wind_life::grain`).
fn sand_shade(life: &mut Life) -> u32 {
    let shades = &crate::wind_life_colours::SAND;
    wl::grain(shades[((life.unit() * shades.len() as f32) as usize).min(shades.len() - 1)])
}

/// Walking through flowers in bloom puffs their petals up.
fn stir(c: &Client, life: &mut Life, st: &abi::ClientState, density: f32) {
    // The player's velocity is in blocks a tick.
    let vel = [st.vel.x * 20.0, st.vel.z * 20.0];
    let moving = vel[0].hypot(vel[1]) as f32;
    if moving < 1.0 {
        return;
    }
    let feet = [st.pos.x.floor() as i32, st.pos.y.floor() as i32, st.pos.z.floor() as i32];
    let Some(state) = c.block(feet) else { return };
    let Source::Flower(colour) = *life.sources.entry(state).or_insert_with(|| source(state)) else { return };
    let dt = st.dt.min(0.25);
    if life.unit() >= wl::STIR_CHANCE * (moving / 4.3) * dt * density.min(2.0) {
        return;
    }
    let (lo, hi) = wl::STIR_PETALS;
    let n = lo + (life.unit() * (hi - lo + 1) as f32) as u32;
    let v = [vel[0] * 0.5, 0.8 + life.unit() as f64 * 0.6, vel[1] * 0.5];
    for _ in 0..n {
        let k = &PETALS[(life.unit() * 4.0) as usize % 4];
        drop(
            c.spawn_particles(k, [st.pos.x, st.pos.y + 0.3, st.pos.z])
                .spread(0.3)
                .velocity(v)
                .speed(0.6)
                .color(colour),
        );
    }
    life.sent[1] += n;
}
