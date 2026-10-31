# Data-only content packages in moddable games: systems and definition formats

Scope: how moddable (mostly voxel) games define content as data-only packages, and which file formats and schema practices work best. Written for a Rust voxel game (Weft fork) planning TOML defs + `assets/` + `package.toml` (id/version/creator/dependencies). Research date: 2026-10-31. Facts are from 2025-2026 sources unless marked otherwise.

## Minecraft data packs and resource packs (structure, versioning, namespacing, data components, data-driven registries, format evolution)

### Takeaway
Minecraft is the main reference design: a `pack.mcmeta` manifest, one namespaced folder tree (`data/<ns>/<registry>/<path>.json` becomes the id `<ns>:<path>`), last-loaded-wins overrides, tags that merge, and a version number that changes often. Since 1.21 Mojang has moved more and more content into data-driven registries (enchantments, paintings, mob variants, damage types, dimension types, villager trades, dialogs, and more). It accepts frequent breaking changes and softens them with three tools: a numeric format version, range declarations (`min_format`/`max_format`), and overlays (per-version subfolders). Packs are not migrated automatically; pack authors absorb the breakage.

### Cited Findings
**Structure and namespacing**
- `pack.mcmeta` is the only mandatory file. `pack.png` is optional. Content lives under `data/<namespace>/`. The `minecraft` namespace holds vanilla files, and a pack can override them by using that namespace — [Data pack, minecraft.wiki](https://minecraft.wiki/w/Data_pack)
- Files at `data/<namespace>/<registry>/<path>.json` load as `<namespace>:<path>`. Functions use `.mcfunction` and structures use `.nbt` — [Data pack](https://minecraft.wiki/w/Data_pack)
- Tags live in `data/<namespace>/tags/<registry>/` and are referenced as `#<namespace>:<path>`. A tag without `"replace": true` merges with the same tag from earlier packs; with it, the tag replaces them — [Data pack](https://minecraft.wiki/w/Data_pack)
- Overrides: when several packs contain the same file, the last one loaded wins. Tags are the exception, since they merge. Load order is set on the Data Packs screen or with `/datapack` and is saved in `level.dat`, so the pack list belongs to the world — [Data pack](https://minecraft.wiki/w/Data_pack)
- A `filter` section in `pack.mcmeta` (since 1.19, format 10) blocks files from lower-priority packs, which is the only way to delete content rather than replace it — [Data pack](https://minecraft.wiki/w/Data_pack)
- Data packs cover gameplay logic (structures, recipes, tags, loot tables, functions). Resource packs cover the client side (textures, models, blockstates, sounds, text). Both use the same versioning system — [Pack format](https://minecraft.wiki/w/Pack_format)

**Data-driven registries (current list)**
- Standard registries: `advancement`, `function`, `item_modifier`, `loot_table`, `predicate`, `recipe`, `structure`, `tags`, plus newer `context_float_provider`, `context_int_provider` and `slot_source` — [Data pack](https://minecraft.wiki/w/Data_pack)
- "Dynamic"/experimental-flagged registries include `banner_pattern`, `cat_variant`, `chat_type`, `chicken_variant`, `cow_variant`, `damage_type`, `decorated_pot_pattern`, `dialog`, `dimension`, `dimension_type`, `enchantment`, `enchantment_provider`, `frog_variant`, `instrument`, `jukebox_song`, `painting_variant`, `pig_variant`, `test_environment`, `test_instance`, `timeline`, `trade_set`, `trial_spawner`, `trim_material`, `trim_pattern`, `villager_trade`, `wolf_sound_variant`, `wolf_variant` and `world_clock`, plus the whole `worldgen/*` tree (biome, configured_feature, density_function, noise_settings, structure_set, template_pool, world_preset, …). The wiki says a valid file in one of these registries marks the pack as using experimental settings. Its own history section also says trims stopped being experimental in 1.20, so the "experimental" marking is inconsistent on the page itself — [Data pack](https://minecraft.wiki/w/Data_pack)
- Reloading: `/reload` hot-applies tags, loot tables, recipes, advancements, item modifiers, predicates, functions and structures. Dynamic registries (enchantments, variants, dimension types, worldgen) need a world or server restart — [Data pack](https://minecraft.wiki/w/Data_pack)
- Timeline of what became data-driven: 1.21 (format 48) enchantments and paintings; 1.21.5 (format 71) pig, cow and chicken variants plus GameTest; 1.21.6 (format 80) dialogs; 1.21.11 (format 94.1) timelines and slot sources; 26.1 (format 101.1) villager trades and world clocks; 26.3 (format 121.0) sulfur cube archetypes — [Data pack](https://minecraft.wiki/w/Data_pack)

**Data components (1.20.5+)**
- Format 41 (1.20.5) replaced unstructured item-stack NBT with structured components — [Pack format](https://minecraft.wiki/w/Pack_format)
- Components are namespaced, structured data that store item information and define behavior. A stack is saved as `id`, `count` and an optional `components` map that holds only the components that differ from the item type's defaults — [Data component format](https://minecraft.wiki/w/Data_component_format)
- Each item type has default components (a prototype). Stacks inherit them and only overrides are saved. `!component` removes a default, for example `item_id[!component]`. Some properties are still hardcoded to the item id and cannot be removed or moved to another item — [Data component format](https://minecraft.wiki/w/Data_component_format)
- Behavior comes from components being present: `consumable` and `equippable` give an item that behavior when attached. `item_model` points to an item model definition, so any item can look like any other (e.g. `netherite_sword[item_model="minecraft:diamond_sword"]`). `custom_data` is free-form storage — [Data component format](https://minecraft.wiki/w/Data_component_format)
- The same component structure is written as SNBT in commands (`item[comp=value]`) and as JSON objects in data files — [Data component format](https://minecraft.wiki/w/Data_component_format)

**Versioning and format evolution**
- `pack_format` is a number in `pack.mcmeta` that says which Java versions a pack works with. A pack with a higher number than the game supports shows as "incompatible" — [Pack format](https://minecraft.wiki/w/Pack_format)
- Snapshot 25w31a (released in 1.21.9) replaced the single `pack_format` with `min_format` and `max_format`, and added minor versions: non-breaking changes bump the minor number (e.g. 82.0 → 82.1) instead of the major. Formats can be written as `[major, minor]` arrays, e.g. `[88, 0]` — [Pack format](https://minecraft.wiki/w/Pack_format); [Data pack](https://minecraft.wiki/w/Data_pack)
- A single integer is read as "any minor version". The old fields are still required when a pack declares support below data pack format 82 or resource pack format 65 — [Pack format mirror (2025-11)](https://wiki.ronlab.site/content/minecraftwiki_en_all_maxi_2025-11/Pack_format); [SchemaStore pack.mcmeta schema](https://www.schemastore.org/minecraft-pack-mcmeta.json)
- `supported_formats` and `overlays` arrived in 23w31a (data packs, format 16) and 1.20.2 (resource packs, format 18). Overlays are subdirectories applied over the base content, so one pack can target several versions — [Pack format](https://minecraft.wiki/w/Pack_format)
- A Mojira report says the overlay key `formats` is deprecated from pack format 82 and replaced by per-overlay `min_format`/`max_format` — [MC-302593](https://mojira.dev/MC-302593)
- Latest formats: data pack 121.0 and resource pack 97.1 (26.3); 26.2 used data 107.1 and resource 88.0 — [Pack format](https://minecraft.wiki/w/Pack_format)
- Examples of breaking changes: 1.21 (format 48) renamed plural folders to singular (`loot_tables` → `loot_table`, `tags/items` → `tags/item`); 1.20.5 (41) brought item components; 1.20 (15) moved sign text NBT (`Text1` → `front_text.messages[0]`); 1.21.2 (57) and 1.21.4 (61) changed the component and loot formats again — [Pack format](https://minecraft.wiki/w/Pack_format); [Data pack](https://minecraft.wiki/w/Data_pack)
- Tooling friction: one pack author reported a "broken or incompatible" warning in 25w31a even though the pack worked — [Modrinth silly-eatables changelog](https://modrinth.com/datapack/silly-eatables/version/v3.3.0-beta2)

**Validation behavior**
- Malformed entries (e.g. a non-existent entry in a vanilla tag) cause an error when the world opens. The player can choose Safe Mode (vanilla only) or go back. Servers have `--safeMode` — [Data pack](https://minecraft.wiki/w/Data_pack)
- Since 20w22a, a failed `/reload` keeps the previous data, and the pack list changes only after a successful reload (transactional reload) — [Data pack](https://minecraft.wiki/w/Data_pack)
- SchemaStore publishes a JSON Schema for `pack.mcmeta`, which gives editors validation for free — [SchemaStore](https://www.schemastore.org/minecraft-pack-mcmeta.json)

### Inferences
- Patterns worth copying: (1) `<ns>:<path>` ids taken from the file path, so the id is never repeated inside the file; (2) one folder per registry; (3) tags as merge-by-default sets with an explicit `replace`; (4) per-world pack list and order; (5) transactional reload; (6) a separate filter or remove mechanism for deletions.
- The component model (type defaults + per-instance diff + `!remove`) fits a Rust ECS-style design well: an item def is a bag of typed optional components, each validated by its own schema. That beats one huge `ItemDef` struct with dozens of optional fields.
- Mojang's lesson: a format version will be needed from day one. Use `major.minor` (minor = additive, still compatible), and let packages declare a range. Overlays are a heavy tool; a small game can more simply support N-1 formats with an in-engine migration step.
- Minecraft does NOT migrate old packs automatically (no pack DataFixer is documented on these pages). Every format bump breaks pack authors, which is a big ecosystem cost. A smaller game can do better by shipping load-time upgraders for old `format` values.

### Gaps
- Could not confirm from the fetched pages exactly which component/registry changes came in 1.21.2 vs 1.21.4 (the data component page did not cover version history in the part read).
- No official minecraft.net changelog for 25w31a was retrieved; details come from the wiki, a wiki mirror, SchemaStore and Mojira.

## Luanti (Minetest): Lua-defined content, games/mods/texture packs, data-only efforts

### Takeaway
In Luanti, all content (nodes, items, crafts) is code: Lua `init.lua` scripts that run on the server and call registration functions. Only metadata (`mod.conf`, `game.conf`) and media (textures, sounds, models) are data. Definitions and media are sent to clients automatically, so clients never install mods. This is easy to start with, but the content cannot be statically analyzed, validated or patched as data.

### Cited Findings
- A mod is a "self-contained bunch of scripts, textures and other related things". Mods run only on the server, and definitions and media are sent to clients automatically — [Luanti Lua API](https://api.luanti.org/)
- Mods load at server startup by running `init.lua` in a shared environment. The `core` namespace replaces `minetest`, which stays as an alias — [Luanti Lua API](https://api.luanti.org/)
- `mod.conf` is a Settings (key=value) file with `name`, `title`, `description`, `depends` (comma-separated, must load first), `optional_depends`, `author` (ContentDB username), `release` (set by ContentDB only) and `textdomain`. Dependencies carry no version constraints; only load order is guaranteed — [Luanti mods docs](https://api.luanti.org/mods/)
- Mods load from `games/<gameid>/mods/`, `mods/` and `worlds/<world>/worldmods/`. A world can bundle its own game in `<world>/game/`, and then nothing outside it is loaded — [Luanti mods docs](https://api.luanti.org/mods/)
- Modpacks are folders with `modpack.conf`. A mod can replace a media file from a mod it depends on by shipping a file with the same name (the docs discourage this) — [Luanti mods docs](https://api.luanti.org/mods/)

### Inferences
- Lessons for Weft: (a) server-authoritative definitions pushed to clients at join time is a good model, and it matches Hytale; (b) replacing media by filename is fragile, so namespaced asset ids are better; (c) `depends` without versions has been workable at Luanti's scale, but it gives no protection against breaking updates; (d) world-bundled content (`worldmods`, `<world>/game`) is the closest analogue to "worlds require content packages".
- (Background knowledge, not verified in this session) Luanti registration takes Lua tables (`core.register_node(name, def)`, `core.register_craft{...}`), so the definitions are table-shaped data inside code. A Weft design could take the same field shapes as pure TOML.

### Gaps
- Found no official or widely used data-only (JSON/TOML) content format for Luanti; any such efforts would be community libraries and were not verified.
- `game.conf` fields and texture-pack override rules were not on the pages fetched.

## Vintage Story: JSON asset system, variants, JSON patching, content vs code mods

### Takeaway
Vintage Story is the strongest proof that a voxel game can define almost all content as data. Blocks, items, entities and recipes are JSON5-style "assets" under `assets/<domain>/`. A compact variant system (`variantgroups` + `*ByType` + wildcards + `{var}` substitution) produces hundreds of variants from one file, and a JSON-Patch-based system lets one mod edit another mod's or vanilla assets. Its known pain points are useful warnings: first-match-wins wildcard ordering, patches that silently overwrite after updates (`add` vs `addmerge`), paths that break when the target file changes, and no tool for debugging patches that apply but do nothing.

### Cited Findings
**Variants**
- `variantgroups` is a list of `{code, states}`. Each state is appended to the object code (e.g. `hammer-{metal}`). Groups multiply by default (every combination); `SelectiveMultiply` with `onVariant` limits a child group to one parent variant — [Modding:Variants](https://wiki.vintagestory.at/Modding:Variants)
- States can come from shared "world properties" files via `loadFromProperties`, so lists like metals or woods are defined once and reused across mods — [Modding:Variants](https://wiki.vintagestory.at/Modding:Variants)
- Adding `ByType` to any property (e.g. `textureByType`, `durabilitybytype`) turns it into a map keyed by variant code with `*` wildcards; keys are checked top to bottom and the first match wins. `{groupcode}` inside strings is replaced by the state — [Modding:Variants](https://wiki.vintagestory.at/Modding:Variants)
- `skipVariants` and `allowedVariants` prune combinations, and skip wins over allow. The wiki contradicts itself on whether wildcards work in these lists — [Modding:Variants](https://wiki.vintagestory.at/Modding:Variants)

**JSON patching**
- Patches live in `assets/<domain>/patches/*.json` as arrays of `{file, op, path, value}`. `file` names the target with its domain prefix (`game:` for vanilla, or another mod's domain). `path` is JSON-Pointer style (`/a/0/b`, `-` appends) — [Modding:JSON Patching](https://wiki.vintagestory.at/Modding:JSON_Patching)
- Operations: `add`, `addmerge` (added 1.19.4: appends to arrays and merges into objects; the wiki recommends it), `addeach`, `replace`, `remove`, `move`, `copy`. `side: "server"|"client"` limits where a patch applies. `"enabled": false` disables an asset — [Modding:JSON Patching](https://wiki.vintagestory.at/Modding:JSON_Patching)
- Caveats on the wiki: patches using `add` silently replaced vanilla arrays after updates; numeric-index paths break when the target file changes; every intermediate key must exist; patching a patch does not work (conflicts need a separate Compatibility Library); errors log the patch index, file and failing step, but silent no-op patches have no debugging tool — [Modding:JSON Patching](https://wiki.vintagestory.at/Modding:JSON_Patching)

**Manifest and dependencies**
- `modinfo.json` has `type` (Theme / Content / Code), `modId` (lowercase alphanumeric only), `name`, `version`, `dependencies` (map of modid → version string), `side` (Server/Client/Universal), `requiredOnClient` and `requiredOnServer` — [Modding:Modinfo](https://wiki.vintagestory.at/Modding:Modinfo)
- Versions are compared as partial SemVer: among prerelease tags only rc > pre > dev is recognized, build metadata is unsupported, and `""` or `"*"` means any version. The page does not say whether a listed version is a minimum or an exact match — [Modding:Modinfo](https://wiki.vintagestory.at/Modding:Modinfo)

### Inferences
- Variants pay off most in voxel games (wood × color × shape). Adopt a template mechanism, but avoid first-match-wins wildcard maps; most-specific-match, or an explicit error on ambiguous overlaps, is easier to debug.
- For patching, prefer patches by id and semantic operation (e.g. "add ingredient to recipe X", "append to tag", "set field") over raw JSON-Pointer array indices, which break when upstream changes. If raw paths are offered, make merge the default and report patches that change nothing.
- The `requiredOnClient`/`side` flags matter because data packages carry both server data (recipes, loot) and client assets (models, textures, sounds). Weft should mark per-file or per-folder side so a server can stream only client-needed assets.
- Use real semver with explicit range syntax (e.g. cargo-style `^1.2`, `>=1.2,<2`) to avoid VS's "minimum or exact?" ambiguity.

### Gaps
- Could not fetch a source measuring how well VS's JSON scales in practice (load times, total asset counts); the wiki is silent. The `conditions`/`dependsOn` fields on patches were not on the page read.
- VS files are JSON5-ish (unquoted keys, comments). Recalled from background knowledge, not confirmed on a fetched page in this session.

## Hytale (Early Access since January 2026): data assets, asset packs, delivery to clients

### Takeaway
Hytale (Early Access launched 13 January 2026) uses server-side modding: Java plugins plus JSON "asset packs" that live only on the server. Players join modded servers without installing anything because the server sends the assets. Assets are JSON split into `Server/` (definitions such as `Server/Item/Items/*.json`) and `Common/` (client-needed art: models, textures, icons). Definitions inherit from a `Parent` asset, where child fields override and some fields merge. Packs are described by a `manifest.json` with Group, Name, Version and Dependencies; ids are namespaced `group:name`. An in-game Asset Editor creates packs. Official docs are still incomplete and parts of the format are expected to change during Early Access.

### Cited Findings
- Early access launched 13 January 2026 — [Hytale, Wikipedia](https://en.wikipedia.org/wiki/Hytale)
- Every asset pack needs a `manifest.json`. The official example includes `Group`, `Name`, `Version`, `Description`, `Authors` (array of `{Name, Email, Url}`), `Website`, `Dependencies`, `OptionalDependencies`, `LoadBefore`, `DisabledByDefault`, `IncludesAssetPack` and `SubPlugins` — [docs.hytale.com: Asset packs](https://docs.hytale.com/creating-content/asset-packs/)
- Assets in a pack override the base-game asset with the same identity; new assets are added the same way. Packs "exist entirely on the server", and only the server or world host downloads them. Packs made with the in-game Asset Editor are stored in the world's `mods` folder and can be copied to the global Mods folder; installed packs appear in the New World options — [docs.hytale.com: Asset packs](https://docs.hytale.com/creating-content/asset-packs/)
- Item definitions live at `Server/Item/Items/*.json`. With inheritance, "the child field will simply replace the value provided by the parent", nested structures follow the same rule, and some fields merge parent and child instead — [Hytale pre-release docs: Items](https://pre-release.docs.hytale.com/assets/item/items/)
- Item top-level fields include `Parent`, `Tags`, `Icon`, `Categories`, `MaxStack`, `Quality`, `MaxDurability`, `Model`, `Texture`, `Animation`, `BlockType`, `Tool`, `Weapon`, `Armor`, `Consumable`, `Interactions`, `Recipe` (typed as an inline `CraftingRecipe`), `ResourceTypes`, `Light`, `SoundEventId` and others. This is a component-like layout where optional sub-objects add behavior — [Hytale pre-release docs: Items](https://pre-release.docs.hytale.com/assets/item/items/)
- Models and textures are referenced from `Common/Blocks/`, `Common/Items/`, `Common/Icons/`, `Common/BlockTextures/` and similar folders. A community reader of Hytale's `Assets.zip` describes `Parent` as "objects merge recursively, everything else is replaced" and notes BlockTypes can have their own Parent — [search summary of docs.hytale.com asset pages and patodois/hytale-unpack](https://github.com/patodois/hytale-unpack)
- A community server-docs mirror says asset packs are enabled with `IncludesAssetPack: true`, files go under `assets/` in a plugin JAR, ids are namespaced `{group}:{name}`, and assets are decoded by codecs (`BuilderCodec`) into typed classes (`BlockType`, `ItemType`, `ModelAsset`, `SoundEvent`, …) held in an `AssetStore`. The document does not name its source, so treat it as unofficial — [hytale-server docs mirror, 08-asset-system](https://gitea.minidev.space/hytale/hytale-server/raw/branch/main/docs/08-asset-system.md)
- Third-party guides: players join modded servers without client mods because the server streams the needed assets into the client cache during the connection handshake. One says manifests now need a `ServerVersion` field tied to a server build. Guides warn that plugins and packs can break between Early Access updates and that the Asset Editor has gaps (NPCs, worldgen, interactions) — [GameServerKings](https://www.gameserverkings.com/knowledge-base/hytale/how-to-install-hytale-mods/); [Winternode](https://winternode.com/blog/hytale/hytale-mods). The `ServerVersion` claim is NOT in the official manifest example fetched.
- Client installs: copy a Thunderstore package's `mods` contents into the game's UserData `mods` folder (for single-player/hosting); for servers, into the server's `mods` folder — [hytalemodding.dev](https://hytalemodding.dev/en/docs/publishing/thunderstore/package-install/asset-packs)

### Inferences
- Hytale closely matches Weft's goal (worlds or servers require data packages; clients get them automatically). Things to copy: the server/common split so only client-relevant files are streamed; namespaced `group:name` ids; `Dependencies`/`OptionalDependencies`/`LoadBefore` in the manifest; Parent-based inheritance with documented merge-vs-replace rules per field; an in-game editor that writes the same format the files use.
- Hytale's typed-codec approach (each asset type has a codec with defaults) maps directly to Rust serde structs with `#[serde(default)]` and `deny_unknown_fields`.

### Gaps
- Official docs on how assets are sent to clients (protocol, caching, hashing) were not found; only third-party guides describe it.
- Exact merge rules for each field under `Parent`, and the `.blockymodel` format details, were not retrieved from official sources.
- `ServerVersion` in manifests: conflict between a third-party guide and the official example.

## Veloren (Rust): RON assets and assets_manager hot reload

### Takeaway
Veloren defines game data (items, loot tables, recipes, entity configs) in RON files loaded through a cache keyed by dot-separated specifiers (`common.items.foo` → `assets/common/items/foo.ron`). The `assets_manager` crate, inspired by Veloren's system, provides typed loading, caching and hot reload in Rust.

### Cited Findings
- `assets_manager`'s README says its original idea was inspired by Veloren's assets system — [assets_manager GitHub](https://github.com/a1phyr/assets_manager)
- Loading uses dot-separated ids: `cache.load::<Point>("common.position")` reads `assets/common/position.ron`. Asset types declare an extension and a loader (`const EXTENSION = "ron"; type Loader = RonLoader`, or a derive with `#[asset_format = "ron"]`) — [assets_manager docs](https://docs.rs/crate/assets_manager/latest)
- Hot reload: call `cache.hot_reload()` in the game loop; for `'static` caches, `enhance_hot_reloading` is preferred. Symlinks inside the cached directory can cause surprising hot-reload behavior — [assets_manager docs](https://docs.rs/crate/assets_manager/latest)
- Veloren wraps this in its own `veloren_common_assets::AssetCache` — [docs.veloren.net](https://docs.veloren.net/veloren_common_assets/struct.AssetCache.html)

### Inferences
- `assets_manager` is a ready-made Rust base for Weft: typed assets, format-agnostic loaders (a TOML loader is easy to write), caching and hot reload. Dot specifiers are a filesystem-path convention; Weft's `ns:path` ids can map onto it.
- (Background knowledge, not verified this session) Veloren's RON uses Rust enum syntax (`Some(...)`, `Tool(Sword)`), which suits developers but is less friendly to non-programmer modders, and Veloren's asset override and mod story is limited.

### Gaps
- The Veloren book page on modularity/assets returned 404; Veloren's specific item/loot RON schema and any asset-override support were not verified.

## Bevy ecosystem: asset formats and scenes

### Takeaway
Bevy has no single blessed definition format. `bevy_common_assets` turns any serde type into an asset from json, ron, toml, yaml, msgpack, xml, csv, postcard or cbor. Compound extensions (`.level.json`) route files to types. Bevy's own scenes historically used `.scn.ron`; a new scene notation (BSN) has been under development, but this session did not verify it.

### Cited Findings
- `bevy_common_assets` provides loaders (and writers) for json, msgpack, postcard, ron, toml, xml, yaml, csv and cbor, each behind a Cargo feature. You derive `Deserialize + TypePath + Asset` and register e.g. `JsonAssetPlugin::<Level>::new(&["level.json", "custom.json"])`, so compound file endings choose the asset type. Crate 0.18 targets Bevy 0.20, and 0.17 targets 0.19 — [bevy_common_assets](https://github.com/NiklasEi/bevy_common_assets)

### Inferences
- Compound extensions (`*.block.toml`, `*.recipe.toml`) are a cheap way to make the type of a file obvious and to bind a schema per type in editors. Folder-per-registry (Minecraft-style) achieves the same; pick one, not both.

### Gaps
- Status of BSN and Bevy's scene format in 2026 was not verified.

## Format comparison for hand-authored definitions: JSON, JSON5, TOML, RON, KDL, YAML

### Takeaway
There is no industry consensus. Minecraft, Hytale and Vintage Story all use JSON or a lenient JSON variant for content; Veloren uses RON; TOML dominates Rust manifests (Cargo) and flat configs. TOML's well-known weakness is deep nesting and arrays of tables, but TOML 1.1.0 (Dec 2025) allows multi-line inline tables with trailing commas, which removes much of that pain once parsers support it. TOML has the best Rust editor tooling outside JSON (Taplo/Tombi with JSON Schema via `#:schema`) and the best built-in error spans (`serde_spanned`). A split of TOML for flat or shallow definitions (blocks, items, package manifest, tags) plus JSON for Blockbench/Minecraft-style models (tool-generated, deeply nested, already read by the engine) is sensible. The weakest spots are loot tables and other deeply nested defs.

### Cited Findings
**TOML**
- TOML 1.1.0 allows "newlines and trailing commas in inline tables (#904)", so nested structures can be written in a JSON-like inline style across lines. The release also clarified dotted-key table definitions. Release files are dated 2025-12-18 — [TOML 1.1.0 release, SourceForge mirror of toml-lang](https://sourceforge.net/projects/toml.mirror/files/1.1.0/); [x-cmd blog](https://www.x-cmd.com/blog/251223)
- Parser adoption is still in progress: a Java TOML parser merged 1.1.0 support in February 2026, and the Tombi formatter applies magic trailing commas to inline tables under 1.1 — [toml-java PR #20](https://github.com/vieiro/toml-java/pull/20); [Tombi docs](https://tombi-toml.github.io/tombi/docs/formatter/magic-trailing-comma)
- Criticism of deep nesting: in TOML spec issues, commenters say deeply nested tables "become unreadable and you lose your place". A dotted path-key flattening proposal got mixed reactions — [toml-lang/toml#781](https://github.com/toml-lang/toml/issues/781); [toml-lang/toml#499](https://github.com/toml-lang/toml/issues/499)
- A PostgreSQL hackers post rejected TOML for a config file because the nested-table and array-of-tables notation is more confusing than "obvious/minimal" suggests (one person's opinion) — [pgsql-hackers](https://www.postgresql.org/message-id/CAEze2Wi-FJHA2--Cv-oBqLi1mVgTLuuFUkbQ5%3DLtmTnaKa5gmg%40mail.gmail.com)
- Tooling: Taplo picks a schema from several sources. A document `#:schema` directive has the highest priority, then a `$schema` root field, then LSP configuration, then catalogs such as SchemaStore. This comes from a DeepWiki summary of the source code — [DeepWiki: taplo lint](https://deepwiki.com/tamasfe/taplo/3.2-lint-command)
- Tombi (a newer TOML LSP/formatter) supports the same `#:schema` directive but only at the top of the document, followed by a blank line — [Tombi schema directive](https://tombi-toml.github.io/tombi/docs/comment-directive/schema-document-directive)
- Editors (VS Code Even Better TOML, Emacs lsp-mode) can also map schemas to files by regex/glob without editing the file, and default to the SchemaStore catalog — [lsp-mode TOML](https://Emacs-lsp.github.io/lsp-mode/page/lsp-toml/)
- Rust errors: the `toml` crate reports parse errors with line, column and a caret snippet, and keeps byte ranges internally — [toml crate de/error.rs](https://docs.rs/toml/latest/src/toml/de/error.rs.html)
- `serde_spanned::Spanned<T>` captures the span of a value during deserialization, but is "mostly only supported by the TOML serde library"; serde_json does not support it. Third-party crates pair spans with miette `SourceSpan` for pretty diagnostics — [Rust users forum](https://users.rust-lang.org/t/any-hacks-to-get-spans-with-serde/139939); [oh_my_toml](https://docs.rs/crate/oh_my_toml/latest/source/src/spanned.rs)
- For JSON, the `json-spanned-value` crate exists because plain serde loses span information and taking line/column from error messages is awkward — [json-spanned-value](https://github.com/MaulingMonkey/json-spanned-value)

**KDL**
- The KDL repository listed the spec as 2.0.0-draft.7 and said most implementations in the wild follow v1.0.0. The page date is unknown, so this may be stale. kdl-rs started a v2-compliant rewrite in 6.0.0-alpha.1 (Oct 2024) and has no serde deserializer; it points to knuffel/kaydle. facet-kdl is early-stage — [kdl-org/kdl](https://github.com/kdl-org/kdl); [kdl-rs CHANGELOG](https://www.github.com/kdl-org/kdl-rs/blob/main/CHANGELOG.md); [facet-kdl](https://docs.rs/crate/facet-kdl/latest/source/arborium-header.html). (Conflict: background knowledge suggests KDL 2.0.0 was finalized around late 2024; not verified here.)

**JSON / JSON5 / RON / YAML in games (from the systems above)**
- Minecraft: strict JSON for data and resource packs; Hytale: JSON for assets and manifests; Vintage Story: JSON-like asset files with patches; Veloren: RON; Luanti: Lua tables — see the sections above for citations.

### Inferences
- Comparison for this use case (authored by modders, read with serde):
  - **TOML**: comments yes; best fit for flat or shallow records (block/item defs, `package.toml`, tag lists); weak for deeply nested trees (loot pools, model element arrays, UI trees) unless written with 1.1 multi-line inline tables; best Rust error spans (`serde_spanned`); good LSP and JSON Schema support (Taplo/Tombi). Risk: the Rust `toml` crate's support for TOML 1.1 should be checked before relying on multi-line inline tables (not verified in this session).
  - **JSON**: no comments (strict); universal tooling, JSON Schema is native, and Blockbench and Minecraft models already use it; noisy for hand authoring; serde_json errors give line/column but no value spans.
  - **JSON5**: comments, trailing commas, unquoted keys; closest to what Vintage Story modders write; Rust serde support exists through third-party crates (not verified this session); editor schema support is weaker than JSON.
  - **RON**: maps Rust enums and structs one to one (great for developers, as in Veloren); unusual syntax for non-Rust modders; limited editor schema tooling.
  - **YAML**: compact for deep nesting, but whitespace and implicit typing make it error-prone; serde_yaml was deprecated in 2024 (background knowledge, not verified this session).
  - **KDL**: pleasant node syntax, but the Rust serde ecosystem is immature and the spec-version situation is confusing; not recommended as a foundation.
- Recommended split for Weft: TOML for package manifests and "record-like" defs (blocks, items, recipes, tags, sounds index, simple loot); JSON for models (Blockbench/Minecraft-compatible, tool-generated, deeply nested); and either TOML 1.1 inline tables or JSON for deeply nested defs (complex loot tables, UI layouts). Let the loader accept both `.toml` and `.json` for every registry through the same serde types; this costs almost nothing with serde. Publish one JSON Schema per registry (generated from Rust types, e.g. with `schemars`) and use it for both formats via Taplo/Tombi `#:schema` and JSON `$schema`.

### Gaps
- No quantitative or survey evidence of a cross-industry trend in game-definition formats was found; the "consensus" is that each engine picks per its ecosystem.
- Not verified: `toml` crate (toml-rs) TOML 1.1 support status; serde JSON5 crate quality; serde_yaml deprecation status; KDL 2.0 finalization date.

## Cross-cutting patterns: templates and variants, patching, namespacing, dependencies and semver, format versions and migrations, load-time validation

### Takeaway
All mature systems converge on: namespaced ids taken from the path; registries as folders; override-by-id with load order; mergeable tags; some form of templates (VS variants, Hytale `Parent`, Minecraft component defaults); a manifest with dependencies; and a format version. Where they differ, and where Weft can improve: explicit semver ranges, automatic migration of old formats, semantic patch operations, and validation that reports all errors with file/line spans before a world loads.

### Cited Findings
- Templates and variants: VS `variantgroups` + `*ByType` + `{var}` substitution + skip/allow lists — [VS Variants](https://wiki.vintagestory.at/Modding:Variants). Hytale `Parent` inheritance with child override and some fields merging — [Hytale Items](https://pre-release.docs.hytale.com/assets/item/items/). Minecraft item-type default components with per-stack diffs and `!` removal — [Data component format](https://minecraft.wiki/w/Data_component_format)
- Patching: VS JSON-Patch ops targeting `domain:path` files, with `addmerge` recommended — [VS JSON Patching](https://wiki.vintagestory.at/Modding:JSON_Patching). Minecraft offers only whole-file override, tag merging and `filter` removal — [Data pack](https://minecraft.wiki/w/Data_pack). Hytale: override by identical identity — [docs.hytale.com](https://docs.hytale.com/creating-content/asset-packs/)
- Namespacing: Minecraft `ns:path` from folders — [Data pack](https://minecraft.wiki/w/Data_pack); Hytale `{group}:{name}` (unofficial mirror) — [08-asset-system](https://gitea.minidev.space/hytale/hytale-server/raw/branch/main/docs/08-asset-system.md); VS `domain:` prefixes — [VS JSON Patching](https://wiki.vintagestory.at/Modding:JSON_Patching)
- Dependencies: Luanti names only, no versions — [Luanti mods](https://api.luanti.org/mods/); VS partial SemVer with `*` — [VS Modinfo](https://wiki.vintagestory.at/Modding:Modinfo); Hytale `Dependencies`/`OptionalDependencies`/`LoadBefore` — [docs.hytale.com](https://docs.hytale.com/creating-content/asset-packs/)
- Format versions: Minecraft `min_format`/`max_format` with major.minor and overlays — [Pack format](https://minecraft.wiki/w/Pack_format)
- Validation: Minecraft safe mode on bad packs and transactional `/reload` — [Data pack](https://minecraft.wiki/w/Data_pack); VS logs patch index/file/failing step but cannot detect silent no-ops — [VS JSON Patching](https://wiki.vintagestory.at/Modding:JSON_Patching); `toml` crate gives line/column snippets and `serde_spanned` gives per-value spans — [toml error.rs](https://docs.rs/toml/latest/src/toml/de/error.rs.html)

### Inferences (concrete recommendations for Weft)
1. **Manifest (`package.toml`)**: `id` (lowercase `[a-z0-9_]`, doubles as namespace), `version` (strict semver), `format = "1.0"` (content format major.minor), `name`, `creator`/`authors`, `description`, `[dependencies]` with cargo-style ranges (`base = "^1.2"`), `[optional_dependencies]`, `load_after`/`load_before`, and per-package `side` hints. Store the resolved package list and versions (plus content hashes) in the world so clients can fetch exactly those, as in Minecraft's level.dat pack list and Hytale's server-streamed packs.
2. **Layout**: `defs/<registry>/<path>.toml` → id `<pkg>:<path>`, so the id comes from the path and never appears inside the file. `assets/{textures,models,sounds,ui}/…` referenced by `<pkg>:<path>`. Keep server-only defs (recipes, loot) separate from client assets so the server streams only what clients need (Hytale's Server/Common split).
3. **Formats**: TOML for defs and the manifest; JSON for models (Blockbench-compatible). Also accept `.json` for any def registry (same serde type), for deep trees and for generated content.
4. **Templates**: support `parent = "pkg:path"` inheritance with documented merge rules (tables merge recursively, arrays and scalars replace), plus a small variant expander (`[variants] wood = [...]`, `{wood}` substitution). Use most-specific-match or an error for overlapping wildcard rules, not VS's first-match-wins.
5. **Patching other packages**: ship a small set of semantic ops before (or instead of) raw JSON Pointer: override whole def by id; `[[patch]] target="pkg:path" set={...}` / `merge={...}` / `remove=[keys]`; tag append/remove (merge by default, `replace = true` opt-in); recipe/loot removal via a filter list. Warn on patches whose target is missing or that change nothing.
6. **Versioning and migration**: a `format` major.minor per package (minor = additive). The engine supports a range and runs in-code upgraders from older formats at load time, so authors are not forced to update on every engine release. That beats Minecraft's no-migration approach.
7. **Validation**: `#[serde(deny_unknown_fields)]` everywhere (catches typos), `serde_spanned` / toml spans plus miette-style rendering, collect all errors in all packages before failing, resolve every cross-reference (item ids in recipes, textures in models, tags) in a link pass with "did you mean" suggestions, generate JSON Schemas from the Rust types for editor validation, and make reload transactional (keep the old registry on failure), as Minecraft does.
8. **Components over monoliths**: model item and block defs as optional typed components (`food`, `tool`, `equippable`, `light`, …), as Minecraft 1.20.5+ and Hytale's item fields do. This keeps each schema small, lets features be added without format breaks (a new optional component = minor bump), and maps cleanly to ECS.

### Gaps
- No published post-mortem comparing the long-term maintenance cost of JSON-patch-style vs override-only modding was found.
- No source was found on content-hash/checksum schemes for server-to-client package delivery in Hytale or Luanti (only the high-level "sent automatically").
