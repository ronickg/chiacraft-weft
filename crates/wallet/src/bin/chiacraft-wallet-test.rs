//! `chiacraft-wallet-test`: run one live wallet step against testnet11 (the default) or mainnet
//! (opt-in), saying what it spends and on which network before it does (identity row I12;
//! docs/identity-live-test.md). The logic is `chiacraft_wallet::chain::tool`; this binary gives it
//! the environment, the terminal and real peers. The words come from `HORIZON_TEST_WALLET_WORDS`
//! or a file (`HORIZON_TEST_WALLET_WORDS_FILE`, `--words-file`) and are never printed.
use chiacraft_wallet::chain::tool::{self, Io};
use std::io::{BufRead, Write};
use std::time::Duration;

/// Warnings and errors of the wallet's own code to standard error (never a secret: the wallet
/// logs none).
struct Logger;
impl log::Log for Logger {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::Level::Warn
    }
    fn log(&self, r: &log::Record) {
        if self.enabled(r.metadata()) {
            eprintln!("{}: {}", r.level(), r.args());
        }
    }
    fn flush(&self) {}
}

fn main() {
    static LOGGER: Logger = Logger;
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(log::LevelFilter::Warn);
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    let var = |name: &str| std::env::var(name).ok();
    let mut ask = |question: &str| {
        print!("{question}");
        let _ = std::io::stdout().flush();
        let mut line = String::new();
        let _ = std::io::stdin().lock().read_line(&mut line);
        matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes")
    };
    let mut out = std::io::stdout();
    let mut io = Io {
        var: &var,
        ask: &mut ask,
        out: &mut out,
        dialer: None,
        wait: Duration::from_secs(20 * 60),
        step: Duration::from_secs(5),
    };
    if let Err(e) = tool::run(&args, &mut io) {
        eprintln!("chiacraft-wallet-test: {e}");
        std::process::exit(1);
    }
}
