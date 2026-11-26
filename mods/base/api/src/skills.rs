//! Skills and the perk web, for every mod (`vitals::skills`, `vitals::perks`; the base mod's
//! `skills.rs` and `web.rs` run them). A mod raises a skill by doing with `give_skill`, reads what
//! a player's web gives with `times` or `has`, and puts its own numbers through the chains below,
//! which the web multiplies.
//!
//! Always the player's own: a tribe shares knowledge and deeds, never these.

use weft_sdk::*;

pub use vitals::perks::Special;
pub use vitals::skills::Skill;
pub use vitals::sockets::{Kind as StoneKind, Source as StoneSource, Stone};
pub use vitals::web::Family;

value! {
    /// A stone for the web's sockets (`vitals::sockets`), on its item stack and in a socket: its
    /// kind's code (0 none), item level, up to three boosts (`f0`-`f2` a family's index, 255
    /// none, `u0`-`u2` its units, `p0`-`p2` the same in whole percent for the tooltip,
    /// `tagged.base.stone`), and a flow-stone's work (`flow` 1 amplify by `share` (`sp` in
    /// percent), 2 turn to family `turn`). Fields one by one so the tooltip can name them.
    #[derive(Default)]
    pub struct StoneData {
        pub kind: u8,
        pub level: u8,
        pub f0: u8,
        pub f1: u8,
        pub f2: u8,
        pub u0: f32,
        pub u1: f32,
        pub u2: f32,
        pub p0: u32,
        pub p1: u32,
        pub p2: u32,
        pub flow: u8,
        pub turn: u8,
        pub share: f32,
        pub sp: u32,
    }
}

impl StoneData {
    pub fn of(st: &Stone) -> StoneData {
        let mut d = StoneData { kind: st.kind.code(), level: st.level, f0: 255, f1: 255, f2: 255, turn: 255, ..Default::default() };
        let roll =
            |i: usize| st.rolls[i].map_or((255, 0.0, 0), |(f, u)| (f.index() as u8, u, (u * vitals::web::UNIT * 100.0).round() as u32));
        (d.f0, d.u0, d.p0) = roll(0);
        (d.f1, d.u1, d.p1) = roll(1);
        (d.f2, d.u2, d.p2) = roll(2);
        match st.flow {
            Some(vitals::sockets::Flow::Amplify(x)) => (d.flow, d.share, d.sp) = (1, x, (x * 100.0).round() as u32),
            Some(vitals::sockets::Flow::Turn(f)) => (d.flow, d.turn) = (2, f.index() as u8),
            None => {}
        }
        d
    }

    pub fn stone(&self) -> Option<Stone> {
        let kind = StoneKind::of_code(self.kind)?;
        let fam = |i: u8| vitals::web::FAMILIES.get(i as usize).copied();
        let (fams, units) = ([self.f0, self.f1, self.f2], [self.u0, self.u1, self.u2]);
        let rolls = std::array::from_fn(|i| fam(fams[i]).map(|f| (f, units[i])));
        let flow = match self.flow {
            1 => Some(vitals::sockets::Flow::Amplify(self.share)),
            2 => fam(self.turn).map(vitals::sockets::Flow::Turn),
            _ => None,
        };
        Some(Stone { kind, level: self.level, rolls, flow })
    }
}

/// A stone's data on its item stack.
pub static STONE: Component<StoneData> = Component::named("base:stone");

/// A rolled stone of `kind` at item `level`, as its item (`base:<kind id>`).
pub fn stone_stack(w: &World, kind: StoneKind, level: f32) -> ItemStack {
    let mut rand = || (w.random() >> 40) as f32 / (1u64 << 24) as f32;
    let st = vitals::sockets::roll(kind, level, &mut rand);
    let Some(item) = Item::lookup(&format!("base:{}", kind.id())) else { return ItemStack::EMPTY };
    ItemStack::new(item, 1).with(&STONE, StoneData::of(&st))
}

/// A drop table's entry: a find at `source` in tier `tier` (0 Stone Age .. 6), by its chance
/// (`vitals::sockets::SOURCES`), dropped at `pos`. A boss calls it on a kill, a ley node's heart
/// when it's opened, a ruin's vault, a rare animal's death; true if a stone fell.
pub fn drop_stone(w: &World, source: StoneSource, tier: u8, pos: [f64; 3]) -> bool {
    let mut rand = || (w.random() >> 40) as f32 / (1u64 << 24) as f32;
    let Some(st) = vitals::sockets::find(source, tier, &mut rand) else { return false };
    let Some(item) = Item::lookup(&format!("base:{}", st.kind.id())) else { return false };
    w.spawn_item(pos, ItemStack::new(item, 1).with(&STONE, StoneData::of(&st)));
    true
}

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
        /// The stones in the sockets, by socket index (`vitals::perks::Node::socket`).
        pub stones: [StoneData; vitals::perks::SOCKETS],
    }
}

impl Default for SkillWeb {
    fn default() -> Self {
        SkillWeb {
            em: [0.0; vitals::skills::COUNT],
            taken: [0; vitals::perks::WORDS],
            refunds: 0,
            shown: [0; vitals::skills::COUNT],
            stones: [StoneData::default(); vitals::perks::SOCKETS],
        }
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
    /// The stones in its sockets.
    pub fn stones(&self) -> vitals::perks::Stones {
        std::array::from_fn(|i| self.stones[i].stone())
    }
    /// What the web and its stones multiply a family by.
    pub fn times(&self, f: Family) -> f32 {
        vitals::perks::times_with(&self.taken, &self.stones(), f)
    }
    /// Whether the web or a relic in it gives a special.
    pub fn has(&self, s: Special) -> bool {
        vitals::perks::has_special_with(&self.taken, &self.stones(), s)
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
    w.component(player, &SKILL_WEB).map_or(1.0, |s| s.times(f))
}

/// Whether `player`'s web has a special node (`Special::LeySight`).
pub fn has(w: &World, player: u64, special: Special) -> bool {
    w.component(player, &SKILL_WEB)
        .is_some_and(|s| s.has(special))
}

/// The same for the local player, on the client.
pub fn has_here(view: &ClientView, special: Special) -> bool {
    view.component(&SKILL_WEB).is_some_and(|s| s.has(special))
}
