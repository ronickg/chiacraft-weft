#!/usr/bin/env python3
"""Builds a bird's nest: a small cup of twigs, lined with dry grass and a little down, for the
birds' eggs (`eggs.rs` `NEST_CUP`; the raptors' and the other ground-nesters' earthen scrape is
`build.py`). Tester 2's eggs playtest (11-06) had ravens laying two small eggs in the raptors' big
dish; Planner: "give birds their own small cup nest, in a tree when there is one".

    ~/.venvs/blocky/bin/python cup.py            (into the mod: models/nest_cup.geo.json,
                                                  textures/entity/nest/cup.png)
    ~/.venvs/blocky/bin/python cup.py --out DIR  (review only)

The real thing: a songbird's or crow's nest is a cup: an outer basket of twigs woven round, an
inner cup of grass, bark strips and moss, a lining of feathers, hair or down; its rim a little
ragged with twig ends. Built at the house's 32 texels a block like everything else, 14 px across
(the eggs.rs cup is made for eggs 0.12 block wide and scaled up for bigger ones: an eagle's is an
eyrie of a block or more).
"""

import json
import os
import subprocess
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", ".."))
SKILL = os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts")
sys.path.insert(0, SKILL)
from house import SCALE, TX, material  # noqa: E402
from paint import tones  # noqa: E402
from prop import el, entity_spec, paint  # noqa: E402

ASSETS = os.path.join(HERE, "..", "..", "assets")
OUT = os.path.join(HERE, "out", "cup")

TWIG, TWIG_DARK, GRASS, DOWN = range(4)
MATERIALS = [material("twig", (128, 94, 60)), material("twig", (92, 66, 44)), material("straw", (196, 172, 116)),
             tones((236, 222, 196), 0.06)]


def elements():
    """The cup (units, 1/64 block): four walls round a lined floor, the side walls a half unit
    lower than the front and back and the bottoms staggered, so no faces share a plane; the twig
    ends on the rim are painted (boxes for them left strips under two texels)."""
    return [
        el("floor", (-10, 1, -10), (10, 3, 10), part="floor"),
        el("wall_n", (-14, 0, -14), (14, 8, -8), part="wall"),
        el("wall_s", (-14, 0, 8), (14, 8, 14), part="wall"),
        el("wall_w", (-13.6, 0.4, -8.5), (-8, 7.5, 8.5), part="wall"),
        el("wall_e", (8, 0.4, -8.5), (13.6, 7.5, 8.5), part="wall"),
    ]


def hashed(x, y, seed):
    h = (x * 374761393 + y * 668265263 + seed * 2147483647) & 0xFFFFFFFF
    h = ((h ^ (h >> 13)) * 1274126177) & 0xFFFFFFFF
    return (h & 0xFFFF) / 65535.0


def area(e, face, p, uu, vv, h):
    """Twigs woven round the walls: dark gaps between lighter stems running across, broken; the
    inner cup grass, its middle down; the twigs across the rim plain bark."""
    x, y, z = p[..., 0], p[..., 1], p[..., 2]
    a = np.full(p.shape[:-1], TWIG)
    if e["part"] == "twig":
        return np.where((np.floor(x * TX) + np.floor(z * TX)) % 5 == 0, TWIG_DARK, TWIG)
    if e["part"] == "floor":
        r = np.hypot(x, z)
        a = np.where(r < 6.5, DOWN, GRASS)
        return np.where((r >= 6.5) & (np.vectorize(lambda i, j: hashed(int(i), int(j), 3))(np.floor(x * TX), np.floor(z * TX)) > 0.7), TWIG, a)
    row = np.floor(y * TX).astype(int)
    col = np.floor((x + z) * TX).astype(int)
    gap = np.vectorize(lambda r, c: hashed(r, c // 3, 7))(row, col) > 0.62
    a = np.where(gap & (row % 2 == 0), TWIG_DARK, a)
    if face == "up":  # the rim: grass tucked in between twig ends
        a = np.where(np.vectorize(lambda i, j: hashed(int(i), int(j), 11))(np.floor(x * TX), np.floor(z * TX)) > 0.55, GRASS, TWIG)
    return a


def build(out):
    os.makedirs(out, exist_ok=True)
    E = elements()
    sheet, uv = paint(E, area, MATERIALS, width=64, seed=5, tx=TX, smooth=0, ground=False)  # a small prop: no darkening by the ground
    Image.fromarray(sheet, "RGBA").save(os.path.join(out, "texture.png"))
    spec = entity_spec(E, uv, sheet, "nest_cup")
    with open(os.path.join(out, "spec.json"), "w") as f:
        json.dump(spec, f)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(out, "spec.json")], check=True)
    from linecheck import boxes_from_spec, density_of, lines  # noqa: E402
    # Meant: the walls' inner faces under the lined floor (a strip between the ground and the floor,
    # closed in by the floor and the ground, never seen).
    allow = ("wall_n.south", "wall_s.north", "wall_w.east", "wall_e.west")
    thin = [h for h in lines(boxes_from_spec(spec), sheet, density_of(boxes_from_spec(spec)), allow=allow) if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    return spec


if __name__ == "__main__":
    out = sys.argv[sys.argv.index("--out") + 1] if "--out" in sys.argv else OUT
    spec = build(out)
    if "--out" not in sys.argv:
        from to_bedrock import export  # noqa: E402

        print("wrote", export(spec, os.path.join(out, "texture.png"), ASSETS, "nest_cup", SCALE, "nest/cup", pose_files={"": {}}))
