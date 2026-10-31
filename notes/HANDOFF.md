# Handoff: chiacraft on Weft

Read `notes/PLAN.md` first: it is the plan and the decisions. This page says where the work stands.

## Where things are (2026-10-31)

- This repository is a fork of [Weft](https://github.com/VortexSo4/Weft) at `f213253`, with Weft's
  history; remote `upstream` points at Weft. Weft's files are MPL-2.0; our crates are MIT.
- Commits on top of Weft:
  1. The texture pack Weft built from a player's Minecraft copy is removed.
  2. `crates/wallet` (`chiacraft-wallet`): the wallet core from the Horizon-based chiacraft
     (`horizon-wallet` at `101dee332`), renamed. Env vars still use the `HORIZON_` prefix
     (`HORIZON_WALLET_STORE`, `HORIZON_WALLET_PASSPHRASE`, the live test tool's); rename later.
  3. `notes/PLAN.md`.
  4. Phase 1, the account (see below).

## Phase 1: built and checked in the game (2026-10-31, on the Mac)

It compiled first time. Wallet (26), account (2) and engine (56) tests pass; clippy is clean on our
crates (Weft's own code has clippy findings, and one deny lint, `mut_from_ref` in `weft_sdk`, left as
upstream has them).

What Phase 1 added is described in `notes/PLAN.md` (phase 1) and in the code: `crates/wallet`,
`crates/account`, `crates/engine/src/account_screen.rs`, the `account` field of `ToServer::Join`.

Checked in the running game:
- `weft --test account` (testkit, `mods/testkit/src/tests/account.rs`): Profile shows the key and
  "Account and Words"; the Account screen; 24 numbered words on request; "I Wrote Them Down";
  nonsense words refused; restoring words; screenshots in `run/test-shots/account/`.
- The account survives a restart, and a restored account is the one the next start opens.
- Two copies with their own `--run-dir`, one hosting (`host` in a `--script`), one joining with the
  ticket: the host logs `joined (account identity …)` with the guest's identity.
- Not checked automatically: the title menu's "Write Down Your Words" after a play session
  (automated runs don't count sessions on purpose).

Fixed on the way: automated runs keep their account in `<run dir>/test-account` (they used the
player's `run/account`); restoring the words of an account this computer already lists switches
to it (it was refused); `chiacraft_*` crates log at info; bare `cargo run` starts the game
(`default-members` is the app and the mods).

## Running and testing

- `cargo run --release` from the repository root (macOS: `brew install molten-vk vulkan-loader`).
- `cargo run --release -- --test account` runs one testkit test against the real game; `--test
  list` names them. Line scripts (`--script`, see `mods/testkit/src/script.rs`) drive host/join.

## Phase 2, packages

Slice 1 (local packages) is done: `crates/packages` (format), `mods/packages` (registers them),
the engine's loader gives each package a slot after the code mods (`modloader.rs`, `ModMeta.package`,
the sort in `content::freeze`). Author guide `docs/packages.md`.

Slice 2 (identity and store) is done: `crates/packages/src/{identity,store}.rs` (BLAKE3 manifest,
store in `run/cache/{blobs,packages}`, verified on every start, packages loaded from the store);
`ModMeta.package` holds the hash and the content hash covers it; `world.txt` pins
`package.<id> = <hash>` and the missing-mods screen lists packages missing or in another version;
blocks of missing content load as `packages:unknown` and keep their names in the save
(`saves.rs`, `Storage.unknown`). The example moved to `examples/packages/gems`; run the game or
tests with `--packages examples/packages` to load it. Tests: `--test packages,package_pins` (with
that flag); the keep-the-name case was checked across three restarts with scripts.

Slice 3 (content swap between worlds) is done: the code mods register once and their
registrations are kept (the raw registration types are `Clone`; a `ColumnCache` clones empty);
`modloader::use_packages(set)` builds the content from them plus a package set, keeps each build
for the life of the game (once per set) and puts it in force (`content::set` is an atomic pointer);
`modloader::for_world` picks a world's set (its pinned versions from the store, plus installed
packages it has not met); `start_world` switches and `Running::content_changed` rebuilds the
client's assets. Mod names by slot (`modloader::NAMES`) and state names (`Content::state_names`)
follow the content. Checked with scripts across runs (two worlds, two versions of gems, A, B, A in one
session) and testkit's new script lines `open <world>` and `leave`. Tests click through menus
with `T::click_until_screen`, since a click during a menu rebuild is lost.

## Next steps

1. Phase 2 slice 4 (configuration phase and transfer) per `notes/PLAN.md`.

## Notes

- Weft draws with Vulkan; on macOS it needs MoltenVK (Weft's release workflow bundles it).
- The Horizon-based project in `~/Coding/horizon` is frozen as it was: batch C (W7-W9) half-done in
  its worktrees, uncommitted; don't prune those worktrees.
- GitHub: `origin` is github.com/ronickg/chiacraft-weft (private); `upstream` is Weft.
