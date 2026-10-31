# Data packages

A data package adds blocks, items, recipes and tags with files only: no code, nothing to compile.
Worlds will name the packages they need, and players who join download them (chiacraft plan,
phase 2). `packages/gems` is a complete example.

## Where packages go

- `run/packages/<folder>/`: yours.
- `packages/<folder>/` in the game's folder: the game's own and examples.

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
