//! Skills and the perk web, for every mod (`vitals::skills`, `vitals::perks`; the base mod's
//! `skills.rs` and `web.rs` run them). A mod raises a skill by doing with `give_skill`, reads what
//! a player's web gives with `times` or `has`, and puts its own numbers through the chains below,
//! which the web multiplies.
//!
//! Always the player's own: a tribe shares knowledge and deeds, never these.

use weft_sdk::*;

pub use vitals::perks::Special;
pub use vitals::skills::Skill;
pub use vitals::web::Family;

value! {
    /// A player's skills and web: effort minutes earned in each skill (`vitals::skills`, the order
    /// of `SKILLS`), the web's nodes taken (a bit each by id, `vitals::perks::Taken`), node
    /// refunds left from unlearning stones, and the skill levels last announced. Kept through
    /// death.
    pub struct SkillWeb {
        pub em: [f32; vitals::skills::COUNT],
        pub taken: [u64; vitals::perks::WORDS],
        pub refunds: u8,
        pub shown: [u8; vitals::skills::COUNT],
    }
}

impl Default for SkillWeb {
    fn default() -> Self {
        SkillWeb { em: [0.0; vitals::skills::COUNT], taken: [0; vitals::perks::WORDS], refunds: 0, shown: [0; vitals::skills::COUNT] }
    }
}

impl SkillWeb {
    /// The level of a skill.
    pub fn level(&self, s: Skill) -> u32 {
        vitals::skills::level_of(self.em[s.index()]).0
    }
    /// Every skill's level.
    pub fn levels(&self) -> [u32; vitals::skills::COUNT] {
        vitals::perks::levels(&self.em)
    }
}

pub static SKILL_WEB: Component<SkillWeb> = Component::named("base:skill_web");

/// Taming progress a feeding or a knockout adds (`entity` the animal, `other` the tamer): the
/// Taming region's taming speed multiplies it.
pub static TAMING_SPEED: Chain = Chain::named("base:taming_speed");
/// How fast a player's food is used (`entity` the player; 1 the usual pace): the web's slower
/// hunger divides it.
pub static HUNGER_PACE: Chain = Chain::named("base:hunger_pace");
/// A player's max health and max stamina as food has them (`entity` the player): the web's
/// health and stamina multiply them.
pub static MAX_HEALTH: Chain = Chain::named("base:max_health");
pub static MAX_STAMINA: Chain = Chain::named("base:max_stamina");
/// The chance (0..) of one more of what a player harvests (`entity` the player, `state` the
/// crop) and of a second portion of what they take off a fire (`item` the food): whole numbers
/// are sure extras.
pub static HARVEST_EXTRA: Chain = Chain::named("base:harvest_extra");
pub static COOK_EXTRA: Chain = Chain::named("base:cook_extra");

/// Raise `player`'s skill by doing: `em` effort minutes of work at skill level `work`
/// (`vitals::skills::for_level` of its item or area level), less as it falls below their skill
/// (`vitals::skills::against`). Nothing in creative or spectator. The skill-up, its XP and its
/// toast follow on the base mod's next second.
pub fn give_skill(w: &World, player: u64, skill: Skill, em: f32, work: f32) {
    if em <= 0.0 || w.outside_survival(player) {
        return;
    }
    let Some(s) = w.component_mut(player, &SKILL_WEB) else { return };
    let i = skill.index();
    let level = vitals::skills::level_of(s.em[i]).0 as f32;
    s.em[i] += em * vitals::skills::against(work, level);
}

/// What `player`'s web multiplies a family by (1 without one).
pub fn times(w: &World, player: u64, f: Family) -> f32 {
    w.component(player, &SKILL_WEB)
        .map_or(1.0, |s| vitals::perks::times(&s.taken, f))
}

/// Whether `player`'s web has a special node (`Special::LeySight`).
pub fn has(w: &World, player: u64, special: Special) -> bool {
    w.component(player, &SKILL_WEB)
        .is_some_and(|s| vitals::perks::has_special(&s.taken, special))
}

/// The same for the local player, on the client.
pub fn has_here(view: &ClientView, special: Special) -> bool {
    view.component(&SKILL_WEB)
        .is_some_and(|s| vitals::perks::has_special(&s.taken, special))
}
