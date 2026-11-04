# Research round 2: six features only we can do, entity ranges, and the risks sized

Researcher, 2026-11-03, for Planner. Read against the trunk at `090ac91`. It follows
`reports/Research round 1.md`, whose ids (R1, N4, P4, E1, K2…) it uses.

## The points that matter most

1. **E1 needs about 80 lines in Weft.** It adds three settings:
   - `core:entity_send_distance`, 96 blocks;
   - `core:entity_activation_distance`, 48 blocks;
   - `core:inactive_entity_interval`, 20 ticks.

   Far entities' moves also go out less often (every tick, every second tick, every fourth tick
   by distance). With entities spread evenly, move traffic drops about **77%** and mod goal work
   about **80%**. A pen of 200 animals today costs each nearby player about 1.6 Mbit/s of moves
   before compression; a host with 8 players needs 13 Mbit/s of upload. The change is generic
   enough to offer to Weft upstream.
2. **`pools.json` reaches tens of megabytes in a long-lived world.** It is rewritten whole every
   30 s while holding the pools lock, which also blocks the animals' world tick:
   - about 370 bytes per pool (measured in saves), about 12 pools per explored 256-block cell;
   - a 10,000-cell world (100 hours of exploring) holds about 45 MB;
   - writing that is about 0.3–0.5 s of stutter every 30 s.

   **Fix:** shards of 8×8 cells, loaded when touched, only dirty shards written, off the tick
   thread. Pools nobody hunted or bred into are pruned.
3. **The outbox:** ten dead inboxes make one pass take up to 280 s, and a fresh link request
   waits behind them. **Fix:** a lane per inbox, eight at a time, one connection per inbox, and
   one save per pass instead of one per message. Fresh messages then go out within seconds.
4. **Content sets kept in memory are a small risk.** A build takes 7–9 ms and has 2.6k states.
   The client already frees the old world's textures on every swap (`textures.rs:278-345`). Only
   the CPU-side `Content` is leaked, and only once per distinct package set. Measure it, don't
   fix it now.
5. **Six features, in build order:**
   1. letters through portals (S–M, Phase 3 extras);
   2. pedigrees (S local, M signed);
   3. genes through portals (M, after the pool fixes);
   4. the network atlas (M, the first slice of Phase 5);
   5. real-place worlds (M, the content plan);
   6. passports and guest books (M, Phase 4).

   Two of them need no new trust machinery at all. A QUIC connection to a world's key already
   proves who answers, so a pool summary or a status read from it needs no extra signature.

---

## 1. Six features only we can do

Each sketch gives what the player sees, what it needs and from where, its effort, and its phase.
"Bridge" means `crates/engine/src/chiacraft/bridge.rs`, the mods' way to ask the engine our own
questions.

### 1.1 Letters through portals (L3)

**What the player sees.**
- A **mailbox** block (base mod) beside their portal or house. They write a letter at it: to the
  owner of a linked world, or to a friend, up to 1,000 characters.
- It goes out at once if the other side is online, otherwise when it next is, within two weeks.
- The recipient's mailbox raises a flag, their title menu says "2 letters", and a Letters screen
  (from Multiplayer and Friends) lists them: from whom, which world, when.
- Reply and Block are one click each.

**What it needs.**
- `links.rs`: a fourth `Message` variant, `Letter { from, vouch, to, world, text, sent, sig }`,
  signed by the sender's identity after `chiacraft letter v1\n` (the BLS vouch requests already
  use).
- It travels on `chiacraft/links/1` to the recipient's account inbox. It is retried by the
  outbox that already exists (`p2p::outbox`, 14 days).
- Received and sent letters are kept in `letters.json` through R1's `JsonStore`.
- Who may write:
  - the owners of worlds linked to yours, and friends;
  - anyone else's letter is dropped silently, like a stranger's refusal today;
  - the blocked list in `links.json` applies.
- Caps: 50 unread per sender, 20 a day per sender, 2 KB on the wire.
- Bridge: `letters` (unread count, for the mailbox flag) and `write <to> <text>`.
- Base mod: the mailbox block and its screen. Every string is a translation key; the letter text
  is the player's own.

**Effort:** S–M. Most of it is the screens; the delivery is already built.

**Phase:** 3 extras. Do it after R1 (atomic stores) and N4 (outbox lanes), or one dead inbox
holds up every letter.

**Later:** with inventory carry-over, a letter can hold a parcel (moved, never copied), but that
waits for carry-over's trust rules.

### 1.2 Pedigrees (L7)

**What the player sees.**
- The animal screen gets a **Line** tab:
  - parents, grandparents and great-grandparents;
  - for each: coat, sex, wild level, who bred it, in which world, and when;
  - wild-caught founders marked "caught in the wild at <place>".
- Below the tree, the **inbreeding coefficient** of the litter: the real breeders' measure
  (Wright's F), worked out from the tree.
- Breeding a brother to a sister shows "F = 25%, high". Later this can cost vigor, which real
  inbreeding depression does.
- Across worlds, a famous breeder's line can be proven.

**What it needs.**
- **A record per animal** (`animals_core::pedigree`). It holds:
  - species, genome (the 40 bytes), sex and born day;
  - mother and father record ids, or "founder" with where and when it was caught;
  - the breeder (the mother's owner at birth, from the bridge's `player_key`) and the world's
    key.

  The id is the BLAKE3 of its canonical text. A record is about 300 bytes.
- **Storage:** a component `animals:pedigree` holding the id (32 bytes), and the world's
  `mods/animals/pedigrees.jsonl`, append-only. A thousand births a year is about 300 KB.
- **Signed (the second step):** a bridge op `sign_pedigree <text>`, in which the world key signs
  only text that starts `chiacraft pedigree v1\n`.
  - The world signs as itself, a notary for births that happened there. That keeps rule 5:
    worlds never sign for a player, and mods can't make the world sign anything else.
  - The engine side is about 15 lines in `bridge.rs`, using N2's `Signable`.
- **Across worlds:**
  - When tames travel (inventory carry-over), their ancestors' records go with them: 31 records
    for five generations, about 9 KB.
  - Or a world serves its births on `chiacraft/pedigree/1` beside its status, for checking
    later.
  - Anchoring champion lines on chain fits Phase 4's achievements, later.

**Effort:** S for the local, unsigned tree and F (the animals mod only); M for signing; with
carry-over for cross-world.

**Phase:** animals now. The signing goes beside Phase 4's world identity work.

**Why it's ours:** pedigrees and inbreeding coefficients are real animal husbandry. Signed by the
world they were born in, they are provable across a network nobody runs.

### 1.3 Genes through portals (L1)

**What the player sees.**
- Near a bound portal, wild animals now and then carry coats from the world on the other side.
- A cold world linked to a warm one slowly gets warm-world coat genes near the portal, and the
  warm world gets cold-world ones.
- Now and then a **wanderer** appears beside the portal. The spyglass shows "came through the
  portal from Fernhollow". Its genes come straight from the other world's pool.
- The Links screen has a per-link toggle, "Let wildlife wander through", which is on.

**What it needs.**
- **The idea:** a portal makes the two portal cells **neighbours** in the gene-pool graph. The
  pools already mix with their eight neighbours at `MIGRATION` (2% a generation,
  `mods/animals/src/evolution.rs`, `step`). The cells either side of a link become one more
  neighbour for each other.
- **The exchange:**
  - A new ALPN `chiacraft/wildlife/1` on the world endpoint. A world asks a linked world for the
    pools of its portal cell for that link: species name, habitat, the gene file's hash, the
    frequencies.
  - Size: about 12 pools × 370 bytes ≈ 4.5 KB.
  - Each world asks once a game day per link when the other side is online. Worlds are mostly
    offline, so it happens when both run, and the generations missed are caught up within
    `CATCH_UP`.
  - **No new signing:** the QUIC connection to the world's key already proves the answer came
    from that world.
- **Safety:** a hostile world can send any numbers, so:
  - only species that live here, in this habitat, with the same gene-file hash, are mixed;
  - frequencies are clamped and normalised;
  - their weight is capped at one neighbour's share of `MIGRATION`.

  The worst a liar can do is nudge allele frequencies near one portal by 2% a generation, and
  only for alleles that already exist.
- **The wanderers:** a natural spawn within 32 blocks of a bound portal, with about 1 in 20
  chance, draws from the other side's summary instead. It gets a component `animals:came_from`
  (the world's name and key), which the spyglass and the animal screen show.
- **Where it touches:**
  - `mods/animals/src/evolution.rs`: the neighbour list takes the link cells, plus the exchange
    and the cache.
  - Bridge: `portal_cells` (each bound portal's link and position) and `wildlife <link>` (the
    other side's last summary).
  - `p2p.rs`: the ALPN, through N1's `answer_once` helper.
- **Depends on:** P4 (cheap pool steps) and the sharded pools (section 3.1). Otherwise more
  neighbours make the hitch worse.

**Effort:** M. **Phase:** 3 extras, with the animals session.

**Later:** with carry-over, a tame taken through and released joins the other world's pool. This
already happens between cells (`count` with +1 on release).

### 1.4 The network atlas (L2)

**What the player sees.**
- An **Atlas** screen (from Multiplayer and from the map key). Your world sits in the middle and
  the worlds you link to sit around it.
- Each world shows:
  - its name and owner;
  - its climate, coloured by Köppen zone;
  - the season and the time of day there;
  - online or not, and players inside.
- One ring further out sit the worlds those worlds link to, if their owners choose to show their
  links: names only, greyed.
- Clicking a world:
  - Join, if it is open to you;
  - Ask to Link, for a second-ring world;
  - Write a Letter (1.1).
- Friends' worlds sit in their own colour.
- It is not a directory. Everything on it comes from worlds you already know, so it needs no
  service at all (rule 1).

**What it needs.**
- **A status v2** on `chiacraft/status/2`, with v1 still served for older games. To today's
  `Status` (`p2p.rs:569`) it adds:
  - the spawn biome's place and Köppen code (`vitals::climate`, through the bridge from the base
    mod);
  - the season and day;
  - the number of links;
  - optionally the list of links: the other worlds' keys and names. This is behind an owner
    setting, "Show my links on others' atlases", which is **off**.
  - About 200 bytes, plus 48 bytes a link.
- **The screen** (`net/screens.rs`): rings, not a force layout. It's simpler and it's stable
  between visits.
- **Data:**
  - ring one comes from `links.json`, `worlds.txt` and the friends' lists;
  - ring two comes from ring one's status answers, cached for 10 minutes;
  - pings go out only for worlds on screen, through one shared endpoint (N1).
- **Later:** Phase 5's chain and DIG listings add more nodes in the same screen.

**Effort:** M (status v2 with its versioning, and the screen). **Phase:** 5, as its first slice.
It makes "finding worlds" start from the links people already have.

### 1.5 Worlds from real places (L4)

**What the player sees.**
- Create New World gets **Home climate**. The 11 overworld places are each shown with a
  one-line description and a little climate chart:
  - Kansas tallgrass prairie;
  - a Hessian beech forest;
  - Moscow's birch forest;
  - Fairbanks taiga;
  - Utqiaġvik tundra;
  - the Sonoran desert at Hermosillo;
  - the Arauca llanos;
  - the Pantanal;
  - Davos;
  - the Azores;
  - the Algarve.
- **The world spawns in that biome**, and that biome dominates the land for a few kilometres
  around. Its real climate, rain, wind, fog and seasons are already the biome's.
- The atlas and the world list show the place, "Pantanal, Brazil".
- Linking a Pantanal world to a Fairbanks one makes the two complements: what grows in one
  doesn't in the other.

**What it needs.**
- **A world rule** `base:home_place`, a `Config<u32>` like `DIFFICULTY` (`base_api`), set on
  Create New World. "Mixed", today's world, is the default.
- **The base worldgen** (`mods/base/src/worldgen.rs`) weights biome choice toward the place's
  biome, more strongly near spawn, falling off over about 3 km. Check `docs/worldgen.md` for
  where biome selection reads its noise. It is a weight per biome, all in our mod, not the
  engine.
- **The climate rows exist** (`vitals::climate::BIOMES`, from `reports/ecology/biome_climates.json`).
  Southern places' rain is already shifted six months, so "its summer is the calendar's". No
  calendar change is needed.
- **The status v2** (1.4) carries the place.

**Version 2:** places beyond today's biomes (Yakutsk, the Kalahari, Patagonia, the Yixian of
125 Ma). Each needs a researched row in the format of `biome_climates.json`, its species and
plants checked by `places.rs`, and maybe a new biome. That is L, one place at a time.

**Effort:** M for v1. **Phase:** the content plan (Living world), with the worldgen owner.

**Why it's ours:** a real place's climate already drives the air, the weather, plants and
animals. Picking one makes the world *be* somewhere, and trade between linked worlds then has a
reason (1.3, L5).

### 1.6 Passports and guest books (L6)

**What the player sees.**
- **Stamps.** Leaving a world after playing there (an hour or more in all), they get a stamp: the
  world's name, its place, first day, hours played.
  - Stamps fill a **passport** on their profile.
  - When they join a world that asks for it, the join sheet says "Fernhollow asks to see your
    stamps", with Show and Don't show.
- **A guest book.** A lectern near spawn holds the world's guest book: who visited, when, and a
  line each wrote if they wanted. It's the visitor's own signature, so the owner can't fake
  entries.
- **Owners get an admission rule:** "stamps from at least N worlds I link to, M hours each".

**What it needs.**
- **A stamp** is one line signed by the **world key** (N2's `Signable`):
  `chiacraft stamp v1\n<world key>\n<visitor identity>\n<first day>\n<hours>\n`.
  - It's given and refreshed when the player leaves (`app.rs` leave paths, through the server's
    `ServerInfo.identity`).
  - It's kept in the player's `stamps.json` (R1's store).
  - It's carried in the join hello only when the player agrees, which needs a protocol bump to
    6.
- **A guest book entry** is signed by the visitor's **identity**, with the BLS vouch:
  `chiacraft guestbook v1\n<world key>\n<day>\n<line>`.
  - It's kept in the world's `guestbook.json`, shown by a lectern block in base.
- **The admission rule** reads stamps. It's Sybil-resistant only as far as the linked worlds are
  trusted, but links are already the owner's trust.
- **Phase 4 makes them stronger:** stamps name the visitor's DID once the join proves it (slice
  6 of the Phase 4 plan), and the DID's age shows beside them.
- **Privacy:** stamps are the player's to show and never sent unasked. A guest book line is
  opt-in.
- A world that later bans someone can't take a stamp back. Its shared ban list (Phase 4 slice 3)
  covers that.

**Effort:** M. **Phase:** 4, after admission and bans (slices 1–2). It needs no chain itself.

---

## 2. E1: send range and activation range

### What the engine does today

- **Sending** (`server/sync.rs`):
  - Each tick, for each player, `send_entities` (`:231`) walks every entity of every region of
    the dimension.
  - It keeps those within the simulation distance: `view()` cuts the full view to
    `core:simulation_distance`, which is 8 cubes (128 blocks) and can go up to 64 cubes.
  - `collect_moves` (`:75-116`) sends every entity that moved this tick, every tick (20 a
    second).
  - A move is `(u64, DVec3, f32, f32, Pose, u64)` (`net/mod.rs:183`): **49 bytes** before LZ4.
  - Correction to round 1: the range is the simulation distance, not the full view distance.
- **Ticking** (`server/dim.rs` `tick_entities`, `:1788`):
  - Every entity of every loaded region goes to its kind's mod `on_tick`, in one batch per kind,
    every tick.
  - `navigation` runs every walking entity's path.
  - Regions are loaded within the simulation distance of some player.
- **Spawning** caps passives at 6 within 64 blocks of each player (`registry.rs:1357`). Passives
  never despawn, so they build up in places players stay. Tames and bred animals aren't capped
  at all.

### What it costs, with numbers

- Moves alone, for 100 moving animals within 128 blocks: 100 × 49 B × 20 = **98 KB/s, 0.8 Mbit/s
  per player**, before LZ4 and framing.
- A breeding pen of 200 animals near spawn costs each player 1.6 Mbit/s. A host with 8 players
  then needs **13 Mbit/s of upload**: a home line's whole upload, and well past what a
  rate-limited n0 relay will carry.
- The scans themselves are cheap: players × entities distance checks, 20 × 2000 = 40k a tick.
  Don't add a spatial grid yet.

### The design

**Three settings,** world scope (synced, like the simulation distance), in `config.rs` beside
`core:simulation_distance` (`:476`):

| setting | unit | default | range | meaning |
|---|---|---|---|---|
| `core:entity_send_distance` | blocks | 96 | 16–1024 | Entities farther than this from a player aren't sent to them. Used as min(this, simulation distance × 16). |
| `core:entity_activation_distance` | blocks | 48 | 0–512 | Entities farther than this from every player in their dimension are *inactive*. 0 means everything is active, as today. |
| `core:inactive_entity_interval` | ticks | 20 | 1–200 | An inactive entity gets its mod tick and its navigation once in this many ticks, on tick `(time + id) % interval == 0`. |

**Move rates by distance** are fixed fractions of the send distance, so there's no fourth
setting:
- within ⅓ of it (32 blocks), every tick;
- within ⅔ (64), every second tick;
- beyond, every fourth tick.

The client already eases non-solid entities toward their last target (`client/receive.rs:328`),
so far ones just glide.

**Always active,** never throttled:
- players;
- entities that ride or are ridden;
- anchored entities;
- entities burning, or hurt in the last 100 ticks;
- kinds with a new flag, `entity_flags::ALWAYS_ACTIVE`, for projectiles and anything a mod needs
  exact (`.always_active()` on the SDK's entity builder).

**What inactive means:**
- left out of its kind's `on_tick` batch;
- left out of `navigation`;
- except on its tick in the interval.

Physics still steps every tick, which is cheap and parallel already (`par_iter_mut`), so nothing
floats or falls through the world.

**For mods,** one new host call, `entity_tick_elapsed(world, id) -> u32`: ticks since this entity's
last mod tick (1 when active). Meters that grow per tick can scale by it.
- The animals mod barely needs it: its upkeep already catches up from a last-seen time
  (`animal.rs`, `seen`).
- Its staggers `(time + id) % 10` and `% 20` divide 20, so they still land.
- Goals that count world-wide (`time % 600`) should switch to "time since I last did this".

### The change to Weft, about 80 lines

1. **`config.rs` and `registry.rs`:** three `ConfigDef`s and their ids, like
   `simulation_distance` (`registry.rs:383, 1500`). About 12 lines.
2. **`server/sync.rs`:**
   - In `send_entities`, `range2` becomes the send distance's square.
   - Moves are filtered by the tier for this player and tick.
   - So a skipped tick loses no update: `collect_moves` stamps `e.moved_at = tick` on the entity.
     A far tier sends, on its tick, the *current* position of every visible entity whose
     `moved_at` falls in the tier's last interval. No per-player history is needed.

   About 30 lines.
3. **`server/dim.rs` `tick_entities`:**
   - Gather the dimension's player positions first (they are gathered later in the same function
     already).
   - An `active(e)` test.
   - The `by_kind` batches and the `walks` filter in `navigation` (`:1579`) take only entities
     that are active, or on their interval tick.

   About 25 lines.
4. **`abi` and `sdk`:** the flag, the builder method and the host call. About 15 lines.

It is a server feature any Weft world wants, not a chiacraft one. Offering it upstream as a pull
request keeps rule 4: if Weft takes it, we carry nothing.

### What it saves

With entities spread evenly over the 128-block disc:
- **Sending:** the 96-block disc is 56% of the area. Inside it:
  - the inner ninth is sent every tick;
  - the next three ninths every second tick;
  - the outer five ninths every fourth tick.

  That is 0.42 of the full rate over 56% of the entities, about **23% of today's move traffic**,
  77% less. The 200-animal pen 80 blocks away goes from 1.6 Mbit/s to 0.4.
- **Ticking:** the 48-block disc is 14% of the 128-block disc. Inactive entities run 1 tick in
  20, so mod goal work falls to 14% + 86%/20, about **18% of today**.
- The real saving is smaller, because tames follow their owners and stay active. But the pens
  and the wild ones, which are most of the count, stop costing.

### Test without the GPU

- A unit test in `server/` builds a dimension with 1000 entities and two players, ticks it, and
  counts the moves sent and the `on_tick` calls per tier.
- Plus T3's iai-callgrind bench for `tick_entities`.
- Tester then runs one world with a pen of 200 for a look at far animals gliding.

---

## 3. The risks sized

### 3.1 `pools.json` (K2)

**Measured.**
- Pools in current saves are 318–396 bytes of JSON each: `run/saves/pens-check`, 4 pools of
  38–55 frequencies (dog, raptor, capybara, cat). An older save, `evo-check`, has 26 pools of
  109–204 bytes, from before the core loci.
- `Pool` is `Vec<Vec<f32>>`, loci × alleles (`animals/core/src/coats.rs:372`).

**How it grows.**
- A pool is made the first time a species is wanted in a (256-block cell, habitat), by a spawn
  (`pool_at`) or a death or release (`count_at`).
- A cell holds 2–3 habitats, and 6–10 species live in each: about **12 pools per explored cell**,
  about 4.5 KB.
- Natural spawns land 20–45 blocks from the player, so walking touches about one to two cells per
  256 blocks. An hour of walking (about 15 km) touches about 100 cells, so **about 450 KB of pools
  an hour**, more riding.

| world | cells | `pools.json` | each save |
|---|---|---|---|
| a weekend's play | 1,000 | 4.5 MB | about 40 ms |
| 100 hours, one explorer | 10,000 | 45 MB | 0.3–0.5 s |
| a busy server, five explorers, a few hundred hours | 50,000 | 225 MB | 1.5–2.5 s |

**Why it bites.**
- `save()` (`src/evolution.rs:99-111`) serialises **all** pools with `serde_json::to_string`
  while it holds the `POOLS` lock. It runs from the world tick every 600 ticks (30 s) whenever
  anything is dirty.
- Something is nearly always dirty: `evolved` marks a pool dirty whenever its day moves on, which
  every spawn does.
- Every spawn, death and release needs the same lock, so the world tick that saves stalls, and
  any animal call waiting on the lock stalls with it.
- Opening the world parses it all, and memory holds it about twice over.
- The R1 bug wipes it all on one bad parse.

**Fix (M, the animals session).**
1. **Shards.** `mods/animals/pools/<sx>.<sz>.json`, one per 8×8 cells (2048 blocks square). A
   shard holds about 64 × 12 pools ≈ 280 KB.
   - It's loaded the first time a pool in it is wanted.
   - Up to 64 shards are kept in memory, least used first out after they are saved.
   - `pools.json` is read once and split into shards (a migration), then renamed
     `pools.json.migrated`.
2. **Save only dirty shards, off the tick:**
   - Under the lock, take the dirty shards' text, which is a few at a time near players.
   - Write it on a thread with the atomic write (R1).
   - About 1 MB per 30 s at worst, a few milliseconds of lock.
3. **Prune.** A pool gets `touched: bool`, set by `count_at` (hunting, taming, releasing). When a
   shard loads, pools that are untouched and more than 400 generations old are dropped and
   remade fresh when next wanted.
   - A fresh pool is already 300 generations of history (`HISTORY`), so an untouched one comes
     back the same in kind.
   - Only what players did there is kept.
4. **Optional, later:** frequencies as `u16` in postcard instead of JSON floats, about 110 bytes a
   pool, 3.4× smaller. Do it only if shards prove too big; JSON is easier to debug.

**After:** opening a world reads only the shards near spawn. A save writes about 1 MB at most.
A 50,000-cell world still costs about 70 MB on disk, but never more than 18 MB in memory or more
than a few shards' writing per 30 s.

### 3.2 The outbox (K4)

**Today** (`p2p.rs:448-470`, `links.rs:641-678`):
- One loop, one message at a time. Each message:
  - asks the world for its owner's inbox, if that isn't known yet (8 s timeout);
  - then connects and delivers (20 s timeout).

  A message to a dead inbox costs up to **28 s**.
- Backoff is 60 s doubling to an hour; messages give up after 14 days.
- `tried()` saves the whole `links.json` after every message.
- `due()` clones the whole outbox each pass.
- New messages wake the loop, but only after the pass running now finishes.
- Nothing caps how many messages wait.

**Sized:**

| waiting | one pass today | a fresh message waits |
|---|---|---|
| 1 dead inbox | 28 s | up to 28 s |
| 10 dead inboxes (a hub world, a few owners gone) | up to 280 s | up to 280 s |
| 50 dead | up to 23 minutes | up to 23 minutes |

Each pass also makes one full rewrite of `links.json` per message, and the file holds every link
and tombstone.

**Fix (S–M, Phase 3 leftovers).**
1. **Lanes.** Group due messages by inbox, or by world while the inbox is unknown. Run up to
   eight lanes at once (a `JoinSet`).
   - A lane opens **one** connection and sends its messages in order, one stream each.
   - On the first failure, the lane's other messages count as tried too, so a dead owner costs
     one connect, not one per message.
2. **A fresh message gets its own lane at once.** The loop `select!`s on `OUTBOX.notified()`
   while lanes run.
3. **Shorter limits:** 10 s to connect, 10 s for the answer.
4. **One save per pass**, through R1's store.
5. **A cap:** 500 waiting in all and 20 per inbox. Past that, the oldest request is dropped
   (never a revocation or a letter's bounce).
6. **Remember the unreachable.** For an inbox that failed, keep "unreachable until <next try>"
   in memory, so the status check isn't repeated per message.

**After:** 10 dead inboxes cost one round of 20 s across two lane batches. A fresh request to a
live owner goes out in seconds whatever else is waiting. N4 in round 1 is the short version of
this.

### 3.3 Content sets kept in memory (K8)

**Measured:**
- The game's logs show a content build of 7–9 ms: 5–6 mods and packages, 116–122 blocks, about
  2.6k states, 231–287 items (`run/logs`).
- `modloader::use_packages` (`modloader.rs:594-635`) leaks each built `Content` with
  `Box::leak` and keeps it in `built` for the life of the game, once per distinct set of package
  hashes. Hopping back and forth between two worlds builds two.
- **The GPU side doesn't grow.** `GameTextures::reload` (`client/textures.rs:278-345`) destroys
  the old textures and buffers on every content change, and textures load lazily.

**So:**
- The cost is the CPU-side `Content`: states, block and item definitions, the registry. It's
  likely 1–5 MB each at today's size.
- A player visiting 20 worlds with 20 different package sets in one session holds about
  20–100 MB.
- Freeing them is not cheap: `content()` hands out `&'static Content` everywhere (server scopes,
  jobs). An `Arc` would mean touching a lot of Weft.

**Do:**
- **S:** log the process's resident memory before and after each build (one call: `getrusage`,
  or `mach_task_basic_info` on macOS), and `built.len()` in the log line that's already there.
  One session of portal hopping by Tester then gives the real size.
- **Only if it proves over about 10 MB a set:** free unused sets at a safe point. That's a swap
  that happens while no world but the new one is loaded and no server `Scope` holds the set. It
  rarely happens with a home world running, so it would mostly help players who visit without
  hosting.
- Otherwise leave it. A distinct package set per world, times worlds visited in one sitting,
  stays small for a long time.

---

## Where each piece goes

| piece | to | after |
|---|---|---|
| E1 (send and activation ranges) | engine; Ronald's yes (rule 4); offer upstream | — |
| 3.1 sharded pools | animals session | R1 |
| 3.2 outbox lanes | Phase 3 leftovers | R1, N1 |
| 3.3 measure content sets | Tester's next portal run (one log line) | — |
| 1.1 letters | Phase 3 leftovers, base mod for the mailbox | R1, 3.2 |
| 1.2 pedigrees | animals session; signing beside Phase 4 | — |
| 1.3 genes through portals | animals session with Phase 3 leftovers | P4, 3.1, N1 |
| 1.4 network atlas | Phase 5's first slice | status v2 |
| 1.5 real-place worlds | worldgen owner (Voxel dog handoff) | — |
| 1.6 passports and guest books | Phase 4 | its slices 1–2 |

## Not checked

- No game was run.
- The pool counts per cell and the cells per hour are worked out from the spawn radius and the
  habitats, not counted in a played world. The sizes per pool are measured.
- The saving percentages assume entities spread evenly.
- The `Content` size is estimated, which is why 3.3 starts by measuring it.
