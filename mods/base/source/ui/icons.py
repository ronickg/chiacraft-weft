"""Icons from a generated sheet (rows of icons on a flat background; `notes/STYLE.md`): each icon
cut out, the background and its tint pulled out of the edges, and shrunk to its on-screen size,
all at one scale (so a small icon stays smaller than a big one) and centered in a square.

    python mods/base/source/ui/icons.py sheet.png out/ --names follow,stay,... --size 24 --smooth 2
"""
import argparse
import os

import numpy as np
from PIL import Image

import extract as ex
import sheet as sheets


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("image")
    ap.add_argument("out")
    ap.add_argument("--names", required=True, help="the icons in reading order, comma separated")
    ap.add_argument("--size", type=int, default=24, help="each icon's square, GUI pixels")
    ap.add_argument("--margin", type=int, default=1, help="GUI pixels left round the biggest icon")
    ap.add_argument("--smooth", type=int, default=2)
    ap.add_argument("--silhouette", action="store_true", help="each as a faint flat shape (empty equipment slots)")
    args = ap.parse_args()
    names = args.names.split(",")
    src = np.array(Image.open(args.image).convert("RGBA"))
    bg = sheets.background(src)
    fg = ~bg
    icons = []
    for (rx0, ry0, rx1, ry1) in ex.boxes(fg, rows=True):
        row = fg[ry0:ry1]
        for (cx0, _, cx1, _) in ex.boxes(row):
            ys = np.where(fg[ry0:ry1, cx0:cx1].sum(1) > 0)[0]
            icons.append((cx0, ry0 + ys.min(), cx1, ry0 + ys.max() + 1))
    if len(icons) != len(names):
        raise SystemExit(f"found {len(icons)} icons, named {len(names)}")
    os.makedirs(args.out, exist_ok=True)
    k = args.smooth
    side = args.size * k
    biggest = max(max(x1 - x0, y1 - y0) for x0, y0, x1, y1 in icons)
    scale = (args.size - 2 * args.margin) * k / biggest
    sheet = []
    for name, (x0, y0, x1, y1) in zip(names, icons):
        crop = src[y0:y1, x0:x1]
        keep = fg[y0:y1, x0:x1]
        rgb = ex.defringe(crop[..., :3].astype(float), keep)
        rgba = np.concatenate([rgb, keep[..., None] * 255.0], -1).clip(0, 255).astype(np.uint8)
        w, h = max(1, round((x1 - x0) * scale)), max(1, round((y1 - y0) * scale))
        im = np.array(Image.fromarray(rgba, "RGBA").resize((w, h), Image.LANCZOS))
        im[..., 3] = np.where(im[..., 3] > 127, 255, 0)
        if args.silhouette:
            # its own light and lines kept, in one faded brown, half seen
            a = im[..., 3] > 0
            lum = im[..., :3].astype(float).mean(-1, keepdims=True) / 255
            dark, light = np.array([46, 32, 24.0]), np.array([150, 116, 88.0])
            im[..., :3] = (dark + (light - dark) * lum).astype(np.uint8)
            im[..., 3] = np.where(a, 140, 0)
        out = Image.new("RGBA", (side, side))
        out.alpha_composite(Image.fromarray(im), ((side - w) // 2, (side - h) // 2))
        out.save(os.path.join(args.out, name + ".png"))
        open(os.path.join(args.out, name + ".gui"), "w").write(f"scaling = stretch\nsize = {args.size} {args.size}\n")
        sheet.append(out)
    print(f"{len(sheet)} icons, {side} x {side} image pixels for {args.size} x {args.size} GUI pixels, to {args.out}")


if __name__ == "__main__":
    main()
