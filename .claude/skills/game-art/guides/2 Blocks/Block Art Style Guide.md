# Block Art Style Guide

This is the block version of `the item guide`. It covers:

- **Part 1:** how Minecraft block textures work: files, faces, models, variants, tinting, transparency, animation and connected textures. With this, a prompt or an artist can produce a complete pack, not just pretty squares.
- **Part 2:** a style analysis of the reference's blocks, measured against Alt reference 1 and Alt reference 2 (sections 1–8).
- **Part 3:** a master prompt with one sub-prompt per texture type, a block material sheet, and a checklist of every texture family.

The reference images to attach with the prompts are in `Block Prompt References/` (see Part 3).

Every number below was measured from the textures. "L" is perceived brightness from 0 (black) to 100 (white).

---

# Part 1: How Minecraft block textures work

## 1.1 Files and where they live

| Folder | What it holds |
|---|---|
| `assets/minecraft/textures/block/` | Every block face texture as a PNG. The vanilla game has about 1,300 of them. |
| `assets/minecraft/textures/entity/` | Blocks drawn as entities: **chests, ender and trapped chests, beds, signs, hanging signs, shulker boxes, banners, bells, decorated pots, conduit, the enchanting-table book, lectern book, skulls**. These are UV-unwrapped sheets, not square tiles. They're easy to miss when planning a pack. |
| `assets/minecraft/models/block/*.json` | The 3D shape and which texture goes on which face. |
| `assets/minecraft/blockstates/*.json` | Which model to use for each state of a block (facing, age, lit, axis…), plus random variants. |
| `assets/minecraft/textures/colormap/grass.png`, `foliage.png` | Biome colour lookups used to tint grey textures. |
| `*.png.mcmeta` next to a texture | Animation settings. |
| `optifine/…` (optional, needs OptiFine or a mod) | Connected textures, extra random textures, emissive glow, custom sky. |

Resolution is the width of a face: 16 px in vanilla, 32 px in the reference and Alt reference 1. All block textures in a pack should share one resolution. A model always maps a face onto a 16-unit grid, so a 32 px texture just gives each unit 2 pixels.

## 1.2 Faces and name suffixes

A full block has 6 faces. Most blocks reuse textures across faces, and the file name says which face a texture is for:

| Suffix | Meaning | Example |
|---|---|---|
| *(none)* | All 6 faces the same (`cube_all`) | `stone`, `dirt`, `white_wool` |
| `_top`, `_bottom`, `_side` | Different top, bottom and 4 sides | `grass_block_top`, `sandstone_bottom` |
| `_end` / `_top` on logs | Cut end of a column. The sides are the bark. | `oak_log` + `oak_log_top` |
| `_front`, `_back` | One special face that turns to face the player | `furnace_front`, `observer_back` |
| `_on`, `_lit`, `_powered` | Active state | `furnace_front_on`, `redstone_lamp_on`, `powered_rail_on` |
| `_stage0…7`, `_age` | Growth or crack stages | `wheat_stage0`–`7`, `destroy_stage_0`–`9` |
| `_top` / `_bottom` on plants and doors | The two halves of a 2-block-tall thing | `tall_grass_top`, `oak_door_bottom` |
| `_overlay` | A layer drawn on top of another texture | `grass_block_side_overlay` |
| `_still`, `_flow` | Liquid surface and liquid running down | `water_still`, `lava_flow` |
| `_inner`, `_particle` | Inside faces, and the texture used for break particles | `composter_inner`, `cauldron` |

An Alt reference 1 count gives a feel for the mix. Of its 1,297 block textures, 175 are `_top`, 89 `_side`, 68 `_bottom`, 61 crop or growth `_stage`, 36 door halves, 32 concrete, 32 terracotta, 33 glass and 30 coral.

## 1.3 Model types and what each needs

| Model (parent) | Shape | What to draw |
|---|---|---|
| `cube_all` | Plain cube | 1 tile that repeats on all sides. **Must tile seamlessly.** |
| `cube_column` | Pillar: logs, basalt, hay, quartz pillar | 1 side tile (tiles vertically) and 1 end tile (rings or a cross-section). The block rotates with `axis=x/y/z`, so the side texture lies sideways too. |
| `cube_bottom_top` | Sandstone, grass block, TNT, mycelium | Separate top, side and bottom. The side's top edge must meet the top face. |
| `orientable` | Furnace, dispenser, carved pumpkin, beehive | Front, side and top. Only the front shows a "face". |
| `cross` | Flowers, saplings, dead bush, mushrooms, cobweb | **2 flat planes crossed in an X**, seen from every angle. Draw the plant standing up, centred, with its base touching the bottom edge and a transparent background. |
| `tinted_cross` | Short grass, fern, sugar cane | Same as cross, but drawn in **grey** so the biome colour can tint it (see 1.5). |
| `crop` | Wheat, carrots, potatoes, beetroots, nether wart | **4 planes in a # shape**, so the texture is seen 4 times side by side. Draw a row of stalks that still reads when repeated, with the base on the bottom edge. |
| `leaves` | All leaves | A cube drawn with cutout transparency. Gaps between leaves are fully transparent, and in "Fancy" graphics you see through to the leaves behind. |
| `pane`, `bars` | Glass panes, iron bars | A thin slab using the full glass texture plus a `_pane_top` edge strip. |
| `door`, `trapdoor` | Doors and trapdoors | A door is 2 tiles (`_top`, `_bottom`). A trapdoor is 1 tile seen face-on. Both use cutout, so windows can be holes. |
| Custom `elements` | Torches, lanterns, anvils, hoppers, cauldrons, the reference's 3D wheat | A small UV sheet. Each box takes its faces from a region of the PNG. **Don't change the layout** without also changing the model. |

## 1.4 Variants: how blocks pick a texture

The blockstate file chooses a model. There are three mechanisms:

1. **State variants:** one model per property value, such as `facing=north`, `age=3`, `lit=true` or `axis=y`. Each state that looks different needs its own texture: a lit furnace front, every crop stage, open versus closed trapdoors, powered versus unpowered rails.
2. **Random variants (vanilla, no mods):** a list of models for one state. The game picks one per block position, and that choice stays the same for that spot. Each entry can set `"weight"` (how often it appears) and rotate the texture with `"y": 90/180/270` (or mirror it via the model). **This is how a pack breaks up the visible grid on big areas of grass, sand, stone or flowers.**
3. **Multipart:** several models stacked depending on state. Fences, walls, panes, redstone dust and vines use it to add arms or sides. The reference also uses it to stack extra pieces on grass blocks.

**How the reference uses random variants**, out of 532 blockstates:

- **142 blocks have random variants and 132 use multipart.**
- **Gravel** has 78 variants: rotations combined with weights.
- **Logs** have 12 per axis, each with weight 3.
- **Flowers** have 12 each (poppy, dandelion, all four tulips), built from 4–6 drawn textures in `textures/block/plants/<flower>/0.png…`.
- **Short grass** picks from 7 grass textures in `textures/block/grass/`.
- **Grass blocks** stack one of 10 models.
- **Wheat** has a normal and a mirrored model for each stage.
- **Sand** has 10 variants. **Lily pads** have 28, and **mushrooms** 24–36.
- **Rule of thumb:**
  - Natural ground and plants get 4–12 variants, or 1 texture plus rotations.
  - Built blocks such as bricks and planks get 1 texture, because rotating them would break the pattern.

On top of that, the reference uses **OptiFine connected textures**, with 327 `.properties` files in `optifine/ctm/`:

- `method=random` swaps between several tiles, for example bricks have 4 variants of each tile.
- `method=repeat` with `width=1 height=2` makes one pattern span 2 blocks.
- `method=ctm` joins glass, bookshelves, planks and logs into one large picture. The borders disappear between neighbouring blocks.

These are extras. A pack has to look right without them.

## 1.5 Tint: grey textures that the game colours

Some textures are drawn in **grey or desaturated colour**, and the game multiplies them by a biome colour. If you draw them green, they turn muddy in some biomes.

| Tinted by | Textures |
|---|---|
| Grass colormap (plains ≈ `#91bd59`) | `grass_block_top`, `grass_block_side_overlay` (only the grass fringe, drawn on top of the dirt side), `short_grass`, `tall_grass_*`, `fern`, `large_fern_*`, `sugar_cane`, `potted fern` |
| Foliage colormap (plains ≈ `#77ab2f`) | `oak`, `jungle`, `acacia`, `dark_oak` and `mangrove` leaves, `vine` |
| Fixed colours | spruce leaves `#619961`, birch leaves `#80a755`, lily pad `#208030`, melon and pumpkin stems (green to orange as they grow), redstone dust (dark to bright red with power level), water (`#3f76e4` in plains, varies by biome) |
| **Not tinted** (draw in full colour) | cherry, azalea and flowering azalea leaves, all flowers, saplings, crops, mushrooms, moss, glow lichen, dead bush |

**How to draw a tinted texture:**

- Draw it as a greyscale value study: lightest leaf tips around L 75–90, deepest gaps around L 25–35.
- Use no hue, or a tiny bit of warmth.
- After tinting, the darkest step should still read as green rather than black, so don't go below about L 20 in the grey version.

The reference ignores this for some textures: its oak and jungle leaves are already drawn green. That works because multiplying green by green just darkens it a little, but those leaves no longer change colour by biome.

## 1.6 Transparency: three kinds

| Kind | Blocks | Rule |
|---|---|---|
| Opaque | Stone, dirt, planks, ores, wool, terracotta | Every pixel fully opaque, no alpha. |
| Cutout | Leaves, plants, crops, doors, trapdoors, ladders, rails, iron bars, cobweb, torches, glass (vanilla) | Each pixel is either **fully opaque or fully transparent**. Semi-transparent pixels render as solid or vanish depending on the setting, so avoid them. |
| Translucent | Water, ice, stained glass, slime, honey, nether portal | Real partial alpha. Water is around 70% opaque. Stained glass has a nearly opaque coloured frame and a 25–60% alpha interior. |

Vanilla clear glass is cutout: an opaque frame with a fully transparent centre and a few opaque streaks. The reference frames glass with a dark wood-like border, around L 29 at the edge versus 46 inside.

## 1.7 Animation

An animated texture is a **vertical strip of square frames**: a 32×512 PNG is 16 frames. A `.png.mcmeta` file next to it controls playback:

```json
{"animation": {"frametime": 4, "interpolate": true, "frames": [0,1,2,3,3,2,1,0]}}
```

- `frametime` is ticks per frame, where 20 ticks is 1 second. In the reference, fire uses 2, water uses default timing, and lava uses 4 with `interpolate: true`, so frames blend smoothly.
- `frames` sets the order and allows ping-pong. `interpolate` blends between frames.
- Vanilla animates water and lava (still and flow), fire and soul fire, nether portal, sea lantern, prismarine, magma, stonecutter saw, command blocks, kelp, seagrass, campfire, lantern flames, sculk, respawn anchor and a few more.
- Animations should loop seamlessly: the last frame must flow into the first.
- **Flowing liquid** (`_flow`) is drawn moving down. The game stretches it to 2× scale, so keep its details twice as fine as you'd expect.

The reference has 382 `.mcmeta` files in total.

## 1.8 Glow (emissive) and light

Vanilla has no glow layer: torches, lanterns and lava look bright only because their pixels are bright. OptiFine adds an emissive overlay. The reference has 95 `_e.png` files that are drawn at full brightness even in the dark (lit furnace mouth, redstone torch, soul fire, lit lamps). If you don't use OptiFine, paint light sources with:

- a **near-white core**, around L 95,
- warm mid-tones, and
- dark surroundings, so the source pops.

## 1.9 Overlays and layered textures

| Texture | What it is |
|---|---|
| `grass_block_side_overlay` | A grey grass fringe placed over `grass_block_side` (dirt with a fringe drawn in full colour, which shows when tint is unavailable, for example on the item). |
| `destroy_stage_0…9` | Grey cracks drawn over any block while it's being mined. |
| `redstone_dust_line`, `_dot` | Grey shapes tinted red by power level. |
| Ores | Self-contained textures: the ore is painted into a copy of stone or deepslate. **The base stone must match `stone.png` / `deepslate.png` exactly**, so ore blocks blend into walls. The reference's ores share stone's exact tile. |

## 1.10 Tiling: the most important rule for cubes

Any texture that sits on a block face beside others (stone, dirt, planks, leaves, wool, sand) must **tile seamlessly**: the left edge continues into the right edge and the top into the bottom. Test a 3×3 repeat before accepting a texture.

- Avoid one big feature that repeats noticeably, like a single bright pebble or a lone dark crack.
- Spread contrast evenly, or provide random variants.
- Built blocks (bricks, stone bricks, planks) tile on their mortar or plank-joint grid. Plan the grid so joints line up across edges. For example, 32 px stone bricks are two rows of 16 px with offset joints.

---

# Part 2: Reference blocks, measured

All three references below are 32×32. Reference is the look you like, Alt reference 1 is a cleaner, flatter baseline, and Alt reference 2 is a variant of Alt reference 1.

## 1. Canvas and scale

- 32×32 per face. Plants and cutouts use 20–70% of the tile:
  - flowers 29%
  - grass and saplings 42%
  - crops 50%
  - leaves 68%
- Cutout plants stand on the bottom edge and rise to 50–100% of the height. Flowers are usually 18–26 px tall with 3–6 px of empty space at the sides.
- the reference swaps several textures for **custom models with UV sheets**: 64×64 wheat stages, 3D torches, lanterns, levers and doors with iron straps. For your own game you can ignore this and draw simple faces.

## 2. Edges, borders and outlines

There are three edge strategies, by block type:

- **Natural blocks** (stone, dirt, sand, leaves, ores): **no border.** The texture runs off the edges and tiles. Shapes such as stones, pebbles and leaves have dark crevices between them, at stone L 28 and dirt L 23–30, which read as outlines around each lump.
- **Built blocks** (stone bricks, planks, bricks, sandstone, smooth stone, quartz, purpur, end stone bricks): **each brick or plank is a bevelled tile.**
  - The light edge is on the top and left, and the dark edge on the bottom and right.
  - Mortar lines are 1–2 px of the darkest step.
  - Many of the reference's built blocks have **a full 1 px frame** around the 32 px face (smooth stone, sandstone top, quartz, workstation faces). This is the "framed block" look that makes builds look panelled.
- **Cutouts** (plants, crops, leaves): **edge pixels are darker than the inside**:
  - crops L 25 vs 32
  - leaves 31 vs 41
  - mushrooms 27 vs 36
  - flowers 37 vs 42

  That's a soft self-outline, the same idea as the items. Alt reference 1's gap is smaller, at only 2–5 points, so its plants look flatter and lighter.

## 3. Colour and palette

| | Reference | Alt reference 1 |
|---|---|---|
| Colours per texture (median) | **60–440** (ores 443, stone 226, utility 212, terrain 139) | **5–11** |
| Brightness range, 5th–95th percentile | Wide: ores 13–73, utility 9–62, metal 16–62 | Narrow: built stone 46–66, metal 39–62 |
| Mean saturation (0–1) | Stone 0.21, terrain 0.38, plants 0.51–0.64 | Similar, but crops reach 0.76 |

- **The reference is painted, while Alt reference 1 is posterised.** The reference uses dozens of close shades with soft noise. Alt reference 1 uses a few flat bands, like vanilla upscaled.
- **The reference's darks are deep.** The darkest 5% sit at L 9–28 depending on the category. Alt reference 1 rarely goes below 27.
- **Hues lean cool and grey-green for stone and warm brown for wood:**
  - The reference's stone is a **sage or teal grey** (`#68716b`→`#a6b8af`), not neutral grey.
  - Its planks are **dark walnut** (`#221c16`→`#7d684a`), darker than Alt reference 1's honey-coloured oak.
  - Its birch log is almost white, and its spruce is near-black brown.
- **Plants are muted, natural greens:** oak leaves `#27380f`→`#74a72e` and grass `#536c32`→`#8bb655`. Flowers stay saturated but small against them: poppy `#d32228`, dandelion `#e4d020`, cornflower `#365bb1`, allium `#83528f`→`#dcc2fa`.

## 4. Light and shading

- Light comes from the **top-left**, the same as the items. Every brick, pebble, plank and ore lump has a lit upper-left edge and a dark lower-right edge.
- Block faces are lit again by the game: top faces at 100%, the sides at about 80% and 60%, and the bottom at 50%. **Keep block textures flatter than items**, with mid-tones dominating, or a wall reads as noisy.
- Ore gems are the brightest thing in the block. Diamond, emerald, gold and lapis have a near-white sparkle (`#c6f9fb`, `#cfdfd8`) and an almost black socket (`#000000` on coal, `#0c214f` on lapis). That socket is what you'd soften.
- Light sources:
  - glowstone runs from `#7c4e2a` to `#faf8a8`
  - lava from `#1b0d0c` to `#f36900`
  - fire from `#cb441a` to `#f6e786`
  - The dark cracks are what make the bright parts glow.

## 5. Materials and families

| Family | Reference look (dark → light ramp) |
|---|---|
| Grass top | Leafy tufts, not noise. `#4c642f #617e3b #779c49 #8fba57` |
| Dirt | Rounded clods with dark gaps. `#3e2c27 #4e3830 #664935 #8e6d49` |
| Sand | Very light, with soft wind ripples and a few white grains. `#dbcc89 → #eee1ac` |
| Gravel | Separate grey-white pebbles on dark brown. `#312a24 … #9da090` |
| Stone | Large sage-grey slabs with dark cracks. `#404542 #68716b #88988f #a6b8af` |
| Deepslate | Purple-grey, with horizontal layers. `#251f1b → #6a777f` |
| Ores | Stone base plus **clusters of 3–8 rounded nuggets** (iron, copper, gold, coal) or **faceted crystals** (diamond, emerald, quartz), each with a dark socket |
| Logs | Deep vertical bark grooves. The top shows growth rings and a bevelled bark rim. |
| Planks | 4 horizontal boards with nail-dark ends, mortar-like gaps and a dark walnut tone |
| Leaves | Individual leaf shapes 3–5 px long, overlapping, with clear gaps. Cherry leaves are pink-white blossoms on grey twigs. |
| Flowers | Thin stems, 2–4 leaves, a head of 5–9 px. Several variants per flower, including clumps of 2–3. |
| Crops | Dense leafy clumps. Root crops show the vegetable peeking out at the bottom (carrot, beet). |
| Built stone | Bevelled bricks with dark mortar. Chiseled versions get a carved relief pattern. |
| Metal blocks | Riveted plates with a frame. Gold has highlights up to `#fcffdf`. Iron is grey with a near-white bevel. |
| Workstations | Iron-banded wood or stone, with tools and details drawn on the front face (furnace arch, anvil, bookshelf books). |
| Liquids | Water is lightly tinted, around `#3a8ad5 → #62b6f4` in plains. Lava is black crust over glowing red-orange. |

## 6. Shapes and construction

- **Clumps over noise.** Every natural texture is made of readable objects: tufts, clods, pebbles, slabs, leaves. Sand is the only one built from soft gradients.
- **Big shapes, few of them.**
  - Stone has 6–9 slabs per tile.
  - Cobblestone has 8–12 stones.
  - Dirt has 10–15 clods.
  - Planks have 4 boards.
  - Stone bricks have 2 rows.
- **Plants are silhouettes first.** A flower must read as that flower from its outline alone, for example a tulip cup, an allium ball or a cornflower star.
- **Workstation faces tell you what they do**: a furnace has a fire mouth, a crafting table has tools, a bookshelf has books, an observer has an eye.

## 7. Pixel craft

- No anti-aliasing against transparency. Cutout edges are hard.
- Soft painted noise inside large areas, a 1–2 shade variation, **without checkerboard dithering**.
- Hand-placed highlight pixels: 1–2 px specks on sand and snow, 2–3 px sparkles on gems.
- Mortar and crevice lines are 1 px, or 2 px at most.

## 8. Mood

Earthy, aged medieval fantasy: mossy sage stone, dark walnut wood, iron-banded workstations, softly glowing ores. It's darker and moodier than Alt reference 1.

---

# Part 3: Making it

## A) Master prompt for blocks (style)

> **Style:** Hand-painted pixel-art Minecraft-style block texture, 32×32, no anti-aliasing, top-left light. Earthy medieval-fantasy look, made of readable clumps rather than noise: rounded stone slabs, dirt clods, pebbles, leaf shapes, bevelled bricks, iron-banded wood. Soft painted grain with 1–2 shade variation, no checkerboard dithering, 20–60 shades per texture. Light and airy: shadows are mid-dark, crevices and mortar lines are a deep shade of the block's own colour (never black, brightness ≥ 20%), and highlights are restrained so walls don't look noisy. Stone is sage-grey, wood is warm walnut-to-honey, and grass and leaves are muted natural greens. Ores and flowers are the only bright saturated accents. Full-cube textures must tile seamlessly on all four edges. Plants and other cutouts sit on a transparent background with hard edges, each pixel fully opaque or fully transparent, and a soft 1-px edge slightly darker than the inside.

> **Background (cutouts):** flat `#FF00FF` magenta where the texture is see-through (gaps in leaves, around a plant); it becomes transparent. Full cubes fill every pixel.
>
> **Tinted textures:** in our game grass tops, leaves, short grass, ferns and water are drawn in light greyscale and tinted by the biome; a grass block's sides carry their own colour.

How to prompt, check and cut it: `0 Making it.md`.

Add **one** of these type lines after it:

| Type | Add to the prompt |
|---|---|
| **Natural cube** (stone, dirt, sand, gravel, ore base) | "Seamless tileable full-block texture, no border, evenly spread detail, no single feature that stands out when repeated." |
| **Grass block top / side** | Top: "seamless leafy grass tufts, drawn in light greyscale for biome tinting." Side: "dirt clods with a grass fringe hanging 4–8 px down from the top edge, irregular tips." |
| **Ore** | "Exactly the same stone base as [stone texture], with 3–6 rounded nuggets of [material] (or 2–3 faceted crystals) in a deep socket of the stone's own dark shade, a near-white sparkle on each, filling about 30% of the tile." |
| **Log side / log top** | Side: "vertical bark with deep grooves, tileable top-to-bottom." Top: "cross-section with 4–6 growth rings in a lighter wood, a 2–3 px bark rim, bevelled." |
| **Planks / bricks / built stone** | "4 horizontal boards" / "2 rows of bevelled bricks, offset joints" / "large bevelled stone blocks". Add: "1-px mortar lines in the darkest shade, light top-left edge on each piece, joints line up across tile edges so it tiles." |
| **Framed block** (smooth stone, quartz, metal and gem blocks) | "A 1-px bevelled frame around the face. Inner panel with [rivets / plates / facets]. Light on the top and left of the frame, shadow on the bottom and right." |
| **Leaves** | "Overlapping individual leaves 3–5 px long with transparent gaps covering 25–35% of the tile, tileable, drawn in greyscale for tinting" (or "in full colour" for cherry and azalea). |
| **Flower / sapling / mushroom (cross)** | "A single [plant] standing upright, centred, base touching the bottom edge, about 20–26 px tall, on a transparent background, readable from its silhouette alone. A thin 1–2 px stem and 2–4 leaves." Ask for 4 variants: "slightly different heights and leaf angles, and one clump of 2–3." |
| **Short grass / fern / tall grass (tinted cross)** | "A clump of blades or fronds rising from the bottom edge, in light greyscale only, for biome tinting." |
| **Crop stage N of M** | "A row of [crop] stalks rising from the bottom edge, about [height] tall at this stage, reads well when the texture is placed 4 times in a # shape." At the final stage, add the produce: "orange carrot tops peeking out at the base" or "golden wheat heads". |
| **Two-tall plant** | "Two tiles: the bottom tile has the stem and leaves rising to the top edge. The top tile continues from its bottom edge to the flower head. The stem must line up at the seam." |
| **Workstation face** | "Front face of a [block] showing [what it does], framed in [wood / stone] with iron bands. Side and top faces are plainer versions of the same frame." Include an `_on` version if it lights up. |
| **Liquid / animated** | "N frames of 32×32 stacked vertically, looping seamlessly. [Water: light greyscale-blue ripples for tinting, mostly flat.] [Lava: dark crust plates over glowing orange-yellow cracks, slow drift.]" |
| **Glass** | "A 1-px [wood-dark / pale] frame. Inside is fully transparent except 2–3 short diagonal glare streaks in the upper-left." Stained glass: "frame in the dye colour, inside the same colour at about 40% opacity." |
| **Light source** | "Near-white core (brightness about 95%) inside warm yellow-orange, set in dark [stone / iron / crust] so it glows." |
| **Colour set** (wool, concrete, terracotta, glass, candles, carpet, banners, beds) | "Same pattern, recoloured for all 16 dye colours: white, light grey, grey, black, brown, red, orange, yellow, lime, green, cyan, light blue, blue, purple, magenta, pink." Wool has soft fibres, concrete is flat with faint speckles, and terracotta is matte clay with faint layers. |

### Reference images to attach

`Block Prompt References/` has one image per type above. They show Reference examples. Cube types also include a 3×3 tiled patch underneath, so the model sees the texture has to repeat seamlessly. Attach the matching image with each prompt, and add one or two of your own UI reference sheets for colour.

| Prompt type | Attach |
|---|---|
| Natural cube | `01 Natural cube.png` |
| Grass block | `02 Grass block.png` |
| Ore | `03 Ores.png` (the coal stays very dark, so ask for a softer socket) |
| Logs | `04 Logs.png` |
| Planks, bricks, built stone | `05 Planks bricks built stone.png` |
| Framed block | `06 Framed blocks.png` |
| Leaves | `07 Leaves.png` |
| Flower, sapling, mushroom | `08 Flowers saplings mushrooms.png` |
| Grass and ferns | `09 Grass and ferns.png` (shown tinted; for the real texture ask for greyscale) |
| Crops | `10 Crops.png` |
| Workstation | `11 Workstations.png` |
| Liquids and light | `12 Liquids and light.png` |
| Glass and colour sets | `13 Glass and colour sets.png` |
| Nether blocks | `14 Nether.png` |
| Two-block-tall plants | `15 Two-tall plants.png` (top half above bottom half; tall grass and fern shown grey, as drawn for tinting) |
| Doors and trapdoors | `16 Doors and trapdoors.png` |
| Crop growth stages | `17 Crop growth stages.png` |
| Ladders, rails, bars, cobweb, scaffolding | `18 Thin and see-through.png` |

### Example block prompts

- **Stone:** master prompt + natural cube + "large sage-grey stone slabs (`#3f4341` → `#a8bab1`) with deep sage crevices, 6–9 slabs."
- **Iron ore:** master prompt + ore + "warm tan-copper iron nuggets (`#784b38` → `#e0c8a8`) with pale highlights."
- **Poppy:** master prompt + flower + "a red poppy (`#d32228`, shadow `#80161b`) with a dark centre, a green stem (`#2f5b28`) and two leaves."
- **Wheat stage 7:** master prompt + crop + "ripe golden wheat (`#56492c` → `#e3d7aa`), bent heads, full height."
- **Oak planks:** master prompt + planks + "warm walnut-to-honey oak (`#221c16` → `#886f4a`), nudged toward your UI wood `#bc6618` for a warmer look."

## B) Block material sheet

These are the reference's measured ramps (4th, 25th, 50th, 75th and 97th percentile). Use them in prompts.

| Material | Ramp (dark → light) |
|---|---|
| Grass top (tinted on its own) | `#4c642f #617e3b #779c49 #8fba57` |
| Short grass | `#536c32 #698940 #7ca24c #8bb655` |
| Dirt | `#3a2926 #4b352e #5c4434 #77563b #94724b` |
| Sand | `#dbcc89 #e0d39c #e7db9f #eee1ac` |
| Gravel | `#302a24 #3b3529 #443c2f #4b4e49 #a3a493` |
| Snow | `#d6dee6 #dbe3e9 #e2e6ed` |
| Stone | `#3f4341 #616661 #81938a #90a398 #a8bab1` |
| Cobblestone | `#2c2f2c #40473f #6d7767 #859181 #97a89d` |
| Deepslate | `#241e1a #443e46 #54535d #5e626c #6c7b82` |
| Obsidian | `#151319 #1f1b26 #2d2a38 #373343 #5b5668` |
| Netherrack | `#1c1418 #522126 #723b40 #964c52 #c08085` |
| Crimson nylium | `#1b030c #2b050e #3e0710 #5f0b15 #8e2934` |
| Oak log bark | `#2a2319 #362c1f #443927 #5b4b33 #7e6643` |
| Oak planks | `#221c16 #3e3228 #5d4d34 #6f5a3f #886f4a` |
| Spruce planks | `#181515 #2b1f1d #452f27 #53372f #69463a` |
| Birch log | `#726960 #a5a29a #bdbdb8 #d1d1ce #dfdfdf` |
| Oak leaves | `#27380f #344b14 #49691c #669328 #74a72e` |
| Spruce leaves | `#12210f #1a2d16 #26421f #2f5328 #3b6434` |
| Poppy | petal `#571114 #80161b #d32228`, stem `#2f5b28` |
| Dandelion | stem `#1e3f1b #315d24 #438632`, head `#e9d636` |
| Ripe wheat | `#56492c #866e39 #a88d51 #c2ae6f #e3d7aa` |
| Pumpkin | `#8e4220 #b04f1b #d2661a #e5861b #edac3f` |
| Red mushroom block | `#8c0804 #970f08 #a3130a`, spots `#f4dfdc` |
| Stone bricks | `#242928 #5d6b64 #788981 #86978f #a0ada5` |
| Bricks | `#37281b #6b432c #a0593d #b86e43 #d0ad8b` |
| Sandstone | `#5d4b3a #bca76b #e4d6a2 #ece0af #fbf2cd` |
| White wool | `#c7c2bb #dcd9d5 #e0deda #e5e3e0` |
| Iron block | `#222322 #595951 #6d7066 #75786e #cbd1d0` (or your UI iron `#8e8884 / #c9c3be / #ece6e2`) |
| Gold block | `#3f341b #806033 #d7a957 #fee15a #feffe9` |
| Copper block | `#351c0b #8f4e20 #934e21 #d97f42` (UI copper `#f07e37` for a brighter look) |
| Water (before tint) | `#3a8ad5 #3d96de #4aa3e9 #62b9f8` |
| Lava | `#190d0d #330e0b #6c0e0c #c51310 #f18f00` |
| Glowstone | `#844a1e #a45b27 #c68d4a #edc444 #f9f9b9` |
| Ice | `#9dcfef #9fd0f2 #a2d3f3` (translucent) |

## C) Full-pack checklist: every texture family

Use this to make sure the prompt set covers everything. The counts are approximate, for vanilla 1.21.

| Family | Textures | Notes |
|---|---|---|
| Ground | ~25 | grass block (top, side, overlay, snow side), dirt, coarse dirt, rooted dirt, podzol, mycelium, farmland (dry and moist), dirt path, mud, clay, sand, red sand, gravel, snow, moss, pale moss |
| Stone and rock | ~45 | stone, cobble, mossy cobble, the 3 stone types and their polished forms, deepslate (+ cobbled, polished, tiles, bricks), tuff (+ variants), calcite, dripstone (+ pointed dripstone stages), basalt, blackstone, obsidian, crying obsidian, bedrock |
| Ores | ~20 | 8 stone ores, 8 deepslate ores, 2 nether ores, ancient debris (side and top), 3 raw ore blocks |
| Wood × 12 types | ~150 | For each of oak, spruce, birch, jungle, acacia, dark oak, mangrove, cherry, pale oak, bamboo, crimson and warped: log side and top, stripped log side and top, planks, door top and bottom, trapdoor, sapling or fungus, leaves, plus signs (entity). Also mangrove roots. |
| Plants (cross) | ~60 | 15 flowers, 4 tall flowers (2 halves each), short and tall grass, fern, large fern, dead bush, saplings, mushrooms, cobweb, sweet berry bush (4 stages), roots, sprouts, hanging roots, spore blossom, pitcher plant, torchflower |
| Climbing and water plants | ~25 | vine, glow lichen, twisting and weeping vines, cave vines (with and without berries), kelp, seagrass, sea pickle, lily pad, 5 corals plus coral fans and blocks, alive and dead |
| Crops and stems | ~35 | wheat 0–7, carrots, potatoes and beetroots 0–3, nether wart 0–2, cocoa 0–2, melon and pumpkin stems (plus attached), torchflower and pitcher crops, bamboo, sugar cane, cactus (top, side, bottom) |
| Farm blocks | ~10 | pumpkin (top, side), carved pumpkin, jack o'lantern, melon (top, side), hay (top, side), composter (side, top, bottom, compost, ready) |
| Built stone | ~60 | stone bricks (+ mossy, cracked, chiseled), bricks, mud bricks, sandstone and red sandstone (+ cut, chiseled, smooth), quartz (+ pillar, chiseled, bricks), prismarine ×3, nether bricks ×3, purpur, end stone bricks, smooth stone |
| Colour sets × 16 | ~130 | wool, carpet (uses wool), concrete, concrete powder, terracotta, glazed terracotta, stained glass + panes, candles (lit and unlit), plus beds, banners and shulker boxes (entity) |
| Glass and ice | ~10 | glass, glass pane top, tinted glass, ice, packed ice, blue ice, frosted ice 0–3 |
| Metals and gems | ~35 | iron, gold, diamond, emerald, lapis, redstone, coal, netherite, amethyst (+ bud and cluster stages), copper ×4 oxidation stages × (block, cut, chiseled, grate, bulb, door, trapdoor) |
| Light and liquids | ~25 | water and lava (still and flow), fire ×2, soul fire ×2, torch, soul and redstone torch, lantern ×2, campfire, glowstone, sea lantern, shroomlight, froglights ×3, redstone lamp (on and off), magma, end rod |
| Workstations | ~45 | crafting table, furnace, smoker, blast furnace (on and off), cartography, fletching, smithing, loom, stonecutter, grindstone, anvil (+ 2 chipped), cauldron, barrel, beehive, bee nest, lectern, bookshelf, chiseled bookshelf, enchanting table, brewing stand, jukebox, note block, crafter |
| Redstone | ~40 | redstone dust, repeater, comparator (on and off), piston (side, top, sticky, inner), observer, dispenser, dropper, hopper, lever, buttons and plates (reuse planks and stone), rails ×4 (on and off), target, daylight detector, TNT, sculk sensor, tripwire |
| Nether and End | ~40 | netherrack, both nyliums, wart blocks, stems, soul sand and soil, blackstone set, basalt, glowstone, end stone, purpur, chorus plant and flower, end portal frame, dragon egg |
| Misc and effects | ~20 | destroy stages 0–9, ladder, scaffolding, chain, iron bars, bell (entity), flower pot, decorated pot (entity), spawner, trial spawner, vault, sculk set, beacon |

**Which entries are just recolours:** the 16-colour sets and the copper oxidation stages. Each is one design repeated in different colours. Generate the base, then recolour it to the colour ramp instead of prompting each one separately, to keep them consistent.
