"""Upstream's six tulips (Weft sync 11-22) from our red tulip: its petals re-shaded in each colour,
stem and leaves kept, so all six are one plant.    python3 notes/blocks/weft_tulips.py"""
import colorsys, os
from PIL import Image
T = os.path.join(os.path.dirname(os.path.abspath(__file__)), "tiles")
COLOURS = {  # hue, saturation, lightness shift of the petals' middle tone
    "orange": (0.07, 0.85, 0.0), "pink": (0.93, 0.62, 0.12), "purple": (0.80, 0.55, -0.04), "white": (0.10, 0.06, 0.30),
}
red = Image.open(os.path.join(T, "tulip_red.png")).convert("RGBA")
petal = lambda r, g, b: r > g + 40 and r > b + 30
for name, (h, s, dl) in COLOURS.items():
    im = red.copy(); p = im.load()
    for y in range(im.height):
        for x in range(im.width):
            r, g, b, a = p[x, y]
            if a and petal(r, g, b):
                _, l, _ = colorsys.rgb_to_hls(r / 255, g / 255, b / 255)
                l = min(0.95, l + dl if name != "white" else 0.55 + l * 0.5)
                p[x, y] = tuple(round(c * 255) for c in colorsys.hls_to_rgb(h, l, s)) + (255,)
    im.save(os.path.join(T, f"tulip_{name}.png"))
print("tulips:", ", ".join(COLOURS))
