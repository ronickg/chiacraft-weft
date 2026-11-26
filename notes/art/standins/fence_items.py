"""Fence item models for the 14 woods (art stand-ins, 11-26). A lone fence block is only its post,
so its slot icon (the engine draws a block item's model) was a plank stick. Each `<wood>_fence`
gets `models/item/<wood>_fence.json`: two posts joined by two rails, in its own planks, the
block's own sizes (posts 4 x 16, rails 2 x 3 at 6-9 and 12-15). Gates show their shut state already.

Usage: python3 fence_items.py [OUT_ASSETS] [--preview out.png]"""
import json
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
WOODS = ["oak", "birch", "spruce", "acacia", "mesquite", "acuri_palm", "moriche_palm", "brazil_nut", "cacao", "andiroba",
         "matamata", "ipe", "cherry", "sequoia"]
FENCE = [((2, 0, 6), (6, 16, 10)), ((10, 0, 6), (14, 16, 10)), ((0, 6, 7), (16, 9, 9)), ((0, 12, 7), (16, 15, 9))]
GATE = [((0, 5, 7), (2, 16, 9)), ((14, 5, 7), (16, 16, 9)), ((2, 6, 7), (14, 9, 9)), ((2, 12, 7), (14, 15, 9)), ((7, 9, 7), (9, 12, 9))]


def model(wood):
    t = f"base:block/planks_{wood}"
    return {"textures": {"planks": t, "particle": t},
            "elements": [{"from": list(lo), "to": list(hi), "faces": {f: {"texture": "#planks"} for f in
                                                                     ("down", "up", "north", "south", "west", "east")}}
                         for lo, hi in FENCE]}


def preview(path):
    import chest_split as cs
    cells = []
    for wood in WOODS:
        tex = np.asarray(Image.open(os.path.join(REPO, f"mods/base/assets/textures/block/planks_{wood}.png")).convert("RGBA"))
        row = []
        for boxes in (FENCE, GATE):
            f = []
            for (x0, y0, z0), (x1, y1, z1) in boxes:
                px = lambda a0, b0, a1, b1: tex[32 - 2 * b1:32 - 2 * b0, 2 * a0:2 * a1]
                f += cs.box_faces((x0, y0, z0), (x1, y1, z1), {"north": px(x0, y0, x1, y1), "west": px(z0, y0, z1, y1),
                                                                "east": px(z0, y0, z1, y1), "south": px(x0, y0, x1, y1),
                                                                "up": tex[2 * z0:2 * z1, 2 * x0:2 * x1]})
            row.append(cs.draw(f, scale=6, size=(180, 170)))
        cells.append((wood, row))
    from PIL import ImageDraw
    W = 7
    sheet = Image.new("RGBA", (W * 370, 2 * 200), (36, 24, 17, 255))
    d = ImageDraw.Draw(sheet)
    for i, (wood, (a, b)) in enumerate(cells):
        x, y = (i % W) * 370, (i // W) * 200
        sheet.alpha_composite(a, (x, y))
        sheet.alpha_composite(b, (x + 180, y))
        d.text((x + 10, y + 178), wood, fill=(232, 214, 176, 255))
    sheet.save(path)


def main():
    out = sys.argv[1] if len(sys.argv) > 1 and not sys.argv[1].startswith("--") else os.path.join(REPO, "mods/base/assets")
    os.makedirs(os.path.join(out, "models/item"), exist_ok=True)
    for wood in WOODS:
        json.dump(model(wood), open(os.path.join(out, "models/item", f"{wood}_fence.json"), "w"), indent=2)
    if "--preview" in sys.argv:
        preview(sys.argv[sys.argv.index("--preview") + 1])


if __name__ == "__main__":
    main()
