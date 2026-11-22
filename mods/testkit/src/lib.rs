//! Automated tests and scripted runs, written on the public client and server mod API.
//!
//! `weft --test all` runs every test (`--test furnace,armor` some, `--test list` names them,
//! `--test bench` the benchmarks), each in a fresh throwaway world, then writes
//! `run/test-reports/report.txt` and exits with the number of failed tests. Screenshots of the
//! run go to `run/test-shots/` for looking at; checks never compare pictures with references.
//! `--test all` runs several at once, `--jobs 1` one by one, `--jobs N` for a number; see the game's `main`.
//! `--test-keep` keeps the test worlds. `--watchdog 45` ends a run that draws no frame for 45 s
//! with exit code 70 and says where it stopped (`watchdog.rs`). `weft --world w --script file.txt` runs a line script (see `script.rs`).
//! With `TESTKIT_CI=1` (CI's software Vulkan) what `ci-skip.txt` names is left out, by name.
//! Without those options the mod only registers its message channels.

mod compare;
mod generation;
mod runner;
mod script;
mod server;
mod tests;
mod watchdog;

use std::sync::OnceLock;

use weft_sdk::*;

// Ours: no dependency on testmod (upstream's has one). testmod is loaded only for `--test` runs
// (`play = false`, run-dir/test-mods), and a `--script` run must start without it: with the
// dependency the loader turned testkit off and no script could run (Tester 2, 11-22).
declare_mod!(deps: [], init: init);

static PLAN: OnceLock<(Option<String>, Option<String>)> = OnceLock::new();

fn init(r: &mut Registry) {
    server::register(r);
    r.key(&runner::PROBE, "F9");
    // Compare mode (`compare.rs`): every control of a like-for-like look shot in one switch.
    r.config(&compare::COMPARE_MODE).bool(false);
    r.client_command("compare", "<state:word(on,off)> [overrides:text]", |cmd| {
        let on = cmd.text("state") == Some("on");
        let overrides = cmd.text("overrides").unwrap_or("").to_string();
        let said = compare::apply(&cmd.client(), on, &overrides);
        weft_sdk::info!("{said}");
        cmd.reply("command.testkit.compare", &[Arg::Text(&said)]);
    });
    // A branch of an engine command, handled by this mod (on the client, like /gizmo).
    r.client_command_branch("gizmo", "probe <n:int>", |cmd| {
        let n = cmd.int("n").unwrap_or(0);
        cmd.reply("command.testkit.gizmo_probe", &[Arg::Int(n as i64)]);
    });
    // An engine command a mod replaces (testkit is never shipped).
    r.command("regions", "", |cmd| cmd.reply("command.testkit.regions", &[]));
    // A command that never reaches the server: answers from the client it runs on.
    r.client_command("probe", "<what:word(alpha,beta)>", |cmd| {
        // A `ClientCommand` only ever runs here; the reply says so.
        let side = "command.testkit.client";
        let what = cmd.text("what").unwrap_or("").to_string();
        cmd.reply("command.testkit.probe", &[Arg::Text(&what), Arg::Key(side)]);
    });
    // Stops the main thread for n seconds, so the watchdog can be seen working
    // (`scripts/watchdog.txt`). testkit is never shipped.
    r.client_command("tk_freeze", "<secs:int>", |cmd| {
        let secs = cmd.int("secs").unwrap_or(1).clamp(0, 600) as u64;
        std::thread::sleep(std::time::Duration::from_secs(secs));
    });
    let (test, script) = (launch_option("test"), launch_option("script"));
    if test.is_none() && script.is_none() {
        return;
    }
    let _ = PLAN.set((test, script));
    r.client_frame(|c| {
        static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        watchdog::beat();
        if !STARTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
            if let Some(secs) = launch_option("watchdog")
                .and_then(|s| s.parse::<f32>().ok())
                .filter(|s| *s > 0.0)
            {
                watchdog::start(secs, format!("{}/test-reports", c.run_dir()));
            }
            start(c);
        }
        runner::frame(c);
    });
}

/// Tests of how quick the game is, which other processes on the machine would fail.
const ALONE: &[&str] = &["edit_while_loading", "skiff_smooth"];

/// This game's `--packages` options, for a copy of it a test starts: a copy loading other content
/// is refused as running different mods.
pub(crate) fn package_args() -> Vec<String> {
    let args: Vec<String> = std::env::args().collect();
    args.windows(2)
        .filter(|w| w[0] == "--packages")
        .flat_map(|w| w.to_vec())
        .collect()
}

/// What CI leaves out (`ci-skip.txt`: a test, `test/scenario` or `test/check` and why), only with
/// `TESTKIT_CI=1`: software rendering draws a few frames a second, and these count on more.
pub fn ci_skip(name: &str) -> Option<&'static str> {
    static ON: OnceLock<bool> = OnceLock::new();
    if !*ON.get_or_init(|| std::env::var("TESTKIT_CI").is_ok_and(|v| v == "1")) {
        return None;
    }
    include_str!("../ci-skip.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .find_map(|l| {
            let (n, why) = l.split_once(char::is_whitespace).unwrap_or((l, ""));
            (n == name).then(|| why.trim())
        })
}

/// Builds the queue on the main thread, where the runner lives.
/// Same conditions on every machine and for every test, whatever the one before left behind (a
/// failure halfway, or leaving a server, which reads the player's settings back).
pub(crate) fn same_conditions(c: &Client) {
    use weft_sdk::settings::*;
    // Tests look at what the hands hold, in the view.
    static FIRST_PERSON_MODEL: Config<bool> = Config::named("core:first_person_model");
    static AUTO_JUMP: Config<bool> = Config::named("core:auto_jump");
    let scripted = PLAN.get().is_some_and(|p| p.1.is_some());
    if !scripted {
        if FIRST_PERSON_MODEL.get() {
            FIRST_PERSON_MODEL.set(false);
        }
        if AUTO_JUMP.get() {
            AUTO_JUMP.set(false);
        }
    }
    // The player's packs would change every picture.
    let packs = !RESOURCE_PACKS.get().is_empty() || !SHADER_PACK.get().is_empty();
    LANGUAGE.set("en_us");
    GUI_SCALE.set(2);
    VIEW_DISTANCE.set(12);
    CLOUD_DISTANCE.set(6144);
    FOV.set(70.0);
    BRIGHTNESS.set(0.5);
    VSYNC.set(true);
    RESOURCE_PACKS.set("");
    SHADER_PACK.set("");
    DEBUG_GROUPS.set(u32::MAX);
    ADVANCED_TOOLTIPS.set(false);
    DEBUG_LINES.set("");
    if packs {
        c.reload_assets();
    }
}

fn start(c: &Client) {
    let Some((test, script)) = PLAN.get() else { return };
    if let Some(path) = script {
        let src = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                error!("cannot read script {path}: {e}");
                c.quit(2);
                return;
            }
        };
        let name = std::path::Path::new(path)
            .file_stem()
            .map_or("script".into(), |s| s.to_string_lossy().into_owned());
        runner::start(vec![runner::TestDef { name, run: Box::new(move |t| Box::pin(script::run(t, src.clone()))) }]);
        return;
    }
    // Before the first test too, so that it starts with them in force.
    same_conditions(c);
    let want = test.as_deref().unwrap_or("all");
    // Benchmarks run by name too, or all of them with `bench`.
    let all = match want {
        "bench" => tests::bench(),
        "all" | "list" => tests::all(),
        _ => tests::all().into_iter().chain(tests::bench()).collect(),
    };
    if want == "list" {
        for t in &all {
            info!("test {}", t.name);
        }
        c.quit(0);
        return;
    }
    let queue: Vec<runner::TestDef> = if want == "all" || want == "bench" {
        all
    } else {
        let names: Vec<&str> = want.split(',').map(str::trim).collect();
        if let Some(bad) = names.iter().find(|n| !all.iter().any(|t| t.name == **n)) {
            error!("no test named {bad}; --test list shows them");
            c.quit(2);
            return;
        }
        all.into_iter()
            .filter(|t| names.contains(&t.name.as_str()))
            .collect()
    };
    // One of several processes running them all (see the game's `--jobs`): every Nth.
    let shard = launch_option("test-shard");
    let warm = shard.is_some();
    let mut queue: Vec<runner::TestDef> = match shard {
        Some(s) if s == "alone" => queue
            .into_iter()
            .filter(|t| ALONE.contains(&t.name.as_str()))
            .collect(),
        Some(s) => {
            let part = |w: Option<&str>| w.and_then(|w| w.parse::<usize>().ok()).unwrap_or(1);
            let mut w = s.split('/');
            let (i, n) = (part(w.next()), part(w.next()));
            queue
                .into_iter()
                .filter(|t| !ALONE.contains(&t.name.as_str()))
                .enumerate()
                .filter(|(k, _)| k % n == i)
                .map(|(_, t)| t)
                .collect()
        }
        None => queue,
    };
    queue.retain(|t| match ci_skip(&t.name) {
        Some(why) => {
            runner::skip(&t.name, why);
            false
        }
        None => true,
    });
    if warm {
        queue.splice(0..0, crate::tests![tests::warm_up]);
    }
    runner::start(queue);
}
