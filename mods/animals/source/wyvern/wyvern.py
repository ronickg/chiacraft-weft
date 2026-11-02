#!/usr/bin/env python3
"""The wyvern: a huge winged dragon in the style of the show dragons (Game of Thrones), our own
design, built with the blocky-model skill in the house style (64 units a meter; see README.md for
the research and the decisions).

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
from house import material  # noqa: E402
from prop import FACES, el, paint, rotm  # noqa: E402
from rig import euler  # noqa: E402

OUT = os.environ.get("WYVERN_OUT") or os.path.join(HERE, "out")

# ------------------------------------------------------------------ materials

BACK, FLANK, BELLY, MEMBRANE, VEIN, HORN, CLAW, TEETH, MOUTH, EYE, SPIKE = range(11)
MATERIALS = [
    material("scale", (58, 54, 56)),      # back: dark slate scales
    material("scale", (82, 74, 70)),      # flanks
    material("scale", (148, 120, 92)),    # belly plates: warm tan
    material("skin", (92, 62, 58)),       # wing membrane: dark leathery red-brown
    material("skin", (64, 42, 40)),       # its veins
    material("horn", (196, 182, 156)),
    material("claw"),
    material("horn", (226, 220, 200)),    # teeth
    material("skin", (70, 32, 30)),       # inside the mouth
    material("skin", (232, 150, 40)),     # eyes: amber
    material("scale", (40, 36, 38)),      # spikes: darker than the back
]

# ------------------------------------------------------------------ the model (units: 64 a meter)

HIP = np.array([0.0, 235.0, 0.0])       # spine_0's joint, inside the hips
CHEST = np.array([0.0, 240.0, 45.0])    # spine_1's joint, where hips and chest overlap
NECK_BASE = np.array([0.0, 262.0, 160.0])
NECK = [(50, 70, 74), (50, 62, 66), (50, 55, 59), (50, 49, 53)]  # length, width, height
TAIL = [(64, 82, 76), (62, 72, 66), (60, 62, 56), (58, 52, 47), (56, 43, 39), (52, 35, 31), (48, 27, 24), (48, 20, 17)]
TAIL_BASE = np.array([0.0, 236.0, -50.0])
SHOULDER = np.array([52.0, 270.0, 140.0])
# Fingers: rest yaw (degrees, + toward the front), segment lengths, widths; leading edge first.
FINGERS = [(12, (120, 104), (12, 9)), (-28, (110, 90), (10, 7)), (-62, (96, 76), (8, 5))]
LEG_X = 62  # the hind legs' middle, out from the hips (52 a side)
WRIST = np.array([306.0, 270.0, 140.0])  # where the hand's bone turns (inside the hand)
MEM_Y = (268.5, 269.5, 270.5, 271.5)  # the membrane panels' heights: mem0, mem1, memb, memo
BODY_EDGE = (np.array([56.0, 270.0, 130.0]), np.array([40.0, 270.0, -40.0]))  # where the membrane grows from the flank


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
        B += [(f"arm_{n}", "spine_1", tuple(sh)), (f"forearm_{n}", f"arm_{n}", (134 * s, 270, 140)),
              (f"hand_{n}", f"forearm_{n}", tuple(WRIST * np.array([s, 1, 1]))),
              (f"thumb_{n}", f"hand_{n}", (310 * s, 270, 148))]
        for k, (w, mid, _) in enumerate(finger_points(s)):
            B += [(f"finger{k}_{n}", f"hand_{n}", tuple(w)), (f"finger{k}b_{n}", f"finger{k}_{n}", tuple(mid))]
        # The panels between the fingers, each on its own bone at the wrist (a child of the finger
        # it hangs from), so folding swings it in against that finger like a closing fan.
        for k in range(len(FINGERS) - 1):
            B.append((f"mem{k}_{n}", f"finger{k}_{n}", tuple(WRIST * np.array([s, 1, 1]) + np.array([0, MEM_Y[k] - 270, 0]))))
        # The membrane from the last finger to the body, in two: the inner panel on the body
        # (turning about the shoulder-to-hip line, where it grows from the flank), the outer one on
        # the last finger (closing with the fan).
        B += [(f"memb_{n}", "spine_1", tuple(BODY_EDGE[0] * np.array([s, 1, 1]) + np.array([0, MEM_Y[2] - 270, 0]))),
              (f"memo_{n}", f"finger{len(FINGERS) - 1}_{n}", tuple(WRIST * np.array([s, 1, 1]) + np.array([0, MEM_Y[3] - 270, 0])))]
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
        el("hips", (-52, 186, -56), (52, 284, 50), bone="spine_0", part="body"),
        el("chest", (-64, 166, 38), (64, 294, 172), bone="spine_1", part="body"),
        # The flight muscles' hump over the shoulders, and the keel under the chest.
        el("shoulders", (-50, 288, 100), (50, 304, 168), bone="spine_1", part="body"),
        el("keel", (-34, 152, 70), (34, 174, 160), bone="spine_1", part="body"),
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
            el(f"arm_{n}", tuple(np.array([40, 252, 122]) * S), tuple(np.array([140, 288, 158]) * S), bone=f"arm_{n}", part="arm"),
            el(f"forearm_{n}", tuple(np.array([130, 257, 127]) * S), tuple(np.array([310, 283, 153]) * S), bone=f"forearm_{n}", part="arm"),
            el(f"hand_{n}", tuple(np.array([298, 262, 130]) * S), tuple(np.array([320, 278, 150]) * S), bone=f"hand_{n}", part="arm"),
            el(f"thumb_{n}", tuple(np.array([304, 263, 144]) * S), tuple(np.array([316, 277, 176]) * S), bone=f"thumb_{n}", part="arm"),
            el(f"thumbclaw_{n}", tuple(np.array([306.5, 264, 174]) * S), tuple(np.array([313.5, 273, 190]) * S), bone=f"thumb_{n}",
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
    T = (fp[2][2] + H_) / 2  # halfway along the trailing edge from the last finger to the hip
    shapes = [("mem0", f"mem0_{n}", [fp[0][0], fp[0][2], fp[1][2]]),
              ("mem1", f"mem1_{n}", [fp[1][0], fp[1][2], fp[2][2]]),
              ("memb", f"memb_{n}", [S_, M, T, H_]),
              ("memo", f"memo_{n}", [M, fp[2][0], fp[2][2], T])]
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


def area(e, face, p, uu, vv, h):
    part = e.get("part")
    shape = p.shape[:-1]
    x, y = p[..., 0], p[..., 1]
    if part == "membrane":
        return np.full(shape, MEMBRANE)
    if part in ("horn",):
        return np.full(shape, HORN)
    if part == "claw":
        return np.full(shape, CLAW)
    if part == "spike":
        return np.full(shape, SPIKE)
    if part == "mouth":
        return np.full(shape, MOUTH)
    if part == "finger":
        return np.full(shape, BACK)
    a = np.full(shape, FLANK)
    if face == "up":
        a[:] = BACK
    elif face == "down":
        a[:] = BELLY if part in ("body", "jaw") else FLANK
    elif face in ("east", "west"):
        top = p[..., 1].max()
        a = np.where(y > top - 0.35 * (top - p[..., 1].min()), BACK, FLANK)
    return a


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
    """Each panel swung in against the finger it hangs from (by the angle to the next finger); the
    outer body panel closed onto the last finger; the inner one turned down about the flank line
    and drawn in to a sliver inside the flank, as a bat's skin contracts when the wing closes
    (hanging at full size it would cut through the thighs)."""
    out = {f"mem{k}_L": (0, -(FINGERS[k][0] - FINGERS[k + 1][0]) * 0.92, 0) for k in range(len(FINGERS) - 1)}
    out["memo_L"] = (0, -100, 0)
    S_, H_ = BODY_EDGE
    u = (H_ - S_) / np.linalg.norm(H_ - S_)
    a = math.radians(FOLD_FLANK) / 2
    out["memb_L"] = tuple(round(v, 2) for v in euler(np.array([*(u * math.sin(a)), math.cos(a)])))
    out["scale"] = {"memb_L": (0.04, 1.0, 0.04)}  # toward its pivot at the shoulder, inside the chest
    return out


FOLD_FLANK = 88  # degrees the inner panel turns about the flank line, folded


def folded_wing():
    w = {k: tuple(v) for k, v in FIT.get("wing", {}).items()}
    w.update(fan_closed())
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


def pose_fly(t):
    """A wingbeat (one a second): the downstroke strong and quick, the upstroke with the wing
    half folded, the fingers trailing behind the arm."""
    ph = t * 2 * math.pi
    up = math.cos(ph)  # 1 at the top of the stroke
    fold = max(0.0, math.sin(ph)) * 0.5  # folding on the way up
    p = flying_body()
    arm = (0, 16 * fold, 28 * up + 4)
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
    arm = (0, -2, 8 + 2 * math.sin(ph))
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
CLAW_TIP = np.array([310.0, 268.0, 188.0])  # the thumb claw's tip at rest: the wrist's foot
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
            d = at(p, "thumb_L", CLAW_TIP) - target
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
    return {"leg": Rl @ (np.array([LEG_X, 8.0, 0.0]) - piv["toes_L"]) + tl, "wing": Rw @ (CLAW_TIP - piv["thumb_L"]) + tw}


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


BLINKS = {}
ANIMS = {"idle": (pose_idle, 4.0), "walk": (pose_gait("walk"), GAITS["walk"]["length"]),
         "run": (pose_gait("run"), GAITS["run"]["length"]), "breathe": (pose_breathe, 3.0),
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
        return
    fit_wing(spec)
    with open(FIT_FILE, "w") as f:
        json.dump(FIT, f, indent=1)


def fit_wing(spec):
    """The folded wing (the wrist, where the thumb's claw is, on the ground ahead of the chest;
    the fingers folded back up along the forearm, the membrane folded between them) and the
    stand's drop (toes on the ground)."""
    from scipy.optimize import minimize
    from ground import lowest, world

    bones_ = spec["bones"]
    piv = {b["name"]: np.array(b["pivot"], float) for b in bones_}
    groups = {"toes": lambda n: n.startswith(("toe", "hind"))}
    base = {}
    base.update(legs_crouch())
    base.update(neck_s())
    base = mirror(base)
    FIT["drop"] = round(float(lowest(spec, base, groups)["toes"]), 2)

    def at(pose, bone, local):
        R, t = world(bones_, pose)[bone]
        return R @ (np.array(local, float) - piv[bone]) + t

    elbow_rest = piv["forearm_L"]
    claw_rest = np.array([310.0, 268.0, 188.0])  # the thumb claw's tip at rest
    ground = FIT["drop"] + 6
    # Where a bat's folded wing puts things: the wrist (the thumb's claw) on the ground ahead of
    # the chest, the elbow high and back by the shoulder, the fingers folded up from the wrist
    # along the forearm and past the elbow, the membrane tight between them.
    wrist_t = np.array([120.0, ground + 4, 206.0])
    elbow_t = np.array([110.0, 222.0, 118.0])
    up = (elbow_t - wrist_t) / np.linalg.norm(elbow_t - wrist_t)
    tips_t = [wrist_t + up * (sum(f[1]) - 10) + np.array([14 + 6 * k, -12 * k, -26 * k]) for k, f in enumerate(FINGERS)]
    names = ["arm_L", "forearm_L", "hand_L", "finger0_L", "finger1_L", "finger2_L"]

    def wing(x):
        return {n: (x[3 * i], x[3 * i + 1], x[3 * i + 2]) for i, n in enumerate(names)}

    def cost(x):
        pose = dict(base)
        pose.update(wing(x))
        e = np.sum((at(pose, "thumb_L", claw_rest) - wrist_t) ** 2)
        e += 0.6 * np.sum((at(pose, "forearm_L", elbow_rest) - elbow_t) ** 2)
        for k, fp in enumerate(finger_points(1)):
            e += 0.4 * np.sum((at(pose, f"finger{k}b_L", fp[2]) - tips_t[k]) ** 2)
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
    sheet, uv = paint(E, area, MATERIALS, cut=cut, width=1024, seed=5)
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
