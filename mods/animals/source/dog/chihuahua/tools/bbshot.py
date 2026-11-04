#!/usr/bin/env python3
"""Screenshots of the open Blockbench project from a few angles, side by side in one image, to
check the texture on the model (flicker, eyes, edges) without touching the app.

    .venv/bin/python tools/bbshot.py out/d_blocky/shots.png
"""

import base64
import io
import sys

from PIL import Image

from bb import run

# Camera positions (Blockbench units, the dog faces +z, about 56 tall).
ANGLES = {"front": (0, 34, 95), "3/4": (70, 50, 70), "side": (100, 30, 0), "back 3/4": (-70, 50, -70)}

JS = """
const p = Preview.selected;
p.camera.position.set(%f, %f, %f);
p.controls.target.set(0, 24, 0);
p.controls.update();
p.render();
return p.canvas.toDataURL('image/png');
"""


def shot(pos):
    url = run(JS % pos)
    return Image.open(io.BytesIO(base64.b64decode(url.split(",", 1)[1]))).convert("RGB")


if __name__ == "__main__":
    imgs = [shot(p) for p in ANGLES.values()]
    w, h = imgs[0].size
    sheet = Image.new("RGB", (w * len(imgs), h))
    for i, im in enumerate(imgs):
        sheet.paste(im, (i * w, 0))
    sheet.save(sys.argv[1])
    print(sheet.size)
