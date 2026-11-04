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

// The palette by how high the sun stands (`scene.sun.y`, about 0.94 at noon): day, golden hour,
// sunset, dusk and night, blended between.
fn palette() -> Palette {
    let e = scene.sun.y;
    let day = key(vec3(0.09, 0.26, 0.85), vec3(0.42, 0.60, 0.95), vec3(1.0, 0.92, 0.78), vec3(1.15, 0.98, 0.76), vec3(0.20, 0.27, 0.42));
    // Golden hour is orange, not white: a warm horizon, an orange glow and sun.
    let golden = key(vec3(0.11, 0.25, 0.78), vec3(0.92, 0.62, 0.42), vec3(1.0, 0.58, 0.26), vec3(1.55, 0.80, 0.38), vec3(0.22, 0.24, 0.38));
    let sunset = key(vec3(0.08, 0.10, 0.38), vec3(1.05, 0.46, 0.34), vec3(1.0, 0.50, 0.26), vec3(1.50, 0.76, 0.38), vec3(0.27, 0.22, 0.36));
    let dusk = key(vec3(0.045, 0.05, 0.16), vec3(0.30, 0.15, 0.27), vec3(0.45, 0.20, 0.25), vec3(0.0, 0.0, 0.0), vec3(0.10, 0.09, 0.17));
    // Night is dark and blue, a little lighter than vanilla's, so lanterns carry the scene.
    let night = key(vec3(0.006, 0.010, 0.030), vec3(0.025, 0.035, 0.070), vec3(0.10, 0.13, 0.22), vec3(0.025, 0.04, 0.10), vec3(0.010, 0.017, 0.050));
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
    p.ambient *= mix(1.0, mix(0.55, 1.0, moon_light()), smoothstep(-0.14, -0.3, e));
    // Overcast: grey and dim, and the sun no more than a brighter part of the sky.
    let r = rain_amount();
    let grey = vec3(luma(p.horizon)) * vec3(0.92, 0.95, 1.0);
    p.zenith = mix(p.zenith, grey * 0.75, r * 0.85);
    p.horizon = mix(p.horizon, grey, r * 0.85);
    p.glow = p.glow * (1.0 - r * 0.8);
    p.ambient = mix(p.ambient, vec3(luma(p.ambient + p.sun * 0.4)) * vec3(0.95, 0.97, 1.0), r * 0.85);
    p.sun = p.sun * (1.0 - r * 0.9);
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
    return select(-scene.sun.xyz, scene.sun.xyz, scene.sun.y > -0.06);
}

// How much of the palette's direct light there is: it fades out as the sun sets and the moon comes
// up a little after.
fn light_strength() -> f32 {
    let e = scene.sun.y;
    return select(smoothstep(0.06, 0.2, -e) * mix(0.2, 1.0, moon_light()), smoothstep(-0.04, 0.05, e), e > -0.06);
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
    let moon = pow(max(-mu, 0.0), 24.0) * smoothstep(-0.05, -0.25, scene.sun.y);
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
    return mix(guess, min(mapped.x, smoothstep(0.3, 0.8, sky)), mapped.y);
}

// ---- light on things ----

// Block light (red, green and blue levels 0..1) as color: warm firelight, whiter right at the
// source, with a slow flicker that runs across the world.
fn block_light(b: vec3<f32>, pos: vec3<f32>) -> vec3<f32> {
    let m = max(max(b.x, b.y), b.z);
    if m <= 0.0 {
        return vec3(0.0);
    }
    let curve = vec3(brightness_curve(b.x), brightness_curve(b.y), brightness_curve(b.z));
    // White light (torches, lanterns) is warmed; colored light keeps its color.
    let grey = 1.0 - (m - min(min(b.x, b.y), b.z)) / m;
    let warm = mix(vec3(1.0), mix(vec3(1.0, 0.56, 0.26), vec3(1.0, 0.82, 0.62), m * m), grey);
    let t = scene.params.w;
    let q = floor(pos * 0.5);
    let flicker = 1.0 + 0.045 * sin(t * 7.3 + q.x * 1.7 + q.z * 2.3) + 0.03 * sin(t * 13.1 + q.y * 3.1);
    return curve * warm * 1.55 * flicker;
}

// How much a surface with normal `n` faces the sun. With the sun low its light wraps a little
// over the tops, which are otherwise only grazed: the warm ground of the golden hour.
fn sun_facing(n: vec3<f32>) -> f32 {
    let wrap = 0.45 * (1.0 - smoothstep(0.05, 0.3, abs(scene.sun.y)));
    return max((dot(n, light_dir()) + wrap) / (1.0 + wrap), 0.0);
}

// Light arriving at a surface with normal `n`. `light` is the world's (sky, red, green, blue, 0..1),
// `shadow` 1 where the sun reaches it, `ao` the corner shading.
fn light_on(n: vec3<f32>, light: vec4<f32>, shadow: f32, ao: f32, pos: vec3<f32>) -> vec3<f32> {
    let p = palette();
    let sky = brightness_curve(light.x);
    // More of the sky's light comes from above than from the sides, and little from below.
    let hemi = 0.62 + 0.38 * n.y;
    var c = p.ambient * sky * hemi * ao;
    // At golden hour the sky round the low sun glows: a warm fill on everything under it, in the
    // shade too, so the hour reaches the ground and not only the sky.
    c += p.glow * golden() * 0.35 * sky * hemi * ao;
    // Direct light only where the sky reaches (it keeps caves out of reach of the traced shadow's
    // gaps), and without corner shading: corners are dark where little sky gets in.
    let open = smoothstep(0.5, 0.95, light.x);
    c += p.sun * light_strength() * sun_facing(n) * shadow * open * mix(0.75, 1.0, ao);
    c += block_light(light.yzw, pos) * mix(0.5, 1.0, ao);
    // A floor so caves are never quite black: the dimension's own and the Brightness setting.
    let floor_light = max(brightness_curve(scene.light.x) * 0.25, mix(0.006, 0.03, scene.light.y));
    c += vec3(floor_light) * vec3(0.85, 0.9, 1.0) * ao;
    return c;
}

// For what has no normal to speak of (particles, rain, the hand): half the direct light.
fn light_round(light: vec4<f32>, pos: vec3<f32>) -> vec3<f32> {
    let p = palette();
    let sky = brightness_curve(light.x);
    var c = p.ambient * sky * 0.85;
    c += p.sun * light_strength() * 0.5 * smoothstep(0.5, 0.95, light.x);
    c += block_light(light.yzw, pos);
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

// Where the fog lies: on the ground about the player (a few blocks under the eye), or on the sea
// if that is higher. Valleys under it fill, hills stand out of it.
fn fog_floor() -> f32 {
    return max(SEA_LEVEL, scene.eye.y - 5.0);
}

// How foggy the air is at `p` (world): a layer lying on the ground and in the valleys, broken into
// banks that drift with the wind and thin out higher up. 0 in clear weather. Per block.
// A storm's share of the fog (blowing snow or dust: `scene.ground` y and z): it fills the air
// high up and races with the wind instead of lying low.
fn storm() -> f32 {
    return clamp(max(scene.ground.y * smoothstep(5.0, 9.0, length(wind_now())), scene.ground.z), 0.0, 1.0);
}

fn fog_density(p: vec3<f32>) -> f32 {
    let f = scene.weather.z;
    let s = storm();
    let h = p.y - fog_floor();
    let low = exp(-max(h - 3.0, 0.0) / mix(7.0, 40.0, s));
    let high = exp(-max(h, 0.0) / 45.0);
    let drift = vec3(wind_now().x, 0.0, wind_now().y) * scene.params.w * mix(0.6, 2.0, s);
    let q = p - drift;
    let shape = noise3(q * vec3(1.0 / 38.0, 1.0 / 12.0, 1.0 / 38.0)) * 0.7 + noise3(q * vec3(1.0 / 15.0, 1.0 / 8.0, 1.0 / 15.0)) * 0.3;
    let eaten = noise3(q * vec3(1.0 / 6.0, 1.0 / 4.0, 1.0 / 6.0) + vec3(0.0, scene.params.w * 0.04, 0.0));
    let banks = smoothstep(0.45, 0.66, shape - 0.25 * eaten + 0.12);
    // Deep under it, in the ground, there is none.
    let open = smoothstep(-24.0, -10.0, h);
    return f * open * (low * (mix(0.05, 0.7, s) + mix(1.7, 0.6, s) * banks) + 0.06 * high) * 0.11;
}

// Light the fog sends toward the eye: the sky's from all around, the sun's mostly forward, so it
// glows white-gold looking toward the sun.
fn fog_light(dir: vec3<f32>) -> vec3<f32> {
    let p = palette();
    let mu = dot(dir, light_dir());
    let g = 0.6;
    let hg = (1.0 - g * g) / pow(1.0 + g * g - 2.0 * g * mu, 1.5) / 12.566;
    let phase = hg * 2.5 + 0.3;
    // Droplets are white, snow whiter, dust the color of sand.
    let tint = mix(mix(vec3(0.9), vec3(1.0, 1.0, 1.03), scene.ground.y), vec3(0.95, 0.68, 0.42), scene.ground.z);
    return ((p.ambient * 1.25 + p.horizon * 0.2) + p.sun * light_strength() * phase) * 0.9 * tint;
}

// The fog between the eye and a point `rel` away (camera relative), walked in a few steps that are
// closer together near the eye; `sky` (0..1) the sky light at the point, so caves stay clear.
fn fog_through(c: vec3<f32>, rel: vec3<f32>, sky: f32) -> vec3<f32> {
    if scene.weather.z < 0.005 || in_liquid() || !open_sky() || sky <= 0.0 {
        return c;
    }
    let dist = min(length(rel), 220.0);
    let dir = rel / max(length(rel), 1e-4);
    let eye = scene.eye.xyz;
    let light = fog_light(dir);
    let steps = 12u;
    var through = 1.0;
    var gained = vec3(0.0);
    var last = 0.0;
    for (var i = 1u; i <= steps; i++) {
        let k = f32(i) / f32(steps);
        let at = dist * (exp(2.2 * k) - 1.0) / (exp(2.2) - 1.0);
        let step = at - last;
        let d = fog_density(eye + dir * (at - step * 0.5)) * sky;
        let kept = exp(-d * step);
        gained += through * (1.0 - kept) * light;
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
    let amount = 1.0 - exp(-dist / (reach * mix(0.9, 0.4, rain)) * mix(0.45, 1.0, thin));
    let edge = smoothstep(end * 0.72, end, dist);
    return mix(c, color, clamp(max(amount * 0.9, edge), 0.0, 1.0));
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
    return mix(1.7, 4.0, smoothstep(0.0, -0.2, scene.sun.y));
}

fn display(c: vec3<f32>) -> vec3<f32> {
    var t = hejl(max(c, vec3(0.0)) * exposure(), 3.0);
    // Only true darks go cool (most daylit ground is mid-tone and keeps its color).
    let lights = smoothstep(0.02, 0.22, luma(t));
    t *= mix(vec3(0.975, 0.99, 1.03), mix(vec3(1.0), vec3(1.12, 1.0, 0.82), golden()), lights);
    // Colour held back in the mid-tones and darks (the land), not in what glows bright (lava, fire,
    // lamps keep their fire).
    let y = luma(t);
    t = max(mix(vec3(y), t, mix(1.08, 1.25, smoothstep(0.05, 0.35, y))), vec3(0.0));
    t += (vec3(1.0) - t) * vec3(0.0018, 0.002, 0.0034);
    return clamp(t, vec3(0.0), vec3(1.0));
}
