# Effects Style Guide

This guide covers small visual effects: **particles** (smoke, flames, sparkles, hearts, bubbles, explosions), **status effect icons**, and the **block-breaking cracks**.

`Effect Prompt References/01 Effects.png` shows the reference's versions: one frame of 12 particles, frame strips for smoke, poof, explosion and flame, 12 status effect icons, and 5 of the 10 crack stages.

---

## 1. What the reference does

### Particles

- **Small:** most particles are 8×8 to 16×16. Big ones (explosion, sonic boom) go up to 32–64.
- **Few colours, very bright cores:** flame runs from deep orange to a near-white yellow centre, crit is a white star burst, and the enchant glyph is a bold red stroke.
- **Animated as a sequence that shrinks or fades:**
  - smoke: 8 frames, from a big puff down to a dot
  - poof: 8 frames, a grey cloud breaking up
  - explosion: 16 frames, an orange fireball turning into dark smoke
- **Pale and grey particles get tinted by the game:** smoke, dust, the generic spell swirl and the falling-leaf particles are drawn grey or white, and the game colours them.
- Some are **simple symbols**: a heart, a music note, an angry red cross-star, a happy green diamond, a bubble ring.

### Status effect icons

- 18×18 drawn at 2× (36×36 here), on a transparent background, shown on a dark tile.
- **A small illustrated object**, not an abstract symbol: a winged boot for speed, a turtle for slowness, a golden pickaxe for haste, a sword for strength, a heart for health, a shield with fire for fire resistance, bubbles for water breathing, a skull and bones for poison, a moon for night vision.
- They use the item style exactly: soft outline, top-left light, a 4–6 shade ramp per material.

### Breaking cracks

- 10 stages, drawn in **grey on a transparent background**, laid over any block. They start as a single crack pixel in the centre and spread outward until stage 9 covers most of the face.
- Lines are 1–2 px, with lighter chipped edges beside the dark cracks.

---

## 2. Rules

- **The same pixel size** as items, blocks and sprites. Particles are never smoothed or scaled in-between.
- **Bright core, darker rim:** fire, magic and sparks have a near-white centre, a saturated middle and a darker rim, so they glow against dark backgrounds.
- **Tintable particles in grey:** if the game recolours a particle (dust, potion swirls, leaves, smoke), draw it in light greys only.
- **Animations shrink, fade or break apart:** 4–8 frames, with each frame smaller or more scattered than the last. Loops (flame, glow) need a seamless last-to-first frame.
- **Outline:** particles under 12 px don't need one. Larger particles and all status icons use the item outline: a soft, darker shade of their own colour, never black.

---

## 3. Prompts

### A) Master prompt

> **Style:** Pixel-art game effect, crisp pixels, no anti-aliasing, transparent background, matching hand-painted 32×32 item icons in a warm fantasy style. Bold simple shapes with very few colours. Glowing effects have a near-white core, a saturated middle and a darker rim. Light from the top-left.

Add **one** of these lines:

| Type | Add to the prompt |
|---|---|
| **Particle** | "A single [flame / spark / leaf / dust mote / heart / note / bubble] particle, [8×8 / 16×16], one bold shape, 3–5 colours." |
| **Particle animation** | "A strip of [N] frames, [size] each, side by side, of [smoke puff / explosion / poof / magic sparkle] that [shrinks / fades / breaks apart] from frame 1 to frame N." |
| **Looping effect** | "[N] frames of [flame / glow / portal swirl] that loop seamlessly, the last frame flowing into the first." |
| **Tintable particle** | "Drawn in light greys only (no colour), so the game can tint it." |
| **Status effect icon** | "An 18×18 icon of [object meaning the effect], a small illustrated object in the item style, a soft outline in its own darker colour, readable on a dark tile." |
| **Cracks** | "10 stages of block-breaking cracks, 32×32 each, grey lines on transparent, starting as one crack in the centre and spreading to cover the face by stage 10, 1–2 px lines with lighter chipped edges." |
| **Hit or damage** | "A [hit spark / damage flash / critical star] of 3–4 frames, white-yellow core, quick burst outward." |

### B) Reference images to attach

| Prompt type | Attach |
|---|---|
| Particles, animations, cracks | `01 Effects.png` |
| Status effect icons | `01 Effects.png`, plus an item image such as `10 Small tricky objects.png` from the `1 Items` folder |

### C) Checklist

- **Particles:** flame, soul flame, smoke (small and large), poof, explosion, sparks and crit, magic sparkle, dust (tintable), leaves (per tree type), water splash, drip, bubble, rain splash, snow, hearts, anger, happy, note, sleep "Z"
- **Status effect icons:** one per buff and debuff in your game, such as speed, slowness, strength, weakness, regeneration, poison, fire resistance, cold, heat, wet, well-fed, hungry, rested, encumbered
- **Combat:** hit spark, critical hit, block or parry flash, damage numbers
- **World:** 10 crack stages, item pickup sparkle, level-up burst, crafting puff
