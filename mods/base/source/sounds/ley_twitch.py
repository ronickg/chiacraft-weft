"""The dowsing rod's tick (base:ley.twitch): a soft glassy chime, synthesised (no recording).
Bell-like inharmonic partials over a short decay, mono 44.1 kHz 16-bit WAV.
    python3 ley_twitch.py <out.wav>"""
import math, struct, sys, wave
RATE, LEN = 44100, 0.42
partials = [(1.0, 1.0, 9.0), (2.76, 0.42, 14.0), (5.40, 0.16, 22.0), (0.5, 0.18, 6.0)]
f0 = 1046.5  # C6
n = int(RATE * LEN)
out = []
for i in range(n):
    t = i / RATE
    attack = min(1.0, t / 0.004)
    v = sum(a * math.exp(-d * t) * math.sin(2 * math.pi * f0 * m * t + 0.7 * m) for m, a, d in partials)
    # A faint shimmer: slow tremolo.
    v *= attack * (0.92 + 0.08 * math.sin(2 * math.pi * 7.0 * t))
    out.append(v)
peak = max(abs(v) for v in out)
fade = int(RATE * 0.03)
for i in range(fade):
    out[n - 1 - i] *= i / fade
with wave.open(sys.argv[1], "wb") as w:
    w.setnchannels(1); w.setsampwidth(2); w.setframerate(RATE)
    w.writeframes(b"".join(struct.pack("<h", int(v / peak * 0.5 * 32767)) for v in out))
