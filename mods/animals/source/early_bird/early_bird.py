#!/usr/bin/env python3
"""The early bird: one model for Confuciusornis sanctus of the Jehol's forests and lakes and the
enantiornithine Gobipteryx minuta of the Gobi (`notes/ECOLOGY.md`, the Cretaceous's tier 2: "an
early bird"), on the birds' build (`../bird/bird.py`, the raven's layout with knobs), from research
(README.md). Its colors are rules (`area`), painted in Confuciusornis's; Gobipteryx is the same
model in its own colors, from its genes.

    ~/.venvs/blocky/bin/python ../bird/export.py early_bird

Confuciusornis was about the size of a crow (0.2-1.5 kg, est.; here 0.5 kg), the first bird with a
toothless beak: the raven's layout drawn 0.85 times its size, a short deep beak, broad wings with a
big clawed finger at their bend (here the fingered tip), a long tail (the male's two ribbon
feathers, here a long notched tail), short legs; carried as a perching bird.
"""

import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "..", ".claude", "skills", "blocky-model", "scripts")))
from house import texels  # noqa: E402
from paint import EYES  # noqa: E402

DARK, BROWN, PALE, SPOT, BILL, LEG, CLAW = range(7)
MATERIALS = [
    ("dark", (72, 66, 64), 0.72, ("coat", 0, -1.5)),  # grey-black: the head, the back's mottling, the wings' ends
    ("brown", (128, 98, 72), 0.02, ("coat", 0, 0)),  # the back and the coverts
    ("pale", (196, 186, 168), 0.62, ("coat", 1, 0)),  # the throat, breast and belly
    ("spot", (214, 202, 178), 0.62, ("coat", 1, 0.5)),  # the pale spots of the wings (Li et al. 2018)
    ("bill", (60, 54, 52), 0.74, "fixed"),  # the horny beak, dark
    ("leg", (92, 80, 72), 0.03, "fixed"),
    ("claw", (40, 34, 32), 0.74, "fixed"),
]

EYES.update({
    # At 64 a block the head's side is about 4 texels by 4: the eye 2 by 2, a catchlight up and
    # forward over the pupil, a dark-brown iris (no record: a crow's dark eye).
    "early_bird_px": dict(pattern=["HP",
                                   "IP"], colors={"H": (234, 232, 226), "P": (18, 14, 12), "I": (90, 62, 40)}, halo="none", iris="I"),
})

# Painted at 64 a block (texels(k=2), 1 texel a unit): the eye-bearing side of the head is about 4.0
# by 3.8 units, 2 texels at 32 (under 3), 4 at 64.
TX = texels(k=2)

HS = 1.0
FRONT = 11.25 + 2.75 * HS


def area(e, face, p, uu, vv, h):
    """Confuciusornis as its fossils' color cells suggest (Li et al. 2018: mottled grey-black and
    brown, the wings spotted pale): a grey-black head, a brown back mottled darker, pale below,
    the wings brown with pale spots and darker ends, the tail dark; a dark beak and claws."""
    n = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, BROWN)
    if n in ("bill", "jaw"):
        a[:] = BILL
        return a
    if n.startswith("toes_"):
        a[:] = CLAW
        return a
    if n.startswith("shin_"):
        a[:] = LEG
        return a
    if n in ("skull", "lid"):
        a[:] = DARK
        return np.where((face == "down") | ((face in ("east", "west")) & (vv > h * 0.75)), PALE, a)  # a pale throat
    if n == "neck":
        return np.where(face == "up", DARK, np.where((face == "down") | (vv > h * 0.4), PALE, BROWN))
    if n in ("chest", "hips") or n.startswith("thigh_"):
        if face == "down":
            a[:] = PALE
        elif face in ("east", "west", "south"):
            a = np.where(vv > h * 0.45, PALE, BROWN)
        mottle = ((np.floor(x / 2.0) + np.floor(z / 2.0)) % 3) == 0
        return np.where((a == BROWN) & mottle, DARK, a)
    if n in ("tail", "tail_fan"):
        a[:] = DARK
        return a
    if n.startswith(("primaries", "primariesf")):
        a[:] = DARK
        return a
    if n.startswith(("wing_", "hand_", "flight", "wingf_")):
        if face == "down":
            a[:] = PALE
            return a
        spots = ((np.floor(x / 2.0) + 2 * np.floor(z / 2.0)) % 4) == 0
        return np.where(spots, SPOT, BROWN)
    return a


def props(e, face, p, a):
    """White where it is pale (the underside); the dark wing ends and tail its points; marks (the
    mottling and the spots) where the back and wings are."""
    white = np.where(np.isin(a, (PALE, SPOT)), 0.9, 0.15)
    points = np.where(a == DARK, 0.6, 0.0)
    pattern = np.where(np.isin(a, (BROWN, DARK, SPOT)), 0.8, 0.1)
    return {"white": white, "points": points, "pattern": pattern}


CFG = dict(
    name="early_bird",
    K=0.85,
    TX=TX,
    D=-0.3,
    head=HS,
    bill=dict(w=1.6, h=2.2, len=3.0, rot=8, hook=False, y=0.9),  # short, deep and toothless
    body_w=1.0,
    tail=dict(len=1.5, w=0.9, cut="notch"),  # the long tail (the male's streamers)
    wing=dict(arm=1.0, hand=0.95, chord=1.1, tip="fingers", folded=1.0),  # broad, the clawed finger at the bend
    shin=0.45,
    toes=1.1,
    tilt=-12,
    eyes=dict(kind="side", pattern="early_bird_px", z=11.8, y=18.0 + 0.3 * HS, lash=(60, 56, 54)),
    materials=MATERIALS,
    feather=(DARK, BROWN, PALE, SPOT),
    area=area,
    props=props,
    idle="look",
    walk="hop",
    flap=dict(beat=36, hand=20, length=0.45, glide=0.1),
    blinks=dict(idle=[0.9, 2.3]),
    idle_len=3.0,
)
