#!/usr/bin/env python3
"""Rebuilds the hare: paints the model and its animations (hare.py, into out/) and exports the game's
files into the mod (`../../assets`): models/hare.geo.json, animations/hare[_lie].animation.json and
textures/entity/hare/hare.png, and its coat layers into `../../genetics/layers/hare`. Blockbench
isn't needed for this. The brown hare and the mountain hare share it.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)

Commit what it writes under `../../assets` and `../../genetics/layers/hare`.
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
from house import SCALE  # noqa: E402  the house density: 64 texels a block, a real-sized hare

# Which animations go in which file: the engine plays `idle`, `walk` and `run` by itself; the form
# (lying low) is a look of its own.
# The behaviour clips (`mods/animals/src/anim.rs`) go in the standing file too, by their own names
# (no rear or charge: too small to stand its ground).
CLIPS = ["stare", "graze", "drink", "sniff", "look", "scratch", "stretch", "sleep", "alarm", "play", "groom", "nuzzle", "flinch",
         "death", "watch", "approach"]
# Asleep it lies in its form (`wild.rs` settle): the sleep clip goes in the lying file too.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", **{c: c for c in CLIPS}},
         "_lie": {"idle": "lie", "walk": "walk", "sleep": "sleep"}}
FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
AXIS = ["neck", "skull", "chest", "hips", "tail"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "hare.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    from to_bedrock import export  # noqa: E402
    from ground import feet_low, gait, grounded, joints, pokes  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    poking = pokes(spec, AXIS)  # no neck corner through the head or chest, no tail through the rump
    assert not poking, f"poking through: {sorted(poking)}"
    for anim in ("walk", "run", "approach"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    # The approach, written as angles, stands on its feet (`hare.py` sets its height from them frame by
    # frame): no foot sinks into the ground or floats over it.
    spec["animations"]["approach"] = grounded(spec, spec["animations"]["approach"], FEET)
    low = feet_low(spec, spec["animations"]["approach"], FEET)
    assert max(abs(v) for v in low) < 0.3, f"approach's feet at {min(low):.2f}..{max(low):.2f}, not on the ground"
    # The standing clips keep a foot on the ground in every frame (sat up on the hind ones, a hind foot
    # raised to the ear, both lifted to thump while the forepaws stay down; the binky leaves it and
    # lands); none sinks.
    for clip in ("stare", "graze", "drink", "sniff", "look", "scratch", "stretch", "alarm", "groom", "nuzzle", "flinch", "watch"):
        low = feet_low(spec, spec["animations"][clip], FEET)
        assert min(low) > -0.3 and max(low) < 0.3, f"{clip}'s feet from {min(low):.2f} to {max(low):.2f}, not on the ground"
    low = feet_low(spec, spec["animations"]["play"], FEET)
    assert abs(min(low)) < 0.3 and low[0] < 0.3 and low[-1] < 0.3, "the binky doesn't take off from and land on the ground"
    # Dying, it rolls onto its side: each frame rests on its lowest point (mid-roll its legs swing out
    # below where it ends up lying).
    everything = [e["name"] for e in spec["elements"]]
    low = feet_low(spec, spec["animations"]["death"], everything)
    assert min(low) > -0.3 and max(low) < 0.3, f"dying, its lowest point from {min(low):.2f} to {max(low):.2f}"
    # A clip played once must not loop (the engine loops a clip whose file says so, whatever the game asks).
    looping = [c for c in CLIPS if spec["animations"][c].get("loop") is True]
    assert sorted(looping) == ["approach", "drink", "graze", "sleep", "stare", "watch"], f"looping: {looping}"
    assert spec["animations"]["death"]["loop"] == "hold_on_last_frame", "the death must hold its last frame"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "hare", SCALE, "hare/hare", pose_files=POSES)
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another, checked
    # on the model as the game has it (`linecheck.py` on the exported geo).
    from linecheck import density_of, lines, load  # noqa: E402
    boxes, tex = load(os.path.join(ASSETS, "models", "hare.geo.json"), os.path.join(ASSETS, "textures", "entity", "hare", "hare.png"))
    thin = [h for h in lines(boxes, tex, density_of(boxes), allow=()) if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "hare"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
