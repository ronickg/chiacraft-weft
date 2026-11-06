//! Eggs by species: which look each laying species' egg has, its real length and the length it is
//! drawn at, so a raptor's egg reads as a raptor's at a glance (Ronald, 2026-11-06: "I don't want
//! them to all look the same") and every egg sits with Minecraft-style blocks and items (Ronald:
//! "a bit too different"). The evidence behind each, species by species, is
//! `reports/Eggs by species.md`; how Minecraft and its mods draw eggs, `reports/Eggs in Minecraft
//! mods.md`. The looks are built by `mods/animals/source/egg/build.py` (a model, a texture
//! `egg/<look>`, an item icon `egg_<look>`).
//!
//! Drawn sizes follow the mods: about five times life, 1.6 pixels (texels at the house's 32 a
//! block) a real cm, 5 to 28 px long; a species is drawn at its family's look scaled by at most
//! 15% (more would change the texels' size: then it gets the look at its own size).

/// One look: a family's egg, and the length its model is built at (px).
pub struct Look {
    pub name: &'static str,
    /// The model (`assets/models/<it>.geo.json`, `egg_<name>`).
    pub model: &'static str,
    /// The egg's length as built, px (a roe clutch: one bead).
    pub px: f32,
}

/// Pixels an egg is drawn long for each real cm, and the smallest and biggest drawn.
pub const PX_PER_CM: f32 = 1.6;
pub const PX_MIN: f32 = 5.0;
pub const PX_MAX: f32 = 28.0;

/// A species' egg drawn length for its real length, px.
pub fn drawn_px(cm: f32) -> f32 {
    (PX_PER_CM * cm).round().clamp(PX_MIN, PX_MAX)
}

pub const LOOKS: &[Look] = &[
    // Long ovals, a little narrower at the bottom, pale sky blue-green with brown blotches and
    // speckles, stood nearly upright (Deinonychus's pigments, Wiemann et al. 2018).
    Look { name: "dromaeosaur", model: "egg_dromaeosaur", px: 14.0 },
    // The same, Utahraptor's size (under the wyvern's).
    Look { name: "dromaeosaur_large", model: "egg_dromaeosaur_large", px: 24.0 },
    // The longest, ends alike, dark teal with lengthwise ridges, lying inclined (Citipati, 18 cm).
    Look { name: "oviraptorid", model: "egg_oviraptorid", px: 28.0 },
    // Soft-shelled, pale buff, dented, buried lying (Protoceratops 10.1 cm, Norell et al. 2020).
    Look { name: "ceratopsian", model: "egg_ceratopsian", px: 16.0 },
    // The same, Psittacosaurus' size (about 7 cm, inferred).
    Look { name: "ceratopsian_small", model: "egg_ceratopsian_small", px: 11.0 },
    // Our pick, no ankylosaur egg known: near round, leathery, dull olive-grey, mottled.
    Look { name: "ankylosaur", model: "egg_ankylosaur", px: 19.0 },
    // Big, near round, warm ivory, a fine net of bumps (hadrosaur-like spheroolithid eggs).
    Look { name: "ornithopod", model: "egg_ornithopod", px: 26.0 },
    // A ball, chalk white, coarse knobs (titanosaur eggs of Auca Mahuevo, about 13 cm).
    Look { name: "sauropod", model: "egg_sauropod", px: 21.0 },
    // Small, both ends alike, a chalky band round the middle, lying (Caiman yacare, 6.8 cm).
    Look { name: "crocodilian", model: "egg_crocodilian", px: 11.0 },
    // Tiny, white, matt, dented.
    Look { name: "lizard", model: "egg_lizard", px: 5.0 },
    // Fish: a clutch of a few 3 px beads (a lungfish's jelly balls 5 px), told apart by color and lay.
    Look { name: "roe_trout", model: "egg_roe_trout", px: 3.0 },
    Look { name: "roe_pacu", model: "egg_roe_pacu", px: 3.0 },
    Look { name: "roe_lycoptera", model: "egg_roe_lycoptera", px: 3.0 },
    Look { name: "roe_lungfish", model: "egg_roe_lungfish", px: 5.0 },
    // Fantasy: the biggest, stone-heavy, in metal-rimmed jewel scales; our own design.
    Look { name: "wyvern", model: "egg_wyvern", px: 28.0 },
    // Birds, lying in the nest's cup, each its own colors. Owls: plain white, nearly round.
    Look { name: "bird_owl", model: "egg_bird_owl", px: 9.0 },
    // Songbirds: grey-white densely speckled, and bluish white with red-brown spots at the
    // blunt end.
    Look { name: "bird_sparrow", model: "egg_bird_sparrow", px: 5.0 },
    Look { name: "bird_bunting", model: "egg_bird_bunting", px: 5.0 },
    // Blotched ovals: the raven's green, the gull's olive-brown, the eagle's red-brown on white,
    // the hawk's bluish white, the vulture's cream wreathed dark brown at the blunt end.
    Look { name: "bird_raven", model: "egg_bird_raven", px: 8.0 },
    Look { name: "bird_gull", model: "egg_bird_gull", px: 11.0 },
    Look { name: "bird_eagle", model: "egg_bird_eagle", px: 12.0 },
    Look { name: "bird_hawk", model: "egg_bird_hawk", px: 9.0 },
    Look { name: "bird_vulture", model: "egg_bird_vulture", px: 11.0 },
    // The Cretaceous birds: long, slim, plain pale, upright and half buried (Gobioolithus).
    Look { name: "bird_mesozoic", model: "egg_bird_mesozoic", px: 6.0 },
];

/// Each laying species (`species::SPECIES` name): its egg's look, real length in cm (a roe
/// clutch: its bead's few millimetres) and drawn length in px (0: by the rule, `drawn_px`).
/// Guesses are marked in the report.
pub const EGGS: &[(&str, &str, f32, f32)] = &[
    ("raptor", "dromaeosaur", 9.0, 0.0),
    ("osmolskae", "dromaeosaur", 9.0, 0.0),
    // Utahraptor's egg is a guess scaled from its mass (report: about 18 x 8 cm); drawn at 24 px,
    // under the wyvern's 28.
    ("utahraptor", "dromaeosaur_large", 18.0, 24.0),
    ("citipati", "oviraptorid", 18.0, 0.0),
    ("protoceratops", "ceratopsian", 10.1, 0.0),
    // No Psittacosaurus egg is known: the same soft egg, smaller (report: about 7 cm).
    ("psittacosaurus", "ceratopsian_small", 7.0, 0.0),
    ("gastonia", "ankylosaur", 12.0, 0.0),
    ("pinacosaurus", "ankylosaur", 12.0, 0.0),
    ("iguanodont", "ornithopod", 16.0, 0.0),
    ("cedarosaurus", "sauropod", 13.0, 0.0),
    ("yacare_caiman", "crocodilian", 6.8, 0.0),
    ("spectacled_caiman", "crocodilian", 6.4, 0.0),
    ("crocodyliform", "crocodilian", 7.0, 0.0),
    ("priscagama", "lizard", 1.6, 0.0),
    ("brown_trout", "roe_trout", 0.5, 0.0),
    ("pacu", "roe_pacu", 0.3, 0.0),
    ("lycoptera", "roe_lycoptera", 0.4, 0.0),
    ("lungfish", "roe_lungfish", 1.0, 0.0),
    // Fantasy: half a metre.
    ("wyvern", "wyvern", 50.0, 0.0),
    ("great_horned_owl", "bird_owl", 5.5, 0.0),
    ("snowy_owl", "bird_owl", 5.6, 0.0),
    ("house_sparrow", "bird_sparrow", 2.1, 0.0),
    ("snow_bunting", "bird_bunting", 2.3, 0.0),
    ("raven", "bird_raven", 4.8, 0.0),
    ("herring_gull", "bird_gull", 7.1, 0.0),
    ("golden_eagle", "bird_eagle", 7.5, 0.0),
    ("savanna_hawk", "bird_hawk", 5.7, 0.0),
    ("turkey_vulture", "bird_vulture", 7.1, 0.0),
    ("confuciusornis", "bird_mesozoic", 4.0, 0.0),
    ("gobipteryx", "bird_mesozoic", 3.6, 0.0),
];

/// The look named `name`.
pub fn look(name: &str) -> Option<&'static Look> {
    LOOKS.iter().find(|l| l.name == name)
}

/// Species `name`'s egg: its look, and the length it is drawn at in px (roe: its look's bead).
pub fn drawn(name: &str) -> Option<(&'static Look, f32)> {
    let &(_, l, cm, px) = EGGS.iter().find(|e| e.0 == name)?;
    let look = look(l)?;
    Some((look, if l.starts_with("roe_") { look.px } else if px > 0.0 { px } else { drawn_px(cm) }))
}

/// Species `name`'s egg: its look and the scale its model is drawn at (its drawn length against
/// the look's).
pub fn of(name: &str) -> Option<(&'static Look, f32)> {
    let (look, px) = drawn(name)?;
    Some((look, px / look.px))
}

/// Species `name`'s egg's real length in cm (its mass goes with it).
pub fn real_cm(name: &str) -> Option<f32> {
    EGGS.iter().find(|e| e.0 == name).map(|e| e.2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::species::SPECIES;

    /// Every species that lays has an egg with a look, and every look is laid by someone.
    #[test]
    fn every_layer_has_its_egg() {
        for s in SPECIES.iter().filter(|s| s.gestation() == 0) {
            assert!(of(s.name).is_some(), "{} lays eggs but has none in eggs::EGGS", s.name);
        }
        for l in LOOKS {
            assert!(EGGS.iter().any(|e| e.1 == l.name), "no species lays the {} egg", l.name);
        }
        for &(s, ..) in EGGS {
            assert!(SPECIES.iter().any(|x| x.name == s && x.gestation() == 0), "{s} in eggs::EGGS doesn't lay");
        }
    }

    /// A species' egg is drawn at its look's size give or take 15%: more would change the texels'
    /// size against the blocks' (`reports/Eggs in Minecraft mods.md`).
    #[test]
    fn eggs_keep_the_house_density() {
        for &(s, ..) in EGGS {
            let (look, k) = of(s).unwrap();
            assert!((0.85..=1.15).contains(&k), "{s}'s egg is drawn {k:.2}x its look {}: give it its own size", look.name);
        }
    }
}
