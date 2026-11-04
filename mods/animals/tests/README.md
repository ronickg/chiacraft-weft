# Testing the animals

Two ways, the same rules (`mods/animals/core`):

- **The rules, fast** (seconds): the game's logic compiled to WASM and checked through its API:
  inheritance, the sexes, refusals, timings, caps, scars, old age, strength by age, wild breeding
  and deaths of want, evolution's direction, every animal unique.

      mods/animals/web/build.sh && node mods/animals/web/test.mjs

  They also check that `mods/animals/SPECIES.md` (every species' settings) is up to date;
  after changing a species, `node mods/animals/web/sheet.mjs` rewrites it.

- **In the game** (about three minutes): four glass pens on flat land (cats, chihuahuas,
  capybaras, raptors, six each, wild), time 1000 times faster (`/animal speed 1000`: about 0.8
  in-game days a second), and a census every 15 seconds (`/animal census`): how many at each age,
  expecting, in love, their mean strength, born and died since (`/animal reset`). The lines are
  in the log as `census:`; screenshots in `run/screenshots/pens*.png`.

      cargo build --release
      target/release/weft --world pens-test --size 1600x900 --script mods/animals/tests/pens.txt
      grep census: <the log>

  What a healthy run looks like: every pen keeps going for the ~150 in-game days, at around its
  species' crowd (`Species::crowd()`, derived from its facts) to twice it, with young, grown and old at once (generations
  overlapping), born and died rising together.

- **Time away** (`away.txt`, about 20 seconds): six wild cats and a tame one, then `/animal
  away 80` (80 days pass as if their area had been unloaded), then away 4000 blocks and back. The
  tame one dies of old age; the wild ones are replaced by successors of mixed ages, and the herd
  keeps its size through the reload.

      target/release/weft --world away-test --size 1600x900 --script mods/animals/tests/away.txt

- **The gateway** (`gateway.txt`, about 30 seconds): builds an amber frame beside the player,
  lights it with a torch, steps in (to the Cretaceous: a gateway is built in front of them
  there), steps into that one and comes back out beside the first. The log says
  `gateway: none near .., built one`, then `gateway: came out of the one at ..`.

      target/release/weft --world gateway-test --size 1600x900 --script mods/animals/tests/gateway.txt

In a world of your own: `/animal speed <n>`, `/animal census [radius]`, `/animal reset`,
`/animal age <days>` (the animal you look at, older), `/animal away <days>` (the animals round
you, as if that long had passed while away), `/animal scar`, `/animal habitats [radius]` (the
land round you by biome, each one's nearest spot, and how much of it each species lives on),
`/animal room [n]` (whether natural spawning would bring a group of `n` of the nearest animal's
kind to where you stand, or turn it away as its room is full: `life::room_for`),
`/dim animals:cretaceous` (straight to the Cretaceous).
