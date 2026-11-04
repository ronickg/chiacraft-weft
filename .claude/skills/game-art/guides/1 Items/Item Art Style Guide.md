# Item Art Style Guide

Reference set: the reference item textures. About 30 representative items were measured in code and inspected at 10× on a mid-grey background. **L** is perceived brightness, 0–100.

Two files don't match the rest of the set and should not be used as reference: `carrot.png` (only 8 colours, no dark outline) and `leather_helmet.png` (pure grey, because the game tints it).

---

## 1. Canvas and scale

- **Canvas:** 32×32 (563 of the item files); a handful are 16×16 or 64×64.
- **How much of the frame each kind fills (measured):**

  | Kind | Bounding box | Pixels filled | Examples |
  |---|---|---|---|
  | Tools and weapons | 30–31 × 31 px, corner to corner | 25–30% | `iron_sword`, `iron_pickaxe`, `bow` |
  | Ingots | 24×25 px | 31% | `iron_ingot`, `gold_ingot` |
  | Gems | 24×23 / 25×26 px | 37% / 39% | `diamond`, `emerald` |
  | Food | 24–27 × 24–26 px | 43–45% | `apple`, `bread`, `cooked_beef` |
  | Armor | 26–28 × 25 px | 44–52% | `iron_chestplate`, `diamond_boots` |
  | Round things | 24×24 px | 43% | `ender_pearl` |

- **Placement:**
  - Tools sit on a 45° diagonal, handle or pommel at the **bottom-left**, tip at the **top-right**, with 0–1 px padding.
  - Everything else is centred with 3–5 px padding.
  - Ingots tilt along the same diagonal, rising toward the top-right.
- **Animation:** 47 items are animated, each a vertical strip of 32×32 frames. One frame lasts one game tick (50 ms) unless the file says otherwise.
  - `diamond_sword`: 13 frames. Frame 0 is held for 35 ticks, then a sparkle grows from 1 pixel to a 5-pixel plus sign and back; this repeats at four places on the blade. Every changed pixel is brighter than frame 0.
  - `diamond`: 26 frames, 1–14 pixels brightening as twinkles.
  - `gold_ingot` (15 frames) and `copper_ingot` (16 frames): a sheen of about 60–84 pixels sweeps across the bar.

## 2. Outline

- **Thickness and position:** 1 px, made from the item's own edge pixels, not added around them. Hard edges only: every visible pixel is fully opaque (alpha 255), no anti-aliasing.
- **Colour:** a very dark shade of each part's own hue, never pure black, in two tones per part:

  | Part | Outline colours | L |
  |---|---|---|
  | Iron blade | `#393f3c` / `#202322` | 24 / 13 |
  | Gold | `#443319` / `#241b0d` | 21 / 11 |
  | Diamond | `#2d5d69` / `#1a363a` | 31 / 18 |
  | Copper | `#472512` / `#241308` | 18 / 9 |
  | Apple | `#4c100a` / `#2e0704` | 13 / 7 |
  | Bone | `#584b37` / `#3e3421` | 30 / 21 |
  | Leaf / stem | `#395123` / `#1d2913` | 27 / 14 |

- **Lit side vs shadow side:** lighter on edges facing up or left, darker on edges facing down or right.

  | Item | Up/left edge L | Down/right edge L |
  |---|---|---|
  | `iron_sword` | 18 | 12 |
  | `gold_ingot` | 19 | 10 |
  | `apple` | 18 | 8 |
  | `diamond` | 31 | 18 |

  Exception: armor. `iron_chestplate` has one uniform near-black outline, `#08060c` (L3).
- **Inner lines:** each separate part is closed off by its own outline colour (a gold guard has its own brown-gold outline against the blade). Handle wraps are split by 1-px dark red-brown lines (`#1f1008`).

## 3. Colour and palette

- **Colours per item:**
  - Clean items: 13–36 colours (`emerald` 13, `copper_ingot` 17, `bone` 23, `apple` 36).
  - Metal tools: 66–116; bread and armor: over 200.
  - The high counts are not extra shading steps. They are tiny variations of one tone (e.g. `#f3f6f6`, `#f4f7f7`, `#f5f8f8` side by side) that give a soft painted feel.
- **Ramps:** about 5–8 meaningful steps per material, with big midtone jumps (gold: L34 → L55 → L77 → L93).
- **Hue shift by material:**
  - **Gold:** shadows orange-brown (hue 36°, sat 44%); highlights lemon (hue 54–56°, sat 100%).
  - **Diamond:** shadows deep teal (hue 192°); highlights cyan to white (hue 182°).
  - **Emerald:** shadows green (hue 139°); highlights toward mint (hue 148–156°).
  - **Iron:** near-neutral with a faint green-cyan cast (hue 150–180°, sat 4–18%); lightest pixels pure white.
  - **Copper, apple, bone:** hue barely changes from dark to light.
- **Saturation:** natural to warm overall. Gems, gold and redstone are vivid; wood, leather, stone and netherite are muted (sat 6–32%). Apple and copper get *more* saturated in shadow (apple's darkest red is 84% saturated).
- **Range:** most items run from L7–12 (darkest outline) to L90–100 (brightest highlight) — high contrast.

## 4. Light and shading

- **Light direction:** top-left, strong. Shown by the outline measurements and where highlights land.
- **How form is shown:**
  - Metal blades: flat planes — a near-white upper bevel (L89–100) beside a slightly greyer lower half.
  - Ingots, bread, beef: three-quarter view, lit top face and a darker side face about 2 px thick along the bottom-right.
  - Round objects (`apple`, `ender_pearl`): smooth spherical shading.
- **Rim shadow / reflected light:** none. Edge darkening comes only from the two-tone outline.
- **Highlights:**
  - Metal: hard white edge along the top of the blade or pick head.
  - Gems: white facet tips plus single-pixel sparkles.
  - Apple: short, soft pale-pink streak at upper left.
  - Pearl: white blob at upper left against a black core at lower right.
- **Surface texture:**
  - Bread crumb: fine speckling.
  - Leather and beef: mottled noise; beef also has diagonal grill stripes.
  - Ingots: faint embossed rune glyph.
  - Wood: gentle grain streaks.
  - No dithering in the shading itself.

## 5. Materials

| Material | Base | Shadow | Highlight | Outline | What makes it read as that material |
|---|---|---|---|---|---|
| Iron | `#ccd4d5` / `#8b9997` | `#5d6764` | `#f5f8f8`→`#ffffff` along the bevel edge | `#393f3c` / `#202322` | Very bright, slightly cool, mostly highlight |
| Gold | `#e4c75c` | `#b3883f` → `#6d522a` | `#fff594` / `#fffbbd` | `#443319` / `#241b0d` | Warm lemon top over orange-brown shadow |
| Copper | `#bb632b` | `#894a1f` | `#edab7e` peach | `#472512` / `#241308` | Constant orange hue, sheen animation |
| Diamond | `#48aeba` | `#387786` / `#2b4951` | `#c6f9fb`→white facets | `#2d5d69` / `#1a363a` | Cluster of faceted shards, sparkle frames |
| Emerald | `#2aa44a` | `#1d8131` | `#8bebc5` mint | `#194e30` / `#063c17` | Star-shaped leafy crystal, cool green |
| Netherite | `#534b41` warm dark grey | `#221e1c` | `#8b7e6a` | `#110b0b` (L5) | Nearly black with brown undertone; gold-and-gem trim does the work |
| Stone | `#626255` olive grey | `#30302a` | `#a7a48b` | `#181815` | Chunky, chipped shape |
| Wood | `#58462e` | `#332c1d` | `#594a30`, low contrast | `#15140d` | Low-saturation brown with grain streaks |
| Handle wrap | `#6b321a` red leather | `#452011` | `#ac8240` gold bands | `#1f1008` | Diagonal bands, gold pommel knob |
| Bone and binding cord | `#d9ccad` | `#998864` | `#f2ebd9` | `#584b37` / `#3e3421` | Warm cream, knobbed ends |
| Leather | `#593e2f` | `#473226` | `#6f4d3a`, mottled | `#291f18` / `#14100c` | Dark, low-contrast, irregular hide shape |
| Food: red apple | `#bc3929` | `#78190d` | `#d75748` + pale streak | `#4c100a` / `#2e0704` | Glossy round shape, stem and leaf |
| Food: bread | crust `#9a5935`, crumb `#e9d6b0` | `#4d2f1e` | `#deca9d` speckled | `#2c1d14` | Cut face showing the crumb |
| Food: cooked meat | `#764e3d` | `#3a2821` | `#9d7553` | `#0f0a09` | Grill stripes, muted brown |
| Redstone / glowing powder | `#f22020` | `#b61414` | `#fb6e6e` | `#5e0000` / `#300707` | Pile of saturated lumps |
| Water | `#2f7cbf` | `#34698c` | `#53778c`, flat | Sits inside the container's rim | Flat blue surface |

## 6. Shapes and construction

- **Silhouettes:** chunky but detailed. Sticks, handles and blades are 2–4 px wide; only fine accents like a bowstring are 1 px.
- **View:** consistent across the set, nothing isometric.
  - Tools: flat side view laid on the diagonal.
  - Blocky items (ingots, bread, beef): three-quarter top view.
  - Containers and armor: front view.
- **Proportions:** big tool heads (the pick crescent spans about two-thirds of the canvas width); long handles ending in a round pommel knob.
- **Recurring details:**
  - Gold collars and bands (`#ac8240` / `#e4c75c`).
  - Red-leather handle wraps (`#6b321a`) with gold or cream cross-bands.
  - Cream bone lashing (`#d0c5a9`) where a head meets the handle.
  - Small cyan gems set into fittings (netherite sword guard, diamond pickaxe collar).

## 7. Pixel craft

- **Diagonals:** blades and handles are clean 1:1 steps; curves (pick crescent, bow) are freer and more painterly.
- **Shading habits:** no pillow shading — light is clearly directional. Stray single pixels appear only as deliberate sparkles. Banding is broken up by near-identical colour variations.
- **Tiny details:**
  - Stitching and wraps: alternating light and dark pixels.
  - Leaves: 2–3 greens plus a dark outline.
  - Sparkles: a single white pixel or a 5-px plus sign.

## 8. Mood

**Warm, rustic-medieval, polished, cozy fantasy, crisp.** Created by:
- warm hue-tinted outlines instead of black
- gold and leather trim on everything
- bright, clean metal against earthy browns
- gentle hand-painted micro-texture

**Never do:**
- pure-black outlines everywhere (armor is the only exception)
- anti-aliased or semi-transparent edges
- flat single-tone fills
- neon colours outside gems and redstone
- tools laid horizontally or vertically
- chunky 16-px simplicity
- pillow shading or isometric views
- cold, sterile greys with no warm trim

---

## A) Style description for an image model

**Style:** Hand-painted pixel art on a 32×32 transparent canvas, fully opaque pixels, no anti-aliasing. Warm rustic-medieval fantasy: red leather wraps and gold fittings with warm silver metal, bright orange copper and wood, sapphire-blue gems and vivid red-orange food, made to sit on dark brown slots inside bronze frames. Light and airy rather than heavy: shadows are mid-dark, never deep, and nothing goes near black. Each material uses a 5–8 step ramp; gold and fire shift toward lemon in the highlights. Light from the top-left. Metal is mostly bright, with a near-white top bevel. Subtle painted grain, no dithering. Small white sparkles on gems.

**Outline:** A soft 1-pixel outline made from the item's own edge pixels: a medium-dark shade of each part's own colour (brightness about 15–45%), never black. Slightly lighter on upper-left edges, slightly darker on lower-right edges. Corner pixels match the edge beside them, with no extra-dark corners. Each part gets its own outline colour.

**View:** Tools lie diagonally, handle bottom-left and tip top-right, filling the canvas. Materials and food are centred in a slight three-quarter top view with 3–5 px of padding.

**Parts:** where materials meet (blade and guard, head and handle, cork and glass), the outline changes with them: dark browns round wood and leather, blue-greys round iron, warm browns round gold, teal round diamond. Never one outline colour round the whole item, and never the red-brown line image models fall back to.

**Background:** flat `#FF00FF` magenta everywhere outside the item (flat `#00FF00` green when the item itself is purple, pink or magenta). It becomes transparent; every pixel of the item is fully opaque.

**Learned from Ronald's answers** (the item icon redo, 2026-11-04; each line is something he approved or sent back):
- Metal is warm silver with a near-white bevel; handles are red-brown leather with gold bands and a round gold pommel; fittings are gold (approved across every tool and armour piece).
- A hoe is a long plain handle with one flat blade like a garden hoe, about 10 px wide and 4-5 px tall, set at a right angle across the top of the handle and pointing down-left: never a block-shaped hammer head (too big) nor a thin hook (too small). Tool heads must read at slot size.
- A leather helmet is a soft cap with stitched panels, a short brim and ear flaps, open face; metal helmets have the visor.
- Boots are a pair seen a little from the side, so each foot and toe shows at the bottom; never shin guards alone.
- Sticks, shafts and handles are a full 3 px thick from end to end.
- Coal is a deep, glossy black-brown with bright blue-grey glints on its facets; dark powders (gunpowder) keep strong contrast between their light and dark sides. Never dull or muddy grey.
- Meat keeps its outline like everything else; cooked meat is a warm brown with darker grill or char marks, never grey.
- An item named for a body part shows that part (a spider eye is an eye, with an iris and a glint, not a plain ball); a food in a container reads as the container (kibble is a small tied sack, not a heap).
- A whole animal (a raw hare) reads as that animal at slot size: its outline and one or two telltale parts (long ears, long hind legs).
- Seeds, grains and small berries need contrast and size: a few big clear grains or berries with a light top-left and a dark outline, never a pale washed-out heap or a tiny dark speck. Grain ears (barley, rye) are full and golden with clear bearded ears, not wispy.
- Beans are clearly beans (kidney-shaped, glossy, with a speckle or a pale eye), never a pink lump that reads as meat; small round berries (crowberries) are glossy and big enough to see one by one.
- Things tied to creatures show a little of their real size against each other, in a gentle range (Ronald, 11-04): the longest side runs from about 16 px of the 32 (60%, tiny things: an earthworm, a grub, a chicken's egg nearer 70%) to about 26 px (100%: a nest, a raptor egg, a saddle for the biggest mount), centred, never so small it can't be read. A saddle matches its mount; a spawn egg is scaled a little by its animal (a chicken's small, a cow's full). Very thin creatures (an earthworm, a snake) get a little more than their real scale, about 20 px, so they stay readable at 1x.

How to prompt, check and cut it: `0 Making it.md`.

### Reference images to attach

`Item Prompt References/` has one image per kind of item. Each shows reference items at 8× size on your dark brown slot colour. Attach the matching image with each prompt, plus one of your own UI sheets for colour.

| Item kind | Attach |
|---|---|
| Tools | `01 Tools (diagonal).png` |
| Swords, bows, crossbows, tridents, shields | `02 Weapons.png` |
| Armour | `03 Armour (front view).png` |
| Ingots and raw ore | `04 Ingots and raw ores.png` |
| Gems and crystals | `05 Gems and crystals.png` |
| Sticks, string, feathers, leather, bones | `06 Crafting materials.png` |
| Food | `07 Food.png` |
| Buckets, bottles, potions | `08 Containers and liquids.png` |
| Books, paper, maps | `09 Paper and books.png` |
| Compass, spyglass, shears, rods, rope | `10 Small tricky objects.png` |
| Saddles, bags, eggs, bowls, fish | `11 Soft and organic.png` |
| Arrows, clocks, horns, charms | `12 Arrows, instruments and charms.png` |

## B) Material sheet

Outline colours are written lit side / shadow side.

```
iron: #c9c3be warm silver base, #8e8884 shadow (#5a5553 deepest), near-white #ece6e2 bevel edge, clean faint brushed sheen, #6a6461/#48423f soft warm-grey outline, pale warm silver that's mostly highlight
gold: #e4c75c base, #bd954e→#7d6236 orange-brown shadow, #fff594 lemon highlight, smooth, #675428/#43371a warm brown outline, warm yellow top shifting to orange shadow
copper: #f07e37 bright orange base, #cc6629 shadow (#9c3f13 deepest), #f7ab68 peach highlight, smooth with faint sheen, #85300b/#6a2608 rust outline, vivid orange bar
gem (sapphire, the reference's crystal): #4d97e2 base, #2f71da shadow (#1f4dad deepest), #b6effd→white facet tips, clustered faceted shards with sparkles, #1f4dad/#183c86 blue outline, bright blue crystal cluster
diamond: #48aeba base, #468594/#3a5a62 teal shadow, #c6f9fb→white facet tips, faceted shards with sparkles, #3a8490/#26555b teal outline, crystal cluster in cyan
emerald: #2aa44a base, #288f3f shadow, #8bebc5 mint highlight, smooth facets, #227c3e/#125e28 green outline, star-shaped green crystal
netherite: #5e564b base, #2e2927 shadow, #8b7e6a muted highlight, faint grain, #312a25/#2c2521 dark warm-grey outline, dark warm grey set off by gold trim and a cyan gem
stone: #958672 warm grey-tan base, #6e5f4f shadow (#4f4234 deepest), #e1d4c2 chipped highlights, rough, #4f4234/#3b3127 outline, chunky chipped lumps
wood: #bc6618 warm orange-brown base, #a25612 shadow (#6f3208 deepest), #dfa05d highlight, grain streaks, #6f3208/#552606 outline, warm plank or log with clear grain
leather wrap: #6b321a red-brown, #542c1a shadow, #ac8240 gold bands, stitched diagonal bands, #532714/#4b2412 outline, banded grip ending in a gold pommel knob
bone/cord: #d9ccad base, #a59573 shadow, #f2ebd9 highlight, clean, #776a53/#584e39 outline, warm cream lashing with knobbed ends
cloth/leather hide: #6a4a38 base, #564033 shadow, #7f5a44 mottled highlight, mottled noise, #533a2c/#37271e outline, soft irregular warm-brown shape
food (fruit, berries, raw meat): #d8221e base, #be1916 shadow (#a21310 deepest), #f36555 plus a pale upper-left streak, glossy, #85251a/#5e1811 red-brown outline, round glossy shapes with a stem or leaf
food (vegetables): carrot #e77f0d base, #cc6c0b shadow (#7f3906 deepest), #e9b32a tip highlight, #7f3906 outline, bright orange with green leaf tuft
food (baked/cooked): crust #9a5935 / meat #764e3d, #5c3d29 / #48352d shadow, #deca9d speckled crumb / #9d7553, speckle or grill stripes, #5b3722/#543320 (bread) or #38251e (meat) outline, cut face or char marks
liquid: #2f7cbf base, #427899 shadow, #53778c flat sheen, flat, outlined by its container's rim (#545c59/#313434), flat blue surface inside a bright metal rim
glass: #7da6b4 base, #537d8e shadow (#3d6170 deepest), #cee9f0 highlight, clean, #3d6170 outline, pale blue-teal block or pane
fire / torch flame: #ee820d base, #cb5509 shadow (#933605 deepest), #fbbf14→#fdf77c core, clean, #933605 outline, bright flame that goes yellow-white in the middle
glowing powder: #f22020 base, #bf1d1d shadow, #fb6e6e highlight, lumpy pile, #a20f0f/#721010 outline, saturated clustered nuggets
armor plate: #b5aea9 warm silver base, #8e8884 shadow, #ece6e2 highlight, clean, #3a3533/#2c2826 outline (the darkest in the set, still not black), front view with gold trim
```

### Example item requests

- "a gold-trimmed iron war hammer (bright warm silver head with a near-white top bevel and a soft warm-grey outline, gold collar band, long handle wrapped in red-brown leather with gold cross-bands and a round gold pommel, no near-black edges or corners), lying diagonally from bottom-left to top-right"
- "a raw sapphire cluster (three faceted shards in bright #4d97e2 blue with a #2f71da shadow and white facet tips, one white plus-shaped sparkle, soft #1f4dad outline that's lighter on the upper left), centred with 4 px of padding, readable on a dark brown slot"
- "a wedge of cheese (warm cream-yellow top face with fine speckle, orange-tinted side face 2 px thick, a few round holes in soft ochre, gentle warm-brown outline rather than a dark one), three-quarter top view, centred"

---

## C) Matching your UI reference

Measured from your crafting and workstations sheet (`UI Asset Reference 7`). Your other sheets (1–6, 8) use the same frames and colours as far as I could see by eye; only sheet 7 was available as a file to measure.

### What I pulled into A and B above

- **Iron** is a warm silver (slightly pinkish grey, hue about 15–25°) instead of the reference's cool green-grey.
- **Copper and wood** are bright, saturated orange (`#f07e37`, `#bc6618`) instead of the reference's muted browns.
- **Gems** are sapphire blue (`#4d97e2`) in your sheets; I added a sapphire line and kept the cyan diamond as an option.
- **Food** is vivid red and orange (meat `#d9443e`, berries `#d8221e`, carrot `#e77f0d`).
- **Stone, glass and torch flame** follow the reference ramps.
- Icons must read on a **dark brown slot (`#352015`)**. That's one more reason for the lighter outlines and bright upper faces: a near-black outline disappears against that slot.

### UI palette (for frames, panels and text around the icons)

| Role | Colour | Notes |
|---|---|---|
| Page background | `#1a1313` | near-black warm brown |
| Panel interior | `#16100f` | slightly darker than the page |
| Slot background | `#352015` | where icons sit; slot rim `#683e26` → `#965735` |
| Bronze frame | `#89451e` → `#a75120` → `#b35a26` | 5–6 px border with corner bevels |
| Title tab / button orange | `#e47c02`, light edge `#f6b24e`, dark edge `#d0730a` | main call-to-action colour |
| Selected glow | `#ffdd20` → `#f3ab1f` | 2-px glowing rim on the selected slot |
| Heading text (gold) | `#f2da9a` (range `#d2b782`–`#fcf0b9`) | serif display headings |
| Body text | `#e4dacf` cream, muted `#c6a383` | |
| Progress bar | fill `#f79701`, track `#271e1a` | |
| Success green | `#8ac80f` (`#557d0a` dark, `#c1ed42` light) | ticks, "Craftable" state |
| Error red | panel `#3f1211`, icon red `#d8221e` | "Missing materials" state |

### Icon rules that tie them to this UI

- Keep each icon's brightest face toward the upper left, so it pops against the dark slot.
- Use warm hues everywhere except gems, glass and water. Even greys lean warm.
- Leave 3–5 px of transparent padding inside a 32×32 slot, as in your inventory examples.
- Stack counts and badges in the UI sit at the bottom-right corner, so keep that corner of the icon fairly simple.


---

## D) Tricky objects: construction recipes

Image models struggle most with containers, glass, thin parts and things seen in perspective. These recipes are based on how the reference actually builds each object, measured pixel by pixel, with colours switched to the lighter, warmer palette above. When you request one of these, **paste its "Prompt line" into the item request** in place of a short name like "a bucket".

### General rules for complex objects

1. **Silhouette first.** Every object should be recognisable as a flat shape in one colour. If a request has more than three main parts, drop one.
2. **Containers are seen from the front and slightly above.** The opening is a flattened ellipse about a third of the object's height, showing the inside. Never a perfect side view, never straight down.
3. **Openings have a bright rim.** A 1–2 px light lip runs all the way round any opening, brighter on the far (top) edge than the near edge.
4. **Glass is outlined in white or very pale blue, not dark.** The outline is the brightest thing on a bottle. The glass itself is mostly transparent, showing the background, with 1–2 white highlight strokes on the upper left.
5. **Liquids are a flat surface plus a body.** The top of a liquid is an ellipse a little lighter than the liquid, with one pale glint; inside glass it fills the lower half to two-thirds.
6. **Thin parts (strings, wires, handles, fishing lines) are 1 px, in a pale colour,** drawn as a clean straight or evenly curved line. Never a dark 1-px line, which looks like a crack.
7. **Symmetric objects** (buckets, bottles, helmets, chestplates, bowls) are drawn mirror-symmetric down the middle, except for lighting: the left half is lighter.
8. **Band and trim stripes** go round the object following its curve: on a round object a band is a slight downward smile, not a straight line.

### Recipes

**Bucket (empty)**
- Size and view: about 25×27 px, centred, front view from slightly above. Slightly tapered: the bottom is about 4 px narrower than the top.
- Top (rows 1–10 of the object): an oval opening about 19 px wide and 8 px tall. Thick warm-silver rim (`#ece6e2` on the far edge, `#c9c3be` on the near edge). The inside is dark warm grey (`#3a3e3d`→`#5a5553`), lighter at the back wall.
- Upper body (next 7 rows): warm silver metal, curved: a bright vertical highlight stripe 2 px wide just left of centre (`#ece6e2`), mid tones `#c9c3be`→`#8e8884`, darker toward both sides.
- Middle band (3 rows): a dark rope or leather band going round the bucket (`#76705d` stitches on `#3e3b2f`), dipping slightly in the middle.
- Lower body (6 rows): same metal, a step narrower, with the same left-of-centre highlight.
- Handle: two small round lugs on the left and right edges at the top of the band, and a thin 1-px pale metal handle arching up behind the rim (optional).
- Avoid: a flat straight-on rectangle; no visible inside; a handle drawn in front of the opening; a dark outline around the rim.
- **Prompt line:** "a metal bucket seen from the front and slightly above, slightly tapered, oval opening showing the dark inside, thick bright warm-silver rim, vertical highlight just left of centre, a dark rope band around the middle, small round handle lugs on both sides"

**Bucket with contents** (water, lava, milk, fish)
- Same bucket. Fill the oval opening to just below the rim with the contents' flat surface:
  - water `#3f8dcc` with a lighter `#50a0da` centre and one pale glint;
  - lava `#c62d1e` with bright `#e88b22` swirls and a deep `#771817` edge;
  - milk `#e0e0e0` with a soft `#d9d9d9` edge.
- For a fish bucket, show the fish's tail or back breaking the water surface.
- **Prompt line:** add "filled to just below the rim with [water / lava / milk]: a flat oval surface with one pale glint".

**Potion / glass bottle**
- Size and view: about 21×30 px, front view.
- Cork (top 3 rows): a small tan cork, `#a89a63`→`#584a13`.
- Neck wrap (next 5 rows): a white cloth or wax collar tied round the neck, with 2–3 thin dark-red cords hanging down onto the bottle.
- Neck (4 rows): narrow, 4–5 px wide.
- Body: a round flask about 19 px wide. The glass outline is white (`#ffffff`) on the outside with a pale grey inner edge (`#dbd9d9`). Inside is transparent, with one curved white highlight stroke on the upper left.
- Contents: fill the lower 60% with the potion colour (honey `#ffc800`→`#b37506`, magic purple `#8850c2`→`#4f2b7c`), with a lighter surface line and a white glint.
- Avoid: a dark outline round the glass (it turns the bottle into a solid object); a fully opaque bottle; a square bottle.
- **Prompt line:** "a round glass potion flask, front view, white glass outline with a pale inner edge, transparent glass with one curved white highlight on the upper left, small tan cork, a white cloth tied round the neck with thin red cords, filled two-thirds with [colour] liquid"

**Bow**
- Size and view: fills the canvas (29×29), lying diagonally.
- Shape: the wooden bow bends in a smooth C from the top-right tip, round the upper left, to the bottom-left tip. The string is a straight 1-px pale cream line (`#eee2c6`) from tip to tip.
- Grip: a red-leather wrap (`#6b321a`) with gold bands in the middle of the bow.
- Tips: small pale bone or metal caps on both ends.
- Avoid: the string drawn dark; the bow bending the wrong way (the string should face the bottom right); an arrow unless asked for.
- **Prompt line:** "a recurve bow lying diagonally, warm brown wooden limbs curving round the top left, red leather grip with gold bands in the middle, small pale caps on both tips, a straight thin cream string from tip to tip"

**Crossbow**
- Fills the canvas diagonally: a long red-brown wooden stock from bottom right to top left, a pale metal bow across it near the front, a cream string pulled back to a small trigger catch, and iron rivets along the stock.
- **Prompt line:** "a crossbow lying diagonally, long red-brown wooden stock with small iron rivets, pale steel bow limbs across the front, cream string drawn back to the trigger"

**Book**
- Size and view: about 27×31 px, a closed book lying at an angle, seen from above so it shows a diamond-shaped cover.
- Cover: deep red leather (`#a1372c`→`#8e2e25`) with a slightly darker border 2 px in from the edge.
- Thickness: the pages show as a cream block (`#d2c8ab`, lines `#9d7158`) about 4 px thick along the lower-left edge.
- Spine and corners: dark-brown spine on the lower right; small pale metal corner caps on the two lower corners.
- Variants: a book and quill adds a white quill pen and a small blue ribbon lying across the cover; an enchanted book adds a faint purple shimmer.
- **Prompt line:** "a closed leather book lying at an angle, deep red cover with a darker border, a cream page block 4 px thick showing on the lower-left edge, dark brown spine, small metal corner caps"

**Map / paper**
- About 26×24 px, a slightly crumpled square sheet lying at an angle.
- Parchment `#e8e1d1`→`#d3c6aa` with soft fold shading and a warm-brown edge `#7a6c50`. A filled map adds faint fold lines and thin pale lines for paths.
- **Prompt line:** "a slightly crumpled square sheet of parchment lying at an angle, soft cream with gentle fold shading and a warm brown edge"

**Helmet**
- Size and view: about 20×24 px, front view, centred.
- Shape: a rounded dome narrowing to the chin. A gold crest ridge runs down the centre of the dome (`#e4c75c`, highlight `#fff594`).
- Face: a horizontal eye slit about 2 px tall across the middle, with a thin gold trim line above it. Below the slit, 3–5 short vertical breathing slots.
- Metal: warm silver, lighter on the left half.
- **Prompt line:** "a knight's great helm, front view, rounded warm-silver dome with a gold crest ridge down the centre, a dark horizontal eye slit with gold trim above, short vertical breathing slots below"

**Chestplate**
- About 28×25 px, front view: a breastplate with rounded shoulder plates, a gold collar piece at the neck, a slight V shape down the middle, and a brown leather belt and straps at the waist.
- **Prompt line:** "a plate chestplate, front view, warm-silver breastplate with rounded shoulder plates, a gold collar at the neck, brown leather belt and straps at the waist"

**Leggings and boots**
- Leggings: about 24×24 px, front view; a brown leather belt with a small buckle at the top, two metal-plated legs with knee plates, a dark gap between the legs.
- Boots: about 24×18 px, a pair side by side, front view; metal shin plates on top, brown leather underneath, and a dark gap between them.
- **Prompt lines:** "plate leggings, front view, brown leather belt with a small buckle, two warm-silver plated legs with knee plates" / "a pair of plated boots side by side, front view, warm-silver shin plates over brown leather"

**Shield**
- About 24×28 px, front view: a heater-shaped (flat top, pointed bottom) wooden shield with a 2 px metal rim, a central metal boss or simple painted emblem, and a light top-left highlight.
- Don't copy the emblem on the reference's shield, which is based on another game's shield. Use a simple original device such as a cross, a band or a single animal.
- **Prompt line:** "a heater-shaped wooden shield, front view, 2 px warm-silver rim, a simple painted [emblem] in the centre, lighter on the upper left"

**Fishing rod**
- Lies diagonally: a long thin wooden rod (2–3 px) from bottom left to top right with a small reel near the handle, a 1-px pale line hanging down from the tip, a red-and-white float and a small metal hook.
- **Prompt line:** "a fishing rod lying diagonally, thin wooden rod with a small reel by the handle, a thin pale line hanging from the tip to a red-and-white float and a small hook"

**Shears**
- Diagonal: two pale steel blades at the top right, crossed handles wrapped in red leather with cream bands, and a round pivot rivet in the middle.
- **Prompt line:** "a pair of shears lying diagonally, two pale steel blades, crossed handles wrapped in red leather with cream bands, a round pivot rivet"

**Compass / clock (round instruments)**
- About 26×24 px, seen from slightly above so the face is a wide oval. A thick metal case (warm silver for a compass, gold for a clock) with a small loop on top.
- Compass face: cream `#d9d5c8` with a red-and-white needle. Clock face: a small painted day-and-night landscape in a round window.
- **Prompt line:** "a round pocket compass seen from slightly above, wide oval cream face with a red-and-white needle, thick warm-silver case, small loop on top"

**Arrow / spear / trident**
- Diagonal from bottom right (fletching or butt) to top left (head) for arrows; tip top right for spears and tridents, matching the tools.
- Shaft 2 px wide; a pale steel head about 5–6 px long; arrow fletching is two or three cream-and-brown feathers. A trident has three prongs and a small gem at the collar.
- **Prompt line:** "an arrow lying diagonally, thin wooden shaft, small pale steel arrowhead, cream-and-brown feather fletching"

**Flint and steel**
- Two objects side by side: a C-shaped pale steel striker loop on the left and a dark grey flint chunk on the right, overlapping slightly.
- **Prompt line:** "flint and steel: a C-shaped pale steel striker loop beside a chunky dark grey flint stone, overlapping slightly"

**Bowl and stew**
- About 26×16 px, seen from the front and slightly above: a wide wooden bowl (`#86673c`→`#3f3023`) with an oval opening showing the inside, and a bright rim.
- Stew: fill the opening with a lumpy surface in the food's colours (cream for mushroom, orange-red with chunks for meat stew) and an optional spoon handle sticking out to the upper left.
- **Prompt line:** "a wide wooden bowl seen from the front and slightly above, oval opening showing the inside, bright rim, filled with [stew] with a spoon handle sticking out to the upper left"

**Egg**
- About 20×24 px, upright oval, slightly narrower at the top. Warm cream `#f9d6a7`→`#bb905c` with a soft highlight on the upper left. Plain, with no outline darker than `#6b4e35`.

**Saddle**
- About 24×24 px, seen from the side and slightly above: a dark brown leather seat with a raised back, a stirrup hanging below with a small metal ring, and a few metal studs. Leather `#9f8167`→`#3a2d25`.

**Lead / rope**
- A loose coil of pale cream rope (`#e8e4df`, shadow `#8b8375`) forming a loop with a tail, 2 px thick, with a small metal clip or knot at one end.

**Bundle / pouch**
- About 18×24 px, upright: a round brown leather sack, gathered and tied at the neck with a cream cord, the top flaring out above the tie.

**Spyglass**
- Diagonal from bottom left (eyepiece) to top right (lens): a copper-and-leather tube in three segments that get wider toward the lens, with a pale blue glint on the lens.

**Fish**
- Lying diagonally with the head at the bottom left and the tail at the top right, about 26×28 px. Belly lighter than back, a visible fin on top, a small dark eye, and scales suggested by soft speckle, not drawn individually.

**Torch** (the reference draws this as a block, so this recipe comes from your UI reference)
- Upright, centred: a short wooden handle 3–4 px wide with a small iron collar, and a flame on top shaped like a teardrop. The flame is `#933605` at the edge, `#ee820d` in the body, and `#fbbf14`→`#fdf77c` in the core.
