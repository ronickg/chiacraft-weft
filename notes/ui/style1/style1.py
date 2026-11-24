"""Style 1 (Ronald's pick, 2026-11-23): the shared UI parts cut from two generated sheets.

The sheets were made with Codex from the chosen title screen (game-art skill, `6 UI` guide), on an
exact grid (one UI pixel = 5 x 5 image pixels), and are kept in the archive:

    ~/Coding/chiacraft-archive/images/ui-redo/2026-11-23/parts/buttons_try3_chosen.png
    ~/Coding/chiacraft-archive/images/ui-redo/2026-11-23/parts/frames_try3_chosen.png

    ~/.venvs/blocky/bin/python notes/ui/style1/style1.py [--out DIR] [--dry]

Cuts each part exactly (the most common colour in each cell's middle), then in code only:
- one footprint for every state (the hover's glow corners go: the guide's "states keep one footprint");
- stretch zones made even (the middle row and column repeated), so nine-slicing doesn't smear
  the bands or grain;
- the main button at rest: the orange face, a step darker, with the bronze rim of the secondary
  (gold only for hover and selected);
- Quit: the secondary button (Ronald: no red Quit);
- the slot highlight: the selected slot's gold ring alone, the inside clear;
- the panel's inside a little see-through (style 1 lets the painting show faintly).

Writes `<name>.png` and `<name>.gui` into mods/base/assets/textures/gui; every file it replaces is
kept first in the archive (`gui-before-style1/`).
"""
import argparse
import os
import shutil
import sys

import numpy as np
from PIL import Image

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "mods", "base", "source", "ui"))
import sheet  # noqa: E402
from pxkit import tone  # noqa: E402

ARC = os.path.expanduser("~/Coding/chiacraft-archive/images/ui-redo/2026-11-23")
BUTTONS = os.path.join(ARC, "parts", "grid_buttons_try1.png")
FRAMES = os.path.join(ARC, "parts", "grid_frames_try1.png")
TABS = os.path.join(ARC, "parts", "grid_tabs_try1.png")
SMALL = os.path.join(ARC, "parts", "grid_small_try2.png")
FRAMES_OLD = os.path.join(ARC, "parts", "frames_try3_chosen.png")  # the slots
ARC2 = os.path.expanduser("~/Coding/chiacraft-archive/images/ui-redo/2026-11-24")
DOOR = os.path.join(ARC2, "parts", "door_try2.png")  # Quit's door (10 image px a pixel)
PENCIL = os.path.join(ARC2, "parts", "pencil_try1.png")  # the editor square (before icons16)
ICONS = os.path.join(ARC2, "parts", "icons16_try3.png")  # the title's six icons, 16 x 16, 10 px a pixel, solid (Planner 11-24: fuller)
CORNERS = os.path.join(ARC2, "parts", "corners_try3.png")  # the title frame's four plates, 48 x 48
SEE = 217  # menu screens' insides: about 85% opaque, the world shows through (Ronald, 11-24)
GUI = os.path.join(ROOT, "mods", "base", "assets", "textures", "gui")
K = 5  # image pixels per UI pixel


def cut(path, box, k=K):
    a = np.array(Image.open(path).convert("RGBA"))
    bg = sheet.background(a)
    x, y, w, h = box
    out = np.zeros((h // k, w // k, 4), np.uint8)
    for j in range(h // k):
        for i in range(w // k):
            cy, cx = y + j * k + k // 2, x + i * k + k // 2
            if bg[cy - 1:cy + 2, cx - 1:cx + 2].mean() > 0.5:
                continue
            cell = a[cy - 1:cy + 2, cx - 1:cx + 2, :3].reshape(-1, 3)
            vals, counts = np.unique(cell, axis=0, return_counts=True)
            out[j, i, :3] = vals[counts.argmax()]
            out[j, i, 3] = 255
    return out


def grab(path, near, size, slack=60, k=K):
    """A part of `size` pixels drawn near `near` (x, y): found by its outline (the non-background
    box around `near`), then sampled at the pitch its drawn width gives (the sheets drift a little
    off 5 image px a pixel)."""
    a = np.array(Image.open(path).convert("RGBA"))
    bg = sheet.background(a)
    x0, y0 = max(near[0] - slack, 0), max(near[1] - slack, 0)
    x1 = min(near[0] + size[0] * k + slack, a.shape[1])
    y1 = min(near[1] + size[1] * k + slack, a.shape[0])
    fg = ~bg[y0:y1, x0:x1]
    ys, xs = np.nonzero(fg)
    bx, by = x0 + xs.min(), y0 + ys.min()
    pw, ph = (xs.max() + 1 - xs.min()) / size[0], (ys.max() + 1 - ys.min()) / size[1]
    out = np.zeros((size[1], size[0], 4), np.uint8)
    for j in range(size[1]):
        for i in range(size[0]):
            cy, cx = int(by + (j + 0.5) * ph), int(bx + (i + 0.5) * pw)
            if bg[cy - 1:cy + 2, cx - 1:cx + 2].mean() > 0.5:
                continue
            cell = a[cy - 1:cy + 2, cx - 1:cx + 2, :3].reshape(-1, 3)
            vals, counts = np.unique(cell, axis=0, return_counts=True)
            out[j, i, :3] = vals[counts.argmax()]
            out[j, i, 3] = 255
    return out, (pw, ph)


def even(a, border, rows=True):
    """The stretch zones made even: inside the border, every row is the middle row and every
    column the middle column (the corners and edges' ends stay as drawn)."""
    l, t, r, b = border
    h, w = a.shape[:2]
    a = a.copy()
    mid_y, mid_x = (t + h - b) // 2, (l + w - r) // 2
    if rows and (t or b):  # no border across: that way doesn't stretch, keep it as drawn
        a[t:h - b, :] = a[mid_y:mid_y + 1, :]
    if l or r:
        a[:, l:w - r] = a[:, mid_x:mid_x + 1]
    return a


BAYER = np.array([[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]) / 16.0


def textured(a, box, seed, light=0.07, dark=0.10, noise=0.03, top=0.22, bottom=0.78):
    """A face made less flat (Ronald, 11-24: "something still looks worse off"; the mock's faces
    have a gradient, a lit top row, a dark bottom row and faint noise): inside `box` (x0, y0, x1,
    y1), lighter at the top and darker at the bottom in ordered-dither steps of 4%, the top row
    lit toward cream, the bottom row darkened, and every pixel nudged by +-`noise` (a fixed
    pattern, so every state of a part shares it)."""
    x0, y0, x1, y1 = box
    a = a.copy()
    rng = np.random.default_rng(seed)
    h = y1 - y0
    rgb = a[..., :3].astype(float)
    for y in range(y0, y1):
        t = (y - y0) / max(h - 1, 1)
        f = 1.0 + light - (light + dark) * t
        for x in range(x0, x1):
            if a[y, x, 3] == 0:
                continue
            q = np.floor((f - 1.0) / 0.04 + BAYER[y % 4, x % 4]) * 0.04 + 1.0
            q *= 1.0 + rng.choice([-noise, 0.0, 0.0, noise])
            rgb[y, x] *= q
    rgb[y0, x0:x1] = rgb[y0, x0:x1] * (1 - top) + np.array([255, 240, 200]) * top
    rgb[y1 - 1, x0:x1] *= bottom
    a[..., :3] = np.clip(rgb, 0, 255).astype(np.uint8)
    return a


def shadowed(a, n=4, dy=3, alpha=120, rgb=(20, 10, 5)):
    """The part on a drop shadow `dy` pixels below it (the mock's buttons sit on one), in an
    `n`-pixel margin like the hover's glow, so every state keeps one footprint (`outset`)."""
    h, w = a.shape[:2]
    out = np.zeros((h + 2 * n, w + 2 * n, 4), np.uint8)
    mask = np.zeros(out.shape[:2], bool)
    mask[n + dy:n + dy + h, n:n + w] = a[..., 3] > 0
    out[mask, :3] = rgb
    out[mask, 3] = alpha
    part = np.zeros_like(out)
    part[n:n + h, n:n + w] = a
    over = part[..., 3] > 0
    out[over] = part[over]
    return out


def glow(a, n=4, rgb=(255, 196, 42), alphas=(150, 90, 45, 20)):
    """A hover's glow: `n` pixels of gold around the part's outline, fading out (the mock's
    hovered button glows). The part grows by `n` on every side; its `.gui` says `outset`."""
    from scipy.ndimage import binary_dilation
    h, w = a.shape[:2]
    out = np.zeros((h + 2 * n, w + 2 * n, 4), np.uint8)
    out[n:n + h, n:n + w] = a
    inside = out[..., 3] > 0
    done = inside.copy()
    cross = np.array([[0, 1, 0], [1, 1, 1], [0, 1, 0]], bool)
    for i in range(n):
        grown = binary_dilation(done, structure=np.ones((3, 3), bool) if i % 2 == 0 else cross)
        ring = grown & ~done
        out[ring, :3] = rgb
        out[ring, 3] = alphas[i]
        done = grown
    return out


def thicken(a, at, n):
    """The frame `n` pixels thicker on every side: row `at` (and its mirror, and the same column
    and its mirror) repeated, inside the oak's grain where the plates are plain."""
    for _ in range(n):
        a = np.concatenate([a[:at + 1], a[at:at + 1], a[at + 1:]], 0)
        b = a.shape[0] - 1 - at
        a = np.concatenate([a[:b], a[b:b + 1], a[b:]], 0)
        a = np.concatenate([a[:, :at + 1], a[:, at:at + 1], a[:, at + 1:]], 1)
        b = a.shape[1] - 1 - at
        a = np.concatenate([a[:, :b], a[:, b:b + 1], a[:, b:]], 1)
    return a


def hexc(h):
    return np.array([int(h[i:i + 2], 16) for i in (1, 3, 5)], float)


def luma(c):
    return 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]


def grain(shape, seed, density=1 / 12):
    """Scattered single pixels one value step off (Pixel Parmesan: gradients first, grain second
    and faint, never a checkerboard): -1, 0 or +1 per pixel, at most one pixel in `1/density`."""
    rng = np.random.default_rng(seed)
    g = np.zeros(shape, int)
    hit = rng.random(shape) < density
    g[hit] = rng.choice([-1, 1], hit.sum())
    return g


def banded(a, face, bands, hi, lo, seed, heights=None, step=0.06):
    """A face in banded value steps (`reports/UI craft.md` change 4): inside `face` (x0, y0, x1,
    y1), one highlight row `hi`, the `bands` colours top to bottom in rows of `heights` (even split
    if None), one dark row `lo`; faint grain one step off on the bands."""
    x0, y0, x1, y1 = face
    a = a.copy()
    n = y1 - y0 - 2
    heights = heights or [n // len(bands) + (1 if i < n % len(bands) else 0) for i in range(len(bands))]
    assert sum(heights) == n, (heights, n)
    rows = [hexc(hi)] + [hexc(c) for c, k in zip(bands, heights) for _ in range(k)] + [hexc(lo)]
    g = grain(a.shape[:2], seed)
    for j, c in enumerate(rows):
        y = y0 + j
        for x in range(x0, x1):
            if a[y, x, 3] == 0:
                continue
            f = 1.0 + (step * g[y, x] if 0 < j < len(rows) - 1 else 0.0)
            a[y, x, :3] = np.clip(c * f, 0, 255)
    return a


def taller(a, h):
    """The part `h` pixels tall: its middle row repeated (the rim's columns come along)."""
    mid = a.shape[0] // 2
    extra = h - a.shape[0]
    return np.concatenate([a[:mid], np.repeat(a[mid:mid + 1], extra, 0), a[mid:]], 0)


def copper(a, depth=7):
    """A button's rim in copper all round (Planner 11-24: every button in the mock has a bright
    copper rim with chamfered corners; ours was dark brown below and right): by each pixel's
    depth from the outline, a lit ramp on the top and left sides and a darker copper ramp on the
    bottom and right, split along the chamfers' diagonals. The outline and the inner dark line
    stay."""
    from scipy.ndimage import distance_transform_cdt
    a = a.copy()
    h, w = a.shape[:2]
    d = distance_transform_cdt(np.pad(a[..., 3] > 0, 1), metric="chessboard")[1:-1, 1:-1]
    lit = ["#a85a28", "#f2b25e", "#d88a44", "#b06a32", "#8a4e28"]
    dark = ["#8a4420", "#c27434", "#a85e2a", "#8e4e24", "#6e3c1e"]
    for y in range(h):
        for x in range(w):
            k = d[y, x] - 2  # 1 is the outline; depths 2..depth-1 are the rim
            if a[y, x, 3] == 0 or k < 0 or k >= len(lit) or d[y, x] >= depth:
                continue
            top_left = min(y, x) <= min(h - 1 - y, w - 1 - x)
            a[y, x, :3] = hexc((lit if top_left else dark)[k])
    return a


def margin(a, n, dy=0):
    """The part in an `n`-pixel clear margin, moved `dy` pixels down inside it (a pressed button)."""
    h, w = a.shape[:2]
    out = np.zeros((h + 2 * n, w + 2 * n, 4), np.uint8)
    out[n + dy:n + dy + h, n:n + w] = a
    return out


def drop(a, n, dy, alpha=150, rgb=(20, 10, 5)):
    """A hard shadow `dy` pixels straight down under the part, in its `n`-pixel margin."""
    out = margin(a, n)
    h, w = a.shape[:2]
    sh = np.zeros(out.shape[:2], bool)
    sh[n + dy:n + dy + h, n:n + w] = a[..., 3] > 0
    sh &= out[..., 3] == 0
    out[sh, :3] = rgb
    out[sh, 3] = alpha
    return out


def rings(a, n, rgb, alphas, width=2):
    """A glow of stepped rings, each `width` pixels, around the part (no blur: it stays on the
    grid), in an `n`-pixel margin."""
    from scipy.ndimage import binary_dilation
    out = margin(a, n)
    done = out[..., 3] > 0
    for al in alphas:
        grown = done
        for _ in range(width):
            grown = binary_dilation(grown, structure=np.ones((3, 3), bool))
        ring = grown & ~done
        out[ring, :3] = rgb
        out[ring, 3] = al
        done = grown
    return out


def title_panel(big):
    """The title's frame (`reports/UI craft.md` changes 5, 6, 9, 14; Planner 11-24: corner plates
    twice the size, over the frame): the 20-pixel oak bands of `big` darkened toward the mock's
    wood and given broken lengthwise grain; the generated 48-pixel bronze plates at the corners,
    3 pixels past the frame; inside darker and cooler than the buttons, in stepped rings toward
    the frame, with a 2-GUI shadow from the top and left frame; a 2-GUI drop shadow below.
    Borders 52 pixels (26 GUI), a 32-pixel middle that repeats (`middle = tile`), a 4-pixel
    margin (`outset = 2`)."""
    m, band, B, T = 4, 20, 52, 32
    W = 2 * B + T
    top = big[:band, 50].astype(float)  # the top band's profile, outside to inside
    left = big[50, :band].astype(float)
    bottom = big[-band:, 50][::-1].astype(float)  # outside first
    right = big[50, -band:][::-1].astype(float)
    # Darker wood: the oak rows (not the outline, not the one lit row) toward the mock's L 23-60.
    lit = int(np.argmax([luma(c) for c in top[:6, :3]]))

    def darker(prof, keep):
        prof = prof.copy()
        for i in range(len(prof)):
            if i != keep and luma(prof[i, :3]) > 30:
                prof[i, :3] *= 0.62
        return prof
    top, left = darker(top, lit), darker(left, int(np.argmax([luma(c) for c in left[:6, :3]])))
    bottom, right = darker(bottom, -1), darker(right, -1)
    rng = np.random.default_rng(3)

    def strip(prof, length, seed):
        """A band `length` long from its profile, with grain: on every 3rd to 5th oak row, darker
        streaks broken every 6 to 12 pixels (periodic in `T`, as the middle repeats)."""
        r = np.random.default_rng(seed)
        out = np.repeat(prof[:, None, :], length, 1)
        y = 6
        while y < band - 4:
            x = r.integers(0, T)
            while x < T * 2:
                run = r.integers(6, 13)
                for xx in range(x, x + run // 2):
                    for xt in range(xx % T, length, T):
                        out[y, xt, :3] *= 0.8
                x += run
            y += r.integers(3, 6)
        return out
    H = W
    img = np.zeros((H, W, 4), float)
    inner = (m + band, m + band, W - m - band, H - m - band)  # the inside's box
    # The inside: darker and cooler than the button faces (L 44), rings by distance to the frame.
    steps = [(2, "#161214"), (6, "#1a1618"), (12, "#1e191c"), (20, "#231d21"), (999, "#2a2228")]
    g = grain((H, W), 4)
    for y in range(inner[1], inner[3]):
        for x in range(inner[0], inner[2]):
            d = min(x - inner[0], y - inner[1], inner[2] - 1 - x, inner[3] - 1 - y)
            c = next(hexc(col) for lim, col in steps if d < lim)
            if y - inner[1] < 5 or x - inner[0] < 5:  # the frame's shadow on the inside, top and left
                c = c * 0.62
            img[y, x, :3] = c * (1 + 0.05 * g[y, x])
            img[y, x, 3] = SEE
    # The bands (corners first by the vertical bands, then the horizontal over them: the plates
    # cover where they meet).
    L = H - 2 * m
    img[m:H - m, m:m + band] = strip(left, L, 5).transpose(1, 0, 2)
    img[m:H - m, W - m - band:W - m] = strip(right, L, 6).transpose(1, 0, 2)[:, ::-1]
    img[m:m + band, m:W - m] = strip(top, L, 7)
    img[H - m - band:H - m, m:W - m] = strip(bottom, L, 8)[::-1]
    # The drop shadow: 4 pixels straight down.
    img[H - m:H, m + 2:W - m - 2, :3] = (20, 10, 5)
    img[H - m:H, m + 2:W - m - 2, 3] = 130
    # The plates, 3 pixels past the frame's outer edge.
    o = m - 3
    for (cx, cy), (px, py) in zip([(300, 150), (900, 150), (300, 600), (900, 600)],
                                  [(o, o), (W - o - 48, o), (o, H - o - 48), (W - o - 48, H - o - 48)]):
        plate = cut(CORNERS, (cx, cy, 240, 240)).astype(float)
        on = plate[..., 3] > 0
        img[py:py + 48, px:px + 48][on] = plate[on]
    out = np.clip(img, 0, 255).astype(np.uint8)
    return out, gui("nine_slice", (B,) * 4, (W, H), "outset = 2\nmiddle = tile\n")


def gui(scaling, border=None, size=None, extra=""):
    s = f"scaling = {scaling}\n"
    if border is not None:
        s += "border = " + " ".join(f"{v / 2:g}" for v in border) + "\n"
    if size is not None:
        s += f"size = {size[0] / 2:g} {size[1] / 2:g}\n"
    return s + extra


def build():
    parts = {}
    pitches = {}

    def take(sheet_, name, near, size, slack=60):
        px, pitch = grab(sheet_, near, size, slack)
        pitches[name] = pitch
        return px

    # Buttons (`reports/UI craft.md` changes 3, 4, 6, 12): the cut's rim (7 pixels, 4-pixel
    # chamfers) around faces repainted in banded value steps, a highlight row on top and a dark
    # row at the bottom, faint grain one step off. At rest a hard shadow 2 GUI straight down;
    # hover (and the title's selected) the mock's deep orange with a bright yellow inner rim and a
    # glow of stepped rings; pressed the face 1 GUI lower with no shadow and no highlight.
    at = {"": (150, 200), "_hover": (850, 200), "_pressed": (150, 560), "_disabled": (850, 560)}
    raw = {k: take(BUTTONS, "button" + k, xy, (96, 40)) for k, xy in at.items()}
    mask = raw[""][..., 3] > 0
    bb = (8, 8, 8, 8)
    for k, a in raw.items():
        a[~mask] = 0
        raw[k] = even(a, bb, rows=False)
    rim = even(copper(raw[""]), bb, rows=False)  # the secondary's rim, copper all round (columns evened again: the chamfers shade the first ones)
    face0 = rim[20, 48, :3].astype(float)  # the brown face, L 44

    def brown(*ls):
        return ["#%02x%02x%02x" % tuple(np.clip(face0 * l / luma(face0), 0, 255).astype(int)) for l in ls]
    orange = ["#d57302", "#c26502", "#ad5702", "#994802"]  # the mock's selected face, top to bottom
    dim = ["#%02x%02x%02x" % tuple((hexc(c) * 0.86).astype(int)) for c in orange]  # the main at rest

    def warm_rim(a):
        """Hover's rim: the bronze warmed toward gold, its two innermost rings orange-gold and bright yellow."""
        a = a.copy()
        h, w = a.shape[:2]
        r = np.zeros((h, w), bool)
        r[:7, :] = r[-7:, :] = r[:, :7] = r[:, -7:] = True
        lit = r & (a[..., 3] > 0) & (a[..., :3].astype(int).sum(-1) > 120)
        a[lit, :3] = np.clip(a[lit, :3] * 0.7 + np.array([255, 176, 64]) * 0.3, 0, 255)
        for d, col in ((5, "#ffb030"), (6, "#ffd54a")):  # two steps, the mock's line about 1 GUI
            ring = np.zeros((h, w), bool)
            ring[d, d:w - d] = ring[h - 1 - d, d:w - d] = ring[d:h - d, d] = ring[d:h - d, w - 1 - d] = True
            a[ring & (a[..., 3] > 0), :3] = hexc(col)
        return a

    def states(h, heights, n, glow_alphas, seed):
        """Every state of a button `h` pixels tall (its face `heights` bands), in an `n`-pixel margin."""
        base = taller(rim, h) if h != rim.shape[0] else rim
        w = base.shape[1]
        face = (7, 7, w - 7, h - 7)
        nb = len(heights)
        pick = (lambda c: c[:nb] if nb == 4 else [c[0], c[1], c[3]])
        lo_brown, hi_brown = brown(30)[0], brown(62)[0]
        item = banded(base, face, brown(50, 46, 42, 38)[:nb] if nb == 4 else brown(48, 44, 40), hi_brown, lo_brown, seed, heights)
        main = banded(base, face, pick(dim), "#e08a20", "#6e3202", seed, heights)
        hover = banded(warm_rim(base), face, pick(orange), "#f0a030", "#7a3802", seed, heights)
        pressed = banded(base, face, pick(dim), pick(dim)[0], "#6e3202", seed, heights)
        gray = item.copy()
        g = gray[..., :3].astype(float)
        gray[..., :3] = (g * 0.55 + g.mean(-1, keepdims=True) * 0.25).astype(np.uint8)
        dy = 4  # 2 GUI
        return {
            "item": drop(item, n, dy), "main": drop(main, n, dy),
            "_hover": rings(hover, n, (255, 150, 30), glow_alphas),
            "_pressed": margin(pressed, n, dy=2),
            "_disabled": drop(gray, n, dy, alpha=70),
        }

    def put(name, a, border, ex):
        parts[name] = (a, gui("nine_slice", border, a.shape[1::-1], ex))
    sel = "selected_offset = 0.5\n"
    # The windows' buttons, 20 GUI (40 pixels): 3 bands of 8; the top and bottom borders take the
    # highlight and the first and last bands, so a taller button only adds more of the middle band.
    # A 4-pixel margin (`outset = 2`): the shadow and a two-ring glow.
    st = states(40, [8, 8, 8], 4, (170, 75), seed=1)
    gb = (12, 20, 12, 20)
    tail = "outset = 2\nmiddle = tile\n"
    for k in ("_hover", "_pressed", "_disabled"):
        put("button_secondary" + k, st[k], gb, sel + tail)
        put("button" + k, st[k], gb, sel + tail)
        if k != "_disabled":
            put("button_quit" + k, st[k], gb, tail)
    put("button_secondary", st["item"], gb, sel + tail)
    put("button_quit", st["item"], gb, tail)
    put("button", st["main"], gb, sel + tail)
    # The title's buttons (`menu_main`, `menu_item`), cut at their height, 26 GUI (52 pixels): 4
    # bands of 9, never stretched; a 10-pixel margin (`outset = 5`) for a glow of five rings 1 GUI
    # each, fading out (the mock's halo spills 4-6 GUI; the buttons are 7 apart). The borders leave
    # a one-row middle (the second band's last row).
    st = states(52, [9, 9, 9, 9], 10, (205, 150, 100, 60, 28), seed=1)
    gb = (18, 35, 18, 36)
    tail = "outset = 5\nmiddle = tile\n"
    for k in ("_hover", "_pressed", "_disabled"):
        put("menu_main" + k, st[k], gb, tail)
        put("menu_item" + k, st[k], gb, tail)
    put("menu_main", st["main"], gb, tail)
    put("menu_item", st["item"], gb, tail)

    # Frames: the title's 96 x 96 (18-pixel oak, 28-pixel bronze plates) and the windows' 64 x 64
    # (10-pixel oak, 16-pixel plates), drawn at the same pixel; pulled from orange toward bronze
    # (the guide's "too yellow" rule); insides a little see-through.
    def frame(near, size, cap, rim, grow=0, at=0, alpha=232):
        f = tone(take(FRAMES, f"frame{size}", near, (size, size)))
        f = even(f, (cap, cap, cap, cap))
        if grow:
            f = thicken(f, at, grow)
            size, cap, rim = size + 2 * grow, cap + grow, rim + grow
        inside = np.zeros(f.shape[:2], bool)
        inside[rim:-rim, rim:-rim] = True
        c = f[size // 2, size // 2, :3].astype(int)
        flat = inside & (np.abs(f[..., :3].astype(int) - c).sum(-1) < 12)
        # Faint noise on the inside (no gradient: the middle repeats).
        noisy = textured(f, (0, 0, size, size), seed=2, light=0.0, dark=0.0, top=0.0, bottom=1.0)
        f[flat] = noisy[flat]
        f[flat, 3] = alpha
        return f
    # The title's frame two pixels thicker than drawn (20 pixels, 10 GUI, the mock's at the same
    # width; plates 30): its row 5 (and column 5, and their mirrors: plain oak and plate, clear of the rivets) repeated.
    big = frame((80, 80), 96, 28, 18, grow=2, at=5)
    parts["menu_panel"] = title_panel(big)
    # In a world, windows stay as they were (232); out of one (the title's menus) the engine draws
    # `panel` as `menu_window`, the same frame with a see-through inside (Ronald, 11-24: "we should
    # make it also a little transparent in the menu screen").
    small = frame((760, 80), 64, 16, 10)
    parts["panel"] = (small, gui("nine_slice", (16,) * 4, (64, 64), "middle = tile\n"))
    see = frame((760, 80), 64, 16, 10, alpha=SEE)
    for name in ("menu_window", "news"):
        parts[name] = (see, gui("nine_slice", (16,) * 4, (64, 64), "middle = tile\n"))
    div = even(tone(take(FRAMES, "divider", (760, 600), (96, 6))), (6, 0, 6, 0))
    parts["divider"] = (div, gui("nine_slice", (6, 0, 6, 0), (96, 6)))

    # Tabs, category buttons, list rows: 4 states each, a 4-pixel rim with 2-pixel chamfers.
    states = ("", "_hover", "_pressed", "_disabled")
    rows = [("tab", (48, 26), 100, (80, 400, 720, 1040), ""),
            ("category", (32, 24), 360, (80, 400, 720, 1040), "selected_offset = 0.5\n"),
            ("list_row", (64, 24), 600, (80, 440, 800, 1160), "selected_offset = 0.5\n")]
    for name, size, y, xs, ex in rows:
        for k, x in zip(states, xs):
            a = even(take(TABS, name + k, (x, y), size, 30), (5, 5, 5, 5), rows=False)
            parts[name + k] = (a, gui("nine_slice", (5, 5, 5, 5), size, ex if k == "" else ""))

    # The small parts.
    small_parts = [
        ("input", (80, 100), (64, 18), 4), ("input_focused", (480, 100), (64, 18), 4),
        ("tooltip", (900, 100), (32, 24), 4), ("well", (1150, 100), (32, 24), 4),
        ("scrollbar", (80, 330), (12, 40), 4),
        ("scrollbar_handle", (220, 330), (12, 28), 4), ("scrollbar_handle_hover", (360, 330), (12, 28), 4),
        ("slider", (520, 330), (62, 10), 4),
        ("slider_knob", (900, 330), (14, 20), 4), ("slider_knob_hover", (1040, 330), (14, 20), 4),
        ("close", (80, 650), (14, 14), 3), ("close_hover", (220, 650), (14, 14), 3),
        ("close_pressed", (360, 650), (14, 14), 3), ("chip", (520, 650), (16, 16), 3),
    ]
    for name, near, size, bd in small_parts:
        a = take(SMALL, name, near, size)
        if not name.startswith("scrollbar_handle"):  # the handle's grip line stays as drawn
            a = even(a, (bd,) * 4)
        parts[name] = (a, gui("nine_slice", (bd,) * 4, size))

    # The title's icons: one icon pixel a UI pixel, like the others (no .gui: drawn at their size).
    # The title's icons, one set at one size (`reports/UI craft.md` change 10): 16 x 16, drawn at
    # twice their pixels (`size = 16 16`), so an icon pixel is a font pixel of the 2x labels.
    names = ("menu_singleplayer", "menu_multiplayer", "menu_mods", "menu_settings", "menu_quit", "menu_editor")
    for i, name in enumerate(names):
        parts[name] = (cut(ICONS, (128 + i * 224, 432, 160, 160), k=10), "scaling = stretch\nsize = 16 16\n")

    # Slots stay the earlier cut (an 18-GUI slot has room for a 2-GUI rim only).
    slot = even(cut(FRAMES_OLD, (660, 100, 180, 180)), (4, 4, 4, 4))
    sel = even(cut(FRAMES_OLD, (960, 100, 180, 180)), (4, 4, 4, 4))
    ring = (np.abs(sel[..., :3].astype(int) - slot[..., :3].astype(int)).sum(-1) > 30) & (sel[..., 3] > 0)
    hl = np.where(ring[..., None], sel, 0).astype(np.uint8)
    parts["slot"] = (slot, gui("nine_slice", (4, 4, 4, 4), (36, 36), "icon = 16\n"))
    parts["slot_highlight"] = (hl, gui("nine_slice", (4, 4, 4, 4), (36, 36)))
    for n, (pw, ph) in sorted(pitches.items()):
        if abs(pw - K) > 0.4 or abs(ph - K) > 0.4:
            print(f"warning: {n} drawn at {pw:.2f} x {ph:.2f} image px a pixel (want {K})")
    return parts


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=GUI)
    ap.add_argument("--dry", action="store_true")
    a = ap.parse_args()
    parts = build()
    keep = os.path.join(ARC, "gui-before-style1")
    for name, (px, g) in sorted(parts.items()):
        png, gf = os.path.join(a.out, name + ".png"), os.path.join(a.out, name + ".gui")
        print(f"{name:28} {px.shape[1]}x{px.shape[0]}")
        if a.dry:
            continue
        if a.out == GUI:
            os.makedirs(keep, exist_ok=True)
            for f in (png, gf):
                if os.path.exists(f) and not os.path.exists(os.path.join(keep, os.path.basename(f))):
                    shutil.copy(f, keep)
        Image.fromarray(px).save(png)
        if g is not None:
            open(gf, "w").write(g)


if __name__ == "__main__":
    main()
