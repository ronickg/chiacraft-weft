#!/usr/bin/env python3
"""Finds faces that will z-fight (flicker) in a box model: two faces of different elements in
the same plane (or closer than `gap`) whose rectangles overlap, where both can be seen. A box's
face hidden inside another box still flickers if its twin is on the surface, so every pair is
reported; quads count from both sides when double-sided.

    python boxcheck.py spec.json      (the rest pose, then every keyframe of every animation)

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


# ---------------------------------------------------------------- posed check (every keyframe)

def _rot(r):
    import math
    rx, ry, rz = (math.radians(a) for a in r)
    import numpy as np
    X = np.array([[1, 0, 0], [0, math.cos(rx), -math.sin(rx)], [0, math.sin(rx), math.cos(rx)]])
    Y = np.array([[math.cos(ry), 0, math.sin(ry)], [0, 1, 0], [-math.sin(ry), 0, math.cos(ry)]])
    Z = np.array([[math.cos(rz), -math.sin(rz), 0], [math.sin(rz), math.cos(rz), 0], [0, 0, 1]])
    return Z @ Y @ X


def _world(bones, rot, root):
    import numpy as np
    piv = {b["name"]: np.array(b["pivot"], float) for b in bones}
    out = {}
    for b in bones:
        n, par = b["name"], b.get("parent")
        R = _rot(rot.get(n, (0, 0, 0))) if n != "root" else np.eye(3)
        local = piv[n] - (piv[par] if par else 0)
        if not par:
            out[n] = (R, local + np.array(root, float))
        else:
            PR, Pt = out[par]
            out[n] = (PR @ R, Pt + PR @ local)
    return out, piv


def _faces_world(e, R, t, pivot):
    """The element's visible faces in the world: (name, normal, 4 corners)."""
    import numpy as np
    lo, hi = np.array(e["lo"], float), np.array(e["hi"], float)
    names = [e["quad"]] if e.get("quad") else list(FACES)
    out = []
    E = _rot(e["rot"]) if e.get("rot") else np.eye(3)
    o = np.array(e["origin"], float) if e.get("origin") else np.zeros(3)
    for f in names:
        ax, sgn = FACES[f]
        others = [a for a in range(3) if a != ax]
        if any(hi[a] - lo[a] <= 1e-6 for a in others):
            continue
        c = []
        for i, j in ((0, 0), (1, 0), (1, 1), (0, 1)):
            p = np.zeros(3)
            p[ax] = hi[ax] if sgn > 0 else lo[ax]
            p[others[0]] = (lo, hi)[i][others[0]]
            p[others[1]] = (lo, hi)[j][others[1]]
            p = E @ (p - o) + o
            c.append(R @ (p - pivot) + t)
        n = np.zeros(3)
        n[ax] = sgn
        out.append((f, R @ (E @ n), np.array(c), bool(e.get("quad"))))
    return out


def _overlap2d(a, b):
    """Separating-axis test for two convex quads in 2D (shrunk a hair so touching edges don't count)."""
    import numpy as np
    for poly in (a, b):
        for i in range(4):
            e = poly[(i + 1) % 4] - poly[i]
            axis = np.array([-e[1], e[0]])
            if np.linalg.norm(axis) < 1e-9:
                continue
            pa, pb = a @ axis, b @ axis
            if pa.max() <= pb.min() + 1e-3 * np.linalg.norm(axis) or pb.max() <= pa.min() + 1e-3 * np.linalg.norm(axis):
                return False
    return True


def check_posed(spec, gap=0.15, every=1, parallel=0.9998):
    """Flicker that only appears in a pose: faces of parts on different bones that land in one
    plane (within 1 degree, closer than `gap` at the smaller face's middle) and overlap, in any
    keyframe of any animation.
    The rest-pose check can't see these (a folded leg's lower part in its upper part's side
    plane, a paw in the body's bottom plane...). Returns {(a, face, b, face): [anim@t, ...]}."""
    import numpy as np
    hits = {}
    els = spec["elements"]
    for anim, a in spec.get("animations", {}).items():
        for fr in a["frames"][::every]:
            W, piv = _world(spec["bones"], fr["rot"], fr.get("root", (0, 0, 0)))
            faces = []
            for e in els:
                R, t = W[e["bone"]]
                for f, n, c, q in _faces_world(e, R, t, piv[e["bone"]]):
                    faces.append((e, f, n, c, q))
            for i in range(len(faces)):
                ea, fa, na, ca, qa = faces[i]
                for j in range(i + 1, len(faces)):
                    eb, fb, nb, cb, qb = faces[j]
                    if ea["bone"] == eb["bone"]:
                        continue  # same bone: the rest check covers it
                    d = float(na @ nb)
                    if abs(d) < parallel:
                        continue
                    if d < 0 and not (qa or qb):
                        continue  # back to back inside a join: never both seen
                    # Distance from the smaller face's middle to the other face's plane.
                    if np.linalg.norm(np.cross(cb[1] - cb[0], cb[3] - cb[0])) < np.linalg.norm(np.cross(ca[1] - ca[0], ca[3] - ca[0])):
                        dist = (cb.mean(0) - ca[0]) @ na
                    else:
                        dist = (ca.mean(0) - cb[0]) @ nb
                    if abs(dist) >= gap:
                        continue
                    u = ca[1] - ca[0]
                    u /= np.linalg.norm(u)
                    v = np.cross(na, u)
                    if _overlap2d(np.c_[(ca - ca[0]) @ u, (ca - ca[0]) @ v], np.c_[(cb - ca[0]) @ u, (cb - ca[0]) @ v]):
                        hits.setdefault((ea["name"], fa, eb["name"], fb), []).append(f"{anim}@{fr['t']}")
    return hits


if __name__ == "__main__":
    spec = json.load(open(sys.argv[1]))
    hits = check(spec["elements"])
    for h in hits:
        print(f"{h[0]}.{h[1]}  <->  {h[2]}.{h[3]}   {h[4]} apart (rest pose)")
    posed = check_posed(spec) if spec.get("animations") else {}
    for (a, fa, b, fb), when in posed.items():
        print(f"{a}.{fa}  <->  {b}.{fb}   in {len(when)} frames, e.g. {', '.join(when[:3])}")
    print(len(hits), "z-fighting pairs at rest,", len(posed), "in poses")
    sys.exit(1 if hits or posed else 0)
