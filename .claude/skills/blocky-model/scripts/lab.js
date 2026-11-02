// The breeding lab: a population run by the game's own rules (animals.wasm, `sim_*` and `evolve`
// in mods/animals/web/src/lib.rs). Add wild animals, mate them, let time pass (carrying, litters,
// eggs hatching, growing up; old age if set, which the game doesn't have yet), follow families,
// and watch a habitat's wild pool evolve. Needs `Animals` (animals.js) and, to show an animal in
// 3D, the viewer's `showAnimal(speciesName, about)`.
(function () {
  "use strict";
  const $ = id => document.getElementById(id);
  const el = (tag, props, ...kids) => { const e = Object.assign(document.createElement(tag), props || {}); e.append(...kids.filter(k => k != null && k !== false)); return e; };
  const hex = c => "#" + c.map(v => Math.round(Math.max(0, Math.min(255, v))).toString(16).padStart(2, "0")).join("");
  const DAY = 24000, HOUR = 1000;
  const STAGES = ["baby", "juvenile", "adolescent", "adult"];
  let species = [], state = null, chosen = null, mother = null, father = null, pool = null, poolKey = "";

  function chip(look) {
    if (!look) return el("span", { className: "chip" });
    const marks = look.pattern ? look.pattern[1] : look.patches || look.merle || look.under;
    return el("span", { className: "chip", title: look.name },
      el("i", { style: `background:${hex(look.base)}` }), el("i", { style: `background:${hex(look.under)}` }),
      el("i", { style: `background:${hex(marks)}` }), look.eye ? el("i", { className: "eye", style: `background:${hex(look.eye)}` }) : null);
  }
  const who = a => a ? `#${a.id}` : "?";
  const byId = id => state && state.animals.find(a => a.id === id);
  function status(a) {
    if (a.died) return `died (${a.cause}) day ${(a.died / DAY).toFixed(1)}`;
    if (a.egg) return `egg, hatches in ${(a.egg / DAY).toFixed(1)} d`;
    if (a.carrying) return `expecting, ${(a.carrying / DAY).toFixed(1)} d`;
    if (a.rest) return `resting ${Math.ceil(a.rest / 1200)} min`;
    return a.stage < 3 ? `growing, ${(a.growing / DAY).toFixed(1)} d` : "ready";
  }
  const ageDays = a => ((a.died || state.tick) - a.born) / DAY;

  function run(req) {
    const r = Animals.call(req);
    if (r.error) { say(r.error, true); return false; }
    state = r;
    draw();
    return true;
  }
  function say(text, bad) {
    const n = $("lab-msg");
    n.textContent = text;
    n.classList.toggle("bad", !!bad);
  }

  function draw() {
    if (!state) return;
    $("lab-day").textContent = `Day ${state.day.toFixed(1)}`;
    const alive = state.animals.filter(a => !a.died).length;
    $("lab-count").textContent = `${alive} alive, ${state.animals.length - alive} dead`;
    const showDead = $("lab-dead").checked;
    const rows = state.animals.filter(a => showDead || !a.died).slice().reverse();
    $("lab-list").replaceChildren(...rows.map(a => {
      const tr = el("tr", { className: (a.died ? "dead " : "") + (chosen === a.id ? "on" : ""), tabIndex: 0 },
        el("td", { className: "num", textContent: a.id }), el("td", {}, chip(a.look)),
        el("td", { textContent: `${a.look ? a.look.name : "?"}` }),
        el("td", { className: "num", textContent: a.female ? "\u2640" : "\u2642" }),
        el("td", { textContent: a.egg ? "egg" : STAGES[a.stage] }),
        el("td", { className: "num", textContent: a.generation }),
        el("td", { className: "num", textContent: a.mother ? `#${a.mother} \u00d7 #${a.father}` : "wild" }),
        el("td", { className: "muted", textContent: status(a) }));
      tr.onclick = tr.onkeydown = e => { if (e.type === "keydown" && e.key !== "Enter") return; chosen = a.id; draw(); };
      return tr;
    }));
    $("lab-pair").textContent = `Mother ${mother ? who(byId(mother)) : "\u2014"} \u00b7 Father ${father ? who(byId(father)) : "\u2014"}`;
    drawChosen();
    $("lab-log").replaceChildren(...state.log.slice(-60).reverse().map(t => el("li", { textContent: t })));
  }

  function card(a, label) {
    if (!a) return el("div", { className: "fam unknown" }, el("span", { className: "muted", textContent: label || "wild" }));
    const b = el("button", { type: "button", className: "fam" + (a.died ? " dead" : "") + (a.id === chosen ? " on" : "") },
      chip(a.look), el("span", { textContent: `#${a.id} ${a.female ? "\u2640" : "\u2642"}` }));
    b.onclick = () => { chosen = a.id; draw(); };
    return b;
  }
  function drawChosen() {
    const box = $("lab-chosen"), tree = $("lab-tree");
    const a = chosen && byId(chosen);
    if (!a) {
      box.replaceChildren(el("p", { className: "muted", textContent: "Pick an animal in the list to see it, its family and what it can do." }));
      tree.replaceChildren();
      return;
    }
    const sp = species[a.species];
    const btn = (t, f, dis) => { const b = el("button", { type: "button", textContent: t, disabled: !!dis }); b.onclick = f; return b; };
    box.replaceChildren(
      el("div", { className: "who" }, chip(a.look), el("b", { textContent: `#${a.id} ${a.look ? a.look.name : ""}` })),
      el("div", { className: "facts" },
        el("span", {}, `${sp.name}${sp.breed ? ` (${sp.breed})` : ""}, ${a.female ? "female" : "male"}, ${a.egg ? "egg" : STAGES[a.stage]}`),
        el("span", {}, `age `, el("b", { textContent: `${Math.max(0, ageDays(a)).toFixed(1)} d` })),
        el("span", {}, `generation `, el("b", { textContent: a.generation })),
        el("span", {}, `size `, el("b", { textContent: `${(a.size * 100).toFixed(0)}%` })),
        el("span", {}, status(a))),
      el("div", { className: "btns" },
        btn("As mother", () => { if (!a.female) return say(`#${a.id} is male`, true); mother = a.id; draw(); }, a.died),
        btn("As father", () => { if (a.female) return say(`#${a.id} is female`, true); father = a.id; draw(); }, a.died),
        btn("Show in 3D", () => window.showAnimal && window.showAnimal(sp, a.genome)),
        btn("Scar", () => run({ op: "sim_scar", id: a.id, kind: 1 + Math.floor(Math.random() * 5) }), a.died),
        btn("Kill", () => run({ op: "sim_kill", id: a.id, cause: "killed" }), a.died)));
    // Its family: grandparents, parents, it, and its young.
    const m = byId(a.mother), f = byId(a.father);
    const kids = state.animals.filter(k => k.mother === a.id || k.father === a.id);
    const sibs = a.mother ? state.animals.filter(k => k.id !== a.id && k.mother === a.mother && k.father === a.father) : [];
    tree.replaceChildren(
      el("div", { className: "gen" }, el("h4", { textContent: "Grandparents" }),
        card(m && byId(m.mother), m ? "wild" : "\u2014"), card(m && byId(m.father), m ? "wild" : "\u2014"),
        card(f && byId(f.mother), f ? "wild" : "\u2014"), card(f && byId(f.father), f ? "wild" : "\u2014")),
      el("div", { className: "gen" }, el("h4", { textContent: "Parents" }), card(m, "wild-born"), card(f, a.mother ? "?" : "wild-born")),
      el("div", { className: "gen" }, el("h4", { textContent: sibs.length ? `It and ${sibs.length} siblings` : "It" }), card(a), ...sibs.slice(0, 11).map(s => card(s))),
      el("div", { className: "gen" }, el("h4", { textContent: kids.length ? `Young (${kids.length})` : "No young yet" }), ...kids.slice(0, 24).map(k => card(k))));
  }

  // Evolution: a habitat's wild pool, generation by generation (one in-game day each).
  function evolve(n) {
    const sp = species[+$("lab-ev-species").value], h = $("lab-ev-habitat").value;
    const key = `${sp.index}/${h}`;
    if (key !== poolKey) { pool = null; poolKey = key; $("lab-ev-gen").textContent = "0"; }
    const r = Animals.call({ op: "evolve", species: sp.index, habitat: h, pool, generations: n, seed: Animals.seed() });
    if (r.error) return say(r.error, true);
    pool = r.pool;
    $("lab-ev-gen").textContent = String(+$("lab-ev-gen").textContent + n);
    $("lab-ev-text").textContent = r.text;
    $("lab-ev-bg").style.background = hex(r.background);
    $("lab-ev-sample").replaceChildren(...r.sample.map(s => el("span", { className: "evs" }, chip(s.look), el("small", { textContent: s.look.name }))));
  }

  function setup() {
    species = Animals.call({ op: "species" }).species;
    const habitats = Animals.call({ op: "species" }).habitats;
    const opts = (sel, list, label) => sel.replaceChildren(...list.map((x, i) => el("option", { value: label ? i : x, textContent: label ? label(x) : x })));
    const spName = s => s.breed ? `${s.name} (${s.breed})` : s.name;
    opts($("lab-species"), species, spName);
    opts($("lab-habitat"), habitats);
    opts($("lab-ev-species"), species, spName);
    opts($("lab-ev-habitat"), habitats);
    const wild = (count, female) => run({ op: "sim_wild", species: +$("lab-species").value, habitat: $("lab-habitat").value, count, female });
    $("lab-add-pair").onclick = () => { wild(1, true) && wild(1, false); };
    $("lab-add-six").onclick = () => wild(6);
    for (const [id, t] of [["lab-hour", HOUR], ["lab-1d", DAY], ["lab-10d", 10 * DAY], ["lab-100d", 100 * DAY]]) $(id).onclick = () => run({ op: "sim_advance", ticks: t });
    $("lab-mate").onclick = () => {
      if (!mother || !father) return say("Pick a mother and a father first (an animal, then As mother / As father).", true);
      if (run({ op: "sim_mate", mother, father })) say(`#${mother} and #${father} mated.`);
    };
    const settings = () => run({ op: "sim_settings", auto: $("lab-auto").checked, lifespan: +$("lab-life").value || 0, cap: +$("lab-cap").value || 150 });
    for (const id of ["lab-auto", "lab-life", "lab-cap"]) $(id).onchange = settings;
    $("lab-dead").onchange = draw;
    $("lab-reset").onclick = () => { chosen = mother = father = null; run({ op: "sim_reset", seed: Animals.seed() }); settings(); say("A new world."); };
    $("lab-ev-10").onclick = () => evolve(10);
    $("lab-ev-100").onclick = () => evolve(100);
    $("lab-ev-reset").onclick = () => { poolKey = ""; evolve(0); };
    $("lab-ev-species").onchange = $("lab-ev-habitat").onchange = () => { poolKey = ""; evolve(0); };
    // A small population to start from: three pairs of cats.
    run({ op: "sim_reset", seed: 20261102 });
    $("lab-species").value = String(species.findIndex(s => s.name === "cat"));
    wild(3, true); wild(3, false);
    settings();
    say("Six wild cats to start. Pick a female and a male, mate them, then let time pass.");
    $("lab-ev-species").value = String(species.findIndex(s => s.name === "raptor"));
    $("lab-ev-habitat").value = "snow";
    evolve(0);
  }
  Animals.ready.then(setup).catch(e => say("The game's logic didn't load: " + e.message, true));
})();
