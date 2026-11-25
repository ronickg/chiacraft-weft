//! What the listener hears of the open air: the cover over and round their head, sampled from the
//! client's own blocks and sky light (`vitals::weather_sound::Cover`, the numbers there). The
//! weather's and the land's beds read it every half second (`mods/base/src/weather_sound.rs`);
//! any mod starting a sound outdoors (ambient life's cries and splashes) reads it as it starts one,
//! through `vitals::weather_sound::outdoors` for its volume and muffle. Each mod has its own copy of
//! this code (mods are separate libraries), so there's no shared state: only the one way to sample.

use std::collections::HashMap;
use std::sync::Mutex;

use vitals::weather_sound::{self as ws, Cover};
use weft_sdk::*;

/// The head above the feet, blocks.
pub const EYE: f64 = 1.62;
/// Leaves within this many blocks over a sampled place count as trees there.
pub const LEAVES_UP: i32 = 12;

/// Block states with "leaves" in their name, as found (a name read once per state).
static LEAFY: Mutex<Option<HashMap<u32, bool>>> = Mutex::new(None);

/// The cover round an eye at `eye`: the head and rings of places round it (`SAMPLE_RINGS`), each
/// open to the sky or not, under leaves or not, and a roof the rain falls on over the head.
pub fn cover(c: &Client, eye: [f64; 3]) -> Cover {
    let head = [eye[0].floor() as i32, eye[1].floor() as i32, eye[2].floor() as i32];
    let sky = |p: [i32; 3]| c.light(p).map_or(0, |l| l & 255);
    let block = |p: [i32; 3]| c.block(p).unwrap_or(0);
    let mut cache = LEAFY.lock().unwrap();
    let cache = cache.get_or_insert_with(HashMap::new);
    let mut leafy = |state: u32| *cache.entry(state).or_insert_with(|| block_name(state).contains("leaves"));
    let mut places = vec![(head, 2.0f32)];
    for r in ws::SAMPLE_RINGS {
        for (dx, dz) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
            places.push(([head[0] + dx * r, head[1], head[2] + dz * r], if r == ws::SAMPLE_RINGS[0] { 1.0 } else { 0.5 }));
        }
    }
    let (mut open, mut leaves, mut all) = (0.0, 0.0, 0.0);
    for (p, w) in places {
        all += w;
        let over_leaves = (1..=LEAVES_UP).any(|dy| leafy(block([p[0], p[1] + dy, p[2]])));
        if over_leaves {
            leaves += w;
        }
        match sky(p) {
            15 => open += w,
            // Under a canopy the sky comes through the leaves, a little less.
            l if l >= 12 && over_leaves => open += w * 0.7,
            _ => {}
        }
    }
    // A roof: the first block over the head, if the sky reaches the top of it (rain falls on it).
    let mut roof = 0.0;
    if sky(head) < 15
        && let Some(dy) = (1..=ws::ROOF_UP).find(|&dy| block([head[0], head[1] + dy, head[2]]) != 0)
    {
        let top = (dy..dy + 6).find(|&y| block([head[0], head[1] + y, head[2]]) == 0);
        if top.is_some_and(|y| sky([head[0], head[1] + y, head[2]]) == 15) {
            roof = 1.0 - (dy - 1) as f32 / ws::ROOF_UP as f32;
        }
    }
    Cover { open: open / all, roof, leaves: leaves / all }
}

/// The cover round this client's player now (open air if there's no player yet).
pub fn cover_here(c: &Client) -> Cover {
    match c.view().player_state() {
        Some(st) => cover(c, [st.pos.x, st.pos.y + EYE, st.pos.z]),
        None => Cover { open: 1.0, roof: 0.0, leaves: 0.0 },
    }
}

/// Starts a one-shot sound from out in the open at `at`, as the listener hears it from where they
/// are: quieter and muffled indoors, nothing deep under ground (`ws::outdoors`). Returns the volume
/// and the muffle it was started with (0 volume: not started).
pub fn start_outdoors(c: &Client, sound: &Sound, at: [f64; 3], volume: f32) -> (f32, f32) {
    let (heard, muffle) = ws::outdoors(cover_here(c));
    if heard <= 0.0 {
        return (0.0, muffle);
    }
    let h = c.start_sound(sound, Some(at), volume * heard, false, 0.0);
    h.muffle(c, muffle, 0.0);
    (volume * heard, muffle)
}
