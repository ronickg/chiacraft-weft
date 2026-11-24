# Branch claude/look-sun-tops: dawn mist banks (state at 759d3fd50 + notes, 11-24)

Stage 2 (match Bliss). Everything is behind settings that default to "as made"; Bliss match holds
the sun disc only. Sheets: `~/Coding/chiacraft-archive/images/tester-2/2026-11-24/live/lv8`, `lv9`.

## What is in

- `base:look_mist_domes` 1: the banks walked per pixel (`bank_march`, look.wgsl), as Bliss's
  cloudy fog (method only, `overworld_fog.glsl`): a 3D noise cut at a threshold (`MIST_SHAPE`),
  frayed by a fine noise (`MIST_ERODE`), times a height fall (full to sea + 10, exp -0.05 above;
  thick cores up to 12 blocks taller, `MIST_SEA_RISE`), over a thin even part (`MIST_SEA_EVEN`).
  Lit per step by the shadow map and the clouds' shadow (no self-shade), a sharp forward peak eased
  to a cap of 3 (`MIST_SEA_PHASE`, `MIST_SEA_CREAM`), the sky's light brighter looking up.
  8 steps spaced exponentially out to 1500 blocks. Evening 0.03 of the morning.
- Knobs: `base:look_bank_dense`, `_sun`, `_shade` (1 as made; **0 also reads as made**).
- `base:look_dawn_sky`: Bliss's dawn before sunrise (deep blue, thin pink line).
- `base:look_debug` 3 the banks' light alone, 4 how much they hide, 5 the sun along the march (red
  the shadow map's light, green the clouds', blue how much mist); views 3-5 skip the haze after them.
- With `base:look_sun_disc` (Bliss match), the sky's tight Mie lobe 0.7 plus a middle one (at 2.2
  it read as a hard disc a third of the frame wide round a low sun).
- Live script (`tools/look_live/live.txt`): compare with clouds off, mist pinned 0.02.

## What lv8/lv9 found

- 18:00 matches (72 79 70 against Bliss's 67 65 74) once the general mist pin is 0.02, not 0.1.
- The 128-256 block fade from Bliss's code is a biome blend there, not an end to the banks; with it
  the vista had no banks at all.
- Bliss's shadow map is 128 blocks; its vista shade is not from a shadow map. Ours works near the
  eye (view 5).
- Density: a ray 12° down crosses about 100 blocks of a 20-block layer, so a per-block density near
  1 is an opaque skin on the crowns.
- Tones match, structure does not (land under the horizon band, `lv9/luma_spread.txt`):
  - 06 vista luma p5/p50/p95: Bliss 52/120/195, ours 55/124/205. 07 likewise.
  - Fine detail (std in 16 px tiles): Bliss 5.8 at 06 against 37.8 at noon, a factor 6.5; ours
    17.9 against 26.4. Bliss's mist veils the crowns' texture almost everywhere and its light and
    dark come from the mist itself; ours is thin, the crowns read through (the "frost" look), and
    its darks are bare forest islands.
  - Saturation: Bliss 0.50 against ours 0.42 at 06 vista; 0.45 against 0.32 at 07 low.
  - Low views: our darks too light (p5 105 against Bliss's 66 at 06 low), broad contrast 22 against
    32: the banks' in-scatter covers the crowns, which are near black with the mist off.
- Tooling: `compare on clouds=1` in a running game does not bring the clouds back after clouds=0.

## lv10 try (Planner's line-by-line read, points 1, 3, 4)

- Bloomy fog over the banks: `bank_hidden` (look.wgsl) walks the banks' density only and gives
  1 - transmittance; post blends toward the bloom by 0.94 times that (`BANK_BLOOMY`, Bliss's
  1.25 x 0.75), capped at 0.95, scaled by `look_bloom`. Before, post's bloomy fog read only the
  froxel mist, near nothing over the banks.
- Banks thinned to 1.5 (from 2.5) so the crowns survive as broad darks under the blur.
- Phase shaped like Bliss's fog phase x 5: sharp peak 9.4 e^-87x (only it eased to the cap of 3),
  2.35 e^-6x, a broad tail 0.76 e^-1.19x (x = 1 - cos): about 12.5 / 3.4 / 1.7 at 0 / 14 / 30 deg
  before the cap.
- haze_lit: with the banks on, no glow lobe and no mist grey over land (a second even wash).
- Not done yet: sky pixels march the banks only 220 blocks (sky.wgsl), Bliss to the render distance.

## Open

- Why Bliss's thick mist still varies broadly (thick, yet bright and dark in large patches) where
  ours goes even when thick: Planner's line-by-line read of our lighting and compositing against
  Bliss's GetVolumetricFog is running.
- The ring round the sun through mist at 06 (yellow-green-pink): maybe hue shift at the curve's
  shoulder.
- Dawn clouds saturated yellow; 19:00 night curve; midday greens; horizon band.
- Before ready: merge dev, niced `cargo check --workspace --all-targets`, keep the ENGINE_CHANGES
  row, bench the banks at Max on the final candidate.
