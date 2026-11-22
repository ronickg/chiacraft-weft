#!/usr/bin/env python3
"""PreToolUse guard for Bash commands in the chiacraft crew (session-crew skill, "Hard rules").

Blocks two things that went wrong on 2026-11-04:
1. Launching the game directly (`weft`, `target/*/weft`, a bare `cargo run` of the weft binary).
   One GPU hang takes Ronald's Mac down, so only the testers launch it, through their scripts
   in ~/Coding/chiacraft-archive/tester-tools/ (allowed), or over ssh on the PC (allowed).
2. `git commit -a` / `-am` / `--all` while a merge is open in this repository: it commits the
   conflict markers (it happened twice). Resolve, `git add` the files, then `git commit`.

Reads the hook JSON on stdin; prints a deny decision or nothing. Never fails the tool call on
its own errors.
"""
import json
import os
import re
import sys

ROOT = "/Users/ronald/Coding/chiacraft-weft"


def deny(reason):
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }))
    sys.exit(0)


def segments(cmd):
    # Split a shell line into simple commands on ; && || | and newlines (good enough for a guard).
    return [s.strip() for s in re.split(r"&&|\|\||[;|\n]", cmd) if s.strip()]


def launches_game(seg):
    words = seg.split()
    # Skip leading env assignments and wrappers like nice/env/time/exec.
    while words and (re.match(r"^[A-Za-z_][A-Za-z0-9_]*=", words[0])
                     or words[0] in ("nice", "env", "time", "exec", "nohup")
                     or (words[0].startswith("-") and len(words) > 1)):
        words = words[1:]
    if not words:
        return False
    head = words[0]
    if head == "weft" or re.search(r"(^|/)target/[^/]+/weft$", head) or head == "./weft":
        return True
    if head == "cargo" and len(words) > 1 and words[1] == "run":
        rest = words[2:]
        # `cargo run -p other` or `--bin other` runs a tool, not the game.
        for i, w in enumerate(rest):
            if w in ("-p", "--package", "--bin") and i + 1 < len(rest):
                return rest[i + 1] in ("weft", "weft_app", "app")
            if w.startswith(("--package=", "--bin=")):
                return w.split("=", 1)[1] in ("weft", "weft_app", "app")
        return True  # the workspace default binary is the game
    return False


def merge_open():
    git = os.path.join(ROOT, ".git")
    if os.path.exists(os.path.join(git, "MERGE_HEAD")):
        return True
    wt = os.path.join(git, "worktrees")
    if os.path.isdir(wt):
        for name in os.listdir(wt):
            if os.path.exists(os.path.join(wt, name, "MERGE_HEAD")):
                return True
    return False


def commit_all(seg):
    if not re.search(r"\bgit\b(\s+-C\s+\S+)?\s+commit\b", seg):
        return False
    after = seg.split("commit", 1)[1]
    return bool(re.search(r"(^|\s)(--all|-a\b|-[a-zA-Z]*a[a-zA-Z]*)(\s|$)", after))


def main():
    try:
        data = json.load(sys.stdin)
    except Exception:
        return
    if data.get("tool_name") != "Bash":
        return
    cmd = (data.get("tool_input") or {}).get("command") or ""
    # The testers' own run scripts and runs over ssh (the PC) are the sanctioned way to launch.
    sanctioned = "tester-tools/" in cmd or re.search(r"(^|\s)ssh\s", cmd)
    for seg in segments(cmd):
        if not sanctioned and launches_game(seg):
            deny("Crew rule: no session launches the game itself (one GPU hang takes Ronald's Mac "
                 "down). Send the build folder and script to Tester; testers launch through "
                 "~/Coding/chiacraft-archive/tester-tools/. See the session-crew skill, Hard rules.")
        if commit_all(seg) and merge_open():
            deny("A merge is open in this repository: `git commit -a` would commit the conflict "
                 "markers. Check `git diff --name-only --diff-filter=U` is empty, `git add` the "
                 "resolved files, then `git commit` without -a (or `git merge --abort`).")


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception:
        pass
