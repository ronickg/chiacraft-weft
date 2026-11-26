"""The scarecrow's own textures (art stand-ins, 11-26), painted in code with the box models' painter
(blocky-model's `paint.py`: one light from the top left, hue-shifted ramps) at 32 texels a block,
since its faces are too small (a 6 px head is 12 texels) for the image model's 32 px canvases.
Its post and arms stay oak bark (`log_oak`). Three 32 x 32 textures, each a small atlas of the
part's faces (rectangles in `UV`, 0..16 as `model_box_faces` takes them):

- `scarecrow_head`: a stuffed burlap sack, two stitched eyes and a stitched mouth on its front,
  straw poking out of its top, tied off at the neck with twine.
- `scarecrow_body`: a faded red homespun shirt with a blue-grey patch, bone buttons, a twine belt,
  straw hanging from under its hem and out of its collar.
- `scarecrow_lantern`: an iron lantern with four horn panes lit by a candle;
  `scarecrow_lantern_e.png` is its glow mask (red x alpha: how much a pixel glows).

Usage: python3 scarecrow.py OUT_DIR [--preview out.png]"""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.expanduser("~/.claude/skills/blocky-model/scripts"))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import paint  # noqa: E402

# materials: tones(base, shadow hue); warm shadows for cloth and straw, purple for iron
MATS = {
    "burlap": paint.tones((176, 148, 100), 0.05),
    "straw": paint.tones((214, 184, 108), 0.06),
    "stitch": paint.tones((58, 44, 36), 0.02),
    "twine": paint.tones((150, 116, 66), 0.05),
    "shirt": paint.tones((150, 76, 56), 0.98),
    "patch": paint.tones((104, 116, 128), 0.70),
    "button": paint.tones((220, 206, 176), 0.08),
    "iron": paint.tones((84, 80, 78), 0.74),
    "pane": paint.tones((246, 186, 92), 0.06),
}
IDS = {n: i for i, n in enumerate(MATS)}
TONES = list(MATS.values())

# Each part's faces in its texture: (u, v, w, h) in texels
HEAD = {"north": (0, 0, 12, 12), "side": (12, 0, 12, 12), "up": (0, 12, 12, 12), "down": (12, 12, 12, 12)}
BODY = {"north": (0, 0, 12, 22), "south": (12, 0, 12, 22), "side": (24, 0, 8, 22), "up": (0, 22, 12, 8), "down": (12, 22, 12, 8)}
LANTERN = {"side": (0, 0, 6, 10), "up": (6, 0, 6, 6), "down": (12, 0, 6, 6)}


def uv(r):
    """A texel rectangle as model_box_faces' (u0, v0, u1, v1) in 0..16."""
    u, v, w, h = r
    return [u / 2, v / 2, (u + w) / 2, (v + h) / 2]


def face_paint(areas, face, seed=0):
    a = np.array([[IDS[c] for c in row] for row in areas]) if isinstance(areas[0], list) else areas
    return paint.paint_face_smooth(a, TONES, face, coat=(), seed=seed, tuft=False)


def grid(w, h, fill):
    return np.full((h, w), IDS[fill])


def straw_edge(a, rows, rng, side="top"):
    """Straw poking out along an edge: ragged, 1 to `rows` texels deep."""
    h, w = a.shape
    for x in range(w):
        d = rng.integers(1, rows + 1)
        if side == "top":
            a[:d, x] = IDS["straw"]
        else:
            a[h - d:, x] = IDS["straw"]


def head():
    rng = np.random.default_rng(3)
    out = np.zeros((32, 32, 4), np.uint8)
    for name, (u, v, w, h) in HEAD.items():
        a = grid(w, h, "burlap")
        if name in ("north", "side"):
            straw_edge(a, 2, rng)
            a[h - 2, :] = IDS["twine"]  # tied off at the neck
            face = "north" if name == "north" else "west"
        else:
            face = name
        if name == "up":
            a[:] = IDS["straw"]
            a[rng.random((h, w)) < 0.12] = IDS["burlap"]
        if name == "down":
            a[:] = IDS["burlap"]
            a[[0, -1], :] = IDS["twine"]
        img = face_paint(a, face, seed=len(name))
        if name == "north":
            dark = paint.tones((40, 30, 26), 0.02)[3]
            for (x, y) in [(3, 4), (4, 4), (3, 5), (4, 5), (7, 4), (8, 4), (7, 5), (8, 5)]:  # stitched eyes
                img[y, x] = dark
            img[4, 4] = img[4, 8] = paint.tones((40, 30, 26), 0.02)[5]  # a glint of thread
            for x in range(3, 9):  # the mouth, a running stitch
                img[8, x] = dark
            for x in (3, 5, 7):
                img[7, x] = img[9, x + 1] = dark
        if name == "side":
            for y in range(3, h - 2, 2):  # the sack's side seam
                img[y, w // 2] = MATS["burlap"][1]
        out[v:v + h, u:u + w, :3] = np.clip(img, 0, 255)
        out[v:v + h, u:u + w, 3] = 255
    return out


def body():
    rng = np.random.default_rng(5)
    out = np.zeros((32, 32, 4), np.uint8)
    for name, (u, v, w, h) in BODY.items():
        a = grid(w, h, "shirt")
        face = {"north": "north", "south": "south", "side": "west"}.get(name, name)
        if name in ("north", "south", "side"):
            straw_edge(a, 2, rng)                   # out of the collar
            a[15:17, :] = IDS["twine"]              # the belt
            straw_edge(a[17:], 4, rng, "bottom")    # from under the hem
            a[17:, :][a[17:, :] != IDS["straw"]] = IDS["shirt"]
            if name == "north":
                a[9:14, 1:5] = IDS["patch"]         # a patch, lower left
                for y in (4, 8, 12):
                    a[y, 6] = IDS["button"]
        if name in ("up", "down"):
            a[:] = IDS["straw"]
            a[rng.random((h, w)) < 0.15] = IDS["shirt"]
        img = face_paint(a, face, seed=len(name) + 7)
        if name == "north":  # the patch's stitches round its edge
            st = MATS["stitch"][4]
            for x in range(1, 5, 2):
                img[9, x] = img[13, x + 1] = st
            for y in range(9, 14, 2):
                img[y, 1] = img[y + 1 if y < 13 else y, 4] = st
        out[v:v + h, u:u + w, :3] = np.clip(img, 0, 255)
        out[v:v + h, u:u + w, 3] = 255
    return out


def lantern():
    out = np.zeros((32, 32, 4), np.uint8)
    glow = np.zeros((32, 32, 4), np.uint8)
    for name, (u, v, w, h) in LANTERN.items():
        a = grid(w, h, "iron")
        if name == "side":
            a[2:9, 1:5] = IDS["pane"]
        if name == "down":
            a[2:4, 2:4] = IDS["pane"]  # the candle's light through the vent
        img = face_paint(a, "west" if name == "side" else name, seed=11)
        if name == "side":
            hot = paint.tones((255, 236, 170), 0.1)[4]
            img[4:7, 2:4] = hot            # the flame behind the horn
            img[1, :] = MATS["iron"][5]    # the cap's lit rim
            img[5, 1:5] = np.where(np.arange(4)[:, None] % 3 == 0, img[5, 1:5], img[5, 1:5])
        out[v:v + h, u:u + w, :3] = np.clip(img, 0, 255)
        out[v:v + h, u:u + w, 3] = 255
        g = (a == IDS["pane"]).astype(float) * 0.75
        if name == "side":
            g[4:7, 2:4] = 1.0
        glow[v:v + h, u:u + w, 0] = (g * 255).astype(np.uint8)
        glow[v:v + h, u:u + w, 3] = 255
    return out, glow


def preview(path, tex, logs):
    import chest_split as cs

    def cut(img, r):
        u, v, w, h = r
        return img[v:v + h, u:u + w]
    hd, bd, ln = tex["scarecrow_head"], tex["scarecrow_body"], tex["scarecrow_lantern"]
    post = logs[:, 0:4]
    f = []
    f += cs.box_faces((7, 0, 7), (9, 30, 9), {k: post for k in ("north", "west", "east", "south")} | {"up": logs[0:4, 0:4]})
    f += cs.box_faces((1, 21, 7), (15, 23, 9), {"north": np.rot90(post), "south": np.rot90(post), "up": np.rot90(post),
                                                 "west": logs[0:4, 0:4], "east": logs[0:4, 0:4]})
    f += cs.box_faces((5, 13, 6), (11, 24, 10), {"north": cut(bd, BODY["north"]), "south": cut(bd, BODY["south"]),
                                                  "west": cut(bd, BODY["side"]), "east": cut(bd, BODY["side"]), "up": cut(bd, BODY["up"])})
    f += cs.box_faces((5, 24, 5), (11, 30, 11), {"north": cut(hd, HEAD["north"]), "west": cut(hd, HEAD["side"]),
                                                  "east": cut(hd, HEAD["side"]), "south": cut(hd, HEAD["side"]), "up": cut(hd, HEAD["up"])})
    f += cs.box_faces((12, 15, 6.5), (15, 20, 9.5), {k: cut(ln, LANTERN["side"]) for k in ("north", "west", "east", "south")} |
                      {"up": cut(ln, LANTERN["up"])})
    img = cs.draw(f, scale=11, size=(420, 520))
    img.save(path)


def main():
    out = sys.argv[1]
    os.makedirs(out, exist_ok=True)
    lt, glow = lantern()
    tex = {"scarecrow_head": head(), "scarecrow_body": body(), "scarecrow_lantern": lt}
    for n, a in tex.items():
        Image.fromarray(a).save(os.path.join(out, n + ".png"))
    Image.fromarray(glow).save(os.path.join(out, "scarecrow_lantern_e.png"))
    if "--preview" in sys.argv:
        repo = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
        logs = np.asarray(Image.open(os.path.join(repo, "mods/base/assets/textures/block/log_oak.png")).convert("RGBA"))
        preview(sys.argv[sys.argv.index("--preview") + 1], tex, logs)
    print("UV head", {k: uv(r) for k, r in HEAD.items()}, "body", {k: uv(r) for k, r in BODY.items()},
          "lantern", {k: uv(r) for k, r in LANTERN.items()})


if __name__ == "__main__":
    main()
