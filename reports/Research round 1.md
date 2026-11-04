# Research round 1: what to simplify, what to borrow, what to add

Researcher, 2026-11-03. Read against the trunk `chihuahua-papillon-model` at `090ac91`, plus the
uncommitted `wild.rs` and `growth.rs` in the main checkout. Other branches checked so this doesn't
repeat them: `claude/ci-workflow` (CI with lavapipe and weekly fuzzing, not merged) and
`claude/phase4-dids-bans-5161f5` (the proposed Phase 4 slices in `notes/PLAN.md`).

Every recommendation has an id, a tag (*simplify*, *improve* or *feature*), an effort (S = an
afternoon, M = a few days, L = a week or more), and the files it touches. Line numbers are at
`090ac91`. Two code reviews read the code line by line. Four of their findings I checked myself
(R1, R2, P1, P3), and each held.

## The top ten

| # | id | tag | effort | what | why now |
|---|---|---|---|---|---|
| 1 | R1 | improve | S | Stop the stores from wiping themselves: `links.json`, `friends.json`, `pools.json`, `worlds.txt` | A damaged or half-written file is read as empty, then the next save overwrites it. Links, revocation tombstones, friends and every gene pool would be lost without a word. |
| 2 | T1 | improve | S | Merge `claude/ci-workflow`, with timeouts and the trunk as a push branch | Ronald's rule is that tests run on CI only, so until it is merged nothing runs them. Its triggers name `main`, not the trunk, and the testkit job has no time limit. |
| 3 | P1 | improve | S | Cache what a block is (heat, food source, water) in a table per block state | `heat_of` matches block names as strings inside 7×7×7 and 33×7×33 scans, once a second per animal. In winter that is the biggest cost found. |
| 4 | P2 | improve | S | Wild plants that are fully grown stop running their growth step | Every grown tuft and ripe bush in a grassland keeps reading 162 blocks per step, forever. |
| 5 | P3 | improve | S | Keep each animal's size factor in a component instead of decoding genes every tick | `coats::size` runs string lookups, a hash and `ln`/`sqrt`/`cos` per animal per tick, and again for every animal a hunter or prey looks at. |
| 6 | P4 | improve | M | Make a new gene pool cheaply: cache the 300-generation start pool per species and habitat | Exploring a new 256-block cell runs 300 generations of 200 full phenotypes on the game thread: a hitch per species. |
| 7 | N1 | simplify | S | One request-and-answer helper for the six one-shot iroh exchanges, and one reused endpoint for status pings | Saves about 70 lines. Pings stop binding a whole new endpoint (relay handshake, DNS) every 10 s for every saved world. |
| 8 | G1 | simplify | M | One creature row and one set of rest poses for the seven full creatures | `cat`, `capybara`, `wolf`, `jaguar`, `utahraptor`, `raptor` and `wyvern` repeat settle, wake and sleep. About 450 lines go, and a new full creature becomes a row. |
| 9 | F1 | feature | M | Clothing as four numbers per item (warmth, windproof, waterproof, warm when wet) with wetness | Clothing is "core next", and the air model already gives wind and rain. DayZ, The Long Dark and Vintage Story have tested the numbers. |
| 10 | E1 | improve | M | Entities sent and ticked by range: a send range shorter than the view, and an activation range | Every tick, sending scans every entity for each player, out to the full view distance. Paper and Luanti cap both; we cap neither. |

The rest of this report gives the detail, the sources and the smaller findings.

---

## 1. Simplify what we have

### Networking and linked worlds (`crates/engine/src/net`, `crates/engine/src/chiacraft`)

**R1. improve, S. The stores wipe themselves when a file is bad.**
- `links::init` (`chiacraft/worlds/links.rs:204-211`) and `friends::init` (`friends.rs:52-59`) read with `.ok().and_then(..).unwrap_or_default()`. A file that fails to parse becomes an empty store, and the next `save` writes that empty store over it with a plain `std::fs::write` (`links.rs:221-231`, `friends.rs:66-72`).
- A power cut during a write is enough to cause that. Links, the permanent revocation tombstones, allow and block lists and used invites would all be gone.
- `mods/animals/src/evolution.rs:83-95` does the same with `pools.json`. It logs the error, but then marks the pools dirty and writes over the file within 30 s, so the world's whole evolved history is lost.
- Plain writes that aren't atomic are also in `saved_worlds::save` (`saved_worlds.rs:69`), `Trust::save` (`package_trust.rs:110`) and `travel::set_spot` (`travel.rs:212`).
- `saved_worlds` also loses updates: `remember`'s background thread and `status_line` on the UI thread both load, change and save `worlds.txt` with no lock (`saved_worlds.rs:101-115`, `188-234`). It also re-reads the file on every screen build.
- **Change:** one small `JsonStore<T>` holding a path and a `Mutex<T>`, with these rules:
  - `load` renames a file it can't parse to `<name>.bad-<time>`, logs it, and only then starts empty, so nothing is overwritten.
  - `update(|t| ..)` saves through the atomic write that `packages::store::write_atomic` (`crates/packages/src/store.rs:297`) already has. Make it public.

  Links, friends, saved worlds and gene pools use it. Trust and portals only need the atomic write.
- **Saves:** about 40 lines.
- **Risk:** `worlds.txt` changing to JSON needs a one-time import; otherwise low.

**N1. simplify, S. One helper for the one-shot iroh exchanges.**
- Six places do "connect, open a stream, write a frame, read one back with a cap, close". Three ask: `status_on` (`p2p.rs:607-645`), `ask_worlds` (`:416-437`) and `deliver` (`:473-488`). Three answer: `serve_links` (`:364-382`), `serve_worlds` (`:386-400`) and `serve_status` (`:583-598`).
- **Change:** `call(ep, addr, alpn, request, max_answer, wait)` and `answer_once(conn, max_request, f)`. The quirk of writing an empty frame so the stream is seen then lives in one place.
- Put the frame caps and timeouts in one table per ALPN. Today they are inline numbers: 16 KiB, 64, 256 KiB, 1024, 8 KiB, … and 10, 10, 5, 8, 4, 5, 20, 30, 3 seconds.
- **Status pings:** `status_of` (`p2p.rs:600-605`) binds and closes a whole iroh endpoint per ping, and `saved_worlds::ping_all` (`saved_worlds.rs:161-175`) pings every saved world every 10 s while the list is shown. Keep one anonymous endpoint per run instead, in a `OnceCell` like `ACCOUNT`.
- **Saves:** about 80 lines. Nothing changes on the wire.

**N2. simplify, S–M. One signing helper per key kind.**
- The same "label plus canonical text, sign, hex; parse key, unhex, verify" code is written in `links.rs` (`Body::signed`, `Revoke::signed`, invites, `verify`), `travel.rs` (`issue`, `admit`) and `package_trust.rs`.
- Hex encoding is written five times: `links.rs:238`, `travel.rs:82` and `:159`, `package_trust.rs:20` and `packages/src/identity.rs:74`. The wallet has a sixth (`wallet/keys.rs:166`).
- **Change:**
  - a `Signable` trait (`LABEL`, `body()`, `sign`, `verify`) for the Ed25519 world and content keys;
  - `sign_line`/`check_line` for the four BLS `check_*` copies in `crates/account/src/lib.rs:194-221`;
  - the `hex` crate, already in the lock, everywhere.
- A single `Signed<T>` envelope for both key kinds isn't worth it: the key types and the wire texts differ and are already fixed.
- **Saves:** about 100 lines. Labels and texts stay byte for byte the same.

**N3. simplify, S. Small repeats that drift.**
- "Parse a certificate, check it names this world, verify it" is written four times (`p2p.rs:427`, `:622`, `links.rs:481`, `:546`). `links.rs:551-553` takes the owner without verifying it. Add `WorldCert::owner_of(text, world)`.
- "My account endpoint id" is worked out three ways (`net/mod.rs:36`, `links::reply_address`, `bridge.rs:83`).
- `check_offers` (`p2p.rs:1075-1082`) re-implements the package id and version rules and has drifted from them: offers accept ids that start with a digit or `_`, which `read_manifest` refuses. Use `packages::valid_id` and make it public.
- Four places make randomness from `RandomState` (`p2p.rs:204`, `links.rs:269`, `travel.rs:88`, `world_address.rs:89`). Use `SecretKey::generate()` and `getrandom`.
- Name the positional wire tuples, `Offer = (String, String, String, u64, String)` read as `o.2`/`o.3`/`o.4` and the 8-tuple status, with the existing `wire_struct!`. It writes fields in order, so the bytes stay the same.
- **Saves:** about 60 lines, mostly in readability.

**N4. improve, S. The outbox.**
- Delivery is one message at a time, up to 28 s each (`p2p.rs:451-468`), so one dead inbox holds up every other message. Use a `JoinSet` limited to four.
- `tried()` finds its entry by JSON equality (`links.rs:664-680`). Give each queued item an id.
- Keep the hand-written backoff: it must persist across restarts, which `backoff` and `tokio-retry` can't do, and `backoff` is unmaintained.

**N5. improve, S–M. Use iroh's `Router`.**
- iroh 1.4.1 has `protocol::Router` and `ProtocolHandler`. They would replace the two hand-written accept loops (`p2p.rs:282-295`, `550-563`) and the close logic in `HOST_ENDPOINT`. `on_accepting` could also refuse a join early when eight are already configuring.
- About even in lines, but it is iroh's own pattern ([iroh 1.0 echo example](https://docs.rs/crate/iroh/1.0.0/source/examples/echo.rs)).
- The world endpoint stays separate from the player's, because its key is the world's address.

**N6. improve, later (L). iroh-blobs for packages: not yet.**
- Hashes are compatible: an iroh-blobs hash is the content's BLAKE3, the same as `Hash::of`. It would replace `serve_packages`, `send_file`, `fetch` and most of `download` (`p2p.rs:760-1002`, about 230 lines) with verified, resumable range downloads.
- But iroh-blobs 0.104 (2026-10-28) says it is "not yet considered production quality" and points production users to 0.35 ([docs.rs](https://docs.rs/crate/iroh-blobs/latest)). It also went from 0.97 to 0.104 in a month. And our per-peer limits (only offered hashes, twice the offered bytes, four streams) would have to be rebuilt on its provider hooks.
- Revisit when packages come from more than the host (Phase 2 "later").
- **Cheap fix now:** `fetch` holds a whole file, up to 64 MiB, in memory (`p2p.rs:913`). Stream it to a temporary file through `blake3::Hasher` instead, about +10 lines.
- Keep the manifest as it is. A HashSeq-shaped manifest (one BLAKE3 per file) is what iroh-blobs would want, and ours already is one in all but encoding.

**Keep as they are:** the store's tidy and LRU (`store.rs:182-261`); the frame-size checks before any allocation (`p2p.rs:234-255`, `codec.rs:101`); the per-peer upload budget (`p2p.rs:774-790`); and the byte-for-byte canonical manifest (`identity.rs:217`). They are what a stranger can reach, and no crate does them better.

### Gameplay (`mods/base/vitals`, `mods/animals`, `growth.rs`, `wild.rs`)

The per-tick findings matter most, because they set how many animals a world can hold (see Risks). The numbers below assume 1000 animals loaded.

**P1. improve, S. A table per block state instead of string matching in scans.**
- `base_api::heat_of` (`mods/base/api/src/lib.rs:277-285`) does `block_name(state)` across the FFI, then `split('[')` and a string match.
- It runs inside `air_at`'s 7×7×7 loop, once a second per animal from `comfort::upkeep`: about 343k string lookups a second.
- In `comfort::seek`, a cold animal makes 12 tries of 343 cells, plus `nearest_fire` (`comfort.rs:214-230`) over 33×7×33 = 7,623 cells. That repeats every 100 ticks per cold animal: in winter about 117k lookups a tick.
- The same pattern is in `forage::food` (`mods/animals/src/forage.rs:106`, a linear `SOURCES` scan for each of 1,734 cells) and in `ground_at`'s water test (`lib.rs:264`, 162 name compares per growth step).
- **Change:** build, once per content (they change on a content swap), a `Vec<f32>` heat, a `Vec<Option<u8>>` food source and a water bit, indexed by block state. Or use the engine's tags, as `comfort::WATER` already does.
- Also search outward in rings and stop at the first ring with a hit, instead of visiting every cell. This applies to `nearest` in `forage.rs`, `nearest_water` (`drink.rs:76`) and `nearest_fire`.
- **Adds:** about 40 lines. Turns string work into array reads.

**P2. improve, S. Grown plants stop growing.**
- The engine's `Content::grows` (`crates/engine/src/content.rs:677-681`) treats any block with an `on_grow` as growing in every state.
- So every grown tuft, ripe bush and ripe seed head that `wild.rs` places keeps firing `growth::grow`. Each time it reads `ground_at` (162 blocks), and `air_at` for frost-tender plants, before finding nothing to do. Over a whole grassland that grows with the loaded area.
- **Change:** in `growth::grow`, return first when the age is at its last step and no frost can apply. The better fix is for the engine to stop timing a block's last state, but that is an engine change (rule 4).
- `wild.rs`'s spreading also scans 9×3×9 blocks per step; cap it or sample a few cells.
- **Owner:** the session doing wild plants.

**P3. improve, S. Size from genes once, not every tick.**
- `animal.rs:155` calls `coats::size(g) * breeding::growth(..)` every tick for every animal. `coats::size` (`core/src/coats.rs:1564`) runs a linear `gene_file` find, `Read::qv("Sz")` with another string-keyed find, a hash, and `ln`/`sqrt`/`cos`.
- `hunt::mass` (`hunt.rs:92`) repeats it for every animal a hunter or prey weighs up.
- **Change:** store the factor in a component when the genome or stage changes. `SCALED` nearly does this already.
- Also cache `Species::hungry_in_the_wild` per species (it walks 35 species × habitats × habitats as string compares, twice a tick per animal) and test "already full" first.

**P4. improve, M. New gene pools without a hitch.**
- The first animal in a new 256-block cell runs `evolution::evolved` (`src/evolution.rs:133`) over `HISTORY` = 300 generations.
- Each generation (`core/src/evolution.rs:118`) samples 200 genomes and builds a full phenotype for each, with strings and vectors, only to read the average colour.
- Parent picking filters and sums all 200 on every call, and `step` formats 54 string map keys and clones the neighbours' pools.
- **Change:**
  - Compute the 300-generation start pool once per species and habitat (a fresh cell has almost no neighbours, so it barely depends on the cell), cache it and copy it in.
  - Pick parents with prefix sums (`rand_distr::WeightedIndex`).
  - Let fitness read only the average colour.
  - Use `(species, x, z, habitat)` tuples as keys.
- **Saves:** about 30 lines; removes tens of millions of operations from the game thread as players explore.

**G1. simplify, M. One creature row, one rest module.**
- `settle`/`wake`/`sleep` with a rest component and `LIE_TICKS` are repeated in `cat.rs:84`, `capybara.rs:83`, `wolf.rs:87`, `jaguar.rs:86`, `utahraptor.rs:90`, `raptor.rs:126` and `wyvern.rs:144`. `wild.rs:103` already has the general version. `wolf.rs` against `jaguar.rs` differs only in names, sizes and taming against knock-out.
- **Change:**
  - a `rest.rs`;
  - a `Creature` row beside `wild::Body` and `birds::Wings`;
  - one `goals(index, extras)` that picks taming or knock-out from `Species::taming`, hunting from `hunts()`, a nest from egg-laying and riding from `seat > 0`.
  - What only one species does (gifts, calm aura, breath, guarding) goes in an extras slot.
- **Saves:** about 450 lines across seven files.
- Don't bring in `big-brain` (built for Bevy's ECS) or `bonsai-bt`: the priority list in `Goals` already is a behaviour-tree selector.

**G2. simplify, S. Species indices written by hand in four places.**
- The order of `core::SPECIES` (`species.rs:311`) is repeated in:
  - the mod's `SPECIES` (`src/species.rs:31`);
  - `coats::FILES` (35 `include_str!` lines at `coats.rs:261`);
  - `const INDEX: usize = 33` in `wolf.rs` and `jaguar.rs`;
  - bare numbers like `birds::register(.., 27)` and `taming::behave(1)`.
- Adding a species in the middle shifts them all without an error.
- **Change:** a `const fn index(name)`, entity kinds built from the names, and `FILES` from a small `macro_rules!`.
- **Saves:** about 80 lines and a quiet class of bug.

**G3. simplify, S. `species.rs` stays Rust, with defaults.**
- Its 555 comment lines are the research, and the types (`Category`, `&FAMILY`, `Tune`) are checked when it compiles. TOML or RON would lose both and save nothing.
- **Change:** a `const BASE: Species` with `..BASE` in each row. `wild_step` and `tamed_step` are the same in all 35 rows, `breed: None` in 34, `seat` 0 in 33, `attacks: &[]` in 22, `tune: Tune::NONE` in 16.
- **Saves:** about 250 lines.

**G4. simplify, M. Split `coats.rs` and make the simple looks data.**
- Split the 2.4k lines into four modules: genes, looks (`:765-1527`), describe and paint.
- About 15 looks have the same shape: mix two colours by one trait, name by a threshold, add fixed underparts, points and pattern. They are deer, trout, pacu, raven, vole, mouse, kryptobaatar, lizard, protoceratops, eagle, vulture, gull, caiman, owl and small bird.
- Give each of those a `"look"` block in its `genetics/<species>.json`, read by one small interpreter (−300, +80 lines). Dog, cat, fox, wolf, jaguar, raptor and wyvern stay code. The sheet and website checks catch any change in output.
- The genome itself (a `#[repr(C)]` byte table of allele indices) is already the standard form; a bitfield would gain nothing.

**G5. simplify, S. One source for each constant.**
- `GROWING` is defined twice (`animals/core/src/biology.rs:154`, `vitals/src/plants.rs:98`), and a test (`core/tests/places.rs:121`) holds them equal. `animals_core` already depends on `vitals`, so define one from the other and delete the test. Better, move all four paces (carrying, growing, living, resting) into a `vitals::pacing` module.
- A day of 24000 ticks is written four times: `calendar::DAY_TICKS`, `life::DAY`, `evolution::GENERATION`, and a bare literal in `base_api` (`lib.rs:193`).
- `needs::DAY_SECS = 600` (the drinking clock) shares a name with `vitals::DAY_SECS = 1200` (a game day). Rename the first.
- `dist2` is written seven times.

**G6. simplify, S. One way to write each sheet.**
- Both sheets are written by Rust (`vitals/src/sheet.rs`, `animals/core/src/sheet.rs`), but the animals' one goes through WASM and node (`web/build.sh`, `sheet.mjs`).
- **Change:** a three-line `animals_core` bin like `vitals`'s, or both staleness tests write the file when `UPDATE_SHEETS=1` (the `expect-test` pattern). That drops a step from `CLAUDE.md` and `SYSTEMS.md` and the node dependency for a number change. The species website keeps its WASM build.

**G7. simplify, M. One errand for forage, drink and seek.**
- `FORAGE`, `DRINK` and `SEEK` are each a `[f64; 4]` (target, time, NaN for "nothing") with three copies of the walk-act-wait logic.
- One `Errand { kind, target, until }` finishes what `notes/ECOLOGY.md` set out to do, "one forage goal", and lets needs be ranked by urgency: a starving animal eats before a slightly thirsty one drinks. Today drinking always comes first.
- **Saves:** about 100 lines.

**G8. improve, L (engine). Catch growth up in one step.**
- The engine catches up timed blocks when an area loads (`dim.rs:836`), but runs each missed `on_grow` with today's air.
- So a field left alone all winter catches up at summer rates, and one loaded on a frosty night can be killed once for every missed step.
- `vitals::plants` is pure, so the owed growth is the sum of `rate(day)` over the missed days. An `on_catch_up(elapsed)` hook would apply it in one call. This is an engine change, so it is Ronald's call (rule 4).

**Keep as they are:** the numbers in pure crates with sheets and staleness tests; evolution worked out as numbers when needed, with capacity standing in for faraway animals (the right shape, see section 2); the table-driven kinds (`wild::Body`, `birds::Wings`, `vitals::wild::WILD`); the staggered goal timers.

---

## 2. How others solved the same problems

### P2P worlds over iroh

- **iroh 1.0** shipped on 2026-06-15 with a stable wire protocol and API, QUIC multipath and NAT traversal; we are on 1.4.1 ([iroh blog](https://iroh.computer/blog/v1)). No other voxel game or linked-worlds project on iroh turned up. The nearest are Fish Folk: Jumpy and a Godot plugin ([Godot asset](https://godotengine.org/asset-library/asset/3948)). We are early.
- **Direct connections aren't a given.**
  - n0 says about 95% of data flows directly ([iroh blog](https://iroh.computer/blog/v1)), and its docs say hole punching works about 90% of the time, unmeasured ([docs PR](https://github.com/n0-computer/docs.iroh.computer/pull/4/files)).
  - The best independent number is libp2p's DCUtR, about 70% ±7% over 4.4M attempts ([arXiv 2510.27500](https://arxiv.org/abs/2510.27500)).
  - Phones on mobile networks are often relayed ([iroh #2317](https://github.com/n0-computer/iroh/issues/2317)).
- **n0's public relays** are rate-limited with unpublished limits, and n0 says not to use them in production ([iroh docs](https://docs.iroh.computer/relays/rate-limiting)). A relay is free to run yourself ([managing relays](https://docs.iroh.computer/deployment/managing-relays)).

**I1. improve, S.** Show "direct" or "relayed" in the connection info and the F3 overlay, and plan the play traffic so a relayed world (10–30% of them) still plays. Files: `net/p2p.rs`, the debug overlay.

**I2. improve, M. Bring our own relay forward from Phase 7.** A `chiacraft:relays` setting (rule 1: anyone can run one), our relay as the default, and n0's as the fallback. Every linked-world feature leans on connections working, and a relay we don't control is the one hard dependency left. Files: `p2p::endpoint`, settings, a small `crates/relay` wrapping `iroh-relay`.

### Identity, moderation and bans

What others found:
- **Veloren** has a central auth server that most servers require ([Veloren book](https://book.veloren.net/players/hosting-a-server.html)). **Luanti** keeps auth and bans per world, with no shared identity ([world format](https://gogs.librecmc.org/OWEALs/minetest/src/e5e94070d66ad83034328c23e805a46ffd71b73c/doc/world_format.txt)). We sit between them: identity is shared, and bans are per world.
- **Shared lists work** when the criteria are narrow and about behaviour (spam, scams), and when each subscriber chooses to apply them. Matrix's Draupnir subscribes rooms to ban-list rooms ([Draupnir](https://github.com/the-draupnir-project/Draupnir)). Bluesky lets users stack up to 20 labelers ([Bluesky docs](https://docs.bsky.app/docs/advanced-guides/moderation)).
- **They fail** when they slide from behaviour into opinion. One Matrix list grew to "trolls, rude people" ([community help](https://matrix-community-help.codestorm.net/automated-tools.html)). A study counted 39,589 Bluesky lists, many of them political ([Lancaster study](https://eprints.lancs.ac.uk/235600/1/bluesky3_0-7.pdf)). Oliphant stopped publishing its widest list as too open to misuse ([Oliphant](https://elektrine.com/remote/oli@olifant.social)).
- **Lists also fail when identity can change:** trojan accounts get listed as harmless, then rename themselves to impersonate someone ([moderation lists](https://sites.google.com/view/moderation-lists)).
- **What resists Sybils:** identity cost (Urbit's capped, priced address space, [Delphi](https://members.delphidigital.io/reports/an-urbit-in-our-stars)) and starting new identities with no weight (Freenet's trust 0, [Freenet WoT](https://github.com/freenet/wiki/wiki/Web-of-Trust)). I found no deployed system that relies on key age alone.

The proposed Phase 4 slices already have bans by key, identity and DID, probation, and signed shared lists served over iroh. What to add to them:

**M1. improve, S.** Match bans only on keys and DIDs, never on names. Names are labels shown beside the key. This closes the rename trick. Files: the Phase 4 `bans.json` slice.

**M2. improve, S.**
- Give shared ban-list entries a category (spam, griefing, cheating, illegal content), and let a subscription choose **apply** or **flag only** per category, as Draupnir and Bluesky labelers do.
- Add "ban only when N of my M lists agree" (Garden Fence's majority rule, [Garden Fence](https://awesome.ecosyste.ms/projects/github.com%2Fgardenfence%2Fblocklist)).
- Each entry gets an optional end date and a note on how to appeal.
- This keeps the lists to behaviour, where they worked. Files: Phase 4 slice 3.

**M3. improve, S.** A player on probation in a world carries no weight in that world's reports or in vouching for others, the way Freenet's new identities start at trust 0. Probation then also dampens report brigading, not just griefing.

### Mod sandboxing with wasmtime components

What others found:
- **Veloren** already runs wasmtime components, with separate `plugin`, `server-plugin` and `animation-plugin` worlds in one `veloren.wit` ([Veloren source](https://docs.veloren.net/src/veloren_common_state/plugin/module.rs.html)). That is the shape our server and client halves want.
- **Zed** freezes its API as one WIT folder per version (`since_v0.0.1`, `since_v0.1.0`, …), keeps one `bindgen!` module per version, and holds a host enum over them. It advertises a min–max range, and unreleased versions load only in dev builds ([Zed source](https://git.secluded.site/zed/blob/v0.149.5/crates/extension/src/wasm_host/wit.rs?raw=1)).
- **CPU limits:** epoch interruption costs about 10%; fuel costs more but is deterministic ([wasmtime docs](https://docs.wasmtime.dev/examples-interrupting-wasm.html)).
- **Memory limits:** `StoreLimits` caps linear memory, which is unlimited by default ([wasmtime limits](https://paritytech.github.io/try-runtime-cli/src/wasmtime/limits.rs.html)).
- **WASI 0.3** was released 2026-06-11, with async built into the component model; Wasmtime 46 turns it on by default and runs 0.2 components alongside ([WASI P3](https://wasi.dev/releases/wasi-p3)).
- **Envoy's proxy-wasm** has a failure policy per plugin: fail closed, fail open or reload ([Envoy docs](https://www.envoyproxy.io/docs/envoy/latest/api-v3/extensions/wasm/v3/wasm.proto)).
- **Hytale** chose server-side plugins and no client mods, so players never need a modded client ([Hytale news](https://hytale.com/news/2025/11/hytale-modding-strategy-and-status)).

For the Phase 6 session:

**W1. improve, S.** Freeze the API the Zed way: `wit/since_vX.Y/` folders, one generated module per version, a host enum, and the version range in the package manifest. A mod outside the range is refused at join, on the join sheet, never in the middle of a tick.

**W2. improve, S.**
- **CPU:** epoch interruption on server halves, with the tick thread bumping the epoch and each event call getting one to two epochs.
- **Memory:** a `StoreLimits` cap per mod (64–256 MB).
- **Failure:** each mod declares itself `required` or `optional`. A trapping required mod stops the world, or sends the player home, with the mod named. An optional one is reloaded once, then turned off.

**W3. improve, M.** Coarse host calls: batched events in, lists of block edits out, rather than one call per block. Measure with wasmtime's `benches/call.rs` before freezing v1; there is no published number for the component model's call cost.

**W4. simplify, S.**
- Data packages already carry the look, so Phase 6 can ship server halves first and treat client halves as rare and opt-in, as Hytale does.
- Target WASI 0.3 from the start: world queries are naturally async, and 0.3 is now released.

### Content-addressed mod and package distribution

We already do what the best of them do: content-addressed files, a lockfile per world, one shared cache, and a check of everything against its hash.
- Luanti compares the server's hashes with its cache and fetches only what is missing, from a static HTTP mirror first since 5.12 ([Luanti docs](https://docs.luanti.org/for-server-hosts/remote-media/)).
- Factorio syncs exact mod versions on join ([Factorio wiki](https://wiki.factorio.com/Mods)).
- What others got wrong:
  - **Garry's Mod** caps downloads from the game server at 64 KB/s, which is why FastDL mirrors exist ([Facepunch wiki](https://wiki.facepunch.com/gmod/Serving_Content)). Our host serves at full speed but alone.
  - **IPFS** discovery was slow and unreliable behind NAT ([kubo #6382](https://github.com/ipfs/kubo/issues/6382)). Don't put package lookup on a DHT.
  - **Chunked deltas** (casync) are only worth it for large single files that change ([LWN](https://lwn.net/Articles/726625/)). Ours are many small files, so hashing per file already gives cheap updates.

**D1. improve, M. Sources other than the host, in order.** First the host. Then any linked world's endpoint that has the same hash; the link already says who you trust. Then an optional static HTTP mirror laid out by hash, as Luanti does. Everything is checked by hash, wherever it came from. This needs no iroh-blobs: it is our own `fetch` with a list of sources. Files: `p2p::download`, a mirror setting.

**D2. feature, S.** Ship the base game's own content inside the game, pinned by hash, so joining a world that uses nothing else downloads nothing.

### Survival systems

The numbers worth borrowing for **clothing**:
- **DayZ** gives each body slot a share of total insulation: top 23%, legs 23%, feet 12%, head 10%, face 10%, vest 8%, gloves 8%, back 6% ([DayZ wiki](https://dayz.wiki.gg/wiki/Clothing)).
- **DayZ wetness** steps:

  | Wetness | Insulation kept | Weight |
  |---|---|---|
  | damp | 90% | 1× |
  | wet | 50% | 1.33× |
  | soaked | 10% | 1.66× |
  | drenched | −15% | 2× |

  A raincoat never gets past damp.
- **The Long Dark:** air temperature minus clothing warmth gives what you feel. Windproofing cuts wind chill. Waterproofing sets how fast an item wets. Wet items freeze, and wool keeps some warmth wet while cotton doesn't ([Hinterland](https://www.thelongdark.com/?p=1191)).
- **Vintage Story:** clothing warmth moves the air you can bear 1:1. It does nothing indoors. Warmth only falls off below 50% condition, so repairs aren't a chore ([VS wiki](https://wiki.vintagestory.at/Clothing), [temperature](https://wiki.vintagestory.at/index.php/Temperature)).

For **preservation and cooking**:
- **TerraFirmaCraft** keeps it as decay multipliers that stack and read easily in a tooltip: salted 0.5, pickled 0.5, sealed 0.5, vinegar 0.1, wood-grilled 0.8, burnt 2.5 ([TFC ServerConfig.java](https://raw.githubusercontent.com/TerraFirmaCraft/TerraFirmaCraft/1.20.x/src/main/java/net/dries007/tfc/config/ServerConfig.java)).
- **Vintage Story** scales spoilage 0.1–2.5× by climate. A cellar holds 5 °C, a sealed crock decays 10× slower, and each preserving step trades a little nutrition for time ([VS food preservation](https://wiki.vintagestory.at/Food_preservation/en)).
- **Vintage Story meals** take up to four ingredients in a pot and six in a pie, and each ingredient delays hunger ([VS satiety](https://wiki.vintagestory.at/Satiety)).

What players hate:
- **Food that changes how you fight.** Valheim's Hearth & Home food rework was re-tuned within a day after pushback ([PC Gamer](https://pcgamer.com/valheims-food-rebalance-has-already-been-rebalanced)).
- **Events you can't escape.** Vintage Story's temporal storms are its most-hated system ([VS forum](https://www.vintagestory.at/forums/topic/20006-temporal-storms-are-a-bad-implementation-of-a-good-idea/?page=7)).
- **Constant threat.** The Long Dark's wolves are its most common complaint ([Steam](https://steamcommunity.com/app/305620/discussions/0/1732088458073734671)).

These go to the Survival session (`claude/survival-clothing-cooking-f0c469`):

**F1. feature, M. Clothing.**
- Each item has warmth, windproof, waterproof, warm-when-wet, and the most wetness it can reach.
- Body-slot shares as DayZ has them, and wetness steps from its table.
- Wet items freeze below 0 °C.
- Warmth falls off only below 50% condition.
- Our air model already gives wind and rain at the player (`base_api::air_at`, `raining_at`). Real textiles behave this way (wool against cotton), so it fits the real-facts rule.
- Files: `vitals` (a `clothing` table), `survival/temperature.rs`, `items.rs`.

**F2. feature, M. Preservation as decay multipliers** in `vitals::food`.
- Salting, pickling and sealing at about 0.5, smoking at about 0.5 (my estimate: TFC has no smoked trait), vinegar 0.1.
- A cellar by its real temperature: the freshness model's temperature factor, applied to the air inside a closed room.
- Each step costs a little nutrition, as in Vintage Story.

**F3. feature, M. Meals from up to four ingredients.**
- A meal takes the freshness of its oldest ingredient.
- Covering more food kinds fills longer. That lines up with our stomach model, which already has nutrients.

**F4. improve, S. A rule for every food effect: good food gives a bonus, poor food gives no penalty to combat.** Write it into `notes/SURVIVAL.md`.

### Animal genetics and ecology sims

- **ARK**
  - Mutation: three rolls at 2.5% each per baby, about 7.3% for at least one; each adds 2 levels to a stat and changes one colour region. A parent's mutation counter caps at 20, which players get around by stacking lines ([ARK wiki](https://ark.wiki.gg/wiki/Mutations)).
  - Imprinting needs a cuddle every 8 hours, which players call an alarm clock ([Steam](https://steamcommunity.com/app/346110/discussions/0/1694919808736240246)).
  - Lag: Ascended added a server-wide soft tame cap of 5000, where tames over the cap are flagged and later deleted ([patch 33.15](https://ark.wiki.gg/wiki/ARK:_Survival_Ascended/Patch/33.15)). Wild animals piling up needs wipes ([Supercraft](https://supercraft.host/wiki/ark/asa_wild_dino_overspawn_fix/)).
- **Minecraft horses:** since 1.19.4 a foal's stats spread around its parents' average, so good lines stop drifting back to average ([allthings.how](https://allthings.how/breeding-horses-in-minecraft-foals-stats-colors-and-mules/)).
- **S.T.A.L.K.E.R.'s A-Life:** creatures beyond about 150 m are "offline", moved on a graph with no animation ([Game Developer](https://www.gamedeveloper.com/game-platforms/interview-inside-the-ai-of-i-s-t-a-l-k-e-r-i-)).
- **Rain World** keeps creatures in an abstract world of one node per room exit; they keep hunting and holding territory offscreen ([devlog](https://candlesign.github.io/Rain-World-Devlog/Pages/032)).
- **Dwarf Fortress** draws animals from world-level population pools ([DF wiki](https://dwarffortresswiki.org/Extinction)).
- **Species: ALRE** makes evolution visible: each child is a slightly mutated copy, and the player's catastrophes do the selecting ([Steam](https://store.steampowered.com/app/774541)).
- I found no ecosystem game called "Equilibrium"; the Steam game of that name is a planet manager. **Ecosystem** (Slug Disco) evolves body and brain from DNA ([Steam](https://store.steampowered.com/app/1133120)).

Our 256-block cell gene pools, with capacity standing in for faraway animals, are the same shape as Dwarf Fortress's pools and Rain World's abstract world. Keep it.

**A1. improve, S. Caps before players find them.**
- A soft tame cap per player and per world, shown on the animal screen, and a hard ceiling of wild animals per cell.
- Breeding cooldowns already follow seasons.
- Cap mutations per stat rather than with one counter, so there is nothing to game around.
- Files: `mods/animals/core` (the numbers), `taming.rs`, `breeding.rs`.

**A2. improve, S. Care in windows, not on a timer.** When imprinting is built (missing), a baby needs care some time in each in-game day, not at an exact moment, so nobody sets an alarm. Files: `breeding.rs`, `notes/ANIMALS.md`.

**A3. Keep: stats from one parent, the stronger 55% of the time.** That is `breeding.rs` today (`life::wild_points`) and ARK's rule, and it already holds a line's gains, which Minecraft only got in 1.19.4 by spreading foals around the parents' average. Nothing to change. What players liked in ARK is the rare, *visible* mutation (a colour region), which our 1% coat mutation already gives: show it on the animal screen ("a colour from neither parent") so it is seen.

**A4. improve, M. Animals near players only.**
- Past a range, a wild animal folds back into its cell's pool count and leaves the world; nearer, animals come out of the pool.
- Wild animals then cost nothing where nobody is, and the engine's region catch-up never has to simulate a herd.
- `capacity` and `evolution` already hold the numbers this needs. Pair it with E1.

### Voxel rendering: Weft already has the big tricks

I looked for the tricks behind Sodium, Luanti, Veloren and Vintage Story. Weft already has nearly all of them:
- **Cave culling:** Tommo's visibility flood fill, which culled 50–99% of geometry in Minecraft Pocket Edition ([Tommo](https://tomcc.github.io/2014/08/31/visibility-1.html)), is in `server/visibility.rs` and used by both server and client.
- **Draws:** face records live in a few big storage-buffer pages, with one indirect draw per cube and layer (`client/world_render.rs`), which is Sodium's region-buffer and multi-draw idea.
- **Distant terrain:** GPU-made LOD tiles (`client/lod.rs`).
- **Textures:** a bindless texture table (`client/textures.rs`) instead of an atlas, which is what Luanti's 5.15 move to array textures was after ([Luanti 5.15](https://blog.luanti.org/2026/01/24/5.15.0-released/)).
- **Also there:** translucent quads kept sorted within 48 blocks, LZ4 on saves and frames, and catch-up ticking instead of random ticks.

**V1. simplify, nothing to do.** Leave the renderer alone (rule 4). Rendering effort goes into the game's look (shader pack, art). Mesh shaders (Nvidium, Meshelium) are vendor-specific and risky under MoltenVK, so don't.

### Network scale

- **Paper:** 75 chunks/s per player, and an entity *activation* range (animals 32 blocks) separate from the *tracking* (send) range (mobs 96). Inactive animals wake 100 ticks out of every 1200 ([Paper docs](https://docs.papermc.io/paper/reference/spigot-configuration/)).
- **Luanti** runs objects within 4 mapblocks, sends within 8, gives block scripts a time budget, and skips sending blocks the client can't see, which it says sends 50–80% fewer blocks ([settingtypes.txt](https://raw.githubusercontent.com/luanti-org/luanti/master/builtin/settingtypes.txt)).
- **Veloren** ticks offscreen NPCs once every 30 ticks ([devlog 134](https://veloren.net/devblog-134)).
- **Ours:** `server/sync.rs` `send_entities` (`:231-262`) scans every entity of every region for each player each tick, sending within the full view radius × 16. `dim.rs` `tick_entities` runs every entity in a loaded region.

**E1. improve, M. An entity send range and an activation range.**
- Ours has neither. As settings: a send range (64–96 blocks) apart from the view distance; an activation range (32–48) past which an entity's mod goals run once every N ticks; a per-region entity list kept by cell, so sending doesn't scan everything.
- This touches the engine, but for scale, not features. The cap belongs where entities are kept.
- Files: `server/sync.rs`, `server/dim.rs`, settings.

### Test and CI

- **Bevy** runs its examples on `ubuntu-latest` with Mesa's software Vulkan under `xvfb-run`, takes screenshots at set frames, and sets a 30-minute timeout per job ([Bevy workflow](https://github.com/bevyengine/bevy/blob/main/.github/workflows/example-run.yml)).
- **wgpu** compares images with NVIDIA FLIP against mean or percentile limits, keeps a reference per backend, and always uploads the actual and difference images ([wgpu image.rs](https://raw.githubusercontent.com/gfx-rs/wgpu/trunk/tests/src/image.rs)).
- **nextest** kills a hung test (`slow-timeout = { period = "30s", terminate-after = 4 }`) ([nextest](https://nexte.st/docs/features/slow-tests/)).
- **iai-callgrind** counts instructions, so benchmark gates are stable on noisy runners ([iai-callgrind](https://iai-callgrind.github.io/iai-callgrind/latest/html/comparison/criterion.html)).
- **Factorio** tests build a small map, tick it, and check the result ([FFF 62](https://factorio.com/blog/post/fff-62)).
- The macOS GPU watchdog can't be tuned ([MoltenVK discussion](https://github.com/KhronosGroup/MoltenVK/discussions/2080)), which supports Ronald's rule: game runs belong on Linux with software Vulkan, where a hang only burns CPU.

`claude/ci-workflow` already does the Bevy pattern: testkit on lavapipe under xvfb, weekly fuzzing, mods' checks and the animals' WASM rules. What it lacks:

**T1. improve, S. Merge it, with three fixes.**
1. `on: push: branches: [main]` should name `chihuahua-papillon-model`, since `main` is stale and pushes to the trunk would otherwise never run.
2. The `testkit` job needs `timeout-minutes: 30`, and the run itself an outer `timeout 20m`. GitHub's default is six hours.
3. `cargo nextest` with a `slow-timeout` profile, so one hung test can't eat the job.

Files: `.github/workflows/ci.yml`, a `.config/nextest.toml`.

**T2. improve, M. Golden images on CI.**
- Compare five to ten testkit `shot`s, starting with the shader pack's scenes, against reference images with `nv-flip` and a percentile limit, as wgpu does.
- Keep references per backend (lavapipe), and upload actual and difference images.
- The Tester session is researching testing (`reports/Testing the game.md`), so this is theirs to take or leave.

**T3. improve, S.** iai-callgrind benches for the hot paths found here (P1's scans, P4's generation, meshing), gated on instruction counts. Then each fix in this report proves itself without a game run.

---

## 3. Features worth adding

The ones that can be had elsewhere, ranked by value against effort. Value comes from what players ask for: Enshrouded's feedback board, with top items over 5k votes ([ComicBook](https://comicbook.com/gaming/news/enshrouded-players-feature-feedback-board-pc)); the most-downloaded Vintage Story mods ([VS mod DB](https://mods.vintagestory.at/list/mod?sortby=downloads&sortdir=desc)) and Valheim mods ([Thunderstore](https://thunderstore.io/c/valheim/?ordering=most-downloaded)); and Unturned's 2024 survey ([devtrackers](https://devtrackers.gg/unturned/p/4bb1716b-unturned-2024-survey-results)). The last column marks what leans on our own pieces: **L** linked worlds and portals, **C** Chia identity, **E** real-world ecology.

| rank | id | tag | effort | feature | evidence | ours |
|---|---|---|---|---|---|---|
| 1 | F1–F3 | feature | M each | Clothing, preservation, meals (above) | VS cooking mods 0.8M downloads each, butchering 0.7M | E |
| 2 | F5 | feature | S | Storage comforts: craft from nearby chests, quick-stack, sort | Valheim's top inventory mods 1.1–1.6M each; #7 on Enshrouded | |
| 3 | F6 | feature | M | Skills that rise by use beside the level points: walking, swimming, cutting wood, taming, cooking | #3 on Enshrouded, #4 in Unturned (36%), VS XSkills 0.4M | |
| 4 | F7 | feature | S | A weather forecast: tomorrow from the rain schedule, which is already deterministic (`vitals::climate::spell`, seeded); a barometer item, a sky to read | Weather is #2 on Enshrouded | E |
| 5 | F8 | feature | M | Fishing and traps, with fish stocks per cell in the gene pools; smoking racks | Primitive Survival 1.0M | E |
| 6 | F9 | feature | M | Exploration: ruins and mini-bosses worth the walk | BetterRuins 1.2M, EpicLoot 2.4M, Monstrum 1M; #1 on Enshrouded | |
| 7 | F10 | feature | S | Planting in rows, auto-replant | PlantEverything 2.9M, PlantEasily 1.3M | |
| 8 | F11 | feature | M | A map you draw: discovered terrain and pins | ProspectTogether 0.4M | L (see L2) |
| 9 | F12 | feature | M | Imprinting and care (already planned, A2's windows) | ARK's most-discussed system | E |
| 10 | F13 | feature | S | Illness, mild and off by default: bad water, rotten meat | DayZ, The Long Dark | E |
| 11 | F14 | feature | L | Carts and pack animals: the Utahraptor carries, a cart pulls | OdinHorse 0.9M; vehicles #1 in Unturned (57%) | |
| 12 | F15 | feature | L | Raids and events, always avoidable or off by default | #6 on Enshrouded; VS storms the cautionary tale | |

Tombstones, rested and comfort, levels and riding already exist, so they aren't on the list.

### Features only we can do

These lean on the pieces nobody else has. They are what would make chiacraft a new game rather than a careful survival game.

| id | effort | feature | leans on |
|---|---|---|---|
| L1 | M | **Genes travel through portals.** A tame taken to a linked world and released there joins that world's pool (migration already exists between cells). A cold world's wolves carry warm-world coats a few generations after visitors bring dogs. Breeders trade lines between worlds. | L, E |
| L2 | M | **A network atlas.** The map shows your world and, beyond each portal, the worlds it links to: name, owner, biome, season, online, players, from their status answers. It is the "finding worlds" screen grown from the links you have. | L |
| L3 | S | **Letters through portals.** A signed note to a world's owner or a player, carried by the outbox that already retries for 14 days. Owners of linked worlds can talk without anyone being online at once. | L, C |
| L4 | M | **Real-place worlds.** A world created from a real place's climate ("Yakutsk", "Kalahari", "Yixian, 125 Ma"): the climate table holds them already. Linking a tropical world to a boreal one makes seasons and goods worth trading. | E, L |
| L5 | M | **Trade between worlds** via the planned inventory carry-over, with goods that only grow in one climate: cane, cacti, cold-hardy rye. Real ecology makes trade happen without invented scarcity. | L, E |
| L6 | M | **A guest book signed by DIDs**: each world keeps who visited, signed by the visitor, so a reputation forms from where people have been. It feeds the spam ranking for finding worlds (Phase 5) and admission ("has visited 3 worlds I link to"). | C, L |
| L7 | S | **Lineage you can prove.** A bred animal's pedigree (parents, breeder, world) signed by the breeder's key and kept with the animal, so a line from a famous breeder is provable across worlds. Achievements on chain could anchor rare ones later. | C, E |

L1, L3 and L6 are small next to what they give, because the outbox, the pools and the signing already exist.

---

## 4. Risks at scale

| id | risk | where | what happens | what to do |
|---|---|---|---|---|
| K1 | **Animals per world** | per-tick goals (P1, P3), `send_entities` (E1) | ~13 goals per animal per tick, string-matching scans and a full entity scan per player: hundreds of animals is fine, thousands will not be | P1, P3, E1, A4; T3 to measure |
| K2 | **The explored world grows `pools.json` forever** | `mods/animals/src/evolution.rs:99-111` | One JSON with every species × cell × habitat, rewritten whole every 30 s on the game thread. After a few thousand cells that is megabytes every 30 s, and a bad parse loses it all (R1). | Keep pools in the region saves beside their chunks, or one small file per cell; drop pools of cells nobody visited for a long time back to the start pool (P4) |
| K3 | **Status pings and warm-ups with many linked worlds** | `saved_worlds::ping_all`, `p2p::warm` | Each saved world binds a new endpoint every 10 s while the list is shown (N1); a hub world with 50 links pings 50 | N1's shared endpoint; ping only what is on screen; back off pings to offline worlds |
| K4 | **The outbox with many links** | `p2p.rs:451-468` | One message at a time, up to 28 s each; a few dead inboxes delay every request and revocation | N4 |
| K5 | **Revocation tombstones and invites never shrink** | `links.json` | Fine for years for one player, but loaded and rewritten whole on every change | Keep tombstones as a sorted list of 32-byte ids; consider SQLite (`rusqlite`) when links, friends, saved worlds and bans together pass a few thousand rows |
| K6 | **Relays** | n0's public relays | Rate-limited, not for production; 10–30% of connections may need one | I2 |
| K7 | **Download sizes** | packages | A 4096² texture is 64 MiB in video memory, and the store keeps 512 MiB of unpinned packages. A world with many packages makes travel slow, and portals feel broken when a trip waits on a download. | Show size and time on the portal before travel (the warm-up already knows); download in the background while near a portal (warm-up does status only today); a soft limit per world shown to its owner |
| K8 | **Content swaps cost memory** | `modloader::use_packages` keeps every built content set for the life of the game | A player hopping through 20 worlds keeps 20 content sets | Keep the last few, rebuilt on return (cheap, the packages are cached) |
| K9 | **Shared ban lists drift** | Phase 4 slice 3 | The pattern every federation saw: lists grow from spam to opinion | M2: categories, apply or flag, N of M |
| K10 | **iroh-blobs churn** | Phase 2 later | Seven versions in a month; not production quality yet | N6: wait, and D1 for more sources without it |

---

## Who would take what

- **Phase 3 leftovers (links, portals):** R1, N1–N5, K3, K4, K5.
- **Packages:** N6's cheap fix, D1, D2, K7, K8.
- **Animals:** P3, P4, G1–G5, G7, A1–A4, K2, and L1 and L7 when Ronald wants them.
- **Wild plants and worldgen:** P2, G8 (an engine change, Ronald's call).
- **Survival:** P1 (`base_api`), F1–F4, F13.
- **Phase 4:** M1–M3, L6.
- **Phase 6:** W1–W4.
- **CI and Tester:** T1–T3.
- **Engine, if Ronald agrees:** E1, G8.
- **Ronald's call:** I2 (our own relay sooner), the "features only we can do" list.

## Not checked

- No game was run.
- The per-tick numbers are worked out from the code, not measured; T3's benches would measure them.
- The iroh NAT numbers are n0's own, and relay prices are partly unconfirmed.
- The download counts are from mod sites, as a stand-in for what players want.
