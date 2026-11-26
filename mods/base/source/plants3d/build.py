"""3D plants as a resource pack (Ronald, 11-05: "let's try model like grass or like sugarcane crops in 3D. I want
to see how that looks like"): tall grass, sugar cane and three crops (wheat, carrots, beans) through their stages,
as box models in the spirit of 3D-crops packs, Better Foliage and Vintage Story, blocky at 32 texels a block.

Each plant's texture gets a `.ctm` in the pack that swaps the flat crossed picture for a model tile
(docs/ctm.md, "Models as tiles"): random variants by position for grass and crops (each its own layout, so a
field doesn't repeat), the vertical method for sugar cane (bottom, middle, top, alone). Only the drawing
changes; collision, selection and light stay the mod's. The pack's switch is the resource pack list.

    python3 mods/base/source/plants3d/build.py      # writes mods/base/packs/plants_3d/
"""
import math
import os
import random
import shutil
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from model import Box, build, write, ramp, along, quads  # noqa: E402

ROOT = os.path.normpath(os.path.join(HERE, "..", "..", "packs", "plants_3d"))
TEX = os.path.join(ROOT, "base", "textures", "block")
MOD = os.path.join(ROOT, "base", "models", "block")
VARIANTS = 3


BLOCK = os.path.normpath(os.path.join(HERE, "..", "..", "assets", "textures", "block"))


def palette(tex, rows=None, drop=None, n=4):
    """A dark-to-light ramp from a flat texture's own pixels (rows: a slice of rows; drop(r,g,b): pixels to leave out)."""
    import numpy as np
    from PIL import Image
    a = np.asarray(Image.open(os.path.join(BLOCK, tex + ".png")).convert("RGBA")).astype(int)
    if rows:
        a = a[rows[0]:rows[1]]
    px = a[a[..., 3] > 127][:, :3]
    if drop is not None and len(px):
        keep = ~np.array([drop(*c) for c in px], bool)
        px = px[keep] if keep.any() else px
    if not len(px):
        return ["#808080", "#a0a0a0"]
    lum = px @ np.array([0.2126, 0.7152, 0.0722])
    px = px[np.argsort(lum)]
    qs = [0.08, 0.35, 0.65, 0.92][:n]
    return ["#%02x%02x%02x" % tuple(px[int(q * (len(px) - 1))]) for q in qs]


def jitter(rng, k):
    return rng.uniform(-k, k)


# ---------- tall grass (tinted: grey, the biome colour comes from the block) ----------

def blade_paint(shade):
    def p(b, face, u, v, w, h):
        t = along(b, face, v, h)
        g = 128 + 100 * t ** 0.8 + shade
        if face == "up":
            g = 226 + shade
        if face in ("east", "west"):
            g -= 10
        g = max(60, min(238, g))
        return (int(g), int(g), int(g), 255)
    return p


def tallgrass(seed):
    rng = random.Random(seed)
    boxes = []
    n = rng.randint(12, 13)
    for i in range(n):
        a = 2 * math.pi * (i + rng.uniform(-0.3, 0.3)) / n
        r = rng.uniform(0.5, 4.2)
        cx, cz = 8 + r * math.cos(a), 8 + r * math.sin(a)
        h = rng.uniform(5.0, 12.5) * (1.0 - 0.25 * r / 4.2)
        # a strap: two texels wide across the lean, half a texel thick, leaning out from the middle about
        # the axis across that direction (so its flat side faces out)
        lean = rng.uniform(10, 34) * (0.5 + r / 4.2)
        if abs(math.cos(a)) > abs(math.sin(a)):
            lo, hi = [cx - 0.125, 0, cz - 0.5], [cx + 0.125, h, cz + 0.5]
            rot = ([cx, 0, cz], "z", -lean * math.copysign(1, math.cos(a)))
        else:
            lo, hi = [cx - 0.5, 0, cz - 0.125], [cx + 0.5, h, cz + 0.125]
            rot = ([cx, 0, cz], "x", lean * math.copysign(1, math.sin(a)))
        boxes.append(Box(lo, hi, blade_paint(rng.randint(-14, 10)), rot, faces=["north", "south", "east", "west", "up"]))
    return boxes


# ---------- sugar cane (not tinted) ----------

CANE = ["#4e6e2a", "#6f9a3c", "#88b24a", "#a4c860"]


def stalk_paint(phase):
    def p(b, face, u, v, w, h):
        if face in ("up", "down"):
            return (*ramp(["#b8d070", "#d4e08a"], 0.5), 255)
        y = int(round(b.hi[1] * 2)) - 1 - v  # texel row in the block, 0 at the bottom
        k = (y + phase) % 10
        side = 0.75 if u == 0 else (0.45 if u == w - 1 else 0.6)
        if face in ("east", "west"):
            side -= 0.1
        c = ramp(CANE, side)
        if k == 0:      # the node: a light ring
            c = ramp(["#a8c464", "#c8dc84"], 0.6)
        elif k == 1:    # and its dark line just below
            c = ramp(CANE, 0.1)
        return (*c, 255)
    return p


def leaf_paint(b, face, u, v, w, h):
    t = along(b, face, v, h)
    c = ramp(["#527428", "#74a03c", "#94bc50", "#a8cc5c"], 0.3 + 0.6 * t)
    if face == "down":
        c = ramp(["#527428", "#74a03c"], 0.5)
    return (*c, 255)


def cane(piece, seed):
    """piece: 0 bottom, 1 middle, 2 top, 3 alone (the vertical method's order)."""
    rng = random.Random(seed)
    boxes = []
    spots = [(5.0, 5.5), (10.5, 6.0), (7.0, 10.8)]
    for j, (x, z) in enumerate(spots):
        x, z = x + jitter(rng, 0.6), z + jitter(rng, 0.6)
        w = 1.5  # 3 texels
        top = 16
        if piece in (2, 3):
            top = [14.5, 12.5, 15.5][j]
        boxes.append(Box([x - w / 2, 0, z - w / 2], [x + w / 2, top, z + w / 2], stalk_paint(3 * j + rng.randint(0, 2)),
                         faces=["north", "south", "east", "west"] + (["up"] if top < 16 else [])))
        # leaves: one now and then on a middle piece, a crown of two or three at the top
        nleaf = {0: 0, 1: 1, 2: 3, 3: 3}[piece]
        for k in range(nleaf):
            a = rng.uniform(0, 2 * math.pi)
            ly = (top - rng.uniform(1.5, 5.0)) if piece in (2, 3) else rng.uniform(5, 12)
            ln = rng.uniform(8, 11)
            lw = 1.5
            dx, dz = math.cos(a), math.sin(a)
            # a thin strap from the stalk out and up, leaning about the axis across it
            if abs(dx) > abs(dz):
                lo = [x + w / 2 - 0.25, ly, z - lw / 2]
                hi = [x + w / 2, ly + ln, z + lw / 2]
                rot = ([x, ly, z], "z", -math.copysign(rng.uniform(40, 55), dx))
            else:
                lo = [x - lw / 2, ly, z + w / 2 - 0.25]
                hi = [x + lw / 2, ly + ln, z + w / 2]
                rot = ([x, ly, z], "x", math.copysign(rng.uniform(40, 55), dz))
            boxes.append(Box(lo, hi, leaf_paint, rot, faces=["north", "south", "east", "west", "up"]))
    return boxes


# ---------- crops (not tinted) ----------

def plain(cols):
    def p(b, face, u, v, w, h):
        t = along(b, face, v, h)
        s = 0.25 + 0.65 * t
        if face in ("east", "west"):
            s -= 0.08
        if face == "up":
            s = 0.95
        return (*ramp(cols, s), 255)
    return p


WHEAT_STEM = {0: ["#3e5a1c", "#5a7c2a", "#78a038"], 1: ["#3e5a1c", "#5a7c2a", "#7aa23a"], 2: ["#466422", "#62882e", "#82aa3e"],
              3: ["#4a6824", "#688e30", "#88ae42"], 4: ["#56702a", "#7a9638", "#9cb448"], 5: ["#6a7430", "#8e9640", "#b0b054"],
              6: ["#7e6a32", "#a68c46", "#c8ac60"], 7: ["#7a5e2e", "#a8823e", "#cca458"]}
WHEAT_EAR = {4: ["#6a8a30", "#8cae42", "#a8c656"], 5: ["#8e9a3e", "#b0b452", "#c8c466"], 6: ["#a6863e", "#c8a656", "#dcc074"],
             7: ["#a87c38", "#cc9e50", "#e4bc6c"]}


def wheat(stage, seed):
    rng = random.Random(seed)
    boxes = []
    h_full = [1.5, 3.0, 4.5, 6.5, 9.0, 11.0, 12.5, 13.0][stage]
    tex = f"wheat_stage_{stage}"
    stem = palette(tex, rows=(18, 32))
    ear = palette(tex, rows=(0, 22), drop=lambda r, g, b: abs(r - g) < 8 and abs(g - b) < 8) if stage >= 4 else None
    # four clumps in a jittered 2 x 2, three stems each, ears on two of them
    for cx, cz in ((4.5, 4.5), (11.5, 4.5), (4.5, 11.5), (11.5, 11.5)):
        cx, cz = cx + jitter(rng, 1.0), cz + jitter(rng, 1.0)
        for s in range(3):
            x, z = cx + jitter(rng, 1.4), cz + jitter(rng, 1.4)
            h = h_full * rng.uniform(0.8, 1.05)
            w = 0.5
            tilt = rng.uniform(-16, 16) * (1.6 if stage == 7 else 1.0)
            rot = ([x, 0, z], rng.choice("xz"), tilt)
            has_ear = ear is not None and s < 2
            stem_h = h - (3.0 if has_ear else 0)
            boxes.append(Box([x - w / 2, 0, z - w / 2], [x + w / 2, stem_h, z + w / 2], plain(stem), rot,
                             faces=["north", "south", "east", "west"] + ([] if has_ear else ["up"])))
            if has_ear:
                ew = 0.75 if stage < 6 else 1.0
                boxes.append(Box([x - ew / 2, stem_h, z - ew / 2], [x + ew / 2, h, z + ew / 2], plain(ear), rot,
                                 faces=["north", "south", "east", "west", "up"]))
    return boxes


CARROT_LEAF = ["#3c5a1e", "#58802c", "#76a23c", "#8cb84a"]
CARROT_ROOT = ["#a8501c", "#d0702a", "#e8903c"]


def carrot(stage, seed):
    rng = random.Random(seed)
    boxes = []
    leafc = palette(f"carrot_stage_{stage}", drop=lambda r, g, b: r > g + 30)
    h_full = [2.0, 4.5, 7.0, 8.5][stage]
    for cx, cz in ((5, 5.5), (11, 6), (8, 11.5)):
        cx, cz = cx + jitter(rng, 0.8), cz + jitter(rng, 0.8)
        if stage == 3:  # the orange shoulder showing at the soil
            boxes.append(Box([cx - 1.0, -0.5, cz - 1.0], [cx + 1.0, 1.0, cz + 1.0], plain(CARROT_ROOT), faces=["north", "south", "east", "west", "up"]))
        n = [2, 3, 3, 4][stage]
        for k in range(n):
            a = 2 * math.pi * k / n + jitter(rng, 0.4)
            h = h_full * rng.uniform(0.75, 1.05)
            w = 0.5
            lo, hi = [cx - w / 2, 0.5 if stage == 3 else 0, cz - w / 2], [cx + w / 2, h, cz + w / 2]
            lean = rng.uniform(18, 35)
            if abs(math.cos(a)) > abs(math.sin(a)):
                rot = ([cx, 0, cz], "z", -lean * math.copysign(1, math.cos(a)))
            else:
                rot = ([cx, 0, cz], "x", lean * math.copysign(1, math.sin(a)))
            boxes.append(Box(lo, hi, plain(leafc), rot, faces=["north", "south", "east", "west", "up"]))
            if stage >= 2 and k % 2 == 0:
                # the feathery top: two short leaflets forking off near the tip
                for side in (-1, 1):
                    fy = h * 0.7
                    lo2, hi2 = [cx - 0.25, fy, cz - 0.25], [cx + 0.25, fy + h * 0.35, cz + 0.25]
                    ax = rot[1]
                    boxes.append(Box(lo2, hi2, plain(leafc), ([cx, 0, cz], "x" if ax == "z" else "z", side * rng.uniform(25, 40)),
                                     faces=["north", "south", "east", "west", "up"]))
    return boxes


BEAN_LEAF = ["#2e5422", "#4a7a30", "#62983c", "#78ac48"]
BEAN_STEM = ["#3a5a22", "#527a2e", "#6a9238"]


def bean_leaf_paint(flowers):
    def p(b, face, u, v, w, h):
        c = ramp(BEAN_LEAF, 0.8 if face == "up" else (0.25 if face == "down" else 0.5))
        if flowers and face == "up" and (u * 7 + v * 3 + int(b.lo[0] * 3)) % 9 == 0:
            c = (236, 232, 220)
        return (*c, 255)
    return p


def beans(stage, seed):
    rng = random.Random(seed)
    boxes = []
    global BEAN_LEAF
    BEAN_LEAF = palette(f"beans_stage_{stage}", drop=lambda r, g, b: r > 200 and g > 200)
    h_full = [2.0, 5.0, 9.0, 10.0][stage]
    plants = [(5.5, 5.5), (10.5, 10.5)]
    for cx, cz in plants:
        cx, cz = cx + jitter(rng, 0.8), cz + jitter(rng, 0.8)
        h = h_full * rng.uniform(0.85, 1.05)
        boxes.append(Box([cx - 0.375, 0, cz - 0.375], [cx + 0.375, h, cz + 0.375], plain(BEAN_STEM), faces=["north", "south", "east", "west"]))
        # leaf pads: trifoliate leaves as small flat boxes up the stem, tilted
        nl = [1, 3, 6, 7][stage]
        for k in range(nl):
            y = h * (0.35 + 0.65 * (k + 1) / nl)
            a = rng.uniform(0, 2 * math.pi)
            d = [1.0, 1.4, 1.9, 2.1][stage]
            x, z = cx + d * math.cos(a), cz + d * math.sin(a)
            s = [0.75, 1.0, 1.25, 1.25][stage]
            rot = ([x, y, z], rng.choice("xz"), rng.choice([-1, 1]) * rng.uniform(15, 40))
            boxes.append(Box([x - s, y, z - s], [x + s, y + 0.5, z + s], bean_leaf_paint(stage == 2), rot, faces=["north", "south", "east", "west", "up"]))
        if stage == 3:  # pods hanging under the leaves
            for k in range(3):
                a = rng.uniform(0, 2 * math.pi)
                x, z = cx + 1.3 * math.cos(a), cz + 1.3 * math.sin(a)
                y = h * rng.uniform(0.35, 0.6)
                boxes.append(Box([x - 0.375, y - 3.0, z - 0.25], [x + 0.375, y, z + 0.25], plain(["#5a8a30", "#78a840", "#94c056"]),
                                 ([x, y, z], "z", rng.uniform(-15, 15)), faces=["north", "south", "east", "west"]))
    return boxes


# ---------- the pack ----------

PLANTS = [("tallgrass", lambda k: tallgrass(100 + k))]
for st in range(8):
    PLANTS.append((f"wheat_stage_{st}", (lambda st: lambda k: wheat(st, 200 + 10 * st + k))(st)))
for st in range(4):
    PLANTS.append((f"carrot_stage_{st}", (lambda st: lambda k: carrot(st, 300 + 10 * st + k))(st)))
    PLANTS.append((f"beans_stage_{st}", (lambda st: lambda k: beans(st, 400 + 10 * st + k))(st)))


def main():
    if os.path.isdir(os.path.join(ROOT, "base")):
        shutil.rmtree(os.path.join(ROOT, "base"))
    os.makedirs(os.path.join(TEX, "p3d"), exist_ok=True)
    os.makedirs(os.path.join(MOD, "p3d"), exist_ok=True)
    with open(os.path.join(ROOT, "pack.cfg"), "w") as f:
        f.write("name = 3D plants\ndescription = Tall grass, sugar cane, wheat, carrots and beans as small box models (a prototype)\n")
    report = []
    for tex, make in PLANTS:
        names = []
        for k in range(VARIANTS):
            name = f"p3d/{tex}_{k}"
            m, img = build(make(k))
            write(m, img, os.path.join(MOD, name + ".json"), os.path.join(TEX, name + ".png"))
            names.append(name)
            report.append((name, quads(m), img.size[0]))
        # upstream's form: the rules in a folder named after the texture, tiles named in full
        os.makedirs(os.path.join(TEX, tex), exist_ok=True)
        with open(os.path.join(TEX, tex, tex + ".ctm"), "w") as f:
            f.write(f"# 3D plants pack (mods/base/source/plants3d/build.py): {VARIANTS} model tiles picked by position\n"
                    f"method = random\ntiles = {' '.join('base:block/' + n for n in names)}\n")
    # sugar cane: the vertical method's four pieces, each a model
    names = []
    for piece in range(4):
        name = f"p3d/reeds_{piece}"
        m, img = build(cane(piece, 500 + piece))
        write(m, img, os.path.join(MOD, name + ".json"), os.path.join(TEX, name + ".png"))
        names.append(name)
        report.append((name, quads(m), img.size[0]))
    os.makedirs(os.path.join(TEX, "reeds"), exist_ok=True)
    with open(os.path.join(TEX, "reeds", "reeds.ctm"), "w") as f:
        f.write("# 3D plants pack (mods/base/source/plants3d/build.py): bottom, middle, top and alone, each a model\n"
                f"method = vertical\nconnect = block\ntiles = {' '.join('base:block/' + n for n in names)}\n")
    # the list base builds into the game (src/plants3d.rs writes them next to it as a resource pack)
    files = []
    for dirpath, _, names in os.walk(ROOT):
        for n in sorted(names):
            full = os.path.join(dirpath, n)
            files.append(os.path.relpath(full, ROOT))
    files.sort()
    src = os.path.normpath(os.path.join(HERE, "..", "..", "src", "plants3d_files.rs"))
    with open(src, "w") as f:
        f.write("// Generated by mods/base/source/plants3d/build.py: the 3D plants pack's files. Don't edit.\n")
        f.write("pub const FILES: &[(&str, &[u8])] = &[\n")
        for r in files:
            f.write(f'    ("{r}", include_bytes!("../packs/plants_3d/{r}")),\n')
        f.write("];\n")
    for name, q, side in report:
        print(f"{name}: {q} quads, {side} px")
    print(f"{len(files)} files in the pack; {src}")


if __name__ == "__main__":
    main()
