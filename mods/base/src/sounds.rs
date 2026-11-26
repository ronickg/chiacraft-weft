//! Sound events: block groups (`block.<group>.break|place|step|hit`) blocks pick with
//! `.sound_group(group)`, and the background music; mobs' voices come with them (`mobs.rs`). Files are in `assets/sounds/`.

use weft_sdk::*;

/// Every block group (`block.<group>.*`), each with `dig/<group>1-6`, `step/<group>1-6`,
/// `place/<group>1-3` and `hit/<group>1-3` (`SOURCES.md`: 1-3 of dig and step from the free
/// packs, the rest AI-made). Which block has which: `assets/sounds/groups.txt` for those not
/// given one where they are made.
pub const GROUPS: [&str; 13] =
    ["stone", "wood", "gravel", "grass", "sand", "glass", "cloth", "dirt", "snow", "leaves", "ice", "mud", "metal"];
static SHEEP: Music = Music::named("sheep");

fn strs(v: &[String]) -> Vec<&str> {
    v.iter().map(String::as_str).collect()
}

pub fn register(r: &mut Registry) {
    for g in GROUPS {
        let files = |kind: &str, n: u32| (1..=n).map(|i| format!("{kind}/{g}{i}")).collect::<Vec<_>>();
        let (dig, step, place, hit) = (files("dig", 6), files("step", 6), files("place", 3), files("hit", 3));
        // Vanilla breaks glass with its own sound; ice cracks and metal clangs when they go.
        let broken = match g {
            "glass" => (1..=3).map(|i| format!("random/{g}{i}")).collect(),
            "ice" | "metal" => files("break", 3),
            _ => dig.clone(),
        };
        r.sound(Sound::generated(format!("block.{g}.break")))
            .files(&strs(&broken))
            .pitch(0.75, 0.85);
        r.sound(Sound::generated(format!("block.{g}.place")))
            .files(&strs(&place))
            .pitch(0.9, 1.1);
        r.sound(Sound::generated(format!("block.{g}.step")))
            .files(&strs(&step))
            .pitch(0.9, 1.1);
        r.sound(Sound::generated(format!("block.{g}.hit")))
            .files(&strs(&hit))
            .volume(0.6)
            .pitch(0.9, 1.1);
    }
    for line in include_str!("../assets/sounds/groups.txt").lines() {
        let Some((group, blocks)) = line.split_once(':').filter(|_| !line.starts_with('#')) else { continue };
        let group = group.trim();
        debug_assert!(GROUPS.contains(&group), "groups.txt: no group {group}");
        for b in blocks.split_whitespace() {
            r.sound_group_on(b, group);
        }
    }
    // Upstream's music (our calm1 track is gone with their asset clean-up).
    r.music(&SHEEP)
        .tracks(&["music/418DreamingSheep"])
        .when(&conditions::SURFACE);
}
