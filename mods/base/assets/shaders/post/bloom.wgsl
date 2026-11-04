// Bloom (chiacraft's shader pack turns it on, `mods/base/src/shaderpack.rs`): what is brighter than
// the screen can show spills a soft glow around it, the sun's disc into a halo, a sunlit cloud's
// edge into the sky beside it, a fire's warm light a little wider. Three rings of samples, near,
// wider and widest, of only the bright part of the picture. Last of the effects, so the shafts'
// light blooms too; it also darkens the corners a little (a vignette).

fn bloom_bright(c: vec3<f32>) -> vec3<f32> {
    let y = dot(c, vec3(0.2126, 0.7152, 0.0722));
    return c * smoothstep(0.85, 1.0, y);
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
    // Warm brights (fire, lamps, a low sun) spill wider than white ones: firelight's glow carries.
    let warm = clamp((glow.r - glow.b) / max(glow.r, 1e-3), 0.0, 1.0);
    var c = p.color.rgb + glow * (0.4 + 0.25 * warm) * min(p.strength, 1.0);
    // A light vignette that keeps the eye in the middle (Valheim's): the corners a fifth darker.
    let d = length((p.uv - vec2(0.5)) * vec2(1.0, p.size.y / p.size.x)) * 2.0;
    c *= 1.0 - 0.2 * smoothstep(0.55, 1.15, d) * min(p.strength, 1.0);
    return vec4(c, p.color.a);
}
