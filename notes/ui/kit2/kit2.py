"""The UI kit, redrawn on one pixel grid (Ronald: "ui better but still seems weird off in places
maybe read up on how one does ui like this as well"). Everything here is drawn in code at 1 UI px
(half a GUI px, the font's and the 32 px items' pixel), never scaled by a fraction:

- icons: 16 x 16 masks drawn by hand, coloured from one palette per material, with a 1 px outline
  and light from the top left added the same way for all of them, so they share one weight;
- buttons, toggles, tabs, bars, panels, slots, chips and keys: flat faces, a 1 px outline, a 1 px
  light top edge and a darker bottom edge (2 px), no gradients;
- text: the game's baked pixel font, its capitals centred in the face (not its line box), with a
  1 px shadow straight down in the surface's own dark colour.

Used by `components_sheet.py` (the sheet for Ronald) and, once approved, to write the game's gui
textures."""
import os

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))

# ---------------------------------------------------------------- palette
# Ronald's pick, 2026-11-04 ("lets go valheim look its good"): variant 2 of
# `notes/ui/palettes.md`, Valheim's quiet dark wood with an orange accent. Near-black brown
# insides, slots and wells a step lighter, a mid-brown wood frame, dark leather-brown buttons, beige
# text, orange for titles, hover and the selected thing. Every shade here is the one
# `variants.palette` drew on the sheet he picked from.
OUTLINE = (14, 10, 7)
INK_DARK = (58, 36, 22)
PANEL = (31, 23, 18)
PANEL_DEEP = (19, 14, 10)
WELL, WELL_D, WELL_L = (41, 31, 24), (31, 23, 17), (57, 45, 36)
SLIDER, SLIDER_D, SLIDER_L = (74, 53, 36), (56, 39, 26), (96, 71, 51)
BRONZE_D, BRONZE, BRONZE_L, BRONZE_HI = (73, 48, 28), (110, 76, 48), (134, 94, 62), (160, 116, 80)
SLOT, SLOT_D, SLOT_L = (51, 40, 31), (40, 31, 23), (72, 58, 47)
CREAM, DIM, GOLD_TEXT, ON_ORANGE = (232, 214, 176), (180, 156, 118), (255, 184, 74), (255, 240, 207)
GLOW, GLOW_D = (255, 161, 60), (217, 134, 45)
GREEN, GREEN_D, GREEN_L = (138, 200, 15), (85, 125, 10), (193, 237, 66)
RED_UI = (216, 34, 30)

# Button faces: (light edge, face, shade) per kind and state.
FACES = {
    "main": ((144, 90, 47), (122, 74, 36), (83, 48, 20)),
    "main_hover": ((158, 99, 54), (137, 84, 43), (88, 51, 22)),
    "main_pressed": ((85, 49, 21), (104, 61, 28), (122, 74, 36)),
    "second": ((68, 50, 37), (58, 42, 30), (39, 28, 19)),
    "second_hover": ((75, 56, 42), (65, 48, 35), (42, 29, 20)),
    "second_pressed": ((41, 29, 20), (49, 35, 25), (58, 42, 30)),
    "disabled": ((58, 53, 50), (48, 44, 41), (36, 33, 30)),
    "danger": ((208, 59, 48), (176, 44, 34), (120, 23, 15)),
}
TEXT_ON = {"main": ON_ORANGE, "second": CREAM, "disabled": (143, 133, 124), "danger": ON_ORANGE}

# Icon materials: (light, base, dark).
MAT = {
    "cream": ((250, 234, 196), (232, 201, 138), (176, 128, 72)),
    "steel": ((226, 232, 238), (174, 184, 196), (108, 116, 130)),
    "brass": ((240, 190, 100), (204, 140, 56), (140, 86, 30)),
    "wood": ((176, 116, 64), (134, 82, 42), (88, 52, 26)),
    "red": ((246, 104, 84), (210, 50, 40), (138, 24, 20)),
    "blue": ((130, 190, 246), (62, 134, 214), (30, 80, 150)),
    "pink": ((252, 170, 198), (230, 112, 152), (166, 58, 98)),
    "white": ((255, 252, 244), (228, 222, 206), (168, 158, 140)),
    "green": (GREEN_L, GREEN, GREEN_D),
    "roof": ((214, 96, 60), (172, 64, 36), (110, 38, 22)),
    "dark": ((120, 84, 60), (84, 56, 38), (56, 36, 24)),
}


# ---------------------------------------------------------------- the font
class Font:
    def __init__(self, d=os.path.join(REPO, "mods/base/assets/fonts")):
        self.atlas = np.array(Image.open(os.path.join(d, "default.png")).convert("L"))
        self.g = {}
        for line in open(os.path.join(d, "default.font")):
            key, _, v = line.partition(" = ")
            if key == "cell":
                self.cw, self.ch = map(int, v.split())
            elif key == "baseline":
                self.baseline = int(float(v))
            elif key == "line":
                self.line = int(float(v))
            elif key == "caps":
                self.caps = int(float(v))
            elif line.startswith("g "):
                c, col, row, left, w, bearing, adv = line.split()[1:]
                self.g[chr(int(c, 16))] = tuple(int(float(x)) for x in (col, row, left, w, bearing, adv))

    def width(self, s):
        """The ink's width: no lead-in, no spacing after the last letter (the engine's measure)."""
        if not s:
            return 0
        w = sum(self.g[c][5] for c in s)
        first, last = self.g[s[0]], self.g[s[-1]]
        return w - first[4] - (last[5] - last[4] - last[3])

    def draw(self, img, s, x, cap_top, color, shadow=None, shadow_dx=0):
        """`s` with its capitals' top at `cap_top`; a shadow is 1 px straight down (`shadow_dx` 1
        draws the old down-right one, for comparison only)."""
        missing = [c for c in s if c not in self.g]
        assert not missing, f"no glyph for {missing!r}"
        # The cell's baseline is the row under the capitals' last one: capitals take rows
        # baseline-caps-1 .. baseline-2 of the cell (measured: an "A" drawn at the old offset
        # inked from cap_top-1, so every label sat a pixel high).
        y0 = cap_top - (self.baseline - self.caps - 1)
        lead = self.g[s[0]][4] if s else 0
        for col, dy in ((shadow, 1), (color, 0)):
            if col is None:
                continue
            pen = -lead + (shadow_dx if dy else 0)
            for c in s:
                cl, rw, left, gw, bearing, adv = self.g[c]
                if c != " " and gw > 0:
                    cell = self.atlas[rw * self.ch:(rw + 1) * self.ch, cl * self.cw + left:cl * self.cw + left + gw]
                    ys, xs = np.nonzero(cell > 127)
                    for yy, xx in zip(ys, xs):
                        px, py = x + pen + bearing + xx, y0 + yy + dy
                        if 0 <= px < img.shape[1] and 0 <= py < img.shape[0]:
                            img[py, px] = col + (255,)
                pen += adv
        return self.width(s)


F = Font()


def canvas(w, h, color=None):
    a = np.zeros((h, w, 4), np.uint8)
    if color:
        a[..., :3] = color
        a[..., 3] = 255
    return a


def blit(dst, src, x, y):
    h, w = src.shape[:2]
    m = src[..., 3] > 0
    dst[y:y + h, x:x + w][m] = src[m]


def rect(a, x, y, w, h, color):
    a[y:y + h, x:x + w, :3] = color
    a[y:y + h, x:x + w, 3] = 255


# ---------------------------------------------------------------- icons, 16 x 16
# Each mask: '.' empty, a letter is a region painted with that material (see ICON_MATS); the
# outline goes round every region's outside, light on a region's top and left edges, dark on its
# bottom and right ones.
ICONS = {
    "follow": ("cream", [
        "................",
        "....###..###....",
        "....###..###....",
        "....###..###....",
        "....###..###....",
        ".###........###.",
        ".###........###.",
        ".###........###.",
        ".###.######.###.",
        "....########....",
        "...##########...",
        "...##########...",
        "...##########...",
        "....########....",
        ".....######.....",
        "................"]),
    "stay": ("cream", [
        "................",
        "......##........",
        "......##.##.....",
        "...##.##.##.....",
        "...##.##.##.##..",
        "...##.##.##.##..",
        "...##.##.##.##..",
        "...##.##.##.##..",
        ".#############..",
        ".#############..",
        ".#############..",
        "..############..",
        "...###########..",
        "....#########...",
        ".....#######....",
        "................"]),
    "wander": ("cream", [
        "................",
        ".......##.......",
        "......####......",
        ".....######.....",
        ".......##.......",
        "...#...##...#...",
        "..##........##..",
        ".#####.##.#####.",
        ".#####.##.#####.",
        "..##........##..",
        "...#...##...#...",
        ".......##.......",
        ".....######.....",
        "......####......",
        ".......##.......",
        "................"]),
    "guard": ({"r": "roof", "#": "cream", "d": "wood"}, [
        "................",
        ".......rr.......",
        "......rrrr......",
        ".....rrrrrr.....",
        "....rrrrrrrr....",
        "...rrrrrrrrrr...",
        "..rrrrrrrrrrrr..",
        ".rrrrrrrrrrrrrr.",
        "................",
        "..############..",
        "..#####dd#####..",
        "..####dddd####..",
        "..####dddd####..",
        "..####dddd####..",
        "..####dddd####..",
        "................"]),
    "passive": ({"p": "wood", "#": "white"}, [
        "................",
        "..pp............",
        "..pp#######.....",
        "..pp##########..",
        "..pp##########..",
        "..pp##########..",
        "..pp##########..",
        "..pp#########...",
        "..pp######......",
        "..pp............",
        "..pp............",
        "..pp............",
        "..pp............",
        "..pp............",
        ".pppp...........",
        "................"]),
    "neutral": ("steel", [
        "................",
        "..############..",
        "..############..",
        "..############..",
        "..############..",
        "..############..",
        "..############..",
        "...##########...",
        "...##########...",
        "....########....",
        "....########....",
        ".....######.....",
        "......####......",
        ".......##.......",
        "................",
        "................"]),
    "aggressive": ({"#": "steel", "g": "brass", "p": "wood"}, [
        "................",
        "............###.",
        "...........####.",
        "..........####..",
        ".........####...",
        "........####....",
        ".......####.....",
        "......####......",
        "..g..####.......",
        "...g####........",
        "....gg..........",
        "...pgggg........",
        "..ppp.g.........",
        ".ppp............",
        ".pp.............",
        "................"]),
    "assist": ({"#": "red", "w": "white"}, [
        "................",
        ".......##.......",
        ".....######.....",
        "....#......#....",
        "...#..wwww..#...",
        "...#.w....w.#...",
        "..##.w.##.w.##..",
        ".####..##..####.",
        ".####..##..####.",
        "..##.w....w.##..",
        "...#.w....w.#...",
        "...#..wwww..#...",
        "....#......#....",
        ".....######.....",
        ".......##.......",
        "................"]),
    "ride": ({"#": "wood", "b": "brass"}, [
        "................",
        "................",
        "..##........##..",
        "..###......###..",
        "..############..",
        ".##############.",
        ".##############.",
        "..############..",
        "....b......b....",
        "....b......b....",
        "....b......b....",
        "...bbb....bbb...",
        "...b.b....b.b...",
        "...bbb....bbb...",
        "................",
        "................"]),
    "tribe": ({"d": "dark", "#": "cream"}, [
        "................",
        "..........ddd...",
        ".........ddddd..",
        "....###..ddddd..",
        "...#####.ddddd..",
        "...#####..ddd...",
        "...#####........",
        "....###..ddddd..",
        "........ddddddd.",
        "..#######dddddd.",
        ".#########ddddd.",
        ".#########ddddd.",
        ".#########......",
        ".#########......",
        "................",
        "................"]),
    "sniff": ({"#": "steel", "w": "blue", "p": "wood"}, [
        "................",
        "....######......",
        "...########.....",
        "..###wwww###....",
        "..##wwwwww##....",
        "..##wwwwww##....",
        "..##wwwwww##....",
        "..##wwwwww##....",
        "..###wwww###....",
        "...########.....",
        "....######pp....",
        "..........ppp...",
        "...........ppp..",
        "............ppp.",
        ".............pp.",
        "................"]),
    "release": ({"#": "steel", "b": "brass"}, [
        "................",
        ".......#####....",
        "......##...##...",
        "......##...##...",
        "......##...##...",
        "......##........",
        "......##........",
        "..bbbbbbbbbb....",
        "..bbbbbbbbbb....",
        "..bbbbbbbbbb....",
        "..bbbb..bbbb....",
        "..bbbb..bbbb....",
        "..bbbbbbbbbb....",
        "..bbbbbbbbbb....",
        "................",
        "................"]),
    "undo": ("cream", [
        "................",
        "....#...........",
        "...##...........",
        "..##########....",
        ".############...",
        "..##########.##.",
        "...##.......###.",
        "....#........##.",
        ".............##.",
        ".............##.",
        "............###.",
        "...##########...",
        "...#########....",
        "................",
        "................",
        "................"]),
    "rename": ({"#": "brass", "w": "pink", "d": "dark"}, [
        "................",
        "............ww..",
        "...........wwww.",
        "..........#wwww.",
        ".........###ww..",
        "........#####...",
        ".......#####....",
        "......#####.....",
        ".....#####......",
        "....#####.......",
        "...#####........",
        "..d####.........",
        "..dd##..........",
        "..ddd...........",
        "................",
        "................"]),
    "male": ("blue", [
        "................",
        ".........######.",
        ".........######.",
        "............###.",
        "...........####.",
        "....####..###.#.",
        "...######.##..#.",
        "..###..####.....",
        "..##....##......",
        "..##....##......",
        "..###..###......",
        "...######.......",
        "....####........",
        "................",
        "................",
        "................"]),
    "female": ("pink", [
        "................",
        ".....######.....",
        "....########....",
        "...###....###...",
        "...##......##...",
        "...##......##...",
        "...###....###...",
        "....########....",
        ".....######.....",
        ".......##.......",
        ".....######.....",
        ".....######.....",
        ".......##.......",
        ".......##.......",
        "................",
        "................"]),
    "taming": ("red", [
        "................",
        "................",
        "..####....####..",
        ".######..######.",
        ".##############.",
        ".##############.",
        ".##############.",
        "..############..",
        "...##########...",
        "....########....",
        ".....######.....",
        "......####......",
        ".......##.......",
        "................",
        "................",
        "................"]),
    "close": ("cream", [
        "................",
        "................",
        "..##........##..",
        "..###......###..",
        "...###....###...",
        "....###..###....",
        ".....######.....",
        "......####......",
        "......####......",
        ".....######.....",
        "....###..###....",
        "...###....###...",
        "..###......###..",
        "..##........##..",
        "................",
        "................"]),
    "plus": ("cream", [
        "................",
        "................",
        "................",
        "......####......",
        "......####......",
        "......####......",
        "...##########...",
        "...##########...",
        "...##########...",
        "...##########...",
        "......####......",
        "......####......",
        "......####......",
        "................",
        "................",
        "................"]),
    "minus": ("cream", [
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        "...##########...",
        "...##########...",
        "...##########...",
        "...##########...",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................"]),
    "check": ("green", [
        "................",
        "................",
        "............###.",
        "...........####.",
        "..........####..",
        ".........####...",
        "..##....####....",
        "..###..####.....",
        "...#######......",
        "....#####.......",
        ".....###........",
        "................",
        "................",
        "................",
        "................",
        "................"]),
}


def icon(name):
    """A 16 x 16 icon: its regions coloured light, base or dark by their edges, then outlined."""
    mats, rows = ICONS[name]
    if isinstance(mats, str):
        mats = {"#": mats}
    assert len(rows) == 16 and all(len(r) == 16 for r in rows), name
    g = np.array([list(r) for r in rows])
    a = canvas(16, 16)
    inside = g != "."
    for ch, mat in mats.items():
        m = g == ch
        light, base, dark = MAT[mat]
        for y, x in zip(*np.nonzero(m)):
            def same(yy, xx):
                return 0 <= yy < 16 and 0 <= xx < 16 and g[yy, xx] == ch
            if not same(y - 1, x) or not same(y, x - 1):
                c = light
            elif not same(y + 1, x) or not same(y, x + 1):
                c = dark
            else:
                c = base
            a[y, x] = c + (255,)
    out = np.zeros((16, 16), bool)
    for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        out |= np.roll(np.pad(inside, 1), (dy, dx), (0, 1))[1:-1, 1:-1]
    out &= ~inside
    a[out] = OUTLINE + (255,)
    return a


# ---------------------------------------------------------------- buttons, toggles, tabs
def button(w, h=22, kind="main", state="normal", label="", ico=None, align="center", pad=6):
    """A button: a 1 px outline with its corners cut, a light top edge, a face, a 2 px shade.
    Hover brightens the face and lights the outline gold; pressed sinks it (shade on top, content
    1 px down); a selected toggle is the main colour pressed; disabled is flat grey-brown."""
    a = canvas(w, h)
    key = {"normal": kind, "hover": kind + "_hover", "pressed": kind + "_pressed"}.get(state, kind)
    if state == "disabled":
        key = "disabled"
    if kind == "danger":
        key = "danger"
    light, face, shade = FACES[key]
    rect(a, 1, 1, w - 2, h - 2, face)
    pressed = state == "pressed"
    if pressed:
        rect(a, 1, 1, w - 2, 2, light)  # the sunk edge: dark on top
    else:
        rect(a, 1, 1, w - 2, 1, light)
        rect(a, 1, h - 3, w - 2, 2, shade)
    rim = GLOW if state == "hover" else OUTLINE
    rect(a, 1, 0, w - 2, 1, rim)
    rect(a, 1, h - 1, w - 2, 1, rim)
    rect(a, 0, 1, 1, h - 2, rim)
    rect(a, w - 1, 1, 1, h - 2, rim)
    tcol = TEXT_ON["disabled" if state == "disabled" else ("danger" if kind == "danger" else kind)]
    shadow = FACES[key][2] if not pressed else None
    # Content centres on the whole inside (rows 1 .. h-2, the shade included: it reads as part of
    # the face), capitals on capitals, never on the font's line box.
    face_top, face_h = 1, h - 2
    dy = 1 if pressed else 0
    cap_top = face_top + (face_h - F.caps) // 2 + dy
    iw = 16 if ico else 0
    gap = 4 if ico and label else 0
    tw = F.width(label)
    need = pad * 2 + iw + gap + tw
    assert need <= w, f"button {label!r} needs {need} px, has {w}"
    x = pad if align == "left" else (w - (iw + gap + tw)) // 2
    if ico:
        ic = icon(ico)
        if state == "disabled":
            g = ic[..., :3].mean(-1, keepdims=True).astype(np.uint8)
            ic[..., :3] = np.where(ic[..., 3:4] > 0, (g * 0.55 + 40).astype(np.uint8), 0)
        blit(a, ic, x, face_top + (face_h - 16) // 2 + dy)
    if label:
        F.draw(a, label, x + iw + gap, cap_top, tcol, shadow)
    return a


def tab(w, label, active, ico=None, h=22):
    """A tab: an active one is the panel's bronze and opens into it; an inactive one sits 2 px
    lower, darker, with its own bottom edge."""
    a = canvas(w, h)
    top = 0 if active else 2
    light, face, shade = (BRONZE_HI, BRONZE, BRONZE_D) if active else ((110, 72, 46), (78, 50, 32), (58, 36, 24))
    rect(a, 1, top + 1, w - 2, h - top - 1, face)
    rect(a, 1, top + 1, w - 2, 1, light)
    rect(a, 1, top, w - 2, 1, OUTLINE)
    rect(a, 0, top + 1, 1, h - top - 1, OUTLINE)
    rect(a, w - 1, top + 1, 1, h - top - 1, OUTLINE)
    if not active:
        rect(a, 1, h - 1, w - 2, 1, OUTLINE)
    cap_top = top + 1 + (h - top - 2 - F.caps) // 2
    iw = 16 if ico else 0
    gap = 4 if ico else 0
    x = (w - iw - gap - F.width(label)) // 2
    if ico:
        blit(a, icon(ico), x, top + 1 + (h - top - 2 - 16) // 2)
    F.draw(a, label, x + iw + gap, cap_top, ON_ORANGE if active else DIM, shade)
    return a


# ---------------------------------------------------------------- bars, slots, chips, keys
BAR_FILLS = {
    "health": ((246, 104, 84), (210, 50, 40), (138, 24, 20)),
    "stamina": ((238, 170, 84), (214, 132, 44), (150, 86, 24)),
    "food": ((232, 160, 90), (196, 120, 50), (132, 76, 28)),
    "water": ((130, 190, 246), (62, 134, 214), (30, 80, 150)),
    "taming": ((248, 196, 96), (230, 156, 40), (160, 100, 20)),
    "torpor": ((190, 130, 240), (150, 80, 220), (96, 44, 150)),
    "xp": ((190, 130, 240), (155, 79, 224), (96, 44, 150)),
    "q0": ((176, 166, 156), (146, 136, 126), (100, 92, 84)),
    "q1": (GREEN_L, GREEN, GREEN_D),
    "q2": ((130, 190, 246), (47, 141, 228), (24, 80, 150)),
    "q3": ((196, 140, 246), (155, 79, 224), (96, 44, 150)),
    "q4": ((252, 206, 110), (240, 170, 40), (160, 104, 20)),
}


def bar(w, share, fill="health", h=10):
    """A bar: outline, a dark track, the fill with a 1 px light top and dark bottom."""
    a = canvas(w, h, OUTLINE)
    rect(a, 1, 1, w - 2, h - 2, WELL_D)
    rect(a, 1, 1, w - 2, 1, PANEL_DEEP)
    fw = int(round((w - 2) * max(0.0, min(1.0, share))))
    if fw:
        light, base, dark = BAR_FILLS[fill]
        rect(a, 1, 1, fw, h - 2, base)
        rect(a, 1, 1, fw, 1, light)
        rect(a, 1, h - 2, fw, 1, dark)
    return a


def slot(size=36, selected=False):
    """A sunken slot: dark top-left inner edge, light bottom-right; selected adds a 1 px gold rim."""
    a = canvas(size, size, OUTLINE)
    rect(a, 1, 1, size - 2, size - 2, SLOT)
    rect(a, 1, 1, size - 2, 1, SLOT_D)
    rect(a, 1, 1, 1, size - 2, SLOT_D)
    rect(a, 1, size - 2, size - 2, 1, SLOT_L)
    rect(a, size - 2, 1, 1, size - 2, SLOT_L)
    if selected:
        for i in (0,):
            rect(a, i, i, size - 2 * i, 1, GLOW)
            rect(a, i, size - 1 - i, size - 2 * i, 1, GLOW)
            rect(a, i, i, 1, size - 2 * i, GLOW)
            rect(a, size - 1 - i, i, 1, size - 2 * i, GLOW)
    return a


def chip(label, kind="plain", h=18):
    """A chip (a fact, a gene, points to place): 1 px rim, flat face, text centred on its caps."""
    w = F.width(label) + 12
    w += w % 2  # even, so what follows stays on the 2 px rhythm
    face, rim, col = {"plain": ((42, 32, 27), (104, 62, 38), CREAM), "points": ((122, 64, 22), (170, 96, 40), ON_ORANGE),
                      "dim": ((32, 24, 20), (78, 54, 38), DIM)}[kind]
    a = canvas(w, h, OUTLINE)
    rect(a, 1, 1, w - 2, h - 2, rim)
    rect(a, 2, 2, w - 4, h - 4, face)
    a[0, 0, 3] = a[0, -1, 3] = a[-1, 0, 3] = a[-1, -1, 3] = 0
    F.draw(a, label, 6, (h - F.caps) // 2, col, None)
    return a


def keycap(letter, size=20, r=3):
    """A key: light, built like a small button, the letter centred on its face. (Defined before
    the rounded parts' helpers; it looks them up when it is called.)"""
    a = canvas(size, size)
    raised(a, shape(size, size, r), (240, 228, 208), (214, 196, 170), (160, 136, 110))
    cap_top = centre_caps(2, size - 5, "key")
    F.draw(a, letter, (size - F.width(letter)) // 2, cap_top, INK_DARK, None)
    check_label(a, letter, INK_DARK, 2, size - 4, "key")
    return a


# ---------------------------------------------------------------- panels
def panel(w, h):
    """A window: outline, a 2 px bronze frame lit top-left, a 1 px groove, a 1 px inner bronze
    line, then the dark inside; corners carry a 2 x 2 rivet. 6 px of frame each side."""
    a = canvas(w, h, OUTLINE)
    rect(a, 1, 1, w - 2, h - 2, BRONZE)
    rect(a, 1, 1, w - 2, 1, BRONZE_HI)
    rect(a, 1, 1, 1, h - 2, BRONZE_HI)
    rect(a, 1, h - 2, w - 2, 1, BRONZE_D)
    rect(a, w - 2, 1, 1, h - 2, BRONZE_D)
    rect(a, 3, 3, w - 6, h - 6, PANEL_DEEP)  # the groove
    rect(a, 4, 4, w - 8, h - 8, BRONZE_D)  # the inner line
    rect(a, 5, 5, w - 10, h - 10, PANEL)
    for cx, cy in ((1, 1), (w - 3, 1), (1, h - 3), (w - 3, h - 3)):
        rect(a, cx, cy, 2, 2, BRONZE_HI)
        a[cy + 1, cx + 1, :3] = BRONZE_D
    # cut the outer corners
    for cx, cy in ((0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)):
        a[cy, cx, 3] = 0
    return a


INNER = 6  # a panel's frame, each side: content starts 6 px in (plus its own padding)


def divider(w):
    a = canvas(w, 2)
    rect(a, 0, 0, w, 1, PANEL_DEEP)
    rect(a, 0, 1, w, 1, (52, 36, 28))
    return a


# ---------------------------------------------------------------- rounded parts (Ronald: "can we
# make it more rounded less just square ? Like some parts if possible ?")
# Every part below is one shape: a rectangle whose corners step in on the pixel grid by a
# profile (how far each of the corner's rows is cut), outlined 1 px all round, so the outline
# keeps one weight whatever the radius. A radius is a profile name: 0 square, 1 one pixel off
# each corner, 2 and 3 stepped, "pill" a half circle as tall as the part.
PROFILES = {0: [], 1: [1], 2: [2, 1], 3: [3, 1, 1], 4: [4, 2, 1, 1]}


def profile(r, h):
    """The cut of each corner row: a named radius, or a pill (a half circle of the part's height)."""
    if r == "pill":
        rad = h / 2
        out = []
        for i in range(int(rad)):
            dy = rad - (i + 0.5)
            dx = (rad * rad - dy * dy) ** 0.5
            out.append(int(round(rad - dx)))
        while out and out[-1] == 0:
            out.pop()
        return out
    return PROFILES[r]


def shape(w, h, r=0, corners="all"):
    """A w x h mask with its corners stepped in (`corners` "all", or "top" for a tab)."""
    m = np.ones((h, w), bool)
    p = profile(r, h if r == "pill" else min(w, h))
    for i, cut in enumerate(p):
        if cut <= 0:
            continue
        m[i, :cut] = m[i, w - cut:] = False
        if corners == "all":
            m[h - 1 - i, :cut] = m[h - 1 - i, w - cut:] = False
    return m


def edge(m):
    """The mask's own outer ring, 1 px: a pixel with a side neighbour outside it."""
    p = np.pad(m, 1)
    inner = p[1:-1, 1:-1] & p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:]
    return m & ~inner


def shrink(m, n=1):
    for _ in range(n):
        m = m & ~edge(m)
    return m


def paint(a, m, color):
    a[m, :3] = color
    a[m, 3] = 255


def raised(a, m, light, face, shade, rim=OUTLINE, pressed=False, shade_px=2):
    """Paints a raised part in mask `m`: the outline, the face, a 1 px light edge under the top
    outline and a `shade_px` shade over the bottom one, each following the corners' steps.
    Pressed sinks it: the shade goes on top instead and there is no light edge."""
    ring = edge(m)
    inner = m & ~ring
    paint(a, ring, rim)
    paint(a, inner, face)
    up = np.zeros_like(m)
    up[1:] = ring[:-1]  # a pixel whose upper neighbour is the outline
    up &= inner
    down = np.zeros_like(m)
    for k in range(1, shade_px + 1):
        d = np.zeros_like(m)
        d[:-k] = ring[k:]  # the outline k below
        down |= d
    down &= inner
    if pressed:
        paint(a, up | (np.roll(up, 1, 0) & inner), light)  # the sunk edge, 2 px, on top
    else:
        paint(a, down, shade)
        paint(a, up, light)


R = 3  # Ronald, 2026-11-04: "i like c with 3px": every part steps its corners 3 px.
DESCENDER = 2  # PixelOperator's g, p, y reach 2 px under the capitals (measured)


def centre_caps(face_top, face_rows, what=""):
    """Where capitals go in a flat face of `face_rows` rows starting at `face_top`: centred, the
    same room above and below. Refuses a face that leaves an odd pixel (the label could only sit
    half a pixel off, the "not centred" Ronald saw) or no room for descenders."""
    left = face_rows - F.caps
    assert left >= 2 * DESCENDER - 1, f"{what}: a face of {face_rows} rows has no room for descenders"
    assert left % 2 == 0, f"{what}: a face of {face_rows} rows leaves an odd pixel round 9 px capitals"
    return face_top + left // 2


def ink_rows(a, color):
    """The rows a colour's pixels take in a part (for checking where a label really sits)."""
    m = np.all(a[..., :3] == color, axis=-1) & (a[..., 3] > 0)
    ys = np.nonzero(m.any(1))[0]
    return (int(ys.min()), int(ys.max())) if len(ys) else None


CHECKS = []  # (part, label, room above the capitals, room below them, descender clearance)


def check_label(a, label, color, face_top, face_bottom, part):
    """Measures a drawn label: its capitals' room above and below inside the face must match,
    and its descenders must stay inside the face."""
    rows = ink_rows(a, color)
    if not rows or not label.strip():
        return
    top = rows[0]
    cap_bottom = top + F.caps - 1
    above, below = top - face_top, face_bottom - cap_bottom
    has_desc = any(c in "gjpqy" for c in label)
    clear = face_bottom - (cap_bottom + DESCENDER) if has_desc else None
    CHECKS.append((part, label, above, below, clear))
    assert above == below, f"{part} {label!r}: {above} px above its capitals, {below} below"
    assert clear is None or clear >= 0, f"{part} {label!r}: descenders leave the face"


def label_into(a, w, label, ico, tcol, shadow, face_top, face_rows, align="center", pad=6, dy=0, part="button"):
    """An icon and a label, the capitals centred in the face (rows face_top .. +face_rows)."""
    cap_top = centre_caps(face_top, face_rows, part) + dy
    iw = 16 if ico else 0
    gap = 4 if ico and label else 0
    tw = F.width(label)
    need = pad * 2 + iw + gap + tw
    assert need <= w, f"{label!r} needs {need} px, has {w}"
    x = pad if align == "left" else (w - (iw + gap + tw)) // 2
    if ico:
        # The icon's middle on the capitals' middle.
        blit(a, icon(ico), x, cap_top + F.caps // 2 - 8)
    if label:
        F.draw(a, label, x + iw + gap, cap_top, tcol, shadow)
        check_label(a, label, tcol, face_top + dy, face_top + face_rows - 1 + dy, part)


def button_r(w, h=22, kind="main", state="normal", label="", ico=None, r=R, align="center", pad=6):
    """A button: outline, a 1 px light edge, the flat face, a 2 px shade. The label centres on the
    face alone (rows 2 .. h-4): 22 px leaves 17 rows, 4 above and 4 below 9 px capitals."""
    a = canvas(w, h)
    key = {"normal": kind, "hover": kind + "_hover", "pressed": kind + "_pressed"}.get(state, kind)
    if state == "disabled":
        key = "disabled"
    if kind == "danger":
        key = "danger"
    light, face, shade = FACES[key]
    pressed = state == "pressed"
    raised(a, shape(w, h, r), light, face, shade, GLOW if state == "hover" else OUTLINE, pressed)
    tcol = TEXT_ON["disabled" if state == "disabled" else ("danger" if kind == "danger" else kind)]
    label_into(a, w, label, ico, tcol, None if pressed else shade, 2, h - 5, align, pad,
               1 if pressed else 0, f"button {w}x{h}")
    return a


def round_button(size=20, kind="main", state="normal", ico="plus", r=R):
    """A small icon button (+, -), the icon centred on the face."""
    a = canvas(size, size)
    key = {"normal": kind, "hover": kind + "_hover", "pressed": kind + "_pressed"}.get(state, kind)
    if state == "disabled":
        key = "disabled"
    light, face, shade = FACES[key]
    pressed = state == "pressed"
    raised(a, shape(size, size, r), light, face, shade, GLOW if state == "hover" else OUTLINE, pressed)
    ic = icon(ico)
    if state == "disabled":
        g = ic[..., :3].mean(-1, keepdims=True).astype(np.uint8)
        ic[..., :3] = np.where(ic[..., 3:4] > 0, (g * 0.55 + 40).astype(np.uint8), 0)
    # face rows 2 .. size-4 (between the light edge and the shade)
    blit(a, ic, (size - 16) // 2, 2 + (size - 5 - 16) // 2 + (1 if pressed else 0))
    return a


def tab_r(w, label, active, ico=None, h=22, r=R):
    """A tab, its top corners stepped. An active one stands 2 px taller and opens into the panel;
    labels centre on the face between the light edge and where the panel's frame starts."""
    a = canvas(w, h)
    top = 0 if active else 2
    m = np.zeros((h, w), bool)
    m[top:] = shape(w, h - top + (2 if active else 0), r, "top")[:h - top]
    light, face, shade = (BRONZE_HI, BRONZE, BRONZE_D) if active else ((110, 72, 46), (78, 50, 32), (58, 36, 24))
    ring = edge(m)
    if active:
        ring[h - 1] = False  # open at the bottom: it joins the panel's frame
        ring[h - 1, 0] = ring[h - 1, w - 1] = m[h - 1, 0]
    inner = m & ~ring
    paint(a, ring, OUTLINE)
    paint(a, inner, face)
    up = np.zeros_like(m)
    up[1:] = ring[:-1]
    paint(a, up & inner, light)
    # inactive: rows top+2 .. h-3 (17 rows); active: the same 17 rows, 2 px higher
    label_into(a, w, label, ico, ON_ORANGE if active else DIM, shade, top + 2, 17, part="tab")
    return a


def bar_r(w, share, fill="health", h=10, r=R):
    """A bar: outline, a dark track, the fill clipped to the same rounded shape."""
    a = canvas(w, h)
    m = shape(w, h, r)
    ring = edge(m)
    inner = m & ~ring
    paint(a, ring, OUTLINE)
    paint(a, inner, WELL_D)
    top = np.zeros_like(m)
    top[1:] = ring[:-1]
    paint(a, top & inner, PANEL_DEEP)
    fw = int(round(w * max(0.0, min(1.0, share))))
    if fw:
        light, base, dark = BAR_FILLS[fill]
        # The fill is the same shape, as long as the share: one that stops short of the end
        # still has a rounded end.
        em = np.zeros_like(m)
        fw = max(fw, 2 * len(profile(r, h)) + 2)
        em[:, :min(fw, w)] = shape(min(fw, w), h, r)
        f = inner & em & ~edge(em)
        paint(a, f, base)
        up = np.zeros_like(m)
        up[1:] = ~f[:-1]
        paint(a, f & up, light)
        dn = np.zeros_like(m)
        dn[:-1] = ~f[1:]
        paint(a, f & dn & ~up, dark)
    return a


def slot_r(size=36, selected=False, r=R):
    """A sunken slot: dark under its top-left outline, light over its bottom-right one."""
    a = canvas(size, size)
    m = shape(size, size, r)
    ring = edge(m)
    inner = m & ~ring
    paint(a, ring, GLOW if selected else OUTLINE)
    paint(a, inner, SLOT)
    up = np.zeros_like(m)
    up[1:] = ring[:-1]
    lf = np.zeros_like(m)
    lf[:, 1:] = ring[:, :-1]
    dn = np.zeros_like(m)
    dn[:-1] = ring[1:]
    rt = np.zeros_like(m)
    rt[:, :-1] = ring[:, 1:]
    paint(a, (dn | rt) & inner, SLOT_L)
    paint(a, (up | lf) & inner, SLOT_D)
    return a


CHIPS = {"plain": ((90, 73, 61), (56, 44, 35), (70, 56, 45), CREAM),
         "points": ((195, 130, 78), (122, 74, 36), (152, 96, 52), ON_ORANGE),
         "dim": ((72, 58, 48), (45, 35, 27), (56, 45, 35), DIM)}


def chip_r(label, kind="plain", h=20, r=R):
    """A chip (a fact, a gene, points to place), built like a small button: outline, a 1 px light
    edge, the face, a 2 px shade. 20 px leaves a 15-row face: 3 above and 3 below the capitals,
    and descenders 1 px clear of the shade."""
    light, face, shade, col = CHIPS[kind]
    w = F.width(label) + 12
    w += w % 2
    a = canvas(w, h)
    raised(a, shape(w, h, r), light, face, shade)
    cap_top = centre_caps(2, h - 5, "chip")
    F.draw(a, label, (w - F.width(label)) // 2, cap_top, col, None)
    check_label(a, label, col, 2, h - 4, "chip")
    return a


def panel_r(w, h, r=3):
    """A window: outline, a 2 px bronze frame lit top-left, a 1 px groove, a 1 px inner bronze
    line, the dark inside; each ring steps in with the corners, one radius smaller each time."""
    a = canvas(w, h)
    m0 = shape(w, h, r)
    paint(a, m0, OUTLINE)
    m1 = shrink(m0) & shape(w, h, r)  # inside the outline
    paint(a, m1, BRONZE)
    # light on the frame's top and left, dark on its bottom and right
    up = np.zeros_like(m1)
    up[1:] = ~m1[:-1]
    lf = np.zeros_like(m1)
    lf[:, 1:] = ~m1[:, :-1]
    dn = np.zeros_like(m1)
    dn[:-1] = ~m1[1:]
    rt = np.zeros_like(m1)
    rt[:, :-1] = ~m1[:, 1:]
    paint(a, m1 & (dn | rt), BRONZE_D)
    paint(a, m1 & (up | lf), BRONZE_HI)
    m3 = shrink(m0, 3)
    paint(a, m3, PANEL_DEEP)
    m4 = shrink(m0, 4)
    paint(a, m4, BRONZE_D)
    m5 = shrink(m0, 5)
    paint(a, m5, PANEL)
    return a
