"""Pictures of a perk web layout as the K screen draws it (`mods/base/src/web.rs`): the game's own
node and link textures at GUI scale 2 (a 32 px node texture 1:1), links as dots every 5 GUI px,
arcs along their circles. Writes the whole web, the screen's view at its opening zoom, and one
region close up, to compare layouts before a game run.

    cargo run -p vitals --bin web_layout -- clusters > layout.json
    ~/.venvs/blocky/bin/python mods/base/source/ui/web_map.py layout.json out_prefix [taken ids...]
"""
import json
import math
import os
import sys

from PIL import Image

GUI = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "assets", "textures", "gui")
SCALE = 2  # screen px per GUI px
SIZES = {"small": 8, "notable": 12, "keystone": 16, "start": 12, "socket": 12}
GATES = [0, 0, 0, 5, 10, 15, 22, 30, 40, 55, 70, 85, 100]
WELL = (41, 31, 24, 255)


def tex(name):
    return Image.open(os.path.join(GUI, name + ".png")).convert("RGBA")


def gate(ring):
    return GATES[max(1, min(len(GATES) - 1, math.ceil(ring)))]


def draw(layout, taken, skill, zoom, centre, size, path):
    w, h = size
    im = Image.new("RGBA", (w * SCALE, h * SCALE), WELL)
    nodes = {n["id"]: n for n in layout["nodes"]}
    near = {}
    for a, b, _ in layout["links"]:
        near.setdefault(a, []).append(b)
        near.setdefault(b, []).append(a)
    times = 2 if zoom >= 2 else 1

    def place(x, y):
        return (round((x - centre[0]) * zoom + w / 2), round((y - centre[1]) * zoom + h / 2))

    dot_far, dot_lit = tex("perk_link"), tex("perk_link_taken")
    d = SCALE * times
    for a, b, arc in layout["links"]:
        na, nb = nodes[a], nodes[b]
        lit = a in taken and b in taken
        dot = (dot_lit if lit else dot_far).resize((d, d), Image.NEAREST)
        pts = []
        if arc:
            cx, cy = arc
            r = math.hypot(na["x"] - cx, na["y"] - cy)
            a0 = math.atan2(na["y"] - cy, na["x"] - cx)
            a1 = math.atan2(nb["y"] - cy, nb["x"] - cx)
            da = (a1 - a0 + math.pi) % (2 * math.pi) - math.pi
            steps = max(1, int(abs(da) * r * zoom / (3 * times)))
            pts = [(cx + r * math.cos(a0 + da * k / steps), cy + r * math.sin(a0 + da * k / steps)) for k in range(1, steps)]
        else:
            ln = math.hypot(nb["x"] - na["x"], nb["y"] - na["y"]) * zoom
            steps = max(1, int(ln / (3 * times)))
            pts = [(na["x"] + (nb["x"] - na["x"]) * k / steps, na["y"] + (nb["y"] - na["y"]) * k / steps) for k in range(1, steps)]
        for x, y in pts:
            px, py = place(x, y)
            if 0 <= px < w and 0 <= py < h:
                im.alpha_composite(dot, (px * SCALE - d // 2, py * SCALE - d // 2))
    for n in layout["nodes"]:
        kind = n["kind"]
        if n["id"] in taken or kind == "start":
            state = "taken"
        elif any(o in taken for o in near.get(n["id"], [])):
            state = "open" if gate(n["ring"]) <= skill else "locked"
        else:
            state = "far" if gate(n["ring"]) <= skill else "locked"
        name = f"perk_{kind}_{state}" if kind != "start" else "perk_start_taken"
        s = SIZES[kind] * times
        t = tex(name).resize((s * SCALE, s * SCALE), Image.NEAREST)
        px, py = place(n["x"], n["y"])
        if s / 2 <= px <= w - s / 2 and s / 2 <= py <= h - s / 2:
            im.alpha_composite(t, (px * SCALE - s * SCALE // 2, py * SCALE - s * SCALE // 2))
    im.convert("RGB").save(path)


def main():
    layout = json.load(open(sys.argv[1]))
    out = sys.argv[2]
    taken = {int(t) for t in sys.argv[3:]} | {0}
    xs = [n["x"] for n in layout["nodes"]]
    ys = [n["y"] for n in layout["nodes"]]
    ext = max(max(map(abs, xs)), max(map(abs, ys))) + 20
    # The whole web at the opening zoom, the screen's view (its well, 340 x 236 GUI px), and Melee
    # close up at the 1.4 step.
    side = int(2 * ext * 0.55) + 20
    draw(layout, taken, 100, 0.55, (0, 0), (side, side), out + "_whole.png")
    melee = [n for n in layout["nodes"] if n["region"] == 0 and not (40 <= n["id"] % 64 <= 42)]
    cx = sum(n["x"] for n in melee) / len(melee)
    cy = sum(n["y"] for n in melee) / len(melee)
    # The screen opens on the player's main island at 0.75 (`web.rs`, `open_on_home`).
    draw(layout, taken, 100, 0.75, (cx, cy), (340, 236), out + "_screen.png")
    draw(layout, taken, 100, 0.55, (0, 0), (340, 236), out + "_hub.png")
    draw(layout, taken, 100, 1.4, (cx, cy), (340, 236), out + "_region.png")


if __name__ == "__main__":
    main()
