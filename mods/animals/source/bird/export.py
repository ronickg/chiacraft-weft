#!/usr/bin/env python3
"""Checks and exports a bird built on `bird.py` (its `<model>.py` sets `CFG`): flicker at rest and in
every keyframe, joints, pokes along the body's axis, a walk that goes forward; then the game's files
(Bedrock `.geo.json` for the ground and flying models, their animations, the texture) into the mod's
assets, and the coat layers into `genetics/layers/<model>`.

    ~/.venvs/blocky/bin/python ../bird/export.py <model>     (from the bird's source folder, or anywhere)
"""

import importlib.util
import json
import os
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.normpath(os.path.join(HERE, "..", "..", "..", ".."))
SKILL = os.path.join(REPO, ".claude", "skills", "blocky-model", "scripts")
ASSETS = os.path.join(HERE, "..", "..", "assets")
sys.path.insert(0, SKILL)
sys.path.insert(0, HERE)
from house import SCALE  # noqa: E402  the house density: 64 texels a block
from PIL import Image  # noqa: E402

import bird  # noqa: E402

# On the ground the engine plays `idle`, `walk` and `run` by itself; flying is a look of its own
# (`<model>_fly`, put on by `birds.rs`), its wings beating whatever it does.
GROUND = {"": {"idle": "idle", "walk": "walk", "run": "run"}}
FLY = {"": {"idle": "flap", "walk": "flap", "run": "flap"}}
FEET = ["toes_l", "toes_r"]


def export(model):
    src = os.path.join(HERE, "..", model)
    spec_ = importlib.util.spec_from_file_location(model, os.path.join(src, f"{model}.py"))
    mod = importlib.util.module_from_spec(spec_)
    spec_.loader.exec_module(mod)
    out = os.path.join(src, "out")
    os.makedirs(out, exist_ok=True)
    ground, fly, sheet = bird.build(mod.CFG, out)
    Image.fromarray(sheet, "RGBA").save(os.path.join(out, "texture.png"))
    for name, spec in (("spec.json", ground), ("spec_fly.json", fly)):
        with open(os.path.join(out, name), "w") as f:
            json.dump(spec, f)
    print(model, len(ground["elements"]), "elements on the ground,", len(fly["elements"]), "flying; texture", sheet.shape[1], "x",
          sheet.shape[0], "; fitted", ground["fitted"])
    for name in ("spec.json", "spec_fly.json"):
        subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(out, name)], check=True)
    from to_bedrock import export as to_bedrock  # noqa: E402
    from ground import gait, joints, pokes  # noqa: E402

    # A neck hidden inside the breast and the head (a songbird, an owl) shows no corners: left out.
    axis = ["skull", "chest", "hips", "tail"] if mod.CFG.get("neck_hidden") else ["neck", "skull", "chest", "hips", "tail"]
    from linecheck import boxes_from_spec, lines  # noqa: E402
    import numpy as np  # noqa: E402

    tex = np.array(Image.open(os.path.join(out, "texture.png")).convert("RGBA"))
    for spec in (ground, fly):
        # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another.
        thin = [h for h in lines(boxes_from_spec(spec), tex) if h["kind"] == "line"]
        assert not thin, f"{spec['name']}: thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
        loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
        assert not loose, f"{spec['name']}: loose joints: {loose}"
        poking = pokes(spec, axis)  # no neck corner through the head or the back
        assert not poking, f"{spec['name']}: poking through: {sorted(poking)}"
    for anim in ("walk", "run"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(ground, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    texture = os.path.join(out, "texture.png")
    made = to_bedrock(ground, texture, ASSETS, model, SCALE, f"{model}/{model}", pose_files=GROUND)
    made += to_bedrock(fly, texture, ASSETS, f"{model}_fly", SCALE, f"{model}/{model}", pose_files=FLY)
    shutil.copytree(os.path.join(out, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", model), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))


if __name__ == "__main__":
    export(sys.argv[1])
