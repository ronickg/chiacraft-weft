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
4. **Generate**: `python3 notes/blocks/blocks.py gen batches/<x>.tsv [image ...]` (4 Codex runs at
   once, a fresh `codex exec` each, so nothing drifts; an image that exists is skipped, delete it
   to redo). Look at each `gen/<image>.png`.
5. **Cut**: `python3 notes/blocks/blocks.py cut batches/<x>.tsv` writes `tiles/<name>.png`.
6. **Preview**: `python3 notes/blocks/blocks.py preview batches/<x>.tsv` writes
   `preview/<x>.png`: each tile flat, and as it stands in the world (a cube with its own sides
   and top, or a plant on the grass block, tints applied). Look at it, then show Ronald.
7. **Install** what he approves: `python3 notes/blocks/blocks.py install batches/<x>.tsv`. It
   replaces the texture of the same name and removes its old random variants and connected
   rules (`<name>.ctm`, `<name>/`), which would show as patches. A new block's texture still
   needs the block registered (`mods/base/src/blocks.rs`, `.texture("name")`).
8. **In the game**: `./target/release/weft --world <fresh name> --script notes/blocks/scene.txt
   --shot-dir "$PWD/notes/blocks/preview"` builds a test patch (stone, cobble, mossy cobble, dirt,
   sand, gravel, three logs, an oak) and shoots it. Copy and change it for other blocks.
9. **Approve**: add the approved tiles to `approved.txt`; they become part of the style key.
   Commit the tiles, the batch and the game textures together, one family a commit.

## Made in code, not generated

`notes/blocks/derive.py` makes what must keep its layout or is a recolour of an approved tile,
then `blocks.py install batches/derived.tsv` puts it in: worn farmland (paler per level), stained
glass (the glass recoloured, see-through inside), water and lava (16 looping frames; water light
grey for its tint), and the small model textures (beds, torches, lever, button, repeater,
comparator, anchor, floater, observer back, piston inner): the old 16 px picture doubled, each of
its colours drawn with the matching approved material. Rerun it after a material it uses changes.

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
- **Ores**: the base must be the same stone as `stone.png`; generate the ore in the same image
  as stone, or say "exactly the same stone base as the stone texture".
- **Families drift between images**: keep a tree, a crop's stages or a colour set in one image,
  and recolour a set (wool, glass) from one approved base in code rather than prompting each.
