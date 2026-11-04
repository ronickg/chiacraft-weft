# The fox's source

One model for the red fox (*Vulpes vulpes*) and the Arctic fox (*Vulpes lagopus*), the living
world's small hunter of the north and the tundra (`notes/ECOLOGY.md`, "The food webs and the species
to add"), made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research
alone, at the house density (64 texels a block). The recipe is
[docs/creatures.md](../../../../docs/creatures.md); how it hunts is `src/hunt.rs`.

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`fox.py` lays out, paints and animates it into `out/` (not committed) and fits lying down to the
ground; `build.py` checks flicker, joints, pokes and gait and exports `models/fox.geo.json`,
`animations/fox[_lie].animation.json` and `textures/entity/fox/fox.png` into `../../assets`, and the
coat layers into `../../genetics/layers/fox`. `FOX_COAT=arctic` paints an Arctic fox's winter white,
to compare in the viewer (the game paints every fox's coat from its genes).

## Research (2026-11-03)

- **Red fox**: about 35-45 cm at the shoulder, 65-90 cm long, the tail about 70% of the head and
  body and reaching the ground; 5-10 kg in the north; orange-red with white below, black stockings
  and ear backs, a white-tipped brush; oval, vertical pupils; silver and cross morphs common in some
  places. Pairs or small families; small rodents first, rabbits, birds, insects, fruit; leaps fences
  over 2 m ([Wikipedia](https://en.wikipedia.org/wiki/Red_fox)). The mousing pounce: a leap up and
  over, the front paws stabbing down through grass or snow.
- **Arctic fox**: 25-30 cm at the shoulder, about 55 cm long, 3.5 kg (vixens 2.9); short, thick ears;
  white in winter (the blue morph about one in ten worldwide); its winter coat keeps it at ease to
  -7 degrees. Breeds in April and May after a 52-day gestation; 6-18 pups in a lemming year; 3-4
  years on average. Lemmings first, voles, birds, eggs, fish, carrion; eaten by eagles, snowy owls,
  wolves, bears and red foxes ([Wikipedia](https://en.wikipedia.org/wiki/Arctic_fox)).
- **The food webs** (`reports/ecology/Overworld food webs.md`): meat 50%, fruit 20%, insects 15%,
  carrion 15%; prey from 0.5% to 80% of its own mass; the coyote's, wolf's and eagle's prey.

Decided: the red fox at real size, the Arctic fox the same model drawn smaller (size genes) with
shorter ears (the ears' build trait); a bushy brush; amber eyes with tall pupils; wild hunters, not
tameable; animations idle (ears turning to sounds), walk (trotting), run (galloping), lie (the head on
its paws, the brush along its side), and its attacks, pounce (the mousing leap) and bite.

## How it's made

- **The brush** is three segments, each 0.4 a side thinner than the one before, at one pitch (the
  bends come from the animations), the tip painted white.
- **Coats from genes**: the stockings and ear backs are its points (black on a red fox, white on a
  white Arctic fox); a cross fox's cross is a saddle mark over the shoulders, a silver fox black
  speckled with silver.
- **Hunting** is the main session's `hunt.rs` (every simple creature gets the goal in `wild.rs`; it
  does nothing for those that don't hunt): seen in the game, a hungry red fox took a wood mouse and
  then a hare, and ate its fill.
