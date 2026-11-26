"""Before/after sheet: pairs.py OUT.png TITLE before1:after1 before2:after2 ...
Each row cell: Weft's (before) and ours (after), nearest-scaled to 96 px on the slot colour, plus 1x."""
import sys, os
from PIL import Image, ImageDraw, ImageFont
out, title, pairs = sys.argv[1], sys.argv[2], [p.split(":") for p in sys.argv[3:]]
S = 96; cols = 4; cw = S * 2 + 60; ch = S + 40
rows = (len(pairs) + cols - 1) // cols
im = Image.new("RGBA", (cols * cw + 20, rows * ch + 50), (26, 19, 19, 255)); d = ImageDraw.Draw(im)
try: f = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 13); ft = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 20)
except Exception: f = ft = None
d.text((12, 12), title + "   (left: Weft's, right: ours)", fill=(242, 218, 154, 255), font=ft)
def put(p, x, y):
    d.rectangle([x, y, x + S - 1, y + S - 1], fill=(53, 32, 21, 255), outline=(150, 87, 53, 255))
    if not p or not os.path.exists(p): d.text((x + 30, y + 40), "none", fill=(200, 160, 130, 255), font=f); return
    t = Image.open(p).convert("RGBA"); k = max(1, (S - 8) // 32 * 32 // max(t.size)) if max(t.size) <= 32 else max(1, (S - 8) // max(t.size))
    t2 = t.resize((t.width * k, t.height * k), Image.NEAREST)
    im.alpha_composite(t2, (x + (S - t2.width) // 2, y + (S - t2.height) // 2))
    im.alpha_composite(t, (x + S + 4, y + S - t.height))
for i, (b, a) in enumerate(pairs):
    x = 10 + (i % cols) * cw; y = 50 + (i // cols) * ch
    put(b, x, y); put(a, x + S + 40, y)
    d.text((x, y + S + 4), (a or b).split("out_blocks/")[-1].split("out_")[-1].split("/",1)[-1][:-4] if "out_blocks/" in (a or "") else os.path.basename(a or b)[:-4], fill=(228, 218, 207, 255), font=f)
im.save(out)
