# Research round 13: a craft guide for pixel-art game UI

Researcher, 2026-11-04, for Planner and the UI session. Ronald: the UI "still seems weird off in places maybe read up on how one does ui like this".

**What was read:** `docs/ui.md`, the game-art UI Style Guide (`.claude/skills/game-art/guides/6 UI/UI Style Guide.md`), `crates/engine/src/ui/{mod,render,text,font}.rs`, `crates/engine/assets/shaders/ui.wgsl`, `settings.rs`/`app.rs` for the GUI scale, `mods/base/assets/fonts/default.font`, `font_bake.py`, and the UI redo's recent commits (`818614e8`, `7efecb0d`). I looked at the audit shots (`~/Coding/chiacraft-archive/images/ui-redo/2026-11-04/check_01_hud_day.png`, `check_03_inventory.png`, `check_08_cooking_pot.png`, `check_23_settings.png`).

**Web research** was done by two sub-agents, with source links and tags for how sure each claim is; full notes in the shared notes folder: `pixel-ui-craft.md` and `game-uis.md`. Every claim below carries a link or says *inference* (my or the agent's reasoning from sourced facts) or *unconfirmed*. Several pages blocked us (stardewvalleywiki.com 403, gameuidatabase.com 403, DeepWiki 429, Celeste's fandom wiki 402), so several game details rest on search snippets or memory and are marked.

## The short answer

1. **The UI redo has already done most of what the craft says.** One pixel size for text, frames and items (a UI pixel is half a GUI pixel); layout snapped to that pixel (`Rect::snapped`); glyph origins rounded; a sharp-bilinear sampler for pictures and nearest for the pixel font; 9-slice borders that clamp; the text rules in the style guide (centre by ink, whole pixels, 1-pixel shadow, 4-pixel button padding). I checked each in the code.
2. **The one big hole is odd GUI scales.** The engine lets the GUI scale be any whole number, and the half-pixel UI only lines up at even ones. At scale 3 a UI pixel is 1.5 screen pixels and the engine rounds text to 2 but not the frames; at 5 or 9 it is the same. The style guide itself says so ("at odd scales (3) ... text and frames can't both be exact"). Minecraft does the same fix for its half-size font: the Unicode font forces an even scale ([minecraft.wiki/w/Options](https://minecraft.wiki/w/Options) and the issue it cites). **Rule 2 below.** On dev the automatic scale at Ronald's 3456×2234 is 9; the UI branch's `GUI_MIN` change makes it 6.
3. **Smaller gaps** (details in the rule table): a pressed button does not shift its label (the art does the darkening, the text stays), a selected tab uses the *pressed* button art rather than a tab that joins the panel, numbers in HUD bars are right-aligned rather than centred in the shots I saw, and the settings screen in `check_23` shows two buttons whose labels render with a magenta/white outline (it looks like a debug or focus marker in the test shot, not a design; the UI session should confirm).

## What is already right, and how I know

| craft rule | Weft | where |
|---|---|---|
| one integer scale, no fractional GUI scale | yes | `settings.rs:18` `max_gui_scale`, `app.rs:584`; the scale is counted in physical pixels (the engine reads no OS scale factor), so Retina and 4K just get a larger number |
| minimum GUI size so menus fit | yes on the UI branch | `claude/ui-ci-fixes` `GUI_MIN 400x320` (`22960101`); dev is 320×240 as vanilla |
| everything snapped to whole pixels | yes | `Rect::snapped(text_unit)` for every box (`ui/mod.rs:178, 680`), glyph origins `.round()` (`ui/render.rs:680`), stack counts and slot numbers rounded (`render.rs:919, 938`) |
| nearest or sharp sampling | yes | `sharp_uv` in `ui.wgsl:53` (edges blended over about one screen pixel, crisp at integer scales); the baked pixel font is `sharp = true`, sampled nearest (`ui.wgsl` font branch) |
| nine-slice that keeps corners | yes | `nine_sliced` (`render.rs:567`): corners fixed, border clamped to half the box so a small box can't overlap them; `.gui` has `scaling`, `border`, `tile` (with `turn = random` so a big background shows no repeat) |
| pixel font used at its design size | yes | `default.font` is baked from PixelOperator-Bold (CC0) and Pixeloid Sans (OFL) with `pixel = 1`, `sharp = true` (`font_bake.py --pixel`) |
| text on the same pixel as art | yes | style guide "One pixel": a font pixel is half a GUI pixel, the same as the frames and the 32 px items |
| centre by ink, round leftover consistently | yes | one-line text box is its ink (`text.rs`), leftover floored (`text.rs:907`); capitals' height stored in the `.font` (`caps = 9`) |
| tabular digits | yes | `mono()` and `{mono}` put every glyph in a digit-wide cell (`docs/ui.md`) |
| text shadow 1 px down-right | yes | guide: `#28180c`, 1 UI pixel; the engine's shadow offset steps by `font.pixel` (`render.rs:665`) |
| slots touch, item inside | yes | 36 px slots touch, 32 px icon 2 UI pixels inside (guide "Grids", "Items in slots") |
| all states of a control | yes | `button`, `_hover`, `_pressed`, `_disabled` (`docs/ui.md` "Looks") |
| no half-pixel layout | yes | `Rect::snapped` leaves "no half pixels" (commit `818614e8`), measured by `placement_check.py` |

## The ten rules that matter most

Each has a do, a don't and where we stand (✓ followed, ~ partly, ✗ not yet). Sources follow the table; *inference* means a rule that follows from sourced facts but that no page states.

**1. One pixel size for everything.**
- **Do:** pick one integer number of screen pixels per art pixel and draw frames, icons, text and the world's HUD pieces with it. Celeste draws 320×180 and scales it, so 1080p is exactly 6× ([Steam thread](https://steamcommunity.com/app/504230/discussions/0/3832043350345268297)); Godot's `integer` stretch rounds down and adds bars instead of distorting ([docs](https://docs.godotengine.org/en/stable/tutorials/rendering/multiple_resolutions.html)); Unity's Pixel Perfect Camera starts from a reference resolution ([docs](https://docs.unity3d.com/Packages/com.unity.2d.pixel-perfect@5.0/manual/index.html)).
- **Don't:** mix pixel sizes ("mixels"): text finer than the art, or a 2× icon in a 1× frame; players read that as "off" even when they can't say why ([Flowlab thread](https://community.flowlab.io/t/mini-rant-stop-using-mixels/38287)).
- **Weft ✓** (half-GUI-pixel UI pixel for frames, text and items), **but only at even scales** (rule 2).

**2. Allow only even GUI scales when the pixel is half a GUI pixel.**
- **Do:** make the automatic scale the largest even number that fits, and let the settings button step by 2. Minecraft forces even scales when its half-size Unicode font is on ([minecraft.wiki](https://minecraft.wiki/w/Options); the bug is MC-41685, page body unread).
- **Don't:** let text round to a whole screen pixel on its own: `text_unit = round(scale·0.5)/scale` (`app.rs:2548`) makes the font pixel 2 screen pixels at scale 3 (and 3 at 5, 5 at 9) while a frame pixel is 1.5, 2.5, 4.5.
- **Weft ✗:** `max_gui_scale` returns any integer and the settings button steps by 1 (`screens.rs:641`). On dev the auto scale is odd at many sizes (1280×720 gives 3, Retina fullscreen 9). The UI branch's 400×320 minimum gives 6 at 3456×2234 by luck, not by rule.

**3. Snap every box and glyph origin to the pixel; centre by whole pixels.**
- **Do:** snap layout edges to the art pixel, and when the leftover is odd, round it the same way every time (here: floor, then one pixel more below or right). Godot defaults to `snap_controls_to_pixels` for the same reason ([docs](https://docs.godotengine.org/en/stable/classes/class_projectsettings.html)); GameMaker guides say fractional scaling blurs even with filtering off ([bugnet](https://bugnet.io/blog/how-to-fix-gamemaker-texture-filtering-blurs-pixel-art)).
- **Don't:** animate positions in fractions of a pixel (a Godot 3.5 report: snapping fixed blurry labels but made scale animations choppy, [issue](https://github.com/godotengine/godot/issues/64294)).
- **Weft ✓** for layout; **~** for animated panels (`animation` in `.gui` moves the part by `position`/`scale` in float: round it per frame to the UI pixel, or the open animation will shimmer).

**4. Nearest filtering for art and the pixel font; sharp-bilinear for the leftovers.**
- **Do:** nearest for pixel art at integer scales; "sharp bilinear" (nearest up to the integer factor, then a one-pixel blend) when a size can't be whole ([Godot Q&A](https://godotengine.org/qa/72525/how-to-apply-sharp-bilinear-filter-fix-pixel-wobble-pixel-art)). Stardew filters linearly and looks soft; its True Zoom mod switches to point filtering only at clean ratios ([mod page](https://www.curseforge.com/stardewvalley/mods/true-zoom)).
- **Don't:** run a baked antialiased font next to hard art: `layer 1` fonts are sampled linearly (`ui.wgsl`); a bitmap font stays sharp only at its native size or whole multiples ([Godot forum](https://forum.godotengine.org/t/why-are-my-labels-blurry/125633), [m5x7 page](https://managore.itch.io/m5x7): "use font size 16, 32, 48").
- **Weft ✓** (`sharp_uv`, `sharp = true`). A pack's smooth font still works, which is fine as long as the base game ships the sharp one.

**5. Nine-slice: the border contains the whole frame, and the centre tiles, never stretches.**
- **Do:** make the slice border at least as wide as the frame, its highlight and its shadow together, so no 1-pixel line is cut. Corners never scale; edges stretch on one axis; Godot's Stretch mode "may cause the texture to be distorted" and Tile does not, but needs a seamless texture ([NinePatchRect](https://docs.godotengine.org/en/stable/classes/class_ninepatchrect.html)); Minecraft's `nine_slice` tiles the centre by default and its borders are in GUI pixels ([wiki](https://minecraft.wiki/w/Resource_pack)); Stardew's frames are assembled from pieces with stretch coordinates listed ([modding wiki](https://stardewmodding.wiki.gg/wiki/Tutorial:_Making_a_UI_Recolor)).
- **Don't:** stretch a dithered or patterned centre by a non-whole factor (*inference*: it breaks the dither's pixel size), or make a panel smaller than twice its border plus one pixel.
- **Weft ~:** corners fixed and clamped ✓, tile mode with random turns ✓. Check: `border` in a `.gui` file is in GUI pixels, so a 3-UI-pixel frame needs `border = 1.5`; confirm the parser takes halves and that no kit part is cut off the half-pixel line. A texture with no `.gui` is "simply stretched" (`docs/ui.md`): for pixel art, make `tile` or `nine_slice` the default and stretch opt-in.

**6. Pixel fonts: design size only, baseline and cap height known, shadow 1 pixel in a darker shade.**
- **Do:** use the font at 1× its design size (or whole multiples), know its cap height and baseline, shadow = same hue darker, 1 pixel down-right. Minecraft's shadow is the glyph colour divided by 4, offset 1 pixel, with the XP number outlined instead ([minecraft.wiki/w/Font](https://minecraft.wiki/w/Font)). Free fonts: m5x7 (CC0), Monogram (CC0, 5 px wide, fixed), Pixel Operator (CC0), Silver (CC BY 4.0, attribution) ([m5x7](https://managore.itch.io/m5x7), [Monogram](https://datagoblin.itch.io/monogram), [Pixel Operator](https://www.dafont.com/pixel-operator.font), [Silver](https://poppyworks.itch.io/silver)).
- **Don't:** ship a vector font scaled by the GUI scale, or a paid one: Minecraftia and VCR OSD Mono can't ship, as the style guide records.
- **Weft ✓.** One thing to check: Celeste and Vintage Story use smooth fonts (Celeste: Renogare loaded as a size-64 BMFont per the Everest wiki, licence unverified; Vintage Story draws its GUI with Cairo: *snippet-level sources*), so "smooth text on pixel art" is not unheard of, but our sheet is pixel text, and the rule holds for the look Ronald picked.

**7. Centre text in controls by its ink and decide the odd pixel once.**
- **Do:** centre the capital-height box (baseline to cap top), not the line box with its descender space; with an odd leftover, give the extra pixel to the bottom/right everywhere; at least 4 UI pixels of padding left and right. Type designers who hint for screens mostly round up, and consistency matters more than which way ([TypeDrawers](https://typedrawers.com/discussion/230/preferences-for-vertical-center-on-even-pixel-height-glyphs)); the cap-height rule itself is *inference*, no page states it.
- **Don't:** centre on the line box, or let descender words ("Settings", "Quit") sit higher than all-caps ones.
- **Weft ✓** (ink box, `caps` in the `.font`, floored leftover, 4-pixel padding in the guide). Check that lowercase with descenders ("Singleplayer") looks level next to "Done" in a shot.

**8. Icons: one grid, one outline rule, one light direction, silhouette first.**
- **Do:** one icon size per use (16×16 items; 9×9 HUD icons), a dark outline in a darker shade of the icon's colour (not pure black), light from the top left, at most one rim and one highlight. "An outline should always increase contrast" ([Lospec](https://lospec.com/articles/pixel-art-outlines-part-2-using-color)); judge the silhouette small ([Wayline](https://www.wayline.io/learn/game-ui-art/2), community). Minecraft's slot is 18×18 with the 16×16 item inside and the border outside it ([Forge forum](https://forums.minecraftforge.net/topic/34408-gui-container-slots/)).
- **Don't:** mix item art at 16 and 32 pixels in one inventory without a rule (*inference*), or light some icons from the right.
- **Weft ✓ by rule** (guide: 32 px items inside 36 px slots, light from the top left, tiny HUD icons 9×9 at 2×). Art's batches check the icons; I did not audit each.

**9. Spacing: one base unit; edges align; groups gap in whole GUI pixels.**
- **Do:** pick one unit (here the UI pixel, or 2 UI pixels) and make the border width, slot gap, padding and group gaps multiples of it, and write the numbers in the style guide. Minecraft's slots touch (18-pixel cell, border inside it); ours touch at 36. Vintage Story has a debug key that outlines every GUI element's bounds ([wiki](https://wiki.vintagestory.at/index.php/Modding:GUIs/en)): a layout check we could copy.
- **Don't:** rely on the eye; the UI redo already measures (`placement_check.py`).
- **Weft ~:** padding rules exist for buttons (4) and labels (6) but not as one table for every screen. Ronald's "off in places" is most likely the screens built before the rules (audit `notes/ui/audit`): a **layout-outline gizmo** that draws every box and shows off-grid edges would find them. (There is one for HUD: `gizmo.rs` has `"cubes"`, `"hitboxes"`; a UI one is new.)

**10. Same footprint for every state; show the pressed state by shifting, tabs by joining.**
- **Do:** derive hover and pressed from the same base sprite so the footprint stays; a pressed button's label moves down 1 pixel (a library default, [pixel_ui](https://pub.dev/packages/pixel_ui); our own guide says "pressed (darker, 1 px lower)"); the selected tab is joined to the panel below it, with no border line between.
- **Don't:** change the button's outer size between states, or make the selected tab look like a pressed button.
- **Weft ~:** the art states exist, but in `render.rs:1025-1031` the label only changes colour for hover and disabled (no pressed shift), and a selected button uses its `_pressed` texture, so tabs are only joined if the `category_pressed` art is drawn that way. Check both in a shot.

## HiDPI, odd window sizes and Retina: how others handle them

- **Count the scale in physical pixels, draw the framebuffer at physical size.** SDL3: macOS and iOS use window coordinates and you opt in to more pixels; Windows and Android use physical pixels ([README-highdpi](https://wiki.libsdl.org/SDL3/README-highdpi)); GLFW: "never pass window size to `glViewport`", use content scale for UI size ([window guide](https://www.glfw.org/docs/latest/window_guide.html)). Weft already counts physical pixels, so GUI scale 1 on a 2× screen is a 1:1 pixel game and looks tiny: the auto scale, not the setting's number, is what players see.
- **Odd window sizes:** Minecraft's rule is `max(1, min(floor(width/320), floor(height/240)))` and scale *n* needs a 320n×240n window ([wiki](https://minecraft.wiki/w/Options)); the scaled GUI size is `window/scale`, rarely whole, so anchor to whole GUI pixels and leave a strip (*inference*). Mojang's own auto scale jumps near thresholds when the window height shrinks a little ([MC-95855](https://bugs.mojang.com/browse/MC-95855)), which is the argument for hysteresis or for fewer, even steps.
- **Windows 125/150%:** legacy apps are rendered at 100% and upscaled; the fix is to declare DPI awareness in the manifest ([issue](https://gitlab.com/armagetronad/armagetronad/-/issues/33)). Worth checking on the Windows build if it ships one.
- **Separate world and UI scales:** Stardew has Zoom and UI Scale (UI range 75–150% in some sources, up to 200% in others, *unconfirmed*); Terraria has Zoom 100–200% and UI Scale 50–200% ([Terraria wiki](https://terraria.wiki.gg/wiki/Settings)). Stardew users report parts cut off below about 85%, and Vintage Story at its maximum scale (16) cuts off the buttons ([forum](https://www.vintagestory.at/forums/topic/5100-gui-scaling-cuts-off-menu-buttons/)): both are the "menus must always fit" problem the UI branch's `GUI_MIN` solves.

## What the six games do

Pictures: the links open a page that has them; I could not view the images, so each line says what the page says they show.

| game | what we can say | where to look |
|---|---|---|
| **Minecraft** (reference) | integer scale, 320×240 base; font height 8, ascent 7; shadow = RGB/4, 1 px; nine-slice mcmeta values: button 200×20 border 3, text field border 1, tooltip 100×100 border 9 ([misode/mcmeta](https://github.com/misode/mcmeta)); slot 18×18, hotbar 182×22, heart 9×9 (*from memory, unconfirmed*) | [Hotbar page: `HUD.png` shows all status bars](https://minecraft.wiki/w/Hotbar) |
| **Stardew Valley** | separate zoom and UI scale; frames built from pieces, backgrounds a small square stretched; UI in the `Cursors` sheet and `Maps/MenuTiles`; bitmap font about 8 px wide (forum, *unconfirmed*); linear filtering soft at some zooms | [the UI recolour tutorial lists the sheet coordinates and stretch lines, with the dialogue box and a labelled inventory](https://stardewmodding.wiki.gg/wiki/Tutorial:_Making_a_UI_Recolor) |
| **Terraria** | separate Zoom and UI Scale sliders (added with 4K support in 1.3.5); Andy Bold (a licensed Monotype font); slot drawn as layered textures (`Slot_Back`, `Slot_Front`, `Slot_Overlay`); 40-slot inventory, the hotbar is row 1 | [Inventory page](https://terraria.wiki.gg/wiki/Inventory) |
| **Celeste** | fixed 320×180 buffer scaled by whole numbers; text is smooth hi-res (Renogare via BMFont, licence unverified) rather than pixel; the 640×480 compatibility mode "distorts the graphics and leaves black borders" | [Steam thread](https://steamcommunity.com/app/504230/discussions/0/3832043350345268297) |
| **Hytale** | the 2018 post shows an experimental inventory, crafting, tooltips, map; the 2025 art post gives 64 px per unit for characters, 32 for blocks and props, and "avoid pure white and pure black"; legacy `.ui` markup uses plain pixels and a `PatchStyle` with border and area; a developer answer (found via snippet) says the UI is moving to NoesisGUI (vector, resolution independent). **I found no Hytale UI design post.** | [2018 UI post](https://hytale.com/news/2018/12/a-sneak-peek-at-hytale-s-user-interface), [2025 modelling post](https://hytale.com/news/2025/12/an-introduction-to-making-models-for-hytale) |
| **Vintage Story** | the GUI is vector-style (Cairo on the CPU, then a texture; default sans-serif, decorative Lora: a third-party summary), so it is not a pixel-art UI and its fractional scaling is not a model for us; a GUI scale slider stored as a float; blurry under Windows DPI scaling | [GUI modding docs](https://wiki.vintagestory.at/index.php/Modding:GUIs/en) |
| **Dome Keeper** | Godot and Aseprite; wiki icons: resources 16 px, keepers 20 px. **No UI statements found**: not used for any rule here | [Steam page screenshots](https://store.steampowered.com/app/1637320/Dome_Keeper/) |

Also: Pedro Medeiros (saint11) has a "UI 9 slice" GIF tutorial and outlines tutorials ([index](https://saint11.art/blog/pixel-art-tutorials); I saw only the file names). Game UI Database ([gameuidatabase.com](https://www.gameuidatabase.com)) is the place to compare inventory and HUD screens across games; it blocked our fetch. I found **no GDC talk on pixel-art UI** I could verify.

## What this means for the UI session (in order)

1. **Make GUI scales even** (rule 2): `max_gui_scale`, the auto pick and the settings stepper. Test at 1280×720, 1600×900, 2560×1440 and 3456×2234, and recheck `check_23_settings.png`-style shots at each.
2. **Pressed label shift and the tab join** (rule 10): one line in `render.rs` for the label, and look at `category_pressed.png` as a tab.
3. **A layout outline gizmo** and a **spacing table** in the style guide (rules 9, 3); run the audit scripts at an even scale only.
4. **Parser check for half-pixel borders** and a safe default for a texture with no `.gui` (rule 5).
5. Confirm the magenta outline on two buttons in `check_23_settings.png` is a test marker.

## Not found / not checked

- No page states the cap-height centring rule, the selected-tab join, text-in-bar styling or a 2/4-pixel spacing grid; those are the guide's own conclusions (marked *inference*) and the UI redo's measurements back them.
- Slot, hotbar and heart sizes for Stardew and Terraria are from memory (marked); image pixel sizes can't be read from page text.
- Celeste's HUD at 1080p, Hytale's current font and scale option, Vintage Story's font names, and Dome Keeper's UI: not found.
- I did not run the game or view the UI at other scales; the odd-scale finding comes from the code and the style guide's own statement, and the audit shots at one scale.
- No code was changed.
