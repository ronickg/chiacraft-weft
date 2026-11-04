#!/usr/bin/env python3
"""Birds from the raven's build (`../raven/raven.py`): one layout with knobs, so the small bird,
the owl, the eagle, the vulture and the gull are each a page of numbers and color rules (their own
`<model>.py`) on a build that already passed the review checklist. Made with the blocky-model
skill: boxes, cut-out feather panels and a hand-painted texture.

A bird is laid out in the raven's units (a raven is 40 long), shaped by its knobs (a bigger head,
a shorter or hooked bill, broader or longer wings, a longer tail, wider body, longer or shorter
legs), then warped into its own size (`warp`: the legs below `LEG_Y` stretched by `D`, everything
`K` times bigger). Small birds paint `FINE` texels a unit (`TX`). One build paints both models on
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
from paint import EYES, occlusion, pack, paint_face_smooth, stamp_eyes, tones  # noqa: E402
from ground import lowest  # noqa: E402

TAU = 2 * math.pi
S = math.sin
LEG_Y = 8.0  # below this (the raven's units) the legs are stretched

DEFAULTS = dict(
    name="bird",
    K=1.0,  # drawn this many times the raven's size
    D=0.0,  # the legs this much longer (raven units; negative shorter)
    TX=1,  # texels a unit (house.FINE for a small bird)
    head=1.0,  # the skull's size against the raven's
    head_y=0.0,  # the skull raised (raven units)
    neck=True,  # a neck box between the chest and the head (an owl's head sits on its shoulders)
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
                leg_x=min(1.9 * bw, 3.4 * bw - max(1.1, c["shin"] + 0.6) - 0.4 / c["K"]), fold_end=-4.0 - 11.0 * w["folded"], eye=eye,
                fan_z1=-8.0, fold_z1=-4.0, prim_x0=hand_x - 0.2, prim_z1=2.8, fold_y=12.0, fold_h=2.0, u=1.0)  # u: the feathers' wiggles, in raven units


def eye_box(c, s):
    """The eye's texels on the warped skull, as `stamp_eyes` rounds them, in warped units: side
    eyes (z0, z1, top, bottom), front eyes (x0, x1, top, bottom)."""
    pat = EYES[c["eyes"]["pattern"]]["pattern"]
    ph, pw = len(pat), len(pat[0])
    TX = c["TX"]
    (x0, y0, z0), (x1, y1, z1) = (warp(v, c) for v in G["skull"])
    ey = warp((0, c["eyes"]["y"] + c["head_y"], 0), c)[1]
    v0 = int(round(TX * (y1 - ey) - ph / 2))
    top, bottom = y1 - v0 / TX, y1 - (v0 + ph) / TX
    if c["eyes"]["kind"] == "side":
        ez = warp((0, 0, c["eyes"]["z"]), c)[2]
        cu = TX * ((z1 - ez) if s > 0 else (ez - z0))
        u0 = int(round(cu - pw / 2))
        za, zb = (z1 - (u0 + pw) / TX, z1 - u0 / TX) if s > 0 else (z0 + u0 / TX, z0 + (u0 + pw) / TX)
        return za, zb, top, bottom
    ex = s * c["eyes"]["x"] * c["K"]
    u0 = int(round(TX * (ex - x0) - pw / 2))
    return x0 + u0 / TX, x0 + (u0 + pw) / TX, top, bottom


def bones(c):
    g = G
    if c.get("layout"):  # a bird laid out in its own units (a small one)
        B = list(c["layout"]["bones"])
    else:
        B = [("root", None, (0, 0, 0)), ("spine_0", "root", (0, 12.5, -2.0)), ("spine_1", "spine_0", (0, 12.5, 0.6)),
             ("neck", "spine_1", (0.0, 13.6, 6.6)), ("head", "neck", (0.0, 17.0 + 0.5 * c["head_y"], 9.6)), ("jaw", "head", g["jaw_pivot"]),
             ("tail", "spine_0", (0, 13.2, -6.4))]
        for s, n in ((1, "L"), (-1, "R")):
            B += [(f"wingf_{n}", "spine_1", (s * (g["side"] - 0.1), 14.6, 5.0)),
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
    E = [el("hips", "spine_0", (-3.4 * bw, 9.0, -7.2), (3.4 * bw, 16.0, 1.4)),
         el("chest", "spine_1", (-4.0 * bw, 8.4, -0.4), (4.0 * bw, 16.6, 8.4))]
    if c["neck"]:
        E.append(el("neck", "neck", (-2.0, 12.4, 5.0), (2.0, 18.2, 10.4)))  # deep in the head
    E += [el("skull", "head", *g["skull"]),
          el("bill", "head", *g["bill"], rot=(c["bill"]["rot"], 0, 0), origin=(0, (g["bill"][0][1] + g["bill"][1][1]) / 2, g["front"] - 0.5)),
          el("jaw", "jaw", *g["jaw"]),  # shorter than the bill
          el("tail", "tail", (-g["tail_w"], 13.2 - max(0.6, 0.4 / c["K"]), g["tail_end"]), (g["tail_w"], 13.2 + max(0.6, 0.4 / c["K"]), -5.0)),
          el("tail_fan", "tail", (-g["fan_w"], 13.2, g["fan_end"]), (g["fan_w"], 13.2, -8.0), quad="up", cut="tail")]
    if c["bill"]["hook"]:  # the hooked tip, turned down over the jaw
        (bx0, by0, bz0), (bx1, by1, bz1) = g["bill"]
        hw = (bx1 - bx0) / 2 - 0.25
        E.append(el("bill_hook", "head", (-hw, by1 - 0.4 - 1.6 * (by1 - by0) / 2.3, bz1 - 1.1), (hw, by1 - 0.4, bz1 - 0.1),
                    rot=(c["bill"]["rot"] + 38, 0, 0), origin=(0, by1 - 0.4, bz1 - 0.6)))
    for s, n in ((1, "L"), (-1, "R")):
        side = g["side"]
        x0, x1 = sorted((s * (side - 0.4), s * (side + 0.9)))
        # Folded along the side, the primaries crossing over the tail's base.
        E.append(el(f"wingf_{n.lower()}", f"wingf_{n}", (x0, 10.6, -6.0), (x1, 15.9, 7.0)))
        px = s * (side + 0.3)
        E.append(el(f"primariesf_{n.lower()}", f"wingf_{n}", (px, 10.4, g["fold_end"]), (px, 14.4, -4.0), quad="east" if s > 0 else "west",
                    cut="folded"))
        # Open: the arm, the hand, and the flight feathers behind them as cut-out panels.
        ax0, ax1 = sorted((s * (side - 0.8), s * g["arm_end"]))
        ta = max(0.9, 0.42 / c["K"])  # thick enough round its feather panel, however small the bird
        E.append(el(f"wing_{n.lower()}", f"wing_{n}", (ax0, 14.9 - ta, 1.8), (ax1, 14.9 + ta * 0.95, 8.0)))
        hx0, hx1 = sorted((s * (g["hand_x"] - 0.4), s * g["hand_end"]))
        th = max(0.45, 0.4 / c["K"])
        E.append(el(f"hand_{n.lower()}", f"hand_{n}", (hx0, 14.9 - th, 2.2), (hx1, 14.9 + th, 7.0)))
        fx0, fx1 = sorted((s * (side - 0.4), s * (g["hand_x"] - 0.4)))
        E.append(el(f"flight_{n.lower()}", f"wing_{n}", (fx0, 14.9, g["flight_z0"]), (fx1, 14.9, 2.4), quad="down", cut="secondaries"))
        gx0, gx1 = sorted((s * (g["hand_x"] - 0.2), s * g["prim_end"]))
        E.append(el(f"primaries_{n.lower()}", f"hand_{n}", (gx0, 14.9, g["prim_z0"]), (gx1, 14.9, 2.8), quad="down", cut="primaries"))
        x = s * g["leg_x"]
        sh, tf = c["shin"], c["toes"]
        th = max(1.1, sh + 0.6)  # the thigh a size bigger than the shin
        E.append(el(f"thigh_{n.lower()}", f"leg_{n}_thigh", (x - th, 5.9, -2.0 - 2 * (th - 1.1)), (x + th, 11.0, 0.8)))  # feathered
        E.append(el(f"shin_{n.lower()}", f"leg_{n}_shin", (x - sh, 0.8, -0.6 - sh * 1.0), (x + sh, 6.6, -0.6 + sh * 1.0)))
        E.append(el(f"toes_{n.lower()}", f"leg_{n}_toes", (x - 1.1 * tf, 0.0, -0.6 - 2.0 * tf), (x + 1.1 * tf, 1.2, -0.6 + 3.2 * tf)))
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
        return (np.abs(x) <= fw * (0.6 + 0.4 * tt)) & (tt <= 0.97 - 0.06 * (np.floor(np.abs(x) * 1.3 / g["u"]) % 2))  # square, feather ends
    if kind == "secondaries":
        u = g["u"]
        scallop = 0.7 * u * np.abs(np.sin(np.abs(x) * 1.3 / u)) * c["wing"]["chord"]
        return z >= g["flight_z0"] + 2.0 * u * c["wing"]["chord"] * np.clip((np.abs(x) - (g["hand_x"] - 6.0 * u)) / (6.0 * u), 0, 1) + scallop
    if kind == "primaries":
        ax = np.abs(x)
        u = g["u"]
        wig = np.abs(np.sin(ax * 1.1 / u)) * u
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
        fw = 1.6 * u * c["wing"]["chord"]
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
    col = paint_face_smooth(a, tones_, light_face, out=cap, coat=c["feather"], y_world=None, occ=occ, quad=bool(e["quad"]),
                            seed=sum(map(ord, e["name"] + face)) % 97, tuft=False)
    img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
    if e["cut"]:
        img[..., 3] = np.where(cut_mask(e, pr, c), 255, 0)
    eyes = c["eyes"]
    if e["name"] == "skull" and ((eyes["kind"] == "side" and face in ("east", "west")) or (eyes["kind"] == "front" and face == "south")):
        (x0, y0, z0), (x1, y1, z1) = (warp(v, c) for v in G["skull"])
        ey = warp((0, eyes["y"] + c["head_y"], 0), c)[1]
        if eyes["kind"] == "side":
            ez = warp((0, 0, eyes["z"]), c)[2]
            cu = TX * ((z1 - ez) if face == "east" else (ez - z0))
            stamp_eyes(img, [(cu, TX * (y1 - ey))], eyes["pattern"])
        else:
            ex = eyes["x"] * c["K"]
            stamp_eyes(img, [(TX * (-ex - x0), TX * (y1 - ey)), (TX * (ex - x0), TX * (y1 - ey))], eyes["pattern"])
    if e["name"].startswith("eyelid"):  # closed, the lash line is its lower edge
        if eyes["kind"] == "side":
            img[:, 0 if e["name"].endswith("L") else w - 1, :3] = eyes["lash"]
        else:
            img[h - 1, :, :3] = eyes["lash"]
    if e["quad"] and e["cut"] in ("primaries", "secondaries", "folded", "tail"):  # the feathers' shafts and edges
        along = (np.abs(pr[..., 0]) if e["cut"] in ("primaries", "secondaries") else pr[..., 2]) / G["u"]
        img[..., :3] = np.where((np.floor(along * 0.9) % 2 == 0)[..., None], (img[..., :3] * 0.88).astype(np.uint8), img[..., :3])
    if e["name"].startswith("toes_") and face == "south":  # the claws
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


# ------------------------------------------------------------------ build


def build(cfg, out_dir):
    """Lays out, paints and animates a bird; returns (ground spec, flying spec, texture)."""
    c = dict(DEFAULTS)
    c.update(cfg)
    for k in ("bill", "tail", "wing", "flap", "eyes"):
        c[k] = dict(DEFAULTS[k], **cfg.get(k, {}))
    G.clear()
    G.update(geometry(c))
    G.update(c.get("layout", {}).get("G", {}))
    (sx0, sy0, sz0), (sx1, sy1, sz1) = (warp(v, c) for v in G["skull"])
    off = 0.35 if c["TX"] == 1 else 0.15  # the eyelids' hinge just outside the face: closed, they sit over the eyes
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
    ground["fitted"] = {"drop": drop, "lowest": round(float(g["all"] - g["toes"]), 2)}
    fly = {"name": c["name"] + "_fly", "texture": ground["texture"], "bones": bones_json, "elements": [e for e in E if not FOLDED(e["name"])]}
    fly["animations"] = animations(c, fly_anims, names)
    return ground, fly, sheet
