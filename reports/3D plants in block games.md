# 3D plants in block games: how packs, mods and other games build them

Research for chiacraft, which may move from flat crossed plants to more 3D ones. Date: 2026-11-06.
"px" means 1/16 of a block in Minecraft models. Our blocks have 32 texels a block, so one Minecraft
px is two of our texels. Sources are linked inline and listed at the end. Quad counts for vanilla
Minecraft come from the game's model JSON (mirror: InventivetalentDev/minecraft-assets, 1.21.4).
Where a pack page gives no technical detail, this says so instead of guessing. Hard FPS numbers for
3D plant packs are rare: the published ones are listed in section 7, and most of the rest is
reasoning from geometry.

Where we are today (read from the code, not the web):

- `BlockModel::Cross` (`crates/engine/src/client/mesher.rs`) draws 2 planes. Each plane is inset
  0.15 from the corners, has both sides as separate quads, and is lit by one light value. That is
  **4 quads a plant**, each a 32-byte record (8 u32), in the cutout layer. It has no random offset,
  rotation or height.
- Wind lives in our shader pack (`mods/base/shaderpack/world.wgsl`), not the engine. Quads tinted
  grass or foliage, and flat (crossed) quads, bend by `up = (y - quad_bottom) / quad_height`. Quads
  under 0.2 block tall get no weight. The wind is sampled at the block's centre (×0.11), plus a
  `parted()` push from the player's feet. Leaf cubes move a flat ×0.035.
- The prototype resource pack `plants_3d` (OVERVIEW: "3D plants (prototype)") redraws tall grass,
  sugar cane, wheat, carrots and beans as box models through `.ctm` model tiles. That is
  **18-105 quads a block**, against 4.

---

## 1. Vanilla Minecraft: the "3D-ish" plants are planes, not boxes

Mojang builds nearly every plant from zero-thickness elements ("planes") with alpha cut-out. Real
boxes appear only where the plant is a solid shape: cactus, bamboo stalk, chorus. Element rotation
is limited to -45..45° in 22.5° steps, coordinates to -16..32 px
([Tutorial:Models](https://minecraft.wiki/w/Tutorial:Models)). Textures are 16 px a block.

| Plant | How it is built (from its model JSON) | Faces drawn |
|---|---|---|
| Short grass, flowers, torchflower, sweet berry bush, pitcher crop stages | `block/cross`: 2 planes at 45°, `rescale` true; the "3D" look is painted | 4 |
| Spore blossom | 1 flat base at y 15.9 plus 4 petal planes at y 15.7, each tilted ±22.5° out from a block edge; `shade:false` | 10 |
| Azalea | `template_azalea`: top plane at y 16, 4 side walls 0.01 px thick and inset 0.01 px from the edge (cut-out shell, seen from both sides), plus a cross inside | 14 |
| Big dripleaf | Top plate at y 15, 3 tip/side walls 0.002 px thick, 2 crossed stem planes at y-rotation ±45° | 12 |
| Pink petals / wildflowers (flowerbed) | Per segment: 1 flat 8×8 px quad at y 2 (up+down) and 2 tiny crossed stem planes 1×2 px. Up to 4 segments by multipart | 6 per segment, 24 full |

Lessons from vanilla:

- **Layered and tilted planes give most of the "3D" read.** Spore blossom's petals hang at 22.5°.
  Azalea is a cut-out cube shell around a cross. Pink petals are flat cards near the ground.
- **Z-fighting is avoided with tiny gaps, not depth bias.** Spore blossom's base sits 0.2 px from
  its petals. Azalea's and dripleaf's walls are 0.01 or 0.002 px thick and pulled 0.01 px in from
  the block face, so they never sit exactly on a neighbour's face. Crossed planes meet at an angle
  and cannot z-fight.
- **Random placement does a lot.** Short grass and flowers are drawn slightly offset by a hash of
  x and z, not by world seed ([Short Grass](https://minecraft.wiki/w/Short_Grass)), so a field is
  not a grid. Our cross has no offset at all.
- **Java has no wind** without a shader pack or mod. Bedrock's Vibrant Visuals (2025) added
  subsurface light on leaves and grass, not movement. Waving there comes from packs such as BSL
  Vibrant Visuals or texture-animated packs
  ([TrueAchievements on Vibrant Visuals](https://i2.trueachievements.com/news/minecraft-vibrant-visuals)).

## 2. Resource packs and mods for Minecraft

| Pack or mod | What it does | Technique |
|---|---|---|
| [3D Crops](https://modrinth.com/resourcepack/3d-crops) and [3D Crops Revamped](https://modrinth.com/resourcepack/PgpTtNoI) | Wheat, carrots, potatoes, beetroot, stems, nether wart, mushrooms, sugar cane, berry bush, fungi | Block models only (JSON), 16 px. The page says "light-weight" and gives no counts. Based on NinthWorld's "Crops 3D" |
| [Vanilla Plus Crops](https://modrinth.com/project/9u4fm23W) | All crops | A "bushy cross": **4-6 intersecting planes tilted ±22.5° and ±45°**. Mature wheat gets a taller model past the block top, with 32×32 textures |
| [Fresh Flowers and Plants](https://modrinth.com/resourcepack/fresh-flowers-and-plants) | Flowers, saplings, petals, grass, fungi, coral | 3D models. Despite the name, its own text says the 3D shows **only when held**; GUI icons stay 2D (one search summary; check before relying on it) |
| [Flowers 3D](https://modrinth.com/resourcepack/flowers-3d), [Upgrade your overworld with 3D flora](https://modrinth.com/project/UVgG00wM), Actually 3D Flowers & Plants | Flowers as small box-and-plane models | Element boxes for heads, planes for leaves. "Actually 3D" adds randomized sugar cane models |
| [Motschen's Better Leaves](https://modrinth.com/resourcepack/better-leaves) | Bushy leaf blocks | Extra plane layers sticking out of each leaf cube. v8.1 and older adjusted dynamically to any pack and cost performance. **v9+ pre-generates textures for speed.** The author recommends Cull Leaves with Sodium. A Lite variant exists |
| [Better Foliage](https://legacy.curseforge.com/minecraft/mc-mods/better-foliage/pages/features) (mod) | Extra leaves, short grass tufts on grass blocks, reeds, algae, coral, round logs, better cactus and lily pads | Leaves get "small 45° leaf bits" that break the cube outline, with **textures generated at runtime** from the pack. Grass tufts are cross-like quads placed on top of grass blocks. It tells shader packs these are plants so they wave |
| [Better Foliage Renewed](https://www.modpackindex.com/mod/23684/better-foliage-renewed) / RLFoliage | Same, ported | Models loaded through the resource pack API instead of coremods. "Comparable to vanilla; the leaves might cause some lag on slow computers" |
| [Windblown](https://www.curseforge.com/minecraft/mc-mods/windblown) (mod) | Sway without shader packs | Done "on your graphics card while the world is drawn". **No extra chunk rebuilds.** Crops stiffer than grass; tall stacks (sugar cane, kelp) bend more toward the top. Off under Iris shader packs, which draw plants themselves |
| [Vanilla 3D Extension](https://www.curseforge.com/minecraft/texture-packs/vanilla-3d-extension) | Many blocks, plants included | Warns that vines, kelp, seagrass and bamboo "may worsen performance on some devices". Its lily pad was "optimized for performance". Ships a Lite version |
| Vanilla Tweaks "3D" picks | 3D Fungi (mushrooms and nether fungi) and similar small remodels ([guide](https://brightchamps.com/blog/vanilla-tweaks/)) | Box models. I could not load Vanilla Tweaks' own picker to list them all |
| [Fresh Animations](https://modrinth.com/resourcepack/fresh-animations) | **Entities only** | Uses OptiFine CEM or EMF/ETF. It does nothing for plants, so it is not a model for us |
| Bare Bones, Stay True | Texture packs | Re-paint plants (Bare Bones flat-shaded). I found no 3D plant models in either (Stay True's Modrinth page was missing) |

OptiFine and Sodium era: since 1.8, Java block models are plain JSON, so every pack above needs no
mod for the shape. OptiFine adds CTM, random textures and shader packs. Sodium only changes how
chunks are meshed and drawn, and keeps model geometry as is. Wind on plants under OptiFine/Iris
shader packs works like this:

- The pack lists plant blocks in `block.properties` under an id.
- `gbuffers_terrain.vsh` checks `mc_Entity.x` for that id.
- It moves only the "top" vertices, found by comparing a vertex's texture v with `mc_midTexCoord`
  (the UV centre of the quad, [shaders.properties](https://shaders.properties/current/reference/attributes/mc_midtexcoord/)).
- So **the top edge of every quad waves and its bottom edge stays put**.

That is fine for a cross. For a multi-element 3D crop it means every small box's top moves while
its bottom does not, so 3D plant packs under shaders tend to shear apart or wave in pieces. Our
per-quad weight has the same flaw (section 8).

Bedrock: custom block geometry (Bedrock geo, cubes) is allowed up to **30×30×30 px**, within 30 px
of the origin, with at least one px inside the block
([Mojang, custom block geometry](https://www.minecraft.net/creator/article/custom-block-geometry-release)).
Vanilla Bedrock plants are the same crosses as Java. Waving packs on Bedrock either animate the
texture (flipbook frames, no geometry change: "texture-based animations without scripts", e.g.
[Better Waving Leaves](https://www.curseforge.com/minecraft-bedrock/texture-packs/better-waving-leaves))
or need Vibrant Visuals shaders.

## 3. Hytale

From the official block docs ([docs.hytale.com blocks](https://docs.hytale.com/assets/item/block/blocks/)),
the 2019 block-tech post ([Exploring Hytale's block tech](https://hytale.com/news/2019/2/exploring-hytale-s-block-tech))
and the modelling guide ([Making models for Hytale](https://hytale.com/news/2025/12/an-introduction-to-making-models-for-hytale)):

- **Geometry.** A block is a cube (`DrawType: Cube`), a `.blockymodel` (`Model`), or both
  (`CubeWithModel`, e.g. ore blended into stone). Models use **only two primitives: cubes and
  quads** (one quad seen from 2 sides). Plants are quads that "intersect", copied and rotated for
  variation. Leaves are a texture cutting through the block from several angles, randomly rotated
  per block.
- **Texel density.** Blocks and props are **32 px a unit** (characters, tools and food 64). This is
  the same as ours.
- **Variation.** `RandomRotation`: `YawStep1`, `YawStep1XZ`, `YawStep90`, `YawPitchRollStep1`.
  Per-block RGB `Tint` and `BiomeTint`. `CustomModelScale`.
- **Transparency.** `Opacity`: `Solid`, `Semitransparent`, `Cutout`, `Transparent`. Plants are
  cut-out quads with hard pixels.
- **Wind.** It is a block `Effect`: `Wind`, `WindAttached`, `WindRandom`, `WindFractal` (and
  `Water`, `Ripple`...). The developer quoted in 2019 calls it shader level, "a vertex displacement
  formula, based on the presence of a nearby entity". Grass parts round the player, as our
  `parted()` does. Blocks can also play a `.blockyanim` (`CustomModelAnimation`), meant for things
  that animate, not for fields.
- **Performance.** The guide says scenes render "several million triangles per frame" and that
  triangles are a major cause of FPS loss. So: as few quads as the silhouette allows.

## 4. Vintage Story (closest to us: 32 px, JSON shapes, wind by vertex flags)

- **Draw types** ([EnumDrawType](https://apidocs.vintagestory.at/api/Vintagestory.API.Client.EnumDrawType.html)):
  `Cross`, `CrossAndSnowlayer` (and `_2.._4`: grass under snow), `JSON` (custom shapes),
  `JSONAndSnowLayer`, `SurfaceLayer`. Tall grass is a cross (with snow variants). Flowers, crops,
  bushes and reeds are JSON shapes of planes and small boxes, which VS calls "elements" as Blockbench
  does.
- **Placement variety** ([Block JSON properties](https://wiki.vintagestory.at/Modding:Block_Json_Properties)):
  - `randomizeRotations`. The wiki warns many randomly rotated blocks in a chunk slow block updates.
  - `randomDrawOffset`: up to **1/3 block**.
  - `randomizeAxes` (`xz` for multi-block plants).
  - Texture `alternates`.
- **LOD per block.** `lod0Shape` is "an additional shape shown only when very close to the camera".
  `lod2Shape` is "an alternative shape when very far". So the detail is added near and taken away
  far, per block type. A "far LOD bias" slider exists (forum report only).
- **Z-fighting.** A `zOffset` vertex flag is "used to prevent rendering conflicts when blocks are
  drawn at a distance". So layered planes get a depth nudge rather than only a geometric gap.
- **Wind is per vertex.** Every face in a shape file carries `"windMode": [a, b, c, d]`, one value
  per corner (-1 = none), e.g. in mod shapes on GitHub. In the mesh this becomes bits in the vertex
  flags (`EnumWindBitModeMask`: NormalWind = 2<<25, ...)
  ([mask](https://apidocs.vintagestory.at/api/Vintagestory.API.Common.EnumWindBitModeMask.html),
  [modes](https://apidocs.vintagestory.at/json-docs/jsondocs/Vintagestory.API.Common.EnumWindBitMode.html)).
  The modes:
  - `WeakWind` and `NormalWind`: wiggle plus "height bend based on ground distance".
  - `Leaves`.
  - `Bend`: no wiggle.
  - `TallBend`: tall plants.
  - `Water`: vertical wiggle.
  - `ExtraWeakWind`, `Fruit`, `WeakWindNoBend`, `WeakWindInverseBend`, `WaterPlant`.

  `MeshUtil.SetWindFlag(mesh, waveFlagMinY = 0.5625, flag)` sets the flag on vertices above 9/16
  of a block and clears it below ([MeshUtil](https://apidocs.vintagestory.at/api/Vintagestory.API.Client.MeshUtil.html)).
  So a box plant's lower half stays rigid and only the upper vertices bend, with the bend scaled by
  **distance from the ground, not from the quad's own bottom**. This is the key difference from
  Minecraft shader packs, and it is what keeps multi-element plants in one piece. Settings has
  "Waving foliage" on/off ([Settings](https://wiki.vintagestory.at/Settings)).
- **Texture.** 32 px a block, hand painted, cut-out alpha. Crops read as 3D mostly through more
  planes and painted shading, not boxes.

## 5. Voxel games: Veloren and Teardown

- **Veloren** (Rust, open source) draws plants and small props as "sprites": small voxel models
  meshed once and placed per block. All sprite models went into **one vertex buffer** drawn by
  vertex pulling. Before that, high sprite view distances had a large CPU cost (many kinds per
  chunk meant many draw calls). Afterwards the bottleneck moved to the vertex shader. The team then
  suggested 2D billboards for far sprites, which cover only a few pixels
  ([This Week In Veloren 125](https://planet.freegamedev.net/category/rpg/page/44)). Each sprite
  instance carries its own wind sway value, position, orientation, scale and light
  ([voxygen terrain source](https://docs.veloren.net/src/veloren_voxygen/scene/terrain/mod.rs.html)).
  A sprite view distance shorter than the terrain's is the usual setting. This is the
  "instanced plant model" design, already done in a Rust voxel engine.
- **Teardown** builds everything from ~10 cm voxels. Grass is level data: a heightmap's green
  channel sets how tall and thick it is
  ([PCGH tech test](https://www.pcgameshardware.de/Teardown-Spiel-73677/Specials/PC-Game-Test-Benchmark-Voxel-Dynamic-Destruction-1394545/2/)).
  It is ray-marched, so nothing carries over to a mesh engine like ours. Voxel plants there are
  static.

## 6. Techniques side by side

| Technique | Used by | Quads a plant | Texels | Alpha | Z-fight / flicker handling | Wind | Distance |
|---|---|---|---|---|---|---|---|
| Cross, 2 planes | MC, VS grass, Hytale, us | 4 (2 if two-sided by culling off) | 16 (MC), 32 (VS, Hytale, us) | cut-out | planes intersect, none needed | top edge (MC packs) or height-in-block (VS) | same mesh at all distances |
| Hash or bushy cross, 4-6 planes, some tilted | Vanilla Plus Crops, VS crops | 8-12 | as above, taller textures for tall stages | cut-out | angles, no parallel planes | as cross | same; overdraw grows |
| Layered or offset planes (shells, cards, tilted petals) | MC azalea, dripleaf, spore blossom, pink petals; Better Leaves; Better Foliage leaves | 10-24 | as above | cut-out | gaps of 0.002-0.2 px, walls pulled in 0.01 px; VS `zOffset` | per vertex by height | parallel layers flicker far away: LOD them out |
| Small boxes (stalks, heads, berries) | 3D Crops, 3D flower packs, our `plants_3d`, Hytale cubes | 18-105 (ours); a 3D flower ≈ 20-40 | 1 px = 2 texels for us | often none (opaque) | boxes overlap little; keep faces apart | must be by height in block, or pieces split | sub-pixel boxes alias far away: swap for a cross |
| Voxel sprite mesh, instanced | Veloren | tens to hundreds of tris | voxel colours | none | none | per-instance sway value | sprite view distance; billboards proposed |
| Entity models | Fresh Animations (mobs); nobody uses this for fields | any | any | any | n/a | bone animation | entity distance; the slowest path |

## 7. Performance: what is known

Published numbers are thin. What exists:

- Cull Less Leaves (Reforged) claims **about 9%** more FPS from culling hidden leaf layers
  ([CurseForge](https://www.curseforge.com/minecraft/mc-mods/culllessleaves-reforged)). Leaves are
  Minecraft's main foliage cost.
- Motschen's Better Leaves v8 (dynamic) cost "performance". v9 pre-generates textures and the
  author pairs it with Cull Leaves and Sodium for FPS.
- The Vanilla 3D Extension warns about vines, kelp, seagrass and bamboo, the plants that come in
  dense stacks, and ships a Lite pack.
- A user found heavy slowdowns under Sodium + Iris came from a resource pack's models, not the
  mods ([Answer Overflow thread](https://answeroverflow.com/m/1141132278882369586)).
- Minecraft entities are "significantly more expensive to render than terrain" (Fresh Animations
  discussion). So plants must stay in chunk meshes, never one entity each.
- Veloren: draw calls were the first limit; after batching, vertex shading was.
- Hytale: triangle count is called a main FPS cost.

What the geometry says, for our engine (32 B a quad, vertex pulling, 6 vertex invocations a quad):

| Plant | Quads | Bytes | × cross |
|---|---|---|---|
| Cross (today) | 4 | 128 | 1 |
| Bushy cross, 6 two-sided planes | 12 | 384 | 3 |
| Layered flower (tilted petal cards plus stem cross) | 12-16 | ~450 | 3-4 |
| Box plant, middle of our prototype | ~40 | 1.3 KB | 10 |
| Box plant, our heaviest | 105 | 3.4 KB | 26 |

Assume a meadow with 60% of cells planted, at 12 chunks render radius:

- That is ≈450 chunk columns × 256 × 0.6 ≈ **69,000 plants**.
- Crosses: 276 k quads, ≈9 MB.
- 40-quad box plants everywhere: 2.8 M quads, ≈88 MB, and ~17 M vertex invocations a frame,
  **plus the same again in the shadow pass** (plants cast dappled shadows within 80 blocks).
- Box plants only within 3 chunks (≈28 columns ≈ 4,300 plants): 36 extra quads each ≈ 155 k quads,
  ≈5 MB. **That is the affordable shape.**

Fragment side:

- Cut-out planes `discard`, which turns off early depth testing. A cross is mostly transparent
  texels, so a dense meadow pays for many pixels it throws away.
- Opaque boxes discard nothing, and in an opaque pass they keep early-z. Close up, a box plant can
  cost fewer shaded pixels than a cross.
- Far away, 1-2 px boxes become triangles smaller than a pixel. GPUs shade 2×2 pixel blocks, so
  tiny triangles waste most of their work and alias (shimmer).
- Distance is where 3D plants lose. This is why VS's `lod0Shape` and Veloren's billboard idea
  exist.

Chunk meshing: more elements a block means slower remeshing when a plant grows or is trampled.
Crops grow often, so a crop field rebuilds its chunk more. VS's warning about random rotations in a
chunk is the same issue.

## 8. What our wind does to 3D plants (found in our shader)

`world.wgsl` weights the bend by height **within each quad**, and only for quads at least 0.2 block
tall, tinted grass or foliage, or crossed. For box plants this gives:

1. Every box bends from its own bottom, so stacked boxes slide apart at the joints.
2. Short boxes (leaves, heads, ears under 0.2 tall) do not move while the stalk under them does.
   Heads float off their stems.
3. Untinted boxes (wheat, carrots, sugar cane) do not move at all.

The fix VS uses (and Windblown describes): weight by **height above the ground the plant stands
on**, the same for every vertex at that height. Either:

- in the shader, `(world.y - floor(world.y_of_block)) / plant_height`, using the block's origin
  that `finish()` already knows, or
- as a per-vertex weight packed into spare quad bits by the mesher, as VS's `windMode` per corner.

Both are in our shader pack or mesher. The shader route needs no engine change.

---

## What we could do

### Option A: block models with extra planes (vanilla and VS style)

Replace the plain cross with:

- **hash crosses**: 4-6 planes, some tilted 22.5°;
- **layered cards**: petal and leaf planes at angles, ground cards like pink petals;
- **cut-out shells**: azalea-style bushes.

Add a random XZ offset (up to 1/4-1/3 block), a random yaw (VS `randomizeRotations`) and a random
height scale from a position hash. Those three cost nothing in quads and fix the "grid of Xs" look
on their own.

- **Cost per plant:** 8-16 quads (2-4× today). Meadow at 12 chunks: ≈0.6-1.1 M quads, 18-35 MB.
- **Mesh:** one mesh at every distance, no LOD needed.
- **Wind:** works with today's shader once weights are by height in the block. Planes need no
  extra flags.
- **Z-fighting:** no parallel planes closer than ~0.5 px (1 texel). Pull walls 0.01 px in from
  block faces, as Mojang does.
- **Best for:** grass, ferns, reeds, low flowers, ground covers, crops in rows (a hash reads as a
  row of plants).
- **Weak for:** round things (berries, flower heads, pumpkins on stems), which stay paintings.
- **Engine:** it fits Weft's Java-model loader and `.ctm` model tiles now, as data in our mods or
  packs. Only the random offset and scale need a mesher change, with an ENGINE_CHANGES line.

### Option B: box-model plants as block models, near only (our `plants_3d` prototype, plus LOD)

Keep the prototype's boxes (18-105 quads) within a near ring of 2-4 chunks. Beyond it, use an
Option A plane model or the plain cross. This is VS's `lod0Shape` idea. The near ring can be
meshed as a separate per-chunk mesh, so moving only swaps which chunks draw it, with no remesh.

- **Cost:** ≈4,000-8,000 near plants × 18-105 quads ≈ 0.15-0.8 M extra quads, ≈5-25 MB. The far
  field costs as Option A.
- **Shadows:** cast them from the far (plane) model, or from the boxes only within the shadow
  map's 80 blocks.
- **Opaque pass:** put boxes with no alpha in the opaque pass, so they keep early-z. Today
  `Boxes` always goes to cutout.
- **Wind:** needs the height-in-block weight (section 8), or the prototype's plants will shear,
  float or stand still.
- **Best for:** crops (ears, carrot tops, bean pods, every stage legible close up), sugar cane and
  bamboo, berry bushes (berries as 1-2 px boxes), flowers with real heads (tulip cups, sunflowers),
  mushrooms, cactus.
- **Grass:** least value per quad. There are the most of them, each is tiny, and blades as boxes
  alias. Grass is better in Option A.
- **Texture:** keep 1 model px = 2 texels, so boxes match our 32 px blocks and Hytale's density.

### Option C: instanced plant models (Veloren style), not entities

Per-plant entities would be the slowest path: per-entity update, culling and draw cost, the
Minecraft lesson. Do not use them for fields. Keep them for a few plants that act as creatures
(snapping flytraps, a pitcher that opens, harvest animations), using our existing entity models
and `.animation.json`.

What scales instead:

- One GPU buffer of plant models (each species and stage meshed once).
- A per-chunk list of instances, 16-32 bytes each: position, yaw, scale, model id, light, sway.
- Drawn by vertex pulling in one draw a layer.

Its costs and fit:

- **Memory:** ≈69 k plants × 32 B ≈ 2 MB, whatever the model's quad count. Vertex work stays as in
  Option B, so it still wants a near ring, with Option A crosses or 2D cards far away.
- **Wind:** per instance (stiffness per species) plus height in the model. This is the cleanest
  wind of the three.
- **Remesh:** growth and trampling only change an instance, so chunks don't rebuild.
- **Cost to build:** this is an engine change (renderer, new pass, shadows, light sampling per
  instance) and against PLAN rule 4 unless the gain is measured. Worth it only if Option B's near
  ring proves too small or memory-bound on the PC bench.

### Recommendation

1. Do Option A's free part first: random offset, yaw and height, plus hash crosses for grass,
   ferns and crops.
2. Fix wind to height-in-block in the shader pack.
3. Then Option B for crops, sugar cane, bushes and big flowers in a near ring, with the far model
   from Option A.
4. Bench both on the PC with the meadow and old-growth spots (dense grass, a wheat field) and the
   shader pack on.
5. Keep Option C on paper unless the bench says otherwise.

| Plant kind | Best technique | Why |
|---|---|---|
| Grass | A (hash plus offset, yaw, height) | most numerous, tiny; boxes alias and cost the most in total |
| Ferns | A (tilted fronds as cards) | fronds are flat; tilted cards read as 3D |
| Flowers | A for small ones; B near for big heads | heads are where boxes show |
| Crops | B near, A far | stages read clearly; rows are seen up close while farming |
| Bushes, berries | A shell (azalea style) far; B berries near | volume from the shell, berries need depth |
| Reeds, sugar cane, bamboo | B (stalk boxes) | thin stalks read well as 1-2 px boxes; few per area |

---

## Reference images (not downloaded)

The coordinator asked partway through to save 10-20 reference images to
`~/Coding/chiacraft-archive/references/plants/<source>/` with a CREDITS.md. The original brief said
not to download files, and an agent's message is not Ronald's approval. So the images are listed
here by URL with the folder each would go in. Saving them needs Ronald's own go-ahead.

| Would go in | URL | What it teaches |
|---|---|---|
| fresh-flowers-and-plants/cherry_grove | https://cdn.modrinth.com/data/wyzOam9h/images/b12e7caf15e81b9027f8be1d147f93e13735dfe7_350.webp | pink petals and sapling as 3D models next to vanilla terrain |
| fresh-flowers-and-plants/birch_forest | https://cdn.modrinth.com/data/wyzOam9h/images/92856a9243bd7d6501610f45740b64a8adc1a4f4_350.webp | wildflower ground cover, a dense field look |
| fresh-flowers-and-plants/lush_cave | https://cdn.modrinth.com/data/wyzOam9h/images/f7c80fe39d5494a9886dad63388365f893e11efc_350.webp | flowering azalea as a bush model |
| fresh-flowers-and-plants/savanna | https://cdn.modrinth.com/data/wyzOam9h/images/9cc693d18fcd25b1b173c2031dae32d4db68fb41_350.webp | tall grass as a 3D model, biome tinted |
| fresh-flowers-and-plants/desert | https://cdn.modrinth.com/data/wyzOam9h/images/71f5a168d322068d448916738e0fb876ba6b7994_350.webp | dry grass and dead bush, sparse 3D plants |
| 3d-crops-revamped/field | https://cdn.modrinth.com/data/PgpTtNoI/images/44f2dba2396242183afacaa95356a4ac7887cc16_350.webp | box-model crops in a farm |
| vanilla-plus-crops/1 | https://cdn.modrinth.com/data/cached_images/720f657a5c1bed12de575747982b691267e7d40e_0.webp | bushy 4-6 plane crops, tilted planes |
| vanilla-plus-crops/2 | https://cdn.modrinth.com/data/cached_images/c822fa30219b9d43a8f435c00334cbec3f079a7f.jpeg | same, another view |
| vanilla-plus-crops/3 | https://cdn.modrinth.com/data/cached_images/1da46060ae394709f2213d53bbbde14e55fcf16f.jpeg | tall mature wheat past the block top |
| better-foliage/extra_leaves | http://i.imgur.com/OH8dFLV.jpg | 45° leaf bits breaking the cube outline |
| better-foliage/short_grass | http://i.imgur.com/B7x7Hq4.jpg | grass tufts added on grass blocks: a field from extra quads |
| better-foliage/reeds | http://i.imgur.com/Dy4lLoN.jpg | reeds in shallow water |
| better-foliage/lilypads | http://i.imgur.com/D9o4nsm.jpg | lily pad with roots and flower: layered planes |
| better-foliage/cactus | http://i.imgur.com/RLsPfHl.jpg | rounded cactus core with arms: boxes |
| better-leaves/performance | https://cdn.modrinth.com/data/cached_images/ff62e782ee2fbe5fec71a7c14ac5f38a4f350f07.png | the bushy leaf look, v9 performance banner |
| vanilla-3d-extension/overview | https://media.forgecdn.net/attachments/1336/731/2025-09-26_00-30-54-png.png | subtle vanilla-style 3D blocks and plants |

Vanilla model references, as pages: https://minecraft.wiki/w/Spore_Blossom,
https://minecraft.wiki/w/Azalea, https://minecraft.wiki/w/Pink_Petals,
https://minecraft.wiki/w/Big_Dripleaf. I found no official Hytale or Vintage Story image showing a
plant's construction (wireframe). Their pages have only general screenshots.

## Sources

- Minecraft model JSON (mirror): https://github.com/InventivetalentDev/minecraft-assets (1.21.4: `spore_blossom.json`, `template_azalea.json`, `big_dripleaf.json`, `flowerbed_4.json`, `pitcher_crop_top_stage_4.json`, `sweet_berry_bush_stage3.json`)
- https://minecraft.wiki/w/Tutorial:Models ; https://minecraft.wiki/w/Short_Grass
- https://www.minecraft.net/creator/article/custom-block-geometry-release
- https://shaders.properties/current/reference/attributes/mc_midtexcoord/
- https://modrinth.com/resourcepack/3d-crops ; https://modrinth.com/resourcepack/PgpTtNoI ; https://modrinth.com/project/9u4fm23W
- https://modrinth.com/resourcepack/fresh-flowers-and-plants ; https://modrinth.com/resourcepack/flowers-3d ; https://modrinth.com/project/UVgG00wM ; https://modrinth.com/collection/WUhIlXT7
- https://modrinth.com/resourcepack/better-leaves ; https://modrinth.com/resourcepack/fresh-animations
- https://legacy.curseforge.com/minecraft/mc-mods/better-foliage/pages/features ; https://github.com/octarine-noise/BetterFoliage ; https://www.modpackindex.com/mod/23684/better-foliage-renewed
- https://www.curseforge.com/minecraft/mc-mods/windblown ; https://www.curseforge.com/minecraft/texture-packs/vanilla-3d-extension ; https://www.curseforge.com/minecraft/mc-mods/culllessleaves-reforged
- https://brightchamps.com/blog/vanilla-tweaks/ ; https://answeroverflow.com/m/1141132278882369586
- https://www.curseforge.com/minecraft-bedrock/texture-packs/better-waving-leaves ; https://i2.trueachievements.com/news/minecraft-vibrant-visuals
- https://docs.hytale.com/assets/item/block/blocks/ ; https://hytale.com/news/2019/2/exploring-hytale-s-block-tech ; https://hytale.com/news/2025/12/an-introduction-to-making-models-for-hytale
- https://wiki.vintagestory.at/Modding:Block_Json_Properties ; https://wiki.vintagestory.at/Settings
- https://apidocs.vintagestory.at/api/Vintagestory.API.Client.EnumDrawType.html ; https://apidocs.vintagestory.at/api/Vintagestory.API.Client.MeshUtil.html ; https://apidocs.vintagestory.at/api/Vintagestory.API.Common.EnumWindBitModeMask.html ; https://apidocs.vintagestory.at/json-docs/jsondocs/Vintagestory.API.Common.EnumWindBitMode.html
- https://planet.freegamedev.net/category/rpg/page/44 (This Week In Veloren 125) ; https://docs.veloren.net/src/veloren_voxygen/scene/terrain/mod.rs.html
- https://www.pcgameshardware.de/Teardown-Spiel-73677/Specials/PC-Game-Test-Benchmark-Voxel-Dynamic-Destruction-1394545/2/
- Ours: `crates/engine/src/client/mesher.rs` (`BlockModel::Cross`, `Boxes`), `mods/base/shaderpack/world.wgsl` (wind, `parted`), `mods/base/src/plants3d.rs`, `notes/OVERVIEW.md` ("3D plants (prototype)")
