"""A small orthographic renderer for review sheets (not the game): blocks as cubes, flat plants as two crossed
pictures, and Java block models' boxes (turned as the file says), each face textured texel for texel, drawn far
to near, lit from above and the front left, tinted like the biome where asked."""
import math

import numpy as np
from PIL import Image

from model import rot_point

YAW, PITCH = math.radians(-35), math.radians(30)
L = np.array([-0.45, 0.8, 0.4])
L = L / np.linalg.norm(L)


def view(p):
    x, y, z = p
    cx, sx = math.cos(YAW), math.sin(YAW)
    x, z = x * cx - z * sx, x * sx + z * cx
    cp, sp = math.cos(PITCH), math.sin(PITCH)
    y, z = y * cp - z * sp, y * sp + z * cp
    return x, -y, z  # screen x, screen y (down), depth (toward the camera is +)


CAM = np.array(view((0, 0, 1)))  # unused helper


def to_cam(n):
    """The camera's look direction in world space: faces whose normal points along -it are seen."""
    x, y, z = view(n)
    return z


class Scene:
    def __init__(self, px=96):
        self.px = px
        self.faces = []  # (depth, corners3d, tex rgba array, (u0,v0,u1,v1) in texels, tint, two_sided)

    def face(self, corners, tex, uvpx, tint=None, two_sided=False, shade=True):
        tl, tr, bl = corners
        br = tuple(tr[i] + bl[i] - tl[i] for i in range(3))
        cen = [(tl[i] + br[i]) / 2 for i in range(3)]
        n = np.cross(np.subtract(bl, tl), np.subtract(tr, tl))
        nn = np.linalg.norm(n)
        n = n / nn if nn else np.array([0, 1, 0])
        if not two_sided and view_dot(n) <= 0:
            return
        light = 0.62 + 0.38 * max(0.0, float(abs(n @ L) if two_sided else n @ L)) if shade else 1.0
        self.faces.append((view(cen)[2], (tl, tr, bl), tex, uvpx, tint, light))

    def cube(self, at, top, side, tint_top=None, bottom=None, tint_side=None):
        x, y, z = at
        lo, hi = (x, y, z), (x + 1, y + 1, z + 1)
        for f, (c, t, ti) in box_faces(lo, hi, {"up": top, "down": bottom or side, "north": side, "south": side, "east": side, "west": side},
                                       {"up": tint_top, "north": tint_side, "south": tint_side, "east": tint_side, "west": tint_side}).items():
            a = np.asarray(t.convert("RGBA"))
            self.face(c, a, (0, 0, a.shape[1], a.shape[0]), ti)

    def cross(self, at, tex, tint=None):
        x, y, z = at
        a = np.asarray(tex.convert("RGBA"))
        e = 0.0625
        for (x0, z0, x1, z1) in ((e, e, 1 - e, 1 - e), (1 - e, e, e, 1 - e)):
            self.face(((x + x0, y + 1, z + z0), (x + x1, y + 1, z + z1), (x + x0, y, z + z0)), a, (0, 0, a.shape[1], a.shape[0]), tint, two_sided=True)

    def model(self, at, m, tex, tint=None):
        a = np.asarray(tex.convert("RGBA"))
        k = a.shape[0] / 16
        x, y, z = at
        for e in m["elements"]:
            lo = [v / 16 for v in e["from"]]
            hi = [v / 16 for v in e["to"]]
            rot = None
            if "rotation" in e:
                r = e["rotation"]
                rot = ([v / 16 for v in r["origin"]], r["axis"], r["angle"])
            fs = box_faces(lo, hi, {f: None for f in e["faces"]}, {})
            for f, (c, _, _) in fs.items():
                c = tuple(tuple(np.add(rot_point(p, rot), (x, y, z))) for p in c)
                u0, v0, u1, v1 = e["faces"][f]["uv"]
                self.face(c, a, (u0 * k, v0 * k, u1 * k, v1 * k), tint)

    def render(self, size, origin=(0.5, 0.5), bg=(159, 196, 222, 255)):
        W, H = size
        out = np.zeros((H, W, 4), float)
        out[...] = bg
        ox, oy = W * origin[0], H * origin[1]
        for depth, (tl, tr, bl), tex, (u0, v0, u1, v1), tint, light in sorted(self.faces, key=lambda f: f[0]):
            P = [np.array(view(p)[:2]) * self.px + (ox, oy) for p in (tl, tr, bl)]
            A = np.column_stack([P[1] - P[0], P[2] - P[0]])
            if abs(np.linalg.det(A)) < 1e-6:
                continue
            Ai = np.linalg.inv(A)
            br = P[1] + P[2] - P[0]
            xs = [p[0] for p in P] + [br[0]]
            ys = [p[1] for p in P] + [br[1]]
            x0, x1 = max(0, int(min(xs))), min(W, int(math.ceil(max(xs))) + 1)
            y0, y1 = max(0, int(min(ys))), min(H, int(math.ceil(max(ys))) + 1)
            if x0 >= x1 or y0 >= y1:
                continue
            gx, gy = np.meshgrid(np.arange(x0, x1) + 0.5, np.arange(y0, y1) + 0.5)
            d = np.stack([gx - P[0][0], gy - P[0][1]], -1) @ Ai.T
            s, t = d[..., 0], d[..., 1]
            inside = (s >= 0) & (s < 1) & (t >= 0) & (t < 1)
            if not inside.any():
                continue
            tu = np.clip((u0 + s * (u1 - u0)).astype(int), 0, tex.shape[1] - 1)
            tv = np.clip((v0 + t * (v1 - v0)).astype(int), 0, tex.shape[0] - 1)
            c = tex[tv, tu].astype(float)
            m = inside & (c[..., 3] > 127)
            rgb = c[..., :3]
            if tint is not None:
                rgb = rgb * (np.array(tint) / 255)
            rgb = rgb * light
            reg = out[y0:y1, x0:x1]
            reg[m, :3] = rgb[m]
            reg[m, 3] = 255
        return Image.fromarray(out.clip(0, 255).astype(np.uint8), "RGBA")


def view_dot(n):
    """> 0 when the face's normal points toward the camera."""
    return view(tuple(n))[2]


def box_faces(lo, hi, tex, tints):
    (x0, y0, z0), (x1, y1, z1) = lo, hi
    C = {
        "north": ((x1, y1, z0), (x0, y1, z0), (x1, y0, z0)),
        "south": ((x0, y1, z1), (x1, y1, z1), (x0, y0, z1)),
        "east": ((x1, y1, z1), (x1, y1, z0), (x1, y0, z1)),
        "west": ((x0, y1, z0), (x0, y1, z1), (x0, y0, z0)),
        "up": ((x0, y1, z0), (x1, y1, z0), (x0, y1, z1)),
        "down": ((x0, y0, z1), (x1, y0, z1), (x0, y0, z0)),
    }
    return {f: (C[f], tex[f], tints.get(f)) for f in tex}
