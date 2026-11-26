"""Models without a skeleton (props: nests, eggs, furniture, anything placed or lying about):
laid out, painted and exported the same way as the creatures, so they match them.

    elements = [el("base", (-32, 0, -32), (32, 4, 32)), el("grass", lo, hi, quad="south",
                rot=(0, 45, 0), origin=..., cut="grass")]
    sheet, uv = paint(elements, area, materials, cut=cut_mask)
    spec = entity_spec(elements, uv, sheet, "nest")
    to_bedrock.export(spec, texture, assets, "nest", house.SCALE, "nest/nest", pose_files={"": {}})

A prop is an entity in the game (placed by its item, broken back into it), not a block: block
models in this engine take only coarse texture coordinates (a 1/128th of the texture), and a painted
sheet on one came out a flat color in the game. Terrain blocks stay the base game's 16-pixel
textures.

Elements are in layout units (house.UNITS a block; painted house.TX texels a unit with `tx`), y up, facing +z, centered on the
origin at the ground. `area(e, face, p, uu, vv, h)` returns the material per texel (ids into
`materials`, a list of `house.material(...)` tones); `p` is each texel's position. `cut(e, p)`
says which texels of a quad (or any face) are solid. Each face is lit, occluded by the other boxes
and shaded like the creatures' (`paint.paint_face_smooth`); `coat` lists the materials painted
soft (fur, straw), the rest stay crisp.
"""

import math

import numpy as np

from paint import majority, occlusion, pack, paint_face_smooth

FACES = {"north": (2, -1), "south": (2, 1), "west": (0, -1), "east": (0, 1), "down": (1, -1), "up": (1, 1)}


def el(name, lo, hi, quad=None, rot=None, origin=None, cut=None, **extra):
    """An element: a box from `lo` to `hi` (a quad: zero thickness, drawn on its `quad` face and
    mirrored on the back), tilted by `rot` (degrees, R = Rz Ry Rx) about `origin`."""
    return {"name": name, "lo": list(map(float, lo)), "hi": list(map(float, hi)), "quad": quad, "rot": rot,
            "origin": origin, "cut": cut, **extra}


def face_frame(lo, hi, face, tx=1.0):
    lo, hi = np.array(lo), np.array(hi)
    X, Y, Z = np.eye(3)
    corner, du, dv = {"south": ((lo[0], hi[1], hi[2]), X, -Y), "north": ((hi[0], hi[1], lo[2]), -X, -Y),
                      "east": ((hi[0], hi[1], hi[2]), -Z, -Y), "west": ((lo[0], hi[1], lo[2]), Z, -Y),
                      "up": ((lo[0], hi[1], lo[2]), X, Z), "down": ((lo[0], lo[1], hi[2]), X, -Z)}[face]
    size = hi - lo
    ax, sgn = FACES[face]
    n = np.zeros(3)
    n[ax] = sgn
    w, h = abs(size @ du), abs(size @ dv)  # in units; in texels at `tx` texels a unit, at least 1
    tw, th = (max(1, int(round(tx * w))), max(1, int(round(tx * h)))) if w > 0 and h > 0 else (0, 0)
    return np.array(corner, float), du, dv, tw, th, n


def rotm(rot):
    if not rot:
        return np.eye(3)
    rx, ry, rz = (math.radians(a) for a in rot)
    X = np.array([[1, 0, 0], [0, math.cos(rx), -math.sin(rx)], [0, math.sin(rx), math.cos(rx)]])
    Y = np.array([[math.cos(ry), 0, math.sin(ry)], [0, 1, 0], [-math.sin(ry), 0, math.cos(ry)]])
    Z = np.array([[math.cos(rz), -math.sin(rz), 0], [math.sin(rz), math.cos(rz), 0], [0, 0, 1]])
    return Z @ Y @ X


def paint(elements, area, materials, coat=(), cut=None, ground=True, width=128, square=False, seed=0, down=None,
          smooth=0, tx=1.0):
    """Paints every face and packs them. Returns (sheet RGBA, {(element index, face): [u0, v0, u1,
    v1]}); `square` pads the sheet square. `down(e, face, p)`: for boxes stacked into one shape (a
    sloped mound of steps, an ear's layers), 0..1 from the top of the whole shape per texel, or
    None: their sides are then lit as one piece, without each box's own edge light (which reads
    as stacked planks). `smooth`: passes of `paint.majority` over the areas (drops specks)."""
    boxes = [(e["name"], np.array(e["lo"]), np.array(e["hi"])) for e in elements if not e["quad"] and not e["rot"]]
    tiles = []
    for i, e in enumerate(elements):
        for face in ([e["quad"]] if e["quad"] else list(FACES)):
            corner, du, dv, w, h, n = face_frame(e["lo"], e["hi"], face, tx)
            if w == 0 or h == 0:
                continue
            uu, vv = np.meshgrid(np.arange(w) + 0.5, np.arange(h) + 0.5)
            size = np.array(e["hi"]) - np.array(e["lo"])
            su, sv = abs(size @ du) / w, abs(size @ dv) / h  # units a texel
            p = corner + (uu * su)[..., None] * du + (vv * sv)[..., None] * dv
            R = rotm(e["rot"])
            if e["rot"]:
                o = np.array(e["origin"], float)
                p = (p - o) @ R.T + o
            a = area(e, face, p, uu, vv, h)
            if smooth and min(a.shape) >= 3:
                a = majority(a, passes=smooth)
            occ = None if e["quad"] else occlusion(p, R @ n, R @ du, R @ dv, boxes, own=e["name"])
            dn = down(e, face, p) if down and face not in ("up", "down") else None
            col = paint_face_smooth(a, materials, face, coat=coat, y_world=p[..., 1] if ground else None, occ=occ,
                                    quad=bool(e["quad"]), seed=(sum(map(ord, e["name"] + face)) + seed) % 97, down=dn)
            img = np.dstack([np.clip(np.round(col), 0, 255), np.full((h, w), 255)]).astype(np.uint8)
            if cut is not None and e["cut"]:
                img[..., 3] = np.where(cut(e, p), 255, 0)
            tiles.append(((i, face), img))
    sheet, uv = pack(tiles, width)
    if square and sheet.shape[0] != sheet.shape[1]:
        side = max(sheet.shape[:2])
        sq = np.zeros((side, side, 4), np.uint8)
        sq[:sheet.shape[0], :sheet.shape[1]] = sheet
        sheet = sq
    return sheet, uv


def entity_spec(elements, uv, sheet, name, bone="body"):
    """A one-bone spec for `to_bedrock.export` (no animations)."""
    els = []
    for i, e in enumerate(elements):
        faces = {f: uv.get((i, f)) for f in ([e["quad"]] if e["quad"] else FACES)}
        els.append({**{k: e[k] for k in ("name", "lo", "hi", "quad", "rot", "origin")}, "bone": bone, "faces": faces})
    return {"name": name, "texture": [sheet.shape[1], sheet.shape[0]],
            "bones": [{"name": bone, "parent": None, "pivot": [0.0, 0.0, 0.0]}], "elements": els, "animations": {}}
