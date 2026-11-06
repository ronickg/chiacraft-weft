// chiacraft's particles: lit and graded like the world.
// Particles: camera-facing quads, one instance each, all kinds in one draw per blend mode.

var<private> CORNER: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0),
);

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) quad: vec2<f32>,
    @location(2) @interpolate(flat) data: vec4<u32>,
    @location(3) @interpolate(flat) life: f32,
    @location(4) dist: f32,
    @location(5) rel: vec3<f32>,
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    let p = particles[ii];
    let c = CORNER[vi % 6u];
    var across = scene.right.xyz;
    var down = -scene.up.xyz;
    if p.extra.y > 0.5 {
        // A card that turns in 3D (a falling leaf): flat, then tipped by `extra.w` about the line
        // its heading `extra.z` gives.
        let h = vec2(cos(p.extra.z), sin(p.extra.z));
        across = vec3(h.x, 0.0, h.y);
        down = vec3(-h.y, 0.0, h.x) * cos(p.extra.w) + vec3(0.0, sin(p.extra.w), 0.0);
    }
    let world = p.pos_size.xyz + (across * (c.x - 0.5) + down * (c.y - 0.5)) * p.pos_size.w;
    var out: VOut;
    out.clip = scene.view_proj * vec4(world, 1.0);
    let r = vec4<f32>(f32(p.data.z & 255u), f32((p.data.z >> 8u) & 255u), f32((p.data.z >> 16u) & 255u), f32(p.data.z >> 24u)) / 255.0;
    out.uv = mix(r.xy, r.zw, c);
    out.quad = c;
    out.data = p.data;
    out.life = p.extra.x;
    out.dist = length(world);
    out.rel = world;
    return out;
}

fn unpack_color(c: u32) -> vec4<f32> {
    let v = vec4<f32>(f32((c >> 24u) & 255u), f32((c >> 16u) & 255u), f32((c >> 8u) & 255u), f32(c & 255u)) / 255.0;
    return vec4(pow(v.rgb, vec3(2.2)), v.a);
}

fn shade(in: VOut) -> vec4<f32> {
    let albedo = textureSampleLevel(textures[in.data.y & PARTICLE_TEX_MASK], nearest_clamp, in.uv, 0.0);
    var color = unpack_color(in.data.x);
    color = particle_shade(in.data.y >> PARTICLE_SLOT_SHIFT, ParticleFrag(in.quad, albedo, color, in.life, scene.params.w));
    if in.data.w != 0xffffffffu {
        color = vec4(color.rgb * light_round(unpack_light(in.data.w), vec3(0.0), in.rel), color.a);
    } else {
        // Fullbright particles (sparks, flames) glow.
        color = vec4(color.rgb * 1.5, color.a);
    }
    let sky = select(1.0, unpack_light(in.data.w).x, in.data.w != 0xffffffffu);
    var rgb = color.rgb;
    // Under water (bubbles): a little lighter than the water's teal, not flat grey, and the ones
    // right at the eye (a bubble rising past the lens filled the screen as a hard pixel ring,
    // Tester 2's under-water shots) blend into the water.
    if in_water() {
        let teal = underwater_color();
        rgb = mix(rgb, teal * 2.0 + vec3(0.02, 0.04, 0.04), 0.6);
        rgb = mix(teal, rgb, smoothstep(0.8, 2.5, length(in.rel)));
    }
    return vec4(display(haze_lit(rgb, in.rel, sky)), color.a);
}

@fragment
fn fs_cutout(in: VOut) -> @location(0) vec4<f32> {
    let c = shade(in);
    if c.a < 0.5 {
        discard;
    }
    return vec4(c.rgb, 1.0);
}

@fragment
fn fs_blend(in: VOut) -> @location(0) vec4<f32> {
    let c = shade(in);
    if c.a < 0.01 {
        discard;
    }
    return c;
}
