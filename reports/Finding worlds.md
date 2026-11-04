# Finding worlds: how players find each other's worlds with no service of ours

For Ronald (phase 5 of `notes/PLAN.md`), 2026-11-04. The research was done the same day: Chia's
full-node source and docs, the DIG Network's repositories, Chia DataLayer, iroh, Mainline DHT,
Nostr, and how Luanti, Veloren, Holepunch, Urbit, Hyphanet, Scuttlebutt, Bluesky, CATalog and
Namesdao find and filter things. Sources at the end; **[inferred]** marks our own reasoning, and
**[not confirmed]** what we looked for and couldn't confirm.

## In one paragraph

Keep the plan's shape: one open index on chain, curated lists on top, live status from the
worlds, and a ranking. But the cheap version of the index fails. A 1-mojo coin with a well-known
hint can be copied by anyone, and a flood of them costs a spammer under a dollar and about ten
minutes. Chia's own DAO proposals broke this way. So **a listing must lock a deposit** (taken
back when the world is delisted). It must **name the world's DID**, and a game shows it only
when that DID checks out both ways (it already can). The game reads the index through the light
wallet protocol with a minimum amount and spent coins filtered out on the node. Rich listings
(description, tags, screenshots) come **from the world itself** over iroh, with no DIG
dependency for now. Curated lists reuse the signed ban-list machinery. Ranking draws on the
deposit, DID age, the links between worlds, the lists a player follows and their friends.

## What it must do (the plan's rules)

1. **No service of ours, and no single one anywhere.** Any Chia full node will do, the world
   serves its own details, and anyone can keep a list.
2. **The chain only where it adds trust.** The index lives there because it is the one public,
   permanent place nobody can quietly edit. Descriptions, pictures and live status stay off it.
3. **The owner chooses.** A world is listed only if its owner lists it; unlisted worlds are still
   reachable by address, by friends and through links, as today.
4. **Players are safe.** A listing can't make the game connect anywhere or run anything:
   it is a world DID and a key, checked before it is shown.

## What we already have

- Every world has an address (its key), a status answer (`chiacraft/status/1`: name, owner,
  players, packages, admission rules, its DID), and from phase 4 a DID. The world DID's metadata
  names the owner's DID and the world's key, and the owner's DID lists the world back.
  `world_did::check` confirms both ways on chain.
- Players already find worlds three ways without the chain: by address, through friends
  (`chiacraft/worlds/1` lists a friend's worlds), and through portal links. Each world knows the
  worlds it is linked to.
- Signed documents served by any world over iroh, followed by a world's address: the shared ban
  lists (`chiacraft/bans/1`). A curated list of worlds is the same thing with other entries.
- The wallet's chain reader already asks full nodes for hinted coins (`request_puzzle_state` with
  the hinted filter). It runs in the game when needed (testnet11 by default, mainnet refused),
  and every spend passes `chain::may_spend` (testnet11 only).

## The index: what the chain can and can't do

**Asking for every coin with a hint works** [Chia's full-node source]. A light client sends
`RequestPuzzleState` with the tag in place of a puzzle hash and `include_hinted` set. The node
looks the tag up in its hint table and returns those coins. It filters spent coins
(`include_spent = false`) and small ones (`min_amount`) in its own query. It pages the answer
(`is_finished`, resume from `height`), never splitting a block, and can keep pushing new listings
afterwards (`subscribe_when_finished`). The caps:

- 100,000 items a response from an untrusted peer;
- 200,000 subscriptions;
- 1,000 such requests a minute per connection;
- two wallet syncs at once per node.

Ours is `chia-sdk-client` 0.37's `Peer::request_puzzle_state`, already in use. No public timings
exist [not confirmed]; a few thousand listings should be a few pages from any node **[inferred]**.

**Don't use the RPC `get_coin_records_by_hint`** (or coinset.org, which wraps it). It reads at
most 50,000 hinted coin ids, spent ones included and in no set order, and only then filters. Past
50,000 coins ever made with the tag, new listings can vanish without an error. coinset.org is
also a single service.

**Spam is the real problem.** A hinted coin costs about 2.7 million cost units. A block holds
about 4,000, and blocks come about every 52 s. While the mempool isn't full, which is normal,
fees are zero **[inferred from Chia's cost table and fee rules]**:

- 50,000 junk listings cost about 50,000 mojos (0.00000005 XCH) and about 11 minutes.
- With the mempool full, they cost about 0.7 XCH at the 5 mojo/cost floor. That is still about a
  dollar.

This is how Chia's DAO proposals broke: hint the treasury id with enough coins and wallets past
the 100,000 cap stop seeing new proposals. Chia's known-issues page has no fix beyond filtering
small or odd coins.

So the index needs three things:

1. **A deposit.** The listing coin holds a real amount, and the node's `min_amount` filter skips
   anything smaller. The owner takes it back by spending the coin, which delists the world. At
   0.01 XCH a listing, 50,000 junk listings lock 500 XCH. That is still possible, which is why
   ranking matters, but it is no longer free. On testnet11 the amount is symbolic.
2. **A verifiable world in every listing.** The coin's second memo is the world DID's launcher
   id. The game shows a listing only when that DID's metadata names a world key and an owner
   DID, the owner's DID lists the world back, and the listing coin sits at the address holding
   the world DID. A coin that merely carries the tag, or names someone else's DID, is never
   shown.
3. **A versioned tag.** Junk coins never expire, so the tag is `sha256("chiacraft worlds v1")`,
   and v2 can start clean if v1 fills with junk.

**Other indexes we looked at.** Only the chain and Nostr can list everything without someone
running an indexer.

| Way | Lists everything? | Notes |
|---|---|---|
| Chia hints (above) | yes, from any full node | permanent, verifiable; spam needs a deposit |
| Nostr relays (replaceable events by kind, `#t` tag) | per relay | many relays, none ours, each with its own caps and rules, and they may drop data; `nostr-sdk` is alpha; proof of work exists (NIP-13); publishers' IPs visible to relays |
| iroh-gossip on a known topic | only what live peers say now | fits iroh, but needs bootstrap peers and anyone can flood a topic; no history |
| Mainline DHT announce on a known hash (BEP 5) | a sample of about 10 per query | exposes IP:port, about 8 nodes carry the whole load |
| pkarr / BEP 44 | no, known keys only | what we already use for "key to address" |

**Recommendation:** the chain index as above. Nostr could be a second, optional mirror of the same
signed listing later; a listing that verifies against the chain would be safe to read from any
relay **[inferred]**. Gossip later for live heartbeats among listed worlds, seeded from the index.

## Rich listings: DIG, DataLayer, or the world itself

- **DIG Network** is active, Rust-native and built on Chia DataLayer (CHIP-0035 stores). But it is
  alpha (dig-node 0.5, "nightly alpha" tooling), and its readers go through coinset.org and
  `rpc.dig.net`. Paid mirroring isn't finished (the rewards coin is "scaffolding"), and the read
  path we would link is **GPL-2.0-only**. Not something to depend on now.
- **Chia DataLayer** (a store as a singleton, data by HTTP mirrors) costs a spend per update
  (0.001 XCH default fee, usually 0 when the mempool isn't full), and mirrors are HTTP servers
  someone runs. `chia-sdk-driver` has CHIP-0035 support. It's useful for a list that must be
  readable while its maintainer is offline, which the world itself can't be.
- **The world itself**: the status answer grows a description, tags, language, a screenshot
  hash, and the rules it already sends. The picture travels over iroh from the world, like
  packages. It is free, always current, and needs nothing new. The cost: an offline world shows
  only its on-chain facts and what lists say about it.

**Recommendation:** details from the world, cached by the games that saw them. A world DID may
later point at a CHIP-0035 store for an offline copy. DIG can come later as one more place to
read such a store, once it is out of alpha.

## Curated lists

Bluesky's model fits: one open index, plus labelers and feeds each user picks. CATalog does the
same on Chia (paid open registration, separate verifiers). Make a curated list the same kind of
document as a shared ban list:

- **What it is:** signed by its maintainer's identity, served by any world that holds it,
  followed by a world's address. It names world DIDs with a short note and tags ("hardcore",
  "builders", "Ronald's picks"), and a newer version replaces an older one.
- **Who keeps one:** a world owner publishes theirs with one command, as with ban lists.
- **What the game ships:** it follows no list by default (rule 2). The finder offers a few to
  follow, the way the sponsor is offered.

Lists can also mark worlds to hide. Followed by many, that is Bluesky's labeler idea, and it
limits what a rich spammer gets for their deposits.

## Ranking (what a player sees first)

All of these can be checked locally. None is a single score from anyone.

1. Worlds the player's friends own or play, and worlds linked to worlds they visit (the link
   graph: each world already knows its links).
2. Worlds on lists the player follows, and never ones those lists hide.
3. Then by world DID age (blocks since launch: a new identity can't be old), the size of the
   deposit, and how many worlds the player trusts link to it.
4. Unverifiable listings are never shown. Offline worlds show their last-seen details, below the
   online ones.

## What it costs a world owner

- Listing: one spend from the owner's wallet, locking the deposit, with fee 0 while the mempool
  isn't full. Delisting gives the deposit back with another spend.
- Testnet11 first, as for everything else, behind `may_spend`. A mainnet deposit needs Ronald's
  decision. The sponsor won't pay deposits: its budget is for DIDs, and a sponsored deposit
  would be free spam again.

## Recommendation, as slices

1. **Listing a world** (testnet11): the owner lists a world that has a DID (Who May Come In,
   "List this world"). That makes a coin at the owner's address holding the deposit, with the
   tag as hint and the world DID as second memo. Delisting spends it back.
2. **The finder**: a new Multiplayer tab reads the index through the chain reader (paged,
   unspent, with the minimum amount, from two or more peers). It keeps only listings whose world
   DID checks both ways and whose coin sits at the DID's holder, asks each world's status, and
   shows name, players, rules and online state. It subscribes for new listings.
3. **Rich details from the world**: the status answer gains description, tags, language and a
   screenshot over iroh, set by the owner and cached by the games that see them.
4. **Curated lists**: a signed world list, served and followed like ban lists, used to filter and
   sort.
5. **Ranking**: friends, links and lists first, then DID age, deposit and trusted links.

Later: Nostr as a second mirror of the same signed listings, gossip heartbeats among listed
worlds, a CHIP-0035 store for details while a world is offline, and DIG as one more reader.

## For Ronald to decide

1. **The deposit.** Recommended: 0.0001 TXCH on testnet11 now, which is symbolic and lets us test
   the filter. The mainnet amount waits for your decision; 0.01 XCH is our suggestion, about a
   cent at today's price. Too small and junk listings are cheap; too big and small owners stay
   off the list (they're still found through friends, links and lists).
2. **Only worlds with a DID can be listed?** Recommended: yes. It's what lets a game tell a real
   listing from junk, and a world DID is free through the sponsor.
3. **Nostr as a second mirror later?** Recommended: not now. Revisit when the finder works.
4. **A list kept by us offered in the finder** (never followed by default)? Recommended: yes,
   offered like the sponsor.

## Sources

- Chia wallet protocol and limits: chia-protocol `wallet_protocol.rs`
  (https://docs.rs/chia-protocol/latest/src/chia_protocol/wallet_protocol.rs.html);
  chia-blockchain `coin_store.py`, `hint_store.py`, `full_node_api.py`, `full_node_rpc_api.py`,
  `rate_limit_numbers.py` (github.com/Chia-Network/chia-blockchain, main);
  https://docs.chia.net/wallet-protocol ; `chia-sdk-client` `Peer`
  (https://docs.rs/chia-sdk-client/latest/chia_sdk_client/struct.Peer.html).
- Hint spam: https://docs.chia.net/dao-known-issues ; CHIP-0020
  (https://github.com/Chia-Network/chips/blob/main/CHIPs/chip-0020.md); the 2021 dust storm
  (https://www.github.com/Chia-Network/chia-blockchain/discussions/9049); costs
  (https://docs.chia.net/coin-set-costs/); fees
  (https://docs.chia.net/chia-blockchain/architecture/mempool/fees/).
- DIG: https://github.com/DIG-Network , https://github.com/DIG-Network/dig-node (and issue 377),
  https://docs.rs/crate/dig-store/latest ,
  https://github.com/DIG-Network/dig-rewards-coin/releases/tag/v0.9.0 .
- DataLayer: https://docs.chia.net/guides/datalayer-user-guide/ ; chia-sdk-driver features
  (https://docs.rs/crate/chia-sdk-driver/latest/features).
- iroh-gossip (https://docs.rs/iroh-gossip/latest/iroh_gossip/), Mainline BEP 5
  (https://www.bittorrent.org/beps/bep_0005.html), `mainline` crate
  (https://docs.rs/mainline/latest/mainline/), pkarr (https://github.com/pubky/pkarr), Nostr
  NIP-01 and NIP-13 (https://github.com/nostr-protocol/nips).
- Games and apps: Luanti server list (docs.luanti.org), Veloren's server browser
  (https://veloren.net/blog/query-serverbrowser-announce/), HyperDHT
  (https://docs.pears.com/building-blocks/hyperdht), Urbit address space
  (urbit.org blog), Hyphanet Web of Trust (https://github.com/hyphanet/plugin-WebOfTrust), SSB
  (https://ssbc.github.io/docs/ssb/faq.html), Bluesky (https://arxiv.org/html/2408.12449).
- Chia registries: CATalog (https://docs.catalog.cat/), Namesdao / CHIP-0009, XCHandles
  (CHIP-0054, status not confirmed), Splash! (https://github.com/dexie-space/splash).

Not confirmed: coinset.org's limits and prices; real `request_puzzle_state` timings; the $DIG
token's terms; the status of CHIP-0013 and CHIP-0054; a Chia hard fork near block 9,562,000 that
one secondary source mentions, which could change costs.
