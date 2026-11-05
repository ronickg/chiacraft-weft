#!/usr/bin/env python3
"""Research round 18: server and bot stats lines -> one row per bot step (the last 60% of each hold).
usage: parse.py <run dir with server.log bots.out meta.txt ps.log>"""
import re, sys, statistics as st
d = sys.argv[1]
strip = lambda l: re.sub(r"\x1b\[[0-9;]*m", "", l)
srv = [strip(l) for l in open(f"{d}/server.log") if "server stats" in l]
bots = [strip(l) for l in open(f"{d}/bots.out") if "bots stats" in l or "bots step" in l]
ps = [l for l in open(f"{d}/ps.log")] if __import__("os").path.exists(f"{d}/ps.log") else []
def f(rx, l, default=0.0):
    m = re.search(rx, l)
    return float(m.group(1)) if m else default
# Bot steps: their start times (bot clock), and the rows of bot stats after each.
steps, rows = [], []
for l in bots:
    t = f(r"^\s*([\d.]+)", l)
    if "bots step" in l:
        steps.append((int(f(r"step: (\d+) bots", l)), t))
    else:
        rows.append((t, l))
# Server clock starts ~10 s before the bots: align by the first stats line with players > 0.
first_srv = next((f(r"^\s*([\d.]+)", l) for l in srv if f(r"players (\d+)", l) > 0), 0)
first_bot = next((t for t, l in rows if f(r"live (\d+)", l) > 0), 0)
off = first_srv - first_bot
print("| bots wanted | live | failed so far | tick avg ms | tick p99 ms | max ms | ticks/s | sync ms | regions ms | server cores | top threads | rss MB | to clients KB/s | KB/s per bot (server, raw) | wire KB/s per bot | join p50/p99 s | view p50/p99 s | ping p50/p99 ms |")
print("|" + "---|" * 18)
for i, (n, t0) in enumerate(steps):
    t1 = steps[i + 1][1] if i + 1 < len(steps) else 1e9
    span = min(t1, max(t for t, _ in rows) if rows else t0) - t0
    lo = t0 + 0.4 * span
    br = [l for t, l in rows if lo <= t < t1]
    ball = [l for t, l in rows if t0 <= t < t1 + 5]
    sr = [l for l in srv if lo + off <= f(r"^\s*([\d.]+)", l) < t1 + off]
    if not br or not sr:
        continue
    avg = lambda rx, L: st.mean(f(rx, l) for l in L)
    live = avg(r"live (\d+)", br)
    out = avg(r"to clients [\d.]+ msgs/s ([\d.]+) KB/s", sr)
    wire0, wire1 = f(r"wire so far rx ([\d.]+) MB", br[0]), f(r"wire so far rx ([\d.]+) MB", br[-1])
    tspan = (len(br) - 1) * 5 or 5
    top = re.search(r"cpu [\d.]+ cores \(([^)]*)\)", sr[-1])
    def pick(name):
        vals = []
        for l in (br if name == 'ping ms' else ball):
            m = re.search(name + r" p50 ([\d.]+) p99 ([\d.]+)(?: max [\d.]+)? \(n (\d+)\)", l)
            if m and int(m.group(3)) > 0:
                vals.append((float(m.group(1)), float(m.group(2)), int(m.group(3))))
        if not vals:
            return "-"
        return f"{st.median(v[0] for v in vals):.2f}/{max(v[1] for v in vals):.2f} (n {sum(v[2] for v in vals)})"
    print(f"| {n} | {live:.0f} | {re.search(r'failed \[([^]]*)\]', br[-1]).group(1)[:60]} | {avg(r'tick avg ([\d.]+)', sr):.2f} | {max(f(r'p99 ([\d.]+) max', l) for l in sr):.2f} | {max(f(r'max ([\d.]+) ms,', l) for l in sr):.1f} | {avg(r'\(([\d.]+)/s\)', sr):.1f} | {avg(r'sync ([\d.]+)', sr):.2f} | {avg(r'regions ([\d.]+)', sr):.2f} | {avg(r'cpu ([\d.]+) cores', sr):.2f} | {top.group(1)[:70] if top else ''} | {avg(r'rss ([\d.]+) MB', sr):.0f} | {out:.0f} | {out / max(live, 1):.1f} | {(wire1 - wire0) * 1000 / tspan / max(live, 1):.1f} | {pick('join s')} | {pick('view s')} | {pick('ping ms')} |")
