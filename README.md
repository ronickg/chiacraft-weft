# chiacraft

chiacraft is a survival block game built on [Weft](https://github.com/Weft-Rust/Weft), a voxel sandbox engine written in Rust by [VortexSo4](https://github.com/VortexSo4). This repository is a fork of Weft: the engine in `crates/` is Weft's, and the game is our mods on top of it.

> chiacraft is not an official Weft project and isn't endorsed by Weft's author. For Weft itself (its releases, Discord and issue tracker), go to [Weft's repository](https://github.com/Weft-Rust/Weft). Please don't report chiacraft bugs there.

> Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft. chiacraft contains no Minecraft code or assets.

## What chiacraft adds

- **The game** (`mods/base`): survival with temperature, seasons, food and levels; farming and crops; progression through the ages.
- **Animals** (`mods/animals`): species with life cycles, eggs, and their own needs and senses.
- **Portals** (`mods/portals`): link your world to other players' worlds and walk between them.
- **Packages** (`mods/packages`, `crates/packages`): data packages. Missing mods download on the way, with consent.
- **Accounts** (`crates/account`, `crates/wallet`, `crates/bridge`): a player identity (a DID) and achievements on Chia, so no one can take an account away.
- **Sandboxed mods** (`crates/sandbox`).
- **A test kit** (`mods/testkit`): scripted game runs for checks.

The goal and the decisions are in [notes/PLAN.md](notes/PLAN.md). Where everything stands is in [notes/OVERVIEW.md](notes/OVERVIEW.md).

## Staying close to Weft

We change Weft's engine only where our own crates and mods can't do the job. Every change we make to Weft's code is listed with its files and the reason in [notes/ENGINE_CHANGES.md](notes/ENGINE_CHANGES.md). We take Weft's updates regularly. Where Weft has fixed something we fixed ourselves, we take Weft's version.

Weft's own README, as of the last sync, is kept at [docs/weft-README.md](docs/weft-README.md). The engine docs in [`docs/`](docs) are Weft's, plus a few of ours ([packages](docs/packages.md), [sandboxed mods](docs/sandboxed-mods.md), [release builds](docs/release-builds.md)).

## Build from source

You need Rust 1.95 or newer and a GPU with Vulkan. On macOS, Vulkan runs through MoltenVK.

```sh
cargo run --release
```

Run it from the repository root. Release builds pack each mod in `mods/` into `run/mods/` when the game starts; settings, saves and logs go to `run/`. To make packaged builds for each platform, see [docs/release-builds.md](docs/release-builds.md) and `tools/release/`.

## Modding

Modding works as it does in Weft. Start from [`examples/examplemod`](examples/examplemod) and the [modding guide](docs/modding.md). `mods/base` is built only on the public SDK, like Weft's own base game.

## Status

Early and private. Saves, the mod interface and the game itself still change often.

## Licence

[MPL-2.0](LICENSE), the same as Weft. Weft is © its authors under MPL-2.0. Files we changed stay under MPL-2.0 with their licence notices kept. Anyone we give a build to can get its source.

Much of chiacraft is written with AI tools (Claude). Weft's [AI policy](AI_POLICY.md) applies to anything sent to Weft. We don't send pull requests there at the moment.
