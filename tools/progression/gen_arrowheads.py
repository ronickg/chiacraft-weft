#!/usr/bin/env python3
"""Writes mods/testkit/scripts/arrowheads.txt: arrows of other heads (vitals::gear::arrow_level),
in ranged.txt's pit: the same Sound yew bow, full draws, first with flint arrows (item level 5,
a plain arrow's), then with bronze ones (16: ×1.6^1.1 = ×1.68).

A0  flint arrows at a level-0 utahraptor: `floor:` lines about 7.6 (15.2 in the head).
A1  bronze arrows at a fresh one: about 12.8 (25.6 in the head); under the large cap (+20).
A2  flint arrows at a zombie: `mob_fight: player ... hit zombie ... for` about 7.6.
A3  bronze arrows at a zombie: about 12.8.
"""
from pathlib import Path

AT = "0.5 206 -1.5"
OUT = Path(__file__).resolve().parents[2] / "mods/testkit/scripts/arrowheads.txt"
lines: list[str] = []
o = lines.append

o("# Arrows of other heads (vitals::gear::arrow_level), written by tools/progression/gen_arrowheads.py.")
o("# Read run.log: `floor:` lines (utahraptor) and `mob_fight:` lines (zombie), `stagger:` breaks.")
o("loaded")
o("idle")
o("chat /rule core:mob_spawning 0")
o("chat /time set 6000")
o("chat /tk attr_mod me core:max_health testkit:hp core:add 200 0")
o("chat /tk mode spectator")
o("chat /tp 0.5 205 -0.5")
o("wait 2")
o("loaded")
o("chat /tk fill -6 196 -5 6 199 7 stone")
o("chat /tk fill -1 197 0 1 199 2 core:air")
o("chat /tk fill -6 200 -5 6 208 7 core:air")
# A roof, so a zombie doesn't burn (ranged1's zombie lost its 6 to the sun).
o("chat /tk fill -6 209 -5 6 209 7 stone")
# The shooter on a pillar just behind the pit, 6 up: out of reach (about 9 blocks to the target), so
# holding use draws the bow rather than touching the animal, and steep enough that the line to the
# target clears the pit's rim (ranged2, from 6 back and 3 up, hit the rim's top short of the hole).
o("chat /tk fill 0 200 -2 0 205 -2 stone")
o(f"chat /tp {AT}")
o("wait 1")
o("loaded")
o("chat /tk mode survival")
o(f"chat /tp {AT}")


def pen(name: str, head: str, spawn: str, level: bool, draw: float, shots: int) -> None:
    o(f"# ---- {name}: yew bow, {head} arrows, {draw} s draws, at {spawn}")
    o("chat /kill @e[type=animals:utahraptor]")
    o("chat /kill @e[type=base:zombie]")
    o("chat /kill @e[type=base:arrow]")
    o("wait 1")
    o("chat /gear bow yew slot=0")
    # Into slot 1, replacing the last pen's: a bow shoots the first arrows it finds.
    o(f"chat /gear arrow {head} 32 slot=1")
    o("chat /tk heal")
    o(f"chat /tk spawn {spawn} 0 197 1")
    o("wait 1")
    if level:
        # `/animal` acts on the animal looked at.
        o("aim 12")
        o("chat /animal level 0")
        # Grown, so every pen's raptor is the same size (ranged2's were 103, 75 and 46 health): 30
        # days, past its 12.3 to grow up and far from its 153-day life (ranged3's 2000 killed them).
        o("chat /animal age 30")
    o("wait 0.5")
    o(f"where {name}_start")
    o("entities 10")
    o(f"shot ranged_{name}_start.png")
    for i in range(1, shots + 1):
        o(f"chat /tp {AT}")
        o("aim 12")
        o(f"hold core:use {draw}")
        o("wait 0.3")
        if i % 6 == 0:
            o("chat /tk heal")
            o("entities 10")
    o(f"shot ranged_{name}_end.png")


pen("A0_flint_raptor", "flint", "animals:utahraptor", True, 1.2, 12)
pen("A1_bronze_raptor", "bronze", "animals:utahraptor", True, 1.2, 12)
pen("A2_flint_zombie", "flint", "base:zombie", False, 1.2, 6)
pen("A3_bronze_zombie", "bronze", "base:zombie", False, 1.2, 6)
o("quit")

OUT.write_text("\n".join(lines) + "\n")
print(f"wrote {OUT} ({len(lines)} lines)")
