# Fireflies and glowing plants (Plants, 2026-11-05)

Built on `ecology-sync` (from the Weft sync) and seen on the PC: fireflies (`fireflies.rs`; `/fireflies`
says how many fly where you stand and why; none round a spectator, whom the world doesn't count
among its players) and the three glowing plants (`wild.rs`). Where it differs from the plan below: foxfire and
goblin's gold are picked as themselves (no loot table), and all three are placed on the forest
floor like the mushrooms, not only under crowns.

Ronald's fantasy picks from `reports/Research round 17.md`: recommendation 1 (fireflies) and two or
three of recommendation 4's glowing plants. To build after the Weft sync, on a branch cut from dev,
after the dung pats merge. Lighting draws the motes, the sky and the pack's night scaling of
emission; Art makes the firefly particle and the plants' textures with their `_e` masks.

The rule from the research holds throughout: few, dim and gated. Nothing glows everywhere, and
everything has a real animal or fungus behind it.

## Fireflies

**Where and when.** Each biome's weight comes from the fireflies of its real place:

| biome | real place | fireflies there | weight |
|---|---|---|---|
| swamp | the Pantanal | swarms over the wet grass | 1.0 |
| plains | Konza Prairie, Kansas | *Photinus pyralis*, the "big dipper" | 0.8 |
| tropical_forest | Manaus | many, at gaps and along water | 0.8 |
| savanna | the llanos | in the wet season | 0.6 |
| forest | Kassel | *Lamprohiza*, *Lampyris* at edges and in meadows | 0.6 |
| birch_forest | Moscow | *Lampyris noctiluca*, fewer | 0.3 |
| all the others | | none: taiga (none in interior Alaska), mountains, sequoia grove (the West's fireflies barely glow), desert, snow, beach, the seas | 0 |

- **Warmth:** fireflies fly from a night air of 12 °C up (`vitals::air` at the player), and only
  in a season whose mean air is at least 16 °C. That gives late spring to early autumn in the
  temperate biomes and the whole year in the tropics, from the climate tables alone.
- **Wet ground:** the count is × (0.3 + 0.7 × soil water) (`base_api::ground_at`; near water the
  soil is soaked). So the llanos come alive in the rains, and the plains are busiest after rain.
- **No rain:** none while it rains where the player is.
- **The hour:**
  - none before sunset (from `vitals::calendar::daylight_hours`);
  - rising to the full count by half an hour after sunset;
  - full to two hours after sunset;
  - falling to 0.3 by midnight and to none at 02:00.

**How many.** At the best place and hour:
- `base:fx_fireflies` = few: 24 within 24 blocks of the player;
- many: 60;
- off: none.

They come in clusters of 3-6, each cluster at least 4 blocks from the next.

**How it's built.**
- **Server:** every 5 s it works out each player's count from the rules above and puts it on the
  player as a component, `base:fireflies` (a u8). The client can't read biome, season or air,
  so the server does it.
- **Client:** a `client_frame` keeps that many alive with `Client::spawn_particles`.
  - Where a cluster goes: a random column within 24 blocks. It hangs 0.5-3 blocks over the top
    block there, in air.
  - Never by a torch: the block light there is at most 7.
  - Under open sky or a forest's edge: the sky light there is at least 10.
- **The particle:** each lives 2-4 s and drifts at 0.05-0.2 blocks a second. It gives no light.
- **Blinking:** the pack's particle shader makes them blink (Lighting): on 0.3-1 s, off 1-3 s, in
  step within a cluster. The look is Art's: a 1-2 texel yellow-green square, fullbright.
- **Cost:** under 0.02 ms (round 17), the count capped at 60.

The numbers (the weights, `FLY_MIN_C` 12, `SEASON_MIN_C` 16, the hour curve, the counts, the
cluster sizes) go in a new `vitals::fireflies`. A check ties them to the climate table: every
biome with a weight has at least two months at or over 16 °C. That's a vitals test, run on the PC.

**Later, from the same system:**
- the Cretaceous's floodplain: fireflies are in Burmese amber at 99 million years
  (*Protoluciola*);
- will-o'-wisps in the wetlands;
- warm seas glowing where you swim.

## Glowing plants

Two, perhaps three, as wild rows (`vitals::wild::WILD`, `vitals::plants::PLANTS`). They grow and
spread like the mushrooms, by warmth and soil water in the shade. Each has a fixed block light;
the sun washes it out by day. The pack's emission scaling by darkness and `base:fx_glow`
(0-1.5) are Lighting's: the setting dims or brightens the `_e` glow, never the block light,
which is the world's and the same for every player.

| block | real | where | light | growth (base, best, max °C; water) | share |
|---|---|---|---|---|---|
| `base:foxfire` | honey fungus (*Armillaria*) and bitter oyster (*Panellus*) mycelium in rotting twigs, glowing green | a flat mat (1/16 high) on the forest floor of the damp forests: forest, birch_forest, taiga, swamp, tropical_forest, sequoia_grove | 2 | 5, 20-25, 30; moderate | 0.004 |
| `base:glow_mycena` | *Mycena chlorophos* and kin, small glowing caps | clusters on the floor of the tropical forest and the swamp | 3 | 18, 24-28, 32; wet | 0.003 |
| `base:goblins_gold` (third, if worldgen can find the spots) | goblin's gold moss (*Schistostega pennata*), whose cells catch the faint light of cave mouths and glow gold-green | on stone under overhangs and in cave mouths: taiga, forest, birch_forest, mountains | 2 | 0, 10-18, 25; moist | Terrain's call: it needs a scatter on shaded rock |

- **Light:** the levels, 2-3, sit under Vintage Story's glow worms (7) and near Minecraft's
  firefly bush (2), as the research asks. They go in `vitals::wild::GLOW`, the way
  `wild::FIBRE` holds the fibre chances.
- **Art (claude/art-glow, approved):**
  - The block uses `glow_mycena_small`: a clump of about 10 px, since Mycena caps are 1-3 cm.
  - The larger `glow_mycena` (about 15 px) stays in the textures, unused for now.
  - Art's `glow_lichen` and `glow_crystal` get no blocks.
  - foxfire and goblins_gold come next from Art, as seamless top textures for a 16x1x16 mat.
- **Giving:** foxfire gives nothing (it is the wood's rot); glow mycena gives itself, like the
  mushrooms. Neither is food.
- **Frost:** both die back at a frost and come again from the next warm spell, as the mushrooms
  do.
- **Later:** foxfire also grows on First hours' felled logs as they rot.

**Checks:**
- every glowing row's light is 1-4 (the restraint);
- each is found only in biomes whose climate lets it grow in some season (the existing "every
  plant can grow where it's found" check covers that);
- the art gets the usual block look in a dark forest at night on Tester 2's queue, with
  `base:fx_glow` at 0, 1 and 1.5.

## Who does what

- **Plants:**
  - `vitals::fireflies` and the server count;
  - the client spawner;
  - the `base:fx_fireflies` row in `video.rs` (off, few, many; default few; `video.base.*` keys
    in en and ru);
  - the glowing plants' rows, blocks, light and placement;
  - the notes and checks;
  - the PC looks.
- **Art:**
  - the firefly particle;
  - `foxfire`, `glow_mycena` (and `goblins_gold`) with `_e` masks, via `block-textures`.
- **Lighting:**
  - the particle shader's blinking;
  - the `base:fx_glow` row and its night scaling of emission;
  - its slot in `scene.options`.
- **Terrain:** whether goblin's gold can have its shaded-rock spots.
