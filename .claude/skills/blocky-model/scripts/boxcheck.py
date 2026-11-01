#!/usr/bin/env python3
"""Finds faces that will z-fight (flicker) in a box model: two faces of different elements in
the same plane (or closer than `gap`) whose rectangles overlap, where both can be seen. A box's
face hidden inside another box still flickers if its twin is on the surface, so every pair is
reported; quads count from both sides when double-sided.

    .venv/bin/python tools/boxcheck.py out/d_blocky/spec.json

Elements: {"name", "lo": [x,y,z], "hi": [x,y,z], "rot": null | [..], "quad": null | face}.
Rotated elements are compared only with rotated elements of the same rotation (rare).
"""

import json
import sys

FACES = {"west": (0, -1), "east": (0, 1), "down": (1, -1), "up": (1, 1), "north": (2, -1), "south": (2, 1)}


def faces_of(e):
    names = [e["quad"]] if e.get("quad") else list(FACES)
    out = []
    for f in names:
        ax, sgn = FACES[f]
        plane = e["hi"][ax] if sgn > 0 else e["lo"][ax]
        others = [a for a in range(3) if a != ax]
        rect = [(e["lo"][a], e["hi"][a]) for a in others]
        if any(hi - lo <= 1e-6 for lo, hi in rect):
            continue
        out.append((f, ax, sgn, plane, rect, bool(e.get("quad"))))
    return out


def check(elements, gap=0.3):
    """[(a, face_a, b, face_b, distance)] for faces nearer than `gap` that overlap."""
    hits = []
    fs = [(e, faces_of(e)) for e in elements]
    for i in range(len(fs)):
        ea, fa = fs[i]
        for j in range(i + 1, len(fs)):
            eb, fb = fs[j]
            if (ea.get("rot") or None) != (eb.get("rot") or None):
                continue
            for (na, ax, sa, pa, ra, qa) in fa:
                for (nb, bx, sb, pb, rb, qb) in fb:
                    if ax != bx or abs(pa - pb) >= gap:
                        continue
                    # Facing each other (back to back inside a join) can't both be seen unless a
                    # double-sided quad is involved.
                    if sa != sb and not (qa or qb):
                        continue
                    overlap = all(min(r1[1], r2[1]) - max(r1[0], r2[0]) > 1e-6 for r1, r2 in zip(ra, rb))
                    if overlap:
                        hits.append((ea["name"], na, eb["name"], nb, round(abs(pa - pb), 3)))
    return hits


if __name__ == "__main__":
    spec = json.load(open(sys.argv[1]))
    hits = check(spec["elements"])
    for h in hits:
        print(f"{h[0]}.{h[1]}  <->  {h[2]}.{h[3]}   {h[4]} apart")
    print(len(hits), "z-fighting pairs")
    sys.exit(1 if hits else 0)
