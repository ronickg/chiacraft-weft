"""The axe head fixes (Ronald, 11-04: "axe heads are kinda bad atm if we can fix them pls"): each
material's head is a hand-painted override, cut from its own drawn axe (the approved icons, or new
ones for flint and bronze, and bearded ones for gold, diamond and bronze) and set on the iron axe's
socket; crude heads (wood, stone, flint) on a plain wooden haft, metal ones on the wrapped haft with
gold fittings. For a material with no drawn head yet, the map fallback gets 10 fill steps instead
of 5 (shown as steel). The sheet puts the approved icon, the first try and the fix side by side.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/axefix.py   # sheets/axe_heads_fixed.png
"""
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import heads  # noqa: E402
import parts  # noqa: E402
import seg  # noqa: E402

OUT = os.path.join(HERE, "out")


def drawn(name):
    """A drawn whole axe: an approved game icon, or one made for this test (out/)."""
    p = os.path.join(seg.ITEM, name + ".png")
    if not os.path.exists(p):
        p = os.path.join(OUT, name + "_px.png")
    return np.array(Image.open(p).convert("RGBA")) if os.path.exists(p) else None


PLAIN = [  # (label, drawn axe for the head, crude?)
    ("wood", "wood_axe", True), ("stone", "stone_axe", True), ("flint (new)", "flint_axe", True),
    ("iron", "iron_axe", False), ("gold", "gold_axe", False), ("diamond", "diamond_axe", False),
    ("bronze (new)", "bronze_axe", False),
]
BEARDED = [("iron", "bearded_iron_axe"), ("gold", "bearded_gold_axe"), ("diamond", "bearded_diamond_axe"),
           ("bronze", "bearded_bronze_axe")]
OLD_MATS = {"wood": ("wood", "wood", "cord"), "stone": ("stone", "wood", "cord"), "iron": ("iron", "leather", "gold fittings"),
            "gold": ("gold", "leather", "gold fittings"), "diamond": ("diamond", "leather", "gold fittings"),
            "bronze": ("bronze", "leather", "gold fittings")}


def steps_n(a, mask, nfill):
    """parts.steps with `nfill` fill steps: 0/1 outline, 2..nfill+1 fill, nfill+2 highlight."""
    idx = np.full(mask.shape, -1, int)
    pad = np.pad(mask, 1)
    up, down, left, right = pad[:-2, 1:-1], pad[2:, 1:-1], pad[1:-1, :-2], pad[1:-1, 2:]
    edge = mask & ~(up & down & left & right)
    idx[edge] = 0
    idx[edge & (~up | ~left)] = 1
    fill = mask & ~edge
    L = parts.lum(a[..., :3])
    if fill.any():
        q = np.quantile(L[fill], np.linspace(0, 1, nfill + 1)[1:-1])
        idx[fill] = 2 + np.searchsorted(q, L[fill])
        idx[fill & (L >= max(np.quantile(L[fill], .95), 200))] = nfill + 2
    return idx


def ramp_n(base, nfill):
    """A material's 8-step ramp stretched to 2 outline + nfill fill + 1 highlight steps."""
    fill = base[2:7]
    xs = np.linspace(0, len(fill) - 1, nfill)
    f = np.array([fill[int(np.floor(x))] * (1 - (x % 1)) + fill[min(int(np.floor(x)) + 1, len(fill) - 1)] * (x % 1)
                  for x in xs])
    return np.vstack([base[:2], f, base[7:8]])


def main():
    rmp = parts.ramps()
    iron = seg.load("iron_axe")
    _, axe = parts.maps("iron_axe", "axe")
    _, iron_socket, _ = heads.cut(iron)
    # the shared socket: the average of every drawn head's base, so the big heads don't clip
    bases = [heads.cut(drawn(n))[1] for _, n, _ in PLAIN + [(l, n, False) for l, n in BEARDED] if drawn(n) is not None]
    socket = np.mean(bases, 0)
    sdx, sdy = int(round(socket[0] - iron_socket[0])), int(round(socket[1] - iron_socket[1]))
    haft_mask, haft_idx = heads.shift(axe["haft"][0], sdy, sdx), heads.shift(axe["haft"][1] + 1, sdy, sdx) - 1
    bind_mask, bind_idx = heads.shift(axe["binding"][0], sdy, sdx), heads.shift(axe["binding"][1] + 1, sdy, sdx) - 1
    # a plain wooden haft, its own part shape, cut from the approved wooden axe (no wrap bands)
    wa = drawn("wood_axe")
    wh, wbase, _ = heads.cut(wa)
    plain = (wa[..., 3] > 0) & ~wh
    (wa2, plain2), _ = heads.on_socket(wa, plain, wbase, socket)
    head_mask_iron, head_idx_iron = axe["head"]

    def layer(img, a, mask):
        img[mask] = a[mask]

    def paint(img, mask, idx, ramp):
        sel = mask & (idx >= 0)
        img[sel, :3] = ramp[idx[sel]].clip(0, 255).astype(np.uint8)
        img[sel, 3] = 255

    def base_axe(crude):
        img = np.zeros((32, 32, 4), np.uint8)
        if crude:
            layer(img, wa2, plain2)  # the plain wooden haft
        else:
            paint(img, haft_mask, haft_idx, rmp["leather"])
            paint(img, bind_mask, bind_idx, rmp["gold fittings"])
        return img

    def fixed(name, crude):
        a = drawn(name)
        if a is None:
            return None
        h, base, _ = heads.cut(a)
        (a2, h2), _ = heads.on_socket(a, h, base, socket)
        img = base_axe(crude)
        layer(img, a2, h2)
        return Image.fromarray(img)

    def old(label, head_layers=axe):
        key = label.split(" ")[0]
        if key not in OLD_MATS:
            return None
        h, hf, b = OLD_MATS[key]
        return parts.compose(head_layers, ["haft", "head", "binding"], {"head": h, "haft": hf, "binding": b}, rmp)

    # the bearded first try: its map from the bearded iron drawing, on the iron socket
    beard_iron = drawn("bearded_iron_axe")
    bh, bbase, _ = heads.cut(beard_iron)
    (ba2, bh2), _ = heads.on_socket(beard_iron, bh, bbase, socket)
    beard_layers = dict(axe, head=(bh2, parts.steps(ba2, bh2)))

    S, pad = 96, 12
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    W = pad + 7 * (S + pad) + 20
    im = Image.new("RGB", (W, 1400), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)

    def tile(img, x, y, label):
        d.rectangle([x, y, x + S - 1, y + S - 1], fill=(0x35, 0x20, 0x15), outline=(0x68, 0x3e, 0x26))
        if img is not None:
            t = img.resize((S - 8, S - 8), Image.NEAREST)
            im.paste(t, (x + 4, y + 4), t)
        else:
            d.text((x + 20, y + S // 2 - 8), "none", fill=(0xc2, 0xac, 0x88), font=font)
        d.text((x, y + S + 3), label, fill=(0xc2, 0xac, 0x88), font=font)

    y = 12
    d.text((pad, y), "Axe heads, fixed: each row the approved icon, the first try and the fix", fill=(0xf5, 0xe4, 0xbc), font=big)
    y += 36

    def row(title, items):
        nonlocal y
        d.text((pad, y), title, fill=(0xff, 0xbe, 0x45), font=big)
        y += 30
        for k, (lbl, img) in enumerate(items):
            tile(img, pad + k * (S + pad), y, lbl)
        y += S + 34

    # plain heads: three rows (approved, first try, fixed)
    row("Approved full icons", [(l, Image.fromarray(drawn(n)) if drawn(n) is not None and os.path.exists(os.path.join(seg.ITEM, n + '.png')) else None)
                                for l, n, _ in PLAIN])
    row("First try: the iron head's map, recoloured", [(l, old(l)) for l, n, _ in PLAIN])
    row("Fixed: each material's own drawn head on the shared socket", [(l, fixed(n, c)) for l, n, c in PLAIN])
    row("Bearded (4B), first try: one map, recoloured", [(l, old(l, beard_layers)) for l, n in BEARDED])
    row("Bearded (4B), fixed: each material's own drawn bearded head", [(l, fixed(n, False)) for l, n in BEARDED])

    # the map fallback with more steps, for materials with no drawn head yet (steel, cooler than iron)
    steel = rmp["iron"] * np.array([0.92, 0.97, 1.08])
    old_steel = parts.compose(axe, ["haft", "head", "binding"], {"head": "steel", "haft": "leather", "binding": "gold fittings"},
                              dict(rmp, steel=steel))
    ih = drawn("iron_axe")
    h, base, _ = heads.cut(ih)
    (a2, h2), _ = heads.on_socket(ih, h, base, socket)
    img = base_axe(False)
    paint(img, h2, steps_n(a2, h2, 10), ramp_n(steel, 10))
    row("Fallback for a material with no drawn head yet (steel): 5 fill steps against 10",
        [("steel, 5 steps", old_steel), ("steel, 10 steps", Image.fromarray(img))])

    out = os.path.join(HERE, "sheets", "axe_heads_fixed.png")
    im = im.crop((0, 0, W, y + 10))
    im.save(out)
    parts.art.archive(out, "sheet_axe_heads_fixed", "sheets")
    print(out, im.size)


if __name__ == "__main__":
    main()
