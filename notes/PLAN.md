# chiacraft on Weft: the plan

Status: draft, 2026-10-31. Decisions are Ronald's from the planning conversation; "Proposed" marks
what still needs his yes.

## What we are building

Players open their own worlds from the game, find other people's worlds without any central
service, ask to link them with portals, and walk between them. Missing mods download on the way,
with consent. Identity and achievements live on Chia, so nobody can take a player's account or
erase what they earned.

## Rules every phase must pass

1. **Decentralised by design.** No single hard dependency. Every service (relays, sponsors,
   directories, package mirrors) has a setting and can be run by anyone. Single player and LAN
   work fully offline. If we disappeared, players keep their identity, achievements, worlds and
   links; they lose only conveniences.
2. **Freedom for owners.** The game gates nothing. Every gate (admission rules, personal mods,
   inventory carry-over, native mods) is the world owner's choice. Defaults are open.
3. **The chain only where it adds trust.** Now: accounts (DIDs) and achievements. Never in the
   game loop. Not for portal links, bans or inventories.
4. **Leave Weft's engine alone where we can.** Our features live in our own crates and mods; the
   engine changes only for identity, the mod sandbox, content switching, the confirm dialog and
   the server mode. Weft is tracked as `upstream` and merged regularly. Weft files stay MPL-2.0,
   ours are MIT. The name players see is chiacraft.
5. **Players are safe in any world.** Downloaded code is sandboxed; servers can never sign
   anything for a player; mods never reach the account.

## Decisions so far

| Topic | Decision |
|---|---|
| Base | Fork of Weft (`f213253`), Minecraft texture importer removed |
| Account | 24 words, made quietly at first start, kept in the OS keychain (`crates/wallet`) |
| Login key | Derived from the words on its own hardened path; can never spend; becomes the iroh key |
| On chain, for now | Player DIDs, world DIDs, achievement anchors. Items, trades, fees later |
| DIDs | Effectively free (1 mojo, usually 0 fee); sponsors give the first mojo; anyone can run one |
| Fees | 0 by default, retry with a small fee when the mempool is full |
| Portals | Link two **worlds**, for all their players |
| Linking | Always a request the other owner accepts, unless the owner pre-allows requests |
| Hosting | A world is joinable while its owner's game runs; headless servers for always-on |
| Discovery | Decentralised: on chain and/or DIG (see "Finding worlds") |
| Mod consent | Asked the first time; trusted mods and creators are not asked again |
| Personal mods | Allowed or not per world, owner's choice |
| Native Weft mods | Allowed, off by default; each native mod set is its own universe, entered with a restart (see "Native mods") |
| Inventory carry-over | Per world (or per link), owner's choice, off chain |
| Bans | By login key and DID; probation, DID age and shared lists make them stick |
| Gating | Open by default; owners combine rules (allow lists, DID age, achievements, NFTs later) |
| World ownership | World DID names the owner's DID and the owner's DID lists the world; both ways |

## Architecture

**Your game:** screens (discover, requests, consent), account (words, login key, DID), package
manager (cache, verify by hash), sandboxed client mods, engine (render, predict, switch content).

**A world** (inside a game or headless, same code): simulation (Weft regions, saves, catch-up),
admission and bans, links and requests inbox, package host, sandboxed server mods, a working key
authorised by the world's DID (the DID itself stays in the owner's wallet).

**Between them:** one encrypted iroh connection carries requests, links, package downloads and play.

**Shared and replaceable:** iroh relays, the Chia chain (DIDs, anchors), DIG (lists, mirrors),
sponsors (free DIDs).

### Finding worlds (proposed)

- **On chain, the base list.** A world that chooses to be listed makes a 1-mojo coin hinted with a
  well-known chiacraft tag; its DID record holds the name, description and current key. Any game
  asks any Chia full node for coins with that hint and gets every listed world, with no directory
  service at all. The wallet already syncs hinted coins (`chain/holdings.rs`).
- **DIG for the rich part.** Screenshots, longer descriptions, and curated lists ("hardcore
  worlds", "Ronald's picks") as DIG stores anyone can publish.
- **Live status from the world itself**: online, players, version. Never on chain.
- **Spam:** listing is nearly free, so the game sorts and filters by world DID age, achievements
  issued, links from known worlds, and the curated lists the player follows.

### Mods: what downloads

| Tier | Downloads | Runs |
|---|---|---|
| Content packages (blocks, items, recipes, models, art, sounds, UI layouts) | automatically, with consent | data only |
| Shader packs | automatically, with consent | validated before the GPU runs them |
| Client mods (HUD, screens, effects, prediction) | automatically, with consent | sandboxed WebAssembly |
| Personal mods (minimap, tweaks) | installed by the player | sandboxed; allowed per world |
| Native Weft mods | only if the player allows trusted creators; restart into their universe | unsandboxed, own universe |
| Engine features (renderer, physics, networking) | part of the game | built in |

Each world pins exact package versions by hash; the cache keeps versions side by side; the game
switches content when a player walks through a portal.

### Native mods: their own universes

Full freedom, clearly separated (decided 2026-10-31):
- **The main universe is package-based worlds.** They all link to each other; travel by portal is
  seamless (download, swap content in place, no restart).
- **Each native mod set is its own universe.** Native code can't be unloaded and Weft freezes
  content at start, so a set of native mods is fixed while the game runs. Worlds with the same set
  (the same hashes for every native mod) link and travel seamlessly among themselves.
- **Between universes is a restart.** A portal or link into a native universe asks "This world
  needs unsandboxed mods: X, Y. Restart into it?"; the game restarts itself with that set and joins
  the world. Coming back is another restart. Each universe gets its own mod profile, like a
  modpack instance, so the player's own setup is untouched.
- **Off by default.** Setting: "Allow native mods from creators I trust", with a clear warning.
  Listings and portals mark these worlds "unsandboxed".
- **Checks:** a hash per system (Windows, Linux, macOS) proves the file is the published one and
  that everyone runs the same build; the creator's DID signature proves who made it; reviewers'
  signed hash lists (on DIG) add trust. Hashes prove integrity, not safety. Reproducible builds
  later.
- **Account safety:** while any native mod is loaded, the account's secrets are not in the game
  process: signing and login go through a small signer helper the game starts.
- **A bridge for authors:** our WebAssembly mod interface mirrors Weft's SDK where it can, so many
  Weft mods recompile to WebAssembly with few changes and join the main universe.

### Inventory carry-over (proposed)

Off chain, per link: two linked worlds can both enable "carry inventories between us". Leaving
moves (never copies) the inventory: the origin world signs it over with its DID and the destination
accepts it only from worlds it trusts. Items whose content the destination lacks wait in a travel
bag until the player returns.

## Phases

The smallest version that works is phases 1-4.

1. **Account.** Wallet crate builds and passes its tests here; login key from the words replaces
   Weft's random `identity.key`; join proves the login key's link to the DID; Account screen
   (fingerprint, show words once, I wrote them down, restore); one signing interface.
2. **Packages.** Data packages through Weft's registry; package list with hashes on join; consent
   screen with trust memory; download over iroh; cache; switch content in place (Weft freezes
   content at load today; Horizon's `content_swap.rs` solved the same problem).
3. **Portals.** World addresses (key, DID later); link requests inbox with pre-allow; links
   signed by both worlds, revocable; portal blocks; travel with the connection prepared ahead; a
   safe return when a world goes offline; keep your own world open while you visit others.
4. **DIDs and achievements.** Player and world DIDs through sponsors; world ownership both ways;
   achievements signed by the world DID, anchored in batches, opt-in; profile view; admission
   rules (allow lists, probation, DID age, achievements); shared ban lists.
5. **Finding worlds.** On-chain listing by hint; DIG listings and curated lists; spam ranking.
6. **Sandboxed code mods.** wasmtime with components (from `horizon-mods`), WIT v1.0 shaped on
   Weft's SDK, server halves then client halves, frozen API versions; personal mods per world.
7. **Headless server and administration.** No window or GPU (Weft's CPU terrain path),
   `server.toml`, console, operators and command permissions (today any player can `/give`),
   movement checks, our own relay, load tests (`horizon-bots`).
8. **Later.** Wallet view and spending with the confirm dialog, items on chain, offers, passes,
   paid links, portal-rights NFTs; launcher and signed updates.

Game content (blocks, items, crafting, gameplay systems) is out of this plan on purpose: it gets
its own plan once the server, universe and account foundations stand. Until then Weft's base game
is the content.

## Open

- crates.io access for the cloud session (builds), the new GitHub repo, the GitHub connection.
- Verify Chia full nodes return hinted coins for an arbitrary hint fast enough for a world list.
- Weft upstream: keep merging (proposed) or cut loose.
- The Horizon-based project: frozen as it is, W7-W9 work left in its worktrees.
