#!/usr/bin/env python3
"""The dead bodies (notes/crew/plans/carcass-and-skinning.md, slice 3): one lying model per body plan, whole and opened,
built as props with the blocky-model skill's tools and the house style, centred on the carcass's place (r5/r6 rolled
the living model onto its side about its root, which left it beside its own place and, on a deer, legs up).

    ~/.venvs/blocky/bin/python mods/animals/source/carcass/body.py

Plans and their reference animal (the model's size; `animals_core::carcass::Plan` scales each species to it by the
cube root of its mass):
- quadruped (a 70 kg white-tailed deer: 1.6 m nose to tail, chest 0.45 m deep, 0.35 m wide): on its right side, legs
  out straight along the ground, head and neck laid forward;
- bird (a 5 kg vulture or eagle: 0.75 m bill to tail): on its side, one wing half open over it;
- biped (a 30 kg raptor: 2 m with its tail): on its side, the long tail out straight, both legs out;
- reptile (a 40 kg caiman: 2 m): on its belly, legs splayed, the tail behind;
- fish (a 3 kg trout: 0.55 m): flat on its side.
Opened: the upper flank (the back for the reptile) cut open, the meat showing with ribs across it, the guts out on the
ground beside it; the bird's top wing gone with a few feathers about.

All ten share one texture (`textures/entity/carcass/body.png`), so the game's coat painter can repaint whichever is
drawn (`mods/animals/src/carcass.rs`): `body_mask.bin` gives each texel's material (0 coat, 1 underside, 2 dark: hooves,
claws, bill; 3 meat; 4 bone; 255 none), and the painter turns coat and underside to the animal's own colours, keeping
the light painted here.
"""

import json
import os
import subprocess
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", ".."))
SKILL = os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts")
sys.path.insert(0, SKILL)
import to_bedrock  # noqa: E402
from boxcheck import check  # noqa: E402
from house import SCALE, TX, material  # noqa: E402
from paint import KS  # noqa: E402
from prop import FACES, el, paint  # noqa: E402

ASSETS = os.path.join(HERE, "..", "..", "assets")
OUT = os.path.join(HERE, "out")

COAT, UNDER, DARK, MEAT, BONE, GUT, LIVER = range(7)
# The default coat is a deer's (the painter recolours from it by channel ratios: keep in step with `carcass.rs`).
COAT_RGB, UNDER_RGB = (124, 80, 50), (212, 194, 166)
MATERIALS = [material("fur", COAT_RGB), material("fur", UNDER_RGB), material("claw"), material("skin", (150, 46, 40)),
             material("horn"), material("skin", (196, 128, 118)), material("skin", (98, 32, 36))]


def b(name, lo, hi, mat=COAT, under=None):
    """A box of `mat`; `under`: (axis, from, sign): its texels past `from` along the axis (toward `sign`) are the
    underside's colour."""
    return el(name, lo, hi, mat=mat, under=under)


def guts(x, z, s=1.0):
    """Guts out on the ground beside an opened body, at (x, z) and `s` times a deer's: a pale coil of intestine, the
    red stomach on it, the dark liver at its foot (Planner on batch 1: one flat cube read as a brick)."""
    def at(lo, hi, mat, name):
        return b(name, (x + lo[0] * s, lo[1] * s, z + lo[2] * s), (x + hi[0] * s, hi[1] * s, z + hi[2] * s), mat)
    return [at((0, 0, 0), (12, 6, 10), GUT, "gut_coil"), at((3, 1, 8.5), (11, 8, 15), MEAT, "gut_stomach"),
            at((7, 1, -5), (14, 3.5, 1.5), LIVER, "gut_liver")]


def leg(name, x, y, z, a, bend, l1=17, l2=15):
    """A leg lying out along +x from (x, y, z) (its top at y + 6), splayed `a` degrees toward the head (about y) and
    bent `bend` more at the knee: the upper part, the lower 1 thinner, the hoof."""
    import math
    k = (x + l1 * math.cos(math.radians(a)), z - l1 * math.sin(math.radians(a)))
    rot1, rot2 = ((0, a, 0), (0, a + bend, 0)) if a or bend else (None, None)
    o1, o2 = ((x, y + 3, z) if rot1 else None), ((k[0], y + 3, k[1]) if rot2 else None)
    return [el(f"{name}_upper", (x, y, z - 3), (x + l1 + 1, y + 6, z + 3), rot=rot1, origin=o1, mat=COAT, under=None),
            el(f"{name}_lower", (k[0] - 1, y + 0.5, k[1] - 2.5), (k[0] + l2, y + 5.5, k[1] + 2.5), rot=rot2, origin=o2,
               mat=COAT, under=None),
            el(f"{name}_hoof", (k[0] + l2 - 0.5, y + 1, k[1] - 2), (k[0] + l2 + 5, y + 5, k[1] + 2), rot=rot2, origin=o2,
               mat=DARK, under=None)]


def ribs(z0, z1, x0, x1, y, step=6, w=2):
    """Bone bars across an opened flank, from z0 to z1, `y` their bottom."""
    return [b(f"rib{i}", (x0, y, z), (x1, y + 2, z + w), BONE) for i, z in enumerate(range(z0, z1, step))]


# ---- quadruped: a deer on its right side; +x is the belly (legs out that way), -x its back; +z the tail.
BELLY = (0, 6, 1)
quad = [
    b("body", (-15, 0.5, -35), (14, 22, 35), under=BELLY),
    b("chest", (-13, 1.5, -40), (12, 20, -30), under=BELLY),
    b("rump", (-14, 1, 28), (11, 21, 40), under=BELLY),
    b("neck", (-10, 3, -56), (2, 17, -38), under=(0, -2, 1)),
    b("head", (-11, 3.5, -74), (4, 15.5, -55), under=(0, 1, 1)),
    b("muzzle", (-7, 5, -84), (2, 13, -73)),
    b("nose", (-6, 6, -86), (0, 12, -83.5), DARK),
    b("ear", (-17, 8, -60), (-10, 13, -51)),
    b("tail", (-8, 8, 39), (-1, 15, 48), under=(0, -4, 1)),
]
# The near (upper) pair bent at the knee, all four splayed a little (Planner on batch 1: four parallel sticks read as
# a deer tipped over, not lying dead): the forelegs toward the head, folded back at the knee; the hind toward the tail,
# the hock forward.
for name, z, y, a, bend in [("fore_top", -22, 13, 14, -32), ("fore_low", -17, 2.5, 5, -6),
                            ("hind_top", 24, 13.5, -12, 28), ("hind_low", 29, 3, -5, 8)]:
    quad += leg(name, 12, y, z, a, bend)
quad_open = quad + [
    b("cut", (-6, 21, -20), (10, 23, 16), MEAT),
    *ribs(-17, 14, -7, 11, 22.5),
    *guts(17, -9),
]

# ---- bird: on its side, bill to -z, legs (+x) tucked; the top wing half open over the body.
bird_body = [
    b("body", (-8, 0.5, -14), (8, 14, 15), under=(0, 3, 1)),
    b("neck", (-5, 2, -21), (3, 11, -13)),
    b("head", (-6, 2.5, -30), (4, 11.5, -20)),
    b("bill", (-3, 5, -37), (2, 9, -29.5), DARK),
    b("tail", (-6, 5, 14), (4, 8, 28)),
    b("leg_a", (7.5, 5, 2), (14, 7, 4), DARK),
    b("leg_b", (7.5, 8, 5), (13, 10, 7), DARK),
]
bird = bird_body + [b("wing", (-7, 14, -7), (11, 15.5, 14)), b("primaries", (-4, 14.5, 13.5), (7, 16, 27))]
bird_open = bird_body + [
    b("cut", (-5, 13, -9), (5, 15, 9), MEAT),
    *ribs(-7, 8, -6, 6, 14.5, step=5, w=1.5),
    b("feather_a", (12, 0, -22), (22, 1, -16)),
    b("feather_b", (-20, 0.5, 4), (-12, 1.5, 9)),
    b("feather_c", (14, 0.25, 18), (22, 1.25, 24)),
]

# ---- biped: a raptor on its side; tail to +z, legs out to +x.
biped = [
    b("body", (-10, 0.5, -16), (10, 16, 16), under=(0, 3, 1)),
    b("neck", (-8, 3, -31), (2, 12, -14)),
    b("head", (-7.5, 2.5, -46), (3.5, 12.5, -30)),
    b("snout", (-5, 4.5, -57), (1, 10, -45.5)),
    b("tail_0", (-7, 3, 15), (3, 12, 40)),
    b("tail_1", (-5, 4.5, 39), (1, 10.5, 64)),
    b("tail_2", (-3.5, 6, 63), (-0.5, 9, 82)),
    b("thigh_top", (8, 8, 2), (25, 18, 12)),
    b("shin_top", (24, 9.5, 3.5), (39, 16.5, 10.5)),
    b("foot_top", (38.5, 9, 3), (45, 17, 11), DARK),
    b("thigh_low", (8.5, 1, 4.5), (23, 8.5, 13.5)),
    b("shin_low", (22.5, 1.5, 6), (35, 7, 12)),
    b("foot_low", (34.5, 1, 5.5), (40, 7.5, 12.5), DARK),
    b("arm", (9, 9.5, -12), (18, 13, -7.5)),
    b("claw", (17.5, 10, -11.5), (21, 12.5, -8), DARK),
]
biped_open = biped + [
    b("cut", (-6, 15, -12), (7, 17, 10), MEAT),
    *ribs(-10, 9, -7, 8, 16.5, step=5),
    *guts(12, -13, 0.7),
]

# ---- reptile: a caiman on its belly; the back up (+y), the tail to +z, the legs splayed out along x.
SIDE = (1, 4, -1)  # its flanks' lower part, under 4 units, the pale belly
reptile = [
    b("body", (-12, 0.5, -20), (12, 12, 20), under=SIDE),
    b("head", (-8, 1, -37), (8, 9, -19), under=SIDE),
    b("snout", (-5.5, 1.5, -55), (5.5, 6.5, -36.5)),
    b("tail_0", (-9, 1, 19), (9, 10, 45), under=SIDE),
    b("tail_1", (-6, 1.5, 44), (6, 7.5, 70)),
    b("tail_2", (-3.5, 2, 69), (3.5, 5.5, 88)),
]
for name, x0, x1, z in [("fl", 11, 24, -16), ("fr", -24, -11, -16), ("hl", 11, 26, 11), ("hr", -26, -11, 11)]:
    foot = (x1 - 0.5, x1 + 4) if x0 > 0 else (x0 - 4, x0 + 0.5)
    reptile += [b(f"leg_{name}", (x0, 1, z), (x1, 5, z + 5)),
                b(f"foot_{name}", (foot[0], 0.5, z - 1), (foot[1], 3, z + 6), DARK)]
reptile_open = reptile + [
    b("cut", (-7, 11, -13), (7, 13, 13), MEAT),
    *ribs(-11, 12, -8, 8, 12.5),
    *guts(14, -6, 0.8),
]

# ---- fish: flat on its side, head to -z, belly to +x.
# (Planner on batch 1: a plain slab.) Deepest behind the head, narrowing to the tail stalk; a forked tail, the dorsal
# fin on the back (-x) in two steps, the anal fin under, a pectoral fin on the upper side, the eye.
fish_body = [
    b("snout", (-4, 1.5, -28), (4, 5, -23.5), under=(0, 1, 1)),
    b("head", (-6, 1, -24), (6, 6, -14), under=(0, 2, 1)),
    b("eye", (-3.5, 6, -21.5), (-1, 6.5, -19), DARK),
    b("body", (-8, 0.5, -15), (8, 6.5, 6), under=(0, 2, 1)),
    b("rear", (-6, 1, 5.5), (6, 6, 14), under=(0, 2, 1)),
    b("stalk", (-3.5, 1.5, 13.5), (3.5, 5.5, 19)),
    b("tail_stem", (-4.5, 2.25, 18.5), (4.5, 4.25, 22)),
    b("tail_top", (-11, 2.75, 21.5), (-2, 3.75, 30)),
    b("tail_low", (2, 2.75, 21.5), (11, 3.75, 30)),
    b("dorsal", (-13, 2, -8), (-7.5, 5, 4)),
    b("dorsal_back", (-10.5, 2.5, 3.5), (-5.5, 4.5, 9)),
    b("anal", (5.5, 2.5, 4), (10, 4, 10)),
]
fish = fish_body + [b("pectoral", (0, 6.5, -11), (5, 7.25, -5))]
# Opened: the flank's meat with the spine along it and fine ribs across, the guts beside.
fish_open = fish_body + [
    b("cut", (-3, 6, -12), (6, 7.5, 4), MEAT),
    b("spine", (-1.5, 7, -13), (0.5, 8.75, 5), BONE),
    *[b(f"rib{i}", (-2.5, 7.5, z), (6.5, 8.25, z + 1), BONE) for i, z in enumerate(range(-11, 4, 3))],
    *guts(9, -6, 0.35),
]

# ---- the bones (eaten or rotted to the bones) and the pile (a long while after: scattered, a skull and a few long
# bones), per plan, in the same axes as its body. Bone, a little meat left on the fresher bones, and a scrap of the
# animal's own hide (coat: the painter turns it to its colour), so a wolf's bones don't read as a deer's.
def arch(name, z, x0, x1, top, w=2.0, foot=1.0):
    """A rib on its side: up from the spine at x0, over at `top`, down at x1 to the ground; its parts each a little
    inset from the next (no two faces in one plane)."""
    return [b(f"{name}_a", (x0, foot, z - 0.25), (x0 + 2, top - 0.5, z + w + 0.25), BONE),
            b(f"{name}_b", (x0 + 0.5, top - 2, z + 0.25), (x1 - 0.5, top, z + w - 0.25), BONE),
            b(f"{name}_c", (x1 - 2, foot + 0.5, z - 0.25), (x1, top - 1, z + w + 0.25), BONE)]


def long_bone(name, x0, x1, z, y=0.5, t=3.0, end=None):
    """A leg's long bone lying out along x at z, `t` thick, with a knob at `end` (dark: a hoof or claw) or a plain
    joint."""
    out = [b(name, (x0, y, z), (x1, y + t, z + t), BONE)]
    if end is not None:
        out.append(b(f"{name}_end", (x1 - 0.5, y - 0.5 if y >= 1 else y + 0.5, z - 0.5), (x1 + 3.5, y + t + 0.5, z + t + 0.5), end))
    return out


def vertebrae(name, x0, x1, y0, y1, z0, z1, n, gap=0.75):
    """A run of `n` vertebrae from z0 to z1 (a tail, a neck), each its own box with a gap between, narrowing toward z1."""
    out, step = [], (z1 - z0) / n
    for i in range(n):
        s = 1 - 0.4 * i / max(n - 1, 1)
        cx, cy = (x0 + x1) / 2, y0 + (y1 - y0) / 2
        hx, hy = (x1 - x0) / 2 * s, (y1 - y0) / 2 * s
        za, zb = sorted((z0 + i * step, z0 + (i + 1) * step - gap * (1 if step > 0 else -1)))
        out.append(b(f"{name}{i}", (cx - hx, max(0.5, cy - hy), za), (cx + hx, cy + hy, zb), BONE))
    return out


quad_bones = [
    b("spine", (-13, 0.5, -36), (-8, 5, 30), BONE),
    *[p for i, z in enumerate(range(-30, 8, 6)) for p in arch(f"rib{i}", z, -12, 9, 19 - abs(z + 12) * 0.2)],
    b("pelvis", (-12.5, 1, 24), (-1, 10, 36), BONE),
    *vertebrae("neck", -12, -6, 1, 6, -37, -54, 4),
    b("skull", (-12, 1.5, -68), (1, 12, -53), BONE),
    b("snout", (-9, 2, -82), (-1.5, 8.5, -67.5), BONE),
    b("jaw", (-0.5, 0.5, -80), (4, 3.5, -60), BONE),
    *vertebrae("tail", -12, -7, 1, 5, 36.5, 52, 3),
    *long_bone("fore_a", 6, 24, -26, end=DARK), *long_bone("fore_b", 4, 20, -19, y=0.75, t=2.5, end=DARK),
    *long_bone("hind_a", 6, 30, 22, end=DARK), *long_bone("hind_b", 5, 26, 30, y=0.75, t=2.5, end=DARK),
    b("meat_a", (-7, 1.25, -20), (4, 6, -4), MEAT),
    b("meat_b", (-9, 5.25, 4), (-3, 8, 14), MEAT),
    b("hide", (-30, 0.25, -12), (-15, 1.25, 14), under=(0, -20, 1)),
]
quad_pile = [
    b("skull", (-6, 0.5, -6), (7, 10.5, 9), BONE),
    b("snout", (-3, 1, -19), (4.5, 7.5, -5.5), BONE),
    b("bone_a", (-26, 0.25, 12), (2, 3.25, 15), BONE),
    b("bone_b", (10, 0.5, -22), (13, 3, 6), BONE),
    b("bone_c", (-22, 0.75, -16), (-6, 3.25, -13.5), BONE),
    b("pelvis", (14, 0.5, 12), (24, 6, 22), BONE),
    *[b(f"rib{i}", (-20 + i * 3.5, 0.25 + (i % 2) * 0.25, 20 + i), (-18.5 + i * 3.5, 1.75 + (i % 2) * 0.25, 32 + i), BONE)
      for i in range(4)],
    b("hoof", (12.5, 0.25, -26), (15.5, 3.25, -22.5), DARK),
    b("hide", (-30, 0.5, -10), (-18, 1.5, 6)),
]
bird_bones = [
    b("keel", (-1.5, 0.5, -7.5), (2.5, 6.5, 6.5), BONE),
    b("spine", (-8, 0.5, -12), (-4, 4, 14), BONE),
    *[p for i, z in enumerate(range(-8, 8, 4)) for p in arch(f"rib{i}", z, -7, 3, 10, w=1.5, foot=1)],
    *vertebrae("neck", -6, -2, 1, 4, -12.5, -22, 3, gap=0.5),
    b("skull", (-5.5, 1, -28), (1.5, 7.5, -21.5), BONE),
    b("bill", (-3, 2.5, -34), (0.5, 5.5, -27.5), DARK),
    b("wing_a", (-14, 1.5, -5), (-3, 3, -3), BONE), b("wing_b", (-24, 0.5, -4.5), (-13.5, 1.75, -3.5), BONE),
    b("leg_a", (4.5, 0.5, 4), (13, 2.5, 6), BONE), b("leg_b", (12.5, 1, 4.5), (15.5, 2, 9), DARK),
    b("feather_a", (8, 0.25, -22), (18, 1.25, -16)), b("feather_b", (-20, 0.5, 6), (-12, 1.5, 11)),
    b("feather_c", (10, 0.25, 16), (18, 1.25, 22)),
]
bird_pile = [
    b("skull", (-3, 0.5, -3), (3, 5.5, 3.5), BONE), b("bill", (-1, 1.5, -9), (2, 4, -2.5), DARK),
    b("keel", (6, 0.25, 0), (10, 3.5, 9), BONE), b("wing", (-14, 0.5, 6), (-4, 1.75, 7.5), BONE),
    b("feather_a", (-12, 0.25, -14), (-4, 1.25, -8)), b("feather_b", (2, 0.25, 12), (10, 1.25, 18)),
    b("feather_c", (12, 0.5, -12), (19, 1.5, -6)),
]
biped_bones = [
    b("spine", (-9, 0.5, -16), (-4, 5, 16), BONE),
    *[p for i, z in enumerate(range(-12, 8, 5)) for p in arch(f"rib{i}", z, -8, 7, 15 - abs(z + 2) * 0.25)],
    b("pelvis", (-8.5, 1.5, 7), (4, 13, 16.5), BONE),
    *vertebrae("neck", -8, -3, 1, 6, -16.5, -30, 4, gap=0.5),
    b("skull", (-7.5, 1.5, -45), (2.5, 11, -29.5), BONE),
    b("snout", (-5.5, 2, -57), (0.5, 8.5, -44.5), BONE),
    b("teeth", (0.25, 2.5, -56), (1.25, 5.5, -40), BONE),
    *vertebrae("tail", -8, -2, 1, 7, 16.5, 80, 9, gap=1),
    *long_bone("thigh_a", 3, 22, 4, t=4), *long_bone("shin_a", 21.5, 36, 4.5, y=1, t=3, end=DARK),
    *long_bone("thigh_b", 3.5, 20, 10.5, y=0.75, t=3.5), *long_bone("shin_b", 19.5, 32, 11, y=1.25, t=2.5, end=DARK),
    b("arm", (5, 0.5, -15), (14, 2.5, -12.5), BONE), b("claw", (13.5, 1, -15.5), (17, 3, -12), DARK),
    b("meat", (-6, 1.25, -8), (3, 6, 4), MEAT),
    b("hide", (-20, 0.25, -6), (-10, 1.25, 12)),
]
biped_pile = [
    b("skull", (-5, 0.5, -10), (4, 9.5, 5), BONE), b("snout", (-3, 1, -22), (2.5, 6.5, -9.5), BONE),
    b("thigh", (6, 0.5, -6), (26, 4.5, -2), BONE), b("shin", (-24, 0.25, 8), (-6, 3.25, 11), BONE),
    *vertebrae("tail", -2, 4, 0.5, 4.5, 10, 40, 5, gap=1.5),
    b("claw", (26.5, 0.5, -7), (30, 3, -3.5), DARK),
    b("hide", (10, 0.5, 8), (22, 1.5, 22)),
]
reptile_bones = [
    b("spine", (-2.5, 0.5, -20), (2.5, 7, 20), BONE),
    *[b(f"rib{i}_{s}", ((-12, 1, z) if s == "l" else (2, 1, z)), ((-2, 5, z + 2) if s == "l" else (12, 5, z + 2)), BONE)
      for i, z in enumerate(range(-16, 14, 5)) for s in "lr"],
    b("pelvis", (-7, 1, 13), (7, 6, 19.5), BONE),
    b("skull", (-7, 0.5, -37), (7, 7, -20.5), BONE),
    b("snout", (-5, 1, -55), (5, 4.5, -36.5), BONE),
    *vertebrae("tail", -4, 4, 0.5, 6, 20.5, 86, 10, gap=1),
    *[p for name, x0, x1, z in [("fl", 6, 22, -15), ("fr", -22, -6, -15), ("hl", 6, 24, 11), ("hr", -24, -6, 11)]
      for p in [b(f"leg_{name}", (x0, 0.5, z), (x1, 3, z + 2.5), BONE),
                b(f"foot_{name}", ((x1 - 0.5, 1, z - 1) if x0 > 0 else (x0 - 3.5, 1, z - 1)),
                  ((x1 + 3.5, 3.5, z + 3.5) if x0 > 0 else (x0 + 0.5, 3.5, z + 3.5)), DARK)]],
    b("meat", (-1.75, 1.25, -12), (1.75, 9, 2), MEAT),
    b("hide", (-16, 0.25, 22), (-5, 1.25, 40), under=(1, 0.75, -1)),
]
reptile_pile = [
    b("skull", (-6, 0.5, -10), (6, 6, 6), BONE), b("snout", (-4, 1, -26), (4, 4, -9.5), BONE),
    *vertebrae("tail", -3, 3, 0.5, 4.5, 12, 48, 6, gap=1.5),
    b("leg", (10, 0.25, -20), (26, 2.5, -17.5), BONE), b("foot", (25.5, 0.75, -21), (29, 3, -16.5), DARK),
    b("hide", (-22, 0.5, 4), (-8, 1.5, 22), under=(1, 1, -1)),
]
fish_bones = [
    b("skull", (-6, 0.5, -24), (6, 5, -14), BONE),
    b("eye", (-3.5, 5, -21.5), (-1, 5.5, -19), DARK),
    b("jaw", (-3, 1, -28.5), (3.5, 3.5, -23.5), BONE),
    b("spine", (-1, 1, -14.5), (1.5, 2.75, 19), BONE),
    *[b(f"rib{i}", (-7.5 + abs(z) * 0.1, 1.5, z), (7.5 - abs(z) * 0.1, 2.25, z + 1), BONE) for i, z in enumerate(range(-12, 14, 3))],
    b("tail_top", (-9, 0.5, 19.5), (-1.5, 1.5, 27)), b("tail_low", (2, 0.75, 19.5), (9.5, 1.75, 27)),
]
fish_pile = [
    b("skull", (-4, 0.5, -5), (4, 4, 2), BONE), b("jaw", (-2, 1, -9), (2.5, 3, -4.5), BONE),
    b("spine", (4, 0.25, 4), (6, 1.75, 22), BONE),
    *[b(f"rib{i}", (0.5, 0.75, z), (10, 1.25, z + 1), BONE) for i, z in enumerate(range(7, 20, 4))],
]

MODELS = [("carcass_quadruped", quad), ("carcass_quadruped_open", quad_open), ("carcass_bird", bird),
          ("carcass_bird_open", bird_open), ("carcass_biped", biped), ("carcass_biped_open", biped_open),
          ("carcass_reptile", reptile), ("carcass_reptile_open", reptile_open), ("carcass_fish", fish),
          ("carcass_fish_open", fish_open),
          ("carcass_quadruped_bones", quad_bones), ("carcass_quadruped_pile", quad_pile),
          ("carcass_bird_bones", bird_bones), ("carcass_bird_pile", bird_pile),
          ("carcass_biped_bones", biped_bones), ("carcass_biped_pile", biped_pile),
          ("carcass_reptile_bones", reptile_bones), ("carcass_reptile_pile", reptile_pile),
          ("carcass_fish_bones", fish_bones), ("carcass_fish_pile", fish_pile)]
APART = 400  # units each model is painted apart from the next (so none shades another)


POINTS = {}  # (element name, face): its texels' positions in the model's own units (`area` keeps them for `layers`)


def area(e, face, p, uu, vv, h):
    a = np.full(p.shape[:2], e["mat"], int)
    q = p - np.array([e["model"] * APART, 0, 0])  # painted apart along x (`APART`)
    POINTS[(e["name"], face)] = q
    if e["under"]:
        ax, at, sgn = e["under"]
        a[(q[..., ax] - at) * sgn > 0] = UNDER
    return a


# ---- coat layers: the living painter's (`animals_core::coats`, the skill's `layers.py`) for these bodies, so a dead
# animal shows its white spotting, points, mask and patterns, not only its two colours. Each texel's place is turned
# back to where it would be on the animal standing (y up, z head to tail): a body on its side has its back at -x and
# its belly, legs out, at +x; the reptile lies on its belly as it stands.
STAND = {"quadruped": (14, 30), "bird": (8, 8), "biped": (10, 26), "fish": (6, 0)}  # belly's x, the legs' length


def standing(name, q):
    plan = name.split("_")[1]
    if plan == "reptile":
        return q
    belly, legs = STAND[plan]
    return np.stack([q[..., 1] - 8, belly + legs - q[..., 0], q[..., 2]], -1)


def props_of(part, under):
    """White, points, mask, extremity and pattern zone (0..1, `layers.py`) for a part by its name, its underside
    texels whiter and less patterned."""
    has = lambda *ks: any(k in part for k in ks)  # noqa: E731
    if has("lower", "_low", "shin"):
        v = (0.6, 0.9, 0, 0.9, 0.5)
    elif has("upper", "_top", "thigh", "leg", "arm", "foot"):
        v = (0.35, 0.7, 0, 0.6, 0.6)
    elif has("muzzle", "snout", "nose"):
        v = (0.5, 0.7, 0.9, 0.7, 0.2)
    elif has("head"):
        v = (0.3, 0.4, 0.4, 0.4, 0.5)
    elif has("ear"):
        v = (0.2, 0.5, 0, 0.95, 0.4)
    elif has("tail"):
        v = (0.2, 0.2, 0, 0.9 if has("_2", "top", "low") else 0.7, 0.8)
    elif has("chest", "neck"):
        v = (0.45, 0.3, 0, 0.1, 0.8)
    elif has("feather", "wing", "primaries", "dorsal", "anal", "pectoral"):
        v = (0.15, 0.1, 0, 0.5, 0.7)
    else:
        v = (0.08, 0, 0, 0, 1)
    v = np.array(v, float)
    return np.where(under[..., None], np.maximum(v * [1, 1, 1, 1, 0.3], [0.75, 0, 0, 0, 0]) + v * [0, 1, 1, 1, 0], v)


def layers(elements, uv, mask, w, h):
    """The four layer images and their bounds, in the texture's layout (gutters copied from each tile's edge)."""
    info, pos, props, props2 = (np.zeros((h, w, 4)) for _ in range(4))
    tiles = []
    for (i, face), (u0, v0, u1, v1) in uv.items():
        e = elements[i]
        q = POINTS.get((e["name"], face))
        if q is None or q.shape[:2] != (v1 - v0, u1 - u0):
            continue
        tiles.append((e, standing(MODELS[e["model"]][0], q), (u0, v0, u1, v1)))
    lo = np.min([s.reshape(-1, 3).min(0) for _, s, _ in tiles], 0) - 1
    hi = np.max([s.reshape(-1, 3).max(0) for _, s, _ in tiles], 0) + 1
    for e, s, (u0, v0, u1, v1) in tiles:
        sl = (slice(v0 - 1, v1 + 1), slice(u0 - 1, u1 + 1))
        m = np.pad(mask[v0:v1, u0:u1], 1, mode="edge")
        coat = (m == COAT) | (m == UNDER)
        pr = props_of(e["name"].split("_", 1)[1], m == UNDER)
        pad = lambda a: np.pad(a, ((1, 1), (1, 1), (0, 0)), mode="edge")  # noqa: E731
        info[sl] = np.dstack([coat * 255, np.full(m.shape, 120), (m == UNDER) * 255, np.full(m.shape, 255)])
        pos[sl] = np.dstack([pad((s - lo) / (hi - lo) * 255), np.full(m.shape, 255)])
        props[sl] = np.dstack([pr[..., :3] * 255, np.full(m.shape, 255)])
        props2[sl] = np.dstack([pr[..., 3] * 255, pr[..., 4] * 255, np.zeros(m.shape), np.full(m.shape, 255)])
    return [np.clip(np.round(a), 0, 255).astype(np.uint8) for a in (info, pos, props, props2)], [lo.tolist(), hi.tolist()]


def main():
    elements = []
    for i, (_, els) in enumerate(MODELS):
        hits = check(els, gap=0.3)
        assert not hits, f"{MODELS[i][0]}: faces in one plane {hits}"
        for e in els:
            moved = dict(e, lo=[e["lo"][0] + i * APART, *e["lo"][1:]], hi=[e["hi"][0] + i * APART, *e["hi"][1:]],
                         name=f"{i}_{e['name']}", model=i,
                         origin=[e["origin"][0] + i * APART, *e["origin"][1:]] if e.get("origin") else None)
            elements.append(moved)
    # Crisp areas (no coat merging): softened, the belly line moved off the mask the painter reads.
    sheet, uv = paint(elements, area, MATERIALS, coat=(), width=256, square=False, seed=3, tx=TX)
    # The same layout painted in flat ids: the mask the game's painter reads.
    flat = [np.tile(np.array([m, 0, 0], float), (len(KS), 1)) for m in range(len(MATERIALS))]
    ids, uv2 = paint(elements, area, flat, coat=(), width=256, square=False, seed=3, tx=TX)
    assert uv2 == uv and ids.shape == sheet.shape
    mask = np.where(ids[..., 3] > 0, ids[..., 0], 255).astype(np.uint8)
    assert set(np.unique(mask)) <= {*range(len(MATERIALS)), 255}, np.unique(mask)
    h, w = mask.shape
    os.makedirs(OUT, exist_ok=True)
    tex = os.path.join(OUT, "body.png")
    Image.fromarray(sheet).save(tex)
    with open(os.path.join(HERE, "body_mask.bin"), "wb") as f:
        f.write(np.array([w, h], "<u2").tobytes() + mask.tobytes())
    maps, bounds = layers(elements, uv, mask, w, h)
    d = os.path.join(HERE, "..", "..", "genetics", "layers", "carcass")
    os.makedirs(d, exist_ok=True)
    for name, img in zip(("info", "pos", "props", "props2"), maps):
        Image.fromarray(img).save(os.path.join(d, f"{name}.png"))
    with open(os.path.join(d, "layers.json"), "w") as f:
        json.dump({"bounds": bounds}, f)
    for i, (name, els) in enumerate(MODELS):
        mine = [(j, e) for j, e in enumerate(elements) if e["model"] == i]
        spec_els = []
        for j, e in mine:
            faces = {f: uv.get((j, f)) for f in FACES}
            spec_els.append({"name": e["name"], "lo": [e["lo"][0] - i * APART, *e["lo"][1:]],
                             "hi": [e["hi"][0] - i * APART, *e["hi"][1:]], "quad": None, "rot": e.get("rot"),
                             "origin": [e["origin"][0] - i * APART, *e["origin"][1:]] if e.get("origin") else None,
                             "bone": "root", "faces": faces})
        spec = {"name": name, "texture": [w, h], "bones": [{"name": "root", "parent": None, "pivot": [0.0, 0.0, 0.0]}],
                "elements": spec_els, "animations": {}}
        with open(os.path.join(OUT, f"{name}.spec.json"), "w") as f:
            json.dump(spec, f)
        to_bedrock.export(spec, tex, ASSETS, name, SCALE, "carcass/body", pose_files={"": {}})
        # No animation file: a carcass doesn't move.
        anim = os.path.join(ASSETS, "animations", f"{name}.animation.json")
        if os.path.exists(anim):
            os.remove(anim)
    print(f"{len(MODELS)} models, texture {w}x{h}")
    subprocess.run([sys.executable, os.path.join(HERE, "sheet.py"), *sys.argv[1:]], check=False)
    subprocess.run([sys.executable, os.path.join(HERE, "sheet.py"), "--bones", *sys.argv[1:]], check=False)


if __name__ == "__main__":
    main()
