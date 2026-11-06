# Minecraft shaders compared

What the best Minecraft shader packs do that chiacraft's look doesn't, how they likely do it, and what we should take.
Ronald (11-05): "find some of the best shaders from Minecraft and do some screenshots ... comparisons ... especially ones
with Voxy or Horizon". Shot on the PC (RTX 5090), 11-05, by the MC shaders session.

**Status: draft.** Bliss is complete (all scenes and the far-view set). The other packs are being shot one held PC window
at a time; their rows fill in as they land. The technique study of all seven packs is done:
[mc-shader-techniques.md](mc-shader-techniques.md).

## How it was shot

- **Minecraft:** 26.3 with Fabric 0.20.1, Sodium 0.9.2, Iris 1.11.7, and either Distant Horizons 3.3.4 or Voxy 0.2.20 (two
  Prism instances). All from Modrinth, sha512-checked. The world is seed 1, created in creative.
- **Our side:** Tester 2's shots of dev, using the same scene names. looks4 (dev 0ced682e, after shader fixes 1–5) for hours,
  night and clouds; looks3 for rain and lightning; looks2 for mist, LOD and shadows.
- **Scenes** (103 per pack, 1920x1080, FOV 70 unless named fov30). Time and weather are held still for each shot.
  - Every 2 hours from a vista 200 blocks up and from inside a dark-oak and birch forest, plus 05, 07, 17 and 19, and dawn and
    dusk facing the sun.
  - Night with and without a held torch.
  - Weather: clear, overcast, light and heavy rain, thunder with a summoned lightning bolt, rain at dusk, rain in the forest,
    wet ground on a flat stone platform, snow and blizzard at a snowy taiga lake.
  - Mist at 05/06 from the vista, the valley, the forest and the lake.
  - Clouds looking up, at the horizon, at sunrise and sunset, and their shadows.
  - The far view at FOV 70 and 30, the near/far seam, turning, at night, in fog, in rain and at low sun.
  - Shadows on a test platform (wall, pillar, overhang, fence), leaf shadows, and where the cascade ends.
- **Far-view set** (Ronald, 11-05: "compare the quality of the LODs ... zoomed in"): vanilla render distance 6/12/24/32 with
  DH at 256 chunks, plus DH at 64 and 128. Each shot is taken with the pack on and with shaders off. FOV 30 targets: far
  forest, far peaks, far snow land, far water and shore, and the seam, with full-resolution centre crops.
- **Tools** (PC copies in `~/mcshaders`), in `~/Coding/chiacraft-archive/tester-tools/mc-shaders/`:
  - `gen_scenes.py` (scene list), `mc.py` (xdotool driver; it checks every command against the game log), `run_pack.sh`,
    `window.sh` (one held queue window), `lodwindow.sh`, `compare.py` and `lodsheet.py` (sheets).
- **Pictures:** `~/Coding/chiacraft-archive/images/mc-shaders/2026-11-05/`: `shots/<pack>/`, `sheets/<pack>/`,
  `sheets/lod_bliss_dh/`, `crops/`.
- **Frame time:** not measurable here. Every setting read 9–10 fps, with shaders on or off, at about 14% GPU. The cap is the
  virtual display (rootful Xwayland inside a virtual KWin), not the packs. The packs cost less than that cap on a 5090. A
  real number needs a run on the PC's own screen.

## The packs

| Pack | Why it's here | Far view |
|---|---|---|
| Bliss 2.1.2 (Ronald's settings) | Ronald's favourite; the vivid fantasy look ours aims at | DH |
| Complementary Unbound r5.9.3 | The most-used pack (Unbound is the stronger restyle) | DH, Voxy |
| Complementary Reimagined r5.9.3 | The same code (one line apart), close to vanilla | DH, Voxy |
| Photon v1.4 | Best-regarded sky, clouds and atmosphere | DH, Voxy |
| Solas 3.9 | Modern fantasy, coloured light, cloud shadows in the god rays | DH, Voxy |
| BSL 10.1.8 | The bright, colourful classic | DH, Voxy |
| Mellow 3.5 | Painterly and stylized; built for gameplay | DH, Voxy |

Left out: Rethinking Voxels (stuck on 1.21.9), Noble (photoreal and path-traced, the opposite of our style), and Euphoria
Patches (an add-on to Complementary).

## Bliss vs ours, scene by scene

Sheets: `sheets/bliss/Bliss_vs_ours_*.jpg` (ours left, Bliss right).

- **Night:** ours is now moonlit; the land is faintly readable and the forest is properly dark without a torch. Bliss's
  night is a deep saturated blue in which land and forest stay clearly readable: night as a mood, not as darkness.
  - The open issue is our held torch. At 00:00 it lights the whole forest almost like dusk. Bliss's torch gives a small warm
    pool near the hand, and the rest stays dark.
- **Dawn and dusk:** ours has a warm sky at 04:00–05:00 and a golden sun at 20:00, but the land stays daylight-coloured. In
  Bliss the land takes the colour: golden ground haze and mist with beams through it at 06:00–07:00, and the sun in a warm
  haze at 18:00–20:00. The gap is sun-tinted aerial perspective and low mist, not the sky.
- **Mist:** Bliss has real morning and valley mist (golden at sunrise, crossed by beams), forest fog with light shafts, and a
  lake that glows at sunrise. Ours shows no mist at 06:00.
- **Clouds:** ours are soft cumulus now, a clear step up from flat slabs. At sunrise and sunset ours stay white, while
  Bliss's undersides glow orange with silver edges, and the sky blooms around the sun. Neither shows clear cloud shadows on
  the land.
- **Rain and thunder:** ours improved (looks3: darker deck, rain streaks, a pink and violet dusk rain that is nicer than
  Bliss's muddy olive). Bliss still leads on three things:
  - contrast: a dark deck over land that stays readable and green
  - sunbeams breaking through gaps in the deck
  - lightning that lights the ground around the strike, where ours brightens the whole sky band evenly
- **Snow and blizzard:** ours reads more like snow. Bliss's snow biome in rain looks clear and dry.
- **Wet ground:** neither shows puddles on flat stone after rain.
- **Colour:** Bliss has stronger contrast and saturation all day. Ours is paler and milkier.

## Far view: ours vs Distant Horizons (with Bliss, and with shaders off)

Sheets: `sheets/lod_bliss_dh/` (distances, FOV 30 targets, full-res crops, verdict.md).

- **DH is better at:**
  - Distance haze: Bliss's blue aerial haze makes far land read as far and hides the seam at every render distance.
  - One geometry: DH's LOD is the same blocks as the near terrain. Ours switches to smooth faceted low-poly peaks far out,
    which reads as another game.
  - Far water: a flat sheet with sky reflection and sun glint. Ours shows vertical "curtain" faces (LOD skirts) and stretched
    columns along far shores.
  - Colour: biome colours stay saturated far out.
- **Ours is better at:**
  - Far trees keep a trunk-and-crown shape, where DH turns distant forest into a flat green carpet. But some of our far trees
    are over-tall pillars.
  - We have real shadows on far tiles (the 400-block cascade).
  - Without shaders DH shows a flipped horizon band and floating LOD pieces at FOV 30; ours has no break like that.
- **Render distance:** it barely matters under Bliss, because the haze covers the seam. With shaders off, DH at 64 chunks
  shows a fog wall about 1 km out.

## The other packs

(Filled in as each pack's window lands: Solas, BSL, Mellow, then the Voxy far view for the six Voxy packs.)

### Forest from above: all packs vs our old-growth spot C

Sheet: `sheets/cross/all_forest_above.jpg`.
- **Our side:** old-growth spot C (seed 1, /tp -4232 210 -392, look 30 -10, spring 8) on dev b656e475, with the old-growth
  biome and far shading. Job `mcspotc`, script `tester-tools/mc-shaders/ours_spotc.txt`.
- **The packs' side:** each pack's vista over Minecraft's dense dark-oak forest at the same hours.

- **06:00:** every pack fills the whole forest with warm golden haze and light shafts, and Bliss goes furthest (sun-gold
  fog between the crowns). Ours has a warm sky over a forest that stays green and clear.
- **10:00:** close. Ours has the cleanest blue sky and readable crowns. The packs add aerial haze toward the horizon and a
  darker, more contrasty canopy.
- **18:00:** ours now has a warm haze (dev's far shading) and reads closest to the packs here. They still tint the land
  itself orange.

### Complementary Reimagined r5.9.3 (with DH)

Sheets: `sheets/reimagined/Reimagined_vs_ours_*.jpg`. It is the same code as Unbound with a closer-to-vanilla style, and
it looks it:
- **Clouds:** vanilla-shaped block slabs that are lit and shaded. Next to these, our soft cumulus is the more distinctive
  choice.
- **Atmosphere:** the same warm dawn and dusk haze as Unbound, slightly less saturated.
- **Wet ground:** the same wet-stone darkening in rain as Unbound.
- **Night:** dark and neutral.
- It adds nothing beyond Unbound to take, which confirms that these traits come from the shared Complementary code
  rather than the Unbound restyle.

### Complementary Unbound r5.9.3 (with DH)

Sheets: `sheets/unbound_v2/Unbound_vs_ours_*.jpg`. The vista, weather, mist, cloud and far-view shots are from a redo at
y170: at y200 the camera sat inside Unbound's cloud layer. Forest, night and shadows are from the first run
(`shots/unbound_merged/README.txt`).

- **Wet ground:** the only pack so far that shows the rain on the ground. The stone platform turns dark and wet in rain
  (`wx_wetground_03s`) and dries over about 30 s. There are no puddles, but surfaces darken and gain a sheen. Ours and Bliss
  show nothing.
- **Dawn and dusk:** a warm orange haze fills the whole valley at 06:00 and 18:00, with sunbeams over the forest
  (`mist_valley_h06`). It is like Photon but more saturated.
- **Rain:** a mauve-grey deck with a pale horizon band, and the land stays green and readable. Ours goes milky white.
- **Snow and blizzard:** cold blue haze with the land readable. Ours is whiter and reads more like snowfall.
- **Night:** very dark and neutral; the land is almost invisible from the vista.
- **Forest:** dark under the canopy, with sun specks and crisp warm-tinted shadows.
- **Clouds:** soft and wispy, lit warm at sunrise and sunset. Cloud shadows on the land are faint at best.

### Photon v1.4 (with DH)

Sheets: `sheets/photon/Photon_vs_ours_*.jpg`, crops in `crops/photon/`.

- **Golden hour is Photon's strongest point,** stronger than Bliss.
  - At 06:00 and 18:00–19:00 the whole land sits in a warm volumetric haze, with rays through the canopy
    (`mist_valley_h06`).
  - There is glowing mist over water at sunrise (`mist_lake_h06`), and far LOD fades into the golden fog (`lod_fog`).
  - Ours stays daylight-blue with no mist at these hours.
- **Clouds:** sunrise and sunset undersides are lit warm, with bright rims toward the sun and bloom around the sun. Photon
  rolls a random coverage each day, so in this world most daytime shots are a grey deck. Its "clear 12:00" is overcast,
  so ours reads better as a sunny day.
- **Night:** a dark neutral grey-blue in which the land is barely readable. The held torch does not light the world
  (handheld light is off by default), so its night is darker than ours and than Bliss's.
- **Forest by day:** dark and contrasty, with green-filtered light under the canopy and sun flecks. Ours is brighter and
  flatter.
- **Shadows:** soft, with no acne and good contact at the wall foot. Far water is a flat dark mirror with sky reflection,
  next to our shore "curtains".
- **Weather:** rain keeps the land green and readable under a grey deck, where ours goes milky white. Snow and blizzard
  look dry and clear; ours is better. No lightning flash was caught.

## What we should take, ranked

Each item names the shot that shows it and the technique; costs are from [mc-shader-techniques.md](mc-shader-techniques.md).

1. **Sun-tinted aerial haze over the land, with bloom into the fog.** Bliss's dawn and dusk vista, and its far view.
   Light the haze with a forward Mie lobe toward a low sun (Bliss `overworld_fog.glsl`), and mix toward the bloom image
   where fog covers the scene (bloomy fog). Fixes our flat, milky depth and the "land stays daylight at dawn". Cost: a
   lobe in the haze term, plus one mix once a mip-chain bloom exists.
2. **Morning and valley mist from a time-of-day density table plus a 7-block-half-life ground layer.** Bliss
   `mist_valley_h06` and `mist_morning_h06`. Cost: arithmetic; the density table can live on the CPU.
3. **Cloud lighting with taps toward the sun, plus Rayleigh reddening near the horizon.** Bliss `clouds_sunset_h18`,
   `clouds_sunrise_h06`. 1–2 extra taps per cloud step give silver edges, dark bellies and orange undersides.
4. **Cloud shadows inside the god rays and the rain deck.** Bliss `wx_heavyrain_30s`, `wx_thunder_h12`: sunbeams through
   gaps in the deck. Multiply each volumetric step by the cloud shadow.
5. **Lightning as a local light.** Bliss `wx_lightning_flash`: a short-lived point light at the strike lighting the ground
   and fog around it, instead of a whole-sky exposure bump.
6. **Wet ground in rain.** Unbound `wx_wetground_03s`: stone open to the sky darkens and gains a sheen, then dries over
   about 30 s. Darken the albedo and raise the specular by a rain-wetness value, masked by sky exposure. Cheap.
7. **A smaller held-torch light at night.** Bliss `forest_torch_h00`: a warm pool near the hand. Ours floods the forest.
8. **Far view:**
   - keep the LOD in the same block language as the near terrain (no smooth low-poly peaks)
   - fix the shore and water "curtains"
   - fade far LOD into the sun-tinted haze
   (`sheets/lod_bliss_dh/lod_crops_Bliss_DH.jpg`)
9. **Purkinje night shift** (blue-tinted luminance in low light) to keep nights dark but blue, as a principled replacement
   for our hand-made cool floor (Bliss `vista_h00`).
10. **A mip-chain bloom with no threshold** (BSL, Complementary, Photon, Solas, Bliss), which several of the above build on.
   Needs multi-pass post at lower resolutions in the engine.

What we already do better: real far-terrain shadows, far trees with a shape, snow and blizzard that read as snow, and dusk
rain colour.
