#!/usr/bin/env python3
"""Placeholder carcass looks (notes/crew/plans/carcass-and-skinning.md, slice 1) until the blocky-model ones per body
plan: `carcass_bones` (a ribcage on its spine, the skull, legs, a little flesh left) and `carcass_pile` (scattered
bones), both at a 70 kg deer's size (`animals_core::carcass::REF_KG`), and their one texture. Run from the repo root:
python3 mods/animals/source/carcass/build.py"""
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


bones = [cube([-1, 3, -11], [2, 2, 22])]  # spine
for z in range(-7, 5, 2):  # ribs: a side, the top
    bones += [cube([-5, 0, z], [1, 5, 1]), cube([-5, 5, z], [4, 1, 1]),
              cube([4, 0, z], [1, 5, 1]), cube([1, 5, z], [4, 1, 1])]
bones += [cube([-3, 2, 7], [6, 2, 3]),  # pelvis
          cube([-2, 1, -16], [4, 4, 5]), cube([-1.5, 0, -16], [3, 1, 4]),  # skull, jaw
          cube([4, 0, -9], [7, 1, 1]), cube([4, 0, -6], [6, 1, 1]),  # legs, lying out
          cube([4, 0, 8], [7, 1, 1]), cube([4, 0, 10], [6, 1, 1]),
          cube([-3, 1, -4], [6, 2, 6], FLESH), cube([-2, 2, 6], [4, 1, 3], FLESH)]
pile = [cube([-2, 0, -2], [4, 3, 4]),  # the skull
        cube([-6, 0, -5], [8, 1, 1]), cube([-4, 0, 3], [1, 1, 7]), cube([3, 0, -6], [1, 1, 8]),
        cube([-7, 1, 1], [9, 1, 1]), cube([0, 0, 4], [6, 1, 1]), cube([-5, 0, -1], [2, 1, 2])]
# Under an opened body: a pool of blood (6 px tiles side by side, none overlapping, so no top fights another) and the
# guts on it, wider than the body lying on it so it shows round it.
gore = [cube([x, 0, z], [6, 1, 6], FLESH) for x in (-12, -6, 0, 6) for z in (-12, -6, 0, 6)
        if (x, z) not in [(-12, -12), (6, 6), (6, -12)]]
gore += [cube([-9, 1, 7], [3, 2, 3], FLESH), cube([8, 1, -3], [3, 2, 4], FLESH), cube([9, 1, 2], [2, 1, 3], FLESH)]
for name, cubes, w, h in [("carcass_bones", bones, 2, 1), ("carcass_pile", pile, 1, 1), ("carcass_gore", gore, 2, 1)]:
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
        img.putpixel((x, y), (122 + v, 34 + v // 2, 30 + v // 2, 255))
d = os.path.join(ROOT, "textures", "entity", "carcass")
os.makedirs(d, exist_ok=True)
img.save(os.path.join(d, "bones.png"))
