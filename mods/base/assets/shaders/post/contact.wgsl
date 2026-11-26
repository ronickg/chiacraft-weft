// Contact shadows (chiacraft's shader pack turns them on, `mods/base/src/shaderpack.rs`): how much of
// the space around each point is closed in by what is drawn near it, from the world's depth. Worked
// out at half the size each way (`shaders/post_low/contact.wgsl`) and upsampled by depth here, so a
// darkening keeps to its own surface; without the half-size image, worked out here as before.
fn apply(p: PostPixel) -> vec4<f32> {
    // None of the pack's look path traced plain (`scene.rt.x` 3, `core:pt_plain`): the picture as
    // the engine draws it with no pack.
    if scene.rt.x > 2.5 {
        return p.color;
    }
    // Not on water (the world image's alpha, which the pack leaves at 0 for solid things and the
    // reflections before this keep): its surface closes nothing in, and on the Mac (MoltenVK, M3
    // Max) a pool's flat top seen at a slant took level stripes from this pass, 8 pixels a pair,
    // 2 to 4% dark, over the whole water (Tester's macws1-9, 11-25: gone with fx_contact 0 alone,
    // not with post_low, froxels, reflections, TAA or the water's own terms; the PC's was smooth).
    if p.color.a > 0.02 {
        return p.color;
    }
    var dark: f32;
    if p.low != 0u {
        dark = post_low_upsampled(p).r;
    } else {
        dark = contact_dark(p);
    }
    return vec4(p.color.rgb * dark, p.color.a);
}
