# Model Block Style Guide

This guide covers blocks that **aren't plain cubes**: chests, ender chests, shulker boxes, beds, signs, banners, bells and decorated pots. The block guide lists them, but they need their own rules because they're drawn as small models or as several layers.

`Model Block Prompt References/01 Model blocks.png` shows the reference's versions as **flat front faces**: chests with their lid, base and lock put back together, the bell, a decorated pot side, a banner with five pattern layers, and beds and signs.

---

## 1. What the reference does

| Object | Ramp (dark → light) | Look |
|---|---|---|
| Chest | `#251d17 #453829 #686b62 #bbc4c2` | Dark walnut planks in a **riveted iron frame** with corner brackets, and an iron lock plate in the middle of the seam |
| Ender chest | `#17161c #221e2b #373343 #686476` | Dark purple-black stone with faint carved lines and a dark eye motif |
| Banner cloth | `#757573 #a4a29b #bcb9b3 #dcd9d5` | Pale grey-white cloth with soft folds and a scalloped bottom edge. The base and every pattern are drawn grey so they can be dyed. |
| Bell | `#74582d #c2944c #f9ca66 #faf877` | Bright gold with a strong vertical highlight down the centre, darker at the rim |
| Decorated pot | Terracotta orange-brown | Red-brown key pattern bands at the top and bottom, a plain middle for the picture |
| Beds | Dark planks + white pillow + coloured blanket | The frame matches the planks, with a soft white pillow and a blanket in the dye colour |
| Signs | Pale parchment board in a 2-px wood frame | Light board so dark text reads, with a wooden post |

Things the reference does consistently:

- **Model blocks reuse the cube textures' materials:** the chest is the same walnut and iron as the workstations, and the bed frame matches the planks.
- **Iron fittings tie it together:** chests, trapdoors and doors all share the same riveted grey-green iron frame (`#686b62 → #bbc4c2`).
- **Anything that can be dyed is drawn grey** (banners, banner patterns, shulker boxes and bed blankets in some packs), so one drawing serves all 16 colours.

---

## 2. Rules

### How each one is built

| Object | What to draw |
|---|---|
| **Chest** | A lid (about 14×5 front) and a base (about 14×10 front) that meet at a seam, plus a small lock (2×4) centred on the seam. In 2D, draw a closed and an open version, and a double chest made of a left and a right half that join seamlessly. |
| **Bed** | Top view: a pillow at the head end, the blanket over the rest, and the frame showing at the sides. Draw it 1×2 blocks. Side view: legs at the corners. |
| **Sign** | A light board with a 2-px frame, readable dark text area, a post below (standing) or chains above (hanging). The board stays plain because the game writes text on it. |
| **Banner** | A tall cloth, 1×2 blocks, on a pole and crossbar. The base cloth is grey, and each **pattern is its own grey layer on a transparent background**, the same size as the cloth, so patterns can stack and be dyed. |
| **Bell** | A rounded bell shape, bright metal with a strong central highlight, hanging from a wooden or stone frame. |
| **Pot** | Front view of a rounded clay pot with a narrow neck. Patterns are drawn on their own square panels so they can be swapped. |
| **Shulker or storage box** | A cube with a lid that lifts, the lid seam about 3/4 of the way up. Drawn grey for dyeing. |

### Style

- The same light (top-left), soft outline and painted grain as the block guide.
- Metal fittings (rivets, brackets, locks) are 1–2 px bright highlights on mid-grey plates, never pure white.
- Dyeable parts: draw in light grey, from about L 45 for the deepest folds to L 87 for highlights, with no colour.

---

## 3. Prompts

### A) Master prompt

> **Style:** Hand-painted pixel-art [object] for a block-based survival game, [front / top] view, 32 px per block, fully opaque pixels, no anti-aliasing, on a transparent background. Made from the same materials as the blocks: dark walnut planks and a riveted grey-green iron frame, painted grain with 1–2 shade variation and no dithering. Light from the top-left: light top and left edges, darker bottom and right. A soft 1-pixel outline in a darker shade of each part's own colour, never black.

Add **one** of these lines:

| Type | Add to the prompt |
|---|---|
| **Chest** | "Wooden chest with a riveted iron frame and corner brackets, a lid and base meeting at a seam, a small iron lock plate centred on the seam. Also draw the open version, and the left and right halves of a double chest." |
| **Special chest** | "[Ender / trapped / copper] chest with the same shape as the wooden chest, made of [dark purple stone with a glowing eye / wood with a red tripwire hint / copper plates]." |
| **Bed** | "Top view of a bed, 32×64, a white pillow at the top, a [colour] blanket, the dark wooden frame showing at the edges. Also draw the side view with legs." |
| **Sign** | "Wooden sign: a pale parchment board with a 2-px wooden frame and a plain centre for text, on a post (standing) or two chains (hanging)." |
| **Banner** | "Tall cloth banner, 32×64, pale grey with soft vertical folds, a scalloped bottom edge, hanging from a wooden crossbar." For patterns: "only the [pattern] shape as a pale grey layer on a transparent background, the same size as the banner cloth." |
| **Bell** | "Gold bell with a strong vertical highlight down the centre and a darker rim, hanging from a dark wooden beam." |
| **Pot** | "Rounded terracotta pot with a narrow neck, red-brown key-pattern bands at the top and bottom, and a [picture] painted in the middle." |
| **Storage box** | "Box with a lid that lifts on a seam about 3/4 up, drawn in light grey only so it can be dyed." |

### B) Reference images to attach

| Prompt type | Attach |
|---|---|
| Any model block | `01 Model blocks.png` |
| For materials | `05 Planks bricks built stone.png` and `16 Doors and trapdoors.png` from the `2 Blocks` folder |

### C) Checklist

- Chests: wooden, trapped, ender, copper (4 oxidation stages); single, double-left, double-right; closed and open
- Beds: 16 blanket colours (one drawing, recoloured), head and foot halves
- Signs and hanging signs: one per wood type
- Banners: base cloth plus pattern layers (stripes, crosses, borders, emblems)
- Bell, decorated pot plus pattern panels, storage boxes (16 colours)
- Skulls and heads, if your game has them: front view of each monster head
