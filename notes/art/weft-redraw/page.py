"""The review page: page.py OUT.html, batches from batches.json [{id, title, status, note, sheets:[png]}]."""
import base64, json, sys, html
B = json.load(open("batches.json"))
uri = lambda p: "data:image/png;base64," + base64.b64encode(open(p, "rb").read()).decode()
secs = []
for b in B:
    imgs = "".join(f'<figure><img src="{uri(s)}" alt="{html.escape(b["title"])} sheet"></figure>' for s in b["sheets"])
    secs.append(f'''<section id="{b["id"]}"><header><h2>{html.escape(b["title"])}</h2><span class="st st-{b["status"].split()[0].lower()}">{html.escape(b["status"])}</span></header>
<p>{b["note"]}</p>{imgs}</section>''')
open(sys.argv[1], "w").write(f'''<title>Weft content redraw</title>
<meta name="claude:viewport-fit-sides" content="cover">
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Alegreya+SC:wght@700&family=Alegreya+Sans:wght@400;500&display=swap">
<style>
/* layout: one column of batches, each a before/after sheet on the game's slot browns */
:root {{ --bg:#1a1313; --panel:#271c17; --fg:#e4dacf; --muted:#c6a383; --gold:#f2da9a; --bronze:#a75120; --ok:#8ac80f; --wait:#f3ab1f;
  --display:"Alegreya SC", Georgia, serif; --body:"Alegreya Sans", system-ui, sans-serif; color-scheme: dark }}
body {{ background:var(--bg); color:var(--fg); font-family:var(--body); font-size:16px; line-height:1.5;
  padding-left:max(16px, env(safe-area-inset-left,0px)); padding-right:max(16px, env(safe-area-inset-right,0px)); padding-block:24px }}
main {{ max-width:1080px; margin:0 auto; display:grid; gap:28px }}
h1 {{ font-family:var(--display); color:var(--gold); font-size:2rem; margin:0; text-wrap:balance }}
h2 {{ font-family:var(--display); color:var(--gold); font-size:1.3rem; margin:0 }}
.lede {{ color:var(--muted); max-width:65ch; margin:0 }}
section {{ background:var(--panel); border:3px solid var(--bronze); padding:16px; display:grid; gap:10px; min-width:0 }}
section header {{ display:flex; flex-wrap:wrap; gap:10px; align-items:center; justify-content:space-between }}
section p {{ margin:0; max-width:75ch }}
figure {{ margin:0; overflow-x:auto }}
figure img {{ image-rendering:pixelated; display:block; max-width:none }}
.st {{ font-size:.8rem; letter-spacing:.06em; text-transform:uppercase; padding:2px 10px; border:2px solid currentColor }}
.st-approved {{ color:var(--ok) }} .st-waiting {{ color:var(--wait) }} .st-redo {{ color:#d8221e }}
</style>
<main><h1>Weft content redraw</h1>
<p class="lede">The art that came in with the 11-22 Weft sync, redrawn in chiacraft's style. Each batch shows Weft's picture on the left and ours on the right, big and at 1&times; in a slot.</p>
{"".join(secs)}</main>''')
