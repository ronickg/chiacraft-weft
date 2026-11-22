//! Compare mode: one switch that makes a look shot like-for-like with a reference (Bliss through
//! `~/mcshaders/bliss_ctl.sh` on the PC, vanilla). Ronald, 11-22: "we gotta make the game easier
//! to compare by like pausing time no clouds etc".
//!
//! `/compare on [override ...]`, the script step `compare on [override ...]`, or
//! `--config testkit:compare_mode=1` (applied at the script's first `loaded`). It sets, in one go:
//! time frozen, weather clear and pinned, no clouds of any kind (soft clouds, their shadows, the
//! pack's cirrus, Weft's own), a fixed exposure, no bloom, no wind and no mist (`/air 0 0 0 mist`),
//! no mob spawning, the HUD hidden, GUI scale 2 and FOV 70. The shader clock is already frozen in
//! scripted runs. `/compare off` puts the settings back as they were and lets time run.
//!
//! Each control can be overridden alone, for the steps that test it: `exposure=1` (auto exposure
//! back on), `bloom=2`, `clouds=1` (all the clouds back), `mist=0.3` (pinned mist), `wind=1`, or
//! any setting by its full name (`base:look_leaf_sun=3`).

use std::sync::Mutex;
use weft_sdk::{Client, Config};

pub static COMPARE_MODE: Config<bool> = Config::named("compare_mode");

/// The settings compare mode sets: full name, value.
const SETTINGS: &[(&str, f64)] = &[
    ("core:soft_clouds", 0.0),
    ("core:cloud_shadows", 0.0),
    ("base:look_cirrus_off", 1.0),
    // Weft's own clouds: none drawn with no cloud distance.
    ("core:cloud_distance", 0.0),
    ("base:look_auto_exposure", 0.0),
    ("core:bloom", 0.0),
    ("core:gui_scale", 2.0),
    ("core:fov", 70.0),
];

/// The settings as they were before compare mode, and whether it hid the HUD.
type Saved = (Vec<(&'static str, f64)>, bool);
static BEFORE: Mutex<Option<Saved>> = Mutex::new(None);

fn set(name: &str, v: f64) {
    let name: &'static str = Box::leak(name.to_string().into_boxed_str());
    Config::<f64>::named(name).set(v);
}

fn get(name: &'static str) -> f64 {
    Config::<f64>::named(name).get()
}

/// Turns compare mode on or off on client `c`; `overrides` are `key=value` words (see the module).
/// The answer is a line for the log or the chat.
pub fn apply(c: &Client, on: bool, overrides: &str) -> String {
    let mut before = BEFORE.lock().unwrap();
    if !on {
        let Some((saved, hid)) = before.take() else { return "compare mode was off".into() };
        for (name, v) in saved {
            set(name, v);
        }
        if hid {
            c.input_key("F1", 0.0);
        }
        c.say("/tick unfreeze");
        return "compare mode off".into();
    }
    let mut values: Vec<(String, f64)> = SETTINGS.iter().map(|&(n, v)| (n.to_string(), v)).collect();
    let (mut mist, mut wind) = (0.0, 0.0);
    for word in overrides.split_whitespace() {
        let Some((k, v)) = word.split_once('=') else { continue };
        let Ok(v) = v.parse::<f64>() else { continue };
        match k {
            "exposure" => values.push(("base:look_auto_exposure".into(), v)),
            "bloom" => values.push(("core:bloom".into(), v)),
            "clouds" if v > 0.0 => {
                values.retain(|(n, _)| !n.contains("cloud"));
            }
            "mist" => mist = v,
            "wind" => wind = v,
            _ => values.push((k.to_string(), v)),
        }
    }
    if before.is_none() {
        let saved = values
            .iter()
            .map(|(n, _)| {
                let n: &'static str = Box::leak(n.clone().into_boxed_str());
                (n, get(n))
            })
            .collect();
        c.input_key("F1", 0.0);
        *before = Some((saved, true));
    }
    for (n, v) in &values {
        set(n, *v);
    }
    for line in [
        "/rule core:mob_spawning 0".to_string(),
        "/weather clear 99999".to_string(),
        format!("/air {mist} {wind} 0 mist"),
        "/tick freeze".to_string(),
    ] {
        c.say(&line);
    }
    format!("compare mode on ({} settings, mist {mist}, wind {wind}){}", values.len(), if overrides.is_empty() { String::new() } else { format!(": {overrides}") })
}

/// The game ticks of hour `h` (06:00 is 0).
pub fn hour_ticks(h: f32) -> i64 {
    (((h - 6.0) * 1000.0).round() as i64).rem_euclid(24000)
}
