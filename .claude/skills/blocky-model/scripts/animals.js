// The animals mod's own logic (mods/animals/core, the game's Rust), compiled to WASM
// (mods/animals/web/build.sh writes animals.wasm next to this page). Load once, then:
//
//   await Animals.ready;
//   Animals.call({ op: "random", species: "cat", seed: 1 })   // -> { genome, look, loci, build, .. }
//   Animals.paint(genomeHex, rgba, width, height)             // paints the coat into rgba in place
//
// Requests and answers are those of mods/animals/web/src/lib.rs; an answer with `error` was
// refused (as the game would refuse it).
(function (root) {
  "use strict";
  let w = null;
  const enc = new TextEncoder(), dec = new TextDecoder();
  const ready = fetch("animals.wasm")
    .then(r => { if (!r.ok) throw new Error("animals.wasm: " + r.status); return r.arrayBuffer(); })
    .then(b => WebAssembly.instantiate(b, {}))
    .then(m => { w = m.instance.exports; });
  function put(bytes) {
    const p = w.alloc(bytes.length);
    new Uint8Array(w.memory.buffer, p, bytes.length).set(bytes);
    return p;
  }
  function call(req) {
    const b = enc.encode(JSON.stringify(req)), p = put(b);
    const out = w.call(p, b.length);
    w.free(p, b.length);
    return JSON.parse(dec.decode(new Uint8Array(w.memory.buffer, out, w.out_len())));
  }
  const unhex = h => Uint8Array.from(h.match(/../g) || [], x => parseInt(x, 16));
  function paint(genome, rgba, width, height) {
    const g = unhex(genome), gp = put(g), px = put(rgba);
    w.paint(gp, g.length, px, width, height);
    rgba.set(new Uint8Array(w.memory.buffer, px, rgba.length));
    w.free(gp, g.length);
    w.free(px, rgba.length);
    return rgba;
  }
  root.Animals = { ready, call, paint, seed: () => (Math.random() * 4294967296) >>> 0 };
})(typeof window !== "undefined" ? window : globalThis);
