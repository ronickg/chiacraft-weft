//! A line script language on the client mod API:
//!
//! ```text
//! wait 2.5            # seconds
//! loaded              # until the world around the player is meshed
//! idle                # until nothing is left to generate, light, send or mesh
//! press core:inventory
//! hold core:forward 1.5  # held 1.5 s, the next line after it; `hold core:forward 1.5 nowait` goes on at once
//! click 0             # mouse button
//! cursor 0.5 0.4      # mouse at fractions of the window
//! scroll -3
//! type /give dia      # text into the focused field
//! key Tab             # a key by its settings name
//! look 90 -20         # yaw, pitch in degrees: a negative pitch looks DOWN, a positive one up
//! chat /give stone 10
//! chatf ticket.txt /banlist follow {}   # chat, with `{}` replaced by that file's text (waits for it)
//! say_config testmod:x  # a config's value as this client has it, into the chat
//! shot name.png       # into run/screenshots (or --shot-dir)
//! bench 5 label       # BENCH line over 5 seconds
//! host ticket.txt     # host, write the join ticket there
//! join ticket.txt     # join with the ticket from that file (waits for it)
//! open name           # leave this world and open (or make) the saved world called name
//! leave               # leave the world for the title screen
//! tap screen query    # once `screen` is open, clicks the element `query` (`id:x`, `text:x`) until it closes or the element goes
//! tap_once screen query  # the same, one click (a text field, a button that stays)
//! screen core:multiplayer  # opens a screen by name (an engine screen or a mod's)
//! mark path           # writes the file (a signal to another copy's await_file)
//! await_file path 120 # until the file exists, at most 120 s
//! origin              # later via points are relative to where the player stands now
//! where label        # logs the player's position
//! entities 16        # logs every entity within 16 blocks as the server and this client have it
//! rec start|stop     # logs a marker; tools/rec.ps1 presses the OBS hotkeys on it
//! via x y z yaw pitch # a camera waypoint (eye position), or: via x y z @ tx ty tz (look at a point)
//! travel 8            # fly through the waypoints in 8 s: spline, eased, a little hand shake
//! glide yaw pitch 3   # turn smoothly in place over 3 s
//! expect <condition>  # checks it now; a miss fails the run (with its line) and the script goes on
//! until <condition> [within 30]  # waits for it, seconds; not in time stops the run
//! quit
//! ```
//!
//! Conditions (`~` is the player's feet, `~3` three blocks off them):
//!
//! ```text
//! block x y z name    # `stone`, `base:stone` or `base:furnace[lit=true]`
//! near x y z [r]      # the player within r blocks (1.5)
//! on_ground | alive | dead | flying
//! mode survival|creative|spectator
//! health < 8          # the engine's health points; < <= > >= = !=
//! fps > 30
//! chat text           # a chat line on screen contains it (any case)
//! screen name         # the top screen (`none` in the world)
//! players 2           # at least 2 players in the world (the host's view)
//! ```
//!
//! `/tk time <ticks>` sets the world's whole clock (day 0 + ticks), so it also undoes a `/season`
//! before it: set the time first, then the season (or use `/time set`, which keeps the day).
//!
//! Before the first step the whole script is read: an unknown command or a condition that can't
//! be read fails the run at once, with every bad line, instead of minutes in.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use weft_sdk::client::client_flags as cf;
use weft_sdk::launch_option;

use crate::runner::{R, T};

/// The step being run (`line N: text`), for the watchdog to name when the game stops.
pub static STEP: Mutex<String> = Mutex::new(String::new());

/// Every command `run_lines` knows; a new one goes here too (the test `every_command_is_listed`
/// reads this file). Some come from branches not merged yet (`await_ui` to `await_file` from
/// Phase 3's portal checks, `chatf` from Phase 4's), so the check doesn't reject their scripts.
const COMMANDS: &[&str] = &[
    "wait",
    "loaded",
    "idle",
    "press",
    "hold",
    "click",
    "cursor",
    "scroll",
    "type",
    "key",
    "look",
    "chat",
    "say_config",
    "set_config",
    "shot",
    "bench",
    "host",
    "join",
    "open",
    "leave",
    "tap",
    "tap_once",
    "origin",
    "where",
    "entities",
    "rec",
    "via",
    "travel",
    "glide",
    "expect",
    "until",
    "quit",
    "await_ui",
    "screen",
    "type_file",
    "close",
    "mark",
    "await_file",
    "chatf",
];
/// Commands that mean nothing without an argument.
const NEED_ARGS: &[&str] = &[
    "wait",
    "press",
    "hold",
    "type",
    "key",
    "look",
    "chat",
    "say_config",
    "set_config",
    "shot",
    "host",
    "join",
    "open",
    "tap",
    "tap_once",
    "via",
    "expect",
    "until",
];

/// Every problem the script has before it runs: unknown commands, missing arguments, conditions
/// that can't be read.
pub fn preflight(src: &str) -> Vec<String> {
    let mut bad = Vec::new();
    for (n, raw) in src.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let (cmd, rest) = line.split_once(' ').unwrap_or((line, ""));
        let rest = rest.trim();
        if !COMMANDS.contains(&cmd) {
            bad.push(format!("line {}: unknown command '{cmd}'", n + 1));
        } else if NEED_ARGS.contains(&cmd) && rest.is_empty() {
            bad.push(format!("line {}: '{cmd}' needs an argument", n + 1));
        } else if cmd == "expect" || cmd == "until" {
            let (cond, _) = split_within(rest);
            if let Err(e) = Cond::parse(cond) {
                bad.push(format!("line {}: {e}", n + 1));
            }
        }
    }
    bad
}

/// Lines that run but probably don't do what was meant: a click or use while the last `look`
/// pointed up (a positive pitch is up, so a script aiming at the ground clicks the sky).
pub fn lint(src: &str) -> Vec<String> {
    let mut warn = Vec::new();
    let mut up: Option<(usize, f32)> = None;
    for (n, raw) in src.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        let (cmd, rest) = line.split_once(' ').unwrap_or((line, ""));
        match cmd {
            "look" => {
                let pitch = rest
                    .split_whitespace()
                    .nth(1)
                    .and_then(|p| p.parse::<f32>().ok())
                    .unwrap_or(0.0);
                up = (pitch > 0.0).then_some((n + 1, pitch));
            }
            "via" | "travel" | "glide" => up = None,
            "click" | "press" | "hold" => {
                let aims = cmd == "click" || rest.contains("core:attack") || rest.contains("core:use");
                if let (true, Some((at, pitch))) = (aims, up) {
                    warn.push(format!(
                        "line {}: '{line}' while looking UP (pitch +{pitch} at line {at}); a negative pitch looks down",
                        n + 1
                    ));
                }
            }
            _ => {}
        }
    }
    warn
}

/// `cond within 12` → (`cond`, 12 s); 30 s without it.
fn split_within(rest: &str) -> (&str, f32) {
    match rest.rsplit_once(" within ") {
        Some((c, s)) if s.trim().parse::<f32>().is_ok() => (c.trim(), s.trim().parse().unwrap_or(30.0)),
        _ => (rest, 30.0),
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Cmp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl Cmp {
    fn parse(s: &str) -> Option<Cmp> {
        Some(match s {
            "<" => Cmp::Lt,
            "<=" => Cmp::Le,
            ">" => Cmp::Gt,
            ">=" => Cmp::Ge,
            "=" | "==" => Cmp::Eq,
            "!=" => Cmp::Ne,
            _ => return None,
        })
    }
    fn holds(self, a: f32, b: f32) -> bool {
        match self {
            Cmp::Lt => a < b,
            Cmp::Le => a <= b,
            Cmp::Gt => a > b,
            Cmp::Ge => a >= b,
            Cmp::Eq => (a - b).abs() < 1e-3,
            Cmp::Ne => (a - b).abs() >= 1e-3,
        }
    }
}

/// A coordinate: a number, or `~` / `~n` off the player's feet.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Coord {
    At(f64),
    Off(f64),
}

impl Coord {
    fn parse(s: &str) -> Option<Coord> {
        match s.strip_prefix('~') {
            Some("") => Some(Coord::Off(0.0)),
            Some(n) => n.parse().ok().map(Coord::Off),
            None => s.parse().ok().map(Coord::At),
        }
    }
    fn at(self, feet: f64) -> f64 {
        match self {
            Coord::At(v) => v,
            Coord::Off(d) => feet + d,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
enum Cond {
    Block([Coord; 3], String),
    Near([Coord; 3], f64),
    Flag(&'static str, u32, bool),
    Mode(u32),
    Health(Cmp, f32),
    Fps(Cmp, f32),
    Chat(String),
    Screen(String),
    Players(u32),
}

impl Cond {
    fn parse(text: &str) -> Result<Cond, String> {
        let w: Vec<&str> = text.split_whitespace().collect();
        let what = w.first().copied().unwrap_or("");
        let coords = |from: usize| -> Option<[Coord; 3]> {
            Some([Coord::parse(w.get(from)?)?, Coord::parse(w.get(from + 1)?)?, Coord::parse(w.get(from + 2)?)?])
        };
        let cmp = || -> Option<(Cmp, f32)> { Some((Cmp::parse(w.get(1)?)?, w.get(2)?.parse().ok()?)) };
        let bad = |form: &str| format!("condition '{text}': want {form}");
        Ok(match what {
            "block" => match (coords(1), w.get(4)) {
                (Some(p), Some(name)) => Cond::Block(p, name.to_string()),
                _ => return Err(bad("block x y z name")),
            },
            "near" => match coords(1) {
                Some(p) => Cond::Near(p, w.get(4).and_then(|r| r.parse().ok()).unwrap_or(1.5)),
                None => return Err(bad("near x y z [r]")),
            },
            "on_ground" => Cond::Flag("on the ground", cf::ON_GROUND, true),
            "flying" => Cond::Flag("flying", cf::FLYING, true),
            "dead" => Cond::Flag("dead", cf::DEAD, true),
            "alive" => Cond::Flag("alive", cf::DEAD, false),
            "mode" => match w.get(1).copied() {
                Some("survival") => Cond::Mode(0),
                Some("creative") => Cond::Mode(1),
                Some("spectator") => Cond::Mode(2),
                _ => return Err(bad("mode survival|creative|spectator")),
            },
            "health" => cmp()
                .map(|(c, v)| Cond::Health(c, v))
                .ok_or_else(|| bad("health <op> n"))?,
            "fps" => cmp()
                .map(|(c, v)| Cond::Fps(c, v))
                .ok_or_else(|| bad("fps <op> n"))?,
            "chat" if w.len() > 1 => Cond::Chat(text["chat".len()..].trim().to_lowercase()),
            "screen" if w.len() == 2 => Cond::Screen(w[1].to_string()),
            "players" => match w.get(1).and_then(|n| n.parse().ok()) {
                Some(n) => Cond::Players(n),
                None => return Err(bad("players n")),
            },
            _ => return Err(format!("unknown condition '{text}'")),
        })
    }

    /// Whether it holds now, and what was there instead.
    async fn check(&self, t: &T) -> Result<(bool, String), String> {
        let s = t.state();
        let feet = [s.pos[0].floor(), s.pos[1].floor(), s.pos[2].floor()];
        Ok(match self {
            Cond::Block(p, want) => {
                let at = [p[0].at(feet[0]) as i32, p[1].at(feet[1]) as i32, p[2].at(feet[2]) as i32];
                let got = t.block(at).await?;
                (block_matches(&got, want), format!("{got} at {} {} {}", at[0], at[1], at[2]))
            }
            Cond::Near(p, r) => {
                let at = [p[0].at(s.pos[0]), p[1].at(s.pos[1]), p[2].at(s.pos[2])];
                let d = ((s.pos[0] - at[0]).powi(2) + (s.pos[1] - at[1]).powi(2) + (s.pos[2] - at[2]).powi(2)).sqrt();
                (d <= *r, format!("at {:.2} {:.2} {:.2}, {d:.2} away", s.pos[0], s.pos[1], s.pos[2]))
            }
            Cond::Flag(_, f, want) => (t.flag(*f) == *want, format!("flags {:#x}", s.flags)),
            Cond::Mode(m) => (s.mode == *m, format!("mode {}", s.mode)),
            Cond::Health(c, v) => (c.holds(s.health, *v), format!("health {}", s.health)),
            Cond::Fps(c, v) => (c.holds(s.fps, *v), format!("fps {:.1}", s.fps)),
            Cond::Chat(want) => {
                let mut last = String::new();
                for i in 0..100 {
                    let line = t.c().chat_line(i);
                    if line.is_empty() {
                        break;
                    }
                    if line.to_lowercase().contains(want.as_str()) {
                        return Ok((true, line));
                    }
                    if i == 0 {
                        last = line;
                    }
                }
                (false, format!("last chat line '{last}'"))
            }
            Cond::Screen(name) => {
                let now = t.c().screen();
                let shown = if now.is_empty() { "none".to_string() } else { now };
                (shown == *name, format!("screen {shown}"))
            }
            Cond::Players(n) => {
                let got: u32 = t.op("players").await?.trim().parse().unwrap_or(0);
                (got >= *n, format!("{got} players"))
            }
        })
    }
}

/// `want` names the block the server reports (`base:furnace[facing=north,lit=true]`): the whole
/// name, the name without its namespace, or the name with some of its properties.
fn block_matches(got: &str, want: &str) -> bool {
    let (g_name, g_props) = got.split_once('[').unwrap_or((got, ""));
    let (w_name, w_props) = want.split_once('[').unwrap_or((want, ""));
    let name_ok = g_name == w_name || (!w_name.contains(':') && g_name.rsplit(':').next() == Some(w_name));
    let have: Vec<&str> = g_props.trim_end_matches(']').split(',').collect();
    name_ok
        && w_props
            .trim_end_matches(']')
            .split(',')
            .filter(|p| !p.is_empty())
            .all(|p| have.contains(&p))
}

pub async fn run(t: T, src: String) -> R {
    // Warnings first: they matter most when the script also has errors to fix.
    for w in lint(&src) {
        weft_sdk::warn!("script: {w}");
    }
    let bad = preflight(&src);
    if !bad.is_empty() {
        return Err(format!("the script has {} bad line(s): {}", bad.len(), bad.join("; ")));
    }
    // A step that stops the run names its line.
    run_lines(t, src)
        .await
        .map_err(|e| format!("{}: {e}", STEP.lock().map(|s| s.clone()).unwrap_or_default()))
}

async fn run_lines(t: T, src: String) -> R {
    let started = Instant::now();
    let mut via: Vec<[f64; 5]> = Vec::new();
    let mut origin = [0.0f64; 3];
    for (n, line) in src.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(mut s) = STEP.lock() {
            *s = format!("line {} '{line}'", n + 1);
        }
        let (cmd, rest) = line.split_once(' ').unwrap_or((line, ""));
        let rest = rest.trim();
        let nums: Vec<f32> = rest
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        let num = |i: usize, d: f32| nums.get(i).copied().unwrap_or(d);
        match cmd {
            "wait" => t.secs(num(0, 1.0)).await?,
            "loaded" => {
                t.loaded().await?;
                weft_sdk::info!("script: loaded at {:.2} s", started.elapsed().as_secs_f32());
            }
            "idle" => {
                t.idle().await?;
                let s = t.state();
                weft_sdk::info!("script: idle at {:.2} s", started.elapsed().as_secs_f32());
                weft_sdk::info!(
                    "load: {} cubes on client ({} of them made on the GPU, {} differing from the CPU), server generated {} ({:.0} us each on a worker), inserted {} ({:.0} us each on the server thread)",
                    s.cubes,
                    s.gpu_cubes,
                    s.gpu_mismatches,
                    s.generated,
                    s.gen_us,
                    s.inserted,
                    s.insert_us
                );
            }
            "press" => t.hold(rest, 0.0),
            // Holds for its seconds before the next line, unless `nowait` (then the next lines run
            // while it is held: steering while walking).
            "hold" => {
                let mut w = rest.split_whitespace();
                let action = w.next().unwrap_or("");
                let secs: f32 = w.next().and_then(|s| s.parse().ok()).unwrap_or(1.0);
                let nowait = w.next() == Some("nowait");
                t.hold(action, secs);
                if !nowait {
                    t.secs(secs).await?;
                }
            }
            "click" => t.c().click(num(0, 0.0) as u32, 0.0),
            "cursor" => t.c().cursor(num(0, 0.5), num(1, 0.5)),
            "scroll" => t.c().scroll(num(0, -1.0)),
            "type" => t.c().type_text(rest),
            "key" => {
                if !t.c().key(rest, 0.0) {
                    t.fail(format!("unknown key {rest}"));
                }
            }
            "look" => t.c().look(num(0, 0.0), num(1, 0.0)),
            "chat" => t.c().chat(rest),
            "chatf" => {
                let (file, text) = rest.split_once(' ').unwrap_or((rest, ""));
                let mut content = String::new();
                t.until("chat file", Duration::from_secs(120), |_| {
                    content = std::fs::read_to_string(file)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    !content.is_empty()
                })
                .await?;
                t.c().chat(&text.replace("{}", &content));
            }
            "say_config" => {
                let cfg = weft_sdk::Config::<f64>::named(Box::leak(rest.to_string().into_boxed_str()));
                t.c().chat(&format!("config {rest} = {}", cfg.get()));
            }
            // `set_config base:fx_reflect 0`: changes a setting mid-run, so one run benches each
            // effect on and off over the same loaded world (Research round 16).
            "set_config" => {
                let (name, value) = rest.split_once(' ').unwrap_or((rest, "0"));
                let cfg = weft_sdk::Config::<f64>::named(Box::leak(name.trim().to_string().into_boxed_str()));
                cfg.set(value.trim().parse().unwrap_or(0.0));
                weft_sdk::info!("set_config {} = {}", name.trim(), cfg.get());
            }
            "shot" => {
                let path = match launch_option("shot-dir") {
                    Some(d) => format!("{d}/{rest}"),
                    None => rest.to_string(),
                };
                t.c().screenshot(&path);
            }
            "bench" => {
                let (secs, label) = rest.split_once(' ').unwrap_or((rest, ""));
                t.bench(secs.parse().unwrap_or(5.0), label.trim()).await?;
            }
            "host" => {
                t.c().host_world();
                t.until("hosting", Duration::from_secs(60), |t| t.flag(cf::HOSTING))
                    .await?;
                let _ = std::fs::write(rest, t.c().ticket());
            }
            "open" => {
                t.c().open_world(rest, 1);
                // The old world's client is gone by the next frames; then wait for the new one.
                t.frames(3).await?;
                t.until("the world open", Duration::from_secs(60), |t| t.flag(cf::JOINED))
                    .await?;
                t.loaded().await?;
            }
            "tap" => {
                // Clicked again while a rebuilding screen may have lost the click; done once the
                // screen changes or the element is gone (a button that did its job).
                let (screen, query) = rest.split_once(' ').ok_or("tap: a screen and an element")?;
                t.screen(screen).await?;
                if !t.click_ui(query, 0) {
                    continue;
                }
                for _ in 0..9 {
                    t.frames(15).await?;
                    if t.c().screen() != screen || !t.c().click_ui(query, 0) {
                        break;
                    }
                }
            }
            // One click, for what stays on screen and must happen once (a text field, "Ask").
            "tap_once" => {
                // Only the screen's name: a focused text field keeps a screen from ever being
                // "ready" (its caret redraws it).
                let (screen, query) = rest
                    .split_once(' ')
                    .ok_or("tap_once: a screen and an element")?;
                let name = screen.to_string();
                t.until(&format!("screen {screen}"), Duration::from_secs(60), move |t| t.c().screen() == name)
                    .await?;
                t.frames(15).await?;
                t.click_ui(query, 0);
                t.frames(15).await?;
            }
            "screen" => {
                let screen = weft_sdk::Screen::generated(rest.to_string());
                if !t.c().open_screen(screen) {
                    t.fail(format!("no screen {rest}"));
                }
                t.screen(rest).await?;
            }
            "mark" => {
                if let Err(e) = std::fs::write(rest, "1") {
                    t.fail(format!("mark {rest}: {e}"));
                }
            }
            "await_file" => {
                let (path, secs) = rest.rsplit_once(' ').unwrap_or((rest, "120"));
                let path = path.to_string();
                let secs = secs.parse().unwrap_or(120.0);
                t.until(&format!("file {path}"), Duration::from_secs_f32(secs), move |_| std::path::Path::new(&path).exists())
                    .await?;
            }
            "leave" => {
                t.c().leave_world();
                t.screen("core:title").await?;
            }
            "join" => {
                let mut ticket = String::new();
                t.until("ticket file", Duration::from_secs(120), |_| {
                    ticket = std::fs::read_to_string(rest)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    !ticket.is_empty()
                })
                .await?;
                t.c().join(&ticket);
            }
            "origin" => origin = t.state().pos,
            "where" => {
                let s = t.state();
                weft_sdk::info!("script: where {:.3} {:.3} {:.3} {rest}", s.pos[0], s.pos[1], s.pos[2]);
            }
            "entities" => entities(&t, num(0, 16.0)).await?,
            "rec" => weft_sdk::info!("REC {rest}"),
            "via" => match waypoint(origin, rest) {
                Some(mut w) => {
                    for k in 0..3 {
                        w[k] += origin[k];
                    }
                    via.push(w)
                }
                None => t.fail(format!("bad waypoint '{line}'")),
            },
            "travel" => {
                let eye = t.c().entity(0).map_or(1.62, |e| e.size[2] as f64);
                let s = t.state();
                let mut pts = vec![[s.pos[0], s.pos[1] + eye, s.pos[2], (s.yaw as f64).to_degrees(), (s.pitch as f64).to_degrees()]];
                pts.append(&mut via);
                fly(&t, pts, num(0, 5.0) as f64, eye).await?;
            }
            "glide" => {
                let s = t.state();
                let from = [s.pos[0], s.pos[1] + 0.0, s.pos[2]];
                let eye = t.c().entity(0).map_or(1.62, |e| e.size[2] as f64);
                let a = [from[0], from[1] + eye, from[2], (s.yaw as f64).to_degrees(), (s.pitch as f64).to_degrees()];
                let b = [a[0], a[1], a[2], num(0, 0.0) as f64, num(1, 0.0) as f64];
                fly(&t, vec![a, b], num(2, 2.0) as f64, eye).await?;
            }
            "expect" => {
                let cond = Cond::parse(rest)?;
                let (ok, seen) = cond.check(&t).await?;
                if ok {
                    weft_sdk::info!("script: line {} expect {rest}: ok ({seen})", n + 1);
                } else {
                    t.fail(format!("line {} expect {rest}: {seen}", n + 1));
                }
            }
            "until" => {
                let (text, secs) = split_within(rest);
                let cond = Cond::parse(text)?;
                let waited = Instant::now();
                let deadline = waited + Duration::from_secs_f32(secs);
                loop {
                    let (ok, seen) = cond.check(&t).await?;
                    if ok {
                        weft_sdk::info!(
                            "script: line {} until {text}: met after {:.1} s ({:.1} s into the script)",
                            n + 1,
                            waited.elapsed().as_secs_f32(),
                            started.elapsed().as_secs_f32()
                        );
                        break;
                    }
                    if Instant::now() > deadline {
                        return Err(format!("until {text}: not within {secs} s ({seen})"));
                    }
                    t.secs(0.25).await?;
                }
            }
            "quit" => return Ok(()),
            _ => t.fail(format!("unknown script line '{line}'")),
        }
        // One step per frame.
        t.frames(1).await?;
    }
    Ok(())
}

/// Every entity within `r` blocks, one line each: where the server has it and where this client
/// does (or that it doesn't), the client's block under its feet and light there. For "the server
/// has it, the client draws nothing".
async fn entities(t: &T, r: f32) -> R {
    let me = t.state().pos;
    let near = t
        .op(&format!("near {:.0} {:.0} {:.0} {r}", me[0].floor(), me[1].floor(), me[2].floor()))
        .await?;
    let mut ids: Vec<u64> = near
        .split_whitespace()
        .filter_map(|p| p.split(':').next()?.parse().ok())
        .collect();
    let c = t.c();
    let mine = c.entities();
    let close = |p: [f64; 3]| (0..3).map(|i| (p[i] - me[i]).powi(2)).sum::<f64>() <= (r as f64).powi(2);
    let only_mine: Vec<u64> = mine
        .iter()
        .copied()
        .filter(|id| !ids.contains(id) && c.entity(*id).is_some_and(|e| close(e.pos)))
        .collect();
    ids.extend(only_mine);
    weft_sdk::info!(
        "entities: me at {:.2} {:.2} {:.2}, server {} within {r}, client knows {} in all",
        me[0],
        me[1],
        me[2],
        near.split_whitespace().count(),
        mine.len()
    );
    for id in ids {
        let server = t
            .op(&format!("entity {id}"))
            .await
            .unwrap_or_else(|e| format!("none ({e})"));
        // Under it as the server has it.
        let under = match server
            .split_whitespace()
            .skip(1)
            .take(3)
            .map(|v| v.parse::<f64>())
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(p) if p.len() == 3 => {
                let at = format!("{:.0} {:.0} {:.0}", p[0].floor(), p[1].floor() - 1.0, p[2].floor());
                let block = t.op(&format!("get {at}")).await.unwrap_or_default();
                let light = t
                    .op(&format!("light {:.0} {:.0} {:.0}", p[0].floor(), p[1].floor(), p[2].floor()))
                    .await
                    .unwrap_or_default();
                format!(", under {block}, light {light}")
            }
            _ => String::new(),
        };
        let client = match c.entity(id) {
            Some(e) => {
                let feet = [e.pos[0].floor() as i32, e.pos[1].floor() as i32, e.pos[2].floor() as i32];
                let below = [feet[0], feet[1] - 1, feet[2]];
                format!(
                    "at {:.2} {:.2} {:.2}, health {}, size {:?}, block under {:?}, light {:?}, cube shown {}",
                    e.pos[0],
                    e.pos[1],
                    e.pos[2],
                    e.health,
                    e.size,
                    c.block(below).map(weft_sdk::block_name),
                    c.light(feet),
                    c.cube_shown(feet)
                )
            }
            None => "MISSING".into(),
        };
        weft_sdk::info!("entities: {id} server [{server}{under}] client [{client}]");
    }
    Ok(())
}

fn waypoint(origin: [f64; 3], rest: &str) -> Option<[f64; 5]> {
    let (head, at) = match rest.split_once('@') {
        Some((h, a)) => (h, Some(a)),
        None => (rest, None),
    };
    let n: Vec<f64> = head
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    match at {
        Some(a) => {
            let m: Vec<f64> = a
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if n.len() < 3 || m.len() < 3 {
                return None;
            }
            let d = [m[0] - n[0], m[1] - n[1], m[2] - n[2]];
            let _ = origin;
            let yaw = (-d[0]).atan2(-d[2]).to_degrees();
            let pitch = d[1].atan2((d[0] * d[0] + d[2] * d[2]).sqrt()).to_degrees();
            Some([n[0], n[1], n[2], yaw, pitch])
        }
        None if n.len() >= 5 => Some([n[0], n[1], n[2], n[3], n[4]]),
        None => None,
    }
}

fn catmull(p: [&[f64; 5]; 4], u: f64) -> [f64; 5] {
    let mut out = [0.0; 5];
    for i in 0..5 {
        let (a, b, c, d) = (p[0][i], p[1][i], p[2][i], p[3][i]);
        out[i] = 0.5 * (2.0 * b + (-a + c) * u + (2.0 * a - 5.0 * b + 4.0 * c - d) * u * u + (-a + 3.0 * b - 3.0 * c + d) * u * u * u);
    }
    out
}

/// Flies the camera through `pts` (eye position, yaw and pitch in degrees) in `secs`: constant
/// speed along a spline, eased at both ends, with a slow uneven drift on the aim like a hand.
async fn fly(t: &T, mut pts: Vec<[f64; 5]>, secs: f64, eye: f64) -> R {
    // Yaw takes the short way round between neighbours.
    for i in 1..pts.len() {
        let mut d = (pts[i][3] - pts[i - 1][3]) % 360.0;
        if d > 180.0 {
            d -= 360.0;
        } else if d < -180.0 {
            d += 360.0;
        }
        pts[i][3] = pts[i - 1][3] + d;
    }
    let n = pts.len();
    let at = |i: isize| &pts[i.clamp(0, n as isize - 1) as usize];
    let mut dense: Vec<([f64; 5], f64)> = Vec::new();
    let mut len = 0.0;
    for seg in 0..n.saturating_sub(1) {
        for k in 0..64 {
            let p = catmull([at(seg as isize - 1), at(seg as isize), at(seg as isize + 1), at(seg as isize + 2)], k as f64 / 64.0);
            if let Some((q, _)) = dense.last() {
                len += ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
            }
            dense.push((p, len));
        }
    }
    dense.push((pts[n - 1], len + 1e-9));
    // Pure turns have no length: spread them over the samples instead.
    let turn = len < 1e-3;
    let last = dense.len() - 1;
    let mut elapsed = 0.0;
    while elapsed < secs {
        t.frames(1).await?;
        elapsed += (t.state().dt as f64).min(0.1);
        let u = (elapsed / secs).min(1.0);
        let e = u * u * u * (u * (u * 6.0 - 15.0) + 10.0);
        let e = (e + 0.012 * (std::f64::consts::PI * u).sin() * (u * secs * 1.7).sin()).clamp(0.0, 1.0);
        let i = if turn {
            ((e * last as f64) as usize).min(last)
        } else {
            let want = e * len;
            dense.partition_point(|(_, l)| *l < want).min(last)
        };
        let (a, la) = dense[i.saturating_sub(1)];
        let (b, lb) = dense[i];
        let f = if turn {
            0.0
        } else if lb > la {
            ((e * len - la) / (lb - la)).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let mut p = [0.0; 5];
        for k in 0..5 {
            p[k] = a[k] + (b[k] - a[k]) * f;
        }
        let tm = elapsed;
        let shake = (u * std::f64::consts::PI).sin().min(1.0);
        p[3] += shake * (0.45 * (tm * 0.9).sin() + 0.25 * (tm * 2.3 + 1.0).sin());
        p[4] += shake * (0.35 * (tm * 0.7 + 2.0).sin() + 0.2 * (tm * 1.9).sin());
        let c = t.c();
        c.control(&weft_sdk::abi::PlayerControl {
            mask: weft_sdk::abi::control_mask::POS | weft_sdk::abi::control_mask::VEL | weft_sdk::abi::control_mask::LOOK,
            pos: [p[0], p[1] - eye, p[2]],
            vel: [0.0; 3],
            yaw: p[3].to_radians() as f32,
            pitch: p[4].to_radians() as f32,
            ..weft_sdk::abi::PlayerControl::DEFAULT
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_command_is_listed() {
        let src = include_str!("script.rs");
        let body = src.split("async fn run_lines").nth(1).unwrap();
        for line in body.lines() {
            let l = line.trim();
            if let Some(name) = l
                .strip_prefix('"')
                .and_then(|r| r.split_once("\" =>"))
                .map(|(n, _)| n)
            {
                assert!(COMMANDS.contains(&name), "script command '{name}' is missing from COMMANDS");
            }
        }
    }

    #[test]
    fn preflight_finds_bad_lines() {
        let bad = preflight("loaded\nwiat 2\nexpect block 1 2\nuntil on_ground within 5\nshot\n# note\nexpect chat hello\n");
        assert_eq!(bad.len(), 3, "{bad:?}");
        assert!(bad[0].starts_with("line 2:"));
        assert!(bad[1].starts_with("line 3:"));
        assert!(bad[2].starts_with("line 5:"));
    }

    #[test]
    fn conditions_read() {
        assert_eq!(
            Cond::parse("block ~ ~-1 ~2 stone"),
            Ok(Cond::Block([Coord::Off(0.0), Coord::Off(-1.0), Coord::Off(2.0)], "stone".into()))
        );
        assert_eq!(Cond::parse("health <= 8"), Ok(Cond::Health(Cmp::Le, 8.0)));
        assert_eq!(Cond::parse("chat You Joined"), Ok(Cond::Chat("you joined".into())));
        assert!(Cond::parse("health about 8").is_err());
        assert_eq!(split_within("players 2 within 90"), ("players 2", 90.0));
        assert_eq!(split_within("on_ground"), ("on_ground", 30.0));
    }

    #[test]
    fn lint_warns_on_clicking_upwards() {
        let w = lint("look 0 85\nclick 0\nlook 0 -85\npress core:attack\nlook 10 20\nhold core:use 1\nhold core:forward 2\n");
        assert_eq!(w.len(), 2, "{w:?}");
        assert!(w[0].starts_with("line 2:"));
        assert!(w[1].starts_with("line 6:"));
    }

    #[test]
    fn block_names_match() {
        let got = "base:furnace[facing=north,lit=true]";
        assert!(block_matches(got, "base:furnace"));
        assert!(block_matches(got, "furnace"));
        assert!(block_matches(got, "furnace[lit=true]"));
        assert!(!block_matches(got, "furnace[lit=false]"));
        assert!(!block_matches(got, "base:stone"));
        assert!(!block_matches("animals:furnace", "base:furnace"));
    }
}
