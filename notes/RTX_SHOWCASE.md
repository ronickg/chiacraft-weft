# RTX showcase blocks

Blocks made to show off the ray-traced ultra mode (PC, RTX 5090): glass, polished and glazed
tiles, metals and lamps, plus a scene script for a glass-and-tile hall. Ronald, 11-06: "I will
need to add some like cool glass or tile things ... because that's sort of where the RTX stuff
really shine." Worker: rtx-showcase-blocks, branch `claude/rtx-showcase-blocks-272636`.
Background: `reports/NVIDIA RTX mod study.md` (branch `claude/rtx-mod-study-5939fb`).

## What NVIDIA's showcase leans on (study only, nothing copied)

From NVIDIA's Remix 1.6 article, the mod's own docs on GitHub, and the CurseForge pages of the
*RTX Remix Showcase World* (BlockWorks) and *RTX Remix Materials*, all read online on 11-06.
- **Scenes:** the world's published shots are a cherry tree, a jungle cathedral and its inside,
  a storeroom and an ice tunnel. Nothing more about its layout is published.
- **Effects:** NVIDIA's article highlights these:
  - reflections on water, including things off screen
  - refraction and caustics under water
  - subsurface light through ice and leaves
  - bounce light that takes the colour of stained glass and painted walls
  - light shafts through windows
  - many lights that cast shadows

  The *Copper Beacon*, a brass lamp with an aimable beam that lights walls and mist, is the lamp they show off.
- **Materials pack:** 64x textures, almost all with LabPBR `_n`/`_s` maps, plus 245 subsurface maps
  of their own (`_sss`: leaves, flowers, wool, coral, ice).
- **How the mod reads the maps:**
  - Metal is on or off: a `_s` green of 230 or more makes full metal, tinted by the albedo, and the metal ids are ignored.
  - Porosity is ignored.
  - A texture with no map is matte: roughness about 0.71.
  - Glass refracts at a fixed IOR of 1.52, water at 1.33. Stained glass takes its colour from the albedo.
  - A block's glow map doesn't light anything: lamps need a real light as well.
- **Not found in any public source:** frosted or tinted glass, polished marble, glazed tiles, copper
  weathering, mirrors, wet surfaces. Those are ours to design.

What follows from this for our blocks:
- Metals: full metal, with the colour in the albedo.
- Tiles: smoothness about 200-215 of 255, with bevelled grout in the normal map and a little height.
- Clear glass: keeps a few opaque frame or streak pixels, so it reads as glass.
- Stained glass: a saturated albedo.
- Every lamp: a real light as well as its glow.

## LabPBR 1.3 (the format our blocks carry)

`<texture>_s.png` and `<texture>_n.png` sit beside the colour texture. They have the same size and frames, and are linear (not sRGB).

| map | R | G | B | A |
|---|---|---|---|---|
| `_s` | perceptual smoothness; roughness = (1 - R/255)² | 0-229 linear F0 (glass and stone 10, so 0.04); 230-237 metals (iron, gold, aluminium, chrome, copper, lead, platinum, silver); 255 metal with albedo as F0 | 0-64 porosity (wet darkening); 65-255 subsurface | emission 0-254, 255 none (already read today) |
| `_n` | normal X (0 left, 255 right) | normal Y, DirectX (0 up, 255 down); Z = sqrt(1-x²-y²) | AO (255 none) | height (255 surface, 0 a quarter block deep; at least 1) |

## What our engine has, and what's missing for reflective blocks

Has:
- Emission only, from `_s` alpha or our `_e`, through `glow_map` (`client/textures.rs`, one word per bindless colour index).
- Water reflections, which are screen-space only: `mods/base/assets/shaders/post/reflect.wgsl`, masked by the alpha that `look.wgsl` writes for water.
- Ray-traced sun shadows (`core:rt_shadows`, branch `claude/rt-sun-shadows-da86bb`), coloured through stained glass.

Missing:
1. **Material maps**: `_s` R/G/B and `_n` aren't loaded at all. There's no table on the GPU and no helper in the prelude.
2. **Raster shading with them**: a sun specular (GGX), sky reflections on smooth surfaces, normal maps in the lighting, and porosity in the rain's wet look.
3. **Screen reflections on anything smooth**: the reflections' mask is water only.
4. **Traced reflections** (RT step 2): a hit needs its texel and material. Today only see-through records are copied for texel lookups, so opaque records need copying too.
5. **Glass refraction**: none. Glass and water blend flat.

Proposed (sent to the RT session on 11-06, waiting on its answer):
- `textures.rs` loads `_s` and `_n` as their own bindless images, as it does the glow masks.
- `mat_map: array<u32>` holds two words per colour index (spec, normal; `u32::MAX` none), next to `glow_map` in the world, entity and RT preludes.
- `mat_at(layer, uv, albedo) -> Mat` decodes LabPBR. With no maps it returns rough 1, F0 0.04 and a flat normal, so today's blocks look the same.

## The proposal (waiting on Ronald)

Sheet: `~/Coding/chiacraft-archive/images/rtx-showcase-blocks/2026-11-06/proposal_sheet.png`.
Its mockups are rough placeholders, not the art.

| block | made from | age | material |
|---|---|---|---|
| Frosted glass | glass + sand | Bronze | smooth 0.45, F0 0.04: soft reflections, milky light |
| Smoked glass | glass + coal; dims light | Bronze | smooth 1.0: dark, mirror-like windows |
| Leaded glass | 4 stained glass + tin ingot → 4 | Bronze | glass smooth 1.0, tin cames as rough metal; coloured light per texel |
| Polished limestone | 4 limestone + sand → 4 | Bronze | smooth 0.85: marble-like |
| Limestone tiles | 4 polished limestone → 4 | Bronze | tiles 0.85, porous grout 0.1, bevelled edges |
| Polished obsidian | obsidian + sand | Iron | smooth 0.95: black mirror floor |
| Slate tiles | 4 slate → 4 | Stone | 0.3 dry, porosity 50: glossy in rain |
| Checker tiles | 2 polished limestone + 2 slate tiles → 4 | Bronze | the hall floor |
| Glazed tiles ×3 (blue, green, red) | 4 bricks + dye → 4 | Bronze | glaze 0.92, each tile's normal tilted a little (sparkle) |
| Copper block | 9 copper ingots (and back) | Bronze | copper metal, 0.75 |
| Verdigris copper | copper block weathers in rain (~3 days), or + water bucket | Bronze | patina rough non-metal, bare copper flecks |
| Bronze plate | 4 bronze ingots → 4 | Bronze | albedo metal, 0.6, rivets and seams in the normal map |
| Steel plate | 4 steel ingots → 4 | Iron | iron metal, 0.7, brushed |
| Bronze lantern block | 4 bronze ingots + frosted glass + glowstone | Bronze | light 15 warm; glowing glass in a metal frame |

Existing blocks that get maps too:
- `glass`: smooth 1.0, plus connected edges
- the red, green and blue stained glass
- `iron_block` (230), `gold_block` (231)
- `copper_lamp`: copper 234, and the recipe it has waited for, now that copper exists
- the three coloured lamps

Thin glass panes are left out: the engine has no connecting-shape block model yet.

## The showcase scene (to build after the OK)

A testkit script builds a glass-and-tile hall on open ground:
- floor: checker tiles
- walls: limestone tiles, with leaded and frosted glass windows
- a pool in blue glazed tiles, with water
- bronze and copper trims, and lanterns on the walls

It's shot by day, at sunset and at night, with shadow maps and with `core:rt_shadows` on, through the PC queue.
