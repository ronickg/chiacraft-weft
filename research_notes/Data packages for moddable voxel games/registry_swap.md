# Registry synchronization, id remapping and content hot-swapping (for Weft fork)

Scope note: research done 2026-10-31. Sources fetched: minecraft.wiki protocol pages (current Java protocol 26.3 / protocol 777), Fabric API source (branch 1.21.4), NeoForge source (1.21.x) and docs (26.1), Luanti engine docs/source, Factorio API docs and forums, Hytale official news (Nov 2025), arc-swap docs (1.9.3). Claims marked "(prior knowledge, unverified)" come from background knowledge, not a source fetched this session. The report writer should treat them as lower confidence.

## Minecraft vanilla: configuration phase, dynamic vs static registries, block state ids

### Takeaway
Vanilla splits registries into two kinds. Built-in registries (block, item, entity_type, sound_event...) are hardcoded, so client and server must ship identical ids. Data-driven registries (biomes, dimension types, damage types, mob variants...) are sent by the server during the configuration phase, and the order of the entries sets the numeric ids. The server can send a play-state client back into configuration to change them without disconnecting, but re-entry is fragile and tears down client state (level, chat, resource pack).

### Cited Findings
- Connection states: Login Acknowledged (serverbound login 0x03) "switches the connection state to configuration". Configuration clientbound packets include Registry Data (0x07), Feature Flags, Update Tags (0x0E), Known Packs (0x0F) and Finish Configuration (0x03). Serverbound packets include Known Packs and Acknowledge Finish Configuration. Play has a clientbound Start Configuration (0x78) and a serverbound Acknowledge Configuration. All of this is for protocol 26.3 / 777. — [minecraft.wiki: Java Edition protocol/Packets](https://minecraft.wiki/w/Java_Edition_protocol/Packets)
- The protocol docs say some registries such as biomes and dimensions "can be customized at runtime by the server," while others such as blocks, items and entities "are hardcoded." Blocks use a "global block state palette" that gives every block state a numeric id. — [minecraft.wiki: Packets](https://minecraft.wiki/w/Java_Edition_protocol/Packets)
- Built-in registries are defined in code, change only through modding, and their ids "can and often do change between versions". Examples are `minecraft:block`, `minecraft:entity_type` and `minecraft:sound_event`. — [minecraft.wiki: Registry data](https://minecraft.wiki/w/Java_Edition_protocol/Registry_data)
- Synchronized data-driven registries are sent one Registry Data packet per registry during configuration. "The ordering of the entries in the Entries array defines the numeric IDs that they will be assigned to." "If an entry is not mentioned in a Registry Data packet, it will not exist in the game session." The client disconnects if anything references a missing entry. — [minecraft.wiki: Registry data](https://minecraft.wiki/w/Java_Edition_protocol/Registry_data)
- Known Packs: the two sides negotiate which data packs they share, so the server can omit entry NBT for content the client already has. Entry names must still be listed. "In practice the vanilla client and server as of 1.21.8 only recognize one data pack, `minecraft:core`." — [minecraft.wiki: Registry data](https://minecraft.wiki/w/Java_Edition_protocol/Registry_data)
- Tags: "Tag data is never sourced from known packs, and must always be sent by the server in full". They arrive in Update Tags after Registry Data and before Finish Configuration. Some registries require particular tags or entries, or the client rejects Finish Configuration / Login (play). — [minecraft.wiki: Registry data](https://minecraft.wiki/w/Java_Edition_protocol/Registry_data)
- Re-entry pitfalls, 1.20.2–1.20.4 era: Start Configuration makes the client call `Minecraft#clearClientLevel()`, which clears chat (MC-265250). The server resource pack is reset. In one report, server-triggered reconfiguration kicked players because a `system_chat` packet was sent after `start_configuration`, and that packet is not registered in the configuration state. — [Mojang bug tracker MC-265250](https://bugs.mojang.com/browse/MC-265250); [MC-265463](https://bugs-legacy.mojang.com/browse/MC-265463); [MC-270378](https://bugs-legacy.mojang.com/browse/MC-270378)
- Fabric's 1.20.2 developer post says the configuration phase lets client and server configure each other after authentication and before the player joins a world, and makes things easier for proxies. — [Fabric 1.20.2 blog](https://fabricmc.net/2023/09/12/1202.html)

### Inferences
- Vanilla's model is a hybrid. Blocks and items, the hottest numeric ids in chunk palettes and item stacks, are static and assumed identical on both sides. Only lower-churn data registries are adopted from the server, with the server's ids. Weft's "freeze at startup" `Content` is currently all-static, like vanilla blocks.
- Re-entering configuration is vanilla's "swap content at a world boundary without restarting the process". Its rules map closely onto the Weft plan:
  1. Drop the client level before swapping.
  2. Make the network state machine reject gameplay packets while swapping, which is where the `system_chat` bug came from.
  3. Treat swap-adjacent state (chat, resource packs) as torn down.
- Known Packs is a content-hash or pack-identity negotiation used only to save bandwidth. Ids still come from the server's ordering. That is the "adopt server ids, skip payload if the client has the pack" design.

### Gaps
- I did not get the exact field layout of Start Configuration, or confirmation of how a re-configured client rebuilds tags and registry-dependent caches internally. The packets page was too long to fully read.
- I did not verify how Velocity or other proxies use reconfiguration for server switching.

## Fabric Registry Sync (fabric-registry-sync-v0)

### Takeaway
Fabric makes the client adopt the server's raw ids at join, via a configuration-phase task. It snapshots the client's original (vanilla-first) order so it can restore that order on disconnect, and it fires remap callbacks so code that caches raw ids can rebuild. Only registries marked SYNCED and MODDED are synced. Unknown remote ids fail the join, and holes in the id space are fatal.

### Cited Findings
- On the server, each registry is included only if it has `RegistryAttribute.SYNCED` and `RegistryAttribute.MODDED`. The code comment reads "Dont do anything with vanilla registries on client sync". The map is `Identifier -> (Identifier -> rawId)` per registry. — [Fabric RegistrySyncManager.java (1.21.4)](https://github.com/FabricMC/fabric/blob/1.21.4/fabric-registry-sync-v0/src/main/java/net/fabricmc/fabric/impl/registry/sync/RegistrySyncManager.java)
- Sending is a `ServerPlayerConfigurationTask`, `SyncConfigurationTask`, with key `"fabric:registry/sync"`. A client without the channel is let in only if every synced registry is `OPTIONAL`. Otherwise it is disconnected with a message listing up to four non-vanilla namespaces. Singleplayer hosts skip the sync. — [RegistrySyncManager.java](https://github.com/FabricMC/fabric/blob/1.21.4/fabric-registry-sync-v0/src/main/java/net/fabricmc/fabric/impl/registry/sync/RegistrySyncManager.java)
- In `remap()` REMOTE mode (the client applying the server's map), every remote id must exist locally. Otherwise it throws `"Received ID map for <registry> contains IDs unknown to the receiver!"` listing them. — [SimpleRegistryMixin.java (1.21.4)](https://raw.githubusercontent.com/FabricMC/fabric/1.21.4/fabric-registry-sync-v0/src/main/java/net/fabricmc/fabric/mixin/registry/sync/SimpleRegistryMixin.java)
- On the first remap, Fabric snapshots the current id order because "it contains the vanilla order of IDs before mods, which is crucial for vanilla server compatibility". — [SimpleRegistryMixin.java](https://raw.githubusercontent.com/FabricMC/fabric/1.21.4/fabric-registry-sync-v0/src/main/java/net/fabricmc/fabric/mixin/registry/sync/SimpleRegistryMixin.java)
- Local-only entries:
  - AUTHORITATIVE mode (applying a saved map) appends them after the highest remote id, with the warning "Adding <id> to saved/remote registry."
  - REMOTE mode also assigns them new ids above the max, even though a code comment says "For REMOTE, we instead drop them".
  - A hole in the local raw-id table throws "Unused id N in registry ...".
  - The tables are cleared before reinsertion, so a late failure leaves the registry partially rebuilt, with no rollback.
  — [SimpleRegistryMixin.java](https://raw.githubusercontent.com/FabricMC/fabric/1.21.4/fabric-registry-sync-v0/src/main/java/net/fabricmc/fabric/mixin/registry/sync/SimpleRegistryMixin.java)
- After a remap, `RegistryIdRemapCallback` fires with the old and new id maps, so listeners can rebuild derived raw-id caches. `unmap()` restores the snapshot by re-running `remap(prev, AUTHORITATIVE)`, re-emits "added" events for entries culled during the session, then clears the snapshot. — [SimpleRegistryMixin.java](https://raw.githubusercontent.com/FabricMC/fabric/1.21.4/fabric-registry-sync-v0/src/main/java/net/fabricmc/fabric/mixin/registry/sync/SimpleRegistryMixin.java)
- With debug flags (`fabric.registry.debug`), Fabric logs objects that don't round-trip through their id, or two objects sharing one raw id. It can also dump registry contents as CSV. — [RegistrySyncManager.java](https://github.com/FabricMC/fabric/blob/1.21.4/fabric-registry-sync-v0/src/main/java/net/fabricmc/fabric/impl/registry/sync/RegistrySyncManager.java)

### Inferences
- Fabric shows the cost of "mutate ids in place inside a live process":
  - every derived raw-id cache needs a remap listener;
  - partial failure corrupts state;
  - code that missed the event silently holds stale ids.
- Weft avoids most of this by building a new immutable `Content` instead of mutating one.
- The snapshot-and-restore idea still applies: keep the client's own "home" `Content` (native mods only) interned, so leaving a server returns to it for free.
- (prior knowledge, unverified) Fabric also saves per-world id maps in the world folder and applies them in AUTHORITATIVE mode on world load. That is how modded worlds keep raw ids stable across mod additions. Block state ids derived from block raw ids get recomputed after a remap. Known community failures are mods that cache `Registry.getRawId(...)` in static fields or arrays at init and then break after a join remap.

### Gaps
- I did not fetch Fabric's world-save id persistence code (the PERSISTED attribute and the save-file name) or its "missing entries on world load" UI, so those details are unverified.
- I did not fetch documented pitfall lists from Fabric docs or issues.

## NeoForge / Forge: snapshots, sync on join, freezing, the 1.20.x+ rework

### Takeaway
NeoForge keeps two cached snapshots: a full vanilla snapshot and a frozen sync snapshot. To apply a server's ids it unfreezes each registry, clears it, re-registers entries with the received ids, collects missing keys and refreezes. Leaving reverts to the frozen snapshot. Integer-id sync is opt-in per registry and documented as for networking only, never persistence.

### Cited Findings
- `takeSnapshot(SYNC_TO_CLIENT)` covers only registries where `doesSync()` is true, and holds ids only. `FULL` covers every registry and holds entries, and is "never sent to the client or saved to disk". At startup NeoForge caches `vanillaSnapshot` (FULL) and `frozenSnapshot` (SYNC_TO_CLIENT). — [NeoForge RegistryManager.java](https://raw.githubusercontent.com/neoforged/NeoForge/1.21.x/src/main/java/net/neoforged/neoforge/registries/RegistryManager.java)
- `generateRegistryPackets(isLocal)` sends nothing for local play. For remote clients it sends one `FrozenRegistryPayload` per synced registry. — [RegistryManager.java](https://raw.githubusercontent.com/neoforged/NeoForge/1.21.x/src/main/java/net/neoforged/neoforge/registries/RegistryManager.java)
- `applySnapshot`:
  - it ignores registries the client doesn't know about "as long as they are empty", and throws otherwise;
  - for each registry it calls `unfreeze()`, clears it, re-registers id mappings, collects keys it doesn't have into a `missing` set, re-adds aliases and calls `freeze()`;
  - it returns the missing set for the caller to handle; a warning notes that missing entries "will be deleted from the save file on next save".
  `revertToFrozen()` simply re-applies `frozenSnapshot`. — [RegistryManager.java](https://raw.githubusercontent.com/neoforged/NeoForge/1.21.x/src/main/java/net/neoforged/neoforge/registries/RegistryManager.java)
- Docs for 26.1:
  - `sync(true)` is "If you want to enable integer id syncing, for networking"; such ids "should only be used in networking contexts".
  - `maxId` is "generally discouraged".
  - `defaultKey` is a fallback entry "similar to minecraft:air".
  - "DO NOT QUERY REGISTRIES WHILE REGISTRATION IS STILL ONGOING!"
  - Datapack registries load "at world load, instead of loading them when the game starts". A null network codec means entries "will not be synced to the client at all".
  — [NeoForge docs: Registries](https://docs.neoforged.net/docs/concepts/registries/)

### Inferences
- NeoForge's model is: frozen baseline, apply remote ids at join, revert to baseline at leave. That matches the Weft plan of a "home" `Content` plus a per-server `Content`, but NeoForge mutates in place where Weft would swap whole objects.
- The docs' insistence that numeric ids are network-only and names are for persistence is direct support for Weft's "names in saves (ids.txt palette), runtime ids only on the wire and in memory".
- The `defaultKey` / air fallback is the standard answer for unknown content. Map unknown names to a placeholder ("unknown block") instead of crashing, as Luanti does.
- (prior knowledge, unverified) The 1.20.2+ NeoForge rework dropped Forge's `ForgeRegistry` wrapper in favour of patched vanilla registries, and moved sync into the configuration phase.

### Gaps
- I did not confirm the exact disconnect or missing-mappings UX for remote joins, or how old Forge `MissingMappingsEvent` / remapping maps onto modern NeoForge.

## Luanti: node ids and client handling

### Takeaway
Luanti is server-authoritative for everything. The server sends serialized node and item definition managers, plus media identified by sha1, after auth and before world data. The client builds a fresh game per connection. On disk, each MapBlock carries its own name-to-id mapping, so global ids are never persisted, and unknown names become a special "unknown" node.

### Cited Findings
- The init sequence is TOSERVER_INIT, then TOCLIENT_HELLO, then TOCLIENT_AUTH_ACCEPT, then TOSERVER_INIT2 ("After this, the server can send data."). TOCLIENT_NODEDEF (0x3a) carries a "serialized NodeDefManager (zstd-compressed)". TOCLIENT_ITEMDEF (0x3d) carries a serialized ItemDefManager. TOCLIENT_ANNOUNCE_MEDIA lists file names with a sha1 digest each, plus an optional remote media server URL. TOCLIENT_BLOCKDATA carries serialized MapBlocks. — [Luanti networkprotocol.h](https://raw.githubusercontent.com/luanti-org/luanti/master/src/network/networkprotocol.h)
- Special nodes are ignore, air and unknown, "the only nodes added by Luanti directly". Unknown is used when "the map contains a node with an unknown/undefined type". The MapBlock disk layout includes `NameIdMapping nimap; // node ids in this block`. — [Luanti docs: Basic data structures](https://docs.luanti.org/for-engine-devs/basic-data-structures/)
- Node ids are 16-bit. Mod ids must stay well below the maximum so there is room for dummy ids created when a MapBlock with unknown names is loaded. A historical change renumbered ids per block, starting from 0, instead of keeping global ids in saves. — [mapnode.h (older fork copy)](https://gogs.librecmc.org/OWEALs/minetest/src/d4ccc8de7924091e004109abd92fa0f6e7124f0c/src/mapnode.h); [commit](https://gogs.librecmc.org/OWEALs/minetest/commit/5d06bdf8ac3b38b1ffb400fd771a245589d92ad1); [MapBlock dictionary](https://github.com/fancyfinn9/mtanvil/wiki/MapBlock-dictionary)

### Inferences
- The client adopts whatever ids are in the server's NodeDefManager. There is no client-side id agreement at all, because Luanti clients carry no native content.
- Unknown names on load become fresh dummy ids rather than errors. The original name survives in the per-block map, so re-adding the mod restores the content.
- Media is cached by sha1, so switching servers only downloads changed files. That is a cheap "content hash" cache Weft can copy for package assets.

### Gaps
- I did not confirm from current source whether network MapBlock serialization uses global ids or a per-block map, or the exact id allocation order (registration order, I believe; prior knowledge, unverified).

## Factorio: mod sync and "restart"

### Takeaway
Factorio needs the client's mod set and startup settings to match the server's exactly. On join it offers to sync (download/enable mods) and then "restarts". The data (prototype) stage runs in a throwaway Lua state, so a mod change means rebuilding all prototypes. Since 1.1.61, mods and settings sync in one pass, cutting the number of restarts.

### Cited Findings
- "The data stage runs in a fresh shared Lua state" and that state is discarded afterwards. Data-stage code that differs between players causes desyncs, and the game errors if it detects the data changed. — [Factorio Data Lifecycle docs](https://lua-api.factorio.com/1.1.0/Data-Lifecycle.html)
- 1.1.61 release notes: mods and settings are synced at the same time, reducing the number of restarts needed, and no password re-entry is needed after mod-sync restarts. — [Factorio forums: Version 1.1.61](https://forums.factorio.com/viewtopic.php?p=570127)
- Before that, joining needed two connects (one to sync mods, one to sync mod settings) when the server's settings differed. — [Forum suggestion: sync mods and settings at same time](https://forums.factorio.com/viewtopic.php?p=430054)
- Bug in the 0.12 era: the join check compared the *enabled* mod list, not the *loaded* one. Enabling mods without restarting let the client join, download the map and desync instantly. Staff acknowledged it. — [Factorio forums 20026](https://forums.factorio.com/20026)
- `script.on_configuration_changed()` fires when mods are added or removed, mod versions change or prototypes change. It is the hook for migrating save data. — [Factorio Data Lifecycle docs](https://lua-api.factorio.com/1.1.0/Data-Lifecycle.html)

### Inferences
- Factorio's "restart" is a full reload of mods and prototypes. I did not confirm whether it is in-process or a process relaunch (see Gaps). Either way, it treats the prototype set as immutable once loaded, exactly like Weft's frozen `Content`.
- The 0.12 bug is the canonical stale-content failure: validate the content that is actually loaded (a hash of the frozen `Content`), not the configured list.

### Gaps
- I found no primary source confirming whether a Factorio mod-sync restart reuses the process (in-process reinit) or relaunches the executable. Also not checked: how Factorio assigns or persists numeric prototype ids in saves. I believe saves carry name-based migration mappings (prior knowledge, unverified).

## Vintage Story / Hytale

### Takeaway
Vintage Story clients are prompted to download the server's required mods when joining. I found no reliable source on whether a client restart follows. Hytale (Early Access, from January 2026) rejects client mods entirely: the server supplies all content, singleplayer is a local server, and the client is meant to stay "stable, secure, and consistent" across servers.

### Cited Findings
- Vintage Story: players connecting to a modded server "will be automatically prompted to download any required mods". The mod must support client-side download. Server mods load only at server start. — [WinterNode guide](https://winternode.com/help/games/vintage-story/setup/mods/install-mods-on-your-vintage-story-server); [Lagless docs](https://docs.lagless.gg/vintage-story/mod-installation) (hosting-provider docs, not official)
- Hytale, November 2025:
  - "Server-side first: All modding in Hytale is based on the host of the server or host of the game";
  - "even when you join singleplayer, you join a local server";
  - players should "join any modded Hytale server without downloading external mods or juggling client packs";
  - the studio does not intend "to support any client mods";
  - blocks, items, NPCs and worldgen are data-driven (JSON assets).
  — [Hytale Modding Strategy and Status](https://hytale.com/news/2025/11/hytale-modding-strategy-and-status)
- Asset packs (blocks, items, models, sounds) and Java plugins are both installed only on the server. — [GameServerKings](https://www.gameserverkings.com/knowledge-base/hytale/how-to-install-hytale-mods/); [CurseForge blog](https://blog.curseforge.com/how-to-create-hytale-mods/)

### Inferences
- Hytale's design means the client never has to reconcile native content with server content. That is the "packages are pure data, the server is authoritative" end of the spectrum. Weft's split (native dylib mods vs data packages) is a hybrid: packages can be server-supplied Hytale/Luanti-style, while native mods need Minecraft/Fabric-style agreement.

### Gaps
- I found no official source on Vintage Story's client-side reload mechanism or how it syncs block/item ids. (Prior knowledge, unverified: the VS server assigns block and item ids and sends them to the client.)
- I found no source on how Hytale assigns block ids or whether the client hot-reloads assets between servers in-process.

## Engine approaches: Bevy, arc-swap, leak-per-generation, generational handles

### Takeaway
For rarely replaced, read-mostly global state, `ArcSwap` gives atomic whole-value replacement with consistent snapshots for readers. That is a direct alternative to `OnceLock<&'static Content>` when swapping at world boundaries. Leaking one `Content` per distinct hash keeps `&'static` borrows valid at a bounded memory cost.

### Cited Findings
- arc-swap 1.9.3 is "optimized for read-mostly scenarios, with consistent performance characteristics". It suits config and routing tables that are read often and replaced rarely. Readers get a consistent version and updates don't block processing. It outperforms `RwLock<Arc<T>>` both contended and uncontended. Canonical pattern: `static X: Lazy<ArcSwap<T>>`; `X.load()` to read; `X.store(new)` to replace. 1.9 strengthened some memory orderings. — [arc-swap docs](https://docs.rs/arc-swap/latest/arc_swap/)

### Inferences
- `&'static Content` vs `ArcSwap<Content>`:
  - `&'static` borrows are free and work with native dylibs, but only leaking (interning) makes them sound across swaps.
  - `ArcSwap` frees old generations, but every holder must re-`load()`, and a `Guard` held across a swap pins the old generation. That is safe, but it is the "stale content" bug in a memory-safe form.
- Given native mods holding `&'static` references and statics, leak-and-intern by hash is the pragmatic choice. Memory grows only with the number of distinct content sets visited per process (usually a handful). The Horizon prototype shows it works.
- A per-generation `content_epoch: u64` stored next to any cached id lets debug builds assert that cached ids match the current `Content` generation. That makes stale-id bugs loud, as Fabric's debug round-trip checks do.
- (prior knowledge, unverified) Bevy's asset hot-reload uses `AssetServer` file watching and `AssetEvent::Modified` events, and handles are generational ids into `Assets<T>`. Bevy has no built-in "registry swap" for gameplay type registries.

### Gaps
- I did not fetch Bevy docs this session, so the Bevy specifics are unverified.

## Id stability strategies and recommendation for Weft

### Takeaway
Every mature system uses names in persistent storage, numeric ids only at runtime and on the wire, and the client adopting the server's id assignment (Fabric, NeoForge, Luanti, vanilla data registries). Requiring byte-identical ids via a content hash only works as a fast path, which is what vanilla Known Packs is.

For Weft, I recommend a server-authoritative id table on the wire plus a deterministic local order: native content in a fixed leading range sorted by name, then package content sorted by name. Native mods resolve their ids by name after each freeze and are never allowed to keep ids across a swap.

### Cited Findings
- Server-ordered ids, client adopts: "The ordering of the entries in the Entries array defines the numeric IDs" — [minecraft.wiki: Registry data](https://minecraft.wiki/w/Java_Edition_protocol/Registry_data); Fabric REMOTE remap — [SimpleRegistryMixin.java](https://raw.githubusercontent.com/FabricMC/fabric/1.21.4/fabric-registry-sync-v0/src/main/java/net/fabricmc/fabric/mixin/registry/sync/SimpleRegistryMixin.java); NeoForge applySnapshot — [RegistryManager.java](https://raw.githubusercontent.com/neoforged/NeoForge/1.21.x/src/main/java/net/neoforged/neoforge/registries/RegistryManager.java)
- Numeric ids are for networking only: "These should only be used in networking contexts" — [NeoForge docs](https://docs.neoforged.net/docs/concepts/registries/)
- Per-chunk name palettes in saves: Luanti's per-MapBlock `nimap` — [Luanti docs](https://docs.luanti.org/for-engine-devs/basic-data-structures/)
- Preserve the original (vanilla/native-first) order and restore it after leaving a server — [SimpleRegistryMixin.java](https://raw.githubusercontent.com/FabricMC/fabric/1.21.4/fabric-registry-sync-v0/src/main/java/net/fabricmc/fabric/mixin/registry/sync/SimpleRegistryMixin.java)
- Built-in ids "can and often do change between versions", which is why vanilla never persists them — [minecraft.wiki: Registry data](https://minecraft.wiki/w/Java_Edition_protocol/Registry_data)

### Inferences (concrete recommendations)
1. **Name-sorted assignment.**
   - Pros: deterministic with no persisted state, and two processes with the same content set get identical ids.
   - Cons: adding one entry shifts every later id, so ids are not stable across content-set changes.
   - That con only matters for anything that persists or caches ids. Saves already use the ids.txt name palette, and the wire can carry the server's table, so name-sorted assignment is acceptable as long as ids never cross a swap.
2. **Native first, packages after.** Packages then never shift native ids, which keeps native ids stable across servers that share the same native mod set.
   - This is still not stable if the native mod set differs, because a new native mod sorted in the middle shifts the others.
   - So native mods must still re-resolve ids by name whenever `Content` is swapped.
   - A per-namespace range (each native mod gets a block keyed by its name hash) is an alternative, but it wastes id space and collides. I don't recommend it over re-resolution.
3. **Wire protocol.**
   - The handshake sends a content hash (of the sorted name list plus definition hashes).
   - If the client has an interned `Content` with that hash, it uses it. This is the Known Packs-style fast path.
   - Otherwise the server sends its `name -> id` table plus the package data, Luanti/Hytale-style with sha1-cached media.
   - The client builds a new `Content` whose ids exactly equal the server's table, and rejects the join if the server lists a native name the client lacks. That is Fabric's "IDs unknown to the receiver" failure, but done before any mutation.
   - Client-only native entries should get ids above the server's max and never go on the wire, like Fabric REMOTE mode.
4. **Native mods caching ids.** Replace raw cached `u16` statics with something that gets rebuilt on each freeze:
   - a `ContentGeneration` callback (Fabric's `RegistryIdRemapCallback` equivalent) that native mods implement to re-resolve ids by name; or
   - a lazily resolved `BlockRef { name, cached: AtomicU32 (epoch<<16 | id) }` that re-resolves when the epoch differs.
   Both make stale ids detectable instead of silently wrong.
5. **Swap protocol** (mirrors vanilla reconfiguration):
   1. unload the world and drop every chunk/mesh/entity referencing old ids;
   2. switch the network into a "configuring" state that rejects gameplay packets;
   3. build or intern the new `Content` and run native re-resolve callbacks;
   4. rebuild GPU atlases and models;
   5. only then accept the world.
   Never let packets from the old session be decoded with the new `Content`.
6. **Unknown content in saves.** Map unknown names from ids.txt to a placeholder "unknown" block that preserves the original name, as Luanti does, so removing a package doesn't destroy data and re-adding it restores the content. Warn on load (NeoForge warns that missing entries will be deleted on save, and Weft should avoid that loss).
7. **Leave and return.** Keep the client's home/singleplayer `Content` interned and switch back to it on disconnect, like Fabric `unmap` and NeoForge `revertToFrozen`.

### Gaps
- I found no source quantifying memory cost or experiences with leak-per-generation in shipped games. That recommendation rests on reasoning and the user's own Horizon prototype.

## Lessons from failures: stale ids and cached references after swapping

### Takeaway
The recurring failures are:
- validating configured content instead of loaded content;
- gameplay packets arriving mid-swap;
- partial in-place remaps that leave inconsistent tables;
- holes or duplicates in id tables;
- code that caches raw ids and misses the remap event.

### Cited Findings
- Factorio checked the enabled mod list instead of the loaded mods, causing an instant desync after join. — [Factorio forums 20026](https://forums.factorio.com/20026)
- Minecraft reconfiguration kicked players when a play-state packet (`system_chat`) arrived after Start Configuration. Chat and resource packs are lost on re-entry. — [MC-270378](https://bugs-legacy.mojang.com/browse/MC-270378); [MC-265250](https://bugs.mojang.com/browse/MC-265250); [MC-265463](https://bugs-legacy.mojang.com/browse/MC-265463)
- Fabric remap clears its tables before reinsertion, so an exception mid-loop leaves a partial registry. Holes ("Unused id") and duplicate raw ids are explicitly detected as errors. — [SimpleRegistryMixin.java](https://raw.githubusercontent.com/FabricMC/fabric/1.21.4/fabric-registry-sync-v0/src/main/java/net/fabricmc/fabric/mixin/registry/sync/SimpleRegistryMixin.java); [RegistrySyncManager.java](https://github.com/FabricMC/fabric/blob/1.21.4/fabric-registry-sync-v0/src/main/java/net/fabricmc/fabric/impl/registry/sync/RegistrySyncManager.java)
- Factorio: data-stage code that differs per player (for example conditional `require`) causes desyncs and false version mismatches. — [Factorio Data Lifecycle](https://lua-api.factorio.com/1.1.0/Data-Lifecycle.html); [Forum: 1.1.61 thread](https://forums.factorio.com/viewtopic.php?p=570170)
- NeoForge: querying registries during registration is forbidden ("DO NOT QUERY REGISTRIES WHILE REGISTRATION IS STILL ONGOING!"), which is the timing version of stale reads. — [NeoForge docs](https://docs.neoforged.net/docs/concepts/registries/)

### Inferences
- Building a new immutable `Content` and swapping it atomically (or interning it) removes the partial-remap class of bugs entirely. The remaining risks are references held across the swap:
  - native statics;
  - GPU resources indexed by id;
  - in-flight packets;
  - cached `ItemStack`s in UI or inventory;
  - async mesh jobs.
- Async chunk meshing and worldgen threads are the most likely Weft-specific stale-id hazard. Join or cancel all workers before the swap, and tag each job with the content epoch.
- Hash the loaded `Content` itself (names plus definitions), not the package list, for the handshake and save compatibility check.

### Gaps
- I didn't collect concrete Fabric or Forge issue reports for "mod cached raw id and broke after join". The pattern is well known (prior knowledge), but I did not cite specific issues.
