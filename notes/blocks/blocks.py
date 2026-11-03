"""Block textures at 32 px, made with Codex from `style.txt` (Ronald's master prompt): four a
generated image, cut into game tiles, checked on a preview sheet, put into the mods.

    python3 notes/blocks/blocks.py gen batches/terrain.tsv [image ...]   # generate (4 at a time)
    python3 notes/blocks/blocks.py cut batches/terrain.tsv [image ...]   # gen/*.png -> tiles/
    python3 notes/blocks/blocks.py preview batches/terrain.tsv           # preview/<batch>.png
    python3 notes/blocks/blocks.py install batches/terrain.tsv           # tiles -> mods/*/textures/block

A batch is a TSV, one generated image a row: `image  mod  entry  entry  entry  entry`, an entry
`name|kind|what it is` (`-` for an empty square). `mod` is where new textures go (base or animals);
a texture the game already has is replaced where it is. `name@mod` overrides the mod for one
entry, `name@-` keeps the tile out of the game. The kinds (`KINDS`) add the master prompt's type
line and say how the square is cut: tiled and seam-healed, one whole picture, see-through on
magenta, greyed for the game's biome tint.
"""
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

import numpy as np
from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
MODS = {m: os.path.join(ROOT, "mods", m, "assets", "textures", "block") for m in ("base", "animals")}
GEN, TILES, PREVIEW = (os.path.join(HERE, d) for d in ("gen", "tiles", "preview"))
CODEX = "/Applications/Codex.app/Contents/Resources/codex-cli/bin/codex"
PALETTE = os.path.join(ROOT, "notes", "texture-test", "palette.png")
SIZE = 32

TILE = "Seamless tileable full-block texture, no border, evenly spread detail, no single feature that stands out when repeated."
PLANT = "On the magenta background, standing upright, centred, base touching the bottom edge, readable from its silhouette alone, with a soft 1-px edge 6-8% darker than the inside."
# kind: (type line from the style guide, tiled, see-through, tinted)
KINDS = {
    "cube": (TILE, True, False, False),
    "grass": ("Seamless leafy grass tufts, not noise. " + TILE, True, False, True),
    "grass_side": ("Dirt clods with a grass fringe hanging 4-8 px down from the top edge, irregular tips; the left and right edges tile.", False, False, False),
    "log": ("Vertical bark with deep grooves, tileable top-to-bottom and left-to-right.", True, False, False),
    "end": ("Cross-section with 4-6 growth rings in a lighter wood, a 2-3 px bark rim, bevelled.", False, False, False),
    "built": ("1-px mortar lines in the darkest shade, light top-left edge on each piece, joints line up across tile edges so it tiles.", True, False, False),
    "framed": ("A 1-px bevelled frame around the face, light on the top and left of the frame, shadow on the bottom and right.", False, False, False),
    "face": ("One picture filling the whole face, seen straight on.", False, False, False),
    "leaves": ("Overlapping individual leaves 3-5 px long, with gaps of flat solid magenta covering 25-35% of the tile, tileable.", False, True, True),
    "plant": (PLANT, False, True, False),
    "tplant": (PLANT + " Light green, it is tinted by the biome in the game.", False, True, True),
    "crop": ("A row of stalks rising from the bottom edge that reads well when the texture is placed 4 times in a # shape. " + PLANT, False, True, False),
    "cutout": ("Fills the square as described, with flat solid magenta wherever it is open or see-through.", False, True, False),
    "flat": ("Seen from straight above, lying flat, centred, not touching the edges, on the magenta background, with a soft 1-px edge darker than the inside.", False, True, False),
}
# Biome tints in the game, for previews only (`mods/base/src/tints.rs` picks the real ones).
GRASS, FOLIAGE = np.array([0x7F, 0xB0, 0x48]), np.array([0x6F, 0x9A, 0x3C])


def batch(path):
    rows = []
    for line in open(path if os.path.isabs(path) else os.path.join(HERE, path)):
        if not line.strip() or line.startswith("#"):
            continue
        image, mod, *entries = line.rstrip("\n").split("\t")
        parsed = []
        for e in entries[:4]:
            if e == "-":
                parsed.append(None)
                continue
            name, kind, what = e.split("|", 2)
            name, _, own = name.partition("@")
            parsed.append({"name": name, "kind": kind, "what": what, "mod": own or mod})
        rows.append((image, parsed + [None] * (4 - len(parsed))))
    return rows


def picked(rows, only):
    return [r for r in rows if not only or r[0] in only]


def approved_sheet():
    """`approved.txt` lists approved tiles: they are the style key attached to every request."""
    names = [n.strip() for n in open(os.path.join(HERE, "approved.txt")) if n.strip() and not n.startswith("#")]
    s, cols = 192, 6
    sheet = Image.new("RGB", (cols * s, -(-len(names) // cols) * s), (255, 0, 255))
    for i, n in enumerate(names):
        t = Image.open(os.path.join(TILES, n + ".png")).convert("RGBA").resize((s, s), Image.NEAREST)
        sheet.paste(t, ((i % cols) * s, (i // cols) * s), t)
    out = os.path.join(GEN, "approved.png")
    sheet.save(out)
    return out


def gen(rows):
    style = open(os.path.join(HERE, "style.txt")).read()
    os.makedirs(GEN, exist_ok=True)
    refs = ["-i", PALETTE, "-i", approved_sheet()]

    def one(row):
        image, entries = row
        if os.path.exists(os.path.join(GEN, image + ".png")):
            return f"{image}: exists"
        parts = [f"{e['what']}. {KINDS[e['kind']][0]}" if e else "Leave this square flat solid magenta." for e in entries]
        prompt = style.replace("NAME", image)
        for key, text in zip(("TL", "TR", "BL", "BR"), parts):
            prompt = prompt.replace(key, text, 1)
        with open(os.path.join(GEN, f"run_{image}.log"), "w") as log:
            subprocess.run([CODEX, "exec", "--skip-git-repo-check", "--sandbox", "workspace-write", "--ephemeral", *refs,
                            "-o", f"last_{image}.txt", prompt], cwd=GEN, stdin=subprocess.DEVNULL, stdout=log, stderr=log)
        return f"{image}: {'ok' if os.path.exists(os.path.join(GEN, image + '.png')) else 'FAILED, see gen/run_' + image + '.log'}"

    with ThreadPoolExecutor(4) as pool:
        for r in pool.map(one, rows):
            print(r, flush=True)


def grey(rgb, mask):
    """A tinted texture's light grey: the game multiplies it by the biome colour."""
    lum = rgb @ [0.3, 0.55, 0.15]
    lum = (lum - lum[mask].mean()) * 1.3 + 185
    return np.repeat(np.clip(lum, 0, 255)[..., None], 3, -1)


def heal(a):
    """Opposite edges averaged: no step where the tile meets itself."""
    for axis in (0, 1):
        first, last = np.take(a, 0, axis), np.take(a, -1, axis)
        mean = (first + last) / 2
        if axis == 0:
            a[0], a[-1] = (a[0] + mean) / 2, (a[-1] + mean) / 2
        else:
            a[:, 0], a[:, -1] = (a[:, 0] + mean) / 2, (a[:, -1] + mean) / 2
    return a


def tile(part, kind):
    """One generated square (any size) to a 32 px RGBA tile."""
    _, tiled, cutout, tinted = KINDS[kind]
    r, g, b = part[..., 0], part[..., 1], part[..., 2]
    alpha = (~((r > 170) & (b > 150) & (g < 110))).astype(float) if cutout else np.ones(part.shape[:2])

    def shrink(x):
        return np.asarray(Image.fromarray(x.astype(np.float32), "F").resize((SIZE, SIZE), Image.BOX)).astype(float)

    a = shrink(alpha)
    rgb = np.stack([shrink(part[..., c] * alpha) for c in range(3)], -1) / np.maximum(a, 1e-6)[..., None]
    solid = a >= 0.5
    if tiled and not cutout:
        rgb = heal(rgb)
    if tinted:
        rgb = grey(rgb, solid)
    return np.dstack([np.clip(rgb, 0, 255), solid * 255.0 if cutout else np.full(a.shape, 255.0)])


def save(a, path):
    Image.fromarray(a.round().astype(np.uint8), "RGBA").save(path)


def cut(rows):
    os.makedirs(TILES, exist_ok=True)
    for image, entries in rows:
        src = os.path.join(GEN, image + ".png")
        if not os.path.exists(src):
            print(f"{image}: not generated")
            continue
        im = np.asarray(Image.open(src).convert("RGB")).astype(float)
        h, w = im.shape[:2]
        inset = w // 170
        for i, e in enumerate(entries):
            if not e:
                continue
            x, y = (i % 2) * w // 2, (i // 2) * h // 2
            part = im[y + inset:y + h // 2 - inset, x + inset:x + w // 2 - inset]
            t = tile(part, e["kind"])
            save(t, os.path.join(TILES, e["name"] + ".png"))
            if e["kind"] == "grass_side":
                # The green fringe becomes the tinted overlay; the side keeps it for the item.
                rgb = t[..., :3]
                green = (rgb[..., 1] > rgb[..., 0] + 8) & (rgb[..., 1] > rgb[..., 2] + 8)
                save(np.dstack([grey(rgb, green), green * 255.0]), os.path.join(TILES, "grass_side_overlay.png"))
        print(f"{image}: {' '.join(e['name'] for e in entries if e)}")


def tinted_view(name, kind):
    t = np.asarray(Image.open(os.path.join(TILES, name + ".png")).convert("RGBA")).astype(float)
    if KINDS[kind][3]:
        t[..., :3] *= (GRASS if kind in ("grass", "tplant") else FOLIAGE) / 255
    return Image.fromarray(t.clip(0, 255).astype(np.uint8), "RGBA")


def cube(top, side, a=96):
    """An isometric block: top face, and the two sides shaded as the game shades them."""
    T = 256
    top, side = top.resize((T, T), Image.NEAREST), side.resize((T, T), Image.NEAREST)

    def shade(im, k):
        x = np.asarray(im).astype(float)
        x[..., :3] *= k
        return Image.fromarray(x.clip(0, 255).astype(np.uint8), "RGBA")

    out = Image.new("RGBA", (2 * a, 2 * a))
    for im, c in [
        (shade(side, 0.8), (T / a, 0, 0, -T / (2 * a), T / a, -T / 2)),
        (shade(side, 0.6), (T / a, 0, -T, T / (2 * a), T / a, -1.5 * T)),
        (top, (T / (2 * a), T / a, -T / 2, -T / (2 * a), T / a, T / 2)),
    ]:
        out.alpha_composite(im.transform((2 * a, 2 * a), Image.AFFINE, c, Image.NEAREST, fillcolor=(0, 0, 0, 0)))
    return out


def preview(rows, name):
    """Each tile flat at 6x and as it stands in the world: a cube, or a plant on a grass block."""
    entries = [e for _, es in rows for e in es if e and os.path.exists(os.path.join(TILES, e["name"] + ".png"))]
    a, cell, cols = 96, 2 * 96 + 220, 4
    sheet = Image.new("RGB", (cols * cell + 20, -(-len(entries) // cols) * (2 * a + 70) + 20), (244, 241, 236))
    d = ImageDraw.Draw(sheet)
    grass = os.path.join(TILES, "grass_top.png")
    ground = cube(tinted_view("grass_top", "grass"), Image.open(os.path.join(TILES, "grass_side.png")).convert("RGBA"), a) if os.path.exists(grass) else None
    for i, e in enumerate(entries):
        x, y = 10 + (i % cols) * cell, 10 + (i // cols) * (2 * a + 70)
        t = tinted_view(e["name"], e["kind"])
        flat = Image.new("RGBA", (192, 192), (156, 198, 224, 255) if KINDS[e["kind"]][2] else (0, 0, 0, 0))
        flat.alpha_composite(t.resize((192, 192), Image.NEAREST))
        sheet.paste(flat, (x, y), flat)
        if KINDS[e["kind"]][2] and e["kind"] != "leaves":
            if ground:
                sheet.paste(ground, (x + 200, y + a // 2), ground)
            s = t.resize((int(1.4 * a), int(1.4 * a)), Image.NEAREST)
            sheet.paste(s, (x + 200 + a - s.width // 2, y + a // 2 + a // 2 + a // 2 - s.height), s)
        else:
            # A top (log end, grass top) shows on its block with the block's own sides.
            side = {"grass_top": "grass_side"}.get(e["name"], e["name"][:-4] if e["name"].endswith("_top") else e["name"])
            kind = next((x["kind"] for x in entries if x["name"] == side), e["kind"])
            c = cube(t, tinted_view(side, kind) if os.path.exists(os.path.join(TILES, side + ".png")) else t, a)
            sheet.paste(c, (x + 200, y), c)
        d.text((x, y + 2 * a + 8), f"{e['name']} ({e['kind']})", fill=(70, 60, 50))
    os.makedirs(PREVIEW, exist_ok=True)
    out = os.path.join(PREVIEW, name + ".png")
    sheet.save(out)
    print(out)


def install(rows):
    done = []
    for _, entries in rows:
        for e in entries:
            if not e or e["mod"] == "-":
                continue
            names = [e["name"]] + (["grass_side_overlay"] if e["kind"] == "grass_side" else [])
            for n in names:
                home = next((d for d in MODS.values() if os.path.exists(os.path.join(d, n + ".png"))), MODS[e["mod"]])
                src = Image.open(os.path.join(TILES, n + ".png"))
                (src if KINDS[e["kind"]][2] or n == "grass_side_overlay" else src.convert("RGB")).save(os.path.join(home, n + ".png"))
                # An older texture's random variants and connected rules would show as patches.
                for old in (os.path.join(home, n + ".ctm"),):
                    if os.path.exists(old):
                        os.remove(old)
                folder = os.path.join(home, n)
                if os.path.isdir(folder):
                    for f in os.listdir(folder):
                        os.remove(os.path.join(folder, f))
                    os.rmdir(folder)
                # A lit face keeps its glow mask (`<name>_e.png`): white where the fire is.
                if os.path.exists(os.path.join(home, n + "_e.png")):
                    a = np.asarray(Image.open(os.path.join(home, n + ".png")).convert("RGB")).astype(int)
                    fire = (a[..., 0] > 190) & (a[..., 1] > 90) & (a[..., 0] - a[..., 2] > 90)
                    m = np.where(fire, 255, 0).astype(np.uint8)
                    Image.fromarray(np.dstack([m, m, m, np.full_like(m, 255)]), "RGBA").save(os.path.join(home, n + "_e.png"))
                done.append(n)
    print(f"{len(done)} textures in the game: {' '.join(done)}")


if __name__ == "__main__":
    cmd, path, *only = sys.argv[1:]
    rows = picked(batch(path), only)
    {"gen": gen, "cut": cut, "install": install}.get(cmd, lambda r: preview(r, os.path.basename(path)[:-4]))(rows)
