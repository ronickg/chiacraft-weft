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
- Compare mode pins `core:bloom 0` (testkit compare.rs, now noted in its doc), so every sheet before lv10 c had no bloom,
  and the bloomy fog never ran; Bliss's refs have its bloom. The live script now says `bloom=1`.
- lv10 c (bloom on), against lv9 k: fine detail 06 vista 13.0 (19 before, Bliss 5.7), 07 vista 13.9
  (19.4, Bliss 9.3), 06 low 5.8 (10.5, Bliss 6.6), 07 low 5.8 (10.7, Bliss 6.7). Low views match in
  fineness now; their darks are still light (p5 108 against Bliss's 66) and the near crowns of the vista read through.
- Sky pixels walk the banks to `MIST_MARCH_FAR` (`fog_through_sky`), the rest of their fog to 220 as before.
- Banks' sun light gold, not cream (`MIST_SEA_CREAM.x` 0.15), its peak toward the sun capped at 1.2
  (was 3): at 3 the long low rays toward the sun went to a blown white blob, whose edge the bloom
  spread into a ring. Lowering all their sun (0.4) instead turned the banks pink (the sky's dawn
  light), not gold.
- lv10 m against Bliss (`lv10/luma_spread_m.txt`): saturation 06 vista 0.47 (Bliss 0.48), 06 low
  0.54 (0.51), 07 vista 0.32 (0.34), 07 low 0.36 (0.45); tones as before; 18:00 unchanged.

### The ring round the low sun (lv11: gone)

- Its cause: post's sky saturation boost (`hdr_light`, up to 2.3x) grows with height, none at the
  horizon, full by about 17° up. A low sun's warm glow stayed pale level with the sun and went
  saturated just above it: a yellow-orange arc over the sun, read against the pink as green.
  With `look_sun_disc` and a low sun, no boost within about 25° of the sun.
- Also with `look_sun_disc` on a low morning sun: the sky's wide round sun glow a third as strong
  (near white, through the curve's top it stood as a flat cream disc about 20° across), its light
  along the horizon instead (`SUN_BAND`); the sky within about 18-45° of the sun in a pale gold at
  its own brightness (`SUN_GOLD`).
- Not it, tested live: the tone curve (hue-keeping curve and clip moved the band 1-2 levels), the
  rainbow, the sun quad, the banks, the mist and the land haze.
- lv11 d against Bliss (`lv11/luma_spread_d.txt`): 06 vista sat 0.50 (Bliss 0.48), 06 low 0.51
  (0.47), 07 low 0.32 (0.40); tones as before; 18:00 and noon unchanged.
- Low-view darks still p5 109 against Bliss's 69: `look_bank_shade` 0.25 moved them 1 level, so
  not the banks' sky light; the banks' sunlight and the bloomy fog near the eye are next to check.

### Low-view darks (lv12, open)

- What lifts them (06 low, p5; Bliss 66): base 109, bloom off 107, banks' sun off 46, banks off 48,
  banks at 0.4 density 89. So the banks' sunlit mist over the near crowns, not the bloom.
- Every global fix costs the vista, which matches Bliss on the same ground (06 vista median 127-129
  against 131, saturation 0.47 against 0.48): banks' sun 0.35 took the low view to p5 79 / median
  139 (Bliss 66 / 122) but the vista to 101 and 0.38; a fall of 0.15 a block above the top
  (from 0.05) took the low to 86 and the vista to 104 and 0.39. More march steps near the eye
  (spread 40, not 11) changed nothing: debug 5 shows that near mist lit by the map and the clouds.
- Bliss's low reference may not be the same ground: it has a forest hill rising close on the
  right, ours a lake on the left and no near tree. To check before tuning to it.
- `look_bliss true` by set_config does not set Bliss match's knobs; `look_sun_disc` and
  `look_glow_fill` were missing from the live script, so fresh games had the sun disc off
  (lv10 start; lv11 had it on only from a toggle). Both now in `live.txt`.

- lv13: the low camera matches Bliss's sunup refs (same spot, facing +x, our pitch +6 = MC -6:
  horizons at pitch 0 both mid-frame; our /tp puts the eye at the height given, 1.6 blocks made no
  visible difference). The near forest is the same ground.
- Fix (Planner's call: local to the near mist, the vista kept): the banks' sunlight fades in from
  the eye, none to 24 blocks, full from 120 (`MIST_NEAR_SUN`). lv13 b against Bliss
  (`lv13/luma_spread_b.txt`): 06 low p5/50/95 56/128/210 (Bliss 66/122/199, before 106/184/227);
  07 low 76/169/231 (101/174/234); vistas as before (06: 52/129/220 against 59/131/213). 18:00
  went 24/65/125 (Bliss 34/65/115) from `look_glow_fill 0`, now set by the live script, not from this.



- Why Bliss's thick mist still varies broadly (thick, yet bright and dark in large patches) where
  ours goes even when thick: Planner's line-by-line read of our lighting and compositing against
  Bliss's GetVolumetricFog is running.
- The ring round the sun through mist at 06 (yellow-green-pink): maybe hue shift at the curve's
  shoulder.
- Dawn clouds saturated yellow; 19:00 night curve; midday greens; horizon band.
- Before ready: merge dev, niced `cargo check --workspace --all-targets`, keep the ENGINE_CHANGES
  row, bench the banks at Max on the final candidate.
