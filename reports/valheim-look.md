# What to take from Valheim's look

Lighting, 2026-11-04, for Ronald ("valheim lighting felt really good too") and Planner. The
references are Valheim's official Steam screenshots in
`chiacraft-archive/references/shaders/valheim/` (Researcher's set, with sources). Iron Gate has
not published how it does its lighting. What is on record is the priority: composition and light
over texture detail. So the points below are read from the pictures, not taken from their code.
Ideas only, as with Bliss.

The sheet (dev, Bliss and Valheim at noon, dusk, night by a campfire and in a forest) is added
here once Tester 2's shots of dev and the Researcher's Bliss pictures are in.

## Bliss and Valheim want different things

- **Bliss** is about clarity: far views, a mirror-like water, a clean, saturated sky.
- **Valheim** is about mood: air you can see, warm light against cool shade, and fire that owns
  the night.

They don't conflict. Bliss's sky and water, with Valheim's air and fire, is the look I'd aim for.

## What the pictures show, and what we'd take

| # | Valheim does | ours now | what we'd take | where | cost |
|---|---|---|---|---|---|
| 1 | **The air glows toward the sun**: in a forest or at dusk, the half of the view toward the sun is a warm orange haze and the other half a cool blue-teal (forward scattering) | our haze is the sky's colour in that direction, nearly the same all round | a sun-side glow in the haze and fog banks, warm toward a low sun, cool away | pack (`haze_lit`, `fog_through`) | ~0 |
| 2 | **Warm light, cool shade**: at a low sun, sunlit faces go orange and shadows blue-violet | the shade is a little cool; the split is mild | a stronger warm/cool split at golden hour and dusk, through the ambient's colour and the grade | pack (`palette`, `display`) | ~0 |
| 3 | **Fire owns the night**: torches and fires throw wide, saturated orange pools on dark ground, with a big soft bloom and embers; the rest is a deep blue-teal, never black | warm block light with a flicker, a short bloom, a dark navy night | firelight warmer and wider near the source, the night's shade blue-teal, more bloom on fire, embers above campfires | pack (`block_light`, bloom); base (particles) | ~0 (a mip-chain bloom later: engine, ~0.1 ms) |
| 4 | **Air you can see in places**: forests, swamps and mornings have fog that starts within tens of metres; the open meadow at noon is clear | the haze grows with distance everywhere; fog banks on low ground | thicker haze where it belongs: inside forests, in swamps, at dawn and in rain, tinted by the biome (green in a swamp, warm in a meadow, white in snow) | pack, plus the camera's biome fog from base (a scene value) | ~0 |
| 5 | **Shafts of light through the trees** | screen-space shafts round the sun only | volumetric light through the shadow map (branch `look-volume`, already built, not yet tested) | engine + pack | ~0.5 ms PC, 2-3 ms Mac |
| 6 | **The far away pales and loses colour** while the near stays rich | haze lightens the far, but its colour stays | take colour out of what is far before the haze covers it | pack (`haze_lit`) | ~0 |
| 7 | **The weather sets the grade more than the hour**: a blizzard is white and flat, a swamp green, a clear day rich | rain greys the palette; snow and biomes don't change the grade | a grade per weather and biome: flat and bright in snow, green in a swamp, richer when clear | pack (`display`) + base (biome) | ~0 |
| 8 | **A soft vignette** that keeps the eye in the middle | none | a light vignette | pack (post) | ~0 |

## What not to take

- **Fog everywhere.** Valheim hides its distance in fog. Ours is a far-view game (the LOD terrain,
  Bliss's far views), so the open land on a clear day stays clear; fog is for forests, swamps,
  mornings and weather.
- **Its textures.** Valheim's low-resolution look is its own; our art follows our style guides.

## Order I'd do them in

Before any of these: depth (Ronald, 2026-11-04: "shadows missing as it still all seems so flat").
That means sun shadows to the horizon, AO and shade on the far land and trees, and clearer
shading toward and away from the sun. Valheim's look rests on that too.

1. **The air and the light** (1, 2, 6, 8): free, and they change every picture. With Ronald's note
   on clouds and haze, the softer, lit clouds go in this step too.
2. **Fire at night** (3): the campfire scene.
3. **Fog where it belongs** (4, 7): needs the camera's biome from base.
4. **Volumetric light** (5): test `look-volume` on the PC. If it costs more than about 0.5 ms
   there, say so before keeping it.

A sheet in the same layout after each step, as for the water.
