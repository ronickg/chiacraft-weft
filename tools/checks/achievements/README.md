# Achievements signed by worlds: the two-copy check

Phase 4 slices 8-9 and the rule "has achievement X" (`notes/PLAN.md`). Alice hosts a fresh world
and gets herself a crafting table (the base game's "A Place to Work"): her own world signs it,
self-signed. Bob joins and gets one from her: her world signs it for him, and his game keeps it.
Both open Profile > Achievements. Then bob leaves; alice asks for Timber! at the door (he is
refused), then for A Place to Work (his game shows the credential her world signed, and he is let
in). About a minute; no chain needed (the world DID check needs one only for a world with a DID); no
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
Then in alice's log: `only players with the achievement base:timber get in now (1 issuers trusted)`,
`bob (…) is not let in: admission.achievement_required`, `… showed 0 achievement(s)` then `… showed 1
achievement(s)`, `bob joined` a second time; in bob's: `disconnected: admission.achievement_required`.

Last, alice withdraws it (`/revoke bob base:workbench`): her log `1 achievement(s) withdrawn
(revocation list version 1)`, bob's `1 of your achievements were withdrawn by their issuer`; his
next join is refused (`admission.achievement_required`) and `… showed 0 achievement(s)`.

Shots: `ach-bob-withdrawn.png` his A Place to Work marked "(withdrawn by the world)";
`ach-bob-refused2.png` the refusal again. `ach-bob-refused.png` "This world lets in only players who earned "Timber!" in a world it trusts";
`ach-bob-back.png` bob back in alice's world. `ach-alice.png` "ach-alice-1: your own world" with "A Place to Work"; `ach-bob.png` "ach-alice-1:
signed by its owner, no world DID" with "A Place to Work" and today's date.

The world-DID line ("its DID and its owner's confirm each other on the chain") needs a world with a DID
(`tools/checks/did` makes one) and the daily anchor a day's wait with `/rule anchor_achievements 1`.
