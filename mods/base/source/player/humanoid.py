"""Weft's humanoid (client/models.rs parts; model.rs `pose`: no parents, euler ZYX, x and z negated) drawn in a pose."""
import json, sys
import numpy as np
from PIL import Image, ImageDraw

PARTS = {  # name: pivot, box origin (relative to pivot), size, colour
    "head": ((0, 24, 0), (-4, 0, -4), (8, 8, 8), (226, 190, 160)),
    "body": ((0, 24, 0), (-4, -12, -2), (8, 12, 4), (70, 110, 170)),
    "rightArm": ((-6, 22, 0), (-2, -10, -2), (4, 12, 4), (200, 160, 130)),
    "leftArm": ((6, 22, 0), (-2, -10, -2), (4, 12, 4), (200, 160, 130)),
    "rightLeg": ((-2, 12, 0), (-2, -12, -2), (4, 12, 4), (60, 60, 120)),
    "leftLeg": ((2, 12, 0), (-2, -12, -2), (4, 12, 4), (60, 60, 120)),
}

def rx(a):
    a = np.radians(a); c, s = np.cos(a), np.sin(a)
    return np.array([[1, 0, 0], [0, c, -s], [0, s, c]])
def ry(a):
    a = np.radians(a); c, s = np.cos(a), np.sin(a)
    return np.array([[c, 0, s], [0, 1, 0], [-s, 0, c]])
def rz(a):
    a = np.radians(a); c, s = np.cos(a), np.sin(a)
    return np.array([[c, -s, 0], [s, c, 0], [0, 0, 1]])
def euler(r):  # model.rs angles(): x and z negated; glam from_euler(ZYX, z, y, x) = Rz * Ry * Rx
    return rz(-r[2]) @ ry(r[1]) @ rx(-r[0])

def boxes(pose):
    root_p = np.array(pose.get("root_pos", [0, 0, 0]), float)
    root_r = euler(pose.get("root_rot", [0, 0, 0]))
    out = []
    for n, (piv, o, s, col) in PARTS.items():
        piv = np.array(piv, float); o = np.array(o, float); s = np.array(s, float)
        R = euler(pose.get(n, [0, 0, 0]))
        off = np.array(pose.get(n + "_pos", [0, 0, 0]), float)
        corners = np.array([[x, y, z] for x in (0, 1) for y in (0, 1) for z in (0, 1)]) * s + o  # relative to pivot
        w = (R @ corners.T).T + piv + off
        w = (root_r @ w.T).T + root_p
        out.append((n, w, col))
    return out

FACES = [(0, 1, 3, 2), (4, 5, 7, 6), (0, 1, 5, 4), (2, 3, 7, 6), (0, 2, 6, 4), (1, 3, 7, 5)]

def draw(pose, yaw, pitch, size=360, ppu=7):
    V = rx(pitch) @ ry(yaw)
    img = Image.new("RGB", (size, size), (150, 170, 140)); d = ImageDraw.Draw(img)
    polys = []
    light = np.array([0.3, 0.8, 0.5]); light /= np.linalg.norm(light)
    for n, w, col in boxes(pose):
        v = (V @ w.T).T
        c = w.mean(0)
        for f in FACES:
            q = v[list(f)]; qw = w[list(f)]
            nrm = np.cross(qw[1] - qw[0], qw[2] - qw[0]); nrm /= np.linalg.norm(nrm) + 1e-9
            if np.dot(nrm, qw.mean(0) - c) < 0: nrm = -nrm
            vn = V @ nrm
            if vn[2] > 0: continue  # facing away (camera looks along -z... towards +z? choose)
            shade = 0.55 + 0.45 * max(0, np.dot(nrm, light))
            polys.append((q[:, 2].mean(), [(size / 2 + p[0] * ppu, size * 0.8 - p[1] * ppu) for p in q], tuple(int(x * shade) for x in col)))
    for _, pts, col in sorted(polys, key=lambda t: t[0]):
        d.polygon(pts, fill=col, outline=(30, 30, 30))
    d.line([(0, size * 0.8), (size, size * 0.8)], fill=(90, 70, 40))
    return img

