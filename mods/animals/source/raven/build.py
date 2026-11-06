#!/usr/bin/env python3
"""Rebuilds the raven: paints its two models and their animations (raven.py, into out/) and exports
the game's files into the mod (`../../assets`): models/raven[_fly].geo.json,
animations/raven[_fly].animation.json and textures/entity/raven/raven.png (one texture for both),
and its coat layers into `../../genetics/layers/raven`. Blockbench isn't needed for this.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)

Commit what it writes under `../../assets` and `../../genetics/layers/raven`.
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

sys.path.insert(0, os.path.join(HERE, "..", "bird"))
import bird  # noqa: E402  the birds' build: the behaviour clips and their checks

# On the ground the engine plays `idle`, `walk` and `run` by itself; flying is a look of its own
# (`raven_fly`, put on by `raven.rs`), its wings beating whatever it does. The behaviour clips
# (`mods/animals/src/anim.rs`, `bird.CLIPS`) go in the standing file, by their own names.
GROUND = {"": {"idle": "idle", "walk": "walk", "run": "run", **{c: c for c in bird.CLIPS}}}
FLY = {"": {"idle": "flap", "walk": "flap", "run": "flap"}}
FEET = ["toes_l", "toes_r"]
AXIS = ["neck", "skull", "chest", "hips", "tail"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "raven.py")], check=True)
    for name in ("spec.json", "spec_fly.json"):
        subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, name)], check=True)
    from to_bedrock import export  # noqa: E402
    from ground import gait, joints, pokes  # noqa: E402

    ground = json.load(open(os.path.join(OUT, "spec.json")))
    fly = json.load(open(os.path.join(OUT, "spec_fly.json")))
    from linecheck import boxes_from_spec, lines  # noqa: E402
    import numpy as np  # noqa: E402
    from PIL import Image  # noqa: E402

    tex = np.array(Image.open(os.path.join(OUT, "texture.png")).convert("RGBA"))
    for spec in (ground, fly):
        # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another.
        thin = [h for h in lines(boxes_from_spec(spec), tex) if h["kind"] == "line"]
        assert not thin, f"{spec['name']}: thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
        loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
        assert not loose, f"{spec['name']}: loose joints: {loose}"
        poking = pokes(spec, AXIS)  # no neck corner through the head or the back
        assert not poking, f"{spec['name']}: poking through: {sorted(poking)}"
    for anim in ("walk", "run"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(ground, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    # The behaviour clips: held ones and the approach loop, the rest once; the feet on the ground (on one
    # foot, or taking off and landing), nothing in it, the fall on its lowest point, the approach going
    # forward, the eyes shut asleep and dead, no wing or foot swung into the body.
    bird.check_clips(ground, bird.config(dict(name="raven", tilt=-18, neck_share=0.4, idle="look", walk="walk")))
    texture = os.path.join(OUT, "texture.png")
    made = export(ground, texture, ASSETS, "raven", SCALE, "raven/raven", pose_files=GROUND)
    made += export(fly, texture, ASSETS, "raven_fly", SCALE, "raven/raven", pose_files=FLY)
    # Its coat layers, for plumage painted from genes in the game (`core/src/coats.rs`).
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "raven"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
