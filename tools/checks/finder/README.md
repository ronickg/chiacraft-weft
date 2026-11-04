# Finding worlds on testnet11: the two-copy check

Phase 5, slices 1-2 (`notes/PLAN.md`, `reports/Finding worlds.md`), against the real testnet11
chain and our sponsor. Alice gets a DID and a world DID (as in `tools/checks/did`), is paid a
little test XCH by the tester, lists her world (a 0.0001 TXCH deposit, setting
`core:listing_deposit`, testnet11 only), and bob finds it in Find Worlds (from Multiplayer)
and joins it from there. Alice then delists it and gets the deposit back. 25-40 minutes;
nothing here touches rendering or shaders; it needs the internet.

From the folder holding `target/release/weft`, with fresh run folders:

```sh
rm -f finder-mark-*
mkdir -p run-finder-alice/config run-finder-bob/config
echo "name = alice" > run-finder-alice/config/core.txt
echo "name = bob" > run-finder-bob/config/core.txt
./target/release/weft --run-dir run-finder-alice --world finder-alice-1 --script tools/checks/finder/alice.txt &
./target/release/weft --run-dir run-finder-bob --script tools/checks/finder/bob.txt
```

## The tester's one step: pay alice

The sponsor's gift (0.00001 TXCH) is too little for a deposit, on purpose. When
`finder-mark-need-coins` appears, send her 0.0002 TXCH from a throwaway testnet wallet (never
print its words) and touch the mark the script waits for:

```sh
cargo build --release -p chiacraft-wallet --features tool --bin chiacraft-wallet-test
ADDR=$(cat run-finder-alice/test-account/address-testnet11.txt)
./target/release/chiacraft-wallet-test send "$ADDR" --amount 200000000 --fee 0 --words-file ~/chiacraft-testnet/server.words --yes
touch finder-mark-funded
```

(If the run uses the OS keystore the file is in `run-finder-alice/account/` instead.)

## What to look for

Alice's log:
- `world DID did:chia:1… is listed in its owner's DID`
- the listing pushed, then seen on chain (`listing.json` in her world folder, `confirmed: true`)
- after the delist, the listing gone and its coin spent back to her

Bob's log:
- `finder: 1 listed worlds check out, 0 left out` (more if others listed on testnet11)
- `finder: 1 of 1 listed worlds online`
- `bob joined` in alice's log

Shots:
- `finder-alice-1-listed`: Who May Come In, "Finding this world": listed, 0.0001 TXCH locked
- `finder-bob-1-found`: Find Worlds: alice's world, "online: 1 playing", "listed 0 days ago, 0.0001 TXCH locked", its rules
- `finder-bob-2-in`: bob in alice's world, joined from the finder
- `finder-alice-2-delisted`: "Finding this world": not listed

If a wait runs out before the chain confirms (a slow block), the later steps fail: the logs say
where it stood.
