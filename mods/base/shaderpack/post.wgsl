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
    let size = vec2<i32>(p.size);
    let at = clamp(vec2<i32>(uv * p.size), vec2(0), size - vec2(1));
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
    var color = textureSampleLevel(textures[first.scene], linear_clamp, in.uv, 0.0);
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
            color = vec4(finish_picture(color.rgb), 1.0);
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
