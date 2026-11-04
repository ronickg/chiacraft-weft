"""Prepare game sound files from CC0 sources: decode with ffmpeg, find hits, trim, level, encode .ogg.
Usage: python3 sfx.py plan.tsv out_root
plan.tsv rows: out_rel_path <TAB> source_file <TAB> mode (copy|hit:N) <TAB> max_seconds <TAB> peak_dbfs
  copy  : the whole source (trimmed of leading silence, cut to max_seconds)
  hit:N : the N-th strongest separate hit in a long recording (0-based), max_seconds long
"""
import sys, os, subprocess, numpy as np
SR = 44100

def decode(path):
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", path, "-ac", "1", "-ar", str(SR), "-f", "f32le", "-"],
                         capture_output=True, check=True).stdout
    return np.frombuffer(raw, dtype=np.float32).copy()

def env(x, win=441):
    k = np.ones(win) / win
    return np.sqrt(np.convolve(x * x, k, mode="same") + 1e-12)

def hits(x, max_s):
    e = env(x)
    gap = int(0.25 * SR)
    order = np.argsort(e)[::-1]
    picks = []
    for i in order:
        if e[i] < e.max() * 0.15:
            break
        if all(abs(i - p) > gap for p in picks):
            picks.append(i)
        if len(picks) >= 12:
            break
    picks.sort(key=lambda i: -e[i])
    out = []
    for p in picks:
        # start a little before the onset: walk back while the envelope is above 10% of the peak
        s = p
        while s > 0 and e[s] > e[p] * 0.1 and p - s < int(0.12 * SR):
            s -= 1
        out.append(max(0, s - int(0.005 * SR)))
    return out

def trim_lead(x, thresh_db=-45):
    t = 10 ** (thresh_db / 20) * max(np.abs(x).max(), 1e-9)
    idx = np.argmax(np.abs(x) > t)
    return x[max(0, idx - int(0.003 * SR)):]

def finish(x, max_s, peak_db):
    x = x[: int(max_s * SR)]
    fade = min(len(x), int(0.03 * SR))
    if fade > 0:
        x[-fade:] *= np.linspace(1, 0, fade)
    pk = np.abs(x).max()
    if pk > 0:
        x = x * (10 ** (peak_db / 20) / pk)
    return x.astype(np.float32)

def encode(x, out):
    os.makedirs(os.path.dirname(out), exist_ok=True)
    subprocess.run(["ffmpeg", "-v", "error", "-y", "-f", "f32le", "-ar", str(SR), "-ac", "1", "-i", "-",
                    "-ac", "2", "-c:a", "vorbis", "-strict", "-2", "-b:a", "112k", out], input=x.tobytes(), check=True)

plan, root = sys.argv[1], sys.argv[2]
for line in open(plan):
    if not line.strip() or line.startswith("#"):
        continue
    rel, src, mode, max_s, peak = line.rstrip("\n").split("\t")
    x = decode(os.path.expanduser(src))
    if mode == "copy":
        y = trim_lead(x)
    else:
        n = int(mode.split(":")[1])
        starts = hits(x, float(max_s))
        if n >= len(starts):
            print("not enough hits", rel, len(starts)); continue
        y = trim_lead(x[starts[n]:])
    y = finish(y, float(max_s), float(peak))
    out = os.path.join(root, rel)
    encode(y, out)
    print(f"{rel}\t{len(y)/SR:.2f}s\t{os.path.getsize(out)} B")
