"""Before/after sheets for the art stand-ins batches (11-26). A spec (JSON) lists rows:
  {"label": "...", "before": [paths], "after": [paths], "kind": "item" | "tile" | "gui" | "box",
   "box": [[x0,y0,z0],[x1,y1,z1]], "top": path, "side": path}   (box: drawn in 3D, side/top by position)
Items sit in the game's slot (#352015, bronze rim) at 1x, 2x and 4x; tiles flat at 4x and tiled 3 x 3;
gui squares at 4x; a box block drawn in 3/4 view before and after.

Usage: python3 sheet.py SPEC.json OUT.png"""
import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import chest_split as cs  # noqa: E402  (its little box drawer)

BG, SLOT, RIM, INK = (36, 24, 17, 255), (53, 32, 21, 255), (134, 94, 62, 255), (232, 214, 176, 255)
try:
    FONT = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 18)
    SMALL = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
except OSError:
    FONT = SMALL = ImageFont.load_default()


def rgba(p):
    return Image.open(p).convert("RGBA")


def slot(img, k):
    s = 32 * k + 8 * max(1, k // 2)
    c = Image.new("RGBA", (s, s), RIM)
    ImageDraw.Draw(c).rectangle([2, 2, s - 3, s - 3], fill=SLOT)
    t = img.resize((img.width * k, img.height * k), Image.NEAREST)
    c.alpha_composite(t, ((s - t.width) // 2, (s - t.height) // 2))
    return c


def item_cell(p):
    im = rgba(p)
    if im.size != (32, 32):
        im = im.resize((32, 32), Image.NEAREST)
    parts = [slot(im, 1), slot(im, 2), slot(im, 4)]
    w = sum(x.width for x in parts) + 8 * len(parts)
    c = Image.new("RGBA", (w, max(x.height for x in parts)), BG)
    x = 0
    for q in parts:
        c.alpha_composite(q, (x, c.height - q.height))
        x += q.width + 8
    return c


def tile_cell(p):
    im = rgba(p)
    big = im.resize((128, 128), Image.NEAREST)
    tiled = Image.new("RGBA", (96, 96))
    for i in range(3):
        for j in range(3):
            tiled.alpha_composite(im, (i * 32, j * 32))
    tiled = tiled.resize((192, 192), Image.NEAREST)
    c = Image.new("RGBA", (128 + 8 + 192, 192), BG)
    c.alpha_composite(big, (0, 64))
    c.alpha_composite(tiled, (136, 0))
    return c


def gui_cell(paths):
    ims = [rgba(p) for p in paths]
    ims = [i.resize((i.width * 3, i.height * 3), Image.NEAREST) for i in ims]  # 54 px a square
    w = sum(i.width + 6 for i in ims)
    c = Image.new("RGBA", (max(w, 10), max(i.height for i in ims)), BG)
    x = 0
    for i in ims:
        c.alpha_composite(i, (x, 0))
        x += i.width + 6
    return c


def box_cell(side, top, lo, hi, front=None):
    side, top = np.asarray(rgba(side)), np.asarray(rgba(top))
    front = side if front is None else np.asarray(rgba(front))
    (x0, y0, z0), (x1, y1, z1) = lo, hi

    def px(img, a0, b0, a1, b1):
        return img[32 - 2 * b1:32 - 2 * b0, 2 * a0:2 * a1]
    f = cs.box_faces(lo, hi, {"up": top[2 * z0:2 * z1, 2 * x0:2 * x1], "north": px(front, x0, y0, x1, y1),
                              "west": px(side, z0, y0, z1, y1), "east": px(side, z0, y0, z1, y1), "south": px(side, x0, y0, x1, y1)})
    return cs.draw(f, scale=10, size=(300, 280))


def row(r):
    k = r.get("kind", "item")
    def cells(paths):
        if k == "item":
            return [item_cell(p) for p in paths]
        if k == "tile":
            return [tile_cell(p) for p in paths]
        if k == "gui":
            return [gui_cell(paths)] if paths else []
        if k == "image":
            out = []
            for p in paths:
                im = rgba(p)
                if im.width > 1000:
                    im = im.resize((1000, round(im.height * 1000 / im.width)), Image.NEAREST)
                out.append(im)
            return out
        if k == "box":
            return [box_cell(*paths, r["box"][0], r["box"][1])] if paths else []
    return cells(r.get("before", [])), cells(r.get("after", []))


def main():
    spec = json.load(open(sys.argv[1]))
    rows = [(r, *row(r)) for r in spec["rows"]]
    W = 1960
    pad = 14
    heights = [max([c.height for c in b + a] + [40]) + 2 * pad + 22 for _, b, a in rows]
    sheet = Image.new("RGBA", (W, 70 + sum(heights)), BG)
    d = ImageDraw.Draw(sheet)
    d.text((pad, 12), spec.get("title", ""), font=FONT, fill=INK)
    d.text((300, 44), "before (in the game now)", font=SMALL, fill=INK)
    d.text((900, 44), "after", font=SMALL, fill=INK)
    y = 70
    for (r, b, a), h in zip(rows, heights):
        d.line([(0, y), (W, y)], fill=RIM, width=1)
        d.text((pad, y + pad), r["label"], font=SMALL, fill=INK)
        if r.get("note"):
            d.text((pad, y + pad + 20), r["note"], font=SMALL, fill=(180, 156, 118, 255))
        x = 300
        for c in b:
            sheet.alpha_composite(c, (x, y + pad + 20))
            x += c.width + 12
        x = max(x + 20, 900)
        for c in a:
            if x + c.width > W:
                break
            sheet.alpha_composite(c, (x, y + pad + 20))
            x += c.width + 12
        y += h
    sheet.save(sys.argv[2])
    print(sys.argv[2])


if __name__ == "__main__":
    main()
