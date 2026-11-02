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
  const STAGES = ["baby", "juvenile", "adolescent", "adult", "elder"];
  let species = [], state = null, chosen = null, mother = null, father = null, pool = null, poolKey = "";
  // What happened over time: a point per step (day, alive per species, born and died since).
  let history = [], lastTick = 0, playing = null, shownId = null;
  const COLORS = ["#d98b52", "#8fbf8a", "#7fa7d6", "#e1977d", "#c9b26b", "#b48ad6", "#6fc2c0"];

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
    if (a.stage === 4) return "old, past breeding";
    return a.stage < 3 ? `growing, ${(a.growing / DAY).toFixed(1)} d` : "ready";
  }
  const ageDays = a => ((a.died || state.tick) - a.born) / DAY;

  function run(req) {
    const r = Animals.call(req);
    if (r.error) { say(r.error, true); return false; }
    if (req.op === "sim_reset") { history = []; lastTick = 0; }
    state = r;
    record();
    draw();
    return true;
  }
  // A point for the charts when time has moved on.
  function record() {
    if (state.tick === lastTick && history.length) return;
    const from = lastTick, to = state.tick;
    const alive = species.map(sp => state.animals.filter(a => !a.died && a.species === sp.index && !a.egg).length);
    const born = state.animals.filter(a => a.mother && a.born > from && a.born <= to && !a.egg).length;
    const died = state.animals.filter(a => a.died > from && a.died <= to).length;
    const days = Math.max((to - from) / DAY, 1e-9);
    history.push({ day: to / DAY, alive, born: born / days, died: died / days });
    if (history.length > 600) history = history.filter((_, i) => i % 2 === 0);
    lastTick = to;
    drawCharts();
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
      tr.onclick = tr.onkeydown = e => { if (e.type === "keydown" && e.key !== "Enter") return; chosen = a.id; shownId = null; draw(); show3d(a); };
      return tr;
    }));
    $("lab-pair").textContent = `Mother ${mother ? who(byId(mother)) : "\u2014"} \u00b7 Father ${father ? who(byId(father)) : "\u2014"}`;
    drawChosen();
    $("lab-log").replaceChildren(...state.log.slice(-60).reverse().map(t => el("li", { textContent: t })));
  }

  // ---------------------------------------------------------------- charts

  const css = name => getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  // A round top for an axis: 1, 2 or 5 times a power of ten.
  const niceMax = v => { if (v <= 0) return 1; const p = 10 ** Math.floor(Math.log10(v)); return [1, 2, 5, 10].map(k => k * p).find(k => k >= v); };
  function chart(canvas, series, colors) {
    const dpr = Math.min(devicePixelRatio || 1, 2), W = canvas.clientWidth, H = canvas.clientHeight;
    if (!W) return;
    canvas.width = W * dpr; canvas.height = H * dpr;
    const g = canvas.getContext("2d");
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    g.clearRect(0, 0, W, H);
    const L = 34, R = 8, T = 8, B = 20, pw = W - L - R, ph = H - T - B;
    const days = history.map(h => h.day), x0 = days[0] || 0, x1 = Math.max(days[days.length - 1] || 1, x0 + 1);
    const top = niceMax(Math.max(1, ...series.flat()));
    const X = d => L + (d - x0) / (x1 - x0) * pw, Y = v => T + ph - v / top * ph;
    g.font = "11px " + css("--mono");
    g.fillStyle = css("--muted"); g.strokeStyle = css("--line"); g.lineWidth = 1;
    for (const v of [0, top / 2, top]) {
      g.beginPath(); g.moveTo(L, Y(v)); g.lineTo(W - R, Y(v)); g.stroke();
      g.fillText(String(+v.toFixed(2)), 2, Y(v) + 4);
    }
    g.fillText(`day ${x0.toFixed(0)}`, L, H - 4);
    const end = `day ${x1.toFixed(0)}`;
    g.fillText(end, W - R - g.measureText(end).width, H - 4);
    series.forEach((ys, k) => {
      if (!ys.some(v => v > 0)) return;
      g.strokeStyle = colors[k]; g.lineWidth = 2; g.beginPath();
      ys.forEach((v, i) => { const px = X(days[i]), py = Y(v); i ? g.lineTo(px, py) : g.moveTo(px, py); });
      g.stroke();
    });
  }
  function drawCharts() {
    if (history.length < 2) return;
    const used = species.map((_, k) => history.some(h => h.alive[k] > 0));
    chart($("lab-chart-alive"), species.map((_, k) => used[k] ? history.map(h => h.alive[k]) : []), COLORS);
    // Smoothed over a few steps: births come in litters.
    const smooth = key => history.map((_, i) => { const w = history.slice(Math.max(0, i - 4), i + 1); return w.reduce((a, h) => a + h[key], 0) / w.length; });
    chart($("lab-chart-flow"), [smooth("born"), smooth("died")], [css("--good"), css("--alt")]);
    $("lab-chart-key").replaceChildren(...species.filter((_, k) => used[k]).map(sp => el("span", {},
      el("span", { className: "key", style: `background:${COLORS[sp.index]}` }), sp.breed || sp.name)));
  }

  // ---------------------------------------------------------------- balance

  // What each species' numbers add up to: how fast a breeder gets young, generations, a
  // recessive coat (the 4th generation is when two carriers' young show one, 1 in 4).
  function balance() {
    const d = t => +(t / DAY).toFixed(2);
    const rows = species.map(sp => {
      const carry = sp.gestation || 6000 + sp.incubation;
      const cycle = (sp.gestation || 6000) + sp.recovery;
      const litter = (sp.litter[0] + sp.litter[1]) / 2;
      const gen = sp.growth + carry;
      return [sp.breed ? `${sp.name} (${sp.breed})` : sp.name, sp.gestation ? `${d(sp.gestation)} d` : `lays at once, hatch ${d(sp.incubation)} d`,
        `${sp.litter[0]}\u2013${sp.litter[1]}`, `${d(sp.recovery)} d`, `${d(sp.growth)} d`, `${d(sp.lifespan)} d`, `${d(gen)} d`,
        (litter * 10 * DAY / cycle).toFixed(1), `${d(3 * gen)} d`, `${(3 * gen / DAY / 3).toFixed(1)} h`];
    });
    const head = ["Species", "Carrying", "Litter", "Rest after", "Grows up in", "Lives about", "A generation", "Young per female per 10 d", "To the 4th generation", "That in play hours"];
    $("lab-balance").replaceChildren(el("thead", {}, el("tr", {}, ...head.map(t => el("th", { textContent: t })))),
      el("tbody", {}, ...rows.map(r => el("tr", {}, ...r.map((t, i) => el("td", { className: i ? "val" : "", textContent: t }))))));
  }

  // ---------------------------------------------------------------- playing and the live view

  function play() {
    if (playing) { clearInterval(playing); playing = null; $("lab-play").textContent = "Play"; return; }
    $("lab-play").textContent = "Pause";
    playing = setInterval(() => {
      const perSecond = +$("lab-speed").value * DAY;
      run({ op: "sim_advance", ticks: Math.round(perSecond / 5) });
    }, 200);
  }
  // The chosen animal in the 3D view (its species' model, painted from its genes), and that view
  // mirrored here while it is.
  function show3d(a) {
    if (!a || !window.showAnimal || shownId === a.id) return;
    shownId = a.id;
    window.showAnimal(species[a.species], a.genome, { scroll: false });
  }
  window.afterRender = src => {
    const c = $("lab-live");
    if (!c || !shownId || c.getBoundingClientRect().bottom < 0 || c.getBoundingClientRect().top > innerHeight) return;
    if (c.width !== c.clientWidth) { c.width = c.clientWidth; c.height = c.clientWidth * 9 / 16; }
    const g = c.getContext("2d"), r = Math.min(c.width / src.width, c.height / src.height);
    g.fillStyle = css("--bg"); g.fillRect(0, 0, c.width, c.height);
    g.drawImage(src, (c.width - src.width * r) / 2, (c.height - src.height * r) / 2, src.width * r, src.height * r);
  };

  function card(a, label) {
    if (!a) return el("div", { className: "fam unknown" }, el("span", { className: "muted", textContent: label || "wild" }));
    const b = el("button", { type: "button", className: "fam" + (a.died ? " dead" : "") + (a.id === chosen ? " on" : "") },
      chip(a.look), el("span", { textContent: `#${a.id} ${a.female ? "\u2640" : "\u2642"}` }));
    b.onclick = () => { chosen = a.id; shownId = null; draw(); show3d(a); };
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
        el("span", {}, `strength `, el("b", { textContent: `${(a.vigor * 100).toFixed(0)}%` })),
        el("span", {}, status(a))),
      el("div", { className: "btns" },
        btn("As mother", () => { if (!a.female) return say(`#${a.id} is male`, true); mother = a.id; draw(); }, a.died),
        btn("As father", () => { if (a.female) return say(`#${a.id} is female`, true); father = a.id; draw(); }, a.died),
        btn("Show in the big view", () => window.showAnimal && window.showAnimal(sp, a.genome)),
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
    $("lab-play").onclick = play;
    balance();
    addEventListener("resize", drawCharts);
    for (const [id, t] of [["lab-hour", HOUR], ["lab-1d", DAY], ["lab-10d", 10 * DAY], ["lab-100d", 100 * DAY]]) $(id).onclick = () => run({ op: "sim_advance", ticks: t });
    $("lab-mate").onclick = () => {
      if (!mother || !father) return say("Pick a mother and a father first (an animal, then As mother / As father).", true);
      if (run({ op: "sim_mate", mother, father })) say(`#${mother} and #${father} mated.`);
    };
    const settings = () => {
      const rule = $("lab-life-rule").value;
      $("lab-life-wrap").hidden = rule !== "days";
      run({ op: "sim_settings", auto: $("lab-auto").checked, lifespan: rule === "days" ? Math.max(1, +$("lab-life").value || 1) : rule, cap: +$("lab-cap").value || 150 });
    };
    for (const id of ["lab-auto", "lab-life", "lab-life-rule", "lab-cap"]) $(id).onchange = settings;
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
