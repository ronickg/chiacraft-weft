"""The round minimap's ring (Ronald, 11-05: "I want a kind of round mini map, like in that one in
the sort of UI images we had there": the compass map on `Voxel Fantasy Survival UI Suite.png`'s
HUD row). Drawn in code on the kit's pixel grid (1 UI px, half a GUI px), with the kit's palette
and its panel frame made round: an outline, a 2 px bronze band lit top-left and shaded
bottom-right, the groove, the inner bronze line and an inner outline round the map. Four round
plates sit on the ring at north, east, south and west; north's is the main orange face. The game
writes the letters (`map.base.north`...), so they translate.

    python3 notes/ui/kit2/minimap_ring.py            # writes mods/base/assets/textures/gui/minimap.png + .gui
    python3 notes/ui/kit2/minimap_ring.py --sheet S  # also a 1x/4x preview at S

The numbers the game needs (`mods/base/src/map.rs`, `vitals::waystones`): the texture is SIDE UI px
(SIDE/2 GUI px) a side, the map shows inside radius INNER UI px, the plates' centres are PLATE_AT
from the centre, each PLATE UI px across."""
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kit2 as k  # noqa: E402

SIDE = 152  # UI px: 76 GUI px
OUTER = 69  # the ring's outer radius (UI px)
INNER = 62  # inside this the map shows (the map's own clip is 64: the ring's inner edge hides its steps)
PLATE = 22
PLATE_AT = 65  # the plates' centres, from the centre (on the bronze band)


def polar(size=SIDE):
    """Each pixel's distance from the centre and how much its outward side faces the light (the
    top left: 1 facing it, -1 facing away)."""
    c = size / 2
    y, x = np.mgrid[0:size, 0:size] + 0.5
    dx, dy = x - c, y - c
    d = np.hypot(dx, dy)
    face = (-dx - dy) / np.maximum(d, 1e-6) / np.sqrt(2)
    return d, face


def disc(r, size=SIDE):
    return polar(size)[0] <= r


def plate(kind):
    """A round plate: an outline, a 1 px bronze rim lit top-left, the kit's face inside (the main
    orange for north, the dark secondary for the others). No letter: the game writes it."""
    a = k.canvas(PLATE, PLATE)
    d, face = polar(PLATE)
    r = PLATE / 2
    k.paint(a, d <= r, k.OUTLINE)
    rim = (d <= r - 1) & (d > r - 2)
    k.paint(a, rim, k.BRONZE)
    k.paint(a, rim & (face > 0.35), k.BRONZE_HI)
    k.paint(a, rim & (face < -0.35), k.BRONZE_D)
    light, body, shade = k.FACES[kind]
    inner = d <= r - 2
    k.paint(a, inner, body)
    k.paint(a, inner & (d > r - 3) & (face > 0.35), light)
    k.paint(a, inner & (d > r - 4) & (face < -0.35), shade)
    return a


def ring():
    """Outline, a 3 px bronze band (its outer row lit where it faces the top left and shaded where
    it faces away, its inner row the other way round: a raised rim), the groove, the inner bronze
    line, the inner outline round the map."""
    a = k.canvas(SIDE, SIDE)
    d, face = polar()
    k.paint(a, (d <= OUTER) & (d > INNER), k.OUTLINE)
    band = (d <= OUTER - 1) & (d > OUTER - 4)
    k.paint(a, band, k.BRONZE)
    outer_row = band & (d > OUTER - 2)
    inner_row = band & (d <= OUTER - 3)
    k.paint(a, outer_row & (face > 0.3), k.BRONZE_HI)
    k.paint(a, outer_row & (face < -0.3), k.BRONZE_D)
    k.paint(a, inner_row & (face < -0.3), k.BRONZE_L)
    k.paint(a, inner_row & (face > 0.3), k.BRONZE_D)
    k.paint(a, (d <= OUTER - 4) & (d > OUTER - 5), k.PANEL_DEEP)  # the groove
    k.paint(a, (d <= OUTER - 5) & (d > OUTER - 6), k.BRONZE_D)  # the inner line
    # OUTER - 6 .. INNER stays the outline painted first: the map's dark edge.
    c = SIDE // 2
    h = PLATE // 2
    for (dx, dy), kind in (((0, -1), "main"), ((1, 0), "second"), ((0, 1), "second"), ((-1, 0), "second")):
        k.blit(a, plate(kind), c + dx * PLATE_AT - h, c + dy * PLATE_AT - h)
    return a


def main():
    a = ring()
    gui = os.path.join(k.REPO, "mods", "base", "assets", "textures", "gui")
    Image.fromarray(a, "RGBA").save(os.path.join(gui, "minimap.png"))
    with open(os.path.join(gui, "minimap.gui"), "w") as f:
        f.write(f"scaling = stretch\nsize = {SIDE // 2} {SIDE // 2}\n")
    if "--sheet" in sys.argv:
        out = sys.argv[sys.argv.index("--sheet") + 1]
        im = Image.fromarray(a, "RGBA")
        s = Image.new("RGBA", (SIDE * 5 + 30, SIDE * 4 + 20), (26, 19, 19, 255))
        s.alpha_composite(im, (10, 10))
        s.alpha_composite(im.resize((SIDE * 4, SIDE * 4), Image.NEAREST), (SIDE + 20, 10))
        s.save(out)
    print("minimap ring", SIDE, "UI px, inner", INNER)


if __name__ == "__main__":
    main()
