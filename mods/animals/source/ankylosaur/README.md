# The ankylosaur's source

One model worn by two species: *Gastonia burgei*, a polacanthid and the floodplain's commonest
animal, and *Pinacosaurus grangeri*, an ankylosaurid of the Gobi scrubland (`notes/ECOLOGY.md`,
"The food webs and the species to add": the Cretaceous's tier 2, "an ankylosaur, *Pinacosaurus* in
the Gobi and *Gastonia* in Utah on one model"). Made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research, at
the house density (32 texels a block: the skull's side, which carries the eye, is 14 by 12 texels).
The recipe is [docs/creatures.md](../../../../docs/creatures.md).

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`ankylosaur.py` lays out, paints and animates it into `out/` (not committed) and fits cropping,
lying down, the tail and the chin to the ground; `build.py` checks flicker, joints, pokes and gait
and exports `models/ankylosaur.geo.json`, `animations/ankylosaur[_lie].animation.json` and
`textures/entity/ankylosaur/ankylosaur.png` into `../../assets`, and the coat layers into
`../../genetics/layers/ankylosaur`. Look at it with the skill's `render.py`
(`render.py out/spec.json out/texture.png shot.png --ppb 100 --views 3q,side,front,head`, add
`--anim idle --t 1` or `--anim lie --t 0`; a blink is `--anim idle --t 2.0`).

## Research (2026-11-04)

- ***Gastonia burgei***: about 5 m long and 1.9 t (Paul 2010; Kirkland 1998 said about 6 m); the
  mounted skeleton is 4.59 m long, the hips 1.12 m up. A fairly small, somewhat pointed skull
  (the holotype 295 by 283 mm), a toothless beak, small squamosal and jugal horns. Two keeled
  quarter rings over the neck; about five pairs of large, flat, triangular spikes along each side of
  the chest, grooved behind; flat spikes on the rump; a sacral shield of fused osteoderms in
  rosettes over the hips; triangular spikes along the tail's sides and oval plates on top; small
  ossicles between. No tail club. A very broad, flat rump and a protruding belly on short, strong
  legs (Kirkland 1998, [Wikipedia](https://en.wikipedia.org/wiki/Gastonia_(dinosaur))). The
  *Utahraptor* type was found with it; bonebeds suggest groups (`reports/ecology/Cretaceous food
  webs.md`).
- ***Pinacosaurus grangeri***: about 5 m long grown (Hill et al. 2003), 1.9 t (Paul 2010), a very
  flat torso; an adult skull about 30 cm long, covered in bony tiles (caputegulae, which give it its
  name); two cervical half rings of keeled segments; rows of small oval keeled osteoderms on the
  back; flat, recurved triangular spikes on the sides of the rump and tail; no continuous shield
  over the hips (Arbour 2014); a small tail club on a stiffened handle. Juveniles in groups (24+
  skeletons, more than any other ankylosaur's), a group at Bayan Mandahu killed together, most
  found lying on their bellies with the legs tucked under (Currie et al. 2011,
  [Wikipedia](https://en.wikipedia.org/wiki/Pinacosaurus)).
- **Colors**: no record for either.
- **Sources**: Kirkland 1998, *New Mexico Museum of Natural History and Science Bulletin* 14;
  Paul 2010, *The Princeton Field Guide to Dinosaurs*; Arbour 2014; Currie, Badamgarav, Koppelhus,
  Sissons & Vickaryous 2011, *Acta Palaeontologica Polonica*
  (https://app.pan.pl/article/item/app20100055.html); the two Wikipedia articles above, read
  2026-11-04.

Decided:
- **Size**: the real one: 314 units from the beak to the club (4.9 m), the top of the hip shield 81
  up (1.27 m), the body 117 wide across the flanks (1.8 m; 160 with the spikes), the belly 22 up.
  The head a little bigger than life (38 wide, 42 from its back to the beak).
- **One look for both**: Gastonia's armor (big flat flank spikes, two tall shoulder spikes a side,
  a mosaic shield over the hips, paired flat spikes down the tail) with **a small, low club at the
  tail's end** (Pinacosaurus's), because a club is what reads as "ankylosaur"; the club is part of
  the last tail segment's bone, not its own element.
- **Spikes**: the shoulder ones as two tapering boxes each; the flank and tail ones as flat cut-out
  blades (they are flat in the fossils), drooping 36 degrees with the flank so they show from the
  side and from above.
- **Colors** (rules in `area`, no record): an earthy olive-brown hide, darker brown between the
  back's plates and over the brow, the plates a paler horn with a lighter keel, the spikes and
  squamosal horns pale horn toward their tips, the underside a pale dun, the beak a mid horn.
- **Eyes**: small, high on the head's sides: a 3 by 2 design (a catchlight up and forward, a dark
  pupil, an amber-brown iris marked `iris` so genes recolor it), mirrored on the right side, no
  halo; eyelids sized from the same `eye_corner`.
- **Animations**: idle (cropping low plants, the neck and head down to the ground, chewing; then up
  to look about; the tail swaying; two blinks), walk (a slow, heavy walk), run (a heavy trot), lie
  (belly down, the legs folded under, the tail and chin on the ground, as the young Pinacosaurus
  were found; two blinks).
