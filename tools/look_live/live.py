#!/usr/bin/env python3
"""Live look tuning on the PC (Planner, 11-23): one game stays open in the testkit's live mode
(`live <file>` in a script, `mods/testkit/src/script.rs`) and runs each line written to its
command file. This drives it from the Mac over ssh. It never starts a game: Tester 2 does
(`live.txt` beside this file is the script it runs, with `--shader-pack chiacraft_live`).

  live.py cmd 'hour 18' 'set_config base:look_glow_fill 0' 'shot g18_f0.png'
        appends the lines to the command file, waits for their results and prints them
  live.py pack
        composes this worktree's shader pack (mods/base/shaderpack, with look.wgsl and MIST_TOP in
        front of each file, as mods/base/src/shaderpack.rs writes it), copies it to the PC and
        reloads it in the game (`reload_pack`)
  live.py sheet <run id> <out.jpg> <hour>:<shot.png> ... [--set normal|twilight|indirect|direct]
        fetches the shots from the run's folder and puts each beside Bliss's at that hour
        (18.5 is 18:30), with each picture's region means (sky, far, near, all)
  live.py out
        the command file's results so far

Paths on the PC (agreed with Tester 2): command file ~/looktest/live/cmd.txt, results
cmd.txt.out beside it, the composed pack in ~/looktest/live/pack/, shots in the run's own
folder ~/looktest/t2_<run id>/ (the runner's --shot-dir).
"""
import os
import re
import shlex
import subprocess
import sys
import time

PC = "ronaldg@192.168.1.13"
LIVE = "/home/ronaldg/looktest/live"
CMD = f"{LIVE}/cmd.txt"
OUT = f"{CMD}.out"
PACK = f"{LIVE}/pack"
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
BLISS = os.path.expanduser("~/Coding/chiacraft-archive/images/tester-2/2026-11-22/bliss_ctl")
SETS = {"normal": "normal", "twilight": "twilight", "indirect": "debug_INDIRECT", "direct": "debug_DIRECT"}
FILES = ["world.wgsl", "sky.wgsl", "model.wgsl", "entity.wgsl", "particle.wgsl", "weather.wgsl", "post.wgsl", "pathtrace.wgsl"]


def ssh(cmd, inp=None):
    r = subprocess.run(["ssh", "-o", "BatchMode=yes", PC, "bash -c " + shlex.quote(cmd)], input=inp, text=True, capture_output=True)
    if r.returncode != 0:
        sys.exit(f"ssh failed: {r.stderr.strip()}")
    return r.stdout


def results_after(size):
    """The result file from byte `size` on."""
    return ssh(f"tail -c +{size + 1} {OUT} 2>/dev/null")


def out_size():
    return int(ssh(f"stat -c %s {OUT} 2>/dev/null || echo 0").strip() or 0)


def send(lines, wait=180.0):
    lines = [l for l in lines if l.split("#")[0].strip()]
    before = out_size()
    ssh(f"mkdir -p {LIVE} && cat >> {CMD}", "".join(l + "\n" for l in lines))
    want = len(lines)
    end = time.time() + wait
    text = ""
    while time.time() < end:
        time.sleep(1.5)
        text = results_after(before)
        done = sum(1 for l in text.splitlines() if l == "ok" or l.startswith("err") or l.startswith("quit") or l.startswith("idle"))
        if done >= want or "live mode ends" in text:
            break
    else:
        print(f"(timed out after {wait:.0f} s: is the live game running? `live.py out` shows its results)")
    print(text, end="")
    return text


def compose():
    look = open(os.path.join(ROOT, "mods/base/shaderpack/look.wgsl")).read()
    air = open(os.path.join(ROOT, "mods/base/vitals/src/air.rs")).read()
    top = float(re.search(r"pub const MIST_TOP: f32 = ([0-9.]+);", air).group(1))
    head = f"// From vitals::air (mods/base/src/shaderpack.rs).\nconst MIST_TOP: f32 = {top:.1};\n"
    look_rs = open(os.path.join(ROOT, "mods/base/src/look.rs")).read()
    options = re.search(r'pub const OPTIONS: &str = "([^"]*)"', look_rs, re.S)
    out = {f: head + look + "\n" + open(os.path.join(ROOT, "mods/base/shaderpack", f)).read() for f in FILES}
    if options:
        out["pack.cfg"] = (
            "name = chiacraft\ndescription = Sun and shadows, sky, haze and water\ntranslucent_depth = true\n"
            "sun_shadows = true\nwater_behind = true\nmerge_effects = true\noptions = " + re.sub(r"\\\n\s*", "", options.group(1)) + "\n"
        )
    else:
        print("(no OPTIONS in look.rs: pack.cfg left as the game wrote it)")
    return out


def pack():
    files = compose()
    ssh(f"mkdir -p {PACK}")
    for name, text in files.items():
        ssh(f"cat > {PACK}/{name}", text)
    print(f"{len(files)} files to {PACK}")
    send([f"reload_pack {PACK}", "wait 2"], wait=120)


def ref_for(hour, which):
    h = float(hour)
    tag = f"{int(h):02d}" + ("30" if abs(h - int(h) - 0.5) < 1e-6 else "")
    return os.path.join(BLISS, SETS[which], f"vista_h{tag}.png")


def means(img):
    import numpy as np
    a = np.asarray(img.convert("RGB")).astype(float)
    h = a.shape[0]
    band = lambda y0, y1: a[int(h * y0):int(h * y1)].reshape(-1, 3).mean(0).astype(int)
    return {"sky": band(0, 0.25), "far": band(0.45, 0.6), "near": band(0.75, 1.0), "all": band(0, 1.0)}


def sheet(run, out, pairs, which):
    from PIL import Image, ImageDraw
    local = os.path.expanduser(f"~/Coding/chiacraft-archive/images/tester-2/{time.strftime('%Y-%m-%d')}/runs/t2_{run}")
    os.makedirs(local, exist_ok=True)
    names = [p.split(":", 1)[1] for p in pairs]
    subprocess.run(["rsync", "-a", "-e", "ssh -o BatchMode=yes"] + [f"{PC}:looktest/t2_{run}/{n}" for n in names] + [local + "/"], check=True)
    W, H, T = 800, 450, 34
    rows = []
    for p in pairs:
        hour, name = p.split(":", 1)
        rows.append((f"Bliss {hour} ({which})", ref_for(hour, which)))
        rows.append((f"ours {name}", os.path.join(local, name)))
    # Two columns: Bliss | ours, one row per pair.
    n = len(pairs)
    canvas = Image.new("RGB", (W * 2, (H + T) * n), "black")
    d = ImageDraw.Draw(canvas)
    for i in range(n):
        for j in range(2):
            label, path = rows[2 * i + j]
            x, y = j * W, i * (H + T)
            if not os.path.exists(path):
                d.text((x + 6, y + 6), f"{label}: missing {path}", fill="red")
                continue
            im = Image.open(path)
            m = means(im)
            canvas.paste(im.convert("RGB").resize((W, H)), (x, y + T))
            txt = f"{label}   " + "  ".join(f"{k} {' '.join(map(str, v))}" for k, v in m.items())
            d.text((x + 6, y + 4), txt, fill="white")
            print(txt)
    canvas.save(out, quality=88)
    print(out)


def main():
    a = sys.argv[1:]
    if not a:
        sys.exit(__doc__)
    if a[0] == "cmd":
        send(a[1:])
    elif a[0] == "pack":
        pack()
    elif a[0] == "out":
        print(ssh(f"cat {OUT} 2>/dev/null"), end="")
    elif a[0] == "sheet":
        which = "normal"
        if "--set" in a:
            i = a.index("--set")
            which = a[i + 1]
            a = a[:i] + a[i + 2:]
        sheet(a[1], a[2], a[3:], which)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
