#!/usr/bin/env python3
"""Way D, second half: build out/d_blocky/spec.json + texture.png as a Hytale Character
project in a running Blockbench (with the Hytale Models plugin) and export what it writes:

    out/d_blocky/voxel_dog.blockymodel    the model (Blockbench's own Hytale exporter)
    out/d_blocky/anims/<name>.blockyanim  one per animation
    out/d_blocky/voxel_dog.glb            the same with animations, for the viewer
    out/d_blocky/voxel_dog.bbmodel        the Blockbench project, to edit by hand

Start Blockbench with a debugging port first (see NOTES.md), then:

    .venv/bin/python tools/bb_build.py
"""

import base64
import json
import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
import bb  # noqa: E402

ROOT = os.path.join(os.path.dirname(__file__), "..")
OUT = os.environ.get("BB_OUT", os.path.join(ROOT, "out", "d_blocky"))

JS = r"""
const SPEC = __SPEC__;
const TEX = "__TEX__";
for (const p of [...ModelProject.all]) await p.close(true);
newProject(Formats.hytale_character);
await new Promise(r => setTimeout(r, 200));
if (Dialog.open) Dialog.open.cancel();  // the new-project settings dialog
Project.name = SPEC.name;
Project.texture_width = SPEC.texture[0];
Project.texture_height = SPEC.texture[1];

const tex = new Texture({ name: "voxel_dog" }).fromDataURL("data:image/png;base64," + TEX).add(false);
await new Promise(r => setTimeout(r, 300));
tex.uv_width = SPEC.texture[0];
tex.uv_height = SPEC.texture[1];

const groups = {};
for (const b of SPEC.bones) {
  const g = new Group({ name: b.name, origin: b.pivot }).init();
  if (b.parent) g.addTo(groups[b.parent]);
  groups[b.name] = g;
}
for (const e of SPEC.elements) {
  const faces = {};
  for (const f of ["north", "south", "east", "west", "up", "down"]) {
    const uv = e.faces[f];
    faces[f] = uv ? { uv, texture: tex.uuid } : { uv: [0, 0, 0, 0], texture: null };
  }
  const c = new Cube({
    name: e.name, from: e.lo, to: e.hi, box_uv: false, autouv: 0,
    origin: e.origin || [(e.lo[0] + e.hi[0]) / 2, (e.lo[1] + e.hi[1]) / 2, (e.lo[2] + e.hi[2]) / 2],
    rotation: e.rot || [0, 0, 0],
    double_sided: !!e.quad && !e.name.startsWith("ear_"),
    faces,
  }).init();
  c.addTo(groups[e.bone]);
}
Project.texture_width = SPEC.texture[0];
Project.texture_height = SPEC.texture[1];
Canvas.updateAll();

// Animations: rotation keyframes on every bone, position on the root.
for (const [name, a] of Object.entries(SPEC.animations)) {
  const anim = new Animation({ name, length: a.length, loop: "loop", snapping: 60 }).add();
  for (const fr of a.frames) {
    for (const [bone, r] of Object.entries(fr.rot)) {
      if (!groups[bone]) continue;
      anim.getBoneAnimator(groups[bone]).addKeyframe({ channel: "rotation", time: fr.t, data_points: [{ x: r[0], y: r[1], z: r[2] }] });
    }
    anim.getBoneAnimator(groups.root).addKeyframe({ channel: "position", time: fr.t, data_points: [{ x: fr.root[0], y: fr.root[1], z: fr.root[2] }] });
  }
}

// Capture what the exporters would save instead of opening save dialogs.
const out = { anims: {} };
const realExport = Blockbench.export;
Blockbench.export = (opts, cb) => { out.anims[opts.name] = opts.content; };
for (const anim of Animation.all) { anim.path = ""; anim.save(); }
Blockbench.export = realExport;
out.model = Codecs.blockymodel.compile();
out.project = Codecs.project.compile();
const gl = await Codecs.gltf.compile({ animations: Animation.all, encoding: "binary" });
const b64 = buf => { const u = new Uint8Array(buf); let s = ""; for (let i = 0; i < u.length; i += 0x8000) s += String.fromCharCode.apply(null, u.subarray(i, i + 0x8000)); return btoa(s); };
out.glb = gl instanceof ArrayBuffer ? b64(gl) : null;
out.gltf_json = gl instanceof ArrayBuffer ? null : (typeof gl === "string" ? gl : JSON.stringify(gl));
out.counts = { cubes: Cube.all.length, groups: Group.all.length, animations: Animation.all.length };
return out;
"""


def main():
    spec = open(os.path.join(OUT, "spec.json")).read()
    tex = base64.b64encode(open(os.path.join(OUT, "texture.png"), "rb").read()).decode()
    res = bb.run(JS.replace("__SPEC__", spec).replace("__TEX__", tex))
    os.makedirs(os.path.join(OUT, "anims"), exist_ok=True)
    model = res["model"] if isinstance(res["model"], str) else json.dumps(res["model"], indent=2)
    open(os.path.join(OUT, "voxel_dog.blockymodel"), "w").write(model)
    for name, content in res["anims"].items():
        open(os.path.join(OUT, "anims", f"{name}.blockyanim"), "w").write(content if isinstance(content, str) else json.dumps(content, indent=2))
    proj = res["project"] if isinstance(res["project"], str) else json.dumps(res["project"])
    open(os.path.join(OUT, "voxel_dog.bbmodel"), "w").write(proj)
    if res.get("glb"):
        open(os.path.join(OUT, "voxel_dog.glb"), "wb").write(base64.b64decode(res["glb"]))
    elif res.get("gltf_json"):
        open(os.path.join(OUT, "voxel_dog.gltf"), "w").write(res["gltf_json"])
    print(res["counts"], "anims:", sorted(res["anims"]), "glb" if res.get("glb") else "gltf json" if res.get("gltf_json") else "no gltf")


if __name__ == "__main__":
    main()
