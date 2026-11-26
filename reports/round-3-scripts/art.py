import os, re, glob, hashlib
ROOT = os.environ.get("CHIACRAFT_ROOT", ".")
T = {m: f"{ROOT}/mods/{m}/assets/textures" for m in ("base", "animals", "portals")}

def h(p):
    return hashlib.md5(open(p, "rb").read()).hexdigest()

# duplicates and tiny files
for m, d in T.items():
    files = [f for f in glob.glob(d + "/**/*.png", recursive=True)]
    by = {}
    for f in files:
        by.setdefault(h(f), []).append(os.path.relpath(f, d))
    print(f"\n== {m}: {len(files)} png")
    for k, v in by.items():
        if len(v) > 1:
            print("  dup:", v)
    small = sorted((os.path.getsize(f), os.path.relpath(f, d)) for f in files)
    print("  smallest:", small[:12])

# referenced block textures in base code
src = ""
for f in glob.glob(f"{ROOT}/mods/base/src/**/*.rs", recursive=True):
    src += open(f).read()
refs = set()
for call in re.findall(r'\.(?:texture|textures|faces|item_texture)\(([^)]*)\)', src):
    for s in re.findall(r'"([a-z0-9_]+)"', call):
        refs.add(s)
blockdir = T["base"] + "/block"
have = {os.path.splitext(f)[0] for f in os.listdir(blockdir)}
itemdir = T["base"] + "/item"
haveitem = {os.path.splitext(f)[0] for f in os.listdir(itemdir)}
print("\n== base literal texture refs missing in block/ and item/:", sorted(r for r in refs if r not in have and r not in haveitem))

exec(open(os.path.join(os.path.dirname(__file__), "names.py")).read())
print("== base items with no item/ png (and no same-name block png):", [n for n in ITEMS if n not in haveitem and n not in have])
print("== base blocks with no same-name block png (may use other textures):", [n for n in BLOCKS if n not in have])
