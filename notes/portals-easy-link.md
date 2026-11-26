# Linking portals in one gesture

Ronald (11-22): "i should be able to place the block and the friend should be able to activate it
or something, or friend can place the block and it automatically then enables the server to server
connection, something that makes it easier but still secure".

Today a link takes these steps:
1. Copy the other world's address.
2. Open your own world, then Multiplayer, then Links, and paste it with a note.
3. The other owner accepts it on their Links screen.
4. Each owner right-clicks a portal and picks the link.

The aim is that the link happens at the portal, in the world.

## What is reused, and the one new trust rule

Nothing new is signed or sent between worlds. Every flow below ends in today's link request
(`links::request`), signed by the asking world's key and checked at the asked owner's inbox
(certificate, vouch, block list, tombstones). The link record is signed by both world keys, and
either side can revoke it. What's new:

- **A friend counts as pre-allowed** (option B). Today the inbox accepts without asking when the
  asking owner is on the allow list, or the request carries an invite. It now also accepts when
  the asking owner is your friend: an identity on your Friends list (you added them from a world
  of theirs or from a link). The rule is one switch per game: "Friends may link to my worlds
  without asking", on by default.
  - This keeps rule 2: a stranger is never auto-accepted, because a friend is someone you added
    yourself.
  - The check is the same identity check the allow list makes: the world's certificate names the
    owner, and the owner's identity vouches for the inbox the answer goes to.
- **A portal waits for its link.** A link's id is the BLAKE3 hash of its body, so the id is known
  when the request is sent. The portal mod keeps a wait for each unbound portal someone asked
  about: the portal's position, the link id, the asking world's key, who asked, and when. It is
  saved in the world's `mods/portals/waits.txt`, because portal positions stay in each world.
  - About once a second the mod looks for the link. When it appears, the portal binds to it, and
    so does every portal block touching it, the way an owner's bind works, with the arrival spot
    beside it.
  - Accepting is still done by the inbox and its rules. The wait only says which portal the
    result belongs to.

## A portal in each world (Ronald, 11-22)

"the friend must also place a block in there world too in order to setup the lunk to where one
joins". A link opens for travel only once both worlds have a portal bound to it. Each world's
portal is where arrivals from the other world land.

- **Pairing.** Both portals are paired by the link they share. That link is the record both world
  keys signed, and each owner agreed to it.
- **Telling the other world.** When a world binds a portal to the link, or loses it (it's unbound,
  bound elsewhere, or broken), its game sends the other owner's inbox a short note its world key
  signs:

  ```text
  chiacraft link portal v1
  link <id>
  by <world key>
  ready <0|1>
  created <secs>
  ```

  The note goes through the same outbox as requests, so it waits while the other game is closed.
  Only the newest note per world counts.
- **Until both exist**, the portal is only its pad. Its screen and chat say "Waiting for a portal
  in 'B's world'". Walking onto it sends nobody, so no arrival ever falls back to a spawn.
- **Finishing it.**
  - In flows 1 and 2, B's game tells B: "Place a portal in 'B's world' and pick this link on it".
  - When B next right-clicks a portal in their own world, that link comes first, marked "new".
  - An owner entering their world hears, once per run, which links still need a portal there.
  - Flow 3 finishes the same way: Bea places her portal at home.

## The look (Ronald, 11-22)

"looks like an portal that opens 2 blocks heght, so the poratl block is place flat in ground, and
then when link is conencted portal shows, and also shows if the friedns server is open or not".

- The portal block is a pad, set flat on the ground. It faces the player who places it.
- The pad shows alone while the portal is unbound, or bound but waiting for the other world's
  portal.
- With both portals, a doorway 2 blocks tall stands over it. It's part of the pad's model, not
  solid: you walk through it onto the pad. It has two states:
  - **open:** lit and moving, while the other world answers its status (`chiacraft/status/1`),
    asked by the host's game about every 8 seconds while players are within 12 blocks;
  - **closed:** dim and still while it doesn't answer. The screen and chat say "'Bea's farm' is
    offline", and walking in sends nobody.
- It's our own design (`notes/STYLE.md`), drawn with the `blocky-model` and `game-art` skills: a
  carved stone pad with a glowing inlay, posts and a lintel, and a violet sheet.
- It's a block state (`state`: base, closed, open) the server sets on every block of the portal,
  so all players see the same doorway.

## The flows

### 1. A places a portal; B, visiting A's world, activates it

1. B right-clicks A's unbound portal. B sees "Link one of your worlds here" and a list of B's own
   worlds. B's game reads that list itself; the server never sees it.
2. B picks one. B's own game signs a link request from that world to the world B is in now (the
   key B's game joined, never a key the server names) and sends it to A's inbox. It hands the
   portal the link id.
3. A's inbox decides:
   - B is A's friend, or B is on A's allow list: linked at once. The portal binds, and both
     players see "The portal now leads to 'B's world'".
   - Otherwise A is asked. A sees a chat line ("Bea asks to lead a portal to 'Bea's farm':
     right-click it to answer"), and the request is also on A's Links screen. A right-clicks the
     portal and sees Accept / Always accept this owner / Decline. Accepting binds it.

Each step proves something:
- B's world key signs the request.
- A's world certificate is checked by the inbox, and B's identity vouches for B's inbox.
- A's agreement is either an earlier decision (adding B as a friend, or allowing B) or the
  Accept click.
- B agreed by picking B's own world.

### 2. B places a portal while visiting A's world

This is the same as flow 1. A portal block placed by a visitor starts unbound, like any portal.
B right-clicks it and picks a world. A's rules decide, as above. Placing a block grants nothing.
If A's world doesn't let visitors build (admission or a protection mod), B can't place one at all.

### 3. Two friends who have never visited each other

1. A places a portal in A's world and right-clicks it as its owner. Beside A's links, the screen
   lists A's friends' worlds: what each friend's game answers on `chiacraft/worlds/1` today, the
   same list as the Friends screen.
2. A picks "Bea's farm". A's game signs a request from A's world to that world's key and sends it
   to Bea's inbox. The portal waits for the link id.
3. Bea's inbox accepts at once if A is Bea's friend and Bea leaves "Friends may link" on.
   Otherwise Bea is asked on her Links screen. Either way A's portal binds when the answer comes.
4. Bea's side of the link exists too. The next time Bea right-clicks a portal in her world, it is
   in her list of links. Until she binds a portal to it, arrivals land at her world's spawn, as
   they do today. Listing first the links no portal leads to, marked "new", is left for later.

Both owners agree, each through their own friend list (or a click). Neither game signs for the
other.

## What an owner sees and can undo

- **A chat line** in the world for every link made at a portal. Made: "A portal here now leads
  to 'Bea's farm'. End the link on the Links screen if you didn't mean it." Asked: "'Bea's farm'
  asks to lead a portal there. Right-click the portal to answer."
- **The portal screen.** As owner you see where the portal leads, the waiting request with
  Accept / Always / Decline, the button to make it lead nowhere, and the "Friends may link"
  switch.
- **The Links screen** lists every link, as today. Ending a link there is final (a tombstone), and
  every portal on it then leads nowhere ("This portal's link has ended").
- **Breaking the portal** drops its wait. Removing a friend stops their automatic links from then
  on; links already made stay until ended.
- `/portalfriends on|off` does the same as the switch.

## Abuse cases

- **Griefers placing portals.** An unbound portal does nothing, and a visitor can only ask.
  - Limits: at most 3 waiting asks per visitor and 16 per world; one chat line to the owner per
    visitor a minute. A request from a blocked owner or world is dropped without a word, as today.
  - A stranger's ask is never accepted without the owner. The owner breaks the block, or declines
    and blocks the asker.
- **Stolen invite tokens.** These flows make no bearer tokens. A friend's automatic link needs a
  request signed by a world whose certificate their identity signed, so their 24 words, not
  anything that can be copied out of a chat. Invites (`i=`) work as before: one use, within a
  week, signed by the world.
- **A visitor binding the owner's portal to their own world.** A visitor can never bind a portal.
  They can only start a request that the owner's inbox accepts by the owner's own rules. Guards:
  - A bound portal can't be asked about; only the owner rebinds it.
  - A wait binds only to a link that didn't exist when the wait was made, whose other world is
    the asker's world. So a visitor can't point an unbound portal at one of the owner's existing
    links.
- **A server tricking a visitor's game into signing.** The visitor's game signs only after the
  visitor clicks, and only toward the world it joined (checked against its own record of the
  join), never toward a key the server sends.
- **A friend's account is stolen.** Their links appear in chat and on the Links screen. End them
  there and remove the friend, or turn the switch off.
- **Many links from one friend.** Each one shows and can be ended. The per-visitor cap limits
  waiting asks.

## Options

**A. Ask at the portal.** The visitor's touch sends a request, and the owner always answers at the
portal or on the Links screen. Nothing new about trust, but it's two gestures by two people, and
the owner must be online.

**B. Friends link at a touch (recommended).** A plus the friend rule: a friend's touch links at
once. Strangers get A. Flow 3 works for friends who have never met in a world. It needs one
switch per game (on by default) and no new tokens. It's Ronald's "friend can place the block and
it automatically enables" for the people he trusts, and the owner still decides, by having added
them.

**C. The owner arms a portal.** The owner right-clicks "Open to visitors for 10 minutes". The
server signs a one-use invite for that portal, and the next visitor who touches it is linked.
This works for non-friends without a click at the moment of linking. But anyone in the world
during those 10 minutes can take it. It could come later, on top of B.

Recommendation: **B**. A is already inside B, as its fallback for strangers, so choosing A later
just means turning the friend switch off by default.

## Where it lives

- `mods/portals`: the guest screen, the waits, the owner's Accept / Decline, friends' worlds on
  the owner's screen, chat lines, `/portal friends`, and lang (en, ru).
- Our engine-side world code (`crates/engine/src/chiacraft`), which moves out after the Weft
  merge (`notes/crew/move-out-plan.md`, `move-links`):
  - new bridge questions;
  - `links`: the friend rule and its switch, and `request` returning the link id;
  - `friends`: friend by identity, and known friends' worlds.

  No Weft file changes. There's one row in `notes/ENGINE_CHANGES.md`.
- Not covered: a dedicated server whose owner's inbox runs in another process still only sees
  links made in its own process (true today too).
