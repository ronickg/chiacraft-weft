"""The shield's model texture (models/item/shield.geo.json UVs, 64 x 64, 32 texels a block) painted
from our own materials: oak planks (the block tile, boards turned upright), the item guide's iron
ramp for the rim and boss, red-brown leather for the grip. Front at (0,0), back at (24,0): each a
20 x 30 stepped heater shape (20 x 24, then 16 x 4, then 8 x 2), as the geo's faces lie."""
from PIL import Image
import sys
root = sys.argv[1]; out = sys.argv[2]
planks = Image.open(f"{root}/mods/base/assets/textures/block/planks_oak.png").convert("RGBA").rotate(90)
H = lambda s: tuple(int(s[i:i + 2], 16) for i in (1, 3, 5)) + (255,)
OUT_L, OUT_D = H("#6a6461"), H("#48423f")
RIM_L, RIM_M, RIM_D, BEV = H("#c9c3be"), H("#a8a29d"), H("#8e8884"), H("#ece6e2")
GOLD, GOLD_D, GOLD_H = H("#e4c75c"), H("#bd954e"), H("#fff594")
LEA, LEA_D, LEA_H = H("#6b321a"), H("#542c1a"), H("#8a4a2a")
def inside(x, y):
    return (0 <= x < 20 and 0 <= y < 24) or (2 <= x < 18 and 24 <= y < 28) or (6 <= x < 14 and 28 <= y < 30)
def dist(x, y):  # how many steps in from the edge (chebyshev)
    for d in range(0, 4):
        if any(not inside(x + i, y + j) for i in range(-d - 1, d + 2) for j in range(-d - 1, d + 2)):
            return d
    return 4
def lit(x, y):  # facing the top-left light: the outside is nearer up or left
    up = sum(not inside(x, y - k) for k in (1, 2, 3)); left = sum(not inside(x - k, y) for k in (1, 2, 3))
    down = sum(not inside(x, y + k) for k in (1, 2, 3)); right = sum(not inside(x + k, y) for k in (1, 2, 3))
    return up + left >= down + right
img = Image.new("RGBA", (64, 64), (0, 0, 0, 0)); p = img.load()
def face(ox, back):
    for y in range(30):
        for x in range(20):
            if not inside(x, y): continue
            d = dist(x, y); L = lit(x, y)
            if d == 0: c = OUT_L if L else OUT_D
            elif d == 1: c = (BEV if (L and (not inside(x, y - 2) or not inside(x - 2, y))) else RIM_L) if L else RIM_D
            else:
                c = planks.getpixel(((x + (7 if back else 3)) % 32, (y + 5) % 32))
                if d == 2 and not L:  # a soft shadow under the rim on the lower right
                    c = tuple(int(v * 0.8) for v in c[:3]) + (255,)
            p[ox + x, y] = c
    if back:  # the grip strap across the middle
        for y in (11, 12, 13, 14):
            for x in range(3, 17):
                p[ox + x, y] = LEA_H if y == 11 else LEA_D if y == 14 else LEA
        for x in (3, 16):
            for y in (11, 12, 13, 14): p[ox + x, y] = GOLD_D if y > 12 else GOLD
    else:  # the boss: a round iron dome with a gold rivet, and two gold studs on the rim
        cx, cy = 10, 12
        for y in range(cy - 4, cy + 4):
            for x in range(cx - 4, cx + 4):
                dx, dy = x - cx + 0.5, y - cy + 0.5
                r2 = dx * dx + dy * dy
                if r2 > 16.5: continue
                if r2 > 10.5: c = OUT_L if dx + dy < 0 else OUT_D
                elif dx + dy < -2.5: c = BEV
                elif dx + dy < 0.5: c = RIM_L
                elif dx + dy < 2.5: c = RIM_M
                else: c = RIM_D
                p[ox + x, y] = c
        p[ox + cx - 1, cy - 1] = GOLD_H; p[ox + cx, cy - 1] = GOLD; p[ox + cx - 1, cy] = GOLD; p[ox + cx, cy] = GOLD_D
        for (x, y) in ((4, 4), (15, 4), (4, 20), (15, 20)):
            p[ox + x, y] = GOLD_H if y == 4 else GOLD
face(0, False); face(24, True)
for y in range(2):  # the edges (east/west/up/down share one 2 x 2 patch): the rim's metal
    for x in range(2): p[48 + x, y] = (RIM_L, RIM_M, RIM_M, RIM_D)[y * 2 + x]
for y in range(8):  # the handle: leather with a gold band
    for x in range(4):
        p[48 + x, 4 + y] = GOLD if y in (0, 7) else (LEA_H if x == 0 else LEA_D if x == 3 else LEA)
img.save(out)
# A picture of the front alone, for the review sheet
img.crop((0, 0, 20, 30)).save(out.replace(".png", "_front.png"))
