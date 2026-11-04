// chiacraft's blocks and liquids: the engine's world shader with the pack's light. Plants and leaves
// move in the wind, and water shows the sky with a glint of the sun.

var<private> AO: array<f32, 4> = array<f32, 4>(0.45, 0.64, 0.82, 1.0);
var<private> NORMAL: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
    vec3(-1.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), vec3(0.0, -1.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, -1.0), vec3(0.0, 0.0, 1.0),
);

// The engine's tint kinds come first, in this order (`content.rs`).
const TINT_GRASS: u32 = 1u;
const TINT_FOLIAGE: u32 = 2u;
const TINT_WATER: u32 = 3u;

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    // Textures (bindless indices) of this frame and the next.
    @location(1) @interpolate(flat) layer: u32,
    // The tint, white under an overlay (which takes it).
    @location(2) color: vec3<f32>,
    @location(3) light: vec4<f32>,
    @location(4) dist: f32,
    @location(5) @interpolate(flat) layer2: u32,
    @location(6) @interpolate(flat) blend: f32,
    // Overlay texture + 1 (0: none) and the tint it takes.
    @location(7) @interpolate(flat) overlay: u32,
    @location(8) tint: vec3<f32>,
    // Glow mask texture (LabPBR emission), all ones: none.
    @location(9) @interpolate(flat) glow: u32,
    // Block shader slot of the texture, 0 none, and the block coordinates for it.
    @location(10) @interpolate(flat) slot: u32,
    @location(11) pos: vec3<f32>,
    @location(12) @interpolate(flat) size: f32,
    // Facing, camera-relative position, the corner shading alone.
    @location(13) normal: vec3<f32>,
    @location(14) rel: vec3<f32>,
    @location(15) ao: f32,
    // x tint kind, y 1 for a moving structure's blocks (not traced), z 1 for plants lit from above.
    @location(16) @interpolate(flat) kind: vec4<u32>,
}

fn shader_of(tex: u32) -> u32 {
    if tex >= arrayLength(&tex_shader) {
        return 0u;
    }
    return tex_shader[tex];
}

fn glow_of(tex: u32) -> u32 {
    if tex >= arrayLength(&glow_map) {
        return 0xffffffffu;
    }
    return glow_map[tex];
}

fn unpack_rgb(c: u32) -> vec3<f32> {
    return vec3<f32>(f32((c >> 16u) & 255u), f32((c >> 8u) & 255u), f32(c & 255u)) / 255.0;
}

fn column_tint(x: i32, z: i32, tint: u32) -> u32 {
    let t = i32(g.ids.w);
    let tx = ((x >> 4u) % t + t) % t;
    let tz = ((z >> 4u) % t + t) % t;
    let i = g.ids.z + u32(tz * t + tx) * 256u + u32((z & 15) * 16 + (x & 15));
    return biomes[i * g.extra.z + tint - 1u];
}

fn tint_color(tint: u32, biome: u32, x: i32, z: i32) -> vec3<f32> {
    if tint == 0u {
        return vec3(1.0);
    }
    var sum = vec3(0.0);
    var n = 0.0;
    for (var k = 0; k < 4; k++) {
        let c = column_tint(x - 1 + (k & 1), z - 1 + (k >> 1u), tint);
        if c != 0u {
            sum += unpack_rgb(c);
            n += 1.0;
        }
    }
    if n > 0.0 {
        return sum / n;
    }
    return unpack_rgb(biomes[biome * g.extra.z + tint - 1u]);
}

struct Frame {
    a: u32,
    b: u32,
    blend: f32,
    size: f32,
}

fn texture_frame(name: u32) -> Frame {
    let base = tex_info[name * 2u];
    let info = tex_info[name * 2u + 1u];
    let size = f32(tex_info[frames[0] * 2u + name]);
    if info == 0u {
        return Frame(base, base, 0.0, size);
    }
    let count = info & 65535u;
    let total = f32((info >> 16u) & 32767u);
    let ticks = scene.light.z;
    let t = ticks - floor(ticks / total) * total;
    var acc = 0.0;
    for (var i = 0u; i < count; i++) {
        let e = frames[base + i];
        let d = f32(e >> 16u);
        if t < acc + d || i + 1u == count {
            let next = frames[base + (i + 1u) % count];
            var f = 0.0;
            if (info >> 31u) == 1u {
                f = (t - acc) / d;
            }
            return Frame(e & 65535u, next & 65535u, f, size);
        }
        acc += d;
    }
    return Frame(base, base, 0.0, size);
}

fn on_body(l: vec4<f32>, k: i32) -> vec4<f32> {
    if k == 0 {
        return l;
    }
    let w = unpack_light(bitcast<u32>(offsets[k].w));
    return vec4(min(l.x, w.x), max(l.yzw, w.yzw));
}

fn finish(local: vec3<f32>, origin: vec4<i32>) -> vec4<f32> {
    let pos = vec3<f32>(origin.xyz) * 16.0 + local - g.cam_frac.xyz + offsets[origin.w].xyz;
    return vec4(pos, 1.0);
}

// ---- wind ----

// How far the wind moves what stands at `p` (world blocks) now; a function of the place alone, so
// leaves sharing a corner move together and show no cracks.
// How far the wind moves what sways at `p`: a lean the way it blows, stronger in its gusts, and a
// flutter that quickens with it. Calm air keeps a little life in the leaves.
fn wind(p_world: vec3<f32>) -> vec3<f32> {
    // Far from spawn the sines would get huge arguments and lose their place (foliage jumps on
    // fast-math backends): positions are taken within 4096 blocks, a seam once every 4096.
    let p = vec3(p_world.x - floor(p_world.x / 4096.0) * 4096.0, p_world.y, p_world.z - floor(p_world.z / 4096.0) * 4096.0);
    let t = scene.params.w;
    let w = wind_now();
    let speed = length(w);
    let dir = select(vec2(1.0, 0.0), w / max(speed, 1e-4), speed > 0.01);
    // A fresh breeze (9 m/s) bends things as far as they go, a gale a little further.
    let s = clamp(speed / 9.0, 0.0, 1.6);
    let g = gust(p);
    let q = t * (1.0 + 0.8 * s);
    let a = sin(q * 1.7 + p.x * 0.63 + p.z * 0.41 + p.y * 0.2) * 0.6 + sin(q * 2.9 + p.x * 1.21 - p.z * 0.83) * 0.25;
    let b = sin(q * 1.3 + p.z * 0.71 - p.x * 0.37) * 0.5 + sin(q * 3.4 + p.z * 1.4 + p.x * 0.6) * 0.2;
    let flutter = vec3(a, 0.0, b) * (0.35 + 0.65 * s) * g;
    let lean = vec3(dir.x, 0.0, dir.y) * s * g * 1.3;
    return flutter + lean;
}

@vertex
fn vs_face(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    let f = vi / 4u;
    let w0 = records[f * 4u];
    let w1 = records[f * 4u + 1u];
    let w2 = records[f * 4u + 2u];
    let w3 = records[f * 4u + 3u];
    let a = face_ao(w0);
    let flip = a.x + a.z < a.y + a.w;
    var c = vi % 4u;
    if flip {
        c = (c + 1u) % 4u;
    }
    let dir = face_dir(w0);
    let local = face_pos(w0) + CORNERS[dir * 4u + c];
    var pos = finish(local, origins[ii]);
    let o = origins[ii] + g.cam_cube;
    let world = vec3<f32>(o.xyz * 16) + local;
    let tint_kind = (w0 >> FACE_TINT_SHIFT) & TINT_MASK;
    // Leaves sway a little.
    if tint_kind == TINT_FOLIAGE && origins[ii].w == 0 {
        pos = vec4(pos.xyz + wind(world) * 0.035, 1.0);
    }
    var out: VOut;
    out.clip = scene.view_proj * pos;
    out.dist = length(pos.xyz);
    out.rel = pos.xyz;
    out.uv = UVS[c];
    let tex_id = w1 & FACE_TEX_MASK;
    let fr = texture_frame(tex_id);
    out.size = fr.size;
    out.layer = fr.a;
    out.layer2 = fr.b;
    out.blend = fr.blend;
    out.glow = glow_of(fr.a);
    out.light = on_body(corner_light(w2, w3, c), origins[ii].w);
    out.slot = shader_of(tex_id);
    out.pos = world;
    out.normal = NORMAL[dir];
    out.ao = AO[a[c]];
    out.kind = vec4(tint_kind, select(0u, 1u, origins[ii].w != 0), 0u, 0u);
    var tint = vec3(1.0);
    if tint_kind != 0u {
        tint = tint_color(tint_kind, w1 >> FACE_BIOME_SHIFT, o.x * 16 + i32(local.x), o.z * 16 + i32(local.z));
    }
    out.tint = tint;
    let ov = frames[g.extra.x + tex_id];
    if ov != 0u {
        out.overlay = texture_frame(ov - 1u).a + 1u;
        out.color = vec3(1.0);
    } else {
        out.overlay = 0u;
        // The tint is a gamma-space color; the pack lights in linear light.
        out.color = pow(tint, vec3(2.2));
    }
    return out;
}

@vertex
fn vs_quad(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    let q = vi / 4u;
    let c = vi % 4u;
    let base = q * 8u;
    let p0 = quad_pos(records[base]);
    let p1 = quad_pos(records[base + 1u]);
    let p2 = quad_pos(records[base + 2u]);
    let p3 = quad_pos(records[base + 3u]);
    let local = quad_pos(records[base + c]);
    let w4 = records[base + 4u];
    let uvw = records[base + 5u];
    let uv = quad_uv_fine(uvw, records[base], records[base + 1u], records[base + 2u], records[base + 3u]);
    var uvs = array<vec2<f32>, 4>(vec2(uv.x, uv.w), vec2(uv.z, uv.w), vec2(uv.z, uv.y), vec2(uv.x, uv.y));
    // A quad squashed to nothing has no normal: up then, never NaN.
    let n = normalize(cross(p1 - p0, p2 - p1) + vec3(0.0, 1e-6, 0.0));
    let flat_ = ((uvw >> QUAD_TINT_SHIFT) & QUAD_FLAT) != 0u;
    let tint_kind = (uvw >> QUAD_TINT_SHIFT) & TINT_MASK;
    var pos = finish(local, origins[ii]);
    let o = origins[ii] + g.cam_cube;
    let world = vec3<f32>(o.xyz * 16) + local;
    // Grass and flowers bend at the top and stay rooted.
    if (tint_kind == TINT_GRASS || tint_kind == TINT_FOLIAGE || flat_) && tint_kind != TINT_WATER && origins[ii].w == 0 {
        let lo = min(min(p0.y, p1.y), min(p2.y, p3.y));
        let hi = max(max(p0.y, p1.y), max(p2.y, p3.y));
        let up = clamp((local.y - lo) / max(hi - lo, 0.001), 0.0, 1.0) * step(0.2, hi - lo);
        pos = vec4(pos.xyz + wind(floor(world) + vec3(0.5)) * 0.11 * up, 1.0);
    }
    var out: VOut;
    out.clip = scene.view_proj * pos;
    out.dist = length(pos.xyz);
    out.rel = pos.xyz;
    out.uv = uvs[c];
    let tex_id = w4 & FACE_TEX_MASK;
    let fr = texture_frame(tex_id);
    out.size = fr.size;
    out.layer = fr.a;
    out.layer2 = fr.b;
    out.blend = fr.blend;
    out.glow = glow_of(fr.a);
    out.light = on_body(corner_light(records[base + 6u], records[base + 7u], c), origins[ii].w);
    out.slot = shader_of(tex_id);
    out.pos = world;
    // Crossed plants are lit as if they faced up, like the ground they grow from.
    out.normal = select(n, vec3(0.0, 1.0, 0.0), flat_);
    out.ao = 1.0;
    out.kind = vec4(tint_kind, select(0u, 1u, origins[ii].w != 0), select(0u, 1u, flat_), 0u);
    var tint = vec3(1.0);
    if tint_kind != 0u {
        tint = tint_color(tint_kind, w4 >> FACE_BIOME_SHIFT, o.x * 16 + i32(round(local.x)), o.z * 16 + i32(round(local.z)));
    }
    out.tint = tint;
    let ov = frames[g.extra.x + tex_id];
    if ov != 0u {
        out.overlay = texture_frame(ov - 1u).a + 1u;
        out.color = vec3(1.0);
    } else {
        out.overlay = 0u;
        // The tint is a gamma-space color; the pack lights in linear light.
        out.color = pow(tint, vec3(2.2));
    }
    return out;
}

// ---- sun shadows ----

// Where the sun reaches: the engine's shadow map near the camera (trees, hills and houses cast
// shadows that move with the sun), the sky light alone beyond it, and never deep under cover. This
// file must not name the engine's block lookup: its name anywhere in it makes the engine upload
// the blocks around the camera every frame.
fn sun_shadow(in: VOut, n: vec3<f32>) -> f32 {
    let sky = smoothstep(0.75, 1.0, in.light.x);
    let mapped = shadow_at(in.rel, n);
    return mix(sky, min(mapped.x, smoothstep(0.3, 0.8, in.light.x)), mapped.y) * cloud_shadow(in.rel);
}

// The shadow pass (`sun_shadows`): only depth, without the see-through texels of leaves and
// plants, so they cast dappled shade.
@fragment
fn fs_shadow(in: VOut) {
    // A plant's flat quad nearly edge-on to the light casts nothing: as a sliver it would draw a
    // dotted line of shadow across the ground. Its true facing, from how its surface runs.
    let facing = normalize(cross(dpdx(in.rel), dpdy(in.rel)) + vec3(0.0, 1e-9, 0.0));
    let a = textureSampleLevel(textures[in.layer], nearest_clamp, in.uv, 0.0).a;
    if a < 0.5 || (in.kind.z != 0u && abs(dot(facing, light_dir())) < 0.3) {
        discard;
    }
}

// ---- shading ----

fn surface_light(in: VOut, n: vec3<f32>) -> vec3<f32> {
    let shadow = sun_shadow(in, n);
    var light = in.color * light_on(n, in.light, shadow, in.ao, in.pos);
    // Leaves and plants let some sun through when it is behind them.
    if in.kind.x == TINT_FOLIAGE || in.kind.z != 0u {
        let back = max(-dot(normalize(in.rel), light_dir()), 0.0);
        light += in.color * palette().sun * light_strength() * pow(back, 3.0) * 0.35 * shadow * smoothstep(0.5, 0.95, in.light.x);
    }
    return light;
}

fn shade(in: VOut, albedo_in: vec4<f32>) -> vec4<f32> {
    let n = normalize(in.normal);
    var albedo = albedo_in;
    // Snow lying on the tops of blocks and leaves (not on plants, which it buries).
    if in.kind.z == 0u {
        let snow = snow_on(n, in.pos, in.light.x);
        albedo = vec4(mix(albedo.rgb, vec3(0.86, 0.9, 0.95), snow), albedo.a);
    }
    var light = surface_light(in, n);
    if in.glow != 0xffffffffu {
        let e = textureSampleLevel(textures[in.glow], nearest_clamp, in.uv, 0.0).r;
        light = max(light, select(pow(in.tint, vec3(2.2)), vec3(1.0), in.overlay != 0u) * e * 1.7);
    }
    var lit: vec4<f32>;
    if in.slot == 0u {
        lit = vec4(albedo.rgb * light, albedo.a);
    } else {
        lit = block_shade(in.slot, BlockPixel(in.uv, albedo, light, in.pos, scene.params.w));
    }
    // Out in the rain, the ground goes dark and wet, with a little of the sky on it (not plants).
    let wet = rain_amount() * smoothstep(0.85, 1.0, in.light.x) * smoothstep(0.3, 0.9, n.y) * select(1.0, 0.0, in.kind.z != 0u || in.kind.x == TINT_FOLIAGE);
    if wet > 0.0 {
        let v = -normalize(in.rel);
        let sheen = pow(1.0 - max(dot(n, v), 0.0), 5.0) * 0.12;
        lit = vec4(lit.rgb * mix(1.0, 0.7, wet) + sky_color(reflect(-v, n)) * sheen * wet, lit.a);
    }
    return vec4(display(haze_lit(lit.rgb, in.rel, in.light.x)), lit.a);
}

// ---- water ----

// The surface as heights (blocks): layers of smooth noise, each stretched into long crests and
// turned another 137.5° (the golden angle, so their grids never line up into a pattern), each
// sliding its own way at about 1.6 blocks a second, as Bliss builds its water (the idea, our code).
// No regular swells: two crossed sines made the repeating blobs Ronald found strange. Ripples a few
// blocks long, finer ones near a block; `detail` fades the finest out with distance (far off they
// only make moiré). A little higher in a strong wind.
fn wave_layer(p: vec2<f32>, t: f32, k: f32, cell: vec2<f32>, amp: f32) -> f32 {
    let a = k * 2.39996;
    let r = mat2x2<f32>(cos(a), sin(a), -sin(a), cos(a));
    let q = r * p;
    let along = vec2(1.6 * t, 0.0);
    let n = noise3(vec3((q - along) / cell, t * 0.08 + k * 17.0));
    return (n - 0.5) * amp;
}

fn wave_height(p: vec2<f32>, t: f32, detail: f32) -> f32 {
    let w = wind_now();
    let speed = length(w);
    // Turned to the wind, so the crests lie across it.
    let dir = select(vec2(0.8, 0.6), w / max(speed, 1e-4), speed > 0.3);
    let q = mat2x2<f32>(dir.x, -dir.y, dir.y, dir.x) * p;
    var h = wave_layer(q, t, 0.0, vec2(6.0, 1.6), 0.10);
    h += wave_layer(q, t, 1.0, vec2(1.6, 6.0), 0.08);
    h += wave_layer(q, t, 2.0, vec2(3.2, 3.2), 0.07);
    h += wave_layer(q, t * 1.3, 3.0, vec2(0.8, 0.8), 0.022) * detail;
    return h * (0.7 + 0.3 * smoothstep(1.0, 8.0, speed));
}

// The surface's normal at `pos` (world) seen from `dist` blocks, per pixel. The slope is taken over
// a step that widens with distance, which smooths the far ripples away instead of letting them
// shimmer (Bliss's way), and flattens them a little more at a glancing look.
fn wave_normal(pos: vec3<f32>, dist: f32, glance: f32) -> vec3<f32> {
    let t = scene.params.w;
    let p = pos.xz;
    let detail = 1.0 - smoothstep(16.0, 56.0, dist);
    let e = 0.12 + min(dist / 200.0, 3.0);
    let h = wave_height(p, t, detail);
    let hx = wave_height(p + vec2(e, 0.0), t, detail);
    let hz = wave_height(p + vec2(0.0, e), t, detail);
    let k = mix(1.0, 0.5, smoothstep(24.0, 160.0, dist)) * mix(0.4, 1.0, smoothstep(0.05, 0.35, glance));
    return normalize(vec3(-(hx - h) / e * k, 1.0, -(hz - h) / e * k));
}

// Rain on water: each drop's ring spreads and fades in its own cell, at its own time. The slope
// it gives the surface, on the texture's pixel grid.
fn rain_rings(p: vec2<f32>) -> vec2<f32> {
    let q = floor(p * 16.0) / 16.0 / 0.8;
    let base = floor(q);
    var slope = vec2(0.0);
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let cell = vec2<i32>(base) + vec2(i, j);
            let h = hash3(vec3(cell.x, 7, cell.y));
            let k = hash3(vec3(cell.x, 13, cell.y));
            let t = fract(scene.params.w * 0.9 + h * 7.31);
            let v = q - (vec2<f32>(cell) + vec2(0.2 + 0.6 * h, 0.2 + 0.6 * k));
            let d = length(v);
            let x = d - t * 1.1;
            let wave = sin(x * 26.0) * exp(-x * x * 70.0) * (1.0 - t) * (1.0 - t);
            slope += v / max(d, 1e-3) * wave;
        }
    }
    return slope * 0.3;
}

// How deep the water is under the surface at `pos` (world): blocks straight down to the first solid
// one, at most `WATER_PROBE` looked at (a few lookups per water pixel, nothing like a traced
// shadow). Deeper, or out where the client keeps no blocks (`solid_at` is false): deep.
const WATER_PROBE: i32 = 8;

fn water_depth(pos: vec3<f32>) -> f32 {
    let c = vec3<i32>(floor(pos - vec3(0.0, 0.05, 0.0)));
    for (var i = 0; i < WATER_PROBE; i++) {
        let b = c - vec3(0, i, 0);
        if solid_at(b) {
            return max(pos.y - f32(b.y + 1), 0.0);
        }
    }
    return f32(WATER_PROBE) + 2.0;
}

// How far `pos` on the surface is from land beside its block (blocks; 1 with none).
fn shore_distance(pos: vec3<f32>) -> f32 {
    let c = vec3<i32>(floor(pos - vec3(0.0, 0.05, 0.0)));
    let f = pos.xz - floor(pos.xz);
    var d = 1.0;
    if solid_at(c + vec3(-1, 0, 0)) {
        d = min(d, f.x);
    }
    if solid_at(c + vec3(1, 0, 0)) {
        d = min(d, 1.0 - f.x);
    }
    if solid_at(c + vec3(0, 0, -1)) {
        d = min(d, f.y);
    }
    if solid_at(c + vec3(0, 0, 1)) {
        d = min(d, 1.0 - f.y);
    }
    return d;
}

// The world copied before the water was drawn (`scene.behind`): its color (as it reached the screen)
// and where it is, camera relative, at screen position `uv`; w -1 where nothing was drawn.
fn behind_color(uv: vec2<f32>) -> vec3<f32> {
    return textureSampleLevel(textures[u32(scene.behind.x)], linear_clamp, uv, 0.0).rgb;
}

fn behind_at(uv: vec2<f32>, size: vec2<f32>) -> vec4<f32> {
    let px = clamp(vec2<i32>(uv * size), vec2(0), vec2<i32>(size) - vec2(1));
    let d = textureLoad(textures[u32(scene.behind.y)], px, 0).r;
    if d <= 0.0 {
        return vec4(0.0, 0.0, 0.0, -1.0);
    }
    let p = scene.inv_view_proj * vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
    return vec4(p.xyz / p.w, 1.0);
}

// Light playing on the bottom through the waves (`p` world xz): bright crossing lines that drift.
fn caustics(p: vec2<f32>) -> f32 {
    let t = scene.params.w;
    let q = p * 0.8;
    let a = 1.0 - abs(noise3(vec3(q.x + t * 0.25, t * 0.4, q.y)) * 2.0 - 1.0);
    let b = 1.0 - abs(noise3(vec3(q.x * 1.7 + 5.0, t * 0.55, q.y * 1.7 - t * 0.2)) * 2.0 - 1.0);
    return pow(a, 6.0) + 0.5 * pow(b, 8.0);
}

// What is seen through the surface at this pixel, as it reaches the screen: the bottom (or what
// stands in the water) bent by the waves (glass-like refraction, index 1.33), its colors taken away
// with depth (red first), light from the scattering water in their place, caustics on the bottom
// in the sun. w: how far the view goes through the water (blocks).
fn water_under(in: VOut, n: vec3<f32>, v: vec3<f32>, scatter: vec3<f32>, shadow: f32) -> vec4<f32> {
    let size = vec2<f32>(textureDimensions(textures[u32(scene.behind.x)]));
    let uv = in.clip.xy / size;
    let straight = behind_at(uv, size);
    let deep = display(scatter);
    if straight.w < 0.0 {
        return vec4(deep, 8.0);
    }
    var through = length(straight.xyz - in.rel);
    // Bent at the surface; where that lands on something standing above the water, unbent.
    let bent = refract(-v, n, 1.0 / 1.33);
    let aim = scene.view_proj * vec4(in.rel + bent * min(through, 6.0), 1.0);
    var at = uv;
    if aim.w > 0.0 {
        let suv = vec2(aim.x / aim.w * 0.5 + 0.5, 0.5 - aim.y / aim.w * 0.5);
        let b = behind_at(suv, size);
        if b.w > 0.0 && length(b.xyz) > length(in.rel) && all(suv > vec2(0.0)) && all(suv < vec2(1.0)) {
            at = suv;
            through = length(b.xyz - in.rel);
        }
    }
    let bottom = behind_color(at);
    let gone = exp(-vec3(0.42, 0.11, 0.07) * through);
    var seen = bottom * gone + deep * (vec3(1.0) - gone);
    // The sun's light focused by the waves onto the bottom, less as it goes deeper.
    let floor_world = in.pos + (straight.xyz - in.rel);
    let lightning = caustics(floor_world.xz) * light_strength() * shadow * exp(-through * 0.25) * smoothstep(0.1, 0.8, through);
    seen += bottom * gone * lightning * 0.9;
    return vec4(seen, through);
}

// Water seen from above: dark and clear looking down close by, the bottom showing through the
// shallows (lighter and greener there), blue and opaque where deep, a mirror of the sky toward the
// horizon (Fresnel), a path of sun glitter toward a low sun and a soft line of foam at the shore.
// The screen's reflections (`post/reflect.wgsl`) then mirror the hills into it.
fn water(in: VOut, albedo: vec4<f32>, see_through: bool) -> vec4<f32> {
    let v = -normalize(in.rel);
    let top = in.normal.y > 0.5;
    // The surface from below, the eye in the water: not the texture's navy but the water's own
    // teal, bright straight up where the sky comes through (Snell's window, about 48° about the
    // vertical) and darker toward the sides, where it mirrors the water below. Tester 2's shots
    // had it a flat dark navy looking up while the water around was teal.
    // (Either face: from below it is usually the downward one, its normal pointing at the eye.)
    if in_water() && abs(in.normal.y) > 0.5 && v.y < 0.0 {
        let window = smoothstep(0.55, 0.85, -v.y);
        let sky = mix(0.6, 2.4, window) * mix(0.15, 1.0, brightness_curve(in.light.x));
        return vec4(display(haze_lit(underwater_color() * sky, in.rel, in.light.x)), 1.0);
    }
    if (top && v.y <= 0.0) || in_liquid() {
        return shade(in, albedo);
    }
    var n = normalize(in.normal);
    var depth = 3.0;
    var shore = 1.0;
    if top {
        n = wave_normal(in.pos, in.dist, v.y);
        let rain = rain_amount() * smoothstep(0.85, 1.0, in.light.x);
        if rain > 0.0 {
            let s = rain_rings(in.pos.xz) * rain;
            n = normalize(n + vec3(-s.x, 0.0, -s.y));
        }
        depth = water_depth(in.pos);
        shore = shore_distance(in.pos);
    }
    let ndv = max(dot(n, v), 0.02);
    // A near mirror at an ordinary look, as in the reference pictures: most of what it shows is
    // what it reflects (the screen's reflections, `post/reflect.wgsl`, use the same).
    let fresnel = 0.08 + 0.92 * pow(1.0 - ndv, 2.5);
    let shadow = sun_shadow(in, vec3(0.0, 1.0, 0.0));
    let lit = light_on(vec3(0.0, 1.0, 0.0), in.light, shadow, 1.0, in.pos);
    // What the light in the water sends back up: a muted teal-green, light in the shallows and dark
    // further down so that what it mirrors carries, in a little of the biome's own tint (a swamp's
    // brown-green; the plain blue of most biomes' water is half taken out), with a little of the
    // texture's pattern near.
    let raw = in.color / max(max(in.color.r, max(in.color.g, in.color.b)), 1e-3);
    let soft = mix(vec3(luma(raw)), raw, 0.5);
    let tint = soft / max(luma(soft), 0.2) * 0.8;
    let pattern = mix(1.0, clamp(luma(albedo.rgb) / 0.55, 0.8, 1.2), 1.0 - smoothstep(8.0, 24.0, in.dist));
    // The biome's tint counts for more where it is deep; the shallows keep their own color.
    let shallow = vec3(0.07, 0.27, 0.20);
    // Deep water a murky teal, not near black: the light scattered back from inside it (Bliss's
    // "dirt" scatter) keeps it lit looking down (Tester 2's COL5 shots: deep water at 0.35 mean
    // brightness, a dark navy).
    let deep = vec3(0.022, 0.072, 0.066);
    let deepness = 1.0 - exp(-0.35 * depth);
    let scatter = mix(shallow, deep, deepness) * mix(vec3(1.0), tint, mix(0.15, 0.35, deepness)) * pattern * lit;
    // How much of the bottom shows: the way through the water to it, longer at a slant; never all
    // of it, so even the shallowest water tints what lies under it.
    let through = exp(-0.28 * depth / max(v.y, 0.2)) * 0.82;
    let r = reflect(-v, n);
    let refl = sky_color(normalize(vec3(r.x, max(r.y, 0.03), r.z))) * mix(0.15, 1.0, brightness_curve(in.light.x));
    // The sun's glitter: a narrow highlight (GGX) on the waves, so it lies in a path toward the sun.
    let l = light_dir();
    // Never a zero vector (light exactly behind the eye): its normalize would be NaN.
    let hv = normalize(l + v + vec3(0.0, 1e-4, 0.0));
    let nh = max(dot(n, hv), 0.0);
    let a2 = 0.0016;
    let dd = nh * nh * (a2 - 1.0) + 1.0;
    let spec = a2 / (3.14159 * dd * dd) * (0.02 + 0.98 * pow(1.0 - max(dot(v, hv), 0.0), 5.0)) * 0.25;
    // Toward a low sun the path stays orange: capped at 1.5 then (it was 8, and every channel went
    // white through the curve: Tester 2's golden-hour shots), so only its red nears white.
    // Sparkle: a much sharper highlight on the fine ripples close by, flickering as they move.
    let a2s = 0.00012;
    let ds = nh * nh * (a2s - 1.0) + 1.0;
    let near = 1.0 - smoothstep(12.0, 40.0, in.dist);
    let sparkle = a2s / (3.14159 * ds * ds) * 0.02 * near;
    let glint = palette().sun * light_strength() * min(spec + sparkle, mix(40.0, 1.5, golden())) * shadow * smoothstep(0.0, 0.1, l.y + 0.05);
    // Foam where it laps the land, in a wobbling line on the pixel grid.
    let px = floor(in.pos.xz * 16.0) / 16.0;
    let wobble = noise3(vec3(px.x * 2.0, scene.params.w * 0.6, px.y * 2.0));
    let foam = (1.0 - smoothstep(0.06, 0.22 + 0.16 * wobble, shore)) * 0.7;
    // With the world's copy (`scene.behind`, drawn in the see-through pass): what lies under the
    // surface itself, bent by the waves, tinted by the true depth, with caustics, and foam wherever
    // the water meets something. Opaque from here: the water does its own mixing.
    if see_through && top && scene.behind.z > 0.5 {
        let under = water_under(in, n, v, scatter, shadow);
        let foam_all = max(foam, (1.0 - smoothstep(0.05, 0.3 + 0.1 * wobble, under.w)) * 0.6);
        let surface = refl * fresnel + glint + (lit * 0.9 + vec3(0.03)) * foam_all;
        let below = under.rgb * (1.0 - fresnel) * (1.0 - foam_all);
        return vec4(below + display(haze_lit(surface, in.rel, in.light.x)), 1.0);
    }
    let lost = (1.0 - fresnel) * through;
    let a = clamp(max(1.0 - lost, max(luma(glint), foam)), 0.02, 1.0);
    let c = (refl * fresnel + scatter * (1.0 - fresnel) * (1.0 - through) + glint + (lit * 0.9 + vec3(0.03)) * foam) / a;
    return vec4(display(haze_lit(c, in.rel, in.light.x)), a);
}

// ---- texture sampling (the engine's) ----

fn nearest_uv(size: f32, uv: vec2<f32>, du: vec2<f32>, dv: vec2<f32>) -> vec2<f32> {
    let inv_s = 1.0 / size;
    let t = uv * size;
    let r = round(t);
    let inv_pp = inverseSqrt(max(du * du + dv * dv, vec2(1e-10))) * inv_s;
    let off = clamp((t - r) * inv_pp + 0.5, vec2(0.0), vec2(1.0));
    return (r - 0.5 + off) * inv_s;
}

fn sample(in: VOut, nuv: vec2<f32>, du: vec2<f32>, dv: vec2<f32>) -> vec4<f32> {
    let a = textureSampleGrad(textures[in.layer], linear_mipped, nuv, du, dv);
    if in.blend <= 0.0 {
        return a;
    }
    let b = textureSampleGrad(textures[in.layer2], linear_mipped, nuv, du, dv);
    return mix(a, b, in.blend);
}

fn with_overlay(in: VOut, albedo: vec4<f32>, nuv: vec2<f32>, du: vec2<f32>, dv: vec2<f32>) -> vec4<f32> {
    if in.overlay == 0u {
        return albedo;
    }
    let ov = textureSampleGrad(textures[in.overlay - 1u], linear_mipped, nuv, du, dv);
    return vec4(mix(albedo.rgb, ov.rgb * pow(in.tint, vec3(2.2)), ov.a), albedo.a);
}

@fragment
fn fs_opaque(in: VOut) -> @location(0) vec4<f32> {
    let du = dpdx(in.uv);
    let dv = dpdy(in.uv);
    let nuv = nearest_uv(in.size, in.uv, du, dv);
    let albedo = with_overlay(in, sample(in, nuv, du, dv), nuv, du, dv);
    return vec4(shade(in, albedo).rgb, SOLID_ALPHA);
}

fn sample_cutout(in: VOut, nuv: vec2<f32>, du: vec2<f32>, dv: vec2<f32>) -> vec4<f32> {
    let size = in.size;
    let lod = max(log2(max(min(length(du * size), length(dv * size)), 1e-6)), 0.0);
    let a = textureSampleLevel(textures[in.layer], linear_mipped, nuv, lod);
    if in.blend <= 0.0 {
        return a;
    }
    let b = textureSampleLevel(textures[in.layer2], linear_mipped, nuv, lod);
    return mix(a, b, in.blend);
}

@fragment
fn fs_cutout(in: VOut) -> @location(0) vec4<f32> {
    let du = dpdx(in.uv);
    let dv = dpdy(in.uv);
    let nuv = nearest_uv(in.size, in.uv, du, dv);
    let albedo = with_overlay(in, sample_cutout(in, nuv, du, dv), nuv, du, dv);
    if albedo.a <= 0.1 {
        discard;
    }
    return vec4(shade(in, albedo).rgb, SOLID_ALPHA);
}

@fragment
fn fs_translucent(in: VOut) -> @location(0) vec4<f32> {
    let du = dpdx(in.uv);
    let dv = dpdy(in.uv);
    let nuv = nearest_uv(in.size, in.uv, du, dv);
    let albedo = with_overlay(in, sample(in, nuv, du, dv), nuv, du, dv);
    if in.kind.x == TINT_WATER {
        return water(in, albedo, true);
    }
    return shade(in, albedo);
}

// Distant terrain (`client::lod`): faces of boxes in the average color of their texture, under
// open sky.
@vertex
fn vs_lod(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    let f = vi / 4u;
    let c = vi % 4u;
    let w0 = records[f * 4u];
    let w1 = records[f * 4u + 1u];
    // A box's face, or a far tile's smooth cell (`lod_corner`, in the prelude).
    let corner = lod_corner(f, c, finish(vec3(0.0), origins[ii]).xyz);
    let dir = corner.dir;
    let local = corner.local;
    let pos = finish(local, origins[ii]);
    var out: VOut;
    out.clip = scene.view_proj * pos;
    out.dist = length(pos.xyz);
    out.rel = pos.xyz;
    out.uv = pos.xz;
    let fr = texture_frame(w1 & FACE_TEX_MASK);
    out.size = fr.size;
    out.layer = fr.a;
    out.layer2 = fr.b;
    out.blend = fr.blend;
    out.glow = 0xffffffffu;
    out.slot = 0u;
    out.overlay = 0u;
    // A little of the near blocks' corner shading: their slopes' sides darker than their tops.
    out.light = vec4(1.0, 0.0, 0.0, 0.0);
    let o = origins[ii] + g.cam_cube;
    out.pos = vec3<f32>(o.xyz * 16) + local;
    out.normal = corner.normal;
    out.ao = mix(0.85, 0.95, clamp(corner.normal.y, 0.0, 1.0));
    // Not traced: it is past the blocks the client keeps. Its tint kind, so far water is drawn as
    // water (and far leaves as leaves).
    let kind = lod_tint(w0);
    out.kind = vec4(kind, 1u, 0u, 0u);
    var tint = vec3(1.0);
    if kind != 0u {
        tint = unpack_rgb(biomes[corner.biome * g.extra.z + kind - 1u]);
    }
    out.tint = tint;
    out.color = pow(tint, vec3(2.2));
    return out;
}

// Whether real cubes show this distant tile pixel's column (`Lod::mask`): then they stand there,
// and the tile is left out.
fn lod_hidden(in: VOut) -> bool {
    let half = i32(g.extra.w);
    let col = vec2<i32>(floor(in.pos.xz / 16.0));
    let d = col - g.cam_cube.xz;
    if abs(d.x) <= half && abs(d.y) <= half {
        let w = 2 * half + 1;
        let i = u32(((col.y % w) + w) % w * w + ((col.x % w) + w) % w);
        return ((lod_mask[i / 32u] >> (i % 32u)) & 1u) != 0u;
    }
    return false;
}

// Distant terrain into the far shadow map: hills and canopies past the render distance cast as the
// near blocks do, so far land has shade at a low sun and does not lie flat. Where cubes stand they
// cast instead, and the water's surface does not.
@fragment
fn fs_lod_shadow(in: VOut) {
    if lod_hidden(in) || in.kind.x == TINT_WATER {
        discard;
    }
}

@fragment
fn fs_lod(in: VOut) -> @location(0) vec4<f32> {
    if lod_hidden(in) {
        discard;
    }
    var albedo = textureSampleLevel(textures[in.layer], linear_mipped, vec2(0.5), 16.0);
    if in.blend > 0.0 {
        albedo = mix(albedo, textureSampleLevel(textures[in.layer2], linear_mipped, vec2(0.5), 16.0), in.blend);
    }
    // Far water is the same water as near: deep there (no blocks are kept that far), with its
    // Fresnel, the sky and the sun's path. Otherwise it would be a flat grey-blue lit like land, a
    // seam at the edge of the render distance. It is left out of the screen's reflections (solid in
    // the mask): looking out to sea, their march over every far-water pixel cost too much
    // (reports/performance.md).
    if in.kind.x == TINT_WATER {
        let w = water(in, albedo, false);
        return vec4(w.rgb, SOLID_ALPHA);
    }
    return vec4(shade(in, albedo).rgb, SOLID_ALPHA);
}
