#!/usr/bin/env python3
"""The pool of blood and guts under an opened carcass (`carcass_gore`) and its texture (`carcass/bones`). The bodies,
their bones and piles are `body.py`'s. Run from the repo root: python3 mods/animals/source/carcass/build.py"""
import json, os, random
from PIL import Image

ROOT = os.path.join(os.path.dirname(__file__), "..", "..", "assets")
BONE, FLESH = [0, 0], [0, 32]  # box-UV corners: the bone region (48 x 24) and the flesh one (24 x 8)


def cube(o, s, uv=BONE):
    return {"origin": o, "size": s, "uv": uv}


def geo(name, cubes, w, h):
    return {"format_version": "1.12.0", "minecraft:geometry": [{
        "description": {"identifier": f"geometry.{name}", "texture_width": 64, "texture_height": 64,
                        "visible_bounds_width": w, "visible_bounds_height": h, "visible_bounds_offset": [0, 0.5, 0]},
        "bones": [{"name": "root", "pivot": [0, 0, 0], "cubes": cubes}]}]}


# Under an opened body: a pool of blood (6 px tiles side by side, none overlapping, so no top fights another) and the
# guts on it, wider than the body lying on it so it shows round it.
gore = [cube([x, 0, z], [6, 1, 6], FLESH) for x in (-12, -6, 0, 6) for z in (-12, -6, 0, 6)
        if (x, z) not in [(-12, -12), (6, 6), (6, -12)]]
gore += [cube([-9, 1, 7], [3, 2, 3], FLESH), cube([8, 1, -3], [3, 2, 4], FLESH), cube([9, 1, 2], [2, 1, 3], FLESH)]
for name, cubes, w, h in [("carcass_gore", gore, 2, 1)]:
    with open(os.path.join(ROOT, "models", f"{name}.geo.json"), "w") as f:
        json.dump(geo(name, cubes, w, h), f, indent=1)

random.seed(7)
img = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
for x in range(48):
    for y in range(24):
        v = random.randint(-14, 10)
        img.putpixel((x, y), (214 + v, 204 + v, 178 + v, 255))
for x in range(24):
    for y in range(32, 40):
        v = random.randint(-20, 16)
        img.putpixel((x, y), (88 + v, 16 + v // 3, 14 + v // 3, 255))  # dark, like blood soaked in (r6: a bright red slab)
d = os.path.join(ROOT, "textures", "entity", "carcass")
os.makedirs(d, exist_ok=True)
img.save(os.path.join(d, "bones.png"))
