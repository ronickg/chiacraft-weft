# Animal Sprite Style Guide

This is the animal companion to the item and block guides. Your animals will be **2D pixel sprites**, but the reference only has animals as unfolded skins wrapped around box models. So this guide does two things:

1. It takes what carries over from the reference: colours, fur and wool, markings, faces and eyes, measured from its skins.
2. It sets the sprite rules (canvas, view, outline, light, animation) so animals sit next to your items and blocks without clashing.

`Animal Prompt References/` holds pieces cut from the reference's skins: faces, flank fur, legs, tails and wool, enlarged. They show **colour, texture and faces**, not poses or silhouettes. When prompting, attach the matching card plus one item image from `Item Prompt References/` (for example `07 Food.png`), so the model also picks up the outline and lighting style.

**L** is perceived brightness, 0–100.

---

## Part 1: What the reference does with animals

### 1. Colour and value

| Animal | Colours | Brightness range | Ramp (dark → light) |
|---|---|---|---|
| Cow | 26 | L 14–97 | dark coat `#27221d #3c332d`, cream patches `#c4b590 #faf7ef` |
| Mooshroom | 26 | L 26–97 | red coat `#69312f #862f3b`, cream patches `#c4b590 #faf7ef` |
| Pig | 20 | L 52–90 | `#ab747d #d5a0a9 #f0c3cf #ffd9eb` |
| Sheep wool | 64 | up to L 87 | `#c7c5c2 #dfddd9` (pale grey, so dye colours can tint it) |
| Wolf | 343 | L 5–90 | `#282828 #474747 #828282 #e6e4e1` |
| Fox | 31 | L 12–93 | `#de6929 #f68d18 #ffa131`, cream belly `#f6eecd` |
| Polar bear | 729 | L 70–97 | `#b6b3a8 #cfccc0 #e1ddd0 #eae6d9 #faf7e8` |
| Panda | 1,904 | L 0–93 | black `#0a0812`, white `#9e9c93 #cdccc6 #efeee8` |
| Tabby cat | 64 | L 25–92 | `#523b27 #5e422d #74563e #957256`, cream chest `#f3e9db` |
| Chicken | 371 | L 24–94 | `#959890 #adafac #d0d3ce #f0f2ee`, red comb `#a81111` |
| Rabbit | 1,032 | L 20–45 | `#3a3228 #493e31 #584a39 #655644 #826f5a` |
| Goat | 419 | L 26–94 | `#4a423a #a29180 #ccbdaf #e6ddd4 #f4efeb` |
| Llama | 61 | L 7–39 | `#32160a #572611 #734019 #8a5f20` |
| Frog | 869 | L 11–63 | `#884a32 #955137 #9f6047 #cc9760` |
| Turtle | 42 | L 12–88 | shell `#1b2213 #313d22`, skin `#857050 #a69670 #e9e0c9` |
| Camel | 7,314 | L 29–82 | `#674423 #c59b58 #e3af67 #ecd09a` |
| Armadillo | 231 | L 24–61 | `#5b422e #836648 #997c59 #ba9a56` |
| Bee | 920 | L 8–87 | dark bands `#240f0a #3b1c15`, yellow `#ddac3e #fee275` |
| Parrot | 752 | L 5–78 | red `#ab2b24 #d02d1e`, blue `#3e2fa9` |

**What this shows:**

- **The classic animals are simpler than blocks.** Cow, pig and fox use only 20–31 colours, against 60–440 on blocks. Furrier animals (bear, panda, camel, rabbit) use hundreds of close shades, but still read as **2–3 colour families**, such as a dark coat plus cream patches, or orange plus cream.
- **Big, bold markings.** Cow patches, the fox's cream belly and muzzle, the panda's black eye patches and legs, and the cat's chest blaze are large, clean shapes. They aren't noisy.
- **Natural, slightly warm colours.** Even the greys lean warm (wolf, chicken, goat). Saturation is high only on the fox, parrot, bee and frog.
- **Whites are cream, not pure.** The cow's light patches top out at `#faf7ef`, the polar bear at `#faf7e8` and the cat's chest at `#f3e9db`.
- **The darkest pixels reach pure black** on the sheep face, wolf, panda and horse. That's harsher than the items. For sprites, use the darkest shade of the coat instead (see Part 2).

### 2. Fur, wool and texture

- **Short fur** (cow, pig, fox, wolf) is drawn as flat areas with **a light 1-px grain**: neighbouring pixels differ by only a few shades. Pig is the smoothest, with soft round gradients instead of grain.
- **Long or thick fur** (polar bear, panda, goat) is a **fine speckle of 3–5 close shades**. The bear body is a soft oval of light speckle that darkens toward the edges.
- **Wool** is a pale grey **chunky speckle** of 2–4 px blobs. It reads as curly, and it's grey so it can be dyed.
- **Tabby stripes** are irregular 2–3 px dark bands across a warm mid-brown.
- **Feathers** (chicken, parrot) are smooth bands of light-to-dark following the wing, with a darker leading edge.
- **Patterns follow the body:** cow patches wrap from back to belly, and fox orange fades into cream from the bottom up (countershading).

### 3. Faces and eyes

- **Eyes are the brightest accent on the head.** They're 2 px wide in the reference's 2× skins, usually **a coloured iris plus one white highlight pixel** on a dark socket:
  - cow and cat: bright sky-blue
  - wolf: olive-gold
  - fox: dark with a cream glint
  - pig: dark with a cream glint
- **Muzzles are lighter than the head:** the cream stripe down a cow's face, the fox's cream lower face and the pig's lighter snout.
- **Noses are a small dark or pink block:** the cat's red-pink nose and the pig's two dark nostrils.
- **Expression comes from eye position and brow pixels**, not mouths. Most animals have no visible mouth.

### 4. Variants

- **Regional variants:** cows, pigs, chickens and frogs come in temperate, warm and cold versions. Wolves come in 9 coats, cats in 11, rabbits in 7, horses in 7 coat colours plus 5 marking overlays, and parrots, llamas and axolotls in several colours.
- **Every animal has a baby texture** in the reference, with the same colours and a bigger head relative to the body.
- **Moods:** wolves have tame and angry versions, and bees have angry and nectar versions. The colours stay the same, and only the eyes and small details change.

---

## Part 2: Sprite rules for your game

### 5. Canvas, scale and view

- **Same pixel size as items.** One sprite pixel equals one item pixel on screen. Don't mix resolutions.
- **Size classes:**

| Class | Canvas | Examples |
|---|---|---|
| Tiny | 16×16 | bee, bug, tadpole, small fish |
| Small | 24×24 or 32×32 | chicken, rabbit, frog, cat, fox, parrot |
| Medium | 48×32 or 48×48 | pig, sheep, wolf, goat, deer, turtle |
| Large | 64×48 or 64×64 | cow, horse, bear, llama, camel, panda |
| Huge | 96×64 or larger | big dinosaurs, sniffer-sized creatures |

- **View:** use a **side view facing right**, with the body in a slight three-quarter turn so the far legs show. Mirror the sprite for left-facing. Keep every animal in the same view.
- **Ground line:** feet touch the bottom row of the canvas, with 1–2 px of headroom above the tallest point.
- **Silhouette first:** an animal must be recognisable as a solid shape. Exaggerate the identifying parts: horns, ears, tails, snouts, humps.

### 6. Outline (matching the items)

- Use a **soft 1-px outline** in a darker shade of each part's own colour: brown around a brown coat, dark rose around a pink pig, and grey-blue around white wool. **Never pure black.**
- Make it lighter on edges facing up or left, and darker on edges facing down or right.
- Inner lines (between legs, under the jaw, around the near ear) use the same outline colour, but only where parts overlap.

### 7. Light and shading

- Light comes from the **top-left**, the same as items and blocks.
- Use **3–5 shades per material:** highlight on the back and head, a mid tone on the flank, a shadow on the belly and the far legs.
- **Far-side legs are 1–2 shades darker** than the near-side legs. This separates them without extra outlines.
- **Countershading:** bellies and muzzles are lighter where the animal has it (fox, deer, cat, goat).
- Texture: use the fur rules from section 2. Grain is subtle, at most 1–2 shades of variation, with no dithering checkerboards.

### 8. Faces

- Show **1 eye** in side view: a 2×2 or 2×3 dark socket with a coloured iris and **1 white highlight pixel** toward the top-left. Small animals can use a single dark pixel plus a highlight.
- Put the eye high on the head and toward the front. Cute or baby animals get bigger, rounder eyes.
- The muzzle or snout is one shade lighter, with a 1–2 px dark nostril or nose.

### 9. Animation

Keep every frame the same canvas size, with the same feet anchor (centre-bottom).

| Animation | Frames | Notes |
|---|---|---|
| Idle | 2–4 | 1-px breathing bob on the body, an ear or tail flick, a blink every few seconds |
| Walk | 4 | Diagonal leg pairs alternate (front-near with back-far). The body bobs 1 px on the passing frames. |
| Run | 4–6 | Longer stride, body stretched, a 1–2 px vertical bounce |
| Eat or graze | 2–3 | Head lowered to the ground, a small chewing bob |
| Hurt | 1–2 | Flash the whole sprite red, or white, for 1 frame |
| Sleep or sit | 1–2 | A lower silhouette with the eyes closed (1 px line) |
| Death | 3–4 | Tip over sideways, then fade out |

**Sheet layout:** one row per animation, frames left to right, and 1 px of transparent padding between frames.

### 10. Variants and babies

- **Colour variants are palette swaps of one drawing**, with the same pixels and new ramps (for example the cow's temperate, warm and cold coats, or wolf coats). This keeps the variants consistent.
- **Babies** use the same palette. The head is about 60% of adult size, with a shorter muzzle, a shorter body, shorter legs and bigger eyes. The canvas is one size class smaller.

---

## Part 3: Prompts

### A) Master prompt for animal sprites

> **Style:** Hand-painted pixel-art game sprite of a [animal], side view facing right with a slight three-quarter turn, standing on the bottom edge of a [W×H] transparent canvas, fully opaque pixels, no anti-aliasing. Matches 32×32 item icons in a warm rustic-medieval fantasy style. The body is built from 2–3 bold colour families with large clean markings, natural slightly warm colours, and creamy whites rather than pure white. Light from the top-left: a lighter back and head, a mid-tone flank, a darker belly, and far-side legs 1–2 shades darker than the near legs. Short fur shown as flat areas with a subtle 1-px grain, no dithering. A soft 1-pixel outline in a darker shade of each part's own colour, never black, lighter on upper-left edges. One eye: a dark socket with a coloured iris and a single white highlight pixel. A clear, readable silhouette with the identifying features slightly exaggerated.

Add **one** of these lines after it:

| Type | Add to the prompt |
|---|---|
| **Farm animal** (cow, pig, sheep, goat) | "Stocky rectangular body, short sturdy legs, friendly face with a lighter muzzle." For sheep: "pale grey chunky curly wool made of 2–4 px blobs, bare dark face and legs." |
| **Small mammal** (rabbit, fox, cat, squirrel) | "Small and agile, a large head and ears relative to the body, a bushy or long tail, cream countershaded belly and muzzle." |
| **Large mammal** (horse, bear, camel, llama, deer) | "Big heavy body, long legs, a defined neck and mane or hump, fur speckled with 3–5 close shades." |
| **Bird** (chicken, parrot, duck) | "Rounded body, feathers as smooth light-to-dark bands following the wing with a darker leading edge, a small bright beak and feet." |
| **Fish or sea animal** (fish, squid, dolphin) | "Side view, smooth scales or skin with a lighter belly, fins in a slightly darker shade, a bright glint on the back." |
| **Reptile or amphibian** (frog, turtle, lizard) | "Low wide body, a mottled skin pattern in 2–3 earthy shades, the shell or back plates as bevelled blocks lit top-left." |
| **Insect** (bee, beetle, butterfly) | "Tiny 16×16, bold banded or shiny body, wings as pale translucent-looking ovals (fully opaque pixels in light tints)." |
| **Dinosaur or fantasy beast** | "Large reptile with a long tail balancing a heavy head, scales as a subtle 2-shade pebble pattern, a lighter belly, bold back stripes or spots, plates or feathers in an accent colour." |
| **Baby version** | "Baby [animal]: the same colours as the adult, a head about 60% of adult size, a shorter snout, short legs, bigger eyes, on a canvas one size smaller." |
| **Animation sheet** | "A sprite sheet of [N] frames for [walk / idle / run], every frame the same size, feet fixed at the bottom centre, diagonal leg pairs alternating, a 1-px body bob, 1 px of transparent padding between frames." |
| **Colour variant** | "The exact same pixel drawing as the attached sprite, recoloured with this palette: [ramp]." |

### Example prompts

- **Cow:** master prompt + farm animal + "64×48, dark brown coat (`#27221d` → `#3c332d`) with big clean cream patches (`#c4b590` → `#faf7ef`), a cream stripe down the face, sky-blue eye, short horns, a pink udder."
- **Fox:** master prompt + small mammal + "32×32, bright orange coat (`#de6929` → `#ffa131`), cream belly, muzzle and tail tip (`#f6eecd`), dark brown legs, pointed ears with dark tips."
- **Sheep:** master prompt + farm animal + "48×32, pale grey curly wool (`#c7c5c2`, `#dfddd9`, outline `#8e8c89`), dark grey bare face and legs, blue-tinted eye."
- **Raptor:** master prompt + dinosaur + "48×48, olive-green scales (`#3e5a2a` → `#8fb24d`), a cream belly, rust-red back stripes, a curved claw on each foot, a quill-feather crest in burnt orange."

### B) Reference images to attach

| Prompt type | Attach |
|---|---|
| Cow, mooshroom, pig, other farm animals | `01 Farm animals.png` |
| Sheep and anything woolly | `02 Sheep and wool.png` (the wool is pale grey so it can be dyed) |
| Wolf, fox, cat, other small and medium mammals | `03 Wild mammals.png` |
| Bear, panda, other big furry animals | `04 Big animals.png` |
| Any animal (for outline and light) | One item image, such as `07 Food.png` or `06 Crafting materials.png` from the `1 Items` folder |

The cards show the reference's faces, fur and markings, cut flat from its skins. **They are not sprites**, so the model should take colours, fur texture and facial features from them, and the pose and silhouette from the prompt text.

### C) Animal checklist for a survival game

- **Farm:** cow, pig, sheep, chicken, goat, horse, donkey, duck, rabbit
- **Forest:** wolf, fox, deer, boar, bear, squirrel, owl, songbirds
- **Cold:** polar bear, snow fox, penguin, yak or mammoth
- **Hot:** camel, armadillo, lizard, scorpion, vulture
- **Wetland and water:** frog, turtle, fish (3–5 kinds), crab, dolphin, squid, ducks
- **Jungle:** parrot, panda, monkey, big cat, snake
- **Insects:** bee, butterfly, beetle, firefly
- **Prehistoric:** raptor, triceratops, stegosaurus, T-rex, pterosaur, long-neck sauropod. Use the dinosaur line, and keep these in the same palette rules so they belong in the same world.
- **For each:** an adult, a baby, 2–4 colour variants, and idle, walk, eat and hurt animations.


---

## Part 4: Our animals are box models

In the game our animals (capybara, chihuahua, cat, Velociraptor, Utahraptor, wyvern) are box
models with a painted texture, built by the `blocky-model` skill: boxes, 64 texels a block, the
light baked in by its painter (`paint.py`). An image model can't lay out a model's texture, so
the sprite rules above are for concept sprites only. What carries over to the models:

- **Carries over:** the colours and ramps (natural, slightly warm, creamy whites, no pure black),
  the 2–3 bold colour families with large clean markings, countershading (lighter bellies and
  muzzles), the fur rules of section 2, and the faces (a dark socket, a coloured iris, one white
  highlight; a lighter muzzle with a small dark nose).
- **Doesn't:** the 1-px outline (a box model's edges are its outline; the painter shades each face
  instead), the sprite canvases and side view, and frame animation (models are rigged and
  animated with keyframes).
- **How to use it:** make a concept sprite with this guide's prompt to agree the colours and face,
  then give the model its coat in those colours (`blocky-model`'s coat data and `house.MATERIALS`).
