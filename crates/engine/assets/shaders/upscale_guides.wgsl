// The guides NVIDIA's Ray Reconstruction reads beside the world (`post::Post::set_upscale`,
// `weft_render::Cmd::Upscale`), at the world's size. Approximate for now: the world's pass has no
// material outputs of its own, so
// - the normal comes from the depth (exact on block faces, the neighbour on the nearer side
//   across an edge), in world axes (camera-relative), w the roughness (0.85 everywhere);
// - the diffuse albedo is the picture itself, tone-mapped into 0..1: Ray Reconstruction then keeps
//   every texel's detail as surface, not as light to smooth;
// - the specular albedo follows from a dielectric's 4 % (Karis's approximation) at that roughness.
// Entry: `scene` the world's picture, `depth` its depth, `size` the world's size.

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

const GUIDE_ROUGHNESS: f32 = 0.85;

// Where the surface at texel `at` is, camera-relative (blocks); the sky (depth 0) far that way.
fn guide_position(depth: u32, at: vec2<i32>, size: vec2<i32>) -> vec3<f32> {
    let p = clamp(at, vec2(0), size - 1);
    let d = max(textureLoad(textures[depth], p, 0).r, 1e-7);
    let uv = (vec2<f32>(p) + 0.5) / vec2<f32>(size);
    let h = scene.inv_view_proj * vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
    return h.xyz / h.w;
}

fn guide_normal(depth: u32, at: vec2<i32>, size: vec2<i32>) -> vec3<f32> {
    let c = guide_position(depth, at, size);
    let l = guide_position(depth, at - vec2(1, 0), size);
    let r = guide_position(depth, at + vec2(1, 0), size);
    let u = guide_position(depth, at - vec2(0, 1), size);
    let d = guide_position(depth, at + vec2(0, 1), size);
    // Each way the nearer neighbour: across an edge the far one lies on another surface.
    let dx = select(c - l, r - c, length(r - c) < length(c - l));
    let dy = select(c - u, d - c, length(d - c) < length(c - u));
    var n = normalize(cross(dy, dx));
    // Facing the camera (which is at the origin).
    if dot(n, c) > 0.0 {
        n = -n;
    }
    return n;
}

struct GuideOut {
    @location(0) normal_roughness: vec4<f32>,
    @location(1) albedo: vec4<f32>,
}

@fragment
fn fs_guides(in: VOut) -> GuideOut {
    let pass_ = passes[in.pass_index];
    let size = vec2<i32>(pass_.size);
    let at = vec2<i32>(in.clip.xy);
    let c = max(textureLoad(textures[pass_.scene], at, 0).rgb, vec3(0.0));
    var out: GuideOut;
    if textureLoad(textures[pass_.depth], at, 0).r <= 0.0 {
        // The sky: no surface, facing back at the camera.
        let v = guide_position(pass_.depth, at, size);
        out.normal_roughness = vec4(-normalize(v), 1.0);
    } else {
        out.normal_roughness = vec4(guide_normal(pass_.depth, at, size), GUIDE_ROUGHNESS);
    }
    out.albedo = vec4(c / (1.0 + c), 1.0);
    return out;
}

// Karis's fit of a dielectric's (4 %) specular reflectance by roughness and view angle.
@fragment
fn fs_specular(in: VOut) -> @location(0) vec4<f32> {
    let pass_ = passes[in.pass_index];
    let size = vec2<i32>(pass_.size);
    let at = vec2<i32>(in.clip.xy);
    if textureLoad(textures[pass_.depth], at, 0).r <= 0.0 {
        return vec4(0.0);
    }
    let n = guide_normal(pass_.depth, at, size);
    let v = -normalize(guide_position(pass_.depth, at, size));
    let nov = abs(dot(n, v));
    let c0 = vec4(-1.0, -0.0275, -0.572, 0.022);
    let c1 = vec4(1.0, 0.0425, 1.04, -0.04);
    let r = GUIDE_ROUGHNESS * c0 + c1;
    let a004 = min(r.x * r.x, exp2(-9.28 * nov)) * r.x + r.y;
    let ab = vec2(-1.04, 1.04) * a004 + r.zw;
    let s = 0.04 * ab.x + ab.y;
    return vec4(vec3(s), 1.0);
}
