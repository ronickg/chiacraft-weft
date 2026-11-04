"""Axe heads cut from whole axe icons by position (colour can't split a wooden head from a wooden
haft): the handle's line is fitted, the head is what stands out sideways from it near the top, and
the collar or lashing just below the head comes with it. Each head is moved so its base sits on
the iron axe's socket, so any head goes on any haft (the parts test fixes, 2026-11-04)."""
import numpy as np
from PIL import Image


def line(mask):
    """The handle's line through a mask's pixels: (centre, unit direction toward the top right)."""
    ys, xs = np.where(mask)
    p = np.stack([xs, ys], 1).astype(float)
    c = p.mean(0)
    u, s, vt = np.linalg.svd(p - c)
    d = vt[0]
    if d[0] - d[1] < 0:  # point from bottom-left to top-right (x up, y down)
        d = -d
    return c, d


def frame(a, c, d):
    """Each pixel's position along the line (t) and distance from it."""
    ys, xs = np.mgrid[0:a.shape[0], 0:a.shape[1]]
    px, py = xs - c[0], ys - c[1]
    t = px * d[0] + py * d[1]
    dist = np.abs(px * d[1] - py * d[0])
    return t, dist


def cut(a, wide=4.0):
    """(head mask incl. collar, base point on the line, handle line) for a whole axe icon. Tools
    lie on the canvas diagonal, handle bottom left: start from that line, refit it on the lower
    half (only handle there), and the head begins where pixels first stand out sideways."""
    m = a[..., 3] > 0
    c, d = np.array([15.5, 15.5]), np.array([1.0, -1.0]) / np.sqrt(2)
    t, dist = frame(a, c, d)
    handle = m & (dist <= 4) & (t < -2)
    c, d = line(handle)
    t, dist = frame(a, c, d)
    handle = m & (dist <= 3) & (t < t[m].min() + (t[m].max() - t[m].min()) * 0.55)
    c, d = line(handle)
    t, dist = frame(a, c, d)
    lo = t[m].min() + (t[m].max() - t[m].min()) * 0.3  # past the pommel and bands
    t_head = t[m & (dist > wide) & (t > lo)].min()
    head = m & ((t >= t_head) | ((t >= t_head - 3) & (dist <= 2.6)))  # head, and its collar or lashing below
    base = c + d * (t_head - 3)
    return head, base, (c, d)


def shift(arr, dy, dx):
    """Move an image or mask by whole pixels; what leaves the canvas is dropped, nothing wraps."""
    out = np.zeros_like(arr)
    h, w = arr.shape[:2]
    ys, yd = (slice(0, h - dy), slice(dy, h)) if dy >= 0 else (slice(-dy, h), slice(0, h + dy))
    xs, xd = (slice(0, w - dx), slice(dx, w)) if dx >= 0 else (slice(-dx, w), slice(0, w + dx))
    out[yd, xd] = arr[ys, xs]
    return out


def moved(a, mask, sh):
    dy, dx = sh
    return shift(a, dy, dx), shift(mask, dy, dx)


def on_socket(a, mask, base, socket):
    """Move a cut head so its base lands on the socket, nudged back inside the canvas if it would
    clip (the handle runs on under the head, so a pixel along it doesn't show)."""
    dx, dy = int(round(socket[0] - base[0])), int(round(socket[1] - base[1]))
    ys, xs = np.where(mask)
    dx = min(max(dx, -xs.min()), 31 - xs.max())
    dy = min(max(dy, -ys.min()), 31 - ys.max())
    return moved(a, mask, (dy, dx)), (dy, dx)
