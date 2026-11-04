#!/usr/bin/env python3
"""The gull: the herring gull (*Larus argentatus*), the coast's hunter and its eater of the dead
(`notes/ECOLOGY.md`, tier 2: "gull"), on the birds' build (`../bird/bird.py`, the raven's layout
with knobs), from research alone (README.md). Its colors are rules (`area`).

    ~/.venvs/blocky/bin/python ../bird/export.py gull

A herring gull is about 60 cm long with a 1.4 m span and weighs 1 kg: the raven's layout drawn 0.95
times its size, long narrow pointed wings (span against length 2.35) whose black tips cross over
the tail when folded, a long yellow bill with a red spot near the tip of the lower mandible, a pale
eye, a short square tail, longer pink legs with webbed feet; carried level.
"""

import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "..", ".claude", "skills", "blocky-model", "scripts")))
from paint import EYES  # noqa: E402

GREY, WHITE, BLACK, BILL, SPOT, LEG = range(6)
MATERIALS = [
    ("grey", (170, 178, 188), 0.62, ("coat", 0, 0)),  # the pale grey mantle and upper wings
    ("white", (238, 236, 232), 0.62, ("coat", 1, 0)),  # the head, the underparts, the tail
    ("black", (40, 40, 46), 0.72, ("coat", 0, -4.0)),  # the wingtips (its points)
    ("bill", (232, 196, 72), 0.06, "fixed"),
    ("spot", (206, 64, 46), 0.04, "fixed"),  # the red spot the chicks peck at
    ("leg", (226, 170, 160), 0.03, "fixed"),
]

EYES.update({
    "gull": dict(pattern=["YYY",
                          "YPH",
                          "YYY"], colors={"Y": (226, 208, 136), "P": (22, 18, 16), "H": (244, 242, 236)}, halo="none"),
})

HS = 1.05
FRONT = 11.25 + 2.75 * HS


def area(e, face, p, uu, vv, h):
    """An adult herring gull: white head, neck, underparts and tail; the back and wings pale grey,
    the wingtips black with white mirrors and white tips; the bill yellow with a red spot near the
    lower tip; the legs pink."""
    n = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, WHITE)
    if n == "bill":
        a[:] = BILL
        return a
    if n == "jaw":
        a[:] = BILL
        return np.where(z > FRONT + 2.3, SPOT, a)  # the red gonys spot
    if n.startswith(("shin_", "toes_")):
        a[:] = LEG
        return a
    if n in ("skull", "neck", "lid", "tail", "tail_fan", "chest") or n.startswith("thigh_"):
        return a
    if n == "hips":
        if face == "up":
            a[:] = GREY
        elif face in ("east", "west"):
            a = np.where(vv < h * 0.3, GREY, WHITE)
        return a
    if n.startswith("wingf_"):
        if face != "down":
            a[:] = GREY
            a = np.where((z < -4.0) & (face != "up"), WHITE, a)  # the white tertials' edge
        return a
    if n.startswith("primariesf"):
        a[:] = BLACK
        t = (-4.0 - z)
        a = np.where((t > 9.0) & (np.abs(y - 12.0) < 0.6), WHITE, a)  # the white tips
        return a
    if n.startswith(("wing_", "hand_")):
        a[:] = GREY if face != "down" else WHITE
        return a
    if n.startswith("flight"):
        a[:] = GREY
        return np.where(z < -3.6, WHITE, a)  # the white trailing edge
    if n.startswith("primaries"):
        ax = np.abs(x)
        x0 = 4.0 + 14.2 * 1.15 - 0.4 - 0.2
        span = 21.4 * 1.25
        t = (ax - x0) / span
        a = np.where(t > 0.45, BLACK, GREY)  # the black tips
        a = np.where((t > 0.78) & (t < 0.9) & (z > -3.0) & (z < 0.6), WHITE, a)  # a white mirror
        return a
    return a


def props(e, face, p, a):
    """White where the gull is white (all its underside and head); the wingtips its points; marks
    (a young gull's brown mottling) faint."""
    white = np.where(a == WHITE, 0.95, 0.2)
    points = np.where(a == BLACK, 1.0, 0.0)
    pattern = np.where(np.isin(a, (GREY, WHITE)), 0.6, 0.0)
    return {"white": white, "points": points, "pattern": pattern}


CFG = dict(
    name="gull",
    K=0.95,
    D=0.7,
    head=HS,
    bill=dict(w=1.3, h=1.9, len=4.8, rot=5, hook=False, y=0.6),
    body_w=1.05,
    tail=dict(len=0.6, w=1.0, cut="square"),
    wing=dict(arm=1.15, hand=1.25, chord=0.9, tip="pointed", folded=1.25),
    shin=0.45,
    toes=1.15,
    tilt=-12,
    eyes=dict(kind="side", pattern="gull", z=11.6, y=18.0 + 0.3 * HS, lash=(150, 150, 156)),
    materials=MATERIALS,
    feather=(GREY, WHITE, BLACK),
    area=area,
    props=props,
    idle="look",
    walk="walk",
    flap=dict(beat=28, hand=16, length=0.8, glide=0.4),
    blinks=dict(idle=[0.9, 2.3]),
    idle_len=3.0,
)
