//! A hang watchdog for test and script runs (`--watchdog <secs>`, off without it).
//!
//! Every client frame beats. A thread of its own checks the beat; when no frame came for the
//! given seconds (a deadlock, a GPU wait that never ends), it writes what it knows, the step the
//! run was on and, on Linux, each thread's state and the kernel function it waits in, to stderr
//! and to `test-reports/watchdog.txt`, then ends the game with exit code 70. A frozen run then
//! says where it stopped instead of going quiet until someone kills it.
//!
//! Off by default: a hidden or minimised window may draw no frames on some systems, which would
//! look like a hang.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// The exit code of a run the watchdog ended.
pub const EXIT_HUNG: i32 = 70;

static EPOCH: OnceLock<Instant> = OnceLock::new();
/// Milliseconds since `EPOCH` at the last frame.
static LAST: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    EPOCH.get_or_init(Instant::now).elapsed().as_millis() as u64
}

/// A frame happened.
pub fn beat() {
    LAST.store(now_ms(), Ordering::Relaxed);
}

/// Starts the watching thread: `limit` seconds without a frame end the run.
pub fn start(limit: f32, report_dir: String) {
    beat();
    let limit_ms = (limit.max(1.0) * 1000.0) as u64;
    weft_sdk::info!("testkit: watchdog on, {limit} s without a frame ends the run with code {EXIT_HUNG}");
    let spawned = std::thread::Builder::new()
        .name("testkit-watchdog".into())
        .spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(500));
                let quiet = now_ms().saturating_sub(LAST.load(Ordering::Relaxed));
                if quiet >= limit_ms {
                    hung(quiet, &report_dir);
                }
            }
        });
    if let Err(e) = spawned {
        weft_sdk::error!("testkit: no watchdog thread: {e}");
    }
}

/// Writes what is known and ends the game. Only std here: the game's own logging may be what is
/// stuck.
fn hung(quiet_ms: u64, report_dir: &str) -> ! {
    // try_lock: the main thread may be stopped while holding it.
    let step = crate::script::STEP
        .try_lock()
        .map(|s| s.clone())
        .unwrap_or_else(|_| "(the step is locked)".into());
    let mut text = format!(
        "testkit: WATCHDOG: no frame for {:.1} s; the run was at: {}\n",
        quiet_ms as f32 / 1000.0,
        if step.is_empty() { "(before the first step)" } else { &step }
    );
    text.push_str(&threads());
    eprintln!("{text}");
    let _ = std::fs::create_dir_all(report_dir);
    let _ = std::fs::write(format!("{report_dir}/watchdog.txt"), &text);
    std::process::exit(EXIT_HUNG);
}

/// Each thread's name, state and the kernel function it waits in (Linux); empty elsewhere.
fn threads() -> String {
    let Ok(dir) = std::fs::read_dir("/proc/self/task") else { return String::new() };
    let mut out = String::from("threads (name, state, waiting in):\n");
    let mut rows: Vec<(u64, String)> = dir
        .flatten()
        .filter_map(|e| {
            let tid: u64 = e.file_name().to_str()?.parse().ok()?;
            let p = e.path();
            let read = |f: &str| {
                std::fs::read_to_string(p.join(f))
                    .unwrap_or_default()
                    .trim()
                    .to_string()
            };
            let stat = read("stat");
            // `pid (comm) S ...`: the state follows the last ')'.
            let state = stat
                .rsplit_once(')')
                .and_then(|(_, r)| r.split_whitespace().next())
                .unwrap_or("?")
                .to_string();
            Some((tid, format!("  {tid} {} {state} {}\n", read("comm"), read("wchan"))))
        })
        .collect();
    rows.sort();
    for (_, r) in rows {
        out.push_str(&r);
    }
    out
}
