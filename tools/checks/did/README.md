# DIDs on testnet11: the two-copy check

Phase 4, slices 4-7 (`notes/PLAN.md`), against the real testnet11 chain and our sponsor (its
address is the game's default `core:sponsors`; it runs on Ronald's Mac, see `notes/HANDOFF.md`,
"Our DID sponsor"). Two copies: alice gets a DID from the sponsor, makes her world ask for DIDs and
gets her world a DID, which her DID then lists; bob is refused for having none, gets one, gets in
(his DID checked on chain at the join), and sees alice's world's DID confirmed both ways in his
saved worlds list. Testnet blocks come about once a minute, so it takes 15-25 minutes. Nothing
here touches rendering or shaders; it needs the internet (iroh relays, Chia full nodes).

From the folder holding `target/release/weft`, with fresh run folders:

```sh
rm -f did-ticket.txt did-mark-*
mkdir -p run-did-alice/config run-did-bob/config
echo "name = alice" > run-did-alice/config/core.txt
echo "name = bob" > run-did-bob/config/core.txt
./target/release/weft --run-dir run-did-alice --world did-alice-1 --script tools/checks/did/alice.txt &
./target/release/weft --run-dir run-did-bob --script tools/checks/did/bob.txt
```

## What to look for

Alice's log:
- `DID did:chia:1… from sponsor d468728d7e0e (pending)`, then `DID did:chia:1… is on the chain, held by this account`
- `world DID did:chia:1… from sponsor d468728d7e0e`, then `world DID did:chia:1… is listed in its owner's DID`
- `bob (…) is not let in: admission.did_required`
- `player … proved did:chia:1… (0 days old)` and `bob joined`

Bob's log:
- `disconnected: admission.did_required`
- `DID did:chia:1… is on the chain, held by this account`

Shots:
- `did-alice-1-account`: the Account screen: her DID and "… is on the chain and yours (0 days old)."
- `did-alice-2-world`: Who May Come In: "DID required: On", the world's DID, "… and your DID name each other on the chain."
- `did-bob-1-refused`: "This world lets in only players with a DID (set one up on the Account screen)."
- `did-bob-2-account`: his DID on the chain
- `did-bob-3-in`: bob in alice's world
- `did-bob-4-saved`: Multiplayer: alice's world "online: … ; a DID its owner's DID confirms"

If a wait runs out before the chain confirms (a slow block), the later steps fail: the logs say
where it stood. The sponsor's own log is `~/chiacraft-testnet-sponsor/sponsor.log` on the Mac.
