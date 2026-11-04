"""The bow's parts for the crafting hook (kind "bow"): its ramps, and once the two bows are drawn,
the maps cut from them. Nothing moves; the bow lies corner to corner.

  parts/bow/bowstring.png        the string as a map, region 0
  parts/bow/self_stave.png       the long stave as a map, region 0 stave, 1 grip
  parts/bow/short_stave.png      the short stave, the same regions
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


if __name__ == "__main__":
    r = ramps()
    print({k: np.round(v).astype(int).tolist() for k, v in r.items() if k == "leather"})
