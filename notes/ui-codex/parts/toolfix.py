"""Part-built tools made readable through texture (Ronald, 11-04: "pieces also look weird ...
maybe it can be fixed with textures", "i do like the depth"): every piece stays, but
- each material's working part (axe head, sword blade) is its own drawn part, with more contrast;
- the joint (collar, lashing, guard) is a 2-3 px band drawn with it, not a separate sticker;
- one outline goes round the whole tool, shaded with the light, and the dark lines between parts
  soften to one shade step, so the parts read as one object with depth.
A 2-piece row (the joint painted as the handle's top) is there to compare.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/toolfix.py   # sheets/axe_textures.png, sheets/sword_textures.png
"""
import colorsys
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
HANDLE, JOINT, WORK = 1, 2, 3


def drawn(name):
    p = os.path.join(seg.ITEM, name + ".png")
    if not os.path.exists(p):
        p = os.path.join(OUT, name + "_px.png")
    return np.array(Image.open(p).convert("RGBA")) if os.path.exists(p) else None


def regions(a, kind):
    """(label map 0/handle/joint/work, base point) for a whole drawn tool, cut by position along its
    handle line. Axe: the head stands out sideways; the joint is the 3 px below it. Sword: the guard
    is the first wide band; the blade is beyond it."""
    m = a[..., 3] > 0
    head, base, (c, d) = heads.cut(a)
    t, dist = heads.frame(a, c, d)
    lab = np.zeros(m.shape, int)
    lab[m] = HANDLE
    if kind == "axe":
        t_head = t[head & (dist > 4.0)].min() if (head & (dist > 4.0)).any() else t[head].min()
        lab[head] = WORK
        lab[head & (t < t_head)] = JOINT
    else:
        t_g = t[head].min() + 3  # the guard: the first 3 px of the wide part
        lab[head] = WORK
        lab[m & (t >= t_g - 3) & (t < t_g)] = JOINT
    return lab, base


def sword_regions(a):
    """(label map, base point) for a sword, by its width along the line: the guard is the widest band
    in the lower 60% (with its neighbours at >= 60% of that width), the blade everything beyond it,
    the grip and pommel everything before. The base is where the blade leaves the guard. (regions()'
    "first wide part" put steel's guard up on the blade.)"""
    m = a[..., 3] > 0
    _, _, (c, d) = heads.cut(a)
    t, dist = heads.frame(a, c, d)
    tb = np.round(t).astype(int)
    lo, hi = tb[m].min(), tb[m].max()
    w = np.array([dist[m & (tb == k)].max() if (m & (tb == k)).any() else 0 for k in range(lo, hi + 1)])
    g = int(np.argmax(w[: int(len(w) * 0.6)]))
    a0, b0 = g, g
    while a0 > 0 and w[a0 - 1] >= w[g] * 0.6:
        a0 -= 1
    while b0 < len(w) - 1 and w[b0 + 1] >= w[g] * 0.6:
        b0 += 1
    lab = np.zeros(m.shape, int)
    lab[m] = HANDLE
    lab[m & (tb >= lo + a0) & (tb <= lo + b0)] = JOINT
    lab[m & (tb > lo + b0)] = WORK
    return lab, np.array(c) + np.array(d) * (lo + b0 + 0.5)


def shift(arr, dy, dx):
    return heads.shift(arr, dy, dx)


def socketed(a, lab, base, socket, keep):
    """Move the pixels with labels in `keep` so the base lands on the socket (nudged inside)."""
    mask = np.isin(lab, keep)
    (a2, m2), (dy, dx) = heads.on_socket(a, mask, base, socket)
    lab2 = shift(lab, dy, dx)
    lab2[~m2] = 0
    return a2, lab2


def contrast(rgb, mask, k=1.2):
    """Stretch a part's brightness round its mean (the working part reads first)."""
    out = rgb.astype(float)
    if not mask.any():
        return out
    L = parts.lum(out)
    mean = L[mask].mean()
    scale = ((L - mean) * k + mean) / np.maximum(L, 1)
    out[mask] = (out[mask] * scale[mask][:, None])
    return out.clip(0, 255)


def shade(c, f):
    """A colour at `f` of its brightness, a little more saturated (the guide's darker shade of its own hue)."""
    h, s, v = colorsys.rgb_to_hsv(*(np.asarray(c, float) / 255))
    r, g, b = colorsys.hsv_to_rgb(h, min(1, s * 1.15 + 0.05), v * f)
    return np.array([r, g, b]) * 255


def one_outline(rgba, lab):
    """One outline round the whole tool (lit side lighter), and the lines between parts softened to
    one shade step of their own part."""
    out = rgba.astype(float)
    m = lab > 0
    pad = np.pad(m, 1)
    up, down, left, right = pad[:-2, 1:-1], pad[2:, 1:-1], pad[1:-1, :-2], pad[1:-1, 2:]
    edge = m & ~(up & down & left & right)
    lit = edge & (~up | ~left)
    L = parts.lum(out[..., :3])
    med = {k: np.median(out[..., :3][(lab == k) & ~edge], 0) if ((lab == k) & ~edge).any() else None for k in (1, 2, 3)}
    medL = {k: np.median(L[(lab == k) & ~edge]) if ((lab == k) & ~edge).any() else 0 for k in (1, 2, 3)}
    lp = np.pad(lab, 1)
    nb = [lp[:-2, 1:-1], lp[2:, 1:-1], lp[1:-1, :-2], lp[1:-1, 2:]]
    for y, x in zip(*np.where(m)):
        k = lab[y, x]
        if edge[y, x]:
            out[y, x, :3] = shade(med[k] if med[k] is not None else out[y, x, :3], 0.62 if lit[y, x] else 0.42)
        elif any(n[y, x] not in (0, k) for n in nb) and L[y, x] < medL[k] * 0.72 and med[k] is not None:
            out[y, x, :3] = shade(med[k], 0.82)  # a soft seam, not a dark sticker line
    out[..., 3] = np.where(m, 255, 0)
    return Image.fromarray(out.clip(0, 255).astype(np.uint8))


def build(kind, mats, handle_metal, handle_crude):
    """Rows for one tool kind: first try, textures fixed (all pieces), 2 pieces, approved."""
    rmp = parts.ramps()
    # the shared socket: the average base of every drawn tool of this kind
    bases = []
    cut = {}
    for label, name, crude in mats:
        a = drawn(name)
        if a is not None:
            cut[name] = (a,) + regions(a, kind)
            bases.append(cut[name][2])
    socket = np.mean(bases, 0)
    hm = drawn(handle_metal); hc = drawn(handle_crude)
    lab_m, base_m = regions(hm, kind)
    lab_c, base_c = regions(hc, kind)
    hm2, hlab_m = socketed(hm, lab_m, base_m, socket, [HANDLE])
    hc2, hlab_c = socketed(hc, lab_c, base_c, socket, [HANDLE])

    def textured(name, crude, two_piece=False):
        if name not in cut:
            return None
        a, lab, base = cut[name]
        a2, lab2 = socketed(a, lab, base, socket, [JOINT, WORK])
        img = np.zeros((32, 32, 4), float)
        lab_out = np.zeros((32, 32), int)
        ha, hl = (hc2, hlab_c) if crude else (hm2, hlab_m)
        img[hl > 0] = ha[hl > 0]; lab_out[hl > 0] = HANDLE
        work = lab2 == WORK
        joint = lab2 == JOINT
        a2 = a2.astype(float)
        a2[..., :3] = contrast(a2[..., :3], work, 1.2)
        img[joint] = a2[joint]; lab_out[joint] = JOINT
        img[work] = a2[work]; lab_out[work] = WORK
        if two_piece:
            # the joint becomes the handle's top: its pixels painted with the handle's colours by brightness
            hp = ha[hl > 0][:, :3].astype(float)
            ramp = parts.ramp_from(hp)
            jL = parts.lum(img[..., :3])
            if joint.any():
                q = np.quantile(jL[joint], np.linspace(0, 1, 9)[1:-1])
                idx = np.searchsorted(q, jL[joint])
                img[joint, :3] = ramp[idx]
            lab_out[joint] = HANDLE
        return one_outline(img, lab_out)

    first = {}
    if kind == "axe":
        _, layers = parts.maps("iron_axe", "axe")
        order, names = ["haft", "head", "binding"], ["head", "haft", "binding"]
    else:
        _, layers = parts.maps("iron_sword", "sword")
        order, names = ["grip", "pommel", "blade", "guard"], ["blade", "grip", "guard", "pommel"]
    OLD = {"wood": ("wood", "wood", "cord"), "stone": ("stone", "wood", "cord"), "iron": ("iron", "leather", "gold fittings"),
           "gold": ("gold", "leather", "gold fittings"), "diamond": ("diamond", "leather", "gold fittings"),
           "bronze": ("bronze", "leather", "gold fittings")}
    rows = {"first": [], "fixed": [], "two": [], "approved": []}
    for label, name, crude in mats:
        key = label.split(" ")[0]
        if key in OLD:
            h, hf, b = OLD[key]
            rows["first"].append((label, parts.compose(layers, order, dict(zip(names, (h, hf, b, b))), rmp)))
        else:
            rows["first"].append((label, None))
        rows["fixed"].append((label, textured(name, crude)))
        rows["two"].append((label, textured(name, crude, True)))
        p = os.path.join(seg.ITEM, name + ".png")
        rows["approved"].append((label, Image.open(p).convert("RGBA") if os.path.exists(p) else None))
    return rows


def sheet(title, rows, out):
    S, pad = 96, 12
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    n = max(len(r) for r in rows.values())
    W = max(pad + n * (S + pad) + 20, 1060)
    im = Image.new("RGB", (W, 60 + 4 * (S + 70) + 10), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)
    d.text((pad, 12), title, fill=(0xf5, 0xe4, 0xbc), font=big)
    y = 48
    for key, name in (("first", "1. First try: one map recoloured, every part outlined"),
                      ("fixed", "2. Textures fixed, all pieces: drawn per material, one outline, soft seams, more contrast"),
                      ("two", "3. Two pieces, to compare: the joint painted as the handle's top"),
                      ("approved", "4. The approved full icon")):
        d.text((pad, y), name, fill=(0xff, 0xbe, 0x45), font=big)
        y += 28
        for k, (lbl, img) in enumerate(rows[key]):
            x = pad + k * (S + pad)
            d.rectangle([x, y, x + S - 1, y + S - 1], fill=(0x35, 0x20, 0x15), outline=(0x68, 0x3e, 0x26))
            if img is not None:
                t = img.resize((S - 8, S - 8), Image.NEAREST)
                im.paste(t, (x + 4, y + 4), t)
            else:
                d.text((x + 20, y + S // 2 - 8), "none", fill=(0xc2, 0xac, 0x88), font=font)
            d.text((x, y + S + 3), lbl, fill=(0xc2, 0xac, 0x88), font=font)
        y += S + 42
    im = im.crop((0, 0, W, y))
    im.save(out)
    parts.art.archive(out, "sheet_" + os.path.basename(out)[:-4], "sheets")
    print(out, im.size)


AXES = [("wood", "wood_axe", True), ("stone", "stone_axe", True), ("flint (new)", "flint_axe", True),
        ("iron", "iron_axe", False), ("gold", "gold_axe", False), ("diamond", "diamond_axe", False),
        ("bronze (new)", "bronze_axe", False)]
SWORDS = [("wood", "wood_sword", True), ("stone", "stone_sword", True), ("iron", "iron_sword", False),
          ("gold", "gold_sword", False), ("diamond", "diamond_sword", False), ("bronze (new)", "bronze_sword", False)]


def main(which=("axe", "sword")):
    if "axe" in which:
        sheet("Axe, all pieces made readable through texture (row 2), beside the first try, 2 pieces and the approved icon",
              build("axe", AXES, "iron_axe", "wood_axe"), os.path.join(HERE, "sheets", "axe_textures.png"))
    if "sword" in which:
        sheet("Sword, the same way: a drawn blade per material on a shared socket, the hilt's small parts as accents",
              build("sword", SWORDS, "iron_sword", "wood_sword"), os.path.join(HERE, "sheets", "sword_textures.png"))


if __name__ == "__main__":
    main(sys.argv[1:] or ("axe", "sword"))


# ---------------------------------------------------------------- v3: heads stay where they're drawn

def seams_only(rgba, lab, own):
    """Soften the dark lines between parts only where two different drawings meet (`own` False
    marks pixels from the handle's drawing next to the head's); the hand-painted outline stays."""
    out = rgba.astype(float)
    m = lab > 0
    L = parts.lum(out[..., :3])
    lp = np.pad(lab, 1)
    nb = [lp[:-2, 1:-1], lp[2:, 1:-1], lp[1:-1, :-2], lp[1:-1, 2:]]
    pad = np.pad(m, 1)
    edge = m & ~(pad[:-2, 1:-1] & pad[2:, 1:-1] & pad[1:-1, :-2] & pad[1:-1, 2:])
    for k in (1, 2, 3):
        inner = (lab == k) & ~edge
        if not inner.any():
            continue
        med, medL = np.median(out[..., :3][inner], 0), np.median(L[inner])
        for y, x in zip(*np.where((lab == k) & ~edge & ~own)):
            if any(n[y, x] not in (0, k) for n in nb) and L[y, x] < medL * 0.72:
                out[y, x, :3] = shade(med, 0.82)
    out[..., 3] = np.where(m, 255, 0)
    return Image.fromarray(out.clip(0, 255).astype(np.uint8))


def anchored(head_name, handle_name, kind):
    """A head (or blade) where its own drawing has it, on another drawing's handle moved so its
    socket meets the head's base. With its own handle this is the drawn icon itself."""
    a, lab, base = (drawn(head_name),) + regions(drawn(head_name), kind)
    img = np.zeros((32, 32, 4), float)
    lab_out = np.zeros((32, 32), int)
    own = np.ones((32, 32), bool)
    if handle_name == head_name:
        hl = lab == HANDLE
        img[hl] = a[hl]
    else:
        h = drawn(handle_name)
        hlab, hbase = regions(h, kind)
        dx, dy = int(round(base[0] - hbase[0])), int(round(base[1] - hbase[1]))
        h2, hl2 = shift(h, dy, dx), shift(hlab, dy, dx)
        hl = hl2 == HANDLE
        img[hl] = h2[hl]
        own[hl] = False
    lab_out[hl] = HANDLE
    for k in (JOINT, WORK):
        sel = lab == k
        img[sel] = a[sel]
        lab_out[sel] = k
        own[sel] = True
    return seams_only(img, lab_out, own) if handle_name != head_name else Image.fromarray(img.clip(0, 255).astype(np.uint8))
