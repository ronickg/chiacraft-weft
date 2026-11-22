# Research round 18: how many players one world holds, and what 20,000 would take

Researcher, 2026-11-05, for Planner and Ronald. Ronald asked how many people can join one world and how to benchmark it, then: "Yeah, let's give it a go, but let's host it on my PC. And let's see how many bots can join. But I'm hoping it's very efficient because it's written in Rust." and "My goal is to have like 20,000 people on one server. That would be amazing."

**What was run:**
- **The server:** a headless world server on Ronald's PC (16 cores, 91 GB, CachyOS), build `C_e9dadf18`.
- **The bots:** `weft-bots` from `crates/bots` (this branch) in one process on the same PC, joining over loopback.
- **Four runs**, each a staircase held 60 s a step:
  - bunched at the spawn, 10 → 2000 bots (`r18a`, `r18b`);
  - spread over 3,000 blocks, 250 → 2000 (`r18c`);
  - spread over 30,000 blocks, 250 → 1500 (`r18d`).
- **Clean slots:** runs b to d ran in Tester 2's exclusive slots (no game windows beside them).

**Files:**
- Raw logs in `reports/research-18/pc/<run>/`; the parser is `reports/research-18/parse.py`.
- The run script is `~/Coding/chiacraft-archive/tester-tools/bots/ramp.sh` (on the PC: `~/looktest/bots_ramp.sh`).
- The sources for other games are in [research-18/architectures.md](research-18/architectures.md).

## The short answer

1. **One world on one server today holds about 300–400 players spread out, or about 600 bunched together.** Past that its 20 ticks a second slow down:
   - bunched at 750, 16.6 ticks a second; at 1000, 10;
   - spread at 500, 8–11.

   Up to there it holds: no failed joins, no disconnects, joins in 0.03–0.7 s, and a command answered in 25–50 ms.
2. **It isn't the computer that runs out. It's one thread.** The server's tick used about 1 of the PC's 16 cores at every size. The parallel region ticks are a small part, and the big parts run one player after another on that thread:
   - **sending each player what changed:** in this "sync" phase each player walks every entity in the world (no spatial index), so it grows as players²;
   - **handling messages:** finding a player's region is a linear search over all regions;
   - **choosing terrain to make.**
3. **Rust isn't the limit; the design of these few loops is.** Hyperion, a Rust server for the Minecraft protocol, runs 5,000 bots at 1.42 ms a tick on a 14-core laptop because it indexes who sees whom and spreads the work. Ours costs about 0.05 ms per bunched player at 250 and grows from there. With the first four fixes below, one world on this PC should reach about 2,000–4,000. That is an estimate from the measured costs, not a run.
4. **20,000 on one server isn't done anywhere.**
   - **Records:** EVE Online's record is about 6,700 in one star system, at a tenth of normal speed; Folia's (a multithreaded Minecraft server) is 1,155.
   - **Bandwidth:** 20,000 players at the 22–112 KB/s each that our bots received on the wire is 0.4–2.2 GB/s going out of one machine. A home connection carries a small fraction of that. Ronald's PC is on Wi-Fi.
   - **How the big ones do it:** they split players across many processes that meet at edges. **Our linked worlds already are that split.** 20,000 people in one game is reachable as, for example, 50 linked worlds of 400 on a few machines, joined by portals, a shared chat and friends layer, and a listing that sends arrivals where there's room.
5. **Hosting at home:** the network is the limit before the CPU. Players ≈ upload speed in KB/s ÷ 20–110 (spread to bunched). A 50 Mbit/s upload (about 6,000 KB/s) is roughly 50–300 players.

## How it was measured

- **The bots (`crates/bots`, binary `weft-bots`):**
  - Each bot is a real client: it joins over the game's own protocol (iroh QUIC, ALPN `weft/1`). The hello, the world's packages and the content hash are as the game sends them, then `Join` and `View`.
  - It moves 20 times a second along a random walk at walking speed, about 40 blocks above its spawn's height. Once in 30 s it starts and finishes digging the block under it, and once in 5 minutes it chats.
  - Every 10 s it sends a command, and the answer comes from the server's tick. Its round trip is the "ping" below: how late the server is.
  - It counts everything it receives. It stands still until its first view has come.
  - All bots share one tokio runtime with one iroh endpoint each. 2000 bots took 1.1 cores and 2.3 GB, so the bot process never competed with the server.
- **The server (`weft --server --stats 5 --log net=debug`)** logs every 5 s:
  - ticks per second, and the tick time (average, p50, p99, max) split into phases: insert, messages, regions, request, sync;
  - players, loaded cubes and entities;
  - its traffic by message kind (bytes as encoded, before compression);
  - its memory, threads and CPU by thread name.

  These hooks are new on this branch (`notes/ENGINE_CHANGES.md`, section 8).
- **On the wire** (after lz4 and QUIC) is what the bots' connections counted: `wire KB/s per bot`.
- **Limits of the method:**
  - Loopback only: no real network, no packet loss, no relay.
  - Bots float and don't build, so their digging is refused for reach (the server still checks every one).
  - The "view" time is only meaningful for spread bots; bunched ones keep getting block changes from each other.

## The curve

Each row is the average over the last 60% of a step's 60 s. "Over budget" means the tick took more than 50 ms, so the world ran slower than 20 ticks a second.

### Bunched at the spawn (everyone sees everyone): `r18a`, `r18b`

| bots | ticks/s | tick avg ms | p99 | sync | regions | server cores | memory | out of the server (raw) | per bot on the wire | command p50 / p99 ms |
|---|---|---|---|---|---|---|---|---|---|---|
| 10 | 20.0 | 0.50 | 1.4 | 0.1 | 0.4 | 0.04 | 0.3 GB | 1.1 MB/s | 23 KB/s | 32 / 36 |
| 50 | 20.0 | 1.97 | 7.0 | 0.5 | 1.3 | 0.09 | 0.4 GB | 5.8 MB/s | 37 KB/s | 28 / 49 |
| 100 | 20.0 | 4.03 | 18 | 1.4 | 2.2 | 0.17 | 0.6 GB | 13 MB/s | 57 KB/s | 24 / 49 |
| 250 | 20.0 | 12.3 | 34 | 6.3 | 5.3 | 0.54 | 1.1 GB | 59 MB/s | 112 KB/s | 30 / 50 |
| 500 | 20.0 | 30.3 | 320 | 19.5 | 10.3 | 1.3 | 2.0 GB | 72 MB/s | 71 KB/s | 33 / 443 |
| **750** | **16.6** | 60.4 | 113 | 43.1 | 14.7 | 2.3 | 3.1 GB | 303 MB/s | 219 KB/s | 29 / 147 |
| 1000 | 10.0 | 100 | 170 | 75.6 | 19.8 | 2.2 | 3.5 GB | 342 MB/s | 187 KB/s | 53 / 196 |
| 1500 | 3.8 | 266 | 466 | 214 | 40.1 | 2.4 | 5.5 GB | 294 MB/s | 100 KB/s | 140 / 409 |
| 2000 | 2.6 | 390 | 542 | 308 | 47.5 | 2.3 | 12.1 GB | 388 MB/s | 94 KB/s | 200 / 493 |

- **Sync** follows 7.7 × 10⁻⁵ × players² ms almost exactly: 43 at 750, 77 predicted at 1000 (75.6 measured), 308 at 2000 (308). Regions grow linearly, about 0.02 ms a player.
- **Traffic:** at 750, Moves are 234–272 of the 288–352 MB/s, then come other entities' components (24 MB/s), cubes, cracks, and each player's own components (4.6 MB/s).
- **Memory** climbed to 12 GB at 2000 although the area holds few cubes. The send queues to each player are unbounded, so when the server falls behind, they fill.
- **The 500 row's p99 of 320 ms** is the burst of 250 joins at that step (the log's max ticks fall at the step's start).

### Spread over 3,000 blocks: `r18c`

| bots | ticks/s | tick avg ms | sync | regions | request | insert | messages | cores (terrain) | memory | loaded cubes | per bot on the wire | view p50 s | command p50 / p99 ms |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 250 | 20.0 | 31.6 | 11.0 | 4.4 | 12.5 | 1.6 | 0.5 | 10.8 (9.8) | 3.1 GB | 243k | 22 KB/s | 15 | 24 / 125 |
| **500** | **10.6** | 97.8 | 40.2 | 27.3 | 24.7 | 7.1 | 1.0 | 8.9 (7.2) | 6.4 GB | 643k | 12.5 KB/s | 47 | 51 / 215 |
| 1000 | 3.4 | 298 | 134 | 88.5 | 54.5 | 10.4 | 18.3 | 5.4 | 9.6 GB | 898k | 8.2 KB/s | 26 | 153 / 467 |
| 2000 | 0.9 | 1071 | 422 | 259 | 188 | 11.7 | 177 | 2.9 | 12.3 GB | 1.1M | 4.9 KB/s | 25 | 532 / 1498 |

### Spread over 30,000 blocks: `r18d`

| bots | ticks/s | tick avg ms | sync | regions | request | messages | memory | per bot on the wire | view p50 s | command p50 / p99 ms |
|---|---|---|---|---|---|---|---|---|---|---|
| 250 | 19.9 | 18.0 | 6.5 | 3.7 | 7.5 | 0.7 | 2.3 GB | 8.9 KB/s | 4.1 | 26 / 87 |
| **500** | **7.6** | 136 | 56.1 | 43.2 | 22.1 | 8.6 | 5.2 GB | 6.5 KB/s | 7.5 | 64 / 340 |
| 1000 | 3.0 | 340 | 130 | 45.4 | 56.8 | **103** | 7.9 GB | 2.2 KB/s | 1.9 | 169 / 482 |
| 1500 | 1.0 | 1214 | 235 | 29.0 | 79.3 | **851** | 10.2 GB | 1.4 KB/s | 3.3 | 666 / 2009 |

Spread players cost more each than bunched ones:
- Each loads its own ground: about 1,000–2,500 cubes and about 10–13 MB of memory per player.
- Generating it took up to 11 cores (the "background" threads): the only part that used the PC's cores.
- Then, per player and one after another:
  - choosing what to generate (request: 7–188 ms);
  - streaming its cubes (sync);
  - ticking more loaded world (regions).
- **"Messages" explodes in the widest spread** (103 ms at 1000, 851 at 1500), where every player is in a region of its own. Every Move looks up the player's region (`server/actions.rs:45-49` `region`) by searching the list of regions (`server/regions.rs:167-179`, `entity_region` and `entity_region_ref`), so the cost is players × regions per tick.
- Per bot on the wire falls past the knee only because the ticks slow down.

Nothing failed in any run: 0 failed joins and 0 disconnects (four joins were told "the world is busy, try again", which the bots did, as a game does). Join p99 stayed under 0.7 s, except 6.8 s once at 1500 spread.

## Where the tick goes, and why

| phase | what runs | how it grows | evidence |
|---|---|---|---|
| sync | for every player, one after another (`server/sync.rs:38-78`): send_far, block changes, mining, the clock, **entities** (`send_entities` walks every entity of every region and checks its distance; `collect_moves` and the `moves` filter walk every moved entity), own state, cubes | bunched: players² (7.7e-5·N² ms); spread: players × their cubes | 308 ms at 2000 bunched; 422 ms at 2000 spread |
| messages | every message, one by one; a Move calls `self.region(pi)`, which searches the regions for the player's entity | players × regions | 851 ms at 1500 widely spread, 0.4–8 ms bunched (one region) |
| request | choosing cubes to generate: a visibility walk per player, every tick | players × view | 188 ms at 2000 spread |
| regions | region ticks (entities, blocks, liquids): in parallel (`mod.rs:693`), but one region when bunched | linear | 47 ms at 2000 bunched (one region, so one thread) |
| network | iroh on **2 tokio threads** for the whole server (`p2p.rs` `runtime`), plus **one OS thread per player** to encode (`pump_out`, "net-out"): 2,048 threads at 2000 | linear | net 0.9–1.3 cores at 750–2000 bunched; not yet the limit |
| players list | every tick the server builds the list of all names; on any join or leave it sends **the whole list to everyone** (`mod.rs:733-743`) | players² bytes per join | small next to Moves here, but 2000 joins × 2000 players × the list |

A Move on the wire is fixed-width: an 8-byte id, three 8-byte floats, two 4-byte floats, the pose and an 8-byte frame, 49 bytes, sent to everyone in range every tick (`net/mod.rs` `Move`). 750 bunched players × 750 × 20 a second × 49 B is the 230–270 MB/s measured. Compressed with lz4 it drops to the 219 KB/s per bot on the wire.

## What it would take, ranked

Gains are estimates from the measured phase costs; each needs its own run. All are engine changes (`crates/engine`, each needs a `notes/ENGINE_CHANGES.md` line).
- **After the sync:** the upstream Weft sync reworks `mesh_draws` and camera modes but not the server's sync loop, so most can land after it without clashing. #1–#3 touch `server/sync.rs`, which upstream may have changed (check first).

| # | change | where | expected gain |
|---|---|---|---|
| 1 | **Index entities by place, and cap who each player hears.** A grid of entities per section, read per player instead of walking all of them. At most the K nearest players at full rate (Hyperion caps at about 700, Albion uses 10 m cells); farther ones less often (`move_every` already has rates; it needs the index). | `server/sync.rs` `send_entities`, `collect_moves`; `regions.rs` | Bunched sync from N² to N·K: at 2000 with K = 200, 308 ms → about 30 ms; Moves traffic about 10× less |
| 2 | **Find an entity's region by a map, not a search** (entity id → region index, kept when an entity changes region) | `server/regions.rs:167-179`, its callers | Messages at 1500 wide spread 851 → about 1 ms; the same in sync's lookups |
| 3 | **Sync players in parallel.** Each player's sends read the world and write only that player's state: a `par_iter_mut` over players, with the shared lists (moved, settled, mining) made before | `server/sync.rs:38-78` | Sync divided by about the cores: about 8–12× on this PC, for bunched and spread alike |
| 4 | **Smaller moves:** a u32 id, the position relative to the player at 1/32 block in i16, yaw and pitch in a byte each, the frame only when not 0, and only fields that changed. About 49 → 10–13 bytes | `net/mod.rs` `Move`, `codec.rs`, the client's receive | Moves bandwidth and encoding about 4× less, before lz4 |
| 5 | **Network threads that scale:** encode on the tokio runtime instead of a thread per player, more workers than 2 (cores/2), and **bounded queues per player** that keep only the newest moves when a player falls behind | `net/p2p.rs` `runtime`, `pump_out`, `accept` | 2,048 threads → 16; memory stops climbing (12 GB at 2000); keeps the server alive past the knee |
| 6 | **Terrain choice and streaming incrementally:** `request_generation`'s per-player visibility walk only when the player has crossed a cube boundary, and spread over ticks; the same for `send_cubes`' unsent lists | `server/load.rs`, `server/sync.rs` `send_cubes`, `send_far` | Spread request 22–188 ms → a few ms; sync's spread share down several times |
| 7 | **The players list as joins and leaves**, not the whole list to everyone each time, and not rebuilt every tick | `server/mod.rs:733-743`, `ToClient::Players` | Removes an N² burst whenever someone joins (thousands joining at once) |
| 8 | **Own components only when they change, or at most 4 a second**; other entities' components likewise (each player got 20 of each a second, 375 B for its own) | `server/sync.rs` `send_own_state`, `send_entities`; base mod's synced components | 6–30 MB/s less at 750 bunched |
| 9 | **Time dilation (EVE):** when the tick runs over, slow the world's clock (to 10% at worst) and tell clients, instead of falling behind and filling queues | `server/mod.rs` `run` | No capacity, but overload becomes slow motion instead of lag and memory growth |
| 10 | **A smaller simulated area per player when crowded:** cut `core:simulation_distance` as the player count rises (clients draw beyond it themselves) | config, `server/mod.rs` view | Spread: fewer loaded cubes (10–13 MB and about 1,000+ cubes a player) and less region and request work, roughly with the area |

With #1–#5 (all inside the current design), the per-player cost is mostly linear and spread over the cores, so on this PC one world should hold about 2,000–4,000. Past that:
- **Per-player linear work.** About 0.02 ms of regions alone per bunched player is 400 ms for 20,000 on one thread.
- **Bandwidth.** 20,000 × 22–112 KB/s is 0.4–2.2 GB/s.

Neither goes away on one machine.

## 20,000: what the others do, and what fits linked worlds

Sources and numbers: [research-18/architectures.md](research-18/architectures.md).

- **Minecraft (Paper):** one main thread, about 200 players.
- **Folia:**
  - **How:** splits a world into regions of nearby chunks that tick on separate threads.
  - **Players:** 1,155 on the biggest server, 6b6t.
  - **Fit:** Weft already has Folia-style regions (`server/regions.rs`), but our per-player phases around them are serial. Fixes #2 and #3 are what let regions pay off.
- **EVE Online:**
  - **How:** one process per star system, systems joined by gates (never a seamless border), busy systems on their own machine, and time dilation to 10%.
  - **Players:** 6,557 in one fight, and 65,303 online at once in the whole game.
  - **Fit:** EVE's star systems are our linked worlds, and its gates are our portals.
- **Albion Online:** one-threaded zones of about 1 km² holding 300 players each, about 600 zones, and handoff through the database.
- **Hyperion (Rust):** 5,000 bots at 1.4 ms a tick. It caps each player's view at about 700 others and uses proxies to multicast.
- **WoW and Guild Wars 2:** copies of busy areas (layering, megaserver). This only fits for temporary or read-only places, since block edits would make copies diverge.
- **SpatialOS:** seamless borders across machines; its games were cancelled.

**What fits our design best: linked worlds as shards.** For example, 50 worlds × 400 players, or 100 × 200, on a few machines. That needs:
- the world listing to show how full a world is and steer arrivals to worlds with room (like Guild Wars 2's megaserver);
- a small shared layer across worlds for identity, friends and chat (BitCraft's `global_module`; our accounts already carry the identity);
- portals as the gates (already built);
- a busy world moved to a bigger machine (EVE's "reinforced" nodes).

Inside each world, #1–#5 above. That is how 20,000 people can play "one server" without anyone having built a single simulation that size.

## Not checked

- **A real network.** Bots on another machine, packet loss, the relay, Ronald's Wi-Fi upload. A second bot process on the Mac would join over the LAN, but Mac runs need Ronald's yes.
- **A server-thread profile** (perf isn't installed on the PC). The phase split and the code read stand in for it. A flamegraph at the knee would confirm #1–#3 before building them.
- **Region parallelism.** The server's regions phase was small next to sync, request and messages even spread over 30,000 blocks. The region count wasn't logged; a line for it belongs in `--stats`.
- **Bots that build, open menus, fight or ride:** these load the server in other places (menus, entities, physics).
