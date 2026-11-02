#!/usr/bin/env python3
"""Rebuilds the raptor: paints the model and its animations (raptor.py, into out/) and exports the game's
files into the mod (`../../assets`): models/raptor.geo.json, animations/raptor[_rest].animation.json
and textures/entity/raptor/raptor.png, and osmolskae.png (the second species' coat). Blockbench isn't needed for this.

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

SCALE = 0.3  # model pixels per unit: about half a block at the hip, 1.7 long (a real Velociraptor: 0.5 m, 1.8-2 m)
# Which animations go in which file: the engine plays `idle` and `walk` by itself, a pose's
# file holds it in `idle`; `run` and `meow` are there for code to play.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", "screech": "screech"},
         "_rest": {"idle": "rest", "walk": "walk"}}

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "raptor.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    sys.path.insert(0, SKILL)
    from to_bedrock import export  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    from ground import gait, joints, pokes  # noqa: E402
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    # Along the body's axis nothing pokes out of its neighbor in any pose (a neck's corner through
    # the head, a tail segment's through the one before).
    poking = pokes(spec, ["neck", "skull", "chest", "hips", "tail_"])
    assert not poking, "poking through: " + "; ".join(f"{a} out of {b} by {d:.2f} ({w[0]})" for (a, b), (d, w) in poking.items())
    for anim in ("walk", "run"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, ["toes_L", "toes_R"]).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "raptor", SCALE, "raptor/raptor", pose_files=POSES)
    # Velociraptor osmolskae wears the same model in its own colors.
    import shutil  # noqa: E402
    shutil.copy(os.path.join(OUT, "osmolskae.png"), os.path.join(ASSETS, "textures", "entity", "raptor", "osmolskae.png"))
    print("wrote", made, "and textures/entity/raptor/osmolskae.png into", os.path.normpath(ASSETS))
