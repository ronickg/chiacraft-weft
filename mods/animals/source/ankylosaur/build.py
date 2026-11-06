#!/usr/bin/env python3
"""Rebuilds Ankylosaur: paints the model and its animations (ankylosaur.py, into out/) and exports the game's
files into the mod (`../../assets`): models/ankylosaur.geo.json, animations/ankylosaur[_lie].animation.json and
textures/entity/ankylosaur/ankylosaur.png, and its coat layers into `../../genetics/layers/ankylosaur`. Blockbench
isn't needed for this.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)

Commit what it writes under `../../assets` and `../../genetics/layers/ankylosaur`.
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
# The behaviour clips (`mods/animals/src/anim.rs`) go in the standing file too, by their own names; at
# 1.9 t it stands its ground against a lone hunter (over `roles::STANDS_KG`): rear (a threat display
# with the tail's club), stare, then charge. No stalk or pounce: a plant eater.
CLIPS = ["stare", "graze", "drink", "sniff", "look", "scratch", "stretch", "sleep", "alarm", "rear", "play", "groom", "nuzzle", "flinch",
         "death", "watch", "approach", "charge"]
# Asleep it lies in its look (`wild.rs` settle): the sleep clip goes in the lying file too.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", **{c: c for c in CLIPS}},
         "_lie": {"idle": "lie", "walk": "walk", "sleep": "sleep"}}
FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
STANDING_CLIPS = ["stare", "graze", "drink", "sniff", "look", "scratch", "stretch", "alarm", "rear", "play", "groom", "nuzzle", "flinch",
                  "watch"]
AXIS = ["neck", "skull", "chest", "hips", "tail_"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "ankylosaur.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    from to_bedrock import export  # noqa: E402
    from ground import feet_low, gait, joints, pokes  # noqa: E402
    from linecheck import boxes_from_spec, lines  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another.
    import numpy as np  # noqa: E402
    from PIL import Image  # noqa: E402
    thin = [h for h in lines(boxes_from_spec(spec), np.array(Image.open(os.path.join(OUT, "texture.png")).convert("RGBA")))
            if h["kind"] in ("line", "thin")]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    poking = pokes(spec, AXIS)  # no neck corner through the head or chest
    assert not poking, f"poking through: {sorted(poking)}"
    for anim in ("walk", "run", "approach", "charge"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    A = spec["animations"]
    # The gaits written as angles stand on their feet (`ankylosaur.ground_clips`): none sinks, and each
    # foot comes down to the ground in its stride (a lean that holds a pair of feet off it fails).
    for clip in ("approach", "charge"):
        low = {f: min(feet_low(spec, A[clip], [f])) for f in FEET}
        assert all(abs(v) < 0.3 for v in low.values()), f"{clip}'s feet reach {low}, not the ground"
    # The standing clips keep a foot on the ground in every keyframe, and no foot sinks into it.
    for clip in STANDING_CLIPS:
        low = feet_low(spec, A[clip], FEET)
        assert min(low) > -0.3 and max(low) < 0.3, f"{clip}'s feet from {min(low):.2f} to {max(low):.2f}, not on the ground"
    # The threat display is braced on all four: every foot planted in every keyframe.
    for f in FEET:
        low = feet_low(spec, A["rear"], [f])
        assert min(low) > -0.3 and max(low) < 0.3, f"rearing, {f} from {min(low):.2f} to {max(low):.2f}"
    # Asleep it lies as in its lie, the belly on the ground.
    everything = [e["name"] for e in spec["elements"]]
    low = feet_low(spec, A["sleep"], everything)
    assert min(low) > -0.3 and max(low) < 0.3, f"asleep, its lowest point from {min(low):.2f} to {max(low):.2f}"
    # Dying, it tips onto its side: each frame rests on its lowest point.
    low = feet_low(spec, A["death"], everything)
    assert min(low) > -0.3 and max(low) < 0.3, f"dying, its lowest point from {min(low):.2f} to {max(low):.2f}"
    # Nothing goes into the ground (the club swung, the head down); the beak crops at it (graze, drink).
    for clip in CLIPS:
        low = min(feet_low(spec, A[clip], everything))
        assert low > -0.3, f"{clip}: {low:.2f} into the ground"
    for clip in ("graze", "drink"):
        low = min(feet_low(spec, A[clip], ["beak", "jaw", "snout"]))
        assert 0 <= low < 1.0, f"{clip}: the beak {low:.2f} over the ground"
    # A clip played once must not loop (the engine loops a clip whose file says so, whatever the game asks).
    looping = [c for c in CLIPS if A[c].get("loop") is True]
    assert sorted(looping) == ["approach", "charge", "drink", "graze", "sleep", "stare", "watch"], f"looping: {looping}"
    assert A["death"].get("loop") == "hold_on_last_frame", "death must hold its last frame"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "ankylosaur", SCALE, "ankylosaur/ankylosaur", pose_files=POSES)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "ankylosaur"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
