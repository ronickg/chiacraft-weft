//! The look's knobs: chiacraft's shader pack reads these settings (`options` in its `pack.cfg`,
//! `scene.options` in the shaders, `look_options` in `look.wgsl`), so the day and the night can be
//! tuned in Mod Settings, or for one run with `--config "base:look_exposure=1.2"` (a PC job can sweep
//! a few values), without editing a shader or an asset. Each is a factor of 1 (or 0) at the look
//! as made; the order here is the order the pack reads them in (`OPTIONS`).

use weft_sdk::*;

pub static LOOK_EXPOSURE: Config<f32> = Config::named("look_exposure");
pub static LOOK_SUN: Config<f32> = Config::named("look_sun");
pub static LOOK_SUN_WARMTH: Config<f32> = Config::named("look_sun_warmth");
pub static LOOK_FILL: Config<f32> = Config::named("look_fill");
pub static LOOK_FILL_COOL: Config<f32> = Config::named("look_fill_cool");
pub static LOOK_CONTRAST: Config<f32> = Config::named("look_contrast");
pub static LOOK_SATURATION: Config<f32> = Config::named("look_saturation");
pub static LOOK_SHADE: Config<f32> = Config::named("look_shade");
pub static LOOK_HAZE: Config<f32> = Config::named("look_haze");
pub static LOOK_FOG: Config<f32> = Config::named("look_fog");
pub static NIGHT_LIGHT: Config<f32> = Config::named("night_light");
pub static NIGHT_MOON: Config<f32> = Config::named("night_moon");
pub static LOOK_CANOPY: Config<f32> = Config::named("look_canopy");
pub static LOOK_SHAFTS: Config<f32> = Config::named("look_shafts");
/// The HDR world's picture (`core:hdr_world`): the tone curve by day (0 Lottes's, 1 Hejl's 2015, as
/// Ronald's Bliss settings, 2 AgX), how strong the bloom is, and how much the exposure follows the view.
pub static LOOK_CURVE: Config<f32> = Config::named("look_curve");
pub static LOOK_BLOOM: Config<f32> = Config::named("look_bloom");
pub static LOOK_AUTO_EXPOSURE: Config<f32> = Config::named("look_auto_exposure");
/// Gap tests against Bliss (Planner, 11-05): the sky's light in the shade, and grass and leaves
/// darker and less yellow (0 as made to 1: 30% darker). Tests, not looks: as made by default.
pub static LOOK_SKY_LIGHT: Config<f32> = Config::named("look_sky_light");
pub static LOOK_FOLIAGE_TEST: Config<f32> = Config::named("look_foliage_test");
/// The HDR world's white balance by day: 0 neutral, + warmer and less blue (1.6 by default: the gap
/// sheets, our land bluer than Bliss's at every hour).
pub static LOOK_WHITE: Config<f32> = Config::named("look_white");
/// A test for the tree diagnosis: 1 takes the corner shade (AO) away, 0 as made.
pub static LOOK_AO_TEST: Config<f32> = Config::named("look_ao_test");
/// The HDR world's contrast-adaptive sharpening, 0 none to 1 (0.4: Bliss runs 0.35-0.4).
pub static LOOK_SHARPEN: Config<f32> = Config::named("look_sharpen");
/// The canopy's light range and holes (HDR; Trees' find, 11-06: our crowns flat mats, luma p5-p95
/// 52-108 against Bliss's 53-176): the sun on leaves' lit faces (2: the light-range sweep lr1,
/// roughness +15-20% at 150 blocks, no more past 2), and leaves' see-through holes kept at a
/// distance (1 on, 0 off).
pub static LOOK_LEAF_SUN: Config<f32> = Config::named("look_leaf_sun");
pub static LOOK_LEAF_HOLES: Config<f32> = Config::named("look_leaf_holes");
/// The light through leaves (HDR; Trees, 11-06): 1 a leaf takes the brighter of the sun on its face
/// and the sun through it (Bliss's leaf subsurface scattering, its SSS types 1-2), 0 none. Off by
/// default and in the Bliss comparisons: Ronald's Bliss (SSS type 3, no LabPBR pack) has none.
pub static LOOK_LEAF_LIGHT: Config<f32> = Config::named("look_leaf_light");

/// The pack's `options` line: the settings in the order `look.wgsl` reads them.
pub const OPTIONS: &str = "base:look_exposure, base:look_sun, base:look_sun_warmth, base:look_fill, base:look_fill_cool, \
base:look_contrast, base:look_saturation, base:look_shade, base:look_haze, base:look_fog, base:night_light, base:night_moon, \
base:look_canopy, base:look_shafts, core:view_distance, base:look_storybook, base:fx_night_sky, base:fx_shooting_stars, \
base:moon_size, base:fx_glow, base:fx_biome_air, base:fx_sparkle, base:look_curve, base:look_bloom, base:look_auto_exposure, base:look_sky_light, base:look_foliage_test, base:look_white, base:look_ao_test, base:look_sharpen, base:look_leaf_sun, base:look_leaf_holes, base:look_leaf_light";

pub fn register(r: &mut Registry) {
    // By day: how bright the picture is taken, the sun's strength and warmth (-1 whiter, +1 toward
    // a 4300 K orange), the sky's fill light in the shade and how cool it is (-1 neutral, +1 bluer),
    // the curve's contrast, the colour after it, how fast the sky's light fades under cover (more:
    // darker forest floors), the haze over distance and the fog's light.
    r.config(&LOOK_EXPOSURE).float(1.0, 0.5, 2.0);
    r.config(&LOOK_SUN).float(1.0, 0.5, 2.0);
    r.config(&LOOK_SUN_WARMTH).float(0.0, -1.0, 1.0);
    r.config(&LOOK_FILL).float(1.0, 0.5, 2.0);
    r.config(&LOOK_FILL_COOL).float(0.0, -1.0, 1.0);
    r.config(&LOOK_CONTRAST).float(1.0, 0.8, 1.3);
    r.config(&LOOK_SATURATION).float(1.0, 0.7, 1.3);
    r.config(&LOOK_SHADE).float(1.0, 0.5, 2.0);
    // 1.6: Ronald's pick, "crisp + more depth" (11-05).
    r.config(&LOOK_HAZE).float(1.6, 0.0, 3.0);
    r.config(&LOOK_FOG).float(1.0, 0.0, 2.0);
    // By night (dark by design): the night's sky light and the moon's light.
    r.config(&NIGHT_LIGHT).float(1.0, 0.3, 2.0);
    r.config(&NIGHT_MOON).float(1.0, 0.3, 2.0);
    // Under a forest's canopy (how closed it is comes from the biome, `vitals::air::CANOPY`): how
    // dim and green the shade under the leaves is, with its sunflecks, and how strong the light
    // shafts through the crown gaps are.
    r.config(&LOOK_CANOPY).float(1.0, 0.2, 2.0);
    r.config(&LOOK_SHAFTS).float(1.0, 0.2, 3.0);
    // The HDR world's picture: 0 Lottes's curve (the look as made), 1 Hejl's (the default, Ronald's
    // pick of 11-05: "Hejl 2015"), 2 AgX; bloom 0 none to 3;
    // the exposure following the view, 0 fixed to 2.
    r.config(&LOOK_CURVE).float(1.0, 0.0, 2.0);
    r.config(&LOOK_BLOOM).float(1.0, 0.0, 3.0);
    r.config(&LOOK_AUTO_EXPOSURE).float(1.0, 0.0, 2.0);
    r.config(&LOOK_SKY_LIGHT).float(1.0, 0.2, 1.5);
    r.config(&LOOK_FOLIAGE_TEST).float(0.0, 0.0, 1.0);
    r.config(&LOOK_WHITE).float(1.6, -1.0, 2.5);
    r.config(&LOOK_AO_TEST).float(0.0, 0.0, 1.0);
    r.config(&LOOK_SHARPEN).float(0.4, 0.0, 1.0);
    r.config(&LOOK_LEAF_SUN).float(1.0, 0.5, 4.0);
    r.config(&LOOK_LEAF_HOLES).float(1.0, 0.0, 1.0);
    r.config(&LOOK_LEAF_LIGHT).float(0.0, 0.0, 1.0);
}
