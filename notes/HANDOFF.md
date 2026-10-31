# Handoff: chiacraft on Weft

Read `notes/PLAN.md` first: it is the plan and the decisions. This page says where the work stands.

## Where things are (2026-10-31)

- This repository is a fork of [Weft](https://github.com/VortexSo4/Weft) at `f213253`, with Weft's
  history; remote `upstream` points at Weft. Weft's files are MPL-2.0; our crates are MIT.
- Commits on top of Weft:
  1. The texture pack Weft built from a player's Minecraft copy is removed.
  2. `crates/wallet` (`chiacraft-wallet`): the wallet core from the Horizon-based chiacraft
     (`horizon-wallet` at `101dee332`), renamed. Env vars still use the `HORIZON_` prefix
     (`HORIZON_WALLET_STORE`, `HORIZON_WALLET_PASSPHRASE`, the live test tool's); rename later.
  3. `notes/PLAN.md`.
  4. Phase 1, the account (see below).

## Phase 1 so far: written, NOT compiled or tested yet

The cloud session that wrote it could not reach crates.io until the end, so nothing below has been
through the compiler. Expect some compile errors; fix them first.

- `crates/wallet/src/sign.rs`: `IdentitySigner::network_seed` (SHA-256 of
  `"chiacraft network key v1"` + identity key 0's secret: the Ed25519 seed of the iroh endpoint),
  `network_link_text`, `verify_network_link`; test `network_key_follows_the_words_and_is_vouched_for`.
- `crates/account` (`chiacraft-account`, new): opens the account once per process (made quietly on
  the first start, `<run dir>/account` or `CHIACRAFT_ACCOUNT_HOME`; secret in the OS keystore, or the
  file store when `use_file_store()` was called, as automated runs do); `Session` gives the network
  seed, `vouch_line`, the words, `mark_backed_up`, `session_ended`, `wants_backup`, `restore`;
  `check_vouch` for worlds. One test.
- Engine (`crates/engine`):
  - `net/p2p.rs`: `identity()` returns the account's network key instead of Weft's random
    `config/identity.key`. `PROTOCOL` 3.
  - `app.rs`: opens the account at start (file store in automated runs); `leave_world` counts a
    play session.
  - `net/mod.rs`, `client/mod.rs`, `server/mod.rs`, `net/codec.rs`: `ToServer::Join` has
    `account` (the identity vouching for the key); the server checks it against the authenticated
    key and keeps `Player.identity`.
  - `account_screen.rs` (new): screen `core:account`, opened from Profile: fingerprint, identity,
    store, show the 24 words (on screen only, no copy), "I wrote them down", restore from words.
  - `screens.rs`: Profile's "Account and Words" button; the title menu's "Write Down Your Words"
    after the first session until backed up.
  - `assets/lang/en_us.lang`: the texts.

## Next steps

1. `cargo test -p chiacraft-wallet -p chiacraft-account --lib`, then `cargo build --workspace` and
   `cargo test -p weft_engine`; fix what fails. `cargo clippy --workspace`.
2. Run it: `cargo run --release` from the repository root. Check on screen: Profile shows the key
   and "Account and Words"; the Account screen; the words; restore; the title menu reminder after
   leaving a world once. The account must survive a restart (same key).
3. Two copies on one machine with different `CHIACRAFT_ACCOUNT_HOME` (and run dirs): host with
   Open to Friends, join with the ticket, and check the host log shows the joining player's identity
   (`Player.identity` set, no "does not check out" warning).
4. Then phase 2 (packages) per the plan.

## Notes

- Weft draws with Vulkan; on macOS it needs MoltenVK (Weft's release workflow bundles it; for
  `cargo run` from source, check its docs or install `molten-vk`).
- The Horizon-based project in `~/Coding/horizon` is frozen as it was: batch C (W7-W9) half-done in
  its worktrees, uncommitted; don't prune those worktrees.
- No GitHub remote for this repository yet: create one and push.
