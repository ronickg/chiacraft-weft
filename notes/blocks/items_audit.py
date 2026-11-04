"""Our item icons against our own guide, our own set and seven packs (Ronald, 11-04: "it might be just our
design, but if so we gotta make sure we consistent").

1. Where our set sits: per pack (vanilla, Faithful 32x, Pixel Perfection CE, OSReP, NariviaCraft,
   CheapMC; sparse checkouts outside the repo, read as images only) and per kind of item, the medians of
   outline darkness, luma, saturation, fill and light direction, over the items we share.
2. Consistency: every one of our items against our guide (`.claude/skills/game-art/guides/1 Items/`:
   outline a dark shade of each part's own colour, light from the top left, fill per kind) and against
   our own median for its kind.

    python3 -E -P notes/blocks/items_audit.py <worktree> <packs dir> <out dir>
"""
import csv
import glob
import os
import statistics as st
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import items_compare as ic  # noqa: E402

PACKS = {
    "vanilla": "vanilla/assets/minecraft/textures/item",
    "faithful32": "faithful32/assets/minecraft/textures/item",
    "pixelperf": "pixelperfection/assets/minecraft/textures/item",
    "osrep": "osrep/OSReP/assets/minecraft/textures/items",
    "nariviacraft": "nariviacraft/assets/minecraft/textures/item",
    "cheapmc": "cheapmc/pack/assets/minecraft/textures/item",
}
# 1.7-era file names (OSReP) for today's
OLD = {"wheat_seeds": "seeds_wheat", "beef": "beef_raw", "cooked_beef": "beef_cooked", "porkchop": "porkchop_raw",
       "cooked_porkchop": "porkchop_cooked", "chicken": "chicken_raw", "cooked_chicken": "chicken_cooked", "melon_slice": "melon",
       "mutton": "mutton_raw", "cooked_mutton": "mutton_cooked", "cod": "fish_cod_raw", "cooked_cod": "fish_cod_cooked",
       "pumpkin_seeds": "seeds_pumpkin", "melon_seeds": "seeds_melon", "sugar_cane": "reeds", "golden_apple": "apple_golden",
       "gold_ingot": "gold_ingot", "iron_ingot": "iron_ingot", "leather_helmet": "leather_helmet"}
# Our guide's fill, the longest side of the bounding box as a share of the 32 px frame (section 1).
KINDS = {
    "tool": ((0.90, 1.00), ("_pickaxe", "_axe", "_shovel", "_hoe", "_sword", "bow", "spear", "knife", "fishing_rod", "shears")),
    "armour": ((0.78, 0.90), ("_helmet", "_chestplate", "_leggings", "_boots", "_cloak", "_robe", "_cape", "_coat", "_hat", "_trousers")),
    "ingot": ((0.72, 0.82), ("_ingot", "raw_iron", "raw_gold", "raw_copper", "brick")),
    "gem": ((0.70, 0.84), ("diamond", "emerald", "coal", "charcoal", "amber", "lapis", "quartz")),
    "food": ((0.72, 0.86), ("apple", "bread", "beef", "porkchop", "chicken", "mutton", "cod", "salmon", "potato", "carrot", "melon",
                            "berries", "stew", "cookie", "pie", "rice", "egg", "fish", "meat", "trout", "venison", "beetroot")),
}
EDGE_MAX = 0.45  # guide, master prompt: the outline's brightness about 15-45%, never black
MATERIALS = ("iron_", "gold", "golden_", "diamond_", "stone_", "wooden_", "wood_", "leather_", "copper_", "bronze_", "flint_", "netherite_")


def kind_of(n):
    if n.endswith("_spawn_egg"):
        return "spawn_egg"  # sized by its animal on purpose (guide)
    if n == "egg":
        return "food"
    for k, (_, keys) in KINDS.items():
        if any(n.endswith(x) or n == x.strip("_") or (x[0] != "_" and x in n) for x in keys):
            return k
    return "other"


def ref(packs, name, pack):
    d = os.path.join(packs, PACKS[pack])
    for cand in (ic.MANUAL.get(name, name), OLD.get(name, "")):
        p = os.path.join(d, cand + ".png")
        if cand and os.path.exists(p):
            return p, cand
    return None, None


def main(ours, packs, out):
    os.makedirs(out, exist_ok=True)
    items = []
    for mod in ("base", "animals"):
        for p in sorted(glob.glob(f"{ours}/mods/{mod}/assets/textures/item/*.png")):
            n = os.path.basename(p)[:-4]
            m = ic.measure(ic.load(p))
            if m:
                items.append(dict(mod=mod, item=n, kind=kind_of(n), path=p, **m))
    # 1. the packs, per kind, over the items they share with us
    summary = []
    for pack in ["ours"] + list(PACKS):
        for kind in ("tool", "armour", "food", "ingot", "gem", "all"):
            ms = []
            for it in items:
                if kind != "all" and it["kind"] != kind:
                    continue
                if pack == "ours":
                    if ref(packs, it["item"], "vanilla")[0]:
                        ms.append(it)
                    continue
                p, cand = ref(packs, it["item"], pack)
                if p:
                    m = ic.measure(ic.tinted(ic.load(p), cand))
                    if m:
                        ms.append(m)
            if len(ms) >= 3:
                summary.append(dict(pack=pack, kind=kind, n=len(ms), **{q: round(st.median(m[q] for m in ms), 3)
                                                                       for q in ("outline", "luma", "sat", "box", "cover", "light")}))
    with open(f"{out}/items_packs_summary.csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(summary[0].keys()))
        w.writeheader()
        w.writerows(summary)
    print("Where our set sits (medians over shared items):")
    print(f"  {'pack':13s} {'kind':7s} {'n':>3s} {'outline':>8s} {'luma':>6s} {'sat':>6s} {'box':>6s} {'light':>7s}")
    for r in summary:
        if r["kind"] in ("all", "tool", "armour"):
            print(f"  {r['pack']:13s} {r['kind']:7s} {r['n']:3d} {r['outline']:8.2f} {r['luma']:6.2f} {r['sat']:6.2f} {r['box']:6.2f} {r['light']:+7.2f}")
    # 2. consistency inside our set
    med = {k: {q: st.median(i[q] for i in items if i["kind"] == k) for q in ("luma", "sat", "outline", "box")}
           for k in set(i["kind"] for i in items)}
    flags = []
    for it in items:
        f = []
        if it["edge"] > EDGE_MAX:
            f.append(f"outline {it['edge']:.2f} bright (guide: 0.15-{EDGE_MAX})")
        elif it["edge"] < 0.04:
            f.append(f"outline {it['edge']:.2f}, near black (guide: never black)")
        mk = med[it["kind"]]
        if it["kind"] in KINDS and abs(it["box"] - mk["box"]) > 0.12:
            f.append(f"fill {it['box']:.2f} vs our {it['kind']} median {mk['box']:.2f}")
        if it["light"] > 0.15:
            f.append(f"lit from the bottom right ({it['light']:+.2f}; check by eye)")
        mat = next((m for m in MATERIALS if it["item"].startswith(m)), None)
        if mat:
            peers = [i["luma"] for i in items if i["item"].startswith(mat) and i["kind"] == it["kind"]]
            mm = st.median(peers) if len(peers) >= 3 else None
            if mm is not None and abs(it["luma"] - mm) > 0.10:
                f.append(f"luma {it['luma']:.2f} vs our {mat.strip('_')} {it['kind']} median {mm:.2f}")
        if f:
            flags.append(dict(it, flags="; ".join(f), n_flags=len(f)))
    with open(f"{out}/items_consistency.csv", "w", newline="") as fh:
        keys = [k for k in flags[0] if k != "path"]
        w = csv.DictWriter(fh, fieldnames=keys, extrasaction="ignore")
        w.writeheader()
        w.writerows(flags)
    print(f"\n{len(items)} of our items; {len(flags)} break our guide or our set:")
    for r in sorted(flags, key=lambda r: -r["n_flags"]):
        print(f"  {r['mod']}:{r['item']:26s} [{r['kind']}] {r['flags']}")
    return items, flags, summary


if __name__ == "__main__":
    main(*sys.argv[1:4])
