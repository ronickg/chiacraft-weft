#!/usr/bin/env python3
"""Rebuilds the lizard: paints the model and its animations (lizard.py, into out/) and exports the game's
files into the mod (`../../assets`): models/lizard.geo.json, animations/lizard[_bask].animation.json and
textures/entity/lizard/lizard.png, and its coat layers into `../../genetics/layers/lizard`. Blockbench
isn't needed for this.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)

Commit what it writes under `../../assets` and `../../genetics/layers/lizard`.
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

# Which animations go in which file: the engine plays `idle`, `walk` and `run` by itself; basking
# is a look of its own.
# The behaviour clips (`mods/animals/src/anim.rs`) go in the standing file too, by their own names (no
# stalk or pounce: not a hunter; no rear or charge: too small; no nuzzle: an agama leaves its eggs).
CLIPS = ["stare", "watch", "graze", "drink", "sleep", "approach", "alarm", "sniff", "look", "scratch", "stretch", "play",
         "groom", "flinch", "death"]
# Asleep it lies as it basks: the sleep clip goes in the basking file too.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", **{c: c for c in CLIPS}},
         "_bask": {"idle": "bask", "walk": "walk", "sleep": "sleep"}}
FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
AXIS = ["neck", "skull", "chest", "hips", "tail"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "lizard.py")], check=True)
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
    # The gait written as angles stands on its feet: the root's height from the feet, frame by frame; so
    # do the clips that raise it on straightened legs (the high stance, the push-ups).
    for clip in ("approach", "stare", "watch", "alarm", "stretch"):
        spec["animations"][clip] = grounded(spec, spec["animations"][clip], FEET)
    # Every standing clip keeps each foot on the ground in every keyframe (the scratching one aside), none
    # sinks; nothing of it goes into the ground (the snout down to drink, to snap, to wipe).
    everything = [e["name"] for e in spec["elements"]]
    for clip in [c for c in CLIPS if c != "death"]:
        for foot in FEET:
            if clip in ("approach",) or (clip == "scratch" and foot == "leg_BL_paw"):
                continue
            low = feet_low(spec, spec["animations"][clip], [foot])
            assert min(low) > -0.3 and max(low) < 0.3, f"{clip}'s {foot} from {min(low):.2f} to {max(low):.2f}, not on the ground"
        low = feet_low(spec, spec["animations"][clip], FEET)
        assert min(low) > -0.3 and max(low) < 0.3, f"{clip}'s feet from {min(low):.2f} to {max(low):.2f}, not on the ground"
        assert min(feet_low(spec, spec["animations"][clip], everything)) > -0.3, f"{clip} goes into the ground"
    # The scratching foot clear of the ground, the approach's belly too.
    assert min(feet_low(spec, spec["animations"]["scratch"], ["leg_BL_paw"])) > -0.3, "the scratching foot goes into the ground"
    belly = feet_low(spec, spec["animations"]["approach"], ["chest", "hips"])
    assert min(belly) > 0, f"approaching, the belly {min(belly):.2f} over the ground"
    # Dying, it rolls over onto its back: each frame rests on its lowest point.
    spec["animations"]["death"] = grounded(spec, spec["animations"]["death"], everything)
    low = feet_low(spec, spec["animations"]["death"], everything)
    assert min(low) > -0.3 and max(low) < 0.3, f"dying, its lowest point from {min(low):.2f} to {max(low):.2f}"
    # A clip played once must not loop (the engine loops a clip whose file says so, whatever the game
    # asks): only the held ones and the gait.
    looping = [c for c in CLIPS if spec["animations"][c].get("loop") is True]
    assert sorted(looping) == ["approach", "drink", "graze", "sleep", "stare", "watch"], f"looping: {looping}"
    # Asleep and dead, its eyes are shut.
    for clip in ("sleep", "death"):
        lids = [abs(f["rot"]["eyelid_L"][2]) for f in spec["animations"][clip]["frames"]]
        assert lids[-1] >= 89 and (clip == "death" or min(lids) >= 89), f"{clip}'s eyes open"
    with open(os.path.join(OUT, "built.json"), "w") as f:  # the clips as exported, for review renders
        json.dump(spec, f)
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "lizard", SCALE, "lizard/lizard", pose_files=POSES)
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another, checked
    # on the model as the game has it (`linecheck.py` on the exported geo).
    from linecheck import density_of, lines, load  # noqa: E402
    boxes, tex = load(os.path.join(ASSETS, "models", "lizard.geo.json"), os.path.join(ASSETS, "textures", "entity", "lizard", "lizard.png"))
    thin = [h for h in lines(boxes, tex, density_of(boxes), allow=()) if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "lizard"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
