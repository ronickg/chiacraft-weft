#!/usr/bin/env python3
"""Rebuilds the caiman: paints the model and its animations (caiman.py, into out/) and exports the
game's files into the mod (`../../assets`): models/caiman.geo.json,
animations/caiman[_bask].animation.json and textures/entity/caiman/caiman.png, and its coat layers
into `../../genetics/layers/caiman`. Blockbench isn't needed for this.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)

Commit what it writes under `../../assets` and `../../genetics/layers/caiman`.
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

# Which animations go in which file: the engine plays `idle`, `walk` and `run` by itself, a hit plays
# the bite; basking is a look of its own.
# The behaviour clips (`mods/animals/src/anim.rs`) go in the standing file too, by their own names
# (no graze: it eats meat; no rear, charge or howl).
CLIPS = ["stare", "watch", "drink", "sleep", "stalk", "approach", "pounce", "alarm", "sniff", "look", "scratch", "stretch", "play",
         "groom", "nuzzle", "flinch", "death"]
# Asleep it lies as it basks (`wild.rs` settle): the sleep clip goes in the basking file too.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", "bite": "bite", **{c: c for c in CLIPS}},
         "_bask": {"idle": "bask", "walk": "walk", "sleep": "sleep"}}
FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
AXIS = ["neck", "skull", "chest", "hips", "tail"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "caiman.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    from to_bedrock import export  # noqa: E402
    from ground import feet_low, gait, grounded, joints, pokes  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    poking = pokes(spec, AXIS)  # no neck corner through the head or chest, no tail through the rump
    assert not poking, f"poking through: {sorted(poking)}"
    for anim in ("walk", "run", "stalk", "approach"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    # The gaits written as angles stand on their feet: the root's height from the feet, frame by frame;
    # so do the hiss and the lunge, up on straightened legs.
    for clip in ("stalk", "approach", "alarm", "pounce"):
        spec["animations"][clip] = grounded(spec, spec["animations"][clip], FEET)
    # Every standing clip keeps a foot on the ground in every keyframe, none sinks; nothing of it goes
    # into the ground (the jaw dipped to drink, the chin asleep).
    everything = [e["name"] for e in spec["elements"]]
    for clip in [c for c in CLIPS if c != "death"]:
        low = feet_low(spec, spec["animations"][clip], FEET)
        assert min(low) > -0.3 and max(low) < 0.3, f"{clip}'s feet from {min(low):.2f} to {max(low):.2f}, not on the ground"
        assert min(feet_low(spec, spec["animations"][clip], everything)) > -0.3, f"{clip} goes into the ground"
    # The stalk creeps with its belly just off the ground, never in it.
    belly = feet_low(spec, spec["animations"]["stalk"], ["chest", "hips"])
    assert 0 < min(belly) and max(belly) < 1.5, f"stalking, the belly from {min(belly):.2f} to {max(belly):.2f} over the ground"
    # Dying, it rolls over onto its back: each frame rests on its lowest point.
    spec["animations"]["death"] = grounded(spec, spec["animations"]["death"], everything)
    low = feet_low(spec, spec["animations"]["death"], everything)
    assert min(low) > -0.3 and max(low) < 0.3, f"dying, its lowest point from {min(low):.2f} to {max(low):.2f}"
    # A clip played once must not loop (the engine loops a clip whose file says so, whatever the game
    # asks): only the held ones and the gaits; the bite plays once too.
    looping = [c for c in CLIPS + ["bite"] if spec["animations"][c].get("loop") is True]
    assert sorted(looping) == ["approach", "drink", "sleep", "stalk", "stare", "watch"], f"looping: {looping}"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "caiman", SCALE, "caiman/caiman", pose_files=POSES)
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another, checked
    # on the model as the game has it (`linecheck.py` on the exported geo).
    from linecheck import density_of, lines, load  # noqa: E402
    boxes, tex = load(os.path.join(ASSETS, "models", "caiman.geo.json"), os.path.join(ASSETS, "textures", "entity", "caiman", "caiman.png"))
    thin = [h for h in lines(boxes, tex, density_of(boxes), allow=()) if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "caiman"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
