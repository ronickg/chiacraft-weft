# Brown UI palettes: research and ten variants

Ronald, 2026-11-04, on kit 2 with his sheet's tones: "I'm not a big fan of how the UI looks like.
Maybe if I look at Valheim or some other color palettes, search online what's the ideal one ...
I do kind of want that brown sort of look ... give me like 10 variants and then I can choose."

Sheet: `~/Coding/chiacraft-archive/images/ui-redo/2026-11-04/ui-variants/ui_variants_10.png`
(drawn by `notes/ui/kit2/variants.py` on real in-game shots: kit 2 draws exact colours, so each
part's colour is swapped for the variant's, and a variant can lay a pixel texture on its panel).

## What the games do

Only Vintage Story publishes its UI colours (in its own source). The rest are read from
screenshots and descriptions: treat them as estimates, and sample real screenshots before any
value is final.

- **Vintage Story** (source: `GuiStyle.cs`,
  https://apidocs.vintagestory.at/api/Vintagestory.API.Client.GuiStyle.html,
  https://github.com/anegostudios/vsapi/blob/master/Client/UI/Elements/GuiStyle.cs):
  - Panel `#403529`, a warm brown of value about 0.25, not charcoal.
  - The border is a black shadow at 30%, not a coloured frame.
  - Slots `#845C43` (wood) with a cream back `#FFE2C2`: lighter than the panel.
  - Buttons `#453424`, text `#E0CFBB`, hover text `#C58948`. Default text `#E9DDCE`; dark text for parchment `#5A4530`.
  - Parchment `#EDCE98`. The selected slot is cyan `#62C5DB`, the one cool colour.
  - Why it reads: a value ladder (dark panel, mid wood, light cream), cream text at about 9:1, and one cool selection colour in an all-warm field.
- **Valheim** (named styles ValheimBeige, ValheimOrange and ValheimYellow, plus a woodpanel style, in the modding API:
  https://valheim-modding.github.io/Jotunn/api/Jotunn.Managers.GUIManager.html; players on it:
  https://steamcommunity.com/app/892970/discussions/2/767437664460950889/):
  - A near-black brown interior (about `#1B1511`) inside a rough mid-brown wooden frame.
  - Dark slots, a little lighter than the panel.
  - Beige text, with orange and yellow for titles, hover and selection.
  - The wood grain is in the frame; the inside stays calm.
- **Stardew Valley:**
  - A parchment panel (about `#F2C27B`) in a thick dark-brown frame.
  - Tan slots, dark red-brown text (dark on light, about 8:1), and a red hover outline.
- **Terraria** (https://terraria.wiki.gg/wiki/Inventory/bg):
  - Blue panels, not brown.
  - Its lesson is that one hue family with clear value steps stays calm, while white outlined text and rarity colours carry the meaning.
- **Hytale** (https://hytale.com/news/2025/12/an-introduction-to-making-models-for-hytale,
  https://hytalemodding.dev/en/docs/official-documentation/custom-ui/common-styling):
  - Dark slate-brown panels with a thin warm gold trim.
  - Slots a little lighter, cream text, gold accents.
- **Minecraft Dungeons:** a very dark brown-black panel with a thin gold border, cream text, and rarity as the accent.
- **Dwarf Fortress** (Steam) (https://progameguides.com/dwarf-fortress/dwarf-fortress-steam-ui-and-symbols-explained/):
  - Grey-brown stone, thin square frames, and bright category-coloured text.
  - Density comes from value and colour, not decoration.
- **Kenshi, Grounded, Enshrouded:** no sources; impressions only.
  - Kenshi: rusty brown with a dirty orange accent.
  - Grounded: light, rounded, earthy, with a yellow-gold or green accent.
  - Enshrouded: dark charcoal-brown with thin antique gold.

## What makes a brown UI read well (and what makes it muddy)

1. **Value steps matter more than hue.** The panel is darkest. Slots and wells are a clear step lighter, never darker. The frame or trim is the lightest or the most saturated part.
2. **Keep the big areas low in saturation** (under about 25%). Put saturation in the small things: trim, hover, the selected item. A large area of strong orange turns loud.
3. **One warm-cool contrast.** An all-brown screen goes muddy. Vintage Story's cyan selection, Valheim's orange and yellow, and Stardew's red-brown on cream each give one clear separation.
4. **Parchment suits pages people read** (books, recipes, tooltips), with a dark frame round it. Dark panels are easier on the eye for slots and windows that stay open.
5. **Texture stays low-contrast** (one or two value steps), and is best on frames. Grain inside a text area makes brown muddy fast.
6. **Cream text beats white on brown** (`#E0CFBB` to `#E9DDCE`). Text should not be lighter than the accent, or the accent stops reading.

## The ten variants (in the sheet's order; the first three are my picks)

| # | name | panel | frame | slot | main button | accent | text | texture |
|---|---|---|---|---|---|---|---|---|
| 1 | Vintage Story: warm brown, wood slots, cyan pick | #403529 | #2b2219 (shadow line) | #6e4f3b | #5a4330 | #62c5db | #e9ddce | flat |
| 2 | Valheim: quiet dark wood, orange accent | #1f1712 | #6e4c30 | #33281f | #7a4a24 | #ffa13c | #e8d6b0 | flat |
| 3 | Hytale: warm wood and gold | #4a3322 | #d1a04c | #33231a | #c98a32 | #fff0b0 | #fbeed2 | wood |
| 4 | Minecraft Dungeons: chunky stone and gold | #2f2a25 | #c9962e | #221e1a | #d7a536 | #ffe27a | #f5eedf | stone |
| 5 | Stardew: warm wood and parchment | #f1d39a | #b0642c | #d8a861 | #c2552b | #e8402a | #4a2614 | parchment |
| 6 | Dwarf Fortress: red-brown stone, gem green | #4a3428 | #b0843e | #5e4434 | #9a6a2a | #4fd07e | #f2e4c6 | stone |
| 7 | Terraria: leather tan and sky | #6a4a2f | #3d2716 | #4c341f | #a8692c | #6cc4ff | #fff4dc | leather |
| 8 | Enshrouded: near-black leather, brass, teal | #1b1613 | #9c7c48 | #2e2620 | #4a3a2a | #5fd6cc | #e6d8bf | leather |
| 9 | Kenshi: rust and bone, iron trim | #4a2c20 | #6d655c | #3a231a | #a04a28 | #e0603c | #e4d9c2 | flat |
| 10 | Grounded: cardboard and bold outline | #c7a273 | #7a4e28 | #a98454 | #5f8f2c | #a8dc4a | #26170b | parchment |

The checklist on each (contrast, WCAG ratio): text on its panel is at least 7.3 in every variant, and dim text at least 4.0. Button text is at least 3.6, slots against their panel at least 1.2. One finding: Vintage Story's frame is only 1.3 against its panel, because its border is a shadow line by design. Nothing is clipped: the screens are the same layouts as kit 2's checked shots, with only the colours changed.

Not shown: a variant's frame and inset shapes. Every variant here keeps kit 2's 3 px stepped corners and frame widths. Once Ronald picks, the palette goes into `kit2.py` (and a texture, if the pick has one, into its panels), then into the game's textures.
