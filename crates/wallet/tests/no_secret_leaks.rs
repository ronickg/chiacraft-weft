//! The rule "no wallet secret ever reaches logs, telemetry, crash reports or the clipboard"
//! (identity-research.md 1.6, PROJECT.md "Wallet rules"), enforced over the repository's Rust:
//!
//! 1. No log, print or panic macro anywhere in `crates/*/src` takes a secret: an argument naming
//!    a mnemonic, passphrase or entropy, or calling `.expose()`, `.words()` or `.entropy()` (the
//!    only ways a secret leaves [`chiacraft_wallet::Secret`] and [`chiacraft_wallet::Mnemonic`]).
//! 2. The telemetry crate and the crash reporter don't know the wallet at all.
//! 3. Nothing puts an account page value on the clipboard (`Event::Copy` in its module).
//! 4. Every secret-bearing type formats as redacted.
//! 5. The chain reader (`src/chain/`, feature `chain`) keeps spending keys only in `Secret` and
//!    logs no spend bundle, coin spend, solution, signature or key.
//!
//! The runtime half, `tools/chiacraft/identity_check.py`, restores a known test account and greps
//! the logs, the telemetry outbox, crash reports, the control socket's replies and the saves for
//! its words.
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
fn telemetry_and_crash_reports_dont_know_the_wallet() {
    let mut files = Vec::new();
    rust_files(&repo().join("crates/horizon-telemetry/src"), &mut files);
    files.push(repo().join("crates/horizon-client/src/crash.rs"));
    files.push(repo().join("crates/horizon-client/src/telemetry.rs"));
    let mut found = Vec::new();
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap_or_default();
        for word in ["chiacraft_wallet", "account::", "Mnemonic", "Secret<"] {
            if text.contains(word) {
                found.push(format!("{}: {word}", f.display()));
            }
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
    let toml = std::fs::read_to_string(repo().join("crates/horizon-telemetry/Cargo.toml")).unwrap();
    assert!(!toml.contains("horizon-wallet"));
}

#[test]
fn the_account_page_copies_nothing() {
    let page = repo().join("crates/horizon-client/src/menus/account.rs");
    let text = std::fs::read_to_string(&page).expect("the account page");
    for word in ["Event::Copy", "clipboard"] {
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
    rust_files(&repo().join("crates/horizon-wallet/src/chain"), &mut out);
    out
}

/// The game's code that signs or spends with wallet keys (identity row I7): the admission policy
/// engine (offers signed with the server account's keys), the player's wallet calls (taking an
/// offer, holding proofs), the admission screen and the door.
fn admission_sources() -> Vec<PathBuf> {
    let client = repo().join("crates/horizon-client/src");
    let mut out = vec![
        client.join("admission.rs"),
        client.join("wallet_chain.rs"),
        client.join("game/admission.rs"),
        client.join("lan/admission.rs"),
        client.join("account.rs"),
    ];
    rust_files(&client.join("admission"), &mut out);
    out
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

/// Identity row I8's code: the sponsor contract, hardened addresses, DIDs made and linked from the
/// Account page (its passphrase prompt), the admission screen's prompt, the verified-DID mark.
fn did_sources() -> Vec<PathBuf> {
    let client = repo().join("crates/horizon-client/src");
    let wallet = repo().join("crates/horizon-wallet/src");
    vec![
        wallet.join("sponsor.rs"),
        wallet.join("keys.rs"),
        wallet.join("accounts.rs"),
        client.join("menus/account.rs"),
        client.join("player_list.rs"),
        client.join("wallet_chain.rs"),
        client.join("game/admission.rs"),
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
    // Both passphrase prompts draw their field as a secret (dots; redacted to the control socket).
    for (file, field) in
        [("crates/horizon-client/src/menus/account.rs", "PASS,"), ("crates/horizon-client/src/game/admission.rs", "\"admission.pass\",")]
    {
        let text = std::fs::read_to_string(repo().join(file)).unwrap();
        let mut seen = 0;
        for (i, _) in text.match_indices(field) {
            let after = &text[i..(i + 400).min(text.len())];
            if after.contains("ui.account.pass\"") {
                seen += 1;
                assert!(after.contains("secret: true"), "{file}: {field} not secret");
            }
        }
        assert!(seen > 0, "{file}: no passphrase field found");
    }
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
    let client = repo().join("crates/horizon-client/src");
    vec![
        repo().join("crates/horizon-wallet/src/achievement.rs"),
        client.join("achievements.rs"),
        client.join("game/achievements.rs"),
        client.join("menus/achievements.rs"),
    ]
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
    let wallet = repo().join("crates/horizon-wallet/src");
    let files = [
        wallet.join("chain/tool.rs"),
        wallet.join("bin/chiacraft-wallet-test.rs"),
        wallet.join("spending.rs"),
        repo().join("crates/horizon-client/src/account.rs"),
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

/// The onboarding server's scripts (I12, `packaging/onboarding/`) never trace or echo the wallet
/// words: no `set -x`, no `echo`/`printf` of a words variable to the terminal, and the templates
/// name no words variable but the comment pointing at the environment. The runtime half is
/// `tools/chiacraft/onboarding_check.py`.
#[test]
fn the_onboarding_scripts_print_no_words() {
    let dir = repo().join("packaging/onboarding");
    let mut found = Vec::new();
    let mut scripts = 0;
    for e in std::fs::read_dir(&dir).unwrap().flatten() {
        let path = e.path();
        if path.is_dir() {
            continue; // the achievements package: content, no script
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let name = path.display().to_string();
        if name.ends_with(".sh") {
            scripts += 1;
        }
        for (i, l) in text.lines().enumerate() {
            let t = l.trim_start();
            if t.starts_with('#') {
                continue;
            }
            let traces = t.contains("set -o xtrace")
                || t.split_whitespace()
                    .collect::<Vec<_>>()
                    .windows(2)
                    .any(|w| w[0] == "set" && w[1].starts_with('-') && w[1].contains('x'));
            if traces {
                found.push(format!("{name}:{}: traces commands", i + 1));
            }
            let words_var =
                ["$HORIZON_SERVER_WORDS", "${HORIZON_SERVER_WORDS", "$HORIZON_SPONSOR_WORDS", "${HORIZON_SPONSOR_WORDS", "${!direct"]
                    .iter()
                    .any(|v| {
                        t.match_indices(v)
                            .any(|(at, _)| !t[at + v.len()..].starts_with("_FILE"))
                    });
            let prints = t.contains("echo ") || t.contains("printf ");
            // The one comparison prints into a pipe that normalises it, never to the terminal.
            if words_var && prints && !t.contains("| tr -s") {
                found.push(format!("{name}:{}: prints a words variable", i + 1));
            }
            if name.ends_with(".template") && t.contains("WORDS") {
                found.push(format!("{name}:{}: a template names words", i + 1));
            }
        }
    }
    assert!(scripts >= 1, "found the scripts in {}", dir.display());
    assert!(found.is_empty(), "{}", found.join("\n"));
}
