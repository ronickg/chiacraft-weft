# The reindeer's source

The reindeer (caribou, *Rangifer tarandus*), the tundra's only big grazer and the wolf's prey there
(`notes/ECOLOGY.md`, "The food webs and the species to add", tier 2), made with the `blocky-model`
skill ([.claude/skills/blocky-model](../../../../.claude/skills/blocky-model/SKILL.md)) from research
alone, at the house density (64 texels a block), on the deer's build ([../deer](../deer)). The
recipe is [docs/creatures.md](../../../../docs/creatures.md).

## Rebuild

```
~/.venvs/blocky/bin/python build.py      # any Python with numpy, scipy and Pillow
```

`reindeer.py` lays out, paints and animates it into `out/` (not committed) and fits lying down and
grazing to the ground; `build.py` checks flicker, joints, pokes and gait and exports
`models/reindeer.geo.json`, `animations/reindeer[_lie].animation.json` and
`textures/entity/reindeer/reindeer.png` into `../../assets`, and the coat layers into
`../../genetics/layers/reindeer`.

## Research (2026-11-03)

- **Size**: 85-150 cm at the shoulder (Svalbard's down to 80), 162-214 cm long, a 14-20 cm tail;
  bulls usually 159-182 kg (up to 318), cows 80-120 kg; bulls lose up to 40% of their weight in the
  rut ([Wikipedia](https://en.wikipedia.org/wiki/Reindeer)).
- **Look**: a stocky deer on shorter, thicker legs than other deer, a deep chest, a long head with
  a broad blunt muzzle furred to the lip (no bare nose), short round ears; a woolly undercoat under
  hollow guard hairs. Grey-brown, with a cream neck and a long pale mane down the throat, a white
  rump and belly, a dark band along the flank, dark legs and face, white above the big crescent
  hooves. Northern ones whiter, southern ones darker; white reindeer turn up in herds.
- **Antlers on both sexes**, the only deer whose cows have them: a bull's big (a beam sweeping
  back and up, then forward to a palm with points; a bez tine; over the face the brow tine, often
  one flattened "shovel"), shed in late autumn after the rut; a cow's smaller, kept until she calves.
- **Life**: the rut from late September to early November; 228-234 days' gestation, one calf in
  May or June; cows breed from 1 to 3 years by their summer's food; cows live to about 17 at most,
  bulls about four years less. Runs at 60-80 km/h; spring herds of 50,000 to 500,000.
- **Food**: lichens through the winter (reindeer lichen, dug from under the snow), willow and
  birch leaves, sedges and grasses in summer, mushrooms; now and then shed antlers, lemmings, eggs.
- **Eaten by** grey wolves, brown and polar bears, wolverines and golden eagles (calves).
- **Cold**: its fur keeps it warm far below -30 °C, its legs' blood cooled by countercurrent
  exchange; its eyes' tapetum turns from gold in summer to blue in winter.

Decided: a bull-and-cow average's proportions, 1.05 m at the shoulder and 1.9 m from the nose to
the tail: the deer's layout drawn 1.12 times bigger, the chest deeper and broader, the legs shorter
against the body and thicker, broad hooves, the neck carried forward and low, a long head with a
blunt furred muzzle, small round ears, a short tail. Antlers on both sexes (the cows' smaller, by
the build gene for horns), the brow shovel on the left only. Idle (grazing with the mouth at the
ground, chewing, looking up, ear and tail flicks, blinks), walk, a gallop and lying down to chew
the cud (blinking).

## How it's made

- **Laid out in the deer's space**: every box and pivot in `reindeer.py` is in the deer's units
  and scaled by `K` before painting, so the texture keeps the house density; the color rules read
  positions back in the deer's space (`p / K`). The eyes and their lids are placed on the scaled
  skull, with the eye's rounding.
- **The mane**: a box down the throat, and a cut-out quad through its middle (in the neck's own
  frame) hanging below it in locks, longest mid-throat.
- **Antlers by the horns gene**: on their own bones (`antler_L`, `antler_R`), scaled by `Ho`; a
  cow's sex turns it down a little (smaller antlers, not none), a calf's more.
- **The tail** is a flap hinged on the rump's surface (the tongue rule), checked by eye.
