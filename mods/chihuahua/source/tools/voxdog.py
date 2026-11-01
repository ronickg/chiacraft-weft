"""Shared pieces for every way of making the voxel dog: the palette, the skeleton and its
animations, rigid rigging (each voxel belongs to one bone), a greedy mesher, and writers for
GLB (animations embedded) and MagicaVoxel .vox, plus a small software renderer for previews.

Coordinates: voxel units, x to the dog's left, y up from the ground, z forward (the dog looks
along +Z, as glTF expects). A model is a dense grid `col` (X, Y, Z) of palette indices
(-1 = empty) with the grid's origin given by `origin` (the voxel at index (0,0,0) sits at
`origin` in voxel units). `bone` (same shape) says which bone each voxel belongs to.
"""

import json
import math
import struct

import numpy as np

# ------------------------------------------------------------------ palette

# The nine swatches on the asset sheet (ref/palette.json) plus the few the face needs.
PALETTE = {
    "brown_dark": (75, 39, 21),
    "brown": (115, 57, 26),
    "brown_light": (146, 72, 36),
    "tan": (206, 139, 93),
    "cream": (241, 214, 188),
    "white": (245, 235, 225),
    "pink": (225, 151, 125),
    "black": (52, 35, 31),
    "grey_white": (228, 217, 209),
    "eye": (150, 82, 34),
    "nose": (128, 62, 62),
    "highlight": (252, 250, 246),
}
NAMES = list(PALETTE)
RGB = np.array([PALETTE[n] for n in NAMES], float)
IDX = {n: i for i, n in enumerate(NAMES)}


def snap(rgb, allowed=None):
    """Nearest palette index for colors (..., 3), in a rough perceptual space."""
    names = allowed or [n for n in NAMES if n not in ("highlight",)]
    pal = np.array([PALETTE[n] for n in names], float)
    w = np.array([0.3, 0.59, 0.11]) ** 0.5 * 2
    d = (((rgb[..., None, :] - pal) * w) ** 2).sum(-1)
    return np.array([IDX[n] for n in names])[d.argmin(-1)]


# ------------------------------------------------------------------ skeleton

LEGS = ["FL", "FR", "BL", "BR"]


def skeleton(L):
    """Bones (name, parent, pivot) from landmarks `L` (voxel units, world). Ordered parents first."""
    b = [
        ("root", None, (0, 0, 0)),
        ("spine_0", "root", (0, L["spine_y"], L["hip_z"])),
        ("spine_1", "spine_0", (0, L["spine_y"], L["mid_z"])),
        ("neck", "spine_1", L["neck"]),
        ("head", "neck", L["head"]),
        ("ear_L", "head", (L["ear_x"], L["ear_y"], L["ear_z"])),
        ("ear_R", "head", (-L["ear_x"], L["ear_y"], L["ear_z"])),
    ]
    tail = L["tail"]
    b += [("tail_0", "spine_0", tail[0]), ("tail_1", "tail_0", tail[1]), ("tail_2", "tail_1", tail[2])]
    for leg in LEGS:
        front = leg[0] == "F"
        x = L["leg_x_f" if front else "leg_x_b"] * (1 if leg[1] == "L" else -1)
        z = L["leg_z_f" if front else "leg_z_b"]
        parent = "spine_1" if front else "spine_0"
        b += [
            (f"leg_{leg}_upper", parent, (x, L["leg_top"], z)),
            (f"leg_{leg}_lower", f"leg_{leg}_upper", (x, L["knee"], z)),
            (f"leg_{leg}_paw", f"leg_{leg}_lower", (x, L["ankle"], z)),
        ]
    return [(n, p, np.array(v, float)) for n, p, v in b]


def autorig(col, origin, L):
    """Bone index per voxel from landmarks. Rules in order: legs, ears, head, tail, body halves."""
    names = [n for n, _, _ in skeleton(L)]
    B = {n: i for i, n in enumerate(names)}
    X, Y, Z = np.meshgrid(*(np.arange(s) + o for s, o in zip(col.shape, origin)), indexing="ij")
    bone = np.full(col.shape, B["spine_0"], int)
    bone[Z >= L["mid_z"]] = B["spine_1"]
    # Tail: near the curve through its pivots (extended past the last one), behind the rump;
    # each voxel goes with the segment it is closest to.
    t = [np.array(p, float) for p in L["tail"]]
    t.append(t[-1] + (t[-1] - t[-2]) * L.get("tail_reach", 1.4))
    P = np.stack([X, Y, Z], -1).astype(float)
    dists = []
    for a, b in zip(t[:-1], t[1:]):
        ab = b - a
        f = np.clip(((P - a) @ ab) / (ab @ ab), 0, 1)
        dists.append(np.linalg.norm(P - (a + f[..., None] * ab), axis=-1))
    dists = np.stack(dists)
    tail = (dists.min(0) < L.get("tail_r", 8)) & (Z < L["tail_z0"] + 1) & (Y > L["tail_y0"])
    seg = dists.argmin(0)
    for k in range(3):
        bone[tail & (seg == k)] = B[f"tail_{k}"]
    # Head (and the ears on it): above the neck line and in front of the neck's back, plus
    # anything well above the back that isn't tail. The neck: the band below the head.
    head = (Y > L["head_y0"]) & (Z > L["neck_z0"]) | (Y > L["head_y0"] + 6) & ~tail & (Z > L["neck_z0"] - 6)
    bone[head] = B["head"]
    neck = (Z > L["neck_z0"]) & (Y > L["neck_y0"]) & ~head
    bone[neck] = B["neck"]
    ears = (Y > L["ear_y"]) & (np.abs(X) > L["ear_x0"]) & (Z > L["head_z0"] - 4)
    bone[ears & (X > 0)] = B["ear_L"]
    bone[ears & (X < 0)] = B["ear_R"]
    # Legs: columns under the belly.
    for leg in LEGS:
        front = leg[0] == "F"
        lx = L["leg_x_f" if front else "leg_x_b"] * (1 if leg[1] == "L" else -1)
        lz = L["leg_z_f" if front else "leg_z_b"]
        col_ = (np.abs(X - lx) <= L["leg_r"]) & (np.abs(Z - lz) <= L["leg_r"] + 1) & (Y < L["leg_top"])
        bone[col_ & (Y >= L["knee"])] = B[f"leg_{leg}_upper"]
        bone[col_ & (Y < L["knee"]) & (Y >= L["ankle"])] = B[f"leg_{leg}_lower"]
        paw = (np.abs(X - lx) <= L["leg_r"] + 1.5) & (np.abs(Z - lz) <= L["leg_r"] + 2.5) & (Y < L["ankle"])
        bone[paw] = B[f"leg_{leg}_paw"]
    bone[col < 0] = -1
    return bone


# ------------------------------------------------------------------ animation


def quat(rx, ry, rz):
    """Quaternion (x, y, z, w) for Euler degrees applied X, then Y, then Z (q = qz qy qx)."""

    def axis(a, i):
        h = math.radians(a) / 2
        q = [0.0, 0.0, 0.0, math.cos(h)]
        q[i] = math.sin(h)
        return np.array(q)

    return qmul(axis(rz, 2), qmul(axis(ry, 1), axis(rx, 0)))


def qmul(a, b):
    ax, ay, az, aw = a
    bx, by, bz, bw = b
    return np.array([
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ])


def qmat(q):
    x, y, z, w = q
    return np.array([
        [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
        [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
        [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
    ])


def euler(q):
    """Euler degrees (x, y, z) for quaternion q, in quat()'s convention (R = Rz Ry Rx)."""
    R = qmat(q)
    ry = math.degrees(math.asin(max(-1.0, min(1.0, -R[2, 0]))))
    rx = math.degrees(math.atan2(R[2, 1], R[2, 2]))
    rz = math.degrees(math.atan2(R[1, 0], R[0, 0]))
    return (rx, ry, rz)


def tail_wag(L, k, wag, base=(0, 0, 0)):
    """Rotation for tail bone k: its base pose, then a side-to-side wag of `wag` degrees about
    the axis across that segment (perpendicular to the segment and to the dog's sides). The
    plume stands up and curls forward, so a wag about the vertical axis would only twist it
    about itself; this swings it side to side whichever way the segment points."""
    t = [np.array(p, float) for p in L["tail"]]
    d = t[k + 1] - t[k] if k + 1 < len(t) else t[k] - t[k - 1]
    a = np.cross(d, (1.0, 0.0, 0.0))
    a /= np.linalg.norm(a)
    h = math.radians(wag) / 2
    qw = np.array([*(a * math.sin(h)), math.cos(h)])
    return euler(qmul(quat(*base), qw))


# Angles: +X pitches a bone's +Z end down (nose down, a hanging leg swings back), +Y turns
# +Z toward +X (the dog's left), +Z rolls +Y toward -X.
S = math.sin
TAU = 2 * math.pi


def pose_stand(t, L):
    """Idle: breathing, a slow head tilt, one ear twitch, a tail wag (2.4 s loop)."""
    p = t / 2.4
    br = S(TAU * p * 2)
    tw = max(0.0, S(TAU * (p - 0.55) * 4)) if 0.55 < p < 0.68 else 0.0
    return {
        "root": (0, 0.12 * br, 0),
        "spine_1": (1.0 * br, 0, 0),
        "neck": (-1.5 * br, 0, 0),
        "head": (2 * S(TAU * p), 4 * S(TAU * p), 5 * S(TAU * p)),
        "ear_L": (-14 * tw, 0, -10 * tw),
        "ear_R": (0, 0, 2 * br),
        "tail_0": tail_wag(L, 0, 16 * S(TAU * p * 4)),
        "tail_1": tail_wag(L, 1, 12 * S(TAU * p * 4 - 0.8)),
        "tail_2": tail_wag(L, 2, 10 * S(TAU * p * 4 - 1.6)),
    }


def legs_gait(p, phases, swing, lift, L):
    out = {}
    for leg, ph in zip(LEGS, phases):
        a = TAU * (p + ph)
        up = max(0.0, S(a + math.pi / 2))  # lifted while swinging forward
        out[f"leg_{leg}_upper"] = (-swing * S(a), 0, 0)
        knee = lift * up
        if leg[0] == "F":
            out[f"leg_{leg}_lower"] = (knee, 0, 0)
            out[f"leg_{leg}_paw"] = (-0.6 * knee, 0, 0)
        else:
            out[f"leg_{leg}_lower"] = (-knee, 0, 0)
            out[f"leg_{leg}_paw"] = (0.7 * knee, 0, 0)
    return out


def pose_walk(t, L):
    """Walk: diagonal pairs (FL with BR), 0.8 s loop, matching the sheet's four key frames."""
    p = t / 0.8
    q = legs_gait(p, (0.0, 0.5, 0.5, 0.0), 24, 34, L)
    bob = abs(S(TAU * p))
    q.update({
        "root": (0, 0.35 * bob, 0),
        "spine_1": (0, 0, 2 * S(TAU * p)),
        "neck": (-2 * bob, 0, 0),
        "head": (2 * S(TAU * p * 2), 3 * S(TAU * p), 0),
        "ear_L": (6 * S(TAU * p * 2 + 1), 0, 0),
        "ear_R": (6 * S(TAU * p * 2 + 1.4), 0, 0),
        "tail_0": tail_wag(L, 0, 14 * S(TAU * p * 2)),
        "tail_1": tail_wag(L, 1, 10 * S(TAU * p * 2 - 0.8)),
        "tail_2": tail_wag(L, 2, 8 * S(TAU * p * 2 - 1.6)),
    })
    return q


def pose_run(t, L):
    """Run: a bounding gait, fronts together then backs, the spine flexing (0.45 s loop)."""
    p = t / 0.45
    q = legs_gait(p, (0.0, 0.08, 0.5, 0.58), 42, 55, L)
    c = S(TAU * p)
    q.update({
        "root": (0, 1.2 * max(0.0, S(TAU * p + 0.6)), 0),
        "spine_0": (-6 * c, 0, 0),
        "spine_1": (9 * c, 0, 0),
        "neck": (-6 * c, 0, 0),
        "head": (-3 * c, 0, 0),
        "ear_L": (22 + 8 * c, 0, -10),
        "ear_R": (22 + 8 * c, 0, 10),
        "tail_0": (18 + 6 * c, 0, 0),
        "tail_1": tail_wag(L, 1, 6 * S(TAU * p), (8, 0, 0)),
        "tail_2": tail_wag(L, 2, 6 * S(TAU * p - 1)),
    })
    return q


def pose_sit(t, L):
    """Sit: rump down, back legs folded under, front legs straight, head level (2.4 s loop).
    Every paw is kept flat on the ground (its pitch cancels its parents'). The angles and drop
    are per model in L (sit_angle, sit_hip, sit_knee, sit_lean, sit_drop, sit_back): fit them with
    tools/ground.py so the rump and all four paws rest on the ground."""
    p = t / 2.4
    br = S(TAU * p * 2)
    a = L.get("sit_angle", 40)  # how far the body tilts up
    hip, knee, lean = L.get("sit_hip", -70), L.get("sit_knee", 110), L.get("sit_lean", -8)
    q = {
        "root": (0, -L["sit_drop"] + 0.1 * br, -L["sit_back"]),
        "spine_0": (-a, 0, 0),
        "spine_1": (1 * br, 0, 0),
        "neck": (a * 0.55, 0, 0),
        "head": (a * 0.45 + 3 * S(TAU * p), 6 * S(TAU * p), 6 * S(TAU * p)),
        "tail_0": (a, 0, 0),  # cancels the body's tilt: the tail stays upright, as when standing
        "tail_1": tail_wag(L, 1, 10 * S(TAU * p * 3)),
        "tail_2": tail_wag(L, 2, 10 * S(TAU * p * 3 - 1)),
    }
    for leg in ("BL", "BR"):
        q[f"leg_{leg}_upper"] = (a + hip, 0, 0)
        q[f"leg_{leg}_lower"] = (knee, 0, 0)
        q[f"leg_{leg}_paw"] = (-hip - knee, 0, 0)
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (a + lean, 0, 0)
        q[f"leg_{leg}_lower"] = (0, 0, 0)
        q[f"leg_{leg}_paw"] = (-lean, 0, 0)
    return q


def pose_lie(t, L):
    """Lie down, head on paws: belly on the ground, front legs forward, back legs tucked. Paws
    flat; per-model angles and drop in L (lie_front, lie_hip, lie_knee, lie_drop), fit with
    tools/ground.py."""
    p = t / 3.0
    br = S(TAU * p * 2)
    q = {
        "root": (0, -L["lie_drop"] + 0.12 * br, 0),
        "spine_1": (0.8 * br, 0, 0),
        "neck": (L["lie_neck"], 0, 0),
        "head": (L["lie_head"], 0, 3 * S(TAU * p)),
        "ear_L": (12, 0, -14),
        "ear_R": (12, 0, 14),
        "tail_0": (50, 30, 0),
        "tail_1": tail_wag(L, 1, 6 * S(TAU * p * 2), (10, 20, 0)),
        "tail_2": tail_wag(L, 2, 6 * S(TAU * p * 2 - 1), (0, 10, 0)),
    }
    front, hip, knee = L.get("lie_front", -85), L.get("lie_hip", -80), L.get("lie_knee", 140)
    for leg in ("FL", "FR"):
        q[f"leg_{leg}_upper"] = (front, 0, 0)
        q[f"leg_{leg}_lower"] = (0, 0, 0)
        q[f"leg_{leg}_paw"] = (-front, 0, 0)
    for leg, s in (("BL", 1), ("BR", -1)):
        q[f"leg_{leg}_upper"] = (hip, 0, s * 12)
        q[f"leg_{leg}_lower"] = (knee, 0, 0)
        q[f"leg_{leg}_paw"] = (-hip - knee, 0, 0)
    return q


ANIMS = {"idle": (pose_stand, 2.4), "walk": (pose_walk, 0.8), "run": (pose_run, 0.45), "sit": (pose_sit, 2.4), "lie": (pose_lie, 3.0)}


def world_transforms(skel, pose):
    """Each bone's (R, t) mapping its local frame (origin at its pivot) to the world."""
    out = {}
    for name, parent, pivot in skel:
        rx, ry, rz = pose.get(name, (0, 0, 0)) if name != "root" else (0, 0, 0)
        R = qmat(quat(rx, ry, rz))
        local_t = pivot - (skel_pivot(skel, parent) if parent else 0)
        if name == "root":
            local_t = local_t + np.array(pose.get("root", (0, 0, 0)), float)
        if parent:
            PR, Pt = out[parent]
            out[name] = (PR @ R, Pt + PR @ local_t)
        else:
            out[name] = (R, local_t)
    return out


def skel_pivot(skel, name):
    for n, _, p in skel:
        if n == name:
            return p
    raise KeyError(name)


# ------------------------------------------------------------------ model


class Model:
    """A voxel dog: colors, bones, landmarks."""

    def __init__(self, col, origin, bone, L, name):
        self.col, self.origin, self.bone, self.L, self.name = col, np.array(origin), bone, L, name
        self.skel = skeleton(L)

    def faces(self):
        """Exposed faces per bone: {bone: (centers_local (n,3), normals (n,3), colors idx (n,))}.
        A face is exposed when the neighbor is empty or belongs to another bone (parts move)."""
        col, bone = self.col, self.bone
        out = {}
        for bi, (name, _, pivot) in enumerate(self.skel):
            m = bone == bi
            if not m.any():
                continue
            cs, ns, ks = [], [], []
            for axis in range(3):
                for sgn in (-1, 1):
                    nb = np.zeros_like(m)
                    sl_src = [slice(None)] * 3
                    sl_dst = [slice(None)] * 3
                    if sgn > 0:
                        sl_src[axis], sl_dst[axis] = slice(1, None), slice(None, -1)
                    else:
                        sl_src[axis], sl_dst[axis] = slice(None, -1), slice(1, None)
                    nb[tuple(sl_dst)] = m[tuple(sl_src)]
                    exp = m & ~nb
                    idx = np.argwhere(exp)
                    n = np.zeros(3)
                    n[axis] = sgn
                    c = idx + self.origin + 0.5 * n - pivot
                    cs.append(c)
                    ns.append(np.repeat(n[None], len(idx), 0))
                    ks.append(col[tuple(idx.T)])
            out[name] = (np.concatenate(cs), np.concatenate(ns), np.concatenate(ks))
        return out

    def stats(self):
        n = int((self.col >= 0).sum())
        counts = {name: int((self.bone == i).sum()) for i, (name, _, _) in enumerate(self.skel)}
        return n, counts


# ------------------------------------------------------------------ greedy mesher


def greedy_quads(model):
    """Per bone: quads (corners (n,4,3) local, normal (n,3), color idx (n,)), greedily merged
    over same-color faces in each slice."""
    col, bone = model.col, model.bone
    out = {}
    for bi, (name, _, pivot) in enumerate(model.skel):
        m = bone == bi
        if not m.any():
            continue
        quads, norms, cols = [], [], []
        for axis in range(3):
            u, v = [a for a in range(3) if a != axis]
            for sgn in (-1, 1):
                nb = np.zeros_like(m)
                a, b = [slice(None)] * 3, [slice(None)] * 3
                if sgn > 0:
                    a[axis], b[axis] = slice(1, None), slice(None, -1)
                else:
                    a[axis], b[axis] = slice(None, -1), slice(1, None)
                nb[tuple(b)] = m[tuple(a)]
                exp = m & ~nb
                for s in np.nonzero(exp.any(axis=tuple(k for k in range(3) if k != axis)))[0]:
                    sl = [slice(None)] * 3
                    sl[axis] = s
                    mask = exp[tuple(sl)].copy()  # indexed (u, v)
                    cc = col[tuple(sl)]
                    for r0, c0, r1, c1, k in _greedy2d(mask, cc):
                        w = s + (1 if sgn > 0 else 0)
                        pts = []
                        for (ru, rv) in ((r0, c0), (r1, c0), (r1, c1), (r0, c1)):
                            p = np.zeros(3)
                            p[axis], p[u], p[v] = w, ru, rv
                            pts.append(p)
                        pts = np.array(pts) + model.origin - pivot
                        if sgn < 0:
                            pts = pts[::-1]
                        # Winding: counter-clockwise seen from outside.
                        if (axis == 1) ^ (sgn < 0) ^ False:
                            pass
                        n = np.zeros(3)
                        n[axis] = sgn
                        if np.dot(np.cross(pts[1] - pts[0], pts[2] - pts[0]), n) < 0:
                            pts = pts[::-1]
                        quads.append(pts)
                        norms.append(n)
                        cols.append(k)
        out[name] = (np.array(quads), np.array(norms), np.array(cols))
    return out


def _greedy2d(mask, cc):
    H, W = mask.shape
    done = ~mask
    res = []
    for r in range(H):
        c = 0
        while c < W:
            if done[r, c]:
                c += 1
                continue
            k = cc[r, c]
            c1 = c + 1
            while c1 < W and not done[r, c1] and cc[r, c1] == k:
                c1 += 1
            r1 = r + 1
            while r1 < H and all(not done[r1, j] and cc[r1, j] == k for j in range(c, c1)):
                r1 += 1
            done[r:r1, c:c1] = True
            res.append((r, c, r1, c1, k))
            c = c1
    return res


# ------------------------------------------------------------------ GLB


def srgb_to_linear(c):
    c = np.asarray(c, float) / 255
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def write_glb(model, path, height_m=0.32, fps=30):
    """The dog as a glTF binary: one node per bone (rigidly parented voxels, vertex colors),
    the animations in ANIMS as node rotations (and the root's translation)."""
    ys = np.nonzero((model.col >= 0).any(axis=(0, 2)))[0]
    vox = height_m / (ys.max() + 1 + model.origin[1])
    quads = greedy_quads(model)
    blob = bytearray()
    views, accs = [], []

    def add(arr, comp, typ, target=None, minmax=False):
        nonlocal blob
        while len(blob) % 4:
            blob += b"\0"
        data = arr.tobytes()
        v = {"buffer": 0, "byteOffset": len(blob), "byteLength": len(data)}
        if target:
            v["target"] = target
        views.append(v)
        blob += data
        a = {"bufferView": len(views) - 1, "componentType": comp, "count": int(arr.shape[0]), "type": typ}
        if minmax:
            a["min"] = arr.min(0).tolist() if arr.ndim > 1 else [float(arr.min())]
            a["max"] = arr.max(0).tolist() if arr.ndim > 1 else [float(arr.max())]
        accs.append(a)
        return len(accs) - 1

    lin = np.concatenate([srgb_to_linear(RGB), np.ones((len(RGB), 1))], 1).astype(np.float32)
    nodes, meshes = [], []
    index = {n: i for i, (n, _, _) in enumerate(model.skel)}
    for name, parent, pivot in model.skel:
        t = pivot - (skel_pivot(model.skel, parent) if parent else 0)
        node = {"name": name, "translation": (t * vox).tolist()}
        if name in quads and len(quads[name][0]):
            q, n, k = quads[name]
            pos = (q.reshape(-1, 3) * vox).astype(np.float32)
            nor = np.repeat(n, 4, 0).astype(np.float32)
            colr = np.repeat(lin[k], 4, 0)
            base = np.arange(len(q)) * 4
            idx = np.stack([base, base + 1, base + 2, base, base + 2, base + 3], 1).reshape(-1).astype(np.uint32)
            prim = {
                "attributes": {
                    "POSITION": add(pos, 5126, "VEC3", 34962, True),
                    "NORMAL": add(nor, 5126, "VEC3", 34962),
                    "COLOR_0": add(colr, 5126, "VEC4", 34962),
                },
                "indices": add(idx, 5125, "SCALAR", 34963),
                "material": 0,
            }
            meshes.append({"name": name, "primitives": [prim]})
            node["mesh"] = len(meshes) - 1
        kids = [index[c] for c, p, _ in model.skel if p == name]
        if kids:
            node["children"] = kids
        nodes.append(node)

    anims = []
    for aname, (fn, dur) in ANIMS.items():
        n = int(round(dur * fps))
        times = np.arange(n + 1, dtype=np.float32) / fps
        times[-1] = dur
        tin = add(times, 5126, "SCALAR", minmax=True)
        samplers, channels = [], []
        poses = [fn(float(t) % dur if t < dur else 0.0, model.L) for t in times]
        for name, parent, pivot in model.skel:
            if name == "root":
                base = pivot
                tr = np.array([base + np.array(p.get("root", (0, 0, 0))) for p in poses], np.float32) * vox
                samplers.append({"input": tin, "output": add(tr.astype(np.float32), 5126, "VEC3"), "interpolation": "LINEAR"})
                channels.append({"sampler": len(samplers) - 1, "target": {"node": index[name], "path": "translation"}})
                continue
            if not any(name in p for p in poses):
                continue
            qs = np.array([quat(*p.get(name, (0, 0, 0))) for p in poses], np.float32)
            for i in range(1, len(qs)):  # keep neighbors on the same hemisphere
                if np.dot(qs[i], qs[i - 1]) < 0:
                    qs[i] = -qs[i]
            samplers.append({"input": tin, "output": add(qs, 5126, "VEC4"), "interpolation": "LINEAR"})
            channels.append({"sampler": len(samplers) - 1, "target": {"node": index[name], "path": "rotation"}})
        anims.append({"name": aname, "samplers": samplers, "channels": channels})

    gltf = {
        "asset": {"version": "2.0", "generator": "voxel-dog tools/voxdog.py"},
        "scene": 0,
        "scenes": [{"name": model.name, "nodes": [0]}],
        "nodes": nodes,
        "meshes": meshes,
        "materials": [{"name": "voxel", "pbrMetallicRoughness": {"baseColorFactor": [1, 1, 1, 1], "metallicFactor": 0, "roughnessFactor": 0.85}}],
        "accessors": accs,
        "bufferViews": views,
        "buffers": [{"byteLength": len(blob)}],
        "animations": anims,
    }
    js = json.dumps(gltf, separators=(",", ":")).encode()
    js += b" " * (-len(js) % 4)
    blob += b"\0" * (-len(blob) % 4)
    total = 12 + 8 + len(js) + 8 + len(blob)
    with open(path, "wb") as f:
        f.write(struct.pack("<4sII", b"glTF", 2, total))
        f.write(struct.pack("<I4s", len(js), b"JSON") + js)
        f.write(struct.pack("<I4s", len(blob), b"BIN\0") + bytes(blob))
    nq = sum(len(v[0]) for v in quads.values())
    return {"voxel_m": vox, "quads": nq, "triangles": 2 * nq, "bytes": total}


# ------------------------------------------------------------------ .vox


def write_vox(model, path):
    """MagicaVoxel .vox (static, rest pose). MagicaVoxel is z-up: (x, z, y) here."""
    idx = np.argwhere(model.col >= 0)
    sx, sy, sz = model.col.shape
    xyzi = b"".join(struct.pack("<BBBB", x, z, y, model.col[x, y, z] + 1) for x, y, z in idx)
    size = struct.pack("<III", sx, sz, sy)
    pal = bytearray()
    for i in range(256):
        c = RGB[i] if i < len(RGB) else (0, 0, 0)
        pal += struct.pack("<BBBB", int(c[0]), int(c[1]), int(c[2]), 255)

    def chunk(tag, content, children=b""):
        return tag + struct.pack("<II", len(content), len(children)) + content + children

    body = chunk(b"SIZE", size) + chunk(b"XYZI", struct.pack("<I", len(idx)) + xyzi) + chunk(b"RGBA", bytes(pal))
    with open(path, "wb") as f:
        f.write(b"VOX " + struct.pack("<I", 150) + chunk(b"MAIN", b"", body))


# ------------------------------------------------------------------ preview renderer


LIGHT = np.array([-0.45, 0.8, 0.55])
LIGHT /= np.linalg.norm(LIGHT)


def render(model, pose, yaw=30.0, pitch=12.0, scale=6.0, size=(420, 420), bg=(36, 38, 46), faces=None, samples=None, rgb=None):
    """Orthographic picture of the model posed: each exposed face drawn as a grid of points
    through a depth buffer, lit by one sun and a sky. Returns an (H, W, 3) uint8 array."""
    faces = faces or model.faces()
    samples = samples or int(math.ceil(scale * 1.5)) + 1
    tf = world_transforms(model.skel, pose)
    cy, sy = math.cos(math.radians(yaw)), math.sin(math.radians(yaw))
    cp, sp = math.cos(math.radians(pitch)), math.sin(math.radians(pitch))
    Ry = np.array([[cy, 0, sy], [0, 1, 0], [-sy, 0, cy]])
    Rx = np.array([[1, 0, 0], [0, cp, -sp], [0, sp, cp]])
    V = Rx @ Ry  # world -> view (looking along -Z of view space)
    g = (np.arange(samples) + 0.5) / samples - 0.5
    gu, gv = np.meshgrid(g, g)
    gu, gv = gu.ravel(), gv.ravel()
    P, N, C = [], [], []
    for name, (c, n, k) in faces.items():
        R, t = tf[name]
        # Two tangents per face.
        ax = np.abs(n).argmax(1)
        tu = np.zeros_like(n)
        tv = np.zeros_like(n)
        tu[np.arange(len(n)), (ax + 1) % 3] = 1
        tv[np.arange(len(n)), (ax + 2) % 3] = 1
        pts = c[:, None, :] + gu[None, :, None] * tu[:, None, :] + gv[None, :, None] * tv[:, None, :]
        pts = pts.reshape(-1, 3) @ R.T + t
        P.append(pts)
        N.append(np.repeat(n @ R.T, samples * samples, 0))
        C.append(np.repeat(k, samples * samples))
    P, N, C = np.concatenate(P), np.concatenate(N), np.concatenate(C)
    top = np.nonzero((model.col >= 0).any(axis=(0, 2)))[0].max() + 1 + model.origin[1]
    center = np.array([0, top * 0.45, 0])
    q = (P - center) @ V.T
    W, H = size
    px = np.round(q[:, 0] * scale + W / 2).astype(int)
    py = np.round(-q[:, 1] * scale + H * 0.52).astype(int)
    depth = -q[:, 2]
    ok = (px >= 0) & (px < W) & (py >= 0) & (py < H)
    vn = N @ V.T
    ok &= vn[:, 2] > -0.05  # back faces away
    px, py, depth, C, N = px[ok], py[ok], depth[ok], C[ok], N[ok]
    pix = py * W + px
    order = np.lexsort((depth, pix))
    pix_s = pix[order]
    first = np.ones(len(pix_s), bool)
    first[1:] = pix_s[1:] != pix_s[:-1]
    sel = order[first]
    img = np.empty((H * W, 3))
    img[:] = bg
    lam = np.clip(N[sel] @ LIGHT, 0, 1)
    sky = 0.5 + 0.5 * N[sel][:, 1]
    shade = 0.42 + 0.5 * lam + 0.16 * sky
    pal = RGB if rgb is None else rgb
    img[pix[sel]] = np.clip(pal[C[sel]] * shade[:, None], 0, 255)
    # A thin dark outline where depth jumps, to read the cubes.
    out = img.reshape(H, W, 3)
    dz = np.full(H * W, np.inf)
    dz[pix[sel]] = depth[sel]
    dz = dz.reshape(H, W)
    edge = np.zeros((H, W), bool)
    for dy, dx in ((0, 1), (1, 0)):
        a, b = dz[: H - dy, : W - dx], dz[dy:, dx:]
        with np.errstate(invalid="ignore"):
            e = np.abs(a - b) > 1.5
        edge[: H - dy, : W - dx] |= e & (a > b)
        edge[dy:, dx:] |= e & (b > a)
    out[edge & np.isfinite(dz)] *= 0.55
    # The ground, as a faint line where y = 0 meets the middle of the picture.
    gy = int(round(-(np.array([0, 0, 0]) - center) @ V.T @ np.array([0, 1, 0]) * scale + H * 0.52))
    if 0 <= gy < H:
        row = out[gy]
        row[~np.isfinite(dz[gy])] = (70, 74, 84)
    return out.astype(np.uint8)
