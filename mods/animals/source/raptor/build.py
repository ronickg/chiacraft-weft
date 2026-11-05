#!/usr/bin/env python3
"""Rebuilds the raptor: paints the model and its animations (raptor.py, into out/) and exports the game's
files into the mod (`../../assets`): models/raptor.geo.json, animations/raptor[_rest].animation.json
and textures/entity/raptor/raptor.png, and osmolskae.png (the second species' coat). Blockbench isn't needed for this.

    ../dog/chihuahua/.venv/bin/python build.py     (or any Python with requirements.txt installed)

Commit what it writes under `../../assets`.
"""

import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", ".."))
SKILL = os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts")
OUT = os.path.join(HERE, "out")
ASSETS = os.path.join(HERE, "..", "..", "assets")

SCALE = 0.3  # model pixels per unit: about half a block at the hip, 1.7 long (a real Velociraptor: 0.5 m, 1.8-2 m)
# Which animations go in which file: the engine plays `idle` and `walk` by itself, a pose's
# file holds it in `idle`; `run` and `meow` are there for code to play.
# The behaviour clips (`mods/animals/src/anim.rs`) go in the standing file too, by their own names.
CLIPS = ["screech", "bite", "kick", "stare", "sniff", "drink", "look", "scratch", "groom", "stretch", "sleep", "stalk", "alarm", "pounce",
         "play", "nuzzle", "flinch", "death", "approach"]
# Asleep it lies in its look (`rest.rs` settle): the sleep clip goes in the resting file too.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", **{c: c for c in CLIPS}},
         "_rest": {"idle": "rest", "walk": "walk", "sleep": "sleep"}}
FEET = ["toes_L", "toes_R"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "raptor.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    sys.path.insert(0, SKILL)
    from to_bedrock import export  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    from ground import feet_low, gait, grounded, joints, pokes  # noqa: E402
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another.
    # Meant: the mouth's dark wedge between the jaw and the upturned snout (the exported model lists it).
    from linecheck import boxes_from_spec, lines  # noqa: E402
    from PIL import Image  # noqa: E402
    import numpy as np  # noqa: E402
    meant = ("mouth.west", "mouth.east", "mouth.south")
    boxes = [dict(b, unit=4 * SCALE) for b in boxes_from_spec(spec)]  # at the size the game draws it (4 * SCALE of linecheck's units)
    thin = [h for h in lines(boxes, np.array(Image.open(os.path.join(OUT, "texture.png")).convert("RGBA")), allow=meant)
            if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    # Along the body's axis nothing pokes out of its neighbor in any pose (a neck's corner through
    # the head, a tail segment's through the one before).
    poking = pokes(spec, ["neck", "skull", "chest", "hips", "tail_"])
    assert not poking, "poking through: " + "; ".join(f"{a} out of {b} by {d:.2f} ({w[0]})" for (a, b), (d, w) in poking.items())
    for anim in ("walk", "run", "stalk", "approach"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    # The gaits written as angles stand on their feet: the root's height from the feet, frame by
    # frame, and checked: no foot sinks into the ground or floats over it.
    for clip in ("stalk", "approach"):
        spec["animations"][clip] = grounded(spec, spec["animations"][clip], FEET)
        low = min(feet_low(spec, spec["animations"][clip], FEET))
        assert abs(low) < 0.3, f"{clip}'s feet at {low:.2f}, not on the ground"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "raptor", SCALE, "raptor/raptor", pose_files=POSES)
    import shutil  # noqa: E402

    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "raptor"), dirs_exist_ok=True)
    # Velociraptor osmolskae wears the same model in its own colors.
    shutil.copy(os.path.join(OUT, "osmolskae.png"), os.path.join(ASSETS, "textures", "entity", "raptor", "osmolskae.png"))
    print("wrote", made, "and textures/entity/raptor/osmolskae.png into", os.path.normpath(ASSETS))
