# Recrafted study: what MR_CHOO's pack does, and what we build from it

Recrafted session, 2026-11-05, for Planner and Ronald. Ronald (11-05): "he has very cool sort of stuff for weathers and shaders ... I think it would give us an even more realistic feeling, because he even has like leaves flying around."

**What was read:**
- Recrafted Early Access v0.2.1: 4,448 files, Ronald's paid copy, unzipped in a scratchpad. Nothing from it was run on the Mac.
- Polytone 26.2-6.8.5: the Fabric mod that runs it. Only its class and resource names were read.
- Our `dev` at 958111da.

The detailed notes, module by module, are in [`recrafted-study/`](recrafted-study/), all in our own words:
- [weather, water and surfaces](recrafted-study/A_weather_water_surfaces.md)
- [light, shadows, post and the Polytone wiring](recrafted-study/B_light_shadow_post_polytone.md)
- [particles, foliage and sounds](recrafted-study/C_particles_foliage_sounds.md)
- [ImmersiveVFX](recrafted-study/D_immersivevfx.md), added on Ronald's word (11-05, through Planner): "This mod also looks pretty interesting ... interesting things here as well that we can learn from, or even add features". It is ImmersiveVFX 1.5.1 by Shinyflvres (NeoForge, MC 26.3, All Rights Reserved), CurseForge file 9451348. Section [ImmersiveVFX](#immersivevfx) below.

**What was seen in game:** on the PC, Minecraft 26.2 with Fabric, Sodium, Polytone and Recrafted (instance `mc-recrafted`), and Minecraft 26.3 with NeoForge and ImmersiveVFX (instance `mc-ivfx`). Shots are in [In the game](#in-the-game) below. Two things there change the plan. ImmersiveVFX's sandstorm shows how far our dust storm is behind: ours is a flat whiteout, while its storm keeps the near air clear and puts the motion into flying sand (item 2, with a pack-only first step). Recrafted's grass tufts are the best look in his part a (item 5). Falling leaves were missed under the canopy in part a. Part b shoots them from a cleared column, and its sheet will be added here.

**Sheets** (in `~/Coding/chiacraft-archive/images/recrafted/2026-11-05/`): `side_by_side_weather.jpg` (ours, Recrafted and ImmersiveVFX side by side), `recrafted_a_sheet.jpg`, `ivfx2_v_sheet.jpg`.

**His licence forbids reuse:** none of his textures, models, sounds, shaders or code may go into projects that don't require Recrafted, or be ported to another game. So this report describes techniques only. Everything we build is our own code, art from the game-art skill, and our own (CC0) sounds. His files stay in Ronald's Downloads and on the PC's private `~/recrafted-study`, never in the repository or the archive's shared folders.

## The short version

Recrafted's realism comes from three things. We have two of them already, and better:

1. **The wind is in everything.** One wind drives the swaying plants, the falling leaves, the rain's slant, the water's waves, the drifts in snow and sand, and the rustle sounds. His wind is simple:
   - It always blows west.
   - Its speed "breathes" on two slow sines (24 s and 67 s) and doubles in rain.
   - Gust fronts roll across the land as noise on an 8-block grid, moving downwind.
   - Ours is richer: wind by biome, season, day and hour, a heading that turns, storm gusts and gust fronts (`vitals::air`, `weather.rs`). **But our particles don't feel it at all**: `particle_sim.wgsl` has no wind term.
2. **Small things move, and some of them stay.** Leaves fall from every leaf block and burst off the downwind side as a gust passes. They flutter, tumble and land flat. In his complete preset they lie for minutes, tumble on along the ground when a gust comes, are lifted again in storms, float on water and are pushed by the player's wake. Petals blow off flowers and puff up as you walk through them. This is what Ronald saw, and **it is the biggest gap we have**: we have no falling leaves, petals, blown snow or dust.
3. **The weather leaves marks.** Ground darkens and puddles grow as it keeps raining and dry by sun. Snow and sand get wind-drifted relief and footprints that fill back in. Frost sits on leaves under snow. Rain stops under slanted roofs. We have wet ground and snow on up-faces, but no puddles, no relief and no footprints.

The rest of his pack mostly rebuilds what an Iris shader pack does, inside a resource pack: shadow maps, volumetric light, clouds, fog, sky, water, reflections, TAA, bloom and coloured light. Our engine and pack already have most of these natively:
- shadow cascades with far terrain casting
- fog banks lit by the shadow map
- cloud shadows
- water with refraction and caustics
- RGB block light
- held light
- SSR reflections

His techniques there are clever workarounds for Minecraft's limits; we don't have those limits. A few ideas are still worth taking (below).

## How it works: Polytone turns a resource pack into a shader pack

Polytone gives a resource pack five hooks, and Recrafted builds a whole frame graph from them:
- **Overlays as feature switches.** 74 folders each turn on by a config expression (for example "shadows or coloured lighting"). The classic, balanced and complete presets are only data.
- **Post chains in priority order.** There are about 40, from the sky LUT at 12 to bloom at 93, each with an activation expression and uniforms the CPU works out every tick.
- **Persistent "atlas" textures as shared memory between passes.** A 3104x6144 atlas holds the heightmap, its colour, the shadow maps, the rain-shelter map and a Hi-Z pyramid. Another holds the weather field, the biome fog field, cloud shadows and the wake simulation.
- **Extra cameras ("viewpoints"):**
  - a sun shadow map: orthographic, 66 blocks, 1056², snapped to a 1/16-block grid;
  - a top-down heightmap: 4096², refreshed every 200 ticks;
  - a voxeliser: a 100-block slab drawn every 4 ticks, whose vertex shader moves marker quads onto texels. The depth buffer becomes a 96x64x96 light grid.
- **Global expressions:** small CPU programs run every tick that can read their own last value. That makes them integrators, latches and filters: wetness, wave phase, held-light fade.
- **Smuggling data:** material ids go in the framebuffer's alpha (face id, leaves, snow, sand, water); flags go in sub-pixel offsets of model vertices. Lightning positions are even read back by recognising the vanilla bolt mesh in a vertex shader.

Our engine is ours to change, so we need none of the smuggling. The same data comes straight from the frame: `scene.weather`, `scene.ground`, `solid_at`, the shadow cascades, `scene.behind`.

## Feature by feature

Costs: **cheap** is a few ALU per pixel or a tiny pass; **moderate** is a full-screen pass or a 512² simulation; **heavy** is a second world render, a half-res raymarch or a 15-pass chain.

| feature | what you see | how he does it | what drives it | cost | ours today |
|---|---|---|---|---|---|
| **Falling leaves** | leaves drift down from every tree, tinted like the tree | Each leaf block has two emitters on vanilla's display tick:<br>• an ambient drop at about 1.25% a tick, varying 0.4-2.1% with the wind's breathing, under the block;<br>• a gust burst of up to 7x that, only while a gust front passes, on the downwind face.<br>No season, biome or "air below" test. Sprites are 5x5 px, 2 per tree, tinted by a per-biome colormap. | wind breathing, gust field, rain (more bursts) | cheap (vanilla particles, ~30 s life, die on contact) | **none** (`notes/OVERVIEW.md`: "Not yet: falling leaves") |
| **Leaf flight** | flutter, side-to-side flips, a slant downwind in storms, landing flat | Velocity relaxes toward wind minus a 1.9 m/s sink, with drag that grows with speed against the air. Each leaf has a seeded flutter (about 1.7 s) added as a derivative, and a tilt that swings as a soft square wave (±100°) for the flips. When slow, it lies flat on the face it touched. Wind is scaled by sky light, so sheltered places get 15%. | wind, gusts, sky light | cheap per leaf | GPU particles have gravity, drag, block collision and landing rules, but **no wind and no 3D tumble** |
| **Leaf litter that moves** (complete preset) | leaves land, settle with a little hop and stay for minutes; a gust flips them 0-3 times along the ground; storms lift some again; they pin against walls, float on water, part around the swimming player | a per-particle state machine (~70 KB script), life 4 min, culled when out of view for 5-30 s | wind threshold per leaf with a cooldown, rain > 0.4 for lift-off, water wind for floating | moderate (heavy script per leaf, bounded by culling) | none moving; `LEAF_LITTER` is a worldgen floor block |
| **Petals** | petals blow off flowers in gusts; walking through flowers kicks them up | same emitters at a lower rate, with a gust factor of 2 and a puff of lift every ~5 s; the player's controller spawns petals from the flower they stand in, by walking speed | wind, gusts, player speed | cheap | none (we have blooms by season, `vitals::wild::BLOOMS`) |
| **Torch embers, spores** | sparks swirl up from torches; spores drift from glow lichen; motes rise from sculk | small spiralling self-lit particles, 1x4 px animated strips | always | cheap | the torch's vanilla-style flame and smoke; fireflies on warm nights (`fireflies.rs`) |
| **Plant sway** | grass and ferns bend in pixel steps, tips most; leaf cubes shimmer inside their outline | fragment-side, not vertex:<br>• plants shift texel columns by a tip-weighted amount, snapped to pixels, stepped at 20 Hz;<br>• leaf faces wrap their texture in 4-texel clumps with their own phase;<br>• "bushy" leaves add ragged tuft planes | wind field, rain (×2), sky light (none under cover) | cheap | vertex sway of plants and leaves: lean plus gust bands plus flutter (`world.wgsl` `wind()`), the same field for neighbours |
| **Player pushes plants** | grass bends away within 1.1 blocks, flattens at 0.45, and stays partly bent after you pass | the player's position and a smoothed velocity history as uniforms; a "plastic" set that decays | player | cheap | none |
| **Rain** | 3D streaks with motion blur, curtains rolling in with the wind, a far veil, rain seen through water | All in post. Three hashed lattices of capsule drops are drawn analytically. Fragment passes bin the drops into short per-tile lists. Wind-advected "front" noise thins the rain into sheets. Drops refract the sky. Two far shells sit at 23 and 30 blocks. Vanilla rain is zeroed. | rain level, weather field, shelter, wind | heavy (~15 passes) | strips per column slanted by the wind; procedural streaks and flakes, lightning lit by the engine (Shader session, in dev) |
| **Where it rains or snows** | snow above a line on mountains, rain below | a 320² weather field (4 blocks a texel): whether the biome precipitates, and a snow line at sea level + 17 + (T − 0.15)×800 | biome temperature, height | cheap | snow or rain per **biome** from real °C (`air::SNOWS_BELOW`), not per height: a rainy biome's peaks get rain |
| **Shelter** | rain stops under roofs, trees and *slanted* overhangs | a 512² map of the highest occluder along the slanted rain line, from the heightmap by log-doubling hops; "reach" per column eases in over ticks | heightmap, rain | moderate, only while wet | the engine's rain columns stop at the top block (straight down only) |
| **Puddles and wet ground** | flat open ground darkens, then puddles grow, mirror the sky with rings in rain, and dry in the sun; footprints in wet ground; a wet band wicks up walls | A global wetness value rises by +0.006 a tick in rain and dries 3-5x slower, faster at midday. The puddle mask is a two-scale fbm threshold on xz snapped to 1/16. It is kept off sheltered ground, darkened by a porous-wet model and reflects the sky occluded by the heightmap. Rings are sized from the rain flux. A persistent 512² footprint map is used. | wetness integrator, sun, shelter | moderate | wet ground: darker, a sky sheen, open and up-facing, not plants, only **while** it rains (`world.wgsl:418`); rain rings on water only |
| **Snow and sand relief** | snow drifts and grain, dunes with sharp wind-aligned crests and ripples, footprints that fill back in (snow ~2 min) | Terrain flags snow and sand tops in alpha. A post pass replaces them with a 128-block tiling height field: wind-stretched noise for drifts, an asymmetric sawtooth for dunes. A persistent 512² footprint map is fed by a ground-contact pass. | wind direction (fixed), sun, shadows | moderate (complete preset) | snow on up-faces in joining patches (`snow_on`), snow layers placed as blocks; sand is its texture |
| **Frost on leaves** | white frost on leaves under snow, ragged by the leaf's own pixels | the leaf model swaps when the block above is snow; the frost mask comes from the texture's luminance | snow above (no temperature test) | cheap | none (snow lies on top faces only) |
| **Ice** | cracks, scratches, depth in ice | cell polylines with bevelled normals, grain, a grazing fade (**turned off in 0.2.1**) | none | cheap | the ice texture; freeze and thaw by °C |
| **Water** | waves steepening in wind; wakes behind swimmers and boats; foam; caustics (off in this release) | A 5-octave dispersive wave spectrum with anti-aliasing, phase integrated so storms speed it without jumps. A 512² wave-equation wake grid is stamped by entities' silhouettes. Refraction is marched against the bed. | water wind (breathing, ×2 in rain), entities | moderate (heavy with the wake) | golden-angle noise waves (Bliss's idea), distance-filtered normals, higher in wind, rain rings, refraction, absorption, caustics, foam, depth from `solid_at`; **no wakes** |
| **Underwater** | Snell's window, a mirror beyond it, marine snow, shafts | Beer-Lambert by hue, total internal reflection, two lattices of drifting specks | eye in water | moderate | teal water fog with sun beams through the waves (`under_water`) |
| **Fog** | ground-hugging haze in the sky's colour along the view; each biome its own sight distance; rain closing in | analytic exponential layer over the real ground height in 6 segments; colour from the sky LUT; a biome visibility field blurred across borders; per-biome clear, rain and storm tables | rain, biome, sky | moderate | fog banks that lie in valleys and drift with the wind, lit through the shadow map (shafts); fog by place, kind and hour (`air::fog`); haze by distance; rain mist. **Ours is richer.** |
| **Sky and clouds** | physical sky (Rayleigh/Mie/ozone baked to a small LUT); voxel cloud slabs raymarched at half res with silver linings and a hole around the sun | LUT once a frame; clouds at 4-8 steps plus 2 light steps; cloud shadows by DDA | sun, rain | sky cheap, clouds heavy | five-key palette sky; a cloud slab and a rain deck; cloud shadows (Shader session) |
| **Lightning** | forked bolts where vanilla's strike is, flicker stages, clouds lit from inside | bolt positions read back from vanilla's mesh; a 16-slot state table; bolts from a pre-drawn atlas (6 shapes × 7 stages) | vanilla lightning | cheap | flashes from `Scene::lightning`, thunder delayed by distance (`vitals::weather_sound`); no bolt |
| **Sun shadows** | soft shadows, light through leaves, contact shadows, far horizon shadows | 1056² sun camera snapped to the grid; 12-tap rotated Poisson PCF; transmission through leaves; far shadows by log-step propagation over the heightmap | sun angle | heavy | engine cascades (near 80 blocks, far 400, far terrain casts), a 3x3 or 4x4 soft edge, dappled leaves |
| **Coloured light** | soul fire blue, redstone red, candles by colour; glass casts coloured shadows | vertex-shader voxeliser into a 96x64x96 grid; 15 flood passes keeping the two strongest sources a cell; a DDA walk through glass per shadow texel | block ids | heavy | **the engine already has RGB block light**; warm flicker in `block_light`; no glass-tinted shadows |
| **Held light** | the torch in hand lights around it and fades smoothly on swap | 25-item table, exponential fade (0.73 a tick) | held item | cheap | `scene.hand_light` (`held_light`, plus a glow in fog); no fade |
| **Bloom, AO, TAA, motion blur** | glow round lamps, corner darkening, smooth edges | emitter-masked two-radius bloom; half-res GTAO with highlight protection; TAA falling back to SMAA | | moderate to heavy | engine AO and our grade; bloom softens placed lights in fog |
| **Sounds** | rustle of leaves and grass by wind and sky; grasshoppers; wind and storm wind; fire crackle; a swish walking through grass; 25 surface sets | 241 short oggs; each listed 2-6 times at pitch 0.9/1.0/1.1 and two volumes (4 files become ~24 variants); ambience chance × sky light; birds wired off | sky light, rain, surfaces, walking | cheap | Vintage Story-style weather beds (rain, heavy rain, roof, wind, wind in trees, thunder near/far, muffled by cover, thunder delayed by distance); 39 step and 39 dig sounds. **Missing:** grass and leaf rustle by wind, insects, walking-through-grass, variants per bed |

## What we have that he doesn't

These are worth keeping in mind when we copy an idea: his drivers are thinner than ours.
- **Wind** (`vitals::air`): his blows west forever. Ours has a speed by biome, season, day and hour, a heading that turns through the day (`air::heading`), storm gusts ×1.8 and a gust front an hour before a dry-country storm.
- **Seasons** (`vitals::foliage`, calendar): he has none. We turn leaves gold and russet by tree kind, can drop more of them in autumn and none from evergreens, and have blooms by season for petals.
- **Weather by place** (`climate`, `weather.rs`): his is vanilla's one rain switch. Our rain spells follow each biome's real place, thunder too, with blizzards and dust storms (`air::blizzard`, `air::dust`) and fog by kind and hour.
- **Snow that lies and melts** over days (`air::Place::snow_lying`), and water that freezes and thaws by °C.

## ImmersiveVFX

ImmersiveVFX is the other way to do it:
- **Bedrock data.** Oreville's "Realistic Biomes" add-on for Bedrock (441 particle files, Molang controllers) runs inside a Java port of Bedrock's particle and Molang runtime.
- **Vanilla's shaders swapped in place.** The terrain, entity, block and particle shaders are replaced by its own forward versions. There is no deferred pipeline and no extra full-screen passes beyond a few cheap ones, which is where its "2-3x Iris+Complementary" speed comes from.

Where it does the same as Recrafted, differently:

| | Recrafted | ImmersiveVFX | ours |
|---|---|---|---|
| **Weather** | vanilla's rain level, plus a per-column snow line by temperature | vanilla's clock **turned by biome**: rain becomes sand wind in desert, mesa and savanna, snow in cold biomes; thunder becomes a sandstorm or a blizzard | real spells by place and season, blizzards and dust storms from wind and humidity (richer than both) |
| **Fog** | analytic height fog in post, biome visibility field | a hand-tuned table per biome (~42 profiles) × 10 phases of the day (771 colours), very short in storms: rain 48 blocks, thunder 32, blizzard 38, sandstorm 96 | fog by place, kind and hour, marched through the shadow map |
| **Rain** | analytic drops in post, tile-binned | camera-parented streak particles (750/s from a 20-block disc 40 up), impacts when a drop leaves air | shader strips per column |
| **Sand and dust** | sand relief on the ground, no storm | sand specks, 5-53-block gust streaks, a fast sand curtain from 40 blocks upwind, fog tinted sand | dust storms as fog (`air::dust`), **no particles** |
| **Leaves** | emitters on every leaf block, flutter and tumble, litter that stays | savanna storms shed leaves from leaf blocks found in a 70x20x70 box; leaves land on water | none |
| **Shadows** | a 1056² sun camera snapped to 1/16 block, Poisson PCF, leaf transmission | one cached map re-drawn only on chunk change or a sun step, cross-faded over 1.5 s; **each pixel's position snapped to its texel's centre before the lookup, so shadow edges step in the blocks' own pixels** | engine cascades redrawn per frame (far one on change) |
| **Water** | wave spectrum, wake simulation, heightmap reflections | shore foam from an 8-neighbour mask with a travelling front in 16-px steps; Voronoi caustics snapped to the texel grid; SSR | noise waves, foam by shore distance, caustics, SSR |

What it has that Recrafted doesn't:
- **Smooth block placement:** a placed block springs in (squash and tilt, 0.42 s, ~4 Hz); a row placed together ripples. The real block stays hidden until its mesh is built, so there is no pop.
- **Burning blocks:** glowing crack veins crawl over them (a heat field: charcoal, red, orange, yellow).
- **Fire light:** a quarter-res screen pass over up to 32 fires, with a closed-form halo.
- **Auroras on half of clear cold nights**, and meteor showers elsewhere.
- **A cave probe:** 40 rays, 2 a tick, that tells outdoor from cave ambience.
- **Ambient motes per biome:** pollen by day, fireflies at night, spores on mushroom islands.

## The plan, ranked

Ranked by how much realism each adds for what it costs, with the leaves first because they are what Ronald asked about. "Engine" means `crates/` (a `notes/ENGINE_CHANGES.md` line each); "pack" means `mods/base/shaderpack`. "Seen" says whether the PC shots (11-05) back the item or it comes from the code only.

| # | What | Cost | Owner | Seen |
|---|---|---|---|---|
| 1 | Leaves in the wind: falling, blown, settling, tumbling | cheap to moderate (GPU particles; engine wind, tumble, rest) | new **Wind life** session | from code; part b's clearing shots to follow |
| 2 | Dust storms in layers: clear near air, fog far, grains and curtains | 2a cheap (pack only, can start now); 2b cheap once 1's wind exists | 2a **Shader session**; 2b Wind life | **yes**: ImmersiveVFX's sandstorm vs our flat beige whiteout |
| 3 | The rest of the small life: petals, spindrift, pollen, drips | cheap after 1 | Wind life | petals on the ground seen (cherry); flying ones from code |
| 4 | Wet ground that lasts, and puddles | cheap | **Shader session** (+ a wetness value in `weather.rs`) | from code; not seen within 20 s of rain |
| 5 | Fuller grass tufts | moderate (more quads per plant; art) | art (game-art / block-textures) + meshing check | **yes**: Recrafted's strongest look in part a |
| 6 | Snow above a line on mountains | cheap | **Shader session** + `vitals` lapse rate | from code |
| 7 | Grass parts round the player | cheap | **Shader session** | from code |
| 8 | Ambient sound: leaves, grass, insects | cheap | weather sound owner | (sound; not shot) |
| 9 | Frost on leaves | cheap | **Shader session** | from code; not visible in the snowy taiga shots |
| 10 | Snow drifts, dunes, footprints | relief cheap; footprints moderate | **Shader session**; footprints a later engine piece | from code |
| 11 | Placed blocks spring in | cheap | engine client, small session | from code |
| 12 | Pixel-snapped shadow edges | cheap ALU + shadow resolution | **Shader session**; a look choice for Ronald | from code |
| 13 | Shore foam that runs along the shore | cheap | **Shader session** | from code; the beach shot missed the shore |
| 14 | Water wakes | moderate | later | from code |
| 15 | Smaller things (below) | each cheap | various | from code |

The Shader session's items, in order: 2a, 4, 6, 7, 9, 10 (relief), 12, 13. Items 1, 2b and 3 share one new engine piece (particles that feel the wind), so they belong in one session.

### 1. Leaves in the wind: falling, blown, settling and tumbling (Ronald's ask)

- **Adds:** the forest comes alive. Leaves drift down all year, more as autumn turns, and in storms they stream off the downwind side of the crowns. They lie on the floor and skitter along it in gusts.
- **Build:**
  - **Engine particles get the wind.** `particle_sim.wgsl` reads `scene.weather.xy` and the same gust bands as `world.wgsl`'s `gust()`, moved into the shared prelude so plants and particles agree. A per-kind `wind(coupling)` pulls a particle toward the air's velocity minus its own sink.
  - **Engine 3D tumble.** Particles get an orientation: a flutter phase and a tilt that swings, drawn as a quad turned in 3D rather than facing the camera. A landing rule lays it flat on the face it hit. Both are new particle-kind flags and new `ParticleRule` actions.
  - **Engine persistence with a view cull.** Leaves that settle stay. When the pool fills, the oldest unseen ones go first. A per-kind `rest_secs` sets how long they lie.
  - **Base emitters by the client** (like `fireflies.rs`). In the leaf blocks near the player that have air under them or on the downwind side, spawn at:

    *rate = base × season (autumn ×4, deciduous only) × (0.2 + wind breathing) × (1 + 6 × gust front)*.

    Colour from the tree's tint now (`vitals::foliage::tint`, so autumn leaves fall gold or russet). Numbers go in a new `vitals::leaves` (rates, sink, flutter, rest time, storm lift) with a sheet line.
  - **Gust tumble on the ground.** A resting leaf that feels a gust over its threshold flips 1-3 times downwind. In storms some lift again. All on the GPU (`particle_sim`), against `scene.weather`.
- **Art:** leaf sprites per tree kind, about 5x5 px with two shapes each, through the game-art skill. Litter colours come from the tint, so one set covers all seasons.
- **Cost:** cheap to moderate. A few thousand GPU particles near the player, one wind sample each. The emitters run on the client and are bounded by the pool.
- **Driven by:** our wind (speed, heading, storm gusts), the seasons (fall rate and colour), weather (storm lift), sky light (calm under cover).
- **Owner:** a new session, "Wind life: leaves, petals, blown snow and dust", engine particles plus base. It touches `particle_sim.wgsl` and `crates/engine/src/client/particles.rs` (Planner to check the board for anyone holding them). The Shader session only needs `gust()` moved to the shared prelude, a small change to agree on.

### 2. Dust storms in layers (seen: ImmersiveVFX)

- **Adds:** a dust storm you stand inside. Today ours turns the whole view a flat beige, even a few blocks away (`ours/rc_desert_dust_0.png`). In ImmersiveVFX's sandstorm (`ivfx2_v/v_desert_sandstorm.png` and its burst) the ground near you stays readable, the far trees fade to yellow silhouettes, and the motion comes from three layers of flying sand: tumbling clumps of several sizes close to the camera, a cloud of fine grains further out with thin fast streaks, and the sky in the same yellow. You see the storm because something moves past you, not because everything is hidden.
- **Build:**
  - **2a, pack, now:** grade the dust fog by distance. Keep a clear bubble of about 8-16 blocks (thinner as the wind rises) and let it close fully only at the far range. Tint the sky and the far fog the same dust colour, with a little darker low-sun warmth in savannas (`air_flags::DUST` with the biome's soil colour, if the air carries it). This needs no particles and fixes the whiteout on its own.
  - **2b, after item 1:** the grain layers as wind particles. Fine specks round the player; a few long, thin gust streaks (5-50 blocks) along the wind; and now and then a fast curtain of sand, spawned 40 blocks upwind, that crosses the view in a few seconds. Plus a handful of bigger tumbling clumps near the camera. They come from our real dust storms (dry places, wind 7-11 m/s), not from "rain in a desert". In savannas the same storm sheds leaves from nearby trees, which item 1 covers.
- **Cost:** 2a cheap (fog maths already paid). 2b a few thousand particles.
- **Driven by:** `air_flags::DUST`, wind speed and heading, sun height.
- **Owner:** 2a the **Shader session**; 2b the Wind life session.

### 3. The rest of the wind's small life, on the same plumbing

- **Adds:**
  - petals blowing off flowers in bloom, and kicked up as you walk through them (`vitals::wild::BLOOMS`). Recrafted's cherry grove (`recrafted_a/cherry_clear.png`) shows the petals that have landed, a pink litter under the crowns, which is the settled end of the same particles (item 1's rest);
  - spindrift streaming across snow in a blizzard (`SNOWING` + wind > `air::BLIZZARD.0`);
  - pollen motes in spring sun;
  - drips from leaves for a while after rain.
- **Cost:** cheap, once 1 exists.
- **Driven by:** the same air, the blooms, our blizzard flag, a wetness value (see 4).
- **Owner:** the same Wind life session, after 1. Art from the game-art skill.

### 4. Wet ground that lasts, and puddles

- **Adds:** after rain the ground stays dark and puddled, then dries by sun and warmth. Puddles reflect the sky and ring in the rain.
- **Build:**
  - **Base:** a wetness value per biome, integrated in `weather.rs`. It rises with rain and dries by sun, temperature and humidity (`vitals::air`, real evaporation), and is sent in the air beside snow lying, as `scene.ground.w`.
  - **Pack:** `world.wgsl` uses wetness instead of rain amount for the darkening. Puddles are an fbm threshold that grows with wetness, snapped to 1/16 block, on flat up-faces open to the sky (sky light ~15). Their mirror is the sky, and the pack's SSR `reflect.wgsl` is reused for what's near. The ring field already exists for water (`rain_rings`).
- **Cost:** cheap in `world.wgsl` (one fbm per wet up-face pixel), plus the SSR it already pays.
- **Driven by:** our rain spells by place, the sun, temperature, humidity.
- **Owner:** **the Shader session** (it owns the pack's weather look now). The wetness value is a small piece of `weather.rs` and `vitals::air`, the same session or a helper.

### 5. Fuller grass tufts (seen: Recrafted)

- **Adds:** a meadow that reads as one field, not a grid of blocks. In Recrafted (`recrafted_a/plains_clear.png`, `flowers_clear.png`) short grass is a tuft of tall, bent blades that spill over the block's edges, and flowers stand in it at the same scale. It's the strongest single look in his part a, and it costs no shader.
- **Build:** a grass model with more, longer and bent blades that lean past the block edge, with a few variants picked by position hash so neighbours differ. Our wind sway (`world.wgsl` `wind()`) then moves each blade's tip. Compare with ours in the same light first: Tester 2 shoots one of our meadows beside his `plains_clear`.
- **Art:** through the block-textures and game-art skills, our own blades (his models are his).
- **Cost:** moderate. More quads per grass plant, which is the most common plant, so measure the mesh size and frame time on the PC bench. An LOD rule (full tufts near, today's beyond 32 blocks) keeps it bounded.
- **Owner:** an art and plants session with the block-textures skill; the Shader session only if the sway needs a per-blade weight.

### 6. Snow above a line on mountains

- **Adds:** a rainy biome's peaks get snow. Rain turns to snow as you climb, with a soft band between.
- **Build:**
  - `vitals::temperature` gets a lapse rate (6.5 °C/km, a block is a metre), if it doesn't have one; none was found.
  - The engine's rain strips pick rain or snow per strip by the biome's °C minus the lapse above sea level: either the air sends °C per biome, or the pack picks in `weather.wgsl` from `scene.ground` plus a °C channel.
  - The server's snow lying follows the same line.
- **Cost:** cheap.
- **Driven by:** climate °C, height.
- **Owner:** the Shader session for the look, Survival or `vitals` for the lapse (it also changes body temperature on peaks, which belongs in `SYSTEMS.md`).

### 7. Grass that parts round the player

- **Adds:** walking through tall grass, ferns and crops bends them away, and they spring back slowly.
- **Build:** the pack's `world.wgsl` `wind()` adds a push away from the camera's feet (the eye minus 1.6) within ~1.1 blocks, flattening within 0.45. The "stays bent a while" part needs a short trail of positions: either `scene.eye` history from the engine (4 points) or skip it.
- **Cost:** cheap.
- **Owner:** the Shader session; the trail is one engine uniform.

### 8. Ambient sound: leaves, grass and insects

- **Adds:**
  - leaves rustling in the crowns as the wind gets up, louder in gusts;
  - grass hissing in open fields;
  - grasshoppers and cicadas on warm summer days;
  - crickets on warm nights;
  - a swish with each step through tall grass;
  - more variety in every bed: list each file several times at pitch 0.9/1.0/1.1 and two volumes, his cheap trick.
- **Build:** new beds in `vitals::weather_sound` (or a sibling `ambience`):
  - leaf and grass rustle volume = wind × canopy or grass cover × sky exposure;
  - insects by °C, season and hour (cricket chirp rate follows temperature: Dolbear's law, a nice real touch).
  - Our own CC0 recordings (freesound), listed in `SOURCES.md`.
- **Cost:** cheap.
- **Driven by:** wind, canopy, climate, calendar, time.
- **Owner:** whoever owns weather sound (Survival's `weather_sound`); else a small new session.

### 9. Frost and snow on leaves

- **Adds:** in a snowy winter the crowns go white on top, not only the ground.
- **Build:** pack only. `snow_on` already works per pixel on up-faces. Extend it to leaf blocks' top faces and the upper part of their sides, masked by the leaf texture's luminance (snow catches on the bright leaves, the gaps stay dark).
- **Cost:** cheap.
- **Owner:** the Shader session.

### 10. Snow drifts, dunes and footprints

- **Adds:** wind-shaped snow and sand that read as surfaces, not flat textures. Tracks that fill in.
- **Build:**
  - **Pack:** relief on snow and sand top faces from noise stretched along our wind heading (`air::heading`, sent in the frame), with an asymmetric dune profile (long windward slope, short lee).
  - **Footprints** need a persistent texture round the player that the engine stamps where entities touch ground, and that fades by snowfall or wind. That is an engine piece.
- **Cost:** moderate (the footprint map).
- **Owner:** relief to the Shader session; footprints to a later engine session.

### 11. Placed blocks spring in (ImmersiveVFX)

- **Adds:** building feels alive. A placed block pops in with a squash and a small tilt for less than half a second, and a row placed together ripples along.
- **Build:** engine client.
  - Keep the real block hidden until its section's mesh is rebuilt.
  - Meanwhile draw a moving copy (the falling-block or structure path the engine already has, `finish()` in `world.wgsl`) on a damped spring: about 0.4 s at about 4 Hz, squash and tilt from the face it was placed on.
  - Hand over with no pop.
- **Cost:** cheap.
- **Driven by:** placement only.
- **Owner:** an engine client piece; small, its own session or a UI/feel session. A feel choice for Ronald first: it's playful, and a survival game may want it subtle.

### 12. Shadow edges in the blocks' own pixels (ImmersiveVFX)

- **Adds:** shadow edges that step in exact 1/16-block pixels, matching the texture grid: the pixel-art look of the rest of the world, instead of soft or diagonal-aliased edges.
- **Build:** in the pack's `sun_shadow` and `shadow_in`, move the pixel's world position to the centre of its texture texel (from screen derivatives) before the shadow lookup. This needs a near cascade at 16 texels a block or close to it.
- **Cost:** cheap (a few ALU), plus whatever shadow resolution it needs.
- **Owner:** the Shader session; **a look choice for Ronald** (a pixel-art edge versus our soft 3x3 or 4x4 edge), shown side by side.

### 13. Shore foam that runs along the shore (ImmersiveVFX)

- **Adds:** foam that laps in and out in pixel steps along a beach, instead of a fixed wobbling line.
- **Build:** our `world.wgsl` foam already uses `shore_distance` on the pixel grid. Make the foam front a wave that travels along the shore and in and out with time, stepped in 1/16 block, with sparse dots on the retreating side. Caustics can be snapped to the same texel grid.
- **Cost:** cheap.
- **Owner:** the Shader session.

### 14. Water wakes

- **Adds:** rings and V-wakes behind swimmers, boats and animals wading.
- **Build:** a 256-512² wave-equation grid round the camera on the GPU, stamped by entities in water, added to `wave_normal`.
- **Cost:** moderate.
- **Owner:** later. The Shader session or a water session.

### 15. Smaller things worth a line each

- **Burning blocks glow** with crawling crack veins (charcoal to orange to yellow), and fires light the air round them in a cheap screen pass (ImmersiveVFX). This is for when fire spreads in our game (pack plus engine; later).
- **A cave probe for ambience:** 40 rays round the player, 2 a tick, giving how open, how stony and how big the space is (ImmersiveVFX). It would feed cave or outdoor sound beds. Our `weather_sound` already samples cover in rings for the muffling, so extend that rather than add another (sound owner).

- **Held light fades** when you swap items, an exponential approach in the engine's `hand_light` (cheap; engine).
- **Lightning bolts drawn** at the strike, procedural and stepped like our pixel style, not an atlas (Shader session; it just added the flashes).
- **Leaf transmission** in the sun's shadows (light through a crown is green and dim, not black). Check what `fs_shadow`'s dappling already gives (Shader session).
- **Glass that tints the sun's light** through it: a colour channel in the shadow map (later; engine).
- **Rain that stops under slanted roofs:** our strips stop at the column's top. A slanted march in `weather.wgsl` through `solid_at` would match the wind's slant (cheap; Shader session).

**Not worth taking:**
- his voxeliser, coloured flood fill, TAA and atlas plumbing, which work around Minecraft; we have RGB light and our own frame;
- his single-scatter sky, since Ronald's look is Bliss's vivid palette;
- his fixed wind;
- ImmersiveVFX's Bedrock/Molang runtime and its trick of probing blocks with invisible particles, since a world query does the same;
- its hand-made fog table per biome and hour (771 colours), since ours comes from the place's weather;
- its camera-emitter rain, since our strips plus the Shader session's streaks already cover rain;
- its aurora, since we have ours (`sky.wgsl`, by latitude, `air::AURORA`).

## In the game

- **Recrafted:** Minecraft 26.2 (Fabric 0.20.1, Fabric API 0.161.0, Sodium 0.9.2, Polytone 26.2-6.8.5, Recrafted 0.2.1 at its defaults, every feature on) in Prism instance `mc-recrafted` on the PC. Part a (forests, petals, rain, snow) uses world `recrafted1` (seed 1). Part b (desert, fog, water, ice, night light) uses world `recrafted2` (seed 2), because seed 1 has no desert, badlands or warm ocean within `/locate`'s reach.
- **ImmersiveVFX:** Minecraft 26.3 with NeoForge 26.3.0.101 and ImmersiveVFX 1.5.1 at its defaults, in instance `mc-ivfx`, world `ivfx2` (seed 2).
- **How the worlds and shots were made:** each world was made headless by Mojang's own server for its version, its biomes found with `/locate`. Scenes come from `~/Coding/chiacraft-archive/tester-tools/recrafted/scenes.py` through the comparison session's `run_pack.sh`, each in a 15-minute queue hold.
- **Ours:** dev 958111da through the PC queue (look `rc_ours1`, `ours_scenes.txt`): an autumn birch forest in wind and storm, a desert dust storm, a lake at noon and in rain, a snowy taiga in winter.

### ImmersiveVFX, seen (11-05, 21:38-21:45 PC time)

Sheet: `~/Coding/chiacraft-archive/images/recrafted/2026-11-05/ivfx2_v_sheet.jpg`, all frames in `ivfx2_v/` next to it. All 15 scenes ran, and the log shows no errors from the mod.

- **Desert, rain turned into sand wind, and thunder into a sandstorm:** this is the shot that sells it, and it's what our dust storm should copy in shape. Three layers do the work.
  - Tumbling sand clumps in several sizes, close to the camera, with a few big flakes.
  - A cloud of fine grains further out, with thin fast streaks.
  - A yellow fog that takes the far trees down to silhouettes and turns the sky the same yellow.
  No rain falls. The step from sand wind to sandstorm is a denser cloud and a thicker fog, nothing new. Our `/air ... dust` today is fog and tint only (see `ours/`), with no flying grains near the camera, and those grains are what give it depth.
- **Savanna in thunder:** an orange haze sky over dark-tinted fog, so the storm reads as dry dust, not rain.
- **Forest at dawn, in rain, in thunder:** the dawn fog phase gives a pink-violet band low on the horizon. In rain and thunder the grey fog thickens with distance and the light drops. Its fog table (biome × time phase) is easy to see working.
- **Swamp in the morning:** small yellow glows in the grass. These may be the mod's or vanilla 26.x firefly bushes; I can't tell from the shot. We already have fireflies.
- **Night fire:** the campfire smoke rises in a soft column, and fire and campfire light the bricks warmly. That's close to ours today.
- **Missed:**
  - The beach scene landed over open ocean, so no shore foam is in shot.
  - The snowy-plains camera faced a cliff, so the blizzard's flurry and the night sky can't be read, and no aurora came out (it only shows on some clear cold nights).
  - Shore foam and the aurora are still known from the code only (see the ImmersiveVFX section).

### Recrafted part a, seen (11-05, 22:52-22:59 PC time)

Sheet: `~/Coding/chiacraft-archive/images/recrafted/2026-11-05/recrafted_a_sheet.jpg`, all frames in `recrafted_a/`. This is the third try, the first with Improved Transparency on (the Fabulous preset) and every command allowed. The earlier tries are kept as `recrafted_a_try2_nocommands/` and `recrafted_a_try3_notransparency/`. The log's only complaints are Polytone's own `Missing texture references` warnings for its marker models (`translucent_marker/*`, `colored_light/*`), which are expected because those models are invisible tags.

- **Grass and flowers:** this is the strongest look in part a. Short grass is a tuft of tall, bent blades that covers the block edges, so a meadow reads as one field and not a grid. Flowers stand in it at the same scale. In the cherry grove, fallen petals cover the ground under the crowns.
- **Light under trees:** soft shade and a mild, desaturated grade, natural and not vivid. Our default is vivid like Bliss, by Ronald's choice, so take this grade only as one option among the looks.
- **Rain:** soft and faint, with thin pale streaks against the sky. There are no splashes in the frames and no visible darkening of the ground within 20 s. The puddles need flat open ground and time to build up (`puddle_wet` rises slowly), and this plain is hilly. Thunder mostly darkens the light.
- **Snowy taiga:** snow lies on the ground and the spruce stands dark beside it. The frames don't show frost on the leaves.
- **Missed: falling leaves.** Standing on the ground under the canopy (`motion_blocking_no_leaves`) put leaves right in front of the camera in the oak, dark-oak and birch forests, so no drifting leaf can be seen against them. In storm the frames only get darker. Part b now begins with two forest scenes in a cleared 11×11 column (oak in calm and in storm, then birch), so falling leaves show against open air and sky.

(Part b and the side-by-side sheets follow.)
