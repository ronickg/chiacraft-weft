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
/// A debug view for matching the light against Bliss's (its debug_DIRECT / debug_INDIRECT): 0 the
/// picture, 1 the sun's light alone, 2 the sky's alone, on white; with the mist's banks on
/// (`look_mist_domes`), 3 the light they scatter alone, on black, 4 how much they hide (grey),
/// 5 where along their march the sun reaches them (red the shadow map's light, green the clouds',
/// blue how much mist); 14 view 13's swatches with no clouds in the cloudy average (the clear
/// sky's fill, against Bliss's capture 5).
pub static LOOK_DEBUG: Config<f32> = Config::named("look_debug");
/// The HDR world's sky light by day in the upper sky's own hue, as Bliss's: 0 the palette's fill as
/// made, 1 all the way.
pub static LOOK_SKY_HUE: Config<f32> = Config::named("look_sky_hue");
/// The HDR world's exposure by day in Bliss's own units (its curve's gentle slope, no near-land dim):
/// 1 as made since 11-24 (with auto exposure, bloom added and fill 1.0 our cave, arch, overhang,
/// forest and vista land within about 10 luma of Bliss's auto-exposed refs: PC lkauto1), 0 the old curve.
pub static LOOK_EXPOSURE_ABS: Config<f32> = Config::named("look_exposure_abs");
/// The HDR world's warmth on sunlit foliage toward a leaf-top yellow: 0.7 as made, 0 none.
pub static LOOK_SPLIT_WARM: Config<f32> = Config::named("look_split_warm");
/// The HDR world's evening mist, Bliss's evening fog (the 18:00 warm veil): with the sun low in the
/// west the mist lies densest at the low land, thinning with height. 0 as made, 1 all the way, up
/// to 3 thicker.
pub static LOOK_EVENING_MIST: Config<f32> = Config::named("look_evening_mist");
/// The HDR world's bloom by day added on top, as Bliss's (+35% at the centre, +12% in the corners at
/// noon): 1 as made since 11-24 (it lifts the shade as Bliss's does: the noon vista's darkest quarter
/// 102 against Bliss's 103, PC lkstack2), 0 mixed in keeping the light's sum.
pub static LOOK_BLOOM_ADD: Config<f32> = Config::named("look_bloom_add");
/// A test switch for like-for-like compares: 1 takes the high cirrus out of the sky (with soft
/// clouds and cloud shadows off and a blank clouds texture, no cloud at all). 0 as made.
pub static LOOK_CIRRUS_OFF: Config<f32> = Config::named("look_cirrus_off");
/// The HDR world's veiling glare (a little of the sky's light over the whole picture, lifting the
/// shade): 1 as made, 0 none (Bliss has none).
pub static LOOK_GLARE: Config<f32> = Config::named("look_glare");
/// The far shade from the sky (`fx_far_ao`) as strong as Bliss's: what a point keeps of the sky's
/// light is its open share to this power (Bliss's 6). 0 as made.
pub static LOOK_AO_POWER: Config<f32> = Config::named("look_ao_power");
/// Water's absorption by depth as Bliss's (per colour, water and dirt, floored dark): 0 as made, 1
/// Bliss's, more stronger.
pub static LOOK_WATER_ABSORB: Config<f32> = Config::named("look_water_absorb");
/// The foam at the shore and over the shallows: 1 as made, 0 none (Bliss has none).
pub static LOOK_WATER_FOAM: Config<f32> = Config::named("look_water_foam");
/// Water's mirror as Bliss's (Fresnel's fifth power from 0.02, the sky where the ripples point):
/// 0 as made, 1 Bliss's.
pub static LOOK_WATER_MIRROR: Config<f32> = Config::named("look_water_mirror");
/// The evening's blue hour as Bliss's: 0 as made; above 0 the sky a deep blue from sunset to about
/// 10 degrees under, the land lit by it at this many times its light.
pub static LOOK_TWILIGHT: Config<f32> = Config::named("look_twilight");
/// How bright the water's own body is, near and far: 1 as made, less darker.
pub static LOOK_WATER_BODY: Config<f32> = Config::named("look_water_body");
/// The blue near air's share with the sun low: 0 as made (a fifth), up to 1 (as with the sun high).
pub static LOOK_LOW_AIR: Config<f32> = Config::named("look_low_air");
/// How much of the low sun's colour the far haze takes: 1 as made, 0 none (the air keeps the sky's).
pub static LOOK_HAZE_SUN: Config<f32> = Config::named("look_haze_sun");
/// A wide white glow round the sun in the air, strongest low: 0 as made (none).
pub static LOOK_SUN_GLOW: Config<f32> = Config::named("look_sun_glow");
/// The low sky away from the evening sun a clear blue: 0 as made, 1 as Bliss's.
pub static LOOK_EVENING_AWAY: Config<f32> = Config::named("look_evening_away");
/// The evening mist lit in the light's own colours (0 as made: the low sun's gold in it, 1 Bliss's:
/// the sun's and the sky's own colour, a pale pink-lavender layer; lsG_ev's 18:00 land went orange).
pub static LOOK_MIST_TINT: Config<f32> = Config::named("look_mist_tint");
/// How much of the evening sun's light the blue hour's dimming keeps: 0 as made, 1 all (Bliss's
/// 18:30 tops stand in a red sun).
pub static LOOK_LOW_SUN_KEEP: Config<f32> = Config::named("look_low_sun_keep");
/// The light in the shade with the evening sun under about 12°: 1 as made, less darker.
pub static LOOK_EVENING_SHADE: Config<f32> = Config::named("look_evening_shade");
/// The air added over the land fading with the evening sun from 10° to 3° up: 0 as made, 1 all.
pub static LOOK_AIR_FADE: Config<f32> = Config::named("look_air_fade");
/// The golden hour's warm fill from the sky round the low sun: 1 as made, 0 none (Bliss's shade
/// takes the averaged sky alone).
pub static LOOK_GLOW_FILL: Config<f32> = Config::named("look_glow_fill");
/// The dawn mist lit in the light's own colours: 0 as made (gold toward the sun, half the sky's
/// light), 1 Bliss's cream-white (the sun's colour and the sky's as they are).
pub static LOOK_MIST_DAWN_TINT: Config<f32> = Config::named("look_mist_dawn_tint");
/// The mist's banks as lumps that stand up out of the layer: 0 as made (a top heaving 5 blocks
/// either way), 1 Bliss's domes (20 either way, a softer top).
pub static LOOK_MIST_DOMES: Config<f32> = Config::named("look_mist_domes");
/// The horizon's haze never darker than the sky behind it (0 as made, 1 as Bliss's, whose sky
/// brightens to the horizon): in the HDR world the mist away from the sun is lit dimmer than the sky,
/// and over the far sky it drew a grey band above the land (lv2's horizon_lv2, 11-24).
pub static LOOK_HORIZON_LIFT: Config<f32> = Config::named("look_horizon_lift");
/// The low sun's disc near white with a wide soft glow, as Bliss's (0 as made, 1 Bliss's): ours went
/// through the HDR curve as a small saturated yellow-green blob at 06:00 (Planner's look notes, 11-24).
pub static LOOK_SUN_DISC: Config<f32> = Config::named("look_sun_disc");
/// The dawn before sunrise as Bliss's (0 as made, 1 Bliss's): with the sun under the horizon, the sky
/// a deep blue with a thin pink line where it will rise, and no disc, glow or glare of the sun yet.
/// Ours at 05:00 (the sun 3.5° down) was a bright pink-lavender sky with the sun's white glow on the
/// horizon, the land under it pale (lv6, 11-24: sky 159 148 180 against Bliss's 59 104 204).
/// Mornings only: the evening's sky stays as fitted.
pub static LOOK_DAWN_SKY: Config<f32> = Config::named("look_dawn_sky");
/// The mist banks' density, sunlight and shade (`look_mist_domes`), each times its own: 0 (or 1) as
/// made. For tuning them live against Bliss (lv8, 11-24).
pub static LOOK_BANK_DENSE: Config<f32> = Config::named("look_bank_dense");
pub static LOOK_BANK_SUN: Config<f32> = Config::named("look_bank_sun");
pub static LOOK_BANK_SHADE: Config<f32> = Config::named("look_bank_shade");
/// How far the sun's shadows reach (blocks): past it a surface open to the sky (sky light 14 or
/// more) is in full sun and one under cover in none, the sky's light alone, as Bliss past its
/// shadow distance (128 by default, `composite1.fsh` LM_shadowMapFallback; method only); 0 as
/// made, the shadows out to the view's end. From the vista Bliss's land is all sunlit, ours mostly
/// in its own crowns' shade (the condition matrix, 11-24).
pub static LOOK_SHADOW_REACH: Config<f32> = Config::named("look_shadow_reach");
/// The mist banks' shape, for tuning against Bliss's soft mist (Ronald, 11-24: ours "blocky"):
/// how fast a bank thins above its top (times, 0 as made = 1), how soft its edges are (0 as made to
/// 1), and the height it lies full to (times, 0 as made = 1; 0.01 no flat full layer).
pub static LOOK_BANK_FALL: Config<f32> = Config::named("look_bank_fall");
pub static LOOK_BANK_EDGE: Config<f32> = Config::named("look_bank_edge");
pub static LOOK_BANK_TOP: Config<f32> = Config::named("look_bank_top");
/// The mist banks' march steps per pixel (0 as made = 8): the quality row's, Low 4 to Max 12; never
/// off (Bliss's VL_SAMPLES, 8).
pub static LOOK_MIST_STEPS: Config<f32> = Config::named("look_mist_steps");
/// The low morning sun's glow in the sky as Bliss's (1): one smooth Mie lobe (Henyey-Greenstein, g 0.8)
/// in the sun's colour over the sky's blue, in place of our ten hand-made layers round the sun, whose
/// windows ended together about 25-30° out as a hard seam (Planner, 11-25); 0 those layers. Past 0
/// it also scales the lobe's strength (tuning).
pub static LOOK_SKY_LOBE: Config<f32> = Config::named("look_sky_lobe");
/// The HDR world's picture as Bliss's chain (1): its one exposure curve, Purkinje, Hejl 2015 and
/// 0.3 of saturation, in place of our stack of per-hour fixes (0, to compare; Ronald, 11-25: "lets
/// better match this", change parts back later if wished).
pub static LOOK_CHAIN: Config<f32> = Config::named("look_chain");
/// The seen sky from the air's own scattering (1), as Bliss's method: Rayleigh, Mie and ozone,
/// worked out once a frame; 0 our hand colour sets and their per-hour fixes, to compare.
pub static LOOK_SKY_PHYS: Config<f32> = Config::named("look_sky_phys");
/// One fog as Bliss's (1): the mist, the rain's fog and the air's haze in one march per pixel before
/// TAA, in place of the froxel volume and the surfaces' haze; 0 ours before, to compare.
pub static LOOK_FOG_ONE: Config<f32> = Config::named("look_fog_one");
/// With the chain, the night's light and sky to this share (0.08 as made; 1 as before).
pub static LOOK_CHAIN_NIGHT: Config<f32> = Config::named("look_chain_night");
/// How strongly rain and snow are drawn over the HDR world's Bliss look, x their defaults (1 as
/// made: flakes as before, rain streaks lit as Bliss lights its particles; 0 Bliss's own, only the
/// mask for its bloomy fog).
pub static LOOK_WEATHER: Config<f32> = Config::named("look_weather");
/// Rain on the ground as before Bliss (1: darker wet ground, a sheen, puddles that grow) or as
/// Bliss's (0 as made: the ground only goes smooth, mirroring the sky and a sharp sun, all_solid's
/// rain smoothness; Planner 11-26, item 2 of the feature audit).
pub static LOOK_RAIN_DARK: Config<f32> = Config::named("look_rain_dark");
/// With Bliss's rain, the screen's reflections on the wet ground too (1 as made; the Extra tab's
/// "Wet reflections" row, `video.rs`).
pub static LOOK_RAIN_MIRROR: Config<f32> = Config::named("look_rain_mirror");
/// Bliss's caves with the HDR world (1 as made): its fog and air's light gone with the eye in a cave
/// (the engine's ten seconds without sky light, `Scene::cloud_shadow.z`) and its blue cave fog; 0
/// neither (feature audit item 4, 11-26).
pub static LOOK_CAVE_FOG: Config<f32> = Config::named("look_cave_fog");
/// A test switch for the A/B sheets against RTX Remix (0 as made, never shipped as a look): Remix's
/// auto exposure, local tonemap and neutral ACES curve in place of the Bliss chain's (`look.wgsl`'s
/// `remix_grade`; path tracing stage 3, 11-26).
pub static LOOK_REMIX_GRADE: Config<f32> = Config::named("look_remix_grade");
/// Bliss's soft sun shadows (Variable_Penumbra_Shadows, 1 as made): a blocker search sets each
/// pixel's filter from 5 to 30 by how far the shadow's caster stands (up to 35 blocks), then 13 taps
/// along its spiral; 0 our fixed 3x3 of the near map (feature audit item 5, 11-26).
pub static LOOK_PCSS: Config<f32> = Config::named("look_pcss");
/// Bliss's screen-space contact shadows toward the sun (SCREENSPACE_CONTACT_SHADOWS, 1 as made):
/// 16 steps of 3 pixels across last frame's solid world toward the light, the darker of it and the
/// map's, also past the map and on the far land; 0 the map alone (feature audit item 6, 11-26).
/// With temporal anti-aliasing on only (as SSAO).
pub static LOOK_CONTACT_SUN: Config<f32> = Config::named("look_contact_sun");
/// Bliss's SSAO (indirect_effect 1; 0 as made until its denoise pass: without it a fine weave
/// stays on the forest floor, lksp_ssao1-6, Planner 11-26): 7 taps (twice, over each 2x2) round
/// each point over last frame's solid world cap the sky's facing light and shade all but the sun by
/// the darker of the corner shading and SSAO^6, in place of the post passes' contact and far shade;
/// 0 those passes as before (feature audit item 6, 11-26). On the land lit from the sky only, and with temporal
/// anti-aliasing on (it reads last frame's depth through last frame's camera, which only TAA keeps).
pub static LOOK_SSAO: Config<f32> = Config::named("look_ssao");
/// The engine's map of the ground's height (`client::ground_map`), which the banks lie on: on with
/// them (`register`'s frame hook keeps it so, however the banks were set).
static GROUND_MAP: Config<bool> = Config::named("core:ground_map");
/// The mist clock: a dawn mist every morning whatever the air (the engine's `evening_mist_floor`),
/// 0 as made (only the air's own), 1 our dampest dawn mist, up to 3.
pub static LOOK_MIST_CLOCK: Config<f32> = Config::named("look_mist_clock");

/// "Bliss match" (Video, Extra): the look's knobs as fitted to Ronald's Bliss at the like-for-like
/// vista (stage 2, 11-22/23: the lsN to lsAA sweeps against bliss_ctl, crew notes
/// bliss-features-missing-1122.md), all switched together. Since 11-24 these are the knobs' own
/// defaults, the game's look (Ronald: "I don't want it to be off by default. Like our game will just
/// look like this", notes/STYLE.md): both columns hold the fitted value, so the switch changes
/// nothing until its Video row goes (the UI session). Each knob
/// keeps its own setting, so one can still be moved after.
pub static LOOK_BLISS: Config<bool> = Config::named("look_bliss");
/// Each knob, its value in the Bliss match, and as made.
pub static BLISS_MATCH: [(&Config<f32>, f32, f32); 22] = [
    // The shade: no glare lift, less fill, the sky light occluded as Bliss's (lsN_ao3).
    (&LOOK_GLARE, 0.0, 0.0),
    (&LOOK_FILL, 1.0, 1.0),
    (&LOOK_AO_POWER, 4.0, 4.0),
    // The evening's blue hour (lsZ_tw2, lsAA_tw3).
    (&LOOK_TWILIGHT, 0.5, 0.5),
    // Water: absorbed by depth, no foam, Bliss's mirror, a darker body (lsP_water).
    (&LOOK_WATER_ABSORB, 1.0, 1.0),
    (&LOOK_WATER_FOAM, 0.0, 0.0),
    (&LOOK_WATER_MIRROR, 1.0, 1.0),
    (&LOOK_WATER_BODY, 0.35, 0.35),
    // The air with the sun low: the near air's blue, the far haze keeping the sky's colour, the
    // sun's wide glare, the evening sky blue away from the sun (lsS_mix, lsX_air3, lsW_air2).
    (&LOOK_LOW_AIR, 0.75, 0.75),
    (&LOOK_HAZE, 1.6, 1.6),
    (&LOOK_HAZE_SUN, 0.0, 0.0),
    (&LOOK_SUN_GLOW, 1.0, 1.0),
    (&LOOK_EVENING_AWAY, 1.0, 1.0),
    // The evening sun kept red on the tops at 18:30, the shade under it darker (lsAL_lowsun3).
    (&LOOK_LOW_SUN_KEEP, 0.6, 0.6),
    (&LOOK_EVENING_SHADE, 0.5, 0.5),
    // The added airs half gone by 18:30, as Bliss's land dims (lsAM_airfade: 1 lost the shade's blue).
    (&LOOK_AIR_FADE, 0.5, 0.5),
    // The shade lit by the sky alone, no golden fill: 18:00 shade light lavender 113 102 118 (Bliss
    // 82 84 107, ours as made cream 186 159 130), 18:30 near land 37 38 73 (Bliss 35 37 70);
    // 08:00 the same, 19:00 outside the golden hours (lsAR_glow2).
    (&LOOK_GLOW_FILL, 0.0, 0.0),
    // The low sun a near-white disc with a wide glow (lv5, 11-24).
    (&LOOK_SUN_DISC, 1.0, 1.0),
    // The dawn mist as banks over the land, and their dawn tint (lv8-lv16, 11-24; 18:00 with them on
    // as before from lv14). They show where the air has mist: a dry dawn has none.
    (&LOOK_MIST_DOMES, 1.0, 1.0),
    (&LOOK_MIST_DAWN_TINT, 1.0, 1.0),
    // The sun's shadows to 128 blocks, then the sky light's mask (the condition matrix, 11-24).
    (&LOOK_SHADOW_REACH, 128.0, 128.0),
    // The sun's glow one smooth lobe, as Bliss's sky (11-25).
    (&LOOK_SKY_LOBE, 1.0, 1.0),
];

/// Switches the Bliss match on (every knob to its fitted value) or off (every knob as made).
pub fn set_bliss_match(on: bool) {
    for (knob, bliss, made) in BLISS_MATCH.iter() {
        knob.set(if on { *bliss } else { *made });
    }
    LOOK_BLISS.set(on);
}

/// The pack's `options` line: the settings in the order `look.wgsl` reads them.
pub const OPTIONS: &str = "base:look_exposure, base:look_sun, base:look_sun_warmth, base:look_fill, base:look_fill_cool, \
base:look_contrast, base:look_saturation, base:look_shade, base:look_haze, base:look_fog, base:night_light, base:night_moon, \
base:look_canopy, base:look_shafts, core:view_distance, base:look_storybook, base:fx_night_sky, base:fx_shooting_stars, \
base:moon_size, base:fx_glow, base:fx_biome_air, base:fx_sparkle, base:look_curve, base:look_bloom, base:look_auto_exposure, base:look_sky_light, base:look_foliage_test, base:look_white, base:look_ao_test, base:look_sharpen, base:look_leaf_sun, base:look_leaf_holes, base:look_leaf_light, base:look_debug, base:look_sky_hue, base:look_exposure_abs, base:look_split_warm, base:look_evening_mist, base:look_bloom_add, base:look_cirrus_off, base:look_glare, base:look_ao_power, base:look_water_absorb, base:look_water_foam, base:look_water_mirror, base:look_twilight, base:look_water_body, base:look_low_air, base:look_haze_sun, base:look_sun_glow, base:look_evening_away, base:look_mist_tint, base:look_low_sun_keep, base:look_evening_shade, base:look_air_fade, base:look_glow_fill, base:look_mist_dawn_tint, base:look_mist_domes, base:look_horizon_lift, base:look_sun_disc, base:look_dawn_sky, base:look_bank_dense, base:look_bank_sun, base:look_bank_shade, base:look_shadow_reach, base:look_bank_fall, base:look_bank_edge, base:look_bank_top, base:look_mist_steps, base:look_sky_lobe, base:look_chain, base:look_sky_phys, base:look_fog_one, base:look_chain_night, base:look_weather, base:look_rain_dark, base:look_rain_mirror, base:look_cave_fog, base:look_remix_grade, base:look_pcss, base:look_contact_sun, base:look_ssao";

/// Every setting here is on the Developer page (`developer::page`): players pick presets and
/// quality on the Video screen, the fitted values are tuned on its Developer tab (Ronald 11-24).
pub fn register(r: &mut Registry) {
    // By day: how bright the picture is taken, the sun's strength and warmth (-1 whiter, +1 toward
    // a 4300 K orange), the sky's fill light in the shade and how cool it is (-1 neutral, +1 bluer),
    // the curve's contrast, the colour after it, how fast the sky's light fades under cover (more:
    // darker forest floors), the haze over distance and the fog's light.
    crate::developer::page(r.config(&LOOK_EXPOSURE).float(1.0, 0.5, 2.0));
    crate::developer::page(r.config(&LOOK_SUN).float(1.0, 0.5, 2.0));
    crate::developer::page(r.config(&LOOK_SUN_WARMTH).float(0.0, -1.0, 1.0));
    crate::developer::page(r.config(&LOOK_FILL).float(1.0, 0.5, 2.0));
    crate::developer::page(r.config(&LOOK_FILL_COOL).float(0.0, -1.0, 1.0));
    crate::developer::page(r.config(&LOOK_CONTRAST).float(1.0, 0.8, 1.3));
    crate::developer::page(r.config(&LOOK_SATURATION).float(1.0, 0.7, 1.3));
    crate::developer::page(r.config(&LOOK_SHADE).float(1.0, 0.5, 2.0));
    // 1.6: Ronald's pick, "crisp + more depth" (11-05).
    crate::developer::page(r.config(&LOOK_HAZE).float(1.6, 0.0, 3.0));
    crate::developer::page(r.config(&LOOK_FOG).float(1.0, 0.0, 2.0));
    // By night (dark by design): the night's sky light and the moon's light.
    crate::developer::page(r.config(&NIGHT_LIGHT).float(1.0, 0.3, 2.0));
    crate::developer::page(r.config(&NIGHT_MOON).float(1.0, 0.3, 2.0));
    // Under a forest's canopy (how closed it is comes from the biome, `vitals::air::CANOPY`): how
    // dim and green the shade under the leaves is, with its sunflecks, and how strong the light
    // shafts through the crown gaps are.
    crate::developer::page(r.config(&LOOK_CANOPY).float(1.0, 0.2, 2.0));
    crate::developer::page(r.config(&LOOK_SHAFTS).float(1.0, 0.2, 3.0));
    // The HDR world's picture: 0 Lottes's curve (the look as made), 1 Hejl's (the default, Ronald's
    // pick of 11-05: "Hejl 2015"), 2 AgX; bloom 0 none to 3;
    // the exposure following the view, 0 fixed to 2.
    crate::developer::page(r.config(&LOOK_CURVE).float(1.0, 0.0, 2.0));
    crate::developer::page(r.config(&LOOK_BLOOM).float(1.0, 0.0, 3.0));
    crate::developer::page(r.config(&LOOK_AUTO_EXPOSURE).float(1.0, 0.0, 2.0));
    crate::developer::page(r.config(&LOOK_SKY_LIGHT).float(1.0, 0.2, 1.5));
    crate::developer::page(r.config(&LOOK_FOLIAGE_TEST).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_WHITE).float(1.6, -1.0, 2.5));
    crate::developer::page(r.config(&LOOK_AO_TEST).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_SHARPEN).float(0.4, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_LEAF_SUN).float(1.0, 0.5, 4.0));
    crate::developer::page(r.config(&LOOK_LEAF_HOLES).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_LEAF_LIGHT).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_DEBUG).float(0.0, 0.0, 14.0));
    crate::developer::page(r.config(&LOOK_SKY_HUE).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_EXPOSURE_ABS).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_SPLIT_WARM).float(0.7, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_EVENING_MIST).float(0.0, 0.0, 3.0));
    crate::developer::page(r.config(&LOOK_BLOOM_ADD).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_CIRRUS_OFF).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_GLARE).float(0.0, 0.0, 2.0));
    crate::developer::page(r.config(&LOOK_AO_POWER).float(4.0, 0.0, 8.0));
    crate::developer::page(r.config(&LOOK_WATER_ABSORB).float(1.0, 0.0, 4.0));
    crate::developer::page(r.config(&LOOK_WATER_FOAM).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_WATER_MIRROR).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_TWILIGHT).float(0.5, 0.0, 8.0));
    crate::developer::page(r.config(&LOOK_WATER_BODY).float(0.35, 0.0, 2.0));
    crate::developer::page(r.config(&LOOK_LOW_AIR).float(0.75, 0.0, 2.0));
    crate::developer::page(r.config(&LOOK_HAZE_SUN).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_SUN_GLOW).float(1.0, 0.0, 4.0));
    crate::developer::page(r.config(&LOOK_EVENING_AWAY).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_MIST_TINT).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_LOW_SUN_KEEP).float(0.6, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_EVENING_SHADE).float(0.5, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_AIR_FADE).float(0.5, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_GLOW_FILL).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_MIST_DAWN_TINT).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_MIST_DOMES).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_HORIZON_LIFT).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_SUN_DISC).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_DAWN_SKY).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_BANK_DENSE).float(0.0, 0.0, 5.0));
    crate::developer::page(r.config(&LOOK_BANK_SUN).float(0.0, 0.0, 5.0));
    crate::developer::page(r.config(&LOOK_BANK_SHADE).float(0.0, 0.0, 10.0));
    crate::developer::page(r.config(&LOOK_SHADOW_REACH).float(128.0, 0.0, 1024.0));
    crate::developer::page(r.config(&LOOK_BANK_FALL).float(0.0, 0.0, 4.0));
    crate::developer::page(r.config(&LOOK_BANK_EDGE).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_BANK_TOP).float(0.0, 0.0, 2.0));
    crate::developer::page(r.config(&LOOK_MIST_STEPS).float(0.0, 0.0, 16.0));
    crate::developer::page(r.config(&LOOK_SKY_LOBE).float(1.0, 0.0, 2.0));
    crate::developer::page(r.config(&LOOK_CHAIN).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_SKY_PHYS).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_FOG_ONE).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_CHAIN_NIGHT).float(0.08, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_WEATHER).float(1.0, 0.0, 10.0));
    crate::developer::page(r.config(&LOOK_RAIN_DARK).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_RAIN_MIRROR).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_CAVE_FOG).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_REMIX_GRADE).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_PCSS).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_CONTACT_SUN).float(1.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_SSAO).float(0.0, 0.0, 1.0));
    crate::developer::page(r.config(&LOOK_MIST_CLOCK).float(0.0, 0.0, 3.0));
    crate::developer::page(r.config(&LOOK_BLISS).bool(true));
    r.client_frame(|_| {
        let want = LOOK_MIST_DOMES.get() > 0.0;
        if GROUND_MAP.get() != want {
            GROUND_MAP.set(want);
        }
    });
}
