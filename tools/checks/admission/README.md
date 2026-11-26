# Admission rules, bans and ban lists: the two-copy check

Phase 4, slices 1-3 (`notes/PLAN.md`). Two copies of the game at once on one machine: alice hosts
her world and sets rules; bob visits, is refused, banned, let in, and finally follows alice's ban
list from a world of his own. About four minutes. Nothing here touches rendering or shaders.

From the folder holding `target/release/weft` (the worktree), with fresh run folders:

```sh
rm -f adm-ticket.txt adm-bob-ticket.txt adm-mark-*
mkdir -p run-adm-alice/config run-adm-bob/config
echo "name = alice" > run-adm-alice/config/core.txt
echo "name = bob" > run-adm-bob/config/core.txt
./target/release/weft --run-dir run-adm-alice --world adm-alice-1 --script tools/checks/admission/alice.txt &
./target/release/weft --run-dir run-adm-bob --script tools/checks/admission/bob.txt
```

Start them in either order: bob waits for alice's ticket, and from there each copy waits for the
other's step through `adm-mark-*` files (testkit's `mark` and `await_file`).

## What to look for

Alice's log (the world's server):
- `bob is on probation: not yet (building)`, twice (once before the ban, once after he is let back in)
- `alice banned bob`, then `bob is sent away: admission.banned`
- `bob (…) is not let in: admission.banned`, then after the unban `bob (…) is not let in: admission.not_listed`
- `bob joined (account identity …)` twice (first visit, after `/allow add bob`)
- `bob is off probation`
- `published ban list 'Alice test list' version 1 (1 entries), maintainer …`

Bob's log:
- `disconnected: admission.banned`, `disconnected: admission.banned`, `disconnected: admission.not_listed`
- after he opens his own world: `following the ban list of its owner from …`, then
  `ban list 'Alice test list' version 1 fetched`

Shots (`adm-*.png`):
- `adm-bob-1-probation`: the chat line "You are new here: building and breaking open up after …", the block not broken
- `adm-bob-2-banned`: "Disconnected: You are banned from this world: testing bans"
- `adm-bob-3-refused`: the same refusal on joining again
- `adm-bob-4-not-listed`: "This world lets in only the players on its allow list."
- `adm-bob-5-free`: a hole where bob broke a block
- `adm-bob-6-following`: chat: `"Alice test list" by …: 1 players, version 1`
- `adm-alice-1`, `adm-alice-2`: the command replies in alice's chat

In testkit a negative pitch faces down: bob's `look 0 -85` faces the ground at his feet.

The owner's screen (pause menu, "Who May Come In") isn't driven by these scripts; open it in
alice's game to see the rules, bans, allow list and followed lists.
