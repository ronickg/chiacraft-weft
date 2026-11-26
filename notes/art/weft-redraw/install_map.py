"""Map every redrawn file to the Weft-added texture it replaces; print what is left over.
    install_map.py [--apply]"""
import os, sys, shutil
from PIL import Image
ROOT = "../../.."
added = [l.strip() for l in open("added.txt") if l.strip()]
outs = {"out_potions": "item", "out_items": "item", "out_fx": "mob_effect", "out_gui": "gui",
        "out_blocks": "block", "out_mach": "block"}
plan, unmatched = [], []
for d, kind in outs.items():
    for dp, _, fs in os.walk(d):
        for f in fs:
            if not f.endswith(".png"): continue
            rel = os.path.relpath(os.path.join(dp, f), d)
            hits = [a for a in added if a.endswith(f"/textures/{kind}/{rel}") or (kind == "gui" and a.endswith("/" + rel))]
            if len(hits) == 1: plan.append((os.path.join(dp, f), hits[0]))
            else: unmatched.append((os.path.join(dp, f), hits))
sh = [a for a in added if a.endswith("/item/shield.png") or a.endswith("shield.png")]
plan.append(("shield_tex.png", sh[0]))
done = {b for _, b in plan}
print("planned", len(plan))
for u in unmatched: print("UNMATCHED", u)
for a in added:
    if a not in done: print("LEFT", a)
for s, dst in plan:
    a, b = Image.open(s).size, Image.open(os.path.join(ROOT, dst)).size
    if a != b and not (a[0] == 2*b[0] and a[1] == 2*b[1]): print("SIZE", s, a, dst, b)
if "--apply" in sys.argv:
    for s, dst in plan: shutil.copy(s, os.path.join(ROOT, dst))
    print("applied")
