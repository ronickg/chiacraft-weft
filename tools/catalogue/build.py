#!/usr/bin/env python3
"""The crew gallery's catalogue: every item, block, part, gem, plant and creature of the game at a
commit, with its picture and what the sources know of it, as one filterable page.

    python3 tools/catalogue/build.py                 # dev's tip, for the crew gallery
    python3 tools/catalogue/build.py --rev <commit>  # any commit
    python3 tools/catalogue/build.py --rev HEAD --out site   # a static site (GitHub Pages)

It reads the sources at that commit (exported with `git archive`, so the checkout it runs from
does not matter): the lang files, `mods/base/progression` (rules, recipes and its sheet), the
vitals, crafting and species sheets, the textures and the box models. The registry the game
itself lists (creative tabs, each block's face textures) comes from the newest `/tk dump`
(`~/Coding/chiacraft-archive/images/catalogue/registry/<commit>.json`, made by a PC run of
`notes/crew/tester-tools/t2/scripts/t2_catalogue_dump.txt`, fetched with `--fetch-dump <job>`); things the dump
hasn't seen still show, from the sources alone. Never runs the game.

Writes the pictures to `~/Coding/chiacraft-archive/images/catalogue/<commit>/` and the page to
`notes/crew/gallery/catalogue/index.html` in the main checkout (the gallery serves it at
/catalogue). With `--out <dir>` it writes a folder that stands alone instead: `<dir>/index.html`
and its pictures under `<dir>/img/`, linked relatively, nothing else read or written outside the
repository (the registry: the newest committed snapshot in `tools/catalogue/registry/`). The
GitHub Pages workflow (`.github/workflows/catalogue.yml`) builds it so. Only the game's own files
are read (`PATHS`: the mods, the engine's textures and lang, the box-model renderer), never
`reports/` or resource packs.
"""

import argparse
import glob
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import numpy as np  # noqa: E402
from PIL import Image  # noqa: E402

import images as I  # noqa: E402
import mobs  # noqa: E402
import sources as S  # noqa: E402

# The repository this script is in (a worktree shares the main checkout's commits).
REPO = subprocess.run(["git", "-C", HERE, "rev-parse", "--show-toplevel"], capture_output=True, text=True).stdout.strip() \
    or os.path.expanduser("~/Coding/chiacraft-weft")
MAIN = os.path.expanduser("~/Coding/chiacraft-weft")
ARCHIVE = os.path.expanduser("~/Coding/chiacraft-archive/images")
OUT_PAGE = os.path.join(MAIN, "notes/crew/gallery/catalogue/index.html")
SNAPSHOTS = os.path.join(HERE, "registry")
PC = "ronaldg@192.168.1.13"

PATHS = ["mods", "crates/engine/assets/lang", "crates/engine/assets/textures", ".claude/skills/blocky-model/scripts"]
# Read when the commit has it (`git archive` fails on a path a commit lacks).
OPTIONAL = ["notes/crew/systems-map/systems.json"]
SKIP = ("/sounds/", "/music/", "/target/", "/source/")

AGES = ["Stone Age", "Bronze Age", "Iron Age", "Age of Steam", "Age of Lightning", "Age of the Future", "Age of Stars"]
HOSTILE = {"base:zombie", "base:skeleton", "base:creeper", "base:spider"}
FARM = {"base:cow", "base:pig", "base:sheep", "base:chicken"}
# The working part's materials drawn per kind, by what a player can make them of today.
SHOW_KINDS = ["axe", "pickaxe", "shovel", "hoe", "sword", "bow", "knife", "spear", "helmet", "chestplate", "leggings", "boots"]


def log(*a):
    print(*a, file=sys.stderr, flush=True)


def export(rev):
    """The source tree at `rev` in a temporary folder, without sounds."""
    commit = subprocess.check_output(["git", "-C", REPO, "rev-parse", "--short=8", rev], text=True).strip()
    full = subprocess.check_output(["git", "-C", REPO, "rev-parse", rev], text=True).strip()
    tmp = tempfile.mkdtemp(prefix="catalogue_")
    have = [p for p in OPTIONAL if subprocess.run(["git", "-C", REPO, "cat-file", "-e", f"{full}:{p}"], capture_output=True).returncode == 0]
    data = subprocess.run(["git", "-C", REPO, "archive", "--format=tar", full, "--"] + PATHS + have, capture_output=True, check=True).stdout
    with tarfile.open(fileobj=io.BytesIO(data)) as t:
        members = [m for m in t.getmembers() if not any(s in "/" + m.name for s in SKIP) and not m.name.startswith("/") and ".." not in m.name]
        t.extractall(tmp, members=members, filter="data")
    subj = subprocess.check_output(["git", "-C", REPO, "log", "-1", "--format=%cd %s", "--date=format:%Y-%m-%d %H:%M", full], text=True).strip()
    return tmp, commit, subj, full


def art_dates(full):
    """{repo path: unix time of the last commit that changed it} for every texture and model at
    commit `full`."""
    out = subprocess.run(["git", "-C", REPO, "log", "--format=@%ct", "--name-only", full, "--",
                          ":(glob)mods/*/assets/textures/**", ":(glob)mods/*/assets/models/**"], capture_output=True, text=True).stdout
    dates, t = {}, 0
    for line in out.split("\n"):
        if line.startswith("@"):
            t = int(line[1:])
        elif line and line not in dates:
            dates[line] = t
    return dates


def newest_registry(commit, archive=True):
    """The dump made at `commit`, else the newest: the archive's (`--fetch-dump`), then the
    snapshot committed in `tools/catalogue/registry/` (its newest by the commit time it names)."""
    dirs = ([os.path.join(ARCHIVE, "catalogue", "registry")] if archive else []) + [SNAPSHOTS]
    for d in dirs:
        exact = os.path.join(d, f"{commit}.json")
        if os.path.exists(exact):
            return exact
    for d in dirs:
        files = sorted(glob.glob(os.path.join(d, "*.json")), key=os.path.getmtime)
        if d == SNAPSHOTS:
            files = sorted(files, key=lambda f: json.load(open(f)).get("_made", {}).get("time", 0))
        if files:
            return files[-1]
    return None


def title_case(i):
    return i.split(":")[-1].replace("_", " ").capitalize()


class Builder:
    def __init__(self, root, commit, outdir, registry, prefix=None):
        self.root, self.commit, self.outdir = root, commit, outdir
        # Pictures' URLs: the gallery's (`/img/catalogue/<commit>/`), or relative (a site).
        self.prefix = prefix if prefix is not None else f"/img/catalogue/{commit}/"
        self.tex = I.Textures(root)
        self.L = S.lang(root)
        self.rules = S.rules(root)
        self.recipes = S.recipes(root)
        self.raw_recipes = S.recipe_raw(root)
        # The clothes macro's (`recipes` adds them after the list): at the table.
        self.raw_recipes += [(r["out"], "Table", 3.0) for r in self.recipes[len(self.raw_recipes):]]
        self.prog = S.progression_sheet(root)
        self.rows = S.sheet_rows(root)
        self.mats, self.gems = S.crafting_sheet(root)
        self.springs = S.springs(root)
        kinds, shapes = S.crafting_kinds(root)
        looks, maps, spots = S.looks(root)
        metals = {m for m, r in self.mats.items() if r.get("Works as") in ("Metal",)} | {"gold"}
        self.kinds, self.shapes = kinds, shapes
        self.looks = I.Looks(looks, kinds, shapes, maps, metals)
        self.looks.shells_drawn, self.looks.drawn_shapes = S.drawn_parts(root)
        self.overrides = S.texture_overrides(root)
        self.src_blocks = S.block_defs(root)
        self.eggs = S.egg_looks(root)
        self.reg = json.load(open(registry)) if registry else {"items": [], "blocks": [], "entities": []}
        self.reg_items = {i["id"]: i for i in self.reg["items"]}
        self.reg_blocks = {b["id"]: b for b in self.reg["blocks"]}
        self.entries = []
        self.made = 0
        self.mat_from = self._materials_from()

    # ------------------------------------------------------------ helpers

    def _materials_from(self):
        """crafting material -> the item it's made from today ('' none), from the sheet."""
        md = open(f"{self.root}/mods/base/progression/PROGRESSION_SHEET.md").read()
        out = {}
        for heads, head, rows in S.tables(md):
            if heads and heads[-1].startswith("Tool parts"):
                for r in rows:
                    out[r[0]] = {"age": r[1], "from": "" if r[2] == "-" else r[2]}
        return out

    def add(self, e):
        """An entry, stamped with when its art last changed (the newest of the files drawn)."""
        files = sorted(os.path.relpath(p, self.root) for p in self.tex.used)
        self.tex.used = set()
        ts = [self.dates.get(f, 0) for f in files]
        if ts and max(ts):
            e["art"] = max(ts)
            e["art_files"] = [[f, time.strftime("%Y-%m-%d %H:%M", time.localtime(self.dates.get(f, 0)))] for f in files][:12]
        self.entries.append(e)

    def name(self, i):
        ns, n = i.split(":", 1)
        for k in (f"item.{ns}.{n}", f"block.{ns}.{n}", f"entity.{ns}.{n}"):
            if k in self.L:
                return self.L[k]
        return title_case(i)

    def mat_note(self, m):
        if self.mats.get(m, {}).get("provisional"):
            return "provisional row"
        return "" if self.mat_from.get(m, {}).get("from") else "not in the game yet"

    def mat_name(self, m):
        return self.L.get(f"material.base.{m}", m.replace("_", " "))

    def img(self, a, rel):
        """Save picture `a` under the commit's folder; its URL for the page."""
        if a is None:
            return None
        p = os.path.join(self.outdir, rel)
        if not os.path.exists(p):
            I.save(a, p)
            self.made += 1
        return self.prefix + rel

    def icon_of(self, i):
        """(picture, how) of item or block `i` without parts: its item texture, a code-named one,
        or its block from the registry (a cube or its face)."""
        ns, n = i.split(":", 1)
        for (ons, key), (tns, path) in self.overrides.items():
            if ons == ns and (key == n or (key.startswith("re:") and re.fullmatch(key[3:], n))):
                a = self.tex.get(f"{tns}:{path}")
                if a is not None:
                    return a, "item"
        a = self.tex.get(f"{ns}:item/{n}")
        if a is not None:
            return a, "item"
        # An egg: its species' look (`animals_core::eggs`), the icon the game gives the item.
        if ns == "animals" and n.endswith("_egg"):
            look = self.egg_look(n[:-len("_egg")])
            a = self.tex.get(f"animals:item/egg_{look}") if look else None
            if a is not None:
                return a, "item"
        # The block as the game's registry lists it, else as its `r.block(..)` chain in the sources says.
        # The registry knows each state's faces and the tints, but not a box model's boxes, a crop's
        # grown stage or which state rests: the sources' look wins there.
        reg, src = self.reg_blocks.get(i), S.block_def(self.src_blocks, i)
        b = reg or src
        if reg and src:
            if "crop" in src:
                b = src
            elif src.get("boxes"):
                b = dict(src, comps=sorted(set(src["comps"]) | set(reg.get("comps", []))))
            elif any(re.search(r"_(lit|on|powered|active)$", t or "") for t in reg["textures"]):
                b = dict(reg, textures=src["textures"])
        if b and "crop" in b:
            # A crop: its last stage.
            stages = glob.glob(f"{self.root}/mods/{ns}/assets/textures/block/{b['crop']}_stage_*.png")
            if stages:
                last = max(stages, key=lambda p: int(re.search(r"_stage_(\d+)\.png$", p).group(1)))
                a = self.tex.get(f"{ns}:block/{os.path.basename(last)[:-4]}")
                if a is not None:
                    return a, "flat"
        elif b:
            a, how = I.block_icon(b, self.tex)
            if a is not None:
                return a, how
        for guess in (f"block/{n}", f"block/{n}_side", f"block/{n}_top"):
            a = self.tex.get(f"{ns}:{guess}")
            if a is not None:
                return (I.iso_cube(a, a, a) if self.universe.get(i) == "block" else a), ("cube" if self.universe.get(i) == "block" else "flat")
        # A branch: a thin log of its wood's bark and end.
        if n.endswith("_branch"):
            wood = n[:-len("_branch")]
            side, end = self.tex.get(f"{ns}:block/branch_{wood}"), self.tex.get(f"{ns}:block/branch_{wood}_end")
            if side is not None:
                return I.iso_boxes([(0.3125, 0, 0.3125, 0.6875, 1, 0.6875)], end if end is not None else side, side, side), "cube"
        # A box model's textures named the other way round (`branch_oak`, `piston_top_sticky`).
        words = set(n.split("_"))
        found = sorted(os.path.basename(p)[:-4] for p in glob.glob(f"{self.root}/mods/{ns}/assets/textures/block/*.png")
                       if words <= set(os.path.basename(p)[:-4].split("_")))
        if found:
            found.sort(key=len)
            top = next((f for f in found if "top" in f.split("_")), found[0])
            side = next((f for f in found if "side" in f.split("_")), None)
            if side is None and not ({"top", "side"} & set(found[0].split("_"))):
                return self.tex.get(f"{ns}:block/{found[0]}"), "flat"
            side = side or next((f for f in found if f.startswith(found[0])), found[0])
            if side == top:
                side = next((f"{ns}:block/{x}" for x in ("piston_side",) if words & set(x.split("_"))), f"{ns}:block/{side}")
            else:
                side = f"{ns}:block/{side}"
            t, sd = self.tex.get(f"{ns}:block/{top}"), self.tex.get(side)
            if t is not None:
                return I.iso_cube(t, sd if sd is not None else t, sd if sd is not None else t), "cube"
        return None, None

    def egg_look(self, kind):
        """The egg look of the species whose entity is `kind` (`velociraptor_osmolskae` is species
        `osmolskae`)."""
        if kind in self.eggs:
            return self.eggs[kind]
        return next((l for sp, l in self.eggs.items() if kind.endswith("_" + sp)), None)

    def tab_of(self, i):
        it = self.reg_items.get(i)
        comps = it["comps"] if it else (self.reg_blocks.get(i) or {}).get("comps", [])
        tabs = [c[4:] for c in comps if c.startswith("tab/")]
        if not tabs:
            return "" if (it or i in self.reg_blocks) else "?"
        t = tabs[0]
        ns, n = t.split(":", 1)
        return self.L.get(f"tab.{ns}.{n}", n)

    # ------------------------------------------------------------ items and blocks

    def made_by(self, i):
        out = []
        for r in self.recipes:
            if S.wildcard(r["out"], i):
                ins = ", ".join(f"{int(n) if n == int(n) else n} × {self.ref(x)}" for x, n in r["ins"])
                n = int(r["n"]) if r["n"] == int(r["n"]) else r["n"]
                out.append(f"{n} from {ins} ({r['at']})" if ins else f"{n} ({r['at']})")
        return out

    def ref(self, x):
        if x.startswith("#"):
            return x[1:].replace("_", " ") + " (any)"
        return self.name(x)

    def tags_of(self, i):
        it = self.reg_items.get(i) or self.reg_blocks.get(i) or {}
        return set(it.get("comps", []))

    def used_in(self, i):
        out = []
        tags = self.tags_of(i)
        for r in self.recipes:
            if any(x == i or (x.startswith("#") and x[1:] in tags) for x, _ in r["ins"]):
                o = r["out"]
                nm = self.name(o) if "*" not in o else (o.split(":")[1].replace("*", "").strip("_").replace("_", " ") + " (each kind)")
                if nm not in out:
                    out.append(nm)
        return out

    def group_of(self, i, kind, how):
        n = i.split(":")[1]
        if kind == "part":
            return "parts"
        if n.startswith("gem_") or n in ("maunduk", "aruduk", "remaru", "emakaun"):
            return "gems"
        if kind == "tool":
            return "tools and weapons"
        if kind == "armour":
            return "armour"
        if kind == "clothing":
            return "clothing"
        if kind == "food" or (kind == "crop" and any(s == "Foods" for _, s, _ in self.rows.get(i, []))):
            return "food"
        if kind in ("plant", "crop") or n.endswith("_sapling") or n == "sapling":
            return "plants"
        if self.ids.get(i) == "block" or how in ("cube",):
            return "blocks"
        return "items"

    def facts_of(self, i):
        out = []
        for sheet, sec, row in self.rows.get(i, []):
            pairs = [[k or "", v] for k, v in list(row.items())[1:] if v not in ("", "-")]
            if pairs:
                out.append([f"{sec} ({sheet} sheet)", pairs])
        return out

    ARMOUR = ("helmet", "chestplate", "leggings", "boots")

    def fits(self, m, kind, slot, role):
        """`materials::fits`: whether material `m` can be `kind`'s part in `slot`."""
        row = self.mats.get(m)
        if not row or role.lower() not in [x.strip() for x in row.get("Roles", "").split(",")]:
            return False
        works = row.get("Works as", "None")
        armour = kind in self.ARMOUR
        if role == "Working":
            makes = {"Edge": not armour and kind != "bow", "Ground": kind in ("axe", "pickaxe", "shovel", "hoe"),
                     "Bone": kind not in ("sword", "bow"), "Wood": kind in ("shovel", "hoe", "spear", "bow"),
                     "Metal": kind != "bow", "Gem": kind != "bow", "Skin": armour}.get(works, False)
            return makes and (kind != "bow" or self.springs.get(m, 0) > 0)
        if role == "Holding":
            return works != "Skin" or slot == "grip"
        if role == "Fitting":
            return not (armour and m == "birch_tar")
        return True

    def built_variants(self, kind, i):
        """Pictures of a kind built of each working material (other parts the game's usual ones),
        and the main one: (main url, [{label, img, note}])."""
        slots = self.kinds.get(kind)
        if not slots:
            return None, []
        working = next(s for s, r in slots if r == "Working")
        fits = [m for m in self.mats if self.fits(m, kind, working, "Working")]
        vs, main = [], None
        for m in fits:
            parts = {}
            metal = self.mats.get(m, {}).get("Works as") in ("Metal", "Gem")
            for slot, role in slots:
                shape = self.looks.first_shape(kind, slot)
                if role == "Working":
                    mat = m
                elif role == "Holding":
                    mat = "leather" if kind == "bow" else "oak"
                elif role == "Fitting":
                    mat = "leather" if kind in ("helmet", "chestplate", "leggings", "boots") else ("sinew" if kind == "bow" else ("wrought_iron" if metal else "plant_cordage"))
                elif role == "Extra":
                    mat = "wrought_iron" if metal else "oak"
                else:
                    mat = "wool"
                parts[slot] = (shape, mat)
            if kind in ("helmet", "chestplate", "leggings", "boots"):
                look = self.looks.part_look(parts[working][0], m)
            else:
                look = self.looks.text(kind, parts)
            if not look:
                continue
            a = I.compose(look, self.tex)
            if a is None:
                continue
            url = self.img(a, f"built/{kind}_{m}.png")
            have = self.mat_from.get(m, {}).get("from", "")
            vs.append({"label": self.mat_name(m), "img": url, "note": self.mat_note(m),
                       "age": self.mat_from.get(m, {}).get("age", "")})
            if m == ("leather" if kind in ("helmet", "chestplate", "leggings", "boots") else "flint"):
                main = url
        if vs and not main:
            main = vs[0]["img"]
        return main, vs

    def gear_numbers(self, kind):
        """Damage by material from the curves (`vitals::gear`), for kinds the sheet lists."""
        k = self.prog["kinds"].get(kind)
        out = []
        if k:
            out.append(["swings a second", k.get("swings a second", "")])
            out.append(["share of the reference's damage", k.get("share", "")])
        col = kind if kind in ("sword", "axe", "spear", "knife", "pickaxe", "shovel", "hoe") else None
        if kind in ("helmet", "chestplate", "leggings", "boots"):
            col = "full armour set: armour / health"
        rows = []
        working = next((sl for sl, ro in self.kinds.get(kind, []) if ro == "Working"), "")
        if col:
            for m, r in self.prog["gear"].items():
                if m not in self.mats or not self.fits(m, kind, working, "Working"):
                    continue
                if r.get(col):
                    rows.append([f"{m.replace('_', ' ')} ({r['age']}, level {r['level']})", r[col]])
        return out, rows

    def item_entries(self):
        self.universe = S.universe(self.L)
        ids = dict(self.universe)
        for i, it in self.reg_items.items():
            if i.startswith(("core:", "testkit:")) or "tab/core:spawn_eggs" in it["comps"]:
                continue
            ids.setdefault(i, "item")
        for i, b in self.reg_blocks.items():
            if b.get("item") and not i.startswith(("core:", "testkit:")):
                ids.setdefault(i, "block")
        self.ids = ids
        log(f"{len(ids)} items and blocks")
        for i in sorted(ids):
            ns, n = i.split(":", 1)
            r = S.rule_of(self.rules, i)
            end, _ = I.shape_of(i)
            base_of = i[:-len(end)] if end else None
            if r is None and base_of:
                r = S.rule_of(self.rules, base_of)
            kind = r["kind"] if r else ("technical" if not self.reg_items.get(i) else "untagged")
            tier = r["tier"] if r else None
            e = {"id": i, "mod": ns, "name": self.name(i), "kind": kind, "is": ids[i],
                 "tier": tier, "age": AGES[tier] if tier is not None else "",
                 "path": r["path"] if r else "", "tab": self.tab_of(i),
                 "em": self.prog["em"].get(i), "note": (f"a {end.strip('_').replace('_', ' ')} of {self.name(base_of)}" if base_of else (r["note"] if r else "")),
                 "desc": self.L.get(f"item.{ns}.{n}.tooltip") or self.L.get(f"block.{ns}.{n}.tooltip") or ""}
            if r:
                g = r["get"]
                if g["how"] == "World":
                    need = g["needs"]
                    e["get"] = f"gathered in the world ({'by hand' if need == 'hand' else ('with an ' if need[0] in 'aeiou' else 'with a ') + need})"
                elif g["how"] == "Drop":
                    e["get"] = "dropped by creatures"
                elif g["how"] == "Made":
                    e["get"] = "made"
                else:
                    e["get"] = "not had as an item"
            e["made"] = self.made_by(i)
            e["uses"] = self.used_in(i)
            e["facts"] = self.facts_of(i)
            it = self.reg_items.get(i)
            if it:
                extra = []
                if it["max_stack"] != 64:
                    extra.append(["stacks to", str(it["max_stack"])])
                if it["durability"]:
                    extra.append(["durability", str(it["durability"])])
                if it["worn"]:
                    extra.append(["worn on", it["worn"]])
                tags = [c for c in it["comps"] if "/" in c and not c.startswith("tab/")]
                if tags:
                    extra.append(["tags", ", ".join(tags)])
                if extra:
                    e["facts"].insert(0, ["In the registry", extra])
            b = self.reg_blocks.get(i)
            if b:
                bx = [["model", b["model"].lower()], ["layer", b["layer"].lower()], ["hardness", str(b["hardness"])]]
                if any(b["light"]):
                    bx.append(["light", "/".join(map(str, b["light"]))])
                e["facts"].insert(0, ["As a block", bx])
            # Pictures.
            a, how = self.icon_of(i)
            e["img"] = self.img(a, f"icons/{ns}/{n}.png") if a is not None else None
            e["pix"] = how != "cube"
            if n in SHOW_KINDS and ns == "base":
                main, vs = self.built_variants(n, i)
                if main:
                    if e["img"]:
                        vs.insert(0, {"label": "its icon", "img": e["img"], "note": "the creative icon"})
                    e["img"], e["pix"], e["variants"] = main, True, vs
                    e["variants_title"] = ("Its shell in each material it takes (as the game draws a built piece's icon)" if n in self.ARMOUR else
                                           "Built of each working material, the other parts the game's usual ones (an oak handle, a cord or wrought-iron fitting)")
                nums, rows = self.gear_numbers(n)
                if nums or rows:
                    e["gear"] = {"kind": nums, "rows": rows}
            m = re.fullmatch(r"(helmet|chestplate|leggings|boots)_(mail|scale|gem)", n)
            if m and ns == "base":
                mat = {"mail": "wrought_iron", "scale": "bronze", "gem": "diamond"}[m.group(2)]
                shell = self.looks.first_shape(m.group(1), "shell")
                a2 = I.compose(self.looks.part_look(shell, mat), self.tex)
                if a2 is not None:
                    e["img"], e["pix"] = self.img(a2, f"built/{n}.png"), True
            if n.startswith("part_") and ns == "base":
                e["variants"] = self.part_variants(n[5:])
                if e["variants"]:
                    e["variants_title"] = "In each material it takes"
            e["group"] = self.group_of(i, kind, how)
            self.add(e)

    def part_variants(self, shape):
        """A loose part drawn in each material its slot takes, where it has a drawing."""
        out = []
        kinds = [(k, sl) for s, k, sl in self.shapes if s == shape]
        if not kinds:
            return out
        kind, slot = kinds[0]
        role = dict(self.kinds.get(kind, [])).get(slot, "")
        l = self.looks.looks.get(kind)
        for m, row in self.mats.items():
            if not self.fits(m, kind, slot, role):
                continue
            look = self.looks.part_look(shape, m)
            if not look and l:
                for layer in l["layers"]:
                    if slot in layer["regions"]:
                        r0 = layer["regions"][0]
                        sh = shape if r0 == slot else self.looks.first_shape(kind, r0)
                        if layer["painted"]:
                            look = f"base:parts/{kind}/{sh}/{self.looks.drawn(kind, m)}"
                        else:
                            ramps = ",".join(f"base:ramps/{m}" if r == slot else "" for r in layer["regions"])
                            look = f"base:parts/{kind}/{sh}@{ramps}"
            if not look:
                continue
            a = I.compose(look, self.tex)
            if a is None or not a[..., 3].any():
                continue
            out.append({"label": self.mat_name(m), "img": self.img(a, f"parts/{shape}_{m}.png"),
                        "note": self.mat_note(m), "age": self.mat_from.get(m, {}).get("age", "")})
        return out

    def worn_entries(self):
        """Worn armour skins, one set per shell material (`looks::worn_look`:
        `textures/entity/armor/worn/<material>_<layer>`), flat at 4x."""
        self.tex.used = set()
        for p in sorted(glob.glob(f"{self.root}/mods/base/assets/textures/entity/armor/worn/*.png")):
            stem = os.path.basename(p)[:-4]
            mat, _, layer = stem.rpartition("_")
            a = self.tex.get(f"base:entity/armor/worn/{stem}")
            if a is None:
                continue
            big = np.array(Image.fromarray(a).resize((a.shape[1] * 4, a.shape[0] * 4), Image.NEAREST))
            self.add({"id": f"base:armor/worn/{stem}", "mod": "base", "name": f"Worn armour: {self.mat_name(mat)} {'outer' if layer == '1' else 'leggings'} layer",
                      "is": "skin", "group": "armour", "kind": "worn skin", "tier": None, "age": "", "path": "tech", "tab": "",
                      "img": self.img(big, f"worn/{stem}.png"), "pix": True, "wide": True,
                      "note": f"{self.mat_name(mat)} shells as they're worn: layer {layer} ({'helmet, chestplate, boots' if layer == '1' else 'leggings'})",
                      "made": [], "uses": [], "facts": []})

    # ------------------------------------------------------------ recipes as screens

    STATION_NAMES = {"hand": "By hand (the inventory's 2 × 2)", "table": "Crafting table", "furnace": "Furnace",
                     "parts": "Crafting table: parts", "assembly": "Crafting table: assembly row",
                     "knapping": "Knapping (a flint nodule struck with a rock)", "fire": "Over a fire (campfire or furnace)",
                     "world": "In the world", "fill": "Filled at water"}

    def screens(self):
        """Every way each thing is made, as the game's screens draw it: {output id: [screen]}.
        A screen: {st (station), name, w, h, cells [[id, n, any] or None], out [id, n, chance], secs,
        slots (the assembly's slot names), outline (knapping)}."""
        out = {}
        have = set()

        def add(o, scr):
            out.setdefault(o, []).append(scr)

        for r in self.reg.get("recipes", []):
            if r.get("hidden") or not r["out"]:
                continue
            o, n, ch = r["out"][0]
            cells = []
            for kind, count, shown in r["in"]:
                cells.append(None if kind == 0 or not shown else [shown, count, kind in (2, 3)])
            if r["kind"] == "base:crafting":
                w, h = r["w"], r["h"]
                if not w:
                    k = len(cells)
                    w, h = (2, 2) if k <= 4 else (3, 3)
                st = "hand" if w <= 2 and h <= 2 else "table"
                # In the game's own grid (2 x 2 by hand, 3 x 3 at the table), from its top left.
                g = 2 if st == "hand" else 3
                full = [None] * (g * g)
                if r["w"]:
                    for k, c in enumerate(cells):
                        full[(k // w) * g + k % w] = c
                else:
                    full[:len(cells)] = cells
                cells, w, h = full, g, g
            elif r["kind"] == "base:smelting":
                st, w, h = "furnace", 1, 1
            else:
                st, w, h = "parts", len(cells), 1
            add(o, {"st": st, "w": w, "h": h, "cells": cells, "out": [o, n, ch], "secs": r["time"] / 20 if r["time"] else None})
            have.add(o)
        kinds_out = {f"base:{k}" for k in self.kinds}
        for r, (ro, raw, secs) in zip(self.recipes, self.raw_recipes):
            o = r["out"]
            if "*" in o:
                continue
            cells = [[x, n, x.startswith("#")] for x, n in r["ins"]]
            for c in cells:
                if c[2]:
                    c[0] = self.tag_example(c[0][1:]) or c[0]
            if raw.startswith("Station"):
                st = re.search(r'"([^"]+)"', raw).group(1)
            elif raw.startswith("Break"):
                st = "world"
            elif raw == "Fill":
                st = "fill"
            elif raw == "Fire":
                st = "fire"
            elif o in kinds_out and raw == "Table":
                k = o.split(":")[1]
                def slot_of(c):
                    shape = c[0].split(":")[-1].removeprefix("part_")
                    return next((sl for sh, kk, sl in self.shapes if sh == shape and kk == k), "")
                add(o, {"st": "assembly", "w": len(cells), "h": 1, "cells": cells, "out": [o, r["n"], 1], "secs": None,
                        "slots": [slot_of(c) for c in cells]})
                continue
            elif o in have:
                continue
            else:
                st = "hand" if raw == "Hand" else ("table" if raw == "Table" else "furnace")
            w = len(cells) if st not in ("hand", "table") else (2 if st == "hand" else 3)
            h = 1 if st not in ("hand", "table") else max(1, -(-len(cells) // w))
            add(o, {"st": st, "w": w, "h": h, "cells": cells, "out": [o, r["n"], 1], "secs": secs, "needs": raw[6:-1].replace("_", " ").lower() if st == "world" else None})
        for shape, kind, outline in S.knapped(self.root):
            add(f"base:part_{shape}", {"st": "knapping", "w": 5, "h": 5, "cells": [], "outline": outline,
                                       "out": [f"base:part_{shape}", 1, 1], "secs": None})
        return out

    GUI = ["panel", "slot", "recipe_slot", "arrow", "arrow_fill", "flame", "flame_fill", "slot_part_head", "slot_part_haft",
           "slot_part_fitting", "slot_part_tool", "knap_stone", "knap_free", "knap_flint", "well"]

    def gui(self):
        """The game's screen parts the page draws recipes with."""
        self.gui_urls = {}
        for n in self.GUI:
            p = self.tex.path(f"base:gui/{n}")
            if p:
                a = np.array(Image.open(p).convert("RGBA"))
                self.gui_urls[n] = self.img(a, f"gui/{n}.png")

    def tag_example(self, tag):
        """An item carrying `tag` (the dump's tags), to show in a slot."""
        for i, it in sorted(self.reg_items.items()):
            if tag in it.get("comps", []) and not i.startswith("core:"):
                return i
        return None

    def attach_screens(self):
        scr = self.screens()
        by = {e["id"]: e for e in self.entries}
        station_of = {"table": "base:crafting_table", "parts": "base:crafting_table", "assembly": "base:crafting_table",
                      "hand": "base:crafting_table", "furnace": "base:furnace", "fire": "base:campfire_laid"}
        makes = {}
        for o, ss in scr.items():
            if o in by:
                by[o]["screens"] = ss
            for x in ss:
                for c in x["cells"]:
                    if c and c[0] in by and o in by and by[o]["name"] not in by[c[0]]["uses"]:
                        by[c[0]]["uses"].append(by[o]["name"])
                home = station_of.get(x["st"], x["st"])
                makes.setdefault(home, []).append(x)
        for home, ss in makes.items():
            if home in by:
                by[home]["makes"] = ss
        self.screen_count = sum(len(v) for v in scr.values())

    # ------------------------------------------------------------ skills and the perk web

    def skill_entries(self):
        md = open(f"{self.root}/mods/base/vitals/WEB_SHEET.md").read()
        T = {(h[-1] if h else ""): (head, rows) for h, head, rows in S.tables(md)}
        skills = T.get("Skills", ([], []))
        regions = {r[0]: dict(zip(T["Regions"][0], r)) for r in T.get("Regions", ([], []))[1]}
        icon = self.tex.get("base:gui/perk_notable_open")
        key_icon = self.tex.get("base:gui/perk_keystone_open")
        sk_url = self.img(icon, "gui/perk_notable_open.png") if icon is not None else None
        web_url = self.img(key_icon, "gui/perk_keystone_open.png") if key_icon is not None else None
        ages = T.get("What each age's levels cost", ([], []))
        pays = T.get("What doing pays (EM, before tier against skill)", ([], []))
        for r in skills[1]:
            row = dict(zip(skills[0], r))
            name = row["Skill"]
            reg = regions.get(name, {})
            facts = [["The skill", [["group", row.get("Group", "")], ["path", row.get("Path", "")], ["raised by", row.get("Raised by", "")]]]]
            if reg:
                facts.append(["Its region of the perk web", [[k, v] for k, v in reg.items() if k != "Region"]])
            facts.append(["Its levels by age", [[a[0], f"levels {a[1]}, {a[2]} EM a level, {a[4]} hours from 0"] for a in ages[1]]])
            self.tex.used = set()
            self.add({"id": "skill:" + re.sub(r"\W+", "_", name.lower()).strip("_"), "mod": "base", "name": name, "is": "skill",
                      "group": "skills", "kind": row.get("Group", "").lower(), "tier": None, "age": "", "path": row.get("Path", ""),
                      "tab": "", "img": sk_url, "pix": True, "note": "raised by " + row.get("Raised by", ""), "made": [], "uses": [],
                      "facts": facts, "region": next((k["id"] for k in self.reg.get("skills", []) if k["name"] == name), None)})
        web = self.reg.get("web")
        facts = []
        for h in ("Families", "Sockets and stones"):
            if h in T:
                head, rows = T[h]
                facts.append([h, [[r[0], " · ".join(f"{a}: {b}" for a, b in zip(head[1:], r[1:]))] for r in rows]])
        self.tex.used = set()
        self.add({"id": "skill:perk_web", "mod": "base", "name": "The perk web", "is": "web", "group": "skills", "kind": "web",
                  "tier": None, "age": "", "path": "shared", "tab": "", "img": web_url, "pix": True,
                  "desc": re.sub(r"\s+", " ", md[md.index("## The web") + 10:].strip().split("\n\n")[0]) if "## The web" in md else "",
                  "web": bool(web), "made": [], "uses": [], "facts": facts})
        if web:
            names = {}
            for n in web["nodes"]:
                k = n["key"]
                names[k] = self.L.get(f"perk.base.{k}") or k.split(".")[-1].replace("_", " ").capitalize()
            fam = {k.split(".")[-1]: v for k, v in self.L.items() if k.startswith("perk.base.family.")}
            self.web = {"nodes": web["nodes"], "links": web["links"], "names": names, "families": fam,
                        "sprites": {k: self.img(self.tex.get(f"base:gui/perk_{k}_open"), f"gui/perk_{k}_open.png")
                                    for k in ("small", "notable", "keystone", "socket") if self.tex.get(f"base:gui/perk_{k}_open") is not None},
                        "start": self.img(self.tex.get("base:gui/perk_start_taken"), "gui/perk_start_taken.png")
                        if self.tex.get("base:gui/perk_start_taken") is not None else None}
        # The stones' own rows.
        if "Sockets and stones" in T:
            head, rows = T["Sockets and stones"]
            by = {e["id"]: e for e in self.entries}
            for r in rows:
                e = by.get(f"base:{r[0]}")
                if e:
                    e["facts"].insert(0, ["In a socket of the perk web", [[head[1], r[1]]]])

    # ------------------------------------------------------------ features

    SYSTEMS = "notes/crew/systems-map/systems.json"

    def feature_entries(self):
        """A card per system of Planner's map (`notes/crew/systems-map/systems.json`, in the tree
        exported or the main checkout's; a site only the tree's), with `features.json`'s how-to,
        its sheet tables and the things it links to."""
        p = next((x for x in [os.path.join(self.root, self.SYSTEMS)] + ([] if self.site else [os.path.join(MAIN, self.SYSTEMS)])
                  if os.path.exists(x)), None)
        if not p:
            log("no systems map: no features")
            return
        sysmap = json.load(open(p))
        groups = {g["id"]: g["name"] for g in sysmap.get("groups", [])}
        extra = json.load(open(os.path.join(HERE, "features.json")))
        sheets = {"survival": f"{self.root}/mods/base/vitals/SHEET.md", "crafting": f"{self.root}/mods/base/crafting/SHEET.md",
                  "progression": f"{self.root}/mods/base/progression/PROGRESSION_SHEET.md"}
        tabs = {k: S.tables(open(v).read()) for k, v in sheets.items() if os.path.exists(v)}
        ids = [e["id"] for e in self.entries]
        names = {e["id"]: e["name"] for e in self.entries}
        icon = self.tex.get("base:gui/hud_stats")
        url = self.img(icon, "gui/hud_stats.png") if icon is not None else None
        for x in sysmap.get("systems", []):
            f = extra.get(x["id"], {})
            facts = [["The system", [["status", x.get("status", "")], ["code", x.get("code", "")],
                                     ["feeds", ", ".join(x.get("feeds", []))]]]]
            for h in f.get("sheets", []):
                for heads, head, rows in tabs.get(f.get("sheet", "survival"), []):
                    if heads and heads[-1] == h:
                        facts.append([f"{h} (numbers)", [[r[0].strip("`"), " · ".join(f"{a}: {b}" for a, b in zip(head[1:], r[1:]) if b)] for r in rows[:40]]])
                        break
            links = [i for i in f.get("links", []) if i in names]
            for w in f.get("match", []):
                links += [i for i in ids if w in i.split(":", 1)[-1] and i not in links and not i.startswith(("skill:", "feature:"))]
            self.tex.used = set()
            first = next((e for e in self.entries if links and e["id"] == links[0]), None)
            self.add({"id": f"feature:{x['id']}", "mod": "", "name": x["name"], "is": "feature", "group": "features",
                      "kind": groups.get(x.get("group"), x.get("group", "")), "status": x.get("status", ""), "tier": None, "age": "",
                      "path": "", "tab": "", "img": first["img"] if first and first.get("img") else url,
                      "pix": first["pix"] if first and first.get("img") else True, "desc": x.get("what", ""), "how": f.get("how", ""),
                      "links": links[:80], "made": [], "uses": [], "facts": facts})

    # ------------------------------------------------------------ creatures

    def render_mod(self):
        if not hasattr(self, "_render"):
            sys.path.insert(0, f"{self.root}/.claude/skills/blocky-model/scripts")
            import render  # noqa: E402  the skill's own renderer
            self._render = render
        return self._render

    def model_pictures(self, model, texture, key):
        geo_p = glob.glob(f"{self.root}/mods/*/assets/models/{model}.geo.json")
        if not geo_p or not os.path.exists(texture):
            return None, None
        geo = json.load(open(geo_p[0]))
        anim = None
        ap = glob.glob(f"{self.root}/mods/*/assets/animations/{model}.animation.json")
        if ap:
            anims = json.load(open(ap[0])).get("animations", {})
            anim = next((v for k, v in anims.items() if k.endswith(".idle")), None)
        return self.render_geo(geo, anim, texture, key, model)

    def render_geo(self, geo, anim, texture, key, model=""):
        out = []
        if texture:
            self.tex.used.add(texture)
        for g in glob.glob(f"{self.root}/mods/*/assets/models/{model}.geo.json") if model else []:
            self.tex.used.add(g)
        for view, box in (("3q", 320), ("side", 480)):
            rel = f"models/{key}_{view}.png"
            p = os.path.join(self.outdir, rel)
            if os.path.exists(p):
                out.append(self.prefix + rel)
                continue
            spec, pose = I.geo_to_spec(geo, anim)
            try:
                a = I.render_model(self.render_mod(), spec, texture, pose, view, box)
            except Exception as ex:  # a model the converter can't read yet
                log(f"render {model} {view}: {ex}")
                return None, None
            out.append(self.img(I.trim(a), rel))
        return out[0], out[1]

    def creature_entries(self):
        sp_rows = self.prog["species"]
        for sp in S.species_sheet(self.root):
            key, model = sp["key"], sp["model"]
            ent = sp["entity"]
            ns, en = ent.split(":")
            name = self.L.get(f"entity.{ns}.{en}") or self.L.get(f"entity.{ns}.{key}") or title_case(key)
            tex_dir = f"{self.root}/mods/animals/assets/textures/entity/{model}"
            texture = next((p for p in (f"{tex_dir}/{key}.png", f"{tex_dir}/{en}.png", f"{tex_dir}/{model}.png") if os.path.exists(p)), "")
            img, side = self.model_pictures(model, texture, key)
            g = sp["glance"]
            row = sp_rows.get(key, {})
            danger = []
            if row:
                danger = [["size class (fights)", row.get("size", "")], ["role", row.get("role", "")],
                          ["health / armour / hit at level 0", row.get("health / armour / hit (level 0)", "")],
                          ["health at level 100 / 200", row.get("health at 100 / 200", "")]]
                if row.get("tweaks"):
                    danger.append(["tweak", row["tweaks"]])
            facts = [["Danger (animals_core::danger, the curves)", danger]] if danger else []
            clean = lambda v: re.sub(r"\*\*?([^*]+)\*\*?", r"\1", v).replace("`", "")
            facts.append(["At a glance", [[k, clean(v)] for k, v in g.items() if k != "Species" and v]])
            facts.append(["Body, life, food and the wild", [[clean(k), clean(v)] for k, v in sp["facts"].items()]])
            clips = S.clips(self.root, model)
            e = {"id": ent, "mod": ns, "name": name, "is": "creature", "group": "animals",
                 "kind": "beast", "tier": 0, "age": "", "path": "beasts", "tab": "",
                 "img": img, "img_side": side, "pix": False, "model": model,
                 "size": g.get("Size", ""), "role": row.get("role", ""), "diet": g.get("Diet", ""),
                 "taming": g.get("Taming", ""), "tame": g.get("Taming", "") not in ("wild", ""),
                 "rarity": g.get("Rarity", ""), "biomes": [b for b, _ in sp["biomes"]],
                 "danger": row.get("health / armour / hit (level 0)", ""),
                 "sci": sp["scientific"], "lineage": sp["lineage"], "spawns": sp.get("spawns", ""),
                 "clips": clips, "facts": facts, "made": [], "uses": [], "species": key}
            self.add(e)
        # Drenkaun, the first biome boss.
        tex = f"{self.root}/mods/animals/assets/textures/entity/drenkaun/drenkaun.png"
        img, side = self.model_pictures("drenkaun", tex, "drenkaun")
        rows = [b for b in self.prog["bosses"] if b.get("boss") == "drenkaun"]
        facts = []
        if rows:
            facts.append(["By age (the curves: a titan at its form's age)",
                          [[f"{b['form']} (level {b['level']}, offering {b['offering']})",
                            f"health {b['health']}, armour {b['armour']}, hit {b['hit']}; to kill / hits to down a player of each age: Stone {b['Stone']}, Bronze {b['Bronze']}, Iron {b['Iron']}, Steam {b['Steam']}"] for b in rows]])
        note = self.prog["boss_notes"].get("drenkaun", "")
        if note:
            facts.append(["The fight", [[p.split(" ")[0], p] for p in re.split(r";\s*", note)]])
        self.add({
            "id": "animals:drenkaun", "mod": "animals", "name": self.L.get("entity.animals.drenkaun.title", "Drenkaun"),
            "is": "creature", "group": "monsters", "kind": "boss", "tier": 0, "age": "Stone Age", "path": "shared", "tab": "",
            "img": img, "img_side": side, "pix": False, "model": "drenkaun", "size": "titan", "role": "apex (boss)",
            "diet": "", "taming": "its calf, from the offering", "tame": True, "rarity": "a biome boss, called at its altar",
            "biomes": ["old forests (its altar among old trees)"], "danger": rows[0]["health"] + " health" if rows else "",
            "clips": S.clips(self.root, "drenkaun"), "facts": facts, "made": [], "uses": [],
            "desc": self.L.get("block.animals.drenkaun_altar.tooltip", "")})
        # The base game's mobs (vanilla's boxes, built in code: `mobs.py` mirrors them).
        for ent in sorted(HOSTILE | FARM):
            ns, n = ent.split(":")
            texture = self.tex.path(f"{ns}:entity/{n}") or ""
            geo = mobs.geometry(n)
            img, side = self.render_geo(geo, None, texture, n, n) if geo and texture else (None, None)
            if img is None:
                a = self.tex.get(f"{ns}:entity/{n}")
                img = self.img(a, f"skins/{n}.png") if a is not None else None
            self.add({
                "id": ent, "mod": ns, "name": self.L.get(f"entity.{ns}.{n}", title_case(ent)), "is": "creature",
                "group": "monsters" if ent in HOSTILE else "animals", "kind": "monster" if ent in HOSTILE else "farm animal",
                "tier": 0, "age": "", "path": "shared", "tab": "", "img": img, "img_side": side, "pix": False,
                "size": "", "role": "hostile" if ent in HOSTILE else "farm", "diet": "", "taming": "", "tame": False,
                "biomes": [], "clips": {}, "facts": [["Model", [["drawn", "vanilla-style boxes built in code (mods/base/src/mobs.rs)"]]]],
                "made": [], "uses": []})

    # ------------------------------------------------------------ page

    def page(self, subj):
        tpl = open(os.path.join(HERE, "page.html")).read()
        data = {"commit": self.commit, "subject": subj, "built": time.strftime("%Y-%m-%d %H:%M"),
                "gui": getattr(self, "gui_urls", {}), "stations": self.STATION_NAMES, "web": getattr(self, "web", None),
                "ages": AGES, "entries": self.entries,
                "registry": os.path.basename(self.registry) if self.registry else "", "site": self.prefix == "img/"}
        js = json.dumps(data, separators=(",", ":"), ensure_ascii=False).replace("</", "<\\/")
        return tpl.replace("/*DATA*/null", js)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--rev", default="dev")
    ap.add_argument("--registry", help="a /tk dump JSON (default: the newest in the archive)")
    ap.add_argument("--page", default=OUT_PAGE)
    ap.add_argument("--fresh", action="store_true", help="redraw every picture")
    ap.add_argument("--out", metavar="DIR", help="write a static site instead: DIR/index.html and DIR/img/ (relative links); "
                    "reads nothing outside the repository")
    ap.add_argument("--fetch-dump", metavar="JOB", help="first copy the registry a PC job's `/tk dump` wrote "
                    "(t2q.py look JOB @dev notes/crew/tester-tools/t2/scripts/t2_catalogue_dump.txt <world> --fresh)")
    ap.add_argument("--dump-commit", help="with --fetch-dump: the commit the job ran (default: --rev's)")
    ap.add_argument("--strict", action="store_true", help="fail when an entry has no picture")
    a = ap.parse_args()
    if a.fetch_dump:
        d = os.path.join(ARCHIVE, "catalogue", "registry")
        os.makedirs(d, exist_ok=True)
        dst = os.path.join(d, f"{a.fetch_dump}.json")
        subprocess.run(["scp", "-q", f"{PC}:looktest/t2_{a.fetch_dump}/registry.json", dst], check=True)
        log(f"registry {dst}")
        # The snapshot the site builds with (commit it): named by the commit the job ran.
        reg = json.load(open(dst))
        ran = (a.dump_commit or subprocess.check_output(["git", "-C", REPO, "rev-parse", "--short=8", a.rev], text=True).strip())
        reg["_made"] = {"commit": ran, "job": a.fetch_dump, "time": int(time.time())}
        os.makedirs(SNAPSHOTS, exist_ok=True)
        for old in glob.glob(os.path.join(SNAPSHOTS, "*.json")):
            os.remove(old)
        with open(os.path.join(SNAPSHOTS, f"{ran}.json"), "w") as f:
            json.dump(reg, f, separators=(",", ":"))
        log(f"snapshot {SNAPSHOTS}/{ran}.json (commit it)")
    t0 = time.time()
    root, commit, subj, full = export(a.rev)
    try:
        site = a.out is not None
        outdir = os.path.join(a.out, "img") if site else os.path.join(ARCHIVE, "catalogue", commit)
        if (a.fresh or site) and os.path.isdir(outdir):
            shutil.rmtree(outdir)
        os.makedirs(outdir, exist_ok=True)
        reg = a.registry or newest_registry(commit, archive=not site)
        log(f"commit {commit} ({subj[:60]}), registry {reg}")
        b = Builder(root, commit, outdir, reg, prefix="img/" if site else None)
        b.site = site
        b.registry = reg
        b.dates = art_dates(full)
        b.item_entries()
        b.worn_entries()
        b.creature_entries()
        b.skill_entries()
        b.feature_entries()
        by = {e["id"]: e for e in b.entries}
        for e in b.entries:
            for l in e.get("links", []):
                if l in by and l != e["id"]:
                    by[l].setdefault("features", []).append(e["id"])
        b.attach_screens()
        b.gui()
        html = b.page(subj)
        page = os.path.join(a.out, "index.html") if site else a.page
        os.makedirs(os.path.dirname(os.path.abspath(page)), exist_ok=True)
        with open(page, "w") as f:
            f.write(html)
        if not site:
            shutil.copyfile(page, os.path.join(outdir, "index.html"))
        with open(os.path.join(outdir, "data.json"), "w") as f:
            json.dump(b.entries, f, indent=0, ensure_ascii=False)
        no_pic = [e["id"] for e in b.entries if not e.get("img")]
        log(f"{len(b.entries)} entries, {b.made} pictures drawn, {len(no_pic)} without a picture: {' '.join(no_pic)}")
        log(f"page {page}, pictures {outdir}, {time.time() - t0:.0f} s")
        if site and no_pic and a.strict:
            sys.exit(f"{len(no_pic)} entries without a picture")
    finally:
        shutil.rmtree(root, ignore_errors=True)


if __name__ == "__main__":
    main()
