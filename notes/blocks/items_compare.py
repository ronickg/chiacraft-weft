"""Compare our item icons with vanilla 1.21.4's and Faithful 32x's (sparse checkouts outside the repo,
read as images only): size on the grid (bounding box and coverage), outline (edge pixels' brightness
against the inside), light direction (brightness falling from top-left to bottom-right), luma and
saturation. Flags an icon outside the two packs' range widened by the tolerances, or lit from the
other side. Writes items_compare.csv and a sheet of the flagged ones (ours | vanilla | Faithful).

    python3 notes/blocks/items_compare.py <worktree> <packs dir> <out dir>
"""
import csv
import glob
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

W = np.array([0.2126, 0.7152, 0.0722])
TOL = dict(box=0.15, cover=0.12, outline=0.15, luma=0.06, sat=0.12)
# our item name -> vanilla's, where they differ
MANUAL = {"raw_beef": "beef", "seeds": "wheat_seeds", "reeds": "sugar_cane", "slime": "slime_ball",
          "fish": "cod", "raw_fish": "cod", "cooked_fish": "cooked_cod", "wool": "white_wool"}


def load(p):
    a = np.asarray(Image.open(p).convert("RGBA")).astype(float) / 255
    if a.shape[0] > a.shape[1]:
        a = a[: a.shape[1]]
    return a


LEATHER = np.array([0xA0, 0x65, 0x40]) / 255  # vanilla's undyed leather armour tint


def tinted(a, name):
    if name.startswith("leather_") and name != "leather":
        b = a.copy()
        b[..., :3] *= LEATHER
        return b
    return a


def measure(a):
    n = a.shape[0]
    s = a[..., 3] > 0.5
    if s.sum() < 4:
        return None
    ys, xs = np.nonzero(s)
    box = max(xs.max() - xs.min() + 1, ys.max() - ys.min() + 1) / n
    cover = s.mean()
    L = a[..., :3] @ W
    pad = np.pad(s, 1)
    inner = pad[:-2, 1:-1] & pad[2:, 1:-1] & pad[1:-1, :-2] & pad[1:-1, 2:]
    edge = s & ~inner
    core = s & inner
    outline = L[edge].mean() / max(L[core].mean(), 1e-3) if core.sum() > 3 else 1.0
    # light: brightness against position along the top-left -> bottom-right diagonal (negative: lit from top-left)
    d = ((xs + ys) / (2 * n))
    Lv = L[s]
    light = float(np.polyfit(d, Lv, 1)[0]) if d.std() > 0 else 0.0
    c = a[..., :3][s]
    mx, mn = c.max(1), c.min(1)
    sat = float(((mx - mn) / np.maximum(mx, 1e-6)).mean())
    return dict(box=round(float(box), 3), cover=round(float(cover), 3), outline=round(float(outline), 3), light=round(light, 3),
                edge=round(float(L[edge].mean()), 3),
                luma=round(float(Lv.mean()), 3), sat=round(sat, 3))


def main(ours, packs, out):
    refs = {"vanilla": f"{packs}/vanilla/assets/minecraft/textures/item", "faithful32": f"{packs}/faithful32/assets/minecraft/textures/item"}
    rows = []
    for mod in ("base", "animals"):
        for p in sorted(glob.glob(f"{ours}/mods/{mod}/assets/textures/item/*.png")):
            n = os.path.basename(p)[:-4]
            v = MANUAL.get(n, n)
            rp = {k: f"{d}/{v}.png" for k, d in refs.items() if os.path.exists(f"{d}/{v}.png")}
            if "vanilla" not in rp:
                continue
            m = measure(load(p))
            rm = {k: measure(tinted(load(q), v)) for k, q in rp.items()}
            if m is None or any(r is None for r in rm.values()):
                continue
            flags = []
            for q in ("box", "cover", "outline", "luma", "sat"):
                lo, hi = min(r[q] for r in rm.values()), max(r[q] for r in rm.values())
                if m[q] < lo - TOL[q] or m[q] > hi + TOL[q]:
                    flags.append(f"{q} {m[q]:.2f} vs {lo:.2f}-{hi:.2f}")
            if all(r["light"] < -0.02 for r in rm.values()) and m["light"] > 0.02:
                flags.append(f"lit from the bottom right ({m['light']:+.2f}; packs {rm['vanilla']['light']:+.2f})")
            row = dict(mod=mod, item=n, vanilla=v, flags="; ".join(flags), n_flags=len(flags), **{f"ours_{k}": x for k, x in m.items()})
            for k, r in rm.items():
                row.update({f"{k}_{q}": x for q, x in r.items()})
            rows.append(row)
    os.makedirs(out, exist_ok=True)
    keys = []
    for r in rows:
        keys += [k for k in r if k not in keys]
    with open(f"{out}/items_compare.csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=keys)
        w.writeheader()
        w.writerows(rows)
    flagged = sorted([r for r in rows if r["flags"]], key=lambda r: -r["n_flags"])
    print(f"{len(rows)} of our items have a vanilla twin; {len(flagged)} flagged")
    for r in flagged:
        print(f"  {r['mod']}:{r['item']:24s} {r['flags']}")
    sheet(flagged, ours, refs, out)


def sheet(rows, ours, refs, out):
    F = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 17)
    S = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 13)
    cols, cw, ch = 2, 640, 150
    im = Image.new("RGB", (cols * cw + 20, 60 + -(-len(rows) // cols) * ch), (42, 32, 26))
    d = ImageDraw.Draw(im)
    d.text((12, 12), f"Our item icons against vanilla 1.21.4 and Faithful 32x: {len(rows)} flagged (ours | vanilla | Faithful, in a slot)", font=F, fill=(245, 228, 188))

    def slot(p):
        t = Image.open(p).convert("RGBA")
        if t.height > t.width:
            t = t.crop((0, 0, t.width, t.width))
        t = t.resize((96, 96), Image.NEAREST)
        g = Image.new("RGBA", (104, 104), (112, 74, 42, 255))
        ImageDraw.Draw(g).rectangle((3, 3, 100, 100), fill=(53, 32, 21, 255))
        g.alpha_composite(t, (4, 4))
        return g.convert("RGB")
    for i, r in enumerate(rows):
        x, y = 12 + (i % cols) * cw, 46 + (i // cols) * ch
        for j, p in enumerate([f"{ours}/mods/{r['mod']}/assets/textures/item/{r['item']}.png"] + [f"{dd}/{r['vanilla']}.png" for dd in refs.values()]):
            if os.path.exists(p):
                im.paste(slot(p), (x + j * 110, y))
        d.text((x + 340, y), f"{r['mod']}:{r['item']}", font=F, fill=(255, 190, 69))
        for k, f in enumerate(r["flags"].split("; ")):
            d.text((x + 340, y + 24 + k * 17), f, font=S, fill=(245, 228, 188))
    im.save(f"{out}/items_compare_sheet.png")
    print(f"{out}/items_compare_sheet.png", im.size)


if __name__ == "__main__":
    main(*sys.argv[1:4])
