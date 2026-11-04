# How the game looks (style)

Decided 2026-11-02 (Ronald): **Hytale-like**; **64 texture pixels a block for everything**: blocks,
creatures and items (picked on the Pixel Density Picker, after first choosing 32 for blocks), so
the world, the animals and what players hold all show the same detail; items are **3D models**
(boxes, as the creatures), drawn from their model in slots and in the hand at one fixed angle;
textures **painted by Claude** by rule; the HUD after designs Ronald sends.

Changed since (2026-11-03/04, Ronald): **32 texture pixels a block for everything** (blocks, item
icons, box models), and items stay flat pictures (the `game-art` skill's guides, "0 Making it").

## Where it stands

- Block size decided (2026-11-03): **32 px**, like the items, from Ronald's block style guide
  (the style guide test, `notes/ui-codex/guides/`). The whole set redone at 32 (the `block-textures`
  skill) is in the game, merged 2026-11-03; the 64 px set below is the one it replaced.
- Blocks (2026-11-03): 107 textures at 64 px, generated with Codex (ChatGPT's image model) from
  one prompt, the palette (`notes/texture-test/palette.png`) and the approved tiles, four an image
  (`notes/texture-test/codex/gen_grid.sh`, `gen_sprites.sh` for plants), cut by `grid_cut.py` and
  put in the game by `install_blocks.py` (see `notes/texture-test/PROMPTS.md`). Still 16 px: the
  model-mapped ones (torches, redstone dust, repeaters, comparators, levers, buttons, beds, the
  piston's inside), glass, water and lava (animated), the anchor and floater.
- Creatures and props (2026-11-04): every box model repainted at **32 texels a block**, like the
  blocks and items (Ronald: "Yes, rebuild at 32"; the blocky-model skill, `.claude/skills/blocky-model`).
  Boxes, rigs, poses and animations kept; eyes redesigned for faces of 4-7 texels. Finer only
  where 32 would lose the face (under 3 texels): raven, gull, vulture and fish at 64; mouse, vole,
  kryptobaatar, small bird and lizard at 128.
- Items: the generated icons (the kit's `items/`, 48 px) are installed for tools, ingots, ores, gems
  and the animals mod's; the rest are old 16 px sprites. To remake as small box models at 64 px a block. To check first:
  block and item models take texture coordinates only in 128ths of the texture (the skill's
  note), which may need an engine change for painted item models.
- Plants (tall grass, flowers, ferns, wheat) are painted 64 px cut-outs.
- The HUD and screens: the UI kit, weathered bronze frames in the HUD (amber only when selected),
  Alegreya Sans text; the layout is still the engine's (see the mockups in `notes/ui-reference/`).

## The look

As the creatures already are (`blocky-model` skill, "Texture"): one light from above and a bit
front-left, baked in; tops lightest, bottoms darkest; smooth light in fine steps, not 3-5 hard
bands; hue-shifted ramps (shadows warmer and more saturated, lights toward yellow); big simple
color areas; soft edges between colors; never pure black or white. Saturated but natural, chunky
readable shapes.

## The HUD and screens: the style brief

Ronald's reference sheets (`notes/ui-reference/`, eight sheets: panels and frames, inventory
slots, buttons and controls, the survival HUD, navigation and icons, taming, crafting and
workstations, the main menu) are the target, made from his brief:

- Detailed, hand-crafted pixel-art fantasy UI: dark charcoal-brown panel interiors, weathered wood
  and bronze frames, warm golden highlights; subtle bevels, squared-off (chamfered) corners,
  pixel-sharp silhouettes, restrained texture and shading.
- The same border thickness, corners, shadows, padding, slot proportions and icon scale on every
  part; warm ivory text in a restrained, readable typeface.
- Selected glows amber-gold, inactive stays muted brown, disabled is desaturated; every part with
  normal, hover, pressed, selected and disabled states where they apply.
- Empty frames and slots, no baked-in labels or numbers: the engine draws text and data.

The sheets are paintings, not pixel-exact assets (edges soft, the same frame a little different
from sheet to sheet), so each part is redrawn at the engine's sizes to match them, not cut out.

Game systems the brief assumes (decided by Ronald, not built yet): health and stamina as bars,
not hearts; three timed food buffs raising maximum health and stamina, no hunger meter (Valheim);
temperature from freezing to hot; crafting grids up to 5x5; shared parts for inventory, chests,
equipment, cooking, building, skills, maps, quests and creatures; taming with progress,
effectiveness, food and status; creature commands follow, stay, move, passive, neutral, attack and
a whistle.

## Type

Decided 2026-11-02 (Ronald, after comparing on the UI kit page): **the reference sheets'
lettering**: **Alegreya Bold** for titles, **Alegreya Sans** for everything else (bold on buttons and
tabs, medium for text, extra bold for numbers), ivory `#F5E4BC`, amber `#FFBE45` for highlights, a
dark brown outline. Smooth letters over pixel-art frames, as the sheets (and Hytale) do; Alegreya has
Cyrillic. Also tried: Pixelify Sans, Silkscreen, Almendra, Press Start 2P (Pixelify Sans is kept in
`mods/base/source/ui/fonts/`, OFL).

In the game (2026-11-03): **Alegreya Sans Medium** for all text, baked from the font file by
`mods/base/source/ui/font_bake.py` (run with `~/.venvs/blocky/bin/python`, it needs fontTools) into
`mods/base/assets/fonts/default.png` and `.font`: a smooth coverage atlas, 3 atlas pixels a font
pixel, capitals 11.5 font pixels tall standing on the pixel font's baseline, the font's lining
figures in place of its old-style ones, wider spaces. The engine (`crates/engine/src/ui/font.rs`,
`Font::coverage`) takes a mod's `fonts/default` over its own pixel font, samples it smoothly, and
fills characters the font lacks from Unifont. Rendering the font file at runtime instead (sharp at
every GUI scale) was considered and left: ab_glyph is already in the build if it is wanted. Still
to do: a second face (Alegreya Bold) for titles and bold buttons, and the fonts' files are in
`mods/base/source/ui/fonts/` (OFL).

## The HUD: what images there are

Easiest: send what you have in mind in any form (screenshots of other games, a Figma frame, a
sketch, a mockup at 1920x1080), and I cut it into the files below. This is the list for whoever
draws them.

**How the engine uses them** (`docs/ui.md`, "GUI textures"):

- Sizes are in **GUI pixels**; the GUI scale setting multiplies them (2x, 3x, 4x on screen).
- A smooth, high-resolution look (Hytale's HUD isn't pixel art) is drawn at 4x and given
  `size = w h` in its `.gui` file: the picture has more pixels than the GUI pixels it covers.
- Backgrounds that stretch (panels, buttons, tooltips) are **nine-slice**: the corners keep their
  size, the edges and middle stretch (`border = 4` in the `.gui`).
- States are separate files by suffix: `_hover`, `_pressed`, `_disabled`, `_focused`.
- Pictures can be animated (frames stacked down) or filled by a shader; panels can animate as they
  open.
- Text colors are named tokens (`color.default`, `color.title`, `color.dim`, `color.button_hover`
  ..., in `mods/base/assets/lang/en_us.lang`), set once for the whole game; the font is a pixel
  font (`crates/engine/assets/fonts`) and can be replaced.
- Players can move and hide every HUD part (Settings > Arrange HUD), so parts are separate pieces,
  not one picture.

**In-game HUD** (size now, in GUI pixels):

| Part | Files | Size |
|---|---|---|
| Crosshair | `crosshair` | 9x9 |
| Hotbar slot, selected slot | `hotbar`, `hotbar_selected` | 18x18, frame |
| Health | `heart_full`, `heart_half`, `heart_empty` | 9x9 each (or one bar) |
| Hunger | `food_full`, `food_half`, `food_empty` | 9x9 each |
| Armor, air | `armor_full/half/empty`, `air_full` | 9x9 each |
| Attack charge | `attack_bar`, `attack_bar_fill` | 16x4 |
| Held item's name, chat | text over a backing | nine-slice |
| Toasts (advancements) | `advancement_toast` | 160x32 |
| Vignette | `vignette` | full screen |

**Screens** (inventory, machines, menus, settings):

| Part | Files | Size |
|---|---|---|
| Panel, sidebar, well (a sunken area), tooltip | `panel`, `sidebar`, `well`, `tooltip` | nine-slice |
| Item slot, highlight | `slot`, `slot_highlight`, `recipe_slot` | 18x18 |
| Equipment slot icons | `slot_helmet`, `_chestplate`, `_leggings`, `_boots` | 16x16 |
| Button (and the red quit button) | `button`, `_hover`, `_pressed`, `_disabled` | nine-slice |
| Text input | `input`, `input_focused`, caret, selection | nine-slice |
| Scrollbar, slider | `scrollbar`, `scrollbar_handle(_hover)`, `slider`, `slider_knob(_hover)` | 6x8, 12x15, 16x16 |
| Progress (furnace) | `arrow` + `arrow_fill`, `flame` + `flame_fill` | 24x16, 14x14 |
| Small icons | `icon_close`, `icon_cycle`, `icon_settings`, LEDs | 16x16, 6x6 |
| Title bar grip, background behind screens | `titlebar_grip`, `background`, `overlay` | 16x6, tiled |

**Main menu**: `logo`, menu icons (`menu_singleplayer`, `menu_multiplayer`, `menu_settings`,
`menu_mods`, 9x9), the news panel, the background.

**The animals' own** (later, to match): the animal screen (stats, wild and tamed points, bond,
food), taming bars and bubbles, the spyglass view, the breeding and egg readouts.

## Prompts for generated art

The UI parts and item icons are made in ChatGPT. The master prompts, the sizes, and how the sheets
are cut into game parts are in the `game-art` skill (`.claude/skills/game-art/SKILL.md`).

## The UI kit in the game

Since 2026-11-02 the game wears the kit: `mods/base/source/ui/install.py` copies `kit/` (the cut
parts) into `mods/base/assets/textures/gui/` under the engine's names, shrunk by 18/32 so the
kit's 32-pixel slot fits the engine's 18-pixel slot layout and every part keeps its size relative
to the others. `ui.wgsl` samples GUI pictures sharp-bilinear (crisp pixels, smooth edges, smooth
when drawn smaller than their pixels). Text colors: ivory, amber titles, dark brown shadows, dark
text on the amber buttons (`lang/en_us.lang`). Rerun `install.py` after changing the kit.

In: slots (hover glow, selected), hotbar, panels, wells, tooltips, buttons (and a red quit button
made from them), inputs, scrollbars, sliders, the crosshair, the furnace arrow and flame, the
advancement toast and tiles, equipment slot silhouettes, close/refresh/settings icons, lights,
hearts and food (half and empty made from the full icons), air bubbles.

### Still to generate

Each with the UI master prompt (`game-art` skill), the approved slot attached:

| What | Message |
|---|---|
| Menu background | `Next asset: menu background tile, 64 x 64 pixels, one state. A dark charcoal-brown stone or wood tile that repeats seamlessly on all four sides, very low contrast so text and buttons read over it, no frame.` |
| Title logo | `Next asset: game logo "Chiacraft", 256 x 64 pixels. Chunky carved letters in amber-gold with a copper-bronze edge and dark outline, light from the top left, no background frame.` |
| Main menu icons | `Next asset: menu icons, 16 x 16 pixels each, in one row: singleplayer (one figure), multiplayer (two figures), settings (gear), mods (puzzle piece). Ivory symbols with a dark outline, no frame.` |
| Armor points | `Next asset: armor points, 9 x 9 pixels each, in one row: full chestplate, half chestplate (left half full, right half dark), empty chestplate (dark outline only). Steel-grey with a copper trim, dark outline.` |
| Attack charge | `Next asset: attack charge bar, 16 x 4 pixels, items left to right: empty frame (dark inside, thin copper frame), fill (amber-gold). Plain even edges so it can stretch.` |
| Advancement line | `Next asset: connector line between advancement tiles, 16 x 4 pixels, states left to right: not done (dark copper), done (bright amber-gold). Plain even edges so it can stretch.` |
| Title bar grip | `Next asset: title bar grip, 16 x 6 pixels, one state: three small raised copper dots in a row on transparent.` |
| Waiting light | `Next asset: small status light, 6 x 6 pixels, one state: amber, like the attached radio button dot.` |
| Creative tabs | already drawn (`tab`, `tab_active`): the creative inventory draws its tabs as buttons; the engine needs a tab class to use them |
| Arrange HUD frames | `Next asset: HUD element frame, 32 x 32 pixels, states left to right: normal (thin dashed copper outline), hidden (the same in grey), selected (amber-gold). Transparent inside.` |
| Death screen, vignette | later: dark red-brown overlay and vignette |
| Bandage (item icon) | with the item master prompt: `One item: bandage (rolled off-white linen strip, a loose end trailing, a small red stain).` |
| Dung (item icon) | with the item master prompt: `One item: dung (a small heap of dark brown droppings, a few fibres of chewed grass in it).` Placeholder: darkened dirt. |
| Compost (item icon) | with the item master prompt: `One item: compost (a handful of dark crumbly earth with flecks of leaf and twig).` Placeholder: the podzol texture. |
| Compost bin (block, textures) | a block texture batch: slatted wooden bin sides, and its top empty, filling (rotting greens and browns) and ready (dark crumbly compost). Placeholder: oak planks, a dirt or podzol top. |

### Engine work still open

- **Font**: still the pixel font. Alegreya Bold (titles) and Alegreya Sans (the rest) need the
  smooth-font change (see "Type" above) and the font files (asking before downloading).
- **Health and stamina bars** instead of hearts and food (the kit's `bar`, `bar_health`,
  `bar_stamina`, `bar_xp`, food buffs with timers, temperature): a HUD change in the base mod.
- **Creative tabs** as tabs (`tab`, `tab_active`, `tab_hover`).
- **Item icons** in slots: the generated icons (`kit/items` when added; `notes/ui-reference/items`)
  replace `textures/item/*.png`, which the hand also shows until each item has its model.
- **Bigger slots**: the kit is drawn for 32-pixel slots at GUI scale 2; the screens lay out 18.

### HUD layout

After Ronald's reference screenshot (2026-11-02): survival rows top left (health, food, armor,
air), the main screens' keys under them (inventory E, recipes R, advancements L, each a framed
picture), the clock (sun by day) and x/z position top right, advancement toasts under them, slot
numbers on the hotbar (`mods/base/src/hud.rs`, `player.rs`; players still move them in Settings >
Arrange HUD).

Not yet, needing more than art:
- **Portrait** of the player's face top left (the face from the skin, framed).
- **XP bar and level** under the food: the game has no experience yet.
- **Minimap** (round, north marker) with the **biome name** under it: a map renderer, and the
  biome on the client.
- **Quest tracker** ("A New Home": goals with counts): no quests yet; advancements could feed it.
- **Bigger HUD**: the reference's hearts and hotbar are about twice ours (the 18-pixel layout).
- **Tool durability** under the slot as the kit's `durability` bar (now plain colored bars).
- A **moon** for the night clock (the sleeper stands in).
