//! Gems in gear (`crafting::gems`; Ronald, 11-05: "pieces and items should also get like gem
//! slots because that will make it even cooler"):
//! - **Gem items** `base:gem_<id>`, each carrying its roll as the web's stones do
//!   (`base_api::skills::StoneData`, kind 0: never a web stone) on its own component, `GEM`.
//! - **Cutting:** a found stone (`#gem_stones/<id>`) into its gem from the book's Parts tab at a
//!   crafting table; the roll is `vitals::sockets::roll_among(GEAR_FAMILIES)` at the gem's level.
//! - **On a built item:** `GEMS` holds its sockets and the gems in them; `parts.rs` builds the
//!   item's numbers with their boosts (`crafting::gems::multipliers`).
//! - **Socketing** at the table's assembly row: a built item and gems make the item with them set.
//!   **Taking out:** the item alone there makes it bare, and the gems come back to the bag (a
//!   cheap one cracks one time in `CRACK_ONE_IN`); taking it apart gives them back the same way.

use base_api::skills::StoneData;
use crafting::gems::{CRACK_ONE_IN, GEMS, Gem};
use vitals::web::{FAMILIES, Family};
use weft_sdk::*;

/// A cut gem's roll, on its item.
pub static GEM: Component<StoneData> = Component::named("base:gem");

value! {
    /// A built item's sockets and the gems set in them (up to three): each gem's kind
    /// (`crafting::code` of its id), item level, and its boosts as `StoneData` does (`f*` a family's
    /// index, 255 none; `u*` units). `n` filled of `sockets`.
    pub struct GemsOf {
        pub sockets: u32,
        pub n: u32,
        pub gem: [u32; 3],
        pub level: [u32; 3],
        pub f: [u32; 9],
        pub u: [f32; 9],
    }
}
pub static GEMS_ON: Component<GemsOf> = Component::named("base:gems");
/// The gem a cutting recipe makes (a recipe value: its gem's code).
pub static GEM_CUT: Component<u32> = Component::named("base:gem_cut");

/// One gem as it sits on an item or its stack: which, its level, its boosts.
#[derive(Clone, Copy, Debug)]
pub struct Set {
    pub gem: &'static Gem,
    pub data: StoneData,
}

impl Set {
    pub fn boosts(&self) -> Vec<(Family, f32)> {
        let d = &self.data;
        [(d.f0, d.u0), (d.f1, d.u1), (d.f2, d.u2)]
            .into_iter()
            .filter_map(|(f, u)| FAMILIES.get(f as usize).map(|f| (*f, u)))
            .collect()
    }
}

fn gem_of(code: u32) -> Option<&'static Gem> {
    GEMS.iter().find(|g| crafting::code(g.id) == code)
}

/// The gem a stack is, if it is one.
pub fn loose(stack: &ItemStack) -> Option<Set> {
    let data = GEM.on_stack_own(stack)?;
    let name = item_name(stack.item);
    let gem = GEMS.iter().find(|g| name == format!("base:{}", g.item()))?;
    Some(Set { gem, data })
}

/// A gem's item with its roll.
pub fn stack_of(set: &Set) -> ItemStack {
    let Some(item) = Item::lookup(&format!("base:{}", set.gem.item())) else { return ItemStack::EMPTY };
    ItemStack::new(item, 1).with(&GEM, set.data)
}

/// A freshly cut gem of `gem`, rolled by `rand`.
pub fn cut(gem: &'static Gem, rand: impl FnMut() -> f32) -> Set {
    let rolls = vitals::sockets::roll_among(&vitals::sockets::GEAR_FAMILIES, gem.level(), rand);
    let st = vitals::sockets::Stone { kind: vitals::sockets::Kind::Mind, level: gem.level().round() as u8, rolls, flow: None };
    // The web's stone data, kind 0: a gem, never a web stone.
    Set { gem, data: StoneData { kind: 0, ..StoneData::of(&st) } }
}

/// The gems set in a built item.
pub fn on(stack: &ItemStack) -> Vec<Set> {
    let Some(g) = GEMS_ON.on_stack_own(stack) else { return Vec::new() };
    (0..g.n.min(3) as usize)
        .filter_map(|i| {
            let gem = gem_of(g.gem[i])?;
            let f = |k: usize| g.f[i * 3 + k] as u8;
            let u = |k: usize| g.u[i * 3 + k];
            let p = |k: usize| (u(k) * vitals::web::UNIT * 100.0).round() as u32;
            let data = StoneData {
                level: g.level[i] as u8,
                f0: f(0),
                f1: f(1),
                f2: f(2),
                u0: u(0),
                u1: u(1),
                u2: u(2),
                p0: p(0),
                p1: p(1),
                p2: p(2),
                turn: 255,
                ..Default::default()
            };
            Some(Set { gem, data })
        })
        .collect()
}

/// The component for an item with `sockets` sockets and `sets` in them.
pub fn gems_of(sockets: u8, sets: &[Set]) -> GemsOf {
    let mut g = GemsOf { sockets: sockets as u32, n: 0, gem: [0; 3], level: [0; 3], f: [255; 9], u: [0.0; 9] };
    for (i, s) in sets.iter().take(sockets.min(3) as usize).enumerate() {
        g.gem[i] = crafting::code(s.gem.id);
        g.level[i] = s.data.level as u32;
        let d = &s.data;
        for (k, (f, u)) in [(d.f0, d.u0), (d.f1, d.u1), (d.f2, d.u2)]
            .into_iter()
            .enumerate()
        {
            g.f[i * 3 + k] = f as u32;
            g.u[i * 3 + k] = u;
        }
        g.n = i as u32 + 1;
    }
    g
}

/// Gives `sets` back to `player` (the bag, or at the feet): each cheap one cracks one time in
/// `CRACK_ONE_IN` (`crafting::gems`), and the player is told.
pub fn give_back(w: &World, player: u64, sets: &[Set]) {
    for s in sets {
        if s.gem.cracks() && w.random().is_multiple_of(CRACK_ONE_IN as u64) {
            w.chat(player, "gem.base.cracked", &[Arg::Key(s.gem.key)]);
            continue;
        }
        let left = w.inventory(player).insert(stack_of(s));
        if !left.is_empty()
            && let Some(e) = w.entity(player)
        {
            w.spawn_item(e.pos, left);
        }
    }
}

pub fn register(r: &mut Registry) {
    r.stack_component(&GEM, 0);
    r.stack_component(&GEMS_ON, 0);
    for g in GEMS {
        r.item(Item::generated(g.item()))
            .tag("gems")
            .tab(&crate::tabs::INGREDIENTS);
        // Cut at a crafting table from the book's Parts tab (`book.rs`, `parts::finish`).
        r.shapeless(&crate::parts::PART_RECIPES, &g.item(), 1, &[&format!("#gem_stones/{}", g.id)], 0)
            .value(&GEM_CUT, crafting::code(g.id));
    }
    // Base's own found stones; another mod tags its own (the animals mod's amber).
    for (id, item) in [("diamond", "diamond"), ("peridot", "peridot"), ("tektite", "tektite")] {
        r.tag(&format!("gem_stones/{id}"), &[item]);
    }
}

/// A cutting recipe's result: its gem, rolled with `w`'s randomness.
pub fn finish(w: &World, r: &Recipe, out: ItemStack) -> Option<ItemStack> {
    let gem = gem_of(r.value(&GEM_CUT)?)?;
    let set = cut(gem, || (w.random() >> 40) as f32 / (1u64 << 24) as f32);
    Some(out.with(&GEM, set.data))
}
