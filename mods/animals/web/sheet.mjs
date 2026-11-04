// Writes every species' settings (mods/animals/SPECIES.md) from the game's species table, through
// the WASM (`core/src/sheet.rs`). Run after changing a species:
//
//   mods/animals/web/build.sh && node mods/animals/web/sheet.mjs [animals.wasm]
import fs from "fs";
import path from "path";

const here = path.dirname(new URL(import.meta.url).pathname);
const file = process.argv[2] || path.join(here, "animals.wasm");
const { instance } = await WebAssembly.instantiate(fs.readFileSync(file), {});
const w = instance.exports, enc = new TextEncoder(), dec = new TextDecoder();
const b = enc.encode(JSON.stringify({ op: "sheet" })), p = w.alloc(b.length);
new Uint8Array(w.memory.buffer, p, b.length).set(b);
const out = w.call(p, b.length);
const { markdown } = JSON.parse(dec.decode(new Uint8Array(w.memory.buffer, out, w.out_len())));
const to = path.join(here, "..", "SPECIES.md");
fs.writeFileSync(to, markdown);
console.log(`wrote ${to}`);
