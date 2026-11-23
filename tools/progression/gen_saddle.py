#!/usr/bin/env python3
"""Writes mods/testkit/scripts/saddle.txt: saddles lift tames by their fitting
(vitals::gear::saddle_lift, Planner 11-23 option A). One tame utahraptor (level 0, grown) fights
zombies in a roofed room beside its owner, first on a plain saddle, then on a bronze-fitted one
swapped on (the same animal).

Read run.log:
- `saddle: <id> saddled, fitting 0, lift ×1.26` then `saddle: <id> swapped fitting 0 for 2, lift now ×2.12`
  (a level-0 utahraptor fights at area level 0; plain is the Stone reference 5, bronze 16).
- `tame_fight: utahraptor <id> (saddle 1, ...)` lines on the plain saddle, `(saddle 3, ...)` on
  bronze: its hits about ×1.68 the plain ones, its max health in the `entities` lines too.
"""
from pathlib import Path

OUT = Path(__file__).resolve().parents[2] / "mods/testkit/scripts/saddle.txt"
AT = "0.5 197 -2.5"
lines: list[str] = []
o = lines.append

o("# Saddles lift tames (vitals::gear::saddle_lift), written by tools/progression/gen_saddle.py.")
o("# Read run.log: `saddle:` lines (the lift), `tame_fight:` lines (its hits), `entities` (its max health).")
o("loaded")
o("idle")
o("chat /rule core:mob_spawning 0")
o("chat /time set 6000")
o("chat /tk attr_mod me core:max_health testkit:hp core:add 200 0")
o("chat /tk mode spectator")
o("chat /tp 0.5 205 -0.5")
o("wait 2")
o("loaded")
# A closed, roofed room 9 x 9 (zombies don't burn), lit by the day through nothing: dark is fine.
o("chat /tk fill -6 195 -6 6 201 6 stone")
o("chat /tk fill -4 197 -4 4 199 4 core:air")
o(f"chat /tp {AT}")
o("wait 1")
o("loaded")
o("chat /tk mode survival")
o(f"chat /tp {AT}")
o("chat /tk heal")
# The tame: grown, level 0, the player's.
o("chat /tk spawn animals:utahraptor 0 197 0")
o("wait 1")
o("look 0 -10")
o("aim 6")
o("chat /animal level 0")
o("chat /animal age 30")
o("chat /animal tame")
o("wait 1")


def saddle(fitting: str, name: str) -> None:
    o(f"# ---- {name}: a {fitting} saddle on it")
    o(f"chat /gear saddle utahraptor {fitting} slot=0")
    o(f"chat /tp {AT}")
    o("aim 6")
    o("press core:use")
    # Its stats go on in its slow upkeep.
    o("wait 3")
    o(f"where {name}_saddled")
    o("entities 10")
    o(f"shot saddle_{name}_start.png")


def round_(name: str) -> None:
    o(f"# ---- {name}: zombies hit its owner; it defends them")
    for x in (-3, 0, 3):
        o(f"chat /tk spawn base:zombie {x} 197 3")
    for i in range(6):
        o("wait 5")
        o("chat /tk heal")
        o(f"chat /tp {AT}")
        if i % 2 == 1:
            o("entities 10")
    o(f"shot saddle_{name}_end.png")
    o("chat /kill @e[type=base:zombie]")
    o("wait 1")


saddle("leather", "plain")
round_("plain")
saddle("bronze", "bronze")
round_("bronze")
o("quit")

OUT.write_text("\n".join(lines) + "\n")
print(f"wrote {OUT} ({len(lines)} lines)")
