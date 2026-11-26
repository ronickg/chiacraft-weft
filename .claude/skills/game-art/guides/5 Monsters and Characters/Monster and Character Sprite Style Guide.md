# Monster and Character Sprite Style Guide

This guide covers monsters, people (villagers, traders, raiders) and **armour as it looks when worn**. Like the animals, they are drawn as **2D pixel sprites**, using the same rules as `Animal Sprite Style Guide.md` (canvas, outline, light, animation). This guide only adds what is different.

`Monster Prompt References/` holds two images:

- **`01 Monsters and characters.png`** shows humanoid monsters laid out as a **flat front view** (head, body, arms and legs unfolded from the reference's skins), plus creature faces and the faces of villagers and raiders.
- **`02 Armour worn on the body.png`** shows every armour set as a flat front view (helmet, chestplate with arms, leggings and boots): leather (dyed brown and undyed grey), chainmail, iron, copper, gold, diamond, netherite, plus the turtle helmet.

These are **not finished sprites**. They are flattened skins, so use them for colour, costume detail and faces, and let the prompt set the pose.

---

## 1. What the reference does

### Colour

| Character | Ramp (dark → light) | Notes |
|---|---|---|
| Zombie | `#191918 #383330 #84583f #aab59d #ece3ca` | Sickly sage-green skin, torn brown leather, grey trousers |
| Husk | `#30312f #556b3b #c6bc9e #d4c8b0 #ded8c8` | Pale sand wrappings over green skin |
| Drowned | `#4e1813 #503e2e #72746a #d1d0cb` | Teal skin, glowing cyan eyes, red-and-white striped rags |
| Enderman | `#000000 #0a0a0a #101010 #131313` | Almost pure black, with small glowing magenta eyes |
| Spider | `#070000 #140b0b #2d2a2a #bec3c2` | Black-brown with bright red eyes |
| Witch | `#1a0b31 #391369 #585858 #b6ad4c` | Purple robes, olive-yellow skin |
| Pillager | `#101812 #2b2019 #665945 #aca28f` | Grey-beige skin, dark brows, a scar |
| Piglin | `#30180f #3f342f #5d4f44 #846e5c #bca694` | Warm brown, red-orange eyes, tusks |
| Villager | `#2d231b #767676 #929292 #d1d1d1` | Olive-yellow face, dark hair, robes in profession colours |

- **Monsters are dark and moody.** Their darkest 5% sits near black, much darker than the items. For your game, raise these to the darkest shade of the character's own colour, as with the animals.
- **Eyes glow.** Drowned (cyan), spider (red), piglin (red-orange) and enderman (magenta) all use small, very bright eyes on dark faces. That's the main way to show a creature is hostile.
- **Clothes are drawn as real costumes:** torn leather, belts, buckles, striped rags and robes with trim. Each piece gets its own 3–4 shade ramp.

### Armour worn on the body

| Set | Ramp (dark → light) | Look |
|---|---|---|
| Leather | brown, tinted by dye (drawn grey) | Soft cap, jerkin with laces, boots with folded tops |
| Chainmail | `#373635 #5d615f #787e78 #bccec8` | Ring pattern of light and dark pixels, worn over a red-brown tabard |
| Iron | `#664632 #8c8f95 #a4acb2 #c6d3d2` | Bright plate with a centre ridge, leather straps and brown trim |
| Gold | `#601814 #a52725 #cba64f #ece17e` | Ornate gold plate with red cloth and red gems |
| Diamond | `#1a363a #5e5243 #52dde8 #d8f9fb` | Cyan crystal plates on a dark leather base |
| Netherite | `#0f0706 #272321 #4d423c #c99b50` | Near-black plate with gold trim lines |
| Copper | bright orange copper with green verdigris | Rounded copper plates with green patina trim and straps |
| Turtle (helmet only) | mossy green | A shell cap with a lighter rim |

- Every set mixes **metal with leather or cloth** underneath: straps, belts and tabards. That makes armour look worn by someone rather than painted on.
- The helmet frames the face. In the front view, the face stays visible through a visor gap or open front.
- **Keep it consistent with the armour icons** in the item guide: the same materials, colours and trim, so a chestplate in the inventory and on the body look like the same object.

### How each set looks on a person, piece by piece

This is what `02 Armour worn on the body.png` shows. Use it when you draw armour **on a character**, not as an inventory icon.

| Set | Helmet | Chest and arms | Leggings | Boots |
|---|---|---|---|---|
| **Leather** | Soft hood or cap over the top and sides of the head, face fully open | Laced jerkin with a collar, a belt with a buckle and a ragged lower hem; plain leather sleeves | Loose trousers with a jagged, torn hem | Soft boots with folded-over tops |
| **Chainmail** | Mail coif over the head and sides, open face with a brown leather rim | Ring-pattern mail shirt and sleeves, a **red-brown tabard** down the front with a small emblem, a thin gold line at the collar | Mail leggings with the tabard flap continuing down the middle | Leather boots with mail cuffs |
| **Iron** | Silver helmet with a **gold crest ridge** over the top, open face framed in leather | Bright plate breastplate with a centre ridge, plate shoulder guards over the arms, brown leather straps and belt | Plate thigh and knee guards over brown leather | Silver plate boots |
| **Copper** | Copper helmet covering the whole head, with carved panels | Rounded copper plates in horizontal bands, **green patina straps** and trim; no arm plates | A green cloth skirt over copper | Green-strapped boots with copper toes |
| **Gold** | Gold helmet with a **red cloth plume** and red side flaps | Ornate gold breastplate with a **red gem** in the centre, red-and-gold shoulder guards, gold-and-black belt | Gold scale pattern with **red gems at the knees** | Gold boots |
| **Diamond** | Cyan crystal helmet over a dark teal under-cap | Layered **cyan chevron plates** over dark leather, cyan shoulder plates | Cyan thigh plates over dark brown leather | Cyan crystal boots |
| **Netherite** | Fully closed **near-black helmet**, with gold drips at the lower edge | Dark grey plate with **gold chevron trim lines** across the chest and arms | Dark plate with the same gold trim lines | Dark plate boots with gold edges |
| **Turtle** | A green shell cap only, no other pieces | — | — | — |

How the pieces sit on the body:

- **Helmet:** covers the top, back and sides of the head. Most sets leave the face open with a 1-px rim around it. Only copper and netherite close it, and the netherite helmet still shows a dark slit at eye level.
- **Chest and arms:** covers the torso and both upper arms. Shoulder guards stick out 1 px beyond the arm outline. Chainmail and leather have full sleeves; plate sets cover the shoulders and leave leather showing below.
- **Leggings:** start at a **belt line at the waist** and cover the hips and legs down to the shins. The belt overlaps the bottom edge of the chestplate.
- **Boots:** cover the feet and lower shins and sit **on top of** the leggings, so the leggings' hem disappears into the boot tops.
- **Layering order** from the body outwards: base clothing, leggings, boots, chest and arms, helmet.
- **Trim tells the tiers apart:** leather and chain show cloth, iron shows leather straps, gold shows red cloth and gems, diamond shows dark leather between the crystal, and netherite shows gold lines. Keep that trim when you design your own sets, so each tier reads at sprite size.

---

## 2. Sprite rules

All the animal rules apply (pixel size, side view, outline, top-left light, animation, babies). On top of them:

### Proportions

| Type | Canvas | Proportions |
|---|---|---|
| Person or humanoid monster | 24×32 or 32×48 | A big head (about a quarter of the height), a square body, short legs. That chunky look matches blocky characters. |
| Tall monster (enderman type) | 32×64 | Thin, long arms and legs, a small head |
| Spider-like | 48×32 | Low and wide, with 4 visible legs per side, bent at the knee |
| Slime or blob | 16×16 to 32×32 | Rounded cube, with a lighter top-left and a darker core visible inside |
| Boss | 64×64 and larger | Exaggerated silhouette, glowing parts, extra details |

### Views

- Use a **side view or three-quarter view, facing right**, the same as the animals.
- Also draw a **front view** for portraits, dialogue and the character screen. The flat front-view references match that.

### Characters wearing armour

- Draw armour as **separate layers** on its own transparent sheet, at exactly the same size and pose as the character: helmet, chest and arms, legs, boots. The game stacks the layers, so one body can wear any combination.
- Every animation frame needs a matching frame for each armour layer.
- Armour adds 1 px of bulk at most around the body outline, so the silhouette stays the same.

### Making monsters read as hostile

- **Glowing eyes:** 1–2 px of a bright, saturated colour (red, cyan, magenta, acid green) with a 1-px darker halo.
- Muted or sickly skin tones: green-grey, bone, pale sand, deep purple.
- Torn edges, dripping or ragged cloth, and visible bones.
- Keep monsters a little darker than friendly characters, but still with a soft outline that's never black, so they read on dark ground at night.

### Animations

| Animation | Frames | Notes |
|---|---|---|
| Idle | 2–4 | Breathing bob. Monsters also sway or twitch. |
| Walk | 4–6 | Shambling (zombie), stiff (skeleton), skittering (spider) |
| Attack | 3–4 | Wind-up, strike and recover. The strike frame has the longest reach. |
| Hurt | 1–2 | Flash red for 1 frame |
| Death | 4–6 | Collapse, then dissolve into smoke particles (see the effects guide) |
| Talk or trade (people) | 2 | Head bob, hand gesture |

---

## 3. Prompts

### A) Master prompt

> **Style:** Hand-painted pixel-art game sprite of a [character], [side view facing right / front view], on a [W×H] transparent canvas with the feet on the bottom edge, fully opaque pixels, no anti-aliasing. Chunky proportions with a big head, a square body and short limbs. Warm rustic-medieval fantasy matching 32×32 item icons. Clothing and armour drawn as real costume pieces (leather, straps, belts, trim), each part with its own 3–4 shade ramp. Light from the top-left. A soft 1-pixel outline in a darker shade of each part's own colour, never black. Eyes are 1–2 px with a white highlight, or glowing for monsters.

Add **one** of these lines:

| Type | Add to the prompt |
|---|---|
| **Villager or trader** | "Friendly person in a simple robe in [profession colour] with an apron or sash, a tool or item in hand, warm skin tones, an expressive face." |
| **Undead** | "Sickly [green-grey / bone / pale sand] skin, torn leather and rags, dark sunken eye sockets with tiny glowing [colour] eyes, shambling posture with arms forward." |
| **Raider or bandit** | "Grey-beige skin, heavy dark brows, a dark tunic with a leather belt and a hood or cap, holding a crossbow or axe, a scowl." |
| **Creature** | "[Spider / slime / tall shadow creature], a dark body with bright glowing [colour] eyes, a strong recognisable silhouette." |
| **Armour layer** | "Only the [helmet / chestplate and arms / leggings / boots] of [material] armour, on a transparent background, exactly fitting a [W×H] character sprite in the same pose, metal plates over leather straps, matching the [material] armour icon." |
| **Boss** | "Large [description], 64×64 or larger, exaggerated horns, armour or wings, glowing core or eyes, more detail than normal monsters but the same outline and light rules." |
| **Portrait** | "Front-view head-and-shoulders portrait, 32×32, for a dialogue box, the same colours as the sprite." |

### B) Reference images to attach

| Prompt type | Attach |
|---|---|
| Any monster or person | `01 Monsters and characters.png` |
| Armour layers, armoured characters | `02 Armour worn on the body.png`, plus `03 Armour (front view).png` from the `1 Items` folder so the icons and worn armour match |
| Any character (for outline and light) | One item image, such as `02 Weapons.png` from the `1 Items` folder |

### C) Checklist

- **Player character:** base body, 4 armour layers × 6 materials, 2–3 skin tones, the same animations as monsters plus tool use (mine, chop, swing)
- **People:** villager professions (farmer, smith, librarian, fisher, shepherd, cleric), trader, guard
- **Hostile people:** bandit, archer, axe raider, mage
- **Undead:** zombie, desert variant, drowned variant, skeleton archer, armoured skeleton
- **Creatures:** spider, cave spider, slime, shadow creature, bats, ghosts
- **Bosses:** 2–4, each with its own colour theme
