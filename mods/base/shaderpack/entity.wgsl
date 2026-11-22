// chiacraft's hand, held and dropped items: the engine's, lit and graded like the world.


struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) @interpolate(flat) tex: u32,
    @location(3) @interpolate(flat) layer: u32,
    @location(4) @interpolate(flat) mode: u32,
    @location(5) dist: f32,
    @location(6) @interpolate(flat) glow: u32,
    // Unlit, unshaded color (linear) glowing texels take.
    @location(7) @interpolate(flat) tint: vec3<f32>,
    @location(8) rel: vec3<f32>,
    // 1 where the world lights it (not on a screen, not a gizmo).
    @location(9) @interpolate(flat) graded: u32,
    // Sky light where it is (0..1): fog gathers only out under the sky.
    @location(10) @interpolate(flat) sky: f32,
}

var<private> NORMAL: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
    vec3(-1.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), vec3(0.0, -1.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, -1.0), vec3(0.0, 0.0, 1.0),
);

// Texel rectangle (x, y, w, h) of a face in the classic box unwrap.
fn face_rect(face: u32, d: vec3<f32>, o: vec2<f32>) -> vec4<f32> {
    let w = d.x;
    let h = d.y;
    let z = d.z;
    switch face {
        // The hand, in vanilla's model space (y down): not `Cuboid::face_rects`, for a box on ours.
        case 0u: { return vec4(o.x + z, o.y + z, -z, h); }
        case 1u: { return vec4(o.x + z * 2.0 + w, o.y + z, -z, h); }
        case 2u: { return vec4(o.x + z + w, o.y, w, z); }
        case 3u: { return vec4(o.x + z, o.y, w, z); }
        case 4u: { return vec4(o.x + z, o.y + z, w, h); }
        default: { return vec4(o.x + z * 2.0 + w, o.y + z, w, h); }
    }
}

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> VOut {
    return box_vertex(boxes[vi / 36u], vi % 36u);
}

fn item_box(vi: u32, ii: u32) -> Box {
    let inst = boxes[ii];
    var b = item_boxes[inst.ids.z + vi / 36u];
    let r0 = b.rows[0];
    let r1 = b.rows[1];
    let r2 = b.rows[2];
    b.rows[0] = inst.rows[0].x * r0 + inst.rows[0].y * r1 + inst.rows[0].z * r2 + vec4(0.0, 0.0, 0.0, inst.rows[0].w);
    b.rows[1] = inst.rows[1].x * r0 + inst.rows[1].y * r1 + inst.rows[1].z * r2 + vec4(0.0, 0.0, 0.0, inst.rows[1].w);
    b.rows[2] = inst.rows[2].x * r0 + inst.rows[2].y * r1 + inst.rows[2].z * r2 + vec4(0.0, 0.0, 0.0, inst.rows[2].w);
    b.color.a = inst.color.a;
    b.ids2.y = inst.ids2.y;
    return b;
}

@vertex
fn vs_item(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    return box_vertex(item_box(vi, ii), vi % 36u);
}

// `v`: the vertex within the box, 6 per face.
fn box_vertex(b: Box, v: u32) -> VOut {
    let face = v / 6u;
    let c = TRI[v % 6u];
    var p = CORNERS[face * 4u + c];
    // One face of a block's box, as the world mesher makes it: the rows place a unit square.
    if b.ids.y == BOX_MODE_QUAD {
        p = vec3(UVS[c].x, 1.0 - UVS[c].y, 0.0);
    }
    let local = vec4(p, 1.0);
    let world = vec3(dot(b.rows[0], local), dot(b.rows[1], local), dot(b.rows[2], local));
    var out: VOut;
    out.clip = scene.view_proj * vec4(world, 1.0);
    out.dist = length(world);
    out.rel = world;
    let uv = UVS[c];
    out.mode = b.ids.y;
    out.tex = b.ids.x;
    out.glow = 0xffffffffu;
    if b.ids.y == BOX_MODE_TEXEL {
        // One texel for the whole box (pixels of held item sprites); `ids.w` = faces to skip.
        if (b.ids.w & (1u << face)) != 0u {
            out.clip = vec4(0.0, 0.0, 0.0, 1.0);
        }
        out.layer = b.ids.z;
        if out.layer < arrayLength(&glow_map) {
            out.glow = glow_map[out.layer];
        }
        out.uv = b.uvo.xy;
    } else if b.ids.y == BOX_MODE_FILLED {
        // Filled gizmo box: its alpha rides along as bits.
        out.layer = b.ids.z;
        out.uv = uv;
    } else if b.ids.y == BOX_MODE_FACES || b.ids.y == BOX_MODE_FLAMES {
        var packed = b.ids.z;
        if face >= 2u && face < 4u {
            packed = b.ids.w;
        } else if face >= 4u {
            packed = b.ids2.x;
        }
        out.layer = (packed >> ((face & 1u) * 16u)) & 65535u;
        if out.layer < arrayLength(&glow_map) {
            out.glow = glow_map[out.layer];
        }
        out.uv = uv;
    } else if b.ids.y == BOX_MODE_QUAD {
        // A box is 6 faces to draw; this is one.
        if face != 0u {
            out.clip = vec4(0.0, 0.0, 0.0, 1.0);
        }
        out.mode = BOX_MODE_FACES;
        out.layer = b.ids.z;
        if out.layer < arrayLength(&glow_map) {
            out.glow = glow_map[out.layer];
        }
        out.uv = mix(b.uvo.xy, b.uvo.zw, UVS[c]);
    } else {
        let r = face_rect(face, b.dims.xyz, b.uvo.xy);
        out.uv = (r.xy + uv * r.zw) / b.uvo.zw;
        out.layer = 0u;
    }
    let hurt = b.color.a;
    var tint = b.color.rgb;
    if (b.ids.y == BOX_MODE_FACES && ((b.ids2.z >> face) & 1u) != 0u) || (b.ids.y == BOX_MODE_QUAD && b.ids2.z != 0u) {
        tint *= vec3(f32((b.ids2.w >> 16u) & 255u), f32((b.ids2.w >> 8u) & 255u), f32(b.ids2.w & 255u)) / 255.0;
    }
    out.tint = pow(tint, vec3(2.2));
    var lit = vec3(1.0);
    let world_lit = b.ids.y != BOX_MODE_SOLID && b.ids.y != BOX_MODE_FILLED && b.ids2.y != 0xffffffffu;
    out.graded = select(0u, 1u, world_lit);
    out.sky = select(1.0, unpack_light(b.ids2.y).x, world_lit);
    if world_lit {
        lit = lightmap(unpack_light(b.ids2.y), scene.params.z, scene.light.x, scene.light.y, scene.light.w, scene.environment.x);
    }
    // Gamma-space shading applied in linear space, as in the world shader.
    let shade = select(SHADE[face], b.dims.x, b.ids.y == BOX_MODE_QUAD);
    if world_lit && b.ids.y != BOX_MODE_FLAMES {
        var d = NORMAL[face];
        if b.ids.y == BOX_MODE_QUAD {
            d = vec3(0.0, 0.0, 1.0);
        }
        let n = normalize(vec3(dot(b.rows[0].xyz, d), dot(b.rows[1].xyz, d), dot(b.rows[2].xyz, d)) + vec3(0.0, 1e-6, 0.0));
        let l = unpack_light(b.ids2.y);
        let light = light_on(n, l, sun_on_body(world, n, l.x), 1.0, world, world) * mix(0.8, 1.0, shade);
        out.color = vec4(pow(tint * mix(vec3(1.0), vec3(1.0, 0.4, 0.4), hurt), vec3(2.2)) * light, 1.0);
    } else {
        out.color = vec4(pow(tint * lit * shade * mix(vec3(1.0), vec3(1.0, 0.4, 0.4), hurt), vec3(2.2)), 1.0);
    }
    return out;
}

// The held item: depth squeezed into the near half (reverse Z), in front of the world (nothing the
// player collides with comes closer than ~0.2 blocks, depth 0.3) but still sorted within itself.
@vertex
fn vs_hand(@builtin(vertex_index) vi: u32) -> VOut {
    var out = box_vertex(boxes[vi / 36u], vi % 36u);
    out.clip.z = (0.5 + 0.5 * out.clip.z / out.clip.w) * out.clip.w;
    return out;
}

@vertex
fn vs_hand_item(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    var out = box_vertex(item_box(vi, ii), vi % 36u);
    out.clip.z = (0.5 + 0.5 * out.clip.z / out.clip.w) * out.clip.w;
    return out;
}

fn fog(in: VOut, rgb: vec3<f32>) -> vec3<f32> {
    if in.graded != 0u {
        return display(haze_lit(rgb, in.rel, in.sky));
    }
    let f = fog_amount(in.dist, scene.params.x, scene.params.y);
    return mix(rgb, scene.fog_color.rgb, f);
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    var albedo = vec4(1.0);
    if in.mode == BOX_MODE_SKIN {
        albedo = textureSample(textures[in.tex], nearest_clamp, in.uv);
    } else if in.mode == BOX_MODE_FACES || in.mode == BOX_MODE_FLAMES {
        albedo = textureSample(textures[in.layer], nearest_mipped, in.uv);
    } else if in.mode == BOX_MODE_TEXEL {
        albedo = textureSampleLevel(textures[in.layer], nearest_mipped, in.uv, 0.0);
    }
    if in.mode == BOX_MODE_FLAMES {
        // Flames give their own light.
        if albedo.a < 0.5 {
            discard;
        }
        return vec4(albedo.rgb, 1.0);
    }
    if albedo.a < 0.5 {
        discard;
    }
    var light = in.color.rgb;
    var lit = albedo.rgb;
    if (in.mode == BOX_MODE_FACES || in.mode == BOX_MODE_TEXEL) && in.glow != 0xffffffffu {
        let mask = textureSampleLevel(textures[in.glow], nearest_clamp, in.uv, 0.0);
        light = max(light, in.tint * pow(mask.r, 2.2));
        lit = albedo.rgb * light;
        // On a blood moon the monsters' eyes burn red: their glow masks' green channel (Art's
        // `entity/<skin>_e.png`, red 0 so no glow on other nights), brighter in totality. Deep red that stays red through the night's curve (at 2.4 0.2 0.05 it
        // clipped to orange, the eyes check, 11-05).
        let g = mask.g * blood_moon();
        lit = mix(lit, vec3(1.15, 0.03, 0.01), g);
    } else {
        lit = albedo.rgb * light;
    }
    // In the world, solid (the reflections' mask); on a screen, opaque.
    return vec4(fog(in, lit), select(1.0, SOLID_ALPHA, in.graded != 0u));
}

// How much a crack darkens the face under it (0 none, 1 black).
const CRACK_DARKEN: f32 = 0.6;

@fragment
fn fs_overlay(in: VOut) -> @location(0) vec4<f32> {
    // Block faces mode: the breaking cracks. They darken the block's own face, as vanilla's do
    // (a multiply), rather than being painted over it: the crack pictures are near black at 0.9
    // alpha, and drawn as they are, lit only by the light inside the block itself, each stage
    // stamped a hard black maze on the face that read as glitchy (Ronald, 11-04). Their alpha
    // still says where a crack is; the face shows through at under half its brightness there.
    if in.mode == BOX_MODE_FACES {
        let t = textureSample(textures[in.layer], nearest_mipped, in.uv);
        if t.a < 0.1 {
            discard;
        }
        return vec4(vec3(0.0), t.a * CRACK_DARKEN);
    }
    if in.mode == BOX_MODE_FILLED {
        return vec4(in.color.rgb, bitcast<f32>(in.layer));
    }
    return vec4(in.color.rgb, 1.0);
}
