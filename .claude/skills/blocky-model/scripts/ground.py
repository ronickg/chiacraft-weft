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


def gait(spec, anim, feet, forward=(0.0, 0.0, 1.0)):
    """Whether a walk or run moves forward. Relative to the body, a foot in the air swings forward
    and a foot on the ground pushes back; a gait with the swing reversed moonwalks (feet lift going
    back, plant going forward). For each foot: its average forward speed while it's in the upper
    half of its lift, minus while it's in the lower half, in units per second. Positive is right;
    negative runs backwards. `feet`: element names; `forward`: the way the creature faces (+z)."""
    els = {e["name"]: e for e in spec["elements"]}
    piv = {b["name"]: np.array(b["pivot"], float) for b in spec["bones"]}
    fwd = np.array(forward, float)
    a = spec["animations"][anim]
    frames, length = a["frames"], a["length"]
    out = {}
    for f in feet:
        e = els[f]
        rows = []
        for fr in frames[:-1]:  # the last frame repeats the first
            W = world(spec["bones"], dict(fr["rot"], root=(0, 0, 0)))  # the body's frame: no bob
            R, t = W[e["bone"]]
            c = (corners(e) - piv[e["bone"]]) @ R.T + t
            rows.append((fr["t"], c[:, 1].min(), c.mean(0) @ fwd))
        rows = np.array(rows)
        n = len(rows)
        tt = np.r_[rows[:, 0], rows[0, 0] + length]
        zz = np.r_[rows[:, 2], rows[0, 2]]
        v = np.diff(zz) / np.maximum(np.diff(tt), 1e-6)  # forward speed from each frame to the next
        h = rows[:, 1]
        mid = (h.min() + h.max()) / 2
        up, down = h > mid, h <= mid
        out[f] = round(float(v[up].mean() - v[down].mean()), 1) if up.any() and down.any() and h.max() - h.min() > 0.3 else None
    return out


def _inside(p, e, tol):
    """Whether point p (rest pose) is inside element e's box (rotated about its origin), or within tol of it."""
    lo, hi = np.array(e["lo"], float), np.array(e["hi"], float)
    if e.get("rot"):
        o = np.array(e["origin"], float)
        p = rot(*e["rot"]).T @ (p - o) + o  # into the box's own unrotated frame
    return bool(np.all(p >= lo - tol) and np.all(p <= hi + tol))


def joints(spec, tol=0.25, skip=("root",)):
    """Whether every joint holds its parts together: a bone's pivot must lie inside a box of the
    bone and a box of its parent (both at rest). Then they turn about a point they share and can't
    come apart in any pose; a pivot outside one of them leaves a gap that opens as the joint turns
    (a head floating above its neck). Bones without boxes pass their check up to their children
    (their parent must hold the pivot). Returns [(bone, what's wrong)]."""
    boxes = {}
    for e in spec["elements"]:
        if not e.get("quad"):
            boxes.setdefault(e["bone"], []).append(e)
    parent = {b["name"]: b.get("parent") for b in spec["bones"]}
    bad = []
    for b in spec["bones"]:
        n, par = b["name"], b.get("parent")
        if n in skip or not par or n not in boxes:
            continue
        p = np.array(b["pivot"], float)
        while par and par not in boxes:  # a bone with only panels or nothing: the next one up holds it
            par = parent.get(par)
        if not any(_inside(p, e, tol) for e in boxes[n]):
            bad.append((n, "its pivot is outside its own boxes"))
        if par and not any(_inside(p, e, tol) for e in boxes[par]):
            bad.append((n, f"its pivot is outside its parent's ({par}) boxes"))
    return bad


def _frames(spec):
    """(label, pose) for the rest pose and every keyframe of every animation in the spec."""
    out = [("rest pose", {})]
    for name, a in spec.get("animations", {}).items():
        for fr in a["frames"]:
            out.append((f"{name}@{fr['t']}", dict(fr["rot"], root=fr.get("root", (0, 0, 0)))))
    return out


def _frame_of(e, W, piv):
    """(R, t, o, lo, hi): world = R @ (box point - o) + t for the element's own unrotated box."""
    R, t = W[e["bone"]]
    E = rot(*e["rot"]) if e.get("rot") else np.eye(3)
    o = np.array(e["origin"], float) if e.get("rot") else piv[e["bone"]]
    # world = R @ (E @ (q - o) + o - pivot) + t
    return R @ E, t + R @ (o - piv[e["bone"]]), o, np.array(e["lo"], float), np.array(e["hi"], float)


def pokes(spec, parts=None, max_out=1.5, min_angle=5.0, tol=0.05):
    """Tilted boxes poking through their neighbors: where two parts meet at a joint at an angle
    (a neck rising into a level head, a tail segment swung against the one before, a thigh against
    the hips), a corner of one sticking just out of the other (more than `tol`, less than
    `max_out` units) cuts a sliver and slanted edges across the other's faces: it looks broken.
    Parts that meet square-on (a jaw under a skull) make a clean step instead, and parts that really
    extend beyond the other (a thigh beside the hips) stick out further; neither is reported. At rest
    and in every keyframe. `parts`: element names or name prefixes to check among (the body's axis:
    neck, head, chest, hips, tail; build scripts assert it's empty for these); None checks every
    joined pair, where limb joints (knee, ankle, toes) and an opening jaw also show, as the usual
    box-joint step. Returns {(poking element, element poked): (worst distance, [frames])}."""
    keep = (lambda n: True) if parts is None else (lambda n: any(n == p or n.startswith(p) for p in parts))
    piv = {b["name"]: np.array(b["pivot"], float) for b in spec["bones"]}
    parent = {b["name"]: b.get("parent") for b in spec["bones"]}
    boxes = [e for e in spec["elements"] if not e.get("quad") and keep(e["name"])]
    pairs = [(a, b) for a in boxes for b in boxes
             if a is not b and (parent.get(a["bone"]) == b["bone"] or parent.get(b["bone"]) == a["bone"])]
    found = {}
    for label, pose in _frames(spec):
        W = world(spec["bones"], pose)
        F = {id(e): _frame_of(e, W, piv) for e in boxes}
        for a, b in pairs:
            Ra, ta, oa, loa, hia = F[id(a)]
            Rb, tb, ob, lob, hib = F[id(b)]
            cos = (np.trace(Ra.T @ Rb) - 1) / 2
            if np.degrees(np.arccos(np.clip(cos, -1, 1))) < min_angle:
                continue
            ca = np.array([[x, y, z] for x in (loa[0], hia[0]) for y in (loa[1], hia[1]) for z in (loa[2], hia[2])])
            wa = (ca - oa) @ Ra.T + ta
            qb = (wa - tb) @ Rb + ob  # into b's own unrotated box frame
            out = np.max(np.maximum(lob - qb, qb - hib), axis=1)
            if not np.any(out <= tol):  # no corner inside: they only touch, or one passes by
                continue
            worst = out[(out > tol) & (out < max_out)]
            if len(worst):
                key = (a["name"], b["name"])
                d, where = found.get(key, (0.0, []))
                found[key] = (max(d, float(worst.max())), where + [label])
    return found


if __name__ == "__main__":
    import json
    import sys

    spec = json.load(open(sys.argv[1]))
    for (a, b), (d, where) in sorted(pokes(spec, sys.argv[2:] or None).items()):
        print(f"{a} pokes out of {b} by {d:.2f} in {len(where)} frames, e.g. {', '.join(where[:3])}")


def feet_low(spec, anim, feet):
    """The lowest point of `feet` (element names) in each frame of animation `anim` (a dict with
    "frames", each {"rot", "root"}), the root's offset included."""
    els = {e["name"]: e for e in spec["elements"]}
    piv = {b["name"]: np.array(b["pivot"], float) for b in spec["bones"]}
    out = []
    for fr in anim["frames"]:
        W = world(spec["bones"], dict(fr["rot"], root=fr["root"]))
        out.append(min(((corners(els[f]) - piv[els[f]["bone"]]) @ W[els[f]["bone"]][0].T + W[els[f]["bone"]][1])[:, 1].min()
                       for f in feet))
    return out


def grounded(spec, anim, feet, bounds=False):
    """`anim` with its root's height set so its feet stand on the ground (y = 0): a gait written
    as angles (a crouched stalk, a head-low approach) sinks or floats where its legs bend. Each
    frame on its lowest foot (`bounds` False: a walk, always a foot down), or the whole clip
    shifted once so its lowest foot over the loop touches (`bounds` True: a run or charge with
    airborne moments, keeping its own bounce)."""
    low = feet_low(spec, anim, feet)
    frames = []
    for fr, y in zip(anim["frames"], low):
        r = list(fr["root"])
        r[1] = r[1] - (min(low) if bounds else y)
        frames.append({**fr, "root": r})
    return {**anim, "frames": frames}
