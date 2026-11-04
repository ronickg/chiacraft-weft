"""Bakes the game's text font (notes/STYLE.md, "Type": Alegreya Sans) into the smooth atlas the
engine reads (`crates/engine/src/ui/font.rs`, `Font::coverage`): every glyph's coverage in a cell of
CELL x CELL pixels, and a `.font` file with where each sits and how wide it is.

    ~/.venvs/blocky/bin/python mods/base/source/ui/font_bake.py   (needs fontTools)
    ~/.venvs/blocky/bin/python mods/base/source/ui/font_bake.py --pixel FONT.ttf [--also OTHER.ttf]

`--pixel` bakes a pixel font instead (the UI redo, 2026-11-04): at its design size, one design pixel
a font pixel (half a GUI pixel, the UI's pixel), no smoothing, sampled sharp; `--also` takes the
characters the first font lacks (Cyrillic) from a second pixel font of the same size.

Alegreya Sans's digits are old-style (a 3 dips below the line): the font's lining figures
(`zero.lf`...) are put in their place first, as a HUD wants "40/40" to stand level.

A line of text is 16 units tall in the engine; a unit is UNIT atlas pixels. A cell is taller than that (CELL_H): capitals are as tall as CAPS units, so the
ascenders and descenders reach above and below the line, standing on `baseline`, which the engine
puts where the pixel font's baseline is. Positions in the `.font` are atlas pixels: `g <code> <col> <row> <left> <width> <bearing>
<advance>`: the glyph's cell, the first inked column in it and the ink's width, where the ink starts
from the pen and how far the pen moves on. Characters the font lacks are left to Unifont.
"""
import os

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
FONT = os.path.join(HERE, "fonts", "AlegreyaSans-Medium.ttf")
OUT = os.environ.get("FONT_OUT") or os.path.join(HERE, "..", "..", "assets", "fonts")
UNIT = 3  # atlas pixels a font pixel
CELL = 64
CELL_H = 80
CAPS = 11.5  # capital height in font units (the pixel font's capitals are 14 of its 16)
COLS = 32
PAD = 6  # where the pen starts in a cell: room for ink left of it
SPACING = 0.6  # extra pixels between letters, for small sizes
SPACE = 4.5  # a space's width in font units at the least

# Latin, Latin-1, Latin Extended-A, Cyrillic, and the punctuation and signs the game's text uses.
CHARS = (
    [chr(c) for c in range(0x20, 0x7F)]
    + [chr(c) for c in range(0xA0, 0x180)]
    + [chr(c) for c in range(0x400, 0x460)]
    + list("–—‘’‚“”„•…‹›€™−×")
)


def lining(path):
    """A copy of the font whose digits draw its lining figures."""
    from fontTools.ttLib import TTFont

    f = TTFont(path)
    order = set(f.getGlyphOrder())
    names = ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine"]
    for table in f["cmap"].tables:
        if table.isUnicode():
            for i, name in enumerate(names):
                if name + ".lf" in order:
                    table.cmap[0x30 + i] = name + ".lf"
    out = os.path.join("/tmp" if not os.environ.get("TMPDIR") else os.environ["TMPDIR"], "font_bake_lining.ttf")
    f.save(out)
    return out


def size_for(path):
    """The point size whose capitals are CAPS units tall."""
    want = CAPS * UNIT
    size = 8
    while size < CELL_H:
        box = ImageFont.truetype(path, size + 1).getbbox("H")
        if box[3] - box[1] > want:
            break
        size += 1
    return size


def main():
    path = lining(FONT)
    size = size_for(path)
    font = ImageFont.truetype(path, size)
    ascent, descent = font.getmetrics()
    top = max(0, (CELL_H - ascent - descent) // 2)
    baseline = top + ascent
    missing = np.asarray(font.getmask(""))
    glyphs = []
    for ch in CHARS:
        if ch != " " and np.array_equal(np.asarray(font.getmask(ch)), missing):
            continue
        img = Image.new("L", (CELL, CELL_H))
        ImageDraw.Draw(img).text((PAD, top), ch, font=font, fill=255)
        a = np.asarray(img)
        cols = np.nonzero(a.max(0))[0]
        if ch.strip() and len(cols) == 0:
            continue
        left, right = (int(cols[0]), int(cols[-1]) + 1) if len(cols) else (PAD, PAD)
        advance = font.getlength(ch) + SPACING
        if ch == " ":
            advance = max(advance, SPACE * UNIT)  # the font's space is narrow at small sizes
        glyphs.append((ch, img, left, right - left, left - PAD, advance))
    rows = (len(glyphs) + COLS - 1) // COLS
    atlas = Image.new("L", (COLS * CELL, rows * CELL_H))
    lines = [
        "# Baked by mods/base/source/ui/font_bake.py from " + os.path.basename(FONT) + "; do not edit.",
        "format = coverage",
        f"cell = {CELL} {CELL_H}",
        f"scale = {UNIT}",
        f"baseline = {baseline}",
    ]
    for n, (ch, img, left, width, bearing, advance) in enumerate(glyphs):
        col, row = n % COLS, n // COLS
        atlas.paste(img, (col * CELL, row * CELL_H))
        lines.append(f"g {ord(ch):04x} {col} {row} {left} {width} {bearing} {advance:.2f}")
    os.makedirs(OUT, exist_ok=True)
    atlas.save(os.path.join(OUT, "default.png"))
    open(os.path.join(OUT, "default.font"), "w").write("\n".join(lines) + "\n")
    print(f"{len(glyphs)} glyphs at {size} pt, ascent {ascent} descent {descent}, atlas {atlas.size[0]}x{atlas.size[1]}")


def design_size(path, probe="HOWxgq"):
    """The smallest size at which the font draws with no grey pixels: one design pixel a pixel."""
    for size in range(6, 49):
        f = ImageFont.truetype(path, size)
        a = np.asarray(f.getmask(probe, mode="L"))
        if a.size and a.max() == 255 and np.isin(a, (0, 255)).all():
            return size
    raise SystemExit(f"{path}: no size draws without grey pixels; is it a pixel font?")


def pixel_main(path, also=None):
    """A pixel font: `format = coverage` with `scale = 1`, `pixel = 1`, `sharp = true`. The line is
    as tall as its capitals need to sit centred on the engine's baseline (font pixel 14):
    as much room above the capitals as below the baseline."""
    fonts = []
    for p in [path] + ([also] if also else []):
        size = design_size(p)
        fonts.append((p, size, ImageFont.truetype(p, size)))
    f0 = fonts[0][2]
    ascent, descent = f0.getmetrics()
    caps = f0.getbbox("H")
    cap_h = caps[3] - caps[1]
    # The line: the capitals with the same room above them as below the baseline (the descenders
    # and one pixel), so text sits centred in buttons and tabs. The engine stands every font's
    # baseline on font pixel 14 of the line; `baseline` is declared that much lower in the cell so
    # the real one lands on the line's own (`room + cap_h`).
    room = descent + 1
    line = cap_h + 2 * room
    cw, ch = 32, ascent + descent + 4
    top = 2
    baseline = top + ascent + (14 - (room + cap_h))
    glyphs = []
    for c in CHARS:
        for p, size, f in fonts:
            if c != " " and np.asarray(f.getmask(c, mode="1")).sum() == 0:
                continue
            if c != " " and not has_glyph(p, c):
                continue
            img = Image.new("L", (cw, ch))
            # every font's baseline on the first one's, a font with smaller capitals lifted by half
            # the difference so its capitals sit centred where the first one's are (Pixeloid's
            # Cyrillic capitals are 7 px to PixelOperator's 9: on the same baseline they sat a pixel
            # low in every button, the Russian pass, 11-04)
            lift = (cap_h - (f.getbbox("H")[3] - f.getbbox("H")[1])) // 2 if f is not f0 else 0
            ImageDraw.Draw(img).text((PAD, top + ascent - f.getmetrics()[0] - lift), c, font=f, fill=255)
            img = img.point(lambda v: 255 if v >= 128 else 0)
            a = np.asarray(img)
            cols = np.nonzero(a.max(0))[0]
            left, right = (int(cols[0]), int(cols[-1]) + 1) if len(cols) else (PAD, PAD)
            advance = round(f.getlength(c))
            glyphs.append((c, img, left, right - left, left - PAD, advance))
            break
    rows = (len(glyphs) + COLS - 1) // COLS
    atlas = Image.new("L", (COLS * cw, rows * ch))
    names = " + ".join(f"{os.path.basename(p)} at {s} px" for p, s, _ in fonts)
    lines = [
        f"# Baked by mods/base/source/ui/font_bake.py --pixel from {names}; do not edit.",
        "format = coverage",
        f"cell = {cw} {ch}",
        "scale = 1",
        f"baseline = {baseline}",
        "pixel = 1",
        f"line = {line}",
        f"caps = {cap_h}",
        "sharp = true",
    ]
    for n, (c, img, left, width, bearing, advance) in enumerate(glyphs):
        col, row = n % COLS, n // COLS
        atlas.paste(img, (col * cw, row * ch))
        lines.append(f"g {ord(c):04x} {col} {row} {left} {width} {bearing} {advance}")
    os.makedirs(OUT, exist_ok=True)
    atlas.save(os.path.join(OUT, "default.png"))
    open(os.path.join(OUT, "default.font"), "w").write("\n".join(lines) + "\n")
    print(f"{len(glyphs)} glyphs ({names}), capitals {cap_h} px, line {line}, atlas {atlas.size[0]}x{atlas.size[1]}")


_CMAPS = {}


def has_glyph(path, c):
    from fontTools.ttLib import TTFont
    if path not in _CMAPS:
        _CMAPS[path] = TTFont(path).getBestCmap()
    return ord(c) in _CMAPS[path]


if __name__ == "__main__":
    import sys
    if len(sys.argv) > 2 and sys.argv[1] == "--pixel":
        also = sys.argv[sys.argv.index("--also") + 1] if "--also" in sys.argv else None
        pixel_main(sys.argv[2], also)
    else:
        main()
