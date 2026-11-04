# Sky, weather and small life: what the best do, and eight changes

Researcher, 2026-11-04, round 8, for Planner and Ronald. It follows `reports/world-looks.md` (round 7), which covered terrain, trees, ground and light.

**What was read:** `dev` at `7f6409ef`; the unmerged `water-look` branch; the main checkout, which has no uncommitted sky or weather work. Nothing was run.

**Decisions taken while Ronald sleeps** ("choose for us, keep it noted"):
- **Clouds:** the stylised, rounded slab (Complementary Reimagined's way), not fully volumetric clouds. It fits our blocky style and costs no new pass.
- **Aurora:** by the real place's latitude, not Vintage Story's temperature rule (a yearly mean under −5 °C).
  - The auroral oval sits at about 60–75° north, so biomes whose place is at 60° or more (`lat` in `reports/ecology/biome_climates.json`) get aurorae on clear nights. That is the taiga (Fairbanks, 64.8°, one of the best aurora places on Earth) and the snowy plains (Utqiaġvik, 71.3°).
  - The temperature rule would have left out Fairbanks, whose yearly mean is −2.1 °C.
  - The birch forest (Moscow, 55.8°) could have a rare faint one.
- **Pictures:** no new ones downloaded. This round reuses round 7's thumbnails and lists the new ones as links with their sizes. Ronald says yes or no to downloading them when he wakes.
- **Code:** ideas only, never code, from Photon and Complementary: their licences don't allow copying. Particle Rain, Falling Leaves, Cosy Critters and Ambient Birds are MIT. AmbientSounds is LGPL.

## Where we are

Some of it is good already:
- **The weather simulation:** rain by season and place, snow under 1 °C, blizzards, dust storms, dawn, sea and ice fog, and gust fronts (`mods/base/src/weather.rs`, `vitals::air`).
- **The sky:** a five-key day-to-night palette, a rose anti-sun at sunset, and ray-marched fog banks that drift with the wind and glow toward the sun (`mods/base/shaderpack/look.wgsl:55-128, 302-356`). Light shafts.
- **Rain and snow:** rain and snow strips that slant with the wind (`weather.wgsl:24`); wet ground that darkens; rain rings on water; snow lying on top faces through the shader.
- **Wind:** fields bend in gust bands (`world.wgsl:141-157, 239-245`).
- **Water:** the `water-look` branch has swells, depth colour, shore foam and a low-sun glitter path (unmerged).

The gaps, worst first:

| area | ours | file |
|---|---|---|
| **Sound** | **None at all.** No wind, rain, thunder, birdsong, crickets or water. The only sounds in the project are two music tracks, one advancement sound, a test thump and three chihuahua barks. | `mods/*/assets/sounds` |
| **Clouds** | Vanilla boxes (12×4 blocks per texture pixel). They drift along +x at a fixed speed whatever the wind (`crates/engine/src/client/sky.rs:423`). Coverage is the same in sun and storm. No cloud shadows, no second layer. | `sky.rs:410-437`, `sky.wgsl:87-207` |
| **Small life** | No fireflies, butterflies, falling leaves, pollen or insects. `ambient_particles` takes only a list of kinds (`crates/sdk/src/content.rs:1170-1186`) and spawns at a block's top centre, with no time, season, weather, biome or chance. Only the torch uses it. Particles ignore the wind (`particle_sim.wgsl:118-154`). | engine `client/mod.rs:1660-1722` |
| **Rain on the ground** | No puddles, no splashes (`core:rain` and `core:snow` particles are registered but unused, `registry.rs:2147-2170`), no streaks on walls, no drips from leaves. | `world.wgsl:339-345` |
| **Night sky** | 1,500 untextured square stars at 0.22 opacity: no twinkle, colours or Milky Way. The moon's phase doesn't change the night light. No aurora. | `look.wgsl:91-94`, `lib.rs:93-104` |
| **Storms** | Lightning is a 0.15 s flash: no bolt, no thunder. No rainbows after rain. | engine `client/mod.rs:2161-2187` |
| **Birds and fish** | Birds hop, take off and fly straight to a point at most a few blocks up (eagle 10, vulture 12). No soaring, circling or flocks; owls aren't tied to night. Fish never jump. | `mods/animals/src/birds.rs:149-187`, `fish.rs` |
| **Sky by place** | The palette reads only the sun's height and rain: the same sky over the Pantanal and the tundra. | `look.wgsl:55-81` |

## What the best do (sources at the end)

- **Clouds.**
  - Complementary Reimagined marches a slab about 8 blocks thick from a 2D cloud map at rounded coordinates, in 6–12 dithered steps. It gives soft blocky puffs in the same pass as the sky.
  - Unbound and Photon do full volumetric layers by weather, in their own low-resolution pass with temporal reprojection. Horizon Zero Dawn's budget was 2 ms; a full-resolution 128-step march at 1080p costs about 300 ms.
  - Vintage Story's clouds follow the regional weather and take sunset colours.
- **Rain on the ground.** Photon's puddles live entirely in the terrain shader:
  - a world-space noise mask, times wetness, times "faces up", times "open to the sky";
  - porous blocks darken more;
  - mirror-smooth puddles sent to reflections, with scrolling ripple normals.
  - Particle Rain adds slanted, wind-blown rain, splashes, ripples, streaks on walls, and fog after rain.
- **Night.**
  - Photon's stars hold still on a grid, twinkle, and are coloured by temperature (4,500–8,500 K). The brightest appear first at dusk.
  - Complementary's aurora is stacked ridged-noise planes, pixel-snapped in its blocky style, at night and only in the chosen biomes.
  - Vintage Story shows aurorae where the year's mean is under −5 °C, and has 8 moon phases and eclipses.
- **After rain.** A rainbow at 42° (and a faint one at 47°) opposite the sun, with strength wetness × (1 − rain) (Photon).
- **Small life.** Vintage Story has:
  - fireflies blinking in clusters over grassland at night;
  - over 160 butterfly kinds, which rest in rain and stay out of the cold;
  - cicadas from tall trees, gnats over water, water striders;
  - maple seeds twirling down.

  Cosy Critters' birds flock and scatter; moths circle lights; Ambient Birds' gulls circle over water and owls come out at night. Falling Leaves sets a spawn rate per tree.
- **Sound.** AmbientSounds plays sounds by region:
  - by biome, temperature and nearby features (trees, water);
  - with volume by the hour (birdsong 0 at 06:00, full from 08:00 to 14:00, gone by 18:00);
  - with an indoor scan that muffles rain and wind.

  Vintage Story has separate forest and open-ground wind.

## Eight changes, ranked by how much better it looks and sounds for the work

| # | change | owner | effort | engine? |
|---|---|---|---|---|
| 1 | **Clouds that follow the weather.** Drift with `vitals::air`'s wind (pass the wind into the cloud uniforms at `sky.rs:423`). More coverage and darker undersides as rain comes, with an overcast deck in storms. Then the rounded slab: 6–12 dithered steps in the sky pass from today's cloud map, with soft edges, shading and sunset colour. Cloud shadows on the ground later. | Lighting | S (drift, coverage), M (slab) | a small Weft change for the wind uniform |
| 2 | **Rain that lands.** Puddles by Photon's recipe in `world.wgsl` (noise × wetness × faces up × open sky), mirror-smooth into our reflections, with ripple normals. Splashes from the registered `core:rain` particle where rain columns meet the ground. Streaks on walls, and wet foliage. Wetness that lingers and dries after rain. | Lighting (shader), Weft (splashes) | S–M | splashes only |
| 3 | **Sound.** An ambient sound system: loops chosen by biome, hour, season and weather. Wind that grows with `vitals::air` (forest and open), rain louder in the open and muffled under a roof (an indoor scan), birdsong by day in the forest, crickets on summer nights, frogs by water, surf on beaches, thunder with its delay. Sounds from CC0 libraries or recorded, never taken from games. | Weft (the system), Chiacraft (the sounds), Voxel dog (which places sound like what) | M | yes, a small system |
| 4 | **Small life, through conditional ambient particles.** Give `ambient_particles` conditions: an hour window, season, weather, biome, light, chance, and spawning in the air above the block. Give the particle simulation the wind. Then as data: fireflies over grass and wetland on summer nights, falling leaves (more in autumn and wind, none from conifers), pollen over spring meadows, gnats over still water, snow blowing off leaves in wind. | Weft (conditions, wind), Voxel dog (data), Chiacraft (particle art) | M | yes, small |
| 5 | **The night sky.** Stars that hold still and twinkle, coloured by temperature, the brightest first at dusk. A faint Milky Way band. Night light by the moon's phase. An aurora (pixel-snapped ridged noise, green to purple) on clear nights in the taiga and snowy plains, by their places' latitude. | Lighting | S | no |
| 6 | **The sky by place and hour.** A table of fog and haze per biome and hour: humid haze in the Pantanal and llanos, crisp dry air in the desert, morning mist in valleys from a dawn Mie term with height falloff. Rain desaturates the world. A rainbow after rain, opposite a low sun. | Lighting | S | no |
| 7 | **Birds in the sky, fish that jump.** Eagles and vultures circle on thermals by day (30–60 blocks up, wide slow circles). Small birds flock and scatter. Gulls circle over the coast. Owls at night only. Fish now and then jump at dusk, with a ring on the water. Nothing new in the engine. | Wave two animals | M | no |
| 8 | **Storms with a bolt.** A lightning bolt drawn as a sky object, a sky flash lit from the bolt's side, and thunder after the distance's delay (with change 3). Snow that builds as layers on the ground stays later work (Vintage Story's way; world and engine). | Lighting + Weft | S–M (bolt), L (snow layers) | yes |

**Before all eight:** merge the `water-look` branch (Lighting's own, built and unmerged). Water then has swells, depth colour, foam and a glitter path.

**Order:**
1. Do changes 1, 2 and 5 first. They are Lighting's alone, in the shader pack, and change every shot.
2. Then 3 and 4 together: both need a small conditions system in the engine on the same data (biome, hour, season, weather). They make the world feel alive in a way no shader can.
3. Then 6, 7 and 8.

## Pictures

**Already in the repository** (round 7, `reports/world-looks/refs/`):
- `complementary_old_growth.jpg`: a sunset through spruces, light rays, reflections;
- `complementary_style_comparison.jpg`: stylised against realistic clouds and grading.

**Not downloaded, awaiting Ronald's yes** (about 12 MB; for internal reference only, since screenshots belong to their authors):

| picture | shows | size | link |
|---|---|---|---|
| Bliss, "Overcast day" | volumetric clouds by weather | 2.3 MB | [image](https://cdn.modrinth.com/data/ZvMtQlho/images/5551fc80b83c97dab65e45231ac7060f26dbbfa8.png), [gallery](https://modrinth.com/shader/bliss-shader/gallery) |
| Bliss, "Rainy weather" | storm sky and wet ground | 1.9 MB | [image](https://cdn.modrinth.com/data/ZvMtQlho/images/8eae19cb16db51436e3f027a22f2309bf043900e.png) |
| Photon, "New rainbows" | rainbow after rain | 0.5 MB | [image](https://cdn.modrinth.com/data/lLqFfGNs/images/a7eb9caed8a932e51631ca269c93d5792c11863d.jpeg), [gallery](https://modrinth.com/shader/photon-shader/gallery) |
| Vintage Story, aurora | the aurora | 0.6 MB | [image](https://wiki.vintagestory.at/images/c/c6/Aurora.PNG), [page](https://wiki.vintagestory.at/index.php/Glow_worm) |
| Vintage Story, fireflies | fireflies at night | 0.4 MB | [image](https://wiki.vintagestory.at/images/1/13/Fireflies.png) |
| Puddles & Floods | puddles with shaders | 4.2 MB | [image](https://cdn.modrinth.com/data/xeRVd78z/images/3d4b5deeb0c50c9f501c3b0c3423e8b94d92f4c8.png), [page](https://modrinth.com/mod/puddles-floods) |
| Vintage Story, snow (GIF) | snow building and melting | 1.4 MB | [image](https://media.vintagestory.at/monthly_2020_07/snowaccum.gif.57fb27cfb081bf3c5236cb34ed081463.gif) |
| Falling Leaves (GIF) | falling leaves | 1.9 MB | [image](https://i.imgur.com/Y6zGq33.gif), [repo](https://github.com/RandomMcSomethin/fallingleaves) |
| Particle Rain | fog after rain | 0.04 MB | [image](https://cdn.modrinth.com/data/nrikgvxm/images/4a4426059ebe345fb204a4915cdcf3743c8cd110.webp), [page](https://modrinth.com/mod/particle-rain) |
| Vintage Story, clouds | stylised cumulonimbus | 0.5 MB | [image](https://media.vintagestory.at/monthly_2019_12/2019-11-25_14-19-58.jpg.80f0483e100d16b6bff462f76129b466.jpg) |

**Ours:** Tester 2's fixed-spot shots at noon and sunset (round 7's request, not yet taken) give the "before" for changes 1, 5 and 6. Change 2 needs a rain run, and 4 a summer-night run, from the same spots.

## Sources

- **Photon** (source read for ideas only): [settings](https://github.com/sixthsurge/photon/blob/main/shaders/lang/en_US.lang), [stars](https://github.com/sixthsurge/photon/blob/main/shaders/include/sky/stars.glsl), [aurora](https://github.com/sixthsurge/photon/blob/main/shaders/include/sky/aurora.glsl), [rainbow](https://github.com/sixthsurge/photon/blob/main/shaders/include/sky/rainbow.glsl), [puddles](https://github.com/sixthsurge/photon/blob/main/shaders/include/surface/rain_puddles.glsl), [fog](https://github.com/sixthsurge/photon/blob/main/shaders/include/weather/fog.glsl), [licence](https://github.com/sixthsurge/photon/blob/main/LICENSE).
- **Complementary** (ideas only): [Reimagined clouds](https://github.com/ComplementaryDevelopment/ComplementaryReimagined/blob/main/shaders/lib/atmospherics/clouds/reimaginedClouds.glsl), [aurora](https://github.com/ComplementaryDevelopment/ComplementaryReimagined/blob/main/shaders/lib/atmospherics/auroraBorealis.glsl), [snow](https://github.com/ComplementaryDevelopment/ComplementaryReimagined/blob/main/shaders/lib/materials/materialMethods/snowyWorld.glsl), [waving](https://github.com/ComplementaryDevelopment/ComplementaryReimagined/blob/main/shaders/lib/materials/materialMethods/wavingBlocks.glsl), [licence](https://github.com/ComplementaryDevelopment/ComplementaryReimagined/blob/main/License.txt).
- **Cloud cost:** [Horizon Zero Dawn's clouds](https://www.guerrilla-games.com/read/the-real-time-volumetric-cloudscapes-of-horizon-zero-dawn), [measured costs](https://ar5iv.labs.arxiv.org/html/1609.05344).
- **Vintage Story:** [Stormy Weathers (1.12)](https://www.vintagestory.at/blog.html/news/the-stormy-weathers-update-v112-pre1-r230/), [Seasons (1.13)](https://www.vintagestory.at/blog.html/news/the-seasons-update-v113-pre1-r251/), [ambience](https://wiki.vintagestory.at/index.php/Glow_worm).
- **Hytale:** [weathers](https://docs.hytale.com/assets/weathers/).
- **Mods:** [Particle Rain](https://modrinth.com/mod/particle-rain), [Cosy Critters](https://modrinth.com/mod/cosy-critters), [Ambient Birds](https://modrinth.com/mod/ambient-birds), [Falling Leaves](https://github.com/RandomMcSomethin/fallingleaves), [AmbientSounds](https://modrinth.com/mod/ambientsounds) ([birdsong curve](https://github.com/CreativeMD/AmbientSounds/blob/1.21/src/main/resources/assets/ambientsounds/basic/sound_categories/bird_day.json)).
