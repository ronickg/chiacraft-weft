# The birds' build

What the wave-two birds share (`notes/ECOLOGY.md`, tier 2): the small bird, the owl, the eagle,
the vulture and the gull. Made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) on the raven's build
([../raven](../raven)), which passed the review checklist.

- `bird.py` lays a bird out in the raven's units, shaped by knobs (`DEFAULTS`: its size `K`, its
  legs `D`, the head, the bill and a hooked tip, the body's width, the tail's length and end, the
  wings' arm, hand, chord and tip, feathered legs, ear tufts, eyes on the sides or the face, the
  stance), then warps it into its own size (`warp`: the legs below `LEG_Y` stretched, everything
  `K` times bigger). A bird too small for the raven's gaps (the small bird) gives its own `layout`
  in its own units instead. Its colors are rules in the bird's module (`area`, in the raven's
  units), painted with the skill's light; it paints both models, standing (wings folded) and
  flying (wings open), on one texture, and records the coat layers.
- Animations: idle (looking about in holds and snaps; an owl's head turned far round), walk (or
  hops), run (hopping), and the flying model's wingbeat, which a big bird holds open in a glide
  part of the cycle. Blinks in idle: side eyelids for side eyes, front lids for an owl's.
- `export.py <model>` builds `../<model>/<model>.py`'s `CFG`, checks flicker (at rest and in every
  keyframe), joints, pokes along the body's axis, thin lines (`linecheck`) and a forward walk, and exports
  `models/<model>[_fly].geo.json`, their animations and `textures/entity/<model>/<model>.png`, and
  the coat layers into `../../genetics/layers/<model>`.

```
~/.venvs/blocky/bin/python export.py small_bird
```
