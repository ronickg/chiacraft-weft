"""Round 2, fixed (Ronald: "ui is much better, but ... text size and maybe like what's the border
around one of the stats etc, it should be clever with ui"): the animal screen, its Genes tab, the
taming plate and the player's stats screen, from the game's own parts (gui textures, the UI kit's
icons, the baked pixel font) at GUI scale 2 (1 UI px = 1 px), shown 2x.

Every part is laid out on fixed columns and checked by `lint()` before the sheet is written:
everything inside its panel, nothing overlapping (so no text on a bar), right-aligned values on
one edge per column, headers centred over their columns. The sheet's own text uses the same
pixel font: titles at twice the UI size, everything else at the UI size.

Usage: python3 mock_taming_v3.py REPO OUT_DIR SHOTS_DIR"""
import os
import sys

import numpy as np
from PIL import Image

REPO, OUT, SHOTS = sys.argv[1], sys.argv[2], sys.argv[3]
sys.path.insert(0, os.path.join(REPO, "mods/base/source/ui"))
import pxkit as k  # noqa: E402

GUI = os.path.join(REPO, "mods/base/assets/textures/gui")
KIT = os.path.join(REPO, "mods/base/source/ui/kit")
ITEM = os.path.join(REPO, "mods/base/assets/textures/item")
AITEM = os.path.join(REPO, "mods/animals/assets/textures/item")
CREAM, GOLD, DIM, SHADOW = (228, 218, 207), (242, 218, 154), (198, 163, 131), (40, 24, 12)
LABEL, ON_ORANGE = (244, 234, 216), (255, 244, 220)
GREEN, RED, BLUE, PURPLE, AMBER = (138, 200, 15), (216, 34, 30), (47, 141, 228), (155, 79, 224), (240, 170, 40)
QUAL = [(150, 140, 130), GREEN, BLUE, PURPLE, AMBER]
# Icon sizes, one per job: in a row beside text, in a button, as a heading or badge.
ROW_ICON, BUTTON_ICON, HEAD_ICON = 16, 20, 24


# ------------------------------------------------------------------ what was drawn, for the lint
class Rec:
    def __init__(self, name):
        self.name, self.boxes, self.inner, self.cols = name, [], None, {}

    def add(self, kind, x, y, w, h, what, col=None, align=None):
        self.boxes.append((kind, int(x), int(y), int(w), int(h), what, col, align))


REC = Rec("none")


class Font:
    def __init__(self, d):
        self.atlas = Image.open(os.path.join(d, "default.png")).convert("L")
        self.g = {}
        for line in open(os.path.join(d, "default.font")):
            key, _, v = line.partition(" = ")
            if key == "cell":
                self.cw, self.ch = map(int, v.split())
            elif key == "baseline":
                self.baseline = float(v)
            elif key == "line":
                self.line = int(float(v))
            elif key == "caps":
                self.caps = int(float(v))
            elif line.startswith("g "):
                c, col, row, left, w, bearing, adv = line.split()[1:]
                self.g[chr(int(c, 16))] = tuple(map(float, (col, row, left, w, bearing, adv)))
        self.top = int((self.line - self.caps) // 2 + self.caps - self.baseline)

    def width(self, s):
        if not s:
            return 0
        w = sum(self.g.get(c, self.g["?"])[5] for c in s)
        first, last = self.g.get(s[0], self.g["?"]), self.g.get(s[-1], self.g["?"])
        return int(w - first[4] - (last[5] - last[4] - last[3]))

    def missing(self, s):
        return [c for c in s if c not in self.g]

    def draw(self, img, s, x, y, color=CREAM, shadow=SHADOW, align="l", w=0, rec=True, col=None):
        assert not self.missing(s), f"no glyph for {self.missing(s)!r} in {s!r}"
        if align == "c":
            x = x + (w - self.width(s)) // 2
        elif align == "r":
            x = x + w - self.width(s)
        if rec:
            # The ink's box: capitals and descenders.
            REC.add("text", x, y + 2, self.width(s), self.line - 3, s, col, align)
        lead = self.g.get(s[0], self.g["?"])[4] if s else 0
        for c_, off in ((shadow, 1), (color, 0)):
            if c_ is None:
                continue
            pen = -lead
            for c in s:
                cl, rw, left, gw, bearing, adv = self.g.get(c, self.g["?"])
                if c != " " and gw > 0:
                    cell = self.atlas.crop((int(cl * self.cw + left), int(rw * self.ch),
                                            int(cl * self.cw + left + gw), int(rw * self.ch + self.ch)))
                    t = Image.new("RGBA", cell.size, c_ + (255,))
                    t.putalpha(cell)
                    img.alpha_composite(t, (int(x + pen + bearing + off), int(y + self.top + off)))
                pen += adv
        return self.width(s)


F = Font(os.path.join(REPO, "mods/base/assets/fonts"))


def load(path):
    return Image.open(path).convert("RGBA")


def gtex(name):
    return np.array(load(os.path.join(GUI, name + ".png")))


def nine(name, w, h, border, src=None):
    a = gtex(name) if src is None else np.array(load(src))
    return Image.fromarray(k.fit9(a, w, h, border))


def put(img, a, x, y):
    img.alpha_composite(a if isinstance(a, Image.Image) else Image.fromarray(a), (int(x), int(y)))


def icon(img, path, x, y, size=ROW_ICON, rec=True):
    a = load(path)
    if a.width != size:
        a = a.resize((size, size), Image.NEAREST)
    put(img, a, x, y)
    if rec:
        REC.add("icon", x, y, size, size, os.path.basename(path))


def kit(name):
    for d in ("icons", "categories", "actions", ""):
        p = os.path.join(KIT, d, name + ".png")
        if os.path.exists(p):
            return p
    raise FileNotFoundError(name)


def stat_icon(img, name, x, y):
    put(img, load(os.path.join(GUI, f"stat_icon_{name}.png")), x, y)
    REC.add("icon", x, y, 16, 16, name)


def panel(img, x, y, w, h):
    put(img, nine("panel", w, h, 9), x, y)
    REC.inner = (x + 9, y + 9, x + w - 9, y + h - 9)


def plate(img, x, y, w, h):
    put(img, nine("hud_plate", w, h, 3), x, y)
    REC.inner = (x + 3, y + 3, x + w - 3, y + h - 3)


def button(img, x, y, w, label="", cls="button", h=22, ico=None, color=None):
    put(img, nine(cls, w, h, 5), x, y)
    REC.add("button", x, y, w, h, label or cls)
    iw = BUTTON_ICON if ico else 0
    gap = 4 if ico and label else 0
    tw = F.width(label)
    assert iw + gap + tw + 8 <= w, f"button {label!r} too narrow: {iw + gap + tw + 8} > {w}"
    sx = x + (w - (iw + gap + tw)) // 2
    if ico:
        icon(img, ico, sx, y + (h - BUTTON_ICON) // 2, BUTTON_ICON, rec=False)
    col = color or (ON_ORANGE if cls in ("button", "button_hover", "button_pressed") else LABEL)
    if cls == "button_disabled":
        col = (154, 143, 134)
    if label:
        F.draw(img, label, sx + iw + gap, y + (h - F.line) // 2, col, rec=False)


def tab(img, x, w, label, active, ico=None, y=0):
    put(img, nine("tab_pressed" if active else "tab", w, 24, (6, 5, 6, 2)), x, y)
    sx = x + (w - ((BUTTON_ICON + 4) if ico else 0) - F.width(label)) // 2
    if ico:
        icon(img, ico, sx, y + 2, BUTTON_ICON, rec=False)
        sx += BUTTON_ICON + 4
    F.draw(img, label, sx, y + (24 - F.line) // 2 + 1, ON_ORANGE if active else DIM, rec=False)


def tabs(img, items):
    x = 14
    for label, active, ico in items:
        w = F.width(label) + 24 + BUTTON_ICON + 4
        tab(img, x, w, label, active, kit(ico))
        x += w + 2


def bar(img, x, y, w, frame, fill, share, h=12):
    """A game bar, its value written beside it (never on it)."""
    put(img, nine(frame, w, h, 3), x, y)
    f = nine(fill, w, h, 3)
    fw = int(round(w * max(0.0, min(1.0, share))))
    if fw > 0:
        put(img, f.crop((0, 0, fw, h)), x, y)
    REC.add("bar", x, y, w, h, frame)


def chip(img, x, y, text, color=CREAM, cls="chip"):
    w = F.width(text) + 12
    put(img, nine(cls, w, 20, 2), x, y)
    REC.add("chip", x, y, w, 20, text)
    F.draw(img, text, x + 6, y + (20 - F.line) // 2, color, rec=False)
    return w


def slot(img, x, y, item=None, count=None, glow=False):
    put(img, gtex("slot"), x, y)
    if glow:
        put(img, gtex("slot_highlight"), x, y)
    REC.add("slot", x, y, 36, 36, item or "slot")
    if item:
        for d in (ITEM, AITEM):
            p = os.path.join(d, item + ".png")
            if os.path.exists(p):
                put(img, load(p), x + 2, y + 2)
                break
    if count is not None:
        F.draw(img, str(count), x + 2, y + 36 - F.line - 1, LABEL, align="r", w=30, rec=False)


def keycap(img, x, y, letter):
    put(img, nine("key", 22, 22, 6, os.path.join(KIT, "key.png")), x, y)
    REC.add("key", x, y, 22, 22, letter)
    F.draw(img, letter, x, y + (22 - F.line) // 2 - 1, (58, 36, 22), shadow=None, align="c", w=22, rec=False)


def hearts(img, x, y, share, n=5):
    for i in range(n):
        v = share * n - i
        name = "heart_full" if v >= 0.75 else "heart_half" if v >= 0.25 else "heart_empty"
        icon(img, os.path.join(GUI, name + ".png"), x + i * 18, y, ROW_ICON)


def segments(img, x, y, n, on, w=18, h=8, color=GREEN):
    for i in range(n):
        a = np.zeros((h, w, 4), np.uint8)
        a[..., 3] = 255
        a[..., :3] = (39, 30, 26)
        a[0, :, :3] = a[-1, :, :3] = (104, 62, 38)
        a[:, 0, :3] = a[:, -1, :3] = (104, 62, 38)
        if i < on:
            c = np.array(color)
            a[1:-1, 1:-1, :3] = c
            a[1, 1:-1, :3] = np.minimum(255, c * 1.25 + 20)
            a[-2, 1:-1, :3] = c * 0.65
        put(img, a, x + i * (w + 2), y)
    REC.add("bar", x, y, n * (w + 2) - 2, h, "segments")


def divider(img, x, y, w):
    a = np.zeros((1, w, 4), np.uint8)
    a[..., :3] = (104, 62, 38)
    a[..., 3] = 255
    put(img, a, x, y)


def arrow(img, x, y, color=DIM):
    """A small right arrow, 7 x 7 (the font has none)."""
    rows = ["0001000", "0001100", "1111110", "1111111", "1111110", "0001100", "0001000"]
    for dy, r in enumerate(rows):
        for dx, ch in enumerate(r):
            if ch == "1":
                img.putpixel((x + dx + 1, y + dy + 6), SHADOW + (255,))
                img.putpixel((x + dx, y + dy + 5), color + (255,))
    REC.add("icon", x, y + 5, 8, 8, "arrow")


def keyed(im, tol):
    """The crop without the backdrop it was shot on: filled from its edges inward, so pixels of the
    animal that happen to match the backdrop's colour stay."""
    from PIL import ImageDraw
    im = im.copy()
    w, h = im.size
    for x, y in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1), (w // 2, 0), (w // 2, h - 1), (0, h // 2), (w - 1, h // 2)]:
        if im.getpixel((x, y))[3]:
            ImageDraw.floodfill(im, (x, y), (0, 0, 0, 0), thresh=tol)
    return im


def portrait_box(img, x, y, w, h, crop):
    put(img, nine("portrait", w, h, 4), x, y)
    c = crop
    if c.width > w - 12 or c.height > h - 12:
        s = min((w - 12) / c.width, (h - 12) / c.height)
        c = c.resize((int(c.width * s), int(c.height * s)), Image.NEAREST)
    put(img, c, x + (w - c.width) // 2, y + (h - c.height) // 2)
    REC.add("portrait", x, y, w, h, "portrait")


def badge(img, x, y, n):
    put(img, gtex("hud_badge"), x, y)
    F.draw(img, str(n), x, y + (48 - F.line) // 2 - 3, LABEL, align="c", w=40, rec=False)


class Col:
    """A column: left edge and width; values right-aligned to its right edge, headers centred."""

    def __init__(self, name, x, w, align="r"):
        self.name, self.x, self.w, self.align = name, x, w, align
        REC.cols[name] = (x, w)

    def value(self, img, s, y, color=LABEL):
        if s:
            F.draw(img, s, self.x, y, color, align=self.align, w=self.w, col=self.name)

    def left(self, img, s, y, color=CREAM):
        if s:
            F.draw(img, s, self.x, y, color, col=self.name, align="l")

    def head(self, img, s, y):
        """A header lines up the way its values do: over their right edge, their centre or start."""
        F.draw(img, s, self.x, y, DIM, align=self.align, w=self.w, col=self.name)


def quality(points):
    return min(4, points // 6)


# ------------------------------------------------------------------ crops from the game's shots
WOLF = keyed(load(os.path.join(SHOTS, "16_animal_screen.png")).crop((284, 82, 478, 200)), 26)
PLAYER = keyed(load(os.path.join(SHOTS, "03_inventory.png")).crop((541, 211, 625, 337)), 16)
NAMES = {"health": "Health", "stamina": "Stamina", "food": "Food", "damage": "Damage", "speed": "Speed",
         "weight": "Weight", "oxygen": "Oxygen", "fortitude": "Fortitude", "torpor": "Torpor"}


# ------------------------------------------------------------------ 1. the animal screen, Overview

def animal_overview():
    W, H = 780, 404
    img = Image.new("RGBA", (W, H + 22), (0, 0, 0, 0))
    tabs(img, (("Overview", True, "follow"), ("Genes", False, "taming"), ("Family", False, "tribe")))
    panel(img, 0, 22, W, H)
    put(img, gtex("close"), W - 26, 32)
    REC.add("icon", W - 26, 32, 14, 14, "close")
    top = 38

    # left: the portrait card, who it is, its life
    L, LW = 16, 216
    portrait_box(img, L, top, LW, 150, WOLF)
    badge(img, L + 6, top + 6, 12)
    icon(img, kit("male"), L + LW - 30, top + 6, HEAD_ICON)
    y = top + 158
    F.draw(img, "Grey Wolf", L, y, GOLD)
    icon(img, kit("rename"), L + F.width("Grey Wolf") + 6, y, ROW_ICON)
    y += 22
    cx = L
    for t in ("Adult, 22 days", "Steady", "Pack hunter"):
        if cx + F.width(t) + 12 > L + LW:
            cx, y = L, y + 24
        cx += chip(img, cx, y, t) + 4
    y += 30
    life_val = Col("life value", L + 150, LW - 150)
    for ico, frame, share, val in (("health", "life_health", 0.97, "33 / 34"), ("food", "life_food", 0.98, "132 / 134"),
                                   ("oxygen", "life_water", 1.0, "100 / 100")):
        stat_icon(img, ico, L, y - 2)
        bar(img, L + 22, y, 122, frame, frame + "_fill", share)
        life_val.value(img, val, y - 2)
        y += 22
    icon(img, kit("taming"), L, y - 2, ROW_ICON)
    hearts(img, L + 22, y - 2, 0.25)
    life_val.value(img, "25 / 100", y - 2)
    y += 26
    icon(img, os.path.join(GUI, "temp_comfortable.png"), L, y - 2, ROW_ICON)
    F.draw(img, "Comfortable, 12 °C", L + 22, y - 2, GREEN)
    y += 20
    icon(img, os.path.join(GUI, "effect_well_fed.png"), L, y - 2, ROW_ICON)
    F.draw(img, "Well fed", L + 22, y - 2, CREAM)

    # middle: the stats; a placed point shows in green until it is kept
    M = 248
    F.draw(img, "Stats", M, top + 2, GOLD)
    chip(img, M + 46, top, "2 to place", ON_ORANGE, cls="chip_points")
    val = Col("stat value", M + 166, 34)
    unit = Col("stat unit", M + 202, 12)
    wild = Col("wild", M + 216, 34, "c")
    tamed = Col("tamed", M + 252, 38, "c")
    wild.head(img, "Wild", top + 22)
    tamed.head(img, "Tamed", top + 22)
    rows = [("health", "33.9", "", 0.55, 8, 2, False), ("stamina", "207", "", 0.6, 12, 0, False),
            ("food", "140", "", 0.5, 5, 1, True), ("damage", "99", "%", 0.45, 3, 0, False),
            ("speed", "94", "%", 0.4, 1, 0, False), ("weight", "56", "", 0.5, 6, 0, False),
            ("oxygen", "81", "", 0.42, 4, 0, False), ("fortitude", "20", "", 0.3, 14, 0, False)]
    y = top + 44
    for name, v, u, share, w_, t_, pending in rows:
        stat_icon(img, name, M, y - 2)
        F.draw(img, NAMES[name], M + 22, y - 2, CREAM)
        q = quality(w_)
        bar(img, M + 96, y, 66, f"stat_q{q}", f"stat_q{q}_fill", share)
        val.value(img, v, y - 2, GREEN if pending else LABEL)
        unit.left(img, u, y - 2, GREEN if pending else LABEL)
        wild.value(img, str(w_), y - 2, QUAL[q])
        tamed.value(img, f"+{t_}" if t_ else "0", y - 2, GREEN if pending else (CREAM if t_ else DIM))
        button(img, M + 294, y - 4, 20, "+", "button", h=20)
        y += 26
    divider(img, M, y - 4, 314)
    y += 6
    stat_icon(img, "torpor", M, y - 2)
    F.draw(img, "Torpor", M + 22, y - 2, DIM)
    bar(img, M + 96, y, 66, "stat_q1", "stat_q1_fill", 0.5)
    val.value(img, "91", y - 2)
    wild.value(img, "9", y - 2, QUAL[1])
    tamed.value(img, "-", y - 2, DIM)
    y += 28
    button(img, M, y, 150, "Keep 1 point", "button", h=22)
    button(img, M + 156, y, 100, "Undo", "button_secondary", h=22, ico=kit("refresh"))
    y += 30
    F.draw(img, "Green: after the point you place.", M, y, DIM)
    F.draw(img, "Bar colour: wild points, grey few,", M, y + 18, DIM)
    F.draw(img, "then green, blue, purple, gold.", M, y + 36, DIM)

    # right: orders, stance, whistles, more
    R, BW = 572, 94
    F.draw(img, "Orders", R, top + 2, GOLD)
    y = top + 22

    def grid(items, y):
        for i, (t, ic, cls) in enumerate(items):
            button(img, R + (i % 2) * (BW + 4), y + (i // 2) * 26, BW, t, cls, h=24, ico=kit(ic), color=LABEL)
        return y + ((len(items) + 1) // 2) * 26

    y = grid((("Follow", "follow", "button_pressed"), ("Stay", "stay", "button_secondary"),
              ("Wander", "move", "button_secondary"), ("Guard", "home", "button_secondary")), y) + 8
    F.draw(img, "Stance", R, y, GOLD)
    y = grid((("Passive", "passive", "button_secondary"), ("Neutral", "neutral", "button_pressed"),
              ("Attack", "attack", "button_secondary"), ("Assist", "whistle", "button_secondary")), y + 20) + 8
    F.draw(img, "Whistles", R, y, GOLD)
    y += 20
    for j, (key, ic) in enumerate((("J", "follow"), ("K", "stay"), ("H", "move"), ("G", "attack"), ("B", "passive"))):
        keycap(img, R + j * 39, y, key)
        icon(img, kit(ic), R + j * 39 + 23, y + 3, ROW_ICON)
    y += 32
    F.draw(img, "More", R, y, GOLD)
    y = grid((("Ride", "ride", "button_disabled"), ("Tribe", "tribe", "button_secondary"),
              ("Sniff out", "feed", "button_secondary"), ("Release", "release", "button_quit")), y + 20)
    F.draw(img, "Ride: needs a saddle on it", R, y + 2, DIM)
    return img


# ------------------------------------------------------------------ 1b. the Genes tab

def genes_tab():
    W, H = 470, 388
    img = Image.new("RGBA", (W, H + 22), (0, 0, 0, 0))
    tabs(img, (("Overview", False, "follow"), ("Genes", True, "taming"), ("Family", False, "tribe")))
    panel(img, 0, 22, W, H)
    top = 38
    F.draw(img, "Coat", 16, top, GOLD)
    cx = 16
    for t, c in (("Grey", CREAM), ("Saddle", CREAM), ("Amber eyes", CREAM), ("Scar: ear", DIM)):
        cx += chip(img, cx, top + 20, t, c) + 4
    y = top + 54
    F.draw(img, "Genes", 16, y, GOLD)
    from_col = Col("alleles", 180, 104)
    what = Col("gene what", 292, W - 16 - 292 - 8)
    from_col.head(img, "mother, father", y)
    y += 22
    genes = [("Dominant black", ("K", "k"), "carries black", None),
             ("Arctic white", ("Ar", "Ar"), "white in winter", None),
             ("Tone", None, "58%", 0.58), ("Size", None, "+8%", 0.33), ("Shade", None, "50%", 0.5),
             ("Thick fur", ("F", "f"), "warmer by 3 °C", None), ("Temperament", ("S", "s"), "steady", None)]
    for name, alleles, text, share in genes:
        F.draw(img, name, 24, y, CREAM)
        if alleles:
            ax = from_col.x
            for al in alleles:
                dom = al[0].isupper()
                ax += chip(img, ax, y - 2, al, GOLD if dom else DIM, cls="chip_points" if dom else "chip") + 2
            what.left(img, text, y, DIM)
        else:
            bar(img, from_col.x, y + 2, from_col.w, "stat_q2", "stat_q2_fill", share)
            what.left(img, text, y, LABEL)
        y += 26
    y += 8
    F.draw(img, "Kind", 16, y, GOLD)
    y += 20
    F.draw(img, "Canis lupus, dog family", 16, y, CREAM)
    F.draw(img, "72 cm at the shoulder, strength 93%", 16, y + 18, DIM)
    F.draw(img, "Breeds in winter, out of season now", 16, y + 36, DIM)
    return img


# ------------------------------------------------------------------ 2. the taming plate

def taming_plate(mode):
    W = 360
    asleep = mode == "asleep"
    H = 152 if asleep else 130
    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    plate(img, 0, 0, W, H)
    icon(img, kit("taming"), 8, 6, HEAD_ICON)
    F.draw(img, "Raptor" if asleep else "Grey Wolf", 38, 10, GOLD)
    lv = Col("level", W - 110, 70)
    lv.value(img, "Level 20" if asleep else "Level 12", 10)
    icon(img, kit("female" if asleep else "male"), W - 34, 6, HEAD_ICON)
    label = Col("plate label", 10, 60)
    value = Col("plate value", 278, 34)
    extra = Col("plate extra", 316, 34)
    share = 0.41 if asleep else 0.68
    eff = 0.79 if asleep else 0.94
    ec = GREEN if eff > 0.85 else AMBER
    y = 38
    label.left(img, "Tamed", y)
    bar(img, 74, y + 2, 198, "life_taming", "life_taming_fill", share)
    value.value(img, f"{int(share * 100)}%", y)
    y += 22
    label.left(img, "Effect", y)
    segments(img, 74, y + 4, 10, round(eff * 10), w=18, h=8, color=ec)
    value.value(img, f"{int(eff * 100)}%", y, ec)
    extra.value(img, f"+{int((20 if asleep else 12) * eff / 2)} lv", y, CREAM)
    y += 22
    if asleep:
        label.left(img, "Wakes", y)
        bar(img, 74, y + 2, 198, "life_torpor", "life_torpor_fill", 0.8)
        value.value(img, "2:40", y)
        y += 22
    sx, sy = 10, y + 2
    if mode == "asking":
        slot(img, sx, sy, "beef", glow=True)
        F.draw(img, "Wants raw beef now", 52, sy + 2, GREEN)
        F.draw(img, "or kibble, you have 4", 52, sy + 19, DIM)
    elif mode == "waiting":
        slot(img, sx, sy, "beef")
        F.draw(img, "Asks again in 12 s", 52, sy + 2, CREAM)
        bar(img, 52, sy + 24, 160, "stat_q0", "stat_q0_fill", 0.6, h=8)
    else:
        slot(img, sx, sy, "mutton", count=3)
        F.draw(img, "Asleep: feed it to tame", 52, sy + 2, CREAM)
        F.draw(img, "3 mutton in its mouth", 52, sy + 19, DIM)
    return img


def taming_steps():
    W, H = 360, 76
    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    plate(img, 0, 0, W, H)
    steps = (("Wild", "follow", DIM), ("Asks", "feed", CREAM), ("Fed", "taming", CREAM), ("Tamed", "confirm", GREEN))
    for i, (t, ic, c) in enumerate(steps):
        x = 12 + i * 88
        put(img, gtex("slot"), x + 12, 6)
        REC.add("slot", x + 12, 6, 36, 36, t)
        icon(img, kit(ic), x + 18, 12, HEAD_ICON, rec=False)
        F.draw(img, t, x, 48, c, align="c", w=60)
        if i < 3:
            arrow(img, x + 68, 13)
    return img


# ------------------------------------------------------------------ 3. the player's stats

def stats_screen():
    W, H = 700, 452
    img = Image.new("RGBA", (W, H + 22), (0, 0, 0, 0))
    tabs(img, (("Inventory", False, "inventory"), ("Stats", True, "skills"), ("Advancements", False, "journal")))
    panel(img, 0, 22, W, H)
    top = 38
    # left: you, your level, what you carry, how you feel
    L, LW = 16, 200
    portrait_box(img, L, top, LW, 168, PLAYER)
    badge(img, L + 6, top + 6, 7)
    y = top + 176
    right = Col("left value", L + 100, LW - 100)
    F.draw(img, "Level 7", L, y, GOLD)
    right.value(img, "108 / 146 XP", y, DIM)
    bar(img, L, y + 20, LW, "stat_xp", "stat_xp_fill", 108 / 146, h=8)
    y += 38
    stat_icon(img, "weight", L, y - 2)
    F.draw(img, "Carrying", L + 22, y - 2, CREAM)
    right.value(img, "21.9 / 100 kg", y - 2)
    bar(img, L, y + 18, LW, "stat_q0", "stat_q0_fill", 21.9 / 100, h=8)
    F.draw(img, "Slower past 100 kg", L, y + 30, DIM)
    y += 58
    for n, t in (("effect_well_fed", "Well fed"), ("effect_warm", "Warm"), ("effect_wet", "Wet")):
        icon(img, os.path.join(GUI, n + ".png"), L, y - 2, ROW_ICON)
        F.draw(img, t, L + 22, y - 2, CREAM)
        y += 20
    # right: the nine stats; a placed point shows now and after until it is kept
    R = 236
    F.draw(img, "Stats", R, top + 2, GOLD)
    chip(img, R + 46, top, "4 to place", ON_ORANGE, cls="chip_points")
    now = Col("now", R + 204, 34)
    unit = Col("unit", R + 245, 36, "l")
    after = Col("after", R + 292, 38)
    placed = Col("placed", R + 364, 20, "c")
    now.head(img, "now", top + 22)
    after.head(img, "after", top + 22)
    rows = [("health", "100", "110", "", 1), ("stamina", "100", None, "", 0), ("food", "100", None, "units", 0),
            ("damage", "100", None, "%", 0), ("speed", "100", None, "%", 0), ("weight", "100", None, "kg", 0),
            ("oxygen", "100", None, "", 0), ("fortitude", "0", None, "", 0), ("torpor", "200", None, "", 0)]
    shares = {"fortitude": 0.02, "damage": 0.4, "speed": 0.4}
    y = top + 44
    for name, n, a, u, p in rows:
        stat_icon(img, name, R, y - 2)
        F.draw(img, "Stomach" if name == "food" else NAMES[name], R + 22, y - 2, CREAM)
        bar(img, R + 100, y, 98, f"stat_{name}", f"stat_{name}_fill", shares.get(name, 0.5))
        now.value(img, n, y - 2)
        unit.left(img, u, y - 2)
        if a:
            arrow(img, R + 282, y - 2)
            after.value(img, a, y - 2, GREEN)
        button(img, R + 340, y - 4, 20, "-", "button_secondary" if p else "button_disabled", h=20)
        placed.value(img, str(p), y - 2, GREEN if p else DIM)
        button(img, R + 388, y - 4, 20, "+", "button_hover" if name == "speed" else "button", h=20)
        y += 26
    y += 6
    button(img, R, y, 150, "Keep 1 point", "button", h=22)
    button(img, R + 156, y, 100, "Undo", "button_secondary", h=22, ico=kit("refresh"))
    # what the hovered + (Speed's) shows
    tx, ty, tw, th = R + 160, y + 32, 270, 80
    put(img, nine("tooltip", tw, th, 3), tx, ty)
    REC.add("tooltip", tx, ty, tw, th, "tooltip")
    F.draw(img, "Speed", tx + 8, ty + 6, GOLD, rec=False)
    F.draw(img, "How fast you walk and run.", tx + 8, ty + 24, CREAM, rec=False)
    F.draw(img, "Now 100%. Next point: +2%", tx + 8, ty + 42, GREEN, rec=False)
    F.draw(img, "Heavy loads slow it down.", tx + 8, ty + 58, DIM, rec=False)
    F.draw(img, "Pointing at Speed's +:", R, ty + 6, DIM)
    return img


# ------------------------------------------------------------------ the lint

OVERLAYS = ("portrait", "tooltip")


def lint(rec):
    """What a UI review catches by measuring: returns a list of problems."""
    out = []
    ix0, iy0, ix1, iy1 = rec.inner
    for kind, x, y, w, h, what, col, align in rec.boxes:
        if x < ix0 or y < iy0 or x + w > ix1 or y + h > iy1:
            out.append(f"outside its panel: {kind} {what!r} at {x},{y} {w}x{h} (inside {rec.inner})")
    b = rec.boxes
    for i in range(len(b)):
        for j in range(i + 1, len(b)):
            k1, x1, y1, w1, h1, a1 = b[i][:6]
            k2, x2, y2, w2, h2, a2 = b[j][:6]
            inside = lambda ax, ay, aw, ah, bx, by, bw, bh: bx <= ax and by <= ay and ax + aw <= bx + bw and ay + ah <= by + bh
            # A badge or mark laid on a portrait or tooltip on purpose is not an overlap.
            if (k1 in OVERLAYS and inside(x2, y2, w2, h2, x1, y1, w1, h1)) or (k2 in OVERLAYS and inside(x1, y1, w1, h1, x2, y2, w2, h2)):
                continue
            if x1 < x2 + w2 and x2 < x1 + w1 and y1 < y2 + h2 and y2 < y1 + h1:
                out.append(f"overlap: {k1} {a1!r} and {k2} {a2!r}")
            elif ("text" in (k1, k2) and y1 < y2 + h2 and y2 < y1 + h1
                  and not any("unit" in (c or "") for c in (b[i][6], b[j][6]))):  # "99 %": a unit sits by its number
                gap = max(x2 - (x1 + w1), x1 - (x2 + w2))
                if gap < 3:
                    out.append(f"too close ({gap} px): {k1} {a1!r} and {k2} {a2!r}")
    for name, (cx, cw) in rec.cols.items():
        edges = {(x + w) for kind, x, y, w, h, what, col, align in b if col == name and align == "r"}
        lefts = {x for kind, x, y, w, h, what, col, align in b if col == name and align == "l"}
        if len(edges) > 1:
            out.append(f"column {name!r}: values end at {sorted(edges)}, not one edge")
        if len(lefts) > 1:
            out.append(f"column {name!r}: values start at {sorted(lefts)}, not one edge")
        for kind, x, y, w, h, what, col, align in b:
            if col == name and align == "c" and abs((x + w / 2) - (cx + cw / 2)) > 1:
                out.append(f"column {name!r}: header {what!r} not centred")
            if col == name and (x < cx - 1 or x + w > cx + cw + 1):
                out.append(f"column {name!r}: {what!r} wider than the column ({w} > {cw})")
    return out


def build(name, fn):
    global REC
    REC = Rec(name)
    img = fn()
    return img, lint(REC)


def big(i, z=2):
    return i.resize((i.width * z, i.height * z), Image.NEAREST)


def sheet_text(sheet, s, x, y, color, scale):
    """The sheet's own words, in the game's pixel font: titles at scale 4, the rest at 2."""
    t = Image.new("RGBA", (F.width(s) + 4, F.line + 4), (0, 0, 0, 0))
    F.draw(t, s, 0, 0, color, rec=False)
    sheet.alpha_composite(big(t, scale), (x, y))
    return F.line * scale


specs = [("overview", animal_overview), ("genes", genes_tab), ("ask", lambda: taming_plate("asking")),
         ("wait", lambda: taming_plate("waiting")), ("asleep", lambda: taming_plate("asleep")),
         ("steps", taming_steps), ("stats", stats_screen)]
parts, problems = {}, {}
for n, fn in specs:
    parts[n], problems[n] = build(n, fn)
os.makedirs(OUT, exist_ok=True)
for n, p in parts.items():
    big(p).save(os.path.join(OUT, f"part_{n}.png"))
report = [f"{n}: {len(v)} problems" + "".join(f"\n  - {m}" for m in v) for n, v in problems.items()]
open(os.path.join(OUT, "lint.txt"), "w").write("\n".join(report) + "\n")
print("\n".join(report))

Z = 2
a, g, s = parts["overview"], parts["genes"], parts["stats"]
col2 = 80 + a.width * Z
W = col2 + max(g.width, parts["ask"].width) * Z + 40
row2 = 90 + a.height * Z + 60
plates_h = sum(parts[n].height * Z + 70 for n in ("ask", "wait", "asleep", "steps"))
H = row2 + 70 + max(s.height * Z, plates_h) + 400
sheet = Image.new("RGBA", (W, H), (26, 19, 19, 255))
sheet_text(sheet, "1  Your animal: Overview", 40, 24, GOLD, 4)
sheet.alpha_composite(big(a), (40, 100))
sheet_text(sheet, "1b  Its Genes tab", col2, 24, GOLD, 4)
sheet.alpha_composite(big(g), (col2, 100))
sheet_text(sheet, "3  Your stats (P)", 40, row2, GOLD, 4)
sheet.alpha_composite(big(s), (40, row2 + 76))
sheet_text(sheet, "2  The taming plate", col2, row2, GOLD, 4)
y = row2 + 76
for n, cap in (("ask", "Asking: the food it wants glows"), ("wait", "Between feedings"),
               ("asleep", "Knocked out: when it wakes, what is in its mouth"),
               ("steps", "Over the animal: wild, asks, fed, tamed")):
    sheet_text(sheet, cap, col2, y, DIM, 2)
    sheet.alpha_composite(big(parts[n]), (col2, y + 40))
    y += 40 + parts[n].height * Z + 30
notes = [
    "Round 2, fixed. One font in two sizes: titles, and everything else. Values sit beside their",
    "bars in fixed columns, right-aligned, one unit each. Selected orders show pressed; a placed",
    "point shows green until Keep. The hovered + glows as in the game. Every part is measured:",
    "inside its panel, nothing overlapping, columns on one edge (lint.txt beside this sheet).",
    "Needs building: Keep and Undo for points, the Family tab, the steps strip.",
]
y = max(y, row2 + 76 + s.height * Z) + 30
for n in notes:
    y += sheet_text(sheet, n, 40, y, CREAM, 2) + 8
sheet = sheet.crop((0, 0, W, y + 30))
path = os.path.join(OUT, "taming_stats_round2_fixed.png")
sheet.convert("RGB").save(path)
print(path, sheet.size)
