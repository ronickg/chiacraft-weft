"""The HUD's level, XP and points to place, round 2 (Ronald, 11-04, on round 1's four corners:
"tbh none, have a look at other games for some ideas"). A strip of other games' HUDs (real
screenshots, `hud-level/refs/NOTES.md`) with what works in each, then four directions built from
them in the Valheim kit, drawn into a real 1280 x 720 shot of the game at GUI scale 2 (one UI pixel a
screen pixel): the shield, the XP bar and "+6" taken out, the top left moved up into their room,
and the new parts drawn where they go.

Every new part is recorded and checked (`lint`): inside the screen, nothing overlapping what it
shouldn't, at least 3 px from its neighbours, every label centred on its capitals.

Usage: python3 mock_level_hud2.py SHOT.png REFS_DIR OUT.png"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "kit2"))
import kit2 as k  # noqa: E402
from mock_level_hud import hud_bar, raised_badge, SHADOW  # noqa: E402

SHOT, REFS, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
LEVEL, XP, NEXT, POINTS = 7, 98, 146, 6

# Where today's parts are in the shot (measured on the outlines, PC run b9_world, 01_hud_day).
SHIELD = (0, 0, 52, 72)
BARS = (54, 8, 246, 48)         # health and stamina with their numbers
XP_ROW = (52, 50, 330, 64)      # the XP bar and "+6"
BELOW = (0, 64, 330, 200)       # food tiles, nutrients, thermometer, effects
HOTBAR = (458, 674, 820, 718)
STATS_ROW = (8, 350, 100, 372)  # the "P Stats" shortcut
WORLD = (130, 400, 460, 600)    # a patch of plain world to cover what moves


class Screen:
    def __init__(self, base):
        self.img = base.copy()
        self.boxes = []
        self.labels = []

    def cover(self, box):
        """Paints over `box` with plain world (the moved part's old place)."""
        x1, y1, x2, y2 = box
        patch = self.base_world.crop((WORLD[0], WORLD[1], WORLD[0] + x2 - x1, WORLD[1] + y2 - y1))
        self.img.paste(patch, (x1, y1))

    def move(self, src, box, to):
        self.img.paste(src.crop(box), to)

    def put(self, name, a, x, y, over=()):
        im = Image.fromarray(a)
        self.img.paste(im, (x, y), im)
        self.boxes.append((name, x, y, a.shape[1], a.shape[0], set(over)))

    def label(self, name, s, face_top, face_rows, color, x=None, centre=None, right=None, parent=None, img=None):
        cap_top = face_top + (face_rows - k.F.caps) // 2
        w = k.F.width(s)
        if right is not None:
            x = right - w
        elif centre is not None:
            x = centre[0] + (centre[1] - centre[0] - w) // 2
        t = np.array(self.img.convert("RGBA"))
        k.F.draw(t, s, x, cap_top, color, SHADOW)
        self.img = Image.fromarray(t).convert("RGB")
        self.labels.append((name, s, face_top, face_rows, cap_top))
        self.boxes.append((name + " text", x, cap_top, w, k.F.caps, {parent or name}))

    def lint(self, keep_out=()):
        """New parts: inside the screen, clear of `keep_out` (the HUD's other parts, as boxes) and
        of each other by 3 px unless one is allowed over the other, labels centred."""
        out = []
        W, H = self.img.size
        others = [(n, x1, y1, x2 - x1, y2 - y1, set()) for n, (x1, y1, x2, y2) in keep_out]
        for n, x, y, w, h, _ in self.boxes:
            if x < 0 or y < 0 or x + w > W or y + h > H:
                out.append(f"{n} off the screen")
        allb = self.boxes + others
        for i, (n1, x1, y1, w1, h1, o1) in enumerate(self.boxes):
            for n2, x2, y2, w2, h2, o2 in allb[i + 1:]:
                if n2 in o1 or n1 in o2:
                    continue
                gx = max(x2 - (x1 + w1), x1 - (x2 + w2))
                gy = max(y2 - (y1 + h1), y1 - (y2 + h2))
                if gx < 0 and gy < 0:
                    out.append(f"{n1} overlaps {n2}")
                elif max(gx, gy) < 3:
                    out.append(f"{n1} and {n2} {max(gx, gy)} px apart")
        for n, s, top, rows, cap in self.labels:
            above, below = cap - top, top + rows - cap - k.F.caps
            if abs(above - below) > 1 or (above != below and (rows - k.F.caps) % 2 == 0):
                out.append(f"{n} '{s}': {above} above, {below} below")
        return out

    def measures(self):
        return [f"{n} '{s}': {cap - top} above / {top + rows - cap - k.F.caps} below" for n, s, top, rows, cap in self.labels]


def chip(label, kind="plain"):
    return k.chip_r(label, kind, 20)


def glow_ring(a):
    """The chip's outline in the glow (selected / calling for you): the pulse's bright frame."""
    b = a.copy()
    ring = np.all(b[..., :3] == k.OUTLINE, -1) & (b[..., 3] > 0)
    b[ring, :3] = k.GLOW
    return b


def top_left_without_level(s, src):
    """The bars to the left edge, the shield and XP row gone, the rest moved up into their room."""
    s.cover((0, 0, 330, 200))
    s.move(src, BARS, (8, 8))
    s.move(src, BELOW, (0, 64 - 14))


# ---------------------------------------------------------------- directions
def above_hotbar(base):
    """1: Minecraft's place. A slim framed XP bar just above the hotbar, as wide as it, the level
    on a wood chip centred on it; the points as an orange chip at the bar's right end that pulses
    (its outline glows every other half second: drawn here lit)."""
    s = Screen(base)
    s.base_world = base
    top_left_without_level(s, base)
    x1, y1, x2, y2 = HOTBAR
    w = x2 - x1
    bar, _ = hud_bar(w, 8, "xp", XP / NEXT, 1)
    by = y1 - 10 - 8
    s.put("xp bar", bar, x1, by)
    lv = chip(str(LEVEL))
    cx = x1 + (w - lv.shape[1]) // 2
    cy = by + 4 - 10
    s.put("level chip", lv, cx, cy, over={"xp bar"})
    pts = glow_ring(chip(f"+{POINTS}", "points"))
    s.put("points chip", pts, x2 + 4, cy)
    return s, [("hotbar", HOTBAR)], (x1 - 20, cy - 20, x2 + 60, y2 + 2)


def top_centre(base):
    """2: Forager's place. One bar like health and stamina at the top centre, the level at its left
    and the XP at its right inside it; the points as a chip after it, pulsing."""
    s = Screen(base)
    s.base_world = base
    top_left_without_level(s, base)
    w = 240
    x = (1280 - w) // 2
    bar, rim = hud_bar(w, 19, "xp", XP / NEXT)
    s.put("xp bar", bar, x, 8)
    s.label("level", f"Level {LEVEL}", 8 + rim, 19 - 2 * rim, k.CREAM, x=x + 6, parent="xp bar")
    s.label("xp", f"{XP} / {NEXT}", 8 + rim, 19 - 2 * rim, k.CREAM, right=x + w - 6, parent="xp bar")
    s.put("points chip", glow_ring(chip(f"+{POINTS}", "points")), x + w + 4, 8)
    return s, [("clock", (1192, 6, 1274, 66))], (x - 40, 0, x + w + 60, 40)


def menu_only(base):
    """3: Valheim's and Don't Starve's way. No level or XP on the HUD: the "P Stats" shortcut
    carries the level as a chip and, while there are points to place, the orange points chip
    (pulsing); the XP shows on the stats screen, and the level up's banner stays."""
    s = Screen(base)
    s.base_world = base
    top_left_without_level(s, base)
    x1, y1, x2, y2 = STATS_ROW
    lv = chip(f"Level {LEVEL}")
    s.put("level chip", lv, x2 + 4, y1 + (y2 - y1 - 20) // 2)
    s.put("points chip", glow_ring(chip(f"+{POINTS}", "points")), x2 + 4 + lv.shape[1] + 4, y1 + (y2 - y1 - 20) // 2)
    return s, [("stats shortcut", STATS_ROW), ("recipes shortcut", (8, 310, 116, 332)), ("advancements shortcut", (8, 390, 158, 412))], (0, 300, 260, 420)


def level_tile(base):
    """4: a level tile at the hotbar's left, as big as a hotbar slot, in wood: the level in it and
    the XP filling it from the bottom in purple; the points as the orange chip on its top-right
    corner, pulsing."""
    s = Screen(base)
    s.base_world = base
    top_left_without_level(s, base)
    x1, y1, x2, y2 = HOTBAR
    side = 40
    tx, ty = x1 - 6 - side, y1 + (y2 - y1 - side) // 2
    m = k.shape(side, side, k.R)
    tile = raised_badge(m)
    # the XP filling it: the face's rows from the bottom, in the XP bar's purple, light on top
    light, base_c, dark = k.BAR_FILLS["xp"]
    face = np.all(tile[..., :3] == k.BRONZE_L, -1)
    rows = np.nonzero(face.any(1))[0]
    fill_rows = int(round(len(rows) * XP / NEXT))
    filled = face.copy()
    filled[: rows.max() + 1 - fill_rows] = False
    k.paint(tile, filled, base_c)
    top_row = rows.max() + 1 - fill_rows
    k.paint(tile, filled & (np.arange(side)[:, None] == top_row), light)
    s.put("level tile", tile, tx, ty)
    s.label("level tile", str(LEVEL), ty + 2, side - 5, k.ON_ORANGE, centre=(tx, tx + side))
    pts = glow_ring(chip(f"+{POINTS}", "points"))
    s.put("points chip", pts, tx + side - pts.shape[1], ty - 14, over={"level tile"})
    return s, [("hotbar", HOTBAR)], (tx - 30, ty - 30, x2 + 4, y2 + 2)


DIRECTIONS = [
    ("1: above the hotbar (my pick)", "the XP as a slim bar on the hotbar, the level centred on it (Minecraft); the points chip at its end, pulsing", above_hotbar),
    ("2: top centre", "one bar like health and stamina, \"Level 7\" and the XP inside it (Forager); the points chip after it, pulsing", top_centre),
    ("3: only in the menu", "no level on the HUD: the Stats shortcut carries the level and, while there are points, the pulsing chip (Valheim, Don't Starve, Enshrouded)", menu_only),
    ("4: a level tile", "a wood tile at the hotbar's left, the level in it, the XP filling it from below; the points chip on its corner, pulsing", level_tile),
]

# ---------------------------------------------------------------- references
REF_CROPS = [
    ("Minecraft", "minecraft_1.png", (495, 628, 866, 662), "the level on a slim bar right above the hotbar: one thing, where the eye already is"),
    ("ARK (extended HUD)", "ark_1.png", (0, 100, 495, 132), "the level as plain text in the corner, only with the extended HUD on (H)"),
    ("ARK", "ark_2.png", (1855, 700, 1915, 1075), "an XP icon heads the stat column; points are spent with + beside each stat in the inventory"),
    ("Stardew Valley", "stardew_1.png", (205, 35, 825, 360), "no level on the HUD: skills in a menu tab, the pips bigger at the levels with a choice"),
    ("World of Warcraft (classic)", "wow_1.jpg", None, "a full-width, thin, segmented XP strip along the bottom"),
    ("World of Warcraft", "wow_2.png", None, "the level number on the player's portrait frame"),
    ("Forager", "forager_1.jpg", (256, 4, 850, 36), "one bold bar at the top centre: \"LEVEL 44\" and the exact XP written in it"),
    ("Enshrouded", "enshrouded_1.jpg", (30, 120, 260, 230), "points to spend: a gold emblem with the count on the skills page, a red ! on its tab"),
]


def text(img, s, x, y, scale=2, color=k.CREAM):
    t = k.canvas(k.F.width(s) + 4, k.F.line + 4)
    k.F.draw(t, s, 0, 2, color, SHADOW)
    im = Image.fromarray(t).resize(((k.F.width(s) + 4) * scale, (k.F.line + 4) * scale), Image.NEAREST)
    img.paste(im, (x, y), im)


def fit(im, w, h):
    r = min(w / im.width, h / im.height)
    return im.resize((max(1, int(im.width * r)), max(1, int(im.height * r))), Image.LANCZOS)


if __name__ == "__main__":
    base = Image.open(SHOT).convert("RGB")
    W = 2300
    sheet = Image.new("RGB", (W, 6000), (26, 19, 19))
    text(sheet, "Level, XP and points to place, round 2: how other games show them, and four directions from them.", 20, 14, 3, k.GOLD_TEXT)
    text(sheet, "Other games (real screenshots; Valheim's and Diablo's wikis refused the download, and Terraria, Vintage Story and V Rising have no XP on the HUD).", 20, 66, 2, k.DIM)
    y = 110
    # the references, four to a row
    cw, ch = 540, 200
    for i, (game, f, box, works) in enumerate(REF_CROPS):
        col, row = i % 4, i // 4
        x0, y0 = 20 + col * (cw + 30), y + row * (ch + 160)
        im = Image.open(os.path.join(REFS, f)).convert("RGB")
        if box:
            im = im.crop(box)
        im = fit(im, cw, ch)
        sheet.paste(im, (x0, y0 + 36))
        text(sheet, game, x0, y0, 2, k.GOLD_TEXT)
        # the line under, wrapped at the column's width
        words, line, ly = works.split(), "", y0 + 44 + ch
        for wd in words:
            if k.F.width(line + " " + wd) * 2 > cw:
                text(sheet, line.strip(), x0, ly, 2, k.CREAM)
                line, ly = wd, ly + 34
            else:
                line += " " + wd
        text(sheet, line.strip(), x0, ly, 2, k.CREAM)
    y += 2 * (ch + 160) + 20
    text(sheet, "Four directions, drawn into a real 1280 x 720 shot (GUI scale 2): the shield, the XP bar and \"+6\" taken out, the top left moved up.", 20, y, 2, k.DIM)
    y += 50
    report = []
    for name, about, f in DIRECTIONS:
        s, keep_out, zoom_box = f(base)
        problems = s.lint(keep_out)
        report.append((name, problems, s.measures()))
        text(sheet, f"{name}: {about}", 20, y, 2, k.GOLD_TEXT)
        half = s.img.resize((640, 360), Image.LANCZOS)
        sheet.paste(half, (20, y + 40))
        z = s.img.crop(zoom_box)
        Z = max(1, min(3, (W - 700) // z.width, 360 // z.height))
        z = z.resize((z.width * Z, z.height * Z), Image.NEAREST)
        sheet.paste(z, (680, y + 40))
        text(sheet, f"the whole screen, half size; right: {Z}x, the new parts", 20, y + 410, 2, k.DIM)
        text(sheet, "lint: 0 problems" if not problems else "lint: " + "; ".join(problems), 20, y + 444, 2,
             k.GREEN if not problems else (240, 150, 90))
        s.img.save(os.path.join(os.path.dirname(OUT), "level2_" + name.split(":")[0] + ".png"))
        y += 500
    sheet.crop((0, 0, W, y)).save(OUT)
    for name, problems, m in report:
        print(name, "problems:", problems or 0, "|", "; ".join(m))
    print(OUT)
