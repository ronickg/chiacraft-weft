//! The one wasmtime `Engine` of the process and the limits every mod's store gets
//! (`docs/sandboxed-mods.md`, "Limits"). Carried over from the Horizon-based project's
//! `horizon-mods`, where it was reviewed.
//!
//! - **Fuel cuts a call off** (`consume_fuel`): about one unit per WebAssembly instruction, counted
//!   the same on every machine, so a call that runs out stops at the same instruction everywhere.
//! - **The epoch watchdog is a backstop** (`epoch_interruption`): a thread ticks the engine's epoch
//!   every [`EPOCH`]; a call still running after [`WATCHDOG`] traps. Only a host stall or a loop
//!   the fuel checks somehow miss reaches it.
//! - **Deterministic floats and no threads**: NaNs are canonicalized, relaxed SIMD is off, and
//!   wasmtime's `threads` feature isn't compiled, so no shared memory or atomics wait.
//! - **Nothing else to import**: the linker holds only the calls of the mod's WIT world. No WASI:
//!   a component that asks for files, sockets, clocks or randomness of its own doesn't link.
use std::{sync::OnceLock, time::Duration};
use wasmtime::{Config, Engine, StoreLimits, StoreLimitsBuilder};

/// How often the watchdog thread ticks the epoch.
pub const EPOCH: Duration = Duration::from_millis(10);
/// How long a single call may run, in wall time, before the watchdog stops it.
pub const WATCHDOG: Duration = Duration::from_millis(500);
/// [`WATCHDOG`] in epochs.
pub const WATCHDOG_EPOCHS: u64 = (WATCHDOG.as_millis() / EPOCH.as_millis()) as u64;

/// A mod's linear memory cap.
pub const MEMORY: usize = 64 << 20;
/// Table elements a mod may hold.
pub const TABLE_ELEMENTS: usize = 10_000;
/// Core instances in one mod's component: its own module plus the shims `wit-component` adds.
pub const INSTANCES: usize = 8;
/// Linear memories: the module's own.
pub const MEMORIES: usize = 1;
/// Tables: the module's own and the shims'.
pub const TABLES: usize = 4;
/// The biggest component the game compiles: compiling is work the game does for someone else's
/// file.
pub const MAX_COMPONENT_BYTES: usize = 16 << 20;

/// Fuel per call, by what the call is. A world tick runs every tick in every region and questions
/// and chain steps sit in hot paths, so theirs are smallest.
pub mod fuel {
    pub const INIT: u64 = 1_000_000_000;
    pub const COMMAND: u64 = 200_000_000;
    pub const EVENT: u64 = 50_000_000;
    pub const TICK: u64 = 10_000_000;
    pub const QUESTION: u64 = 2_000_000;
    pub const MODIFY: u64 = 2_000_000;
}

pub fn config() -> Config {
    let mut config = Config::new();
    config
        .wasm_component_model(true)
        .consume_fuel(true)
        .epoch_interruption(true)
        .cranelift_nan_canonicalization(true)
        .wasm_relaxed_simd(false)
        .relaxed_simd_deterministic(true);
    config
}

/// The process's engine, made on first use with its watchdog thread.
pub fn engine() -> Result<&'static Engine, String> {
    static ENGINE: OnceLock<Result<Engine, String>> = OnceLock::new();
    ENGINE
        .get_or_init(|| {
            let engine = Engine::new(&config()).map_err(|e| e.to_string())?;
            let ticking = engine.weak();
            std::thread::Builder::new()
                .name("Mod watchdog".into())
                .spawn(move || {
                    while let Some(engine) = ticking.upgrade() {
                        engine.increment_epoch();
                        drop(engine);
                        std::thread::sleep(EPOCH);
                    }
                })
                .map_err(|e| e.to_string())?;
            Ok(engine)
        })
        .as_ref()
        .map_err(Clone::clone)
}

/// A store's limits: memory, table elements, instances, memories and tables, a failed grow
/// trapping instead of returning -1.
pub fn limits() -> StoreLimits {
    StoreLimitsBuilder::new()
        .memory_size(MEMORY)
        .table_elements(TABLE_ELEMENTS)
        .instances(INSTANCES)
        .memories(MEMORIES)
        .tables(TABLES)
        .trap_on_grow_failure(true)
        .build()
}
