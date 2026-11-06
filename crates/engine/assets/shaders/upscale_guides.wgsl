// The guides NVIDIA's Ray Reconstruction reads beside the world (`post::Post::set_upscale`,
// `weft_render::Cmd::Upscale`), at the world's size. Approximate for now: the world's pass has no
// material outputs of its own, so
// - the normal comes from the depth (exact on block faces, the neighbour on the nearer side
//   across an edge), in world axes (camera-relative), w the roughness (0.85 everywhere);
// - the diffuse albedo is the picture over twice its local light (the 5x5 mean luminance round it),
//   so what RR takes for light (colour / albedo) is that smooth local level, and every texel's
//   detail stays surface. The first try, the picture tone-mapped (c / (1 + c)), left the light
//   as uneven as the picture: RR smoothed it and lifted the dark gaps between leaves (washed out,
//   Proton run 11-06);
// - the specular albedo follows from a dielectric's 4 % (Karis's approximation) at that roughness.
// Entry: `scene` the world's picture, `depth` its depth, `size` the world's size.
// When the frame was path traced, `fs_pt_guides` draws exact ones from its G-buffer instead.

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

fn guide_luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3(0.2126, 0.7152, 0.0722));
}

// The picture's mean luminance over the 5x5 texels round `at`: the local light level.
fn guide_light(scene_tex: u32, at: vec2<i32>, size: vec2<i32>) -> f32 {
    var sum = 0.0;
    for (var y = -2; y <= 2; y++) {
        for (var x = -2; x <= 2; x++) {
            let p = clamp(at + vec2(x, y), vec2(0), size - 1);
            sum += guide_luma(max(textureLoad(textures[scene_tex], p, 0).rgb, vec3(0.0)));
        }
    }
    return sum / 25.0;
}

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
    // Twice the local light: a texel as bright as its surroundings is mid-grey surface; one over
    // twice as bright keeps the rest as light (a glint).
    let light = max(guide_light(pass_.scene, at, size), 1e-4);
    out.albedo = vec4(clamp(c / (2.0 * light), vec3(0.0), vec3(1.0)), 1.0);
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
    return vec4(pt_spec_albedo(vec3(0.04), GUIDE_ROUGHNESS, nov), 1.0);
}

struct PtGuideOut {
    @location(0) specular: vec4<f32>,
    @location(1) hit_distance: vec4<f32>,
}

// From the path tracer's G-buffer (`post::PtGuides`): the specular albedo by the surface's own F0
// and roughness (the same `pt_spec_albedo` the path tracer divided by), and the specular hit
// distance in r. Entry: `scene` the F0 image, `low` the normals + roughness + 1, `bloom` the path
// tracer's specular light + 1 (its alpha the hit distance), `depth`, `size`.
@fragment
fn fs_pt_guides(in: VOut) -> PtGuideOut {
    let pass_ = passes[in.pass_index];
    let size = vec2<i32>(pass_.size);
    let at = vec2<i32>(in.clip.xy);
    var out: PtGuideOut;
    if textureLoad(textures[pass_.depth], at, 0).r <= 0.0 || pass_.low == 0u || pass_.bloom == 0u {
        out.specular = vec4(0.0);
        out.hit_distance = vec4(0.0);
        return out;
    }
    let nr = textureLoad(textures[pass_.low - 1u], at, 0);
    let f0 = textureLoad(textures[pass_.scene], at, 0).rgb;
    let v = -normalize(guide_position(pass_.depth, at, size));
    let n = normalize(nr.xyz + vec3(0.0, 0.0, 1e-6));
    out.specular = vec4(pt_spec_albedo(f0, nr.w, abs(dot(n, v))), 1.0);
    out.hit_distance = vec4(max(textureLoad(textures[pass_.bloom - 1u], at, 0).a, 0.0), 0.0, 0.0, 1.0);
    return out;
}
