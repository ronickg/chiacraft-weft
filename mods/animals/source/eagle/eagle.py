#!/usr/bin/env python3
"""The eagle: one model for the golden eagle and the savanna hawk, the day's big hunters of the
open land (`notes/ECOLOGY.md`, tier 2: "eagle"), on the birds' build (`../bird/bird.py`, the
raven's layout with knobs), from research alone (README.md). Its colors are rules (`area`),
painted in a golden eagle's; the savanna hawk of the llanos and the Pantanal is the same model drawn
smaller and rufous by its genes.

    ~/.venvs/blocky/bin/python ../bird/export.py eagle

A golden eagle is about 85 cm long with a 2 m span and weighs 4.5 kg: the raven's layout drawn 1.35
times its size, the wings longer and broader for soaring (span against length 2.35, the raven's
1.9) with fingered tips, a strong hooked bill with a yellow cere, a heavy brow over the eye,
feathered legs (trousers) to yellow toes, a square tail.
"""

import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "..", ".claude", "skills", "blocky-model", "scripts")))
from house import TX  # noqa: E402
from paint import EYES  # noqa: E402

BROWN, DARK, GOLD, PALE, BILL, CERE, FOOT = range(7)
MATERIALS = [
    ("brown", (92, 68, 50), 0.02, ("coat", 0, 0)),  # the dark brown body and coverts
    ("dark", (52, 42, 38), 0.02, ("coat", 0, -2.0)),  # the flight feathers, the tail's tip
    ("gold", (198, 152, 88), 0.02, ("coat", 0, 2.0)),  # the golden crown and nape (its points)
    ("pale", (138, 104, 74), 0.02, ("coat", 1, 0)),  # the paler trousers and under-tail
    ("bill", (72, 70, 76), 0.74, "fixed"),
    ("cere", (226, 188, 72), 0.06, "fixed"),
    ("foot", (224, 184, 70), 0.06, "fixed"),
]

EYES.update({
    "eagle": dict(pattern=[".AA.",
                           "APHA",
                           "APPA",
                           ".aa."], colors={"A": (132, 84, 38), "a": (164, 108, 48), "P": (18, 12, 12), "H": (238, 236, 228)},
                  halo="round"),
    # At 32 a block the head's side is 4 texels by 4: the eye 2 by 2 at its front, under the
    # golden crown: a highlight and the pupil over the amber iris.
    "eagle_px": dict(pattern=["HP",
                              "Aa"], colors={"A": (176, 112, 44), "a": (204, 146, 62), "P": (18, 12, 12), "H": (238, 236, 228)},
                     halo="none", iris="Aa"),
})

HS = 1.15
K = 1.35
# Painted at 32 a block (house.TX, 0.5 texels a unit): the eye-bearing side of the head is 8.5 by
# 7.8 units, 4 by 4 texels at 32 (3+, so no finer).
TEX = 1 / (TX * K)  # a texel, in the raven's units


def area(e, face, p, uu, vv, h):
    """A golden eagle: dark brown, the crown and nape golden, the trousers paler, the flight
    feathers darker with a grey-barred base; the bill grey, darker at the hook, the cere and the
    toes yellow."""
    n = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, BROWN)
    if n in ("bill", "jaw", "bill_hook"):
        a[:] = BILL
        if n == "bill":
            front = 11.25 + 2.75 * HS
            a = np.where(z < front + 0.9, CERE, BILL)  # the yellow cere at its base
        return a
    if n == "toes_l" or n == "toes_r":
        a[:] = FOOT
        return a
    if n == "lid":
        a[:] = GOLD
        return a
    if n == "skull":
        if face in ("up", "north"):
            a[:] = GOLD
        elif face in ("east", "west"):
            a = np.where(y > 18.0 + 0.9, GOLD, BROWN)  # the golden crown down the nape
            a = np.where(z < 9.0, GOLD, a)
        return a
    if n == "neck":
        if face in ("north", "up", "east", "west"):
            a = np.where((face in ("north", "up")) | (z < 7.4), GOLD, BROWN)  # the golden hackles
        return a
    if n.startswith(("thigh_", "shin_")):
        a[:] = PALE
        return a
    if n.startswith(("primaries", "flight", "primariesf")):
        a[:] = DARK
        if n.startswith(("primaries_", "flight_")):
            a = np.where((z < -2.0) & (np.floor(np.abs(x) / max(1 / 0.7, 2 * TEX)) % 2 == 0), BROWN, a)  # faint barring at the base, 2+ texels a bar
        return a
    if n.startswith("tail"):
        a[:] = BROWN
        a = np.where(z < -5.0 - 14.6 * 0.8 + 3.0, DARK, a)  # the dark band at the tip
        return a
    if n.startswith(("wingf_", "wing_", "hand_")) and face == "up":
        a = np.where(z < 2.5, PALE, BROWN)  # the paler upperwing coverts
        return a
    return a


def props(e, face, p, a):
    """White barely ever (a few pale young); the golden crown and nape its points; marks (a savanna
    hawk's barring) on the body and the wings' undersides."""
    white = np.full(p.shape[:2], 0.1) + np.where(a == PALE, 0.3, 0.0)
    points = np.where(a == GOLD, 1.0, 0.0)
    pattern = np.where(np.isin(a, (BROWN, PALE)), 1.0, 0.0)
    return {"white": white, "points": points, "pattern": pattern}


CFG = dict(
    name="eagle",
    K=K,
    TX=TX,
    D=0.0,
    head=HS,
    bill=dict(w=1.9, h=2.3, len=3.6, rot=10, hook=True, y=0.7),
    body_w=1.15,
    tail=dict(len=0.8, w=1.2, cut="square"),
    wing=dict(arm=1.25, hand=1.2, chord=1.35, tip="fingers", folded=1.0),
    shin=0.95,
    toes=1.25,
    tilt=-30,
    eyes=dict(kind="side", pattern="eagle_px", z=11.25 + 2.75 * HS * 0.42, y=18.0 + 0.25 * HS, lash=(40, 30, 24)),
    materials=MATERIALS,
    feather=(BROWN, DARK, GOLD, PALE),
    area=area,
    props=props,
    idle="look",
    walk="walk",
    flap=dict(beat=24, hand=12, length=1.2, glide=0.6),
    blinks=dict(idle=[0.9, 2.3]),
    idle_len=3.0,
)
