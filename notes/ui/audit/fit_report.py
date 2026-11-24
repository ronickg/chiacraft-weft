#!/usr/bin/env python3
"""The text-fit list from the `--ui-audit` runs (`fit_all.txt`): every fault the engine logged
(`ui-audit: screen | fault | path | "words" | by | lang | scale | window`), one line per screen,
fault and widget, with the languages, GUI scales and windows it shows in, worst first. Also the
screens the sweep couldn't open.

    python3 fit_report.py run1/run.log run2/run.log ... [--md out.md]
"""
import re
import sys
from collections import defaultdict

LINE = re.compile(r'ui-audit: (.*?) \| (.*?) \| (.*?) \| "(.*)" \| ([-\d.]+) \| (\S+) \| ([\d.]+) \| (\d+x\d+)')
NOTE = re.compile(r"peek: (\S+) (did not open|.*)")
ANSI = re.compile(r"\x1b\[[0-9;]*m")


def main(argv):
    md = None
    if "--md" in argv:
        i = argv.index("--md")
        md = argv[i + 1]
        argv = argv[:i] + argv[i + 2:]
    faults = defaultdict(lambda: {"by": 0.0, "where": set()})
    unopened = set()
    for path in argv:
        for raw in open(path, errors="replace"):
            line = ANSI.sub("", raw)
            m = LINE.search(line)
            if m:
                screen, what, wpath, text, by, lang, scale, window = m.groups()
                f = faults[(screen, what, wpath, text)]
                f["by"] = max(f["by"], abs(float(by)) if what != "squeezed" else 1 - float(by))
                f["where"].add((lang, float(scale), window))
                continue
            n = NOTE.search(line)
            if n:
                unopened.add(line.split("peek: ", 1)[1].strip())
    out = []
    screens = defaultdict(list)
    for key, f in faults.items():
        screens[key[0]].append((key, f))
    out.append(f"# Text-fit check: {len(faults)} faults on {len(screens)} screens\n")
    for screen in sorted(screens, key=lambda s: -len(screens[s])):
        out.append(f"## {screen} ({len(screens[screen])})\n")
        for (s, what, wpath, text), f in sorted(screens[screen], key=lambda kf: (kf[0][1], -kf[1]["by"])):
            langs = sorted({w[0] for w in f["where"]})
            scales = sorted({w[1] for w in f["where"]})
            wins = sorted({w[2] for w in f["where"]}, key=lambda x: int(x.split("x")[0]))
            by = f"{f['by'] * 100:.0f}% smaller" if what == "squeezed" else f"{f['by']:.1f} px"
            out.append(
                f"- **{what}** \"{text[:60]}\" ({by}) `{wpath[-90:]}` | {', '.join(langs)} | scale "
                f"{', '.join(f'{x:g}' for x in scales)} | {', '.join(wins)}"
            )
        out.append("")
    if unopened:
        out.append("## Not opened by the sweep\n")
        out += [f"- {u}" for u in sorted(unopened)]
    text = "\n".join(out)
    if md:
        open(md, "w").write(text + "\n")
    print(text)


if __name__ == "__main__":
    main(sys.argv[1:])
