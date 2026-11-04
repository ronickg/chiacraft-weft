//! What every species sounds like (`notes/SYSTEMS.md`, creature voices): a voice family each
//! (its files in `mods/animals/assets/sounds/voice/<family>/`, all CC0, `SOURCES.md` there),
//! pitched by the species' size: a smaller animal of a family calls higher. The animals mod gives
//! each species' kind events `animals:<species>.hurt|death|ambient` from its family's files.
//!
//! Dinosaurs get no mammal roars: closed-mouth calls like a crocodile's bellow or an emu's boom,
//! and birdlike hisses and squawks for the small feathered ones, which is what their living
//! relatives (crocodilians and birds) suggest.

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

pub const FAMILIES: [Family; 16] = [
    Family { name: "canine", hurt: 3, death: 2, ambient: 3, volume: 1.0, range: 32.0 },
    Family { name: "fox", hurt: 2, death: 1, ambient: 2, volume: 0.9, range: 24.0 },
    Family { name: "cat", hurt: 2, death: 1, ambient: 3, volume: 0.8, range: 16.0 },
    Family { name: "big_cat", hurt: 2, death: 1, ambient: 2, volume: 1.2, range: 32.0 },
    Family { name: "deer", hurt: 2, death: 1, ambient: 2, volume: 0.9, range: 24.0 },
    Family { name: "rodent", hurt: 2, death: 1, ambient: 2, volume: 0.6, range: 8.0 },
    // Hares only scream when caught.
    Family { name: "hare", hurt: 2, death: 1, ambient: 0, volume: 0.8, range: 16.0 },
    Family { name: "songbird", hurt: 1, death: 1, ambient: 3, volume: 0.6, range: 24.0 },
    Family { name: "hawk", hurt: 1, death: 1, ambient: 2, volume: 1.0, range: 48.0 },
    Family { name: "owl", hurt: 1, death: 1, ambient: 2, volume: 0.9, range: 40.0 },
    Family { name: "gull", hurt: 1, death: 1, ambient: 3, volume: 0.9, range: 40.0 },
    Family { name: "corvid", hurt: 1, death: 1, ambient: 3, volume: 0.9, range: 40.0 },
    // Crocodiles, lizards and vultures hiss; the big ones bellow.
    Family { name: "hiss", hurt: 2, death: 1, ambient: 1, volume: 0.8, range: 12.0 },
    Family { name: "bellow", hurt: 2, death: 1, ambient: 2, volume: 1.3, range: 48.0 },
    Family { name: "small_dino", hurt: 2, death: 1, ambient: 2, volume: 1.0, range: 24.0 },
    Family { name: "dragon", hurt: 1, death: 1, ambient: 2, volume: 1.4, range: 64.0 },
];

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
    v("jaguar", "big_cat", 1.0),
    v("white_tailed_deer", "deer", 1.05),
    v("red_deer", "deer", 0.9),
    v("reindeer", "deer", 0.85),
    v("capybara", "rodent", 0.55),
    v("wood_mouse", "rodent", 1.2),
    v("field_vole", "rodent", 1.1),
    v("kryptobaatar", "rodent", 1.0),
    v("hare", "hare", 0.75),
    v("mountain_hare", "hare", 0.7),
    v("house_sparrow", "songbird", 1.0),
    v("snow_bunting", "songbird", 1.1),
    v("confuciusornis", "songbird", 0.85),
    v("gobipteryx", "songbird", 0.95),
    v("golden_eagle", "hawk", 0.9),
    v("savanna_hawk", "hawk", 1.05),
    v("great_horned_owl", "owl", 0.95),
    v("snowy_owl", "owl", 1.05),
    v("herring_gull", "gull", 1.0),
    v("raven", "corvid", 0.95),
    v("turkey_vulture", "hiss", 1.1),
    v("yacare_caiman", "hiss", 0.9),
    v("spectacled_caiman", "hiss", 0.95),
    v("crocodyliform", "bellow", 1.1),
    v("priscagama", "hiss", 1.4),
    v("raptor", "small_dino", 1.0),
    v("osmolskae", "small_dino", 1.1),
    v("utahraptor", "small_dino", 0.75),
    v("citipati", "small_dino", 0.9),
    v("psittacosaurus", "bellow", 1.35),
    v("protoceratops", "bellow", 1.2),
    v("gastonia", "bellow", 0.95),
    v("pinacosaurus", "bellow", 1.0),
    v("iguanodont", "bellow", 0.85),
    v("cedarosaurus", "bellow", 0.6),
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
