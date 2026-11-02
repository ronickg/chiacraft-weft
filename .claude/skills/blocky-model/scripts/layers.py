#!/usr/bin/env python3
"""Layers for coats painted on the fly: genes, breeding and a paint screen recolor a creature
without repainting it. Next to its texture a creature gets, per texel:

- `info.png`: R 255 for coat (recolored), 0 for fixed parts (eyes, nose, teeth, claws: kept as
  painted); G the light level (ramp index x 10, as `paint.tones` builds it, 0..240); B how far
  toward the underside color (0 the main coat, 255 the underside, between at soft edges).
- `pos.png`: where the texel is on the body at rest (x, y, z scaled into `layers.json` bounds),
  so patterns (stripes, spots, patches) run across faces and seams in 3D.
- `props.png`: R white spotting propensity (white appears where it is above 1 - the amount, so
  paws, chest and belly whiten first); G points (where tan points or a colorpoint's dark shows:
  brows, muzzle, chest, legs); B mask (a dark face mask: the muzzle).
- `props2.png`: R extremity (colorpoint: ears, face, paws, tail tip); G where body patterns show
  (back and sides yes, face and belly less).

All four use the creature's UV layout (packed in the same order and sizes as the texture). The
viewer's coat renderer and the game read them with the species' gene file. Painters record each
face with `Recorder.face`; builds save with `Recorder.save`.

A material's role (per species): `("coat", u, shift)` — part of the coat, `u` toward the
underside (0..1), `shift` light steps (negative darker: a darker back, feather tips); or "fixed".
"""

import json
import os

import numpy as np
from PIL import Image

STEP = 0.25  # paint.STEP: light levels are ramp steps of this


def _rgba(r, g, b):
    h, w = r.shape
    return np.dstack([np.clip(np.round(c), 0, 255) for c in (r, g, b)] + [np.full((h, w), 255)]).astype(np.uint8)


class Recorder:
    """Collects layer tiles alongside a painter's texture tiles (same keys)."""

    def __init__(self, bounds):
        self.lo, self.hi = np.array(bounds[0], float), np.array(bounds[1], float)
        self.tiles = {}

    def face(self, key, weights, level, roles, smooth_rgb, final_rgb, p, props):
        """One face: `weights` (n, h, w) and `level` (h, w) from `paint_face_smooth(out=...)`;
        `roles` per material; `smooth_rgb` the painter's colors before stamps (eyes, brows) and
        `final_rgb` after: a stamp that only darkens becomes a light shift, any other is fixed;
        `p` (h, w, 3) rest positions; `props` dict of (h, w) 0..1 maps (white, points, mask,
        extremity, pattern; missing ones 0, pattern 1)."""
        n, h, w = weights.shape
        coat_w = np.zeros((h, w))
        u = np.zeros((h, w))
        shift = np.zeros((h, w))
        for m in range(n):
            r = roles[m] if m < len(roles) else "fixed"
            if r == "fixed":
                continue
            _, um, sm = r
            coat_w += weights[m]
            u += weights[m] * um
            shift += weights[m] * sm
        coat = coat_w > 0.5
        u = u / np.maximum(coat_w, 1e-6)
        lvl = level + shift / np.maximum(coat_w, 1e-6) / STEP
        # Stamps after painting: a pure darkening (a brow) is light; anything else stays as painted.
        a, b = np.asarray(smooth_rgb, float), np.asarray(final_rgb, float)[..., :3]
        changed = np.abs(a - b).max(-1) > 4
        ratio = b / np.maximum(a, 1)
        dark = changed & (ratio.std(-1) < 0.06) & (ratio.mean(-1) < 1)
        lvl = np.where(dark, lvl + (ratio.mean(-1) - 1) / 0.15 / STEP, lvl)
        coat = coat & ~(changed & ~dark)
        lvl = np.clip(lvl, 0, 24)
        q = (p - self.lo) / np.maximum(self.hi - self.lo, 1e-6) * 255
        g = lambda k, d=0.0: np.clip(props.get(k, np.full((h, w), d)), 0, 1) * 255  # noqa: E731
        self.tiles[key] = (_rgba(coat * 255, lvl * 10, u * 255), _rgba(q[..., 0], q[..., 1], q[..., 2]),
                           _rgba(g("white"), g("points"), g("mask")), _rgba(g("extremity"), g("pattern", 1.0), np.zeros((h, w))))

    def save(self, out_dir, keys, pack, width=512, extra=None):
        """Packs the layers in `keys` order (the texture's tile order) with `pack` (the painter's
        packer, so they land where the texture's tiles do) and writes them into `out_dir/layers`."""
        d = os.path.join(out_dir, "layers")
        os.makedirs(d, exist_ok=True)
        for i, name in enumerate(("info", "pos", "props", "props2")):
            sheet, _ = pack([(k, self.tiles[k][i]) for k in keys], width)
            Image.fromarray(sheet, "RGBA").save(os.path.join(d, f"{name}.png"))
        meta = {"bounds": [self.lo.tolist(), self.hi.tolist()], "step": STEP, **(extra or {})}
        json.dump(meta, open(os.path.join(d, "layers.json"), "w"))
        return d


def bounds(elements, pad=1.0):
    """The model's rest box (all elements), a little padded: what `pos.png` is scaled into."""
    lo = np.min([e["lo"] for e in elements], 0) - pad
    hi = np.max([e["hi"] for e in elements], 0) + pad
    return lo, hi
