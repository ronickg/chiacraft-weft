#!/usr/bin/env python3
"""Rebuilds the deer: paints the model and its animations (deer.py, into out/) and exports the game's
files into the mod (`../../assets`): models/deer.geo.json, animations/deer[_lie].animation.json and
textures/entity/deer/deer.png, and its coat layers into `../../genetics/layers/deer`. Blockbench
isn't needed for this. The white-tailed deer and the red deer share it.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)

Commit what it writes under `../../assets` and `../../genetics/layers/deer`.
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
CLIPS = ["stare", "graze", "drink", "sniff", "look", "scratch", "stretch", "sleep", "alarm", "rear", "play", "groom", "nuzzle", "flinch",
         "death", "watch", "approach", "charge"]
# Asleep it lies in its look (`wild.rs` settle): the sleep clip goes in the lying file too.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", **{c: c for c in CLIPS}},
         "_lie": {"idle": "lie", "walk": "walk", "sleep": "sleep"}}
FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
AXIS = ["neck", "skull", "chest", "hips"]  # the tail is a flap hinged on the rump (the tongue rule), checked by eye

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "deer.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    from to_bedrock import export  # noqa: E402
    from ground import feet_low, gait, grounded, joints, pokes  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    poking = pokes(spec, AXIS)  # no neck corner through the head or chest
    assert not poking, f"poking through: {sorted(poking)}"
    for anim in ("walk", "run", "approach", "charge"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    # The gaits written as angles stand on their feet: the root's height from the feet, frame by frame
    # (a run's bounce kept), and checked: no foot sinks into the ground or floats over it.
    for clip, bounds in (("approach", False), ("charge", True)):
        spec["animations"][clip] = grounded(spec, spec["animations"][clip], FEET, bounds)
        low = min(feet_low(spec, spec["animations"][clip], FEET))
        assert abs(low) < 0.3, f"{clip}'s feet at {low:.2f}, not on the ground"
    # The standing clips keep a hoof on the ground in every frame (rearing on the hind ones; the pronk
    # leaves it and lands); none sinks.
    for clip in ("stare", "graze", "drink", "sniff", "look", "scratch", "stretch", "alarm", "rear", "groom", "nuzzle", "flinch", "watch"):
        low = feet_low(spec, spec["animations"][clip], FEET)
        assert min(low) > -0.3 and max(low) < 0.3, f"{clip}'s feet from {min(low):.2f} to {max(low):.2f}, not on the ground"
    spec["animations"]["play"] = grounded(spec, spec["animations"]["play"], FEET, True)  # the pronk lands on the ground
    assert abs(min(feet_low(spec, spec["animations"]["play"], FEET))) < 0.3, "the pronk lands in the ground"
    # Dying, it rolls onto its side: mid-roll its legs swing out below where it ends up lying (11 units
    # into the ground), so each frame rests on its lowest point (the fitted drop in `deer.py` is only
    # the last frame's).
    everything = [e["name"] for e in spec["elements"]]
    spec["animations"]["death"] = grounded(spec, spec["animations"]["death"], everything)
    low = feet_low(spec, spec["animations"]["death"], everything)
    assert min(low) > -0.3 and max(low) < 0.3, f"dying, its lowest point from {min(low):.2f} to {max(low):.2f}"
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another
    # (linecheck; flush, or two texels clear).
    from linecheck import boxes_from_spec, lines  # noqa: E402
    from PIL import Image  # noqa: E402
    import numpy as np  # noqa: E402
    thin = [h for h in lines(boxes_from_spec(spec), np.array(Image.open(os.path.join(OUT, "texture.png")).convert("RGBA")))
            if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "deer", SCALE, "deer/deer", pose_files=POSES)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "deer"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
