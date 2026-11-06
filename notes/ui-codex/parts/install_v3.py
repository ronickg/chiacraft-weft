"""Installs the version 3 tool parts and armour shells (art session, `notes/ui-codex/redo-v3/`,
`~/Coding/chiacraft-archive/images/art-redo-v3/tools/`) into the parts system, once Ronald has picked.
The worn armour skins (`entity/armor/worn/`) stay as approved; nothing here touches them.

What it does, per drawing `<material>__<part>_px.png` (32 x 32, cut by art.py):
- **Heads and blades the game already draws** (axe, pickaxe, shovel and hoe heads, the sword's straight
  blade): laid over today's drawing of the same material, so the handle still meets it at today's
  anchor (`crafting::looks::LOOKS`) and nothing in the code changes. A head is placed where its outline
  overlaps today's most; a blade where its tip meets today's tip (its base hides under the hilt). Each
  gets a score, how much of today's outline the new one covers (shape likeness, 0-1), and is flagged to
  look at when it floats off its haft (`gap`, texels from the haft's end to the head), pixels fell off
  the canvas, or a gem spot (`GEM_SPOTS`) misses the icon. A flagged drawing wants redrawing to today's
  size and socket, not moving: a loose head's eye sits mid-head, while today's hafts stop where the old
  heads covered them.
- **Parts the game draws by a plain icon today** (the knife's blade, art `short_blade`, and the spear's
  leaf point): written as `parts/knife/knife_blade/<art>.png` and `parts/spear/leaf_point/<art>.png` with
  their anchor (the tang's or socket's end, the corner toward the bottom left), and a Rust snippet for
  their `LOOKS` rows (a knife grip from the sword's hilt map, a spear shaft drawn in code from the
  haft's shading), in `looks_snippet.rs`.
- **Armour shells** (`<material>__<shell>`, leather, bone, iron, gold, diamond): written as painted shells
  `parts/armour/<shell>/<art>.png` for `part_look` to use where drawn (the others keep their family's
  shade map); reported as waiting while they aren't drawn.

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/install_v3.py             # dry run: everything into
                                                                              #   the archive, a sheet, a report
    ~/.venvs/blocky/bin/python notes/ui-codex/parts/install_v3.py --apply \\
        --only iron,steel --parts felling_axe_head,pick_head                  # into mods/base/assets/textures

`--only` and `--parts` limit it to Ronald's picks (materials, parts); without them, everything drawn.
Every image it replaces is kept in the archive (`replaced/`) first.
"""
import argparse
import json
import os
import shutil
import sys

import numpy as np
from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
TEX = os.path.join(ROOT, "mods", "base", "assets", "textures")
SRC = os.path.expanduser("~/Coding/chiacraft-archive/images/art-redo-v3/tools")
OUT = os.path.expanduser("~/Coding/chiacraft-archive/images/art-redo-v3/install")

MATS = ["wood", "stone", "flint", "copper", "bronze", "iron", "steel", "gold", "diamond"]
# v3 part -> (kind, the game's shape, how it's placed)
HEADS = {
    "felling_axe_head": ("axe", "felling_axe_head", "overlap"),
    "pick_head": ("pickaxe", "pick_head", "overlap"),
    "shovel_blade": ("shovel", "shovel_blade", "overlap"),
    "hoe_blade": ("hoe", "hoe_blade", "overlap"),
    "straight_blade": ("sword", "straight_blade", "tip"),
}
NEW = {"short_blade": ("knife", "knife_blade"), "leaf_point": ("spear", "leaf_point")}
SHELLS = ["helmet_shell", "chest_shell", "leg_shell", "boot_shell"]
SHELL_MATS = ["leather", "bone", "iron", "gold", "diamond"]
# Today's anchors and sockets (crafting::looks::LOOKS), for the previews.
LOOKS_RS = os.path.join(ROOT, "mods", "base", "crafting", "src", "looks.rs")


def load(path):
    return np.array(Image.open(path).convert("RGBA")).astype(np.int32)


def moved(a, dx, dy):
    out = np.zeros_like(a)
    h, w = a.shape[:2]
    ys, xs = slice(max(0, dy), min(h, h + dy)), slice(max(0, dx), min(w, w + dx))
    yo, xo = slice(max(0, -dy), min(h, h - dy)), slice(max(0, -dx), min(w, w - dx))
    out[ys, xs] = a[yo, xo]
    return out


def place_overlap(new, old):
    """The shift that lays `new`'s outline most over `old`'s (ties: the smallest shift)."""
    om, best = old[..., 3] > 0, (-1.0, 0, 0)
    for dy in range(-12, 13):
        for dx in range(-12, 13):
            nm = moved(new, dx, dy)[..., 3] > 0
            iou = (nm & om).sum() / max(1, (nm | om).sum())
            if iou > best[0] + 1e-9 or (abs(iou - best[0]) < 1e-9 and abs(dx) + abs(dy) < abs(best[1]) + abs(best[2])):
                best = (iou, dx, dy)
    return best


def tip(a):
    """The drawing's top-right end along the diagonal (max x - y)."""
    ys, xs = np.nonzero(a[..., 3] > 0)
    k = np.argmax(xs - ys)
    return xs[k], ys[k]


def base(a):
    """Its bottom-left end (min x - y): a tang's or socket's end, where a handle meets it."""
    ys, xs = np.nonzero(a[..., 3] > 0)
    k = np.argmin(xs - ys)
    return float(xs[k]), float(ys[k])


def gem_misses(a, spots):
    """Gem spots (centres of 5 x 5) that fall less than half on the drawing."""
    on = a[..., 3] > 0
    out = []
    for x, y in spots:
        box = on[max(0, y - 2):y + 3, max(0, x - 2):x + 3]
        if box.size == 0 or box.mean() < 0.5:
            out.append([x, y])
    return out


def gem_spots():
    import re
    src = open(LOOKS_RS).read()
    return {k: [(int(a), int(b)) for a, b in re.findall(r"\[(\d+), (\d+)\]", v)]
            for k, v in re.findall(r'\("(\w+)", (\[\[.*?\]\])\)', src)}


def place_tip(new, old):
    (nx, ny), (ox, oy) = tip(new), tip(old)
    dx, dy = int(ox - nx), int(oy - ny)
    nm, om = moved(new, dx, dy)[..., 3] > 0, old[..., 3] > 0
    return (nm & om).sum() / max(1, (nm | om).sum()), dx, dy


def clipped(a, dx, dy):
    """Pixels a shift pushes off the canvas."""
    return int((a[..., 3] > 0).sum() - (moved(a, dx, dy)[..., 3] > 0).sum())


def anchors_today():
    """{kind: (socket, {art: anchor})} parsed from looks.rs."""
    import re
    src = open(LOOKS_RS).read()
    out = {}
    for m in re.finditer(r'kind: "(\w+)",\s*socket: \[([\d.]+), ([\d.]+)\],\s*anchors: &\[(.*?)\],', src, re.S):
        a = {n: (float(x), float(y)) for n, x, y in re.findall(r'\("(\w+)", \[([\d.]+), ([\d.]+)\]\)', m.group(4))}
        out[m.group(1)] = ((float(m.group(2)), float(m.group(3))), a)
    return out


def ramp(name):
    return np.array(Image.open(os.path.join(TEX, "ramps", name + ".png")).convert("RGBA")).astype(np.int32)[0]


def paint(m, rmps):
    """A shade map painted with ramps, as the engine does (step R/36, region G)."""
    out = np.zeros_like(m)
    for y, x in zip(*np.nonzero(m[..., 3] > 0)):
        r = rmps[m[y, x, 1]] if m[y, x, 1] < len(rmps) else None
        if r is not None:
            out[y, x] = list(r[min(len(r) - 1, m[y, x, 0] // 36)][:3]) + [255]
    return out


def over(dst, src):
    on = src[..., 3] > 0
    dst[on] = src[on]
    return dst


def icon(kind, head, shift, handle):
    """A tool's icon from its head (fixed) and its handle (a painted map, moved by `shift`)."""
    c = np.zeros((32, 32, 4), np.int32)
    if kind == "sword":
        return over(over(c, head), moved(handle, *shift))
    return over(over(c, moved(handle, *shift)), head)


def shaft_map(socket):
    """The spear's shaft as a shade map (region 0), drawn in code: two texels wide on the diagonal from
    the bottom-left corner to `socket`, its upper-left edge lit and its lower-right in shadow, like the
    haft's shading."""
    m = np.zeros((32, 32, 4), np.int32)
    sx, sy = socket
    for t in np.linspace(0, 1, 200):
        x, y = 1 + (sx - 1) * t, 30 + (sy - 30) * t
        for off, step in ((0, 5), (1, 3)):
            X, Y = int(round(x + off)), int(round(y))
            if 0 <= X < 32 and 0 <= Y < 32:
                m[Y, X] = [step * 36, 0, 0, 255]
        X, Y = int(round(x)), int(round(y - 1))
        if 0 <= X < 32 and 0 <= Y < 32 and m[Y, X, 3] == 0:
            m[Y, X] = [6 * 36, 0, 0, 255]
    return m


def keep_old(path, rel):
    if os.path.exists(path):
        dst = os.path.join(OUT, "replaced", rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        if not os.path.exists(dst):
            shutil.copy(path, dst)


def write(arr, rel, apply):
    dst = os.path.join(TEX if apply else os.path.join(OUT, "textures"), rel)
    if apply:
        keep_old(dst, rel)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    Image.fromarray(arr.clip(0, 255).astype(np.uint8)).save(dst)
    return dst


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--only", default="")
    ap.add_argument("--parts", default="")
    ap.add_argument("--src", default=SRC)
    args = ap.parse_args()
    only = set(filter(None, args.only.split(",")))
    pick = set(filter(None, args.parts.split(",")))
    want = lambda m, p: (not only or m in only) and (not pick or p in pick)
    os.makedirs(OUT, exist_ok=True)
    looks = anchors_today()
    spots = gem_spots()
    oak, leather, iron = ramp("oak"), ramp("leather"), ramp("wrought_iron")
    report, rows = {"heads": {}, "new": {}, "shells": {}, "missing": []}, []

    for part, (kind, shape, how) in HEADS.items():
        socket, anchors = looks[kind]
        handle_map = load(os.path.join(TEX, "parts", kind, "grip.png" if kind == "sword" else "haft.png"))
        handle = paint(handle_map, [leather, iron, iron] if kind == "sword" else [oak])
        for m in MATS:
            src = os.path.join(args.src, f"{m}__{part}_px.png")
            if not want(m, part):
                continue
            if not os.path.exists(src):
                report["missing"].append(f"{m}__{part}")
                continue
            new = load(src)
            old_p = os.path.join(TEX, "parts", kind, shape, f"{m}.png")
            old = load(old_p) if os.path.exists(old_p) else None
            a = anchors.get(m)
            shift = (int(round(a[0] - socket[0])), int(round(a[1] - socket[1]))) if a else (0, 0)
            if old is None:
                # A material today's kind doesn't draw (flint on a sword): placed as drawn, flagged.
                score, dx, dy = 0.0, 0, 0
            else:
                score, dx, dy = (place_tip if how == "tip" else place_overlap)(new, old)
            placed = moved(new, dx, dy)
            write(placed, f"parts/{kind}/{shape}/{m}.png", args.apply)
            full = icon(kind, placed, shift, handle)
            misses = gem_misses(full, spots.get(kind, []))
            cl = clipped(new, dx, dy)
            # How far the head floats off its haft: from the haft's end to the nearest head pixel.
            gap = 0.0
            if kind != "sword":
                ex, ey = tip(moved(handle, *shift))
                ys, xs = np.nonzero(placed[..., 3] > 0)
                gap = float(np.sqrt(((xs - ex) ** 2 + (ys - ey) ** 2).min())) if len(xs) else 99.0
            info = {"score": round(float(score), 2), "moved": [dx, dy], "clipped": cl, "gap": round(gap, 1),
                    "gem_misses": misses, "anchor": a}
            info["flag"] = bool(old is None or cl > 0 or misses or gap > 1.5)
            report["heads"][f"{kind}/{m}"] = info
            rows.append((f"{kind} {m}", icon(kind, old, shift, handle) if old is not None else None, full, info))

    # The knife and the spear: drawn by plain icons today.
    grip = load(os.path.join(TEX, "parts", "sword", "grip.png"))
    gsock = looks["sword"][0]
    snippet = {}
    for part, (kind, shape) in NEW.items():
        anchors = {}
        for m in MATS:
            src = os.path.join(args.src, f"{m}__{part}_px.png")
            if not want(m, part):
                continue
            if not os.path.exists(src):
                report["missing"].append(f"{m}__{part}")
                continue
            new = load(src)
            write(new, f"parts/{kind}/{shape}/{m}.png", args.apply)
            anchors[m] = base(new)
        if not anchors:
            continue
        ref = anchors.get("iron") or next(iter(anchors.values()))
        if kind == "knife":
            # The sword's hilt, its socket moved to the iron blade's base; regions grip, fitting (the guard).
            hmap, sock = grip, gsock
            regions = '&["grip", "fitting"]'
            layers = [("blade", '&["blade"]', "false", "true"), ("hilt", regions, "true", "false")]
            handle_rel = None
        else:
            hmap, sock = shaft_map(ref), ref
            handle_rel = f"parts/{kind}/shaft.png"
            write(hmap, handle_rel, args.apply)
            layers = [("shaft", '&["shaft"]', "true", "false"), ("point", '&["point", "fitting"]', "false", "true")]
        report["new"][kind] = {m: [round(a[0], 1), round(a[1], 1)] for m, a in anchors.items()}
        handle = paint(hmap, [leather, iron] if kind == "knife" else [oak])
        for m, a in anchors.items():
            new = load(os.path.join(args.src, f"{m}__{part}_px.png"))
            shift = (int(round(a[0] - sock[0])), int(round(a[1] - sock[1])))
            c = np.zeros((32, 32, 4), np.int32)
            if kind == "knife":
                over(over(c, new), moved(handle, *shift))
            else:
                over(over(c, moved(handle, *shift)), new)
            rows.append((f"{kind} {m} (new)", None, c, {"score": None, "anchor": a}))
        anc = ",\n".join(f'            ("{m}", [{a[0]:.1f}, {a[1]:.1f}])' for m, a in anchors.items())
        lay = ",\n".join(f'            Layer {{ name: "{n}", regions: {r}, area: [0, 0, 31, 31], moves: {mv}, painted: {p} }}'
                         for n, r, mv, p in layers)
        snippet[kind] = (f'    KindLook {{\n        kind: "{kind}",\n        socket: [{sock[0]:.1f}, {sock[1]:.1f}],\n'
                         f'        anchors: &[\n{anc},\n        ],\n        layers: &[\n{lay},\n        ],\n    }},\n'
                         + (f"    // its handle map: {handle_rel} (written by install_v3.py)\n" if handle_rel else
                            "    // its hilt map: parts/knife/grip.png, a copy of parts/sword/grip.png\n"))
        if kind == "knife":
            write(grip, "parts/knife/grip.png", args.apply)
    open(os.path.join(OUT, "looks_snippet.rs"), "w").write("".join(snippet.values()))

    # Armour shells: painted per material where drawn.
    for m in SHELL_MATS:
        for s in SHELLS:
            src = os.path.join(args.src, f"{m}__{s}_px.png")
            if not want(m, s):
                continue
            if not os.path.exists(src):
                report["shells"][f"{m}/{s}"] = "waiting: not drawn yet"
                continue
            new = load(src)
            write(new, f"parts/armour/{s}/{m}.png", args.apply)
            report["shells"][f"{m}/{s}"] = "written"
            rows.append((f"{s} {m}", None, new, {"score": None}))

    # The sheet: today and v3 side by side, at 3x, each with its score.
    S, cw = 3, 2 * 96 + 20
    per = 5
    n = len(rows)
    sheet = Image.new("RGBA", (per * (cw + 10) + 10, ((n + per - 1) // per) * 130 + 30), (53, 32, 21, 255))
    d = ImageDraw.Draw(sheet)
    d.text((10, 8), "v3 parts as installed: today | v3, where today's head sits (score = likeness to today's shape); red = floats off the haft, clipped or a gem misses", fill=(198, 163, 131, 255))
    for i, (label, a, b, info) in enumerate(rows):
        x, y = 10 + (i % per) * (cw + 10), 30 + (i // per) * 130
        flag = info.get("flag")
        sc = info.get("score")
        gp = info.get("gap")
        d.text((x, y), label + (f"  {sc:.2f}" if sc is not None else "") + (f"  gap {gp:.0f}" if gp else ""), fill=(230, 90, 70, 255) if flag else (240, 228, 200, 255))
        for k, arr in enumerate((a, b)):
            if arr is not None:
                im = Image.fromarray(arr.clip(0, 255).astype(np.uint8)).resize((96, 96), Image.NEAREST)
                sheet.alpha_composite(im, (x + k * (96 + 10), y + 16))
    sheet.save(os.path.join(OUT, "install_sheet.png"))
    json.dump(report, open(os.path.join(OUT, "report.json"), "w"), indent=1)
    flagged = [k for k, v in report["heads"].items() if v["flag"]]
    print(f"{'applied to ' + TEX if args.apply else 'dry run into ' + OUT}: {len(report['heads'])} heads "
          f"({len(flagged)} to look at: {', '.join(flagged)}), new {list(report['new'])}, "
          f"shells {sum(v == 'written' for v in report['shells'].values())} written, "
          f"{sum(v != 'written' for v in report['shells'].values())} waiting, missing {report['missing']}")
    print("sheet:", os.path.join(OUT, "install_sheet.png"))


if __name__ == "__main__":
    sys.exit(main())
