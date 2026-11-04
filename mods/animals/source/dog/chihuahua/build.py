#!/usr/bin/env python3
"""Rebuilds the chihuahua: paints the model and its animations (tools/blocky.py, into
out/d_blocky/) and exports the game's files into the mod (`../../assets`): models/chihuahua.geo.json
and chihuahua_pup.geo.json, animations/chihuahua[_pup][_sit|_lie].animation.json and
textures/entity/chihuahua/chihuahua.png. Blockbench isn't needed for this.

    python3 -m venv .venv && .venv/bin/pip install -r requirements.txt   # once
    .venv/bin/python build.py

Commit what it writes under `../../assets`.
"""

import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", "..", ".."))
EXPORT = os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts")
OUT = os.path.join(HERE, "out", "d_blocky")
ASSETS = os.path.join(HERE, "..", "..", "..", "assets")

SCALE = 0.122  # model pixels per unit: a real chihuahua, about 20 cm at the shoulder (15-23 cm)
PUP, PUP_HEAD = 0.5, 1.3  # a puppy: half the size, a bigger head

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "tools", "blocky.py")], check=True)
    # No flicker at rest or in any keyframe (the skill's check).
    subprocess.run([sys.executable, os.path.join(EXPORT, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    sys.path.insert(0, EXPORT)
    from to_bedrock import export  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    from ground import gait, joints  # noqa: E402
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    for anim in ("walk", "run"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    from to_bedrock import POSE_FILES  # noqa: E402
    poses = {**POSE_FILES, "": {**POSE_FILES[""], "bite": "bite"}}  # its bite, for code to play
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "chihuahua", SCALE, "chihuahua/chihuahua", PUP, PUP_HEAD,
                  pose_files=poses)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    import shutil  # noqa: E402
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "..", "genetics", "layers", "chihuahua"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
