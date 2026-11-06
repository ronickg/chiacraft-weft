#!/usr/bin/env python3
"""A review sheet of every egg look (`build.py --out DIR` first): per look a close-up from two
sides, the egg at its real size beside a 1 m block's edge, and its item icon in a slot if one is
given.

    ~/.venvs/blocky/bin/python sheet.py DIR out.png [--icons ICONDIR] [--title "..."]
"""

import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "..", "..", ".claude", "skills", "blocky-model", "scripts"))
sys.path.insert(0, HERE)
from render import render  # noqa: E402

from build import CM, LOOKS  # noqa: E402

LABELS = {
    "dromaeosaur": ("Raptors", "Velociraptor, V. osmolskae, Utahraptor"),
    "oviraptorid": ("Oviraptorid", "Citipati"),
    "ceratopsian": ("Soft-shelled", "Protoceratops, Psittacosaurus"),
    "ankylosaur": ("Ankylosaurs", "Gastonia, Pinacosaurus"),
    "ornithopod": ("Ornithopod", "Iguanacolossus"),
    "sauropod": ("Sauropod", "Cedarosaurus"),
    "crocodilian": ("Crocodilians", "caimans, neosuchian"),
    "lizard": ("Lizard", "Priscagama"),
    "roe_trout": ("Trout roe", "brown trout"),
    "roe_pacu": ("Pacu roe", "pacu"),
    "roe_lycoptera": ("Lycoptera roe", "Lycoptera"),
    "roe_lungfish": ("Lungfish eggs", "Ceratodus"),
    "wyvern": ("Wyvern", "fantasy"),
}
BG = (40, 30, 24, 255)
CARD = (58, 44, 34, 255)
INK = (238, 226, 204)
DIM = (176, 160, 136)
SLOT = (53, 32, 21, 255)
FRAME = (150, 104, 60, 255)


def font(n):
    for f in ("/System/Library/Fonts/Supplemental/Arial Bold.ttf", "/System/Library/Fonts/Helvetica.ttc"):
        if os.path.exists(f):
            return ImageFont.truetype(f, n)
    return ImageFont.load_default()


def fit(img, w, h):
    """Scale an RGBA render to fit w x h with nearest pixels (whole multiples where it can)."""
    ih, iw = img.shape[:2]
    k = min(w / iw, h / ih)
    k = max(1, int(k)) if k >= 1 else k
    im = Image.fromarray(img, "RGBA")
    return im.resize((max(1, int(iw * k)), max(1, int(ih * k))), Image.NEAREST)


def card(d, name, icons=None, W=300, H=470):
    spec = json.load(open(os.path.join(d, name, "spec.json")))
    tex = np.array(Image.open(os.path.join(d, name, "texture.png")).convert("RGBA"))
    c = Image.new("RGBA", (W, H), CARD)
    dr = ImageDraw.Draw(c)
    title, who = LABELS.get(name, (name, ""))
    dr.text((12, 8), title, font=font(20), fill=INK)
    dr.text((12, 32), who, font=font(13), fill=DIM)
    # Close-ups: three-quarters from above, and from the side.
    for k, (yaw, pitch) in enumerate(((90, 25), (30, 15))):
        img = render(spec, tex, yaw=yaw, pitch=pitch, ppu=24.0, pad=4)
        im = fit(img, 136, 170)
        c.alpha_composite(im, (12 + k * 142 + (136 - im.width) // 2, 56 + (170 - im.height) // 2))
    # As laid: a nest's clutch (or the roe as it lies), and the egg at its real size (a block's
    # 32 texels = 128 px here) beside a 10 cm bar.
    y0 = 236
    cl = os.path.join(d, "clutch", name)
    look = LOOKS[name]
    if os.path.exists(os.path.join(cl, "spec.json")):
        cs, ct = json.load(open(os.path.join(cl, "spec.json"))), np.array(Image.open(os.path.join(cl, "texture.png")).convert("RGBA"))
        img = render(cs, ct, yaw=35, pitch=35, ppu=8.0, pad=4)
        im = fit(img, 150, 110)
        dr.text((12, y0), "a clutch, as laid", font=font(12), fill=DIM)
        c.alpha_composite(im, (12 + (150 - im.width) // 2, y0 + 18 + (110 - im.height) // 2))
    else:
        img = render(spec, tex, yaw=35, pitch=35, ppu=12.0, pad=4)
        im = fit(img, 150, 110)
        dr.text((12, y0), "as it lies", font=font(12), fill=DIM)
        c.alpha_composite(im, (12 + (150 - im.width) // 2, y0 + 18 + (110 - im.height) // 2))
    y1 = y0 + 136
    img = render(spec, tex, yaw=60, pitch=25, ppu=2.0, pad=2)
    im = Image.fromarray(img, "RGBA")
    dr.text((12, y1), "real size", font=font(12), fill=DIM)
    c.alpha_composite(im, (20, y1 + 16 + max(0, 66 - im.height)))
    dr.rectangle((20 + im.width + 8, y1 + 80, 20 + im.width + 8 + int(10 * CM * 2), y1 + 82), fill=DIM)
    dr.text((20 + im.width + 8, y1 + 64), "10 cm", font=font(11), fill=DIM)
    if "L" in look:
        dr.text((12, H - 22), f"{look['L'] / CM:.0f} x {look['W'] / CM:.0f} cm", font=font(12), fill=DIM)
    # The icon in a slot.
    icon = icons and os.path.join(icons, f"{name}.png")
    sx, sy = W - 12 - 104, y0 + 4
    dr.rectangle((sx - 4, sy - 4, sx + 100, sy + 100), fill=FRAME)
    dr.rectangle((sx, sy, sx + 96, sy + 96), fill=SLOT)
    if icon and os.path.exists(icon):
        ic = Image.open(icon).convert("RGBA").resize((96, 96), Image.NEAREST)
        c.alpha_composite(ic, (sx, sy))
        ic1 = Image.open(icon).convert("RGBA")
        dr.rectangle((sx + 30, sy + 112, sx + 66, sy + 148), fill=SLOT)
        c.alpha_composite(ic1, (sx + 32, sy + 114))
    else:
        dr.text((sx + 18, sy + 40), "icon next", font=font(12), fill=DIM)
    return c


if __name__ == "__main__":
    d, out = sys.argv[1], sys.argv[2]
    icons = sys.argv[sys.argv.index("--icons") + 1] if "--icons" in sys.argv else None
    title = sys.argv[sys.argv.index("--title") + 1] if "--title" in sys.argv else "Eggs by species"
    names = [n for n in LOOKS if os.path.exists(os.path.join(d, n, "spec.json"))]
    cols = 5
    W, H = 300, 470
    rows = (len(names) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * (W + 10) + 10, rows * (H + 10) + 60), BG)
    ImageDraw.Draw(sheet).text((14, 14), title, font=font(26), fill=INK)
    for i, n in enumerate(names):
        sheet.alpha_composite(card(d, n, icons, W, H), (10 + (i % cols) * (W + 10), 56 + (i // cols) * (H + 10)))
    wc = os.path.join(d, "wyvern_colors")
    if os.path.isdir(wc):
        ks = sorted(os.listdir(wc))
        strip = Image.new("RGBA", (cols * (W + 10) + 10, 250), BG)
        dw = ImageDraw.Draw(strip)
        dw.text((14, 8), "Wyvern egg: colorings to choose from", font=font(20), fill=INK)
        for i, k in enumerate(ks):
            spec = json.load(open(os.path.join(wc, k, "spec.json")))
            tex = np.array(Image.open(os.path.join(wc, k, "texture.png")).convert("RGBA"))
            for j, (yaw, pitch) in enumerate(((60, 25), (200, 15))):
                im = fit(render(spec, tex, yaw=yaw, pitch=pitch, ppu=6.0, pad=4), 170, 180)
                strip.alpha_composite(im, (14 + i * 380 + j * 180, 40))
            dw.text((14 + i * 380, 224), k.replace("_", " "), font=font(14), fill=DIM)
        full = Image.new("RGBA", (sheet.width, sheet.height + strip.height), BG)
        full.alpha_composite(sheet, (0, 0))
        full.alpha_composite(strip, (0, sheet.height))
        sheet = full
    sheet.convert("RGB").save(out)
    print(out)
