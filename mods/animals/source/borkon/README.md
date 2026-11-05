# Borkon, the Height-Keeper: its box model

The mountains' boss (`mods/animals/src/borkon.rs`, `animals_core::bosses`), made with the
`blocky-model` skill on Drenkaun's build (`../drenkaun`, itself the reindeer's), at 32 texels a
block, in concept A's colors (Ronald, 11-05: "A: ibex"; the concept in
`~/Coding/chiacraft-archive/images/boss-borkon/2026-11-05/borkon_a.png`).

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`borkon.py` lays out, paints and animates it into `out/` (not committed); `build.py` checks
flicker (at rest and in every clip), joints, thin lines (linecheck: none), pokes and gait, and
exports `models/borkon.geo.json`, `animations/borkon.animation.json` (idle, walk, run and the
fight's clips) and `textures/entity/borkon/borkon.png` into `../../assets`. Its body box is
`animals_core::bodies` ("borkon", 4.2 by 4.8). No coat genes: one of its kind.

## How it's made

- Drenkaun's layout in the deer's units (the body, legs with knees and hocks, the head 1.22 times
  bigger), drawn `K = 3.5` times bigger: the withers 3.3 m up, 5.3 blocks nose to tail, its horns'
  arc 5.8 up.
- The horns: eight lengths of horn a side from roots on the skull's top, up and back in a long
  scimitar arc over the neck, the tip turning down, each narrower than the one before (6.8 to 2.6
  units across, 1.25 times as deep). Each length's front face keeps to the arc's outer side
  (`horn_rod`), where the ridges are painted: a dark groove and a pale ridge every six texels,
  plainer on the inner side, lichen in patches on the middle lengths. Each horn its own bone
  (`horn_L`, `horn_R`: the `left_horn` and `right_horn` breaks).
- The ruff (the skill's rule 6, as Drenkaun's moss): one shell over the shoulders' front half, open
  behind, its hem and back edge cut in shaggy locks; long hair under the throat (a cut-out plane),
  a crest of tufts along the neck, a mat and locks down the chest; the front legs pale and shaggy
  to the knee. A lower coat over the hips, shaggy at its edges.
- A goat's head: the muzzle 63% of the skull's width, pale, with a small dark nose at its tip
  (nostrils as soft slits), a pale beard from the chin in two boxes narrowing to a point, narrow
  pointed ears held out under the horns.
- Snow lies in ragged drifts along its back (shoulders, hips, the ruff's top) and runs down the
  hips' sides in streaks.
- Eyes: Drenkaun's `warden_amber` (the skill's `hytale_amber` drawn round at 12 by 10, Ronald's
  pick for Drenkaun), on the head's sides, mirrored so the catchlights face forward, with lids
  that blink. `BORKON_EYES=ibex_amber` draws the same eye with a goat's long pupil across it.

## Its clips

`rise` (out of the ground), `charge_tell` (head down, a hoof scraping) and `charge` (a gallop,
horns levelled), `toss_tell` (head low, horns hooked under) and `toss` (flung up and round),
`rockfall_tell` (rearing high) and `rockfall` (down hard on the forefeet), `leap_tell` (crouched on
its haunches, fitted so all four hooves stay on the ground) and `leap` (forelegs tucked, hind legs
stretched, then reaching down), `stunned` and `down` (on its belly, heaving); with `idle`, `walk`,
`run` and `lie`. One-off clips don't loop.
