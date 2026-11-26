//! Fences, gates and the scarecrow: keeping animals off a field (`notes/ECOLOGY.md`, crop raids;
//! numbers in `vitals::fences`).
//! - **Fence**: wood, crafted early (planks and sticks). It joins its neighbours like a wall (any
//!   block tagged `fence_joins`: fences and gates), and stands a block and a half tall
//!   (`vitals::fences::FENCE_HEIGHT`), so nothing paths through it or jumps it: an animal steps up a
//!   block at most, and the pathfinder finds no room over it.
//! - **Fence gate**: as tall shut; opened and shut by a use (a player's, never an animal's), it
//!   lets anything through open.
//! - **Scarecrow**: a post with a straw body two blocks tall. A torch used on it hangs as its
//!   lantern: lit, its light (`vitals::fences::SCARECROW_LIGHT`) keeps the ground round it at the
//!   raid rule's block light (`RAID_LIGHT` 8, Animals' crop raids) out to `SCARECROW_REACH` blocks.
//!   Broken, it gives the torch back. Tagged `crop_guards`: Animals' raid guard keeps raiders off
//!   the crops within 4 blocks of it, lit or not.

use vitals::fences as fv;
use weft_sdk::*;

/// The woods with planks (`blocks.rs`'s `planks_<wood>` tiles): a `<wood>_fence` and a
/// `<wood>_fence_gate` each, of its own planks and drawn in them.
pub const WOODS: [&str; 14] = [
    "oak",
    "birch",
    "spruce",
    "acacia",
    "mesquite",
    "acuri_palm",
    "moriche_palm",
    "brazil_nut",
    "cacao",
    "andiroba",
    "matamata",
    "ipe",
    "cherry",
    "sequoia",
];
pub static SCARECROW: Block = Block::named("scarecrow");
static TORCH_ITEM: Item = Item::named("torch");

/// The fence's and gate's top, 1/16 blocks.
fn top() -> f32 {
    fv::FENCE_HEIGHT * 16.0
}

pub fn register(r: &mut Registry) {
    for wood in WOODS {
        fence(r, wood);
        gate(r, wood);
    }

    // A post, arms, a straw body and head; lit, a lantern hangs from its right arm.
    r.block(&SCARECROW)
        .sound_group("wood")
        .texture("hay_bale_side")
        // Animals' crop raid guard (animals-plants): anything tagged keeps raiders off the crops
        // within 4 blocks, lit or not, as a decoy; lit, its light covers the field further.
        .tag("crop_guards")
        .prop("lit", ["false", "true"])
        .states(|p, s| {
            let post = ("log_oak", [0.0, 0.0, 2.0, 16.0]);
            let straw = ("hay_bale_side", [0.0, 0.0, 16.0, 16.0]);
            let head = ("hay_bale_top", [0.0, 0.0, 16.0, 16.0]);
            let s = s
                .cutout()
                .model_box_faces([7.0, 0.0, 7.0], [9.0, 30.0, 9.0], [post; 6])
                .model_box_faces([1.0, 21.0, 7.0], [15.0, 23.0, 9.0], [post; 6])
                .model_box_faces([5.0, 13.0, 6.0], [11.0, 24.0, 10.0], [straw; 6])
                .model_box_faces([5.0, 24.0, 5.0], [11.0, 30.0, 11.0], [head; 6])
                .hitbox([5.0, 0.0, 5.0], [11.0, 30.0, 11.0]);
            if p.is("lit", "true") {
                let lamp = ("bronze_lantern", [0.0, 0.0, 16.0, 16.0]);
                s.model_box_faces([13.0, 20.0, 7.5], [14.0, 21.0, 8.5], [post; 6])
                    .model_box_faces([12.0, 15.0, 6.5], [15.0, 20.0, 9.5], [lamp; 6])
                    .light_rgb(fv::SCARECROW_LIGHT, fv::SCARECROW_LIGHT - 2, fv::SCARECROW_LIGHT - 5)
            } else {
                s
            }
        })
        .needs_below("")
        .hardness(1.0)
        .mined_by("axe", tool_tier::HAND)
        .on_use(|u| {
            let w = u.world();
            let pos = u.pos();
            if w.prop(pos, "lit") == Some("true") {
                return false;
            }
            let player = u.player();
            let inv = w.inventory(player);
            let slot = w.selected_slot(player);
            let held = inv.get(slot);
            if held.item != TORCH_ITEM.id() {
                return false;
            }
            inv.set(slot, held.shrink(1));
            w.prop_set(pos, "lit", "true");
            crate::player_sounds::door(&w, pos, Some(u.face()));
            weft_sdk::info!("scarecrow: a lantern hung at {pos:?}");
            true
        })
        .tab(&crate::tabs::FUNCTIONAL);
    r.loot(&SCARECROW)
        .pool(Pool::new().item("scarecrow"))
        .pool(Pool::new().when(Cond::state("lit=true")).item("torch"));
    r.shaped(&base_api::recipes::CRAFTING, "scarecrow", 1, &[" f ", "sfs", " s "], &[('f', "plant_fibre"), ('s', "#stick")]);
}

/// A fence of `wood`: a post and two rails toward each joined side.
fn fence(r: &mut Registry, wood: &str) {
    let planks = format!("planks_{wood}");
    r.block(Block::generated(format!("{wood}_fence")))
        .sound_group("wood")
        .texture(&planks)
        .tag("fences")
        .tag("fence_joins")
        .joins(Facings::HORIZONTAL, "#fence_joins")
        .states(|p, s| {
            let mut s = s
                .cutout()
                .model_box([6.0, 0.0, 6.0], [10.0, 16.0, 10.0])
                .hitbox([6.0, 0.0, 6.0], [10.0, top(), 10.0]);
            for d in [Dir::North, Dir::South, Dir::East, Dir::West] {
                if !p.is(d.name(), "true") {
                    continue;
                }
                let o = d.offset();
                // From the post's side out to the cell's edge on that side.
                let span = |v: i32, lo: f32, hi: f32| {
                    if v < 0 {
                        (0.0, lo)
                    } else if v > 0 {
                        (hi, 16.0)
                    } else {
                        (7.0, 9.0)
                    }
                };
                let (x0, x1) = span(o.x, 6.0, 10.0);
                let (z0, z1) = span(o.z, 6.0, 10.0);
                for (y0, y1) in [(6.0, 9.0), (12.0, 15.0)] {
                    s = s.model_box([x0, y0, z0], [x1, y1, z1]);
                }
                let wide = |a: f32, b: f32| if (a, b) == (7.0, 9.0) { (6.0, 10.0) } else { (a, b) };
                let ((hx0, hx1), (hz0, hz1)) = (wide(x0, x1), wide(z0, z1));
                s = s.hitbox([hx0, 0.0, hz0], [hx1, top(), hz1]);
            }
            s
        })
        .needs_below("")
        .hardness(2.0)
        .mined_by("axe", tool_tier::HAND)
        .tab(&crate::tabs::BUILDING);
    let of = format!("#planks/{wood}");
    r.shaped(&base_api::recipes::CRAFTING, &format!("{wood}_fence"), fv::FENCES_MADE, &["psp", "psp"], &[('p', &of), ('s', "#stick")]);
}

/// A fence gate of `wood`.
fn gate(r: &mut Registry, wood: &str) {
    // Written facing north (across x); open, it swings against the west side and stops nothing.
    let planks = format!("planks_{wood}");
    r.block(Block::generated(format!("{wood}_fence_gate")))
        .sound_group("wood")
        .texture(&planks)
        .tag("fence_gates")
        .tag("fence_joins")
        .facing(Facings::HORIZONTAL, Toward::Player)
        .prop("open", ["false", "true"])
        .states(|p, s| {
            let s = s.cutout();
            if p.is("open", "true") {
                s.not_solid()
                    .model_box([0.0, 5.0, 0.0], [2.0, 16.0, 2.0])
                    .model_box([0.0, 6.0, 2.0], [2.0, 9.0, 16.0])
                    .model_box([0.0, 12.0, 2.0], [2.0, 15.0, 16.0])
                    .model_box([0.0, 6.0, 14.0], [2.0, 15.0, 16.0])
                    .hitbox([0.0, 0.0, 0.0], [2.0, 16.0, 16.0])
                    // Not solid, so only outlined: the doorway too, so a use where it stood shuts it
                    // (fences4: the second use looked through the open gate).
                    .hitbox([0.0, 0.0, 6.0], [16.0, 16.0, 10.0])
            } else {
                s.model_box([0.0, 5.0, 7.0], [2.0, 16.0, 9.0])
                    .model_box([14.0, 5.0, 7.0], [16.0, 16.0, 9.0])
                    .model_box([2.0, 6.0, 7.0], [14.0, 9.0, 9.0])
                    .model_box([2.0, 12.0, 7.0], [14.0, 15.0, 9.0])
                    .model_box([7.0, 9.0, 7.0], [9.0, 12.0, 9.0])
                    .hitbox([0.0, 0.0, 6.0], [16.0, top(), 10.0])
            }
        })
        .needs_below("")
        .hardness(2.0)
        .mined_by("axe", tool_tier::HAND)
        .on_use(|u| {
            u.world().prop_cycle(u.pos(), "open");
            crate::player_sounds::door(&u.world(), u.pos(), Some(u.face()));
            true
        })
        .tab(&crate::tabs::BUILDING);
    let of = format!("#planks/{wood}");
    r.shaped(&base_api::recipes::CRAFTING, &format!("{wood}_fence_gate"), 1, &["sps", "sps"], &[('p', &of), ('s', "#stick")]);
}
