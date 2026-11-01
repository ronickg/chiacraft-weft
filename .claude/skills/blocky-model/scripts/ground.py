#!/usr/bin/env python3
"""Where a pose puts a box model's parts: each group's lowest point in the world (y = 0 is the
ground), to fit sitting and lying poses so paws and body rest on the ground.

    from ground import lowest
    low = lowest(spec, pose, groups={"paws": lambda n: n.endswith("_paw"), ...})

`spec`: {"bones": [{"name", "parent", "pivot"}], "elements": [{"name", "bone", "lo", "hi",
"rot"?, "origin"?}]} (parents listed before children). `pose`: {bone: (rx, ry, rz) degrees,
"root": (x, y, z) offset}. Rotations are R = Rz Ry Rx (Blockbench / Hytale).
"""

import numpy as np


def rot(rx, ry, rz):
    x, y, z = np.radians([rx, ry, rz])
    Rx = np.array([[1, 0, 0], [0, np.cos(x), -np.sin(x)], [0, np.sin(x), np.cos(x)]])
    Ry = np.array([[np.cos(y), 0, np.sin(y)], [0, 1, 0], [-np.sin(y), 0, np.cos(y)]])
    Rz = np.array([[np.cos(z), -np.sin(z), 0], [np.sin(z), np.cos(z), 0], [0, 0, 1]])
    return Rz @ Ry @ Rx


def world(bones, pose):
    """{bone: (R, t)}: a point p given in model space at rest maps to R @ (p - pivot) + t."""
    piv = {b["name"]: np.array(b["pivot"], float) for b in bones}
    out = {}
    for b in bones:
        n, par = b["name"], b.get("parent")
        R = rot(*pose.get(n, (0, 0, 0))) if n != "root" else np.eye(3)
        local = piv[n] - (piv[par] if par else 0)
        if not par:
            local = local + np.array(pose.get("root", (0, 0, 0)), float)
            out[n] = (R, local)
        else:
            PR, Pt = out[par]
            out[n] = (PR @ R, Pt + PR @ local)
    return out


def corners(e):
    lo, hi = np.array(e["lo"], float), np.array(e["hi"], float)
    c = np.array([[x, y, z] for x in (lo[0], hi[0]) for y in (lo[1], hi[1]) for z in (lo[2], hi[2])])
    if e.get("rot"):
        o = np.array(e["origin"], float)
        c = (c - o) @ rot(*e["rot"]).T + o
    return c


def lowest(spec, pose, groups):
    """{group: lowest world y} for the elements each group's test(name) accepts."""
    W = world(spec["bones"], pose)
    piv = {b["name"]: np.array(b["pivot"], float) for b in spec["bones"]}
    out = {}
    for e in spec["elements"]:
        R, t = W[e["bone"]]
        y = ((corners(e) - piv[e["bone"]]) @ R.T + t)[:, 1].min()
        for g, test in groups.items():
            if test(e["name"]):
                out[g] = min(out.get(g, np.inf), y)
    return out
