import random
from PIL import Image, ImageDraw
O="out_touch/"
out=Image.new("RGBA",(2*(6*48)+60, 4*48+150),(30,24,20,255)); dr=ImageDraw.Draw(out)
rng=random.Random(5)
for k,n in enumerate(("raspberry_leaves_berried","snowberry_leaves_berried")):
    v=[Image.open(f"{O}{n}/{i}.png").convert("RGBA") for i in (1,2,3)]
    x0=20+k*(6*48+20)
    dr.text((x0,6),n.replace("_leaves_berried","")+": 6 x 4 blocks, a random variant each",fill=(240,220,160))
    for j in range(4):
        for i in range(6):
            out.alpha_composite(rng.choice(v).resize((48,48),Image.NEAREST),(x0+i*48,22+j*48))
    dr.text((x0,4*48+30),"the three variants",fill=(240,220,160))
    for i,t in enumerate(v):
        out.alpha_composite(t.resize((96,96),Image.NEAREST),(x0+i*100,4*48+46))
out.save("sheet_touch_bushes.png")
