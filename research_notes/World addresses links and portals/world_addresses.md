# Per-world addresses, discovery and address books for iroh-based P2P worlds

Research date: 2026-10-31. Current iroh release on docs.rs is 1.4.1. In 1.x the "discovery" module was renamed `address_lookup`, and "Node ID" became `EndpointId`. Blog posts and older docs pages still say "discovery" / "NodeId"; treat those names as pre-1.0.

## 1. How iroh looks up an address in 2026 (DNS/pkarr, mainline DHT, mDNS), what it publishes, TTLs, privacy, and multiple endpoints/keys

### Takeaway
An iroh 1.x endpoint is reachable by its `EndpointId` alone once an address lookup service is configured. The default is n0's pkarr publisher plus DNS resolution at dns.iroh.link. It republishes every 5 minutes and by default publishes only the home relay URL, not IP addresses. Mainline DHT and mDNS are opt-in crates. One `Endpoint` has exactly one `SecretKey`, and the relay disconnects the older of two connections that use the same key. So each world key needs its own `Endpoint`, and that key must be live in only one place at a time.

### Cited Findings
**Version and modules**
- iroh 1.4.1 is the current docs.rs release. Its modules include `address_lookup` ("looks up the address of an endpoint ID"), `endpoint`, `endpoint_info` (DNS resource records for dialing by endpoint ID), `protocol` (accept loop/router) and `dns`. — [docs.rs/iroh](https://docs.rs/iroh/latest/iroh/)
- `EndpointId` is a type alias for `PublicKey` (Ed25519). Each endpoint has one `SecretKey`, which authenticates and encrypts its connections. Dialing by `EndpointId` alone requires an address lookup service; without one you must supply a relay URL or direct addresses. — [docs.rs/iroh](https://docs.rs/iroh/latest/iroh/)
- Current docs use "address lookup". Older pages and blog posts say "discovery" and "Node ID". The DNS record name is `_iroh.<endpoint-id>`. — [docs.iroh.computer address lookup](https://docs.iroh.computer/concepts/address-lookup); [iroh dns blog](https://iroh.computer/blog/iroh-dns)

**Lookup services in iroh 1.x**
- `MemoryLookup`: the app adds and removes addresses out of band. Good for saved server lists and tickets. — [iroh 1.0.0 address_lookup.rs](https://docs.rs/crate/iroh/1.0.0/source/src/address_lookup.rs)
- `DnsAddressLookup`: resolves through ordinary DNS. Publishing to it requires a `PkarrPublisher`. — same source
- `PkarrPublisher` / `PkarrResolver`: publish to and resolve from a pkarr relay over HTTP. — same source
- mDNS and Mainline DHT live in separate crates, `iroh-mdns-address-lookup` and `iroh-mainline-address-lookup`. — same source
- Calling `Endpoint::builder(...).address_lookup(...)` several times combines the services into `AddressLookupServices`, which queries them all concurrently. At runtime, `ep.address_lookup()` returns a registry with `add`, `add_boxed`, `clear`, `len` and `is_empty`. The docs example pairs `PkarrPublisher::n0_dns()` with `DnsAddressLookup::n0_dns()`. — same source
- `AddressLookup` trait: `publish(&self, data: &EndpointData)` is fire-and-forget. `resolve(&self, endpoint_id)` returns an optional stream of `Item`. Each `Item` carries `EndpointInfo`, a `provenance` string and an optional `last_updated` timestamp in microseconds. The trait has no `subscribe`. `EndpointData` has `with_user_data`, so a small user-data string can travel with the record. — same source
- `iroh-mainline-address-lookup` 0.6.1 (listed 20 Oct 2026) depends on iroh ^1.0 and `n0-mainline` ^0.7. It exposes `DhtAddressLookup` and a `Builder`, described as "Pkarr Mainline DHT and relay server address lookup". It uses both pkarr relays and the DHT directly. — [docs.rs iroh-mainline-address-lookup](https://docs.rs/iroh-mainline-address-lookup/latest/iroh_mainline_address_lookup/)
- By default iroh uses DNS/pkarr lookup with origin domain dns.iroh.link, run by n0. Mainline DHT is opt-in. The n0 DNS path "does not interact with the Mainline DHT, so is a more central service". — [docs.iroh.computer discovery](https://docs.iroh.computer/concepts/discovery) (via search summary)

**What is published, TTLs and lifetime**
- `PkarrPublisher` publishes only the `RelayUrl` by default (`AddrFilter::relay_only()`), which "avoids leaking IP addresses to the public pkarr server". `AddrFilter::unfiltered()` also publishes direct IPs, which can make dialing faster. — [iroh 1.0.0 pkarr.rs](https://docs.rs/crate/iroh/1.0.0/source/src/address_lookup/pkarr.rs)
- `DEFAULT_PKARR_TTL` is 30 s. The docs say this TTL is ignored by iroh-dns-server and pkarr relays. `DEFAULT_REPUBLISH_INTERVAL` is 5 min, and the record is republished even when unchanged. Failed publishes back off linearly. — same source
- Relay endpoints: `N0_DNS_PKARR_RELAY_PROD` is `https://dns.iroh.link/pkarr` and `N0_DNS_PKARR_RELAY_STAGING` is `https://staging-dns.iroh.link/pkarr`. Setting `IROH_FORCE_STAGING_RELAYS` selects staging. — same source
- Builder API: `PkarrPublisher::builder(url)` / `::n0_dns()`, then `.ttl(u32)`, `.republish_interval(Duration)`, `.addr_filter(AddrFilter)`, `.dns_resolver(..)` and `.build(SecretKey, rustls::ClientConfig)`. Data is updated with `PkarrPublisher::update_endpoint_data(&EndpointData)`. Resolver: `PkarrResolver::n0_dns()`. — same source
- The pkarr DNS TXT spec defines `relay=<url>` and `addr=<addr> ...`. Records are signed, so a resolver can verify that the record came from that EndpointId. Others must already know the EndpointId to look it up. — [docs.iroh.computer address lookup](https://docs.iroh.computer/concepts/address-lookup)
- pkarr turns Ed25519 keys into self-signed DNS packets (A/AAAA/TXT/CNAME) on the Mainline DHT. Packets are limited to 1000 bytes ("PKARR is for discovery, not storage"). Records are ephemeral: the DHT drops them after hours, so they must be republished. Clients and relays "cache aggressively". The FAQ suggests building "petname systems, phonebooks, or DNS bridges" on top for human names. — [pubky/pkarr README](https://github.com/pubky/pkarr)
- BEP 44 (the DHT layer under pkarr): mutable items are stored under the SHA-1 hash of the public key. An optional salt (at most 64 bytes) lets one key store several items. `seq` must strictly increase, and `cas` gives compare-and-swap. Items "MAY expire in 2 hours" without re-announcement and "SHOULD be re-announced once an hour". Any interested node may re-announce an item; it does not need the private key, because the signed blob is self-verifying. — [BEP 44](https://bittorrent.org/beps/bep_0044.html)

**Running several endpoints, and two endpoints sharing one key**
- iroh docs: "Generally, an application will have a single endpoint instance", so that connections share state. By default an endpoint generates a new random identity on every launch. To keep a stable ID, "store the endpoint's `SecretKey` and load it on every launch", so that tickets keep working. When address lookup is on, store only the `EndpointID`, because relay URLs and direct addresses change quickly. — [docs.iroh.computer endpoints](https://docs.iroh.computer/concepts/endpoints.md)
- Relay protocol: `RelayToClientMsg::Status(Status::SameEndpointIdConnected)` means "Another endpoint connected with the same endpoint id. No more messages will be received." So the relay keeps only the newest connection per key. — [iroh-relay 0.98 protos/relay.rs](https://docs.rs/crate/iroh-relay/0.98.0/source/src/protos/relay.rs)

### Inferences
- An endpoint has one `SecretKey`, so a per-world key means one `Endpoint` per hosted world, with its own socket, relay connection and pkarr publisher. Nothing found forbids several `Endpoint`s in one process; the docs just default to one. A player visiting while hosting would run the account endpoint plus N world endpoints. The cost is roughly one relay connection and one republish every 5 minutes per world. Measure this before committing.
- Never run the same world key on two machines at once, for example the owner's laptop and a headless server. The relay will silently kick one of them, and pkarr records will flap between the two. Hand-off needs an explicit "stop here, start there" step, or a separate key per host (see section 2).
- For privacy, keep `AddrFilter::relay_only()` for public worlds. Consider `unfiltered()` only for LAN or friends-only setups where faster dials matter.
- A combined lookup stack fits a game: `PkarrPublisher::n0_dns()` + `DnsAddressLookup::n0_dns()` (fast, central), plus `DhtAddressLookup` (survives n0 outages), plus mDNS (LAN parties), plus `MemoryLookup` seeded from saved-world entries that carry last-known relay and addresses.
- The `last_updated` field on resolved `Item`s, together with the 5-minute republish interval, gives a coarse "seen recently" liveness signal for free (see section 4).

### Gaps
- How long iroh-dns-server (dns.iroh.link) keeps a record after the last publish. I found no retention policy, rate limits or terms. This determines how soon a stopped world "disappears".
- `DhtAddressLookup` defaults (republish interval, relay-only filter, DHT vs relay precedence) were not on the docs page I read.
- The maximum length of `UserData` (it is defined in `iroh-dns` endpoint_info, which I did not read).
- The mDNS crate's behaviour and its privacy (it broadcasts the EndpointId on the LAN).
- Whether the iroh client automatically reconnects after `SameEndpointIdConnected`, which could cause a reconnect ping-pong between two instances.
- Official guidance or benchmarks for running many `Endpoint`s in one process.

## 2. Deriving per-world keys from an account seed, and how other systems rotate a service key while keeping a stable name

### Takeaway
Derive world keys with hardened-only derivation from the account seed, for example SLIP-0010 Ed25519 paths or HKDF with a domain-separated label and a world index. In every system surveyed, the address is the public key: Tor v3, IPNS, pkarr, iroh, and Hyperswarm/HyperDHT. Changing the key changes the address. A stable name across rotation needs one level of indirection: a long-lived "name" key, or a DID, that signs a pointer to the current operational key. That is the pattern of Tor's offline identity key and of DID verification-method rotation.

### Cited Findings
**Deriving keys**
- SLIP-0010 for Ed25519 supports only hardened derivation: children come from private parent keys via HMAC-SHA512 over the chain code with a 0x00-prefixed private key. Non-hardened derivation fails for this curve. Spec status is Draft. Libraries (e.g. vti-common) treat a non-hardened index as an error. One fork of the spec describes non-hardened Ed25519 and contradicts the official text; use the satoshilabs/bcndev text. — [SLIP-0010 (bcndev mirror)](https://github.com/bcndev/slips/blob/master/slip-0010.md); [vti-common slip10](https://docs.rs/crate/vti-common/latest/source/src/slip10/mod.rs)
- Precedent for HKDF label chains: the Nostr "Identity Chain" NIP-41 draft derives a fixed chain of 1024 internal keys via HKDF, so that rotation events can be verified without trusted timestamps. — [Appollo41/NIP-41](https://github.com/Appollo41/NIP-41)

**Tor onion v3**
- "A hidden service's name is its long term master identity key". The address is the whole key in Base32 with a version byte and checksum, plus ".onion" (56 characters). — [Tor rend-spec overview](https://spec.torproject.org/rend-spec/protocol-overview.html)
- Key hierarchy: the master identity key "is long term and not used on its own to sign anything". It derives per-period blinded signing keys, which sign descriptor signing keys that are stored online. The identity key "may be stored offline", and a thief of the online keys "can only impersonate it for a limited period of time". The spec notes "offline keys are not implemented as of 0.3.2.1-alpha". — same source
- Blinding: given the master public key and a period nonce, anyone can derive the blinded public key, but blinded keys cannot be linked to the master key without knowing it. — [Tor key blinding](https://spec.torproject.org/rend-spec/keyblinding-scheme.html)

**IPNS**
- The IPNS name is "a Multihash of a serialized PublicKey" (CIDv1, libp2p-key codec, Base36). Records carry Value, Validity (EOL), Sequence and TTL (suggested 5 min). DHT records should expire within 48 h, and "DHT peers only keep records for up to 48 hours". "Whoever controls the private key has full control over the name." The spec has no rotation mechanism. — [IPNS record spec](https://specs.ipfs.tech/ipns/ipns-record/)

**pkarr**
- The README does not address key rotation; the name is the key. — [pubky/pkarr](https://github.com/pubky/pkarr)

**Holepunch (Hyperswarm/HyperDHT)**
- HyperDHT identifies peers by public key, which is looked up in the DHT to find IP and port. Hyperswarm topics are 32-byte values, usually a Hypercore `discoveryKey`, and joining "will leak the topic to DHT nodes". — [Pears docs: Hyperswarm](https://docs.pears.com/building-blocks/hyperswarm); [HyperDHT how-to](https://pear-docs-preview-iimt7-commit-393c3b3.kinsta.page/how-to/connect-to-peers/connect-two-peers-by-key-with-hyperdht/)

**Nostr**
- Rotation is unresolved. PR #829 proposed kind 1776/1777 migration events with OpenTimestamps and a 30-day contest window; reviewers suggested reframing it as "identity recovery". A separate "Identity Chain" draft uses kind 1041. In December 2025 the W3C public-nostr list asked whether any of this work was still active. — [nips PR #829](https://github.com/nostr-protocol/nips/pull/829); [NIP-41 draft](https://github.com/Appollo41/NIP-41); [public-nostr Dec 2025](https://lists.w3.org/Archives/Public/public-nostr/2025Dec/0000.html)

**W3C DID**
- The DID stays fixed while verification methods change. A verification method that is not in the latest DID document "is considered invalid or revoked", and each DID method defines how. A controller change through key ownership "might be indistinguishable from a standard key rotation". Service endpoints are "a network address ... at which services operate on behalf of a DID subject". — [W3C DID Core 1.0](https://www.w3.org/TR/did-1.0/)

### Inferences
- Recommended derivation: `world_sk = SLIP10_ed25519(account_seed, m/<purpose>'/<world_index>'/<key_generation>')`. Alternatively, `HKDF-SHA256(ikm=account_seed_bytes, info="chiacraft/world/v1/" || world_index || "/" || generation)` fed into `SecretKey::from_bytes`. Keep a `generation` component so a compromised world key can be replaced without touching the account key. Never derive the account endpoint key and world keys from the same path. Store the world index in the world's metadata so the key can be re-derived from the 24 words.
- Derivation ties the world to the account seed. That conflicts with "worlds may later be owned by a Chia DID or run on a headless server". The headless server would need the world private key, which is fine if it is a derived child, but it reveals nothing about the seed only if derivation is hardened (it is).
- What breaks when the key changes: every saved-world entry, every ticket or QR, every pkarr/DNS record (the name itself changes), and any friend's petname mapping. Mitigation is a two-level design:
  - (a) A stable world name key `W`, derived or DID-controlled and rarely online. It publishes a small pkarr record (TXT) naming the current operational `EndpointId` `E`, with a signed delegation and an expiry. This mirrors Tor's identity, blinded and descriptor key chain.
  - (b) Clients bookmark `W`. They resolve `W` to `E`, then let iroh resolve `E` to relay/addresses.
  - Because BEP 44 lets anyone re-announce a signed item, the server hosting `E` can keep `W`'s record alive on the DHT without holding `W`'s private key, until `W` publishes a higher seq.
  - Cost: one extra lookup on join. The alternative is to accept "address = endpoint key" (simplest, what iroh does natively) and treat rotation as rare, with a signed "moved to" notice served by the old key.
- If a Chia DID owns the world, the DID document's service endpoint could list the world's current EndpointId(s), so DID key rotation replaces level (a). Verify against Chia DID specifics; not researched here.

### Gaps
- Chia DID method specifics (how service endpoints and verification methods are updated on-chain). Out of scope for the sources I read.
- How Hypercore/Keet handle key compromise. I found no rotation mechanism; the Hypercore key is fixed and multi-writer is handled by Autobase, which I did not verify.
- Whether iroh-dns-server accepts CNAME/TXT indirection records from arbitrary keys (pkarr allows them; iroh's server parses `_iroh` TXT, and other records are unverified).
- I did not reach the text of DID Core sections 9.7 (rotation), 9.9 (recovery) and 9.16 (persistence); the quotes above come from the appendix and definitions.

## 3. What an address should look like to users, and how games and P2P apps keep friends and saved-server lists

### Takeaway
Raw keys cannot be both human-readable and secure without a central registry (Zooko's triangle). Best practice is a petname system: show a user-chosen or self-proposed name, keep the key underneath, and share by QR or link rather than by typing. Games keep a local saved-server list, ping each entry for live status, and use a friend or presence channel for "join". Optional central lists (Luanti, Minecraft listings) are for discovery only, not identity.

### Cited Findings
**Petnames**
- Names can be at most two of decentralized, globally unique and human-meaningful. DIDs and .onion addresses are unique and decentralized but unreadable: "If we ever show a DID to a user we have failed." — [Lemmer-Webber et al., Petnames paper](https://files.spritely.institute/papers/petnames.html)
- A petname system has three kinds of name: petnames (local, user-set), edge names (names supplied by contacts, shown as paths such as "Uncle Bob ⇒ Sarah Smith") and proposed names (self-asserted, such as a profile display name). — same source
- UI guidance:
  - Show petnames first, then edge names.
  - Render proposed names distinctly, for example "Pizza Piano.2" or "Pizza Piano…", so they cannot impersonate a petname.
  - Treat bookmarks as petnames.
  - Guard against bait-and-switch links.
  - Prefer QR scanning over typing; typing a large cryptographic name is "far more dangerous".
  - Edge names can be shared as a signed list.
  — same source
- The pkarr FAQ likewise suggests building petname systems and phonebooks on top of key-names. — [pubky/pkarr](https://github.com/pubky/pkarr)
- pkarr keys work as domain-style names ("domain names that you truly own"). — same source

**Saved and public server lists**
- Minecraft Server List Ping (SLP): the client queries each saved server directly on its game port and gets the MOTD, online and max player counts, and version. Servers may disable SLP. Open-to-LAN uses multicast `[MOTD]`/`[AD]` announcements. — [wiki.vg Server List Ping](https://wiki.vg/Server_List_Ping)
- Luanti public list: a server opts in with `server_announce = true` and POSTs JSON to `<serverlist_url>/announce`. The JSON includes name, description, version, protocol range, game id, player counts and flags; player names are sent only if enabled. `server_address` must be a domain that points at the server. Players browse the list under "Join Game" or at luanti.org/servers. — [Luanti source serverlist.cpp (mirror)](https://gitcode.com/gh_mirrors/mi/minetest/blob/master/src/server/serverlist.cpp); [Luanti docs: setting up a server](https://docs.luanti.org/for-server-hosts/setup/)
- Luanti liveness can be probed by connecting and disconnecting after `TOCLIENT_INIT`. — [Luanti network protocol](https://docs.luanti.org/engine/network-protocol)

**Steam presence**
- Steam rich presence: setting the `connect` key, a UTF-8 command-line string, enables "Join Game" in friends lists. If the game is not running, Steam launches it with that string; if it is, Steam fires `GameRichPresenceJoinRequested`. A user can set up to 20 keys. — [Heathen KB: Rich Presence](https://kb.heathen.group/steam/multiplayer/rich-presence); [Unity Steamworks API](https://docs.unity3d.com/Packages/com.unity.steamworks-api@1.0/api/Unity.Steamworks.Interop.GameRichPresenceJoinRequested_t.html)

**Keet/Holepunch**
- Rooms rest on Hyperswarm topics and HyperDHT public keys (see section 2). I found no primary documentation of Keet's invite format. — [Pears docs: Hyperswarm](https://docs.pears.com/building-blocks/hyperswarm)

### Inferences
- Use a three-layer user-facing model:
  - (1) The canonical address is a world key encoded like iroh/pkarr (z-base32 or base32), in a link such as `chiacraft://world/<world-key>?name=<proposed>&relay=<hint>`. Hints are optional and the key alone must suffice.
  - (2) Share via QR or deep link, never by typing.
  - (3) The local "Worlds" list stores per entry: world key, petname, proposed name (shown distinctly), who shared it (an edge name, e.g. "via Alice"), last-known relay and addresses (to seed `MemoryLookup`), last seen, and an optional owner key or DID.
- Keep the current ticket format (`endpoint-id;relay;addrs`) as a "fast hint" form of the link. An iroh ticket is essentially key plus hints, so key-only links keep working as long as pkarr lookup does.
- Friends' world lists map naturally to signed edge-name lists. Each player publishes a signed "my worlds / favourite worlds" list, either served over an iroh ALPN on the account endpoint or as a small pkarr TXT pointer. That reproduces Steam-style "Join friend" without a central service.
- Never present a world's self-proposed name as trusted. Two worlds named "Spawn City" must be visually distinguishable, for example by a key-fingerprint suffix or identicon.

### Gaps
- The exact iroh ticket encoding in 1.x (crate name and string prefix). I did not verify an `iroh-tickets` 1.x API.
- Minecraft `servers.dat` format (local NBT list) was not confirmed from a source.
- Keet invite and room-link format: no primary source found.
- Steam official (Valve partner) docs were not retrieved; the Steam claims rely on third-party SDK documentation.

## 4. Presence and liveness: how a client knows a world is online without a central service

### Takeaway
There is no global "online" bit. The standard decentralised approach combines three things: a freshly republished signed record as a hint, a direct status ping when the world list is opened as the truth, and push presence from friends who are connected. Record freshness is coarse (minutes for n0 DNS, about 1-2 hours for the DHT), so only a direct ping is authoritative.

### Cited Findings
- iroh republishes endpoint info every 5 minutes even when nothing changed, and resolved `Item`s carry an optional `last_updated` timestamp. — [iroh pkarr.rs](https://docs.rs/crate/iroh/1.0.0/source/src/address_lookup/pkarr.rs); [iroh address_lookup.rs](https://docs.rs/crate/iroh/1.0.0/source/src/address_lookup.rs)
- On the DHT, items may expire 2 hours after the last announce and should be re-announced hourly. Any node can re-announce a signed item, so a record's presence does not prove its publisher is running. — [BEP 44](https://bittorrent.org/beps/bep_0044.html)
- pkarr clients and relays cache aggressively, so a stale record can be served after the publisher has gone. — [pubky/pkarr](https://github.com/pubky/pkarr)
- IPNS suggests a 5-minute TTL as a caching hint; DHT peers keep records up to 48 h. — [IPNS spec](https://specs.ipfs.tech/ipns/ipns-record/)
- Minecraft gets liveness by pinging each saved server directly (SLP), which returns MOTD, player counts and version. Luanti's servers push announcements to a central list; a direct probe is a connect followed by disconnect after `TOCLIENT_INIT`. — [wiki.vg SLP](https://wiki.vg/Server_List_Ping); [Luanti protocol](https://docs.luanti.org/engine/network-protocol)
- Steam presence is push-based: the client sets rich-presence keys and friends see them. — [Heathen KB](https://kb.heathen.group/steam/multiplayer/rich-presence)

### Inferences
- Implement a tiny status protocol on its own ALPN (e.g. `chiacraft/status/1`) on every world endpoint. It should return world name, version and protocol range, player count, mod or package manifest hash, and whether joining is open. When the Worlds screen opens, dial every saved entry in parallel with a short timeout, like Minecraft SLP.
- Use iroh's 0-RTT/QUIC and relay fallback so pings work behind NAT.
- Treat any pkarr or DNS record with no fresh `last_updated` (roughly over 10 minutes for n0 DNS) as "probably offline" and skip or de-prioritise the dial.
- Optionally put a short status digest (players, version) in `EndpointData` user data so it shows without a dial. Stay within the pkarr 1000-byte packet limit, and remember it is public.
- For friends, keep an account-endpoint gossip channel (e.g. iroh-gossip, or direct connections among online friends) that pushes "I'm hosting world X / I'm in world Y". This gives Steam-like "Join" without a server.
- A headless DID-owned server simply keeps its world endpoint and publisher running. A world hosted only by its owner's client is online only while that client runs, which matches the Phase 3 "your world runs on while you visit others" model. The UI should show "last seen" from saved state and from record freshness.
- An optional public world directory, like Luanti's list, can be layered on later as a pkarr-keyed or gossip index. It should never be a requirement for joining by key.

### Gaps
- No source quantified dns.iroh.link record retention, so the exact "disappears after X" time for a stopped world is unknown.
- I found no iroh-native presence or status protocol; the status-ALPN design above is an inference.
- I found no source on how Keet shows room or peer online status.
