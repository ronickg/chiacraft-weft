---
name: block-textures
description: Block textures for the game (cubes, logs, leaves, plants, crops, saplings, bushes, workstation faces) at 32 px in one hand-painted pixel style, generated with Codex from Ronald's block master prompt, cut into game tiles, checked on a preview sheet and in a test world, and put into the mods. Use when Ronald asks for new or redone block textures, a new tree, plant, crop or bush and its stages, "the block prompt", a block that looks off next to the others, or wants to see how blocks look in the game.
---

# Block textures: 32 px, Ronald's master prompt

Everything lives in `notes/blocks/`: `style.txt` (the master prompt), `blocks.py` (generate, cut,
preview, install), `batches/*.tsv` (what to make), `approved.txt` (the style key), `tiles/` (the
32 px results, kept in git), `scene.txt` (a test world).

How the style was chosen (2026-11-03): the old blocks were 64 px, soft and painterly. Ronald
compared them with a crisp 32 px set and then with his own master prompt (his `Block Art Style
Guide.md`, Part 3); he picked the master prompt. It asks for 32 px, readable clumps (slabs, clods,
pebbles, leaves, bevelled bricks), sage-grey stone, walnut-to-honey wood, muted greens, crevices a
deep shade of the block's own colour (never black), restrained highlights, top-left light.

## Rules

- **Never attach other games' or resource packs' art** to a generation, and keep "Minecraft"
  out of the prompt. Ronald's guide comes with reference sheets cut from a resource pack
  (Dokucraft Light); they are not attached. The style key is our own approved tiles
  (`approved.txt`, built into a sheet on every run) plus `notes/texture-test/palette.png`.
- **Every request gets the style key**, and every result is looked at before it is cut: the
  image model sometimes stops following the prompt.
- **One resolution**: every block texture is 32 px. Don't mix in 16 or 64 px textures.
- **Tinted textures are grey**: grass top, the grass side's fringe (its overlay), all leaves, tall
  grass, ferns marked `tplant`. The game multiplies them by the biome colour (`tints.rs`).
  `blocks.py` greys them; the prompt still asks for natural colours so the shading reads.
- **Cutouts are hard-edged**: plants, crops, leaves and flat things are drawn on #FF00FF magenta,
  which becomes fully transparent. No half-transparent pixels.

## The way, every time

1. **Batch**: one row per generated image, four textures each (`batches/first.tsv` is the worked
   example):
   ```
   image<TAB>mod<TAB>name|kind|what it is<TAB>... (four entries, `-` for an empty square)
   ```
   `mod` is where a new texture goes (`base`, or `animals` for the Cretaceous ones); a texture
   the game already has is replaced where it is. `name@-` keeps a tile out of the game,
   `name@animals` sends one entry elsewhere. Put four of a family in one image (a tree's log, top,
   leaves and planks; a crop's four stages) so they match each other.
2. **What it is**: material and colours, then shape and size: "Stone: large sage-grey stone slabs
   (#4e5250 #707570 #8fa097 #9daea4 #b3c3bb) with deep sage crevices, 6-9 slabs". Use the colour
   ramps below. Plants say how tall they stand of the 32 px ("a small mushroom, 10-14 px tall";
   "a low bush, about 16 px tall"); crops by stage ("stage 1 of 4: short shoots a quarter tall").
3. **Kind** adds the master prompt's type line and says how the square is cut:

   | kind | for | cut |
   |---|---|---|
   | `cube` | stone, dirt, sand, gravel, ores' base, planks-less natural blocks | tiled, seams healed |
   | `grass` | grass block top | tiled, greyed for tint |
   | `grass_side` | grass block side | whole; its green fringe becomes `grass_side_overlay`, greyed |
   | `log` | bark | tiled |
   | `end` | log tops, cut ends | one picture |
   | `built` | planks, bricks, stone bricks | tiled |
   | `framed` | smooth stone, metal and gem blocks | one picture |
   | `face` | workstation faces, compost bin top, fruit blocks | one picture |
   | `leaves` | all leaves | magenta gaps 25-35%, cut out, greyed for tint |
   | `plant` | flowers, saplings, mushrooms, bushes, cacti | cut out |
   | `tplant` | tall grass, ferns, grazed grass | cut out, greyed for tint |
   | `crop` | crop stages | cut out (drawn to read 4 times in a # shape) |
   | `cutout` | ladder, glass frame: fills the square, open parts magenta | cut out |
   | `flat` | lily pads, water hyacinth (seen from above) | cut out |
   | `fleaves` | leaves with flowers or fruit drawn in (dogwood) | cut out like leaves, kept in their own colours (a tint would turn the flowers green) |
4. **Generate**: `python3 notes/blocks/blocks.py gen batches/<x>.tsv [image ...]` (2 Codex runs at
   once, the crew's limit; run it niced, a fresh `codex exec` each, so nothing drifts; an image that exists is skipped, delete it
   to redo). Look at each `gen/<image>.png`. The approved key holds no plants yet, so for plants and
   for a family that must match a texture (a tree's mossy bark, a column's top) attach the game's own
   with `BLOCK_LIKE=kelp,cattail,/abs/path/bark.png` (names from the mods or `tiles/`, or absolute
   paths), and say the batch's `BLOCK_LIKE` in its first comment line. A column's top must start its
   stalk where the block below ends it: check the bottom row's pixels and shift the tile if needed.
5. **Cut**: `python3 notes/blocks/blocks.py cut batches/<x>.tsv` writes `tiles/<name>.png`.
6. **Preview**: `python3 notes/blocks/blocks.py preview batches/<x>.tsv` writes
   `preview/<x>.png`: each tile flat, and as it stands in the world (a cube with its own sides
   and top, or a plant on the grass block, tints applied). Look at it, then show Ronald.
7. **Install** what he approves: `python3 notes/blocks/blocks.py install batches/<x>.tsv`. It
   replaces the texture of the same name and removes its old random variants and connected
   rules (`<name>.ctm`, `<name>/`), which would show as patches. A new block's texture still
   needs the block registered (`mods/base/src/blocks.rs`, `.texture("name")`).
8. **In the game**: `notes/blocks/scene.txt` builds a test patch (stone, cobble, mossy cobble,
   dirt, sand, gravel, three logs, an oak) and shoots it; `scene_all.txt` shows every block. Copy
   and change one for other blocks. **Don't launch the game yourself** (Ronald's rule since
   2026-11-04: two games at once or one GPU stall can restart the Mac). Build the binary
   (`cargo build --release`), then SendMessage the session "Test sessions for all sessions"
   with the dir holding `target/release/weft`, the script's absolute path, a fresh world name,
   what to look at (the shots), and that it changes no rendering or shaders (texture swaps
   don't). It runs games one at a time and sends back the verdict and the log path.
9. **Approve**: add the approved tiles to `approved.txt`; they become part of the style key.
   Commit the tiles, the batch and the game textures together, one family a commit.

## The grade: tune, don't redraw

Ronald, 11-04: "ideally it should be something we can tune easily without having to like now redo
every asset of the game if it's too dark or too light or high contrast, low contrast ... but ideally
something that's quick". So a tile that is too dark, too light, too flat, too busy or off-colour is
fixed in the grade table first; only a tile wrong in shape or idea is regenerated.

- **Sources stay untouched**: every graded tile's approved drawing (and its CTM variants) lives in
  `mods/<mod>/source/blocks/tiles/`; the game's tile is built from it.
- **The table**: `mods/base/source/blocks/grade.toml`, material groups (stone, ore, wood, planks,
  leaves, grass, sand, soil, clay, plants, liquids) and per-tile rows, with the knobs gamma, gain,
  contrast or spread (target p10-p90 in codes), saturation, hue and pull. The file explains them.
- **One command**: `python3 notes/blocks/regrade.py --all [--packs DIR] [--preview out.png]` rebuilds
  every graded tile in under a second, prints luma, saturation and spread before and after against
  the vanilla band, and draws a tiled before/after of each changed tile. `--with other.toml --dry`
  previews proposals without writing (`notes/blocks/vanilla_proposals.py <packs>` writes the vanilla
  comparison's as `grade_proposed.toml`; approved rows move into `grade.toml`).
- **Item icons too**: `[item_groups.*]` and `[items.*]` in the same table grade
  `mods/<mod>/assets/textures/item/` from `mods/<mod>/source/items/tiles/`; the knob `outline` scales
  an icon's 1-px edge only (our items guide: a dark shade of each part's own colour, 15-45%).
  `notes/blocks/items_audit.py` checks the set against the guide, our own medians and seven packs.
- **Ores in a rock** are rows with `base = "<host tile>"` and `overlay = "<name>"`: the overlay
  (drawn alone on magenta, `mods/base/source/blocks/overlays/`) is laid on the host's own graded tile
  and variants, so the ore follows the rock's tuning.
- **Safe with other sessions**: a tile graded for the first time is adopted from the game as its
  source; a game tile someone replaced since the last build (`grade.lock.json`) becomes the new
  source and is graded on top, never silently overwritten. After `blocks.py install`, run the grade.

## Made in code, not generated

`notes/blocks/derive.py` makes what must keep its layout or is a recolour of an approved tile,
then `blocks.py install batches/derived.tsv` puts it in: worn farmland (paler per level), stained
glass (the glass recoloured, see-through inside), water and lava (16 looping frames; water light
grey for its tint), and the small model textures (beds, torches, lever, button, repeater,
comparator, anchor, floater, observer back, piston inner): the old 16 px picture doubled, each of
its colours drawn with the matching approved material. Rerun it after a material it uses changes.

Spring bloom on leaves (`notes/blocks/bloom.py a|b <out>`): each tree's flower as small stamps on
its approved leaf tile, wrapping across the edges. A tinted (grey) leaf blooms as a pair, the
engine's tinted overlay as on grass_side: `leaves_<x>_bloom` (untinted base: silhouette plus
flowers in their own colours) and `leaves_<x>_bloom_overlay` (the leaf with the flowers cut out).
A leaf drawn in its own colours (willow, creosote, juniper) blooms as one tile. Copy the sources
to source/blocks/tiles and the game folder, then `regrade.py --all` (the leaves group grades them).
`bloom.py bushes <out>` adds flowers to the fruiting bushes' stage 2 (their drawings kept in
source/blocks/bloom/, outside the grade); copy the results to the game folder. Cloudberry keeps its
one flower to a stem.

## Colour ramps (dark to light)

From Ronald's guide, Part 3 B (measured, then lifted with gamma 0.85). Wood and metal may lean
toward the UI's own colours (`notes/STYLE.md`).

| Material | Ramp |
|---|---|
| Grass top | `#5b733d #708c49 #85a858 #9cc366` |
| Short grass | `#627b40 #78964f #8aad5b #98bf64` |
| Dirt | `#483633 #5a433b #6b5342 #856549 #a1815a` |
| Sand | `#e0d396 #e4d9a8 #eae0ab #f0e5b6` |
| Gravel | `#3e3730 #494336 #534b3d #5a5d58 #aeafa0` |
| Snow | `#dce3ea #e0e7ec #e6eaf0` |
| Stone | `#4e5250 #707570 #8fa097 #9daea4 #b3c3bb` |
| Cobblestone | `#393d39 #4f564e #7c8576 #939e8f #a3b3a9` |
| Deepslate | `#302925 #534d55 #63626c #6d717b #7b8990` |
| Obsidian | `#1f1c23 #2b2633 #3a3746 #454152 #6a6577` |
| Oak bark | `#372f23 #44392b #534734 #6a5a41 #8c7552` |
| Oak planks | `#2e2720 #4d4035 #6c5c42 #7e694e #957e59` |
| Spruce planks | `#221f1f #382b28 #543d34 #62453d #785548` |
| Birch log | `#81786f #b0ada6 #c6c6c1 #d7d7d5 #e4e4e4` |
| Oak leaves | `#344617 #425a1d #587827 #75a035 #83b23b` |
| Spruce leaves | `#1b2d17 #253a20 #33512b #3d6235 #497342` |
| Ripe wheat | `#655839 #947d47 #b39a60 #cab87e #e7ddb5` |
| Pumpkin | `#9b512c #ba5e26 #d87525 #e99426 #f0b64e` |
| Red mushroom | `#990d07 #a3170d #ae1c10`, spots `#f6e4e1` |
| Stone bricks | `#303635 #6c7a73 #86968f #94a39c #acb7b0` |
| Bricks | `#453526 #7a5239 #ac684c #c17d52 #d6b798` |
| Sandstone | `#6c5a48 #c5b27a #e8dcad #efe4b9 #fcf4d4` |
| White wool | `#cfcac4 #e1dedb #e4e3df #e9e7e4` |
| Gold block | `#4e4226 #8e6f41 #ddb466 #fee569 #feffec` |
| Copper block | `#432712 #9c5d2c #a05d2d #de8d51` |
| Lava | `#231414 #411612 #7b1613 #cd1c18 #f39c00` |
| Glowstone | `#925929 #af6a34 #ce9a59 #f0cc53 #fafac2` |

## Known weak spots

- **Thin plants go muddy at 32 px** (the first cattail almost vanished, the mushroom's spots
  blurred): give them a clear silhouette, a size in px, and few parts. If one still reads
  badly, ask for "a 1-px outline in a darker shade of its own colour".
- **Tile values follow the daytime report** (`reports/Daytime look targets.md`, (e); Ronald, 11-04:
  "Yes, switch", replacing the earlier "at least 0.24" floor). Measure luma (0.2126 R + 0.7152 G +
  0.0722 B on sRGB, 0..1) over opaque pixels, tinted tiles with vanilla's plains tints:
  - tile means from **0.15 to 0.85**; snow and sand near 0.8-0.95, so the top of the range is used;
  - every pixel at least **30 sRGB codes** (50 for a strict check); obsidian and coal are the exceptions;
  - **darkness in shade is the shader's job**: don't lift a whole dark tile to make shade readable;
  - leaves darker than the ground they shade, about 0.25-0.32;
  - in-tile contrast as the p10-p90 spread in codes: **quiet ground 20-30** (grass top, stone, sand,
    podzol), **busy blocks 45-80** (cobble, dirt, logs, planks, ores). Ronald, 11-04: "Yes, stone,
    gravel, planks": stone calmed to 30, gravel to 42, planks given grain at 48.
  - **walked-on ground's saturation** (Ronald, 11-04: "Yes, apply"; the red sand floor read neon orange
    in the PC's light, since Lighting's look adds saturation): keep it in the middle of vanilla, Faithful
    32x and Pixel Perfection, not at their top. Measured bands: red sand about 0.63 (vanilla 0.83, Pixel
    Perfection 0.50), level with our red sandstone (0.62); dirt and coarse dirt 0.48 (vanilla 0.50, PP
    0.35); leaf litter 0.55; sand 0.38, grass top 0.53 (tinted), podzol 0.42, gravel 0.25 already fine.
    A Cretaceous copy of a tile (animals) gets the same row as base's.
  Fix a tile's level, contrast or colour in the grade table, not in its pixels (see "The grade"). The six tiles lifted for the
  earlier floor (slate, termite mound, red sandstone, sequoia and spruce bark, podzol) stay until
  they're re-judged under Lighting's new grade; leaves and grass top wait for it too ("After the
  grade").
- **Close to vanilla's colour, so packs and shaders fit** (Ronald, 11-04: "it doesn't have to be
  one-to-one matching, but it has to be close, because when it's similar, then also the shaders or
  other resource packs will just fit in seamlessly"). Check every new or changed tile against vanilla
  1.21.4 and two vanilla-style packs (Faithful 32x, Pixel Perfection CE):
  `notes/blocks/vanilla_compare.py <worktree> <packs dir> <out>` then `vanilla_sheet.py` for the sheet.
  Fetch the packs' `assets/minecraft/textures/block/` with a sparse git checkout into the scratchpad
  (InventivetalentDev/minecraft-assets branch 1.21.4, Faithful-Resource-Pack/Faithful-32x-Java
  java-latest, Athemis/PixelPerfectionCE master), read them as images only, and never commit them.
  A tile passes when its mean hue is within 15 deg (coloured tiles only), luma within 0.05 and
  saturation within 0.10 of the packs' range, tinted tiles measured with vanilla's tints (grass
  #91BD59, foliage #77AB2F, spruce #619961, birch #80A755, lily pad #208030). Ours without a vanilla
  twin are compared with their nearest relative (sequoia with spruce, termite mound with terracotta,
  canyon bands with sandstone and terracotta) and may keep their own identity.
- **Streaks and stripes never repeat in step from block to block** (Ronald, 11-04, on the karst
  limestone: "Yes, loosen streaks"; the first version put rain streaks in the same columns in every
  variant, and a tall tower looked corrugated). Give each variant its own positions, gaps and
  lengths, ending inside the tile, so a face reads as one weathered surface. Strata that must join
  sideways (a band's level lines) are the exception: those rows stay fixed and only run across.
- **Blades finer than a game pixel break into dots** when cut (the first two-block grass's upper
  half, 11-04): ask for "each blade 2 px wide (never 1 px), solid shapes that read at 32 px".
- **Random variants go in after `install`, and `install` wipes them** (coral, 11-04): `install` removes
  a tile's `<name>/` variants and `<name>.ctm` on purpose, so reinstalling a batch (to add its other
  rows) deleted the coral blocks' variants. Install a batch once, then copy the `_vN` tiles to
  `<name>/N.png` and write the `.ctm`; if a batch must be reinstalled, put them back afterwards, and
  copy the ungraded tiles to `source/blocks/tiles` too, or the grade takes an already-graded tile as its
  source and grades it twice.
- **Check a name is free in `tiles/` before a batch**: a new `fern` overwrote the approved
  Cretaceous fern's tile (11-04). Give a new plant its own name (`bracken`) and let the block point
  at it.
- **Two-block plants** (tall grass): put the upper half above the lower in one image (top right
  over bottom right) and say both continue across the line; check the joining rows meet.
- **Ores**: the base must be the same stone as `stone.png`; generate the ore in the same image
  as stone, or say "exactly the same stone base as the stone texture".
- **Families drift between images**: keep a tree, a crop's stages or a colour set in one image,
  and recolour a set (wool, glass) from one approved base in code rather than prompting each.
