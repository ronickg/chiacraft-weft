# Research round 17: a fantasy feel in the light, sky and air

Researcher, 2026-11-05, for Ronald, Planner and Lighting. Research only: no code changed, no game run.

**Ronald's words:**
- "Also maybe if we can bring in some fantasy feeling into it as well i think that would be dope"
- "Yes well lets add we can also have the option to turn on off parts"

**What was read:**
- **Our pack on dev `eca61f43`:** the lit blocky clouds with cirrus, the smooth storm, the cheaper mist, the night sky, the aurora and the light. Also the unmerged `look-afterrain` (the rainbow, torch-lit fog).
- **Engine and mod code:** the engine's particles and emission (`_e` textures), `Client::spawn_particles`, the air flags, `scene.options`, and `mods/base/src/video.rs` (the Video screen and its presets).
- **Pack sources, read for ideas only:** Bliss, Photon, Complementary (Reimagined and Unbound), Rethinking Voxels, Solas (a pack that calls itself "fantasy stylised") and Lux (BSL-based). For Euphoria Patches only the option names and descriptions are public (its shader code ships encrypted), so its defaults are unverified.
- **Games:** art-direction talks and wikis for Zelda BotW and TotK, Genshin, Valheim, Vintage Story, Hytale, Terraria, Minecraft Dungeons and Legends, Wynncraft, and Sky: Children of the Light.
- **Costs:** round 16's PC numbers (`side-session-notes/.../r16-gpu-costs.md`: RTX 5090 at 3456×2234; Ronald's Mac at its auto 0.67 scale is about ×2.25 of them).

**Licences:**
- Bliss, Complementary, Solas and Lux are all rights reserved.
- Photon allows learning but not reuse in a paid game.
- So everything below is an idea to write anew, never their code.

## The short answer

1. **We already have the base most fantasy packs build on**, so this round is about mood, not machinery:
   - a golden hour, an aurora by latitude, a Milky Way and coloured stars;
   - canopy shafts and gold dawn mist;
   - a pixel rainbow and torch-lit fog (on `look-afterrain`);
   - bloom and emissive textures (`_e`).
2. **What makes it read "fantasy" rather than "cartoon"**, which every source agrees on:
   - **Gate the magic behind rules.** Night only, near water, in summer, in the north, after rain. Nothing glows everywhere all the time. Vintage Story's aurora appears only where the yearly mean is under −5 °C. Minecraft's fireflies need a firefly bush near water at light ≤ 13.
   - **Keep the glow dim and the light pockets small.** Vintage Story's glow worms are light 7 and Minecraft's firefly bush is 2. Sky: Children of the Light gives one focal light and keeps the rest "visually quieter".
   - **Make rare things rare and announced.** BotW's Blood Moon lasts about 30 minutes. Terraria's meteor shower comes about 1 night in 10. Vintage Story warns of temporal storms 8 hours and 29 minutes ahead.
   - **Grade from the light, not from a saturation knob.** BotW tuned weather × hour × colour into 1,440 variations and avoided anything older players would find "childish".
   - **Keep effects at the world's pixel density.** Minecraft Dungeons reused fewer than 150 sprites, made its circles hexagons, and regretted faking bloom with painted gradients.
3. **The top six (below) cost almost nothing on the GPU.** Each is under 0.1 ms on the PC and under 0.25 ms on the Mac. What they need is art (glowing plants, crystals, fireflies), a little base-mod code (particles, a sky-event schedule), and settings. Only two need a small engine value.

## What each pack and game does

| effect | who does it, and how | what it adds | cost (PC 5090 at 3456 / Mac) | ours now | needs | fits our pixel style? |
|---|---|---|---|---|---|---|
| **Golden, teal and lavender hour** | Solas: orange key light (255, 114, 72)×1.6 with a **lavender ambient** (205, 184, 236)×0.35 at sunrise and sunset, sky blue by day, blue (88, 152, 254) at night (`lib/common.glsl:473-531`). Photon: teal saturation +0.10 (`GRADE_TEAL_SAT_BOOST`). Bliss: a 15000 K blackbody moon and a fake Purkinje shift (tint 0.4, 0.7, 1.0), which make its blue nights. Genshin keeps a warm hue shift in its tone LUT. | Dusk turns violet-gold in the shade instead of grey-orange: the most "storybook" change for nothing | 0 | partial: golden hour is orange; shade is a cool blue at golden hour, no lavender; nights by moon phase | pack only | yes: palette numbers only |
| **Magical night sky** | Complementary: `NIGHT_NEBULAE` (off by default), FBM in three colours (blue, amber, cyan), different each night; stars snapped to a 1024 grid, pixel-crisp. Solas: Milky Way 2.5, a generated nebula 1.5, **shooting stars on** (chance 0.2, up to 6, trail 0.6). Euphoria: nebula colours, `PIXELATED_NEBULA`, twinkling stars, sky below the horizon. Photon and Unbound: big round sun and moon (moon 3°; Unbound about 4°). | A night sky worth looking up at | nebula ~0.05 ms / ~0.1 ms, sky pixels, nights only; shooting stars ~0 | partial: a faint Milky Way (0.03), coloured twinkling stars, no nebula colour, no shooting stars, moon 23° box (texture) | pack only (day number from `scene.light.z` ticks) | yes if pixel-snapped (our aurora already is) |
| **Aurora palettes and aurora storms** | Photon: 13 two-colour palettes, one per night by a hash of the day (green-blue, red-purple, magenta, pink, orange…), rare (1 night in 5 in snow biomes), lighting clouds 0.40 and ground 0.10. Solas: a nightly Kp index (+3 in snowy biomes). Euphoria: 14 named presets, random chance 10-90 %. Complementary: pixelated in Reimagined. Vintage Story: only under −5 °C yearly mean. | Each aurora night different; a rare storm night reaches further south and lights the snow | ~0 | partial: one green-to-purple curtain by latitude, no ground light | pack only; aurora storms via the sky-event value (below) | yes (ours is pixel-snapped) |
| **Fireflies** | Vintage Story: blinking clusters over grassland at night. Minecraft 1.21.5: firefly bush (light 2), particles at night near water, light ≤ 13, within 5 blocks. Zelda: fireflies at night, catchable. No shader pack draws them; packs only restyle particles. | Summer nights alive; the single most "fantasy" thing in the sources | particles: <0.02 ms; spawning on the CPU, trivial | no | **base mod only**: `Client::spawn_particles` each frame near the player by season, hour, biome, water and light; a fullbright blinking particle (`particle.wgsl`) | yes: 1-2 texel squares at the world's density, blinking, no soft sprites |
| **Motes and pollen in sunbeams** | BotW: "air particles catch sunlight… glints in the air". No pack has it (Bliss's patchy fog gives broken beams instead). | Forest shafts and meadows feel enchanted by day | <0.02 ms | no (shafts and canopy mist yes) | base mod particles; the pack's `particle.wgsl` fades each mote by `sun_reaches()`, so they show only inside beams | yes |
| **Glowing plants and mushrooms** | Hytale: glowing plants in the swamp zone, glowing mushrooms in sheltered pockets. Valheim: glowing Yggdrasil roots. Terraria: the mushroom biome. Vintage Story: glow worms at light 7. Euphoria: emissive flowers "only when dark or raining". Complementary: glow lichen glows only where sky light is low. | Pockets of light in dark forests and caves: landmarks | 0 (emission and bloom already exist) | no (`testmod` has glow flower, moss and leaves as examples) | art (`_e` masks), a few blocks with light 2-7 (base mod); the pack scales emission at night | yes, with `_e` masks at 32 px |
| **Crystals and ore flecks in caves** | Complementary: ores glow only in the Unbound style, flecks only (a colour-ratio mask), per ore; amethyst glows. Solas: emissive emerald, diamond, lapis and redstone on, copper, gold and iron off (strength 5). | Caves sparkle with what's worth mining; a reason to carry a torch and look | 0 | no | `_e` masks for gem ores and crystal blocks (art); a setting for strength | yes, flecks only |
| **Enchanted fog by biome** | Bliss: swamp green (0.15, 0.25, 0.05) dense, jungle green-blue, dark forest blue. Photon: Rayleigh colour and density per biome (swamp 0.0014, pale garden 0.03). Solas: light tinted per biome (cherry pink ×(1.095, 0.925, 1.025), swamp ×(1.025, 1.285, 0.785)). Valheim: a named fog per biome. Wynncraft: fog and skybox per region. Vintage Story: liquids and fog tinted by climate. Warning: Valheim's dense purple Mistlands fog drove players to mods. | Each place has its own air | 0 (the mist march is already paid where there's fog) | partial: fog amount by biome and hour (`vitals::air`), canopy mist; colour is the same everywhere | a biome tint value: a small engine change to pass the 8 free air-flag bits (24-31) into the scene, or a pack option | yes, but restrained: tints, not purple soup |
| **Sparkle on snow, sand and water** | No pack has a dedicated snow glitter. Euphoria: `GLITTER_RAIN`, pixel `SNOW_NOISE`/`SAND_NOISE`. Unbound water: a sharp sun glint. BotW: "snow should be detailed". | Fresh snow and sun on water glitter as you move | ~0.02 ms / 0.05 ms | water yes (GGX sparkle); snow and sand no | pack only: a texel hash × view angle × sun on snow and sand tops | yes, one texel per glint |
| **Dreamy bloom and halos** | Bliss: `BLOOM_STRENGTH 1.0`, **`BLOOMY_FOG 1.5`** (bloom scattered by fog: a big part of its dreamy look). Solas: bloom per dimension. Dungeons: "I wish I just leaned towards using bloom". Lens flare: Solas on (0.6, anamorphic); Complementary warns it is costly. | Lanterns and torches wear soft halos at night and in mist | +0.05-0.1 ms / +0.1-0.25 ms | partial: bloom on bright pixels (3 rings, 24 taps), on/off | pack only (`bloom.wgsl`; fog amount from the scene) | yes: halos are soft light, not pixels |
| **Sky colour by season** | Bliss: seasons tint the biome colour (autumn leaves (1.8, 0.9, 0.7), winter purple-grey). Euphoria: seasons, desaturation in autumn and winter, spring flowers. No game source documents a season sky. | Autumn evenings amber, winter skies pale and clear, spring fresh | 0 | partial: seasons change the sun's path and so the palette; foliage tints by season exist | a season value in the scene (the air flags, as above) | yes |
| **Rare sky events** | BotW: Blood Moon (red moon, red sky, about 30 min, random). Terraria: Blood Moon (a red filter) and meteor showers (about 1 night in 10, fallen stars to pick up). Vintage Story: temporal storms (rust fog, warned ahead), eclipse, supermoon. Euphoria: `BLOOD_MOON` every N nights, never on the first or at new moon, red flowers glow. Stardew: Green Rain once a summer. | Nights players talk about | ~0 | no (the aurora only) | base mod schedule (server, same on every machine) plus a scene value (air flags) plus chat or advancement hints; gameplay effects are a separate decision | yes if rare |
| **Will-o'-wisps** | Valheim: wisps at night in the Mistlands; Wisplights push mist back. | Swamps at night feel haunted | particles ~0 | no | base mod particles | risky: too "magic" for real-place biomes unless rare |
| **Bioluminescent shores** | Vintage Story: plankton glows round the swimmer in the far south. | Warm seas at night glow where you swim | particles ~0 | no | base mod particles in warm seas at night | yes, rare and local |
| **Tilted sun, big moon** | Unbound and Solas: sun path −40°; Photon moon 3°, Unbound about 4°. | Long light all day; a moon that dominates the night | 0 | done: path leaned 35°; moon 23° wide box | a setting for moon size (sky body `.size`) | yes |
| **Palette snap, world curvature, chromatic aberration** | Euphoria: `PALETTE_SWAP` to 6 colours, `WORLD_CURVATURE`. Packs leave chromatic aberration off. | Gimmicks | | no | | no: off-style |

## Recommendations, in order

Costs are on the PC (RTX 5090 at 3456×2234) and then on the Mac (about ×2.25 at its auto 0.67 scale), estimated as in round 16. Bench each on Tester 2's queue.

**1. Fireflies on summer nights, motes in sunbeams by day.** Owner: base mod (a worker); art: the Chiacraft art session.
- **What it adds:** the strongest fantasy signal in every source, and it makes a place feel alive.
- **Cost:** under 0.02 ms on both.
- **Needs:** mod and pack only, no engine change.
- **Slices:**
  1. A client system in base (`client_frame`) spawns fireflies with `Client::spawn_particles`:
     - 20-60 within 24 blocks of the player;
     - from dusk to about 02:00, late spring to early autumn, over grassland, wetland and forest edges, more near water;
     - none in rain or below about 10 °C (`vitals::air`, the climate tables);
     - at most a cluster every few blocks; they blink in groups (a `particle.wgsl` snippet: on for 0.3-1 s, off for 1-3 s), drift slowly and give no light.
  2. A firefly look: a 1-2 texel yellow-green square, fullbright, at the world's pixel density.
  3. Motes by day in forests and meadows: slow, pale, a texel each. The pack's particle shader multiplies their alpha by `sun_reaches()`, so they show only inside the beams that `volume.wgsl` and the canopy mist already draw.
  4. Later, the same system gives:
     - will-o'-wisps (rare, wetland nights);
     - bioluminescent shores (warm seas at night);
     - pollen (spring meadows);
     - falling leaves (round 8's change 4).

**2. A storybook hour: lavender shade at dusk, blue moonlight at night.** Owner: Lighting.
- **What it adds:** big for nothing. Every dusk and night shot changes.
- **Cost:** 0.
- **Needs:** pack only.
- **Slices:**
  1. In `palette()`, at golden hour and blue hour the **ambient goes lavender** (about Solas's 205, 184, 236 at a third of the key light) while the key light stays orange. Blue hour (the sun just under the horizon) gets a violet zenith.
  2. At night, moonlight from a high-temperature blackbody (Bliss 15000 K) and a gentle **Purkinje shift**: dim, colourful pixels slide toward a blue-grey (Photon 0.05 × setting, off in block light). This keeps "nights dark by design" and makes them silvery instead of muddy green.
  3. A strength setting, so Ronald can take it from a hint to a storybook.

**3. A magical night sky.** Owner: Lighting.
- **What it adds:** big on clear nights.
- **Cost:** about 0.05 ms on the PC and 0.1 ms on the Mac, nights only, on sky pixels.
- **Needs:** pack only.
- **Slices:**
  1. **Nebula colour in the Milky Way.** Two or three soft colour clouds (blue, violet, a little amber) along our band, pixel-snapped like the aurora, different each season. Stronger than today's 0.03 band, but still fainter than the moon.
  2. **More, crisper stars:** snapped to a grid, the brightest few larger, a denser band in the Milky Way.
  3. **Shooting stars:** about one a minute on a clear night (Solas: chance 0.2, trail 0.6), a short pixel line.
  4. **An aurora palette per night:** a hash of the day picks one of about 8 two-colour pairs (green-teal, green-violet, rose-violet, rare red). Aurora nights light the snow a little (Photon's ground 0.10).
  5. A **moon-size** setting (1×, 1.5×, 2×), through the sky body's `.size` in the base mod.

**4. Glow in the dark: plants, mushrooms, crystals, ore flecks.** Owners: art (the `_e` masks), the base mod (blocks and light levels), Lighting (night scaling).
- **What it adds:** big in caves and dark forests; landmarks and a reason to explore.
- **Cost:** 0 (the engine already draws `_e` emission and our bloom picks it up).
- **Needs:** art and base content; pack only for the scaling.
- **Slices:**
  1. **Gem ore flecks:** `_e` masks on the gem pixels only (diamond, emerald, lapis; not iron, coal or gold, as Solas does). The pack scales emission by darkness, so they glint in a dark cave and are plain by day.
  2. **Two or three glowing plants**, at light 2-5 and only where sky light is low or at night:
     - a glow mushroom in damp forests and caves;
     - glow lichen on cave walls;
     - a night-blooming flower in wetlands.
     - Each needs a real-world excuse, such as foxfire fungus (*Panellus*, *Armillaria*) or bioluminescent mushrooms (*Mycena*).
  3. **Crystal geodes,** if worldgen adds them: emissive cores at light 4-6.
- **Restraint:** dim, few, never in open daylight.

**5. Enchanted air by biome and season.** Owners: Lighting (pack), the base mod (values), plus a small engine change.
- **What it adds:** medium-big. Each place gets its own air: wetland mist green-grey, rainforest teal, taiga blue at dawn, desert amber dust, autumn evenings amber, winter skies pale.
- **Cost:** 0. It recolours the mist and haze already drawn.
- **Needs:** a biome mood and the season in the scene. The air flags have 8 free bits (24-31): the base mod puts a mood index (4 bits) and the season (2 bits) there, and the engine passes them into `scene`, as it does for canopy and aurora. One `ENGINE_CHANGES.md` line.
- **Slices:**
  1. The engine field.
  2. A small table of fog, haze and light tints per mood in `look.wgsl` (Bliss's and Photon's per-biome numbers as the start, toned down).
  3. A season tint on the sky palette.
- **Restraint:** tints of under about 15 % except the wetland mist. Valheim's dense purple fog drove players to mods.

**6. Rare sky events: meteor showers, a blood moon, aurora storms.** Owners: the base mod (schedule, hints), Lighting (looks).
- **What it adds:** big moments, rarely.
- **Cost:** ~0.
- **Needs:** a base-mod schedule (seeded by world and day, the same on every machine, like `weather.rs`), sent through the same air-flag bits or one more scene value.
- **Slices:**
  1. **A meteor shower** about 1 clear night in 15: shooting stars ×20 for an hour, announced at dusk ("The sky is restless tonight").
  2. **An aurora storm** about 1 night in 30: the aurora reaches mid-latitudes, brighter, lighting the land.
  3. **A blood moon** about 1 night in 20 (never the first night or a new moon): a red-orange moon and a faint red cast on the night.
- **Decision for Ronald:** whether events touch gameplay (more hostile mobs on a blood moon, fallen stars to pick up after a shower) or stay looks only.

**After the six, smaller:**
- sparkle on fresh snow and sand in sun (a texel hash, ~0.02 / 0.05 ms);
- a "dreamy" bloom level with halos round lights at night and bloom scattered by fog (Bliss's bloomy fog; +0.05-0.1 / +0.1-0.25 ms);
- bioluminescent shores.

**Not recommended:** palette snapping, world curvature, chromatic aberration, lens flare (costly and off-style), and dense purple fog.

## Settings: every part on or off (Ronald: "the option to turn on off parts")

**Where they go.** A **"Fantasy" group** on the Video screen (`mods/base/src/video.rs`), under the quality rows. Its rows are in the same ‹ value › style as the rest, and its first row is a **master** that sets the others the way a quality preset does:
- Off;
- Subtle (the default);
- Storybook;
- Custom (as soon as a row is changed).

**Quality presets and the master do different jobs:**
- The **quality** presets (Low, Medium, High, Ultra) touch only the rows that cost something: particle counts, nebula detail, dreamy bloom.
- The **Fantasy** master sets the taste rows.

| setting | values | default | Subtle | Storybook | quality presets | what it controls |
|---|---|---|---|---|---|---|
| `base:fantasy` (master) | off, subtle, storybook, custom | subtle | | | none | sets the rows below |
| `base:fx_fireflies` | off, few, many | few | few | many | Low: few; Medium and up: as the master | fireflies on summer nights (rec. 1) |
| `base:fx_motes` | off, on | on | on | on | Low: off | motes in sunbeams, pollen (rec. 1) |
| `base:look_storybook` | 0-1 | 0.4 | 0.4 | 1.0 | none (free) | lavender dusk, blue hour, silver moonlight (rec. 2) |
| `base:fx_night_sky` | off, subtle, vivid | subtle | subtle | vivid | Low: subtle (fewer octaves) | nebula colour, star density (rec. 3) |
| `base:fx_shooting_stars` | off, on | on | on | on | none | shooting stars (rec. 3) |
| `base:moon_size` | 1×, 1.5×, 2× | 1× | 1× | 1.5× | none | the moon's size (rec. 3) |
| `base:fx_glow` | 0-1.5 | 1 | 1 | 1.5 | none | how bright glowing plants, crystals and ore flecks are at night (rec. 4); 0 makes them plain |
| `base:fx_biome_air` | 0-1 | 0.6 | 0.6 | 1 | none | fog, haze and light tints by biome and season (rec. 5) |
| `base:fx_sky_events` | off, looks only, on | on | on | on | none | meteor showers, blood moons, aurora storms (rec. 6). "Looks only" leaves gameplay out. This is the one row that is also a world rule if events change gameplay: then a `/rule`, not a video setting |
| `base:fx_sparkle` | off, on | on | on | on | Low: off | glints on snow, sand, water |
| `base:fx_bloom` (exists, on/off) | off, soft, dreamy | soft | soft | dreamy | Low: off; Medium: soft; High and Ultra: as the master | halos and bloomy fog |

**How the pack reads them.** The pack reads settings through `scene.options`, which holds 16 values, and 15 are taken (the 14 look knobs plus `core:view_distance`). Two ways out:
- **(a) Raise `options` to 32 values in the engine.** One `gpu_layout.rs` line and a `notes/ENGINE_CHANGES.md` line. Recommended: simple, and room for later knobs.
- **(b) Pack the on/off and 3-level rows into the one free slot,** as 2-bit fields of a whole number (exact in an f32 up to 12 fields).

Rows that only the base mod reads (fireflies, motes, moon size, sky events) need no slot at all. Strings go in `video.base.*` keys, in English and Russian.

## Who takes what

- **Lighting:**
  - **now:** rec. 2 (pack only).
  - **then:** rec. 3, the pack sides of 4 and 5, and the snow sparkle.
- **A base-mod worker (new, or First hours):**
  - rec. 1 (the firefly and mote system);
  - rec. 6's schedule;
  - the settings rows.
  - The firefly and mote system touches no engine code.
- **The Chiacraft art session:** firefly and mote particles, `_e` masks for the gem ores, two or three glowing plants. Follow the `game-art` skill; plants via `block-textures`.
- **Engine (after the upstream sync):**
  - `scene.options` to 32;
  - the air-flag bits 24-31 into the scene (biome mood, season, sky event).
  - Each needs its `ENGINE_CHANGES.md` line.
- **Ronald decides:**
  - how far the master goes by default (Subtle or Storybook);
  - whether sky events touch gameplay;
  - the moon size.

## Pictures

**The sheet:**
- Full size: `~/Coding/chiacraft-archive/images/research-fantasy/2026-11-05/fantasy_sheet.jpg`, with its spec beside it.
- A small copy: `reports/fantasy/fantasy_sheet.jpg`.

**Columns:** ours on dev (Tester 2's PC runs; nothing new was run), a tasteful fantasy reference, a second tasteful reference, and **"too far"**: what to avoid.

**Rows, one per recommendation:**

| row | ours | tasteful references | too far |
|---|---|---|---|
| 1. Fireflies | an empty dark night | Complementary's pixel fireflies over a meadow; Visual Effects+ in a swamp | neon sparkles |
| 2. Storybook hour | an orange sunset with grey-blue shade | Unbound's honey-gold haze with lilac clouds; Valheim's warm winter haze | an orange flood with magenta clouds |
| 3. Night sky | a faint band, a green night | Genshin's dust-fine Milky Way; a pastel aurora | neon aurora fans |
| 4. Glow in the dark | a cave lit by the torch alone | Hytale's glowing mushrooms by a moonlit pond; Unbound's ore flecks | every ore lit in a black cave |
| 5. Enchanted air | gold dawn shafts in a rainforest | Valheim's pink-to-blue sea mist; Snowimagined's lavender winter | a flat pastel tint |
| 6. Rare events | none | BotW's blood moon as a grade; one shooting star at a pink dusk | a flat red sky |
| 7. Halos and sparkle | torch light in dusk mist (`look-afterrain`) | Hytale's campfire pooling light; Photon's sun glitter | twinkle sprites everywhere |

**The 70 references** are in `~/Coding/chiacraft-archive/images/research-fantasy/2026-11-05/refs/`, with one line each in `INDEX.md`: the look, whether it reads tasteful or cartoonish, and its source page. They are internal reference only; the images belong to their authors. Gaps:
- Terraria's wiki blocked downloads.
- Zelda and Genshin are YouTube thumbnails only.
- There is no good glittering-snow shot.
- The Valheim swamp picture is painted key art.

## Sources

- **Pack sources** (read for ideas only; file:line cited in the tables):
  - Solas: https://github.com/Septonious/Solas-Shader, "All Rights Reserved unless otherwise explicitly stated".
  - Lux: https://github.com/Aurealis-Graphics/Lux-Shader.
  - Euphoria Patches: https://github.com/EuphoriaPatches (language files only).
  - Complementary, Photon, Bliss and Rethinking Voxels as in round 15.
- **Web only:**
  - https://modrinth.com/shader/fantasy-shader-unbound
  - https://modrinth.com/shader/makeup-ultra-fast-shaders
  - https://modrinth.com/shader/nostalgia-shader
- **Zelda:**
  - CEDEC art talk: https://www.thefamicast.com/2017/12/part-5-art-and-implementation-of-3d.html
  - GDC 2017: https://www.thumbsticks.com/gdc17-designing-zelda-breath-of-the-wild/ and https://zeldauniverse.net/2017/03/14/takizawa-explains-how-the-wind-waker-hd-inspired-the-visual-style-for-breath-of-the-wild/
  - Blood Moon: https://samurai-gamers.com/legend-zelda-breath-wild/blood-moon-mechanic/
  - TotK sky islands: https://gamedeveloper.com/production/zelda-devs-originally-made-sky-islands-look-like-specks-of-trash
  - Lightroots: https://nintendoeverything.com/zelda-tears-of-the-kingdom-devs-explain-raurus-seal-shrines-lightroots/
- **Genshin:** https://nugglet.github.io/posts/2022/12/console_graphics_rendering_pipeline_genshin_impact
- **Valheim:**
  - https://www.pcgamesn.com/valheim/ps1-graphics
  - https://thunderstore.io/c/valheim/p/Azumatt/Foglands
  - https://pcgamer.com/valheim-wisp-fountain-torches-blue-lights-mistlands/
- **Vintage Story:**
  - https://www.vintagestory.at/blog.html/news/ambient-and-sky-r7/
  - https://wiki.vintagestory.at/Temporal_storm
  - https://wiki.vintagestory.at/index.php/Glow_worm
- **Hytale:** https://hytale.com/news/2019/11/hytale-graphics-update
- **Terraria:**
  - https://terraria.wiki.gg/wiki/Blood_Moon
  - https://terraria.wiki.gg/wiki/Meteor_Shower
  - https://terraria.wiki.gg/wiki/Biome_background
- **Minecraft:**
  - Dungeons VFX: https://www.vfxapprentice.com/blog/mojang-vfx-of-minecraft-dungeons
  - Spring to Life fireflies: https://www.nintendolife.com/news/2025/03/minecrafts-latest-update-is-now-live-here-are-the-patch-notes
- **Sky: Children of the Light:** https://80.lv/articles/how-to-design-emotional-game-environments-for-sky-children-of-the-light/
- **Stardew:** https://stardewvalleywiki.com/Green_Rain

---

# Addendum: sky events with gameplay (2026-11-05)

Ronald chose "Gameplay too" for sky events. The master setting defaults to Subtle and the moon stays at 1× (Planner, relayed). This addendum designs:
- **the blood moon**: more and bolder monsters;
- **meteor showers**: fallen stars to find, with meteoric iron;
- **aurora storms**: kept as looks only.

It also says where their numbers live, the rows they add to `notes/SYSTEMS.md`, and the owners.

**What was read:**
- `notes/SYSTEMS.md`, `notes/SURVIVAL.md` (steps 11, 14 and 15: the fair first night, "Campfire keeps you safe", dying and the way back) and `notes/PLAN.md`;
- `mods/base/src/mobs.rs` (the four monsters, their spawn rules, `keep_from_fire`);
- the engine's natural spawner (`crates/engine/src/server/dim.rs:1700-1790`: every 40 ticks, 4 tries per player 20-45 blocks out, light 0-6, category caps within 64 blocks; `core:hostile` cap 8 (`registry.rs:1378`); the `question::NATURAL_SPAWN` veto);
- `vitals::{calendar, difficulty, levels}`, the crafting materials (`mods/base/crafting/SHEET.md`), the craters in `base_api::land` (`CRATER_*`), and the moon (`lib.rs`: 8 phases, full on day 0 mod 8).

## The rules this keeps

These come from `notes/SURVIVAL.md` and round 17's restraint rules:
- **"Campfire keeps you safe"** holds on every night, blood moon included. Event monsters spawn only at light 6 or less and stay out of a campfire's ring (`keep_from_fire`). A blood moon is a night to sit by the fire, not a night that breaks the promise.
- **No death spiral.** The respawn grace (`GRACE_SECS` 180) and the tombstone run hold. Gentle's first three nights stay free of monsters (`grace_nights`).
- **Rare and announced.** Each event is foretold at the night hint's hour, two game hours before sunset (the same slot as `hints.rs` `night`). The sky shows it from dusk.
- **Real-world anchors**, as in the rest of the game:
  - a blood moon is a total lunar eclipse, which happens only at a full moon;
  - meteor showers come on fixed dates each year, as the Perseids and Geminids do;
  - meteoric iron is iron-nickel that people worked cold before smelting (Inuit blades from the Cape York meteorite; Tutankhamun's dagger, about 11 % nickel);
  - aurora storms come most near the equinoxes.
- **One event a night at most,** and the same on every machine: a hash of the world's seed and the day, as rain's `climate::spell` is.

## 1. The blood moon (a total eclipse of the full moon)

**When:**
- **Only on a full moon** (day mod 8 = 0).
- **Frequency:** `BLOOD_MOON_CHANCE` = 1 in 3 full moons. That is about 2-3 a year (a year is 56 days, 7 full moons), about once every 24 days, or 8 real hours of play. For comparison, Terraria's is 1 in 9 nights, which would be every 3 hours of play: too often for a survival game with long builds.
- **Not before** `BLOOD_MOON_FROM_DAY` 16 (the second full moon after a new world's start on day 7), and never in gentle's grace nights.
- **Totality:** from `BLOOD_MOON_HOURS` (22.0, 2.0). The moon reddens over the hour before and clears over the hour after, so the event runs 21:00-03:00.

**Announcement and look:**
- **Two game hours before sunset:** "The full moon will rise dark red tonight. Monsters walk boldly under it: stay by your fire." (`hint.base.blood_moon`, en and ru)
- **At dusk** the moon rises copper. In totality it is a deep red disc, and the night is as dark as a new moon's: the moon's light goes, a red cast stays.
- **Pack side:** the sky event value (air-flag bits 24-31, round 17 rec. 5) drives the pack's moon colour, the night light and a faint red grade (`base:fx_sky_events` = looks).

**Gameplay** (`base:fx_sky_events` = on and difficulty above gentle; on gentle the look only):
- **More monsters.** A second category, `base:blood_moon`, with its own cap `BLOOD_MOON_CAP` 6, on top of the hostile 8, so up to 14 within 64 blocks.
  - Its spawn rules are registered like the normal ones: light 0-6, the same biomes.
  - The base mod's `NATURAL_SPAWN` interceptor turns them down outside totality. That needs no engine change; the engine already asks.
  - The weights favour pack hunters and keep griefing low:

    | monster | weight |
    |---|---|
    | zombie | 3 |
    | spider | 3 |
    | skeleton | 2 |
    | creeper | 1 (fewer craters in bases) |

- **Bolder monsters:**
  - event monsters spawn with +20 % max health (`BLOOD_MOON_HEALTH` 1.2) and +10 % speed (`BLOOD_MOON_SPEED` 1.1), as attribute modifiers;
  - their sight reach for players is ×1.5 (`hostile(reach)` × `BLOOD_MOON_REACH`);
  - **no extra damage**: the danger is numbers and pace, not one-hit deaths.
- **Red eyes:** an emissive eye mask (`_e`) on each monster model, shown while the event flag is set (art and the entity shader).
- **Dawn:** at 03:00 the leftover event monsters walk away from players and despawn once out of sight (or at 40 blocks). Daylight ends them for good.
- **Harsh:** cap 8 and health ×1.35. **Gentle:** looks only. Both go in `vitals::difficulty` (`blood_moon_cap`, `blood_moon_health`).

**Rewards:**
- XP ×1.5 for kills in totality (`BLOOD_MOON_XP`, through `levels::xp::kill`).
- An advancement, **"Red Night"**: see a blood moon through to 03:00 without dying.
- A rare drop worth having: 1 in 8 event monsters drops one **"eclipse ash"**, a dye or ingredient for a red-black cloak dye (`clothing`). It's cosmetic, so it never becomes a grind.

**Animals (Animals mod):** herds bunch closer and run sooner during totality (`hunt::wary` × 1.5). Wolves howl once at totality (the voices system).

## 2. Meteor showers and fallen stars

**When** (`METEOR_SHOWERS`, by calendar date, every year):

| shower | dates (real equivalent) | peak night |
|---|---|---|
| summer | summer days 9-11 (the Perseids, mid-August) | day 10 |
| winter | winter days 2-4 (the Geminids, mid-December) | day 3 |

- **Peak night:** shooting stars ×20 (Solas-like streaks) from 23:00 to dawn.
- **Each side of the peak:** shooting stars ×6.
- **A lone fireball:** `FIREBALL_CHANCE` = 1 clear night in 20 brings a single bright fireball that lands.
- Rain and cloud hide the streaks but not the landings: you hear the boom.

**Announcement:**
- **Peak:** "The sky is restless tonight: stars will fall after midnight." (`hint.base.meteor_shower`)
- **A lone fireball** has no warning; that is the point of it.

**What falls: a meteorite.**
- **Landings** (`METEOR_LANDINGS`):
  - 1-2 per player on the peak night, 0-1 on a side night, 1 for a fireball;
  - each lands 48-160 blocks from the player (`METEOR_REACH`), under the open sky (sky light 15), on land;
  - never in water (over water it splashes and sinks, lost);
  - never within 24 blocks of a player-placed block or a campfire (no griefing, `METEOR_CLEAR`).
- **What the player sees and hears:** a streak across the sky toward the impact point (pack, from the event value plus the landing's place), then a boom heard after the distance's delay (`weather_sound::thunder_delay`).
- **On the ground:**
  - a scorched pit 3-5 blocks across: grass turns to coarse dirt, nothing set alight;
  - 1-3 `base:meteorite` blocks with a black fusion crust, glowing hot: light 9 fading to 0 over `METEORITE_COOLS` = 1 game hour;
  - smoke for 5 real minutes, a column to walk toward;
  - touching it hot burns, like a campfire.
- **Old impacts:** worldgen's craters (`CRATER_*`, one per 640-block square) hold buried meteorites. 1 crater in 3 has 2-5 `base:meteorite` blocks 2-6 under its floor (`CRATER_METEORITES`). So a daytime player can dig for it too: the Vintage Story idea, with no event needed.

**What it gives: meteoric iron.**
- A `base:meteorite` block mines at stone hardness with a flint-tier pick or better (`harvest` 100). It drops 1-3 `meteoric_iron` lumps (`METEORITE_DROPS`), plus stone.
- **Meteoric iron** is a new crafting material row in `mods/base/crafting` (`materials.rs`):
  - roles: working and fitting;
  - 7.9 g/cm³, harvest 200, uses 240, speed 6.5, damage 2;
  - trait `cold_worked`.
- It is **hammered cold** at a stone anvil or a hammer station, **with no furnace or bloomery**, as the Inuit worked Cape York's iron.
- **Its place in the progression:** a lucky early step to iron-tier cutting edges (a knife, an axe or a spear head) before the player can smelt. It comes in small amounts:
  - a shower night gives about 2-6 lumps a player;
  - a crater gives 4-15.
  - A head takes 2 lumps, so 1-3 heads a night: enough to matter, not enough to skip copper and bronze for armour, tools in bulk, or anything that needs ingots.
- It sits between bronze and wrought iron, below steel (VS's meteoric iron sits above iron, but there it needs an iron pick; ours rewards the night walk instead).
- **Risk and reward:** the stars fall at night, when monsters are out. Going to a smoke column after midnight is a choice, made safer with a torch and a campfire near the impact.

**Advancements:**
- **"Starfall"**: pick up meteoric iron.
- **"Wish Upon"**: be outside under a shower's peak.

## 3. Aurora storms: looks only

**When:**
- `AURORA_STORM_CHANCE` = 1 clear night in 30, about 2 a year.
- ×2 within 4 days of either equinox (spring day 7, autumn day 7), the Russell-McPherron effect that makes real geomagnetic storms likeliest then.
- Never on a blood moon or a meteor peak.

**Look:**
- The aurora reaches south of its usual biomes: full in the north as now, 0.6 in the birch forest and temperate plains, 0.2 further south. It is brighter, picks a rarer palette (red-violet, round 17 rec. 3), and lights the snow a little.
- Dusk line: "The north is glowing tonight."

**Gameplay: none.** It is a reward for being outside, not a hazard. Two events with gameplay are enough to learn.
- The only hook is an advancement, **"Sky on Fire"**: see an aurora storm.
- A later idea, if Ronald wants one: compasses, if we add them, swing on such nights, as real ones do in a geomagnetic storm.

## Where the numbers live

**A new module, `mods/base/vitals/src/events.rs`.** It belongs beside `calendar` and `difficulty`, since events are calendar dates, moon phases and a difficulty's share. It holds:
- every constant above:
  - `BLOOD_MOON_CHANCE`, `BLOOD_MOON_FROM_DAY`, `BLOOD_MOON_HOURS`;
  - `BLOOD_MOON_CAP`, `BLOOD_MOON_HEALTH`, `BLOOD_MOON_SPEED`, `BLOOD_MOON_REACH`, `BLOOD_MOON_XP`;
  - `METEOR_SHOWERS`, `FIREBALL_CHANCE`, `METEOR_LANDINGS`, `METEOR_REACH`, `METEOR_CLEAR`;
  - `METEORITE_COOLS`, `METEORITE_DROPS`;
  - `AURORA_STORM_CHANCE`, `EQUINOX_BOOST`;
- one function, `tonight(seed, day) -> Option<Event>`. It is deterministic like `climate::spell` and applies the priorities: blood moon, then a shower's peak, then an aurora storm, then a fireball.

Per-difficulty values go in `vitals::difficulty` (`blood_moon_cap`, `blood_moon_health`, `blood_moon_gameplay`).

**Checks** in `vitals::balance` (or a test in `events.rs`):
- over 100 simulated years:
  - blood moons 2-3 a year, only on full moons, none before day 16;
  - 2 showers a year on their dates;
  - aurora storms 1-3 a year;
  - never two events a night;
- the cap and multipliers keep a player in arrival clothes by a campfire unharmed (the same check as step 14's);
- the meteoric iron a night gives is under the iron a crafted tool set needs (the check against crafting's early needs, as with the faces' ores).

**Other places:**
- `CRATER_METEORITES` goes in `base_api::land` beside the other crater numbers.
- The material row goes in `mods/base/crafting` (`materials.rs`).
- After the numbers change, regenerate the sheets (`vitals --bin sheet`, `crafting --bin sheet`).

## Rows for `notes/SYSTEMS.md`

| System | Code | Numbers | Reads | Read by |
|---|---|---|---|---|
| Sky events | `vitals::events`, `mods/base/src/events.rs` (dusk lines, the event flag, landings) | `events.rs` constants; `difficulty` (`blood_moon_*`) | the calendar (date, hour), the moon's phase (day mod 8), the world's seed, difficulty and gentle's grace nights, the rain where the player stands (streaks hidden), the biomes' aurora shares | monsters (blood moon spawning and strength), the pack (air-flag bits 24-31: moon colour, night light, streaks, aurora reach), hints (the dusk lines), advancements, weather sound (the impact boom), animals (wariness, howls) |
| Monsters on a blood moon | `mods/base/src/mobs.rs` (the `base:blood_moon` category and its rules, the `NATURAL_SPAWN` veto outside totality, the modifiers, the dawn walk-off) | `BLOOD_MOON_CAP`, `_HEALTH`, `_SPEED`, `_REACH`, weights | sky events, light (0-6), campfires (`keep_from_fire`), the respawn grace | players' nights, XP (`BLOOD_MOON_XP`), eclipse ash drops |
| Meteorites, meteoric iron | `mods/base/src/events.rs` (landings), `blocks.rs` (`base:meteorite`, hot and cooling), `base_api::land` craters (`CRATER_METEORITES`), `mods/base/crafting` (`meteoric_iron`) | `METEOR_*`, `METEORITE_*`, `CRATER_METEORITES`, the material row | sky events, open sky and players' builds (where it may land), worldgen's craters | crafting (iron-tier heads before smelting), advancements, the smoke and glow (pack and particles) |

Also update the **"When you change something, check"** list with one line: *A sky event or its numbers*: `vitals::events`' checks (frequency, no two a night), the first-night check by a campfire, and crafting's early iron.

## Owners

| piece | owner | needs the engine? |
|---|---|---|
| `vitals::events` (the schedule, numbers, checks) and `events.rs` (dusk lines, the event flag, landings, the meteorite block) | a base worker: **First hours** (it owns hints, the first nights and the campfire rule), or a new "Sky events" worker | no (`NATURAL_SPAWN` and categories exist); the flag to the pack needs round 17's small engine change (air-flag bits into `scene`) |
| Blood-moon monsters (category, modifiers, dawn walk-off) | **First hours** (`mobs.rs` is base's; Planner's note named Animals, but the four monsters live in base) | no |
| Animals' wariness and howls on a blood moon | **Animals** | no |
| Meteoric iron (material row, cold working, heads) | **Survival** or whoever owns crafting (`mods/base/crafting`) | no |
| Crater meteorites | **Worldgen** | no |
| Looks: moon colour and darkness, red cast, streaks and the landing trail, hot glow, aurora storm reach and palette | **Lighting** | the same air-flag change |
| Art: meteorite block (crust, hot glow `_e`), meteoric iron lump, eclipse ash, monster eye masks | **Art** (the `game-art` and `block-textures` skills) | no |
| Strings (dusk lines, advancements, items), en and ru | each owner with its piece | no |

**Order:**
1. `vitals::events` with its checks and the dusk lines, using today's look (the existing aurora plus the shooting stars).
2. The blood moon (base monsters, then Lighting's look).
3. Meteor landings, the meteorite block and meteoric iron, then crater meteorites.
4. The aurora storm look, alongside Lighting's rec. 3.

Each step gets a PC run through Tester 2's queue: a world with `/tk time` set to a blood moon or shower night, with spawning on.

**Sources for this addendum:**
- Terraria's blood moon is 1 in 9 nights, never at a new moon, and decided at dusk: [Blood Moon](https://terraria.wiki.gg/wiki/Blood_Moon).
- Terraria's meteorites fall at about 2 % a night, with ore and Meteor Heads at the site: [Meteorite](https://terraria.wiki.gg/wiki/Meteorites).
- Vintage Story's meteoric iron comes from crash sites, needs a tier-4 pick, and outperforms iron: [Meteoric iron](https://wiki.vintagestory.at/Meteoric_iron).
- Tutankhamun's blade is meteoric iron at about 10.8-12.8 % nickel: [Eos](https://eos.org/articles/pharaohs-iron-dagger-made-from-a-meteorite-study-confirms).

---

# Addendum 2: big impacts and rare star materials (2026-11-05)

Ronald (via Planner): "what also would be super cool is if we had like meteorites which could come down from ... the space ... and then like land onto Earth, which is like sort of random. I'm not exactly sure where it would happen, but it can then bring like some new like resources onto the world which are like super rare."

This extends Addendum 1, the sky-events design Ronald approved ("Yes, as designed", dev `52ee42ba`; First hours leads the build after the sync). Addendum 1's small fallen stars from the showers stay as they are. A **big impact** is a separate, rarer event:
- a crater is carved live;
- a core of rare material is left behind;
- the whole world sees it.

**What was read:**
- the engine's block edits (`World::fill_sphere` / `fill_sphere_keeping`: creepers already carve with them, `mobs.rs` `blast`), `get_loaded`, `players`;
- base's tombstone pointer (`base_api::GRAVE`, the hotbar line "Tombstone: 42 m, ahead to the left"), `base_api::hint`;
- the worldgen craters (`base_api::land` `CRATER_*`, radius 20-30 every 640-block square);
- crafting's material ladder (`mods/base/crafting/SHEET.md`; `notes/CRAFTING.md`: "the ladder stays open at the top", diamond 400 above steel 300);
- `notes/PLAN.md` (rule 2, freedom for owners; linked worlds and inventory carry-over).

## How real meteorites anchor this

- **Most falls are stone, few are iron.** About 5 % of meteorites are irons (a commonly cited figure). Stony-irons are about 2 % of known meteorites, and of those the **pallasites** (gem olivine, *peridot*, set in nickel-iron) number under a hundred known, only four of them seen to fall ([Wikipedia](https://en.wikipedia.org/wiki/Pallasite)).
- **Iron meteorites are nickel-iron:**
  - kamacite holds about 5-12 % Ni, taenite about 12-45 %;
  - cut and etched, octahedrites show the **Widmanstätten pattern** of their interlocking plates ([Wikipedia](https://en.wikipedia.org/wiki/Widmanst%C3%A4tten_pattern), [AMNH](https://www.amnh.org/exhibitions/permanent/meteorites/building-planets/iron-crystals)).
  - Vintage Story's meteoric iron is worked into tools better than iron ([VS wiki](https://wiki.vintagestory.at/Meteoric_iron)).
- **Tektites** are not from space. They are the ground melted and thrown out by the impact: **moldavite**, green glass from the Ries crater in Germany about 14.7-15 million years ago, scattered in a fan about 75° wide ([Wikipedia](https://en.wikipedia.org/wiki/Moldavite)).
- **Big falls are seen by day.** The 1947 Sikhote-Alin iron shower fell in daylight with a smoke trail that hung for hours (from memory, not checked this round).

## 1. The event

**How rare** (`vitals::events`):
- `BIG_IMPACTS_PER_YEAR` = 3 per world. A year is 56 days, so that is about one every 19 game days, or one per 6 hours of play.
- They never come closer than `BIG_IMPACT_GAP` 10 days apart, and never before `BIG_IMPACT_FROM_DAY` 21: the end of the first season, when a player has a base to come back to.
- They are picked by the same seeded schedule as Addendum 1 (`tonight(seed, day)`), but **at any hour**, since big falls come by day too. They take priority over every other event that day.

**Where:**
- **Far wilds, but in loaded land.** The site is picked among loaded columns between `IMPACT_FROM_PLAYERS` 160 blocks from every player and their view edge, using `get_loaded`, so no unloaded or ungenerated land is touched.
- **On land with open sky:** dry, flat enough (the same tests worldgen craters use), not in water or a river.
- **Never near builds.** Never within `IMPACT_FROM_BUILDS` 256 blocks of any player-placed block, campfire, bed or tombstone, and never within 256 of the world spawn.
  - The base mod keeps a coarse map of built columns: a bitset per 16×16, set on `BLOCK_PLACED` by players and saved with the world.
- **If there's no site** (a player boxed in by builds, or only sea in reach), it falls in the sea: you see the streak, a white plume and a boom. No crater and no loot this time, and the next one is not delayed.

**How you notice:**
1. **Before** (T−20 s): the sky brightens toward its side, the low rumble starts, and animals go wary (Animals).
2. **The streak** (T−6 s to 0): a bright head with a long trail across the whole sky, seen from anywhere in the world.
   - By night it's a fireball brighter than the moon.
   - By day it's a white-hot spark with a **smoke trail that hangs for minutes** (Sikhote-Alin's).
   - Lighting draws it from the event value plus the impact's position (below).
3. **Impact:**
   - a white flash lights the horizon in that direction, and a dust ring rises;
   - **the boom** comes after the distance's delay (`weather_sound::thunder_delay`, 343 m/s, so 3 s a kilometre);
   - near (under 500 blocks) the ground shakes: a short camera shake.
4. **After:**
   - the crater's hot core glows (light 12 fading to 0 over `IMPACT_HOT` 2 game hours);
   - a **smoke column** rises for 10 real minutes, seen over the land;
   - at night an orange glow sits on the horizon above it.
5. **Pointer (the compass hint):**
   - at impact every player gets a line: **"A star has fallen, far to the north-east."** It names the 8-point direction from that player and a band: near under 500 blocks, far under 1.5 km, very far past that;
   - then the tombstone pointer's hotbar line reads **"Fallen star: 1,240 m, ahead to the left"** for one game day, or until the player is within 16 blocks;
   - a setting (`base:fx_sky_events`) can hide the pointer for those who'd rather search by the smoke.
6. **Rumour:** at the next dawn, a player who hasn't reached it gets a hint: *"The fallen star still smokes to the north-east."*

**Does the crater form live? Yes, at a modest size.** Worldgen's craters are radius 20-30, ancient and filled with buried meteorites (Addendum 1). A live impact carves a **fresh crater of radius `IMPACT_RADIUS` 7-12** (cost below), spread over `IMPACT_CARVE_TICKS` 20 ticks:
1. The bowl: `fill_sphere_keeping` a sphere sunk to `IMPACT_DEPTH` 0.6 radius, keeping liquids and unbreakables.
2. The rim: a ring raised 2-4 blocks of the local ground and broken stone (suevite, the impact breccia).
3. Melt and ejecta:
   - the bowl's lining (1 block) turns to **impact glass**;
   - sand or gravel within 1.2-2.5 radii turns partly to tektite pieces, in a fan downrange of the streak's heading (moldavite's 75° strewn field);
   - trees inside 2 radii are knocked flat, as fallen logs, using felling's `felled` pieces;
   - grass turns to scorched dirt, with no fire spread.
4. The core: at the bottom, under 1-2 blocks of broken stone, the meteorite mass (below), hot.

## 2. What it brings: four materials found only from the sky

| material | what it is (real) | found | for | place on the ladder |
|---|---|---|---|---|
| **Meteoric iron** (exists, Addendum 1) | iron-nickel, worked cold (Inuit, Cape York) | shower falls 1-3; worldgen craters 4-15; a big impact's ejecta 6-12 lumps round the rim | iron-grade heads before smelting | early, between bronze and wrought iron |
| **Star steel** (Widmanstätten nickel-iron, an octahedrite) | the big impact's iron core: kamacite and taenite plates that show the pattern when etched | the core: `STAR_STEEL_CORE` 8-16 lumps per big impact, so 2-4 tools' worth | tools, weapons, armour: a **new tier between steel (300) and diamond (400)**: harvest 350, uses 650, speed 8.5, damage 3.2, trait `patterned` (the etched look on the tool, no rust, repairs at half cost); fittings 500 uses | late: needs the forge (iron age) to work it, so a reason to travel when you can use it; armour pieces take 6-8 lumps |
| **Peridot** (from a **pallasite**) | olivine gems in nickel-iron; under 100 known pallasites | a quarter of big impacts are pallasites (`PALLASITE_CHANCE` 0.25): 3-8 peridot plus a few **pallasite** blocks (green crystals in metal, translucent) | inlays on tools and armour (`CRAFTING.md`'s inlays: looks and trade value), jewellery, a stunning decor block | any age; rare goods for trade between linked worlds (`PLAN.md` carry-over) |
| **Tektite glass** (moldavite) | the ground melted and thrown out by the impact | 12-30 pieces in the strewn fan round each big crater, plus the bowl's impact-glass lining | a working material like obsidian: knife and arrow heads, harvest 100, uses 50, speed 3.2, trait `razor`; green glass panes and blocks; a lens (spyglass, later) | early-mid: no tools needed to pick up, a sharp edge for a newcomer who reaches a crater |
| **Starheart** (the fantasy one) | none: a fragment of the core that never cools, faintly blue-white | 1 in 3 big impacts (`STARHEART_CHANCE` 0.33), one piece at the very centre | a light that needs no fuel: **star lantern** (light 12, no warmth, never goes out, cold to touch); a beacon block that shows on the far land at night; held, it lights like a torch (the pack's `hand_light`, a cool tint) | rare reward, not power: light only, so it doesn't break survival's fire and warmth loop (a campfire still warms and keeps monsters off; a star lantern only lights) |

Why these fit the game:
- **Three grounded, one fantasy.**
  - The Starheart is round 17's fantasy feel: dim, rare and useful as light, never a weapon.
  - Star steel gives the open ladder its first rung above steel from a story, not a deeper ore.
  - Peridot and tektite give decoration and trade, which linked worlds will want.
- **Each impact rewards a different stage.** A newcomer can take tektite edges and meteoric iron from the rim. A late player digs out the star steel core.

## 3. Multiplayer

- **Everyone in the dimension sees it:** the streak and flash for all, the boom by each player's distance, and the direction line from each player's own position.
- **Who gets it:** whoever reaches it. There are no claims, which keeps owners free (`PLAN.md` rule 2). To keep a server from racing to a single prize:
  - the loot is many pieces: lumps round the rim, glass in the fan, the core in a dozen lumps;
  - the core scales with players online within 2 km: ×1 alone, +40 % per extra player, up to ×2.2 (`IMPACT_SHARE`).
- **The world owner chooses** with a world rule, `base:sky_impacts`:
  - 0 off;
  - 1 looks only: the streak and boom, the sea plume, no crater;
  - 2 on: the default.
- **Multiplayer announcement:** the direction line goes to every player in the dimension. There is no chat spam: one line each, plus the pointer.
- **Achievements:** "Starfall" (meteoric iron) exists in Addendum 1. "Heart of a Star" (find a Starheart) could be one of the achievements anchored on chain (`PLAN.md`: achievements on Chia), since it is rare and earned.

## 4. Risks

- **Griefing and builds.** Never within 256 blocks of anything a player built or of the world spawn. Never in unloaded land. A player can't call one down: it is the world's schedule. With `base:sky_impacts` an owner can turn it off.
- **Players hurt:** none. The site is at least 160 blocks from everyone, so the blast damage (`blast`'s rule, radius ×2) reaches no one. A player who walks into the hot bowl takes campfire-like burns until it cools.
- **Performance of a live carve.** A radius-12 bowl removes about 4,300 blocks (a cap of a sphere) and sets about 600 more (rim, lining, glass, core). That touches about 9-16 engine sections (16³).
  - Spread over 20 ticks, that is about 250 edits a tick. Creepers already do 113-block spheres in one tick.
  - Remeshing and light run on the engine's usual queues.
  - Risks: a hitch on the Mac if the crater falls in view; a light-update spike.
  - Mitigations:
    - carve only while the server's tick is under 35 ms, otherwise wait a second;
    - keep the radius at 12 or less;
    - bench it first on Tester 2's queue: frame time during a scripted impact at 200 blocks, PC and Mac.
- **Save size.** The built-columns bitset is 1 bit per 256 m², a few KB for a big world.
- **Too much loot over time.** 3 a year at 8-16 star steel lumps gives about 2-4 tools a year per world, plus diamond's ladder above. It is checked against crafting's needs, as Addendum 1's meteoric iron is.

## Numbers for `vitals::events` (beside Addendum 1's)

| number | value | note |
|---|---|---|
| `BIG_IMPACTS_PER_YEAR` | 3 | per world; a daily chance of 3/56 with the gap below |
| `BIG_IMPACT_GAP` | 10 days | |
| `BIG_IMPACT_FROM_DAY` | 21 | after the first season |
| `IMPACT_FROM_PLAYERS` | 160 blocks | out to the view edge, loaded only |
| `IMPACT_FROM_BUILDS` | 256 blocks | and the world spawn |
| `IMPACT_RADIUS` | 7-12 | live crater; worldgen's ancient ones stay 20-30 |
| `IMPACT_DEPTH`, `IMPACT_RIM` | 0.6 radius; 2-4 blocks | |
| `IMPACT_CARVE_TICKS` | 20 | and only while a tick is under 35 ms |
| `IMPACT_HOT` | 2 game hours | light 12 fading; smoke 10 real minutes |
| `IMPACT_WARN` | 20 s | brightening and rumble before the streak |
| `RIM_METEORIC_IRON` | 6-12 lumps | |
| `STAR_STEEL_CORE` | 8-16 lumps | ×`IMPACT_SHARE` (1 + 0.4 per extra player within 2 km, at most 2.2) |
| `PALLASITE_CHANCE` | 0.25 | 3-8 peridot, 2-4 pallasite blocks |
| `TEKTITE_PIECES` | 12-30 | in a 75° fan, 1.2-2.5 radii out |
| `STARHEART_CHANCE` | 0.33 | one piece |
| `POINTER_DAYS` | 1 | the hotbar line, or until within 16 blocks |

**Crafting rows** go in `mods/base/crafting` (`materials.rs`): `star_steel`, `tektite` and `peridot` (fitting, inlay). `meteoric_iron` comes from Addendum 1.

**Checks:**
- the schedule over 100 simulated years: 2-4 a year, gaps kept, none before day 21;
- star steel per year under a full tool and armour set;
- the impact site test: never within 256 of a placed block, in a game test with a built column.

## Rows for `notes/SYSTEMS.md`

| System | Code | Numbers | Reads | Read by |
|---|---|---|---|---|
| Big impacts | `vitals::events` (schedule), `mods/base/src/events.rs` (site, streak flag, live carve, the pointer and dusk/dawn lines), the built-columns map (`BLOCK_PLACED`, saved) | the table above; the world rule `base:sky_impacts` | the calendar, the world's seed, players' places and what is loaded, built columns, the land's flatness and water (`base_api::land` tests), the server's tick time | the world (a crater, glass, ejecta, felled trees), crafting (star steel, peridot, tektite, meteoric iron), the pack (streak, flash, smoke, glow: event value and position), weather sound (the boom), animals (wariness), hints and the pointer, advancements |
| Star materials | `mods/base/crafting` (`star_steel`, `tektite`, `peridot`), `blocks.rs` (`base:meteorite`, `base:pallasite`, impact glass, tektite glass, star lantern, Starheart) | material rows; `STAR_STEEL_CORE`, `PALLASITE_CHANCE`, `TEKTITE_PIECES`, `STARHEART_CHANCE` | big impacts, Addendum 1's falls and craters | tools and armour above steel, inlays and trade, light without fuel (`hand_light` when held) |

**Check line** for "When you change something": *A star material or impact numbers*: crafting's ladder (star steel stays below diamond), the yearly yield check, and the build-distance test.

## Owners

| piece | owner | engine? |
|---|---|---|
| Schedule, site picking, built-columns map, pointer and lines, world rule | **First hours** (with Addendum 1's `events.rs`) | no |
| The live carve (bowl, rim, glass fan, felled trees, core placement) | **Worldgen** (it owns the crater shapes and land tests) | no (`fill_sphere_keeping`, `set`) |
| Star steel, tektite, peridot rows; star lantern and inlays | **Survival** or crafting's owner | no |
| Streak by day and night, flash, smoke column, horizon glow, hot core, Starheart's cool light, pallasite and glass looks | **Lighting** | **small**: the impact's position in the scene (a vec4 beside round 17's air-flag bits), so the pack can draw the streak and glow toward it |
| Textures: meteorite crust, suevite, impact and tektite glass, pallasite block, star steel ingot (the etched pattern), peridot, Starheart, star lantern | **Art** | no |
| Animals' wariness before an impact; the camera shake if wanted | **Animals**; **UI** or the engine for the shake | shake: maybe |
| The boom (a deeper, longer thunder) | First hours, with `weather_sound` | no |

**Order:** after Addendum 1's events land (First hours, after the sync):
1. the schedule and site picking with the sea plume only;
2. the live carve and loot, benched;
3. the looks;
4. the materials in crafting.

## Summary for Ronald

Big impacts are rare, world-wide events, separate from the yearly showers:
- **How often:** about 3 a game year (about one per 6 hours of play), never in the first season.
- **What you see and hear:** a streak across the whole sky, a fireball by night or a smoking trail by day, a flash on the horizon, and a boom that arrives seconds later by distance.
- **Where:** they land in the far wilds of loaded land, at least 160 blocks from everyone and 256 from anything built.
- **The crater** is carved live (radius 7-12), with glass, broken rock and flattened trees round it. It smokes and glows for hours.
- **Finding it:** every player gets "A star has fallen, far to the north-east" and a pointer like the tombstone's.

They bring four materials found only there:

| material | real or fantasy | what it gives |
|---|---|---|
| star steel | real (nickel-iron with the Widmanstätten pattern) | a new tool and armour tier between steel and diamond |
| peridot | real (from rare pallasites) | gems for inlays, decoration and trade |
| tektite glass | real (like moldavite) | sharp early blades and green glass |
| the Starheart | fantasy, 1 in 3 impacts | a fragment that never cools: a lantern that never goes out |

On a server whoever gets there first takes it, but the loot is many pieces and grows with the players nearby. The world owner can turn impacts off or to looks only.
