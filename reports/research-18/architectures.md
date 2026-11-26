# Research round 18: how others reach big player counts (sources)

A sub-agent read web pages as text, 2026-11-05. "[unconfirmed]" means no primary source was found.

## Records and ceilings

| who | how many at once | how | source |
|---|---|---|---|
| EVE Online, one solar system | 6,557 at once (Fury at FWST-8, 8,825 total); 6,739 in M2-XFE; 13,770 across three reinforced systems | one node = one single-threaded process per system; busy systems on a dedicated "reinforced" blade; **time dilation** down to 10% speed | [Guinness](https://guinnessworldrecords.com/news/2020/10/eve-onlines-record-breaking-fury-at-fwst-8), [M2-XFE](https://www.eveonline.com/news/view/the-second-timer-in-m2-xfe), [TiDi](https://www.eveonline.com/news/view/introducing-time-dilation-tidi) |
| EVE, whole cluster | 65,303 peak (5 May 2013) | ~90-100 blades, 2 nodes each (2008 blog); systems joined by gates, never seamless | [mein-mmo](https://mein-mmo.de/en/eve-online-is-still-going-strong-after-21-years-celebrates-player-record-for-2024-after-update,1139809), [node blog](https://www.eveonline.com/news/view/my-node-was-equipped-with-the-following...) |
| Folia (6b6t, a heavy fork) | 1,155 concurrent (14 Aug 2026); "no lag at 300" | regionised multithreading: regions of nearby chunks tick independently on a pool, merge/split by proximity | [6b6t blog](https://blog.6b6t.org/how-6b6t-plans-to-reach-1000-players-online-2/), [Folia](https://github.com/PaperMC/Folia), [region logic](https://docs.papermc.io/folia/reference/region-logic) |
| Folia's own sizing | test server ~330 peak; guide: >= 16 cores, per 200-300 players ~4 net threads, 3 chunk IO, 2 chunk workers | | [Folia README](https://github.com/PaperMC/Folia) |
| Paper / vanilla (2b2t before Folia) | ~200 | one main thread; Paper adds async chunks and a distance map in the entity tracker | [holy.gg](https://www.holy.gg/en/post/how-to-improve-your-survival-server-s-performance-with-folia), [Paper patch](https://ayakael.net/mirrors/papermc/blame/commit/a0a5615e6e45093edd3350613c84684e5387e20f/patches/server/Use-distance-map-to-optimise-entity-tracker.patch) |
| Albion Online | 300 per cluster (single-threaded, ~1 km2, ~500 mobs, 10k+ objects); ~15k concurrent in all | ~600 clusters; handoff through the database; grid interest of 10x10 m cells | [2016 talk](https://www.slideshare.net/slideshow/albion-online-software-architecture-of-an-mmo-talk-at-quo-vadis-2016-berlin/62724504) |
| BitCraft (SpacetimeDB) | ~3k concurrent (headline) | one `game` module per region + a `global_module` (identity, empires, which region a player is in) | [SpacetimeDB](https://github.com/clockworklabs/SpacetimeDB), [deepwiki](https://deepwiki.com/clockworklabs/BitCraftPublic/9-global-module) |
| Hyperion (Rust, Minecraft protocol) | **5,000 bots at 1.42 ms a tick** on a 14-core MacBook (bot-only); each player sees at most ~700 others | ECS; tick split into ingress, systems, egress; **proxies** multicast the broadcasts; BVH for local broadcast | [hyperion](https://github.com/hyperion-mc/hyperion), [reddit](https://r.enesgenc.dev/r/rust/comments/1fmznnh/hyperion_10k_player_minecraft_game_engine/lohestr/?context=3) |
| WoW | "~3,000 per layer" [unconfirmed] | layering (whole-realm copies) and sharding (zone copies over a cap) | [forum](https://us.forums.blizzard.com/en/wow/t/layering-is-exactly-the-same-as-sharding-lets-do-some-math/220522) |
| Guild Wars 2 | as many map copies as needed | megaserver: copies by party, guild, language | [ArenaNet](https://www.guildwars2.com/en/news/introducing-the-megaserver-system/) |
| SpatialOS (Improbable) | games cancelled in 2019; a later "10,000 players" demo | entity-component ownership per worker, migrating across machines | [Wikipedia](https://en.wikipedia.org/wiki/Improbable_(company)) |
| Hytale | default 100 (hosts' docs); no official capacity [unconfirmed] | Java server, QUIC on UDP 5520 | [hytale 2019](https://hytale.com/news/2019/1/an-overview-of-hytales-server-technology), [winternode](https://winternode.com/help/games/hytale/configuration/max-players) |
| Valence, FerrumC, Pumpkin, Horizon (Rust) | no published player benchmarks | | [valence](https://www.Github.com/valence-rs/valence), [pumpkin](https://snowiiii.github.io/Pumpkin-Website/about/benchmarks.html), [horizon](https://github.com/horizon-rust/horizon) |

**Nobody has published 20,000 real players interacting in one simulation.**

## Interest management and bandwidth

- **Grid interest.** Unreal's Replication Graph (Fortnite) keeps actor lists per grid cell, so a client copies its cell's list instead of a distance check per actor ([Epic](https://www.unrealengine.com/tech-blog/replication-graph-overview-and-proper-replication-methods)). Albion uses 10×10 m cells.
- **The N² problem.** In Quake III each player needs about 12n kb/s for n players ([Donnybrook](https://microsoft.com/en-us/research/wp-content/uploads/2016/02/donnybrook.pdf)). Source uses delta snapshots against the last acknowledged state at 20 updates a second.
- **Minecraft bandwidth.** About 30–50 KB/s steady, with bursts of 200–500+ KB/s while exploring (vendor figures, [papyrus](https://calculator.papyrus.vip/)).
- **20,000 × 50 KB/s is about 1 GB/s (8 Gbit/s)** out of one machine.

## Which fit a world-per-server, portals-between-worlds design

| design | fit | what it takes | what it buys |
|---|---|---|---|
| (a) regions of one world on threads (Folia) | fits, inside one world | regions that own their chunks and entities (merge/split by proximity, no two ticking neighbours), per-region deadlines, cross-region actions as queued messages | 3-5× for spread-out players; nothing for one dense crowd |
| (b) regions on separate machines with seamless handoff (SpatialOS) | poor, costly | ownership migration, border replication | its games died; EVE avoids seams on purpose |
| (c) layering / instancing (WoW, GW2) | partial | copies of a busy area | hubs, events, read-only or temporary worlds; persistent edits make copies diverge |
| (d) linked worlds as shards (EVE systems, Albion clusters) | **best fit** | the chain listing as a load balancer (fullness, steering arrivals), handoff through the account, a small global layer (identity, friends, chat), bigger machines for busy worlds | how 20,000 is reachable: e.g. 100 worlds × 200 |
| (e) time dilation (EVE) | fits cheaply | scale dt in the fixed tick when it overruns, 10% floor, tell clients the factor | graceful overload, no extra capacity |
