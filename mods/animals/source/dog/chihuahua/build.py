#!/usr/bin/env python3
"""Rebuilds the chihuahua: paints the model and its animations (tools/blocky.py, into
out/d_blocky/) and exports the game's files into the mod (`../../assets`): models/chihuahua.geo.json
and chihuahua_pup.geo.json, animations/chihuahua[_pup][_sit|_lie].animation.json and
textures/entity/chihuahua/chihuahua.png. Blockbench isn't needed for this.

    python3 -m venv .venv && .venv/bin/pip install -r requirements.txt   # once
    .venv/bin/python build.py

Commit what it writes under `../../assets`.
"""

import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", "..", ".."))
EXPORT = os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts")
OUT = os.path.join(HERE, "out", "d_blocky")
ASSETS = os.path.join(HERE, "..", "..", "..", "assets")

SCALE = 0.122  # model pixels per unit: a real chihuahua, about 20 cm at the shoulder (15-23 cm)
PUP, PUP_HEAD = 0.5, 1.3  # a puppy: half the size, a bigger head

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "tools", "blocky.py")], check=True)
    # No flicker at rest or in any keyframe (the skill's check).
    subprocess.run([sys.executable, os.path.join(EXPORT, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    sys.path.insert(0, EXPORT)
    from to_bedrock import export  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    from ground import feet_low, gait, grounded, joints  # noqa: E402
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    # No thin lines (Ronald, 11-05): every box on or in another flush with it or two texels clear.
    # Allowed: the nose, a one-texel pad on a muzzle front of 2 x 1 texels, so the white round it
    # can be neither painted nor covered without losing the nose.
    import numpy as np  # noqa: E402
    from PIL import Image  # noqa: E402
    from linecheck import boxes_from_spec, lines  # noqa: E402
    thin = [h for h in lines(boxes_from_spec(spec), np.array(Image.open(os.path.join(OUT, "texture.png")).convert("RGBA")),
                             allow=("muzzle.south",)) if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
    for anim in ("walk", "run", "stalk", "approach"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    # The gaits written as angles stand on their feet: the root's height from the feet, frame by
    # frame, and checked: no foot sinks into the ground or floats over it.
    for clip in ("stalk", "approach"):
        spec["animations"][clip] = grounded(spec, spec["animations"][clip], FEET)
        low = min(feet_low(spec, spec["animations"][clip], FEET))
        assert abs(low) < 0.3, f"{clip}'s feet at {low:.2f}, not on the ground"
    # Lying, the back legs and paws rest on the ground (tools/ground.py fits lie_hip and lie_knee).
    for part in (["thigh_BL", "thigh_BR"], ["leg_BL_paw", "leg_BR_paw"]):
        low = feet_low(spec, spec["animations"]["lie"], part)
        assert all(abs(y) < 0.3 for y in low), f"lying, {part} at {min(low):.2f}..{max(low):.2f}, not on the ground"
    from to_bedrock import POSE_FILES  # noqa: E402
    # Its bite and the behaviour clips (`mods/animals/src/anim.rs`), for code to play.
    sys.path.insert(0, os.path.join(HERE, "tools"))
    from blocky import CLIP_LENGTHS  # noqa: E402
    # Asleep it lies in its look (`rest.rs` settle): the sleep clip goes in the lying file too.
    poses = {**POSE_FILES, "": {**POSE_FILES[""], "bite": "bite", **{c: c for c in CLIP_LENGTHS}},
             "_lie": {**POSE_FILES["_lie"], "sleep": "sleep"}}
    # `--anims-only`: the animations alone, the models, texture and coat layers left as they are
    # (2026-11-05: Upstream was looking into the chihuahua's texture; its skin wasn't to change).
    only = "--anims-only" in sys.argv
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "chihuahua", SCALE, "chihuahua/chihuahua", PUP, PUP_HEAD,
                  pose_files=poses, anims_only=only)
    if only:
        print("wrote the animations of", made, "into", os.path.normpath(ASSETS))
        sys.exit(0)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    import shutil  # noqa: E402
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "..", "genetics", "layers", "chihuahua"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
