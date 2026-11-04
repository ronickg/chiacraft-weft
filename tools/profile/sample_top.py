#!/usr/bin/env python3
"""Where the server's time goes, from a macOS `sample` file (tools/profile/sample_windows.sh).

Sums the call trees of the server's threads (`server` and its pool, `server N`) and prints the
functions with the most samples: inclusive (it or what it calls) and self (it alone), as a share
of the server threads' busy samples and in CPU ms per tick (one sample a millisecond over a window
of `--secs`, 20 ticks a second; summed over the pool's threads, so it can pass the tick's wall time).

    python3 tools/profile/sample_top.py run.sample.txt [--secs 28] [--top 40] [--threads all]
"""
import argparse
import re
from collections import Counter, defaultdict

NODE = re.compile(r"^(?P<indent>[ +!:|]*)(?P<count>\d+) (?P<name>.+?)(?:  \(in (?P<lib>[^)]+)\))?(?: \+ \S+)?(?:  \[[^\]]*\])?(?:  \S+:\d+)?\s*$")
THREAD = re.compile(r"^\s*(?P<count>\d+) Thread_\d+(?:\s+DispatchQueue_\S+)?(?::\s*(?P<name>.*))?$")
IDLE = ("__psynch_cvwait", "__ulock_wait", "__semwait_signal", "mach_msg2_trap", "mach_msg_trap", "__workq_kernreturn", "kevent", "__select", "__psynch_mutexwait")


# The pool's threads looking for work (rayon spins a little before it sleeps): not work.
SPIN = ("wait_until_cold", "crossbeam_deque", "rayon_core::sleep", "WorkerThread::find_work", "WorkerThread::take_local_job", "sched_yield", "swtch_pri", "thread_switch")


def waiting(name):
    return any(name.startswith(w) for w in IDLE) or any(w in name for w in SPIN)


def short(name):
    # Rust symbols: drop the hash and generic noise, keep the path.
    name = re.sub(r"::h[0-9a-f]{16}$", "", name)
    name = re.sub(r"_\$LT\$|\$GT\$|\$u20\$|\$C\$|\$RF\$", "", name)
    return name[:140]


def parse(path, want):
    threads = []  # (name, [(depth, count, name)])
    cur = None
    in_tree = False
    for line in open(path, errors="replace"):
        if line.startswith("Call graph:"):
            in_tree = True
            continue
        if in_tree and (line.startswith("Total number in stack") or line.startswith("Sort by top of stack")):
            break
        if not in_tree:
            continue
        m = THREAD.match(line)
        if m:
            name = (m.group("name") or "").strip()
            cur = (name, int(m.group("count")), [])
            threads.append(cur)
            continue
        if cur is None:
            continue
        m = NODE.match(line.rstrip("\n"))
        if not m:
            continue
        depth = len(m.group("indent"))
        cur[2].append((depth, int(m.group("count")), short(m.group("name").strip()), m.group("lib") or ""))
    return [t for t in threads if want(t[0])]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("file")
    ap.add_argument("--secs", type=float, default=28.0, help="the window sampled (sample_windows.sh: 28)")
    ap.add_argument("--top", type=int, default=40)
    ap.add_argument("--threads", default="server", help="'server' (server and its pool) or 'all'")
    a = ap.parse_args()
    want = (lambda n: True) if a.threads == "all" else (lambda n: n == "server" or n.startswith("server "))
    threads = parse(a.file, want)
    incl, self_ = Counter(), Counter()
    total = idle = 0
    print("threads:")
    for name, count, nodes in threads:
        total += count
        print(f"  {count:7d}  {name or '(unnamed)'}")
        # Self time: a node's count less its children's. Inclusive: each busy self sample also
        # counts for every function on its stack (once each, so recursion counts once).
        stack = []
        for i, (d, c, n, lib) in enumerate(nodes):
            while stack and stack[-1][0] >= d:
                stack.pop()
            stack.append((d, n))
            j = i + 1
            while j < len(nodes) and nodes[j][0] > d:
                j += 1
            sub = nodes[i + 1 : j]
            kids = sum(x[1] for x in sub if x[0] == min(y[0] for y in sub)) if sub else 0
            s = c - kids
            if s <= 0:
                continue
            self_[n] += s
            if waiting(n):
                idle += s
                continue
            for name in {x[1] for x in stack}:
                incl[name] += s
    busy = max(total - idle, 1)
    print(f"samples: {total} on these threads, {idle} waiting (blocked, or the pool looking for work), {busy} busy")
    ticks = a.secs * 20
    print(f"CPU ms per tick on these threads: {busy / ticks:.2f} busy (of 50 ms a tick)")
    unit = ", CPU ms per tick"
    ms = lambda c: f"  {c / ticks:6.2f}"
    print(f"\ntop self (share of busy{unit}):")
    for n, c in [x for x in self_.most_common() if not waiting(x[0])][: a.top]:
        print(f"  {100 * c / busy:5.1f}%{ms(c)}  {n}")
    print(f"\ntop inclusive (share of busy{unit}):")
    for n, c in incl.most_common(a.top * 2)[: a.top]:
        print(f"  {100 * c / busy:5.1f}%{ms(c)}  {n}")


if __name__ == "__main__":
    main()
