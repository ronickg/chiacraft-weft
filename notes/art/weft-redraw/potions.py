"""Every potion and the empty bottle from one approved flask (potion_base_a): the green liquid is
re-shaded in each effect's colour (mods/base/src/effects.rs), lightness clamped so nothing goes near
black or washes out; the empty bottle has the liquid taken out."""
import colorsys, os, sys
from PIL import Image
EFFECTS = dict(speed=0x33EBFF, slowness=0x8BAFE0, haste=0xD9C043, mining_fatigue=0x4A4217, strength=0xFFC700,
    weakness=0x484D48, jump_boost=0xFDFF84, slow_falling=0xF3CFB9, levitation=0xCEFFFF, regeneration=0xCD5CAB,
    poison=0x87A363, wither=0x736156, resistance=0x9146F0, fire_resistance=0xFF9900, health_boost=0xF87D23,
    night_vision=0xC2FF66, blindness=0x1F1F23, invisibility=0xF6F6F6, instant_health=0xF82423,
    instant_damage=0xA9656A, hunger=0x587653, water_breathing=0x98DAC0, nausea=0x551D4A, glowing=0x94A061)
src = Image.open(sys.argv[1]).convert("RGBA"); out = sys.argv[2]; os.makedirs(out, exist_ok=True)
px = src.load(); W, H = src.size
GLASS = (106, 134, 166, 255)
for y in range(H):
    for x in range(W):
        r, g, b, a = px[x, y]
        if a and r > 120 and b > 160 and g < 120:  # magenta fringe left at the glass's corners
            px[x, y] = GLASS
liquid = lambda p: p[3] and p[1] > p[0] + 30 and p[1] > p[2] + 30
hls = lambda p: colorsys.rgb_to_hls(*(c / 255 for c in p[:3]))
base_l = hls((82, 190, 42))[1]
def shade(col):
    im = src.copy(); q = im.load()
    h, l, s = hls(((col >> 16) & 255, (col >> 8) & 255, col & 255))
    L = min(max(l, 0.34), 0.70); s = max(s, 0.18) if s > 0.05 else s
    for y in range(H):
        for x in range(W):
            if liquid(q[x, y]):
                d = hls(q[x, y])[1] - base_l
                nl = min(max(L + d * 1.15, 0.12), 0.93)
                q[x, y] = tuple(round(c * 255) for c in colorsys.hls_to_rgb(h, nl, s)) + (255,)
    return im
for name, col in EFFECTS.items():
    shade(col).save(f"{out}/potion_{name}.png")
empty = src.copy(); q = empty.load()
for y in range(H):
    for x in range(W):
        if liquid(q[x, y]): q[x, y] = (0, 0, 0, 0)
empty.save(f"{out}/glass_bottle.png")
