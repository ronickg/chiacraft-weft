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

**What was seen in game:** Minecraft 26.2 with Fabric, Sodium, Polytone and Recrafted on the PC (instance `mc-recrafted`). Shots are in [In the game](#in-the-game) below.

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

## The plan, ranked

Ranked by how much realism each adds for what it costs, with the leaves first because they are what Ronald asked about. "Engine" means `crates/` (a `notes/ENGINE_CHANGES.md` line each); "pack" means `mods/base/shaderpack`.

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

### 2. The rest of the wind's small life, on the same plumbing

- **Adds:**
  - petals blowing off flowers in bloom, and kicked up as you walk through them (`vitals::wild::BLOOMS`);
  - spindrift streaming across snow in a blizzard (`SNOWING` + wind > `air::BLIZZARD.0`);
  - sand and dust streaming low over deserts in a dust storm (`air_flags::DUST`; this is where we beat him, he has none);
  - pollen motes in spring sun;
  - drips from leaves for a while after rain.
- **Cost:** cheap, once 1 exists.
- **Driven by:** the same air, the blooms, our dust and blizzard flags, a wetness value (see 3).
- **Owner:** the same Wind life session, after 1. Art from the game-art skill.

### 3. Wet ground that lasts, and puddles

- **Adds:** after rain the ground stays dark and puddled, then dries by sun and warmth. Puddles reflect the sky and ring in the rain.
- **Build:**
  - **Base:** a wetness value per biome, integrated in `weather.rs`. It rises with rain and dries by sun, temperature and humidity (`vitals::air`, real evaporation), and is sent in the air beside snow lying, as `scene.ground.w`.
  - **Pack:** `world.wgsl` uses wetness instead of rain amount for the darkening. Puddles are an fbm threshold that grows with wetness, snapped to 1/16 block, on flat up-faces open to the sky (sky light ~15). Their mirror is the sky, and the pack's SSR `reflect.wgsl` is reused for what's near. The ring field already exists for water (`rain_rings`).
- **Cost:** cheap in `world.wgsl` (one fbm per wet up-face pixel), plus the SSR it already pays.
- **Driven by:** our rain spells by place, the sun, temperature, humidity.
- **Owner:** **the Shader session** (it owns the pack's weather look now). The wetness value is a small piece of `weather.rs` and `vitals::air`, the same session or a helper.

### 4. Snow above a line on mountains

- **Adds:** a rainy biome's peaks get snow. Rain turns to snow as you climb, with a soft band between.
- **Build:**
  - `vitals::temperature` gets a lapse rate (6.5 °C/km, a block is a metre), if it doesn't have one; none was found.
  - The engine's rain strips pick rain or snow per strip by the biome's °C minus the lapse above sea level: either the air sends °C per biome, or the pack picks in `weather.wgsl` from `scene.ground` plus a °C channel.
  - The server's snow lying follows the same line.
- **Cost:** cheap.
- **Driven by:** climate °C, height.
- **Owner:** the Shader session for the look, Survival or `vitals` for the lapse (it also changes body temperature on peaks, which belongs in `SYSTEMS.md`).

### 5. Grass that parts round the player

- **Adds:** walking through tall grass, ferns and crops bends them away, and they spring back slowly.
- **Build:** the pack's `world.wgsl` `wind()` adds a push away from the camera's feet (the eye minus 1.6) within ~1.1 blocks, flattening within 0.45. The "stays bent a while" part needs a short trail of positions: either `scene.eye` history from the engine (4 points) or skip it.
- **Cost:** cheap.
- **Owner:** the Shader session; the trail is one engine uniform.

### 6. Ambient sound: leaves, grass and insects

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

### 7. Frost and snow on leaves

- **Adds:** in a snowy winter the crowns go white on top, not only the ground.
- **Build:** pack only. `snow_on` already works per pixel on up-faces. Extend it to leaf blocks' top faces and the upper part of their sides, masked by the leaf texture's luminance (snow catches on the bright leaves, the gaps stay dark).
- **Cost:** cheap.
- **Owner:** the Shader session.

### 8. Snow drifts, dunes and footprints

- **Adds:** wind-shaped snow and sand that read as surfaces, not flat textures. Tracks that fill in.
- **Build:**
  - **Pack:** relief on snow and sand top faces from noise stretched along our wind heading (`air::heading`, sent in the frame), with an asymmetric dune profile (long windward slope, short lee).
  - **Footprints** need a persistent texture round the player that the engine stamps where entities touch ground, and that fades by snowfall or wind. That is an engine piece.
- **Cost:** moderate (the footprint map).
- **Owner:** relief to the Shader session; footprints to a later engine session.

### 9. Water wakes

- **Adds:** rings and V-wakes behind swimmers, boats and animals wading.
- **Build:** a 256-512² wave-equation grid round the camera on the GPU, stamped by entities in water, added to `wave_normal`.
- **Cost:** moderate.
- **Owner:** later. The Shader session or a water session.

### 10. Smaller things worth a line each

- **Held light fades** when you swap items, an exponential approach in the engine's `hand_light` (cheap; engine).
- **Lightning bolts drawn** at the strike, procedural and stepped like our pixel style, not an atlas (Shader session; it just added the flashes).
- **Leaf transmission** in the sun's shadows (light through a crown is green and dim, not black). Check what `fs_shadow`'s dappling already gives (Shader session).
- **Glass that tints the sun's light** through it: a colour channel in the shadow map (later; engine).
- **Rain that stops under slanted roofs:** our strips stop at the column's top. A slanted march in `weather.wgsl` through `solid_at` would match the wind's slant (cheap; Shader session).

**Not worth taking:**
- his voxeliser, coloured flood fill, TAA and atlas plumbing, which work around Minecraft; we have RGB light and our own frame;
- his single-scatter sky, since Ronald's look is Bliss's vivid palette;
- his fixed wind.

## In the game

Minecraft 26.2 (Fabric 0.20.1, Fabric API 0.161.0, Sodium 0.9.2, Polytone 26.2-6.8.5, Recrafted 0.2.1 at its defaults, every feature on) in Prism instance `mc-recrafted` on the PC. World seed 1, made headless by Mojang's 26.2 server. Scenes by `~/Coding/chiacraft-archive/tester-tools/recrafted/scenes.py` through the comparison session's `run_pack.sh`, in two 15-minute queue holds. Seed 1 on 26.2 has no desert, badlands or warm ocean within `/locate`'s reach, so the beach stands in for sand.

(Shots and the side-by-side sheets follow when the runs are done.)
