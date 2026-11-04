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
    return clamp(scene.fog_color.w, 0.0, 1.0);
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

fn under_leaves(sky_light: f32) -> f32 {
    // Not in deep cover (a cave in the forest): the leaves' light is from what the sky reaches.
    return min(canopy(), 1.0) * (1.0 - smoothstep(0.8, 1.0, sky_light)) * smoothstep(0.35, 0.6, sky_light);
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
fn palette() -> Palette {
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
    let golden = key(vec3(0.11, 0.25, 0.78), vec3(0.92, 0.62, 0.42), vec3(1.0, 0.58, 0.26), vec3(1.55, 0.80, 0.38), vec3(0.22, 0.24, 0.38));
    let sunset = key(vec3(0.08, 0.10, 0.38), vec3(1.05, 0.46, 0.34), vec3(1.0, 0.50, 0.26), vec3(1.50, 0.76, 0.38), vec3(0.27, 0.22, 0.36));
    let dusk = key(vec3(0.045, 0.05, 0.16), vec3(0.30, 0.15, 0.27), vec3(0.45, 0.20, 0.25), vec3(0.0, 0.0, 0.0), vec3(0.10, 0.09, 0.17));
    // Night is dark and blue, a little lighter than vanilla's, so lanterns carry the scene.
    // Night is dark and blue, so firelight's orange stands out against it. (A blue-teal shade, as
    // Valheim's, mixed with the orange of a fire into olive on the ground: Tester 2's campfire.)
    let night = key(vec3(0.006, 0.010, 0.030), vec3(0.025, 0.035, 0.070), vec3(0.10, 0.13, 0.22), vec3(0.17, 0.21, 0.30) * look_opt(OPT_NIGHT_MOON, 1.0), vec3(0.012, 0.017, 0.030) * look_opt(OPT_NIGHT_LIGHT, 1.0));
    var p: Palette;
    if e > 0.08 {
        p = blend(golden, day, smoothstep(0.08, 0.28, e));
    } else if e > -0.02 {
        p = blend(sunset, golden, smoothstep(-0.02, 0.08, e));
    } else if e > -0.14 {
        p = blend(dusk, sunset, smoothstep(-0.14, -0.02, e));
    } else {
        p = blend(night, dusk, smoothstep(-0.3, -0.14, e));
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
    p.ambient *= mix(1.0, mix(0.25, 6.0, scene.moon.w), smoothstep(-0.14, -0.3, e));
    // Overcast: grey and dim, and the sun no more than a brighter part of the sky.
    let r = rain_amount();
    let grey = vec3(luma(p.horizon)) * vec3(0.92, 0.95, 1.0);
    p.zenith = mix(p.zenith, grey * 0.75, r * 0.85);
    p.horizon = mix(p.horizon, grey, r * 0.85);
    p.glow = p.glow * (1.0 - r * 0.8);
    p.ambient = mix(p.ambient, vec3(luma(p.ambient + p.sun * 0.4)) * vec3(0.95, 0.97, 1.0), r * 0.85);
    p.sun = p.sun * (1.0 - r * 0.9);
    // At night rain and its clouds darken the sky's light too (by day overcast only greys it).
    p.ambient *= mix(1.0, 1.0 - 0.6 * r, smoothstep(-0.14, -0.3, e));
    return p;
}

// How much of golden hour and sunset there is: 0 by day and at night, 1 with the sun just above the
// horizon. The grade warms with it, and the glowing sky lights the ground.
fn golden() -> f32 {
    let e = scene.sun.y;
    return (1.0 - smoothstep(0.08, 0.35, e)) * smoothstep(-0.06, 0.03, e);
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
fn light_strength() -> f32 {
    let e = scene.sun.y;
    // At night the moon's light: its phase times how high it stands (`scene.moon.w`), none when it
    // is down or new.
    return select(smoothstep(0.06, 0.2, -e) * scene.moon.w, smoothstep(-0.04, 0.05, e), e > -0.06);
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
    // Away from the sun the sunset sky goes rose and violet.
    let away = (0.5 - 0.5 * mu) * (1.0 - h) * low;
    c = mix(c, c * vec3(0.97, 0.80, 1.06) + vec3(0.03, 0.0, 0.03), away * 0.4);
    // Toward it, a warmer band low on the horizon.
    let toward = pow(max(mu, 0.0), 2.0) * pow(1.0 - h, 3.0) * low;
    c += vec3(0.55, 0.22, 0.05) * toward;
    c += sun_glow(dir, p);
    // The glow about the moon where it stands, as bright as its light (`scene.moon.w`): on a new
    // moon there is none, and a glow alone read as a dark-blue ring (the world playtester's nights).
    let moon = pow(max(dot(dir, scene.moon.xyz), 0.0), 24.0) * smoothstep(-0.05, -0.25, scene.sun.y) * scene.moon.w;
    c += vec3(0.05, 0.07, 0.12) * moon;
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
fn shadow_in(m: mat4x4<f32>, info: vec4<f32>, round: f32, depth: f32, rel: vec3<f32>, n: vec3<f32>) -> vec2<f32> {
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
    // A 4x4 of the map about the point; each of the 3x3 taps takes the nearest to the light of its
    // texel and the ones right and below. That closes the one-texel cracks between neighbouring
    // meshes, through which sunlight would draw thin dotted lines across a shadow.
    var map: array<f32, 16>;
    for (var j = 0; j < 4; j++) {
        for (var i = 0; i < 4; i++) {
            let t = clamp(at + vec2(i - 1, j - 1), vec2(0), vec2(size - 1));
            map[j * 4 + i] = textureLoad(textures[index], t, 0).r;
        }
    }
    var lit = 0.0;
    for (var j = 0; j < 3; j++) {
        for (var i = 0; i < 3; i++) {
            let k = j * 4 + i;
            // Reverse depth: what the light meets first is the larger.
            let nearest = max(max(map[k], map[k + 1]), max(map[k + 4], map[k + 5]));
            lit += select(0.0, 1.0, nearest <= ndc.z + bias);
        }
    }
    lit /= 9.0;
    // Near its edge it hands over to the caller's guess, and so it does as the light nears the
    // horizon (long smeared shadows, then the swap from the sun to the moon).
    let low = smoothstep(0.02, 0.06, light_dir().y);
    return vec2(lit, (1.0 - smoothstep(0.85, 0.97, reach)) * low);
}

// How much of the sun reaches `rel` (camera relative) facing `n`: the near cascade's map (80 blocks
// about the player, fine) where it holds, the far one's (400 blocks, coarser) beyond, handing over
// smoothly; x 1 lit to 0 in shadow, y how far to trust it.
fn shadow_at(rel: vec3<f32>, n: vec3<f32>) -> vec2<f32> {
    let near = shadow_in(scene.shadow_view_proj, scene.shadow, 80.0, 400.0, rel, n);
    if near.y >= 0.999 {
        return near;
    }
    let far = shadow_in(scene.shadow_far_view_proj, scene.shadow_far, 400.0, 1200.0, rel, n);
    let trust = max(near.y, far.y);
    if trust <= 0.0 {
        return vec2(1.0, 0.0);
    }
    return vec2(mix(far.x, near.x, near.y / trust), trust);
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

// How much of the sun the clouds let through to `rel` (camera relative): 1 under open sky, about
// a fifth under a cloud's middle, soft at its edges. The point is followed toward the light to the
// slab's middle, where the four nearest cells are blended as the sky draws them, so the shadows
// are the clouds overhead, drifting with them across near and far land alike (Bliss's moving
// shade on the hills).
fn cloud_shadow(rel: vec3<f32>) -> f32 {
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
    let cover = mix(mix(a, b, u.x), mix(d, e, u.x), u.y);
    // Fading in as the light climbs: long low shadows from far clouds would smear across the land.
    let lift = smoothstep(0.03, 0.15, l.y);
    // The sky's puffs fill in over this range of the blend at their middle (`cloud_density`).
    return 1.0 - 0.8 * smoothstep(0.35, 0.6, cover) * lift;
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
    return max(l - vec3(d / 15.0), vec3(0.0)) * facing;
}

// Light arriving at a surface with normal `n`. `light` is the world's (sky, red, green, blue, 0..1),
// `shadow` 1 where the sun reaches it, `ao` the corner shading, `rel` where it is from the camera.
fn light_on(n: vec3<f32>, light: vec4<f32>, shadow: f32, ao: f32, pos: vec3<f32>, rel: vec3<f32>) -> vec3<f32> {
    let p = palette();
    // The sky's light falls off steeply under cover (Bliss's lightmap, (L^15 x 2 + L^2.5) / 3: 1 in the
    // open, 0.39 a step under, 0.21 two): a forest floor sits a few times below open shade, the open
    // stays as it was.
    let steep = look_opt(OPT_SHADE, 1.0);
    let sky = (pow(light.x, 15.0 * steep) * 2.0 + pow(light.x, 2.5 * steep)) / 3.0;
    let under = under_leaves(light.x);
    let leaves = mix(vec3(1.0), LEAF_LIGHT, under * 0.5);
    // More of the sky's light comes from above than from the sides, and less from below: tops 1,
    // sides 0.75, bottoms 0.5 (Complementary's), so shade still shows form.
    let hemi = 0.75 + 0.25 * n.y;
    var c = p.ambient * sky * hemi * ao * leaves;
    // Under a forest's canopy most of the floor's light is the sun's, come down through the leaves:
    // green and soft (a rainforest's floor gets 1-2 % of the sun, more of it this way than from the
    // sky). Without it the floor of Plants' rainforest sat at luma 12-17 at noon, under the daytime
    // targets' 15-60 for deep forest shade (Lighting's canopy check, 11-04).
    c += p.sun * light_strength() * 0.035 * LEAF_LIGHT * under * hemi * ao;
    // At golden hour the sky round the low sun glows: a warm fill from its side of the sky, on
    // the faces turned that way; those turned away keep the sky's cool light (Valheim's warm light,
    // blue shade).
    let l = light_dir();
    let side = normalize(vec3(l.x, 0.0, l.z) + vec3(1e-4, 0.0, 0.0));
    let toward = clamp(dot(n, side) * 0.5 + 0.5, 0.0, 1.0);
    c += p.glow * golden() * 0.35 * mix(0.3, 1.1, toward) * sky * hemi * ao * leaves;
    // Direct light only where the sky reaches (it keeps caves out of reach of the traced shadow's
    // gaps), and without corner shading: corners are dark where little sky gets in.
    // Under a forest's canopy the sun comes through the gaps the shadow map shows at nearly its
    // full strength (sunflecks), though the leaves cut the sky light there to 0.6-0.8.
    let c_open = min(canopy(), 1.0);
    let open = smoothstep(mix(0.5, 0.25, c_open), mix(0.95, 0.55, c_open), light.x);
    c += p.sun * light_strength() * sun_facing(n) * shadow * open * mix(0.75, 1.0, ao);
    // A held torch's light where it is brighter than the placed lights there (no doubling).
    c += block_light(max(light.yzw, held_light(rel, n)), pos) * mix(0.5, 1.0, ao);
    // A floor so caves are never quite black: the dimension's own and the Brightness setting.
    let floor_light = max(brightness_curve(scene.light.x) * 0.25, mix(0.006, 0.03, scene.light.y));
    c += vec3(floor_light) * vec3(0.85, 0.9, 1.0) * ao;
    return c;
}

// For what has no normal to speak of (particles, rain, the hand): half the direct light.
fn light_round(light: vec4<f32>, pos: vec3<f32>, rel: vec3<f32>) -> vec3<f32> {
    let p = palette();
    let sky = brightness_curve(light.x);
    var c = p.ambient * sky * 0.85;
    c += p.sun * light_strength() * 0.5 * smoothstep(0.5, 0.95, light.x);
    c += block_light(max(light.yzw, held_light(rel, vec3(0.0))), pos);
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
    let w = wind_now();
    let speed = length(w);
    let dir = select(vec2(1.0, 0.0), w / max(speed, 1e-4), speed > 0.01);
    let t = scene.params.w;
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

fn fog_density(p: vec3<f32>, floor: f32) -> f32 {
    let f = scene.weather.z;
    let s = storm();
    let h = p.y - floor;
    // In a forest the morning mist rises into the crowns (a rainforest's lies on its canopy at
    // dawn, 25-35 blocks up).
    let low = exp(-max(h - 3.0, 0.0) / mix(7.0 + 12.0 * min(canopy(), 1.0), 40.0, s));
    let high = exp(-max(h, 0.0) / 45.0);
    let drift = vec3(wind_now().x, 0.0, wind_now().y) * scene.params.w * mix(0.6, 2.0, s);
    let q = p - drift;
    let shape = noise3(q * vec3(1.0 / 38.0, 1.0 / 12.0, 1.0 / 38.0)) * 0.7 + noise3(q * vec3(1.0 / 15.0, 1.0 / 8.0, 1.0 / 15.0)) * 0.3;
    let eaten = noise3(q * vec3(1.0 / 6.0, 1.0 / 4.0, 1.0 / 6.0) + vec3(0.0, scene.params.w * 0.04, 0.0));
    let banks = smoothstep(0.45, 0.66, shape - 0.25 * eaten + 0.12);
    // Deep under it, in the ground, there is none.
    let open = smoothstep(-24.0, -10.0, h);
    // The sun thins the banks by day to a third (mornings and evenings keep them); a storm's
    // blowing snow or dust it leaves.
    let burn = mix(1.0, mix(0.35, 1.0, s), smoothstep(0.25, 0.65, scene.sun.y));
    return f * open * burn * (low * (mix(0.05, 0.7, s) + mix(1.7, 0.6, s) * banks) + 0.06 * high) * 0.11;
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

fn fog_light(dir: vec3<f32>) -> FogLight {
    let p = palette();
    let mu = dot(dir, light_dir());
    let g = 0.6;
    let hg = (1.0 - g * g) / pow(1.0 + g * g - 2.0 * g * mu, 1.5) / 12.566;
    // The sun's glow in the fog capped: uncapped it was over twice the sunlight looking toward a high
    // sun, and noon fog by a river or on snow went a milky white-out (the world playtester's shots,
    // frame luma 0.76, saturation 0.06). Valheim's daytime fog reads grey and soft.
    let phase = min(hg * 1.6, 0.9) + 0.3;
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
    let warm = 1.0 - smoothstep(0.1, 0.65, scene.sun.y);
    let sky_part = (p.ambient * 1.25 + p.horizon * 0.2) * mix(1.0, 0.5, warm);
    let sun_part = p.sun * mix(vec3(1.0), MIST_GOLD * 0.65, warm) * light_strength() * phase;
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
            return mix(1.0, lit, smoothstep(0.02, 0.06, light_dir().y));
        }
    }
    if scene.shadow_far.z > 0.5 {
        let c = scene.shadow_far_view_proj * vec4(rel, 1.0);
        let ndc = c.xyz / c.w;
        if abs(ndc.x) < 0.98 && abs(ndc.y) < 0.98 && ndc.z > 0.0 && ndc.z < 1.0 {
            let at = vec2<i32>(vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * scene.shadow_far.y);
            lit = select(0.0, 1.0, textureLoad(textures[u32(scene.shadow_far.x)], at, 0).r <= ndc.z + 0.0004);
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
    if scene.weather.z < 0.005 || in_liquid() || !open_sky() || sky <= 0.0 {
        return c;
    }
    let dist = min(length(rel), 220.0);
    let dir = rel / max(length(rel), 1e-4);
    let eye = scene.eye.xyz;
    let floor = fog_floor(rel);
    let light = fog_light(dir);
    let leafy = min(canopy(), 1.0);
    // In the canopy's shade the mist is lit by what comes through the leaves, warm green, more than
    // by the sky they hide: the sky's blue alone, tinted green, made it teal (the dawn check on
    // Plants' rainforest, 11-04).
    let shaded_sky = light.sky * mix(vec3(1.0), LEAF_LIGHT * 0.2, leafy * 0.85) + light.leaf * leafy;
    // Jittered along the view per pixel, so the shafts' edges don't band.
    let jitter = fract(sin(dot(rel, vec3(12.9898, 78.233, 37.719))) * 43758.547) - 0.5;
    let steps = 12u;
    var through = 1.0;
    var gained = vec3(0.0);
    var last = 0.0;
    for (var i = 1u; i <= steps; i++) {
        let k = f32(i) / f32(steps);
        let at = dist * (exp(2.2 * k) - 1.0) / (exp(2.2) - 1.0);
        let step = at - last;
        // Jittered by under half a step: at 0.8 of one the shafts showed a fixed grain.
        let mid = at - step * (0.5 + 0.35 * jitter);
        let d = fog_density(eye + dir * mid, floor) * sky;
        let kept = exp(-d * step);
        let lit = sun_reaches(dir * mid);
        gained += through * (1.0 - kept) * (mix(shaded_sky, light.sky, lit) + light.sun * lit);
        through *= kept;
        last = at;
    }
    return c * through + gained;
}

// How much snow lies on a surface with normal `n` at `pos` (world) with `sky` light: on what
// faces up under the open sky, in patches that join up as it deepens.
fn snow_on(n: vec3<f32>, pos: vec3<f32>, sky: f32) -> f32 {
    let deep = scene.ground.x;
    if deep <= 0.0 {
        return 0.0;
    }
    let q = vec3<i32>(floor(pos * 16.0));
    let spot = noise3(pos * 0.35) * 0.8 + hash3(q) * 0.2;
    return smoothstep(0.85, 1.0, sky) * smoothstep(0.55, 0.9, n.y) * smoothstep(1.0 - deep * 1.3, 1.05 - deep * 1.3, spot);
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

fn haze_lit(c_in: vec3<f32>, rel: vec3<f32>, sky: f32) -> vec3<f32> {
    let c = fog_through(c_in, rel, smoothstep(0.25, 0.75, sky));
    let dist = length(rel);
    if in_water() {
        return under_water(c, rel);
    }
    if in_liquid() || !open_sky() {
        return mix(c, scene.fog_color.rgb, fog_amount(dist, scene.params.x, scene.params.y));
    }
    let dir = rel / max(dist, 1e-4);
    let end = scene.params.y;
    // Straight down there is no level direction left: never a zero vector (its normalize is NaN,
    // and the Mac's fast math has no safety net).
    let air = vec3(dir.x, max(dir.y, 0.0) * 0.35 + 1e-3, dir.z);
    let color = sky_color(normalize(air));
    // Thinner looking up at what is high, so hills stand out against a hazy valley.
    let thin = exp(-max(rel.y, 0.0) / 90.0);
    let rain = rain_amount();
    // The air's own depth, whatever the view distance: with distant terrain the fog's end lies
    // thousands of blocks off, and the haze would barely start where the kept blocks give way to it.
    let reach = min(end, 2500.0);
    // A little hazier than it was (0.9): Bliss's air veils the middle distance more, lighter.
    // (Half as thick as it was: with it off a forest frame's darkest twentieth went 35 -> 25 and its
    // saturation 0.41 -> 0.57; the haze was the grey veil over the middle distance, Colour 9.)
    let amount = 1.0 - exp(-dist / (reach * mix(0.75, 0.4, rain)) * mix(0.45, 1.0, thin) * 0.5 * look_opt(OPT_HAZE, 1.0));
    let edge = smoothstep(end * 0.72, end, dist);
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
    // The far loses its colour before the haze covers it: far ridges go pale and grey-blue while the
    // near stays rich.
    let pale = mix(c, vec3(luma(c)), clamp(amount * 0.6, 0.0, 0.35));
    return mix(pale, lit, clamp(max(amount * 0.9, edge), 0.0, 1.0));
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

fn hejl(c: vec3<f32>, white: f32) -> vec3<f32> {
    let v = vec4(c, white);
    let a = 1.425 * v + vec4(0.05);
    let f = (v * a + vec4(0.004)) / (v * (a + vec4(0.55)) + vec4(0.0491)) - vec4(0.0821);
    return pow(max(f.rgb / f.w, vec3(0.0)), vec3(2.2));
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
    return mix(day, 4.0, smoothstep(0.0, -0.2, scene.sun.y));
}

fn display(c: vec3<f32>) -> vec3<f32> {
    // By day Timothy Lottes's curve (his published formula, GDC 2016; Complementary's settings:
    // contrast 1.05, mid-grey 0.25 in and out, peak 8) at exposure 1.3: black stays black (no toe
    // lift, which raised the black point and halved saturation: Colour 4-7) and the darks keep their
    // colour, where Hejl's toe crushed them. At night and under water Hejl's as before (nights
    // dark by design), blended as the sun comes up.
    let day = smoothstep(-0.05, 0.15, scene.sun.y) * select(1.0, 0.0, in_liquid());
    let night_t = hejl(max(c, vec3(0.0)) * exposure(), 3.0);
    let day_t = min(lottes(max(c, vec3(0.0)) * 1.3 * look_opt(OPT_EXPOSURE, 1.0)), vec3(1.0));
    var t = mix(night_t, day_t, day);
    // Only true darks go cool (most daylit ground is mid-tone and keeps its color).
    let lights = smoothstep(0.02, 0.22, luma(t));
    let shade = mix(vec3(0.975, 0.99, 1.03), vec3(0.92, 0.96, 1.10), golden());
    t *= mix(shade, mix(vec3(1.0), vec3(1.12, 1.0, 0.82), golden()), lights);
    // The near-black loses some colour: at night 60 % of it (a dim light's hue was pushed to a
    // saturated blue), by day a quarter below a tenth of luminance (Complementary's).
    let dim = luma(t);
    let keep = mix(0.4 + 0.6 * smoothstep(0.0, 0.05, dim), 0.75 + 0.25 * smoothstep(0.0, 0.1, dim), day);
    t = mix(vec3(dim), t, keep);
    // By day 1.2 of saturation after the curve (Colour 9; 1.07, Complementary's, read pale here).
    let y = luma(t);
    t = max(mix(vec3(y), t, mix(1.0, 1.2 * look_opt(OPT_SATURATION, 1.0), day)), vec3(0.0));
    // A cool floor so the night is never pure black; by day black stays black.
    t += (vec3(1.0) - t) * vec3(0.0018, 0.002, 0.0034) * (1.0 - day);
    return clamp(t, vec3(0.0), vec3(1.0));
}
