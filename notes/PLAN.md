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
   inventory carry-over, native mods) is the world owner's choice. Defaults are open to
   *receiving* (requests, visitors), never to auto-accepting strangers.
3. **The chain only where it adds trust.** Now: accounts (DIDs) and achievements. Never in the
   game loop. Not for portal links, bans or inventories.
4. **Leave Weft's engine alone where we can.** Our features live in our own crates and mods; the
   engine changes only for identity, the mod sandbox, content switching, the confirm dialog and
   the server mode. Weft is tracked as `upstream` and merged regularly. Weft files stay MPL-2.0,
   ours are MIT. The name players see is chiacraft.

   **Engine changes we offer back to Weft** (only with Ronald's yes for each):
   - Sent: the GPU fix, https://github.com/Weft-Rust/Weft/pull/83.
   - To offer later: entity send and activation ranges (E1, `reports/Research round 2.md`).
     These are the rules `core:entity_send_distance`, `core:entity_activation_distance` and
     `core:inactive_entity_interval`, the `ALWAYS_ACTIVE` flag and `tick_elapsed`. Ronald's
     yes, 11-04: "Yes, later". Offer it once it has run in `dev` a while.
   - To offer: worn layers mapped by the skin's layout (`client/entity_render.rs`,
     `worn_tex_size`), in use since Ronald's "all 32" for the player and clothes (11-04). A
     128 x 64 armour or clothes layer on a 64 x 32 skin then maps over all of it, as Minecraft's
     armour does in 32x packs; one at the layout's width reads as before. One general line and a
     test (branch `claude/clothes-worn`).
   - To offer: settings carried over by name on a content switch (`config::content_switched`,
     phase 6). Weft numbers settings by name within each content but keeps their values in one
     table by id, so switching to a content with one more setting (a mod's key) moved the values
     after it: a joining player read an empty name. Now the values follow their names into the new
     content's ids, and mods' setting handles get the new content's ids back. Small, general,
     with a check (`a_content_switch_keeps_values_by_name`).
   - To offer: room for more worn layers (`client/models.rs`, `MAX_LAYERS` 8 to 12). Armour's
     four, clothes' four and a cloak are nine over the skin, and past 8 the rest were dropped
     silently (the cloak never drew). One constant (branch `claude/cloaks`).
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
2. **Packages.** Decided 2026-10-31 after research (`reports/Data packages for moddable voxel
   games.md`), in slices:
   1. *Local packages* (done): `package.toml` + `defs/<kind>/<path>.toml` (TOML; id from the path)
      + `assets/` like a mod's; registered through the SDK by the `packages` mod into their own
      slots; package ids sort after code mods' so packages never move them; broken packages are
      left out, not fatal. Format: `docs/packages.md`.
   2. *Identity and store* (done): a package is the BLAKE3 hash of a canonical manifest listing every
      file's path, size and BLAKE3 (not a hash of a zip); files cached by hash in
      `run/cache/blobs/`, never unpacked to author-chosen paths; each world keeps a lockfile of
      exact package hashes; unknown blocks in saves load as a placeholder that keeps their name.
   3. *Content swap between worlds* (done): an atomic pointer to `Content` interned by hash (each kept
      once); swap only with no world loaded; native mods re-resolve package ids by name per swap.
   4. *Configuration phase and transfer* (done; files are checked whole, by BLAKE3, not piece by
      piece with bao-tree yet): protocol 4; the hello carries the world's lock hash; then
      offer, have/need/declined, verified streaming (bao-tree) on extra streams of the iroh
      connection, status per package, and the hash of the content actually loaded before play.
      Joining stops freezing the window (a progress screen instead).
   5. *Signing and trust* (done): a content-signing key derived from the words on its own path and
      certified by the account (DID later); signatures in an envelope outside the pinned hash;
      join sheet with sizes, creators and key fingerprints; trust defaults to this package or this
      world's set, trusting a creator is a deliberate step.
   6. *Hardening* (done): format allow-list by folder and extension with a size per kind
      (`identity::check_file`, enforced on import and on every downloaded manifest); PNGs capped at
      4096 a side and decoded under `catch_unwind`; package reading can't panic across the FFI;
      packed frames checked before decompressing; time limits on the hello, the configuration
      phase and stalled downloads; the host serves only offered hashes, at most twice each, four
      streams a peer, eight joins at once, files streamed in pieces; the guest refuses malformed or
      oversized offers; store tidy at start (pins + installed kept, 512 MiB LRU for the rest,
      orphan files and crash leftovers removed); cargo-fuzz targets for the manifest, signature
      and definition parsers (`crates/packages/fuzz`, nightly, not run in CI yet).
   Later: iroh-blobs for downloads from other players and mirrors, reviewer lists, key rotation.
3. **Portals.** World addresses (key, DID later); link requests inbox with pre-allow; links
   signed by both worlds, revocable; portal blocks; travel with the connection prepared ahead; a
   safe return when a world goes offline; keep your own world open while you visit others. In
   slices:
   1. *Your world runs on* (done): decided 2026-10-31 that a server keeps its own world's state
      instead of a second process or one shared package set. `server::Scope` holds its world's
      `Content` and its own values of the synced settings; its thread, its own rayon pool and its
      jobs on shared pools take them as theirs (`content::for_thread`/`with`,
      `config::for_thread`/`with`), so the game can switch content for another world meanwhile.
      Joining another world while yours is hosted keeps it running (`Running.home`); "Back to
      <world>" in the pause menu; back home automatically when the visited world goes away or the
      join fails. Joins went out through the hosting endpoint while both used the login key;
      slice 2 gives worlds their own keys and joins go back to the account's.
   Decided 2026-10-31 after research (`reports/World addresses links and portals.md`):
   2. *World addresses* (done): each world has its own Ed25519 key, derived from the words on its own
      label with a **random** index saved in the world (a counter would collide across machines)
      and a generation (to replace a leaked key); the account signs a world certificate. The world
      key is the address and its hosting endpoint's id; joins go out through the account's key.
      Address `chiacraft://w/<key>?n=<name>&r=<relay>&a=<ip:port,..>` (the key alone is enough;
      the old ticket still works). Found through n0 DNS/pkarr and the Mainline DHT, relay URL
      only. A world answers `chiacraft/status/1` (name, certificate, players, packages, join
      policy); before hosting, the game pings its own world key so one world never runs twice. A
      saved-worlds list (proposed name, owner, last seen, status). Not yet: petnames, the join
      policy in the status answer, rotating a key (the generation is kept; the "world moved"
      notice needs slice 3's account endpoint).
   3. *Link requests* (done, apart from friends' worlds, "world moved", invite tokens and a block
      list screen): the account endpoint runs while the game does and is the inbox for all
      the owner's worlds (worlds are offline most of the time); requesters keep an outbox and
      retry; friends' worlds and "world moved" notices are served there (`chiacraft/worlds/1`).
      A link is one labelled record both world keys sign over the same bytes (id: its BLAKE3);
      declines are silent; either side revokes alone with a permanent tombstone; pre-allow by
      world or owner key, or an invite token. Signatures carry key ids, so world DIDs replace
      only the certificate later.
   4. *Portal blocks* (done): built into the game (the `portals` mod, not a package, whose blocks
      differ per world), holding only a link id; binding one needs the world owner (checked on the
      server; anyone can `/give` today); portal positions stay in each world. Walking in joins the
      linked world (at its spawn until slice 5), or goes home if it is yours; a player is sent only
      after a second outside every portal, so arriving in one never bounces. The mod asks the engine
      what Weft's interface doesn't carry through `bridge` (owner? links?).
   5. *Travel prepared ahead* (done, apart from warming up before the player reaches the portal
      and the few seconds' protection): a 60-second travel ticket signed by the origin world
      (`travel`), carried in the protocol 5 hello, its player key checked against the
      connection, used once, for a link the destination has with the origin; arrivals stand
      beside the destination's portal for that link (the spot the mod finds when the owner binds
      it, kept in the world's `portals.txt`), coming home too; the player stays in the world they
      are in while the trip connects, asks and downloads, and leaves only when the destination's
      packages are ready (a failed trip leaves them where they were); when a world reached through
      a portal goes away, back to the world joined before it, else home, else the title; a
      second outside every portal before a portal sends anyone.
   Extras (done): invites (a token the world signs, in its address as `i=`, one use within a
   week; a request carrying it is accepted without asking); warm-up as a player nears a bound
   portal (their game asks the destination's status through its own endpoint, and says where the
   portal leads); five seconds' protection after arriving; a scrolling Links screen; friends
   (added from a saved world's owner or a link, mutual; `chiacraft/worlds/1` on the account
   endpoint lists a player's worlds to friends, open ones joinable from the Friends screen);
   "world moved" notices (a world given a new address from the world list leaves a notice its
   owner's identity signs; saved entries follow it when the old address stops answering). Left:
   arrival spots in dimensions other than the spawn's (the SDK has no dimension id), carrying
   links over to a world's new address.
4. **DIDs and achievements.** Player and world DIDs through sponsors; world ownership both ways;
   achievements signed by the world DID, anchored in batches, opt-in; profile view; admission
   rules (allow lists, probation, DID age, achievements); shared ban lists.
   Slices (agreed 2026-11-03, the open points answered as recommended through Planner: the
   sponsor speaks iroh; ours runs on this Mac from the throwaway testnet11 wallets; it launches
   world DIDs too, with the owner's DID and the world key written in at launch, at most one player
   DID and 8 world DIDs per identity a day, and a few test mojos where testnet needs a fee;
   `testnet11` is the default chain; mainnet stays off). What is already there: `crates/wallet` reads
   DIDs from the chain, launches them for others and makes its own (`chain/did.rs`,
   `chain/sponsor.rs`, `chain/reader.rs`), signs as a DID's owner (`OwnerSigner`) and keeps a
   linked DID in `accounts.json` (`DidLink`); Horizon's sponsor service (`horizon-sponsor`: queue,
   batches, daily budget, per-IP limit, a log replayed at start, no secret in its output) ran live
   on testnet11 on 2026-10-30 from throwaway wallets kept in `~/chiacraft-testnet/`. Bans and
   admission don't need the chain and come first, so they can be tried while the DID slices land.
   1. *Admission rules per world* (built, seen working between two games 2026-11-04): `admission.json` in the world folder, read at every
      join (no restart), set by the owner from the pause menu's "Who May Come In" screen and by
      commands (`/allow`, `/probation`). Rules,
      all of which must pass: open, or only the allow list (by login key, identity or DID);
      probation for newcomers (see below); DID required and DID at least N days old (slice 6:
      `/didrule`, and on the same screen). The owner and operators always get in.
      Default: open, no probation (rule 2). A refusal is a translation key with its arguments, shown
      on the joining player's screen. The world's status answer carries a summary ("allow list",
      "DID 30 days or older"), so the Multiplayer screen shows it before joining. The dedicated
      server reads the same file; its `whitelist.txt` becomes the allow list.
      **Probation:** a player new to the world can't break or place blocks or open containers
      (proposed default) until they have played there for N hours (owner's number, 0 = off) or the
      owner lifts it (`/probation end <player>`); the owner picks which of the three apply.
   2. *Bans* (built, seen working): `bans.json` in the world: entries by login key, identity or DID, each
      with a reason, who banned, when and an optional end. `/ban <player|key|did:chia:1…>
      [duration] [reason]`, `/unban`, `/bans`; banning someone online bans their login key,
      identity and proven DID together and disconnects them; a banned DID refuses every identity
      that proves it. The same screen lists and lifts them. Probation and DID age are what
      make a ban stick: a new identity has no old DID and starts on probation.
   3. *Shared ban lists* (built, seen working): a list is a JSON document signed by its
      maintainer's identity key (a sequence number, entries with reasons). Any world that holds
      it serves it over iroh (`chiacraft/bans/1` on the world's own key; signed, so any copy is as
      good as the original). A world follows a list by a world's address (`/banlist follow
      <address>`: that world's owner's list, the owner taken from its certificate) or by
      maintainer key, fetches it then and daily while open, and applies its entries as bans, its
      own allow list winning. `/banlist publish` signs the world's bans for good as the owner's
      list. Never on by default. Not yet: lists from a URL.
   4. *The chain in the game* (built, seen working on testnet11 2026-11-04, `chiacraft/chain.rs`): the engine links the
      wallet with `chain`. One chain reader per game or server, started only when something needs
      it (getting, linking or checking a DID), on the network the setting `core:chain` names: `off`, `testnet11` (the default while we test)
      or `mainnet` (only after Ronald's yes, and a confirmation in the game); peers from the DNS
      introducers or the setting's own nodes; its cache in the run folder. Never in the game loop.
   5. *The sponsor and the account's DID* (built, seen working on testnet11): `horizon-sponsor` ported as `crates/sponsor`
      (`chiacraft-sponsor`, not a default member), speaking **iroh** instead of HTTP: ALPN
      `chiacraft/sponsor/1` on its own key, the same JSON and the same signed request (the
      wallet's `sponsor.rs`), so anyone can run one at home with no domain, TLS certificate or open
      port; HTTP stays an option for a web page. Setting `chiacraft:sponsors`: sponsor addresses,
      tried in order; the default is ours. Ours runs on testnet11 from the throwaway sponsor
      wallet. Account screen: "Get a DID" asks a sponsor to launch one to the account's first
      wallet address (so Sage shows it after importing the words), shows it pending until the
      chain has it (about a minute; the launcher id is saved, so a restart keeps waiting), then
      links it (`DidLink`); "Link a DID I have" finds one these words hold (`OwnerSigner::find`).
      A sponsor gives each identity one player DID and up to 8 world DIDs a day. Identities and
      endpoints cost nothing, so each request carries a proof of work (22 zero bits by default,
      a second or two of CPU) and the day's budget opens an hour's share at a time (research
      round 4, P7).
   6. *The join proves the DID* (built, seen working on testnet11): beside the identity's vouching for the login key, the join
      carries the DID's launcher id, its owner key and that key's signature of `chiacraft did v1
      <launcher> <identity>`. It needn't name the session: the identity is already bound to the
      login key the connection proved, so a replay only proves the same link again. The world
      checks the signature, then on chain that the DID's current owner is that key's address
      (cached per DID and followed, so a transfer is seen; without the chain: the cache, else
      "unverified", which DID rules treat as no DID). The check runs on a thread of its own; a
      join waits for it (25 s at most) only when the world's rules ask for a DID. A DID's age is
      the chain's height since its launch (4,608 blocks a day). The DID rules of slice 1 start
      counting, and a ban by DID refuses whoever proves it.
   7. *World DIDs, ownership both ways* (built, seen working on testnet11): a world can get a DID from a sponsor, launched to the
      owner's address, so the DID stays in the owner's wallet and the world holds only its key.
      Its metadata names the owner's DID and the world's current key and generation, set at
      launch (the player spends nothing). The owner's DID lists its worlds in its metadata (one
      spend of the owner's DID per world added; fee 0 on mainnet while the mempool isn't full). A
      world counts as the DID's only when both say so. The status answer and the address
      (`d=`) carry the world DID; saved worlds and links show it; giving a world a new address
      updates its DID (one spend). The account's certificate of the world key stays as it is.
   Done when: a player has a DID made through our sponsor, their world and their DID point at
   each other, and a world owner sets admission rules and bans players, seen between two games
   or a game and a headless server, on testnet11 (done 2026-11-04). Then achievements, decided
   2026-11-04 overnight by Phase 4 under Ronald's "decide alone" (`notes/crew/OVERNIGHT.md`), on
   the wallet's credential (`achievement.rs`: a record naming player, issuer and achievement,
   signed by the issuer's identity key, an optional issuer DID with its owner key's proof,
   revocation lists, a daily Merkle anchor):
   8. *Worlds sign achievements* (built 2026-11-04, not yet seen in a game): when a world grants an advancement (the base mod's, or any mod
      through the bridge's `achievement` question), the world signs a credential for the player:
      the issuer is the owner's identity key (the world runs in the owner's game or a headless
      server with the owner's account), named after the world, with the world's DID and its
      owner key's proof when the world has one, and the world's address as where to ask about it.
      Players without an account identity get none. The player's game checks it and keeps it in
      the account home (`achievements.json`), so it follows the words to any computer. Free and
      off chain; on in every world (an owner can turn it off).
   9. *The profile view* (built): the Profile screen gains the player's achievements, by world: title,
      when, and how sure it is (signature; the issuing world's DID confirmed both ways on chain;
      anchored). Self-signed ones (earned in your own world) say so.
   10. *The daily anchor* (opt-in, the owner's; built, not yet run on chain): once a day the owner's game commits one Merkle
      root of the day's credentials on chain (the wallet's `anchor`, a spend from the owner's
      wallet; testnet11 for now) and gives each player the inclusion proof for theirs when they
      next join.
   Later: the admission rule "has achievement X" (the join carrying the credential), revocation
   lists served by worlds, achievements a sandboxed mod grants.
5. **Finding worlds.** On-chain listing by hint; DIG listings and curated lists; spam ranking.
6. **Sandboxed code mods.** wasmtime with components (from `horizon-mods`), WIT v1.0 shaped on
   Weft's SDK, server halves then client halves, frozen API versions; personal mods per world.
   Decided 2026-11-03 (`reports/Sandboxed code mods.md`, "Decisions": Ronald took its
   recommendations): wasmtime 36 LTS components with the host in our crates and WIT v1.0 frozen
   once released; code at `code/server.wasm` inside the package; the limits and strikes below;
   server halves on the host only; Weft mods port through the `weft_sdk` alias; a package's
   command that clashes is left out and the world still opens; one consent sheet that marks code.
   In slices:
   1. *Server halves* (built): `crates/sandbox` (wasmtime 36 LTS components, fuel per call, a
      500 ms epoch watchdog, 64 MiB memory, per-call counts, storage quotas, three strikes stop a
      mod in a world), API `chiacraft:mods@1.0.0` (`crates/sandbox/wit/v1.0`, frozen once
      released), the guest SDK `crates/mod-sdk` with Weft's SDK names (commands, events,
      questions, chains, world ticks, `World`, handles; storage instead of components). A
      package carries it as `code/server.wasm`, under its hash and signature; the `packages` mod
      registers its hooks through Weft's SDK and runs it per world run on the host only, with
      storage in the world's save. Sample: `clans` (the animals mod's tribes, ported). Author
      guide `docs/sandboxed-mods.md`.
   2. *Consent says it is code* (built): protocol 6 offers say whether a package carries code,
      the join sheet marks it ("runs sandboxed code"), and a package whose manifest says otherwise
      is refused; a package's clashing command is left out instead of failing the world.
   3. *More of the SDK* (1.1 built): API 1.1 adds per-entity components, `set_blocks`, sounds
      and particles, and imports are checked at load; the host binds the newest 1.x and runs
      every older one (1.x only adds; wasmtime matches by semver). Sample `tally`. Decided
      2026-11-04 (Planner for Ronald): the API version comes from a mod's imports and travels in
      the offer (protocol 8), so the join sheet refuses a too-new mod before downloading (built);
      a `required` flag per package at the next `package.toml` change, optional by default (a
      stopped required mod closes its world to joins and sends its players home); a call-cost
      benchmark (`callcost`) before a release freezes 1.0. API 1.2 (built): callbacks on the
      package's own blocks and items, `schedule`, channels; a second export (`content-hooks`)
      found at run time so older components still run. Sample `bell`. The mod UI ABI's `at`
      came with it (Weft mods rebuild once).
   4. *Client halves* (built): `code/client.wasm`, a component for the world `client-mod`: HUD
      elements, screens, keys, packets from its server half and frame hooks, its UI built as parts
      the game replays with its own kit (`crates/mod-sdk-client`). Sample: the bell's.
   5. *Personal mods per world* (built; Ronald: "Yes, as planned"): the player's own client
      halves in `<run dir>/personal/<name>`, run only where the world rule Personal Mods allows
      (off by default, on Create World); joining says which are off. They run through shared
      slots (a HUD element per anchor, a screen, eight keys) so the content stays the same.
   6. *Operations* (`/mods` built): calls, time, strikes and why one stopped, per mod in this
      world; `/mods resume <package>` for admins. Later: cached compiled components, a per-tick
      budget across mods.
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
- Settings in the game's own store that servers read without being synced (text settings,
  client-scope ones) are still shared by a world kept running and the game; none matters yet.
