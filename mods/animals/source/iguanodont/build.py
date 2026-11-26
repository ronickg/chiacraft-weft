#!/usr/bin/env python3
"""Rebuilds Iguanodont: paints the model and its animations (iguanodont.py, into out/) and exports the game's
files into the mod (`../../assets`): models/iguanodont.geo.json, animations/iguanodont[_lie].animation.json and
textures/entity/iguanodont/iguanodont.png, and its coat layers into `../../genetics/layers/iguanodont`. Blockbench
isn't needed for this.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)

Commit what it writes under `../../assets` and `../../genetics/layers/iguanodont`.
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
# big plant eater (4000 kg, over `roles::STANDS_KG`) stands its ground: it rears, stares, then charges.
CLIPS = ["stare", "graze", "drink", "sniff", "look", "scratch", "stretch", "sleep", "alarm", "rear", "play", "groom", "nuzzle", "flinch",
         "death", "watch", "approach", "charge"]
# Asleep it lies in its look (`wild.rs` settle): the sleep clip goes in the lying file too.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", **{c: c for c in CLIPS}},
         "_lie": {"idle": "lie", "walk": "walk", "sleep": "sleep"}}
FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
STANDING_CLIPS = ["stare", "graze", "drink", "sniff", "look", "scratch", "stretch", "alarm", "rear", "groom", "nuzzle", "flinch", "watch"]
AXIS = ["neck", "skull", "chest", "hips", "tail_"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "iguanodont.py")], check=True)
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
    # The gaits written as angles stand on their feet (`iguanodont.ground_clips`): none sinks, and each
    # foot comes down to the ground in its stride (a lean that holds a pair of feet off it fails).
    for clip in ("walk", "run", "approach", "charge"):
        low = {f: min(feet_low(spec, A[clip], [f])) for f in FEET}
        assert all(abs(v) < 0.3 for v in low.values()), f"{clip}'s feet reach {low}, not the ground"
    # The standing clips keep a foot on the ground in every keyframe, none sunk into it.
    for clip in STANDING_CLIPS:
        low = feet_low(spec, A[clip], FEET)
        assert min(low) > -0.3 and max(low) < 0.3, f"{clip}'s feet from {min(low):.2f} to {max(low):.2f}, not on the ground"
    # Rearing, the hind feet stay planted in every keyframe, the forefeet lifted, the tail behind on or
    # near the ground and never in it.
    low = feet_low(spec, A["rear"], FEET[2:])
    assert min(low) > -0.3 and max(low) < 0.3, f"rearing, the hind feet from {min(low):.2f} to {max(low):.2f}"
    assert min(feet_low(spec, A["rear"], FEET[:2])) > -0.3, "rearing, a forefoot in the ground"
    tail = [e["name"] for e in spec["elements"] if e["name"].startswith("tail_")]
    assert 0 <= min(feet_low(spec, A["rear"], tail)) < 3.0, "rearing, the tail in the ground or far over it"
    # The play's bounces land on the ground and never in it.
    assert abs(min(feet_low(spec, A["play"], FEET))) < 0.3, "the play lands in or over the ground"
    # Dying, it rolls onto its side: each frame rests on its lowest point.
    everything = [e["name"] for e in spec["elements"]]
    low = feet_low(spec, A["death"], everything)
    assert min(low) > -0.3 and max(low) < 0.3, f"dying, its lowest point from {min(low):.2f} to {max(low):.2f}"
    # Nothing goes into the ground in any clip (a big head or tail swinging down included), and nothing
    # but the feet standing; the beak crops at it (graze, drink).
    for clip in CLIPS:
        low = min(feet_low(spec, A[clip], everything))
        assert low > -0.3, f"{clip}: {low:.2f} into the ground"
    body = [n for n in everything if not n.startswith(("leg_", "thigh_", "meta_", "thumb_"))]
    for clip in CLIPS:
        if clip not in ("sleep", "death"):
            low = min(feet_low(spec, A[clip], body))
            assert low > -0.05, f"{clip}: the body {low:.2f} into the ground"
    for clip in ("graze", "drink"):
        low = min(feet_low(spec, A[clip], ["beak", "jaw", "predentary"]))
        assert 0 <= low < 1.0, f"{clip}: the beak {low:.2f} over the ground"
    # A clip played once must not loop (the engine loops a clip whose file says so, whatever the game asks).
    looping = [c for c in CLIPS if A[c].get("loop") is True]
    assert sorted(looping) == ["approach", "charge", "drink", "graze", "sleep", "stare", "watch"], f"looping: {looping}"
    # Lids shut asleep and dead (the last frame of the fall).
    for clip, fr in (("sleep", A["sleep"]["frames"]), ("death", A["death"]["frames"][-1:])):
        assert all(abs(f["rot"]["eyelid_L"][2]) > 89 and abs(f["rot"]["eyelid_R"][2]) > 89 for f in fr), f"{clip}: eyes open"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "iguanodont", SCALE, "iguanodont/iguanodont", pose_files=POSES)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "iguanodont"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
