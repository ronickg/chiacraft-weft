"""The block tiles' grade, built from untouched sources (Ronald, 11-04: "something we can tune easily without
having to ... redo every asset").

    python3 notes/blocks/regrade.py --all [--with grade_proposed.toml] [--preview out.png] [--packs DIR] [--dry]

reads mods/base/source/blocks/grade.toml (groups and per-tile rows; the knobs are explained there), and
rebuilds every graded tile of base and animals, CTM variants included, from its source in
mods/<mod>/source/blocks/tiles/. It prints each tile's luma, saturation and p10-p90 spread, before and
after, and the vanilla band when the packs are at --packs (see game-art's guides/2 Blocks/Full set.md, "Close to vanilla's
colour"), and with --preview draws a tiled 3x3 before / after of every tile the grade changes.

Sources: a tile a group or row grades for the first time is adopted from the game's current tile. A
game tile that changed since the last build (grade.lock.json) was replaced by someone, so it becomes the
new source and is graded on top, never overwritten.

The old single-file form still works:
    python3 notes/blocks/regrade.py <in.png> <out.png> <gamma> <sat> [r,g,b pull] [amount]
"""
import argparse
import colorsys
import fnmatch
import glob
import hashlib
import json
import os
import shutil
import sys
import tomllib

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
MODS = ("base", "animals")
TABLE = os.path.join(ROOT, "mods", "base", "source", "blocks", "grade.toml")
LOCK = os.path.join(ROOT, "mods", "base", "source", "blocks", "grade.lock.json")
W = np.array([0.2126, 0.7152, 0.0722])
NEUTRAL = dict(gamma=1.0, gain=1.0, contrast=1.0, spread=None, saturation=1.0, hue=0.0, pull=None, pull_amount=0.0, outline=1.0)
# What the grade covers: block tiles and item icons, each with its sources and its table sections.
KINDS = {"block": dict(game="block", src=("blocks", "tiles"), groups="groups", tiles="tiles"),
         "item": dict(game="item", src=("items", "tiles"), groups="item_groups", tiles="items")}


def src_dir(mod, kind="block"):
    return os.path.join(ROOT, "mods", mod, "source", *KINDS[kind]["src"])


def game_dir(mod, kind="block"):
    return os.path.join(ROOT, "mods", mod, "assets", "textures", KINDS[kind]["game"])


def sha(p):
    return hashlib.sha1(open(p, "rb").read()).hexdigest()


# --- the knobs --------------------------------------------------------------------------------------

def rgb_to_hsv(rgb):
    mx, mn = rgb.max(-1), rgb.min(-1)
    d = mx - mn
    h = np.zeros_like(mx)
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    m = d > 1e-9
    rm, gm, bm = m & (mx == r), m & (mx == g) & (mx != r), m & (mx == b) & (mx != r) & (mx != g)
    h[rm] = ((g - b)[rm] / d[rm]) % 6
    h[gm] = (b - r)[gm] / d[gm] + 2
    h[bm] = (r - g)[bm] / d[bm] + 4
    s = np.where(mx > 1e-9, d / np.maximum(mx, 1e-9), 0)
    return np.stack([h / 6, s, mx], -1)


def hsv_to_rgb(hsv):
    h, s, v = hsv[..., 0] * 6, hsv[..., 1], hsv[..., 2]
    i = np.floor(h).astype(int) % 6
    f = h - np.floor(h)
    p, q, t = v * (1 - s), v * (1 - s * f), v * (1 - s * (1 - f))
    sel = [(v, t, p), (q, v, p), (p, v, t), (p, q, v), (t, p, v), (v, p, q)]
    out = np.zeros(hsv.shape)
    for k, (a, b, c) in enumerate(sel):
        m = i == k
        out[..., 0][m], out[..., 1][m], out[..., 2][m] = a[m], b[m], c[m]
    return out


def contrast_k(a, target):
    """The contrast factor that gives the p10-p90 spread `target` (codes), as contrast.py finds it."""
    s = a[..., 3] > 0.5
    k = 1.0
    for _ in range(6):
        sp = spread(apply_contrast(a, k), s)
        k *= target / max(sp, 1e-6)
    return k


def apply_contrast(a, k):
    b = a.copy()
    s = b[..., 3] > 0.5
    L = b[..., :3] @ W
    mean = L[s].mean()
    t = mean + (L - mean) * k
    b[..., :3] = np.clip(b[..., :3] * (t / np.maximum(L, 1e-6))[..., None], 0, 1)
    return b


def spread(a, s=None):
    s = a[..., 3] > 0.5 if s is None else s
    L = (a[..., :3][s] @ W) * 255
    return np.percentile(L, 90) - np.percentile(L, 10)


def grade(a, kn, k=None):
    """One tile through the knobs (a: RGBA 0..1). `k`: a contrast factor fixed by the main tile."""
    b = a.copy()
    rgb = b[..., :3] ** kn["gamma"] * kn["gain"]
    b[..., :3] = np.clip(rgb, 0, 1)
    if k is not None and k != 1.0:
        b = apply_contrast(b, k)
    rgb = b[..., :3]
    lum = (rgb @ W)[..., None]
    rgb = lum + (rgb - lum) * kn["saturation"]
    if kn["hue"]:
        hsv = rgb_to_hsv(np.clip(rgb, 0, 1))
        hsv[..., 0] = (hsv[..., 0] + kn["hue"] / 360) % 1
        rgb = hsv_to_rgb(hsv)
    if kn["pull"] is not None and kn["pull_amount"]:
        ref = np.array(kn["pull"], float) / 255
        ref = ref / (ref @ W)
        rgb = rgb * (1 - kn["pull_amount"]) + lum * ref * kn["pull_amount"]
    b[..., :3] = np.clip(rgb, 0, 1)
    if kn.get("outline", 1.0) != 1.0:  # the 1-px edge (opaque pixels beside a transparent one) darkened or lightened
        s = b[..., 3] > 0.5
        pad = np.pad(s, 1)
        inner = pad[:-2, 1:-1] & pad[2:, 1:-1] & pad[1:-1, :-2] & pad[1:-1, 2:]
        edge = s & ~inner
        b[..., :3][edge] = np.clip(b[..., :3][edge] * kn["outline"], 0, 1)
    return b


# --- the table --------------------------------------------------------------------------------------

def load_table(paths):
    t = {"groups": {}, "tiles": {}, "item_groups": {}, "items": {}}
    for p in paths:
        d = tomllib.load(open(p, "rb"))
        for sec in t:
            for n, row in d.get(sec, {}).items():
                t[sec].setdefault(n, {}).update(row)
    return t


def knobs(t, key, kind="block"):
    groups, tiles = t[KINDS[kind]["groups"]], t[KINDS[kind]["tiles"]]
    name = key.split(":", 1)[1] if ":" in key else key
    kn = dict(NEUTRAL)
    for g, row in groups.items():
        if any(fnmatch.fnmatch(name, m) for m in row.get("match", [])):
            kn.update({k: v for k, v in row.items() if k in NEUTRAL})
            break
    kn.update({k: v for k, v in tiles.get(key, {}).items() if k in NEUTRAL})
    if "contrast" in tiles.get(key, {}):
        kn["spread"] = None if "spread" not in tiles.get(key, {}) else kn["spread"]
    return kn


def neutral(kn):
    return all(kn[k] == v for k, v in NEUTRAL.items())


def files_of(mod, name, base):
    """The tile and its CTM variants, relative to `base`."""
    out = [name + ".png"] if os.path.exists(os.path.join(base, name + ".png")) else []
    out += sorted(os.path.relpath(p, base) for p in glob.glob(os.path.join(base, name, "*.png")))
    return out


# --- the build --------------------------------------------------------------------------------------

def load(p):
    return np.asarray(Image.open(p).convert("RGBA")).astype(float) / 255


def save(a, p, like):
    im = Image.fromarray((a * 255).round().astype(np.uint8), "RGBA")
    if Image.open(like).mode != "RGBA" and (a[..., 3] >= 0.999).all():
        im = im.convert("RGB")
    im.save(p)


def measure(a):
    s = a[..., 3] > 0.5
    c = a[..., :3][s]
    h, sat, v = colorsys.rgb_to_hsv(*c.mean(0))
    return dict(luma=(c @ W).mean(), sat=sat, hue=h * 360, spread=spread(a, s))


def build(table_paths, dry=False):
    t = load_table(table_paths)
    lock = json.load(open(LOCK)) if os.path.exists(LOCK) else {}
    done = []
    for kind in KINDS:
        done += build_kind(t, lock, dry, kind)
    done += overlays(t, lock, dry)
    if not dry:
        json.dump(dict(sorted(lock.items())), open(LOCK, "w"), indent=1)
    return done


OVERLAYS = os.path.join(ROOT, "mods", "base", "source", "blocks", "overlays")
def build_kind(t, lock, dry, kind):
    """Every tile (block) or icon (item) a group or row grades, and every one that has a source."""
    tiles = t[KINDS[kind]["tiles"]]
    keys = set()
    for mod in MODS:
        for p in glob.glob(os.path.join(game_dir(mod, kind), "*.png")):
            n = os.path.basename(p)[:-4]
            if n.endswith("_e"):
                continue
            key = n if mod == "base" else f"{mod}:{n}"
            if "base" in tiles.get(key, {}):  # built from its host by overlays()
                continue
            if not neutral(knobs(t, key, kind)) or os.path.exists(os.path.join(src_dir(mod, kind), n + ".png")):
                keys.add((mod, n, key))
    done = []
    lockpre = "" if kind == "block" else kind + "/"
    for mod, n, key in sorted(keys):
        kn = knobs(t, key, kind)
        gd, sd = game_dir(mod, kind), src_dir(mod, kind)
        for rel in files_of(mod, n, gd):
            g, s = os.path.join(gd, rel), os.path.join(sd, rel)
            lk = f"{mod}/{lockpre}{rel}"
            if not os.path.exists(s):  # graded for the first time: the game's tile is the drawing
                if dry:
                    continue
                os.makedirs(os.path.dirname(s), exist_ok=True)
                shutil.copyfile(g, s)
                print(f"  adopted {lk} as its source")
            elif lk in lock and sha(g) != lock[lk] and not dry:  # replaced since the last build: a new drawing
                shutil.copyfile(g, s)
                print(f"  {lk} changed outside the grade: taken as the new source")
        if dry and not os.path.exists(os.path.join(sd, n + ".png")):
            sd = gd  # a dry run reads a tile not yet adopted from the game
        rels = files_of(mod, n, sd)
        main = load(os.path.join(sd, rels[0]))
        k = None
        if kn["spread"] is not None:
            k = contrast_k(grade(main, kn), kn["spread"])
        elif kn["contrast"] != 1.0:
            k = kn["contrast"]
        for rel in rels:
            a = load(os.path.join(sd, rel))
            out = grade(a, kn, k)
            g = os.path.join(gd, rel)
            if not dry:
                save(out, g, os.path.join(sd, rel))
                lock[f"{mod}/{lockpre}{rel}"] = sha(g)
        done.append((mod, n, key if kind == "block" else f"item:{key}", kn, main, grade(main, kn, k)))
    return done


def inside_shifts(ov):
    """Where each variant's marks sit: moved within the overlay's own margins, so no mark wraps round the
    tile's edge and shows cut in half at a block border (a seam sets its own sideways `shifts`)."""
    ys, xs = np.nonzero(ov[..., 3] > 0.5)
    n = ov.shape[0]
    left, right, top, bottom = int(xs.min()), int(n - 1 - xs.max()), int(ys.min()), int(n - 1 - ys.max())
    return [[0, 0], [right, -top], [-left, bottom], [(right - left) // 2, (bottom - top) // 2]]


def overlays(t, lock, dry):
    """Rows with `base` and `overlay` (ores in a rock face): the host's own graded tile and variants with
    the overlay's marks laid on, so the rock under them is exactly the wall's and follows its tuning.
    `shifts` sets where the marks sit per variant; a seam keeps dy 0 so it stays level and tiles."""
    done = []
    for key, row in sorted(t["tiles"].items()):
        if "base" not in row or "overlay" not in row:
            continue
        mod, n = key.split(":", 1) if ":" in key else ("base", key)
        hkey = row["base"]
        hmod, hn = hkey.split(":", 1) if ":" in hkey else ("base", hkey)
        hkn = knobs(t, hkey)
        sd = src_dir(hmod) if os.path.exists(os.path.join(src_dir(hmod), hn + ".png")) else game_dir(hmod)
        rels = files_of(hmod, hn, sd)
        main = load(os.path.join(sd, rels[0]))
        k = contrast_k(grade(main, hkn), hkn["spread"]) if hkn["spread"] is not None else (hkn["contrast"] if hkn["contrast"] != 1.0 else None)
        ov = load(os.path.join(OVERLAYS, row["overlay"] + ".png"))
        shifts = row.get("shifts") or inside_shifts(ov)
        outs = []
        for i, rel in enumerate(rels):
            host = grade(load(os.path.join(sd, rel)), hkn, k)
            dx, dy = shifts[i % len(shifts)]
            o = np.roll(ov, (dy, dx), (0, 1))
            m = o[..., 3:] > 0.5
            out = np.where(m, o, host)
            out[..., 3] = 1.0
            outs.append(out)
            dst = rel.replace(hn, n, 1)
            g = os.path.join(game_dir(mod), dst)
            if not dry:
                os.makedirs(os.path.dirname(g), exist_ok=True)
                save(out, g, os.path.join(sd, rel))
                lock[f"{mod}/{dst}"] = sha(g)
        ctm = os.path.join(game_dir(hmod), hn + ".ctm")
        if not dry and len(rels) > 1 and os.path.exists(ctm):
            text = open(ctm).read().replace(hn, n)
            open(os.path.join(game_dir(mod), n + ".ctm"), "w").write(text)
        done.append((mod, n, key, hkn, grade(main, hkn, k), outs[0]))
    return done


def report(done, packs):
    vc = None
    if packs and os.path.isdir(packs):
        sys.path.insert(0, HERE)
        import vanilla_compare as vc
        refs = vc.refs_in(packs)
    print(f"{'tile':34s} {'luma':>11s} {'sat':>11s} {'spread':>9s}   vanilla band (luma / sat)")
    rows = []
    for mod, n, key, kn, before, after in done:
        b, a = measure(before), measure(after)
        band = ""
        if vc and not key.startswith("item:"):
            vname, kind = vc.vanilla_name(n)
            tint = vc.tint_for(vname)
            rm = [vc.measure(vc.load(f"{d}/{vname}.png"), tint) for d in refs.values() if os.path.exists(f"{d}/{vname}.png")]
            if rm:
                band = (f"{min(r['luma'] for r in rm):.2f}-{max(r['luma'] for r in rm):.2f} / "
                        f"{min(r['sat'] for r in rm):.2f}-{max(r['sat'] for r in rm):.2f}  ~ {vname} ({kind})")
        changed = not np.allclose(before, after, atol=1 / 255)
        print(f"{key:34s} {b['luma']:.2f}->{a['luma']:.2f} {b['sat']:.2f}->{a['sat']:.2f} {b['spread']:3.0f}->{a['spread']:3.0f}   {band}")
        rows.append((key, before, after, b, a, band, changed))
    return rows


def preview(rows, path, tables="grade.toml"):
    rows = [r for r in rows if r[6]]
    F = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 17)
    S = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 13)
    cols, cw, ch = 3, 420, 210
    im = Image.new("RGB", (cols * cw + 20, 50 + -(-len(rows) // cols) * ch), (42, 32, 26))
    d = ImageDraw.Draw(im)
    d.text((12, 12), f"The grade, before (source) and after, {len(rows)} tiles changed by {tables}", font=F, fill=(245, 228, 188))

    def tiled(a):
        t = Image.fromarray((a * 255).round().astype(np.uint8), "RGBA").resize((48, 48), Image.NEAREST)
        g = Image.new("RGBA", (144, 144), (156, 198, 224, 255))
        for x in range(3):
            for y in range(3):
                g.alpha_composite(t, (x * 48, y * 48))
        return g.convert("RGB")
    for i, (key, before, after, b, a, band, _) in enumerate(rows):
        x, y = 12 + (i % cols) * cw, 46 + (i // cols) * ch
        im.paste(tiled(before), (x, y + 20))
        im.paste(tiled(after), (x + 152, y + 20))
        d.text((x, y), key, font=F, fill=(255, 190, 69))
        d.text((x, y + 168), f"luma {b['luma']:.2f}->{a['luma']:.2f}  sat {b['sat']:.2f}->{a['sat']:.2f}  spread {b['spread']:.0f}->{a['spread']:.0f}", font=S, fill=(245, 228, 188))
        if band:
            d.text((x, y + 186), "vanilla " + band.split("  ~")[0], font=S, fill=(194, 172, 136))
    im.save(path)
    print(path, im.size)


def old_cli(argv):
    pull = [int(v) for v in argv[4].split(",")] if len(argv) > 4 else None
    kn = dict(NEUTRAL, gamma=float(argv[2]), saturation=float(argv[3]), pull=pull, pull_amount=float(argv[5]) if len(argv) > 5 else 0.5)
    a = load(argv[0])
    save(grade(a, kn), argv[1], argv[0])


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] != "--all":
        old_cli(sys.argv[1:])
        sys.exit()
    ap = argparse.ArgumentParser()
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--with", dest="extra", action="append", default=[], help="another table on top (proposals)")
    ap.add_argument("--preview")
    ap.add_argument("--packs", default=os.environ.get("VANILLA_PACKS"))
    ap.add_argument("--dry", action="store_true", help="measure and preview, write nothing")
    ap.add_argument("--only", choices=["block", "item"], help="preview only blocks or only items")
    o = ap.parse_args()
    rows = report(build([TABLE] + o.extra, o.dry), o.packs)
    if o.only:
        rows = [r for r in rows if r[0].startswith("item:") == (o.only == "item")]
    if o.preview:
        preview(rows, o.preview, " + ".join(["grade.toml"] + [os.path.basename(p) for p in o.extra]))
