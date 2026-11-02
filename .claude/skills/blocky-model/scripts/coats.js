// Coats painted on the fly from genes: the genome of an animal (random from the wild, or a baby
// of two parents), what it looks like (phenotype rules per species, from real coat genetics,
// simplified), and a renderer that paints that look on a creature's layers (`layers.py`: coat or
// fixed, light, position on the body, where white, points, a mask and patterns show). The game
// paints its animals with the same rules in Rust (`mods/animals/src/coats.rs`): change both.
//
//   const genes = <species gene file (mods/animals/genetics/<species>.json)>;
//   const animal = Coats.random(genes, rng, null, "chihuahua");  // {sex, genome, seed, breed}
//   const baby = Coats.breed(genes, mother, father, rng);
//   const look = Coats.phenotype(genes, animal);        // {name, base, under, pattern, ...}
//   Coats.render(layers, defaultRGBA, look, paint) -> RGBA bytes (the texture)
//
// `paint`: a paint screen's choices, on top of the genes: {base, under, markings, points}
// colors (any color) and a stencil {kind: "stripes" | "spots" | "patches", color}.
//
// The core every species shares (`mods/animals/genetics/core.json`): `Coats.withCore(genes,
// core)` adds its traits after a species' own loci (size, shade, warmth, white, the marks' size,
// density, contrast, edge and stretch, eyes, build), each showing partly by chance (its
// heritability `h`), shifted by sex (`sexes`) and youth (`young`; `animal.stage` 0 a baby, 3
// grown). A gene file's `habitats` are where wild ones start in each country:
// `Coats.random(genes, rng, sex, breed, habitat)`.
(function (root) {
  "use strict";

  // ---------------------------------------------------------------- randomness and genes

  function rng(seed) {
    let s = (seed >>> 0) || 0x9e3779b9;
    return () => { s ^= s << 13; s >>>= 0; s ^= s >>> 17; s ^= s << 5; s >>>= 0; return s / 4294967296; };
  }
  const clamp01 = v => Math.max(0, Math.min(1, v));
  const gauss = r => Math.sqrt(-2 * Math.log(r() + 1e-9)) * Math.cos(2 * Math.PI * r());
  // An allele as common as it is in the wild (or in a breed: `freq` {allele: share}).
  function wildAllele(L, r, freq) {
    const f = a => (freq && a.a in freq ? freq[a.a] : a.freq);
    let x = r() * L.alleles.reduce((s, a) => s + f(a), 0);
    for (const a of L.alleles) { x -= f(a); if (x <= 0) return a.a; }
    return L.alleles[L.alleles.length - 1].a;
  }
  const OWN = 16;
  // A species' gene file with the core traits after its own loci (each locus gets its `slot`),
  // as the game reads it.
  function withCore(genes, core) {
    const g = Object.assign({}, genes, { loci: genes.loci.slice(0, OWN).map((L, i) => Object.assign({}, L, { slot: i })) });
    (core.loci || []).forEach((C, i) => {
      const own = g.loci.find(L => L.id === C.id);
      if (own) { if (own.h === undefined) own.h = C.h; return; }
      const t = (genes.traits || {})[C.id];
      g.loci.push(Object.assign({}, C, { slot: OWN + i }, t ? { mean: t.mean, sd: t.sd } : {}));
    });
    return g;
  }
  // A new animal (of `breed`, from the species' `breeds`, or of `habitat`'s kind): one copy on
  // the X for males (an X-linked locus), two otherwise.
  function random(genes, r, sex, breed, habitat) {
    sex = sex || (r() < 0.5 ? "female" : "male");
    const over = [(breed && genes.breeds && genes.breeds[breed]) || {}, (habitat && genes.habitats && genes.habitats[habitat]) || {}];
    const genome = {};
    for (const L of genes.loci) {
      if (L.quantitative) {
        const q = over.map(b => (b.quantitative || {})[L.id]).find(Boolean) || L;
        genome[L.id] = [0, 1].map(() => clamp01(q.mean + q.sd * gauss(r)));
      } else {
        const freq = Object.assign({}, ...over.slice().reverse().map(b => (b.freq || {})[L.id] || {}));
        genome[L.id] = (L.xlinked && sex === "male" ? [0] : [0, 1]).map(() => wildAllele(L, r, freq));
      }
    }
    return { sex, genome, seed: Math.floor(r() * 1e9), breed: breed || null, habitat: habitat || null, stage: 3 };
  }
  // A baby: one allele of each locus from each parent (a son gets his X from his mother only),
  // quantitative traits drift a little, and now and then (1%) an allele mutates.
  function breed(genes, mother, father, r) {
    const sex = r() < 0.5 ? "female" : "male";
    const one = a => a[Math.floor(r() * a.length)];
    const genome = {};
    for (const L of genes.loci) {
      const m = mother.genome[L.id], f = father.genome[L.id];
      if (L.quantitative) { genome[L.id] = [one(m), one(f)].map(v => clamp01(v + 0.05 * gauss(r))); continue; }
      const g = L.xlinked ? (sex === "female" ? [one(m), f[0]] : [one(m)]) : [one(m), one(f)];
      if (r() < 0.01) g[Math.floor(r() * g.length)] = wildAllele(L, r);
      genome[L.id] = g;
    }
    const breed = mother.breed === father.breed ? mother.breed : "mixed";
    return { sex, genome, seed: Math.floor(r() * 1e9), breed, stage: 0 };
  }
  // What shows at a locus: the most dominant allele present.
  function shows(genes, a, id) {
    const L = genes.loci.find(l => l.id === id), have = a.genome[id];
    for (const x of L.alleles) if (have.includes(x.a)) return x.a;
    return have[0];
  }
  const copies = (a, id, x) => (a.genome[id] || []).filter(v => v === x).length;
  // A quantitative trait's genes: the mean of its two values (what it passes on).
  const genetic = (a, id) => a.genome[id] ? a.genome[id].reduce((s, v) => s + v, 0) / a.genome[id].length : 0.5;
  // How young it is: 1 a baby, 0 grown.
  const youth = a => (3 - Math.min(3, a.stage ?? 3)) / 3;
  // A quantitative trait as it shows: its genes, its own chance for the share genes don't decide
  // (from its seed: not passed on), and what its sex and youth shift (the game's `Read::qv`).
  let G = null;  // the gene file `phenotype` is reading against
  function qv(a, id) {
    const L = G && G.loci.find(l => l.id === id);
    let v = genetic(a, id);
    if (!L) return v;
    const h = Math.max(0.05, Math.min(1, L.h ?? 1));
    if (h < 1) {
      const u1 = hash3(L.slot, 1, 7, a.seed | 0), u2 = hash3(L.slot, 2, 7, a.seed | 0);
      v += L.sd * Math.sqrt((1 - h) / h) * Math.sqrt(-2 * Math.log(u1 + 1e-6)) * Math.cos(2 * Math.PI * u2);
    }
    v += ((G.sexes || {})[a.sex] || {})[id] || 0;
    v += youth(a) * (((G.young || {}).traits || {})[id] || 0);
    return clamp01(v);
  }
  function genotype(genes, a) {
    return genes.loci.map(L => L.quantitative ? `${L.id} ${qv(a, L.id).toFixed(2)}` : `${L.id} ${a.genome[L.id].join("/")}`).join(" \u00b7 ");
  }

  // ---------------------------------------------------------------- colors

  const mix = (a, b, t) => [0, 1, 2].map(i => a[i] + (b[i] - a[i]) * t);
  const lighten = (c, t) => mix(c, [246, 240, 230], t);
  const darken = (c, t) => mix(c, [24, 18, 18], t);
  const WHITE = [242, 238, 230];
  const BLUE_EYE = [92, 150, 214];  // eyes without pigment

  // ---------------------------------------------------------------- phenotypes per species

  const PHENOTYPES = {
    dog(g, a) {
      const brown = shows(g, a, "B") === "b", dil = shows(g, a, "D") === "d";
      const eu = brown ? (dil ? [150, 128, 124] : [94, 58, 40]) : (dil ? [92, 98, 108] : [38, 32, 32]);
      const euName = brown ? (dil ? "Lilac" : "Chocolate") : (dil ? "Blue" : "Black");
      const I = qv(a, "I");
      let pheo = I < 0.5 ? mix([236, 216, 180], [206, 152, 92], I * 2) : mix([206, 152, 92], [168, 84, 38], (I - 0.5) * 2);
      if (dil) pheo = mix(pheo, [198, 182, 166], 0.35);
      const pheoName = I < 0.3 ? "Cream" : I < 0.7 ? "Fawn" : "Red";
      const E = shows(g, a, "E"), K = shows(g, a, "K"), A = shows(g, a, "A");
      const c = { base: eu, under: lighten(eu, 0.06), white: 0 };
      let name;
      if (E === "e") {
        Object.assign(c, { base: pheo, under: lighten(pheo, 0.3) }); name = pheoName;
      } else if (K === "KB") {
        name = euName; c.eu = true;
      } else if (A === "ay") {
        Object.assign(c, { base: pheo, under: lighten(pheo, 0.3), pattern: { kind: "sable", color: eu, amount: 0.35 + 0.65 * qv(a, "Sh") } });
        name = `${pheoName} sable`;
      } else if (A === "at") {
        Object.assign(c, { points: pheo }); c.eu = true; name = `${euName} and tan`;
      } else { name = euName; c.eu = true; }
      if (E !== "e" && K === "kbr" && A !== "a") {
        if (A === "ay") c.pattern = { kind: "brindle", color: eu, amount: 0.8 };
        else c.pointsPattern = { kind: "brindle", color: eu, amount: 0.7 };
        name += " brindle";
      }
      if (E === "Em") { c.mask = eu; name += ", masked"; }
      if (shows(g, a, "M") === "M" && c.eu) { c.merle = mix(eu, [184, 186, 194], 0.6); name = name.replace(euName, euName + " merle"); if (qv(a, "Ey") > 0.6) c.eye = BLUE_EYE; }
      const sp = copies(a, "S", "sp");
      c.white = [0.08, 0.34, 0.66][sp];
      if (sp === 2) name += " parti"; else if (sp === 1) name += " with white";
      c.name = name;
      return c;
    },

    cat(g, a) {
      const B = shows(g, a, "B"), dil = shows(g, a, "D") === "d";
      const EU = { B: [[44, 38, 38], [112, 118, 128]], b: [[92, 58, 42], [170, 152, 148]], bl: [[130, 80, 50], [190, 164, 144]] };
      const EUN = { B: ["Black", "Blue"], b: ["Chocolate", "Lilac"], bl: ["Cinnamon", "Fawn"] };
      const eu = EU[B][dil ? 1 : 0], euName = EUN[B][dil ? 1 : 0];
      const pheo = dil ? [234, 204, 156] : [214, 124, 56], pheoName = dil ? "Cream" : "Red";
      const Ru = qv(a, "Ru");
      const ground = mix(lighten(eu, 0.5), [200, 150, 92], 0.35 + 0.4 * Ru);
      const o = copies(a, "O", "O"), red = a.sex === "male" ? o === 1 : o === 2, tortie = a.sex === "female" && o === 1;
      const tabby = shows(g, a, "A") === "A" || red;
      const kind = shows(g, a, "Ta") === "Ta" ? "ticked" : shows(g, a, "Sp") === "Sp" ? "spotted" : shows(g, a, "Mc") === "Mc" ? "mackerel" : "classic";
      const tabbyName = { ticked: "ticked tabby", spotted: "spotted tabby", mackerel: "mackerel tabby", classic: "classic tabby" }[kind];
      const redCoat = { base: pheo, under: lighten(pheo, 0.3), pattern: { kind, color: darken(pheo, 0.38), amount: 1 } };
      const plain = tabby ? { base: ground, under: lighten(ground, 0.3), pattern: { kind, color: eu, amount: 1 } }
                          : { base: eu, under: lighten(eu, 0.05) };
      let c, name;
      if (red) { c = redCoat; name = `${pheoName} ${tabbyName}`; }
      else if (tortie) {
        c = Object.assign({}, plain, { patches: redCoat });
        name = `${euName} ${tabby ? "torbie" : "tortoiseshell"}`;
      } else { c = plain; name = tabby ? `${euName === "Black" ? "Brown" : euName} ${tabbyName}` : euName; }
      if (copies(a, "C", "cs") === 2) {  // colorpoint: a pale body, the color on the points
        const point = red ? pheo : eu;
        c = { base: mix(c.base, [238, 226, 206], 0.78), under: [242, 234, 220], ext: { color: point, amount: 1 },
              pattern: c.pattern ? Object.assign({}, c.pattern, { amount: 0.2 }) : null };
        name = `${{ Black: "Seal", Blue: "Blue", Chocolate: "Chocolate", Lilac: "Lilac", Cinnamon: "Cinnamon", Fawn: "Fawn" }[euName]} point${red ? " (flame)" : ""}`;
        c.eye = BLUE_EYE;  // the same gene keeps the eyes' pigment from forming
      }
      c.white = [0.03, 0.36, 0.7][copies(a, "S", "S")];
      if (copies(a, "S", "S") === 2) name += ", van"; else if (copies(a, "S", "S") === 1) name += " with white";
      if (shows(g, a, "W") === "W") { c.white = 1.2; name = "White"; if (qv(a, "Ey") > 0.45) c.eye = BLUE_EYE; }
      c.name = name;
      return c;
    },

    capybara(g, a) {
      const Dk = qv(a, "Dk"), Ru = qv(a, "Ru"), Bl = qv(a, "Bl");
      let base = mix([190, 142, 96], [84, 56, 40], Dk);
      base = mix(base, Ru > 0.5 ? [150, 80, 42] : [118, 108, 98], Math.abs(Ru - 0.5) * 0.7);
      if (copies(a, "Le", "le") === 2) {
        return { name: "Leucistic", base: [226, 208, 176], under: [238, 226, 204], white: 0 };
      }
      const name = (Dk < 0.33 ? "Light tan" : Dk < 0.66 ? "Brown" : "Dark brown") + (Ru > 0.62 ? ", reddish" : Ru < 0.38 ? ", greyish" : "");
      return { name, base, under: lighten(base, 0.12 + 0.4 * Bl), ext: { color: darken(base, 0.4), amount: 0.7 }, white: 0 };
    },

    raptor(g, a) {
      const P = { br: [[104, 80, 60], "Umber"], ru: [[152, 86, 50], "Rufous"], gr: [[112, 110, 108], "Ash"], sa: [[194, 160, 110], "Sandy"] }[shows(g, a, "Pg")];
      const Dk = qv(a, "Dk");
      let base = Dk > 0.5 ? darken(P[0], (Dk - 0.5) * 0.7) : lighten(P[0], (0.5 - Dk) * 0.5);
      let under = mix(lighten(base, 0.5), [226, 210, 178], 0.4), name = P[1];
      const kinds = { bars: "bars", bands: "bands", spots: "rosettes", speckle: "speckle", plain: null };
      const pt = shows(g, a, "Pt"), kind = kinds[pt];
      let pattern = kind ? { kind, color: darken(base, 0.5), amount: 1 } : null;
      if (kind) name += { bars: ", barred", bands: ", banded", rosettes: ", spotted", speckle: ", speckled" }[kind];
      let saddle = null;
      if (copies(a, "Sd", "sd") === 2) { saddle = { kind: "saddle", color: darken(base, 0.55), amount: 0.9 }; name += ", saddled"; }
      if (copies(a, "Mel", "mel") === 2) { base = [44, 38, 36]; under = [74, 66, 60]; pattern = pattern && Object.assign({}, pattern, { color: [24, 20, 20], amount: 0.4 }); name = "Melanistic"; }
      if (copies(a, "Lc", "lc") === 2) { base = [228, 218, 200]; under = [244, 238, 226]; pattern = pattern && Object.assign({}, pattern, { color: [196, 184, 166], amount: 0.6 }); name = "Leucistic"; }
      return { name, base, under, pattern, pattern2: saddle, white: 0 };
    },
  };
  PHENOTYPES.wyvern = function (g, a) {
    // Scales and an accent (the wing skin, the spikes' tips), as the show's dragons wear them.
    const SC = { bronze: [[132, 92, 52], [96, 54, 38], "Bronze"], green: [[58, 78, 52], [150, 104, 52], "Green"],
                 black: [[40, 38, 42], [128, 36, 30], "Black"], red: [[118, 40, 34], [52, 26, 24], "Red"],
                 white: [[222, 212, 190], [196, 150, 70], "Cream"], slate: [[92, 92, 98], [112, 70, 60], "Slate"] };
    const sc = shows(g, a, "Sc"), [scales, accent, cname] = SC[sc] || SC.slate;
    const Dk = qv(a, "Dk");
    const base = Dk > 0.5 ? darken(scales, (Dk - 0.5) * 0.6) : lighten(scales, (0.5 - Dk) * 0.4);
    const under = mix(lighten(base, 0.3), [170, 140, 110], 0.4);
    const breath = shows(g, a, "Br") || "fire";
    const ink = sc === "white" ? mix(accent, base, 0.3) : darken(base, 0.5);
    const MK = { bands: ["bands", ", banded"], blotches: ["blotches", ", blotched"], mottle: ["mottle", ", mottled"], stripes: ["streaks", ", striped"] };
    const mk = a.genome.Mk ? MK[shows(g, a, "Mk")] : null;
    return { name: `${cname}${mk ? mk[1] : ""}, breathes ${breath}`, base, under, points: accent,
             pointsReach: (qv(a, "Ac") - 0.5) * 2, pattern: mk ? { kind: mk[0], color: ink, amount: 0.85 } : null, white: 0 };
  };
  PHENOTYPES.utahraptor = PHENOTYPES.raptor;
  PHENOTYPES.osmolskae = PHENOTYPES.raptor;  // Velociraptor's second species: the same loci

  function phenotype(genes, a) {
    G = genes;
    const c = PHENOTYPES[genes.species](genes, a);
    core(genes, a, c);
    c.seed = a.seed;
    G = null;
    return c;
  }

  // The core traits on a species' look, the same for every species (the game's `core`).
  function core(g, a, c) {
    const own = id => g.loci.some(L => L.id === id && (L.slot ?? 0) < OWN);
    const recolor = f => {
      c.base = f(c.base); c.under = f(c.under); if (c.points) c.points = f(c.points);
      if (c.patches) { c.patches = Object.assign({}, c.patches, { base: f(c.patches.base), under: f(c.patches.under) }); }
    };
    if (!own("Dk") && a.genome.Dk) {
      const dk = qv(a, "Dk");
      recolor(x => dk > 0.5 ? darken(x, (dk - 0.5) * 0.5) : lighten(x, (0.5 - dk) * 0.35));
    }
    const t = (qv(a, "Wm") - 0.5) * 2;
    const warm = x => [x[0] * (1 + 0.07 * t), x[1] * (1 + 0.015 * t), x[2] * (1 - 0.09 * t)];
    recolor(warm);
    for (const k of ["pattern", "pattern2", "pointsPattern"]) if (c[k]) c[k] = Object.assign({}, c[k], { color: warm(c[k].color) });
    if (c.white > 0 && c.white < 1) c.white = Math.min(0.97, c.white * (0.6 + 0.8 * qv(a, "Wh")));
    c.marks = { size: qv(a, "Ms"), density: qv(a, "Md"), contrast: qv(a, "Ct"), edge: qv(a, "Me"), stretch: qv(a, "Mx") };
    c.scale = g.mark_scale || 1;
    if (!c.eye && g.eyes && g.eyes.length) {
      const e = qv(a, "Ey") * (g.eyes.length - 1), i = Math.min(g.eyes.length - 1, Math.floor(e));
      c.eye = mix(g.eyes[i], g.eyes[Math.min(i + 1, g.eyes.length - 1)], e - i);
    }
    const y = youth(a);
    if (g.young && g.young.pattern && y > 0 && !c.pattern2) c.pattern2 = { kind: g.young.pattern, color: lighten(c.base, 0.5), amount: y };
  }

  // ---------------------------------------------------------------- patterns in 3D (model units)

  function hash3(x, y, z, s) {
    let h = Math.imul(x | 0, 374761393) ^ Math.imul(y | 0, 668265263) ^ Math.imul(z | 0, 2147483647) ^ Math.imul(s | 0, 1274126177);
    h = Math.imul(h ^ (h >>> 13), 1103515245);
    return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
  }
  function noise3(x, y, z, s) {
    const xi = Math.floor(x), yi = Math.floor(y), zi = Math.floor(z);
    const f = t => t * t * (3 - 2 * t), u = f(x - xi), v = f(y - yi), w = f(z - zi);
    const L = (a, b, t) => a + (b - a) * t;
    const c = (dx, dy, dz) => hash3(xi + dx, yi + dy, zi + dz, s);
    return L(L(L(c(0, 0, 0), c(1, 0, 0), u), L(c(0, 1, 0), c(1, 1, 0), u), v),
             L(L(c(0, 0, 1), c(1, 0, 1), u), L(c(0, 1, 1), c(1, 1, 1), u), v), w);
  }
  const frac = v => v - Math.floor(v);
  const smooth = (v, a, b) => { const t = clamp01((v - a) / (b - a)); return t * t * (3 - 2 * t); };
  const band = (t, width) => { const f = frac(t); return smooth(f, 0, 0.08) * (1 - smooth(f, width - 0.08, width)); };

  const DEFAULT_MARKS = { size: 0.5, density: 0.5, contrast: 0.5, edge: 0.5, stretch: 0.5 };
  // How much a pattern covers a texel (0..1): p in model units (x side to side, y up, z along the
  // body; divided by the species' `mark_scale`), u how far toward the underside, m how this
  // individual's marks are drawn: bigger marks are the pattern drawn on a smaller body, stretched
  // ones on a body shortened along its length, ragged ones on a warped body, denser ones with
  // wider marks (the game's `pattern`).
  function pattern(kind, p, u, s, m) {
    m = m || DEFAULT_MARKS;
    const k = Math.pow(2, (m.size - 0.5) * 1.6), st = Math.pow(2, (m.stretch - 0.5) * 1.2);
    const q = [p[0] / k * Math.sqrt(st), p[1] / k * Math.sqrt(st), p[2] / (k * st)];
    const warp = 0.5 + 3 * m.edge;
    const n = o => (noise3(q[0] * 0.4, q[1] * 0.4, q[2] * 0.4, s + o) - 0.5) * warp;
    const x = q[0] + n(11), y = q[1] + n(12), z = q[2] + n(13);
    const d = m.density - 0.5, w = width => Math.max(0.1, Math.min(0.85, width * (1 + 1.1 * d)));
    switch (kind) {
      case "mackerel": case "bars": return band((z + 1.6 * Math.sin(y * 0.45) + 0.5 * Math.abs(x)) / 5.5, w(0.32));
      case "classic": { const r = Math.hypot(z - 2, y - 14); return band(r / 6.5 + 0.15 * noise3(x * 0.3, y * 0.3, z * 0.3, s), w(0.42)); }
      case "spotted": return band((z + 1.6 * Math.sin(y * 0.45)) / 5.5, w(0.4)) * smooth(noise3(x * 0.55, y * 0.55, z * 0.55, s), 0.45 - 0.3 * d, 0.55 - 0.3 * d);
      case "ticked": return 0.35 * smooth(noise3(x * 1.4, y * 1.4, z * 1.4, s), 0.55 - 0.3 * d, 0.7 - 0.3 * d);
      case "bands": return band((z + 0.4 * y) / 7, w(0.38));
      case "streaks": return band((y + 1.5 * noise3(z * 0.08, x * 0.2, 0, s)) / 4.5, w(0.3));
      case "rosettes": {
        const cx = Math.floor(x / 4), cy = Math.floor(y / 4), cz = Math.floor(z / 4);
        if (hash3(cx, cy, cz, s) > 0.55 + 0.6 * d) return 0;
        const dist = Math.hypot(x - (cx + 0.5) * 4, z - (cz + 0.5) * 4, (y - (cy + 0.5) * 4) * 0.6);
        return smooth(dist, 0.6, 0.8) * (1 - smooth(dist, 1.5, 1.7));
      }
      case "blotches": {  // solid spots, one at a jittered place in each cell (a reptile's blotches)
        const cx = Math.floor(x / 6), cy = Math.floor(y / 6), cz = Math.floor(z / 6);
        let best = 0;
        for (const [ox, oy, oz] of [[0, 0, 0], [1, 0, 0], [-1, 0, 0], [0, 0, 1], [0, 0, -1], [0, 1, 0], [0, -1, 0]]) {
          const i = cx + ox, j = cy + oy, l = cz + oz;
          if (hash3(i, j, l, s + 21) > 0.5 + 0.6 * d) continue;
          const at = (o, base) => (base + 0.2 + 0.6 * hash3(i, j, l, s + o)) * 6;
          const dist = Math.hypot(x - at(22, i), (y - at(23, j)) * 0.8, z - at(24, l));
          const r = 1.4 + 1.2 * hash3(i, j, l, s + 25);
          best = Math.max(best, 1 - smooth(dist, r - 0.3, r + 0.3));
        }
        return best;
      }
      case "mottle": {
        const v = noise3(x * 0.22, y * 0.22, z * 0.22, s + 31) * 0.65 + noise3(x * 0.6, y * 0.6, z * 0.6, s + 32) * 0.35;
        return smooth(v, 0.55 - 0.25 * d, 0.62 - 0.25 * d);
      }
      case "speckle": return hash3(Math.floor(x), Math.floor(y), Math.floor(z), s) < 0.14 * Math.max(0.2, 1 + 2 * d) ? 1 : 0;
      case "saddle": return smooth(0.55 + 0.3 * d - u, 0, 0.25);
      case "sable": return smooth(0.75 + 0.3 * d - u, 0, 0.5) * (0.55 + 0.45 * noise3(x * 0.6, y * 0.6, z * 0.6, s));
      case "brindle": return band((z + 2.5 * noise3(x * 0.25, y * 0.25, z * 0.25, s) * 3) / 2.2, w(0.3));
      // The paint screen's stencils (not genes).
      case "stripes": return band(z / 4.5, 0.4);
      case "spots": return smooth(noise3(x * 0.4, y * 0.4, z * 0.4, s), 0.62, 0.68);
      case "patches": return smooth(noise3(x * 0.15, y * 0.15, z * 0.15, s), 0.5 - 0.25 * d, 0.56 - 0.25 * d);
      default: return 0;
    }
  }

  // ---------------------------------------------------------------- light (paint.tones)

  function rgb2hsv([r, g, b]) {
    r /= 255; g /= 255; b /= 255;
    const mx = Math.max(r, g, b), mn = Math.min(r, g, b), d = mx - mn;
    let h = 0;
    if (d) h = mx === r ? ((g - b) / d) % 6 : mx === g ? (b - r) / d + 2 : (r - g) / d + 4;
    return [((h / 6) + 1) % 1, mx ? d / mx : 0, mx];
  }
  function hsv2rgb(h, s, v) {
    const i = Math.floor(h * 6), f = h * 6 - i, p = v * (1 - s), q = v * (1 - f * s), t = v * (1 - (1 - f) * s);
    const [r, g, b] = [[v, t, p], [q, v, p], [p, v, t], [p, q, v], [t, p, v], [v, p, q]][((i % 6) + 6) % 6];
    return [r * 255, g * 255, b * 255];
  }
  // A shade `k` ramp steps from a base color (negative darker), hue-shifted: shadows warm and a
  // little more saturated, lights toward yellow and a little less.
  function tone(rgb, k, shadowHue = 0.02) {
    const [h, s, v] = rgb2hsv(rgb);
    const toward = (h0, target, amt) => { const d = ((target - h0 + 0.5) % 1 + 1) % 1 - 0.5; return ((h0 + Math.sign(d) * Math.min(Math.abs(d), amt)) % 1 + 1) % 1; };
    if (k < 0) return hsv2rgb(toward(h, shadowHue, 0.025 * -k), Math.min(1, s + 0.06 * -k), Math.max(v * (1 - 0.15 * -k), 0.08));
    return hsv2rgb(toward(h, 0.12, 0.02 * k), s * (1 - 0.13 * k), Math.min(0.96, v * (1 + 0.09 * k)));
  }

  // ---------------------------------------------------------------- the renderer

  // layers: {w, h, info, pos, props, props2 (RGBA Uint8 arrays, w*h), bounds: [lo, hi]};
  // base: the painted texture's RGBA at least w*h (it may be padded wider/taller: `stride`).
  function render(layers, base, stride, look, paint) {
    const { w, h, info, pos, props, props2, bounds } = layers;
    const out = new Uint8ClampedArray(base);
    const [lo, hi] = bounds, s = look.seed | 0;
    // The eyes' iris in its color, keeping its painted light (the game's `render`).
    const lum = o => 0.3 * base[o] + 0.59 * base[o + 1] + 0.11 * base[o + 2];
    let brightest = 0;
    if (look.eye) for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
      if (props2[(y * w + x) * 4 + 2] > 127) brightest = Math.max(brightest, lum((y * stride + x) * 4));
    }
    paint = paint || {};
    const coatBase = paint.base || look.base, coatUnder = paint.under || look.under;
    const markColor = c => paint.markings || c;
    for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
      const i = (y * w + x) * 4, o = (y * stride + x) * 4;
      if (look.eye && props2[i + 2] > 127 && brightest > 0) {
        const k = lum(o) / brightest;
        out[o] = look.eye[0] * k; out[o + 1] = look.eye[1] * k; out[o + 2] = look.eye[2] * k;
        continue;
      }
      if (info[i] < 128) continue;
      const level = info[i + 1] / 10, u = info[i + 2] / 255;
      const p = [0, 1, 2].map(k => (lo[k] + pos[i + k] / 255 * (hi[k] - lo[k])) / Math.max(0.01, look.scale || 1));
      const white = props[i] / 255, points = props[i + 1] / 255, mask = props[i + 2] / 255;
      const ext = props2[i] / 255, zone = props2[i + 1] / 255;
      const m = look.marks || DEFAULT_MARKS, contrast = 0.45 + 1.1 * m.contrast;  // stark marks show fully
      let b = coatBase, un = coatUnder, pat = look.pattern;
      if (look.patches && !paint.base && pattern("patches", p, u, s + 7, m) > 0.5) {
        b = look.patches.base; un = look.patches.under; pat = look.patches.pattern;
      }
      let c = mix(b, un, u);
      if (look.merle && !paint.base) {
        const k = 0.3 / Math.pow(2, (m.size - 0.5) * 1.6);
        c = mix(c, look.merle, 0.8 * (1 - smooth(noise3(p[0] * k, p[1] * k, p[2] * k, s + 3), 0.58, 0.64)));
      }
      for (const pt of [pat, look.pattern2]) {
        if (pt) c = mix(c, markColor(pt.color), Math.min(1, pattern(pt.kind, p, u, s, m) * zone * (pt.amount ?? 1) * contrast));
      }
      if (look.points || paint.points) {
        const at = 0.4 - 0.3 * (look.pointsReach || 0), t = smooth(points, at, at + 0.22);
        let pc = paint.points || look.points;
        if (look.pointsPattern) pc = mix(pc, look.pointsPattern.color, Math.min(1, pattern(look.pointsPattern.kind, p, u, s, m) * look.pointsPattern.amount * contrast));
        c = mix(c, pc, t);
      }
      if (look.mask) c = mix(c, look.mask, 0.9 * mask);
      if (look.ext) c = mix(c, look.ext.color, smooth(ext * look.ext.amount, 0.25, 0.9));
      if (paint.stencil && paint.stencil.color) c = mix(c, paint.stencil.color, pattern(paint.stencil.kind, p, u, s + 11) * zone);
      if (look.white > 0) {  // white's edges as ragged as the marks'
        const wn = white + (noise3(p[0] * 0.22, p[1] * 0.22, p[2] * 0.22, s + 5) - 0.5) * (0.15 + 0.3 * m.edge), thr = 1 - look.white;
        c = mix(c, WHITE, smooth(wn, thr, thr + 0.05));
      }
      const t = tone(c, -3 + level * 0.25);
      out[o] = t[0]; out[o + 1] = t[1]; out[o + 2] = t[2];
    }
    return out;
  }

  root.Coats = { rng, withCore, random, breed, phenotype, genotype, render, tone };
})(typeof window !== "undefined" ? window : globalThis);
