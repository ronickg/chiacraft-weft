// Bloom (chiacraft's shader pack turns it on, `mods/base/src/shaderpack.rs`): what is brighter than
// the screen can show spills a soft glow around it. Its glow is gathered at half the size each way
// (`shaders/post_low/bloom.wgsl`: the Mac's bloom read 24 taps on every pixel, Research round 16)
// and filtered up here; without the half-size image, gathered here as before. It also darkens the
// corners a little (a vignette).
fn apply(p: PostPixel) -> vec4<f32> {
    var glow: vec3<f32>;
    // The HDR world's bloom pyramid glows in the last pass (`scene.frame.y`, chiacraft's pack):
    // here only the vignette then.
    if scene.frame.y > 0.5 {
        let d = length((p.uv - vec2(0.5)) * vec2(1.0, p.size.y / p.size.x)) * 2.0;
        return vec4(p.color.rgb * (1.0 - 0.2 * smoothstep(0.55, 1.15, d) * min(p.strength, 1.0)), p.color.a);
    }
    if p.low != 0u {
        glow = post_low_at(p, p.uv).rgb;
    } else {
        glow = bloom_glow(p);
    }
    // Warm brights (fire, lamps, a low sun) spill wider than white ones: firelight's glow carries.
    let warm = clamp((glow.r - glow.b) / max(glow.r, 1e-3), 0.0, 1.0);
    var c = p.color.rgb + glow * (0.4 + 0.25 * warm) * min(p.strength, 1.0);
    // A light vignette that keeps the eye in the middle (Valheim's): the corners a fifth darker.
    let d = length((p.uv - vec2(0.5)) * vec2(1.0, p.size.y / p.size.x)) * 2.0;
    c *= 1.0 - 0.2 * smoothstep(0.55, 1.15, d) * min(p.strength, 1.0);
    return vec4(c, p.color.a);
}
