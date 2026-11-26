#!/usr/bin/env python3
"""Pack texture variants (same UV layout, e.g. different eyes, coats, markings) for the viewer's
switcher. Each PNG is padded to the size of the texture embedded in the exported GLB (Blockbench
pads its glTF texture, e.g. 512x96 -> 512x128; a variant at the unpadded size lands at the wrong
UVs and scrambles the model), then written as data URLs to one JSON file.

    python variants.py model.glb out/variants.json name1=tex1.png name2=tex2.png ...
"""

import base64
import io
import json
import struct
import sys

import numpy as np
from PIL import Image


def glb_texture_size(path):
    b = open(path, "rb").read()
    n = struct.unpack("<I", b[12:16])[0]
    j = json.loads(b[20:20 + n])
    start = 20 + n + 8
    bv = j["bufferViews"][j["images"][0]["bufferView"]]
    off = start + bv.get("byteOffset", 0)
    return Image.open(io.BytesIO(b[off:off + bv["byteLength"]])).size


def pack(glb, out, named):
    w, h = glb_texture_size(glb)
    data = {}
    for name, path in named:
        t = np.asarray(Image.open(path).convert("RGBA"))
        pad = np.zeros((h, w, 4), np.uint8)
        pad[:t.shape[0], :t.shape[1]] = t  # top-left aligned, as Blockbench pads
        buf = io.BytesIO()
        Image.fromarray(pad).save(buf, "PNG")
        data[name] = "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode()
    json.dump(data, open(out, "w"))
    return (w, h), list(data)


if __name__ == "__main__":
    size, names = pack(sys.argv[1], sys.argv[2], [a.split("=", 1) for a in sys.argv[3:]])
    print("padded to", size, names)
