#!/usr/bin/env python3
"""The glider's frame seen from one's own eyes while gliding (`mods/base/src/glider.rs`: an overlay
across the top of the view, `hud_anchor::TOP`): the wing model (`build.py`) drawn in perspective
from just under its hand bar, looking ahead and a little up, so the underside of the hide and the
sticks running to the nose fill the top of the view. Drawn from the model and its texture, texel for
texel, so it matches what others see over the player.

    ~/.venvs/blocky/bin/python frame_view.py [out.png]   (default: ../../assets/textures/gui/glider_frame.png)

A real camera at the eyes sees none of it (the wing is 56 degrees and more over the eye line); the
overlay is the game's way of saying "the glider is open over you", as hats and visors do.
"""

import math
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import build  # noqa: E402

# The picture: its size in texels (drawn at a GUI pixel a texel, stretched to the screen's width),
# the eye (layout units, under the bar and a little behind it), where it looks and how wide.
SIZE = (640, 120)
EYE = np.array([0.0, 104.0, -100.0])
PITCH = 22.0
FOV_X = 80.0
# The picture's lower edge fades out over this many rows, so it sits on the view instead of a seam.
FADE = 18


def faces(spec, sheet):
    """Every drawn face: (corner, U, V, uv rect, quad) in layout units, tilted elements turned."""
    from prop import FACES, face_frame, rotm

    out = []
    for e in spec["elements"]:
        for face, rect in e["faces"].items():
            if rect is None:
                continue
            corner, du, dv, w, h, _ = face_frame(e["lo"], e["hi"], face, 1.0)
            size = np.array(e["hi"]) - np.array(e["lo"])
            U, V = du * abs(size @ du), dv * abs(size @ dv)
            if e.get("rot"):
                R, o = rotm(e["rot"]), np.array(e["origin"], float)
                corner = (corner - o) @ R.T + o
                U, V = U @ R.T, V @ R.T
            out.append((corner, U, V, rect, bool(e.get("quad"))))
    return out


def render(spec, sheet):
    W, H = SIZE
    fx = math.tan(math.radians(FOV_X / 2))
    fy = fx * H / W
    p = math.radians(PITCH)
    fwd = np.array([0.0, math.sin(p), math.cos(p)])
    right = np.array([-1.0, 0.0, 0.0])  # the model faces +z: its right hand is -x
    up = np.cross(fwd, right) * -1
    gx, gy = np.meshgrid((np.arange(W) + 0.5) / W * 2 - 1, 1 - (np.arange(H) + 0.5) / H * 2)
    # The picture is the TOP of a taller view: rows map to the upper part of the frustum.
    d = fwd + gx[..., None] * fx * right + (gy[..., None] * 0.5 + 0.5) * fy * 2 * up
    img = np.zeros((H, W, 4), np.uint8)
    zb = np.full((H, W), np.inf)
    th, tw = sheet.shape[:2]
    for corner, U, V, rect, quad in faces(spec, sheet):
        n = np.cross(U, V)
        denom = d @ n
        with np.errstate(divide="ignore", invalid="ignore"):
            t = ((corner - EYE) @ n) / denom
        hit = EYE + d * t[..., None] - corner
        # Coordinates along U and V (they are perpendicular box edges).
        s = hit @ U / (U @ U)
        r = hit @ V / (V @ V)
        ok = (t > 0) & (s >= 0) & (s < 1) & (r >= 0) & (r < 1) & (t < zb)
        if not ok.any():
            continue
        u0, v0, u1, v1 = rect
        back = denom > 0  # seen from inside the box's face direction: the quad's mirrored back
        su = np.where(back & quad, 1 - s, s)
        tu = np.clip(np.floor(u0 + su * (u1 - u0)).astype(int), 0, tw - 1)
        tv = np.clip(np.floor(v0 + r * (v1 - v0)).astype(int), 0, th - 1)
        col = sheet[tv, tu]
        ok &= col[..., 3] > 127
        if not quad:
            ok &= ~back
        zb[ok] = t[ok]
        img[ok] = col[ok]
    # Softer underneath (it's the shaded side), and fading out at the lower edge.
    img[..., :3] = (img[..., :3] * 0.82).astype(np.uint8)
    fade = np.clip((H - 1 - np.arange(H)) / FADE, 0, 1)
    img[..., 3] = (img[..., 3] * fade[:, None]).astype(np.uint8)
    return img


def main(out):
    import tempfile

    tmp = tempfile.mkdtemp()
    spec = build.build(tmp)
    sheet = np.array(Image.open(os.path.join(tmp, "textures", "glider_wing.png")).convert("RGBA"))
    img = render(spec, sheet)
    os.makedirs(os.path.dirname(out), exist_ok=True)
    Image.fromarray(img).save(out)
    print(out, img.shape[1], "x", img.shape[0], "covered", round(float((img[..., 3] > 0).mean()), 2))


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else os.path.join(HERE, "..", "..", "assets", "textures", "gui", "glider_frame.png"))
