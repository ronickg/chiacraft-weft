import re, os, glob
ROOT = os.environ.get("CHIACRAFT_ROOT", ".")
exec(open(os.path.join(os.path.dirname(__file__), "langload.py")).read())

pref = ("hud.", "command.", "screen.", "advancement.", "advancements.", "stat.", "season.", "tagged.", "levels.", "bed.",
        "menu.", "key.", "config.", "tab.", "portals.", "entity.", "item.", "block.", "species.", "family.", "packages.", "tooltip.", "recipe.")
alls = set()
for k in L: alls |= set(L[k])
mods = {"base": f"{ROOT}/mods/base/src", "animals": f"{ROOT}/mods/animals/src", "portals": f"{ROOT}/mods/portals/src", "packages": f"{ROOT}/mods/packages/src"}
for m, d in mods.items():
    miss = {}
    for f in glob.glob(d + "/**/*.rs", recursive=True):
        src = open(f, encoding="utf-8").read()
        for i, line in enumerate(src.split("\n"), 1):
            if line.strip().startswith("//"):
                continue
            for s in re.findall(r'"([a-z][a-z0-9_]*\.[a-z0-9_.{}]+)"', line):
                if not s.startswith(pref):
                    continue
                if "{" in s:
                    continue
                if s.endswith("."):
                    # prefix used with format
                    continue
                if s not in alls and not any(k.startswith(s + ".") for k in alls):
                    miss.setdefault(s, f"{os.path.relpath(f, ROOT)}:{i}")
    print(m, len(miss))
    for k, v in sorted(miss.items()):
        print("   ", k, v)
