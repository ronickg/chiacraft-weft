"""Gold and diamond shears from the approved iron pair: the steel (the greys) mapped by brightness onto
the item guide's gold and diamond ramps; the leather handles and the rivet stay."""
import colorsys, sys
from PIL import Image
H = lambda s: tuple(int(s[i:i + 2], 16) for i in (1, 3, 5))
RAMPS = {  # dark to light: outline, shadow, base, highlight, bevel
    "gold": [H("#43371a"), H("#7d6236"), H("#bd954e"), H("#e4c75c"), H("#fff594"), H("#fffbd0")],
    "diamond": [H("#26555b"), H("#3a8490"), H("#468594"), H("#48aeba"), H("#8fdde4"), H("#c6f9fb")],
}
src = Image.open(sys.argv[1]).convert("RGBA")
grey = lambda p: p[3] and colorsys.rgb_to_hsv(*(c / 255 for c in p[:3]))[1] < 0.14
lums = sorted(sum(p[:3]) / 3 for p in src.getdata() if grey(p))
lo, hi = lums[0], lums[-1]
for name, ramp in RAMPS.items():
    im = src.copy(); q = im.load()
    for y in range(im.height):
        for x in range(im.width):
            if grey(q[x, y]):
                import bisect; t = bisect.bisect_left(lums, sum(q[x, y][:3]) / 3) / len(lums)  # by rank: every step of the ramp used
                f = t * (len(ramp) - 1); i = min(int(f), len(ramp) - 2); k = f - i
                q[x, y] = tuple(round(a + (b - a) * k) for a, b in zip(ramp[i], ramp[i + 1])) + (255,)
    im.save(f"out_items/{name}_shears.png")
