//! The wallet rule "no secret in logs" over the sponsor, the one service holding a key of ours:
//!
//! 1. No log, print or panic macro in `src/` takes the funding words or a key, or a spend's
//!    private parts (a bundle before it is pushed, coin spends, solutions, signatures), or the
//!    endpoint key's bytes; a wallet key is held only inside `chiacraft_wallet::Secret` (no
//!    `SecretKey` field). The endpoint's key, iroh's, is named in full (`iroh::SecretKey`) and
//!    lives only in the endpoint and `sponsor.key`.
//! 2. At run time: a sponsor started from known throwaway words, served over iroh on loopback,
//!    launches DIDs, and neither the words, their entropy nor the endpoint's key appear in
//!    anything it logged, in its request log or in its chain cache.
mod common;

use chiacraft_wallet::{Mnemonic, chain::sim::Sim};
use common::*;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MACROS: &[&str] = &[
    "trace!",
    "debug!",
    "info!",
    "warn!",
    "error!",
    "log!",
    "println!",
    "eprintln!",
    "print!",
    "eprint!",
    "panic!",
    "unreachable!",
    "todo!",
    "unimplemented!",
];

/// Names that mean a secret or a spend's private parts is in hand.
const WORDS: &[&str] = &[
    "words",
    "mnemonic",
    "passphrase",
    "entropy",
    "seed",
    "secret",
    "keys",
    "sk",
    "synthetic",
    "coin_spends",
    "spends",
    "solution",
    "signature",
    "built",
];
const CALLS: &[&str] = &[".expose(", ".expose_mut(", ".words()", ".entropy()", ".seed()", ".to_bytes()"];

fn sources() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    out.sort();
    out
}

/// Each macro call's argument text, from `name!(` to its closing bracket.
fn macro_args<'a>(text: &'a str, name: &str) -> Vec<(usize, &'a str)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = text[from..].find(name) {
        let at = from + i;
        from = at + name.len();
        if text[..at]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            continue;
        }
        let rest = &text[from..];
        let Some(open) = rest.find(['(', '[', '{']) else {
            continue;
        };
        let mut depth = 0;
        let mut end = rest.len();
        for (j, c) in rest[open..].char_indices() {
            match c {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = open + j;
                        break;
                    }
                }
                _ => {}
            }
        }
        let line = text[..at].lines().count();
        out.push((line, &rest[open + 1..end]));
    }
    out
}

/// A secret word (as a whole word, in code or a format argument) or call in `args`.
fn leaks(args: &str) -> Option<String> {
    let lower = args.to_ascii_lowercase();
    let word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
    for w in WORDS {
        let mut from = 0;
        while let Some(i) = lower[from..].find(w) {
            let at = from + i;
            from = at + w.len();
            if !word(lower[..at].chars().next_back()) && !word(lower[from..].chars().next()) {
                return Some((*w).to_owned());
            }
        }
    }
    CALLS
        .iter()
        .find(|c| args.contains(*c))
        .map(|c| (*c).to_owned())
}

#[test]
fn no_log_print_or_panic_takes_a_secret() {
    let files = sources();
    assert!(files.len() >= 5, "found the sources ({})", files.len());
    let mut found = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        for name in MACROS {
            for (line, args) in macro_args(&text, name) {
                if let Some(what) = leaks(args) {
                    found.push(format!("{}:{line}: {name} with {what}", path.display()));
                }
            }
        }
        for (i, l) in text.lines().enumerate() {
            let t = l.trim_start();
            if !t.starts_with("//") && (t.contains(": SecretKey") || t.contains("<SecretKey")) {
                found.push(format!("{}:{}: a SecretKey outside Secret", path.display(), i + 1));
            }
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn the_scanner_finds_a_leak() {
    let text = r#"fn f() { log::info!("funding {}", words.join(" ")); warn!("ok {n}"); }"#;
    let found: Vec<_> = macro_args(text, "info!")
        .into_iter()
        .filter_map(|(_, a)| leaks(a))
        .collect();
    assert_eq!(found, ["words"]);
    assert!(leaks("\"pushed {bundle:?}\", m.expose()").is_some());
    assert!(leaks("\"{} confirmed at height {height}\", did").is_none());
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Everything logged while the runtime test runs.
static LOGGED: Mutex<String> = Mutex::new(String::new());
struct Capture;
impl log::Log for Capture {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, r: &log::Record) {
        let mut l = LOGGED.lock().unwrap();
        l.push_str(&format!("{} {}\n", r.level(), r.args()));
    }
    fn flush(&self) {}
}

#[test]
fn a_running_sponsor_logs_and_keeps_no_secret() {
    log::set_logger(&Capture).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
    let sim = Sim::start().unwrap();
    let dir = scratch("leaks");
    let words = Mnemonic::generate().unwrap();
    let phrase = words.words().expose().join(" ");
    let entropy = hex(words.entropy().expose());
    let endpoint_key = {
        let served = Served::start(sponsor(&sim, config(&dir), &words, 1_000_000));
        let ep = served.asker();
        for _ in 0..3 {
            let (status, body) = served.ask(&ep, &Player::new().request());
            assert_eq!(status, 200, "{body}");
        }
        // A refusal, and a request that isn't one, are logged too, if at all.
        assert_eq!(served.ask(&ep, &Player::new().request_at(1)).0, 400);
        assert_eq!(served.send(&ep, b"nonsense").unwrap().0, 400);
        served.sponsor.check_pending();
        assert!(format!("{:?}", served.sponsor).contains("[redacted]"));
        hex(&std::fs::read(dir.join(chiacraft_sponsor::net::KEY_FILE)).unwrap())
    };
    let logged = LOGGED.lock().unwrap().clone();
    assert!(logged.contains("pushed"), "the sponsor logged its work: {logged}");
    let mut texts = vec![("the log".to_owned(), logged)];
    for f in ["sponsor-log.jsonl", "chain-cache.json"] {
        let text = std::fs::read_to_string(dir.join(f)).unwrap();
        texts.push((f.to_owned(), text));
    }
    let list = words.words();
    let runs: Vec<String> = list.expose().windows(3).map(|w| w.join(" ")).collect();
    for (name, text) in &texts {
        assert!(!text.contains(&phrase), "{name} holds the words");
        assert!(!text.contains(&entropy), "{name} holds the entropy");
        assert!(!text.contains(&endpoint_key), "{name} holds the endpoint's key");
        for run in &runs {
            assert!(!text.contains(run.as_str()), "{name} holds three of the words in a row");
        }
    }
    let _ = std::fs::remove_dir_all(dir);
}
