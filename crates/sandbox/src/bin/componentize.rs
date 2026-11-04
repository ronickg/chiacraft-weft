//! `componentize <module.wasm> <out.wasm>`: turns a mod built with `chiacraft-mod-sdk` for
//! `wasm32-unknown-unknown` into the component a package carries as `code/server.wasm`, and loads
//! it as that package (the id from `<out>/../../package.toml`) to check it.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, from, to] = &args[..] else {
        eprintln!("usage: componentize <module.wasm> <out.wasm>");
        std::process::exit(2);
    };
    let run = || -> Result<(), String> {
        let core = std::fs::read(from).map_err(|e| format!("{from}: {e}"))?;
        let component = chiacraft_sandbox::componentize(&core)?;
        // Loaded as the package it goes into (`<package>/code/server.wasm`), whose own blocks and
        // items its callbacks name.
        let package = std::path::Path::new(to)
            .parent()
            .and_then(|code| code.parent())
            .and_then(|dir| chiacraft_packages::read_manifest(dir).ok())
            .map_or_else(|| "componentize".to_string(), |m| m.id);
        let m = chiacraft_sandbox::Mod::load(&package, &component).map_err(|e| format!("the component doesn't load: {e}"))?;
        std::fs::write(to, &component).map_err(|e| format!("{to}: {e}"))?;
        println!("{to}: {} bytes, registers:", component.len());
        for (i, h) in m.hooks().iter().enumerate() {
            println!("  {i}: {h:?}");
        }
        for d in m.decls() {
            println!("  declares {d:?}");
        }
        Ok(())
    };
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
