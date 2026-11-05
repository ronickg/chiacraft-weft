# Drenkaun, the Wood-Warden: its box model

The old forests' boss (`mods/animals/src/drenkaun.rs`, `animals_core::bosses`), made with the
`blocky-model` skill on the reindeer's build (`../reindeer`), at 32 texels a block, in concept A's
colors (Ronald, 11-05: "A: lighter, leaner elk"; the concept in
`~/Coding/chiacraft-archive/images/boss-drenkaun/2026-11-05/concept_A_clean.png`).

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`drenkaun.py` lays out, paints and animates it into `out/` (not committed); `build.py` checks
flicker (at rest and in every clip), joints, pokes and gait, and exports
`models/drenkaun.geo.json`, `animations/drenkaun.animation.json` (idle, walk, run and the fight's
clips) and `textures/entity/drenkaun/drenkaun.png` into `../../assets`. Its body box is
`animals_core::bodies` ("drenkaun", 4.8 by 5.4). No coat genes: one of its kind.

## How it's made

- The reindeer's layout in the deer's units, drawn `K = 4` times bigger: the withers 3.75 m up,
  6.6 blocks nose to tail, its antlers 5.9 blocks across and 7 up.
- Ronald's round on the first model (11-05, "Closer to A first"): legs thicker with knees and
  hocks, a brisket, shoulders and a lower belly at the back, a thicker neck, the head 1.22 times
  bigger (`HS`) with a 5 by 4 amber eye, the moss as a mane of uneven tufts (crest tufts, shoulder
  tufts, locks of random lengths) with strands from the antlers, the bark as overlapping streaked
  scales down the back and flanks, the antlers bigger with seven tines a side.
- Antlers like oak boughs, wide like a lyre: four beam lengths and five tines a side, bark-brown
  to bone-pale tips, moss dripping from the beam, leaves at the tines' roots; each antler its own
  bone (`antler_L`, `antler_R`: the `left_antler` and `right_antler` breaks).
- The moss mane: down the front of the neck and chest in locks of uneven length, tufts along the
  neck's crest, then over the shoulders and along the back in tufts each narrower than the last,
  thinning to the hips and stopping short of the rump; locks down the shoulders, shorter toward
  the back. No bark plates (Ronald, round 4: "Drop the plates").
- Eyes: the skill's `hytale_amber` drawn round at 12 by 10 (`EYES["warden_amber"]`, Ronald: "A
  amber", "eyes a bit bigger"), on the head's sides, mirrored so the catchlights face forward,
  with blinking lids; a narrow muzzle tapering to a dark nose, the nostrils soft slits low on it.
- Standing on bent legs: the knees and hocks bend, fitted so all four hooves stand level
  (`STANCE`, `stance_drop`); the withers stand higher than the hips.
- Carriage: the neck raised and the head level in every standing clip (`HEAD_UP`).

## The real animals it is built on

A guardian, not a real species, so its body comes from the biggest deer that lived and live:

| | shoulder | head and body | antler span | mass |
|---|---|---|---|---|
| Irish elk (*Megaloceros giganteus*), the largest deer known | about 2.1 m | about 3.1 m | up to 3.6 m, palmate | 450-700 kg |
| Moose (*Alces alces*) | 1.8-2.1 m | 2.4-3.1 m | up to 2 m, palmate | 380-700 kg |
| Red deer stag (*Cervus elaphus*) | 1.2-1.4 m | 1.8-2.4 m | about 1 m, branching tines | 160-240 kg |

Drenkaun is a titan (`vitals::combat::Size::Titan`: any boss), so it is drawn bigger than any of
them: about **4 m at the shoulder** (256 units), head and body about 6 m (384), antlers branching
like a red deer's (tines, not palms: "like oak boughs") and spreading about 5 m (320). That keeps
the elk's proportions: a deep chest and high withers, the neck carried forward and up, legs about
half the shoulder height.

## What makes it read

- the antlers: branching oak boughs, bark-brown with bone-pale tips, moss and a few leaves hanging
  (the part breaks: `left_antler`, `right_antler` are their own bones so a broken one can be hidden;
  `crown` is the bark plates on the head and shoulders);
- a mane of moss over the neck and shoulders; bark plates on the shoulders and back;
- calm amber eyes on the sides of the head, a lighter muzzle, dark legs and hooves.

## Its clips (with the animation system on `claude/animal-animations`)

The fight plays these by name (`drenkaun.rs`): `rise` (out of the ground), `charge_tell` (head
down, a hoof scraping), `charge` (the run, antlers forward), `sweep_tell` and `sweep` (the antlers
drawn back, then swung across), `stomp_tell` and `stomp` (rearing, then both forefeet down),
`roots_tell` and `roots` (head low to the ground, then raised), `stunned` (swaying, head low, looped),
`down` (fallen onto its side, looped), plus `idle`, `walk` and `run`.

## Colors

From its concept look (the `game-art` skill, `art.py animal drenkaun`), once Ronald approves it.
