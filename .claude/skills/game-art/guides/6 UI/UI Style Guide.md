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

### Yellow and gold only for state, never at rest (Ronald, 2026-11-04)

Round 2 of the parts was "too yellow everywhere". The rule:
- **At rest:** frames, slots, tabs and secondary buttons in the bronze and brown of the palette (`#89451e`–`#b35a26`, slots `#3b2e27`). Main buttons and the active title tab in the orange (`#e47c02`), a little desaturated and darker, never gold.
- **Gold (`#ffdd20`–`#f3ab1f`) only for state:** hover, selected, the active toggle or tick.
- **Bars:** red health, a muted orange stamina; no gold fills.
- **Text:** cream (`#e4dacf`); gold text only for a name or heading, sparingly.
- In prompts, ask for bronze and brown frames and say "no gold except where I name it". After cutting, `pxkit.tone()` pulls any leftover orange-yellow toward bronze (hue about 25 degrees, saturation x0.78, value x0.84); the states (`glow`) add the gold back where it belongs.

### Text (decided 2026-11-04)

Ronald found the smooth Alegreya "off and not well positioned". The text is now a **pixel font on the UI's own grid**:

- **Faces:** PixelOperator (CC0, his sheet's pick) for Latin, with Pixeloid Sans (OFL; Reserved Font Name, so never call our bake "Pixeloid") for Cyrillic, in the same weight. Both are in `mods/base/source/ui/fonts/` with their licences. Minecraftia (paid for commercial use) and VCR OSD Mono (private use only) can't ship.
- **One pixel:** one design pixel of the font is one font pixel, which is half a GUI pixel: the same pixel as the frames and the 32 px items. The engine already rounds text so a font pixel covers whole screen pixels. At GUI scale 2 and 4 everything lines up; at odd scales (3) a UI pixel is 1.5 screen pixels and text and frames can't both be exact.
- **Baking:** `font_bake.py --pixel FONT.ttf --also CYRILLIC.ttf` finds the design size (the smallest size with no grey pixels), keeps glyphs black-and-white, puts the second font's glyphs on the first one's baseline, and writes `pixel = 1`, `line`, `sharp = true` into `default.font`.
- **The line:** capitals plus the same room above them as below the baseline (descender + 1 pixel). PixelOperator: capitals 9, line 17. Text then sits centred in a button or tab with even padding.
- **Placement:** text starts on whole UI pixels; centre by whole pixels (round the leftover down); buttons and tabs at least 4 UI pixels of padding left and right of the text, title tabs 6 plus the icon; a one-pixel shadow down-right in `#28180c`.
- **Colors:** titles and labels on orange `#fff4dc`, body cream `#e4dacf`, names and headings gold `#f2da9a`, dim `#c6a383`.
- **Never** draw letters in a generated part: the game writes all text.

### C) Checklist

- **HUD:** hotbar, selected-slot highlight, health, stamina and XP bars, hunger or food buffs, armour, air, temperature, status-effect tiles, crosshair, boss bar
- **Windows:** inventory, crafting, each workstation (furnace, anvil, smithing, loom, brewing), storage chest (small and large), trading, creature or taming, map, journal, skills
- **Controls:** buttons (4 colours × 4 states), tabs, checkboxes, toggles, radios, sliders, dropdowns, text input, number stepper, scroll bar
- **Menus:** title screen, pause, settings, world select, loading screen, death screen
- **Feedback:** tooltips, toasts or notifications, badges, item pickup popups, damage numbers
- **Text:** a heading font and a body font, in gold and cream
