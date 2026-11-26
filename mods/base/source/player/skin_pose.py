#!/usr/bin/env python3
"""The player's `skin` clip (kneeling over a carcass, `animals` skinning), fitted on Weft's humanoid (`humanoid.py`: no
parent bones, the body hanging from the neck). The torso, head and arms turn by `LEAN` about the hips, each moved by a
position key so its joint stays where the lean takes it (the head on the neck, the arms at the shoulders, the torso on
the hips); the legs, one box each, run back from the hips to the knees on the ground; the root drops until the lowest
corner is just above the ground. Prints the clip's bones and checks; `--sheet out.png` draws keys 0 and 0.4."""
import json
import os
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from humanoid import PARTS, boxes, draw, euler  # noqa: E402

HIP = np.array([0, 12, 0.0])
LEAN, HEAD, LEGS, SPREAD = 30, 10, 60, 5
RIGHT = {"0": [-70, 10, -4], "0.4": [-55, -16, -2], "0.75": [-74, 12, -6], "1.1": [-55, -16, -2], "1.5": [-70, 10, -4]}
LEFT = {"0": [-65, -12, 6], "0.75": [-68, -8, 8], "1.5": [-65, -12, 6]}


def pose(right, left, head=(HEAD, 0, 0)):
    p = {"rightLeg": [LEGS, SPREAD, 0], "leftLeg": [LEGS, -SPREAD, 0]}
    R = euler([LEAN, 0, 0])
    for n, extra in (("body", (0, 0, 0)), ("head", head), ("rightArm", right), ("leftArm", left)):
        piv = np.array(PARTS[n][0], float)
        p[n + "_pos"] = [round(float(v), 2) for v in R @ (piv - HIP) + HIP - piv]
        p[n] = [LEAN + extra[0], extra[1], extra[2]]
    lo = min(w[:, 1].min() for n, w, _ in boxes(dict(p, root_pos=[0, 0, 0])))
    p["root_pos"] = [0, round(float(-lo) + 0.1, 2), 0]
    return p


def main():
    p0 = pose(RIGHT["0"], LEFT["0"])
    for t in RIGHT:
        p = pose(RIGHT[t], LEFT.get(t, LEFT["0"]))
        lo = min(w[:, 1].min() for _, w, _ in boxes(p))
        assert lo > 0.05, (t, lo)  # nothing under the ground
    head = [w for n, w, _ in boxes(p0) if n == "head"][0]
    print(json.dumps(p0), f"\nhead centre {head[:, 1].mean() / 16:.2f} blocks, top {head[:, 1].max() / 16:.2f}")
    if "--sheet" in sys.argv:
        from PIL import Image
        poses = [p0, pose(RIGHT["0.4"], LEFT["0"])]
        views = [(90, 0), (-60, 15), (150, 15)]
        sheet = Image.new("RGB", (360 * len(views), 360 * len(poses)))
        for i, p in enumerate(poses):
            for j, (yw, pt) in enumerate(views):
                sheet.paste(draw(p, yw, pt), (360 * j, 360 * i))
        sheet.save(sys.argv[sys.argv.index("--sheet") + 1])


if __name__ == "__main__":
    main()
