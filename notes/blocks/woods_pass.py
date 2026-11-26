"""The wood pass (Planner, 11-05): every wood's log, log top, planks and stripped log into the packs' band
(vanilla 1.21.4, Faithful 32x, Pixel Perfection CE), the way the ground was done. A wood with a vanilla
twin moves to the band's edge; one without moves half way toward its nearest relative's band; the tiles
Ronald approved as identity (sequoia bark and planks, mesquite planks) stay. Prints grade rows (gain and
saturation, on top of each tile's current row) for grade.toml.

    python3 -E -P notes/blocks/woods_pass.py <packs dir> [woods ...]
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import regrade as rg  # noqa: E402
import vanilla_compare as vc  # noqa: E402

TWIN = {"oak": "oak", "birch": "birch", "spruce": "spruce", "acacia": "acacia"}
RELATIVE = {"mesquite": "acacia", "sequoia": "spruce", "dogwood": "cherry", "acuri_palm": "jungle", "moriche_palm": "jungle",
            "brazil_nut": "jungle", "cacao": "jungle", "andiroba": "jungle", "matamata": "jungle", "ipe": "dark_oak"}
IDENTITY = {"log_sequoia", "planks_sequoia", "planks_mesquite"}
PARTS = [("log_{w}", "{v}_log"), ("log_{w}_top", "{v}_log_top"), ("planks_{w}", "{v}_planks"),
         ("log_{w}_stripped", "stripped_{v}_log"), ("log_{w}_stripped_top", "stripped_{v}_log_top")]


def band(packs, vname):
    rs = [vc.measure(vc.load(f"{d}/{vname}.png"), None) for d in vc.refs_in(packs).values() if os.path.exists(f"{d}/{vname}.png")]
    return {q: (min(r[q] for r in rs), max(r[q] for r in rs)) for q in ("luma", "sat")} if rs else None


def solve(name, b, half):
    t = rg.load_table([rg.TABLE])
    kn = rg.knobs(t, name)
    p = os.path.join(rg.src_dir("base"), name + ".png")
    src = rg.load(p if os.path.exists(p) else os.path.join(rg.game_dir("base"), name + ".png"))
    k = rg.contrast_k(rg.grade(src, kn), kn["spread"]) if kn["spread"] is not None else (kn["contrast"] if kn["contrast"] != 1 else None)
    now = vc.measure(rg.grade(src, kn, k), None)
    target = {}
    for q in ("luma", "sat"):
        lo, hi = b[q]
        if now[q] < lo or now[q] > hi:
            edge = lo if now[q] < lo else hi
            target[q] = (now[q] + edge) / 2 if half else edge
    row = {}
    if "sat" in target:
        for _ in range(8):
            m = vc.measure(rg.grade(src, kn, k), None)
            kn["saturation"] *= target["sat"] / max(m["sat"], 1e-3)
        row["saturation"] = round(kn["saturation"], 3)
    if "luma" in target:
        for _ in range(8):
            m = vc.measure(rg.grade(src, kn, k), None)
            kn["gain"] *= target["luma"] / max(m["luma"], 1e-3)
        row["gain"] = round(kn["gain"], 3)
    after = vc.measure(rg.grade(src, kn, k), None)
    return now, after, row


def main(packs, woods):
    for w in woods:
        v = TWIN.get(w) or RELATIVE[w]
        half = w not in TWIN
        for ours, theirs in PARTS:
            name, vname = ours.format(w=w), theirs.format(v=v)
            if name in IDENTITY or not os.path.exists(os.path.join(rg.game_dir("base"), name + ".png")):
                continue
            b = band(packs, vname)
            if not b:
                continue
            now, after, row = solve(name, b, half)
            if row:
                print(f"[tiles.{name}]  # luma {now['luma']:.2f}->{after['luma']:.2f}, sat {now['sat']:.2f}->{after['sat']:.2f}; "
                      f"~ {vname} ({b['luma'][0]:.2f}-{b['luma'][1]:.2f}, {b['sat'][0]:.2f}-{b['sat'][1]:.2f}){' half way' if half else ''}")
                for kk, vv in row.items():
                    print(f"{kk} = {vv}")
                print()


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2:] or list(TWIN) + list(RELATIVE))
