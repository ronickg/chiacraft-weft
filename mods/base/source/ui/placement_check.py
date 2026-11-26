"""Measures where items, stack counts and button labels sit in a game screenshot, against the UI
guide's placement rules (Ronald, 2026-11-04: "items need to be positioned correctly, text as
well"). Works on shots at an even GUI scale, where a UI pixel is whole screen pixels.

    ~/.venvs/blocky/bin/python mods/base/source/ui/placement_check.py SHOT.png [--scale 2] [--out marked.png]

Finds every slot and button by their textures (`assets/textures/gui`, exact pixels), then:
- items: which item is in the slot (matched against `textures/item/*.png`) and its offset from
  the rule (the 32 px icon 2 px inside the 36 px slot, equal on all sides); 0,0 is right
- counts: the digits' gap to the icon area's right and bottom edges (rule: 2 and 2, ink to edge)
- buttons: the label's margins left/right and above the capitals/below the baseline; the rule is
  equal margins (a difference of 0, or 1 where the leftover is odd)
Prints one line per finding and a summary; `--out` writes the shot with boxes drawn on it.
"""
import argparse
import glob
import os

import numpy as np
from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
ASSETS = os.path.join(HERE, "..", "..", "assets", "textures")
TOL = 8  # per channel: the HUD comes out a few levels off the texture (blending)
TEXT = np.array([244, 234, 216])  # color.button and stack counts (#f4ead8)
TEXT_HOVER = np.array([255, 248, 236])


def tex(name, k):
    a = np.array(Image.open(os.path.join(ASSETS, "gui", name + ".png")).convert("RGBA"))
    return a.repeat(k, 0).repeat(k, 1) if k > 1 else a


def find(shot, patch, step=1):
    """Every place where `patch`'s opaque pixels equal the shot's exactly (within 3 per channel)."""
    H, W = shot.shape[:2]
    h, w = patch.shape[:2]
    m = patch[..., 3] > 250
    ys, xs = np.nonzero(m)
    y0, x0 = ys[0], xs[0]
    first = patch[y0, x0, :3].astype(int)
    cand = np.argwhere((np.abs(shot[:H - h + 1 + y0, :W - w + 1 + x0, :3].astype(int) - first) <= TOL).all(-1))
    hits = []
    pv = patch[..., :3].astype(int)
    for cy, cx in cand:
        y, x = cy - y0, cx - x0
        if y < 0 or x < 0 or y + h > H or x + w > W:
            continue
        win = shot[y:y + h, x:x + w, :3].astype(int)
        if (np.abs(win - pv)[m] <= TOL).all():
            hits.append((y, x))
    return hits


def items(k):
    out = {}
    for p in glob.glob(os.path.join(ASSETS, "item", "*.png")):
        a = np.array(Image.open(p).convert("RGBA"))
        if a.shape[:2] != (32, 32):
            continue
        out[os.path.basename(p)[:-4]] = a.repeat(k, 0).repeat(k, 1) if k > 1 else a
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("shot")
    ap.add_argument("--scale", type=int, default=2)
    ap.add_argument("--out")
    a = ap.parse_args()
    k = a.scale // 2  # screen pixels a UI pixel
    shot = np.array(Image.open(a.shot).convert("RGBA"))
    marked = Image.open(a.shot).convert("RGB")
    d = ImageDraw.Draw(marked)
    lines, bad = [], 0
    icons = items(k)
    # slots: the slot's (and the hotbar slot's) top-left 6 x 6 and its other corners; a slot is
    # its texture's size (menus 36, the HUD's hotbar 40), the icon centred in it
    for cls in ("slot", "hotbar"):
        t = tex(cls, k)
        S = t.shape[0]
        inset = (S - 32 * k) // 2
        corner = t[:6 * k, :6 * k]
        for y, x in find(shot, corner):
            if y + S > shot.shape[0] or x + S > shot.shape[1]:
                continue
            br = t[-2 * k:, -2 * k:]  # only the frame: a count may sit just inside it
            win = shot[y + S - 2 * k:y + S, x + S - 2 * k:x + S, :3].astype(int)
            mm = br[..., 3] > 250
            if not (np.abs(win - br[..., :3].astype(int))[mm] <= TOL).all():
                continue
            # what differs from the empty slot inside the frame
            inner = shot[y:y + S, x:x + S, :3].astype(int)
            diff = (np.abs(inner - t[..., :3].astype(int)).sum(-1) > 12) & (t[..., 3] > 0)
            d.rectangle([x, y, x + S - 1, y + S - 1], outline=(0, 255, 255))
            if diff.sum() < 8 * k * k:
                continue
            best = None
            for name, ic in icons.items():
                am = ic[..., 3] > 128
                for dy in range(-3 * k, 3 * k + 1, k):
                    for dx in range(-3 * k, 3 * k + 1, k):
                        oy, ox = inset + dy, inset + dx
                        if oy < 0 or ox < 0 or oy + 32 * k > S or ox + 32 * k > S:
                            continue
                        sub = inner[oy:oy + 32 * k, ox:ox + 32 * k]
                        err = np.abs(sub - ic[..., :3].astype(int)).sum(-1)[am]
                        score = (err < 30).mean()
                        if best is None or score > best[0]:
                            best = (score, name, dx // k, dy // k)
            if best and best[0] > 0.6:
                score, name, dx, dy = best
                ok = dx == 0 and dy == 0
                bad += not ok
                lines.append(f"{'ok ' if ok else 'BAD'} item  {name:<18} in {cls} at {x},{y}: offset {dx:+d},{dy:+d} UI px (match {score:.0%})")
                d.rectangle([x + inset + dx * k, y + inset + dy * k, x + inset + dx * k + 32 * k - 1, y + inset + dy * k + 32 * k - 1],
                            outline=(0, 255, 0) if ok else (255, 0, 0))
            # the count: count-colored ink in the slot's bottom right half
            # (only the digits' corner: the bottom 13 rows and the right 22 columns of the icon area,
            # and only where the dark shadow sits one pixel down-right, which item pixels lack)
            ink = (np.abs(inner - TEXT).sum(-1) < 30)
            shadow = np.zeros_like(ink)
            shadow[:-k, :-k] = np.abs(inner[k:, k:] - np.array([40, 24, 12])).sum(-1) < 40
            ink &= shadow
            ink[:S - inset - 13 * k] = False
            ink[:, :S - inset - 22 * k] = False
            if ink.sum() >= 4 * k * k:
                ys, xs = np.nonzero(ink)
                gr = (S - inset - 1 - xs.max()) // k  # ink to the icon area's right edge
                gb = (S - inset - 1 - ys.max()) // k
                ok = gr == 2 and gb == 2
                bad += not ok
                lines.append(f"{'ok ' if ok else 'BAD'} count in {cls} at {x},{y}: gap right {gr}, bottom {gb} UI px (rule 2, 2)")
                d.rectangle([x + xs.min(), y + ys.min(), x + xs.max(), y + ys.max()], outline=(255, 0, 255))
    # buttons, at any size (nine-sliced): the top-left corner, then down the left edge to the
    # bottom-left corner and along the top to the top-right one
    for cls in ("button", "button_hover", "button_quit"):
        t = tex(cls, k)
        c = 5 * k
        tl, bl, tr = t[:c, :c], t[-c:, :c], t[:c, -c:]

        def at(p, y, x):
            if y < 0 or x < 0 or y + c > shot.shape[0] or x + c > shot.shape[1]:
                return False
            m = p[..., 3] > 250
            return (np.abs(shot[y:y + c, x:x + c, :3].astype(int) - p[..., :3].astype(int))[m] <= TOL).all()

        for y, x in find(shot, tl):
            h = next((hh for hh in range(2 * c, 80 * k) if at(bl, y + hh - c, x)), None)
            w = next((ww for ww in range(2 * c, 1200 * k) if at(tr, y, x + ww - c)), None)
            if not h or not w:
                continue
            box = shot[y:y + h, x:x + w, :3].astype(int)
            col = TEXT_HOVER if cls == "button_hover" else TEXT
            ink = np.abs(box - col).sum(-1) < 40
            if ink.sum() < 6:
                continue
            # the label is the widest run of ink: an icon beside it, 10 or more pixels away, is not
            cols = np.nonzero(ink.any(0))[0]
            runs, start = [], cols[0]
            for a_, b_ in zip(cols[:-1], cols[1:]):
                if b_ - a_ > 10 * k:
                    runs.append((start, a_))
                    start = b_
            runs.append((start, cols[-1]))
            c0, c1 = max(runs, key=lambda r: r[1] - r[0])
            ink[:, :c0] = False
            ink[:, c1 + 1:] = False
            ys, xs = np.nonzero(ink)
            # the baseline: the row most of the letters stand on (descenders hang below it)
            lows = [np.nonzero(ink[:, c])[0].max() for c in range(c0, c1 + 1) if ink[:, c].any()]
            base = np.bincount(lows).argmax()
            l, r_ = xs.min() // k, (w - 1 - xs.max()) // k
            t_, b = ys.min() // k, (h - 1 - base) // k
            ok = abs(l - r_) <= 1 and abs(t_ - b) <= 1 and min(l, r_, t_, b) >= 4
            bad += not ok
            lines.append(f"{'ok ' if ok else 'BAD'} label in {cls} at {x},{y} ({w // k} x {h // k}): left {l} right {r_}, above the capitals {t_}, below the baseline {b} UI px")
            d.rectangle([x, y, x + w - 1, y + h - 1], outline=(255, 255, 0))
            d.rectangle([x + xs.min(), y + ys.min(), x + xs.max(), y + ys.max()], outline=(255, 0, 255))
    for line in lines:
        print(line)
    print(f"{len(lines)} measured, {bad} off the rules")
    if a.out:
        marked.save(a.out)


if __name__ == "__main__":
    main()
