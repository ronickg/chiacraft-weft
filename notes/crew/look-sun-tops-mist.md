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

### The arc over the sun and the sky round it (lv14)

- Not a ring of its own: our sky round a low sun stood tan-yellow out to about 45° (06 vista blue
  channel 140-155, Bliss's 185-190 all over; Bliss is warm white within about 12°, lavender at
  15-25°, blue beyond), and its edge against the lavender read as a yellow-green arc. At 06 low a
  saturation bump about 7° out (hue 44, 0.28; Bliss's hue falls evenly 46 to 340 at 0.14-0.19)
  through the curve from the golden key's saturated gold glow. It stayed with the mist, banks,
  bloom, wide glow, sun disc and post's saturation boost each off.
- With Bliss's sun on a low morning sun (`sky_color`): the sun's glow all a pale cream
  (`SUN_GLOW_CREAM`), the dawn's cream drawn in to a lobe round the sun in a peach
  (`SUN_CREAM_TIGHT`), the peach band nearer the sun (`SUN_GOLD` 8-25°), and away from the sun
  the sky lavender from about 10° and blue beyond 30° at its own brightness (`SKY_AWAY`).
- lv14 k (`lv14/sheet_lv14_k.jpg`, Bliss | lv13 b | now): the 06 vista's arc gone, its right side
  blue (a little teal where it meets the tan); 06 low a faint ring left (saturation 0.21 at 7°
  against 0.18 either side); 07 and 18:00 unchanged. 06 vista saturation 0.47 to 0.40 (Bliss
  0.48): the land's haze reads the sky's colour, less gold now.
- Tried and dropped: a bright core lobe round the sun (cos^60-150, x1.5-8): the sun barely grew
  and a white blotch came up on the near forest. A brighter, tighter horizon band (`SUN_BAND` 10 9 2)
  did not show: the land and banks cover that sky.
- Live script: a fresh game has `look_mist_domes` and `look_mist_dawn_tint` off (as made); earlier
  games had them set by hand (lv14's first set had no banks). Both now in `live.txt`.

### Seam, ring, horizon glow (lv15)

- The away-from-sun ramp wider (`SKY_AWAY` from about 6° to 53°, blue from 18° to 63°, a lighter
  lavender): the near-vertical seam with a grey-pink band in lv14 k's 06 vista is gone. Under Bliss's
  sun less of the horizon band's darkening: the grey band low on the right is gone.
- The faint ring at 06 low (saturation peak about 100 px out): not the tone curve (it stays put at
  exposure 0.6 to 1.5), not post's saturation boost, not any one of sun_glow, the Mie lobes or the
  dawn cream (each off only moved it). The golden key's yellow horizon (hue 40) showed through where
  the creamed glow fades: with Bliss's sun the horizon's colour a peach too, and the glow fully cream.
  Peak 0.28 in lv14's base to 0.21 now against 0.18 either side: faint, not gone.
- Horizon glow: the banks' sun far out, even ninefold, did not reach the band under the sun (from the
  vista's height the march ends before it), dropped. What reaches it is the sky's horizon band
  (`SUN_BAND` now 8) and the far air's haze: a cream glow toward the sun from 200 to 800 blocks
  (`HAZE_SUN_GLOW`, `haze_lit`). lv15 b: the horizon under the sun glows into the far mist.
- The land haze takes a third of the away tint (`SKY_AWAY_LAND`): no change in the vista's land
  saturation (0.40), so that drop was not the haze's sky colour. With the glow it is now 0.36 (Bliss 0.48).
- The streaks low in the haze at the horizon (06 low): in lv13 too, so older than the sky change.
- Live game: the third pack reload in a row took about 90 s (lv14, lv15), the others a few seconds.

## Open

- 06 vista land saturation 0.36 against Bliss's 0.48: find what took it from 0.47 (lv13) to 0.40 (lv14), then the glow's share.
- A faint arc high on the right of the 06 vista (lv14 k and lv15 b).
- The faint ring at 06 low; the streaks in the far haze at 06 low.
- 07 low darks a little deep (p5 76 against Bliss's 101); 06 low as it is (Planner).
- Bliss's large gold-white glow round the low sun, bleeding into the mist below: not in the sky
  (the land covers it); it wants the banks' or the land haze's light toward the sun, or the bloom.
- The 06 vista's sky still tan-grey between the sun and the blue, and its land 0.40 saturation.
- Dawn clouds saturated yellow; 19:00 night curve; midday greens; horizon band.
- Before ready: merge dev, niced `cargo check --workspace --all-targets`, keep the ENGINE_CHANGES
  row, bench the banks at Max on the final candidate.
