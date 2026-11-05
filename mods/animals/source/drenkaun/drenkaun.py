#!/usr/bin/env python3
"""Drenkaun, the Wood-Warden: the old forests' boss (`mods/animals/src/drenkaun.rs`), made with the
blocky-model skill on the reindeer's build (`../reindeer/reindeer.py`, itself on the deer's): laid
out in the deer's units, drawn `K` times bigger (about 4 m at the shoulder), with antlers like oak
boughs, a moss cape and mane, bark plates on its back and a bark crown between its antlers. Its
colors are concept A's (Ronald, 11-05: "A: lighter, leaner elk"; `README.md`); its clips are the
fight's (`drenkaun.rs` plays them by name).

    ~/.venvs/blocky/bin/python drenkaun.py     # writes out/spec.json, out/texture.png

Units are 1/64 of a block, y up, it faces +z; painted at the house density, 32 texels a block.
It has no coat genes (a boss, one of its kind), so no coat layers.
"""

import json
import math
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
SKILL = os.path.normpath(os.path.join(HERE, "..", "..", "..", "..", ".claude", "skills", "blocky-model", "scripts"))
sys.path.insert(0, SKILL)
from paint import EYES, eye_corner, occlusion, pack, paint_face_smooth, stamp_eyes, tones  # noqa: E402
from rig import fit  # noqa: E402
from ground import lowest  # noqa: E402
from house import TX  # noqa: E402

OUT = os.environ.get("DRENKAUN_OUT") or os.path.join(HERE, "out")
TAU = 2 * math.pi
S = math.sin
K = 4.0  # the deer's layout drawn this much bigger: the withers 3.75 m up, 6.9 m nose to tail

# ------------------------------------------------------------------ materials and eyes

FUR, BACK, BELLY, DARK, EARIN, MOUTH, ANTLER, HOOF, TAILTOP, LEG, PALE, BONE, MOSS, MOSSDK, BARK, BARKDK, BARKLT, MUZZLE, NOSE = range(19)
# From concept A: a warm mid-brown coat (FUR), darker along the spine (BACK), a cream-tan belly,
# throat and muzzle (BELLY, PALE), dark brown legs (LEG), dark hooves, a cream tail; antlers of
# bark-brown (ANTLER) to bone-pale tips (BONE); a yellow-green moss cape and mane (MOSS, its shade
# MOSSDK); grey-brown bark plates (BARK).
BASE = [(150, 88, 46), (104, 60, 32), (224, 190, 136), (48, 34, 26), (190, 140, 110), (84, 42, 44), (140, 92, 54),
        (56, 40, 32), (226, 196, 146), (86, 52, 30), (210, 164, 108), (232, 218, 186), (156, 158, 46), (102, 108, 30),
        (122, 92, 66), (84, 62, 44), (152, 120, 88), (166, 118, 74), (84, 60, 46)]
SHADOW = [0.02, 0.02, 0.06, 0.95, 0.97, 0.95, 0.04, 0.74, 0.06, 0.02, 0.06, 0.06, 0.12, 0.12, 0.05, 0.04, 0.06, 0.03, 0.03]
TONES = [tones(c, h) for c, h in zip(BASE, SHADOW)]
COAT = (FUR, BACK, BELLY, LEG, PALE, MOSS, MOSSDK)

# Its eyes are the skill's designed ones (`paint.EYES`; Ronald, 11-05, on round 2's small amber
# block: "idk the eyes look weird lol, are u using our skill ??"), stamped on the head's sides as a
# deer's are, with the design's darker ring round them and lighter fur below, mirrored on the
# west side so both catchlights look forward, and lids that blink. At HS the head's side is 34
# texels by 29, so Hytale's 8-wide eyes sit as on its own creatures. Three to pick from:
# `hytale_amber` (concept A's amber), `hytale` (the type Ronald picked before), `hytale_cute`.
EYE_TYPES = ("hytale_amber", "hytale", "hytale_cute")


def scaled(design, k):
    """A design drawn `k` times bigger, texel for texel (nearest)."""
    pat = design["pattern"]
    h, w = len(pat), len(pat[0])
    H, W = round(h * k), round(w * k)
    return {**design, "pattern": ["".join(pat[min(h - 1, int(r / k))][min(w - 1, int(c / k))] for c in range(W)) for r in range(H)]}


# Ronald, 11-05, round 3: "A amber", "eyes a bit bigger": the amber design 1.5 times as big.
# Drawn round at 12 by 10 rather than stretched (a nearest stretch doubles rows unevenly): the same
# dark ring, amber iris, near-black pupil and catchlight, its warmer amber low.
EYES["warden_amber"] = {**EYES["hytale_amber"], "pattern": [
    "...LLLLLL...",
    ".LLAAAAAALL.",
    ".LAAAAAAAAL.",
    "LAAHHHPPPAAL",
    "LAAHHPPPPAAL",
    "LAAPPPPPPAAL",
    "LAAPPPPPPAAL",
    ".LAaPPPPaAL.",
    ".LLaaAAaaLL.",
    "...LLLLLL...",
]}
assert all(len(r) == 12 for r in EYES["warden_amber"]["pattern"])
for _t in EYE_TYPES + ("warden_amber",):
    EYES[_t + "_m"] = {**EYES[_t], "pattern": [row[::-1] for row in EYES[_t]["pattern"]]}
EYE_STYLE = os.environ.get("DRENKAUN_EYES", "warden_amber")

# In the deer's space (scaled by K when the model is built). The head is drawn `HS` times its
# reindeer size about its pivot (Ronald, 11-05: "the head is small: scale it up a little").
HEAD = np.array([0.0, 66.0, 38.0])
# The moss over the shoulders and back: "tufts" (round 5, approved: pads thinning to the hips) or
# "shell" (the skill's rule 6, Ronald 11-05 "Yes, all 11": one shell a little bigger than the
# shoulders, its hem cut out ragged, as Mojang's sniffer wears its moss). A try to pick from.
MOSS_STYLE = os.environ.get("DRENKAUN_MOSS", "tufts")
SHELL = (-13.3, 42.0, -9.0), (13.3, 65.0, 30.6)  # 2 units out from the shoulders on every side
HS = 1.22


def hs(p):
    """A point of the head, drawn `HS` times bigger about the head's pivot."""
    return tuple(HEAD + (np.array(p, float) - HEAD) * HS)


SKULL0 = (hs((-7.0, 60.0, 32.0)), hs((7.0, 72.0, 46.0)))
EYE_Z0, EYE_Y0 = hs((0, 67.6, 40.5))[2], hs((0, 67.6, 40.5))[1]
SKULL = tuple(tuple(v * K for v in c) for c in SKULL0)
EYE_Z, EYE_Y = EYE_Z0 * K, EYE_Y0 * K
LID_X = SKULL[1][0] + 0.4  # the eyelids' hinge plane, just outside the head's side (scaled)

# ------------------------------------------------------------------ the model


def el(name, bone, lo, hi, quad=None, rot=None, origin=None, cut=None):
    return dict(name=name, bone=bone, lo=[float(v) for v in lo], hi=[float(v) for v in hi], quad=quad, rot=rot,
                origin=[float(v) for v in origin] if origin is not None else None, cut=cut)


def rotm(rot):
    if not rot:
        return np.eye(3)
    rx, ry, rz = (math.radians(a) for a in rot)
    X = np.array([[1, 0, 0], [0, math.cos(rx), -math.sin(rx)], [0, math.sin(rx), math.cos(rx)]])
    Y = np.array([[math.cos(ry), 0, math.sin(ry)], [0, 1, 0], [-math.sin(ry), 0, math.cos(ry)]])
    Z = np.array([[math.cos(rz), -math.sin(rz), 0], [math.sin(rz), math.cos(rz), 0], [0, 0, 1]])
    return Z @ Y @ X


def rod(name, bone, start, rot, length, w, back=0.4):
    """A box along +y from `start` turned by `rot` (about `start`), `w` across, reaching `back`
    behind its start (inside what it grows from); and its far end."""
    s = np.array(start, float)
    e = el(name, bone, (s[0] - w / 2, s[1] - back, s[2] - w / 2), (s[0] + w / 2, s[1] + length, s[2] + w / 2), rot=rot, origin=s)
    return e, s + rotm(rot) @ np.array([0, length - 0.6, 0])


def eye_style(side):
    """The eye design on the head's east (side > 0) or west side: mirrored on the west, so both
    catchlights face forward."""
    return EYE_STYLE + "_m" if side < 0 and EYE_STYLE + "_m" in EYES else EYE_STYLE


def skull_grid():
    """The (scaled) skull side's texels: (width, height) and the size of one texel in units (u, v)."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h = max(1, round(TX * (z1 - z0))), max(1, round(TX * (y1 - y0)))
    return w, h, (z1 - z0) / w, (y1 - y0) / h


def eye_center(side):
    """The eye's center on the skull's east (side > 0) or west side, in that face's texels."""
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    return ((z1 - EYE_Z) if side > 0 else (EYE_Z - z0)) / su, (y1 - EYE_Y) / sv


def eye_box(side):
    """The painted eye on the (scaled) skull's side, in units: z from, z to, top, bottom; rounded
    exactly as `stamp_eyes` places it, so the eyelid covers it to the texel."""
    pat = EYES[eye_style(side)]["pattern"]
    ph, pw = len(pat), len(pat[0])
    (x0, y0, z0), (x1, y1, z1) = SKULL
    w, h, su, sv = skull_grid()
    u0, v0 = eye_corner(*eye_center(side), pat, w)
    za, zb = (z1 - (u0 + pw) * su, z1 - u0 * su) if side > 0 else (z0 + u0 * su, z0 + (u0 + pw) * su)
    top = y1 - v0 * sv
    return za, zb, top, top - ph * sv


LEGS = ("FL", "FR", "BL", "BR")
FRONT_X, FRONT_Z, BACK_X, BACK_Z = 6.4, 21.0, 6.2, -30.0
# The neck carried forward and low (45 degrees), the head held out in front: a reindeer's way.
NECK_PITCH, NECK_LEN, NECK_W = -45.0, 30.0, 13.0  # a thick neck (Ronald: "a thicker neck rising to the head")
NECK0 = HEAD - (NECK_LEN - 2.5) * np.array([0, math.sin(math.radians(-NECK_PITCH)), math.cos(math.radians(-NECK_PITCH))])
NECK_ROT = (NECK_PITCH + 90, 0, 0)
EAR = [(70.5, 72.5, 1.8), (72.5, 76.5, 2.6), (76.5, 78.5, 1.7)]  # y0, y1, half width: short and round
EAR_X, EAR_Z = 5.4, (34.0, 35.6)
TAIL0 = np.array([0.0, 53.5, -40.0])  # on the rump's back face: a flap hinged on the surface
ANTLER0 = hs((3.6, 71.4, 36.5))  # where the left antler's burr sits on the skull


def antler(side):
    """Drenkaun's antler, like an oak bough: a beam rising up, out and back, then forward, in four
    lengths; a brow tine over the face, a bez and a trez tine rising from the beam, a fork at its
    end; moss hanging in drips from the beam and a few leaves where tines grow. Each antler is its
    own bone, so a break can take it (`bosses::Part` left_antler, right_antler)."""
    s = 1 if side == "L" else -1
    k = side.lower()
    bone = f"antler_{side}"
    x0, y0, z0 = ANTLER0
    start = (s * x0, y0, z0)
    E = [el(f"burr_{k}", bone, (s * x0 - 1.4, y0 - 1.0, z0 - 1.6), (s * x0 + 1.4, y0 + 2.6, z0 + 1.6))]
    # Wide like a lyre (concept A): out and back, then up, the tips curving in a little.
    beam0, p1 = rod(f"beam0_{k}", bone, (start[0], y0 + 2.0, z0), (-20, 0, -s * 76), 17.0, 3.4)
    beam1, p2 = rod(f"beam1_{k}", bone, tuple(p1), (-12, 0, -s * 58), 16.0, 3.0)
    beam2, p3 = rod(f"beam2_{k}", bone, tuple(p2), (2, 0, -s * 34), 15.0, 2.6)
    beam3, p4 = rod(f"beam3_{k}", bone, tuple(p3), (16, 0, -s * 8), 11.0, 2.1)
    E += [beam0, beam1, beam2, beam3]
    # Branchy as concept A: seven tines a side, the upper ones forking off the beam's outer side.
    tines = [("brow", np.array(start) + np.array([s * 0.6, 3.0, 0.8]), (70, 0, -s * 22), 12.0, 2.1),
             ("bez", p1 + (p2 - p1) * 0.1, (48, 0, -s * 18), 12.0, 2.0),
             ("trez", p1 + (p2 - p1) * 0.75, (34, 0, -s * 10), 13.0, 1.9),
             ("royal", p2 + (p3 - p2) * 0.45, (28, 0, s * 10), 11.0, 1.8),
             ("sur", p2 + (p3 - p2) * 0.9, (-4, 0, -s * 52), 9.0, 1.7),
             ("crown", p3 + (p4 - p3) * 0.55, (8, 0, s * 22), 9.0, 1.6),
             ("fork", p3 + (p4 - p3) * 0.5, (-12, 0, -s * 40), 9.5, 1.6)]
    for name, at, rot, length, w in tines:
        t, _ = rod(f"tine_{name}_{k}", bone, tuple(at), rot, length, w)
        E.append(t)
    # Moss hanging from the beam, in drips of different lengths, and leaves at the tines' roots.
    # Strands of moss hanging from the beams and tines, of different lengths (concept A).
    drips = ((p1, p2, 11.0, 0.3, 1.2), (p2, p3, 8.0, 0.6, 1.0), (p1, p2, 6.0, 0.8, 0.9), (p0 := np.array(start), p1, 7.0, 0.7, 1.1),
             (p3, p4, 6.5, 0.3, 0.9), (p2, p3, 4.5, 0.2, 0.8))
    for j, (q, f, ln, u, w) in enumerate(drips):
        at = q + (f - q) * u
        E.append(el(f"moss_drip{j}_{k}", bone, (at[0] - w / 2, at[1] - ln, at[2] - w / 2 + 0.13 * j),
                    (at[0] + w / 2, at[1] - 0.6, at[2] + w / 2 + 0.13 * j)))
    for j, (name, at, rot, length, w) in enumerate(tines[1:4]):
        E.append(el(f"leaf{j}_{k}", bone, (at[0] - 1.6, at[1] + 0.2, at[2] - 1.05), (at[0] + 1.6, at[1] + 0.9, at[2] + 1.55)))
    return E


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 48, -22)), ("spine_1", "spine_0", (0, 46, -7)),
         ("neck", "spine_1", tuple(NECK0)), ("head", "neck", tuple(HEAD)), ("jaw", "head", hs((0, 60.1, 45.0))),
         ("tail", "spine_0", tuple(TAIL0))]
    for s, n in ((1, "L"), (-1, "R")):
        B += [(f"ear_{n}", "head", hs((s * EAR_X, 71.0, sum(EAR_Z) / 2))), (f"antler_{n}", "head", (s * ANTLER0[0], ANTLER0[1], ANTLER0[2]))]
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x = FRONT_X * s
            B += [(f"leg_{leg}_upper", "spine_1", (x, 40, FRONT_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 21, FRONT_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 5.0, FRONT_Z))]
        else:
            x = BACK_X * s
            B += [(f"leg_{leg}_upper", "spine_0", (x, 49, BACK_Z)), (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, 29, BACK_Z)),
                  (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, 6.0, BACK_Z))]
    B = [(n, p, tuple(v * K for v in pv)) for n, p, pv in B]
    for s, n in ((1, "L"), (-1, "R")):  # the eyelids, placed on the scaled skull with the eye's rounding
        za, zb, top, _ = eye_box(s)
        B.append((f"eyelid_{n}", "head", (s * LID_X, top, (za + zb) / 2)))
    return B


def elements():
    # Higher at the withers than at the hips (Planner, relaying Ronald's round: "taller at the
    # shoulders than at the hips, not one box").
    E = [el("hips", "spine_0", (-10.6, 35, -40), (10.6, 55.0, 0)),  # reaching deep into the chest, below the withers
         el("chest", "spine_1", (-11.4, 30, -8), (11.4, 62.5, 30)),  # deep and broad, the withers high
         # Ronald: "a deeper chest and shoulders ... and a lower belly line at the back".
         el("brisket", "spine_1", (-9.6, 25.5, 12.0), (9.6, 34.0, 32.5)),
         # Under the shell the shoulders reach back over the chest, so the shell hugs one box.
         el("shoulders", "spine_1", (-12.8, 44.0, -8.5 if MOSS_STYLE == "shell" else 4.0), (12.8, 64.2, 26.5)),
         el("belly", "spine_0", (-9.2, 31.6, -31.0), (9.2, 38.0, -5.0)),
         el("skull", "head", *SKULL0),
         # A stag's muzzle, narrow and tapering to a dark nose (Ronald, round 4: the big flat pale
         # box read as a cow's).
         # Their bottoms flush with the skull's, a half unit apart (linecheck: a lower edge a texel
         # below the next box's read as a thin line under the face).
         el("muzzle", "head", hs((-4.4, 59.88, 45.5)), hs((4.4, 66.6, 53.0))),
         el("nose", "head", hs((-3.4, 59.78, 52.5)), hs((3.4, 65.2, 56.6))),
         el("mouth", "head", hs((-2.7, 59.0, 44.5)), hs((2.7, 60.6, 52.0))),
         el("jaw", "jaw", hs((-3.1, 57.9, 44.0)), hs((3.1, 60.3, 52.4)))]  # shorter than the muzzle
    neck, _ = rod("neck", "neck", tuple(NECK0), NECK_ROT, NECK_LEN, NECK_W, back=6.0)  # slimmer than the head
    E.append(neck)
    n0 = NECK0
    hw = NECK_W / 2
    # The moss mane (Ronald: "a mane that drapes over the neck and shoulders in uneven tufts"):
    # under the throat a long beard of locks; along the neck's crest tufts of different sizes; over
    # the withers and shoulders more tufts, their locks hanging down the sides unevenly.
    E.append(el("mane", "neck", (-5.6, n0[1] + 3, n0[2] + hw - 1.5), (5.6, n0[1] + 24, n0[2] + hw + 3.1), rot=NECK_ROT, origin=tuple(n0)))
    E.append(el("fringe_mane", "neck", (0, n0[1] + 3.5, n0[2] + hw), (0, n0[1] + 24.5, n0[2] + hw + 21.0), quad="east", rot=NECK_ROT,
                origin=tuple(n0), cut="mane"))
    for j, (t, ln, w, d) in enumerate(((2.0, 8.0, 1.6, 3.2), (8.5, 7.0, 2.3, 2.6), (14.5, 8.5, 1.3, 3.6), (21.5, 6.5, 1.42, 2.4))):
        # Each sunk a unit deeper into the neck than the one before: their inner ends two texels apart
        # (0.2 apart left a half-texel line along each tuft's end; linecheck).
        E.append(el(f"moss_crest{j}", "neck", (-hw - w, n0[1] + t, n0[2] - hw - d), (hw + w, n0[1] + t + ln, n0[2] - hw + 2.0 + 1.0 * j),
                    rot=NECK_ROT, origin=tuple(n0)))
    # It hangs down the front, as a mane (Ronald, round 2: "the moss hangs down the front of the
    # neck and chest as a mane instead of lying on top like a blanket"): a mat over the chest's
    # front under the throat, locks hanging from it down the brisket, and down the shoulders' fronts
    # only; on top just the tufts where the neck meets the withers.
    # The tuft over the withers two texels wider than the shoulders (0.8 left a strip of their front
    # beside it); the chest's mat reaching back into the chest, over the brisket's top (it left a
    # texel and a half of the brisket showing between them).
    if MOSS_STYLE == "shell":
        E.append(el("moss_shell", "spine_1", *SHELL, cut="shell"))
    else:
        E.append(el("moss_tuft0", "spine_1", (-13.9, 54.0, 19.5), (13.9, 65.4, 30.7)))
    E.append(el("moss_chest", "spine_1", (-10.4, 34.0, 29.6), (10.4, 58.0, 34.9)))
    E.append(el("fringe_chest", "spine_1", (-11.4, 18.0, 35.4), (11.4, 58.0, 35.4), quad="south", cut="chest"))
    for s_, q in ((1, "east"), (-1, "west")) if MOSS_STYLE != "shell" else ():
        E.append(el(f"fringe_moss_{q}", "spine_1", (s_ * 14.8, 30.0, -6.0), (s_ * 14.8, 64.6, 30.5), quad=q, cut="moss"))
    # No bark plates (Ronald, round 4: "Drop the plates"): the moss mane carries on over the
    # shoulders and along the back in uneven tufts, each narrower and lower than the one before,
    # thinning toward the hips and stopping short of the rump (concept A).
    # The first pad lies over the shoulders' edges (at 12.0 their top showed beside it as a texel and
    # a half of brown: Ronald, 11-05, "a thin line there, which makes no sense"); the others are
    # clearly narrower than the back they lie on.
    for j, (bone_, x, y0, y1, z0, z1) in enumerate((("spine_1", 12.93, 61.0, 66.2, 8.5, 20.2), ("spine_1", 10.2, 60.6, 65.4, -1.5, 9.2),
                                                     ("spine_1", 8.0, 59.9, 63.9, -9.5, -0.8), ("spine_0", 6.4, 53.6, 57.6, -18.0, -8.6),
                                                     ("spine_0", 4.4, 53.9, 56.5, -26.0, -17.4)) if MOSS_STYLE != "shell" else ()):
        E.append(el(f"moss_back{j}", bone_, (-x, y0, z0), (x, y1, z1)))
    E.append(el("tail", "tail", (-2.6, 45.5, -42.8), (2.6, 54.5, -39.75), rot=(12, 0, 0), origin=tuple(TAIL0)))
    for s, n in ((1, "L"), (-1, "R")):
        cx = EAR_X * s
        rot, origin = (-14, 0, -s * 45), (cx, 71.0, sum(EAR_Z) / 2)
        ears = [el(f"ear_core_{n.lower()}{k}", f"ear_{n}", (cx - hw_, y0, EAR_Z[0]), (cx + hw_, y1, EAR_Z[1]), rot=rot, origin=origin)
                for k, (y0, y1, hw_) in enumerate(EAR)]
        zc = sum(EAR_Z) / 2
        ears.append(el(f"fringe_ear_{n.lower()}", f"ear_{n}", (cx - 3.1, 70, zc), (cx + 3.1, 79.6, zc), quad="south", rot=rot,
                       origin=origin, cut="ear"))
        for e in ears:  # the ears go with the bigger head
            e["lo"], e["hi"], e["origin"] = list(hs(e["lo"])), list(hs(e["hi"])), list(hs(e["origin"]))
        E += ears
        E += antler(n)
    # Legs (Ronald: "thicker, with knees and hocks, taper to the hooves"): each a thick upper part,
    # a knee or hock where it bends, a slimmer shank, a broad dark hoof.
    for leg in LEGS:
        s = 1 if leg[1] == "L" else -1
        if leg[0] == "F":
            x, z = FRONT_X * s, FRONT_Z
            E.append(el(f"leg_{leg}_upper", f"leg_{leg}_upper", (x - 4.4, 18.5, z - 4.4), (x + 4.4, 44.6, z + 4.6)))
            E.append(el(f"knee_{leg}", f"leg_{leg}_lower", (x - 3.7, 17.0, z - 3.6), (x + 3.7, 23.5, z + 3.9)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 3.0, 4.6, z - 3.0), (x + 3.0, 20.0, z + 3.0)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 3.4, 0, z - 3.3), (x + 3.4, 5.4, z + 3.9)))  # the broad hoof
        else:
            x, z = BACK_X * s, BACK_Z
            # The ham: two texels out from the hips' side, a haunch (4.6 left 0.4 of a texel poking out).
            xi, xo = sorted((x - 4.6 * s, x + 5.5 * s))
            # Its top two texels under the hips' back (at 55.5 it stood a texel above it: a lip; linecheck).
            E.append(el(f"thigh_{leg}", f"leg_{leg}_upper", (xi, 27.5, z - 7.0), (xo, 54.0, z + 6.5)))
            E.append(el(f"hock_{leg}", f"leg_{leg}_lower", (x - 3.5, 24.5, z - 3.4), (x + 3.5, 31.5, z + 3.6)))
            E.append(el(f"leg_{leg}_lower", f"leg_{leg}_lower", (x - 2.9, 5.4, z - 2.9), (x + 2.9, 26.0, z + 2.9)))
            E.append(el(f"leg_{leg}_paw", f"leg_{leg}_paw", (x - 3.4, 0, z - 3.2), (x + 3.4, 6.0, z + 3.8)))
    for e in E:  # the deer's space, drawn K times bigger
        e["lo"] = [v * K for v in e["lo"]]
        e["hi"] = [v * K for v in e["hi"]]
        if e["origin"]:
            e["origin"] = [v * K for v in e["origin"]]
    for s, n in ((1, "L"), (-1, "R")):  # the eyelids, on the scaled skull
        za, zb, top, bottom = eye_box(s)
        h = top - bottom
        lx0, lx1 = (LID_X - h, LID_X) if s > 0 else (-LID_X, -LID_X + h)
        E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, za), (lx1, top, zb), quad="down"))
    return E


# ------------------------------------------------------------------ painting

FACES = {"north": (2, -1), "south": (2, 1), "west": (0, -1), "east": (0, 1), "down": (1, -1), "up": (1, 1)}


def face_frame(lo, hi, face):
    lo, hi = np.array(lo), np.array(hi)
    X, Y, Z = np.eye(3)
    corner, du, dv = {"south": ((lo[0], hi[1], hi[2]), X, -Y), "north": ((hi[0], hi[1], lo[2]), -X, -Y),
                      "east": ((hi[0], hi[1], hi[2]), -Z, -Y), "west": ((lo[0], hi[1], lo[2]), Z, -Y),
                      "up": ((lo[0], hi[1], lo[2]), X, Z), "down": ((lo[0], lo[1], hi[2]), X, -Z)}[face]
    size = hi - lo
    ax, sgn = FACES[face]
    n = np.zeros(3)
    n[ax] = sgn
    # At TX texels a unit; never under 1 (an antler's 1-unit point is a texel, not nothing).
    return np.array(corner, float), du, dv, max(1, int(round(TX * abs(size @ du)))), max(1, int(round(TX * abs(size @ dv)))), n


ANTLER_PARTS = ("burr_", "beam", "tine_", "moss_drip", "leaf")


def local_y(e, p):
    """How far along a rod a texel is, as a share of its length (0 at its start)."""
    o = np.array(e["origin"])
    loc = (p - o) @ rotm(e["rot"])
    return (loc[..., 1] - (e["lo"][1] - o[1])) / max(e["hi"][1] - e["lo"][1], 1e-6)


def area(e, face, p, uu, vv, h):
    """Concept A's colors, in the deer's space (`p` / K): a warm mid-brown coat, darker along the
    spine; cream-tan under the belly, the throat, the muzzle and the tail; dark brown legs and
    hooves; the moss cape and mane, its locks a shade darker toward their tips; grey-brown bark;
    bark-brown antlers to bone-pale tips, moss drips and leaves on them."""
    name = e["name"]
    p = p / K
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(x.shape, FUR)
    side = face in ("east", "west")
    t = vv / max(h, 1)
    if name in ("brisket", "shoulders", "belly"):
        a[:] = FUR
        if face == "down":
            a[:] = BELLY
        elif name == "belly" and face != "up":
            a = np.where(y < 34.5, BELLY, FUR)
        elif name == "brisket" and face == "south":
            a = np.where(y < 29.0, BELLY, FUR)
    elif name in ("hips", "chest"):
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = BELLY
        else:
            a = np.where(t < 0.12, BACK, np.where(y < 34.5, BELLY, FUR))
            if name == "chest" and face == "south":
                a = np.where(y > 36.0, FUR, BELLY)  # behind the moss mane's locks
            if name == "hips" and face == "north":
                a = np.where((np.abs(x) < 6.0) & (y > 40), BELLY, a)  # the pale rump round the tail
    elif name in ("mane", "fringe_mane", "moss_chest", "fringe_chest") or name.startswith(("fringe_moss", "moss_crest", "moss_tuft", "moss_back", "moss_shell")):
        a[:] = MOSS
        if e["quad"]:
            if not e["rot"]:
                a = np.where(y < 44.0, MOSSDK, MOSS)  # the locks darker toward their tips
        elif face == "down":
            a[:] = MOSSDK
        elif face != "up":  # tufts: darker below, in uneven tongues
            tongue = 0.35 + 0.25 * np.sin(uu * 1.7 + len(name)) + 0.15 * np.sin(uu * 0.6)
            a = np.where(t > 1 - tongue * 0.6, MOSSDK, MOSS)
    elif name == "neck":
        a[:] = FUR
        if face in ("north", "up"):
            a[:] = MOSS  # the mane runs over the neck's top
        elif face == "south" or face == "down":
            a[:] = PALE
    elif name.startswith("bark") or name == "crown":
        # Brown-grey bark: long grooves running along the shingle (front to back), each its own
        # length, lighter worn ridges between; moss creeping along its lower and back edges.
        a[:] = BARK
        seed = sum(map(ord, name))
        if e["quad"] is None:
            along_u = face in ("up", "down")  # on the top the grooves run down the face's v; on a side along u
            col = (uu if along_u else vv).astype(int)
            run = vv if along_u else uu
            span = run.max() + 1
            r1 = (np.sin(col * 12.9898 + seed) * 43758.5453) % 1.0
            r2 = (np.sin(col * 78.233 + seed * 3) * 12345.678) % 1.0
            groove = (r1 < 0.2) & (np.abs(run - r2 * span) < span * 0.3)
            a = np.where(groove, BARKDK, np.where(r1 > 0.82, BARKLT, BARK))
            if face != "down":
                # Moss along its lower and back edges, in a wavering band two or three texels deep.
                depth = 2.2 + 1.2 * np.sin(uu * 0.9 + seed) + 0.8 * np.sin(vv * 1.7)
                if face == "up":
                    edge = (uu > uu.max() - depth) | (vv > vv.max() - depth * 0.7)
                else:
                    edge = (vv > vv.max() - depth) | ((uu < depth * 0.6) & (face == "north"))
                a = np.where(edge, MOSS, a)
    elif name == "skull":
        if face == "up":
            a[:] = BACK
        elif face == "down":
            a[:] = PALE
        elif side:
            a = np.where(z > hs((0, 0, 44.0))[2], MUZZLE, FUR)
        elif face == "north":
            a[:] = FUR
    elif name == "muzzle":
        a[:] = MUZZLE
        if face == "up":
            a[:] = FUR
        elif face == "down":
            a[:] = PALE
    elif name == "nose":
        a[:] = NOSE  # dark to the lip; the nostrils are soft slits (`paint_face`)
        if face == "down":
            a[:] = MUZZLE
    elif name == "jaw":
        a[:] = MOUTH if face == "up" else PALE
    elif name == "mouth":
        a[:] = MOUTH
    elif name.startswith(("moss_drip", "leaf")):
        a[:] = MOSS if name.startswith("leaf") else MOSSDK
    elif name.startswith(("burr_", "beam", "tine_")):
        a[:] = ANTLER
        if name.startswith("tine_") or name.startswith("beam3"):
            a = np.where(local_y(e, p * K) > 0.72, BONE, ANTLER)
    elif name == "tail":
        a[:] = TAILTOP
    elif name.startswith("ear_core"):
        a[:] = FUR
        if face == "south":
            cx = e["origin"][0] / K
            a = np.where(np.abs(x - cx) < 1.7, EARIN, np.where(np.abs(x - cx) < 2.3, FUR, DARK))
    elif name.startswith(("knee_", "hock_")):
        a[:] = LEG
    elif name.startswith("thigh_"):
        a[:] = FUR
        if face == "up":
            a[:] = BACK
        a = np.where(y < 36, LEG, a)
    elif name.startswith("leg_"):
        a[:] = LEG
        if name.endswith("_paw"):
            a = np.where(y < 3.0, HOOF, LEG)
            if face == "down":
                a[:] = HOOF
    return a


def cut_mask(e, p):
    """Cut-out outlines: the ear's short oval, the mane's locks (in the neck's own frame)."""
    if e["cut"] == "chest":  # the mane's locks down the chest, each its own length, longest mid-chest
        x, y = p[..., 0] / K, p[..., 1] / K
        period = max(2.0, 3 / (TX * K))
        idx = np.floor(x / period)
        rnd = (np.sin(idx * 12.9898 + 3.1) * 43758.5453) % 1.0
        lock = np.abs(((x / period) % 1.0) - 0.5) * 2
        tip = 22.0 + 7.0 * (np.abs(x) / 11.4) ** 1.5 + 9.0 * rnd + 2.0 * lock
        return y >= tip
    if e["cut"] == "shell":  # the shell's hem: ragged locks, longest over the shoulders' front
        x, y, z = p[..., 0] / K, p[..., 1] / K, p[..., 2] / K
        (x0, y0, z0), (x1, y1, z1) = SHELL
        period = max(2.0, 3 / (TX * K))
        on_side = np.abs(np.abs(x) - x1) < 0.05
        on_back = np.abs(z - z0) < 0.05
        on_down = np.abs(y - y0) < 0.05
        along = np.where(on_side, z, x)  # the locks run along the hem
        lock = np.abs(((along / period) % 1.0) - 0.5) * 2
        rnd = (np.sin(np.floor(along / period) * 12.9898 + 7.0 * on_back + 3.0 * np.sign(x)) * 43758.5453) % 1.0
        u = np.clip((z - z0) / (z1 - z0), 0, 1)  # 0 at the back, 1 at the front
        tip_side = 57.0 - 11.0 * u ** 0.7 + 2.0 * lock + 3.5 * rnd
        tip_back = 57.5 + 2.0 * lock + 3.0 * rnd
        keep = np.where(on_side, y >= tip_side, np.where(on_back, y >= tip_back, True))
        return keep & ~on_down
    if e["cut"] == "moss":  # locks hanging down the shoulders, uneven, shorter toward the back
        y, z = p[..., 1] / K, p[..., 2] / K
        u = np.clip((z - 14.0) / 16.5, 0, 1)
        back = np.clip((14.0 - z) / 20.0, 0, 1)  # 0 at the shoulder, 1 at mid-back
        period = max(2.0, 3 / (TX * K))
        lock = np.abs(((z / period) % 1.0) - 0.5) * 2
        idx = np.floor(z / period)
        rnd = (np.sin(idx * 12.9898 + e["lo"][0]) * 43758.5453) % 1.0  # each lock its own length: uneven tufts
        tip = 46.0 - 12.0 * np.sin(math.pi * u) ** 0.8 + 3.0 * lock + 7.0 * rnd + 14.0 * back ** 0.8  # shaggy, longest over the shoulder
        return y >= tip
    if e["cut"] == "ear":
        x, y = p[..., 0] / K, p[..., 1] / K
        cx = e["origin"][0] / K
        t = (y - 74.6) / 4.6
        half = 2.9 * np.sqrt(np.clip(1 - t ** 2, 0, 1)) + 0.3 * ((np.floor(y * 1.3) % 4) == 0) * (np.abs(t) < 0.8)
        return (np.abs(x - cx) <= half) & (y >= 70)
    o = np.array(e["origin"])
    local = (p - o) @ rotm(e["rot"]) / K  # back into the neck's frame, the deer's space
    along, depth = local[..., 1], local[..., 2]
    u = np.clip((along - 3.5) / 21.0, 0, 1)
    period = max(2.5, 3 / (TX * K))  # a lock every 3 texels or more (finer ones alias at 32)
    lock = np.abs(((along / period) % 1.0) - 0.5) * 2  # points every few texels
    tip = 12.0 + 10.0 * np.sin(math.pi * u) ** 0.7 - max(1.6, 1 / (TX * K)) * lock  # notches a texel deep
    return depth <= tip


LASH = (40, 30, 28)

def paint_face(e, face, boxes):
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face)
    size = np.array(e["hi"]) - np.array(e["lo"])
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)  # texel centers, in texels
    su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel
    p = corner + (uu * su)[..., None] * du + (vv * sv)[..., None] * dv
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    light_face = face
    if e["name"].startswith("eyelid"):
        light_face = "east"  # closed, it is the head's side
    if face == "down" and e["name"].startswith(("leg_", "thigh_", "jaw", "tail", "mane")):
        light_face = "north"
    a = area(e, face, p, uu, vv, h)
    occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    stacked = e["name"].startswith("ear_core")
    down = np.clip((79.0 * K - p[..., 1]) / (8.5 * K), 0, 1) if stacked else None
    cap = {}
    col = paint_face_smooth(a, TONES, light_face, out=cap, coat=COAT,
                            y_world=p[..., 1] if not e["name"].startswith(ANTLER_PARTS) else None,
                            occ=occ, quad=bool(e["quad"]), seed=sum(map(ord, e["name"] + face)) % 97, down=down,
                            tuft=e["name"] not in ("skull", "muzzle", "nose") and not e["name"].startswith(("bark", "crown")),
                            merge=e["name"] != "skull" and not e["name"].startswith(("bark", "crown")))  # keep the eye ring
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"]:
        img[..., 3] = np.where(cut_mask(e, p), 255, 0)
    if e["name"] == "skull" and face in ("east", "west"):
        s = 1 if face == "east" else -1
        stamp_eyes(img, [eye_center(s)], eye_style(s))
    if e["name"] == "nose" and face == "south":
        # Nostrils as short soft slits low on the nose, the lip a line below them: never two dark
        # marks high on the face, which read as a second pair of eyes (the skill's face rule).
        r = max(1, int(h * 0.62))
        soft = (52, 36, 28)
        if w >= 10:
            for c0 in (int(w * 0.18), int(w * 0.70)):
                img[r, c0:c0 + max(2, w // 8), :3] = soft
            img[h - 2, w // 2 - 1:w // 2 + 1, :3] = (img[h - 2, w // 2 - 1:w // 2 + 1, :3] * 0.8).astype(np.uint8)
    if e["name"].startswith("eyelid"):
        col_idx = 0 if e["name"].endswith("L") else w - 1
        img[:, col_idx, :3] = LASH
    if e["name"].endswith("_paw") and face == "south" and w >= 3 and w % 2:  # the cloven hoof: a split line, on a middle texel
        c = w // 2
        img[:, c, :3] = (img[:, c, :3] * 0.6).astype(np.uint8)
    return img


# ------------------------------------------------------------------ animation

L = {"lie_front": 80, "lie_hip": -70, "lie_knee": 140, "lie_drop": 30, "graze_neck": 40, "graze_head": 30, "graze_lean": 4, "graze_splay": 0}


# Standing, its legs bend where a deer's do (Ronald's round: "the legs bend visibly at knee and
# hock"): the hind leg's thigh back and its cannon forward, so the hock points back; the foreleg a
# little at the knee; each hoof kept flat (the angles of a leg sum to 0). The drop it takes to
# keep the hooves on the ground is fitted (`fit_poses`, `stance_drop`).
STANCE = {"F": (-10.0, 18.0, -8.0), "B": (10.0, -17.0, 7.0)}  # fitted so both pairs of hooves stand level


def with_stance(q):
    """`q` with the standing bend added to each leg's three bones."""
    q = dict(q)
    for leg in LEGS:
        for part, a in zip(("upper", "lower", "paw"), STANCE[leg[0]]):
            k = f"leg_{leg}_{part}"
            x, y, z = q.get(k, (0, 0, 0))
            q[k] = (x + a, y, z)
    return q


def gait(p, phases, swing_amp, lift):
    """A foot swings forward while it's lifted, back while it's down."""
    out = {}
    for leg, ph in zip(LEGS, phases):
        a = TAU * (p + ph)
        up = max(0.0, S(a + math.pi / 2))
        out[f"leg_{leg}_upper"] = (-swing_amp * S(a), 0, 0)
        knee = lift * up
        if leg[0] == "F":
            out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (knee, 0, 0), (-0.5 * knee, 0, 0)
        else:
            out[f"leg_{leg}_lower"], out[f"leg_{leg}_paw"] = (-knee, 0, 0), (0.6 * knee, 0, 0)
    return with_stance(out)


def chew(t, rate=2.5, amp=7.0):
    return {"jaw": (amp * max(0.0, S(TAU * t * rate)), 0, 0)}


def graze_pose(g, P=L):
    """Neck and head down toward the lichen by `g` (0..1); the front legs straddled forward a
    little to lower the shoulders, the hooves kept flat on the ground."""
    sp = P["graze_splay"] * g
    q = {"spine_1": (P["graze_lean"] * g, 0, 0), "neck": (P["graze_neck"] * g, 0, 0), "head": (P["graze_head"] * g, 0, 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (-sp - P["graze_lean"] * g, 0, 0)
        q[f"leg_{leg}_paw"] = (sp, 0, 0)
    return with_stance(q)


HEAD_UP = (-24, 0, 0), (20, 0, 0)  # neck raised, head held level: a stag's carriage (concept A)


def stand(q=None):
    """Its standing carriage under every clip: the neck raised, the head level."""
    q = dict(q or {})
    q.setdefault("neck", HEAD_UP[0])
    q.setdefault("head", HEAD_UP[1])
    if not any(k.startswith("leg_") for k in q):
        q = with_stance(q)
    return q


def pose_idle(t):
    """Standing guard: breathing deep, looking slowly about, an ear and a tail flick (8 s)."""
    p = t / 8.0
    br = S(TAU * p * 2)
    look = 18 * S(TAU * p)
    fl = max(0.0, S(TAU * (p - 0.3) * 12)) if 0.3 < p < 0.34 else 0.0
    tf = max(0.0, S(TAU * (p - 0.7) * 10)) if 0.7 < p < 0.75 else 0.0
    return stand({"root": (0, 0.6 * br, 0), "spine_1": (0.6 * br, 0, 0), "neck": (HEAD_UP[0][0] + 2 * br, 0.3 * look, 0),
                  "head": (HEAD_UP[1][0], 0.7 * look, 0), "ear_L": (0, 0, -15 * fl), "ear_R": (0, 0, 2 * br),
                  "tail": (-25 * tf, 0, 0)})


def pose_walk(t):
    """The long-striding walk, the head nodding low (1.5 s)."""
    p = t / 1.5
    q = gait(p, (0.0, 0.5, 0.75, 0.25), 20, 36)
    q.update({"root": (0, 0.6 * abs(S(TAU * p * 2)), 0), "neck": (HEAD_UP[0][0] + 3 * S(TAU * p * 2), 0, 0),
              "head": (HEAD_UP[1][0] - 3 * S(TAU * p * 2), 0, 0), "tail": (-4, 0, 0)})
    return q


def pose_run(t):
    """The gallop, the head out low and the short tail up (0.55 s)."""
    p = t / 0.55
    q = gait(p, (0.0, 0.1, 0.5, 0.6), 32, 60)
    c = S(TAU * p)
    q.update({"root": (0, 2.4 * max(0.0, S(TAU * p + 0.6)), 0), "spine_0": (-4 * c, 0, 0), "spine_1": (6 * c, 0, 0),
              "neck": (6 - 5 * c, 0, 0), "head": (-4, 0, 0), "ear_L": (-20, 0, 8), "ear_R": (-20, 0, -8),
              "tail": (-40, 0, 0)})
    return q


def pose_lie(t, P=L):
    """Lying down, the legs folded under, the head up and chewing the cud (6 s)."""
    p = t / 6.0
    br = S(TAU * p * 2)
    q = {"root": (0, -P["lie_drop"] + 0.15 * br, 0), "spine_1": (0.5 * br, 0, 0), "head": (0, 8 * S(TAU * p), 0)}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (P["lie_front"], 0, 0)
        q[f"leg_{leg}_lower"] = (-150, 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_front"] + 150 - 40, 0, 0)
    for leg, s in (("BL", 1), ("BR", -1)):
        q[f"leg_{leg}_upper"] = (P["lie_hip"], 0, s * 5)
        q[f"leg_{leg}_lower"] = (P["lie_knee"], 0, 0)
        q[f"leg_{leg}_paw"] = (-P["lie_hip"] - P["lie_knee"], 0, 0)
    q.update(chew(t, 1.0, 4.0))
    return q


def ease(p, a, b):
    """0 before `a`, 1 after `b`, smooth between."""
    u = min(1.0, max(0.0, (p - a) / max(b - a, 1e-6)))
    return u * u * (3 - 2 * u)


def pose_rise(t):
    """Out of the ground (3 s): sunk to its back, the body pitched up as it climbs."""
    p = t / 3.0
    u = ease(p, 0.0, 0.85)
    q = stand({"root": (0, -62.0 * K * (1 - u), 0), "spine_0": (-14 * (1 - u), 0, 0)})
    q["neck"] = (HEAD_UP[0][0] - 10 * S(math.pi * u), 0, 0)
    return q


def pose_charge_tell(t):
    """Before the charge (1.2 s): the head down, antlers forward, a forehoof scraping twice."""
    p = t / 1.2
    d = ease(p, 0.0, 0.3)
    scrape = S(TAU * p * 2) if p < 0.9 else 0.0
    return stand({"neck": (HEAD_UP[0][0] + 45 * d, 0, 0), "head": (HEAD_UP[1][0] + 21 * d, 0, 0), "spine_1": (4 * d, 0, 0),
                  "leg_FR_upper": (-23 * max(0.0, scrape), 0, 0), "leg_FR_lower": (30 * max(0.0, scrape), 0, 0),
                  "tail": (-30 * d, 0, 0)})


def pose_charge(t):
    """The charge (2.5 s): a gallop with the head down and the antlers levelled."""
    q = pose_run(t % 0.55)
    q.update({"neck": (HEAD_UP[0][0] + 48, 0, 0), "head": (HEAD_UP[1][0] + 16, 0, 0), "tail": (-45, 0, 0)})
    return q


def pose_sweep_tell(t):
    """Before the sweep (0.8 s): the antlers drawn back to its right, the weight on its left."""
    d = ease(t / 0.8, 0.0, 0.7)
    return stand({"neck": (HEAD_UP[0][0] + 20 * d, -32 * d, 0), "head": (HEAD_UP[1][0] + 10 * d, -10 * d, -8 * d),
                  "spine_1": (0, -3 * d, 0)})


def pose_sweep(t):
    """The sweep (0.75 s): the antlers swung across from its right to its left, then back."""
    p = t / 0.75
    sw = ease(p, 0.0, 0.35) - ease(p, 0.55, 1.0) * 0.6
    yaw = -32 + 74 * sw  # mostly the neck's: the head and chest turn a little with it (no neck through the skull)
    low = 20 * (1 - ease(p, 0.7, 1.0))
    return stand({"neck": (HEAD_UP[0][0] + low, yaw, 0), "head": (HEAD_UP[1][0] + 10, 0.3 * yaw, 0.25 * yaw), "spine_1": (0, 0.09 * yaw, 0)})


def front_up(r):
    """The front legs drawn up while it rears by `r` (0..1)."""
    q = {}
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (-55 * r, 0, 0)
        q[f"leg_{leg}_lower"] = (95 * r, 0, 0)
    return q


def pose_stomp_tell(t):
    """Before the stomp (0.9 s): rearing on its hind legs."""
    r = ease(t / 0.9, 0.0, 0.8)
    q = stand({"spine_0": (-22 * r, 0, 0), "neck": (HEAD_UP[0][0] - 8 * r, 0, 0), "tail": (-20 * r, 0, 0)})
    q.update(front_up(r))
    for leg in ("BL", "BR"):  # the hind legs stay planted as the body tips back over them
        q[f"leg_{leg}_upper"] = (STANCE["B"][0] + 22 * r, 0, 0)
    return q


def pose_stomp(t):
    """The stomp (0.6 s): down hard on both forefeet, the head thrown down, then up."""
    p = t / 0.6
    r = 1 - ease(p, 0.0, 0.3)
    q = stand({"spine_0": (-22 * r, 0, 0), "neck": (HEAD_UP[0][0] + 25 * (1 - r) * (1 - ease(p, 0.6, 1.0)), 0, 0),
               "root": (0, -2.0 * K * (1 - r) * (1 - ease(p, 0.4, 1.0)), 0)})
    q.update(front_up(r))
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (STANCE["B"][0] + 22 * r, 0, 0)
    return q


def pose_roots_tell(t, P=L):
    """Before the roots (1.4 s): the head to the ground, as if listening to it."""
    g = ease(t / 1.4, 0.0, 0.5)
    q = graze_pose(g, P)
    q["head"] = (q["head"][0], 8 * S(TAU * t / 1.4 * 2) * g, 0)
    return q


def pose_roots(t, P=L):
    """The roots called (0.8 s): the head thrown up from the ground."""
    p = t / 0.8
    g = 1 - ease(p, 0.0, 0.35)
    q = graze_pose(g, P)
    up = S(math.pi * ease(p, 0.2, 1.0))
    q["neck"] = (q["neck"][0] - 20 * up, 0, 0)
    q["head"] = (q["head"][0] - 15 * up, 0, 0)
    return stand(q)


def pose_stunned(t):
    """Stunned after charging into something (2 s, looped): the head low, swaying."""
    p = t / 2.0
    sw = S(TAU * p)
    return stand({"neck": (HEAD_UP[0][0] + 38, 10 * sw, 0), "head": (HEAD_UP[1][0] + 10, 12 * sw, 8 * sw),
                  "spine_1": (0, 0, 2.5 * sw), "root": (0, -0.8 * K * abs(sw), 0)})


def pose_down(t, P=L):
    """Knocked down (4 s, looped): on its belly, the legs folded, the head to the ground, heaving."""
    q = pose_lie(t * 1.5 % 6.0, P)
    br = S(TAU * t / 2.0)
    q.update({"neck": (35 + 2 * br, 0, 0), "head": (20, 0, 0), "spine_1": (1.5 * br, 0, 0), "jaw": (6, 0, 0)})
    return q


BLINKS = {"idle": [2.0, 5.9]}
BLINK = 0.12
ANIMS = {"idle": (pose_idle, 8.0), "walk": (pose_walk, 1.5), "run": (pose_run, 0.55), "lie": (pose_lie, 6.0),
         "rise": (pose_rise, 3.0), "charge_tell": (pose_charge_tell, 1.2), "charge": (pose_charge, 2.5),
         "sweep_tell": (pose_sweep_tell, 0.8), "sweep": (pose_sweep, 0.75), "stomp_tell": (pose_stomp_tell, 0.9),
         "stomp": (pose_stomp, 0.6), "roots_tell": (pose_roots_tell, 1.4), "roots": (pose_roots, 0.8),
         "stunned": (pose_stunned, 2.0), "down": (pose_down, 4.0)}
# One-off clips play once (an action that loops would play on: `drenkaun.rs` asks for each by name).
LOOPED = ("idle", "walk", "run", "lie", "stunned", "down")
CLIPS = [n for n in ANIMS if n not in ("idle", "walk", "run", "lie")]


def eyelids(name, t):
    a = 90 * max([0.0] + [1 - abs(t - b) / BLINK for b in BLINKS.get(name, [])])
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)}


def frame_times(name, dur):
    n = {"idle": 96, "lie": 60, "walk": 24, "run": 20, "charge": 100, "down": 60, "stunned": 40}.get(name, 30)
    ts = {round(dur * k / n, 4) for k in range(n + 1)}
    for b in BLINKS.get(name, []):
        ts |= {round(b + d, 4) for d in (-BLINK, -BLINK / 2, 0, BLINK / 2, BLINK)}
    return sorted(ts)


def animations(bone_names):
    out = {}
    for name, (fn, dur) in ANIMS.items():
        frames = []
        for t in frame_times(name, dur):
            tt = t % dur if t < dur else 0.0
            pose = fn(tt)
            pose.update(eyelids(name, tt))
            root = list(pose.get("root", (0, 0, 0)))
            if name not in ("lie", "down"):  # standing on bent legs, the hooves on the ground
                root[1] -= L.get("stance_drop", 0.0)
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in root]})
        out[name] = {"length": dur, "frames": frames, "loop": name in LOOPED}
    return out


# ------------------------------------------------------------------ fitting poses to the ground

GROUPS = {"front paws": lambda n: n in ("leg_FL_paw", "leg_FR_paw"), "back paws": lambda n: n in ("leg_BL_paw", "leg_BR_paw"),
          "body": lambda n: n in ("hips", "chest", "brisket", "belly"), "muzzle": lambda n: n in ("nose", "jaw"),
          "legs": lambda n: n.startswith(("leg_", "thigh_", "knee_", "hock_")), "antlers": lambda n: n.startswith(ANTLER_PARTS),
          "mane": lambda n: n in ("mane", "fringe_mane")}


def fit_poses(spec):
    """Standing on its bent legs (the drop that keeps the hooves on the ground), lying (the belly
    on the ground, the folded legs under it, nothing below) and grazing (the mouth at the ground),
    by forward kinematics."""
    g = lowest(spec, stand(), GROUPS)
    L["stance_drop"] = round(float(min(g["front paws"], g["back paws"])), 2)
    print("standing, lowest:", {k: round(v, 2) for k, v in g.items()})
    def lie_cost(P):
        g = lowest(spec, pose_lie(0.0, dict(L, **P, lie_drop=0)), GROUPS)
        f = min(v for k, v in g.items() if k != "mane")
        return (g["body"] - f - 0.8) ** 2 + 0.3 * (g["front paws"] - f) ** 2 + 0.3 * (g["back paws"] - f) ** 2
    grid = {"lie_front": list(range(60, 101, 5)), "lie_hip": list(range(-100, -49, 5)), "lie_knee": list(range(115, 166, 5))}
    best = fit(grid, lie_cost)[1]
    g = lowest(spec, pose_lie(0.0, dict(L, **best, lie_drop=0)), GROUPS)
    L.update(best, lie_drop=round(float(min(g.values())), 2))

    def graze_cost(P):
        q = graze_pose(1.0, dict(L, **P))
        g = lowest(spec, q, GROUPS)
        # The mouth at the ground, the antlers and the mane's locks clear of it.
        return (g["muzzle"] - 0.5) ** 2 + 0.002 * (P["graze_neck"] ** 2 + P["graze_head"] ** 2) + 0.05 * P["graze_lean"] ** 2 \
            + 20 * max(0.0, 1.0 - g["antlers"]) ** 2 + 20 * max(0.0, 0.5 - g["mane"]) ** 2 + 40 * (g["front paws"] - g["back paws"]) ** 2 + 60 * min(0.0, g["front paws"]) ** 2
    best = fit({"graze_neck": list(range(30, 126, 5)), "graze_head": list(range(-45, 61, 5)), "graze_lean": [0, 4, 8, 12, 16],
                "graze_splay": [0, 6, 12, 18]}, graze_cost)[1]
    L.update(best)
    gl = lowest(spec, graze_pose(1.0), GROUPS)
    print("grazing, lowest:", {k: round(v, 2) for k, v in gl.items()})
    ll = lowest(spec, pose_lie(0.0), GROUPS)
    print("lying, lowest:", {k: round(v, 2) for k, v in ll.items()})


# ------------------------------------------------------------------ build


def build():
    B = bones()
    E = elements()
    # A shell with a cut-out hem casts no occlusion: what shows through its hem is lit as open fur.
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"] and not e["rot"] and e["cut"] != "shell"]
    tiles = []
    for i, e in enumerate(E):
        for f in ([e["quad"]] if e["quad"] else list(FACES)):
            tiles.append(((i, f), paint_face(e, f, boxes)))
    sheet, uv = pack(tiles, 1024)
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    spec = {"name": "drenkaun", "texture": [sheet.shape[1], sheet.shape[0]],
            "bones": [{"name": n, "parent": p, "pivot": [float(v) for v in pv]} for n, p, pv in B], "elements": E}
    fit_poses(spec)
    spec["animations"] = animations([n for n, _, _ in B])
    spec["fitted"] = dict(L)
    return spec, sheet


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    spec, sheet = build()
    Image.fromarray(sheet, "RGBA").save(os.path.join(OUT, "texture.png"))
    with open(os.path.join(OUT, "spec.json"), "w") as f:
        json.dump(spec, f)
    print(len(spec["elements"]), "elements;", len(spec["bones"]), "bones; texture", sheet.shape[1], "x", sheet.shape[0],
          "; fitted", spec["fitted"])
