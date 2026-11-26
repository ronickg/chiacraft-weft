#!/usr/bin/env python3
"""Where a pose puts Way D's parts: the lowest point of each part in the world (0 = ground),
to check that paws and rump rest on the ground instead of floating or sinking.

    .venv/bin/python tools/ground.py sit lie
"""

import os
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(__file__))
import blocky  # noqa: E402
import voxdog as vd  # noqa: E402

GROUPS = {
    "front paws": lambda n: n.startswith("leg_F") and n.endswith("_paw"),
    "back paws": lambda n: n.startswith("leg_B") and n.endswith("_paw"),
    "back legs": lambda n: n.startswith("thigh_") or (n.startswith("leg_B") and not n.endswith("_paw")),
    "rump": lambda n: n in ("hips", "britches"),
    "chest": lambda n: n in ("chest", "bib"),
    "belly fringe": lambda n: n == "bib_locks" or n.startswith("fringe_front") or n.startswith("fringe_back"),
    "tail": lambda n: n.startswith("tail") or n.startswith("plume") or n.startswith("curl"),
    "head": lambda n: n in ("skull", "muzzle", "nose", "jaw"),
    "tongue": lambda n: n.startswith("tongue"),
}


def corners(e):
    lo, hi = np.array(e["lo"]), np.array(e["hi"])
    c = np.array([[x, y, z] for x in (lo[0], hi[0]) for y in (lo[1], hi[1]) for z in (lo[2], hi[2])])
    if e["rot"]:
        o = np.array(e["origin"])
        c = (c - o) @ vd.qmat(vd.quat(*e["rot"])).T + o
    return c


def lowest(pose, L=blocky.L, elements=None):
    """{group: lowest y} for a pose dict."""
    skel = [(n, p, np.array(v, float)) for n, p, v in blocky.skeleton(L)]
    W = vd.world_transforms(skel, pose)
    out = {}
    for e in elements or blocky.elements():
        R, t = W[e["bone"]]
        pivot = vd.skel_pivot(skel, e["bone"])
        y = ((corners(e) - pivot) @ R.T + t)[:, 1].min()
        for g, test in GROUPS.items():
            if test(e["name"]):
                out[g] = min(out.get(g, 1e9), y)
    return out


if __name__ == "__main__":
    E = blocky.elements()
    for name in sys.argv[1:]:
        fn, dur = vd.ANIMS[name]
        pose = fn(0.0, blocky.L)
        pose.update(blocky.mouth(name, 0.0, dur))
        low = lowest(pose, elements=E)
        print(name, "  ".join(f"{g} {y:.1f}" for g, y in low.items()))
