"""A small isometric block renderer for art sheets: boxes (full cubes or Minecraft-style model boxes in
sixteenths) with a top and two side textures, drawn back to front with simple face shading. A box face
shows only the part of its texture that matches the box's position, as in the game.

    import iso
    s = iso.Scene()
    s.box((0, 0, 0), top=img, side=img)                         # a full cube at x 0, y 0, z 0
    s.box((1, 0, 0), top=t, side=sd, box=(2, 0, 2, 14, 16, 14)) # a model box, in sixteenths
    s.render(scale=2)                                           # an RGBA PIL image
"""
import numpy as np
from PIL import Image

T = 32                       # texture pixels per block
EX, EY, EZ = np.array([1.0, 0.5]), np.array([-1.0, 0.5]), np.array([0.0, -1.0])  # screen px per texture px
SHADE = {"top": 1.0, "left": 0.82, "right": 0.66}


def _face(canvas, tex, origin, a, b, shade):
    """Paint `tex` (RGBA array) onto canvas so texture (u, v) lands at origin + u*a + v*b."""
    M = np.array([a, b]).T
    inv = np.linalg.inv(M)
    h, w = canvas.shape[:2]
    ys, xs = np.mgrid[0:h, 0:w]
    rel = np.stack([xs + 0.5 - origin[0], ys + 0.5 - origin[1]], -1) @ inv.T
    u, v = rel[..., 0], rel[..., 1]
    th, tw = tex.shape[:2]
    on = (u >= 0) & (u < tw) & (v >= 0) & (v < th)
    ui, vi = np.clip(u.astype(int), 0, tw - 1), np.clip(v.astype(int), 0, th - 1)
    px = tex[vi, ui].astype(float)
    on &= px[..., 3] > 0
    canvas[on, :3] = px[on, :3] * shade
    canvas[on, 3] = 255


def _arr(img):
    return np.array(img.convert("RGBA")) if not isinstance(img, np.ndarray) else img


class Scene:
    def __init__(self):
        self.items = []

    def box(self, pos, top, side, box=(0, 0, 0, 16, 16, 16), bottom=None, left=None, right=None):
        """pos = block (x, y, z); box = (x0, y0, z0, x1, y1, z1) in sixteenths inside it."""
        self.items.append((pos, box, _arr(top), _arr(left if left is not None else side),
                           _arr(right if right is not None else side)))

    def render(self, scale=2, pad=8):
        blocks = [p for p, *_ in self.items]
        xs = [(x - z) * T for x, y, z in blocks]
        ys = [(x + z) * T / 2 - y * T for x, y, z in blocks]
        W = int(max(xs) - min(xs) + 2 * T + 2 * pad)
        H = int(max(ys) - min(ys) + 2 * T + 2 * pad)
        ox, oy = -min(xs) + T + pad, -min(ys) + pad
        canvas = np.zeros((H, W, 4))
        # back to front: far (small x + z) first, then low to high
        for (x, y, z), (x0, y0, z0, x1, y1, z1), top, left, right in sorted(
                self.items, key=lambda it: (it[0][0] + it[0][2], it[0][1])):
            k = T / 16
            # the box's corners in texture px from the block's top-back corner
            bx0, bx1, bz0, bz1 = x0 * k, x1 * k, z0 * k, z1 * k
            by1 = y1 * k
            base = np.array([(x - z) * T + ox, (x + z) * T / 2 - y * T + oy + T])  # screen of block corner (x, y, z) at y = 0
            # top: u along x, v along z, at height y1
            o_top = base + bx0 * EX + bz0 * EY + by1 * EZ
            _face(canvas, top[int(bz0):int(bz1), int(bx0):int(bx1)], o_top, EX, EY, SHADE["top"])
            # left face (the +z side, facing down-left): u along x, v down from y1
            o_l = base + bx0 * EX + bz1 * EY + by1 * EZ
            vt0, vt1 = int(T - y1 * k), int(T - y0 * k)
            _face(canvas, left[vt0:vt1, int(bx0):int(bx1)], o_l, EX, -EZ, SHADE["left"])
            # right face (the +x side, facing down-right): u along -z... reading left to right means z from z1 to z0
            o_r = base + bx1 * EX + bz1 * EY + by1 * EZ
            r = right[vt0:vt1, int(T - bz1):int(T - bz0)]
            _face(canvas, r, o_r, -EY, -EZ, SHADE["right"])
        img = Image.fromarray(canvas.clip(0, 255).astype(np.uint8))
        return img.resize((W * scale, H * scale), Image.NEAREST)
