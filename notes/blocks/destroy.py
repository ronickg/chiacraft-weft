"""Block-breaking cracks at 32 px (Ronald, 11-05: "the block break animation looks pretty bad, glitchy"; Lighting
asked for a redraw). The engine's were 16 px, so on our 32 px blocks their pixels came out twice the blocks'
own and read as chunky noise, a dense maze by stage 9 (half the face).

Grown in code so each stage holds the last: a crack tree from one point near the middle, each crack a
one-pixel line that wanders a little and forks; every pixel gets the time the crack reached it, and stage k
shows the pixels reached by its time. One dark colour (20, 20, 20) at alpha 230, as before (the pack reads
only the alpha). Stage 9 covers under a fifth of the face.

    python3 notes/blocks/destroy.py <out dir>      # destroy_0.png .. destroy_9.png
"""
import math
import os
import sys

import numpy as np
from PIL import Image

N = 32
COLOUR = (20, 20, 20, 230)
SEED = 7


def grow(seed=SEED):
    rng = np.random.default_rng(seed)
    t = np.full((N, N), np.inf)
    origin = (15.5 + rng.uniform(-1.5, 1.5), 16.5 + rng.uniform(-1.5, 1.5))
    # (x, y, angle, length, start time, depth)
    stack = []
    base = rng.uniform(0, 2 * math.pi)
    for i in range(4):  # four main cracks from the point, unevenly spaced
        a = base + i * (math.pi / 2) + rng.uniform(-0.45, 0.45)
        stack.append((origin[0], origin[1], a, rng.uniform(15, 21), 0.0, 0))
    while stack:
        x, y, a, length, t0, depth = stack.pop()
        steps = int(length)
        last = None
        for s in range(steps):
            a += rng.normal(0, 0.22)  # a crack wanders
            x += math.cos(a)
            y += math.sin(a)
            ix, iy = int(math.floor(x)), int(math.floor(y))
            if not (0 <= ix < N and 0 <= iy < N):
                break
            if last is not None and abs(ix - last[0]) == 1 and abs(iy - last[1]) == 1:
                # keep the line one pixel wide but connected: no diagonal gaps, one corner pixel
                cx, cy = (ix, last[1])
                t[cy, cx] = min(t[cy, cx], t0 + s - 0.5)
            t[iy, ix] = min(t[iy, ix], t0 + s)
            last = (ix, iy)
            # forks: rarer deeper down, shorter
            if depth < 3 and s > 2 and rng.random() < (0.2 if depth == 0 else 0.13):
                side = rng.choice([-1, 1])
                stack.append((x, y, a + side * rng.uniform(0.6, 1.1), length * rng.uniform(0.35, 0.6) * (0.8 ** depth), t0 + s, depth + 1))
    return t


def stages(t, n=10):
    finite = np.sort(t[np.isfinite(t)])
    out = []
    for k in range(n):
        # stage 0 a small star round the point; then the reach grows, the last stage everything
        share = 0.06 + 0.94 * (k / (n - 1)) ** 1.15
        cut = finite[min(len(finite) - 1, int(share * (len(finite) - 1)))]
        mask = t <= cut
        a = np.zeros((N, N, 4), np.uint8)
        a[mask] = COLOUR
        out.append(a)
    return out


def main(out):
    os.makedirs(out, exist_ok=True)
    t = grow()
    for k, a in enumerate(stages(t)):
        Image.fromarray(a).save(os.path.join(out, f"destroy_{k}.png"))
        print(f"destroy_{k}: {(a[..., 3] > 0).mean():.3f} of the face")


if __name__ == "__main__":
    main(sys.argv[1])
