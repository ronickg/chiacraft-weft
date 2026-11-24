# shadercheck2

Validates chiacraft's shader pack (`mods/base/shaderpack`) offline, without launching the game. Each pack file is put together the way the engine does it: the shader's prelude and `common.wgsl`, then `MIST_TOP`, `look.wgsl` and the file, then the hook dispatch. The CALM and DRY versions are checked too. Each result is validated with naga. Then each entry point goes the game's Mac path: naga writes SPIR-V with the Vulkan backend's options, SPIRV-Cross (the `spirv-cross2` crate, as MoltenVK runs it) turns it into MSL 3.0 with argument buffers, and Apple's `metal` compiler must accept it. naga's own MSL output passed while MoltenVK's failed (11-24: `float4[3]` to `spvUnsafeArray`, no world opened on macOS). The .metal files go to `$TMPDIR/shadercheck2-msl`. `--no-msl` skips that pass.

```bash
cp Cargo.lock notes/crew/tester-tools/shadercheck2/   # pin the same crate versions as the game
(cd notes/crew/tester-tools/shadercheck2 && CARGO_TARGET_DIR=../../../../target nice -n 19 cargo build)
target/debug/shadercheck2 .            # takes a few minutes with the Metal pass
```

- It prints `ok` or `FAIL` for each shader and exits 1 if any fail.
- `MIST_TOP` is written in by hand (18.0, `vitals::air`). Change it here if that number changes.
- When the engine adds a shader that takes a pack file, or a new hook, add it to `files` in `src/main.rs`.
- Written 11-22 during the Weft sync. It lives in the repo because the chiacraft-archive copy of the old checker was wiped.
