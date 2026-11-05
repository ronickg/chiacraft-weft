#!/usr/bin/env python3
"""Rebuilds the Utahraptor: paints the model and its animations (utahraptor.py, into out/) and exports the
game's files into the mod (`../../assets`): models/utahraptor.geo.json,
animations/utahraptor[_rest].animation.json and textures/entity/utahraptor/utahraptor.png. Blockbench
isn't needed for this.

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

SCALE = 0.83  # model pixels per unit: 1.5 blocks at the hip, about 4.7 long (a real Utahraptor: 1.5 m, 4.7-6 m)
# Which animations go in which file: the engine plays `idle` and `walk` by itself, a pose's
# file holds it in `idle`; `run` and `meow` are there for code to play.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", "screech": "screech", "bite": "bite", "kick": "kick"},
         "_rest": {"idle": "rest", "walk": "walk"}}

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "utahraptor.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    sys.path.insert(0, SKILL)
    from to_bedrock import export  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    from ground import gait, joints, pokes  # noqa: E402
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another,
    # measured at the size the game draws it (a layout unit here is 4 * SCALE of linecheck's).
    from linecheck import boxes_from_spec, lines  # noqa: E402
    from PIL import Image  # noqa: E402
    import numpy as np  # noqa: E402
    boxes = [dict(b, unit=4 * SCALE) for b in boxes_from_spec(spec)]
    thin = [h for h in lines(boxes, np.array(Image.open(os.path.join(OUT, "texture.png")).convert("RGBA"))) if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    # Along the body's axis nothing pokes out of its neighbor in any pose (a neck's corner through
    # the head, a tail segment's through the one before).
    poking = pokes(spec, ["neck", "skull", "chest", "hips", "tail_"])
    assert not poking, "poking through: " + "; ".join(f"{a} out of {b} by {d:.2f} ({w[0]})" for (a, b), (d, w) in poking.items())
    for anim in ("walk", "run"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, ["toe3_L", "toe3_R"]).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "utahraptor", SCALE, "utahraptor/utahraptor", pose_files=POSES)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    import shutil  # noqa: E402
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "utahraptor"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
