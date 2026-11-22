"""Pictures for the catalogue, made from the mods' textures and models without the game:
flat icons, items built from parts composed as the engine's `core:look` does
(`crates/engine/src/client/stack_look.rs`), blocks as small isometric cubes, and box models drawn
with the blocky-model skill's own renderer."""

import json
import math
import os
import re

import numpy as np
from PIL import Image

# Biome tints for grey textures (`comp::TINT`), a temperate plains' colours.
TINTS = {"grass": (124, 189, 76), "foliage": (92, 160, 48), "water": (63, 118, 228)}


def tile(folder):
    """A connected-texture folder's first tile (`0.png`, else its lowest number), or None."""
    if not os.path.isdir(folder):
        return None
    tiles = [f for f in os.listdir(folder) if re.fullmatch(r"\d+\.png", f)]
    return os.path.join(folder, min(tiles, key=lambda f: int(f[:-4]))) if tiles else None


class Textures:
    """Texture files of a source tree by the engine's names (`base:block/log_oak`)."""

    def __init__(self, root):
        self.root = root
        self.cache = {}
        # The files read since `used` was last emptied (an entry's art, for its dates).
        self.used = set()

    def path(self, name):
        """The file of texture `name`: `<path>.png`, or a connected-texture folder's first tile
        (`<path>/0.png`, beside its `<name>.ctm`, since the Weft sync of 11-22); a tinted layer's
        `@kind` is left out."""
        name = name.split("@", 1)[0]
        ns, rest = name.split(":", 1) if ":" in name else ("base", name)
        for base in (f"{self.root}/mods/{ns}/assets/textures", f"{self.root}/crates/engine/assets/textures"):
            p = f"{base}/{rest}.png"
            if os.path.exists(p):
                return p
            p = tile(f"{base}/{rest}")
            if p:
                return p
        return None

    def get(self, name):
        """RGBA array of the texture's first frame (a strip's top square), or None. `a+b` is `b`
        drawn over `a` (a block face's layers), `name@kind` tinted by the biome colour of `kind`
        (grass, foliage, water)."""
        if "+" in name or "@" in name:
            if name in self.cache:
                return self.cache[name]
            out = None
            for part in name.split("+"):
                base, _, kind = part.partition("@")
                a = self.get(base)
                if a is None:
                    continue
                a = a.astype(float)
                kind = kind.split(":")[-1]
                if kind:
                    # Our own leaf kinds (`evergreen`, `yellow_leaves`) are the biome's foliage.
                    a[..., :3] *= np.array(TINTS.get(kind, TINTS["foliage"])) / 255.0
                if out is None:
                    out = a
                elif a.shape == out.shape:
                    al = a[..., 3:4] / 255.0
                    out[..., :3] = a[..., :3] * al + out[..., :3] * (1 - al)
                    out[..., 3:4] = np.maximum(out[..., 3:4], a[..., 3:4])
            out = None if out is None else np.clip(out, 0, 255).astype(np.uint8)
            self.cache[name] = out
            return out
        p = self.path(name)
        if p:
            self.used.add(p)
        if name in self.cache:
            return self.cache[name]
        a = None
        if p:
            im = Image.open(p).convert("RGBA")
            w, h = im.size
            if h > w and h % w == 0:
                im = im.crop((0, 0, w, w))
            a = np.array(im)
        self.cache[name] = a
        return a

    def raw(self, name):
        """The whole image (a ramp's row), not cut to a frame."""
        p = self.path(name)
        if p:
            self.used.add(p)
        return np.array(Image.open(p).convert("RGBA")) if p else None


def save(a, path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    Image.fromarray(a.astype(np.uint8), "RGBA").save(path, optimize=True)


# ------------------------------------------------------------------ core:look


def compose(look, tex):
    """`stack_look::compose`: layers back to front, `;` separated; a layer is a texture or
    `map@ramp,ramp...`, with `+dx,dy` to move it."""
    out = None
    for layer in [l.strip() for l in look.split(";") if l.strip()]:
        shift = (0, 0)
        if "+" in layer:
            l, at = layer.rsplit("+", 1)
            m = re.fullmatch(r"\s*(-?\d+)\s*,\s*(-?\d+)\s*", at)
            if m:
                layer, shift = l, (int(m.group(1)), int(m.group(2)))
        name, ramps = (layer.split("@", 1) + [None])[:2]
        px = tex.get(name)
        if px is None:
            continue
        w = px.shape[1]
        if px.shape[0] != w:
            continue
        if out is None:
            out = np.zeros((w, w, 4), np.uint8)
        if out.shape[0] != w:
            continue
        ys, xs = np.nonzero(px[..., 3] > 0)
        tx, ty = xs + shift[0], ys + shift[1]
        ok = (tx >= 0) & (tx < w) & (ty >= 0) & (ty < w)
        xs, ys, tx, ty = xs[ok], ys[ok], tx[ok], ty[ok]
        if ramps is None:
            out[ty, tx] = px[ys, xs]
            continue
        rs = [tex.raw(r.strip()) for r in ramps.split(",")]
        for x, y, X, Y in zip(xs, ys, tx, ty):
            p = px[y, x]
            region = int(p[1])
            if region >= len(rs) or rs[region] is None:
                continue
            r = rs[region][0]
            step = min(int(p[0]) // 36, 7, r.shape[0] - 1)
            out[Y, X] = [r[step][0], r[step][1], r[step][2], p[3]]
    return out


class Looks:
    """`crafting::looks` and `parts::look_text`, from the parsed tables (`sources.looks`)."""

    def __init__(self, looks, kinds, shapes, part_maps, metals):
        self.looks, self.kinds, self.shapes, self.part_maps, self.metals = looks, kinds, shapes, part_maps, metals
        self.shells_drawn, self.drawn_shapes = [], {}

    @staticmethod
    def art_name(m):
        return {"oak": "wood", "birch": "wood", "spruce": "wood", "mesquite": "wood", "branch": "wood",
                "wrought_iron": "iron"}.get(m, m)

    def drawn(self, kind, m):
        l = self.looks.get(kind)
        if not l:
            return self.art_name(m)
        has = lambda a: not l["anchors"] or a in l["anchors"]
        art = self.art_name(m)
        for _ in range(3):
            if has(art):
                return art
            art = {"meteoric_iron": "iron", "star_steel": "steel", "chert": "flint", "obsidian": "flint",
                   "tektite": "flint", "flint": "stone", "bone": "stone", "antler": "stone"}.get(art, "iron")
        return art

    def shift(self, kind, m):
        l = self.looks.get(kind)
        art = self.drawn(kind, m)
        if not l or art not in l["anchors"]:
            return (0, 0)
        a = l["anchors"][art]
        return (round(a[0] - l["socket"][0]), round(a[1] - l["socket"][1]))

    def first_shape(self, kind, slot):
        return next((s for s, k, sl in self.shapes if k == kind and sl == slot), None)

    # `looks::shell_family`: how a shell is built, and so drawn.
    FAMILY = {"leather": "leather", "rawhide": "leather", "bone": "bone", "antler": "bone", "diamond": "diamond"}

    def part_look(self, shape, m):
        """`looks::part_look` (art redo v3): a loose head or blade's own drawing
        (`parts/loose/<shape>/<art>`), a shell drawn for its material (`parts/armour/<shell>/<art>`),
        else its family's map painted with the material's ramp."""
        kind_slot = next(((k, sl) for s, k, sl in self.shapes if s == shape), None)
        if kind_slot:
            l = self.looks.get(kind_slot[0])
            if l and any(x["painted"] and x["regions"][0] == kind_slot[1] for x in l["layers"]):
                return f"base:parts/loose/{self.drawn_shapes.get(shape, shape)}/{self.drawn(kind_slot[0], m)}"
        if shape not in self.part_maps:
            return None
        buckle = ""
        if shape == "straps":
            buckle = f",base:ramps/{m}" if m in self.metals else ",base:ramps/wrought_iron"
        if shape.endswith("_shell") and self.art_name(m) in self.shells_drawn:
            return f"base:parts/armour/{shape}/{self.art_name(m)}"
        fam = self.FAMILY.get(m, "plate")
        drawn = "cap_shell" if (fam, shape) == ("leather", "helmet_shell") else (f"bone_{shape}" if fam == "bone" and shape.endswith("_shell") else shape)
        return f"base:parts/armour/{drawn}@base:ramps/{m}{buckle}"

    def text(self, kind, parts):
        """`look_text`: `parts` {slot: (shape, material)}."""
        l = self.looks.get(kind)
        if not l:
            return ""
        working = next((s for s, r in self.kinds[kind] if r == "Working"), None)
        wm = parts.get(working, (None, None))[1]
        sh = self.shift(kind, wm) if wm else (0, 0)
        out = []
        for layer in l["layers"]:
            at = f"+{sh[0]},{sh[1]}" if layer["moves"] and sh != (0, 0) else ""
            r0 = layer["regions"][0]
            if r0 not in parts:
                continue
            shape, m = parts[r0]
            if layer["painted"]:
                out.append(f"base:parts/{kind}/{shape}/{self.drawn(kind, m)}{at}")
                continue
            if any(r not in parts for r in layer["regions"]):
                continue
            ramps = ",".join(f"base:ramps/{parts[r][1]}" for r in layer["regions"])
            out.append(f"base:parts/{kind}/{shape}@{ramps}{at}")
        return ";".join(out)


# ------------------------------------------------------------------ blocks


def iso_boxes(boxes, top, south, east, size=128, tint_top=None, tint_sides=None):
    """Boxes of a block (x0, y0, z0, x1, y1, z1 in block units) as the inventory shows a block:
    tops and two sides (south on the left, east on the right) in 2:1 isometric, sides darker,
    each face showing the part of its texture where it sits in the block, nearest drawn last."""
    # As Minecraft's inventory draws a block (turned 45°, tipped 30°): the top a 2:1 diamond and
    # the upright edges cos 30° / sin 45° = 1.225 times the diamond's half-width (they were drawn
    # 1.0 times, which squashed every block; Ronald, 11-05: "the blocks are sqished a litlte").
    u = size / (1 + 1.2247)
    V = 1.2247 * u
    out = np.zeros((size, size, 4), np.uint8)
    depth = np.full((size, size), -1e9)
    gy, gx = np.mgrid[0:size, 0:size]
    px, py = gx + 0.5, gy + 0.5
    P = lambda x, y, z: np.array([size / 2 + (x - z) * u, (x + z) * 0.5 * u + (1 - y) * V])

    def face(img, c0, c1, c2, uv, shade, tint):
        """The face spanned from corner c0 along c1 - c0 and c2 - c0 (3D); `uv(a, b)` the texture
        point (0-1) at fractions a, b along those edges."""
        if img is None:
            return
        o, e1, e2 = P(*c0), P(*c1) - P(*c0), P(*c2) - P(*c0)
        det = e1[0] * e2[1] - e1[1] * e2[0]
        if abs(det) < 1e-9:
            return
        dx, dy = px - o[0], py - o[1]
        a = (dx * e2[1] - dy * e2[0]) / det
        b = (e1[0] * dy - e1[1] * dx) / det
        ok = (a >= 0) & (a < 1) & (b >= 0) & (b < 1)
        h, w = img.shape[:2]
        u, v = uv(a, b)
        tu = np.clip((u * w).astype(int), 0, w - 1)
        tv = np.clip((v * h).astype(int), 0, h - 1)
        p3 = np.array(c0, float)[:, None, None] + (np.array(c1) - np.array(c0))[:, None, None] * a + (np.array(c2) - np.array(c0))[:, None, None] * b
        d = p3.sum(0)
        col = img[tv, tu].astype(float)
        if tint is not None:
            col[..., :3] *= np.array(tint) / 255.0
        col[..., :3] *= shade
        ok &= (col[..., 3] > 0) & (d > depth)
        out[ok] = np.clip(col[ok], 0, 255).astype(np.uint8)
        depth[ok] = d[ok]

    for x0, y0, z0, x1, y1, z1 in boxes:
        face(top, (x0, y1, z0), (x1, y1, z0), (x0, y1, z1),
             lambda a, b: (x0 + a * (x1 - x0), z0 + b * (z1 - z0)), 1.0, tint_top)
        face(south, (x0, y1, z1), (x1, y1, z1), (x0, y0, z1),
             lambda a, b: (x0 + a * (x1 - x0), 1 - (y1 - b * (y1 - y0))), 0.8, tint_sides)
        face(east, (x1, y1, z1), (x1, y1, z0), (x1, y0, z1),
             lambda a, b: (1 - (z1 - a * (z1 - z0)), 1 - (y1 - b * (y1 - y0))), 0.62, tint_sides)
    return out


def iso_cube(top, left, right, size=128, tint_top=None, tint_sides=None):
    """A whole block (`iso_boxes` of one full box)."""
    return iso_boxes([(0, 0, 0, 1, 1, 1)], top, left, right, size, tint_top, tint_sides)


# Shapes of a block by its id's ending, in block units.
SHAPES = {"_vertical_slab": [(0, 0, 0, 1, 1, 0.5)], "_slab": [(0, 0, 0, 1, 0.5, 1)],
          "_stairs": [(0, 0, 0, 1, 0.5, 1), (0, 0.5, 0, 1, 1, 0.5)],
          # Upstream's (`shapes.rs`, the Weft sync of 11-22): a beam is a quarter block
          # (1 x 1/2 x 1/2), an eighth half a block each way; drawn in the middle of the cell.
          "_beam": [(0, 0, 0.25, 1, 0.5, 0.75)], "_eighth": [(0.25, 0, 0.25, 0.75, 0.5, 0.75)]}


def shape_of(i):
    for end, boxes in SHAPES.items():
        if i.endswith(end):
            return end, boxes
    return None, None


def tint_of(block):
    comps = block.get("comps", [])
    if "tint" not in comps:
        return None, False
    i = block["id"]
    if "water" in i or "kelp" in i or "seagrass" in i:
        c = TINTS["water"] if "water" == i.split(":")[1] else TINTS["grass"]
    elif "leaves" in i or "vine" in i or "liana" in i:
        c = TINTS["foliage"]
    else:
        c = TINTS["grass"]
    return c, "tint_faces" in comps


def block_icon(block, tex):
    """(image, how) for a block from the dump: a cube for cubes, its face flat for plants and
    the rest."""
    t = block["textures"]
    imgs = [tex.get(x) if x else None for x in t]
    # A face named `name@kind` carries its own tint (`Textures.get`).
    tint, top_only = tint_of(block) if not any("@" in (x or "") for x in t) else (None, False)
    end, boxes = shape_of(block["id"])
    if block.get("boxes") and imgs[3] is not None:
        # Boxes the sources give (a snow layer, a button, loose stones): drawn where they sit.
        side = lambda k: imgs[k] if imgs[k] is not None else imgs[3]
        return iso_boxes(block["boxes"], imgs[3], side(5), side(1), tint_top=tint, tint_sides=None if top_only else tint), "cube"
    if boxes and imgs[3] is not None:
        return iso_boxes(boxes, imgs[3], imgs[5] if imgs[5] is not None else imgs[3], imgs[1] if imgs[1] is not None else imgs[3]), "cube"
    if block["model"] == "Boxes" and imgs[3] is not None and imgs[5] is not None and t[3] != t[5]:
        # A box model with a top of its own (a chest): drawn as a cube, its front turned to us.
        front = next((k for k in range(6) if "front" in t[k]), 5)
        return iso_cube(imgs[3], imgs[front], imgs[1] if front != 1 else imgs[5], tint_top=tint, tint_sides=None if top_only else tint), "cube"
    if block["model"] == "Cube" and imgs[3] is not None:
        return iso_cube(imgs[3], imgs[5] if imgs[5] is not None else imgs[3], imgs[1] if imgs[1] is not None else imgs[3],
                        tint_top=tint, tint_sides=None if top_only else tint), "cube"
    for i in (5, 1, 3, 0, 2, 4):
        if imgs[i] is not None:
            a = imgs[i].copy()
            if tint is not None:
                a = a.astype(float)
                a[..., :3] *= np.array(tint) / 255.0
                a = a.astype(np.uint8)
            return a, "flat"
    return None, None


# ------------------------------------------------------------------ box models


def _keys(v):
    """An animation channel's first value: a list, {"0": list}, {"0": {"post": list}}."""
    if isinstance(v, list):
        return v
    if isinstance(v, dict) and v:
        k = sorted(v, key=lambda x: float(x) if re.fullmatch(r"[\d.]+", x) else 0)[0]
        x = v[k]
        if isinstance(x, dict):
            x = x.get("post") or x.get("pre") or [0, 0, 0]
        return x
    return [0, 0, 0]


def _num(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return 0.0


def _at(v, t):
    """A channel's value at the key nearest time `t`."""
    if not isinstance(v, dict) or not v:
        return _keys(v)
    ks = [k for k in v if re.fullmatch(r"[\d.]+", k)]
    if not ks:
        return _keys(v)
    k = min(ks, key=lambda k: abs(float(k) - t))
    x = v[k]
    if isinstance(x, dict):
        x = x.get("post") or x.get("pre") or [0, 0, 0]
    return x


def calmest(anim, names):
    """The time in a clip whose pose turns the bones least (an idle that starts mid-graze is
    drawn standing)."""
    times = set()
    for ch in anim.get("bones", {}).values():
        if isinstance(ch.get("rotation"), dict):
            times |= {float(k) for k in ch["rotation"] if re.fullmatch(r"[\d.]+", k)}
    if not times:
        return 0.0

    def turn(t):
        s = 0.0
        for n, ch in anim.get("bones", {}).items():
            if n in names and "rotation" in ch:
                r = _at(ch["rotation"], t)
                if isinstance(r, list):
                    s += sum(abs(_num(x)) for x in r)
        return s

    return min(sorted(times), key=turn)




def box_uv(u, v, size, mirror=False):
    """Bedrock's box UV layout for a cube of `size`: {file face: [u0, v0, u1, v1]} (seen from
    outside)."""
    w, h, d = size
    f = {"east": [u, v + d, u + d, v + d + h], "north": [u + d, v + d, u + d + w, v + d + h],
         "west": [u + d + w, v + d, u + 2 * d + w, v + d + h], "south": [u + 2 * d + w, v + d, u + 2 * d + 2 * w, v + d + h],
         "up": [u + d + w, v + d, u + d, v], "down": [u + d + 2 * w, v, u + d + w, v + d]}
    if mirror:
        f = {k: [r[2], r[1], r[0], r[3]] for k, r in f.items()}
        f["east"], f["west"] = f["west"], f["east"]
    return f


def geo_to_spec(geo, anim=None):
    """A Bedrock geometry (as the game reads it) as the blocky-model renderer's spec, and its pose
    at rest (each bone's own rotation, plus `anim`'s first frame when given). The inverse of the
    skill's `to_bedrock.geometry`: the model turned back to face +z (x and z negated, north and
    south swapped), up and down faces' rects unturned."""
    g = geo["minecraft:geometry"][0] if "minecraft:geometry" in geo else geo
    desc = g.get("description", {})
    tw, th = desc.get("texture_width", 64), desc.get("texture_height", 64)
    flip = lambda p: [-_num(p[0]), _num(p[1]), -_num(p[2])]
    bones, elements, pose = [], [], {}
    names = set()
    for b in g.get("bones", []):
        n = b["name"]
        names.add(n)
        bone = {"name": n, "pivot": flip(b.get("pivot", [0, 0, 0]))}
        if b.get("parent"):
            bone["parent"] = b["parent"]
        bones.append(bone)
        if b.get("rotation"):
            pose[n] = [_num(x) for x in b["rotation"]]
        for k, c in enumerate(b.get("cubes", [])):
            o, s = c["origin"], c["size"]
            inf = _num(c.get("inflate", 0))
            a = np.array([_num(x) for x in o]) - inf
            z = np.array([_num(x) for x in s]) + 2 * inf
            p0, p1 = flip(a), flip(a + z)
            lo, hi = np.minimum(p0, p1).tolist(), np.maximum(p0, p1).tolist()
            uv = c.get("uv", [0, 0])
            faces = {}
            if isinstance(uv, list):
                fr = box_uv(_num(uv[0]), _num(uv[1]), [_num(x) for x in s], c.get("mirror", b.get("mirror", False)))
            else:
                fr = {}
                for face, d in uv.items():
                    u0, v0 = d["uv"]
                    us, vs = d.get("uv_size", [0, 0])
                    if face == "up":
                        fr[face] = [u0 + us, v0 + vs, u0, v0]
                    elif face == "down":
                        fr[face] = [u0 + us, v0, u0, v0 + vs]
                    else:
                        fr[face] = [u0, v0, u0 + us, v0 + vs]
            swap = {"north": "south", "south": "north", "east": "east", "west": "west", "up": "up", "down": "down"}
            for face, r in fr.items():
                faces[swap[face]] = r
            e = {"name": f"{n}_{k}", "bone": n, "lo": lo, "hi": hi, "faces": faces}
            if c.get("rotation") and any(_num(x) for x in c["rotation"]):
                e["rot"] = [_num(x) for x in c["rotation"]]
                e["origin"] = flip(c.get("pivot", [0, 0, 0]))
            elements.append(e)
    if anim:
        t_pose = calmest(anim, names)
        for n, ch in anim.get("bones", {}).items():
            if n in names and "rotation" in ch:
                r = _at(ch["rotation"], t_pose)
                if isinstance(r, list) and len(r) == 3:
                    base = pose.get(n, [0, 0, 0])
                    pose[n] = [base[i] + _num(r[i]) for i in range(3)]
            if n == "root" and "position" in ch:
                p = _at(ch["position"], t_pose)
                if isinstance(p, list) and len(p) == 3:
                    pose["root"] = flip(p)
    # parents before children
    order, seen = [], set()
    by = {b["name"]: b for b in bones}

    def visit(b):
        if b["name"] in seen:
            return
        if b.get("parent") in by:
            visit(by[b["parent"]])
        elif b.get("parent"):
            b.pop("parent")
        seen.add(b["name"])
        order.append(b)

    for b in bones:
        visit(b)
    return {"texture": [tw, th], "bones": order, "elements": elements}, pose


def render_model(render_mod, spec, texture, pose, view, box=300):
    """The model at `view` ('3q' or 'side'), fitted into `box` pixels, crisp texels."""
    yaw, pitch = render_mod.VIEWS[view]
    tex = np.array(Image.open(texture).convert("RGBA")) if isinstance(texture, str) else texture
    tw, th = spec["texture"]
    if tex.shape[1] != tw:  # rects are in the file's units, the texture may be denser
        f = tex.shape[1] / tw
        for e in spec["elements"]:
            e["faces"] = {k: [x * f for x in r] for k, r in e["faces"].items()}
        spec["texture"] = [tex.shape[1], tex.shape[0]]
    probe = render_mod.render(spec, tex, pose=pose, yaw=yaw, pitch=pitch, ppu=1.0, pad=0)
    h, w = probe.shape[:2]
    ppu = max(0.5, min((box - 16) / max(w, 1), (box - 16) / max(h, 1)))
    return render_mod.render(spec, tex, pose=pose, yaw=yaw, pitch=pitch, ppu=ppu, pad=8)


def trim(a, pad=4):
    ys, xs = np.nonzero(a[..., 3])
    if not len(xs):
        return a
    y0, y1, x0, x1 = max(0, ys.min() - pad), ys.max() + 1 + pad, max(0, xs.min() - pad), xs.max() + 1 + pad
    return a[y0:y1, x0:x1]
