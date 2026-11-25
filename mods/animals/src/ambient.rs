//! Ambient life (`animals_core::ambient` has the numbers and the why; `reports/sky-weather-life.md`,
//! change 7): birds of prey circling on thermals by day, a flock of gulls over the coast, an owl
//! crossing at night, fish jumping in lakes and rivers, most at dusk and dawn. Not entities: the
//! server only tells each player's client every few seconds what flies here now (one number on
//! the player, `ambient::Now::pack`, from the place's species, the hour, the season and the rain),
//! and the client draws them round its player.
//!
//! Each bird is a particle born every frame where its path is and moving as it moves, living a
//! few frames: the GPU turns a streaked kind's card along its motion (`particle.wgsl`), so one
//! sprite seen from below follows the bird's heading, climb and turns, and the particles' one
//! draw a blend mode draws them all (no draw calls of their own). Circling birds bank into their
//! turn (a sprite drawn banked, mirrored for the other way round). A leaping fish is one particle
//! thrown up and falling back, its nose along its arc, with a splash where it leaves the water and
//! where it comes down.
//!
//! Each makes its sound from where it is (`am::EAGLE_CRY_EVERY` and the rest): an eagle's cry
//! from one of the circling birds, a gull's call from one of the flock, an owl's hoot as it passes
//! nearest, a splash where a fish leaves the water and where it falls back. One sound started now
//! and then, a few a minute: nothing a frame.

use std::f64::consts::TAU;
use std::sync::Mutex;
use std::time::Instant;

use animals_core::ambient as am;
use weft_sdk::*;

/// Each sprite level, by `am::SPRITES`' order: eagle, vulture, gull, owl, fish.
static LEVEL: [Particle; 5] = [
    Particle::named("ambient_eagle"),
    Particle::named("ambient_vulture"),
    Particle::named("ambient_gull"),
    Particle::named("ambient_owl"),
    Particle::named("ambient_fish"),
];
/// The circling birds (eagle, vulture, gull) banked into a turn to their left, and to their right.
static BANK_LEFT: [Particle; 3] =
    [Particle::named("ambient_eagle_bank_l"), Particle::named("ambient_vulture_bank_l"), Particle::named("ambient_gull_bank_l")];
static BANK_RIGHT: [Particle; 3] =
    [Particle::named("ambient_eagle_bank_r"), Particle::named("ambient_vulture_bank_r"), Particle::named("ambient_gull_bank_r")];
static SPLASH: Particle = Particle::named("core:splash");
static WATER: Tag = Tag::named("water");
static CRY: Sound = Sound::named("ambient.eagle");
static SPLASHING: Sound = Sound::named("ambient.splash");
/// The land's calls (`mods/base/src/ambience.rs`): the flock and the owl are the gulls and owls
/// it calls for.
static GULL_CALL: Sound = Sound::named("base:ambient.call.gull");
static OWL_CALL: Sound = Sound::named("base:ambient.call.owl");

/// What flies here now (`am::Now::pack`), the server's.
static NOW: Component<u32> = Component::named("ambient_life");

/// A bird's particle lives this long, a few frames: born again each frame, a frame missed at 15
/// fps leaves no gap, and the copies overlap where it was a hair before.
const DRAWN_FOR: f32 = 0.09;
/// A turn's sharpest (radians a second) that still flies level, and the one fully banked.
const BANK_FROM: f64 = 0.12;

pub fn init(r: &mut Registry) {
    r.entity_component(&NOW, 0, component_flags::SYNC_OWNER);
    r.edit(&PLAYER).attach(&NOW);
    let bird = |r: &mut Registry, kind: &'static Particle, texture: &str, span: f32| {
        r.particle(kind)
            .textures(&[texture])
            .life(DRAWN_FOR, DRAWN_FOR)
            .size(span, span)
            .gravity(0.0)
            .drag(1.0)
            // A card along its motion, as wide as it is long (`size`): the sprite's head first.
            .streak(1e-4);
    };
    for (i, kind) in LEVEL.iter().enumerate().take(4) {
        bird(r, kind, am::SPRITES[i], am::SPAN[i]);
    }
    for i in 0..3 {
        bird(r, &BANK_LEFT[i], &format!("{}_bank_l", am::SPRITES[i]), am::SPAN[i]);
        bird(r, &BANK_RIGHT[i], &format!("{}_bank_r", am::SPRITES[i]), am::SPAN[i]);
    }
    let flight = 2.0 * am::FISH_LEAP / am::FISH_FALL;
    r.particle(&LEVEL[4])
        .textures(&[am::SPRITES[4]])
        .life(flight + 0.3, flight + 0.3)
        .size(am::SPAN[4], am::SPAN[4])
        .gravity(am::FISH_FALL)
        .drag(1.0)
        .streak(1e-4)
        .on_enter_liquid(When::any(), Then::die());
    r.sound(&CRY)
        .files(&["ambient/eagle1", "ambient/eagle2", "ambient/eagle3"])
        .category(&sound_categories::AMBIENT)
        .volume(0.55)
        .pitch(0.92, 1.08)
        .range(am::CRY_RANGE)
        .priority(20)
        .voices(2);
    r.sound(&SPLASHING)
        .files(&["ambient/splash1", "ambient/splash2", "ambient/splash3", "ambient/splash4"])
        .category(&sound_categories::AMBIENT)
        .volume(0.6)
        .pitch(0.85, 1.2)
        .range(am::SPLASH_RANGE)
        .priority(20)
        .voices(3);
    r.world_tick(tell);
    r.client_frame(frame);
    // What lives where you stand and what flies now; `force` sets what flies round you (any of
    // soar, gull, owl, fish, dusk, busy; the sprites the place's or an eagle's, a gull's, a great horned
    // owl's) until `off`, for looks in a place that has none.
    r.command("ambient", "[what:text]", |cmd| {
        let w = cmd.world();
        let player = cmd.player();
        let Some(st) = w.entity(player) else { return };
        let what = cmd.text("what").unwrap_or("").to_string();
        let mut words = what.split_whitespace();
        match words.next() {
            Some("off") => {
                FORCED.lock().unwrap().retain(|(p, _)| *p != player);
            }
            Some("force") => {
                let life = am::life_in(name_of(abi::entry_kind::BIOME, w.biome(st.pos.block().to_array())));
                let mut n = am::Now::default();
                for word in words {
                    match word {
                        "soar" => (n.soar, n.soarers) = (if life.soar != 0 { life.soar } else { 1 }, am::SOARERS as u8),
                        "gull" => (n.gull, n.gulls) = (3, am::GULLS as u8),
                        "owl" => n.owl = if life.owl != 0 { life.owl } else { 4 },
                        "fish" => (n.fish, n.fish_rate) = (5, n.fish_rate.max(10)),
                        "dusk" => n.fish_rate = (am::FISH_DUSK * 10.0) as u8,
                        // For looks: as many jumps as the word holds (6.3 times the day's).
                        "busy" => n.fish_rate = 63,
                        _ => {}
                    }
                }
                let mut forced = FORCED.lock().unwrap();
                forced.retain(|(p, _)| *p != player);
                forced.push((player, n.pack()));
            }
            _ => {}
        }
        let block = st.pos.block().to_array();
        let biome = name_of(abi::entry_kind::BIOME, w.biome(block));
        let on_you = w.component(Holder::entity(player), &NOW).unwrap_or(0);
        let forced = FORCED.lock().unwrap().iter().any(|(p, _)| *p == player);
        cmd.reply(
            "command.animals.ambient",
            &[
                Arg::Text(biome),
                Arg::Text(&format!("{:?}", am::life_in(biome))),
                Arg::Text(&format!("{:?}", am::Now::unpack(on_you))),
                Arg::Int(forced as i64),
            ],
        );
    });
}

/// Players whose word `/ambient force` set, and the word.
static FORCED: Mutex<Vec<(u64, u32)>> = Mutex::new(Vec::new());

/// Every `TELL_EVERY` ticks, what flies and jumps round each player now. Spectators are not in
/// `players()`: scripts that look at it play in creative.
fn tell(w: &World) {
    if !w.game_time().is_multiple_of(am::TELL_EVERY) {
        return;
    }
    let date = vitals::calendar::date(w.day_time());
    let hour = (date.fraction * 24.0 + 6.0) % 24.0;
    let swing = base_api::calendars::swing(base_api::SEASONS.on_dimension(w.dimension()).unwrap_or(0));
    let sunset = 12.0 + vitals::calendar::daylight_hours(date.day as f32 + date.fraction, swing) / 2.0;
    let winter = crate::breeding::season_bit(w) == Some(animals_core::biology::seasons::WINTER);
    for player in w.players() {
        let Some(st) = w.entity(player) else { continue };
        let block = st.pos.block().to_array();
        let life = am::life_in(name_of(abi::entry_kind::BIOME, w.biome(block)));
        let forced = FORCED
            .lock()
            .unwrap()
            .iter()
            .find(|(p, _)| *p == player)
            .map(|&(_, n)| n);
        let n = forced.unwrap_or_else(|| am::now(life, hour, sunset, w.raining_at(block), winter).pack());
        if let Some(v) = w.component_mut(player, &NOW) {
            *v = n;
        }
    }
}

/// A bird on a circle round a moving middle.
#[derive(Clone, Copy)]
struct Circler {
    /// Its share of the middle's circle: radius, height over the middle, seconds a lap, where it
    /// started (radians), +1 anticlockwise seen from above, -1 the other way.
    radius: f64,
    height: f64,
    lap: f64,
    phase: f64,
    turn: f64,
}

/// A middle that birds circle (a thermal, a flock), drifting.
#[derive(Clone, Copy)]
struct Middle {
    at: [f64; 3],
    drift: [f64; 2],
}

struct Owl {
    from: [f64; 3],
    vel: [f64; 3],
    born: Instant,
    secs: f64,
    /// When in its crossing (seconds) it hoots: nearest the player, and maybe once more.
    hoots: Vec<f64>,
}

struct Sky {
    seed: u64,
    start: Option<Instant>,
    last: Option<Instant>,
    /// What the server said last, for what to keep when it changes.
    told: am::Now,
    thermal: Option<Middle>,
    soarers: Vec<Circler>,
    flock: Option<Middle>,
    gulls: Vec<Circler>,
    owl: Option<Owl>,
    /// Splashes still to come: when, and where.
    splashes: Vec<(Instant, [f64; 3])>,
    /// Since the last log line: the sounds started (cries, gull calls, hoots, splashes).
    sounds: [u32; 4],
    /// Since the last log line: frames, spawns, fish jumps and the time spent (s).
    frames: u32,
    spawned: u32,
    jumps: u32,
    /// Searches for water that found none, and the last block one stopped at instead.
    dry: u32,
    last_ground: &'static str,
    spent: f64,
    logged: Option<Instant>,
}

static SKY: Mutex<Sky> = Mutex::new(Sky {
    seed: 0x2545_F491_4F6C_DD1D,
    start: None,
    last: None,
    told: am::Now { soar: 0, soarers: 0, gull: 0, gulls: 0, owl: 0, fish: 0, fish_rate: 0 },
    thermal: None,
    soarers: Vec::new(),
    flock: None,
    gulls: Vec::new(),
    owl: None,
    splashes: Vec::new(),
    sounds: [0; 4],
    frames: 0,
    spawned: 0,
    jumps: 0,
    dry: 0,
    last_ground: "",
    spent: 0.0,
    logged: None,
});

impl Sky {
    /// 0 to 1.
    fn unit(&mut self) -> f64 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        (self.seed >> 11) as f64 / (1u64 << 53) as f64
    }
    fn between(&mut self, (a, b): (f32, f32)) -> f64 {
        a as f64 + (b - a) as f64 * self.unit()
    }
    fn circler(&mut self, height: (f32, f32), radius: (f32, f32), lap: (f32, f32)) -> Circler {
        Circler {
            radius: self.between(radius),
            height: self.between(height),
            lap: self.between(lap),
            phase: self.unit() * TAU,
            turn: if self.unit() < 0.5 { 1.0 } else { -1.0 },
        }
    }
    fn middle(&mut self, me: [f64; 3], within: f64, drift: f32) -> Middle {
        let (a, d) = (self.unit() * TAU, within * self.unit().sqrt());
        let b = self.unit() * TAU;
        let speed = drift as f64 * (0.4 + 0.6 * self.unit());
        Middle { at: [me[0] + a.cos() * d, me[1], me[2] + a.sin() * d], drift: [b.cos() * speed, b.sin() * speed] }
    }
}

/// Where a circler is at `t` seconds round `m` and how it moves (blocks a second), and how hard it
/// turns (radians a second, signed as `turn`).
fn on_circle(c: &Circler, m: &Middle, t: f64) -> ([f64; 3], [f64; 3], f64) {
    let rate = c.turn * TAU / c.lap;
    // A circle that breathes in and out over three laps, and rises and sinks a little on it: no
    // two laps alike.
    let breathe = 1.0 + 0.18 * (t * TAU / (c.lap * 3.0) + c.phase).sin();
    let r = c.radius * breathe;
    let a = c.phase + rate * t;
    let rise = 1.5 * (t * TAU / (c.lap * 1.7) + c.phase * 2.0).sin();
    let at = [m.at[0] + a.cos() * r, m.at[1] + c.height + rise, m.at[2] + a.sin() * r];
    let rise_rate = 1.5 * TAU / (c.lap * 1.7) * (t * TAU / (c.lap * 1.7) + c.phase * 2.0).cos();
    let vel = [-a.sin() * r * rate + m.drift[0], rise_rate, a.cos() * r * rate + m.drift[1]];
    (at, vel, rate)
}

/// The kind a circling bird of sprite `sprite` (1-based) is drawn with while turning at `rate`.
/// In our axes (x east, z south, y up) a growing angle on the circle runs clockwise seen from
/// above: a + rate is a turn to the bird's right.
fn banked(sprite: u8, rate: f64) -> &'static Particle {
    let i = (sprite as usize).saturating_sub(1).min(4);
    if i >= 3 || rate.abs() < BANK_FROM {
        &LEVEL[i]
    } else if rate < 0.0 {
        &BANK_LEFT[i]
    } else {
        &BANK_RIGHT[i]
    }
}

fn frame(c: &Client) {
    let began = Instant::now();
    let view = c.view();
    let (Some(me), Some(word)) = (view.player_state(), view.component(&NOW)) else { return };
    let now_word = am::Now::unpack(word);
    let mut s = SKY.lock().unwrap();
    let start = *s.start.get_or_insert(began);
    let dt = s
        .last
        .map_or(0.0, |l| began.duration_since(l).as_secs_f64())
        .min(0.25);
    s.last = Some(began);
    let t = began.duration_since(start).as_secs_f64();
    let me = [me.pos.x, me.pos.y, me.pos.z];
    let keep = am::KEEP as f64;
    let far = |p: [f64; 3]| (p[0] - me[0]).hypot(p[2] - me[2]) > keep * 0.6;
    let mut spawned = 0u32;

    // Soarers: a thermal near the player, its birds high over it.
    if now_word.soar != s.told.soar || now_word.soarers == 0 {
        s.soarers.clear();
    }
    if now_word.soarers > 0 {
        let gone = s
            .thermal
            .is_none_or(|m| far(m.at) || (m.at[1] - me[1]).abs() > 24.0);
        if gone {
            s.thermal = Some(s.middle(me, 24.0, am::THERMAL_DRIFT));
        }
        while s.soarers.len() < now_word.soarers as usize {
            let b = s.circler(am::SOAR_HEIGHT, am::SOAR_RADIUS, am::SOAR_LAP);
            s.soarers.push(b);
        }
        s.soarers.truncate(now_word.soarers as usize);
        if let Some(m) = s.thermal.as_mut() {
            m.at[0] += m.drift[0] * dt;
            m.at[2] += m.drift[1] * dt;
            // The thermal keeps to the ground the player is on, a slow follow.
            m.at[1] += (me[1] - m.at[1]) * (dt * 0.1).min(1.0);
        }
        let m = s.thermal.unwrap();
        // Eagles (sprite 1) cry now and then, from one of them; vultures (2) are silent.
        let cry = now_word.soar == 1 && s.unit() < dt / am::EAGLE_CRY_EVERY as f64;
        let crier = (s.unit() * s.soarers.len() as f64) as usize;
        for (k, b) in s.soarers.clone().into_iter().enumerate() {
            let (at, vel, rate) = on_circle(&b, &m, t);
            drop(
                c.spawn_particles(banked(now_word.soar, rate), at)
                    .velocity(vel),
            );
            spawned += 1;
            if cry && k == crier {
                let _ = c.start_sound(&CRY, Some(at), 1.0, false, 0.0);
                s.sounds[0] += 1;
            }
        }
    } else {
        s.thermal = None;
    }

    // Gulls: a flock over the water near the shore, drifting along it and back.
    if now_word.gull != s.told.gull || now_word.gulls == 0 {
        s.gulls.clear();
    }
    if now_word.gulls > 0 {
        let reach = am::FLOCK_REACH as f64;
        if s.flock.is_none_or(|m| far(m.at)) {
            let mut m = s.middle(me, 12.0, am::FLOCK_DRIFT);
            if let Some(w) = water_near(c, &mut s, me, reach) {
                m.at = [w[0], me[1].max(w[1]), w[2]];
            }
            s.flock = Some(m);
        }
        while s.gulls.len() < now_word.gulls as usize {
            let b = s.circler(am::GULL_HEIGHT, am::GULL_RADIUS, am::GULL_LAP);
            s.gulls.push(b);
        }
        s.gulls.truncate(now_word.gulls as usize);
        if let Some(m) = s.flock.as_mut() {
            m.at[0] += m.drift[0] * dt;
            m.at[2] += m.drift[1] * dt;
            // Past its reach it turns back along the shore.
            let (dx, dz) = (m.at[0] - me[0], m.at[2] - me[2]);
            if dx.hypot(dz) > reach && dx * m.drift[0] + dz * m.drift[1] > 0.0 {
                m.drift = [-m.drift[0], -m.drift[1]];
            }
            m.at[1] += (me[1] - m.at[1]) * (dt * 0.1).min(1.0);
        }
        let m = s.flock.unwrap();
        let call = s.unit() < dt / am::GULL_CALL_EVERY as f64;
        let caller = (s.unit() * s.gulls.len() as f64) as usize;
        for (k, b) in s.gulls.clone().into_iter().enumerate() {
            let (at, vel, rate) = on_circle(&b, &m, t);
            drop(
                c.spawn_particles(banked(now_word.gull, rate), at)
                    .velocity(vel),
            );
            spawned += 1;
            if call && k == caller {
                let _ = c.start_sound(&GULL_CALL, Some(at), 1.0, false, 0.0);
                s.sounds[1] += 1;
            }
        }
    } else {
        s.flock = None;
    }

    // An owl crossing at night: now and then, low over the player and on.
    if now_word.owl == 0 {
        s.owl = None;
    } else if s.owl.is_none() && (s.told.owl == 0 || s.unit() < dt / am::OWL_EVERY as f64) {
        // The first as they come out at dusk (or the player comes into their wood), then now
        // and then.
        let a = s.unit() * TAU;
        let (dir, side) = ([a.cos(), a.sin()], (s.unit() - 0.5) * 16.0);
        let from = am::OWL_FROM as f64;
        let height = s.between(am::OWL_HEIGHT);
        let speed = am::OWL_SPEED as f64;
        let secs = 2.0 * from / speed;
        let mut hoots = vec![secs / 2.0];
        if s.unit() < am::OWL_SECOND_HOOT as f64 {
            hoots.push(secs * (0.2 + 0.6 * s.unit()));
        }
        s.owl = Some(Owl {
            from: [me[0] - dir[0] * from - dir[1] * side, me[1] + height, me[2] - dir[1] * from + dir[0] * side],
            vel: [dir[0] * speed, 0.0, dir[1] * speed],
            born: began,
            secs,
            hoots,
        });
    }
    if let Some(o) = s.owl.as_mut() {
        let age = began.duration_since(o.born).as_secs_f64();
        if age > o.secs {
            s.owl = None;
        } else {
            // Its slow beats lift and drop it a little.
            let bob = 0.4 * (age * 2.2).sin();
            let at = [o.from[0] + o.vel[0] * age, o.from[1] + bob, o.from[2] + o.vel[2] * age];
            let vel = [o.vel[0], 0.4 * 2.2 * (age * 2.2).cos(), o.vel[2]];
            drop(c.spawn_particles(&LEVEL[3], at).velocity(vel));
            spawned += 1;
            let due = o.hoots.len();
            o.hoots.retain(|&h| h > age);
            if o.hoots.len() < due {
                let _ = c.start_sound(&OWL_CALL, Some(at), 1.0, false, 0.0);
                s.sounds[2] += 1;
            }
        }
    }

    // Fish: a jump somewhere on the water near the player now and then, more at dusk and dawn.
    if now_word.fish != 0 && now_word.fish_rate > 0 {
        let rate = now_word.fish_rate as f64 / 10.0 / am::FISH_EVERY as f64;
        if s.unit() < dt * rate
            && let Some(w) = water_near(c, &mut s, me, am::FISH_REACH as f64)
        {
            let a = s.unit() * TAU;
            let across = am::FISH_ACROSS as f64 * (0.6 + 0.4 * s.unit());
            let leap = am::FISH_LEAP as f64 * (0.75 + 0.25 * s.unit());
            let at = [w[0], w[1] + 0.05, w[2]];
            drop(
                c.spawn_particles(&LEVEL[4], at)
                    .velocity([a.cos() * across, leap, a.sin() * across]),
            );
            drop(
                c.spawn_particles(&SPLASH, at)
                    .count(6)
                    .spread(0.15)
                    .speed(1.2),
            );
            let _ = c.start_sound(&SPLASHING, Some(at), am::SPLASH_OUT, false, 0.0);
            s.sounds[3] += 1;
            let flight = 2.0 * leap / am::FISH_FALL as f64;
            let down = [at[0] + a.cos() * across * flight, at[1], at[2] + a.sin() * across * flight];
            s.splashes
                .push((began + std::time::Duration::from_secs_f64(flight), down));
            s.jumps += 1;
            spawned += 2;
        }
    }
    let due: Vec<[f64; 3]> = s
        .splashes
        .iter()
        .filter(|(when, _)| *when <= began)
        .map(|&(_, at)| at)
        .collect();
    s.splashes.retain(|(when, _)| *when > began);
    for at in due {
        drop(
            c.spawn_particles(&SPLASH, at)
                .count(8)
                .spread(0.2)
                .speed(1.6),
        );
        let _ = c.start_sound(&SPLASHING, Some(at), 1.0, false, 0.0);
        s.sounds[3] += 1;
        spawned += 1;
    }

    s.told = now_word;
    s.frames += 1;
    s.spawned += spawned;
    s.spent += began.elapsed().as_secs_f64();
    // A line for the logs every 10 s: what flies, and what it costs a frame.
    if s.logged
        .is_none_or(|l| began.duration_since(l).as_secs_f32() >= 10.0)
    {
        if s.frames > 0 && s.logged.is_some() {
            weft_sdk::info!(
                "ambient: {:?}; {} soarers, {} gulls, owl {}, {} fish jumps ({} searches found no water, last on {}); sounds: {} cries, {} gull calls, {} hoots, {} splashes; {:.1} spawns and {:.4} ms a frame",
                now_word,
                s.soarers.len(),
                s.gulls.len(),
                s.owl.is_some(),
                s.jumps,
                s.dry,
                s.last_ground,
                s.sounds[0],
                s.sounds[1],
                s.sounds[2],
                s.sounds[3],
                s.spawned as f64 / s.frames as f64,
                s.spent * 1000.0 / s.frames as f64
            );
        }
        s.logged = Some(began);
        (s.frames, s.spawned, s.jumps, s.dry, s.spent, s.sounds) = (0, 0, 0, 0, 0.0, [0; 4]);
    }
}

/// The top of open water near `me` within `reach`: a few columns tried, each read down from a
/// little over the player to the first block that isn't air. None: no water found this time.
fn water_near(c: &Client, s: &mut Sky, me: [f64; 3], reach: f64) -> Option<[f64; 3]> {
    for _ in 0..am::FISH_TRIES {
        let (a, d) = (s.unit() * TAU, reach * s.unit().sqrt());
        let (x, z) = ((me[0] + a.cos() * d).floor() as i32, (me[2] + a.sin() * d).floor() as i32);
        let top = me[1].floor() as i32 + 6;
        for y in (top - 20..=top).rev() {
            match c.block([x, y, z]) {
                Some(0) => continue,
                Some(state) if is_water(state) => {
                    return Some([x as f64 + 0.5, y as f64 + 1.0, z as f64 + 0.5]);
                }
                Some(state) => {
                    s.last_ground = block_name(state);
                    break;
                }
                None => {
                    s.last_ground = "(not loaded)";
                    break;
                }
            }
        }
    }
    s.dry += 1;
    None
}

/// Water, by its liquid (a liquid's states with their levels all count) or its tag.
fn is_water(state: u32) -> bool {
    WATER.on_block(state) || (liquid_of(state).is_some() && block_name(state).contains("water"))
}
