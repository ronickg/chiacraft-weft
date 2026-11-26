# Linking at the portal: the two-copy checks

`notes/portals-easy-link.md`, option B.
- A link opens only with a portal bound in each world.
- A portal is a pad on the ground. A doorway 2 blocks tall stands over it once the link is
  whole: lit while the other world is open, dim while it's offline.

Each run uses two copies of the game on one machine (alice and bob), with fresh run folders and
world names. They don't touch shaders, but the shots judge the doorway's look, its light and its
animated texture. Each run takes about four minutes.

Run from the folder holding the release build (the worktree), through the testers' launcher.

## Run A: a stranger asks, the owner answers; then bob's own portal

Bob is not alice's friend.
1. He places a pad in her world and links his world there, and alice accepts at the pad. Her
   pad stays a pad: it waits for bob's portal.
2. Bob goes home and binds his pad, which the screen lists first as "new". Alice's doorway
   appears, dim, because bob's world isn't hosted yet.
3. Bob hosts: the doorway lights up, alice walks in and arrives beside bob's portal.

Before the run:
1. Delete `pe-*.txt` and `pe-mark-*`.
2. Make the run folders `run-pe-alice/config` and `run-pe-bob/config`.
3. Write `name = alice` (and `name = bob`) into each folder's `core.txt`.

Start in this order:
1. Alice: `--run-dir run-pe-alice --world pe-alice-1 --script tools/checks/portals-easy/ask-alice.txt`, in the background.
2. Bob: `--run-dir run-pe-bob --script tools/checks/portals-easy/ask-bob.txt`.

Alice's log:
- `link request from 'pe-bob-home' (owner …) for 'pe-alice-1'`. Not "accepted": bob is a
  stranger.
- After her Accept: `linked 'pe-alice-1' with 'pe-bob-home'` and
  `the portal at [5, 151, 4] now leads to 'pe-bob-home'`.
- After bob binds his portal: `the world '…' has a portal for its link`.
- `a portal leads to a world that doesn't answer`, then once bob hosts
  `a portal leads to 'pe-bob-home' (… playing)`.
- `going to 'pe-bob-home' through a portal`.

Bob's log:
- `'pe-alice-1' accepted the link with 'pe-bob-home'`.
- `… comes through a portal` after he hosts.

Shots, alice:
- `pe-alice-1-asks`: chat says "'pe-bob-home' asks to lead a portal there".
- `pe-alice-2-screen`: the request, with Accept / Always / Decline / Block.
- `pe-alice-3-linked`: only the pad shows.
- `pe-alice-4-waiting`: "Waiting for a portal in 'pe-bob-home'".
- `pe-alice-5-closed`: a dim, still doorway 2 blocks tall over the pad.
- `pe-alice-6-open`: the doorway lit (and moving, if recorded).
- `pe-alice-7-arrived`: alice in pe-bob-home, beside bob's portal.

Shots, bob:
- `pe-bob-1-screen`: "Link 'pe-bob-home' here".
- `pe-bob-2-asked`: "Asked to link …".
- `pe-bob-3-finish`: "Linked here. Now place a portal in 'pe-bob-home' …".
- `pe-bob-4-reminder`: "The link with 'pe-alice-1' needs a portal here".
- `pe-bob-5-new`: "pe-alice-1 (new: no portal here yet)" listed first.
- `pe-bob-6-alice-here`: alice by his open doorway.

Shots are frozen pictures. To judge the doorway's motion, run with `-- --record 1`.

## Run B: friends

Bob has two worlds and hosts one. Alice and bob add each other as friends: each visits the other
once, then taps "Add friend" in the Multiplayer list.
1. **Flow 1.** Bob touches alice's pad and it links at once, with no answer from alice. Bob is
   told to place his portal.
2. **Flow 3.** Alice asks bob's other world, which nobody has visited, from her second pad. Bob's
   game accepts it without asking. That pad stays waiting, because nobody places a portal in
   pe2-bob-other.
3. Bob reopens his home world and binds his pad to the first link ("new"), then hosts again.
   Alice's first doorway opens and she walks through.

Before the run:
1. Delete `pe2-*.txt` and `pe2-mark-*`.
2. Make the run folders `run-pe2-alice/config` and `run-pe2-bob/config`, with names as in run A.

Start in this order:
1. Bob: `--run-dir run-pe2-bob --script tools/checks/portals-easy/friends-bob.txt`, in the background.
2. Alice: `--run-dir run-pe2-alice --script tools/checks/portals-easy/friends-alice.txt`. Give no
   `--world`: alice opens `pe2-alice-1` after visiting bob.

Alice's log:
- `added a friend …, met through 'pe2-bob-home'`.
- `link request from 'pe2-bob-home' for 'pe2-alice-1': accepted (its owner is a friend)`.
- `the portal at [4, 151, 2] now leads to 'pe2-bob-home'`.
- `'pe2-bob-other' accepted the link with 'pe2-alice-1'` and
  `the portal at [6, 151, 2] now leads to 'pe2-bob-other'`.
- `the world '…' has a portal for its link`.
- `going to 'pe2-bob-home' through a portal`.

Bob's log:
- `added a friend …, met through 'pe2-alice-1'`.
- `link request from 'pe2-alice-1' for 'pe2-bob-other': accepted (its owner is a friend)`.

Shots:
- `pe2-bob-1-screen`: both of bob's worlds offered.
- `pe2-bob-2-linked`: "Linked here. Now place a portal in 'pe2-bob-home' …".
- `pe2-alice-3-friends`: "Link with 'pe2-bob-other'".
  - If the list is empty, bob's game hadn't answered yet. Open the screen again: the script
    already does that once.
- `pe2-alice-4-waiting`: "Waiting for a portal in 'pe2-bob-other'".
- `pe2-bob-3-new`: bob's screen lists the new link first.
- `pe2-alice-5-open`: the doorway over the first pad, lit.
- `pe2-alice-6-arrived`: alice in pe2-bob-home.

## Aiming

In testkit a negative pitch faces down. Look yaw 0 faces -z.
- The platforms are stone at y 150, from 0 0 to 8 8. The pads sit at y 151 and are 2 pixels
  tall.
- From 2 blocks south of a pad, `look 0 -37` aims at it. Bob places his pad in alice's world
  from 1.6 blocks with `look 0 -45`. Alice aims at bob's pad from 3 blocks with `look 0 -27`.

If a press misses, fix the aim in the script and tell the portals worker in one line.
