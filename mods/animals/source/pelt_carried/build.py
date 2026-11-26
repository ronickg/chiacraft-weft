#!/usr/bin/env python3
"""Builds the pelts a player carries, drawn on every player's body while a hide or a fur is in their
inventory (`mods/animals/src/pelt_look.rs`: `r.bone_model(&PLAYER, "animals:pelt_hide", "body")` and the
fur's, each hidden on everyone not carrying one), with the blocky-model skill's prop tools and the house
style (32 texels a block): `../../assets/models/pelt_{hide,fur}.geo.json` and
`../../assets/textures/pelt_{hide,fur}.png`.

    ~/.venvs/blocky/bin/python build.py [out_dir]     (default: into the mod's assets)

Look only (Planner, 11-26: a carried pelt shows on the player; no slowdown, no shoulder slot). The two
match their icons (`mods/base/assets/textures/item/{hide,fur}.png`): the hide a flat skin, here folded
over the left shoulder, a flap down the chest and a longer one down the back ending in two legs; the fur
rolled, slung across the back under the shoulder blades on a strap from the left shoulder to the right
hip, two bands round the roll.

Layout units are 1/64 block (`house.UNITS`), y up from the feet, the player facing +z (export turns it to
face -z). The player (`mods/base/source/clothes/build.py`): body x -16..16, y 48..96, z -8..8; the left
arm x 16..32; the head y 96..128, z -16..16; a coat stands 1.2 off the body. So everything here stays
2 or more off the body and clear of the arms' swing (no flap in front of or behind an arm).
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

# The hide's icon: hair side (182, 154, 130) to (162, 134, 110), a darker edge. The fur's: (146, 106, 90),
# its rolled layers (130, 90, 78), the inner side paler (182, 142, 122). Straps of tanned leather.
HAIR, EDGE, FUR, LAYER, INNER, STRAP = range(6)
MATERIALS = [material("skin", (176, 148, 124)), material("skin", (140, 112, 90)), material("fur", (146, 106, 90)),
             material("fur", (118, 80, 66)), material("fur", (182, 142, 122)), material("skin", (96, 62, 44))]


def strap(name, z0, z1):
    """From the left shoulder (x 14, y 98) to the right hip (x -14, y 62), 3 wide, on the chest or back."""
    (x0, y0), (x1, y1) = (14.0, 98.0), (-14.0, 62.0)
    length = math.hypot(x1 - x0, y1 - y0)
    ang = math.degrees(math.atan2(x0 - x1, y0 - y1))
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    return el(name, (cx - 1.5, cy - length / 2, z0), (cx + 1.5, cy + length / 2, z1), rot=(0, 0, -ang),
              origin=(cx, cy, (z0 + z1) / 2), part="strap")


def hide_layout():
    return [
        # Over the shoulder and the arm's top, its inner end under the head, and down the arm's outside
        # (an arm swings about x: its outer face stays in its plane, so a flap past it never meets it).
        el("fold", (13.0, 97.0, -13.0), (34.5, 101.0, 12.0), part="hide"),
        el("side", (33.0, 84.0, -8.5), (35.5, 100.0, 8.0), part="hide"),
        el("front", (5.0, 76.0, 10.0), (16.5, 99.5, 12.5), part="hide"),
        el("back", (-7.0, 60.0, -12.5), (16.5, 99.0, -10.0), part="hide"),
        # A foreleg off the front flap's foot, the hind legs off the back flap's.
        el("leg_F", (11.0, 69.0, 10.5), (14.5, 76.5, 12.0), part="hide"),
        el("leg_L", (10.5, 51.0, -12.0), (14.5, 60.5, -10.5), part="hide"),
        el("leg_R", (-5.5, 53.0, -12.0), (-1.5, 60.5, -10.5), part="hide"),
    ]


def fur_layout():
    return [
        el("roll", (-15.0, 76.0, -22.0), (15.0, 88.0, -10.5), part="roll"),
        el("band_L", (7.5, 75.5, -22.5), (10.5, 88.5, -10.0), part="strap"),
        el("band_R", (-10.5, 75.5, -22.5), (-7.5, 88.5, -10.0), part="strap"),
        strap("strap_front", 9.8, 10.8),
        strap("strap_back", -10.4, -9.6),
    ]


def area(e, face, p, uu, vv, h):
    shape = p.shape[:-1]
    if e["part"] == "strap":
        return np.full(shape, STRAP)
    if e["part"] == "hide":
        out = np.full(shape, HAIR)
        # A darker rim where the skin was cut, and down the legs' tips.
        y = p[..., 1]
        if face in ("east", "west", "down") and e["name"] != "fold":
            out[:] = EDGE
        if e["name"].startswith("leg"):
            out[y < p[..., 1].min() + 2.5] = EDGE
        return out
    # The roll: fur outside, its ends a spiral of fur and paler inner side.
    out = np.full(shape, FUR)
    if face in ("east", "west"):
        cy, cz = 82.0, -16.25
        r = np.hypot(p[..., 1] - cy, p[..., 2] - cz)
        a = np.arctan2(p[..., 1] - cy, p[..., 2] - cz)
        # Turns 2.5 units apart, growing outward with the angle.
        turn = (r - 2.5 * (a + math.pi) / (2 * math.pi)) % 2.5
        out[turn < 1.0] = INNER
        out[turn > 2.0] = LAYER
    else:
        # Lengthwise streaks of the hair along the roll.
        z = p[..., 2]
        out[np.sin(z * 1.3) > 0.85] = LAYER
    return out


def build_one(out_dir, name, E):
    for e in E:  # boxes from `el` keep lo <= hi
        lo, hi = np.minimum(e["lo"], e["hi"]), np.maximum(e["lo"], e["hi"])
        e["lo"], e["hi"] = list(map(float, lo)), list(map(float, hi))
    sheet, uv = paint(E, area, MATERIALS, ground=False, width=128, tx=TX, smooth=0)
    spec = entity_spec(E, uv, sheet, name, bone=name)
    bad = boxcheck.check(spec["elements"])
    assert not bad, bad
    geo = to_bedrock.geometry(spec, f"geometry.{name}", SCALE)
    os.makedirs(os.path.join(out_dir, "models"), exist_ok=True)
    os.makedirs(os.path.join(out_dir, "textures"), exist_ok=True)
    to_bedrock.write(os.path.join(out_dir, "models", f"{name}.geo.json"), geo)
    Image.fromarray(sheet).save(os.path.join(out_dir, "textures", f"{name}.png"))
    # The spec for review renders (`render.py`), beside the build, never into the assets.
    os.makedirs(os.path.join(HERE, "out"), exist_ok=True)
    with open(os.path.join(HERE, "out", f"{name}.spec.json"), "w") as f:
        json.dump(spec, f)
    print(name, "texture", sheet.shape[1], "x", sheet.shape[0], "elements", len(E), "coplanar", bad)
    return spec


def build(out_dir):
    return build_one(out_dir, "pelt_hide", hide_layout()), build_one(out_dir, "pelt_fur", fur_layout())


if __name__ == "__main__":
    build(sys.argv[1] if len(sys.argv) > 1 else ASSETS)
