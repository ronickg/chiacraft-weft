# The Utahraptor's source

The biggest raptor and the animals mod's mount: a feathered, heavily built Utahraptor at its real
size, made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research and
the Velociraptor's source ([../raptor](../raptor/README.md)): the same skeleton and animations,
heavier everywhere. The recipe is [docs/creatures.md](../../../../docs/creatures.md).

## Rebuild

```
../dog/chihuahua/.venv/bin/python build.py      # any Python with ../dog/chihuahua/requirements.txt
```

`utahraptor.py` lays out, paints and animates it into `out/` (not committed) and fits its stance
and resting pose to the ground; `build.py` checks flicker (rest and every keyframe), joints and
gait and exports `models/utahraptor.geo.json`, `animations/utahraptor[_rest].animation.json` and
`textures/entity/utahraptor/utahraptor.png` into `../../assets` (scale 0.83: 1.5 blocks at the
hip, about 4.7 long, 2.5 to the top of the head). For Blockbench and a GLB:
`python ../../../../.claude/skills/blocky-model/scripts/bb_build.py out`.

## Research (2026-11-01)

- **The real animal** (Utahraptor ostrommaysi, Early Cretaceous Utah): the largest specimen about
  4.65 m long, 1.5 m at the hip and 280 kg (2012 estimate); others 5-7 m and up to 500 kg; half of
  it tail ([Wikipedia](https://wikipedia.com/wiki/Utahraptor),
  [A Dinosaur a Day](https://a-dinosaur-a-day.com/post/186705462685/utahraptor-ostrommaysi),
  [USU Prehistoric Museum](https://eastern.usu.edu/prehistoric-museum/_files/exhibits/handout-utahraptor.pdf)).
- **Build**: heavier than other dromaeosaurs, strong legs (an ambush hunter rather than a chaser),
  a boxy, elongated skull like Velociraptor's with very large orbits (big eyes), small serrated
  teeth, a sickle claw of 22-24 cm on each foot.
- **Feathers**: not found on it, but certain from its relatives; quill knobs on the large
  Dakotaraptor show even big dromaeosaurs had wing feathers; Carpenter's 2017 restoration covers
  it like an emu with vulture-like wing feathers
  ([Deposits](https://depositsmag.com/2017/09/05/a-new-look-for-utahraptor/)).

- **Restorations**: Carpenter's 2017 restoration (for the new skeletal mount) covers it in simple,
  hair-like feathers modelled on an emu's, broad wing feathers like a vulture's, and alligator
  eyes. No source supports a crest or a mane.
- **In games**: Minecraft has none; Hytale's raptors are mods (PreHystoric Beasts; a rideable "Cave
  Raptor" in MountableTrexAndRaptor), their look undocumented
  ([CurseForge](https://www.curseforge.com/hytale/mods/mountabletrexandraptor)).

Decided: real size (1.5 at the hip, about 4.7 long), the Velociraptor's skeleton heavier everywhere
(a wider, deeper body, a thick neck and tail, thighs 8 wide and shins 5.2, legs set wide); a big,
boxy skull (9 tall) with a deep, straight snout over a shorter jaw; recognizable by its shagginess
(emu), its broad dark-tipped wings (vulture), its big alligator eyes under a heavy brow and its
three-toed feet with the raised sickle claw. Animations as the Velociraptor's: idle (bird-like
head jerks), walk, run, rest, screech.

## How it's made

- **Eyes** (`utah`): 8 wide and 5 high on each side of the head, an amber iris lighter below, a
  vertical slit pupil, a dark ring, a cool highlight, under a heavy dark brow. Five high so the
  eyelids (as wide as the eye is high, lying inside the head when open) don't meet in the middle.
- **Face**: nostrils as slits on the sides near the tip, not on the front (a dark pair there,
  above the teeth, read as a second face); the jaw in the face's color, pale only underneath (all
  pale, it was a grin from the front).
- **Feet**: three toes as boxes each thinner than the foot (III long, IV outside it, both on the
  ground; II inside, held up), claws on their tips, the sickle claw on II tilted up 30 degrees.
- **Feathers**: shaggy locks over each thigh (a cut-out panel on the hips, 0.6 outside the
  thighs' side planes), broad wing panels with long primaries at the back and pointed lock tips,
  a fanned tail with pointed tips; the body coat painted as soft streaks.
- **Coats**: six, on the same layout (`out/coats/<coat>.png`, `UTAH_COAT` picks the exported one):
  `umber` (barred), `emu` (speckled grey-brown), `desert` (sandy with rosettes), `forest` (dark
  olive-brown with wide bands), `ash` (plain slate with white wing and tail bars, like a vulture),
  `rufous` (rusty with a dark saddle and a pale face). Earthy colors only, as on big ground birds.
- **Flicker fixes** for the heavier build: the arms' undersides above the chest's, the jaw 0.6
  below the skull, the outer toes 0.3 inside the shins' side planes (the folded rest brings them
  together), the chest's front shortened so the neck never meets it, and the resting toes 5
  degrees off flat so they never lie in the belly's plane.
- **Resting**: legs fold further than the Velociraptor's, fitted over a wider range.
- **Neck into the head**: the neck (6.4 thick, 12 long) is slimmer than the skull and ends deep
  in it, so no corner of it pokes out of the head when the head turns and tilts; `build.py`
  asserts `ground.pokes` is empty along the body's axis.
