#!/usr/bin/env python3
"""Rebuilds the wyvern: lays out, paints and animates the model (wyvern.py, into out/) and exports
the game's files into the mod (`../../assets`): models/wyvern.geo.json,
animations/wyvern[_rest].animation.json and textures/entity/wyvern/wyvern.png. Blockbench isn't
needed for this.

    ../dog/chihuahua/.venv/bin/python build.py     (or any Python with numpy, pillow and scipy)

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

sys.path.insert(0, SKILL)
from house import SCALE  # noqa: E402  (the house style: 64 texels a block)

# Which animations go in which file: the engine plays `idle`, `walk` and `run` by itself (walk
# and run by speed); `breathe`, `fly`, `glide` and the attacks (`bite`, `claw`, `tail`) are there
# for code to play. Lying down (asleep,
# knocked out) is its own look, whose file holds it in `idle`.
# The behaviour clips (`mods/animals/src/anim.rs`, `wyvern.py`) go in the standing file too, by their
# own names; asleep it lies in its resting look, so the sleep clip goes in the resting file too. A big
# hunter that doesn't ambush: no graze, rear, charge, stalk or pounce.
HELD = ["stare", "watch", "drink", "sleep"]
GAITS = ["approach"]
ONCE = ["alarm", "sniff", "look", "scratch", "stretch", "play", "groom", "nuzzle", "flinch"]
STRIKES = ["bite", "claw", "tail", "breathe"]
CLIPS = HELD + GAITS + ONCE + ["death"]
POSES = {"": {"idle": "idle", "walk": "walk", "run": "run", "breathe": "breathe", "fly": "fly", "glide": "glide",
            "bite": "bite", "claw": "claw", "tail": "tail", **{c: c for c in CLIPS}},
         "_rest": {"idle": "rest", "sleep": "sleep"}}
# What it stands on: each hind foot (its toes and claws) and each wrist (the hand of the folded wing).
SIDE_FEET = {f"{part}_{s}": [f"{n}_{s}" for n in names] for s in ("L", "R") for part, names in
             (("hind", ["toe0", "toe1", "toe2", "toeclaw0", "toeclaw1", "toeclaw2", "toetip0", "toetip1", "toetip2", "hindtoe",
                        "hindclaw"]), ("wrist", ["hand", "thumbclaw"]))}
# Where a wrist leaves the ground: the stretch lifts both wings (stands on its hind legs while they're
# up), the play's hop the forequarters, the scratch the left wrist (fractions of the clip).
LIFTED = {"stretch": {"wrist_L": (0.08, 0.94), "wrist_R": (0.08, 0.94)}, "play": {"wrist_L": (0.3, 0.62), "wrist_R": (0.3, 0.62)},
          "scratch": {"wrist_L": (0.0, 1.0)}}

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "wyvern.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    from to_bedrock import export  # noqa: E402
    from ground import feet_low, gait, joints  # noqa: E402

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
    for anim in ["walk", "run"] + GAITS:  # a foot in the air must swing forward, or it moonwalks
        back = {f: v for f, v in gait(spec, anim, ["foot_L", "foot_R", "thumbclaw_L", "thumbclaw_R"]).items()
                if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"

    # The behaviour clips. Only the held ones and the gait loop (the engine loops a clip whose file
    # says so, whatever the game asks); the strikes and one-offs play once, the death holds its last frame.
    A = spec["animations"]
    looping = sorted(c for c in CLIPS if A[c].get("loop") is True)
    assert looping == sorted(HELD + GAITS), f"looping: {looping}"
    assert A["death"]["loop"] == "hold_on_last_frame"
    assert all(not A[c].get("loop") for c in ONCE), "a one-off loops"
    sys.path.insert(0, HERE)
    import wyvern  # noqa: E402  (its measures: membranes by their cut outlines, scaled as posed)
    wyvern.SKELETON[:] = spec["bones"]

    def pose(fr):
        return dict(fr["rot"], root=fr["root"], **({"scale": fr["scale"]} if fr.get("scale") else {}))

    def lowest(clip, names=None):
        return [wyvern.low_points(pose(fr), names) for fr in A[clip]["frames"]]
    # Standing, every foot stays on the ground in every keyframe (a wrist off it only where it's lifted),
    # nothing of it goes into the ground (the snout dipped to drink, a wing swung).
    for clip in GAITS:  # stepping, a foot is down in every keyframe (each frame on its lowest foot)
        low = feet_low(spec, A[clip], [n for names in SIDE_FEET.values() for n in names])
        assert min(low) > -0.3 and max(low) < 0.3, f"{clip}'s lowest foot from {min(low):.2f} to {max(low):.2f}"
    for clip in [c for c in CLIPS if c not in ("sleep", "death")]:
        frames = A[clip]["frames"]
        for foot, names in ({} if clip in GAITS else SIDE_FEET).items():
            low = feet_low(spec, A[clip], names)
            span = LIFTED.get(clip, {}).get(foot)
            down = [v for f, v in zip(frames, low) if not (span and span[0] * A[clip]["length"] < f["t"] < span[1] * A[clip]["length"])]
            assert min(low) > -0.3 and max(down) < 0.3, f"{clip}: {foot} from {min(low):.2f} to {max(down):.2f}, not on the ground"
        low = min(lowest(clip))
        assert low > -0.3, f"{clip} goes {low:.2f} into the ground"
    for clip in LIFTED:  # (and is really lifted)
        assert max(feet_low(spec, A[clip], SIDE_FEET["wrist_L"])) > 15, f"{clip} doesn't lift its wrist"
    # Lying asleep and dead, each frame rests on its lowest point.
    for clip in ("sleep", "death"):
        low = lowest(clip)
        assert min(low) > -0.3 and max(low) < 0.3, f"{clip}: its lowest point from {min(low):.2f} to {max(low):.2f}"
    # The head laid on the ground at the snout to drink (half a unit over it).
    snout = min(lowest("drink", set(wyvern.MOUTH_PARTS)))
    assert -0.3 < snout < 1.0, f"drinking, the snout {snout:.2f} over the ground"
    # The wings: no part of a wing goes into the body or the other wing (beyond the stand's own, by
    # half a unit) in any keyframe, opening, lifting or lying dead; nor an upper arm into the neck or
    # head. Grooming and scratching, the head touches the left wing (the snout along the forearm, the
    # cheek on the thumb's claw), a little.
    body = wyvern.body_parts(spec)
    head_neck = [n for n in body if n.startswith(wyvern.HEAD_PARTS + ("neck_",)) and n != "neck_0"]  # (neck_0's root is in the chest)

    def hits(q):
        out = {}
        for s, o in (("L", "R"), ("R", "L")):
            out.update(wyvern.overlaps(spec, q, wyvern.wing_parts(spec, s), body + wyvern.wing_parts(spec, o)))
            out.update(wyvern.overlaps(spec, q, [f"arm_{s}"], head_neck))
        return out
    base = hits(pose(A["idle"]["frames"][0]))
    for clip in CLIPS:
        for fr in A[clip]["frames"]:
            bad = {k: round(v, 1) for k, v in hits(pose(fr)).items() if v > base.get(k, 0.0) + 0.5
                   and not (clip in ("scratch", "groom") and k[0].endswith("_L") and k[1].startswith(wyvern.HEAD_PARTS) and v < 3)}
            assert not bad, f"{clip}@{fr['t']}: a wing in the body: {bad}"
    # The keys run on: no bone's angles jump between one key and the next (the engine blends the
    # angles, so a turn written two ways spins the part round).
    for clip in CLIPS:
        frames = A[clip]["frames"]
        for f0, f1 in zip(frames, frames[1:]):
            jump = max(max(abs(a - b) for a, b in zip(f0["rot"][n], f1["rot"][n])) for n in f0["rot"])
            assert jump < 40, f"{clip}: a bone turns {jump:.0f} degrees from {f0['t']} to {f1['t']}"

    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "wyvern", SCALE, "wyvern/wyvern", pose_files=POSES)
    # Its coat layers, for coats painted from genes in the game (`core/src/coats.rs`).
    import shutil  # noqa: E402
    shutil.copytree(os.path.join(OUT, "layers"), os.path.join(HERE, "..", "..", "genetics", "layers", "wyvern"), dirs_exist_ok=True)
    print("wrote", made, "into", os.path.normpath(ASSETS))
