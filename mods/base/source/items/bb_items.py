"""The item models in Blockbench (the blocky-model skill's way to build and look at a model): each
`<name>_spec.json` from `tools.py out/` built there as a Generic Model at the game's size (16 units a
block), with its texture, saved as `models/<name>.bbmodel` beside this script (open it in Blockbench
to look closer or edit), and shot from a few angles with the skill's `bbshot.py`, the icon first.

Blockbench must be running with `--remote-debugging-port=9333`.

    ~/.venvs/blocky/bin/python mods/base/source/items/bb_items.py out/ shots.png [names...] [--angles=front_flat,3/4,side] [--cols=5]
"""
import base64
import io
import json
import os
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
SKILL = os.path.join(HERE, "..", "..", "..", "..", ".claude", "skills", "blocky-model", "scripts")
sys.path.insert(0, SKILL)
from bb import run  # noqa: E402
from bbshot import ANGLES, BOUNDS, MORE, shot  # noqa: E402

K = 16 / 64  # Blockbench units (16 a block) per texel

BUILD = """
newProject(Formats.free);
if (Dialog.open) Dialog.open.cancel();
const spec = %s;
Project.name = spec.name;
Project.texture_width = Project.texture_height = spec.size;
const tex = new Texture({name: spec.name}).fromDataURL(spec.texture).add(false);
for (const e of spec.elements) {
  const faces = {};
  for (const [f, uv] of Object.entries(e.faces)) faces[f] = {uv, texture: tex};
  new Cube({name: e.name, from: e.from, to: e.to, origin: e.origin, rotation: e.rotation, box_uv: false, faces}).init();
}
Canvas.updateAll();
return Codecs.project.compile();
"""


def build(folder, name):
    spec = json.load(open(os.path.join(folder, name + "_spec.json")))
    png = open(os.path.join(folder, name + ".png"), "rb").read()
    els = []
    for e in spec["elements"]:
        lo, hi = [v * K for v in e["lo"]], [v * K for v in e["hi"]]
        o = [v * K for v in e["origin"]] if e["origin"] else [(a + b) / 2 for a, b in zip(lo, hi)]
        els.append({"name": e["name"], "from": lo, "to": hi, "origin": o, "rotation": list(e["rot"] or (0, 0, 0)),
                    "faces": e["faces"]})
    data = {"name": name, "size": spec["size"], "elements": els,
            "texture": "data:image/png;base64," + base64.b64encode(png).decode()}
    model = run(BUILD % json.dumps(data))
    os.makedirs(os.path.join(HERE, "models"), exist_ok=True)
    open(os.path.join(HERE, "models", name + ".bbmodel"), "w").write(model)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    angles = ["front_flat", "3/4", "side"]
    for a in sys.argv[1:]:
        if a.startswith("--angles="):
            angles = a.split("=", 1)[1].split(",")
    folder, out, names = args[0], args[1], args[2:]
    if not names:
        names = sorted(f[:-10] for f in os.listdir(folder) if f.endswith("_spec.json"))
    dirs = {**ANGLES, **MORE}
    rows = []
    for name in names:
        build(folder, name)
        bounds = run(BOUNDS)
        ims = [shot(dirs[a], bounds) for a in angles]
        icon = Image.open(os.path.join(folder, name + "_icon.png")).convert("RGBA")
        h = ims[0].height
        icon = icon.resize((h * 3 // 4, h * 3 // 4), Image.NEAREST)
        row = Image.new("RGB", (h + sum(i.width for i in ims), h), (52, 62, 72))
        row.paste(icon, (h // 8, h // 8), icon)
        x = h
        for im in ims:
            row.paste(im, (x, 0))
            x += im.width
        rows.append(row)
        run("Project.close(true); return true")
    cols = next((int(a.split("=")[1]) for a in sys.argv[1:] if a.startswith("--cols=")), 1)
    W, H = max(r.width for r in rows), rows[0].height
    sheet = Image.new("RGB", (W * cols, H * ((len(rows) + cols - 1) // cols)))
    for k, r in enumerate(rows):
        sheet.paste(r, ((k % cols) * W, (k // cols) * H))
    W = sheet.width
    scale = min(1.0, 1600 / W)
    sheet.resize((int(W * scale), int(sheet.height * scale))).save(out)


if __name__ == "__main__":
    main()
