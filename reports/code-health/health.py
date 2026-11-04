#!/usr/bin/env python3
"""Code health report for chiacraft: is the code getting bloated?

Read-only. Standard library only. Run from a checkout's root:

    python3 -I reports/code-health/health.py              # markdown to stdout
    python3 -I reports/code-health/health.py --json out.json
    python3 -I reports/code-health/health.py --since f213253 --upstream upstream/master

What it reports:
  1. Rust lines per crate and mod (code lines: not blank, not comment-only), and test lines apart.
  2. Growth since a commit, split into Weft's (what upstream merges brought) and ours.
  3. The biggest files and functions.
  4. Duplicate blocks: the same 8+ code lines (after trimming) in two places.
  5. Dependencies a crate lists but never names in its code.
  6. Debug leftovers: dbg!, todo!, unimplemented!, allow(dead_code), FIXME/XXX/HACK.
  7. Big checked-in files that are not code (generated sheets, images over 1 MB, lockfiles).

Thresholds (warnings, not failures) are at the top; CI can fail on --max-* flags.
"""

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from collections import defaultdict

BIG_FILE = 1500  # code lines
BIG_FN = 150  # code lines
DUP_WINDOW = 8  # identical trimmed code lines in a row
SKIP_DIRS = {"target", ".git", "node_modules", ".claude", "run", "fuzz/target"}


def git(*args):
    return subprocess.run(["git", *args], capture_output=True, text=True, check=False).stdout


def tracked(ext=None):
    files = [f for f in git("ls-files", "-z").split("\0") if f]
    return [f for f in files if (ext is None or f.endswith(ext)) and os.path.isfile(f)]


def unit_of(path):
    """The crate or mod a file belongs to: crates/<x>, mods/<x>, mods/<x>/<sub crate>, else top dir."""
    parts = path.split("/")
    if parts[0] in ("crates", "backends", "examples") and len(parts) > 2:
        return "/".join(parts[:2])
    if parts[0] == "mods" and len(parts) > 2:
        # Sub crates of a mod (mods/base/vitals, mods/animals/core, mods/base/api).
        if len(parts) > 3 and os.path.isfile(os.path.join(*parts[:3], "Cargo.toml")):
            return "/".join(parts[:3])
        return "/".join(parts[:2])
    return parts[0]


def is_test_path(path):
    return "/tests/" in path or path.endswith("_test.rs") or "/benches/" in path or "/fuzz/" in path


def code_lines(text):
    """Lines that are code: not blank, not // comments, not inside /* */ blocks."""
    out = []
    in_block = False
    for line in text.split("\n"):
        s = line.strip()
        if in_block:
            if "*/" in s:
                in_block = False
            continue
        if not s or s.startswith("//"):
            continue
        if s.startswith("/*"):
            if "*/" not in s:
                in_block = True
            continue
        out.append(s)
    return out


def split_tests(text):
    """(non-test text, test text): a `#[cfg(test)] mod ... {` block to its closing brace is test."""
    lines = text.split("\n")
    main, test = [], []
    i = 0
    while i < len(lines):
        if lines[i].strip() == "#[cfg(test)]" and i + 1 < len(lines) and re.match(r"\s*(pub\s+)?mod\s+\w+\s*\{", lines[i + 1]):
            depth = 0
            j = i + 1
            while j < len(lines):
                depth += lines[j].count("{") - lines[j].count("}")
                test.append(lines[j])
                j += 1
                if depth <= 0:
                    break
            i = j
            continue
        main.append(lines[i])
        i += 1
    return "\n".join(main), "\n".join(test)


FN_RE = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?(const\s+)?(async\s+)?(unsafe\s+)?(extern\s+\"C\"\s+)?fn\s+(\w+)")


def functions(path, text, bodies=None):
    """(name, start line, code lines) for each fn, by brace counting from its signature. With
    `bodies`, also records each body's normalized hash (names and comments kept out)."""
    lines = text.split("\n")
    out = []
    for i, line in enumerate(lines):
        m = FN_RE.match(line)
        if not m:
            continue
        depth, started, n = 0, False, 0
        for j in range(i, min(len(lines), i + 5000)):
            l = lines[j]
            # Strip strings and chars roughly so braces in them don't count.
            l2 = re.sub(r'"(\\.|[^"\\])*"', '""', l)
            l2 = re.sub(r"'(\\.|[^'\\])'", "''", l2)
            l2 = l2.split("//")[0]
            if not started and ";" in l2 and "{" not in l2:
                break  # a declaration (trait method) without a body
            depth += l2.count("{") - l2.count("}")
            if "{" in l2:
                started = True
            if l.strip() and not l.strip().startswith("//"):
                n += 1
            if started and depth <= 0:
                out.append((m.group(7), i + 1, n))
                if bodies is not None and n >= 3:
                    body = [l.strip() for l in lines[i + 1:j + 1] if l.strip() and not l.strip().startswith("//")]
                    sig = re.sub(r"\bfn\s+\w+", "fn _", lines[i].strip())
                    h = hashlib.blake2b(("\n".join([sig] + body)).encode(), digest_size=10).hexdigest()
                    bodies[h].append((path, i + 1, m.group(7), n))
                break
    return out


def numstat(a, b, pathspec="*.rs"):
    """Lines added and removed per unit between two commits, Rust files only."""
    added, removed = defaultdict(int), defaultdict(int)
    for line in git("diff", "--numstat", a, b, "--", pathspec).splitlines():
        parts = line.split("\t")
        if len(parts) != 3 or parts[0] == "-":
            continue
        path = parts[2]
        if " => " in path:  # renames: {a => b}
            path = re.sub(r"\{[^}]*=> ([^}]*)\}", r"\1", path).replace("//", "/")
        u = unit_of(path)
        added[u] += int(parts[0])
        removed[u] += int(parts[1])
    return added, removed


def cargo_deps(toml_text):
    """Dependency names in [dependencies] (not dev or build), as Rust names."""
    deps, section = [], None
    for line in toml_text.split("\n"):
        s = line.strip()
        if s.startswith("["):
            section = s.strip("[]").strip()
            continue
        if section in ("dependencies",) or (section and section.startswith("target.") and section.endswith(".dependencies")):
            m = re.match(r"^([A-Za-z0-9_\-]+)\s*(\.workspace)?\s*=", s)
            if m:
                name = m.group(1)
                pkg = re.search(r'package\s*=\s*"([^"]+)"', s)
                deps.append((name, pkg.group(1) if pkg else name))
        m = re.match(r"^\[dependencies\.([A-Za-z0-9_\-]+)\]", s)
        if m:
            deps.append((m.group(1), m.group(1)))
    return deps


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--since", default="f213253", help="the fork point (default: Weft at the fork)")
    ap.add_argument("--upstream", default=None, help="the newest Weft commit merged in (default: merge-base with upstream/master)")
    ap.add_argument("--json", help="also write the numbers as JSON here")
    ap.add_argument("--top", type=int, default=15)
    ap.add_argument("--max-dup-blocks", type=int, default=None, help="exit 1 if more duplicate blocks than this")
    ap.add_argument("--max-big-fns", type=int, default=None, help="exit 1 if more functions over BIG_FN than this")
    args = ap.parse_args()

    rs = [f for f in tracked(".rs") if not any(f.startswith(d + "/") or f"/{d}/" in f for d in SKIP_DIRS)]
    per_unit = defaultdict(lambda: {"code": 0, "tests": 0, "files": 0})
    files_info, fns, windows = [], [], defaultdict(list)
    bodies = defaultdict(list)
    leftovers = defaultdict(list)
    texts = {}
    for f in rs:
        try:
            text = open(f, encoding="utf-8", errors="replace").read()
        except OSError:
            continue
        texts[f] = text
        main_text, test_text = split_tests(text)
        u = unit_of(f)
        main_code = code_lines(main_text)
        test_code = code_lines(test_text)
        if is_test_path(f):
            per_unit[u]["tests"] += len(main_code) + len(test_code)
        else:
            per_unit[u]["code"] += len(main_code)
            per_unit[u]["tests"] += len(test_code)
        per_unit[u]["files"] += 1
        files_info.append((len(main_code) + len(test_code), f))
        if not is_test_path(f):
            for name, line, n in functions(f, main_text, bodies):
                fns.append((n, f, line, name))
            # Duplicate windows: only lines with some substance, so `}` runs don't match.
            numbered = [(i + 1, l.strip()) for i, l in enumerate(main_text.split("\n"))]
            code = [(i, s) for i, s in numbered if s and not s.startswith("//") and len(s) > 3]
            for k in range(len(code) - DUP_WINDOW + 1):
                block = [s for _, s in code[k:k + DUP_WINDOW]]
                if sum(len(s) for s in block) < 200:
                    continue
                h = hashlib.blake2b("\n".join(block).encode(), digest_size=10).hexdigest()
                windows[h].append((f, code[k][0]))
        for i, line in enumerate(text.split("\n"), 1):
            s = line.strip()
            for pat, label in ((r"\bdbg!\(", "dbg!"), (r"\btodo!\(", "todo!"), (r"\bunimplemented!\(", "unimplemented!"),
                               (r"allow\(dead_code\)", "allow(dead_code)"), (r"\b(FIXME|XXX|HACK)\b", "FIXME/XXX/HACK"),
                               (r"^\s*println!\(", "println! (outside a bin)")):
                if re.search(pat, s):
                    if label.startswith("println") and ("/bin/" in f or "/examples/" in f or f.endswith("main.rs") or is_test_path(f) or "/build.rs" in f):
                        continue
                    leftovers[label].append(f"{f}:{i}")

    # Merge overlapping duplicate windows into blocks: one entry per pair of places.
    dup_pairs = defaultdict(int)
    for places in windows.values():
        uniq = sorted(set(places))
        if len(uniq) < 2:
            continue
        a = uniq[0]
        for b in uniq[1:]:
            if a[0] == b[0] and abs(a[1] - b[1]) < DUP_WINDOW:
                continue
            key = (a[0], (a[1] // 40), b[0], (b[1] // 40))
            dup_pairs[key] += 1
    dup_blocks = sorted(((n + DUP_WINDOW - 1, k) for k, n in dup_pairs.items()), reverse=True)

    # Unused dependencies, per crate: names never seen as `name::`, `use name`, `extern crate name`, or `name!`.
    unused = []
    for toml in tracked("Cargo.toml"):
        d = os.path.dirname(toml) or "."
        if any(seg in SKIP_DIRS for seg in d.split("/")):
            continue
        src = "\n".join(t for f, t in texts.items() if f.startswith((d + "/") if d != "." else ""))
        if d == ".":
            continue  # the workspace root lists shared versions only
        build = os.path.join(d, "build.rs")
        if os.path.isfile(build):
            src += open(build, encoding="utf-8", errors="replace").read()
        for name, pkg in cargo_deps(open(toml, encoding="utf-8").read()):
            rust = name.replace("-", "_")
            if not re.search(rf"\b{re.escape(rust)}\s*(::|!)|\buse\s+{re.escape(rust)}\b|extern\s+crate\s+{re.escape(rust)}\b|#\[{re.escape(rust)}::", src):
                unused.append(f"{toml}: {name}" + (f" (package {pkg})" if pkg != name else ""))

    # Growth: Weft's merges against ours.
    upstream = args.upstream
    if upstream is None:
        mb = git("merge-base", "HEAD", "upstream/master").strip()
        upstream = mb or None
    growth = {}
    if upstream:
        wa, wr = numstat(args.since, upstream)
        oa, orr = numstat(upstream, "HEAD")
        for u in set(wa) | set(oa) | set(wr) | set(orr):
            growth[u] = {"weft_added": wa[u], "weft_removed": wr[u], "ours_added": oa[u], "ours_removed": orr[u]}

    # Non-code files checked in that are big.
    big_other = []
    for f in tracked():
        if f.endswith(".rs"):
            continue
        try:
            size = os.path.getsize(f)
        except OSError:
            continue
        if size > 1_000_000 or (f.endswith((".md", ".json", ".html")) and size > 150_000):
            big_other.append((size, f))
    big_other.sort(reverse=True)

    big_fns = sorted(fns, reverse=True)
    out = []
    w = out.append
    w("# Code health\n")
    w(f"Commit `{git('rev-parse', '--short', 'HEAD').strip()}`. Code lines are lines that aren't blank or comments; tests are `#[cfg(test)]` modules and `tests/`, `benches/`, `fuzz/`.\n")
    w("## Lines per crate and mod\n")
    w("| unit | files | code | tests |")
    w("|---|---|---|---|")
    tc = tt = 0
    for u, v in sorted(per_unit.items(), key=lambda kv: -kv[1]["code"]):
        tc += v["code"]
        tt += v["tests"]
        w(f"| {u} | {v['files']} | {v['code']:,} | {v['tests']:,} |")
    w(f"| **all** | | **{tc:,}** | **{tt:,}** |\n")
    if growth:
        w(f"## Growth since `{args.since}`: Weft's merges (to `{upstream[:9]}`) and ours\n")
        w("Raw diff lines in `.rs` files (code, comments and blanks).\n")
        w("| unit | Weft + | Weft − | ours + | ours − | ours net |")
        w("|---|---|---|---|---|---|")
        tot = defaultdict(int)
        for u, g in sorted(growth.items(), key=lambda kv: -(kv[1]["ours_added"] - kv[1]["ours_removed"])):
            for k, v in g.items():
                tot[k] += v
            net = g["ours_added"] - g["ours_removed"]
            if g["weft_added"] + g["ours_added"] + g["weft_removed"] + g["ours_removed"] < 50:
                continue
            w(f"| {u} | {g['weft_added']:,} | {g['weft_removed']:,} | {g['ours_added']:,} | {g['ours_removed']:,} | {net:,} |")
        w(f"| **all** | **{tot['weft_added']:,}** | **{tot['weft_removed']:,}** | **{tot['ours_added']:,}** | **{tot['ours_removed']:,}** | **{tot['ours_added'] - tot['ours_removed']:,}** |\n")
    w(f"## Biggest files (over {BIG_FILE} code lines marked)\n")
    for n, f in sorted(files_info, reverse=True)[: args.top]:
        w(f"- {'**' if n > BIG_FILE else ''}{f}: {n:,}{'**' if n > BIG_FILE else ''}")
    w(f"\n## Biggest functions ({sum(1 for n, *_ in big_fns if n > BIG_FN)} over {BIG_FN} lines)\n")
    for n, f, line, name in big_fns[: args.top]:
        w(f"- `{name}` {f}:{line}: {n}")
    w(f"\n## Duplicate blocks ({len(dup_blocks)} places with {DUP_WINDOW}+ identical code lines)\n")
    for n, (fa, la, fb, lb) in dup_blocks[: args.top]:
        w(f"- ~{n} lines: {fa}:~{la * 40 or 1} and {fb}:~{lb * 40 or 1}")
    # Trait-method one-liners (`fn world(&self) -> &World { self.w }`) are noise: keep bodies of 5+
    # lines, or smaller ones copied under one name into 3+ places (hex, dist2).
    same_fns = sorted(
        (v for v in bodies.values()
         if len({p for p, *_ in v}) >= 2 and (v[0][3] >= 5 or (len({n for _, _, n, _ in v}) == 1 and len(v) >= 3))),
        key=lambda v: -(len(v) * v[0][3]))
    w(f"\n## Functions written more than once ({len(same_fns)} bodies, the same code in 2+ files)\n")
    w("Same signature (but the name) and same body, after trimming. Small helpers the block check misses.\n")
    for v in same_fns[: args.top * 2]:
        names = sorted({n for _, _, n, _ in v})
        w(f"- `{'/'.join(names)}` ×{len(v)}, {v[0][3]} lines: " + ", ".join(f"{p}:{l}" for p, l, _, _ in v[:6]) + (", …" if len(v) > 6 else ""))
    w(f"\n## Dependencies never named in their crate's code ({len(unused)})\n")
    w("Check each by hand: a crate can be used only through a macro, a feature or `build.rs`.\n")
    for u in unused:
        w(f"- {u}")
    w("\n## Debug leftovers\n")
    for label, places in sorted(leftovers.items()):
        w(f"- {label}: {len(places)}" + (f" ({', '.join(places[:6])}{', …' if len(places) > 6 else ''})" if places else ""))
    w("\n## Big checked-in files that aren't code\n")
    for size, f in big_other[: args.top]:
        w(f"- {f}: {size / 1e6:.2f} MB")
    print("\n".join(out))

    if args.json:
        with open(args.json, "w") as fh:
            json.dump({
                "units": per_unit, "growth": growth,
                "big_files": [(n, f) for n, f in sorted(files_info, reverse=True)[:50]],
                "big_functions": big_fns[:50], "duplicate_blocks": len(dup_blocks),
                "unused_deps": unused, "leftovers": {k: len(v) for k, v in leftovers.items()},
            }, fh, indent=1)

    bad = False
    if args.max_dup_blocks is not None and len(dup_blocks) > args.max_dup_blocks:
        print(f"\nmore duplicate blocks than allowed: {len(dup_blocks)} > {args.max_dup_blocks}", file=sys.stderr)
        bad = True
    nbig = sum(1 for n, *_ in big_fns if n > BIG_FN)
    if args.max_big_fns is not None and nbig > args.max_big_fns:
        print(f"\nmore big functions than allowed: {nbig} > {args.max_big_fns}", file=sys.stderr)
        bad = True
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
