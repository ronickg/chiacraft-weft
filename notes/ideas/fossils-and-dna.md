# Idea: fossils, DNA and cloning (late game)

Status: written down, not started (Ronald, 2026-11-06). Parked on `notes/crew/BOARD.md`.

## Ronald's words

> "I used to play Zoo Tycoon, and I kind of liked the sort of DNA thing that there was. Like, it's
> very rudimentary, but I think if we could do something similar, but give it more thought behind
> it, with DNA and stuff. So for more like the late game where you can start cloning or doing more
> cool things with dinosaurs. I'm not sure if Ark has something like that ... But we would probably
> also need to sort of add fossils and stuff to the world as well."

## The loop, as a first sketch (to be designed and researched)

1. **Fossils in the world.** Fossil-bearing rock in the right strata and biomes (badlands, deserts,
   cliffs and quarries, the Gobi-like places our dinosaurs come from), found by digging, by erosion
   on cliff faces, or with a survey tool. They're species-specific: bones, teeth, eggs (ties in with
   the per-species eggs, `local_69e7095d`), amber with insects.
2. **Excavation and preparation.** Careful digging (brushes and picks; breaking it carelessly lowers
   quality), then cleaning at a preparation bench. Each find has a quality.
3. **DNA.** Extract fragments at a lab, each giving a share of a species' genome. Fragments from
   several finds add up (Zoo Tycoon's percentage, made deeper): quality, age and source (amber and
   eggs best) decide the yield; gaps can be filled with a modern relative's DNA (birds and
   crocodilians), which changes traits.
4. **Cloning and incubation.** Late game: synthesise an egg at a lab, then incubate it with the
   right temperature and humidity (our eggs and temperature systems already exist), hatch it, and
   raise it with the existing life, food and taming.
5. **Traits and variation.** Genome completeness and gap-fillers affect health, lifespan, temperament
   and look; breeding clones mixes traits; "more cool things" later (hybrids? only if it fits).

## To look at before designing

- Zoo Tycoon: Dinosaur Digs (fossil dig, DNA, incubation), Jurassic World Evolution (dig sites,
  genome %, gene modification), Ark (fossils, cryo; check what it actually has), Jurassic Park lore
  (amber, frog DNA gap filler).
- Real science, for flavour and limits: DNA degrades (half-life about 521 years), so real dinosaur
  DNA is out; what survives is proteins and soft tissue traces, ancient DNA from ice-age animals
  (mammoth), and de-extinction work (Colossal's dire wolves, woolly mice). The game can be honest
  that it's fantasy-science, or bend it with a magic or lore reason (ley lines?).
- What already exists to build on: `mods/animals` (species facts, life, eggs in `mods/animals/core`),
  survival temperature, worldgen strata, the ley-line network.

## Where it would sit

A late-game piece after the animals and survival loops; a new part under "animals" or "living" in
`notes/overview/pieces.json` once Ronald says go.
