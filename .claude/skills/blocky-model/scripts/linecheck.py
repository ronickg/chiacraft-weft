#!/usr/bin/env python3
"""Finds the thin lines that make a box model read broken (Ronald, 11-05, on Drenkaun's back: "a
thin line there, which makes no sense to have there"): a strip of one box showing beside a box that
covers the rest of it, narrower than about two texels.

    python linecheck.py spec.json [texture.png]        (a build's out/spec.json, at rest)
    python linecheck.py model.geo.json texture.png     (an exported model, as the game has it)
    python linecheck.py --all [repo]                   (every model in mods/*/assets/models)
    ... --shot out.png        the model from three sides, plain and with each line marked magenta
    ... --all-kinds           list the seams and undersides too

What it reports, at rest:
- `line` (between boxes lined up with each other; where one enters another at an angle the
  visible bit is a wedge, `ground.pokes`' business): part of a face left uncovered by a box lying on or in it, under 2 texels wide (and over
  `FLUSH`, the half unit `boxcheck` asks for between faces) and at
  least 2 long, in a colour unlike the box that covers the rest (a fur box's top showing beside a
  moss pad, a body box's edge between two tufts). These are the thin lines.
  A strip whose texels are cut out (a shell's open hem) shows nothing and is skipped.
- `seam`: the same strip in nearly the same colour as its cover: a small step, read as a crease.
  Fine on a joint (an upper leg's rim round the lower), worth a look on a flat back.
- `thin`: a whole face under one texel wide that nothing covers (a box thinner than a texel seen
  edge-on): it can't hold a texel, so it is drawn as a smeared line. Listed, never fails: the
  stacked cores of ears, wings and fins are this thin on purpose (lit as one piece); a lone
  slab (a paw, a lip) this thin should be a texel or more.
- `bleed`: a face whose texture rectangle isn't whole texels, or whose one-texel gutter (the ring
  round it in the atlas) isn't a copy of its edge: at the face's edge the renderer can pick the
  neighbouring face's texel and draw a line of the wrong colour.

Each hit names the box, its face, the strip's width and length in texels (the face's own texels:
fine-painted models count theirs), and the box that covers the rest with how far it would have
to reach to hide the strip. Fix by making the cover reach past the edge (half a unit past, so
the faces don't share a plane), pulling the poking box in, or letting it show on purpose by two
texels or more. Exit 1 if any `line` or `bleed` hit (the rest are listed, never fail).

Build scripts call `lines(boxes_from_spec(spec), texture)` and assert there are no `line` hits
(`allow` names the strips that are meant: `box`, `box.face` or `prefix*`). Checked at rest: in a
pose a joint's bend opens and closes such strips anyway.
"""

import glob
import json
import math
import os
import sys

import numpy as np

FACES = {"west": (0, -1), "east": (0, 1), "down": (1, -1), "up": (1, 1), "north": (2, -1), "south": (2, 1)}
# Seen from outside: corner, u and v directions (u right, v down) as the painters lay a face out.
FRAMES = {"south": ("lo0 hi1 hi2", (1, 0, 0), (0, -1, 0)), "north": ("hi0 hi1 lo2", (-1, 0, 0), (0, -1, 0)),
          "east": ("hi0 hi1 hi2", (0, 0, -1), (0, -1, 0)), "west": ("lo0 hi1 lo2", (0, 0, 1), (0, -1, 0)),
          "up": ("lo0 hi1 lo2", (1, 0, 0), (0, 0, 1)), "down": ("lo0 lo1 hi2", (1, 0, 0), (0, 0, -1))}
SPT = 4  # samples a texel along each side
MAX_W, MIN_LEN = 2.0, 2.0  # a line: under 2 texels wide and at least 2 long
FLUSH = 0.6  # units: a strip this narrow is the half-unit step boxcheck asks for between faces, read as flush
CONTRAST = 34.0  # mean RGB distance from the cover's colour that turns a seam into a line


def rot(rx, ry, rz):
    rx, ry, rz = (math.radians(a) for a in (rx, ry, rz))
    X = np.array([[1, 0, 0], [0, math.cos(rx), -math.sin(rx)], [0, math.sin(rx), math.cos(rx)]])
    Y = np.array([[math.cos(ry), 0, math.sin(ry)], [0, 1, 0], [-math.sin(ry), 0, math.cos(ry)]])
    Z = np.array([[math.cos(rz), -math.sin(rz), 0], [math.sin(rz), math.cos(rz), 0], [0, 0, 1]])
    return Z @ Y @ X


# ---------------------------------------------------------------- loading: boxes in the world at rest

def boxes_from_spec(spec):
    """[{name, bone, lo, hi, A, c, quad, faces}]: a point p of the box (lo..hi, the element's own
    frame) is at A @ p + c in the world."""
    piv = {b["name"]: np.array(b["pivot"], float) for b in spec["bones"]}
    world = {}
    for b in spec["bones"]:  # at rest bones don't turn: only an element's own `rot`
        world[b["name"]] = (np.eye(3), piv[b["name"]])
    out = []
    for e in spec["elements"]:
        R, t = world[e["bone"]]
        E = rot(*e["rot"]) if e.get("rot") and any(e["rot"]) else np.eye(3)
        o = np.array(e["origin"], float) if e.get("rot") and e.get("origin") else np.zeros(3)
        p0 = piv[e["bone"]]
        A = R @ E
        c = R @ (o - E @ o - p0) + t
        faces = {f: r for f, r in (e.get("faces") or {}).items() if r is not None}
        out.append(dict(name=e["name"], bone=e["bone"], lo=np.array(e["lo"], float), hi=np.array(e["hi"], float),
                        A=A, c=c, quad=e.get("quad"), faces=faces, up=chain(spec["bones"], e["bone"]), pivots=piv))
    return out


def chain(bones, name):
    """The bone and its parents, nearest first, leaving out the tilt bones `to_bedrock` adds."""
    par = {b["name"]: b.get("parent") for b in bones}
    out = []
    while name:
        if not name.endswith("_tilt"):
            out.append(name)
        name = par.get(name)
    return out


GEO_TO_OURS = {"east": "east", "west": "west", "up": "up", "down": "down", "north": "south", "south": "north"}


def boxes_from_geo(geo):
    """The same boxes from an exported Bedrock model (`to_bedrock` turned it to face -z: turned
    back here, so faces and names read as in the spec). Cubes are named <bone>#<n>."""
    g = geo["minecraft:geometry"][0]
    F = np.diag([-1.0, 1.0, -1.0])
    bones = {b["name"]: b for b in g["bones"]}
    world = {}

    def wb(name):
        if name in world:
            return world[name]
        b = bones[name]
        piv = F @ np.array(b.get("pivot", [0, 0, 0]), float)
        R = rot(*b["rotation"]) if b.get("rotation") else np.eye(3)
        if b.get("parent") in bones:
            PA, Pc = wb(b["parent"])
            A, c = PA @ R, PA @ (piv - R @ piv) + Pc
        else:
            A, c = R, piv - R @ piv
        world[name] = (A, c)
        return world[name]

    out = []
    pivots = {n: wb(n)[0] @ (F @ np.array(bones[n].get("pivot", [0, 0, 0]), float)) + wb(n)[1] for n in bones}
    for b in g["bones"]:
        A, c = wb(b["name"])
        for k, cube in enumerate(b.get("cubes", [])):
            o, s = np.array(cube["origin"], float), np.array(cube["size"], float)
            inf = float(cube.get("inflate", 0))
            a, z = F @ (o - inf), F @ (o + s + inf)
            lo, hi = np.minimum(a, z), np.maximum(a, z)
            if cube.get("rotation"):
                cp = F @ np.array(cube.get("pivot", [0, 0, 0]), float)
                E = rot(*cube["rotation"])
                A2, c2 = A @ E, A @ (cp - E @ cp) + c
            else:
                A2, c2 = A, c
            faces = {}
            per_face = cube.get("uv") if isinstance(cube.get("uv"), dict) else {}  # box UV (a list): no per-face rectangles
            for gf, uv in per_face.items():
                if not isinstance(uv, dict):
                    continue
                u, v = uv["uv"]
                w, h = uv["uv_size"]
                faces[GEO_TO_OURS[gf]] = [min(u, u + w), min(v, v + h), max(u, u + w), max(v, v + h)]
            thin = [ax for ax in range(3) if s[ax] + 2 * inf <= 1e-6]
            quad = None
            if thin:  # a flat panel: name it by the face that carries a texture
                ax = thin[0]
                quad = next((f for f, (a_, _) in FACES.items() if a_ == ax and f in faces), None) or "up"
            out.append(dict(name=f"{b['name']}#{k}", bone=b["name"], lo=lo, hi=hi, A=A2, c=c2, quad=quad, faces=faces,
                            up=chain(g["bones"], b["name"]), pivots=pivots))
    return out


# ---------------------------------------------------------------- geometry

def face_points(b, face, nu, nv):
    """World points at the centres of an nu x nv grid on the face (u right, v down from outside),
    their outward normal, and the face's size in the element's units along u and v."""
    lo, hi = b["lo"], b["hi"]
    spec, du, dv = FRAMES[face]
    pick = {"lo": lo, "hi": hi}
    corner = np.array([pick[s[:2]][int(s[2])] for s in spec.split()], float)
    du, dv = np.array(du, float), np.array(dv, float)
    size = hi - lo
    U, V = du * abs(size @ du), dv * abs(size @ dv)
    uu, vv = np.meshgrid((np.arange(nu) + 0.5) / nu, (np.arange(nv) + 0.5) / nv)
    p = corner + uu[..., None] * U + vv[..., None] * V
    ax, sg = FACES[face]
    n = np.zeros(3)
    n[ax] = sg
    return p @ b["A"].T + b["c"], b["A"] @ n, abs(size @ du), abs(size @ dv)


def inside(b, w, eps=1e-4):
    """Whether world points `w` (..., 3) lie inside box `b` (not on its surface)."""
    p = (w - b["c"]) @ b["A"]  # A is a rotation: its inverse is its transpose
    return np.all((p > b["lo"] + eps) & (p < b["hi"] - eps), axis=-1)


def world_bounds(b):
    lo, hi = b["lo"], b["hi"]
    c = np.array([[x, y, z] for x in (lo[0], hi[0]) for y in (lo[1], hi[1]) for z in (lo[2], hi[2])])
    w = c @ b["A"].T + b["c"]
    return w.min(0), w.max(0)


def texels_of(b, face, du_len, dv_len, density):
    """The face's texels along u and v: from its texture rectangle, else from the density."""
    r = b["faces"].get(face)
    if r is not None:
        return max(1, int(round(abs(r[2] - r[0])))), max(1, int(round(abs(r[3] - r[1]))))
    return max(1, int(round(du_len * density))), max(1, int(round(dv_len * density)))


def face_colours(b, face, tex, nu, nv, w, h):
    """The texture's colour at each sample (nv, nu, 3), or None."""
    r = b["faces"].get(face)
    if tex is None or r is None:
        return None
    u0, v0 = int(round(r[0])), int(round(r[1]))
    ui = np.clip((np.arange(nu) * w) // nu, 0, w - 1) + u0
    vi = np.clip((np.arange(nv) * h) // nv, 0, h - 1) + v0
    ui = np.clip(ui, 0, tex.shape[1] - 1)
    vi = np.clip(vi, 0, tex.shape[0] - 1)
    return tex[vi[:, None], ui[None, :], :4].astype(float)


def mean_face_colour(b, face, tex):
    r = b["faces"].get(face)
    if tex is None or r is None:
        return None
    u0, v0, u1, v1 = (int(round(x)) for x in r)
    patch = tex[v0:v1, u0:u1]
    if patch.size == 0:
        return None
    a = patch[..., 3] > 0
    return patch[..., :3][a].mean(0) if a.any() else None


def colour_beside(cover, pts, tex):
    """The mean colour of `cover`'s surface nearest world points `pts` (n, 3): what shows right
    next to a strip."""
    q = (pts - cover["c"]) @ cover["A"]
    lo, hi = cover["lo"], cover["hi"]
    over = np.stack([lo - q, q - hi], -1)  # (n, 3, 2): how far outside each side
    k = over.reshape(len(q), 6).argmax(1)
    cl = np.clip(q, lo, hi)
    cols = []
    for idx in range(len(q)):
        ax, side = divmod(int(k[idx]), 2)
        face = {(0, 0): "west", (0, 1): "east", (1, 0): "down", (1, 1): "up", (2, 0): "north", (2, 1): "south"}[(ax, side)]
        r = cover["faces"].get(face)
        if r is None:
            continue
        spec, du, dv = FRAMES[face]
        pick = {"lo": lo, "hi": hi}
        corner = np.array([pick[s_[:2]][int(s_[2])] for s_ in spec.split()], float)
        du, dv = np.array(du, float), np.array(dv, float)
        size = hi - lo
        fu = ((cl[idx] - corner) @ du) / max(abs(size @ du), 1e-9)
        fv = ((cl[idx] - corner) @ dv) / max(abs(size @ dv), 1e-9)
        u0, v0, u1, v1 = r
        x = int(np.clip(math.floor(u0 + fu * (u1 - u0)), min(u0, u1), max(u0, u1) - 1))
        y = int(np.clip(math.floor(v0 + fv * (v1 - v0)), min(v0, v1), max(v0, v1) - 1))
        if 0 <= y < tex.shape[0] and 0 <= x < tex.shape[1] and tex[y, x, 3] > 0:
            cols.append(tex[y, x, :3].astype(float))
    return np.mean(cols, 0) if cols else None


def facing(b, n):
    """The box's face whose outward normal is nearest `n` (world)."""
    best, bd = None, -2
    for f, (ax, sg) in FACES.items():
        m = np.zeros(3)
        m[ax] = sg
        d = float((b["A"] @ m) @ n)
        if d > bd:
            best, bd = f, d
    return best


# ---------------------------------------------------------------- the check

def runs(mask):
    """For each True cell, the length of the run of True cells along its row it belongs to."""
    out = np.zeros(mask.shape, float)
    for r in range(mask.shape[0]):
        row = mask[r]
        if not row.any():
            continue
        d = np.diff(np.r_[0, row.astype(np.int8), 0])
        starts, ends = np.nonzero(d == 1)[0], np.nonzero(d == -1)[0]
        for a_, b_ in zip(starts, ends):
            out[r, a_:b_] = b_ - a_
    return out


def allowed(name, face, allow):
    return any(name == a or f"{name}.{face}" == a or (a.endswith("*") and name.startswith(a[:-1])) for a in allow)


def lines(boxes, tex=None, density=0.5, allow=()):
    """[{kind, box, face, width, length, cover, reach, contrast}] (width, length, reach in the
    face's texels). `density`: texels a unit, for faces with no texture rectangle."""
    from scipy import ndimage

    solid = [b for b in boxes if not b["quad"] and np.all(b["hi"] - b["lo"] > 1e-6)]
    bounds = [world_bounds(b) for b in solid]
    hits = []
    for i, b in enumerate(solid):
        for face in FACES:
            if b["faces"] and face not in b["faces"]:
                continue  # not drawn
            size = b["hi"] - b["lo"]
            _, du, dv = FRAMES[face]
            du_len, dv_len = abs(size @ np.array(du, float)), abs(size @ np.array(dv, float))
            w, h = texels_of(b, face, du_len, dv_len, density)
            nu, nv = w * SPT, h * SPT
            if nu * nv > 400_000:
                nu, nv = min(nu, 600), min(nv, 600)
            pts, n, _, _ = face_points(b, face, nu, nv)
            probe = pts + n * 0.02  # just outside the face
            plo, phi = probe.reshape(-1, 3).min(0), probe.reshape(-1, 3).max(0)
            cover_id = np.full((nv, nu), -1)
            for j, c in enumerate(solid):
                if j == i:
                    continue
                clo, chi = bounds[j]
                if np.any(chi < plo - 1e-3) or np.any(clo > phi + 1e-3):
                    continue
                m = inside(c, probe) & (cover_id < 0)
                cover_id[m] = j
            covered = cover_id >= 0
            # texels per sample along u and v (a face's texels need not be square)
            tu, tv = w / nu, h / nv
            if not covered.any():  # a whole face under a texel across, nothing on it
                if min(du_len * density, dv_len * density) < 1 and max(du_len, dv_len) * density >= MIN_LEN:
                    hits.append(dict(kind="thin", box=b["name"], face=face, width=round(min(du_len, dv_len) * density, 2),
                                     length=round(max(du_len, dv_len) * density, 1), cover=None, reach=None, contrast=None))
                continue
            vis = ~covered
            if not vis.any():
                continue
            # The thin parts of what shows: what an opening by a square MAX_W texels a side takes
            # away (a strip along a wide area's edge stays its own part: the U round a pad).
            sq = (max(1, int(round(MAX_W / tv))), max(1, int(round(MAX_W / tu))))
            thin = vis & ~ndimage.binary_opening(vis, structure=np.ones(sq, bool), border_value=0)
            lab, k = ndimage.label(thin)
            if k == 0:
                continue
            across = np.minimum(runs(vis) * tu, runs(vis.T).T * tv)  # how wide what shows is, at each sample
            cols = None
            for comp in range(1, k + 1):
                m = lab == comp
                width = float(np.median(across[m]))  # the strip's width, in texels
                units = width / max(min(w / max(du_len, 1e-9), h / max(dv_len, 1e-9)), 1e-9)
                if width >= 0.95 * MAX_W or units < FLUSH:  # a wedge where a tilted box crosses, or flush
                    continue
                area = m.sum() * tu * tv
                length = area / max(width, 1e-6)
                if length < MIN_LEN:
                    continue
                # the box that covers the rest beside it
                ring = ndimage.binary_dilation(m, iterations=2) & covered
                ids = cover_id[ring]
                if ids.size == 0:
                    continue
                cj = int(np.bincount(ids).argmax())
                cover = solid[cj]
                M = cover["A"].T @ b["A"]
                if not np.allclose(np.abs(M), np.round(np.abs(M)), atol=1e-3):
                    continue  # boxes at an angle: a wedge where one enters the other (ground.pokes)
                contrast = None
                if tex is not None:
                    if cols is None:
                        cols = face_colours(b, face, tex, nu, nv, w, h)
                    near = pts[m]
                    if len(near) > 400:
                        near = near[:: len(near) // 400 + 1]
                    cc = colour_beside(cover, near, tex)
                    if cols is not None and (cols[m][:, 3] > 0).mean() < 0.2:
                        continue  # cut out: the strip's texels are see-through (a shell's open hem)
                    if cols is not None and cc is not None:
                        shown = cols[m][cols[m][:, 3] > 0]
                        contrast = round(float(np.linalg.norm(shown[:, :3].mean(0) - cc)), 1)
                kind = "line" if contrast is None or contrast >= CONTRAST else "seam"
                if joint_rim(b, cover, n):
                    kind = "seam"  # the end of a part round the next one in its chain (a hoof's rim)
                elif n[1] < -0.7 and kind == "line":
                    kind = "under"  # on an underside: listed, seen only from below
                if allowed(b["name"], face, allow):
                    kind = "seam"  # meant (a build's `allow`: names, name.face, or prefix*)
                r = b["faces"].get(face)
                texels = set()
                if r is not None:
                    rr, cc_ = np.nonzero(m)
                    u0, v0 = int(round(min(r[0], r[2]))), int(round(min(r[1], r[3])))
                    texels = {(u0 + int(c_ * w // nu), v0 + int(r_ * h // nv)) for r_, c_ in zip(rr, cc_)}
                hits.append(dict(kind=kind, box=b["name"], face=face, width=round(width, 2), length=round(length, 1),
                                 cover=cover["name"], reach=round(width, 2), contrast=contrast, texels=texels))
    return merged(hits)


def joint_rim(b, cover, n):
    """Whether a strip of `b` beside `cover` is the rim a chain's joint leaves: the two on
    neighbouring bones (one the other's parent) and the strip on `b`'s end facing the joint (a
    hoof's top round the shank, the chest's back round the hips). A strip on a face along the
    part (a thigh's side poking a texel above the hips) is not."""
    sb, cb = b["up"][0], cover["up"][0]
    if sb == cb:
        return False
    if cb in b["up"][1:2]:
        joint = b["pivots"][sb]
    elif sb in cover["up"][1:2]:
        joint = cover["pivots"][cb]
    else:
        return False
    d = joint - (b["A"] @ ((b["lo"] + b["hi"]) / 2) + b["c"])
    L = np.linalg.norm(d)
    return L > 1e-6 and float(d @ n) / L > 0.7


def merged(hits):
    """One hit per box face and cover: the widest strip, the lengths added."""
    out = {}
    for h in hits:
        k = (h["kind"], h["box"], h["face"], h["cover"], h.get("why"))
        if k in out:
            o = out[k]
            o["length"] = round((o["length"] or 0) + (h["length"] or 0), 1)
            o["width"] = max(o["width"], h["width"]) if o["width"] is not None else h["width"]
            o["reach"] = o["width"]
            o["count"] += 1
            o["texels"] = o.get("texels", set()) | h.get("texels", set())
        else:
            out[k] = {**h, "count": 1}
    return list(out.values())


def bleed(boxes, tex, tol=24):
    """Faces whose texture rectangle isn't whole texels or whose gutter isn't its own edge."""
    hits = []
    if tex is None:
        return hits
    H, W = tex.shape[:2]
    seen = set()
    for b in boxes:
        for face, r in b["faces"].items():
            u0, v0, u1, v1 = r
            key = (u0, v0, u1, v1)
            if key in seen:
                continue
            seen.add(key)
            if any(abs(x - round(x)) > 1e-3 for x in r):
                hits.append(dict(kind="bleed", box=b["name"], face=face, width=None, length=None, cover=None, reach=None,
                                 contrast=None, why=f"rectangle {r} not on whole texels"))
                continue
            u0, v0, u1, v1 = (int(round(x)) for x in r)
            if u1 - u0 < 1 or v1 - v0 < 1:
                continue
            bad = 0
            for (edge, gut) in (((slice(v0, v1), u0), (slice(v0, v1), u0 - 1)), ((slice(v0, v1), u1 - 1), (slice(v0, v1), u1)),
                                ((v0, slice(u0, u1)), (v0 - 1, slice(u0, u1))), ((v1 - 1, slice(u0, u1)), (v1, slice(u0, u1)))):
                gi = [g.start if isinstance(g, slice) else g for g in gut]
                if not (0 <= (gut[1] if not isinstance(gut[1], slice) else 0) < W and 0 <= (gut[0] if not isinstance(gut[0], slice) else 0) < H):
                    continue  # the atlas edge: clamped, so it repeats the edge itself
                e, g = tex[edge].astype(int), tex[gut].astype(int)
                opaque = e[..., 3] > 0
                d = np.abs(e[..., :3] - g[..., :3]).max(-1)
                bad += int(((d > tol) | (g[..., 3] == 0)) [opaque].sum())
            if bad:
                hits.append(dict(kind="bleed", box=b["name"], face=face, width=None, length=None, cover=None, reach=None,
                                 contrast=None, why=f"{bad} gutter texels unlike the face's edge"))
    return hits


def load(path, texture=None):
    data = json.load(open(path))
    if "minecraft:geometry" in data:
        boxes = boxes_from_geo(data)
        g = data["minecraft:geometry"][0]
    else:
        boxes = boxes_from_spec(data)
    tex = None
    if texture and os.path.exists(texture):
        from PIL import Image
        tex = np.array(Image.open(texture).convert("RGBA"))
    return boxes, tex


def density_of(boxes):
    """Texels a unit, the most common over the faces with a texture rectangle."""
    d = []
    for b in boxes:
        size = b["hi"] - b["lo"]
        for f, r in b["faces"].items():
            _, du, _ = FRAMES[f]
            L = abs(size @ np.array(du, float))
            if L > 1e-6:
                d.append(round(abs(r[2] - r[0]) / L, 2))
    return float(np.median(d)) if d else 0.5


def report(name, hits, show_all=False):
    """Prints the hits (seams and undersides only with `show_all`); returns the failing ones."""
    order = "line thin bleed under seam".split()
    bad = [h for h in hits if h["kind"] in ("line", "bleed")]
    for h in sorted(hits, key=lambda h: (order.index(h["kind"]), -(h["length"] or 0))):
        if h["kind"] in ("seam", "under", "thin") and not show_all:
            continue
        if h["kind"] in ("line", "seam", "under"):
            c = f", contrast {h['contrast']:.0f}" if h["contrast"] is not None else ""
            n = f" ({h['count']} strips)" if h.get("count", 1) > 1 else ""
            print(f"  {h['kind']:5} {h['box']}.{h['face']}: {h['width']:.2g} texels wide, {h['length']:g} long{n}, beside "
                  f"{h['cover']}{c}")
        elif h["kind"] == "thin":
            print(f"  thin  {h['box']}.{h['face']}: the whole face {h['width']} texels across, {h['length']} long")
        else:
            print(f"  bleed {h['box']}.{h['face']}: {h['why']}")
    counts = {k: sum(h["kind"] == k for h in hits) for k in order}
    print(f"{name}: {counts['line']} lines, {counts['bleed']} bleed ({counts['thin']} thin faces, "
          f"{counts['under']} on undersides, {counts['seam']} seams: listed with --all-kinds)")
    return bad


MARK = {"line": (255, 0, 220), "under": (255, 140, 0), "seam": (0, 200, 255)}


def as_spec(boxes, tex):
    """A spec `render.py` can draw: every box on the root, its rotation as an element's own."""
    els = []
    for b in boxes:
        A, c = b["A"], b["c"]
        if np.allclose(A, np.eye(3)):
            lo, hi, rot_, origin = b["lo"] + c, b["hi"] + c, None, None
        else:
            ry = math.degrees(math.asin(max(-1.0, min(1.0, -A[2, 0]))))
            rx = math.degrees(math.atan2(A[2, 1], A[2, 2]))
            rz = math.degrees(math.atan2(A[1, 0], A[0, 0]))
            origin = np.linalg.lstsq(np.eye(3) - A, c, rcond=None)[0]
            lo, hi, rot_ = b["lo"], b["hi"], [rx, ry, rz]
            if not np.allclose((np.eye(3) - A) @ origin, c, atol=1e-3):  # not a turn about one point
                continue
        els.append(dict(name=b["name"], bone="root", lo=list(lo), hi=list(hi), rot=rot_,
                        origin=list(origin) if origin is not None else None, quad=b["quad"], faces=b["faces"]))
    return {"texture": [tex.shape[1], tex.shape[0]], "bones": [{"name": "root", "pivot": [0, 0, 0]}], "elements": els}


def shot(boxes, tex, hits, out, kinds=("line", "under")):
    """The model from front-left above, back-right above and the side, with each strip marked
    (lines magenta, undersides orange); the plain model beside for comparison."""
    from PIL import Image
    from render import render
    marked = tex.copy()
    for h in hits:
        if h["kind"] in kinds:
            for (x, y) in h.get("texels", ()):
                if 0 <= y < marked.shape[0] and 0 <= x < marked.shape[1]:
                    marked[y, x, :3] = MARK[h["kind"]]
    spec = as_spec(boxes, tex)
    lo = np.min([np.minimum(e["lo"], e["hi"]) for e in spec["elements"]], 0)
    hi = np.max([np.maximum(e["lo"], e["hi"]) for e in spec["elements"]], 0)
    ppu = 900.0 / max(hi - lo)
    rows = []
    for t in (tex, marked):
        ims = [render(spec, t, {}, yaw, pitch, ppu, bg=(236, 233, 228, 255)) for yaw, pitch in ((-40, -35), (140, -35), (90, -10))]
        ims = [i if isinstance(i, Image.Image) else Image.fromarray(i) for i in ims]
        rows.append(ims)
    W = sum(i.width for i in rows[0])
    H = sum(max(i.height for i in r) for r in rows)
    sheet = Image.new("RGBA", (W, H), (236, 233, 228, 255))
    y = 0
    for r in rows:
        x = 0
        for i in r:
            sheet.paste(i, (x, y))
            x += i.width
        y += max(i.height for i in r)
    sheet.save(out)
    return out


def texture_for(geo_path, repo):
    """The texture an exported model uses: textures/entity/<name>/<name>.png or <name>.png."""
    mod = os.path.dirname(os.path.dirname(geo_path))
    name = os.path.basename(geo_path).replace(".geo.json", "")
    for base in (name, name.replace("_pup", "").replace("_fly", "")):
        for cand in (f"textures/entity/{base}/{base}.png", f"textures/entity/{base}.png"):
            p = os.path.join(mod, cand)
            if os.path.exists(p):
                return p
        hits = glob.glob(os.path.join(mod, "textures", "entity", "**", f"{base}.png"), recursive=True)
        if hits:
            return hits[0]
    return None


if __name__ == "__main__":
    argv = sys.argv[1:]
    if "--shot" in argv:
        i = argv.index("--shot")
        argv = argv[:i] + argv[i + 2:]
    args = [a for a in argv if not a.startswith("--")]
    seams = "--all-kinds" in sys.argv
    if "--all" in sys.argv:
        repo = args[0] if args else "."
        total = 0
        for geo in sorted(glob.glob(os.path.join(repo, "mods", "*", "assets", "models", "*.geo.json"))):
            tex = texture_for(geo, repo)
            boxes, t = load(geo, tex)
            hits = lines(boxes, t, density_of(boxes)) + bleed(boxes, t)
            total += len(report(os.path.relpath(geo, repo) + ("" if tex else " (no texture found)"), hits, seams))
        sys.exit(1 if total else 0)
    boxes, tex = load(args[0], args[1] if len(args) > 1 else None)
    hits = lines(boxes, tex, density_of(boxes)) + bleed(boxes, tex)
    bad = report(args[0], hits, seams)
    if "--shot" in sys.argv and tex is not None:
        out = sys.argv[sys.argv.index("--shot") + 1]
        print("wrote", shot(boxes, tex, hits, out))
    sys.exit(1 if bad else 0)
