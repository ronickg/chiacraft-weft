"""The base game's mobs, whose models are built in code (`mods/base/src/mobs.rs`, the SDK's
`humanoid` and `spider`), as Bedrock geometry the catalogue's renderer reads: each part's pivot
(model pixels, y up), its boxes relative to the pivot with vanilla's box UV, mirrored or grown
like the code's. Keep in step with those files when a mob's model changes."""

import math


def part(name, pivot, boxes, rot=None):
    """`ModelPart::new(name, pivot)` with `.cuboid(origin, size, uv)` boxes (each optionally
    (origin, size, uv, mirror, inflate)) and `.rotation(x, y, z)`."""
    return {"name": name, "pivot": list(pivot), "rot": rot, "boxes": [b if len(b) == 5 else (*b, False, 0.0) for b in boxes]}


def vanilla(name, pivot, boxes, rot=None, mirror=False):
    """`vanilla_part`: written y down from 24 as vanilla's models are."""
    out = []
    for at, size, uv in boxes:
        out.append(([at[0], -(at[1] + size[1]), at[2]], size, uv, mirror, 0.0))
    return part(name, [pivot[0], 24.0 - pivot[1], pivot[2]], out, rot)


def vanilla_legs(x, y):
    h = 24.0 - y
    leg = lambda n, px, pz: vanilla(n, [px, y, pz], [([-2.0, 0.0, -2.0], [4.0, h, 4.0], [0.0, 16.0])])
    return [leg("leg0", -x, 7.0), leg("leg1", x, 7.0), leg("leg2", x, -5.0), leg("leg3", -x, -5.0)]


def creeper():
    leg = lambda n, x, z: vanilla(n, [x, 18.0, z], [([-2.0, 0.0, -2.0], [4.0, 6.0, 4.0], [0.0, 16.0])])
    return [vanilla("head", [0, 6, 0], [([-4, -8, -4], [8, 8, 8], [0, 0])]),
            vanilla("body", [0, 6, 0], [([-4, 0, -2], [8, 12, 4], [16, 16])]),
            leg("leg0", -2, 4), leg("leg1", 2, 4), leg("leg2", 2, -4), leg("leg3", -2, -4)], (64, 32)


def skeleton():
    f = 90.0
    return [vanilla("head", [0, 0, 0], [([-4, -8, -4], [8, 8, 8], [0, 0])]),
            vanilla("body", [0, 0, 0], [([-4, 0, -2], [8, 12, 4], [16, 16])]),
            vanilla("rightArm", [-5, 2, 0], [([-1, -2, -1], [2, 12, 2], [40, 16])], rot=[f, 0, 0], mirror=True),
            vanilla("leftArm", [5, 2, 0], [([-1, -2, -1], [2, 12, 2], [40, 16])], rot=[f, 0, 0]),
            vanilla("rightLeg", [-2, 12, 0], [([-1, 0, -1], [2, 12, 2], [0, 16])], mirror=True),
            vanilla("leftLeg", [2, 12, 0], [([-1, 0, -1], [2, 12, 2], [0, 16])])], (64, 32)


def sheep():
    wool = lambda p: [p[0], p[1] + 32.0]
    head = vanilla("head", [0, 6, -8], [([-3, -4, -6], [6, 6, 8], [0, 0])])
    head["boxes"].append(([-3.0, -2.0, -4.0], [6, 6, 6], wool([0, 0]), False, 0.6))
    body = vanilla("body", [0, 5, 2], [([-4, -10, -7], [8, 16, 6], [28, 8])], rot=[-90, 0, 0])
    body["boxes"].append(([-4.0, -6.0, -7.0], [8, 16, 6], wool([28, 8]), False, 1.75))
    legs = vanilla_legs(3.0, 12.0)
    for l in legs:
        l["boxes"].append(([-2.0, -6.0, -2.0], [4, 6, 4], wool([0, 16]), False, 0.5))
    return [head, body] + legs, (64, 64)


def chicken():
    leg = lambda n, x: vanilla(n, [x, 19, 1], [([-1, 0, -3], [3, 5, 3], [26, 0])])
    return [vanilla("head", [0, 15, -4], [([-2, -6, -2], [4, 6, 3], [0, 0]), ([-2, -4, -4], [4, 2, 2], [14, 0]), ([-1, -2, -3], [2, 2, 2], [14, 4])]),
            vanilla("body", [0, 16, 0], [([-3, -4, -3], [6, 8, 6], [0, 9])], rot=[-90, 0, 0]),
            vanilla("wing0", [-4, 13, 0], [([0, 0, -3], [1, 4, 6], [24, 13])]),
            vanilla("wing1", [4, 13, 0], [([-1, 0, -3], [1, 4, 6], [24, 13])]),
            leg("rightLeg", -2), leg("leftLeg", 1)], (64, 32)


def cow():
    head = vanilla("head", [0, 4, -8], [([-4, -4, -6], [8, 8, 6], [0, 0]), ([-3, 1, -7], [6, 3, 1], [1, 33]),
                                         ([-5, -5, -5], [1, 3, 1], [22, 0]), ([4, -5, -5], [1, 3, 1], [22, 0])])
    body = vanilla("body", [0, 5, 2], [([-6, -10, -7], [12, 18, 10], [18, 4]), ([-2, 2, -8], [4, 6, 1], [52, 0])], rot=[-90, 0, 0])
    return [head, body] + vanilla_legs(4.0, 12.0), (64, 64)


def pig():
    head = vanilla("head", [0, 12, -6], [([-4, -4, -8], [8, 8, 8], [0, 0]), ([-2, 0, -9], [4, 3, 1], [16, 16])])
    body = vanilla("body", [0, 11, 2], [([-5, -10, -7], [10, 16, 8], [28, 8])], rot=[-90, 0, 0])
    return [head, body] + vanilla_legs(3.0, 18.0), (64, 64)


def humanoid(arms_forward):
    a = 90.0 if arms_forward else 0.0
    return [part("head", [0, 24, 0], [([-4, 0, -4], [8, 8, 8], [0, 0])]),
            part("body", [0, 24, 0], [([-4, -12, -2], [8, 12, 4], [16, 16])]),
            part("rightArm", [-6, 22, 0], [([-2, -10, -2], [4, 12, 4], [40, 16], True, 0.0)], [a, 0, 0]),
            part("leftArm", [6, 22, 0], [([-2, -10, -2], [4, 12, 4], [40, 16])], [a, 0, 0]),
            part("rightLeg", [-2, 12, 0], [([-2, -12, -2], [4, 12, 4], [0, 16], True, 0.0)]),
            part("leftLeg", [2, 12, 0], [([-2, -12, -2], [4, 12, 4], [0, 16])])], (64, 32)


def spider():
    q = math.pi / 4
    parts = [part("head", [0, 9, -3], [([-4, -4, -8], [8, 8, 8], [32, 4])]),
             part("neck", [0, 9, 0], [([-3, -3, -3], [6, 6, 6], [0, 0])]),
             part("body", [0, 9, 9], [([-5, -4, -6], [10, 8, 12], [0, 12])])]
    for i, (z, splay, fan) in enumerate([(2.0, 1.0, 2.0), (1.0, 0.74, 1.0), (0.0, 0.74, -1.0), (-1.0, 1.0, -2.0)]):
        parts.append(part(f"leg{i * 2}", [-4, 9, z], [([-15, -1, -1], [16, 2, 2], [18, 0])], [0, math.degrees(fan * q / 2), math.degrees(splay * q)]))
        parts.append(part(f"leg{i * 2 + 1}", [4, 9, z], [([-1, -1, -1], [16, 2, 2], [18, 0])], [0, math.degrees(-fan * q / 2), math.degrees(-splay * q)]))
    return parts, (64, 32)


MOBS = {"zombie": lambda: humanoid(True), "skeleton": skeleton, "creeper": creeper, "spider": spider,
        "pig": pig, "cow": cow, "sheep": sheep, "chicken": chicken}


def geometry(name):
    """Bedrock geometry of mob `name`, or None."""
    if name not in MOBS:
        return None
    parts, (w, h) = MOBS[name]()
    bones = []
    for p in parts:
        cubes = []
        for origin, size, uv, mirror, inflate in p["boxes"]:
            c = {"origin": [p["pivot"][k] + origin[k] for k in range(3)], "size": list(size), "uv": list(uv)}
            if mirror:
                c["mirror"] = True
            if inflate:
                c["inflate"] = inflate
            cubes.append(c)
        b = {"name": p["name"], "pivot": p["pivot"], "cubes": cubes}
        if p["rot"]:
            # Code models turn in file space (the engine doesn't flip them as it does a Bedrock
            # file's bones), so as a file's bone their x and z turns change sign.
            b["rotation"] = [-p["rot"][0], p["rot"][1], -p["rot"][2]]
        bones.append(b)
    return {"minecraft:geometry": [{"description": {"identifier": f"geometry.{name}", "texture_width": w, "texture_height": h}, "bones": bones}]}
