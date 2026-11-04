#!/usr/bin/env python3
"""Research round 16: BENCH lines -> per-pass GPU ms tables, and each effect's cost as (all - no_<fx>).
usage: parse.py bench.txt [more bench.txt...]   (one file per window size)"""
import re, sys
PARTS = ["upload", "compute", "shadows", "world", "translucent", "effects", "ui", "rest"]
def read(path):
    rows = {}
    for l in open(path):
        m = re.search(r"BENCH (\S+) \| (\d+) frames, (\d+) fps.*?gpu ([\d.]+) ms: (.*?) \| per frame: (\d+) draws", l)
        if not m: continue
        label, frames, fps, gpu, parts, draws = m.groups()
        p = dict((k, float(v)) for k, v in re.findall(r"(\w+) ([\d.]+)", parts))
        rows[label] = dict(fps=int(fps), gpu=float(gpu), draws=int(draws), **p)
    return rows
for path in sys.argv[1:]:
    rows = read(path)
    print(f"\n## {path}\n")
    print("| bench | fps | gpu | " + " | ".join(PARTS) + " | draws |")
    print("|---|" + "---|" * (len(PARTS) + 3))
    for k, r in rows.items():
        print(f"| {k} | {r['fps']} | {r['gpu']:.2f} | " + " | ".join(f"{r.get(p,0):.2f}" for p in PARTS) + f" | {r['draws']} |")
    scenes = sorted({k.rsplit('_all', 1)[0] for k in rows if k.endswith('_all')})
    print("\n| scene | all gpu | drift (all2-all) | reflect | contact | volume | shafts | bloom | all fx | far map (near-only) | shadows total |")
    print("|---|---|---|---|---|---|---|---|---|---|---|")
    for s in scenes:
        a = rows[f"{s}_all"]; a2 = rows.get(f"{s}_all2", a)
        base = (a["gpu"] + a2["gpu"]) / 2
        d = lambda k: f"{base - rows[k]['gpu']:+.2f}" if k in rows else "-"
        print(f"| {s} | {base:.2f} | {a2['gpu']-a['gpu']:+.2f} | {d(s+'_no_reflect')} | {d(s+'_no_contact')} | {d(s+'_no_volume')} | {d(s+'_no_shafts')} | {d(s+'_no_bloom')} | {d(s+'_no_fx')} | {d(s+'_shadows_near')} | {d(s+'_shadows_off')} |")
    for k in ("far_scale50", "far_lod_off", "still_far", "move_far"):
        if k in rows: print(f"\n{k}: gpu {rows[k]['gpu']:.2f}, draws {rows[k]['draws']}")
