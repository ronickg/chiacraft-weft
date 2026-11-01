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

Slice 4 (joining downloads packages) is done: protocol 4 (`net/p2p.rs`). The hello carries the
code mods' hash (`native_hash`); the host answers with its world's packages (`Offer`: id, version,
hash, size) and serves their manifests and files from its store on extra streams of the same
connection (`serve_packages`), nothing else; the joining game downloads what its store lacks
(`download`: every manifest and file checked against its hash and the declared sizes), loads exactly
the host's set (`use_packages`) and sends the hash of its loaded content, which the host checks
before letting it in. Joining no longer blocks the window: `start_join` reports `JoinEvent`s that
`Running::poll_join` answers, with a `core:joining` screen. Checked with two copies (host with gems,
guest with no packages: it downloads, loads and sees the host's blocks). Not yet: piece-by-piece
verified streaming (bao-tree), resuming, downloading from other players.

Slice 5 (signing and trust) is done. The content key: `IdentitySigner::content_seed` (wallet,
own label), vouched for by the identity key (`content_link_text`, `Session::content_vouch_line`,
`check_content_vouch`). The envelope `package.sig` (`crates/packages/src/signature.rs`) sits beside
the package, outside its hash, and is kept beside it in the store. Signing and checking are in
`crates/engine/src/package_trust.rs` (Ed25519 via iroh's key types), with `weft --sign-package <dir>`.
Offers carry the envelope; a downloaded one is kept only if it checks out. Trust (`trust.txt`,
`test-trust.txt` in automated runs) holds trusted package hashes and creators. When a join needs
untrusted packages, the join sheet (`core:join_sheet`) asks: Download and Join, or also trust the
creators. `examples/packages/gems` is signed by a throwaway example account. Checked with two copies:
first join asks and trusts the creator; that creator's next version downloads without asking; an
unsigned package asks again; a tampered signature shows as refused. testkit's script line
`tap <screen> <element>` clicks through screens.

Slice 6 (hardening) is done, and with it Phase 2's slices. The allow-list is
`identity::check_file` (folder, extension and size per kind; `docs/packages.md`, "What a package may
hold"), checked by `FileManifest::of_dir` and `FileManifest::parse`, so a local package with other
files is left out and a joining game refuses such a manifest before fetching anything.
`assets::decode_png` refuses textures over 4096 a side and runs under `catch_unwind`; the
`packages` mod catches panics while reading a package. In `net/p2p.rs`: `read_frame_max` checks a
packed frame's claimed size before decompressing (that was open to any peer, before admission);
the hello, offer, ready and verdict have small size caps and time limits; the configuration phase
has 15 minutes; `serve_packages` gives each peer a byte budget (twice what is offered), a request
cap and four streams, and streams files in 256 KiB pieces (`send_file`); `Configuring` caps joins
in progress at eight; the guest checks offers (`check_offers`), totals and stalls. `Store::tidy`
runs at start (`modloader::tidy_store`): keeps installed packages and every world's pins, keeps the
most recently used others within 512 MiB (`Store::touch` on use), removes orphan files and crash
leftovers. Fuzz targets in `crates/packages/fuzz` (own workspace; `cargo +nightly fuzz run
manifest`, also `signature`, `defs`); not run yet. Checked with two copies: a 2.4 MB sound streamed
to the guest; a package holding `run.sh` was left out; an 8192-pixel texture showed as missing
without a crash; the store tidy removed a half-done package and stray files and kept the pinned
ones. The per-peer limits and the frame check weren't exercised by a hostile client.

Phase 3 slice 1 (your world runs on) is done. `server::Scope` (`server/mod.rs`): the world's
`Content` and `config::ServerValues` (its synced settings). The server thread enters it, runs its
loop on its own rayon pool whose threads enter it too (`pool.install`), and wraps its two jobs on
the shared background pool in `Scope::with`. `content::content()` reads the thread's content first,
then the game's; `config::get`/`put` read and write the server's values for synced settings on a
server's threads. `ServerHandle.content` is what hosting checks joins against (`p2p::host` takes
it; `accept` computes hashes and decodes with it). In `app.rs`: `leave_for_another` keeps a hosted
world as `Running.home`; `return_home` (`Request::ReturnHome`, pause menu "Back to <world>")
switches back with `modloader::use_built` and `play_local`; a disconnect or failed join goes home;
leaving to the title or quitting stops it (`close_home`). `p2p::HOST_ENDPOINT`: joining while
hosting connects through the hosting endpoint, and `stop_hosting` now closes it. Checked with two
copies swapping worlds (each hosting its own, with different packages, each visiting the other while
the other was away): each guest saw the other world's blocks, both came back to their own through
the pause menu, and a guest whose host quit was taken home.

## Next steps

1. Phase 3 slice 2 (world addresses) per `notes/PLAN.md`.
2. Phase 2 "later" items: bao-tree verified streaming and resume, iroh-blobs for downloads from
   other players and mirrors, reviewer lists, key rotation; running the fuzz targets (nightly,
   perhaps a scheduled CI job).

## Notes

- Weft draws with Vulkan; on macOS it needs MoltenVK (Weft's release workflow bundles it).
- The Horizon-based project in `~/Coding/horizon` is frozen as it was: batch C (W7-W9) half-done in
  its worktrees, uncommitted; don't prune those worktrees.
- GitHub: `origin` is github.com/ronickg/chiacraft-weft (private); `upstream` is Weft.
