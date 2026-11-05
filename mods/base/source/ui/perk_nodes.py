"""The perk web's nodes from one generated sheet (game-art UI flow, `notes/ui-codex/gen.sh
perk_web_nodes`): rows small (16 px), notable (24), keystone (32), the start (24); each row's states
taken, open, closed ("far"), locked, then the same four hovered. Writes `perk_<kind>_<state>` with
`_hover`, `_pressed` (= normal) and `_disabled` (= normal), and a `.gui` each (stretch, at half
size in GUI pixels, as the screen draws them: `mods/base/src/web.rs`).

    ~/.venvs/blocky/bin/python mods/base/source/ui/perk_nodes.py notes/ui-codex/perk_web_nodes.png mods/base/assets/textures/gui
    ~/.venvs/blocky/bin/python mods/base/source/ui/perk_nodes.py notes/ui-codex/perk_web_sockets.png mods/base/assets/textures/gui
"""
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
        im.save(f"{out}/{name}{suffix}.png")
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
                im.save(f"{out}/{name}.png")
                with open(f"{out}/{name}.gui", "w") as f:
                    f.write(f"scaling = stretch\nsize = {n // 2} {n // 2}\n")
    print("done")


if __name__ == "__main__":
    main()
