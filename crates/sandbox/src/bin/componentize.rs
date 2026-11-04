//! `componentize <module.wasm> <out.wasm>`: turns a mod built with `chiacraft-mod-sdk` for
//! `wasm32-unknown-unknown` into the component a package carries as `code/server.wasm`.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, from, to] = &args[..] else {
        eprintln!("usage: componentize <module.wasm> <out.wasm>");
        std::process::exit(2);
    };
    let run = || -> Result<(), String> {
        let core = std::fs::read(from).map_err(|e| format!("{from}: {e}"))?;
        let component = chiacraft_sandbox::componentize(&core)?;
        let m = chiacraft_sandbox::Mod::load("componentize", &component).map_err(|e| format!("the component doesn't load: {e}"))?;
        std::fs::write(to, &component).map_err(|e| format!("{to}: {e}"))?;
        println!("{to}: {} bytes, registers:", component.len());
        for (i, h) in m.hooks().iter().enumerate() {
            println!("  {i}: {h:?}");
        }
        Ok(())
    };
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
