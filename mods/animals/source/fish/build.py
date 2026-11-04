#!/usr/bin/env python3
"""Rebuilds the fish: paints the model and its animations (fish.py, into out/) and exports the game's
files into the mod (`../../assets`): models/fish.geo.json, animations/fish[_flop].animation.json and
textures/entity/fish/fish.png, and its coat layers into `../../genetics/layers/fish`. Blockbench
isn't needed for this. The brown trout and the pacu share it.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)

Commit what it writes under `../../assets` and `../../genetics/layers/fish`.
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

# Which animations go in which file: the engine plays `idle`, `walk` and `run` by itself; stranded
# on land is a look of its own.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run"},
         "_flop": {"idle": "flop", "walk": "flop"}}
AXIS = ["head", "body", "tail_"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "fish.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    from to_bedrock import export  # noqa: E402
    from ground import joints, pokes  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    poking = pokes(spec, AXIS)  # no tail part through the body or the one before
    assert not poking, f"poking through: {sorted(poking)}"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "fish", SCALE, "fish/fish", pose_files=POSES)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "fish"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
