# The owl's source

The night's hunter of every land biome (`notes/ECOLOGY.md`, "The food webs and the species to add",
tier 2: "owl"), the rodent's, the hare's and the small bird's: one model for the great horned owl
and the snowy owl (both *Bubo*), made with the `blocky-model` skill on the birds' build
([../bird](../bird)), from research alone, at the house density (64 texels a block).

## Rebuild

```
~/.venvs/blocky/bin/python ../bird/export.py owl
```

`owl.py` sets the owl's knobs and colors; `../bird/export.py` paints, checks and exports
`models/owl[_fly].geo.json`, their animations and `textures/entity/owl/owl.png` into `../../assets`,
and the coat layers into `../../genetics/layers/owl`.

## Research (2026-11-03)

- **Great horned owl** (*Bubo virginianus*): 46-63 cm long, a 1.0-1.45 m span, 0.9-2.5 kg (females
  the bigger). A big round head with long ear tufts, a rufous facial disc rimmed black, white brows
  and throat, big yellow eyes facing forward; mottled grey-brown above, buff below barred dark;
  broad rounded wings, a short tail, legs and toes feathered. The earliest nester (January and
  February): 1-4 eggs, usually two, 30-37 days' incubation, the young out of the nest at six weeks.
  About 13 years in the wild. Rodents and rabbits first, birds, skunks, small cats and dogs, fish,
  insects; prey to about twice its own mass. Across the Americas, from the edge of the tundra to the
  desert and the swamps. It turns its head about 270 degrees.
- **Snowy owl** (*Bubo scandiacus*): 52-71 cm, a 1.25-1.5 m span, 1.6-3 kg; white, the male nearly
  pure, the female and the young barred dark; tiny ear tufts; feathered to the toes. Lays 3-11 eggs
  in May by the lemmings (none in a poor year), 31-33 days. Lemmings and voles (3-5 a day),
  ptarmigan, Arctic hares; hunts by day in the Arctic summer.
- **The food webs** (`reports/ecology/Overworld food webs.md`): meat 100%, prey from 1% to 150% of
  its mass; nocturnal (the snowy owl also by day); pairs.

Decided: the great horned owl's proportions (the raven's layout 0.95 times its size: about 55 cm,
a 1.2 m span), the snowy owl the same model drawn bigger by its size genes, white by its genes and
its ear tufts scaled nearly to nothing by the horns build gene (`Ho` on `tuft_L`, `tuft_R`); a big
round head 1.75 times the raven's sitting on the shoulders, a short dark bill turned down into the
disc, the eyes on the face with front lids that blink, a stocky upright body, feathered legs, broad
barred round wings, a short round tail. Animations idle (perched upright, the head turned far round
one way and the other in holds and snaps, a tilt, blinks while it looks ahead), walk, hop, and a slow
wingbeat with glides.

## How it's made

- **Eyes on the face** (`eyes.kind` "front" in the birds' build): stamped on the skull's front, the
  lids hinged at the eyes' top and swung down about x, as the fox's.
- **The neck** is a hidden link inside the shoulders and the head (no corners show, so the poke
  check leaves it out); the head is leveled by itself when the body stands upright.
- **Coats from genes**: the facial disc is its points (rufous to grey in a great horned owl, white
  in a snowy owl); the bars are the `streaks` pattern, heavy in a snowy owl female, light in a male.
