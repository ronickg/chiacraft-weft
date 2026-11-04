"""Status bars from a generated image of stacked bars (`extract.py --rows`): the first an empty
frame, the rest the same frame partly filled. Writes the frame, and each fill cut out of its frame
on its own: the game draws the frame, then the fill stretched to the value inside it.

    python mods/base/source/ui/bars.py image.png out/ --names bar,bar_health,bar_stamina,bar_xp --size 128 --height 14 --smooth 2
"""
import argparse
import os

import numpy as np
from PIL import Image

import extract as ex


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("image")
    ap.add_argument("out")
    ap.add_argument("--names", required=True)
    ap.add_argument("--size", type=int, required=True, help="the frame's width in GUI pixels")
    ap.add_argument("--height", type=int, required=True)
    ap.add_argument("--smooth", type=int, default=2)
    ap.add_argument("--corner", type=int, default=6)
    args = ap.parse_args()
    names = args.names.split(",")
    src = np.array(Image.open(args.image).convert("RGBA"))
    bg = ex.background(src, tight=True)
    found = ex.boxes(~bg, rows=True)
    os.makedirs(args.out, exist_ok=True)
    k = args.smooth
    x0, y0, x1, y1 = found[0]
    scale_x, scale_y = args.size * k / (x1 - x0), args.height * k / (y1 - y0)

    # the frame has no purple: the full key there, magenta-tinted edges and all
    bg_frame = ex.background(src)

    def shrink(box, w, h):
        crop = src[box[1]:box[3], box[0]:box[2]]
        keep = ~bg_frame[box[1]:box[3], box[0]:box[2]]
        rgb = ex.defringe(crop[..., :3].astype(float), keep)
        rgba = np.concatenate([rgb, keep[..., None] * 255.0], -1).clip(0, 255).astype(np.uint8)
        a = np.array(Image.fromarray(rgba, "RGBA").resize((w, h), Image.LANCZOS))
        a[..., 3] = np.where(a[..., 3] > 127, 255, 0)
        return a

    frame = shrink(found[0], args.size * k, args.height * k)
    Image.fromarray(frame).save(os.path.join(args.out, names[0] + ".png"))
    open(os.path.join(args.out, names[0] + ".gui"), "w").write(f"scaling = nine_slice\nborder = {args.corner}\nsize = {args.size} {args.height}\n")
    # the fill: what differs from the empty frame at the same place
    empty = src[y0:y1, x0:x1, :3]
    for name, (bx0, by0, bx1, by1) in zip(names[1:], found[1:]):
        crop = src[by0:by1, bx0:bx1, :3].astype(float)
        ref = np.array(Image.fromarray(empty).resize((bx1 - bx0, by1 - by0), Image.LANCZOS)).astype(float)
        diff = np.abs(crop - ref).sum(-1) > 90
        # the fill's rows: the longest run of rows partly different (a frame a pixel off makes
        # single rows that differ almost everywhere; the fill covers only part of the width)
        m = diff.mean(1)
        part = (m > 0.3) & (m < 0.9)
        best, run, start = (0, 0), 0, 0
        for y, on in enumerate(list(part) + [False]):
            if on:
                start, run = (y, 1) if run == 0 else (start, run + 1)
            elif run:
                best = max(best, (run, start))
                run = 0
        rows = np.arange(best[1], best[1] + best[0])
        band = diff[rows.min():rows.max() + 1]
        cols = np.where(band.mean(0) > 0.5)[0]
        fy, fh = by0 + rows.min(), rows.max() - rows.min() + 1
        fx, fw = bx0 + cols.min(), cols.max() - cols.min() + 1
        h_gui = max(1, round(fh * scale_y / k))
        inner_w = round((x1 - x0 - 2 * (fx - bx0)) * scale_x / k)
        # the fill as drawn (its rounded left end kept, 3 GUI pixels), stretched by the game to the value
        piece = np.array(Image.fromarray(src[fy:fy + fh, fx:fx + fw]).resize((inner_w * k, h_gui * k), Image.LANCZOS))
        piece[..., 3] = 255
        Image.fromarray(piece).save(os.path.join(args.out, name + ".png"))
        open(os.path.join(args.out, name + ".gui"), "w").write(f"scaling = nine_slice\nborder = 3 0 0 0\nsize = {inner_w} {h_gui}\n")
        print(name, f"fill {inner_w} x {h_gui} GUI px, inset {(fx - bx0) * scale_x / k:.1f} x {(fy - by0) * scale_y / k:.1f}")
    print("frame", args.size, "x", args.height)


if __name__ == "__main__":
    main()
