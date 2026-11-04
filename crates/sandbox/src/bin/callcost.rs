//! `callcost <component or module.wasm> [calls]`: times each kind of call a mod registered
//! (commands with no arguments, events with empty and 64-entry batches, questions, chain steps,
//! world ticks) over `calls` calls (default 20000), against [`NullGame`], and prints the
//! microseconds per call. A core module (straight from `cargo build`) is made into a component
//! first. Release builds only give numbers worth reading.
use std::sync::Arc;
use std::time::Instant;

use chiacraft_sandbox::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let Some(path) = args.get(1) else {
        eprintln!("usage: callcost <component or module.wasm> [calls]");
        std::process::exit(2);
    };
    let calls: u32 = args.get(2).and_then(|n| n.parse().ok()).unwrap_or(20_000);
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let wasm = if chiacraft_packages::code::api_of(&bytes).is_ok() { bytes } else { componentize(&bytes).expect("componentize") };
    let m = Arc::new(Mod::load("callcost", &wasm).expect("load"));
    let mut inst = Instance::new(m.clone(), None);
    let game = || Box::new(NullGame) as Box<dyn Game>;
    assert!(inst.start(game()), "the mod didn't start");
    let q = Query { entity: 1, other: 0, pos: (0, 64, 0), state: 1, item: 0, dim: 0, cause: 0, amount: 0.0 };
    let batch: Vec<EventValue> = (0..64)
        .map(|i| EventValue::BlockChange(BlockChange { player: 1, pos: (i, 64, 0), state: 1, item: 0 }))
        .collect();
    println!("{calls} calls each, microseconds per call:");
    for (i, hook) in m.hooks().iter().enumerate() {
        let cb = i as u32;
        let time = |label: &str, inst: &mut Instance, f: &mut dyn FnMut(&mut Instance)| {
            let start = Instant::now();
            for _ in 0..calls {
                f(inst);
            }
            let us = start.elapsed().as_secs_f64() * 1e6 / calls as f64;
            println!("  {i} {label}: {us:.2}");
        };
        match hook {
            Hook::Command { name, .. } => {
                time(&format!("/{name}"), &mut inst, &mut |inst| inst.command(game, cb, &Invocation { player: 1, args: Vec::new() }))
            }
            Hook::Event(e) => {
                time(&format!("{e:?}, empty batch"), &mut inst, &mut |inst| inst.event(game, cb, &[]));
                time(&format!("{e:?}, 64 block changes"), &mut inst, &mut |inst| inst.event(game, cb, &batch));
            }
            Hook::Question(qn) => time(&format!("{qn:?}"), &mut inst, &mut |inst| {
                inst.question(game, cb, q);
            }),
            Hook::Modify { chain, .. } => time(&format!("{chain:?}"), &mut inst, &mut |inst| {
                inst.modify(game, cb, q, 8.0, 8.0);
            }),
            Hook::WorldTick => time("world tick", &mut inst, &mut |inst| inst.world_tick(game, cb)),
        }
    }
    if let Some(why) = inst.stopped() {
        println!("(the mod stopped: {why})");
    }
}
