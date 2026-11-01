# Link protocols: requesting, accepting, recording and revoking a mutual world link

Scope: prior art for how two independently owned peers (chiacraft worlds, Ed25519 keys now, Chia DIDs later, iroh QUIC transport, online only while the owner plays or runs a headless server) can request a link, accept it, record it as a record signed by both, and revoke it. Requests must reach the other side even when it is offline. Researched 2026-10-31. Sources are primary specs unless noted. Some pages could not be fetched (Briar wiki/README returned 403; Matrix client-server sections on ACLs and invite permission were past the readable window); those points are listed as gaps.

## 1. How do Matrix, ActivityPub, Nostr, SSB, Holepunch, Veilid and Briar handle requests, acceptance and offline delivery?

### Takeaway
Two patterns recur. (a) A request/response handshake with an explicit accept that the requester waits for before it records anything: ActivityPub Follow then Accept, and Matrix knock then invite, where the invite event is co-signed by both servers. (b) Store-and-forward through an always-on helper that the recipient picks and that cannot read the payload: Nostr inbox relays (kind 10050), Briar Mailbox, SSB pubs, Holepunch blind peers. No system delivers to a fully offline peer without such a helper. In every case, rejection is optional or silent so that the requester learns little.

### Cited Findings

**Matrix (Server-Server API v1.19)**
- Invites to a user on another server are co-signed: the inviting server must get the invite event "signed by both the inviting homeserver and the invited homeserver" before it goes to the other room servers. The invited server returns the event with its own signature added, and "All other fields of the events should remain untouched." — [Matrix S2S API v1.19](https://spec.matrix.org/latest/server-server-api/)
- The invited server validates the invite before signing. It returns `M_INVALID_PARAM` on a bad signature, a wrong event type or membership, a sender not on the origin server, or a state_key not on the receiving server. It can refuse with 403 `M_FORBIDDEN` ("homeserver doesn't allow invites", "refuses to participate"). Since v1.18, `M_INVITE_BLOCKED` signals rejection due to invite blocking. — [Matrix S2S API](https://spec.matrix.org/latest/server-server-api/)
- Knocking is a request to join that room admins answer. It uses `/make_knock` (the server returns an unsigned template) and `/send_knock` (the knocker signs it). On success the knocker gets `knock_room_state`, which "MUST contain the `m.room.create` event". Acceptance is not a separate message: "invites are used to indicate that knocks were accepted". A knock is withdrawn through the leave flow. There is no dedicated reject message; rejecting goes through make_leave/send_leave, and the spec warns against join-then-leave because it can look like acceptance. — [Matrix S2S API](https://spec.matrix.org/latest/server-server-api/)
- Offline catch-up: a sending server "must wait and retry for a 200 OK response before sending a transaction with a different txnId". Transactions hold at most 50 PDUs and 100 EDUs. A server that was away fills in history with `/backfill` and `/get_missing_events`, which walk `prev_events`. — [Matrix S2S API](https://spec.matrix.org/latest/server-server-api/)
- Server ACLs (`m.room.server_acl`) let a room exclude whole servers. The S2S API refuses `/state_ids` with 403 when the requester "is excluded from the room via `m.room.server_acl`". — [Matrix S2S API](https://spec.matrix.org/latest/server-server-api/)
- The invite, knock and leave federation endpoints are all marked "Rate-limited: No". Abuse control is left to each server's policy. — [Matrix S2S API](https://spec.matrix.org/latest/server-server-api/)

**ActivityPub (W3C Recommendation, 23 Jan 2018; older but still the deployed baseline)**
- Follow is a request. The sender SHOULD add the target to its `following` collection only after it receives an `Accept` of that Follow (s6.5). The receiver SHOULD send back `Accept` or `Reject`, either automatically or after a user decides, and MAY omit the `Reject` "for example to protect privacy" (s7.5). — [ActivityPub](https://www.w3.org/TR/activitypub/)
- After a Reject, the actor MUST NOT be added to followers or following (s7.5, s7.7). Undo must come from the same actor as the activity it undoes, and its side effects should be reversed (s6.10, s7.12). — [ActivityPub](https://www.w3.org/TR/activitypub/)
- Block: the server SHOULD stop the blocked actor from interacting, and SHOULD NOT deliver the Block to the blocked actor (s6.9). Blocking is unilateral and silent. — [ActivityPub](https://www.w3.org/TR/activitypub/)
- Delivery is a POST to the recipient's inbox. It SHOULD be asynchronous, and the sender "should additionally retry delivery to recipients if it fails due to network error". Removing a follower that stays unreachable is left to the server's discretion. — [ActivityPub](https://www.w3.org/TR/activitypub/)
- The security appendix is non-normative. It says authentication had no agreed mechanism at standardisation (B.1). It advises verifying federated content by dereferencing object IDs (B.2), filtering spam (B.6), rate limiting with exponential backoff against federation DoS (B.7), and recursion-depth limits (B.5). The recommendation does not address message size limits. — [ActivityPub](https://www.w3.org/TR/activitypub/)

**Nostr (NIP-17 / NIP-59, current master)**
- NIP-59 gift wrap has three layers. The rumor is unsigned, so a leaked rumor "cannot be verified". The seal (kind 13) is signed by the real author and holds the encrypted rumor, and its "Tags MUST always be empty". The gift wrap (kind 1059) is signed by a one-time random key and carries only the recipient `p` tag. — [NIP-59](https://github.com/nostr-protocol/nips/blob/master/59.md)
- Timestamps on the outer layers are randomised (and set in the past) so that timing cannot be correlated. Kind 21059 is a variant that relays must not store, for live-only use. Kind 1059 is the offline-capable one. — [NIP-59](https://github.com/nostr-protocol/nips/blob/master/59.md)
- Relays act as mailboxes. They should require NIP-42 AUTH and serve kind 1059 only to the tagged recipient. — [NIP-59](https://github.com/nostr-protocol/nips/blob/master/59.md)
- Kind 10050 is the recipient's list of DM inbox relays (keep it to 1–3). "Clients MUST only publish events to the relays listed in the recipient's kind 10050 event". If there is no list, "the user is not ready to receive messages and clients shouldn't try." — [NIP-17](https://github.com/nostr-protocol/nips/blob/master/17.md)
- Recipients MUST check that the seal's pubkey equals the rumor's pubkey. Otherwise anyone could impersonate a sender. — [NIP-17](https://github.com/nostr-protocol/nips/blob/master/17.md)
- Spam: because wrappers use one-time keys, "relays cannot apply traditional anti-spam based on pubkey reputation, WoT, etc." The mitigations are NIP-13 proof-of-work on the wrapper, relays requiring AUTH before accepting, or relays declining to store wraps. — [NIP-17](https://github.com/nostr-protocol/nips/blob/master/17.md); [NIP-59](https://github.com/nostr-protocol/nips/blob/master/59.md)

**Secure Scuttlebutt (protocol guide; the ecosystem is largely dormant)**
- An invite code is the pub's host, port and public key plus "~" and an invite secret key. The pub generates a fresh Ed25519 keypair per invite and stores the public key with conditions such as uses left or expiry. Redeeming means handshaking with the invite key and calling `invite.use` with your feed ID. The pub then follows you, and you post a follow-back. — [SSB Protocol Guide](https://ssbc.github.io/scuttlebutt-protocol-guide/)
- Relationships are unilateral log entries: `contact` messages with a `following` boolean (and a `blocking` field). Unfollowing is a later message with `following: false`. Mutual follow is two independent records, not one co-signed record. — [SSB Protocol Guide](https://ssbc.github.io/scuttlebutt-protocol-guide/)
- Pubs are always-on peers that act as replication sources while your device is offline. Replication uses `createHistoryStream` or EBT. — [SSB Protocol Guide](https://ssbc.github.io/scuttlebutt-protocol-guide/)

**Holepunch / Pear**
- Blind peers are servers that "help keep hypercores available". Clients register cores or Autobases with them through the `blind-peering` library (`addCore` / `addAutobase`, or the background variants), passing a list of blind-peer public keys. — [holepunchto/blind-peer](https://github.com/holepunchto/blind-peer); [socket.dev blind-peer summary](https://socket.dev/npm/package/blind-peer)
- Blind peers store and serve encrypted blocks without the read capability. (This comes from a search-result summary of the docs; the doc page itself did not resolve.) — [socket.dev blind-peer](https://socket.dev/npm/package/blind-peer)
- The blind-peer wire encodings define an `addMailbox` RPC, a `post` RPC and a `Mailbox` storage record, which shows that mailbox-style posting to blind peers exists. Its semantics are not documented in the README. — [blind-peer-encodings README](https://unpkg.com/blind-peer-encodings@3.3.1/README.md)

**Briar**
- Default delivery is synchronous peer-to-peer, so both sides must be online. Briar Mailbox (released June 2023) is a helper app on a spare always-on Android device, linked to the owner by QR code. "Contacts can leave messages on your mailbox", and Briar fetches them the next time it comes online. Each mailbox only relays between its owner and the owner's existing contacts, over Tor. — [Briar Mailbox release](https://briarproject.org/news/2023-briar-mailbox-released/); [Briar Mailbox download page](https://briarproject.org/download-briar-mailbox/)

**Veilid**
- `appMessage` / `appCall` target either a node ID or an imported private-route blob, which another node can publish and import. Since 0.2.0 the DHT supports offline subkey writes. Records take an optional secret that grants writer capability. — [Veilid routing_context (mirror)](https://git.hackliberty.org/Git-Mirrors/veilid/src/commit/6f23c4bd0e9f40a5c4cbd6faa9533854598ff74e/veilid-wasm/src/veilid_routing_context_js.rs); [Veilid CHANGELOG (mirror)](https://git.hackliberty.org/Git-Mirrors/veilid/src/branch/main/CHANGELOG.md)

### Inferences
- The shape that fits chiacraft is the ActivityPub/Matrix one: Request, then Accept or silent ignore, with an optional Reject. The requester records nothing until it holds the Accept, which is the second signature. Matrix's co-signed invite (B adds its signature and leaves every other field unchanged) is the closest existing analogue of a link record signed by both worlds.
- For offline delivery, chiacraft has three options:
  1. Sender-side retry, as in ActivityPub and Matrix. A keeps the request and redelivers when it sees B online. This is free, and it works when both worlds' online windows overlap.
  2. Recipient-chosen mailboxes, as in Nostr 10050, Briar Mailbox and Holepunch blind-peer mailboxes. B advertises where requests can be left, for example a friend's headless server or a community relay. The mailbox stores opaque, size-capped blobs addressed to B and only serves them to B.
  3. A DHT slot, as in Veilid. iroh has no store-and-forward DHT for arbitrary payloads, so this would have to be built.

  Recommended: 1 plus an optional 2. The world's address record should list "inbox endpoints", in the same way as Nostr's 10050. If the list is empty, B is reachable only directly, and senders should not try anywhere else.
- Silence as rejection (the ActivityPub MAY-omit-Reject rule) is good for players, because it avoids harassment loops. It needs requests to expire on the requester's side.
- Blocking should be unilateral, local and not announced (ActivityPub Block).

### Gaps
- Matrix `m.room.server_acl` field semantics and the v1.18 client-side "invite permission" config were in sections the fetch could not reach. Only the `M_INVITE_BLOCKED` error code is confirmed.
- Briar's contact-adding handshake (both sides exchange links or scan QR codes) and the mailbox's encryption details: the Briar wiki and README returned 403.
- Holepunch: how Autobase adds a writer (pairing/invite flow) and how the blind-peer mailbox `post` semantics work. The docs pages were unreachable.
- Veilid's own docs on what happens when an AppCall target is offline. VeilidChat's invitation design was not fetched.

## 2. How are mutual agreements represented and verified, and how do they expire and get revoked, including when one side is offline?

### Takeaway
Capability systems (UCAN, Biscuit) and the WebPKI all conclude that revocation is unreliable. Expiry, short lifetimes and renewal are the primary control, and a revocation list is the backstop. Revocation is an append-only, irreversible signed message that the party who controls the resource must cache. It may arrive before the thing it revokes, and it can be garbage-collected once the target has expired. A link signed by both sides fits a "two signatures over one canonical body" record better than a delegation chain. Revocation by either side can be unilateral: a single signed tombstone that references the link's hash.

### Cited Findings
- UCAN Revocation (v1.0.0-rc.1, release candidate): "An Issuer of a particular Delegation in a proof chain MAY revoke that Delegation". A revocation is itself a signed invocation, `do: "ucan/revoke"` with `args.revoke` set to the CID of the revoked delegation. Its `nonce` is empty because revocation is idempotent. — [UCAN Revocation spec](https://github.com/ucan-wg/revocation)
- UCAN storage rules: the agent that controls a resource MUST cache revocations where it is the subject, and MAY cache gossiped ones and forward them. Stores are append-only, and revocations "MUST be immutable and irreversible". A mistaken revocation is fixed by issuing a new delegation. Revocations MAY be evicted after the referenced UCAN expires plus a clock-skew buffer. Revocations that arrive before their target should be accepted, because out-of-order delivery is normal. The spec "MAY operate in fully eventually consistent contexts." — [UCAN Revocation spec](https://github.com/ucan-wg/revocation)
- UCAN prefers prevention: least-authority measures such as expiry "SHOULD be the preferred method", and revocation "SHOULD be considered the last line of defense". — [UCAN Revocation spec](https://github.com/ucan-wg/revocation)
- Biscuit: a block's revocation ID is its signature. The authorizer gets a `revocation_id(index, bytes)` fact per block and checks it against revocation lists it supplies itself. Expiry is a Datalog check such as `check if time($0), $0 < 2019-02-05T23:00:00Z`, evaluated against an ambient `time` fact. — [Biscuit specification](https://doc.biscuitsec.org/reference/specifications)
- Biscuit tokens are a signature chain: "Each block contains the serialized Datalog, the next public key, and the signature by the previous key." Third-party blocks carry an extra external signature. They are bound to one specific token by including the previous signature, so that "a third-party block can only be used for a specific biscuit token". — [Biscuit specification](https://doc.biscuitsec.org/reference/specifications)
- WebPKI: Let's Encrypt states "certificate revocation doesn't work very well". Its six-day certificates (first issued in February 2025, renewal recommended every 2–3 days) "greatly reduce the potential compromise window" and "will not include OCSP or CRL URLs". The sample certificate in the same post still shows an OCSP URI, an inconsistency in the post itself. — [Let's Encrypt, first short-lived cert](https://letsencrypt.org/2025/02/20/first-short-lived-cert-issued/)
- Let's Encrypt has ended OCSP and moved to CRLs only. Sources disagree on the date: one says no OCSP URLs in certificates from early May 2025, another says the service was retired in August 2025. — [Feisty Duck, "The slow death of OCSP"](https://www.feistyduck.com/newsletter/issue_121_the_slow_death_of_ocsp); [LE community thread](https://community.letsencrypt.org/t/the-slow-death-of-ocsp/232883)
- ActivityPub models withdrawing a relationship as `Undo` of the earlier Follow or Accept, by the same actor. — [ActivityPub](https://www.w3.org/TR/activitypub/)
- Matrix models a mutual relationship as a membership event in a replicated DAG. For an invite to another server, the event carries both servers' signatures in a `signatures` object keyed by entity and key ID. — [Matrix S2S API](https://spec.matrix.org/latest/server-server-api/); [Matrix appendices](https://spec.matrix.org/latest/appendices/)

### Inferences
- **Link record.** One canonical body, `{v, purpose:"chiacraft/link/v1", a: worldKeyA, b: worldKeyB, terms…, created, expires?, nonce}`, plus `sigs: {A: …, B: …}`. This is the Matrix pattern: signatures sit outside the signed bytes, and the second signer must not change any field. The link ID is the hash of the canonical body (not of the signatures), so both copies have the same ID. A valid link needs both signatures. A Biscuit- or UCAN-style delegation chain is overkill for a two-party, human-approved, symmetric relationship.
- **Revocation.** A single-signer tombstone, `{purpose:"chiacraft/link-revoke/v1", link: <id>, by: <key>, at}`, that either party can issue. Following UCAN: it is irreversible (relinking means a new request and a new link ID), it is idempotent, it is accepted even if the revoker has not seen the link, and it is cached by both sides. It can be dropped after the link's `expires`.
- **Offline at revocation time.** Revocation takes effect locally at once: the revoker's world stops honouring the portal. The tombstone is delivered by the same retry/inbox path as requests. When the other world reconnects and presents the link, the revoker answers with the tombstone. That is safe because a portal needs both worlds online to traverse, so a stale link on one side cannot be used against the revoker. The only harm is a dangling portal shown on the unaware side.
- **Expiry and renewal.** Short-lived links with automatic renewal while both sides still agree (the Let's Encrypt approach) are probably unnecessary for a game. Cross-world traversal is inherently live, so the live peer can check revocation state at handshake time. An optional `expires` field (for example, a temporary event portal) is cheap to include.
- **Key rotation.** Link records name world keys. When Chia DIDs arrive, a world's DID document can rotate keys, and links should reference the stable world identity, with a key ID included in each signature (Matrix's `ed25519:<id>` style) so rotated keys can be told apart.

### Gaps
- Macaroons (caveats, HMAC chaining, third-party caveats), zcap-ld, and RFC 5280 (CRLs) / RFC 6960 (OCSP) primary texts were not fetched this session. Their general properties (bearer tokens with attenuation; CRLs with nextUpdate; online status responders) are background knowledge only, and none of the recommendations depend on them.
- I found no system that uses a record co-signed by two independent peers and revoked unilaterally by either, outside Matrix invites and blockchain multisig. This design is a synthesis, not an adopted standard.

## 3. What does a request inbox need, and how do these systems store pending requests?

### Takeaway
Inboxes protect themselves with five things: authentication of the sender (Nostr AUTH, Matrix request signing), recipient-controlled reachability (Nostr 10050 "no list means don't try", Briar Mailbox "contacts only"), per-sender and global rate limits, optional proof-of-work for unknown senders (NIP-13), and silent drop or blocking. Pending requests sit in the recipient's server state (Matrix knock membership in room state, the ActivityPub inbox collection) or as opaque blobs on a mailbox until the owner fetches them.

### Cited Findings
- ActivityPub recommends rate limiting with exponential backoff against federation DoS, spam filtering, and limits on collection page and response size. It says nothing on message size limits. — [ActivityPub, App. B](https://www.w3.org/TR/activitypub/)
- Matrix federation endpoints for invite and knock are marked not rate-limited. A server can refuse invites outright (403, "homeserver doesn't allow invites") or with `M_INVITE_BLOCKED` (v1.18+). Pending knocks live as `membership: knock` state in the room until an invite (acceptance) or a leave. — [Matrix S2S API](https://spec.matrix.org/latest/server-server-api/)
- Nostr: the recipient lists 1–3 DM inbox relays. Relays should require AUTH, serve wraps only to the recipient, and may require PoW (NIP-13) or decline to store wraps. Pubkey-reputation filtering is impossible on the outer layer because wrap keys are one-time. — [NIP-17](https://github.com/nostr-protocol/nips/blob/master/17.md); [NIP-59](https://github.com/nostr-protocol/nips/blob/master/59.md)
- SSB pub invites carry server-side conditions (number of uses left, expiry), stored against the invite's public key. — [SSB Protocol Guide](https://ssbc.github.io/scuttlebutt-protocol-guide/)
- A Briar Mailbox only delivers between its owner and the owner's existing contacts. Unknown parties cannot drop messages. — [Briar Mailbox release](https://briarproject.org/news/2023-briar-mailbox-released/)
- Matrix transactions are capped at 50 PDUs and 100 EDUs per transaction. — [Matrix S2S API](https://spec.matrix.org/latest/server-server-api/)

### Inferences
Recommended inbox policy for chiacraft. Links per world are few and approved by humans, so the bar is low but abuse must be cheap to stop.
- **Policy modes per world:** `closed` (no requests accepted), `allowlist` (only listed world keys or DIDs may request), `open` (anyone may request; unknown senders go to a "pending" list).
- **Pre-allow, analogous to Follow auto-accept:** a list of world keys whose requests are counter-signed automatically, optionally restricted to specific terms such as portal type or location.
- **Limits:** a request is a single signed record, at most about 4–8 KiB including an optional note (the size is a suggestion, not from a source). Allow one pending request per (sender, recipient) pair, where a newer one replaces the older. Rate-limit per sender key and per remote node ID. Cap the total pending list (for example, about 100) and evict the oldest. Expire requests after N days.
- **Unknown-sender cost:** optional hashcash-style PoW on requests from senders not on the allowlist (NIP-13 style), or require that the request come from a world the recipient's friends have linked (web-of-trust). Both are optional knobs; for a first version, rate limits plus silent drop are probably enough.
- **Storage:** the recipient stores pending requests locally, keyed by request hash, as part of world state. Mailbox helpers store only opaque, size-capped, TTL-bound blobs addressed to the recipient's key, and hand them over only after the recipient authenticates, as in Nostr AUTH.
- **Encryption:** encrypt the request to B's key if it goes through a third-party mailbox, so the mailbox learns only "something for B". Gift-wrap-level metadata hiding (random outer keys, jittered timestamps) is probably not needed for a game.

### Gaps
- No published numbers on the size or rate limits that Mastodon, Synapse or Nostr relays actually enforce were gathered. The AP/Matrix specs leave them to implementations.
- No primary-source security analyses of Fediverse or Matrix invite-spam waves were fetched this session.

## 4. Canonical serialisation, domain separation, versioning and replay protection for signed records

### Takeaway
Systems that sign re-serialised JSON have run into trouble: SSB's V8 `JSON.stringify` byte order is a known quirk, and Matrix needs a strict canonical-JSON profile with no floats. Newer designs sign explicit byte strings with a domain-separation prefix (SSHSIG's `"SSHSIG"` preamble plus namespace; Biscuit v1's `\0BLOCK\0`, `\0EXTERNAL\0` tags). For chiacraft, signing a deterministic binary encoding (for example postcard or borsh) of a versioned struct, prefixed by a purpose label, is the low-risk choice. Replay protection comes from nonces, timestamps and expiry, plus idempotent records identified by hash.

### Cited Findings
- Matrix canonical JSON: "the shortest UTF-8 JSON encoding with dictionary keys lexicographically sorted by Unicode codepoint". Integers only, in the range ±(2^53−1), with no exponents and no `-0`. "Float values are not permitted." Signing excludes `signatures` and `unsigned`. Signatures go under the signing entity's name, keyed by a signing key ID made of algorithm plus key ID (for example `ed25519:1`). Verifiers drop key IDs with algorithms they do not understand. — [Matrix appendices](https://spec.matrix.org/latest/appendices/)
- SSB signs a specific JSON formatting: two-space indent, LF line endings, fixed field order (`previous, author, sequence, timestamp, hash, content`). Content field order must be kept exactly as received, because re-serialising changes the bytes. The reference implementation rejects non-canonical messages. Each message carries `sequence` and `previous` (a hash chain), which gives replay and fork detection. — [SSB Protocol Guide](https://ssbc.github.io/scuttlebutt-protocol-guide/)
- SSHSIG signs `"SSHSIG" ‖ namespace ‖ reserved ‖ hash_alg ‖ H(message)`. The namespace "prevents cross-protocol attacks caused by signatures intended for one intended domain being accepted in another." — [OpenSSH PROTOCOL.sshsig](https://raw.githubusercontent.com/openssh/openssh-portable/master/PROTOCOL.sshsig)
- Biscuit signature v1 prefixes every field with an ASCII tag (`\0BLOCK\0`, `\0PAYLOAD\0`, `\0ALGORITHM\0`, `\0NEXTKEY\0`, `\0PREVSIG\0`). External signatures begin with `\0EXTERNAL\0`. Payloads carry an explicit `version` (absent means 0, deprecated). Third-party blocks bind to the previous signature so they cannot be replayed onto another token. — [Biscuit specification](https://doc.biscuitsec.org/reference/specifications)
- UCAN revocations use an empty nonce because they are idempotent, and they are identified by the CID (content hash) of the target. — [UCAN Revocation spec](https://github.com/ucan-wg/revocation)
- Nostr randomises outer-layer timestamps and keeps the real `created_at` inside the encrypted rumor. — [NIP-59](https://github.com/nostr-protocol/nips/blob/master/59.md)
- Matrix requires a transaction to be acknowledged before a new `txnId` is used, which gives idempotent retries. — [Matrix S2S API](https://spec.matrix.org/latest/server-server-api/)

### Inferences
Recommended encoding for chiacraft link messages:
- **Signed bytes** = `b"chiacraft-sig\0" ‖ purpose ‖ b"\0" ‖ encode(body)`. The purpose is one of `link-request/v1`, `link-accept/v1`, `link/v1`, `link-revoke/v1`, `inbox-policy/v1`. The encoding is a deterministic binary format (postcard or borsh, fixed field order, no maps with unordered keys, no floats). The purpose label works like an SSHSIG namespace. It stops an Ed25519 world key's signature on a link from being reused as a package signature (Phase 2 already signs packages; give those a different label).
- **Body fields:** `version`, `from`, `to`, `created_at` (unix seconds), `expires_at`, `nonce` (16 random bytes), and the link terms. The request ID is `blake3(signed bytes)`.
- **Two-step co-signing:**
  1. A signs the `link/v1` body and sends it as the request.
  2. B verifies, checks that nothing it cares about is missing, and signs the identical bytes. This is the Matrix "all other fields untouched" rule.
  3. B returns `{body, sigA, sigB}`.

  Both store the record. Since there is no mutation between the steps, the accepted record is exactly the request plus a second signature. That removes a class of "accepted something different from what was asked" bugs.
- **Replay and freshness:** reject requests past `expires_at` or with `created_at` too far in the future (skew tolerance, for example ±10 minutes). Dedupe by request ID. A replayed accepted link is harmless, because it is idempotent and keyed by hash. A replayed revocation is harmless for the same reason. Relinking after revocation needs a fresh nonce, so a new link ID that the old tombstone does not cover.
- **Key IDs and algorithm agility:** include a `key_id` (for example `ed25519:<short-hash>`, later a DID verification-method ID) with each signature. Verifiers ignore unknown algorithms, as Matrix does. This leaves room for the move to Chia DIDs (BLS keys) without changing the record shape.
- **Future option:** if interoperability with external tooling ever matters, RFC 8785 JCS or DAG-CBOR are the standard canonical forms. They were not fetched or verified in this session. For a closed Rust-to-Rust protocol, a deterministic binary encoding plus a purpose prefix avoids the canonical-JSON pitfalls seen in SSB.

### Gaps
- RFC 8785 (JCS) and the DAG-CBOR spec were not fetched this session, so their exact rules (number serialisation, map key ordering) are not quoted here.
- Whether postcard and borsh guarantee canonical (unique) encodings for every type was not verified. For example, postcard varints should be minimal on decode, which needs checking. This should be checked before relying on re-encoding for verification. The safer approach is to verify over the exact received bytes and never re-encode.
