# Bliss vs ours, code read 2026-11-22

Ronald, 11-22: "send a research to look at bliss code again and compare to ours, will probably help".
This is a stage-by-stage read of Bliss v2.1.2 next to our pack and engine lighting on today's dev
(65aebd7c8, after the Weft sync merge a6464cc14). It lists every place where we differ, so the Bliss
matching session can fix them one at a time.

## Sources and method

- **Bliss v2.1.2**: the zip from Ronald's PC (Prism instance `mc-dh`, read-only copy) is unpacked at
  `~/Coding/chiacraft-archive/references/shaders/bliss/pack-v2.1.2/`. His settings file is next to
  it as `ronald-settings-1106.txt`.
  - GitHub has no v2.1.2 tag, so the zip is the source.
  - Bliss is all rights reserved (Chocapic13's licence). This report gives file:line and the method in
    words only. Refs are relative to `pack-v2.1.2/shaders/`.
- **Ronald's Bliss settings** (all of them matter below): Hejl 2015, `SATURATION` 0.3,
  `Sun_temp` 4300, `sun_illuminance` 250000, `Haze_amount` 1.25, `FOG_START_HEIGHT` 50,
  `Evening_Uniform_Fog` 0.18, `Evening_Cloudy_Fog` 0.09, `Morning_Uniform_Fog` 0.1,
  `Morning_Cloudy_Fog` 0.45, `BLOOMY_FOG` 1.25, `SSS_TYPE` 3 (no SSS without a LabPBR pack),
  `SHARPENING` 0.4. Distant Horizons is on. Everything else is at its default:
  - `indirect_effect` 1 (SSAO);
  - `shadowDistance` 128;
  - screen-space contact shadows on;
  - `BLOOM_STRENGTH` 1;
  - auto exposure on;
  - Border fog on.
- **Ours**:
  - `mods/base/shaderpack/{look,world,post,sky}.wgsl`;
  - `mods/base/assets/shaders/post*/{contact,far_ao}.wgsl`;
  - `mods/base/src/look.rs` (defaults);
  - `crates/engine/src/client/frame_look.rs`;
  - all at dev 65aebd7c8.
  - We run the HDR world with `base:look_curve` 1 (Bliss's Hejl chain), `look_white` 1.6 and
    `look_leaf_sun` 1.
- **Numbers marked (computed)** come from my own numerical re-derivation of Bliss's formulas, not
  from runs:
  - its sky, sun transmittance, sky-light average and haze, in Python;
  - the scripts are in the archive next to the pack (`bsky.py`, `tone.py`, `meas.py`).
  - They reproduce the 11-06 study's noon values: ambient (5.45, 7.34, 8.53) without haze, against
    its (5.5, 7.44, 8.4).
  - The tone script runs both tone chains, ours and Bliss's, on the same light.
- **Shots measured**: `images/bliss-stage2/2026-11-22/`.
  - Bliss: `ref/bliss_dbg_*`, `ref/bliss_mcw`. Ours: `runs/t2_lsA_d0/d1/d2` (picture, sun only, sky
    only).
  - Crop x 0.3-0.7, y 0.62-1 (the land). lit20 is the mean of the brightest 20 % of pixels, shade40
    the darkest 40 %.
- **Earlier reads, still valid where not contradicted here**:
  - `side-session-notes/local_a93a48fb…/{look-bliss-source-grade,look-bliss-source-air,bliss-stage-diff,bliss-leaves,forest-exposure}.md`;
  - `side-session-notes/local_b5e0fafd…/bliss-source.md`.

## The two things the debug views showed, explained from the code

Bliss's debug views at noon against ours (land crop):

| | Bliss | ours |
|---|---|---|
| sun only, lit20 | 251 236 205 | 213 204 171 |
| sky only, lit20 | 127 161 204 | 178 184 197 |
| sky only, shade40 | 45 79 141 | 103 113 140 |
| picture, lit20 (crown tops) | 172 175 114 | 132 142 96 |

### 1. Our sky fill is pale grey-blue where Bliss's is clear blue

Four causes, all in the code. The first two are the big ones.

- **Our sky light is about 1.43× too strong against the sun.**
  - Bliss multiplies all sky light by `ambient_brightness × 0.7` (`lib/diffuse_lighting.glsl:71`)
    after the same lightmap curve and facing weights we copied (`dimensions/composite1.fsh:1114-1117`,
    `diffuse_lighting.glsl:69`).
  - We copied the curve (`look.wgsl:1076`, /2 by day) and the facing (`look.wgsl:1089`), but not the
    0.7 (`look.wgsl:1092`).
  - A brighter fill sits higher on Hejl's per-channel shoulder, which greys it.
- **Its hue is less blue.**
  - Bliss's sky light is the sky itself. It averages 9 reads of the sky LUT (with the haze) east and
    west at about 6°, 23°, 45° and 62° up, then normalises the colour and caps its length
    (`dimensions/deferred.vsh:121-151`, `deferred.fsh:219, 292-296`).
  - At noon that is (4.89, 6.56, 9.45): **B/R 1.93, B/G 1.44 (computed)**.
  - Ours is the hand-keyed palette fill (0.85, 0.95, 1.2) × 0.3 (`look.wgsl:281`): **B/R 1.41, B/G 1.26**.
- **Then the white balance takes more blue out on land.**
  - `look_white` 1.6 at noon gives ×(1.032, 1, 0.944) (`look.wgsl:2207`). The fill's B/R on land
    becomes 1.29.
  - Bliss has no white balance; its warmth comes only from the 4300 K sun.
- **Glare.** A grey-blue glare of 0.02 × sky goes over every pixel (`post.wgsl:439`). In our shade
  that is about +17 %.

Through both tone chains, a white top lit by the sky alone at noon (computed):

| | display sRGB |
|---|---|
| Bliss | 163 186 210 |
| ours | 192 195 204 |

That is the same pattern as the debug shots: brighter and paler.

### 2. Our sun has Bliss's hue but is dimmer

- **The hue matches.**
  - Ours is a 4300 K blackbody × Kasten-Young air mass with Bliss's coefficients (`look.wgsl:208, 231-244`).
  - Bliss's is the same blackbody × a 25-step transmittance (`deferred.vsh:163-165`).
  - At 55° (computed): Bliss (1, 0.614, 0.317), ours (1, 0.583, 0.307). At 11°: (1, 0.474, 0.126)
    against (1, 0.479, 0.131).
- **The ratio is where we differ.**
  - On an open sunlit top at noon, Bliss's sun is **3.49×** its sky light. Ours is **2.10×** (computed).
  - So against the fill our sun is at 0.60 of Bliss's.
  - The two factors:
    - the missing ×0.7 above (1.43);
    - the palette's sun/sky luma ratio, 9.6 against Bliss's 11.3 (1.18).
- **Two smaller cuts that Bliss lacks:**
  - We multiply the sun by `mix(0.75, 1, ao)` (`look.wgsl:1117`). Bliss never applies AO to the sun
    (`composite1.fsh:1288`).
  - The near land is dimmed ×0.86 at noon (`post.wgsl:449`).
- **Bliss's bloom is additive** (stage 7). It adds +35 % of a blurred image at the screen centre at
  noon, which lifts its lit tops further. Ours keeps energy at 10 %.
- **The crown-top gap is mostly exposure.**
  - At one leaf albedo fitted to Bliss's lit crown top (170 173 100), our chain at the code's nominal
    exposure gives 161 170 88 (computed).
  - Our measured 124 139 101 needs about 0.63× that light, plus a grey-blue lift (the veil and glare).
  - So stage 6 (exposure) and the cuts above account for most of the crown-top gap. The hue is not
    the gap.

## Stage by stage

Bliss refs are under `pack-v2.1.2/shaders/`, ours under `mods/base/shaderpack/` unless a path is given.

### 1. Albedo and texture sampling

| Bliss (method) | Ours | Difference | Expected effect |
|---|---|---|---|
| `all_solid.fsh:405`: albedo = texture × vertex colour. Linearised with a cubic sRGB fit (`color_transforms` `toLinear`) | sRGB texture, exact decode; `mc_leaf_colours` tint | The cubic fit is under 1 % off the exact curve | None |
| `all_solid.fsh:408`: alpha test 0.1 on the mipped texture. Far leaves' averaged alpha passes, so a far canopy closes up | `world.wgsl:1397-1403` `cutout_edge`: in HDR with `look_leaf_holes` 1 (Max), alpha > 0.5 at mip lod−1, so far crowns keep their holes | Far crowns: Bliss solid, ours holed | Ours shows dark specks in crowns past ~100 blocks (seen 11-07, "dark speckled crowns"). It lowers mean crown luma and adds roughness. A/B `look_leaf_holes` 0 at the vista |
| No albedo grade; grass and plants untouched | `far_flat` on far leaves (`world.wgsl:1287`); far look bits 1/2/4 | Small, matched in stage 1 | — |

### 2. Sky light: colour, curve and facing

| Bliss | Ours | Difference | Expected effect |
|---|---|---|---|
| **Colour**: average of 9 sky+haze LUT reads at 6-62° up east/west, `normalize × min(3·luma, 2.5)`, ×5 when read (`deferred.vsh:121-151`, `composite1.fsh:909`). Length 12.5 by day (always capped). B/R 1.93 at 55°, 1.60 at 32°, 1.24 at 18°, 1.53 at 11°, 2.06 at 9° (computed). It follows the sky, clouds and haze | Palette fill (0.85, 0.95, 1.2)·0.3 (`look.wgsl:281`), × `hdr_scale` 0.45-0.65 (`:190`). B/R 1.41 at noon, 1.28, 1.25, 1.19, 1.18 at the same heights | Ours is less blue at every height but 18°. It is fixed, so it does not follow clouds or haze | **Pale fill, the debug view's grey-blue.** At noon B/R 1.41 against 1.93 |
| **Strength**: indirect = colour × facing × lightmap curve × **0.7** (`diffuse_lighting.glsl:69-71`) | The same curve and facing, **no 0.7** (`look.wgsl:1076, 1089, 1092`) | Our sky light is ×1.43 against the sun | **The fill too bright, the sun relatively dim.** It is the largest single factor in the sun:sky gap |
| **Facing**: tops 2.5, sides 0.93, bottoms max(…, 0.2+0.8(1−lm)) (`composite1.fsh:1114-1117`). **Grass forced to the top value** (`:1115`) | The same by day (`look.wgsl:1089`); grass not forced | Grass blades' sides get 0.93 in ours, 2.5 in Bliss | Short grass and plants darker in ours (near ground, meadows) |
| **SSAO cap on the facing**: `min(skylight, mix(0.95, 2.5, (1−√(1−ssao))²))` (`composite1.fsh:1120`) | None | Bliss's tops in crevices and between crowns lose up to 60 % | Bliss's crown gaps darker, which adds roughness |
| **Clouds**: none locally on the sky light. Clouds are in the LUT average, so the whole fill greys when it is cloudy | Soft cloud shadow also ×0.6 on the sky light under a cloud (`look.wgsl:1097`) | Different method | Cloud shadows in ours take the fill too. Our debug shots ran with clouds on, Bliss's with clouds off (`VOLUMETRIC_CLOUDS=false` in `ref/bliss_dbg_*/pack_settings.txt`), so the debug shots are not like-for-like |

### 3. Sun: strength, colour by height, shadows, SSS

| Bliss | Ours | Difference | Expected effect |
|---|---|---|---|
| Colour: blackbody(4300) × 250000/4000 × 25-step transmittance, ×150/80 when read (`deferred.vsh:163-165`, `composite1.fsh:908`). Luma by height, against noon: 1.00 (55°), 0.92 (32°), 0.79 (18°), 0.64 (11°), 0.57 (9°) (computed) | Palette sun luma with the blackbody × transmittance hue (`look.wgsl:206-210`). Luma against noon: 1.00, **0.78**, 0.82, 0.67, 0.62 | The hue matches within 3 %. Ours dips mid-morning (32°: 0.78 against 0.92) from the `warm` key and the morning blue cut (`look.wgsl:318-324`) | 08:00-10:00 and 14:00-16:00 land about 15 % short of sun against Bliss's |
| Sun/sky luma, raw: 11.3 (55°), 10.0, 8.2, 7.0, 7.0 | 9.6, 10.7, 11.3, 9.8, 9.2 | At noon ours is 15 % low. At low sun ours is 40 % high | Noon tops dim against the fill. Evening (18:00) flat tops over-lit by the sun |
| **N·L remapped**: `clamp((255·N·L − 15)/240)` (`composite1.fsh:1005`). Zero under N·L 0.06 | `sun_facing`: N·L plus a **wrap** of up to 0.45 at a low sun (`look.wgsl:1009-1012`; 0.21 at 18:00) | At 18:00 (sun ~10° up) a flat top gets 0.12 of the sun in Bliss and **0.31 in ours** | 18:00: our tops 2.5× warmer and brighter. The low sun's warmth lands on our ground. Bliss puts it in the air (stage 5) |
| AO never on the sun (`composite1.fsh:1288`) | Sun × `mix(0.75, 1, ao)` (`look.wgsl:1117`) | Up to −25 % sun in corners | Lit crown tops (leaf cubes have corner AO) a little dimmer in ours |
| **Shadows**: a 2048 map to **128 blocks**, faded by 144 (`settings.glsl:126`, `composite1.fsh:1015-1048`). Past it, the lightmap fallback (1 in the open) **min screen-space contact shadows**: 16 steps of 3-6 px, a hit within 3.5 % depth (`composite1.fsh:376-436, 1252`) | Traced or far shadow map over all land (Max) | The Bliss vista camera is 130 blocks above the land, so **its whole vista is outside the shadow map**. Its crowns are lit by N·L and short screen-space rays only | Ours shadows more of each crown's sides and the gaps between crowns ("deep %" 18 against 5 at noon on 11-07, since narrowed). For the vista only, not near views |
| SSS: none with SSS_TYPE 3 and no LabPBR (`all_solid.fsh:531-533`, `composite1.fsh:823`) | `look_leaf_light` 0; `leaf_through` on plants at half strength (`world.wgsl:546-548`) | Matched for leaves | — |
| Cloud shadow on the sun only (`composite1.fsh:1270`) | Soft cloud shadow on the sun, and on the sky ×0.6 | See stage 2 | — |

### 4. AO

| Bliss | Ours | Difference | Expected effect |
|---|---|---|---|
| Indirect × `min((1−vAO²)^5, SSAO^6)` (`composite1.fsh:1180-1188`). SSAO 7 samples, the radius grows with distance, to the DH distance (`DH_AMBIENT_OCCLUSION`, `settings.glsl:733`). Never on the sun | Vertex AO, linear, on the sky light (`look.wgsl:1092`) and blocks; contact shadows to 64 blocks (`post_low/contact.wgsl:51`) multiply the whole colour; `far_ao` (Max: `fx_far_ao` 2, `video.rs:264`) 24 blocks to 0.84 × view distance, strength 0.6, on the **final HDR colour, haze included** (`post/far_ao.wgsl:15`, `post_low/far_ao.wgsl:16, 134`) | Bliss's AO darkens only the surface's sky light, before the haze. Ours (far_ao) darkens what the haze added too, and the sun on a lit face if the shadow test misses | Our far gaps darker *and* their haze darker, so they read less blue than Bliss's navy gaps. A/B `fx_far_ao` 0/2 at the vista |
| SSAO caps the facing (stage 2) | None | | |

### 5. Fog and air in-scatter, near, far, and the 18:00 veil

| Bliss | Ours | Difference | Expected effect |
|---|---|---|---|
| **Haze**: additive Rayleigh + Mie from the eye. Per block ×24 × `Haze_amount` 1.25 (atmosphereMult 1 with DH): R/G/B 1.74/4.05/9.93e-4 at sea level, Rayleigh fading with height. Lit by sun × Rayleigh phase × fog phase, plus sky light (`overworld_fog.glsl:118-150, 257-268`). Never dims the land | Additive "day air" to 250 blocks (B 3.5e-4/block), fading out 300-700 (`look.wgsl:1912-1930`); veil 90-450 blocks onward, capped 0.8, 70 % of hue kept (`look.wgsl:1715-1745, 1762-1905`) | At 250 blocks at noon both add about +39 % blue to a lit crown top (computed: matched by the 11-07 near-air work). Past 300-700 ours turns from adding into mixing | Mid-far land (300-1000): Bliss keeps the land's own light under a growing blue; ours greys toward the sky colour |
| **Evening fog (the 18:00 veil)**: time weights peak at tick 12000 = 18:00 (`climate_settings.glsl:213, 232`). Ronald's 0.18 uniform (default 0.05) × `2^(−0.15(y−53))` plus 0.09 cloudy noise banks to y ~73 (`overworld_fog.glsl:16-35`). Marched with a step of length/8 (`:105`), so σ ≈ 0.008/block at y 63. Lit by the 10° sun, (83, 39.5, 10.5) (computed), × fog phase ×5 (strongly forward), plus sky-ground light | **No evening fog.** Our mist is the dawn mist only (`mods/base/src/weather.rs:177-183`, `fog_density_with` `look.wgsl:1271`) | Bliss has a warm, sun-lit fog layer over the low land at 17:00-19:00 (half at 17:00, full at 18:00); ours has none | **The 18:00 warm veil.** From the vista (land ~130 blocks below) a ray crosses ~50 blocks of it: τ ≈ 0.15. That adds ~14 % of an orange fog light whose luma (~19) is twice the land's. Plus bloomy fog below. Ours puts the warmth on the tops (stage 3 wrap) |
| **Bloomy fog**: where the fog's transmittance T < 1, the picture is mixed toward the bloom image by (1−T)·1.25·0.75 (`composite11.fsh:152-154`) | `mist_over` × 0.6 to the glow, mist only (`post.wgsl:414-419`) | Bliss's evening fog softens and glows; ours has no fog to do it with | Part of the 18:00 veil's softness |
| Border fog to the averaged sky over the last third of the DH distance (`composite1.fsh:1305-1323`) | Handover to the horizon colour over the last 16 % (`look.wgsl` `edge`) | Known (11-06 air study) | Horizon line |
| Morning fog: 0.1 uniform + 0.45 cloudy at 06:00, gone by 08:00 | Dawn mist by weather | Matched separately (mist work) | — |

### 6. Exposure

| Bliss | Ours | Difference | Expected effect |
|---|---|---|---|
| Meter: 50 R2 points over the central 70 % of a bloom tile, geometric mean of luma (`deferred.vsh:225-245`). Smoothed 0.95/frame | 8×8 taps over the whole frame, centre-weighted 1 → 0.3 (`post.wgsl:210-232`). τ 0.8 s brighter / 2.5 s darker | Coverage and speed (known) | Small at a steady shot |
| Target `E = 0.1116 / log2(2.5L + 1.045)`, absolute, in full (`deferred.vsh:250`) | `k = target(L)/target(0.35) × 0.83`, clamp 0.6-4 (`post.wgsl:239-262`), × fixed 2.2 (`look.wgsl:2093`) × 0.389 (`:2182`) × near land 0.86 at noon (`post.wgsl:449`) | **Our light is in units ~41× smaller than Bliss's** (sun luma 1.77 against 72.7), so the same curve works in a different region. Its slope d ln E / d ln L is **−0.71 at our L ≈ 0.3, −0.28 at Bliss's L ≈ 12** (computed) | Our land swings 2.5× more with how much sky or bright land is in view. The measured crown tops imply ~0.63× of the nominal exposure on our vista (section 2 above). Fix: feed the curve L × 41 and drop the 0.35 reference, the 0.83 and the land dim, then re-set the 0.389 once |
| One curve day and night | Night half-curve, ceilings 0.7/1.2 | Known (11-06) | Night only |

### 7. Tone curve, saturation, grade

| Bliss | Ours | Difference | Expected effect |
|---|---|---|---|
| Hejl 2015 fit (×0.85 in, 1.75, white 3, zero-offset), then the sRGB OETF on its display-coded output (`color_transforms.glsl:34-40`, `composite11.fsh:164`) | The same chain with ×0.389 (`look.wgsl:2178-2185`) | Matched | — |
| +30 % saturation in display space, Rec.601 luma (`composite12.fsh:126-132`) | The same (`look.wgsl:2258-2261`) | Matched | — |
| **Bloom: additive** normalised 7-level blur × `0.5·E^0.2·(1.5 − 2r²)` (`composite11.fsh:99, 141, 154`): **+35 % at the centre, +12 % in the corners at noon** (E 0.0225), more at night | Energy-kept mix of 10 % (`post.wgsl:265, 419`) | Bliss adds light; ours only spreads it | Bliss's shade and mids lifted by a local average (shade40 in the DIRECT view 86 against our 29 is mostly this plus the +0.5), tops +10-20 %, the centre brighter than the edges |
| **No grade**: no white balance, no shade tint, no leaf yellowing, no near-black drain | White balance ×(1.032, 1, 0.944) at noon on land, more in the morning (`:2196-2208`); shade tint (`:2211`); **SPLIT_WARM 0.7** leaf tops toward (1.35, 1.08, 0.45)·luma (`:2040, 2214-2231`); near-black drain to 75 % (`:2235`) | Four grades Bliss lacks. They were tuned when our sun was near-white and the curve different | With the blackbody sun, the white balance and SPLIT_WARM double-count the warmth on tops, and the white balance strips the fill's blue (stage 2). Remove them one at a time as stages 2-3 are matched |
| Sky: none extra | Sky saturation ×1.4-2.3 up the sky and ×0.75 by day (`post.wgsl:374-408`) | Sky only | — |

### 8. Clouds and their shadows

| Bliss | Ours | Difference | Expected effect |
|---|---|---|---|
| Volumetric clouds, layers 0-2 at Ronald's coverage 0.5 / 0.6 / 0.4. Shadow on the sun only (`composite1.fsh:1270`). Clouds enter the sky light through the LUT average (`deferred.fsh:290-296`) | Soft clouds (pack) or upstream's clouds without the pack; soft shadow on the sun and ×0.6 on the sky light (`look.wgsl:1097`) | Local sky-light darkening in ours | Darker, more contrasted cloud-shadow patches in ours. Not in Bliss's debug refs (clouds off) |
| Haze cut above the cloud base (`overworld_fog.glsl:251`) | — | — | Sky only |

## Ranked: how much of the remaining gap each likely explains

1. **Sky light ×0.7 missing and a 15 % lower sun/sky ratio.**
   - Code: `look.wgsl:1092` against `diffuse_lighting.glsl:71`; palette `:281` and `hdr_scale` `:190`.
   - Sun:sky on an open top is 2.10 against 3.49.
   - It explains "sun dimmer" and "fill too bright and pale" together.
   - Try: the HDR sky light × 0.7, and the noon sun × 1.18 (or the fill × 0.85 with exposure
     re-set). Check lit tops / shade by facing on one fixed-exposure pair.
2. **Sky-light hue from the sky, not the palette.**
   - Code: `look.wgsl:281` against `deferred.vsh:121-151`.
   - B/R 1.41 against 1.93 at noon. The method: average our own sky colour at 6-62° up
     east/west, normalise, keep the length we use.
   - Or as a first step, a noon fill hue of (0.52, 0.69, 1.0), with the B/R targets by sun height
     from stage 2.
   - It explains "pale grey-blue against clear blue".
3. **Exposure in the wrong units.** Code: `post.wgsl:240-262, 449`; `look.wgsl:2093, 2182`.
   - The curve's slope is −0.71 against −0.28, and our vista's crown tops get ~0.63× the light the
     nominal chain implies.
   - Feed L × 41 (the sun-luma ratio), use Bliss's absolute target, drop the 0.35 reference, the
     0.83 and the land dim, and re-set 0.389 once.
   - It explains much of the crown-top brightness (124 139 against 170 173). The yellow follows from
     brightness through Hejl's shoulder.
4. **Additive bloom.** Code: `post.wgsl:419` against `composite11.fsh:141, 154`.
   - +35 % centre, +12 % corners at noon. It lifts the shade and mids and the tops a little.
   - It is part of Bliss's brighter, softer land and of its centre-bright framing.
5. **The 18:00 veil is Bliss's evening fog, not haze.**
   - Code: `climate_settings.glsl:213, 232`; `overworld_fog.glsl:16-35, 105`; Ronald's 0.18 / 0.09.
   - A low sun-lit fog layer: σ ≈ 0.008/block at y 63, halving every 6.7 blocks up, lit orange
     with a strong forward lobe, then softened by bloomy fog.
   - Ours has none, and instead puts the low sun on flat tops through the 0.21 wrap
     (`look.wgsl:1010`; Bliss remaps N·L down, `composite1.fsh:1005`).
   - Try: an evening mist term at 17:00-19:00 with that profile, and the N·L remap in place of
     the wrap in the HDR world.
6. **Grades Bliss lacks.** Code: `look.wgsl:2207, 2214-2231, 2211, 2235`.
   - White balance 1.6, SPLIT_WARM 0.7, shade tint, near-black drain.
   - With the blackbody sun now in, they double-count warmth on tops and take the fill's blue.
   - Turn them off one by one after 1-3 (`look_white` 0 first).
7. **Far shadows.** Bliss's whole vista lies outside its 128-block shadow map, so it gets only
   N·L × short screen-space contact shadows. Ours is traced.
   - Our vista has more shaded crown sides and gaps.
   - A test switch "no sun shadow past 144 blocks, contact only" would isolate it for the
     comparison. Not a look to keep.
8. **Far leaf holes.** `world.wgsl:1397-1403` against `all_solid.fsh:408`.
   - Our far crowns stay holed; Bliss's close up. Dark specks, lower crown mean.
   - A/B `look_leaf_holes` 0.
9. **AO placement.**
   - `far_ao` darkens our haze too (`post/far_ao.wgsl:15`), and the sun gets AO (`look.wgsl:1117`).
     Bliss's AO is on the sky light only, before the haze.
   - Grass is not forced to the top facing (`composite1.fsh:1115`), and there is no SSAO cap on
     the facing (`:1120`).
10. **Mid-morning sun.** Ours at 32° is 0.78 of noon against 0.92 (`look.wgsl:318-324`, the warm
    key). 08:00-10:00 and 14:00-16:00 only.
11. **Cloud shadow on the sky light** ×0.6 (`look.wgsl:1097`). Bliss has none locally.
12. **Haze past 300-700 blocks.** Additive in Bliss, a veil in ours. Near-air already matched it to
    300.

## Comparing with the debug views: what to keep the same

- **Clouds.** Bliss's debug refs ran with clouds off (`ref/bliss_dbg_*/pack_settings.txt`); our
  `lsA` runs had them on. Run ours with clouds off, or Bliss's with them on.
- **Auto exposure.** Each pack auto-exposes on its own debug picture: a white world lit by the sun
  alone, or by the sky alone. So only ratios within one shot compare: top/side, near/far, hue.
  - For absolute numbers, fix the exposure on both: Bliss `AUTO_EXPOSURE` off with
    `Manual_exposure_value` ≈ 0.0225 (its noon vista value), ours `base:look_auto_exposure` 0.
- **Bliss's DIRECT view adds +0.5** to every land pixel (`composite1.fsh:1351-1354`). Haze,
  bloom and the grade run after both debug views, in both packs.
- **Bloom.** Bliss's bloom (additive) still runs in its debug views. For a clean light compare,
  `BLOOM_STRENGTH` 0 and ours `look_bloom` 0.

## Numbers for targets (computed from Bliss's formulas, Ronald's settings, clear sky, DH on)

| sun height | DirectLightColor | sky light (AmbientLightColor) | sky light B/R | sun/sky luma |
|---|---|---|---|---|
| 55° (noon) | 107.7 66.2 34.1 | 4.89 6.56 9.45 | 1.93 | 11.3 |
| 32° | 102.9 60.4 27.7 | 5.56 6.81 8.88 | 1.60 | 10.0 |
| 18° | 94.1 50.5 18.4 | 6.46 7.08 8.03 | 1.24 | 8.2 |
| 11° | 83.3 39.5 10.5 | 5.78 6.67 8.85 | 1.53 | 7.0 |
| 9° | 77.9 34.5 7.7 | 4.82 5.88 9.92 | 2.06 | 7.0 |

- These are before the facing weights. Then:
  - an open top takes sun × N·L (remapped) against sky × 2.5 × 1.5 × 0.7;
  - a side takes sky × 0.93 × 1.5 × 0.7.
- Ours on the same footing: sun × `sun_facing` against sky × 2.5 × 1.5 (sides 0.93 × 1.5).
- These assume the sun's path at 35° from the zenith and a camera 200 blocks up.
- Clouds would grey the sky light and lower its B/R.
