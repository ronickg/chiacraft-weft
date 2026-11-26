"""Turn the vanilla comparison's flags into grade-table rows: for each tile, the hue turn, saturation and
gain (on top of its current row in grade.toml) that bring the graded tile into the packs' own range,
or half way for the tiles that keep their own identity. Writes mods/base/source/blocks/grade_proposed.toml,
which `regrade.py --all --with <it>` lays over grade.toml to preview; approved rows move into grade.toml.

    python3 notes/blocks/vanilla_proposals.py <packs dir>
"""
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import regrade as rg  # noqa: E402
import vanilla_compare as vc  # noqa: E402

OUT = os.path.join(rg.ROOT, "mods", "base", "source", "blocks", "grade_proposed.toml")
# The vanilla twins that drift (fix now) and the identity tiles moved half way. Leaves, grass and ferns
# wait for Lighting's Colour 8; water is a tint question; gold ore needs a redraw; glass keeps its frame;
# the observer and piston model textures go with a later redstone pass.
FIX = ["kelp", "lily_pad", "snow", "wool_white", "gravel", "crafting_table_side", "crafting_table_front", "granite", "podzol",
       "coarse_dirt", "moss_block_top", "glowstone", "slime_block", "ladder", "redstone_lamp_on", "lily_of_the_valley", "sapling_oak",
       "mushroom_red", "mushroom_brown", "redstone_ore", "obsidian", "iron_block", "log_acacia_top", "piston_side", "piston_top", "lever"]
HALF = ["animals:termite_mound_side", "animals:termite_mound_upper_side", "ochre_sandstone_side", "ochre_sandstone_top", "planks_mesquite"]
TOL = dict(hue=15.0, luma=0.05, sat=0.10)


def target_for(m, rm, half):
    t = {}
    for q in ("luma", "sat"):
        lo, hi = min(r[q] for r in rm), max(r[q] for r in rm)
        if m[q] < lo - TOL[q] or m[q] > hi + TOL[q]:
            edge = lo if m[q] < lo else hi
            t[q] = (m[q] + edge) / 2 if half else edge
    if m["sat"] > 0.12 and min(r["sat"] for r in rm) > 0.12:
        hs = [r["hue"] for r in rm]
        if min(vc.hdist(m["hue"], h) for h in hs) > TOL["hue"]:
            near = min(hs, key=lambda h: vc.hdist(m["hue"], h))
            step = ((near - m["hue"] + 180) % 360) - 180
            t["hue_step"] = (step / 2) if half else (step - np.sign(step) * 5)
    return t


def solve(key, packs):
    mod, n = (key.split(":") if ":" in key else ("base", key))
    vname, _ = vc.vanilla_name(n)
    tint = vc.tint_for(vname)
    refs = [f"{d}/{vname}.png" for d in vc.refs_in(packs).values() if os.path.exists(f"{d}/{vname}.png")]
    rm = [vc.measure(vc.load(p), tint) for p in refs]
    t = rg.load_table([rg.TABLE])
    kn = rg.knobs(t, key)
    gd = rg.game_dir(mod)
    sd = rg.src_dir(mod)
    src = rg.load(os.path.join(sd if os.path.exists(os.path.join(sd, n + ".png")) else gd, n + ".png"))
    k = rg.contrast_k(rg.grade(src, kn), kn["spread"]) if kn["spread"] is not None else (kn["contrast"] if kn["contrast"] != 1 else None)
    now = vc.measure(rg.grade(src, kn, k), tint)
    tg = target_for(now, rm, key in HALF)
    row = {}
    if "hue_step" in tg:
        kn["hue"] = round(kn["hue"] + tg["hue_step"], 1)
        row["hue"] = kn["hue"]
    if "sat" in tg:
        for _ in range(6):
            m = vc.measure(rg.grade(src, kn, k), tint)
            kn["saturation"] *= tg["sat"] / max(m["sat"], 1e-3)
        row["saturation"] = round(kn["saturation"], 3)
    if "luma" in tg:
        for _ in range(6):
            m = vc.measure(rg.grade(src, kn, k), tint)
            kn["gain"] *= tg["luma"] / max(m["luma"], 1e-3)
        row["gain"] = round(kn["gain"], 3)
    after = vc.measure(rg.grade(src, kn, k), tint)
    return row, now, after, vname


def main(packs):
    lines = ["# Proposed rows from the vanilla comparison (11-04), on top of grade.toml. Preview:",
             "#   python3 notes/blocks/regrade.py --all --with mods/base/source/blocks/grade_proposed.toml --dry --preview out.png",
             "# Rows Ronald approves move into grade.toml. Each comment: before -> after (luma, saturation, hue), and its vanilla match.", ""]
    for key in FIX + HALF:
        row, now, after, vname = solve(key, packs)
        if not row:
            continue
        name = f'"{key}"' if ":" in key else key
        lines.append(f"[tiles.{name}]  # luma {now['luma']:.2f}->{after['luma']:.2f}, sat {now['sat']:.2f}->{after['sat']:.2f}, "
                     f"hue {now['hue']:.0f}->{after['hue']:.0f}; ~ {vname}{' (half way)' if key in HALF else ''}")
        for k, v in row.items():
            lines.append(f"{k} = {v}")
        lines.append("")
    open(OUT, "w").write("\n".join(lines))
    print(OUT)


if __name__ == "__main__":
    main(sys.argv[1])
