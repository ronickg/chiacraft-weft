"""Make one piece of game art from the style guides: the guide's master prompt, its material colors
and the matching reference image, drawn by ChatGPT (the Codex command line) on the exact 32-pixel
grid, then checked (and made again when a check fails), cut to its exact pixels, and optionally put
in the game. The guides and their reference images are in ../guides/ (`0 Making it.md` first).

    art.py item NAME --what "a red apple: round glossy ... centred"
    art.py block NAME --type "natural cube" --what "large sage-grey slabs ..."
    art.py block NAME --type flower --cutout --what "a red poppy ..."
    art.py animal NAME --type "small mammal" --what "a chihuahua ..." [--size 48x32]

Attached to every run: our own style key for the kind (`Our key.png` in the guide's folder, approved
art of ours), the game's UI sheet, and the grid. Never the guides' reference images: those are cut
from another texture pack, for reading only.
    art.py refs                       # every reference image and type line, by kind
    art.py prompt item NAME ...       # print the prompt instead of running it
    art.py check item NAME image.png  # the checks on an image already made
    art.py install item NAME --mod base [--dir OUT]   # the cut sprite over the game's texture

    art.py batch jobs.json            # many at once, four at a time: a list of {kind, name, what, ref, ...}

Every image is also kept for good in the archive (ART_ARCHIVE, default
/Users/ronald/Coding/chiacraft-archive/images/chiacraft-art/<date>/): each try, each cut sprite, each
game texture an install replaces. Nothing is ever deleted from it.

Options: --dir (default run/art/, kept out of git), --green (purple or pink things, so the background
can be keyed), --pale (pale things whose outline may be light), --tries N (default 3). Output in --dir: NAME.png (as generated), NAME_px.png (the exact sprite),
NAME_checks.json (every try's numbers), failed tries as NAME__tryK.png.
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
GUIDES = os.path.join(HERE, "..", "guides")
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", "..", ".."))
UI_DIR = os.path.join(ROOT, "mods", "base", "source", "ui")
CODEX = "/Applications/Codex.app/Contents/Resources/codex-cli/bin/codex"
SCRATCH = os.environ.get("ART_DIR", os.path.join(ROOT, "run", "art"))
# Every image ever generated is kept here, outside the repo, and nothing is deleted from it (Ronald,
# 2026-11-04: "never throw away generated images pls so I can also check back"): each try as made, the
# cut sprite, and every game texture an install replaces.
ARCHIVE = os.environ.get("ART_ARCHIVE", "/Users/ronald/Coding/chiacraft-archive/images/chiacraft-art")


def archive(path, label, sub=""):
    """Copy `path` into today's archive folder as <time>__<label><ext>; never overwrites, never deletes."""
    import datetime
    if not os.path.exists(path):
        return None
    now = datetime.datetime.now()
    folder = os.path.join(ARCHIVE, now.strftime("%Y-%m-%d"), sub)
    os.makedirs(folder, exist_ok=True)
    ext = os.path.splitext(path)[1]
    dst = os.path.join(folder, f"{now.strftime('%H%M%S')}__{label}{ext}")
    k = 1
    while os.path.exists(dst):
        dst = os.path.join(folder, f"{now.strftime('%H%M%S')}__{label}_{k}{ext}")
        k += 1
    shutil.copy2(path, dst)
    return dst

KINDS = {
    "item": ("1 Items/Item Art Style Guide.md", "1 Items/Item Prompt References"),
    "block": ("2 Blocks/Block Art Style Guide.md", "2 Blocks/Block Prompt References"),
    "animal": ("4 Animals/Animal Sprite Style Guide.md", "4 Animals/Animal Prompt References"),
    "model_block": ("3 Model Blocks/Model Block Style Guide.md", "3 Model Blocks/Model Block Prompt References"),
    "monster": ("5 Monsters and Characters/Monster and Character Sprite Style Guide.md",
                "5 Monsters and Characters/Monster Prompt References"),
    "ui": ("6 UI/UI Style Guide.md", "6 UI/UI Prompt References"),
    "effect": ("7 Effects/Effects Style Guide.md", "7 Effects/Effect Prompt References"),
    "sky": ("8 Sky/Sky Style Guide.md", "8 Sky/Sky Prompt References"),
}
# what each kind is called in the prompt, and whose style key it borrows until it has its own
LABEL = {"item": "item icon", "block": "block texture", "animal": "animal sprite", "model_block": "model block",
         "monster": "character sprite", "ui": "UI element", "effect": "effect", "sky": "sky element"}
KEY_FROM = {"model_block": "block", "monster": "animal", "effect": "item", "sky": "item"}
UI_SHEETS = os.path.join(GUIDES, "6 UI", "UI Prompt References")
UI_SUITE = os.path.join(UI_SHEETS, "Voxel Fantasy Survival UI Suite.png")  # Ronald's target for every UI part
# Ronald's own UI sheets (his, not a pack's), by the UI guide's type
UI_KEY = {"slot": "Pixel-Fantasy Inventory UI Reference.png", "frame": "Pixel-Fantasy Inventory UI Reference.png",
          "button": "Fantasy UI Controls Reference Sheet.png", "tab": "Fantasy UI Controls Reference Sheet.png",
          "control": "Fantasy UI Controls Reference Sheet.png", "bar": "Pixel-Art Survival HUD Reference Panel.png",
          "hud": "Pixel-Art Survival HUD Reference Panel.png", "status": "Pixel-Art Survival HUD Reference Panel.png",
          "window": "Pixel-Fantasy Inventory UI Reference.png", "tooltip": "Pixel-Fantasy Inventory UI Reference.png"}
# image sizes the image model makes, and so the sprite sizes that fit them at a whole number of image
# pixels per sprite pixel
SIZES = [(1024, 1024), (1536, 1024), (1024, 1536)]


def cell_for(w, h):
    for W, H in SIZES:
        if W % w == 0 and H % h == 0 and W // w == H // h:
            return W // w
    sys.exit(f"{w} x {h} doesn't fit a 1024 x 1024, 1536 x 1024 or 1024 x 1536 image evenly: try 16x16, 32x32, "
             "48x32, 32x48, 64x64, 96x64 or 64x96")
UI_REF = os.path.join(GUIDES, "6 UI", "UI Prompt References", "Crafting & Workstations UI Reference.png")


# ---------------------------------------------------------------- reading the guides

def guide(kind):
    return open(os.path.join(GUIDES, KINDS[kind][0])).read()


def section(text, start, end):
    t = text[text.index(start):text.index(end, text.index(start))]
    lines = [l[2:] if l.startswith("> ") else ("" if l == ">" else l) for l in t.split("\n")]
    return "\n".join(l for l in lines if not l.startswith("How to prompt, check and cut it")).strip()


def style(kind):
    g = guide(kind)
    if kind == "item":
        a = section(g, "**Style:**", "### Reference images to attach")
        sheet = section(g, "## B) Material sheet", "### Example item requests").split("```")[1].strip()
        return a + "\n\nMaterial colors (outline colors are lit side / shadow side):\n" + sheet
    if kind == "block":
        return (section(g, "> **Style:**", "Add **one** of these") + "\n\nMeasured color ramps:\n" +
                section(g, "| Material | Ramp", "## C)"))
    return section(g, "> **Style:**", "Add **one** of these")


def type_lines(kind):
    """The guide's "add one of these" table: {type name: line}."""
    g = guide(kind)
    out = {}
    for row in g.split("\n"):
        m = re.match(r"\|\s*\*\*(.+?)\*\*(.*?)\|\s*(.+?)\s*\|\s*$", row)
        if m and kind != "item" and '"' in m.group(3):
            out[m.group(1).strip().lower()] = m.group(3).strip()
    return out


def type_line(kind, name):
    lines = type_lines(kind)
    hits = [k for k in lines if name.lower() in k]
    if not hits:
        sys.exit(f"no {kind} type like {name!r}; the guide has: {', '.join(lines)}")
    line = lines[hits[0]].replace('"', "").replace("Top: ", "").strip()
    return re.sub(r"\s*For [a-z]+: [^.]*\.", "", line)  # drop asides about other animals ("For sheep: ...")


def key_path(kind):
    """Our own style key for a kind (approved art of ours), or None. Never the guides' pack sheets."""
    p = os.path.join(GUIDES, os.path.dirname(KINDS[kind][1]), "Our key.png")
    if os.path.exists(p):
        return p
    if kind in KEY_FROM:
        return key_path(KEY_FROM[kind])
    print(f"warning: no 'Our key.png' for {kind}s yet: generating without a style key "
          "(make one from an approved set; never attach the guide's pack sheets)", file=sys.stderr)
    return None


def ref_path(kind, name):
    d = os.path.join(GUIDES, KINDS[kind][1])
    if os.path.exists(os.path.join(d, name)):
        return os.path.join(d, name)
    hits = [f for f in sorted(os.listdir(d)) if name.lower() in f.lower()]
    if not hits:
        sys.exit(f"no reference like {name!r} in {d}: {', '.join(sorted(os.listdir(d)))}")
    return os.path.join(d, hits[0])


# ---------------------------------------------------------------- the prompt

GRID = ("Draw it on an exact {w} x {h} pixel grid: the image is {W} x {H}, so every sprite pixel is exactly one "
        "{c} x {c} block of a single flat color, all blocks aligned to a grid starting at the top-left corner of "
        "the image. No anti-aliasing, no blur, no soft edges, no gradients inside a block, no half blocks. The "
        "last attached image is that grid (cells on magenta): put every pixel exactly in one of its cells, but "
        "do not draw the grid lines.")
TAIL = ("Use your image generation tool to make ONE image, then save it as a PNG file named {name}.png in the "
        "current folder. Do not write any code or edit any other files. When done, reply with the file name.")


def grid_image(w, h):
    from PIL import Image, ImageDraw
    c = cell_for(w, h)
    path = os.path.join(SCRATCH, f"grid_{w}x{h}.png")
    if not os.path.exists(path):
        os.makedirs(SCRATCH, exist_ok=True)
        im = Image.new("RGB", (w * c, h * c), (255, 0, 255))
        d = ImageDraw.Draw(im)
        for k in range(0, w * c + 1, c):
            d.line([(k, 0), (k, h * c - 1)], fill=(200, 0, 200))
        for k in range(0, h * c + 1, c):
            d.line([(0, k), (w * c - 1, k)], fill=(200, 0, 200))
        im.save(path)
    return path


def build(a):
    """(prompt, reference images, (w, h), full tile?) for one request."""
    bg = "solid flat #00FF00 green" if a.green else "solid flat #FF00FF magenta"
    key = key_path(a.kind) if a.kind != "ui" else None
    if a.kind == "item":
        w, h = 32, 32
        t = (f"Make one item icon for my block game.\n\n{style('item')}\n\n{GRID.format(w=w, h=h, W=1024, H=1024, c=32)} "
             f"The item sits inside the 32 x 32 grid; everything outside it is {bg} (it will be cut out). Use about "
             "16-32 colors.\n\nAttached: first, approved items of my game, the style key: take their size, outline, "
             "light and level of detail, not their designs. Second, the game's UI: its colors and the dark brown slot "
             f"(#352015) the icon must read on.\n\nThe item: {a.what}.\n\n")
        refs, full = [key, UI_REF], False
    elif a.kind == "block":
        w, h = 32, 32
        full = not a.cutout
        fill = ("Every one of the 32 x 32 cells is filled: the texture covers the whole image edge to edge and tiles "
                "seamlessly." if full else
                f"Where there is nothing (gaps, around the plant) the image is {bg}, which stands for transparent.")
        t = (f"Make one block texture for my block game.\n\n{style('block')}\n\n{type_line('block', a.type)}\n\n"
             f"{GRID.format(w=w, h=h, W=1024, H=1024, c=32)} {fill}\n\nAttached: first, approved block textures of my "
             "game, the style key: take their look, not their designs. Second, the game's UI, for its colors.\n\n"
             f"The block: {a.what}.\n\n")
        refs = [key, UI_REF]
    elif a.kind == "animal":
        w, h = map(int, a.size.lower().split("x"))
        W, H = w * 32, h * 32
        t = (f"Make one animal sprite for my block game.\n\n"
             f"{style('animal').replace('[animal]', a.name.replace('_', ' ')).replace('[W×H]', f'{w} x {h}')}\n\n"
             f"{type_line('animal', a.type)}\n\n{GRID.format(w=w, h=h, W=W, H=H, c=32)} Make the image {W} x {H}. The "
             f"animal stands on the bottom row of the grid; everything around it is {bg} (it will be cut out).\n\n"
             "Attached: first, approved animal sprites of my game, the style key (take their look, not their "
             "animals). Second, approved item icons, for the outline and light. Third, the game's UI, for its "
             f"colors.\n\nThe animal: {a.what}.\n\n")
        refs, full = [key, key_path("item"), UI_REF], False
    else:  # model blocks, monsters and characters, UI, effects, sky: the guide's master prompt and type line
        w, h = map(int, a.size.lower().split("x"))
        c = cell_for(w, h)
        W, H = w * c, h * c
        label = LABEL[a.kind]
        st = (style(a.kind).replace("[W×H]", f"{w} x {h}").replace("[object]", a.name.replace("_", " "))
              .replace("[character]", a.name.replace("_", " ")))
        full = a.tile
        place = ("Every cell is filled: it covers the whole image edge to edge and tiles seamlessly." if full else
                 f"Everything around it is {bg}, which stands for transparent (it will be cut out); every other pixel "
                 "is fully opaque.")
        if a.kind == "ui":
            sheet = next((v for k, v in UI_KEY.items() if k in a.type.lower()), "Fantasy Pixel UI Reference Poster.png")
            key = os.path.join(UI_SHEETS, sheet)
            if getattr(a, "guide_only", False):  # Ronald's A/B (11-04): the guide's words and our own art only
                refs = [os.path.join(ROOT, "notes", "ui-codex", "approved_parts.png"), key_path("item")]
                attached = ("Attached: first, UI parts already approved for my game; second, approved item icons "
                            "that sit in its slots: together the style key. Take their frames, colors and light.")
            else:
                refs = [UI_SUITE, key, UI_REF]
                attached = ("Attached: first, my game's whole UI as I want it (menus, HUD, inventory, components), "
                            "then two more of my own UI sheets: together the style key. Take their frames, colors "
                            "and light, not their layout.")
        else:
            refs = [key, key_path("item") if a.kind != "item" else None, UI_REF]
            attached = ("Attached: first, approved art of my game, the style key: take its look, not its designs. "
                        "Second, approved item icons, for the outline and light. Third, the game's UI, for its colors.")
        t = (f"Make one {label} for my block game.\n\n{st}\n\n{type_line(a.kind, a.type)}\n\n"
             f"{GRID.format(w=w, h=h, W=W, H=H, c=c)} Make the image {W} x {H}. {place}\n\n{attached}\n\n"
             f"The {label}: {a.what}. Where the text above offers a choice in [brackets], take it from this "
             "description.\n\n")
    if not key:  # no style key of ours for this kind yet: say so rather than describe a missing image
        t = t.replace(", the style key:", " (none attached yet, so follow the words):")
    refs = list(dict.fromkeys(r for r in refs if r))  # each image once
    return t + TAIL.format(name=a.name), refs + [grid_image(w, h)], (w, h), full


# ---------------------------------------------------------------- cut and check

def corner_green(path):
    """Whether the image's background is green rather than magenta: the model picks green for pink and
    purple things by itself (the guides tell it to), so the cutter looks instead of assuming."""
    import numpy as np
    from PIL import Image
    a = np.array(Image.open(path).convert("RGB")).astype(int)
    c = np.concatenate([a[:16, :16], a[:16, -16:], a[-16:, :16], a[-16:, -16:]]).reshape(-1, 3)
    green = ((c[:, 1] > 190) & (c[:, 0] < 90) & (c[:, 2] < 90)).mean()
    magenta = ((c[:, 0] > 190) & (c[:, 1] < 90) & (c[:, 2] > 190)).mean()
    return bool(green > magenta)


def is_key(rgb, green):
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    return ((g > 190) & (r < 90) & (b < 90)) if green else ((r > 190) & (g < 90) & (b > 190))


def snap(path, w, h, full, green=False):
    """The exact w x h sprite: the 32-px grid's phase from where color edges fall, each cell the most
    common color in its middle, transparent where that is the background color."""
    import numpy as np
    from PIL import Image
    src = np.array(Image.open(path).convert("RGB").resize((w * 32, h * 32), Image.NEAREST)).astype(float)
    green = corner_green(path) if not full else green
    phase = []
    for ax in (1, 0):
        g = np.abs(np.diff(src, axis=ax)).sum(-1).sum(1 - ax)
        fold = np.bincount(np.arange(len(g)) % 32, weights=g, minlength=32)
        p = (int(np.argmax(fold)) + 1) % 32
        phase.append(p if p < 16 else p - 32)
    px, py = phase
    out = np.zeros((h, w, 4), np.uint8)
    H, W = src.shape[:2]
    for j in range(h):
        for i in range(w):
            y0, x0 = py + j * 32 + 8, px + i * 32 + 8
            cell = src[max(0, y0):min(H, y0 + 16), max(0, x0):min(W, x0 + 16)].reshape(-1, 3)
            if not len(cell):
                continue
            key = is_key(cell, green)
            if not full and key.mean() > 0.5:
                continue
            c = cell[~key] if (~key).any() else cell
            cols, counts = np.unique((c // 4).astype(int), axis=0, return_counts=True)
            out[j, i, :3] = cols[counts.argmax()] * 4 + 2
            out[j, i, 3] = 255
    return out


def check(path, w, h, full, green=False, pale=False, kind="item"):
    """`0 Making it.md`'s checks: chunky pixels on one grid; for sprites an outline darker than the
    inside (unless the thing is pale itself), not the red-brown line, a sensible fill; for full tiles
    no empty cell and no seam. Returns (passed, numbers)."""
    import numpy as np
    from PIL import Image
    sys.path.insert(0, UI_DIR)
    import check_items
    import outline
    rgb = np.array(Image.open(path).convert("RGB").resize((w * 32, h * 32), Image.NEAREST)).astype(float)
    green = corner_green(path) if not full else green
    keep = np.ones(rgb.shape[:2], bool) if full else ~is_key(rgb, green)
    res = {"grid": round(check_items.grid_strength(rgb, keep)[0], 3)}
    ok = res["grid"] >= 0.12
    sprite = snap(path, w, h, full, green)
    if full:
        a = sprite[..., :3].astype(float)
        inner = (np.abs(np.diff(a, axis=1)).mean() + np.abs(np.diff(a, axis=0)).mean()) / 2
        seam = (np.abs(a[:, 0] - a[:, -1]).mean() + np.abs(a[0] - a[-1]).mean()) / 2
        res["empty"] = int((sprite[..., 3] == 0).sum())
        res["seam"] = round(seam / max(inner, 1e-9), 2)
        return bool(ok and res["empty"] == 0 and res["seam"] < 2.2), res
    m = sprite[..., 3] > 0
    if not m.any():
        return False, res
    lum = sprite[..., :3].astype(float).mean(-1)
    pad = np.pad(m, 1)
    edge = m & ~(pad[2:, 1:-1] & pad[:-2, 1:-1] & pad[1:-1, 2:] & pad[1:-1, :-2])
    inside = m & ~edge
    res["outline"] = round(float(np.median(lum[edge]) / max(np.median(lum[inside]), 1)), 2) if inside.any() else 1.0
    res["red_line"] = round(outline.share(sprite[..., :3].astype(float), m), 2)
    res["fill"] = round(float(m.mean()), 2)
    if kind in ("ui", "effect", "sky"):  # glows, frames and particles: no outline rule, any fill
        return bool(ok and res["fill"] > 0.01), res
    ok &= (res["outline"] < 0.9 or pale) and res["red_line"] < 0.45 and 0.04 < res["fill"] < 0.85
    return bool(ok), res


# ---------------------------------------------------------------- running it

def make(a):
    os.makedirs(a.dir, exist_ok=True)
    out = os.path.join(a.dir, a.name + ".png")
    text, refs, (w, h), full = build(a)
    tries = []
    for k in range(a.tries):
        if os.path.exists(out):  # an earlier image under this name: keep it in the archive, then make room
            archive(out, f"{a.name}__earlier", "generated")
            os.remove(out)
        args = [CODEX, "exec", "--skip-git-repo-check", "--sandbox", "workspace-write", "--ephemeral"]
        for r in refs:
            args += ["-i", r]
        args += ["-o", os.path.join(a.dir, f"{a.name}_last.txt"), text]
        with open(os.path.join(a.dir, f"{a.name}_run.log"), "a") as log:
            subprocess.run(args, cwd=a.dir, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT)
        if not os.path.exists(out):
            tries.append({"made": False})
            continue
        archive(out, f"{a.name}__try{k + 1}", "generated")  # every try, passed or not
        ok, res = check(out, w, h, full, a.green, a.pale, a.kind)
        tries.append(dict(res, passed=ok))
        if ok:
            break
        shutil.copy(out, os.path.join(a.dir, f"{a.name}__try{k + 1}.png"))
    json.dump({"kind": a.kind, "what": a.what, "size": [w, h], "full": full, "green": a.green, "tries": tries},
              open(os.path.join(a.dir, a.name + "_checks.json"), "w"), indent=1)
    if not os.path.exists(out):
        made = [k for k in range(a.tries, 0, -1) if os.path.exists(os.path.join(a.dir, f"{a.name}__try{k}.png"))]
        if not made:
            return f"{a.name}: nothing made ({len(tries)} tries)"
        shutil.copy(os.path.join(a.dir, f"{a.name}__try{made[0]}.png"), out)
    from PIL import Image
    pxp = os.path.join(a.dir, a.name + "_px.png")
    Image.fromarray(snap(out, w, h, full, a.green)).save(pxp)
    archive(pxp, f"{a.name}__px", "cut")
    passed = tries and tries[-1].get("passed")
    return f"{a.name}: {'passed' if passed else 'FAILED the checks'} after {len(tries)} tr{'y' if len(tries) == 1 else 'ies'} {tries[-1] if tries else ''}"


def install(a):
    src = os.path.join(a.dir, a.name + "_px.png")
    folder = {"item": "item", "block": "block"}.get(a.kind)
    if not folder:
        sys.exit("animals are box models: give the coat to blocky-model instead of installing a sprite")
    dst = os.path.join(ROOT, "mods", a.mod, "assets", "textures", folder, a.name + ".png")
    if os.path.exists(dst):
        shutil.copy(dst, os.path.join(a.dir, a.name + "_was.png"))
        archive(dst, f"{a.name}__replaced_{a.mod}", "replaced")  # the old game texture, kept for good
    shutil.copy(src, dst)
    print(f"{src} -> {dst}")


def main():
    if len(sys.argv) > 2 and sys.argv[1] == "batch":
        jobs = json.load(open(sys.argv[2]))
        base = dict(what="", ref="", type="", guide_only=False, cutout=False, size="48x32", tile=False, green=False, pale=False, tries=3,
                    dir=SCRATCH, mod="base")
        with ThreadPoolExecutor(4) as ex:
            for r in ex.map(lambda j: make(argparse.Namespace(**dict(base, **j))), jobs):
                print(r, flush=True)
        return
    if len(sys.argv) > 1 and sys.argv[1] == "refs":
        for kind, (_, d) in KINDS.items():
            folder = os.path.join(GUIDES, d)
            print(f"== {kind}: reading sheets in guides/{d} (never attached; not in git)")
            for f in sorted(os.listdir(folder)) if os.path.isdir(folder) else ["(not here)"]:
                print("  ", f)
            for k, line in type_lines(kind).items():
                print(f"   type {k!r}: {line[:90]}")
        return
    cmd = None
    if len(sys.argv) > 1 and sys.argv[1] in ("prompt", "check", "install"):
        cmd = sys.argv.pop(1)
    ap = argparse.ArgumentParser()
    ap.add_argument("kind", choices=list(KINDS))
    ap.add_argument("name")
    ap.add_argument("image", nargs="?", help="check: the image to check")
    ap.add_argument("--what", default="", help="what to draw: material and colors, then shape and view")
    ap.add_argument("--ref", default="", help="unused for generating (the guides' pack sheets are never attached)")
    ap.add_argument("--type", default="", help="blocks and animals: the guide's type line (part of its name)")
    ap.add_argument("--cutout", action="store_true", help="blocks: see-through (plants, leaves)")
    ap.add_argument("--size", default="48x32", help="animals and the other kinds: the canvas in pixels (16x16, 32x32, 48x32, 64x64...)")
    ap.add_argument("--tile", action="store_true", help="the other kinds: a full tile with no background (clouds, star field)")
    ap.add_argument("--green", action="store_true", help="on green: the thing is purple, pink or magenta")
    ap.add_argument("--pale", action="store_true", help="a pale thing whose outline may be light (paper, feather)")
    ap.add_argument("--guide-only", action="store_true", help="ui: no UI sheets of Ronald's attached, only the guide's words and our approved parts")
    ap.add_argument("--tries", type=int, default=3)
    ap.add_argument("--dir", default=SCRATCH)
    ap.add_argument("--mod", default="base", help="install: the mod whose texture it replaces")
    a = ap.parse_args()
    if cmd == "install":
        return install(a)
    if cmd == "check":
        cj = os.path.join(a.dir, a.name + "_checks.json")
        info = json.load(open(cj)) if os.path.exists(cj) else None
        w, h = info["size"] if info else build(a)[2]
        full = info["full"] if info else build(a)[3]
        print(check(a.image, w, h, full, a.green, a.pale))
        return
    if cmd == "prompt":
        text, refs, _, _ = build(a)
        print(text, "\n\nReferences:", *refs, sep="\n")
        return
    if not a.what or (a.kind != "item" and not a.type):
        sys.exit("say --what (and --type for every kind but items); `art.py refs` lists the types")
    print(make(a))


if __name__ == "__main__":
    main()
