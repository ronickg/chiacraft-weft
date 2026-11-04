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

## What a mod can do (API 1.1)

The contract is `crates/sandbox/wit/v1.1/mod.wit` (1.0 and the calls 1.1 added). The SDK gives
it Weft's names:

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
| content builders, block and item callbacks, screens, HUD, keys, channels, client hooks | not yet: content comes from the package's definitions; block and item callbacks and channels are planned for 1.2, client halves after |

Ours, beyond Weft's: `w.tell(player, key, args)` (a line in a player's chat, in their language),
`w.player_key(player)` (who they are, the same in every world), `w.teleport(id, pos)`, and
`storage::{get, get_text, set, remove, keys}` (the mod's own data in this world's save).

Every line a mod shows players is a translation key and arguments, worded by each player's game
from the package's `assets/lang` files, like Weft's command replies.

## Limits

| What | Limit |
|---|---|
| Fuel (about one WebAssembly instruction each) | `init` 10⁹, a command 2·10⁸, an event batch 5·10⁷, a world tick 10⁷, a question or chain step 2·10⁶ |
| Wall time per call | 500 ms (a backstop; fuel ends calls long before) |
| Memory | 64 MiB |
| Component size | 16 MiB (`code/server.wasm`) |
| Per call | 64 log lines, 32 tells, 32 replies, 4096 block changes (`set_blocks` included), 64 spawns, 64 sounds, 64 particle bursts of at most 256; `near` within 64 blocks, 256 entities |
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

## Versions

API versions are frozen once released. `crates/sandbox/wit/v1.0/` and `v1.1/` never change
after that. A 1.x version only adds calls, so the game binds the newest it has, and a component
built for an older 1.x links unchanged (wasmtime matches 1.x imports and exports by semver). A
component is refused when it loads, with the reason in the log, if it imports anything but this
API (WASI, say) or a newer 1.x than the game runs. The host's offer names the API each package's
code needs, so a game too old for it refuses on joining, before downloading anything; the join
sheet shows the version ("runs sandboxed code (mod API 1.1)"). A change that removes or alters a call would
be 2.0, which nothing plans.

The SDK builds for the newest version (1.1): a mod built with it needs a game that runs 1.1.
`examples/packages/clans` is built for 1.0, `examples/packages/tally` for 1.1.
