---
name: game-art
description: Generate any art for the game from Ronald's style guides — item icons, block textures (cubes, plants, ores, crops, liquids, glass, workstations), animal and monster looks, UI parts, effects, sky — with ChatGPT through the Codex command line, then check it, cut it to exact pixels, show it on a page and put it in the game. Use whenever Ronald says "generate/make/draw/redo <anything visual>", asks for new or redone icons, textures, sprites, UI parts or an icon set, wants to see how art looks in the inventory or the world, sends generated images to put in the game, or wants a style guide changed. Box models themselves (creatures, props) are built by blocky-model; this skill gives them their colors.
---

# Game art: generate anything from the style guides

One way for every kind of art. Ronald says what he wants ("generate a copper pickaxe", "redo the
dirt texture", "make the wyvern's look"); Claude works out the kind, picks the guide, the type
line and the reference image, writes the description, generates, checks, cuts, shows, and installs
on his yes. Claude doesn't draw these pictures itself (a kit painted in code was judged ugly); it
makes every variant of an approved part (states, recolors, sizes) in code.

## The guides: the one place the style lives

`guides/` holds Ronald's style guides and their reference images, kept up to date here (his
originals: `~/Downloads/Game Art Style Guides`). Change the style by editing a guide, never by
patching a prompt in a script: every script reads the guides.

| Folder | For |
|---|---|
| `guides/0 Making it.md` | read first: which guide for what, facts about our game, how to prompt, check, cut and show |
| `guides/1 Items/` | item icons, 32 × 32: master prompt (section A), material sheet (B), the reference image per kind of item, recipes for tricky objects (D) |
| `guides/2 Blocks/` | block textures: how Minecraft-style textures work, the master prompt with one type line per texture kind, ramps, the full-pack checklist |
| `guides/3 Model Blocks/` | blocks with their own shape (torch, lantern, door) |
| `guides/4 Animals/` | animal looks; Part 4: our animals are box models, what carries over |
| `guides/5 Monsters and Characters/` | as animals |
| `guides/6 UI/` | UI parts (the kit below follows it) |
| `guides/7 Effects/`, `guides/8 Sky/` | particles and effects; sun, moon, clouds |

**Never attach other games' or texture packs' art to a generation, and keep "Minecraft" out of the
prompt** (Ronald's rule; he finds Faithful too clean and wants our own style). The guides'
reference images are cut from a resource pack: read them, don't attach them. `art.py` attaches
our own style key, `Our key.png` in the guide's folder (approved sets of ours), and warns when a
kind has none yet.

**Never throw away a generated image** (Ronald, 2026-11-04: "never throw away generated images pls
so I can also check back"). Every try, every candidate, every review sheet and every old texture an
install replaces is kept for good in `/Users/ronald/Coding/chiacraft-archive/images/chiacraft-art/<date>/`
(outside the repo; `ART_ARCHIVE` overrides it). `art.py` does this on its own (`generated/` each try,
`cut/` each sprite, `replaced/` each old texture), and the redo page's builder keeps each sheet before
it changes (`sheets/`). Anything else that makes images copies them there with `art.archive()`.
Never delete from the archive, and never `rm` a generated image anywhere else either.

**Ronald's answers are lessons.** After every batch he approves or sends back, write what his answer
taught into the guide (for items, the "Learned from Ronald's answers" list in the item guide's section
A, which goes into every prompt), so the next generation gets it right the first time.

When a lesson is learned (a prompt that works, a check that catches a drift), write it into the
guide it belongs to, or into `0 Making it.md` if it holds for every kind.

## Generating: `scripts/art.py`

    P=~/.venvs/blocky/bin/python; A=.claude/skills/game-art/scripts/art.py
    $P $A refs                                   # the type lines (and the guides' reading sheets), by kind
    $P $A item copper_pickaxe --what "a copper pickaxe: bright orange head ... lying diagonally, handle bottom left, tip top right"
    $P $A block dirt --type "natural cube" --what "rounded brown dirt clods (#3a2926 to #94724b) ..."
    $P $A block poppy --type flower --cutout --what "a red poppy (#d32228) ..."
    $P $A animal wyvern --type "dinosaur" --size 48x32 --what "a wyvern: ..."
    $P $A batch jobs.json                        # many at once (four at a time): [{kind, name, what, type, ...}]
    $P $A prompt item NAME ...                   # print the prompt and references instead
    $P $A install item NAME --mod base           # the cut sprite over mods/<mod>/assets/textures/item/NAME.png

Each run builds the prompt from the guide (master prompt, material colors or ramps, the type line,
the exact-grid text with the grid image attached last, our style key and the UI sheet), runs a fresh `codex exec` (a minute or two), checks the image, and makes it again (up to
`--tries 3`) when a check fails. Output in `run/art/` (outside git): `NAME.png` as generated,
`NAME_px.png` the exact sprite, `NAME_checks.json`, failed tries as `NAME__tryK.png`.

Writing `--what` (the part that decides most):
- Material and colors first, then shape, then view and placement, from the guide: tools diagonal
  (handle bottom left, tip top right), blocky things three-quarter top view, armour and containers
  front view. Colors from the material sheet (items) or ramps (blocks), as hex where it matters.
- For a tricky object (bucket, potion, bow, book, helmet, shield, rod, shears, compass, spyglass,
  torch...) paste the recipe's "Prompt line" from the item guide's section D.
- `--green` for anything purple, pink or magenta (sedatives, potions, amethyst); `--pale` for pale
  things whose outline may be light (paper, feathers, string).
- Grass tops, leaves, short grass, ferns and water go in the game grey (tinted by biome): ask for
  "light greyscale for biome tinting" unless it's a look test.
- Our own things and real facts (a capybara's coat, a ginkgo seed's shape): look them up and say
  them; the guides only know Minecraft's set.

Then look at every result: the checks catch drift (a smooth painting, a black outline, the
red-brown line, a seam), not whether it is the thing asked for.

## Kinds of art and where they go

| Kind | Made by | Into the game |
|---|---|---|
| Item icon | `art.py item` | `install item NAME --mod base|animals` (32 × 32, `mods/<mod>/assets/textures/item/`); a new item also needs its item definition and a lang key |
| Block face | `art.py block` | `install block NAME` (32 × 32, decided 2026-11-03; the game's whole 32 px set and its batches are the `block-textures` skill's, `notes/blocks/`) |
| Animal, monster | `art.py animal` (a concept sprite) | its colors, coat and face go to the box model (`blocky-model`: coat data, `house.MATERIALS`); never a sprite in the game |
| UI part | the UI kit flow below | `mods/base/source/ui/` cutters, `install.py` |
| Effect, sky | the guide's prompt by hand with `codex exec` | as the guide says |

## Showing it

Before anything goes into the game, show it on a page: as generated, the cut sprite in the game's
slots (dark brown `#352015`, bronze frame) at 2-4×, the game's current version beside it, blocks
tiled 3 × 3, and for blocks a small world scene with items and animals, so they're judged
together. The worked example is the **Style Guide Test** page (`notes/ui-codex/guides/`: `gt.py`
made the set, `build.py` draws the cards and the isometric island, `page.html` is the template).
The older **Item Icon Bench** (`notes/ui-codex/viewer/`) shows the game's current icons.

## Item icons in the game now

The game's icons are 32 × 32 (the "A2" set, merged 2026-11-03). Earlier ways, kept only as history:
sheets of 16 items cut and shrunk smoothly to 48 × 48 (`gen_items.sh`, `items_style.txt`,
`icons.py`), and the item lab's prompt rounds (`notes/ui-codex/lab/`, where the exact grid with
its guide image won). Don't "clean" generated pixel art (grid guessing, palette cuts): it mangled
the icons. Items stay flat pictures, as in Minecraft; 3D held models are set aside
(`mods/base/source/items/`).

## UI parts (the kit)

UI parts come from the master prompt below, run through Codex (`notes/ui-codex/gen.sh NAME
request.txt`, which attaches `approved_parts.png` and Ronald's two mockups,
`notes/ui-reference/hud-in-game-mockup.jpg` and `screens-overview-mockup.jpg`). Many small parts
can share a sheet ("one game pixel is 6 image pixels", "plenty of magenta between parts"); glows
come out as a pink ring on magenta, so take those pixels out after cutting. The UI guide
(`guides/6 UI/`) is the style; the master prompt below predates it and matches it.

Master prompt, pasted once per chat, with the reference sheet for that kind of part
(`notes/ui-reference/`) and every approved part attached (the approved slot at least):

```text
You are making UI assets for my game, one component at a time. Keep everything in this chat in one consistent style.

Style: copy the attached images exactly: the same copper-bronze frames with a dark outline, a bright top edge and lit left side, shaded right side, small 45° cut corners with faceted corner blocks, dark charcoal-brown insides, warm ivory symbols, amber-gold for selected, grey-brown for disabled. Light always comes from the top left.

Every asset I ask for:
- Chunky pixel art at the size I give in pixels (a slot is 32 x 32 pixels), enlarged with hard square pixels of one even size and sharp edges.
- Only that one asset, centered, with its states side by side in one row, all the same size and shape, evenly spaced.
- On a flat solid #FF00FF magenta background, filling the whole image.
- Empty: the game adds text, numbers, items and glow.
- Straight edges plain and even along their length, details only in the corners, so the game can stretch it.
- Landscape image, 1536 x 1024.

Reply "ready" and wait for my first request.
```

One message per part:

```text
Next asset: [name], [W x H] pixels, states left to right: [list]. [Anything specific.]
Match the attached approved slot exactly in frame thickness, colors and corners.
```

Several small parts may share a sheet ("items left to right: ..."); stacked states (timer strips)
are fine too. Sizes asked for:

| Part | Size | States |
|---|---|---|
| Slot | 32x32 | normal, hover, pressed, selected, disabled |
| Window panel | 64x64 (cut at 96 GUI px, border 21) | normal |
| Title plate | 64x16 | normal |
| Well (sunken area), tooltip | 48x48, 48x32 | normal |
| Close button | 16x16 | normal, hover, pressed |
| Button | 64x20 | normal, hover, pressed, disabled |
| Tab | 56x20 | normal, hover, active, disabled |
| Text input | 96x20 | normal, focused, disabled |
| Checkbox, radio | 16x16 | off, on, disabled |
| Toggle | 32x16 | on, off, disabled |
| Slider | track 96x8, fill 96x6, knob 16x16 | knob normal, hover |
| Scrollbar | track 12x64, handle 12x28, arrows 12x12 | normal, hover |
| Bars | frame 128x14, fills 128x8 | health, stamina, XP |
| Food buff slot, timers | 28x28, 24x4 | green, yellow, red |
| Temperature | bar 80x8, marker 6x10 | |
| Level badge, toast, crosshair | 20x22, 160x32, 9x9 | toast normal, success; crosshair normal, hit |
| Furnace progress | arrow 24x16, flame 14x14 | empty, full |
| Slot markers | 10x10; durability 24x4 | durability green, yellow, red |
| Icon sets | 24x24, rows of 8 | |

## Material palette (planned, not built yet)

Ronald wants a material to keep one color everywhere: a tool's oak handle the color of oak planks,
in the icon, the block and the model. Plan: one palette file with a ramp of 5-7 shades per
material (oak, birch, spruce, stone, iron, gold, diamond...), the one source for block textures and
models (painted from it in code), and for icons: the hex colors go into the prompt, then after
cutting, each material area of an icon is found by hue and remapped onto its ramp by brightness (a
gradient map), so every oak handle comes out exactly the same. The same remap makes other woods'
variants (a birch handle) without asking ChatGPT again. A check reports icons far off the palette.

## Cutting the sheets

All in `mods/base/source/ui/`, run on the PNG Ronald sends, into the scratchpad first:

| Sheet | Script | Example |
|---|---|---|
| One part, states in a row | `extract.py` | `extract.py slot.png out/ --names slot,slot_hover,slot_pressed,slot_selected,slot_disabled --size 32 --corner 10 --smooth 2` |
| Stacked bars (empty frame, then filled) | `bars.py` | `bars.py bars.png out/ --names bar,bar_health,bar_stamina,bar_xp --size 128 --height 14 --smooth 2` |
| Icon grid, item grid | `icons.py` | `icons.py items.png out/ --names iron_ingot,gold_ingot,... --size 24 --smooth 2` |
| Mixed parts of different sizes | `sheet.py` | `sheet.py s.png out/ --names "track,fill,knob/knob_hover" --ref track=96 --border track=4,fill=3` |

- `--names` in reading order. `sheet.py`: rows by `;`, parts by `,`, a stack in one place by `/`;
  `--ref name=width` sets the scale from one part's width in GUI pixels; `--border` names the parts
  that stretch (nine-slice) and their border; the rest are pictures (`scaling = stretch`).
- Each part gets `<name>.png` (2 image px a GUI px) and `<name>.gui` (`scaling`, `border`,
  `size` in GUI px), the format the engine reads.
- The background key is the corner color; near colors go, plus magenta-tinted edges on magenta.
  If a part loses pixels (a purple fill on magenta), ask for it again on green.
- A script that reports the wrong count ("found 11 parts") means two parts touch or one has a
  gap: look at the sheet, then split by hand or ask for more space between them. Diagonal tools
  often reach into the next one's columns without touching it: then take the five biggest
  connected shapes (flood fill on `sheet.background`), paste them spaced apart on magenta, and
  cut that (done for the stone tool row).
- Empty-slot equipment comes out better as faded silhouettes: `--silhouette` (luminance mapped
  to one brown, alpha 140).

## Checking and showing

Before reporting on a batch, look at every cut part enlarged next to its source and check:
outline, copper and amber match the approved parts; light from the top left; edges between the
corners plain (so they stretch); nothing of the background left in the edges; states the same
size and shape. Say which images are fine and which need a redo, and give the redo message.

Show the parts on the UI kit page (the artifact "UI kit", built from the scratchpad's
`kitpage/`: `mock3.py` draws the mock screen from the parts, nine-sliced by their `.gui` as the
engine does, and writes the text spans the page lays over it; the gallery lists every part). In
the mock, keep each window's contents inside its frame (the panel's frame is 8 GUI px).
