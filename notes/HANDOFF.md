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

## Staying up to date with Weft

`upstream` is VortexSo4/Weft. Merge `upstream/master` into a branch, build, check in the game
(one world, and two copies hosting and joining), then fast-forward `main`. Last merge: 6e4e168
(2026-11-01, 74 commits). Weft formats with `rustfmt.toml` and checks it on pull requests: run
`cargo fmt --all` (and `--manifest-path examples/examplemod/Cargo.toml`) before committing, so
merges stay small. What we keep apart on purpose: our networking in `net/p2p.rs` (world
addresses, accounts, worlds hosted under their own keys; Weft's join by IP and port isn't wired
in), and no texture pack built from a Minecraft copy (`minecraft.rs` and its lists stay deleted).
Weft's new random grass (`tallgrass.ctm`) draws as white sticks in Weft itself too.

On macOS, `cargo run` signs the game with the first Apple Development identity
(`tools/macos-sign-run.sh`, `.cargo/config.toml`), so the Keychain's "Always Allow" for the
account key survives rebuilds. Two copies on one Mac: `--run-dir run-alice` and `--run-dir
run-bob`, each with `name = ...` in `config/core.txt` (their accounts live in the run dir's file
store, no Keychain).

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

Phase 3 research (`reports/World addresses links and portals.md`) was adopted into the plan.
Slice 2 (world addresses) is done. `IdentitySigner::world_seed(index, generation)` (wallet, label
"chiacraft world key v1") and `world_cert_text`; `Session::world_seed`, `world_cert_line`,
`check_world_cert` (account). `crates/engine/src/world_address.rs`: `WorldIdentity` (key and
`WorldCert`), `for_world` (gives an old world a random index, saved), the `chiacraft://w/` address
and its parser (old tickets still parse). `WorldMeta.key_index`, `key_generation`, `created`
(`world.txt`). `Game`/`Home` carry the identity; `net::host` hosts under the world key
(`p2p::HostWorld`); `p2p::endpoint(settings, Who)`: the player's key (joins), a world's key (hosting:
ALPNs `weft/1` and `chiacraft/status/1`, publishes to n0 DNS and the Mainline DHT through
`iroh-mainline-address-lookup`), or a throwaway key (status pings). `serve_status`/`status_of`/
`ping`: name, certificate (checked against the key that answered), players, packages. Hosting
first pings its own key and refuses if the world answers. `crates/engine/src/saved_worlds.rs`:
`worlds.txt` in the run folder, remembered on a successful join, pinged while the Multiplayer
screen shows it (online with players and packages, or offline with last seen; owner from the
certificate). Checked with copies: a guest joined by the bare key (looked up, no hints); the
saved list showed the world online, then offline after the host quit, with the owner's identity;
a second copy of the same account and world was refused hosting while the first ran; the
world-swap test from slice 1 still passes with joins on the player's own key.

Slice 3 (link requests) is done. `p2p::ACCOUNT`: the player's endpoint (login key), opened at start
(`start_account`) and kept; joins go out through it (both tries), and it serves
`chiacraft/links/1` (`serve_links`) and runs the outbox loop (`outbox`, woken by `wake_outbox`,
otherwise every 20 s; backoff 60 s doubling to an hour; gives up after 14 days). The world status
now carries the owner's inbox (account endpoint id) with the identity's vouching, checked against
the certificate's owner. `crates/engine/src/links.rs`: `Body` (canonical text; id = BLAKE3; signed
after `chiacraft link v1\n` by each world key), `Request`/`Accept`/`Revoke` messages (JSON on the
wire), `Store` in `links.json` (links, incoming, outgoing, outbox, revoked, allow, blocked);
`request`, `accept(id, always)`, `decline(id, block)`, `revoke`, `withdraw`, `receive` (checks: the
request comes from the account it names for answers, the world signed it, its certificate checks,
the owner vouches for that account; strangers' refusals are silent; 100 waiting at most; one per
pair of worlds). The Links screen (`net::screens::Links`, from Multiplayer): ask from your open
world (address and note), answer requests (Accept, Always from them, Decline), see what you sent,
end links. testkit script: `tap_once` (one click; waits for the screen's name only, since a
focused text field keeps a screen from ever being "ready"), and `tap` stops once its element is
gone. Checked with two copies: a request by bare key reached the other owner's inbox (looked up
through the world's status), was accepted, ended from the other side; "Always from them" accepted
the next request without asking; an ending sent while the other game was closed waited in the
outbox and arrived seconds after both games started again.

Slice 4 (portal blocks) is done. `mods/portals` (MIT): the block `portals:portal` (not solid,
translucent, light 11, animated texture) with a block entity holding the link id
(`portals:link`, `[u8; 64]`, saved). Right-click sends the owner's game the world's links
(`portals:open`), the screen `portals:portal` picks one and sends `portals:bind`; the server checks
the owner again and binds every connected portal block (up to 64). `world_tick` sends a player who
steps in (after a second outside every portal) `portals:travel`; the client joins the world's key
next frame. Messages go through `/portal_note <key> <arg>` so they are translated. The engine side:
`crates/engine/src/bridge.rs` (`chiacraft_bridge` handed to mods at load; `owner`, `links`,
`link`), `server::ServerInfo` (the server's players and world key, on its threads through the
Scope), `Shared.players`, `ServerConfig.world_key`. Joining your own home world's address returns
home. Checked with two copies (alice and bob; same names made `/tp` confusing): the owner's screen
listed the link and bound the portal; stepping in took alice to bob's world while hers ran on; a
guest's right-click got "only the owner"; bob's portal took her home without bouncing back. Test
scripts: player names via `config/core.txt` (`name = alice`).

Slice 5 (travel) is done, and with it Phase 3's slices. `crates/engine/src/travel.rs`: `Ticket`
(one line, signed by the origin world after `chiacraft travel v1\n`), `issue`, `admit` (player =
connection, to = this world, not expired, signed, a link this world has with the origin, nonce
used once), `ARRIVING` (player key → link), `arrival(world dir, key)`, `set_spot`, `expect` (coming
home). `ServerConfig.identity`, `ServerInfo.identity` and `dir`; `Server::join` places an arriving
player at the spot. Bridge: `ticket <entity> <link>`, `spot`/`unspot`. Protocol 5: the hello is
`(protocol, native hash, travel ticket)`. Addresses carry a ticket as `t=`. The portals mod sends
the ticket with `portals:travel` and finds the arrival spot when binding (`arrival_spot`). In
`app.rs`: `Running.travel` (a trip stays in the world it set out from until `JoinEvent::Packages`),
`remote_address`, `came_from` (on a disconnect: the world joined before, else home, else the
title); a failed trip shows `travel.failed` and the player stays. Checked with two copies: alice's
trip to bob's world placed her beside bob's portal ("arrives through a portal at 16 95 0"); bob's
portal took her home beside hers; with bob's game closed her portal left her in her world with
"The portal couldn't take you there: timed out". Not checked: going back to a world joined before
(needs three games).

Phase 3 extras are done. Invites: `links::invite` (world-signed token, `i=` in the address,
`invites_used`), `Request::MakeInvite`, Links screen "Make an Invite". Warm-up: the portals mod's
`near_portals` (within 4 blocks, about twice a second, once per link per 30 s) sends
`portals:near`; the client asks the bridge `warm <key>` (`p2p::warm`: status through the account
endpoint, `warmed`). Arrival protection: `ARRIVAL_PROTECTION` (100 ticks of `hurt_cooldown`).
Friends: `crates/engine/src/friends.rs` (`friends.json`: friends and this player's moves;
`answer_for`, `new_address`, `follow`, the Friends and ConfirmNewAddress screens), `p2p::serve_worlds`
/ `ask_worlds` (`chiacraft/worlds/1`; worlds only to friends, certificates and moves checked
against the friend's identity). Saved worlds keep the owner's inbox (sixth column), learn it right
after a join, and follow a move (`Ping::Following`). World list: "Address" gives the selected world
a new key. Checked with two copies: an invite linked without asking; walking up to a portal warmed
the trip ("on the way"); both added each other from the link; alice joined bob's world from her
Friends screen; bob gave his world a new address; alice's saved entry followed it. Test scripts:
build scripts that contain addresses with Python, not sed (`&` in the replacement).

## Side mods

`mods/animals` (MIT, was `mods/chihuahua` until 2026-11-01, so entity ids are `animals:*` now): wild long-haired chihuahuas (a breed of the species dog: `src/dog.rs`, `source/dog/chihuahua`, `species.rs` `breed`), orange tabby cats, small packs of feathered velociraptors (two species: V. mongoliensis on grassland, the paler V. osmolskae on desert sand), groups of capybaras and rare Utahraptors on grassland (spawn rules like pigs', rarer). Cats (`src/cat.rs`) wander and now and then sit, then lie down as a loaf; their source is `source/cat` (built from research alone, see its README). Velociraptors (`src/raptor.rs`, `source/raptor`) wander and now and then rest; they don't hunt yet. Capybaras (`src/capybara.rs`, `source/capybara`) wander slowly, graze, sit and flop down. The design for taming, stats, breeding and genetics is `notes/ANIMALS.md`; its core is built (2026-11-01): every animal spawns with a wild level (1-150) as points on nine stats (`src/species.rs` per species, `src/animal.rs`), health and speed applied as attribute modifiers and damage through the attack damage chain; sneak-use one you own for the animal screen (stats, points, placing tamed points), the spyglass (`animals:spyglass`, crafted from glass and iron) shows any animal's species, sex, stage and level from 64 blocks; Wild chihuahuas, cats and capybaras are tamed by feeding them when they ask (a bubble with the food above them; favorites and kibble tame faster, mistakes lower effectiveness, a clean tame gives bonus levels; `src/taming.rs`). Tames take commands (follow, stay, wander, guard) and stances (passive, neutral, aggressive, assist) from the animal screen, plain right click (follow/stay) and whistle keys (J follow, K stay, H come, G attack, B passive); tribes (`/tribe`, `/tribeinvite`) share tames (`src/commands.rs`). Signatures (`src/signatures.rs`): chihuahuas bark at and mark monsters and track animals by their drops, cats bring gifts, capybaras calm the animals around them. Raptors are tamed by knocking them out (clubs, tranquilizer darts, sedatives; `src/knockout.rs`) and feeding them while out; wild ones fight back in packs. Saddled tame Utahraptors (`src/utahraptor.rs`, `source/utahraptor`: the mount, 1.5 m at the hip; Velociraptors are too small to ride) can be ridden: the engine now has riding (`World::mount`, held keys, a seated pose; docs/modding.md), steered by the mod (`src/riding.rs`). `/animal [show|tame|points N|wild|here]` for testing.
The dog (2026-11-01) is a papillon-style chihuahua built in Blockbench the way Hytale makes
creatures: 49 boxes and flat fur panels with a hand-painted 512x96 texture, big solid butterfly
ears, a jaw and a tongue, eyelids that blink, toes; animations idle (panting, blinking), walk,
run, sit, lie and lick. Its source is `mods/animals/source/dog/chihuahua` (rebuild with its `build.py`); the
files are exported with the `blocky-model` skill's `to_bedrock.py`
([docs/creatures.md](../docs/creatures.md) has the recipe and the commands). Every animal's coat
comes from its genes (`src/coats.rs`, component `animals:genome`): each client paints it over the
kind's texture through the engine's skin hook (`Registry::entity_skin`), so only 40 bytes travel
per animal. Wild animals spawn from local gene pools that evolve (`src/evolution.rs`: a pool per
species, 256-block cell and habitat, a generation an in-game day, camouflage selection, drift,
mutation, migration; wild ones that die or are tamed leave it, tames released with the screen's
Release join it), saved in the world's `mods/animals/pools.json` (the bridge's `data_dir`). Animals are at real
size (a block a meter) and vary by a size gene and sex through the engine's per-entity scale
(`World::set_scale`, synced in `core:shape`). The old generator (`tools/chihuahua.py`, 96 painted coats) and the 96 coat looks are
gone; they are in git history. Each dog draws as a generated look kind per age (grown or puppy)
and pose (stand, sit, lie down after 30 s sitting), so the pose
reaches every player that comes by (a played animation is a one-off event). A bone tames (1 in 3); a
tamed dog sits on the start, the owner's empty-handed or non-food right-click toggles sitting, it
follows its owner (teleports when 24 blocks behind). Meat puts two tamed adults in love; a male and a female meet
(a sitting one waits for the other) and have a puppy with one allele of each gene from each
(`coats::breed`: it looks like them, hidden recessives surprise, 1% mutate), half the size, grown after a day (meat speeds it up). The owner is the player's key,
asked of the engine bridge (`player_key`, added to `bridge.rs`). Checked in the game: model, coats,
poses, taming, sitting, love, and a puppy born to two dogs in love (with a local patch that started
dogs in love: aiming `press core:use` at wandering dogs from a script is too hit and miss). If the
game aborts at start with "unable to open IOSurface kernel service", the Mac has run out of
IOSurface clients (leaked by hung `simctl` runs); only a reboot clears it.

## Next steps

1. Phase 4 (DIDs and achievements) per `notes/PLAN.md`.
2. Small Phase 3 leftovers: links carried over to a world's new address; arrival spots in other
   dimensions.
2. Phase 2 "later" items: bao-tree verified streaming and resume, iroh-blobs for downloads from
   other players and mirrors, reviewer lists, key rotation; running the fuzz targets (nightly,
   perhaps a scheduled CI job).

## Notes

- Weft draws with Vulkan; on macOS it needs MoltenVK (Weft's release workflow bundles it).
- The Horizon-based project in `~/Coding/horizon` is frozen as it was: batch C (W7-W9) half-done in
  its worktrees, uncommitted; don't prune those worktrees.
- GitHub: `origin` is github.com/ronickg/chiacraft-weft (private); `upstream` is Weft.
