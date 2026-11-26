import math
from PIL import Image, ImageDraw
B="../../../mods/base/assets/textures/block/"; O="out_touch/"
def front(d, radius):
    side=Image.open(d+"wheel_side.png").convert("RGBA"); hub=Image.open(d+"wheel_hub.png").convert("RGBA")
    im=Image.new("RGBA",(32,32)); p=im.load(); s=side.load(); h=hub.load(); hr=radius*0.33*2
    for y in range(32):
        for x in range(32):
            if abs(x-15.5)<hr and abs(y-15.5)<hr: p[x,y]=h[x,y]
            elif math.hypot(x-15.5,y-15.5)<radius*2: p[x,y]=s[x,y]
    return im
row=[]
for d in (B,O):
    for r in (4,6,8): row.append(front(d,r))
    row.append(Image.open(d+"wheel_tread.png").convert("RGBA"))
for n in ("raspberry_leaves_berried","snowberry_leaves_berried"):
    for d in (B,O):
        t=Image.open(d+n+".png").convert("RGBA"); g=Image.new("RGBA",(96,96))
        for i in range(3):
            for j in range(3): g.alpha_composite(t,(i*32,j*32))
        row.append(g)
out=Image.new("RGBA",(4*205+10, 3*215+30),(30,24,20,255)); dr=ImageDraw.Draw(out)
labels=["now: small wheel","now: wheel","now: large wheel","now: tread","new: small wheel","new: wheel","new: large wheel","new: tread","raspberry now (3x3)","raspberry new","snowberry now","snowberry new"]
for i,im in enumerate(row):
    x=(i%4)*205+10; y=(i//4)*215+30
    out.alpha_composite(im.resize((192,192),Image.NEAREST),(x,y)); dr.text((x,y-14),labels[i],fill=(240,220,160))
out.save("sheet_touch_look.png")
