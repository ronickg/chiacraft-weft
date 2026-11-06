//! Eggs by species: which look each laying species' egg has and how long it is, so a raptor's
//! egg reads as a raptor's at a glance (Ronald, 2026-11-06: "I don't want them to all look the
//! same"). The evidence behind each, species by species, is `reports/Eggs by species.md`; the
//! looks are built by `mods/animals/source/egg/build.py` (a model `egg_<look>` or a shared roe
//! model, a texture `egg/<look>`, an item icon `egg_<look>`).

/// One look: the families of eggs that read alike, and the length its model is built at (cm),
/// which a species' own egg length scales.
pub struct Look {
    pub name: &'static str,
    /// The model (`assets/models/<it>.geo.json`): its own, or one shared with other roe.
    pub model: &'static str,
    /// The egg's length as built, in cm (a roe clutch: 1, never scaled).
    pub built_cm: f32,
}

pub const LOOKS: &[Look] = &[
    // Long ovals, a little narrower at the bottom, pale sky blue-green with brown blotches and
    // speckles, stood nearly upright (Deinonychus's pigments, Wiemann et al. 2018).
    Look { name: "dromaeosaur", model: "egg_dromaeosaur", built_cm: 9.0 },
    // The longest, ends alike, dark teal with lengthwise ridges, lying inclined (Citipati, 18 cm).
    Look { name: "oviraptorid", model: "egg_oviraptorid", built_cm: 18.0 },
    // Soft-shelled, pale buff, dented, buried lying (Protoceratops 10.1 cm, Norell et al. 2020).
    Look { name: "ceratopsian", model: "egg_ceratopsian", built_cm: 10.1 },
    // Our pick, no ankylosaur egg known: near round, leathery, dull olive-grey, mottled.
    Look { name: "ankylosaur", model: "egg_ankylosaur", built_cm: 12.0 },
    // Big, near round, warm ivory, a fine net of bumps (hadrosaur-like spheroolithid eggs).
    Look { name: "ornithopod", model: "egg_ornithopod", built_cm: 16.0 },
    // A ball, chalk white, coarse knobs (titanosaur eggs of Auca Mahuevo, about 13 cm).
    Look { name: "sauropod", model: "egg_sauropod", built_cm: 13.0 },
    // Small, both ends alike, a chalky band round the middle, lying (Caiman yacare, 6.8 cm).
    Look { name: "crocodilian", model: "egg_crocodilian", built_cm: 6.8 },
    // Tiny, white, matt, dented (drawn about twice an agamid's real 1.6 cm, to be seen).
    Look { name: "lizard", model: "egg_lizard", built_cm: 3.0 },
    // Fish: a clutch of beads, each about three times its real size, told apart by color and lay.
    Look { name: "roe_trout", model: "egg_roe", built_cm: 1.0 },
    Look { name: "roe_pacu", model: "egg_roe_loose", built_cm: 1.0 },
    Look { name: "roe_lycoptera", model: "egg_roe_loose", built_cm: 1.0 },
    Look { name: "roe_lungfish", model: "egg_jelly", built_cm: 1.0 },
    // Fantasy: big, stone-heavy, in metal-rimmed jewel scales; our own design.
    Look { name: "wyvern", model: "egg_wyvern", built_cm: 50.0 },
    // Birds, lying in the nest's cup: a shape per group, a coloring per species.
    // Owls: plain white, nearly round.
    Look { name: "bird_owl", model: "egg_bird_round", built_cm: 5.6 },
    // Songbirds (drawn half again their real 2.2 cm): grey-white densely speckled, and bluish
    // white with red-brown spots gathered at the blunt end.
    Look { name: "bird_sparrow", model: "egg_bird_small", built_cm: 3.3 },
    Look { name: "bird_bunting", model: "egg_bird_small", built_cm: 3.3 },
    // Blotched ovals: the raven's blue-green, the gull's olive-brown, the eagle's red-brown on
    // white, the hawk's bluish white, the vulture's cream wreathed dark brown at the blunt end.
    Look { name: "bird_raven", model: "egg_bird", built_cm: 5.0 },
    Look { name: "bird_gull", model: "egg_bird", built_cm: 5.0 },
    Look { name: "bird_eagle", model: "egg_bird", built_cm: 5.0 },
    Look { name: "bird_hawk", model: "egg_bird", built_cm: 5.0 },
    Look { name: "bird_vulture", model: "egg_bird", built_cm: 5.0 },
    // The Cretaceous birds: long, ends nearly alike, plain pale (Gobipteryx's Gobioolithus).
    Look { name: "bird_mesozoic", model: "egg_bird_long", built_cm: 3.6 },
];

/// Each laying species (`species::SPECIES` name): its egg's look and length in cm (a roe clutch:
/// 1). Guesses are marked in the report; sizes there.
pub const EGGS: &[(&str, &str, f32)] = &[
    ("raptor", "dromaeosaur", 9.0),
    ("osmolskae", "dromaeosaur", 9.0),
    // Utahraptor's egg is a guess scaled from its mass (report: about 18 x 8 cm).
    ("utahraptor", "dromaeosaur", 18.0),
    ("citipati", "oviraptorid", 18.0),
    ("protoceratops", "ceratopsian", 10.1),
    // No Psittacosaurus egg is known: the same soft egg, smaller (report: about 7 cm).
    ("psittacosaurus", "ceratopsian", 7.0),
    ("gastonia", "ankylosaur", 12.0),
    ("pinacosaurus", "ankylosaur", 12.0),
    ("iguanodont", "ornithopod", 16.0),
    ("cedarosaurus", "sauropod", 13.0),
    ("yacare_caiman", "crocodilian", 6.8),
    ("spectacled_caiman", "crocodilian", 6.4),
    ("crocodyliform", "crocodilian", 7.0),
    ("priscagama", "lizard", 3.0),
    ("brown_trout", "roe_trout", 1.0),
    ("pacu", "roe_pacu", 1.0),
    ("lycoptera", "roe_lycoptera", 1.0),
    ("lungfish", "roe_lungfish", 1.0),
    ("wyvern", "wyvern", 50.0),
    ("great_horned_owl", "bird_owl", 5.6),
    ("snowy_owl", "bird_owl", 5.7),
    ("house_sparrow", "bird_sparrow", 3.3),
    ("snow_bunting", "bird_bunting", 3.3),
    ("raven", "bird_raven", 5.0),
    ("herring_gull", "bird_gull", 7.1),
    ("golden_eagle", "bird_eagle", 7.5),
    ("savanna_hawk", "bird_hawk", 5.8),
    ("turkey_vulture", "bird_vulture", 7.1),
    ("confuciusornis", "bird_mesozoic", 3.6),
    ("gobipteryx", "bird_mesozoic", 3.0),
];

/// The look named `name`.
pub fn look(name: &str) -> Option<&'static Look> {
    LOOKS.iter().find(|l| l.name == name)
}

/// Species `name`'s egg: its look and the scale its model is drawn at (its egg's length against
/// the look's built length).
pub fn of(name: &str) -> Option<(&'static Look, f32)> {
    let &(_, l, cm) = EGGS.iter().find(|(s, _, _)| *s == name)?;
    let look = look(l)?;
    Some((look, cm / look.built_cm))
}

/// How much longer species `name`'s egg is than a Velociraptor's (a nest grows with it; 1 for
/// roe and anything smaller).
pub fn against_raptor(name: &str) -> f32 {
    let raptor = EGGS[0].2;
    EGGS.iter()
        .find(|(s, _, _)| *s == name)
        .map_or(1.0, |&(_, l, cm)| if l.starts_with("roe_") { 1.0 } else { cm / raptor })
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
            assert!(EGGS.iter().any(|(_, k, _)| *k == l.name), "no species lays the {} egg", l.name);
        }
        for (s, _, _) in EGGS {
            assert!(SPECIES.iter().any(|x| x.name == *s && x.gestation() == 0), "{s} in eggs::EGGS doesn't lay");
        }
    }
}
