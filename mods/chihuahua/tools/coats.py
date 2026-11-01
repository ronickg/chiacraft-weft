#!/usr/bin/env python3
"""Paints the chihuahua coats: 8 base colors x 6 patterns x 2 marking layouts = 96 skins, on the
model's 64x32 box layout (see `model` in `src/lib.rs`). Deterministic: run it again after changing
it and commit the pictures.

    python3 mods/chihuahua/tools/coats.py
"""

import os
import random
import struct
import zlib

OUT = os.path.join(os.path.dirname(__file__), "..", "assets", "textures", "entity", "chihuahua")

# Base coat colors (sRGB), named as breeders do.
COLORS = [
    ("fawn", (214, 160, 102)),
    ("cream", (236, 214, 172)),
    ("chocolate", (98, 54, 34)),
    ("black", (40, 34, 32)),
    ("blue", (98, 106, 116)),
    ("red", (176, 96, 48)),
    ("white", (238, 232, 222)),
    ("golden", (204, 146, 64)),
]
WHITE = (244, 240, 232)
TAN = (196, 132, 74)
PINK = (226, 160, 150)
NOSE = (52, 38, 36)
EYE = (24, 18, 18)
EYE_SHINE = (90, 74, 70)

# Patterns: solid; white chest, paws, chin and tail tip; piebald (a white blaze, chest, belly and
# legs, the color left on the back and ears); tricolor (black or the color with tan points and a
# white chest); sable (dark tips along the back); patched (white with patches of the color).
PATTERNS = ["solid", "marked", "piebald", "tricolor", "sable", "patched"]

# The model's boxes on the 64x32 sheet: name, uv origin, size (w, h, d), as in `src/lib.rs`.
BOXES = [
    ("skull", (0, 0), (5, 4, 4)),
    ("snout", (18, 0), (2, 2, 2)),
    ("ear_r", (26, 0), (2, 3, 1)),
    ("ear_l", (32, 0), (2, 3, 1)),
    ("body", (0, 10), (4, 3, 6)),
    ("leg", (24, 10), (1, 3, 1)),
    ("tail", (30, 10), (1, 1, 4)),
]


def faces(origin, size):
    """The box layout's faces: name -> (x, y, w, h) on the sheet."""
    u, v = origin
    w, h, d = size
    return {
        "top": (u + d, v, w, d),
        "bottom": (u + d + w, v, w, d),
        "right": (u, v + d, d, h),
        "front": (u + d, v + d, w, h),
        "left": (u + d + w, v + d, d, h),
        "back": (u + 2 * d + w, v + d, w, h),
    }


def mix(a, b, t):
    return tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))


def shade(c, f):
    return tuple(max(0, min(255, round(x * f))) for x in c)


def paint(color, pattern, marks):
    base = color[1]
    rng = random.Random(f"{color[0]}-{pattern}-{marks}")
    # A tricolor of a light color is drawn on black, like the real ones.
    if pattern == "tricolor" and sum(base) > 300:
        base = (40, 34, 32)
    dark = shade(base, 0.72)
    img = [[(0, 0, 0, 0)] * 64 for _ in range(32)]

    def put(x, y, c):
        j = rng.uniform(0.94, 1.06)
        img[y][x] = shade(c, j) + (255,)

    # Patch centers for "patched", and how wide the blaze is, by the marking layout.
    patches = [(rng.uniform(0, 1), rng.uniform(0, 1)) for _ in range(3 + marks)]
    blaze = 1 if marks == 0 else 2

    for name, origin, size in BOXES:
        for face, (fx, fy, fw, fh) in faces(origin, size).items():
            for y in range(fh):
                for x in range(fw):
                    # 0..1 across the face, y from the top.
                    s, t = (x + 0.5) / fw, (y + 0.5) / fh
                    c = base
                    white = False
                    tan = False
                    if pattern == "marked":
                        white = (name == "body" and face in ("bottom", "front")) or (name == "leg" and t > 0.6) or (name == "snout" and face in ("bottom", "front") and t > 0.5) or (name == "tail" and face == "back")
                    elif pattern == "piebald":
                        white = (
                            (name == "body" and (face in ("bottom", "front") or (face in ("left", "right") and t > 0.55)))
                            or name == "leg"
                            or name == "snout"
                            or (name == "skull" and face == "front" and abs(s - 0.5) * fw < blaze * 0.5 + 0.01)
                            or (name == "skull" and face == "top" and abs(s - 0.5) * fw < blaze * 0.5 + 0.01 and t > 0.4)
                            or (name == "skull" and face == "bottom")
                            or (name == "tail" and face == "back")
                        )
                    elif pattern == "tricolor":
                        white = (name == "body" and face == "front" and t > 0.3) or (name == "leg" and t > 0.8)
                        tan = (name == "leg" and t > 0.35) or name == "snout" or (name == "skull" and face == "bottom") or (name == "body" and face == "bottom")
                    elif pattern == "sable":
                        if face == "top" or (face in ("left", "right") and t < 0.35 and name in ("body", "skull")):
                            c = dark
                        if name == "body" and face == "bottom":
                            c = mix(base, WHITE, 0.45)
                    elif pattern == "patched":
                        white = True
                        if name in ("ear_r", "ear_l"):
                            white = False
                        elif name == "skull" and not (face == "front" and abs(s - 0.5) * fw < 0.6):
                            white = marks == 1 and s < 0.5
                        elif name == "body":
                            # Patches on the back half by the marking layout.
                            px = s if face in ("top", "bottom", "front", "back") else (1 - s if face == "left" else s)
                            for (cx, cy) in patches:
                                if (px - cx) ** 2 + (t - cy) ** 2 < 0.09:
                                    white = False
                    if white:
                        c = WHITE
                    elif tan:
                        c = TAN
                    put(fx + x, fy + y, c)

        # The face: eyes, eyebrows, nose, inner ears.
        if name == "skull":
            fx, fy, fw, fh = faces(origin, size)["front"]
            for ex in (fx + 1, fx + 3):
                img[fy + 1][ex] = EYE + (255,)
                img[fy + 2][ex] = EYE_SHINE + (255,)
                if pattern == "tricolor":
                    img[fy][ex] = TAN + (255,)
        if name == "snout":
            fx, fy, fw, fh = faces(origin, size)["front"]
            img[fy][fx] = NOSE + (255,)
            img[fy][fx + 1] = NOSE + (255,)
            img[fy + 1][fx] = shade(img[fy + 1][fx][:3], 0.85) + (255,)
        if name in ("ear_r", "ear_l"):
            fx, fy, fw, fh = faces(origin, size)["front"]
            for y in range(1, fh):
                img[fy + y][fx + (1 if name == "ear_r" else 0)] = mix(PINK, base, 0.25 if y == 1 else 0.0) + (255,)
    return img


def png(path, img):
    raw = b"".join(b"\x00" + bytes(c for px in row for c in px) for row in img)

    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    data = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", 64, 32, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
    with open(path, "wb") as f:
        f.write(data)


def main():
    os.makedirs(OUT, exist_ok=True)
    i = 0
    for color in COLORS:
        for pattern in PATTERNS:
            for marks in range(2):
                png(os.path.join(OUT, f"coat_{i}.png"), paint(color, pattern, marks))
                i += 1
    print(f"{i} coats in {os.path.normpath(OUT)}")


if __name__ == "__main__":
    main()
