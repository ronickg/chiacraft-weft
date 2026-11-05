"""The HUD's level corner, four ways (Ronald, 11-04, on the food tiles sheet: "the part with the
level thing still looks kinda ugly imo"). Each variant keeps the health and stamina bars as they
are in the game (kit 2: 192 x 19, a 2 px wood frame round a 13-row track) and redraws the level,
the XP bar and the points to place. Drawn at GUI scale 2 (one UI pixel a screen pixel) over a
piece of a real shot, then zoomed 3x.

Every box is recorded and checked (`lint`): inside the corner, nothing overlapping that shouldn't,
at least 2 px between neighbours (the bars' own gap), and every label measured: its capitals'
room above and below inside its face (equal, or the layout's odd leftover rounded down).

Usage: python3 mock_level_hud.py OUT.png [BACKGROUND_SHOT.png]"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "kit2"))
import kit2 as k  # noqa: E402

OUT = sys.argv[1]
BG = sys.argv[2] if len(sys.argv) > 2 else None
SHADOW = (12, 8, 7)

BAR_W, BAR_H, BAR_GAP = 192, 19, 2
XP_H = 8


# ---------------------------------------------------------------- parts
def hud_bar(w, h, fill=None, share=1.0, frame_px=2):
    """The game's HUD bar (`notes/ui/kit2/install.py`, hud_bar), with its fill cut to `share`."""
    a = k.canvas(w, h)
    m = k.shape(w, h, k.R)
    ring = k.edge(m)
    k.paint(a, ring, k.OUTLINE)
    m = m & ~ring
    for i in range(frame_px):
        f = k.edge(m)
        if i == 0:
            k.paint(a, f, k.BRONZE_L)
            up = np.zeros_like(f)
            up[1:] = ring[:-1]
            k.paint(a, f & up, k.BRONZE_HI)
            dn = np.zeros_like(f)
            dn[:-1] = ring[1:]
            k.paint(a, f & dn, k.BRONZE)
        else:
            k.paint(a, f, k.BRONZE_D)
        m = m & ~f
    track = m
    k.paint(a, track, k.WELL_D)
    top = np.zeros_like(track)
    top[1:] = ~track[:-1]
    k.paint(a, track & top, k.PANEL_DEEP)
    if fill:
        light, base, dark = k.BAR_FILLS[fill]
        cut = track.copy()
        rim = 1 + frame_px
        cut[:, rim + int(round((w - 2 * rim) * share)):] = False
        k.paint(a, cut, base)
        dn = np.zeros_like(track)
        dn[:-1] = ~track[1:]
        k.paint(a, cut & top, light)
        k.paint(a, cut & dn & ~top, dark)
    return a, 1 + frame_px


def raised_badge(mask):
    """A badge built like kit 2's buttons (outline, 1 px light, the face, 2 px shade, outline: the
    capitals get an even leftover in an even height), in the bars' wood: light edge, wood face,
    dark shade."""
    h, w = mask.shape
    a = k.canvas(w, h)
    k.raised(a, mask, k.BRONZE_HI, k.BRONZE_L, k.BRONZE_D)
    return a


def hexagon(w, h):
    """A hexagon standing on a point-less side: flat top and bottom, the sides stepping in 1 px a
    row over the top and bottom quarter."""
    m = np.ones((h, w), bool)
    q = h // 4
    for y in range(q):
        inset = q - y
        m[y, :inset] = m[y, w - inset:] = False
        m[h - 1 - y, :inset] = m[h - 1 - y, w - inset:] = False
    return m


def circle(d):
    yy, xx = np.mgrid[0:d, 0:d]
    r = d / 2
    return (xx + 0.5 - r) ** 2 + (yy + 0.5 - r) ** 2 <= r * r


def plate(w, h):
    """A HUD plate: the bars' frame (outline, 2 px wood) round the panel's dark."""
    a, _ = hud_bar(w, h, None, 1.0, 2)
    m = np.all(a[..., :3] == k.WELL_D, -1) | np.all(a[..., :3] == k.PANEL_DEEP, -1)
    a[m, :3] = k.PANEL
    return a


# ---------------------------------------------------------------- a corner being drawn
class Corner:
    def __init__(self, w, h):
        self.a = k.canvas(w, h)
        self.boxes = []  # (name, x, y, w, h, may_overlap: set of names)
        self.labels = []  # (name, text, face_top, face_rows, cap_top, align)

    def put(self, name, img, x, y, over=()):
        h, w = img.shape[:2]
        k.blit(self.a, img, x, y)
        self.boxes.append((name, x, y, w, h, set(over)))

    def label(self, name, s, x0, face_top, face_rows, color, align="center", x1=None, right=None, parent=None):
        """`s` centred on its capitals in the face rows [face_top, face_top + face_rows), the odd
        leftover rounded down (as the game's layout); centred in [x0, x1], or ending at `right`."""
        cap_top = face_top + (face_rows - k.F.caps) // 2
        w = k.F.width(s)
        if right is not None:
            x = right - w
        elif align == "center":
            x = x0 + (x1 - x0 - w) // 2
        else:
            x = x0
        k.F.draw(self.a, s, x, cap_top, color, SHADOW)
        self.labels.append((name, s, face_top, face_rows, cap_top))
        self.boxes.append((name + " text", x, cap_top, w, k.F.caps, {parent or name}))

    def bars(self, x, y, health=(31, 53), stamina=(73, 73)):
        for i, (name, fill, (v, m)) in enumerate((("health", "health", health), ("stamina", "stamina", stamina))):
            b, rim = hud_bar(BAR_W, BAR_H, fill, v / m)
            yy = y + i * (BAR_H + BAR_GAP)
            self.put(name, b, x, yy)
            self.label(name, f"{v} / {m}", 0, yy + rim, BAR_H - 2 * rim, k.CREAM, right=x + BAR_W - 6)

    def lint(self):
        """Problems: a box outside the corner, two boxes overlapping that aren't allowed to, two
        closer than 2 px, a label whose room above and below its capitals differs by more than the
        layout's rounding."""
        out = []
        H, W = self.a.shape[:2]
        for n, x, y, w, h, _ in self.boxes:
            if x < 0 or y < 0 or x + w > W or y + h > H:
                out.append(f"{n} outside the corner")
        for i, (n1, x1, y1, w1, h1, o1) in enumerate(self.boxes):
            for n2, x2, y2, w2, h2, o2 in self.boxes[i + 1:]:
                if n2 in o1 or n1 in o2:
                    continue
                gap_x = max(x2 - (x1 + w1), x1 - (x2 + w2))
                gap_y = max(y2 - (y1 + h1), y1 - (y2 + h2))
                if gap_x < 0 and gap_y < 0:
                    out.append(f"{n1} overlaps {n2}")
                elif max(gap_x, gap_y) < 2:
                    out.append(f"{n1} and {n2} {max(gap_x, gap_y)} px apart")
        for n, s, top, rows, cap in self.labels:
            above, below = cap - top, top + rows - (cap + k.F.caps)
            if abs(above - below) > 1 or (above != below and (rows - k.F.caps) % 2 == 0):
                out.append(f"{n} '{s}': {above} above, {below} below")
        return out

    def measures(self):
        return [f"{n} '{s}': {cap - top} above / {top + rows - cap - k.F.caps} below" for n, s, top, rows, cap in self.labels]


# ---------------------------------------------------------------- the four variants
LEVEL, XP, NEXT, POINTS = 7, 98, 146, 6


def now():
    """What the game draws today: the tall shield, the bars, the thin XP bar and "+6" beside it."""
    c = Corner(300, 64)
    shield = np.array(Image.open(os.path.join(k.REPO, "mods/base/assets/textures/gui/hud_badge.png")).convert("RGBA"))
    c.put("shield", shield, 2, 2)
    c.label("shield", str(LEVEL), 2, 2 + 15, 17, k.CREAM, x1=42)
    c.bars(48, 2)
    xp, _ = hud_bar(BAR_W, XP_H, "xp", XP / NEXT, 1)
    c.put("xp", xp, 48, 2 + 2 * (BAR_H + BAR_GAP))
    c.label("points", f"+{POINTS}", 48 + BAR_W + 6, 42, 17, k.CREAM, align="left")
    return c


def variant_a():
    """A: one plate. A hex badge set into a framed plate with the bars, the XP bar inside it under
    the bars, its numbers beside it; the points as a chip after the plate."""
    pw, ph = 4 + 40 + 4 + BAR_W + 4, 4 + 2 * BAR_H + BAR_GAP + 4 + XP_H + 4
    c = Corner(pw + 4 + 40, ph + 4)
    c.put("plate", plate(pw, ph), 0, 0, over={"hex", "health", "stamina", "xp", "health text", "stamina text", "hex text"})
    hx = hexagon(40, ph - 8)
    c.put("hex", raised_badge(hx), 4, 4, over={"plate"})
    c.label("hex", str(LEVEL), 4, 4 + 2, ph - 8 - 5, k.ON_ORANGE, x1=44)
    c.bars(48, 4)
    xp, _ = hud_bar(BAR_W, XP_H, "xp", XP / NEXT, 1)
    c.put("xp", xp, 48, 4 + 2 * BAR_H + BAR_GAP + 4, over={"plate"})
    chip = k.chip_r(f"+{POINTS}", "points", 20)
    c.put("points chip", chip, pw + 4, (ph - 20) // 2)
    return c


def variant_b():
    """B: a square badge as tall as the two bars, built like a button (its face 35 rows: the level
    13 above and 13 below), the XP bar under badge and bars, as wide as both; the points as a chip
    at its end."""
    side = 2 * BAR_H + BAR_GAP
    chip = k.chip_r(f"+{POINTS}", "points", 20)
    xp_w = side + 4 + BAR_W
    c = Corner(xp_w + 4 + chip.shape[1], side + 3 + 20)
    sq = k.shape(side, side, k.R)
    c.put("badge", raised_badge(sq), 0, 0)
    c.label("badge", str(LEVEL), 0, 2, side - 5, k.ON_ORANGE, x1=side)
    c.bars(side + 4, 0)
    xp, _ = hud_bar(xp_w, XP_H, "xp", XP / NEXT, 1)
    xy = side + 3 + (20 - XP_H) // 2
    c.put("xp", xp, 0, xy)
    c.put("points chip", chip, xp_w + 4, side + 3)
    return c


def variant_c():
    """C: no badge; the XP as a third bar like the others, the level at its left and the XP at its
    right inside it; the points as a chip after it."""
    chip = k.chip_r(f"+{POINTS}", "points", 20)
    # the chip (20, its face 15 rows: 3 and 3) top-aligned with the XP bar (19)
    c = Corner(BAR_W + 4 + chip.shape[1], 3 * BAR_H + 2 * BAR_GAP + 1)
    c.bars(0, 0)
    y = 2 * (BAR_H + BAR_GAP)
    xp, rim = hud_bar(BAR_W, BAR_H, "xp", XP / NEXT)
    c.put("xp", xp, 0, y)
    c.label("xp level", f"Level {LEVEL}", 6, y + rim, BAR_H - 2 * rim, k.CREAM, align="left", parent="xp")
    c.label("xp", f"{XP} / {NEXT}", 0, y + rim, BAR_H - 2 * rim, k.CREAM, right=BAR_W - 6)
    c.put("points chip", chip, BAR_W + 4, y)
    return c


def variant_d():
    """D: a round badge over the bars' left ends (built like a button: its face 39 rows, the level
    15 above and 15 below), the XP bar thin under the bars, the points as a chip at its end."""
    d = 44
    chip = k.chip_r(f"+{POINTS}", "points", 20)
    bx = d - 10
    c = Corner(bx + BAR_W + 4 + chip.shape[1], max(d, 2 + 2 * BAR_H + BAR_GAP + 3 + 20))
    c.bars(bx, 2)
    c.put("round", raised_badge(circle(d)), 0, 0, over={"health", "stamina"})
    c.label("round", str(LEVEL), 0, 2, d - 5, k.ON_ORANGE, x1=d)
    xp, _ = hud_bar(BAR_W - 10, XP_H, "xp", XP / NEXT, 1)
    xy = 2 + 2 * BAR_H + BAR_GAP + 3
    c.put("xp", xp, bx + 10, xy + (20 - XP_H) // 2)
    c.put("points chip", chip, bx + BAR_W + 4, xy)
    return c


# ---------------------------------------------------------------- the sheet
def text(img, s, x, y, scale=2, color=k.CREAM):
    t = k.canvas(k.F.width(s) + 4, k.F.line + 4)
    k.F.draw(t, s, 0, 2, color, SHADOW)
    im = Image.fromarray(t).resize(((k.F.width(s) + 4) * scale, (k.F.line + 4) * scale), Image.NEAREST)
    img.paste(im, (x, y), im)


def on_ground(c, bg):
    h, w = c.a.shape[:2]
    g = bg.crop((0, 0, w + 16, h + 16)).convert("RGBA")
    g.alpha_composite(Image.fromarray(c.a), (8, 8))
    return g.convert("RGB")


VARIANTS = [
    ("Now", "the tall shield, the thin XP bar, \"+6\" loose beside it", now),
    ("A: one plate", "a wood hex badge set into one framed plate with the bars and the XP bar; the points as a chip", variant_a),
    ("B: square badge", "a wood badge as tall as the two bars; the XP bar under badge and bars; the points as a chip", variant_b),
    ("C: three bars (my pick)", "no badge: the XP is a third bar like the others, the level and XP inside it; the points as a chip", variant_c),
    ("D: round badge", "a round wood badge over the bars' left ends, the XP bar under them; the points as a chip", variant_d),
]

if __name__ == "__main__":
    bg = Image.open(BG).convert("RGB") if BG else Image.new("RGB", (400, 120), (70, 96, 60))
    rows = []
    report = []
    for name, about, f in VARIANTS:
        c = f()
        problems = c.lint() if name != "Now" else []
        rows.append((name, about, on_ground(c, bg), problems))
        report.append((name, problems, c.measures()))
    Z = 3
    W = max(2300, 40 + max(r[2].width for r in rows) * (Z + 1) + 60)
    H = 140 + sum(r[2].height * Z + 120 for r in rows)
    sheet = Image.new("RGB", (W, H), (26, 19, 19))
    text(sheet, "The HUD's level corner, four ways, in the Valheim kit: at GUI scale 2 (1x) and zoomed 3x.", 20, 14, 3, k.GOLD_TEXT)
    text(sheet, "Health and stamina stay as in the game. Measured: every number centred on its capitals; no box outside, overlapping or under 2 px apart.", 20, 66, 2, k.DIM)
    y = 120
    for name, about, img, problems in rows:
        text(sheet, f"{name}: {about}", 20, y, 2, k.GOLD_TEXT)
        sheet.paste(img, (20, y + 40))
        z = img.resize((img.width * Z, img.height * Z), Image.NEAREST)
        sheet.paste(z, (40 + img.width, y + 40))
        verdict = "lint: 0 problems" if not problems else "lint: " + "; ".join(problems)
        if name != "Now":
            text(sheet, verdict, 20, y + 46 + img.height, 2, k.GREEN if not problems else (240, 150, 90))
        y += 40 + z.height + 60
    sheet.crop((0, 0, W, y)).save(OUT)
    for name, problems, m in report:
        print(name, "problems:", problems or 0)
        for line in m:
            print("   ", line)
    print(OUT)
