# Drenkaun, the Wood-Warden: its box model (to build)

The old forests' boss (`mods/animals/src/drenkaun.rs`, `animals_core::bosses`). Until this model is
built and approved, the game draws the reindeer's model 3.2 times as big (`drenkaun::SCALE`).

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
