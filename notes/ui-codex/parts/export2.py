"""The parts art in the exact files the crafting hook reads (crafting session, e7afa85e), laid out under
deliver/textures/ like mods/base/assets/textures/, ready to copy after Ronald's pick:

  parts/<kind>/<shape>/<art>.png painted head (with its lashing or collar) where its drawing has it
                                 (felling_axe_head, pick_head, shovel_blade, hoe_blade)
  parts/<kind>/haft.png          the haft as a shade map (R = step x 36, G = region 0, A), where it meets the iron head
  parts/sword/straight_blade/<art>.png  painted blade (the blade alone), where its drawing has it
  parts/sword/grip.png           the hilt as a map, regions 0 grip, 1 guard, 2 pommel, where it meets the iron blade
  ramps/<id>.png                 8 x 1 ramps for handle materials: oak, birch, spruce, mesquite, bone, antler
  anchors.json                   every head's and blade's anchor (the handle moves by anchor - socket)

<art> is wood, stone, flint, copper, bronze, iron, steel, gold, diamond.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/export2.py
"""
import json
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, HERE)
import heads  # noqa: E402
import parts  # noqa: E402
import toolfix as tf  # noqa: E402

OUT = os.path.join(HERE, "deliver", "textures")
BLOCK = os.path.join(ROOT, "mods", "base", "assets", "textures", "block")
M = ["wood", "stone", "flint", "copper", "bronze", "iron", "steel", "gold", "diamond"]
SHAPE = {"axe": "felling_axe_head", "pickaxe": "pick_head", "shovel": "shovel_blade", "hoe": "hoe_blade"}
TOOLS = {
    "axe": lambda m: "axe_C_felling_1" if m == "iron" else f"felling_{m}_axe",
    "pickaxe": lambda m: f"pick2_{m}_pickaxe",
    "shovel": lambda m: f"fix_{m}_shovel",
    "hoe": lambda m: "hoeL_iron_1" if m == "iron" else f"hoeL_{m}",
}


def save(arr, rel):
    path = os.path.join(OUT, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    Image.fromarray(arr.clip(0, 255).astype(np.uint8)).save(path)
    parts.art.archive(path, rel.replace("/", "__")[:-4], "parts")
    return path


def as_map(a, mask, region):
    """A shade map: R = the 8-step value x 36, G = the region per pixel, A = on."""
    st = parts.steps(a, mask)
    out = np.zeros((32, 32, 4))
    out[..., 0] = np.where(mask, np.clip(st, 0, 7) * 36, 0)
    out[..., 1] = np.where(mask, region, 0)
    out[..., 3] = np.where(mask, 255, 0)
    return out


def main():
    anchors = {}
    for kind, name in TOOLS.items():
        anchors[kind] = {}
        for m in M:
            a = tf.drawn(name(m))
            lab, base = tf.regions(a, "axe")
            sel = (lab == tf.WORK) | (lab == tf.JOINT)
            layer = np.zeros((32, 32, 4))
            layer[sel] = a[sel]
            layer[..., 3] = np.where(sel, 255, 0)
            save(layer, f"parts/{kind}/{SHAPE[kind]}/{m}.png")
            anchors[kind][m] = [round(float(base[0]), 1), round(float(base[1]), 1)]
        # the haft: the plain wooden one (no wrap bands, so any wood ramp reads), moved to meet the iron head
        w = tf.drawn(name("wood"))
        wl, wb = tf.regions(w, "axe")
        hm = wl == tf.HANDLE
        ia = anchors[kind]["iron"]
        dx, dy = int(round(ia[0] - wb[0])), int(round(ia[1] - wb[1]))
        w2, hm2 = heads.shift(w, dy, dx), heads.shift(hm, dy, dx)
        save(as_map(w2, hm2, np.zeros((32, 32))), f"parts/{kind}/haft.png")
    # the sword: blades alone; the hilt (guard, grip, pommel) as one map
    anchors["sword"] = {}
    for m in M:
        a = tf.drawn(f"{m}_sword")
        if a is None:
            continue
        lab, base = tf.sword_regions(a)
        sel = lab == tf.WORK
        layer = np.zeros((32, 32, 4))
        layer[sel] = a[sel]
        layer[..., 3] = np.where(sel, 255, 0)
        save(layer, f"parts/sword/straight_blade/{m}.png")
        anchors["sword"][m] = [round(float(base[0]), 1), round(float(base[1]), 1)]
    s = tf.drawn("iron_sword")
    sl, sb = tf.sword_regions(s)
    hilt = (sl == tf.HANDLE) | (sl == tf.JOINT)
    _, _, (c, d) = heads.cut(s)
    t, dist = heads.frame(s, c, d)
    region = np.zeros((32, 32))
    region[sl == tf.JOINT] = 1                       # the guard
    tmin = t[hilt].min()
    region[(sl == tf.HANDLE) & (t <= tmin + 3)] = 2  # the pommel: the last 3 px at the handle's end
    save(as_map(s, hilt, region), "parts/sword/grip.png")  # named by its region-0 part, the grip
    # ramps for handle materials, by the crate's ids
    r = parts.ramps()

    def block_ramp(n):
        p = os.path.join(BLOCK, n + ".png")
        return parts.ramp_from(np.array(Image.open(p).convert("RGB")).reshape(-1, 3)) if os.path.exists(p) else None
    bone = parts.ramp_from(parts.icon_pixels("bone"))
    ramps = {"oak": r["wood"], "birch": block_ramp("planks_birch"), "spruce": block_ramp("planks_spruce"),
             "mesquite": block_ramp("planks_acacia"),  # no mesquite wood in the game yet: acacia's red-brown stands in
             "bone": bone, "antler": bone * np.array([0.92, 0.86, 0.78])}  # antler: bone, warmer and darker
    for k, v in ramps.items():
        if v is not None:
            save(v.reshape(1, 8, 3), f"ramps/{k}.png")
    json.dump(anchors, open(os.path.join(OUT, "anchors.json"), "w"), indent=1)
    print({k: len(v) for k, v in anchors.items()}, list(ramps))


if __name__ == "__main__":
    main()
