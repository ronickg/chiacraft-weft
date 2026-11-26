# Weft modding quick start

This page gets you from nothing to a working mod and points you to other pages.\
If you get stuck, see [Getting help and community](#getting-help-and-community).

## Introduction
The base game is a mod itself, and it has no private hooks and no special priority over other mods, so anything the base game can do, your mod can do too.\
When in doubt, look in `mods/base` and `mods/testmod` and copy from there.

## Preparation
1. You need an editor before you start. I recommend JetBrains RustRover: it is free and has everything you need.
2. Clone the repo with the built-in Clone Repository feature, use `https://github.com/Weft-Rust/Weft` as the link. This creates a folder with the code on your computer.
3. Configure a Run Configuration. Find the Run Configuration tab in the top-right corner, set the Cargo command to `run` and name it whatever you like. The game will then launch from your local copy of the repo.

## Creating a mod in two minutes

To get started, copy `examples/examplemod/` to some location on your PC.

It is a ready-made mod with one item, a setting and an event. Every kind of registration is written out there as a commented example, with a pointer to where the base game uses it.

Rename the three things listed at the top of `lib.rs` and you are ready to start modding.

To run the mod:
1. Build the mod with `cargo build --release`
2. Create a folder in `run/mods/`, copy the built library from `target/release/` into it.
3. Symlink `assets/` into it. (e.g. on Linux: `ln -s my-mods/mymod/assets/ weft/run/mods/mymod/assets`).

You should end up with something like that:

```
run/mods/mymod/
    mymod.dll          (libmymod.so / libmymod.dylib might be also there)
    assets/
        textures/block/ruby_ore.png
        textures/item/ruby.png
        lang/en_us.lang
```

If you can't see the mod in-game, double check that the folder name matches the one in `Cargo.toml`.\
If you're stuck, you can always join [our Discord server](https://discord.com/invite/zTEekbrc7U) to get some help.

To send the mod to someone, or to get it into the launcher, see [Packaging and publishing](publishing.md).

## Three ideas to know first

- **Names.** Everything a mod adds is called `namespace:path`. This goes for blocks, items, dimensions, particles or any other stuff. In your own mod you write `ruby` and it becomes `mymod:ruby`, and you reference stuff from other mods by their full name (`base:furnace`). You can also use more complex selectors (`#logs`, `!`, `base:furnace[lit=true]`). See [Names, references and handles](names-and-references.md).
- **Registering.** Registering is how your mod adds content to the game. In your mod's `init` function you get `r`, the registry: call `r.block(..)`, `r.item(..)` and so on to add content. The same `r` also changes things that already exist, no matter what mod added them, the engine included: `r.edit(..)` changes some properties, `r.replace(..)` starts over and `r.remove(..)` deletes the thing entirely. See [Changing something of another mod](details/editing-others.md).
- **Components and tags.** Every property of a block or item, like fuel, food or hardness, is a *component*: a named value such as `fuel = 800`. A *tag* is a label that groups things, like `#sand` or `#logs`. Any mod can read and write both. So instead of checking whether something is `base:coal`, check whether it has `fuel`, and instead of checking for `base:sand`, check for `#sand`. Your code then works with other mods' content too. See [Components and tags](details/components-and-tags.md).

## Where to go next

New to Weft? Read [blocks](blocks.md), then [items](items.md), then [registration](registration.md).

Each guide starts with the common case and goes on to the rare ones. The reference lists every method you can call.

**Content you register**

| I want to make | Guide | Reference |
|---|---|---|
| Blocks | [blocks](blocks.md) | [reference](reference/blocks.md) |
| Blocks with data, inventories, menus, energy | [interactive blocks](block_entities.md) | |
| Items | [items](items.md) | [reference](reference/items.md) |
| Mobs and other entities | [entities](entities.md), [poses and game modes](details/poses-and-modes.md) | [reference](reference/entities.md) |
| Status effects and potions | [effects](details/effects.md) | [reference](reference/entities.md) |
| Liquids | [liquids](details/liquids.md) | |
| Recipes and loot | [recipes and loot](details/recipes-loot.md) | [reference](reference/recipes.md) |
| Biomes, dimensions, ores, structures, spawns | [world content](details/world-content.md), [worldgen](worldgen.md) | [reference](reference/worldgen.md) |
| Redstone-style logic | [redstone](redstone.md) | |
| Every kind of content, with its components | [registration](registration.md) | [reference](reference/registration.md) |

**Making things happen**

| I want to | Guide | Reference |
|---|---|---|
| Hook into game events,<br>change damage or an attribute | [events](events.md), [attributes](details/attributes.md) | [reference](reference/events.md) |
| Test conditions | [conditions](details/conditions.md) | |
| Add commands | [commands](details/commands.md) | [reference](reference/commands.md) |
| Work with the world from code | [world API](world-api.md), [hitboxes](details/hitboxes.md), [world blocks](details/world-blocks.md), [world entities](details/world-entities.md) | [reference](reference/world.md) |
| Read or post-process generation | [reading generation](worldgen.md#reading-generation) | |
| Change another mod's content | [editing others](details/editing-others.md) | |

**Client side and looks**

| I want to | Guide | Reference |
|---|---|---|
| Keybinds, packets, settings,<br>code that runs every frame | [client](client.md) | [reference](reference/client.md) |
| Screens and the HUD | [UI](ui.md) | [reference](reference/ui.md) |
| Camera modes | [camera](details/camera.md) | |
| Models and animations | [models](models.md) | |
| Particles and sounds | [particles and sounds](details/particles-sounds.md) | [reference](reference/effects.md) |
| Fog, sky lighting, post effects, acoustics | [environment](environment.md), [sky](sky.md) | |
| Shaders | [shaders](shaders.md) | |
| Textures, connected textures | [textures](textures.md), [connected textures](ctm.md) | |
| Translations and rich text | [localization](localization.md), [text on screen](details/text.md) | |
| A resource pack | [resource packs](resourcepacks.md) | |

**Shipping and tooling**

- [Packaging and publishing](publishing.md): zips, all platforms, the launcher, versions
- [Debugging](debugging.md): logs, gizmos, reloads, crash reports
- [Content editor methods](editor.md)

## Command levels (chiacraft)

Every chat command asks for a permission level, Minecraft's 0-4: **0** any player, **1** moderator (gets past protections, joins full worlds), **2** game master (changes the game: `/give`, `/tp`, `/time`, `/weather`, `/gamemode`, `/tk`, every debug command), **3** admin (rules, ticks, operators, bans, admission), **4** the host.

- **One gate.** The level is declared when the command registers (`r.command(..).permission(permission::EVERYONE)`, `r.client_command(..)` the same), and the engine checks it where commands are dispatched: `Server::command` for server commands, `client_command_lines` for client ones (against the level the server gave the player). A mod never checks it itself, so it can't forget. A command that says nothing asks for **2**, client commands too (Weft's client commands asked for 0). A branch (`command_branch`) has its command's level.
- **Refused:** the player reads `command.no_permission` in their language (engine en, ru, fr) and isn't offered the command by Tab or `/help`; the host's log has `refused /<cmd> for <name>: it asks for level N, they have M`.
- **Who has which level.** On a world made with cheats on, the player on the hosting machine (single-player, a LAN or linked world hosted from a game) is **4**, like Minecraft's "allow cheats"; without cheats, 0. **The owner's moderation exception:** even without cheats the world's owner (the hosting player, or the account its certificate names) may use the ban and admission commands (`/ban`, `/tempban`, `/unban`, `/bans`, `/allow`, `/probation`, `/revoke`, `/achievementrule`, `/didrule`, `/banlist`; `server::OWNER_MODERATION`), so a host can always put out a troublemaker, but not `/op`, `/rule`, `/tick` or any level-2 command. It is an exception on those commands, not a level: mods asking `w.permission(player)` still read 0, and `/help` lists them. There is no `/kick`: `/tempban` does it. Tab offers them to the player whose world runs in their own game (single-player, LAN, hosted from a game); a certificate's owner joining a dedicated server isn't offered them but may still type them. Everyone else has the level saved for their key in the world's `ops` (`/op <who> [level]`, `/deop`; the dedicated server's console `op`), else 0.
- **Testkit.** A script's own player is the host, so 4. In runs of several games in one world the host adds `--config testkit:op_joiners=true` and every joining player is made 4 (or the host's script runs `chat /op <name> 4`).
- **Packages** (sandboxed wasm, `docs/sandboxed-mods.md`): a package's command is registered through the same `r.command(..).permission(level)` with the level its code declared (2 unless it says otherwise), so it goes through the same gate. A package can't change anyone's level.
- **A mod's finer checks** (`w.permission(player)`), for parts of a command or things that aren't commands: `/mods resume` (3), `/portalfriends` (the world's owner only), a game master's extras at ruins and waystones.

| level | commands |
|---|---|
| 0 | engine: `/help`, `/msg`, `/log`, `/music`. base: `/sethome`, `/home`, `/delhome`, `/homes`, `/back`, `/inspect`, `/skills`, `/warmth`, `/rename`, `/videocosts`, `/developer_menu` (client: the player's own look settings). animals: `/tribe`, `/tribeinvite`. portals: `/portalfriends`, `/portal_note` (the mod's own). packages: `/mods` (listing), `/client_mods`, `/packages_say` (the mod's own). |
| 2 | engine: `/tp`, `/dim`, `/give`, `/time`, `/summon`, `/kill`, `/clear`, `/setblock`, `/insert`, `/block`, `/gamemode`, `/heal`, `/effect`, `/regions`, `/seed`, `/weather`, `/title`, `/say`, `/tellraw`, `/structure`, `/gizmo`, `/sounds`. base: `/advancements`, `/attribute`, `/fireflies`, `/litter`, `/growth`, `/impact`, `/knap`, `/xp`, `/ley`, `/gear`, `/ruin`, `/skill`, `/stone`, `/socket`, `/perk`, `/feed`, `/season`, `/spawncar`, `/waterholes`, `/waystone`, `/air`, `/rain`, `/windleaf`, `/ambience`, `/rot`. animals: `/anim`, `/boss`, `/drenkaun`, `/journal`, `/animalprof`, `/animal`, `/ambient`. testkit: `/tk`, `/regions`, `/compare`, `/probe`, `/tk_freeze`, `/gizmo probe`. testmod: all. |
| 3 | engine: `/tick`, `/op`, `/deop`, `/rule`, `/ban`, `/tempban`, `/unban`, `/bans`, `/allow`, `/probation`, `/revoke`, `/achievementrule`, `/didrule`, `/banlist`. |
| 4 | the server console (`stop`, `list`, `say`, `op`, `deop`), only on the hosting machine. |

## Getting help and community

The [Discord server](https://discord.com/invite/zTEekbrc7U) is the best place to:

- ask questions and get help with your mod
- see what other people are working on and share your own
- see sneak peeks for upcoming features and suggest ideas

If you had to work around the engine to do something reasonable, that is an engine flaw. Tell me about it instead of building a hack, so I can adjust the engine for those cases.\
You can open an issue, tell us about it on the Discord server, or contact me directly on Discord (`@vortex_so`).
