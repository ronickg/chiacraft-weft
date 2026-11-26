#!/usr/bin/env python3
"""Birds from the raven's build (`../raven/raven.py`): one layout with knobs, so the small bird,
the owl, the eagle, the vulture and the gull are each a page of numbers and color rules (their own
`<model>.py`) on a build that already passed the review checklist. Made with the blocky-model
skill: boxes, cut-out feather panels and a hand-painted texture.

A bird is laid out in the raven's units (a raven is 40 long), shaped by its knobs (a bigger head,
a shorter or hooked bill, broader or longer wings, a longer tail, wider body, longer or shorter
legs), then warped into its own size (`warp`: the legs below `LEG_Y` stretched by `D`, everything
`K` times bigger). It paints `TX` texels a unit: the house's 32 a block (`house.TX`, 0.5), finer
(64, 128 a block) for a bird whose eye-bearing face would be under 3 texels at 32 (`house.fine`);
each bird's module sets it and says why. One build paints both models on
one texture: standing with the wings folded along the sides, and flying with them open
(`<model>_fly`, a look its mod puts on while it flies).

A bird module sets `CFG` (see `DEFAULTS`) and calls `build(CFG)`; `../bird/export.py` checks and
exports it.
"""

import math
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
SKILL = os.path.normpath(os.path.join(HERE, "..", "..", "..", "..", ".claude", "skills", "blocky-model", "scripts"))
sys.path.insert(0, SKILL)
from layers import Recorder, bounds as layer_bounds  # noqa: E402
from paint import EYES, eye_corner, occlusion, pack, paint_face_smooth, stamp_eyes, tones  # noqa: E402
from house import TX as HOUSE_TX  # noqa: E402
from ground import lowest  # noqa: E402

TAU = 2 * math.pi
S = math.sin
LEG_Y = 8.0  # below this (the raven's units) the legs are stretched

DEFAULTS = dict(
    name="bird",
    K=1.0,  # drawn this many times the raven's size
    D=0.0,  # the legs this much longer (raven units; negative shorter)
    TX=HOUSE_TX,  # texels a unit: 32 a block (0.5); a bird with a small head sets its own (`house.fine`)
    head=1.0,  # the skull's size against the raven's
    head_y=0.0,  # the skull raised (raven units)
    neck=True,  # a neck box between the chest and the head (an owl's head sits on its shoulders)
    neck_w=1.0,  # the neck's thickness against the raven's (a vulture's bare neck is thin)
    bill=dict(w=2.0, h=2.3, len=5.7, rot=6, hook=False, y=1.0),  # its top `y` above the skull's middle
    body_w=1.0,  # the body's width against the raven's
    tail=dict(len=1.0, w=1.0, cut="wedge"),  # wedge, square, round, notch
    wing=dict(arm=1.0, hand=1.0, chord=1.0, tip="fingers", folded=1.0),  # fingers, pointed, round
    shin=0.5,  # the shin's half width (feathered legs thicker)
    toes=1.0,  # the feet's size
    tufts=None,  # ear tufts: (height, width) in raven units
    tilt=-18,  # the body tipped up at the front, standing
    neck_share=0.4,  # how much of the head's leveling the neck does (0: a bird with no neck to speak of)
    eyes=dict(kind="side", pattern="raven", z=12.4, y=17.5, lash=(12, 10, 14)),  # front: x, y on the face
    materials=[],  # (name, rgb, shadow hue, role)
    feather=(),  # the materials that are plumage
    area=None,  # area(e, face, p, uu, vv, h) -> material ids, p in the raven's units
    props=None,  # layer_props(e, face, p, a)
    idle="look",  # look (holds and snaps), owl (the head turned far round), perch
    walk="walk",  # walk, hop
    flap=dict(beat=34, hand=18, length=0.5, glide=0.0),  # glide: share of the cycle held open
    blinks=dict(idle=[0.9, 2.3]),
    idle_len=3.0,
)

G = {}  # the build's geometry, for the area rules and cut-outs (raven units)


def warp(v, c):
    v = np.asarray(v, float)
    y = v[..., 1]
    y2 = np.where(y < LEG_Y, y * (LEG_Y + c["D"]) / LEG_Y, y + c["D"])
    return np.stack([v[..., 0] * c["K"], y2 * c["K"], v[..., 2] * c["K"]], -1)


def unwarp(v, c):
    v = np.asarray(v, float)
    y = v[..., 1] / c["K"]
    y2 = np.where(y < LEG_Y + c["D"], y * LEG_Y / (LEG_Y + c["D"]), y - c["D"])
    return np.stack([v[..., 0] / c["K"], y2, v[..., 2] / c["K"]], -1)


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


# ------------------------------------------------------------------ the layout (raven units)


def geometry(c):
    """The parts' sizes and places from the knobs, in the raven's units."""
    hs, bw = c["head"], c["body_w"]
    T = 1.0 / (c["TX"] * c["K"])  # a texel, in the raven's units
    # The shin two texels thick at least, as painted (a texel thick, it read as a line under the thigh; linecheck).
    shin = max(c["shin"], 0.78 * T)
    thigh = max(1.1, c["shin"] + 0.6, shin + 0.35 / c["K"])  # a size bigger than the shin (a thickened one: a flush step)
    cy, cz = 18.0 + c["head_y"], 11.25
    hx, hy, hz = 2.6 * hs, 2.5 * hs, 2.75 * hs
    skull = ((-hx, cy - hy, cz - hz), (hx, cy + hy, cz + hz))
    front = skull[1][2]
    b = c["bill"]
    btop = cy + b["y"] * hs
    bill = ((-b["w"] / 2, btop - b["h"], front - 0.5), (b["w"] / 2, btop, front - 0.5 + b["len"]))
    jaw_top = bill[0][1] + 0.2
    jaw = ((-(b["w"] / 2 - 0.2), jaw_top - min(1.0, 0.45 * b["h"]), front - 0.8), (b["w"] / 2 - 0.2, jaw_top, front - 0.8 + 0.82 * b["len"]))
    side = 4.0 * bw
    w = c["wing"]
    hand_x = side - 0.4 + 14.2 * w["arm"]
    t = c["tail"]
    eye = c["eyes"]
    return dict(skull=skull, cy=cy, cz=cz, front=front, bill=bill, jaw=jaw, jaw_pivot=(0, jaw_top - 0.3, front - 0.4),
                side=side, hand_x=hand_x, arm_end=side - 0.8 + 15.2 * w["arm"], hand_end=hand_x - 0.4 + 20.6 * w["hand"],
                prim_end=hand_x - 0.2 + 21.4 * w["hand"], flight_z0=2.4 - 9.9 * w["chord"], prim_z0=2.8 - 9.3 * w["chord"],
                tail_end=-5.0 - 14.6 * t["len"], fan_end=-8.0 - 13.0 * t["len"], tail_w=2.2 * t["w"], fan_w=3.6 * t["w"],
                leg_x=min(1.9 * bw, 3.4 * bw - thigh - 0.4 / c["K"]), T=T, shin=shin, thigh=thigh, fold_end=-4.0 - 11.0 * w["folded"], eye=eye,
                fan_z1=-8.0, fold_z1=-4.0, prim_x0=hand_x - 0.2, prim_z1=2.8, fold_y=12.0, fold_h=2.0, u=1.0)  # u: the feathers' wiggles, in raven units


def eye_design(c, face):
    """The eye design stamped on `face`: a side eye's design mirrored on the west face, so both
    sides carry it the same way round (highlight and pupil toward the bill)."""
    name = c["eyes"]["pattern"]
    if c["eyes"]["kind"] != "side" or face != "west":
        return name
    m = name + "~west"
    if m not in EYES:
        EYES[m] = dict(EYES[name], pattern=[row[::-1] for row in EYES[name]["pattern"]])
    return m


def eye_grid(c, face):
    """The skull face that carries the eyes, as `face_frame` cuts it: its texels (w, h), the units
    a texel (su, sv), and the eye centers in its texels [(cu, cv)] (front eyes: left one first)."""
    (x0, y0, z0), (x1, y1, z1) = (warp(v, c) for v in G["skull"])
    TX = c["TX"]
    eyes = c["eyes"]
    ey = warp((0, eyes["y"] + c["head_y"], 0), c)[1]
    across = (z1 - z0) if eyes["kind"] == "side" else (x1 - x0)
    w, h = max(1, int(round(TX * across))), max(1, int(round(TX * (y1 - y0))))
    su, sv = across / w, (y1 - y0) / h
    cv = (y1 - ey) / sv
    if eyes["kind"] == "side":
        ez = warp((0, 0, eyes["z"]), c)[2]
        centers = [(((z1 - ez) if face == "east" else (ez - z0)) / su, cv)]
    else:
        ex = eyes["x"] * c["K"]
        centers = [((-ex - x0) / su, cv), ((ex - x0) / su, cv)]
    return w, h, su, sv, centers


def eye_box(c, s):
    """The eye's texels on the warped skull, exactly as `stamp_eyes` places them (`eye_corner` on
    the face's own texel grid), in warped units: side eyes (z0, z1, top, bottom), front eyes
    (x0, x1, top, bottom)."""
    (x0, y0, z0), (x1, y1, z1) = (warp(v, c) for v in G["skull"])
    if c["eyes"]["kind"] == "side":
        face = "east" if s > 0 else "west"
        w, h, su, sv, centers = eye_grid(c, face)
        cu, cv = centers[0]
    else:
        face = "south"
        w, h, su, sv, centers = eye_grid(c, face)
        cu, cv = centers[0 if s < 0 else 1]
    pat = EYES[eye_design(c, face)]["pattern"]
    ph, pw = len(pat), len(pat[0])
    u0, v0 = eye_corner(cu, cv, pat, w)
    top, bottom = y1 - v0 * sv, y1 - (v0 + ph) * sv
    if c["eyes"]["kind"] == "side":
        za, zb = (z1 - (u0 + pw) * su, z1 - u0 * su) if s > 0 else (z0 + u0 * su, z0 + (u0 + pw) * su)
        return za, zb, top, bottom
    return x0 + u0 * su, x0 + (u0 + pw) * su, top, bottom


def bones(c):
    g = G
    if c.get("layout"):  # a bird laid out in its own units (a small one)
        B = list(c["layout"]["bones"])
    else:
        B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 12.5, -2.0)), ("spine_1", "spine_0", (0, 12.5, 0.6)),
             ("neck", "spine_1", (0.0, 13.6, 6.6)), ("head", "neck", (0.0, 17.0 + 0.5 * c["head_y"], 9.6)), ("jaw", "head", g["jaw_pivot"]),
             ("tail", "spine_0", (0, 13.2, -6.4))]
        for s, n in ((1, "L"), (-1, "R")):
            # The folded wing hinged at its shoulder, on the chest's side a unit under its top and before its
            # middle, so it opens (rolled out, swung out behind, raised) away from the body: only the band
            # above and before the hinge turns into the chest, as a shoulder does (at (side - 0.1, 14.6, 5.0)
            # the wing's whole top went into it; at its top front corner, (side, 16.6, 7.0), linecheck no
            # longer reads the chest's strip beside the wing as the rim of a joint).
            B += [(f"wingf_{n}", "spine_1", (s * g["side"], 15.6, 6.0)),
                  (f"wing_{n}", "spine_1", (s * (g["side"] - 0.4), 14.9, 5.0)), (f"hand_{n}", f"wing_{n}", (s * g["hand_x"], 14.9, 5.0)),
                  (f"leg_{n}_thigh", "spine_0", (s * g["leg_x"], 10.0, -0.6)), (f"leg_{n}_shin", f"leg_{n}_thigh", (s * g["leg_x"], 6.4, -0.6)),
                  (f"leg_{n}_toes", f"leg_{n}_shin", (s * g["leg_x"], 1.0, -0.6))]
            if c["tufts"]:
                (x0, y0, z0), (x1, y1, z1) = g["skull"]
                B.append((f"tuft_{n}", "head", (s * 0.6 * x1, y1 - 0.3, z1 - 0.45 * (z1 - z0))))
        B = [(n, p, tuple(float(v) for v in warp(pv, c))) for n, p, pv in B]
    for s, n in ((1, "L"), (-1, "R")):
        a0, a1, top, _ = eye_box(c, s)
        if c["eyes"]["kind"] == "side":
            B.append((f"eyelid_{n}", "head", (s * G["lid"], top, (a0 + a1) / 2)))
        else:
            B.append((f"eyelid_{n}", "head", ((a0 + a1) / 2, top, G["lid"])))
    return B


def elements(c):
    if c.get("layout"):
        E = [dict(e) for e in c["layout"]["elements"]]
    else:
        E = raven_elements(c)
    add_eyelids(c, E)
    return E


def raven_elements(c):
    g = G
    bw, t = c["body_w"], c["tail"]
    T = g["T"]
    two = 0.78 * T  # half of a part two texels thick as painted (a face 1.56 texels across paints 2)
    E = [el("hips", "spine_0", (-3.4 * bw, 9.0, -7.2), (3.4 * bw, 16.0, 1.4)),
         el("chest", "spine_1", (-4.0 * bw, 8.4, -0.4), (4.0 * bw, 16.6, 8.4))]
    if c["neck"]:
        nw, nz = 2.0 * c["neck_w"], 7.7 + (1 - c["neck_w"])  # a thin neck still holds both its joints
        if g["side"] - nw < 2.05 * T:  # the chest's top beside it two texels wide (under two, a line along the neck; linecheck)
            nw = min(nw, g["side"] - 2.1 * T)
        bottom = g["skull"][0][1]  # its throat, before the chest and under the head, two texels deep (1.85 read as a strip; linecheck)
        # Where it shows over the chest's top (16.6) and before its front (8.4), flush or two texels out (at
        # 18.2 and 10.4, 1-1.75 texels read as a rim round the neck's root and a strip of throat; linecheck).
        y0, y1, z0, z1 = c.get("neck_box") or (min(12.4, bottom - 2.1 * T), 16.6 + clear(1.6, c, T), nz - 2.7 * c["neck_w"],
                                               8.4 + clear(nz + 2.7 * c["neck_w"] - 8.4, c, T))
        E.append(el("neck", "neck", (-nw, y0, z0), (nw, y1, z1)))  # deep in the head
    E += [el("skull", "head", *g["skull"]),
          el("bill", "head", *g["bill"], rot=(c["bill"]["rot"], 0, 0), origin=(0, (g["bill"][0][1] + g["bill"][1][1]) / 2, g["front"] - 0.5)),
          el("jaw", "jaw", *g["jaw"]),  # shorter than the bill
          # Two texels thick at least (a texel thick, its sides read as a line out of the hips; linecheck).
          el("tail", "tail", (-g["tail_w"], 13.2 - max(0.6, 0.4 / c["K"], two), g["tail_end"]), (g["tail_w"], 13.2 + max(0.6, 0.4 / c["K"], two), -5.0)),
          el("tail_fan", "tail", (-g["fan_w"], 13.2, g["fan_end"]), (g["fan_w"], 13.2, -8.0), quad="up", cut="tail")]
    if c["bill"]["hook"]:  # the hooked tip, turned down over the jaw
        (bx0, by0, bz0), (bx1, by1, bz1) = g["bill"]
        hw = (bx1 - bx0) / 2 - 0.25
        E.append(el("bill_hook", "head", (-hw, by1 - 0.4 - 1.6 * (by1 - by0) / 2.3, bz1 - 1.1), (hw, by1 - 0.4, bz1 - 0.1),
                    rot=(c["bill"]["rot"] + 38, 0, 0), origin=(0, by1 - 0.4, bz1 - 0.6)))
    for s, n in ((1, "L"), (-1, "R")):
        side = g["side"]
        # On the chest's side, not in it (from side - 0.4, raised flush with the back, it hid the chest's top
        # but for a strip under two texels beside the neck; linecheck).
        wt = max(0.9, 2 * two)  # two texels thick (no boxes a texel thick: at 0.9 it was 0.6-0.9 texels; SKILL.md)
        x0, x1 = sorted((s * side, s * (side + wt)))
        # Folded along the side, the primaries crossing over the tail's base.
        # Its top just over the chest's (16.6): flush (at 15.9 the wing's top, a texel wide beside the chest,
        # read as a line along the back; linecheck).
        E.append(el(f"wingf_{n.lower()}", f"wingf_{n}", (x0, 10.6, -6.0), (x1, 16.6 + 0.45 / c["K"], 7.0)))
        px = s * (side + wt / 2)  # mid-wing: clear of its inner face, now on the chest's side (at + 0.3 they flickered)
        E.append(el(f"primariesf_{n.lower()}", f"wingf_{n}", (px, 10.4, g["fold_end"]), (px, 14.4, -4.0), quad="east" if s > 0 else "west",
                    cut="folded"))
        # Open: the arm, the hand, and the flight feathers behind them as cut-out panels.
        ax0, ax1 = sorted((s * (side - 0.8), s * g["arm_end"]))
        # The hand two texels thick at least (no boxes a texel thick: thinner, its front and back read as lines
        # beside the arm; linecheck), and the arm round it with a flush step, so the arm too is two texels
        # and more (thinner, its front and back read as lines beside the chest).
        th = max(0.45, 0.4 / c["K"], two)
        ta = max(0.9, 0.42 / c["K"], 2 * two / 1.95, (th + 0.35 / c["K"]) / 0.95)  # thick enough round its feather panel
        E.append(el(f"wing_{n.lower()}", f"wing_{n}", (ax0, 14.9 - ta, 1.8), (ax1, 14.9 + ta * 0.95, 8.0)))
        hx0, hx1 = sorted((s * (g["hand_x"] - 0.4), s * g["hand_end"]))
        E.append(el(f"hand_{n.lower()}", f"hand_{n}", (hx0, 14.9 - th, 2.2), (hx1, 14.9 + th, 7.0)))
        fx0, fx1 = sorted((s * (side - 0.4), s * (g["hand_x"] - 0.4)))
        E.append(el(f"flight_{n.lower()}", f"wing_{n}", (fx0, 14.9, g["flight_z0"]), (fx1, 14.9, 2.4), quad="down", cut="secondaries"))
        gx0, gx1 = sorted((s * (g["hand_x"] - 0.2), s * g["prim_end"]))
        E.append(el(f"primaries_{n.lower()}", f"hand_{n}", (gx0, 14.9, g["prim_z0"]), (gx1, 14.9, 2.8), quad="down", cut="primaries"))
        x = s * g["leg_x"]
        sh, tf = g["shin"], c["toes"]
        th = g["thigh"]  # the thigh a size bigger than the shin
        E.append(el(f"thigh_{n.lower()}", f"leg_{n}_thigh", (x - th, 5.9, -2.0 - 2 * (th - 1.1)), (x + th, 11.0, 0.8)))  # feathered
        E.append(el(f"shin_{n.lower()}", f"leg_{n}_shin", (x - sh, 0.8, -0.6 - sh * 1.0), (x + sh, 6.6, -0.6 + sh * 1.0)))
        # Round the shin each edge of the toes flush with it or out two texels or more (a texel and a half
        # out, the toes' top read as a line round the shin; linecheck).
        ox, of, ob = (clear(o, c, T, out) for o, out in ((1.1 * tf - sh, False), (3.2 * tf - sh, True), (2.0 * tf - sh, True)))
        toe_h = max(1.2, 2 * two * LEG_Y / (LEG_Y + c["D"]))  # two texels tall, as warped (no boxes a texel thick; SKILL.md)
        E.append(el(f"toes_{n.lower()}", f"leg_{n}_toes", (x - sh - ox, 0.0, -0.6 - sh - ob), (x + sh + ox, toe_h, -0.6 + sh + of)))
        if c["tufts"]:
            th, tw = c["tufts"]
            (sx0, sy0, sz0), (sx1, sy1, sz1) = g["skull"]
            cx, cz = s * 0.6 * sx1, sz1 - 0.45 * (sz1 - sz0)
            E.append(el(f"tuft_{n.lower()}", f"tuft_{n}", (cx - tw / 2, sy1 - 0.8, cz - 0.6), (cx + tw / 2, sy1 + th, cz + 0.6),
                        rot=(-16, 0, -s * 14), origin=(cx, sy1 - 0.3, cz)))
    for e in E:  # the raven's space, warped into this bird's
        e["lo"], e["hi"] = [float(v) for v in warp(e["lo"], c)], [float(v) for v in warp(e["hi"], c)]
        if e["origin"]:
            e["origin"] = [float(v) for v in warp(e["origin"], c)]
    return E


def clear(o, c, T, out=True):
    """How far a box's edge stands out from the box it lies in (raven units; `T` a texel): as asked
    when that is flush (under 0.6 units: at least 0.3 apart, or they flicker) or two texels or more;
    a step between would show a strip under two texels, a thin line (linecheck): then flush, or
    with `out` two texels out."""
    if 0.3 <= o * c["K"] < 0.6 or o >= 2.05 * T:
        return o
    return 2.1 * T if out and o * c["K"] >= 0.6 else 0.45 / c["K"]


def add_eyelids(c, E):
    for s, n in ((1, "L"), (-1, "R")):
        # The eyelid: a quad the eye's size lying flat inside the head, hinged at the eye's top; a
        # blink swings it down over the eye. Tilted a few degrees about its hinge, so it is never
        # in a face's plane.
        a0, a1, top, bottom = eye_box(c, s)
        h = top - bottom
        if c["eyes"]["kind"] == "side":
            lx0, lx1 = (G["lid"] - h, G["lid"]) if s > 0 else (-G["lid"], -G["lid"] + h)
            E.append(el(f"eyelid_{n}", f"eyelid_{n}", (lx0, top, a0), (lx1, top, a1), quad="down", rot=(0, 0, s * 4),
                        origin=(s * G["lid"], top, (a0 + a1) / 2)))
        else:
            E.append(el(f"eyelid_{n}", f"eyelid_{n}", (a0, top, G["lid"] - h), (a1, top, G["lid"]), quad="down", rot=(-4, 0, 0),
                        origin=((a0 + a1) / 2, top, G["lid"])))


FOLDED = lambda n: n.startswith(("wingf_", "primariesf_"))  # noqa: E731
OPEN = lambda n: n.startswith(("wing_", "hand_", "flight_", "primaries_"))  # noqa: E731

# ------------------------------------------------------------------ painting

FACES = {"north": (2, -1), "south": (2, 1), "west": (0, -1), "east": (0, 1), "down": (1, -1), "up": (1, 1)}


def face_frame(lo, hi, face, TX):
    lo, hi = np.array(lo), np.array(hi)
    X, Y, Z = np.eye(3)
    corner, du, dv = {"south": ((lo[0], hi[1], hi[2]), X, -Y), "north": ((hi[0], hi[1], lo[2]), -X, -Y),
                      "east": ((hi[0], hi[1], hi[2]), -Z, -Y), "west": ((lo[0], hi[1], lo[2]), Z, -Y),
                      "up": ((lo[0], hi[1], lo[2]), X, Z), "down": ((lo[0], lo[1], hi[2]), X, -Z)}[face]
    size = hi - lo
    ax, sgn = FACES[face]
    n = np.zeros(3)
    n[ax] = sgn
    return (np.array(corner, float), du, dv, max(1, int(round(TX * abs(size @ du)))), max(1, int(round(TX * abs(size @ dv)))), n)


def cut_mask(e, p, c):
    """Feather outlines (in the raven's units): the open wing's primaries (fingered, pointed or
    round), the secondaries' scalloped trailing edge, the folded primaries' tapering tips, the
    tail's end."""
    g = G
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    kind = e["cut"]
    tex = 1.0 / (c["TX"] * c["K"])  # a texel, in the raven's units: wiggles narrower than a few alias into noise
    if kind == "tail":
        L = g["fan_z1"] - g["fan_end"]
        tt = np.clip((g["fan_z1"] - z) / L, 0, 1)  # along the tail, 1 at the tip
        style = c["tail"]["cut"]
        fw = g["fan_w"]
        if style == "wedge":
            return np.abs(x) <= fw * (0.55 + 0.4 * np.sin(np.pi * np.clip(tt * 0.85, 0, 1))) - 1.6 * g["u"] * np.clip(tt - 0.8, 0, 1) * 5
        if style == "round":
            return (np.abs(x) <= fw * (0.6 + 0.35 * tt)) & (tt <= 1 - 0.18 * (np.abs(x) / fw) ** 2)
        if style == "notch":
            return (np.abs(x) <= fw * (0.55 + 0.4 * tt)) & (tt <= 0.9 + 0.1 * np.abs(x) / fw)
        ends = np.floor(np.abs(x) / max(g["u"] / 1.3, 1.5 * tex)) % 2  # square, feather ends: each at least 1.5 texels wide, a texel deep
        return (np.abs(x) <= fw * (0.6 + 0.4 * tt)) & (tt <= 0.97 - max(0.06, tex / L) * ends)
    if kind == "secondaries":
        u = g["u"]
        scallop = 0.7 * u * np.abs(np.sin(np.abs(x) * min(1.3 / u, np.pi / (3 * tex)))) * c["wing"]["chord"]  # each scallop 3+ texels
        return z >= g["flight_z0"] + 2.0 * u * c["wing"]["chord"] * np.clip((np.abs(x) - (g["hand_x"] - 6.0 * u)) / (6.0 * u), 0, 1) + scallop
    if kind == "primaries":
        ax = np.abs(x)
        u = g["u"]
        wig = np.abs(np.sin(ax * min(1.1 / u, np.pi / (3 * tex)))) * u
        x0, x1 = g["prim_x0"], g["prim_end"]
        span = x1 - x0
        t = np.clip((ax - x0) / span, 0, 1)
        z0, z1 = g["prim_z0"], g["prim_z1"]
        chord = z1 - z0
        tip = c["wing"]["tip"]
        if tip == "pointed":  # a gull's: the trailing edge sweeping forward to a point
            trail = z0 + chord * 0.92 * t ** 1.4 + 0.5 * wig
            return (z >= trail) & (ax <= x1 - 0.4 * u * np.clip((z - (z1 - 2.0 * u)) / (2.0 * u), 0, 1))
        if tip == "round":  # an owl's or a sparrow's: a rounded hand
            trail = z0 + chord * 0.35 * t ** 2 + 0.5 * wig
            reach = x1 - span * 0.25 * np.clip((z0 + chord * 0.4 - z) / (chord * 0.6), 0, 1) ** 2
            return (z >= trail) & (ax <= reach)
        trail = z0 + chord * 0.32 * t + 0.6 * wig
        # The fingers: gaps between the outermost feathers near the tip.
        fw = max(1.6 * u * c["wing"]["chord"], 2 * tex)  # a finger and a gap 2+ texels each
        fingers = (t > 0.72) & (np.floor((z - z0) / fw) % 2 == 1) & (ax > x1 - 5.0 * u * (z - z0) / chord)
        end = ax <= x1 - 3.0 * u * np.clip((z - (z1 - 1.8 * u)) / (1.8 * u), 0, 1)
        return (z >= trail) & end & ~fingers
    if kind == "folded":
        L = g["fold_z1"] - g["fold_end"]
        tt = np.clip((g["fold_z1"] - z) / L, 0, 1)
        return np.abs(y - g["fold_y"]) <= g["fold_h"] * (1 - 0.6 * tt)
    return np.ones(x.shape, bool)


def paint_face(e, face, boxes, c, tones_, rec, key):
    TX = c["TX"]
    corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face, TX)
    size = np.array(e["hi"]) - np.array(e["lo"])
    uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)
    p = corner + uu[..., None] * du * (abs(size @ du) / w) + vv[..., None] * dv * (abs(size @ dv) / h)
    R = rotm(e["rot"])
    if e["rot"]:
        o = np.array(e["origin"])
        p = (p - o) @ R.T + o
    pr = unwarp(p, c)  # the raven's space, for the rules
    a = c["area"](e, face, pr, uu, vv, h)
    light_face = face
    if e["name"].startswith("eyelid"):  # seen only closed, as the head round the eye
        a = c["area"](dict(e, name="lid"), face, pr, uu, vv, h)
        light_face = "east" if c["eyes"]["kind"] == "side" else "south"
    occ = None if e["quad"] or OPEN(e["name"]) else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
    cap = {}
    # A head's face under 6 texels keeps its marks: merging into big shapes (a blur 1.1 texels wide)
    # would hand a 4-texel owl's face, or a sparrow's cheek, all to one color. The body keeps
    # merging, so its feathers stay big simple areas as they were.
    col = paint_face_smooth(a, tones_, light_face, out=cap, coat=c["feather"], y_world=None, occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, tuft=False,
                            merge=min(w, h) >= 6 or e["name"] != "skull")
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"]:
        img[..., 3] = np.where(cut_mask(e, pr, c), 255, 0)
    eyes = c["eyes"]
    if e["name"] == "skull" and ((eyes["kind"] == "side" and face in ("east", "west")) or (eyes["kind"] == "front" and face == "south")):
        gw, gh, _, _, centers = eye_grid(c, face)
        assert (gw, gh) == (w, h), (face, gw, gh, w, h)
        stamp_eyes(img, centers, eye_design(c, face))
    if e["name"].startswith("eyelid"):  # closed, the lash line is its lower edge
        if eyes["kind"] == "side":
            img[:, 0 if e["name"].endswith("L") else w - 1, :3] = eyes["lash"]
        else:
            img[h - 1, :, :3] = eyes["lash"]
    if e["quad"] and e["cut"] in ("primaries", "secondaries", "folded", "tail"):  # the feathers' shafts and edges
        along = (np.abs(pr[..., 0]) if e["cut"] in ("primaries", "secondaries") else pr[..., 2]) / G["u"]
        band = max(1 / 0.9, 1.5 / (TX * c["K"] * G["u"]))  # each shade at least 1.5 texels wide (it was at 64 a block)
        img[..., :3] = np.where((np.floor(along / band) % 2 == 0)[..., None], (img[..., :3] * 0.88).astype(np.uint8), img[..., :3])
    if e["name"].startswith("toes_") and face == "south" and h >= 2:  # the claws (not on a toe a texel high: all of it would go dark)
        img[h - 1, :, :3] = (img[h - 1, :, :3] * 0.55).astype(np.uint8)
    if rec is not None:
        roles = [m[3] for m in c["materials"]]
        props = c["props"](e, face, pr, a) if c["props"] else {"white": np.full(pr.shape[:2], 0.2), "pattern": np.zeros(pr.shape[:2])}
        rec.face(key, cap["weights"], cap["level"], roles, np.clip(np.round(col), 0, 255), img, p, props)
    return img


# ------------------------------------------------------------------ animation


def stand(c, drop):
    """Standing: the body tipped up at the front, the head held level, the legs straight under it."""
    t, nk = c["tilt"], c["neck_share"]
    q = {"root": (0, -drop, 0), "spine_0": (t, 0, 0), "neck": (-t * nk, 0, 0), "head": (-t * (1 - nk), 0, 0), "tail": (-t * 0.8, 0, 0)}
    for n in ("L", "R"):
        q[f"leg_{n}_thigh"] = (-t, 0, 0)
        q[f"leg_{n}_shin"] = (0, 0, 0)
        q[f"leg_{n}_toes"] = (0, 0, 0)
    return q


def add(q, more):
    for k, v in more.items():
        a = q.get(k, (0, 0, 0))
        q[k] = tuple(a[i] + v[i] for i in range(3))
    return q


def poses(c, drop):
    il = c["idle_len"]

    def idle(t):
        p = t / il
        q = stand(c, drop)
        if c["idle"] == "owl":
            # The head turned far round in holds, a tilt, the body still.
            look = float(np.interp(p, [0, .15, .2, .4, .45, .62, .67, .85, .9, 1], [0, 0, 95, 95, 0, 0, -110, -110, 0, 0]))
            tilt = float(np.interp(p, [0, .45, .5, .58, .62, 1], [0, 0, 18, 18, 0, 0]))
            add(q, {"root": (0, 0.06 * S(TAU * p * 2), 0), "head": (0, look, tilt)})
            return q
        look = float(np.interp(p, [0, .2, .23, .5, .53, .78, .81, 1], [0, 0, 18, 18, -14, -14, 0, 0]))
        tilt = float(np.interp(p, [0, .5, .53, .78, .81, 1], [0, 0, 4, 4, 0, 0]))
        flick = max(0.0, S(TAU * (p - 0.6) * 8)) if 0.6 < p < 0.66 else 0.0
        add(q, {"root": (0, 0.08 * S(TAU * p * 2), 0), "head": (0, look, tilt), "tail": (-12 * flick, 0, 0)})
        return q

    def walk(t):
        p = t / 0.7
        a = TAU * p
        q = stand(c, drop)
        for n, ph in (("L", 0.0), ("R", 0.5)):
            b = TAU * (p + ph)
            up = max(0.0, S(b + math.pi / 2))
            add(q, {f"leg_{n}_thigh": (-26 * S(b), 0, 0), f"leg_{n}_shin": (24 * up, 0, 0), f"leg_{n}_toes": (26 * S(b) - 24 * up, 0, 0)})
        add(q, {"root": (0, 0.3 * abs(S(a)), 0), "spine_0": (0, 4 * S(a), 0), "neck": (5 * S(2 * a), 0, 0), "head": (-5 * S(2 * a), 0, 0)})
        return q

    def hop(t):
        p = t / 0.4
        a = TAU * p
        up = max(0.0, S(a))
        q = stand(c, drop)
        for n in ("L", "R"):
            add(q, {f"leg_{n}_thigh": (-30 * S(a), 0, 0), f"leg_{n}_shin": (34 * up, 0, 0), f"leg_{n}_toes": (30 * S(a) - 34 * up, 0, 0)})
        add(q, {"root": (0, 1.4 * up, 0), "tail": (-10 * up, 0, 0)})
        return q

    f = c["flap"]

    def fly(t):
        """A wingbeat, the hand following the arm, the legs tucked back; big birds glide part of
        the cycle with the wings held open."""
        p = t / f["length"]
        gl = f["glide"]
        if p < 1 - gl:
            ph = p / (1 - gl)
            cc, hh = S(TAU * ph), S(TAU * ph - 0.6)
        else:
            ph = (p - (1 - gl)) / max(gl, 1e-6)
            cc, hh = 0.12 * S(TAU * ph), 0.0
        q = {"spine_0": (2, 0, 0), "neck": (-4, 0, 0), "head": (4, 0, 0), "root": (0, 2.0 + 0.8 * cc, 0), "tail": (0, 0, 0)}
        for n, s in (("L", 1), ("R", -1)):
            q[f"wing_{n}"] = (0, 0, s * (f["beat"] * cc + (6 if p >= 1 - gl else 0)))
            q[f"hand_{n}"] = (0, 0, s * f["hand"] * hh)
            q[f"leg_{n}_thigh"] = (70, 0, 0)
            q[f"leg_{n}_toes"] = (40, 0, 0)
        return q

    ground = {"idle": (idle, il), "walk": (hop, 0.4) if c["walk"] == "hop" else (walk, 0.7), "run": (hop, 0.4)}
    return ground, {"flap": (fly, f["length"])}


BLINK = 0.08


def frame_times(c, name, dur):
    n = {"idle": int(16 * c["idle_len"]), "walk": 20, "run": 16, "flap": 24}.get(name, 16)
    ts = {round(dur * k / n, 4) for k in range(n + 1)}
    for b in c["blinks"].get(name, []):
        ts |= {round(b + d, 4) for d in (-BLINK, -BLINK / 2, 0, BLINK / 2, BLINK)}
    if c["idle"] == "owl" and name == "idle":  # the snaps of the head
        ts |= {round(dur * k, 4) for k in (.15, .2, .4, .45, .5, .58, .62, .67, .85, .9)}
    return sorted(ts)


def animations(c, anims, bone_names):
    out = {}
    side = c["eyes"]["kind"] == "side"
    for name, (fn, dur) in anims.items():
        frames = []
        for t in frame_times(c, name, dur):
            tt = t % dur if t < dur else 0.0
            pose = fn(tt)
            a = 90 * max([0.0] + [1 - abs(tt - b) / BLINK for b in c["blinks"].get(name, [])])
            pose.update({"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)} if side else {"eyelid_L": (-a, 0, 0), "eyelid_R": (-a, 0, 0)})
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [round(float(v), 4) for v in pose.get("root", (0, 0, 0))]})
        out[name] = {"length": dur, "frames": frames}
    return out


# ------------------------------------------------------------------ behaviour clips (`mods/animals/src/anim.rs`)
# Each a whole-body pose over what plays below on the bones it moves (`to_bedrock.as_action`): the held
# states and the approach loop, the rest once. The eagle and the owl hunt from the air, so no bird
# has a stalk or a pounce, and none stands its ground (no rear or charge). Worked out from the
# standing model itself (`spec`, any layout: the raven's, a bird's warped, the small bird's own), so
# the raven's build (`../raven/raven.py`) calls the same `behaviour`.

CLIPS = ["stare", "graze", "drink", "sleep", "watch", "approach", "alarm", "sniff", "look", "scratch", "stretch", "play", "groom",
         "nuzzle", "flinch", "death"]
# The held ones and the approach loop; a clip played once must not (the engine loops a clip whose
# file says so, whatever the game asks); the death holds its last frame.
LOOPS = {"stare": True, "graze": True, "drink": True, "sleep": True, "watch": True, "approach": True, "death": "hold_on_last_frame"}
BLENDS = {"flinch": (0.05, 0.2), "death": (0.1, 0.0), "sleep": (0.6, 0.6), "graze": (0.4, 0.4), "drink": (0.4, 0.4), "watch": (0.4, 0.4),
          "stare": (0.3, 0.3), "approach": (0.3, 0.3)}
LENGTHS = {"stare": 2.0, "graze": 2.0, "drink": 2.4, "sleep": 6.0, "watch": 4.0, "approach": 1.0, "alarm": 1.6, "sniff": 2.2, "look": 2.5,
           "scratch": 2.0, "stretch": 2.6, "play": 1.3, "groom": 2.8, "nuzzle": 2.2, "flinch": 0.45, "death": 1.6}
FRAMES = {"stare": 24, "graze": 40, "drink": 36, "sleep": 36, "watch": 48, "approach": 24, "alarm": 32, "sniff": 33, "look": 30,
          "scratch": 48, "stretch": 40, "play": 39, "groom": 56, "nuzzle": 33, "flinch": 12, "death": 24}
CLIP_BLINKS = {"stare": [1.3], "watch": [1.6, 3.4], "look": [1.25], "drink": [1.9]}
# The feet each clip stands on, its height set from them frame by frame: on one leg (the scratch, the
# stretch, asleep), walking; None: every part (dying, it rests on its lowest point).
PLANTED = {"scratch": ["toes_r"], "stretch": ["toes_r"], "sleep": ["toes_r"], "death": None}
# The snaps of the head: key times (shares of the clip) the holds start and end at, so they stay sharp.
SNAPS = {"watch": [.08, .1, .3, .32, .55, .57, .78, .8], "look": [.1, .14, .4, .44, .7, .74, .88, .92], "sniff": [.6, .63, .75, .78],
         "stare": [.1, .14, .86, .92], "groom": [.06, .09, .44, .47, .52, .55, .86, .89], "nuzzle": [.12, .15, .88, .91]}
WING_PARTS = ("wingf_", "primariesf_")
# The bones a clip may turn a little more, frame by frame, to keep a part out of the others (`Kit.unclip`).
UNCLIP = {"scratch": ["leg_L_thigh", "leg_L_shin", "leg_L_toes", "wingf_L"], "stretch": ["leg_L_thigh", "leg_L_shin", "leg_L_toes"],
          "play": ["leg_L_thigh", "leg_R_thigh"], "death": ["wingf_L", "wingf_R"]}
LEG_PARTS = ("shin_", "toes_")
GRID5 = np.array([[a, b, d] for a in np.linspace(0, 1, 5) for b in np.linspace(0, 1, 5) for d in np.linspace(0, 1, 5)])


def axis_of(c):
    """The body's axis the pokes check runs along (a neck hidden in the breast and head left out)."""
    return ["skull", "chest", "hips", "tail"] if c.get("neck_hidden") else ["neck", "skull", "chest", "hips", "tail"]


class Kit:
    """What the clips need from the standing model: its parts, joints, size, the bill's tip."""

    def __init__(self, spec, c, drop):
        from ground import corners
        self.spec, self.c, self.drop = spec, c, drop
        self.E = {e["name"]: e for e in spec["elements"]}
        self.piv = {b["name"]: np.array(b["pivot"], float) for b in spec["bones"]}
        pts = np.concatenate([corners(e) for e in spec["elements"]])
        self.ks = float(pts[:, 2].max() - pts[:, 2].min()) / 40.0  # the bird's size against the raven's (40 long)
        bill = corners(self.E["bill"])
        self.tip0 = bill[np.argsort(bill[:, 2])[-4:]].mean(0)  # the bill's tip at rest, on the head
        self.names = [e["name"] for e in spec["elements"]]
        self.owl = c["idle"] == "owl"
        self.side = c["eyes"]["kind"] == "side"
        self.axis = axis_of(c)
        self.tol = max(0.12, 0.3 * self.ks)  # how much deeper than at rest a part may go into another
        self.pairs = self._pairs()
        # How far they go into each other standing, and in the idle (an owl's head turned far round
        # sinks into its shoulders as it does): no clip goes further.
        poses = [stand(c, drop)] + [dict(f["rot"], root=f["root"]) for f in spec.get("animations", {}).get("idle", {}).get("frames", [])[::4]]
        Ws = [self.world(q) for q in poses]
        self.rest = {p: max(self._depth(W, *p) for W in Ws) for p in self.pairs}

    def world(self, q):
        from ground import world
        return world(self.spec["bones"], q)

    def at(self, W, bone, p):
        R, t = W[bone]
        return R @ (np.asarray(p, float) - self.piv[bone]) + t

    def tip(self, W):
        return self.at(W, "head", self.tip0)

    def low(self, q, groups):
        return lowest(self.spec, q, groups)

    def box_point(self, name, f):
        """A point of a part's box at rest, `f` its shares along x, y, z."""
        e = self.E[name]
        lo, hi = np.array(e["lo"]), np.array(e["hi"])
        return lo + np.asarray(f, float) * (hi - lo)

    def _pairs(self):
        """Parts that must not go into each other more than they do at rest: the folded wings into the
        body, the legs, the head and each other; the shins and feet into the body, the wings and the
        head; the head into the body (the joints' own pairs left out: they turn into each other)."""
        n = self.names
        wings = [x for x in n if x.startswith(WING_PARTS)]
        legs = [x for x in n if x.startswith(LEG_PARTS)]
        body = [x for x in n if x in ("chest", "hips", "tail", "skull") or x.startswith("thigh_")]
        out = []
        for a in wings:
            out += [(a, b) for b in body + [w for w in wings if w[-1] != a[-1]]]
        for a in legs:
            out += [(a, b) for b in body + [w for w in wings if w.startswith("wingf_")] if not (b.startswith("thigh_") and b[-1] == a[-1])]
        if not self.c.get("neck_hidden"):  # a head sitting on the breast (an owl, a songbird) joins it there
            out += [("skull", b) for b in ("chest", "hips")]
        return out

    def _depth(self, W, a, b):
        """How deep part a's points (a 5x5x5 grid through its box) go into part b's box, both ways. A
        folded wing and the chest leave out the shoulder (the points near the wing's hinge): turning,
        a joint's corners go into each other."""
        from ground import _frame_of
        best = 0.0
        for x, y in ((a, b), (b, a)):
            Ra, ta, oa, loa, hia = _frame_of(self.E[x], W, self.piv)
            Rb, tb, ob, lob, hib = _frame_of(self.E[y], W, self.piv)
            if np.any(hib - lob < 1e-6):  # a panel has no inside
                continue
            pts = loa + GRID5 * (hia - loa)
            wing = x if x.startswith("wingf_") else y if y.startswith("wingf_") else None
            if wing and "chest" in (x, y):
                # The shoulder: the points near the hinge, and the band above it and before it, turn into
                # the chest's side as the wing opens (hidden in it, as a shoulder joint), and the chest's
                # edge there into the wing.
                e = self.E[wing]
                pv = self.piv[e["bone"]]
                r = 0.6 * (e["hi"][0] - e["lo"][0]) + 0.6 * self.ks
                far = np.linalg.norm(pts - pv, axis=1) > r
                pts = pts[far & (pts[:, 1] < pv[1] - 0.05) & (pts[:, 2] < pv[2] - 0.05)]
                if not len(pts):
                    continue
            elif wing and "hips" in (x, y):
                # The same band above the hinge, along the back: raised, it sinks into the back's top.
                pts = pts[pts[:, 1] < self.piv[self.E[wing]["bone"]][1] - 0.05]
                if not len(pts):
                    continue
            w = (pts - oa) @ Ra.T + ta
            qb = (w - tb) @ Rb + ob
            best = max(best, float(np.minimum(qb - lob, hib - qb).min(1).max()))
        return best

    def into(self, q, only=None, tucked=False, touch=False, slack=True):
        """{(a, b): depth past the rest's} for the parts that go into each other further than `tol`
        (`tucked`: the left leg drawn up into the belly, as asleep on one leg, may go into the body;
        `touch`: the left foot at the head, scratching, touches it; asleep the head nestles twice as deep
        into its fluffed shoulders; `slack` False: any at all, for the fits). The head may sink a little more
        into the shoulders (three times as far: a bird's head settles into its plumage, and preens or tucks into the wing)."""
        W = self.world(q)
        out = {}
        for p in self.pairs:
            if only and not any(x.startswith(only) for x in p):
                continue
            if tucked and p[0] in ("shin_l", "toes_l") and p[1].startswith(("chest", "hips", "thigh_")):
                continue
            if touch and p == ("toes_l", "skull"):
                continue
            d = self._depth(W, *p) - self.rest[p]
            if d > (self.tol * ((6 if tucked else 3) if "skull" in p else 1) if slack else 0.02):
                out[p] = round(d, 2)
        return out

    def repair(self, q):
        """`q`, or the nearest pose to it free of pokes: the turn and the bow shared differently
        between the neck and the head, and the tilt (a neck box turned partway pokes a corner
        through the breast or the head, where turned less or further it doesn't)."""
        if self.pokes_ok(q):
            return q
        tries = sorted(((dy, dx, dz) for dy in range(-60, 61, 5) for dx in range(-30, 31, 5) for dz in range(-30, 31, 10) if dy or dx or dz),
                       key=lambda v: abs(v[0]) + abs(v[1]) + abs(v[2]))
        for dy, dx, dz in tries:
            r = add(dict(q), {"neck": (dx, dy, dz), "head": (-dx, -dy, -dz)})
            if self.pokes_ok(r):
                return r
        raise AssertionError(f"{self.c.get('name')}: no pose near {q} free of pokes")

    def unclip(self, q, bones, feet=("toes_l", "toes_r"), **kw):
        """`q`, or with `bones` turned (about x and z) as little as gets every part out of the others
        (a leg swinging through the chest on its way up), none of it under the feet it stands on."""
        every = {"all": lambda n: True, "feet": lambda n: n in feet}

        def under(r):
            g = self.low(r, every)
            return max(0.0, g["feet"] - g["all"] - 0.05)

        if not bones or not (self.into(q, **kw) or under(q) > 0.1):
            return q
        from scipy.optimize import minimize

        def moved(x):
            return add(dict(q), {b: (x[2 * i], 0, x[2 * i + 1]) for i, b in enumerate(bones)})

        def cost(x):
            hit = self.into(moved(x), slack=False, **kw)
            r = moved(x)
            return (1e-3 * float(np.sum(np.square(x))) + 100 * sum(max(0.0, v - 0.5 * self.tol) ** 2 for v in hit.values()) / self.ks ** 2
                    + 100 * under(r) ** 2)
        for x0 in ([0] * (2 * len(bones)), [0, 20] + [0] * (2 * len(bones) - 2)):
            r = minimize(cost, x0, method="Nelder-Mead", options={"maxiter": 800, "maxfev": 800, "xatol": 0.1, "fatol": 1e-7})
            if not self.into(moved(r.x), **kw) and under(moved(r.x)) <= 0.1:
                return moved(r.x)
        return moved(r.x)

    def flicker(self, q):
        """Faces of parts on different bones that land in one plane in this pose (`boxcheck`)."""
        from boxcheck import check_posed
        one = {"bones": self.spec["bones"], "elements": self.spec["elements"],
               "animations": {"p": {"frames": [{"t": 0, "rot": {k: list(v) for k, v in q.items() if k != "root"}, "root": list(q.get("root", (0, 0, 0)))}]}}}
        return check_posed(one)

    def unflicker(self, q, **kw):
        """`q`, or with one of the bones whose faces meet in a plane turned a few degrees (a jaw
        opening as far as the bill is tilted, a tucked foot's side in its thigh's)."""
        hits = self.flicker(q)
        if not hits:
            return q
        before = set(self.into(q, **kw))
        bones = []
        for a, _, b, _ in hits:
            for n in (b, a):
                bone = self.E[n]["bone"]
                if bone not in bones and bone not in ("root", "spine_0", "spine_1"):
                    bones.append(bone)
        for d in (2, -2, 4, -4, 6, -6, 9, -9):
            for b in bones:
                for ax in (0, 1, 2):
                    r = r4(add(dict(q), {b: tuple(d if i == ax else 0 for i in range(3))}))
                    if not self.flicker(r) and self.pokes_ok(r) and set(self.into(r, **kw)) <= before:
                        return r
        raise AssertionError(f"{self.c.get('name')}: flicker in {q}: {list(hits)}")

    def pokes_ok(self, q):
        from ground import pokes
        one = {"bones": self.spec["bones"], "elements": self.spec["elements"],
               "animations": {"p": {"frames": [{"t": 0, "rot": {k: v for k, v in q.items() if k != "root"}, "root": q.get("root", (0, 0, 0))}]}}}
        return not pokes(one, self.axis)


def lids(K, a):
    # Nearly shut a lid lies in the face's plane (it is tilted 4 degrees about its hinge): flicker. Past
    # 75 it is shut.
    a = 90.0 if a >= 75 else a
    return {"eyelid_L": (0, 0, a), "eyelid_R": (0, 0, -a)} if K.side else {"eyelid_L": (-a, 0, 0), "eyelid_R": (-a, 0, 0)}


def lean(K, q, d, level=True):
    """The body tipped forward `d` degrees about the hips (back, if negative), the legs kept upright
    under it, the tail following less, the head kept level (`level`)."""
    nk = K.c["neck_share"]
    add(q, {"spine_0": (d, 0, 0), "tail": (-0.85 * d, 0, 0), "leg_L_thigh": (-d, 0, 0), "leg_R_thigh": (-d, 0, 0)})
    if level:
        add(q, {"neck": (-d * nk, 0, 0), "head": (-d * (1 - nk), 0, 0)})
    return q


def bend(q, k, legs=("L", "R")):
    """The legs bent `k` degrees: the knee forward, the heel back, the feet kept flat."""
    for n in legs:
        add(q, {f"leg_{n}_thigh": (-k, 0, 0), f"leg_{n}_shin": (2 * k, 0, 0), f"leg_{n}_toes": (-k, 0, 0)})
    return q


def wings(q, roll=0.0, out=0.0, down=0.0, side=("L", "R")):
    """The folded wings opened about the shoulder: rolled out from the bottom (`roll`; 90 sticks
    out flat, 160 up), the back end swung out (`out`) and lowered (`down`), degrees."""
    for n in side:
        s = 1 if n == "L" else -1
        add(q, {f"wingf_{n}": (-down, -s * out, s * roll)})
    return q


def fit_clips(K):
    """The clips' fits on the standing model: the bill's tip to the ground (the pecks), a foot to the
    side of the head (the scratch), the bill to the wing and the breast (preening), the head turned
    back with the bill in the wing's top (asleep), a leg tucked up into the belly (asleep on one leg)
    and stretched back (the stretch). Each free of pokes and of parts going into each other."""
    from scipy.optimize import minimize
    c, ks = K.c, K.ks
    F = {}
    bill = {"bill": lambda n: n in ("bill", "jaw", "bill_hook"), "toes": lambda n: n.startswith("toes_"),
            "rest": lambda n: n not in ("bill", "jaw", "bill_hook") and not n.startswith(("toes_", "shin_", "thigh_"))}

    def peck(d, n, h, k):
        q = lean(K, stand(c, K.drop), d, level=False)
        return add(bend(q, k), {"neck": (n, 0, 0), "head": (h, 0, 0)})

    # The peck: the bill's tip just over the ground, by tipping the body and bowing the neck and head,
    # the legs bent only where that can't reach; the rest of it clear of the ground.
    rows = []
    for d in range(0, 61, 5):
        for n in range(-10, 91, 5):
            for h in range(-20, 91, 10):
                for k in (0, 10, 20, 30, 40, 50):
                    g = K.low(peck(d, n, h, k), bill)
                    cost = ((g["bill"] - g["toes"] - 0.15 * ks) / ks) ** 2 + 1e-4 * (1.5 * d * d + n * n + 0.5 * h * h) + 4e-4 * k * k
                    cost += 50 * max(0.0, g["toes"] + 0.6 * ks - g["rest"]) ** 2 / ks ** 2
                    rows.append((cost, d, n, h, k))
    rows.sort()
    for cost, d, n, h, k in rows[:400]:
        q = peck(d, n, h, k)
        if K.pokes_ok(q) and not K.into(q):
            break
    else:
        raise AssertionError(f"{c.get('name')}: no peck free of pokes")
    g = K.low(peck(d, n, h, k), bill)
    F.update(pk_d=d, pk_n=n, pk_h=h, pk_k=k, pk_gap=round(float(g["bill"] - g["toes"]), 2))
    K.peck = lambda s: peck(s * d, s * n, s * h, s * k)

    def bounded(x, lim):
        return sum(max(0.0, lo - v, v - hi) ** 2 for v, (lo, hi) in zip(x, lim))

    def best(cost, starts):
        r = [minimize(cost, s, method="Nelder-Mead", options={"maxiter": 3000, "maxfev": 3000, "xatol": 0.05, "fatol": 1e-6}) for s in starts]
        return min(r, key=lambda z: z.fun)

    def search(cost, lim):
        """The whole range searched (`lim`), then the best polished: the pokes make the cost jump."""
        from scipy.optimize import differential_evolution
        r = differential_evolution(cost, lim, seed=1, maxiter=100, popsize=10, tol=1e-6, polish=False)
        return best(cost, [r.x])

    def poked(q):
        return 0.0 if K.pokes_ok(r4(q)) else 1000.0  # as stored (to the 4th decimal)

    def depth_pen(q, only):
        return sum(v * v for v in K.into(q, only, slack=False).values())

    # The scratch: the left foot brought up outside the wing to the side of the head, the head bowed
    # and tipped down to it, standing on the right leg.
    sk = K.box_point("skull", (1.0, 0.3, 0.3))
    toe = K.box_point("toes_l", (0.5, 0.5, 1.0))
    lim = [(-20, 50), (-10, 80), (-60, 10), (-60, 10), (-170, 40), (0, 80), (-170, 170), (-150, 150), (0, 30)]

    def scr_pose(x):
        d, nrx, nrz, hrz, thx, thz, shx, tox, wr = x
        q = lean(K, stand(c, K.drop), d, level=False)
        add(q, {"neck": (nrx, 0, nrz), "head": (0, 0, hrz), "leg_L_thigh": (thx, 0, thz), "leg_L_shin": (shx, 0, 0), "leg_L_toes": (tox, 0, 0)})
        return wings(q, roll=wr, side=("L",))

    def scr_cost(x):
        q = scr_pose(x)
        W = K.world(q)
        g = K.low(q, {"r": lambda n: n == "toes_r", "head": lambda n: n in ("skull", "bill", "jaw", "bill_hook")})
        reach = np.sum((K.at(W, "leg_L_toes", toe) - K.at(W, "head", sk)) ** 2) / ks ** 2
        hit = {k: max(0.0, v - 0.5 * K.tol) for k, v in K.into(q, ("shin_l", "toes_l", "wingf_l"), touch=True, slack=False).items()}
        pen = bounded(x, lim) + max(0.0, g["r"] + 4 * ks - g["head"]) ** 2 / ks ** 2 + 20 * sum(v * v for v in hit.values()) / ks ** 2
        return float(reach + 5 * pen + poked(q) + 2e-5 * np.sum(np.square(x)))
    r = search(scr_cost, lim)
    F["scr"] = [float(v) for v in r.x]  # unrounded: a tenth of a degree can poke
    W = K.world(scr_pose(r.x))
    F["scr_miss"] = round(float(np.linalg.norm(K.at(W, "leg_L_toes", toe) - K.at(W, "head", sk))), 2)
    K.scr_pose = scr_pose

    # Preening: the bill's tip at the left wing's outer face (the wing eased out), then at the breast.
    wing_pt = K.box_point("wingf_l", (1.0, 0.7, 0.55))
    ch = K.E["chest"]
    breast = np.array([0.3 * ch["hi"][0], ch["lo"][1] + 0.4 * (ch["hi"][1] - ch["lo"][1]), ch["hi"][2]])
    hlim = [(-20, 45), (-50, 50), (-30, 30), (-60, 110), (-180, 180), (-60, 60)]

    def pre_pose(x, roll=10.0):
        nrx, nry, nrz, hrx, hry, hrz = x
        q = add(stand(c, K.drop), {"neck": (nrx, nry, nrz), "head": (hrx, hry, hrz)})
        return wings(q, roll=roll, side=("L",))

    def pre_cost(target, bone, x):
        q = pre_pose(x)
        W = K.world(q)
        reach = np.sum((K.tip(W) - (K.at(W, bone, target) + (np.array([0.25 * ks, 0, 0]) if bone == "wingf_L" else np.array([0, 0, 0.25 * ks])))) ** 2)
        return float(reach / ks ** 2 + 5 * bounded(x, hlim) + 200 * depth_pen(q, ("skull",)) / ks ** 2 + poked(q) + 2e-5 * np.sum(np.square(x)))
    r = search(lambda x: pre_cost(wing_pt, "wingf_L", x), hlim)
    F["gr_wing"] = [float(v) for v in r.x]  # unrounded: a tenth of a degree can poke
    r = search(lambda x: pre_cost(breast, "spine_1", x), hlim)
    F["gr_breast"] = [float(v) for v in r.x]  # unrounded: a tenth of a degree can poke
    K.pre_pose = pre_pose

    # Asleep: the head turned back over the left shoulder, the bill tucked in the wing's top.
    tuck = K.box_point("wingf_l", (0.5, 1.0, 0.6))
    slim = [(-30, 20), (-40, 40), (-40, 90), (110, 250), (-45, 45)]  # turned back: 110 to 250 (-110) degrees

    def sl_pose(x):
        nrx, nry, hrx, hry, hrz = x
        q = add(stand(c, K.drop), {"neck": (nrx, nry, 0), "head": (hrx, hry, hrz)})
        bend(q, 8, legs=("R",))
        return add(wings(q, roll=4.0), {"tail": (4, 0, 0)})

    def sl_cost(x):
        q = sl_pose(x)
        W = K.world(q)
        reach = np.sum((K.tip(W) - K.at(W, "wingf_L", tuck)) ** 2) / ks ** 2
        return float(reach + 5 * bounded(x, slim) + 2000 * depth_pen(q, ("skull",)) / ks ** 2 + poked(q) + 2e-5 * np.sum(np.square(x)))
    r = search(sl_cost, slim)
    if not K.pokes_ok(sl_pose(r.x)):
        # Nothing near pokes free (the neck's box through the head turned partway): the head turned
        # right round instead, the best of a grid of those that are.
        rows = []
        for nrx in range(-30, 21, 5):
            for nry in (0, 10, 20, -10):
                for hrx in range(-40, 91, 10):
                    for hry in (160, 170, 180, 190):
                        for hrz in (-20, 0, 20):
                            x = [nrx, nry, hrx, hry, hrz]
                            q = sl_pose(x)
                            W = K.world(q)
                            rows.append((float(np.sum((K.tip(W) - K.at(W, "wingf_L", tuck)) ** 2) / ks ** 2), x))
        rows.sort(key=lambda r_: r_[0])
        for _, x in rows:
            q = sl_pose(x)
            if K.pokes_ok(q) and not K.into(q, ("skull",)):
                break
        r = best(sl_cost, [x])
    F["sl"] = [float(v) for v in r.x]  # unrounded: a tenth of a degree can poke
    K.sl_pose = sl_pose

    # On one leg: the left tucked up into the belly, the toes as high as they go under the body.
    belly = K.box_point("thigh_l", (0.5, 0.8, 0.5))
    tlim = [(-120, 30), (0, 170), (-170, 90)]

    def tuck_pose(x, q=None):
        thx, shx, tox = x
        q = q if q is not None else stand(c, K.drop)
        return add(q, {"leg_L_thigh": (thx, 0, 0), "leg_L_shin": (shx, 0, 0), "leg_L_toes": (tox, 0, 0)})

    def tuck_cost(x):
        q = tuck_pose(x)
        W = K.world(q)
        out = sum(v * v for v in K.into(q, ("shin_l", "toes_l"), tucked=True, slack=False).values())  # not through the tail or a wing
        return float(np.sum((K.at(W, "leg_L_toes", toe) - K.at(W, "spine_0", belly)) ** 2) / ks ** 2 + 5 * bounded(x, tlim) + 200 * out / ks ** 2
                     + 2e-5 * np.sum(np.square(x)))
    r = best(tuck_cost, ([-40, 120, -60], [-60, 140, -40], [-20, 100, -80], [-40, 120, 40]))
    F["tuck"] = [float(v) for v in r.x]  # unrounded: a tenth of a degree can poke
    q = tuck_pose(r.x)
    W = K.world(q)
    F["tuck_miss"] = round(float(np.linalg.norm(K.at(W, "leg_L_toes", toe) - K.at(W, "spine_0", belly))), 2)
    K.tuck_pose = tuck_pose

    # The stretch: the left leg pushed straight back, the foot off the ground and clear of the tail.
    heel = K.box_point("toes_l", (0.5, 0.0, 0.5))
    stlim = [(0, 100), (-60, 40), (-30, 120)]

    def st_pose(x, q=None, out=1.0):
        thx, shx, tox = x
        q = q if q is not None else stand(c, K.drop)
        return add(q, {"leg_L_thigh": (thx, 0, 16 * out), "leg_L_shin": (shx, 0, 0), "leg_L_toes": (tox, 0, 0)})

    def st_cost(x):
        q = st_pose(x)
        W = K.world(q)
        g = K.low(q, {"r": lambda n: n == "toes_r", "l": lambda n: n.startswith(("toes_l", "shin_l"))})
        back = K.at(W, "leg_L_toes", heel)[2] / ks
        pen = bounded(x, stlim) + max(0.0, g["r"] + 1.0 * ks - g["l"]) ** 2 / ks ** 2 + 200 * depth_pen(q, ("shin_l", "toes_l")) / ks ** 2
        return float(back + 10 * pen + 1e-4 * np.sum(np.square(x)))
    r = best(st_cost, ([40, -20, 60], [60, 0, 80], [30, -30, 30]))
    F["st"] = [float(v) for v in r.x]  # unrounded: a tenth of a degree can poke
    K.st_pose = st_pose

    # Both wings raised over the back as far as they go (to 150 degrees) clear of the chest and the head.
    for roll in range(150, 59, -5):
        if not K.into(wings(stand(c, K.drop), roll=roll, out=6), WING_PARTS, slack=False):
            break
    F["up"] = roll
    return F


def behaviour(spec, c, drop):
    """The behaviour clips of a standing bird (`spec`, with its idle, walk and run): {name: animation}
    to add to its animations, and what was fitted."""
    K = Kit(spec, c, drop)
    F = fit_clips(K)
    ks, owl = K.ks, K.owl
    base = lambda: stand(c, drop)  # noqa: E731
    big = 2.75 if owl else 1.0  # an owl turns its head far round (as far as its idle: 110)

    def held(t, period):
        """Breathing, nothing else (the chest rising)."""
        return add(base(), {"spine_1": (0.6 * S(TAU * t / period), 0, 0)})

    def snap(p, keys, values):
        return float(np.interp(p, keys, values))

    def stare(t):
        """Half aware: frozen, the head cocked to one side, only the breath (2 s, held)."""
        p = t / 2.0
        k = snap(p, [0] + SNAPS["stare"] + [1], [0, 0, 1, 1, 0, 0])
        return add(held(t, 2.0), {"head": (-4 * k, 10 * k * big ** 0.5, (24 if owl else 16) * k)})

    def watch(t):
        """A sentinel: stood up tall, the neck stretched, the head turning in snaps and holding (an
        owl's far round) (4 s, held)."""
        p = t / 4.0
        yaw = snap(p, [0] + SNAPS["watch"] + [1], [0, 0, 35, 35, -30, -30, 15, 15, 0, 0]) * big
        tilt = snap(p, [0] + SNAPS["watch"] + [1], [0, 0, 6, 6, -8, -8, 0, 0, 0, 0])
        q = lean(K, held(t, 4.0), -10)
        return add(q, {"neck": (-6, 0, 0), "head": (6, yaw, tilt)})

    def graze(t):
        """Pecking at the ground: three pecks, the bill's tip down to it, half up between, stepping
        the aim aside (2 s, held)."""
        p = t / 2.0
        g = snap(p, [0, .1, .16, .28, .38, .44, .56, .66, .72, .84, 1], [.5, 1, 1, .5, 1, 1, .5, 1, 1, .5, .5])
        aim = snap(p, [0, .28, .34, .56, .62, .84, .9, 1], [0, 0, 8, 8, -6, -6, 0, 0])
        return add(K.peck(g), {"head": (0, aim, 0), "jaw": (6 * max(0.0, 1 - abs(g - 1) * 6), 0, 0)})

    def drink(t):
        """Drinking: the bill down to the water, then the head tipped back to swallow, the throat
        working (2.4 s, held)."""
        p = t / 2.4
        g = snap(p, [0, .14, .34, .48, 1], [0, 1, 1, 0, 0])
        b = snap(p, [0, .46, .56, .82, .94, 1], [0, 0, 1, 1, 0, 0])
        gulp = b * abs(S(TAU * t * 3))
        return add(K.peck(g), {"neck": (-6 * b, 0, 0), "head": (-36 * b, 0, 0), "jaw": (4 * gulp + 5 * (g > 0.95), 0, 0)})

    def sniff(t):
        """Probing: the bill down near the ground, two quick probing pecks, then up and a look about
        (2.2 s, once)."""
        p = t / 2.2
        g = snap(p, [0, .15, .22, .27, .34, .4, .5, .58, 1], [0, .8, 1, .8, 1, .8, .7, 0, 0])
        yaw = snap(p, [0, .6, .63, .75, .78, 1], [0, 0, 28, 28, 0, 0]) * big
        return add(K.peck(g), {"head": (0, yaw, 10 * yaw / max(big, 1) / 28)})

    def look(t):
        """The head turned in snaps one way, the other, holding each (an owl's far round) (2.5 s, once)."""
        p = t / 2.5
        k = [0] + SNAPS["look"] + [1]
        yaw = snap(p, k, [0, 0, 40, 40, -35, -35, 20, 20, 0, 0]) * big
        tilt = snap(p, k, [0, 0, 8, 8, -10, -10, 4, 4, 0, 0])
        return add(held(t, 2.5), {"neck": (-4, 0, 0), "head": (4, yaw, tilt)})

    def alarm(t):
        """Alarm: crouched, the wings half open and the tail up, the plumage raised, two harsh calls
        with the bill open wide (1.6 s, once; it stays on the ground)."""
        p = t / 1.6
        d = snap(p, [0, .12, .85, 1], [0, 1, 1, 0])
        call = sum(max(0.0, S(math.pi * (p - a) / 0.2)) for a in (.22, .55) if a < p < a + 0.2)
        q = bend(base(), 16 * d)
        wings(q, roll=38 * d, out=16 * d, down=4 * d)
        return add(q, {"tail": (16 * d, 0, 0), "neck": (-8 * d + 6 * call, 0, 0), "head": (4 * d, 0, 0), "jaw": (26 * call, 0, 0),
                       "spine_1": (-4 * d, 0, 0)})

    def scratch(t):
        """An itch: the left wing eased down, the left foot brought up over it to the side of the
        head, which bows and tips down to it, scratching fast, the eyes half shut (2 s, once)."""
        p = t / 2.0
        d = snap(p, [0, .25, .8, 1], [0, 1, 1, 0])
        k = d * S(TAU * t * 5)
        x = np.array(F["scr"]) * d
        q = K.scr_pose(x)
        # On the way the leg (and the wing) turned frame by frame where it would go through the chest or the
        # wing (`UNCLIP`).
        # The foot's flicks, away from the breast only (the shin's, or the toes' the other way, drove it in).
        add(q, {"leg_L_toes": (14 * abs(k), 0, 0), "head": (0, 0, 3 * k)})
        q.update(lids(K, 50 * d))
        return q

    def stretch(t):
        """Waking up: the left wing and leg stretched down and back together, then both wings raised
        over the back, a yawn (2.6 s, once)."""
        p = t / 2.6
        a = snap(p, [0, .14, .38, .5, 1], [0, 1, 1, 0, 0])
        b = snap(p, [0, .5, .64, .84, .96, 1], [0, 0, 1, 1, 0, 0])
        x = np.array(F["st"]) * a
        x[2] *= a  # the toes turn late: early, pointed down, they went through the ground (lifted, into the belly)
        q = K.st_pose(x, base(), out=a)
        wings(q, roll=40 * a, out=22 * a, down=18 * a, side=("L",))
        wings(q, roll=F["up"] * b, out=6 * b)
        return add(q, {"neck": (-6 * b, 0, 0), "head": (-10 * b, 0, 0), "jaw": (22 * b, 0, 0), "tail": (10 * a, 0, 0)})

    def groom(t):
        """Preening: the head turned back to the left wing, the bill working through its feathers,
        then down to the breast (2.8 s, once)."""
        p = t / 2.8
        w = snap(p, [0] + SNAPS["groom"][:4] + [1], [0, 0, 1, 1, 0, 0])
        br = snap(p, [0] + SNAPS["groom"][4:] + [1], [0, 0, 1, 1, 0, 0])
        x = np.array(F["gr_wing"]) * w + np.array(F["gr_breast"]) * br
        nib = (w + br) * S(TAU * t * 4)  # the bill working (the head still: nodding, the neck's box poked through it)
        q = K.pre_pose(x, roll=10 * w)
        add(q, {"jaw": (10 * abs(nib), 0, 0)})
        q.update(lids(K, 35 * (w + br)))
        return q

    def play(t):
        """A hop and flap: a crouch, a jump with the wings beating twice, a landing (1.3 s, once).
        Its height in the air from `air`, the rest of it from the ground."""
        p = t / 1.3
        k = snap(p, [0, .14, .24, .3, .7, .76, .86, 1], [0, 24, 24, 0, 0, 22, 0, 0])
        tuck = snap(p, [0, .3, .38, .62, .7, 1], [0, 0, 1, 1, 0, 0])
        flap = abs(S(TAU * (p - .26) / 0.48 * 2)) if .26 < p < .74 else 0.0
        q = bend(base(), k)
        add(q, {"leg_L_thigh": (-18 * tuck, 0, 0), "leg_R_thigh": (-18 * tuck, 0, 0), "leg_L_shin": (40 * tuck, 0, 0),
                "leg_R_shin": (40 * tuck, 0, 0), "leg_L_toes": (-22 * tuck, 0, 0), "leg_R_toes": (-22 * tuck, 0, 0),
                "tail": (10 * tuck, 0, 0), "neck": (-6 * tuck, 0, 0)})
        return wings(q, roll=12 * tuck + 64 * flap, out=6 * flap)

    def air_play(t):
        p = t / 1.3
        return 3.2 * ks * math.sin(math.pi * float(np.clip((p - .3) / .4, 0, 1)))

    def nuzzle(t):
        """Allopreening a mate beside it: the head turned aside, the bill nibbling at the height of
        its head (2.2 s, once)."""
        p = t / 2.2
        d = snap(p, [0] + SNAPS["nuzzle"] + [1], [0, 0, 1, 1, 0, 0])
        nib = d * S(TAU * t * 3)
        q = add(base(), {"neck": (-12 * d, 14 * d, 0), "head": (6 * d + 5 * nib, 30 * d, 8 * d), "jaw": (10 * abs(nib), 0, 0)})
        q.update(lids(K, 30 * d))
        return q

    def flinch(t):
        """Hit: a jerk, the head snatched back, the wings jolted, the eyes squeezed (0.45 s, once)."""
        p = t / 0.45
        d = snap(p, [0, .15, .45, 1], [0, 1, .8, 0])
        q = bend(base(), 10 * d)
        wings(q, roll=14 * d, out=6 * d)
        add(q, {"spine_0": (0, 0, 4 * d), "neck": (-14 * d, 0, 0), "head": (10 * d, 0, 0), "tail": (-8 * d, 0, 0)})
        q.update(lids(K, 70 * d))
        return q

    def sleep(t):
        """Asleep: the head turned back over the shoulder and the bill tucked in the wing's top, the
        plumage fluffed (wings eased out), on one leg where it fits, the eyes shut (6 s, held)."""
        br = S(TAU * t / 6.0 * 2)
        q = K.sl_pose(F["sl"])
        if F["one_leg"]:
            K.tuck_pose(F["tuck"], q)
        add(q, {"wingf_L": (0, 0, 1.0 * br), "wingf_R": (0, 0, -1.0 * br)})
        q.update(lids(K, 90))
        return q

    def death(t):
        """Dying: the legs give, it falls onto its right side, the legs stiff, the wings loose (the
        upper one slid loose, the lower spread), the head dropped, the eyes shut; it stays (1.6 s, holds)."""
        p = t / 1.6
        f = snap(p, [0, .3, .65, 1], [0, .35, 1, 1])
        q = bend(base(), 20 * f * (1 - f) * 4)
        add(q, {"spine_0": (-c["tilt"] * 0.6 * f, 0, 84 * f), "neck": (24 * f, 0, 0), "head": (20 * f, 0, -30 * f), "jaw": (12 * f, 0, 0),
                "tail": (-6 * f, 0, 0)})
        for n in ("L", "R"):
            add(q, {f"leg_{n}_thigh": (-30 * f, 0, 0), f"leg_{n}_shin": (-10 * f, 0, 0), f"leg_{n}_toes": (60 * f, 0, 0)})
        # The upper one lying loose along the body, drooped (swung out, it stood up in the air), the lower
        # one spread on the ground.
        wings(q, roll=6 * f, down=10 * f, side=("L",))
        wings(q, roll=35 * f, out=10 * f, side=("R",))
        q.update(lids(K, 90 * f))
        return q

    walker = c["walk"] != "hop"

    def approach(t):
        """Curious, coming to look: slow careful steps (or small hops), the neck stretched forward and
        the head low (1 s a cycle, in place of the walk)."""
        p = t / 1.0
        a = TAU * p
        q = lean(K, base(), 10)
        if walker:
            for n, ph in (("L", 0.0), ("R", 0.5)):
                b = TAU * (p + ph)
                up = max(0.0, S(b + math.pi / 2))
                add(q, {f"leg_{n}_thigh": (-20 * S(b), 0, 0), f"leg_{n}_shin": (34 * up, 0, 0), f"leg_{n}_toes": (20 * S(b) - 34 * up, 0, 0)})
            add(q, {"neck": (10 + 4 * S(2 * a), 0, 0), "head": (-4 - 4 * S(2 * a), 0, 0)})
        else:  # two small hops a cycle
            up = max(0.0, S(2 * a))
            for n in ("L", "R"):
                add(q, {f"leg_{n}_thigh": (-24 * S(2 * a), 0, 0), f"leg_{n}_shin": (26 * up, 0, 0), f"leg_{n}_toes": (24 * S(2 * a) - 26 * up, 0, 0)})
            add(q, {"neck": (10, 0, 0), "head": (-4, 0, 0), "tail": (-6 * up, 0, 0)})
        return q

    def air_approach(t):
        return 0.0 if walker else 0.8 * ks * max(0.0, S(2 * TAU * t / 1.0))

    # On one leg where the tucked leg fits up under the belly (its toes within reach of it, nothing
    # pushed through the wing or the body).
    tq = K.tuck_pose(F["tuck"], K.sl_pose(F["sl"]))
    F["one_leg"] = bool(F["tuck_miss"] < 1.2 * ks and not K.into(tq, ("shin_l", "toes_l"), tucked=True))
    fns = {"stare": stare, "graze": graze, "drink": drink, "sleep": sleep, "watch": watch, "approach": approach, "alarm": alarm, "sniff": sniff,
           "look": look, "scratch": scratch, "stretch": stretch, "play": play, "groom": groom, "nuzzle": nuzzle, "flinch": flinch, "death": death}
    airs = {"play": air_play, "approach": air_approach}
    return clip_animations(K, fns, airs), F


def r4(q):
    return {k: tuple(round(float(v), 4) for v in r) for k, r in q.items()}


def clip_animations(K, fns, airs):
    from ground import feet_low
    spec, c = K.spec, K.c
    bone_names = [b["name"] for b in spec["bones"]]
    out = {}
    for name in CLIPS:
        fn, dur = fns[name], LENGTHS[name]
        loops = LOOPS.get(name, False)
        n = FRAMES[name]
        sn = SNAPS.get(name, [])
        # A snap of the head is a frame: no key inside it (the head turned partway, the neck's box pokes
        # a corner through the head or the breast: it is clean where it starts and where it ends).
        inside = lambda t: any(dur * a < t < dur * b for a, b in zip(sn[0::2], sn[1::2]))  # noqa: E731
        ts = {round(dur * k / n, 4) for k in range(n + 1) if not inside(dur * k / n)} | {round(dur * k, 4) for k in sn}
        for b in CLIP_BLINKS.get(name, []):
            ts |= {round(b + d, 4) for d in (-BLINK, -BLINK / 2, 0, BLINK / 2, BLINK)}
        frames, air = [], []
        for t in sorted(ts):
            # A loop's last frame is its first; a clip played once ends where it ends.
            tt = t % dur if t < dur else (0.0 if loops is True else dur)
            # Checked as stored, to the 4th decimal: a corner just at a face flips with the rounding.
            pose = K.unclip(K.repair(r4(fn(tt))), UNCLIP.get(name, ()), feet=K.names if name == "death" else PLANTED.get(name, ("toes_l", "toes_r")),
                            tucked=name == "sleep", touch=name == "scratch")
            pose = K.unflicker(K.repair(r4(pose)), tucked=name == "sleep", touch=name == "scratch")
            blink = 90 * max([0.0] + [1 - abs(tt - b) / BLINK for b in CLIP_BLINKS.get(name, [])])
            own = abs(pose.get("eyelid_L", (0, 0, 0))[2 if K.side else 0])
            pose.update(lids(K, max(blink, own)))
            h = airs[name](tt) if name in airs else 0.0
            frames.append({"t": round(t, 4), "rot": {b: [round(float(v), 4) for v in pose.get(b, (0, 0, 0))] for b in bone_names if b != "root"},
                           "root": [0.0, h, 0.0]})
            air.append(h)
        anim = {"length": dur, "frames": frames, "loop": loops, "blend": BLENDS.get(name, (0.25, 0.3))}
        # Its height from the ground: each frame on its planted feet (or, dying, its lowest point), the
        # airborne ones of a hop that high over where its feet would stand.
        feet = PLANTED.get(name, ["toes_l", "toes_r"]) or K.names
        low = feet_low(spec, anim, feet)
        for fr, y, h in zip(frames, low, air):
            fr["root"][1] = round(float(fr["root"][1] - (y - h)), 4)
        out[name] = anim
    # A clip goes over what plays below only on the bones it writes, and the exporter leaves out a bone
    # that stays at 0: kept at a hundredth of a degree, so the idle's look and the walk don't show
    # through a clip that holds them still. The eyelids are left to the idle's blinks.
    moved = {b for a in ("idle", "walk", "run") for f in spec["animations"][a]["frames"] for b, r in f["rot"].items()
             if any(abs(v) > 1e-6 for v in r)} - {"eyelid_L", "eyelid_R"}
    for anim in out.values():
        for b in moved:
            if all(not any(abs(v) > 1e-6 for v in f["rot"][b]) for f in anim["frames"]):
                for f in anim["frames"]:
                    f["rot"][b] = [0.01, 0.0, 0.0]
    for name, anim in out.items():
        for f in anim["frames"]:
            assert K.pokes_ok(dict(f["rot"], root=f["root"])), f"{c.get('name')} {name}@{f['t']}: pokes once held at a hundredth"
    return out


def check_clips(spec, c):
    """The clips' checks, for `export.py` and the raven's build: loops, feet on the ground, nothing in
    the ground, the gait, the lids, nothing opened or swung into the body."""
    from ground import feet_low, gait
    A = spec["animations"]
    looping = sorted(k for k in CLIPS if A[k].get("loop") is True)
    assert looping == ["approach", "drink", "graze", "sleep", "stare", "watch"], f"looping: {looping}"
    assert A["death"]["loop"] == "hold_on_last_frame", "the death must hold its last frame"
    feet = ["toes_l", "toes_r"]
    everything = [e["name"] for e in spec["elements"]]
    for clip in CLIPS:
        low = feet_low(spec, A[clip], everything)
        assert min(low) > -0.3, f"{clip}: a part {min(low):.2f} into the ground"
        if clip == "death":  # each frame rests on its lowest point
            assert max(low) < 0.3, f"dying, its lowest point up to {max(low):.2f}"
            continue
        low = feet_low(spec, A[clip], feet)
        if clip == "play":  # takes off from the ground and lands on it
            assert abs(min(low)) < 0.3 and low[0] < 0.3 and low[-1] < 0.3, f"the hop doesn't start and end on the ground: {low[0]:.2f}, {low[-1]:.2f}"
        elif clip == "approach" and c["walk"] == "hop":
            assert abs(min(low)) < 0.3, f"approach's hops from {min(low):.2f}"
        else:
            assert min(low) > -0.3 and max(low) < 0.3, f"{clip}'s feet from {min(low):.2f} to {max(low):.2f}, not on the ground"
    back = {f: v for f, v in gait(spec, "approach", feet).items() if v is not None and v <= 0}
    assert not back, f"approach goes backwards: {back}"
    shut = lambda f: abs(f["rot"]["eyelid_L"][2 if c["eyes"]["kind"] == "side" else 0]) >= 89.9  # noqa: E731
    assert all(shut(f) for f in A["sleep"]["frames"]), "asleep with its eyes open"
    assert shut(A["death"]["frames"][-1]), "dead with its eyes open"
    K = Kit({**spec, "animations": {"idle": A["idle"]}}, c, -A["idle"]["frames"][0]["root"][1])
    bad = {}
    for clip in CLIPS:
        for fr in A[clip]["frames"]:
            hit = K.into(dict(fr["rot"], root=fr["root"]), tucked=clip == "sleep", touch=clip == "scratch")
            if hit:
                bad.setdefault(clip, []).append((fr["t"], hit))
    assert not bad, f"parts opened or swung into each other: {[(k, v[:2]) for k, v in bad.items()]}"


# ------------------------------------------------------------------ build


def config(cfg):
    """A bird's knobs over the defaults."""
    c = dict(DEFAULTS)
    c.update(cfg)
    for k in ("bill", "tail", "wing", "flap", "eyes"):
        c[k] = dict(DEFAULTS[k], **cfg.get(k, {}))
    return c


def build(cfg, out_dir):
    """Lays out, paints and animates a bird; returns (ground spec, flying spec, texture)."""
    c = config(cfg)
    G.clear()
    G.update(geometry(c))
    G.update(c.get("layout", {}).get("G", {}))
    (sx0, sy0, sz0), (sx1, sy1, sz1) = (warp(v, c) for v in G["skull"])
    off = 0.35 if c["TX"] <= 1 else 0.15  # the eyelids' hinge just outside the face: closed, they sit over the eyes
    G["lid"] = (sx1 + off) if c["eyes"]["kind"] == "side" else (sz1 + off)
    tones_ = [tones(m[1], m[2]) for m in c["materials"]]
    B = bones(c)
    E = elements(c)
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in E if not e["quad"] and not e["rot"] and not OPEN(e["name"])]
    rec = Recorder(layer_bounds(E, 2.0 * c["K"]))
    tiles = []
    for i, e in enumerate(E):
        for f in ([e["quad"]] if e["quad"] else list(FACES)):
            tiles.append(((i, f), paint_face(e, f, boxes, c, tones_, rec, (i, f))))
    sheet, uv = pack(tiles, 512)  # the layers pack at the same width (`Recorder.save`)
    rec.save(out_dir, [k for k, _ in tiles], pack)
    for i, e in enumerate(E):
        e["faces"] = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
    names = [n for n, _, _ in B]
    bones_json = [{"name": n, "parent": p, "pivot": [float(v) for v in pv]} for n, p, pv in B]
    ground = {"name": c["name"], "texture": [sheet.shape[1], sheet.shape[0]], "bones": bones_json,
              "elements": [e for e in E if not OPEN(e["name"])]}
    g = lowest(ground, stand(c, 0.0), {"toes": lambda n: n.startswith("toes_"), "all": lambda n: True})
    drop = round(float(g["toes"]), 2)
    ground_anims, fly_anims = poses(c, drop)
    ground["animations"] = animations(c, ground_anims, names)
    clips, fitted = behaviour(ground, c, drop)  # the behaviour clips, on the standing model
    ground["animations"].update(clips)
    ground["fitted"] = {"drop": drop, "lowest": round(float(g["all"] - g["toes"]), 2), "clips": fitted}
    fly = {"name": c["name"] + "_fly", "texture": ground["texture"], "bones": bones_json, "elements": [e for e in E if not FOLDED(e["name"])]}
    fly["animations"] = animations(c, fly_anims, names)
    return ground, fly, sheet
