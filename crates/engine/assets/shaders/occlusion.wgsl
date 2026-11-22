// What cubes get drawn is decided here. `table` has every cube of the world with something opaque
// or cutout to draw; `cull` writes the draws of those in view and not behind what the last frame
// drew. `depths` holds that frame's depth and under it ever smaller copies, each texel the farthest
// of those it covers. `test` does the same for the draws the CPU made, by zeroing their instances.
// The prelude (`gpu_layout::occlusion_prelude`) declares the bindings and `args`.

// The depth buffer has 1 at the near plane and 0 at infinity: farther is less.

// Levels of the pyramid under the depth, each texel the farthest of the 2 x 2 under it (the edge
// texel repeated where a size is odd). `depths` holds levels 1 and on (half size first): level 0
// is the depth texture itself, read here and never copied. A dispatch makes up to `args.count`
// levels from one (`args.size`): the first from `depths` at `args.src`, or from the depth texture
// `textures[args.pad - 1]` when `pad` isn't 0, written from `args.dst` on, one after the other;
// each next level from the last in workgroup memory, a 32 x 32 tile halving each time.
var<workgroup> tile: array<f32, 1024>;

fn source(x: u32, y: u32, s: vec2<u32>) -> f32 {
    if args.pad != 0u {
        return textureLoad(textures[args.pad - 1u], vec2<i32>(i32(x), i32(y)), 0).r;
    }
    return depths[args.src + y * s.x + x];
}

@compute @workgroup_size(32, 32)
fn down(@builtin(global_invocation_id) id: vec3<u32>, @builtin(local_invocation_id) local: vec3<u32>, @builtin(workgroup_id) group: vec3<u32>) {
    var s = args.size;
    var d = (s + vec2<u32>(1u)) / 2u;
    // Out of the level is nearest of all, which a farthest-of never picks: as the edge repeated.
    var v = 1.0;
    if id.x < d.x && id.y < d.y {
        let lo = id.xy * 2u;
        let hi = min(lo + vec2<u32>(1u), s - vec2<u32>(1u));
        v = min(min(source(lo.x, lo.y, s), source(hi.x, lo.y, s)), min(source(lo.x, hi.y, s), source(hi.x, hi.y, s)));
        depths[args.dst + id.y * d.x + id.x] = v;
    }
    tile[local.y * 32u + local.x] = v;
    var dst = args.dst + d.x * d.y;
    var n = 32u;
    var origin = group.xy * 32u;
    s = d;
    for (var l = 1u; l < args.count; l++) {
        workgroupBarrier();
        d = (s + vec2<u32>(1u)) / 2u;
        n = n / 2u;
        origin = origin / 2u;
        let inside = local.x < n && local.y < n;
        var r = 1.0;
        if inside {
            let at = local.y * 64u + local.x * 2u;
            r = min(min(tile[at], tile[at + 1u]), min(tile[at + 32u], tile[at + 33u]));
        }
        workgroupBarrier();
        if inside {
            tile[local.y * 32u + local.x] = r;
            let g = origin + local.xy;
            if g.x < d.x && g.y < d.y {
                depths[dst + g.y * d.x + g.x] = r;
            }
        }
        dst += d.x * d.y;
        s = d;
    }
}

// Where `level` (1 and on) starts in `depths`, and how big it is.
fn level_of(level: u32) -> vec3<u32> {
    var s = (view.size + vec2<u32>(1u)) / 2u;
    var at = 0u;
    for (var l = 1u; l < level; l++) {
        at += s.x * s.y;
        s = (s + vec2<u32>(1u)) / 2u;
    }
    return vec3<u32>(at, s);
}

// All of the box from `lo` to `hi`, relative to this frame's camera, behind the last frame's depth.
fn hidden(cube_lo: vec3<f32>, cube_hi: vec3<f32>) -> bool {
    if view.hiz == 0u {
        return false;
    }
    let grow = vec3<f32>(view.moved.w);
    let lo = cube_lo + view.moved.xyz - grow;
    let hi = cube_hi + view.moved.xyz + grow;
    var small = vec2<f32>(1e9);
    var large = vec2<f32>(-1e9);
    var nearest = 0.0;
    for (var c = 0u; c < 8u; c++) {
        let p = vec3<f32>(
            select(lo.x, hi.x, (c & 1u) != 0u),
            select(lo.y, hi.y, (c & 2u) != 0u),
            select(lo.z, hi.z, (c & 4u) != 0u),
        );
        let clip = view.prev * vec4<f32>(p, 1.0);
        // Reaches behind that camera: there is nothing to go by.
        if clip.w < 0.05 {
            return false;
        }
        let ndc = clip.xyz / clip.w;
        small = min(small, ndc.xy);
        large = max(large, ndc.xy);
        nearest = max(nearest, ndc.z);
    }
    // Any of it off that frame's screen: what is there wasn't seen.
    if small.x < -1.0 || small.y < -1.0 || large.x > 1.0 || large.y > 1.0 {
        return false;
    }
    let screen = vec2<f32>(view.size);
    let a = vec2<f32>(small.x + 1.0, 1.0 - large.y) * 0.5 * screen;
    let b = vec2<f32>(large.x + 1.0, 1.0 - small.y) * 0.5 * screen;
    // Level 0, the depth itself, isn't kept: a box under 2 texels goes by level 1, a little farther.
    let level = clamp(u32(ceil(log2(max(max(b.x - a.x, b.y - a.y), 1.0)))), 1u, max(view.levels - 1u, 1u));
    let info = level_of(level);
    let top = vec2<u32>(screen) - vec2<u32>(1u);
    let from_ = min(vec2<u32>(a), top) >> vec2<u32>(level);
    let to = min(vec2<u32>(b), top) >> vec2<u32>(level);
    var far = 1.0;
    for (var y = from_.y; y <= to.y; y++) {
        for (var x = from_.x; x <= to.x; x++) {
            far = min(far, depths[info.x + y * info.y + x]);
        }
    }
    return nearest < far;
}

@compute @workgroup_size(256)
fn test(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= args.count {
        return;
    }
    // Distant terrain isn't in cubes: its tiles' boxes follow the draws' origins, two corners each
    // (upstream a80a4e07f).
    if i >= view.skip.x && i < view.skip.y {
        let at = args.count + 2u * (i - view.skip.x);
        let lo = vec3<f32>(origins[at].xyz) - view.cam_frac.xyz;
        let hi = vec3<f32>(origins[at + 1u].xyz) - view.cam_frac.xyz;
        if hidden(lo, hi) {
            draws[i * 5u + 1u] = 0u;
        }
        return;
    }
    let o = origins[i];
    if o.w != 0 {
        return;
    }
    let lo = vec3<f32>(o.xyz) * 16.0 - view.cam_frac.xyz;
    if hidden(lo, lo + vec3<f32>(16.0)) {
        draws[i * 5u + 1u] = 0u;
    }
}

fn put_draw(bucket: u32, rel: vec3<i32>, start: u32, quads: u32) {
    let at = buckets[bucket].base + atomicAdd(&buckets[bucket].count, 1u);
    let d = at * 5u;
    out_draws[d] = quads * 6u;
    out_draws[d + 1u] = 1u;
    out_draws[d + 2u] = 0u;
    out_draws[d + 3u] = start * 4u;
    out_draws[d + 4u] = at;
    out_origins[at] = vec4<i32>(rel, 0);
}

const EMPTY: u32 = 4294967295u;

@compute @workgroup_size(256)
fn cull(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= args.count {
        return;
    }
    let e = table[id.x];
    if e.bucket == EMPTY {
        return;
    }
    let rel = e.cube - view.cam_cube.xyz;
    let lo = vec3<f32>(rel) * 16.0 - view.cam_frac.xyz;
    let center = vec4<f32>(lo + vec3<f32>(8.0), 1.0);
    // A cube's half diagonal.
    for (var p = 0u; p < 4u; p++) {
        if dot(view.planes[p], center) <= -13.86 {
            return;
        }
    }
    if hidden(lo, lo + vec3<f32>(16.0)) {
        return;
    }
    if (e.bucket & 2u) != 0u {
        put_draw(e.bucket, rel, e.start, e.counts[0]);
        atomicAdd(&faces_drawn.count, e.counts[0]);
        return;
    }
    // Only the sides that face the camera, each run of neighbours one draw.
    var seen = array<bool, 6>((lo.x < 0.0), (lo.y < 0.0), (lo.z < 0.0), (lo.x > -16.0), (lo.y > -16.0), (lo.z > -16.0));
    var at = e.start;
    var run = 0u;
    var total = 0u;
    for (var rank = 0u; rank < 6u; rank++) {
        let n = (table[id.x].counts[rank / 2u] >> ((rank % 2u) * 16u)) & 65535u;
        if n == 0u {
            continue;
        }
        if seen[rank] {
            run += n;
            total += n;
        } else if run > 0u {
            put_draw(e.bucket, rel, at, run);
            at += run;
            run = 0u;
            at += n;
        } else {
            at += n;
        }
    }
    if run > 0u {
        put_draw(e.bucket, rel, at, run);
    }
    atomicAdd(&faces_drawn.count, total);
}
