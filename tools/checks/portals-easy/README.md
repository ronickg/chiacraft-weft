# Linking at the portal: the two-copy checks

`notes/portals-easy-link.md`, option B. Each run uses two copies of the game on one machine
(alice and bob), with fresh run folders and world names. Nothing here touches rendering or
shaders. Each run takes about three minutes.

Run from the folder holding the release build (the worktree), through the testers' launcher.

## Run A: a stranger asks, the owner answers

Bob is not alice's friend. He places a portal in her world and links his own world there. Alice
accepts at the portal.

Before the run:
1. Delete `pe-ticket.txt` and `pe-mark-*`.
2. Make the run folders `run-pe-alice/config` and `run-pe-bob/config`.
3. Write `name = alice` (and `name = bob`) into each folder's `core.txt`.

Start in this order:
1. Alice: `--run-dir run-pe-alice --world pe-alice-1 --script tools/checks/portals-easy/ask-alice.txt`, in the background.
2. Bob: `--run-dir run-pe-bob --script tools/checks/portals-easy/ask-bob.txt`.

Alice's log:
- `link request from 'pe-bob-home' (owner …) for 'pe-alice-1'`. Not "accepted": bob is a
  stranger.
- After her Accept: `linked 'pe-alice-1' with 'pe-bob-home'`, then
  `the portal at [5, 151, 4] now leads to 'pe-bob-home' (1 blocks)`.

Bob's log: `'pe-alice-1' accepted the link with 'pe-bob-home'`.

Shots:
- `pe-bob-1-screen`: the visitor's portal screen, with "Link 'pe-bob-home' here".
- `pe-bob-2-asked`: chat says "Asked to link 'pe-bob-home' here …".
- `pe-alice-1-asks`: alice's chat says "'pe-bob-home' asks to lead a portal there …".
- `pe-alice-2-screen`: the owner's screen with the request, and Accept / Always / Decline / Block.
- `pe-alice-3-linked`, `pe-alice-4-leads`: "A portal here now leads to 'pe-bob-home'", and the
  screen says "This portal leads to 'pe-bob-home'".
- `pe-bob-3-bound`: bob's chat says "The portal now leads to 'pe-bob-home'."

## Run B: friends

Bob has two worlds and hosts one. Alice and bob add each other as friends: each visits the other
once, then taps "Add friend" in the Multiplayer list.
- **Flow 1.** Bob touches alice's portal and it links at once.
- **Flow 3.** Alice asks bob's other world, which nobody has visited, from her portal screen, and
  bob's game accepts it without asking.
- Then bob walks through the first portal, home.

Before the run:
1. Delete `pe2-*.txt` and `pe2-mark-*`.
2. Make the run folders `run-pe2-alice/config` and `run-pe2-bob/config`, with names as in run A.

Start in this order:
1. Bob: `--run-dir run-pe2-bob --script tools/checks/portals-easy/friends-bob.txt`, in the background.
2. Alice: `--run-dir run-pe2-alice --script tools/checks/portals-easy/friends-alice.txt`. Give no
   `--world`: alice opens `pe2-alice-1` after visiting bob.

Alice's log:
- `added a friend (identity …), met through 'pe2-bob-home'`.
- `link request from 'pe2-bob-home' for 'pe2-alice-1': accepted (its owner is a friend)`.
- `the portal at [4, 151, 2] now leads to 'pe2-bob-home'`.
- Later `'pe2-bob-other' accepted the link with 'pe2-alice-1'`, then
  `the portal at [6, 151, 2] now leads to 'pe2-bob-other'`.

Bob's log:
- `added a friend …, met through 'pe2-alice-1'`.
- `link request from 'pe2-alice-1' for 'pe2-bob-other': accepted (its owner is a friend)`.
- At the end, the trip home through the portal.

Shots:
- `pe2-bob-1-screen`: both of bob's worlds offered.
- `pe2-bob-2-linked`: "The portal now leads to 'pe2-bob-home'", with no answer from alice
  needed.
- `pe2-alice-3-friends`: "Or ask a friend's world:" with "Link with 'pe2-bob-other'".
  - If the list is empty, bob's game hadn't answered yet. Open the screen again: the script
    already does that once.
- `pe2-alice-4-bound`, `pe2-alice-5-leads`: the second portal leads to 'pe2-bob-other'.
- `pe2-bob-3-home`: bob is back in his own world.

## Aiming

In testkit a negative pitch faces down. Look yaw 0 faces -z.
- The platform is stone at y 150, from 0 0 to 8 8.
- Run A: bob places at 5 151 4. Alice aims at it from 5.5 151 7.5 at `look 0 -30`.
- Run B: the portals are at 4 151 2 and 6 151 2, two tall. They are touched from 1.5-2.5 blocks
  south of them at `look 0 -20`.

If a press misses, fix the aim in the script and tell the portals worker in one line.
