#!/usr/bin/env python3
"""Rig and pose helpers for box models (Blockbench / Hytale conventions).

    from rig import quat, qmul, qmat, euler, swing, fit

Rotations are Euler degrees (x, y, z) composed R = Rz Ry Rx, the order Blockbench and Hytale
use. +X pitches a bone's +Z end down, +Y turns +Z toward +X, +Z rolls +Y toward -X.
"""

import itertools
import math

import numpy as np


def quat(rx, ry, rz):
    """Quaternion (x, y, z, w) for Euler degrees, R = Rz Ry Rx."""
    def axis(a, i):
        h = math.radians(a) / 2
        q = [0.0, 0.0, 0.0, math.cos(h)]
        q[i] = math.sin(h)
        return np.array(q)
    return qmul(axis(rz, 2), qmul(axis(ry, 1), axis(rx, 0)))


def qmul(a, b):
    ax, ay, az, aw = a
    bx, by, bz, bw = b
    return np.array([aw * bx + ax * bw + ay * bz - az * by,
                     aw * by - ax * bz + ay * bw + az * bx,
                     aw * bz + ax * by - ay * bx + az * bw,
                     aw * bw - ax * bx - ay * by - az * bz])


def qmat(q):
    x, y, z, w = q
    return np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                     [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                     [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])


def euler(q):
    """Euler degrees (x, y, z) for quaternion q, in quat()'s convention; round-trips."""
    R = qmat(q)
    ry = math.degrees(math.asin(max(-1.0, min(1.0, -R[2, 0]))))
    return (math.degrees(math.atan2(R[2, 1], R[2, 2])), ry, math.degrees(math.atan2(R[1, 0], R[0, 0])))


def swing(segment, angle, base=(0, 0, 0), side=(1.0, 0.0, 0.0)):
    """Euler rotation for a bone: its `base` pose, then a side-to-side swing of `angle` degrees
    about the axis across the bone (perpendicular to `segment`, the bone's direction at rest, and
    to `side`, the creature's sideways axis). Use for tails, necks, antennae, trunks: a part that
    points up and swings about the world's vertical axis would only twist in place."""
    a = np.cross(np.asarray(segment, float), np.asarray(side, float))
    a /= np.linalg.norm(a)
    h = math.radians(angle) / 2
    return euler(qmul(quat(*base), np.array([*(a * math.sin(h)), math.cos(h)])))


def fit(grid, cost):
    """Grid search: `grid` {param: values}, `cost(params) -> float`. Returns (best cost, params).
    Use with ground.lowest() to fit resting poses: e.g. cost = front_feet**2 + back_feet**2 +
    0.3 * body**2 after shifting the pose down so the lowest solid part touches y = 0."""
    keys = list(grid)
    best = None
    for vals in itertools.product(*(grid[k] for k in keys)):
        p = dict(zip(keys, vals))
        c = cost(p)
        if best is None or c < best[0]:
            best = (c, p)
    return best
