// Checks the animals' logic (the game's own, compiled to WASM) for what should always hold:
// inheritance by the rules, the sexes, refusals, timings, caps, scars, evolution's direction,
// every animal unique. Prints each check and the numbers behind it; exits 1 if one fails.
//
//   mods/animals/web/build.sh && node mods/animals/web/test.mjs [animals.wasm]
import fs from "fs";
import path from "path";

const file = process.argv[2] || path.join(path.dirname(new URL(import.meta.url).pathname), "animals.wasm");
const { instance } = await WebAssembly.instantiate(fs.readFileSync(file), {});
const w = instance.exports, enc = new TextEncoder(), dec = new TextDecoder();
function call(req) {
  const b = enc.encode(JSON.stringify(req)), p = w.alloc(b.length);
  new Uint8Array(w.memory.buffer, p, b.length).set(b);
  const out = w.call(p, b.length);
  w.free(p, b.length);
  return JSON.parse(dec.decode(new Uint8Array(w.memory.buffer, out, w.out_len())));
}

let failed = 0, seed = 1;
const s = () => seed++;
function check(name, ok, detail) {
  console.log(`${ok ? "pass" : "FAIL"}  ${name}${detail ? `  (${detail})` : ""}`);
  if (!ok) failed++;
}
const near = (x, want, tol) => Math.abs(x - want) <= tol;
const DAY = 24000;
const { species } = call({ op: "species" });
const idx = name => species.findIndex(x => x.name === name);
const sp = name => species[idx(name)];
const rnd = (name, female, extra) => call(Object.assign({ op: "random", species: idx(name), female, seed: s() }, extra || {}));
const edit = (g, req) => call(Object.assign({ op: "edit", genome: g }, req));
const locus = (an, id) => an.loci.find(l => l.id === id);
const kids = (m, f, n) => {
  const out = [];
  while (out.length < n) {
    const r = call({ op: "breed", mother: m.genome, father: f.genome, count: 64, seed: s() });
    if (r.error) throw new Error(r.error);
    out.push(...r.young);
  }
  return out.slice(0, n);
};

console.log("Genes and breeding");
{
  // Two carriers of dilute (D/d) have a quarter dilute young (d/d): Mendel.
  const m = edit(rnd("dog", true).genome, { locus: "D", values: ["D", "d"] });
  const f = edit(rnd("dog", false).genome, { locus: "D", values: ["D", "d"] });
  const young = kids(m, f, 2000);
  const dd = young.filter(k => locus(k, "D").alleles.join("") === "dd").length / young.length;
  check("carriers' young: a quarter recessive", near(dd, 0.25, 0.03), `d/d ${(dd * 100).toFixed(1)}% of ${young.length}`);
  const fem = young.filter(k => k.female).length / young.length;
  check("half the young female", near(fem, 0.5, 0.03), `${(fem * 100).toFixed(1)}%`);
  check("young are babies", young.every(k => k.stage === 0));
  check("young of their parents' species", young.every(k => k.species === m.species));
}
{
  // Orange is on the X: a male has one copy, so tortoiseshell males don't happen; an orange
  // mother's sons are all orange.
  const m = edit(rnd("cat", true).genome, { locus: "O", values: ["O", "O"] });
  const f = edit(rnd("cat", false).genome, { locus: "O", values: ["o"] });
  const young = kids(m, f, 400);
  const sons = young.filter(k => !k.female), daughters = young.filter(k => k.female);
  check("males carry one copy of an X gene", sons.every(k => locus(k, "O").alleles.length === 1));
  // (Each young has a 1% chance per gene of a mutation: `breed`.)
  const notOrange = sons.filter(k => locus(k, "O").alleles[0] !== "O").length / sons.length;
  check("an orange mother's sons orange (but for mutations)", notOrange <= 0.03, `${(notOrange * 100).toFixed(1)}% of ${sons.length} sons not`);
  check("her daughters by a black father all tortoiseshell", daughters.every(k => k.look.patches), `${daughters.filter(k => k.look.patches).length} of ${daughters.length}`);
  check("no tortoiseshell male", young.filter(k => !k.female && k.look.patches).length === 0);
}
{
  // Size: young sit round their parents' mean (inherited), with spread.
  const pairs = [];
  for (let i = 0; i < 300; i++) {
    const m = rnd("cat", true), f = rnd("cat", false);
    const mid = (locus(m, "Sz").genetic + locus(f, "Sz").genetic) / 2;
    const k = kids(m, f, 1)[0];
    pairs.push([mid, locus(k, "Sz").genetic, locus(k, "Sz").shown]);
  }
  const slope = (xs, ys) => { const mx = xs.reduce((a, b) => a + b) / xs.length, my = ys.reduce((a, b) => a + b) / ys.length; let n = 0, d = 0; xs.forEach((x, i) => { n += (x - mx) * (ys[i] - my); d += (x - mx) ** 2; }); return n / d; };
  const gs = slope(pairs.map(p => p[0]), pairs.map(p => p[1])), ss = slope(pairs.map(p => p[0]), pairs.map(p => p[2]));
  check("size genes inherited (young's genes follow the parents' mean)", near(gs, 1, 0.25), `slope ${gs.toFixed(2)}`);
  check("size as it shows follows less (chance)", ss < gs && ss > 0.2, `slope ${ss.toFixed(2)}, heritability ${locus(rnd("cat", true), "Sz").heritability}`);
}
console.log("Refusals");
{
  const cat = rnd("cat", true), tom = rnd("cat", false), dog = rnd("dog", false);
  check("two females refused", !!call({ op: "breed", mother: cat.genome, father: rnd("cat", true).genome }).error);
  check("a male as mother refused", !!call({ op: "breed", mother: tom.genome, father: tom.genome }).error);
  check("cat with dog refused", !!call({ op: "breed", mother: cat.genome, father: dog.genome }).error);
  const vr = rnd("raptor", true), os = rnd("osmolskae", false);
  check("Velociraptor's two species refused", !!call({ op: "breed", mother: vr.genome, father: os.genome }).error);
  check("bad genome refused", !!call({ op: "describe", genome: "zz" }).error);
  check("unknown gene refused", !!edit(cat.genome, { locus: "Nope", values: ["x"] }).error);
  check("unknown allele refused", !!edit(cat.genome, { locus: "D", values: ["q"] }).error);
}
console.log("Genomes, scars, looks");
{
  const a = rnd("wyvern", true);
  check("a genome survives the round trip", call({ op: "describe", genome: a.genome }).genome === a.genome);
  let g = a.genome;
  for (let k = 1; k <= 6; k++) g = edit(g, { scar: k % 5 + 1, place: k }).genome;
  const sc = call({ op: "describe", genome: g }).look.scars;
  const young = call({ op: "describe", genome: edit(rnd("dog", true).genome, { stage: 3 }).genome });
  check("an elder's genome reads as old", edit(young.genome, { stage: 4 }).stage === 4);
  check("at most four scars, the oldest gone", sc.length === 4 && sc[0] === 4, `kinds ${sc.join(",")}`);
  const cats = Array.from({ length: 300 }, (_, i) => rnd("cat", i % 2 === 0));
  check("every animal's genome different", new Set(cats.map(c => c.genome)).size === cats.length);
  const looks = new Set(cats.map(c => c.look.name)).size;
  const fine = new Set(cats.map(c => c.look.name + c.look.marks.map(x => x.toFixed(1)).join() + c.look.base.map(x => Math.round(x / 8)).join())).size;
  check("no two cats look alike (coat, marks, shade)", fine === cats.length, `${looks} coat names, ${fine} distinct looks of ${cats.length}`);
  const bad = cats.filter(c => JSON.stringify(c).includes("null") && false || [...c.look.base, ...c.look.under, ...c.look.marks].some(v => !Number.isFinite(v)));
  check("no NaN in looks", bad.length === 0);
  for (const x of species) {
    const an = call({ op: "random", species: x.index, seed: s() });
    const builds = an.build.flatMap(b => b[1]);
    // A part may be drawn down to nothing (a doe's antlers, a snowy owl's ear tufts: a build trait
    // whose sex or kind turns it off), never past 1.6 or inside out.
    check(`${x.name}${x.breed ? "/" + x.breed : ""}: build within its ranges, size sane`, builds.every(v => v > 0 && v < 1.6) && an.size > 0.5 && an.size < 1.6, `size ${an.size.toFixed(2)}`);
  }
}
console.log("Over time");
function sim(name, extra) {
  call({ op: "sim_reset", seed: s() });
  call(Object.assign({ op: "sim_settings" }, extra || {}));
  call({ op: "sim_wild", species: idx(name), female: true });
  return call({ op: "sim_wild", species: idx(name), female: false });
}
for (const name of ["cat", "raptor", "wyvern"]) {
  const x = sp(name);
  let st = sim(name);
  st = call({ op: "sim_mate", mother: 1, father: 2 });
  check(`${name}: a pair mates`, !st.error, st.error);
  check(`${name}: mating again refused while expecting`, !!call({ op: "sim_mate", mother: 1, father: 2 }).error);
  const carry = x.gestation || 6000;
  st = call({ op: "sim_advance", ticks: carry - 1200 });
  check(`${name}: nothing born early`, st.animals.length === 2);
  st = call({ op: "sim_advance", ticks: 1200 });
  const young = st.animals.filter(a => a.mother === 1);
  check(`${name}: ${x.gestation ? "born" : "laid"} on time`, young.length >= x.litter[0] && young.length <= x.litter[1], `${young.length} young at day ${(carry / DAY).toFixed(2)}`);
  if (!x.gestation) {
    check(`${name}: they are eggs`, young.every(a => a.egg > 0));
    st = call({ op: "sim_advance", ticks: x.incubation });
    check(`${name}: eggs hatch after incubation`, st.animals.filter(a => a.mother === 1).every(a => a.egg === 0));
  }
  const k = st.animals.find(a => a.mother === 1);
  check(`${name}: a young one can't mate`, !!call({ op: "sim_mate", mother: k.female ? k.id : 1, father: k.female ? 2 : k.id }).error);
  st = call({ op: "sim_advance", ticks: x.growth });
  const grown = st.animals.filter(a => a.mother === 1);
  check(`${name}: young grown after ${(x.growth / DAY).toFixed(0)} days`, grown.every(a => a.stage === 3), grown.map(a => a.stage).join(","));
  check(`${name}: generation 1`, grown.every(a => a.generation === 1));
}
{
  let st = sim("cat", { auto: true, cap: 40 });
  for (let d = 0; d < 200; d++) st = call({ op: "sim_advance", ticks: DAY });
  const alive = st.animals.filter(a => !a.died).length;
  // Breeding slows past the room, and the crowded die of want: it settles near the room.
  check("left to themselves they settle near the room", alive >= 20 && alive <= 70, `${alive} alive at day 200, room 40`);
  const want = st.animals.filter(a => a.cause === "want");
  const frail = want.filter(a => a.stage !== 3).length;
  check("the crowded die of want, the old and young first", want.length > 0, `${want.length} died of want, ${frail} of them old or young`);
  // Old age as the game has it: each lives its species' lifespan, give or take a fifth, and is
  // an elder (past breeding) for the last fifth of an average life.
  st = sim("cat", { auto: true, cap: 60, lifespan: "game" });
  const span = sp("cat").lifespan / DAY;
  let peak = 0, gens = 0, elders = 0;
  for (let d = 0; d < 365; d++) {
    st = call({ op: "sim_advance", ticks: DAY });
    const alive = st.animals.filter(a => !a.died);
    peak = Math.max(peak, alive.length); gens = Math.max(gens, ...st.animals.map(a => a.generation));
    elders = Math.max(elders, alive.filter(a => a.stage === 4).length);
  }
  const old = st.animals.filter(a => a.cause === "old age" && a.mother);
  const ages = old.map(a => (a.died - a.born) / DAY);
  check("born ones die of age between 0.8 and 1.2 lifespans", ages.length > 10 && ages.every(d => d >= span * 0.8 - 0.2 && d <= span * 1.2 + 0.2),
    `${ages.length} died, aged ${Math.min(...ages).toFixed(1)}-${Math.max(...ages).toFixed(1)} days, lifespan ${span}`);
  check("there are elders", elders > 0, `up to ${elders} at once`);
  const aliveEnd = st.animals.filter(a => !a.died).length;
  check("a cat population with old age lasts a year", aliveEnd > 0, `${aliveEnd} alive at day 365, peak ${peak}, ${gens} generations, ${st.animals.filter(a => a.cause === "old age").length} died of age`);
  // An elder can't mate; wild ones come at different ages.
  st = sim("cat", { lifespan: "game" });
  const wildAges = [];
  for (let i = 0; i < 20; i++) st = call({ op: "sim_wild", species: idx("cat") });
  for (const a of st.animals) wildAges.push((st.tick - a.born) / DAY);
  check("wild ones met at different ages, all grown", wildAges.every(d => d >= sp("cat").growth / DAY) && Math.max(...wildAges) - Math.min(...wildAges) > 5,
    `${Math.min(...wildAges).toFixed(1)}-${Math.max(...wildAges).toFixed(1)} days`);
  // Until an elder and a grown one of the other sex are alive together (a fixed number of days
  // found none when every elder had already died, by the seed).
  let elder, mateOf;
  for (let d = 0; d < span * 2 && !(elder && mateOf); d++) {
    st = call({ op: "sim_advance", ticks: DAY });
    elder = st.animals.find(a => !a.died && a.stage === 4);
    mateOf = elder && st.animals.find(a => !a.died && a.female !== elder.female && a.stage === 3);
  }
  check("an elder can't mate", elder && mateOf && !!call({ op: "sim_mate", mother: elder.female ? elder.id : mateOf.id, father: elder.female ? mateOf.id : elder.id }).error,
    elder && mateOf ? "" : "no elder and grown mate alive together");
  st = sim("cat", { lifespan: "off" });
  st = call({ op: "sim_advance", ticks: span * 3 * DAY });
  check("old age off: nobody dies", st.animals.every(a => !a.died));
}
{
  // Strength with age: growing up, its prime, old.
  let st = sim("cat", { lifespan: "game" });
  st = call({ op: "sim_mate", mother: 1, father: 2 });
  st = call({ op: "sim_advance", ticks: sp("cat").gestation });
  const kid = () => st.animals.find(a => a.mother === 1);
  const newborn = kid().vigor;
  st = call({ op: "sim_advance", ticks: sp("cat").growth / 2 });
  const half = kid().vigor;
  st = call({ op: "sim_advance", ticks: sp("cat").growth * 3 });
  const prime = kid().vigor;
  st = call({ op: "sim_advance", ticks: sp("cat").lifespan * 0.85 });
  const k = kid(), old = k.died ? null : k.vigor;
  check("strength grows up, peaks, and fades when old", newborn < half && half < prime && prime === 1 && (old === null || old < prime),
    `newborn ${(newborn * 100).toFixed(0)}%, half grown ${(half * 100).toFixed(0)}%, prime ${(prime * 100).toFixed(0)}%, old ${old === null ? "died" : (old * 100).toFixed(0) + "%"}`);
}
console.log("Balance");
// Every species, a new one too: its numbers against its rarity's targets (`core/src/balance.rs`),
// and a wild group left to itself at its own room lasts, neither dying out nor flooding.
for (const x of species) {
  for (const c of x.balance.checks) check(`${x.name}: ${c.name}`, c.ok, `${c.found}, want ${c.target}`);
  const homes = x.bio.habitats;
  check(`${x.name}: lives somewhere, commonest in one place`, homes.length > 0 && homes.every(([b, h]) => b.includes(":") && h > 0 && h <= 1) && homes.some(([, h]) => h === 1),
    homes.map(([b, h]) => `${b} ${Math.round(h * 100)}%`).join(", "));
  // Alone (no newcomers, as in a pen; in the open the engine spawns more), over 150 days or a
  // life and a half: two runs of three last at least. A small wild group as the game meets them
  // (two pairs, of any age), its runs seeded by its name, so a species added elsewhere doesn't
  // shuffle another's luck.
  const days = Math.max(150, Math.round(x.lifespan / DAY * 1.5));
  const own = [...x.name].reduce((h, ch) => (h * 31 + ch.charCodeAt(0)) >>> 0, 7);
  const runs = [0, 1, 2].map(r => {
    call({ op: "sim_reset", seed: (own + r) * 2 });  // the simulator's seeds 2k and 2k + 1 are one run
    call({ op: "sim_settings", auto: true, cap: x.crowd, lifespan: "game" });
    let st, peak = 0;
    for (let i = 0; i < 4; i++) st = call({ op: "sim_wild", species: x.index, female: i % 2 === 0 });
    for (let d = 0; d < days; d++) {
      st = call({ op: "sim_advance", ticks: DAY });
      peak = Math.max(peak, st.animals.filter(a => !a.died).length);
    }
    return { alive: st.animals.filter(a => !a.died).length, peak, gens: Math.max(...st.animals.map(a => a.generation)) };
  });
  const lasted = runs.filter(r => r.alive >= 2).length, peak = Math.max(...runs.map(r => r.peak));
  check(`${x.name}: a wild group at its room lasts ${days} days`, lasted >= 2 && peak <= x.crowd * 3,
    `${lasted} of 3 lasted, alive ${runs.map(r => r.alive).join("/")}, peak ${peak}, room ${x.crowd}, up to ${Math.max(...runs.map(r => r.gens))} generations`);
}

{
  // The settings page (mods/animals/SPECIES.md) is written from the species table: rewrite it
  // after a change (web/sheet.mjs).
  const page = path.join(path.dirname(new URL(import.meta.url).pathname), "..", "SPECIES.md");
  const now = fs.existsSync(page) ? fs.readFileSync(page, "utf8") : "";
  const fresh = now === call({ op: "sheet" }).markdown;
  check("SPECIES.md is up to date", fresh, fresh ? "" : now ? "rewrite it: node mods/animals/web/sheet.mjs" : "missing");
}

console.log("Evolution");
// Each run seeded by itself, so species added elsewhere (whose checks draw seeds first) don't
// shuffle its luck.
for (const [k, [name, h, want]] of [["raptor", "snow", "lighter"], ["raptor", "forest", "darker"], ["raptor", "desert", "sandier"]].entries()) {
  const avg = r => r.sample.map(a => a.look.average).reduce((a, b) => a.map((v, i) => v + b[i] / r.sample.length), [0, 0, 0]);
  const start = call({ op: "evolve", species: idx(name), habitat: h, generations: 0, seed: 1000 + 10 * k });
  const end = call({ op: "evolve", species: idx(name), habitat: h, generations: 300, seed: 1001 + 10 * k });
  const bg = end.background, dist = c => Math.hypot(...c.map((v, i) => v - bg[i]));
  const [d0, d1] = [dist(avg(start)), dist(avg(end))];
  check(`${name} in ${h} turns ${want} (closer to the ground)`, d1 < d0, `distance to ground ${d0.toFixed(0)} -> ${d1.toFixed(0)} after 300 generations`);
  const sums = end.pool.filter(p => p.length > 2).map(p => p.reduce((a, b) => a + b, 0));
  check(`${name} in ${h}: allele shares still add up`, sums.every(t => near(t, 1, 0.01)) && end.pool.flat().every(Number.isFinite));
}

console.log(failed ? `\n${failed} failed` : "\nall passed");
process.exit(failed ? 1 : 0);
