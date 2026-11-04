"""Packs the base and animals mods as they are in the source tree (their built libraries and
assets) into run-tex/mods, for test runs: python3 notes/texture-test/pack_test.py"""
import os
import zipfile

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")
for mod in ("base", "animals"):
    lib = os.path.join(ROOT, "target", "release", f"lib{mod}.dylib")
    assets = os.path.join(ROOT, "mods", mod, "assets")
    out = os.path.join(ROOT, "run-tex", "mods", f"{mod}.zip")
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        z.write(lib, f"lib{mod}.dylib")
        for dirpath, _, files in os.walk(assets):
            for f in files:
                p = os.path.join(dirpath, f)
                z.write(p, os.path.relpath(p, os.path.join(ROOT, "mods", mod)))
    print(out, os.path.getsize(out) // 1024, "KB")
