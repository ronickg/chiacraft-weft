#!/usr/bin/env python3
"""Builds the egg (an entity with no skeleton) with the blocky-model skill's prop tools and the
game's house style: `../../assets/models/egg.geo.json` and `textures/entity/egg/egg.png`.

    ../dog/chihuahua/.venv/bin/python build.py              (the chosen markings, into the mod)
    ../dog/chihuahua/.venv/bin/python build.py --variants   (every marking into out/variants/)

The real thing: Velociraptor's eggs aren't known. Dromaeosaur eggs are elongated, about twice as
long as wide, with a blunt and a pointed end; their relatives stood them in the nest nearly
upright, pointed end down, half buried, tilted with the top toward the nest's middle. The eggshell
of the relative Deinonychus held both pigments birds' eggs have (blue-green biliverdin, red-brown
protoporphyrin): a colored, marked egg, as suits an open nest. Birds' marks gather toward the blunt
end, darker marks over paler, greyish ones lying deeper in the shell. A Velociraptor's egg here is
about 15 cm long (10 units), 8 wide; a bigger species' are scaled up in the game (`eggs.rs`).

Markings (one layout, a texture each, to choose from):
- blotched: blue-green, dark brown blotches gathered in a ring near the blunt end, pale lilac-grey
  ones under them, a few small spots lower down;
- speckled: cream, fine rufous speckles all over, denser at the top;
- rufous: washed red-brown, heavily marked darker (like a kestrel's);
- plain: deep green, almost unmarked (like an emu's).
"""

import json
import math
import os
import subprocess
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", ".."))
SKILL = os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts")
sys.path.insert(0, SKILL)
from house import SCALE, material  # noqa: E402
from prop import el, entity_spec, paint  # noqa: E402

ASSETS = os.path.join(HERE, "..", "..", "assets")
OUT = os.path.join(HERE, "out")
HEIGHT = 10.0
TILT = 20  # degrees, the top toward +z (the nest's middle when laid)

# name: (shell, deep marks, surface marks)
MARKINGS = {
    "blotched": ((164, 198, 186), (160, 168, 170), (104, 74, 54)),
    "speckled": ((228, 216, 190), (206, 188, 168), (150, 86, 56)),
    "rufous": ((206, 160, 122), (176, 120, 88), (120, 62, 42)),
    "plain": ((70, 104, 80), (84, 118, 92), (58, 86, 66)),
}
CHOSEN = "blotched"  # chosen by Ronald, 2026-11-02
SHELL, DEEP, MARK = range(3)


def elements(tilt=TILT, yaw=0.0, at=(0.0, 0.0, 0.0), name="egg"):
    """Seven layers from the pointed end (down) to the blunt end, rounding off at both ends, the
    widest a little above the middle (toward the blunt end); neighbors differ in width by a unit
    and reach half a unit into each other, so no faces meet."""
    layers = [(1.5, 0, 1), (3, 0.5, 2.5), (4, 2, 4.5), (5, 4, 7.5), (4, 7, 8.75), (3, 8.25, 9.5), (2, 9, 10)]
    x, y, z = at
    return [el(f"{name}{k}", (x - w / 2, y + y0, z - w / 2), (x + w / 2, y + y1, z + w / 2),
               rot=(tilt, yaw, 0) if tilt or yaw else None, origin=at, part="egg", at=at, yaw=yaw)
            for k, (w, y0, y1) in enumerate(layers)]


def _blobs(rng, n, y_range, size):
    return [(rng.uniform(0, 2 * math.pi), rng.uniform(*y_range), rng.uniform(*size), rng.uniform(0, 6)) for _ in range(n)]


def area(p, kind, tilt=TILT, origin=(0, 0, 0), yaw=0.0):
    """The markings at points `p` on an egg standing at `origin`, by angle round it and height up
    it (its turn and tilt undone), so they wrap round the corners."""
    t, r = math.radians(tilt), math.radians(yaw)
    q = p - np.array(origin, float)
    q = np.stack([math.cos(r) * q[..., 0] - math.sin(r) * q[..., 2], q[..., 1],
                  math.sin(r) * q[..., 0] + math.cos(r) * q[..., 2]], axis=-1)
    y = q[..., 1] * math.cos(t) + q[..., 2] * math.sin(t)  # along the egg
    z = -q[..., 1] * math.sin(t) + q[..., 2] * math.cos(t)
    th = np.arctan2(z, q[..., 0])
    a = np.full(p.shape[:-1], SHELL)
    rng = np.random.default_rng({"blotched": 3, "speckled": 4, "rufous": 5, "plain": 6}[kind])

    def stamp(blobs, mat, rough=0.35):
        nonlocal a
        for b_th, b_y, r, ph in blobs:
            d_th = (th - b_th + math.pi) % (2 * math.pi) - math.pi
            u, v = d_th * 2.4, y - b_y  # 2.4 units round per radian at the widest
            ang = np.arctan2(v, u)
            edge = r * (1 + rough * np.sin(3 * ang + ph))
            a = np.where(np.hypot(u, v) < edge, mat, a)

    if kind == "blotched":
        stamp(_blobs(rng, 4, (6.5, 8.8), (0.6, 0.9)), DEEP)
        stamp(_blobs(rng, 8, (6.8, 9.4), (0.5, 0.9)), MARK)
        stamp(_blobs(rng, 3, (2.5, 5.5), (0.35, 0.5)), MARK, rough=0.1)
    elif kind == "speckled":
        stamp(_blobs(rng, 6, (4.0, 9.0), (0.4, 0.6)), DEEP, rough=0.2)
        stamp(_blobs(rng, 22, (1.0, 9.8), (0.28, 0.42)), MARK, rough=0.1)
        stamp(_blobs(rng, 10, (7.5, 9.8), (0.3, 0.45)), MARK, rough=0.1)
    elif kind == "rufous":
        stamp(_blobs(rng, 10, (2.0, 9.5), (0.7, 1.1)), DEEP)
        stamp(_blobs(rng, 12, (3.0, 9.8), (0.4, 0.75)), MARK)
    else:
        stamp(_blobs(rng, 5, (2.0, 9.0), (0.6, 1.0)), DEEP, rough=0.5)
    return a


def materials(kind):
    shell, deep, mark = MARKINGS[kind]
    return [material("shell", shell), material("shell", deep), material("shell", mark)]


def build(kind, out):
    os.makedirs(out, exist_ok=True)
    E = elements()
    down = lambda e, face, p: np.clip((HEIGHT - p[..., 1]) / HEIGHT, 0, 1)  # lit as one piece
    sheet, uv = paint(E, lambda e, f, p, uu, vv, h: area(p, kind), materials(kind), ground=False, width=32, down=down)
    return E, sheet, uv


if __name__ == "__main__":
    from to_bedrock import export  # noqa: E402

    if "--variants" in sys.argv:
        for k in MARKINGS:
            _, sheet, _ = build(k, os.path.join(OUT, "variants"))
            Image.fromarray(sheet, "RGBA").save(os.path.join(OUT, "variants", f"{k}.png"))
            print("built", k)
    E, sheet, uv = build(CHOSEN, OUT)
    Image.fromarray(sheet, "RGBA").save(os.path.join(OUT, "texture.png"))
    spec = entity_spec(E, uv, sheet, "egg")
    with open(os.path.join(OUT, "spec.json"), "w") as f:
        json.dump(spec, f)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    if "--variants" not in sys.argv:
        made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "egg", SCALE, "egg/egg", pose_files={"": {}})
        print("wrote", made, "(", CHOSEN, ")")
