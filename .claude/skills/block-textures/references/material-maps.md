# Material maps: shiny, metal and bumpy blocks

A block texture can carry LabPBR 1.3 material maps beside it: `<name>_s.png` (specular) and
`<name>_n.png` (normal). The engine loads them (`docs/textures.md#materials`). Chiacraft's shader pack gives them:
- a sun highlight
- the sky (or, under a roof, the room's light) mirrored by Fresnel
- normal-mapped light
- screen reflections on smooth floors, with the HDR world

Traced reflections (the RT session's next step) read the same maps. A texture without maps looks
exactly as before. The checks: a forest before/after showed only run-to-run noise (11-06).

## When to make them

- Any new block that should shine, reflect or read as metal: polished or glazed stone, tiles, metal
  blocks and trims, glass, lamps.
- Not for natural blocks (dirt, logs, leaves, plants): a texture without maps reads as rough stone,
  which is right for them.
- After every redo of a texture that has maps: the maps are made from the picture, so make them again.

## How

1. Install the texture first (`blocks.py install`, or copy the tile into `mods/<mod>/assets/textures/block/`).
2. Add a row to `notes/blocks/batches/matmaps.tsv`: `name<TAB>mod<TAB>rules` (rules below).
3. `python3 notes/blocks/matmaps.py batches/matmaps.tsv <name> ...` writes `<name>_s.png` and `<name>_n.png`
   next to the texture. Commit them with the texture.

Rules, `key=value` separated by spaces:

| rule | meaning |
|---|---|
| `smooth=0.85` | smoothness 0-1 (LabPBR red; roughness = (1 - smooth)²) |
| `f0=10` | reflectance looking straight on, /255 (stone and glass 10, which is 4%) |
| `metal=iron\|gold\|copper\|aluminium\|chrome\|lead\|platinum\|silver\|albedo` | a metal (the pack reflects in the texture's own colour) |
| `porosity=0.5` / `sss=0.5` | how much rain darkens it / subsurface light |
| `normal=lum:1.0` | bumps from the picture's brightness, at this strength; `normal=flat` none |
| `emit=0.6` | texels brighter than 0.85 glow at this strength |
| `tile=0` | for framed pictures (metal blocks, lanterns): the bumps don't wrap at the edge |

Overrides, each `selector=<number>:key=value:...`, the last that matches a texel wins:

| selector | picks |
|---|---|
| `dark=0.3` | texels darker than 0.3: grout, seams, crevices (`depth=0.3` sinks them, a groove the normals show) |
| `light=0.8` | texels brighter than 0.8: glazed tiles' pale grout, a lantern's glowing glass |
| `solid=` | fully opaque texels of a see-through texture: glass frames, lead cames |
| `greenish=` | texels greener than red: verdigris patina on copper |
| `grid=16` | every 16th row and column: a code-made tile's grout |

## Values that worked (RTX showcase, 11-06)

| material | rules |
|---|---|
| polished stone (limestone, obsidian) | `f0=10 smooth=0.86..0.95 normal=flat` |
| floor tiles | `smooth=0.86 dark=0.55:smooth=0.12:porosity=0.8:depth=0.3` |
| glazed tiles | `smooth=0.92 light=0.72:smooth=0.2:porosity=0.6:depth=0.25` |
| slate (glossy in rain) | `smooth=0.35 porosity=0.75 normal=lum:0.5` |
| copper, bronze, steel plates | `metal=... tile=0 smooth=0.65..0.78 normal=lum:0.6..1.2` with dark seams |
| clear and tinted glass | `f0=10 smooth=0.95..0.97 normal=flat solid=:f0=10:smooth=0.3` |
| frosted glass | `f0=10 smooth=0.45` |
| lamps | `smooth=0.8 normal=lum:0.6 emit=0.5` |

## Checks

- Look at the `_n` map lit from the top left (as `matmaps_check_metal.png` in the archive did): bevels
  should light from the top left, as the picture's own shading does. If they light from the bottom
  right, the height is upside down.
- `_s` alpha stays 255 (no glow) unless `emit` is set: a lamp's face then glows at night.
- In the game, check the material and not only the picture. In the hall's first run, copper with
  LabPBR's copper F0 under a bright sky read as pale lavender; the pack now reflects metals in their
  albedo. Shoot with `--shader-pack chiacraft --config core:hdr_world=1` (the floors' screen
  reflections need the HDR world).
- The showcase hall (`notes/blocks/showcase_scene.txt`) is the test scene for shiny blocks.
