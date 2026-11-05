#!/usr/bin/env python3
"""Rebuilds the fox: paints the model and its animations (fox.py, into out/) and exports the game's
files into the mod (`../../assets`): models/fox.geo.json, animations/fox[_lie].animation.json and
textures/entity/fox/fox.png, and its coat layers into `../../genetics/layers/fox`. Blockbench
isn't needed for this. The red fox and the Arctic fox share it.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)

Commit what it writes under `../../assets` and `../../genetics/layers/fox`.
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
# The behaviour clips (`mods/animals/src/anim.rs`, `../_clips/canid.py`) go in the standing file too,
# by their own names; asleep it lies in its look, so the sleep clip goes in the lying file too.
CLIPS = ["stare", "sniff", "drink", "look", "scratch", "stretch", "sleep", "stalk", "alarm", "play", "groom", "nuzzle", "flinch", "death",
         "approach"]
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", "pounce": "pounce", "bite": "bite", **{c: c for c in CLIPS}},
         "_lie": {"idle": "lie", "walk": "walk", "sleep": "sleep"}}
FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
AXIS = ["neck", "skull", "chest", "hips", "tail_"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "fox.py")], check=True)
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
    # Meant: whole parts under two texels across, not strips of a bigger box: the back shanks (one texel)
    # whose tops enter the thighs, the jaw (one texel) and the muzzle's sides (one texel tall) whose
    # backs enter the skull, the muzzle's front round the nose (two texels tall). Open: the cheeks
    # beside the muzzle are one whole texel (a face four wide, the muzzle two); two would need a head
    # half again as wide (skull.south).
    from linecheck import boxes_from_spec, lines  # noqa: E402
    from PIL import Image  # noqa: E402
    import numpy as np  # noqa: E402
    thin = [h for h in lines(boxes_from_spec(spec), np.array(Image.open(os.path.join(OUT, "texture.png")).convert("RGBA")),
                             allow=("leg_BL_lower", "leg_BR_lower", "jaw", "muzzle.west", "muzzle.east", "muzzle.south", "skull.south")) if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    for anim in ("walk", "run", "stalk", "approach"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    # The gaits written as angles stand on their feet: the root's height from the feet, frame by
    # frame, and checked: no foot sinks into the ground or floats over it.
    for clip in ("stalk", "approach"):
        spec["animations"][clip] = grounded(spec, spec["animations"][clip], FEET)
        low = min(feet_low(spec, spec["animations"][clip], FEET))
        assert abs(low) < 0.3, f"{clip}'s feet at {low:.2f}, not on the ground"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "fox", SCALE, "fox/fox", pose_files=POSES)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "fox"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
