#!/usr/bin/env python3
"""The wyvern: a huge winged dragon in the style of the show dragons (Game of Thrones), our own
design, built with the blocky-model skill in the house style (64 units a meter, painted at 32
texels a meter, `TX`; see README.md for the research and the decisions).

Two hind legs and two wings that are its arms, as a bat's: an upper arm, a forearm, a clawed thumb
it walks on, and three long fingers carrying the membrane (two panels between the fingers, one
from the last finger back to the body). Long neck, horned head with a jaw, long spiked tail.
About 4 m tall standing, 16 m long, about 17 m across the wings.

The rest pose (what the boxes describe) has the wings spread flat and the legs straight; every
other pose is rotations of the bones (`poses`).

    python wyvern.py      writes out/spec.json and out/texture.png (build.py exports them)
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
from paint import EYES  # noqa: E402
from prop import FACES, el, rotm  # noqa: E402
from rig import euler  # noqa: E402
from house import TX  # noqa: E402

# Painted at the house density, 32 texels a block (TX = 0.5 texels a unit): the face that carries
# the eye (the skull's side) is 35 by 24 texels there, far above house.MIN_FACE, so no finer.

OUT = os.environ.get("WYVERN_OUT") or os.path.join(HERE, "out")

# ------------------------------------------------------------------ materials and coats
#
# As the show's dragons: one main color with an accent (Drogon black with red, Rhaegal green
# with bronze, Viserion cream with gold). A coat is three colors: the scales (back), the belly
# plates (under) and the accent (the wing skin and the spikes' tips). The game repaints the scales
# from the genes the same way (`coats.rs`, `wyvern`): back and flank from the scales toward the
# belly color, the accent as "points" (`layers.py`), every texel keeping the light painted here.

(BACK, BACK_EDGE, BACK_HI, FLANK, FLANK_EDGE, FLANK_HI, BELLY, BELLY_EDGE, MEMBRANE, MEM_VEIN, MEM_EDGE,
 HORN, HORN_RING, HORN_TIP, CLAW, CLAW_TIP, TEETH, MOUTH, SPIKE, SPIKE_TIP, DARK) = range(21)
# Each material: (where its color comes from, light shift in ramp steps). "coat" materials are
# repainted by genes: u toward the belly color, or the accent; the rest are fixed.
SCALE_ROLES = {
    BACK: ("coat", 0.0, 0.0), BACK_EDGE: ("coat", 0.0, -1.25), BACK_HI: ("coat", 0.0, 0.5),
    FLANK: ("coat", 0.3, 0.0), FLANK_EDGE: ("coat", 0.3, -1.25), FLANK_HI: ("coat", 0.3, 0.5),
    BELLY: ("coat", 1.0, 0.0), BELLY_EDGE: ("coat", 1.0, -1.0),
    MEMBRANE: ("accent", 0.0, 0.0), MEM_VEIN: ("accent", 0.0, -1.25), MEM_EDGE: ("accent", 0.0, -0.75),
    SPIKE: ("coat", 0.0, -1.5), SPIKE_TIP: ("accent", 0.0, -0.5),
}
FIXED = {HORN: ("horn", (200, 186, 158)), HORN_RING: ("horn", (150, 134, 112)), HORN_TIP: ("horn", (232, 224, 204)),
         CLAW: ("claw", (44, 36, 36)), CLAW_TIP: ("horn", (150, 138, 118)), TEETH: ("horn", (234, 228, 210)),
         MOUTH: ("skin", (104, 42, 40)), DARK: ("claw", (24, 18, 20))}
STEP = 0.25  # paint.STEP: a light shift of k ramp steps is k / STEP indices

# The genes' coats (`genetics/wyvern.json`, Sc): scales, belly, accent; as `coats.rs` makes them.
COATS = {
    "slate": ((92, 92, 98), (112, 70, 60)),
    "bronze": ((132, 92, 52), (96, 54, 38)),
    "green": ((58, 78, 52), (150, 104, 52)),
    "black": ((40, 38, 42), (128, 36, 30)),
    "red": ((118, 40, 34), (52, 26, 24)),
    "cream": ((222, 212, 190), (196, 150, 70)),
}
COAT = os.environ.get("WYVERN_COAT", "black")


def coat_colors(name):
    """(scales, belly, accent) for a coat, as the game's `wyvern` look makes them (mid darkness)."""
    scales, accent = COATS[name]
    s = np.array(scales, float)
    light = s + (np.array([246, 240, 230]) - s) * 0.3
    under = light + (np.array([170, 140, 110]) - light) * 0.4
    return s, under, np.array(accent, float)


def materials(coat):
    """Every material's tones for `coat`."""
    from paint import tones
    from house import MATERIALS as HM
    scales, under, accent = coat_colors(coat)
    out = []
    for m in range(21):
        if m in SCALE_ROLES:
            src, u, shift = SCALE_ROLES[m]
            c = accent if src == "accent" else scales + (under - scales) * u
            t = tones(tuple(c), HM["scale"][1])
            # Painted already shifted (the layers record the shift on the light level).
            idx = np.clip(np.arange(len(t)) + int(round(shift / STEP)), 0, len(t) - 1)
            out.append(t[idx])
        else:
            kind, rgb = FIXED[m]
            out.append(tones(rgb, HM[kind][1]))
    return out


def roles():
    """Each material's role in the coat layers (`layers.Recorder`)."""
    return [("coat", r[1], r[2]) if m in SCALE_ROLES else "fixed" for m in range(21) for r in [SCALE_ROLES.get(m)]]


# ------------------------------------------------------------------ the model (units: 64 a meter)

HIP = np.array([0.0, 235.0, 0.0])       # spine_0's joint, inside the hips
CHEST = np.array([0.0, 240.0, 45.0])    # spine_1's joint, where hips and chest overlap
NECK_BASE = np.array([0.0, 262.0, 160.0])
NECK = [(50, 70, 74), (50, 62, 66), (50, 55, 59), (50, 49, 53)]  # length, width, height
TAIL = [(64, 82, 76), (62, 72, 66), (60, 62, 56), (58, 52, 47), (56, 43, 39), (52, 35, 31), (48, 27, 24), (48, 20, 17)]
TAIL_BASE = np.array([0.0, 236.0, -50.0])
SHOULDER = np.array([60.0, 270.0, 140.0])
# The wing as the show's dragons' (Drogon's span is about 1.5 times his length): a short thick
# upper arm, a long forearm (long enough to reach the ground from an elbow at the height of the
# back, folded), and four long fingers fanning to a broad, deep wing, the last pointing back to
# the hips. Fingers: rest yaw (degrees, + toward the front), segment lengths, widths; leading edge
# first. About 24 m across.
ELBOW_X = 150.0
FINGERS = [(10, (190, 170), (13, 10)), (-20, (175, 160), (11, 8)), (-50, (160, 140), (10, 7)), (-80, (135, 115), (8, 6))]
LEG_X = 62  # the hind legs' middle, out from the hips (56 a side)
WRIST = np.array([420.0, 270.0, 140.0])  # where the hand's bone turns (inside the hand)
# The membrane panels' heights: between each pair of fingers, then the body panel, then the
# outer one; inside the fingers' thickness, a unit apart (no two share a plane).
MEM_Y = (268.0, 269.0, 270.0, 271.0, 272.0)
BODY_EDGE = (np.array([64.0, 270.0, 130.0]), np.array([46.0, 270.0, -40.0]))  # where the membrane grows from the flank


def neck_points():
    pts = [NECK_BASE.copy()]
    for length, _, _ in NECK:
        pts.append(pts[-1] + np.array([0.0, 0.0, length]))
    return pts


def tail_points():
    pts = [TAIL_BASE.copy()]
    for length, _, _ in TAIL:
        pts.append(pts[-1] + np.array([0.0, 0.0, -length]))
    return pts


def yaw_dir(deg):
    """A direction in the wing's plane, `deg` from +x toward +z (the front)."""
    a = math.radians(deg)
    return np.array([math.cos(a), 0.0, math.sin(a)])


def finger_points(side):
    """Each finger's joints in the rest pose: the wrist, the middle joint, the tip."""
    out = []
    for yaw, (la, lb), _ in FINGERS:
        d = yaw_dir(yaw) * np.array([side, 1, 1])
        w = WRIST * np.array([side, 1, 1])
        out.append((w, w + d * la, w + d * (la + lb)))
    return out


def bones():
    B = [("root", None, (0, 0, 0)), ("spine_0", "root", tuple(HIP)), ("spine_1", "spine_0", tuple(CHEST))]
    npts = neck_points()
    for k in range(len(NECK)):
        B.append((f"neck_{k}", "spine_1" if k == 0 else f"neck_{k - 1}", tuple(npts[k])))
    head = npts[-1]
    B += [("head", f"neck_{len(NECK) - 1}", tuple(head)), ("jaw", "head", (0, 243, head[2] + 12))]
    tpts = tail_points()
    for k in range(len(TAIL)):
        # Each segment a little lower than the last: its joint at its own middle height.
        B.append((f"tail_{k}", "spine_0" if k == 0 else f"tail_{k - 1}", (0.0, 236.0 - 2 * k, float(tpts[k][2]))))
    for s, n in ((1, "L"), (-1, "R")):
        x = LEG_X * s
        B += [(f"thigh_{n}", "spine_0", (46 * s, 215, -5)), (f"shin_{n}", f"thigh_{n}", (x, 118, -4)),
              (f"foot_{n}", f"shin_{n}", (x, 46, -4)), (f"toes_{n}", f"foot_{n}", (x, 8, 0))]
        sh = SHOULDER * np.array([s, 1, 1])
        B += [(f"arm_{n}", "spine_1", tuple(sh)), (f"forearm_{n}", f"arm_{n}", (ELBOW_X * s, 270, 140)),
              (f"hand_{n}", f"forearm_{n}", tuple(WRIST * np.array([s, 1, 1]))),
              (f"thumb_{n}", f"hand_{n}", ((WRIST[0] + 4) * s, 270, 148))]
        for k, (w, mid, _) in enumerate(finger_points(s)):
            B += [(f"finger{k}_{n}", f"hand_{n}", tuple(w)), (f"finger{k}b_{n}", f"finger{k}_{n}", tuple(mid))]
        # The panels between the fingers, each on its own bone at the wrist (a child of the finger
        # it hangs from), so folding swings it in against that finger like a closing fan.
        for k in range(len(FINGERS) - 1):
            B.append((f"mem{k}_{n}", f"finger{k}_{n}", tuple(WRIST * np.array([s, 1, 1]) + np.array([0, MEM_Y[k] - 270, 0]))))
        # The membrane from the last finger to the body, in two: the inner panel on the body
        # (turning about the shoulder-to-hip line, where it grows from the flank), the outer one on
        # the last finger (closing with the fan).
        nf = len(FINGERS)
        B += [(f"memb_{n}", "spine_1", tuple(BODY_EDGE[0] * np.array([s, 1, 1]) + np.array([0, MEM_Y[nf - 1] - 270, 0]))),
              (f"memo_{n}", f"finger{nf - 1}_{n}", tuple(WRIST * np.array([s, 1, 1]) + np.array([0, MEM_Y[nf] - 270, 0])))]
    return B


def box_along(name, bone, a, b, w, h, part, extra=0.0, **kw):
    """A box from `a` to `b` (a horizontal run in the wing's plane), `w` wide across and `h` high,
    reaching `extra` past both ends: laid along x, then tilted about y to run from a to b."""
    d = b - a
    length = float(np.linalg.norm(d))
    mid = (a + b) / 2
    yaw = math.degrees(math.atan2(d[2], d[0]))
    lo = (mid[0] - length / 2 - extra, mid[1] - h / 2, mid[2] - w / 2)
    hi = (mid[0] + length / 2 + extra, mid[1] + h / 2, mid[2] + w / 2)
    # rotm's y turn takes +x toward -z for a positive angle: turn by -yaw.
    return el(name, lo, hi, rot=(0, -yaw, 0) if abs(yaw) > 0.01 else None, origin=tuple(mid), bone=bone, part=part, **kw)


def elements():
    E = [
        el("hips", (-56, 186, -56), (56, 284, 50), bone="spine_0", part="body"),
        # The chest shaped by the flight muscles, as a flier's: a core, the ribcage narrowing back
        # to the hips, the hump over the shoulders, a pectoral either side of the breastbone
        # bulging forward, a wing-root muscle on each shoulder widening the top where the wings
        # grow, the keel under it all.
        el("chest", (-66, 172, 44), (66, 290, 166), bone="spine_1", part="body"),
        el("ribs", (-62, 176, 20), (62, 286, 60), bone="spine_1", part="body"),
        el("shoulders", (-58, 286, 98), (58, 304, 168), bone="spine_1", part="body"),
        el("pec_L", (4, 178, 150), (58, 252, 182), bone="spine_1", part="body"),
        el("pec_R", (-58, 178, 150), (-4, 252, 182), bone="spine_1", part="body"),
        el("delt_L", (52, 226, 102), (84, 298, 164), bone="spine_1", part="body"),
        el("delt_R", (-84, 226, 102), (-52, 298, 164), bone="spine_1", part="body"),
        el("keel", (-40, 150, 68), (40, 174, 158), bone="spine_1", part="body"),
    ]
    npts = neck_points()
    for k, (length, w, h) in enumerate(NECK):
        z0 = npts[k][2] - 6
        E.append(el(f"neck_{k}", (-w / 2, 262 - h / 2, z0), (w / 2, 262 + h / 2, npts[k + 1][2] + 4), bone=f"neck_{k}", part="body"))
    hz = npts[-1][2]
    E += [
        el("skull", (-27, 236, hz - 10), (27, 284, hz + 60), bone="head", part="head"),
        el("snout", (-19, 240, hz + 56), (19, 273, hz + 110), bone="head", part="head"),
        el("nose", (-14, 266, hz + 96), (14, 277, hz + 112), bone="head", part="head"),
        el("brow_L", (19, 272, hz + 26), (31, 286, hz + 52), bone="head", part="brow"),
        el("brow_R", (-31, 272, hz + 26), (-19, 286, hz + 52), bone="head", part="brow"),
        el("jaw", (-17, 222, hz + 4), (17, 242, hz + 102), bone="jaw", part="jaw"),
        el("mouth", (-14, 238, hz + 10), (14, 243, hz + 98), bone="jaw", part="mouth"),
    ]
    for s, n in ((1, "L"), (-1, "R")):
        # A crown: a great horn swept up, back and out in three tapering pieces from the skull's
        # back corner, a lesser one below it, and three spikes down the back of the jaw. (A
        # positive x turn lifts a backward-pointing piece; a y turn of -s swings it outward.)
        base = np.array([s * 21, 280, hz])
        E += [el(f"horn_{n}0", (base[0] - 7, 274, hz - 34), (base[0] + 7, 288, hz + 6), rot=(22, -s * 12, 0), origin=tuple(base),
                 bone="head", part="horn"),
              el(f"horn_{n}1", (base[0] - 5, 277, hz - 66), (base[0] + 5, 288, hz - 30), rot=(32, -s * 16, 0), origin=tuple(base),
                 bone="head", part="horn"),
              el(f"horn_{n}2", (base[0] - 3, 279.5, hz - 92), (base[0] + 3, 287.5, hz - 62), rot=(42, -s * 20, 0), origin=tuple(base),
                 bone="head", part="horn"),
              el(f"horn_{n}low", (s * 26 - 4, 256, hz - 28), (s * 26 + 4, 266, hz + 6), rot=(10, -s * 26, 0),
                 origin=(s * 26, 260, hz), bone="head", part="horn")]
        for j in range(3):  # each smaller than the one before, a little apart
            z, w, h = hz + 8 - 17 * j, 3.5 - 0.75 * j, 5 - j
            E.append(el(f"cheek_{n}{j}", (s * 25 - w, 232 - h, z - 14 + 2 * j), (s * 25 + w, 232 + h, z), rot=(-10, -s * 30, 0),
                        origin=(s * 22, 232, z), bone="jaw", part="horn"))
    tpts = tail_points()
    for k, (length, w, h) in enumerate(TAIL):
        z_hi = tpts[k][2] + 6
        y = 236 - 2 * k
        E.append(el(f"tail_{k}", (-w / 2, y - h / 2, tpts[k + 1][2] - 4), (w / 2, y + h / 2, z_hi), bone=f"tail_{k}", part="body"))
    # Spikes along the back, each a base, a middle and a point leaning back: biggest over the
    # shoulders, smaller up the neck and down the tail.
    spine = [(npts[k][2] + 25, 262 + NECK[k][2] / 2, f"neck_{k}", 0.75 + 0.1 * (3 - k)) for k in range(len(NECK))]
    spine += [(150, 304, "spine_1", 1.35), (122, 304, "spine_1", 1.45), (75, 294, "spine_1", 1.3), (25, 284, "spine_0", 1.15),
              (-22, 284, "spine_0", 1.05)]
    spine += [((tpts[k][2] + tpts[k + 1][2]) / 2, 236 - 2 * k + TAIL[k][2] / 2, f"tail_{k}", 1.0 - 0.1 * k) for k in range(len(TAIL) - 1)]
    for k, (z, top, bone, big) in enumerate(spine):
        o = (0, top, z)
        E += [el(f"spike{k}a", (-5 * big, top - 4, z - 9 * big), (5 * big, top + 8 * big, z + 9 * big), rot=(-12, 0, 0), origin=o,
                 bone=bone, part="spike"),
              el(f"spike{k}b", (-3.5 * big, top + 6 * big, z - 9 * big), (3.5 * big, top + 15 * big, z + 3 * big), rot=(-22, 0, 0),
                 origin=o, bone=bone, part="spike"),
              el(f"spike{k}c", (-2 * big, top + 13 * big, z - 12 * big), (2 * big, top + 21 * big, z - 4 * big), rot=(-30, 0, 0),
                 origin=o, bone=bone, part="spike")]
    for s, n in ((1, "L"), (-1, "R")):
        # Hind legs like a big bird of prey's or a lion's: a heavy drumstick of a thigh standing
        # out of the hips, a thick shin and ankle, and a broad foot with three spread toes, a
        # hind toe, and hooked black talons.
        x = LEG_X * s
        E += [
            el(f"thigh_{n}", (x - 34, 108, -46), (x + 34, 236, 34), bone=f"thigh_{n}", part="leg"),
            el(f"shin_{n}", (x - 26, 40, -30), (x + 26, 124, 20), bone=f"shin_{n}", part="leg"),
            el(f"foot_{n}", (x - 21, 7.5, -21), (x + 21, 52, 14), bone=f"foot_{n}", part="leg"),
        ]
        for j, (dx, yaw, L, w, h) in enumerate(((-15, -16, 56, 17, 17), (0, 0, 66, 19, 20), (15, 16, 56, 17, 17))):
            # Toes fan out from under the foot (bases overlapping, not side by side), each
            # ending in a talon: a thick base and a tip hooked down to the ground.
            o = (x + dx * s, 8, 0)
            r = (0, yaw * s, 0) if yaw else None
            lo = 0.6 if yaw else 0
            E += [el(f"toe{j}_{n}", (o[0] - w / 2, lo, -10), (o[0] + w / 2, lo + h, L), rot=r, origin=o, bone=f"toes_{n}", part="leg"),
                  el(f"toeclaw{j}_{n}", (o[0] - w / 2 + 3, lo + 3, L - 6), (o[0] + w / 2 - 3, lo + h - 2, L + 12), rot=r, origin=o,
                     bone=f"toes_{n}", part="claw")]
            # The talon's tip turns down about its own base (yawed with the toe).
            b = np.array(o) + rotm((0, yaw * s, 0)) @ np.array([0, lo + 15 - o[1], L + 6])
            E += [el(f"toetip{j}_{n}", b + (-3.5, -6, 0), b + (3.5, 3, 13), rot=(40, yaw * s, 0), origin=tuple(b),
                     bone=f"toes_{n}", part="claw")]
        E += [el(f"hindtoe_{n}", (x - 6, 1, -44), (x + 6, 14, -14), bone=f"toes_{n}", part="leg"),
              el(f"hindclaw_{n}", (x - 3.5, 5.5, -58), (x + 3.5, 15, -40), rot=(-20, 0, 0), origin=(x, 10, -42), bone=f"toes_{n}",
                 part="claw")]
        # The wing: upper arm, forearm, wrist, thumb with its claw, fingers, membrane.
        S = np.array([s, 1, 1])
        E += [
            # A short, thick upper arm in the shoulder's muscle, a long forearm, the wrist.
            el(f"arm_{n}", tuple(np.array([44, 248, 118]) * S), tuple(np.array([158, 292, 162]) * S), bone=f"arm_{n}", part="arm"),
            el(f"forearm_{n}", tuple(np.array([146, 256, 126]) * S), tuple(np.array([424, 284, 154]) * S), bone=f"forearm_{n}", part="arm"),
            el(f"hand_{n}", tuple(np.array([410, 261, 129]) * S), tuple(np.array([436, 279, 151]) * S), bone=f"hand_{n}", part="arm"),
            el(f"thumb_{n}", tuple(np.array([418, 263, 144]) * S), tuple(np.array([430, 277, 176]) * S), bone=f"thumb_{n}", part="arm"),
            # Two texels in from the thumb's sides and flush with its bottom (at 420.5-427.5 and 264, 1.25-texel
            # bands of the thumb's end showed round it; linecheck).
            el(f"thumbclaw_{n}", tuple(np.array([422, 263.4, 174]) * S), tuple(np.array([426, 273, 190]) * S), bone=f"thumb_{n}",
               part="claw"),
        ]
        for k, ((w, mid, tip), (_, _, (wa, wb))) in enumerate(zip(finger_points(s), FINGERS)):
            E += [box_along(f"finger{k}_{n}", f"finger{k}_{n}", w, mid, wa, wa, "finger", extra=3),
                  box_along(f"finger{k}b_{n}", f"finger{k}b_{n}", mid, tip, wb, wb, "finger", extra=2)]
        E += membranes(s, n)
    for e in E:  # sort out lo/hi after mirroring
        lo, hi = np.minimum(e["lo"], e["hi"]), np.maximum(e["lo"], e["hi"])
        e["lo"], e["hi"] = lo.tolist(), hi.tolist()
    return E


def membranes(s, n):
    """The membrane: a panel between each finger and the next (on the first of the two) and one
    from the last finger back to the body (on the forearm), each a flat quad cut to its shape,
    in the middle of the fingers' thickness, as a bat's membrane grows from the bones, each a unit
    apart and a unit from every finger face, so no two share a plane, even mid-wingbeat."""
    fp = finger_points(s)
    out = []
    S_, H_ = BODY_EDGE[0] * np.array([s, 1, 1]), BODY_EDGE[1] * np.array([s, 1, 1])
    M = (fp[0][0] + S_) / 2 + np.array([s * 30, 0, 4])  # on the forearm, halfway out
    T = (fp[-1][2] + H_) / 2  # halfway along the trailing edge from the last finger to the hip
    shapes = [(f"mem{k}", f"mem{k}_{n}", [fp[k][0], fp[k][2], fp[k + 1][2]]) for k in range(len(fp) - 1)]
    shapes += [("memb", f"memb_{n}", [S_, M, T, H_]), ("memo", f"memo_{n}", [M, fp[-1][0], fp[-1][2], T])]
    for k, (name, bone, poly) in enumerate(shapes):
        poly = [np.array(p, float) for p in poly]
        xs, zs = [p[0] for p in poly], [p[2] for p in poly]
        y = MEM_Y[k]
        out.append(el(f"{name}_{n}", (min(xs), y, min(zs)), (max(xs), y, max(zs)), quad="up", bone=bone, part="membrane",
                      cut="membrane", poly=[p.tolist() for p in poly]))
    return out


# ------------------------------------------------------------------ shapes and areas

def in_poly(x, z, poly):
    """Inside a convex polygon (either winding), for arrays of points."""
    o = orientation(poly)
    inside = np.ones(x.shape, bool)
    for i in range(len(poly)):
        (ax, _, az), (bx, _, bz) = poly[i], poly[(i + 1) % len(poly)]
        inside &= ((bx - ax) * (z - az) - (bz - az) * (x - ax)) * o >= 0
    return inside


def orientation(poly):
    a = 0.0
    for i in range(len(poly)):
        (ax, _, az), (bx, _, bz) = poly[i], poly[(i + 1) % len(poly)]
        a += ax * bz - bx * az
    return 1.0 if a > 0 else -1.0


def cut(e, p):
    """A membrane's outline: its polygon with a scalloped trailing edge (a bite out of each edge
    between two tips, as a bat's wing sags between its fingers)."""
    x, z = p[..., 0], p[..., 2]
    poly = [np.array(v) for v in e["poly"]]
    keep = in_poly(x, z, poly)
    # The trailing edges: tip to tip (and the last tip to the body).
    edges = [(poly[1], poly[2])] if len(poly) == 3 else [(poly[2], poly[3])]
    for a, b in edges:
        m = (a + b) / 2
        d = b - a
        L = float(np.hypot(d[0], d[2]))
        nrm = np.array([-d[2], 0, d[0]]) / max(L, 1e-6)
        # outward: away from the polygon's middle
        c = np.mean(poly, axis=0)
        if np.dot(nrm, m - c) < 0:
            nrm = -nrm
        r = L * 0.75
        center = m + nrm * (r - L * 0.16)
        keep &= np.hypot(x - center[0], z - center[2]) > r
    return keep


# How the scales are drawn (options for Ronald to pick; the same layout): scale size on the body
# in units (the old texels, at 64 a block), and the kind.
STYLES = {
    "fine": dict(size=7, kind="scallop", hi=0.10),       # small overlapping scales, a dark rim each
    "plated": dict(size=11, kind="scallop", hi=0.22),    # big armor-like scales, more catching the light
    "pebbled": dict(size=6, kind="pebble", hi=0.15),     # rounded pebbly scales with dark gaps (a lizard's)
    "banded": dict(size=7, kind="scallop", hi=0.10, bands=True),  # fine scales, darker bands down the back and tail
}
CHOSEN = "plated"  # chosen by Ronald, 2026-11-02
MIN_SCALE = 4  # the smallest scale, in texels (at 32 a block)
STYLE = os.environ.get("WYVERN_STYLE", CHOSEN)

# Each part's scale size against the body's: smaller on the head, feet and wings, down the tail.
def part_scale(name):
    if name in ("skull", "snout", "nose", "jaw") or name.startswith(("brow", "cheek")):
        return 0.55
    if name.startswith(("neck_",)):
        return 0.85
    if name.startswith("tail_"):
        return 0.95 - 0.06 * int(name.split("_")[1])
    if name.startswith(("foot", "toe", "hindtoe", "finger", "hand", "thumb")):
        return 0.5
    if name.startswith(("shin", "arm", "forearm")):
        return 0.7
    return 1.0


def hashed(i, j, seed=0):
    """A stable 0..1 number per scale."""
    v = np.sin(i * 12.9898 + j * 78.233 + seed * 37.719) * 43758.5453
    return v - np.floor(v)


def scale_cells(a, b, size, kind):
    """Scales on a face, from 2D coordinates on the body (`a` along a row, `b` up the rows, in
    units): which scale each texel is in, whether it is on the scale's rim (dark), and the
    scale's middle on `b`. Scallops overlap like roof tiles, each rim a U open toward the next
    row (scales lie back and down over the ones below); pebbles are rounded, a dark gap round each.
    At 32 a block a rim is a texel (1 / TX units) and a scale at least MIN_SCALE texels across, so
    the smallest (head, feet) still read as scales, not speckle."""
    size = max(size, MIN_SCALE / TX)
    if kind == "pebble":
        row = size * 0.87
        j = np.floor(b / row)
        a2 = a / size + 0.5 * (j % 2)
        i = np.floor(a2)
        fa, fb = a2 - i - 0.5, b / row - j - 0.5
        rim = (fa * size) ** 2 + (fb * row) ** 2 > (0.5 * size - 0.9 / TX) ** 2
        return i, j, rim, (j + 0.5) * row
    row = size * 0.8
    j = np.floor(b / row)
    a2 = a / size + 0.5 * (j % 2)
    i = np.floor(a2)
    fa, fb = a2 - i, b / row - j
    rim = fb * row < 1.0 / TX + 0.55 * row * (2 * fa - 1) ** 2
    return i, j, rim, (j + 0.5) * row


def body_axis(name):
    """Along which coordinate a part runs (its scales' rows run across it): z for the body, neck
    and tail and the legs' sides, x for the wings' bones."""
    return 0 if name.startswith(("arm", "forearm", "hand", "thumb", "finger")) else 2


def scaled_area(e, face, p, regions):
    """Scales over a face: `regions(center_height_fraction, face)` picks back, flank or belly per
    scale, so the colors meet along whole scales; each region its base, rim and catching-light
    materials."""
    st = STYLES[STYLE]
    size = st["size"] * part_scale(e["name"])
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    ax = body_axis(e["name"])
    along = z if ax == 2 else x
    if face in ("up", "down"):
        a, b = (x, z) if ax == 2 else (z, x)
        b = -b  # rows stacked from the front back: rims face the tail
    elif face in ("east", "west"):
        a, b = (-z, y) if ax == 2 else (x, y)
    else:
        a, b = (x, y) if ax == 2 else (z, y)
    i, j, rim, mid_b = scale_cells(a, b, size, st["kind"])
    lo, hi = e["lo"][1], e["hi"][1]
    if face in ("east", "west", "north", "south"):
        frac = (mid_b - lo) / max(hi - lo, 1e-3)
    else:
        frac = np.full(x.shape, 1.0 if face == "up" else 0.0)
    region = regions(frac, face)  # 0 back, 1 flank, 2 belly
    base = np.choose(region, [BACK, FLANK, BELLY])
    edge = np.choose(region, [BACK_EDGE, FLANK_EDGE, BELLY_EDGE])
    hi_m = np.choose(region, [BACK_HI, FLANK_HI, BELLY])
    lit = hashed(i, j, sum(map(ord, e["name"])) % 13) < st["hi"]
    out = np.where(rim, edge, np.where(lit, hi_m, base))
    if st.get("bands") and e["name"].startswith(("tail_", "hips", "chest", "shoulders", "neck_")):
        band = np.mod(along, 56.0) < 12
        out = np.where(band & (region < 2) & ~rim, BACK_EDGE, out)
    return out


def belly_plates(e, p):
    """Plates across the belly, throat and under the tail: bands across the body's axis, a dark
    seam between each, the plates' sides running into the flank's scales."""
    x, z = p[..., 0], p[..., 2]
    width = 9 * part_scale(e["name"]) if not e["name"].startswith("neck") else 7
    width = max(width, 3 / TX)  # at least 3 texels a plate, at 32 a block
    seam = np.mod(z, width) < 1 / TX  # a seam a texel wide
    half = (e["hi"][0] - e["lo"][0]) / 2
    side = np.abs(x - (e["hi"][0] + e["lo"][0]) / 2) > half * 0.78
    return np.where(side, FLANK_EDGE, np.where(seam, BELLY_EDGE, BELLY))


def unrotated(e, p):
    """A tilted element's points back in its own (unturned) frame."""
    if not e.get("rot"):
        return p
    o = np.array(e["origin"], float)
    return (p - o) @ rotm(e["rot"]) + o


def horn_area(e, p, pieces=3):
    """A horn or spike piece: ringed toward the base, paling to the tip. `t` 0 at the horn's base,
    1 at its tip, from which piece of the horn this is and where along it."""
    q = unrotated(e, p)
    z = q[..., 2]
    lo, hi = e["lo"][2], e["hi"][2]
    n = e["name"]
    k = int(n[-1]) if n[-1].isdigit() else {"a": 0, "b": 1, "c": 2}.get(n[-1], 0)
    t = (k + (hi - z) / max(hi - lo, 1e-3)) / pieces
    if e["part"] == "spike":
        return np.where(t > 0.72, SPIKE_TIP, SPIKE)
    ring = (np.mod(z, 8.0) < 1 / TX) & (t < 0.62)  # a ring a texel wide, 4 texels apart (32 a block)
    return np.where(t > 0.78, HORN_TIP, np.where(ring | (t < 0.08), HORN_RING, HORN))


def teeth_rows(face, uu, vv, w, h, top, rows=2, period=4.0, phase=0.0, front=1.0):
    """Teeth along an edge of a face: pointed, `rows` tall, every `period` texels (`uu`, `vv` the
    texel centers, in texels: a design, 3 wide and 2 tall at 32 a block); `top` the edge they hang
    from is the face's top (the lower jaw's, pointing up) or its bottom (the upper's)."""
    d = (vv if top else h - vv) - 0.5  # rows from the edge, 0 at it
    c = np.mod(uu - 0.5 + phase, period) - period / 2  # across each tooth
    tooth = np.abs(c) < (rows - d) * 0.55
    along = (uu / w) if face == "east" else (1 - uu / w)
    return tooth & (d < rows) & (along < front)


def membrane_area(e, p):
    """Wing skin: the accent color, veins fanning from where the panel grows (the wrist, or along
    the body), and a darker trailing edge."""
    from scipy import ndimage
    shape = p.shape[:-1]
    a = np.full(shape, MEMBRANE)
    poly = [np.array(v) for v in e["poly"]]
    if len(poly) == 3:      # between two fingers: from the wrist toward the edge between the tips
        origins = [poly[0], poly[0]]
        ends = [poly[1] + (poly[2] - poly[1]) * k for k in (1 / 3, 2 / 3)]
    else:                   # the body panels: from their first edge toward the trailing edge
        origins = [poly[0] + (poly[3] - poly[0]) * k for k in (1 / 3, 2 / 3)]
        ends = [poly[1] + (poly[2] - poly[1]) * k for k in (1 / 3, 2 / 3)]
    x, z = p[..., 0], p[..., 2]
    for o, b in zip(origins, ends):
        d = np.array([b[0] - o[0], b[2] - o[2]])
        L = np.hypot(*d) + 1e-6
        t = np.clip(((x - o[0]) * d[0] + (z - o[2]) * d[1]) / L ** 2, 0, 1)
        dist = np.hypot(x - (o[0] + t * d[0]), z - (o[2] + t * d[1]))
        a = np.where(dist < (0.75 - 0.25 * t) / TX, MEM_VEIN, a)  # a texel and a half, a texel toward the edge
    keep = cut(e, p)
    inner = ndimage.binary_erosion(keep, iterations=2)  # the trailing edge: 2 texels (4 units)
    return np.where(keep & ~inner & (a == MEMBRANE), MEM_EDGE, a)


def area(e, face, p, uu, vv, h):
    """`uu`, `vv`: texel centers, in texels (for the stamps: teeth, the lip, nostrils)."""
    part, n = e.get("part"), e["name"]
    shape = p.shape[:-1]
    w = shape[1]
    if part == "membrane":
        return membrane_area(e, p)
    if part in ("horn", "spike"):
        return horn_area(e, p)
    if part == "claw":
        return np.full(shape, CLAW_TIP if n.startswith(("toetip", "hindclaw")) or n.startswith("thumbclaw") else CLAW)
    if part == "mouth":
        return np.full(shape, MOUTH)
    if n == "snout" and face == "down":  # the roof of the mouth, teeth round its edge
        a = np.full(shape, MOUTH)
        edge = (uu < 1) | (uu > w - 1) | (vv > h - 1)  # a texel round its edge
        return np.where(edge, TEETH, a)
    if n in ("hips", "chest", "ribs", "keel", "jaw") or n.startswith(("neck_", "tail_", "pec_")):
        if face == "down":
            return belly_plates(e, p)
        under = 0.22 if n != "jaw" else 0.4

        def regions(frac, f):
            return np.where(frac > 0.6, 0, np.where(frac < under, 2, 1))
    else:
        def regions(frac, f):
            if f == "up":
                return np.zeros(frac.shape, int)
            if f == "down":
                return np.ones(frac.shape, int)
            return np.where(frac > 0.55, 0, 1)
    a = scaled_area(e, face, p, regions)
    if n.startswith("brow"):
        a = np.where(a == BACK_HI, BACK, a)
    if n == "snout" and face in ("east", "west"):
        a = np.where(teeth_rows(face, uu, vv, w, h, top=False, front=0.8), TEETH, a)
        a[-3:][~np.isin(a[-3:], [TEETH])] = DARK  # the lip's shadow between the teeth, a texel above them
    if n == "jaw" and face in ("east", "west"):
        rows = (vv >= 1) & (vv < 3)  # (the top row, 2 units, is behind the snout)
        a = np.where(rows & teeth_rows(face, uu, vv - 1, w, h, top=True, phase=2, front=0.8), TEETH, a)
    if n == "nose" and face == "up" and w >= 8 and h >= 4:  # nostrils: slits near the front (a pair on the front reads as eyes)
        a[h - 2:h - 1, 1:3] = DARK  # (14 by 8 texels at 32 a block: 2-texel slits a texel in)
        a[h - 2:h - 1, w - 3:w - 1] = DARK
    return a


def layer_props(e, face, a):
    """The accent (wing skin, spike tips) as `points`; where its markings show (`Mk`): fully on
    the scales of the back and flanks, faintly on the belly plates and through the wing skin."""
    pts = np.isin(a, [MEMBRANE, MEM_VEIN, MEM_EDGE, SPIKE_TIP]).astype(float)
    zone = np.select([np.isin(a, [BACK, BACK_EDGE, BACK_HI, FLANK, FLANK_EDGE, FLANK_HI]),
                      np.isin(a, [SPIKE]), np.isin(a, [MEMBRANE, MEM_VEIN, MEM_EDGE]), np.isin(a, [BELLY, BELLY_EDGE])],
                     [1.0, 0.6, 0.35, 0.25], 0.0)
    return {"points": pts, "pattern": zone}


# The eye: an amber reptile's, a slit pupil, under the brow on each side of the skull. At 32 a
# block the skull's side is 35 by 24 texels: the old 13 by 8 design ("reptile", at 64) halved, a
# dark rim, the amber iris (lighter low), a slit pupil two texels tall, a catchlight up and toward
# the snout (mirrored on the right side, where the snout is the other way). The halo's dark ring
# sits under the brow.
EYES.update({
    "wyvern_px": dict(pattern=[".LLLLL.",
                               "LHAPAAL",
                               "LAaPaaL",
                               ".LLLLL."],
                      colors={"L": (52, 30, 20), "A": (196, 124, 34), "a": (232, 170, 64), "P": (14, 10, 10),
                              "H": (246, 230, 190)}, halo="round", iris="Aa"),
})
EYE = ("wyvern_px", 18.0)  # design, how far down from the skull's top (units: under the brow)
EYE_Z = 40.0  # from the skull's back face forward


EYES[EYE[0] + "_w"] = dict(EYES[EYE[0]], pattern=[row[::-1] for row in EYES[EYE[0]]["pattern"]])


def face_frame(lo, hi, face):
    """`prop.face_frame` with the face's size in texels at TX (32 a block), at least a texel for
    any face with an area (painting only: the geometry stays in units)."""
    from prop import face_frame as unit_frame
    corner, du, dv, _, _, n = unit_frame(lo, hi, face)
    size = np.array(hi, float) - np.array(lo, float)
    fu, fv = abs(size @ du), abs(size @ dv)
    w, h = int(round(TX * fu)), int(round(TX * fv))
    if fu > 0 and fv > 0:
        w, h = max(1, w), max(1, h)
    return corner, du, dv, w, h, n


def paint_sheet(E, coat=None, style=None, rec=None):
    """Paints every face (the house light, ambient occlusion), stamps the eyes, packs the sheet.
    Returns (sheet, uv, keys); `rec` a `layers.Recorder` to record the coat layers into."""
    global STYLE
    from paint import occlusion, paint_face_smooth, pack, stamp_eyes
    old = STYLE
    STYLE = style or STYLE
    tone_sets = materials(coat or COAT)
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"] and not e["rot"]]
    tiles = []
    for i, e in enumerate(E):
        for face in ([e["quad"]] if e["quad"] else list(FACES)):
            corner, du, dv, w, h, nrm = face_frame(e["lo"], e["hi"], face)
            if w == 0 or h == 0:
                continue
            uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)  # texel centers, in texels
            size = np.array(e["hi"]) - np.array(e["lo"])
            su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel
            p = corner + (uu * su)[..., None] * du + (vv * sv)[..., None] * dv
            R = rotm(e["rot"])
            if e["rot"]:
                o = np.array(e["origin"], float)
                p = (p - o) @ R.T + o
            a = area(e, face, p, uu, vv, h)
            occ = None if e["quad"] else occlusion(p, R @ nrm, R @ du, R @ dv, boxes, own=e["name"])
            # Undersides seen only when a pose lifts or folds them, lit as backs (not black).
            light = "north" if face == "down" and e["part"] in ("leg", "arm", "finger", "jaw", "claw") else face
            cap = {}
            col = paint_face_smooth(a, tone_sets, light, coat=(), y_world=p[..., 1], occ=occ, quad=bool(e["quad"]),
                                    seed=(sum(map(ord, e["name"] + face)) + 5) % 97, out=cap)
            img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
            if e["cut"]:
                img[..., 3] = np.where(cut(e, p), 255, 0)
            if e["name"] == "skull" and face in ("east", "west"):
                lo, hi = e["lo"], e["hi"]
                u = (hi[2] - (lo[2] + EYE_Z)) if face == "east" else EYE_Z
                design = EYE[0] if face == "east" else EYE[0] + "_w"  # the catchlight toward the snout
                stamp_eyes(img, [(u / su, EYE[1] / sv)], design)
            if rec is not None:
                rec.face((i, face), cap["weights"], cap["level"], roles(), np.clip(np.round(col), 0, 255), img, p,
                         layer_props(e, face, a))
            tiles.append(((i, face), img))
    sheet, uv = pack(tiles, 1024)
    STYLE = old
    return sheet, uv, [k for k, _ in tiles]


# ------------------------------------------------------------------ poses and animations
#
# Angles in degrees, Blockbench's convention (R = Rz Ry Rx; +X pitches a bone's +z end down, so a
# negative X lifts the neck; for the left wing, which runs along +x, -Z lowers it and +Y swings it
# back). The right side mirrors the left: Y and Z negated.

FIT = {}  # fitted by fit_poses: the stand's drop, the folded wing's angles


def mirror(pose):
    """The left side's angles copied to the right, mirrored."""
    out = dict(pose)
    for k, v in pose.items():
        if k.endswith("_L"):
            out[k[:-2] + "_R"] = (v[0], -v[1], -v[2])
    if "scale" in pose:  # {bone: (x, y, z)}: sizes, the same on both sides
        out["scale"] = {**pose["scale"], **{k[:-2] + "_R": v for k, v in pose["scale"].items() if k.endswith("_L")}}
    return out


def legs_crouch(thigh=-46.0, shin=94.0, foot=-66.0):
    toes = -(thigh + shin + foot)  # toes flat on the ground
    return {"thigh_L": (thigh, 0, 0), "shin_L": (shin, 0, 0), "foot_L": (foot, 0, 0), "toes_L": (toes, 0, 0)}


def neck_s(lift=1.0):
    """The neck raised in an S, the head held about level."""
    return {"neck_0": (-24 * lift, 0, 0), "neck_1": (-14 * lift, 0, 0), "neck_2": (8 * lift, 0, 0), "neck_3": (16 * lift, 0, 0),
            "head": (12 * lift, 0, 0)}


def tail_curve(droop=6.0, sway=0.0, t=0.0):
    """The tail drooping toward the ground (it points back: a negative X lowers it), swaying."""
    return {f"tail_{k}": (-droop * (1 if k < 3 else 0.4), sway * math.sin(t * 2 * math.pi - k * 0.6), 0) for k in range(len(TAIL))}


def fan_closed():
    """The fan half closed, turned up by the hand as a sail beside the body: each finger turned
    toward the first (the spread halved), each panel between two fingers centred in the gap left
    (a rigid panel can't shrink with it: it reaches a little past both fingers, over its
    neighbours, a unit apart from them); the skin from the last finger back to the body drawn in,
    as a bat's contracts when the wing closes: the outer panel to a sliver at the wrist, the inner
    one turned down about the flank line to a sliver inside the flank (at full size they would
    hang through the legs)."""
    out = {}
    first = FINGERS[0][0]
    for k, (yaw, _, _) in enumerate(FINGERS):
        # Toward the first finger (a smaller yaw angle is +Y here: Ry turns +x toward -z).
        out[f"finger{k}_L"] = (0, (yaw - first) * (1 - FAN_CLOSED), 0)
    for k in range(len(FINGERS) - 1):
        gap = FINGERS[k][0] - FINGERS[k + 1][0]
        out[f"mem{k}_L"] = (0, -gap * (1 - FAN_CLOSED) / 2, 0)
    S_, H_ = BODY_EDGE
    u = (H_ - S_) / np.linalg.norm(H_ - S_)
    a = math.radians(FOLD_FLANK) / 2
    out["memb_L"] = tuple(round(v, 2) for v in euler(np.array([*(u * math.sin(a)), math.cos(a)])))
    # Toward their pivots: the shoulder (inside the chest) and the wrist (inside the hand).
    out["scale"] = {"memb_L": (0.04, 1.0, 0.04), "memo_L": (0.01, 1.0, 0.01)}
    return out


FOLD_FLANK = 88  # degrees the inner panel turns about the flank line, folded
FAN_CLOSED = 0.5  # how much of the fingers' spread is left, folded


def folded_wing():
    w = fan_closed()
    w.update({k: tuple(v) for k, v in FIT.get("wing", {}).items()})
    return w


def stand(t=0.0):
    p = {}
    p.update(legs_crouch())
    p.update(neck_s())
    p.update(tail_curve())
    p.update(folded_wing())
    p["root"] = (0, -FIT.get("drop", 0.0), 0)
    return mirror(p)


def pose_idle(t):
    p = stand()
    br = math.sin(t / 4.0 * 2 * math.pi)
    p["spine_1"] = (br * 1.0, 0, 0)  # breathing
    look = 10 * math.sin(t / 4.0 * 2 * math.pi * 0.5)
    p["neck_3"] = (p["neck_3"][0], look * 0.5, 0)
    p["head"] = (p["head"][0], look, 0)
    p.update(mirror(tail_curve(sway=5, t=t / 4.0)))
    return p


def flying_body():
    """Legs tucked back along the body, neck stretched forward, tail straight behind."""
    p = {"thigh_L": (55, 0, 0), "shin_L": (40, 0, 0), "foot_L": (30, 0, 0), "toes_L": (20, 0, 0)}
    p.update({"neck_0": (-6, 0, 0), "neck_1": (-2, 0, 0), "neck_2": (2, 0, 0), "neck_3": (4, 0, 0), "head": (6, 0, 0)})
    return p


SWEEP = -12  # degrees the arms reach forward in flight: the leading edge angled forward, as Drogon's


def pose_fly(t):
    """A wingbeat (one a second): the downstroke strong and quick, the upstroke with the wing
    half folded, the fingers trailing behind the arm."""
    ph = t * 2 * math.pi
    up = math.cos(ph)  # 1 at the top of the stroke
    fold = max(0.0, math.sin(ph)) * 0.5  # folding on the way up
    p = flying_body()
    arm = (0, SWEEP + 16 * fold, 28 * up + 4)
    p.update({
        "arm_L": arm,
        "memb_L": arm,  # the inner membrane follows the upper arm in flight
        "forearm_L": (0, -14 * fold, 8 * up - 24 * fold),
        "hand_L": (0, 0, 10 * math.cos(ph - 0.6) - 14 * fold),
        # The fingers flex together behind the hand (out of step, panels and fingers would cross).
        "finger0_L": (0, 0, 5 * math.cos(ph - 1.1)),
        "finger1_L": (0, 0, 5 * math.cos(ph - 1.1)),
        "finger2_L": (0, 0, 5 * math.cos(ph - 1.1)),
        "spine_1": (2 * up, 0, 0),
    })
    p.update(tail_curve(droop=-2, sway=3, t=t))
    p["root"] = (0, 14 * math.sin(ph + 0.5), 0)  # the body rises on each downstroke
    return mirror(p)


def pose_glide(t):
    ph = t / 3.0 * 2 * math.pi
    p = flying_body()
    arm = (0, SWEEP, 8 + 2 * math.sin(ph))
    p.update({"arm_L": arm, "memb_L": arm, "forearm_L": (0, 0, -4), "hand_L": (0, 0, -3 + 2 * math.sin(ph + 1))})
    p.update(tail_curve(droop=-2, sway=4, t=t / 3.0))
    return mirror(p)


# ------------------------------------------------------------------ walking and running
#
# On the ground it goes on four: the hind feet and the wrists of the folded wings (the thumb's
# claw), as bats and pterosaurs do. Each foot's path is set and the limb's angles solved to follow
# it (inverse kinematics on the model's own bones), so feet on the ground don't slide: Weft plays
# a walk 0.6 seconds of clip per block walked, so a planted foot moves back GROUND_SPEED units
# per clip second (a block is house.DENSITY units).

GROUND_SPEED = 64 / 0.6
SKELETON = []  # the spec's bones, for the solver (set by build)
THUMB_TIP = np.array([424.0, 268.0, 188.0])  # the thumb claw's tip at rest: the wrist's foot
HAND_LOW = None  # the hand box's corners (set by build)
GAITS = {
    # length (clip seconds: the stride over GROUND_SPEED), the part of the stride a foot is down,
    # when each foot sets down (fraction of the stride), how high the hind feet and wrists lift,
    # how far out a wing swings round as it comes forward, how much the body rolls and shifts
    # over the planted wing. As the show's dragons (animated from gorillas, which walk on their
    # knuckles; a bat's hop was tried and dropped), the wings step one after the other and carry
    # the walk: each swings round and plants, the shoulders roll and the weight shifts onto it.
    # A walk, heavy and deliberate, in a primate's order: left leg, right wing, right leg, left wing.
    "walk": {"length": 1.8, "duty": 0.66, "phase": {"leg_L": 0.0, "wing_R": 0.25, "leg_R": 0.5, "wing_L": 0.75},
             "lift": {"leg": 34, "wing": 58}, "out": 16, "roll": 1.0},
    # A gallop: one wing strikes, then the other, vaulting the body forward; the legs land under
    # it one after the other and drive it on; a moment in the air after each pair.
    "run": {"length": 3.0, "duty": 0.3, "phase": {"wing_L": 0.0, "wing_R": 0.16, "leg_L": 0.5, "leg_R": 0.64},
            "lift": {"leg": 52, "wing": 64}, "out": 12, "roll": 0.6},
}


def foot_path(neutral, gait, limb, s):
    """Where a foot is at stride fraction `s` since it set down: on the ground moving back at the
    ground speed, then lifted and swung forward (eased) to set down ahead again."""
    g = GAITS[gait]
    travel = GROUND_SPEED * g["length"] * g["duty"]
    lift = g["lift"][limb]
    if s < g["duty"]:
        return neutral + np.array([0, 0, travel / 2 - travel * s / g["duty"]]), 0.0
    v = (s - g["duty"]) / (1 - g["duty"])
    out = g["out"] * math.sin(math.pi * v) if limb == "wing" else 0.0  # (the left side: +x is out)
    return neutral + np.array([out, lift * math.sin(math.pi * v), -travel / 2 + travel * (0.5 - 0.5 * math.cos(math.pi * v))]), v


def solve_limb(body, limb, target, swing, guess):
    """The left limb's angles that put its foot at `target` (world, the body posed by `body`);
    the right side mirrors them. Hind leg: thigh, shin and ankle, the toes flat (curled down
    mid-swing, flat again to set down). Wing: the upper arm and forearm turned from the fold, the hand and fingers kept."""
    from scipy.optimize import least_squares
    from ground import world

    piv = {b["name"]: np.array(b["pivot"], float) for b in SKELETON}

    def at(pose, bone, local):
        R, t = world(SKELETON, pose)[bone]
        return R @ (np.asarray(local, float) - piv[bone]) + t

    if limb == "leg":
        base = legs_crouch()
        hips = body.get("spine_0", (0, 0, 0))[0]  # the toes level with the ground, not the hips

        def pose_of(x):
            p = dict(body)
            th, sh, ft = base["thigh_L"][0] + x[0], base["shin_L"][0] + x[1], base["foot_L"][0] + x[2]
            p.update({"thigh_L": (th, 0, 0), "shin_L": (sh, 0, 0), "foot_L": (ft, 0, 0), "toes_L": (-(hips + th + sh + ft) + 16 * math.sin(math.pi * swing), 0, 0)})
            return p

        def res(x):
            p = pose_of(x)
            d = at(p, "toes_L", (LEG_X, 8, 0)) - target
            heel = at(p, "foot_L", (LEG_X, 7.5, -21))[1]  # the heel lifts rather than sink
            tip = at(p, "toes_L", (LEG_X, 0, 66))[1]  # nor do the toes' tips
            return [d[1], d[2], 0.08 * x[0], 0.05 * x[1], 0.15 * x[2], 3 * min(0.0, heel - 1), 3 * min(0.0, tip - 1)]
        n = 3
    else:
        fold = {k: np.array(v, float) for k, v in FIT["wing"].items()}

        def pose_of(x):
            p = dict(body)
            p["arm_L"] = tuple(fold["arm_L"] + x[:3])
            p["forearm_L"] = tuple(fold["forearm_L"] + x[3:])
            return p

        def res(x):
            p = pose_of(x)
            d = at(p, "thumb_L", THUMB_TIP) - target
            low = min(at(p, "hand_L", c)[1] for c in HAND_LOW)  # the wrist's underside stays above ground
            return [*d, *(0.06 * np.asarray(x)), 3 * min(0.0, low - 1)]
        n = 6
    r = least_squares(res, guess if guess is not None else np.zeros(n), xtol=1e-4)
    p = pose_of(r.x)
    own = ("thigh", "shin", "foot", "toes") if limb == "leg" else ("arm", "forearm")
    return {f"{b}_L": p[f"{b}_L"] for b in own}, r.x


def gait_body(gait, u):
    """The body's own movement at stride fraction `u`: the bob, the shoulders and hips rolling
    over the planted feet, the neck and tail swinging against each other."""
    p = {}
    p.update(neck_s())
    g = GAITS[gait]
    # 1 with the weight on the left wing (the middle of its time down), -1 on the right: the body
    # shifts over it, the shoulders roll down onto it and turn the other wing's shoulder forward
    # as that wing swings round, the hips turn against them, the head stays on what it looks at.
    lean = g["roll"] * math.cos(2 * math.pi * (u - g["phase"]["wing_L"] - g["duty"] / 2))
    if gait == "walk":
        w = 2 * math.pi * u
        p["root"] = (5 * lean, -FIT["drop"] - 3 * math.cos(2 * w), 0)
        p["spine_0"] = (0, -2.5 * lean, 1.5 * lean)
        p["spine_1"] = (0, 6 * lean, -3.5 * lean)
        p["neck_0"] = (p["neck_0"][0] + 2 * math.cos(2 * w), -4 * lean, 0)
        p["head"] = (p["head"][0] - 2 * math.cos(2 * w), -3 * lean, 0)
        p.update({f"tail_{k}": (-5 * (1 if k < 3 else 0.4), -7 * math.sin(2 * math.pi * (u - g["phase"]["wing_L"]) - 0.6 * k), 0)
                  for k in range(len(TAIL))})
    else:
        w = 2 * math.pi * u
        # Two leaps a stride, the back flexing (gathered
        # when the legs reach forward under it, stretched when the wings reach out).
        # Highest in the moments in the air (after the wings leave, after the legs leave).
        p["root"] = (3 * lean, -FIT["drop"] + 4 + 7 * math.cos(2 * w + 0.25), 0)
        p["spine_0"] = (-4 * math.cos(w - 0.6), -2 * lean, 0)
        p["spine_1"] = (8 * math.cos(w - 0.6), 5 * lean, -3 * lean)
        p.update(neck_s(lift=0.45))
        p["neck_0"] = (p["neck_0"][0] + 8, 0, 0)  # neck low and forward, head up to see ahead
        p["head"] = (p["head"][0] - 14 - 5 * math.cos(w - 0.6), 0, 0)
        p["jaw"] = (6 + 3 * math.sin(2 * w), 0, 0)
        # The tail straight out behind as a counterweight, rising and falling with the back
        # (each segment's turn adds to the last's, so small ones).
        p.update({f"tail_{k}": (-1 + 2.2 * math.sin(w - 0.6 - 0.5 * k), 2 * math.sin(w - 0.4 * k), 0) for k in range(len(TAIL))})
    return p


GAIT_POSES = {}  # gait -> [pose per frame], solved once


def neutral_feet():
    """Where the left hind foot (the toes' joint) and wrist (the thumb claw's tip) stand."""
    from ground import world
    rest = mirror({**legs_crouch(), **{k: v for k, v in folded_wing().items() if k != "scale"}, "root": (0, -FIT["drop"], 0)})
    W = world(SKELETON, rest)
    piv = {b["name"]: np.array(b["pivot"], float) for b in SKELETON}
    (Rl, tl), (Rw, tw) = W["toes_L"], W["thumb_L"]
    return {"leg": Rl @ (np.array([LEG_X, 8.0, 0.0]) - piv["toes_L"]) + tl, "wing": Rw @ (THUMB_TIP - piv["thumb_L"]) + tw}


def flipped(body):
    """The body's turns and position seen in a mirror (left for right), so a right limb can be
    solved as a left one."""
    return {k: (-v[0], v[1], v[2]) if k == "root" else (v[0], -v[1], -v[2]) for k, v in body.items()}


def solve_gait(gait):
    """Every frame's pose: the body's movement, then each limb solved to its foot's path with the
    body as it is (the right limbs solved as left ones in the mirrored body)."""
    if gait in GAIT_POSES:
        return GAIT_POSES[gait]
    g = GAITS[gait]
    fold = folded_wing()
    neutral = neutral_feet()
    neutral["wing"] = neutral["wing"] + (0, 3, 0)  # clear of the ground as the body bobs onto it
    guesses = {}
    frames = []
    for t in frame_times(gait, g["length"]):
        u = (t / g["length"]) % 1.0
        body = gait_body(gait, u)
        limbs = mirror({**{k: v for k, v in fold.items() if k != "scale"}, **legs_crouch()})
        seen_from = {"L": {**body, **limbs}, "R": {**flipped(body), **limbs}}
        pose = dict(body)
        pose.update(fold)
        for side in ("L", "R"):
            for limb in ("leg", "wing"):
                s = (u - g["phase"][f"{limb}_{side}"]) % 1.0
                target, swing = foot_path(neutral[limb], gait, limb, s)
                key = (limb, side)
                angles, guesses[key] = solve_limb(seen_from[side], limb, target, swing, guesses.get(key))
                pose.update(angles if side == "L" else {k[:-2] + "_R": (v[0], -v[1], -v[2]) for k, v in angles.items()})
        frame = mirror({k: v for k, v in pose.items() if not k.endswith("_R")})  # the fold and the body
        frame.update({k: v for k, v in pose.items() if k.endswith("_R")})  # the right limbs as solved
        frames.append(frame)
    GAIT_POSES[gait] = frames
    return frames


def pose_gait(gait):
    def fn(t):
        times = frame_times(gait, GAITS[gait]["length"])
        return solve_gait(gait)[min(range(len(times)), key=lambda i: abs(times[i] - t))]
    return fn


# ------------------------------------------------------------------ breathing fire (or frost)

def blend(a, b, k):
    """Two poses mixed, `k` of the way from a to b."""
    out = {}
    for key in set(a) | set(b):
        if key == "scale":
            out[key] = a.get(key) or b.get(key)
            continue
        va, vb = np.array(a.get(key, (0, 0, 0)), float), np.array(b.get(key, (0, 0, 0)), float)
        out[key] = tuple(va + (vb - va) * k)
    return out


def ease(x):
    x = min(max(x, 0.0), 1.0)
    return x * x * (3 - 2 * x)


def pose_breathe(t):
    """A breath attack, 3 seconds: it rears and draws breath (chest up, neck back, head high,
    wings flaring a little), lunges forward with the neck stretched and the jaws wide, holds the
    stream while sweeping the head across, then settles. The game draws the fire, frost or
    whatever the wyvern breathes from the mouth while the jaws are open (0.8 to 2.6 s)."""
    base = stand()
    draw = dict(base)
    draw.update(mirror({
        "spine_1": (-7, 0, 0), "neck_0": (-38, 0, 0), "neck_1": (-20, 0, 0), "neck_2": (2, 0, 0), "neck_3": (8, 0, 0),
        "head": (-4, 0, 0), "jaw": (4, 0, 0),
        "arm_L": tuple(np.array(FIT["wing"]["arm_L"]) + (0, 0, 8)),
    }))
    draw["root"] = (0, -FIT["drop"] + 6, 0)
    blast = dict(base)
    blast.update(mirror({
        # The neck thrust out nearly straight, the head aimed ahead and a little down.
        "spine_1": (5, 0, 0), "neck_0": (-18, 0, 0), "neck_1": (-6, 0, 0), "neck_2": (4, 0, 0), "neck_3": (6, 0, 0),
        "head": (6, 0, 0), "jaw": (38, 0, 0),
        "arm_L": tuple(np.array(FIT["wing"]["arm_L"]) + (0, 0, 4)),
    }))
    blast["root"] = (0, -FIT["drop"], 0)
    # The wrists stay planted under the lunge (solved to where they stand).
    wing, _ = solve_limb({k: v for k, v in blast.items() if k != "scale"}, "wing", neutral_feet()["wing"], 0.0, None)
    blast.update(mirror(wing))
    if t < 0.7:
        p = blend(base, draw, ease(t / 0.7))
    elif t < 0.95:
        p = blend(draw, blast, ease((t - 0.7) / 0.25))
    elif t < 2.6:
        p = dict(blast)
        sweep = math.sin((t - 0.95) / 1.65 * 2 * math.pi) * 14  # across and back
        p["neck_2"] = (p["neck_2"][0], sweep * 0.5, 0)
        p["head"] = (p["head"][0] + 1.5 * math.sin(t * 40), sweep * 0.6, 0)  # the force of it
        p["jaw"] = (38 + 2 * math.sin(t * 33), 0, 0)
    else:
        p = blend(blast, base, ease((t - 2.6) / 0.4))
    return p


def pose_rest(t):
    """Lying down (asleep, resting, knocked out): breathing slowly, the tail tip stirring."""
    p = rest_pose(FIT["rest"])
    br = math.sin(t / 5.0 * 2 * math.pi)
    p["spine_1"] = (br * 0.8, 0, 0)
    p["neck_0"] = (p["neck_0"][0] - br * 0.6, 0, 0)
    for k in (6, 7):
        p[f"tail_{k}"] = (p[f"tail_{k}"][0], p[f"tail_{k}"][1] + 4 * math.sin(t / 5.0 * 2 * math.pi * 2 - k), 0)
    p["scale"] = mirror({"scale": fan_closed()["scale"]})["scale"]
    return p


# ------------------------------------------------------------------ attacks
#
# Played by the game when it strikes (`commands::fight`), one at random: a bite, a claw (a wing-arm
# lifted and brought down on what's ahead, solved like the gaits so the claw lands on the ground
# it strikes), a lash of the tail.

def pose_bite(t):
    """The neck draws back, then lunges forward and down with the jaws wide, snapping shut; the
    body leans into it."""
    p = t / 0.9
    q = stand()
    back = {"neck_0": -36, "neck_1": -20, "neck_2": 4, "neck_3": 14, "head": 14}
    lunge = {"neck_0": -8, "neck_1": -2, "neck_2": 6, "neck_3": 8, "head": 6}
    d = float(np.interp(p, [0, .3, .5, .65, 1], [0, -1, 1, 1, 0]))  # -1 drawn back, 1 lunging
    for b, v in back.items() if d < 0 else lunge.items():
        q[b] = (q[b][0] + (v - q[b][0]) * abs(d), 0, 0)
    q["spine_1"] = (4 * d, 0, 0)
    q["jaw"] = (float(np.interp(p, [0, .3, .48, .56, 1], [0, 20, 36, 0, 0])), 0, 0)
    return q


CLAW_KEYS = ([0, .35, .55, .7, 1], [(0, 0, 0), (25, 150, 40), (0, 3, 130), (0, 3, 130), (0, 0, 0)])  # out, up, ahead


def pose_claw(t):
    """A wing-arm's strike: the left wrist lifts high and comes down on what's ahead, the claw
    hitting the ground there; the shoulders lean into it and the head follows."""
    p = t / 1.0
    q = stand()
    keys, offs = CLAW_KEYS
    off = np.array([float(np.interp(p, keys, [o[k] for o in offs])) for k in range(3)])
    lean = float(np.interp(p, [0, .35, .55, 1], [0, -1, 1, 0]))
    q["spine_1"] = (4 * lean, 0, -3 * max(lean, 0))
    body = {k: v for k, v in q.items() if k != "scale"}
    target = neutral_feet()["wing"] + off
    wing, _ = solve_limb(body, "wing", target, 0.0, None)
    q.update(wing)
    q["head"] = (q["head"][0] + 6 * max(lean, 0), -8 * max(lean, 0), 0)
    q["jaw"] = (12 * max(lean, 0), 0, 0)
    return q


def pose_tail(t):
    """A lash of the tail: it swings out to one side, then whips across to the other in a wave
    running down it, the hips turning against it; the head looks back."""
    p = t / 1.0
    q = stand()
    swing = float(np.interp(p, [0, .3, .55, .75, 1], [0, 1, -1, -0.6, 0]))  # +: out to the left
    for k in range(len(TAIL)):
        lag = float(np.interp(p - 0.04 * k, [0, .3, .55, .75, 1], [0, 1, -1, -0.6, 0]))
        q[f"tail_{k}"] = (q[f"tail_{k}"][0], 9 * lag, 0)
    q["spine_0"] = (0, -6 * swing, 0)
    q["neck_3"] = (q["neck_3"][0], -10 * swing, 0)
    q["head"] = (q["head"][0], -12 * swing, 0)
    return q


BLINKS = {}
ANIMS = {"idle": (pose_idle, 4.0), "walk": (pose_gait("walk"), GAITS["walk"]["length"]),
         "run": (pose_gait("run"), GAITS["run"]["length"]), "breathe": (pose_breathe, 3.0), "rest": (pose_rest, 5.0),
         "bite": (pose_bite, 0.9), "claw": (pose_claw, 1.0), "tail": (pose_tail, 1.0),
         "fly": (pose_fly, 1.0), "glide": (pose_glide, 3.0)}


def frame_times(name, dur):
    n = 24 if name in ("fly",) else 32
    return [round(dur * k / n, 4) for k in range(n + 1)]


def animations(bone_names):
    out = {}
    for name, (fn, dur) in ANIMS.items():
        frames = []
        for t in frame_times(name, dur):
            pose = fn(t % dur if t < dur else 0.0)
            frames.append({"t": t, "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))],
                           **({"scale": {b: [float(v) for v in sc] for b, sc in pose["scale"].items()}} if "scale" in pose else {})})
        out[name] = {"length": dur, "frames": frames}
    return out


FIT_FILE = os.path.join(HERE, "fit.json")


def fit_poses(spec):
    """`fit_wing` once (minutes), then from `fit.json` until the geometry changes: rerun with
    `--refit` after moving the wing's or legs' bones."""
    if os.path.exists(FIT_FILE) and "--refit" not in sys.argv:
        FIT.update(json.load(open(FIT_FILE)))
    else:
        fit_wing(spec)
    if "rest" not in FIT or "--refit" in sys.argv or "--refit-rest" in sys.argv:
        fit_rest(spec)
    with open(FIT_FILE, "w") as f:
        json.dump(FIT, f, indent=1)


REST_CURL = 9  # degrees each tail segment turns to the side lying down, curling it round
REST_NAMES = ["thigh_L", "shin_L", "foot_L", "neck_0", "neck_1", "neck_2", "neck_3", "head"] + [f"tail_{k}" for k in range(len(TAIL))]


def rest_pose(x, curl=REST_CURL):
    """The lying pose from the fitted numbers: how far the body drops, the legs' and neck's and
    tail's pitches (the toes flat, the tail curled to the side), the wings folded."""
    p = {"root": (0, -x[0], 0)}
    th, sh, ft = x[1:4]
    p.update({"thigh_L": (th, 0, 0), "shin_L": (sh, 0, 0), "foot_L": (ft, 0, 0), "toes_L": (-(th + sh + ft), 0, 0)})
    for i, n in enumerate(REST_NAMES[3:8]):
        p[n] = (x[4 + i], 0, 0)
    for k in range(len(TAIL)):
        p[f"tail_{k}"] = (x[9 + k], curl if k else curl / 2, 0)
    fold = {k: v for k, v in folded_wing().items() if k != "scale"}
    # The folded wing raised and drawn in to the body as it lies down (the arm and forearm turned
    # from the stand's fold).
    fold["arm_L"] = tuple(np.array(fold["arm_L"]) + x[17:20])
    fold["forearm_L"] = tuple(np.array(fold["forearm_L"]) + x[20:23])
    p.update(fold)
    return mirror(p)


def fit_rest(spec):
    """Lying down (resting, asleep, knocked out): the belly on the ground, the legs folded under
    it with the toes flat, the neck lowered and the head resting level on the ground ahead, the
    tail lying along the ground curled to one side, nothing below the ground."""
    from scipy.optimize import minimize
    from ground import lowest

    def starts(n):
        return lambda name: name.startswith(n)
    groups = {"keel": starts("keel"), "toes": starts(("toe", "hind")), "jaw": starts("jaw"),
              "solid": lambda n: not n.startswith("mem")}
    groups.update({f"tail_{k}": (lambda k: lambda n: n == f"tail_{k}")(k) for k in range(len(TAIL))})

    def cost(x):
        lo = lowest(spec, rest_pose(x), groups)
        e = 50 * min(0.0, lo["solid"] + 0.3) ** 2
        e += (lo["keel"] - 1) ** 2 + (lo["toes"] - 0.5) ** 2 + 0.5 * (lo["jaw"] - 1) ** 2
        e += sum(0.3 * (lo[f"tail_{k}"] - 1) ** 2 for k in range(2, len(TAIL)))
        e += 0.02 * (sum(x[4:9])) ** 2  # the head level with the ground
        # Joints in their natural range: the thighs forward as a resting bird folds them, the
        # neck and tail bending gently, the wings turned no more than they must.
        def beyond(v, lim):
            return max(0.0, abs(v) - lim) ** 2
        e += 0.5 * max(0.0, x[1] + 10) ** 2
        e += 0.2 * sum(beyond(v, 40) for v in x[4:17]) + 0.2 * sum(beyond(v, 50) for v in x[17:23])
        return e + 0.0005 * float(np.sum(np.asarray(x[1:]) ** 2)) + 0.002 * float(np.sum(np.asarray(x[17:]) ** 2))

    x0 = [150, -60, 120, -50, 20, 10, 0, -10, -10, -10, -6, -4, 0, 0, 0, 0, 0] + [0, 0, 20, 0, 0, 0]
    r = minimize(cost, x0, method="Powell", options={"maxiter": 60000, "xtol": 1e-2, "ftol": 1e-6})
    FIT["rest"] = [round(float(v), 2) for v in r.x]
    FIT["rest_cost"] = round(float(r.fun), 2)
    lo = lowest(spec, rest_pose(r.x), groups)
    print("rest:", {k: round(float(v), 1) for k, v in lo.items() if not k.startswith("tail_")})


def fit_wing(spec):
    """The folded wing, as the show's dragons fold theirs (a tall pleated sail beside the body):
    the wrist (where the thumb's claw is) on the ground ahead of the chest and out from it, the
    forearm rising steeply to the elbow at the height of the back, the fingers fanned up and back
    from the wrist over the flank toward the hips, the membrane pleated between them; and the
    stand's drop (toes on the ground)."""
    from scipy.optimize import minimize
    from ground import corners, lowest, world

    bones_ = spec["bones"]
    piv = {b["name"]: np.array(b["pivot"], float) for b in bones_}
    groups = {"toes": lambda n: n.startswith(("toe", "hind"))}
    base = {}
    base.update(legs_crouch())
    base.update(neck_s())
    base = mirror(base)
    FIT["drop"] = round(float(lowest(spec, base, groups)["toes"]), 2)
    base.update(mirror({k: v for k, v in fan_closed().items() if k != "scale"}))  # the fingers half closed

    def at(pose, bone, local):
        R, t = world(bones_, pose)[bone]
        return R @ (np.array(local, float) - piv[bone]) + t

    elbow_rest = piv["forearm_L"]
    claw_rest = THUMB_TIP  # the thumb claw's tip at rest
    ground = FIT["drop"] + 6
    # Planted wide, the forearm sloping out from the elbow to the ground (an A from the front,
    # strong and broad), the elbow by the shoulder at the height of the back.
    wrist_t = np.array([240.0, ground + 4, 215.0])
    elbow_t = np.array([140.0, 292.0, 104.0])
    # The open fan stood up beside the body: the first finger rising nearly straight up from the
    # wrist (the sail's top about one and a half times the back's height, as the show's), the last
    # running back toward the hips, the fan leaning a little out.
    ups = {0: np.array([-0.12, 0.82, -0.57]), len(FINGERS) - 1: np.array([0.0, 0.17, -0.98])}
    tips_t = {k: wrist_t + d / np.linalg.norm(d) * (sum(FINGERS[k][1]) - 10) for k, d in ups.items()}
    names = ["arm_L", "forearm_L", "hand_L"]

    def wing(x):
        return {n: (x[3 * i], x[3 * i + 1], x[3 * i + 2]) for i, n in enumerate(names)}

    # The arm's, hand's and fingers' corners: none below the ground (the claw's tip on it, the
    # hand turned over above it).
    solid = [(e["bone"], c) for e in spec["elements"] if e["bone"].endswith("_L")
             and e["bone"].startswith(("forearm", "hand", "thumb", "finger")) for c in corners(e)]

    def cost(x):
        pose = dict(base)
        pose.update(wing(x))
        W = world(bones_, pose)
        low = min((W[b][0] @ (c - piv[b]) + W[b][1])[1] for b, c in solid)
        e = 30 * min(0.0, low - FIT["drop"]) ** 2
        e += np.sum((at(pose, "thumb_L", claw_rest) - wrist_t) ** 2)
        e += 0.6 * np.sum((at(pose, "forearm_L", elbow_rest) - elbow_t) ** 2)
        fps = finger_points(1)
        for k, t in tips_t.items():
            e += 0.4 * np.sum((at(pose, f"finger{k}b_L", fps[k][2]) - t) ** 2)
        return e + 0.01 * np.sum(np.array(x) ** 2)

    best = None
    rng = np.random.default_rng(1)
    for start in range(8):
        x0 = rng.uniform(-120, 120, 3 * len(names)) if start else np.zeros(3 * len(names))
        r = minimize(cost, x0, method="Powell", options={"maxiter": 40000, "xtol": 1e-3, "ftol": 1e-5})
        if best is None or r.fun < best.fun:
            best = r
    r = best
    FIT["wing"] = {k: [round(float(v), 2) for v in val] for k, val in wing(r.x).items()}
    FIT["wing_cost"] = round(float(r.fun), 2)


# ------------------------------------------------------------------ build

def spec_of(E, uv, sheet):
    B = bones()
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    keep = ("name", "bone", "lo", "hi", "quad", "rot", "origin", "faces")
    return {"name": "wyvern", "texture": [sheet.shape[1], sheet.shape[0]],
            "bones": [{"name": n, "parent": p, "pivot": [float(v) for v in pv]} for n, p, pv in B],
            "elements": [{k: e.get(k) for k in keep} for e in E], "animations": {}}


def build():
    E = elements()
    from layers import Recorder, bounds as layer_bounds
    rec = Recorder(layer_bounds(E, 3.0))
    sheet, uv, keys = paint_sheet(E, rec=rec)
    from paint import pack
    rec.save(OUT, keys, lambda tiles, width: pack(tiles, 1024))
    spec = spec_of(E, uv, sheet)
    fit_poses(spec)
    SKELETON[:] = spec["bones"]
    global HAND_LOW
    hand = next(e for e in spec["elements"] if e["name"] == "hand_L")
    HAND_LOW = [(x, y, z) for x in (hand["lo"][0], hand["hi"][0]) for y in (hand["lo"][1], hand["hi"][1]) for z in (hand["lo"][2], hand["hi"][2])]
    spec["animations"] = animations([b["name"] for b in spec["bones"]])
    spec["fitted"] = dict(FIT)
    return spec, sheet


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    spec, sheet = build()
    Image.fromarray(sheet, "RGBA").save(os.path.join(OUT, "texture.png"))
    with open(os.path.join(OUT, "spec.json"), "w") as f:
        json.dump(spec, f)
    print(len(spec["elements"]), "elements;", len(spec["bones"]), "bones; texture", sheet.shape[1], "x", sheet.shape[0],
          "; fitted", spec["fitted"])
