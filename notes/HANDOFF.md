# Handoff: chiacraft on Weft

Read `notes/PLAN.md` first: it is the plan and the decisions. This page says where the work stands.

## Where things are (2026-10-31)

- This repository is a fork of [Weft](https://github.com/VortexSo4/Weft) at `f213253`, with Weft's
  history; remote `upstream` points at Weft. Weft's files are MPL-2.0; our crates are MIT.
- Commits on top of Weft:
  1. The texture pack Weft built from a player's Minecraft copy is removed.
  2. `crates/wallet` (`chiacraft-wallet`): the wallet core from the Horizon-based chiacraft
     (`horizon-wallet` at `101dee332`), renamed. Env vars still use the `HORIZON_` prefix
     (the live test tool's); `CHIACRAFT_WALLET_STORE` and `CHIACRAFT_WALLET_PASSPHRASE` replaced
     `HORIZON_WALLET_*` (2026-11-04; the old names are still read).
  3. `notes/PLAN.md`.
  4. Phase 1, the account (see below).

## Phase 1: built and checked in the game (2026-10-31, on the Mac)

It compiled first time. Wallet (26), account (2) and engine (56) tests pass; clippy is clean on our
crates (Weft's own code has clippy findings, and one deny lint, `mut_from_ref` in `weft_sdk`, left as
upstream has them).

What Phase 1 added is described in `notes/PLAN.md` (phase 1) and in the code: `crates/wallet`,
`crates/account`, `crates/engine/src/chiacraft/account_screen.rs`, the `account` field of `ToServer::Join`.

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
`crates/engine/src/chiacraft/package_trust.rs` (Ed25519 via iroh's key types), with `weft --sign-package <dir>`.
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
`check_world_cert` (account). `crates/engine/src/chiacraft/worlds/world_address.rs`: `WorldIdentity` (key and
`WorldCert`), `for_world` (gives an old world a random index, saved), the `chiacraft://w/` address
and its parser (old tickets still parse). `WorldMeta.key_index`, `key_generation`, `created`
(`world.txt`). `Game`/`Home` carry the identity; `net::host` hosts under the world key
(`p2p::HostWorld`); `p2p::endpoint(settings, Who)`: the player's key (joins), a world's key (hosting:
ALPNs `weft/1` and `chiacraft/status/1`, publishes to n0 DNS and the Mainline DHT through
`iroh-mainline-address-lookup`), or a throwaway key (status pings). `serve_status`/`status_of`/
`ping`: name, certificate (checked against the key that answered), players, packages. Hosting
first pings its own key and refuses if the world answers. `crates/engine/src/chiacraft/worlds/saved_worlds.rs`:
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
the certificate's owner. `crates/engine/src/chiacraft/worlds/links.rs`: `Body` (canonical text; id = BLAKE3; signed
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
`crates/engine/src/chiacraft/bridge.rs` (`chiacraft_bridge` handed to mods at load; `owner`, `links`,
`link`), `server::ServerInfo` (the server's players and world key, on its threads through the
Scope), `Shared.players`, `ServerConfig.world_key`. Joining your own home world's address returns
home. Checked with two copies (alice and bob; same names made `/tp` confusing): the owner's screen
listed the link and bound the portal; stepping in took alice to bob's world while hers ran on; a
guest's right-click got "only the owner"; bob's portal took her home without bouncing back. Test
scripts: player names via `config/core.txt` (`name = alice`).

Slice 5 (travel) is done, and with it Phase 3's slices. `crates/engine/src/chiacraft/worlds/travel.rs`: `Ticket`
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
Friends: `crates/engine/src/chiacraft/worlds/friends.rs` (`friends.json`: friends and this player's moves;
`answer_for`, `new_address`, `follow`, the Friends and ConfirmNewAddress screens), `p2p::serve_worlds`
/ `ask_worlds` (`chiacraft/worlds/1`; worlds only to friends, certificates and moves checked
against the friend's identity). Saved worlds keep the owner's inbox (sixth column), learn it right
after a join, and follow a move (`Ping::Following`). World list: "Address" gives the selected world
a new key. Checked with two copies: an invite linked without asking; walking up to a portal warmed
the trip ("on the way"); both added each other from the link; alice joined bob's world from her
Friends screen; bob gave his world a new address; alice's saved entry followed it. Test scripts:
build scripts that contain addresses with Python, not sed (`&` in the replacement).

## Phase 4: admission rules, bans and ban lists (built 2026-11-03, seen working 2026-11-04)

Slices 1-3 of phase 4 (`notes/PLAN.md`). `crates/engine/src/chiacraft/worlds/admission.rs`: the
world's `admission.json` (`Rules`: `only_listed`, `allow`, `probation`, `lists`) and `bans.json`
(`Bans`), the verdict on a join (`verdict`: bans, then the allow list, then followed lists, then
"allow list only"; the owner and operators always in), probation (`probation_for`), signed ban lists
(`BanList`, text `chiacraft ban list v1`, the account's `sign_text`/`check_text`), copies kept in the
world's `banlists/` (newer `seq` wins; `owners.json` maps a followed world's key to its certified
owner), and `Gate`, the server's view of all of it, read again when the files change.
`crates/engine/src/server/admission.rs`: `trusted` (the world certificate's owner, ops; connection
order only for a world without a certificate), `may`/`barred` (probation on breaking, placing, and
containers a block keeps, checked in `actions.rs` and `open_menu`), `send_banned_away` (every 5 s
when the files changed, and on `/ban`), and the commands `/ban`, `/tempban`, `/unban`, `/bans`,
`/allow`, `/probation`, `/banlist` (ADMIN). The join (`player_file.rs`) asks the gate instead of
`whitelist.txt` (a dedicated server's whitelist becomes the allow list once); player files keep
`identity`, `played` (seconds) and `probation = lifted`. Protocol 7: `ToClient::Refused(Msg)` (the
reason in the player's language), the status answer carries the rules' summary (shown in the saved
worlds list), and worlds serve ban lists on `chiacraft/bans/1` and fetch the ones they follow when
hosting starts and daily (`p2p::fetch_lists`). The owner's screen: pause menu, "Who May Come In"
(`admission_screen.rs`), which acts through the same commands. Unit tests in `admission.rs` (not run
here). testkit's new script line `chatf <file> <text with {}>`. The check: `tools/checks/admission`
(two copies, alice and bob, stepping together through `mark`/`await_file`): passed on the PC
(Tester 2, 20e4522): probation refused bob's digging, the ban sent him away and refused him, the
allow list refused then admitted him, the owner lifted his probation, and his own world fetched
alice's published list from hers.

Slices 4 and 6 (2026-11-04, built, not yet seen on testnet11): the engine links the wallet with
`chain`; `crates/engine/src/chiacraft/chain.rs` holds one `ChainReader` per process, configured at
start (`chain::configure`, settings `core:chain` = `testnet11` by default, `off`; `mainnet` refused
in this build; `core:chain_peers`), started by the first `chain::get()`, never from an async task
(the reader runs its own runtime and `block_on`s), cache `<run dir>/chain/<network>.json`. The
account keeps its DID's owner key (`Session::link_did`, `did_proof`, `check_did_proof`; text
`chiacraft did v1`, `sign::did_link_text`). The join carries `did` (protocol 7 also covers it);
the server checks the proof, then the chain on a thread (`server/admission.rs` `check_did`,
`did_answers`), waits up to 25 s only when the rules ask for a DID (`Rules::needs_did`), and keeps
`did` in the player file. `/didrule required on|off`, `/didrule age <days>`, and the screen.

Slices 4-7 passed the two-copy check on testnet11 on the Mac (2026-11-04, Tester, fd673712,
`tools/checks/did`, about 20 minutes): alice's DID from our sponsor reached the chain in about 70 s,
her world's DID in about 3 minutes and her DID's listing of it 3 minutes later; bob was refused for
having no DID, got one, was let in once the world checked it on chain, and his saved worlds list
showed alice's world "DID required, a DID its owner's DID confirms".

Slices 5 and 7 (2026-11-04). `crates/sponsor`
(`chiacraft-sponsor`): Horizon's sponsor on iroh (`chiacraft/sponsor/1`, JSON request and answer,
key in `<data_dir>/sponsor.key`), feature `service` for the binary; a proof of work per request and
the budget released by the hour; world requests (`Request::sign_world`, `WorldAsk`: a world DID
launched to the owner's address with `WorldMeta` and a gift for the listing fee). The wallet:
`chain/world.rs` (`WorldMeta`, `OwnerMeta`, `DidMetadata`, `world_link_expecting`),
`build_eve_dids_with`, `set_did_metadata` (an update carries the new metadata as a memo, since
chia-sdk-driver's `parse_child` assumes it unchanged). The game: `chiacraft/did.rs` (the Account
screen's Get a DID / Link a DID, asking `core:sponsors` through `p2p::ask_sponsor`), and
`worlds/world_did.rs` (`did.json` in the world folder; Get a DID for This World on the Who May
Come In screen, then the owner's DID lists it; the status answer carries both DIDs and the saved
worlds list checks them both ways). Our sponsor: see "Our DID sponsor" below.

## Phase 6, sandboxed code mods

Slice 1 (server halves) is built; the design for Ronald to agree is `reports/Sandboxed code
mods.md` (its "Decisions to agree"), the author guide `docs/sandboxed-mods.md`.
- `crates/sandbox` (`chiacraft-sandbox`, MIT): wasmtime 36 LTS with components. `engine.rs` (the
  engine, fuel per call kind, the 500 ms epoch watchdog, 64 MiB memory and store limits; from
  Horizon's `horizon-mods`), `wit/v1.0/mod.wit` (API `chiacraft:mods@1.0.0`: `registry`, `world`,
  `names`, `storage`, `command`, exported `hooks`), `lib.rs` (`Game`, the game's side of one
  callback; `Mod::load` compiles and runs `init` once to learn the `Hook`s; `Instance`, the mod in
  one world: started on first call, strikes, stopped after three in ten minutes; per-call counts),
  `storage.rs` (keys checked, quotas, written beside and moved over). `componentize` (feature
  `pack`) turns a wasm32 module into a component and loads it to check.
- `crates/mod-sdk` (`chiacraft-mod-sdk`): the guest SDK with Weft's names (`declare_mod!`,
  `Registry::command/on/intercept/modify/world_tick`, `ServerCommand`, `World`, `Block/Item/Entity`
  handles looked up by name, `storage`). Depend on it as `weft_sdk = { package = "chiacraft-mod-sdk" }`.
- Packages: `code/server.wasm` is the one code file the allow-list takes (`identity::SERVER_CODE`,
  16 MiB). `mods/packages/src/sandboxed.rs` loads it when the package registers and turns each hook
  into a Weft SDK registration; each callback finds the instance for its server run (the bridge's
  new `world` question: `ServerInfo.run` and the folder) with storage in
  `<world>/mods/packages/<id>/`. A mod's nested callbacks skip it. The engine's commands are
  reserved. `w.tell` goes to the client as `packages:say` and shows through the client command
  `/packages_say`. Log lines go under the package's slot (`mod:<package>`).
- The join sheet's note no longer says packages are never code.
- Sample `examples/wasm/clans` (the animals mod's tribes, ported: `/clan`, `/claninvite`, storage
  by player key, tells) builds into `examples/packages/clans/code/server.wasm` (`build.sh`).
- Tests `crates/sandbox/tests/sandbox.rs` (with `tests/guest`, built for wasm32 there: limits,
  strikes, storage, allowances) and fuzz targets `crates/sandbox/fuzz` (`component`, `storage`)
  are written and compile; not run here (CI; it needs `rustup target add wasm32-unknown-unknown`).
Checked with two copies (run by the Tester session, scripts in the shared notes' `phase6/`): alice
hosted a world with `--packages examples/packages`; bob, with no packages, was asked, downloaded
clans (with its code) and gems, and loaded them with no error; on alice's world `/clan create
Wolves` ran in the sandbox ("clan Wolves founded" in the log, "Clan Wolves founded." in chat), no
strikes. In the second run bob's game froze when alice's invite reached it: a deadlock in the
`packages` mod (the client's `packages:say` handler held its lock while running `/packages_say`,
which takes it again), fixed in 5223d51; a second possible one (two packages on two region threads
each waiting for the other's lock through a nested question) is avoided by nested calls only
trying the lock. The third run (Linux PC, two copies, 5223d51, `run/test-results/phase6_pc3/`)
passed it all: the join sheet listed clans with "runs sandboxed code", bob downloaded and loaded
it, alice founded Wolves and invited bob, bob joined ("player 7 joined clan Wolves" in alice's
log), both saw "Your clan: Wolves, 2 members.", alice was told "A new member joined Wolves.", no
strikes, panics or errors. Not seen: the invite line in bob's chat; it was sent while bob's world
was still loading in, and a line that reaches a client then may be lost (not looked into).
Slice 2 (consent marks code), after Ronald agreed the report's decisions: protocol 6, the
`Offer` tuple's sixth field says whether the package carries `code/server.wasm`
(`modloader::current_packages`); `download` refuses a package whose manifest says otherwise; the
join sheet marks it (`join_sheet.code`, `OfferRow.code`). In `content::freeze_commands` a
package's command whose name is taken is left out with a warning instead of failing the content
build (code mods keep the old error).
Slice 3, API 1.1 (built; not yet seen in the game): `crates/sandbox/wit/v1.1/mod.wit` is 1.0
plus `registry.component`/`attach` (declarations, `Decl`, compared on restart like hooks),
`world.get-component`/`set-component`, `set-blocks`, `play-sound`, `spawn-particles`. The host
binds 1.1 only: a 1.0 component links by wasmtime's semver matching (checked with clans, and a
test loads its committed wasm). `check_imports` refuses imports outside `chiacraft:mods/*` and
newer 1.x at load. `crates/mod-sdk` is 1.1.0 (`value!`, `Component`, `entity_component`,
`attach`, `w.component`/`set_component`/`update_component`, `Sound`, `Particle`, `Burst`,
`set_blocks`). In `mods/packages`, components register as `Component<[u8; N]>` in buckets of 8
to 256 (`Comp`); sounds and particles are checked by `find` and made into handles once per name.
Sample `examples/wasm/tally` (a per-player count of blocks broken, a chime and particles every
tenth, `/tally pillar` with `set_blocks`), packaged as `examples/packages/tally`. Checked in one game on the Linux PC (Tester 2, with
the API-in-offer work below in the build): `/tally` 0, `/tally pillar` built 5 glass blocks with
one `set_blocks`, digging logged "has broken 10 blocks" up to 90 and told the player every tenth,
`/tally` 94, and after leaving and opening the world again still 94 (the component saved, `KEEP`);
no strikes or load errors. The particles weren't visible in the shots (a dark shaft), the chime
isn't in the log: neither is confirmed. Script `tally.txt` in the shared notes' `phase6/`
(testkit: negative pitch looks down, `hold` doesn't wait).
After slice 3, decided by Planner for Ronald: the API version a mod needs is read from its
imports (`chiacraft_packages::code`, with `wasmparser`; also the one place the game's API, 1.1,
is written) and travels in the offer (protocol 8, after Phase 4's 7: the sixth field is `1.1`, or empty for no
code), so `check_offers` refuses a too-new mod before downloading and the sheet shows "mod API
1.1"; the sandbox checks imports with the same rule before compiling. Storage now holds the WIT's
1 MiB written per call. `callcost` (feature `pack`) times each kind of call against a `NullGame`;
no numbers yet (it needs a release build, which the Mac doesn't do now). The hostile fuzz target
(`crates/sandbox/fuzz`, `hostile` with `hostile-guest`, written by a subagent): 4,188 replayed
inputs broke no promise. Security fix (Research round 4, S8): a mod's `.permission` used to apply
by name to whatever command had that name, so a package whose clashing command was left out could
open another mod's (`/tk give` to everyone); a permission now carries its mod and applies to that
mod's own command only (`content::freeze_commands`, test
`a_left_out_command_leaves_its_permission_too`).
Checked on the Linux PC (Tester 2, c41da9e4, with Phase 4 merged in; logs `~/looktest/p6pair8`):
bob's join sheet listed clans "runs sandboxed code (mod API 1.0)", tally "(mod API 1.1)" and the
signed gems; bob downloaded all three, joined, and the clan flow ran as before (2 members); no
protocol, strike or load errors. `callcost` on that PC (Ryzen 7 9800X3D), microseconds per call:
a command 0.6 to 0.9, an empty event batch 0.2 to 0.3, a batch of 64 block changes 3.5 (clans,
ignoring them) to 14 (tally, a component call each): about 0.2 µs per crossing into the mod and
back. Questions, chain steps and world ticks not measured yet (the test guest needs the wasm32
target, not on the PC).
API 1.2 (built, and checked in one game on the Linux PC by Tester 2 at e6866f7b with the `at`
ABI change: the striker's use on the sky wore it from 3 to 2 ("wears down: 2 left"); using the
bell while holding the striker rang the bell, not the striker (the block's use goes first); the
log had "the bell at [16, 95, 0] rang (1)" and, after stone beside it, the neighbour line; no
strikes or load errors; the scheduled puff wasn't in the shot. Placing a block also gives it a
neighbour update for itself, Weft's way): `crates/sandbox/wit/v1.2/mod.wit` adds
`registry.on-block-use`/`on-block-neighbor`/`on-block-scheduled`/`on-item-use` (the package's own
blocks and items only, `State::own`), `registry.channel`, `world.schedule` and `world.send`, and
the export `content-hooks` (`block-use`, `block-update`, `item-use`, `packet`). The game binds
the world `host`, which exports `hooks` only, and finds `content-hooks` at run time
(`ContentHooks::find`, typed functions with `post_return`): checked that clans (1.0) and tally
(1.1) still load and run in it with `callcost`. A mod registering 1.2 callbacks without the
export is refused. In `mods/packages`, `sandboxed::load` runs before the package's content, so
`block`/`item` take the callbacks on their builders (`on_block`, `on_item`), and
`sandboxed::register` does the rest and refuses a callback for a block or item the package
lacks; channels are `Channel<Vec<u8>>` with at most 64 packets a tick handed over. The SDK is
1.2.0 (`r.block(&B).on_use/on_neighbor/on_scheduled`, `r.item(&I).on_use`, `Use`, `BlockEvent`,
`ItemUse`, `Channel<T>`, `Payload`, `r.packet`, `w.send`, `w.schedule`). `componentize` loads a
component as the package it goes into. Sample `examples/wasm/bell` (a bell block that rings, a
striker item that wears down, a channel), built as a size-optimised dev build since the Mac does
no release builds now. With it, the mod UI ABI got `at` (b2877719: `UiNode::at`, `S::at`,
`S::at_pivot`, onto the engine's `Style::at`), for the UI redo's numbers inside bars.
Slice 4, client halves (built; not yet seen in the game): `code/client.wasm` (allowed beside
`code/server.wasm`, `identity::is_code`), a component for the world `client-mod` in
`crates/sandbox/wit/v1.2` (`client`, `client-registry`, `ui` parts, exported `client-hooks`).
`crates/sandbox/src/client.rs`: `ClientMod`, `ClientInstance` (strikes as server halves;
`well_formed` checks a build's parts), `ClientGame` (the player's game; `can_act` false in
bindings). `crates/mod-sdk-client` (Weft's client names: `hud_element`, `screen`, `key`,
`client_packet`, `client_frame`, `Ui` parts, `S` with `at`, `ClientView`/`Client`).
`mods/packages/src/client_half.rs` registers its hooks through Weft's SDK in every game and
replays its parts into `Ui` (classes interned; flags limited); a client reads a sandboxed
component through the same byte buckets (`sandboxed::client_component`). The offer's API is the
newer half's; "has code" means either half. Sample: `examples/wasm/bell-client` (a `hud_badge`
with the ring count, an `hud_xp` bar with the count inside it through `at`, the key N), with the
bell's server half now sending `bell:rings`. Tests `crates/sandbox/tests/client.rs`.
Checked on the Linux PC (Tester 2, aa03294f): alice's side of the client-half pair ran: the
bell rang and the puff showed above it ("a puff above the bell", seen in the shot; it had been
inside the block), the join sheet listed the bell "(mod API 1.2)"; but the badge drew as the
missing texture: `hud_badge` comes with the UI redo, not in `dev` yet, so the sample now uses
`tooltip`. Bob's side is untested: his game ran out of GPU memory both times on the PC, next to
vLLM ("allocating 67108864 bytes for quad page: Out of memory", a panic in the Vulkan backend).
Bob also joined with an empty name although his `core.txt` says `name = bob`. That was this
work's: the bell's client half registered a key, a key's binding is a setting, and the engine
numbers settings by name per content while their values sit in one table by id, so the content
switch on joining shifted `core:name` onto an empty slot. Fixed in the engine, generally (Planner's call):
`content::set` calls `config::content_switched`, which carries the game's setting values, the
run's overrides and a remote server's values over to the new content's ids by name, and writes
mods' setting handles (`Game::config_cells`) back to the new content's ids; `VALUES` is a table
per content (the old ones kept, so held slices stay valid). Check
`a_content_switch_keeps_values_by_name`. So packages may add settings, and client halves' keys
are their own again (an interim fix had given them shared slots).
Checked on the Mac (Tester, c4a527ea, two copies): bob joined with his name; the sheet listed
the bell "(mod API 1.2)"; `/mods` answered ("bell (mod API 1.2): 1 calls, 121.0 µs each ...").
Found and fixed after it: the input registered key bindings only at start, so a key a package
brings on joining was unknown on the joining game (`content_changed` in `app.rs` now registers
the new content's keys with the player's bindings); the badge's text wrapped a letter a line (it
needs `no_wrap` in a HUD panel). Checked on the PC (Tester 2, 47a8be49, two copies at 480x270 beside vLLM): the whole pair. Bob joined as "bob" and downloaded bell, clans, gems and tally; alice's use rang the bell and the puff showed above it; the badge "Bell: 2 rings" on one line over the bar "2/10", in both games; bob's key: "You listen for the bell." in his chat and "player 2 pinged: ping" in alice's log; `/mods` gave the bell's and clans' calls with 0 strikes. Two rings from one use: the script held use 0.3 s and the engine repeats use every 0.2 s while it is held (the script now holds 0.15 s). Not covered: other kinds of
handle cells (blocks, items, sounds) when a content built earlier comes back.
Slice 5, personal mods (built, and checked in one game on the PC: off by default with "Your
personal mods are off in this world: clock.", on after `/rule personal_mods 1` with the clock
showing 07:07 top left, its key in slot 1 hid it, off again; Create World's choice not looked at): `mods/packages/src/personal.rs`. The
world rule `packages:personal_mods` (choice off/client, off by default, a world rule so it is on
Create World and synced to players); the player's own client halves from `<run dir>/personal/`
(up to 16, loaded on the first client frame); slots registered the same in every game so the
content hash is unchanged: a HUD element per anchor (`personal_<anchor>`, order 1000), the
screen `packages:personal`, keys `packages:personal_1..8` (Shift+Alt+1..8), and a client frame
that runs their frames and tells the player once per world which are off or on. A personal mod's
`Half` has its own `Words` (from its `assets/lang`, the player's language over English), and no
channels or components. Sample `examples/personal/clock` (from `examples/wasm/clock`).
Slice 6 (built; not yet seen in the game): `/mods` (`sandboxed::register_command`, in this
mod's init): each server half running in this world (by the bridge's run) with its API, calls,
average and slowest time and strikes in ten minutes (`Instance::stats`), and why one stopped;
`/mods resume <package>` for admins (`Instance::resume`). Every `Runs` is kept in `ALL` (weak).
Left: the `required` flag (agreed: at the next `package.toml` change), callcost for questions
and chain steps, cached compiled components and a per-tick budget across mods, `/mods` for
client halves, the engine's settings hazard on a content switch (Planner).

## Phase 4: achievements (2026-11-04 overnight, built, not yet seen in a game)

Slices 8-10. `crates/engine/src/chiacraft/achievements.rs` (the player's store, `achievements.json`
in the account home; `received`, `anchored`; `check_issuer`, the world DID confirmed both ways),
`server/achievements.rs` (the server signs what mods grant through the bridge's `achievement`
question, `Shared.grants`, with the owner's account as issuer and the world DID's proof kept in
`did.json`; `achievements/issued.jsonl`; the daily anchor behind the world rule
`anchor_achievements`, `achievements/anchors.json`, proofs sent on join), `achievements_screen.rs`
(Profile > Achievements). The base mod asks the bridge in `advancements.rs` `grant_where`.
World rules `sign_achievements` (on) and `anchor_achievements` (off). Protocol 9. The check:
`tools/checks/achievements` (two copies, no chain).

## Our DID sponsor (testnet11, since 2026-11-04)

Ronald's yes: "testnet only". It runs on Ronald's Mac from the throwaway testnet11 wallet
(`~/chiacraft-testnet/sponsor.words`, never printed or copied), in `~/chiacraft-testnet-sponsor/`:
`sponsor.toml` (testnet11, `data_dir` and `words_file`), its own copy of the binary, `sponsor.log`,
and `data/` (`sponsor.key`, the iroh key behind its address; `sponsor-log.jsonl`, what it launched).
Its address, the game's default `core:sponsors`, is
`d468728d7e0e3f417c822af6e87ff924c0adbdeeb89a3c0f2d0745b2de40a792`. Funding address
`txch124f375vnry5zfk5ss8qv2ucw3ayhhnjz25x7m9w4gspw0rh8u8zs7xhj5x` (top up from a testnet faucet).
To start it again:

```sh
cd ~/chiacraft-testnet-sponsor && ./chiacraft-sponsor --config sponsor.toml >> sponsor.log 2>&1 &
```

Mainnet stays off: the binary refuses mainnet without `allow_mainnet = true`, and that needs
Ronald's own yes.

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
follows its owner (teleports when 24 blocks behind). A favorite food puts a tamed adult with bond 50 in love (`src/breeding.rs`, every species:
litters for mammals, eggs for raptors); a male and a female meet (a sitting one waits for the
other) and have young with one allele of each gene from each (`coats::breed`: they look like
them, hidden recessives surprise, 1% mutate), a puppy half the size, grown after three days (a favorite food speeds it up). The owner is the player's key,
asked of the engine bridge (`player_key`, added to `chiacraft/bridge.rs`). Checked in the game: model, coats,
poses, taming, sitting, love, and a puppy born to two dogs in love (with a local patch that started
dogs in love: aiming `press core:use` at wandering dogs from a script is too hit and miss). If the
game aborts at start with "unable to open IOSurface kernel service", the Mac has run out of
IOSurface clients (leaked by hung `simctl` runs); only a reboot clears it.

Server identity (2026-11-04, branch `claude/server-identity-519f6e`, off `dev`): `weft --server`
exited with "has no address of its own: no account is open". `chiacraft_account::open_server`
opens (making it once) an account called "Server" in `<run dir>/server-account` (file store),
and `dedicated.rs` opens it before the world's key; the world's keys and certificate derive from it
as from a player's account. Phase 7's part (an owner certifying their server) is in the plan.

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
