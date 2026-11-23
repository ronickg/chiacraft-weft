#!/usr/bin/env python3
"""Writes mods/testkit/scripts/ranged.txt: bows on the curve (vitals::gear::arrow_hit), in the
floors' pit (floors.txt F0): the player on a pillar behind it shoots down at what's in it.

R0  a Sound yew bow, full draws, at a level-0 utahraptor: `floor:` lines hit about 7.6 (15.2 in
    the head), the 10 s floor still holds, stagger breaks.
R1  a Sound spruce bow (spring 0.5), full draws, at a fresh one: about 3.8.
R2  the yew bow half drawn (0.4 s, power 0.32): about 2.4.
R3  the yew bow, full draws, at a zombie: `mob_fight: player ... hit zombie ... for` about 7.6.
"""
from pathlib import Path

AT = "0.5 206 -1.5"
OUT = Path(__file__).resolve().parents[2] / "mods/testkit/scripts/ranged.txt"
lines: list[str] = []
o = lines.append

o("# Bows on the curve (vitals::gear::arrow_hit), written by tools/progression/gen_ranged.py.")
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


def pen(name: str, bow: str, spawn: str, level: bool, draw: float, shots: int) -> None:
    o(f"# ---- {name}: {bow} bow, {draw} s draws, at {spawn}")
    o("chat /kill @e[type=animals:utahraptor]")
    o("chat /kill @e[type=base:zombie]")
    o("chat /kill @e[type=base:arrow]")
    o("wait 1")
    o(f"chat /gear bow {bow} slot=0")
    # After the bow: given first, the arrows filled slot 0 and the bow replaced them (ranged1).
    o("chat /tk give base:arrow 32")
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


pen("R0_yew_full", "yew", "animals:utahraptor", True, 1.2, 18)
pen("R1_spruce_full", "spruce", "animals:utahraptor", True, 1.2, 6)
pen("R2_yew_half", "yew", "animals:utahraptor", True, 0.4, 6)
pen("R3_yew_zombie", "yew", "base:zombie", False, 1.2, 6)
o("quit")

OUT.write_text("\n".join(lines) + "\n")
print(f"wrote {OUT} ({len(lines)} lines)")
