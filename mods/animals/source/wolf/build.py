#!/usr/bin/env python3
"""Rebuilds the wolf: paints the model and its animations (wolf.py, into out/) and exports the
game's files into the mod (`../../assets`): models/wolf.geo.json,
animations/wolf[_lie].animation.json and textures/entity/wolf/wolf.png, and its coat layers
into `../../genetics/layers/wolf`. Blockbench isn't needed for this.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)

Commit what it writes under `../../assets` and `../../genetics/layers/wolf`.
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
# The behaviour clips (`mods/animals/src/anim.rs`) go in the standing file too, by their own names.
CLIPS = ["pounce", "bite", "stare", "sniff", "drink", "look", "scratch", "stretch", "sleep", "stalk", "alarm", "howl", "play", "groom",
         "nuzzle", "flinch", "death", "approach"]
# Asleep it lies in its look (`rest.rs` settle): the sleep clip goes in the lying file too.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", **{c: c for c in CLIPS}},
         "_lie": {"idle": "lie", "walk": "walk", "sleep": "sleep"}, "_sit": {"idle": "sit", "walk": "walk"}}
FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
AXIS = ["neck", "skull", "chest", "hips", "tail_"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "wolf.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    from to_bedrock import export  # noqa: E402
    from ground import feet_low, gait, grounded, joints, pokes  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    poking = pokes(spec, AXIS)  # no neck corner through the head or chest, no brush segment through the one before
    assert not poking, f"poking through: {sorted(poking)}"
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another
    # (linecheck; flush, or two texels clear).
    from linecheck import boxes_from_spec, lines  # noqa: E402
    from PIL import Image  # noqa: E402
    import numpy as np  # noqa: E402
    thin = [h for h in lines(boxes_from_spec(spec), np.array(Image.open(os.path.join(OUT, "texture.png")).convert("RGBA")),
                             allow=()) if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    for anim in ("walk", "run", "stalk", "approach"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    # The gaits written as angles stand on their feet: the root's height from the feet, frame by frame
    # (a run's bounce kept), and checked: no foot sinks into the ground or floats over it.
    for clip, bounds in (("stalk", False), ("approach", False)):
        spec["animations"][clip] = grounded(spec, spec["animations"][clip], FEET, bounds)
        low = min(feet_low(spec, spec["animations"][clip], FEET))
        assert abs(low) < 0.3, f"{clip}'s feet at {low:.2f}, not on the ground"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "wolf", SCALE, "wolf/wolf", pose_files=POSES)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "wolf"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
