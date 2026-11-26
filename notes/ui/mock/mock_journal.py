"""The Field Journal (Ronald, 11-05: "we need a dossier for our animals ... available from the hub or
somewhere in the inventory"): a mockup in kit 2, drawn into a real 1280 x 720 shot of the game at
GUI scale 2 (one UI pixel a screen pixel), before it is built. His picks: a "Field Journal" tab
in the inventory and its own key; entries unlock when an animal is first seen up close and fill
in as it is tamed, bred or hunted; a turning picture and the basics, then the survival facts.

Two states: the grey wolf found and tamed (drops and breeding still to learn), and an animal not
found yet. The list's pictures are placeholders (the game draws each animal's own model there),
the wolf's portrait is the animal screen's, the facts are the wolf's own (SPECIES.md).

Usage: python3 mock_journal.py WORLD_SHOT.png PORTRAIT_SHOT.png OUT.png"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "kit2"))
import kit2 as k  # noqa: E402

WORLD, PORTRAIT_SHOT, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
REPO = k.REPO
GOLD = k.GOLD_TEXT
LOCKED = (120, 104, 84)

W, H = 760, 400          # the window's panel, UI px (380 x 200 GUI px)
LIST_W = 236             # the list column
TILE = 40                # a list tile (20 GUI px)


def paste(dst, a, x, y):
    im = Image.fromarray(a) if isinstance(a, np.ndarray) else a
    dst.paste(im, (x, y), im if im.mode == "RGBA" else None)


def text(img, s, x, y, color, shadow=(14, 10, 7)):
    """`s` with its capitals' top at `y` on PIL image `img`."""
    a = np.array(img.convert("RGBA"))
    k.F.draw(a, s, x, y, color, shadow)
    img.paste(Image.fromarray(a), (0, 0))
    return k.F.width(s)


def wrap(s, width):
    words, lines, line = s.split(" "), [], ""
    for w in words:
        t = (line + " " + w).strip()
        if k.F.width(t) > width and line:
            lines.append(line)
            line = w
        else:
            line = t
    return lines + [line]


def silhouette(pic):
    """A picture as a dark shape: what isn't found yet."""
    a = np.array(pic.convert("RGBA")).copy()
    ink = a[..., 3] > 0
    a[ink, :3] = (34, 26, 20)
    return Image.fromarray(a)


def tile(selected, pic=None, found=True):
    a = k.slot_r(TILE, selected)
    im = Image.fromarray(a)
    if pic is not None:
        p = pic if found else silhouette(pic)
        p = p.resize((TILE - 8, TILE - 8), Image.NEAREST)
        im.paste(p, (4, 4), p)
    if not found:
        text(im, "?", TILE // 2 - k.F.width("?") // 2, TILE // 2 - k.F.caps // 2, (150, 126, 96))
    return im


def window(state):
    base = Image.open(WORLD).convert("RGB").resize((1280, 720))
    # In a world a screen dims what is behind it.
    base = Image.fromarray((np.array(base) * 0.55).astype(np.uint8))
    img = base.convert("RGBA")
    x0, y0 = (1280 - W) // 2, 118
    # The window's tabs on its frame: the inventory's, the journal's open.
    tx = x0 + 12
    for label, active in (("Inventory", False), ("Stats", False), ("Advancements", False), ("Field Journal", True)):
        w = k.F.width(label) + 16
        paste(img, k.tab_r(w, label, active), tx, y0 - 22 + 3)
        tx += w + 4
    paste(img, k.panel_r(W, H), x0, y0)
    px, py = x0 + 14, y0 + 14

    # The list: found and how many, then the animals.
    text(img, "Animals", px, py + 2, GOLD)
    text(img, "6 of 42 found", px + 150, py + 2, k.DIM)
    portrait = Image.open(PORTRAIT_SHOT).convert("RGBA").crop((322, 165, 446, 296))
    bg = np.array(portrait)
    # The portrait's backdrop out: the wolf alone, for the list's picture.
    wolf = bg.copy()
    back = np.all(np.abs(wolf[..., :3].astype(int) - np.array([41, 31, 24])) < 18, axis=-1)
    wolf[back, 3] = 0
    wolf_pic = Image.fromarray(wolf).crop((8, 0, 94, 131))
    icons = {n: Image.open(os.path.join(REPO, "mods/animals/assets/textures/item", n + ".png")).convert("RGBA")
             for n in ("hare", "trout", "venison", "pacu")}
    tiles = [("wolf", True), ("hare", True), ("trout", True), ("venison", True), ("pacu", True), ("wolf", True)]
    tiles += [("wolf", False)] * 14
    cols = 5
    for i, (name, found) in enumerate(tiles):
        pic = wolf_pic if name == "wolf" else icons[name]
        sel = (state == "found" and i == 0) or (state == "unknown" and i == 7)
        t = tile(sel, pic, found)
        paste(img, t, px + (i % cols) * (TILE + 4), py + 22 + (i // cols) * (TILE + 4))
    text(img, "Placeholder pictures: the game", px, py + 22 + 4 * (TILE + 4) + 6, (110, 94, 76))
    text(img, "draws each animal's own model.", px, py + 22 + 4 * (TILE + 4) + 22, (110, 94, 76))

    # The page.
    qx, qy = x0 + LIST_W + 24, py
    page_w = W - LIST_W - 38
    if state == "found":
        well = k.canvas(200, 150)
        paste(img, Image.fromarray(np.array(Image.fromarray(k.slot_r(150, False)).resize((200, 150), Image.NEAREST))), qx, qy)
        paste(img, portrait, qx + 38, qy + 9)
        text(img, "drag to turn", qx + 100 - k.F.width("drag to turn") // 2, qy + 156, (130, 112, 90))
        rx = qx + 214
        text(img, "Grey Wolf", rx, qy + 4, GOLD)
        text(img, "Canis lupus", rx, qy + 24, k.DIM)
        cy = qy + 44
        for chips in (("Medium", "0.8 m · 42 kg"), ("By day", "In packs"), ("Defends itself",)):
            cx = rx
            for c in chips:
                a = k.chip_r(c, "plain")
                paste(img, a, cx, cy)
                cx += a.shape[1] + 4
            cy += 24
        text(img, "Lives in", rx, cy + 4, GOLD)
        for i, line in enumerate(wrap("taiga, forests, mountains, snowy plains, sequoia groves", page_w - 214)):
            text(img, line, rx, cy + 22 + i * 17, k.CREAM)
        # The survival facts, what's not learned yet in dim with how to learn it.
        sy = qy + 206
        paste(img, k.divider(page_w), qx, sy - 8)
        rows = [
            ("Eats", "meat and carrion; mutton is its favourite", True),
            ("Taming", "passive: feed it 10 times while it eats", True),
            ("Riding", "can't be ridden", True),
            ("Drops", "hunt one to learn what it drops", False),
            ("Breeding", "breed a pair to learn about its young", False),
        ]
        for label, s, known in rows:
            text(img, label, qx, sy + 2, GOLD if known else LOCKED)
            for i, line in enumerate(wrap(s, page_w - 96)):
                text(img, line, qx + 96, sy + 2 + i * 17, k.CREAM if known else LOCKED)
            sy += 17 * len(wrap(s, page_w - 96)) + 9
    else:
        paste(img, Image.fromarray(np.array(Image.fromarray(k.slot_r(150, False)).resize((200, 150), Image.NEAREST))), qx, qy)
        sil = silhouette(portrait)
        sil_a = np.array(sil)
        back = np.all(np.abs(np.array(portrait)[..., :3].astype(int) - np.array([41, 31, 24])) < 18, axis=-1)
        sil_a[back, 3] = 0
        paste(img, Image.fromarray(sil_a), qx + 38, qy + 9)
        rx = qx + 214
        text(img, "???", rx, qy + 4, GOLD)
        for i, line in enumerate(wrap("Not found yet. See one up close to start its page.", page_w - 214)):
            text(img, line, rx, qy + 26 + i * 17, k.DIM)
        sy = qy + 206
        paste(img, k.divider(page_w), qx, sy - 8)
        for label in ("Eats", "Taming", "Riding", "Drops", "Breeding"):
            text(img, label, qx, sy + 2, LOCKED)
            text(img, "?", qx + 96, sy + 2, LOCKED)
            sy += 26
    return img.convert("RGB")


a, b = window("found"), window("unknown")
sheet = Image.new("RGB", (1280, 720 * 2 + 40), (24, 20, 18))
sheet.paste(a, (0, 0))
sheet.paste(b, (0, 760))
s = np.array(sheet.convert("RGBA"))
k.F.draw(s, "Field Journal, a mockup: found and tamed (above), not found yet (below)", 16, 728, k.GOLD_TEXT, (14, 10, 7))
Image.fromarray(s).convert("RGB").save(OUT)
print(OUT)
