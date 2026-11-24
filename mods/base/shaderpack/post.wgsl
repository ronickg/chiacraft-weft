// chiacraft's screen effects' frame: the engine's post.wgsl with look.wgsl in front (as every file of
// the pack), so the effects' last pass can finish the HDR world's picture (`finish_picture`). Kept in step with
// crates/engine/assets/shaders/post.wgsl.
// Screen effects: each active effect is one full screen pass over the previous result.

// The effect's half-size image (`p.low`) at `uv`, filtered; zeros when it has none.
fn post_low_at(p: PostPixel, uv: vec2<f32>) -> vec4<f32> {
    if p.low == 0u {
        return vec4(0.0);
    }
    return textureSampleLevel(textures[p.low - 1u], linear_clamp, uv, 0.0);
}

// The half-size image at this pixel, from its four nearest texels weighted by how near their
// depth is to this pixel's, so it doesn't bleed across a silhouette (Photon's and Unity's
// half-resolution volumetrics).
fn post_low_upsampled(p: PostPixel) -> vec4<f32> {
    if p.low == 0u {
        return vec4(0.0);
    }
    let low = textures[p.low - 1u];
    let dims = vec2<i32>(textureDimensions(low));
    let f = p.uv * vec2<f32>(dims) - 0.5;
    let base = vec2<i32>(floor(f));
    let t = fract(f);
    let here = length(post_position(p, p.uv));
    var sum = vec4(0.0);
    var weight = 0.0;
    for (var j = 0; j < 4; j++) {
        let o = vec2(j % 2, j / 2);
        let at = clamp(base + o, vec2(0), dims - 1);
        let bil = mix(1.0 - t.x, t.x, f32(o.x)) * mix(1.0 - t.y, t.y, f32(o.y));
        let uv = (vec2<f32>(at) + 0.5) / vec2<f32>(dims);
        let there = length(post_position(p, uv));
        let w = bil / (0.02 + abs(there - here) / max(here, 1.0)) + 1e-5;
        sum += textureLoad(low, at, 0) * w;
        weight += w;
    }
    return sum / weight;
}

// The image so far anywhere else (blur, waves).
fn scene_at(p: PostPixel, uv: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(textures[p.scene], linear_clamp, uv, 0.0);
}

// The world's depth at `uv` (reverse Z: 0 where nothing was drawn, the sky). Read texel by texel,
// as not every GPU filters depth.
fn post_depth(p: PostPixel, uv: vec2<f32>) -> f32 {
    // By the depth's own size: under an upscaler the effects run at the screen's, the depth at
    // the world's (`Post::set_upscale`).
    let size = vec2<i32>(textureDimensions(textures[p.depth]));
    let at = clamp(vec2<i32>(uv * vec2<f32>(size)), vec2(0), size - vec2(1));
    return textureLoad(textures[p.depth], at, 0).r;
}

// Where the world is at `uv`, camera relative (blocks); the sky (depth 0) is very far that way.
fn post_position(p: PostPixel, uv: vec2<f32>) -> vec3<f32> {
    let d = max(post_depth(p, uv), 1e-7);
    let ndc = vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
    let w = scene.inv_view_proj * ndc;
    return w.xyz / w.w;
}

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) pass_index: u32,
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
    var out: VOut;
    out.clip = vec4(p * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2(p.x, 1.0 - p.y);
    out.pass_index = ii;
    return out;
}

// One effect, or with `pad` > 0 that many in a row on this pixel (`packs::merge_effects`), each
// on the color the one before left; only the first sees the image's alpha.
@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let first = passes[in.pass_index];
    // Scaled up by `core:upscaler`'s filter when the world is drawn smaller (`world_upscaled`).
    var color = world_upscaled(first.scene, in.uv, in.clip.xy, scene.upscale.x);
    let count = max(first.pad, 1u);
    for (var k = 0u; k < count; k++) {
        let pass_ = passes[in.pass_index + k];
        let px = PostPixel(in.uv, color, pass_.strength, pass_.time, pass_.size, first.scene, pass_.depth, pass_.low);
        color = vec4(post_apply(pass_.slot, px).rgb, 1.0);
        // The HDR world's last effect finishes the picture (chiacraft's tone curve and grade,
        // `finish_picture` in look.wgsl): the surfaces left it in linear light. Checked on each
        // entry, as merged (`pad`) the last effect is not the first entry, which alone was read
        // before, so a merged pass never finished the picture.
        if pass_.spare == 1u && scene.frame.x > 0.5 {
            color = vec4(finish_picture(hdr_light(color.rgb + sharpen_delta(first.scene, in.uv), in.uv, pass_)), 1.0);
        }
    }
    return color;
}

// An effect's own half-size image (`shaders/post_low/<name>.wgsl`), drawn before the effects: the
// world's image and depth as the effects see them, at a quarter of the pixels (chiacraft).
@fragment
fn fs_low(in: VOut) -> @location(0) vec4<f32> {
    let pass_ = passes[in.pass_index];
    let color = textureSampleLevel(textures[pass_.scene], linear_clamp, in.uv, 0.0);
    let px = PostPixel(in.uv, color, pass_.strength, pass_.time, pass_.size, pass_.scene, pass_.depth, 0u);
    return post_low(pass_.slot, px);
}

// A copy of the world before what is seen through is drawn (`Post::copy_behind`): its color and
// depth, for packs whose water bends and tints what lies under it (`scene.behind`).
struct BehindOut {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn fs_copy_behind(in: VOut) -> BehindOut {
    let pass_ = passes[in.pass_index];
    let px = vec2<i32>(in.clip.xy);
    var out: BehindOut;
    out.color = textureLoad(textures[pass_.scene], px, 0);
    out.depth = textureLoad(textures[pass_.depth], px, 0).r;
    return out;
}

// ---- the HDR world's bloom and exposure (`post::Pyramid`, `core:bloom`) ----
// Linear light past 1 spills a soft glow round it at every scale, as a lens and the eye do, from a
// pyramid of the world's picture: each level half the one before (a 13-tap filter, the
// "next generation post processing" talk's, Jimenez, SIGGRAPH 2014), then back up, each level
// blended with the wider one over it, so the widest holds them all. No threshold: everything glows
// a little and the brightest most, which is what softens a bright sky's edges and the sun.

// A texel of the picture, finite and capped (one NaN or a sun texel of thousands would spread
// through every level).
fn bloom_tap(tex: u32, uv: vec2<f32>) -> vec3<f32> {
    let c = textureSampleLevel(textures[tex], linear_clamp, uv, 0.0).rgb;
    let ok = all(c == c) && all(c < vec3(1e6));
    return select(vec3(0.0), min(max(c, vec3(0.0)), vec3(256.0)), ok);
}

// Four taps' average, weighed down where it is bright (Karis's average): on the first level only,
// so a few sparkling texels don't flicker into big blots.
fn bloom_group(a: vec3<f32>, b: vec3<f32>, c: vec3<f32>, d: vec3<f32>, karis: bool) -> vec3<f32> {
    if !karis {
        return (a + b + c + d) * 0.25;
    }
    let wa = 1.0 / (1.0 + luma(a));
    let wb = 1.0 / (1.0 + luma(b));
    let wc = 1.0 / (1.0 + luma(c));
    let wd = 1.0 / (1.0 + luma(d));
    return (a * wa + b * wb + c * wc + d * wd) / (wa + wb + wc + wd);
}

// One level down: `scene` the level above it (the world's picture for the first, `pad` 0).
@fragment
fn fs_bloom_down(in: VOut) -> @location(0) vec4<f32> {
    let pass_ = passes[in.pass_index];
    let src = pass_.scene;
    let t = 1.0 / vec2<f32>(textureDimensions(textures[src]));
    let uv = in.uv;
    let a = bloom_tap(src, uv + t * vec2(-2.0, -2.0));
    let b = bloom_tap(src, uv + t * vec2(0.0, -2.0));
    let c = bloom_tap(src, uv + t * vec2(2.0, -2.0));
    let d = bloom_tap(src, uv + t * vec2(-1.0, -1.0));
    let e = bloom_tap(src, uv + t * vec2(1.0, -1.0));
    let f = bloom_tap(src, uv + t * vec2(-2.0, 0.0));
    let g = bloom_tap(src, uv);
    let h = bloom_tap(src, uv + t * vec2(2.0, 0.0));
    let i = bloom_tap(src, uv + t * vec2(-1.0, 1.0));
    let j = bloom_tap(src, uv + t * vec2(1.0, 1.0));
    let k = bloom_tap(src, uv + t * vec2(-2.0, 2.0));
    let l = bloom_tap(src, uv + t * vec2(0.0, 2.0));
    let m = bloom_tap(src, uv + t * vec2(2.0, 2.0));
    let karis = pass_.pad == 0u;
    let sum = bloom_group(d, e, i, j, karis) * 0.5
        + (bloom_group(a, b, f, g, karis) + bloom_group(b, c, g, h, karis) + bloom_group(f, g, k, l, karis) + bloom_group(g, h, l, m, karis)) * 0.125;
    return vec4(sum, 1.0);
}

// How much of the wider levels each level keeps on the way up: 1 weighs each level about as Bliss
// does its seven (1, 1/2, 1/3 ... of a normalised blur); 1.3 a little wider, for the glare round a
// low sun.
const BLOOM_SPREAD: f32 = 1.3;

// One level up: this level (`scene`) and the wider one over it (`low` - 1), filtered by a 3x3 tent.
@fragment
fn fs_bloom_up(in: VOut) -> @location(0) vec4<f32> {
    let pass_ = passes[in.pass_index];
    let wide = pass_.low - 1u;
    let t = 1.0 / vec2<f32>(textureDimensions(textures[wide]));
    let uv = in.uv;
    var tent = bloom_tap(wide, uv) * 4.0;
    tent += (bloom_tap(wide, uv + vec2(t.x, 0.0)) + bloom_tap(wide, uv - vec2(t.x, 0.0)) + bloom_tap(wide, uv + vec2(0.0, t.y)) + bloom_tap(wide, uv - vec2(0.0, t.y))) * 2.0;
    tent += bloom_tap(wide, uv + t) + bloom_tap(wide, uv - t) + bloom_tap(wide, uv + vec2(t.x, -t.y)) + bloom_tap(wide, uv + vec2(-t.x, t.y));
    let here = bloom_tap(pass_.scene, uv);
    return vec4((here + tent / 16.0 * BLOOM_SPREAD) / (1.0 + BLOOM_SPREAD), 1.0);
}

// The exposure: how bright the view is, as the geometric mean of its luma (a dark forest floor and
// a bright sky both count, neither swamps), from the pyramid's narrowest level, the middle weighed
// most, eased over time as the eye does: toward brighter in under a second, toward darker over a
// few. r the eased log of it, g the mean itself; last frame's in `low` - 1, the seconds since in
// `strength`, `spare` 1 when there is no last frame.
@fragment
fn fs_exposure(in: VOut) -> @location(0) vec4<f32> {
    let pass_ = passes[in.pass_index];
    var sum = 0.0;
    var weight = 0.0;
    for (var j = 0; j < 8; j++) {
        for (var i = 0; i < 8; i++) {
            let uv = (vec2(f32(i), f32(j)) + 0.5) / 8.0;
            let w = 1.0 - 0.7 * smoothstep(0.1, 0.75, length(uv - vec2(0.5)));
            sum += w * log(max(luma(bloom_tap(pass_.scene, uv)), 1e-5));
            weight += w;
        }
    }
    let now = sum / weight;
    var eased = now;
    if pass_.spare == 0u {
        let before = textureLoad(textures[pass_.low - 1u], vec2(0, 0), 0).r;
        if before == before && abs(before) < 50.0 {
            let tau = select(2.5, 0.8, now > before);
            eased = mix(before, now, 1.0 - exp(-pass_.strength / tau));
        }
    }
    return vec4(eased, exp(eased), 0.0, 1.0);
}

// How much brighter or darker the view is taken for how bright it is (`mean`, the exposure's luma):
// a gentle log curve as Bliss's (0.18 / log2(2.5 L + 1.045)), relative to an average sunlit view so a
// clear day keeps the look as made, half of it applied (bright views stay bright, dark ones dark),
// and clamped: never more than a fifth brighter at night outdoors (nights stay dark), so a torch
// still matters.
const EXPOSURE_REF: f32 = 0.35;
fn exposure_abs() -> f32 {
    return clamp(fantasy_opt(OPT_EXPOSURE_ABS), 0.0, 1.0);
}
fn auto_exposure(mean: f32) -> f32 {
    let target_at = 0.18 / log2(2.5 * max(mean, 1e-5) + 1.045);
    let ref_at = 0.18 / log2(2.5 * EXPOSURE_REF + 1.045);
    // By day all of Bliss's curve, as its eye (dimensions/deferred.vsh:248-254 applies it in full):
    // a forest seen from below came out 1.4-1.9x darker than Bliss's while its own light, at a
    // matched vista, was already brighter (Trees, forest-exposure.md, 11-06): Bliss lifts a dark
    // forest about 4.8x over the open vista, half the curve lifted ours 1.6x. Times 0.83 so the open
    // vista stays where it was, and up to 4 (below). Trees' e3: forest mean 08/12/18 65/75/81
    // against Bliss's 71/71/73 (dev 41/49/55); the vista's land 84/97/96 (dev 87/100/100, Bliss
    // 85/109/98). Only with the sun up: night and the 05:00 blue hour keep half the curve.
    let day_e = smoothstep(0.0, 0.15, scene.sun.y) * select(0.0, 1.0, open_sky());
    var k = pow(target_at / ref_at, mix(0.5, 1.0, day_e)) * mix(1.0, 0.83, day_e);
    // Or by day Bliss's curve in its own units (`base:look_exposure_abs`): our light x41.
    let abs_k = (log2(2.5 * EXPOSURE_REF * 41.0 + 1.045)) / log2(2.5 * max(mean, 1e-5) * 41.0 + 1.045);
    k = mix(k, abs_k, day_e * exposure_abs());
    let night = smoothstep(0.0, -0.15, scene.sun.y) * select(1.0, 0.0, !open_sky());
    // Under rain the view is darker and stays so (a storm's mood; brought up, its mist went milky):
    // no brighter than as made, in a storm a little darker.
    let wet = rain_amount() * select(1.0, 0.0, !open_sky());
    // Before sunrise (the sun below, coming up in the east: `scene.sun.x` above 0) the night is
    // brought up less, 0.7 at most, so 04:00 stays dark as Bliss's (land luma 40 against 16 with
    // blue hour v4, 11-06); after sunset it keeps 1.2 (Bliss's 20:00 is brighter than ours).
    let ceiling = mix(1.2, 0.7, smoothstep(-0.2, 0.2, scene.sun.x));
    return clamp(k, 0.6, mix(mix(mix(2.0, 4.0, day_e), ceiling, night), 1.0 - 0.1 * scene.weather.w, wet));
}

// How strong the bloom is: a share of the glow mixed in (it keeps the light's sum), more in rain
// and fog, where the air itself scatters.
const BLOOM_MIX: f32 = 0.1;

// Contrast-adaptive sharpening (AMD's CAS, its published method; Bliss runs it at 0.35-0.4) for the
// HDR world's last pass, `base:look_sharpen`: what it would add at `uv` from the world's own picture
// (`scene`, before the effects), added to the finished light, so the effects between stay as
// they are. The bloom's share of a blur and the curve's toe softened the canopy's fine detail
// (roughness 5.2 against today's 9.9 at spot C's noon).
fn sharpen_delta(scene_tex: u32, uv: vec2<f32>) -> vec3<f32> {
    let k = clamp(fantasy_opt(OPT_SHARPEN), 0.0, 1.0);
    if k <= 0.0 || scene.frame.x < 0.5 {
        return vec3(0.0);
    }
    let t = textures[scene_tex];
    let size = vec2<i32>(textureDimensions(t));
    let px = clamp(vec2<i32>(uv * vec2<f32>(size)), vec2(1), size - vec2(2));
    let c = textureLoad(t, px, 0).rgb;
    let n = textureLoad(t, px + vec2(0, -1), 0).rgb;
    let s = textureLoad(t, px + vec2(0, 1), 0).rgb;
    let e = textureLoad(t, px + vec2(1, 0), 0).rgb;
    let w = textureLoad(t, px + vec2(-1, 0), 0).rgb;
    // On the light's log, so a bright sky and a dark forest are sharpened alike.
    let lc = log2(1.0 + luma(c));
    let ln = log2(1.0 + luma(n));
    let ls = log2(1.0 + luma(s));
    let le = log2(1.0 + luma(e));
    let lw = log2(1.0 + luma(w));
    let mn = min(lc, min(min(ln, ls), min(le, lw)));
    let mx = max(lc, max(max(ln, ls), max(le, lw)));
    let amp = sqrt(clamp(min(mn, 2.0 - mx) / max(mx, 1e-4), 0.0, 1.0));
    // Where the picture moves under temporal anti-aliasing (`scene.taa_frame`: how far each pixel
    // moved this frame), more: a gathered picture is read between its texels every frame it moves
    // and comes out softer while the camera turns (mc_forest mid-turn grain 7.3 against 10.4 still).
    // Up to the full CAS lobe and twice its push by 6 px a frame, held inside the neighbourhood so
    // edges don't ring. Only here, on the way to the screen: in the gathered picture itself it would
    // build up frame on frame.
    let moving = taa_motion_px(px, size);
    let m = smoothstep(0.75, 6.0, moving);
    let wt = -amp * mix(1.0 / 8.0, 1.0 / 5.0, clamp(k + m, 0.0, 1.0));
    let sharp = (c + (n + s + e + w) * wt) / (1.0 + 4.0 * wt);
    let pushed = c + (sharp - c) * (1.0 + m);
    if m <= 0.0 {
        // Still, or without the gathering: CAS as it was.
        return max(sharp, vec3(0.0)) - c;
    }
    let lo = min(c, min(min(n, s), min(e, w)));
    let hi = max(c, max(max(n, s), max(e, w)));
    return clamp(max(pushed, vec3(0.0)), lo, hi) - c;
}

// How far the pixel at `px` of an image `size` moved this frame, in pixels, from temporal
// anti-aliasing's count image (`scene.taa_frame`, g); 0 without it.
fn taa_motion_px(px: vec2<i32>, size: vec2<i32>) -> f32 {
    if scene.taa_frame.x < 0.5 {
        return 0.0;
    }
    let t = textures[u32(scene.taa_frame.x) - 1u];
    let dims = vec2<i32>(textureDimensions(t));
    let at = clamp(px * dims / max(size, vec2(1)), vec2(0), dims - 1);
    return textureLoad(t, at, 0).g;
}

// How much of the view at `uv` the mist covers (0 none to 1), from the mist volume (`froxels`) at
// the world's depth there; 0 without the volume.
fn mist_over(uv: vec2<f32>, pass_: Pass) -> f32 {
    if scene.froxels.x < 0.5 {
        return 0.0;
    }
    let size = vec2<i32>(textureDimensions(textures[pass_.depth]));
    let at = clamp(vec2<i32>(uv * vec2<f32>(size)), vec2(0), size - vec2(1));
    let d = max(textureLoad(textures[pass_.depth], at, 0).r, 1e-7);
    let w = scene.inv_view_proj * vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
    let dist = min(length(w.xyz / w.w), FROXEL_FAR);
    let f = froxel_slice(dist);
    let k = i32(floor(f));
    let v = mix(froxel_at(uv, k - 1), froxel_at(uv, min(k, i32(FROXEL_SLICES) - 1)), fract(f));
    return clamp(1.0 - v.a, 0.0, 1.0);
}

// Whether the pixel at `uv` with depth `d` is the sky's: nothing drawn (depth 0), or a cloud (the
// clouds write their depth too). Land, never that high, is the rest. With the clouds counted as
// land, the sky's boost and dimming stopped at their faint edges (where they stop drawing), cut
// hard edges round the blue between them and a flat band under the deck's far end, and the clouds
// took the land's brightening instead (mc_forest noon cloud 228 against Bliss's 201, gr5).
fn sky_or_cloud(uv: vec2<f32>, d: f32) -> bool {
    if d <= 0.0 {
        return true;
    }
    let w = scene.inv_view_proj * vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
    // The soft deck's far end: low over the horizon its looks reach the layer past `SOFT_REACH`,
    // and its depth is held at that distance, under the deck's height there, so the height test
    // below took a band of it for land (a dashed line at the deck's end that temporal
    // anti-aliasing, gathering the deck's faint edge, made a solid pale band: mc_forest noon, 11-06).
    // Only at that distance: distant terrain reaches past it (256 chunks), and its far land
    // stays land.
    if scene.soft_clouds.x > 0.5 && abs(length(w.xyz / w.w) - SOFT_REACH) < 4.0 {
        return true;
    }
    // The soft deck's underside (as the lightning's, `look.wgsl`), less a margin; the blocky
    // clouds' slab without soft ones.
    let lift = select(0.0, SOFT_LIFT * (1.0 - 0.7 * max(min(rain_amount() * 0.75 + scene.weather.w * 0.25, 1.0), scene.weather.w)), scene.soft_clouds.x > 0.5);
    return w.y / w.w > scene.clouds.z + lift - 20.0;
}

// How much of the sky's light the HDR world's veiling glare lays over the picture (`hdr_light`).
const GLARE_HDR: f32 = 0.02;
// The land's lift in the evening twilight (`hdr_light`).
const TWILIGHT_LAND_HDR: f32 = 1.55;

// The HDR world's light before the tone curve: the bloom mixed in and the exposure applied.
fn hdr_light(c: vec3<f32>, uv: vec2<f32>, pass_: Pass) -> vec3<f32> {
    var col = max(c, vec3(0.0));
    // The sky a third more saturated by day (where nothing was drawn, depth 0): the curve's
    // shoulder takes colour out of a bright sky, and ours read grey-white where Bliss's is blue
    // overhead and gold toward a low sun (08:00, saturation 26 against 43). Not in rain.
    let size = vec2<i32>(textureDimensions(textures[pass_.depth]));
    let px = clamp(vec2<i32>(uv * vec2<f32>(size)), vec2(0), size - vec2(1));
    let sky_px = sky_or_cloud(uv, textureLoad(textures[pass_.depth], px, 0).r);
    LAND_PIXEL = select(1.0, 0.0, sky_px);
    if sky_px {
        let y = luma(col);
        // More as the sun climbs, where the land's saturation is taken down most (0.62 of it with a
        // high sun, `finish_picture`): the sky kept its blue, which already read paler than Bliss's
        // (mc_forest noon 22 against 31, 08:00 27 against 43).
        let open = smoothstep(0.0, 0.15, scene.sun.y) * (1.0 - rain_amount());
        // (Twice the first try's rise: 0.6 more was all taken back by the land's lower saturation
        // and the curve's shoulder, noon 22 still, gr2. At 1.2 the blue between the clouds went
        // past Bliss's, noon 67 against 48 (dev 33): 0.8.)
        // Only up the sky, not low over the horizon, where Bliss's stays pale haze: boosted there
        // the band between the clouds' far end and the land went deep blue, and the clouds' cut
        // edge along the far land's outline showed (mc_forest noon, gr5).
        let wv = scene.inv_view_proj * vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 1.0, 1.0);
        let up = smoothstep(0.03, 0.3, normalize(wv.xyz / wv.w).y);
        // Most in the morning, less at noon: with the clouds counted as sky the noon blue between
        // them went to saturation 75 against Bliss's 48 and the clouds' shaded bellies steel blue,
        // while 08:00's stayed 35 against 45 (skf).
        let k = 1.0 + (1.3 - 0.9 * smoothstep(0.45, 0.8, scene.sun.y)) * open * up;
        col = max(vec3(y) + (col - vec3(y)) * k, vec3(0.0));
        // And dimmer by day: ours read brighter than Bliss's at every hour (mc_forest 08:00
        // 216 against 208, noon 217 against 192, 17:00 193 against 171). The day's brightening
        // below leaves it out (on it too, the 08:00 sky went to 224, gr2).
        // (30%: at 15% the sky, mostly bright cloud on the curve's shoulder, barely moved, noon 216
        // to 213, gr3.)
        // (25%: with the clouds dimmed too, 30% took the blue between them to 155 against 172, skf.)
        col *= 1.0 - 0.25 * open;
    }
    if pass_.bloom != 0u {
        let glow = bloom_tap(pass_.bloom - 1u, uv);
        let air = clamp(max(rain_amount(), air_mist()), 0.0, 1.0);
        // Bloomy fog (Bliss's): where mist covers the view (the mist volume's own opacity there,
        // `froxels`), it takes much more of the glow, so thick mist reads as soft light rather than
        // a grey wall, and its banks' noise melts.
        // Not in rain: its mist taking the glow too turned a storm's land to milk (the storm sheet,
        // 11-05), where Bliss's stays dark and readable.
        let fog = mist_over(uv, pass_) * (1.0 - 0.85 * rain_amount());
        // Or by day added on top, as Bliss's (`base:look_bloom_add`, read raw: 0 as made, 1 all the
        // way): the glow x 0.5 E^0.2 (0.23 at its noon exposure) x (1.5 - 2 r^2), so +35% at the
        // centre and +12% in the corners (`composite11.fsh:99, 141, 154`; Research's code read,
        // 11-22). The mist's share stays mixed.
        let add = clamp(fantasy_opt(OPT_BLOOM_ADD), 0.0, 1.0) * smoothstep(0.0, 0.15, scene.sun.y);
        let kept = BLOOM_MIX * (1.0 + air * (1.0 - rain_amount())) * (1.0 - add);
        col = mix(col, glow, clamp((kept + fog * 0.6) * fantasy_opt(OPT_BLOOM), 0.0, 0.8));
        let r2 = dot(uv - vec2(0.5), uv - vec2(0.5));
        col += glow * 0.234 * (1.5 - 2.0 * r2) * add * fantasy_opt(OPT_BLOOM);
    }
    // Veiling glare: a little of the sky's light over the whole picture, as a lens and the eye
    // scatter it, so the darkest shade is a dim sky blue, not black. In Bliss's own world (imported)
    // its shade is blue at every distance (sRGB 27 47 69 at noon), where ours went to 0 0 0 in the
    // curve's toe (Tester 2's calm noon: the shaded block sides; more fill light didn't lift them,
    // sh1b). Not at night or in rain.
    // As much as the view is bright (the exposure's mean against an average sunlit view's), as a
    // lens's glare is: at a fixed share of the sky's light, a dark forest's shade went a neutral
    // blue-grey (shaded leaves' hue 172-180°, saturation 0.17-0.20) where Bliss's is green and
    // near black (69-107°, 0.92-1.0); without the glare ours went green again (Trees' runs
    // trb_s2, Bliss's world, mc_world_forest). Open land keeps it.
    var lens = 1.0;
    if pass_.exposure != 0u {
        let m = textureLoad(textures[pass_.exposure - 1u], vec2(0, 0), 0).g;
        if m == m && m > 0.0 {
            lens = clamp(m / EXPOSURE_REF, 0.0, 1.5);
        }
    }
    let p = palette();
    col += (p.zenith * 0.4 + p.horizon * 0.6) * GLARE_HDR * max(fantasy_opt(OPT_GLARE), 0.0) * lens * smoothstep(-0.05, 0.2, scene.sun.y) * (1.0 - rain_amount());
    if pass_.exposure != 0u {
        let e = textureLoad(textures[pass_.exposure - 1u], vec2(0, 0), 0).g;
        if e == e && e > 0.0 {
            col *= pow(auto_exposure(e), fantasy_opt(OPT_AUTO_EXPOSURE));
            // The land a little dimmer against the sky with the sun 30-55° up: in Bliss's own world
            // the sky matched at every hour (08:00 204 against 208, noon 189 against 192), the land
            // was over it (113 against 89 at 08:00, 132 against 113 at noon; 17:00 115 against 113).
            // Not the sky (its own dim is above), not in rain.
            let morning = smoothstep(0.38, 0.52, scene.sun.y);
            let land = 1.0 - 0.2 * morning + 0.06 * smoothstep(0.6, 0.82, scene.sun.y);
            // The near land only, fading out from 300 to 900 blocks: on the far band too it took
            // mc_forest's noon far land from 135 to 109 (Bliss's 148, Far view's measure).
            let d = max(textureLoad(textures[pass_.depth], px, 0).r, 1e-7);
            let wp = scene.inv_view_proj * vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
            let near = 1.0 - smoothstep(300.0, 900.0, length(wp.xyz / wp.w));
            col *= mix(1.0, land, (1.0 - rain_amount()) * select(1.0, 0.0, sky_px) * near * (1.0 - exposure_abs()));
            // Evening twilight, the sun just under the horizon in the west: the land lit up against the
            // sky, as Bliss's evening (its own world at 19:00: land 86 against a sky of 138, ours 60
            // against 162; a stronger exposure lifted both, tw1). Its morning at the same depth stays
            // a dark blue, as ours. Not out to the far ridges, which meet the sky's own colour, and
            // not into the night (dark by design).
            let dusk = smoothstep(0.05, -0.05, scene.sun.x) * smoothstep(0.03, -0.02, scene.sun.y) * smoothstep(-0.16, -0.08, scene.sun.y);
            let near_dusk = 1.0 - smoothstep(600.0, 1500.0, length(wp.xyz / wp.w));
            col *= mix(1.0, TWILIGHT_LAND_HDR, dusk * (1.0 - rain_amount()) * select(1.0, 0.0, sky_px) * near_dusk);
        }
    }
    return col;
}
