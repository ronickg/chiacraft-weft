#!/usr/bin/env python3
"""Rebuilds the cat: paints the model and its animations (cat.py, into out/) and exports the game's
files into the mod (`../../assets`): models/cat.geo.json, animations/cat[_sit|_lie].animation.json
and textures/entity/cat/cat.png. Blockbench isn't needed for this.

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

SCALE = 0.174  # model pixels per unit: a real house cat, about 24 cm at the shoulder (23-25 cm)
# Which animations go in which file: the engine plays `idle` and `walk` by itself, a pose's
# file holds it in `idle`; `run` and `meow` are there for code to play.
# The behaviour clips (`mods/animals/src/anim.rs`, `cat.py`) go in the standing file too, by their own
# names; asleep it lies in its look, so the sleep clip goes in the lying file too.
HELD = ["stare", "watch", "drink", "sleep"]
GAITS = ["stalk", "approach"]
ONCE = ["alarm", "pounce", "sniff", "look", "scratch", "stretch", "play", "groom", "nuzzle", "flinch"]
CLIPS = HELD + GAITS + ONCE + ["death"]
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", "meow": "meow", "swipe": "swipe", "bite": "bite", **{c: c for c in CLIPS}},
         "_sit": {"idle": "sit", "walk": "walk"},
         "_lie": {"idle": "lie", "walk": "walk", "sleep": "sleep"}}
FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
AXIS = ["neck", "skull", "chest", "hips", "tail_"]
LEAP = (0.5, 0.8)  # the pounce is a leap: off the ground only from 0.5 to 0.8 s

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "cat.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    sys.path.insert(0, SKILL)
    from to_bedrock import export  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    from ground import feet_low, gait, joints, pokes  # noqa: E402
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another
    # (linecheck; flush, or two texels clear).
    # Meant: the cat is small at 32 a block, so an upper leg's face is 2 by 2 texels and any of it the
    # chest or hips cover leaves the rest under two (the legs' tops showing 1-1.25 texels under the
    # body at their joints); the white throat a texel under the skull.
    from linecheck import boxes_from_spec, lines  # noqa: E402
    from PIL import Image  # noqa: E402
    import numpy as np  # noqa: E402
    thin = [h for h in lines(boxes_from_spec(spec), np.array(Image.open(os.path.join(OUT, "texture.png")).convert("RGBA")),
                             allow=("leg_FL_upper", "leg_FR_upper", "thigh_*", "neck.south")) if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    for anim in ("walk", "run", "stalk", "approach"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    # The behaviour clips poke nothing along the body's axis that the cat's own clips don't (the neck
    # out of the chest or skull at a slant, a tail segment out of the one before): per pair, no further
    # than in idle, walk, run, sit, lie and the strikes (its rest tail already pokes a little).
    own = pokes(dict(spec, animations={k: v for k, v in spec["animations"].items() if k not in CLIPS}), AXIS)
    for clip in CLIPS:
        new = pokes(dict(spec, animations={clip: spec["animations"][clip]}), AXIS)
        worse = {k: round(d, 2) for k, (d, _) in new.items() if d > own.get(k, (0.0,))[0] + 1e-6}
        assert not worse, f"{clip} pokes through: {worse}"
    # Only the held clips and the gaits loop (the engine loops a clip whose file says so, whatever the
    # game asks: a hit cat would flinch forever); the strikes and the one-offs play once, the death
    # holds its last frame.
    looping = sorted(c for c, a in spec["animations"].items() if a.get("loop") is True)
    assert looping == sorted(HELD + GAITS), f"looping: {looping}"
    assert spec["animations"]["death"]["loop"] == "hold_on_last_frame"
    # The gaits and the scratch's sitting down stand on their feet (set by `cat.py`, `ground.grounded`).
    for clip in GAITS + ["scratch"]:
        low = feet_low(spec, spec["animations"][clip], FEET)
        assert max(abs(v) for v in low) < 0.3, f"{clip}'s feet at {min(low):.2f}..{max(low):.2f}, not on the ground"
    # Every standing clip keeps a paw on the ground in every keyframe, none sinking, and nothing goes
    # into the ground; the pounce leaves it only in its leap and lands.
    everything = [e["name"] for e in spec["elements"]]
    for clip in HELD + GAITS + ONCE:
        a = spec["animations"][clip]
        if clip == "sleep":  # lying: the body on the ground, the paws tucked
            low = feet_low(spec, a, everything)
            assert min(low) > -0.3 and max(low) < 0.3, f"asleep, its lowest point from {min(low):.2f} to {max(low):.2f}"
            continue
        low = feet_low(spec, a, FEET)
        down = [v for f, v in zip(a["frames"], low) if not (clip == "pounce" and LEAP[0] < f["t"] < LEAP[1])]
        assert min(low) > -0.3 and max(down) < 0.3, f"{clip}'s paws from {min(low):.2f} to {max(down):.2f}, not on the ground"
        assert min(feet_low(spec, a, everything)) > -0.3, f"{clip} goes into the ground"
    assert max(feet_low(spec, spec["animations"]["pounce"], FEET)) > 3, "the pounce doesn't leave the ground"
    # Dying, it rolls onto its side: each frame rests on its lowest point (set by `cat.py`).
    low = feet_low(spec, spec["animations"]["death"], everything)
    assert min(low) > -0.3 and max(low) < 0.3, f"dying, its lowest point from {min(low):.2f} to {max(low):.2f}"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "cat", SCALE, "cat/cat", pose_files=POSES)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    import shutil  # noqa: E402
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "cat"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
