"""The shade map of Weft's 3D bow skin (11-26): mods/base/assets/textures/item/bow.png (Weft's,
16 px) to textures/parts/bow/skin.png, painted per stack with its stave, grip and string ramps
(`crafting::looks::bow_skin`, the engine's `<file>|<skin>` model). A shade map's red is the ramp
step times 36, green the region: 0 the stave's wood, 1 the grip, 2 the string. Rerun when Weft's
skin changes:  python3 mods/base/source/items/bow_skin_map.py
"""
import os
from PIL import Image

ROOT = os.path.join(os.path.dirname(__file__), "..", "..", "assets", "textures")
# Weft's colours, each to (region, step): the wood dark to light, the grip's wrap, the string.
STEPS = {
    (0x2C, 0x1D, 0x0F): (0, 0), (0x3C, 0x28, 0x16): (0, 1), (0x5C, 0x44, 0x24): (0, 3),
    (0x72, 0x55, 0x2E): (0, 4), (0x8A, 0x68, 0x3A): (0, 5),
    (0x4A, 0x2E, 0x20): (1, 1), (0x6C, 0x46, 0x2E): (1, 3),
    (0xB4, 0xB4, 0xB4): (2, 4), (0xDE, 0xDE, 0xDE): (2, 6),
}

src = Image.open(os.path.join(ROOT, "item", "bow.png")).convert("RGBA")
out = Image.new("RGBA", src.size)
for y in range(src.size[1]):
    for x in range(src.size[0]):
        r, g, b, a = src.getpixel((x, y))
        if not a:
            continue
        region, step = STEPS[(r, g, b)]
        out.putpixel((x, y), (step * 36, region, 0, a))
out.save(os.path.join(ROOT, "parts", "bow", "skin.png"))
print("wrote textures/parts/bow/skin.png")
