"""The star lantern as a model (Planner, 11-05: "a proper lantern model for the Starheart"; it was a full cube):
a dark iron lantern, a hand tall, its glowing Starheart crystal standing inside see-through panes. Boxes in model
units (16 a block, 2 texels a unit at our 32 a block), painted into one picture: the block's own texture
(star_lantern.png), which every face of models/block/star_lantern.json shows (it names none), and its glow mask
(star_lantern_e.png: the crystal, and faintly the glint on the panes).

    python3 mods/base/source/star_lantern/build.py

The block must not be a full opaque cube (`.not_solid()` and not opaque), or the faces next to it are hidden
behind the lantern's gaps. model.py and render.py are copies of mods/base/source/plants3d/'s.
"""
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from model import Box, build, ramp, along, quads  # noqa: E402

ROOT = os.path.normpath(os.path.join(HERE, "..", ".."))
IRON = ["#1c1e24", "#2a2d35", "#3a3e48", "#525864", "#6c7380", "#8a929e"]
CRYSTAL = ["#4f7ed8", "#7eaaf4", "#aecdff", "#e2eeff"]


def iron(b, face, u, v, w, h):
    t = along(b, face, v, h)
    s = {"up": 0.85, "down": 0.1, "north": 0.45, "south": 0.55, "east": 0.35, "west": 0.6}[face]
    s += 0.2 * (t - 0.5)
    if face not in ("up", "down") and (u == 0 or u == w - 1):
        s -= 0.15  # the box's own edges a touch darker
    if face == "up" and (u in (0, w - 1) or v in (0, h - 1)):
        s += 0.1   # a lit rim on top
    return (*ramp(IRON, s), 255)


def crystal(b, face, u, v, w, h):
    t = along(b, face, v, h)
    mid = 1 - abs((u + 0.5) / w - 0.5) * 2
    s = 0.45 + 0.35 * mid + 0.2 * t
    if face == "up":
        s = 0.95
    return (*ramp(CRYSTAL, s), 255)


def glass(b, face, u, v, w, h):
    # see-through, save one diagonal glint across each pane
    if (u + v) == w - 2 and 1 <= v <= 3:
        return (190, 218, 255, 255)
    return (0, 0, 0, 0)


def boxes(e=False):
    """e: the glow mask instead of the colours (the same boxes, so the same layout)."""
    dark = (lambda *a: (0, 0, 0, 0))
    I = dark if e else iron
    C = (lambda b, f, u, v, w, h: (int(255 * (0.6 + 0.4 * along(b, f, v, h))),) * 3 + (255,)) if e else crystal
    G = (lambda b, f, u, v, w, h: (100, 100, 100, 255) if glass(b, f, u, v, w, h)[3] else (0, 0, 0, 0)) if e else glass
    sides = ["north", "south", "east", "west"]
    bs = [Box([4.5, 0, 4.5], [11.5, 1, 11.5], I)]                                     # the base plate
    for x, z in ((4.5, 4.5), (10.5, 4.5), (4.5, 10.5), (10.5, 10.5)):                 # four corner posts
        bs.append(Box([x, 1, z], [x + 1, 7, z + 1], I, faces=sides))
    bs.append(Box([5, 1, 5], [11, 7, 11], G, faces=sides))                             # the panes
    bs.append(Box([6.5, 1.5, 6.5], [9.5, 6.5, 9.5], C, rot=([8, 4, 8], "y", 45), faces=sides + ["up"]))  # the Starheart
    bs.append(Box([4.25, 7, 4.25], [11.75, 8, 11.75], I))                              # the cap
    bs.append(Box([5.5, 8, 5.5], [10.5, 9, 10.5], I, faces=sides + ["up"]))            # the roof
    bs.append(Box([7.5, 9, 7.5], [8.5, 10, 8.5], I, faces=sides))                      # the handle's neck
    bs.append(Box([6.25, 10, 7.5], [9.75, 11, 8.5], I))                                # and its bar
    return bs


def main():
    m, img = build(boxes())
    _, eimg = build(boxes(e=True))
    a = np.asarray(eimg).copy()
    a[..., 3] = np.where(a[..., 0] > 0, 255, 0)
    from PIL import Image
    img.save(os.path.join(ROOT, "assets", "textures", "block", "star_lantern.png"))
    Image.fromarray(a, "RGBA").save(os.path.join(ROOT, "assets", "textures", "block", "star_lantern_e.png"))
    import json
    with open(os.path.join(ROOT, "assets", "models", "block", "star_lantern.json"), "w") as f:
        json.dump(m, f, separators=(",", ":"))
    print(f"star_lantern: {len(m['elements'])} boxes, {quads(m)} quads, picture {img.size[0]} px")


if __name__ == "__main__":
    main()
