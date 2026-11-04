"""The page for the style-guide test: every generated asset (as generated, cut to its pixel grid, and
the game's current one beside it), blocks tiled 3 x 3, and one small world scene drawn twice (the
guide blocks with the guide animals, and today's game blocks) so they can be judged together.

    ROUND=1 ~/.venvs/blocky/bin/python notes/ui-codex/guides/gt.py data   # the first run (guides as written)
    ~/.venvs/blocky/bin/python notes/ui-codex/guides/gt.py data           # the second (guides + our lessons)
    ~/.venvs/blocky/bin/python notes/ui-codex/guides/build.py
"""
import base64
import html
import io
import json
import os

import numpy as np
from PIL import Image

import gt

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
GAME_TEX = [os.path.join(ROOT, "mods", m, "assets", "textures") for m in ("base", "animals", "testmod")]
GRASS, FOLIAGE = (0x91, 0xBD, 0x59), (0x77, 0xAB, 0x2F)  # roughly plains, for the grey game textures


def uri(im, scale=1):
    if scale != 1:
        im = im.resize((im.width * scale, im.height * scale), Image.NEAREST)
    b = io.BytesIO()
    im.save(b, "PNG", optimize=True)
    return "data:image/png;base64," + base64.b64encode(b.getvalue()).decode()


def game(kind, name):
    for t in GAME_TEX:
        p = os.path.join(t, kind, name + ".png")
        if os.path.exists(p):
            im = Image.open(p).convert("RGBA")
            if kind == "block":
                im = im.crop((0, 0, im.width, im.width))  # first frame of an animation strip
            return im
    return None


def tint(im, rgb):
    a = np.array(im.convert("RGBA")).astype(float)
    a[..., :3] *= np.array(rgb) / 255
    return Image.fromarray(a.clip(0, 255).astype(np.uint8))


def game_block(name):
    """Today's game block, with the grey ones tinted as the game would."""
    if name == "grass_top":
        return tint(game("block", "grass_top"), GRASS)
    if name == "grass_side":
        base = game("block", "grass_side")
        over = game("block", "grass_side_overlay")
        if over is not None:
            base = Image.alpha_composite(base, tint(over, GRASS))
        return base
    if name == "leaves_oak":
        return tint(game("block", "leaves_oak"), FOLIAGE)
    return game("block", name)


def new_block(name):
    p = os.path.join(HERE, "px2", name + ".png")
    return Image.open(p).convert("RGBA") if os.path.exists(p) else None


# ---------------------------------------------------------------- the world scene

A = 64  # half a block's screen width: one 32-px texel is 2 screen px along a face


def scene(tex, sprites, models=False):
    """A small island: three layers of ground, a tree, a crafting table, flowers and animals, drawn
    isometric with the game's face light (top 100%, left side 80%, right side 60%)."""
    X, Z = 8, 6
    cells = {}
    for x in range(X):
        for z in range(Z):
            cells[(x, 0, z)] = ("stone" if (x * 7 + z * 3) % 9 else "iron_ore",) * 3
            if (x, z) in ((6, 5), (7, 2)):
                cells[(x, 0, z)] = ("cobblestone",) * 3
            cells[(x, 1, z)] = ("dirt",) * 3
            cells[(x, 2, z)] = ("grass_top", "grass_side", "grass_side")
    for y in (3, 4, 5):
        cells[(1, y, 1)] = ("log_top", "log_oak", "log_oak")
    for x in range(0, 3):
        for z in range(0, 3):
            for y in (5, 6):
                if (x, y, z) != (1, 5, 1) and not (y == 6 and (x, z) in ((0, 0), (2, 2), (0, 2), (2, 0))):
                    cells[(x, y, z)] = ("leaves_oak",) * 3
    cells[(5, 3, 1)] = ("planks_oak", "crafting_table_front", "crafting_table_front")
    animals = [((2.6, 3, 3.4), "capybara"), ((2.2, 3, 4.9), "chihuahua"), ((5.0, 3, 4.5), "utahraptor")]
    bills = [((0.5, 3, 4.5), "flower_rose"), ((6.5, 3, 4.5), "flower_rose")] + ([] if models else animals)

    W, H = (X + Z) * A + 2 * A, (X + Z) * A // 2 + 8 * A
    cx, cy = Z * A + A, 7 * A
    img = np.zeros((H, W, 3), float)
    sky = np.linspace([170, 205, 235], [228, 236, 240], H)
    img[:] = sky[:, None, :]

    def P(x, y, z):
        return np.array([cx + (x - z) * A, cy + (x + z) * A / 2 - y * A], float)

    yy, xx = np.mgrid[0:H, 0:W]

    def face(t, O, U, V, shade):
        if t is None:
            return
        tx = np.array(t.convert("RGBA")).astype(float)
        th, tw = tx.shape[:2]
        pts = np.array([O, O + U, O + V, O + U + V])
        x0, y0 = np.floor(pts.min(0)).astype(int)
        x1, y1 = np.ceil(pts.max(0)).astype(int)
        x0, y0, x1, y1 = max(x0, 0), max(y0, 0), min(x1, W), min(y1, H)
        px = xx[y0:y1, x0:x1] + 0.5 - O[0]
        py = yy[y0:y1, x0:x1] + 0.5 - O[1]
        det = U[0] * V[1] - U[1] * V[0]
        u = (px * V[1] - py * V[0]) / det
        v = (py * U[0] - px * U[1]) / det
        m = (u >= 0) & (u < 1) & (v >= 0) & (v < 1)
        ti = np.clip((v * th).astype(int), 0, th - 1)
        tj = np.clip((u * tw).astype(int), 0, tw - 1)
        s = tx[ti, tj]
        m &= s[..., 3] > 127
        reg = img[y0:y1, x0:x1]
        reg[m] = s[..., :3][m] * shade

    def billboard(name, x, y, z):
        sp = sprites.get(name)
        if sp is None:
            return
        foot = P(x, y, z)
        k = 2 if name in gt.ANIMALS else 64 / sp.width  # a plant spans one block face, whatever its size
        w, h = sp.width * k, sp.height * k
        O = np.array([foot[0] - w / 2, foot[1] - h + 6])
        face(sp, O, np.array([w, 0.0]), np.array([0.0, h]), 1.0)

    def model(name, x, y, z):
        """One of our box models at rest, turned to show its face, each face lit as the game lights
        a block's (top 100%, left side 80%, right side 60%, mixed for slanted faces)."""
        import models3d
        d = np.array([1.0, 1.0, 1.0])
        base = np.array([x, y, z])
        fs = [f for f in models3d.faces(name, yaw=180) if f[3] @ d > 1e-9]
        for p, u, v, n, crop in sorted(fs, key=lambda f: (f[0] + f[1] / 2 + f[2] / 2) @ d):
            nn = np.clip(n / np.linalg.norm(n), 0, None)
            shade = (1.0 * nn[1] + 0.8 * nn[2] + 0.6 * nn[0]) / max(nn.sum(), 1e-9)
            O = P(*(base + p))
            face(crop, O, P(*(base + p + u)) - O, P(*(base + p + v)) - O, shade)

    items = [(x + z, y, 0, (x, y, z)) for (x, y, z) in cells] + \
            [(bx + bz - 0.5, by, 1, (bx, by, bz, n)) for (bx, by, bz), n in bills] + \
            ([(bx + bz - 0.5, by, 2, (bx, by, bz, n)) for (bx, by, bz), n in animals] if models else [])
    for _, _, kind, it in sorted(items, key=lambda r: (r[0], r[1])):
        if kind == 1:
            billboard(it[3], *it[:3])
            continue
        if kind == 2:
            model(it[3], *it[:3])
            continue
        x, y, z = it
        top, sx_, sz_ = cells[(x, y, z)]
        T = tex.get
        if top == "log_top":
            top = None  # no log end texture in the test; the leaves cover it
        if (x, y + 1, z) not in cells or cells[(x, y + 1, z)][0] == "leaves_oak":
            face(T(top), P(x, y + 1, z), P(x + 1, y + 1, z) - P(x, y + 1, z), P(x, y + 1, z + 1) - P(x, y + 1, z), 1.0)
        if (x, y, z + 1) not in cells or cells[(x, y, z + 1)][0] == "leaves_oak":
            face(T(sz_), P(x, y + 1, z + 1), P(x + 1, y + 1, z + 1) - P(x, y + 1, z + 1), P(x, y, z + 1) - P(x, y + 1, z + 1), 0.8)
        if (x + 1, y, z) not in cells or cells[(x + 1, y, z)][0] == "leaves_oak":
            face(T(sx_), P(x + 1, y + 1, z + 1), P(x + 1, y + 1, z) - P(x + 1, y + 1, z + 1), P(x + 1, y, z + 1) - P(x + 1, y + 1, z + 1), 0.6)
    return Image.fromarray(img.clip(0, 255).astype(np.uint8))


# ---------------------------------------------------------------- the page

def thumb(path, size=320):
    im = Image.open(path).convert("RGB")
    im.thumbnail((size, size), Image.NEAREST)
    return uri(im)


def main():
    rows = json.load(open(os.path.join(HERE, "gt2.json")))
    r1 = {r["id"]: r for r in json.load(open(os.path.join(HERE, "gt1.json")))}
    names = [r["id"] for r in rows]
    tex = {n: new_block(n) for n in gt.BLOCKS}
    sprites = {n: Image.open(os.path.join(HERE, "px2", n + ".png")).convert("RGBA")
               for n in list(gt.ANIMALS) + ["flower_rose"] if n in names}
    now_tex = {n: game_block(n) for n in gt.BLOCKS}
    now_sprites = {"flower_rose": now_tex["flower_rose"]}
    os.makedirs(os.path.join(HERE, "scenes"), exist_ok=True)
    scene(tex, sprites, models=True).save(os.path.join(HERE, "scenes", "guides.png"))
    scene(tex, sprites).save(os.path.join(HERE, "scenes", "guides_sprites.png"))
    scene(now_tex, now_sprites, models=True).save(os.path.join(HERE, "scenes", "now.png"))

    order = ["Tools", "Weapons", "Armour", "Ingots", "Gems", "Crafting materials", "Food", "Containers",
             "Paper and books", "Small tricky objects", "Soft and organic", "Arrows and charms"]
    by = {r["id"]: r for r in rows}

    def card(r):
        k, n = r["kind"], r["id"]
        raw = os.path.join(gt.OUT, n + ".png")
        sp = Image.open(os.path.join(HERE, "px2", n + ".png")).convert("RGBA")
        p1 = os.path.join(HERE, "px", n + ".png")
        sp1 = Image.open(p1).convert("RGBA") if n in r1 and os.path.exists(p1) else None
        if k == "block" and gt.BLOCKS[n][3]:
            big = Image.new("RGBA", (96, 96))
            for i in range(3):
                for j in range(3):
                    big.paste(sp, (i * 32, j * 32))
            new_html = f'<div class="pic tile"><img src="{uri(big, 3)}" alt=""></div>'
            cur = game_block(n)
            if cur is not None:
                cb = Image.new("RGBA", (cur.width * 3, cur.width * 3))
                for i in range(3):
                    for j in range(3):
                        cb.paste(cur, (i * cur.width, j * cur.width))
                cur_html = f'<div class="pic tile"><img src="{uri(cb, 288 // cb.width)}" alt=""></div>'
            else:
                cur_html = '<div class="pic none">none</div>'
            caps = ("second run, tiled 3 × 3", f"game now, {cur.width if cur else '–'} px")
        else:
            scale = 8 if k != "animal" else 6
            new_html = f'<div class="pic slot"><img src="{uri(sp, scale)}" alt=""></div>'
            cur = game("item", n) if k == "item" else (game("block", n) if k == "block" else None)
            if cur is not None:
                cur_html = f'<div class="pic slot"><img src="{uri(cur, max(1, 256 // cur.width))}" alt=""></div>'
            else:
                cur_html = '<div class="pic none">a 3D model in the game</div>' if k == "animal" else \
                    '<div class="pic none">none</div>'
            caps = (f"second run, {r['w']} × {r['h']}", f"game now, {cur.width} px" if cur else "game now")
        if sp1 is None:
            first_html = '<div class="pic none">not made in the first run</div>'
        elif k == "block" and gt.BLOCKS[n][3]:
            b1 = Image.new("RGBA", (96, 96))
            for i in range(3):
                for j in range(3):
                    b1.paste(sp1, (i * 32, j * 32))
            first_html = f'<div class="pic tile"><img src="{uri(b1, 3)}" alt=""></div>'
        else:
            first_html = f'<div class="pic slot"><img src="{uri(sp1, 8 if k != "animal" else 6)}" alt=""></div>'
        tries = r.get("tries", [])
        made = [t for t in tries if t.get("made", True)]
        verdict = ("passed the checks" + (f" on try {len(tries)}" if len(tries) > 1 else "")) if tries and tries[-1].get("passed") \
            else (f"failed the checks {len(made)} times; the last try is shown" if tries else "")
        last = tries[-1] if tries else {}
        nums = ", ".join(f"{key} {last[key]}" for key in ("grid", "outline", "red_line", "seam") if key in last)
        slots = ""
        if k == "item":
            slots = ('<div class="slots">' + "".join(
                f'<span class="slot"><img src="{uri(sp, s)}" width="{32 * s // 2}" height="{32 * s // 2}" alt=""></span>'
                for s in (2, 3, 4)) + "</div>")
        return f"""<article class="card" id="{n}">
  <header><h3>{html.escape(n.replace('_', ' '))}</h3><span class="tag">{r['colors']} colors</span></header>
  <div class="quad">
    <figure><div class="pic raw{' green' if gt.key_color(n) == 'green' else ''}"><img src="{thumb(raw)}" alt=""></div><figcaption>as generated</figcaption></figure>
    <figure>{new_html}<figcaption>{caps[0]}</figcaption></figure>
    <figure>{first_html}<figcaption>first run</figcaption></figure>
    <figure>{cur_html}<figcaption>{caps[1]}</figcaption></figure>
  </div>{slots}
  <p class="checks">{verdict}{(': ' + nums) if nums else ''}</p>
</article>"""

    sections = []
    hot_new = "".join(f'<span class="slot"><img src="{uri(Image.open(os.path.join(HERE, "px2", r["id"] + ".png")), 2)}" width="32" height="32" alt="{r["id"]}" title="{r["id"]}"></span>'
                      for r in rows if r["kind"] == "item")
    hot_now = "".join(f'<span class="slot"><img src="{uri(game("item", r["id"]).resize((32, 32), Image.NEAREST))}" width="32" height="32" alt="{r["id"]}" title="{r["id"]}"></span>'
                      for r in rows if r["kind"] == "item" and game("item", r["id"]) is not None)
    for g in order:
        rs = [r for r in rows if r["kind"] == "item" and r["group"] == g]
        if rs:
            sections.append(f'<h3 class="kind">{g}</h3><div class="cards">' + "".join(card(r) for r in rs) + "</div>")
    blocks = "".join(card(r) for r in rows if r["kind"] == "block")
    animals = "".join(card(r) for r in rows if r["kind"] == "animal")
    missing = [n for k, n in gt.all_jobs() if n not in by]

    more = more_cards()
    page = open(os.path.join(HERE, "page.html")).read()
    page = page.replace("{{MORE}}", more)
    page = (page.replace("{{ITEMS}}", "\n".join(sections)).replace("{{BLOCKS}}", blocks)
            .replace("{{ANIMALS}}", animals).replace("{{HOT_NEW}}", hot_new).replace("{{HOT_NOW}}", hot_now)
            .replace("{{COUNT}}", str(len(rows)))
            .replace("{{MISSING}}", ("Not made: " + ", ".join(missing) + ".") if missing else ""))
    open(os.path.join(HERE, "index.html"), "w").write(page)
    print("index.html,", len(rows), "assets;", "missing:", missing)


def more_cards():
    """The other five guides (model blocks, monsters and characters, UI, effects, sky), made with the
    game-art skill's art.py into more/: each as generated, its exact sprite, and how the checks went."""
    jobs = json.load(open(os.path.join(HERE, "more_jobs.json")))
    titles = {"model_block": "Model blocks", "monster": "Monsters and characters", "ui": "UI",
              "effect": "Effects", "sky": "Sky"}
    game_now = {"chest": ("block", "chest_front"), "zombie": ("entity", "zombie"), "spider": ("entity", "spider")}
    out = []
    for kind, title in titles.items():
        cards = []
        for j in [j for j in jobs if j["kind"] == kind]:
            n = j["name"]
            raw, px = os.path.join(HERE, "more", n + ".png"), os.path.join(HERE, "more", n + "_px.png")
            if not os.path.exists(px):
                cards.append(f'<article class="card"><header><h3>{n.replace("_", " ")}</h3></header><p class="checks">not made yet</p></article>')
                continue
            sp = Image.open(px).convert("RGBA")
            scale = max(1, 256 // max(sp.size))
            info = json.load(open(os.path.join(HERE, "more", n + "_checks.json")))
            tries = info["tries"]
            last = tries[-1] if tries else {}
            verdict = (("passed the checks" + (f" on try {len(tries)}" if len(tries) > 1 else "")) if last.get("passed")
                       else f"failed the checks {len(tries)} times; the last try is shown")
            if info.get("note"):
                verdict = info["note"]
            nums = ", ".join(f"{k} {last[k]}" for k in ("grid", "outline", "fill") if k in last)
            cards.append(f"""<article class="card">
  <header><h3>{html.escape(n.replace('_', ' '))}</h3><span class="tag">{sp.width} × {sp.height}</span></header>
  <div class="duo">
    <figure><div class="pic raw wide"><img src="{thumb(raw, 360)}" alt=""></div><figcaption>as generated</figcaption></figure>
    <figure><div class="pic slot wide"><img src="{uri(sp, scale)}" alt=""></div><figcaption>cut, {sp.width} × {sp.height}</figcaption></figure>
  </div>
  <p class="what">{html.escape(j['what'])}</p>
  <p class="checks">{verdict}{(': ' + nums) if nums else ''}</p>
</article>""")
        out.append(f'<h3 class="kind">{title}</h3><div class="cards">' + "".join(cards) + "</div>")
    return "\n".join(out)


if __name__ == "__main__":
    main()
