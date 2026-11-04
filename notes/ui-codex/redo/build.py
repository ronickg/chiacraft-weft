"""The Item Icon Redo page: every batch made so far (out/), each icon as generated, its exact 32 x 32
sprite in the game's slots, today's game icon beside it, and how the checks went; a hotbar of the
batch new against old on top.

    ~/.venvs/blocky/bin/python notes/ui-codex/redo/build.py
"""
import base64
import html
import io
import json
import os
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
OUT = os.path.join(HERE, "out")
TITLES = {"1_tools": "Tools and weapons", "2_armour": "Armour", "3_materials": "Materials, ores and gems",
          "4_food": "Food", "5_plants": "Plants, nuts, berries and seeds", "6_misc": "Containers, tools, creatures, eggs",
          "7_survival": "Survival's new items (for its branch)",
          "2b_redo": "Redos: hoes, leather cap, boots",
          "3b_redo": "Redos: coal, gunpowder, stick, three hoes, five foods",
          "5b_hoes": "Hoes, third try",
          "5c_wild": "New wild foods (for the Voxel dog handoff)",
          "5d_redo": "Redos: crowberries, beans, barley, rye, melon seeds, diamond hoe"}


def uri(im, scale=1):
    if scale != 1:
        im = im.resize((im.width * scale, im.height * scale), Image.NEAREST)
    b = io.BytesIO()
    im.save(b, "PNG", optimize=True)
    return "data:image/png;base64," + base64.b64encode(b.getvalue()).decode()


def thumb(path, size=256):
    im = Image.open(path).convert("RGB")
    im.thumbnail((size, size), Image.NEAREST)
    return uri(im)


def game_icon(name, mod):
    p = os.path.join(ROOT, "mods", mod, "assets", "textures", "item", name + ".png")
    if not os.path.exists(p):  # Survival's new items live on its branch
        p = os.path.join(ROOT, "mods", "base", "assets", "textures", "item", name + ".png")
    return Image.open(p).convert("RGBA") if os.path.exists(p) else None


def card(job):
    n = job["name"]
    raw, px = os.path.join(OUT, n + ".png"), os.path.join(OUT, n + "_px.png")
    if not os.path.exists(px) or not os.path.exists(raw):
        return f'<article class="card" id="{n}"><header><h3>{n.replace("_", " ")}</h3></header><p class="checks">{'being remade' if os.path.exists(px) else 'not made yet'}</p></article>'
    sp = Image.open(px).convert("RGBA")
    info = json.load(open(os.path.join(OUT, n + "_checks.json")))
    tries = info["tries"]
    last = tries[-1] if tries else {}
    made = [t for t in tries if t.get("made", True)]
    verdict = (("passed the checks" + (f" on try {len(tries)}" if len(tries) > 1 else "")) if last.get("passed")
               else f"failed the checks {len(made)} times; the last try is shown")
    nums = ", ".join(f"{k} {last[k]}" for k in ("grid", "outline", "red_line") if k in last)
    old = game_icon(n, job.get("mod", "base"))
    old_html = (f'<div class="pic slot"><img src="{uri(old, max(1, 256 // old.width))}" alt=""></div>' if old else
                '<div class="pic none">new item</div>')
    flag = "" if last.get("passed") else " fail"
    return f"""<article class="card{flag}" id="{n}">
  <header><h3>{html.escape(n.replace('_', ' '))}</h3><span class="tag">{'green' if job.get('green') else ''}</span></header>
  <div class="trio">
    <figure><div class="pic raw{' green' if info.get('green') or job.get('green') else ''}"><img src="{thumb(raw)}" alt=""></div><figcaption>as generated</figcaption></figure>
    <figure><div class="pic slot"><img src="{uri(sp, 8)}" alt=""></div><figcaption>new, 32 × 32</figcaption></figure>
    <figure>{old_html}<figcaption>game now</figcaption></figure>
  </div>
  <div class="slots">{''.join(f'<span class="slot"><img src="{uri(sp, s)}" width="{16 * s}" height="{16 * s}" alt=""></span>' for s in (2, 3, 4))}</div>
  <p class="checks">{verdict}{(': ' + nums) if nums else ''}</p>
</article>"""


def sheet(batch, jobs):
    """A PNG of the batch to see inline: per item its name, today's icon and the new one at 4x on the
    slot color, in rows of four pairs; sheets/<batch>.png."""
    from PIL import ImageDraw, ImageFont
    made = [j for j in jobs if os.path.exists(os.path.join(OUT, j["name"] + "_px.png"))]
    if not made:
        return None
    S, cols, pad, lab = 128, 4, 16, 22
    cw = 2 * S + 3 * pad
    rows = (len(made) + cols - 1) // cols
    im = Image.new("RGB", (cols * cw + pad, rows * (S + lab + 2 * pad) + 44 + pad), (0x1b, 0x13, 0x10))
    d = ImageDraw.Draw(im)
    try:
        font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 16)
        big = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial Bold.ttf", 22)
    except OSError:
        font = big = ImageFont.load_default()
    d.text((pad, 12), f"{TITLES.get(batch, batch)}: left the game now, right new (32 x 32 at 4x)", fill=(0xf5, 0xe4, 0xbc), font=big)
    for k, j in enumerate(made):
        x0 = pad + (k % cols) * cw
        y0 = 44 + pad + (k // cols) * (S + lab + 2 * pad)
        fail = not (json.load(open(os.path.join(OUT, j["name"] + "_checks.json")))["tries"] or [{}])[-1].get("passed")
        d.text((x0, y0), j["name"].replace("_", " ") + ("  (failed checks)" if fail else ""),
               fill=(0xe0, 0x57, 0x4a) if fail else (0xc2, 0xac, 0x88), font=font)
        for c, icon in enumerate((game_icon(j["name"], j.get("mod", "base")),
                                  Image.open(os.path.join(OUT, j["name"] + "_px.png")).convert("RGBA"))):
            bx = x0 + c * (S + pad)
            d.rectangle([bx, y0 + lab, bx + S - 1, y0 + lab + S - 1], fill=(0x35, 0x20, 0x15), outline=(0x68, 0x3e, 0x26))
            if icon is not None:
                big_icon = icon.resize((S - 16, S - 16), Image.NEAREST)
                im.paste(big_icon, (bx + 8, y0 + lab + 8), big_icon)
            else:
                d.text((bx + 30, y0 + lab + S // 2 - 8), "new item", fill=(0xc2, 0xac, 0x88), font=font)
    os.makedirs(os.path.join(HERE, "sheets"), exist_ok=True)
    path = os.path.join(HERE, "sheets", batch + ".png")
    b = io.BytesIO()
    im.save(b, "PNG")
    if os.path.exists(path) and open(path, "rb").read() != b.getvalue():
        # a sheet Ronald may have seen: keep the old one for good before it changes (his rule, 11-04)
        sys.path.insert(0, os.path.join(ROOT, ".claude", "skills", "game-art", "scripts"))
        import art
        art.archive(path, f"sheet_{batch}", "sheets")
    open(path, "wb").write(b.getvalue())
    return path


def main():
    sections, total, done, failed = [], 0, 0, []
    for b in sorted(f[:-5] for f in os.listdir(os.path.join(HERE, "batches")) if f.endswith(".json")):
        jobs = json.load(open(os.path.join(HERE, "batches", b + ".json")))
        made = [j for j in jobs if os.path.exists(os.path.join(OUT, j["name"] + "_px.png"))]
        if not b.endswith("_redo"):  # redos remake icons already counted in their own batch
            total += len(jobs)
            done += len(made)
        if not made:
            continue
        print("sheet:", sheet(b, jobs))
        for j in made:
            t = json.load(open(os.path.join(OUT, j["name"] + "_checks.json")))["tries"]
            if not (t and t[-1].get("passed")) and j["name"] not in failed:
                failed.append(j["name"])
        hot_new = "".join(f'<span class="slot"><img src="{uri(Image.open(os.path.join(OUT, j["name"] + "_px.png")), 2)}" width="32" height="32" alt="{j["name"]}" title="{j["name"]}"></span>' for j in made)
        hot_old = "".join(f'<span class="slot"><img src="{uri(o.resize((32, 32), Image.NEAREST))}" width="32" height="32" alt="{j["name"]}" title="{j["name"]}"></span>'
                          for j in made if (o := game_icon(j["name"], j.get("mod", "base"))) is not None)
        sections.append(f"""<section id="{b}">
  <h2>{TITLES.get(b, b)} <span class="count">{len(made)} of {len(jobs)}</span></h2>
  <div class="hotlabel">New</div><div class="hotbar">{hot_new}</div>
  {'<div class="hotlabel">The game now</div><div class="hotbar">' + hot_old + '</div>' if hot_old else ''}
  <div class="cards">{''.join(card(j) for j in jobs)}</div>
</section>""")
    page = open(os.path.join(HERE, "page.html")).read()
    page = (page.replace("{{SECTIONS}}", "\n".join(reversed(sections)))
            .replace("{{DONE}}", str(done)).replace("{{TOTAL}}", str(total))
            .replace("{{FAILED}}", ("Failed every try, the last one shown and marked: " + ", ".join(failed) + ".")
                     if failed else "Every icon so far passed the checks."))
    open(os.path.join(HERE, "index.html"), "w").write(page)
    print(f"index.html: {done} of {total}; failed: {failed}")


if __name__ == "__main__":
    main()
