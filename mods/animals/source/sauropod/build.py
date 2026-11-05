#!/usr/bin/env python3
"""Rebuilds Sauropod: paints the model and its animations (sauropod.py, into out/) and exports the game's
files into the mod (`../../assets`): models/sauropod.geo.json, animations/sauropod[_lie].animation.json and
textures/entity/sauropod/sauropod.png, and its coat layers into `../../genetics/layers/sauropod`. Blockbench
isn't needed for this.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)

Commit what it writes under `../../assets` and `../../genetics/layers/sauropod`.
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
from house import SCALE  # noqa: E402  0.25 model pixels a unit (64 units a block)

# Which animations go in which file: the engine plays `idle`, `walk` and `run` by itself; lying
# down is a look of its own.
# The behaviour clips (`mods/animals/src/anim.rs`) go in the standing file too, by their own names; a
# 10 t plant eater stands its ground against a lone hunter (over `roles::STANDS_KG`): rear (up on the
# hind legs and the tail), stare, then charge. No stalk or pounce: it hunts nothing.
CLIPS = ["stare", "graze", "drink", "sniff", "look", "scratch", "stretch", "sleep", "alarm", "rear", "play", "groom", "nuzzle", "flinch",
         "death", "watch", "approach", "charge"]
# Asleep it lies in its look (`wild.rs` settle): the sleep clip goes in the lying file too.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", **{c: c for c in CLIPS}},
         "_lie": {"idle": "lie", "walk": "walk", "sleep": "sleep"}}
FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
STANDING_CLIPS = ["stare", "graze", "drink", "sniff", "look", "scratch", "stretch", "alarm", "play", "groom", "nuzzle", "flinch", "watch"]
AXIS = ["neck", "skull", "chest", "hips", "tail_"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "sauropod.py")], check=True)
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
    # The gaits written as angles stand on their feet (`sauropod.ground_clips`): none sinks, and each
    # foot comes down to the ground in its stride.
    for clip in ("approach", "charge"):
        low = {f: min(feet_low(spec, A[clip], [f])) for f in FEET}
        assert all(abs(v) < 0.3 for v in low.values()), f"{clip}'s feet reach {low}, not the ground"
    # The standing clips keep a foot on the ground in every keyframe, none sunk into it.
    for clip in STANDING_CLIPS:
        low = feet_low(spec, A[clip], FEET)
        assert min(low) > -0.3 and max(low) < 0.3, f"{clip}'s feet from {min(low):.2f} to {max(low):.2f}, not on the ground"
    # Rearing, the hind feet stay planted in every keyframe, the forefeet over the ground, and the tail
    # (the tripod's third leg) down on it at the top.
    low = feet_low(spec, A["rear"], FEET[2:])
    assert min(low) > -0.3 and max(low) < 0.3, f"rearing, the hind feet from {min(low):.2f} to {max(low):.2f}"
    assert min(feet_low(spec, A["rear"], FEET[:2])) > -0.3, "rearing, a forefoot in the ground"
    tail = feet_low(spec, A["rear"], [f"tail_{k}" for k in range(5)])
    assert min(tail) > -0.3 and min(tail) < 1.0, f"rearing, the tail's lowest {min(tail):.2f}: not on the ground"
    # Dying, it rolls onto its side: each frame rests on its lowest point.
    everything = [e["name"] for e in spec["elements"]]
    low = feet_low(spec, A["death"], everything)
    assert min(low) > -0.3 and max(low) < 0.3, f"dying, its lowest point from {min(low):.2f} to {max(low):.2f}"
    # Nothing goes below the ground in any clip (nor the idle and the lie): not the long neck's head
    # swung down, not the tail.
    for clip in CLIPS + ["idle", "lie"]:
        low = min(feet_low(spec, A[clip], everything))
        assert low > -0.3, f"{clip}: {low:.2f} into the ground"
    # Cropping and drinking, the mouth at the ground.
    for clip in ("graze", "drink"):
        low = min(feet_low(spec, A[clip], ["snout", "jaw", "mouth"]))
        assert 0 <= low < 1.0, f"{clip}: the mouth {low:.2f} over the ground"
    # Asleep, the head laid on the ground.
    low = min(feet_low(spec, A["sleep"], ["snout", "jaw", "mouth", "skull", "dome"]))
    assert -0.3 < low < 1.0, f"asleep, the head {low:.2f} over the ground"
    # A clip played once must not loop (the engine loops a clip whose file says so, whatever the game asks).
    looping = [c for c in CLIPS if A[c].get("loop") is True]
    assert sorted(looping) == ["approach", "charge", "drink", "graze", "sleep", "stare", "watch"], f"looping: {looping}"
    assert A["death"].get("loop") == "hold_on_last_frame", "death must hold its last frame"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "sauropod", SCALE, "sauropod/sauropod", pose_files=POSES)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "sauropod"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
