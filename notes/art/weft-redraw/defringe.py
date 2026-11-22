"""Magenta fringe off cut sprites: an edge pixel (next to transparency) with a magenta hue
(285-340 degrees, saturated) takes its inner neighbour's colour a step darker (the outline), or goes
when it has no inner neighbour.   defringe.py in.png [out.png]"""
import colorsys, sys
from PIL import Image
def mag(p):
    if not p[3]: return False
    h, s, v = colorsys.rgb_to_hsv(*(c / 255 for c in p[:3]))
    return 285 <= h * 360 <= 340 and s > 0.3 and v > 0.25
def run(src, dst):
    im = Image.open(src).convert("RGBA"); p = im.load(); W, H = im.size; n = 0
    for _ in range(3):
        fix = []
        for y in range(H):
            for x in range(W):
                if not mag(p[x, y]): continue
                nb = [(x + dx, y + dy) for dx, dy in ((1,0),(-1,0),(0,1),(0,-1),(1,1),(-1,-1),(1,-1),(-1,1)) if 0 <= x + dx < W and 0 <= y + dy < H]
                edge = any(p[q][3] == 0 for q in nb) or x in (0, W - 1) or y in (0, H - 1)
                inner = [p[q] for q in nb if p[q][3] and not mag(p[q])]
                if not edge and not inner: continue
                if inner:
                    c = min(inner, key=lambda q: sum(q[:3]))
                    fix.append(((x, y), tuple(int(v * (0.85 if edge else 1)) for v in c[:3]) + (255,)))
                else:
                    fix.append(((x, y), (0, 0, 0, 0)))
        for xy, c in fix: p[xy] = c
        n += len(fix)
        if not fix: break
    im.save(dst); return n
if __name__ == "__main__":
    print(run(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else sys.argv[1]))
