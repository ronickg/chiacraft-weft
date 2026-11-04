//! Liquids flow in the engine, following modern vanilla's FlowingFluid step by step: a liquid
//! block reacts to neighbor changes by scheduling a tick after its delay; a tick recomputes the
//! level from the neighbors, falls if it can and otherwise spreads sideways only towards the
//! nearest way down within `slope_find`. Mods describe liquids and reactions as data
//! (`abi::LiquidDesc`, `abi::ReactionDesc`), nothing calls into a mod per update. A block that
//! stands in a liquid (`StateInfo::holds`) is a still source of it here: it spreads like one,
//! nothing flows into it, and the liquid stays when the block goes (`Content::left`).

use glam::{DVec3, IVec3};
use weft_abi::{ItemStack, block_flags as bf};

use super::dim::Dim;
use crate::content::{AIR, Content, LiquidDef};
use crate::world::DIRS;

/// North, east, south, west: vanilla's horizontal order, which decides ties.
const HORIZONTAL: [IVec3; 4] = [IVec3::new(0, 0, -1), IVec3::new(1, 0, 0), IVec3::new(0, 0, 1), IVec3::new(-1, 0, 0)];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Fluid {
    id: u16,
    /// 1..8, 8 for sources and falling liquid.
    amount: u32,
    source: bool,
    falling: bool,
}

fn decode(c: &Content, state: u32) -> Option<Fluid> {
    let st = &c.states[state as usize];
    if let Some(id) = st.holds {
        return Some(Fluid { id, amount: 8, source: true, falling: false });
    }
    let id = st.liquid?;
    let i = state - c.liquids[id as usize].first_state;
    let (source, falling) = (i == 0, i >= 8);
    Some(Fluid { id, amount: if source || falling { 8 } else { 8 - i }, source, falling })
}

fn encode(def: &LiquidDef, amount: u32, falling: bool) -> u32 {
    if falling { def.first_state + 8 } else { def.first_state + 8 - amount.clamp(1, 8) }
}

impl Dim {
    fn fluid(&self, c: &Content, p: IVec3) -> Option<Fluid> {
        self.get(p).and_then(|s| decode(c, s))
    }

    /// Room for liquid: neither solid nor a ladder (unloaded counts as solid).
    fn holds_liquid(&self, c: &Content, p: IVec3) -> bool {
        self.get(p)
            .is_some_and(|s| c.states[s as usize].flags & (bf::SOLID | bf::CLIMBABLE) == 0)
    }

    fn solid(&self, c: &Content, p: IVec3) -> bool {
        self.get(p)
            .is_none_or(|s| c.states[s as usize].flags & bf::SOLID != 0)
    }

    /// What liquid `l` would make of `p` from its neighbors: a state, or `None` for nothing.
    fn new_liquid(&self, c: &Content, p: IVec3, l: u16, down: IVec3) -> Option<u32> {
        let def = &c.liquids[l as usize];
        let (mut max, mut sources) = (0, 0);
        for d in HORIZONTAL {
            if let Some(f) = self.fluid(c, p + d).filter(|f| f.id == l) {
                sources += f.source as u32;
                max = max.max(f.amount);
            }
        }
        if sources >= def.renewable {
            let under = self.fluid(c, p + down);
            if self.solid(c, p + down) || under.is_some_and(|f| f.id == l && f.source) {
                return Some(def.first_state);
            }
        }
        if self.fluid(c, p - down).is_some_and(|f| f.id == l) {
            return Some(encode(def, 8, true));
        }
        let k = max as i32 - def.drop_off as i32;
        (k > 0).then(|| encode(def, k as u32, false))
    }

    fn is_hole(&self, c: &Content, below: IVec3, l: u16) -> bool {
        self.fluid(c, below).is_some_and(|f| f.id == l) || self.holds_liquid(c, below)
    }

    fn passes(&self, c: &Content, p: IVec3, l: u16) -> bool {
        !self.fluid(c, p).is_some_and(|f| f.id == l && f.source) && self.holds_liquid(c, p)
    }

    /// Whether `l` may flow into `p` from direction `dir`: into empty room, or down into another
    /// liquid it reacts with.
    fn can_spread_to(&self, c: &Content, p: IVec3, l: u16, falling_in: bool) -> bool {
        match self.fluid(c, p) {
            Some(f) if f.id == l => false,
            Some(_) => falling_in && self.falls_into(c, l, p).is_some(),
            None => self.holds_liquid(c, p),
        }
    }

    fn falls_into(&self, c: &Content, l: u16, p: IVec3) -> Option<u32> {
        let state = self.get(p)?;
        let mine = c.liquids[l as usize].first_state;
        c.reactions
            .iter()
            .find(|r| r.falls_into.is_some() && c.matches(&r.liquid, mine) && c.cell_matches(&r.touching, state))
            .and_then(|r| r.falls_into)
    }

    fn spread_to(&mut self, c: &'static Content, p: IVec3, state: u32, falling_in: bool, l: u16) {
        let Some(old) = self.get(p) else { return };
        if c.states[old as usize].liquid_in().is_some() {
            if falling_in {
                if let Some(b) = self.falls_into(c, l, p) {
                    // A block standing in what turns goes with it.
                    if c.states[old as usize].holds.is_some() {
                        self.drop_block_loot(c, p, old, ItemStack::EMPTY, false, None);
                    }
                    self.set(c, p, b);
                }
            }
            return;
        }
        self.set(c, p, state);
        if old != AIR {
            // Plants and torches wash away with their drops, like vanilla.
            self.drop_block_loot(c, p, old, ItemStack::EMPTY, false, None);
        }
    }

    fn slope_distance(&self, c: &Content, p: IVec3, depth: u32, from: IVec3, l: u16, down: IVec3) -> u32 {
        let mut min = 1000;
        for d in HORIZONTAL {
            if d == from {
                continue;
            }
            let n = p + d;
            if self.passes(c, n, l) {
                if self.is_hole(c, n + down, l) {
                    return depth;
                }
                if depth < c.liquids[l as usize].slope_find {
                    min = min.min(self.slope_distance(c, n, depth + 1, -d, l, down));
                }
            }
        }
        min
    }

    /// Directions to spread in: the ones nearest to a way down, each with the state it gets.
    fn spread_dirs(&self, c: &Content, p: IVec3, l: u16, down: IVec3) -> Vec<(IVec3, u32)> {
        let mut min = 1000;
        let mut out = Vec::new();
        for d in HORIZONTAL {
            let n = p + d;
            if !self.passes(c, n, l) {
                continue;
            }
            let Some(state) = self.new_liquid(c, n, l, down) else { continue };
            let slope = if self.is_hole(c, n + down, l) { 0 } else { self.slope_distance(c, n, 1, -d, l, down) };
            if slope < min {
                out.clear();
            }
            if slope <= min {
                out.push((d, state));
                min = slope;
            }
        }
        out
    }

    fn spread_sides(&mut self, c: &'static Content, p: IVec3, f: Fluid, down: IVec3) {
        let def = &c.liquids[f.id as usize];
        let amount = if f.falling { 7 } else { f.amount as i32 - def.drop_off as i32 };
        if amount <= 0 {
            return;
        }
        for (d, state) in self.spread_dirs(c, p, f.id, down) {
            if self.can_spread_to(c, p + d, f.id, false) {
                self.spread_to(c, p + d, state, false, f.id);
            }
        }
    }

    /// The scheduled tick of a liquid block.
    pub(super) fn liquid_tick(&mut self, c: &'static Content, p: IVec3, state: u32) {
        let Some(mut f) = decode(c, state) else { return };
        let def = &c.liquids[f.id as usize];
        let down = if def.rises { IVec3::Y } else { IVec3::NEG_Y };
        if !f.source {
            match self.new_liquid(c, p, f.id, down) {
                None => {
                    self.set(c, p, AIR);
                    return;
                }
                Some(s) if s != state => {
                    self.set(c, p, s);
                    f = decode(c, s).unwrap_or(f);
                }
                _ => {}
            }
        }
        let below = p + down;
        if self.can_spread_to(c, below, f.id, true) {
            let state = self
                .new_liquid(c, below, f.id, down)
                .unwrap_or(encode(def, 8, true));
            self.spread_to(c, below, state, true, f.id);
            let sources = HORIZONTAL
                .iter()
                .filter(|&&d| {
                    self.fluid(c, p + d)
                        .is_some_and(|n| n.id == f.id && n.source)
                })
                .count();
            if sources >= 3 {
                self.spread_sides(c, p, f, down);
            }
        } else if f.source || !self.is_hole(c, below, f.id) {
            self.spread_sides(c, p, f, down);
        }
    }

    /// A neighbor of a liquid changed: react with what touches it, else flow after the delay.
    pub(super) fn liquid_neighbor(&mut self, c: &'static Content, p: IVec3, state: u32) {
        let Some(f) = decode(c, state) else { return };
        let def = &c.liquids[f.id as usize];
        let down = if def.rises { IVec3::Y } else { IVec3::NEG_Y };
        // A block standing in it reacts as its liquid's source would.
        let me = if c.states[state as usize].holds.is_some() { def.first_state } else { state };
        for d in DIRS {
            // What lies in the direction it flows does not react with it (lava over water).
            if d == down {
                continue;
            }
            let Some(n) = self.get(p + d) else { continue };
            let hit = c
                .reactions
                .iter()
                .find(|r| c.matches(&r.liquid, me) && c.cell_matches(&r.touching, n));
            if let Some(b) = hit.and_then(|r| if f.source { r.source } else { r.flowing }) {
                if me != state {
                    self.drop_block_loot(c, p, state, ItemStack::EMPTY, false, None);
                }
                self.set(c, p, b);
                return;
            }
        }
        self.schedule(p, def.delay);
    }

    /// Bucket logic for `player` holding `held`: pour a liquid item, or fill an empty container
    /// from a source it looks at. The new held stack, or `None` if nothing happened.
    pub fn use_container(&mut self, c: &'static Content, player: u64, held: ItemStack) -> Option<ItemStack> {
        if held.is_empty() {
            return None;
        }
        let creative = self
            .entities
            .get(player)
            .is_some_and(|e| e.mode == crate::world::GameMode::Creative);
        if let Some(&(l, empty)) = c.item_liquid.get(&held.item) {
            let reach = self.attribute(c, player, super::extend::BLOCK_REACH, false);
            let hit = self
                .look_ray(c, player, reach, false)
                .filter(|h| h.frame == 0)?;
            let replaceable = |s: u32| c.states[s as usize].flags & bf::REPLACEABLE != 0;
            // Into a block that can stand in it (a slab), else into the cell it looks at or before.
            let fills = c.filled.get(&(hit.state, l)).copied();
            let target = if replaceable(hit.state) || fills.is_some() { hit.block } else { hit.block + DIRS[hit.face as usize] };
            let old = self
                .get(target)
                .filter(|&s| fills.is_some() || replaceable(s))?;
            let source = fills.unwrap_or(c.liquids[l as usize].first_state);
            // Only into what would become a source, and with the same say as placing a block.
            if c.still_liquid(old) == Some(l) {
                return None;
            }
            let mut q =
                weft_abi::Query { entity: player, pos: target.to_array(), state: source, item: held.item, ..weft_abi::Query::DEFAULT };
            if !self.ask(c, weft_abi::question::PLACE_BLOCK, &mut q) {
                return None;
            }
            if c.states[old as usize].liquid.is_some() || fills.is_some() {
                self.set(c, target, source);
            } else {
                self.spread_to(c, target, source, false, l);
            }
            return Some(if creative || empty == 0 { held } else { ItemStack::new(empty, 1) });
        }
        let reach = self.attribute(c, player, super::extend::BLOCK_REACH, false);
        let hit = self
            .look_ray(c, player, reach, true)
            .filter(|h| h.frame == 0)?;
        let f = decode(c, hit.state).filter(|f| f.source)?;
        let filled = *c.containers.get(&(held.item, f.id))?;
        // A block standing in it gives it up only if it can be dry.
        let after = if c.states[hit.state as usize].holds.is_some() { *c.drained.get(&hit.state)? } else { AIR };
        let mut q =
            weft_abi::Query { entity: player, pos: hit.block.to_array(), state: hit.state, item: held.item, ..weft_abi::Query::DEFAULT };
        if !self.ask(c, weft_abi::question::BREAK_BLOCK, &mut q) {
            return None;
        }
        self.set(c, hit.block, after);
        if creative {
            return Some(held);
        }
        if held.count == 1 {
            return Some(ItemStack::new(filled, 1));
        }
        let rest = self
            .entities
            .insert_item(c, player, ItemStack::new(filled, 1));
        if !rest.is_empty() {
            let pos = self.entities.get(player).map_or(DVec3::ZERO, |e| e.pos);
            self.spawn_item(c, pos, rest);
        }
        Some(ItemStack { count: held.count - 1, ..held })
    }

    /// The block an entity looks at from its eyes, in the world or a moving structure.
    pub fn look_ray(&self, c: &Content, id: u64, max: f64, liquids: bool) -> Option<crate::physics::Hit> {
        let e = self.entities.get(id)?;
        let eye = e.pos + DVec3::Y * e.eye(c);
        let (sy, cy) = (e.yaw as f64).sin_cos();
        let (sp, cp) = (e.pitch as f64).sin_cos();
        let dir = DVec3::new(-sy * cp, sp, -cy * cp);
        let structures: Vec<(u64, crate::physics::Transform, &dyn crate::physics::Blocks)> = self
            .bodies
            .iter()
            .filter_map(|(&sid, s)| {
                self.entities
                    .get(sid)
                    .map(|b| (sid, crate::physics::Transform::of(b.pos), &**s as &dyn crate::physics::Blocks))
            })
            .collect();
        // Asked from a body's blocks, it looks along those.
        if self.frame != 0 {
            return crate::physics::pick(c, &|p| self.get(p), &[], self.place.to_local(eye), dir, max, liquids);
        }
        crate::physics::pick(c, &|p| self.get(p), &structures, eye, dir, max, liquids)
    }
}

/// Liquids from this hot burn what touches them.
pub const BURNING: u32 = 1000;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;

    use glam::{DVec3, IVec3};
    use weft_abi::{self as abi, ItemStack, Layer, ModId, Model, block_flags as bf};

    use super::Dim;
    use crate::components::RawValue;
    use crate::content::{AIR, Content, ModMeta, RawBlock, RawRegistrar, RawSignal, RawState};
    use crate::world::cube::Cube;

    fn state(flags: u32, opacity: u8, model: Model, components: Vec<RawValue>) -> RawState {
        RawState {
            flags,
            filter: [0; 3],
            opacity,
            layer: if opacity == 15 { Layer::Opaque } else { Layer::Cutout },
            model,
            textures: std::array::from_fn(|_| "t:block/x".to_string()),
            boxes: Vec::new(),
            collision: Vec::new(),
            selection: Vec::new(),
            signal: RawSignal { kind: abi::SignalDesc::DEFAULT.kind, attach: 6, front: 6, delay: 0, mode: 0, head: String::new() },
            components,
        }
    }

    fn block(name: &str, states: Vec<RawState>, props: Vec<(String, Vec<String>)>) -> RawBlock {
        RawBlock {
            name: name.into(),
            cell: 0,
            states,
            props,
            components: Vec::new(),
            cb: Default::default(),
            replace: false,
            own_states: true,
            retexture: Vec::new(),
            opens: String::new(),
            extra_cells: Vec::new(),
        }
    }

    fn holds_water() -> RawValue {
        RawValue { name: abi::comp::HOLDS_LIQUID.into(), schema: "block".into(), bytes: Vec::new(), text: Some("t:water".into()) }
    }

    /// Stone, water, kelp (always in water, held up by what is under it) and a slab that can be
    /// waterlogged, in an editable world with block updates.
    fn content() -> &'static Content {
        let mods =
            vec![ModMeta { id: "t".into(), version: "1".into(), deps: Vec::new(), order: 0, assets: "/nonexistent".into(), package: None }];
        let mut r = RawRegistrar::new(ModId(0));
        r.blocks
            .push(block("t:stone", vec![state(bf::SOLID | bf::OPAQUE, 15, Model::Cube, Vec::new())], Vec::new()));
        r.blocks.push(block(
            "t:water",
            (0..16)
                .map(|_| state(0, 2, Model::Cube, Vec::new()))
                .collect(),
            vec![("level".into(), (0..16).map(|v| v.to_string()).collect())],
        ));
        let support = RawValue { name: abi::comp::SUPPORT_FACE.into(), schema: "u8".into(), bytes: vec![2], text: None };
        r.blocks
            .push(block("t:kelp", vec![state(bf::REPLACEABLE, 0, Model::Cross, vec![holds_water(), support])], Vec::new()));
        r.blocks.push(block(
            "t:slab",
            vec![state(bf::SOLID, 0, Model::Cube, Vec::new()), state(bf::SOLID, 0, Model::Cube, vec![holds_water()])],
            vec![("waterlogged".into(), vec!["false".into(), "true".into()])],
        ));
        r.game
            .liquids
            .push(("t:water".into(), String::new(), abi::LiquidDesc { renewable: 2, ..abi::LiquidDesc::DEFAULT }));
        let mut c = crate::content::freeze(mods, vec![r]).unwrap_or_else(|e| panic!("{e:?}"));
        let void = c.void_dim as usize;
        c.dims[void].editable = true;
        c.dims[void].block_updates = true;
        Box::leak(Box::new(c))
    }

    fn id(c: &Content, name: &str) -> u32 {
        c.parse_state(name).unwrap()
    }

    /// Stone below y = 0, air above, three cubes each way.
    fn world(c: &'static Content) -> Dim {
        let mut d = Dim::new(c.void_dim, 1, Arc::new(AtomicU64::new(1)));
        let stone = id(c, "t:stone");
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let fill = if y < 0 { stone } else { AIR };
                    d.space
                        .cubes
                        .insert(IVec3::new(x, y, z), Arc::new(Cube::uniform(fill, 0)));
                }
            }
        }
        d
    }

    fn run(c: &'static Content, d: &mut Dim, ticks: u32) {
        d.settle(c);
        for _ in 0..ticks {
            d.tick(c);
        }
    }

    #[test]
    fn freeze_pairs_wet_and_dry_states() {
        let c = content();
        let (water, kelp) = (id(c, "t:water"), id(c, "t:kelp"));
        let (dry, wet) = (id(c, "t:slab[waterlogged=false]"), id(c, "t:slab[waterlogged=true]"));
        assert_eq!(c.states[kelp as usize].holds, Some(0));
        assert_eq!(c.states[kelp as usize].liquid_in(), Some(0));
        assert_eq!(c.states[kelp as usize].liquid, None);
        assert_eq!(c.states[dry as usize].holds, None);
        assert_eq!(c.drained.get(&wet), Some(&dry));
        assert_eq!(c.filled.get(&(dry, 0)), Some(&wet));
        assert!(!c.drained.contains_key(&kelp), "kelp has no dry state");
        // It shades like the water it stands in.
        assert!(c.states[kelp as usize].absorb[0] >= c.states[water as usize].absorb[0]);
        assert_eq!(c.left(kelp), water);
        assert_eq!(c.left(id(c, "t:stone")), AIR);
    }

    #[test]
    fn placing_into_water_fills_and_out_of_it_drains() {
        let c = content();
        let (water, kelp, stone) = (id(c, "t:water"), id(c, "t:kelp"), id(c, "t:stone"));
        let (dry, wet) = (id(c, "t:slab[waterlogged=false]"), id(c, "t:slab[waterlogged=true]"));
        assert_eq!(c.placed_into(dry, water), Some(wet));
        assert_eq!(c.placed_into(dry, kelp), Some(wet), "into a cell a block stands in water in");
        assert_eq!(c.placed_into(dry, AIR), Some(dry));
        assert_eq!(c.placed_into(dry, water + 3), Some(dry), "flowing water is no source");
        assert_eq!(c.placed_into(wet, AIR), Some(dry));
        assert_eq!(c.placed_into(kelp, water), Some(kelp));
        assert_eq!(c.placed_into(kelp, AIR), None, "kelp only into water");
        assert_eq!(c.placed_into(stone, water), Some(stone));

        let mut d = world(c);
        let p = IVec3::new(3, 0, 3);
        d.set(c, p, water);
        run(c, &mut d, 1);
        assert!(d.place(c, p, 3, dry, [0.0; 3], 0, 0));
        assert_eq!(d.get(p), Some(wet));
        let q = IVec3::new(-3, 0, -3);
        assert!(!d.place(c, q, 3, kelp, [0.0; 3], 0, 0), "kelp in the air");
        assert_eq!(d.get(q), Some(AIR));
    }

    #[test]
    fn breaking_leaves_the_water() {
        let c = content();
        let (water, kelp, wet) = (id(c, "t:water"), id(c, "t:kelp"), id(c, "t:slab[waterlogged=true]"));
        let mut d = world(c);
        let (a, b) = (IVec3::new(0, 0, 0), IVec3::new(4, 0, 0));
        d.set(c, a, kelp);
        d.set(c, b, wet);
        run(c, &mut d, 1);
        assert!(d.break_block(c, a, ItemStack::EMPTY, 0, false));
        assert!(d.break_block(c, b, ItemStack::EMPTY, 0, false));
        assert_eq!(d.get(a), Some(water));
        assert_eq!(d.get(b), Some(water));
        // A mod's `set` to air too; and air over the water itself does empty it.
        d.set(c, a, kelp);
        d.set(c, a, AIR);
        assert_eq!(d.get(a), Some(water));
        d.set(c, a, AIR);
        assert_eq!(d.get(a), Some(AIR));
    }

    #[test]
    fn losing_its_support_breaks_it_off_into_water() {
        let c = content();
        let (water, kelp) = (id(c, "t:water"), id(c, "t:kelp"));
        let mut d = world(c);
        let (under, p) = (IVec3::new(2, -1, 2), IVec3::new(2, 0, 2));
        d.set(c, p, kelp);
        run(c, &mut d, 1);
        assert_eq!(d.get(p), Some(kelp));
        d.set(c, under, AIR);
        d.settle(c);
        assert_eq!(d.get(p), Some(water));
    }

    #[test]
    fn it_spreads_as_a_source_and_nothing_flows_into_it() {
        let c = content();
        let (water, kelp) = (id(c, "t:water"), id(c, "t:kelp"));
        let mut d = world(c);
        let p = IVec3::new(0, 0, 0);
        d.set(c, p, kelp);
        // A source two blocks off: neither replaces the other, and the gap between them, beside
        // two sources over stone, becomes one (the kelp counting as a source).
        d.set(c, p + IVec3::X * 2, water);
        run(c, &mut d, 40);
        assert_eq!(d.get(p), Some(kelp), "kelp stays, not washed away");
        assert_eq!(d.get(p + IVec3::X * 2), Some(water));
        assert_eq!(d.get(p + IVec3::X), Some(water), "a new source between them");
        let west = d.get(p - IVec3::X).unwrap();
        assert_eq!(c.states[west as usize].liquid, Some(0), "its water flows out");
        assert_ne!(west, water, "as flowing water");
        // Breaking it leaves a still source, which stays.
        assert!(d.break_block(c, p, ItemStack::EMPTY, 0, false));
        run(c, &mut d, 20);
        assert_eq!(d.get(p), Some(water));
    }

    #[test]
    fn area_fill_keeping_liquids_keeps_the_water() {
        let c = content();
        let (water, kelp) = (id(c, "t:water"), id(c, "t:kelp"));
        let mut states = [AIR; crate::world::cube::VOLUME];
        states[0] = kelp;
        let cube = Cube::from_states(&states);
        let fill = |keep_liquids| super::super::area::AreaFill {
            id: 1,
            center: DVec3::splat(0.5),
            radius: 3.0,
            state: AIR,
            keep_unbreakable: false,
            keep_liquids,
        };
        let (kept, _) = fill(true).apply(c, IVec3::ZERO, &cube).unwrap();
        assert_eq!(kept.get(0), water);
        let (gone, _) = fill(false).apply(c, IVec3::ZERO, &cube).unwrap();
        assert_eq!(gone.get(0), AIR);
    }

    #[test]
    fn swimming_through_it() {
        let c = content();
        let kelp = id(c, "t:kelp");
        let at = |p: IVec3| Some(if p == IVec3::ZERO { kelp } else { AIR });
        assert!(crate::physics::in_liquid(c, &at, DVec3::new(0.5, 0.1, 0.5), 0.0));
        assert_eq!(crate::physics::liquid_at(c, &at, DVec3::new(0.5, 0.5, 0.5)), Some(0));
        // A source's surface, 8/9 up.
        assert_eq!(crate::physics::liquid_at(c, &at, DVec3::new(0.5, 0.95, 0.5)), None);
        let stack = |p: IVec3| Some(if p.x == 0 && p.z == 0 && (0..2).contains(&p.y) { kelp } else { AIR });
        assert_eq!(crate::physics::liquid_at(c, &stack, DVec3::new(0.5, 0.95, 0.5)), Some(0), "covered by more");
    }
}
