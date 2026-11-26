"""Ghost icons for the parts crafting row (UI's request, 11-05), in the armour slots' look: the armour
ghosts (gui/slot_helmet.png) are chunky full-colour icons mapped by brightness onto one faded brown at
alpha 140 (mods/base/source/ui/icons.py --silhouette). The same here, from bold generated part icons
(mods/base/source/ui/kit/parts/: a stone axe head, a haft, a cord binding, the whole hafted axe), each
fitted to 38 px of the 48 like the helmet's. The sources keep their own dark outline.

    python3 notes/blocks/part_ghosts.py
"""
import numpy as np
from PIL import Image

SRC = "mods/base/source/ui/kit/parts/"
G = "mods/base/assets/textures/gui/"
DARK, LIGHT = np.array([46, 32, 24.0]), np.array([150, 116, 88.0])
ALPHA, SIZE, BOX = 140, 48, 38


def ghost(path):
    a = np.asarray(Image.open(path).convert("RGB")).astype(float)
    r, g, b = a[..., 0], a[..., 1], a[..., 2]
    keep = ~((r > 180) & (b > 180) & (g < 110))  # the magenta ground out
    ys, xs = np.where(keep)
    y0, y1, x0, x1 = ys.min(), ys.max() + 1, xs.min(), xs.max() + 1
    rgba = np.dstack([a, keep * 255.0])[y0:y1, x0:x1].astype(np.uint8)
    s = BOX / max(y1 - y0, x1 - x0)
    w, h = max(1, round((x1 - x0) * s)), max(1, round((y1 - y0) * s))
    im = np.array(Image.fromarray(rgba, "RGBA").resize((w, h), Image.LANCZOS)).astype(float)
    m = im[..., 3] > 127
    lum = im[..., :3].mean(-1, keepdims=True) / 255
    rgb = DARK + (LIGHT - DARK) * lum
    out = np.zeros((SIZE, SIZE, 4))
    ox, oy = (SIZE - w) // 2, (SIZE - h) // 2
    out[oy:oy + h, ox:ox + w, :3] = rgb
    out[oy:oy + h, ox:ox + w, 3] = np.where(m, ALPHA, 0)
    return out.clip(0, 255).astype(np.uint8)


def main():
    for n in ("head", "haft", "fitting", "tool"):
        Image.fromarray(ghost(SRC + n + ".png")).save(G + f"slot_part_{n}.png")
        print(G + f"slot_part_{n}.png")


if __name__ == "__main__":
    main()
