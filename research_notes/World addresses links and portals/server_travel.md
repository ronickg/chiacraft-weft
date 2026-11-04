# Server-to-server travel in games: portals, transfers, region crossings, prepared moves and safe returns

Scope: prior art for moving a player from one world server to another (Minecraft Java/Bedrock transfer and cookies, Velocity/BungeeCord proxies, Hytale referrals, Second Life / OpenSimulator region crossings and Hypergrid, EVE Online session changes, seamless-world handoff such as Star Citizen server meshing). Each section ends with inferences for chiacraft, which has no central server: worlds run on players' machines, links go over iroh P2P QUIC, packages are downloaded with consent and swapped in place, and the player's own world keeps running while they visit.

Source dating: Minecraft transfer/cookie material dates from 2024 (24w03a, which shipped in 1.20.5). Hytale docs are 2026 but unofficial. OpenSimulator Hypergrid pages run from 2010 to 2025; each is marked with its last-modified date. Second Life wiki pages run from 2008 to 2024. EVE figures come from community sources dated 2009 to 2016. Star Citizen material is 2023–2024.

## 1. Minecraft Java Transfer and Cookie packets, proxies (Velocity/BungeeCord), Bedrock transfer

### Takeaway
Since 1.20.5 (2024), vanilla Minecraft can move a player without a proxy. The Transfer packet carries only a host and port; the client drops its current connection and reconnects with handshake intent 3. Small cookies (at most 5 KiB each) stored on the client survive the hop, which lets servers pass state and skip re-authentication. Because the client holds the cookies, it can read, change or replay them, so servers sign them with an HMAC (a keyed hash), add an expiry and bind them to the client's address. Proxies such as Velocity predate this. They keep one client connection and swap backends behind it, which allows instant switches, a fallback list and centrally forwarded identity.

### Cited Findings
**Transfer packet (Java)**
- Snapshot 24w03a (17 Jan 2024) added it: "Custom servers can now request that clients connect to another server with a new packet." Clients connect with "a new transfer intent (id 3)". By default "servers don't accept incoming transfers and disconnect the client", and setting `accepts-transfers=true` in `server.properties` changes that. "Resource packs are maintained across transfers." "In the case of a transfer, custom servers can skip authentication with a new flag." — [Minecraft Wiki: Java Edition 24w03a](https://minecraft.wiki/w/Java_Edition_24w03a)
- Packet layout: Transfer is clientbound in both Configuration (0x0C) and Play (0x84). Fields are `Host` (String, max 32767) and `Port` (VarInt). The client closes its current connection, connects to the new address and sends a Handshake with intent 3. "If the server chooses to accept the transfer, the usual login process will follow." "Cookies previously stored are preserved between server transfers." — [Minecraft Wiki: Java Edition protocol/Packets](https://minecraft.wiki/w/Java_Edition_protocol/Packets)
- Handshake intent is "1 for Status, 2 for Login, 3 for Transfer", and intents 2 and 3 both lead to Login. "If the server is not expecting transfers, it may choose to reject the connection by replying with a Disconnect (login) packet." — [Minecraft Wiki: Java Edition protocol/Packets](https://minecraft.wiki/w/Java_Edition_protocol/Packets)

**Cookie packets (Java)**
- Store Cookie is clientbound in Configuration (0x0B) and Play (0x7A). Fields are `Key` (Identifier) and `Payload` (byte array, limit 5120). The vanilla client "only accepts cookies of up to 5 kiB in size". The page describes it as storing "arbitrary data on the client, which persists between server transfers." Cookie Request (Login/Config/Play) asks for a key. Cookie Response returns the key and an optional payload, and "the vanilla server only accepts responses of up to 5 kiB." — [Minecraft Wiki: Java Edition protocol/Packets](https://minecraft.wiki/w/Java_Edition_protocol/Packets)
- From the changelog: cookies can be requested during login, configuration and play, but only stored during configuration and play. "Cookies are persisted across server transfers but are not persisted when the player disconnects." — [Minecraft Wiki: 24w03a](https://minecraft.wiki/w/Java_Edition_24w03a)

**How servers use cookies securely (proxyless example, 2024–2025)**
- Passage, a proxyless Minecraft entry point, uses two cookies:
  - `passage:authentication` is HMAC-signed. It holds a timestamp, the client address (IP and port), username, UUID, the target backend, profile properties and an `extra` field, prefixed by a 32-byte HMAC-SHA256 signature.
  - `passage:session` is unsigned. It holds a UUIDv4, the handshake address and a W3C `traceparent`.
  - Verification checks the signature, the expiry and the bound IP. Default expiry is 6 h, and the docs recommend 60 s for production.
  - A valid cookie skips Mojang authentication and is reissued with a new timestamp and target on each hop. An invalid one falls back to full authentication.
  - Rotating the secret invalidates every outstanding cookie.
  - The secret "must be identical across all Passage instances and backend servers."
  — [Passage docs: Cookies](https://passage.scrayos.net/advanced/cookies)
- The Biscuit! Fabric mod likewise requires the cookie secret to be "the same on all servers requesting this cookie". — [Modrinth: Biscuit!](https://modrinth.com/mod/biscuit!)
- The Server Redirect mod sends a transfer packet and offers a fallback address for when the target server rejects the player. — [Modrinth: Server Redirect](https://modrinth.com/mod/YqkxTQEh)

**Proxies: Velocity / BungeeCord**
- Backends run with `online-mode=false` because the proxy authenticates players. Forwarding then passes IP, UUID and skin from the proxy to the backend.
  - Velocity "modern" forwarding is a compact binary format with a MAC, which makes it "much more difficult to trick the server into impersonating your Velocity proxy". It requires 1.13 or newer.
  - Legacy BungeeCord forwarding "is fundamentally insecure" unless only the proxy can reach the backend. BungeeGuard adds a shared token to it, which is better but "less ideal than Velocity modern forwarding".
  — [PaperMC docs: Velocity player information forwarding](https://docs.papermc.io/velocity/player-information-forwarding)
- Velocity settings related to failover:
  - `[servers] try` lists the servers tried at login and "when a player is kicked from a server".
  - `forced-hosts` maps a hostname to its own try list.
  - `failover-on-unexpected-server-disconnect` chooses between failing over and disconnecting the user after an unclean backend drop.
  - `connection-timeout` and `read-timeout` bound how long the proxy waits.
  - `accepts-transfers`: "If disabled, the proxy will disconnect transferred clients."
  — [PaperMC docs: Velocity configuration](https://docs.papermc.io/velocity/configuration)

**Bedrock**
- Bedrock's TransferPacket (per PocketMine-MP's implementation) carries `address` (string), `port` (u16 little-endian, default 19132) and a `reloadWorld` boolean. — [PocketMine-MP API docs: TransferPacket](https://apidoc.pmmp.io/d0/d0b/classpocketmine_1_1network_1_1mcpe_1_1protocol_1_1_transfer_packet.html)
- BedrockConnect uses the packet only to hand the client a new IP and port. — [BedrockConnect technical explanation](https://wiki.bedrockconnect.app/general-help/bedrockconnect-a-technical-explanation)
- Geyser validates a non-blank address and a port from 0 to 65535 before sending it. — [Geyser commit](https://gitdab.com/GeyserMC/Geyser/commit/f8e983887e96fcdc36df54c22f090afec605ab37)

### Inferences
- The Minecraft model maps closely onto chiacraft. The origin issues a small, client-carried, signed blob and an address, and the destination checks it on arrival. Minecraft needs a shared HMAC secret because servers in one network trust each other. chiacraft has no shared secret between strangers' worlds, so the natural equivalent is a public-key signature by the origin world's iroh node key (iroh node IDs are Ed25519 public keys). The destination can then verify the ticket against the link record without any prior shared secret.
- Two size budgets show what fits in a ticket: Minecraft allows 5 KiB per cookie and Hytale allows 4 KB per referral. That is enough for identity, return address, arrival-portal ID, link ID, nonce, expiry and a signature, but not for a full inventory.
- "Reject transfers by default" (`accepts-transfers=false`) is a sensible stance for chiacraft too. A world should only admit travellers that arrive through a link it has accepted.
- "Resource packs are maintained across transfers" corresponds to chiacraft's in-place package swap. It argues for keeping the origin's packages cached rather than discarding them, so the return trip can swap back without a download.
- A proxy is unavailable in a P2P design, but its key benefit can be imitated. The player's own client keeps the origin connection (or the origin world itself) alive while connecting to the destination, which gives the same switch-without-drop property and a fallback target, like Velocity's `try` list.

### Gaps
- I did not fetch PaperMC's API docs for `Player#transfer` / `storeCookie` / `retrieveCookie`, nor Folia's region threading. They were not verified, so no claims are made about them.
- The behaviour of Bedrock's `reloadWorld` flag is not documented in the sources I found.
- No official Mojang security note on cookies was found. The "sign and expire" guidance comes from third-party projects.

## 2. Hytale referrals (2026 early access) and Luanti

### Takeaway
Hytale ships a native, proxyless "referral". The server sends `ClientReferral(host, port, data≤4096 B)`, and the client reconnects with the data plus the source address. The destination receives this unauthenticated and is told to verify it with an HMAC, a source allow-list and replay protection. The destination can also re-refer the player elsewhere. Luanti has no built-in server-to-server travel.

### Cited Findings
- The transfer is "entirely client-driven". `ClientReferral` (packet 18, server to client) carries `hostTo` (HostAddress: host ≤256 chars, port) and `data` (byte[] ≤4096). The client's `Connect` (packet 0) to the destination includes `referralData` and `referralSource`. The API is `PlayerRef.referToServer(host, port[, data])`. — [Hytale Docs (community, craftserve): Server Transfer](http://hytale-docs-mcp.craftserve.com:8080/protocol/server-transfer/)
- The destination handles `PlayerSetupConnectEvent`, which exposes `isReferralConnection()`, `getReferralData()`, `getReferralSource()`, `setCancelled`/`setReason`, and `referToServer(...)` to re-route an incoming player before they join. — [same source](http://hytale-docs-mcp.craftserve.com:8080/protocol/server-transfer/)
- The server logs the warning "unauthenticated - plugins must validate!" for referred players. The docs recommend three checks: source verification against trusted servers, an HMAC or signature over the data, and timestamps or nonces against replay. They include an example that HMAC-signs (UUID, timestamp, ...). — [same source](http://hytale-docs-mcp.craftserve.com:8080/protocol/server-transfer/)
- Built-in test commands are `/refer <player> <host> <port>` (alias `/transfer`), with permissions `command.refer.self` and `command.refer.other`. — [same source](http://hytale-docs-mcp.craftserve.com:8080/protocol/server-transfer/)
- A third-party router mod (Eros Server Handler) uses shared-secret authentication between lobby and target servers. — [CurseForge: Eros Server Handler](https://www.curseforge.com/hytale/mods/eros-server-handler/files/7589364)
- Luanti/Minetest: in a 2022 IRC exchange, a user asked whether players could travel between servers and was told no. The only alternative suggested was moving builds by WorldEdit export/import. — [Luanti IRC log 2022-03-10](https://irc.luanti.org/minetest/2022-03-10/text)

### Inferences
- Hytale's re-route hook is a useful pattern for chiacraft. A destination that is full, has revoked the link or is shutting down can bounce the traveller instead of dropping them, either back to the `referralSource` (the origin) or to an alternative. Combined with a source address carried in the ticket, it gives a built-in return path.
- The `referralSource` field (the client tells the destination where it came from) is the minimal "return address". chiacraft should sign it inside the ticket rather than trust it as a bare field.

### Gaps
- No official Hypixel Studios documentation was found. The docs above are a community decompilation and should be checked against the shipped server. Nothing found on Hytale's client UX during referral (loading screen, timeouts, failure screen).
- No Luanti feature request or mod for server travel was found, but I did not search the Luanti GitHub or ContentDB.

## 3. Second Life / OpenSimulator: region crossings, teleports, child agents, Hypergrid

### Takeaway
Second Life and OpenSimulator prepare the destination before the move. Neighbouring regions already hold a "child agent" (a non-authoritative presence) for the viewer, so crossing a border just promotes it to root. Teleports first ask the destination to create the agent and only then tell the client to switch. The destination calls back the origin ("release") once the client has actually arrived, and only then does the origin close its copy. Hypergrid extends this to foreign grids. The destination's Gatekeeper verifies the visitor by calling back the visitor's home grid (UserAgent `VerifyAgent`). Inventory and asset requests are relayed to the home grid, and admins can allow or deny lists of grids and block asset types from being imported or exported.

### Cited Findings
**Region crossings (Second Life wiki, last edited 11 Feb 2024; unofficial, written by third-party viewer developers)**
- Regions are 256 m squares, each run by its own simulator. A crossing triggers when the root prim's centre is "one meter past the boundary", which prevents flip-flopping. For a vehicle, its root prim decides, so riders stay together. — [SL Wiki: Region crossing](https://wiki.secondlife.com/wiki/Region_crossing)
- Sequence:
  1. The losing and gaining sims exchange server-side messages.
  2. The viewer gets `CrossedRegion` via the event poller.
  3. The viewer sends `CompleteAgentMovement`, and the gaining sim answers `AgentMovementComplete`. Arrival order "is not guaranteed".
  4. The avatar stays out of sync until the gaining region sends an ObjectUpdate.
  
  An out-of-sync state that "persists for more than a few seconds" means the crossing failed. Double crossings at corners are "generally unsatisfactory" because the sims are not interlocked. Multiple avatars on one vehicle "tends to be unreliable". The page calls crossing "a rather brittle operation". — [SL Wiki: Region crossing](https://wiki.secondlife.com/wiki/Region_crossing)
- Neighbour setup: the region sends `EnableSimulator`, then `EstablishAgentCommunication`, which carries the agent ID, a seed capability URL and the neighbour's IP and port. Until it arrives, "all functions which require capabilities or events are broken", and region crossings may be affected. SL sims sometimes send it about a minute late. — [SL Wiki: Adding neighbor region sequence](https://wiki.secondlife.com/wiki/Adding_neighbor_region_sequence); [SL Wiki: EstablishAgentCommunication (old rev.)](https://wiki.secondlife.com/w/index.php?oldid=164443&title=EstablishAgentCommunication)
- `EnableSimulator` is sent whenever the sim wants the client connected to another simulator. "This includes establishing child agents and is a prerequisite to 'see into' regions that you're not current in" (2008 mailing list). — [sldev list, Sept 2008](https://list-archives.secondlife.com/sldev/2008-September/011904.html)

**OpenSimulator simulation service (source interfaces, 0.7.x–0.9.0, older)**
- `ISimulationService` defines:
  - `CreateAgent(destination, circuitData, flags, out reason)`, which asks the destination to create the agent before the move.
  - `UpdateAgent`, in a full child-agent update form and a short, mostly-position form.
  - A release message "sent from the receiving region back to the departing region to say the client has made contact".
  - `CloseAgent`.
  
  Commit notes say the receiving region calls back the sending region after the client becomes root there (after CompleteMovement). They also say the origin's child agents must be closed before the receiving region's child agents are updated. — [ISimulationService.cs (opensim-SC mirror)](https://sledjhamr.org/cgit/opensim-SC_OLD/plain/OpenSim/Services/Interfaces/ISimulationService.cs); [IEntityTransferModule.cs 0.7.4](https://sledjhamr.org/cgit/opensim-SC_OLD/tree/OpenSim/Region/Framework/Interfaces/IEntityTransferModule.cs?h=0.7.4); [ScenePresence log](https://sledjhamr.org/cgit/opensim-SC_OLD/log/OpenSim/Region/Environment/Scenes/ScenePresence.cs?showmsg=1)
- 0.7.4 `IEntityTransferModule` declares `EnableChildAgents`, `Cross` and `AgentArrivedAtDestination`. In 0.9.0 the entry points are `Teleport` and `CrossAgentToNewRegionAsync`. — [IEntityTransferModule.cs 0.9.0](https://sledjhamr.org/cgit/opensim-SC_OLD/plain/OpenSim/Region/Framework/Interfaces/IEntityTransferModule.cs?h=0.9.0)

**Hypergrid (OpenSimulator wiki)**
- Hypergrid "allows you to link your opensim to other opensims on the internet, and … supports seamless agent transfers". The user reaches the foreign world "without having to logout from the world where she came from, and while still having access to her inventory". The address format is `hop://<host>:<port>[/<region>][/x/y]`, and if no region is given the destination grid's default region is used. — [OpenSim wiki: Hypergrid (modified 24 Dec 2021)](http://opensimulator.org/wiki/Hypergrid)
- The Hypergrid model lets each instance add "neighbors" to its own map, so there is no central map. Agents of users "registered elsewhere" can travel, and the transfer carries "the collection of servers that the transferring user needs", meaning the visitor's home services. — [OpenSim wiki: Hypergrid Concepts](http://opensimulator.org/wiki/Hypergrid_Concepts)
- Protocol (HG 2.0+):
  1. The source asks the destination Gatekeeper about the region. The Gatekeeper returns host and port, or "a gateway region" if direct teleport is disallowed, or a failure.
  2. The Gatekeeper receives a login for `Joe @ http://grid-source:8002` and authenticates it with the source grid's UserAgent service.
  3. The Gatekeeper notifies the destination region that a connection is incoming.
  4. The region waits for the viewer's seed-capability connection and verifies the user with the home UserAgent service again.
  5. Attachments then follow.
  
  If the home UserAgent is unreachable, verification fails and the teleport is denied: "Failed to verify user presence in the grid … access denied". The destination relays inventory requests to the home grid, fetches assets referenced by rezzed objects from the home asset service, and copies them locally if import is allowed. The viewer talks to no grid services during the teleport. — [OpenSim wiki: Hypergrid Protocol](http://opensimulator.org/wiki/Hypergrid_Protocol)
- HG 2.0 controls:
  - `[GatekeeperService] ForeignAgentsAllowed`, with `AllowExcept` / `DisallowExcept` lists of grids.
  - `[HGAssetService] DisallowExport = "LSLText, LSLBytecode"` and `DisallowImport = "LSLBytecode"`.
  - The HGInventoryService "Suitcase", a limited inventory exposed to foreign grids.
  - `[UserAgentService] ForeignTripsAllowed_Level_<n>` with per-level allow/deny lists.
  — [OpenSim wiki: Hypergrid 2.0](http://opensimulator.org/wiki/Hypergrid_2.0); [Hypergrid Security (modified 8 Apr 2025)](http://opensimulator.org/wiki/Hypergrid_Security)
- Threat model, historical but explicit: a malicious host "has your session id, so it can request your inventory items on your behalf … [and] could simply wipe out your inventory". Items used while visiting are cached there. Visitors' purchases end up as copies on their own asset servers, where permissions can be changed (piracy). — [OpenSim wiki: Hypergrid Security](http://opensimulator.org/wiki/Hypergrid_Security)

### Inferences
- The child-agent idea fits chiacraft's prepare-while-approaching step directly:
  1. When the player nears a portal, open the iroh connection to the destination and perform a `QueryAccess`/`CreateAgent`-style reservation: check that the link is valid, the visitor is allowed and the arrival portal exists.
  2. Download the packages (with consent).
  3. Optionally stream the area around the arrival portal at low priority. This is the child-agent "see-into" step, and could even render a preview through the portal.
  4. On contact, only promote the reservation to a live player.
- Adopt the release pattern. The origin keeps the traveller's authoritative state (inventory, position at the portal) until the destination confirms that the client has arrived and become the active player. If no confirmation arrives within a timeout, the origin restores the player at the portal. This is the core of a safe return.
- Hypergrid's call-home verification and "suitcase" are the closest prior art to decentralised identity and inventory. A visitor's home (in chiacraft, the player's own world or identity key) vouches for them. Only a scoped subset of inventory is exposed to foreign worlds, and certain content types (scripts there, maybe Weft code packages in chiacraft) are never imported or exported.
- The 1 m hysteresis and the "not interlocked" double-crossing failures argue for a short cooldown and a single in-flight travel per player (see EVE below), plus an arrival placement that is offset from the destination portal's trigger volume so the traveller is not immediately sent back.
- Hypergrid's warning that a malicious host can act on your session applies to any capability handed to a visited world. chiacraft tickets should be single-purpose (admission to this link and arrival), short-lived and not usable to act on the home world.

### Gaps
- No current (2024–2026) step-by-step OpenSimulator teleport-flow document was found. The wiki's "Teleports and Region Crossings" page does not exist. The sequence above is pieced together from older source interfaces.
- No Second Life official (Linden) documentation of `TeleportFinish` / teleport failure handling was retrieved.
- Exactly what Hypergrid transfers in the agent circuit data (appearance, attachments, service URLs) was not documented beyond the summary above.

## 4. EVE Online session changes, MMO zone handoff and seamless worlds (Star Citizen, SpatialOS, Dual Universe)

### Takeaway
EVE treats a gate jump as a discrete "session change" behind a loading transition. A session-change timer stops back-to-back changes, and a post-jump cloak protects the arriving player. Seamless designs (WoW-style ghosts, Photon's overlap zones, Star Citizen's replication layer) avoid loading screens by giving a player a non-authoritative "ghost" on the neighbouring server ahead of time and transferring authority at the boundary. That is the same idea as Second Life's child agents.

### Cited Findings
- EVE: a session change occurs whenever the character changes location (dock, undock, gate jump). Sources disagree on the timer length: about 30 s historically, later 10 s, and 15 s in a 2012 post. A 2011 thread notes the gate cloak lasts 60 s while the session-change lock was 30 s, so a player cannot jump straight back. Undock invulnerability was 30 s per a 2016 post. All community sources are older than 2017. — [EVE forums archive](https://forums-archive.eveonline.com/topic/7363/); [Engadget EVE Evolved (2009)](https://www.engadget.com/2009/05/31/eve-evolved-eve-onlines-aggression-mechanics-part-2/); [EVE forums archive 282359](https://forums-archive.eveonline.com/topic/282359)
- Star Citizen: in 3.21 (2023) the replication layer was moved off game servers into a standalone service, so "the game server [can] be an authority over a specific area, rather than the entire game universe". — [RSI Roadmap Roundup, 4 Oct 2023](https://api.star-citizen.wiki/comm-links/19505)
- In a March 2024 test, players travelled between Stanton and Pyro through jump tunnels, "with each system streaming in and out seamlessly". The letter stated 4.0's jump tunnels as intended to transition between game servers. — [Letter from the Chairman 2024-03-14](https://starcitizen.tools/Comm-Link:Letter_from_the_Chairman_-_2024-03-14)
- CIG Q&A: moving between servers requires "the ability to transfer authority over entities from one server to another, as well as bring new servers online and shut down old ones". — [Server Meshing and Persistent Streaming Q&A](https://api.star-citizen.wiki/comm-links/18397)
- Photon (2024–2025 blog, pre-product): a seamless multi-node world uses an overlap zone in which each node predicts and reconciles entities it doesn't own. They guarantee that every entity operation is executed exactly once, including during transfer. — [Photon blog: Building an MMO Backend](https://blog.photonengine.com/building-a-seamless-mmo-backend-across-machines/)
- WoW-style zoning (practitioner comments): the player truly lives on one zone server, but "a 'ghost' of that player lives in the other ones". The ghost has no physics and routes everything back to the owning zone. — [r/gamedev thread (2021)](https://redlib.hbubli.cc/r/gamedev/comments/opoikw/world_of_warcraft_tech)

### Inferences
- chiacraft portals are discrete jumps between unrelated worlds, not a contiguous map, so the EVE/teleport model (discrete session change with a short transition) fits better than seamless meshing. The prewarming techniques from seamless worlds still apply to the approach phase: ghost connections, early authority negotiation and streaming the neighbouring area.
- Borrow EVE's two timers:
  - A per-player travel cooldown, or "session lock", prevents portal ping-pong and races such as a second portal trigger while one travel is in flight.
  - Short arrival protection (invulnerability or no-collision for a few seconds) covers arriving while chunks or entities are still loading.
- Exactly-once transfer (Photon) is the property chiacraft needs for inventory carried between worlds. An item must never exist in both worlds or in neither, which argues for an explicit two-phase handover or simply not carrying items at all.

### Gaps
- No primary SpatialOS documentation on load balancing or boundary handoff was found, and Improbable has largely wound SpatialOS down. Nothing citable on Dual Universe's single-shard handoff ("CSSO") was found either.
- Whether Star Citizen 4.x shipped live server-to-server jump-tunnel transfers, and how they handle a destination server crash, was not confirmed by 2025–2026 sources in this pass.
- No current CCP source gives the present session-change timer value.

## 5. What travels with the player, and how it is authenticated

### Takeaway
Across systems the payload stays small:
- an identity proof
- where the player came from (the return address)
- where they should arrive
- a timestamp or nonce
- optionally some opaque game state

It is carried by the client (Minecraft cookies ≤5 KiB, Hytale referral ≤4 KB) or sent server-to-server (OpenSim CreateAgent circuit data). Because client-carried data can be tampered with, it is signed (HMAC with a shared secret in centralised networks), expired quickly, bound to the client and checked for replay. Federated systems (Hypergrid) instead verify identity by calling back the player's home.

### Cited Findings
- Passage's signed cookie holds timestamp, client IP and port, username and UUID, target server, profile properties and `extra`. It is verified for signature, expiry (60 s recommended) and IP binding. — [Passage docs](https://passage.scrayos.net/advanced/cookies)
- Hytale's guidance calls for source allow-list, HMAC/signature, and timestamp or nonce replay protection, and the server warns that referral data is "unauthenticated". — [Hytale Docs (community)](http://hytale-docs-mcp.craftserve.com:8080/protocol/server-transfer/)
- Velocity modern forwarding MACs the forwarded identity so backends can trust it. — [PaperMC Velocity forwarding](https://docs.papermc.io/velocity/player-information-forwarding)
- Hypergrid's Gatekeeper and the destination region both verify the agent against the home grid's UserAgent service, and the teleport fails if home is unreachable. — [OpenSim wiki: Hypergrid Protocol](http://opensimulator.org/wiki/Hypergrid_Protocol)
- Hypergrid relays inventory and assets from the home grid instead of moving them, and exposes only a "Suitcase". Some asset types can never be imported or exported. — [OpenSim wiki: Hypergrid 2.0](http://opensimulator.org/wiki/Hypergrid_2.0)

### Inferences
- A suggested chiacraft "travel ticket", signed by the origin world's node key and well under 4 KiB:
  - player identity: public key, plus the player's signature over a destination-supplied challenge during the QUIC handshake to prove they hold the key
  - origin world ID (node ID) and the link ID
  - origin portal position (the return point)
  - destination world ID and the arrival portal ID
  - issued-at and expires-at (tens of seconds)
  - nonce
  - optional allowed-items manifest or hash
  
  The destination checks four things: the link record (both ends signed it, it is not revoked), the signature, the expiry, and that the nonce has not been seen before.
- In a decentralised setting, prefer "call home" verification (Hypergrid-style) only as an option. The origin world is the player's own and keeps running while they visit, so the destination can reach it to confirm presence. But it may go offline, so a self-contained signed ticket is more robust.
- Inventory: either carry none (safest, matches Hypergrid's relay model), or carry a signed manifest. The destination treats it as visitor-only or temporary items, and the origin keeps the authoritative copy (two-phase, exactly-once).

### Gaps
- No source describes public-key (non-HMAC) travel tickets in a shipped game. That recommendation is an inference.

## 6. Arrival placement, link revocation, destination failure and return

### Takeaway
Systems handle failure at three points:
- Before the switch: the destination refuses admission, so the player stays put (Hypergrid denial, Minecraft disconnect at login, Hytale `setCancelled`).
- During the handshake: the destination re-routes the player elsewhere (Hytale `referToServer` in the connect event, Hypergrid "gateway region").
- After arrival, when the destination dies: an orchestrator moves the player to a fallback (Velocity `try` list with `failover-on-unexpected-server-disconnect`).

With no proxy, the client or the origin has to provide that fallback itself.

### Cited Findings
- Velocity's `try` list is used "when a player is kicked from a server", and `failover-on-unexpected-server-disconnect` chooses failover over disconnect after an unclean backend drop. — [PaperMC Velocity configuration](https://docs.papermc.io/velocity/configuration)
- Server Redirect falls back to a configured address if the target rejects the player. — [Modrinth: Server Redirect](https://modrinth.com/mod/YqkxTQEh)
- The Hypergrid Gatekeeper can return "a gateway region" instead of the requested region, or a failure. A missing region in a `hop://` address falls back to the grid's default region. — [Hypergrid Protocol](http://opensimulator.org/wiki/Hypergrid_Protocol); [Hypergrid](http://opensimulator.org/wiki/Hypergrid)
- Hytale destinations can cancel with a reason, or re-refer an incoming player (passing the referral data along) before they join. — [Hytale Docs (community)](http://hytale-docs-mcp.craftserve.com:8080/protocol/server-transfer/)
- OpenSim's release callback lets the origin keep its copy until the destination confirms contact. — [ISimulationService.cs](https://sledjhamr.org/cgit/opensim-SC_OLD/plain/OpenSim/Services/Interfaces/ISimulationService.cs)
- Minecraft cookies vanish on disconnect, so a failed transfer that becomes a disconnect loses the travel state. — [Minecraft Wiki: 24w03a](https://minecraft.wiki/w/Java_Edition_24w03a)
- Second Life crossing failures show up as an out-of-sync avatar for more than a few seconds. Crossing into a not-yet-live neighbour "usually works" and forces it live. — [SL Wiki: Region crossing](https://wiki.secondlife.com/wiki/Region_crossing)

### Inferences
- Suggested chiacraft failure ladder:
  1. Prepare phase fails (no route, link revoked, download declined or failed): the portal shows "unavailable" and nothing happens. The player never leaves.
  2. Admission refused at switch time: stay in the origin, show the reason on screen (Hytale-style reason string).
  3. Arrival portal missing or blocked: the destination picks a fallback spawn (nearest free spot to the portal, else world spawn, as with Hypergrid's default region). If its policy is to refuse, it bounces the player back via the ticket's return address.
  4. Destination goes offline during the visit: the client detects the QUIC connection loss or timeout and automatically returns to the origin portal. That is the player's own running world, so it is almost always available. If the origin itself is unreachable (for example a friend's world, not their own), fall back to the player's home world. This is the decentralised version of Velocity's `try` list.
  5. Show a disconnect screen only if every fallback fails.
- Keep travel state outside the session that might fail. Minecraft cookies die on disconnect, so chiacraft should persist the return ticket and origin snapshot locally on the client and in the origin world, not only in connection state.
- Revocation mid-travel: tickets have short expiry and name a link ID. The destination re-checks the link at admission. A visitor already present when a link is revoked can be allowed to stay until they leave, with return through the bound return address still honoured (simplest), or be returned politely. The choice is a policy question.

### Gaps
- No source found describes how Velocity, Hytale or SL present failure to the user in detail (screen copy, retry countdowns).

## 7. UX: transition screens, timing, cancelling, prefetch triggers, bandwidth

### Takeaway
Loading-screen transfers (Minecraft, Hytale, EVE gate jumps, SL teleports) hide the switch behind a short transition. Seamless systems hide it by prewarming a ghost connection and streaming the neighbour early. Prefetch triggers are spatial: SL child agents follow draw distance, and seamless MMOs start the neighbour session when the player enters a border region.

### Cited Findings
- SL/OpenSim keep child agents in regions within view so players can "see into" them before crossing. — [sldev list 2008](https://list-archives.secondlife.com/sldev/2008-September/011904.html)
- Border-region prewarming in seamless prototypes:
  1. When a player enters the border, Server A notifies Server B, and B opens an area-of-interest session.
  2. When the player is close enough, the client connects to B and replicates it as ghost data.
  3. Authority then switches to B.
  — [r/gamedev discussion](https://redlib.hbubli.cc/r/gamedev/comments/opoikw/world_of_warcraft_tech)
- Minecraft keeps resource packs across transfers, which avoids a re-download on each hop. — [Minecraft Wiki: 24w03a](https://minecraft.wiki/w/Java_Edition_24w03a)
- Star Citizen's jump tunnels act as a themed transition while systems stream in and out (intended behaviour, 2024). — [Letter from the Chairman 2024-03-14](https://starcitizen.tools/Comm-Link:Letter_from_the_Chairman_-_2024-03-14)
- EVE's session-change lock and post-jump cloak give a protected arrival window. — [EVE forums archive 282359](https://forums-archive.eveonline.com/topic/282359)

### Inferences
- Proximity prefetch for chiacraft, in three zones:
  - Outer zone, about 32 blocks from the portal: resolve the link and open QUIC.
  - Middle zone, about 16 blocks: fetch the package manifest, request download consent if needed, and start the download at background priority.
  - Inner zone, at the portal surface: commit.
  
  If the download isn't finished when the player enters the portal, show a transition screen with progress and a Cancel button. Cancel keeps the player in the origin; nothing has been committed yet. Distances are illustrative, not sourced.
- Themed transitions (Star Citizen's tunnel, EVE's warp effect) can mask package swap time. A brief portal "vortex" screen is acceptable and needs less engineering than seamless meshing.
- Bandwidth: cache packages keyed by content hash so repeat visits and return trips cost nothing. Limit concurrent prefetches when several portals are nearby, and cancel a prefetch if the player walks away (with hysteresis, like SL's 1 m rule).

### Gaps
- No published measurements were found for transfer latency, acceptable loading times or prefetch radii in these games, and no GDC talk was retrieved in this pass.
