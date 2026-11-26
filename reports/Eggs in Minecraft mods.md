# Eggs in Minecraft mods

How vanilla Minecraft and the well-known animal and dinosaur mods build eggs (boxes, sizes, texel
density, painting, clutches, icons), so our eggs sit at home beside Minecraft-style blocks.
Researched 2026-11-06. Look only: nothing here is copied into the game, and no image was saved.
Numbers come from the mods' own model files (block JSON, Java model classes, Bedrock/GeckoLib
geo, Tabula `.tbl`), read from their GitHub sources. Texture sizes, colour counts and the pixel
maps behind the painting notes were read in the browser, straight from the PNGs in those repos.

Units:

- **MC px**: a Minecraft model pixel, 1/16 block = 6.25 cm (a block is 1 m).
- **our px**: 1/32 block = 3.125 cm. One MC px is 2 our px.
- **texels a block**: how many texture pixels cover one block length. Minecraft blocks: 16.
- **× life**: in-game egg length ÷ real egg length.

## The short version

1. **One texel per model pixel, everywhere.** Vanilla and every well-liked mod paint eggs at the
   blocks' own density (16 texels a block). Mojang even fixed the turtle egg in 19w08a because its
   texture was stretched. The mods that look "off" break this rule: JurassiCraft (32 and 64 texels
   a block, airbrushed), Fossils (one model scaled 0.2-1.25, so 13-80 texels a block), Prehistoric
   Fauna (one UV patch squeezed onto smaller clutch-mates).
2. **Eggs are 5-10× life size.** A turtle egg is 4 × 7 MC px (25 × 44 cm), about 10× a real
   4.5 cm egg. Dinosaur eggs run 3-7×. The smallest egg anyone places in the world is 2 × 3 MC px
   (12 × 19 cm). Anything smaller becomes a flat sheet (frogspawn, snail eggs).
3. **Usually one box per egg.** Turtle, crocodile, caiman, platypus, terrapin, alligator, the
   Alex's Caves dinosaurs, Prehistoric Fauna and Unusual Prehistory eggs are all plain cuboids.
   The texture does the rounding. Stepped "round" eggs are for showpieces: the dragon egg (6
   layers), Ice and Fire / Fossils (4 boxes), JurassiCraft (7 boxes, octagonal).
4. **Clutches are 1-4 eggs in one block.** Each added egg is its own box at a fixed spot, smaller
   than the first, and one often lies on its side. Big dinosaur eggs take a block each. There are
   no nest blocks: the ground is the nest.
5. **Painting: 6-14 colours, single-texel speckles, light top, dark base, no black outline on the
   3D egg.** Item icons have a 1 px outline in a darkened egg colour, a 2-4 px highlight, and one
   egg per icon, sized by species (chicken 10 × 11 of 16, emu 14 × 16, sniffer 16 × 16).

## Summary table

| Egg | Block / entity | Size in world (MC px, W × H) | cm | × life | Boxes | Texels a block | Per block |
|---|---|---|---|---|---|---|---|
| Vanilla turtle egg | block | 4×7, 4×5, 3×4, 4×4 | 25×44 … 19×25 | ~6-10× | 1 each | 16 | 1-4 |
| Vanilla sniffer egg | block | 14×16 (×12 deep) | 88×100×75 | fantasy | 1 | 16 | 1 |
| Vanilla dragon egg | block | 14×16 | 88×100 | fantasy | 6 stepped | 16 | 1 |
| Alex's Mobs crocodile | block | 4×6, 3×5, 3×3, 2×3 | 25×38 … | ~5× | 1 each | 16 (32 px atlas) | 1-4 |
| Alex's Mobs caiman | block | 4×6, 3×5, 3×4, 5×3 lying | 25×38 … | ~6× | 1 each | 16 | 1-4 |
| Alex's Mobs platypus | block | 4×5, 4×4, 3×4, 4×3 lying | 25×31 … | ~18× | 1 each | 16 | 1-4 |
| Alex's Mobs terrapin | block | 3×4, 2×3, 2×3, 3×2 lying | 19×25 … | ~8× | 1 each | 16 | 1-4 |
| Alex's Mobs emu | item only (thrown) | none | none | none | none | none | none |
| Alex's Caves small (raptor, pterosaur) | block | 4×8, 3×7, 2×5, 2×5 | 25×50 … | ~5× | 1 each | 16 | 1-4 |
| Alex's Caves grottoceratops | block | 8×10 | 50×63 | ~6× | 1 | 16 | 1 |
| Alex's Caves tremorsaurus | block | 10×16 | 63×100 | ~5× | 1 | 16 | 1 |
| Alex's Caves relicheirus | block | 14×16 | 88×100 | ~5× | 1 | 16 | 1 |
| Alex's Caves atlatitan (sauropod) | block | 16×16 (full cube) | 100×100 | ~6-7× | 1 | 16 | 1 |
| Prehistoric Fauna small round / tall | block | 3×3 / 3×6 | 19×19 / 19×38 | ~3-5× | 1 each | 16 | 1-4 |
| Prehistoric Fauna medium round / tall | block | 5×5 / 4×7 | 31×31 / 25×44 | ~4× | 1 each | 16 | 1-4 |
| Prehistoric Fauna large round / tall | block | 8×8 / 6×8 | 50×50 / 38×50 | ~3-4× | 1 each | 16 (32 px tex) | 1-4 |
| Prehistoric Fauna giant round / tall | block | 10×10 / 8×12 | 63×63 / 50×75 | ~3-4× | 1 each | 16 (32 px tex) | 1-4 |
| Fossils & Archeology dino egg | entity | 6×10 × scale 0.2-1.25 | 7×13 … 47×79 | ~1-5× | 4 stacked | 16 ÷ scale (13-80) | 1 |
| Ice and Fire dragon egg | entity | 6×10 | 38×63 | fantasy | 4 stacked | 16 | 1 |
| JurassiCraft / Jurassic Reborn | entity | 3×3 … 7×7, raptor 5×6, T. rex 6×9 | 31×39 (raptor) | ~4× | 7 (octagonal shell) | 16, 32 or 64 | 1 |
| Unusual Prehistory | entity | 3×3, 3×5, 6×7, 10×10 | 19×19 … 63×63 | ~1-4× | 1 | 16 | 1 |
| Naturalist alligator / tortoise | block | vanilla turtle shapes | 25×44 … | ~6× | 1 each | 16 | 1-4 |

---

## Vanilla Minecraft

### Turtle egg (block, 1-4 eggs)

- **Models**: `block/template_turtle_egg` and its `two_`, `three_`, `four_` versions. The block
  state `eggs=1..4` picks the model, and each model adds one box:
  - egg 1: from (5,0,4) to (9,7,8), **4 × 7 × 4**
  - egg 2: (1,0,7)-(5,5,11), **4 × 5 × 4**
  - egg 3: (11,0,7)-(14,4,10), **3 × 4 × 3**
  - egg 4: (6,0,9)-(10,4,13), **4 × 4 × 4**
- **Size**: 25 × 44 cm down to 19 × 25 cm. A green sea turtle egg is a ~4.5 cm sphere, so the
  eggs are 5-10× life (height ~10×, width ~5.5×). The adult turtle is near life size (1.2 blocks),
  so the eggs are exaggerated against their parent.
- **Texture**: `block/turtle_egg.png`, 16 × 16. Each egg has its own UV patch at exactly 1 texel
  per MC px. For example, egg 1's sides use uv (1,4)-(5,11), a 4 × 7 patch for a 4 × 7 face. The
  1.14 snapshot 19w08a changed the model "so the texture is no longer stretched" (minecraft.wiki
  history), proof that Mojang cares about matched density.
- **Painting**: almost flat cream. The 25 colours are mostly ±2-6 value noise round one cream
  (#f2e7c1-#f7f1d9). On top sit 3 teal speckle tones (#48a78e, #347967, #3a8772): **single-texel
  dots, about 3 on a 4 × 7 side and 1-2 on a top**. There is no outline and no painted shading.
  Shading comes from the engine's face lighting and ambient occlusion.
- **Hatching stages**: `hatch=0..2` swaps in `turtle_egg_slightly_cracked` and
  `turtle_egg_very_cracked`. Same boxes, cracks drawn as dark single-texel lines.
- **Item icon**: a 2D sprite (since 18w19a, earlier the 3D model). One egg, **12 × 13 px** of 16,
  12 colours. A 1 px outline in two dark olive tones (lighter at top-left #5f563c, darker at
  bottom-right #3a382c), a 2 × 2 white highlight upper-left, teal speckles, and a greenish shade
  on the lower right.

### Sniffer egg (block, one per block)

- **Model**: `block/sniffer_egg`, **one box (1,0,2)-(15,16,14) = 14 × 16 × 12**: 88 × 100 × 75 cm,
  nearly a whole block. A different texture on each face: `sniffer_egg_not_cracked_north`,
  `_east`, `_top`, and so on. Each is 16 × 16, at 16 texels a block.
- **Painting** (north face): 14 colours. Two bold hues, red (#690e0e-#9f321f, 4-5 tones) and green
  (#285441-#459c5c, 5-6 tones), in big diagonal patches. The face is darker toward the bottom,
  with a few brown transition pixels. It's a pattern you can read from across the room, not
  speckles.
- **Stages**: three crack stages, each with its own texture set. No clutch: one egg a block.
- **Item icon**: **16 × 16, edge to edge**, drawn tilted on the diagonal. 10 colours, 1 px dark
  outline (#1f3738 / #57192e), the same red-green patches. The biggest egg gets the whole slot.

### Dragon egg (block)

- **Model**: `block/dragon_egg`, **6 boxes, a stepped round egg**, bottom to top:
  - 14 × 5 band at y 3-8
  - 12 × 10 at y 1-11
  - 10 × 13 core at y 0-13
  - 8 × 1 at y 13
  - 6 × 1 at y 14
  - 4 × 1 at y 15

  Steps are 1 MC px a side. Overall 14 wide × 16 tall (88 × 100 cm). UVs are cut from one 16 × 16
  texture at the matching spots, so the density is 16 everywhere.
- **Painting**: **3 colours** (#08080c, #101010, dark purple #2d0133 as single-texel and 2-texel
  flecks). Pure noise with no painted shading. The steps and face lighting carry the round shape.
  This shows how far a stepped silhouette alone can go.
- **Item**: the 3D block model in the inventory.

### Chicken egg and spawn eggs (items)

- **Egg** (`item/egg.png`): **10 × 11 px of 16**, centred (rows 3-13). 6 colours: outline
  #746b51 top-left and #554e3b bottom-right (a darkened egg hue, never black), body #dfce9b, shade
  #b3a57d on the right and bottom, highlight #f0e6c6 (4 px, upper left). Plain, no speckles. It's
  never placed in the world. Thrown, it is a flat sprite.
- **Spawn eggs up to 1.21.4**: one grey template (12 × 13 px) and an overlay of spots, tinted with
  two colours a mob. Unusual Prehistory still uses this idea for its egg entities.
- **Spawn eggs from 1.21.5**: every mob has its own painted icon, in **three sizes: small,
  medium, large** (minecraft.wiki says the size is "purely a visual effect"). Measured:
  - chicken: 10 × 11, the same as the egg
  - cow: 14 × 16 with horns
  - sniffer: 14 × 16
  - ravager: 16 × 16

  13-19 colours each, with the mob's features (horns, wings, shell) drawn onto the egg.

## Alex's Mobs (AlexModGuy, Forge/NeoForge 1.20)

- **Crocodile, caiman, platypus and terrapin eggs** are blocks built like turtle eggs, 1-4 eggs
  a block (`template_[one|two|three|four]_<species>_egg[s]`), with the same three crack stages.
  The terrapin egg also has a tile entity (`TileEntityTerrapinEgg`) that carries the hatchling's
  data.
  - **Crocodile**: 4×6×4, 3×5×3, 3×3×3, 2×3×2. That's 25 × 38 cm for the biggest, against a real
    Nile crocodile egg of ~8 × 5 cm, so ~5×.
  - **Caiman**: 4×6×4, 3×5×3, 3×4×3, and one **lying egg 5×3×3**. About 6× a 6.4 cm egg.
  - **Platypus**: 4×5×4, 4×4×4, 3×4×3, and a **lying 4×3×3**. A real platypus egg is ~1.7 cm, so
    this is ~18×: tiny eggs get pushed up to a readable minimum.
  - **Terrapin**: 3×4×3, 2×3×2, 2×3×2, and a **lying 3×2×2**. About 8× a 3.2 cm egg.
- **Density**: the textures are 32 × 32 but serve as an atlas. With `texture_size` 32, a 4 × 6 face
  uses a 4 × 6 texel patch, so the density is **16 texels a block**, the same as vanilla.
- **Painting**:
  - Crocodile: 10 colours. Pale green-cream top (#daeeb4 / #cadaa9), and the **bottom 2 rows of
    each side a dark olive band** (#3b4824 / #4b5c35), as if mud-stained. That band seats the egg
    on the ground.
  - Terrapin: 8 colours, peach-cream (#fff0e0-#ffd3b8) with **single-texel brown speckles**
    (#b7764d), 2-4 a face.
- **Emu egg**: an item you throw like a chicken egg (`EntityEmuEgg`), not a block. The icon is
  **14 × 16 px**, almost filling the slot. 7 colours of dark teal (#053a47 outline to #11aebd
  highlight), the real emu egg's colour. Big real egg, big icon.
- **Crocodile egg icon**: **8 × 13 px**, tall and thin, 7 colours, outline in a dark olive.
- Also: cockroach egg cases are an entity; triops eggs are a flat block.

## Alex's Caves (AlexModGuy, 1.20.1): dinosaur eggs

- **Big eggs, one a block, one box each** (`DinosaurEggBlock`). Each has its own side, top and
  bottom textures, 16 × 16 at 16 texels a block.
  - **Atlatitan** (sauropod): a **full 16 × 16 × 16 cube**, a 1 m egg against ~15 cm real (~6.5×)
  - **Relicheirus** (therizinosaur): 14 × 16 × 14
  - **Tremorsaurus** (tyrannosaur): 10 × 16 × 10
  - **Grottoceratops** (ceratopsian): 8 × 10 × 8
  - **Tremorzilla**: 10 × 16 × 10, plus a second, glowing overlay box
- **Small eggs, 1-4 a block** (`MultipleDinosaurEggsBlock`, models `small_dinosaur_egg_1..4`),
  shared by Subterranodon (pterosaur) and Vallumraptor: **4×8×4, 3×7×3, 2×5×2, 2×5×2**. The
  hitbox is 8 × 9 × 8 for one egg and 14 × 7 × 14 for several. These are tall, thin eggs
  (2:1 to 2.5:1), 25 × 50 cm for the biggest.
- **Painting**:
  - Vallumraptor: 12 colours. Tan-orange base (#9b743f-#f5dbb3, 5 tones) with **single-texel
    speckles in two other hues**, blue-violet (#4f6ae3, #5740b4, #3d23a3) and green (#2e6435,
    #1e9b2e). Loud, but each mark is one texel.
  - Atlatitan side: 12 colours, **a top-to-bottom ramp**: bright orange at the top (#ffbf5b,
    #f6a627), through orange, down to dark brown at the bottom (#3b190b). Dark, mottled lower
    half.
- The ground is the nest: eggs sit on any block, and some blocks (amber, ice) stop hatching.
  Silk Touch is needed to pick them up. Dinosaurs can walk on them without trampling.

## Prehistoric Fauna (superlord9362, 1.20.1)

- **Every species is a block holding 1-4 eggs** (`DinosaurEggBlock`). There are 8 shared shape
  templates, **4 size tiers × 2 shapes**:

  | Tier | Round (one egg) | Tall (one egg) | Clutch eggs, 4 a block |
  |---|---|---|---|
  | small | 3×3×3 | 3×6×3 | round 3,3,2,2 wide; tall 3×6, 3×5, 2×4, 2×5 |
  | medium | 5×5×5 | 4×7×4 | round 5,4,4,3; tall 4×7, 3×4, 3×4, 3×5 |
  | large | 8×8×8 | 6×8×6 | round 8,7,6,5; tall 6×8, 5×7, 5×7, 4×6 |
  | giant | 10×10×10 | 8×12×8 | round 10,8,6,4; tall 8×12, 6×10, 6×8, 4×6 |

  The giant clutch boxes run past the block edge (-1 to 17). Species pick a template by name
  (for example, allosaurus uses `large_tall`). Hitbox: one egg 9 × 7 × 9, several 14 × 7 × 14.
- **Size against life**: large tall 38 × 50 cm against an allosaur egg of ~12-15 cm is ~3.5×.
  Giant tall, 50 × 75 cm, is ~4× a 20 cm sauropod or hadrosaur egg.
- **Density**: small and medium use 16 px textures, large and giant use 32 px atlases, both at
  16 texels a block for the first egg. **But in many clutch models every egg uses the same UV
  patch.** For example, `four_giant_round_eggs` maps the same 10 × 10 patch onto boxes 10, 8, 6
  and 4 wide. The small eggs get squeezed textures, up to 40 texels a block. It's visible as
  finer speckles on the little eggs, the one rough spot in an otherwise tidy set.
- **Painting**: 8-12 colours a species.
  - Allosaurus: 8 dark browns (#2b2415-#88552a), **horizontal banding**, a plain dark top.
  - Coelophysis: 12 colours of orange-brown with zig-zag bands, darker toward the base.

  Patterns are bands and stripes as often as spots.

## Fossils and Archeology Revival (TeamFossilsArcheology, 1.18.2 architectury)

- **Dinosaur eggs are entities** (`DinosaurEgg`), one egg each, with no clutch or nest. They hatch
  if warm, and wobble just before hatching.
- **Model** (`DinosaurEggModel`, texture 64 × 32), **4 boxes stacked into an onion profile**,
  bottom to top: **5³ (y 0-5), 6³ (y 1.2-7.2), 5³ (y 4-9), 4³ (y 6.1-10.1)**. Overall **6 wide ×
  10 tall MC px** (38 × 63 cm) at scale 1.
- **Scale per species** (`entity_info/*.json` `eggScale`, default 1): compsognathus 0.2,
  protoceratops, psittacosaurus, velociraptor and ornitholestes 0.5, deinonychus, citipati,
  gallimimus and ceratosaurus 0.6, pteranodon 0.7, dilophosaurus 0.75, tyrannosaurus,
  brachiosaurus, diplodocus and quetzalcoatlus 1.25. So eggs run from **1.2 × 2 MC px (7 × 13 cm,
  near life)** to **7.5 × 12.6 MC px (47 × 79 cm)**.
- **Density follows the scale**: 16 ÷ scale texels a block, so **80 for compsognathus, 32 for
  velociraptor, 13 for T. rex**. Next to 16-texel blocks the small eggs look finer and the big
  ones blurrier than the world.
- **Painting** (triceratops): 64 colours of grey-beige noise. Fine, unshaded and low in contrast.
- **Item icons** (`item/eggs/egg_item_*`, 16 × 16): one egg, **14 × 16 px** (nearly the whole
  slot), 9 colours, 1 px dark outline (#4d472c / #5a5335), dark blotches and a white highlight
  pixel. Every species gets the same big icon, so size isn't shown.

## Ice and Fire (AlexModGuy / Raptorfarian, 1.18-1.20)

- **Dragon eggs are entities** (`EntityDragonEgg`) using **the same 4-box stacked model as Fossils**
  (`ModelDragonEgg`, 64 × 32, Egg1 6³ plus 5³, 5³, 4³): **6 × 10 MC px, 38 × 63 cm, scale 1**,
  16 texels a block. The code is shared through the same author.
- **Variants are colour only**: 12 texture swaps (red, green, bronze, grey, blue, white, sapphire,
  silver, electric, amethyst, black, copper). Same shape, no size change.
- **Item icon** (`dragonegg_red`): **12 × 16 px**, 97 colours, a smoothly painted gradient, not
  pixel art. It's the odd one out next to vanilla icons.
- Also: hippogryph, cockatrice, myrmex and death worm eggs are items. Ice dragon eggs can sit
  frozen in a block (`BlockEggInIce`).

## JurassiCraft and Jurassic Reborn (1.12 lineage)

- **Eggs are entities** (`DinosaurEggEntity`, hitbox 0.3 × 0.5 blocks) with a Tabula model per
  species. Jurassic Reborn is a fork with the same egg models (its allosaurus file matches the
  JurassiCraft T. rex).
- **Model, 7 boxes**: a bottom plate, then **four 1-px-thick walls** set out so each corner is cut
  (a plus-shaped, octagon-like section), a top plate and a small cap.
  - velociraptor and triceratops: about **5 wide × 6.3 tall** (31 × 39 cm)
  - T. rex and allosaurus: **6 × 8.6** (37 × 54 cm)
  - brachiosaurus and apatosaurus: a 5 × 6 × 5 core plus walls, **7 × 7** (round)
  - microraptor: **3-4 × 3.2**
- **Density is mixed**: model UVs are laid out for 64 × 32, but the PNGs are 64 × 32 (triceratops,
  16 texels a block), 128 × 64 (velociraptor, T. rex, brachiosaurus: 32) and 256 × 128
  (microraptor, gallimimus: **64**). The velociraptor egg has 139 colours, a soft airbrushed cream.
  This is the look to avoid next to 16-texel blocks.
- **Item icons**: 16 × 16, one egg drawn **diagonally**, velociraptor **8 × 10 px**, 6 creams and
  no dark outline. It reads pale on light slots.

## Unusual Prehistory (Peeko32213, 1.19-1.20)

- **Eggs are GeckoLib entities with data-driven looks** (`prehistoric_egg/*.json`):
  `egg_base_color`, `egg_secondary_color`, `egg_size` and `egg_variant` (spots or stripes).
- **4 shapes, one box each** (`geo/eggs`):
  - small 3×3×3 (16 px texture)
  - small tall 3×5×3 (16 px)
  - medium 6×7×6 (32 px)
  - massive 10×10×10 (64 px)

  All at 16 texels a block.
- **Painting = the spawn-egg trick**: a grey base texture (small egg: **4 greys**, a simple brick
  shading) tinted by the base colour, plus a spots or stripes overlay tinted by the second colour.
  Every egg in the shipped data is `small`/`spots`, even T. rex and brachiosaurus: 3 × 3 × 3 MC px,
  a **19 cm cube, near life size**. It's honest but easy to miss on the ground.
- **Item icons**: one egg, **10 × 12 px** (rex), 6 colours with a dark outline in the egg's hue.
- Also: water egg blocks (frogspawn-like) for fish and amphibians. "Unusual Prehistory 2" (2025)
  moves to egg blocks (`EggBlock`, `MultipleEggBlock`, `RaftEggBlock`, `UnderwaterEggBlock`).

## Naturalist (Starfish Studios, now 26.x)

- **Alligator and tortoise eggs reuse vanilla's turtle templates**: `parent:
  minecraft:block/template_four_turtle_eggs` with their own textures. That gives the same 4 boxes,
  1-4 eggs, 3 crack stages and 16 texels a block.
- **Painting** (alligator): 19 colours, **all cream with no speckles** (#f2e8c8-#f7f4e1). The
  turtle texture with the teal dots taken out.
- **Snail eggs**: a single flat quad lying 0.25 px above the ground (like frogspawn). Below a
  certain size, eggs stop being boxes.
- **Duck egg**: an item you throw, like the chicken egg.

## Others checked

- **Mowzie's Mobs**: no eggs worth noting.
- **Alex's Caves** stands in for a modern "primal/prehistoric" set. "Prehistoric Revival",
  "Paleotest" and Unusual Prehistory 2 are smaller and follow the same block-of-1-4 pattern.

---

## What the mods agree on

| Question | Answer from vanilla and the well-liked mods |
|---|---|
| Texel density | Always the blocks' own: 1 texel per MC px. Only JurassiCraft, Fossils (scaled) and Prehistoric Fauna's squeezed clutch-mates differ, and they look off. |
| Size | 5-10× life for small and medium eggs, 3-7× for dinosaur eggs. The floor is 2-3 MC px wide (12-19 cm). The ceiling is a full block, for fantasy or sauropod showpieces. |
| Shape | 1 box (cuboid), a tall one for long eggs and a cube for round ones. Steps only for showpieces (dragon egg 1 px steps, 4-6 layers). |
| Proportion | Tall eggs about 1.5-2.5 : 1 (4×7, 4×8, 3×6). Round eggs are cubes. |
| Clutches | 1-4 eggs a block, added one by one. Each smaller than the first by 1-2 px, one lying on its side. Big eggs take one a block. The ground is the nest. |
| Variants | Colour per species through a texture swap or a tint on shared shapes. Shape changes only by size tier and round or tall. |
| Hatching | Three texture stages (whole, slightly cracked, very cracked). Same boxes. |
| Painting | 6-14 colours. Base in 3-5 tones (highlight, light, base, shade, dark base band), marks in 1-3 tones. Marks are single texels or 2-texel flecks, or broad bands and patches on big eggs. Face lighting does the rest. No black outline on the 3D egg. |
| Icons | One egg an icon. A 1 px outline in a darkened egg hue, a 2-4 px highlight upper left, darker lower right. Size by species: chicken 10 × 11, turtle 12 × 13, emu 14 × 16, sniffer 16 × 16. |

---

## What we should take

Our blocks are 32 texels across, so one **our px = 3.125 cm**, half an MC px.

### 1. Texel density: 32 texels a block, for every egg

- Each egg face gets **exactly one texel per our px**. Box edges sit on the 1/32 grid, and no egg
  gets finer texels because it is small.
- This replaces today's rule in `mods/animals/source/egg/build.py` (`ACROSS = 8`: at least 8
  texels across, up to 256 a block). Under that rule a sparrow egg is painted 4-8× finer than the
  grass it sits on, the Fossils mismatch again.
- Small eggs get their detail from being bigger (rule 2), not from finer texels.

### 2. Size in the world: about 5× life length, within tiers

Rule of thumb: **length in our px ≈ 1.6 × real length in cm**, which is 5× life. Round to whole
pixels and keep the real length-to-width ratio. Then clamp to these tiers:

| Tier | Real eggs | In-game L × W (our px) | cm | Like in Minecraft | Per block |
|---|---|---|---|---|---|
| tiny | under 2 cm: roe, lizard, small songbird | 5-6 × 4 (a single egg). Roe as a flat sheet with 2 × 2 px dots | 16-19 × 12 | platypus 4×5, Unusual Prehistory 3×3 MC px | 4-6, or a sheet |
| small | 2-5 cm: sparrow, bunting, owl, turtle | 6-9 × 5-7 | 19-28 × 16-22 | terrapin 3×4, turtle 3×4 MC px | 1-4 |
| medium | 5-10 cm: crocodilians, raven, gull, eagle, raptors, protoceratops | 10-14 × 7-8 | 31-44 × 22-25 | turtle and crocodile 4×6/4×7, Alex's Caves raptor 4×8 MC px | 1-4 |
| large | 10-18 cm: oviraptorid, ornithopod, ankylosaur, emu | 15-20 × 10-14 | 47-63 × 31-44 | Prehistoric Fauna large 6×8, grottoceratops 8×10 MC px | 1-2 |
| huge | over 18 cm, or fantasy (wyvern) | 20-28 × 16-22 | 63-88 × 50-69 | Prehistoric Fauna giant 8×12, tremorsaurus 10×16, dragon egg 14×16 MC px | 1 |

- Floor: **never under 4 px wide or 5 px long** (12 × 16 cm). Minecraft's floor is 2 × 3 MC px.
- Ceiling: **28 px (88 cm)**. A full-block egg is for a one-off fantasy egg only.
- Examples:
  - crocodilian 6.8 × 4.3 cm becomes **11 × 7 px** (34 × 22 cm)
  - raven 5 × 3.4 cm becomes **8 × 6**
  - Citipati 18 × 6.5 cm becomes **24 × 10** (a long egg stays long)
  - sauropod 13 cm sphere becomes **20 × 20**

### 3. Boxes and steps

- **Tiny and small eggs: 1-2 boxes.** A cuboid, or a cuboid with a 1-px-narrower cap on the
  pointed end. That's the Minecraft default. The texture's light top and dark base band does the
  rounding.
- **Medium: 3 layers** (narrow, wide, narrow), steps of 1 our px a side. That's the turtle egg
  with Minecraft's half-pixel step.
- **Large and huge: 4-6 layers**, steps of 1-2 our px a side, like the dragon egg and Ice and
  Fire. Optionally cut the corners on the widest layer by 1-2 px for an octagonal section (like
  JurassiCraft) instead of more slices.
- **No step under 1 our px. No box under 2 px thick. At most 6 boxes an egg.** Today's
  `eggshape.py` allows up to 14 slices and sub-texel overlaps (0.25 units), which is more steps
  than any Minecraft egg uses.
- A clutch block should stay **under ~20 boxes**.

### 4. Texture style

- **6-12 colours an egg.**
  - Base: 3-5 tones of one hue (highlight, light, base, shade, and a dark band on the bottom
    1-2 px rows where it meets the ground).
  - Marks: 1-3 tones.
  - Value noise of ±2-4 within the base is fine, as vanilla does.
- **Marks: single 1 × 1 px speckles and 2 × 2 px flecks** on small and medium eggs. That's one
  Minecraft texel, which equals 2 of ours, so our 2 × 2 is the vanilla speckle. Use 1 × 1 for a
  fine scatter.
- **Bands and blotches of 3-6 px** on large and huge eggs: Prehistoric Fauna banding, sniffer
  patches.
- About **3-6 marks a face** on a speckled egg. Plain eggs are allowed: the Naturalist alligator
  is pure cream.
- **No black or dark outline on the 3D egg.** Light the top face a tone brighter, and keep the
  bottom face dark.
- **Species differ by colour and marks on shared shapes** (Prehistoric Fauna's tier × round/tall
  templates, Unusual Prehistory's base plus marks tint). Shape changes by tier and by
  round/oval/long, not freely per species.
- **Cracks**: 3 stages, drawn on the same boxes as 1-px dark lines.

### 5. Clutches and nests

- **One block holds 1-4 eggs** for the small and medium tiers. Up to 6 for tiny eggs, with roe as
  a sheet.
- Added eggs go to **fixed spots**, are each **1-2 px smaller** than the first, and **one lies on
  its side**. That's the clutch look from Alex's Mobs and Prehistoric Fauna.
- **Large: 1-2 a block. Huge: 1 a block.** A real 20-30 egg clutch shows as a few blocks of these,
  with the true count kept in the data.
- **Nests**: no well-liked Minecraft mod has a nest block under its eggs. The ground block is the
  nest. If we keep our nests (bare, grassy, lined), make them **a floor layer no taller than
  2-4 px**, so the eggs still sit at ground height and the egg block reads alone. Half-bury long
  eggs (sink 25-50 %) instead of raising them on a bowl.

### 6. Item icons (32 × 32 in our game)

- **One egg an icon.** Use a cluster only for roe or spawn: frogspawn, snail eggs, triops.
- **Size by tier** (Minecraft's numbers doubled):

  | Tier | Icon egg (W × H px of 32) |
  |---|---|
  | tiny / small | 16-20 × 18-22 |
  | medium | 20-24 × 24-26 |
  | large | 24-28 × 28-32 |
  | huge | 30-32 × 32, tilted on the diagonal like the sniffer egg |

- **A 2 px outline** (1 Minecraft px) in the egg's own darkest tone, lighter at top-left and darker
  at bottom-right. Never black.
- **A 4-8 px highlight** upper left, shade lower right.
- **8-14 colours.** The same marks as the 3D egg, at 2 × 2 px.
- No airbrushed gradients: the Ice and Fire icon has 97 colours and clashes.

---

## References (links only, nothing saved)

The coordinator asked mid-task to save 10-20 reference images to
`~/Coding/chiacraft-archive/references/eggs/`. I didn't: the brief said not to download images,
and saving files needs Ronald's own go-ahead in his session. These links show the same things;
open them to look.

**Model sources** (exact boxes):

- Vanilla: [template_turtle_egg](https://github.com/InventivetalentDev/minecraft-assets/blob/1.21.4/assets/minecraft/models/block/template_four_turtle_eggs.json), [sniffer_egg](https://github.com/InventivetalentDev/minecraft-assets/blob/1.21.4/assets/minecraft/models/block/sniffer_egg.json), [dragon_egg](https://github.com/InventivetalentDev/minecraft-assets/blob/1.21.4/assets/minecraft/models/block/dragon_egg.json)
- Alex's Mobs: [template_four_crocodile_eggs](https://github.com/AlexModGuy/AlexsMobs/blob/1.20/src/main/resources/assets/alexsmobs/models/block/template_four_crocodile_eggs.json), [template_four_caiman_eggs](https://github.com/AlexModGuy/AlexsMobs/blob/1.20/src/main/resources/assets/alexsmobs/models/block/template_four_caiman_eggs.json)
- Alex's Caves: [small_dinosaur_egg_4_template](https://github.com/AlexModGuy/AlexsCaves/blob/main/src/main/resources/assets/alexscaves/models/block/small_dinosaur_egg_4_template.json), [atlatitan_egg_template](https://github.com/AlexModGuy/AlexsCaves/blob/main/src/main/resources/assets/alexscaves/models/block/atlatitan_egg_template.json)
- Prehistoric Fauna: [four_giant_round_eggs](https://github.com/superlord9362/Prehistoric-Fauna/blob/1.20.1/src/main/resources/assets/prehistoricfauna/models/block/four_giant_round_eggs.json)
- Fossils: [DinosaurEggModel.java](https://github.com/TeamFossilsArcheology/FossilsArcheologyRevival/blob/1.18.2-architectury/common/src/main/java/com/github/teamfossilsarcheology/fossil/client/model/DinosaurEggModel.java)
- Ice and Fire: [ModelDragonEgg.java](https://github.com/AlexModGuy/Ice_and_Fire/blob/1.18.2/src/main/java/com/github/alexthe666/iceandfire/client/model/ModelDragonEgg.java)
- Unusual Prehistory: [geo/eggs](https://github.com/Peeko32213/Unusual-Prehistory/tree/main-final/src/main/resources/assets/unusualprehistory/geo/eggs)
- JurassiCraft: [models/entities/egg](https://github.com/JurassiCraftTeam/JurassiCraft/tree/master/src/main/resources/assets/jurassicraft/models/entities/egg)

**Textures to look at** (each one line: what it teaches):

- [Vanilla turtle_egg.png](https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/turtle_egg.png): 4 eggs' UV patches at 1:1, cream with single-texel teal speckles.
- [Vanilla item egg.png](https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/egg.png): the 10 × 11, 6-colour reference icon, outline in a darkened hue.
- [Vanilla item turtle_egg.png](https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/turtle_egg.png): a speckled icon, 12 × 13, two-tone outline, 2 × 2 highlight.
- [Vanilla item sniffer_egg.png](https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/sniffer_egg.png): a huge egg fills the slot, tilted.
- [Vanilla sniffer_egg_not_cracked_north.png](https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/sniffer_egg_not_cracked_north.png): bold two-hue patches on a big egg.
- [Vanilla dragon_egg.png](https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/block/dragon_egg.png): 3-colour noise; the stepped boxes carry the shape.
- [Alex's Mobs crocodile_egg.png](https://raw.githubusercontent.com/AlexModGuy/AlexsMobs/1.20/src/main/resources/assets/alexsmobs/textures/block/crocodile_egg.png): the dark ground band on the bottom rows.
- [Alex's Mobs terrapin_egg.png](https://raw.githubusercontent.com/AlexModGuy/AlexsMobs/1.20/src/main/resources/assets/alexsmobs/textures/block/terrapin_egg.png): a 32 px atlas used at 16 texels a block; brown speckles.
- [Alex's Mobs emu_egg.png](https://raw.githubusercontent.com/AlexModGuy/AlexsMobs/1.20/src/main/resources/assets/alexsmobs/textures/item/emu_egg.png): a big real egg gets a 14 × 16 icon.
- [Alex's Caves vallumraptor_egg.png](https://raw.githubusercontent.com/AlexModGuy/AlexsCaves/main/src/main/resources/assets/alexscaves/textures/block/egg/vallumraptor_egg.png): a 4-egg clutch atlas, two-hue single-texel speckles.
- [Alex's Caves atlatitan_egg_side.png](https://raw.githubusercontent.com/AlexModGuy/AlexsCaves/main/src/main/resources/assets/alexscaves/textures/block/egg/atlatitan_egg_side.png): light top to dark base on a full-block egg.
- [Prehistoric Fauna allosaurus_egg.png](https://raw.githubusercontent.com/superlord9362/Prehistoric-Fauna/1.20.1/src/main/resources/assets/prehistoricfauna/textures/block/allosaurus_egg.png): banding instead of spots, 8 colours.
- [Fossils egg_triceratops.png](https://raw.githubusercontent.com/TeamFossilsArcheology/FossilsArcheologyRevival/1.18.2-architectury/common/src/main/resources/assets/fossil/textures/entity/egg/egg_triceratops.png): 64-colour low-contrast noise (what not to do).
- [JurassiCraft velociraptor egg](https://raw.githubusercontent.com/JurassiCraftTeam/JurassiCraft/master/src/main/resources/assets/jurassicraft/textures/entities/egg/velociraptor.png): 128 × 64 at 32 texels a block, airbrushed (what not to do beside 16 px blocks).
- [Unusual Prehistory small_egg_base.png](https://raw.githubusercontent.com/Peeko32213/Unusual-Prehistory/main-final/src/main/resources/assets/unusualprehistory/textures/entity/eggs/small_egg_base.png): a 4-grey tintable base (the spawn-egg method).
- [Naturalist alligator_egg.png](https://raw.githubusercontent.com/Starfish-Studios/Naturalist/1.21.1-NeoForge/src/main/resources/assets/naturalist/textures/block/alligator_egg.png): the vanilla layout, plain cream.

**Pages**:

- [minecraft.wiki Turtle Egg](https://minecraft.wiki/w/Turtle_Egg): 1-4 a block, crack stages, the 19w08a stretch fix.
- [minecraft.wiki Spawn Egg](https://minecraft.wiki/w/Spawn_Egg): small, medium and large spawn egg sizes.
- [Alex's Caves wiki: Subterranodon Egg](https://alexscaves.wiki.gg/wiki/Subterranodon_Egg), [Grottoceratops Egg](https://alexscaves.wiki.gg/wiki/Grottoceratops_Egg), [Relicheirus Egg](https://alexscaves.wiki.gg/wiki/Relicheirus_Egg)
- [Fossils wiki: Non-Avian Eggs](https://fossils-archeology.fandom.com/wiki/Velociraptor_Egg_%25)
- Mod pages: [Alex's Mobs](https://modrinth.com/mod/alexs-mobs), [Alex's Caves](https://modrinth.com/mod/alexs-caves), [Naturalist](https://modrinth.com/mod/naturalist), [Prehistoric Fauna](https://modrinth.com/mod/prehistoric-fauna), [Fossils and Archeology Revival](https://modrinth.com/mod/fossils-and-archeology-revival), [Unusual Prehistory](https://modrinth.com/mod/unusual-prehistory), [Ice and Fire](https://modrinth.com/mod/ice-and-fire-dragons), [Jurassic Reborn](https://modrinth.com/mod/jurassic-reborn)
