// chiacraft's sky: the engine's, with the pack's atmosphere behind everything, clouds lit by the
// sun and hazed into the sky, and every color brought to the screen by `display`.
// The sky, drawn after the opaque world wherever nothing covers it. Every object is one entry of
// `sky`; things that fill the screen are a single triangle, bodies a quad, stars one quad per star
// made here from their number.

var<private> CORNER: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0),
);

struct VOut {
    @builtin(position) clip: vec4<f32>,
    // Screen position (-1..1) for what fills the screen, else the direction.
    @location(0) at: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) @interpolate(flat) index: u32,
    // A star's number (stars only).
    @location(3) @interpolate(flat) star: u32,
}

fn mat(s: Sky) -> mat3x3<f32> {
    return mat3x3(s.axes[0].xyz, s.axes[1].xyz, s.axes[2].xyz);
}

fn hash(v: u32) -> u32 {
    let s = v * 747796405u + 2891336453u;
    let w = ((s >> ((s >> 28u) + 4u)) ^ s) * 277803737u;
    return (w >> 22u) ^ w;
}

fn unit(v: u32) -> f32 {
    return f32(hash(v)) / 4294967295.0;
}

fn far_point(dir: vec3<f32>) -> vec4<f32> {
    // At depth 0, the far plane: the sky is behind everything.
    let c = scene.view_proj * vec4(dir * 100.0, 1.0);
    return vec4(c.xy, 0.0, c.w);
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    let s = sky[ii];
    var out: VOut;
    out.index = ii;
    out.star = 0u;
    if s.ids.y == SKY_BODY {
        let c = CORNER[vi % 6u];
        let local = vec3(1.0, (0.5 - c.y) * 2.0 * s.extra.x, (c.x - 0.5) * 2.0 * s.extra.x);
        let dir = mat(s) * local;
        out.clip = far_point(dir);
        out.at = dir;
        out.uv = mix(s.uv.xy, s.uv.zw, c);
    } else if s.ids.y == SKY_STARS {
        // A random point on the sphere, a random size and turn, from the star's number alone.
        let star = vi / 6u + u32(s.uv.y) * 7919u;
        let z = unit(star * 4u) * 2.0 - 1.0;
        let a = unit(star * 4u + 1u) * 6.2831853;
        let r = sqrt(1.0 - z * z);
        let toward = vec3(r * cos(a), z, r * sin(a));
        // The bright ones bigger (`star_look` picks the same brightness).
        let size = s.uv.x * (0.6 + 0.4 * unit(star * 4u + 2u)) * 2.0 * (0.85 + 0.9 * pow(unit(star * 9u + 5u), 3.0));
        let turn = unit(star * 4u + 3u) * 6.2831853;
        var side = cross(toward, vec3(0.0, 1.0, 0.0));
        if dot(side, side) < 0.001 {
            side = vec3(1.0, 0.0, 0.0);
        }
        side = normalize(side);
        let up = cross(side, toward);
        let c = CORNER[vi % 6u] - vec2(0.5);
        let q = vec2(c.x * cos(turn) - c.y * sin(turn), c.x * sin(turn) + c.y * cos(turn));
        let dir = mat(s) * (toward + (side * q.x + up * q.y) * size);
        out.clip = far_point(dir);
        out.at = dir;
        out.uv = CORNER[vi % 6u];
        out.star = star;
    } else {
        let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u)) * 2.0 - 1.0;
        // The overlay of a liquid goes over everything, the hand too; depth 1 is the near plane.
        out.clip = vec4(p, select(0.0, 1.0, s.ids.y == SKY_OVERLAY), 1.0);
        out.at = vec3(p, 1.0);
        out.uv = vec2(0.0);
    }
    return out;
}

// Clouds: rounded slabs (Complementary Reimagined's idea, written anew). The picture's cells are
// blended into soft rounded shapes that thin toward the top and bottom of a slab, and each sky
// pixel's ray takes a few dithered samples through it, gathering light and cover: soft blocky puffs,
// lit from the sun above and darker underneath, in the sky's own pass.
// uv: the camera's place in its cell (x, z), the cell in the picture (x, z). extra: cell size,
// thickness, height over the camera, how far they reach. axes[0]: the weather (`sky.rs`): x cover
// as rain comes, y how dark the undersides go, z a storm's deck.
// Ten are enough with the dithered start (the dither hides the steps); 24 cost Ronald's Mac a
// great deal at Retina size (reports/performance.md).
const CLOUD_SAMPLES: u32 = 10u;
// The slab is at least this thick (blocks), whatever the layer's own thickness.
const CLOUD_THICKNESS: f32 = 9.0;
// How far one sample looks along the ray at most (blocks): far clouds take no more samples.
const CLOUD_SPAN: f32 = 400.0;

struct CloudOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) at: vec3<f32>,
    @location(1) @interpolate(flat) index: u32,
}

struct CloudPixel {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@vertex
fn vs_clouds(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> CloudOut {
    let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u)) * 2.0 - 1.0;
    var out: CloudOut;
    out.clip = vec4(p, 0.0, 1.0);
    out.at = vec3(p, 1.0);
    out.index = ii;
    return out;
}

// 1 where cell `cell` (picture cells from the camera's) is cloud: the picture's, and as rain comes
// more of the gaps (a hash per cell, so they grow as whole cells), until a storm's deck closes
// nearly all of them.
// (`look.wgsl`'s `cloud_cell_of`, which the clouds' shadows read too.)
fn cloud_cell(s: Sky, cell: vec2<f32>, dims: vec2<f32>) -> f32 {
    return cloud_cell_of(s.ids.x, cell, s.uv.zw, dims, s.axes[0].x * 0.6 + s.axes[0].z * 0.35);
}

// How dense the cloud is at `cellp` (picture cells, fractional) and `h` (0 at the slab's bottom,
// 1 at its top): the four cells about the point blended smoothly, eaten away toward the top and
// bottom so their edges round.
fn cloud_density(s: Sky, cellp: vec2<f32>, h: f32, dims: vec2<f32>) -> f32 {
    let q = cellp - vec2(0.5);
    let base = floor(q);
    let f = q - base;
    let u = f * f * (3.0 - 2.0 * f);
    let a = cloud_cell(s, base, dims);
    let b = cloud_cell(s, base + vec2(1.0, 0.0), dims);
    let c = cloud_cell(s, base + vec2(0.0, 1.0), dims);
    let d = cloud_cell(s, base + vec2(1.0, 1.0), dims);
    let cover = mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
    let middle = 1.0 - pow(abs(h * 2.0 - 1.0), 2.0);
    // A storm's deck is one sheet: rounded puffs would leave lens-shaped gaps where four meet, the
    // bright sky showing through them.
    let deck = s.axes[0].z * smoothstep(0.0, 0.4, middle);
    return max(smoothstep(0.35, 0.6, cover - (1.0 - middle) * 0.5), deck);
}

@fragment
fn fs_clouds(in: CloudOut) -> CloudPixel {
    let s = sky[in.index];
    let dir = normalize(mat(sky[0]) * in.at);
    let size = s.extra.x;
    let low = s.extra.z;
    let thick = max(s.extra.y, CLOUD_THICKNESS);
    let reach = s.extra.w;
    let high = low + thick;
    var near = 0.0;
    var far = reach / max(length(dir.xz), 0.0001);
    if abs(dir.y) > 0.00001 {
        near = max(near, min(low / dir.y, high / dir.y));
        far = min(far, max(low / dir.y, high / dir.y));
    } else if low > 0.0 || high < 0.0 {
        discard;
    }
    if near >= far {
        discard;
    }
    // What the opaque world shows at this pixel (its depth's copy, `scene.behind`, made before the
    // sky pass): clouds behind a hill or a wall are not marched at all, and the rest stop at it.
    // The pass writes its depth, so the GPU cannot skip those pixels by itself.
    if scene.behind.z > 0.5 {
        let d = textureLoad(textures[u32(scene.behind.y)], vec2<i32>(in.clip.xy), 0).r;
        if d > 0.0 {
            let size = vec2<f32>(textureDimensions(textures[u32(scene.behind.y)]));
            let uv = in.clip.xy / size;
            let w = scene.inv_view_proj * vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
            let solid = length(w.xyz / w.w);
            if solid <= near {
                discard;
            }
            far = min(far, solid);
        }
    }
    far = min(far, near + CLOUD_SPAN);
    let dims = vec2<f32>(f32(s.ids.z), f32(s.ids.w));
    let step = (far - near) / f32(CLOUD_SAMPLES);
    // A different start per pixel, against banding.
    let jitter = fract(sin(dot(in.clip.xy, vec2(12.9898, 78.233))) * 43758.5453);
    let p = palette();
    let lit_sky = open_sky() && !in_liquid();
    let dark = s.axes[0].y;
    let mu = max(dot(dir, scene.sun.xyz), 0.0);
    var through = 1.0;
    var light = vec3(0.0);
    var seen = far;
    for (var i = 0u; i < CLOUD_SAMPLES; i++) {
        let t = near + (f32(i) + jitter) * step;
        let rel = dir * t;
        let h = clamp((rel.y - low) / thick, 0.0, 1.0);
        let density = cloud_density(s, s.uv.xy + rel.xz / size, h, dims);
        if density <= 0.0 {
            continue;
        }
        let a = 1.0 - exp(-density * step * 0.18);
        // Lit from above, darker underneath (more so in rain), glowing toward the sun.
        var c = vec3(1.0);
        if lit_sky {
            let below = mix(p.horizon * 0.9 + p.ambient * 0.6, p.ambient * 1.6, h) * mix(1.0, 0.4, dark * (1.0 - h));
            c = below + p.sun * light_strength() * (0.25 + 0.75 * h) * 0.95 * (1.0 - dark * 0.6);
            c += p.glow * (pow(mu, 3.0) * 0.5 + pow(mu, 12.0) * 0.8) * light_strength() * (0.4 + 0.6 * h);
        } else {
            c = vec3(mix(0.7, 1.0, h));
        }
        if through > 0.5 && through * (1.0 - a) <= 0.5 {
            seen = t;
        }
        light += through * a * c;
        through *= 1.0 - a;
        if through < 0.02 {
            break;
        }
    }
    let alpha_cloud = 1.0 - through;
    if alpha_cloud < 0.01 {
        discard;
    }
    // The engine's color (`sky.rs`) brings rain's grey and thunder's dark; the palette the light.
    var color = s.color.rgb;
    let rel = dir * seen;
    let dist = length(rel.xz) / reach;
    if lit_sky {
        let d = clamp(scene.sun.y * 2.0 + 0.5, 0.0, 1.0);
        let dimmed = pow(vec3(d * 0.9 + 0.1, d * 0.9 + 0.1, d * 0.85 + 0.15), vec3(2.2));
        let albedo = s.color.rgb / max(dimmed, vec3(0.005));
        color = albedo * light / alpha_cloud;
        color = mix(color, sky_color(normalize(vec3(dir.x, max(dir.y, 0.0) + 1e-3, dir.z))), smoothstep(0.3, 1.0, dist));
        color = display(fog_through(color, dir * 220.0, 1.0));
    } else {
        color = mix(s.color.rgb * light / alpha_cloud, scene.fog_color.rgb, smoothstep(0.5, 1.0, dist));
    }
    let alpha = s.color.a * alpha_cloud * (1.0 - smoothstep(0.8, 1.0, dist));
    let clip = scene.view_proj * vec4(rel, 1.0);
    return CloudPixel(vec4(color * alpha, alpha), clip.z / clip.w);
}

// A star as the pack sees it: round, of its own brightness (few bright, many faint) and color (its
// temperature: orange, white, blue-white), twinkling more low down, and showing only once the sky
// is dark enough for it (the brightest first at dusk). rgb tints, a scales the engine's star.
fn star_look(star: u32, uv: vec2<f32>, dir: vec3<f32>) -> vec4<f32> {
    let bright = pow(unit(star * 9u + 5u), 3.0);
    let warmth = unit(star * 9u + 6u);
    var tint = mix(vec3(1.0, 0.62, 0.38), vec3(1.0), smoothstep(0.0, 0.3, warmth));
    tint = mix(tint, vec3(0.66, 0.8, 1.0), smoothstep(0.55, 1.0, warmth));
    // Stars are a pixel or two across: no shape of their own, the size tells the bright ones.
    let shape = 1.0;
    let speed = 2.0 + 3.0 * unit(star * 9u + 7u);
    let twinkle = 1.0 - (0.08 + 0.2 * (1.0 - clamp(dir.y, 0.0, 1.0))) * (0.5 + 0.5 * sin(scene.params.w * speed + unit(star * 9u + 8u) * 40.0));
    let dark = smoothstep(0.05, -0.18, scene.sun.y);
    let needs = 1.0 - (0.25 + 0.75 * bright);
    let shows = smoothstep(needs, needs + 0.15, dark);
    return vec4(tint, shape * (1.0 + 1.6 * bright) * twinkle * shows);
}

// The Milky Way: a faint band across the night, fixed among the stars (it turns with the sky, as the
// sun does), mottled, with dark lanes along its middle; washed out by a bright moon and haze low down.
fn milky_way(dir: vec3<f32>) -> vec3<f32> {
    let dark = smoothstep(-0.08, -0.25, scene.sun.y);
    if dark <= 0.0 || dir.y <= 0.0 {
        return vec3(0.0);
    }
    let a = atan2(scene.sun.y, scene.sun.x);
    let local = vec3(dir.x * cos(a) + dir.y * sin(a), -dir.x * sin(a) + dir.y * cos(a), dir.z);
    let d = dot(local, normalize(vec3(0.25, 0.35, 0.9)));
    let band = exp(-(d / 0.16) * (d / 0.16));
    let mottle = noise3(local * 6.0) * 0.6 + noise3(local * 14.0) * 0.4;
    let lane = smoothstep(0.55, 0.75, noise3(local * 9.0 + vec3(3.0))) * exp(-(d / 0.05) * (d / 0.05));
    let glow = band * (0.4 + 0.8 * mottle) * (1.0 - 0.6 * lane);
    let clear = (1.0 - rain_amount()) * (1.0 - 0.6 * scene.moon.w) * smoothstep(0.0, 0.25, dir.y);
    return vec3(0.75, 0.78, 0.95) * glow * 0.03 * dark * clear;
}

// An aurora on clear nights where the place lies under the auroral oval (`scene.ground.w`, by
// latitude): curtains of ridged noise in the northern sky, green low and purple higher up, drifting
// slowly, on a coarse grid of directions so they stay blocky.
fn aurora(dir: vec3<f32>) -> vec3<f32> {
    let strength = scene.ground.w * smoothstep(-0.1, -0.25, scene.sun.y) * (1.0 - rain_amount()) * (1.0 - scene.weather.z);
    if strength <= 0.0 || dir.y <= 0.02 {
        return vec3(0.0);
    }
    let d = floor(dir * 160.0) / 160.0;
    let north = smoothstep(-0.1, 0.6, -d.z);
    let t = scene.params.w;
    // On a ceiling over the land, so the curtains run along the sky and close toward the horizon.
    let p = d.xz / max(d.y, 0.05);
    let ridge = 1.0 - abs(noise3(vec3(p.x * 0.35 + t * 0.03, t * 0.05, p.y * 0.12)) * 2.0 - 1.0);
    let curtain = pow(ridge, 6.0) * (0.6 + 0.4 * noise3(vec3(p.x * 1.3, t * 0.2, p.y * 0.4)));
    let height = smoothstep(0.02, 0.15, d.y) * (1.0 - smoothstep(0.35, 0.75, d.y));
    let color = mix(vec3(0.1, 0.9, 0.45), vec3(0.55, 0.25, 0.85), smoothstep(0.15, 0.55, d.y));
    return color * curtain * height * north * strength * 0.12;
}

// Vanilla's sky is a disc 16 blocks above the eye fading into fog with distance.
fn gradient(dir: vec3<f32>, s: Sky) -> vec4<f32> {
    // Under water, the water's own color all round (`look.wgsl`), not the engine's dark fog.
    if in_water() {
        return vec4(display(underwater_color()), 1.0);
    }
    // In another liquid (lava): its fog all round (the engine draws the gradient alone then).
    if s.extra.x > 0.5 {
        return vec4(scene.fog_color.rgb, 1.0);
    }
    if open_sky() && !in_liquid() {
        var c = sky_color(dir) + milky_way(dir) + aurora(dir);
        if dir.y < 0.0 {
            // From below sea level, vanilla's dark of the void.
            c *= 1.0 - s.uv.x * smoothstep(0.0, 0.08, -dir.y);
        }
        // The fog on the land hides the horizon and fades upward.
        c = fog_through(c, dir * 220.0, 1.0);
        return vec4(display(c), 1.0);
    }
    if dir.y <= 0.0 {
        // Under the horizon: fog, and from below sea level the dark of vanilla's void.
        let dark = s.uv.x * smoothstep(0.0, 0.08, -dir.y);
        return vec4(scene.fog_color.rgb * (1.0 - dark), 1.0);
    }
    return vec4(mix(s.color.rgb, scene.fog_color.rgb, clamp(0.0625 / dir.y, 0.0, 1.0)), 1.0);
}

// Vanilla's sunset fan: an oval lying on the horizon on the side of the body.
fn glow(dir: vec3<f32>, s: Sky) -> vec4<f32> {
    let toward = dot(dir, s.axes[0].xyz);
    if toward <= 0.0 {
        return vec4(0.0);
    }
    let x = dot(dir, s.axes[2].xyz) / toward / 1.2;
    let y = dir.y / toward / max(0.4 * s.color.a, 0.001);
    var a = s.color.a * max(0.0, 1.0 - sqrt(x * x + y * y));
    if open_sky() {
        a *= 0.25;
    }
    return vec4(display(s.color.rgb), a);
}

// Panorama: longitude and latitude. Box: three by two faces; first row down, up, south; second
// west, north, east.
fn dome_uv(l: vec3<f32>, s: Sky) -> vec2<f32> {
    if s.uv.x == 0.0 {
        return vec2(atan2(l.x, -l.z) / 6.2831853 + 0.5, 0.5 - asin(clamp(l.y, -1.0, 1.0)) / 3.1415927);
    }
    let m = abs(l);
    var face: vec2<f32>;
    var cell: vec2<f32>;
    if m.y >= m.x && m.y >= m.z {
        face = vec2(l.x, select(-l.z, l.z, l.y > 0.0)) / m.y;
        cell = vec2(select(0.0, 1.0, l.y > 0.0), 0.0);
    } else if m.z >= m.x {
        face = vec2(select(l.x, -l.x, l.z > 0.0), -l.y) / m.z;
        cell = select(vec2(1.0, 1.0), vec2(2.0, 0.0), l.z > 0.0);
    } else {
        face = vec2(select(-l.z, l.z, l.x > 0.0), -l.y) / m.x;
        cell = select(vec2(0.0, 1.0), vec2(2.0, 1.0), l.x > 0.0);
    }
    return (cell + face * 0.5 + 0.5) / vec2(3.0, 2.0);
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let s = sky[in.index];
    if s.ids.y == SKY_OVERLAY {
        // Vanilla's: four tiles across the screen, sliding with the look. A texture with
        // animation frames shows its first.
        if s.extra.y != 0.0 && in.at.y > -0.25 {
            return vec4(0.0);
        }
        let tile = fract((in.at.xy * vec2(0.5, -0.5) + 0.5) * s.extra.x + s.uv.xy) * s.uv.zw + vec2(0.0, s.extra.z);
        let c = textureSampleLevel(textures[s.ids.x], nearest_clamp, tile, 0.0) * s.color;
        return vec4(c.rgb * c.a, c.a);
    }
    var dir = in.at;
    if s.ids.y == SKY_GRADIENT || s.ids.y == SKY_GLOW || s.ids.y == SKY_DOME {
        dir = mat(sky[0]) * in.at;
    }
    dir = normalize(dir);
    if s.ids.y == SKY_GRADIENT {
        return gradient(dir, s);
    }
    if s.ids.y == SKY_GLOW {
        let c = glow(dir, s);
        return vec4(c.rgb * c.a, c.a);
    }
    let local = dir * mat(s);
    var uv = in.uv;
    var albedo = vec4(1.0);
    if s.ids.y == SKY_DOME {
        uv = dome_uv(local, s);
        if s.ids.x != 0xffffffffu {
            albedo = textureSampleLevel(textures[s.ids.x], linear_clamp, uv, 0.0);
        }
    } else if s.ids.x != 0xffffffffu {
        albedo = textureSampleLevel(textures[s.ids.x], nearest_clamp, uv, 0.0);
    }
    var c = sky_shade(s.ids.z, SkyPixel(dir, local, uv, albedo, vec4(s.color.rgb, 1.0), scene.params.z, scene.params.w));
    var a = c.a * s.color.a;
    var rgb = c.rgb;
    if s.ids.y == SKY_STARS && open_sky() && !in_liquid() {
        let star = star_look(in.star, in.uv, dir);
        rgb *= star.rgb;
        a *= star.a;
    }
    if open_sky() && !in_liquid() {
        // Bodies shine through the tone curve; stars dim in the haze low down.
        // The sun shines through the tone curve; the moon keeps its markings.
        var shape = 1.0;
        // (Within about 25° of the sun: the moon lags the day now and can stand on its side of the sky.)
        if s.ids.y == SKY_BODY && dot(dir, scene.sun.xyz) > 0.9 {
            // The sun as a round disc with a soft glow (Bliss's), not the picture's square: that
            // square, about 20° across with a pale border, went white through the curve, washed
            // out the golden hour (and the water's path with it), and turned into a diamond on the
            // leaned path. White-gold by day, a deep orange low down, the glow gone before the
            // quad's edges.
            let half = max(abs(s.uv.zw - s.uv.xy) * 0.5, vec2(1e-4));
            let r = length((in.uv - (s.uv.xy + s.uv.zw) * 0.5) / half);
            let disc = 1.0 - smoothstep(0.12, 0.15, r);
            let halo = exp(-r * 6.0) * (1.0 - smoothstep(0.55, 0.95, r)) * 0.7;
            let g = golden();
            rgb = mix(vec3(1.0, 0.96, 0.86), vec3(1.0, 0.52, 0.2), g) * (disc * mix(3.0, 1.5, g) + halo);
            shape = smoothstep(0.0, 0.004, disc + halo);
        }
        a *= 1.0 - rain_amount() * 0.9;
        // `display` lifts black a little: outside the sun's shape nothing may be added.
        rgb = display(rgb) * shape;
    }
    return vec4(rgb * a, select(a, 0.0, s.ids.w != 0u));
}
