// chiacraft's mobs and players: the engine's model shader, lit and graded like the world.
// Entity models: boxes and free-form triangles of every kind in two static tables; each entity
// layer (skin or something worn) is one instance pointing at its bone matrices, posed on the CPU
// (hierarchy, keyframes, vanilla's limb swing) and uploaded per frame.

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec3<f32>,
    @location(2) @interpolate(flat) tex: u32,
    @location(3) dist: f32,
    @location(4) @interpolate(flat) shader: u32,
    @location(5) @interpolate(flat) glow: u32,
    @location(6) rel: vec3<f32>,
    // 1 where the world lights it (not in a preview on a screen).
    @location(7) @interpolate(flat) graded: u32,
    // Sky light where it stands (0..1): fog gathers only out under the sky.
    @location(8) @interpolate(flat) sky: f32,
}

var<private> NORMAL: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
    vec3(-1.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), vec3(0.0, -1.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, -1.0), vec3(0.0, 0.0, 1.0),
);

// A direction turned by a bone (its rows without the move).
fn turn(bone: u32, d: vec3<f32>) -> vec3<f32> {
    let b = bones[bone];
    return normalize(vec3(dot(b.rows[0].xyz, d), dot(b.rows[1].xyz, d), dot(b.rows[2].xyz, d)) + vec3(0.0, 1e-6, 0.0));
}

fn place(bone: u32, p: vec3<f32>) -> vec3<f32> {
    let b = bones[bone];
    let q = vec4(p, 1.0);
    return vec3(dot(b.rows[0], q), dot(b.rows[1], q), dot(b.rows[2], q));
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    let inst = instances[ii];
    var out: VOut;
    var world: vec3<f32>;
    var shade: f32;
    var normal = vec3(0.0, 1.0, 0.0);
    let boxes = inst.cuboid_count * 36u;
    if vi < boxes {
        let cub = cuboids[inst.cuboids + vi / 36u];
        if ((inst.roles >> cub.role) & 1u) == 0u {
            out.clip = vec4(0.0, 0.0, 0.0, 1.0);
            return out;
        }
        let face = (vi % 36u) / 6u;
        let c = TRI[vi % 6u];
        let grow = cub.inflate + inst.inflate;
        let local = cub.origin - vec3(grow) + CORNERS[face * 4u + c] * (cub.size + vec3(2.0 * grow));
        world = place(inst.bones + cub.bone, local);
        // Constant indices: SPIRV-Cross cannot copy a runtime-indexed array out of a storage buffer to Metal.
        var r: vec4<f32>;
        switch face {
            case 0u: { r = cub.faces[0]; }
            case 1u: { r = cub.faces[1]; }
            case 2u: { r = cub.faces[2]; }
            case 3u: { r = cub.faces[3]; }
            case 4u: { r = cub.faces[4]; }
            default: { r = cub.faces[5]; }
        }
        out.uv = (r.xy + UVS[c] * r.zw) / inst.tex_size;
        shade = SHADE[face];
        normal = turn(inst.bones + cub.bone, NORMAL[face]);
    } else {
        let k = vi - boxes;
        let v = corners[inst.vertices + k];
        if ((inst.roles >> v.role) & 1u) == 0u {
            out.clip = vec4(0.0, 0.0, 0.0, 1.0);
            return out;
        }
        world = place(inst.bones + v.bone, v.pos);
        out.uv = v.uv / inst.tex_size;
        // Face shading from the triangle's normal, like the world's free-form quads.
        let t = inst.vertices + k - k % 3u;
        let a = place(inst.bones + corners[t].bone, corners[t].pos);
        let b = place(inst.bones + corners[t + 1u].bone, corners[t + 1u].pos);
        let c2 = place(inst.bones + corners[t + 2u].bone, corners[t + 2u].pos);
        let n = normalize(cross(b - a, c2 - a) + vec3(0.0, 1e-6, 0.0));
        shade = n.x * n.x * 0.6 + n.z * n.z * 0.8 + n.y * n.y * select(0.5, 1.0, n.y > 0.0);
        normal = n;
    }
    out.clip = scene.view_proj * vec4(world, 1.0);
    out.dist = length(world);
    out.rel = world;
    out.tex = inst.tex;
    var lit = vec3(1.0);
    if inst.light != 0xffffffffu {
        lit = lightmap(unpack_light(inst.light), scene.params.z, scene.light.x, scene.light.y);
    }
    let flash = select(mix(vec3(1.0), vec3(1.0, 0.4, 0.4), inst.hurt), vec3(2.0), inst.hurt < 0.0);
    out.graded = select(0u, 1u, inst.light != 0xffffffffu);
    out.sky = select(1.0, unpack_light(inst.light).x, inst.light != 0xffffffffu);
    if inst.light != 0xffffffffu {
        // The pack's light; no traced shadow here, the sky light stands in for it.
        let l = unpack_light(inst.light);
        let shadow = smoothstep(0.8, 1.0, l.x);
        out.color = pow(inst.color * flash, vec3(2.2)) * light_on(normal, l, shadow, 1.0, world);
    } else {
        out.color = pow(inst.color * lit * shade * flash, vec3(2.2));
    }
    out.shader = inst.shader;
    out.glow = inst.glow;
    return out;
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let albedo = textureSample(textures[in.tex], nearest_clamp, in.uv);
    var light = in.color;
    if in.glow != 0xffffffffu {
        let e = textureSampleLevel(textures[in.glow], nearest_clamp, in.uv, 0.0).r;
        light = max(light, vec3(pow(e, 2.2)));
    }
    let c = entity_shade(in.shader, EntityPixel(in.uv, albedo, light, scene.params.w));
    if c.a < 0.5 {
        discard;
    }
    if in.graded != 0u {
        return vec4(display(haze_lit(c.rgb, in.rel, in.sky)), SOLID_ALPHA);
    }
    let f = fog_amount(in.dist, scene.params.x, scene.params.y);
    return vec4(mix(c.rgb, scene.fog_color.rgb, f), 1.0);
}
