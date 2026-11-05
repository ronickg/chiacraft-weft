#!/usr/bin/env python3
"""Rebuilds the capybara: paints the model and its animations (capybara.py, into out/) and exports the game's
files into the mod (`../../assets`): models/capybara.geo.json, animations/capybara[_sit|_lie].animation.json
and textures/entity/capybara/capybara.png. Blockbench isn't needed for this.

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

SCALE = 0.34  # model pixels per unit: about half a block at the shoulder, 1.1 long (a real capybara: 0.5-0.6 m, 1-1.3 m)
# Which animations go in which file: the engine plays `idle` and `walk` by itself, a pose's
# file holds it in `idle`; `run` and `meow` are there for code to play.
# The behaviour clips (`mods/animals/src/anim.rs`) go in the standing file too, by their own names
# (rear and charge too: a mother, 50 kg, stands her ground for her young).
CLIPS = ["stare", "graze", "drink", "sniff", "look", "scratch", "stretch", "sleep", "alarm", "rear", "play", "groom", "nuzzle", "flinch",
         "death", "watch", "approach", "charge"]
# Asleep it lies in its look (`wild.rs` settle): the sleep clip goes in the lying file too.
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", **{c: c for c in CLIPS}},
         "_sit": {"idle": "sit", "walk": "walk"},
         "_lie": {"idle": "lie", "walk": "walk", "sleep": "sleep"}}
FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
AXIS = ["neck", "skull", "chest", "hips"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "capybara.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    sys.path.insert(0, SKILL)
    from to_bedrock import export  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    from ground import feet_low, gait, grounded, joints, pokes  # noqa: E402
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    # No neck or head corner through the head or chest in any clip's key (the idle's grazing and the
    # run's rocking spine, made before this check, poke a little and stay as they are).
    poking = pokes(dict(spec, animations={c: spec["animations"][c] for c in CLIPS}), AXIS)
    assert not poking, f"poking through: {sorted((k, round(d, 2), w[:3]) for k, (d, w) in poking.items())}"
    for anim in ("walk", "run", "approach", "charge"):  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    # The gaits written as angles stand on their feet: the root's height from the feet, frame by frame
    # (the charge's bounce kept), and checked: no foot sinks into the ground or floats over it.
    for clip, bounds in (("approach", False), ("charge", True)):
        spec["animations"][clip] = grounded(spec, spec["animations"][clip], FEET, bounds)
        low = min(feet_low(spec, spec["animations"][clip], FEET))
        assert abs(low) < 0.3, f"{clip}'s feet at {low:.2f}, not on the ground"
    # The standing clips keep a foot on the ground in every frame (rearing on the hind ones; the play's
    # bounces leave it and land); none sinks.
    for clip in ("stare", "graze", "drink", "sniff", "look", "scratch", "stretch", "alarm", "rear", "groom", "nuzzle", "flinch", "watch"):
        low = feet_low(spec, spec["animations"][clip], FEET)
        assert min(low) > -0.3 and max(low) < 0.3, f"{clip}'s feet from {min(low):.2f} to {max(low):.2f}, not on the ground"
    # Rearing, the hind feet stay planted (the front ones lift).
    low = feet_low(spec, spec["animations"]["rear"], FEET[2:])
    assert min(low) > -0.3 and max(low) < 0.3, f"rearing, the hind feet from {min(low):.2f} to {max(low):.2f}"
    spec["animations"]["play"] = grounded(spec, spec["animations"]["play"], FEET, True)  # the bounces land on the ground
    assert abs(min(feet_low(spec, spec["animations"]["play"], FEET))) < 0.3, "the play lands in the ground"
    # Dying, it rolls onto its side: mid-roll its legs swing out below where it ends up lying, so each
    # frame rests on its lowest point (the fitted drop in `capybara.py` is only the last frame's).
    everything = [e["name"] for e in spec["elements"]]
    spec["animations"]["death"] = grounded(spec, spec["animations"]["death"], everything)
    low = feet_low(spec, spec["animations"]["death"], everything)
    assert min(low) > -0.3 and max(low) < 0.3, f"dying, its lowest point from {min(low):.2f} to {max(low):.2f}"
    # A clip played once must not loop (the engine loops a clip whose file says so, whatever the game asks).
    looping = [c for c in CLIPS if spec["animations"][c].get("loop") is True]
    assert sorted(looping) == ["approach", "charge", "drink", "graze", "sleep", "stare", "watch"], f"looping: {looping}"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "capybara", SCALE, "capybara/capybara", pose_files=POSES)
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another, checked
    # on the model as the game has it (`linecheck.py` on the exported geo).
    from linecheck import density_of, lines, load  # noqa: E402
    boxes, tex = load(os.path.join(ASSETS, "models", "capybara.geo.json"), os.path.join(ASSETS, "textures", "entity", "capybara", "capybara.png"))
    thin = [h for h in lines(boxes, tex, density_of(boxes), allow=()) if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    import shutil  # noqa: E402
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "capybara"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
