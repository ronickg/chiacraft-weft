#!/usr/bin/env python3
"""Export a box model (this skill's spec.json + texture.png) to Bedrock files, as Weft / chiacraft
(and Minecraft Bedrock) read them:

    <out>/models/<name>.geo.json                    (and <name>_pup.geo.json with --pup)
    <out>/animations/<name><suffix>.animation.json  one per pose file (see POSE_FILES)
    <out>/textures/entity/<texture>.png

    python to_bedrock.py spec.json texture.png <out> --name papillon --scale 0.17 \
        --texture chihuahua/papillon --pup 0.5 --pup-head 1.3

spec.json: {"texture": [w, h], "bones": [{name, parent, pivot}], "elements": [{name, bone, lo, hi,
quad?, rot?, origin?, faces: {face: [u0, v0, u1, v1]}}], "animations": {name: {length, frames:
[{t, rot: {bone: [x, y, z]}, root: [x, y, z]}]}}}, in Blockbench units, y up, facing +z, rotations
R = Rz Ry Rx with +X pitching the +z end down (Blockbench's own convention).

What it handles (each was a bug before it was handled):
- Bedrock models face -z: the model is turned 180 degrees about y. Positions become (-x, y, -z);
  face names swap north/south; rotation angles stay as they are (the engine flips x and z itself).
- Texture orientation per face, including "up" and "down" (turned, and "down" stored flipped).
- Engines that don't rotate single cubes (Weft): every rotated element gets its own child bone
  `<element>_tilt` carrying the rotation, so nothing changes visually.
- Quads (zero-thickness): the back face gets the same texture mirrored, so both sides show and the
  cut-out outline lines up; engines without double-sided faces draw each side from its own face.
- No per-creature scale in the engine: the scale is baked into every position and size (texel
  density is untouched, UVs are absolute). A puppy is the same at `--pup` with a bigger head.
- Parents before children; plain numbers only (no Molang); looping, linear keys.
"""

import argparse
import json
import os
import re
import shutil

import numpy as np

# Pose files the engine picks by kind: suffix -> {animation name in the file: name in the spec}.
# Weft plays `idle` (holding the pose), `walk` and `run` (by speed) by itself; others play when
# code asks for them.
POSE_FILES = {"": {"idle": "idle", "walk": "walk", "run": "run", "lick": "lick"},
              "_sit": {"idle": "sit", "walk": "walk"},
              "_lie": {"idle": "lie", "walk": "walk"}}
OUR_TO_FILE = {"east": "east", "west": "west", "up": "up", "down": "down", "south": "north", "north": "south"}
OPPOSITE = {"east": ("west", "u"), "west": ("east", "u"), "north": ("south", "u"), "south": ("north", "u"),
            "up": ("down", "v"), "down": ("up", "v")}


def r4(v):
    return [round(float(x), 4) for x in v]


def file_uv(face, u0, v0, w, h):
    """A face's rect (as painted, seen from outside, u right, v down; w, h may be negative) as
    the file stores it for the face it becomes."""
    if face in ("up",):  # turned 180 degrees with the model
        return {"uv": r4([u0 + w, v0 + h]), "uv_size": r4([-w, -h])}
    if face == "down":  # turned, and stored flipped in v as Blockbench writes it
        return {"uv": r4([u0 + w, v0]), "uv_size": r4([-w, h])}
    return {"uv": r4([u0, v0]), "uv_size": r4([w, h])}


def descendants(bones, root):
    kids = {b["name"]: [] for b in bones}
    for b in bones:
        if b.get("parent"):
            kids[b["parent"]].append(b["name"])
    out, todo = set(), [root]
    while todo:
        n = todo.pop()
        out.add(n)
        todo += kids.get(n, [])
    return out


def geometry(spec, ident, scale, head_scale=1.0, head="head"):
    bones = spec["bones"]
    piv = {b["name"]: np.array(b["pivot"], float) for b in bones}
    in_head = descendants(bones, head) if head in piv and head_scale != 1.0 else set()
    hp = piv.get(head, np.zeros(3))

    def at(v, bone):
        v = np.array(v, float)
        if bone in in_head:
            v = hp + (v - hp) * head_scale
        return np.array([-v[0], v[1], -v[2]]) * scale

    out = {b["name"]: {"name": b["name"], "pivot": r4(at(b["pivot"], b["name"])), "cubes": []} for b in bones}
    order = [b["name"] for b in bones]
    for b in bones:
        if b.get("parent"):
            out[b["name"]]["parent"] = b["parent"]
    for e in spec["elements"]:
        bone = e["bone"]
        if e.get("rot") and any(e["rot"]):  # its own bone carries the tilt
            tilt = f"{e['name']}_tilt"
            out[tilt] = {"name": tilt, "parent": bone, "pivot": r4(at(e["origin"], bone)),
                         "rotation": r4(e["rot"]), "cubes": []}
            order.append(tilt)
            target = tilt
        else:
            target = bone
        a, c = at(e["lo"], bone), at(e["hi"], bone)
        lo, hi = np.minimum(a, c), np.maximum(a, c)
        uv = {}
        for face, rect in e["faces"].items():
            if rect is None:
                continue
            u0, v0, u1, v1 = rect
            uv[OUR_TO_FILE[face]] = file_uv(OUR_TO_FILE[face], u0, v0, u1 - u0, v1 - v0)
            if e.get("quad"):  # the back of a flat panel: the same picture, mirrored to line up
                back, flip = OPPOSITE[face]
                if flip == "u":
                    uv[OUR_TO_FILE[back]] = file_uv(OUR_TO_FILE[back], u1, v0, u0 - u1, v1 - v0)
                else:
                    uv[OUR_TO_FILE[back]] = file_uv(OUR_TO_FILE[back], u0, v1, u1 - u0, v0 - v1)
        out[target]["cubes"].append({"origin": r4(lo), "size": r4(hi - lo), "uv": uv})
    tw, th = spec["texture"]
    return {"format_version": "1.12.0",
            "minecraft:geometry": [{"description": {"identifier": ident, "texture_width": tw, "texture_height": th},
                                    "bones": [out[n] for n in order]}]}


def animation(spec_anim, scale):
    """One animation: rotation keys per bone (all-zero bones left out), scale keys for the bones a
    frame sizes, the root's position."""
    frames = spec_anim["frames"]
    bones = {}
    names = sorted({b for f in frames for b in f["rot"]})
    for b in names:
        keys = {f"{f['t']:g}": r4(f["rot"][b]) for f in frames}
        if any(any(abs(x) > 1e-6 for x in v) for v in keys.values()):
            bones[b] = {"rotation": keys}
    for b in sorted({b for f in frames for b in f.get("scale", {})}):  # a part drawn in (a folded wing's skin)
        bones.setdefault(b, {})["scale"] = {f"{f['t']:g}": r4(f.get("scale", {}).get(b, [1, 1, 1])) for f in frames}
    root = {f"{f['t']:g}": r4(np.array([-f["root"][0], f["root"][1], -f["root"][2]]) * scale) for f in frames}
    if any(any(abs(x) > 1e-6 for x in v) for v in root.values()):
        bones.setdefault("root", {})["position"] = root
    return {"loop": spec_anim.get("loop", True), "animation_length": spec_anim["length"], "bones": bones}


# The engine's standing clips; every other name is an action (`World::action_play`, a held state or
# a one-off: a pounce, grazing, an alarm), written to go *over* what plays below on the bones it
# names (Weft's `override_previous_animation`), easing in and out. Added on top instead, a whole
# pose (head down to graze) stacked on the idle's own moves (a wolf's howl) and twisted.
STANDING = ("idle", "walk", "run")
ACTION_BLEND = (0.25, 0.3)


def as_action(anim):
    """An action clip: over what is below on its bones, easing in and out (`ACTION_BLEND`, or the
    animation's own `blend`)."""
    blend_in, blend_out = anim.pop("blend", ACTION_BLEND)
    return {**anim, "override_previous_animation": True, "blend_weight": 1, "blend_in": blend_in, "blend_out": blend_out}


# Weft (like Bedrock) adds a walk or run on top of the idle it plays all along, so in each pose file
# these are written as offsets from the file's idle (its first frame): full poses added to a held
# pose (a crouch, folded wings) would count it twice.
LAYERED = ("walk", "run")


def offset_from(spec_anim, base):
    """The animation as offsets from the pose `base` (a frame): rotations and the root's position
    less the base's, scales over the base's (kept where they differ from it)."""
    frames = []
    for f in spec_anim["frames"]:
        rot = {b: (np.array(f["rot"].get(b, [0, 0, 0])) - np.array(base["rot"].get(b, [0, 0, 0]))).tolist()
               for b in set(f["rot"]) | set(base["rot"])}
        g = {"t": f["t"], "rot": rot, "root": (np.array(f["root"]) - np.array(base["root"])).tolist()}
        fs, bs = f.get("scale", {}), base.get("scale", {})
        sc = {b: (np.array(fs.get(b, [1, 1, 1])) / np.array(bs.get(b, [1, 1, 1]))).tolist() for b in set(fs) | set(bs)}
        g["scale"] = {b: v for b, v in sc.items() if any(abs(x - 1) > 1e-6 for x in v)}
        frames.append(g)
    return {**spec_anim, "frames": frames}


def write(path, value):
    """Indented JSON with every list of numbers on one line (readable, diffs stay small)."""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    text = json.dumps(value, indent=1)
    text = re.sub(r"\[\s*(-?[\d.e+-]+(?:,\s*-?[\d.e+-]+)*)\s*\]", lambda m: "[" + ", ".join(x.strip() for x in m.group(1).split(",")) + "]", text)
    with open(path, "w") as f:
        f.write(text + "\n")


def export(spec, texture, out, name, scale, tex_name, pup=None, pup_head=1.3, pose_files=POSE_FILES):
    variants = [(name, scale, 1.0)] + ([(f"{name}_pup", scale * pup, pup_head)] if pup else [])
    for model, s, hs in variants:
        write(os.path.join(out, "models", f"{model}.geo.json"), geometry(spec, f"geometry.{model}", s, hs))
        for suffix, names in pose_files.items():
            idle = spec["animations"].get(names.get("idle"))
            anims = {f"animation.{model}{suffix}.{k}":
                     (lambda a: a if k in STANDING else as_action({**a, "blend": spec["animations"][v].get("blend", ACTION_BLEND)}))(
                         animation(offset_from(spec["animations"][v], idle["frames"][0]) if k in LAYERED and idle else spec["animations"][v], s))
                     for k, v in names.items() if v in spec["animations"]}
            write(os.path.join(out, "animations", f"{model}{suffix}.animation.json"),
                  {"format_version": "1.8.0", "animations": anims})
    dst = os.path.join(out, "textures", "entity", f"{tex_name}.png")
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    shutil.copyfile(texture, dst)
    return [v[0] for v in variants]


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("spec")
    ap.add_argument("texture")
    ap.add_argument("out")
    ap.add_argument("--name", required=True)
    ap.add_argument("--scale", type=float, required=True, help="model pixels per spec unit")
    ap.add_argument("--texture-name", dest="tex_name", required=True, help="path under textures/entity, no .png")
    ap.add_argument("--pup", type=float, help="a puppy model at this fraction of the scale")
    ap.add_argument("--pup-head", type=float, default=1.3)
    a = ap.parse_args()
    print(export(json.load(open(a.spec)), a.texture, a.out, a.name, a.scale, a.tex_name, a.pup, a.pup_head))
