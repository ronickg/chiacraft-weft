---
name: game-art
description: Generated art for the game's UI and item icons, made in ChatGPT in one warm fantasy pixel style (copper-bronze frames, light from the top left) and cut into game parts here. Use when Ronald asks for a prompt for a UI part, an icon set or item icons (a new one, a redo, "the master prompt again"), when he sends generated sheets or images to put in the game or on the UI kit page, or when a generated part needs a new state, size or color.
---

# Generated game art: UI parts and item icons

Ronald makes the art in ChatGPT with the prompts below; Claude cuts each sheet into game parts
(`mods/base/source/ui/`), checks it, and shows it on the UI kit page. Claude does not draw these
pictures itself (a kit painted in code was judged ugly), but it does make every variant of an
approved part: new states (hover, pressed, disabled by shifting colors), recolors, other sizes
(nine-slice), combinations (icon on a slot, a marker in a corner). New subject matter (a new icon,
a new item, an ornate piece) goes back to ChatGPT. Block textures, creature skins and item models
are painted in code (`blocky-model`), not here; an item's icon is the reference for its model, so
the model in the hand matches the icon in the slot.

The style and the reasons behind it are in `notes/STYLE.md`; the reference sheets in
`notes/ui-reference/`.

## What ChatGPT can't do, and what the prompts do about it

- **No exact pixel grid or sizes.** Its "pixels" are uneven and it draws bigger than asked (26-33
  for 24). So prompts ask for the design and a rough size; the cutter shrinks the picture as drawn
  (`--smooth 2`: 2 image pixels a GUI pixel), every part of a sheet at one scale.
- **No real transparency** (it paints a checkerboard). Ask for flat magenta `#FF00FF`; for purple
  or pink content (poison, XP, potions, sedatives) flat green `#00FF00`.
- **Without a reference image and the words "pixel art ... small sprite enlarged with hard square
  pixels", it draws realistic 3D renders** on a dark background with grid lines (the first item
  sheet came out like that). Words like "3D object", "hand-painted shading" or "Hytale" pull it
  further toward realism. If a chat has drifted into realism, start a new chat rather than
  correcting it.
- **Style drifts between chats.** Attach the reference sheet and the approved parts to the master
  prompt; that is what keeps a new chat matching.
- **Short, positive rules work; long lists of "no X" get partly ignored.**
- Ask for **PNG**, not WebP. An image sent in the middle of a turn may not be saved to disk: if
  it isn't in the conversation's files, ask for it again.

## UI parts

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

## Item icons

A separate master prompt (the UI one would put frames round the items). The style comes from the
images attached with it far more than from the words, so always attach references:
- **Until an item sheet is approved:** `notes/ui-reference/pixel-fantasy-inventory-ui-reference.jpg`
  and `crafting-workstations-ui-reference.jpg`. They show items in the target style (ingots, ores,
  gems, tools, food in slots: three-quarter view, chunky, dark brown outline, top-left light).
- **Once a sheet is approved:** save it as `notes/ui-reference/items/<family>.png` and attach one
  or two approved sheets plus the inventory reference with every new chat. The approved sheets
  become the style key, so later sheets and redos match them.

Master prompt:

```text
For this whole chat you make pixel-art item icons for my game. Copy the style of the items in the attached reference images exactly: the pickaxes, ingots, ores, gems and food shown in their slots.

Every icon:
- Chunky pixel art, drawn as a small 32 x 32 pixel sprite and enlarged with hard square pixels of one even size and sharp edges. Not a 3D render, not a painting, not realistic: a game sprite like the reference.
- Simple shapes and few colors: 3 or 4 flat shades per color, light from the top left (top and left lighter, bottom and right darker), and a dark brown outline (not black) around the whole item.
- Warm, rich but not oversaturated colors, as in the reference.
- Seen slightly from above at a three-quarter angle, like the reference ingots and blocks. Long items (tools, weapons, sticks) lie diagonally, handle at the bottom left, head at the top right. Items of one family (all ingots, all pickaxes) share one shape and differ only in color.
- Every item about the same size, filling about 75% of its cell.
- Material tiers must never look alike, even small in a slot. Each tier differs in brightness, color and finish at once, always like this:
  - Wood: warm light brown, matte, visible grain.
  - Stone: dark brownish-grey, the darkest tier, rough and chipped with bumpy edges, no shine.
  - Iron: pale cool silver with a blue tint, the lightest grey, smooth, one bright white edge highlight.
  - Gold: saturated yellow-gold, shiny, small white glints.
  - Diamond: cyan crystal with sharp facets and white sparkles.
  Any other material I name gets its own distinct look, unlike every tier above.

Background: one flat solid #FF00FF magenta color over the whole image, unless I say green (#00FF00). No grid lines, cell borders, slots, frames, shadows, glow, text or numbers.

I send one of three kinds of request:
- A list: one sheet, the items in a grid, left to right, top to bottom, evenly spaced, no two touching.
- One item: one image with that item alone, centered.
- "Redo [item]: [what to change]": that item alone, centered, the same angle, size and shape as before, changing only what I say.

Reply "ready" and wait for my first request.
```

Requests:

```text
[rows] x [columns] grid. Left to right, top to bottom:
[item] ([material and colors], [shape]), [item] (...), ...
```
```text
One item: [item] ([material and colors], [shape]).
```
```text
Redo [item]: [what to change]. Keep everything else the same.
```

Writing the list:
- Each item by material and colors, then shape: "iron ingot (cool grey-silver bar with beveled
  edges)". Name the family shape again for each member ("same shape, warm yellow-gold").
- At most 16 a sheet (4 x 4); more come out small and alike. Group a sheet by family.
- Tiers differ in brightness, color and finish at once, not color alone: the first tool sheets had
  stone and iron both light grey, alike in a slot. The master prompt fixes the look of each tier
  (wood, stone, iron, gold, diamond); a new tier (copper, bronze, obsidian...) gets its own line
  there before its first sheet, chosen unlike every other tier, and is added here too. Check a
  new tier sheet in slots next to the other tiers before approving it.
- Family shapes to keep across sheets: ingots a beveled bar; raw ores a rock chunk with the metal
  showing; gems cut with bright facets; dusts a small heap; tools diagonal with a wooden handle;
  armor seen from the front-left, empty inside.
- The game's items: `mods/base/assets/textures/item/`, `mods/animals/assets/textures/item/`
  (file names are the item names).

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
