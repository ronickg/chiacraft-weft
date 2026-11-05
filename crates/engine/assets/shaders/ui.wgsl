var<private> CORNER: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0),
);

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) @interpolate(flat) tex: u32,
    @location(3) @interpolate(flat) layer: u32,
    @location(4) @interpolate(flat) mode: u32,
    @location(5) @interpolate(flat) rect: vec4<f32>,
    @location(6) @interpolate(flat) fx: u32,
    @location(7) @interpolate(flat) size: vec2<f32>,
    @location(8) uv_next: vec2<f32>,
    @location(9) @interpolate(flat) blend: f32,
}

fn unpack(c: u32) -> vec4<f32> {
    let v = vec4<f32>(f32((c >> 24u) & 255u), f32((c >> 16u) & 255u), f32((c >> 8u) & 255u), f32(c & 255u)) / 255.0;
    return vec4(pow(v.rgb, vec3(2.2)), v.a);
}

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> VOut {
    let q = quads[vi / 6u];
    let c = CORNER[vi % 6u];
    var p = q.p0 + q.u * c.x + q.v * c.y;
    var out: VOut;
    out.color = unpack(q.color);
    if q.mode == UI_MODE_FONT && (q.fx & 0x7fffffffu) != 0u {
        let g = apply_fx(q.fx & 0x7fffffffu, Glyph(vec2(0.0), out.color, screen.size.z, q.index, c, screen.size.w, (q.fx >> 31u) == 1u));
        p += g.offset;
        out.color = g.color;
    }
    out.clip = vec4(p.x / screen.size.x * 2.0 - 1.0, 1.0 - p.y / screen.size.y * 2.0, 0.0, 1.0);
    out.uv = mix(q.uv.xy, q.uv.zw, c);
    out.uv_next = out.uv;
    out.blend = 0.0;
    // A strip of frames (`.anim` next to the texture): the one to show now, picked on the CPU.
    if q.layer != 0u && q.mode != UI_MODE_FONT {
        let frames = f32(q.layer >> UI_STRIP_COUNT_SHIFT);
        out.uv_next.y = (out.uv.y + f32((q.layer >> UI_STRIP_BITS) & UI_STRIP_MASK)) / frames;
        out.uv.y = (out.uv.y + f32(q.layer & UI_STRIP_MASK)) / frames;
        out.blend = q.index;
    }
    out.tex = q.tex;
    out.layer = q.layer;
    out.mode = q.mode;
    out.rect = q.clip;
    out.fx = q.fx;
    out.size = vec2(length(q.u), length(q.v));
    return out;
}

// Where to sample a picture so its pixels stay square and crisp at any GUI scale, their edges
// blended over about one screen pixel (sharp bilinear); a picture drawn smaller than its pixels
// (a high-resolution part, `size` in its `.gui`) is simply filtered. `step`: the uv's change per
// screen pixel.
fn sharp_uv(tex: u32, uv: vec2<f32>, step: vec2<f32>) -> vec2<f32> {
    let dims = vec2<f32>(textureDimensions(textures[tex]));
    let p = uv * dims;
    let px = max(1.0 / max(step * dims, vec2(1e-5)), vec2(1.0));
    let c = floor(p - 0.5) + 0.5;
    let f = clamp((p - c - 0.5) * px + 0.5, vec2(0.0), vec2(1.0));
    return (c + f) / dims;
}

// An icon strip's frame, blended into the next one while it animates (`blend`), each sampled
// sharp (`sharp_uv`).
fn strip_pixel(in: VOut, step: vec2<f32>) -> vec4<f32> {
    let t = textureSampleLevel(textures[in.tex], linear_clamp, sharp_uv(in.tex, in.uv, step), 0.0);
    if in.blend > 0.0 {
        return mix(t, textureSampleLevel(textures[in.tex], linear_clamp, sharp_uv(in.tex, in.uv_next, step), 0.0), in.blend);
    }
    return t;
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let step = fwidth(in.uv);
    if in.rect.z > in.rect.x && (in.clip.x < in.rect.x || in.clip.x > in.rect.z || in.clip.y < in.rect.y || in.clip.y > in.rect.w) {
        discard;
    }
    if in.mode == UI_MODE_SOLID {
        return in.color;
    }
    if in.mode == UI_MODE_GUI {
        return gui_fill(in.fx, GuiPixel(in.uv, in.size, in.color, screen.size.z));
    }
    if in.mode == UI_MODE_ICON {
        let t = strip_pixel(in, step);
        if t.a < 0.1 {
            discard;
        }
        return t * in.color;
    }
    if in.mode == UI_MODE_FONT {
        // `layer` 1: a smooth font's coverage (linear); else a bitmap's whole pixels.
        var t = textureSampleLevel(textures[in.tex], nearest_clamp, in.uv, 0.0);
        if in.layer == 1u {
            t = textureSampleLevel(textures[in.tex], linear_clamp, in.uv, 0.0);
        }
        return vec4(in.color.rgb, in.color.a * t.r);
    }
    return strip_pixel(in, step) * in.color;
}
