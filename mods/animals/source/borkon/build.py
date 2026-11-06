#!/usr/bin/env python3
"""Rebuilds Borkon: paints the model and its clips (borkon.py, into out/) and exports the game's
files into the mod (`../../assets`): models/borkon.geo.json, animations/borkon.animation.json
(idle, walk and run, the fight's clips, and watch, stare and death by name) and textures/entity/borkon/borkon.png.

    ~/.venvs/blocky/bin/python build.py     (any Python with numpy, scipy and Pillow)
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
sys.path.insert(0, HERE)
from house import SCALE  # noqa: E402  the house density: 32 texels a block

FEET = ["leg_FL_paw", "leg_FR_paw", "leg_BL_paw", "leg_BR_paw"]
AXIS = ["neck", "skull", "chest", "hips"]

if __name__ == "__main__":
    subprocess.run([sys.executable, os.path.join(HERE, "borkon.py")], check=True)
    subprocess.run([sys.executable, os.path.join(SKILL, "boxcheck.py"), os.path.join(OUT, "spec.json")], check=True)
    from to_bedrock import export  # noqa: E402
    from ground import gait, joints, pokes  # noqa: E402
    from linecheck import boxes_from_spec, lines  # noqa: E402
    from borkon import CLIPS  # noqa: E402

    spec = json.load(open(os.path.join(OUT, "spec.json")))
    loose = joints(spec)
    assert not loose, f"loose joints: {loose}"
    # No thin lines (Ronald, 11-05): no strip of a box under two texels showing beside another.
    
    from PIL import Image  # noqa: E402
    import numpy as np  # noqa: E402
    thin = [h for h in lines(boxes_from_spec(spec), np.array(Image.open(os.path.join(OUT, "texture.png")).convert("RGBA")),
                             allow=()) if h["kind"] in ("line", "thin")]
    assert not thin, f"thin lines: {[(h['box'], h['face'], h['cover'], h['width']) for h in thin]}"
    poking = pokes(spec, AXIS)
    assert not poking, f"poking through: {sorted(poking)}"
    for anim in ("walk", "run"):
        back = {f: v for f, v in gait(spec, anim, FEET).items() if v is not None and v <= 0}
        assert not back, f"{anim} goes backwards: {back}"
    from ground import feet_low, grounded  # noqa: E402
    from borkon import ANIMS, STARE_SCRAPES  # noqa: E402
    A = spec["animations"]
    # Only the held clips loop (`boss.rs` holds watch and stare; the engine loops a clip whose file
    # says so); the fight's one-offs play once; death holds its last frame.
    for c in CLIPS:
        want = "hold_on_last_frame" if c == "death" else c in ("stunned", "down", "watch", "stare")
        assert A[c]["loop"] == want, f"{c} loops {A[c]['loop']}, not {want}"
    # Watch and stare stand on all four hooves in every keyframe (the scraping forehoof excepted
    # while it scrapes: it starts and ends in its place); nothing goes into the ground.
    everything = [e["name"] for e in spec["elements"]]
    for c in ("watch", "stare"):
        for f in FEET:
            for fr, y in zip(A[c]["frames"], feet_low(spec, A[c], [f])):
                p = fr["t"] / ANIMS[c][1]
                if c == "stare" and f == "leg_FR_paw" and any(a < p < b for a, b in STARE_SCRAPES):
                    assert y > -0.3, f"stare's scraping hoof at {y:.2f}, in the ground, at {fr['t']}"
                else:
                    assert abs(y) < 0.3, f"{c}'s {f} at {y:.2f}, not on the ground, at {fr['t']}"
        low = feet_low(spec, A[c], everything)
        assert min(low) > -0.3, f"{c} goes {min(low):.2f} into the ground"
    # Dying, it kneels, sinks and rolls onto its side: each frame rests on its lowest point (the
    # legs and horns swing below where it ends up mid-fall).
    A["death"] = grounded(spec, A["death"], everything)
    low = feet_low(spec, A["death"], everything)
    assert min(low) > -0.3 and max(low) < 0.3, f"dying, its lowest point from {min(low):.2f} to {max(low):.2f}"
    # The standing fight clips (and the idle) on their planted hooves in every keyframe (`plant` in
    # borkon.py; the hooves a move lifts may only not go into the ground).
    from borkon import LEGS, PLANTED, planted  # noqa: E402
    for c in PLANTED:
        a = A[c]
        for g in LEGS:
            for fr, y in zip(a["frames"], feet_low(spec, a, [f"leg_{g}_paw"])):
                t = fr["t"] if fr["t"] < a["length"] or a["loop"] is not True else 0.0
                if g in planted(c, t):
                    assert abs(y) < 0.3, f"{c}'s {g} hoof at {y:.2f}, not on the ground, at {fr['t']}"
                else:
                    assert y > -0.3, f"{c}'s lifted {g} hoof at {y:.2f}, in the ground, at {fr['t']}"
    for c in ("walk", "run", "charge"):
        low = min(feet_low(spec, A[c], FEET))
        assert abs(low) < 0.3, f"{c}'s feet at {low:.2f}, not on the ground"
    # Nothing goes into the ground in any clip, but `rise`, which climbs out of it.
    for c, a in A.items():
        if c != "rise":
            low = min(feet_low(spec, a, everything))
            assert low > -0.3, f"{c} goes {low:.2f} into the ground"
    poses = {"": {"idle": "idle", "walk": "walk", "run": "run", **{c: c for c in CLIPS}}}
    made = export(spec, os.path.join(OUT, "texture.png"), ASSETS, "borkon", SCALE, "borkon/borkon", pose_files=poses)
    print("wrote", made, "into", os.path.normpath(ASSETS))
