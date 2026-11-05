"""Small Java block models in code, for 3D plants (Ronald, 11-05: "let's try model like grass or like sugarcane
crops in 3D"): boxes in model units (16 a block; at our 32 texels a block, a unit is 2 texels), each painted into
one square picture named like the model (a face with no `texture` shows the picture named like its CTM tile,
docs/ctm.md "Models as tiles"). Faces are packed into the picture with a 1 texel gutter of their own colour."""
import json
import math

import numpy as np
from PIL import Image

TX = 2  # texels a model unit (32 a block)
FACES = ["north", "south", "east", "west", "up", "down"]


class Box:
    def __init__(self, lo, hi, paint, rot=None, faces=FACES, tag=""):
        self.lo, self.hi = [float(v) for v in lo], [float(v) for v in hi]
        self.paint = paint        # paint(box, face, u, v, w, h) -> rgba (u, v texels from the face's top left)
        self.rot = rot            # (origin, axis, angle)
        self.faces = [f for f in faces]
        self.tag = tag

    def size(self, face):
        dx, dy, dz = (self.hi[i] - self.lo[i] for i in range(3))
        w, h = {"north": (dx, dy), "south": (dx, dy), "east": (dz, dy), "west": (dz, dy), "up": (dx, dz), "down": (dx, dz)}[face]
        return max(1, round(w * TX)), max(1, round(h * TX))


def pack(boxes):
    """Shelf-pack every face (+1 texel gutter all round) into the smallest square of 16, 32, 64 or 128."""
    rects = [(b, f, *b.size(f)) for b in boxes for f in b.faces]
    order = sorted(range(len(rects)), key=lambda i: -rects[i][3])
    for side in (16, 32, 64, 128):
        x = y = row = 0
        place, ok = {}, True
        for i in order:
            w, h = rects[i][2] + 2, rects[i][3] + 2
            if x + w > side:
                x, y, row = 0, y + row, 0
            if y + h > side or w > side:
                ok = False
                break
            place[i] = (x + 1, y + 1)
            x, row = x + w, max(row, h)
        if ok:
            return side, rects, place
    raise ValueError("faces don't fit in 128 x 128")


def build(boxes, ambient=None):
    side, rects, place = pack(boxes)
    img = np.zeros((side, side, 4), np.uint8)
    elements = []
    faces_of = {}
    for i, (b, f, w, h) in enumerate(rects):
        x0, y0 = place[i]
        for v in range(h):
            for u in range(w):
                img[y0 + v, x0 + u] = b.paint(b, f, u, v, w, h)
        # gutter: the face's edge texels copied outward
        img[y0 - 1, x0:x0 + w] = img[y0, x0:x0 + w]
        img[y0 + h, x0:x0 + w] = img[y0 + h - 1, x0:x0 + w]
        img[y0 - 1:y0 + h + 1, x0 - 1] = img[y0 - 1:y0 + h + 1, x0]
        img[y0 - 1:y0 + h + 1, x0 + w] = img[y0 - 1:y0 + h + 1, x0 + w - 1]
        k = 16 / side
        faces_of.setdefault(id(b), {})[f] = {"uv": [round(x0 * k, 4), round(y0 * k, 4), round((x0 + w) * k, 4), round((y0 + h) * k, 4)]}
    for b in boxes:
        e = {"from": [round(v, 3) for v in b.lo], "to": [round(v, 3) for v in b.hi], "faces": faces_of[id(b)]}
        if b.rot:
            o, ax, ang = b.rot
            e["rotation"] = {"origin": [round(v, 3) for v in o], "axis": ax, "angle": round(ang, 2)}
        elements.append(e)
    return {"elements": elements}, Image.fromarray(img, "RGBA")


def quads(model):
    return sum(len(e["faces"]) for e in model["elements"])


def write(model, img, json_path, png_path, note=""):
    m = dict(model)
    if note:
        m = {"__comment": note, **m}
    with open(json_path, "w") as f:
        json.dump(m, f, separators=(",", ":"))
    img.save(png_path)


# ---------- painting helpers ----------

def ramp(cols, t):
    """Colour along a ramp of hex colours, t in 0..1."""
    cs = [tuple(int(c[i:i + 2], 16) for i in (1, 3, 5)) for c in cols]
    t = min(max(t, 0.0), 1.0) * (len(cs) - 1)
    i = min(int(t), len(cs) - 2)
    f = t - i
    return tuple(int(round(cs[i][k] * (1 - f) + cs[i + 1][k] * f)) for k in range(3))


def along(b, face, v, h):
    """0 at the box's bottom, 1 at its top, for a row v of a side face (up face: 1, down: 0)."""
    if face == "up":
        return 1.0
    if face == "down":
        return 0.0
    return 1.0 - (v + 0.5) / h


def rot_point(p, rot):
    if not rot:
        return p
    o, ax, ang = rot
    a = math.radians(ang)
    x, y, z = (p[i] - o[i] for i in range(3))
    c, s = math.cos(a), math.sin(a)
    if ax == "x":
        y, z = y * c - z * s, y * s + z * c
    elif ax == "y":
        x, z = x * c + z * s, -x * s + z * c
    else:
        x, y = x * c - y * s, x * s + y * c
    return (x + o[0], y + o[1], z + o[2])
