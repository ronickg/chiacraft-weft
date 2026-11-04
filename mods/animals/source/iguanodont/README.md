# Iguanodont's source

One model for *Iguanacolossus fortis*, the Yellow Cat floodplain's big plant eater and what the
Utahraptor was trapped hunting at Stikes (`notes/ECOLOGY.md`, "The food webs and the species to
add"; `reports/ecology/Cretaceous food webs.md`: the row stands for the Yellow Cat's iguanodonts and
the Yixian's *Jinzhousaurus*), made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research, at
the house density (32 texels a block: the skull's side, which carries the eye, is 20 by 16 texels).
The recipe is [docs/creatures.md](../../../../docs/creatures.md).

## Rebuild

```
nice -n 15 ~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`iguanodont.py` lays out, paints and animates it into `out/` (not committed) and fits its lying
down (legs folded, belly and tail on the ground) and its grazing (the beak at the ground) with
forward kinematics; `build.py` checks flicker, joints, pokes and gait and exports
`models/iguanodont.geo.json`, `animations/iguanodont[_lie].animation.json` and
`textures/entity/iguanodont/iguanodont.png` into `../../assets`, and the coat layers into
`../../genetics/layers/iguanodont`. Look at it with the skill's `render.py`
(`render.py out/spec.json out/texture.png shot.png --ppb 60 --views 3q,side,front,head`, with
`--anim idle --t 1` or `--anim lie --t 0`).

## Research (2026-11-04)

- **The animal**: *Iguanacolossus fortis* McDonald, Kirkland, DeBlieux, Madsen, Cavin, Milner &
  Panzarin 2010, from Don's Ridge in the lower Yellow Cat Member of the Cedar Mountain Formation,
  eastern Utah (Valanginian, about 139-135 million years ago). Known from one partial skeleton
  (UMNH VP 20205): a predentary, a right maxilla, the squamosal and quadrates, teeth, the axis,
  13 back vertebrae and ribs, 5 tail vertebrae with chevrons, the scapula, ilium, pubis, a fibula
  and two metatarsals. A robust styracosternan iguanodont, near *Dakotadon* and *Lanzhousaurus*;
  the describers restore it as "a large, somewhat ponderous beast with robust limbs"
  ([McDonald et al. 2010, *PLoS ONE*](https://pmc.ncbi.nlm.nih.gov/articles/PMC2989904);
  [Wikipedia](https://en.wikipedia.org/wiki/Iguanacolossus)).
- **Size**: about 9 m, like *Iguanodon bernissartensis*; about 5 t (Paul), 1-5 t across estimates
  (the report); the game takes ~4 t. Its skull, hands and most limbs are unknown, so the build is a
  styracosternan's (as *Iguanodon* and *Mantellisaurus*): a long, deep, horse-like head with a
  toothless beak (premaxilla above, predentary below) in front of the cheek teeth, a thick neck,
  a deep body, the hips the highest point, long robust hind legs on three hoofed toes, arms about
  three quarters as long with the three middle fingers bound into a hoof and a thumb spike, and a
  long tail held out level, stiffened by ossified tendons along the spine. Mostly on all fours
  when walking slowly, able to rise on its hind legs.
- **Life**: herds (inference from relatives' bonebeds); browse up to about 4 m: ferns, horsetails,
  conifers, cycadophytes (inference). Eaten by the Utahraptor (the Stikes block: a pack caught in
  quicksand with at least one iguanodont) and a large allosauroid known from teeth.
- **Colors**: no record. Chosen as a big floodplain grazer's: olive-dun above, a darker olive-grey
  along the top, pale buff below, faint darker bands across the back running down the flanks and
  tail, dark horn on the beak, hooves and thumb spike; a brown-gold eye.

Decided: the real size (575 units from the beak to the tail's tip, the hips 140 up, the back 164
at its highest), the head about 1.6 times life (81 units against a real skull of about 0.8 m; the house look); on all fours in every
animation; the body as a core with masses (hips, a slung belly, a deep chest, the shoulders, a
thick neck root under a slimmer neck); chunky limbs (thighs 30 wide, the arms 22), the hands with
a hoofed front and a thumb spike on the inside, three hoofed toes on the feet; the tail five
segments swung from its base, the later ones barely bending (stiff); scales only as faint big
rounded plates on the body, tail and thighs (smaller ones are noise at 32 a block); the eye a
3 by 2 design (a catchlight, a dark pupil, a brown-gold iris marked for the genes), mirrored on
the right side, with eyelids from the same corner; animations idle (cropping low plants, then the
head up to look about, chewing, the tail swaying a little, two blinks; 10 s), walk (a slow lateral
walk on all fours, 1.6 s), run (a heavy trot, 1.0 s), lie (on its belly, the legs folded under, the
tail along the ground, two blinks; 8 s).
