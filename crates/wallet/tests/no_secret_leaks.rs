//! The rule "no wallet secret ever reaches logs, telemetry, crash reports or the clipboard"
//! (identity-research.md 1.6, PROJECT.md "Wallet rules"), enforced over the repository's Rust:
//!
//! 1. No log, print or panic macro anywhere in `crates/*/src` takes a secret: an argument naming
//!    a mnemonic, passphrase or entropy, or calling `.expose()`, `.words()` or `.entropy()` (the
//!    only ways a secret leaves [`chiacraft_wallet::Secret`] and [`chiacraft_wallet::Mnemonic`]).
//! 2. The engine's diagnostics and log setup (what a crash report or a shared log is made of)
//!    don't know the wallet at all.
//! 3. Nothing on the Account screen goes to the clipboard.
//! 4. Every secret-bearing type formats as redacted.
//! 5. The chain reader (`src/chain/`, feature `chain`) keeps spending keys only in `Secret` and
//!    logs no spend bundle, coin spend, solution, signature or key.
//!
//! The paths are this repository's (chiacraft on Weft): `crates/wallet`, `crates/account`, the
//! engine's `chiacraft/` parts and its admission code. The sponsor service (`crates/sponsor`) has
//! a test of its own over its sources, logs and files.
use std::path::{Path, PathBuf};

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
/// Words and calls that mean a secret is in hand.
const SECRET_WORDS: &[&str] = &["mnemonic", "passphrase", "entropy"];
const SECRET_CALLS: &[&str] = &[".expose(", ".expose_mut(", ".words()", ".entropy()"];

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Every `crates/<crate>/src` file.
fn sources() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for c in std::fs::read_dir(repo().join("crates")).unwrap().flatten() {
        rust_files(&c.path().join("src"), &mut out);
    }
    assert!(out.len() > 100, "found the sources ({})", out.len());
    out
}

/// The text of each macro call's arguments: from `name!(` to its closing bracket.
fn macro_args<'a>(text: &'a str, name: &str) -> Vec<(usize, &'a str)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = text[from..].find(name) {
        let at = from + i;
        from = at + name.len();
        // A whole macro name (`log::warn!`, not `xwarn!`).
        if text[..at]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            continue;
        }
        let rest = &text[from..];
        let Some(open) = rest.chars().next().filter(|c| matches!(c, '(' | '[' | '{')) else {
            continue;
        };
        let close = match open {
            '(' => ')',
            '[' => ']',
            _ => '}',
        };
        let (mut depth, mut in_str, mut escaped) = (0i32, false, false);
        for (j, c) in rest.char_indices() {
            if in_str {
                match (escaped, c) {
                    (true, _) => escaped = false,
                    (false, '\\') => escaped = true,
                    (false, '"') => in_str = false,
                    _ => {}
                }
                continue;
            }
            match c {
                '"' => in_str = true,
                c if c == open => depth += 1,
                c if c == close => {
                    depth -= 1;
                    if depth == 0 {
                        let line = text[..at].lines().count();
                        out.push((line, &rest[1..j]));
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    out
}

fn leaks(args: &str) -> Option<String> {
    let lower = args.to_ascii_lowercase();
    for w in SECRET_WORDS {
        let mut from = 0;
        while let Some(i) = lower[from..].find(w) {
            let at = from + i;
            from = at + w.len();
            let before = lower[..at].chars().next_back();
            let after = lower[from..].chars().next();
            let word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
            if !word(before) && !word(after) {
                return Some((*w).to_owned());
            }
        }
    }
    SECRET_CALLS
        .iter()
        .find(|c| args.contains(**c))
        .map(|c| (*c).to_owned())
}

#[test]
fn no_log_print_or_panic_takes_a_secret() {
    let mut found = Vec::new();
    for path in sources() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for name in MACROS {
            for (line, args) in macro_args(&text, name) {
                if let Some(what) = leaks(args) {
                    found.push(format!("{}:{line}: {name} with {what}", path.display()));
                }
            }
        }
    }
    assert!(found.is_empty(), "wallet secrets must never be logged, printed or put in a panic (docs/identity.md):\n{}", found.join("\n"));
}

#[test]
fn diagnostics_and_logging_dont_know_the_wallet() {
    let files = [repo().join("crates/engine/src/diag.rs"), repo().join("crates/engine/src/log.rs")];
    let mut found = Vec::new();
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        for word in ["chiacraft_wallet", "chiacraft_account", "Mnemonic", "Secret<"] {
            if text.contains(word) {
                found.push(format!("{}: {word}", f.display()));
            }
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn the_account_screen_copies_nothing() {
    let page = repo().join("crates/engine/src/chiacraft/account_screen.rs");
    let text = std::fs::read_to_string(&page).expect("the account screen");
    for word in ["clipboard", "Event::Copy"] {
        assert!(
            !text
                .lines()
                .any(|l| l.contains(word) && !l.trim_start().starts_with("//")),
            "{} uses {word}",
            page.display()
        );
    }
}

#[test]
fn secret_types_format_redacted() {
    let m = chiacraft_wallet::Mnemonic::generate().unwrap();
    let words = m.words();
    let shown = [
        format!("{m:?}"),
        format!("{words:?}"),
        format!("{:?}", m.entropy()),
        format!("{:?}", m.seed()),
        format!("{:?}", chiacraft_wallet::IdentitySigner::from_mnemonic(&m, 0)),
    ];
    for s in &shown {
        for w in words.expose() {
            assert!(!s.split_whitespace().any(|x| x == w), "{s} shows a word");
        }
    }
    assert!(!shown[2].contains(&chiacraft_wallet::keys::hex(m.entropy().expose())));
}

#[test]
fn the_scanner_finds_a_leak() {
    let src = r#"fn f() { log::info!("words {}", m.words().expose().join(" ")); warn!("ok {seed}"); }"#;
    let hits: Vec<_> = macro_args(src, "info!")
        .into_iter()
        .filter_map(|(_, a)| leaks(a))
        .collect();
    assert_eq!(hits.len(), 1);
    assert!(
        macro_args(src, "warn!")
            .iter()
            .all(|(_, a)| leaks(a).is_none())
    );
    assert!(leaks("\"{passphrase}\"").is_some());
    assert!(leaks("\"passphrases_wanted\"").is_none());
}

/// The chain reader's words for what must never be logged: a spend's private parts (its
/// solutions and signatures, before it is pushed), and keys.
const CHAIN_WORDS: &[&str] = &["bundle", "coin_spend", "coin_spends", "solution", "signature", "secret", "keys", "sk", "synthetic"];

fn chain_sources() -> Vec<PathBuf> {
    let mut out = Vec::new();
    rust_files(&repo().join("crates/wallet/src/chain"), &mut out);
    out
}

/// The game's code that signs or spends with wallet keys (phase 4): the account (its DID's owner
/// key, the join's DID proof), the chain reader's use in the game, the account's and the world's
/// DIDs (the owner's metadata spend), and admission (the DID check at the join).
fn admission_sources() -> Vec<PathBuf> {
    let engine = repo().join("crates/engine/src");
    vec![
        repo().join("crates/account/src/lib.rs"),
        engine.join("chiacraft/chain.rs"),
        engine.join("chiacraft/did.rs"),
        engine.join("chiacraft/worlds/world_did.rs"),
        engine.join("chiacraft/worlds/admission.rs"),
        engine.join("server/admission.rs"),
    ]
}

fn mentions(args: &str, words: &[&str]) -> Option<String> {
    let lower = args.to_ascii_lowercase();
    let word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
    for w in words {
        let mut from = 0;
        while let Some(i) = lower[from..].find(w) {
            let at = from + i;
            from = at + w.len();
            if !word(lower[..at].chars().next_back()) && !word(lower[from..].chars().next()) {
                return Some((*w).to_owned());
            }
        }
    }
    None
}

#[test]
fn the_chain_reader_logs_no_spend_or_key() {
    let files = chain_sources();
    assert!(files.len() >= 8, "found the chain module ({})", files.len());
    let mut found = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        for name in MACROS {
            for (line, args) in macro_args(&text, name) {
                if let Some(what) = mentions(args, CHAIN_WORDS).or_else(|| leaks(args)) {
                    found.push(format!("{}:{line}: {name} with {what}", path.display()));
                }
            }
        }
        // A spending key is held only inside `Secret` (no `SecretKey` field or static).
        for (i, l) in text.lines().enumerate() {
            let t = l.trim_start();
            if t.starts_with("//") {
                continue;
            }
            // An owned key in a field, a binding's type or a container (borrowed `&SecretKey`
            // arguments and `SecretKey::from_bytes` at the point of signing are fine).
            let owned = t.contains(": SecretKey") || t.contains("<SecretKey");
            if owned || t.contains("static") && t.contains("SecretKey") {
                found.push(format!("{}:{}: a SecretKey outside Secret", path.display(), i + 1));
            }
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn admission_logs_no_spend_or_key() {
    let files = admission_sources();
    let mut found = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for name in MACROS {
            for (line, args) in macro_args(&text, name) {
                if let Some(what) = mentions(args, CHAIN_WORDS).or_else(|| leaks(args)) {
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
fn owner_signers_format_redacted() {
    let m = chiacraft_wallet::Mnemonic::generate().unwrap();
    let o = chiacraft_wallet::OwnerSigner::from_mnemonic(&m, 3, true);
    let shown = format!("{o:?}");
    assert!(shown.contains("[redacted]"), "{shown}");
    assert!(!shown.contains(&chiacraft_wallet::keys::hex(&o.public())));
}

#[test]
fn the_chain_scanner_finds_a_leak() {
    assert!(mentions("\"pushed {bundle:?}\"", CHAIN_WORDS).is_some());
    assert!(mentions("\"{}\", hex(sk.to_bytes())", CHAIN_WORDS).is_some());
    assert!(mentions("\"chain: synced to height {h}\"", CHAIN_WORDS).is_none());
    assert!(mentions("\"bundles_sent\"", CHAIN_WORDS).is_none());
}

#[cfg(feature = "chain")]
#[test]
fn wallet_keys_format_redacted() {
    use chiacraft_wallet::chain::{ChainCache, ChainNetwork, WalletKeys};
    let m = chiacraft_wallet::Mnemonic::generate().unwrap();
    let keys = WalletKeys::from_mnemonic(&m, ChainNetwork::Testnet11, 2);
    let shown = format!("{keys:?}");
    assert!(shown.contains("[redacted]"));
    let words = m.words();
    for w in words.expose() {
        assert!(!shown.split_whitespace().any(|x| x == w));
    }
    // The cache holds public facts only: puzzle hashes, never the master key or the entropy.
    let cache = ChainCache::new(ChainNetwork::Testnet11, keys.puzzle_hashes());
    let json = serde_json::to_string(&cache).unwrap();
    assert!(!json.contains(&chiacraft_wallet::keys::hex(m.entropy().expose())));
    assert!(!json.contains(&chiacraft_wallet::keys::hex(&chiacraft_wallet::keys::facts_of(&m).master)));
}

/// The DID code (phase 4): the sponsor contract, keys and hardened addresses, the accounts file
/// (the linked DID), the Account screen, and getting and linking DIDs in the game.
fn did_sources() -> Vec<PathBuf> {
    let wallet = repo().join("crates/wallet/src");
    let engine = repo().join("crates/engine/src");
    vec![
        wallet.join("sponsor.rs"),
        wallet.join("keys.rs"),
        wallet.join("accounts.rs"),
        repo().join("crates/account/src/lib.rs"),
        engine.join("chiacraft/account_screen.rs"),
        engine.join("chiacraft/did.rs"),
        engine.join("chiacraft/worlds/world_did.rs"),
    ]
}

#[test]
fn did_code_logs_no_spend_key_or_passphrase() {
    let mut found = Vec::new();
    for path in did_sources() {
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for name in MACROS {
            for (line, args) in macro_args(&text, name) {
                if let Some(what) = mentions(args, CHAIN_WORDS)
                    .or_else(|| mentions(args, &["pass", "prompt"]))
                    .or_else(|| leaks(args))
                {
                    found.push(format!("{}:{line}: {name} with {what}", path.display()));
                }
            }
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn sponsor_requests_carry_nothing_secret() {
    let m = chiacraft_wallet::Mnemonic::generate().unwrap();
    let s = chiacraft_wallet::IdentitySigner::from_mnemonic(&m, 0);
    let r = chiacraft_wallet::sponsor::Request::sign(&s, "testnet11", "txch1x", 1).unwrap();
    let json = r.to_json();
    assert!(!json.contains(&chiacraft_wallet::keys::hex(m.entropy().expose())));
    assert!(!json.contains(&chiacraft_wallet::keys::hex(&chiacraft_wallet::keys::facts_of(&m).master)));
    let words = m.words();
    assert!(!json.contains(&words.expose()[..3].join(" ")));
}

/// Achievements (identity row I10): the credential, the store, the game's granting and the page.
fn achievement_sources() -> Vec<PathBuf> {
    // The game grants none yet (phase 4 after DIDs): the credential alone.
    vec![repo().join("crates/wallet/src/achievement.rs")]
}

/// Whether line `i` of `text` comes after its `#[cfg(test)]` module starts (tests make
/// throwaway keys).
fn in_test(text: &str, i: usize) -> bool {
    text.lines()
        .take(i)
        .any(|l| l.trim_start().starts_with("#[cfg(test)]"))
}

/// The server's identity key signs credentials and revocation lists only through the `Signer`
/// interface, for the call: no achievement code reveals words, derives or holds a secret key, or
/// keeps a signer in its state; and none logs a key, a signature or a spend.
#[test]
fn achievements_sign_only_through_the_signer_for_the_call() {
    let mut found = Vec::new();
    for path in achievement_sources() {
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for name in MACROS {
            for (line, args) in macro_args(&text, name) {
                if let Some(what) = mentions(args, CHAIN_WORDS).or_else(|| leaks(args)) {
                    found.push(format!("{}:{line}: {name} with {what}", path.display()));
                }
            }
        }
        let mut in_state = false;
        for (i, l) in text.lines().enumerate() {
            let t = l.trim_start();
            if t.starts_with("//") {
                continue;
            }
            for bad in ["SecretKey", ".expose(", "reveal(", "from_mnemonic(", "WalletKeys"] {
                if t.contains(bad) && !in_test(&text, i) {
                    found.push(format!("{}:{}: {bad}", path.display(), i + 1));
                }
            }
            if t.starts_with("pub(super) struct AchievementsState") {
                in_state = true;
            }
            if in_state && t.contains("Signer") {
                found.push(format!("{}:{}: the state keeps a signer", path.display(), i + 1));
            }
            if in_state && t == "}" {
                in_state = false;
            }
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}

/// The live test tool (identity row I12, `chain/tool.rs` and its binary) writes what it does to
/// its output: none of its writes, lines or messages takes a secret, its words leave `Secret` only
/// to be parsed, and the binary installs no logger below warnings. The server's words (I12,
/// `account::server_words`) the same.
#[test]
fn the_live_test_tool_writes_no_secret() {
    let wallet = repo().join("crates/wallet/src");
    let files = [
        wallet.join("chain/tool.rs"),
        wallet.join("bin/chiacraft-wallet-test.rs"),
        wallet.join("spending.rs"),
        repo().join("crates/account/src/lib.rs"),
    ];
    let mut found = Vec::new();
    for path in &files {
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for name in ["writeln!", "write!", "format!", "say", "Error::new"]
            .iter()
            .chain(MACROS)
        {
            for (line, args) in macro_args(&text, name) {
                if let Some(what) = leaks(args) {
                    found.push(format!("{}:{line}: {name} with {what}", path.display()));
                }
            }
        }
    }
    let tool = std::fs::read_to_string(wallet.join("chain/tool.rs")).unwrap();
    // The words are read into a Secret and exposed once: to parse them.
    let exposed: Vec<&str> = tool.lines().filter(|l| l.contains(".expose(")).collect();
    assert!(exposed.len() == 1 && exposed[0].contains("Mnemonic::parse("), "{exposed:?}");
    assert!(!tool.contains(".words()"), "the tool never lists the words");
    let bin = std::fs::read_to_string(wallet.join("bin/chiacraft-wallet-test.rs")).unwrap();
    assert!(bin.contains("log::LevelFilter::Warn") && !bin.contains("LevelFilter::Debug"));
    assert!(found.is_empty(), "{}", found.join("\n"));
}

/// The sponsor's example settings and README name the words only by where they are kept (a file
/// or an environment variable), never hold any.
#[test]
fn the_sponsors_settings_hold_no_words() {
    let dir = repo().join("crates/sponsor");
    let mut found = Vec::new();
    for name in ["sponsor.example.toml", "README.md"] {
        let path = dir.join(name);
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for (i, l) in text.lines().enumerate() {
            let t = l.trim_start();
            // A words value written in: `words = "…"` (only `words_file` may be set).
            if t.starts_with("words") && !t.starts_with("words_file") && t.contains('=') {
                found.push(format!("{}:{}: words written in", path.display(), i + 1));
            }
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}
