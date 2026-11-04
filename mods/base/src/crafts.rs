//! Crafts that wait (`vitals::crafts`, `notes/CONTENT.md` decision 6): a tanning frame brain-tans a
//! raw hide into buckskin leather, a tanning pit soaks hides in oak-bark liquor into sole leather,
//! a charcoal clamp chars logs into coal, each in its real time on the growing pace. One thing
//! goes in (and what it's worked with), and after its time what it makes comes out; the work goes
//! on while the area is unloaded, as a furnace's does.

use vitals::crafts::{self, Station};
use weft_sdk::*;

pub use base_api::CRAFTING;

pub static TANNING_FRAME: Block = Block::named("tanning_frame");
pub static TANNING_PIT: Block = Block::named("tanning_pit");
pub static CHARCOAL_CLAMP: Block = Block::named("charcoal_clamp");
static FRAME_MENU: Menu = Menu::named("tanning_frame");
static PIT_MENU: Menu = Menu::named("tanning_pit");
static CLAMP_MENU: Menu = Menu::named("charcoal_clamp");
/// A raw hide off an animal (tanned at a frame or in a pit), sole leather from the pit, and the
/// oak bark that tans it.
pub static HIDE: Item = Item::named("hide");
pub static SOLE_LEATHER: Item = Item::named("sole_leather");
pub static OAK_BARK: Item = Item::named("oak_bark");
static AXES: Tag = Tag::named("tools/axe");
/// Bark an oak log gives when stripped.
const BARK_PER_LOG: u32 = 2;

value! {
    /// A station's work: which station (`STATIONS` index), how far along (0 to 1), and the item
    /// it's for (another starts over).
    pub struct Waiting { pub station: u32, pub done: f32, pub item: u32 }
}
static WAITING: Component<Waiting> = Component::named("waiting_craft");

/// Whether `item` is what `want` names (an item, or a `#tag`).
fn is(item: u32, want: &'static str) -> bool {
    if item == 0 {
        return false;
    }
    match want.strip_prefix('#') {
        Some(tag) => Tag::named(tag).on_item(item),
        None => item_name(item) == want,
    }
}

pub fn register(r: &mut Registry) {
    r.item(&HIDE).tag("hides").tab(&crate::tabs::INGREDIENTS);
    r.item(&SOLE_LEATHER)
        .tag("materials/leather")
        .tab(&crate::tabs::INGREDIENTS);
    r.item(&OAK_BARK)
        .tag("bark_tannin")
        .tab(&crate::tabs::INGREDIENTS);
    // Bark is peeled off an oak log with an axe (`strip`, Planner, 11-05): one log in the grid
    // makes planks, as in vanilla.
    r.shaped(&CRAFTING, "tanning_frame", 1, &["sts", "t t", "sts"], &[('s', "#stick"), ('t', "#string")]);
    r.shaped(&CRAFTING, "tanning_pit", 1, &["p p", "p p", "ppp"], &[('p', "#planks")]);
    r.shaped(&CRAFTING, "charcoal_clamp", 1, &["ddd", "dld", "ddd"], &[('d', "dirt"), ('l', "#logs")]);

    r.entity_component(&WAITING, Waiting { station: 0, done: 0.0, item: 0 }, component_flags::SAVE | component_flags::SYNC_MENU);
    r.system(&WAITING, tick);

    station(r, Station::TanningFrame, &TANNING_FRAME, &FRAME_MENU, "wood", "axe", [2.0, 0.0, 2.0], [14.0, 16.0, 14.0]);
    station(r, Station::TanningPit, &TANNING_PIT, &PIT_MENU, "wood", "axe", [0.0, 0.0, 0.0], [16.0, 12.0, 16.0]);
    station(r, Station::CharcoalClamp, &CHARCOAL_CLAMP, &CLAMP_MENU, "gravel", "shovel", [0.0, 0.0, 0.0], [16.0, 12.0, 16.0]);
}

/// An axe used on a log strips it (Planner, 11-05: "Bark comes from stripping a log with an
/// axe"): its stripped twin (`stripped_<wood>_log`, `blocks::log`) in its place, lying the same
/// way, and from an oak log `BARK_PER_LOG` oak bark at the player's feet, the tannin for the pit;
/// the axe wears by one. Anything else in hand uses the log as usual (places against it).
pub fn strip(u: &mut Use) -> bool {
    let (w, player, pos, state) = (u.world(), u.player(), u.pos(), u.state());
    if player == 0 {
        return false;
    }
    let (hotbar, slot) = (w.inventory(player).group("hotbar"), w.selected_slot(player));
    let axe = hotbar.get(slot);
    if !AXES.on_item(axe.item) {
        return false;
    }
    // A state's name carries its props (`base:oak_log[axis=y,..]`): the block's is before them.
    let name = name_of(abi::entry_kind::BLOCK, block_of(state))
        .split('[')
        .next()
        .unwrap_or("");
    let Some(wood) = name
        .strip_prefix("base:")
        .and_then(|n| n.strip_suffix("_log"))
    else {
        return false;
    };
    let Some(stripped) = Block::lookup(&format!("base:stripped_{wood}_log")) else { return false };
    let axis = state_prop(state, "axis")
        .filter(|&a| a != "bark")
        .unwrap_or("y");
    let felled = state_prop(state, "felled").unwrap_or("no");
    let stripped = [("axis", axis), ("felled", felled)]
        .iter()
        .try_fold(stripped, |s, (k, v)| state_with(s, k, v));
    if !stripped.is_some_and(|s| w.set_block(pos, s)) {
        return false;
    }
    if wood == "oak" {
        let at = w
            .entity(player)
            .map_or([pos.x as f64 + 0.5, pos.y as f64 + 1.0, pos.z as f64 + 0.5], |e| e.pos.into());
        w.spawn_item(at, ItemStack::new(OAK_BARK.id(), BARK_PER_LOG));
    }
    if !w.creative(player) {
        hotbar.set(slot, crate::felling::worn(axe, 1, DURABILITY.on_item(axe.item).unwrap_or(0)));
    }
    true
}

#[allow(clippy::too_many_arguments)]
fn station(
    r: &mut Registry,
    s: Station,
    block: &'static Block,
    menu: &'static Menu,
    sound: &'static str,
    tool: &'static str,
    from: [f32; 3],
    to: [f32; 3],
) {
    let c = crafts::craft(s);
    let index = crafts::STATIONS.iter().position(|&x| x == s).unwrap_or(0) as u32;
    let name = s.name();
    let mut m = r
        .machine(block, menu)
        .block(move |b| {
            b.sound_group(sound)
                .textures(&format!("{name}_top"), &format!("{name}_side"), &format!("{name}_side"))
                .model_box(from, to)
                .hardness(1.5)
                .mineable(tool)
                .tab(&crate::tabs::FUNCTIONAL)
        })
        .group(
            "input",
            Slots::single()
                .accepts(c.input.trim_start_matches('#'))
                .limit(s.holds()),
        );
    if let Some(with) = c.with {
        m = m.group("with", Slots::single().accepts(with.trim_start_matches('#')));
    }
    m.group("output", Slots::single().output())
        .attach_with(&WAITING, Waiting { station: index, done: 0.0, item: 0 })
        .screen(move |ui| {
            crate::window::window(ui, S::column().pad(7.0).gap(4.0).stretch(), &[], |ui| {
                ui.row(S::row().gap(8.0), |ui| {
                    ui.spacer();
                    ui.column(S::column().gap(2.0), |ui| {
                        ui.group("input");
                        if c.with.is_some() {
                            ui.group("with");
                        }
                    });
                    ui.recipe_arrow_bind(|c| c.menu_component(&WAITING).map_or(0.0, |w| w.done));
                    ui.group("output");
                    ui.spacer();
                });
                ui.player_inventory(true);
            });
        });
}

/// Every station of a region, catching up when its area loads again.
fn tick(b: &mut SystemBatch<Waiting>) {
    let w = b.world();
    for (id, s, ticks) in b.iter_mut() {
        let station = crafts::STATIONS[(s.station as usize).min(crafts::STATIONS.len() - 1)];
        let c = crafts::craft(station);
        let inv = w.inventory(id);
        let (input, output) = (inv.group("input"), inv.group("output"));
        let st = input.get(0);
        if st.item != s.item {
            (s.item, s.done) = (st.item, 0.0);
        }
        let with = c.with.map(|_| inv.group("with"));
        let enough_with = match (c.with, &with) {
            (Some(want), Some(g)) => is(g.get(0).item, want) && g.get(0).count >= st.count,
            _ => true,
        };
        let Some(out_item) = Item::lookup(c.output) else { continue };
        let out = output.get(0);
        let fits = out.is_empty() || (out.item == out_item && out.count + st.count <= max_stack(out_item));
        if !is(st.item, c.input) || !enough_with || !fits {
            s.done = 0.0;
            continue;
        }
        s.done = (s.done + ticks as f32 / (c.secs() * vitals::TICKS as f32)).min(1.0);
        if s.done >= 1.0 {
            output.set(0, if out.is_empty() { ItemStack::new(out_item, st.count) } else { out.grow(st.count) });
            if let Some(g) = with {
                g.set(0, g.get(0).shrink(st.count));
            }
            input.set(0, ItemStack::EMPTY);
            (s.item, s.done) = (0, 0.0);
        }
    }
}
