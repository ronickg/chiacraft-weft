"""Renders the viewer from a few cameras: python3 shots.py out.png "x,y,z@ty" ..."""
import subprocess, sys
from PIL import Image
out, views = sys.argv[1], sys.argv[2:]
h = open("viewer.html").read()
ims = []
for k, v in enumerate(views):
    cam, ty = v.split("@") if "@" in v else (v, "44")
    d = h.replace('"3/4": [90, 90, 140]', f'"3/4": [{cam}]').replace("controls.target.set(0, 44, 0);", f"controls.target.set(0, {ty}, 0);")
    open("debug.html", "w").write(d)
    subprocess.run(["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", "--headless=new", "--use-angle=swiftshader",
                    "--enable-unsafe-swiftshader", "--hide-scrollbars", "--window-size=1100,900", "--virtual-time-budget=6000",
                    f"--screenshot=/tmp/_s{k}.png", "http://localhost:8765/debug.html"], capture_output=True)
    ims.append(Image.open(f"/tmp/_s{k}.png").crop((25, 228, 800, 780)))
cols = 2
c = Image.new("RGB", (775 * cols, 552 * ((len(ims) + 1) // cols)))
for k, im in enumerate(ims):
    c.paste(im, ((k % cols) * 775, (k // cols) * 552))
c.save(out)
