//! The test list. Each test is an `async fn(T) -> R`; failed checks are collected, `?` stops it.

mod account;
mod audit;
mod bodies;
mod extend;
mod gameplay;
mod held;
mod looks;
mod packages;
mod probe;
pub mod redstone;
mod scene;
mod sync;
mod ui;
mod visual;
mod world;

use std::time::Duration;

use crate::runner::{R, T, TestDef};

/// The player's max health in the engine's points (the base game's 20): the survival vitals set it
/// from what's in the stomach (a new player's light meal comes to about 10.8), so checks compare
/// with this, not with 20.
pub fn max_health(t: &crate::runner::T) -> f32 {
    t.c()
        .view()
        .component(&base_api::VITALS)
        .map_or(20.0, |v| v.max_health / base_api::HEALTH_SCALE)
}

/// Whether `health` is the full `max_health` (it drifts down a little as the stomach digests).
pub fn full(t: &crate::runner::T, health: f32) -> bool {
    health >= max_health(t) - 0.1
}

pub fn add(p: [i32; 3], d: [i32; 3]) -> [i32; 3] {
    [p[0] + d[0], p[1] + d[1], p[2] + d[2]]
}

pub fn at(p: [i32; 3]) -> String {
    format!("{} {} {}", p[0], p[1], p[2])
}

pub fn all() -> Vec<TestDef> {
    crate::tests![
        scene::scene,
        ui::screens,
        account::account,
        packages::packages,
        packages::package_pins,
        ui::rebind,
        ui::f3,
        ui::hud_editor,
        ui::debug_lines,
        ui::inventory_crafting,
        ui::creative_inventory,
        ui::chat,
        ui::chat_overflow,
        ui::text_on_screen,
        ui::mod_settings,
        ui::recipe_viewer,
        ui::inventory_hotkeys,
        world::persistence,
        world::missing_dimension,
        world::dimensions,
        world::world_seed,
        world::riding,
        world::camera,
        world::lod_columns,
        world::entity_edit,
        world::virtual_menus,
        world::worldgen,
        world::plants_on_cube_tops,
        world::foreign_terrain,
        world::tick_control,
        world::growth,
        world::catch_up,
        world::crater_saved,
        world::edit_while_loading,
        world::gpu_terrain,
        gameplay::sprint_and_sneak_modes,
        gameplay::mining_and_wear,
        gameplay::creative_and_invulnerable,
        gameplay::block_shapes,
        gameplay::cell_parts,
        gameplay::hitboxes,
        gameplay::replaced_plants_drop,
        gameplay::swimming,
        gameplay::attack_charge,
        gameplay::permissions,
        gameplay::armor,
        gameplay::slot_accepts,
        gameplay::liquid_slot,
        gameplay::slot_limit,
        gameplay::bucket,
        gameplay::advancements,
        gameplay::beds,
        gameplay::second_hand,
        gameplay::model_bones,
        gameplay::beds_far,
        gameplay::dedicated_server,
        gameplay::base_alone,
        gameplay::names_translated,
        gameplay::burning,
        gameplay::entity_fields,
        gameplay::fall_damage,
        gameplay::explosions_and_water,
        gameplay::sneak_edge,
        gameplay::movement_attributes,
        gameplay::mob_physics,
        gameplay::player_box,
        gameplay::food,
        gameplay::every_food_has_nutrition,
        gameplay::clothes_and_stew,
        gameplay::calendar_follows_time,
        gameplay::drop_and_pickup,
        gameplay::creative_sword,
        gameplay::spawn_eggs,
        gameplay::toast_with_the_gui,
        gameplay::advancements_by_tags,
        gameplay::air_in_creative,
        gameplay::swimming_no_bob,
        gameplay::own_names,
        extend::events_batched,
        extend::vetoes_and_timers,
        extend::block_properties,
        held::held_water_place,
        held::held_water_break,
        held::held_water_piston,
        held::held_water_swim,
        held::held_water_mesh,
        sync::menu_sync,
        visual::liquid_layers,
        visual::break_overlay,
        visual::tooltips,
        visual::night_light,
        visual::multiplayer,
        visual::client_hooks,
        visual::held_items,
        visual::hide_hud,
        audit::saved_entities_stay_gone,
        audit::death_is_a_state,
        audit::reach_and_mining_attributes,
        audit::respawn_keeps_mode,
        audit::kill_and_void,
        audit::leaving_keeps_cursor,
        audit::containers,
        audit::water_keeps_flowing,
        audit::ladder_no_fall,
        audit::bucket_not_wasted,
        audit::shift_click_skips_grid,
        audit::no_block_into_mob,
        audit::tp_nan,
        audit::changes_in_one_tick,
        audit::no_ghost_light,
        audit::chat_down_keeps_text,
        audit::break_refused,
        audit::creative_sword_no_flicker,
        looks::sky,
        looks::clouds,
        looks::light_by_day,
        looks::ctm_any_model,
        looks::third_person_shakes,
        looks::block_dust_scatters,
        looks::translucent_order,
        looks::attack_indicator,
        redstone::redstone_wire,
        redstone::redstone_on_walls,
        redstone::redstone_conduction,
        redstone::redstone_torch,
        redstone::redstone_repeater,
        redstone::redstone_comparator,
        redstone::redstone_gate,
        redstone::redstone_piston,
        redstone::redstone_piston_moves,
        redstone::redstone_door,
        redstone::redstone_contest,
        redstone::redstone_rides,
        redstone::redstone_body_edges,
        redstone::redstone_floaters_together,
        redstone::redstone_floater_riders,
        redstone::redstone_floater,
        redstone::redstone_body_light,
        redstone::redstone_body_and_world,
        redstone::redstone_moves_chest,
        redstone::redstone_timing,
        redstone::redstone_anchor,
        redstone::redstone_flying_machine,
        redstone::redstone_flying_rider,
        redstone::redstone_by_hand,
        redstone::redstone_looks,
        redstone::redstone_clock,
        redstone::redstone_saved,
        redstone::redstone_on_a_structure,
        redstone::redstone_turned,
        looks::shaders_and_glow,
        looks::texture_reload,
        looks::pack_lang,
        looks::broken_shader_pack,
        looks::rich_text,
        looks::models,
        bodies::lift,
        bodies::ride_frames,
        bodies::skiff_smooth,
        gameplay::respawn,
        bodies::skiff_blocks,
        bodies::body_chest,
        bodies::body_sneak_edge,
        bodies::body_place,
        redstone::body_walking,
        bodies::zombie_chases,
        bodies::neutral_mobs,
    ]
}

/// Measurements without a verdict worth failing a run over; `--test bench`.
pub fn bench() -> Vec<TestDef> {
    crate::tests![
        redstone::redstone_bench,
        world::fly_load,
        world::load_benchmark,
        world::item_farm,
        world::mob_crowd,
        looks::particle_crowd,
        looks::layered_icon,
        looks::particle_rules,
        world::crater,
        gameplay::furnace_one_coal,
        gameplay::crafting_table_tags,
        gameplay::creative_middle_click,
        gameplay::mob_in_the_way,
        visual::pick_block,
        visual::wall_torch,
        visual::ladder_placement,
        ui::scrollbar,
        ui::pack_drag,
        ui::mods_switch,
        looks::texture_seams,
        looks::ctm_off_the_cube,
        looks::item_model_file,
        looks::block_model_file,
        looks::distant_terrain_from_above,
        world::distant_terrain,
        world::distant_terrain_underground,
        world::crater_view,
        world::sky_column_edit,
        bodies::natural_spawning,
        redstone::redstone_observer,
        redstone::redstone_dispenser,
        redstone::redstone_copper_lamp,
        redstone::redstone_spit,
        redstone::redstone_saved_moving,
        redstone::redstone_floater_up,
        audit::liquid_surface,
        visual::third_person,
        visual::crouch_pose,
        visual::inventory_figure,
        visual::player_skin,
        gameplay::sounds,
        redstone::redstone_first_piece,
        redstone::redstone_head_stays_inside,
        redstone::redstone_lift_api,
        bodies::skiff_pause,
        bodies::body_lit_at_once
    ]
}

/// A process's first world is slow to open and to settle, which timings shouldn't be measuring.
/// Left out of the report unless it fails.
pub async fn warm_up(t: T) -> R {
    t.flat().await?;
    t.secs(0.5).await
}

impl T {
    /// A fresh world moved to the flat test dimension, survival, at noon, loaded and idle.
    pub async fn flat(&self) -> R {
        self.world(1).await?;
        self.cmd("/dim base:flatlands").await?;
        self.until("flatlands", Duration::from_secs(20), |t| t.state().dim != 0 || t.state().pos.y < 10.0)
            .await?;
        self.loaded().await?;
        self.op("mode survival").await?;
        self.idle().await
    }

    /// Turns the local player to look at the middle of block `p` from its eyes.
    pub fn look_at(&self, p: [i32; 3]) {
        let s = self.state();
        let eye = [s.pos.x, s.pos.y + self.c().entity(0).map_or(1.62, |e| e.size.eye as f64), s.pos.z];
        let d = [p[0] as f64 + 0.5 - eye[0], p[1] as f64 + 0.5 - eye[1], p[2] as f64 + 0.5 - eye[2]];
        let yaw = (-d[0]).atan2(-d[2]).to_degrees() as f32;
        let pitch = d[1].atan2((d[0] * d[0] + d[2] * d[2]).sqrt()).to_degrees() as f32;
        self.c().set_look(yaw, pitch);
    }

    /// Counts entities of a kind near the player as the server has them.
    pub async fn count_near(&self, kind: &str, r: f64) -> Result<usize, String> {
        let list = self.op(&format!("near ~ ~ ~ {r}")).await?;
        Ok(list
            .split_whitespace()
            .filter(|e| e.ends_with(&format!(":{kind}")))
            .count())
    }
}
