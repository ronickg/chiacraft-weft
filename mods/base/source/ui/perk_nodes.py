"""The perk web's nodes, drawn in code on the UI kit's pixel grid and palette (`notes/ui/kit2/kit2.py`;
the UI guide's pixel craft rules: one grid, flat faces, a 1 px outline, a 1 px light top edge and a
2 px shade, a gold outline on hover, pressed sunk, one footprint for every state). Each node is
drawn at the size it's shown in UI px (16 small, 24 notable and the start, 32 keystone), and the
screen (`mods/base/src/web.rs`) shows it at 1x or 2x, never a fraction.

States: taken (the orange accent: done), open (lit wood: can be taken now), far (dark wood: not
next to one taken, or no point), locked (grey with a padlock: the skill is too low).

    ~/.venvs/blocky/bin/python mods/base/source/ui/perk_nodes.py mods/base/assets/textures/gui [sheet.png]
"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..", "notes", "ui", "kit2"))
import kit2 as k  # noqa: E402

KINDS = [("small", 16), ("notable", 24), ("keystone", 32), ("start", 24), ("socket", 24)]
# A taken socket holding a stone: the stone's colour in the hole (legend: cream a mind-stone,
# blue a flow-stone, the ley's colour; gold a relic).
FILLED = {"mind": "cream", "flow": "blue", "relic": "brass"}
STATES = ["taken", "open", "far", "locked"]

# The ring per state: (light edge, face, shade).
RING = {
    "taken": ((255, 206, 128), k.GLOW, k.GLOW_D),
    "open": (k.BRONZE_HI, k.BRONZE_L, k.BRONZE_D),
    "far": (k.BRONZE, k.BRONZE_D, (52, 34, 20)),
    "locked": k.FACES["disabled"],
}
# The ring on hover: one step brighter (the outline turns gold).
RING_HOVER = {
    "taken": ((255, 222, 160), (255, 178, 84), k.GLOW),
    "open": ((182, 136, 96), k.BRONZE_HI, k.BRONZE),
    "far": (k.BRONZE_L, k.BRONZE, k.BRONZE_D),
    "locked": ((70, 64, 60), (58, 53, 50), (44, 40, 37)),
}
# The centre stone of notables, keystones and the start: (light, base, dark).
STONE = {
    "taken": k.MAT["cream"],
    "open": ((96, 71, 51), (74, 53, 36), (56, 39, 26)),
    "far": (k.WELL_L, k.WELL, k.WELL_D),
    "locked": ((52, 48, 45), (42, 39, 36), (34, 31, 29)),
}
PADLOCK = [
    ".###.",
    "#...#",
    "#####",
    "##.##",
    "#####",
]


def disc(n):
    """A round mask n across, its edge stepped on the grid (kit2's pill profile, both ways)."""
    return k.shape(n, n, "pill") & k.shape(n, n, "pill").T


def octagon(n):
    """An octagon n across: corners cut at 45 degrees, a third of a side each."""
    m = np.ones((n, n), bool)
    cut = n // 3 - 1
    for i in range(cut):
        c = cut - i
        m[i, :c] = m[i, n - c:] = m[n - 1 - i, :c] = m[n - 1 - i, n - c:] = False
    return m


def stone(a, m, mat):
    """A cut stone in mask m: light on its top-left half, dark on its bottom-right, base between."""
    light, base, dark = mat
    k.paint(a, m, base)
    ys, xs = np.nonzero(m)
    cy, cx = ys.mean(), xs.mean()
    for y, x in zip(ys, xs):
        d = (x - cx) + (y - cy)
        if d < -1.5:
            a[y, x, :3] = light
        elif d > 1.5:
            a[y, x, :3] = dark


def draw(kind, n, state, hover=False, pressed=False, filled=None):
    a = np.zeros((n, n, 4), np.uint8)
    m = octagon(n) if kind == "keystone" else disc(n)
    light, face, shade = (RING_HOVER if hover else RING)[state]
    rim = k.GLOW if hover and state in ("taken", "open") else k.OUTLINE
    k.raised(a, m, light, face, shade, rim=rim, pressed=pressed)
    ring = {"small": 3, "notable": 4, "keystone": 4, "start": 4, "socket": 4}[kind]
    inner = k.shrink(m, ring)
    # The well inside the ring, sunk: its outline dark, its top row in shade.
    k.paint(a, inner, k.WELL_D)
    k.paint(a, k.shrink(inner, 1), k.WELL)
    dy = 1 if pressed else 0
    centre = np.roll(k.shrink(inner, 1), dy, 0) & inner
    if kind == "keystone":
        # A second ring inside the first.
        k.raised(a, inner, light, face, shade, rim=k.OUTLINE)
        centre = np.roll(k.shrink(inner, 3), dy, 0)
    if kind == "socket":
        # An empty socket is a deep hole (darker than a node's well) with a hint of its rim; a
        # filled one shows its stone.
        k.paint(a, centre, k.PANEL_DEEP)
        if filled:
            stone(a, k.shrink(centre, 1), k.MAT[FILLED[filled]])
    elif kind == "start":
        stone(a, centre, k.MAT["brass"])
    elif kind in ("notable", "keystone"):
        stone(a, centre, STONE[state])
    elif state == "taken":
        stone(a, centre, k.MAT["cream"])
    elif state == "open":
        c = n // 2 - 1 + dy
        a[c:c + 2, n // 2 - 1:n // 2 + 1] = (*k.CREAM, 255)
    if state == "locked" and kind != "start":
        h, w = len(PADLOCK), len(PADLOCK[0])
        y0, x0 = (n - h) // 2 + dy, (n - w) // 2
        for j, row in enumerate(PADLOCK):
            for i, ch in enumerate(row):
                if ch == "#":
                    a[y0 + j, x0 + i] = (*k.DIM, 255)
    return Image.fromarray(a)


def link(color):
    a = np.zeros((2, 2, 4), np.uint8)
    a[...] = (*color, 255)
    return Image.fromarray(a)


def main():
    out = sys.argv[1]
    made = {}
    for kind, n in KINDS:
        for state in ["taken"] if kind == "start" else STATES:
            for suffix, kw in [("", {}), ("_hover", {"hover": True}), ("_pressed", {"pressed": True}), ("_disabled", {})]:
                name = f"perk_{kind}_{state}{suffix}"
                im = draw(kind, n, state, **kw)
                made[name] = im
                im.save(f"{out}/{name}.png")
                with open(f"{out}/{name}.gui", "w") as f:
                    f.write(f"scaling = stretch\nsize = {n // 2} {n // 2}\n")
    for fill in FILLED:
        for suffix, kw in [("", {}), ("_hover", {"hover": True}), ("_pressed", {"pressed": True}), ("_disabled", {})]:
            name = f"perk_socket_{fill}{suffix}"
            im = draw("socket", 24, "taken", filled=fill, **kw)
            made[name] = im
            im.save(f"{out}/{name}.png")
            with open(f"{out}/{name}.gui", "w") as f:
                f.write("scaling = stretch\nsize = 12 12\n")
    for name, col in [("perk_link", k.BRONZE_D), ("perk_link_taken", k.GLOW)]:
        link(col).save(f"{out}/{name}.png")
        with open(f"{out}/{name}.gui", "w") as f:
            f.write("scaling = stretch\nsize = 1 1\n")
    if len(sys.argv) > 2:
        sheet(made, sys.argv[2])
    print(f"{len(made)} nodes")


def sheet(made, path):
    """Every node and state at 1x (as at GUI scale 2) and 4x, on the well and the panel."""
    rows = [(kind, n) for kind, n in KINDS] + [("socket_filled", 24)]
    cols = ["", "_hover", "_pressed"]
    cell = 34
    scale = 4
    w = 8 + len(STATES) * len(cols) * (cell * scale + 8)
    h = 8 + len(rows) * (cell * scale + 8) + 8
    s = Image.new("RGBA", (w, h), (*k.PANEL, 255))
    for r, (kind, n) in enumerate(rows):
        x = 8
        for state in STATES:
            for suf in cols:
                name = f"perk_{kind}_{state}{suf}" if kind != "socket_filled" else f"perk_socket_{list(FILLED)[min(STATES.index(state), 2)]}{suf}"
                if kind == "socket_filled" and STATES.index(state) > 2:
                    continue
                y = 8 + r * (cell * scale + 8)
                Image.Image.paste(s, Image.new("RGBA", (cell * scale, cell * scale), (*k.WELL, 255)), (x, y))
                if name in made:
                    big = made[name].resize((n * scale, n * scale), Image.NEAREST)
                    s.alpha_composite(big, (x + (cell * scale - n * scale) // 2, y + (cell * scale - n * scale) // 2))
                    s.alpha_composite(made[name], (x + 2, y + 2))
                x += cell * scale + 8
    s.convert("RGB").save(path)


if __name__ == "__main__":
    main()
