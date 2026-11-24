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

ARC = os.path.expanduser("~/Coding/chiacraft-archive/images/ui-redo/2026-11-23")
BUTTONS = os.path.join(ARC, "parts", "buttons_try3_chosen.png")
FRAMES = os.path.join(ARC, "parts", "frames_try3_chosen.png")
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


def even(a, border):
    """The stretch zones made even: inside the border, every row is the middle row and every
    column the middle column (the corners and edges' ends stay as drawn)."""
    l, t, r, b = border
    h, w = a.shape[:2]
    a = a.copy()
    mid_y, mid_x = (t + h - b) // 2, (l + w - r) // 2
    if t or b:  # no border across: that way doesn't stretch, keep it as drawn
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
    # Buttons: 64 x 22, a 3-pixel rim, the face's bands in the top and bottom 9 rows (only the
    # flat middle stretches when a button is taller).
    cols = {"": 35, "_hover": 420, "_pressed": 805, "_disabled": 1190}
    raw = {k: cut(BUTTONS, (x, 455, 320, 110)) for k, x in cols.items()}
    mask = raw[""][..., 3] > 0
    bb = (4, 9, 4, 9)
    for k, a in raw.items():
        a[~mask] = 0
        raw[k] = even(a, bb)
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
    ring[:3, :] = ring[-3:, :] = True
    ring[:, :3] = ring[:, -3:] = True
    main[ring] = raw[""][ring]
    parts["button"] = (main, gui("nine_slice", bb, main.shape[1::-1], extra))
    for k in ("_hover", "_pressed", "_disabled"):
        parts["button" + k] = (raw[k], gui("nine_slice", bb, raw[k].shape[1::-1], extra))
    # The panel: 64 x 64, corner caps 10 pixels, its inside a little see-through.
    panel = even(cut(FRAMES, (100, 100, 320, 320)), (10, 10, 10, 10))
    inside = np.zeros(panel.shape[:2], bool)
    inside[9:-9, 9:-9] = True
    flat = inside & (np.abs(panel[..., :3].astype(int) - panel[32, 32, :3].astype(int)).sum(-1) < 12)
    panel[flat, 3] = 232
    for name in ("panel", "news"):
        parts[name] = (panel, gui("nine_slice", (10, 10, 10, 10), (64, 64)))
    # Slots: 36 x 36, a 4-pixel border; the highlight is the selected slot's ring alone.
    slot = even(cut(FRAMES, (660, 100, 180, 180)), (4, 4, 4, 4))
    sel = even(cut(FRAMES, (960, 100, 180, 180)), (4, 4, 4, 4))
    ring = (np.abs(sel[..., :3].astype(int) - slot[..., :3].astype(int)).sum(-1) > 30) & (sel[..., 3] > 0)
    hl = np.where(ring[..., None], sel, 0).astype(np.uint8)
    parts["slot"] = (slot, gui("nine_slice", (4, 4, 4, 4), (36, 36), "icon = 16\n"))
    parts["slot_highlight"] = (hl, gui("nine_slice", (4, 4, 4, 4), (36, 36)))
    # The divider: 96 x 6, its end studs kept, only the middle stretches.
    div = even(cut(FRAMES, (100, 650, 480, 30)), (6, 0, 6, 0))
    parts["divider"] = (div, gui("nine_slice", (6, 0, 6, 0), (96, 6)))
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
