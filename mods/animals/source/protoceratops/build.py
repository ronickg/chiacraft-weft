#!/usr/bin/env python3
"""Rebuilds Protoceratops: paints the model and its animations (protoceratops.py, into out/) and exports the game's
files into the mod (`../../assets`): models/protoceratops.geo.json, animations/protoceratops[_lie].animation.json and
textures/entity/protoceratops/protoceratops.png, and its coat layers into `../../genetics/layers/protoceratops`. Blockbench
isn't needed for this.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)

Commit what it writes under `../../assets` and `../../genetics/layers/protoceratops`.
"""

import json
import os
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", ".."))
SKILL = os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts")
OUT = os.path.join(HERE, "out")
ASSETS = os.path.join(HERE, "..", "..", "assets")

sys.path.insert(0, SKILL)
from house import SCALE  # noqa: E402  the house density: 64 texels a block

# Which animations go in which file: the engine plays `idle`, `walk` and `run` by itself; lying
# down is a look of its own.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run"},
         "_lie": {"idle": "lie", "walk": "walk"}}
FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
AXIS = ["neck", "skull", "chest", "hips", "tail_"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "protoceratops.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    from to_bedrock import export  # noqa: E402
    from ground import gait, joints, pokes  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    poking = pokes(spec, AXIS)  # no neck corner through the head or chest
    assert not poking, f"poking through: {sorted(poking)}"
    for anim in ("walk", "run"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "protoceratops", SCALE, "protoceratops/protoceratops", pose_files=POSES)
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another, checked
    # on the model as the game has it (`linecheck.py` on the exported geo). Meant: the hind feet's tops
    # round the shank, a joint's rim (the long foot reaches forward, so linecheck doesn't see it as one).
    from linecheck import density_of, lines, load  # noqa: E402
    boxes, tex = load(os.path.join(ASSETS, "models", "protoceratops.geo.json"), os.path.join(ASSETS, "textures", "entity", "protoceratops", "protoceratops.png"))
    thin = [h for h in lines(boxes, tex, density_of(boxes), allow=("leg_BL_paw#0.up", "leg_BR_paw#0.up")) if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "protoceratops"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
