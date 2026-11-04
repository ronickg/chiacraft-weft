# chiacraft-sponsor

chiacraft's DID sponsor: a small service that gives a new player a DID on **testnet11** without the player owning
any coins. The sponsor launches the DID **straight to the player's address** (chia-wallet-sdk's
`Launcher::create_eve_did`): the player signs only the request, with the game's identity key; the sponsor pays a
mojo for the DID's coin and the fee, and never holds the DID: from its launch its puzzle names the player's
address as the owner, so only the player's key can ever spend it. Requests arriving together go in one spend
bundle. A daily budget caps what it spends. Anyone can run one: we run ours, and a server can run its own for its
players. It is reached over **iroh**, by its endpoint id, as the game reaches worlds: no web server, no domain, no
TLS certificate, no open port (iroh's relays carry it when a direct path can't be made).

It was Horizon's `horizon-sponsor` (an HTTP service); the service is the same, the transport is iroh. Keys, coin
selection, the eve-DID spend and the chain reader come from `chiacraft-wallet` (feature `chain`; the spend is
`chiacraft_wallet::chain::sponsor`), and the request and the answer are the wallet's `sponsor` module, which the
game uses too.

The crate is two parts:

- **The client** (always built; iroh and the wallet without `chain`): `chiacraft_sponsor::ask` and
  `chiacraft_sponsor::client::status`, for the game.
- **The service** (feature `service`): the sponsor and the `chiacraft-sponsor` binary
  (`cargo build --release -p chiacraft-sponsor --features service`).

## The protocol

ALPN `chiacraft/sponsor/1`, one bidirectional stream per connection:

1. The asker writes its request and finishes its side of the stream: a DID request's JSON (at most 4 KiB), or the
   6 bytes `status`.
2. The sponsor answers with one JSON object and finishes: `{"status": <code>, "body": <JSON>}`. The codes and bodies
   are the ones Horizon's HTTP API gave.
3. The asker closes the connection (the sponsor waits a few seconds for it).

The sponsor gives each step of an asker 10 seconds (the connection, the stream, the request). A DID request's
answer comes once its bundle is pushed: usually in a few seconds, at most `max_wait_seconds` and the chain's time
(the client waits up to 200 s).

| Request | Answer |
| --- | --- |
| `{ "network": "testnet11", "address": "txch1…", "identity": "<96 hex>", "signature": "<192 hex>", "time": <unix seconds> }` | `200 { "did": "did:chia:1…", "launcher": "<64 hex>", "coin": "<64 hex>", "status": "pending" }` |
| `status` | `200 { network, phase, height, address, balance, spendable, budget_left_today, sponsored_today, pending, queued, fee_per_did, per_endpoint_per_day }` |

**The signature.** The identity key (the game's `chiacraft_wallet::Signer`, identity key 0 by default) signs the
text

    chiacraft sponsor v1 <address> <time>

with the address exactly as sent and `time` in decimal, as every identity text is signed: a Chia signed message
(CHIP-0002), BLS AugSchemeMPL over the CLVM tree hash of `("Chia Signed Message" . text)`. `identity` is the key's
48 bytes in hex, `signature` the 96-byte signature in hex: `chiacraft_wallet::sponsor::Request::sign` makes it.
`address` is the player's standard address on the network (`txch1…`, lower case): its puzzle hash becomes the
DID's owner. `launcher` is the DID's id; `coin` is its eve coin (the DID's coin until the player first spends it).

**Refusals**, each with the body `{ "error": "<why>" }`:

| Status | When |
| --- | --- |
| `400` | Not JSON or a field missing; larger than 4 KiB; `address` not a testnet11 address; `identity` not a public key; `time` more than `max_skew_seconds` (300) from the sponsor's clock; `signature` not the identity key's signature of the text |
| `403` | The identity already has a sponsored DID (the body also carries its `did` and `launcher`); `network` isn't the one served |
| `429` | A request for this identity is already in progress; the asking endpoint's DIDs for today (`per_endpoint_per_day`); today's budget (`daily_budget_mojos`, UTC day) is spent; too many requests waiting |
| `503` | The sponsor isn't synced with the chain; none of its coins was free within `max_wait_seconds` (all in unconfirmed spends, or it is out of funds); the chain refused the bundle |

A DID whose launch isn't confirmed within `drop_after_seconds` is **dropped**: its identity may ask again and its
cost goes back to the day's budget.

## How a game finds it and asks

A sponsor's address is its endpoint id. The game's setting `chiacraft:sponsors` holds the sponsor addresses it
asks. With its own iroh endpoint, the game asks one:

```rust
let request = chiacraft_wallet::sponsor::Request::sign(&signer, "testnet11", &address, now)?;
let (status, body) = chiacraft_sponsor::ask(&endpoint, sponsor_id, &request).await?; // Err: no answer
let reply = chiacraft_wallet::sponsor::read_answer(status, &body)?; // a DID, the one it already has, or why not
```

`read_answer` turns a `403` naming the identity's existing DID into a reply with status `existing`, for the game
to watch and link that one (after checking on chain that the player's address holds it, as for a new one).

## Running it

```sh
cp crates/sponsor/sponsor.example.toml sponsor.toml   # every setting, with its default
cargo build --release -p chiacraft-sponsor --features service
./target/release/chiacraft-sponsor --config sponsor.toml --address
#   → the sponsor's address (endpoint id): give it to games (chiacraft:sponsors)
CHIACRAFT_SPONSOR_WORDS_FILE=/path/to/words ./target/release/chiacraft-sponsor --config sponsor.toml --funding-address
#   → txch1…: fund this address with test coins (a testnet11 faucet, docs.chia.net/testnets)
CHIACRAFT_SPONSOR_WORDS_FILE=/path/to/words ./target/release/chiacraft-sponsor --config sponsor.toml
```

- **Its address** is the endpoint id of the key in `<data_dir>/sponsor.key` (32 random bytes, made the first time,
  readable by its owner only), so it stays the same across restarts; it is printed at start and by `--address`.
  Back the file up with the log: a new key is a new address that games don't know. A damaged key file refuses to
  start rather than quietly changing it.
- **The funding wallet** is a 24-word Chia wallet of its own (make one in Sage, or any BIP-39 tool). Its words
  come from `CHIACRAFT_SPONSOR_WORDS`, or a file named by `CHIACRAFT_SPONSOR_WORDS_FILE` (or `words_file` in the
  config; Horizon's `HORIZON_SPONSOR_*` names are read when these aren't set); they are read once at start, held in
  the wallet crate's redacting `Secret`, and never logged. It is the only wallet key the sponsor holds: no player
  key ever reaches it. Keep only what a few days of budget need in it.
- **testnet11 by default; mainnet only opted in**: `network = "mainnet"` refuses to start unless
  `allow_mainnet = true` is set too. On mainnet the budget fields left out default small (`fee_per_did`
  1,000,000 mojos, `daily_budget_mojos` 50,000,000 = 0.00005 XCH, `per_endpoint_per_day` 1, `max_batch` 5), and
  `max_fee_per_did` (100,000,000 = 0.0001 XCH on mainnet, unlimited on testnet11 unless set) is a hard cap: a
  higher `fee_per_did` refuses to start, and the chain reader refuses any bundle paying more per DID. The start
  logs a REAL XCH warning with the budgets. Addresses must be `xch1…` on mainnet, `txch1…` on testnet11 (the
  signed text binds the address, so a request can't be replayed on the other network).
- **Chain access** is the wallet crate's chain reader: testnet11 full nodes through the DNS introducer over TLS
  (outgoing TCP 58444), or `peers` / `CHIACRAFT_CHAIN_PEERS` (`host:port`; `ws://127.0.0.1:port` for a local
  simulator). Requests get `503` until it has synced (a few seconds).
- **The network**: iroh with n0's preset (its relays and address lookup), outgoing UDP (and HTTPS to the relays).
  Nothing to open or forward.
- **Its data** (`data_dir`): `sponsor-log.jsonl`, one JSON line per event (a DID launched: identity, address,
  launcher, coin, cost, a daily hash of the asking endpoint's id, transaction; a bundle: its coins, fee and change;
  confirmations; drops), replayed at start (Horizon's logs replay too); `chain-cache.json`, the reader's cache; and
  `sponsor.key`. Back up the log: it is what remembers who was served. No endpoint id is stored, only a hash of
  the day and the id.
- **The per-endpoint limit** counts DIDs per asking iroh endpoint per day. Endpoint ids cost nothing to make, so
  it only slows a careless client (Horizon counted IP addresses); what holds is one DID per identity and the daily
  budget.
- **Logs** go to standard error (`CHIACRAFT_SPONSOR_LOG` = `info` by default, `debug`, `warn`; iroh's own lines
  are left out): the address, bundles pushed (count, transaction, fee), confirmations, drops. No key, words,
  spend or signature is ever logged (`tests/no_secret_leaks.rs` checks the source and a running sponsor's output
  and files).
- **Coins.** Each bundle spends some of the wallet's coins and returns the change; until the chain confirms it (a
  block or two on testnet11) those coins are busy, and requests wait for a free one. With one large coin the
  sponsor still launches up to `max_batch` DIDs per block; split the funding into several coins to launch more
  often.

## Running your own (server operators)

A server that wants its new players to have DIDs runs its own sponsor with its own funding wallet, budget and
limits: the same binary and config, and gives its players the sponsor's address. A DID launched by any sponsor is
an ordinary DID owned by the player. On chain, anyone can see which wallet paid for a DID's launch (the launcher's
parent coin), so a server can admit only DIDs its own sponsor made if it wants.

## Tests

`cargo test -p chiacraft-sponsor --features service` (offline, throwaway keys, chia-sdk-test's peer simulator,
iroh endpoints on loopback with no relay): a request launches a DID owned by the player's address, which the
player can spend and the sponsor can't; a second request for the same identity is refused; bad signatures, stale
times, other networks and bad addresses are refused; the budget and the per-endpoint limit answer 429; requests
arriving together share one bundle; a restart keeps identities, budget and pending launches; no free coin answers
503; the protocol over iroh matches this page; the address survives a restart; mainnet only opted in and within
its caps; no secret in logs or files. A live testnet11 run needs a funded throwaway testnet wallet and is not
automated.
