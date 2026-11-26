"""A less saturated copper pick (art stand-ins, 11-26): the drawn copper pick head (the tool's and
the loose / cast head's) with its copper pixels' saturation scaled down and a touch darker, the
binding and the haft's colours left alone. Variants for the review page; `--install K` writes one
over the game's two copper pick-head pictures (the old ones archived first).

Usage: python3 copper_pick.py OUT_DIR [--install 0.8]"""
import colorsys
import datetime
import os
import shutil
import sys

import numpy as np
from PIL import Image

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
PARTS = os.path.join(REPO, "mods/base/assets/textures/parts")
FILES = ["pickaxe/pick_head/copper.png", "loose/pick_head/copper.png"]
ARCHIVE = os.path.expanduser("~/Coding/chiacraft-archive/images/art-standins")
LEVELS = [1.0, 0.8, 0.68, 0.56]


def tone(im, k, dark=0.04):
    a = np.asarray(im.convert("RGBA")).astype(float) / 255
    out = a.copy()
    for y in range(a.shape[0]):
        for x in range(a.shape[1]):
            r, g, b, al = a[y, x]
            if al == 0:
                continue
            h, s, v = colorsys.rgb_to_hsv(r, g, b)
            # Copper's orange (hue 5-33 degrees, saturated); the cream and gold binding sits above it.
            if 5 / 360 <= h <= 33 / 360 and s > 0.35:
                s2, v2 = s * k, v * (1 - dark * (1 - k) / 0.2)
                out[y, x, :3] = colorsys.hsv_to_rgb(h, s2, min(1, v2))
    return Image.fromarray((out * 255).round().astype(np.uint8))


def main():
    out = sys.argv[1]
    os.makedirs(out, exist_ok=True)
    for f in FILES:
        src = Image.open(os.path.join(PARTS, f))
        for k in LEVELS:
            tone(src, k).save(os.path.join(out, f"{f.split('/')[0]}_s{int(k * 100)}.png"))
    if "--install" in sys.argv:
        k = float(sys.argv[sys.argv.index("--install") + 1])
        day = os.path.join(ARCHIVE, datetime.date.today().isoformat(), "replaced")
        os.makedirs(day, exist_ok=True)
        for f in FILES:
            p = os.path.join(PARTS, f)
            shutil.copy2(p, os.path.join(day, f.replace("/", "__")))
            tone(Image.open(p), k).save(p)
            print("wrote", p)


if __name__ == "__main__":
    main()
