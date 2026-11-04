// Bloom (chiacraft's shader pack turns it on, `mods/base/src/shaderpack.rs`): what is brighter than
// the screen can show spills a soft glow around it, the sun's disc into a halo, a sunlit cloud's
// edge into the sky beside it. Three rings of samples, near, wider and widest, of only the bright
// part of the picture. Last of the effects, so the shafts' light blooms too.

fn bloom_bright(c: vec3<f32>) -> vec3<f32> {
    let y = dot(c, vec3(0.2126, 0.7152, 0.0722));
    return c * smoothstep(0.7, 1.0, y);
}

fn apply(p: PostPixel) -> vec4<f32> {
    let aspect = vec2(p.size.y / p.size.x, 1.0);
    var glow = vec3(0.0);
    var weight = 0.0;
    let rings = array<vec2<f32>, 3>(vec2(0.006, 0.5), vec2(0.016, 0.3), vec2(0.04, 0.2));
    for (var r = 0; r < 3; r++) {
        let radius = rings[r].x;
        for (var i = 0; i < 8; i++) {
            let a = (f32(i) + f32(r) * 0.37) * 0.785398;
            let at = p.uv + vec2(cos(a), sin(a)) * radius * aspect;
            glow += bloom_bright(scene_at(p, at).rgb) * rings[r].y;
            weight += rings[r].y;
        }
    }
    glow /= weight;
    return vec4(p.color.rgb + glow * 0.55 * min(p.strength, 1.0), p.color.a);
}
