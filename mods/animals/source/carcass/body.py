#!/usr/bin/env python3
"""The dead bodies (notes/crew/plans/carcass-and-skinning.md, slice 3): one lying model per body plan, whole and opened,
built as props with the blocky-model skill's tools and the house style, centred on the carcass's place (r5/r6 rolled
the living model onto its side about its root, which left it beside its own place and, on a deer, legs up).

    ~/.venvs/blocky/bin/python mods/animals/source/carcass/body.py

Plans and their reference animal (the model's size; `animals_core::carcass::Plan` scales each species to it by the
cube root of its mass):
- quadruped (a 70 kg white-tailed deer: 1.6 m nose to tail, chest 0.45 m deep, 0.35 m wide): on its right side, legs
  out straight along the ground, head and neck laid forward;
- bird (a 5 kg vulture or eagle: 0.75 m bill to tail): on its side, one wing half open over it;
- biped (a 30 kg raptor: 2 m with its tail): on its side, the long tail out straight, both legs out;
- reptile (a 40 kg caiman: 2 m): on its belly, legs splayed, the tail behind;
- fish (a 3 kg trout: 0.55 m): flat on its side.
Opened: the upper flank (the back for the reptile) cut open, the meat showing with ribs across it, the guts out on the
ground beside it; the bird's top wing gone with a few feathers about.

All ten share one texture (`textures/entity/carcass/body.png`), so the game's coat painter can repaint whichever is
drawn (`mods/animals/src/carcass.rs`): `body_mask.bin` gives each texel's material (0 coat, 1 underside, 2 dark: hooves,
claws, bill; 3 meat; 4 bone; 255 none), and the painter turns coat and underside to the animal's own colours, keeping
the light painted here.
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
import to_bedrock  # noqa: E402
from boxcheck import check  # noqa: E402
from house import SCALE, TX, material  # noqa: E402
from paint import KS  # noqa: E402
from prop import FACES, el, paint  # noqa: E402

ASSETS = os.path.join(HERE, "..", "..", "assets")
OUT = os.path.join(HERE, "out")

COAT, UNDER, DARK, MEAT, BONE = range(5)
# The default coat is a deer's (the painter recolours from it by channel ratios: keep in step with `carcass.rs`).
COAT_RGB, UNDER_RGB = (124, 80, 50), (212, 194, 166)
MATERIALS = [material("fur", COAT_RGB), material("fur", UNDER_RGB), material("claw"), material("skin", (150, 46, 40)),
             material("horn")]


def b(name, lo, hi, mat=COAT, under=None):
    """A box of `mat`; `under`: (axis, from, sign): its texels past `from` along the axis (toward `sign`) are the
    underside's colour."""
    return el(name, lo, hi, mat=mat, under=under)


def ribs(z0, z1, x0, x1, y, step=6, w=2):
    """Bone bars across an opened flank, from z0 to z1, `y` their bottom."""
    return [b(f"rib{i}", (x0, y, z), (x1, y + 2, z + w), BONE) for i, z in enumerate(range(z0, z1, step))]


# ---- quadruped: a deer on its right side; +x is the belly (legs out that way), -x its back; +z the tail.
BELLY = (0, 6, 1)
quad = [
    b("body", (-15, 0.5, -35), (14, 22, 35), under=BELLY),
    b("chest", (-13, 1.5, -40), (12, 20, -30), under=BELLY),
    b("rump", (-14, 1, 28), (11, 21, 40), under=BELLY),
    b("neck", (-10, 3, -56), (2, 17, -38), under=(0, -2, 1)),
    b("head", (-11, 3.5, -74), (4, 15.5, -55), under=(0, 1, 1)),
    b("muzzle", (-7, 5, -84), (2, 13, -73)),
    b("nose", (-6, 6, -86), (0, 12, -83.5), DARK),
    b("ear", (-17, 8, -60), (-10, 13, -51)),
    b("tail", (-8, 8, 39), (-1, 15, 48), under=(0, -4, 1)),
]
for name, z, y in [("fore_top", -25, 13), ("fore_low", -20, 2.5), ("hind_top", 21, 13.5), ("hind_low", 26, 3)]:
    quad += [b(f"{name}_leg", (12, y, z), (43, y + 6, z + 6)),
             b(f"{name}_hoof", (42.5, y + 0.5, z + 0.5), (48, y + 5.5, z + 5.5), DARK)]
quad_open = quad + [
    b("cut", (-6, 21, -20), (10, 23, 16), MEAT),
    *ribs(-17, 14, -7, 11, 22.5),
    b("gut_a", (16, 0, -10), (29, 8, 2), MEAT),
    b("gut_b", (20, 0.5, 2.5), (32, 6, 12), MEAT),
]

# ---- bird: on its side, bill to -z, legs (+x) tucked; the top wing half open over the body.
bird_body = [
    b("body", (-8, 0.5, -14), (8, 14, 15), under=(0, 3, 1)),
    b("neck", (-5, 2, -21), (3, 11, -13)),
    b("head", (-6, 2.5, -30), (4, 11.5, -20)),
    b("bill", (-3, 5, -37), (2, 9, -29.5), DARK),
    b("tail", (-6, 5, 14), (4, 8, 28)),
    b("leg_a", (7.5, 5, 2), (14, 7, 4), DARK),
    b("leg_b", (7.5, 8, 5), (13, 10, 7), DARK),
]
bird = bird_body + [b("wing", (-7, 14, -7), (11, 15.5, 14)), b("primaries", (-4, 14.5, 13.5), (7, 16, 27))]
bird_open = bird_body + [
    b("cut", (-5, 13, -9), (5, 15, 9), MEAT),
    *ribs(-7, 8, -6, 6, 14.5, step=5, w=1.5),
    b("feather_a", (12, 0, -22), (22, 1, -16)),
    b("feather_b", (-20, 0.5, 4), (-12, 1.5, 9)),
    b("feather_c", (14, 0.25, 18), (22, 1.25, 24)),
]

# ---- biped: a raptor on its side; tail to +z, legs out to +x.
biped = [
    b("body", (-10, 0.5, -16), (10, 16, 16), under=(0, 3, 1)),
    b("neck", (-8, 3, -31), (2, 12, -14)),
    b("head", (-7.5, 2.5, -46), (3.5, 12.5, -30)),
    b("snout", (-5, 4.5, -57), (1, 10, -45.5)),
    b("tail_0", (-7, 3, 15), (3, 12, 40)),
    b("tail_1", (-5, 4.5, 39), (1, 10.5, 64)),
    b("tail_2", (-3.5, 6, 63), (-0.5, 9, 82)),
    b("thigh_top", (8, 8, 2), (25, 18, 12)),
    b("shin_top", (24, 9.5, 3.5), (39, 16.5, 10.5)),
    b("foot_top", (38.5, 9, 3), (45, 17, 11), DARK),
    b("thigh_low", (8.5, 1, 4.5), (23, 8.5, 13.5)),
    b("shin_low", (22.5, 1.5, 6), (35, 7, 12)),
    b("foot_low", (34.5, 1, 5.5), (40, 7.5, 12.5), DARK),
    b("arm", (9, 9.5, -12), (18, 13, -7.5)),
    b("claw", (17.5, 10, -11.5), (21, 12.5, -8), DARK),
]
biped_open = biped + [
    b("cut", (-6, 15, -12), (7, 17, 10), MEAT),
    *ribs(-10, 9, -7, 8, 16.5, step=5),
    b("gut_a", (12, 0, -13), (24, 6, -2), MEAT),
]

# ---- reptile: a caiman on its belly; the back up (+y), the tail to +z, the legs splayed out along x.
SIDE = (1, 4, -1)  # its flanks' lower part, under 4 units, the pale belly
reptile = [
    b("body", (-12, 0.5, -20), (12, 12, 20), under=SIDE),
    b("head", (-8, 1, -37), (8, 9, -19), under=SIDE),
    b("snout", (-5.5, 1.5, -55), (5.5, 6.5, -36.5)),
    b("tail_0", (-9, 1, 19), (9, 10, 45), under=SIDE),
    b("tail_1", (-6, 1.5, 44), (6, 7.5, 70)),
    b("tail_2", (-3.5, 2, 69), (3.5, 5.5, 88)),
]
for name, x0, x1, z in [("fl", 11, 24, -16), ("fr", -24, -11, -16), ("hl", 11, 26, 11), ("hr", -26, -11, 11)]:
    foot = (x1 - 0.5, x1 + 4) if x0 > 0 else (x0 - 4, x0 + 0.5)
    reptile += [b(f"leg_{name}", (x0, 1, z), (x1, 5, z + 5)),
                b(f"foot_{name}", (foot[0], 0.5, z - 1), (foot[1], 3, z + 6), DARK)]
reptile_open = reptile + [
    b("cut", (-7, 11, -13), (7, 13, 13), MEAT),
    *ribs(-11, 12, -8, 8, 12.5),
    b("gut_a", (14, 0, -6), (26, 6, 6), MEAT),
]

# ---- fish: flat on its side, head to -z, belly to +x.
fish = [
    b("body", (-8, 0.5, -14), (8, 6.5, 16), under=(0, 2, 1)),
    b("head", (-6.5, 1, -22), (6.5, 6, -13.5), under=(0, 2, 1)),
    b("tail", (-9, 2.5, 15.5), (9, 4, 25)),
    b("fin", (-12, 2, -4), (-7.5, 5, 8)),
]
fish_open = fish + [
    b("cut", (-3, 6, -8), (6, 7.5, 10), MEAT),
    b("gut_a", (10, 0, -6), (16, 3, 2), MEAT),
]

MODELS = [("carcass_quadruped", quad), ("carcass_quadruped_open", quad_open), ("carcass_bird", bird),
          ("carcass_bird_open", bird_open), ("carcass_biped", biped), ("carcass_biped_open", biped_open),
          ("carcass_reptile", reptile), ("carcass_reptile_open", reptile_open), ("carcass_fish", fish),
          ("carcass_fish_open", fish_open)]
APART = 400  # units each model is painted apart from the next (so none shades another)


def area(e, face, p, uu, vv, h):
    a = np.full(p.shape[:2], e["mat"], int)
    if e["under"]:
        ax, at, sgn = e["under"]
        q = p[..., ax] - (e["model"] * APART if ax == 0 else 0)  # painted apart along x (`APART`)
        a[(q - at) * sgn > 0] = UNDER
    return a


def main():
    elements = []
    for i, (_, els) in enumerate(MODELS):
        hits = check(els, gap=0.3)
        assert not hits, f"{MODELS[i][0]}: faces in one plane {hits}"
        for e in els:
            moved = dict(e, lo=[e["lo"][0] + i * APART, *e["lo"][1:]], hi=[e["hi"][0] + i * APART, *e["hi"][1:]],
                         name=f"{i}_{e['name']}", model=i)
            elements.append(moved)
    # Crisp areas (no coat merging): softened, the belly line moved off the mask the painter reads.
    sheet, uv = paint(elements, area, MATERIALS, coat=(), width=256, square=False, seed=3, tx=TX)
    # The same layout painted in flat ids: the mask the game's painter reads.
    flat = [np.tile(np.array([m, 0, 0], float), (len(KS), 1)) for m in range(len(MATERIALS))]
    ids, uv2 = paint(elements, area, flat, coat=(), width=256, square=False, seed=3, tx=TX)
    assert uv2 == uv and ids.shape == sheet.shape
    mask = np.where(ids[..., 3] > 0, ids[..., 0], 255).astype(np.uint8)
    assert set(np.unique(mask)) <= {0, 1, 2, 3, 4, 255}, np.unique(mask)
    h, w = mask.shape
    os.makedirs(OUT, exist_ok=True)
    tex = os.path.join(OUT, "body.png")
    Image.fromarray(sheet).save(tex)
    with open(os.path.join(HERE, "body_mask.bin"), "wb") as f:
        f.write(np.array([w, h], "<u2").tobytes() + mask.tobytes())
    for i, (name, els) in enumerate(MODELS):
        mine = [(j, e) for j, e in enumerate(elements) if e["model"] == i]
        spec_els = []
        for j, e in mine:
            faces = {f: uv.get((j, f)) for f in FACES}
            spec_els.append({"name": e["name"], "lo": [e["lo"][0] - i * APART, *e["lo"][1:]],
                             "hi": [e["hi"][0] - i * APART, *e["hi"][1:]], "quad": None, "rot": None, "origin": None,
                             "bone": "root", "faces": faces})
        spec = {"name": name, "texture": [w, h], "bones": [{"name": "root", "parent": None, "pivot": [0.0, 0.0, 0.0]}],
                "elements": spec_els, "animations": {}}
        with open(os.path.join(OUT, f"{name}.spec.json"), "w") as f:
            json.dump(spec, f)
        to_bedrock.export(spec, tex, ASSETS, name, SCALE, "carcass/body", pose_files={"": {}})
        # No animation file: a carcass doesn't move.
        anim = os.path.join(ASSETS, "animations", f"{name}.animation.json")
        if os.path.exists(anim):
            os.remove(anim)
    print(f"{len(MODELS)} models, texture {w}x{h}")
    subprocess.run([sys.executable, os.path.join(HERE, "sheet.py"), *sys.argv[1:]], check=False)


if __name__ == "__main__":
    main()
