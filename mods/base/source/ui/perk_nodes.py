"""The perk web's nodes from one generated sheet (game-art UI flow, `notes/ui-codex/gen.sh
perk_web_nodes`): rows small (16 px), notable (24), keystone (32), the start (24); each row's states
taken, open, closed ("far"), locked, then the same four hovered. Writes `perk_<kind>_<state>` with
`_hover`, `_pressed` (= normal) and `_disabled` (= normal), and a `.gui` each (stretch, at half
size in GUI pixels, as the screen draws them: `mods/base/src/web.rs`).

    ~/.venvs/blocky/bin/python mods/base/source/ui/perk_nodes.py notes/ui-codex/perk_web_nodes.png mods/base/assets/textures/gui
    ~/.venvs/blocky/bin/python mods/base/source/ui/perk_nodes.py notes/ui-codex/perk_web_sockets.png mods/base/assets/textures/gui
"""
import os
import sys

import numpy as np
from PIL import Image

import extract as ex

KINDS = [("small", 16), ("notable", 24), ("keystone", 32), ("start", 24)]
STATES = ["taken", "open", "far", "locked"]


def runs(mask):
    out, start = [], None
    for i, on in enumerate(list(mask) + [False]):
        if on and start is None:
            start = i
        elif not on and start is not None:
            out.append((start, i))
            start = None
    return out


# Ronald, 11-05: "Generated ones", cut down to the kit's palette (the UI guide's pixel craft):
# every pixel to the nearest of kit2's colours (`notes/ui/kit2/kit2.py`), compared in Lab so the
# bevel's light and shade keep their steps.
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..", "notes", "ui", "kit2"))
import kit2 as k  # noqa: E402

PALETTE = [k.OUTLINE, k.INK_DARK, k.PANEL, k.PANEL_DEEP, k.WELL, k.WELL_D, k.WELL_L, k.SLIDER, k.SLIDER_D, k.SLIDER_L,
           k.BRONZE_D, k.BRONZE, k.BRONZE_L, k.BRONZE_HI, k.SLOT, k.SLOT_D, k.SLOT_L, k.CREAM, k.DIM, k.GOLD_TEXT, k.ON_ORANGE,
           k.GLOW, k.GLOW_D] + [c for name, face in k.FACES.items() if name != "danger" for c in face] + [c for m in ("cream", "brass", "wood", "blue", "dark") for c in k.MAT[m]]
# (no "danger" red: the bevel's orange shade fell onto it and showed red on the rims)


def lab(rgb):
    c = np.asarray(rgb, float) / 255.0
    c = np.where(c > 0.04045, ((c + 0.055) / 1.055) ** 2.4, c / 12.92)
    xyz = c @ np.array([[0.4124, 0.2126, 0.0193], [0.3576, 0.7152, 0.1192], [0.1805, 0.0722, 0.9505]])
    xyz = xyz / np.array([0.9505, 1.0, 1.089])
    f = np.where(xyz > 0.008856, np.cbrt(xyz), 7.787 * xyz + 16 / 116)
    return np.stack([116 * f[..., 1] - 16, 500 * (f[..., 0] - f[..., 1]), 200 * (f[..., 1] - f[..., 2])], -1)


PAL_LAB = lab(np.array(sorted(set(PALETTE)), float))
PAL_RGB = np.array(sorted(set(PALETTE)), np.uint8)


def quantise(im):
    a = np.array(im)
    m = a[..., 3] > 0
    if m.any():
        d = ((lab(a[m][:, :3])[:, None, :] - PAL_LAB[None, :, :]) ** 2).sum(-1)
        a[m, :3] = PAL_RGB[d.argmin(1)]
        a[m, 3] = 255
    return Image.fromarray(a)


def cut(src, fg, box, n):
    """The box's art at n x n: each output pixel the colour at its cell's middle, magenta clear."""
    y0, y1, x0, x1 = box
    out = np.zeros((n, n, 4), np.uint8)
    for j in range(n):
        for i in range(n):
            y = int(y0 + (j + 0.5) * (y1 - y0) / n)
            x = int(x0 + (i + 0.5) * (x1 - x0) / n)
            r, g, b, _ = (int(v) for v in src[y, x])
            # Magenta's anti-aliased fringe (pink, purple) is background too.
            if fg[y, x] and not (r > 150 and b > 120 and g < r - 60) and not (r > g + 25 and b > g + 25):
                out[j, i] = src[y, x]
    return Image.fromarray(out)


def write(out, name, normal, hover, n):
    for suffix, im in [("", normal), ("_pressed", normal), ("_disabled", normal), ("_hover", hover)]:
        quantise(im).save(f"{out}/{name}{suffix}.png")
        with open(f"{out}/{name}{suffix}.gui", "w") as f:
            f.write(f"scaling = stretch\nsize = {n // 2} {n // 2}\n")


def sockets(img, out):
    """The socket sheet (`notes/ui-codex/perk_web_sockets.png`): row 1 the four states then their
    hovers, row 2 a taken socket holding a mind-stone, a flow-stone and a relic, then their hovers."""
    src = np.array(Image.open(img).convert("RGBA"))
    fg = ~ex.background(src, tight=True)
    bands = [b for b in runs(fg.sum(1) > 3) if b[1] - b[0] > 20]
    assert len(bands) == 2, f"{len(bands)} rows of sockets"
    n = 24
    for (y0, y1), names in zip(bands, [[f"perk_socket_{s}" for s in STATES], ["perk_socket_mind", "perk_socket_flow", "perk_socket_relic"]]):
        cols = [c for c in runs(fg[y0:y1].sum(0) > 2) if c[1] - c[0] > 20]
        boxes = []
        for x0, x1 in cols:
            rows = np.nonzero(fg[y0:y1, x0:x1].any(1))[0]
            boxes.append((y0 + rows[0], y0 + rows[-1] + 1, x0, x1))
        assert len(boxes) == 2 * len(names), f"{len(boxes)} parts for {names}"
        for k, name in enumerate(names):
            hb = boxes[k + len(names)]
            grow = ((hb[1] - hb[0]) - (boxes[k][1] - boxes[k][0])) / 2
            hover = cut(src, fg, (int(hb[0] + grow), int(hb[1] - grow), int(hb[2] + grow), int(hb[3] - grow)), n)
            write(out, name, cut(src, fg, boxes[k], n), hover, n)
    print("sockets done")


def main():
    img, out = sys.argv[1], sys.argv[2]
    if "sockets" in img:
        return sockets(img, out)
    src = np.array(Image.open(img).convert("RGBA"))
    fg = ~ex.background(src, tight=True)
    bands = [b for b in runs(fg.sum(1) > 3) if b[1] - b[0] > 20]
    assert len(bands) == len(KINDS), f"{len(bands)} rows of nodes, want {len(KINDS)}"
    for (kind, n), (y0, y1) in zip(KINDS, bands):
        cols = [c for c in runs(fg[y0:y1].sum(0) > 2) if c[1] - c[0] > 20]
        boxes = []
        for x0, x1 in cols:
            rows = np.nonzero(fg[y0:y1, x0:x1].any(1))[0]
            boxes.append((y0 + rows[0], y0 + rows[-1] + 1, x0, x1))
        states = ["taken"] if kind == "start" else STATES
        assert len(boxes) >= len(states), f"{kind}: {len(boxes)} parts"
        # A hover is drawn on the normal one's grid: its glow ring a pixel outside, cut off.
        for k, state in enumerate(states):
            normal = cut(src, fg, boxes[k], n)
            hover = normal
            if len(boxes) >= 2 * len(states):
                hb = boxes[k + len(states)]
                grow = ((hb[1] - hb[0]) - (boxes[k][1] - boxes[k][0])) / 2
                hover = cut(src, fg, (int(hb[0] + grow), int(hb[1] - grow), int(hb[2] + grow), int(hb[3] - grow)), n)
            for suffix, im in [("", normal), ("_pressed", normal), ("_disabled", normal), ("_hover", hover)]:
                name = f"perk_{kind}_{state}{suffix}"
                quantise(im).save(f"{out}/{name}.png")
                with open(f"{out}/{name}.gui", "w") as f:
                    f.write(f"scaling = stretch\nsize = {n // 2} {n // 2}\n")
    print("done")


if __name__ == "__main__":
    main()
