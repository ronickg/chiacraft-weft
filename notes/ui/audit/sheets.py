#!/usr/bin/env python3
"""Builds the two UI sheets for Ronald from a pair of PC shot runs: a captioned grid of the
1080p shots and one 4K crop at its own pixels.

    sheets.py <runs dir> <look-on tag> <batch 3 tag> <out dir>

<runs dir> holds t2_<tag>_lo_1920x1080 and _3840x2160 for the look-on tag, t2_<tag>_b3_... for
batch 3's (t2q fetch).
Writes look_on.png and batch3.png (each with its 4K crop below the grid) into <out dir>.
"""
import pathlib
import sys

from PIL import Image, ImageDraw, ImageFont

LOOK_ON = [
    ("fs_02_video.png", "Video: presets, one quality row per feature"),
    ("fs_02b_video_end.png", "Video, scrolled to the end"),
    ("fs_03_medium.png", "Preset Medium"),
    ("fs_04_custom.png", "One row changed: Custom"),
    ("fs_05_developer.png", "Developer page (/developer_menu on)"),
    ("fs_06_developer_knobs.png", "Developer page: the look's knobs"),
    ("fs_08_mod_settings_base.png", "Mod Settings: no look knobs, no Video settings"),
    ("fs_01_world.png", "The look on at a player's first start", None),
]
LOOK_ON_CROP = ("fs_02_video.png", "4K, 1:1 pixels: the Video tab")

BATCH3 = [
    ("b3_01_multiplayer_world.png", "Multiplayer (in a world)"),
    ("b3_02_links_world.png", "Links"),
    ("b3_03_admission.png", "Admission"),
    ("b3_04_hud_options.png", "HUD options: debug lines"),
    ("b3_06_touch_buttons.png", "Touch buttons"),
    ("b3_07_controls_scrolled.png", "Controls, scrolled"),
    ("b3_10_multiplayer.png", "Multiplayer (title)"),
    ("b3_11_links.png", "Links (title)"),
    ("b3_12_friends.png", "Friends"),
    ("b3_13_finder.png", "Find Worlds"),
    ("b3_14_account.png", "Account"),
    ("b3_15_achievements.png", "Achievements"),
    ("b3_16_profile_scrolled.png", "Profile"),
    ("b3_17_new_address.png", "New address?"),
    ("b3_18_delete_world.png", "Delete world?"),
]
BATCH3_CROP = ("b3_14_account.png", "4K, 1:1 pixels: Account")

# The Video and Mod Settings windows at 1080p (scale 2), shown at their own pixels.
LOOK_ON_BOX = (490, 190, 1430, 880)
CAP = 34
PAD = 12
BG = (24, 18, 14)
INK = (236, 222, 196)


def font(size):
    for f in ["/System/Library/Fonts/Helvetica.ttc", "/Library/Fonts/Arial.ttf"]:
        try:
            return ImageFont.truetype(f, size)
        except OSError:
            pass
    return ImageFont.load_default()


def build(title, shots, crop, d1080, d4k, out, cols, cell, box=None):
    """`box`: the part of each 1080p shot shown, at its own pixels (else the whole shot, scaled
    to `cell`). Shots in a list as (file, caption) or (file, caption, own box or None)."""
    shots = [s if len(s) == 3 else (*s, box) for s in shots if (d1080 / s[0]).exists()]
    rows = (len(shots) + cols - 1) // cols
    big = Image.open(d4k / crop[0]).convert("RGB")
    cw, ch = 1920, 1080
    cx, cy = (big.width - cw) // 2, (big.height - ch) // 2
    piece = big.crop((cx, cy, cx + cw, cy + ch))
    w = max(cols * cell[0] + (cols + 1) * PAD, cw + 2 * PAD)
    head = 60
    h = head + rows * (cell[1] + CAP + PAD) + PAD + CAP + ch + PAD
    sheet = Image.new("RGB", (w, h), BG)
    d = ImageDraw.Draw(sheet)
    d.text((PAD, 14), title, fill=INK, font=font(30))
    f = font(22)
    for i, (name, cap, b) in enumerate(shots):
        r, c = divmod(i, cols)
        x = PAD + c * (cell[0] + PAD)
        y = head + r * (cell[1] + CAP + PAD)
        im = Image.open(d1080 / name).convert("RGB")
        if b:
            im = im.crop(b)
        im.thumbnail(cell, Image.LANCZOS)
        sheet.paste(im, (x + (cell[0] - im.width) // 2, y + (cell[1] - im.height) // 2))
        d.text((x, y + cell[1] + 6), cap, fill=INK, font=f)
    y = head + rows * (cell[1] + CAP + PAD) + PAD
    d.text((PAD, y), crop[1], fill=INK, font=f)
    sheet.paste(piece, (PAD, y + CAP))
    out.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out)
    print(out, sheet.size)


def main():
    runs, lo_tag, b3_tag, out = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3], pathlib.Path(sys.argv[4])
    lo = (runs / f"t2_{lo_tag}_lo_1920x1080", runs / f"t2_{lo_tag}_lo_3840x2160")
    b3 = (runs / f"t2_{b3_tag}_b3_1920x1080", runs / f"t2_{b3_tag}_b3_3840x2160")
    build("Look always on: Video tab, presets, Developer page (1920x1080)", LOOK_ON, LOOK_ON_CROP, *lo,
          out / "look_on.png", 2, (940, 690), LOOK_ON_BOX)
    build("Batch 3: the last menus on the new frame (1920x1080, at half size)", BATCH3, BATCH3_CROP, *b3,
          out / "batch3.png", 2, (960, 540))


if __name__ == "__main__":
    main()
