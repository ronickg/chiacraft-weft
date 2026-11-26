"""Our animals' box models (Bedrock geometry from mods/animals/assets/models, their painted textures)
as faces for the isometric world scene: each cube's six faces, placed by its bone chain at rest,
each a parallelogram with its texture rectangle, so the scene's face drawing can paint them.

Units: Bedrock geometry is in model pixels, 16 a block.
"""
import json
import os

import numpy as np
from PIL import Image

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
ANIMALS = os.path.join(ROOT, "mods", "animals", "assets")


def rot(deg):
    """Bedrock bone rotation (degrees about x, y, z), applied z, then y, then x."""
    x, y, z = np.radians(deg)
    rx = np.array([[1, 0, 0], [0, np.cos(x), -np.sin(x)], [0, np.sin(x), np.cos(x)]])
    ry = np.array([[np.cos(y), 0, np.sin(y)], [0, 1, 0], [-np.sin(y), 0, np.cos(y)]])
    rz = np.array([[np.cos(z), -np.sin(z), 0], [np.sin(z), np.cos(z), 0], [0, 0, 1]])
    return rx @ ry @ rz


def load(name):
    geo = json.load(open(os.path.join(ANIMALS, "models", name + ".geo.json")))["minecraft:geometry"][0]
    tex = Image.open(os.path.join(ANIMALS, "textures", "entity", name, name + ".png")).convert("RGBA")
    return geo, tex


def faces(name, yaw=0.0):
    """[(corner, edge_u, edge_v, normal, texture crop)] in block units, the model standing at the
    origin on y = 0, turned by `yaw` degrees about the vertical."""
    geo, tex = load(name)
    bones = {b["name"]: b for b in geo["bones"]}
    cache = {}

    def world(bname):
        """The bone's transform at rest: (matrix, offset), p -> M p + t."""
        if bname in cache:
            return cache[bname]
        b = bones[bname]
        piv = np.array(b.get("pivot", [0, 0, 0]), float)
        piv[0] = -piv[0]  # Bedrock's x is mirrored against the cubes' left-to-right
        R = rot([-b.get("rotation", [0, 0, 0])[0], b.get("rotation", [0, 0, 0])[1] * -1,
                 b.get("rotation", [0, 0, 0])[2]])
        M, t = R, piv - R @ piv
        if b.get("parent") in bones:
            PM, Pt = world(b["parent"])
            M, t = PM @ M, PM @ t + Pt
        cache[bname] = (M, t)
        return M, t

    Y = rot([0, yaw, 0])
    out = []
    for b in geo["bones"]:
        M, t = world(b["name"])
        for c in b.get("cubes", []):
            o = np.array(c["origin"], float)
            s = np.array(c["size"], float)
            x0, x1 = -(o[0] + s[0]), -o[0]  # mirrored x
            y0, y1 = o[1], o[1] + s[1]
            z0, z1 = o[2], o[2] + s[2]
            # face: (corner at the texture's top-left, u edge, v edge) in model space
            spec = {
                "north": ((x1, y1, z0), (x0 - x1, 0, 0), (0, y0 - y1, 0)),
                "south": ((x0, y1, z1), (x1 - x0, 0, 0), (0, y0 - y1, 0)),
                "east": ((x1, y1, z1), (0, 0, z0 - z1), (0, y0 - y1, 0)),
                "west": ((x0, y1, z0), (0, 0, z1 - z0), (0, y0 - y1, 0)),
                "up": ((x0, y1, z0), (x1 - x0, 0, 0), (0, 0, z1 - z0)),
                "down": ((x0, y0, z1), (x1 - x0, 0, 0), (0, 0, z0 - z1)),
            }
            for f, (corner, eu, ev) in spec.items():
                uv = c["uv"].get(f)
                if not uv:
                    continue
                (u, v), (du, dv) = uv["uv"], uv["uv_size"]
                if du == 0 or dv == 0:
                    continue
                crop = tex.crop((int(min(u, u + du)), int(min(v, v + dv)), int(max(u, u + du)), int(max(v, v + dv))))
                if du < 0:
                    crop = crop.transpose(Image.FLIP_LEFT_RIGHT)
                if dv < 0:
                    crop = crop.transpose(Image.FLIP_TOP_BOTTOM)
                P = Y @ (M @ np.array(corner) + t)
                U = Y @ (M @ np.array(eu))
                V = Y @ (M @ np.array(ev))
                centre = Y @ (M @ np.array([(x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2]) + t)
                n = (P + U / 2 + V / 2) - centre  # outward: from the cube's middle to the face's
                out.append((P / 16, U / 16, V / 16, n, crop))
    return out
