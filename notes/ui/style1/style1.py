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
GUI = os.path.join(ROOT, "mods", "base", "assets", "textures", "gui")
K = 5  # image pixels per UI pixel


def cut(path, box):
    a = np.array(Image.open(path).convert("RGBA"))
    bg = sheet.background(a)
    x, y, w, h = box
    out = np.zeros((h // K, w // K, 4), np.uint8)
    for j in range(h // K):
        for i in range(w // K):
            cy, cx = y + j * K + K // 2, x + i * K + K // 2
            if bg[cy - 1:cy + 2, cx - 1:cx + 2].mean() > 0.5:
                continue
            cell = a[cy - 1:cy + 2, cx - 1:cx + 2, :3].reshape(-1, 3)
            vals, counts = np.unique(cell, axis=0, return_counts=True)
            out[j, i, :3] = vals[counts.argmax()]
            out[j, i, 3] = 255
    return out


def grab(path, near, size, slack=60):
    """A part of `size` pixels drawn near `near` (x, y): found by its outline (the non-background
    box around `near`), then sampled at the pitch its drawn width gives (the sheets drift a little
    off 5 image px a pixel)."""
    a = np.array(Image.open(path).convert("RGBA"))
    bg = sheet.background(a)
    x0, y0 = max(near[0] - slack, 0), max(near[1] - slack, 0)
    x1 = min(near[0] + size[0] * K + slack, a.shape[1])
    y1 = min(near[1] + size[1] * K + slack, a.shape[0])
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

    # Buttons: 96 x 40 (20 GUI, the buttons' own height), a 7-pixel rim with 4-pixel chamfers.
    # Only the columns are made even: the faces' gradient bands run across, and a taller button
    # stretches them (nearest) rather than repeating one row.
    at = {"": (150, 200), "_hover": (850, 200), "_pressed": (150, 560), "_disabled": (850, 560)}
    raw = {k: take(BUTTONS, "button" + k, xy, (96, 40)) for k, xy in at.items()}
    mask = raw[""][..., 3] > 0
    bb = (8, 8, 8, 8)
    for k, a in raw.items():
        a[~mask] = 0
        raw[k] = even(a, bb, rows=False)
    extra = "selected_offset = 0.5\n"
    for k, a in raw.items():
        parts["button_secondary" + k] = (a, gui("nine_slice", bb, a.shape[1::-1], extra))
        if k != "_disabled":
            parts["button_quit" + k] = (a, gui("nine_slice", bb, a.shape[1::-1]))
    # The main button at rest: the hover's orange face a step darker, inside the secondary's rim.
    main = raw["_hover"].copy()
    main[..., :3] = (main[..., :3].astype(float) * 0.88).astype(np.uint8)
    h, w = main.shape[:2]
    ring = np.zeros((h, w), bool)
    ring[:7, :] = ring[-7:, :] = True
    ring[:, :7] = ring[:, -7:] = True
    main[ring] = raw[""][ring]
    parts["button"] = (main, gui("nine_slice", bb, main.shape[1::-1], extra))
    for k in ("_hover", "_pressed", "_disabled"):
        parts["button" + k] = (raw[k], gui("nine_slice", bb, raw[k].shape[1::-1], extra))

    # Frames: the title's 96 x 96 (18-pixel oak, 28-pixel bronze plates) and the windows' 64 x 64
    # (10-pixel oak, 16-pixel plates), drawn at the same pixel; pulled from orange toward bronze
    # (the guide's "too yellow" rule); insides a little see-through.
    def frame(near, size, cap, rim):
        f = tone(take(FRAMES, f"frame{size}", near, (size, size)))
        f = even(f, (cap, cap, cap, cap))
        inside = np.zeros(f.shape[:2], bool)
        inside[rim:-rim, rim:-rim] = True
        c = f[size // 2, size // 2, :3].astype(int)
        flat = inside & (np.abs(f[..., :3].astype(int) - c).sum(-1) < 12)
        f[flat, 3] = 232
        return f
    big = frame((80, 80), 96, 28, 18)
    parts["menu_panel"] = (big, gui("nine_slice", (28,) * 4, (96, 96)))
    small = frame((760, 80), 64, 16, 10)
    for name in ("panel", "news"):
        parts[name] = (small, gui("nine_slice", (16,) * 4, (64, 64)))
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
        open(gf, "w").write(g)


if __name__ == "__main__":
    main()
