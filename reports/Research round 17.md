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
