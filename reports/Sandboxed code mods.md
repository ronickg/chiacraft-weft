# Sandboxed code mods: the design (phase 6)

Status: 2026-11-03, agreed (see "Decisions"). Slices 1 (server halves) and 2 (consent marks code)
are built on the branch `claude/phase6-sandboxed-mods-ef818a`.

## The goal

A player walks through a portal into a world whose mods they don't have. The mods download on the
way, with consent, and run. Rule 5 of the plan: players are safe in any world, and mods never
reach the account. Data packages (phase 2) already download and are checked by hash; phase 6 lets
a package carry code that runs in a sandbox, so the main universe gets behaviour mods (commands,
rules, game logic) without native libraries.

## What we started from

- **Horizon's `horizon-mods`** (`~/Coding/horizon/crates/horizon-mods`, frozen): a wasmtime 36
  component host with server and client halves, 11 frozen API versions (1.0 to 1.10), fuel, an
  epoch watchdog, memory caps, strikes, storage quotas and per-version tests. Its engine setup
  (`engine.rs`) is carried over nearly as it was; its API was Horizon's, not Weft's, so the WIT is
  new.
- **Weft's SDK** (`crates/sdk`): mods register closures through `Registry` (commands, events,
  questions, value chains, world ticks, content builders, client hooks) and get a `World` during
  each callback. Closures are boxed `Fn + Send + Sync`, which is what lets one native mod register
  callbacks that call into wasm.
- **Packages** (`crates/packages`, `mods/packages`): a package is the BLAKE3 hash of a manifest of
  its files; each content build registers each package in its own mod slot through the `packages`
  mod; joining downloads what the world pins, verified file by file; trust and consent are per
  package and creator.

## Choices

**wasmtime with the component model**, not an interpreter (wasmi), wasmer or a scripting language
(Lua, Rhai). Components give a typed contract (WIT) checked at link time, strings and lists across
the boundary without hand-written glue, and versioned interfaces; wasmtime is the reference
implementation with an LTS line (36, security fixes into 2027), deterministic fuel metering and
epoch interruption, and it was already reviewed and cached on this machine. Cost: cranelift makes
the `packages` library bigger and a component takes some milliseconds to compile (cached by hash).

**The host lives in our crates** (plan rule 4): `crates/sandbox` (wasmtime, the WIT, limits,
strikes, storage; knows nothing of Weft) and `mods/packages/src/sandboxed.rs` (turns a mod's
registrations into Weft SDK registrations and answers its world calls with the SDK). The engine
changed by one bridge question (`world`: which run of a server, and its folder) and one field
(`ServerInfo.run`).

**The WIT is shaped on Weft's SDK** (`crates/sandbox/wit/v1.0/mod.wit`): `registry` (command,
on, intercept, modify, world-tick, only during `init`), `world` (the SDK's `World` calls),
`names` (names and ids of content), `storage`, `command` (reply), and the exported `hooks` the
game calls with a callback number. The guest SDK (`crates/mod-sdk`) gives these the SDK's own
names and types, closures and all, so `use weft_sdk::*;` with the crate renamed in Cargo.toml
compiles the server code of a Weft mod. What it can't take yet: content builders (content comes
from the package's definitions), components (storage by player key instead), client hooks.

**A package carries it at `code/server.wasm`**, the only code file the allow-list accepts (16
MiB), so it is covered by the package's hash, downloaded and verified like any file, signed with
the package, and pinned by the world's lockfile. Nothing new on the wire.

**Limits** (`docs/sandboxed-mods.md`, "Limits"): fuel per call by kind (deterministic, the same
cut-off on every machine), a 500 ms epoch watchdog as a backstop, 64 MiB memory, store limits on
instances and tables, per-call counts on what reaches the game (tells, replies, block changes,
spawns, logs), storage quotas, and strikes: a failed call drops the instance, three within ten
minutes stop the mod in that world.

**What it may never see.** The linker has only the WIT world's calls: no WASI, so no files,
sockets, clocks, environment or randomness of its own. The world calls are answered by the host
with checks of its own (only real block states and items, no data indexes, never removing a
player, no spawning players, finite coordinates, the engine's own commands reserved). The account
(words, login key, content key, world keys) lives in the game process, but a component only ever
addresses its own linear memory, and no call returns or signs anything with those keys;
`player-key` gives the public key players are already known by.

**Per world.** Each content build registers the package once; each callback finds the instance
for the server run it is on (a world opened again is a new run) and starts it there with that
world's storage (`<world>/mods/packages/<package>/`). Two worlds with the same package never share
memory. Switching content between linked worlds needs nothing new: a visited world's mods run on
its host; the visitor's game only builds the content they describe.

**Re-entrancy.** A mod's own world call can trigger a question or chain the same mod hooks; the
nested callback skips the mod (it is busy) rather than re-entering the store.

## Slices

1. *Server halves* (built): the host, API 1.0, the guest SDK, `code/server.wasm` in packages,
   registration through the `packages` mod, per-world instances and storage, limits and strikes,
   the sample `clans` (the animals mod's tribes) with tests and fuzz targets. Seen working: see
   `notes/HANDOFF.md`.
2. *Consent says it is code* (built): the join sheet's note says code runs sandboxed; offers carry
   "has code" (protocol 6) and the sheet marks those packages; a host that hides code is refused
   when the manifest arrives. Later, perhaps: a trust setting that refuses code alone.
3. *More of the SDK*: components (per-entity values with flags, the biggest gap for porting),
   channels to a mod's client half, block and item callbacks (`on_use`, `on_neighbor`), gizmos,
   sounds and particles. Each is a new frozen API version (1.1, ...) beside 1.0.
4. *Client halves*: `code/client.wasm` in the same package, run in the player's game with
   client calls only (HUD lines, screens built from the SDK's UI parts, keys, channels to its
   server half), the same limits plus a frame budget. Downloaded on join like the rest.
5. *Personal mods per world*: a player's own client-half packages (minimap, tweaks), installed by
   them, never downloaded; each world's owner allows them or not (`world.txt`), and the join tells
   the player which of theirs are off there.
6. *Operations*: `/mods` (which run, their cost per tick, strikes, resume a stopped one),
   ahead-of-time compiled components cached in the store, per-tick budgets across all mods.

## Decisions (agreed 2026-11-03)

Ronald took the recommendations ("Take its recommendations", through Planner):

1. wasmtime 36 LTS components, the host in our crates, WIT v1.0 as written, frozen once a release
   ships it.
2. A package's code lives at `code/server.wasm` (later `code/client.wasm`), under the package's
   hash and signature; no separate code packages.
3. The limits and the strike rule above.
4. Server halves run only on the world's host; joining players download the code (their content
   must match) but never run it.
5. Weft mods port through the alias `weft_sdk = { package = "chiacraft-mod-sdk" }`.
6. A package's command whose name another mod or package already has is left out (with a
   warning) and the world still opens; the engine's own commands stay reserved. Built in slice 2
   (`content::freeze_commands`).
7. One consent sheet for data and code, marking "runs sandboxed code" on the packages that carry
   it. Built in slice 2: offers say whether a package carries code (protocol 6) and the joining
   game refuses a package whose manifest says otherwise.

## Checked against the Researcher's round 1 (W1-W4)

`reports/Research round 1.md` (Researcher's branch), "Mod sandboxing with wasmtime components":

- **W1, freeze the Zed way.** Built that way already: one folder per version
  (`crates/sandbox/wit/v1.0`), one `bindgen!` module; a host enum over the modules comes with the
  second version. Missing: a version range. Proposed: read the version a component needs from its
  own imports (`chiacraft:mods/...@1.x`, which it can't misstate), refuse a too-new one when the
  package loads, and carry it in the offer so the join sheet refuses before downloading. Built with
  1.1, since today there is one version.
- **W2, limits and failures.** Memory: `StoreLimits` at 64 MiB, built. CPU: we meter fuel per call
  (deterministic, the same cut-off on every machine, costlier than epochs) and keep a 500 ms epoch
  watchdog only as a backstop; epochs alone would let a slow machine cut calls a fast one finishes.
  Kept as built; measure the cost before the API is released. Failures: a strike drops the
  instance and starts it over, three in ten minutes stop the mod in that world (the "optional"
  policy). Proposed for later: a `required` flag in `package.toml` (the next format change), where
  a stopped required mod closes the world to joins and sends its players home with the mod named.
- **W3, coarse calls.** Events already come in batches. Block edits are one call each (at most
  4096 a call). Proposed: a `set-blocks` list call in 1.1 and a call-cost benchmark (wasmtime's
  `benches/call.rs` shape) before 1.0 is frozen by a release.
- **W4, WASI 0.3.** Not taken: a mod imports no WASI at all (that is the sandbox), host calls
  answer synchronously inside the callback, and the agreed wasmtime 36 LTS has no 0.3. Client
  halves rare and opt-in, as Hytale does: agreed, they stay slice 4.

## Not done or not checked

- The tests (`crates/sandbox/tests/sandbox.rs`) and fuzz targets (`crates/sandbox/fuzz`) are
  written and compile; CI needs `rustup target add wasm32-unknown-unknown` to build the test mod.
- A hostile component (one calling every import with extreme arguments) hasn't been run; the
  `component` fuzz target only throws bytes at loading.
- Instances of runs that ended stay until four newer runs push them out (memory only).
