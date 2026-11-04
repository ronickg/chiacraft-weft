"""Measures every label in a game shot (Ronald: "Pls be sure u get text correct position,
vertically, sizing etc should all be perfect"). Kit 2 draws exact colours and the game draws text
in the theme's exact colours (within one level), so the parts and their labels can be found by
colour alone:

- each button, tab, chip and row: a component of one of kit 2's face colours, its face the rows
  between the light edge and the shade;
- its label: the pixels in the theme's text colours inside it (icons are other colours);
- vertical: the room above the capitals (9 px) and below the baseline inside the face, which
  must be equal;
- horizontal: the room left and right of the label group (icon and text) inside the face,
  equal when centred (or a fixed left pad);
- crisp: at a GUI scale s, a UI pixel is s/2 screen pixels; every such block must be one colour,
  and text pixels must be a theme colour (no blur, no half pixels);
- clear: no text pixel within 2 UI px of an outline or frame pixel.

Usage: python3 label_check.py SHOT.png [--scale 2] [--json out.json] [--zoom out.png]"""
import argparse
import json
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kit2 as k  # noqa: E402

LANG = os.path.join(k.REPO, "mods/base/assets/lang/en_us.lang")
CAPS = 9


def theme_text_colours(lang=LANG):
    out = set()
    for line in open(lang, encoding="utf-8"):
        key, _, v = line.partition(" = ")
        v = v.strip()
        if key.startswith("color.") and not key.endswith("_shadow") and v.startswith("{#") and v.endswith("}"):
            h = v[2:-1]
            out.add(tuple(int(h[i:i + 2], 16) for i in (0, 2, 4)))
    return out


PALETTE = None  # None: kit 2's own; "A": the tones before the Valheim look (`variants.A`)


def faces():
    """Face colour -> (kind, light edge, shade): every kit 2 part with a flat face."""
    if PALETTE == "A":
        import variants
        fs, chips, bronze = variants.A["FACES"], variants.A["CHIPS"], (variants.A["BRONZE"], variants.A["BRONZE_HI"], variants.A["BRONZE_D"])
    else:
        fs, chips, bronze = k.FACES, {n: v[:3] for n, v in k.CHIPS.items()}, (k.BRONZE, k.BRONZE_HI, k.BRONZE_D)
    f = {}
    for name, (light, face, shade) in fs.items():
        f[face] = (f"button {name}", light, shade)
    for name, (light, face, shade) in chips.items():
        f.setdefault(face, (f"chip {name}", light, shade))
    f.setdefault(bronze[0], ("tab (open)", bronze[1], bronze[2]))
    return f


def ui_pixels(img, scale):
    """The shot at one UI pixel per pixel, and the share of blocks that weren't one colour."""
    n = scale // 2 if scale >= 2 else 1
    if n <= 1:
        return img, 0.0
    h, w = img.shape[0] // n * n, img.shape[1] // n * n
    b = img[:h, :w].reshape(h // n, n, w // n, n, 3)
    first = b[:, :1, :, :1]
    mixed = (np.abs(b.astype(int) - first.astype(int)).sum(-1) > 3).any(axis=(1, 3))
    return b[:, 0, :, 0], float(mixed.mean())


def near(a, colours, tol=1):
    m = np.zeros(a.shape[:2], bool)
    for c in colours:
        m |= (np.abs(a.astype(int) - np.array(c)).max(-1) <= tol)
    return m


def components(mask, min_w=8, min_h=8):
    """Bounding boxes of 4-connected regions (iterative, no scipy)."""
    h, w = mask.shape
    seen = np.zeros_like(mask)
    boxes = []
    ys, xs = np.nonzero(mask)
    for y0, x0 in zip(ys, xs):
        if seen[y0, x0]:
            continue
        stack = [(y0, x0)]
        seen[y0, x0] = True
        y1 = y2 = y0
        x1 = x2 = x0
        n = 0
        while stack:
            y, x = stack.pop()
            n += 1
            y1, y2, x1, x2 = min(y1, y), max(y2, y), min(x1, x), max(x2, x)
            for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                yy, xx = y + dy, x + dx
                if 0 <= yy < h and 0 <= xx < w and mask[yy, xx] and not seen[yy, xx]:
                    seen[yy, xx] = True
                    stack.append((yy, xx))
        if x2 - x1 + 1 >= min_w and y2 - y1 + 1 >= min_h:
            boxes.append((x1, y1, x2, y2, n))
    return boxes


def measure(path, scale=2, lang=LANG):
    raw = np.array(Image.open(path).convert("RGB"))
    a, mixed = ui_pixels(raw, scale)
    text_cols = theme_text_colours(lang)
    text = near(a, text_cols)
    found = []
    for face, (kind, light, shade) in faces().items():
        m = np.all(a == np.array(face), axis=-1)
        for x1, y1, x2, y2, n in components(m):
            # The face's own rows: between the light edge above and the shade below (the label's
            # ink lies inside the face's box, which the component may not cover where text is).
            box = a[y1:y2 + 1, x1:x2 + 1]
            # The label may stick out of the face (a low one runs into the shade): look 3 rows
            # beyond it each way; rows are counted from the face's first row.
            pad = 3
            ya, yb = max(0, y1 - pad), min(a.shape[0], y2 + 1 + pad)
            t = text[ya:yb, x1:x2 + 1]
            if not t.any():
                continue
            ty = np.nonzero(t.any(1))[0] + (ya - y1)
            tx = np.nonzero(t.any(0))[0]
            it, ib = int(ty.min()), int(ty.max())
            ink_h = ib - it + 1
            fh = y2 - y1 + 1
            if ink_h >= CAPS:
                # capitals: their 9 rows from the top of the ink; below them, descenders
                cap_top, cap_bottom = it, it + CAPS - 1
                desc = max(0, ib - cap_bottom)
            else:
                # no capitals ("+", "-"): the ink itself centres
                cap_top, cap_bottom, desc = it, ib, 0
            above, below = cap_top, fh - 1 - cap_bottom
            # horizontal: the whole label group (text and icon), in the face's middle rows (its
            # stepped corners reach 1 px into the first and last face rows)
            content = ~np.all(box == np.array(face), axis=-1)
            rows = content[max(1, cap_top):min(fh - 1, cap_bottom + 1)]
            cx = np.nonzero(rows.any(0))[0]
            left, right = (int(cx.min()), (x2 - x1) - int(cx.max())) if len(cx) else (0, 0)
            # clear: distance from text to the outline or frame
            edge_cols = [k.OUTLINE, k.BRONZE, k.BRONZE_D, k.GLOW]
            found.append(dict(kind=kind, box=[int(x1), int(y1), int(x2), int(y2)], face_rows=int(fh),
                              above=int(above), below=int(below), descender=int(desc),
                              left=left, right=right, ink=[int(tx.min()), it, int(tx.max()), ib],
                              vertical_ok=above == below or abs(above - below) == 1 and (fh - CAPS) % 2 == 1,
                              even_face=(fh - CAPS) % 2 == 0, centred=abs(left - right) <= 1,
                              descender_clear=below - desc >= 1 if desc else True))
    # Clearance: no text pixel within 2 UI px of an outline or a frame (panels, tooltips, HUD
    # plates as well as the parts above).
    if PALETTE == "A":
        import variants
        edge_cols = [variants.A["OUTLINE"], variants.A["BRONZE_D"], variants.A["BRONZE_HI"]]
    else:
        # outlines and a frame's rim lines; not the frame's own face (an open tab is that colour)
        edge_cols = [k.OUTLINE, k.BRONZE_D, k.BRONZE_HI]
    edges = near(a, edge_cols, 0)
    near_edge = edges.copy()
    for _ in range(2):
        g = near_edge.copy()
        g[1:] |= near_edge[:-1]
        g[:-1] |= near_edge[1:]
        g[:, 1:] |= near_edge[:, :-1]
        g[:, :-1] |= near_edge[:, 1:]
        near_edge = g
    close = text & near_edge
    ys, xs = np.nonzero(close)
    spots = sorted({(int(x) // 8 * 8, int(y) // 8 * 8) for y, x in zip(ys, xs)})
    return dict(shot=os.path.basename(path), scale=scale, mixed_blocks=mixed, parts=found,
                text_near_edges=len(spots), near_edge_spots=spots[:20])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("shots", nargs="+")
    ap.add_argument("--scale", type=int, default=2)
    ap.add_argument("--json")
    ap.add_argument("--palette", help='"A": the tones before the Valheim look')
    ap.add_argument("--lang", default=LANG, help="the lang file whose color.* keys the shot was drawn with")
    a = ap.parse_args()
    global PALETTE
    PALETTE = a.palette
    out = []
    for s in a.shots:
        r = measure(s, a.scale, a.lang)
        out.append(r)
        bad = [p for p in r["parts"] if not (p["vertical_ok"] and p["centred"] and p["descender_clear"])]
        print(f"{r['shot']}: {len(r['parts'])} labels measured, {len(bad)} off; blocks not one colour: {r['mixed_blocks']:.4f};"
              f" text within 2 px of a border: {r['text_near_edges']} spots {r['near_edge_spots'][:6]}")
        for p in r["parts"]:
            flag = "" if p in [] else ("  OFF" if p in bad else "")
            print(f"   {p['kind']:24s} at {p['box'][0]},{p['box'][1]} face {p['face_rows']} rows: above {p['above']} below {p['below']}"
                  f" (desc {p['descender']}), left {p['left']} right {p['right']}{'' if p['even_face'] else ' odd face'}{flag}")
    if a.json:
        json.dump(out, open(a.json, "w"), indent=1)


if __name__ == "__main__":
    main()
