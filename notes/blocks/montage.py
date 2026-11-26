"""Generated images side by side, small, to look over a run: python3 notes/blocks/montage.py out.png gen/a.png ..."""
import sys
from PIL import Image, ImageDraw
out, srcs = sys.argv[1], sys.argv[2:]
s, cols = 420, 3
im = Image.new("RGB", (cols * (s + 8), -(-len(srcs) // cols) * (s + 22)), (60, 60, 60))
d = ImageDraw.Draw(im)
for i, p in enumerate(srcs):
    x, y = (i % cols) * (s + 8), (i // cols) * (s + 22)
    im.paste(Image.open(p).convert("RGB").resize((s, s)), (x, y))
    d.text((x, y + s + 4), p.split("/")[-1], fill=(255, 255, 255))
im.save(out)
