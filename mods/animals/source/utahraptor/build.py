#!/usr/bin/env python3
"""Rebuilds the Utahraptor: paints the model and its animations (utahraptor.py, into out/) and exports the
game's files into the mod (`../../assets`): models/utahraptor.geo.json,
animations/utahraptor[_rest].animation.json and textures/entity/utahraptor/utahraptor.png. Blockbench
isn't needed for this.

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

SCALE = 0.83  # model pixels per unit: 1.5 blocks at the hip, about 4.7 long (a real Utahraptor: 1.5 m, 4.7-6 m)
# Which animations go in which file: the engine plays `idle` and `walk` by itself, a pose's
# file holds it in `idle`; `run` and `meow` are there for code to play.
# The behaviour clips (`mods/animals/src/anim.rs`, `utahraptor.py`) go in the standing file too, by
# their own names; asleep it lies in its resting look, so the sleep clip goes in the resting file too.
# A hunter: no rear or charge (those are for plant eaters and mothers that stand their ground).
HELD = ["stare", "watch", "drink", "sleep"]
GAITS = ["stalk", "approach"]
ONCE = ["alarm", "pounce", "sniff", "look", "scratch", "stretch", "play", "groom", "nuzzle", "flinch"]
STRIKES = ["screech", "bite", "kick"]
CLIPS = HELD + GAITS + ONCE + ["death"]
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", **{c: c for c in STRIKES + CLIPS}},
         "_rest": {"idle": "rest", "walk": "walk", "sleep": "sleep"}}
FEET = ["toe3_L", "toe3_R", "toe4_L", "toe4_R"]  # toe II and its sickle claw are held up
LEAP = (0.4, 0.78)  # the pounce is a leap: off the ground only from 0.4 to 0.78 s (`utahraptor.LEAP`)

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "utahraptor.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    sys.path.insert(0, SKILL)
    from to_bedrock import export  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    from ground import feet_low, gait, joints, pokes  # noqa: E402
    loose = joints(spec)  # every joint inside both parts it joins, or they come apart when it turns
    assert not loose, f"loose joints: {loose}"
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another,
    # measured at the size the game draws it (a layout unit here is 4 * SCALE of linecheck's).
    from linecheck import boxes_from_spec, lines  # noqa: E402
    from PIL import Image  # noqa: E402
    import numpy as np  # noqa: E402
    boxes = [dict(b, unit=4 * SCALE) for b in boxes_from_spec(spec)]
    thin = [h for h in lines(boxes, np.array(Image.open(os.path.join(OUT, "texture.png")).convert("RGBA"))) if h["kind"] == "line"]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    # Along the body's axis nothing pokes out of its neighbor in any pose (a neck's corner through
    # the head, a tail segment's through the one before).
    poking = pokes(spec, ["neck", "skull", "chest", "hips", "tail_"])
    assert not poking, "poking through: " + "; ".join(f"{a} out of {b} by {d:.2f} ({w[0]})" for (a, b), (d, w) in poking.items())
    for anim in ["walk", "run"] + GAITS:  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, ["toe3_L", "toe3_R"]).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    # Only the held clips and the gaits loop (the engine loops a clip whose file says so, whatever the
    # game asks: a hit raptor would flinch forever); the strikes and the one-offs play once, the
    # death holds its last frame.
    A = spec["animations"]
    looping = sorted(c for c, a in A.items() if a.get("loop") is True)
    assert looping == sorted(HELD + GAITS), f"looping: {looping}"
    assert all(not A[c].get("loop") for c in STRIKES), "a strike loops"
    assert A["death"]["loop"] == "hold_on_last_frame"
    # The gaits stand on their feet in every frame (`ground.grounded` in `utahraptor.py`).
    for clip in GAITS:
        low = feet_low(spec, A[clip], FEET)
        assert max(abs(v) for v in low) < 0.3, f"{clip}'s feet at {min(low):.2f}..{max(low):.2f}, not on the ground"
    # Every standing clip (the strikes and the run included) keeps a foot on the ground in every
    # keyframe, none sinking, and nothing goes into the ground; the pounce leaves it only in its leap.
    everything = [e["name"] for e in spec["elements"]]
    for clip in ["idle", "walk", "run"] + STRIKES + HELD + GAITS + ONCE:
        a = A[clip]
        if clip == "sleep":  # lying: the belly on the ground, the legs folded
            # Not the shaggy thigh feathers: hung from the hips 4 units below them, lying they go 3.2 into
            # the ground, as in the resting look (the model's, not the clip's: kept as it is).
            low = feet_low(spec, a, [n for n in everything if not n.startswith("shag_")])
            assert min(low) > -0.3 and max(low) < 0.3, f"asleep, its lowest point from {min(low):.2f} to {max(low):.2f}"
            assert min(feet_low(spec, a, ["shag_L", "shag_R"])) > -3.5, "the thigh feathers sink deeper asleep than resting"
            continue
        low = feet_low(spec, a, FEET)
        # (The run's own bounce leaves the ground: only its sinking is checked. The walk stands on its
        # planted foot in every frame.)
        down = [v for f, v in zip(a["frames"], low) if not (clip == "pounce" and LEAP[0] < f["t"] < LEAP[1])
                and clip != "run"] or [0.0]
        assert min(low) > -0.3 and max(down) < 0.3, f"{clip}'s feet from {min(low):.2f} to {max(down):.2f}, not on the ground"
        assert min(feet_low(spec, a, everything)) > -0.3, f"{clip} goes into the ground"
    assert max(feet_low(spec, A["pounce"], FEET)) > 3, "the pounce doesn't leave the ground"
    # Dying, it topples onto its side: each frame rests on its lowest point.
    low = feet_low(spec, A["death"], everything)
    assert min(low) > -0.3 and max(low) < 0.3, f"dying, its lowest point from {min(low):.2f} to {max(low):.2f}"
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "utahraptor", SCALE, "utahraptor/utahraptor", pose_files=POSES)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    import shutil  # noqa: E402
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "utahraptor"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
