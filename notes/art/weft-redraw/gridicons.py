"""icons.py's cut, with icons found per grid cell (an icon of several pieces, a broken sword, stays
one icon): gridicons.py sheet.png out/ COLS ROWS name,name,... [--scale-from other_sheet_biggest]"""
import os, sys
import numpy as np
from PIL import Image
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../../mods/base/source/ui"))
import extract as ex, sheet as sheets
img, out, cols, rows, names = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), sys.argv[5].split(",")
src = np.array(Image.open(img).convert("RGBA")); fg = ~sheets.background(src)
H, W = fg.shape; icons = []
for r in range(rows):
    for c in range(cols):
        y0, y1, x0, x1 = r * H // rows, (r + 1) * H // rows, c * W // cols, (c + 1) * W // cols
        ys, xs = np.where(fg[y0:y1, x0:x1])
        if len(ys) < 50: continue
        icons.append((x0 + xs.min(), y0 + ys.min(), x0 + xs.max() + 1, y0 + ys.max() + 1))
assert len(icons) == len(names), (len(icons), len(names))
size, k, margin = int(os.environ.get('SIZE', 24)), 2, int(os.environ.get('MARGIN', 1)); side = size * k
biggest = float(os.environ.get("BIGGEST") or max(max(x1 - x0, y1 - y0) for x0, y0, x1, y1 in icons))
scale = (size - 2 * margin) * k / biggest
print("biggest", biggest)
os.makedirs(out, exist_ok=True)
for name, (x0, y0, x1, y1) in zip(names, icons):
    crop = src[y0:y1, x0:x1]; keep = fg[y0:y1, x0:x1]
    rgb = ex.defringe(crop[..., :3].astype(float), keep)
    rgba = np.concatenate([rgb, keep[..., None] * 255.0], -1).clip(0, 255).astype(np.uint8)
    w, h = max(1, round((x1 - x0) * scale)), max(1, round((y1 - y0) * scale))
    im = np.array(Image.fromarray(rgba, "RGBA").resize((w, h), Image.LANCZOS))
    im[..., 3] = np.where(im[..., 3] > 127, 255, 0)
    if os.environ.get('SILHOUETTE'):  # icons.py --silhouette: one faded brown, half seen
        a = im[..., 3] > 0
        lum = im[..., :3].astype(float).mean(-1, keepdims=True) / 255
        dark, light = np.array([46, 32, 24.0]), np.array([150, 116, 88.0])
        im[..., :3] = (dark + (light - dark) * lum).astype(np.uint8)
        im[..., 3] = np.where(a, 140, 0)
    o = Image.new("RGBA", (side, side)); o.alpha_composite(Image.fromarray(im), ((side - w) // 2, (side - h) // 2))
    o.save(os.path.join(out, name + ".png"))
