"""The vanilla comparison sheet from vanilla_compare.csv: per flagged tile ours | vanilla | Faithful 32x |
Pixel Perfection | ours corrected, each tiled 3x3 (16 px packs shown at 32), with the numbers; grouped
(fix now, own identity, waits for the grade, not a tile), worst first.
python3 -E -P vanilla_sheet.py <our worktree> <packs dir> <out dir>"""
import csv, os, sys
import numpy as np
from PIL import Image, ImageDraw, ImageFont

OURS, PACKS, OUT = sys.argv[1:4]
REFS = [("vanilla", f"{PACKS}/vanilla/assets/minecraft/textures/block"), ("faithful32", f"{PACKS}/faithful32/assets/minecraft/textures/block"),
        ("pixelperf", f"{PACKS}/pixelperfection/assets/minecraft/textures/block")]
TINTS = {"grass": "91BD59", "foliage": "77AB2F", "spruce": "619961", "birch": "80A755", "lily": "208030"}
# Flags a tone shift can't fix well: shown, no corrected tile.
NOTES = {"gold_ore": "needs a redraw: more gold drawn into our stone; raising saturation only tints the stone",
         "glass": "keep: the wooden frame is our design; the saturation is the frame's",
         "observer_back": "a model texture made from old 16 px pictures: darken with the redstone set later",
         "observer_back_on": "as observer_back", "piston_inner": "a model texture: darken with the redstone set later"}
WAIT = ("leaves_", "tallgrass", "grass_", "bracken", "sword_fern", "fern")
NOT_TILE = ("water_still",)
IDENTITY = ("sequoia", "mesquite", "limestone", "ochre_sandstone", "slate", "termite", "banded_clay", "dogwood", "honey_fungus", "porcini",
            "cream_sandstone")
F = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 18)
S = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 13)
IV, AM, GR = (245, 228, 188), (255, 190, 69), (194, 172, 136)


def tint_for(v):
    if v == "lily_pad": return "lily"
    if v.startswith("spruce_leaves"): return "spruce"
    if v.startswith("birch_leaves"): return "birch"
    if v.endswith("_leaves"): return "foliage"
    if v in ("grass_block_top", "short_grass", "fern"): return "grass"
    return None


def panel(p, tint):
    im = Image.open(p).convert("RGBA")
    if im.height > im.width:
        im = im.crop((0, 0, im.width, im.width))
    a = np.asarray(im).astype(float)
    s = a[..., 3] > 127
    c = a[..., :3][s]
    if tint and ((c.max(1) - c.min(1)) / np.maximum(c.max(1), 1)).mean() < 0.05:
        a[..., :3] *= np.array([int(TINTS[tint][i:i + 2], 16) for i in (0, 2, 4)]) / 255
    t = Image.fromarray(a.clip(0, 255).astype(np.uint8), "RGBA").resize((48, 48), Image.NEAREST)
    g = Image.new("RGBA", (144, 144), (156, 198, 224, 255))
    for x in range(3):
        for y in range(3):
            g.alpha_composite(t, (x * 48, y * 48))
    return g


def group(r):
    n = r["tile"]
    if any(n.startswith(w) or n == w for w in WAIT): return 2
    if n in NOT_TILE: return 3
    if any(k in n for k in IDENTITY): return 1
    return 0


def main():
    rows = [r for r in csv.DictReader(open(f"{OUT}/vanilla_compare.csv")) if r["flags"]]
    for r in rows:
        r["score"] = float(r["score"])
    rows.sort(key=lambda r: (group(r), -r["score"]))
    titles = ["Fix now: our tile is a vanilla twin and drifts from it (my pick: apply the corrected tile)",
              "Own identity: ours has no vanilla twin, compared with its nearest relative (my pick: keep, or the mild fix where noted)",
              "Waits for Lighting's Colour 8 (Ronald: leaves and grass \"After the grade\")",
              "Not a tile fix"]
    RH = 196
    H = 120 + len(rows) * RH + 4 * 50
    sh = Image.new("RGB", (1240, H), (42, 32, 26))
    d = ImageDraw.Draw(sh)
    d.text((14, 12), "Our blocks against vanilla 1.21.4, Faithful 32x and Pixel Perfection CE: the ones outside the band", font=F, fill=IV)
    d.text((14, 40), "Band: the three references' range, widened by hue 15 deg, luma 0.05, saturation 0.10 (hue only for coloured tiles). Tinted tiles with vanilla's tints.", font=S, fill=GR)
    d.text((14, 58), "Columns: ours | vanilla | Faithful 32x | Pixel Perfection | ours corrected in code (hue turned, saturation and level scaled; the drawing stays ours).", font=S, fill=GR)
    d.text((14, 76), f"{len(rows)} of 147 matched tiles are outside the band. Full numbers: vanilla_compare.csv beside this sheet.", font=S, fill=GR)
    y, g_prev = 104, None
    for r in rows:
        g = group(r)
        if g != g_prev:
            d.text((14, y), titles[g], font=F, fill=AM)
            y += 34
            g_prev = g
        v = r["vanilla"]
        tint = tint_for(v)
        x = 14
        ours = f"{OURS}/mods/{r['mod']}/assets/textures/block/{r['tile']}.png"
        cols = [("ours", ours)] + [(k, f"{dd}/{v}.png") for k, dd in REFS] + [("corrected", f"{OUT}/corrected/{r['mod']}_{r['tile']}.png")]
        for lab, p in cols:
            if os.path.exists(p) and not ((g >= 2 or r["tile"] in NOTES) and lab == "corrected"):
                sh.paste(panel(p, tint).convert("RGB"), (x, y))
            d.text((x, y + 146), lab, font=S, fill=GR)
            x += 154
        d.text((x + 6, y + 4), f"{r['mod']}:{r['tile']}", font=F, fill=IV)
        d.text((x + 6, y + 30), f"~ {v} ({r['match']})", font=S, fill=GR)
        d.text((x + 6, y + 50), f"ours: luma {float(r['ours_luma']):.2f} sat {float(r['ours_sat']):.2f} hue {float(r['ours_hue']):.0f}", font=S, fill=IV)
        d.text((x + 6, y + 68), f"vanilla: luma {float(r['vanilla_luma']):.2f} sat {float(r['vanilla_sat']):.2f} hue {float(r['vanilla_hue']):.0f}", font=S, fill=IV)
        if r.get("fixed_luma"):
            d.text((x + 6, y + 86), f"corrected: luma {float(r['fixed_luma']):.2f} sat {float(r['fixed_sat']):.2f} hue {float(r['fixed_hue']):.0f}", font=S, fill=IV)
        wrap = r["flags"].split("; ") + ([NOTES[r["tile"]]] if r["tile"] in NOTES else [])
        for i, f in enumerate(wrap):
            d.text((x + 6, y + 110 + i * 18), f, font=S, fill=AM)
        y += RH
    sh = sh.crop((0, 0, 1240, y + 10))
    sh.save(f"{OUT}/vanilla_compare_sheet.png")
    print(f"{OUT}/vanilla_compare_sheet.png", sh.size)


if __name__ == "__main__":
    main()
