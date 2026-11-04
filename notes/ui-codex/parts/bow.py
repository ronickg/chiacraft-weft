"""The bow's parts for the crafting hook (kind "bow"): its ramps, and once the two bows are drawn,
the maps cut from them. Nothing moves; the bow lies corner to corner.

  parts/bow/bowstring.png        the string as a map, region 0: a straight 1 px line drawn by code
                                 between the self stave's tips (a drawn 1 px pale string doesn't survive the cut)
  parts/bow/self_stave.png       the long stave as a map, region 0 stave, 1 grip
  parts/bow/short_stave.png      the short stave, the same regions (bow_short_stave_3: the drawing _2 turned 2
                                 degrees so its tips land on the self stave's, where the one string ends)
  ramps/<id>.png                 8 x 1, dark to light: yew, elm, osage_orange, sinew, rawhide, leather

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/bow.py ramps   # the ramps
    ~/.venvs/blocky/bin/python notes/ui-codex/parts/bow.py         # ramps, maps and the sheet
"""
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import parts  # noqa: E402

OUT = os.path.join(HERE, "deliver", "textures")

# Drawn by hand with the oak ramp's range (0-1 outline, 2-6 fill, 7 highlight): the plank ramps are
# too flat for a thin stave. Woods from their real colour: yew orange-red, elm a greyish mid brown,
# osage orange a bright golden yellow. Sinew amber; rawhide paler and yellower.
RAMPS = {
    "yew": [[70, 24, 10], [96, 36, 16], [138, 58, 28], [168, 78, 40], [192, 98, 52], [212, 120, 68], [230, 148, 96], [242, 176, 128]],
    "elm": [[64, 42, 26], [86, 58, 36], [118, 84, 54], [142, 104, 68], [162, 122, 82], [180, 140, 98], [200, 162, 120], [218, 186, 146]],
    "osage_orange": [[88, 52, 8], [118, 72, 12], [164, 106, 20], [196, 134, 30], [218, 158, 40], [234, 180, 58], [246, 204, 96], [252, 224, 142]],
    "sinew": [[70, 44, 22], [96, 62, 32], [146, 98, 56], [176, 124, 74], [198, 146, 92], [216, 168, 112], [232, 194, 144], [244, 218, 180]],
    "rawhide": [[90, 74, 44], [120, 100, 62], [178, 156, 110], [202, 182, 132], [218, 200, 150], [230, 214, 168], [242, 230, 192], [250, 242, 214]],
}


def save(arr, rel):
    path = os.path.join(OUT, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    Image.fromarray(np.asarray(arr).clip(0, 255).astype(np.uint8)).save(path)
    parts.art.archive(path, rel.replace("/", "__")[:-4], "parts")
    return path


def ramps():
    r = {k: np.array(v, float) for k, v in RAMPS.items()}
    r["leather"] = parts.ramp_from(parts.icon_pixels("leather"))  # our leather icon's own colours
    for k, v in r.items():
        save(v.reshape(1, 8, 3), f"ramps/{k}.png")
    return r


def as_map(a, mask, region):
    """R = the 8-step value x 36 (steps taken within this part), G = region, A = on."""
    st = parts.steps(a, mask)
    out = np.zeros((32, 32, 4))
    out[..., 0] = np.where(mask, np.clip(st, 0, 7) * 36, 0)
    out[..., 1] = np.where(mask, region, 0)
    out[..., 3] = np.where(mask, 255, 0)
    return out


def tips(m):
    """The stave's two tips: the pixels furthest toward the bottom-left and top-right corners."""
    ys, xs = np.nonzero(m)
    along = xs - ys
    return np.array([xs[np.argmin(along)], ys[np.argmin(along)]]), np.array([xs[np.argmax(along)], ys[np.argmax(along)]])


def line(p0, p1):
    """The 1 px string from tip to tip (every pixel the straight line passes, no gaps), without the
    tips themselves, which belong to the stave."""
    m = np.zeros((32, 32), bool)
    n = int(np.abs(p1 - p0).max()) * 2
    for k in range(1, n):
        x, y = np.round(p0 + (p1 - p0) * k / n).astype(int)
        m[y, x] = True
    m[p0[1], p0[0]] = m[p1[1], p1[0]] = False
    return m


def load(name):
    return np.array(Image.open(os.path.join(HERE, "out", name + "_px.png")).convert("RGBA")).astype(float)


def stave_map(a, p0, p1):
    """The stave as a map: region 1 the grip (within 3 px of the chord's middle, measured along it), 0 the rest."""
    m = a[..., 3] > 0
    ys, xs = np.mgrid[0:32, 0:32]
    d = (p1 - p0) / np.linalg.norm(p1 - p0)
    t = (xs - p0[0]) * d[0] + (ys - p0[1]) * d[1]
    grip = m & (np.abs(t - np.linalg.norm(p1 - p0) / 2) <= 3)
    return as_map(a, m, np.where(grip, 1, 0))


def maps(staves, ramps_):
    """staves = {"self_stave": out name, "short_stave": out name}; the string runs between the self stave's tips."""
    a = load(staves["self_stave"])
    p0, p1 = tips(a[..., 3] > 0)
    for part, name in staves.items():
        s_ = load(name)
        q0, q1 = tips(s_[..., 3] > 0)
        print(part, "tips", q0.tolist(), q1.tolist(), "(self:", p0.tolist(), p1.tolist(), ")")
        save(stave_map(s_, p0, p1), f"parts/bow/{part}.png")
    string = line(p0, p1)
    sm = np.zeros((32, 32, 4))
    sm[..., 0] = np.where(string, 5 * 36, 0)  # one even shade: a 1 px cord has no room for more
    sm[..., 3] = np.where(string, 255, 0)
    save(sm, "parts/bow/bowstring.png")
    # the kind icon: the self bow strung with a pale rawhide string
    icon = a.copy()
    icon[string, :3] = ramps_["rawhide"][6]
    icon[string, 3] = 255
    p = os.path.join(HERE, "deliver", "fallback", "item", "bow.png")
    Image.fromarray(icon.astype(np.uint8)).save(p)
    Image.fromarray(icon.astype(np.uint8)).save(os.path.join(HERE, "out", "bow_icon_px.png"))
    parts.art.archive(p, "fallback_bow", "fallback")


if __name__ == "__main__":
    r = ramps()
    if sys.argv[1:] != ["ramps"]:
        maps({"self_stave": "bow_self_stave", "short_stave": sys.argv[1] if sys.argv[1:] else "bow_short_stave_3"}, r)
