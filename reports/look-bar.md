# The look Ronald wants: Bliss as the bar, in passes

Lighting, 2026-11-04, for Planner and Ronald ("the game should look like a good shader pack by
default"; his Bliss v2.1.1 settings in `chiacraft-archive/references/shaders/`). Ideas only: Bliss
and Complementary may not be copied.

## What chiacraft's pack already has

- **Sun shadows:** a 2048 map, 80 blocks around the player, from blocks, mobs and the player.
- **Contact shadows:** in corners and under overhangs.
- **Light shafts:** screen-space, round a low sun and in fog.
- **Water:** with branch `water-best`, it bends and tints what's under it, has caustics, foam where it
  meets things, Fresnel, screen reflections and a sun path. It's the same past the render distance.
- **Sky:** a day-to-night palette, rounded clouds that follow the weather, and a night sky with
  stars, Milky Way, moon phases and aurorae.
- **Air:** fog banks, wind, snow, rain, wet ground.
- **Seasons and grade:** grass and leaves by season, and a grade that warms at golden hour and keeps
  the darks from pure black.

## What Bliss has that we lack, in the order I'd add them

Costs are estimates until measured; each step gets a bench on the PC (RTX 5090) and, where
possible, the M3 Max.

| # | step | where | PC | M3 Max |
|---|---|---|---|---|
| 1 | **Tone curve and colour** (Hejl 2015 filmic, saturation about +0.25, a warmer, stronger sun): preview on branch `look-bliss` | pack | ~0 | ~0 |
| 2 | **Bloom** (sun halo, sunlit clouds, lamps, emissive glow): one 24-tap pass now, a mip-chain bloom later | pack (engine for the mip chain) | ~0.1 ms | ~0.5 ms |
| 3 | **Volumetric light**: real sun shafts through trees and fog, marched through the shadow map at half resolution; replaces the screen-space shafts | engine (half-resolution pass) + pack | ~0.5 ms | ~2-3 ms |
| 4 | **Far shadows**: a second, coarser cascade to about 400 blocks, so hills and forests seen from afar have shadows (the "no shadows" in Ronald's high views) | engine | ~0.3 ms | ~1 ms |
| 5 | **Material AO**: a wider screen-space AO (a light GTAO) on top of the corner AO, for depth in forests and between blocks | pack | ~0.3 ms | ~1.5 ms |
| 6 | **Rough reflections and wet surfaces**: puddles in rain, wet blocks, polished stone and metal in the screen reflections, blurred by roughness | pack | ~0.3 ms | ~1 ms |
| 7 | **Leaves and skin lit through** (subsurface scattering): leaves and grass glowing when the sun is behind them, with thickness from the shadow map | pack | ~0 | ~0.2 ms |
| 8 | **Haze and fog by time of day**: Bliss's bloomy fog and haze, morning mist, night fog, rain fog: the haze's distance (branch `lod-edge`) and per-hour amounts | pack | ~0 | ~0 |
| 9 | **More cloud layers and cloud shadows** on the ground | pack | ~0.1 ms | ~0.4 ms |
| 10 | **Coloured shadows** through stained glass and water | engine (a color shadow map) + pack | ~0.2 ms | ~0.6 ms |
| 11 | **Sharpening and depth of field** (Ronald's sharpening 0.4; DOF as an option) | pack | ~0.05 ms | ~0.2 ms |
| 12 | **Parallax** (POM, depth 0.05): needs height maps for our textures, an art pass first | art + pack | ~0.3 ms | ~1 ms |

Steps 1, 2 and 8 change every picture at no real cost, so they come first; 3 and 4 fix what Ronald
noticed most (no shadows from afar, flat light); 5 and 6 give depth and shine. Together 1-8 are
about 1.5 ms on the PC and 5-6 ms on the M3 Max: within 60 fps on both, with a setting to turn
the costly ones down on laptops.

## How we'll know

The four-way sheet (pack off, our default, the preview, a reference picture) at fixed spots,
repeated after each step, and a frame-time bench per step. Ronald picks between our default and
the preview before step 1 becomes the default.
