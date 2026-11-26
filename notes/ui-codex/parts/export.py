"""Every finished tool's layers in the crafting session's format (notes/CRAFTING.md, "The art format";
mods/base/crafting/src/looks.rs), done the way Ronald settled: each material's working part (with its
joint band) painted where its drawing has it, plus its anchor; the handle layers (wrapped and plain)
painted with their sockets; the game moves the handle to meet the head.

  deliver/parts/<kind>/<layer>/<material>.png   painted override, 32 x 32 RGBA
  deliver/parts/<kind>/<layer>/<material>.json  {"kind","layer","material","regions","anchor"}
  deliver/parts/<kind>/handle/{wrapped,plain}.png/.json  ({"socket": where it meets a head})

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/export.py
"""
import json
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import parts  # noqa: E402
import toolfix as tf  # noqa: E402

OUT = os.path.join(HERE, "deliver", "parts")
IDS = {"wood": "oak", "iron": "wrought_iron"}  # the crafting crate's material ids
M = ["wood", "stone", "flint", "copper", "bronze", "iron", "steel", "gold", "diamond"]
KINDS = {
    "axe": (lambda m: "axe_C_felling_1" if m == "iron" else f"felling_{m}_axe", "head", ["head", "fitting"], "haft"),
    "pickaxe": (lambda m: f"pick2_{m}_pickaxe", "head", ["head", "fitting"], "haft"),
    "shovel": (lambda m: f"fix_{m}_shovel", "head", ["head", "fitting"], "haft"),
    "hoe": (lambda m: "hoeL_iron_1" if m == "iron" else f"hoeL_{m}", "head", ["head", "fitting"], "haft"),
    "sword": (lambda m: f"{m}_sword", "blade", ["blade", "guard"], "hilt"),
}
CRUDE = {"wood", "stone", "flint"}


def save(arr, path, meta):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    Image.fromarray(arr.astype(np.uint8)).save(path)
    json.dump(meta, open(path[:-4] + ".json", "w"), indent=1)
    parts.art.archive(path, os.path.relpath(path, OUT).replace("/", "__")[:-4], "parts")


def main():
    summary = {}
    for kind, (name, work, regions, handle) in KINDS.items():
        ck = "sword" if kind == "sword" else "axe"
        done = []
        for m in M:
            a = tf.drawn(name(m))
            if a is None:
                continue
            lab, base = tf.regions(a, ck)
            sel = (lab == tf.WORK) | (lab == tf.JOINT)
            layer = np.zeros((32, 32, 4))
            layer[sel] = a[sel]
            layer[..., 3] = np.where(sel, 255, 0)
            mid = IDS.get(m, m)
            save(layer, os.path.join(OUT, kind, work, mid + ".png"),
                 {"kind": f"base:{kind}", "layer": work, "material": mid, "override": True, "regions": regions,
                  "joint_region_pixels": int((lab == tf.JOINT).sum()),
                  "anchor": [round(float(base[0]), 1), round(float(base[1]), 1)]})
            done.append(mid)
        for style, src in (("wrapped", name("iron")), ("plain", name("wood"))):
            a = tf.drawn(src)
            if a is None:
                continue
            lab, base = tf.regions(a, ck)
            sel = lab == tf.HANDLE
            layer = np.zeros((32, 32, 4))
            layer[sel] = a[sel]
            layer[..., 3] = np.where(sel, 255, 0)
            save(layer, os.path.join(OUT, kind, handle, style + ".png"),
                 {"kind": f"base:{kind}", "layer": handle, "style": style, "override": True,
                  "socket": [round(float(base[0]), 1), round(float(base[1]), 1)],
                  "note": "moved so this socket meets the head's anchor"})
        summary[kind] = done
    json.dump(summary, open(os.path.join(OUT, "index.json"), "w"), indent=1)
    print(json.dumps(summary))


if __name__ == "__main__":
    main()
