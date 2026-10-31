# Package signing, trust models and consent UX for auto-downloaded world content

Research date: 2026-10-31. Context: decentralised Rust voxel game, no central service; account = BIP-39 mnemonic -> Ed25519/BLS identity key (later linked to a Chia DID); worlds pin data-only packages by hash; first-join consent; trust remembered per creator key and per package hash; later, reviewer-signed hash lists.

Sourcing note: claims marked **[fetched]** were read from the source in this session. Claims marked **[snippet]** / **[fetched-snippet]** come from search-result summaries of the linked page (not a full read). Claims marked **[bg]** come from prior knowledge with a canonical URL that I did not re-fetch this session; the report writer should treat them as well-known but not re-verified today.

---

## 1. Signing tools and schemes, and what package ecosystems do in 2025–2026

### Takeaway
For a serverless game, the right model is minisign/signify-style: plain Ed25519 detached signatures over a structured manifest, with a signed "trusted comment"-like metadata area, plus a few TUF ideas (versions, expiry, threshold/rotation). Sigstore, npm provenance, PyPI attestations and crates.io trusted publishing all rely on central OIDC identity providers and hosted transparency logs, so they do not fit as-is. Their useful lesson is to bind artifacts to a stable identity and make the binding auditable.

### Cited Findings
**minisign / signify**
- minisign signature file = 4 lines: untrusted comment; base64(algorithm, 8-byte key ID, signature); trusted comment; base64 global signature. The global signature is Ed25519 over (signature || trusted comment), so the trusted comment can't be changed without the secret key. The untrusted comment can be edited freely. **[fetched]** — [minisign docs](https://jedisct1.github.io/minisign/)
- Algorithm `ED` (hashed) = Ed25519 over the BLAKE2b-512 hash of the file. `Ed` (legacy) = Ed25519 over the raw data. "New implementations must use the hashed signature format", and legacy support "will eventually be removed". **[fetched]** — [minisign docs](https://jedisct1.github.io/minisign/)
- Key ID = 8 random bytes that must match the public key, so the verifier can tell which key is meant. Useful for rotation, not a security property. **[fetched]** — [minisign docs](https://jedisct1.github.io/minisign/)
- OpenBSD signify (Ted Unangst, 2014–2015) signs a SHA256 checksum file listing every release file, not each file. Each release ships the *next* release's public key, so keys rotate per release without a CA. **[bg]** — [signify: Securing OpenBSD From Us To You](https://www.openbsd.org/papers/bsdcan-signify.html)

**SSH signatures**
- `ssh-keygen -Y sign -n <namespace>` produces an SSHSIG blob. The namespace is mandatory and is included in the signed data (with a "SSHSIG" magic preamble and the message hash), so a signature made for "file" can't be reused as a signature for "git" or for another protocol. This is built-in domain separation. **[bg]** — [OpenSSH PROTOCOL.sshsig](https://github.com/openssh/openssh-portable/blob/master/PROTOCOL.sshsig)

**TUF (The Update Framework)**
- Spec v1.0.37 (last modified 8 Oct 2026) defines these roles:
  - Root: lists the trusted keys for all roles. Keys should be offline.
  - Targets: signs metadata (hashes, lengths) about target files, not the files themselves.
  - Snapshot: lists the versions of all targets metadata, which prevents mix-and-match.
  - Timestamp: re-signed frequently with an online key, so a freeze is detected quickly.

  **[fetched]** — [TUF spec](https://theupdateframework.github.io/specification/latest/)
- Targets can delegate trust to other roles by path pattern or hash prefix. Delegations can be prioritized or terminating, and a delegation is revoked by publishing new metadata without it. **[fetched]** — [TUF spec](https://theupdateframework.github.io/specification/latest/)
- Each role has a set of keys and a threshold. Metadata is trusted only when signed by at least the threshold of distinct keys. **[fetched]** — [TUF spec](https://theupdateframework.github.io/specification/latest/)
- Every metadata file carries a `version` and an `expires`. Clients reject expired metadata and anything with a version lower than the one they already trust (rollback protection). Root versions must increment by exactly 1. **[fetched]** — [TUF spec](https://theupdateframework.github.io/specification/latest/)
- Defended attacks:
  - Arbitrary installation
  - Endless data
  - Extraneous dependencies
  - Fast-forward
  - Indefinite freeze
  - Malicious mirrors
  - Mix-and-match
  - Rollback
  - Wrong software
  - Compromise of fewer than a threshold of keys

  **[fetched]** — [TUF spec](https://theupdateframework.github.io/specification/latest/)
- Root rotation: the new `root.json` has version+1 and is signed by a threshold of the *old* root keys and a threshold of the *new* root keys. Clients walk each intermediate root version in order. **[fetched]** — [TUF spec](https://theupdateframework.github.io/specification/latest/)

**Sigstore / cosign / Rekor**
- Rekor v2 reached GA on 10 Oct 2025. It is a tile-backed transparency log, with upload/verify support in Cosign v2.6.0 and the Go, Python and Java clients. **[fetched-snippet]** — [Sigstore blog](https://blog.sigstore.dev/)
- In v2 a bundle embeds the log entry, inclusion proof and checkpoint, so it can be verified offline against a trusted root that holds the log key. Keyless signing still needs network access to Fulcio and Rekor. **[fetched-snippet]** — [NVIDIA AICR Rekor v2 docs](https://docs.nvidia.com/aicr/latest/contributor-guide/rekor-v-2-signing)
- Keyless model: Fulcio issues a short-lived certificate (about 10 minutes) binding an ephemeral key to an OIDC identity (GitHub, Google, Microsoft and so on), and the signature is logged in Rekor. **[bg]** — [sigstore.dev overview](https://docs.sigstore.dev/about/overview/)

**in-toto**
- in-toto defines signed "layouts" (expected supply-chain steps and who may perform them) and "links" (evidence that each step happened). Its attestation format is the envelope used by SLSA provenance, npm provenance and PyPI attestations. **[bg]** — [in-toto.io](https://in-toto.io/)

**Package ecosystems, 2025–2026**
- crates.io Trusted Publishing launched 11 Jul 2025 (RFC #3691):
  - A configured GitHub repository gets a short-lived OIDC-issued publish token (via `rust-lang/crates-io-auth-action`).
  - The first release must be published manually.
  - At launch only GitHub Actions was supported, and the post doesn't mention attestations or Sigstore.

  **[fetched]** — [Rust blog, 2025-07-11](https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07)
- The 21 Jan 2026 crates.io update added enforcing Trusted Publishing per crate and GitLab CI support. **[snippet]** — [Rust blog 2026-01-21](https://blog.rust-lang.org/2026/01/21/crates-io-development-update); [Help Net Security](https://www.helpnetsecurity.com/2026/01/21/rust-crates-io-security-update/)
- npm moved to trusted publishing (OIDC, npm CLI ≥ 11.5.1) and short-lived tokens through 2025. Classic tokens were reportedly all revoked on 9 Dec 2025. Provenance is generated automatically with trusted publishing, but it does not guarantee the code is benign. These are secondary sources, so verify against the GitHub changelog. **[snippet]** — [vpncentral](https://vpncentral.com/?p=244837); [hackernoon](https://hackernoon.com/trusted-publishing-for-npm-the-missing-steps-the-docs-dont-spell-out)
- PyPI PEP 740 attestations went live on 14 Nov 2024 and are Sigstore-based. They are on by default for Trusted Publishing via `pypa/gh-action-pypi-publish`. At launch about 5% of the top 360 projects attested. **[snippet]** — [PyPI blog 2024-11-14](https://blog.pypi.org/posts/2024-11-14-pypi-now-supports-digital-attestations/); [PEP 740](https://peps.python.org/pep-0740)
- PyPI 2025 review: more than 50,000 projects use trusted publishing, and more than 20% of uploads in the year came via trusted publishers. **[snippet]** — [PyPI 2025 in review](https://blog.pypi.org/posts/2025-12-31-pypi-2025-in-review)
- The broader 2025 trend: registries replace long-lived secrets with workload identity (OIDC) and attach provenance tied to a source repo and build. The motivation was supply-chain worms like Shai-Hulud (2025) that stole tokens. **[snippet]** — [Socket blog](https://socket.dev/blog/crates-launches-trusted-publishing)

### Inferences
- **Sigstore keyless is a poor fit for the core path.** It depends on centralized OIDC IdPs, Fulcio and the public Rekor. That conflicts with "no central service by design" and with offline or LAN play. The game already has what keyless replaces: a long-lived identity key held by the player. An optional later extension could let creators *additionally* log their signature in Rekor for public auditability, but nothing should require it.
- **Minisign's format is a good template and is directly implementable.** It gives detached Ed25519 signatures, prehashing with BLAKE2b, a key ID, and a signed metadata area alongside an unsigned one. Rust implementations exist (e.g. `minisign`, `minisign-verify` crates **[bg]**). Using it verbatim is optional: a custom canonical manifest signed with Ed25519 plus a domain tag gives the same properties with a tighter schema.
- **TUF fits the reviewer-list and creator-key-rotation problems, not the per-world pin.** A world's pinned hash is already content-addressed, so integrity needs no signature at all. Signatures only answer "who made this / who vouches for it". TUF concepts worth borrowing:
  - Monotonic version + expiry on reviewer lists, for rollback/freeze protection.
  - Threshold signing for reviewer groups ("2 of 3 reviewers").
  - Root-style rotation: the new key set is signed by the old and new keys.
  - "Endless data" protection: the manifest declares lengths, and the client enforces them while downloading.

  Snapshot and timestamp roles add little, because there is no single repository whose freshness must be proven.
- **in-toto/SLSA provenance is overkill for data packages.** There is no build step to attest, apart from an optional "converted from source assets by tool X".

### Gaps
- I did not re-fetch the Rust crate docs for `minisign`/`minisign-verify` or `ed25519-dalek`'s `Ed25519ph`/`ctx` support. Check the API surface before choosing.
- I could not find the exact current npm documentation pages this session. The npm dates come from secondary sources.

---

## 2. What exactly to sign: hash vs manifest, domain separation, rotation/revocation, timestamps

### Takeaway
Sign a canonical manifest, never a bare hash:
- Fields: magic/domain tag, schema version, package id, semantic version, creator public key, file list with sizes and hashes, total size, declared asset limits, created-at, optional `prev` hash.
- Use explicit domain separation (context string) for every signature type.

The package hash the world pins should be the hash of that manifest, which commits to all file hashes. Key rotation without a server can follow the signify/TUF pattern (the old key signs the new key, as an append-only chain). Once linked, the Chia DID can be the root of authority listing the current keys.

### Cited Findings
- signify signs a checksum *list* (manifest) rather than each artifact. minisign prehashes the content and signs the digest together with a trusted comment. In both, the signed object is a small, structured statement about content. **[fetched]** — [minisign docs](https://jedisct1.github.io/minisign/); **[bg]** [signify paper](https://www.openbsd.org/papers/bsdcan-signify.html)
- TUF targets metadata records each target's hashes *and length*, so clients can stop downloads at the declared size ("endless data" attack). **[fetched]** — [TUF spec](https://theupdateframework.github.io/specification/latest/)
- SSHSIG puts a mandatory namespace into the signed preimage, which prevents cross-protocol signature reuse. **[bg]** — [PROTOCOL.sshsig](https://github.com/openssh/openssh-portable/blob/master/PROTOCOL.sshsig)
- RFC 8032 defines Ed25519ctx and Ed25519ph, which take a context string of up to 255 bytes. Plain Ed25519 has no context, so applications must prefix a domain tag themselves. **[bg]** — [RFC 8032 §5.1](https://www.rfc-editor.org/rfc/rfc8032#section-5.1)
- BLS signatures (IETF draft, used by Chia's BLS12-381 scheme) need a domain separation tag (DST) in hash-to-curve. Chia uses the AugSchemeMPL ciphersuite, which prepends the public key to the message. **[bg]** — [draft-irtf-cfrg-bls-signature](https://datatracker.ietf.org/doc/draft-irtf-cfrg-bls-signature/)
- TUF rotation: the new root is signed by a threshold of the old and new keys, and clients walk the chain version by version. Expiry bounds how long a withheld update can freeze a client. **[fetched]** — [TUF spec](https://theupdateframework.github.io/specification/latest/)
- W3C DID Core: a DID document lists `verificationMethod`s and relationships (`authentication`, `assertionMethod`, ...). Rotation and deactivation are defined by each DID method. `did:key` cannot rotate, while ledger-anchored methods can. **[bg]** — [W3C DID Core 1.0](https://www.w3.org/TR/did-core/)
- Nostr illustrates the failure mode of "identity = key": once an npub is compromised, the protocol has no clean way to separate pre-breach from post-breach events, and "the identity is NOT the key". **[snippet]** — [Damus discussion](https://damus.io/npub16xnpfx85k8wzdhctang6860g3u64lds5kac73ddjwlg0lxdg9g3su56z6l); [nostr.lucianmarin.com](https://nostr.lucianmarin.com/post/158fb1db0c545c09a071c1351f398b7820c4af2cd779481ff5141e8fd8cf7d99)

### Inferences (recommended design)
- **Manifest (canonical encoding).** Use deterministic CBOR (RFC 8949 §4.2) or a fixed-order binary struct, not free-form JSON. Re-serialization ambiguity must be impossible. Fields:
  - `magic = "weft-pkg-manifest/v1"`
  - `package_id` (creator-key-scoped name, e.g. `creatorkey/slug`)
  - `version` (monotonic per id)
  - `creator_pubkey`, `key_epoch`/`key_id`
  - `created_at` (creator's claim; informational, not trusted)
  - `files: [{path, size, blake3}]` (normalized relative paths, sorted)
  - `total_size`
  - `content_rating`/tags (self-declared)
  - `license`
  - `prev_manifest_hash` (optional, links versions)
  - `limits` (max texture dimensions used, etc.)
- **Package identity = BLAKE3(manifest bytes).** The world pins this. The signature is a separate envelope `{manifest_hash, sig_alg, key_id, sig}`, so adding endorsements (reviewers, co-signers) doesn't change the pinned hash. This keeps "integrity" (hash pin) separate from "authorship/endorsement" (signatures).
- **Domain separation.** Signed preimage = `"weft:v1:" || purpose || 0x00 || manifest_hash`, where purpose ∈ {`pkg-manifest`, `reviewer-list`, `key-rotation`, `revocation`, `world-pin`, ...}. If the same key also signs chat, world-join proofs or Chia coin spends, different purposes must never collide. Never sign raw user-supplied bytes with the identity key. Prefer deriving a *dedicated content-signing subkey* from the mnemonic (separate BIP-32/SLIP-10 path), certified by the account root key. Then a leaked content key doesn't compromise the account, and vice versa.
- **Rotation without a server.**
  1. Phase 1: the creator publishes a `key-rotation` statement `{old_key, new_key, epoch+1}`, signed by both the old and the new key (TUF-style), and distributes it inside the next package or via gossip. Clients that trusted the old key carry the trust forward.
  2. Phase 2: when the Chia DID is linked, the DID (on-chain singleton) becomes the root of authority. Its current inner puzzle/metadata lists valid content keys, so rotation and recovery come from the chain rather than from gossip.
- **Revocation.** A creator-signed `revocation{key or manifest_hash, reason}` statement, gossiped. Clients keep a local revocation set. Revocation of a key should *not* retroactively block content the player already trusts by hash. Instead, show a warning and re-ask. The risk is that a thief issues a revocation, so treat it as "re-prompt", not "delete".
- **Timestamps.** A creator's `created_at` is unverifiable. For ordering, use the version counter and the `prev` chain. For "signed before compromise" evidence, a Chia block height (anchoring the manifest hash in a coin or DID metadata update) can later serve as a decentralised timestamp. That is optional and costs fees.

### Gaps
- Chia DID specifics (how keys are listed in DID metadata, cost of updates, CHIP number) were not researched this session. See the Chia docs.
- No source this session on whether `ed25519-dalek` exposes Ed25519ctx. Manual prefixing works in any case.

---

## 3. How games and platforms handle trust and consent for downloaded content

### Takeaway
Most games either ask once with no information (Minecraft), or don't ask and rely on the platform's moderation (Steam Workshop, Roblox, Hytale's server-pushed assets). Prompts with no actionable information become click-through: decades of warning and permission research show that most people ignore or misread them. Consent is meaningful when the prompt is rare, specific and backed by a remembered decision. Specific means it shows creator identity, size, what changed, and who vouches. Remembered means per world/server and per creator, so it isn't repeated.

### Cited Findings
**Minecraft (Java)**
- `server.properties` options:
  - `resource-pack` URL, max 250 MiB.
  - `resource-pack-sha1`: optional, lowercase hex. A mismatch only produces a console warning on the server side.
  - `resource-pack-id`: UUID.
  - `require-resource-pack`: if declined, the player is disconnected.
  - `resource-pack-prompt`: custom prompt text, chat components.

  **[fetched]** — [Minecraft Wiki: server.properties](https://minecraft.wiki/w/Server.properties)
- The client server list stores a per-server "Server Resource Packs: Prompt / Enabled / Disabled" setting. This is per-server trust memory: once a player accepts, it is never asked again for that server. **[bg]** — [Minecraft Wiki: Server list](https://minecraft.wiki/w/Server_list)
- Why it's "mostly accept": the prompt shows no pack size, author or content information. With `require-resource-pack`, declining means you can't play, so the choice is coerced. The only meaningful decision is "do I want to play on this server". *(Inference from the property descriptions above.)*

**CurseForge / fractureiser (June 2023)**
- Malware was embedded in Minecraft mods on CurseForge and BukkitDev, some of which were pulled into popular modpacks. It targeted Windows and Linux. A mitigation team formed, and a community meeting on 2023-06-08 discussed preventive measures. **[fetched]** — [fractureiser README](https://github.com/fractureiser-investigation/fractureiser/blob/main/README.md)
- It was named after the uploading CurseForge account. Kaspersky's writeup notes downloads from CurseForge and dev.bukkit.org. **[snippet]** — [Kaspersky blog](https://www.kaspersky.com.au/blog/wp-json/wp/v2/posts/32087)
- Background **[bg]**, from the investigation's tech/timeline docs, not re-fetched: the spread came from compromised author accounts uploading trojaned versions of legitimate projects. Follow-up community discussion focused on 2FA, mod signing and reproducible builds. — [fractureiser docs/](https://github.com/fractureiser-investigation/fractureiser/tree/main/docs)

**Hytale (early access 2026)**
- Hosting guides say Hytale is server-first. Players don't pre-install mods: the client pulls models, textures, sounds and config from the server at connect time, behind a "Downloading Server Assets" screen. Heavy packs strain host bandwidth. These are *secondary hosting-provider sources, not official*, and one guide still mentions client-only mods. **[snippet]** — [supercraft.host](https://supercraft.host/wiki/hytale/hytale_curseforge_modding_servers/); [winternode](https://winternode.com/blog/hytale/hytale-mods); [baohost](https://baohost.com/kb/hytale-mods); [survivalservers](https://www.survivalservers.com/wiki/How_to_Install_Mods_on_a_Hytale_Server)

**Luanti (ex-Minetest)**
- Servers push media (textures, sounds, models) to clients automatically on join, with no prompt. Client mods can't supply their own media. Only Ogg Vorbis sounds are supported, which keeps the format surface small. ContentDB is the curated distribution hub, and it hides non-free (CC-NC) media from the client by default. **[snippet]** — [Luanti client_lua_api.md](https://github.com/luanti-org/luanti/blob/master/doc/client_lua_api.md); [Luanti docs: ContentDB](https://docs.luanti.org/contentdb)

**Steam Workshop / Roblox / Thunderstore / Flatpak / browsers**
- Steam Workshop: subscribing auto-downloads and auto-updates items. There's no per-item install prompt. Valve doesn't pre-review content, but games can curate (e.g., per-game item approval). **[bg]** — [Steamworks Workshop docs](https://partner.steamgames.com/doc/features/workshop)
- Roblox moderates uploaded assets (images, audio, meshes) before they're usable in experiences. Trust is centralized platform moderation, not user consent. **[bg]** — [Roblox Creator Hub: moderation](https://create.roblox.com/docs/production/moderation)
- Thunderstore/r2modman: a community mod index with a mod manager. I found no primary source this session on its signing or review practice. → Gap.
- Flatpak/Flathub: the repo (OSTree) is signed. Flathub shows "verified" developer badges (domain-verified publishers) and lists sandbox permissions at install. Updates that add permissions require re-confirmation. **[bg]** — [Flathub docs: verification](https://docs.flathub.org/docs/for-app-authors/verification)
- Chrome extensions: when an update requests new permissions that trigger warnings, Chrome disables the extension until the user approves. This is consent-on-escalation, not consent-on-every-update. **[bg]** — [Chrome developers: permission warnings](https://developer.chrome.com/docs/extensions/develop/concepts/permission-warnings)

**Consent and fatigue research**
- Felt et al. 2012 (Android install-time permissions): only 17% of participants paid attention to permissions during installation, and only 3% of survey respondents answered all comprehension questions correctly. **[bg]** — [Felt et al., SOUPS 2012](https://www.cs.berkeley.edu/~afelt/permissions-soups2012.pdf)
- Akhawe & Felt 2013 ("Alice in Warningland"): SSL warning click-through was about 70% in Chrome vs about 33% in Firefox. Design differences change behaviour substantially. **[bg]** — [USENIX Security 2013](https://www.usenix.org/conference/usenixsecurity13/technical-sessions/presentation/akhawe)

### Inferences (UX recommendations)
- **Ask about the decision the player can actually make.** The primary decision is "join this world with its content". Show a single join sheet with:
  - World host identity.
  - Number of packages and total download size (exact, from manifests).
  - Each package's creator, as a petname or display name plus a key fingerprint glyph, marked "new to you" / "trusted" / "endorsed by N reviewers you follow".
  - Content-rating tags.
- **Default "trust" scopes (narrow to broad):**
  1. This package hash.
  2. All packages this world pins (until they change).
  3. All packages from this creator key.
  4. Packages endorsed by reviewer X.

  The join sheet should default to (1)/(2). Creator-level trust should be an explicit secondary action ("Always allow content from Ava ⟨fingerprint⟩"), as in Minecraft's per-server toggle and Chrome's escalation model.
- **Re-prompt only on meaningful change:**
  - A new creator key.
  - A size jump above a threshold.
  - A new content-rating tag (e.g. "mature").
  - A creator key rotation or revocation notice.
  - A reviewer withdrawing an endorsement.

  Same creator, new version, similar size: no prompt, just a toast ("Ava's Furniture updated 1.2 → 1.3"). This mirrors Chrome's update behaviour.
- **Make declining non-destructive.** Let players join with "placeholder" assets, or as a spectator, without the content when feasible. This avoids Minecraft's coercive accept-or-leave. Where a world can't function without the content, say so plainly.
- **Keep trust state local, visible and revocable.** Provide a "Trusted creators & packages" screen in the Account area, with last-used time, per-world overrides, and "forget".
- **Pre-cache sizes** so the prompt never says "unknown size". Manifests carry `total_size`, and the client enforces it during download.

### Gaps
- No primary source this session for Thunderstore/r2modman, for Hytale's official modding/asset-trust statements, or for documented Steam Workshop malware cases. Cities: Skylines (2022) and others are reported in the press but were not verified here.
- No quantitative data found on Minecraft resource-pack accept rates.

---

## 4. Safety of data-only content (bombs, parser bugs, zip slip, limits)

### Takeaway
"No code" is not "safe". The real risks are:
- Resource exhaustion: decompression bombs, giant dimensions, millions of files, overlapping zip entries.
- Path traversal on extraction.
- Memory-safety bugs in C parsers. libwebp's CVE-2023-4863 was exploited in the wild.

Mitigate with a pure-Rust decoding stack and hard limits enforced *before and during* decode. Better still, avoid zip extraction entirely: store files in a content-addressed blob store keyed by hash, never written to user-chosen paths.

### Cited Findings
- Zip Slip: crafted archive entry names like `../../evil.sh` write outside the extraction directory, leading to arbitrary file overwrite and often RCE. Snyk disclosed it publicly on 5 Jun 2018. It affected thousands of projects across Java, JS, Ruby, .NET and Go, and was especially common in Java because of copy-pasted extraction code. **[fetched]** — [Snyk Zip Slip research](https://security.snyk.io/research/zip-slip-vulnerability)
- Rust `image` crate `Limits` (0.25.x):
  - Fields: `max_image_width`, `max_image_height` (strict, default unlimited) and `max_alloc` (default 512 MiB, caps the decoder's simultaneous allocations including the output buffer).
  - `Limits::no_limits()` disables all of them.
  - Non-strict limits are best-effort only.

  **[snippet]** — [image limits.rs (docs.rs)](https://docs.rs/crate/image/latest/source/src/io/limits.rs); [rerun re-export docs](https://ref.rerun.io/docs/rust/stable/rerun/external/image/struct.Limits.html)
- Zip bombs: David Fifield's 2019 "A better zip bomb" uses overlapping local file entries to reach roughly 42 kB → 5.5 GB, 10 MB → 281 TB, and (Zip64) 46 MB → 4.5 PB. The defence is to reject overlapping entries and to enforce declared vs actual sizes and compression ratios. **[bg]** — [bamsoftware.com/hacks/zipbomb](https://www.bamsoftware.com/hacks/zipbomb/)
- libwebp CVE-2023-4863: a heap buffer overflow in lossless WebP Huffman decoding, exploited in the wild (Sept 2023). It affected every app that embeds libwebp (browsers, Electron, etc.). **[bg]** — [NVD CVE-2023-4863](https://nvd.nist.gov/vuln/detail/CVE-2023-4863)
- ImageTragick (2016, CVE-2016-3714): ImageMagick's broad format support enabled RCE through crafted image files. It is the classic argument for small format allow-lists. **[bg]** — [imagetragick.com](https://imagetragick.com/)
- The Rust `zip` crate provides `ZipFile::enclosed_name()`, which returns `None` for absolute or `..`-escaping paths. Extraction code should use it rather than the raw name. **[bg]** — [docs.rs/zip](https://docs.rs/zip/latest/zip/read/struct.ZipFile.html)
- Luanti restricts sounds to Ogg Vorbis, a deliberately narrow format surface. **[snippet]** — [Luanti client_lua_api.md](https://github.com/luanti-org/luanti/blob/master/doc/client_lua_api.md)
- TUF's "endless data" defence: declared lengths in signed metadata, and clients stop reading past them. **[fetched]** — [TUF spec](https://theupdateframework.github.io/specification/latest/)

### Inferences (recommended limits and architecture)
- **Container.** Don't use zip as the on-disk or extract format. Transfer the package as manifest + individual blobs, each verified by BLAKE3 against the manifest (streaming, abort on overrun). Store blobs as `cache/blobs/<hash>`. Paths in the manifest are *logical asset names* that are only looked up in an in-memory map and never joined onto filesystem paths. This removes zip slip, symlinks, case-folding collisions and overlapping-entry bombs as a class. If an archive is used for authoring import, validate names (UTF-8, `[a-z0-9_./-]`, no `..`, no leading `/`, no drive letters, NFC, max depth/length) and reject duplicates after case folding.
- **Format allow-list.**
  - Images: PNG only (optionally also KTX2/QOI if needed). WebP only via the pure-Rust `image-webp`, never libwebp.
  - Audio: Ogg Vorbis/Opus via pure-Rust decoders (`lewton` or `symphonia`).
  - Models: the game's own schema in JSON/CBOR, or glTF *without* external URIs.
  - Text: JSON with depth and size limits.
  - No SVG, no fonts in v1 (font parsers are historically bug-prone), no executable-adjacent formats.
- **Suggested starting limits (tune with real packs):**
  - Package total ≤ 64–256 MiB (Minecraft allows 250 MiB per pack **[fetched]**, [Minecraft Wiki](https://minecraft.wiki/w/Server.properties)).
  - Per-world total download cap, with a user setting.
  - File count ≤ 4,096 per package.
  - Single file ≤ 16 MiB.
  - Texture ≤ 4096×4096. Block textures should be power-of-two and ≤ 512.
  - Decoded image bytes ≤ 64 MiB each (set `image::Limits` width/height strictly and lower `max_alloc` from its 512 MiB default).
  - Compression ratio cap (e.g. decoded/encoded ≤ 100×), checked incrementally.
  - Audio ≤ 10 min, ≤ 48 kHz, ≤ 2 channels, with a decoded PCM cap.
  - JSON ≤ 1 MiB per file, nesting depth ≤ 64, with string/array length caps. Mesh vertex/index counts capped.
  - Total decoded GPU memory budget per world.
- **Isolation.** Rust removes most memory-safety risk but not panics, infinite loops or OOM. Decode on a worker thread with `catch_unwind`, a time budget and allocation caps. Ideally, also do a one-time "import" pass in a separate low-privilege process that re-encodes assets into a canonical internal format (e.g. raw RGBA or KTX2, PCM/Opus). After that, only trusted, re-encoded data reaches the renderer. The re-encoded cache can be keyed by the original hash.
- **Offensive content can't be solved technically.** Use creator self-declared rating tags, reviewer endorsements and per-player hiding. Also let a player block a creator key or package hash, which then shows a placeholder.

### Gaps
- I didn't verify current fuzzing/OSS-Fuzz status or the recent RUSTSEC advisory history for `png`, `image`, `image-webp`, `symphonia`, `lewton` or `zip`. Check rustsec.org and each repo before freezing the stack.
- No authoritative "recommended limits" standard exists. The numbers above are engineering suggestions, not sourced norms.

---

## 5. Web-of-trust, reviewer-signed lists, impersonation

### Takeaway
Decentralised systems (Nostr, Scuttlebutt, PGP's WoT) show that:
- A key is not an identity.
- Display names are trivially spoofable.
- Global trust scores are hard to compute.
- Personalized trust works: "people/reviewers *I* follow" is more robust than global scores.

So reviewer lists should be signed, versioned and expiring (TUF-style), opt-in per player, and displayed through local petnames and fingerprint glyphs, never by bare display name.

### Cited Findings
- Nostr's web of trust is derived from social activity. Personalized PageRank is seeded from NIP-05-verified accounts and propagated through the follow graph, and the article states WoT "is not a solved problem". Hosted WoT services (Vertex) exist for clients. **[snippet]** — [Damus article](https://damus.io/naddr1qqgrwd34vvek2wf5ve3nwcmrxycrxq3qklkk3vrzme455yh9rl2jshq7rc8dpegj3ndf82c3ks2sk40dxt7qxpqqqp65wrp5t4t)
- A 2024 draft NIP proposed anchoring npubs to a Bitcoin address as an anti-spam credibility signal for new users. This parallels linking a game identity to a Chia DID. It is a draft, not adopted. **[snippet]** — [nostr.at](https://nostr.at/nevent1qqsqke8fx597a03f2tv0n42k3hat3hyg94gy5wy9vruraskzv776l7cudltdv)
- Nostr NIP-51 defines signed, replaceable lists (mute lists, follow sets, bookmark sets, etc.) published by a key. This is a direct model for "reviewer publishes a signed list of package hashes". **[bg]** — [NIP-51](https://github.com/nostr-protocol/nips/blob/master/51.md)
- Scuttlebutt: each identity is an Ed25519 key with a signed append-only feed. Names are assigned by others and locally (petnames), so there is no global name registry. **[bg]** — [Scuttlebutt Protocol Guide](https://ssbc.github.io/scuttlebutt-protocol-guide/)
- Zooko's triangle / petname systems: names can't be simultaneously secure, decentralized and human-meaningful. Petnames (local names bound to keys) are the standard resolution. **[bg]** — [Petname systems (Stiegler)](http://www.skyhunter.com/marcs/petnames/IntroPetNames.html)
- TUF delegations and thresholds give a ready model for "reviewer group": a delegated role signing a list, revocable by omission, with an N-of-M threshold. **[fetched]** — [TUF spec](https://theupdateframework.github.io/specification/latest/)
- W3C's DID work includes verifiable-credential-based trust registries (e.g., Trust over IP). A DID can hold credentials ("verified creator") issued by other DIDs. **[bg]** — [W3C DID Core](https://www.w3.org/TR/did-core/)

### Inferences (recommendations)
- **Reviewer list format.** `{magic:"weft-reviewlist/v1", reviewer_key, list_id, version (monotonic), expires, entries:[{manifest_hash, verdict: ok|mature|block, tags, note}], prev_hash}`, signed with domain `reviewer-list`. Clients:
  - Reject lower versions (rollback).
  - Treat lists past expiry as stale ("endorsement expired") rather than invalid (freeze).
  - Support N-of-M group lists.
  - Distribute lists alongside world data or via peer gossip, with no central host.
- **Impersonation defences:**
  1. Always render creators with a stable visual fingerprint: e.g. a short base32 of the key plus a deterministic glyph/colour (identicon) or word pair from the key hash.
  2. Let the player assign a local petname on first trust, and show the petname afterwards.
  3. If a new key presents a display name equal to, or confusable with (Unicode skeleton per UTS #39), a creator the player already trusts, show an explicit warning: "Not the Ava you trust".
  4. After the Chia DID link, show "DID-linked" status. It doesn't make the creator *good*, but it makes the identity costly to mint (anti-Sybil) and recoverable.
- **Spam and Sybil.** Free keys mean anyone can mint creators and reviewers. Trust must flow only from things the player chose: their own trust decisions, reviewers they subscribed to, or the world host they're joining (implicitly, "the host vouches for its pins"). There should be no global "most-endorsed" ranking, because that can be gamed. An optional default reviewer subscription (e.g. the game project's own reviewer key) is the pragmatic bootstrap. Users should see it and be able to remove it.
- **Key compromise.** Combine creator-signed rotation, DID-based recovery and reviewer lists. Reviewers can mark "manifests signed by key K after date/version V are untrusted", which the client applies as a re-prompt.

### Gaps
- No empirical data found on spam and impersonation rates in Nostr or SSB clients, or on how well petname UIs work in practice.
- I didn't check whether any game currently ships reviewer-signed content lists. I found no example.
