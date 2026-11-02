#!/usr/bin/env python3
"""Rebuilds the cat: paints the model and its animations (cat.py, into out/) and exports the game's
files into the mod (`../../assets`): models/cat.geo.json, animations/cat[_sit|_lie].animation.json
and textures/entity/cat/cat.png. Blockbench isn't needed for this.

    ../dog/chihuahua/.venv/bin/python build.py     (or any Python with requirements.txt installed)

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

SCALE = 0.174  # model pixels per unit: a real house cat, about 24 cm at the shoulder (23-25 cm)
# Which animations go in which file: the engine plays `idle` and `walk` by itself, a pose's
# file holds it in `idle`; `run` and `meow` are there for code to play.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", "meow": "meow", "swipe": "swipe", "bite": "bite"},
         "_sit": {"idle": "sit", "walk": "walk"},
         "_lie": {"idle": "lie", "walk": "walk"}}

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "cat.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    sys.path.insert(0, SKILL)
    from to_bedrock import export  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    from ground import gait, joints  # noqa: E402
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    for anim in ("walk", "run"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "cat", SCALE, "cat/cat", pose_files=POSES)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    import shutil  # noqa: E402
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "cat"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
