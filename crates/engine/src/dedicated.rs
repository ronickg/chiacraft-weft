//! `weft --server`: the world and the network with nobody playing on this machine, no window and
//! no GPU. Options: `--world name` (made if new), `--seed n`, `--port n`, `--run-dir dir`. What it
//! keeps is where a game keeps it (`config/`, `saves/`). `config/whitelist.txt`, keys one a line,
//! limits who may join; without it anyone can, and nobody may do more than play until `op`.
//! `config/disabled_mods.txt`, ids one a line, leaves mods out as the mods screen does.
//! `server-account/` is the server's own identity (chiacraft, `chiacraft_account::open_server`):
//! made on the first start, its worlds' keys and certificates come from it, so the server's
//! addresses stay the same across restarts and need nobody's account.

use std::io::BufRead;

use crate::app::Options;
use crate::content::content;
use crate::server::{self, ServerConfig};
use crate::{config, modloader, net, saves, settings::Settings};

pub fn run(opts: Options) -> Result<(), String> {
    let off = modloader::off_file(&opts.run_dir, false);
    let _mods = modloader::load(&opts.mod_dirs, &opts.package_dirs, &opts.cache_dir, &off)?;
    crate::settings::migrate(&opts.run_dir);
    config::load(&opts.run_dir);
    if let Some(port) = crate::app::api::launch_options().get("port") {
        config::override_for_run("core:server_port", port);
    }
    if let Some(v) = opts.view {
        config::override_for_run("core:view_distance", &v.to_string());
    }
    let settings = Settings::new(&opts.run_dir);
    // Joining players' DIDs are read from this chain (chiacraft), when the world asks for them.
    crate::chiacraft::chain::configure(&settings.run_dir, &settings.chain(), settings.chain_peers());

    let name = opts.world.as_deref().unwrap_or("world");
    let saves_dir = settings.saves_dir();
    let mut meta = saves::list(&saves_dir)
        .into_iter()
        .find(|m| m.name == name)
        .unwrap_or_else(|| saves::create(&saves_dir, name, opts.seed.unwrap_or_else(saves::now_secs)));
    if meta.format > saves::SAVE_FORMAT {
        return Err(format!("world '{name}' was saved in format {}, this build reads {}", meta.format, saves::SAVE_FORMAT));
    }
    for m in meta.missing_mods(content()) {
        log::warn!("the world was played with mod {m}, which is not loaded");
    }
    // The world's own key (chiacraft): its address, which players join by. A server has an
    // identity of its own (`server-account` in its folder) that its worlds' keys come from, as a
    // player's account is for their worlds; nobody's account is needed to run one.
    chiacraft_account::open_server(&opts.run_dir).map_err(|e| format!("the server's identity: {e}"))?;
    let identity =
        std::sync::Arc::new(crate::world_address::for_world(&mut meta).map_err(|e| format!("'{name}' has no address of its own: {e}"))?);
    // chiacraft, test only: `--import-mc` loads a Minecraft world into a new world first.
    crate::chiacraft::mc_import::maybe_import(&meta, content());
    let storage = saves::Storage::open(meta.clone(), content());
    let whitelist = crate::paths::config(&opts.run_dir).join("whitelist.txt");
    let mut server = server::spawn(
        ServerConfig {
            seed: meta.seed,
            view_radius: settings.view_distance(),
            dedicated: true,
            whitelist: Some(whitelist),
            identity: Some(identity.clone()),
        },
        Some(storage),
    );
    net::host(&server, identity, name, &settings);
    // `--stats [secs]`: a line of tick times, traffic and the process's use every few seconds
    // (a load test, `crates/bots`). Traffic is counted with `--log net=debug`.
    if let Some(secs) = crate::app::api::launch_options().get("stats") {
        stats(server.shared.clone(), secs.parse().unwrap_or(5.0f64).max(1.0));
    }
    log::info!("server running world '{name}'; type stop to save and leave, anything else for help");

    // With no input at all (a service), it runs until stopped from outside.
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        if line.trim() == "stop" {
            server.stop();
            log::info!("saved, stopped");
            return Ok(());
        }
        let _ = server.console.send(line);
    }
    loop {
        std::thread::park();
    }
}

/// Every `secs`: the ticks run (time per tick, its phases, how many fit the second), players, loaded
/// cubes and entities, traffic each way by message kind, and the process's memory, threads and CPU
/// by thread name (Linux: `/proc`), so a load test sees which part of the server takes the time.
fn stats(shared: std::sync::Arc<server::Shared>, secs: f64) {
    use std::sync::atomic::Ordering;
    shared.stats.store(true, Ordering::Relaxed);
    let _ = std::thread::Builder::new().name("stats".into()).spawn(move || {
        let mut last_cpu = threads_cpu();
        let mut last = std::time::Instant::now();
        loop {
            std::thread::sleep(std::time::Duration::from_secs_f64(secs));
            let wall = last.elapsed().as_secs_f64();
            last = std::time::Instant::now();
            let samples = std::mem::take(&mut *shared.tick_samples.lock());
            let mut ticks: Vec<u32> = samples.iter().map(|s| s.0).collect();
            ticks.sort_unstable();
            let pct = |p: f64| ticks.get(((ticks.len().max(1) - 1) as f64 * p).round() as usize).copied().unwrap_or(0) as f64 / 1000.0;
            let n = samples.len().max(1) as f64;
            let avg = ticks.iter().map(|&t| t as f64).sum::<f64>() / n / 1000.0;
            let over = ticks.iter().filter(|&&t| t > 50_000).count();
            let phases: Vec<String> = server::PHASES
                .iter()
                .enumerate()
                .map(|(i, k)| format!("{k} {:.2}", samples.iter().map(|s| s.1[i] as f64).sum::<f64>() / n / 1000.0))
                .collect();
            let traffic = |to_server: bool| {
                let mut kinds = crate::diag::take_net(to_server);
                let (count, bytes) = kinds.iter().fold((0, 0), |a, k| (a.0 + k.1, a.1 + k.2));
                kinds.sort_by_key(|k| std::cmp::Reverse(k.2.max(k.1)));
                let top: Vec<String> = kinds
                    .iter()
                    .take(6)
                    .map(|(k, c, b)| format!("{k} {:.0}/s {:.1} KB/s", *c as f64 / wall, *b as f64 / wall / 1024.0))
                    .collect();
                format!("{:.0} msgs/s {:.1} KB/s ({})", count as f64 / wall, bytes as f64 / wall / 1024.0, top.join(", "))
            };
            let (out, inn) = (traffic(false), traffic(true));
            let cpu = threads_cpu();
            let mut by_name: Vec<(String, f64)> = Vec::new();
            for (name, ticks) in &cpu {
                let before = last_cpu.iter().find(|(n, _)| n == name).map_or(0.0, |c| c.1);
                by_name.push((name.clone(), (ticks - before) / wall));
            }
            last_cpu = cpu;
            let total: f64 = by_name.iter().map(|c| c.1).sum();
            by_name.sort_by(|a, b| b.1.total_cmp(&a.1));
            let cores: Vec<String> = by_name
                .iter()
                .take(8)
                .filter(|c| c.1 >= 0.005)
                .map(|(n, c)| format!("{n} {c:.2}"))
                .collect();
            let (rss, threads) = process_memory();
            log::info!(
                "server stats per {wall:.1} s: {} ticks ({:.1}/s), tick avg {avg:.2} p50 {:.2} p99 {:.2} max {:.2} ms, {over} over 50 ms | phases avg {} | players {}, cubes {}, entities {} | to clients {out} | to server {inn} | cpu {total:.2} cores ({}) | rss {rss:.0} MB, {threads} threads",
                samples.len(),
                samples.len() as f64 / wall,
                pct(0.5),
                pct(0.99),
                pct(1.0),
                phases.join(", "),
                shared.players.read().len(),
                shared.cubes.load(Ordering::Relaxed),
                shared.entities.load(Ordering::Relaxed),
                cores.join(", "),
            );
        }
    });
}

/// CPU seconds so far by thread name, the digits taken off (`server 3` and `server 7` are one
/// pool); empty where `/proc` isn't.
fn threads_cpu() -> Vec<(String, f64)> {
    let mut out: Vec<(String, f64)> = Vec::new();
    let Ok(dir) = std::fs::read_dir("/proc/self/task") else { return out };
    for task in dir.flatten() {
        let Ok(stat) = std::fs::read_to_string(task.path().join("stat")) else { continue };
        // pid (name) state ... utime stime are the 14th and 15th fields, counted after the name.
        let (Some(open), Some(close)) = (stat.find('('), stat.rfind(')')) else { continue };
        let name: String = stat[open + 1..close]
            .trim_end_matches(|c: char| c.is_ascii_digit() || c == ' ')
            .to_string();
        let fields: Vec<&str> = stat[close + 2..].split(' ').collect();
        let ticks = fields
            .get(11)
            .and_then(|u| u.parse::<f64>().ok())
            .unwrap_or(0.0)
            + fields
                .get(12)
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
        let secs = ticks / 100.0;
        match out.iter_mut().find(|(n, _)| *n == name) {
            Some(e) => e.1 += secs,
            None => out.push((name, secs)),
        }
    }
    out
}

/// Resident memory (MB) and thread count (Linux `/proc/self/status`; 0 elsewhere).
fn process_memory() -> (f64, u64) {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let field = |k: &str| {
        status
            .lines()
            .find(|l| l.starts_with(k))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    (field("VmRSS:") / 1024.0, field("Threads:") as u64)
}
