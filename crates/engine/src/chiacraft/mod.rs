//! chiacraft's own parts of the engine, kept apart from Weft's so merging Weft stays easy
//! (`notes/PLAN.md`, rule 4; `notes/OVERVIEW.md` for the pieces): the account screen, trust in
//! signed packages, linked worlds, and the bridge that answers what mods ask beyond Weft's SDK.
//! The crate re-exports each module under its old name, so Weft's files that call them stay as
//! they were.

pub mod account_screen;
pub mod achievements;
pub mod achievements_screen;
pub mod bridge;
pub mod chain;
pub mod did;
pub mod package_trust;
pub mod worlds;

/// Which plural form a count takes in a language (`en_us`, `ru_ru`, `fr_fr`): `one`, `few` or
/// `other`, the suffix of the lang key a screen picks (`achievements.count.one`). Only for text a
/// screen formats in the player's own language; Weft's templates have no plurals.
pub fn plural(language: &str, n: i64) -> &'static str {
    let n = n.unsigned_abs();
    match language.split('_').next().unwrap_or("") {
        "ru" => match (n % 10, n % 100) {
            (1, m) if m != 11 => "one",
            (2..=4, m) if !(12..=14).contains(&m) => "few",
            _ => "other",
        },
        "fr" if n <= 1 => "one",
        "fr" => "other",
        _ if n == 1 => "one",
        _ => "other",
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn plural_forms() {
        use super::plural;
        assert_eq!([0, 1, 2, 5].map(|n| plural("en_us", n)), ["other", "one", "other", "other"]);
        assert_eq!([0, 1, 2].map(|n| plural("fr_fr", n)), ["one", "one", "other"]);
        assert_eq!(
            [1, 2, 5, 11, 12, 21, 22, 25, 111].map(|n| plural("ru_ru", n)),
            ["one", "few", "other", "other", "other", "one", "few", "other", "other"]
        );
    }
}
