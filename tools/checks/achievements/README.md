# Achievements signed by worlds: the two-copy check

Phase 4 slices 8-9 (`notes/PLAN.md`). Alice hosts a fresh world and gets herself a crafting table
(the base game's "A Place to Work"): her own world signs it, self-signed. Bob joins and gets one
from her: her world signs it for him, and his game keeps it. Both open Profile > Achievements.
About a minute; no chain needed (the world DID check needs one only for a world with a DID); no
rendering or shader changes.

```sh
rm -f ach-ticket.txt ach-mark-*
mkdir -p run-ach-alice/config run-ach-bob/config
echo "name = alice" > run-ach-alice/config/core.txt
echo "name = bob" > run-ach-bob/config/core.txt
./target/release/weft --run-dir run-ach-alice --world ach-alice-1 --script tools/checks/achievements/alice.txt &
./target/release/weft --run-dir run-ach-bob --script tools/checks/achievements/bob.txt
```

Look for, in alice's log: `alice earned 'base:workbench': signed` and `bob earned 'base:workbench': signed`;
in her game's log too `achievement 'base:workbench' from 'ach-alice-1' kept`. In bob's:
`achievement 'base:workbench' from 'ach-alice-1' kept`. Files: `run-ach-bob/test-account/achievements.json`
(one credential, issuer name ach-alice-1), `run-ach-alice/saves/ach-alice-1/achievements/issued.jsonl` (two lines).
Shots: `ach-alice.png` "ach-alice-1: your own world" with "A Place to Work"; `ach-bob.png` "ach-alice-1:
signed by its owner, no world DID" with "A Place to Work" and today's date.

The world-DID line ("its DID and its owner's confirm each other on the chain") needs a world with a DID
(`tools/checks/did` makes one) and the daily anchor a day's wait with `/rule anchor_achievements 1`.
