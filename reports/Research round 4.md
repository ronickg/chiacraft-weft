# Research round 4: hostile peers, finding worlds, and the first hour

Researcher, 2026-11-04, for Planner.

**What was read:**
- `dev` at `55d3a16`, which includes Phase 6 slice 3;
- the committed work on the Phase 4 branch (admission, bans, `chiacraft/bans/1`, the join's DID proof);
- the committed work on the Phase 3 leftovers branch (the shared store, outbox lanes);
- the sponsor service (`crates/sponsor`) and the survival worktree's week-one fixes, both uncommitted, as they stand on disk.

**How.** Four reviews read the code: network protocols, sandbox and packages, Phase 5, and the first hour. The Phase 5 review also searched the web. I checked these myself in the code, and each held:
- the name injection;
- the owner-by-index;
- the decoder recursion;
- the package id and translation takeovers;
- that `n0-mainline` 0.7.1 has signed peers and iroh 1.4.1 has `clear_ip_transports`;
- that R does nothing outside a screen;
- the unused hint key.

No game was run. No hostile client was run either: section 1 says what one should test.

## The top ten

| # | id | kind | owner | effort | finding |
|---|---|---|---|---|---|
| 1 | S1 | spoof, privilege | engine (+ Phase 4) | S | **A player's name rewrites their own save file.** `Join { name }` is written raw as the first line of the player file (`server/player_file.rs:163`), and `kv::get` takes the first match (`kv.rs:69`). A name of `x\nmode = creative` makes the next join creative. The same trick sets Phase 4's `probation`, `played` and `identity`. |
| 2 | S2 | spoof, takeover | Phase 2 | S | **A joined world's package can take the id `base`** and replace base's whole asset tree for the visitor. `build()` adds the world's packages with no id check (`modloader.rs:680-690`); only installed packages are checked (`:497`). |
| 3 | S3 | phishing | Phase 2 + engine | S | **Any package can rewrite any translation string**, and it outlives the world. Translations are one flat map where a later file wins (`ui/lang.rs:24-41`), and leaving doesn't put the installed content back. A hostile world can reword the join sheet ("verified by chiacraft") and the account-words prompt shown on the title screen after leaving. |
| 4 | S4 | privilege | engine | S | **While the owner is away visiting, the first newcomer gets OWNER.** Permission is `OWNER if pi == 0` (`player_file.rs:71`). When the owner travels, their local connection leaves the list and the home world runs on. A guest who holds a connection open and joins after that is index 0. |
| 5 | S5 | crash, hang, disk | engine | M | **One joined player can make the host run out of memory, hang it or fill its disk.** After the join, frames may be 64 MiB (`p2p.rs:30`), and lz4 lets about 260 KiB unpack to that. Queues are unbounded. Chat is logged whole and copied to every player. `Join` can be repeated, each one running a BLS check on the server thread. A client that stops reading grows the host's outgoing queue without end. |
| 6 | S6 | crash, hang | engine, Phase 2 | S | **A hostile host can crash or wedge a visitor's game,** and with it the visitor's own world running behind them. `Msg::get` recurses with no depth limit (`codec.rs:247`): about 100 KB overflows the stack. `Cube::read` doesn't check palette indices (`world/cube.rs:397-405`). A package's PNGs are deflate bombs: a 50 KB file decodes to 64 MiB, times up to 4096 textures. |
| 7 | S7 | hang, drain | Phase 2 | S | **Eight throwaway keys lock everyone out of joining for 15 minutes:** the configuring cap is 8, held for up to 15 min (`p2p.rs:47, 702`). The upload budget is per connection, so reconnecting drains it without end. Package-serving tasks outlive their connection (`:795`), and 64 trickling connections use up macOS's 256 file descriptors. |
| 8 | S8 | escape | Phase 6 | S | **A sandboxed mod can lower the permission of another mod's command.** The `.permission(level)` it gives its own clashing command is recorded by name and applied to the existing one (`content.rs:2815-2829`). With `tk` at 0, every visitor can run `/tk give`. |
| 9 | H1 | first hour | Survival + engine | S–M | **The first death is a spiral.** The tombstone takes the worn clothes too, and arrival clothes are given only once. So the player respawns naked, with an empty stomach (max health 40%), often at night, and the death screen gives no cause and no tombstone location. |
| 10 | H2 | first hour | Survival + engine/UI | M | **Nothing leads a new player into the survival loop.** No hint system, no advancement for fire, clothes, rest, a hot meal or taming. R (recipes) does nothing outside a screen (`app.rs:1519-1531`), and the campfire needs a crafting table. A small state-driven hints module and a "Surviving" advancement group would carry players through the first night (section 3). |

One more for Planner: the S items cross four owners but are one kind of work. **A "hostile peers" session** could take S1–S7 and the hostile-client harness (section 1.4) together, leaving S8 to Phase 6 and the sponsor to Phase 4.

---

## 1. Hostile peers

**The threat model.** A stranger, with free iroh keys, can:
- reach a world's endpoint (anyone with its address);
- reach a player's account endpoint (anyone who reads the owner's inbox from a world's status);
- join a world and play;
- host a world others visit;
- call the sponsor.

### 1.1 What a joined player or a stranger can do to a host

Worst first, damage × ease:

**S1. The name injection** (top ten 1).
- **Attack:** `Join { name: "x\nmode = creative\nprobation = lifted\nplayed = 99999999\nidentity = <a victim's identity>" }`, leave, join again.
- **What it gets:** creative (so `CreativeTake` of any item), no probation, and bans by name pointed at the victim's identity.
- **Fix:** at `Join`, allow 1–32 printable characters, refuse anything else, and escape values in `save_player`.

**S4. Owner by index** (top ten 4).
- Phase 4's `trusted()` (`server/admission.rs:100`) fixed this for bans, not for the permission. A guest who gets OWNER is also "trusted", so bans then pass them too.
- **Fix:** mark the local connection explicitly (a flag on `local_pair`'s `Conn`, or the owner's identity from the world certificate), never by its place in a list.

**S5. One joined player against the host** (top ten 5).
- **Fixes:**
  - **Per-message caps:** chat and commands 256 B, names 32, `Drag.slots` at most the slot count.
  - **Smaller frames:** at most 1 MiB after the join (enough for a skin).
  - **Queues:** a bounded `to_server` queue (about 256), disconnecting on overflow, and a per-tick message budget per player (`actions.rs:6-13` drains everything in one tick today).
  - **One join:** `Join` accepted once per connection.
  - **Slow readers:** a byte-bounded outgoing queue (`pump_out`, `p2p.rs:1017-1039`), disconnecting past a few MB or after a 30 s stalled write.

**S7. Configuring slots and upload** (top ten 7).
- **Fixes:**
  - Free the slot after 60 s with no package request in flight; keep 15 minutes only as a hard cap.
  - Check key and identity bans at the hello. Phase 4 checks them only after configuring, so banned keys take slots and upload.
  - Queue joins instead of refusing them.
  - Abort serving tasks with the connection (a `JoinSet`), with a whole-stream deadline (size / 64 KiB/s + 30 s).
  - A per-hour upload budget across all connections.
- Also: the protocol refusal prints the mod list (`:708`), and the native hash is FNV over it, so the hello's hash is free to learn. That's fine, as long as slots are cheap to lose.

**N1. Phase 3, S. The link inbox fills and blocks.**
- `receive_revoke` stores the `link` string of an unknown link, unsigned and uncapped (`links.rs:572-578`): one 16 KiB frame each, from throwaway keys.
- `receive()` saves on every message, even junk (`:465`). It saves under the global `STATE` lock, on one of the two tokio workers.
- The game thread's `links::snapshot()` (`net/screens.rs:60, 243`) and portals' `of_world` wait on that lock.
- **Fix:**
  - Keep a pre-revocation only for an id in `outgoing` or `incoming`, and cap `revoked`.
  - Save only on a change.
  - Run `receive` in `spawn_blocking`.
  - A semaphore of about 4 on the links ALPN.

**N2. Phase 3, S–M. Request eviction.** `MAX_INCOMING` = 100 drops the oldest (`links.rs:521`), so 100 forged-but-valid requests from free identities push out a real one. **Fix:** never evict requests from friends or saved worlds' owners, and give strangers their own small bucket.

**N3. Phase 4, S. DID checks per message.**
- `check_did` starts an OS thread per `Join` carrying a proof, with up to 20 s of chain lookup (`server/admission.rs:214-240`, Phase 4 branch).
- The proof's launcher id is free to choose, so each one misses the cache.
- Refused or banned connections are never closed, so they repeat.
- **Fix:** one check per connection, a shared worker with a small queue, close after `Refused`, and cache misses.

**N4. Engine, M. No global caps.**
- There is no maximum player count, no limit on connected-but-unjoined players, and one `net-out` OS thread per connection (`p2p.rs:1023`).
- **Fix:** `max_players`, a cap on unjoined connections, a semaphore per ALPN, and quinn's `max_incoming`.

**N5. Engine, S. Peer text in logs.** Names, chat, `from_name` and notes reach the log with ANSI escapes and newlines, so fake log lines can be written, and `latest.log` has no size cap (`log.rs:231-250`). **Fix:** one `printable(s, 120)` for every peer string logged; cap the log at about 100 MB.

**N6. Phase 4, S. Names aren't unique.** `whom()` takes the first match by name, so a griefer joining as "Bob" turns `/ban Bob` on the real Bob, and chat impersonation is trivial. **Fix:** refuse a name in use, show a short key tag on conflicts, and have ban commands confirm whom they mean.

**N7. Phase 3, S. A moved world's old key can still end its links.** On the Phase 3 branch, `receive_revoke` accepts the old keys. A leaked key is exactly why a world moves. **Fix:** only the current keys.

**Small items:**
- `chiacraft/worlds/1` hands out `moved`, which ties an owner's old and new keys together. That may be meant, but it's a privacy choice.
- Phase 4's `serve_status` reads `admission.json` on an async worker, unauthenticated.
- A visitor's interned stack table grows with every unique `ItemStack` a host sends.
- Every new key leaves a player file.

### 1.2 What a hostile world can do to its visitors

**S2 and S3, the package takeovers** (top ten 2 and 3).
- **Fixes:**
  - In `build()`, refuse a package whose id matches a code mod or another package.
  - A package may define only keys under its own `<id>.`; drop the rest at load.
  - On leaving a world, put the installed content back (`leave_world`, `app.rs:1000-1011`).

**S6, crashes** (top ten 6).
- **Fixes:**
  - A depth argument on `Msg::get` (at most 8).
  - Check palette indices in `Cube::read`.
  - For package textures:
    - a decoded-pixel budget per package and in total (about 256 MiB);
    - at most 64 frames per strip (a 1×4096 strip is 4096 bindless images today, `assets.rs:345-352`);
    - a 512 side cap.
  - Sounds decode in full, with no duration cap and outside `catch_unwind` (`audio.rs:152-169`). Today a package's sounds are reachable only through S2.

**P1. Phase 6, M. Visitors compile the host's wasm on the main thread.**
- While joining, `use_packages` (from `poll_join`) compiles up to 16 MiB with Cranelift and runs `init` (1e9 fuel, 500 ms).
- **Fix:** compile off the main thread with a time limit, or declare hooks in `package.toml` so visitors never compile server halves at all.

**P2. Phase 2, S. Trust comes in bundles.**
- "Download, Trust These Creators" trusts every signed creator on the sheet at once (`app.rs:1184-1188`). A hostile world lists a popular creator beside its own key.
- Once trusted, a creator's packages download without the sheet even when they carry code (`app.rs:758`).
- The store is tidied only at start (`app.rs:368`).
- **Fix:** a trust button per creator; always ask when a package carries code; tidy after each join; cap unpinned bytes.

**P3. Phase 2, S. Definition numbers aren't range-checked.** TOML accepts `nan` and `inf`, and only `light` is checked (`crates/packages/src/lib.rs:431`). `hardness`, `friction`, `mining_speed`, `attack_damage`, `max_stack` and drop counts can carry NaN into physics. **Fix:** finite, in range.

### 1.3 Sandboxed mods and the sponsor

**S8, the permission takeover** (top ten 8). **Fix:** store `(owner, name, level)` and apply it only to the owner's own command, and refuse the clashing package at load instead of leaving its command out.

**P4. Phase 6, M. No budget per tick or per mod.**
- Up to 256 hooks per mod (`lib.rs:61`), each with its own 10 M fuel: seconds a tick, and never a trap.
- `spawn_item` is uncounted (`lib.rs:507`), and so are `teleport`, `push` and `damage`.
- `play_sound` is clamped at 4.0 (+12 dB).
- `Storage::set` writes to disk on the tick thread with no per-call limit (`storage.rs:66-86`), though its doc promises one.
- **Fix:**
  - One world-tick hook per mod.
  - A fuel and wall-time budget per mod per tick, overruns counted as strikes.
  - Count spawns, and cap a mod's live entities.
  - Volume at most 1.0.
  - Write storage at save time, with a byte rate limit.

**P5. Phase 6, M. A visitor can make a guard mod fail open.**
- Fuel per call is fixed whatever the batch size. So a visitor who causes huge event batches (TNT, fast breaking) or long command arguments runs an honest mod out of fuel.
- Three strikes and it is stopped for the rest of the run (`lib.rs:804`). From then on its questions answer `Abstain` (`:886-889`), and a claims mod stops guarding without a word.
- **Fix:**
  - Fuel in proportion to the batch, or split batches.
  - A mod may declare questions `fail-closed`.
  - Tell the owner when a mod stops.
  - Don't count the caller's volume as a full strike.

**P6. Phase 6, S. Chat spoofing.**
- `tell` and `reply` take any key (`lib.rs:512`, `sandboxed.rs:655-670`). `tell(p, "chat.message", [Name("Ronald"), Text(..)])` looks exactly like Ronald talking.
- `log.write` keeps control characters.
- **Fix:** keys only under `<package>.`; mod lines get a `[package]` prefix in their own style; strip control and bidi characters from text arguments and logs.

**P7. Phase 4, S–M. The sponsor can be drained.**
- Both caps key on free things: iroh ids (`service.rs:266, 282`) and fresh BLS identities. A script takes the whole daily budget at 00:00 UTC, about 199 DIDs on testnet, and real players are refused for the day.
- 200 signed requests from fresh keys also fill `max_queue`.
- A launch unconfirmed after an hour is refunded and its identity freed (`store.rs:175-190`), even though it can still confirm. On mainnet that means two DIDs for one identity and spending past the budget.
- **Fix:**
  - Release the budget hourly (a 24th each hour).
  - A proof of work, or a host-signed invite, on the request.
  - Drop a launch only once its coins are known unspent and out of the mempool.
- The money at risk is bounded by the budget: free on testnet, the 0.00005 XCH a day default on mainnet.

### 1.4 What is solid (don't redo it)

**Frames and decoding:**
- `read_frame_max` checks the claimed and unpacked sizes before allocating.
- The codec caps list lengths and preallocates at most 4096; the `ToServer` decode has no recursion.
- No `unbounded_depth` anywhere, so serde_json's limit of 128 holds.
- The join's claimed key is replaced by the key the connection proved.
- QUIC address validation means no ALPN can be bounced at a third party.

**Status, links, tickets, friends:**
- **Status:** the certificate must be for the key that answered, the inbox needs the owner's vouching, and the name is cut to 64.
- **Link requests** are world-signed with a checked certificate, and `reply_to` must be the sender.
- **Accepts and revokes:** accepts match a signed outgoing request; revoking a known link is signed.
- **Invites** are signed, expire and work once.
- **Travel tickets** are bound to the player, the destination and the link, last 60 s, work once and are signed.
- **Friends' lists** keep only certificates the friend signed.

**Packages:**
- Manifests are canonical and hash-checked.
- Paths are allow-listed ASCII with no dot segments or links.
- Store paths are hex hashes, and every blob is checked before it is stored.
- Writes are atomic.
- The host serves only offered hashes.
- PNG decoding runs under `catch_unwind` with libpng limits.

**The sandbox:**
- No WASI.
- Fuel, an epoch watchdog, and caps on memory, tables and instances.
- Deterministic floats.
- Engine commands are reserved.
- Storage keys can't name paths.
- Client halves don't exist yet (`code/client.wasm` is refused by the allow-list).

**Shader packs** install only locally and are validated by naga; packages can't ship shaders.

**Ban lists:** signed, `seq` only goes up, capped at 2 MiB, 10k entries and 16 lists. They are fetched, never pushed, so nobody can flood updates.

**The Phase 3 outbox:** caps of 500 and 20 a lane, and `JsonStore` keeps unreadable files aside (round 1's R1, done).

### 1.5 The hostile-client harness

A Rust test client speaking our ALPNs (`crates/engine/tests/hostile.rs`, run by CI, never a game window). It runs these cases:

**Joining and downloads:**
1. Eight right hellos then silence: a ninth honest join still gets in within 60 s.
2. Four `GET_FILE` streams read at 1 byte/s, then a wrong ready, ×100 with new keys: open files and tasks return to where they started.
3. The upload budget across 50 reconnecting keys stays bounded.
4. Frames claiming 4 GiB, and lz4 claiming more than the cap, on every ALPN: refused with no allocation.

**Playing:**

5. A 64 MiB chat line, 10k `Join`s, a `Drag` of 1M slots, and a stream of tiny moves: the tick stays in budget, and memory and the log stay bounded.
6. A client that stops reading for 5 minutes: the host's memory stays flat, and the client is disconnected.
7. A name with newlines and `mode = creative`: refused, and the rejoin is survival.
8. The owner leaves while a guest holds an unjoined connection: the guest gets EVERYONE.

**A hostile host:**

9. A `Msg` nested 100k deep, and a cube with a palette index out of range: the visitor disconnects without crashing.
10. A package with the id `base`, one with `join_sheet.*` keys, and one with 64 deflate-bomb PNGs: refused, dropped, and refused or capped.

**The account endpoint:**

11. `links/1` with 10k random revokes, junk JSON, and 20 messages per connection: `links.json` size and write count stay bounded, and `snapshot()` on the game thread stays under 5 ms.
12. 150 valid requests from fresh identities: a friend's waiting request survives.
13. Forgeries:
    - a request whose `reply_to` isn't the sender;
    - an accept with another world's signature;
    - a revoke by an old key after a move.

**Phase 4:**

14. A banned key is refused at the hello (no slot, no download), and the connection is closed. 1000 joins with random DID proofs keep the thread count bounded.

**Volume:**

15. 10k parallel `status/1` and `bans/1` connections: no file-descriptor or thread exhaustion, and the workers keep answering.

---

## 2. Phase 5: finding worlds without a central service

### 2.1 What the research settles

**The DHT can't list.**
- BEP 44 items are at most 1000 bytes, may expire in 2 hours, and sit under their writer's own key, so nobody can ask for "all of them" ([BEP 44](https://www.bittorrent.org/beps/bep_0044.html)).
- pkarr says it's for discovery, not storage ([pkarr](https://github.com/pubky/pkarr)).

So any DHT listing must be a rendezvous on a shared topic:
- **BEP 5 announces** store an IP and port, which leaks the IP and is useless to iroh ([BEP 5](https://www.bittorrent.org/beps/bep_0005.html)).
- **A shared BEP 44 slot** from a derived key holds 5 records and anyone can overwrite it ([distributed-topic-tracker](https://raw.githubusercontent.com/rustonbsd/distributed-topic-tracker/main/PROTOCOL.md)). Bootstrap only.
- **Signed peers**, a draft BEP: `(key, timestamp, signature)` and **no IP**, about 10 peers a response, up to about 400 across 20 nodes ([draft BEP](https://github.com/Nuhvi/mainline/blob/main/beps/bep_signed_peers.rst)).
  - We ship it already: `n0-mainline` 0.7.1 is in our tree, with `announce_signed_peer` and `get_signed_peers` behind the `unstable_signed_peers` feature (`src/dht.rs:318, 360`; I checked it).
  - Only `mainline` v6+ nodes take part, so it is experimental.

**The privacy leak is the listed world, not the asker.**
- Direct iroh connections exchange IPs ([iroh security](https://docs.iroh.computer/concepts/security-privacy)).
- But iroh 1.4.1 has `Builder::clear_ip_transports()` (`endpoint.rs:508`; I checked it), so our pinger can be relay-only today.
- Our DHT publishing already uses `AddrFilter::relay_only()`.
- So: **anyone who pings a listed world learns its host's IP**. That is what the "list publicly" warning has to say.

**How others do it:**
- Luanti announces to one server list, off by default, and **sends player names by default** ([settingtypes](https://raw.githubusercontent.com/luanti-org/luanti/master/builtin/settingtypes.txt)).
- Veloren lists only servers it approves ([serverbrowser](https://gitlab.com/veloren/serverbrowser/-/issues/130)).
- Tox and Briar find only contacts.
- Matrix servers each publish their own directory ([matrix.org](https://www.matrix.org/blog/2019/11/09/avoiding-unwelcome-visitors-on-private-matrix-servers)).
- Nostr's relay monitors are many, and clients "MUST NOT require" any one ([NIP-66](https://github.com/nostr-protocol/nips/blob/master/66.md)).
- Bluesky's starter packs are lists anyone makes.
- 300,000 Sybil nodes were once measured on the Mainline DHT, so a DHT key with nothing else behind it has zero weight.

**Chia:**
- `RequestPuzzleState` matches hints, with `include_hinted` and `min_amount` filters, in pages ([CHIP-26](https://raw.githubusercontent.com/Chia-Network/chips/main/CHIPs/chip-0026.md)).
- Our `ChainSource::puzzle_state` (`crates/wallet/src/chain/source.rs:52`) already pages it. A listing tag is one more puzzle hash.
- `holdings.rs`, which PLAN cites, only finds our own coins.
- The world DID's metadata (Phase 4) holds the owner DID, world key and generation, with no name. That corrects PLAN's "its DID record holds the name".

**DIG** is active but small (repositories pushed October 2026; stores cost XCH plus its token), so it can only ever be an optional mirror.

### 2.2 What the player sees

**Atlas** (round 2's sketch, now with its data):
- Your world sits in the middle.
- **Ring 1:** your links, saved worlds and friends' worlds, friends in their own colour.
- **Ring 2:** what ring-1 worlds link to, only where their owners show links; greyed, names only.
- Each node shows its name with a key fingerprint, the owner, its Köppen climate colour, the season and the hour there, online or not, and the player count.
- Actions: Join, Ask to Link, Write a Letter.

**Find Worlds**, a list with tabs:
- **Near you** (rings 1–2 and friends' recommendations);
- **Lists you follow**;
- **Listed** (on chain);
- **Online now** (the DHT topic);
- **New**: collapsed and unranked, fingerprints only until opened.

Filters: open to me, my version, language, mode, climate, "no unsandboxed code". Every row says why it's there ("linked from Oakvale", "on 2 of your lists"). Hiding a world or blocking where it came from is local and silent.

### 2.3 Sources, in order, and what each reveals

| # | source | reveals, to whom | needs |
|---|---|---|---|
| 1 | your links and saved worlds | their status, to you; your pings hide your IP | status v2; a relay-only pinger (`clear_ip_transports`) |
| 2 | friends' worlds | your worlds, to friends (today) | `friends.rs` as it is |
| 3 | friends of friends | worlds a friend chose to recommend, to that friend's friends; **never the friend list** | `Answer.recommends` in `worlds/1`; a per-world "Recommend to friends" switch |
| 3b | ring 2 | a world's links, to whoever pings it, only if the owner allows | status v2 links |
| 4 | DHT topic | a listed world's key and that it's online, to anyone watching; no IP until pinged | `n0-mainline` with `unstable_signed_peers`, a new `net/topics.rs` |
| 5 | on-chain listing | forever and in public: the world DID, key and owner address | a wallet listing spend and query; Phase 4 slice 7 |
| 6 | curated lists | what the maker includes, to followers | `chiacraft/lists/1`, a copy of Phase 4's shared ban list code; an optional URL or DIG mirror |

Crawlers that walk sources 4 and 5 and ping what they find publish a signed list (source 6), like Nostr's monitors. Directories are allowed and never required (rule 1).

### 2.4 Formats

**Status v2.**
- **No new ALPN.** `serve_status` never reads the request, so a request frame of `[2]` is enough, and Phase 4 already appends fields after the v1 tuple that older readers ignore.
- **Added after `admission`:**
  - a version byte;
  - the world DID (optional);
  - climate: the Köppen code and the place (at most 32 B);
  - season, day and hour;
  - capacity, and a policy (open, via links, closed);
  - tags (mode, hardcore…);
  - language;
  - whether it runs unsandboxed code;
  - the link count, and the links themselves (key, name, link id) only when "Show my links" is on.
- **Size:** about 200 B plus about 100 B a link, so about 70 links fit `status_on`'s 8 KiB frame.
- **Ring 2:** readers draw an edge only when both sides' status agree.
- Player names are never included, only the count.

**DHT topic.**
- The infohash is `sha1("chiacraft topic v1/" + network + "/" + topic)[..20]`. Topics: `all`, `lang/xx`, `tag/<mode>`.
- A listed, hosted world announces its key every 30 minutes (my estimate; the draft gives no lifetime).
- 104 B on the wire.

**Chain listing.**
- **The coin:** 1 mojo at the owner's own address, so only the owner can unlist, and unlisting gives the mojo back.
- **Memos, about 270 B in a normal spend, usually fee 0:**
  - the hint `sha256("chiacraft listing v1")`;
  - the world DID launcher id;
  - the world key and generation;
  - the owner DID;
  - at most 128 B of version, name, language, tags and the unsandboxed flag.
- **Readers:**
  1. Page `puzzle_state([TAG], unspent)`.
  2. Fetch each new coin's parent spend once (cached forever) to read the memos.
  3. Check the world DID's metadata matches.
  4. Check the owner DID lists the world.

  The name shown comes from the live status; the memo is only a fallback.

**Curated list.** A signed JSON document: maintainer key, `seq`, title, entries `{world key or DID, note, tags}`. Served by any world or account holding it, and followed by address or key.

### 2.5 Ranking and privacy

**Ranking:**
- **Strong:** distance (mine, a friend's, recommended, linked, ring 2); on N of the M lists you follow (round 1's M2); links that both sides confirm, to worlds you know.
- **Medium:** world and owner DID age (4,608 blocks a day); uptime you saw yourself; achievements issued (later); version matches and it answers.
- **Zero:** a new key or DID with no links and on no lists. It goes in New, never on top.
- **Down:** blocked origin, on a ban list you follow, no answer to a ping (DHT keys that don't answer are dropped).
- Listing stake (`min_amount`) is a filter for readers, never a way to buy rank.

**Privacy defaults:**

| setting | default |
|---|---|
| listed on chain, announced on the DHT | **off**: one switch, "List this world publicly", warning that visitors and anyone who pings learn the host's IP |
| visible to friends | on (as today) |
| recommend to friends | off, per world |
| show my links on others' atlases | **off** |
| players | a count only, never names |
| my pings | a relay-only throwaway endpoint |
| climate, season, hour | on |

### 2.6 Slices

1. **Status v2 and Atlas ring 1** (M): the version byte and fields, the rings screen (`net/screens.rs`), the relay-only pinger. It needs round 1's N1 (one shared endpoint).
2. **Ring 2 and recommendations** (S): status links with the two-sided check; `Answer.recommends`.
3. **Find Worlds, ranking, hide and block** (M), over sources 1–3. It works offline-safe.
4. **Curated lists** (S): Phase 4's shared ban list code as `chiacraft/lists/1`; follow, N-of-M; publish one from the Atlas.
5. **On-chain listing** (M, after Phase 4 slice 7): `list_world`/`unlist_world` in the wallet, a `listings()` reader with paging, the parent-memo cache and DID checks; the Listed tab.
6. **DHT topics** (M, experimental, behind a setting): `unstable_signed_peers`, announcing while hosted and listed; the Online Now tab.
7. **Mirrors** (S, later): lists from a URL or a DIG store; a headless crawler that publishes a live directory as a list.

**What makes it cheap:** the status answer, friends' worlds, moves, saved worlds, links, warm-ups, the chain reader's hint query, the shared ban list's signed-document machinery, and the DHT code in our tree already exist. Slices 1–4 need no chain and no new service.

**Not known yet:**
- how long signed peers live on the DHT, and how many v6 nodes there are;
- a full node's page size for `RequestPuzzleState`;
- the listing spend's exact cost (estimated, not measured).

---

## 3. The first hour, with the week-one fixes in

### 3.1 Where the fixes stand

**In the survival worktree, uncommitted:**
- **Plain clothes on arrival:** a linen hood, a wool sweater, buckskin trousers and boots, about 0.9 clo, given once per player (`vitals/clothing.rs:69`, `base clothing.rs:52-58`).
- **The year starts at spring day 8:** `calendar::START_DAY = 7` (`survival/mod.rs:71-75`).
- **Cheaper clothes:** 1/2/1/1 pieces, 2 fibre a string.
- **Also:** the strongest meal only (B6), spoiled food refused (D1), leather armor as hide (C1), still air for objects (B5).
- **A campfire, partly:** +20 °C, counts for rest and smoking. But it is a 3×3 recipe (3 sticks, 4 logs, so a crafting table), burns forever with no fuel, has no smoke, and **cooks nothing**. Its textures are untracked files.

**Still only planned, in no worktree:**
- meat from wild animals (B2);
- fibre from sedge, cottongrass and cattail;
- wild rice and cattail root as food;
- unripe bushes dropping nothing.

**Order matters:** if "unripe drops nothing" lands before wild rice and cattail root are food, spring plains have no plant food at all.

### 3.2 The walk

1. **First start.** The account is made quietly. "Write Down Your Words" waits until after the first session. Good: nothing blocks play.
2. **Title screen:** Singleplayer, Multiplayer, Profile, Mods, Settings, Quit. The footer says "Weft {0} | {1} mods loaded": the engine's name, not the game's.
3. **Select World:** "No worlds yet." with Play, Create, Delete, **Address**, Folder, Back. "Address" (a new network address) is multiplayer jargon on the very first screen.
4. **Create New World:** a name, a seed, **"Survival: Normal"**, "Mobs spawn: On". "Survival" reads like a mode, not a difficulty, and its tooltip is never shown, because this screen sets no tooltips. Gentle means cold, heat and poison can't kill. That deserves a visible line.
5. **Spawn.**
   - Nearly always in plains: the `spawn_biome` tag is only on plains (`worldgen.rs:62`; the search spirals to about 3,200 blocks).
   - At 06:00 on spring day 8, with 12 hours of daylight. **Night comes 10 real minutes after spawning** and lasts 10.
   - The player has an empty inventory, the arrival clothes, and bread and an apple in the stomach.
   - With the fixes, the plains week has no freezing or cold hours (round 3's script). Taiga still has 6 freezing and 21 cold; the snowy plains 79 and 60.
6. **The HUD:**
   - health and stamina as numbers;
   - "Lv 1";
   - food icons with minutes left;
   - "P 40% E 20% F… G… W…" with no legend;
   - a feeling icon;
   - a thermometer of core temperature;
   - "12°C", unlabelled, which is the felt air;
   - unnamed effect icons, several of them identical files (round 3);
   - the clock, the date and the position;
   - E, R, P and L shortcut pictures.

   **Nothing says what to do.** Getting cold changes an icon; an empty stomach quietly caps health and stamina at 40%.
7. **The loop.**
   - **Tools:** vanilla's ladder, every step with an advancement: logs, planks, a crafting table, a wooden pickaxe, cobblestone, a furnace from 8 cobble, torches.
   - **The campfire** needs the table. Its light (15) keeps monsters away, and under a roof it makes you Rested after 20 s.
   - **The first clothing upgrade** is a buckskin shirt (2 leather).
   - **Food:** cooking needs a furnace, since the campfire doesn't cook. The stew pot needs 7 iron. Until B2, a deer gives only leather or fur.
   - **A bed** needs 3 sheep's wool.
   - **Night:** vanilla monsters only, since wild hunters leave players alone.
8. **Death** (`survival/mod.rs:52`, `levels.rs:126-156`).
   - The stomach empties and 25% of the level's XP is lost.
   - **Everything carried goes into the tombstone, including worn clothes** (equipment is part of the inventory, `sdk game.rs:377`). The arrival clothes are given only once.
   - The death screen says "You Died!" with Respawn and Quit: no cause, no tombstone location (`game_screens.rs:164-180`).

### 3.3 Where they get lost, most likely and worst first

1. **The first death spiral:** naked, an empty stomach, at night, not knowing where the tombstone is (H1).
2. **Recipes they can't find:**
   - The HUD shows R, but R does nothing without an item hovered in a screen (`app.rs:1519-1531`).
   - The real way in is the arrow in the inventory's crafting grid.
   - The campfire needs a 3×3 table.
   - The pot and the rack register no recipes, so what they make isn't in the viewer.
3. **No path into the survival loop:** nothing points at fire, clothes, or cooking in a furnace (H2).
4. **An unreadable HUD:** unnamed effects, P/E/F/G/W with no legend, "°C" without saying it is felt air.
5. **Food gaps:** no meat from hunting until B2; spring plant food at risk from the fix order; the campfire doesn't cook.
6. **Cold outside the plains:** taiga, mountains, snowy plains, with no hint of what helps.
7. **The menus' words:** Address, Key, Identity, Account 3141592653, "Survival: Normal".

### 3.4 Hints that fire from the game's state

Each fires once per player and can be turned off in settings. Text in the game's plain style.

| # | when | text | where | owner |
|---|---|---|---|---|
| 1 | 60 s played, no logs ever held | Hold {key:core:attack} on a tree trunk to get logs. | hint bar | engine/UI |
| 2 | holding logs, no crafting table | Press {key:core:inventory}: logs make planks, planks a crafting table. The arrow shows every recipe. | hint bar | engine/UI |
| 3 | the first inventory opened | Point at an item and press {key:core:recipes} for its recipes, {key:core:uses} for its uses. | hint bar (the unused `hud.hint.*` keys) | engine/UI |
| 4 | 16:00 or later, no campfire, torch or bed placed | Night falls at 18:00. A campfire (logs and sticks) warms you and keeps monsters away. | hint bar | Survival |
| 5 | Chilly or Cold | Getting cold. Stand by a fire, get out of the wind, or wear warmer clothes. | hint bar | Survival |
| 6 | Freezing (each time it starts) | Freezing: you are losing health. Get to a fire, or dig into the ground. | hint bar | Survival |
| 7 | Wet, air under 15 °C | Wet clothes keep little warmth. Dry off under a roof or by a fire. | hint bar | Survival |
| 8 | stomach empty | Your stomach is empty: you can't reach full health or stamina. Eat meat, berries or bread. | hint bar | Survival |
| 9 | holding raw meat, never held a furnace | Raw meat is better cooked. A furnace cooks it, and hot food warms you. | hint bar | Survival |
| 10 | respawned | Your things are in a tombstone where you died: x {0}, z {1}. | hint bar and the death screen | Survival |
| 11 | Hot or Heatstroke | Too hot. Find shade or water, and take off warm clothes. | hint bar | Survival |
| 12 | near a fire in the open at night | Under a roof by a fire you rest, and wake Rested. | hint bar | Survival |
| 13 | the first taming bubble within 8 blocks | It's asking for food. Hold what's in the bubble and use it on the animal. | hint bar | Animals |

**Tooltips, "comes from" lines** (Survival, Wild plants):
- **fur:** "From furred wild animals."
- **plant fibre:** "From cutting tall grass, sedge or cattail."
- **salt:** "Boiled out of a sea-water bucket in a furnace."
- **leather, string, wool:** likewise.
- **The arrival clothes:** "Plain clothes: about enough for spring. Hide, wool and fur are warmer."

**A "Surviving" advancement group:**

| id | when | title / description | owner |
|---|---|---|---|
| hearth | place a campfire | Hearth / Build a campfire from logs and sticks | Survival |
| spun | have string | Spun / Spin plant fibre into string | Survival, Wild plants |
| tailor | take a crafted piece of clothing | Tailor / Sew a piece of clothing | Survival |
| rested | the Rested effect | A Night In / Rest under a roof by a fire | Survival |
| hot_meal | eat food that is still hot | Hot Meal / Eat something hot from the fire | Survival |
| stew | have a stew | One Pot / Cook a stew | Survival |
| preserved | have dried, smoked or salted food | Put By / Preserve food for later | Survival |
| tame | tame an animal (a hook the animals mod calls) | A Friend / Tame an animal | Animals |

Hearth, Spun, Tailor and A Night In carry a player through the first night. "Hunter" (kill a farm animal) should take a wild one once B2 lands.

**What it needs.**
- **The engine already has:**
  - the advancement toast (`advancements.rs:204-247`);
  - the level-up banner (`levels.rs:197-212`);
  - the bed's message bar, fed from the server (`beds.rs:54, 466`);
  - client-side `{if:mod:x}` text conditions (`crates/sdk/src/effects.rs:313`);
  - `PLAYER_RESPAWNED` and the other events (`crates/sdk/src/extend.rs:130-141`).
- **To build:** a hints module in base.
  - A per-player "seen" bit set, saved and kept through death like advancements.
  - A once-a-second server check over the player's state, inventory and the hour.
  - A message to the client, and one hint bar at a time, about 6 s, built like the bed's.
  - A setting to turn it off.
- **Advancements:** new `Rule` kinds for an effect, a hot meal and clothes, plus a `base_api` grant hook for other mods (taming).
- **Text:** every key in en and ru.

### 3.5 Fixes that aren't hints

- **Survival, S:** respawn in plain clothes again, or keep worn clothes out of the tombstone.
- **Survival + engine/UI, S:** the death screen names the cause and the tombstone's x and z (a mod can replace `core:death`).
- **Engine/UI, S:** R and U with nothing hovered open the whole crafting list (`app.rs:1519`).
- **Engine/UI, S:** Create New World shows the rule's tooltip, and "Survival" becomes "Survival difficulty" (`saves.rs:860`).
- **Phase 3, S:** "Address" moves behind multiplayer (into the world's Links or a "…" menu).
- **Survival, S:**
  - The campfire cooks: a small furnace with no fuel slot.
  - Its recipe fits the 2×2 grid, because a fire before a table is how people start.
  - It goes out without fuel after a night.
- **Survival, art, S–M:**
  - name each HUD effect on hover;
  - a legend for the nutrient letters;
  - the degree line says "feels 12°C".
- **Engine, S:** the title footer says chiacraft.

---

## Where each goes

| owner | ids |
|---|---|
| a "hostile peers" session (proposed), else engine and Phase 2 | S1, S2, S3, S4, S5, S6, S7, N4, N5, the harness (1.5) |
| Phase 2 (packages) | S2, S3, S6 (textures), S7, P2, P3 |
| Phase 3 leftovers | N1, N2, N7, "Address" off the first screen |
| Phase 4 | N3, N6, P7 (sponsor); S1's probation and identity fields |
| Phase 6 | S8, P1, P4, P5, P6 |
| Phase 5 (to start) | section 2's slices 1–4 first; 5 after Phase 4 slice 7; 6 behind a setting |
| Survival | H1, the hints module, the Surviving group, tooltips, the campfire's cooking and 2×2 recipe, the HUD labels |
| Animals | hint 13, the `tame` advancement hook, B2 still |
| Wild plants | the fix order in 3.1, fibre from sedge, cottongrass and cattail |
| Engine/UI | R and U with nothing hovered, the Create World tooltip and name, the death screen, the footer |

## Not checked

- No game and no hostile client were run. Every attack here is read from the code and should be proven by the harness (1.5) before and after its fix.
- `dev` moved to `55d3a16` while it was read; the Phase 6 line numbers are from that commit.
- The sponsor and the survival fixes are uncommitted, so their line numbers may move.
- Phase 5's DHT numbers (signed-peer lifetime, v6 node count) and Chia page sizes aren't published.
