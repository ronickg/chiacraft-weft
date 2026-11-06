// Shared by the engine's shaders: put in front of each one (shader packs' too).

// The uv of each face corner (`CORNERS`, from the prelude), the two triangles of a face, and
// vanilla's face shading per direction (-X +X -Y +Y -Z +Z).
var<private> UVS: array<vec2<f32>, 4> = array<vec2<f32>, 4>(vec2(0.0, 1.0), vec2(1.0, 1.0), vec2(1.0, 0.0), vec2(0.0, 0.0));
var<private> TRI: array<u32, 6> = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u);
var<private> SHADE: array<f32, 6> = array<f32, 6>(0.6, 0.6, 0.5, 1.0, 0.8, 0.8);

// How far into the fog a point at `dist` is, 0..1.
fn fog_amount(dist: f32, start: f32, end: f32) -> f32 {
    if dist <= start {
        return 0.0;
    }
    return clamp((dist - start) / max(end - start, 1.0), 0.0, 1.0);
}

fn brightness_curve(level: f32) -> f32 {
    return level / (4.0 - 3.0 * level);
}

// Light as the world stores it, a byte each for sky, red, green, blue (0..15), to 0..1.
fn unpack_light(l: u32) -> vec4<f32> {
    let v = vec4<f32>(f32(l & 255u), f32((l >> 8u) & 255u), f32((l >> 16u) & 255u), f32(l >> 24u));
    return min(v, vec4(15.0)) / 15.0;
}

// Vanilla 26.1's lightmap: dim ambient, sky light that turns blue and weaker at night, warm block
// light that whitens towards both ends, then the Brightness setting bends it up keeping the hue.
// `day` daylight 0..1, `ambient` the dimension's floor 0..1, `bright` the Brightness setting.
fn lightmap(light: vec4<f32>, day: f32, ambient: f32, bright: f32) -> vec3<f32> {
    let d = clamp((day - 0.12) / 0.88, 0.0, 1.0);
    let sky = brightness_curve(light.x) * mix(0.24, 1.0, d);
    var c = max(vec3(0.04), vec3(brightness_curve(ambient))) + mix(vec3(0.478, 0.478, 1.0), vec3(1.0), d) * sky;
    let block = light.yzw;
    if any(block > vec3(0.0)) {
        // Vanilla's warmth of block light: yellower the dimmer it is, white at full, by the
        // level of its brightest channel so a colored lamp only grows brighter toward it.
        let level = max(max(block.x, block.y), block.z);
        let tint = vec3(1.0, (level * 0.6 + 0.4) * 0.6 + 0.4, level * level * 0.6 + 0.4);
        // Added to the sky's, each channel on its own: a lamp only ever brightens, and a
        // colored one shows by day too.
        c += min(tint * vec3(brightness_curve(block.x), brightness_curve(block.y), brightness_curve(block.z)) * 1.4, vec3(0.85));
    }
    // Light adds up past what a screen shows: what is bright bends toward full instead of
    // being cut there, so sun and lamp together are brighter than the sun alone.
    let knee = 0.8;
    c = select(c, knee + (1.0 - knee) * (1.0 - exp(-(c - knee) / (1.0 - knee))), c > vec3(knee));
    c = clamp(c, vec3(0.0), vec3(1.0));
    let m = max(max(c.x, c.y), c.z);
    if m > 1e-4 {
        let one_minus_m = 1.0 - m;
        let m2 = one_minus_m * one_minus_m;
        let m4 = m2 * m2;
        let lifted = c * ((1.0 - m4) / m);
        return mix(c, lifted, bright);
    }
    return c;
}

// What snippets get (`shaders.rs`), here so a shader pack replacing an engine shader keeps them.

// What a block shader (`shaders/block/<name>.wgsl`, `fn shade(p: BlockPixel) -> vec4<f32>`) gets;
// it returns the lit color before fog.
struct BlockPixel {
    // 0..1 across the face.
    uv: vec2<f32>,
    // The texel (with its overlay) and the light on it (shading, AO, tint, glow), both linear.
    albedo: vec4<f32>,
    light: vec3<f32>,
    // Block coordinates of the pixel (in a moving structure, the structure's own).
    pos: vec3<f32>,
    time: f32,
}

// What an entity shader (`fn shade(p: EntityPixel) -> vec4<f32>`) gets; it returns the lit
// color, alpha below 0.5 cuts the pixel out.
struct EntityPixel {
    uv: vec2<f32>,
    // Skin texel and the light on it (shading, hurt flash and glow included), linear.
    albedo: vec4<f32>,
    light: vec3<f32>,
    time: f32,
}

// What a text effect (`shaders/fx/<name>.wgsl`, `fn effect(g: Glyph) -> Glyph`) may change.
struct Glyph {
    // Added to this corner of the glyph, physical pixels.
    offset: vec2<f32>,
    // Linear color and alpha.
    color: vec4<f32>,
    time: f32,
    // Number of the glyph in its text, for waves running along it.
    index: f32,
    // Which corner, 0..1 in each axis.
    corner: vec2<f32>,
    // Physical pixels per GUI pixel.
    scale: f32,
    // The dark copy drawn under the text.
    shadow: bool,
}

// A themed panel filled by a shader (`shaders/gui/<name>.wgsl`, `fn fill(p: GuiPixel) -> vec4<f32>`).
struct GuiPixel {
    // 0..1 across the panel.
    uv: vec2<f32>,
    // Panel size in physical pixels.
    size: vec2<f32>,
    // The theme color, linear.
    color: vec4<f32>,
    time: f32,
}

// What a screen effect (`shaders/post/<name>.wgsl`, `fn apply(p: PostPixel) -> vec4<f32>`) gets.
struct PostPixel {
    // 0..1 across the screen, top left first.
    uv: vec2<f32>,
    // The image so far at `uv`, linear.
    color: vec4<f32>,
    // 0..1 as the mod set it, for fading in and out.
    strength: f32,
    time: f32,
    // Pixels.
    size: vec2<f32>,
    scene: u32,
    // The world's depth image (`post_depth`, `post_position`).
    depth: u32,
    // Bindless index + 1 of the effect's own half-size image (`post_low_at`, `post_low_upsampled`),
    // drawn before the effects by its `shaders/post_low/<name>.wgsl` (`fn low(p: PostPixel) ->
    // vec4<f32>`); 0 when it has none.
    low: u32,
}

// What a sky shader (`shaders/sky/<name>.wgsl`, `fn shade(p: SkyPixel) -> vec4<f32>`) gets; it
// returns the color and how much it covers, and the opacity is applied after it.
struct SkyPixel {
    // Unit direction of the pixel in the world, and in the object's own frame, which turns with
    // its orbit (x points at a body).
    dir: vec3<f32>,
    local: vec3<f32>,
    // 0..1 across the quad, or across the picture of a dome.
    uv: vec2<f32>,
    // The picture's sample (white without one) and the object's color, both linear.
    albedo: vec4<f32>,
    color: vec4<f32>,
    daylight: f32,
    time: f32,
}

// What a particle shader (`shaders/particle/<name>.wgsl`, `fn shade(p: ParticleFrag) -> vec4<f32>`)
// gets; it returns the color before lighting and fog.
struct ParticleFrag {
    // 0..1 across the quad.
    uv: vec2<f32>,
    // The texture sample and the kind's color at this age, both linear.
    albedo: vec4<f32>,
    color: vec4<f32>,
    // 0 at birth, 1 at the end of its life.
    life: f32,
    time: f32,
}

// ---- upscaling (`core:upscaler`, `scene.upscale.x`) ----

// AMD FidelityFX Super Resolution 1's edge-adaptive upscale (EASU), its 32-bit path ported from
// `ffx_fsr1.h` (GPUOpen-Effects/FidelityFX-FSR, MIT licence: Copyright (c) 2021 Advanced Micro
// Devices, Inc.; notes/ENGINE_CHANGES.md). The world image `textures[t]` (an index: a texture
// passed as an argument broke the Vulkan pipelines), drawn smaller than the screen
// (`core:render_scale`), read at the screen's `uv`: twelve texels round it, a Lanczos-like kernel
// stretched along the edge it finds. The light is filtered as x / (1 + x), as the HDR world's goes
// past 1 and FSR 1 takes a picture in 0..1.
fn easu_in(t: u32, p: vec2<i32>, size: vec2<i32>) -> vec3<f32> {
    let c = max(textureLoad(textures[t], clamp(p, vec2(0), size - 1), 0).rgb, vec3(0.0));
    return c / (1.0 + c);
}

fn easu_luma(c: vec3<f32>) -> f32 {
    return c.b * 0.5 + (c.r * 0.5 + c.g);
}

// One of the four bilinear corners' direction and length (`FsrEasuSetF`): `w` its bilinear weight,
// a, b, c, d, e the '+' round it (c in the middle).
fn easu_set(dir: ptr<function, vec2<f32>>, len: ptr<function, f32>, w: f32, la: f32, lb: f32, lc: f32, ld: f32, le: f32) {
    let dc = ld - lc;
    let cb = lc - lb;
    var len_x = 1.0 / max(max(abs(dc), abs(cb)), 1e-8);
    let dir_x = ld - lb;
    len_x = clamp(abs(dir_x) * len_x, 0.0, 1.0);
    len_x *= len_x;
    let ec = le - lc;
    let ca = lc - la;
    var len_y = 1.0 / max(max(abs(ec), abs(ca)), 1e-8);
    let dir_y = le - la;
    len_y = clamp(abs(dir_y) * len_y, 0.0, 1.0);
    len_y *= len_y;
    *dir += vec2(dir_x, dir_y) * w;
    *len += (len_x + len_y) * w;
}

// One tap of the kernel (`FsrEasuTapF`).
fn easu_tap(acc: ptr<function, vec4<f32>>, off: vec2<f32>, dir: vec2<f32>, len: vec2<f32>, lob: f32, clp: f32, c: vec3<f32>) {
    var v = vec2(off.x * dir.x + off.y * dir.y, off.x * -dir.y + off.y * dir.x);
    v *= len;
    let d2 = min(dot(v, v), clp);
    var wb = 2.0 / 5.0 * d2 - 1.0;
    var wa = lob * d2 - 1.0;
    wb *= wb;
    wa *= wa;
    wb = 25.0 / 16.0 * wb - (25.0 / 16.0 - 1.0);
    let w = wb * wa;
    *acc += vec4(c * w, w);
}

fn easu(t: u32, uv: vec2<f32>) -> vec3<f32> {
    let size = vec2<i32>(textureDimensions(textures[t]));
    var pp = uv * vec2<f32>(size) - 0.5;
    let fp = floor(pp);
    pp -= fp;
    let f0 = vec2<i32>(fp);
    //    b c
    //  e f g h
    //  i j k l
    //    n o
    let b = easu_in(t, f0 + vec2(0, -1), size);
    let c = easu_in(t, f0 + vec2(1, -1), size);
    let e = easu_in(t, f0 + vec2(-1, 0), size);
    let f = easu_in(t, f0, size);
    let g = easu_in(t, f0 + vec2(1, 0), size);
    let h = easu_in(t, f0 + vec2(2, 0), size);
    let i = easu_in(t, f0 + vec2(-1, 1), size);
    let j = easu_in(t, f0 + vec2(0, 1), size);
    let k = easu_in(t, f0 + vec2(1, 1), size);
    let l = easu_in(t, f0 + vec2(2, 1), size);
    let n = easu_in(t, f0 + vec2(0, 2), size);
    let o = easu_in(t, f0 + vec2(1, 2), size);
    let bl = easu_luma(b);
    let cl = easu_luma(c);
    let el = easu_luma(e);
    let fl = easu_luma(f);
    let gl = easu_luma(g);
    let hl = easu_luma(h);
    let il = easu_luma(i);
    let jl = easu_luma(j);
    let kl = easu_luma(k);
    let ll = easu_luma(l);
    let nl = easu_luma(n);
    let ol = easu_luma(o);
    var dir = vec2(0.0);
    var len = 0.0;
    easu_set(&dir, &len, (1.0 - pp.x) * (1.0 - pp.y), bl, el, fl, gl, jl);
    easu_set(&dir, &len, pp.x * (1.0 - pp.y), cl, fl, gl, hl, kl);
    easu_set(&dir, &len, (1.0 - pp.x) * pp.y, fl, il, jl, kl, nl);
    easu_set(&dir, &len, pp.x * pp.y, gl, jl, kl, ll, ol);
    let dir2 = dir * dir;
    var dir_r = dir2.x + dir2.y;
    let zero = dir_r < 1.0 / 32768.0;
    dir_r = select(inverseSqrt(max(dir_r, 1e-12)), 1.0, zero);
    dir.x = select(dir.x, 1.0, zero);
    dir *= dir_r;
    len = len * 0.5;
    len *= len;
    let stretch = dot(dir, dir) / max(abs(dir.x), abs(dir.y));
    let len2 = vec2(1.0 + (stretch - 1.0) * len, 1.0 - 0.5 * len);
    let lob = 0.5 + ((1.0 / 4.0 - 0.04) - 0.5) * len;
    let clp = 1.0 / lob;
    let mn = min(min(f, g), min(j, k));
    let mx = max(max(f, g), max(j, k));
    var acc = vec4(0.0);
    easu_tap(&acc, vec2(0.0, -1.0) - pp, dir, len2, lob, clp, b);
    easu_tap(&acc, vec2(1.0, -1.0) - pp, dir, len2, lob, clp, c);
    easu_tap(&acc, vec2(-1.0, 1.0) - pp, dir, len2, lob, clp, i);
    easu_tap(&acc, vec2(0.0, 1.0) - pp, dir, len2, lob, clp, j);
    easu_tap(&acc, vec2(0.0, 0.0) - pp, dir, len2, lob, clp, f);
    easu_tap(&acc, vec2(-1.0, 0.0) - pp, dir, len2, lob, clp, e);
    easu_tap(&acc, vec2(1.0, 1.0) - pp, dir, len2, lob, clp, k);
    easu_tap(&acc, vec2(2.0, 1.0) - pp, dir, len2, lob, clp, l);
    easu_tap(&acc, vec2(2.0, 0.0) - pp, dir, len2, lob, clp, h);
    easu_tap(&acc, vec2(1.0, 0.0) - pp, dir, len2, lob, clp, g);
    easu_tap(&acc, vec2(1.0, 2.0) - pp, dir, len2, lob, clp, o);
    easu_tap(&acc, vec2(0.0, 2.0) - pp, dir, len2, lob, clp, n);
    let s = min(mx, max(mn, acc.rgb / acc.w));
    return s / max(1.0 - s, vec3(1e-4));
}

// Snapdragon Game Super Resolution 1 (`core:upscaler` 2), its mobile shader ported from
// `sgsr1_shader_mobile.frag` (SnapdragonStudios/snapdragon-gsr, BSD-3-Clause: Copyright (c) 2023-2025,
// Qualcomm Innovation Center, Inc. All rights reserved. Redistribution and use in source and binary
// forms, with or without modification, are permitted provided that the conditions of the BSD-3-Clause
// licence are met; notes/ENGINE_CHANGES.md). Cheaper than EASU on a phone (on the Pixel 10 at 67 %,
// 1.1 ms over bilinear, EASU 2.7): one channel (green) in four gathers, and only where the
// 2 x 2 round the pixel has an edge; a Lanczos-like kernel there, sharpened, added to the bilinear
// colour. As in `easu`, the light as x / (1 + x); and the change taken as a ratio of green's, not
// the same step on every channel, which paled dark colours along edges.
fn gsr_in(v: vec4<f32>) -> vec4<f32> {
    let m = max(v, vec4(0.0));
    return m / (1.0 + m);
}

fn gsr_weight(dx: f32, dy: f32, c: f32, spread: f32) -> vec2<f32> {
    let x = (dx * dx + dy * dy) * 0.55 + clamp(abs(c) * spread, 0.0, 1.0);
    var wa = x - 4.0;
    let wb = x * wa - wa;
    wa *= wa;
    let w = wb * wa;
    return vec2(w, w * c);
}

fn gsr(t: u32, uv: vec2<f32>, bilinear: vec3<f32>) -> vec3<f32> {
    let size = vec2<f32>(textureDimensions(textures[t]));
    let inv = 1.0 / size;
    let b = max(bilinear, vec3(0.0));
    var cc = b / (1.0 + b);
    let img = uv * size + vec2(-0.5, 0.5);
    let ip = floor(img);
    var coord = ip * inv;
    let pl = img - ip;
    var left = gsr_in(textureGather(1, textures[t], linear_clamp, coord));
    let edge = abs(left.z - left.y) + abs(cc.g - left.y) + abs(cc.g - left.z);
    if edge <= 8.0 / 255.0 {
        return bilinear;
    }
    coord.x += inv.x;
    var right = gsr_in(textureGather(1, textures[t], linear_clamp, coord + vec2(inv.x, 0.0)));
    let up = gsr_in(textureGather(1, textures[t], linear_clamp, coord + vec2(0.0, -inv.y)));
    let down = gsr_in(textureGather(1, textures[t], linear_clamp, coord + vec2(0.0, inv.y)));
    let mean = (left.y + left.z + right.x + right.w) * 0.25;
    left -= vec4(mean);
    right -= vec4(mean);
    let ud = vec4(up.w, up.z, down.y, down.x) - vec4(mean);
    let cw = cc.g - mean;
    let sum = dot(abs(left), vec4(1.0)) + dot(abs(right), vec4(1.0)) + dot(abs(ud), vec4(1.0));
    let spread = 2.181818 / max(sum, 1e-6);
    var a = gsr_weight(pl.x, pl.y + 1.0, ud.x, spread);
    a += gsr_weight(pl.x - 1.0, pl.y + 1.0, ud.y, spread);
    a += gsr_weight(pl.x - 1.0, pl.y - 2.0, ud.z, spread);
    a += gsr_weight(pl.x, pl.y - 2.0, ud.w, spread);
    a += gsr_weight(pl.x + 1.0, pl.y - 1.0, left.x, spread);
    a += gsr_weight(pl.x, pl.y - 1.0, left.y, spread);
    a += gsr_weight(pl.x, pl.y, left.z, spread);
    a += gsr_weight(pl.x + 1.0, pl.y, left.w, spread);
    a += gsr_weight(pl.x - 1.0, pl.y - 1.0, right.x, spread);
    a += gsr_weight(pl.x - 2.0, pl.y - 1.0, right.y, spread);
    a += gsr_weight(pl.x - 2.0, pl.y, right.z, spread);
    a += gsr_weight(pl.x - 1.0, pl.y, right.w, spread);
    let mx = max(max(left.y, left.z), max(right.x, right.w));
    let mn = min(min(left.y, left.z), min(right.x, right.w));
    let fy = clamp(2.0 * a.y / a.x, mn, mx);
    let d = clamp(fy - cw, -23.0 / 255.0, 23.0 / 255.0);
    cc = clamp(cc * ((cc.g + d) / max(cc.g, 1e-4)), vec3(0.0), vec3(0.9999));
    return cc / (1.0 - cc);
}

// The world's image at the screen's `uv` (`frag` the fragment's position, in pixels): bilinear, or
// when it is smaller than the screen FSR 1's EASU (`core:upscaler` 1) or Snapdragon GSR 1 (2).
fn world_upscaled(t: u32, uv: vec2<f32>, frag: vec2<f32>, upscaler: f32) -> vec4<f32> {
    let bilinear = textureSampleLevel(textures[t], linear_clamp, uv, 0.0);
    if upscaler < 0.5 {
        return bilinear;
    }
    let screen = frag / max(uv, vec2(1e-6));
    if f32(textureDimensions(textures[t]).x) >= screen.x - 0.5 {
        return bilinear;
    }
    if upscaler >= 1.5 {
        return vec4(gsr(t, uv, bilinear.rgb), bilinear.a);
    }
    return vec4(easu(t, uv), bilinear.a);
}

// ---- temporal anti-aliasing's motion (`post::Post`, `weft_render::MOTION_FORMAT`) ----

// How far a point drawn at `clip` (this frame's `view_proj`, jittered) moved on the screen since
// last frame, in UV (y down), without the jitter: `prev_clip` is its place then (`scene.taa_prev`
// times where it was, this frame's camera-relative), `jitter` is `scene.taa.xy`. Per vertex: a
// face's motion is close to linear across it.
fn screen_motion(clip: vec4<f32>, prev_clip: vec4<f32>, jitter: vec2<f32>) -> vec2<f32> {
    if clip.w <= 1e-4 || prev_clip.w <= 1e-4 {
        return vec2(0.0);
    }
    let now = clip.xy / clip.w - jitter;
    let then = prev_clip.xy / prev_clip.w;
    return clamp((then - now) * vec2(0.5, -0.5), vec2(-1.0), vec2(1.0));
}

// What a surface writes to the motion image (`@location(1)`): with `m.z` 1, its own motion `m.xy`
// tagged with its depth (`clip.z` in the fragment), which the gathering checks against the world's
// depth there; else a depth of -1, the camera's motion (`taa.wgsl`).
fn motion_texel(m: vec3<f32>, depth: f32) -> vec4<f32> {
    if m.z > 0.5 {
        return vec4(m.xy, depth, 0.0);
    }
    return vec4(0.0, 0.0, -1.0, 0.0);
}
