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

// Rain, read from the fog: rain turns it grey (`sky::rained`), which takes the blue out of it.
// Clear daylight fog is about 0.25 more blue than red, full rain 0.16. The sunrise and sunset warm
// the fog toward where the camera looks, so it is only read while the sun is up.
fn rain_amount() -> f32 {
    let f = scene.fog_color.rgb;
    let blue = (f.b - f.r) / max(f.b, 1e-4);
    return clamp((0.245 - blue) / 0.08, 0.0, 1.0) * smoothstep(0.12, 0.3, scene.sun.y);
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3(0.2126, 0.7152, 0.0722));
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

// The palette by how high the sun stands (`scene.sun.y`, about 0.94 at noon): day, golden hour,
// sunset, dusk and night, blended between.
fn palette() -> Palette {
    let e = scene.sun.y;
    let day = key(vec3(0.13, 0.30, 0.80), vec3(0.52, 0.68, 0.95), vec3(1.0, 0.92, 0.78), vec3(1.25, 1.12, 0.96), vec3(0.30, 0.38, 0.56));
    let golden = key(vec3(0.16, 0.26, 0.70), vec3(0.98, 0.70, 0.52), vec3(1.0, 0.72, 0.42), vec3(1.30, 0.92, 0.58), vec3(0.32, 0.32, 0.50));
    let sunset = key(vec3(0.17, 0.15, 0.46), vec3(1.05, 0.46, 0.34), vec3(1.0, 0.50, 0.26), vec3(1.10, 0.52, 0.28), vec3(0.30, 0.22, 0.38));
    let dusk = key(vec3(0.045, 0.05, 0.16), vec3(0.30, 0.15, 0.27), vec3(0.45, 0.20, 0.25), vec3(0.0, 0.0, 0.0), vec3(0.11, 0.10, 0.20));
    let night = key(vec3(0.006, 0.010, 0.030), vec3(0.025, 0.035, 0.070), vec3(0.10, 0.13, 0.22), vec3(0.10, 0.14, 0.24), vec3(0.045, 0.060, 0.11));
    var p: Palette;
    if e > 0.15 {
        p = blend(golden, day, smoothstep(0.15, 0.5, e));
    } else if e > 0.02 {
        p = blend(sunset, golden, smoothstep(0.02, 0.15, e));
    } else if e > -0.12 {
        p = blend(dusk, sunset, smoothstep(-0.12, 0.02, e));
    } else {
        p = blend(night, dusk, smoothstep(-0.3, -0.12, e));
    }
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

// Where direct light comes from: the sun while it is up, then the moon opposite it.
fn light_dir() -> vec3<f32> {
    return select(-scene.sun.xyz, scene.sun.xyz, scene.sun.y > -0.06);
}

// How much of the palette's direct light there is: it fades out as the sun sets and the moon comes
// up a little after.
fn light_strength() -> f32 {
    let e = scene.sun.y;
    return select(smoothstep(0.06, 0.2, -e), smoothstep(-0.04, 0.05, e), e > -0.06);
}

// ---- the sky ----

fn sun_glow(dir: vec3<f32>, p: Palette) -> vec3<f32> {
    let mu = max(dot(dir, scene.sun.xyz), 0.0);
    let horizon = 1.0 - clamp(dir.y, 0.0, 1.0);
    let wide = pow(mu, 5.0) * (0.28 + 0.4 * horizon);
    let near = pow(mu, 60.0) * 0.9;
    return p.glow * (wide + near);
}

// The sky in `dir` (unit, world): horizon to zenith, the warm side toward the sun at sunset and the
// purple opposite, the glow round the sun, a faint one round the moon.
fn sky_color(dir: vec3<f32>) -> vec3<f32> {
    let p = palette();
    let h = clamp(dir.y, 0.0, 1.0);
    var c = mix(p.horizon, p.zenith, pow(h, 0.48));
    let mu = dot(dir, scene.sun.xyz);
    let low = 1.0 - smoothstep(0.0, 0.35, abs(scene.sun.y));
    // Away from the sun the sunset sky goes rose and violet.
    let away = (0.5 - 0.5 * mu) * (1.0 - h) * low;
    c = mix(c, c * vec3(0.95, 0.72, 1.12) + vec3(0.04, 0.0, 0.05), away * 0.7);
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

// Light arriving at a surface with normal `n`. `light` is the world's (sky, red, green, blue, 0..1),
// `shadow` 1 where the sun reaches it, `ao` the corner shading.
fn light_on(n: vec3<f32>, light: vec4<f32>, shadow: f32, ao: f32, pos: vec3<f32>) -> vec3<f32> {
    let p = palette();
    let sky = brightness_curve(light.x);
    // More of the sky's light comes from above than from the sides, and little from below.
    let hemi = 0.62 + 0.38 * n.y;
    var c = p.ambient * sky * hemi * ao;
    // Direct light only where the sky reaches (it keeps caves out of reach of the traced shadow's
    // gaps), and without corner shading: corners are dark where little sky gets in.
    let open = smoothstep(0.5, 0.95, light.x);
    let ndl = max(dot(n, light_dir()), 0.0);
    c += p.sun * light_strength() * ndl * shadow * open * mix(0.75, 1.0, ao);
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

// ---- haze and fog ----

// The air between the eye and a point `rel` away (camera relative): a haze the color of the sky
// behind it that thickens with distance and thins with height, full where the terrain ends. In a
// liquid or a dimension without sky, the engine's fog.
fn haze(c: vec3<f32>, rel: vec3<f32>) -> vec3<f32> {
    let dist = length(rel);
    if in_liquid() || !open_sky() {
        return mix(c, scene.fog_color.rgb, fog_amount(dist, scene.params.x, scene.params.y));
    }
    let dir = rel / max(dist, 1e-4);
    let end = scene.params.y;
    let air = vec3(dir.x, max(dir.y, 0.0) * 0.35, dir.z);
    let color = sky_color(normalize(air));
    // Thinner looking up at what is high, so hills stand out against a hazy valley.
    let thin = exp(-max(rel.y, 0.0) / 90.0);
    let rain = rain_amount();
    let amount = 1.0 - exp(-dist / (end * mix(0.9, 0.4, rain)) * mix(0.45, 1.0, thin));
    let edge = smoothstep(end * 0.72, end, dist);
    return mix(c, color, clamp(max(amount * 0.9, edge), 0.0, 1.0));
}

// ---- to the screen ----

// Filmic tone curve (Narkowicz's fit of ACES) on exposed light, a little more color, and a warm
// lift in the shadows: linear light for the sRGB screen.
fn display(c: vec3<f32>) -> vec3<f32> {
    let x = max(c, vec3(0.0)) * 1.05;
    var t = clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), vec3(0.0), vec3(1.0));
    let y = luma(t);
    t = max(mix(vec3(y), t, 1.14), vec3(0.0));
    t = t + vec3(0.010, 0.004, 0.0) * (1.0 - smoothstep(0.0, 0.2, y));
    return clamp(t, vec3(0.0), vec3(1.0));
}
