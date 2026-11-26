"""The block textures made in code from the approved 32 px tiles rather than generated: their
shapes must stay where the models read them, or they are another tile recoloured.

    python3 notes/blocks/derive.py        # writes tiles/, then: blocks.py install batches/derived.tsv

- Worn farmland (`farmland_worn_1..3`): the farmland, paler and duller as it wears out.
- Glass: the generated glare streaks made pale; stained glass: the glass frame recoloured, its inside the dye colour, see-through.
- Water, lava and the amber gateway: 16 frames of a seamless pattern that drifts and loops (`<name>.anim` stays).
- Mechanical faces (beds, torches, lever, button, redstone parts, the anchor, the floater): the
  old 16 px picture doubled, each of its colours drawn with the matching approved material (oak,
  wool, smooth stone) so the shapes stay where the models expect them.
"""
import os
import subprocess

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
TILES = os.path.join(HERE, "tiles")
# The old 16 px pictures, as the game had them before the 32 px redo.
OLD = "79be3ec"


def tile(name):
    return np.asarray(Image.open(os.path.join(TILES, name + ".png")).convert("RGBA")).astype(float)


def save(a, name):
    Image.fromarray(np.clip(a, 0, 255).round().astype(np.uint8), "RGBA").save(os.path.join(TILES, name + ".png"))


def old(name):
    data = subprocess.run(["git", "show", f"{OLD}:mods/base/assets/textures/block/{name}.png"], cwd=ROOT, capture_output=True, check=True).stdout
    from io import BytesIO
    return np.asarray(Image.open(BytesIO(data)).convert("RGBA")).astype(float)


def worn_farmland():
    base = tile("farmland")
    dust = np.array([0x9A, 0x86, 0x6C], float)
    for level in (1, 2, 3):
        a = base.copy()
        k = 0.17 * level
        mean = a[..., :3].mean((0, 1))
        a[..., :3] = (a[..., :3] - mean) * (1 - 0.15 * level) + mean
        a[..., :3] = a[..., :3] * (1 - k) + dust * k
        save(a, f"farmland_worn_{level}")


def clear_glass():
    """The generated glass's glare streaks pick up a pink cast from the magenta: pale blue-white."""
    glass = tile("glass")
    inside = np.zeros(glass.shape[:2], bool)
    inside[3:-3, 3:-3] = True
    glare = inside & (glass[..., 3] > 0)
    glass[glare, :3] = [0xDC, 0xEC, 0xF2]
    save(glass, "glass")


def stained_glass():
    glass = tile("glass")
    for name, rgb in (("red", (0xC0, 0x30, 0x30)), ("green", (0x3E, 0x9A, 0x40)), ("blue", (0x34, 0x5E, 0xC0))):
        c = np.array(rgb, float)
        a = glass.copy()
        lum = (a[..., :3] @ [0.3, 0.55, 0.15]) / 255
        solid = a[..., 3] > 0
        a[..., :3] = np.where(solid[..., None], c * (0.45 + 0.75 * lum[..., None]), c)
        a[..., 3] = np.where(solid, 255, 100)
        save(a, f"glass_{name}")


def showcase_glass():
    """The RTX showcase's glass (notes/RTX_SHOWCASE.md), from the approved glass: frosted (a milky
    pane with frost crystals), smoked (a dark tinted pane, its glare grey) in the same frame, and
    leaded glass (no wooden frame: panes of the dye colours and amber between tin cames)."""
    glass = tile("glass")
    solid = glass[..., 3] > 0
    inside = np.zeros(glass.shape[:2], bool)
    inside[3:-3, 3:-3] = True
    rng = np.random.default_rng(7)
    frost = glass.copy()
    n = rng.random(glass.shape[:2])
    crystals = (n > 0.82) & inside
    pane = ~solid | (solid & inside)
    frost[pane, :3] = [0xE4, 0xEC, 0xF0]
    frost[pane, 3] = 170
    frost[crystals, :3] = [0xF6, 0xFA, 0xFC]
    frost[crystals, 3] = 205
    save(frost, "frosted_glass")
    smoke = glass.copy()
    glare = solid & inside
    smoke[~solid, :3] = [0x2A, 0x2C, 0x33]
    smoke[~solid, 3] = 165
    smoke[glare, :3] = [0x6A, 0x6E, 0x78]
    smoke[glare, 3] = 200
    save(smoke, "smoked_glass")
    leaded_glass()


def leaded_glass():
    """Leaded glass (Ronald, 11-06, redo: "glass panes in lead ... not a patchwork of opaque blocks"):
    diamond quarries 16 px across between thin dark lead cames (1 px, diagonal), each pane see-through
    and lightly tinted (mostly clear and pale amber, every few a deep red or blue), a short glint
    near each pane's top; seamless, as the lattice repeats every 16 px."""
    yy, xx = np.mgrid[0:32, 0:32]
    u, v = (xx + yy) % 16, (xx - yy) % 16
    came = (u == 0) | (v == 0)
    cu, cv = (xx + yy) // 16, ((xx - yy) % 32) // 16
    tints = {
        "clear": ((0xDC, 0xEC, 0xF2), 70),
        "amber": ((0xF0, 0xD0, 0x80), 105),
        "green": ((0xB8, 0xE0, 0xB0), 90),
        "red": ((0xC0, 0x2C, 0x2C), 150),
        "blue": ((0x2E, 0x58, 0xC0), 150),
    }
    pattern = [["clear", "amber"], ["green", "clear"], ["amber", "red"], ["clear", "blue"]]
    lead = np.zeros((32, 32, 4))
    for y in range(32):
        for x in range(32):
            if came[y, x]:
                continue
            name = pattern[(cu[y, x] + cv[y, x]) % 4][(cu[y, x]) % 2]
            c, a = tints[name]
            lead[y, x, :3] = c
            lead[y, x, 3] = a
            # A glint: near the pane's upper corner, a short light streak.
            if 2 <= u[y, x] <= 4 and 5 <= v[y, x] <= 7:
                lead[y, x, :3] = np.minimum(np.array(c, float) * 0.5 + 150, 255)
                lead[y, x, 3] = min(a + 70, 220)
    lead[came, :3] = [0x3A, 0x3E, 0x46]
    lead[came, 3] = 255
    # The cames' light edge: the row beside each came, on its upper side, a little lit.
    lit = (u == 15) | (v == 1)
    lead[lit, :3] = np.minimum(lead[lit, :3] * 0.8 + 50, 255)
    lead[lit, 3] = np.minimum(lead[lit, 3] + 40, 230)
    save(lead, "leaded_glass")


def showcase_tiles():
    """From the generated showcase tiles: checker tiles (polished limestone and slate tiles in 16 px
    squares with a grout line), verdigris copper (`verdigris`), and the green and red glazed tiles (the
    blue glaze recoloured, its grout and glints kept)."""
    lime = tile("polished_limestone")
    regrid("rsb_stone", 3, 4, [0x26, 0x2A, 0x2E], "slate_tiles", edge=0.08)
    slate = tile("slate_tiles")
    check = lime.copy()
    yy, xx = np.mgrid[0:32, 0:32]
    dark = ((yy // 16) + (xx // 16)) % 2 == 1
    check[dark] = slate[dark] * 0.82
    grout = (yy % 16 == 0) | (xx % 16 == 0)
    check[grout, :3] = [0x8E, 0x86, 0x78]
    check[..., 3] = 255
    save(check, "checker_tiles")
    verdigris()
    regrid("rsb_metal", 3, 4, [0xD8, 0xCF, 0xBA], "glazed_tiles_blue")
    blue = tile("glazed_tiles_blue")
    rgb = blue[..., :3]
    lum = (rgb @ [0.3, 0.55, 0.15]) / 255
    sat = rgb.max(-1) - rgb.min(-1)
    glaze = (sat > 40) & (rgb[..., 2] > rgb[..., 0])
    for name, c in (("green", (0x2E, 0x8A, 0x52)), ("red", (0xA8, 0x30, 0x2A))):
        a = blue.copy()
        tint = np.array(c, float) * (0.35 + 1.3 * lum[..., None])
        a[..., :3] = np.where(glaze[..., None], np.minimum(tint, 255), rgb)
        save(a, f"glazed_tiles_{name}")


def verdigris():
    """Verdigris copper (Ronald, 11-06, redo: "a finer, even patina, mostly blue-green with small
    bare-copper flecks at edges and rivets, and seamless across 2x2"): the copper block's own shading
    (its bevel, rivets) under a fine two-tone blue-green mottle, a little darker in its hollows;
    bare copper only on the brightest texels near the edge (rivet heads, the frame's lit top) and a
    few single flecks along the frame."""
    copper = tile("copper_block")
    rng = np.random.default_rng(23)
    lum = (copper[..., :3] @ [0.3, 0.55, 0.15]) / 255
    shade = np.clip(0.55 + 1.6 * (lum - lum.mean()), 0, 1)
    ramp = np.array([[0x2E, 0x6C, 0x60], [0x3E, 0x88, 0x78], [0x52, 0xA0, 0x8E], [0x6C, 0xB8, 0xA4]], float)
    pick = np.clip((rng.random((32, 32)) * 0.35 + shade * 0.85) * 3.2, 0, 3).astype(int)
    v = copper.copy()
    v[..., :3] = ramp[pick] * (0.75 + 0.5 * shade[..., None])
    yy, xx = np.mgrid[0:32, 0:32]
    edge = np.minimum(np.minimum(xx, 31 - xx), np.minimum(yy, 31 - yy))
    # Bare copper: the rivet heads (the brightest texels in each corner) and a few flecks on the rim.
    corner = (np.minimum(xx, 31 - xx) <= 6) & (np.minimum(yy, 31 - yy) <= 6)
    hot = corner & (lum > np.quantile(lum[corner], 0.85))
    bare = hot | ((edge <= 1) & (rng.random((32, 32)) < 0.06))
    v[bare, :3] = copper[bare, :3]
    v[..., :3] = np.clip(v[..., :3], 0, 255)
    v[..., 3] = 255
    save(v, "verdigris_copper")


def regrid(image, quadrant, n, grout, out, edge=0.12):
    """A tile set of n x n square tiles from a generated image's quadrant (0 top left .. 3 bottom
    right), redrawn on an exact grid: each cell's middle averaged down to (32/n - 1) px, a 1-px grout
    line below and right of it, its top and left rows a little lighter (the bevel). Cutting the whole
    square to 32 px left the tiles and grout uneven (11-06, glazed and slate tiles)."""
    g = np.asarray(Image.open(os.path.join(HERE, "gen", image + ".png")).convert("RGB")).astype(float)
    h = g.shape[0] // 2
    q = g[(quadrant // 2) * h:(quadrant // 2 + 1) * h, (quadrant % 2) * h:(quadrant % 2 + 1) * h]
    step = 32 // n
    inner = step - 1
    a = np.zeros((32, 32, 4))
    a[..., :3] = grout
    a[..., 3] = 255
    span = q.shape[0] / n
    for cy in range(n):
        for cx in range(n):
            m = int(span * 0.1)
            cell = q[int(cy * span) + m:int((cy + 1) * span) - m, int(cx * span) + m:int((cx + 1) * span) - m]
            small = np.asarray(Image.fromarray(cell.astype(np.uint8)).resize((inner, inner), Image.BOX)).astype(float)
            small[0, :] = np.minimum(small[0, :] * (1 + edge) + 6, 255)
            small[:, 0] = np.minimum(small[:, 0] * (1 + edge) + 6, 255)
            a[cy * step:cy * step + inner, cx * step:cx * step + inner, :3] = small
    save(a, out)


def drift(size=32, frames=16, seed=0, top=4):
    """A seamless pattern in [0, 1] that moves one full period over the frames, so it loops."""
    rng = np.random.default_rng(seed)
    y, x = np.mgrid[0:size, 0:size] / size * 2 * np.pi
    waves = [(rng.integers(1, top), rng.integers(1 - top, top), rng.uniform(0, 2 * np.pi), rng.uniform(0.5, 1)) for _ in range(6)]
    out = []
    for f in range(frames):
        t = f / frames * 2 * np.pi
        v = sum(amp * np.sin(kx * x + ky * y + ph + t * (1 if i % 2 else -1)) for i, (kx, ky, ph, amp) in enumerate(waves))
        out.append((v - v.min()) / (v.max() - v.min()))
    return out


def water():
    # Light grey, partly see-through: the game tints it by the biome's water colour.
    frames = []
    for v in drift(seed=3):
        g = 214 + np.round(v * 4) / 4 * 38
        frames.append(np.dstack([g, g, g, np.full_like(g, 170)]))
    save(np.vstack(frames), "water_still")


def lava():
    ramp = np.array([[0x41, 0x16, 0x12], [0x7B, 0x16, 0x13], [0xCD, 0x1C, 0x18], [0xE8, 0x5A, 0x10], [0xF3, 0x9C, 0x00], [0xFA, 0xD8, 0x6A]], float)
    frames = []
    for v in drift(seed=7, top=3):
        # Dark crust plates; where the pattern crosses its middle, a glowing crack.
        crack = np.clip(1 - np.abs(v - 0.5) * 6, 0, 1)
        i = np.clip(np.round((0.2 + 0.3 * v + 0.8 * crack) * (len(ramp) - 1)), 0, len(ramp) - 1).astype(int)
        rgb = ramp[i]
        frames.append(np.dstack([rgb, np.full(v.shape, 255)]))
    save(np.vstack(frames), "lava_still")


def gateway():
    """The amber gateway's surface (animals): a slow amber swirl, partly see-through."""
    ramp = np.array([[0x96, 0x32, 0x06], [0xB8, 0x52, 0x10], [0xD8, 0x7A, 0x1E], [0xF0, 0xA8, 0x34], [0xFF, 0xC4, 0x40]], float)
    frames = []
    for v in drift(seed=11, top=3):
        i = np.clip(np.round(v * (len(ramp) - 1)), 0, len(ramp) - 1).astype(int)
        frames.append(np.dstack([ramp[i], np.full(v.shape, 215)]))
    Image.fromarray(np.vstack(frames).round().astype(np.uint8), "RGBA").save(os.path.join(TILES, "gateway.png"))


# Each old colour family -> how to draw it at 32 px.
def material(rgb, a):
    r, g, b = rgb
    if a == 0:
        return None
    if (r, g, b) == (0x46, 0x46, 0x46):
        return "frame"
    if r > 200 and g < 60:
        return "red_on"
    if 0x50 < r < 0x70 and g < 0x30:
        return "red_off"
    if r > 200 and g > 200 and b > 200:
        return "white"
    if r > 140 and g < 60:
        return "wool_red"
    if 0x70 < r < 0x80 and g < 0x20:
        return "wool_red"
    if r > 0xE0 and g > 0xA0 and b < 0x80:
        return "flame"
    if g > r + 20 and g > b:
        return "green"
    if r < 0x40 and abs(r - g) < 12 and abs(g - b) < 12:
        return "socket"
    if r > g + 18 and g > b + 8:
        return "wood"
    return "stone"


def remap(name, wood="planks_oak"):
    src = old(name)
    big = src.repeat(32 // src.shape[0], 0).repeat(32 // src.shape[1], 1) if src.shape[0] == 16 else src
    stone, oak, red, white, slime = tile("smooth_stone"), tile(wood), tile("wool_red"), tile("wool_white"), tile("slime_block")
    out = np.zeros((32, 32, 4))
    kinds = np.empty((32, 32), object)
    for y in range(32):
        for x in range(32):
            kinds[y, x] = material(tuple(int(c) for c in big[y, x, :3]), big[y, x, 3])
    for y in range(32):
        for x in range(32):
            k = kinds[y, x]
            if k is None:
                continue
            # Symbols are lit from the top left: lighter on an edge facing up or left, darker down or right.
            edge = 0
            if k in ("red_on", "red_off", "flame", "green", "socket"):
                if y == 0 or kinds[y - 1, x] != k or x == 0 or kinds[y, x - 1] != k:
                    edge = 1
                elif y == 31 or kinds[y + 1, x] != k or x == 31 or kinds[y, x + 1] != k:
                    edge = -1
            px = {
                "frame": stone[y, x, :3] * 0.55,
                "stone": stone[y, x, :3],
                "wood": oak[y, x, :3],
                "wool_red": red[y, x, :3],
                "white": white[y, x, :3] if name.startswith("bed") else big[y, x, :3],
                "red_on": np.array([0xD8, 0x2A, 0x22]),
                "red_off": np.array([0x6E, 0x22, 0x1C]),
                "flame": big[y, x, :3],
                "green": slime[y, x, :3],
                "socket": np.array([0x2E, 0x2A, 0x28]),
            }[k]
            out[y, x, :3] = px * (1.15 if edge > 0 else 0.8 if edge < 0 else 1)
            out[y, x, 3] = big[y, x, 3]
    save(out, name)


MECHANICAL = [
    "anchor", "anchor_on", "button", "comparator", "comparator_on", "repeater", "repeater_on", "dispenser_front_vertical",
    "floater_back", "floater_front", "floater_side", "floater_side_going", "floater_side_stuck",
    "lever", "torch", "redstone_torch", "redstone_torch_off",
    "bed_foot_east", "bed_foot_south", "bed_foot_west", "bed_head_east", "bed_head_north", "bed_head_west",
    "bed_top_foot", "bed_top_foot_90", "bed_top_foot_180", "bed_top_foot_270",
    "bed_top_head_0", "bed_top_head_90", "bed_top_head_180", "bed_top_head_270",
]
def observer_back_on():
    """The observer's back when it fires: the same back as observer_back, its socket's core lit red
    with a light rim (the old 16 px picture was a different slab with a red square; 11-04)."""
    a = tile("observer_back")
    # brighter than wanted by 1/0.75: the grade darkens the observer's back by 0.75 (grade.toml)
    lit = (np.array([[0xFF, 0x9A, 0x7A], [0xF0, 0x40, 0x30], [0xC8, 0x1E, 0x1E], [0x7A, 0x10, 0x10]], float) / 0.75).clip(0, 255)
    for y in range(14, 18):
        for x in range(14, 18):
            k = 0 if (y, x) == (14, 14) else 1 if (y == 14 or x == 14) else 3 if (y == 17 or x == 17) else 2
            a[y, x, :3] = lit[k]
    for y in range(13, 19):  # the socket's rim catches the light
        for x in range(13, 19):
            if not (14 <= y <= 17 and 14 <= x <= 17):
                a[y, x, :3] = a[y, x, :3] * 0.6 + np.array([0x9A, 0x22, 0x1A]) * 0.4
    save(a, "observer_back_on")


def piston_inner():
    """Inside the piston's base, under the head: the base's stone (piston_bottom) with a square
    shaft in the middle, framed in the head's iron, as vanilla's (11-04; the old one was a pale slab)."""
    a = tile("piston_bottom")
    iron = tile("piston_top")[0:2, :, :3].reshape(-1, 3).mean(0)  # the frame's iron
    for y in range(11, 21):
        for x in range(11, 21):
            ring = y in (11, 20) or x in (11, 20)
            if ring:
                a[y, x, :3] = iron * (1.15 if (y == 11 or x == 11) else 0.75)
            else:
                d = min(y - 12, x - 12, 19 - y, 19 - x)
                a[y, x, :3] = np.array([0x1E, 0x1C, 0x1A]) + (6 - min(d, 3) * 2) * np.array([4, 4, 4])
    save(a, "piston_inner")


# Redstone dust is grey (the game tints it by power): its old picture doubled.
GREY = ["redstone_dust_dot", "redstone_dust_line"]


if __name__ == "__main__":
    import sys
    if len(sys.argv) > 1:  # just these: python3 derive.py observer_back_on piston_inner
        for f in sys.argv[1:]:
            globals()[f]()
        raise SystemExit
    observer_back_on()
    piston_inner()
    worn_farmland()
    clear_glass()
    stained_glass()
    water()
    lava()
    gateway()
    for n in MECHANICAL:
        remap(n, "log_oak" if n in ("lever", "torch", "redstone_torch", "redstone_torch_off") else "planks_oak")
    for n in GREY:
        a = old(n)
        save(a.repeat(2, 0).repeat(2, 1), n)
    print("derived:", "farmland_worn_1-3 glass_red/green/blue water_still lava_still", " ".join(MECHANICAL + GREY))
