#!/usr/bin/env python3
"""Rebuilds the wyvern: lays out, paints and animates the model (wyvern.py, into out/) and exports
the game's files into the mod (`../../assets`): models/wyvern.geo.json,
animations/wyvern[_rest].animation.json and textures/entity/wyvern/wyvern.png. Blockbench isn't
needed for this.

    ../dog/chihuahua/.venv/bin/python build.py     (or any Python with numpy, pillow and scipy)

Commit what it writes under `../../assets`.
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
from house import SCALE  # noqa: E402  (the house style: 64 texels a block)

# Which animations go in which file: the engine plays `idle`, `walk` and `run` by itself (walk
# and run by speed); `breathe`, `fly` and `glide` are there for code to play. Lying down (asleep,
# knocked out) is its own look, whose file holds it in `idle`.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", "breathe": "breathe", "fly": "fly", "glide": "glide"},
         "_rest": {"idle": "rest"}}

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "wyvern.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    from to_bedrock import export  # noqa: E402
    from ground import gait, joints  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    for anim in ("walk", "run"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, ["foot_L", "foot_R", "thumbclaw_L", "thumbclaw_R"]).items()
                if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "wyvern", SCALE, "wyvern/wyvern", pose_files=POSES)
    # Its coat layers, for coats painted from genes in the game (`src/coats.rs`).
    import shutil  # noqa: E402
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "wyvern"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
