# Research round 15: volumetric clouds, fog and every other weather effect

Researcher, 2026-11-04, for Ronald, Planner and Lighting. Research only: no code changed, no game run.

Ronald asked: "also wanna add awesome volumentric clouds and fog ? Or we already got ? Are there any other weather/shader stuff like this that other games add or not ? And first do reserch on the topic", then "also photon shaders as well".

**What was read:**
- Our pack: `mods/base/shaderpack/{sky,look,world,weather}.wgsl`, `mods/base/assets/shaders/post/*.wgsl`, `mods/base/src/shaderpack.rs`, the engine's `client/sky.rs`, `client/mod.rs` (weather, lightning) and `post.rs`, and `docs/sky.md` and `docs/shaders.md`.
- Sources of Bliss (Stable), Photon, Complementary Reimagined/Unbound and Rethinking Voxels, cloned and read for ideas only (licences below). File:line citations are relative to each pack's `shaders/` folder.
- Talks and papers: Horizon Zero Dawn clouds (Schneider 2015), Toft et al. 2016, Wronski 2014, Hillaire 2016 and 2020, Unreal's documentation.
- Game documentation: the Vintage Story wiki, Hytale's weather assets, the Minecraft wiki on Vibrant Visuals and clouds, a Teardown frame breakdown, Valheim mod pages.
- Our shots: Tester 2's `t2_rain2_dev` (dev in clear, rain and thunderstorm).

## The short answer

1. **We already have more than Planner's quick read gave us credit for**, and more than most packs ship by default:
   - Clouds are a ray-marched rounded slab, not boxes.
   - Fog is a real 12-step volumetric ground fog, lit by the shadow maps.
   - The night sky has an aurora, a Milky Way, coloured stars and moon phases.
   - Snow lies on surfaces, the ground goes wet and dark in rain, and rain rings form on water.

   What we lack is **the weather's drama**:
   - The storm reads as a flat grey wash under a near-black lid.
   - Rain leaves no puddles behind.
   - Lightning is only a 0.15 s brightening of the day, with no bolt.
   - Clouds are lit by their height alone, so they look like flat cut-outs, not volumes.
   - Nothing marks *after* the rain: no rainbow, no mist, no drying.
2. **"Volumetric clouds" comes in two kinds, and the blocky kind suits us.** The packs split two ways:
   - **Soft, noise-built cumulus** (Bliss, Photon, SEUS). Photon's needs a quarter-resolution buffer with its own temporal reprojection to be affordable. Bliss's runs at half resolution and leans on TAA.
   - **Blocky volumes from a 2D cell map** (Complementary Reimagined, Photon's `BLOCKY_CLOUDS`, the Better Clouds and Cloudscape mods, vanilla's fancy clouds). These are cheap and run at full resolution.

   Ours is the blocky kind already. The biggest gain is to **light it as a volume**: self-shadowing toward the sun, a bright rim toward it, dark bellies, and storm towers. That costs about 0.1-0.25 ms more. Soft Bliss-style cumulus would need an engine change (a low-resolution cloud target with history) and costs 3-5× as much. Minecraft's own Vibrant Visuals made the same choice: its clouds stay blocky (lit at the edges, casting shadows), and its budget goes on fog, light shafts and water.
3. **The storm problem has a clear cause and cheap fixes.**
   - None of the three packs lets a thunderstorm sky go dark:
     - Complementary floors it at a light grey-blue (0.63, 0.67, 0.73).
     - Bliss has no separate storm look at all.
     - Photon darkens its clouds' albedo, which keeps their shape.
   - Ours goes near black, flat, with a hard line where the deck meets a pale mist.
   - I sent the numbers to Lighting, who is retuning the storm now (notes: `side-session-notes/.../weather-fx-rain-findings.md`).

## Where we are (corrected)

| effect | ours | where | verdict |
|---|---|---|---|
| Clouds | A rounded slab, at least 9 blocks thick, from the 12-block cell map. 10 dithered samples per sky pixel at full resolution. Skips pixels behind terrain (depth copy). Lit by height (dark below, sun on top) plus a glow toward the sun. Rain fills the gaps cell by cell; thunder closes them into one deck. Drift with the wind (×0.12). One layer, at about 224. | `sky.wgsl:85-252`, engine `client/sky.rs:367-470` | **Partial.** Volume-shaped, but not lit as a volume: no light march, no self-shadow, no silver lining, no second layer, and the storm deck is a flat sheet. |
| Cloud shadows | The cell map projected along the sun, 0.8 at a cloud's middle, fading in as the sun climbs. On terrain, water and the shadow sampling. | `look.wgsl:313-361` | **Yes** (Bliss, Photon and VV have it too). |
| Volumetric light | A post effect: 16 jittered steps through both shadow maps (8 on Low, +8 under a canopy), HG g = 0.65, gold at a low sun. Off at night, in rain and under water. | `post/volume.wgsl` | **Yes.** Bliss does 8 steps at half resolution, Photon 8-25 at half. |
| Light shafts | Screen-space, 40 steps toward the sun's screen position (20 on Low), cut by depth. | `post/shafts.wgsl` | **Yes.** |
| Ground and valley fog | `fog_through`: 12 steps per pixel, spaced for near or far, through a height fog lying on the ground and the sea that fills valleys. Banks drift with the wind, the sun burns them to a third by day, and canopy mist rises into the crowns. Each step taps the shadow map (lit mist versus shade, gold at dawn). Blowing snow and dust fill the air higher up. Density comes from `scene.weather.z`, the biome's and hour's fog (`vitals::air`: dawn, sea and ice fog). | `look.wgsl:503-651` | **Yes, rich.** It is per fragment in the world shader, though, so its cost rises with overdraw. |
| Distance haze | Rayleigh-like per channel (blue first) from 90 to 450 blocks, plus Mie grey with fog, canopy and rain. Rain mist from the eye. A forward glow toward the sun. Handover to the sky colour at the far end. | `look.wgsl:731-799` | **Yes** (`look_haze` 1.6). |
| Fog lit by moon | `fog_light` follows `light_dir` and `light_strength`, which hand over to the moon. | `look.wgsl:553` | **Partial**: there is no moon-phase term in the fog itself. |
| Fog lit by torches | None. Block light and the held torch light surfaces only. | | **No.** |
| Wet ground | Rain darkens up-facing surfaces under the open sky by 30 %, with a little sky sheen. It stops the moment rain stops. | `world.wgsl:382-388` | **Partial**: no puddles, no ripples on land, no lingering or drying. |
| Rain on water | Rings in 3×3 cells on the texture's pixel grid. | `world.wgsl:437-456` | **Yes.** |
| Rain and snow | Engine strips per column, slanting with the wind, lit and hazed. | `weather.wgsl`, engine `client/weather.rs` | **Yes**, but in a storm the streaks read big and pale blue (t2_rain2_dev). |
| Splashes | `core:rain` particle registered but never spawned. | engine | **No.** |
| Lightning | Engine: a 0.15 s flash that raises daylight by 0.5, about every 12 s in a storm, counted for the thunder sound. Nothing drawn. | engine `client/mod.rs:2251-2283` | **Partial**: no bolt, no lit clouds, no point light. |
| Rainbow | None. | | **No.** |
| Aurora | Ridged-noise curtains, pixel-snapped (1/160 of a radian), by latitude. | `sky.wgsl:328-345` | **Yes.** |
| Stars and Milky Way | Coloured by temperature, twinkling, brightest first at dusk; a mottled Milky Way with dark lanes. | `sky.wgsl:281-326` | **Yes.** |
| Moon phases | 8 phases; night light and moon shadows follow phase × height. | `docs/sky.md`, Lighting's Night 4 | **Yes.** |
| Snow on surfaces | Shader patches on tops from `scene.ground.x`, plus real `base:snow_layer` blocks near the player. | `look.wgsl:653-665`, `snow.rs` | **Yes.** |
| Wind on leaves and grass | A lean with the wind, plus a flutter that quickens, plus gust bands sweeping across fields. | `world.wgsl:162-185`, `look.wgsl:491-501` | **Yes** (rain doesn't add sway the way it does in Photon). |
| Water | Golden-angle noise waves with distance smoothing, GGX glitter and sparkle, refraction, depth colour, caustics on the bottom, foam, SSR. | `world.wgsl:390-700`, `post/reflect.wgsl` | **Yes.** |
| Under water | Murky teal, beams from the surface, Snell's window, bubbles. | `look.wgsl:671-729` | **Yes.** |
| Sky by season | The palette reads the sun's height and rain. Sun height changes with the season, so the colours do too. There is no hue by season or place. | `look.wgsl:98-160` | **Partial.** |
| Heat haze, dust, sandstorm | Dust fills the fog (`scene.ground.z`, sand-coloured). No shimmer. | `look.wgsl:516-560` | **Partial.** |
| Raindrops on the lens | None. | | **No.** |

**What the engine allows the pack** (this sets how hard each idea is):
- A pack can replace the sky (clouds included), world, weather, particle and model shaders.
- It can add screen effects, but **effects can't add passes, can't keep last frame's picture and can't run at lower resolution** (`post.rs`; effects run at the world's size under `render_scale`, merged into one pass).
- So anything that needs history (cloud reprojection) or a half-resolution target (cheap soft clouds, froxel fog) is an **engine change** in `crates/engine`. It needs a `notes/ENGINE_CHANGES.md` line and, for now, waits until the upstream sync freeze ends.
- Scene values the pack can read today:
  - `weather` (wind, fog, thunder), `ground` (snow, snowfall, dust, aurora), `clouds`, `cloud_cells`, `moon`, `canopy`, `hand`;
  - 16 option slots, 14 of them used.
- The pack cannot read: a rain wetness that lingers, the lightning's position or timer, the temperature, the biome's humidity.

## Volumetric clouds: what the best do

| pack or game | technique | steps (primary / light) | resolution | reprojection | rain and storms |
|---|---|---|---|---|---|
| **Bliss** | Two ray-marched cumulus slabs: 100 blocks thick at y 250, a larger one at 500. Density from a 2D noise texture faked into 3D (two offset slices). Plus a flat altostratus plane at y 2000 (`lib/settings.glsl:378-391`, `lib/volumetricClouds.glsl:54-66,298-333`). | 15 / 3 toward the sun, plus 1 cross-layer each (`settings.glsl:407-408`, `volumetricClouds.glsl:365-380`) | half per axis (`CLOUDS_QUALITY 0.5`) | none of its own; global TAA (blend 0.12) | Low layer coverage → 0.95, density → 1.0; upper layer off; altostratus ×(1 + 3·rain) brighter (`:41-47,559`). No thunder path. |
| **Photon** | Up to 4 layers in a ×10 planet-scaled space: cumulus (or towering congestus), altocumulus, planar cirrus, noctilucent. 2D coverage map plus 64³ Worley detail (`include/sky/clouds/*`). | Cumulus 40 at the horizon to 20 at the zenith / 6 light / 2 ambient. Altocumulus 12-6 / 4 / 2 (`settings.glsl:207-241`). | **one pixel in each 4×4 per frame** (`CLOUDS_TEMPORAL_UPSCALING 4`) | own history: wind-aware, ages up to 20 frames, clamped when the camera moves fast (`d2_clouds_upscaling.fsh`) | A weather model (humidity, temperature, wind random walks) picks the cloud types. In rain: albedo ×0.66, extinction ×0.67 (darker, same shapes); air 80 % grey (`weather/clouds.glsl:156-170`). |
| **Photon blocky** | Vanilla `clouds.png` as a 13-thick slab at 192 (and 280), rounded edges (`include/sky/blocky_clouds.glsl`) | 12 / 4 | full | TAA | sun ×(1 − rain) |
| **Complementary Reimagined** | Blocky slab ±4.2 from a 2D map with rounded corners; stops at the first hit, opaque (`reimaginedClouds.glsl`) | 6 to 12+ by quality / 2 taps on the map | full | TAA | ambient ×0.75, toward the grey sky colour |
| **Complementary Unbound** | 3D fractal noise slab ±11-18, alpha built up front to back (`unboundClouds.glsl`) | distance/16-32 / none (a gradient) | full | TAA | +0.40 coverage. Its changelog claims "no extra performance cost" over Reimagined (developer's claim). |
| **Rethinking Voxels "Fantasy"** | 5 stacked 2D noise planes (`planarClouds.glsl`) | 5 / none | full | TAA | coverage up, grey |
| **SEUS PTGI** | Closed source. E12 improved cirrus and cumulus, coverage changing to the horizon, reworked rain light and fog ([Patreon](https://www.patreon.com/posts/32990443)). | not known | not known | not known | not known |
| **Vintage Story** | Volumetric by default ("classic" selectable); 8 cloud patterns per 512×512 region (cumulonimbus, stratus, haze…), which thicken and darken as rain grows; tile-like gaps ([wiki](https://wiki.vintagestory.at/Weather)) | not published | not published | not published | cumulonimbus with "ceiling fog" |
| **Hytale** | 2D texture layers per weather, each with colours and speeds by hour ([docs](https://docs.hytale.com/assets/weathers)) | none | | | per-weather colours |
| **Minecraft Vibrant Visuals** | **Blocky clouds kept**: lit at the edges, darker inside, casting shadows, reflected in water, out to 128 chunks ([wiki](https://minecraft.wiki/w/Vibrant_Visuals)) | | | | rain and snow hide the sun, change fog and cloud colour |
| **Teardown, Valheim** | Sky textures and particles. Teardown's volumetric light runs at quarter resolution with reprojection and a 50/50 history blend ([breakdown](https://juandiegomontoya.github.io/teardown_breakdown.html)). Valheim's Mistlands mist is a particle system plus fog. | | | | |
| **ours** | Rounded slab from the cell map | 10 / 0 | full | none | gaps fill; flat deck in thunder |

**Cost, from the literature:**
- **Horizon Zero Dawn.**
  - Naive: about 20 ms on a PS4.
  - With a quarter-resolution buffer updating one pixel per 4×4 block each frame: about **2 ms on a PS4** (1.8 TFLOPS), 64-128 steps with 6 light samples ([Schneider 2015](https://www.advances.realtimerendering.com/s2015/The%20Real-time%20Volumetric%20Cloudscapes%20of%20Horizon%20-%20Zero%20Dawn%20-%20ARTR.pdf)).
- **Toft et al. 2016, GTX 1080 at 1080p** ([arXiv 1609.05344](https://arxiv.org/pdf/1609.05344.pdf)):

  | setup | time |
  |---|---|
  | full resolution, 128 steps | 297.7 ms |
  | half resolution, 128 steps | 128 ms |
  | half resolution, 8 steps | 2.3 ms |
  | quarter resolution, 8 steps, jitter + TAA | **2.4 ms** |

- **Vibrant Visuals' fog:** "Ultra" halves the frame rate against "High" (144 → 72 fps on an RTX 3090 Ti, [MCPE-216798](https://mojira.dev/MCPE-216798)). Volumetric anything can eat a frame.
- **Old SEUS volumetric clouds:** 80-120 fps → 15-22 fps on a Radeon 7970 (anecdotal).

**Blocky or soft for us.**
- `notes/STYLE.md` sets the world at 32 px a block, Hytale-like, with "soft edges between colors" and "chunky readable shapes", and "Bliss's sky and water with Valheim's air". Hytale's own clouds are soft painted layers; Minecraft's and Vibrant Visuals' are blocky.
- Blocky volumetric clouds are a well-liked look in their own right: Better Clouds, Cloudscape Volumetric, Complementary Reimagined, Photon's blocky option.
- My recommendation: **keep the low clouds blocky-volumetric** (our cells and rounded slab), lit like a real volume, and add **one soft, high, thin layer** (altostratus or cirrus, a 2D plane like Bliss's at y 2000). That gives depth and a Bliss-like sky behind blocky puffs, for almost nothing.
- Soft Bliss/Photon cumulus stays possible later behind a setting, once the engine has a low-resolution cloud target (rec. 7).
- This is a taste call: Ronald should see both on the sheet and choose.

## Fog: what the best do

| | ours | Bliss | Photon | Complementary |
|---|---|---|---|---|
| Method | 12 steps per fragment, in the world shader | 8 steps, exponential spacing, at half resolution, bilateral upsample (`settings.glsl:166`, `res_params.glsl:16`) | 8 + 0.1·distance steps (25 at most), half resolution (`fog/overworld/raymarched.glsl`, `VL_RENDER_SCALE 0.5`) | analytic (no march); shafts 6-30 samples |
| Ground and valley fog | height layer on the ground and sea; drifting banks | 3 exponential height falloffs from y 60, plus a noisy "cloudy fog" layer | Mie layer from sea level + 7 with a 7-block half-life, ×4·noise² | height-limited above 63, fading over 60 blocks |
| Morning mist | the sun burns banks to a third; gold at dawn | per hour: morning 0.01/0.05, noon 0, evening 0.05, night 0.2 (`settings.glsl:172-182`) | Mie per hour: morning 0.007, noon 0.0001, evening 0.005; humidity up to 8× (`settings.glsl:379-385`) | none |
| By biome | the biome's fog (`vitals::air`), canopy mist | swamp 0.3 green (0.15, 0.25, 0.05), jungle 0.1-0.2, dark forest 0.1 (`settings.glsl:222-240`) | Rayleigh density per biome: arid 0.0003 … swamp 0.0014; pale garden 0.03 | special biome weather |
| Rain | rain mist from the eye, thicker in thunder | `(0.5·low + exp2(−0.06h))·rain`, ×0.1 | Mie 0.030; Mie albedo 0.9 → 0.5 | `RAIN_STYLE 2` heavy fog |
| Torch-lit fog | no | `LPV_VL_FOG_ILLUMINATION` (off by default) | through its light-propagation volume (24 steps) | coloured-light fog (0.65) |
| Moon | via `light_dir` | sun- and moon-phased | sun and moon feed its LUT | |

**Cost:**
- Froxel fog is a fixed cost: Wronski's 160×90×64 grid costs about 1.1 ms on consoles whatever the screen size. Unreal's runs about 1 ms on a PS4 at High and about 3 ms on a GTX 970 at Epic.
- Ours runs per fragment at full resolution, so on foggy mornings its cost follows the pixel count and overdraw: about 290 hashes per pixel on foggy days, per `reports/performance.md`.
- If fog ever shows in the Mac's rows, the step is to move it into a half-resolution pass, which is an engine change.

**What would add the most:**
- fog that glows round torches and the held light (Valheim's night);
- mist that lingers after rain (needs a lingering wetness value);
- a few biome tints: swamp green, desert dust, jungle haze.

## Every other weather and sky effect

Cost is at 1920×1080 on an RTX 3060-class GPU (see "How the costs were estimated"). Difficulty: S = days in the pack, M = a week or an engine field, L = engine work.

| effect | what it is | who has it | cost | ours | how hard here |
|---|---|---|---|---|---|
| Wet surfaces | Darker albedo, lower roughness, sky reflection; wets over about 70 s, dries over about 300 s | Photon (`settings.glsl:23-24`), Bliss, Complementary | ~0.02 ms | partial (instant, no drying) | S, plus an engine scene value for lingering wetness (or kept in the base mod and sent through `set_air`) |
| Puddles | Noise mask × wetness × faces up × open sky; mirror-smooth (f0 0.02, roughness 0.002); albedo ×0.33, porous ×0.67 | Photon (`surface/rain_puddles.glsl`), Complementary (off by default), Bliss (needs SSR), Teardown | world +0.03-0.08 ms; SSR on puddles +0.2-0.5 ms | no | S-M: the mask in `world.wgsl`; write the puddle into the reflections' alpha mask so `reflect.wgsl` treats it as water |
| Rain ripples on land and water | 2 scrolling noise normals, or expanding rings, while it rains | Photon, Complementary, ours (water only) | <0.02 ms | partial | S (reuse `rain_rings` on puddles) |
| Splashes | Particles or quads where rain hits the ground | Teardown, Particle Rain mod, VS | CPU and particles, tiny | no | M: engine `client/weather.rs` spawns `core:rain` at column bottoms |
| Rain streak look | Thin, mostly transparent streaks (Photon opacity 0.25, snow 0.75), slanted with a wobble | all | 0 | yes (too pale and wide in storms) | S (Lighting, now) |
| Raindrops on the lens | Screen-space drops when looking up in rain | Lynx's Post Processing, Rainy Glass mod | 0.05 ms | no | S: a post effect, behind a setting (it can annoy) |
| Lightning bolt | A jagged emissive line drawn in the sky | Bliss (Iris bolt, bright blue), Complementary (emissive bolt material), VS | ~0.01 ms | no | M: the engine picks a bolt position and timer, and the pack draws it in the sky pass |
| Lightning light | Lights clouds from inside (Photon: +4× the clouds' ambient for 0.75 s) and the ground as a 300-block point light (Bliss) | Photon, Bliss | ~0.02-0.05 ms during a flash | partial (daylight +0.5 for 0.15 s) | M, with the bolt |
| Rainbow | Arcs at 42° (4° wide) and 47° (3°), opposite the sun, × wetness × (1 − rain) | Photon (`sky/rainbow.glsl`), Complementary (pixelated, r5.7) | ~0 | no | S, needs lingering wetness. Pixelated suits our style. |
| Aurora | Curtains of noise on clear northern nights | Complementary, Photon (64 steps), VS | ours ~0 | yes | done |
| Stars and Milky Way | Coloured, twinkling, a galaxy band | Photon, Solas, Hytale | ~0 | yes | done |
| Moon phases | Light by phase (Bliss `\|4−phase\|/4`; Photon down to 0.5 at new moon) | Bliss, Photon, Complementary, Hytale | 0 | yes (stronger than theirs) | done |
| Snow building up | Shader cover plus real layers | VS (up to 3/8 block below 0 °C), Bedrock, Complementary `SNOWY_WORLD` | 0 | yes | done |
| Wind on leaves and grass | Sway with the wind, more in rain (Photon: skylight² × (0.25 + 0.66·rain)) | all | ~0 | yes (rain doesn't add sway) | S: one term in `wind()` |
| Heat haze | Screen ripple over hot ground, lava, deserts at noon | Boralances (lava); rare in packs | 0.05 ms | no | S: a post effect from depth, sun height and a desert/heat scene value |
| Sandstorm and dust | Dense coloured fog and racing particles | Bliss, Photon (density 0.2, extinction (0.2, 0.27, 0.45)), VS | ours ~0 extra | partial (dust fog) | S: tune colours; M: particles |
| Hail | Particles plus damage | VS | particles | no | M (gameplay too) |
| Cloud shadows | Clouds' shade moving over the land | Bliss, Photon (a 512² map), VV, Veloren | ours ~0.02 ms | yes | done |
| God rays in fog | Shadow-mapped in-scattering | all | ours 0.04 ms on the PC | yes | done |
| Crepuscular rays through clouds | Beams under a broken cloud deck | Photon (20-4 steps by humidity and coverage) | 0.1-0.2 ms | no | M: in `volume.wgsl`, tap `cloud_shadow` per step (sun beams from cloud gaps) |
| Underwater caustics and fog | | all, VV per biome | ours | yes | done |
| Waving water | | all | ours | yes | done |
| Sky colour by season and place | Hue keyed by hour and weather | Hytale (everything by hour), Euphoria seasons | 0 | partial | S: a season/biome tint on the palette, from a scene value |
| Fog after rain, ground mist | Mist rising after a shower | VS, Particle Rain | already paid | no | S, with lingering wetness |
| Torch-lit fog | The fog glows round lights at night | Bliss (option), Photon (LPV), Valheim | 0.02-0.05 ms (analytic, held light) | no | S: a closed-form point-light in-scatter along the view for `hand`; a glow for placed lights |
| Storm towers | Cumulonimbus with anvils and a dark base | Photon congestus (30-50 km), VS cumulonimbus | +0.05 ms in our slab | no | S-M: thicker cells and a taller slab in thunder |
| Noctilucent and high clouds | Thin high layers lit after sunset | Photon (80 km, random days), Bliss altostratus | ~0.02 ms | no | S: one 2D plane in `fs_clouds` |
| Shooting stars | | BSU, Newb X | 0 | no | S |
| 22° halo, mirages, frost on glass | Halo round the sun in thin cirrus; frost on panes | none found in packs | | no | not worth it yet |

## Photon in depth (Ronald: "also photon shaders as well")

**Licence.** The "Photon Shaders License Agreement", © 2021-2025 Benjamin Stott, "All rights reserved unless otherwise explicitly stated".
- It allows use, personal modification, examining and learning from the source, and redistributing portions, but not on platforms that pay creators (Modrinth, CurseForge) and not for money without written permission. The licence must ship with copies.
- So **ideas and learning are allowed**. Copied code would put his licence into our game and forbid selling it. We take ideas only and write our own WGSL from the method, as with Bliss. This is not legal advice.

**Clouds.**
- **Layers:**
  - cumulus (or congestus towers far off);
  - altocumulus;
  - a planar cirrus;
  - rare noctilucent clouds.
- **Shape:** a 2D coverage map (512², precomputed and distorted) times 64³ Worley detail. Cumulus has an egg-shaped height profile: wispy bottoms and hard tops.
- **Steps:**
  - cumulus 40 at the horizon to 20 at the zenith, with 6 light steps that double in length each time and 2 ambient steps upward;
  - altocumulus 12 to 6, with 4 light steps;
  - cirrus 1 density sample plus 6 light steps.
- **Lighting** (`clouds/common.glsl:57-85`, `cumulus.glsl:135-188`):
  - single scattering with 0.8·max(Klein-Nishina, HG 0.8) + 0.2·HG(−0.2), which gives the bright silver lining toward the sun;
  - **multiple scattering in 8 octaves**: each octave's extinction ×0.4 and phase g ×0.8. This is what gives a cloud its soft lit interior, not a black core;
  - a **powder** term that darkens the near side's edges;
  - ambient from the sky above, from the 2 upward steps;
  - a ground bounce (albedo 0.4).
- **Speed:**
  - one pixel of each 4×4 block traced per frame (a 16-frame checkerboard, buffer at 0.25 per axis);
  - reprojected by each cloud's distance and moved by its layer's wind;
  - history aged up to 20 frames, clamped only when the camera moves fast;
  - young pixels blurred by a mip bias.
  - This is HZD's method, and the only way soft clouds become cheap.
- **Weather drives the clouds** (`weather/core.glsl`):
  - temperature, humidity and wind wander slowly with a bias per biome;
  - humidity raises cumulus coverage;
  - cold, still air turns it into a stratus sheet;
  - hot, humid, windy air builds congestus towers.
  - Rain darkens the clouds' albedo (×0.66) rather than thickening them.

**Fog and atmosphere.**
- **Sky:** a Bruneton-style precomputed scattering LUT (transmittance 256×64; scattering 16×64×32, shipped as data), with the sky drawn into a 192×108 map and sky light from spherical harmonics.
- **Air fog:**
  - 8-25 steps at half resolution;
  - Rayleigh from sea level + 30 and Mie from sea level + 7 (the valley fog);
  - 4 multiple-scattering octaves;
  - phase 0.7·HG(0.5) + 0.3·HG(−0.2), and +75 % glow near sunset;
  - Mie density per hour and humidity (the table above).
- **Torch fog:** comes through its light-propagation volume.

**Rain and weather.**
- **Rain colour:** sun × lerp(0.033, 0.66, sun height) × (0.49, 0.65, 1.0), a blue-grey that follows the sun's height.
- **Sky:** the sun, moon and stars fade with rain.
- **Wetness:** comes in with a 70 s half-life and dries with a 300 s one.
- **Puddles and ripples:** as in the table above.
- **Lightning:** lights the clouds from inside.
- **Rainbows:** at 42° and 47° in Lab colours.
- **Rain streaks:** tilt 0.25 at 30° with a wobble per position, opacity 0.25.
- **Plants:** sway more in rain.

**Quality settings.**
- The profiles (low, medium, high, ultra) change only:
  - shadow map size;
  - volumetric light on or off;
  - SSAO or GTAO;
  - reflections;
  - caustics;
  - water parallax;
  - coloured shafts and lights.
- Cloud step counts are separate sliders.
- Its own cost notes:
  - reflections are the "big performance impact";
  - TAAU "massively improves performance";
  - a second blocky layer "doubles" that cost.
- Photon's README says it is not supported on Apple Metal.

**What ports to our pack, and how:**

| Photon idea | where it goes here | engine? |
|---|---|---|
| Rain darkens cloud albedo, keeps the shape | `sky.wgsl` `fs_clouds`: scale the lit colour, not the density | no |
| Multiple-scattering octaves, powder, KN/HG silver lining | `fs_clouds`: per sample, 2-3 taps of `cloud_density` toward the sun, then the octave sum (3 octaves is plenty for our slab) | no |
| Planar cirrus or altocumulus | one extra plane hit in `fs_clouds` (1 sample + 2 light taps) | no |
| Weather picks the cloud type (stratus sheet when cold and still, towers when hot, humid and windy) | the cell picture's fill and slab thickness from `scene.weather` and the temperature | a temperature or humidity scene value (small) |
| Wetness 70 s in / 300 s out; puddles; ripples | `world.wgsl`; wetness kept in the base mod, sent to the pack | a scene value (small) |
| Rainbow 42°/47° × wetness × (1 − rain) | `sky.wgsl` `gradient`, pixel-snapped like our aurora | wetness value |
| Lightning lights clouds from inside | `fs_clouds`: add to ambient while a flash is on; better with the bolt's position | flash timer and position (small) |
| Rain streak opacity 0.25, wobble | `weather.wgsl` | no |
| Plants sway more in rain | `world.wgsl` `wind()` | no |
| Quarter-resolution clouds with wind-aware reprojection | an engine cloud target plus history; `fs_clouds` writes into it | **yes (L)** |
| Fog at half resolution | an engine half-resolution effect target | **yes (M)** |

## Recommendations, in order

Each lists its look gain, cost (1080p on a 3060-class GPU / Ronald's Mac at 3456×2234 with render scale 0.67), owner and slices.

**1. The storm and the rain, recoloured** (Lighting, in progress)
- **Gain:** big. The storm is Ronald's complaint ("very grey weird").
- **Cost:** 0 / 0.
- **Slices:**
  1. Floor the deck: thunder darkens it by about 15-25 %, never to black (Complementary's floor of 0.63, 0.67, 0.73 × daylight).
  2. Draw the deck and the mist from one colour, so no hard horizon line.
  3. Make the ambient a cool blue-grey, brighter than the warm, dimmed direct light (Complementary's day rain about 0.45, 0.34, 0.28).
  4. Make the streaks thinner and darker at about 0.25 opacity.
  5. Keep some contrast and bloom (Bliss's bloomy fog ×(1 + rain)) instead of a grey wash.

**2. Clouds lit as volumes, still blocky** (Lighting, right after 1)
- **Gain:** big in every daytime shot, the "awesome volumetric clouds" Ronald asked for.
- **Cost:** +0.1-0.25 ms / +0.2-0.4 ms. Today's clouds are about 0.2-0.4 / 0.3-0.6 ms.
- **Slices:**
  1. **Light march:** at each of the 10 samples that has density, 2 taps toward the sun through `cloud_density`. Beer-Lambert plus a powder term, a two-lobe phase (forward 0.8, back −0.2) for the silver lining, and 3 multiple-scattering octaves.
  2. **Storm deck with structure:** in thunder, lumpy bellies (density varied per cell) and darker undersides, by albedo, not by blackening the colour.
  3. **Storm towers:** in thunder, a few cells grow 3-4× thicker (cumulonimbus).
  4. **A soft high layer:** a 2D altostratus or cirrus plane far above, lit by the sky, taking sunset colours and the moon at night.
  5. Optional: a "true voxel" style that walks the cell grid exactly (cubic edges, like Better Clouds and vanilla's fancy clouds), for Ronald to compare.
- **Bench:** `sky` before and after on Tester 2's queue; `--no-clouds` if a toggle is wanted.

**3. Rain that lands: wetness, puddles, ripples** (Lighting for the shader, the base mod for wetness)
- **Gain:** big in rain and after it, the Bliss/Photon rain look.
- **Cost:** +0.05 ms in the world pass; +0.2-0.5 / +0.3-0.8 ms while puddles fill the screen with reflections on.
- **Slices:**
  1. **Lingering wetness:** the base mod tracks rain at the player (rises over about 70 s, dries over about 300 s, per biome) and sends it to the pack. Either a new scene value (an engine line in `ENGINE_CHANGES.md`) or packed into the air flags as the canopy and aurora already are.
  2. **Wet surfaces from wetness,** not from `rain_amount()`: darker, glossier, drying from the edges.
  3. **Puddles:** a world-space noise mask × wetness × faces up × sky light 15; albedo ×0.33, smooth; `rain_rings` on them while it rains.
  4. **Reflections in puddles:** write them into the alpha mask that `reflect.wgsl` treats as water. On "Low", the sky colour only.
  5. **Splashes:** the engine spawns the registered `core:rain` particle at rain columns' bottoms. This is an engine change; it can wait.

**4. Lightning you can see** (Lighting; a small engine field)
- **Gain:** medium-big. Storms get their drama.
- **Cost:** ~0.03 / 0.05 ms during a flash, 0 otherwise.
- **Slices:**
  1. **Engine:** when a flash fires, choose a bolt position (random within about 100-400 blocks in the rain) and put it in the scene with the flash's age. Keep the daylight bump, a little longer and smoothed (Photon smooths over 0.75 s).
  2. **Clouds lit from inside** near the bolt (`fs_clouds`, 4× ambient falling off with distance).
  3. **A bolt:** a jagged emissive line from the cloud base to the ground, in the sky pass, pixel-snapped to suit the style. 2-3 frames bright, then an afterglow.
  4. **The ground lit** blue-white within about 300 blocks of the bolt (Bliss's colour 77, 153, 255).
- The thunder sound already follows the flash count (`weather_sound.rs`).

**5. After the rain: a rainbow and rising mist** (Lighting, after 3's wetness)
- **Gain:** medium. A moment players remember.
- **Cost:** ~0 / ~0.
- **Slices:**
  1. **A pixelated rainbow:** 42° primary, 47° faint secondary, opposite a low sun, × wetness × (1 − rain).
  2. **Fog after rain:** wetness adds to `scene.weather.z` near the ground for a few minutes. The base mod adds it to the biome's fog in `vitals::air`.
  3. **Plants sway more while it rains:** Photon's 0.25 + 0.66·rain term.

**6. Fog that glows round torches, and fog by biome** (Lighting)
- **Gain:** medium at night: Valheim's warm firelight in mist, which Ronald likes.
- **Cost:** +0.02-0.05 / +0.04-0.08 ms on foggy nights.
- **Slices:**
  1. **The held light's in-scatter along the view:** a closed form for a point light in uniform fog (the airlight integral), so no extra steps. Its colour is the torch's warm flicker.
  2. **Placed lights:** a glow from the shaded point's own block light, added to the fog at that pixel.
  3. **Biome tints:** swamp green-grey, jungle humid haze, desert dust, taiga crisp. Bliss's and Photon's per-biome densities are the start, from `vitals::air`.
  4. **The moon in fog:** scale by phase, as the night light already is.

**7. Clouds at a quarter of the pixels, with reprojection** (engine work; after the upstream sync, Lag fixes or Lighting)
- **Gain:** only if wanted.
  - It makes today's clouds 3-10× cheaper on the Mac.
  - It also opens soft Bliss/Photon cumulus as an option: 20-40 steps plus 4-6 light, about 0.5-1.0 / 1.0-2.0 ms with the reprojection. Without it, at full resolution, they would cost 3-8 ms and are not worth it.
- **Slices:**
  1. A cloud target at ¼ per axis, plus a history texture, plus last frame's view-projection in the scene.
  2. `fs_clouds` traces one pixel in each 4×4 per frame, Photon-style reprojection by distance and wind, an age cap of 20.
  3. A depth-aware upsample into the sky pass.
- Do it **only if** the cloud pass shows in the Mac's rows after 2, or if Ronald picks soft clouds on the sheet.

**8. Small touches** (anyone, later)
- heat haze over deserts at noon and round lava (a post effect, ~0.05 / 0.1 ms);
- raindrops on the lens when looking up in rain (a post effect behind a setting);
- shooting stars on clear nights;
- dust storm particles;
- crepuscular rays from cloud gaps: `volume.wgsl` taps `cloud_shadow` per step, ~0.1 / 0.2 ms.

## Who takes what, and when

- **Lighting:**
  - **now:** 1, already under way.
  - **next:** 2 then 3 (shader slices first, since they need no engine change).
  - **then:** 4's pack side, 5 and 6.
  - All of these are pack work, so they can go ahead during the upstream sync freeze on their own branch. Only the engine fields (wetness, bolt position) wait for the freeze to end.
- **The engine pieces:** 3's wetness value, 3.5's splashes, 4's bolt field and 7's cloud target. Each needs its `notes/ENGINE_CHANGES.md` line. They go to Lighting or Lag fixes after the sync, and Upstream should know they are coming.
- **The base mod and Survival:** lingering wetness and fog after rain in `vitals::air` and `weather.rs`, with a `notes/SYSTEMS.md` row (rain → wetness → fog, puddles, crops later).
- **Ronald decides:** soft clouds (7) or blocky volumetric (2). See the sheet.

## How the costs were estimated

- **The PC.** Our own measured rows (Tester 2, perf table; Lighting's notes):
  - On the RTX 5090 at 3200×1800, total GPU time is 1.5-2.6 ms, of which effects are 0.5-1.4 ms; the volume effect costs +0.04 ms.
  - "3060-class at 1080p" scales the 5090 numbers by throughput (about 8× less) and pixels (2.8× fewer), so about ×3.
- **The Mac.**
  - Tester's row after Perf 7 at 3456×2234 with render scale 0.67 (Ronald's M3 Max): effects 3.3-4.0 ms of 6.7 ms GPU.
  - That is lower than the ×1.7 pixel scaling from the 3060 would give, so Mac figures are given as 1-1.7× the 3060's.
  - The cloud costs come from counting texture loads and ALU per sky pixel, checked against the literature (HZD, Toft et al.).
- **All are estimates until measured.** Every slice should be benched on Tester 2's queue (`bench`, before and after) and once on the Mac through Tester with Ronald's yes.

## Pictures

**The comparison sheet:**
- Full size: `~/Coding/chiacraft-archive/images/research-weather-fx/2026-11-04/weather_fx_sheet.jpg`, with its spec `weather_fx_sheet.json` beside it.
- A small copy is in `reports/weather-fx/weather_fx_sheet.jpg`.
- It was built with Lighting's `sheet.py`.
- The columns are: ours (dev), soft volumetric packs, blocky or stylised, and one more reference.

**What the sheet shows, row by row:**
1. **Clear day clouds.**
   - Ours are flat-lit puffs.
   - Photon's soft cumulus have silver edges.
   - Nostalgia's blocky clouds are lit as volumes, with pink tops and mauve bellies. That is the look rec. 2 aims for.
   - Complementary's rounded slabs are close to ours.
2. **Sunset clouds.**
   - Ours take the dusk colour evenly.
   - The references have lit rims and dark cores.
3. **Rain.**
   - Ours is an even grey wash under a honeycomb deck.
   - Bliss has a structured deck with light breaking through.
   - Vintage Story and Opal keep colour and contrast in the rain.
4. **Thunderstorm.**
   - Ours: a near-black lid, a hard line at the horizon and big pale streaks.
   - Pegasus's storm is bright grey and keeps contrast.
   - Spooklementary shows a lightning bolt.
   - Bliss's overcast is a lumpy, lit deck.
5. **Ground fog.**
   - Ours at dawn by a lake: the mist reads faint in this shot.
   - Photon's lake mist, Escheridia's sunrise valley fog, and Valheim's god rays.
6. **Not built yet:**
   - a rainbow (Photon);
   - puddles in rain (Snowimagined);
   - swamp fog by biome (Bliss);
   - Vibrant Visuals keeping blocky clouds.

Our shots are from Tester 2's existing PC runs: `t2_rain2_dev` (dev in clear, rain and thunderstorm at 12:22-12:46), `t2_ARCH_D` (golden hour) and `t2_FOGFIX_D` (dawn by a lake). Nothing new was run.

**All 106 references** are in `~/Coding/chiacraft-archive/images/research-weather-fx/2026-11-04/refs/`, with one line each in `INDEX.md` (what it shows and its source page). They are internal reference only; the pictures belong to their authors.
- **What I couldn't find:** heat-haze shots, official Vibrant Visuals rain, and Teardown rain.
- **Smaller packs:** the rain and storm rows use them, because Complementary has no rain shot.
- **Weather mods:** the Vintage Story shots tagged `vintage-story-mod-*` come from weather mods, not the base game.
- **Old Hytale shots:** the Hytale shots are from its 2019 posts.

## Sources

- Bliss: https://github.com/X0nk/Bliss-Shader (Stable). Licence: "© 2020 Chocapic13. All rights reserved."
- Photon: https://github.com/sixthsurge/photon. Licence: Photon Shaders License Agreement.
- Complementary Reimagined: https://github.com/ComplementaryDevelopment/ComplementaryReimagined. Licence: Complementary License Agreement 1.7.
- Rethinking Voxels: gri573's GitHub mirror (last commit 2024-03-28).
- BSL: not read (no public source; Modrinth lists it as ARR).
- SEUS PTGI: closed (Patreon E12 notes).
- Clouds and fog:
  - Horizon Zero Dawn: https://www.advances.realtimerendering.com/s2015/The%20Real-time%20Volumetric%20Cloudscapes%20of%20Horizon%20-%20Zero%20Dawn%20-%20ARTR.pdf
  - Nubis: https://www.guerrilla-games.com/read/nubis-evolved
  - Toft et al.: https://arxiv.org/pdf/1609.05344.pdf
  - Wronski: https://bartwronski.com/wp-content/uploads/2014/08/bwronski_volumetric_fog_siggraph2014.pdf
  - Hillaire 2020: https://blog.selfshadow.com/publications/s2020-shading-course/hillaire/s2020_pbs_hillaire_slides.pdf
  - Unreal: https://dev.epicgames.com/documentation/en-us/unreal-engine/volumetric-fog-in-unreal-engine
- Games:
  - https://wiki.vintagestory.at/Weather
  - https://docs.hytale.com/assets/weathers
  - https://minecraft.wiki/w/Vibrant_Visuals
  - https://minecraft.wiki/w/Cloud
  - https://mojira.dev/MCPE-216798
  - https://juandiegomontoya.github.io/teardown_breakdown.html
  - https://veloren.net/devblog-90
- Blocky clouds:
  - https://github.com/Qendolin/better-clouds
  - https://www.curseforge.com/minecraft/mc-mods/cloudscape-volumetric
- Effects:
  - https://betterbedrock.com/project/preview/lynxs_post_processing
  - https://modrinth.com/mod/rainy-glass
  - https://skinmc.net/project/boralancesshaders
