//! What every species sounds like (`notes/SYSTEMS.md`, creature voices): a voice family each
//! (its files in `mods/animals/assets/sounds/voice/<family>/`, all CC0, `SOURCES.md` there),
//! pitched by the species' size: a smaller animal of a family calls higher. The animals mod gives
//! each species' kind events `animals:<species>.hurt|death|ambient` from its family's files.
//!
//! Dinosaurs get no mammal roars: closed-mouth calls like a crocodile's bellow or an emu's boom,
//! and birdlike hisses and squawks for the small feathered ones, which is what their living
//! relatives (crocodilians and birds) suggest.
//!
//! Families marked AI are made with Stable Audio 3 Small SFX (Ronald, 11-25: "Try AI sounds", then
//! "sounds great pick whatever"), for animals no CC0 recording fitted: the bosses and their young,
//! the dinosaurs, and species that borrowed another animal's voice. Prompts, seeds and picks:
//! `notes/crew/plans/ai-sound.md`; licence rows in the animals' `SOURCES.md`.

/// A family of calls; `hurt`, `death` and `ambient` are how many files of each it has (0: that
/// one is silent).
#[derive(Clone, Copy, Debug)]
pub struct Family {
    pub name: &'static str,
    pub hurt: u32,
    pub death: u32,
    pub ambient: u32,
    /// Times the creature category's volume.
    pub volume: f32,
    /// Blocks away it is still heard.
    pub range: f32,
}

pub const FAMILIES: [Family; 26] = [
    Family { name: "canine", hurt: 3, death: 2, ambient: 3, volume: 1.0, range: 32.0 },
    Family { name: "fox", hurt: 2, death: 1, ambient: 2, volume: 0.9, range: 24.0 },
    Family { name: "cat", hurt: 2, death: 1, ambient: 3, volume: 0.8, range: 16.0 },
    Family { name: "deer", hurt: 2, death: 1, ambient: 2, volume: 0.9, range: 24.0 },
    Family { name: "rodent", hurt: 2, death: 1, ambient: 2, volume: 0.6, range: 8.0 },
    // Hares only scream when caught (AI).
    Family { name: "hare", hurt: 2, death: 1, ambient: 0, volume: 0.8, range: 16.0 },
    Family { name: "songbird", hurt: 1, death: 1, ambient: 3, volume: 0.6, range: 24.0 },
    Family { name: "hawk", hurt: 1, death: 1, ambient: 2, volume: 1.0, range: 48.0 },
    Family { name: "owl", hurt: 1, death: 1, ambient: 2, volume: 0.9, range: 40.0 },
    Family { name: "gull", hurt: 1, death: 1, ambient: 3, volume: 0.9, range: 40.0 },
    Family { name: "corvid", hurt: 1, death: 1, ambient: 3, volume: 0.9, range: 40.0 },
    // Crocodiles, lizards and vultures hiss; the big crocodyliform bellows.
    Family { name: "hiss", hurt: 2, death: 1, ambient: 1, volume: 0.8, range: 12.0 },
    Family { name: "bellow", hurt: 2, death: 1, ambient: 2, volume: 1.3, range: 48.0 },
    Family { name: "dragon", hurt: 1, death: 1, ambient: 2, volume: 1.4, range: 64.0 },
    // AI from here on.
    Family { name: "jaguar", hurt: 2, death: 1, ambient: 3, volume: 1.2, range: 32.0 },
    Family { name: "reindeer", hurt: 2, death: 1, ambient: 3, volume: 0.9, range: 24.0 },
    Family { name: "capybara", hurt: 2, death: 1, ambient: 3, volume: 0.7, range: 12.0 },
    Family { name: "eagle", hurt: 1, death: 1, ambient: 3, volume: 1.0, range: 48.0 },
    Family { name: "snowy_owl", hurt: 1, death: 1, ambient: 3, volume: 0.9, range: 40.0 },
    Family { name: "bunting", hurt: 1, death: 1, ambient: 3, volume: 0.6, range: 24.0 },
    // The plant-eating dinosaurs: booms, honks and rumbles, pitched by size.
    Family { name: "horned_dino", hurt: 3, death: 2, ambient: 3, volume: 1.3, range: 48.0 },
    // The small feathered hunters: birdlike chirps, hisses and squawks.
    Family { name: "raptor", hurt: 3, death: 2, ambient: 3, volume: 1.0, range: 24.0 },
    // The bosses' young.
    Family { name: "goat_kid", hurt: 2, death: 1, ambient: 3, volume: 0.8, range: 16.0 },
    Family { name: "calf", hurt: 2, death: 1, ambient: 3, volume: 0.9, range: 20.0 },
    // The bosses: heard across the fight, with more calls of their own (`BOSS_CALLS`).
    Family { name: "drenkaun", hurt: 3, death: 2, ambient: 3, volume: 1.5, range: 96.0 },
    Family { name: "borkon", hurt: 3, death: 2, ambient: 3, volume: 1.5, range: 96.0 },
];

/// A boss's calls beyond hurt, death and ambient, each `voice/<boss>/<event><n>`: `summon` as it
/// rises at its altar, `attack` as an attack's wind-up starts, `stunned` when it runs into a wall.
pub const BOSS_CALLS: [(&str, &str, u32); 6] = [
    ("drenkaun", "summon", 2),
    ("drenkaun", "attack", 3),
    ("drenkaun", "stunned", 2),
    ("borkon", "summon", 2),
    ("borkon", "attack", 3),
    ("borkon", "stunned", 2),
];

/// The bosses' young (not species): their kind and voice family.
pub const YOUNG: [(&str, &str); 2] = [("borkon_kid", "goat_kid"), ("drenkaun_calf", "calf")];

/// A species' voice: its family and playback speed (1 as recorded).
#[derive(Clone, Copy, Debug)]
pub struct Voice {
    pub species: &'static str,
    pub family: &'static str,
    pub pitch: f32,
}

const fn v(species: &'static str, family: &'static str, pitch: f32) -> Voice {
    Voice { species, family, pitch }
}

/// Every species with a voice; fish are silent and have none.
pub const VOICES: [Voice; 42] = [
    v("dog", "canine", 1.35),
    v("grey_wolf", "canine", 0.9),
    v("coyote", "canine", 1.1),
    v("red_fox", "fox", 1.0),
    v("arctic_fox", "fox", 1.05),
    v("cat", "cat", 1.0),
    v("jaguar", "jaguar", 1.0),
    v("white_tailed_deer", "deer", 1.05),
    v("red_deer", "deer", 0.9),
    v("reindeer", "reindeer", 1.0),
    v("capybara", "capybara", 1.0),
    v("wood_mouse", "rodent", 1.2),
    v("field_vole", "rodent", 1.1),
    v("kryptobaatar", "rodent", 1.0),
    v("hare", "hare", 0.75),
    v("mountain_hare", "hare", 0.7),
    v("house_sparrow", "songbird", 1.0),
    v("snow_bunting", "bunting", 1.0),
    v("confuciusornis", "songbird", 0.85),
    v("gobipteryx", "songbird", 0.95),
    v("golden_eagle", "eagle", 1.0),
    v("savanna_hawk", "hawk", 1.05),
    v("great_horned_owl", "owl", 0.95),
    v("snowy_owl", "snowy_owl", 1.0),
    v("herring_gull", "gull", 1.0),
    v("raven", "corvid", 0.95),
    v("turkey_vulture", "hiss", 1.1),
    v("yacare_caiman", "hiss", 0.9),
    v("spectacled_caiman", "hiss", 0.95),
    v("crocodyliform", "bellow", 1.1),
    v("priscagama", "hiss", 1.4),
    v("raptor", "raptor", 1.0),
    v("osmolskae", "raptor", 1.1),
    v("utahraptor", "raptor", 0.75),
    v("citipati", "raptor", 0.9),
    v("psittacosaurus", "horned_dino", 1.35),
    v("protoceratops", "horned_dino", 1.2),
    v("gastonia", "horned_dino", 0.95),
    v("pinacosaurus", "horned_dino", 1.0),
    v("iguanodont", "horned_dino", 0.85),
    v("cedarosaurus", "horned_dino", 0.6),
    v("wyvern", "dragon", 1.0),
];

pub fn family(name: &str) -> Option<&'static Family> {
    FAMILIES.iter().find(|f| f.name == name)
}

pub fn of(species: &str) -> Option<&'static Voice> {
    VOICES.iter().find(|v| v.species == species)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_voice_has_a_family_and_every_species_is_known() {
        for v in VOICES {
            assert!(family(v.family).is_some(), "{} has no family {}", v.species, v.family);
            assert!(crate::species::SPECIES.iter().any(|s| s.name == v.species), "no species {}", v.species);
        }
        for (kind, fam) in YOUNG {
            assert!(family(fam).is_some(), "{kind} has no family {fam}");
        }
        for (boss, event, n) in BOSS_CALLS {
            assert!(family(boss).is_some() && n > 0, "{boss}.{event}");
        }
        // Every species not a fish has a voice.
        let fish = ["brown_trout", "pacu", "lungfish", "lycoptera"];
        for s in crate::species::SPECIES
            .iter()
            .filter(|s| !s.name.is_empty())
        {
            assert!(fish.contains(&s.name) || of(s.name).is_some(), "{} has no voice", s.name);
        }
    }
}
