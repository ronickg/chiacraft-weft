// chiacraft's look, put in front of every file of the pack (`mods/base/src/shaderpack.rs`): the
// colors of the day, the sky, how light falls on things, the haze and the final grade. Everything
// is worked out in linear light above 1 where it is bright, and `display` brings it to the screen.

// ---- what kind of frame this is ----

// A dimension without sky light gets the sun straight up and nothing above the horizon
// (`Client::sun`): the pack then leaves its sky and fog to the engine.
fn open_sky() -> bool {
    return !(scene.sun.x == 0.0 && scene.sun.z == 0.0);
}

// With the eye in a liquid the engine's fog no longer starts at 0.6 of its end.
fn in_liquid() -> bool {
    return abs(scene.params.x - 0.6 * scene.params.y) > 0.5;
}

// How hard it rains, 0..1 (the engine puts it beside the fog's color).
fn rain_amount() -> f32 {
    // None in the calm-and-dry pipelines (`DRY`: the engine draws with them only when it is 0).
    if DRY {
        return 0.0;
    }
    return clamp(scene.fog_color.w, 0.0, 1.0);
}

// How wet the ground is where the player is, 0..1: it wets as rain falls and dries over minutes
// after (`scene.canopy.y`, the engine's, at the pace the base mod's air gives).
fn ground_wet() -> f32 {
    if DRY {
        return 0.0;
    }
    return clamp(scene.canopy.y, 0.0, 1.0);
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3(0.2126, 0.7152, 0.0722));
}

// What solid things write in the world image's alpha: nothing, so that what water, glass and ice
// blend over them leaves their own share there, a mask the reflections effect reads
// (`mods/base/assets/shaders/post/reflect.wgsl`). The screen never shows the alpha.
const SOLID_ALPHA: f32 = 0.0;

// ---- the look's knobs (`mods/base/src/look.rs`: settings in Mod Settings, `--config` for a run) ----
// `scene.options` holds them in this order; a setting the engine can't find reads 0, so a factor
// falls back to `missing` (1) and an offset to 0.
const OPT_EXPOSURE: u32 = 0u;
const OPT_SUN: u32 = 1u;
const OPT_SUN_WARMTH: u32 = 2u;
const OPT_FILL: u32 = 3u;
const OPT_FILL_COOL: u32 = 4u;
const OPT_CONTRAST: u32 = 5u;
const OPT_SATURATION: u32 = 6u;
const OPT_SHADE: u32 = 7u;
const OPT_HAZE: u32 = 8u;
const OPT_FOG: u32 = 9u;
const OPT_NIGHT_LIGHT: u32 = 10u;
const OPT_NIGHT_MOON: u32 = 11u;
const OPT_CANOPY: u32 = 12u;
const OPT_SHAFTS: u32 = 13u;
// The Fantasy group (round 17; `mods/base/src/video.rs`, rows UI made): read raw with `fantasy_opt`,
// as their 0 is off, where `look_opt` takes 0 for a setting not found.
const OPT_STORYBOOK: u32 = 15u;
const OPT_NIGHT_SKY: u32 = 16u;
const OPT_SHOOTING_STARS: u32 = 17u;
const OPT_MOON_SIZE: u32 = 18u;
const OPT_GLOW: u32 = 19u;
const OPT_BIOME_AIR: u32 = 20u;
const OPT_SPARKLE: u32 = 21u;
// The HDR world's picture (`base:look_curve`, `look_bloom`, `look_auto_exposure`), read raw: 0 is
// Lottes's curve (1 Hejl's, 2 AgX), no bloom, a fixed exposure.
const OPT_CURVE: u32 = 22u;
const OPT_BLOOM: u32 = 23u;
const OPT_AUTO_EXPOSURE: u32 = 24u;
// Gap tests (Planner's breakdown against Bliss, 11-05), read raw: the sky's light in the shade
// (`base:look_sky_light`, 1 as made), and grass and leaves darker and less yellow
// (`base:look_foliage_test`, 0 as made to 1: 30% darker).
const OPT_SKY_LIGHT: u32 = 25u;
const OPT_FOLIAGE_TEST: u32 = 26u;
// The HDR world's white balance after the curve (`base:look_white`, 0 neutral as made, + warmer).
const OPT_WHITE: u32 = 27u;
// A test (the tree diagnosis, 11-05): 1 takes the corner shade (AO) away, 0 as made.
const OPT_AO_TEST: u32 = 28u;
// The HDR world's sharpening (`base:look_sharpen`, 0 none to 1; 0.4 by default, Bliss's).
const OPT_SHARPEN: u32 = 29u;
// The canopy (HDR): the sun on leaves' lit faces (`base:look_leaf_sun`, 2) and their holes kept at
// a distance (`base:look_leaf_holes`, read raw: 1 on, 0 off).
const OPT_LEAF_SUN: u32 = 30u;
const OPT_LEAF_HOLES: u32 = 31u;
// The light through leaves (`base:look_leaf_light`, read raw: 0 none, 1 on).
const OPT_LEAF_LIGHT: u32 = 32u;
// A debug view (`base:look_debug`, read raw): 0 the picture, 1 the sun's light alone, 2 the sky's
// alone, both on white (no texture or tint), haze and curve as usual: Bliss's DEBUG_VIEW
// debug_DIRECT / debug_INDIRECT (composite1.fsh:1347-1354), to compare the light pixel by pixel.
const OPT_DEBUG: u32 = 33u;
// The HDR world's sky light by day in the upper sky's own hue, as Bliss's ambient (its sky averaged
// between the horizon and the zenith, normalised: `deferred.vsh:121-151`), `base:look_sky_hue` read
// raw: 0 the palette's fill as made, 1 all the way to the sky's hue.
const OPT_SKY_HUE: u32 = 34u;
// The HDR world's exposure by day in Bliss's own units (`base:look_exposure_abs`, read raw: 0 as
// made, 1 all the way): its curve fed our light x41 (our light is in units about 41x smaller than
// Bliss's, sun luma 1.77 against 72.7: Research's code read, 11-22), so its slope is Bliss's gentle
// one; without our 0.83 and the near land's dim. Still 1 at an average sunlit view.
const OPT_EXPOSURE_ABS: u32 = 35u;
// The HDR world's warmth on sunlit foliage (`base:look_split_warm`, read raw: 0.7 as made, 0
// none). With the 4300 K sun it counts the warmth twice (Bliss has no such grade).
const OPT_SPLIT_WARM: u32 = 36u;
// The HDR world's evening mist (`base:look_evening_mist`, read raw: 0 as made, 1 all the way): with
// the sun low in the west the mist lies as Bliss's evening fog does (`overworld_fog.glsl:16-35`,
// Ronald's 0.18 uniform + 0.09 cloudy), densest at the low land and halving every 6.7 blocks up, no
// sharp-topped layer: the 18:00 warm veil over the land (Research's code read, 11-22).
const OPT_EVENING_MIST: u32 = 37u;
// The HDR world's bloom by day added on top as Bliss's (`base:look_bloom_add`, read raw: 0 as made,
// mixed in keeping the light's sum; 1 all the way).
const OPT_BLOOM_ADD: u32 = 38u;
// A test switch for like-for-like compares (`base:look_cirrus_off`, read raw: 0 as made, 1 no high
// cirrus): with `core:soft_clouds` 0, `core:cloud_shadows` 0 and a blank clouds texture (mcclear)
// the sky then has no cloud at all, as Bliss's with VOLUMETRIC_CLOUDS off (Ronald, 11-22: "removing
// clouds could help with bliss vs ours").
const OPT_CIRRUS_OFF: u32 = 39u;
// The HDR world's veiling glare, a share of the sky's light over the whole picture
// (`base:look_glare`, read raw: 1 as made, 0 none). Bliss has none: its shade's lift is its added
// bloom (controlled frames, 11-22: with bloom and auto exposure off its noon shade is 0 19 65,
// ours 24 46 79).
const OPT_GLARE: u32 = 40u;
// The far shade from the sky as strong as Bliss's (`base:look_ao_power`, read raw by
// `post_low/far_ao.wgsl` as option 41: 0 as made).
const OPT_AO_POWER: u32 = 41u;
// Water as Bliss's (lookstage2, 11-22: Bliss's lakes dark navy with a smooth shore, ours light blue
// with cream foam and blotches). Absorption by depth (`base:look_water_absorb`): 0 as made, 1 Bliss's
// per-colour coefficients (water and dirt), floored dark; more is stronger.
const OPT_WATER_ABSORB: u32 = 42u;
// The foam at the shore and over the shallows (`base:look_water_foam`): 1 as made, 0 none (Bliss's).
const OPT_WATER_FOAM: u32 = 43u;
// The mirror (`base:look_water_mirror`): 0 as made; 1 Bliss's Fresnel (fifth power from 0.02), the
// sky mirrored where the ripples point it, no extra share at a slant.
const OPT_WATER_MIRROR: u32 = 44u;
// The evening's blue hour as Bliss's (`base:look_twilight`): 0 as made; above 0 the sky goes a deep,
// saturated blue from sunset to about 10° under, and the land is lit by that sky at this many times
// its light (Bliss's bliss_tw, 11-22: land 5 23 78 at 19:00, 0 10 75 at 19:30, dark at 20:00, where
// ours was grey-black, 10 13 21, at 19:00).
const OPT_TWILIGHT: u32 = 45u;
// The blue hour's end (with `OPT_TWILIGHT` on): full to about 15° under, gone by about 24°.
fn twilight_window() -> f32 {
    return smoothstep(-0.42, -0.26, scene.sun.y);
}
// How far under the horizon the sun must be before the night sky's stars show (`dark` from 0 to
// 1): with `OPT_TWILIGHT` on, Bliss's, none until about 16° under and all by 27° (none in its 20:00
// sky, the sun 16° under; ours from 19:00, 3° under; from 13° under a third were still out at 20:00,
// lsY_tw). `day_from`, `night_at` as made.
fn night_sky_dark(day_from: f32, night_at: f32) -> f32 {
    if fantasy_opt(OPT_TWILIGHT) > 0.0 {
        return smoothstep(-0.28, -0.45, scene.sun.y);
    }
    return smoothstep(day_from, night_at, scene.sun.y);
}
// How bright the water's own body is, near and far (`base:look_water_body`): 1 as made, less darker.
// Looking down at a lake the mirror is a few percent; the rest is the body's light, which kept our
// open water a light blue (lsL, 11-22: 102 123 133 against Bliss's dark navy 68 93 125).
const OPT_WATER_BODY: u32 = 46u;
// The blue near air's share with the sun low (`base:look_low_air`): 0 as made (a fifth), up to 1
// (all of it, as with the sun high), for Bliss's blue shade at 08:00 and 18:00.
const OPT_LOW_AIR: u32 = 47u;
// How much of the low sun's colour the far haze takes (`base:look_haze_sun`): 1 as made, 0 none.
// Bliss's far air at the like-for-like vista stays a cool white-blue with the sun low (lsT_haze,
// 11-22: 08:00 far band 151 155 168, ours 151 133 102; 18:00 122 123 150 against 143 143 125). The
// share was tuned on Bliss's own world (11-05/06), where its 08:00 far land read warm.
const OPT_HAZE_SUN: u32 = 48u;
fn haze_sun() -> f32 {
    let h = fantasy_opt(OPT_HAZE_SUN);
    return select(1.0, h, h >= 0.0);
}
// A wide glow round the sun in the air (`base:look_sun_glow`): 0 as made (none), more brighter.
// Bliss's 08:00 sun stands in a white glare some 40° across that washes the sky and the far land
// toward it (lsT_haze, 11-22); ours is a tight disc and a small gold halo.
const OPT_SUN_GLOW: u32 = 49u;
// The low sky away from the evening sun a clear blue (`base:look_evening_away`): 0 as made, 1 Bliss's.
const OPT_EVENING_AWAY: u32 = 50u;
// The mist with the sun low in the west lit in the light's own colours (`base:look_mist_tint`): 0 as
// made (the low sun's gold in the sunlit mist and the sky's light in it), 1 Bliss's: its fog takes
// the sun's colour and the sky's as they are (`overworld_fog.glsl`: light colour times phase, plus
// the ambient), so its evening layer reads pale pink-lavender where ours went orange (lsG_ev, 11-22).
const OPT_MIST_TINT: u32 = 51u;
// How much of the evening sun's light the blue hour's dimming keeps (`base:look_low_sun_keep`): 0 as
// made (a quarter at 4° up), 1 all of it. Bliss's 18:30 tops stand in its red sun (106 74 81) where
// ours were lit lavender by the sky (144 142 164, lsAH_mist).
const OPT_LOW_SUN_KEEP: u32 = 52u;
// The light in the shade with the evening sun under about 12° (`base:look_evening_shade`): 1 as
// made, less darker. Bliss's 18:30 shade is half ours (22 32 73 against 51 65 121, lsAH_mist).
const OPT_EVENING_SHADE: u32 = 53u;
// The air added over the land fading with the evening sun from 10° to 3° up (`base:look_air_fade`):
// 0 as made, 1 all of it gone by 3° and back from sunset, where the blue hour's air takes over.
// Bliss's 18:30 land is dim (mean 38 41 80) where our near and twilight airs kept the 18:00
// sky's strength (72 68 85 with the sun kept, lsAL_lowsun3).
const OPT_AIR_FADE: u32 = 54u;
// The golden hour's warm fill from the sky round the low sun (`base:look_glow_fill`): 1 as made, 0
// none. Bliss's shade light is the averaged sky alone (deferred.vsh:118-152), lavender-grey at 18:00
// (86 87 112 on white); ours with this fill went cream, 185 159 133 (lsAN_dbg18).
const OPT_GLOW_FILL: u32 = 55u;
// The dawn mist lit in the light's own colours (`base:look_mist_dawn_tint`): 0 as made (gold toward
// the sun, half the sky's light away from it), 1 as Bliss's, the sun's and the sky's colours as they
// are. Bliss's 06:00 mist is cream-white, 229 201 167, where ours was orange, 219 184 122
// (lsAO_mistnat).
const OPT_MIST_DAWN_TINT: u32 = 56u;
// The mist's banks as domes standing out of the layer (`base:look_mist_domes`): 0 as made (the top
// heaving 5 blocks either way), 1 up to 20 either way with a softer top, so the banks stand as lit
// domes with shaded sides, as Bliss's 06:00-07:00 mist does, where ours lay as a flat sheet
// (lsAO_mistnat).
const OPT_MIST_DOMES: u32 = 57u;
// The sky's haze never darker than the sky behind it (`LOOK_HORIZON_LIFT`).
const OPT_HORIZON_LIFT: u32 = 58u;
// The low sun's disc near white with a wide glow (`LOOK_SUN_DISC`).
const OPT_SUN_DISC: u32 = 59u;
// The dawn before sunrise as Bliss's (`LOOK_DAWN_SKY`).
const OPT_DAWN_SKY: u32 = 60u;
// The mist banks' density, sunlight and shade, each times its own (`LOOK_BANK_DENSE`, `_SUN`,
// `_SHADE`; 0 is 1, as made), for tuning them live against Bliss.
const OPT_BANK_DENSE: u32 = 61u;
const OPT_BANK_SUN: u32 = 62u;
const OPT_BANK_SHADE: u32 = 63u;
// How much of it this morning: the option, in the east, with the sun under the horizon (full from
// 1.7° down, none at 1° up) and not yet night (gone by about 12° down, where the night sky takes it).
fn dawn_sky() -> f32 {
    let k = clamp(fantasy_opt(OPT_DAWN_SKY), 0.0, 1.0);
    if k <= 0.0 {
        return 0.0;
    }
    let east = smoothstep(-0.05, 0.05, scene.sun.x);
    let under = 1.0 - smoothstep(-0.03, 0.02, scene.sun.y);
    return k * east * under * smoothstep(-0.3, -0.18, scene.sun.y);
}
// With `OPT_MIST_DOMES` at 1: how much higher the layer's top lies (blocks) and how much denser it is.
const MIST_DEEP: f32 = 0.0;
// The banks as Bliss's (lv8, 11-24, from its source, method only: `overworld_fog.glsl`): not domes
// on the ground but a level sea of mist at one height, that the hills and crowns rise out of. The
// domes followed the land and covered every crown, and read as paint on it (Planner on lv8).
// The sea's full height over sea level, and how fast it thins above it (per block, exp).
const MIST_SEA_LEVEL: f32 = 26.0;
const MIST_SEA_FALL: f32 = 0.12;
// Its density per block at `/air` 1 (Bliss's morning, about 0.45 a block where full, at `/air` 0.05).
const MIST_SEA_DENSE: f32 = 12.0;
// Its shape: a broad noise (cells this wide and tall, blocks) cut at a threshold (from, to: none
// below, full above), so banks with clear air between them, not an even veil; and a fine noise
// (cells this wide and tall) that eats their edges (`MIST_ERODE.z`, how much), shifted by the shape
// so the rims break up differently from bank to bank.
const MIST_SHAPE: vec4<f32> = vec4(36.0, 18.0, 0.35, 0.75);
const MIST_ERODE: vec3<f32> = vec3(1.5, 3.0, 0.4);
// Its light from the sun (Bliss's fog: no shade of its own, the land's and the clouds' shadow at each
// step): a sharp peak toward the sun and a broad forward tail, about 12 toward it, a quarter across,
// a fifteenth away (peak, its sharpness, tail, its fall), so the mist's sunny side toward a low sun
// glows and away from it the sky's light alone shows.
const MIST_SEA_PHASE: vec4<f32> = vec4(11.7, 30.0, 0.77, 1.2);
// The sun's light in it: how much of the low sun's orange goes to cream, how bright, and the most the
// forward peak may give (Bliss's sunlit tops are cream and brighter than the land; ours took the
// dawn sun's orange and read as sand, and from inside the mist the uncapped peak blew the sky
// round the sun out to a yellow ball: lv8).
const MIST_SEA_CREAM: vec3<f32> = vec3(0.45, 1.8, 3.0);
// Its light from the sky, brighter looking up through it (how much, how sharply), and its share (Bliss's
// shaded hollows are a dark blue-grey; at the full sky's light ours read as pale frost: lv8).
const MIST_SEA_SKY: vec3<f32> = vec3(5.0, 4.0, 0.45);
// What of the banks the evening keeps (Bliss's evening about a fifth of its morning).
const MIST_BANK_EVENING: f32 = 0.03;
const MIST_DEEP_DENSE: f32 = 1.2;
// With it, how much light the mist in the sun's shade keeps (Bliss's in shade is dark, the shafts
// show): of the bounce, and of the sky's.
const MIST_DEEP_SHADE: f32 = 0.2;
fn evening_air_fade() -> f32 {
    let k = clamp(fantasy_opt(OPT_AIR_FADE), 0.0, 1.0);
    if k <= 0.0 {
        return 1.0;
    }
    let evening = 1.0 - smoothstep(-0.05, 0.05, scene.sun.x);
    let w = (1.0 - smoothstep(0.05, 0.17, scene.sun.y)) * smoothstep(-0.03, 0.0, scene.sun.y);
    return 1.0 - k * w * evening;
}
fn water_body() -> f32 {
    let b = fantasy_opt(OPT_WATER_BODY);
    return select(1.0, b, b >= 0.0);
}

// How much of the mist lies as the evening's (`OPT_EVENING_MIST`): none with the sun 25° up in the
// west (17:00), a little at 12° (18:00), all with it on the horizon, into the dusk. (It began at 32°
// up and drowned the 17:00 land in orange, where Bliss's shows its golden layer only at sunset:
// lsG_ev against bliss_ctl, 11-22.)
fn evening_window() -> f32 {
    let west = smoothstep(0.1, -0.1, scene.sun.x);
    let low = 1.0 - smoothstep(0.03, 0.3, scene.sun.y);
    return west * low * smoothstep(-0.2, -0.05, scene.sun.y);
}
fn evening_mist() -> f32 {
    return clamp(fantasy_opt(OPT_EVENING_MIST), 0.0, 1.0) * evening_window();
}
// The mist the evening layer has of its own (the client's `evening_mist_floor`, the same sum): 0.3
// at 1, thicker past it up to 3 (at 1 it was a faint grey veil next to Bliss's, lsAH_mist).
fn evening_floor() -> f32 {
    return 0.3 * clamp(fantasy_opt(OPT_EVENING_MIST), 0.0, 3.0) * evening_window();
}

// The air's own mist (`scene.weather.z` less the evening layer's floor, `evening_mist_floor` in the
// client): what every mist but the evening layer's, the haze and the grade go by, so the evening
// layer brings no dawn mist, haze or grey with it (lsAF_mist03).
fn air_mist() -> f32 {
    return max(scene.weather.z - evening_floor(), 0.0);
}

// The surface's light from the sun and in all, before its tint (`light_on`, `surface_light`), for
// the debug view.
var<private> LIGHT_SUN: vec3<f32> = vec3(0.0);
var<private> LIGHT_ALL: vec3<f32> = vec3(0.0);

fn look_debug() -> u32 {
    return u32(round(fantasy_opt(OPT_DEBUG)));
}

// The colour a grass or leaf tint is multiplied by for the foliage test: 1 as made.
fn foliage_test() -> vec3<f32> {
    return mix(vec3(1.0), vec3(0.62, 0.72, 0.66), clamp(fantasy_opt(OPT_FOLIAGE_TEST), 0.0, 1.0));
}

fn fantasy_opt(i: u32) -> f32 {
    return scene.options[i / 4u][i % 4u];
}

fn look_opt(i: u32, missing: f32) -> f32 {
    let v = scene.options[i / 4u][i % 4u];
    return select(v, missing, v == 0.0);
}

// ---- under the leaves ----

// How closed the forest's canopy is where the player is (`scene.canopy`, from the biome's air:
// `vitals::air::CANOPY`, 1 a rainforest's), times its knob.
fn canopy() -> f32 {
    return clamp(scene.canopy.x * look_opt(OPT_CANOPY, 1.0), 0.0, 2.0);
}

// The light that comes down through leaves is green (they pass green and swallow red and blue):
// the sky's light on a forest floor, by how much of the sky the leaves cut (`sky_light` the sky
// light level, 0..1) and how closed the canopy is. Other cover (rock, a roof) cuts the sky too, but
// only in a forest is it leaves; the canopy number tells which.
const LEAF_LIGHT: vec3<f32> = vec3(0.6, 1.0, 0.5);

// The tint of that light on what stands under the leaves (`light_on`): neutral in the HDR world, as
// Bliss's (its sky light under a canopy is the sky's own colour, dimmed by the lightmap: crew notes
// bliss-leaves.md). Tinted, its birch trunks read green, sRGB 69 83 68 at noon against Bliss's
// 145 142 140, and the floor's grass a cyan-green, hue 98 against 91 (Trees' trb_s3_lk5: neutral
// 83 83 81 and 82). The fog's light through leaves keeps its green.
fn under_tint() -> vec3<f32> {
    return select(LEAF_LIGHT, vec3(1.0), scene.frame.x > 0.5);
}

fn under_leaves(sky_light: f32) -> f32 {
    // Not in deep cover (a cave in the forest): the leaves' light is from what the sky reaches.
    // Faint down to a sky light of about 0.05: from 0.35 the foot of a rainforest giant, between
    // its buttresses under its thick crown, went near black at noon (frame luma 0.03-0.06, the
    // world playtester's GIANT_F); a cave, with no sky light at all, stays dark.
    return min(canopy(), 1.0) * (1.0 - smoothstep(0.8, 1.0, sky_light)) * smoothstep(0.03, 0.3, sky_light);
}

// ---- the colors of the day ----

struct Palette {
    // The sky straight up and at the horizon.
    zenith: vec3<f32>,
    horizon: vec3<f32>,
    // The glow around the sun (or the moon).
    glow: vec3<f32>,
    // Direct light from the sun or the moon, and the light from the whole sky.
    sun: vec3<f32>,
    ambient: vec3<f32>,
}

fn key(z: vec3<f32>, h: vec3<f32>, g: vec3<f32>, s: vec3<f32>, a: vec3<f32>) -> Palette {
    return Palette(z, h, g, s, a);
}

fn blend(a: Palette, b: Palette, t: f32) -> Palette {
    return Palette(mix(a.zenith, b.zenith, t), mix(a.horizon, b.horizon, t), mix(a.glow, b.glow, t), mix(a.sun, b.sun, t), mix(a.ambient, b.ambient, t));
}

// The palette by how high the sun stands (`scene.sun.y`, about 0.82 at noon on the leaned path): day, golden hour,
// sunset, dusk and night, blended between.
// The palette, as the sky worked it out once this frame (`client::frame_look`: five texels, each a
// member), or worked out here without that (previews, a pack reload's first frame). Worked out
// here it was some 150 instructions in up to ten places of every pixel, and the Mac's world pass
// spilled registers (Tester's trace, 11-05).
// Read once per invocation and kept (`PAL`): each call read five texels of the frame's row again,
// and the far land's pixel calls it some ten times (the haze's two sky colours, the light, the
// water); on the Pixel 10 the world pass went 52.3 -> 45.0 ms at the vista, 42.0 -> 37.0 in the
// forest, the same values (Android port, 11-06).
var<private> PAL: Palette;
var<private> PAL_READ: bool = false;

fn palette() -> Palette {
    if !PAL_READ {
        PAL = palette_now();
        PAL_READ = true;
    }
    return PAL;
}

fn palette_now() -> Palette {
    var p = palette_read();
    // The sky's light test (`base:look_sky_light`): its light in the shade scaled, the sun's kept.
    // In the HDR world 0.45 of it as made in the morning (0.55 under a forest's canopy), 0.65 with
    // the sun high (the gap sheets:
    // our shade never went dark against Bliss's, the near forest at 08:00 luma 93 against 58; at
    // noon the greens then read dark, 41 against 50; agreed with Shader, 11-05, so today's approved
    // look keeps its keys).
    let k = fantasy_opt(OPT_SKY_LIGHT);
    let hdr_scale = mix(mix(0.45, 0.55, min(canopy(), 1.0)), 0.65, smoothstep(0.5, 0.85, scene.sun.y));
    p.ambient *= select(1.0, k, k > 0.0) * select(1.0, hdr_scale, scene.frame.x > 0.5);
    // A lightning flash isn't the sky's light in the shade: in the HDR world it comes back after
    // that scaling, and twice as strong (the HDR world has the range for a real flash): scaled
    // with it, a storm's flashes lifted the land by 2 where they had lifted it by 46 (PC st2_e
    // against st_e, 11-05).
    if scene.frame.x > 0.5 {
        let night = smoothstep(-0.05, -0.25, scene.sun.y);
        p.ambient += vec3(0.5, 0.55, 0.7) * lightning_flash() * mix(1.0, 0.5, night) * (2.0 - hdr_scale);
    }
    // And its sunlight a little warmer with the sun high (the warmth on Bliss's noon land is its
    // sun's, the sky and the far haze staying blue: our whole-picture white balance had greyed the
    // noon sky, saturation 19 against 31).
    // In the HDR world the sunlight's colour as Bliss makes it: a blackbody's (Ronald's Bliss runs
    // its sun at 4300 K) through the air along the sun's path, at the palette's brightness. Bliss's
    // lit canopy tops read yellow (hue 52 at 08:00, 62 at noon), ours green (79, 100) under a
    // near-white sun (Trees' measures, 11-06). (It replaces the small warming with the sun high.)
    if scene.frame.x > 0.5 && scene.sun.y > -0.05 {
        let hue = sun_kelvin_linear() * sky_transmittance(scene.sun.y);
        let day = smoothstep(-0.05, 0.05, scene.sun.y);
        p.sun = mix(p.sun, hue * (luma(p.sun) / max(luma(hue), 1e-4)), day);
    }
    // The sky's light in the upper sky's hue (`base:look_sky_hue`): our fill, a pale near-white blue
    // (white tops lit by the sky alone 192 195 204 where Bliss's are 163 186 210), diluted the
    // 4300 K sun on leaf tops and kept them green (the debug view, lsA_d2 against Bliss's
    // debug_INDIRECT, 11-22). By day only; its brightness kept.
    let sky_hue = clamp(fantasy_opt(OPT_SKY_HUE), 0.0, 1.0);
    if scene.frame.x > 0.5 && sky_hue > 0.0 {
        let day_h = smoothstep(0.0, 0.12, scene.sun.y);
        let upper = p.zenith * 0.65 + p.horizon * 0.35;
        let hue = upper * (luma(p.ambient) / max(luma(upper), 1e-4));
        p.ambient = mix(p.ambient, hue, day_h * sky_hue);
    }
    // In the HDR world the twilight's light grows as the sun nears the horizon, as real twilight's
    // does (about tenfold over its last few degrees): the dusk key held one level from 8 degrees
    // under to 1, so 05:00 was as dark as 04:00 (land sRGB 25 24 30 where Bliss's is 32 70 125,
    // its 04:00 dark; Mist's m11 sheet, 11-06). Dimmer at the night's end, brighter at the sun's.
    if scene.frame.x > 0.5 {
        // Mornings only: the evening's twilight has its own lift (the palette's evening light and
        // the post's), and this on top took the 19:00 land to a bright teal-green (PC shprA).
        let morning = smoothstep(-0.05, 0.05, scene.sun.x);
        let twilight = smoothstep(-0.3, -0.14, scene.sun.y) * (1.0 - smoothstep(-0.02, 0.06, scene.sun.y)) * morning;
        p.ambient *= mix(1.0, mix(0.7, TWILIGHT_RISE, smoothstep(-0.16, -0.02, scene.sun.y)), twilight);
    }
    // The evening's blue hour (`OPT_TWILIGHT`): from the sun 7° up to 10° under, fading out by 17°
    // under; the sky's own colour pushed to a deep blue (its brightness kept), and the light in the
    // shade that blue sky's, at the setting's strength.
    let blue_k = max(fantasy_opt(OPT_TWILIGHT), 0.0);
    if blue_k > 0.0 {
        let evening = 1.0 - smoothstep(-0.05, 0.05, scene.sun.x);
        // To about 24° under (20:30 on our clock): Bliss's 20:00, the sun 16° under, is still a deep
        // navy (sky 0 3 113, land 0 0 21), where ours had gone grey night (it ended at 17° under).
        let tw = twilight_window() * (1.0 - smoothstep(0.02, 0.12, scene.sun.y)) * evening * (1.0 - rain_amount());
        let deep = vec3(0.08, 0.3, 1.0);
        // Nearly all of it (Bliss's 20:00 sky is a pure navy, 0 4 114; at seven tenths ours kept a
        // grey in it, 36 56 106, lsY_tw).
        p.zenith = mix(p.zenith, deep * (luma(p.zenith) / luma(deep)), tw * 0.95);
        p.horizon = mix(p.horizon, mix(p.horizon, deep * (luma(p.horizon) / luma(deep)), 0.8), tw);
        let sky = p.zenith * 0.65 + p.horizon * 0.35;
        // The land lit by it only once the sun is down: from 4° up it turned the 18:30 land a
        // bright green (lsO_tw2: 125 136 88 at 4, where Bliss's is already dim, 35 36 69).
        let down = 1.0 - smoothstep(-0.04, 0.0, scene.sun.y);
        p.ambient = mix(p.ambient, sky * blue_k, tw * down);
        // The evening sun dims over its last 10° (the long path through the air): Bliss's 18:30
        // land, the sun 4° up, is already dim and blue-lit, 35 36 69, where ours stood in its full
        // light, 69 61 44 (lsS_mix after f3cfb0f84). At 4° up about two fifths, none off by 10°.
        // (From 2° under, two fifths at 18:30 took the land only 70 62 48 -> 66 60 60, lsY_tw: now a
        // quarter.)
        let keep = clamp(fantasy_opt(OPT_LOW_SUN_KEEP), 0.0, 1.0);
        p.sun *= mix(1.0, mix(smoothstep(0.0, 0.2, scene.sun.y), 1.0, keep), evening * (1.0 - rain_amount()));
    }
    // The evening's shade darker with the sun low (`OPT_EVENING_SHADE`): from 12° up to the horizon,
    // gone again by 2° under, where the blue hour's sky lights it.
    let shade_k = fantasy_opt(OPT_EVENING_SHADE);
    if scene.frame.x > 0.5 && shade_k >= 0.0 {
        let evening = 1.0 - smoothstep(-0.05, 0.05, scene.sun.x);
        let low = (1.0 - smoothstep(0.1, 0.22, scene.sun.y)) * smoothstep(-0.04, 0.02, scene.sun.y);
        p.ambient *= mix(1.0, shade_k, low * evening * (1.0 - rain_amount()));
    }
    return p;
}

// How much brighter the twilight's sky light is just before the sun is up than as the dusk key made it.
const TWILIGHT_RISE: f32 = 2.6;

// A 4300 K blackbody in linear light, red 1 (Bliss's `blackbody` at Ronald's Sun_temp, computed in
// the crew's bliss-source notes: linear 1.0 0.668 0.426).
fn sun_kelvin_linear() -> vec3<f32> {
    return vec3(1.0, 0.668, 0.426);
}

// What of the sun's light gets through the air to the ground with the sun `e` (sin of its height)
// up: Rayleigh's scattering by wavelength and a little grey Mie, over the air mass along the path
// (Kasten and Young's), from Bliss's own coefficients (Rayleigh 5.8e-6, 1.35e-5, 3.31e-5 per metre
// over an 8 km scale height; Mie 3e-6 over 1.2 km). About 0.94 0.87 0.72 at noon (55 degrees up).
fn sky_transmittance(e: f32) -> vec3<f32> {
    let h = degrees(asin(clamp(e, 0.0, 1.0)));
    let air = 1.0 / (sin(radians(h)) + 0.50572 * pow(h + 6.07995, -1.6364));
    let depth = vec3(0.0464, 0.108, 0.265) + vec3(0.0036);
    return exp(-depth * air);
}

fn palette_read() -> Palette {
    if scene.frame_look.x > 0.5 {
        let t = textures[u32(scene.frame_look.x) - 1u];
        return Palette(
            textureLoad(t, vec2(0, 0), 0).rgb,
            textureLoad(t, vec2(1, 0), 0).rgb,
            textureLoad(t, vec2(2, 0), 0).rgb,
            textureLoad(t, vec2(3, 0), 0).rgb,
            textureLoad(t, vec2(4, 0), 0).rgb,
        );
    }
    return palette_made();
}

fn palette_made() -> Palette {
    let e = scene.sun.y;
    // Day, after Bliss's own colours (reports in the crew notes, bliss-source.md): a muted blue
    // overhead, about sRGB 50 92 148 (it was 6 139 237), going to a light grey-blue at the horizon
    // (155 178 202), as Bliss's default sky; not a saturated #4A90E2 blue, and not the grey-beige
    // of Bliss with Ronald's 4300 K sun, which read overcast and olive here (Tester 2). Values for
    // the day's exposure of 2.2 (`exposure`). A warm sun; the sky's light in the shade about a
    // quarter of the sun's (it was a third; Bliss's is a sixth, which our curve's toe crushes to
    // near black), still blue, so sun and shade part and the land stops looking flat. Then half as
    // bright again (Colour 6): shade under leaves and on walls still read near black by day (Art's
    // regraded tiles, the sun/shade test's walls at sRGB 33 35 45).
    // (Colour 8, reports/Daytime look targets.md: values for the day's Lottes curve at exposure 1.3.
    // A sky a little brighter than before, about sRGB 118 157 196 overhead and 169 188 205 at the
    // horizon; a 6300 K sun (1.0 0.91 0.81, Bliss's); a neutral-cool fill, blue 1.4x red, which
    // `light_on` gives a direction.)
    // The sun's warmth (-1 whiter, +1 toward a 4300 K orange) and the fill's coolness (-1 neutral,
    // +1 bluer) from the knobs, each kept about as bright.
    let warmth = look_opt(OPT_SUN_WARMTH, 0.0);
    let sun_hue = select(mix(vec3(1.0, 0.91, 0.81), vec3(1.0, 0.97, 0.94), -warmth), mix(vec3(1.0, 0.91, 0.81), vec3(1.0, 0.74, 0.52), warmth), warmth >= 0.0);
    let cool = look_opt(OPT_FILL_COOL, 0.0);
    let fill_hue = select(mix(vec3(0.85, 0.95, 1.2), vec3(1.0, 1.0, 1.0), -cool), mix(vec3(0.85, 0.95, 1.2), vec3(0.7, 0.92, 1.45), cool), cool >= 0.0);
    let day = key(vec3(0.137, 0.289, 0.612), vec3(0.378, 0.538, 0.797), vec3(1.0, 0.92, 0.78), sun_hue * 1.92 * look_opt(OPT_SUN, 1.0), fill_hue * 0.3 * look_opt(OPT_FILL, 1.0));
    // Golden hour is orange, not white: a warm horizon, an orange glow and sun.
    // Golder sunrises and sunsets with the sun just up (0-12°): horizons and glow toward gold-yellow,
    // less red-violet (the Look session's Bliss checkpoint, 11-05: ours read pink-lavender at
    // 05:00-07:00 where Bliss's are gold). The pinks and violets stay for the dusk, under the horizon.
    // (Before: golden horizon 0.92 0.62 0.42, glow 1.0 0.58 0.26; sunset horizon 1.05 0.46 0.34,
    // glow 1.0 0.50 0.26, ambient 0.27 0.22 0.36.)
    // (A first pass, sunset horizon 1.05 0.58 0.30, hardly showed on the forest vista: PC shv_gold.)
    let golden = key(vec3(0.11, 0.25, 0.78), vec3(1.0, 0.78, 0.38), vec3(1.0, 0.70, 0.24), vec3(1.55, 0.88, 0.36), vec3(0.26, 0.25, 0.30));
    let sunset = key(vec3(0.08, 0.10, 0.38), vec3(1.08, 0.66, 0.26), vec3(1.0, 0.64, 0.20), vec3(1.50, 0.82, 0.34), vec3(0.30, 0.25, 0.27));
    // The blue hour deep blue, as Bliss's (its land at 05:00 warmth -88, ours violet, about 0: the
    // MC comparison's spring 8 sheets, 11-05), a pink band left low on the horizon.
    // (Before: zenith 0.045 0.05 0.16, horizon 0.30 0.15 0.27, ambient 0.10 0.09 0.17.)
    // (v4: the shade light a deeper, more saturated blue, 0.03 0.07 0.25: at 0.05 0.08 0.20 the land
    // read grey-violet, saturation 81-102 against Bliss's 189-254, Look's measures.)
    let dusk = key(vec3(0.02, 0.05, 0.20), vec3(0.22, 0.15, 0.30), vec3(0.45, 0.20, 0.25), vec3(0.0, 0.0, 0.0), vec3(0.03, 0.07, 0.25));
    // Night is dark and blue, a little lighter than vanilla's, so lanterns carry the scene.
    // Night is dark and blue, so firelight's orange stands out against it. (A blue-teal shade, as
    // Valheim's, mixed with the orange of a fire into olive on the ground: Tester 2's campfire.)
    // (Its horizon a little lighter, so ridges stand against it: 0.025 0.035 0.070 before.)
    let night = key(vec3(0.006, 0.010, 0.030), vec3(0.032, 0.044, 0.085), vec3(0.10, 0.13, 0.22), vec3(0.17, 0.21, 0.30) * look_opt(OPT_NIGHT_MOON, 1.0), vec3(0.012, 0.017, 0.030) * look_opt(OPT_NIGHT_LIGHT, 1.0));
    // The morning and the late afternoon between: a gold sun and a warm, cream horizon, the shade
    // still blue, so the day runs gold, warm, clear at noon, warm, gold. Golden hour ended 16° up
    // (sun.y 0.28), and on the leaned path of a long summer day the sun is over that from 05:00 to
    // 19:00: those hours all looked alike (Tester 2's hours sheet, 11-05). Now gold to about 20° up,
    // warm to about 51°, clear above that (about 08:00 to 16:00 in summer).
    // (Its warmth on the land, a gold sun, the horizon still a light blue: with a cream horizon,
    // 0.62 0.58 0.56, and a paler gold sun, 05:00 and 19:00 went a pale cream haze and 06:00-07:00
    // barely changed, PC run sh3.)
    // (Variant B: a deeper gold sun, the horizon and the shade a little warm too.)
    let warm = key(vec3(0.13, 0.27, 0.66), vec3(0.54, 0.53, 0.62), vec3(1.0, 0.68, 0.38), vec3(1.0, 0.68, 0.38) * 2.0 * look_opt(OPT_SUN, 1.0), mix(fill_hue, vec3(1.0, 0.9, 0.85), 0.25) * 0.3 * look_opt(OPT_FILL, 1.0));
    var p: Palette;
    if e > 0.4 {
        // A gentle warmth through mid-morning and mid-afternoon, clear only about noon (the MC
        // comparison's spring 8 sheets, 11-05: Bliss's land stays warmer than its noon to 10:00,
        // ours was full day by 10:00). On the equinox's leaned path sun.y is 0.55 at 08:00, 0.76 at
        // 10:00, 0.82 at noon: a little warm left at 10:00 and 14:00, none at noon.
        // (It was smoothstep(0.4, 0.78): all day from about 09:30.)
        p = blend(warm, day, smoothstep(0.4, 0.86, e));
        // And the sun itself a little warmer still until near noon (variant B: the blend alone
        // moved the land's warmth by under 0.01, PC run wh_aft): blue about 10% down at 10:00 and
        // 14:00 on the equinox, none at noon.
        let morning = 1.0 - smoothstep(0.6, 0.86, e);
        p.sun *= mix(vec3(1.0), vec3(1.0, 0.88, 0.72), morning);
    } else if e > 0.08 {
        p = blend(golden, warm, smoothstep(0.08, 0.34, min(e, 0.34)));
    } else if e > 0.01 {
        p = blend(sunset, golden, smoothstep(0.01, 0.08, e));
    } else if e > -0.14 {
        // The blue hour before sunrise and after sunset: the land in dusk's deep blue until the sun
        // is about at the horizon, the sky's horizon band and glow going gold a little earlier, as
        // the sun nears it (the MC comparison's spring 8 sheets and the Look session's measures,
        // 11-06: Bliss lets warm light onto the land only once the sun is up, its land at -3° warmth
        // -88; ours at 05:00 was sunrise-gold, +68).
        // (It was one blend over -0.14..-0.02, then -0.08..-0.02, then -0.05..-0.02: v1-v3.)
        p = blend(dusk, sunset, smoothstep(-0.02, 0.01, e));
        let band = blend(dusk, sunset, smoothstep(-0.08, -0.01, e));
        p.horizon = band.horizon;
        p.glow = band.glow;
    } else {
        p = blend(night, dusk, smoothstep(-0.3, -0.14, e));
    }
    // In the HDR world an evening sun still up (about 3-20 degrees) leaves the sky blue, as Bliss's
    // 18:00 (warmth -30), not the golden key's gold horizon, which ours stood in (+49 over the
    // horizon, +37 with the cream clouds; the 18:00 sun is 10-11 degrees up, as Minecraft's). Mornings
    // keep their gold (the dawn pick), the sunset itself its colours (under about 3 degrees). The sky's
    // own colours only, at their brightness: the sun and the land's light stay golden.
    if scene.frame.x > 0.5 {
        let evening = smoothstep(0.05, -0.05, scene.sun.x) * smoothstep(0.03, 0.1, e) * (1.0 - smoothstep(0.3, 0.42, e));
        if evening > 0.0 {
            let sky_blue = vec3(0.72, 0.86, 1.1);
            let hz = sky_blue * (luma(p.horizon) / luma(sky_blue));
            p.horizon = mix(p.horizon, hz, evening * 0.7);
            let zn = vec3(0.30, 0.48, 1.0) * (luma(p.zenith) / luma(vec3(0.30, 0.48, 1.0)));
            p.zenith = mix(p.zenith, zn, evening * 0.6);
        }
        // And the evening's twilight, the sun from about 2 degrees up to 8 under: Bliss's sky stays
        // blue (19:00 warmth -35) and its land a lit blue-grey (luma 86, 75 87 103), where ours went a
        // pink-lavender sky over near-black land (46) in the dusk key; at the same sun depth in the
        // morning Bliss's land is a dark deep blue (55, -85), which the blue hour keeps (PC sheb4
        // against Bliss's vista, 11-06; our 19:00 sun 2.9 degrees down, Minecraft's 3.5).
        let twilight = smoothstep(0.05, -0.05, scene.sun.x) * smoothstep(-0.16, -0.08, e) * (1.0 - smoothstep(0.0, 0.04, e));
        if twilight > 0.0 {
            let hb = vec3(0.75, 0.95, 1.3);
            p.horizon = mix(p.horizon, hb * (luma(p.horizon) / luma(hb)), twilight * 0.8);
            let zb = vec3(0.32, 0.5, 1.0);
            p.zenith = mix(p.zenith, zb * (luma(p.zenith) / luma(zb)), twilight * 0.7);
            // (0.42 0.52 0.72, about three times: at 0.15 0.19 0.27 the 19:00 land came only from 46 to
            // 51 in its colour, Bliss's 86; PC shtw1.)
            p.ambient = mix(p.ambient, vec3(0.42, 0.52, 0.72), twilight * 0.8);
        }
    }
    // A new moon's night is darker than a full moon's.
    // The night follows the moon (Ronald, 11-04: "if the moon is light, then it can be lighter, and
    // if there's no moon, it should be darker"): under a full moon high up a dim cool light to
    // walk by, darker as it sets or wanes, near black with no moon up (a torch needed).
    // (Full and high 3x the night's sky light, new or down a quarter: at 1.8x a full moon's night
    // measured only a little brighter than a new moon's, luma 0.115 against 0.100.)
    // (Night 4, Ronald "a bit lighter": a full moon high 6x, so shapes read and you can walk by it,
    // grass about luma 77 in the grade model; a new moon still a quarter, near black, a torch needed.
    // Its light less blue than it was, 0.17 0.21 0.30: blue 3.4x red turned stone a saturated blue.)
    // (Moonlit nights, Ronald's pick of 11-05: with no moon up, a quarter left the land pure black
    // under the stars from 22:00 to 02:00 at the vista, the night curve's toe swallowing it. 1.6:
    // the land's shapes, the water and the ridges against the sky faintly there, still far darker
    // than a full moon's night, with its direct light, and a torch still needed. (At 2.5 the land
    // read as moonlit, mean luma 26 against 9, the clouds glowing: PC run sh5.)
    p.ambient *= mix(1.0, mix(1.6, 6.0, scene.moon.w), smoothstep(-0.14, -0.3, e));
    // Overcast: grey and dim, and the sun no more than a brighter part of the sky. A thunderstorm
    // darker again, its sky a moody slate (Bliss's rain: the land darker but in its colours, the
    // sky dark blue-grey). With the sky's light 0.4 of the sun's the land went a bright flat grey
    // (Ronald, 11-05: "looks very grey weird").
    let r = rain_amount();
    let storm = scene.weather.w;
    let grey = vec3(luma(p.horizon)) * vec3(0.9, 0.94, 1.0);
    p.zenith = mix(p.zenith, grey * (0.72 - 0.32 * storm), r * 0.85);
    p.horizon = mix(p.horizon, grey * (1.0 - 0.45 * storm), r * 0.85);
    p.glow = p.glow * (1.0 - r * 0.8);
    // (A little dimmer, a storm more so, with more of it from above in `light_on`: lit evenly from
    // all round, rain's land read flat; Tester 2's weather sheet, 11-05.)
    p.ambient = mix(p.ambient, vec3(luma(p.ambient + p.sun * 0.2)) * vec3(0.93, 0.97, 1.02) * (1.0 - 0.42 * storm), r * 0.85);
    p.sun = p.sun * (1.0 - r * 0.9);
    // At night rain and its clouds darken the sky's light too (by day overcast only greys it).
    p.ambient *= mix(1.0, 1.0 - 0.6 * r, smoothstep(-0.14, -0.3, e));
    // A lightning flash lights the land and the sky, a cold white, for its strokes.
    let flash = lightning_flash();
    // At night half as much: at the day's strength a near strike lit the night's land brighter than
    // a storm's noon (PC run sh2_thunder_burst, frame luma 119 against 112).
    p.ambient += vec3(0.5, 0.55, 0.7) * flash * mix(1.0, 0.5, smoothstep(-0.05, -0.25, e));
    p.zenith += vec3(0.22, 0.24, 0.32) * flash;
    p.horizon += vec3(0.3, 0.32, 0.42) * flash;
    // A storybook hour (round 17, rec. 2; `base:look_storybook`, 0 to 1): at golden hour and blue
    // hour the shade goes lavender (Solas's 205 184 236) while the sun stays orange, the blue
    // hour's zenith violet; at night the moon's light silver-blue (a 15000 K blackbody, Bliss's),
    // not muddy green. Clear skies only: rain keeps its greys.
    let tale = clamp(fantasy_opt(OPT_STORYBOOK), 0.0, 1.0) * (1.0 - r);
    if tale > 0.0 {
        // Blue hour: the sun just under the horizon, to about 14° below (it fell short of the
        // 18:57 check at -0.2..-0.08).
        let blue_hour = smoothstep(0.04, -0.02, e) * smoothstep(-0.32, -0.16, e);
        // (`golden()` by hand: `palette` has a local of that name.)
        let gold_hour = (1.0 - smoothstep(0.08, 0.35, e)) * smoothstep(-0.06, 0.03, e);
        let hour = max(gold_hour, blue_hour);
        // At 0.92 0.82 1.18 the lavender hardly showed at a sheet's size (the storybook check, 11-05).
        p.ambient = mix(p.ambient, vec3(luma(p.ambient)) * vec3(0.95, 0.72, 1.4) * 1.15, tale * hour * 0.9);
        p.horizon = mix(p.horizon, vec3(luma(p.horizon)) * vec3(1.08, 0.8, 1.15), tale * hour * 0.45);
        p.zenith = mix(p.zenith, vec3(luma(p.zenith)) * vec3(0.8, 0.55, 1.6), tale * blue_hour * 0.8);
        let moonlit = smoothstep(-0.14, -0.3, e);
        p.sun = mix(p.sun, vec3(luma(p.sun)) * vec3(0.78, 0.9, 1.16), tale * moonlit);
        p.ambient = mix(p.ambient, vec3(luma(p.ambient)) * vec3(0.82, 0.92, 1.12), tale * moonlit * 0.6);
    }
    return p;
}

// How much of golden hour and sunset there is: 0 by day and at night, 1 with the sun just above the
// horizon. The grade warms with it, and the glowing sky lights the ground.
fn golden() -> f32 {
    let e = scene.sun.y;
    // To about 26° up (it was 20°): the golden hours run longer at both ends.
    return (1.0 - smoothstep(0.1, 0.55, e)) * smoothstep(-0.06, 0.03, e);
}

// The moon's light by its phase: 1 full, 0 new (`scene.eye.w`, the engine's).
fn moon_light() -> f32 {
    return clamp(scene.eye.w, 0.0, 1.0);
}

// Where direct light comes from: the sun while it is up, then the moon opposite it.
fn light_dir() -> vec3<f32> {
    // At night the moon where it stands (`scene.moon`: it lags the day by its phase), not simply
    // opposite the sun.
    return select(scene.moon.xyz, scene.sun.xyz, scene.sun.y > -0.06);
}

// How much of the palette's direct light there is: it fades out as the sun sets and the moon comes
// up a little after.
// ---- sky events (`scene.sky_event`: a blood moon, meteors, a fireball, an aurora storm) ----
// Round 17, rec. 6, with gameplay (Ronald, "Gameplay too"): the base mod's schedule says when
// (`vitals::events`), the pack draws them; none when `base:fx_sky_events` is off (nothing is sent).

// How strong the sky event of `kind` (`weft_abi::sky_event`) is now, 0 when another or none.
fn sky_event(kind: f32) -> f32 {
    return select(0.0, clamp(scene.sky_event.y, 0.0, 1.0), abs(scene.sky_event.x - kind) < 0.5);
}

// The blood moon, 0..1: under 0.5 its copper dusk, 1 totality.
fn blood_moon() -> f32 {
    return sky_event(1.0);
}

// The moon's colour on a blood moon night: copper as it reddens, a deep red in totality.
fn blood_tint() -> vec3<f32> {
    let b = blood_moon();
    return mix(vec3(1.0), mix(vec3(1.0, 0.45, 0.2), vec3(0.75, 0.06, 0.02), smoothstep(0.5, 1.0, b)), smoothstep(0.0, 0.35, b));
}

// How far over the blocky clouds' height the soft layer's underside stands (blocks), less as rain
// comes (a storm's deck hangs low): cumulus three hundred blocks across at the blocky clouds'
// height, 150 blocks over a forest floor, filled every gap in the canopy with one flat grey base,
// looking up from under the trees (Tester 2's looks4, 11-05); higher, they stand in the sky as
// clouds with blue between them, as Bliss's do.
const SOFT_LIFT: f32 = 220.0;
// How far a look into the soft clouds is followed (blocks): beyond, they fade into the sky. Their
// depth is held there, so what stands at this distance is cloud, never land (`sky_or_cloud`).
const SOFT_REACH: f32 = 4000.0;

// ---- lightning (`scene.lightning`, the engine's flash where the player is) ----
// Ronald, 11-05: "add lightning flashes" (Tester 2's four storm shots showed none: the engine's
// flash only lifted the daylight for 0.15 s, which the pack's light never read). A flash is two to
// four strokes over half a second, each a sharp rise and a fast fade, as real return strokes; it
// lights the land and the deck, and two strikes in three show their bolt in the sky, the rest a
// glow inside the clouds. Where and how far is the flash's own random number.

fn lightning_stroke(t: f32, at: f32, k: f32, fade: f32) -> f32 {
    return select(0.0, k * exp(-(t - at) / fade), t >= at);
}

// The strokes of the flash, each fading over `fade` seconds.
fn lightning_strokes(fade: f32) -> f32 {
    let t = scene.lightning.x;
    if t > 1.2 || scene.lightning.z < 0.05 {
        return 0.0;
    }
    let r = scene.lightning.y;
    var f = lightning_stroke(t, 0.0, 1.0, fade) + lightning_stroke(t, 0.06 + 0.06 * fract(r * 7.3), 0.75, fade);
    f += select(0.0, lightning_stroke(t, 0.2 + 0.1 * fract(r * 13.7), 0.6, fade), fract(r * 3.1) > 0.3);
    f += select(0.0, lightning_stroke(t, 0.42 + 0.15 * fract(r * 29.1), 0.45, fade), fract(r * 5.7) > 0.55);
    return f * lightning_near() * smoothstep(0.05, 0.4, scene.lightning.z);
}

// How bright the flash is now, 0 (none) to about 1.5 at a stroke, times how near it struck.
fn lightning_flash() -> f32 {
    return lightning_strokes(0.055);
}

// How near the strike is, 1 close to 0.35 far.
fn lightning_near() -> f32 {
    return mix(1.0, 0.35, fract(scene.lightning.y * 47.3));
}

// Toward the strike along the horizon (a unit vector, x and z). Two strikes in three come down
// within about 40° of where the camera looked as the flash began: anywhere round
// the compass, a quarter of them were in view and a burst of 400 shots caught flashes but no bolt
// (Tester 2's looks3, 11-05). The rest anywhere, so storms still flash all round.
fn lightning_toward() -> vec2<f32> {
    let r = scene.lightning.y;
    var a = r * 6.2831853;
    // Where the camera faced as the flash began (`scene.lightning.w`, the engine's), so the bolt
    // stays put while the player turns (Ronald, 11-05: it followed the camera).
    if fract(r * 61.3) < 0.67 {
        a = scene.lightning.w + (fract(r * 23.1) - 0.5) * 1.4;
    }
    return vec2(sin(a), -cos(a));
}

// How far off the strike is (blocks).
fn lightning_far() -> f32 {
    return mix(250.0, 1200.0, fract(scene.lightning.y * 47.3));
}

// The bolt in front of land that lies beyond it (`haze_lit`): drawn in the sky alone, the land
// covered all of it below the horizon and only a short piece showed between the horizon and the
// deck (PC shbo2_bolts). Nothing while no flash is on.
fn lightning_bolt_over(dir: vec3<f32>, dist: f32) -> vec3<f32> {
    if scene.lightning.x > 1.2 || scene.lightning.z < 0.05 || dist < lightning_far() {
        return vec3(0.0);
    }
    return lightning_bolt(dir);
}

// In the HDR world the strike lights the land and the air round where it hits (a bolt's flash is
// thousands of times the night; the whole-sky flash is the deck's): camera-relative `rel`, a cold
// white falling off over about 180 blocks from the strike, at the place `lightning_bolt` draws it.
// None for a strike inside the clouds, or without the HDR world (`palette`'s even flash alone).
fn lightning_glow_at(rel: vec3<f32>) -> vec3<f32> {
    let r = scene.lightning.y;
    if scene.frame.x < 0.5 || fract(r * 91.7) < 0.33 {
        return vec3(0.0);
    }
    let f = lightning_flash();
    if f <= 0.0 {
        return vec3(0.0);
    }
    let at = lightning_toward() * lightning_far();
    let d = length(rel.xz - at);
    let near = 180.0 * 180.0 / (180.0 * 180.0 + d * d);
    return vec3(0.75, 0.8, 1.0) * f * 3.0 * near;
}

// The bolt in the sky along `dir` (linear light): a jagged line from the deck's underside down to
// the ground where it struck, a few hundred to two and a half thousand blocks off, with one branch;
// none for a strike inside the clouds.
fn lightning_bolt(dir: vec3<f32>) -> vec3<f32> {
    // The channel glows on a little after each stroke's flash (0.12 s against 0.055), as a real
    // bolt's afterimage does: it showed for a frame or two only.
    let f = lightning_strokes(0.12);
    let r = scene.lightning.y;
    if f <= 0.0 || fract(r * 91.7) < 0.33 {
        return vec3(0.0);
    }
    // Nearer than before (400-2500) and from the deck's underside, the soft one 220 higher, so the
    // bolt stands tall in view: it was a short squiggle at the horizon (PC shbo_bolts).
    let far = lightning_far();
    let base = scene.clouds.z + select(0.0, SOFT_LIFT * (1.0 - 0.7 * max(min(rain_amount() * 0.75 + scene.weather.w * 0.25, 1.0), scene.weather.w)), scene.soft_clouds.x > 0.5);
    let top = atan2(max(base, 40.0), far);
    let bottom = atan2(-max(scene.eye.y - SEA_LEVEL, 2.0), far);
    let el = asin(clamp(dir.y, -1.0, 1.0));
    let u = (el - bottom) / max(top - bottom, 1e-3);
    if u < 0.0 || u > 1.0 {
        return vec3(0.0);
    }
    // The angle along the horizon from the strike, and the bolt's wander from it as it comes down.
    let toward = lightning_toward();
    let flat = normalize(vec2(dir.x, dir.z) + vec2(1e-5, 0.0));
    let side = toward.x * flat.y - toward.y * flat.x;
    let across = atan2(side, dot(toward, flat)) * cos(el);
    let wander = (noise3(vec3(u * 5.0, r * 97.0, 1.0)) - 0.5) * 0.07 + (noise3(vec3(u * 19.0, r * 53.0, 2.0)) - 0.5) * 0.02;
    let dx = across - wander * (top - bottom) * 2.5;
    // A branch splitting off a third of the way down, drifting off to one side.
    let fork = clamp((0.68 - u) / 0.3, 0.0, 1.0) * select(0.0, 1.0, u < 0.68 && u > 0.38);
    let dx2 = dx - (0.68 - u) * select(-0.09, 0.09, fract(r * 17.0) > 0.5) * (top - bottom) * 2.0;
    let w = 0.0011;
    let core = exp(-(dx / w) * (dx / w)) + 0.5 * fork * exp(-(dx2 / w) * (dx2 / w));
    let glow = exp(-abs(dx) / 0.012) * 0.35 + fork * exp(-abs(dx2) / 0.012) * 0.15;
    let fade = smoothstep(0.0, 0.06, u) * smoothstep(1.0, 0.92, u);
    return vec3(0.82, 0.86, 1.0) * (core * 9.0 + glow) * f * fade;
}

// The flash inside the deck along `dir`: brightest toward the strike, all of it lit a little.
fn lightning_in_clouds(dir: vec3<f32>) -> vec3<f32> {
    let f = lightning_flash();
    if f <= 0.0 {
        return vec3(0.0);
    }
    let flat = normalize(vec2(dir.x, dir.z) + vec2(1e-5, 0.0));
    let toward = dot(flat, lightning_toward()) * 0.5 + 0.5;
    return vec3(0.62, 0.66, 0.85) * f * (0.18 + 0.9 * pow(toward, 6.0));
}

fn light_strength() -> f32 {
    let e = scene.sun.y;
    // At night the moon's light: its phase times how high it stands (`scene.moon.w`), none when it
    // is down or new.
    // In a blood moon's totality the night is as dark as a new moon's (round 17's design).
    let moon = scene.moon.w * mix(1.0, 0.12, smoothstep(0.5, 1.0, blood_moon()));
    return select(smoothstep(0.06, 0.2, -e) * moon, smoothstep(-0.04, 0.05, e), e > -0.06);
}

// ---- the sky ----

fn sun_glow(dir: vec3<f32>, p: Palette) -> vec3<f32> {
    let mu = max(dot(dir, scene.sun.xyz), 0.0);
    let horizon = 1.0 - clamp(dir.y, 0.0, 1.0);
    // Less of the wide glow in golden hour, so the sky round a low sun stays orange instead of
    // washing out to white.
    let wide = pow(mu, 5.0) * (0.2 + 0.28 * horizon) * (1.0 - 0.35 * golden());
    let near = pow(mu, 60.0) * 0.75;
    return p.glow * (wide + near);
}

// The sky in `dir` (unit, world): horizon to zenith, the warm side toward the sun at sunset and the
// purple opposite, the glow round the sun, a faint one round the moon.
fn sky_color(dir: vec3<f32>) -> vec3<f32> {
    let p = palette();
    let h = clamp(dir.y, 0.0, 1.0);
    var c = mix(p.horizon, p.zenith, pow(h, 0.4));
    let mu = dot(dir, scene.sun.xyz);
    let low = 1.0 - smoothstep(0.0, 0.2, abs(scene.sun.y));
    // (This and the moon's and the rain's parts below are skipped for the whole frame where they
    // add nothing: the same colour, fewer powers per pixel; Android port, 11-06.)
    if low > 0.0 {
        // Away from the sun the sunset sky goes rose and violet.
        // Most of it once the sun is down (the blue hour); with the sun up a little only (it tinted
        // the whole sunrise sky lavender).
        let away = (0.5 - 0.5 * mu) * (1.0 - h) * low * mix(0.35, 1.0, smoothstep(0.03, -0.05, scene.sun.y));
        c = mix(c, c * vec3(0.97, 0.80, 1.06) + vec3(0.03, 0.0, 0.03), away * 0.4);
        // Toward it, a warmer band low on the horizon.
        let toward = pow(max(mu, 0.0), 2.0) * pow(1.0 - h, 3.0) * low;
        c += vec3(0.55, 0.22, 0.05) * toward;
    }
    // (B5: on the sky's own colour, before the sun's glow and glare are added: on top of them it
    // turned the glare gold, and the dawn mist that reads it went flat orange over the land, PC
    // shdawnB3/B4.)
    if scene.frame.x > 0.5 {
        // With the sun low, the whole of its half of the sky goes gold-cream at its own brightness,
        // as Bliss's dawn: ours stayed lavender-blue there round a gold glare (Bliss's own world at
        // 06:00, sky band saturation 105 and warmth +85 against Bliss's 122 and +94; Look, 11-06).
        // Its own window, full with the sun up to about 9 degrees and gone by about 20 (the sunset
        // band's `low` is nearly 0 by Bliss's 06:00 sun at 10 degrees, so this did nothing, PC
        // shdawnB2).
        // Mornings only (the sun rises at +x): at 18:00, facing the setting sun, it warmed the band
        // over the horizon where Bliss's evening is a blue-grey (Look's measures, 11-06).
        let morning = (1.0 - smoothstep(0.15, 0.35, scene.sun.y)) * smoothstep(-0.03, 0.02, scene.sun.y) * smoothstep(-0.05, 0.05, scene.sun.x);
        if morning > 0.0 {
            // (That orange, 231 172 68 against Bliss's 201 172 108 at 06:00, was the dawn mist,
            // which the script left free and which came and went between runs: with it pinned off
            // a third at 1.0 0.87 0.66 only halved the lavender gaps, 6.7 % to 3.3 %; PC shdawnm0,
            // shdawnm5.)
            let cream = vec3(1.0, 0.84, 0.58);
            // Over the sun's whole half, overhead too (about a third there): a lobe about the sun
            // alone left the gaps between the clouds lavender (7 % of the 06:00 sky band, Bliss's
            // none; PC shdawn0).
            let wide = 0.5 + 0.5 * mu;
            let dawn = pow(wide, 1.5) * morning * (1.0 - rain_amount());
            c = mix(c, cream * (luma(c) / luma(cream)), dawn * 0.7);
        }
    }
    // In the HDR world by day the band just over the horizon darker and a blue-grey, not the sky's
    // brightest part: in Bliss's own world 2-6 degrees up reads 164-177 at noon and 152-158 at 17-18,
    // ours 187-208, a flat warm cream at 17-18, and the far land fogged into it as bright (Look's
    // measures, mist off, 11-06). Here, so the land's haze that reads this colour comes down with it.
    if scene.frame.x > 0.5 {
        let band = 1.0 - smoothstep(0.0, 0.13, dir.y);
        let day = smoothstep(0.0, 0.1, scene.sun.y) * (1.0 - rain_amount());
        let k = band * day;
        // A clear blue, not a grey (H2: grey-tinted it reached Bliss's brightness, 160-172 against
        // 154-173, but stayed pale and from 16:00 warm, saturation 5-48 against 39-83; PC shhb1).
        let blue = vec3(luma(c)) * vec3(0.74, 0.92, 1.2);
        // On mornings with the sun low, looking away from it, lighter and bluer: at spot C 08:00
        // (facing away) the band read 172 / warmth -13 where Bliss's is 183 / -31, and the far lake,
        // nearly all a mirror of it, went flat grey (Far view's find). Toward the sun as it was (the
        // forest vista stays warm, Bliss's +27 there).
        let away = smoothstep(0.2, -0.4, mu);
        let low_morning = smoothstep(-0.05, 0.05, scene.sun.x) * (1.0 - smoothstep(0.45, 0.7, scene.sun.y));
        let aw = away * low_morning;
        c = mix(c, blue, k * mix(0.6, 0.85, aw)) * (1.0 - mix(0.4, 0.15, aw) * k);
        // The evening's side away from the sun (`OPT_EVENING_AWAY`): Bliss's sky low in the east at
        // 18:00 is a clear blue, 90 129 186 two to five degrees up, where ours stayed a grey-cream,
        // 143 145 151, and the far land hazed into it (lsV_air, 11-23). A deep blue at a little under
        // the sky's brightness, over the low sky (to about 20° up), most straight away from the sun.
        let eve_k = max(fantasy_opt(OPT_EVENING_AWAY), 0.0);
        if eve_k > 0.0 {
            let evening = (1.0 - smoothstep(-0.05, 0.05, scene.sun.x)) * (1.0 - smoothstep(0.3, 0.55, scene.sun.y)) * day;
            let low_sky = 1.0 - smoothstep(0.0, 0.35, dir.y);
            let deep = vec3(0.48, 0.69, 1.0);
            let e = clamp(smoothstep(0.3, -0.5, mu) * evening * low_sky * eve_k, 0.0, 1.0);
            c = mix(c, deep * (luma(c) * 0.85 / luma(deep)), e);
        }
    }
    // Before sunrise (`OPT_DAWN_SKY`): a deep blue at a little over half the sky's brightness, with a
    // thin pink line low toward where the sun will rise; the sun's glow and glare not yet up.
    let dawn = dawn_sky();
    if dawn > 0.0 {
        let deep = vec3(0.2, 0.42, 1.0);
        let line = (1.0 - smoothstep(0.0, 0.035, dir.y)) * pow(max(mu, 0.0), 3.0);
        let pink = vec3(1.0, 0.5, 0.62);
        let base = luma(c) * 0.6;
        c = mix(c, mix(deep * (base / luma(deep)), pink * (base * 1.3 / luma(pink)), line), dawn);
    }
    let glow_up = 1.0 - dawn;
    c += sun_glow(dir, p) * glow_up;
    // In the HDR world the air round the sun glows as bright as it really is (Mie scattering's
    // forward peak, a wide lobe and a tight one in the sun's own colour, strongest low), which the
    // curve rolls off and the bloom spreads: Bliss's low sun stands in a great gold glare. The
    // haze over the land toward the sun takes it too (it reads this colour).
    if scene.frame.x > 0.5 {
        let m = max(mu, 0.0);
        let mie = (pow(m, 8.0) * 0.45 + pow(m, 64.0) * 2.2) * (0.35 + 0.65 * low) * (1.0 - rain_amount());
        // In a gold of the sun's own brightness (Mie scattering keeps the low sun's colour), and a
        // third weaker: in the sun's pale mid-morning colour the glow went cream-white through the
        // curve's shoulder (08:00 sky saturation 23 against Bliss's 43, warmth +15 against +36).
        let gold = vec3(1.0, 0.72, 0.4) * luma(p.sun) * 1.4;
        c += mix(p.sun, gold, 0.6 * (1.0 - smoothstep(0.6, 0.9, scene.sun.y))) * mie * 0.34 * glow_up;
        // The wide glow (`OPT_SUN_GLOW`), near white (forward Mie scattering by haze and droplets
        // keeps most of the light's colour, the eye sees it whitish), the more so with the sun low.
        // The far land toward the sun takes it through the haze, which reads this colour.
        let wide_k = max(fantasy_opt(OPT_SUN_GLOW), 0.0);
        if wide_k > 0.0 {
            // Gone by noon (Bliss's noon sky by the sun 112 149 203; with a third of it left ours went
            // 113 -> 126 red, lsW_air2), full under about 27° up (sun.y 0.45), where 08:00 stands.
            let not_high = 1.0 - smoothstep(0.45, 0.8, scene.sun.y);
            let wide = (pow(m, 3.0) * 0.7 + pow(m, 12.0) * 0.3) * not_high * (1.0 - rain_amount());
            c += mix(vec3(luma(p.sun)), p.sun, 0.3) * light_strength() * wide * 0.5 * wide_k * glow_up;
        }
    }
    // The glow about the moon where it stands, as bright as its light (`scene.moon.w`): on a new
    // moon there is none, and a glow alone read as a dark-blue ring (the world playtester's nights).
    if scene.sun.y < -0.05 && scene.moon.w > 0.0 {
        let moon = pow(max(dot(dir, scene.moon.xyz), 0.0), 24.0) * smoothstep(-0.05, -0.25, scene.sun.y) * scene.moon.w;
        // On a blood moon its glow goes copper, then red.
        c += vec3(0.05, 0.07, 0.12) * mix(vec3(1.0), blood_tint() * vec3(2.0, 1.4, 1.0), smoothstep(0.0, 0.4, blood_moon())) * moon;
    }
    // Rain's sky low down, and the mist over the land that takes this colour (`haze_lit`), a darker
    // slate, darker in a storm: the deck fades into it at the horizon and the land's mist meets it
    // there, one colour (a brighter band of sky lay between a storm's deck and the land, and the
    // deck's edge drew a hard line along it: Tester 2's rain and thunder shots, 11-05).
    if rain_amount() > 0.0 {
        let wet = rain_amount() * (1.0 - smoothstep(0.0, 0.35, dir.y));
        c *= mix(vec3(1.0), vec3(0.76, 0.8, 0.86) - vec3(0.12) * scene.weather.w, wet);
    }
    if dir.y < 0.0 {
        // Below the horizon it darkens a little toward the ground the haze stands on.
        c = mix(c, p.horizon * 0.6 + p.ambient * 0.2, smoothstep(0.0, 0.4, -dir.y));
    }
    return c;
}

// ---- the sun's shadows (the engine's shadow map, `pack.cfg` sun_shadows) ----

// How much of the sun reaches `rel` (camera relative) on a surface facing `n`, from the shadow
// map: x 1 lit to 0 in shadow (a 3x3 of texels softens the edge), y how far to trust it, 1 well
// inside the map and fading to 0 at its edge and where it holds nothing this frame (the caller
// then guesses as it would without one).
// One cascade of the sun's shadows: its matrix `m`, `info` (bindless index, size, 1 when it holds
// this frame's, the blocks a texel covers), its round reach and depth (blocks).
// `cheap`: four texels filtered (a 2x2 of the map, weighed by where the point lies among them)
// instead of the 4x4 and its nine taps, for the far land, where a far-map texel is blocks wide and
// the cracks the 4x4 closes don't show (Distant terrain's probe: the far cascade's 16 loads were
// 0.21 ms of the far land at 3456).
fn shadow_in(m: mat4x4<f32>, info: vec4<f32>, round: f32, depth: f32, rel: vec3<f32>, n: vec3<f32>, cheap: bool) -> vec2<f32> {
    if info.z < 0.5 {
        return vec2(1.0, 0.0);
    }
    let texel = info.w;
    // Off the surface along its normal so it does not shade itself, twice as far where the light
    // grazes it (Complementary's and Photon's normal offset), and a hair along the light.
    let ndl = clamp(dot(n, light_dir()), 0.0, 1.0);
    let c = m * vec4(rel + n * texel * 1.2 * (2.0 - ndl), 1.0);
    let ndc = c.xyz / c.w;
    // The nearer of the box's edge and its round reach, so the corners fade too.
    let reach = max(max(abs(ndc.x), abs(ndc.y)), length(rel.xz) / round);
    if reach > 0.97 || ndc.z <= 0.0 || ndc.z >= 1.0 {
        return vec2(1.0, 0.0);
    }
    let size = i32(info.y);
    // Rows run down the map as y runs up the light's view: the engine's shaders draw with y up, the
    // way the screen effects read the screen (`0.5 - ndc.y * 0.5`). Read the other way up, every
    // shadow was mirrored across the camera's row of the map, and one side of open ground lay in
    // the mirror image of the ground itself.
    let at = vec2<i32>(vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * f32(size));
    let index = u32(info.x);
    // A few hundredths of a block along the light, more on surfaces the light grazes, in the map's
    // depth: no floating shadows at the foot of things.
    let slope = min(sqrt(max(1.0 - ndl * ndl, 0.0)) / max(ndl, 0.05), 4.0);
    let bias = (0.03 + texel * 0.5 * slope) / depth;
    let low = smoothstep(0.02, 0.06, light_dir().y);
    if cheap {
        let p = vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * f32(size) - 0.5;
        let base = vec2<i32>(floor(p));
        let f = fract(p);
        var lit4 = vec4(0.0);
        for (var k = 0; k < 4; k++) {
            let t = clamp(base + vec2(k % 2, k / 2), vec2(0), vec2(size - 1));
            lit4[k] = select(0.0, 1.0, textureLoad(textures[index], t, 0).r <= ndc.z + bias);
        }
        let lit = mix(mix(lit4.x, lit4.y, f.x), mix(lit4.z, lit4.w, f.x), f.y);
        return vec2(lit, (1.0 - smoothstep(0.85, 0.97, reach)) * low);
    }
    // A 4x4 of the map about the point; each of the 3x3 taps takes the nearest to the light of its
    // texel and the ones right and below. That closes the one-texel cracks between neighbouring
    // meshes, through which sunlight would draw thin dotted lines across a shadow. Read a row at a
    // time, each row's neighbouring pairs kept as a vec3: as a 16-float array indexed in loops, the
    // Mac's compiler kept it in memory (the world pass's register spill, Tester's trace 11-05).
    var lit = 0.0;
    var above = vec3(0.0);
    for (var j = 0; j < 4; j++) {
        let y = clamp(at.y + j - 1, 0, size - 1);
        let x0 = clamp(at.x - 1, 0, size - 1);
        let x1 = clamp(at.x, 0, size - 1);
        let x2 = clamp(at.x + 1, 0, size - 1);
        let x3 = clamp(at.x + 2, 0, size - 1);
        let row = vec4(
            textureLoad(textures[index], vec2(x0, y), 0).r,
            textureLoad(textures[index], vec2(x1, y), 0).r,
            textureLoad(textures[index], vec2(x2, y), 0).r,
            textureLoad(textures[index], vec2(x3, y), 0).r,
        );
        // Reverse depth: what the light meets first is the larger.
        let pairs = max(row.xyz, row.yzw);
        if j > 0 {
            let nearest = max(above, pairs);
            lit += dot(select(vec3(0.0), vec3(1.0), nearest <= vec3(ndc.z + bias)), vec3(1.0));
        }
        above = pairs;
    }
    lit /= 9.0;
    // Near its edge it hands over to the caller's guess, and so it does as the light nears the
    // horizon (long smeared shadows, then the swap from the sun to the moon).
    return vec2(lit, (1.0 - smoothstep(0.85, 0.97, reach)) * low);
}

// How much of the sun reaches `rel` (camera relative) facing `n`: the near cascade's map (80 blocks
// about the player, fine) where it holds, the far one's (400 blocks, coarser) beyond, handing over
// smoothly; x 1 lit to 0 in shadow, y how far to trust it.
fn shadow_at(rel: vec3<f32>, n: vec3<f32>) -> vec2<f32> {
    let near = shadow_in(scene.shadow_view_proj, scene.shadow, 80.0, 400.0, rel, n, false);
    // With traced shadows (`scene.rt`) the near map holds the models alone and the far one the
    // terrain: what doesn't trace (mobs, the mist) takes the darker of both.
    if scene.rt.x > 0.5 {
        let far_reach = max(scene.shadow_far.w * scene.shadow_far.y * 0.5, 1.0);
        let far = shadow_in(scene.shadow_far_view_proj, scene.shadow_far, far_reach, far_reach * 3.0, rel, n, length(rel) > 250.0);
        let trust = max(near.y, far.y);
        if trust <= 0.0 {
            return vec2(1.0, 0.0);
        }
        return vec2(min(mix(1.0, near.x, near.y), mix(1.0, far.x, far.y)), far.y);
    }
    if near.y >= 0.999 {
        return near;
    }
    // Past 250 blocks (the far land's tiles start about 180 out), the far map's cheap filter.
    // Its reach from its texels (`core:shadow_far`: 400 as made, more at ultra), its depth three times that.
    let far_reach = max(scene.shadow_far.w * scene.shadow_far.y * 0.5, 1.0);
    let far = shadow_in(scene.shadow_far_view_proj, scene.shadow_far, far_reach, far_reach * 3.0, rel, n, length(rel) > 250.0);
    let trust = max(near.y, far.y);
    if trust <= 0.0 {
        return vec2(1.0, 0.0);
    }
    return vec2(mix(far.x, near.x, near.y / trust), trust);
}

// With traced shadows (`scene.rt`): the shade of the models (mobs, the player) at `rel` facing
// `n`, from the near map that then holds them alone; 1 out of it.
fn models_shadow(rel: vec3<f32>, n: vec3<f32>) -> f32 {
    let m = shadow_in(scene.shadow_view_proj, scene.shadow, 80.0, 400.0, rel, n, false);
    return mix(1.0, m.x, m.y);
}

// The sun on a mob or a model at `rel` facing `n` with sky light `sky`: the shadow map where it
// reaches (they stand in the shadows of trees and houses), the sky light alone elsewhere.
fn sun_on_body(rel: vec3<f32>, n: vec3<f32>, sky: f32) -> f32 {
    let guess = smoothstep(0.8, 1.0, sky);
    let mapped = shadow_at(rel, n);
    return mix(guess, min(mapped.x, smoothstep(0.3, 0.8, sky)), mapped.y) * cloud_shadow(rel);
}

// ---- the clouds' shadows (`scene.clouds`, the sky's cloud layer) ----

// The slab is at least this thick (blocks): `sky.wgsl`'s clouds (CLOUD_THICKNESS there).
const CLOUD_SLAB: f32 = 9.0;

// 1 where cell `cell` (picture cells from `origin`, the camera's) is cloud: the picture's
// (bindless `tex`, `dims` cells), and as the weather fills them (`fill`) more of the gaps, a hash
// per cell so they grow as whole cells. The sky's clouds and their shadows both read this.
fn cloud_cell_of(tex: u32, cell: vec2<f32>, origin: vec2<f32>, dims: vec2<f32>, fill: f32) -> f32 {
    let at = fract((cell + origin + vec2(0.5)) / dims);
    if textureSampleLevel(textures[tex], nearest_clamp, at, 0.0).a >= 0.5 {
        return 1.0;
    }
    let w = vec2<i32>(cell + origin);
    return select(0.0, 1.0, hash3(vec3(w.x, 51, w.y)) < fill);
}

// Fuzz for the clouds' edges at picture cell `cell` (absolute: the camera's cell plus the place
// from it), -0.5..0.5: noise about a cell and a third across, so the cells' straight sides break
// into soft, uneven puffs. The sky's clouds and their shadows both take it, only where a cloud's
// edge is (its middle and the open sky ask no noise). (Softer clouds, Ronald's pick of 11-05: flat
// slabs with hard edges and big flat polygons for shadows on the desert, Tester 2's hours sheet.)
fn cloud_fuzz(cell: vec2<f32>) -> f32 {
    let w = fract(cell / 4096.0) * 4096.0;
    return noise3(vec3(w.x * 0.75, 7.3, w.y * 0.75)) - 0.5;
}

// How much of the sun the clouds let through to `rel` (camera relative): 1 under open sky, about
// a fifth under a cloud's middle, soft at its edges. The point is followed toward the light to the
// slab's middle, where the four nearest cells are blended as the sky draws them, so the shadows
// are the clouds overhead, drifting with them across near and far land alike (Bliss's moving
// shade on the hills).
// The soft clouds' shadow map (`client::cloud_shadow`, the HDR world): this many blocks across,
// about the camera, on the plane through the cloud layer's middle (its height over the camera in
// every texel's g), r how much of the sun gets through there.
const CLOUD_SHADOW_SPAN: f32 = 4096.0;

// The soft clouds' shadow at `rel` from their map, 1 out of it or without it.
fn soft_cloud_shadow(rel: vec3<f32>) -> f32 {
    let l = light_dir();
    if l.y < 0.03 {
        return 1.0;
    }
    let t = textures[u32(scene.cloud_shadow.x) - 1u];
    let half = i32(scene.cloud_shadow.y) / 2;
    let mid = textureLoad(t, vec2(half, half), 0).g;
    let q = rel.xz + l.xz * (mid - rel.y) / l.y;
    let uv = q / CLOUD_SHADOW_SPAN + 0.5;
    if any(uv < vec2(0.0)) || any(uv > vec2(1.0)) {
        return 1.0;
    }
    let through = textureSampleLevel(t, linear_clamp, uv, 0.0).r;
    // Fading in as the light climbs: long low shadows from far clouds would smear across the land.
    return mix(1.0, through, smoothstep(0.03, 0.15, l.y));
}

fn cloud_shadow(rel: vec3<f32>) -> f32 {
    // In the HDR world the soft clouds' own shadow, where their map is (Bliss's cloud masses throw
    // shadow patches across the land; the blocky picture's matched nothing in the sky).
    if scene.frame.x > 0.5 && scene.cloud_shadow.x > 0.5 {
        return soft_cloud_shadow(rel);
    }
    let c = scene.clouds;
    let l = light_dir();
    if c.x < 0.5 || l.y < 0.03 {
        return 1.0;
    }
    let t = (c.z + CLOUD_SLAB * 0.5 - rel.y) / l.y;
    if t <= 0.0 {
        return 1.0;
    }
    let tex = u32(c.x - 0.5);
    let dims = vec2<f32>(textureDimensions(textures[tex]));
    let q = scene.cloud_cells.xy + (rel.xz + l.xz * t) / c.y - vec2(0.5);
    let base = floor(q);
    let f = q - base;
    let u = f * f * (3.0 - 2.0 * f);
    let o = scene.cloud_cells.zw;
    let a = cloud_cell_of(tex, base, o, dims, c.w);
    let b = cloud_cell_of(tex, base + vec2(1.0, 0.0), o, dims, c.w);
    let d = cloud_cell_of(tex, base + vec2(0.0, 1.0), o, dims, c.w);
    let e = cloud_cell_of(tex, base + vec2(1.0, 1.0), o, dims, c.w);
    var cover = mix(mix(a, b, u.x), mix(d, e, u.x), u.y);
    if cover > 0.02 && cover < 0.98 {
        cover = clamp(cover + cloud_fuzz(base + o + f) * 0.45, 0.0, 1.0);
    }
    // Fading in as the light climbs: long low shadows from far clouds would smear across the land.
    let lift = smoothstep(0.03, 0.15, l.y);
    // The sky's puffs fill in over this range of the blend at their middle (`cloud_density`), and
    // their shade's edge is wider than the cloud's own, a few blocks of soft penumbra (at 0.35-0.6
    // it was a hard polygon on open sand); a little lighter in the middle (0.8 before).
    return 1.0 - 0.72 * smoothstep(0.22, 0.75, cover) * lift;
}

// ---- the rain's cloud deck, shared by the sky and the light ----

// The deck's large forms at `cell` (the cloud picture's cells, absolute: the place in the cell plus
// the cell, as `scene.cloud_cells` and the sky's `uv` give them), 0..1: masses about nine cells
// (a hundred blocks) across with forms of forty inside them. Low where the deck breaks open, high
// where it stands up in towers. At three and a half cells the deck read as small repeating lumps,
// "scales", beside Bliss's clouds hundreds of blocks big (Ronald: "Still looks so much worse the
// bliss why ?", 11-05).
fn deck_field(cell: vec2<f32>) -> f32 {
    return noise3(vec3(cell.x / 9.0, 5.5, cell.y / 9.0)) * 0.7 + noise3(vec3(cell.x / 3.5, 9.5, cell.y / 3.5)) * 0.3;
}

// How open the deck is at a field value `n`, 0 closed to 1 a clear break; `wide` (0..1) opens the
// breaks further (toward the sun, where light breaks through).
fn deck_open(n: f32, wide: f32) -> f32 {
    // At 0.24-0.4 a break opened in about one place in twenty and no beams showed (c2f2eb1c).
    return 1.0 - smoothstep(0.32 + 0.08 * wide, 0.46 + 0.08 * wide, n);
}

// ---- light on things ----

// Block light (red, green and blue levels 0..1) as color: warm firelight, whiter right at the
// source, with a slow flicker that runs across the world.
fn block_light(b: vec3<f32>, pos: vec3<f32>) -> vec3<f32> {
    let m = max(max(b.x, b.y), b.z);
    if m <= 0.0 {
        return vec3(0.0);
    }
    // Wider pools than the levels alone give (Valheim's fires light a wide ring of ground): the
    // dim edge of a torch's reach brought up.
    let curve = pow(vec3(brightness_curve(b.x), brightness_curve(b.y), brightness_curve(b.z)), vec3(0.9));
    // White light (torches, lanterns) is warmed; colored light keeps its color.
    let grey = 1.0 - (m - min(min(b.x, b.y), b.z)) / m;
    let warm = mix(vec3(1.0), mix(vec3(1.0, 0.56, 0.26), vec3(1.0, 0.82, 0.62), m * m), grey);
    let t = scene.params.w;
    let q = floor(pos * 0.5);
    let flicker = 1.0 + 0.045 * sin(t * 7.3 + q.x * 1.7 + q.z * 2.3) + 0.03 * sin(t * 13.1 + q.y * 3.1);
    // A little less than the levels alone gave: wider at 1.55, the middle of the pool went yellow.
    return curve * warm * 1.4 * flicker;
}

// How much a surface with normal `n` faces the sun. With the sun low its light wraps a little
// over the tops, which are otherwise only grazed: the warm ground of the golden hour.
fn sun_facing(n: vec3<f32>) -> f32 {
    let wrap = 0.45 * (1.0 - smoothstep(0.05, 0.3, abs(scene.sun.y)));
    return max((dot(n, light_dir()) + wrap) / (1.0 + wrap), 0.0);
}

// The light the player holds (`scene.hand`, `scene.hand_light`: a torch, a lantern) at `rel`
// (camera relative) on a surface facing `n`, as block light levels per channel: it falls off a
// level a block, as placed torches do, and is a little weaker on faces turned away (`n` zero: no
// facing, particles). Nights are dark by design and torches light the way (Ronald, 11-04).
fn held_light(rel: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    let l = scene.hand_light.rgb;
    if max(max(l.x, l.y), l.z) <= 0.0 {
        return vec3(0.0);
    }
    let to = scene.hand.xyz - rel;
    let d = length(to);
    var facing = 0.85;
    if dot(n, n) > 0.5 {
        facing = mix(0.5, 1.0, clamp(dot(n, to / max(d, 1e-3)), 0.0, 1.0));
    }
    // A warm pool that lights the path ahead and fades into the dark over its eight to ten blocks (Ronald, 11-05: "between the two", Bliss's small pool at the hand and ours, a level a
    // block to fourteen, which lit the whole night forest like dusk). It still lights the way.
    // (From 2.5 blocks, gone by 10: at 6-11 all within six blocks kept its full light and a close
    // forest still read as dusk, PC sht_torch.)
    let pool = 1.0 - smoothstep(2.5, 10.0, d);
    return max(l - vec3(d / 15.0), vec3(0.0)) * facing * pool;
}

// Light arriving at a surface with normal `n`. `light` is the world's (sky, red, green, blue, 0..1),
// `shadow` 1 where the sun reaches it, `ao` the corner shading, `rel` where it is from the camera.
// How much of block light shows where the sky's light is `sky` (0..1): all of it at night and
// under cover, a quarter in the open by day. Vanilla takes the brighter of the two, not their sum:
// summed, a lamp's level 15 at noon washed the stone about it out to a pale, glassy blue (First
// hours' Starheart lantern, 11-05).
fn block_by_day(sky: f32) -> f32 {
    // By the sun's height, not the daylight value: that stays above zero at night, and a torch
    // at midnight lost some of its warmth.
    return 1.0 - 0.75 * smoothstep(0.5, 1.0, sky) * smoothstep(0.0, 0.25, scene.sun.y);
}

// The sun's light through leaves in the HDR world, as a share of the sun's (times the leaf's paint):
// looking toward the sun a crown glows (Bliss's leaves up to about ten times brighter that way, never
// black), a little all round as light passes through the thin leaves; `shadow` the sun's reach there,
// `sky` the sky light level (none deep in the dark of a canopy), `rel` camera relative. 0 without
// the HDR world. Far crowns (`fs_lod`) call it too, so near and far tops light alike.
fn leaf_through(rel: vec3<f32>, shadow: f32, sky: f32) -> f32 {
    if scene.frame.x < 0.5 {
        return 0.0;
    }
    let back = max(-dot(normalize(rel), light_dir()), 0.0);
    let open = smoothstep(0.4, 0.9, sky);
    return (pow(back, 4.0) * 1.6 + 0.12) * light_strength() * mix(0.25, 1.0, shadow) * open;
}

fn light_on(n: vec3<f32>, light: vec4<f32>, shadow: f32, ao_in: f32, pos: vec3<f32>, rel: vec3<f32>) -> vec3<f32> {
    let ao = mix(ao_in, 1.0, clamp(fantasy_opt(OPT_AO_TEST), 0.0, 1.0));
    let p = palette();
    // The sky's light falls off steeply under cover (Bliss's lightmap, (L^15 x 2 + L^2.5) / 3: 1 in the
    // open, 0.39 a step under, 0.21 two): a forest floor sits a few times below open shade, the open
    // stays as it was.
    let steep = look_opt(OPT_SHADE, 1.0);
    // In the HDR world halved, not a third (Bliss's own curve: 1.5 in the open): a forest floor
    // under a canopy sits at sky light 13-14 of 15, where a third gave it a third of Bliss's
    // ambient (floor 44-51 against Bliss's 127-149; Trees' /tk light grid, 11-06).
    // By day only, eased out through dusk and dawn: at night the moon's sky light took it too and
    // the night land came out 60 % brighter (22:00 land luma 23 against 14; Shader's shprJ, 11-06).
    let bliss_sky = select(0.0, smoothstep(-0.1, 0.02, scene.sun.y), scene.frame.x > 0.5);
    let sky = (pow(light.x, 15.0 * steep) * 2.0 + pow(light.x, 2.5 * steep)) / mix(3.0, 2.0, bliss_sky);
    let under = under_leaves(light.x);
    let leaves = mix(vec3(1.0), under_tint(), under * 0.5);
    // More of the sky's light comes from above than from the sides, and less from below: tops 1,
    // sides 0.75, bottoms 0.5 (Complementary's), so shade still shows form.
    // Under rain's overcast more so (tops 1, sides 0.6, bottoms 0.2): the sky's light is nearly all
    // the light then, and even from all round it made the land flat.
    var hemi = mix(0.75 + 0.25 * n.y, 0.6 + 0.4 * n.y, rain_amount());
    // In the HDR world by clear sky Bliss's split by facing: tops 2.5, sides 0.93, bottoms 0.2
    // (with the curve above, a top in the shade takes 3.75x what it did): crowns lit from above
    // with dark sides and gaps, and a forest floor as bright as Bliss's (the vista from above and
    // Trees' forest, k6 runs, 11-06).
    if bliss_sky > 0.0 {
        let facing = select(mix(0.93, 0.2, -n.y), mix(0.93, 2.5, n.y), n.y >= 0.0);
        hemi = mix(hemi, mix(facing, hemi, rain_amount()), bliss_sky);
    }
    var c = p.ambient * sky * hemi * ao * leaves;
    // In the HDR world a cloud overhead takes some of the sky's light too, not the sun's alone: with
    // the sky's full fill a cloud's shadow kept the land at about 0.72 of its lit brightness, where
    // Bliss's shaded forest falls to 0.4-0.6 (PC shcs_off/shcs_on against Bliss's vista, 11-06).
    if scene.frame.x > 0.5 && scene.cloud_shadow.x > 0.5 {
        c *= mix(0.6, 1.0, soft_cloud_shadow(rel));
    }
    // Under a forest's canopy most of the floor's light is the sun's, come down through the leaves:
    // green and soft (a rainforest's floor gets 1-2 % of the sun, more of it this way than from the
    // sky). Without it the floor of Plants' rainforest sat at luma 12-17 at noon, under the daytime
    // targets' 15-60 for deep forest shade (Lighting's canopy check, 11-04).
    c += p.sun * light_strength() * 0.045 * under_tint() * under * hemi * ao;
    // At golden hour the sky round the low sun glows: a warm fill from its side of the sky, on
    // the faces turned that way; those turned away keep the sky's cool light (Valheim's warm light,
    // blue shade).
    let l = light_dir();
    let side = normalize(vec3(l.x, 0.0, l.z) + vec3(1e-4, 0.0, 0.0));
    let toward = clamp(dot(n, side) * 0.5 + 0.5, 0.0, 1.0);
    let glow_k = fantasy_opt(OPT_GLOW_FILL);
    c += p.glow * golden() * 0.35 * select(1.0, glow_k, glow_k >= 0.0) * mix(0.3, 1.1, toward) * sky * hemi * ao * leaves;
    // Direct light only where the sky reaches (it keeps caves out of reach of the traced shadow's
    // gaps), and without corner shading: corners are dark where little sky gets in.
    // Under a forest's canopy the sun comes through the gaps the shadow map shows at nearly its
    // full strength (sunflecks), though the leaves cut the sky light there to 0.6-0.8.
    let c_open = min(canopy(), 1.0);
    let open = smoothstep(mix(0.5, 0.25, c_open), mix(0.95, 0.55, c_open), light.x);
    let sun_part = p.sun * light_strength() * sun_facing(n) * shadow * open * mix(0.75, 1.0, ao);
    c += sun_part;
    LIGHT_SUN = sun_part;
    // The ground round a lightning strike, lit by it (the HDR world). A held torch's light where it
    // is brighter than the placed lights there (no doubling). Neither in a calm scene (`CALM`).
    var held = vec3(0.0);
    if !CALM {
        c += lightning_glow_at(rel) * sky * hemi * ao;
        held = held_light(rel, n);
    }
    let blocks = block_light(max(light.yzw, held), pos) * mix(0.5, 1.0, ao) * block_by_day(light.x);
    NIGHT_SKY_SHARE = night_sky_share(c, blocks);
    c += blocks;
    // A floor so caves are never quite black: the dimension's own and the Brightness setting.
    let floor_light = max(brightness_curve(scene.light.x) * 0.25, mix(0.006, 0.03, scene.light.y));
    c += vec3(floor_light) * vec3(0.85, 0.9, 1.0) * ao;
    LIGHT_ALL = c;
    return c;
}

// What share of a surface's light is the sky's and the moon's, not a lamp's, for the night's blue
// (`display`): set by `light_on` and `light_round` for the pixel they light; 1 for what they don't
// (the sky, clouds). Keyed on where the light comes from, not how bright it is: by brightness, a
// lantern-lit room at midnight went blue-violet too, its light small before the night's exposure
// lifts it (NVIDIA's storeroom, rt-sun-shadows' find, 11-06).
var<private> NIGHT_SKY_SHARE: f32 = 1.0;

fn night_sky_share(sky_part: vec3<f32>, lamp_part: vec3<f32>) -> f32 {
    let s = luma(sky_part);
    let l = luma(lamp_part);
    return clamp(s / max(s + 3.0 * l, 1e-6), 0.0, 1.0);
}

// For what has no normal to speak of (particles, rain, the hand): half the direct light.
fn light_round(light: vec4<f32>, pos: vec3<f32>, rel: vec3<f32>) -> vec3<f32> {
    let p = palette();
    let sky = brightness_curve(light.x);
    var c = p.ambient * sky * 0.85;
    c += p.sun * light_strength() * 0.5 * smoothstep(0.5, 0.95, light.x);
    var held = vec3(0.0);
    if !CALM {
        held = held_light(rel, vec3(0.0));
    }
    let blocks = block_light(max(light.yzw, held), pos) * block_by_day(light.x);
    NIGHT_SKY_SHARE = night_sky_share(c, blocks);
    c += blocks;
    c += vec3(max(brightness_curve(scene.light.x) * 0.25, mix(0.006, 0.03, scene.light.y)));
    return c;
}

// ---- the air: wind and fog (`scene.weather`, from the weather in `mods/base/src/weather.rs`) ----

// Sea level in chiacraft's worlds (`base_api::SEA`): low fog lies on it and in the valleys.
const SEA_LEVEL: f32 = 63.0;

fn hash3(p: vec3<i32>) -> f32 {
    var h = bitcast<u32>(p.x) * 73856093u ^ bitcast<u32>(p.y) * 19349663u ^ bitcast<u32>(p.z) * 83492791u;
    h = (h ^ (h >> 16u)) * 0x45d9f3bu;
    h = (h ^ (h >> 16u)) * 0x45d9f3bu;
    h = h ^ (h >> 16u);
    return f32(h & 0xffffffu) / 16777215.0;
}

// Smooth noise 0..1: random values at whole points, blended between.
fn noise3(p: vec3<f32>) -> f32 {
    let i = vec3<i32>(floor(p));
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = mix(mix(hash3(i), hash3(i + vec3(1, 0, 0)), u.x), mix(hash3(i + vec3(0, 1, 0)), hash3(i + vec3(1, 1, 0)), u.x), u.y);
    let b = mix(mix(hash3(i + vec3(0, 0, 1)), hash3(i + vec3(1, 0, 1)), u.x), mix(hash3(i + vec3(0, 1, 1)), hash3(i + vec3(1, 1, 1)), u.x), u.y);
    return mix(a, b, u.z);
}

// The wind where the player is: m/s along the world's x and z, the way it blows.
fn wind_now() -> vec2<f32> {
    return scene.weather.xy;
}

// How hard the wind pushes at `p` now, about 1 for its mean: gusts are bands of stronger wind that
// sweep across the land the way it blows, so a field bends in waves.
fn gust(p: vec3<f32>) -> f32 {
    return gust_at(p, scene.params.w);
}

// `gust` at the shaders' clock `t` (last frame's for the motion image, `scene.taa.w`).
fn gust_at(p: vec3<f32>, t: f32) -> f32 {
    let w = wind_now();
    let speed = length(w);
    let dir = select(vec2(1.0, 0.0), w / max(speed, 1e-4), speed > 0.01);
    let along = dot(p.xz, dir) - t * max(speed, 1.0) * 0.9;
    let band = noise3(vec3(along / 14.0, t * 0.05, dot(p.xz, vec2(-dir.y, dir.x)) / 30.0));
    return 0.55 + 1.1 * smoothstep(0.35, 0.85, band);
}

// Where the fog lies along a look `rel` away (camera relative): on the ground about the player (a few
// blocks under the eye), or lower where the look ends lower (the ground there), or on the sea if
// that is higher. Valleys under it fill, hills stand out of it. Pinned to the eye alone, a layer hung
// 5 under a player high above the land and filled the air 24 below that: looking straight down from
// y 120 over a sinkhole at 14:00 went a milky white-out (the world playtester, 11-04).
fn fog_floor(rel: vec3<f32>) -> f32 {
    return max(SEA_LEVEL, min(scene.eye.y - 5.0, scene.eye.y + rel.y));
}

// How foggy the air is at `p` (world): a layer lying on the ground and in the valleys, broken into
// banks that drift with the wind and thin out higher up. 0 in clear weather. Per block.
// A storm's share of the fog (blowing snow or dust: `scene.ground` y and z): it fills the air
// high up and races with the wind instead of lying low.
fn storm() -> f32 {
    return clamp(max(scene.ground.y * smoothstep(5.0, 9.0, length(wind_now())), scene.ground.z), 0.0, 1.0);
}

// A dust storm's air by distance from the eye, 0..1 of its fog: clear near (a bubble of 16 blocks in
// a moderate wind, 8 in the strongest), thickening outward, so the ground about the player stays
// readable and the storm closes only some 90 blocks out, the sky in the same dust (it is fogged
// over 220). The fog filling the air from the eye turned the whole view a flat beige a few blocks
// off (the Recrafted study, item 2a; ImmersiveVFX's sandstorm closes at about 96).
fn dust_reach(p: vec3<f32>) -> f32 {
    let dust = clamp(scene.ground.z, 0.0, 1.0);
    if dust <= 0.0 {
        return 1.0;
    }
    let bubble = mix(16.0, 8.0, smoothstep(7.0, 11.0, length(wind_now())));
    let near = mix(0.12, 1.0, smoothstep(bubble * 0.5, bubble * 3.0, length(p - scene.eye.xyz)));
    return mix(1.0, near * 0.6, dust);
}

fn fog_density(p: vec3<f32>, floor: f32) -> f32 {
    return fog_density_with(p, floor, fog_banks_at(p, floor));
}

// The banks' noise at `p` (`fog_density`): walked only where the banks count, their mean (0.3)
// elsewhere.
fn fog_banks_at(p: vec3<f32>, floor: f32) -> f32 {
    let s = storm();
    let h = p.y - mix(min(floor, SEA_LEVEL + MIST_TOP), floor, s);
    let low = exp(-max(h - 3.0, 0.0) / mix(7.0 + 12.0 * min(canopy(), 1.0), 40.0, s));
    let open = smoothstep(-24.0, -10.0, h);
    let near_floor = scene.frame.x > 0.5 && p.y - floor < 14.0 && p.y < SEA_LEVEL + MIST_TOP + 50.0;
    // Bliss's banks (`OPT_MIST_DOMES`) stand free in the air, not only by what the ray ends on: the
    // noise all through their height. (With the constant away from the ground they lay as a coat on
    // the crowns, edged by them: the live sheet's banks try.)
    let banks_free = scene.frame.x > 0.5 && fantasy_opt(OPT_MIST_DOMES) > 0.0 && s < 0.5
        && p.y < SEA_LEVEL + MIST_SEA_LEVEL + 24.0;
    if low * open > 0.02 || near_floor || banks_free {
        return fog_banks(p);
    }
    return 0.3;
}

// Banks: a broad octave, and a fine one that both shapes and eats them, rising slowly (one octave
// fewer than before: three noises a step were most of the mist's cost, +3.4 ms on the PC at 3456
// px, Research round 15). Its weight keeps the mix's spread, so the banks cover as much.
fn fog_banks(p: vec3<f32>) -> f32 {
    let drift = vec3(wind_now().x, 0.0, wind_now().y) * scene.params.w * mix(0.6, 2.0, storm());
    let q = p - drift;
    let broad = noise3(q * vec3(1.0 / 38.0, 1.0 / 12.0, 1.0 / 38.0));
    let fine = noise3(q * vec3(1.0 / 9.0, 1.0 / 5.5, 1.0 / 9.0) + vec3(0.0, scene.params.w * 0.03, 0.0));
    return smoothstep(0.45, 0.66, broad * 0.7 + fine * 0.39 - 0.05);
}

// `fog_density` with the banks' noise `banks` given (`fog_banks`): the mist volume takes it once a
// cell and the height's shape at several places through it (`froxel_cell`).
fn fog_density_with(p: vec3<f32>, floor: f32, banks: f32) -> f32 {
    let f = scene.weather.z * dust_reach(p);
    // Every layer but the evening's by the air's own mist (`air_mist`).
    let f_air = air_mist() * dust_reach(p);
    let s = storm();
    // The mist (not a storm's dust or snow) lies below a height of its own, `MIST_TOP` over the
    // sea (`vitals::air`), as Bliss's does: rivers, lakes and valleys fill, and ridges and forests higher up stand
    // out of it. Lying on whatever ground was under it, it blanketed the hills too, and from the
    // forest vista 210 up the dawn mist was a pink-white sea from 04:00 to 08:00 (the Look
    // session's checkpoint, 11-05; the same rule as its HDR path, 5124c25f).
    let h = p.y - mix(min(floor, SEA_LEVEL + MIST_TOP), floor, s);
    // In a forest the morning mist rises into the crowns (a rainforest's lies on its canopy at
    // dawn, 25-35 blocks up).
    let low = exp(-max(h - 3.0, 0.0) / mix(7.0 + 12.0 * min(canopy(), 1.0), 40.0, s));
    let high = exp(-max(h, 0.0) / 45.0);
    // Deep under it, in the ground, there is none.
    let open = smoothstep(-24.0, -10.0, h);
    // The sun thins the banks by day to a third (mornings and evenings keep them); a storm's
    // blowing snow or dust it leaves.
    let burn = mix(1.0, mix(0.35, 1.0, s), smoothstep(0.25, 0.65, scene.sun.y));
    // In the HDR world the mist lies in banks, as Bliss's does: the low layer is mostly banks with
    // clear lanes between them (their edges sharper), a fifth of the thin haze up high, so ridges
    // and tree tops stand out of it and the beams have clear air to cross. (One even veil read as
    // milk over the whole view: the mist sheet, 11-05.)
    if scene.frame.x > 0.5 {
        // As the sun climbs the banks lift and dissolve into a thin even haze (they burned off as
        // grey smudges on the forest by 08:00, the checkpoint sheet).
        let b = mix(smoothstep(0.25, 0.75, banks), 0.3, smoothstep(0.15, 0.45, scene.sun.y));
        // A dense layer up to a sharp top at the mist line (sea level + MIST_TOP), as Bliss's,
        // Photon's and Unbound's valley mist: it fills the low land to an edge that crowns and
        // ridges stand out of, the banks only varying it inside (a soft fall with height read as a
        // thin even veil with no top: the MC comparison's natural-mist sheets, 11-05). Storms' dust
        // and snow keep the old shape.
        // The evening's: Bliss's height fall (2^(-0.15 (y - 53)), floored 8 blocks under it), its
        // cloudy part as the banks; about 0.008 a block at sea level with `/air` 0.3.
        let ev = evening_mist() * (1.0 - s);
        var even = 0.0;
        if ev > 0.0 {
            even = f * open * 0.075 * exp2(-0.15 * (max(p.y, 45.0) - 53.0)) * (0.67 + smoothstep(0.25, 0.75, banks));
        }
        if s < 0.5 {
            // The top heaves with the banks, up to 5 blocks either way, so it is a lumpy sea that
            // crowns break through, not a flat sheet (a level top laid straight stripes along
            // every terrace of a valley's sides: spot A's first layer sheet, 11-06).
            // With `OPT_MIST_DOMES` the heave up to 20 blocks either way, on the broad banks, and the
            // top softer, so they stand as domes the sun lights on one side.
            let domes = clamp(fantasy_opt(OPT_MIST_DOMES), 0.0, 1.0);
            // and the layer deeper, its top over the canopy (MIST_DEEP blocks up at 1), as Bliss's
            // banks reach 30 blocks over its fog line: ours, a few blocks over the land, tinted the
            // ground orange instead of hiding it (lsAS_mistdawn2, Ronald: "10 x better" in Bliss).
            let top = SEA_LEVEL + MIST_TOP + MIST_DEEP * domes + (b - 0.45) * mix(10.0, 40.0, domes);
            // The top a little softer (6 blocks of fade, not 5.5: the pancakes along the river
            // had cut edges, try 2's sheet).
            let layer = 1.0 - smoothstep(top - mix(5.0, 10.0, domes), top + mix(2.5, 4.0, domes), p.y);
            // Wisps in the crowns above the layer, in patches: Bliss's dawn mist lies among the
            // tree tops all over, not only in the valleys (try 2 left the forests above the mist
            // line clear, the mist flat pancakes on the rivers). Within a few blocks of what the
            // ray ends on, in the banks' thick parts only; gone by the time the sun is 30° up.
            // Not above about 40 blocks over the mist line: a ray into the sky has its floor at the
            // eye, and wisps there drew a dark orange band along the horizon (try 3's sheet). Their
            // edges soft (hard-edged blobs on the crowns, the same sheet).
            let wisp = exp(-max(p.y - floor, 0.0) / 5.0) * smoothstep(0.4, 0.9, smoothstep(0.25, 0.75, banks))
                * (1.0 - smoothstep(0.2, 0.5, scene.sun.y)) * (1.0 - layer) * (1.0 - smoothstep(top + 25.0, top + 45.0, p.y))
                // None with the deep layer: the wisps lie on whatever the ray ends on, a coat of
                // mist on every crown (the live sheet's 07:00, ours sand-coloured tops).
                * (1.0 - domes);
            let made = f_air * open * burn * (layer * mix(2.2, 5.5, b) + wisp * 1.8 + 0.012 * high) * 0.11;
            // With `OPT_MIST_DOMES` Bliss's shape instead: sparse banks, dense inside (the noise's top
            // part only), clear air between, standing tall with a soft fall from a little over the
            // sea up (its banks by exp(-0.05 (y - 73)) on a noise over 0.6). The layer lay as an even
            // sand-coloured sheet on the crowns, a veil when thinner (the live sheets, 07:00).
            // The banks themselves are walked per pixel (`bank_march`); the volume keeps the thin
            // haze up high. (Sampled at each pixel's own surface in the volume's last stretch, they
            // lay as paint on the land, edged by the crowns: the live debug sheet.)
            let bliss = f_air * open * 0.012 * high * 0.11;
            return mix(mix(made, bliss, domes), even, ev);
        }
        return mix(f_air * open * burn * (low * (mix(0.05, 0.7, s) + mix(6.5, 0.6, s) * b) + 0.012 * high) * 0.11, even, ev);
    }
    return f_air * open * burn * (low * (mix(0.05, 0.7, s) + mix(1.7, 0.6, s) * banks) + 0.06 * high) * 0.11;
}

// Light the fog sends toward the eye: the sky's from all around, the sun's mostly forward, so it
// glows white-gold looking toward the sun.
// The color sunlight takes through mist with the sun low.
const MIST_GOLD: vec3<f32> = vec3(1.0, 0.7, 0.36);

struct FogLight {
    sky: vec3<f32>,
    sun: vec3<f32>,
    // The sun's light come down through a canopy's leaves, all round (no forward glow).
    leaf: vec3<f32>,
}

// Henyey-Greenstein's phase for asymmetry `g` at the cosine `mu` from the light, 1 where it scatters
// alike every way.
fn hg_phase(mu: f32, g: f32) -> f32 {
    return (1.0 - g * g) / pow(max(1.0 + g * g - 2.0 * g * mu, 1e-4), 1.5);
}

fn fog_light(dir: vec3<f32>) -> FogLight {
    let p = palette();
    let mu = dot(dir, light_dir());
    let g = 0.6;
    let hg = (1.0 - g * g) / pow(1.0 + g * g - 2.0 * g * mu, 1.5) / 12.566;
    // The sun's glow in the fog capped: uncapped it was over twice the sunlight looking toward a high
    // sun, and noon fog by a river or on snow went a milky white-out (the world playtester's shots,
    // frame luma 0.76, saturation 0.06). Valheim's daytime fog reads grey and soft.
    var phase = min(hg * 1.6, 0.9) + 0.3;
    // In the HDR world the curve rolls the brights off, so the glow toward the sun can be what
    // droplets really do: strongly forward (two Henyey-Greenstein lobes, g 0.8 and 0.1, as Bliss's
    // mist: about 30x toward the sun what it is across it), so a low sun through mist is a bright
    // gold haze with beams, and the mist away from it stays dim and lets the land through.
    if scene.frame.x > 0.5 {
        phase = (0.7 * hg_phase(mu, 0.8) + 0.3 * hg_phase(mu, 0.1)) * 0.45;
    }
    // Droplets are white, snow whiter, dust the color of sand.
    let tint = mix(mix(vec3(0.9), vec3(1.0, 1.0, 1.03), scene.ground.y), vec3(0.95, 0.68, 0.42), scene.ground.z);
    // At dawn and dusk the mist is lit gold toward the low sun and dims away from it (the
    // Amazon's dawn fog, the canopy refs' 07): less of the sky's cool light, the sun's glow warmer.
    // Before, a misty morning read milky white everywhere (the world playtester, Plants, 11-04).
    // Sunlight through mist turns gold well before golden hour, while the sun is under about 30°
    // (the refs' dawn shafts, 16; low sun scattered through haze loses its blue): with golden
    // hour's colour alone the 07:30 shafts in a rainforest's mist read grey-white (canopy sheet).
    // Dimmer too: thick mist lit at the sun's full strength toward it clipped to white, and white
    // shows no colour (the dawn check's 07:30 mist, luma p50 170, saturation 0.03).
    var warm = 1.0 - smoothstep(0.1, 0.65, scene.sun.y);
    // In the evening, as Bliss's, no gold of its own (`OPT_MIST_TINT`): the sun's colour is already
    // its sunset colour.
    // In the morning likewise with `OPT_MIST_DAWN_TINT`.
    let plain = clamp(fantasy_opt(OPT_MIST_TINT), 0.0, 1.0) * (1.0 - smoothstep(-0.05, 0.05, scene.sun.x))
        + clamp(fantasy_opt(OPT_MIST_DAWN_TINT), 0.0, 1.0) * smoothstep(-0.05, 0.05, scene.sun.x);
    warm *= 1.0 - plain;
    // With the sun up and low, the sky's light in the mist goes gold too (sunlight scattered all
    // round in it): lit by the sky's cool fill, a dawn mist read pale pink-white where Bliss's is
    // gold (the Look session's checkpoint, 11-05). None before sunrise: the blue hour keeps its blue.
    let sunlit_low = warm * smoothstep(-0.02, 0.06, scene.sun.y) * (1.0 - rain_amount());
    var sky_part = mix(p.ambient * 1.25 + p.horizon * 0.2, vec3(luma(p.ambient * 1.25 + p.horizon * 0.2)) * vec3(1.25, 1.0, 0.62), sunlit_low * 0.7) * mix(1.0, 0.5, warm);
    // In the HDR world the sun's forward glow carries the mist's light toward it, and away from it
    // the mist is dim, the sky's light alone (Bliss's mist across the sun is a blue-grey that lets
    // the land through, not a bright veil).
    // With the evening's tint (`OPT_MIST_TINT`) the sky's light at full, as Bliss's fog takes it.
    if scene.frame.x > 0.5 {
        sky_part *= mix(0.55, 1.0, plain);
    }
    // In the HDR world the sunlit mist a deeper gold (Bliss's dawn mist, saturation 181 against our 99).
    let gold = select(MIST_GOLD * 0.65, vec3(1.0, 0.58, 0.2) * 0.7, scene.frame.x > 0.5);
    let sun_part = p.sun * mix(vec3(1.0), gold, warm) * light_strength() * phase;
    let k = 0.75 * tint * look_opt(OPT_FOG, 1.0);
    let leaf_part = p.sun * mix(vec3(1.0), MIST_GOLD, warm) * light_strength() * 0.06 * LEAF_LIGHT;
    return FogLight(sky_part * k, sun_part * k, leaf_part * k);
}

// 1 where the sun reaches camera-relative `rel` (one tap of the near shadow map, else the far one;
// 1 beyond both), for the fog's light: shafts where it comes through, dark mist in the shade.
fn sun_reaches(rel: vec3<f32>) -> f32 {
    var lit = 1.0;
    if scene.shadow.z > 0.5 {
        let c = scene.shadow_view_proj * vec4(rel, 1.0);
        let ndc = c.xyz / c.w;
        if abs(ndc.x) < 0.98 && abs(ndc.y) < 0.98 && ndc.z > 0.0 && ndc.z < 1.0 {
            let at = vec2<i32>(vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * scene.shadow.y);
            lit = select(0.0, 1.0, textureLoad(textures[u32(scene.shadow.x)], at, 0).r <= ndc.z + 0.0005);
            // With traced shadows (`scene.rt`) the near map holds the models alone, the terrain is
            // in the far one: the darker of both, as surfaces take (`shadow_at`). Returning the near
            // map's alone, the mist within 80 blocks at Max was lit through hills and canopies: a
            // forest's dawn mist glowed orange under the leaves (Mist's forest-floor shots, 11-06).
            if scene.rt.x < 0.5 {
                return mix(1.0, lit, smoothstep(0.02, 0.06, light_dir().y));
            }
        }
    }
    if scene.shadow_far.z > 0.5 {
        let c = scene.shadow_far_view_proj * vec4(rel, 1.0);
        let ndc = c.xyz / c.w;
        if abs(ndc.x) < 0.98 && abs(ndc.y) < 0.98 && ndc.z > 0.0 && ndc.z < 1.0 {
            let at = vec2<i32>(vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * scene.shadow_far.y);
            lit = min(lit, select(0.0, 1.0, textureLoad(textures[u32(scene.shadow_far.x)], at, 0).r <= ndc.z + 0.0004));
        }
    }
    return mix(1.0, lit, smoothstep(0.02, 0.06, light_dir().y));
}

// The fog between the eye and a point `rel` away (camera relative), walked in a few steps that are
// closer together near the eye; `sky` (0..1) the sky light at the point, so caves stay clear. Each
// step asks the shadow map whether the sun reaches it: mist in the shade of a hill or a forest's
// canopy gets only the sky's light (under a canopy dim and green, the leaves cut the sky too), mist
// the sun reaches its glow, so light shafts show where it comes through. Lit as if all in the open,
// a rainforest's morning mist was a white wall between the trunks (Lighting's canopy checks, 11-04).
fn fog_through(c: vec3<f32>, rel: vec3<f32>, sky: f32) -> vec3<f32> {
    // No fog in a calm scene's air (`CALM`: `scene.weather.z` 0).
    if CALM || scene.weather.z < 0.005 || in_liquid() || !open_sky() || sky <= 0.0 {
        return c;
    }
    if scene.froxels.x > 0.5 {
        let banked = bank_march(c, rel);
        // The banks' debug views show their march alone (`bank_march`).
        if look_debug() >= 3u && look_debug() <= 5u {
            return banked;
        }
        return froxel_fog(banked, rel, sky);
    }
    // The whole view: walked only to 220 blocks, the fog lying on the land ended 220 blocks out, a
    // flat pale disc seen from high up (the world playtester, y 250 over snow, 11-04).
    let dist = min(length(rel), 1500.0);
    let dir = rel / max(length(rel), 1e-4);
    let eye = scene.eye.xyz;
    let floor = fog_floor(rel);
    // Looking down on the fog from well above it, the steps crowd toward the far end, where the
    // layer lies; from in it or beside it, toward the eye.
    let from_end = eye.y - floor > 24.0;
    let light = fog_light(dir);
    let leafy = min(canopy(), 1.0);
    // In the canopy's shade the mist is lit by what comes through the leaves, warm green, more than
    // by the sky they hide: the sky's blue alone, tinted green, made it teal (the dawn check on
    // Plants' rainforest, 11-04).
    let shaded_sky = light.sky * mix(vec3(1.0), LEAF_LIGHT * 0.2, leafy * 0.85) + light.leaf * leafy;
    // Jittered along the view per pixel, so the shafts' edges don't band.
    // Interleaved gradient noise over the screen (a fixed grid of it), moved on each frame (`scene.frame.w`, which counts
    // frames even while the clock is frozen): temporal anti-aliasing averages it. A hash of the
    // point stood still, a fixed grain on everything in the mist with the volume off.
    let clip = scene.view_proj * vec4(rel, 1.0);
    let px = trunc((vec2(clip.x, -clip.y) / max(clip.w, 1e-4) * 0.5 + 0.5) * vec2(2048.0, 1152.0)) + 5.588238 * scene.frame.w;
    let jitter = fract(52.9829189 * fract(dot(px, vec2(0.06711056, 0.00583715)))) - 0.5;
    // 8 steps (12 before), and the shadow map asked once for each pair of them, between the two:
    // the jitter blends the coarser shafts (Research round 15: the mist's cost, the shadow taps
    // about 3 ms of it on the Mac).
    let steps = 8u;
    var through = 1.0;
    var gained = vec3(0.0);
    var last = 0.0;
    var lit = 1.0;
    for (var i = 1u; i <= steps; i++) {
        let k = f32(i) / f32(steps);
        let near = (exp(2.2 * k) - 1.0) / (exp(2.2) - 1.0);
        let far = 1.0 - (exp(2.2 * (1.0 - k)) - 1.0) / (exp(2.2) - 1.0);
        let at = dist * select(near, far, from_end);
        let step = at - last;
        // Jittered by under half a step: at 0.8 of one the shafts showed a fixed grain.
        let mid = at - step * (0.5 + 0.35 * jitter);
        let d = fog_density(eye + dir * mid, floor) * sky;
        let kept = exp(-d * step);
        if (i & 1u) == 1u {
            // At this step's far end, the pair's middle.
            lit = sun_reaches(dir * (at - step * 0.35 * jitter));
        }
        gained += through * (1.0 - kept) * (mix(shaded_sky, light.sky, lit) + light.sun * lit);
        through *= kept;
        last = at;
    }
    return c * through + gained;
}

// Bliss's banks (`OPT_MIST_DOMES`), walked per pixel through the height they stand in: 8 steps,
// jittered (TAA blends them), each with its own noise and sun shadow, so a bank stands free in the
// air, lit on its sunny side, dark in the crowns' shade, with clear air between banks. Bliss walks
// its fog the same way (`overworld_fog.glsl`, method only); ours in the volume's coarse cells could
// not hold a bank's edge or a tree's shadow.
const MIST_MARCH_FAR: f32 = 1500.0;
// The ground's top under the open sky at `rel` (camera relative), leaves left out (the engine's
// `client::ground_map`, `core:ground_map`): x its height, y 1 where known (0 off the map or before
// the engine has built it), z 1 over water.
// The ground's height at `rel` blended between the four nearest cells (x, y 1 where any is known): read
// from one cell, the banks' foot stepped every 4 blocks and their edges looked cut (lv7).
fn ground_smooth(rel: vec3<f32>) -> vec2<f32> {
    let g = scene.ground_map;
    if g.x < 0.5 {
        return vec2(0.0);
    }
    let q = (rel.xz - g.zw) / g.y - vec2(0.5);
    let f = fract(q);
    var sum = 0.0;
    var weight = 0.0;
    for (var k = 0; k < 4; k++) {
        let o = vec2(f32(k % 2), f32(k / 2));
        let mid = g.zw + (floor(q) + o + vec2(0.5)) * g.y;
        let at = ground_at(vec3(mid.x, rel.y, mid.y));
        let w = mix(1.0 - f.x, f.x, o.x) * mix(1.0 - f.y, f.y, o.y) * at.y;
        sum += at.x * w;
        weight += w;
    }
    return select(vec2(0.0), vec2(sum / weight, 1.0), weight > 1e-4);
}
fn ground_at(rel: vec3<f32>) -> vec3<f32> {
    let g = scene.ground_map;
    if g.x < 0.5 {
        return vec3(0.0);
    }
    let t = textures[u32(g.x) - 1u];
    let size = vec2<i32>(textureDimensions(t));
    let at = vec2<i32>(floor((rel.xz - g.zw) / g.y));
    if any(at < vec2(0)) || any(at >= size) {
        return vec3(0.0);
    }
    let v = textureLoad(t, at, 0);
    let h = round(v.r * 255.0) * 256.0 + round(v.g * 255.0) - 2048.0;
    return vec3(h, step(0.5, v.a), step(0.5, v.b));
}

// The mist's banks at `p` (world), `rel` from the camera: a sea of mist full up to its level and
// thinning above (`MIST_SEA_LEVEL`), cut into banks by a broad noise and frayed by a fine one
// (`MIST_SHAPE`, `MIST_ERODE`), drifting with the wind, out to the march's end (`MIST_MARCH_FAR`).
// Not ended 256 blocks out: Bliss's distance term there blends its biomes' fog, and with it ours
// left the vista from 200 up with no banks at all (lv8's debug views).
fn bank_density(p: vec3<f32>, rel: vec3<f32>) -> f32 {
    let q = p - vec3(wind_now().x, 0.0, wind_now().y) * scene.params.w * 0.6;
    let shape = noise3(q / MIST_SHAPE.xyx);
    let cut = clamp((shape - MIST_SHAPE.z) / max(MIST_SHAPE.w - MIST_SHAPE.z, 1e-3), 0.0, 1.0);
    if cut <= 0.0 {
        return 0.0;
    }
    let erode = noise3(q / MIST_ERODE.xyx - vec3(cut * 7.3));
    let banks = max(cut - erode * MIST_ERODE.z, 0.0);
    let fall = exp(-MIST_SEA_FALL * max(p.y - (SEA_LEVEL + MIST_SEA_LEVEL), 0.0));
    // Thinner as the sun climbs, gone by about 40° up; in the evening a little only.
    let burn = (1.0 - smoothstep(0.3, 0.65, scene.sun.y)) * mix(MIST_BANK_EVENING, 1.0, smoothstep(-0.1, 0.1, scene.sun.x));
    return air_mist() * MIST_SEA_DENSE * look_opt(OPT_BANK_DENSE, 1.0) * banks * fall * burn;
}
// The banks' sunlight by the angle from the sun (`MIST_SEA_PHASE`).
fn bank_phase(mu: f32) -> f32 {
    let off = 1.0 - mu;
    return MIST_SEA_PHASE.x * exp(-MIST_SEA_PHASE.y * off) + MIST_SEA_PHASE.z * exp(-MIST_SEA_PHASE.w * off);
}
fn bank_march(c: vec3<f32>, rel: vec3<f32>) -> vec3<f32> {
    let deep = clamp(fantasy_opt(OPT_MIST_DOMES), 0.0, 1.0);
    if deep <= 0.0 || scene.frame.x < 0.5 || storm() > 0.5 || air_mist() <= 0.0 {
        return c;
    }
    let dist = length(rel);
    let dir = rel / max(dist, 1e-4);
    let eye = scene.eye.xyz;
    let lo = SEA_LEVEL - 20.0;
    let hi = SEA_LEVEL + MIST_SEA_LEVEL + 60.0;
    var t0 = 0.0;
    var t1 = min(dist, MIST_MARCH_FAR);
    if abs(dir.y) > 1e-4 {
        let ta = (lo - eye.y) / dir.y;
        let tb = (hi - eye.y) / dir.y;
        t0 = max(t0, min(ta, tb));
        t1 = min(t1, max(ta, tb));
    } else if eye.y < lo || eye.y > hi {
        return c;
    }
    if t1 <= t0 {
        return c;
    }
    let clip = scene.view_proj * vec4(rel, 1.0);
    let px = trunc((vec2(clip.x, -clip.y) / max(clip.w, 1e-4) * 0.5 + 0.5) * vec2(2048.0, 1152.0)) + 5.588238 * scene.frame.w;
    let jitter = fract(52.9829189 * fract(dot(px, vec2(0.06711056, 0.00583715))));
    let p = palette();
    let k = 0.75 * look_opt(OPT_FOG, 1.0);
    // The sun's light, cream (`MIST_SEA_CREAM`), by the phase; the sky's, brighter looking up.
    let cream = mix(p.sun, vec3(luma(p.sun)) * vec3(1.0, 0.92, 0.78), MIST_SEA_CREAM.x) * MIST_SEA_CREAM.y;
    let sun_in = cream * light_strength() * min(bank_phase(dot(dir, light_dir())), MIST_SEA_CREAM.z) * k
        * look_opt(OPT_BANK_SUN, 1.0);
    let sky_in = p.ambient * (1.0 + MIST_SEA_SKY.x * pow(clamp(dir.y * 0.5 + 0.5, 0.0, 1.0), MIST_SEA_SKY.y))
        * MIST_SEA_SKY.z * k * look_opt(OPT_BANK_SHADE, 1.0);
    // 8 steps spaced ever wider from the near end (u^11 curve, as Bliss's), jittered per pixel (TAA
    // blends them): close steps where the mist near the eye shows its shape.
    let n = 8u;
    let e = 11.0;
    let span = t1 - t0;
    var through = 1.0;
    var gained = vec3(0.0);
    // Debug view 5: where along the march the sun reaches the mist (Planner on lv8: no shafts and no
    // shaded hollows showed, so check the shadow terms): red the shadow map's light, green the
    // clouds', blue how much mist. White lit by both, cyan in the land's shade, magenta in a cloud's,
    // blue in both, black no mist.
    var shaft = vec2(0.0);
    // Debug views with the banks on (the world and the evening's mist left out): 3 the light the banks
    // scatter toward the eye alone, on black; 4 how much they hide (1 - through) as grey.
    for (var i = 0u; i < n; i++) {
        let u = (f32(i) + jitter) / f32(n);
        let t = t0 + span * (pow(e, u) - 1.0) / (e - 1.0);
        let step = span * pow(e, u) * log(e) / (f32(n) * (e - 1.0));
        let d = bank_density(eye + dir * t, dir * t) * deep;
        if d <= 0.0 {
            continue;
        }
        // The land's and the clouds' shadow: shafts where the sun comes through, the sky's light alone
        // in the crowns' shade.
        let land = sun_reaches(dir * t);
        let clouds = cloud_shadow(dir * t);
        let lit = land * clouds;
        let kept = exp(-d * step);
        shaft += through * (1.0 - kept) * vec2(land, clouds);
        gained += through * (1.0 - kept) * (sky_in + sun_in * lit);
        through *= kept;
    }
    if look_debug() == 3u {
        return gained;
    }
    if look_debug() == 4u {
        return vec3(1.0 - through);
    }
    if look_debug() == 5u {
        return vec3(shaft, 1.0 - through);
    }
    return c * through + gained;
}

// The mist as a volume (`client::froxels`, `core:froxels`): the view cut into cells an eighth of
// the pixels each way and 32 slices deep, out to 1500 blocks, closer together near the eye. The
// sky fills it before the world is drawn (`fs_froxel_light`: each cell's own mist and light, as
// `fog_through` walks them; `fs_froxel_sum`: all of it from the eye to each cell's far end), and
// every surface reads it in two filtered taps instead of walking the mist in 8 steps: the mist,
// with its shadow taps, was among the Mac's dearest parts (Research rounds 15 and 19).
const FROXEL_SLICES: f32 = 32.0;
const FROXEL_FAR: f32 = 1500.0;
const FROXEL_CURVE: f32 = 6.0;

// Where slice `k` ends, blocks from the eye (the first under a block, the last at 1500).
fn froxel_end(k: f32) -> f32 {
    return FROXEL_FAR * (exp(FROXEL_CURVE * (k + 1.0) / FROXEL_SLICES) - 1.0) / (exp(FROXEL_CURVE) - 1.0);
}

// The slices from the eye to `d` blocks: k + 1 at the end of slice k.
fn froxel_slice(d: f32) -> f32 {
    return FROXEL_SLICES * log(1.0 + d * (exp(FROXEL_CURVE) - 1.0) / FROXEL_FAR) / FROXEL_CURVE;
}

// The sums from the eye to the end of slice `k` at screen `uv`: rgb the light gathered, a how much
// gets through (none before the first: nothing, all).
fn froxel_at(uv: vec2<f32>, k: i32) -> vec4<f32> {
    if k < 0 {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    let grid = scene.froxels.zw;
    let cell = clamp(uv * grid, vec2(0.5), grid - 0.5);
    let tile = vec2(f32(k % 8), f32(k / 8));
    let at = (tile * grid + cell) / (grid * vec2(8.0, FROXEL_SLICES / 8.0));
    return textureSampleLevel(textures[u32(scene.froxels.x) - 1u], linear_clamp, at, 0.0);
}

// `fog_through` from the volume. `sky` thins the mist on the way as it did each step there: what
// gets through to the power `sky`, and the light gathered by as much.
fn froxel_fog(c: vec3<f32>, rel: vec3<f32>, sky: f32) -> vec3<f32> {
    let clip = scene.view_proj * vec4(rel, 1.0);
    let uv = clamp(vec2(clip.x, -clip.y) / max(clip.w, 1e-4) * 0.5 + 0.5, vec2(0.0), vec2(1.0));
    let dist = min(length(rel), FROXEL_FAR);
    let k = i32(trunc(froxel_slice(dist)));
    // The volume's sums up to where the last slice starts, then the rest of the way to the point
    // walked here, for this pixel: four taps of the mist from the slice's start to the point, lit
    // with the slice's own light. Read off the volume by how far into its slice the point lay (as
    // if the slice's mist were even through it), a mist lying on the land, all at the far end of
    // the slice where the ground is, came in by where the ground fell in its slice: rings round
    // the eye again, one a slice (the ring fix's third run, 11-06). And the mist where it lies on
    // the land is at the pixel's own detail, not the volume's cells.
    let before = froxel_at(uv, k - 1);
    let a0 = froxel_end(f32(k) - 1.0);
    let dir = rel / max(length(rel), 1e-4);
    let flo = fog_floor(rel);
    let banks = fog_banks_at(scene.eye.xyz + dir * (0.5 * (a0 + dist)), flo);
    // With the deep layer (`OPT_MIST_DOMES`) this last stretch, where the layer on the land lies, is
    // lit per pixel, each tap with its own shadow: a slice 400 blocks out is 80 deep and a cell an
    // eighth of the screen, so the crowns' shadows in the mist averaged away and it lay evenly lit,
    // a sheet of sand (the live sheet's 07:00), where Bliss's, walked per pixel, has their shafts.
    let deep = clamp(fantasy_opt(OPT_MIST_DOMES), 0.0, 1.0);
    let stretch = max(dist - a0, 0.0) / 4.0;
    var depth = 0.0;
    var own = vec3(0.0);
    var passed = 1.0;
    var light = FogLight(vec3(0.0), vec3(0.0), vec3(0.0));
    if deep > 0.0 {
        light = fog_light(dir);
    }
    for (var i = 0; i < 4; i++) {
        let t = mix(a0, dist, (f32(i) + 0.5) / 4.0);
        let d = fog_density_with(scene.eye.xyz + dir * t, flo, banks);
        depth += d;
        if deep > 0.0 && d > 0.0 {
            let lit = sun_reaches(dir * t);
            let shade = mix(1.0, MIST_DEEP_SHADE, 1.0 - lit);
            let kept_i = exp(-d * stretch);
            own += passed * (1.0 - kept_i) * (mix(light.sky * shade, light.sky, lit) + light.sun * lit + mist_bounce(d, lit) * shade);
            passed *= kept_i;
        }
    }
    let kept = exp(-depth * stretch);
    let left = before.a * kept;
    let cell_light = froxel_light_at(uv, min(k, i32(FROXEL_SLICES) - 1));
    let own_light = select(cell_light * (1.0 - kept), own, deep > 0.0);
    let gained = before.rgb + before.a * mix(cell_light * (1.0 - kept), own_light, deep);
    // The deep layer's banks stand in the open air: not thinned by the surface's own sky light, which
    // edged them by the crowns.
    let through = pow(max(left, 1e-6), mix(sky, 1.0, deep));
    return c * through + gained * (1.0 - through) / max(1.0 - left, 1e-4);
}

// Slice `k`'s own light at screen `uv` (`fs_froxel_light`'s rgb: what its mist sends toward the eye
// where it is thick), for the last stretch `froxel_fog` walks itself.
fn froxel_light_at(uv: vec2<f32>, k: i32) -> vec3<f32> {
    let grid = scene.froxels.zw;
    let cell = clamp(uv * grid, vec2(0.5), grid - 0.5);
    let tile = vec2(f32(max(k, 0) % 8), f32(max(k, 0) / 8));
    let at = (tile * grid + cell) / (grid * vec2(8.0, FROXEL_SLICES / 8.0));
    return textureSampleLevel(textures[u32(scene.froxels.y)], linear_clamp, at, 0.0).rgb;
}

// How much of what lies `rel` away (camera relative) the mist lets through, 0..1, with `sky` (0..1)
// the sky light there as `fog_through` takes it: for what is added after the haze (the bottom seen
// through water, `world.wgsl`'s `water`). From the volume; 1 without it.
fn mist_kept(rel: vec3<f32>, sky: f32) -> f32 {
    if CALM || scene.weather.z < 0.005 || in_liquid() || !open_sky() || sky <= 0.0 || scene.froxels.x < 0.5 {
        return 1.0;
    }
    let clip = scene.view_proj * vec4(rel, 1.0);
    let uv = clamp(vec2(clip.x, -clip.y) / max(clip.w, 1e-4) * 0.5 + 0.5, vec2(0.0), vec2(1.0));
    let f = froxel_slice(min(length(rel), FROXEL_FAR));
    let k = i32(trunc(f));
    let v = mix(froxel_at(uv, k - 1), froxel_at(uv, min(k, i32(FROXEL_SLICES) - 1)), fract(f));
    return pow(max(v.a, 1e-6), sky);
}

// One cell's own mist and light for the volume (`fs_froxel_light`): toward `dir`, `at` blocks out,
// with the mist's floor `floor` (`fog_floor` of what lies behind it): rgb the light the mist sends
// toward the eye there (as `fog_through` lights each step: the sky's, and the sun's where it
// reaches), a its density (per block).
fn froxel_cell(dir: vec3<f32>, at: f32, floor: f32) -> vec4<f32> {
    let light = fog_light(dir);
    let leafy = min(canopy(), 1.0);
    let shaded_sky = light.sky * mix(vec3(1.0), LEAF_LIGHT * 0.2, leafy * 0.85) + light.leaf * leafy;
    let k = froxel_slice(at);
    let span = froxel_end(k) - froxel_end(k - 1.0);
    // The mist through the cell's whole depth: the banks' noise at its two ends, the height's
    // shape (the layer's top, its fall) at eight places spread through it. Taken at the middle
    // alone, a low layer was all in a slice where the middle fell in it and none where it didn't,
    // so the land under a mist wore rings round the eye at the slices' distances, 50-120 blocks
    // deep out there (Ronald, 11-06, pt_compare_v2 "low over the river": "what is this weird, like
    // a ripple thing"). Jittered taps blended by temporal anti-aliasing left a dot pattern instead.
    let slice = trunc(froxel_slice(at));
    let a0 = froxel_end(slice - 1.0);
    let a1 = froxel_end(slice);
    // The noise at the cell's two ends, blended between them through it: a slice's end is the next
    // one's start, so the banks run on unbroken from slice to slice. Taken once at a cell's middle,
    // each slice had its own (and the layer's top with it), and faint bands stayed round the eye
    // where the slices meet (the ring fix's second run, 11-06).
    let banks0 = fog_banks_at(scene.eye.xyz + dir * a0, floor);
    let banks1 = fog_banks_at(scene.eye.xyz + dir * a1, floor);
    var d = 0.0;
    for (var i = 0; i < 8; i++) {
        let t = (f32(i) + 0.5) / 8.0;
        d += fog_density_with(scene.eye.xyz + dir * mix(a0, a1, t), floor, mix(banks0, banks1, t));
    }
    d /= 8.0;
    if d <= 0.0 {
        return vec4(0.0);
    }
    // Three taps of the shadow map through the cell's depth (a quarter, a half and three quarters
    // between `at`'s neighbours' ends), so a shaft narrower than a cell is part-lit, not all or none.
    var lit = (sun_reaches(dir * (at - span * 0.25)) + sun_reaches(dir * at) + sun_reaches(dir * (at + span * 0.25))) / 3.0;
    // The clouds' shadow in the air too (the HDR world): beams of sun through their gaps, the mist
    // under a cloud dim.
    if scene.frame.x > 0.5 && scene.cloud_shadow.x > 0.5 {
        lit *= soft_cloud_shadow(dir * at);
    }
    // Ultra (`core:froxels` 2, finer cells: `scene.frame.z` 4): six taps, the beams' edges finer.
    if scene.frame.z > 0.5 && scene.frame.z < 6.0 {
        lit = lit * 0.5 + (sun_reaches(dir * (at - span * 0.4)) + sun_reaches(dir * (at - span * 0.1)) + sun_reaches(dir * (at + span * 0.4))) / 6.0;
    }
    // With the deep layer (`OPT_MIST_DOMES`) the shade darker: the bounce all round less, the sky's
    // light in it less, so shafts and the hollows under the crowns show.
    let deep = clamp(fantasy_opt(OPT_MIST_DOMES), 0.0, 1.0);
    let shade = mix(1.0, MIST_DEEP_SHADE, deep * (1.0 - lit));
    return vec4(mix(shaded_sky * shade, light.sky, lit) + light.sun * lit + mist_bounce(d, lit) * shade + lightning_glow_at(dir * at) * 0.6, d);
}

// Sunlight scattered many times in a dense mist (the HDR world): a thick bank glows all round like
// a white surface in the sun, not only toward it, and light creeps into its shaded parts. Single
// scattering alone (forward-peaked) left the valley mist across the sun a dim brown-orange sheet
// under green hills, where Bliss's is a bright gold sea (spot A's layer sheet, 11-06). Only where
// the mist is dense (the valley layer, about 0.03 a block), so the thin haze up high stays clear.
fn mist_bounce(d: f32, lit: f32) -> vec3<f32> {
    if scene.frame.x < 0.5 {
        return vec3(0.0);
    }
    let p = palette();
    let warm = 1.0 - smoothstep(0.1, 0.65, scene.sun.y);
    let thick = smoothstep(0.003, 0.03, d);
    let tint = mix(vec3(1.0), vec3(1.0, 0.78, 0.5), warm);
    return p.sun * tint * light_strength() * mix(lit, 1.0, 0.45) * thick * 0.16 * look_opt(OPT_FOG, 1.0);
}

// How much snow lies on a surface with normal `n` at `pos` (world) with `sky` light: on what
// faces up under the open sky, in patches that join up as it deepens.
fn snow_on(n: vec3<f32>, pos: vec3<f32>, sky: f32) -> f32 {
    let deep = scene.ground.x;
    if DRY || deep <= 0.0 {
        return 0.0;
    }
    let q = vec3<i32>(floor(pos * 16.0));
    let spot = noise3(pos * 0.35) * 0.8 + hash3(q) * 0.2;
    // Patches while it is a dusting, every top covered from about 0.3: the snow layers near the
    // player (Terrain's base:snow_layer) cover the ground from 0.2, and at the old rate (full only
    // near 0.8) the land past them stayed green with white specks, a white disc round the player
    // (Terrain's edge look, 11-04).
    return smoothstep(0.85, 1.0, sky) * smoothstep(0.55, 0.9, n.y) * smoothstep(1.0 - deep * 3.2, 1.05 - deep * 3.2, spot);
}

// ---- haze and fog ----

// The air between the eye and a point `rel` away (camera relative): a haze the color of the sky
// behind it that thickens with distance and thins with height, full where the terrain ends. In a
// liquid or a dimension without sky, the engine's fog.
fn haze(c: vec3<f32>, rel: vec3<f32>) -> vec3<f32> {
    return haze_lit(c, rel, 1.0);
}

// `haze` for a point with `sky` (0..1) sky light: fog gathers only out under the sky.
// The water itself seen from inside it: teal, lit by the sky and the sun above (the engine's water
// fog is a dark navy). Its fog is blue rather than lava's red, so lava keeps the engine's.
fn in_water() -> bool {
    return in_liquid() && scene.fog_color.b > scene.fog_color.r;
}

fn underwater_color() -> vec3<f32> {
    let p = palette();
    let light = p.ambient * 1.4 + p.sun * light_strength() * 0.5;
    // A murky green-teal (Bliss's under water), not the bright cyan step 2 showed.
    return vec3(0.040, 0.19, 0.15) * light * 1.8;
}

// Where the water's surface is above the eye under water: the sea's, or just over the eye in a
// lake higher up.
fn water_surface() -> f32 {
    return max(SEA_LEVEL, scene.eye.y + 0.5);
}

// The water between the eye and a point `rel` away, from inside it: bright teal near the surface,
// darker as it goes down, clear for about 20 blocks (the engine's water fog is a few), and beams of
// sunlight slanting down through it, broken up by the waves above. Six steps along the way.
fn under_water(c: vec3<f32>, rel: vec3<f32>) -> vec3<f32> {
    let dist = length(rel);
    let surface = water_surface();
    let y = scene.eye.y + rel.y;
    let down = exp(-max(surface - y, 0.0) / 28.0);
    let fog = underwater_color() * mix(0.35, 1.0, down);
    var out = mix(c * mix(0.6, 1.0, down), fog, 1.0 - exp(-dist / 18.0));
    let l = light_dir();
    if l.y > 0.05 && light_strength() > 0.0 {
        let reach = min(dist, 32.0);
        let t = scene.params.w;
        var beams = 0.0;
        for (var i = 0; i < 6; i++) {
            let at = rel * ((f32(i) + 0.5) / 6.0) * (reach / max(dist, 1e-4));
            let wy = scene.eye.y + at.y;
            // Straight up the light to the surface: where it came through.
            let q = (at.xz + scene.eye.xz) + l.xz / l.y * (surface - wy);
            let n = noise3(vec3(q.x * 0.18, t * 0.06, q.y * 0.18));
            beams += pow(1.0 - abs(n * 2.0 - 1.0), 6.0) * exp(-max(surface - wy, 0.0) / 20.0);
        }
        let p = palette();
        out += p.sun * light_strength() * vec3(0.55, 0.95, 0.85) * beams * (reach / 6.0) * 0.012;
    }
    return out;
}

// How much of what lies `rel` away (camera relative, `dist` its length) the air veils, per channel:
// clear air's blue haze, the grey of fog and damp air, rain's mist (`haze_lit`). What is seen
// through the water (`world.wgsl`) is veiled by it too.
fn haze_veil(rel: vec3<f32>, dist: f32) -> vec3<f32> {
    let thin = exp(-max(rel.y, 0.0) / 90.0);
    // The air's own depth, whatever the view distance: with distant terrain the fog's end lies
    // thousands of blocks off, and the haze would barely start where the kept blocks give way to it.
    // Clear air veils by wavelength, as Bliss's and Photon's atmospheres do: the sky's blue
    // (Rayleigh) goes first, red hardly at all, so far land goes a little blue and keeps its
    // contrast. And it starts further out (Ronald, 11-05: "it should be a bit hazy ... that's in
    // real life the same ... but in the close proximity it can be a bit more clear"): next to
    // nothing within about 150 blocks, rising steeply past it to the far haze as it was (in the
    // blue, about a fifth of the way to the sky's colour at 1000 blocks). Grey haze (Mie) comes
    // with the conditions: rain, the weather's fog, a forest's damp air, and a little always.
    let k = look_opt(OPT_HAZE, 1.6) * mix(0.45, 1.0, thin);
    // A third thicker than it was (Far view, 11-05: Bliss's far land is about two thirds sky-blue
    // at 2000 blocks, ours was under 0.6): far land reads as far, the near stays as clear.
    let rayleigh = vec3(0.35, 0.6, 1.0) * 3.4e-4 * k;
    let mie = (0.4e-4 + 2.0e-4 * air_mist() + 0.8e-4 * min(canopy(), 1.0)) * k;
    let far = dist * smoothstep(90.0, 450.0, dist);
    // Rain's own mist starts at the eye (Ronald, 11-05: "when raining shouldn't it be more
    // misty??"): rain alone a soft veil (a third gone at 100 blocks), a thunderstorm down to about
    // 150 (at twice that, a thunderstorm went white).
    // Its mist mainly in the distance, as Bliss's: next to none within about 20 blocks, so the near
    // land keeps its colour (it began at the eye and the whole land went grey).
    // (Then thinner, and darker in `sky_color`: at 0.0035 + 0.0055 a storm hid all but the nearest
    // land in a flat light grey, a wash brighter than the deck over it, from the vista 200 blocks up
    // nothing but grey (Tester 2's weather sheet, 11-05). Now a third gone at about 180 blocks in
    // rain, about 90 in a storm: the land keeps its shapes and colours, darker, under a slate sky.)
    return vec3(1.0) - exp(-((rayleigh + vec3(mie)) * far + vec3(wet_depth(dist))));
}

// Rain's mist between the eye and what lies `dist` blocks off, as an optical depth (`haze_veil`);
// the clouds take it too (`sky.wgsl`), so a deck and the land at the same distance go the same
// colour and meet at the horizon without a line.
fn wet_depth(dist: f32) -> f32 {
    // A quarter of what it was (0.0022 + 0.0030): a storm's land lost 60 % of its colour at 200
    // blocks, one veil over the whole land, contrast 7-11 on the forest vista where Photon's and
    // Unbound's stay green and readable under the deck (the Look session's rain check, 11-05). Now
    // the land at 100-300 blocks keeps most of its own colour (a storm a fifth gone at 300), the
    // veil builds far out (two thirds gone at 1500), and the horizon still fades into the deck.
    let downpour = rain_amount() * (0.0006 + 0.0007 * scene.weather.w);
    return downpour * max(dist - 20.0, 0.0) * smoothstep(10.0, 60.0, dist);
}

// The HDR world's low evening mist band over the far land (`haze_lit`): its density per block at
// its floor and its depth (blocks).
const BAND_HDR: f32 = 0.006;
const BAND_DEPTH: f32 = 12.0;

fn haze_lit(c_in: vec3<f32>, rel: vec3<f32>, sky: f32) -> vec3<f32> {
    let c = fog_through(c_in, rel, smoothstep(0.25, 0.75, sky));
    // The mist banks' debug views (3 to 5) alone, without the haze after them.
    if look_debug() >= 3u && look_debug() <= 5u {
        return c;
    }
    let dist = length(rel);
    if !CALM && in_water() {
        return under_water(c, rel);
    }
    if in_liquid() || !open_sky() {
        return mix(c, scene.fog_color.rgb, fog_amount(dist, scene.params.x, scene.params.y));
    }
    let dir = rel / max(dist, 1e-4);
    let end = scene.params.y;
    // Straight down there is no level direction left: never a zero vector (its normalize is NaN,
    // and the Mac's fast math has no safety net).
    // Aerial perspective: the air's own blue, the sky's a little above the horizon (about 6°, a
    // third of the way to the zenith's colour), not the horizon's pale band: veiled toward that, far
    // land went milky white and lost its contrast, where Bliss's goes a deep sky blue (Far view, the
    // Distant Horizons comparison, 11-05). It hands over to the horizon's colour where the terrain
    // ends, so the last ridges still meet the sky without a line, standing a little bluer than it.
    // Each sky colour only where it counts: nearer than the handover (most of the land) the
    // horizon's has no weight, and at the end the blue none, so neither is worked out there (the
    // same colour; each is a dozen powers, Android port, 11-06).
    let to_horizon = smoothstep(end * 0.55, end, dist);
    let air = vec3(dir.x, max(dir.y, 0.0) * 0.35 + 1e-3, dir.z);
    var color: vec3<f32>;
    if to_horizon <= 0.0 {
        color = sky_color(normalize(vec3(dir.x, max(dir.y, 0.0) * 0.35 + 0.1, dir.z)));
    } else if to_horizon >= 1.0 {
        color = sky_color(normalize(air));
    } else {
        let blue = sky_color(normalize(vec3(dir.x, max(dir.y, 0.0) * 0.35 + 0.1, dir.z)));
        color = mix(blue, sky_color(normalize(air)), to_horizon);
    }
    // In a dust storm the far land goes into the dust, as the sky does (it is fogged over 220
    // blocks): veiled toward the clear sky's colour, far ridges stood blue-lavender against a
    // sand-coloured sky (the dust-layers sheet, 11-06). The fog's own light at full depth.
    let dust = clamp(scene.ground.z, 0.0, 1.0) * smoothstep(0.0, 0.3, air_mist());
    if !CALM && dust > 0.0 {
        let fl = fog_light(dir);
        color = mix(color, fl.sky + fl.sun, dust);
    }
    let rain = rain_amount();
    // Thinner looking up at what is high, so hills stand out against a hazy valley (`haze_veil`).
    let veil = haze_veil(rel, dist);
    // Only the last sixth of the view hands over to the sky's colour where the terrain ends.
    let edge = smoothstep(end * 0.84, end, dist);
    // Valheim's air: lit from behind by the sun, so the haze toward it glows warm (a forward
    // scattering lobe, Henyey-Greenstein) and the side away goes a cool blue-teal, most at a low
    // sun. Toward the far end it hands back to the sky's own color, which the terrain fades into.
    let p = palette();
    let mu = dot(dir, light_dir());
    let g = 0.7;
    let lobe = min((1.0 - g * g) / pow(1.0 + g * g - 2.0 * g * mu, 1.5) / 12.566, 0.9);
    let near_end = 1.0 - edge;
    var lit = color + p.glow * light_strength() * lobe * mix(0.25, 0.7, golden()) * (1.0 - rain) * near_end;
    lit = mix(lit, lit * vec3(0.86, 0.97, 1.08), (0.5 - 0.5 * mu) * golden() * 0.6 * near_end);
    // Rain's mist is as dark as the rain's sky, darker in a thunderstorm: lit as a clear day's
    // air, a storm's mist was pale under a dark sky.
    // (The sky's colour it starts from is already the rain's; darker than it, the land under a
    // storm's horizon met a bright band of sky.)
    // (Now in `sky_color` itself, low in the sky: the deck fades into the same colour at the
    // horizon, the sky's gradient there too, so no brighter band lies between the deck and the land.)
    // Far ridges still lose a little colour (as before, but only where the veil has begun), then
    // the veil per channel (its blue thicker than its red), then the handover at the end.
    // In rain everything takes a little grey-blue wash, near too.
    // (Clear air takes less of its colour: a fifth of the veil, a tenth at most, as far biome colours
    // under Bliss stay saturated, a snowfield white and a forest green; rain's grey as before.)
    let grey = mix(clamp(veil.b * 0.2, 0.0, 0.1), clamp(veil.b * 0.6, 0.0, 0.3), max(rain, air_mist()));
    let pale = mix(mix(c, vec3(luma(c)), grey), vec3(luma(c)) * vec3(0.92, 0.97, 1.05), rain * 0.08);
    // Haze keeps the land's colour (Ronald, 11-05, on the glass slabs: "Haze keeps land colour"):
    // in clear air the veil stops at four fifths, the land veils toward a tenth under the sky's
    // brightness, and its hue and saturation take only seven tenths of the veil (its brightness all
    // of it). Veiled all the way to the sky's own colour, pale bare rock 400-500 blocks off read as
    // glass, the land behind seen through it. Rain and the weather's fog veil as before (the Shader
    // session's), and the handover where the terrain ends is still the sky's.
    let wet = clamp(max(rain, air_mist()), 0.0, 1.0);
    var v = mix(min(veil, vec3(0.8)), veil, wet);
    var toward = lit * mix(0.9, 1.0, wet);
    // The clear air's haze takes the sun's colour, most with it low, not the sky's blue: Bliss's 08:00
    // far land is warm (red minus blue +27), ours was blue-grey (-15) at the same brightness, and at
    // noon Bliss's is near neutral (saturation 54, -19) where ours was bluer and more saturated (87,
    // -48; the Look session's measures on the forest vista, 11-05). 0.6 of the way to the sun's hue
    // with it under about 33°, easing to 0.3 by 64°, and the veil's channels evened out, about a third
    // with the sun low, half at noon (less of Rayleigh's blue; half took too much of 08:00's
    // colour, and seven tenths left the noon far land grey where Bliss's is a coloured blue-green:
    // saturation 35 against 54 at spot C, 11-05), and at noon the veil a little darker. Tuned at the
    // test world's like-for-like vista (base:mc_forest, the Look session's measures, 11-06): with
    // three tenths and a quarter darker, from spot C, the noon far band came out darker and bluer
    // than Bliss's (luma 124, saturation 66, warmth -31 against 148, 54, -19). Less
    // evening with the sun low left 08:00 greyer, not more coloured (saturation 36 to 33). The HDR path only
    // (`scene.frame.x`), so today's look stays as it is.
    let hdr_air = select(0.0, 1.0, scene.frame.x > 0.5) * (1.0 - wet) * smoothstep(-0.02, 0.06, scene.sun.y);
    if hdr_air > 0.0 {
        let noon = smoothstep(0.55, 0.9, scene.sun.y);
        let low_sun = mix(0.3, 1.0, 1.0 - noon);
        let sun_tint = p.sun / max(luma(p.sun), 1e-3);
        // With the sun about 20-40° up, nine tenths far off, from 250 blocks to 800, easing back to six
        // by 45° so noon holds: the 08:00 far band stayed grey at six (saturation 36 and warmth +17
        // against Bliss's 58 and +27), and at nine everywhere the land nearer went past Bliss's (99
        // against 72; the Look session, 11-06).
        let morning = smoothstep(0.2, 0.34, scene.sun.y) * (1.0 - smoothstep(0.6, 0.7, scene.sun.y));
        let far_off = smoothstep(250.0, 800.0, dist);
        // In the evening with the sun low in the west (it rises at +x), a third of it by about 10° up:
        // Bliss's far land at 18:00 is near neutral (warmth +0 to +9 by the horizon) where ours
        // took the sun's gold (+43), and its mornings stay warm (07:00 +72), so not then (lk_ebc).
        let dusk = smoothstep(0.05, -0.05, scene.sun.x) * (1.0 - smoothstep(0.15, 0.3, scene.sun.y));
        // Toward the sun the haze takes its colour, away from it far less (a quarter): looking away
        // from a low sun Bliss's far land is cool under a blue-grey band (its spot C at 08:00:
        // land warmth -19, ours +46, a warm strip under the band's grey; toward it, at the forest
        // vista, warm, +27). Taking it alike every way gave that seam (rt-sun-shadows, Shader).
        let toward_sun = mix(0.25, 1.0, smoothstep(-0.3, 0.6, mu));
        toward = mix(toward, luma(toward) * sun_tint, mix(0.6, 0.9, morning * far_off) * low_sun * hdr_air * haze_sun() * mix(1.0, 0.35, dusk) * toward_sun) * (1.0 - 0.05 * noon * hdr_air);
        v = mix(v, vec3((v.r + v.g + v.b) / 3.0), mix(0.35, 0.5, noon) * hdr_air);
    }
    let mixed = mix(pale, toward, v);
    let own = mix(pale, toward, v * mix(0.7, 1.0, wet));
    let seen = max(vec3(luma(mixed)) + own - vec3(luma(own)), vec3(0.0));
    var out = mix(seen, lit, edge);
    // The HDR world with the sun low: a thin mist lying on the far land at about the sea's height,
    // a flat white band along the horizon with hills standing out of it, as Bliss's evening (its
    // 17:00-18:00 vista: rows at 40% of the height 20 brighter than round them, saturation about
    // 20; none at noon). Only far off (from about 900 blocks), so the near land stays clear, and
    // as deep as a look runs inside the layer, so it is a band, not a veil.
    if hdr_air > 0.0 {
        // (Through 17:00 too, the sun 22° up: from 0.2 it was a third there and barely showed, lk_eb3.)
        let low = (1.0 - smoothstep(0.32, 0.52, scene.sun.y));
        // (After the handover to the sky where the terrain ends: before it, the farthest land, where
        // the band lies, took the sky's colour over it, lk_eb1.)
        // (From 600 blocks it lay on the nearer hills too, a wide pale wash, not Bliss's thin band
        // at the far horizon: lk_eb2.)
        let far_band = smoothstep(1000.0, 2200.0, dist) * low;
        if far_band > 0.0 {
            let y = scene.eye.y + rel.y - (SEA_LEVEL + 4.0);
            let inside = min(min(BAND_DEPTH / max(-dir.y, 0.008), dist), 1500.0);
            let band = 1.0 - exp(-BAND_HDR * exp(-max(y, 0.0) / BAND_DEPTH) * inside * far_band);
            // A cool white, a little brighter than the haze round it: at 1.25 and warm it read
            // 188-190 / saturation 49-52 / warmth +40 at 18:00 against Bliss's 172-174 / 19-22 / +3
            // (on the blue horizon band, lk_ebh2).
            let white = vec3(luma(toward)) * vec3(0.96, 1.0, 1.06) * 1.12;
            out = mix(out, white, band);
        }
    }
    // The held light in the air and a bolt before far land: none in a calm scene (`CALM`), and
    // their code left out of its shader (Android port: on the Pixel 10 the size of these rare
    // paths, the fog walk, the dust and the water's cost the vista 34.2 -> 26.0 ms, in a calm
    // scene drawing nothing).
    // The HDR world by day: the near air's own light added over what is seen, as Bliss's clear air
    // adds it (its overworld_fog.glsl atmosphere haze, purely additive: the land keeps its light),
    // built over the first 250 blocks and gone by 700, where the far haze above takes over. Its
    // colour is the sky's of the hour: with the sun high the sky's blue, nearly pure (Bliss's noon
    // vista adds about +0.10 blue to the shade 100-300 blocks off, linear, green flat: our leaf sides
    // were green 48 56 44 against its navy 30 54 100); a fifth of it in the morning (08:00 +0.02);
    // with the sun at the horizon the sunset band's pink (its 18:00 shade gains red 1, green 0.14,
    // blue 0.45: warm grey 61 59 45 where ours stood a bare green 35 56 30). Trees' bands, 11-06.
    if scene.frame.x > 0.5 {
        let day_air = smoothstep(-0.08, 0.02, scene.sun.y) * (1.0 - rain_amount());
        if day_air > 0.0 {
            let fade = 1.0 - smoothstep(300.0, 700.0, dist);
            let high = smoothstep(0.45, 0.8, scene.sun.y);
            let low = 1.0 - smoothstep(0.03, 0.3, scene.sun.y);
            let near = vec3(1.0) - exp(-vec3(4.9e-5, 1.0e-4, 3.5e-4) * min(dist, 250.0));
            let blue_sky = sky_color(normalize(vec3(dir.x, max(dir.y, 0.0) * 0.35 + 0.3, dir.z)));
            // With the sun low a fifth of it as made; `base:look_low_air` sets that share (Bliss's
            // 08:00 and 18:00 shade is blue, 9 26 58 and 28 35 58, ours green 11 16 22 and 41 47 30:
            // its near air keeps the sky's blue with the sun low, as it is lit by the whole sky).
            let low_k = fantasy_opt(OPT_LOW_AIR);
            var add = blue_sky * near * mix(select(0.2, low_k, low_k > 0.0), 1.0, high);
            // With the sun low the air takes the sun's own colour through its phase, as Bliss's
            // clear air does (overworld_fog.glsl:256-268: sun x shadow x (Rayleigh phase x its
            // coefficient + Mie phase x its one), plus the averaged sky): gold all round, brightest
            // toward the sun. A fixed pink here turned water and lit land violet at 18:00 against
            // Bliss's golden air (lk_na sheets, 11-06), though the shade's numbers matched.
            let mu = dot(dir, normalize(scene.sun.xyz));
            // Rayleigh's phase (3/4 (1 + mu^2)) and a forward Mie lobe, weighted 0.8 / 0.2 as
            // Bliss's coefficients' luma near the sea (Rayleigh x24 x1.25 against Mie), scaled to 1
            // at right angles to the sun so the side views keep the strength that matched.
            let mie = min(hg_phase(mu, 0.7) / hg_phase(0.0, 0.7), 6.0);
            let phase = (0.8 * 0.75 * (1.0 + mu * mu) + 0.2 * mie) / 0.8;
            let gold = p.sun / max(luma(p.sun), 1e-4);
            let grey = 1.0 - exp(-2.5e-4 * min(dist, 250.0));
            add += gold * luma(p.horizon) * phase * grey * low * 0.6;
            out += add * day_air * fade * evening_air_fade();
        }
        // The blue hour's air (`OPT_TWILIGHT`): after sunset Bliss's land is a deep blue to the
        // near ground (19:00 land 5 23 78, far 47 72 164; bliss_tw, 11-22), its own light near
        // gone and the air between lit by the blue sky. Ours lit the land by that sky and it went
        // grey-green, 22 36 40 (lsO_tw2), not blue. So the air adds the twilight sky's colour,
        // blue first, filled in over the first few hundred blocks, at the setting's strength.
        let blue_k = max(fantasy_opt(OPT_TWILIGHT), 0.0);
        if blue_k > 0.0 {
            let evening = 1.0 - smoothstep(-0.05, 0.05, scene.sun.x);
            // From sunset: begun with the sun 6° up, the 18:30 land went a grey-blue (lsR_tw 0.5: 87 83 88
            // where Bliss's is dim and warm, 35 36 69).
            // From about 6° up, under a third by 4° (18:30): with the sun dimmed there the land is
            // Bliss's blue haze over dark ground, 35 38 74. From sunset ours stayed warm, 66 60 60
            // (lsY_tw); half by 4° overshot, 69 88 143 (lsZ_tw2).
            let tw = twilight_window() * (1.0 - smoothstep(0.0, 0.1, scene.sun.y)) * evening * (1.0 - rain_amount());
            if tw > 0.0 {
                let fill = vec3(1.0) - exp(-vec3(1.5e-3, 3.0e-3, 6.0e-3) * min(dist, 600.0));
                let sky_up = sky_color(normalize(vec3(dir.x, max(dir.y, 0.0) * 0.35 + 0.3, dir.z)));
                out += sky_up * fill * tw * blue_k * evening_air_fade();
            }
        }
    }
    var extra = vec3(0.0);
    if !CALM {
        extra = held_in_fog(rel, sky) + lightning_bolt_over(dir, dist);
    }
    return out + extra;
}

// ---- light in the fog: torches (Research round 15, 6: Valheim's warm firelight in the mist) ----

// How thick the air is about the eye, per block: the weather's mist where the player stands, rain's
// own, and a little always; `sky` (0..1) the sky light where it is seen, as the mist and the rain
// are out under the sky only.
fn fog_here(sky: f32) -> f32 {
    let mist = fog_density(scene.eye.xyz, fog_floor(vec3(0.0)));
    let rain = rain_amount() * (0.0035 + 0.0055 * scene.weather.w);
    return (mist + rain) * smoothstep(0.2, 0.7, sky) + 0.0015;
}

// The held light (`scene.hand`) scattered by the air between the eye and `rel`, `sigma` the air's
// thickness (`fog_here`). A point light in even fog has a closed form along a straight look (the
// airlight integral, without its dimming): the light falls off as the square of the distance, so
// the integral of 1 / (h^2 + (t - t0)^2) over the look, h how far the look passes the light. The
// flame is no point: h is never under about 0.7. Looking where the torch is held the glow is
// strongest; looking back, half as much.
// `sky` the sky light where it is seen (`fog_here`'s), asked only when a light is held: as an
// argument the mist at the eye (two noises) was worked out for every pixel of the world, light or
// none (Distant terrain's probe: 0.13 ms of the far land's 1.5 at 3456).
fn held_in_fog(rel: vec3<f32>, sky: f32) -> vec3<f32> {
    let l = scene.hand_light.rgb;
    if max(max(l.x, l.y), l.z) <= 0.0 {
        return vec3(0.0);
    }
    let sigma = fog_here(sky);
    let len = length(rel);
    let dist = min(len, 48.0);
    let dir = rel / max(len, 1e-4);
    let at = scene.hand.xyz;
    let t0 = dot(at, dir);
    let h = sqrt(max(dot(at, at) - t0 * t0, 0.0) + 0.5);
    let along = (atan((dist - t0) / h) + atan(t0 / h)) / h;
    // Lit as surfaces are (a block away, its full light), scattered all round. Thick mist is capped:
    // with its whole thickness, and a quarter of what a surface sends back, a torch in a dawn bank
    // washed the whole view beige, the sky too (the after-rain check, 11-05).
    // At 0.07 it hardly read at a sheet's size (+11 of 255 in red); twice that reads as firelight in
    // the mist without washing it.
    return block_light(l, scene.eye.xyz + at) * min(sigma, 0.04) * along * 0.14;
}

// Placed lights in the fog: the air just before a lit surface glows with the light there (`b`, its
// block light levels), as deep as a few blocks of fog. Cheaper than walking the light: it stops at
// the silhouettes, which the bloom softens.
// `sky` as `held_in_fog`'s, asked only where there is block light.
fn placed_in_fog(b: vec3<f32>, rel: vec3<f32>, pos: vec3<f32>, sky: f32) -> vec3<f32> {
    if max(max(b.x, b.y), b.z) <= 0.0 {
        return vec3(0.0);
    }
    return block_light(b, pos) * min(fog_here(sky), 0.04) * min(length(rel), 4.0) * 0.6;
}

// ---- to the screen ----

// Light as it is up to `KNEE`, then a soft shoulder that never quite reaches white: the darks
// stay as vanilla shows them and only what is brighter than the screen is folded back in.
const KNEE: f32 = 0.6;

fn shoulder(x: vec3<f32>) -> vec3<f32> {
    let over = max(x - vec3(KNEE), vec3(0.0));
    return min(x, vec3(KNEE)) + (1.0 - KNEE) * (vec3(1.0) - exp(-over / (1.0 - KNEE)));
}

// The tone curve and a little more color: linear light for the sRGB screen.
// The grade on top: the darks a little cool and the lights warmer as the sun goes down (warm
// light, blue shade), colour held back a little (vanilla's, not a saturated look), and a cool
// floor so nothing is ever pure black.
// Jim Hejl's 2015 filmic curve (his published formula): a toe that deepens the darks and a long
// shoulder that keeps bright skies and sunlit faces from flattening; its output is display-coded,
// so it is brought back to linear for the sRGB screen.
// Timothy Lottes's tone curve, our code from his published formula: x^a / (x^(a d) b + c), with b
// and c solved so that 0.25 maps to 0.25 and 8 to 1 (contrast a 1.05, shoulder d 0.97). Display-
// linear out, black at black.
fn lottes(x: vec3<f32>) -> vec3<f32> {
    // The contrast knob scales a (1.2 as made: Complementary's 1.05 read flat, Colour 9); b and c
    // follow from it.
    let a = 1.2 * look_opt(OPT_CONTRAST, 1.0);
    let ad = a * 0.97;
    let mid = 0.25;
    let peak = 8.0;
    let den = (pow(peak, ad) - pow(mid, ad)) * mid;
    let b = (-pow(mid, a) + pow(peak, a) * mid) / den;
    let c = (pow(peak, ad) * pow(mid, a) - pow(peak, a) * pow(mid, ad) * mid) / den;
    let xa = pow(max(x, vec3(1e-6)), vec3(a));
    return xa / (pow(max(x, vec3(1e-6)), vec3(ad)) * b + vec3(c));
}

// The light under Hejl's toe kept at this share of itself (`finish_picture`).
const HEJL_TOE: f32 = 0.25;

// The HDR world's daytime warmth on sunlit foliage (`finish_picture`): how far it goes toward a
// leaf-top yellow (`base:look_split_warm`, 0.7 by default).
fn split_warm() -> f32 {
    return clamp(fantasy_opt(OPT_SPLIT_WARM), 0.0, 1.0);
}
// 1 where the picture's pixel is land, 0 sky or cloud (set by `hdr_light` before `finish_picture`).
var<private> LAND_PIXEL: f32 = 1.0;

fn hejl(c: vec3<f32>, white: f32) -> vec3<f32> {
    let v = vec4(c, white);
    let a = 1.425 * v + vec4(0.05);
    let f = (v * a + vec4(0.004)) / (v * (a + vec4(0.55)) + vec4(0.0491)) - vec4(0.0821);
    return pow(max(f.rgb / f.w, vec3(0.0)), vec3(2.2));
}

// AgX, Troy Sobotka's display transform, in Benjamin Wrensch's minimal fit (MIT License,
// Copyright (c) 2024 Missing Deadlines (Benjamin Wrensch), iolite-engine.com): the light into a
// log space between -12.47 and +4.03 stops round mid-grey, through a polynomial S curve, back out of
// its inset primaries. Brights roll off toward white in their own hue instead of clipping.
fn agx_curve(x: vec3<f32>) -> vec3<f32> {
    let x2 = x * x;
    let x4 = x2 * x2;
    return 15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4 - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - vec3(0.00232);
}

// Linear light in, display-linear out (as `lottes`), for the sRGB screen.
fn agx(c: vec3<f32>) -> vec3<f32> {
    let inset = mat3x3<f32>(
        vec3(0.842479062253094, 0.0423282422610123, 0.0423756549057051),
        vec3(0.0784335999999992, 0.878468636469772, 0.0784336),
        vec3(0.0792237451477643, 0.0791661274605434, 0.879142973793104),
    );
    let outset = mat3x3<f32>(
        vec3(1.19687900512017, -0.0528968517574562, -0.0529716355144438),
        vec3(-0.0980208811401368, 1.15190312990417, -0.0980434501171241),
        vec3(-0.0990297440797205, -0.0989611768448433, 1.15107367264116),
    );
    let min_ev = -12.47393;
    let max_ev = 4.026069;
    let v = clamp(log2(max(inset * c, vec3(1e-10))), vec3(min_ev), vec3(max_ev));
    let shaped = agx_curve((v - min_ev) / (max_ev - min_ev));
    return pow(clamp(outset * shaped, vec3(0.0), vec3(1.0)), vec3(2.2));
}

// How bright the picture is taken: the filmic curve deepens the mid-tones, so by day it is opened
// up to keep them where they were, and more as night falls and under water, the eye adjusting.
fn exposure() -> f32 {
    // Under water a little less than in lava's glare (3.0 there): it washed the water to cyan.
    if in_water() {
        return 2.2;
    }
    if in_liquid() {
        return 3.0;
    }
    // By day 2.2 (it was 1.7): with less of the sky's light in the shade (Colour 1) the picture went
    // darker than Bliss's (mean brightness 0.55 against about 0.6, shallows dark navy). Golden hour
    // keeps 1.7, its palette made for it.
    let day = mix(1.7, 2.2, smoothstep(0.1, 0.35, scene.sun.y));
    // The night's 4.0 only once the sun is well down: rising from the horizon, it showed the dusk
    // palette (made for about 1.7) twice as bright, a light lavender sky at 04:00 and 19:30 where
    // Bliss's blue hour is deep blue (the MC comparison's spring 8 sheets, 11-05). Night unchanged.
    // (It was smoothstep(0.0, -0.2).)
    return mix(day, 4.0, smoothstep(-0.1, -0.3, scene.sun.y));
}

// What a surface writes: into the HDR world (`scene.frame.x`, the engine's `core:hdr_world`) its light
// as it is, linear and past 1, which the screen effects work on and the effects' last pass then
// finishes (`finish_picture`, in this pack's post.wgsl); into the screen, finished here as always.
fn display(c: vec3<f32>) -> vec3<f32> {
    if scene.frame.x > 0.5 {
        // At night the dim world seen blue, at its own brightness (the eye's night sight): Bliss's
        // moonlit land is a deep blue (22:00 sRGB 31 63 134, saturation about 200-250) where ours
        // stood a neutral green-black (18 27 26, about 100), its sky a grey-blue (Bliss's world, mist
        // off, PC shnt0, 11-06). Only the dim: a torch's light keeps its warmth, and the night
        // stays as dark as made.
        // Only the sky's and moon's share of a surface's light (`NIGHT_SKY_SHARE`): a lamp's keeps
        // its warmth in a dark room as outdoors.
        // By how dim the world's light is, not by the hour, as Bliss's rods (its Purkinje: strong
        // when the view is dark, on its dim pixels): the night and the blue hour before sunrise and
        // after sunset alike, gone as the sun's light comes. Bliss's 05:00 land is a deep blue (sRGB
        // 32 70 125, warmth -93), ours stood grey-violet (25 24 30) with the sun 3 degrees under, past
        // this blue's old gate at 7 (sun.y -0.12; Mist's m11 sheet, 11-06). And how dim a pixel is
        // as seen, at the exposure it is taken at (at night's 4.0 as before).
        let p = palette();
        // Full in the night and the morning's blue hour (that light about 0.08-0.11), about two
        // thirds in the evening's brighter twilight (about 0.19), as Bliss's 19:00 keeps its greens
        // and the cherry trees' pink in the blue where its 05:00 doesn't; gone by the sunset's 0.5.
        // (0.15 to 0.5 took 19:00 a uniform blue, the trees' green gone, PC shprC; 0.08 to 0.3 with
        // the morning's rise in the evening too kept hardly any, warmth -17 against -61, shprA.)
        let world = 1.0 - smoothstep(0.12, 0.3, luma(p.ambient) + 0.5 * luma(p.sun));
        let seen = luma(c) * exposure() / 4.0;
        let night = world * (1.0 - smoothstep(0.04, 0.3, seen)) * NIGHT_SKY_SHARE;
        if night > 0.0 {
            // (N2: deeper and all the way: at 0.3 0.6 1.3 and three quarters the land's saturation rose
            // only from about 100 to 140, the curve taking some back; Bliss's is 200-250, PC shnt1.)
            // (N3: at 0.18 0.5 1.6 all the way the trees went a uniform snow-blue, Bliss's keep a
            // dark green in the blue: a little green kept, 0.85 of the way; PC shnt2.)
            let blue = vec3(0.24, 0.56, 1.4);
            return max(mix(c, blue * (luma(c) / luma(blue)), night * 0.85), vec3(0.0));
        }
        return max(c, vec3(0.0));
    }
    return finish_picture(c);
}

// The picture finished: exposure, the tone curve and the grade, from linear light to the screen.
fn finish_picture(c_in: vec3<f32>) -> vec3<f32> {
    // The HDR world's morning twilight (the sun 0-7° under, in the east): dim land brought up after
    // the exposure, as Bliss's eye lifts its dim twilight land after its own (up to about 3x), where
    // ours took back a brighter twilight in the meter's square root (05:00 land luma 35 against
    // Bliss's 66 with Shader's pre-dawn blue; its runs shpr0/A/B, 11-06). Up to 2.35x: at 2.2 the
    // land came to 61 (Shader's shprJ6). 04:00 stays dark.
    var c = c_in;
    if scene.frame.x > 0.5 {
        let dawn = smoothstep(-0.13, -0.04, scene.sun.y) * (1.0 - smoothstep(0.0, 0.08, scene.sun.y)) * smoothstep(-0.2, 0.2, scene.sun.x);
        let dim = 1.0 - smoothstep(0.02, 0.15, luma(c));
        c *= 1.0 + 1.35 * dawn * dim * LAND_PIXEL;
    }
    // By day Timothy Lottes's curve (his published formula, GDC 2016; Complementary's settings:
    // contrast 1.05, mid-grey 0.25 in and out, peak 8) at exposure 1.3: black stays black (no toe
    // lift, which raised the black point and halved saturation: Colour 4-7) and the darks keep their
    // colour, where Hejl's toe crushed them. At night and under water Hejl's as before (nights
    // dark by design), blended as the sun comes up.
    let day = smoothstep(-0.05, 0.15, scene.sun.y) * select(1.0, 0.0, in_liquid());
    let night_t = hejl(max(c, vec3(0.0)) * exposure(), 3.0);
    var day_t = min(lottes(max(c, vec3(0.0)) * 1.3 * look_opt(OPT_EXPOSURE, 1.0)), vec3(1.0));
    // Or Hejl's 2015 curve by day (`base:look_curve` 1, the HDR world's default: Ronald's pick of
    // 11-05, "Hejl 2015", as in his Bliss settings): a real toe, so
    // the shade goes deep and the mids keep their colour, a long shoulder to white at 3.5.
    // Or AgX (`base:look_curve` 2): Blender's and Bliss's current default.
    // Only in the HDR world: without it the look stays as made (Lottes's), whatever the setting.
    let curve = select(0.0, fantasy_opt(OPT_CURVE), scene.frame.x > 0.5);
    if curve > 1.5 {
        day_t = agx(max(c, vec3(0.0)) * 0.55 * exposure() * look_opt(OPT_EXPOSURE, 1.0));
    } else if curve > 0.5 {
        // A floor under Hejl's toe, which takes everything under about 1% to black: there the
        // light at `HEJL_TOE` of itself, so deep shade keeps its colour (Bliss's near shade is a
        // dark blue, sRGB 19 39 65; ours went to 0 0 0 on Tester 2's calm noon, more fill light
        // didn't lift it, sh1b; at 0.15 the grass spot's darkest tenth was 17, at 0.25 24, today's
        // LDR 30, the land's middle and brights the same). Above about 0.1 the curve is Hejl's own.
        let x = max(c, vec3(0.0)) * exposure() * look_opt(OPT_EXPOSURE, 1.0);
        // Bliss's chain (its Hejl 2015 fit: 0.85 in, 1.75, white 3), its output taken as the
        // light itself with no gamma undone, at 0.389 of the light so mid-grey stays where it was:
        // deeper darks, steeper mids, a gentler shoulder that keeps bright leaf tops' colour (ours
        // undid the fit's baked-in gamma, so its darks sat on the toe floor and its brights went
        // pale; Look's reading of Bliss's source, 11-06).
        let vb = vec4(x * 0.389 * 0.85, 3.0);
        let ab = 1.75 * vb + vec4(0.05);
        let fb = (vb * ab + vec4(0.004)) / (vb * (ab + vec4(0.55)) + vec4(0.0491)) - vec4(0.0821 - 0.000633604888);
        day_t = max(fb.rgb / fb.w, vec3(0.0));
    }
    var t = mix(night_t, day_t, day);
    // The HDR world's white balance by day (`base:look_white`): warmer, less blue (our land read
    // much bluer than Bliss's at every hour, red less blue -37 against -10 at noon: the gap sheet).
    if scene.frame.x > 0.5 {
        // Mostly with the sun part way up: at noon a quarter (the sky and the far haze stay blue,
        // as Bliss's).
        // At golden hour none of the extra, and less of the rest: it was a cast over the whole
        // view, land and sky alike, on top of the palette's own gold, and at the like-for-like
        // mc_forest vista the HDR picture's 18:00 came out warmth +63 on the land and +99 on the
        // sky where today's picture from the same palette had +17 and +33, Bliss's +14 and -21
        // (Shader's evening sheet, 11-06). A low sun's gold is the palette's, the haze's and the
        // mist's to give.
        // In mist the gold kept, more of it (Bliss's misty dawn is all gold, 181 of saturation and
        // +106 warmth; with the cut, ours fell from 174/+101 to 151/+88, gr5m): the low sun's gold
        // is the mist's light.
        let misty = smoothstep(0.02, 0.1, air_mist()) * (1.0 - rain_amount());
        let w = fantasy_opt(OPT_WHITE) * day * mix(1.0, 0.25, smoothstep(0.4, 0.75, scene.sun.y)) * mix(1.0 - 0.7 * golden(), 1.0 + 0.6 * golden(), misty);
        // On the land only: on the sky it took the blue out of the horizon band at 08:00
        // (Shader's spot C rows, 11-06).
        let wl = w * LAND_PIXEL;
        t *= vec3(1.0 + 0.08 * wl, 1.0, 1.0 - 0.14 * wl);
    }
    // Only true darks go cool (most daylit ground is mid-tone and keeps its color).
    let lights = smoothstep(0.02, 0.22, luma(t));
    let shade = mix(vec3(0.975, 0.99, 1.03), vec3(0.92, 0.96, 1.10), golden());
    t *= mix(shade, mix(vec3(1.0), vec3(1.12, 1.0, 0.82), golden()), lights);
    // The HDR world by day: sunlit foliage and grass toward a leaf-top yellow, as Bliss's grade
    // (its lit leaf tops olive-yellow, red about equal to green, where ours were green at red 0.4 of
    // green: the vista from above, 11-06). Only what is green (water, haze, grey bark, sand and stone
    // keep their colour; on all the land it cast sepia over lakes and sand), and a green pixel stays
    // green, its red at most 0.95 of its green (grass between the crowns went straw-tan).
    if scene.frame.x > 0.5 {
        let l = luma(t);
        let k = day * (1.0 - rain_amount());
        // Green leading red as well: terracotta, sand and cream stone have more green than blue too,
        // and went pale yellow (our mesa's red bands, the nave's orange floor).
        let leafy_px = smoothstep(0.0, 0.08, (t.g - t.b) / max(l, 1e-4) * 0.5) * smoothstep(0.0, 0.04, (t.g - t.r) / max(l, 1e-4));
        let hi = smoothstep(0.06, 0.3, l) * k * split_warm() * LAND_PIXEL * leafy_px;
        let was_green = t.g > t.r;
        var warm = mix(t, vec3(l) * vec3(1.35, 1.08, 0.45), hi);
        if was_green {
            warm.r = min(warm.r, max(t.r, warm.g * 0.95));
        }
        t = warm;
    }
    // The near-black loses some colour: at night 60 % of it (a dim light's hue was pushed to a
    // saturated blue), by day a quarter below a tenth of luminance (Complementary's).
    let dim = luma(t);
    let keep = mix(0.4 + 0.6 * smoothstep(0.0, 0.05, dim), 0.75 + 0.25 * smoothstep(0.0, 0.1, dim), day);
    t = mix(vec3(dim), t, keep);
    // By day 1.2 of saturation after the curve (Colour 9; 1.07, Complementary's, read pale here).
    let y = luma(t);
    // In the HDR world 0.8 of it (its curve keeps more colour; at 1.2 the noon land measured 131
    // of saturation against Bliss's 75: the gap sheets, 11-05).
    // In the HDR world it follows the sun: 0.95 of it with the sun under about 17° (with the far
    // haze's gold, 1.12 overshot dawn: 222 against Bliss's 181), 0.78 of
    // it from about 48° (the match checkpoints: 08:00 100 and noon 89 against Bliss's 72 and 75).
    // With a high sun 0.62 (the like-for-like scene, Ley lines' mc_forest vista: at 0.78 the land
    // measured 97-101 against Bliss's 72-75 at 08:00 and noon, with Minecraft's own textures and
    // leaf colours as much: the grade's, not the art's; the sweep gr1 put 0.8 of it on Bliss's).
    // With a low sun 0.95 in mist (a gold dawn mist's colour) but 0.75 in clear air: on a clear
    // evening the land at 17:00 and 18:00 measured 139-153 of saturation against Bliss's 56-84
    // (mc_forest, gr4), where Bliss's misty mornings run 181.
    let low_sat = mix(0.75, 0.95, smoothstep(0.02, 0.1, air_mist()));
    // In Bliss's own world (imported block for block) the land with the sun 30-55° up came out
    // grey-green at the cut tuned on the test world (0.62 at noon): land saturation 52 against
    // Bliss's 72 at 08:00, 61 against 92 at noon, where 17:00 matched (75 against 83). A quarter
    // more there (the canopy sweep cs1: x1.25 took 08:00 to 67, noon to 78), as it was with the sun
    // under about 20°.
    let sat = 1.2 * look_opt(OPT_SATURATION, 1.0) * select(1.0, mix(low_sat, 0.85, smoothstep(0.38, 0.55, scene.sun.y)), scene.frame.x > 0.5);
    if scene.frame.x > 0.5 && curve > 0.5 && curve < 1.5 {
        // With Bliss's chain, its +30 % of colour in display space by day, in place of ours.
        let g = pow(max(t, vec3(0.0)), vec3(1.0 / 2.2));
        let gy = dot(g, vec3(0.299, 0.587, 0.114));
        t = pow(max(g + (g - vec3(gy)) * 0.3 * day, vec3(0.0)), vec3(2.2));
    } else {
        t = max(mix(vec3(y), t, mix(1.0, sat, day)), vec3(0.0));
    }
    // A cool floor so the night is never pure black; by day black stays black.
    t += (vec3(1.0) - t) * vec3(0.0018, 0.002, 0.0034) * (1.0 - day);
    // A storybook night (`base:look_storybook`): dim colours slide toward a blue-grey, as the eye's
    // rods take over (Purkinje; Photon's, 0.05 × its setting), firelight kept: warm pixels, whose
    // red outweighs their blue, keep their colour.
    let dim_px = 1.0 - smoothstep(0.02, 0.16, luma(t));
    let warm = smoothstep(-0.02, 0.06, t.r - t.b);
    let rods = clamp(fantasy_opt(OPT_STORYBOOK), 0.0, 1.0) * (1.0 - day) * dim_px * (1.0 - warm) * 0.45;
    t = mix(t, vec3(luma(t)) * vec3(0.84, 0.95, 1.18), rods);
    // A blood moon's faint red cast over the night.
    t *= mix(vec3(1.0), vec3(1.1, 0.88, 0.86), blood_moon() * 0.6 * (1.0 - day));
    return clamp(t, vec3(0.0), vec3(1.0));
}
