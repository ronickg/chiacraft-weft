"""Any generated sheet of UI parts (`notes/STYLE.md`): parts in rows, a stack of parts in one place
written with `/`, each part cut out with its background and the background's tint pulled out of its
edges, then shrunk as drawn, every part of the sheet at one scale (set by one part's size in GUI
pixels), so their sizes stay as drawn relative to each other.

    python mods/base/source/ui/sheet.py sheet.png out/ \\
        --names "track,fill,knob,knob_hover" --ref track=96 --border track=4,fill=3 [--smooth 2]

`--names`: rows separated by `;`, parts in a row by `,`, a stack of parts in one place by `/`.
`--border`: parts that stretch (nine-slice) and their border in GUI pixels; the rest are pictures.
The background is read from the corner (magenta or green); only colors near it are taken out, so a
green check mark on a green sheet stays.
"""
import argparse
import os

import numpy as np
from PIL import Image

import extract as ex


def background(src):
    """The sheet's background: its corner color, and colors close to it (and, on magenta, the
    magenta-tinted edges; no colors of the style look like that)."""
    rgb = src[..., :3].astype(float)
    key = np.median(rgb[:8, :8].reshape(-1, 3), 0)
    bg = (src[..., 3] < 128) | (np.abs(rgb - key).sum(-1) < 90)
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    if key[0] > 180 and key[2] > 180 and key[1] < 90:
        bg |= (r - g > 50) & (b - g > 50) & (np.abs(r - b) < 70)
    return bg


def parts(fg, layout):
    """Boxes in the order of `layout` (rows of places, each place a stack of names)."""
    rows = ex.boxes(fg, rows=True)
    if len(rows) != len(layout):
        raise SystemExit(f"found {len(rows)} rows, named {len(layout)}")
    out = []
    for (x0, y0, x1, y1), names in zip(rows, layout):
        cols = ex.boxes(fg[y0:y1], rows=False)
        if len(cols) != len(names):
            raise SystemExit(f"row {names}: found {len(cols)} parts")
        for (cx0, _, cx1, _), stack in zip(cols, names):
            sub = fg[y0:y1, cx0:cx1]
            pieces = ex.boxes(sub, rows=True) if len(stack) > 1 else None
            if pieces is not None and len(pieces) != len(stack):
                raise SystemExit(f"{stack}: found {len(pieces)} stacked")
            for k, name in enumerate(stack):
                if pieces is None:
                    ys = np.where(sub.sum(1) > 0)[0]
                    xs = np.where(sub.sum(0) > 0)[0]
                    out.append((name, (cx0 + xs.min(), y0 + ys.min(), cx0 + xs.max() + 1, y0 + ys.max() + 1)))
                else:
                    px0, py0, px1, py1 = pieces[k]
                    out.append((name, (cx0 + px0, y0 + py0, cx0 + px1, y0 + py1)))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("image")
    ap.add_argument("out")
    ap.add_argument("--names", required=True)
    ap.add_argument("--ref", required=True, help="name=width: that part's width in GUI pixels")
    ap.add_argument("--border", default="", help="name=b,..: parts that stretch, their border")
    ap.add_argument("--smooth", type=int, default=2)
    ap.add_argument("--silhouette", action="store_true", help="each part as a faint flat shape (empty equipment slots)")
    args = ap.parse_args()
    layout = [[place.split("/") for place in row.split(",")] for row in args.names.split(";")]
    src = np.array(Image.open(args.image).convert("RGBA"))
    bg = background(src)
    found = parts(~bg, layout)
    ref_name, ref_w = args.ref.split("=")
    ref_box = dict(found)[ref_name]
    k = args.smooth
    scale = int(ref_w) * k / (ref_box[2] - ref_box[0])
    borders = dict((kv.split("=")[0], kv.split("=")[1]) for kv in args.border.split(",") if kv)
    os.makedirs(args.out, exist_ok=True)
    for name, (x0, y0, x1, y1) in found:
        keep = ~bg[y0:y1, x0:x1]
        rgb = ex.defringe(src[y0:y1, x0:x1, :3].astype(float), keep)
        rgba = np.concatenate([rgb, keep[..., None] * 255.0], -1).clip(0, 255).astype(np.uint8)
        w_gui = max(1, round((x1 - x0) * scale / k))
        h_gui = max(1, round((y1 - y0) * scale / k))
        im = np.array(Image.fromarray(rgba, "RGBA").resize((w_gui * k, h_gui * k), Image.LANCZOS))
        im[..., 3] = np.where(im[..., 3] > 127, 255, 0)
        if args.silhouette:
            # one faded brown, its edge a little darker: shows faintly inside an empty slot
            # its own light and lines kept, in one faded brown, half seen
            a = im[..., 3] > 0
            lum = im[..., :3].astype(float).mean(-1, keepdims=True) / 255
            dark, light = np.array([46, 32, 24.0]), np.array([150, 116, 88.0])
            im[..., :3] = (dark + (light - dark) * lum).astype(np.uint8)
            im[..., 3] = np.where(a, 140, 0)
        Image.fromarray(im, "RGBA").save(os.path.join(args.out, name + ".png"))
        with open(os.path.join(args.out, name + ".gui"), "w") as f:
            if name in borders:
                f.write(f"scaling = nine_slice\nborder = {borders[name]}\nsize = {w_gui} {h_gui}\n")
            else:
                f.write(f"scaling = stretch\nsize = {w_gui} {h_gui}\n")
        print(f"{name}: {w_gui} x {h_gui}")


if __name__ == "__main__":
    main()
