# Abuse, spam and safety risks for decentralised links and travel between player-run worlds

Context: chiacraft (Rust, iroh P2P QUIC, Ed25519 player keys from 24-word accounts, Chia DIDs later). World owners send link requests (inbox; accept or pre-allow); accepted links become portals. No central moderation; owners decide, defaults open, players must be safe. Research done 2026-10-31. Sources are 2023-2026 unless marked **[older]**.

## 1. Request spam and harassment in decentralised networks: what worked

### Takeaway
Every open "anyone can address you" inbox in a decentralised network (Matrix invites, Fediverse open sign-ups, Nostr, email) has been flooded, and the fixes that worked in practice were **closing the default** (approval mode, invite toggles, allow-lists), **proactive filtering by a party the community chose** (Matrix policy servers, Nostr web-of-trust relays), and **subscribable shared denylists**. Reactive blocklists alone always let the first wave through, and proof-of-work on its own is widely judged too weak against botnets unless it is weighted by reputation.

### Cited Findings

**Matrix (invite spam is the closest analogue to link requests)**
- Matrix DMs are rooms, so accepting DMs from new people means accepting room invites; the invite endpoint takes only a user ID, and IDs are semi-public (visible to anyone sharing a room). The protocol cannot tell a friend's invite from a spammer's — [Frederickson, "My thoughts today on Matrix and spam", 10 Aug 2025](https://www.terracrypt.net/posts/matrix-spam.html)
- August 2025 invite-spam wave: invites carried hateful content in the user IDs and room names themselves, i.e. the abuse is in the request metadata, before any acceptance — [Frederickson, Aug 2025](https://www.terracrypt.net/posts/matrix-spam.html)
- Shared policy lists and Draupnir are "largely reactive", comparable to email RBLs: they only act after recipients have received and reported spam; IP-reputation measures are crude and hit innocent users; shared lists add operational load — [Frederickson, Aug 2025](https://www.terracrypt.net/posts/matrix-spam.html)
- Proposed alternative: **capabilities / invite codes** — a per-person or per-room token that both names the recipient and grants the right to send; withholding tokens acts as an allow-list, and incoming requests reveal which token they used (so a leaked token can be traced and revoked). Medium-term idea: an optional or mandatory invite-code parameter on the invite endpoint — [Frederickson, Aug 2025](https://www.terracrypt.net/posts/matrix-spam.html)
- Ubuntu Matrix incident, 13 Sep–29 Oct 2024: spammers invited users of Ubuntu rooms into a room containing illegal content. Response: users "Reject and ignore user", auto-ignore known spammers via ban lists; operators banned accounts and **blocked the whole homeserver** used for the abuse — [Ubuntu advisory, 2024](https://discourse.ubuntu.com/t/advisory-spam-invite-incident-2024-09-13/48034)
- April 2025: Matrix.org Foundation reported "ongoing attacks on Matrix" and introduced **policy servers** (MSC4284): rooms opt in to a community-chosen policy server; servers send each event to it for evaluation before showing it to users; events that fail never reach clients. Described as "an overlapping layer of protection" next to moderation bots (Draupnir, Mjolnir, Meowlnir), not a replacement — [Matrix.org, 17 Apr 2025](https://matrix.org/blog/2025/04/introducing-policy-servers/)
- policyserv v1.0.0 (18 Dec 2025, open source): since April 2025 it evaluated "over 5 million events" in protected rooms, with a "marked reduction in harmful content" (no further metrics). Example filter: no dozens of mentions in a single message — [Matrix.org, Dec 2025](https://matrix.org/blog/2025/12/policyserv/)
- Matrix spec v1.18 (26 Mar 2026) standardised policy servers, an **invite-blocking on/off toggle** (MSC4380, explicitly "a minimal first step"; finer blocking by server, user or Space membership left to MSC4155), standard account suspension/locking APIs, and reporting changes: report endpoints may return 200 whether or not the target exists, or add random delay (to avoid leaking existence) — [Matrix v1.18 release](https://matrix.org/blog/2026/03/26/matrix-v1.18-release/)
- MSC4155 (invite filtering): an account-data event saying which users or servers may invite you, with allow-list or block-list defaults; clients hide blocked invites. Implemented experimentally in Synapse and Continuwuity — [MSC4155 text](https://github.meowingcats01.workers.dev/Johennes/matrix-spec-proposals/blob/johannes/invite-filtering/proposals/4155-invite-filtering.md); [Continuwuity PR #2194](https://forgejo.ellis.link/continuwuation/continuwuity/pulls/2194)
- Server-side options: Tuwunel's `block_non_admin_invites` restricts remote invites to admins — [Tuwunel moderation docs](https://matrix-construct.github.io/tuwunel/moderation.html)

**Fediverse**
- February 2024 wave: automated scripts mass-created accounts on small and **abandoned** Mastodon/Misskey servers with open registration. Eugen Rochko advised approval-mode registration and blocking disposable email; Mastodon **changed new instances to not-open by default** and added a banner that fully open instances must be actively moderated — [TechCrunch, 20 Feb 2024](https://techcrunch.com/2024/02/20/spam-attack-on-twitter-x-rival-mastodon-highlights-fediverse-vulnerabilities/embed/); [TechRadar](https://www.techradar.com/pro/mastodon-is-being-hit-with-an-increasing-number-of-spam-attacks-and-there-doesnt-seem-to-be-a-fix-any-time-soon)
- Admins aggregated and shared a list of instances hosting the bots; many chose to **limit** rather than fully defederate — [Lemmy admin post, 2024](https://lemmy.ndlug.org/post/713985); [Fediverse Report](https://fediversereport.com/?p=1780) (secondary; dates vary 15/16 Feb)
- FIRES (Fediverse Intelligence Recommendations Replication Endpoint Server, NLnet-funded 2024-02 to 2026-01): protocol for publishing moderation data over time as **advisories that can escalate to recommendations**, labelled with common vocabularies; covers actors, hashtags and links, not only domains; even "drop" is a suggestion to consuming software, not an automatic block — [FediMod/fires GitHub](https://github.com/FediMod/fires); [NLnet project page](https://nlnet.nl/project/FediMod/)
- IFTAS (Jul 2026) is retiring its hosted denylist service in favour of **downloadable and subscribable denylist files**, crediting FIRES — [IFTAS, 30 Jul 2026](https://about.iftas.org/2026/07/30/deprecation-notice-cariad-domain-observatory-transition/)

**Bluesky / AT Protocol (shared lists without one central moderator)**
- "Stackable" moderation: network takedowns, labels from user-chosen labelers, and user mutes/blocks; app hardcodes Bluesky's own labeler and allows ~9 more subscriptions — [Bluesky blog, Mar 2024](https://bsky.social/about/blog/03-12-2024-stackable-moderation); [Bluesky moderation docs](https://www.docs.bsky.app/docs/advanced-guides/moderation)
- Subscribable block lists add members to your blocks without touching personal blocks; risk noted that an untrustworthy list owner can add people out of revenge, getting them blocked "for everyone" subscribed — [Virtual Curiosities guide](https://www.virtualcuriosities.com/?p=2957) (secondary)

**Nostr**
- NIP-13: PoW = number of leading zero bits in the event id (nonce tag); cheap for any relay/client to verify; because the id does not commit to the signature, PoW can be **outsourced** to PoW providers (helps phones, but also means money buys PoW) — [NIP-13](https://nips.4rs.nl/nips/13)
- Practitioner pattern: high PoW only for new/unknown keys, no PoW for keys inside the user's web of trust. Critics: no threshold stops botnets without hurting phone users; "10 minutes of intensive computing on a smartphone" makes it unusable. Informal forum evidence only — [nostr post (pro)](https://nostr.lucianmarin.com/post/408e3d0cc6e740103282f0f01bd35fe2b0796dfcde7dcfcaa72e25b827b93ab0); [nostr post (contra)](https://nostr.lucianmarin.com/post/cf038bcac7c9d355426bd56b7d9bb01e281c392016cdc2ea256dd430841a74c7)
- Web-of-trust filtering: trust scored from kind-3 follow lists; a spammer buying 10,000 fake follows still cannot reach a user whose own contacts vouched for none of them. Shipped client-side in Amethyst, Damus, Coracle, NDK, Flotilla; Vertex serves personalised PageRank. Caveats: computation currently mostly on centralised services (Vertex, Nostr.Band), and public follow lists **leak the social graph** — [Damus article](https://damus.io/naddr1qqgrwd34vvek2wf5ve3nwcmrxycrxq3qklkk3vrzme455yh9rl2jshq7rc8dpegj3ndf82c3ks2sk40dxt7qxpqqqp65wrp5t4t); [OpenSats WoT topic](https://opensats.org/topics/web-of-trust)
- WoT relay spec (ORLY/Brainstorm) combines hops, GrapeRank confidence, verified follower counts and NIP-56 report counts, published as NIP-85 kind 30382 events — [WOT_SPEC](https://git.imwald.eu/silberengel/next.orly.dev/src/branch/main/pkg/neo4j/WOT_SPEC.md)

**Email / proof-of-work economics [older]**
- **[older, 2004]** Laurie & Clayton, "Proof-of-Work Proves Not to Work": a scheme where every email carries PoW is not viable; spammers can afford the compute and botnets remove the cap — [paper](https://www.cl.cam.ac.uk/~rnc1/proofwork.pdf)
- **[older, 2006]** Liu & Camp, "Proof of Work can Work": PoW stays effective if the work required is **weighted by a reputation function** (known senders pay little, unknown pay a lot) — [paper](https://sites.cs.ucsb.edu/~rich/class/cs293b-cloud/papers/lui-pow.pdf)

### Inferences
- A chiacraft link request is structurally a Matrix invite: unsolicited, addressed by a public key, and carrying attacker-chosen text (world name, message, icon). The request metadata itself is an abuse vector, so the inbox should render unknown requests collapsed (no images, no rich names) until the owner opens them.
- The Matrix trajectory (open invites -> reactive lists -> proactive policy servers -> invite toggle -> allow/deny by server/user) suggests shipping layers from day one: (a) per-world toggle "accept requests from: anyone / friends-of-linked-worlds / nobody", (b) local block per key, (c) optional subscription to shared denylists, (d) a per-sender rate limit.
- Capability-style **link invite codes** (owner hands out a token; a request carrying a valid token skips the "unknown" queue) fit P2P well and map to "pre-allow". They give traceability: a token used by a spammer reveals who leaked it.
- "Defaults open" is in tension with the Mastodon lesson (open-by-default instances became spam farms; Mastodon reversed the default). A workable reading: open to *receiving* requests, but unknown requests are quarantined, rate-limited and never auto-accepted.
- PoW is useful as a **cost on unknown senders only** (Liu & Camp / Nostr WoT pattern) and as DoS friction for the inbox, not as primary anti-spam.
- Denylists in chiacraft should be FIRES-like: signed, published by anyone, advisory, labelled with reasons; the client subscribes to a few, and an entry from one list should hide/quarantine rather than hard-block (Bluesky revenge-listing risk).

### Gaps
- No quantitative effectiveness data was found for Nostr PoW or WoT filtering, or for Matrix invite blocking; Matrix gives only "5 million events evaluated" and "marked reduction".
- Exact default of the Matrix v1.18 invite toggle was not stated in the release post.
- Did not find post-mortems of decentralised games (rather than chat) facing request spam.

## 2. Sybil resistance and ban evasion

### Takeaway
Cheap keys (24-word accounts) mean any per-key block or rate limit is trivially evaded; the literature groups defences into trusted certification, resource cost, and social-graph trust, and games show that even "hardware" identity is spoofable. The practical answer is to make new or unknown identities **low-privilege by default** and let trust accrue (age, vouching, on-chain cost), rather than to try to stop key creation.

### Cited Findings
- **[older, 2013]** Sybil defences fall into three families: trusted certification, resource testing (cost), and social networks — [Mohaisen & Kim survey, arXiv 1312.6349](https://arxiv.org/pdf/1312.6349)
- **[older, 2014]** Social-graph defences (SybilGuard, SybilLimit and successors) rely on the sparse cut between honest and Sybil regions and on random-walk theory — [Alvisi et al., SoK: Evolution of Sybil Defense via Social Networks](https://research.google/pubs/sok-the-evolution-of-sybil-defense-via-social-networks/)
- A reputation system's vulnerability depends on how cheaply identities can be made, whether it accepts input from entities without a chain of trust to a trusted one, and whether it treats all entities identically — [Wikipedia: Sybil attack](https://wikipedia.com/wiki/Sybil_attack)
- VRChat's Trust Rank (Visitor -> New User -> User -> Known User -> Trusted User) reflects time in game, content contributions, friendships and hidden factors; visitors cannot upload content until promoted; the formula is kept secret to prevent gaming. Introduced 2018 **[older]** in response to griefing — [VRChat wiki: Trust Rank](https://wiki.vrchat.com/wiki/Trust_Rank); [Ryan Schultz, 2018](https://ryanschultz.com/2018/09/29/vrchat-institutes-a-new-safety-and-trust-system-to-combat-griefers)
- Academic study of VRChat's trust system: users become "way too obsessed with rank" (status effects of visible trust tiers) — [EUSSET paper](https://dl.eusset.eu/items/725fc58b-700e-4dda-9a66-a633d0d54ab6/full)
- Games use hardware-ID bans (Riot League of Legends restricts all accounts tied to banned hardware; Apex/Respawn HWID bans), but HWIDs are spoofed by cheat tools and second-hand hardware can carry a ban to an innocent buyer — [GamesRadar](https://www.gamesradar.com/games/league-of-legends/after-nearly-15-years-league-of-legends-finally-gets-the-anti-cheat-boost-that-might-help-solve-its-massive-botting-problem); [PC Gamer](https://www.pcgamer.com/respawns-latest-anti-cheat-measures-sees-apex-cheaters-banned-by-hardware-ids/); [Cybernews](https://cybernews.com/security/second-hand-cpu-unable-to-play-valorant-hardware-hwid-ban/)
- Nostr WoT: fake follows bought in bulk do not help a spammer reach users whose own graph does not include them — [Damus article](https://damus.io/naddr1qqgrwd34vvek2wf5ve3nwcmrxycrxq3qklkk3vrzme455yh9rl2jshq7rc8dpegj3ndf82c3ks2sk40dxt7qxpqqqp65wrp5t4t)
- Fediverse responses targeted the cheap-identity source (block disposable email, approval mode) and the hosting server (limit/defederate instance), i.e. ban the **issuer**, not just the identity — [TechCrunch, Feb 2024](https://techcrunch.com/2024/02/20/spam-attack-on-twitter-x-rival-mastodon-highlights-fediverse-vulnerabilities/embed/); [Ubuntu advisory](https://discourse.ubuntu.com/t/advisory-spam-invite-incident-2024-09-13/48034)

### Inferences
- In chiacraft, the "issuer" equivalents are: the origin world (for arriving players), and any DID/registrar later. Blocking a world key blocks all players vouched for by it — the Matrix "block the homeserver" move.
- Key age needs a verifiable anchor to mean anything in P2P: e.g. a timestamped attestation (a Chia DID registration block height, or a co-signature from a world that has seen the key for N days). A self-claimed age is worthless.
- Chia DIDs with an on-chain cost are a "resource testing" Sybil defence; they should raise trust (skip PoW, enter the "known" queue) rather than be required, so free players still work.
- Graduated trust per world (visitor -> resident -> trusted) mirrors VRChat and avoids a global reputation score that would need a central service; keep any score local/private to avoid rank obsession and gaming.
- HWID-style bans are not available or desirable in an open-source P2P client; accept that bans are per key/world and rely on low default privileges for new keys.

### Gaps
- No 2023–2026 survey of Sybil resistance specific to P2P games was found; the main academic sources are 2013–2014.
- No source found on how much on-chain identity cost (e.g. DID fee) actually deters game griefers.

## 3. Portal and travel abuse, and what existing virtual worlds do

### Takeaway
Existing worlds converge on: **destination-controlled admission** (OpenSim Gatekeeper allow/deny of foreign grids; Second Life ban lists), **owner-set arrival points** (SL landing point / teleport routing), **limited exposure of the traveller's belongings to the foreign host** (OpenSim HG 1.5 suitcase), **portal previews and timeouts** (VRChat shows world, dropper, instance type, 30 s timer), and **proxies with authenticated forwarding plus anti-bot filters** for join floods (Minecraft Velocity). Data carried by the client across a teleport must be treated as replayable (Roblox).

### Cited Findings

**OpenSimulator Hypergrid (closest analogue: independent grids linked by travel)**
- Gatekeeper setting `ForeignAgentsAllowed` with exceptions: if true, deny-list via `AllowExcept`; if false, allow-list via `DisallowExcept` (comma-separated grid addresses). Parallel outbound controls by user level in UserAgentService: `ForeignTripsAllowed_Level_N`, `AllowExcept_Level_N`, `DisallowExcept_Level_N` — [example config (pastebin)](https://pastebin.com/GH3p6cU9); [StandaloneCommon.ini.example 0.7.5](https://sledjhamr.org/cgit/opensim-SC/plain/bin/config-include/StandaloneCommon.ini.example?h=0.7.5) **[older config examples]**
- Gatekeeper can filter incoming foreign agents, e.g. "anonymize all foreign user agents and strip them off their clothes before allowing them in" — [opensim-dev list](https://mail-archive.com/opensim-dev@lists.berlios.de/msg05022.html) **[older]**
- Hypergrid 1.5 locked foreign grids out of a traveller's inventory except one "My Suitcase" folder, to stop content theft from visitors (some operators at the time thought theft was theoretical) — [Hypergrid Business](https://www.hypergridbusiness.com/?p=35167) **[older, ~2011]**
- 2016 security paper on HG 2.0: "none of the vulnerabilities discussed in this document are theoretical"; private invite-only grids offer the strongest content protection — [Heilmann & Maxwell, "Securing the Hypergrid"](https://www.hypergridbusiness.com/wp-content/uploads/2016/09/Securing-the-Hypergrid-Michael-Heilmann-and-Douglas-Maxwell.pdf) **[older, 2016]**; [OSGrid forum](https://forums.osgrid.org/viewtopic.php?p=15962) **[older, 2011]**
- Protocol-version splits fragmented travel (HG 1.0 grids could only reach HG 1.0, 1.5 only 1.5) — [Hypergrid Business](https://hypergridbusiness.com/?p=34376) **[older]**

**Second Life**
- Teleport routing per parcel: "Blocked" (visitors land outside), "Landing Point" (everyone arrives at one owner-set spot and orientation), "Anywhere" — [SL wiki: About Land – Options](https://wiki.secondlife.com/wiki/Viewerhelp:About_Land_-_Options)
- Ban forbids entry to parcel, region or estate; a banned avatar teleporting in is sent to the nearest allowed spot; bans scriptable via `llAddToLandBanList` — [SL wiki: Banlist](https://wiki.secondlife.com/wiki/Banlist); [LSL reference](https://create.secondlife.com/script/lsl-reference/functions/lladdtolandbanlist/)
- Community anti-griefing recipe: name bans, No Fly, fixed teleport routing so griefers cannot skip past defences by local teleport; but clients can circumvent No Fly — [SL community thread](https://community.secondlife.com/forums/topic/500592-griefing-protection-tools/)

**VRChat**
- A dropped portal shows others: world name, **who dropped it**, region, instance number, instance type, hosting group, capacity, platform compatibility, and a 30-second countdown after which it disappears; portals to Friends instances are not shown to non-friends — [VRChat wiki: Portals](https://wiki.vrchat.com/wiki/Portals)
- Feature requests (not confirmed implemented) ask for group-membership or trust-rank gating of portal placement, implying portal spam/griefing is a real concern — [VRChat feedback](https://feedback.vrchat.com/udon/p/vrc-group-instances-portal-placement-permissions)
- Safety controls: Shield levels with per-trust-rank restrictions (voice, avatar, audio, lights/particles, shaders, animations…); one-tap **Safe Mode** (non-friend avatars become a plain robot; warns "No Safety" exposes you to crash attempts); kick votes (New User+ rank, 1-hour kick); instance moderators can kick/ban and disable items, drones etc.; block hides users from each other (global scripts may still leak presence) — [VRChat wiki: Trust and Safety](https://wiki.vrchat.com/wiki/Trust_and_Safety)
- Worlds go through "Community Labs" before public release — [VRChat wiki: Trust and Safety](https://wiki.vrchat.com/wiki/Trust_and_Safety)

**Minecraft server networks**
- Legacy BungeeCord IP forwarding passes the player's IP/UUID in the handshake **without authentication**: anyone who can reach a backend directly can spoof any UUID (e.g. an admin's); backends must be firewalled to the proxy — [MineGuard](https://mineguard.pro/en/blog/velocity-ddos-protection-minecraft-network-guide) (vendor source)
- Velocity "modern" forwarding signs forwarded player data with HMAC-SHA256 using a shared secret (`forwarding.secret` / Paper `paper-global.yml`); backend rejects unsigned joins. PaperMC's patch states legacy forwarding "is not secure" — [PaperMC Velocity forwarding patch](https://ayakael.net/mirrors/papermc/blame/commit/26325213c781854022887d95769c02e73c7e8bd1/patches/server/Add-Velocity-IP-Forwarding-Support.patch)
- Anti-bot proxies (e.g. Sonar for Velocity/BungeeCord/Bukkit) add verification layers to incoming connections to filter automated joins, packet floods and fake players — [Sonar listing](https://www.boxtoplay.com/en/minecraft-hosting/minecraft-server/modrinth-plugins/sonar-49697) (marketing text only)

**Roblox**
- Teleport data travels through the client; it is integrity-checked (tampered data arrives as nil), but can be **replayed** — community advice to assume a client can roll back teleport data up to 48 hours, and clients can create their own teleports with fake data. Recommended: only immutable, non-critical data in teleport payloads; valuable state in server-side stores — [Roblox DevForum](https://devforum.roblox.com/t/to-what-extent-can-exploiters-modify-teleportdata/2794032); [DevForum](https://devforum.roblox.com/t/what-is-the-best-way-to-protect-my-teleport-data/738908) (community, not official)

### Inferences (threat -> mitigation for chiacraft)
- **Griefers arriving through portals** -> destination world decides admission per arrival (allow/deny by origin world key and player key, OpenSim-style `AllowExcept`/`DisallowExcept`), new arrivals get a "visitor" role (no build/break, no chat links, no item drops) until promoted.
- **Trapping or camping the arrival point** (lava, enclosed box, spawn-kill, offensive builds at the landing spot) -> the *destination* owner sets the landing point (SL), but the *arriving client* should enforce a short arrival grace (invulnerable, can't be pushed), verify the landing spot is safe (not inside blocks, not over void) and always offer "go back" / "go home" that the host cannot disable.
- **Linking to offensive or malicious worlds** -> a link request names the target world; the accepting owner sees target name, owner key/DID, package list and existing labels before accepting. Shared labels/denylists (FIRES-style) can flag worlds. Players see the same preview at the portal (VRChat shows world + dropper + instance type).
- **Forced downloads** (portal leads to a world whose packages are huge, unsigned, or contain unwanted content) -> never download on step-through without consent; show total size, signer and trust state of each package; reuse the existing package-signing/trust work; HG 1.5 "suitcase" idea suggests only exposing a minimal profile/inventory to foreign hosts.
- **DoS by many arrivals** -> per-origin-world and per-key arrival rate limits, a capacity cap, and a lightweight pre-admission check (Sonar-style) before the world loads chunks for the visitor; optional PoW for arrivals from unknown origins.
- **Impersonating a world** -> worlds are addressed by key, not name; portals bind to the destination world key, and the client checks the QUIC/iroh peer key matches before showing "Arrived at X". Display names must be shown with a key fingerprint or petname (owner's local nickname) to resist look-alike names.
- **Revoked link still used** -> links should be signed records with an expiry and a revocation that either side can publish; the destination checks its own current allow-list at arrival time (destination-side check is authoritative, so a stale portal on the origin side fails safely).
- **Spoofed identity via a relay/proxy** (Minecraft BungeeCord lesson) -> never trust identity asserted by an intermediary; the arriving player must prove key possession to the destination end-to-end.

### Gaps
- Could not load the current OpenSimulator wiki (TLS error), so current Hypergrid defaults and precedence of `ForeignAgentsAllowed` vs exception lists are unverified; config evidence is from older examples.
- No official VRChat source on who may drop portals or rate limits on portal drops.
- No primary source found for Roblox's teleport-data signing; only community understanding.
- No independent evaluation of Minecraft anti-bot plugins.

## 4. Identity of an arriving player, replay protection, and IP privacy

### Takeaway
Use end-to-end key proof (the iroh/QUIC connection already authenticates an Ed25519 endpoint key) plus a short-lived signed travel ticket bound to player key, origin world, destination world, nonce and expiry; never trust an intermediary's claim of identity. Direct P2P connections expose IP addresses to the other side, so arriving into an untrusted world reveals the player's IP unless traffic is relayed.

### Cited Findings
- iroh uses Ed25519 keys for endpoint identity and encryption by default; traffic is end-to-end encrypted and relays cannot read it — [iroh docs: security & privacy](https://docs.iroh.computer/concepts/security-privacy)
- Relays see metadata: source/destination IPs, which pairs talk, when and how much; they are not notified when a pair switches to direct — [iroh docs](https://docs.iroh.computer/concepts/security-privacy)
- Direct connections require endpoints to exchange IP addresses; in consumer P2P apps with untrusted peers this enables location tracking and targeted attacks (e.g. DDoS on a home connection) — [iroh docs](https://docs.iroh.computer/concepts/security-privacy); a docs PR says direct connections are "fundamentally incompatible with IP address privacy" — [iroh docs PR #9](https://github.com/n0-computer/docs.iroh.computer/pull/9)
- iroh privacy options: run your own relay/DNS (public relays carry no privacy guarantees); Tor custom transport (both ends need it); upcoming **relay-only mode** (disables hole punching; relay operator still sees who talks to whom); direct connections, port mapping, local discovery and relaying can each be disabled. DNS discovery publishes only the home relay URL, not IPs — [iroh docs](https://docs.iroh.computer/concepts/security-privacy)
- Idea discussed (not shipped): delay hole punching until the application has authenticated the peer, so IPs aren't shared with unverified endpoints — [iroh docs PR #9](https://github.com/n0-computer/docs.iroh.computer/pull/9)
- Relays you run can restrict access to a device list or require a shared password — [iroh docs](https://docs.iroh.computer/concepts/security-privacy)
- Minecraft: identity forwarded by a proxy without a signature is spoofable; HMAC-signed forwarding (Velocity modern) fixed it — [PaperMC patch](https://ayakael.net/mirrors/papermc/blame/commit/26325213c781854022887d95769c02e73c7e8bd1/patches/server/Add-Velocity-IP-Forwarding-Support.patch)
- Roblox: client-carried teleport data is tamper-evident but replayable for a long window — [Roblox DevForum](https://devforum.roblox.com/t/to-what-extent-can-exploiters-modify-teleportdata/2794032)
- OpenSim HG: the traveller's home grid vouches for the user agent ("base trust" established by travelling from the home grid); foreign grids see only the suitcase — [opensim-dev, 2017](https://mail-archive.com/opensim-dev@opensimulator.org/msg01592.html) **[older]**; [Hypergrid Business](https://www.hypergridbusiness.com/?p=35167) **[older]**
- Matrix v1.18 lets report endpoints avoid leaking whether the target exists (constant 200 / random delay) — a pattern for not leaking presence — [Matrix v1.18](https://matrix.org/blog/2026/03/26/matrix-v1.18-release/)

### Inferences
- **Proving which account arrives:** the player's QUIC connection to the destination is authenticated by its endpoint key; the player key (from the 24-word account) should sign a challenge from the destination inside that session (channel binding), so a relay or origin world cannot assert identity on the player's behalf.
- **Binding to the origin world:** the origin world signs a travel ticket `{player_key, origin_world_key, dest_world_key, link_id, issued_at, expires_at (seconds to minutes), nonce}`; the player countersigns. Destination checks: link_id still allowed, both signatures, dest key = itself, not expired, nonce unseen (keep a nonce cache for the ticket lifetime). This blocks replay to a different world (dest binding), replay later (expiry), and replay twice (nonce). Lesson from Roblox: keep tickets free of valuable mutable state; inventory/progress transfer needs its own protocol.
- The origin-world signature is optional evidence ("came via an accepted link"), not authority over identity; a destination should also accept direct travel if its policy allows.
- **IP privacy:** default to relayed connections when visiting a world whose owner the player hasn't marked trusted, and only hole-punch after admission (the iroh "authenticate before hole punch" idea). Offer "always relay" as a privacy setting; warn that the relay operator sees who visits whom. Hosts face the mirror risk: every visitor learns the host's IP on direct connect.
- Don't reveal presence: a world receiving a denied arrival or a blocked link request should not tell the sender that it was blocked (Matrix reporting pattern), reducing harassment feedback loops.

### Gaps
- No source found on iroh's current relay-only mode status after 1.0 or its latency cost for real-time games.
- No existing game found that uses signed, origin-bound travel tickets in a P2P setting; the ticket design is an inference.

## 5. Safety defaults: what is off by default, what to show before stepping through, how to remember trust

### Takeaway
Platforms that started open moved toward safer defaults after abuse (Mastodon closed new instances by default; Matrix added invite blocking; VRChat added Shield/Safe Mode). The consistent pattern is: unknown parties get a restricted experience by default, the user can loosen per party, a one-tap panic control exists, and previews show who is behind a destination before you commit.

### Cited Findings
- Mastodon changed new instances to not-open by default after the Feb 2024 wave — [TechCrunch](https://techcrunch.com/2024/02/20/spam-attack-on-twitter-x-rival-mastodon-highlights-fediverse-vulnerabilities/embed/)
- Matrix v1.18 added a user-level invite-blocking toggle; MSC4155 adds allow/deny lists by user/server — [Matrix v1.18](https://matrix.org/blog/2026/03/26/matrix-v1.18-release/); [MSC4155](https://github.meowingcats01.workers.dev/Johennes/matrix-spec-proposals/blob/johannes/invite-filtering/proposals/4155-invite-filtering.md)
- VRChat: Shield presets (Maximum recommended for newcomers/unfamiliar public spaces; Normal; Custom per trust rank); Safe Mode panic button; friends exempt from Safe Mode; avatars of unknown users can be replaced by a plain robot — [VRChat wiki: Trust and Safety](https://wiki.vrchat.com/wiki/Trust_and_Safety)
- VRChat portal preview fields: world name, dropper, region, instance type, hosting group, capacity, platform compatibility, 30 s timer — [VRChat wiki: Portals](https://wiki.vrchat.com/wiki/Portals)
- Second Life: owner chooses teleport routing (Blocked / Landing Point / Anywhere) — [SL wiki](https://wiki.secondlife.com/wiki/Viewerhelp:About_Land_-_Options)
- OpenSim: inbound foreign visitors and outbound trips are configurable per grid and per user level; HG 1.5 exposes only a suitcase folder to foreign grids — [config example](https://pastebin.com/GH3p6cU9); [Hypergrid Business](https://www.hypergridbusiness.com/?p=35167) **[older]**
- Bluesky: users pick moderation services; personal blocks persist independently of list subscriptions — [Bluesky blog](https://bsky.social/about/blog/03-12-2024-stackable-moderation)
- Nostr WoT: trust derived from the user's own follow graph, not a global authority — [OpenSats WoT](https://opensats.org/topics/web-of-trust)

### Inferences (suggested defaults for chiacraft)
- **Off / quarantined by default:** auto-accepting link requests from unknown keys; rich content (images, long text, links) in unknown requests; downloading packages on step-through without a prompt; direct (IP-revealing) connections to untrusted worlds; visitors building/breaking/dropping items/posting links; voice or custom skins/shaders from unknown visitors (VRChat Shield analogue).
- **On by default:** per-key and per-world rate limits on requests and arrivals; arrival grace period and safe-spawn check; always-available "go home"/"go back"; local block of a key or a whole world; hidden-from-sender blocking.
- **"Defaults open" reconciled:** worlds accept link requests and visitors by default, but into a quarantine/visitor tier; owners can pre-allow (allow-list, invite tokens, "friends of my linked worlds"), or close.
- **Show before stepping through:** destination world name + key fingerprint/petname + owner (key, DID if any), whether this link was accepted by both sides and when, labels from subscribed lists, package list with total download size and signer trust state, connection mode (relay vs direct, i.e. whether your IP will be visible), visitor rules (can you build?), current/max players. Mirrors VRChat's preview fields plus P2P-specific items.
- **Remembering trust:** store per-player local trust records keyed by world key (trusted / visited / blocked, with first-seen date and who introduced it), petnames chosen by the player, and "always relay"/"always ask before download" overrides. Trust in a world should not automatically extend to every world it links to; at most one hop of "introduced by" as a hint (WoT-style), never as auto-acceptance.
- **Owner-side memory:** accepted links, allow-listed keys, invite tokens with expiry and usage counts, and a denylist subscription list — all exportable/signable so owners can share them as public lists.

### Gaps
- No user research found on which preview fields actually change players' decisions before teleporting.
- No source found comparing open-vs-closed defaults in game portal systems specifically (evidence is from chat/social networks).
