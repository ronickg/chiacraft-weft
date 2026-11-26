"""The knapping screen's squares in kit 2 (Planner, 11-05: "bring it into our UI theme"): a 5 x 5
flint nodule, 18 GUI px a square (36 texels), every corner stepped 3 px like the rest of the kit.

- `knap_stone`: what is struck away, the nodule's chalky cortex, raised. Hover: a strike mark and
  the glow rim (gold for a state only, the UI guide). Pressed: sunk.
- `knap_head`: the head's outline, dark glassy flint, as if the cortex were already off. Hover: a
  red rim and a crack, a warning that striking there breaks the nodule.
- `knap_gone`: a square struck away, a sunken hollow.
- `knap_cracked`: the square that broke it (the half second the screen stays).
- `knap_free`: the head's squares once it comes free (the same half second), glow rim.

Usage: python3 knap.py [--out DIR]   (writes mods/base/assets/textures/gui/knap_*.png and .gui)"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kit2 as k  # noqa: E402

OUT = sys.argv[sys.argv.index("--out") + 1] if "--out" in sys.argv else os.path.join(k.REPO, "mods/base/assets/textures/gui")
S = 36

# Cortex: chalk over flint (light, face, shade), and its flecks.
CORTEX = ((214, 204, 180), (188, 177, 152), (140, 128, 106))
CORTEX_FLECK = (160, 149, 126)
# Flint inside: blue-grey glass, a glint on it.
FLINT = ((104, 108, 120), (72, 75, 86), (46, 48, 57))
FLINT_GLINT = (150, 156, 170)
STRIKE = k.GLOW_D
CRACK = (24, 16, 12)


def stone(face, fleck, rim=k.OUTLINE, pressed=False, seed=1):
    a = k.canvas(S, S)
    m = k.shape(S, S, k.R)
    light, mid, shade = face
    k.raised(a, m, light, mid, shade, rim=rim, pressed=pressed)
    inner = k.shrink(m, 3)
    rng = np.random.default_rng(seed)
    flecks = (rng.random((S, S)) < 0.07) & inner
    k.paint(a, flecks, fleck)
    return a


def glint(a):
    """A flint's glint: a short light stroke up and to the left of the middle."""
    for i in range(4):
        a[10 + i, 9 + i, :3] = FLINT_GLINT
        a[10 + i, 10 + i, :3] = FLINT_GLINT


def strike(a):
    """A strike mark: a small starburst in the middle, cream."""
    c = S // 2
    pts = [(0, 0), (-1, 0), (0, -1), (-1, -1)]
    for d in (2, 3):
        pts += [(-d - 1, -d - 1), (d, d), (-d - 1, d), (d, -d - 1), (-d - 2, 0), (d + 1, 0), (0, -d - 2), (0, d + 1)]
    for dy, dx in pts:
        a[c + dy, c + dx, :3] = STRIKE
        a[c + dy, c + dx, 3] = 255


def crack(a, color=CRACK):
    """A crack across the face: a zigzag from the top left to the bottom right, 1 px."""
    y, x = 6, 9
    steps = [(1, 0), (1, 1), (1, 0), (0, 1), (1, 1), (1, 0), (1, 1), (0, 1), (1, 1), (1, 0), (1, 1), (1, 0),
             (0, 1), (1, 1), (1, 0), (1, 1), (1, 1), (0, 1), (1, 0), (1, 1), (1, 1), (1, 0)]
    for dy, dx in steps:
        y, x = y + dy, x + dx
        if 3 <= y < S - 3 and 3 <= x < S - 3:
            a[y, x, :3] = color
            a[y, x, 3] = 255
    # A short branch off the middle.
    for i in range(5):
        if 3 <= 18 + i < S - 3:
            a[17 - i // 2, 18 + i, :3] = color


def save(name, a):
    os.makedirs(OUT, exist_ok=True)
    Image.fromarray(a).save(os.path.join(OUT, name + ".png"))
    open(os.path.join(OUT, name + ".gui"), "w").write(f"size = {S // 2} {S // 2}\n")
    print(name)


save("knap_stone", stone(CORTEX, CORTEX_FLECK))
h = stone(CORTEX, CORTEX_FLECK, rim=k.GLOW)
strike(h)
save("knap_stone_hover", h)
p = stone(CORTEX, CORTEX_FLECK, rim=k.GLOW, pressed=True)
strike(p)
save("knap_stone_pressed", p)

f = stone(FLINT, FLINT[2], seed=2)
glint(f)
save("knap_head", f)
fh = stone(FLINT, FLINT[2], rim=k.RED_UI, seed=2)
glint(fh)
crack(fh, k.RED_UI)
save("knap_head_hover", fh)
fp = stone(FLINT, FLINT[2], rim=k.RED_UI, pressed=True, seed=2)
crack(fp, k.RED_UI)
save("knap_head_pressed", fp)

fr = stone(FLINT, FLINT[2], rim=k.GLOW, seed=2)
glint(fr)
save("knap_free", fr)

c = stone(FLINT, FLINT[2], rim=k.RED_UI, seed=2)
crack(c)
crack(np.roll(c, 0, 1))
save("knap_cracked", c)

# A hollow: sunken like a well, a few chips of cortex left at its rim.
g = k.canvas(S, S)
m = k.shape(S, S, k.R)
ring = k.edge(m)
inner = m & ~ring
k.paint(g, ring, k.OUTLINE)
k.paint(g, inner, k.WELL)
up = np.zeros_like(m)
up[1:] = ring[:-1]
lf = np.zeros_like(m)
lf[:, 1:] = ring[:, :-1]
k.paint(g, (up | lf) & inner, k.WELL_D)
rng = np.random.default_rng(3)
chips = (rng.random((S, S)) < 0.05) & inner & ~k.shrink(m, 4)
k.paint(g, chips, CORTEX[2])
save("knap_gone", g)
