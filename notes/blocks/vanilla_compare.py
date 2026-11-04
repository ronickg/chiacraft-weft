"""Compare our block tiles with vanilla 1.21.4, Faithful 32x and Pixel Perfection CE (downloaded to the
scratchpad, read as images only). Per tile: mean sRGB, hue, saturation (HSV of the mean), luma, p10-p90
spread in codes, over opaque pixels; grey tiles tinted with vanilla's tints. A tile is flagged when it
falls outside the references' range widened by hue 15 deg, luma 0.05, saturation 0.10. Each flagged tile
gets a corrected copy of ours, made in code (hue rotated, saturation and level scaled to the band edge).

python3 -I vanilla_compare.py <our worktree> <packs dir> <out dir>
"""
import colorsys, csv, glob, os, sys
import numpy as np
from PIL import Image

OURS, PACKS, OUT = sys.argv[1:4]
REFS = {"vanilla": f"{PACKS}/vanilla/assets/minecraft/textures/block",
        "faithful32": f"{PACKS}/faithful32/assets/minecraft/textures/block",
        "pixelperf": f"{PACKS}/pixelperfection/assets/minecraft/textures/block"}
TINTS = {"grass": "91BD59", "foliage": "77AB2F", "spruce": "619961", "birch": "80A755", "lily": "208030"}
TOL = dict(hue=15.0, luma=0.05, sat=0.10)
# our name -> vanilla name (rules below cover the regular ones)
MANUAL = {"grass_top": "grass_block_top", "grass_side": "grass_block_side", "grass_side_overlay": "grass_block_side_overlay",
          "stonebrick": "stone_bricks", "tallgrass": "short_grass", "mushroom_brown": "brown_mushroom", "mushroom_red": "red_mushroom",
          "deadbush": "dead_bush", "reeds": "sugar_cane", "hay_bale_side": "hay_block_side", "hay_bale_top": "hay_block_top",
          "sandstone_side": "sandstone", "red_sandstone_side": "red_sandstone", "water_still": "water_still", "lava_still": "lava_still",
          "snow": "snow", "podzol": "podzol_top", "farmland_wet": "farmland_moist", "wool_white": "white_wool", "wool": "white_wool",
          "melon_side": "melon_side", "pumpkin_side": "pumpkin_side", "pumpkin_top": "pumpkin_top", "glass": "glass", "ice": "ice",
          "moss_block_top": "moss_block", "coarse_dirt": "coarse_dirt", "leaf_litter_top": "leaf_litter", "bookshelf": "bookshelf",
          "crafting_table_top": "crafting_table_top", "furnace_front": "furnace_front", "clay": "clay", "gravel": "gravel"}
# ours with no vanilla twin -> its nearest vanilla relative
RELATIVE = {"log_sequoia": "spruce_log", "log_sequoia_top": "spruce_log_top", "planks_sequoia": "spruce_planks", "leaves_sequoia": "spruce_leaves",
            "log_sequoia_mossy": "spruce_log", "termite_mound_side": "terracotta", "termite_mound_top": "terracotta",
            "termite_mound_upper_side": "terracotta", "banded_clay_side": "orange_terracotta", "banded_clay_top": "orange_terracotta",
            "cream_sandstone_side": "sandstone", "cream_sandstone_top": "sandstone_top", "ochre_sandstone_side": "sandstone",
            "ochre_sandstone_top": "sandstone_top", "limestone_side": "calcite", "limestone_top": "calcite", "weathered_limestone_side": "calcite",
            "slate_side": "deepslate", "slate_top": "deepslate_top", "log_dogwood": "oak_log", "leaves_dogwood_green": "birch_leaves",
            "leaves_willow": "oak_leaves", "leaves_juniper": "spruce_leaves", "leaves_creosote": "acacia_leaves", "planks_acacia": "acacia_planks",
            "planks_mesquite": "acacia_planks", "planks_acuri_palm": "jungle_planks", "planks_moriche_palm": "jungle_planks",
            "log_mesquite": "acacia_log", "bracken": "fern", "sword_fern": "fern", "honey_fungus": "brown_mushroom", "porcini": "brown_mushroom"}


def vanilla_name(n):
    if n in MANUAL:
        return MANUAL[n], "twin"
    if n in RELATIVE:
        return RELATIVE[n], "relative"
    for pre, fmt in (("log_", "{}_log"), ("planks_", "{}_planks"), ("leaves_", "{}_leaves"), ("sapling_", "{}_sapling")):
        if n.startswith(pre):
            rest = n[len(pre):]
            if rest.endswith("_top"):
                return fmt.format(rest[:-4]) + "_top", "twin"
            return fmt.format(rest), "twin"
    return n, "twin"


def hexc(h):
    return np.array([int(h[i:i + 2], 16) for i in (0, 2, 4)]) / 255


def tint_for(vname):
    if vname == "lily_pad":
        return "lily"
    if vname.startswith("spruce_leaves"):
        return "spruce"
    if vname.startswith("birch_leaves"):
        return "birch"
    if vname.endswith("_leaves") or vname == "vine":
        return "foliage"
    if vname in ("grass_block_top", "short_grass", "fern", "tall_grass_top", "tall_grass_bottom", "grass_block_side_overlay"):
        return "grass"
    return None


def load(p):
    a = np.asarray(Image.open(p).convert("RGBA")).astype(float) / 255
    if a.shape[0] > a.shape[1]:
        a = a[: a.shape[1]]
    return a


def grey(a):
    s = a[..., 3] > 0.5
    c = a[..., :3][s]
    mx, mn = c.max(1), c.min(1)
    return ((mx - mn) / np.maximum(mx, 1e-6)).mean() < 0.05


def measure(a, tint):
    s = a[..., 3] > 0.5
    c = a[..., :3][s].copy()
    if tint is not None and grey(a):
        c *= hexc(TINTS[tint])
    mean = c.mean(0)
    h, sat, v = colorsys.rgb_to_hsv(*mean)
    luma = c @ [0.2126, 0.7152, 0.0722]
    return dict(r=int(mean[0] * 255), g=int(mean[1] * 255), b=int(mean[2] * 255), hue=round(h * 360, 1), sat=round(sat, 3),
                luma=round(luma.mean(), 3), spread=int(round((np.percentile(luma, 90) - np.percentile(luma, 10)) * 255)))


def hdist(a, b):
    d = abs(a - b) % 360
    return min(d, 360 - d)


def correct(a, tint, m, target):
    """Ours moved to the band: hue rotated, saturation and level scaled, per pixel (detail kept)."""
    b = a.copy()
    s = b[..., 3] > 0.5
    rgb = b[..., :3]
    hsv = np.array([colorsys.rgb_to_hsv(*px) for px in rgb.reshape(-1, 3)]).reshape(rgb.shape)
    if target.get("hue") is not None:
        hsv[..., 0] = (hsv[..., 0] + (target["hue"] - m["hue"]) / 360) % 1
    if target.get("sat") is not None and m["sat"] > 0.01:
        hsv[..., 1] = np.clip(hsv[..., 1] * target["sat"] / m["sat"], 0, 1)
    out = np.array([colorsys.hsv_to_rgb(*px) for px in hsv.reshape(-1, 3)]).reshape(rgb.shape)
    b[..., :3] = out
    if target.get("luma") is not None:
        cur = measure(b, tint)["luma"]
        b[..., :3] = np.clip(b[..., :3] * target["luma"] / max(cur, 1e-3), 0, 1)
    b[..., :3][~s] = a[..., :3][~s]
    return b


def main():
    rows = []
    for mod in ("base", "animals"):
        root = f"{OURS}/mods/{mod}/assets/textures/block"
        for p in sorted(glob.glob(root + "/*.png")):
            n = os.path.basename(p)[:-4]
            if n.endswith("_e"):
                continue
            vname, kind = vanilla_name(n)
            refs = {k: f"{d}/{vname}.png" for k, d in REFS.items() if os.path.exists(f"{d}/{vname}.png")}
            if "vanilla" not in refs:
                continue
            tint = tint_for(vname)
            ours = load(p)
            if (ours[..., 3] > 0.5).sum() < 20:
                continue
            m = measure(ours, tint)
            rm = {k: measure(load(v), tint) for k, v in refs.items()}
            row = dict(mod=mod, tile=n, vanilla=vname, match=kind, **{f"ours_{k}": v for k, v in m.items()})
            for k, r in rm.items():
                row.update({f"{k}_{q}": r[q] for q in ("r", "g", "b", "hue", "sat", "luma", "spread")})
            flags, target, score = [], {}, 0.0
            for q in ("luma", "sat"):
                lo, hi = min(r[q] for r in rm.values()) - TOL[q], max(r[q] for r in rm.values()) + TOL[q]
                if not lo <= m[q] <= hi:
                    edge = lo if m[q] < lo else hi
                    flags.append(f"{q} {m[q]:.2f} vs {lo + TOL[q]:.2f}-{hi - TOL[q]:.2f}")
                    target[q] = lo + TOL[q] if m[q] < lo else hi - TOL[q]  # into the packs' own range
                    score += abs(m[q] - edge) / TOL[q]
            if m["sat"] > 0.12 and min(r["sat"] for r in rm.values()) > 0.12:
                ref_h = [r["hue"] for r in rm.values()]
                d = min(hdist(m["hue"], h) for h in ref_h)
                if d > TOL["hue"]:
                    nearest = min(ref_h, key=lambda h: hdist(m["hue"], h))
                    flags.append(f"hue {m['hue']:.0f} vs {nearest:.0f}")
                    step = ((nearest - m["hue"] + 180) % 360) - 180
                    target["hue"] = (m["hue"] + step - np.sign(step) * 5) % 360  # within 5 deg of the nearest pack
                    score += (d - TOL["hue"]) / TOL["hue"]
            row.update(flags="; ".join(flags), score=round(score, 2))
            if flags:
                os.makedirs(f"{OUT}/corrected", exist_ok=True)
                c = correct(ours, tint, m, target)
                Image.fromarray((c * 255).round().astype(np.uint8), "RGBA").save(f"{OUT}/corrected/{mod}_{n}.png")
                cm = measure(c, tint)
                row.update({f"fixed_{q}": cm[q] for q in ("hue", "sat", "luma")})
            rows.append(row)
    keys = []
    for r in rows:
        keys += [k for k in r if k not in keys]
    os.makedirs(OUT, exist_ok=True)
    with open(f"{OUT}/vanilla_compare.csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=keys)
        w.writeheader()
        w.writerows(rows)
    flagged = sorted([r for r in rows if r["flags"]], key=lambda r: -r["score"])
    print(f"{len(rows)} of our tiles have a vanilla match; {len(flagged)} outside the band")
    for r in flagged:
        print(f"  {r['score']:5.2f} {r['mod']}:{r['tile']:28s} ~ {r['vanilla']:24s} {r['match']:8s} {r['flags']}")


if __name__ == "__main__":
    main()
