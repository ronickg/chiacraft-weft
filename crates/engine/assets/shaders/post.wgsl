// Screen effects: each active effect is one full screen pass over the previous result.

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
        let px = PostPixel(in.uv, color, pass_.strength, pass_.time, pass_.size, first.scene, pass_.depth);
        color = vec4(post_apply(pass_.slot, px).rgb, 1.0);
    }
    return color;
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
