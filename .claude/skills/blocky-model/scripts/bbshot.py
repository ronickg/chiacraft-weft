#!/usr/bin/env python3
"""Screenshots of the open Blockbench project from a few angles, side by side in one image, to
check the texture on the model (flicker, eyes, edges) without touching the app.

    .venv/bin/python tools/bbshot.py out/d_blocky/shots.png [--anim <name> [<seconds>]]

`--anim` shows that animation's frame at that time (default 0) instead of the rest pose.
"""

import base64
import io
import sys

from PIL import Image

from bb import run

# Camera directions (the model faces +z), each set back to frame the whole model: its bounds are
# measured in Blockbench, so a 15 cm egg and a 16 m dragon fill the shot alike.
ANGLES = {"front": (0, 0.36, 1.0), "3/4": (0.74, 0.53, 0.74), "side": (1.0, 0.3, 0.0), "back 3/4": (-0.74, 0.53, -0.74)}

BOUNDS = """
const box = new THREE.Box3();
for (const c of Outliner.elements) if (c.mesh) box.expandByObject(c.mesh);
const s = box.getSize(new THREE.Vector3()), m = box.getCenter(new THREE.Vector3());
return [m.x, m.y, m.z, Math.max(s.x, s.y, s.z)];
"""

JS = """
const p = Preview.selected;
p.camera.position.set(%f, %f, %f);
p.controls.target.set(%f, %f, %f);
p.controls.update();
p.render();
return p.canvas.toDataURL('image/png');
"""


def shot(direction, bounds):
    cx, cy, cz, size = bounds
    d = [v / max(sum(w * w for w in direction) ** 0.5, 1e-9) for w in [0] for v in direction]
    dist = 1.6 * size
    pos = (cx + d[0] * dist, cy + d[1] * dist, cz + d[2] * dist)
    url = run(JS % (*pos, cx, cy, cz))
    return Image.open(io.BytesIO(base64.b64decode(url.split(",", 1)[1]))).convert("RGB")


POSE = """
Modes.options.animate.select();
const a = Animation.all.find(a => a.name === "%s");
a.select();
Timeline.setTime(%f);
Animator.preview();
return true;
"""


if __name__ == "__main__":
    if "--anim" in sys.argv:
        i = sys.argv.index("--anim")
        t = float(sys.argv[i + 2]) if len(sys.argv) > i + 2 else 0.0
        run(POSE % (sys.argv[i + 1], t))
    bounds = run(BOUNDS)
    imgs = [shot(d, bounds) for d in ANGLES.values()]
    w, h = imgs[0].size
    sheet = Image.new("RGB", (w * len(imgs), h))
    for i, im in enumerate(imgs):
        sheet.paste(im, (i * w, 0))
    sheet.save(sys.argv[1])
    print(sheet.size)
