#!/usr/bin/env python3
"""The owl: one model for the great horned owl and the snowy owl (*Bubo*), the night's hunter of
every land biome (`notes/ECOLOGY.md`, tier 2), on the birds' build (`../bird/bird.py`, the raven's
layout with knobs), from research alone (README.md). Its colors are rules (`area`), painted in a
great horned owl's; the snowy owl is the same model in its own colors and nearly without ear tufts,
from its genes.

    ~/.venvs/blocky/bin/python ../bird/export.py owl

A great horned owl is about 55 cm long with a 1.2 m span and weighs 1.4 kg: the raven's layout
drawn 0.95 times its size, the head 1.75 times the raven's (an owl's big round head, wider than it
is deep, sitting on its shoulders), a short hooked bill turned down into the facial disc, both
eyes on the face, ear tufts on their own bones, a stocky upright body, broad rounded wings, a short
rounded tail, feathered legs and toes.
"""

import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "..", ".claude", "skills", "blocky-model", "scripts")))
from house import SCALE, texels  # noqa: E402
from paint import EYES  # noqa: E402

BROWN, DARK, BUFF, RUFOUS, WHITE, BILL = range(6)
MATERIALS = [
    ("brown", (124, 100, 78), 0.02, ("coat", 0, 0)),  # the mottled grey-brown upperparts
    ("dark", (60, 48, 42), 0.02, ("coat", 0, -2.5)),  # bars and mottling, the facial disc's rim
    ("buff", (198, 170, 132), 0.06, ("coat", 1, 0)),  # the underparts, the feathered legs
    ("rufous", (176, 116, 70), 0.02, ("coat", 0.5, 0)),  # the facial disc
    ("white", (234, 228, 216), 0.62, ("coat", 1, 0.75)),  # the throat and the brows
    ("bill", (54, 50, 52), 0.74, "fixed"),
]

EYES.update({
    "owl": dict(pattern=[".AA.",
                         "APHA",
                         "APPA",
                         ".aa."], colors={"A": (226, 168, 40), "a": (240, 190, 70), "P": (16, 12, 12), "H": (240, 238, 228)},
                halo="round"),
    # At 32 a block the face is 4 texels by 4: each eye a texel wide and two tall at the face's
    # edges, the pupil over the yellow iris (the house's px2 order), the rufous disc and the bill
    # between them. Two 2x2 eyes would fill the middle rows edge to edge and read as one band.
    "owl_px": dict(pattern=["P",
                            "A"], colors={"A": (226, 168, 40), "P": (16, 12, 12)}, halo="none", iris="A"),
})

HS, HY = 1.75, 1.5  # the head's size against the raven's, and how much higher it sits
CY = 18.0 + HY
EYE_X, EYE_Y = 2.2, 18.0 + 0.55 * HS  # on the face (EYE_Y without the head's lift)
K = 0.95
# Painted at 64 a block (1 texel a unit), an exception to the house's 32: an owl's face is its two
# big ringed eyes, and at 32 the face is 4 by 4 texels, where they shrink to a texel each and the
# owl stops reading as one (tried: `owl_px`). At 64 the face is 8 by 8 and keeps the 4x4 design.
TX = texels(SCALE, 2)
TEX = 1 / (TX * K)  # a texel, in the raven's units
EYE_PX_X = EYE_X  # the painted eyes' centers (at 32: 3.4, the face's outer texel columns)


def mottle(x, y, z, s=0):
    """Fixed dark flecks: a hash of where the texel is."""
    f = min(1.3, 1 / TEX)  # a fleck at least a texel
    h = np.sin(np.floor(x * f) * 12.9898 + np.floor(y * f) * 78.233 + np.floor(z * f) * 37.719 + s) * 43758.5453
    return (h - np.floor(h)) > 0.72


def area(e, face, p, uu, vv, h):
    """A great horned owl: mottled grey-brown above, buff below barred dark, a rufous facial disc
    rimmed black, white brows and throat, dark ear tufts; the bill dark grey, the legs and toes
    feathered buff."""
    n = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, BROWN)
    if n in ("bill", "jaw", "bill_hook"):
        a[:] = BILL
        return a
    if n == "lid":
        a[:] = RUFOUS
        return a
    if n == "skull":
        if face == "south":
            d = np.minimum(np.hypot(x - EYE_X, y - (EYE_Y + HY)), np.hypot(x + EYE_X, y - (EYE_Y + HY)))
            a = np.where(d < 2.9, RUFOUS, np.where(d < 2.9 + max(0.7, TEX), DARK, BROWN))  # the rim at least a texel
            a = np.where((np.abs(x) < 1.0) & (y > EYE_Y + HY - 0.6) & (y < EYE_Y + HY + 2.4), WHITE, a)  # the pale brows meeting over the bill
            a = np.where((y < CY - 2.6) & (np.abs(x) < 3.0), WHITE, a)  # the white throat
        elif face == "down":
            a[:] = WHITE
        else:
            a = np.where(mottle(x, y, z), DARK, BROWN)
        return a
    if n.startswith("tuft"):
        a[:] = DARK
        a = np.where(mottle(x, y, z, 3), BROWN, a)
        return a
    if n in ("chest", "hips", "neck"):
        if face == "up" or (face == "north" and n == "hips"):
            return np.where(mottle(x, y, z, 1), DARK, BROWN)
        a = np.where(vv < h * (0.25 if face in ("east", "west") else 0.0), BROWN, BUFF)
        bars = (np.floor(y / max(1 / 1.1, TEX)) % 3 == 0) & (face != "down")  # a bar a texel, two between
        a = np.where((a == BUFF) & bars, DARK, a)  # the barred breast and belly
        if n == "chest" and face == "south":
            a = np.where(y > 14.6, WHITE, a)  # the white bib under the face
        return a
    if n.startswith(("wingf_", "wing_", "hand_")):
        if face == "down":
            a[:] = BUFF
            return a
        return np.where(mottle(x, y, z, 2), DARK, np.where(mottle(x, y, z, 5), BUFF, BROWN))
    if n.startswith(("primaries", "flight", "primariesf")):
        along = np.abs(x) if not n.startswith("primariesf") else -z
        return np.where(np.floor(along / max(1 / 0.8, 1.5 * TEX)) % 2 == 0, DARK, BROWN)  # the barred flight feathers, 1.5+ texels a bar
    if n.startswith("tail"):
        return np.where(np.floor(-z / max(1 / 0.8, 1.5 * TEX)) % 2 == 0, DARK, BROWN)
    if n.startswith(("thigh_", "shin_", "toes_")):
        a[:] = BUFF
        return a
    return a


def props(e, face, p, a):
    """White where the owl is pale (a snowy owl whitens all over); the facial disc its points; marks
    (a snowy owl's dark bars, `streaks`) over the body and wings."""
    y = p[..., 1]
    white = np.clip((16.0 - y) / 9.0, 0, 1) * 0.5 + np.where((a == BUFF) | (a == WHITE), 0.4, 0.0) + 0.1
    points = np.where(a == RUFOUS, 1.0, 0.0)
    pattern = np.where(np.isin(a, (BROWN, DARK, BUFF)), 1.0, 0.0)
    return {"white": white, "points": points, "pattern": pattern}


CFG = dict(
    name="owl",
    K=K,
    TX=TX,
    D=-1.2,
    head=HS,
    head_y=HY,
    bill=dict(w=1.6, h=2.0, len=2.2, rot=48, hook=False, y=-0.2),
    body_w=1.3,
    tail=dict(len=0.5, w=1.15, cut="round"),
    wing=dict(arm=0.95, hand=0.85, chord=1.3, tip="round", folded=0.6),
    shin=0.95,
    toes=1.2,
    tufts=(3.6, 1.5),
    tilt=-50,
    neck_share=0.0,
    neck_hidden=True,  # the head sits on the shoulders: the neck is a hidden link
    eyes=dict(kind="front", pattern="owl", x=EYE_PX_X, y=EYE_Y, lash=(52, 34, 26)),
    materials=MATERIALS,
    feather=(BROWN, DARK, BUFF, RUFOUS, WHITE),
    area=area,
    props=props,
    idle="owl",
    walk="walk",
    flap=dict(beat=30, hand=14, length=0.7, glide=0.35),
    blinks=dict(idle=[0.5, 3.0]),  # while it looks straight ahead
    idle_len=5.0,
)
