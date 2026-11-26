# The hare's source

One model for the brown hare (*Lepus europaeus*) and the mountain hare (*Lepus timidus*), the
living world's mid-sized prey (`notes/ECOLOGY.md`, "The food webs and the species to add"), made
with the `blocky-model` skill ([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md))
from research alone, at the house density (64 texels a block). The recipe is
[docs/creatures.md](../../../../docs/creatures.md).

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`hare.py` lays out, paints and animates it into `out/` (not committed) and fits the crouch, the
form and grazing to the ground; `build.py` checks flicker, joints, pokes and gait and exports
`models/hare.geo.json`, `animations/hare[_lie].animation.json` and
`textures/entity/hare/hare.png` into `../../assets`, and the coat layers into
`../../genetics/layers/hare`. For Blockbench and a GLB:
`python ../../../../.claude/skills/blocky-model/scripts/bb_build.py out`. The other eye designs
(`HARE_EYES=hare_gold`, `hare_dark`, `hare_bead`; `HARE_RING=1` for the pale ring) are there to
compare in the viewer.

## Research (2026-11-03)

- **The brown hare**: 55-65 cm head and body (53-76 in one review), the tail 7.5-14 cm, the ears
  9.4-11 cm from the notch, the hind foot 14-16 cm, 3.5-5 kg; grizzled yellow-brown on the back,
  rufous on the shoulders, legs, neck and throat, the belly white, the ear tips and the top of the
  tail black. Mostly nocturnal; lies up by day in a form (a shallow scrape) and outruns what
  hunts it (70 km/h). Breeds January to August, a 6-week gestation, 2-4 litters of 3-4 leverets;
  mature at 4-8 months; up to 12 years
  ([Wikipedia](https://en.wikipedia.org/wiki/European_hare)).
- **The mountain hare**: 45-65 cm, the tail 4-8 cm, 2-5.3 kg (females a little heavier);
  brown in summer, white or nearly so in winter (grey on the Faroes; Irish ones stay brown), the
  tail white all year, which tells it from the brown hare. Tundra, boreal forest, moors and the
  Alps (700-3,800 m). The red fox is its main enemy (up to 90% of a fox's food where they
  overlap), the golden eagle regionally, and eagle-owls, lynx, wolves, martens and stoats
  ([Wikipedia](https://en.wikipedia.org/wiki/Mountain_hare)).
- **The food webs** (`reports/ecology/Overworld food webs.md`, `overworld_web.json`): the hare is
  the prey of the coyote, fox, wolf, eagle, owl, lynx and jaguar, its young the cat's and the
  chihuahua's; it eats grass, forbs, and twigs and bark in winter; alone; common.
- **In block games**: Minecraft's rabbit (`bedrock-samples/.../rabbit.geo.json`) is a body box,
  haunches, long flat hind feet, thin front legs, a small head, two thin ear boxes and a tail: the
  hind feet and the haunch are what read as a rabbit. Hytale has rabbits as small critters.

Decided: real proportions for the body (about 58 cm nose to rump, 30 cm at the shoulder), the
head and ears a little bigger than life; a hare's crouched stance in every pose, the back arched up
to a high round rump and the head held up on a raised neck; amber eyes on the sides of the head
(Ronald's pick; the pale ring round them read as goggles at this size); animations idle (the nose
twitching, the ears turning one at a time, grazing and chewing), walk (lolloping), run (bounding,
the ears laid back) and lie (pressed into its form, the ears flat along the back).

## How it's made

- **Legs**: modeled straight down from the body; `crouch` folds them in every pose (thigh forward,
  shin back, the long hind foot flat), fitted so the hind feet stand on the ground with the front
  paws and the heel sits a little behind the hip, and raises the body to stand on the ground.
- **Ears**: thin steps narrowing to a rounded tip, lit as one piece, with a cut-out panel through
  the middle for the outline; black tips (both hares have them), a pale inside.
- **Coat**: the brown hare's colors as rules (`area`), recolored per animal from its genes in the
  game (`genetics/hare.json`, `genetics/mountain_hare.json`, `core/src/coats.rs` `hare`): grizzled
  agouti grey-brown to rufous, the mountain hare's white coat, the tail's black top (its own
  extremity layer), black very rarely. No moult yet: a white mountain hare is white all year.
- **Face**: eyes on the side faces, with eyelids that swing down about z to blink; a split lip
  with a hint of the incisors; the nose on its own bone, twitching.
