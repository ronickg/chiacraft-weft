# Items built from parts

Research for `notes/CRAFTING.md` (the parts system), 2026-11-04. *Unsure* marks what couldn't be
confirmed in a source; the Fandom wikis, the Dwarf Fortress wiki and Tetra's player wiki blocked
or were missing, so those rows lean on the mods' own docs and changelogs.

## Tinkers' Construct 3 (1.18–1.20)
- **Stat types:** each part maps to one stat type.
  - **Head** (the repairable part): durability, mining tier, mining speed, attack.
  - **Handle**: percent changes to durability, speed, damage and attack speed.
  - **Binding**: no numbers, only its material's trait.
  - **Ranged** (limb, grip, string) works the same way.
  - **Armour** uses plating, with a durability factor per slot (helmet ×11, chest ×16, legs ×15,
    boots ×13, shield ×18), plus maille (a trait only).
- **How stats combine:**
  - Head stats are taken as they are; several heads average, weighted by part.
  - Handles sum their bonuses and multiply: +0.25 and +0.1 give ×1.35.
- **Traits:** one per material per tool kind. Small tools get 3 traits and large ones 4.
- **Modifier slots:**
  - Small tools start with 3 upgrade slots and 1 ability slot; large tools with 2 upgrades.
  - Extra upgrade slots come from rare things: a book, a music disc, a mob head, rose gold.
  - The usual cap is about 10 upgrades and 2 abilities. Armour has defense slots.
- **Stations:**
  - The part builder takes patterns, which are used up (a gold cast lasts).
  - Sand casts break, gold casts last.
  - The Tinker Station builds small tools; the Tinker's Anvil builds all of them.
  - Repair takes the head's material. *Unsure:* broken tools stay in the bag; parts are swapped
    at the station.
- **Rendering:**
  - Parts are greyscale sprites using 6 grey levels (0, 63, 102 for the border, 140, 178, 216 for
    the body, 255 for the highlight).
  - Each material has render info: a palette or sprite generator, with fallbacks ("metal" is
    shinier than "rock").
  - A command bakes every part-and-material sprite. The addon Too Many Tinkers does the mapping
    live in the shader instead.
  - Tool models are layered per part.
- **Tinkers' Tool Leveling (1.12):** XP from use; each level gives a free modifier slot, and the
  XP needed doubles every level.

## Construct's Armory
Armour is a core (defense, toughness), plates (durability, weight) and a trim (durability, a second
trait). A chest takes 3 plates. *Unsure:* fan wiki.

## Tetra
- **The workbench** is a crafting table you hit with a hammer.
- **Modules:** tools are two heads, a handle and a binding, with synergies for some pairings.
  Swords are a blade and a hilt, with fuller and guard slots.
- **Integrity** is a budget that limits how much a tool can carry; handles give it, heavy heads
  cost it.
- **Hammer tier** gates upgrades.
- **Schematics** come from ruins or traders.
- **Honing:** use fills a bar; when it's full, you hone the tool at the bench for one stronger
  improvement, and the bar starts again.
- **Durability:** at zero a tool can't be used until it's repaired; it doesn't vanish.
- **Players' view:** closer to vanilla than Tinkers, and less overpowered.

## Silent Gear
- **Parts:**
  - main (head, blade or plate)
  - rod
  - cord for bows
  - optional binding, grip, tip (traits, often a higher harvest level), coating (big stats; it
    replaces the head's look) and lining (armour)
- **Combining:** each stat has an operator: weighted average (most), add, multiply or max. Harvest
  level is the highest part's.
- **Synergy:** mixing materials raises a bonus with diminishing steps; mismatched tiers lower it.
  The wiki gives no formula, and players find it opaque.
- **Grades:** E, D, C, B, A, S, SS, SSS and MAX.
  - The grader rolls on a normal curve, using up a catalyst each roll; a better catalyst raises
    the median.
  - A re-roll keeps only a better grade.
  - *Unsure:* the per-grade numbers.
- **The salvager** breaks gear into parts that keep their grade.

## Other games
- **Vintage Story:** heads are knapped, cast or smithed, then joined to a stick. Hammer durability
  runs copper 500, tin bronze 750, iron 1800, steel 4500.
- **Valheim:** quality 1–4, one step per bench level; repair is free.
- **Diablo and other ARPGs:** 5–8 colour tiers. Diablo 2: grey, white, blue, yellow, green, gold.
  Diablo 4: white, blue, yellow, orange, gold.
- **Dwarf Fortress:** 6 qualities, shown by marks round the name: plain, -well-crafted-,
  +finely-crafted+, \*superior\*, ≡exceptional≡, ☼masterful☼. Values run about 2–12×.
  - Masterworks are announced, and losing one makes its maker unhappy.
  - Artifacts are named one-off works.
  - *Unsure:* named weapons keeping their kills.
- **Mount & Blade II: Bannerlord:** a weapon is blade, guard, grip and pommel, unlocked by smelting
  and smithing. Its stats come from the geometry: length gives reach, mass gives thrust, and the
  balance gives swing speed.

## What players love and hate
- **Loved:**
  - seeing your choices on the item
  - tools that grow with use (Tool Leveling, Tetra's honing)
  - tools that don't vanish at zero
  - the thrill of a high roll (Silent Gear, Dwarf Fortress masterworks)
  - mixing materials for a reason
- **Hated:**
  - too many stations and steps (Tinkers' smeltery)
  - opaque formulas (Silent Gear's synergy)
  - paying to re-roll
  - gates locked behind ruins
  - slot stacks that make late tools absurd
  - bags full of parts, patterns and casts
  - tooltips that need Shift and Ctrl

## Sources
- Tinkers' Construct:
  - stat types: https://slimeknights.github.io/docs/json/stat-types, https://slimeknights.github.io/docs/json/stat-types/1.18
  - FAQ: https://slimeknights.github.io/faq/tinkers-construct-3
  - what's new: https://slimeknights.github.io/docs/whats-new/
  - material models: https://slimeknights.github.io/docs/json/models/material
  - texture generators: https://slimeknights.github.io/docs/guides/texture-generators
  - baking part textures: https://slimeknights.github.io/docs/commands/tinkers/generate-part-textures
  - slot types: https://slimeknights.github.io/docs/json/slot-types
  - Too Many Tinkers: https://www.curseforge.com/minecraft/mc-mods/too-many-tinkers
  - Tool Leveling: https://www.curseforge.com/minecraft/mc-mods/tinkers-tool-leveling
- Construct's Armory: https://www.9minecraft.net/constructs-armory-mod/wiki/, https://www.curseforge.com/minecraft/mc-mods/constructs-armory
- Tetra: https://www.curseforge.com/minecraft/mc-mods/tetra, https://tetra.mickelus.se/faq, https://tetra.mickelus.se/wiki/1.20/tech, https://www.mc-mod.net/getting-started-with-tetra-mod-weapon-customization/, https://modrinth.com/mod/tetravsikt, https://www.modpackindex.com/mod/108327/tetrainherit-improvement
- Silent Gear: https://github.com/SilentChaos512/Silent-Gear/wiki/Parts, https://github.com/SilentChaos512/Silent-Gear/wiki/Grades, https://github.com/SilentChaos512/Silent-Gear/wiki/Synergy, https://github.com/SilentChaos512/Silent-Gear/wiki/Stats
- Vintage Story: https://wiki.vintagestory.at/index.php/Knapping, https://wiki.vintagestory.at/index.php/Hammer
- Diablo: https://us.forums.blizzard.com/en/d4/t/rarity-levels-of-each-diablo-listed/73963, https://segmentnext.com/diablo-4-item-rarity
- Dwarf Fortress: https://dwarffortresswiki.org/Quality
- Bannerlord: https://gamerempire.net/mount-blade-2-bannerlord-how-to-unlock-smithing-parts/
- Players: https://moddex.gg/mod/tinkers-construct, https://reddit.rtrace.io/r/feedthebeast/comments/12w7cy8/
