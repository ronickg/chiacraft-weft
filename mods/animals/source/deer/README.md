# The deer's source

One model for the white-tailed deer (*Odocoileus virginianus*) and the red deer (*Cervus elaphus*),
the living world's big browser and grazer and the wolf's and jaguar's prey (`notes/ECOLOGY.md`,
"The food webs and the species to add"), made with the `blocky-model` skill
([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research
alone, at the house density (64 texels a block). The recipe is
[docs/creatures.md](../../../../docs/creatures.md).

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`deer.py` lays out, paints and animates it into `out/` (not committed) and fits lying down and
grazing to the ground; `build.py` checks flicker, joints, pokes and gait and exports
`models/deer.geo.json`, `animations/deer[_lie].animation.json` and `textures/entity/deer/deer.png`
into `../../assets`, and the coat layers into `../../genetics/layers/deer`. `DEER_COAT=red` paints
a red deer's colors, to compare in the viewer (the game paints every deer's coat from its genes).

## Research (2026-11-03)

- **White-tailed deer**: 53-120 cm at the shoulder, 95-220 cm long with a 10-37 cm tail; bucks
  usually 68-136 kg, does 40-90; reddish brown in summer, grey-brown in winter, the tail's white
  underside raised as a flag when alarmed; fawns spotted through their first summer. Antlers only on
  bucks, grown in velvet from late spring, shed in winter. The rut in late October and November,
  one to three fawns in May and June; does breed at about 18 months. Up to 60 km/h. From forest to
  prairie and desert edge in North America, and the llanos' savannas in South America. Eaten by
  wolves, cougars, alligators and jaguars; the fawns by coyotes, lynx and bears
  ([Wikipedia](https://en.wikipedia.org/wiki/White-tailed_deer)).
- **Red deer**: 95-130 cm at the shoulder, stags 160-240 kg and hinds 120-170; red-brown in
  summer, greyer in winter with a buff rump patch; antlers on stags only. The rut from August into
  early winter, a 240-262 day gestation, one calf, spotted; 10-13 years in the wild. Herds; the
  wolf is its main predator ([Wikipedia](https://en.wikipedia.org/wiki/Red_deer)).

Decided: the white-tailed buck's proportions (0.95 m at the shoulder, 1.7 m long), the red deer the
same model drawn about 1.25 times bigger by its size genes; white-tailed antlers (a beam sweeping
forward, three tines); antlers on males only; animations idle (grazing with the mouth at the grass,
chewing, looking up, ear and tail flicks), walk, run (bounding, the white tail up as a flag) and
lie (legs folded, chewing the cud).

## How it's made

- **Antlers by sex, no engine code**: they sit on their own bones (`antler_L`, `antler_R`), which
  the build trait for horns (`Ho`) scales; in the gene files its range is nearly 1, so the trait's
  bottom draws them to nothing. A doe's sex shifts it all the way down, a buck's sits near the
  middle (antlers of varying size), and a young one's is lower still (spikes).
- **Antlers as cut-out planes** (the skill's rule 7, Ronald 11-05): one quad an antler facing
  sideways, turned out with the beam and leaning out, the beam and three tines drawn a texel thick
  in its texture (`ANTLER_ART`); they were rods under a texel thick. The nose is painted on the
  muzzle (rule 4), and the chest is flush with the hips at the sides and 2.25 texels higher at the
  withers (rule 3).
- **Fawns are spotted**: the gene files' `young` pattern, fading as they grow.
- **The neck** rises at 55 degrees from low in the chest, long enough (with the front legs
  straddled a little) that the mouth reaches the grass with all four hooves on the ground.
- **The tail** is a flap hinged on the rump's surface (the tongue rule), checked by eye in the flag
  pose rather than by `ground.pokes`.
