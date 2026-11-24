# UI Style Guide

This guide covers the game's interface: windows, panels, slots, buttons, bars, HUD icons, tabs and tooltips. It is built on **your own UI reference sheets**, which are included in `UI Prompt References/`, with the palette measured from the crafting and workstations sheet.

The reference texture set's own UI (`01 Reference UI pieces.png`) is included for comparison. It's darker and plainer than your sheets: dark brown-grey panels with bevelled slots and a pale rim. **Your sheets are the target.** Use the reference pieces only for how small HUD icons are built at low resolution.

---

## 1. Your UI style (from your sheets)

### Palette

| Role | Colour | Notes |
|---|---|---|
| Page background | `#1a1313` | near-black warm brown |
| Panel interior | `#16100f` | slightly darker than the page |
| Slot background | `#352015` | where icons sit; slot rim `#683e26` → `#965735` |
| Bronze frame | `#89451e` → `#a75120` → `#b35a26` | 5–6 px border with corner bevels |
| Title tab and main button | `#e47c02`, light edge `#f6b24e`, dark edge `#d0730a` | the main call-to-action colour |
| Hover or selected glow | `#ffdd20` → `#f3ab1f` | 2-px glowing rim on the selected slot or button |
| Heading text (gold) | `#f2da9a` (range `#d2b782`–`#fcf0b9`) | serif display headings |
| Body text | `#e4dacf` cream, muted `#c6a383` | |
| Progress bar | fill `#f79701`, track `#271e1a` | |
| Health | red, about `#e0302a` with a lighter top edge | |
| Stamina | orange-yellow, about `#f5a623` | |
| Experience | purple, about `#9b4fe0` | |
| Success or confirm | `#8ac80f` (`#557d0a` dark, `#c1ed42` light) | ticks, "Craftable", confirm buttons |
| Error or cancel | panel `#3f1211`, icon red `#d8221e` | "Missing materials", cancel buttons |
| Info or upgrade | bright blue, about `#2f8de4` | special buttons only |

### Components

| Component | How it looks |
|---|---|
| **Panel** | A dark interior inside a 5–6 px bronze frame with bevelled corners, and a title tab in orange or bronze breaking the top edge |
| **Section box** | The same frame, thinner (3–4 px), with a small bronze label tab at the top-left |
| **Slot** | A dark brown square (`#352015`) with a 1–2 px inner rim, slightly sunken (darker top-left inner edge, lighter bottom-right) |
| **Selected slot** | The same slot with a 2-px yellow-orange glow rim |
| **Button** | A rounded rectangle, orange for main actions and bronze-brown for secondary ones. States: normal, hover (brighter, with a glow), pressed (darker, 1 px lower) and disabled (grey-brown, flat). Optional icon on the left. |
| **Tab** | Like a button, but the active tab is bright orange and joined to the panel below. Inactive tabs are dark bronze. |
| **Bar** | A dark track with a 1-px bronze rim. The fill has a lighter top line and a darker bottom line, and the value is written in cream text on top. |
| **Toggle, checkbox, radio** | Orange or green when on, dark when off, with a bronze rim |
| **Status icon** | A small square bronze-framed tile with one clear symbol in a bright colour (heart, bolt, snowflake, flame, drop, skull) |
| **Tooltip** | A dark panel with a thin bronze rim, a gold title and cream body text |
| **Badge** | A small red or orange circle or tag with white or cream text |

### Rules

- **Warm everywhere:** bronze, orange and cream. Use cool colours only for meaning (cold, water, mana, upgrade).
- **Icons sit in slots:** item icons follow the item guide and must read on `#352015`.
- **One pixel scale:** UI pixels are the same size as item and sprite pixels. Frames are multiples of that pixel, never half-pixels or smooth gradients.
- **Clear states:** every interactive piece has normal, hover, pressed and disabled versions.
- **Readable at small sizes:** chunky symbols, high contrast against the slot, and at most 2–3 colours per icon.
- **Pixel text:** a pixel serif for headings and a clean pixel sans for body text, in gold and cream.

### What the reference adds for HUD icons

The reference's own heart, food, armour and air icons are tiny (9×9 drawn at 2×) but very readable:

- Heart `#480000 #8d0000 #ad0300` with a single pink highlight pixel `#ffd6d6`
- Food (drumstick) `#4f301f #865538 #ce9957 #e2cfa7`
- Each icon has a "full", "half" and "empty" version. Half icons are the left half filled and the right half dark. Empty icons are a dark silhouette with a light rim.

---

## 2. Prompts

### A) Master prompt

> **Style:** Pixel-art game UI element for a warm fantasy survival game, crisp pixels, no anti-aliasing, no smooth gradients. Dark warm-brown panels (`#16100f`) inside bevelled bronze frames (`#89451e` → `#b35a26`) with orange title tabs (`#e47c02`). Slots are dark brown (`#352015`) and slightly sunken. Main buttons are orange, secondary ones bronze. The hover state glows yellow (`#ffdd20`). Headings in a pixel serif in gold (`#f2da9a`), body text in cream (`#e4dacf`). Light from the top-left: frames have a lighter top-left edge and a darker bottom-right edge. Matches 32×32 item icons that sit inside the slots.

Add **one** of these lines:

| Type | Add to the prompt |
|---|---|
| **Window or panel** | "A [inventory / crafting / storage / trading] window, [W×H], with a title tab reading '[title]', [N] slots in a grid, and [sections]." |
| **Button set** | "A button in four states side by side: normal, hover, pressed and disabled. [Orange main / bronze secondary / green confirm / red cancel], [with / without] an icon, text '[label]'." |
| **Bar** | "A [health / stamina / XP / progress] bar, [W] px wide, dark track, bronze rim, [colour] fill with a lighter top line, cream value text, shown full, half and nearly empty." |
| **HUD icon** | "A [9×9 / 16×16] HUD icon of a [heart / drumstick / shield / bubble / snowflake] in full, half and empty states, one clear shape with a single highlight pixel." |
| **Status-effect tile** | "A 24×24 square tile with a bronze frame and a dark centre holding one bright [symbol], for a [buff / debuff]." |
| **Slot** | "One empty item slot, [W×H], dark brown (#352015) and slightly sunken: a 1-px dark outline, a thin bronze rim lit top-left and shaded bottom-right, nothing inside." |
| **Frame piece** | "One empty [title tab / section box / well / tooltip box] of [W×H] for nine-slice stretching: bevelled corners, plain even straight edges, nothing written on it." |
| **Tabs** | "A row of [N] tabs with icons and labels: active (bright orange, joined to the panel), inactive (dark bronze), hover, disabled." |
| **Small controls** | "[Checkbox / toggle / radio / slider / dropdown / text input], all states, bronze rim, orange or green when on." |
| **Tooltip** | "A tooltip box: dark panel, thin bronze rim, gold item name, cream description, a coloured stat line (green for good, red for bad)." |
| **HUD layout** | "The full in-game HUD: hotbar of 9 slots at the bottom centre with a selected-slot glow, health and stamina bars, food buffs with timers, status icons." |

### B) Reference images to attach

| Prompt type | Attach |
|---|---|
| Windows, panels, inventory | `Pixel-Fantasy Inventory UI Reference.png`, `Crafting & Workstations UI Reference.png` |
| Buttons, tabs, controls | `Fantasy UI Controls Reference Sheet.png` |
| HUD, bars, status icons | `Pixel-Art Survival HUD Reference Panel.png` |
| Menus, navigation | `Pixel Art UI Navigation Reference.png` |
| Creature or taming screens | `Creature Taming UI Asset Reference.png` |
| Overall style | `Fantasy Pixel UI Reference Poster.png` |
| Every UI part (attached first) | `Voxel Fantasy Survival UI Suite.png`: Ronald's whole-game UI target (main menu, HUD, inventory, crafting, chests, taming, creatures, components) |
| Tiny HUD icons at low resolution | `01 Reference UI pieces.png` (for icon construction only, not colours) |

Decided by Ronald (2026-11-04): **always attach `Voxel Fantasy Survival UI Suite.png`** (`art.py ui` does). An A/B round with and without it came out the same for simple parts ("dont see any diff"); the sheet costs nothing and carries what words don't (corner bevels and rivets, the double line, tab icons, the hover glow). `--guide-only` (without it) is kept only for such comparisons.

What a first round missed against his sheet, so say it in every frame prompt: thick bronze frames with a double line (an outer and an inner bronze line with a dark groove between), bevelled corner blocks with a rivet; slots lighter and warmer than the panel; title tabs in orange with an icon; a soft gold glow around the hovered part.

### Style 1: the carved oak panel (Ronald's pick, 2026-11-23)

Ronald found the title menu "cheap" and picked style 1 of six generated directions (`~/Coding/chiacraft-archive/images/ui-redo/2026-11-23/menu_styles_sheet_v2.jpg`): the menu in a framed panel with an oak frame, bronze corner caps with rivets, a dark charcoal inside that lets the painting show faintly, bevelled plates with a bronze rim, orange with a gold rim on hover. He looked at six variants of it (1a-1f) and kept the original. It is the style for the whole game: the shared parts come first, and every screen follows from them.

- The parts were generated with Codex from a close-up of the chosen screen (attach it first; the first round without it came out flat), on an exact grid of 5 image pixels a UI pixel, and cut exactly by `notes/ui/style1/style1.py`, which writes `button*`, `button_secondary*`, `button_quit*` (the secondary's look: no red Quit), `panel`, `news`, `slot`, `slot_highlight` and `divider`.
- These parts replace kit 2's: don't re-run `kit2/install.py` over them. Kit 2 still draws the parts style 1 hasn't reached yet (tabs, wells, tooltips, the HUD).

### Yellow and gold only for state, never at rest (Ronald, 2026-11-04)

Round 2 of the parts was "too yellow everywhere". The rule:
- **At rest:** frames, slots, tabs and secondary buttons in the bronze and brown of the palette (`#89451e`–`#b35a26`, slots `#3b2e27`). Main buttons and the active title tab in the orange (`#e47c02`), a little desaturated and darker, never gold.
- **Gold (`#ffdd20`–`#f3ab1f`) only for state:** hover, selected, the active toggle or tick.
- **Bars:** red health, a muted orange stamina; no gold fills.
- **Text:** cream (`#e4dacf`); gold text only for a name or heading, sparingly.
- In prompts, ask for bronze and brown frames and say "no gold except where I name it". After cutting, `pxkit.tone()` pulls any leftover orange-yellow toward bronze (hue about 25 degrees, saturation x0.78, value x0.84); the states (`glow`) add the gold back where it belongs.

### Text (decided 2026-11-04)

Ronald found the smooth Alegreya "off and not well positioned". The text is now a **pixel font on the UI's own grid**:

- **Faces (Ronald picked "A bold"):** PixelOperator Bold (CC0, his sheet's pick) for Latin, with Pixeloid Sans Bold (OFL; Reserved Font Name, so never call our bake "Pixeloid") for Cyrillic, in the same weight. Both are in `mods/base/source/ui/fonts/` with their licences. Minecraftia (paid for commercial use) and VCR OSD Mono (private use only) can't ship.
- **One pixel:** one design pixel of the font is one font pixel, which is half a GUI pixel: the same pixel as the frames and the 32 px items. The engine already rounds text so a font pixel covers whole screen pixels. At GUI scale 2 and 4 everything lines up; at odd scales (3) a UI pixel is 1.5 screen pixels and text and frames can't both be exact.
- **Baking:** `font_bake.py --pixel FONT.ttf --also CYRILLIC.ttf` finds the design size (the smallest size with no grey pixels), keeps glyphs black-and-white, puts the second font's glyphs on the first one's baseline, and writes `pixel = 1`, `line`, `sharp = true` into `default.font`.
- **The line:** capitals plus the same room above them as below the baseline (descender + 1 pixel). PixelOperator: capitals 9, line 17. Text then sits centred in a button or tab with even padding.
- **Placement:** text starts on whole UI pixels; centre by whole pixels (round the leftover down); buttons and tabs at least 4 UI pixels of padding left and right of the text, title tabs 6 plus the icon; a one-pixel shadow down-right in `#28180c`.
- **Items in slots:** the 32 px icon centred in the slot on the pixel grid: 2 UI pixels inside the 36 px menu slot, 4 inside the HUD's 40 px hotbar slot (20 GUI px in the engine; its texture is made at 40, the selected frame at 44) (`icon = 16` in `slot.gui`), the same in the hotbar, inventory, chests, the pot and the rack. Every laid-out box's edges are rounded to the UI pixel (`Rect::snapped`, the font pixel), so centring never leaves half pixels.
- **Stack counts:** bottom right of the icon, the digits' ink 2 UI pixels inside the icon's right and bottom edges, the shadow 1; cream `#f4ead8`, shadow `#28180c`. Hotbar numbers: capitals' top 3 pixels inside the slot's top-left corner, 3 from its left.
- **Labels in buttons and tabs:** a one-line text's box is its ink (no lead-in, no spacing after the last letter, engine `text.rs`), centred by whole pixels, equal margins (one pixel more below or right where the leftover is odd), at least 4 UI pixels between the ink and the button's outer edge. Labels in panels left-aligned, 6 UI pixels (3 GUI) from the frame's inside.
- **Grids:** slots touch (36 px apart, the slot's dark groove is the gap); panels and their groups spaced in whole GUI pixels.
- **Measured, not eyeballed:** `mods/base/source/ui/placement_check.py SHOT.png --out marked.png` finds every slot and button in a shot at an even GUI scale and reports each item's offset (0,0 is right), each count's gap (2, 2) and each label's margins.
- **Colors:** titles and labels on orange `#fff4dc`, body cream `#e4dacf`, names and headings gold `#f2da9a`, dim `#c6a383`.
- **Never** draw letters in a generated part: the game writes all text.

### C) Checklist

- **HUD:** hotbar, selected-slot highlight, health, stamina and XP bars, hunger or food buffs, armour, air, temperature, status-effect tiles, crosshair, boss bar
- **Windows:** inventory, crafting, each workstation (furnace, anvil, smithing, loom, brewing), storage chest (small and large), trading, creature or taming, map, journal, skills
- **Controls:** buttons (4 colours × 4 states), tabs, checkboxes, toggles, radios, sliders, dropdowns, text input, number stepper, scroll bar
- **Menus:** title screen, pause, settings, world select, loading screen, death screen
- **Feedback:** tooltips, toasts or notifications, badges, item pickup popups, damage numbers
- **Text:** a heading font and a body font, in gold and cream

### D) Self-review before Ronald sees it

Ronald (2026-11-04), on the round-2 taming and stats sheet: "ui is much better, but ... there still lots of issues i find, text size and maybe like what's the border around one of the stats etc, it should be clever with ui". The UI worker catches these itself. Run every check on **every mock-up sheet and every in-game shot** before it goes out, and list in the report what the checks caught and what was fixed.

- **Type scale.** In the game, use one pixel font at one size. Headings differ by colour (gold) and placement, not by size. A sheet's own words use the same font in two sizes: titles at twice the UI size, everything else at the UI size. A third size needs a reason written next to it.
- **Alignment grid and fixed columns.** Lay every repeated value on a column: numbers right-aligned to one edge, counts centred, text left-aligned. A header lines up the way its values do. A unit gets its own column beside the number ("100 kg", "99 %"), so a unit or an arrow never shifts the numbers. Spell units out ("units", not "u").
- **Every border and colour means something.** A frame or glow is a state the game really shows: hover is the game's button_hover on the control itself; selected or active is button_pressed or the pressed tab. A still sheet doesn't show a hover box around a whole row. Green means "after this change" or good, red means bad or danger, and the wild-point colours mean quality. Name a colour's meaning in a legend line, or don't use the colour.
- **Nothing outside its panel and nothing clipped.** Every label, number and widget sits inside its panel's inner edge, with at least 3 UI px of air beside its neighbours. A label never runs into the next column.
- **Contrast of text on bars.** Don't write a value on a bar's fill: it sits beside the bar, in its column. Text on a coloured fill (a button, a chip) uses that part's text colour (`#fff4dc` on orange).
- **Consistent units and icon sizes.** One icon size per job: 16 UI px in a row beside text, 20 in a button or tab, 24 for a heading or badge. The same stat has the same icon, name and unit everywhere: the animal screen, the stats screen and the HUD.
- **Labels centred on their capitals, measured.** Ronald (2026-11-04): "the chip text still seems wrong, like not centered". A label's capitals sit in the flat face, between the 1 px light edge and the 2 px shade, with the same room above and below, and descenders (g, p, y) stay inside the face. PixelOperator: capitals 9 px, x-height 7, descenders 2. So a part's face must leave an even number of rows round 9: buttons and tabs 22 px tall (4 above, 4 below), chips and keys 20 px (3 and 3). `kit2.py` refuses a face with an odd leftover (`centre_caps`), and measures every drawn label (`check_label`). The old mock-up font's offset put every label 1 px high: a chip had 1 px above its capitals and 4 below. Check it at even GUI scales, in the game too, with `placement_check.py`.
- **Panel value against the reference, measured.** Ronald (2026-11-04), on kit 2 in the game: "Something is weird with the dark part of the UI ... It looks a bit too dark". Sample the median colour of a part's face in a shot, and of the same kind of part in `Voxel Fantasy Survival UI Suite.png`. Compare value (V) and warmth (hue, saturation). Measured on his sheet: window insides V 0.10-0.12, nearly neutral charcoal (about #1e1a19); slots and insets V 0.20, a soft warm grey (about #342c28, saturation about 0.25) that is lighter than the panel, never darker. Kit 2 had a redder near-black panel (#16100f, V 0.09), a saturated brown slot (#352015, saturation 0.43) and wells darker than the panel. A part more than 0.03 V off the sheet, or wells darker than their panel, is a finding.
- **Disabled controls say why.** A greyed-out button carries a line or tooltip with the reason ("Ride: needs a saddle on it"), or it isn't shown.
- **At his sizes.** Check every screen at Ronald's Mac (1728x1117 pt, 3456x2234 px: GUI scale 6) and at 1280x720 (GUI scale 2), using `notes/ui/audit/ui_fit.txt` on a tester's run. Nothing should be cut off, and every window should keep a margin round it.

How to check:
- **Measured:** mock-ups are drawn by a script that records every box it draws. Its `lint()` reports anything outside the panel, overlapping, closer than 3 px, off its column's edge, or wider than its column; see `notes/ui/mock/mock_taming_v3.py`. It must report 0 problems. For in-game shots, use `mods/base/source/ui/placement_check.py` (slots, counts, label margins).
- **By eye, after the measuring:** zoom to 1:1 and look for what a measure can't see: a colour with no meaning, a gap that reads as a missing value, a crop that cut into a figure, the same thing named two ways. Fix it, then run the measure again.

### E) Pixel craft: how UI like this is made

Ronald (2026-11-04), on the round-2 stance buttons: "ui better but still seems weird off in places maybe read up on how one does ui like this as well". What was off: kit icons are 48 px generated pictures with about 1,000 colours each, and the mock-up shrank them to 20 px (2.4x) and 16 px. Their pixels came out uneven, their weights differed, and the crossed swords meant "Passive". On top of that, button text was centred on the font's line box, and the shadow was heavy. The rules, measured where possible:

1. **One pixel grid.** A UI pixel is the font's pixel and the 32 px items' pixel (half a GUI pixel). Every part is authored at the size it is shown and scaled only by whole numbers. Never show art at a fraction of its size: a 48 px icon at 20 px makes pixels of different sizes. The GUI scale is even (2, 4, 6), because a UI pixel is half a GUI pixel: at 3 a frame pixel is 1.5 screen px and the text no longer matches it, and at 1 a UI pixel is half a screen pixel, so the text (never drawn under a screen pixel a font pixel) comes out twice the size of its boxes (Playtest B9, 11-04: the animal screen's labels over each other). 1 is only a fallback for a window too small for 2 (under 800 x 640). The engine keeps to this (`settings::applied_gui_scale`, claude/gui-scale-even 361bea1f; `settings::even_scale`, b94cf94b; Researcher round 13; Minecraft does the same for its half-size font, https://minecraft.wiki/w/Options#Video_settings). Textures use nearest filtering with no mipmaps (https://pixnote.net/en/learn/game-assets).
2. **Icons: one canvas, one weight.** Every UI icon is 16 x 16 with a 1 px dark outline (`#28180c`), light from the top left, at most 3 tones per material from one shared palette, and a silhouette that reads at 1x on dark and light ground (https://pixnote.net/en/learn/game-assets, https://photonstorm.com/art/tutorials-art/16x16-pixel-art-tutorial). They are drawn as masks in `notes/ui/kit2/kit2.py`, where code adds the outline and the light the same way for every icon. Don't put a generated picture in place of a UI icon; generated art is for items and portraits.
3. **Icons mean one thing.** Corners: every part steps its corners 3 px on the pixel grid (Ronald, 2026-11-04: "i like c with 3px"), drawn as a 9-slice corner so every size stays crisp. Stances: Passive is a white flag, Neutral a shield, Attack a sword, Assist a target ("attacks what you attack"). Orders: Follow is a paw, Stay a hand, Wander four-way arrows, Guard a house. Colour adds to the shape and never carries the meaning alone. (These conventions are practice, not from one source.)
4. **Small borders stay simple.** A 9-slice's border is the drawn corner exactly (https://create.roblox.com/docs/ui/9-slice). Edges are flat or repeat evenly. At button size, use an outline, one light edge and one shade (2 px). The double line, groove and rivets belong only to panels' 6 px frames; below about 4 px they turn to mud.
5. **Text centres on its capitals.** Centre the capitals (9 px in PixelOperator) inside the button's whole inside, shade included, not the line box: the descender room makes text sit high. In a grid of buttons, icons and labels start at the same left padding (6 px), so icons line up in a column. The gap between icon and text is always 4 px.
6. **The shadow is light.** Use 1 px straight down, in the surface's own dark tone: the button's shade colour on a button, `#0e0a09` on a panel. Don't use a near-black down-right shadow, and don't shadow text on light ground.
7. **A 2 px rhythm.** Positions and sizes are even UI px (whole GUI px); spacings come from 2, 4, 6, 8, 12, 16. `components_sheet.py` refuses a part placed off the rhythm.
8. **States differ in shape, not only in brightness.** Normal is raised. Hover is a brighter face with a gold outline. Pressed is sunk (the dark edge on top, content 1 px down). A selected toggle is the main orange, pressed, so it can't be mistaken for hover. Disabled is flat grey-brown and says why. All states keep one footprint.
9. **Contrast.** Light text on dark faces, or dark text (`#3a2416`) on light keys. Never put brown on bronze, and never write text on a bar's fill.

Where these come from: the pages linked above (fetched 2026-11-04), plus common practice in Stardew Valley, Terraria and Celeste (crisp 9-sliced frames, a 1 px darker shadow, whole-pixel layout); I didn't find a primary source for those games' rules. Slynyrd's palettes (https://www.slynyrd.com/blog/2018/1/10/pixelblog-1-color-palettes): shift the hue toward warm as a ramp brightens.

**Round parts** (Ronald, 11-05, on the round minimap: "Yes, as drawn"): a round frame is the kit's panel frame made round, drawn in code (`notes/ui/kit2/minimap_ring.py`): outline, a 3 px bronze band whose outer row is lit where it faces the top left and inner row the other way round, the groove, the inner bronze line, an inner outline; plates on it are round kit faces with a 1 px bronze rim, the main orange marking the one that matters (north). The game writes letters on them (translated). The darker kit bronze was kept over his sheet's brighter, thicker ring.

The shared parts are drawn by `notes/ui/kit2/kit2.py`, and `notes/ui/kit2/components_sheet.py` draws the sheet of them at 1x and 2x. Ronald sees a new or changed part on that sheet before any screen uses it.
