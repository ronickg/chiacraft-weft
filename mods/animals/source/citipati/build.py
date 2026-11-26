#!/usr/bin/env python3
"""Rebuilds Citipati: paints the model and its animations (citipati.py, into out/) and exports the game's
files into the mod (`../../assets`): models/citipati.geo.json, animations/citipati[_lie].animation.json and
textures/entity/citipati/citipati.png, and its coat layers into `../../genetics/layers/citipati`. Blockbench
isn't needed for this.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)

Commit what it writes under `../../assets` and `../../genetics/layers/citipati`.
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
# The behaviour clips (`mods/animals/src/anim.rs`) go in the standing file too, by their own names; a
# mother stands her ground for her young (90 kg, over `roles::GUARDS_KG`): rear, then stare, then charge.
# No stalk or pounce: not a hunter.
CLIPS = ["stare", "graze", "drink", "sniff", "look", "scratch", "stretch", "sleep", "alarm", "rear", "play", "groom", "nuzzle", "flinch",
         "death", "watch", "approach", "charge"]
# Asleep it lies in its look, brooding (`wild.rs` settle): the sleep clip goes in the lying file too.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", **{c: c for c in CLIPS}},
         "_lie": {"idle": "lie", "walk": "walk", "sleep": "sleep"}}
FEET = ["toes_L", "toes_R"]
STANDING_CLIPS = ["stare", "graze", "drink", "sniff", "look", "scratch", "stretch", "alarm", "rear", "groom", "nuzzle", "flinch", "watch"]
AXIS = ["neck", "skull", "chest", "hips", "tail_"]  # "neck" takes in both neck segments

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "citipati.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    from to_bedrock import export  # noqa: E402
    from ground import feet_low, gait, joints, pokes  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another.
    from linecheck import boxes_from_spec, lines  # noqa: E402
    from PIL import Image  # noqa: E402
    import numpy as np  # noqa: E402
    thin = [h for h in lines(boxes_from_spec(spec), np.array(Image.open(os.path.join(OUT, "texture.png")).convert("RGBA")))
            if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    poking = pokes(spec, AXIS)  # no neck corner through the head or chest
    assert not poking, f"poking through: {sorted(poking)}"
    for anim in ("walk", "run", "approach", "charge"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    A = spec["animations"]
    # The gaits written as angles stand on their feet (`citipati.ground_clips`): the approach a foot on
    # the ground in every frame, the charge's feet coming down to it in its stride; none sinks.
    low = feet_low(spec, A["approach"], FEET)
    assert max(abs(v) for v in low) < 0.3, f"approach's feet at {min(low):.2f}..{max(low):.2f}, not on the ground"
    for f in FEET:
        low = min(feet_low(spec, A["charge"], [f]))
        assert abs(low) < 0.3, f"charge: {f} comes down to {low:.2f}, not the ground"
    # The standing clips keep a foot on the ground in every keyframe, none sunk into it (rearing, both
    # planted; the stretch's leg pushed out behind, the other standing).
    for clip in STANDING_CLIPS:
        low = feet_low(spec, A[clip], FEET)
        assert min(low) > -0.3 and max(low) < 0.3, f"{clip}'s feet from {min(low):.2f} to {max(low):.2f}, not on the ground"
    for f in FEET:
        low = feet_low(spec, A["rear"], [f])
        assert min(low) > -0.3 and max(low) < 0.3, f"rearing, {f} from {min(low):.2f} to {max(low):.2f}: not planted"
    # The play's hops take off from the ground and land on it.
    low = feet_low(spec, A["play"], FEET)
    assert abs(min(low)) < 0.3 and abs(low[0]) < 0.3 and abs(low[-1]) < 0.3, "the play doesn't take off from and land on the ground"
    # Dying, it falls onto its side: each frame rests on its lowest point.
    everything = [e["name"] for e in spec["elements"]]
    low = feet_low(spec, A["death"], everything)
    assert min(low) > -0.3 and max(low) < 0.3, f"dying, its lowest point from {min(low):.2f} to {max(low):.2f}"
    # Nothing goes into the ground in any clip (a head, wing or tail swung down included).
    for clip in CLIPS:
        low = min(feet_low(spec, A[clip], everything))
        assert low > -0.3, f"{clip}: a part {low:.2f} into the ground"
    # Pecking (the idle's jabs, graze, drink) brings the beak to the ground: its tip, not the crest; the
    # idle stands on its feet and nothing of it goes into the ground.
    low = feet_low(spec, A["idle"], FEET)
    assert min(low) > -0.3 and max(low) < 0.3, f"idle's feet from {min(low):.2f} to {max(low):.2f}, not on the ground"
    assert min(feet_low(spec, A["idle"], everything)) > -0.3, "idle: a part into the ground"
    for clip in ("idle", "graze", "drink"):
        low = min(feet_low(spec, A[clip], ["beak", "jaw"]))
        head = min(feet_low(spec, A[clip], ["skull", "snout", "crest", "crest_1"]))
        assert 0 <= low < 1.0 and head > low, f"{clip}: the beak {low:.2f} over the ground, the head {head:.2f}"
    # Lids shut asleep and dead.
    for clip in ("sleep", "death"):
        f = A[clip]["frames"][-1]["rot"]
        assert abs(f["eyelid_L"][2]) >= 89 and abs(f["eyelid_R"][2]) >= 89, f"{clip}: eyes open"
    # A clip played once must not loop (the engine loops a clip whose file says so, whatever the game asks).
    looping = [c for c in CLIPS if A[c].get("loop") is True]
    assert sorted(looping) == ["approach", "charge", "drink", "graze", "sleep", "stare", "watch"], f"looping: {looping}"
    assert A["death"]["loop"] == "hold_on_last_frame", "the death must hold its last frame"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "citipati", SCALE, "citipati/citipati", pose_files=POSES)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "citipati"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
