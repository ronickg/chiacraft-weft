# Psittacosaurus's source

One model for *Psittacosaurus lujiatunensis*, the Jehol fern forest's small plant eater and the
Utahraptor's prey there (`notes/ECOLOGY.md`, "The food webs and the species to add": the
Cretaceous's tier 2), made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research, at
the house density (32 texels a block: the skull's side, which carries the eye, is 6 by 6 texels).
The recipe is [docs/creatures.md](../../../../docs/creatures.md); how it lives is
`src/psittacosaurus.rs` and `src/wild.rs`.

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`psittacosaurus.py` lays out, paints and animates it into `out/` (not committed) and fits its
stance, cropping and lying down to the ground; `build.py` checks flicker, joints, pokes and gait
and exports `models/psittacosaurus.geo.json`, `animations/psittacosaurus[_lie].animation.json` and
`textures/entity/psittacosaurus/psittacosaurus.png` into `../../assets`, and the coat layers into
`../../genetics/layers/psittacosaurus`. Look at it with the skill's `render.py`
(`render.py out/spec.json out/texture.png shot.png --ppb 400 --views 3q,side,head`).

## Research (2026-11-04)

- **Size**: *P. lujiatunensis* about 1.3-1.9 m long and 20-30 kg as an adult (*P. mongoliensis*
  over 20 kg); hips about half a metre up. Grown ones walked on their hind legs, the arms short
  (hatchlings went on all fours: Zhao et al. 2013). A short, deep, nearly square skull with a
  parrot-like beak and a horn out of each cheek (the jugals)
  ([Wikipedia](https://en.wikipedia.org/wiki/Psittacosaurus); `reports/ecology/Cretaceous food
  webs.md`).
- **Skin and colors**: known, the only dinosaur's from its own skin. The Senckenberg specimen
  (SMF R 4970) is countershaded, dark brown above and pale below, for a shaded forest floor;
  darker flecks and patches on the shoulders and back, dark cheek horns (Vinther et al. 2016,
  *Current Biology*). A fringe of long bristles, up to about 16 cm, stood along the top of the
  tail's base (Mayr et al. 2002).
- **Life**: the Yixian's dominant dinosaur, the commonest one there. Juveniles in groups of the
  same age, without adults (Zhao et al. 2014); a nest of 24 young with one adult (Meng et al.
  2004). Tough plants, ground with stomach stones even as hatchlings (Wang et al. 2025). Growth
  lines put it grown at about 9 to 11 (Erickson et al. 2009).
- **Hunted by**: *Repenomamus*, a mammal with a young one in its gut (Hu et al. 2005), and
  theropods; here the Utahraptor and the wyvern.

Decided: the real size (101 units long, the hips about 31 up), the head a little bigger than
life; its colors as the fossil's, the flecks a texel or more; the bristles one cut-out panel on
each of the tail's first two segments, a quill every other texel; a dark-brown eye (no record),
3 by 2 texels; wild, in herds; animations idle (tipped forward at the hips to crop ferns, the tail
held level, then up to look about), walk, run (leaning forward), lie (like a bird: the legs folded
under, belly and tail down).
