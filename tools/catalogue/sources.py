"""Reading the game's sources for the catalogue: names, the progression rules and recipes, the
generated sheets (progression, vitals, crafting, species) and the part looks. Everything here
reads text files from a tree of the repository at one commit (`build.py` exports it); nothing
runs the game or cargo."""

import glob
import json
import os
import re

MODS = ["base", "animals", "portals", "packages"]


# ------------------------------------------------------------------ small parsers


def split_top(s, sep=","):
    """`s` split at `sep` outside brackets and strings."""
    out, depth, cur, i, q = [], 0, [], 0, False
    while i < len(s):
        c = s[i]
        if q:
            cur.append(c)
            if c == "\\" and i + 1 < len(s):
                cur.append(s[i + 1])
                i += 1
            elif c == '"':
                q = False
        elif c == '"':
            q = True
            cur.append(c)
        elif c in "([{":
            depth += 1
            cur.append(c)
        elif c in ")]}":
            depth -= 1
            cur.append(c)
        elif c == sep and depth == 0:
            out.append("".join(cur).strip())
            cur = []
        else:
            cur.append(c)
        i += 1
    if "".join(cur).strip():
        out.append("".join(cur).strip())
    return out


def calls(text, names):
    """Each call `name(...)` (or struct literal `name { ... }`) in `text`, as (name, inner text)."""
    out = []
    pat = re.compile(r"(?<![A-Za-z0-9_])(" + "|".join(map(re.escape, names)) + r")\s*(\(|\{)")
    pos = 0
    while True:
        m = pat.search(text, pos)
        if not m:
            break
        start = m.end() - 1
        depth, i, q = 0, start, False
        while i < len(text):
            c = text[i]
            if q:
                if c == "\\":
                    i += 1
                elif c == '"':
                    q = False
            elif c == '"':
                q = True
            elif c in "([{":
                depth += 1
            elif c in ")]}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        out.append((m.group(1), text[start + 1:i]))
        pos = i + 1
    return out


def strings(s):
    return [x.replace('\\"', '"') for x in re.findall(r'"((?:[^"\\]|\\.)*)"', s)]


def unstr(s):
    s = s.strip()
    if s.startswith('"') and s.endswith('"'):
        return s[1:-1].replace('\\"', '"')
    return s


def strip_comments(text):
    # Line comments only, outside strings (a `//` inside a string is kept).
    out = []
    for line in text.split("\n"):
        q, cut = False, None
        for i, c in enumerate(line):
            if c == '"' and (i == 0 or line[i - 1] != "\\"):
                q = not q
            elif not q and line.startswith("//", i):
                cut = i
                break
        out.append(line if cut is None else line[:cut])
    return "\n".join(out)


def tables(md):
    """Every markdown table in `md`: (heading path, header cells, rows of cells)."""
    out, heads, lines = [], [], md.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        h = re.match(r"^(#+)\s+(.*)", line)
        if h:
            lvl = len(h.group(1))
            heads = heads[:lvl - 1] + [h.group(2).strip()]
        if line.startswith("|") and i + 1 < len(lines) and re.match(r"^\|[\s:|-]+\|$", lines[i + 1].strip()):
            head = [c.strip() for c in line.strip().strip("|").split("|")]
            rows = []
            i += 2
            while i < len(lines) and lines[i].startswith("|"):
                rows.append([c.strip() for c in lines[i].strip().strip("|").split("|")])
                i += 1
            out.append((list(heads), head, rows))
            continue
        i += 1
    return out


# ------------------------------------------------------------------ names


def lang(root):
    """{key: English text} over every mod's en_us.lang, the engine's too."""
    out = {}
    files = [f"{root}/crates/engine/assets/lang/en_us.lang"] + [f"{root}/mods/{m}/assets/lang/en_us.lang" for m in MODS]
    for f in files:
        if not os.path.exists(f):
            continue
        for line in open(f, encoding="utf-8"):
            if "=" in line and not line.lstrip().startswith("#"):
                k, v = line.split("=", 1)
                out[k.strip()] = v.strip()
    return out


def universe(L):
    """Every named item and block (`content::universe`): {id: 'item'|'block'}; a block wins."""
    out = {}
    for k in L:
        m = re.fullmatch(r"(item|block)\.([a-z]+)\.([a-z0-9_]+)", k)
        if m and m.group(2) in MODS:
            i = f"{m.group(2)}:{m.group(3)}"
            if out.get(i) != "block":
                out[i] = m.group(1)
    return out


# ------------------------------------------------------------------ progression


def wildcard(pat, i):
    parts = pat.split("*")
    if len(parts) == 1:
        return pat == i
    if not i.startswith(parts[0]) or not i[len(parts[0]):].endswith(parts[-1]) or len(i) < len(parts[0]) + len(parts[-1]):
        return False
    rest = i[len(parts[0]):len(i) - len(parts[-1])]
    for mid in parts[1:-1]:
        j = rest.find(mid)
        if j < 0:
            return False
        rest = rest[j + len(mid):]
    return True


def rules(root):
    """`content::RULES`: [{pats, tier, path, kind, get, note}], first match wins."""
    text = strip_comments(open(f"{root}/mods/base/progression/src/content.rs").read())
    body = text[text.index("pub const RULES"):]
    body = body[body.index("= &[") + 4:]
    body = body[:body.index("\n];")]
    consts = {"Hand": "hand"}
    for m in re.finditer(r'pub const (\w+): Needs = Needs::Tool\("(\w+)", (\d+)\)', text):
        consts[m.group(1)] = m.group(2) + (f" (harvest {m.group(3)})" if m.group(3) != "0" else "")
    out = []
    for name, inner in calls(body, ["r", "n"]):
        a = split_top(inner)
        get = a[4]
        g = {"how": get.split("(")[0].strip()}
        gm = re.match(r"(\w+)\((.*)\)", get, re.S)
        if gm:
            args = split_top(gm.group(2))
            if g["how"] == "World":
                g["needs"] = consts.get(args[0], args[0])
                g["em"] = float(args[1])
            elif g["how"] == "Drop":
                g["em"] = float(args[0])
        out.append({
            "pats": strings(a[0]),
            "tier": int(a[1]),
            "path": a[2].strip().lower(),
            "kind": a[3].strip().lower(),
            "get": g,
            "note": unstr(a[5]) if name == "n" and len(a) > 5 else "",
        })
    return out


def rule_of(rs, i):
    for r in rs:
        if any(wildcard(p, i) for p in r["pats"]):
            return r
    return None


AT_NAMES = {"Hand": "by hand", "Table": "crafting table", "Fire": "over a fire", "Furnace": "furnace",
            "Fill": "filled at water"}


def at_text(at):
    at = at.strip()
    m = re.match(r'Station\(\s*"([^"]+)"\s*\)', at)
    if m:
        return m.group(1)
    m = re.match(r"Break\((.*)\)", at, re.S)
    if m:
        a = m.group(1).strip()
        return "picked by hand" if a in ("Needs::Hand", "Hand") else f"broken with {a.lower().replace('_', ' ')}"
    return AT_NAMES.get(at, at)


def recipes(root):
    """`graph::RECIPES`: [{out, n, ins: [(id, n)], at}]."""
    text = strip_comments(open(f"{root}/mods/base/progression/src/graph.rs").read())
    body = text[text.index("const RECIPES_LIST"):]
    body = body[body.index("= &[") + 4:]
    body = body[:body.index("\n];")]
    out = []

    def ins_of(s):
        return [(a, float(b)) for a, b in re.findall(r'\(\s*"([^"]+)"\s*,\s*([\d.]+)\s*\)', s)]

    for name, inner in calls(body, ["c", "s", "b", "Recipe"]):
        if name == "Recipe":
            f = {}
            for part in split_top(inner):
                if ":" in part:
                    k, v = part.split(":", 1)
                    f[k.strip()] = v.strip()
            if "out" in f:
                out.append({"out": unstr(f["out"]), "n": float(f.get("n", "1")), "ins": ins_of(f.get("ins", "")), "at": at_text(f.get("at", ""))})
            continue
        a = split_top(inner)
        try:
            if name == "c":
                out.append({"out": unstr(a[0]), "n": float(a[1]), "ins": ins_of(a[2]), "at": at_text(a[3])})
            elif name == "s":
                out.append({"out": unstr(a[0]), "n": 1.0, "ins": ins_of(a[1]), "at": at_text(a[2])})
            elif name == "b":
                out.append({"out": unstr(a[0]), "n": float(a[1]), "ins": ins_of(a[2]), "at": at_text(f"Break({a[3]})")})
        except (IndexError, ValueError):
            pass
    # The clothes macro: hat, coat, trousers, boots of each cloth.
    for mat, x in re.findall(r'clothes!\("(\w+)", "([^"]+)"\)', text):
        for piece, k, s in (("hat", 2, 1), ("coat", 4, 2), ("trousers", 3, 2), ("boots", 2, 1)):
            out.append({"out": f"base:{mat}_{piece}", "n": 1.0, "ins": [(x, k), ("#string", s)], "at": "crafting table"})
    return out


def progression_sheet(root):
    """From PROGRESSION_SHEET.md: {id: EM}, the ages, species rows, gear rows, boss rows."""
    md = open(f"{root}/mods/base/progression/PROGRESSION_SHEET.md").read()
    em = {}
    sec = md[md.index("## What exists"):]
    nxt = re.search(r"\n## ", sec[5:])
    sec = sec[:nxt.start() + 5] if nxt else sec
    for line in sec.split("\n"):
        if line.startswith("- ") and ":" in line:
            for name, v in re.findall(r"([a-z0-9_:]+) \(([\d.]+|-)\)", line):
                i = name if ":" in name else f"base:{name}"
                em[i] = None if v == "-" else float(v)
    out = {"em": em, "ages": [], "species": {}, "gear": {}, "kinds": {}, "bosses": [], "boss_notes": {}}
    for heads, head, rows in tables(md):
        h = heads[-1] if heads else ""
        if h == "The ages":
            out["ages"] = [{"tier": int(r[0]), "name": r[1], "hours": r[2]} for r in rows]
        elif h == "Species":
            for r in rows:
                out["species"][r[0]] = dict(zip(head, r))
        elif h == "Weapons, tools and armour" and head[0] == "material":
            for r in rows:
                out["gear"][r[0]] = dict(zip(head, r))
        elif h == "Weapons, tools and armour" and head[0] == "kind":
            for r in rows:
                out["kinds"][r[0]] = dict(zip(head, r))
        elif h == "Biome bosses":
            out["bosses"] += [dict(zip(head, r)) for r in rows]
    if "## Biome bosses" in md:
        bsec = md[md.index("## Biome bosses"):]
        for m in re.finditer(r"^(\w+): (a titan.*)$", bsec, re.M):
            out["boss_notes"][m.group(1)] = m.group(2)
    return out


def sheet_rows(root):
    """Rows of the generated sheets keyed by the id in their first column: {id: [(sheet, section,
    {col: value})]}."""
    files = [("survival", "mods/base/vitals/SHEET.md"), ("crafting", "mods/base/crafting/SHEET.md")]
    out = {}
    for name, f in files:
        p = f"{root}/{f}"
        if not os.path.exists(p):
            continue
        for heads, head, rows in tables(open(p).read()):
            for r in rows:
                k = r[0].strip("`*")
                if re.fullmatch(r"[a-z]+:[a-z0-9_]+", k):
                    out.setdefault(k, []).append((name, heads[-1] if heads else "", dict(zip(head, r))))
    return out


def crafting_sheet(root):
    md = open(f"{root}/mods/base/crafting/SHEET.md").read()
    mats, gems = {}, {}
    for heads, head, rows in tables(md):
        h = heads[-1] if heads else ""
        if h == "Materials":
            for r in rows:
                row = dict(zip(head, r))
                row["provisional"] = "provisional" in r[0]
                mats[r[0].split(" ")[0]] = row
        elif h == "Gems":
            for r in rows:
                gems[r[0]] = dict(zip(head, r))
    return mats, gems


# ------------------------------------------------------------------ species


def species_sheet(root):
    """SPECIES.md per species: model, entity, scientific name, biomes, the 'at a glance' row and the
    facts tables."""
    md = open(f"{root}/mods/animals/SPECIES.md").read()
    glance = {}
    for heads, head, rows in tables(md):
        if heads and heads[-1] == "At a glance":
            for r in rows:
                glance[r[0]] = dict(zip(head, r))
    out = []
    for p in re.split(r"^## ", md, flags=re.M)[1:]:
        title = p.split("\n", 1)[0].strip()
        m = re.search(r"Model `(\w+)`, entity `([\w:]+)`", p)
        if not m:
            continue
        sci = re.search(r"^\*([^*]+)\*,\s*(.*?)\. Model", p, re.M)
        sp = {"key": title.split(" (")[0], "title": title, "model": m.group(1), "entity": m.group(2),
              "scientific": sci.group(1) if sci else "", "lineage": sci.group(2) if sci else "",
              "biomes": [], "facts": {}, "glance": glance.get(title, {})}
        for heads, head, rows in tables("## " + p):
            if head and head[0] == "Where it lives":
                sp["biomes"] = [(r[0].strip("`"), r[1]) for r in rows]
            elif len(head) >= 2:
                for r in rows:
                    if r and r[0]:
                        sp["facts"][r[0]] = " · ".join(x for x in r[1:] if x)
        spawn = re.search(r"^Spawns .*$", p, re.M)
        if spawn:
            sp["spawns"] = spawn.group(0)
        out.append(sp)
    return out


def clips(root, model):
    """Animation clips of a model, by file: {file: [clip names]}."""
    out = {}
    for f in sorted(glob.glob(f"{root}/mods/*/assets/animations/{model}*.animation.json")):
        stem = os.path.basename(f)[:-len(".animation.json")]
        if stem != model and not re.fullmatch(re.escape(model) + r"_(sit|lie|rest|bask|fly|flop)", stem):
            continue
        try:
            names = list(json.load(open(f)).get("animations", {}))
        except ValueError:
            continue
        out[stem] = [n.split(".")[-1] for n in names]
    return out


# ------------------------------------------------------------------ parts and looks


def crafting_kinds(root):
    """crafting::kinds: {kind: [(slot, role)]}, shapes [(id, kind, slot)]."""
    t = strip_comments(open(f"{root}/mods/base/crafting/src/kinds.rs").read())
    consts = {}
    for name, body in re.findall(r"const (\w+): &\[Slot\] = &\[(.*?)\];", t, re.S):
        consts[name] = re.findall(r's\("(\w+)", Role::(\w+)\)', body)
    kinds = {}
    for _, inner in calls(t[t.index("pub const KINDS"):t.index("pub fn kind(")], ["Kind"]):
        k = re.search(r'id: "(\w+)"', inner).group(1)
        s = re.search(r"slots: (&\[.*?\]|\w+),", inner, re.S).group(1)
        kinds[k] = consts[s] if s in consts else re.findall(r's\("(\w+)", Role::(\w+)\)', s)
    shapes = re.findall(r'sh\("(\w+)", "(\w+)", "(\w+)"', t)
    return kinds, shapes


def looks(root):
    """crafting::looks: LOOKS as {kind: {socket, anchors: {art: [x, y]}, layers}}, PART_MAPS and
    GEM_SPOTS."""
    t = strip_comments(open(f"{root}/mods/base/crafting/src/looks.rs").read())
    body = t[t.index("pub const LOOKS"):t.index("pub fn look(")]
    out = {}
    for _, inner in calls(body, ["KindLook"]):
        if 'kind: "' not in inner:
            continue
        k = re.search(r'kind: "(\w+)"', inner).group(1)
        sock = [float(x) for x in re.search(r"socket: \[([\d., -]+)\]", inner).group(1).split(",")]
        anchors = {a: [float(x), float(y)] for a, x, y in re.findall(r'\("(\w+)", \[([\d.-]+), ([\d.-]+)\]\)', inner)}
        layers = []
        for _, li in calls(inner, ["Layer"]):
            if "regions" not in li:
                continue
            layers.append({"name": re.search(r'name: "(\w+)"', li).group(1),
                           "regions": strings(re.search(r"regions: &\[(.*?)\]", li).group(1)),
                           "moves": "moves: true" in li, "painted": "painted: true" in li})
        out[k] = {"socket": sock, "anchors": anchors, "layers": layers}
    maps = strings(re.search(r"PART_MAPS: \[&str; \d+\] = \[(.*?)\]", t).group(1))
    spots = {k: [[int(a), int(b)] for a, b in re.findall(r"\[(-?\d+), (-?\d+)\]", v)]
             for k, v in re.findall(r'\("(\w+)", \[(\[[^\]]+\], \[[^\]]+\], \[[^\]]+\])\]\)', t)}
    return out, maps, spots


# ------------------------------------------------------------------ item textures named in code


def texture_overrides(root):
    """Items whose icon is another texture (`r.item(X).texture("t")`, a block's
    `.item_texture("t")`): {(ns, item name or 're:'regex): (texture ns, path under textures)}."""
    out = {}
    for ns in MODS:
        for f in glob.glob(f"{root}/mods/{ns}/src/**/*.rs", recursive=True):
            t = strip_comments(open(f, encoding="utf-8").read())
            statics = dict(re.findall(r'static (\w+): (?:Item|Block) = (?:Item|Block)::named\("([\w:]+)"\)', t))
            for m in re.finditer(r"r\s*\.\s*(item|block)\(", t):
                end = t.find(";", m.end())
                stmt = t[m.start():end]
                tex = re.search(r'\.(item_texture|texture)\("([\w/:]+)"\)', stmt)
                if not tex or (m.group(1) == "block" and tex.group(1) != "item_texture"):
                    continue
                arg = stmt[m.end() - m.start():]
                key = None
                a = re.match(r"\s*&?(\w+)\s*\)", arg)
                s = re.match(r'\s*&?(?:Item|Block)::(?:named|generated)\(\s*"([\w:]+)"', arg)
                if s:
                    key = s.group(1)
                elif a and a.group(1) in statics:
                    key = statics[a.group(1)]
                elif a:
                    before = t[max(0, m.start() - 600):m.start()]
                    g = re.findall(r"let " + re.escape(a.group(1)) + r'\s*=\s*(?:Item|Block)::(?:named|generated)\(\s*"([\w:]+)"', before)
                    if g:
                        key = g[-1]
                    else:
                        g = re.findall(r"let " + re.escape(a.group(1)) + r'\s*=\s*Item::generated\(format!\("([^"]+)"', before)
                        if g:
                            key = "re:" + re.sub(r"\\\{[^}]*\\\}", ".+", re.escape(g[-1]))
                if key:
                    name = tex.group(2)
                    tns = ns
                    if ":" in name:
                        tns, name = name.split(":", 1)
                    if "/" not in name:
                        name = "item/" + name
                    out[(ns, key.split(":", 1)[1] if ":" in key and not key.startswith("re:") else key)] = (tns, name)
    return out


def springs(root):
    """`materials::SPRING`: the woods a bow stave can be, {material: spring}."""
    t = strip_comments(open(f"{root}/mods/base/crafting/src/materials.rs").read())
    m = re.search(r"SPRING: &\[\(&str, f32\)\]\s*=\s*&\[(.*?)\];", t, re.S)
    return {a: float(b) for a, b in re.findall(r'\("(\w+)", ([\d.]+)\)', m.group(1))} if m else {}


def knapped(root):
    """`vitals::first::KNAPPED`: [(part shape, kind, 5 rows of '.'/'X')]."""
    t = open(f"{root}/mods/base/vitals/src/first.rs").read()
    return [(a, b, strings(c)) for a, b, c in re.findall(r'Knapped \{ shape: "(\w+)", kind: "(\w+)", outline: \[(.*?)\]', t)]


def recipe_raw(root):
    """graph.rs recipes with their `at` kept raw (Hand, Table, Fire, Furnace, Station, Break, Fill)."""
    text = strip_comments(open(f"{root}/mods/base/progression/src/graph.rs").read())
    body = text[text.index("const RECIPES_LIST"):]
    body = body[body.index("= &[") + 4:]
    body = body[:body.index("\n];")]
    out = []
    for name, inner in calls(body, ["c", "s", "b", "Recipe"]):
        if name == "Recipe":
            f = {}
            for part in split_top(inner):
                if ":" in part:
                    k, v = part.split(":", 1)
                    f[k.strip()] = v.strip()
            if "out" in f:
                out.append((unstr(f["out"]), f.get("at", "").strip(), float(f.get("secs", "3"))))
            continue
        a = split_top(inner)
        try:
            if name == "c":
                out.append((unstr(a[0]), a[3].strip(), 3.0))
            elif name == "s":
                out.append((unstr(a[0]), a[2].strip(), 10.0))
            elif name == "b":
                out.append((unstr(a[0]), f"Break({a[3]})", float(a[4])))
        except IndexError:
            pass
    return out


# ------------------------------------------------------------------ blocks from the sources


def stmt_end(t, i):
    """Where the statement from `i` ends: the first `;` outside brackets and strings."""
    depth, q = 0, False
    while i < len(t):
        c = t[i]
        if q:
            if c == "\\":
                i += 1
            elif c == '"':
                q = False
        elif c == '"':
            q = True
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth < 0:
                return i
        elif c == ";" and depth == 0:
            return i
        i += 1
    return i


def _num_expr(x):
    """A box corner's number: `16.0`, `2.0 * n` (each unknown name taken as 1)."""
    class One(dict):
        def __missing__(self, k):
            return 1.0
    try:
        return float(eval(x.replace("f32", ""), {"__builtins__": {}}, One()))
    except Exception:
        return None


def _tex_name(ns, name):
    tns = ns
    if ":" in name:
        tns, name = name.split(":", 1)
    if "/" not in name:
        name = "block/" + name
    return f"{tns}:{name}"


def _boxes(s):
    out = []
    for a, b in re.findall(r"\[([^\[\]]+)\]\s*,\s*\[([^\[\]]+)\]", s):
        lo, hi = [_num_expr(x) for x in a.split(",")], [_num_expr(x) for x in b.split(",")]
        if len(lo) == 3 and len(hi) == 3 and None not in lo + hi:
            out.append(tuple(v / 16.0 for v in lo + hi))
    return out


def block_defs(root):
    """Each block's look as its `r.block(...)` chain says (`.texture`, `.textures` top/side/bottom,
    `.faces` -X +X -Y +Y -Z +Z, `.model_box`, `.plant`, `.tint`), in the shape of a `/tk dump`
    block: {id or 're:'regex: {textures [6], model, boxes, comps, fill}}; a `format!` name's
    `{var}` is `{0}` in its textures, filled from the regex's group. Blocks put down by a helper
    (`loose(r, &LOOSE_ROCK, .., "cobblestone", &[boxes])`) take its last texture and its boxes.
    The game's own registry (`/tk dump`) wins where there is one; this needs no game."""
    out = {}
    for ns in MODS:
        files = sorted(glob.glob(f"{root}/mods/{ns}/**/*.rs", recursive=True))
        def is_tex(n):
            tns, path = _tex_name(ns, n.split("@", 1)[0]).split(":", 1)
            f = f"{root}/mods/{tns}/assets/textures/{path}"
            return os.path.exists(f + ".png") or (os.path.isdir(f) and any(re.fullmatch(r"\d+\.png", x) for x in os.listdir(f)))
        # The whole mod's statics (its `api` crate too) and `use .. {X as Y}` aliases, and its
        # helper functions (`fn wood_block(r, handle, wood) { r.block(handle)...texture(format!(..)) }`).
        texts = [strip_comments(open(f, encoding="utf-8").read()) for f in files]
        statics = {}
        for t in texts:
            statics.update(re.findall(r'static (\w+): Block = Block::named\("([\w:]+)"\)', t))
        for t in texts:
            for a0, a1 in re.findall(r"\b([A-Z][A-Z0-9_]+) as ([A-Z][A-Z0-9_]+)\b", t):
                if a0 in statics:
                    statics.setdefault(a1, statics[a0])
        fns = {}
        for t in texts:
            for m in re.finditer(r"\bfn (\w+)\(([^)]*)\)[^{]*\{", t):
                body_end = stmt_end(t, m.end())
                fns.setdefault(m.group(1), ([x.split(":")[0].strip() for x in m.group(2).split(",") if x.strip()], t[m.end():body_end]))

        def expand(name, args):
            """A helper call's `r.block(param)` statement with its arguments put in, or None."""
            if name not in fns:
                return None
            params, body = fns[name]
            if len(params) != len(args) or "r.block(" not in body.replace(" ", ""):
                return None
            vals = dict(zip(params, [x.strip() for x in args]))
            bm = re.search(r"\br\s*\.\s*block\(\s*(\w+)\s*\)", body)
            if not bm or bm.group(1) not in vals:
                return None
            stmt = body[bm.start():stmt_end(body, bm.start())]
            stmt = stmt.replace(bm.group(0), f"r.block({vals[bm.group(1)]})", 1)

            def lit(m):
                text = m.group(1)
                for p, v in vals.items():
                    if v.startswith('"'):
                        text = text.replace("{" + p + "}", v.strip('"'))
                return '"' + text + '"' if "{" not in text else m.group(0)
            return re.sub(r'&?format!\(\s*"([^"]*)"\s*\)', lit, stmt) + ";"

        for t in list(texts):
            for m in re.finditer(r"\b(?!block\b|machine\b|item\b)(\w+)\(\s*r\s*,", t):
                e = expand(m.group(1), split_top(t[m.end() - 2 + 1:stmt_end(t, m.end() - 1)].strip()))
                if e:
                    texts.append(e)
        for t in texts:
            heads = [(m, "block") for m in re.finditer(r"\br\s*\.\s*(?:block|machine)\(\s*", t)]
            heads += [(m, "helper") for m in re.finditer(r"\b(?!block\b|machine\b|item\b)(\w+)\(\s*r\s*,\s*", t)]
            for m, how in heads:
                end = stmt_end(t, m.start())
                stmt = t[m.start():end]
                arg = t[m.end():end]
                key, var = None, None
                a = re.match(r"&?(\w+)\s*[,)]", arg)
                s = re.match(r'&?Block::(?:named|generated)\(\s*"([\w:]+)"', arg)
                g = re.match(r'&?Block::generated\(\s*format!\(\s*"([^"]+)"', arg)
                if s:
                    key = s.group(1)
                elif g:
                    var = re.findall(r"\{(\w*)\}", g.group(1))
                    if len(var) != 1:
                        continue
                    var = var[0]
                    key = "re:" + re.escape(g.group(1)).replace(re.escape("{" + var + "}"), "(.+)")
                elif a and a.group(1) in statics:
                    key = statics[a.group(1)]
                if not key:
                    continue
                if ":" in key and not key.startswith("re:"):
                    key = key.split(":", 1)[1]
                fill = lambda x: x.replace("{" + var + "}", "{0}") if var else x
                d = {"model": "Cube", "boxes": [], "comps": [], "textures": [None] * 6}
                if how == "helper":
                    if a is None or a.group(1) not in statics:
                        continue
                    texs = [x for x in strings(arg) if is_tex(x)]
                    bx = _boxes(arg)
                    if not texs or not bx:
                        continue
                    d["textures"] = [_tex_name(ns, texs[-1])] * 6
                    d["model"], d["boxes"] = "Boxes", bx
                    out.setdefault(f"{ns}:{key}" if not key.startswith("re:") else key, d)
                    continue
                # The face presets in the order said (the SDK's `pick`: the last one said wins),
                # `.layer(..)` over the faces picked last, `.tint_kind(..)` / `.tint(..)` on their
                # top layer: faces -X +X -Y +Y -Z +Z, each a list of layers `name@kind`.
                faces = [[] for _ in range(6)]
                picked = []
                PICKS = {"texture": range(6), "top": [3], "bottom": [2], "ends": [2, 3], "sides": [0, 1, 4, 5],
                         "front": [4], "back": [5], "north": [4], "south": [5], "west": [0], "east": [1]}
                for c in re.finditer(r"\.(texture|textures|faces|top|bottom|ends|sides|front|back|north|south|west|east|layer|tint_kind|tint)\(", stmt):
                    body = stmt[c.end():stmt_end(stmt, c.end())]
                    what = c.group(1)
                    if what in ("tint", "tint_kind"):
                        k = re.search(r"tints::(\w+)", body)
                        if k and picked:
                            for i in picked:
                                if faces[i] and "@" not in faces[i][-1]:
                                    faces[i][-1] += "@" + k.group(1).lower()
                        elif k:
                            d["comps"].append("tint")
                        continue
                    names = [fill(x) for x in re.findall(r'"([\w/:{}#@ ]+)"', body)]
                    names = [x.split()[0] for x in names if x.split() and ("{" in x or is_tex(x.split()[0].split("#")[0]))]
                    if not names:
                        continue
                    if what == "textures" and len(names) >= 3:
                        tp, sd, bt = (_tex_name(ns, x) for x in names[:3])
                        faces = [[sd], [sd], [bt], [tp], [sd], [sd]]
                        picked = list(range(6))
                    elif what == "faces":
                        rep = re.search(r'\[\s*"([\w/:]+)"\s*;\s*6\s*\]', body)
                        fs = [rep.group(1)] * 6 if rep else names[:6]
                        if len(fs) == 6:
                            faces = [[_tex_name(ns, x)] for x in fs]
                            picked = list(range(6))
                    elif what == "layer":
                        for i in picked:
                            faces[i].append(_tex_name(ns, names[0].split("#")[0]))
                    else:
                        # A state's choice (`if lit { "x_lit" } else { "x" }`): the resting one, as the item shows.
                        rest = [x for x in names if not re.search(r"_(lit|on|powered|active)$", x)] or names
                        picked = list(PICKS[what])
                        for i in picked:
                            faces[i] = [_tex_name(ns, rest[0].split("#")[0])]
                if all(faces):
                    d["textures"] = ["+".join(f) for f in faces]
                elif any(faces):
                    first = next(f for f in faces if f)
                    d["textures"] = ["+".join(f or first) for f in faces]
                for c in re.finditer(r"\.model_box(?:_faces)?\(", stmt):
                    body = stmt[c.end():stmt_end(stmt, c.end())]
                    d["boxes"] += _boxes(body)[:1]
                    if "model_box_faces" in stmt[c.start():c.end()]:
                        n = [x for x in re.findall(r'"([\w/:]+)"', body) if is_tex(x)]
                        if n:
                            d["textures"] = [_tex_name(ns, n[0])] * 6
                if d["boxes"]:
                    d["model"] = "Boxes"
                if re.search(r"\.plant\(\)", stmt):
                    d["model"] = "Plant"
                if re.search(r"\.tint_top\(", stmt):
                    d["comps"] += ["tint", "tint_faces"]
                if d["textures"][0] is None:
                    continue
                out.setdefault(f"{ns}:{key}" if not key.startswith("re:") else f"re:{ns}:{key[3:]}", d)
    # Crops (`crops.rs`): their stages' textures by the look's name, `<look>_stage_<n>`.
    p = f"{root}/mods/base/src/crops.rs"
    if os.path.exists(p):
        t = open(p, encoding="utf-8").read()
        fn = re.search(r"fn texture\(.*?\n\}", t, re.S)
        if fn:
            for b, look in re.findall(r'"(\w+)"\s*=>\s*"(\w+)"', fn.group(0)):
                out.setdefault(f"base:{b}", {"crop": look})
    return out


def block_def(defs, i):
    """Block `i`'s look from `block_defs` (exact, then a `format!` name), as a dump block, or None."""
    d = defs.get(i)
    if d is None:
        for k, v in defs.items():
            m = re.fullmatch(k[3:], i) if k.startswith("re:") else None
            if m:
                d = dict(v, textures=[x.replace("{0}", m.group(1)) if x else x for x in v["textures"]])
                break
    if d is None or "crop" in d:
        return d
    return dict(d, id=i)


def egg_looks(root):
    """{species: egg look} (`animals_core::eggs::EGGS`): the item `<species>_egg` is drawn as
    `item/egg_<look>`."""
    p = f"{root}/mods/animals/core/src/eggs.rs"
    if not os.path.exists(p):
        return {}
    t = strip_comments(open(p, encoding="utf-8").read())
    m = re.search(r"pub const EGGS[^=]*=\s*&\[(.*?)\];", t, re.S)
    return dict(re.findall(r'\(\s*"(\w+)"\s*,\s*"(\w+)"', m.group(1))) if m else {}


def drawn_parts(root):
    """`crafting::looks`' drawn parts (art redo v3): SHELLS_DRAWN (shells drawn per material) and
    `drawn_shape`'s mapping (a shape drawn as another's picture)."""
    t = strip_comments(open(f"{root}/mods/base/crafting/src/looks.rs").read())
    m = re.search(r"SHELLS_DRAWN: \[&str; \d+\] = \[(.*?)\]", t)
    shells = strings(m.group(1)) if m else []
    f = re.search(r"pub fn drawn_shape\(.*?\n\}", t, re.S)
    mapping = {}
    if f:
        for pats, to in re.findall(r'((?:"\w+"\s*\|?\s*)+)=>\s*"(\w+)"', f.group(0)):
            for p in strings(pats):
                mapping[p] = to
    return shells, mapping


def cylinders(root):
    """Blocks drawn as a cylinder of box strips (`.cylinder(..)` in a loop over
    `(block, radius, half_length, ..)`, upstream's wheels in `vehicles.rs`): {id: dump-like block
    with its strips and hubs as boxes, fitted into one block}."""
    out = {}
    for f in glob.glob(f"{root}/mods/*/src/**/*.rs", recursive=True):
        t = strip_comments(open(f, encoding="utf-8").read())
        if ".cylinder(" not in t:
            continue
        ns = f.split("/mods/")[1].split("/")[0]
        statics = {}
        for g in glob.glob(f"{root}/mods/{ns}/**/*.rs", recursive=True):
            statics.update(re.findall(r'static (\w+): Block = Block::named\("([\w:]+)"\)', open(g, encoding="utf-8").read()))
        faces = re.search(r'let tread = "(\w+)";\s*let side = "(\w+)";', t)
        hub = re.search(r'\[\("(\w+)", UV_FROM_BOX\); 6\]', t)
        if not faces:
            continue
        for st, radius, half in re.findall(r"\(&(\w+),\s*([\d.]+)(?:f32)?,\s*([\d.]+)(?:f32)?,", t):
            if st not in statics:
                continue
            r, hl = float(radius), float(half)
            boxes = []
            for j in range(16):
                y0, y1 = 2 * r * (j / 16 - 0.5), 2 * r * ((j + 1) / 16 - 0.5)
                mid = (y0 + y1) / 2
                x = (r * r - mid * mid) ** 0.5
                boxes.append((8 - x, 8 + y0, 9 - hl, 8 + x, 8 + y1, 7 + hl))
            h = r * 0.33
            boxes += [(8 - h, 8 - h, 7 + hl, 8 + h, 8 + h, 8 + hl)]
            lo = min(min(b[:3]) for b in boxes)
            hi = max(max(b[3:]) for b in boxes)
            fit = lambda v: (v - lo) / (hi - lo)
            tread, side = (_tex_name(ns, x) for x in faces.groups())
            out[statics[st] if ":" in statics[st] else f"{ns}:{statics[st]}"] = {
                "model": "Boxes", "comps": [], "boxes": [tuple(fit(v) for v in b) for b in boxes],
                "textures": [tread, tread, tread, tread, side, side]}
    return out
