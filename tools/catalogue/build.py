#!/usr/bin/env python3
"""The crew gallery's catalogue: every item, block, part, gem, plant and creature of the game at a
commit, with its picture and what the sources know of it, as one filterable page.

    python3 tools/catalogue/build.py                 # dev's tip
    python3 tools/catalogue/build.py --rev <commit>  # any commit

It reads the sources at that commit (exported with `git archive`, so the checkout it runs from
does not matter): the lang files, `mods/base/progression` (rules, recipes and its sheet), the
vitals, crafting and species sheets, the textures and the box models. The registry the game
itself lists (creative tabs, each block's face textures) comes from the newest `/tk dump`
(`~/Coding/chiacraft-archive/images/catalogue/registry/<commit>.json`, made by a PC run of
`tester-tools/t2/t2_catalogue_dump.txt`, fetched with `--fetch-dump <job>`); things the dump
hasn't seen still show, from the sources alone. Never runs the game.

Writes the pictures to `~/Coding/chiacraft-archive/images/catalogue/<commit>/` and the page to
`notes/crew/gallery/catalogue/index.html` in the main checkout (the gallery serves it at
/catalogue).
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

REPO = os.path.expanduser("~/Coding/chiacraft-weft")
ARCHIVE = os.path.expanduser("~/Coding/chiacraft-archive/images")
OUT_PAGE = os.path.join(REPO, "notes/crew/gallery/catalogue/index.html")
PC = "ronaldg@192.168.1.13"

PATHS = ["mods", "crates/engine/assets/lang", "crates/engine/assets/textures", ".claude/skills/blocky-model/scripts"]
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
    data = subprocess.run(["git", "-C", REPO, "archive", "--format=tar", full, "--"] + PATHS, capture_output=True, check=True).stdout
    with tarfile.open(fileobj=io.BytesIO(data)) as t:
        members = [m for m in t.getmembers() if not any(s in "/" + m.name for s in SKIP) and not m.name.startswith("/") and ".." not in m.name]
        t.extractall(tmp, members=members, filter="data")
    subj = subprocess.check_output(["git", "-C", REPO, "log", "-1", "--format=%cd %s", "--date=format:%Y-%m-%d %H:%M", full], text=True).strip()
    return tmp, commit, subj


def newest_registry(commit):
    d = os.path.join(ARCHIVE, "catalogue", "registry")
    exact = os.path.join(d, f"{commit}.json")
    if os.path.exists(exact):
        return exact
    files = sorted(glob.glob(os.path.join(d, "*.json")), key=os.path.getmtime)
    return files[-1] if files else None


def title_case(i):
    return i.split(":")[-1].replace("_", " ").capitalize()


class Builder:
    def __init__(self, root, commit, outdir, registry):
        self.root, self.commit, self.outdir = root, commit, outdir
        self.tex = I.Textures(root)
        self.L = S.lang(root)
        self.rules = S.rules(root)
        self.recipes = S.recipes(root)
        self.prog = S.progression_sheet(root)
        self.rows = S.sheet_rows(root)
        self.mats, self.gems = S.crafting_sheet(root)
        self.springs = S.springs(root)
        kinds, shapes = S.crafting_kinds(root)
        looks, maps, spots = S.looks(root)
        metals = {m for m, r in self.mats.items() if r.get("Works as") in ("Metal",)} | {"gold"}
        self.kinds, self.shapes = kinds, shapes
        self.looks = I.Looks(looks, kinds, shapes, maps, metals)
        self.overrides = S.texture_overrides(root)
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
        return f"/img/catalogue/{self.commit}/{rel}"

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
        b = self.reg_blocks.get(i)
        if b:
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
            self.entries.append(e)

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
            look = None
            if shape in self.looks.part_maps:
                look = self.looks.part_look(shape, m)
            elif l:
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
        for view, box in (("3q", 320), ("side", 480)):
            rel = f"models/{key}_{view}.png"
            p = os.path.join(self.outdir, rel)
            if os.path.exists(p):
                out.append(f"/img/catalogue/{self.commit}/{rel}")
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
            self.entries.append(e)
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
        self.entries.append({
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
            self.entries.append({
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
                "ages": AGES, "entries": self.entries,
                "registry": os.path.basename(self.registry) if self.registry else ""}
        js = json.dumps(data, separators=(",", ":"), ensure_ascii=False).replace("</", "<\\/")
        return tpl.replace("/*DATA*/null", js)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--rev", default="dev")
    ap.add_argument("--registry", help="a /tk dump JSON (default: the newest in the archive)")
    ap.add_argument("--page", default=OUT_PAGE)
    ap.add_argument("--fresh", action="store_true", help="redraw every picture")
    ap.add_argument("--fetch-dump", metavar="JOB", help="first copy the registry a PC job's `/tk dump` wrote "
                    "(t2q.py look JOB @dev ~/Coding/chiacraft-archive/tester-tools/t2/t2_catalogue_dump.txt <world>)")
    a = ap.parse_args()
    if a.fetch_dump:
        d = os.path.join(ARCHIVE, "catalogue", "registry")
        os.makedirs(d, exist_ok=True)
        dst = os.path.join(d, f"{a.fetch_dump}.json")
        subprocess.run(["scp", "-q", f"{PC}:looktest/t2_{a.fetch_dump}/registry.json", dst], check=True)
        log(f"registry {dst}")
    t0 = time.time()
    root, commit, subj = export(a.rev)
    try:
        outdir = os.path.join(ARCHIVE, "catalogue", commit)
        if a.fresh and os.path.isdir(outdir):
            shutil.rmtree(outdir)
        os.makedirs(outdir, exist_ok=True)
        reg = a.registry or newest_registry(commit)
        log(f"commit {commit} ({subj[:60]}), registry {reg}")
        b = Builder(root, commit, outdir, reg)
        b.registry = reg
        b.item_entries()
        b.creature_entries()
        html = b.page(subj)
        os.makedirs(os.path.dirname(a.page), exist_ok=True)
        with open(a.page, "w") as f:
            f.write(html)
        shutil.copyfile(a.page, os.path.join(outdir, "index.html"))
        with open(os.path.join(outdir, "data.json"), "w") as f:
            json.dump(b.entries, f, indent=0, ensure_ascii=False)
        no_pic = [e["id"] for e in b.entries if not e.get("img")]
        log(f"{len(b.entries)} entries, {b.made} pictures drawn, {len(no_pic)} without a picture: {' '.join(no_pic)}")
        log(f"page {a.page}, pictures {outdir}, {time.time() - t0:.0f} s")
    finally:
        shutil.rmtree(root, ignore_errors=True)


if __name__ == "__main__":
    main()
