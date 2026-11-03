#!/usr/bin/env python3
"""Writes the project overview from `pieces.json`: `notes/OVERVIEW.md` (to read in the repo) and
`notes/overview/index.html` (the visual board, also published as the "Chiacraft Map" artifact).

    python3 notes/overview/build.py

Edit `pieces.json` (a piece's parts, their status, what's missing), then run this. Statuses are
kept by hand there; the numbers live in the generated sheets (`mods/base/vitals/SHEET.md`,
`mods/animals/SPECIES.md`), how gameplay connects in `notes/SYSTEMS.md`."""

import json
from pathlib import Path

HERE = Path(__file__).parent
NOTES = HERE.parent
MARK = {"built": "■", "partial": "◧", "missing": "□", "later": "◌"}


def counts(parts):
    c = {s: 0 for s in MARK}
    for p in parts:
        c[p["status"]] += 1
    return c


def markdown(data):
    every = [p for piece in data["pieces"] for p in piece["parts"]]
    c = counts(every)
    out = [
        "# chiacraft: the whole project, piece by piece",
        "",
        "Written by `notes/overview/build.py` from `notes/overview/pieces.json`; edit that, then run the script. "
        "The visual board is `notes/overview/index.html`. How the gameplay systems connect: `notes/SYSTEMS.md`.",
        "",
        f"**The goal.** {data['goal']}",
        "",
        " · ".join(f"{MARK[s]} {data['statuses'][s].lower()}: {c[s]}" for s in MARK),
        "",
        "## Core next",
        "",
        "What's missing that the rest leans on (marked `core` in the data).",
        "",
    ]
    for piece in data["pieces"]:
        for p in piece["parts"]:
            if p.get("core") and p["status"] != "built":
                out.append(f"- **{piece['name']}: {p['name']}**. {p.get('note', '')}".rstrip())
    out.append("")
    for piece in data["pieces"]:
        pc = counts(piece["parts"])
        out += [
            f"## {piece['name']}",
            "",
            f"{piece['what']} {pc['built']} of {len(piece['parts'])} parts built.",
            "",
            "| | Part | Notes |",
            "|---|---|---|",
        ]
        for p in piece["parts"]:
            docs = ", ".join(f"`{d}`" for d in p.get("docs", []))
            code = ", ".join(f"`{c}`" for c in p.get("code", []))
            note = p.get("note", "") + (f" Code: {code}." if code else "") + (f" Docs: {docs}." if docs else "")
            core = " (core)" if p.get("core") and p["status"] != "built" else ""
            out.append(f"| {MARK[p['status']]} | {p['name']}{core} | {note.strip()} |")
        out += [
            "",
            "Code: " + ", ".join(f"`{c}`" for c in piece["code"]) + ". Docs: " + ", ".join(f"`{d}`" for d in piece["docs"]) + ".",
            "",
        ]
    return "\n".join(out)


def missing_paths(data):
    """Code and docs paths named in the data that aren't in the repo (moved or renamed)."""
    root = NOTES.parent
    named = [x for piece in data["pieces"] for x in piece["code"] + piece["docs"]]
    named += [d for piece in data["pieces"] for p in piece["parts"] for d in p.get("docs", []) + p.get("code", [])]
    return sorted({x for x in named if not (root / x).exists()})


def main():
    data = json.loads((HERE / "pieces.json").read_text())
    gone = missing_paths(data)
    if gone:
        raise SystemExit("pieces.json names paths that don't exist (moved or renamed?):\n  " + "\n  ".join(gone))
    (NOTES / "OVERVIEW.md").write_text(markdown(data))
    page = (HERE / "template.html").read_text().replace("/*PIECES*/null", json.dumps(data, ensure_ascii=False))
    (HERE / "index.html").write_text(page)
    print("wrote notes/OVERVIEW.md and notes/overview/index.html")


if __name__ == "__main__":
    main()
