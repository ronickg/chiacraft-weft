"""The crafting parts test (Ronald approved the parts system, 2026-11-04): an axe (head, haft,
binding) and a sword (blade, guard, grip, pommel) built from parts in six materials, beside the
approved full icons, plus one alternate head shape (a bearded axe, his choice 4B).

- Part maps are cut from the approved iron axe and iron sword (seg.py): each pixel a step 0-7:
  0/1 the outline on the shadow/lit side, 2-6 the fill fitted to the part's own brightness, 7 the
  highlight.
- Material ramps (8 colors, dark to light) come from approved icons of one material (ingots, the
  diamond, the stick, the tools' own fittings); bronze and cord, new, from the item guide's sheet.
- An item is its part layers, back to front, each painted with its material's ramp.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/parts.py   # sheets/parts_test.png
"""
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import seg  # noqa: E402

sys.path.insert(0, os.path.join(HERE, "..", "..", "..", ".claude", "skills", "game-art", "scripts"))
import art  # noqa: E402

ITEM = seg.ITEM
BINS = [0, .08, .18, .34, .50, .66, .82, .95, 1.0]  # where each of the 8 steps sits in a material's brightness


def lum(rgb):
    rgb = np.asarray(rgb, float)
    return rgb[..., 0] * .299 + rgb[..., 1] * .587 + rgb[..., 2] * .114


# ---------------------------------------------------------------- part maps

def part_map(mask):
    """Steps 0-7 for one part's mask, from the reference icon's pixels (set by `maps`)."""
    raise NotImplementedError


def steps(a, mask):
    """The part's 8-step map: outline (0 shadow side, 1 lit side), fill 2-6 by its own brightness
    quantiles, 7 its brightest fill pixels when they are near white (a bevel, a glint)."""
    idx = np.full(mask.shape, -1, int)
    pad = np.pad(mask, 1)
    up, down = pad[:-2, 1:-1], pad[2:, 1:-1]
    left, right = pad[1:-1, :-2], pad[1:-1, 2:]
    edge = mask & ~(up & down & left & right)
    lit = edge & (~up | ~left)
    idx[edge] = 0
    idx[lit] = 1
    fill = mask & ~edge
    L = lum(a[..., :3])
    if fill.any():
        q = np.quantile(L[fill], [.2, .4, .6, .8])
        f = 2 + np.searchsorted(q, L[fill])
        idx[fill] = f
        hi = fill & (L >= max(np.quantile(L[fill], .93), 200))
        idx[hi] = 7
    return idx


def maps(icon, kind):
    """{part: (mask, steps)} cut from an approved icon."""
    a = seg.load(icon)
    m, lab = seg.label(a)
    metal = m & (lab == "metal")
    gold = m & (lab == "gold")
    wrap = m & (lab == "wrap")
    if kind == "axe":
        parts = {"haft": wrap, "head": metal, "binding": gold}
    else:
        # the gold splits into the guard (by the blade) and the pommel (the handle's end)
        ys, xs = np.where(metal)
        by, bx = ys.mean(), xs.mean()
        gy, gx = np.where(gold)
        d = (gy - by) ** 2 + (gx - bx) ** 2
        cut = np.median(d)
        guard = np.zeros_like(gold)
        pommel = np.zeros_like(gold)
        near = d <= np.quantile(d, .55)
        guard[gy[near], gx[near]] = True
        pommel[gy[~near], gx[~near]] = True
        parts = {"grip": wrap, "pommel": pommel, "blade": metal, "guard": guard}
    return a, {k: (v, steps(a, v)) for k, v in parts.items()}


# ---------------------------------------------------------------- material ramps

def ramp_from(pixels):
    """8 colors, dark to light, from a material's pixels: the mean of each brightness band."""
    px = np.asarray(pixels, float)
    order = np.argsort(lum(px))
    px = px[order]
    n = len(px)
    out = []
    for k in range(8):
        lo, hi = int(BINS[k] * n), max(int(BINS[k + 1] * n), int(BINS[k] * n) + 1)
        out.append(px[lo:hi].mean(0))
    return np.array(out)


def icon_pixels(name, family=None):
    a = seg.load(name)
    m, lab = seg.label(a)
    sel = m if family is None else (m & (lab == family))
    return a[sel][:, :3]


def reds(px):
    """Only the red-brown leather (drops gold-band pixels that sit in the wrap)."""
    import colorsys
    keep = [p for p in px if not (30 <= colorsys.rgb_to_hsv(*(np.asarray(p) / 255))[0] * 360 <= 70)]
    return np.array(keep)


def hexes(*h):
    return np.array([[int(x[i:i + 2], 16) for i in (1, 3, 5)] for x in h], float)


def ramps():
    r = {
        "iron": ramp_from(icon_pixels("iron_ingot")),
        "gold": ramp_from(icon_pixels("gold_ingot")),
        "diamond": ramp_from(icon_pixels("diamond")),
        "stone": ramp_from(icon_pixels("stone_axe", "metal")),
        "wood": ramp_from(icon_pixels("stick")),
        "leather": ramp_from(reds(icon_pixels("iron_axe", "wrap"))),
        "gold fittings": ramp_from(icon_pixels("iron_axe", "gold")),
        # new materials: from the item guide's material sheet (no approved icon yet)
        "bronze": hexes("#4a2a10", "#6a3e18", "#8c5624", "#a86e32", "#c48a44", "#d8a458", "#ecc47a", "#f8e2a8"),
        "cord": hexes("#584e39", "#776a53", "#a59573", "#bcae8c", "#cfc2a2", "#d9ccad", "#e6dcc2", "#f2ebd9"),
    }
    return r


# ---------------------------------------------------------------- composing

def compose(layers, order, mats, rmp):
    """Paint each part's steps with its material's ramp, back to front."""
    out = np.zeros((32, 32, 4), np.uint8)
    for part in order:
        mask, idx = layers[part]
        ramp = rmp[mats[part]]
        sel = mask & (idx >= 0)
        out[sel, :3] = ramp[idx[sel]].clip(0, 255).astype(np.uint8)
        out[sel, 3] = 255
    return Image.fromarray(out)


TIERS = [  # (label, head or blade, haft or grip, binding/guard/pommel, approved icon prefix)
    ("wood", "wood", "wood", "cord", "wood"),
    ("stone", "stone", "wood", "cord", "stone"),
    ("iron", "iron", "leather", "gold fittings", "iron"),
    ("gold", "gold", "leather", "gold fittings", "gold"),
    ("diamond", "diamond", "leather", "gold fittings", "diamond"),
    ("bronze (new)", "bronze", "leather", "gold fittings", None),
]


def bearded_head(axe_layers):
    """The bearded head from the generated bearded axe, moved so its collar sits where the iron
    axe's does (the socket), to go on the iron axe's haft and binding."""
    p = os.path.join(HERE, "out", "bearded_iron_axe_px.png")
    a = np.array(Image.open(p).convert("RGBA"))
    m, lab = seg.label(a)
    head = m & (lab == "metal")
    gold = m & (lab == "gold")
    # socket: the collar's centre in each
    gy, gx = np.where(axe_layers["binding"][0])
    by, bx = np.where(gold)
    dy, dx = int(round(gy.mean() - by.mean())), int(round(gx.mean() - bx.mean()))
    moved_a = np.roll(np.roll(a, dy, 0), dx, 1)
    moved = np.roll(np.roll(head, dy, 0), dx, 1)
    return a, (moved, steps(moved_a, moved)), (dy, dx)


# ---------------------------------------------------------------- the sheet

def main():
    rmp = ramps()
    axe_src, axe = maps("iron_axe", "axe")
    sw_src, sword = maps("iron_sword", "sword")
    have_beard = os.path.exists(os.path.join(HERE, "out", "bearded_iron_axe_px.png"))
    if have_beard:
        beard_src, beard_head, shift = bearded_head(axe)
        beard = dict(axe, head=beard_head)

    S, pad = 96, 12
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    cols = 7
    W = max(pad + cols * (S + pad) + 160, 1140)
    rows_h = [40, S + 40, 30, S + 30, S + 40, 30, S + 30, S + 40, 30, S + 40, 30, 110]
    im = Image.new("RGB", (W, sum(rows_h) + 40), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)
    y = 12
    d.text((pad, y), "Crafting parts test: tools built from part maps and material ramps, beside the approved full icons",
           fill=(0xf5, 0xe4, 0xbc), font=big)
    y += 40

    def tile(img, x, y, label=None, scale=S):
        d.rectangle([x, y, x + scale - 1, y + scale - 1], fill=(0x35, 0x20, 0x15), outline=(0x68, 0x3e, 0x26))
        if img is not None:
            t = img.resize((scale - 8, scale - 8), Image.NEAREST)
            im.paste(t, (x + 4, y + 4), t)
        else:
            d.text((x + 18, y + scale // 2 - 8), "none", fill=(0xc2, 0xac, 0x88), font=font)
        if label:
            d.text((x, y + scale + 3), label, fill=(0xc2, 0xac, 0x88), font=font)

    def map_img(mask, idx):
        g = np.zeros((32, 32, 4), np.uint8)
        sel = mask & (idx >= 0)
        v = (40 + idx[sel] * 30).clip(0, 255).astype(np.uint8)
        g[sel, 0] = v; g[sel, 1] = v; g[sel, 2] = v; g[sel, 3] = 255
        return Image.fromarray(g)

    # the part maps
    x = pad
    for name, (mask, idx) in list(axe.items()) + list(sword.items()) + ([("bearded head", beard_head)] if have_beard else []):
        tile(map_img(mask, idx), x, y, name)
        x += S + pad
    d.text((x + 6, y + 10), "part maps: grey =\nthe 8 steps\n(0-1 outline,\n2-6 fill, 7 highlight)", fill=(0xc2, 0xac, 0x88), font=font)
    y += S + 40

    def tier_rows(title, layers, order, part_names, approved_kind):
        nonlocal y
        d.text((pad, y), title, fill=(0xff, 0xbe, 0x45), font=big)
        y += 30
        x = pad
        for label, a_mat, h_mat, b_mat, pre in TIERS:
            mats = dict(zip(part_names, (a_mat, h_mat, b_mat, b_mat)))
            tile(compose(layers, order, mats, rmp), x, y, f"{label}: parts")
            x += S + pad
        d.text((x + 6, y + 30), "built from parts", fill=(0xc2, 0xac, 0x88), font=font)
        y += S + 30
        if approved_kind:
            x = pad
            for label, *_ , pre in TIERS:
                icon = None
                if pre:
                    p = os.path.join(ITEM, f"{pre}_{approved_kind}.png")
                    icon = Image.open(p).convert("RGBA") if os.path.exists(p) else None
                tile(icon, x, y, f"{label}: approved" if pre else "no icon yet")
                x += S + pad
            d.text((x + 6, y + 30), "approved full icon", fill=(0xc2, 0xac, 0x88), font=font)
            y += S + 40

    tier_rows("Axe: head, haft, binding", axe, ["haft", "head", "binding"], ["head", "haft", "binding"], "axe")
    tier_rows("Sword: blade, guard, grip, pommel", sword, ["grip", "pommel", "blade", "guard"],
              ["blade", "grip", "guard", "pommel"], "sword")
    if have_beard:
        tier_rows("Alternate head (4B): a bearded axe head on the same haft and binding", beard,
                  ["haft", "head", "binding"], ["head", "haft", "binding"], None)

    # the ramps
    d.text((pad, y), "Material ramps (8 steps, dark to light), from approved icons; bronze and cord from the item guide",
           fill=(0xff, 0xbe, 0x45), font=big)
    y += 30
    x = pad
    for name, r in rmp.items():
        for k, c in enumerate(r):
            d.rectangle([x + k * 12, y, x + k * 12 + 11, y + 22], fill=tuple(int(v) for v in c))
        d.text((x, y + 26), name, fill=(0xc2, 0xac, 0x88), font=font)
        x += 8 * 12 + 18
        if x > W - 120:
            x = pad
            y += 52
    os.makedirs(os.path.join(HERE, "sheets"), exist_ok=True)
    out = os.path.join(HERE, "sheets", "parts_test.png")
    im = im.crop((0, 0, W, y + 60))
    im.save(out)
    art.archive(out, "sheet_parts_test", "sheets")
    print(out, im.size, "bearded:" if have_beard else "", shift if have_beard else "")


if __name__ == "__main__":
    main()
