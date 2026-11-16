use weft_sdk::{abi::assembly_result as result, *};

static TOOL: Item = Item::named("assembly_tool");
static HELM: Block = Block::named("helm");

pub fn tool_held(w: World, player: u64) -> bool {
    let stack = w.inventory(player).get(w.selected_slot(player));
    !stack.is_empty() && stack.item == TOOL.id()
}

pub fn use_controller(w: World, player: u64, p: BlockPos, tool: bool) -> bool {
    feedback(w, player, w.assembly_select(player, Some(p), tool))
}

fn feedback(w: World, player: u64, result: u32) -> bool {
    let key = match result {
        result::FIRST_CORNER => "assembly.base.first_corner",
        result::BOUNDS => "assembly.base.bounds",
        result::PREVIEW => "assembly.base.preview",
        result::CONFIRMED => "assembly.base.confirmed",
        result::TOO_LARGE => "assembly.base.too_large",
        result::UNAVAILABLE => "assembly.base.unavailable",
        result::CLEARED => "assembly.base.cleared",
        result::DOCKING => "assembly.base.docking",
        result::SEATED => return true,
        _ => return false,
    };
    w.hint(key, &[]).to(player);
    true
}

pub fn register(r: &mut Registry) {
    r.item(&TOOL)
        .texture("stick")
        .max_stack(1)
        .tab(&crate::tabs::TOOLS)
        .on_use(|u| {
            let w = u.world();
            let player = u.entity();
            let cell = u.target_block().map(|(p, _)| p);
            feedback(w, player, w.assembly_select(player, cell, true))
        });
    r.block(&HELM)
        .facing(Facings::HORIZONTAL, Toward::Away)
        .body_helm()
        .density(0.65)
        .texture("planks_spruce")
        .states(|p, s| {
            let mut faces = [("planks_spruce", UV_FROM_BOX); 6];
            faces[4].0 = "log_spruce_top";
            s.orientation(Orientation::front(p.facing()))
                .model_box_faces([0.0; 3], [16.0; 3], faces)
        })
        .sound_group("wood")
        .hardness(2.0)
        .mined_by("axe", tool_tier::HAND)
        .tab(&crate::tabs::REDSTONE)
        .on_use(|u| use_controller(u.world(), u.player(), u.pos(), tool_held(u.world(), u.player())));

    r.set("#materials/wood", &DENSITY, 0.55);
    r.set("#logs", &DENSITY, 0.62);
    r.set("#planks", &DENSITY, 0.55);
    r.set("base:iron_block", &DENSITY, 2.4);
    r.set("base:gold_block", &DENSITY, 3.5);
}
