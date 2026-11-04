//! `chiacraft-sponsor [--config sponsor.toml] [--address | --funding-address]`: serves the
//! sponsor over iroh (ALPN `chiacraft/sponsor/1`) at the endpoint whose key is kept in the data
//! folder (`sponsor.key`, made the first time). `--address` prints that endpoint's id (the
//! sponsor's address, for games' `chiacraft:sponsors` setting) and exits; `--funding-address`
//! prints the funding wallet's address (to fund it) and exits. The funding words come from
//! `CHIACRAFT_SPONSOR_WORDS` or `CHIACRAFT_SPONSOR_WORDS_FILE` (or the config's `words_file`);
//! they are never logged. Log levels: `CHIACRAFT_SPONSOR_LOG` (`info` by default; `debug`,
//! `warn`).
use chiacraft_sponsor::{Config, Sponsor, config::env_var, load_words, net};
use chiacraft_wallet::chain::{self, WalletKeys};
use std::sync::Arc;
use std::time::Duration;

/// Log lines to standard error: time, level, text.
struct Logger(log::LevelFilter);
impl log::Log for Logger {
    fn enabled(&self, m: &log::Metadata) -> bool {
        // Only ours: iroh's own lines are many and say nothing to an operator.
        m.level() <= self.0 && m.target().starts_with("chiacraft")
    }
    fn log(&self, r: &log::Record) {
        if self.enabled(r.metadata()) {
            eprintln!("{} {:<5} {}", chiacraft_sponsor::service::now_seconds(), r.level(), r.args());
        }
    }
    fn flush(&self) {}
}

fn fail(text: String) -> ! {
    eprintln!("chiacraft-sponsor: {text}");
    std::process::exit(2)
}

const USAGE: &str = "chiacraft-sponsor [--config sponsor.toml] [--address | --funding-address]";

fn main() {
    let level = env_var("SPONSOR_LOG")
        .and_then(|l| l.into_string().ok())
        .and_then(|l| l.parse().ok())
        .unwrap_or(log::LevelFilter::Info);
    let _ = log::set_boxed_logger(Box::new(Logger(level))).map(|()| log::set_max_level(level));
    let mut args = std::env::args().skip(1);
    let mut config_path = None;
    let (mut address_only, mut funding_only) = (false, false);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--config" => {
                config_path = Some(
                    args.next()
                        .unwrap_or_else(|| fail("--config needs a file".into())),
                )
            }
            "--address" => address_only = true,
            "--funding-address" => funding_only = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return;
            }
            other => fail(format!("unknown argument {other:?} (try --help)")),
        }
    }
    let config = match &config_path {
        Some(p) => Config::load(p.as_ref()),
        None => Ok(Config::default()),
    }
    .unwrap_or_else(|e| fail(e));
    let network = config.chain_network().unwrap_or_else(|e| fail(e));
    let key = net::load_key(&config.data_dir).unwrap_or_else(|e| fail(e));
    if address_only {
        println!("{}", key.public());
        return;
    }
    let words = load_words(&config).unwrap_or_else(|e| fail(e));
    if funding_only {
        let funding = chain::address(WalletKeys::from_mnemonic(&words, network, 1).primary(), network);
        println!("{funding}");
        return;
    }
    let sponsor = Arc::new(Sponsor::start(config, &words).unwrap_or_else(|e| fail(e)));
    drop(words);
    if !sponsor.wait_synced(Duration::from_secs(60)) {
        log::warn!("not synced with the chain yet: requests get 503 until it is");
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap_or_else(|e| fail(e.to_string()));
    runtime.block_on(async {
        let ep = net::endpoint(key).await.unwrap_or_else(|e| fail(e));
        log::info!("serving at {} (ALPN chiacraft/sponsor/1)", ep.id());
        tokio::select! {
            () = net::serve(sponsor.clone(), ep.clone()) => {}
            _ = tokio::signal::ctrl_c() => {}
        }
        ep.close().await;
    });
    log::info!("stopped");
}
