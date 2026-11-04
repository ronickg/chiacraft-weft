# Overnight decisions (2026-11-04)

Ronald, asleep: "just accept and choose for us, but keep it noted" and "dont do any pushes".
Each session logs here the choices that would have gone to him, with the alternatives, so he can
change any of them in the morning.

## Crafting (claude/crafting-d352ab)

1. **Tool numbers per stack as values on the stack, not callback chains.**
   - Chosen: a stack carries its own durability, mining speed, tool level and attack damage,
     and the engine prefers them over the item's. It's simple, and the client's mining
     prediction still works.
   - Alternative: value chains a mod answers each time (more flexible, a call each frame).
   - Planner agreed.
2. **Grade 10 (Legendary) needs 92 points.**
   - Chosen: work from memory tops out at 90 (40 + 30 + 20), so only hands-on work reaches 10.
   - Alternative: 90, which lets luck lift a from-memory part to Legendary.
3. **Hafts wear a tenth as fast as heads.**
   - Chosen: so a steel head outlasts its haft, and a worn haft is replaced, not the tool.
   - Alternative: a quarter, which made steel axes last no longer than iron ones.
4. **Hafts, grips and shafts are split from a log of their wood; wooden heads are cut from
   planks.**
   - Chosen: real, and it lets the book tell an oak haft from an oak head.
   - Alternative: everything from planks.
5. **Worn-out tools built from parts stay, broken, and work like an empty hand until repaired.**
   - Chosen: an engine tag, `keep_broken`.
   - Alternative: they vanish, as today's tools do.
6. **Repair: one unit of a part's material gives back half its life.**
   - Chosen: chert's trait (half the cost) gives twice as much; copper is hammered back fully
     for free. For now there's no loss of maximum per repair.
   - Alternative: Valheim's free repair, or a loss of maximum per repair.
7. **Diamond's harvest level is 400, one tier above steel's 300.**
   - Chosen: following Ronald's "more ores / items with higher tiers".
   - Alternative: 300, level with steel.
8. **Tools built from parts sit beside today's wood-to-diamond tools.**
   - Chosen: until Content's ladder replaces them (Content's decision 1).
   - Alternative: replace them now.
9. **Most book-made parts come out about Sound (grade 4) for now.**
   - Chosen: a beginner's work from memory (10) + material (20–30) + a table (10). Skills will
     raise it.
   - Alternative: start higher, so early parts feel better.
10. **The book's categories use UI redo's `category` class.**
    - Chosen: the top tabs stay for Inventory and Crafting.
11. **The look hook (`core:look`) is built in full**, other players' held items included.
    - Chosen: their look travels as a new synced field, `core:held_look`, since mods read
      `core:held` as an item id. A map pixel takes its colour from the ramp and its alpha from
      the map.
    - Alternative: local player only, leaving others' held tools plain.
12. **Placeholder part art is committed** (`textures/parts/axe/...`, `textures/ramps/...`).
    - Chosen: cut from today's stone axe icon, so the hook can be tested before Chiacraft's art.
      Chiacraft's files replace them at the same paths.
    - Alternative: test with no art (the look would fall back to the plain icon, testing
      nothing).
13. **Discovery stores what a player has held as a fixed bitset of item-name hashes.**
    - Chosen: two bits an item, 1 KB a player, saved and kept through death. It survives new
      items and a changed load order. A false "held" (a few in a thousand) only shows a recipe
      early.
    - What you hold right now counts at once; the bag is looked at every 2 seconds.
    - Near misses show as "?" with "you've held 2 of its 3 materials". They can't be picked yet.
    - Alternative: an exact list of item names (needs a variable-size component, which the engine
      doesn't have).
14. **Heads stay put and handles move to meet them** (Chiacraft's rule, which made built icons
    match the approved ones pixel for pixel).
    - Chosen: each kind's handle socket is where it meets the iron head, which the handles were
      drawn against. Each material's head anchor shifts the handle by whole pixels (`+dx,dy` in
      `core:look`).
    - Wooden heads are drawn once as "wood" and shared by every wood; wrought iron's drawings are
      named "iron".
    - Alternative: a painted head per wood species, and a socket point stored with each handle
      drawing.
15. **A bow kind built from parts:** a stave (self, or short: quicker, lighter), a grip and a
    string. Its power is its stave wood's spring: yew and Osage orange 1, elm 0.85, oak 0.7, birch
    0.6, spruce 0.5.
    - Yew, elm and Osage orange are provisional rows until their trees grow.
    - Staves are split from a log.
    - Shooting itself waits for Content's tier 1b (the bow); this is the parts and the art.
    - The composite bow (horn and sinew) comes later.
    - Alternative: no bow until tier 1b.
16. **Layer art is named by the shape it draws:** `parts/<kind>/<shape>/<art>.png` painted,
    `parts/<kind>/<shape>.png` maps. Then a felling and a bearded axe head each have their own
    drawing.
    - Alternative: by layer only (`parts/axe/head/...`), which can't tell two head shapes apart.
