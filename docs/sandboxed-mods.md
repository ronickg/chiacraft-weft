# Sandboxed mods

A package (`docs/packages.md`) may carry code: a **server half**, `code/server.wasm`, a
WebAssembly component the game runs where the world is simulated. It downloads with the package
when a player joins a world, the same way blocks and textures do, pinned by the package's hash.
It runs in a sandbox: it reaches the game only through the calls listed below, never the
player's account, words or keys, files, the network or the computer's clock, and every call it
gets has limits on time and memory. (chiacraft plan, phase 6; the design is
`reports/Sandboxed code mods.md`.)

Native Weft mods (`docs/modding.md`) are a different thing: unsandboxed libraries, off by
default, each set of them its own universe. A sandboxed mod joins the main universe.

## A mod in five minutes

`examples/wasm/clans` is a complete one: the animals mod's tribes, ported. A mod is a `cdylib`
crate built for `wasm32-unknown-unknown` against `crates/mod-sdk`:

```toml
[package]
name = "clans"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
# Under the name of Weft's SDK, so code moved from a Weft mod keeps its `use weft_sdk::*;`.
weft_sdk = { package = "chiacraft-mod-sdk", path = "../../../crates/mod-sdk" }

[workspace]
```

```rust
use weft_sdk::*;

declare_mod!(deps: [], init: init);

fn init(r: &mut Registry) {
    r.command("clan", "[action:word(create,join,leave,info)] [name:text]", |cmd| {
        cmd.reply("command.clans.none", &[]);
    })
    .permission(permission::EVERYONE);
    r.on(&PLAYER_JOINED, |b| {
        let w = b.world();
        for e in b.iter() {
            w.tell(e.player, "clans.welcome", &[Arg::Text("Wolves")]);
        }
    });
}
```

Build it and put it in its package:

```sh
rustup target add wasm32-unknown-unknown     # once
cargo build --release --target wasm32-unknown-unknown
cargo run -p chiacraft-sandbox --features pack --bin componentize -- \
    target/wasm32-unknown-unknown/release/clans.wasm ../../packages/clans/code/server.wasm
```

`componentize` turns the module into a component, loads it in the sandbox as the game would and
prints what it registers. The package then holds:

```
clans/
  package.toml
  code/server.wasm
  assets/lang/en_us.lang      (the lines the mod's replies and tells use)
```

Start a game with `--packages examples/packages` and a new world plays with it; players who join
download it.

## What a mod can do (API 1.3)

The contract is `crates/sandbox/wit/v1.3/mod.wit` (1.0 and the calls 1.1, 1.2 and 1.3 added). The
SDK gives it Weft's names:

| Weft's SDK | Sandboxed SDK |
|---|---|
| `r.command(name, usage, f).permission(..)` | the same; `cmd.player()`, `int`, `float`, `text`, `id`, `targets`, `pos`, `has`, `reply(key, args)` |
| `r.on(&BLOCK_BROKEN, ..)` and the other built-in events | the same: `BLOCK_BROKEN`, `BLOCK_PLACED`, `ENTITY_DIED`, `ENTITY_HURT`, `PLAYER_JOINED`, `PLAYER_RESPAWNED`, `ITEM_USED`, `ITEM_PICKED_UP` |
| `r.intercept(&question::BREAK_BLOCK, ..)` | the same, every engine question |
| `r.modify(&chain::DAMAGE, stage::MULTIPLY, ..)` | the same, `DAMAGE` and `ATTACK_DAMAGE` |
| `r.world_tick(..)` | the same |
| `w.get`, `set`, `time`, `seed`, `random`, `random_below`, `dimension`, `light`, `biome`, `players`, `entity`, `near`, `push`, `damage`, `spawn`, `remove`, `creative`, `inventory(..).insert`, `spawn_item`, `prop`, `set_prop` | the same |
| `Block::named`, `Item::named`, `Entity::named`, `.id()`, `.is(state)`, `block_name`, `item_name`, `state_prop`, `state_with` | the same; ids are looked up in the world the mod runs in |
| `info!`, `warn!`, ... | the same, in the game's log under the package's name |
| `value!`, `Component::named`, `r.entity_component(&C, default, flags)`, `r.attach(&KIND, &C)`, `w.component(id, &C)` | the same (1.1), but by value: `w.component` gives a copy, `w.set_component(id, &C, v)` and `w.update_component(id, &C, \|v\| ..)` write it; at most 256 bytes, the mod's own components only |
| `w.play_sound(&S, pos, volume)`, `w.spawn_particles(&P, pos).count(n)` | the same (1.1); sounds and particles by full name (`base:ui.advancement`, `core:poof`) |
| many `w.set` | `w.set_blocks(&[(pos, state)])` (1.1), one call |
| `r.block(&B).on_use(\|u\| ..).on_neighbor(..).on_scheduled(..)`, `w.schedule(pos, ticks)` | the same (1.2), on the package's own blocks only (from its `defs/blocks`): the callbacks of Weft's block builder without the rest, which is in the definitions. A use on the block goes before the held item's, and a block just placed gets a neighbour update for itself |
| `r.item(&I).on_use(\|u\| ..)`, `u.set_stack(..)` | the same (1.2), on the package's own items (`defs/items`); `u.slot()` is where the stack is |
| `static C: Channel<T>`, `r.packet(&C, \|p, v\| ..)`, `w.send(player, &C, &v)`, `p.reply(..)` | the same (1.2), the package's own channels, at most 16 KiB a packet and 64 packets a tick handed to the mod; on the wire it is a `Channel<Vec<u8>>`, so a native client mod talks to it with one of those |
| content builders, screens, HUD, keys, client hooks | not yet: content comes from the package's definitions; client halves come later |

Ours, beyond Weft's: `w.tell(player, key, args)` (a line in a player's chat, in their language),
`w.player_key(player)` (who they are, the same in every world), `w.teleport(id, pos)`,
`storage::{get, get_text, set, remove, keys}` (the mod's own data in this world's save), and
`w.grant_achievement(player, id, title, description, icon)` (1.3, below).

### Achievements (1.3)

`w.grant_achievement(player, "first_ring", "bell.achievement.first_ring",
"bell.achievement.first_ring.desc", Some("bell:bell"))` tells the world the player earned it: the
world signs an achievement credential for them, as it does for the base game's advancements
(`notes/PLAN.md`, phase 4, slice 8), and their game keeps it in their account, on the Profile's
Achievements screen, in every world they go to. The id gets the package's namespace
(`bell:first_ring`), so a mod can't grant another's; title and description are the package's lang
keys (or text), at most 64 and 256 characters; the icon is an item's full name. Granting the same
id again signs it again, so a mod keeps who has what itself (`storage`). A player without an
account gets none, and a world whose rule `sign_achievements` is off signs none. A world's owner
can ask for one at the door (`/achievementrule bell:first_ring`).

Every line a mod shows players is a translation key and arguments, worded by each player's game
from the package's `assets/lang` files, like Weft's command replies.

## Client halves

A package may also carry `code/client.wasm`, a client half (API 1.2): a component for the world
`chiacraft:mods/client-mod`, built against `crates/mod-sdk-client`, that runs in each player's
own game (the host's too). It reads what that game shows the player (`c.me()`, `day_time`,
`raining`, `creative`, the package's components synced to them, `inventory_slot`, `screen`) and
adds, with Weft's client names:

- HUD elements and screens: `r.hud_element(&H, hud_anchor::TOP_RIGHT, 0, |ui| ..)`,
  `r.screen(&S, |ui| ..).pauses()`, built from `ui.panel`, `row`, `column`, `text`, `text_bind`,
  `progress_bind`, `icons`, `image`, `item`, `spacer`, `button` with `S` styles, `at` included;
- keys: `r.key(&K, "N").on_press(|c| ..)`, rebindable in Controls (named by
  `key.<package>.<name>` in the package's language file);
- packets from its server half: `r.client_packet(&CH, |c, v| ..)`, and `c.send(&CH, &v)` back;
- `r.client_frame(|c, seconds| ..)`, every frame (off unless needed).

A HUD element or screen is built as a list of parts the game replays into its own UI, so a
client half draws with the game's kit: use its classes (`panel`, `tooltip`, `button`, `slot`,
`hud_frame`, the `hud_health`, `hud_stamina`, `hud_xp` and `hud_temperature` bars; with the UI
redo also `hud_badge`, `tab` and `category`; text classes `title`, `dim`, `button`, `chat`) and
it looks like the game. A class the game has no texture for draws as the missing texture. A
binding (`text_bind`, `progress_bind`, `icons`) may only read; keys, buttons, packets and frames
may also send, open screens and show lines. It never reaches the account, files or network, and
the same limits and strikes hold (a failing client half stops in the player's game). Lines it
shows are translation keys from the package's `assets/lang`; key names too (`key.<package>.<name>`).

`examples/wasm/bell-client` is the bell's client half: a `hud_badge` with the ring count and an
`hud_xp` bar with the count inside it, fed by the server half on `bell:rings`, and the key N
asking for it on `bell:ping`. Build it like a server half; `componentize` treats an output named
`client.wasm` as a client half.

## Personal mods

A player's own client halves, never downloaded and never shared: a minimap, a clock, a tweak.
Each is a folder like a package's with `package.toml` and `code/client.wasm` (and its
`assets/lang`), in `<run dir>/personal/<name>/`. `examples/personal/clock` is one.

Each world's owner decides whether they run there: the world rule **Personal Mods** (Off or
Client Halves; off by default), on Create World and in the world's settings, or `/rule
personal_mods 1`. Where it is off they don't run, and on joining the player is told which of
theirs are off ("Your personal mods are off in this world: clock").

A personal mod is written like any client half, with these differences, because it must not
change what the world and the player's game agree on (the join checks the content's hash):
- its HUD elements show through shared slots, one per anchor, in their order;
- its screens open in one shared screen;
- its keys take the slots Personal Key 1 to 8 in the order they are registered (Shift+Alt+1 to
  8 unless rebound in Controls), not their own default bindings;
- it has no server half, channels or components; its lines come from its own `assets/lang`.

## Limits

| What | Limit |
|---|---|
| Fuel (about one WebAssembly instruction each) | `init` 10⁹, a command 2·10⁸, an event batch 5·10⁷, a world tick 10⁷, a question or chain step 2·10⁶ |
| Wall time per call | 500 ms (a backstop; fuel ends calls long before) |
| Memory | 64 MiB |
| Component size | 16 MiB (`code/server.wasm`) |
| Per call | 64 log lines, 32 tells, 8 achievements, 32 replies, 4096 block changes (`set_blocks` included), 64 spawns, 64 sounds, 64 particle bursts of at most 256, 256 scheduled ticks (up to an hour ahead), 32 sends of 16 KiB; `near` within 64 blocks, 256 entities |
| Storage | 1024 keys, 4 MiB in all, 1 MiB written per call, keys of `a-z 0-9 _ - .` |
| Registrations | 256 hooks, 64 components, 256 attaches; commands of `a-z 0-9 _`, not the engine's (`/op`, `/give`, `/tp`, ...); a name another mod or package already has is left out (a warning in the log) and the world still opens |

A call that runs out of fuel, traps (a panic is a trap) or grows its memory past the cap is a
**strike**: the mod's memory is dropped and it starts over with `init` (its storage stays). Three
strikes within ten minutes stop it in that world until the world is opened again. A question
asked of a failing mod abstains; a chain step leaves the value as it was.

Floats are deterministic (NaNs canonicalized, no relaxed SIMD) and there are no threads.

## Where and when it runs

- On the world's host only: players who join download the package (and with it the code) so
  their content matches, but their games never call it. The join sheet tells them which packages
  carry code ("runs sandboxed code"); a host that doesn't say so truly is refused.
- Once per world: each world (each run of it: opening it again starts afresh) has its own
  instance, memory and storage (`<world>/mods/packages/<package>/`). A player walking through a
  portal into another world meets that world's mods, never this one's.
- `init` runs once when the package loads, with no world, to learn what the mod registers, and
  again in each world on its first call there. Register the same things every time; a mod whose
  registrations differ is stopped in that world.
- A mod's own call into the world can ask a question or run a chain the mod itself hooks; that
  nested call skips the mod instead of re-entering it.

## Seeing what runs: `/mods`

`/mods` lists the sandboxed server halves running in the world you are in: each one's API
version, its calls so far, their average and slowest time, and its strikes in the last ten
minutes, and why one has stopped. An admin starts a stopped one again with `/mods resume
<package>` (its strikes forgotten, its memory new, its storage kept).

`/client_mods` is the same for the client halves running in your own game, packages' and
personal ones, by frame: each one's time a frame on average and in the worst frame of the last
240 (about four seconds), its calls a frame, its slowest call, its strikes, and then all of them
together. Every call into a half is timed (starting it again after a strike included): HUD
builds, bindings, keys, packets and its frame hook.

## Versions

API versions are frozen once released. `crates/sandbox/wit/v1.0/` and `v1.1/` never change
after that. A 1.x version only adds calls, so the game binds the newest it has, and a component
built for an older 1.x links unchanged (wasmtime matches 1.x imports and exports by semver). A
component is refused when it loads, with the reason in the log, if it imports anything but this
API (WASI, say) or a newer 1.x than the game runs. The host's offer names the API each package's
code needs, so a game too old for it refuses on joining, before downloading anything; the join
sheet shows the version ("runs sandboxed code (mod API 1.1)"). A change that removes or alters a call would
be 2.0, which nothing plans.

The SDK builds for the newest version (1.3): a mod built with it needs a game that runs 1.3.
1.3 added one call, `world.grant-achievement` (phase 4); the client SDK stays on 1.2, whose
client world 1.3 didn't change.
`examples/packages/clans` is built for 1.0, `tally` for 1.1, `bell` for 1.2. The callbacks 1.2
added are a second export of the component, `content-hooks`, which the game looks for at run
time, so a component built for 1.0 or 1.1, which lacks it, still runs.
