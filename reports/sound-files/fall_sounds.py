"""A falling tree's sounds (mods/base/src/fall.rs), made here so every part is ours or CC0:
- tree/creak1-2: the hinge giving as the tree starts to lean, synthesized: stick-slip pulses whose
  rate climbs and wavers, rung through a few wood-like resonances, swelling, ending on a crack.
- tree/fall1-2: the rush of the crown through the air: noise swept up in pitch and loudness,
  with base's own CC0 leaf rustles (dig/leaves1-3, Kenney) scattered through it.
- tree/thud1-2: the landing: Kenney's impactWood_heavy (CC0, Impact Sounds) an octave down, a low
  thump under it, and the crown's leaves crashing (dig/leaves).
Usage: python3 fall_sounds.py out_root (base's assets/sounds). Needs numpy and ffmpeg.
"""
import os, subprocess, sys
import numpy as np

SR = 44100
KENNEY = os.path.expanduser("~/Coding/chiacraft-archive/references/sound/kenney/impact-sounds/Audio")
rng = np.random.default_rng(11)


def decode(path):
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", path, "-ac", "1", "-ar", str(SR), "-f", "f32le", "-"],
                         capture_output=True, check=True).stdout
    return np.frombuffer(raw, dtype=np.float32).copy()


def encode(x, out):
    os.makedirs(os.path.dirname(out), exist_ok=True)
    subprocess.run(["ffmpeg", "-v", "error", "-y", "-f", "f32le", "-ar", str(SR), "-ac", "1", "-i", "-",
                    "-ac", "2", "-c:a", "vorbis", "-strict", "-2", "-b:a", "112k", out],
                   input=x.astype(np.float32).tobytes(), check=True)


def resonator(x, f, q):
    """A two-pole band-pass at f Hz."""
    w = 2 * np.pi * f / SR
    r = np.exp(-w / (2 * q))
    a1, a2 = -2 * r * np.cos(w), r * r
    y = np.zeros_like(x)
    y1 = y2 = 0.0
    g = 1 - r
    for i in range(len(x)):
        v = g * x[i] - a1 * y1 - a2 * y2
        y2, y1 = y1, v
        y[i] = v
    return y


def lowpass(x, f):
    a = np.exp(-2 * np.pi * f / SR)
    y = np.empty_like(x)
    s = 0.0
    for i in range(len(x)):
        s = (1 - a) * x[i] + a * s
        y[i] = s
    return y


def level(x, peak_db):
    x = x - np.mean(x)
    fade = int(0.03 * SR)
    x[-fade:] *= np.linspace(1, 0, fade)
    return x / (np.abs(x).max() + 1e-9) * 10 ** (peak_db / 20)


def pitch(x, factor):
    """Played `factor` times slower (lower)."""
    t = np.arange(0, len(x) - 1, 1 / factor)
    return np.interp(t, np.arange(len(x)), x)


def creak(seed, secs=1.15):
    r = np.random.default_rng(seed)
    n = int(secs * SR)
    x = np.zeros(n)
    t = 0.0
    while t < secs - 0.12:
        s = t / secs
        rate = 14 + 70 * s ** 1.5 + 12 * np.sin(2 * np.pi * 2.3 * t + seed)
        i = int(t * SR)
        x[i] += (0.4 + 0.6 * s) * (0.7 + 0.6 * r.random())
        t += (1 / rate) * (0.8 + 0.4 * r.random())
    body = resonator(x, 190, 9) + 0.7 * resonator(x, 430, 12) + 0.35 * resonator(x, 980, 14)
    swell = np.minimum(1, np.linspace(0.2, 1.4, n))
    body *= swell
    # The crack at the end: a light wood hit.
    crack = decode(f"{KENNEY}/impactWood_light_00{seed % 5}.ogg")[: int(0.15 * SR)]
    at = n - len(crack) - int(0.02 * SR)
    body[at:at + len(crack)] += crack * np.abs(body).max() * 1.2 / (np.abs(crack).max() + 1e-9)
    return level(body, -4.0)


def leaves():
    return [decode(os.path.join(ROOT, f"dig/leaves{i}.ogg")) for i in (1, 2, 3)]


def rush(seed, secs=2.6):
    r = np.random.default_rng(seed)
    n = int(secs * SR)
    noise = r.standard_normal(n)
    # Swept band: low and quiet at first, brighter and louder toward the ground.
    s = np.linspace(0, 1, n)
    lo = lowpass(noise, 600)
    hi = noise - lowpass(noise, 1800)
    x = (lo * (1 - s) + 0.5 * hi * s) * (0.15 + 0.85 * s ** 1.8)
    for leaf in leaves() * 3:
        at = int(r.uniform(0.2, 0.95) * (n - len(leaf)))
        x[at:at + len(leaf)] += leaf * (0.3 + 0.7 * at / n) * 0.6
    return level(x, -6.0)


def thud(seed, secs=1.3):
    r = np.random.default_rng(seed)
    n = int(secs * SR)
    x = np.zeros(n)
    hit = pitch(decode(f"{KENNEY}/impactWood_heavy_00{seed % 5}.ogg"), 2.0)[:n]
    x[: len(hit)] += hit / (np.abs(hit).max() + 1e-9)
    t = np.arange(n) / SR
    x += 0.9 * np.sin(2 * np.pi * (55 - 15 * t) * t) * np.exp(-t * 7)
    for leaf in leaves():
        at = int(r.uniform(0.0, 0.12) * SR)
        m = min(len(leaf), n - at)
        x[at:at + m] += leaf[:m] * 0.5
    return level(x, -1.5)


ROOT = sys.argv[1]
for i in (1, 2):
    encode(creak(i), os.path.join(ROOT, f"tree/creak{i}.ogg"))
    encode(rush(10 + i), os.path.join(ROOT, f"tree/fall{i}.ogg"))
    encode(thud(i), os.path.join(ROOT, f"tree/thud{i}.ogg"))
print("ok")
