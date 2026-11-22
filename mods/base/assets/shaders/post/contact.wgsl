// Contact shadows (chiacraft's shader pack turns them on, `mods/base/src/shaderpack.rs`): how much of
// the space around each point is closed in by what is drawn near it, from the world's depth. Worked
// out at half the size each way (`shaders/post_low/contact.wgsl`) and upsampled by depth here, so a
// darkening keeps to its own surface; without the half-size image, worked out here as before.
fn apply(p: PostPixel) -> vec4<f32> {
    var dark: f32;
    if p.low != 0u {
        dark = post_low_upsampled(p).r;
    } else {
        dark = contact_dark(p);
    }
    return vec4(p.color.rgb * dark, p.color.a);
}
