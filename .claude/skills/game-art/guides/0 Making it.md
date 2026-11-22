# Making it: from a guide to a file in the game

The numbered guides say what the art should look like. This page says how to get an image model to
actually draw it, how to check what comes back, and how it becomes a game file. It holds what the
item lab, the UI kit and the model skill learned (2026-10 to 11), so every guide shares it.

## Which guide for what

| You want | Guide | Made as |
|---|---|---|
| An item icon (tools, food, materials, anything held) | `1 Items` | one 32 × 32 sprite per image |
| A block face (cube, plant, crop, liquid, glass) | `2 Blocks` | one 32 × 32 tile per image |
| A block with its own shape (torch, lantern, door) | `3 Model Blocks` | faces per image, then a model |
| An animal | `4 Animals` | colors, coat and face for its box model (Part 4 of that guide); a sprite only as a concept |
| A monster or character | `5 Monsters and Characters` | as animals |
| A UI part (slot, frame, button, bar, icon set) | `6 UI` | parts on a sheet |
| A particle or effect | `7 Effects` | frames in a strip |
| Sky (sun, moon, clouds, stars) | `8 Sky` | one image per piece |

## Facts about our game the prompts rely on

- **Items stay flat, like Minecraft.** An item is its icon, held and dropped as a picture one
  texel thick. No 3D item models (tried and set aside).
- **Icons are 32 × 32** and drawn at 16 GUI pixels (the UI is 2 image pixels a GUI pixel), on a
  dark brown slot `#352015` inside bronze frames. A near-black outline vanishes on that slot.
- **Grass tops, leaves, short grass, ferns and water are drawn grey and tinted by the biome**
  (block tint `core:grass`, `core:foliage`, `core:water`). A grass block's sides carry their own
  color. Ask for light greyscale for those textures; full color only for a look test.
- **Animals and monsters are box models** (the `blocky-model` skill: boxes, a painted texture,
  32 texels a block like the blocks and items, light baked in by its painter). An image model cannot lay out a model's
  texture: from it we take colors, coat patterns and faces, never the skin itself.
- **Block textures are 32 × 32** (Ronald's choice, 2026-11-03, after the style guide test), like
  the items. The whole block set, redone at 32 from the block master prompt, is in the game
  (`guides/2 Blocks/Full set.md`, `notes/blocks/`, merged 2026-11-03; it was the `block-textures` skill until 11-05).
- **One pixel size everywhere: 32 a block.** Box models are being rebuilt at 32 texels a block to
  match (Ronald, 2026-11-04, "Yes, rebuild at 32"; the session "Animal models rebuilt at 32 texels
  a block"); very small creatures still paint finer so their faces fit.
- Our things, by kind: tools in wood, stone, iron, gold and diamond (pickaxe, axe, shovel, hoe,
  sword); club, tranquilizer dart, arrow; leather, iron, gold and diamond armour; raw iron and
  gold, ingots, coal, diamond, amber, redstone; many seeds, grains, berries, nuts and wild fruits;
  meats and fish raw and cooked; buckets (water, lava), the sedative; book, paper; spyglass,
  torch, lever; eggs (raptor, chicken), saddles (utahraptor, wyvern), mushrooms, earthworm,
  insects, grubs. Animals: capybara, chihuahua, cat, Velociraptor, Utahraptor, wyvern.

## Prompting (ChatGPT image generation through Codex)

Generated here with the Codex command line on Ronald's ChatGPT plan, one run per image:

    codex exec --skip-git-repo-check --sandbox workspace-write --ephemeral \
      -i <reference> -i <ui sheet> -i <grid guide> -o last.txt "<prompt>" < /dev/null

- **Every image is a fresh run with the whole prompt and every reference attached.** In a long
  chat the model drifts from the prompt after a few images; a fresh run does not.
- **The exact grid.** Say: "draw it on an exact 32 × 32 pixel grid: the image is 1024 × 1024, so
  every sprite pixel is one 32 × 32 block of a single flat color", and attach the grid image
  (32-pixel cells on magenta) as the last reference. Without it the model's pixels are uneven and
  come out at no single size; with it they land on the grid. It sometimes returns 1254 × 1254
  instead: the grid is still 32 cells across, so shrink it with nearest neighbour.
- **Our key first, then the guide's sheet, made ours; keep "Minecraft" out of the prompt.** The
  style key is our own approved art (`Our key.png` in each guide's folder, made from sets Ronald
  approved), attached first. Since guides 2 (Ronald, 11-05) the guide's reference sheet for the
  kind comes next (its "Reference images to attach" table; whole sets also get their set sheet,
  13-26 for items), with the family's "How each family looks" notes and its "Make it yours" line in
  the prompt, so the model takes how things are built and not their designs. The sheets are cut
  from a resource pack: never commit them, and look at every result for a copied design.
- **Attach our style key of the same kind and the game's UI sheet**, and say what to take from each ("its size, outline, light and detail, not its exact
  design"). Without a pixel-art reference and the words "pixel art", it paints realistic 3D
  renders. "3D object", "hand-painted", "Hytale" pull it toward realism.
- **No real transparency**: it paints a checkerboard. Ask for flat magenta `#FF00FF` everywhere
  outside the thing, and flat green `#00FF00` when the thing itself is purple, pink or magenta
  (a sedative, a potion, an amethyst).
- **Short positive rules work; long lists of "never" are partly ignored.** Say what it should be.
- **Its default outline is one saturated red-brown line round everything.** Ask in words for
  "each part outlined in a darker shade of its own color, lighter on the upper-left edges, darker
  on the lower-right; the outline changes where the material changes".
- Ask for PNG, one thing per image (a sheet of many drifts in size and style), and write no code.

## Checking what comes back

Image models drift, so every image is measured before it is used, and one that fails is made
again (once or twice) rather than fixed by hand:

- **Chunky pixels**: its color edges fall on one regular grid (`check_items.py`'s
  `grid_strength`: pixel art 0.16 and up, smooth paintings about 0.06; 0.12 to pass).
- **Outline**: the edge is darker than the inside, and not the red-brown line (under 45% of the
  edge). A thing that is red-brown itself (a spider, a brown hide) has its own outline flagged
  as that line: look, and keep it if the outline is its own darker shade.
  A dark, glossy thing (coal) has bright glints on its edges, so its edge measures lighter than
  its inside and fails the outline rule too: look, and keep it if it reads right.
  Glass (a potion flask) is outlined in white on purpose (the item guide's recipe), so it fails the
  outline rule by design: pass `--pale` for glass.
- **Full faces** (a chest front, a workstation face, a cloud or star tile) fill their canvas: make
  them as tiles (`--tile`, blocks without `--cutout`), or the fill check fails them.
- **Background**: flat magenta (or green) all round, nothing stray on it.
- **Fit**: an item inside the grid with its margin; a block filling every cell; a tile that
  repeats without a seam (compare its left and right, top and bottom edges).
- Then look at it: whether it is the thing asked for is for eyes, not numbers.

## Cutting it

- **The exact cut**: fold the color edges onto a 32-pixel period to find the grid's phase, then
  take the most common color in the middle of each cell. That gives the sprite exactly as drawn,
  at 32 × 32, with nothing smoothed or merged.
- Don't "clean" it afterwards (guessing a grid, reducing the palette, voting colors): that was
  tried and mangled the icons. The one cleanup kept is `art.py`'s `pixfix` on item icons (Ronald via
  Planner, 11-22, after the pickaxe colouring test): a speckle pixel with no neighbour of its own
  color takes its neighbours' color (highlights and 1 px details of another material stay), and
  the silhouette's edge becomes an unbroken dark outline in each material's own deep shade.
  `NAME_raw_px.png` keeps the cut before it; `--no-fix` skips it.
- Magenta (or green) cells become transparent; every other pixel is fully opaque.
- Plants with pink, purple or magenta flowers AND green leaves go on flat `#00FFFF` cyan (`--cyan`): magenta would eat the petals and green the leaves (the 45 biome flowers, 11-04).
- On green, don't ask for a shadow under the thing: it gets drawn in the key color and stays as a
  green fringe. Key out pixels where green clearly beats red and blue (the beans, 11-04).
- Glows (the sun, fire, magic) blend into magenta as a pink ring: ask for glows on green, or take
  the pink pixels out after cutting (`r > 200, g < 160, b > 100`).

## Part-built tools (crafting)

Tools built from parts (head, haft, binding; blade, guard, grip, pommel) sit on fixed sockets on the
32 px diagonal. What Ronald's first answer taught (11-04, "axe heads are kinda bad"): a head
recoloured from one material's map loses what makes each material read (crude knapped stone and
carved wood, gold's flared shape and shadow bands, diamond's facets), and the collar thins out.
So every head is drawn per material (an override: cut from that material's drawn tool by position,
`notes/ui-codex/parts/heads.py`, set on the shared socket at the average of the heads' bases),
crude heads go on a plain wooden haft (its own part shape), and the shade map with a material ramp
is only the fallback for a material with no drawn head yet, with 10 fill steps, not 5.

## Keeping it

Nothing generated is ever thrown away: each try, cut sprite, review sheet and replaced texture goes
to `/Users/ronald/Coding/chiacraft-archive/images/chiacraft-art/<date>/`, outside the repo, and is
never deleted (Ronald's rule, 2026-11-04). `art.py` does it on its own.

## Showing it

Every new set is shown on a page before it goes into the game: as generated, the cut sprite at
several sizes in the game's slots, the game's current version beside it, blocks tiled 3 × 3, and
for a set of blocks a small world scene, so items, blocks and animals are judged together.
