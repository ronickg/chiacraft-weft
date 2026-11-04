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

// A storm deck's structure at `cellp`, 0..1: one smooth field about three and a half cells across
// (the picture's cells, as the wind carries them). Where it is high the deck stands up in towers,
// where it is lowest it opens into lighter breaks. Picked cell by cell (a hash), the towers' dark
// bottoms read from below as a checkered ceiling (Ronald: "Merge, then refine storm", 11-05).
fn storm_field(s: Sky, cellp: vec2<f32>) -> f32 {
    let at = cellp + s.uv.zw;
    return noise3(vec3(at.x / 3.5, 5.5, at.y / 3.5));
}

// How much a storm deck stands up in a tower at a field value `n`.
fn storm_tower(n: f32) -> f32 {
    return smoothstep(0.55, 0.8, n);
}

// The deck's lumps, 0..1, varying with the height in it too, so its belly darkens by depth and not
// by cell.
fn cloud_lumps(s: Sky, cellp: vec2<f32>, along: f32) -> f32 {
    let at = cellp + s.uv.zw;
    return noise3(vec3(at.x / 1.6, along * 2.5, at.y / 1.6));
}

// Henyey-Greenstein's phase for the clouds' light, `g` forward (> 0) or back (< 0).
fn cloud_hg(mu: f32, g: f32) -> f32 {
    return (1.0 - g * g) / pow(1.0 + g * g - 2.0 * g * mu, 1.5) / 12.566;
}

// Where `along` (0 at the slab's bottom, 1 at its top as tall as a storm makes it) is within the
// cloud at `cellp`: its own height 0..1, over 1 above it. A storm's towers (`storm_field`) rise
// smoothly to the whole height; the rest of the deck fills the bottom third.
fn cloud_height(s: Sky, cellp: vec2<f32>, along: f32, storm: f32) -> f32 {
    if storm <= 0.0 {
        return along;
    }
    let span = mix(1.0 / mix(1.0, 3.0, storm), 1.0, storm_tower(storm_field(s, cellp)));
    return along / span;
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
    // Where the storm's field is lowest the deck breaks open a little.
    var deck = s.axes[0].z * smoothstep(0.0, 0.4, middle);
    if scene.weather.w > 0.0 {
        deck *= mix(1.0, smoothstep(0.12, 0.3, storm_field(s, cellp)), scene.weather.w);
    }
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
    // In a thunderstorm the slab grows: a few cells stand up as towers three times as tall.
    let storm = scene.weather.w;
    let tall = thick * mix(1.0, 3.0, storm);
    let high = low + tall;
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
    // Lit as a volume (Research round 15, rec. 2; Ronald: "both, a setting", blocky by default):
    // two taps toward the light through the slab (Beer-Lambert, with a powder term), a two-lobe
    // phase for the silver lining and the backlit glow, and three octaves of multiple scattering.
    let l = light_dir();
    let mu_l = dot(dir, l);
    let phase = 0.7 * cloud_hg(mu_l, 0.8) + 0.3 * cloud_hg(mu_l, -0.2);
    for (var i = 0u; i < CLOUD_SAMPLES; i++) {
        let t = near + (f32(i) + jitter) * step;
        let rel = dir * t;
        let cellp = s.uv.xy + rel.xz / size;
        let along = (rel.y - low) / tall;
        let h = cloud_height(s, cellp, along, storm);
        if h > 1.0 {
            continue;
        }
        var density = cloud_density(s, cellp, h, dims);
        if density <= 0.0 {
            continue;
        }
        // A storm's deck is lumpy, its density varying from cell to cell, blended between them as
        // the density is (a step at each cell's edge showed as a seam running off to the horizon).
        if storm > 0.0 {
            density *= mix(1.0, 0.45 + 1.1 * cloud_lumps(s, cellp, along), storm * 0.8);
        }
        let a = 1.0 - exp(-density * step * 0.18);
        var c = vec3(1.0);
        if lit_sky {
            // How much of the light reaches here through the cloud toward it.
            var od = 0.0;
            for (var j = 1; j <= 2; j++) {
                let q = rel + l * (thick * 0.45 * f32(j));
                let qc = s.uv.xy + q.xz / size;
                let qh = cloud_height(s, qc, (q.y - low) / tall, storm);
                if qh <= 1.0 && qh >= 0.0 {
                    od += cloud_density(s, qc, qh, dims) * thick * 0.45 * 0.18;
                }
            }
            var scatter = 0.0;
            for (var o = 0; o < 3; o++) {
                let k = pow(0.5, f32(o));
                scatter += k * exp(-od * k) * mix(1.0, phase * 4.0, k);
            }
            let powder = 1.0 - exp(-2.0 * (od + density * 0.5));
            // Undersides darker by their albedo in a storm, not blackened.
            // By the height in the deck (the same at the same height everywhere), and darker under a
            // tower's whole column, smoothly, as the towers rise.
            var albedo = 1.0;
            if storm > 0.0 {
                let deep = mix(0.45, 1.0, smoothstep(0.0, 0.5, along)) * mix(1.0, 0.75, storm_tower(storm_field(s, cellp)));
                albedo = mix(1.0, deep, storm);
            }
            let below = mix(p.horizon * 0.9 + p.ambient * 0.6, p.ambient * 1.6, h) * mix(1.0, 0.4, dark * (1.0 - h)) * albedo;
            c = below + p.sun * light_strength() * scatter * mix(0.6, 1.0, powder) * 0.55 * (1.0 - dark * 0.6) * albedo;
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
        // A thunderstorm's deck floored at a mid grey: the engine darkens its colour to a fifth, and
        // the deck went near black over pale mist with a bright band of sky at the horizon between.
        // None of the packs lets a storm sky go that dark (Complementary floors it at about 0.65;
        // the Researcher's weather findings, 11-05); the bellies still darken (`dark`).
        let albedo = max(s.color.rgb / max(dimmed, vec3(0.005)), vec3(0.55, 0.58, 0.64) * scene.weather.w);
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

// A rainbow opposite the sun while a shower passes with the sun out (Research round 15, 5): the
// primary bow 39.5-42.5° from the point opposite the sun, red outside, and a faint second one at
// 50-53.6° with its colours the other way; the sky inside the primary a little brighter. Six flat
// bands, pixel-snapped as the aurora is, to suit the blocks (Complementary's pixel rainbow). Only in
// light rain: a downpour hides the sun, and a thunderstorm makes none. Once the scene carries how
// wet the land still is (after the sync), it lingers as the rain clears.
const BOW: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
    vec3(0.45, 0.2, 0.9), vec3(0.2, 0.35, 1.0), vec3(0.25, 0.9, 0.35), vec3(1.0, 0.95, 0.25), vec3(1.0, 0.55, 0.12), vec3(1.0, 0.18, 0.1),
);

fn rainbow(dir: vec3<f32>) -> vec3<f32> {
    let r = rain_amount();
    let showers = smoothstep(0.03, 0.2, r) * (1.0 - smoothstep(0.4, 0.85, r)) * (1.0 - scene.weather.w);
    // The sun out and under 42°, so the bow stands above the horizon.
    let low_sun = smoothstep(0.0, 0.08, scene.sun.y) * (1.0 - smoothstep(0.5, 0.67, scene.sun.y));
    let strength = showers * low_sun;
    if strength <= 0.0 || dir.y <= 0.0 {
        return vec3(0.0);
    }
    let d = normalize(floor(dir * 100.0) / 100.0);
    let deg = degrees(acos(clamp(dot(d, -scene.sun.xyz), -1.0, 1.0)));
    var c = vec3(0.0);
    let one = (deg - 39.5) / (3.0 / 6.0);
    if one >= 0.0 && one < 6.0 {
        c = BOW[u32(one)];
    }
    let two = (deg - 50.0) / (3.6 / 6.0);
    if two >= 0.0 && two < 6.0 {
        c = BOW[5u - u32(two)] * 0.45;
    }
    // Inside the bow the sky is lit a little by the light it bends there.
    c += vec3(0.12) * (1.0 - smoothstep(30.0, 40.0, deg));
    // It thins toward the horizon, where less rain lies along the look.
    let p = palette();
    // At 0.08 the bow was a faint thread through the shower (the after-rain check, 11-05).
    return c * luma(p.sun) * light_strength() * strength * smoothstep(0.0, 0.06, d.y) * 0.25;
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
        // The rainbow in the rain, in front of the high cirrus.
        c = cirrus(c, dir) + rainbow(dir);
        if dir.y < 0.0 {
            // From below sea level, vanilla's dark of the void.
            c *= 1.0 - s.uv.x * smoothstep(0.0, 0.08, -dir.y);
        }
        // The fog on the land hides the horizon and fades upward.
        c = fog_through(c, dir * 220.0, 1.0);
        // A held torch's glow in the mist goes on over the sky (`haze_lit` gives it the land's).
        c += held_in_fog(dir * 48.0, fog_here(1.0));
        return vec4(display(c), 1.0);
    }
    if dir.y <= 0.0 {
        // Under the horizon: fog, and from below sea level the dark of vanilla's void.
        let dark = s.uv.x * smoothstep(0.0, 0.08, -dir.y);
        return vec4(scene.fog_color.rgb * (1.0 - dark), 1.0);
    }
    return vec4(mix(s.color.rgb, scene.fog_color.rgb, clamp(0.0625 / dir.y, 0.0, 1.0)), 1.0);
}

// A soft high layer of thin cloud, far above the blocky ones (Research round 15, rec. 2 slice 4:
// Bliss's altostratus plane at y 2000): wisps on a plane, drifting slowly with the wind, lit by the
// sky and taking the sunset's colours and the moon's light, thicker under rain. Behind the blocky
// clouds it gives the sky depth.
fn cirrus(c: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    if dir.y <= 0.02 {
        return c;
    }
    let p = palette();
    let t = scene.params.w;
    let at = dir.xz / dir.y * 2000.0 + scene.eye.xz + wind_now() * t * 6.0;
    // Even both ways: stretched along one axis its wisps fanned out from the sun like a warp.
    let q = at / 600.0;
    let n = noise3(vec3(q, t * 0.002)) * 0.6 + noise3(vec3(q * 2.7 + 13.0, t * 0.003)) * 0.3 + noise3(vec3(q * 7.1 + 41.0, 0.0)) * 0.1;
    let rain = rain_amount();
    // Gone toward the horizon, where the plane's texture crowds into streaks.
    let cover = smoothstep(0.55 - 0.25 * rain, 0.85, n) * smoothstep(0.12, 0.4, dir.y);
    let mu = max(dot(dir, light_dir()), 0.0);
    let lit = p.horizon * 0.6 + p.ambient * 0.8 + p.sun * light_strength() * 0.35 + p.glow * pow(mu, 4.0) * light_strength() * 0.8;
    return mix(c, lit, cover * 0.55);
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
