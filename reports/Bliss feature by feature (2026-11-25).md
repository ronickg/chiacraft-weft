# Bliss feature by feature, against ours (2026-11-25)

Ronald, 11-25: "have a look at bliss code again and compare each feature to ours". Four readers went through Bliss's source
(`chiacraft-archive/references/shaders/bliss/Bliss-Shader-git`, Stable 81e403e, method only: "© Chocapic13, all rights
reserved") and our pack (`mods/base/shaderpack`) with the engine parts. File:line references are at the tip of
`claude/look-list` (dec243379). Items marked (unsure) were not verified.

**The pattern in one line:** where Bliss has one physical formula, we often have the same formula plus several hand-tuned
fixes stacked on it, each added to close one gap in one shot. The sun's glow was the first case swapped back (ten layers →
one Mie lobe, 11-25) and the clouds the second (quarter size + own history → Bliss's half size, TAA). The same swap is
open in about a dozen more places, and in most of them it would make ours both closer to Bliss and smaller.

## Verdict table

| Area | Feature | Bliss | Ours | Verdict | Swap to Bliss's method: closer and simpler? |
|---|---|---|---|---|---|
| Sky | Colour model | Physical air march (Rayleigh, Mie, ozone, moon), baked to a 256² texture each frame (ROBOBO_sky.glsl:95-165, deferred.fsh:260) | Five hand colour sets blended by sun height (look.wgsl:636-808), a gradient with per-hour fixes (1030-1245) | ours more complex | **Yes, the biggest one**: about -300 lines net; the colour sets stay where they light the land |
| Sky | Sun disc | A tiny HDR disc, no halo (sky_gradient.glsl:3-5) | Disc, halo, two core tints, a variant, x16 gain (sky.wgsl:1205-1233) | more complex | Yes, about -20 lines |
| Sky | Glow round the sun | Only the Mie lobe in the sky | The new lobe, plus the old layers still behind `keep` (look.wgsl:1006-1224), the vanilla fan (sky.wgsl:1069) | more complex | Yes: drop the old layers (-120 to -150 lines), waiting on Ronald's look at the lobe |
| Sky | Dawn, dusk, twilight colours | Fall out of the physics (reddened path, ozone) | About nine hand overrides (golden, sunset, dusk sets; blue hour; dawn lift; rose, lavender, cream, evening blue) | more complex | Yes, with the physical sky (-150 more) |
| Sky | Night sky | Hash stars, a flat moon with 8 phase lights, moonlight in the baked sky | Engine star quads with tint and twinkle, a lit moon sphere; Milky Way, aurora, meteors are ours only | stars/moon more complex; extras ours | No (Bliss's moon is cruder); keep ours |
| Clouds | Shape | Two cumulus layers (250, 500, 100 thick) and an altostratus plane at 2000; 2D coverage + 3D erosion | One soft layer, 3D billows + Worley erosion; cirrus as a 2D fake; the old blocky clouds still in | same size; ours misses the second layer | Closer, +60 lines (second layer); also the "cauliflower" item |
| Clouds | Lighting | Single + multiple scattering (exp -10 / -3 od), powder, two forward lobes, 3 light taps, layer-to-layer shadow | 5 taps, 3 octaves, powder, plus per-hour sun fixes, an HDR "deep" fill and an HDR rim | more complex | Yes, about -45 lines |
| Clouds | Resolution | Half size, jitter, bicubic upsample, TAA | Now the same (dec243379), bilinear upsample, own history only without TAA | close | Bicubic upsample +25 lines |
| Clouds | Shadows on land | Coverage density per pixel, exp2(-100 s²) | A 4096-block shadow pre-pass, 6-step march, plus the blocky clouds' path | more complex | Simpler code, costlier per pixel; low priority |
| Clouds | Rain | Coverage and density move to rain values; a random cloud mix per day | Coverage, thickness, darkening, slate near deck, grey colour sets | more complex | Partly |
| Fog | The march | One march (8 steps, base 11) for fog, atmosphere and rain together | Three: mist banks, froxels, and `fog_through` when froxels are off | more complex | **Yes**: one half-size march; about -600 lines and froxels.rs; Mac/phone cost to measure |
| Fog | Density | Uniform + cloudy + rain, height falloffs, a weight table by hour and biome | Banks already like Bliss's; `fog_density_with` has ~80 lines of storm/evening/wisp/dome branches | more complex; no biome table | Yes, -100 / +20 |
| Fog | Sun and sky in-scatter | One phase, one shadow tap per step, cloud shadow | Banks: Bliss's; froxels: two lobes, 3-6 taps, a fake bounce | banks same; rest more complex | Yes, comes with the single march |
| Fog | Far haze | Rayleigh/Mie air in the same loop, border fog to the sky | `haze_veil` + `haze_lit` (~225 lines: lobe, tint, evening band, day air, twilight air, glow) | more complex | **Yes**, about -150 lines (unsure it covers our twilight blue) |
| Fog | Rain fog | A rain density in the march, lit; bloomy fog up in rain | An unlit optical depth; bloomy fog cut in rain | ours simpler, opposite on bloom | Closer, same size |
| Fog | Under water | 8-step march to 32 blocks with the shadow map, then absorption | One exponential + 6 noise beams | ours simpler | Closer, +40 lines |
| Fog | Caves | Volumetric fog off, a blue distance fog in caves | Fog simply off | missing in ours | +15 lines |
| Fog | Bloomy fog | One mix by the fog's transmittance | Two mixes, one of them a second 8-step march (`bank_hidden`) | more complex | Yes, -60 lines (store the march's transmittance) |
| Fog | Resolution | Half size, bilateral upsample | Banks at full size; froxels at 1/8 | different | Half-size banks: +50 lines, about 4x cheaper |
| Light | Sun shadows | One 128-block map, PCSS by blocker depth, 13 taps, coloured through glass, contact ray, sky-light past reach | Two cascades, fixed 3x3, RT option, plus a canopy cover curve, plant lift, edge-on discard | no PCSS, no coloured raster shadows | Yes: a blocker-scaled filter could replace the cover/lift fixes |
| Light | Sky light | One lightmap curve x facing weight x minimum | The same curve, then a sliding divisor, hemisphere, rain hemisphere, facing split, cloud fill, canopy sun, golden fill | more complex | **Yes**, a clear stacked-fixes case |
| Light | AO | 7-sample SSAO, min(vanilla^5, SSAO^6), indirect only | Vertex AO + contact (2-4%) + far AO ring that guesses sunlit faces | different | Partly: one SSAO pass; "indirect only" needs it in the lit pass |
| Light | GI | Off by default | Path tracer + denoiser (ours only) | ours extra | n/a |
| Light | Leaves (SSS) | Sun SSS by blocker depth with a forward phase; sky SSS; max(NdotL·shadow, SSS) | A re-derived sun SSS (off), plus four stacked fixes (leaf_through, extra sun, back-light hold-back, lime/red cuts); no sky SSS | more complex, sky SSS missing | **Yes**, strongly |
| Light | Torches | One warm colour on a lightmap curve, boosted at low exposure | RGB light, warming, flicker, day quarter-cut, held-light pool | more complex (coloured light is ours) | Partly: Bliss's exposure boost for the day cut |
| Light | Emissives, PBR | LabPBR emission; GGX + VNDF | Glow texture; GGX + RT + subsurface | about the same | No |
| Light | Foliage colour, wind | Vertex colour; sine waving | Biome average, lime fixes; gusts, rain flutter, parting | ours extra | No |
| Light | Night, Purkinje | Moon 15000 K x50; one exposure-driven rod blend | `display`'s night blue + storybook rods + cool floor | more complex | Yes: one Purkinje |
| Light | Caves, leaks | Eye-brightness leak fix + minimum light | Canopy-bent cover curves + floor light | similar; cave fog missing | Yes |
| Water | Waves | Three noise layers, golden-angle turned | Four procedural layers to the wind, glance/distance flattening, duplicated in reflect.wgsl | more complex | A little |
| Water | Colour | One extinction + dirt scatter, marched | Bliss's coefficients behind an option, stacked on five constants, biome tint, grey, pattern | stacked fixes | **Yes**: a closed-form single scatter |
| Water | Foam, refraction, RT | No foam; UV-offset refraction; no RT | Foam, Snell refraction, RT reflections (ours only) | ours extra | Keep |
| Water | Caustics | Wave heightmap along the sun | Separate noise ridges | different | Closer, same size |
| Water | Reflections | Sky texture, Schlick power 5 from 0.02; SSR in the water pass | `sky_color` + 0.85 lift; Fresnel power 4 from 0.04 by default; SSR in a post pass with blur, masks, fades, +0.15 | more complex | Yes: Bliss's Fresnel as default, drop the lift |
| Post | Bloom | 7 levels from quarter size, about 3x our radius, added | 6 from half size, mixed + added | known (lkbl1) | Hold (Planner): retune after the clouds |
| Post | Tonemap | One curve (AgX minimal by default) | Hejl at night blended with a day curve by sun height | more complex | Yes: one curve all day |
| Post | Exposure | 50 samples, geometric mean, one log curve | The same idea plus day/night power, 0.83, absolute option, night/dawn ceilings, rain cap, clamp, per-hour exposures, land dimming, dusk lift, dawn lift, sky dim | **much more complex** | **Yes, strongly**: Bliss's curve in absolute units |
| Post | TAA, sharpen | 3x3 clamp, Catmull-Rom, fixed blend; CAS 0.35 | Variance clip, frame counts, motion and rain inputs; CAS + motion push | more complex | Partly (ours handles more cases) |
| Post | Grading | Saturation and grade off by default | About ten stacked steps (white balance, splits, leaf-yellow, saturation by sun, +30% "Bliss", cool floor, rods, blood moon, sky saturation) | **much more complex** | **Yes, strongly**; the +30% is right: Ronald's Bliss profile sets SATURATION 0.3 (Bliss's default is 0), so the chain keeps it |
| Post | Vignette, glare | No darkening vignette, no glare | 20% vignette, veiling glare | ours extra | Dropping both is closer |
| Post | DOF, motion blur | Off by default | None | missing, off in Bliss | Not needed |

## Where swapping in Bliss's method gives both (in order of size)

1. **One fog march** (fog + atmosphere + rain, half size, stored transmittance): replaces about ten fog/mist/haze systems
   with one. About -600 WGSL lines and froxels.rs. Risk: Mac and phone cost without froxels; needs a bench.
2. **Physical sky texture** (Rayleigh/Mie/ozone, baked): replaces `sky_color`'s per-hour fixes and the twilight overrides,
   about -300 net. Ronald said it may go into the engine. The colour sets that light the land stay for now.
3. **Exposure and grading**: Bliss's single exposure curve (we have `look_exposure_abs` already) and its near-empty grade
   would replace about twenty stacked steps.
4. **Sky light and leaves**: one lightmap curve with Bliss's facing term; leaf SSS by blocker depth with one max().
5. **Water colour**: one closed-form scatter in place of five constants; Bliss's Fresnel as default.
6. Small: drop the old sun layers (after Ronald's look), the sun disc's halo/tints, cloud lighting's three fixes, the
   vignette and glare, the duplicated wave code.

Each would land as its own branch, judged on the matrix shots against Bliss (lkmxT1 is the current baseline) and benched.

## Ours only, keep

Milky Way, aurora, meteors; RT shadows and reflections; the path tracer; coloured block light; wind, gusts and parting
plants; water foam and real refraction; TAA's extra cases.

## Missing in ours

A second cumulus layer and the altostratus; sky SSS on leaves; PCSS and coloured shadows in raster; cave fog; Bliss's
random daily cloud mix.
