//! What the weather sounds like where the player stands (`notes/SYSTEMS.md`, weather sound): five
//! looped beds (light rain, heavy rain, rain on a roof, wind in the open, wind in the trees) whose
//! volumes ease toward the levels `vitals::weather_sound` gives for the weather now and the cover
//! over the player, and thunder after each lightning flash, as late as its distance makes it.
//!
//! The weather itself comes from the engine (`ClientState::rain`, `thunder`, `wind`, `snowing`,
//! `lightnings`), which has it from `weather.rs`: rain, storms and wind by biome and season. The
//! cover is sampled a few times a second from the sky light and the blocks around the head.

use std::sync::Mutex;

use vitals::weather_sound::{self as ws, Cover, Levels};
use weft_sdk::*;

static RAIN: Sound = Sound::named("weather.rain");
static RAIN_HEAVY: Sound = Sound::named("weather.rain_heavy");
static RAIN_ROOF: Sound = Sound::named("weather.rain_roof");
static WIND: Sound = Sound::named("weather.wind");
static WIND_TREES: Sound = Sound::named("weather.wind_trees");
static THUNDER_NEAR: Sound = Sound::named("weather.thunder_near");
static THUNDER_FAR: Sound = Sound::named("weather.thunder_far");

/// Seconds between looks at the cover; each change eases over `EASE`.
const LOOK: f32 = 0.25;
const EASE: f32 = 0.8;
/// A bed fades in over this many seconds when it starts, out over `FADE_OUT` when it ends.
const FADE_IN: f32 = 2.0;
const FADE_OUT: f32 = 3.0;
/// Quieter than this a bed is stopped, giving its voice back.
const SILENT: f32 = 0.01;
/// Seconds between log lines while a bed plays (for scripts: `weather sound:`).
const LOG_EVERY: f32 = 10.0;
/// The head above the feet, blocks.
const EYE: f64 = 1.62;

struct State {
    /// Rain, heavy rain, roof, wind, trees.
    beds: [SoundHandle; 5],
    next_look: f32,
    next_log: f32,
    /// A bed played at the last log line: the line after they all stop says so once.
    logged_playing: bool,
    flashes: u32,
    /// Thunder on its way: seconds left, metres away.
    coming: Vec<(f32, f32)>,
    /// Seconds since a thunder was last heard; far ones closer together merge (`THUNDER_GAP`).
    since_thunder: f32,
    rng: u64,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

/// The cover last found over the player (open sky before the first look), for the ambience.
static COVER: Mutex<Cover> = Mutex::new(Cover { open: 1.0, roof: 0.0, leaves: 0.0 });

pub(crate) fn cover_now() -> Cover {
    *COVER.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn register(r: &mut Registry) {
    use sound_categories::AMBIENT;
    // Beds over footsteps and creatures: the weather is never crowded out by a herd.
    for (s, file, volume) in [
        (&RAIN, "weather/rain1", 0.9),
        (&RAIN_HEAVY, "weather/rain_heavy1", 1.3),
        (&RAIN_ROOF, "weather/rain_roof1", 0.9),
        (&WIND, "weather/wind1", 0.8),
        (&WIND_TREES, "weather/wind_trees1", 0.8),
    ] {
        r.sound(s)
            .files(&[file])
            .category(&AMBIENT)
            .volume(volume)
            .priority(50)
            .voices(2);
    }
    r.sound(&THUNDER_NEAR)
        .files(&["weather/thunder_near1", "weather/thunder_near2"])
        .category(&AMBIENT)
        .pitch(0.9, 1.05)
        .priority(60)
        .voices(3);
    r.sound(&THUNDER_FAR)
        .files(&["weather/thunder_far1", "weather/thunder_far2", "weather/thunder_far3", "weather/thunder_far4"])
        .category(&AMBIENT)
        .priority(60)
        .voices(3);
    r.client_frame(frame);
}

fn frame(c: &Client) {
    let st = c.state();
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    if st.flags & client::client_flags::JOINED == 0 {
        // Out of the world: the engine has ended its sounds; the next world starts afresh (its
        // flash count from zero, its beds not yet started).
        *guard = None;
        return;
    }
    let s = guard.get_or_insert_with(|| State {
        beds: [SoundHandle::default(); 5],
        next_look: 0.0,
        next_log: 0.0,
        logged_playing: false,
        flashes: st.lightnings,
        coming: Vec::new(),
        since_thunder: ws::THUNDER_GAP,
        rng: 0x9E37_79B9_7F4A_7C15 ^ st.frame,
    });
    let dt = st.dt.clamp(0.0, 0.5);
    let eye = [st.pos.x, st.pos.y + EYE, st.pos.z];
    // A flash: its thunder is on the way.
    if st.lightnings != s.flashes {
        s.flashes = st.lightnings;
        let metres = ws::strike_distance(random(&mut s.rng));
        s.coming.push((ws::thunder_delay(metres), metres));
    }
    s.next_look -= dt;
    s.since_thunder += dt;
    for t in &mut s.coming {
        t.0 -= dt;
    }
    let due = s.coming.iter().any(|t| t.0 <= 0.0);
    if s.next_look > 0.0 && !due {
        return;
    }
    let cover = base_api::hearing::cover(c, eye);
    *COVER.lock().unwrap_or_else(|e| e.into_inner()) = cover;
    for &(_, metres) in s.coming.iter().filter(|t| t.0 <= 0.0) {
        if ws::thunder_heard(s.since_thunder, metres) {
            thunder(c, metres, cover);
            s.since_thunder = 0.0;
        } else {
            debug!("thunder: {metres:.0} m away merged into the last one, {:.1} s ago", s.since_thunder);
        }
    }
    s.coming.retain(|t| t.0 > 0.0);
    if s.next_look > 0.0 {
        return;
    }
    s.next_look = LOOK;
    let speed = (st.wind[0] * st.wind[0] + st.wind[1] * st.wind[1]).sqrt();
    let l = ws::levels(st.rain, st.thunder, speed, st.snowing, cover);
    beds(c, s, &l);
    s.next_log -= LOOK;
    let playing = s.beds.iter().any(|b| b.playing(c));
    if (playing || s.logged_playing) && s.next_log <= 0.0 {
        s.next_log = LOG_EVERY;
        s.logged_playing = playing;
        // `enclosed` is the one cover value the muffle follows (1 - open).
        info!(
            "weather sound: rain {:.2} heavy {:.2} roof {:.2} wind {:.2} trees {:.2} (enclosed {:.2}, open {:.2} roof {:.2} leaves {:.2}, rain {:.2} thunder {:.2} snow {:.2}, wind {speed:.1} m/s, muffle {:.0} Hz){}",
            l.rain,
            l.heavy,
            l.roof,
            l.wind,
            l.trees,
            1.0 - cover.open,
            cover.open,
            cover.roof,
            cover.leaves,
            st.rain,
            st.thunder,
            st.snowing,
            l.lowpass,
            if playing { "" } else { ", all beds quiet" }
        );
    }
}

/// Eases each bed to its level, starting it when it should be heard and stopping it when not.
fn beds(c: &Client, s: &mut State, l: &Levels) {
    let wanted: [(&Sound, f32, Option<f32>, bool); 5] = [
        (&RAIN, l.rain, None, true),
        (&RAIN_HEAVY, l.heavy, None, true),
        // The roof is heard from inside, clear.
        (&RAIN_ROOF, l.roof, None, false),
        (&WIND, l.wind, Some(l.wind_pitch), true),
        (&WIND_TREES, l.trees, Some(l.wind_pitch), true),
    ];
    for (bed, (sound, level, pitch, muffled)) in s.beds.iter_mut().zip(wanted) {
        let on = bed.playing(c);
        if level < SILENT {
            if on {
                bed.stop(c, FADE_OUT);
                *bed = SoundHandle::default();
            }
            continue;
        }
        if !on {
            *bed = c.start_sound(sound, None, level, true, FADE_IN);
        } else {
            bed.volume(c, level, EASE);
        }
        if let Some(p) = pitch {
            bed.pitch(c, p, EASE);
        }
        bed.muffle(c, if muffled { l.lowpass } else { 20_000.0 }, EASE);
    }
}

/// One strike's thunder, `metres` away: a crack near, a rumble far, lower, later and duller the
/// farther it is, and duller indoors.
fn thunder(c: &Client, metres: f32, cover: Cover) {
    let sound = if metres < ws::CRACK_WITHIN { &THUNDER_NEAR } else { &THUNDER_FAR };
    let volume = ws::thunder_volume(metres) * (ws::THROUGH_WALLS + (1.0 - ws::THROUGH_WALLS) * cover.open).max(0.5);
    let h = c.start_sound(sound, None, volume, false, 0.0);
    h.pitch(c, ws::thunder_pitch(metres), 0.0);
    h.muffle(c, ws::thunder_lowpass(metres).min(ws::muffle(cover.open.max(0.3))), 0.0);
    info!("thunder: {metres:.0} m away, heard {:.1} s after its flash, volume {volume:.2}", ws::thunder_delay(metres));
}

/// How open the head is to the sky, whether a roof is close over it, and how much of what is
/// around has leaves overhead.
fn random(r: &mut u64) -> f32 {
    *r ^= *r << 13;
    *r ^= *r >> 7;
    *r ^= *r << 17;
    (*r >> 40) as f32 / (1u64 << 24) as f32
}
