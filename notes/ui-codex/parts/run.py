"""The full parts run (Ronald, 11-04: "Yes, do the rest"): every tool kind built from parts in every
material, with the textures he settled (row 2 of the axe and sword sheets): the working part drawn
per material on the kind's shared socket, the joint as a 2-3 px band, one outline per tool, soft seams.

Per kind it writes a sheet (the composed tool per material beside the approved icon) and the
crafting session's files (notes/CRAFTING.md, "The art format"):
  deliver/<kind>/<material>_<work>.png          painted override layer (head, blade), 32x32 at the socket
  deliver/<kind>/<material>_<work>_regions.png  G = region (0 the working part, 1 the joint band), A
  deliver/<kind>/handle_wrapped.png, handle_plain.png   painted handle layers
  deliver/<kind>/handle_map.png                 R = step x 36 (0/1 outline, 2-6 fill, 7 highlight), G = region, A
  deliver/<kind>/*.json                         {"kind","layer","regions","steps"}
  deliver/<kind>/socket.json                    the socket and the layers' boxes
  deliver/strips/<material>.png                 8x1 ramps for handle materials (oak, birch, antler, leather...)

    ~/.venvs/blocky/bin/python notes/ui-codex/parts/run.py pickaxe shovel hoe axe sword
"""
import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import parts  # noqa: E402
import toolfix as tf  # noqa: E402

DELIVER = os.path.join(HERE, "deliver")
MATERIALS = ["wood", "stone", "flint", "copper", "bronze", "iron", "steel", "gold", "diamond"]
CRUDE = {"wood", "stone", "flint"}
WORK = {"pickaxe": "head", "shovel": "head", "hoe": "head", "axe": "head", "sword": "blade"}
JOINT = {"pickaxe": "fitting", "shovel": "fitting", "hoe": "fitting", "axe": "fitting", "sword": "guard"}
HANDLE = {"pickaxe": "haft", "shovel": "haft", "hoe": "haft", "axe": "haft", "sword": "hilt"}


def tool_name(kind, mat):
    return f"{mat}_{kind}"


def cut_kind(kind):
    return "sword" if kind == "sword" else "axe"  # every hafted tool cuts like an axe: the head stands out


def save(img, path, meta=None):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    Image.fromarray(img.astype(np.uint8)).save(path)
    parts.art.archive(path, os.path.relpath(path, DELIVER).replace("/", "__")[:-4], "parts")
    if meta:
        json.dump(meta, open(path[:-4] + ".json", "w"), indent=1)


def run(kind):
    ck = cut_kind(kind)
    mats = [(m, tool_name(kind, m), m in CRUDE) for m in MATERIALS if tf.drawn(tool_name(kind, m)) is not None]
    cut = {}
    for m, n, crude in mats:
        a = tf.drawn(n)
        cut[m] = (a,) + tf.regions(a, ck)
    socket = np.mean([c[2] for c in cut.values()], 0)
    metal = next((m for m in ("iron", "gold", "diamond") if m in cut), None)
    crude_src = "wood" if "wood" in cut else next(iter(cut))
    hm = tf.socketed(cut[metal][0], cut[metal][1], cut[metal][2], socket, [tf.HANDLE])
    hc = tf.socketed(cut[crude_src][0], cut[crude_src][1], cut[crude_src][2], socket, [tf.HANDLE])
    out = os.path.join(DELIVER, kind)
    boxes = {"work": [], "joint": []}
    rows = []
    for m, n, crude in mats:
        a, lab, base = cut[m]
        a2, lab2 = tf.socketed(a, lab, base, socket, [tf.JOINT, tf.WORK])
        work, joint = lab2 == tf.WORK, lab2 == tf.JOINT
        a2 = a2.astype(float)
        a2[..., :3] = tf.contrast(a2[..., :3], work, 1.2)
        # the override layer: the working part and its joint band, painted
        layer = np.zeros((32, 32, 4))
        layer[work | joint] = a2[work | joint]
        layer[..., 3] = np.where(work | joint, 255, 0)
        save(layer, os.path.join(out, f"{m}_{WORK[kind]}.png"),
             {"kind": f"base:{kind}", "layer": WORK[kind], "material": m, "override": True,
              "regions": [WORK[kind], JOINT[kind]], "socket": [round(float(socket[0]), 1), round(float(socket[1]), 1)]})
        reg = np.zeros((32, 32, 4))
        reg[..., 1] = np.where(joint, 1, 0)
        reg[..., 3] = np.where(work | joint, 255, 0)
        save(reg, os.path.join(out, f"{m}_{WORK[kind]}_regions.png"))
        for k, msk in (("work", work), ("joint", joint)):
            ys, xs = np.where(msk)
            if len(xs):
                boxes[k].append((int(xs.min()), int(ys.min()), int(xs.max()), int(ys.max())))
        # the composed tool, as the game would show it (one outline, soft seams)
        ha, hl = hc if crude else hm
        img = np.zeros((32, 32, 4))
        lab_out = np.zeros((32, 32), int)
        img[hl > 0] = ha[hl > 0]; lab_out[hl > 0] = tf.HANDLE
        img[joint] = a2[joint]; lab_out[joint] = tf.JOINT
        img[work] = a2[work]; lab_out[work] = tf.WORK
        composed = tf.one_outline(img, lab_out)
        p = os.path.join(parts.seg.ITEM, n + ".png")
        approved = Image.open(p).convert("RGBA") if os.path.exists(p) else None
        rows.append((m, composed, approved))
    # the handle layers: painted, and as a map for ramp materials
    for name, (ha, hl) in (("handle_wrapped", hm), ("handle_plain", hc)):
        layer = np.zeros((32, 32, 4))
        layer[hl > 0] = ha[hl > 0]
        layer[..., 3] = np.where(hl > 0, 255, 0)
        save(layer, os.path.join(out, name + ".png"), {"kind": f"base:{kind}", "layer": HANDLE[kind], "override": True,
                                                       "regions": [HANDLE[kind]], "steps": 8})
    ha, hl = hc
    mask = hl > 0
    steps = parts.steps(ha, mask)
    mp = np.zeros((32, 32, 4))
    mp[..., 0] = np.where(mask, np.clip(steps, 0, 7) * 36, 0)
    mp[..., 3] = np.where(mask, 255, 0)
    save(mp, os.path.join(out, "handle_map.png"), {"kind": f"base:{kind}", "layer": HANDLE[kind], "regions": [HANDLE[kind]],
                                                   "steps": 8})
    ys, xs = np.where(hm[1] > 0)

    def span(bs):
        b = np.array(bs)
        return [int(b[:, 0].min()), int(b[:, 1].min()), int(b[:, 2].max()), int(b[:, 3].max())]
    sock = {"kind": f"base:{kind}", "socket": [round(float(socket[0]), 1), round(float(socket[1]), 1)],
            WORK[kind]: span(boxes["work"]), JOINT[kind]: span(boxes["joint"]) if boxes["joint"] else None,
            HANDLE[kind]: [int(xs.min()), int(ys.min()), int(xs.max()), int(ys.max())]}
    json.dump(sock, open(os.path.join(out, "socket.json"), "w"), indent=1)
    sheet(kind, rows)
    return sock


def strips():
    """8x1 ramps for handle and fitting materials."""
    r = parts.ramps()
    birch = parts.ramp_from(np.array(Image.open(os.path.join(parts.seg.ITEM, "..", "block", "planks_birch.png"))
                                     .convert("RGB")).reshape(-1, 3))
    antler = parts.ramp_from(parts.icon_pixels("bone"))
    for name, ramp in {"oak": r["wood"], "birch": birch, "antler": antler, "leather": r["leather"],
                       "gold": r["gold fittings"], "cord": r["cord"]}.items():
        save(ramp.reshape(1, 8, 3).clip(0, 255), os.path.join(DELIVER, "strips", name + ".png"))


def sheet(kind, rows):
    S, pad = 96, 12
    font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)
    big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 20)
    W = max(pad + len(rows) * (S + pad) + 20, 900)
    im = Image.new("RGB", (W, 2 * (S + 70) + 60), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)
    d.text((pad, 12), f"{kind.capitalize()} from parts, every material (top), beside the approved icon (bottom)",
           fill=(0xf5, 0xe4, 0xbc), font=big)
    for r, (title, col) in enumerate((("Built from parts: drawn working part per material, one outline, soft seams", 1),
                                      ("The approved full icon", 2))):
        y = 48 + r * (S + 70)
        d.text((pad, y), title, fill=(0xff, 0xbe, 0x45), font=big)
        for k, row in enumerate(rows):
            x = pad + k * (S + pad)
            img = row[col]
            d.rectangle([x, y + 28, x + S - 1, y + 28 + S - 1], fill=(0x35, 0x20, 0x15), outline=(0x68, 0x3e, 0x26))
            if img is not None:
                t = img.resize((S - 8, S - 8), Image.NEAREST)
                im.paste(t, (x + 4, y + 32), t)
            else:
                d.text((x + 16, y + 28 + S // 2 - 8), "new", fill=(0xc2, 0xac, 0x88), font=font)
            d.text((x, y + 28 + S + 3), row[0], fill=(0xc2, 0xac, 0x88), font=font)
    out = os.path.join(HERE, "sheets", f"run_{kind}.png")
    im.save(out)
    parts.art.archive(out, f"sheet_run_{kind}", "sheets")
    print(out)


if __name__ == "__main__":
    kinds = sys.argv[1:] or ["pickaxe", "shovel", "hoe", "axe", "sword"]
    socks = {k: run(k) for k in kinds}
    strips()
    json.dump(socks, open(os.path.join(DELIVER, "sockets.json"), "w"), indent=1)
    print(json.dumps(socks))
