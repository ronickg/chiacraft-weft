# Finding worlds on testnet11: the two-copy check

Phase 5, slices 1-2 (`notes/PLAN.md`, `reports/Finding worlds.md`), against the real testnet11
chain and our sponsor. Alice gets a DID and a world DID (as in `tools/checks/did`), lists her
world, and bob finds it in Find Worlds (from Multiplayer) and joins it from there. Alice then
delists it and gets the deposit back. 20-35 minutes; nothing here touches rendering or shaders;
it needs the internet. No one sends coins: the check pays for itself.

The game's deposit is 0.0001 TXCH (`core:listing_deposit`), more than a fresh account has on
purpose. This check sets it to 0.000001 TXCH in both copies (`listing_deposit = 1000000`): alice
pays it from what the sponsor gave with her world DID (0.00001 TXCH, less the 0.000005 fee of her
DID's listing spend), and bob's finder reads listings of at least that much. Spending stays
testnet11 only.

From the folder holding `target/release/weft`, with fresh run folders:

```sh
rm -f finder-mark-*
mkdir -p run-finder-alice/config run-finder-bob/config
printf "name = alice\nlisting_deposit = 1000000\n" > run-finder-alice/config/core.txt
printf "name = bob\nlisting_deposit = 1000000\n" > run-finder-bob/config/core.txt
./target/release/weft --run-dir run-finder-alice --world finder-alice-1 --script tools/checks/finder/alice.txt &
./target/release/weft --run-dir run-finder-bob --script tools/checks/finder/bob.txt
```

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
- `finder-alice-1-listed`: Who May Come In, "Finding this world": listed, 0.000001 TXCH locked
- `finder-bob-1-found`: Find Worlds: alice's world, "online: 1 playing", "listed 0 days ago, 0.000001 TXCH locked", its rules
- `finder-bob-2-in`: bob in alice's world, joined from the finder
- `finder-alice-2-delisted`: "Finding this world": not listed

If a wait runs out before the chain confirms (a slow block), the later steps fail: the logs say
where it stood.
