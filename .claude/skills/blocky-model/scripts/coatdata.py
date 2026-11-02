#!/usr/bin/env python3
"""Pack a creature's coat layers (`layers.py`) and its gene file for the viewer's coat panel:

    python coatdata.py <out>/layers <genes.json> <viewer>/models/<animal>_coat.json [breed]

Writes {"species", "breed", "bounds", "layers": {info, pos, props, props2: data URLs}, "genes",
"core"} (a breed, for species that have them: random animals come from its frequencies; the core
traits every species shares, `core.json` next to the gene file, if it is there).
The viewer paints coats with `coats.js` on these layers (Random, breeding, paint)."""

import base64
import json
import os
import sys


def main(layers_dir, genes_path, out, breed=None):
    meta = json.load(open(os.path.join(layers_dir, "layers.json")))
    genes = json.load(open(genes_path))
    data = {"species": genes["species"], "breed": breed, "bounds": meta["bounds"], "genes": genes, "layers": {}}
    core = os.path.join(os.path.dirname(genes_path), "core.json")
    if os.path.exists(core):
        data["core"] = json.load(open(core))
    for n in ("info", "pos", "props", "props2"):
        b = open(os.path.join(layers_dir, f"{n}.png"), "rb").read()
        data["layers"][n] = "data:image/png;base64," + base64.b64encode(b).decode()
    json.dump(data, open(out, "w"))
    print("wrote", out)


if __name__ == "__main__":
    main(*sys.argv[1:5])
