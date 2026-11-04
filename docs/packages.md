# Data packages

A data package adds blocks, items, recipes and tags with files only: no code, nothing to compile.
Worlds will name the packages they need, and players who join download them (chiacraft plan,
phase 2). `examples/packages/gems` is a complete example; start the game with
`--packages examples/packages` to try it.

## Where packages go

- `run/packages/<folder>/`: yours.
- `packages/<folder>/` in the game's folder: the game's own.
- any folder given with `--packages <folder>`.

A package with mistakes is left out when the game starts; the log says which file and line.

## Layout

```
gems/
  package.toml
  defs/
    blocks/ruby_block.toml      -> block gems:ruby_block (and its item)
    items/ruby.toml             -> item gems:ruby
    recipes/ruby_block.toml     -> a recipe
    tags/gems/red.toml          -> tag gems/red
  assets/
    textures/block/ruby_block.png   (16 x 16)
    textures/item/ruby.png
    models/block/<name>.json        (optional: a Blockbench Java block model, used by the block of that name)
    lang/en_us.lang                 (block.gems.ruby_block = Block of Ruby)
```

A definition's name is its path under `defs/<kind>/`, in lowercase letters, digits, `_` and `/`.
Inside a package a name without a namespace is the package's own: `ruby` means `gems:ruby`. Other
packages' and mods' things are named in full (`base:glowstone`); tags are written `#ores/ruby`.

## package.toml

```toml
format = "1.0"          # the definition format; this game reads 1.x
id = "gems"             # the namespace: lowercase letters, digits, _
version = "0.1.0"
name = "Gems"
description = "Rubies."

[dependencies]          # mods and packages it uses, with versions ("*" for any)
base = "*"
```

## Blocks: `defs/blocks/<name>.toml`

| Field | Meaning | Default |
|---|---|---|
| `texture` | one texture for all faces | `block/<name>` |
| `textures` | `{ top, bottom, side, north, south, east, west }`; `side` fills the sides not named | |
| `hardness` | how long it takes to break (stone 1.5, dirt 0.5) | |
| `unbreakable` | `true` for bedrock-like blocks | `false` |
| `light` | light it gives, 0 to 15 | 0 |
| `sound` | sound group: `stone`, `wood`, `grass`, `gravel`, `sand`, `glass`, `cloth` (base's), or `mod:group` | |
| `mineable` | tool class that mines it fast: `pickaxe`, `axe`, `shovel` | |
| `tool_level` | tool level needed for it to drop anything | |
| `drops` | `"self"`, `"nothing"`, an item, or `{ item = "...", count = 2 }` | `"self"` |
| `render` | `"solid"`, `"cutout"` (holes, like leaves), `"translucent"` (see-through) | `"solid"` |
| `solid` | `false` for blocks you walk through | `true` |
| `climbable`, `friction` | ladders; ice-like sliding | |
| `tags` | tags it gets | |
| `item` | whether it has an item that places it | `true` |
| `item_texture` | that item's texture | the block's |
| `tab` | creative tab, e.g. `base:building` | |

## Items: `defs/items/<name>.toml`

`texture` (default `item/<name>`), `max_stack`, `durability`, `tool` (`pickaxe`, `axe`, `shovel`,
`sword`), `tool_level`, `mining_speed`, `attack_damage`, `fuel` (ticks in a furnace), `places` (a
block), `tags`, `tab`.

## Recipes: `defs/recipes/<name>.toml`

Shaped, in a crafting table:

```toml
pattern = [" R ", "RGR", " R "]
key = { R = "ruby", G = "base:glowstone" }
result = "ruby_lamp"
count = 1
```

Shapeless (`inputs`), or in a furnace with `kind = "base:smelting"` and `time` in ticks:

```toml
kind = "base:smelting"
inputs = ["raw_ruby"]
result = "ruby"
time = 200
```

## Tags: `defs/tags/<tag>.toml`

```toml
entries = ["ruby", "ruby_block", "#other_tag"]
```

## What a package may hold

A package holds only files the game reads, each kind within a size. Anything else (a `README.md`,
a script, an archive, a texture in another format) and the package is left out with the reason in
the log; a joining game refuses such a package before downloading any of it.

| Where | Files | Each at most |
|---|---|---|
| top level | `package.toml` | 64 KiB |
| `defs/blocks/`, `defs/items/`, `defs/recipes/`, `defs/tags/` | `.toml` | 256 KiB |
| `assets/lang/` | `.lang` | 1 MiB |
| `assets/textures/` | `.png` (at most 4096 pixels a side) | 16 MiB |
| `assets/textures/` | `.anim`, `.ctm`, `.gui` | 64 KiB |
| `assets/models/` | `.json` | 1 MiB |
| `assets/sounds/` | `.ogg`, `.wav` | 64 MiB |

A package has at most 4096 files and 256 MiB in all, and at most 4096 definitions. A texture
larger than 4096 pixels on a side shows as the missing texture. `package.sig` (see below) and hidden
files are not part of the package.

## How a package is named

A package is named by a hash of its exact files: the BLAKE3 hash of a manifest that lists every
file's path, size and BLAKE3 hash (`crates/packages/src/identity.rs`). The same files give the same
hash on every computer, however they were copied or zipped; changing any file gives a new one. The
game shows its first 12 hex digits (`gems 0.1.0 (7a73bcf1cf25)` in the log).

On start each package is copied into the store in `run/cache` (each file once, by hash), checked
against its hash, and loaded from there, the same way packages downloaded from other players will be.
File names may use letters, digits, `_`, `-` and `.`; hidden files (starting with a dot) are left out.

The store keeps every package a world on this computer pins and every installed package. Other
packages (those downloaded for friends' worlds) are kept up to 512 MiB, the most recently used
first; on start the game removes the rest, and any file no kept package needs.

## Worlds remember their packages

A world's `world.txt` lists the exact package it was played with: `package.gems = 7a73bc…`, and plays
with exactly that version from then on, even after you install a newer one: two worlds can use two
versions of the same package, and switching between them needs no restart (the game builds its
content again for the other world's packages). A package a world has not met yet (newly installed)
joins it the next time it is opened. Opening a world whose package version is in neither the package
folders nor the store asks first. Blocks of a missing package show as
unknown blocks (magenta and black) and keep their real names in the save, so they come back when the
package does.

## Joining a friend's world

When you join, the host offers the packages its world plays with (name, version, hash and size).
Your game downloads the ones it doesn't have from the host, over the same connection, checks every
file and the package's manifest against their hashes (and the sizes the host declared), keeps them in
the store, loads exactly the host's set, and then shows the host the hash of the content it loaded:
only a match gets in. The host serves nothing but the packages its world uses. Code mods are not
downloaded: both games must have the same ones.

Limits keep a join from hurting either side. A world offers at most 256 packages and 1 GiB in all;
a download that sends nothing for 30 seconds has stalled. The host serves a player each offered
package about twice at most, four requests at a time, and lets eight players download at once
(others are asked to try again in a minute); a player has 15 minutes to download and load the
world's packages.

## Signing a package

```sh
cargo run --release -- --sign-package path/to/mypackage
```

signs the package with your account's content key and writes `package.sig` beside its files. The
content key is derived from your 24 words on its own path (it is neither your identity key nor
your network key), and your account's identity key vouches for it, so a signature says which
account made the package, not just a name anyone could copy. `package.sig` is not one of the files
the package's hash covers: signing (or adding other signatures later) never changes the package or
any world's pin. Change a file and you need to sign again.

## Who you trust

When a world needs packages you don't have, the game asks first, showing each package's size and who
made it (the first digits of the creator's content key and account), or that it is unsigned, or
that its signature is refused. "Download and Join" trusts those exact packages; "Download, Trust
These Creators" also trusts their creators, so later packages and versions from them download
without asking. Nothing is asked about packages you already have or trust. What you accept is kept
in `run/trust.txt`.
