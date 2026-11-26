# Research round 5: is the code getting bloated?

Researcher, 2026-11-04, for Planner and Ronald. I read `dev` at `30de857`.

## The verdict for Ronald

**No. It is big, but most of the growth is real features and Weft's own merges. The copied code
is small (about 1,450 lines, under 3% of ours), and two thirds of it sits in the animals mod.**
The things worth fixing aren't size:
- a secrets test that checks code we no longer have;
- a store bug written a third time;
- some art stored twice.

## 1. Ours or Weft's

Raw Rust lines (comments and blanks included):

| point | Rust lines | what |
|---|---|---|
| the fork, `f213253` | 64,398 | Weft as we took it |
| Weft merged, `6e4e168` | 93,870 | +29.5k net from merging Weft (74 commits, 2026-11-01) |
| `dev`, `30de857` | 146,015 | +52.1k net of ours |

**About 36% of today's Rust is ours, and 64% is Weft's.** Most of the engine's growth is Weft's:
its merge brought the engine +23.8k / −5.8k lines, against our +9.7k / −1.2k.

Ours, by crate and mod (raw lines added and removed, from `reports/code-health/health.py`):

| unit | ours + | ours − | what it is |
|---|---|---|---|
| crates/wallet | 9,771 | 0 | the wallet, ported whole from the Horizon project (2.5k of it tests) |
| mods/animals | 9,744 | 0 | 35 species' behaviour, taming, breeding, eggs, hunting, screens |
| crates/engine | 9,700 | 1,233 | accounts, links, travel, friends, admission, `p2p.rs`, entity ranges |
| mods/animals/core | 7,231 | 0 | species facts (a third of it research comments), genetics, the sheet |
| mods/base/vitals | 4,126 | 0 | survival numbers: climate, air, food, plants, clothing |
| mods/base | 2,960 | 217 | survival, clothing, cooking, wild plants |
| crates/sandbox, mod-sdk | 3,702 | 0 | Phase 6 |
| crates/packages, mods/packages | 2,546 | 0 | Phase 2 |
| mods/testkit | 1,092 | 66 | our tests (Weft's 7.1k are its own) |
| the rest | about 3,000 | about 30 | account, portals, base API, the species website, examples |

Code lines today, blanks and comments out:

| crate or mod | code lines |
|---|---|
| engine | 57.2k |
| Weft's SDK | 7.7k |
| animals mod | 7.7k |
| wallet | 5.5k |
| animals core | 5.5k |
| base | 5.4k |
| vitals | 2.8k |

Tests are 16k lines on top, 9.4k of them testkit, which is 93% Weft's.

**The pace.** 238 commits of ours since Weft's merge, 161 of them since 2026-11-03. Nearly all of
the 52k are features from the plan's phases 1–4 and 6, and the survival and animal systems. Only
the wallet arrived whole.

## 2. Where the bloat is

### Written twice across sessions

**Total that can go with nothing lost: about 1,450 lines**, plus about 220 more if simple coat
looks become data.

| owner | lines |
|---|---|
| animals | about 880 |
| Phase 4 | 150 |
| Phase 3 | 100 |
| wallet and account | 65 |
| survival | 35 |
| the shadow pass | 35 |
| sandbox | 30 |
| packages | 25 |
| wild plants | 25 |

**Half of round 1's fixes are done, but not in `dev`.** The Phase 3 leftovers branch (7 commits
ahead) has:
- a `crates/store` crate (`chiacraft-store`: `JsonStore`, `read_json`, `write_atomic`,
  `keep_aside`), used by links, friends, saved worlds, trust, letters and the animals' pools (R1);
- a `request`/`respond` pair for the one-shot iroh exchanges (N1).

Merging that branch comes first; several fixes below build on it.

**Files are loaded and saved 15 ways in `dev`,** and 3 more on branches:

| how | where |
|---|---|
| plain write | `links.rs:225`, `friends.rs:68`, `saved_worlds.rs:69`, `package_trust.rs:110`, `travel.rs:212`, `server/admission.rs:506` |
| own temporary file and rename | `animals/store.rs:22`, `packages/store.rs:297`, `sandbox/storage.rs:79`, `chiacraft/worlds/admission.rs:425`, `wallet/chain/cache.rs:101`; on branches, `world_did.rs:57` and the store crate |
| private files written by hand (mode 0600) | `wallet/store/file.rs:109`, `mnemonic.rs:93`, `sponsor/net.rs:58` |

**And R1's bug is back.** `chiacraft/worlds/admission.rs:414` `read_json` turns a file it can't
parse into an empty one, and the next `write_json` keeps it. A damaged `bans.json` loses every
ban. I checked this one myself.

**Copies, from the code-health script and the reviews:**
- `wake` is written 7 times and `settle` twice in the animal files, with about 70 lines of rest
  code in each of 7 species files (`capybara.rs:83/116`, `cat.rs:84/117`, `wolf.rs:87/120`,
  `jaguar.rs:86/123`, `raptor.rs:126/152`, `utahraptor.rs:90/117`, `wyvern.rs:144/171`).
- Hex is written 9 times, though the `hex` crate is already in `Cargo.lock`:
  - `package_trust.rs:20`, `links.rs:238`, `travel.rs:82` and `:159`;
  - `wallet/keys.rs:166`, `account/lib.rs:476`, `packages/identity.rs:81`;
  - `animals/web/src/lib.rs:126`, and `world_did.rs:47` on Phase 4.
- "Parse a certificate, check it's this world's, verify" is written 4 times; "parse a key, unhex,
  verify" 3 times; random numbers from `RandomState` 4 times.
- Short ids for people are written 6 ways in 3 formats. `server/admission.rs:337` slices
  `&v[..16]` by bytes, which panics on a non-ASCII character there.
- Clock helpers are written 7 times outside the engine, and `dist2` 7 times in animals.
- The one-shot iroh call gets a third copy in the sponsor (`sponsor/client.rs:55`, `net.rs:117`).
- `chiacraft_bridge` and its `ask` are written 3 times (`mods/{animals,packages,portals}/src/lib.rs`).
- Foods are listed twice, in `items.rs:74-109` and `vitals::food::FOODS`.
- The old berry bush has its own code in 5 places beside the wild plants' table.
- `species.rs` repeats default fields in nearly all of its 37 rows: `wild_step`/`tamed_step` in
  37, `breed: None` in 36, `flight: 0.0` in 36, and so on.
- The order of species is written by hand in 4 places, with about 30 bare numbers like
  `behave(1)`.

### Dead, stale or compiled in for nothing

- **The wallet's secrets test checks code we no longer have.**
  - `crates/wallet/tests/no_secret_leaks.rs` (573 lines) reads `crates/horizon-client`,
    `horizon-wallet`, `horizon-telemetry` and `packaging/onboarding`, none of which exist. I
    checked this myself.
  - About 8 of its 15 tests panic on a missing file, or fail `files.len() >= 8` (`:277`), when CI
    runs them.
  - So the "no secret reaches a log" rule isn't checked on `crates/account`,
    `engine/src/chiacraft` or `server/admission.rs` at all.
- **The wallet's live test tool is compiled into the game.**
  - `chain/tool.rs` (799 lines) is used only by the `chiacraft-wallet-test` bin. But the engine
    turns on `chain`, so the game builds it.
  - A `tool` feature with `required-features` on the bin takes 845 lines out of the game.
- **Horizon leftovers:**
  - The game reads `HORIZON_WALLET_STORE` and `HORIZON_WALLET_PASSPHRASE` (`accounts.rs:110, 151`),
    beside `CHIACRAFT_ACCOUNT_HOME`. There are more `HORIZON_*` names in `chain/tool.rs` and
    `tests/chain_live.rs`.
  - `examples/chain-sim.rs` (86 lines) served Horizon's admission tests.
  - 26 comments cite files that don't exist (`docs/identity-live-test.md`, `PROJECT.md`,
    `tools/chiacraft/*.py`, `packaging/`).
- **CI steps for crates not in `dev`:** "Tests (sponsor)" and "Tests (store)" (`ci.yml:63-75`).
- **Wallet tests repeat their helpers:** `until()` 5 times and `reader()` 3 times across
  `tests/*_sim.rs`. `chia-sdk-test` is both optional and a dev dependency.
- **Small dead code, about 35 lines:** `achievement.rs:283 issuer_fingerprint`,
  `accounts.rs:261 create_with`, `chain/reader.rs:472 cached_did` and `:505 unwatch`,
  `vitals/food.rs:77 CATEGORIES`, `vitals/stamina.rs:11, 14`.
- **Kept on purpose:** the Phase 4 wallet code (`achievement.rs`, `spending.rs`, `offer.rs`,
  `holdings.rs`, `anchor.rs`, 1.6k lines) is unused today but planned.
- **Clean otherwise:**
  - all of `engine/src/chiacraft` is used;
  - no commented-out code blocks;
  - no TODO or FIXME;
  - the only allowances are 4 `too_many_arguments` and one `unsafe_code`.

### Dependencies

- `Cargo.lock` holds 871 packages, 78 of them at two or more versions, mostly from Weft (winit,
  objc2, naga, kira's glam).
- **Ours worth fixing: the wasm tools come in three versions:**
  - 0.236 from wasmtime 36;
  - 0.239 from `chiacraft-packages`' `wasmparser = "0.239"` and sandbox's `wit-component`;
  - 0.247 from mod-sdk's `wit-bindgen`.

  Pinning packages and sandbox to wasmtime's 0.236 takes 5 crates and a `wasmparser` copy out of
  the game build.
- **The Chia stack** pulls the `rue-*` compiler crates (about 25, including a second `chia-bls`)
  into the game, because `chia-sdk-types` 0.37 doesn't make them optional. That needs an upstream
  feature request (L).
- **Our direct dependencies are all used.** The script's one hit, `raw-window-handle` in
  `backends/vulkan`, is Weft's and still pulled in by `weft_render`, so leave it.

### Big files and functions

- **Big because they are tables (fine):**
  - `species.rs`: 1,912 lines of rows and 633 of research comments;
  - base's `blocks.rs register` (495 lines, ours +243);
  - the worldgen, items and hud `register`s;
  - the species sheet's `species()`.
- **Weft's biggest:** `registry.rs freeze` (616 lines), `actions.rs handle` (510),
  `entity_render.rs draw` (423). These are rule 4's to leave.
- **Ours worth splitting:**

  | file | why | seams |
  |---|---|---|
  | `admission_screen.rs build` | 292 lines | a stepper written twice, a toggle 5 times, a list row 3 times; helpers and five sections save about 60 lines |
  | `p2p.rs` | 1,357 lines (1,445 on Phase 3), edited by three sessions; splitting is for merge conflicts, not lines | wire and endpoints; account inbox, links and worlds; host, status and bans; accepting a join; packages; joining |
  | `links.rs` | 744 lines, about 1,100 after Phase 3 | signed types, requests and answers, invites, the outbox |
  | `coats.rs` | 2,451 lines; already has section headers | genes, genomes, looks, described, paint |
  | animals `screen.rs` | 719 lines | the look-at ray and the `/animal` command into their own files |

- The shadow pass we added to Weft's `world_render.rs` repeats `draw`'s buffer filling (14- and
  10-line blocks). A shared helper would touch Weft's `draw`, so it's Ronald's call.

### The guest SDK against Weft's SDK

- `crates/mod-sdk` shares 66 function names with `crates/sdk`, but only about 36 lines are the
  same: they mirror the API's shape, not its code. Generating it would gain little.
- **The names have already drifted:** `mod-sdk:477-482` calls permission level 4 `HOST`, where
  `weft_abi:2590` calls it `OWNER`. Re-export `permission`, `stage` and `component_flags` from
  `weft_abi` (it is `no_std` and builds for wasm).
- The event, question and chain lists are written by hand three times: the WIT, the mod-sdk
  statics, and `mods/packages/src/sandboxed.rs:294-347`. A check that one sample mod compiles
  against both SDKs keeps them honest.

### Tests

testkit's 9.4k lines are 93% Weft's, untouched since its merge. Ours are small and not
copy-pasted: `account.rs` 98 lines, `packages.rs` 124, about 630 more in `gameplay.rs` and
`script.rs`, and `watchdog.rs` 103.

### Debug and temporary code

- **`/feed` is open to everyone** (`mods/base/src/survival/food.rs:51-65`). It is a test command
  that fills the stomach, so in multiplayer it is a cheat. Removing `.permission(EVERYONE)` puts
  it at game master.
- All the other test commands are game-master only: `/animal`, `/growth`, `/air`, `/rain`,
  `/season`, `/xp`, `/advancements`. `/tk` doesn't ship.
- **Info logging on every animal's routine:** each bite, drink, comfort move, spawn and despawn
  (`drink.rs:145, 157`, `forage.rs:191, 258`, `comfort.rs:201`, `evolution.rs:450`, about 10 more
  in `hunt.rs` and `breeding.rs`). With hundreds of wild animals the log floods. Make them debug.

### Checked-in weight

Git tracks 3,355 files, 54.9 MB, packed to 63 MB. **The weight is art, not code:**

| kind | size |
|---|---|
| PNG | 27.5 MB, 1,935 files (14.0 MB in the game-art skill, about 8 MB in `notes/`, 6.1 MB in base's assets) |
| JPG | 6.7 MB |
| JSON | 4.8 MB, 3.3 MB of it animations |
| WAV | 4.7 MB (two of Weft's music tracks) |
| fonts | 1.3 MB |
| generated | 188 KB (sheets and the overview board, checked by tests: fine) |

What can go:
- **The 7 UI reference sheets are stored twice:** as PNGs in
  `.claude/skills/game-art/guides/6 UI/UI Prompt References/` (13.7 MB, used by the art
  generation) and as JPGs in `notes/ui-reference/`. Drop the JPGs, 2.9 MB.
- **Animations are 55% held keys:** keys that repeat their neighbours on both sides (wyvern:
  7,074 of 11,518). The engine interpolates linearly unless a key says otherwise (`model.rs:344`),
  so the exporter can drop them and write compact JSON. That saves about 2 MB and changes nothing
  on screen.
- **`notes/texture-test/`** (2.1 MB) is the old 16 px pipeline, replaced by `notes/blocks/`. Keep
  `palette.png`.
- **The static Alegreya Sans fonts** are covered by the variable one, 0.8 MB.
- **Later:** git LFS or a separate art repository for `.claude/skills/game-art` and `notes/ui-*`,
  about 22 MB.

## 3. Quick wins, ranked by lines saved × safety

| # | what | saves | risk | owner | effort |
|---|---|---|---|---|---|
| 1 | Merge the Phase 3 leftovers branch, then move every load-and-save onto `chiacraft-store` (with a mode parameter for 0600 files). This fixes `bans.json`'s empty-on-damage bug. | about 100 lines and a data-loss bug | low | Planner, then Phase 3 and Phase 4 | S + M |
| 2 | Point the wallet's secrets test at our code (`wallet/src/chain`, `crates/account`, `engine/src/chiacraft`, `server/admission.rs`), and drop the Horizon-only tests | about 120 lines, and the rule checked again | none | wallet/account | M |
| 3 | One `rest.rs` and a posed-creature row for the 7 animal files | about 450 lines | low | animals | M |
| 4 | `..BASE` defaults in `species.rs` (and `Biology`) | about 265 lines | very low (the sheet test catches changes) | animals | S |
| 5 | Gate the wallet's test tool behind a feature; rename `HORIZON_*` to `CHIACRAFT_*`; drop `chain-sim` and the CI steps for crates not in `dev` | 845 lines out of the game build, 90 lines gone | low | wallet, CI | S |
| 6 | One source for hex, signing checks, random numbers and short ids (`hex` crate, `WorldCert::owner_of`, `getrandom`, one `short_id`). Fixes the byte-slice panic. | about 200 lines | low | Phase 3, account, packages | S |
| 7 | Species index by name (`const fn index`) instead of hand-kept order and bare numbers | about 50 lines, and a class of quiet bugs | low | animals | S |
| 8 | Pin the wasm tools to one version | 5 crates out of the game build | low | packages, sandbox | S |
| 9 | Re-export `permission`, `stage`, `component_flags` from `weft_abi` in mod-sdk (`HOST` vs `OWNER`) | about 30 lines, and a drift fixed | low | Phase 6 | S |
| 10 | `/feed` to game master; per-animal info logs to debug | a cheat closed, the log readable | none | survival, animals | S |

**Then, in the order they pay:**
- the admission screen's helpers (about 60 lines, Phase 4);
- `chiacraft_bridge!` as one macro (about 35, animals, packages, portals);
- foods registered from `FOODS` (about 30, survival);
- the berry bush as a `WILD` row (about 25, wild plants);
- the wallet tests' `common` module (about 60, wallet);
- `dist2` and the clocks (about 35);
- the art weight (about 6 MB, art);
- splitting `p2p.rs`, `links.rs` and `coats.rs` along their seams, which saves no lines but cuts
  merge conflicts between sessions.

## 4. The repeatable check

`reports/code-health/health.py`, with `reports/code-health/README.md`.
- Standard library only, read-only, under 2 seconds on `dev`.
- It reports:
  - lines per crate and mod (code and tests apart);
  - growth since the fork, split into Weft's and ours;
  - the 15 biggest files and functions;
  - duplicate blocks (8+ identical lines);
  - functions written more than once (same body, 5+ lines, or a small helper in 3+ places);
  - dependencies never named in their crate;
  - debug leftovers (`dbg!`, `todo!`, `allow(dead_code)`, FIXME, `println!` outside bins);
  - big checked-in files that aren't code.
- `--json` writes the numbers for comparing runs. `--max-dup-blocks` and `--max-big-fns` make it
  fail.
- **Today's report** is `reports/code-health/2026-11-04 dev 30de857.md`, the baseline:
  - 105.8k code lines and 16.0k test lines;
  - 33 duplicate blocks;
  - 21 functions written more than once;
  - 37 functions over 150 lines;
  - 1 dependency to check by hand.

**For CI:** the README has the job to add to `.github/workflows/ci.yml`.
- It runs on pushes to `dev` and the trunk with `fetch-depth: 0`, and fetches Weft's `master` for
  the growth table.
- It writes the report into the run's summary and uploads `health.md` and `health.json`.
- It fails nothing at first. Once quick wins 3, 4 and 6 land, add
  `--max-dup-blocks N --max-big-fns M` at the day's numbers plus a little, so a new copy-pasted
  block fails its pull request with the reason.
- Optionally, add `cargo machete` in the same job for unused dependencies (it needs no build).

**For Planner:** `python3 -I reports/code-health/health.py` from `dev`'s worktree, every few days.
Compare its tables with the last report's.

## Not checked

- No game was run, and no tests were run.
- The secrets test's failures are read from the code; CI's next run on `dev` will show them.
- The duplicate finder works on trimmed text, so copies that were renamed or reordered slip past
  it. The reviews found those by reading.
- Line savings are estimates from the code read.
