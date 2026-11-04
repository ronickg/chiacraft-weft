"""Each wild flower's out-of-bloom look (`<name>_green.png`), from its picture: the petals turned
the leaf green of the same sprite, shaded as they were. Run it again whenever a flower's picture
changes (`vitals::wild::FLOWERS` names them):

    python3 mods/base/source/flowers/greens.py
"""
import re
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[4]
BLOCKS = ROOT / "mods/base/assets/textures/block"
TABLE = ROOT / "mods/base/vitals/src/wild.rs"


def green(name):
    im = Image.open(BLOCKS / f"{name}.png").convert("RGBA")
    px = im.load()
    w, h = im.size
    leaf = [px[x, y][:3] for y in range(h) for x in range(w) if px[x, y][3] and px[x, y][1] >= max(px[x, y][0], px[x, y][2])]
    tone = tuple(sum(c[i] for c in leaf) // max(1, len(leaf)) for i in range(3)) if leaf else (70, 120, 50)
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            if a == 0 or g >= max(r, b):
                continue
            lum = (0.3 * r + 0.59 * g + 0.11 * b) / 160.0
            px[x, y] = (min(255, int(tone[0] * lum)), min(255, int(tone[1] * lum)), min(255, int(tone[2] * lum)), a)
    im.save(BLOCKS / f"{name}_green.png")


names = re.findall(r'f\("base:(\w+)"', TABLE.read_text())
for n in names:
    green(n)
print(f"{len(names)} green looks")
