#!/usr/bin/env python3
"""The small bird: one model for the house sparrow and the snow bunting, the songbirds of every
land biome (`notes/ECOLOGY.md`, tier 2: "small bird"), on the birds' build (`../bird/bird.py`, the
raven's layout with knobs), from research alone (README.md). Its colors are rules (`area`), painted
in a male house sparrow's; the snow bunting is the same model in its own colors, from its genes.

    ~/.venvs/blocky/bin/python ../bird/export.py small_bird

A house sparrow is 16 cm long with a 23 cm span and weighs 30 g: laid out in its own units (`layout`:
the raven's drawn this small would put faces too near each other), 11 from the bill to the tail's
tip and 15 across the open wings; a songbird's round head, a short thick seed-cracking bill, short
rounded wings, a short notched tail, short legs. Painted at 128 a block (2 texels a unit), finer
than the house's 32, so its face still carries an eye.
"""

import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.normpath(os.path.join(HERE, "..", "..", "..", "..", ".claude", "skills", "blocky-model", "scripts")))
from house import texels  # noqa: E402
from paint import EYES  # noqa: E402

BROWN, STREAK, GREY, CHESTNUT, BIB, CHEEK, BILL, LEG, BAR = range(9)
MATERIALS = [
    ("brown", (140, 98, 62), 0.02, ("coat", 0, 0)),  # the back, the wings' coverts
    ("streak", (70, 50, 38), 0.02, ("coat", 0, -2.5)),  # the back's dark streaks, the wings' centers
    ("grey", (176, 172, 164), 0.62, ("coat", 1, 0)),  # the crown, the breast and belly
    ("chestnut", (148, 82, 46), 0.02, ("coat", 0, -0.5)),  # the band from the eye round the nape
    ("bib", (44, 38, 36), 0.95, ("coat", 0, -4.0)),  # the male's black bib and the line through the eye
    ("cheek", (222, 216, 204), 0.62, ("coat", 1, 0.5)),  # the pale cheeks
    ("bill", (64, 58, 60), 0.74, "fixed"),
    ("leg", (176, 136, 112), 0.03, "fixed"),
    ("bar", (230, 224, 210), 0.62, ("coat", 1, 0.75)),  # the white wing bar
]

EYES.update({
    "small_bird": dict(pattern=[".PP.",
                                "PHPP",
                                "PPPP",
                                ".II."], colors={"I": (80, 54, 38), "P": (26, 18, 16), "H": (236, 234, 228)}, halo="none"),
    # At 128 a block the head's side is 4 texels by 4: the dark bead of an eye 2 by 2 in the
    # chestnut band, a highlight up front, the brown iris under the pupil.
    "small_bird_px": dict(pattern=["HP",
                                   "PI"], colors={"I": (96, 64, 44), "P": (26, 18, 16), "H": (236, 234, 228)}, halo="none", iris="I"),
})

# Painted at 128 a block (texels(k=4), 2 texels a unit; it was house.FINE, 256): the eye-bearing
# side of the head is 2.2 by 2.0 units, 1.1 by 1.0 texels at 32 and 2.2 by 2.0 at 64 (under 3),
# 4 by 4 at 128.
TX = texels(k=4)


def area(e, face, p, uu, vv, h):
    """A male house sparrow: brown back streaked dark, a grey crown and breast, a chestnut band from
    the eye round the nape, white cheeks, a black bib and eye line, a white wing bar; the bill dark,
    the legs pinkish brown."""
    n = e["name"]
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, BROWN)
    if n in ("bill", "jaw"):
        a[:] = BILL
        return a
    if n.startswith(("shin_", "toes_")):
        a[:] = LEG
        return a
    if n == "lid":  # a closed eye: the chestnut band it sits in
        a[:] = CHESTNUT
        return a
    if n == "skull":
        if face == "up":
            a[:] = GREY
        elif face == "down":
            a[:] = BIB
        elif face == "north":
            a = np.where(y > 5.9, GREY, CHESTNUT)
        elif face == "south":
            a = np.where(y < 4.75, BIB, np.where(y > 5.9, GREY, CHESTNUT))
        else:
            a = np.where(y > 5.95, GREY, np.where(y > 5.15, CHESTNUT, CHEEK))
            a = np.where((y > 5.2) & (y < 5.6) & (z > 3.3), BIB, a)  # the line from the eye to the bill
            a = np.where((y < 4.75) & (z > 2.9), BIB, a)  # the bib's top under the bill
        return a
    if n == "neck":
        a[:] = CHESTNUT if face in ("north", "up") else BIB
        if face in ("east", "west"):
            a = np.where(z > 2.3, BIB, np.where(y > 4.3, CHESTNUT, GREY))
        return a
    if n == "chest":
        if face == "up":
            a[:] = BROWN
        elif face == "down":
            a[:] = GREY
        elif face == "south":
            a = np.where(y > 3.3, BIB, GREY)  # the black bib down the throat and upper breast
        else:
            a = np.where(vv < h * 0.3, BROWN, GREY)
            a = np.where((z > 1.5) & (y > 3.5), BIB, a)
        return a
    if n == "hips":
        if face in ("up", "north"):
            a[:] = BROWN
            a = np.where((np.floor(x * 2) % 3 == 0) & (face == "up"), STREAK, a)  # the streaked back (a streak a texel)
        elif face == "down":
            a[:] = GREY
        else:
            a = np.where(vv < h * 0.35, BROWN, GREY)
        return a
    if n.startswith(("wingf_", "wing_", "hand_")):
        a[:] = BROWN
        if face in ("east", "west", "up"):
            a = np.where(np.floor(z * 2) % 3 == 0, STREAK, a)  # the coverts' dark centers (a texel each)
            a = np.where((y > 3.03) & (y < 3.53) & (face != "up"), BAR, a)  # the white wing bar, a texel
            a = np.where((z > 0.8) & (z < 1.3) & (face == "up"), BAR, a)
        if face == "down":
            a[:] = GREY
        return a
    if n.startswith(("primaries", "flight")):
        a[:] = STREAK
        a = np.where((np.abs(x) % 1.5) < 0.5, BROWN, a)  # pale edges to the dark feathers (a texel, two dark between)
        return a
    if n.startswith("primariesf"):
        a[:] = STREAK
        return a
    if n.startswith("tail"):
        a[:] = STREAK if face == "up" else BROWN
        return a
    if n.startswith("thigh_"):
        a[:] = GREY
        return a
    return a


def props(e, face, p, a):
    """White where the bird is pale (the snow bunting whitens from the belly up); the bib and eye
    line its points; streaks (marks) over the back and wings."""
    y = p[..., 1]
    white = np.clip((4.4 - y) / 2.2, 0, 1) * 0.6 + np.where((a == GREY) | (a == CHEEK) | (a == BAR), 0.4, 0.0)
    points = np.where(a == BIB, 1.0, 0.0)
    pattern = np.where(np.isin(a, (BROWN, STREAK)), 1.0, 0.0)
    return {"white": white, "points": points, "pattern": pattern}


def el(name, bone, lo, hi, quad=None, rot=None, origin=None, cut=None):
    return dict(name=name, bone=bone, lo=[float(v) for v in lo], hi=[float(v) for v in hi], quad=quad, rot=rot,
                origin=[float(v) for v in origin] if origin is not None else None, cut=cut)


def layout():
    """The sparrow in its own units (the raven's layout drawn this small would put neighboring
    faces closer than they can be without flickering): every pair of facing planes 0.3 apart or
    more, each joined part a size apart from the next."""
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 3.4, -0.9)), ("spine_1", "spine_0", (0, 3.4, 0.2)),
         ("neck", "spine_1", (0, 3.9, 2.0)), ("head", "neck", (0, 4.9, 2.0)), ("jaw", "head", (0, 4.7, 3.7)),
         ("tail", "spine_0", (0, 3.4, -1.9))]
    E = [el("hips", "spine_0", (-1.25, 2.9, -2.2), (1.25, 4.2, 0.4)),
         el("chest", "spine_1", (-1.6, 2.2, -0.2), (1.6, 4.6, 2.4)),
         el("neck", "neck", (-0.3, 2.55, 1.95), (0.3, 5.3, 2.05)),  # hidden: the head sits on the breast
         el("skull", "head", (-1.15, 4.3, 1.6), (1.15, 6.3, 3.8)),  # a songbird's round head
         el("bill", "head", (-0.55, 4.75, 3.6), (0.55, 5.45, 4.95), rot=(10, 0, 0), origin=(0, 5.1, 3.6)),  # short and thick
         el("jaw", "jaw", (-0.2, 4.62, 3.5), (0.2, 4.98, 4.6)),
         el("tail", "tail", (-0.75, 3.2, -5.6), (0.75, 3.9, -1.6)),
         el("tail_fan", "tail", (-1.25, 3.55, -6.2), (1.25, 3.55, -2.5), quad="up", cut="tail")]
    for s, n in ((1, "L"), (-1, "R")):
        k = n.lower()
        B += [(f"wingf_{n}", "spine_1", (s * 1.55, 3.6, 1.2)), (f"wing_{n}", "spine_1", (s * 1.1, 3.6, 1.3)),
              (f"hand_{n}", f"wing_{n}", (s * 4.8, 3.6, 1.3)), (f"leg_{n}_thigh", "spine_0", (s * 0.45, 3.0, -0.45)),
              (f"leg_{n}_shin", f"leg_{n}_thigh", (s * 0.45, 1.8, -0.45)), (f"leg_{n}_toes", f"leg_{n}_shin", (s * 0.45, 0.5, -0.45))]
        X = lambda a, b: sorted((s * a, s * b))  # noqa: E731
        # On the chest's side and two texels thick, its top just under the chest's (from 1.2 to 1.95 and up
        # to 3.85 it stood 1.4 texels out of the hips and showed its top and back as lines beside the
        # chest and the hips; linecheck).
        (x0, x1) = X(1.6, 2.6)
        E.append(el(f"wingf_{k}", f"wingf_{n}", (x0, 2.55, -1.8), (x1, 4.25, 2.0)))
        E.append(el(f"primariesf_{k}", f"wingf_{n}", (s * 2.1, 2.5, -4.4), (s * 2.1, 3.6, -1.0), quad="east" if s > 0 else "west", cut="folded"))  # mid-wing
        (x0, x1) = X(1.0, 5.0)
        # Round the thicker hand with a flush step, its top a flush step under the chest's (was 2.85 to 4.26).
        E.append(el(f"wing_{k}", f"wing_{n}", (x0, 2.55, 0.4), (x1, 4.25, 2.0)))
        (x0, x1) = X(4.6, 6.8)
        # Two texels thick (at 0.7 its front and back read as lines beside the arm; linecheck).
        E.append(el(f"hand_{k}", f"hand_{n}", (x0, 2.9, 0.75), (x1, 3.9, 1.65)))
        (x0, x1) = X(1.1, 4.6)
        E.append(el(f"flight_{k}", f"wing_{n}", (x0, 3.6, -2.4), (x1, 3.6, 0.6), quad="down", cut="secondaries"))
        (x0, x1) = X(4.7, 7.4)
        E.append(el(f"primaries_{k}", f"hand_{n}", (x0, 3.6, -2.2), (x1, 3.6, 0.7), quad="down", cut="primaries"))
        (x0, x1) = X(0.0, 0.9)
        # Feathered. Its bottom a flush step under the chest's, its back two texels behind the chest's (from 1.6
        # and -0.9, 1.2 and 1.4 texels of it showed as lines beside the chest and the hips; linecheck).
        E.append(el(f"thigh_{k}", f"leg_{n}_thigh", (x0, 1.7, -1.3), (x1, 3.2, 0.0)))
        (x0, x1) = X(0.3, 0.6)
        E.append(el(f"shin_{k}", f"leg_{n}_shin", (x0, 0.35, -0.57), (x1, 1.95, -0.33)))
        (x0, x1) = X(0.0, 0.9)
        E.append(el(f"toes_{k}", f"leg_{n}_toes", (x0, 0.0, -0.9), (x1, 0.7, 1.0)))
    G = dict(skull=((-1.15, 4.3, 1.6), (1.15, 6.3, 3.8)), hand_x=4.8, flight_z0=-2.4, prim_x0=4.7, prim_end=7.4, prim_z0=-2.2,
             prim_z1=0.7, fan_end=-6.2, fan_w=1.25, fan_z1=-2.5, fold_end=-4.4, fold_z1=-1.0, fold_y=3.05, fold_h=0.55, u=0.25)
    return dict(bones=B, elements=E, G=G)


CFG = dict(
    name="small_bird",
    K=1.0,
    D=0.0,
    layout=layout(),
    TX=TX,  # 128 a block (see TX above)
    head=1.5,
    bill=dict(w=2.6, h=2.6, len=3.0, rot=12, hook=False, y=0.55),
    body_w=1.12,
    tail=dict(len=0.62, w=0.9, cut="notch"),
    wing=dict(arm=0.72, hand=0.78, chord=1.05, tip="round", folded=0.7),
    shin=0.42,
    toes=0.9,
    tilt=-24,
    neck_share=0.0,
    neck_hidden=True,  # the head sits on the breast: the neck is a hidden link
    eyes=dict(kind="side", pattern="small_bird_px", z=2.9, y=5.5, lash=(40, 30, 26)),  # z: the eye a texel back from the bill
    materials=MATERIALS,
    feather=(BROWN, STREAK, GREY, CHESTNUT, BIB, CHEEK, BAR),
    area=area,
    props=props,
    idle="look",
    walk="hop",
    flap=dict(beat=52, hand=26, length=0.16, glide=0.0),
    blinks=dict(idle=[0.8, 2.2]),
    idle_len=3.0,
)
