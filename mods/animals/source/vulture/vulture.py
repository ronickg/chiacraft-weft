#!/usr/bin/env python3
"""The vulture: the turkey vulture (*Cathartes aura*), the warm biomes' eater of the dead
(`notes/ECOLOGY.md`, tier 2: "vulture"), on the birds' build (`../bird/bird.py`, the raven's layout
with knobs), from research alone (README.md). Its colors are rules (`area`).

    ~/.venvs/blocky/bin/python ../bird/export.py vulture

A turkey vulture is about 70 cm long with a 1.7 m span and weighs 1.4 kg: the raven's layout drawn
1.1 times its size, the wings long and fingered (span against length 2.45, the raven's 1.9), a
small bald red head on a thin bare neck with a ruff of black feathers at its base, a short hooked
ivory bill, a long rounded tail, pinkish legs. Its head is only about 4 units across, so it is
painted `house.FINE` (4 texels a unit), as small creatures are.
"""

import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "..", ".claude", "skills", "blocky-model", "scripts")))
from house import FINE  # noqa: E402
from paint import EYES  # noqa: E402

BLACK, BROWN, SILVER, SKIN, BILL, LEG = range(6)
MATERIALS = [
    ("black", (50, 42, 42), 0.72, ("coat", 0, 0)),  # the blackish-brown plumage
    ("brown", (86, 68, 56), 0.02, ("coat", 0, 1.0)),  # the coverts' brown edges
    ("silver", (128, 124, 122), 0.7, ("coat", 0.5, 0)),  # the flight feathers' silvery grey
    ("skin", (176, 66, 58), 0.06, "fixed"),  # the bare red head and neck
    ("bill", (232, 222, 196), 0.07, "fixed"),  # ivory
    ("leg", (196, 150, 136), 0.03, "fixed"),
]

EYES.update({
    "vulture": dict(pattern=[".PP.",
                             "PHPP",
                             "PPPP",
                             ".II."], colors={"P": (44, 30, 26), "H": (232, 228, 218), "I": (92, 52, 40)}, halo="none"),
})

HS = 0.75


def area(e, face, p, uu, vv, h):
    """A turkey vulture: blackish-brown, the wing coverts edged brown, the flight feathers silvery
    grey; the head and the upper neck bare red skin, a ruff of black at the neck's base; the bill
    ivory, the legs pinkish."""
    n = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, BLACK)
    if n in ("bill", "jaw", "bill_hook"):
        a[:] = BILL
        return a
    if n.startswith(("shin_", "toes_")):
        a[:] = LEG
        return a
    if n in ("skull", "lid"):
        a[:] = SKIN
        return a
    if n == "neck":
        return np.where(y > 15.6, SKIN, BLACK)  # bare above the ruff
    if n.startswith(("primaries", "flight", "primariesf")):
        a[:] = SILVER
        a = np.where((np.floor(np.abs(x) * 0.9) % 2 == 0) & (face == "up"), BLACK, a)
        return a
    if n.startswith(("wingf_", "wing_", "hand_")) and face != "down":
        return np.where(np.floor(z * 0.9) % 3 == 0, BROWN, BLACK)  # the coverts' brown edges
    if n.startswith("tail"):
        a[:] = BLACK if face == "up" else SILVER
        return a
    return a


def props(e, face, p, a):
    """White very rarely (a leucistic vulture); the coverts' edges marks."""
    white = np.full(p.shape[:2], 0.15)
    pattern = np.where(np.isin(a, (BLACK, BROWN)), 1.0, 0.0)
    return {"white": white, "pattern": pattern}


CFG = dict(
    name="vulture",
    K=1.1,
    D=0.4,
    TX=FINE,  # the bald head is only about 4 units: painted finer so the eye is designed, as small creatures are
    head=HS,
    neck_w=0.6,
    neck_box=(12.4, 18.5, 6.5, 11.5),  # the bare neck out of the ruff, its end deep in the small head
    bill=dict(w=1.2, h=1.4, len=2.6, rot=14, hook=False, y=0.45),  # too slim for a hook box: curved down instead
    body_w=1.2,
    tail=dict(len=1.05, w=1.1, cut="round"),
    wing=dict(arm=1.3, hand=1.25, chord=1.15, tip="fingers", folded=1.1),
    shin=0.5,
    toes=1.0,
    tilt=-26,
    eyes=dict(kind="side", pattern="vulture", z=11.25 + 2.75 * HS * 0.35, y=18.0 + 0.3 * HS, lash=(120, 40, 36)),
    materials=MATERIALS,
    feather=(BLACK, BROWN, SILVER),
    area=area,
    props=props,
    idle="look",
    walk="walk",
    flap=dict(beat=16, hand=8, length=1.8, glide=0.75),
    blinks=dict(idle=[0.9, 2.3]),
    idle_len=3.0,
)
