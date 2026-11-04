"""Lift tiles that read too dark or too saturated in the darker colour grade (Lighting, 11-04), in
code, so the approved drawing stays: a gamma lift (shadows more than highlights), a saturation
scale, and an optional pull of the hue toward a reference grey. Used for the grade review's proposals.

    python3 notes/blocks/regrade.py <in.png> <out.png> <gamma> <sat> [r,g,b pull 0..1]
"""
import sys

import numpy as np
from PIL import Image


def regrade(src, gamma, sat, pull=None, amount=0.0):
    a = np.asarray(Image.open(src).convert("RGBA")).astype(float) / 255
    rgb = a[..., :3] ** gamma
    lum = (rgb @ [0.2126, 0.7152, 0.0722])[..., None]
    rgb = lum + (rgb - lum) * sat
    if pull is not None and amount:
        ref = np.array(pull, float) / 255
        ref = ref / (ref @ [0.2126, 0.7152, 0.0722])
        rgb = rgb * (1 - amount) + lum * ref * amount
    a[..., :3] = np.clip(rgb, 0, 1)
    return Image.fromarray((a * 255).round().astype(np.uint8), "RGBA")


if __name__ == "__main__":
    pull = [int(v) for v in sys.argv[5].split(",")] if len(sys.argv) > 5 else None
    regrade(sys.argv[1], float(sys.argv[3]), float(sys.argv[4]), pull, float(sys.argv[6]) if len(sys.argv) > 6 else 0.5).save(sys.argv[2])
