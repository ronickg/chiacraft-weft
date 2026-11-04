"""Bakes the game's text font (notes/STYLE.md, "Type": Alegreya Sans) into the smooth atlas the
engine reads (`crates/engine/src/ui/font.rs`, `Font::coverage`): every glyph's coverage in a cell of
CELL x CELL pixels, and a `.font` file with where each sits and how wide it is.

    ~/.venvs/blocky/bin/python mods/base/source/ui/font_bake.py   (needs fontTools)

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
OUT = os.path.join(HERE, "..", "..", "assets", "fonts")
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


if __name__ == "__main__":
    main()
