// Ray tracing's scene (`client::world_render::rt`): one mesh's records (faces, free quads or a
// distant tile's boxes) turned into the four corners of each, three floats a corner, for a
// bottom-level acceleration structure; the see-through ones (from `job.copy_from` on) also
// copied whole into `rt_records`, where a ray's hit looks up its texture.
// Not a shader packs may replace: it decides what casts shadows, not how anything looks.

const RT_NONE: u32 = 0xFFFFFFFFu;

fn nan() -> f32 {
    return bitcast<f32>(0x7FC00000u);
}

@compute @workgroup_size(64)
fn cs_expand(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x + id.y * 65535u * 64u;
    if i >= job.count {
        return;
    }
    let r = job.src + i;
    var c0 = vec3(0.0);
    var c1 = vec3(0.0);
    var c2 = vec3(0.0);
    var c3 = vec3(0.0);
    var words = 4u;
    switch job.kind {
        case 0u: {
            let w0 = records[r * 4u];
            let p = face_pos(w0);
            let d = face_dir(w0) * 4u;
            c0 = p + CORNERS[d];
            c1 = p + CORNERS[d + 1u];
            c2 = p + CORNERS[d + 2u];
            c3 = p + CORNERS[d + 3u];
        }
        case 1u: {
            words = 8u;
            c0 = quad_pos(records[r * 8u]);
            c1 = quad_pos(records[r * 8u + 1u]);
            c2 = quad_pos(records[r * 8u + 2u]);
            c3 = quad_pos(records[r * 8u + 3u]);
        }
        default: {
            let w0 = records[r * 4u];
            let w2 = records[r * 4u + 2u];
            let w3 = records[r * 4u + 3u];
            let d = lod_dir(w0);
            if d == LOD_FACE_GRID {
                // A smooth tile's cell (none in chiacraft): left out, a corner that is not a
                // number makes the triangles inactive.
                c0 = vec3(nan());
                c1 = c0;
                c2 = c0;
                c3 = c0;
            } else {
                let lo = lod_pos(w0, w2) - vec3(0.0, 0.125, 0.0);
                let s = lod_size(w2, w3);
                c0 = lo + CORNERS[d * 4u] * s;
                c1 = lo + CORNERS[d * 4u + 1u] * s;
                c2 = lo + CORNERS[d * 4u + 2u] * s;
                c3 = lo + CORNERS[d * 4u + 3u] * s;
            }
        }
    }
    let o = (job.dst + i * 4u) * 3u;
    positions[o] = c0.x;
    positions[o + 1u] = c0.y;
    positions[o + 2u] = c0.z;
    positions[o + 3u] = c1.x;
    positions[o + 4u] = c1.y;
    positions[o + 5u] = c1.z;
    positions[o + 6u] = c2.x;
    positions[o + 7u] = c2.y;
    positions[o + 8u] = c2.z;
    positions[o + 9u] = c3.x;
    positions[o + 10u] = c3.y;
    positions[o + 11u] = c3.z;
    if job.copy != RT_NONE && i >= job.copy_from {
        let to = job.copy + (i - job.copy_from) * words;
        for (var w = 0u; w < words; w++) {
            rt_records[to + w] = records[r * words + w];
        }
    }
}
