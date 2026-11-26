"""Ramps for the parts crafting paints with a ramp: sword guards and pommels (metals) and lashings
(fibres and glue). Metals and stone come from our own painted heads and blades (every tool's head in
that material, pooled), so a bronze guard matches a bronze blade. Cordage and tar are drawn by hand
with the oak ramp's range. Also writes sheets/ramps.png with every ramp in deliver/textures/ramps/.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/moreramps.py
"""
import glob
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import parts  # noqa: E402
import toolfix as tf  # noqa: E402
from export2 import TOOLS  # noqa: E402

T = os.path.join(HERE, "deliver", "textures")
SHAPES = ["axe/felling_axe_head", "pickaxe/pick_head", "shovel/shovel_blade", "hoe/hoe_blade", "sword/straight_blade"]
METALS = {"copper": "copper", "bronze": "bronze", "wrought_iron": "iron", "steel": "steel", "gold": "gold", "stone": "stone"}
HAND = {
    # plant cordage: a straw-tan twisted fibre, a little greener than rawhide
    "plant_cordage": [[66, 52, 26], [92, 74, 38], [140, 116, 64], [164, 140, 82], [184, 160, 98], [200, 178, 114], [218, 200, 140], [234, 220, 168]],
    # birch tar: near-black glossy brown, with a warm glint at the top
    "birch_tar": [[14, 8, 6], [24, 14, 10], [38, 22, 14], [50, 30, 18], [64, 38, 22], [80, 50, 28], [112, 74, 42], [168, 122, 76]],
}


def step_ramp(art):
    """Step k of the ramp = the mean colour of every pixel that parts.steps puts at step k, over each
    tool's working part in this material: the outline steps (0-1) keep our warm outline, the fill
    steps (2-6) only the metal, 7 its glint. (Brightness bands mixed outline pixels into the fill.)"""
    sums, counts = np.zeros((8, 3)), np.zeros(8)
    for a, sel in parts_of(art):
        st = parts.steps(a, sel)
        for k in range(8):
            on = sel & (st == k)
            sums[k] += a[on][:, :3].sum(0)
            counts[k] += on.sum()
    for k in range(8):  # an empty step takes its neighbours' mean
        if counts[k] == 0:
            nb = [j for j in (k - 1, k + 1) if 0 <= j < 8 and counts[j]]
            sums[k], counts[k] = sum(sums[j] / counts[j] for j in nb) / len(nb), 1
    return sums / counts[:, None]


def parts_of(art):
    out = []
    for kind, name in TOOLS.items():
        a = tf.drawn(name(art))
        if a is not None:
            lab, _ = tf.regions(a, "axe")
            out.append((a.astype(float), lab == tf.WORK))
    a = tf.drawn(f"{art}_sword")
    if a is not None:
        lab, _ = tf.sword_regions(a)
        out.append((a.astype(float), lab == tf.WORK))
    return out


def head_pixels(art):
    """The metal (or stone) itself: each tool's working part only, without its collar, lashing or
    handle, so no wood or leather gets into the ramp."""
    px = []
    for kind, name in TOOLS.items():
        a = tf.drawn(name(art))
        if a is not None:
            lab, _ = tf.regions(a, "axe")
            px.append(a[lab == tf.WORK][:, :3])
    a = tf.drawn(f"{art}_sword")
    if a is not None:
        lab, _ = tf.sword_regions(a)
        px.append(a[lab == tf.WORK][:, :3])
    return np.concatenate(px)


def main():
    out = {}
    for rid, art in METALS.items():
        out[rid] = step_ramp(art)
    out.update({k: np.array(v, float) for k, v in HAND.items()})
    for k, v in out.items():
        p = os.path.join(T, "ramps", k + ".png")
        Image.fromarray(v.reshape(1, 8, 3).clip(0, 255).astype(np.uint8)).save(p)
        parts.art.archive(p, "ramps__" + k, "parts")
    # the sheet: every ramp we deliver
    files = sorted(glob.glob(os.path.join(T, "ramps", "*.png")))
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    im = Image.new("RGB", (560, 50 + 30 * len(files)), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)
    d.text((14, 12), f"Every ramp for crafting ({len(files)}), dark to light", fill=(0xf5, 0xe4, 0xbc), font=big)
    for i, f in enumerate(files):
        y = 46 + i * 30
        r = Image.open(f).convert("RGB").resize((320, 24), Image.NEAREST)
        im.paste(r, (150, y))
        d.text((14, y + 4), os.path.basename(f)[:-4], fill=(0xc2, 0xac, 0x88), font=font)
        d.text((480, y + 4), "new" if os.path.basename(f)[:-4] in out else "", fill=(0xff, 0xbe, 0x45), font=font)
    s = os.path.join(HERE, "sheets", "ramps.png")
    im.save(s)
    parts.art.archive(s, "sheet_ramps", "sheets")
    print(s, {k: np.round(v[[0, 4, 7]]).astype(int).tolist() for k, v in out.items()})


if __name__ == "__main__":
    main()
