"""A waystone waking (base:ley.wake): a low swell rising into a bright chime, synthesised (no
recording). Mono 44.1 kHz 16-bit WAV.
    python3 ley_wake.py <out.wav>"""
import math, struct, sys, wave
RATE, LEN = 44100, 2.2
n = int(RATE * LEN)
out = []
for i in range(n):
    t = i / RATE
    # The swell: a low fifth breathing in over 0.9 s and fading by 2 s.
    swell = min(1.0, t / 0.9) * math.exp(-max(0.0, t - 0.9) * 2.2)
    low = swell * (0.55 * math.sin(2 * math.pi * 130.8 * t) + 0.35 * math.sin(2 * math.pi * 196.0 * t + 0.4)) * (0.85 + 0.15 * math.sin(2 * math.pi * 5.0 * t))
    # The chime at 0.8 s: bell partials over C6 and G6.
    c = 0.0
    if t > 0.8:
        u = t - 0.8
        for f0, a in [(1046.5, 1.0), (1568.0, 0.6)]:
            for m, am, d in [(1.0, 1.0, 3.0), (2.76, 0.4, 5.0), (5.4, 0.15, 8.0)]:
                c += a * am * math.exp(-d * u) * math.sin(2 * math.pi * f0 * m * u)
        c *= min(1.0, u / 0.005)
    out.append(low + 0.45 * c)
peak = max(abs(v) for v in out)
fade = int(RATE * 0.05)
for i in range(fade):
    out[n - 1 - i] *= i / fade
with wave.open(sys.argv[1], "wb") as w:
    w.setnchannels(1); w.setsampwidth(2); w.setframerate(RATE)
    w.writeframes(b"".join(struct.pack("<h", int(v / peak * 0.6 * 32767)) for v in out))
