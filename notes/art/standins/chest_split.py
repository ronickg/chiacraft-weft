"""Our chest split into a body and a lid (art stand-ins, 11-26), for the engine's block animations
(`animated_part`, upstream be14c5a8d; ours in claude/weft-te-shadows-1126, 484ad5f93). The art is
the approved chest (`chest_front`, `chest_side`, `chest_top`, 32 a block), cut where it already
splits: the lid is the top 4 px of the 14 px box (texture rows 4-12, the iron strap its lower rim),
the body the 10 px under it. The gold latch is painted out of the front and becomes its own small
box on the lid, hanging a pixel over the body. The only new picture is `chest_inside` (the boards
inside, generated with the block master prompt).

Writes, under OUT (default the base mod's assets):
  textures/block/chest_lid.png        the lid's atlas, 64 x 64 (32 texels a block)
  textures/block/chest_body_front.png chest_front without its latch (the body's front wall)
  models/block/chest_body.json        the open box: floor and four walls, 10 px tall
  models/block/chest_lid.geo.json     the lid and latch, hinged at the back top edge
  animations/block/chest_lid.animation.json   "open": 0 to -90 degrees in 0.28 s
  models/item/chest.json              the whole chest in the hand and slot, latch and all
and with --preview PNG, a drawing of it shut and open.

Usage: python3 chest_split.py [--out DIR] [--inside chest_inside.png] [--preview out.png]"""
import json
import os
import sys

import numpy as np
from PIL import Image

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
ASSETS = os.path.join(REPO, "mods/base/assets")


def arg(name, default):
    return sys.argv[sys.argv.index(name) + 1] if name in sys.argv else default


OUT = arg("--out", ASSETS)
TEX = os.path.join(ASSETS, "textures/block")
LATCH = (7, 15, 13, 19)  # texel rows 7-14, columns 13-18 of chest_front: the gold latch and its outline


def load(name):
    return np.asarray(Image.open(os.path.join(TEX, name + ".png")).convert("RGBA")).copy()


def textures(inside_path):
    front, side, top = load("chest_front"), load("chest_side"), load("chest_top")
    inside = np.asarray(Image.open(inside_path).convert("RGBA")).copy()
    y0, y1, x0, x1 = LATCH
    latch = front[y0:y1, x0:x1].copy()
    body_front = front.copy()
    body_front[y0:y1, x0:x1] = front[y0:y1, x0 - 6:x1 - 6]  # the same rows, from beside it
    lid = np.zeros((64, 64, 4), np.uint8)
    lid[0:28, 0:28] = top[2:30, 2:30]           # up
    lid[0:28, 28:56] = inside[2:30, 2:30]       # down: the lid's underside
    lid[28:36, 0:28] = body_front[4:12, 2:30]   # north (front)
    lid[36:44, 0:28] = side[4:12, 2:30]         # south
    lid[28:36, 28:56] = side[4:12, 2:30]        # east
    lid[36:44, 28:56] = side[4:12, 2:30]        # west
    lid[0:8, 56:62] = latch                     # latch front
    lid[8:16, 56:58] = latch[:, :2]             # latch sides: its own edge columns
    lid[8:16, 58:60] = latch[:, -2:]
    lid[16:18, 56:62] = latch[:2]               # latch top and bottom
    lid[18:20, 56:62] = latch[-2:]
    lid[20:28, 56:62] = latch                   # latch back (hidden)
    return lid, body_front, inside, front, side, top


def face(u, v, w, h):
    return {"uv": [u, v], "uv_size": [w, h]}


def lid_geo():
    ly0, ly1, lx0, lx1 = LATCH
    # the latch in model px: columns 13-19 of 32 are x 6.5-9.5; rows 7-15 are y 12.5-8.5
    lx, ly = lx0 / 2 - 8, 16 - ly1 / 2
    return {
        "format_version": "1.12.0",
        "minecraft:geometry": [{
            "description": {"identifier": "geometry.chest_lid", "texture_width": 64, "texture_height": 64},
            "bones": [{
                "name": "lid",
                "pivot": [0, 10, 7],
                "cubes": [
                    {"origin": [-7, 10, -7], "size": [14, 4, 14], "uv": {
                        "up": face(0, 0, 28, 28), "down": face(28, 0, 28, 28),
                        "north": face(0, 28, 28, 8), "south": face(0, 36, 28, 8),
                        "east": face(28, 28, 28, 8), "west": face(28, 36, 28, 8)}},
                    {"origin": [lx, ly, -8], "size": [(lx1 - lx0) / 2, (ly1 - ly0) / 2, 1], "uv": {
                        "north": face(56, 0, 6, 8), "south": face(56, 20, 6, 8),
                        "east": face(56, 8, 2, 8), "west": face(58, 8, 2, 8),
                        "up": face(56, 16, 6, 2), "down": face(56, 18, 6, 2)}},
                ],
            }],
        }],
    }


ANIM = {
    "format_version": "1.8.0",
    "animations": {"animation.chest_lid.open": {
        "animation_length": 0.28,
        "bones": {"lid": {"rotation": {"0.0": [0, 0, 0], "0.05": [-8, 0, 0], "0.14": [-45, 0, 0], "0.23": [-82, 0, 0], "0.28": [-90, 0, 0]}}},
    }},
}

BODY = {
    "textures": {"side": "base:block/chest_side", "front": "base:block/chest_body_front", "rim": "base:block/chest_top",
                 "inside": "base:block/chest_inside"},
    "elements": [
        {"from": [1, 0, 1], "to": [15, 2, 15], "faces": {
            "down": {"texture": "#rim"}, "up": {"texture": "#inside", "uv": [2, 2, 14, 14]},
            "east": {"texture": "#side"}, "west": {"texture": "#side"}, "south": {"texture": "#side"}, "north": {"texture": "#front"}}},
        {"from": [1, 2, 1], "to": [15, 10, 2], "faces": {
            "up": {"texture": "#rim"}, "north": {"texture": "#front"}, "south": {"texture": "#inside", "uv": [2, 6, 14, 14]},
            "east": {"texture": "#side"}, "west": {"texture": "#side"}}},
        {"from": [1, 2, 14], "to": [15, 10, 15], "faces": {
            "up": {"texture": "#rim"}, "south": {"texture": "#side"}, "north": {"texture": "#inside", "uv": [2, 6, 14, 14]},
            "east": {"texture": "#side"}, "west": {"texture": "#side"}}},
        {"from": [1, 2, 2], "to": [2, 10, 14], "faces": {
            "up": {"texture": "#rim"}, "west": {"texture": "#side"}, "east": {"texture": "#inside"}}},
        {"from": [14, 2, 2], "to": [15, 10, 14], "faces": {
            "up": {"texture": "#rim"}, "east": {"texture": "#side"}, "west": {"texture": "#inside"}}},
    ],
}

ITEM = {
    "textures": {"top": "base:block/chest_top", "side": "base:block/chest_side", "front": "base:block/chest_front"},
    "elements": [{"from": [1, 0, 1], "to": [15, 14, 15], "faces": {
        "up": {"texture": "#top"}, "down": {"texture": "#top"}, "north": {"texture": "#front"},
        "south": {"texture": "#side"}, "east": {"texture": "#side"}, "west": {"texture": "#side"}}}],
}


# --- a small drawing of it (no engine): boxes of textured faces, painter's order, 3/4 view from the front left.
def project(p):
    x, y, z = p
    # camera looking from north-west-above toward south-east: x to the right-down, z to the left-down
    return np.array([(x - z) * 0.87, -(y * 1.0) - (x + z) * 0.5]), (x + z) - y * 0.6


def draw(faces, scale=14, size=(440, 520)):
    """Faces drawn with a depth buffer (nearer pixel wins), so boxes passing through each other read right."""
    W, H = size
    out = np.zeros((H, W, 4), np.uint8)
    out[...] = (53, 32, 21, 255)
    zbuf = np.full((H, W), np.inf)
    shade = {"up": 1.0, "north": 0.86, "west": 0.7, "east": 0.6, "south": 0.6, "down": 0.5}
    o = np.array([W / 2, H * 0.97])
    for corners, tex, name in faces:
        pts = [project(c) for c in corners]
        (p0, d0), (p1, d1), _, (p3, d3) = pts
        P0, P1, P3 = p0 * scale + o, p1 * scale + o, p3 * scale + o
        h, w = tex.shape[:2]
        A = np.array([P1 - P0, P3 - P0]).T
        if abs(np.linalg.det(A)) < 1e-6:
            continue
        Ai = np.linalg.inv(A)
        a, b, c = Ai[0, 0] * w, Ai[0, 1] * w, -(Ai[0] @ P0) * w
        d, e, f = Ai[1, 0] * h, Ai[1, 1] * h, -(Ai[1] @ P0) * h
        arr = tex.astype(float)
        arr[..., :3] *= shade[name]
        t = Image.fromarray(arr.clip(0, 255).astype(np.uint8), "RGBA")
        warped = np.asarray(t.transform(size, Image.AFFINE, (a, b, c, d, e, f), Image.NEAREST))
        yy, xx = np.mgrid[0:H, 0:W] + 0.5
        u = (a * xx + b * yy + c) / w
        v = (d * xx + e * yy + f) / h
        depth = d0 + u * (d1 - d0) + v * (d3 - d0)
        m = (warped[..., 3] > 0) & (depth < zbuf - 1e-4)
        out[m] = warped[m]
        zbuf[m] = depth[m]
    return Image.fromarray(out)


def box_faces(lo, hi, tex, rot=None):
    """Six faces of a box (lo, hi in px), each (corners p0 top-left, p1 top-right, p2, p3 bottom-left), texture."""
    (x0, y0, z0), (x1, y1, z1) = lo, hi
    F = {
        "up": [(x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1)],
        "down": [(x0, y0, z1), (x1, y0, z1), (x1, y0, z0), (x0, y0, z0)],
        "north": [(x1, y1, z0), (x0, y1, z0), (x0, y0, z0), (x1, y0, z0)],
        "south": [(x0, y1, z1), (x1, y1, z1), (x1, y0, z1), (x0, y0, z1)],
        "west": [(x0, y1, z0), (x0, y1, z1), (x0, y0, z1), (x0, y0, z0)],
        "east": [(x1, y1, z1), (x1, y1, z0), (x1, y0, z0), (x1, y0, z1)],
    }
    out = []
    for n, cs in F.items():
        if n not in tex:
            continue
        cs = [rot(np.array(c, float)) if rot else np.array(c, float) for c in cs]
        out.append((cs, tex[n], n))
    return out


def preview(path, lid, body_front, inside, front, side, top):
    def uvs(img, u, v, w, h):
        return img[v:v + h, u:u + w]

    def px(img, x0, y0, x1, y1):  # a block tile's area by model px (y up)
        return img[32 - 2 * y1:32 - 2 * y0, 2 * x0:2 * x1]

    def body():
        f = []
        # floor and walls: outside faces from the tiles by position, inside from chest_inside
        f += box_faces((1, 0, 1), (15, 10, 15), {"north": px(body_front, 1, 0, 15, 10), "west": px(side, 1, 0, 15, 10),
                                                  "east": px(side, 1, 0, 15, 10), "south": px(side, 1, 0, 15, 10)})
        return f

    def lid_faces(angle):
        import math
        a = math.radians(angle)
        piv = np.array([8.0, 10.0, 15.0])

        def rot(p):
            q = p - piv
            y = q[1] * math.cos(a) - q[2] * math.sin(a)
            z = q[1] * math.sin(a) + q[2] * math.cos(a)
            return np.array([q[0], y, z]) + piv
        f = box_faces((1, 10, 1), (15, 14, 15), {"up": uvs(lid, 0, 0, 28, 28), "down": uvs(lid, 28, 0, 28, 28),
                                                  "north": uvs(lid, 0, 28, 28, 8), "west": uvs(lid, 28, 36, 28, 8),
                                                  "east": uvs(lid, 28, 28, 28, 8), "south": uvs(lid, 0, 36, 28, 8)}, rot)
        f += box_faces((6.5, 8.5, 0), (9.5, 12.5, 1), {"north": uvs(lid, 56, 0, 6, 8), "west": uvs(lid, 58, 8, 2, 8),
                                                       "east": uvs(lid, 56, 8, 2, 8), "up": uvs(lid, 56, 16, 6, 2)}, rot)
        return f

    inner = box_faces((2, 2, 2), (14, 10, 14), {"up": inside[4:28, 4:28]})  # stands for the inside: floor seen through the open top
    shut = draw(body() + lid_faces(0))
    opened = draw([(c, t, n) for c, t, n in inner if False] + body() + [(
        [np.array(p) for p in ((2, 2.01, 2), (14, 2.01, 2), (14, 2.01, 14), (2, 2.01, 14))], inside[4:28, 4:28], "up")] +
        [(
            [np.array(p) for p in ((2, 10, 14), (14, 10, 14), (14, 2, 14), (2, 2, 14))], inside[4:20, 4:28], "north")] +
        lid_faces(80))
    sheet = Image.new("RGBA", (shut.width * 2 + 10, shut.height), (53, 32, 21, 255))
    sheet.alpha_composite(shut, (0, 0))
    sheet.alpha_composite(opened, (shut.width + 10, 0))
    sheet.save(path)


def main():
    inside = arg("--inside", os.path.join(TEX, "chest_inside.png"))
    lid, body_front, ins, front, side, top = textures(inside)
    if "--preview-only" not in sys.argv:
        for sub in ("textures/block", "models/block", "models/item", "animations/block"):
            os.makedirs(os.path.join(OUT, sub), exist_ok=True)
        Image.fromarray(lid).save(os.path.join(OUT, "textures/block/chest_lid.png"))
        Image.fromarray(body_front).save(os.path.join(OUT, "textures/block/chest_body_front.png"))
        json.dump(BODY, open(os.path.join(OUT, "models/block/chest_body.json"), "w"), indent=2)
        json.dump(lid_geo(), open(os.path.join(OUT, "models/block/chest_lid.geo.json"), "w"), indent=2)
        json.dump(ANIM, open(os.path.join(OUT, "animations/block/chest_lid.animation.json"), "w"), indent=2)
        json.dump(ITEM, open(os.path.join(OUT, "models/item/chest.json"), "w"), indent=2)
    if "--preview" in sys.argv:
        preview(arg("--preview", "chest.png"), lid, body_front, ins, front, side, top)


if __name__ == "__main__":
    main()
