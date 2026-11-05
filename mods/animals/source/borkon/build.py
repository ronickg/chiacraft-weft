#!/usr/bin/env python3
"""Rebuilds Borkon: paints the model and its clips (borkon.py, into out/) and exports the game's
files into the mod (`../../assets`): models/borkon.geo.json, animations/borkon.animation.json
(idle, walk and run, and the fight's clips by name) and textures/entity/borkon/borkon.png.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)
"""

import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", ".."))
SKILL = os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts")
OUT = os.path.join(HERE, "out")
ASSETS = os.path.join(HERE, "..", "..", "assets")

sys.path.insert(0, SKILL)
sys.path.insert(0, HERE)
from house import SCALE  # noqa: E402  the house density: 32 texels a block

FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
AXIS = ["neck", "skull", "chest", "hips"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "borkon.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    from to_bedrock import export  # noqa: E402
    from ground import gait, joints, pokes  # noqa: E402
    from linecheck import boxes_from_spec, lines  # noqa: E402
    from borkon import CLIPS  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    loose = joints(spec)
    assert not loose, f"loose joints: {loose}"
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another.
    
    from PIL import Image  # noqa: E402
    import numpy as np  # noqa: E402
    thin = [h for h in lines(boxes_from_spec(spec), np.array(Image.open(os.path.join(OUT, "texture.png")).convert("RGBA")),
                             allow=()) if h["kind"] in ("line", "thin")]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    poking = pokes(spec, AXIS)
    assert not poking, f"poking through: {sorted(poking)}"
    for anim in ("walk", "run"):
        back = {f: v for f, v in gait(spec, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    poses = {"": {"idle": "idle", "walk": "walk", "run": "run", **{c: c for c in CLIPS}}}
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "borkon", SCALE, "borkon/borkon", pose_files=poses)
    print("wrote", made, "into", os.path.normpath(ASSETS))
