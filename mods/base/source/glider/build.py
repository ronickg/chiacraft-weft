#!/usr/bin/env python3
"""Builds the open glider's wing, drawn on every player's body while they glide
(`mods/base/src/glider.rs`: `r.bone_model(&PLAYER, "base:glider_wing", "body")`, hidden on everyone
not gliding), with the blocky-model skill's prop tools and the house style (32 texels a block):
`../../assets/models/glider_wing.geo.json` and `../../assets/textures/glider_wing.png`.

    ~/.venvs/blocky/bin/python build.py [out_dir]     (default: into the mod's assets)

The real thing: the earliest hang gliders were a sail of hide or cloth on a frame of poles, the
pilot hanging under it by the hands. Ours is the leather glider (`vitals::glide::HIDE`): a delta
wing 3 blocks across and about 1.4 deep, its hide in panels stepping down toward the tips (the
sail sags between the battens), a keel and two leading-edge sticks meeting at the nose, a crossbar under the hide, two
struts down to the bar the player holds over the head (the `glide` clip raises the arms to it).

Layout units are 1/64 block (`house.UNITS`), y up from the feet, the model facing +z (export turns
it to face -z); the player's head tops out at 128, its raised hands near 136.
"""

import json
import math
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
SKILL = os.path.expanduser("~/.claude/skills/blocky-model/scripts")
sys.path.insert(0, SKILL)
from house import SCALE, TX, material  # noqa: E402
from prop import el, entity_spec, paint  # noqa: E402
import boxcheck  # noqa: E402
import to_bedrock  # noqa: E402

ASSETS = os.path.join(HERE, "..", "..", "assets")
HIDE, HIDE_DARK, SEAM, STICK, STRING = range(5)
MATERIALS = [material("skin", (138, 88, 58)), material("skin", (116, 72, 48)), material("skin", (88, 54, 38)),
             material("twig"), material("straw", (226, 210, 172))]

WING_Y = 176.0
# The delta: its nose, its trailing edge and its tips (units).
NOSE_Z, BACK_Z, TIP_X = 48.0, -34.0, 96.0


def half_width(z):
    """How far out the hide reaches at `z`: the leading edges from the nose to the tips."""
    return TIP_X * (NOSE_Z - z) / (NOSE_Z - BACK_Z)


def trailing(x):
    """The trailing edge's z at `x`: scalloped, sagging in between the keel, the struts' line and
    the tips."""
    return BACK_Z + 4.0 * np.sin(np.pi * np.abs(x) / 48.0) ** 2


def cut(e, p):
    x, z = p[..., 0], p[..., 2]
    return (np.abs(x) <= half_width(z)) & (z >= trailing(x))


def layout():
    E = []
    # The hide: one panel cut to the delta (a quad, painted on both sides), with the keel's
    # stitched seams and a scalloped trailing edge between the battens.
    E.append(el("hide", (-TIP_X, WING_Y, BACK_Z - 1), (TIP_X, WING_Y, NOSE_Z), quad="up", cut="delta", part="hide"))
    # The frame: a keel down the middle on top, two leading-edge sticks along the front edges,
    # a crossbar under the hide, struts down to the bar in the hands.
    E.append(el("keel", (-1.5, WING_Y + 1.5, -40), (1.5, WING_Y + 4.5, 47), part="stick"))
    for s, side in ((-1, "L"), (1, "R")):
        # From the nose out and back to the tip: along its own x, turned about y.
        L = math.hypot(TIP_X, NOSE_Z - BACK_Z)
        ang = math.degrees(math.atan2(NOSE_Z - BACK_Z, TIP_X))
        E.append(el(f"edge_{side}", (0, WING_Y + 0.5, NOSE_Z - 1.5), (s * L, WING_Y + 3.5, NOSE_Z + 1.5),
                    rot=(0, s * ang, 0), origin=(0, WING_Y, NOSE_Z), part="stick"))
        E.append(el(f"strut_{side}", (s * 38.5, 139.0, -4.5), (s * 41.5, 173.5, -1.5), part="stick"))
    E.append(el("crossbar", (-90, 171.5, -6.0), (90, 174.5, -2.0), part="stick"))
    E.append(el("bar", (-44, 136.0, -3.5), (44, 140.0, 0.5), part="stick"))
    for e in E:  # boxes from `el` keep lo <= hi
        lo, hi = np.minimum(e["lo"], e["hi"]), np.maximum(e["lo"], e["hi"])
        e["lo"], e["hi"] = list(map(float, lo)), list(map(float, hi))
    return E


def area(e, face, p, uu, vv, h):
    if e["part"] == "stick":
        out = np.full(p.shape[:-1], STICK)
        # Lashings where sticks cross: string wrapped round the bar at the struts, the struts at
        # the crossbar, the crossbar at the keel and the struts.
        x, y = np.abs(p[..., 0]), p[..., 1]
        if e["name"] == "bar":
            out[(x > 36) & (x < 44)] = STRING
        elif e["name"].startswith("strut"):
            out[(y < 145) | (y > 168)] = STRING
        elif e["name"] == "crossbar":
            out[(x < 5) | ((x > 36) & (x < 44))] = STRING
        return out
    out = np.full(p.shape[:-1], HIDE)
    x, z = np.abs(p[..., 0]), p[..., 2]
    # Three hides sewn together: seams running from the nose out to the trailing edge, darker
    # hides on the outer panels, a hem along the trailing edge and the leading edges.
    share = x / np.maximum(half_width(z), 1e-3)
    out[share > 0.62] = HIDE_DARK
    out[np.abs(share - 0.33) < 0.035] = SEAM
    out[np.abs(share - 0.62) < 0.035] = SEAM
    out[z < trailing(x) + 2.5] = SEAM
    out[share > 0.95] = SEAM
    return out


def build(out_dir):
    E = layout()
    sheet, uv = paint(E, area, MATERIALS, cut=cut, ground=False, width=256, tx=TX, smooth=0)
    spec = entity_spec(E, uv, sheet, "glider_wing", bone="glider")
    bad = boxcheck.check(spec["elements"])
    assert not bad, bad
    geo = to_bedrock.geometry(spec, "geometry.glider_wing", SCALE)
    os.makedirs(os.path.join(out_dir, "models"), exist_ok=True)
    os.makedirs(os.path.join(out_dir, "textures"), exist_ok=True)
    to_bedrock.write(os.path.join(out_dir, "models", "glider_wing.geo.json"), geo)
    Image.fromarray(sheet).save(os.path.join(out_dir, "textures", "glider_wing.png"))
    with open(os.path.join(out_dir, "glider_wing.spec.json"), "w") as f:
        json.dump(spec, f)
    print("texture", sheet.shape[1], "x", sheet.shape[0], "elements", len(E), "coplanar", bad)
    return spec


if __name__ == "__main__":
    build(sys.argv[1] if len(sys.argv) > 1 else ASSETS)
