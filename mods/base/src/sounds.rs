//! Sound events: block groups (`block.<group>.break|place|step|hit`) blocks pick with
//! `.sound_group(group)`, and the background music; mobs' voices come with them (`mobs.rs`). Files are in `assets/sounds/`.

use weft_sdk::abi::music_when as mw;
use weft_sdk::*;

/// Every block group (`block.<group>.*`), each with `dig/<group>1-3` and `step/<group>1-3`
/// (`SOURCES.md`). Which block has which: `assets/sounds/groups.txt` for those not given one
/// where they are made.
pub const GROUPS: [&str; 13] =
    ["stone", "wood", "gravel", "grass", "sand", "glass", "cloth", "dirt", "snow", "leaves", "ice", "mud", "metal"];

pub fn register(r: &mut Registry) {
    for g in GROUPS {
        let files = |kind: &str| {
            (1..=3)
                .map(|i| format!("{kind}/{g}{i}"))
                .collect::<Vec<_>>()
        };
        let (dig, step) = (files("dig"), files("step"));
        let dig: Vec<&str> = dig.iter().map(String::as_str).collect();
        let step: Vec<&str> = step.iter().map(String::as_str).collect();
        // Vanilla breaks glass with its own sound and knocks on it like on stone; ice cracks
        // and metal clangs when they go.
        let own = |dir: &str| (1..=3).map(|i| format!("{dir}{g}{i}")).collect::<Vec<_>>();
        let broken = match g {
            "glass" => own("random/"),
            "ice" | "metal" => own("break/"),
            _ => Vec::new(),
        };
        let breaks: Vec<&str> = if broken.is_empty() { dig.clone() } else { broken.iter().map(String::as_str).collect() };
        r.sound(Sound::generated(format!("block.{g}.break")))
            .files(&breaks)
            .pitch(0.75, 0.85);
        r.sound(Sound::generated(format!("block.{g}.place")))
            .files(&dig)
            .pitch(0.75, 0.85);
        r.sound(Sound::generated(format!("block.{g}.step")))
            .files(&step)
            .pitch(0.9, 1.1);
        r.sound(Sound::generated(format!("block.{g}.hit")))
            .files(&step)
            .volume(0.6)
            .pitch(0.5, 0.55);
    }
    for line in include_str!("../assets/sounds/groups.txt").lines() {
        let Some((group, blocks)) = line.split_once(':').filter(|_| !line.starts_with('#')) else { continue };
        let group = group.trim();
        debug_assert!(GROUPS.contains(&group), "groups.txt: no group {group}");
        for b in blocks.split_whitespace() {
            r.sound_group_on(b, group);
        }
    }
    r.music(&Music::named("calm"))
        .tracks(&["music/calm1"])
        .when(mw::SURFACE);
    r.music(&Music::named("deep"))
        .tracks(&["music/deep1"])
        .when(mw::UNDERGROUND);
}
