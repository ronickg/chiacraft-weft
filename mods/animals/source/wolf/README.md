# The wolf's source

The grey wolf (*Canis lupus*), the top hunter of the north's forests, taiga, tundra and mountains,
and the dog's ancestor, so a natural tame (`notes/ECOLOGY.md`, "The food webs and the species to
add", tier 1): a full creature, made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research
alone, at the house density, on the coyote's build ([../coyote](../coyote), itself the fox's).

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`wolf.py` lays out, paints and animates it into `out/` (not committed) and fits lying down and
sitting to the ground; `build.py` checks flicker, joints, pokes and gait and exports
`models/wolf.geo.json`, `animations/wolf[_lie|_sit].animation.json` and
`textures/entity/wolf/wolf.png` into `../../assets`, and the coat layers into
`../../genetics/layers/wolf`.

## Research (2026-11-03)

- **Grey wolf** (*Canis lupus*): 66-90 cm at the shoulder (about 80), 1.05-1.6 m long with a 29-50 cm
  tail; males 30-80 kg (about 45), females about a fifth lighter; the biggest in the north. A broad
  head and muzzle, shorter, rounder ears than a coyote's, long legs, big paws, a bushy tail carried
  low. Grizzled grey with a darker saddle, buff legs and ears, a cream muzzle, throat and belly, the
  tail tipped dark; black wolves (a dominant black from dogs long ago) are common in the forests,
  Arctic wolves white. Amber-yellow eyes. Packs of 4-8, a family; only the leading pair breeds:
  mating January to March, 63 days, 4-6 pups (up to 11); breeding at about 22 months; five to seven
  years in the wild, 12-16 kept. Hoofed animals above all (deer, moose, reindeer, boar, bison
  calves), hares and beavers, carrion; the NPS's Yellowstone wolves eat about 3.25 kg a day.
  Its howl gathers the pack.
- **The food webs** (`reports/ecology/Overworld food webs.md`): meat 85%, carrion 15%; prey from 5%
  to 400% of its mass (a pack takes moose and elk); crepuscular and nocturnal; packs.

Decided: 80 cm at the shoulder and 1.2 m from the nose to the rump: the fox's layout warped (the
legs longer by 3 units, everything 1.7 times bigger), shorter ears, a bushier tail carried low;
amber eyes with round pupils and front lids that close fully. A full creature: tamed by feeding as
the cat is (twice the feedings), then commands; animations idle (ears to sounds, then a long howl),
walk (trot), run (gallop), sit (fitted: the front paws, the haunches and the hind feet on the
ground, the tail laid back), lie (the head on its paws), and the attacks pounce and bite.

## How it's made

- **Warped from the fox** (`warp`, `unwarp`), as the coyote; the eyelids a quarter texel bigger all
  round than the eye and just in front of the face, so a closed lid hides it from any side.
- **Coats from genes**: the buff parts its points, the saddle a mark; black (dominant, commoner in
  the forest) and white (recessive, commoner on the snow) wolves.
