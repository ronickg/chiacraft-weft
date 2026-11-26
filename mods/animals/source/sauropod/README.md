# Sauropod's source

One model for *Cedarosaurus weiskopfae*, the Yellow Cat floodplain's brachiosaurid, which also
stands for *Dongbeititan* in the fern forest (`notes/ECOLOGY.md`, "The food webs and the species to
add"; `reports/ecology/Cretaceous food webs.md`: the Yellow Cat row "*Cedarosaurus weiskopfae*
(brachiosaurid), *Moabosaurus* (turiasaur)", ~10 t, high browse), made with the `blocky-model`
skill ([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from
research, at the house density (32 texels a block: the skull's side, which carries the eye, is 16
by 17 texels). The recipe is [docs/creatures.md](../../../../docs/creatures.md).

## Rebuild

```
nice -n 15 ~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`sauropod.py` lays out, paints and animates it into `out/` (not committed) and fits its lying down
(legs folded, belly and tail on the ground) and its drinking (the mouth at the water) with forward
kinematics; `build.py` checks flicker, joints, pokes and gait and exports
`models/sauropod.geo.json`, `animations/sauropod[_lie].animation.json` and
`textures/entity/sauropod/sauropod.png` into `../../assets`, and the coat layers into
`../../genetics/layers/sauropod`. Look at it with the skill's `render.py`
(`render.py out/spec.json out/texture.png shot.png --ppb 40 --views 3q,side,front,head`, with
`--anim idle --t 9` for drinking or `--anim lie --t 0`; its `3q` and `head` views look up from
below, so pass your own negative pitch to `render.render` to see the back).

## Research (2026-11-04)

- **The animal**: *Cedarosaurus weiskopfae* Tidwell, Carpenter & Brooks 1999, from the Yellow Cat
  Member of the Cedar Mountain Formation, eastern Utah (Valanginian, about 139-135 million years
  ago), a brachiosaurid macronarian known from a partial skeleton (back and tail vertebrae, ribs,
  a forelimb, hind limb bones, 115 gastroliths found with it). Its forearm bones are more slender
  than those of its neighbor *Venenosaurus*
  ([Wikipedia](https://en.wikipedia.org/wiki/Cedarosaurus); Tidwell, Carpenter & Brooks 1999,
  *Oryctos* 2; Sanders, Manley & Carpenter 2001 on the gastroliths).
- **Size**: about 15 m long and 10 t (Paul 2010, *The Princeton Field Guide to Dinosaurs*; the
  report's "~10 t (estimate)"). The skeleton is far from complete, so the build is a
  brachiosaurid's after *Brachiosaurus* and *Giraffatitan* scaled to 15 m (about 0.68 of a 22 m
  *Giraffatitan*): the forelimbs longer than the hind limbs (the humerus at least 0.9 of the
  femur), so the shoulders stand well above the hips and the back slopes down to them; a deep,
  fairly short trunk with a small shoulder hump; a long neck of elongated vertebrae leaving the
  trunk at a sharp angle and held high in a gentle S; a tail short against the neck and body; a
  small, tall skull (about 0.7-0.8 m in *Brachiosaurus*, so about 0.5-0.55 m here) with a long,
  blunt snout, the bony nasal arch over and in front of the eyes, the nostril openings high on it;
  columnar limbs, the hand a horseshoe-shaped column with one thumb claw, the foot with big claws
  on its inner toes ([Wikipedia, *Brachiosaurus*](https://en.wikipedia.org/wiki/Brachiosaurus);
  Taylor 2009, *Journal of Vertebrate Paleontology* 29; D'Emic & Carrano 2020 on the skull).
- **Life**: herds (inference, the report); high browse: *Araucaria*-type conifers, with horsetails
  and ferns low down (Hummel et al. 2008, *Proc. R. Soc. B* 275); gathered at the last water in
  the dry season (the report's floodplain row). Eaten as young by the Utahraptor and a large
  allosauroid; its carcass a giant meal for the dead.
- **Colors**: no record. Chosen as a big browser's: a grey-brown to olive hide, a darker olive-grey
  along the top, pale buff below and in a stripe down the throat, faint darker bands across the
  back and the tail and in rings down the neck, dark horn on the soles, the thumb claw and the toe
  claws; a dark amber eye.

Decided: the real size (about 990 units, 15.5 m, from the snout to the tail's tip measured along
the neck, body and tail; 12 m from the snout to the tail's tip standing, as the neck rises), the
shoulders 262 up and the hips 214 (the front legs' joint 210 up, the hind legs' 168), the head's
joint 522 up (8.2 m; the head 9.1 m up browsing), the head about 1.6 times life (60 units long against a
real skull of about 0.55 m; the house look); the body as a core with masses (the hips, a slung
belly, a deep ribcage in front, a back slab sloping up to the shoulders turned 14 degrees, the
shoulder girdle's muscle widening the top); the neck four turned segments on four bones
(`neck_0`..`neck_3`, each slimmer than the one before) rising 48 to 60 degrees, the tail five
turned segments drooping a little, held off the ground; pillar legs (the front ones 34 wide and
longer, the thighs 42), a round horseshoe hand with a thumb claw on its inside, the feet with three
toe claws split by grooves; no scales painted (at 32 a block they are noise), only the faint bands;
the eye a 3 by 2 design (a catchlight, a dark pupil, a dark amber iris marked for the genes),
mirrored on the right side, with eyelids from the same corner; the nostrils two slits on top of the
dome. Animations: idle (browsing high, the neck reaching up and swung from side to side across the
trees, the head tipped up and chewing, then down to drink, gulping, and up again; two blinks;
16 s), walk (a slow lateral walk on all fours, the swing sized so a planted foot moves at the
ground's speed, 2.4 s), run (a faster amble on the same footfalls, higher steps, the neck a little
lower, 2.0 s), lie (on its belly, the front legs folded camel-fashion with the forearms under the
chest, the hind legs folded, the neck lowered, the tail along the ground; two blinks; 10 s).

Behaviour clips (11-05, `src/anim.rs`): stare, graze (the neck swept low, the mouth at the ground
plants), drink, watch (the neck raised high, turning slowly), sleep (lying, the neck turned round and
lowered to the ground, the head laid on it; also in the lying file), approach and charge (a ponderous
head-low amble, the neck level and thrust forward) loop; sniff, look, scratch (the flank rubbed on a
trunk, the body swaying over planted feet), stretch, alarm (a bellow, the tail whipped), rear, play
(a tail swish), groom (the neck arched round to the flank), nuzzle and flinch play once; death (rolled
onto its side, slow) holds. It stands its ground (10 t, over `roles::STANDS_KG`), so it rears: up on
the hind legs 40 degrees with the tail laid on the ground behind as the tripod's third leg (fitted),
the hind feet planted, the hips kept over them. Fitted in `fit_clips`. The idle's drink was fitted on
the whole head, and its top touched the water with the head upside down: refitted on the lips with
the crown above them (11-05); the idle's and lie's chest breath sank the forefeet and the drinking
snout (0.7-2.2 units) and is gone. `build.py` asserts the loops, every standing clip's feet within 0.3
of the ground, the gaits' and rear's grounding, the mouth at the ground grazing and drinking, nothing
below -0.3 in any clip, and death on its lowest point each frame.
