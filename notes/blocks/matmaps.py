#!/usr/bin/env python3
"""Material maps (LabPBR 1.3 `_s` and `_n`, docs/textures.md#materials) for block textures, derived
from each texture's own picture by a few rules per block, so they always line up with the art.

    python3 notes/blocks/matmaps.py batches/matmaps.tsv [name ...]

A batch row: `name<TAB>mod<TAB>rules`, the rules `key=value` separated by spaces:
- `smooth=0.8`      LabPBR smoothness 0-1 (red)
- `f0=10` or `metal=copper|iron|gold|aluminium|chrome|lead|platinum|silver|albedo`  (green)
- `porosity=0.5`    0-1 (blue 0-64), or `sss=0.5` (blue 65-255)
- `normal=lum:2`    height from the texel's brightness (light high, dark low), as a normal of
                    this strength; `normal=flat` none
- `dark=0.3:smooth=0.1:porosity=0.8:f0=10:depth=0.4`
                    texels darker than 0.3 (luma 0-1: grout, seams, crevices) take these instead,
                    and sit `depth` lower (a groove the normal map shows)
- `light=0.8:smooth=0.95`  texels brighter than 0.8 take these
- `emit=0.6`        emission for texels with an `_e`-like brightness over 0.85 (alpha), else none
- `tile=1`          the picture wraps (its normals seam-free across block edges); default 1
Only opaque texels count (a cutout's holes get the neutral). Writes next to the texture:
`mods/<mod>/assets/textures/block/<name>_s.png` and `_n.png`.
"""
import math
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
METALS = {"iron": 230, "gold": 231, "aluminium": 232, "chrome": 233, "copper": 234, "lead": 235, "platinum": 236,
          "silver": 237, "albedo": 255}


def rules_of(text):
    base, dark, light = {}, None, None
    for part in text.split():
        k, _, v = part.partition("=")
        if k in ("dark", "light"):
            bits = v.split(":")
            over = {"at": float(bits[0])}
            for b in bits[1:]:
                kk, _, vv = b.partition("=")
                over[kk] = vv
            if k == "dark":
                dark = over
            else:
                light = over
        else:
            base[k] = v
    return base, dark, light


def green(r):
    if "metal" in r:
        return METALS[r["metal"]]
    return max(0, min(229, int(float(r.get("f0", 10)))))


def blue(r):
    if "sss" in r:
        return 65 + int(round(float(r["sss"]) * 190))
    return int(round(float(r.get("porosity", 0)) * 64))


def luma(p):
    return (0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]) / 255.0


def make(name, mod, text):
    tex = ROOT / f"mods/{mod}/assets/textures/block/{name}.png"
    img = Image.open(tex).convert("RGBA")
    w, h = img.size
    px = img.load()
    base, dark, light = rules_of(text)
    tile = base.get("tile", "1") == "1"
    nmode = base.get("normal", "flat")
    strength = float(nmode.split(":")[1]) if nmode.startswith("lum") else 0.0
    spec = Image.new("RGBA", (w, h))
    height = [[1.0] * w for _ in range(h)]
    for y in range(h):
        for x in range(w):
            p = px[x, y]
            if p[3] < 8:
                spec.putpixel((x, y), (0, 10, 0, 255))
                continue
            l = luma(p)
            r = dict(base)
            depth = 0.0
            if dark and l < dark["at"]:
                r.update({k: v for k, v in dark.items() if k != "at"})
                depth = float(dark.get("depth", 0.0))
            elif light and l > light["at"]:
                r.update({k: v for k, v in light.items() if k != "at"})
            emit = 255
            if "emit" in base and l > 0.85:
                emit = int(float(base["emit"]) * 254)
            spec.putpixel((x, y), (int(round(float(r.get("smooth", 0)) * 255)), green(r), blue(r), emit))
            height[y][x] = (l if strength > 0 else 1.0) - depth
    normal = Image.new("RGBA", (w, h))
    for y in range(h):
        for x in range(w):
            def hh(xx, yy):
                if tile:
                    return height[yy % h][xx % w]
                return height[min(max(yy, 0), h - 1)][min(max(xx, 0), w - 1)]
            # Sobel: the slope across u (x) and down v (y), as DirectX's y (down the picture).
            dx = (hh(x + 1, y - 1) + 2 * hh(x + 1, y) + hh(x + 1, y + 1)) - (hh(x - 1, y - 1) + 2 * hh(x - 1, y) + hh(x - 1, y + 1))
            dy = (hh(x - 1, y + 1) + 2 * hh(x, y + 1) + hh(x + 1, y + 1)) - (hh(x - 1, y - 1) + 2 * hh(x, y - 1) + hh(x + 1, y - 1))
            k = max(strength, 1.0 if dark and dark.get("depth") else 0.0)
            nx, ny, nz = -dx * k / 4.0, -dy * k / 4.0, 1.0
            ln = math.sqrt(nx * nx + ny * ny + nz * nz)
            nx, ny = nx / ln, ny / ln
            hgt = max(1, min(255, int(round(height[y][x] * 255))))
            normal.putpixel((x, y), (int(round((nx * 0.5 + 0.5) * 255)), int(round((ny * 0.5 + 0.5) * 255)), 255, hgt if strength > 0 else 255))
    out = tex.with_name(f"{name}_s.png")
    spec.save(out)
    normal.save(tex.with_name(f"{name}_n.png"))
    print(f"{name}: {out.relative_to(ROOT)} and _n")


def main():
    batch = Path(sys.argv[1])
    if not batch.is_absolute():
        batch = Path(__file__).resolve().parent / batch
    only = set(sys.argv[2:])
    for line in batch.read_text().splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        name, mod, text = line.split("\t", 2)
        if only and name not in only:
            continue
        make(name, mod, text)


if __name__ == "__main__":
    main()
