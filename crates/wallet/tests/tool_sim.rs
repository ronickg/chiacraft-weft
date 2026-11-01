//! Identity row I12: the live test tool (`chiacraft-wallet-test`, `chain::tool`) run on
//! chia-sdk-test's peer simulator, never a real network: each spending step says what it spends
//! and on which network and asks first, the caps refuse above them, mainnet needs its
//! confirmation, and the words never reach the output. Throwaway keys only.
#![cfg(feature = "sim")]

use chiacraft_wallet::{
    Mnemonic,
    chain::{
        ChainNetwork, WalletKeys,
        reader::Dialer,
        sim::Sim,
        tool::{self, Io},
    },
    spending::MAINNET_CONFIRM,
};
use std::{
    io::{Read, Write},
    path::PathBuf,
    time::Duration,
};

/// A throwaway wallet's words as the tool reads them, and its keys.
struct Wallet {
    words: String,
    keys: WalletKeys,
}

impl Wallet {
    fn new() -> Self {
        let m = Mnemonic::generate().unwrap();
        let words = m.words().expose().join(" ");
        let keys = WalletKeys::from_mnemonic(&m, ChainNetwork::Testnet11, 1);
        Self { words, keys }
    }
}

struct Run {
    result: Result<(), String>,
    out: String,
    asked: usize,
}

fn scratch(name: &str) -> PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("hz-tool-{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Run the tool with `args`, `wallet`'s words in the environment, answering `answer`.
fn run(sim: Option<&Sim>, wallet: Option<&Wallet>, args: &str, answer: bool) -> Run {
    let dir = scratch("data");
    let mut args: Vec<String> = args.split_whitespace().map(str::to_owned).collect();
    args.splice(0..0, ["--data".to_owned(), dir.display().to_string()]);
    run_args(sim, wallet, args, answer)
}

fn run_args(sim: Option<&Sim>, wallet: Option<&Wallet>, args: Vec<String>, answer: bool) -> Run {
    let words = wallet.map(|w| w.words.clone());
    let var = move |name: &str| match name {
        tool::WORDS_VAR => words.clone(),
        _ => None,
    };
    let mut asked = 0;
    let mut ask = |_: &str| {
        asked += 1;
        answer
    };
    let mut out = Vec::new();
    let make = sim.map(|s| {
        let inner = s.sim.clone();
        move || -> Box<dyn Dialer> { Box::new(chiacraft_wallet::chain::sim::SimDialer(inner.clone())) }
    });
    let make_ref: Option<&dyn Fn() -> Box<dyn Dialer>> = make.as_ref().map(|m| m as _);
    let mut io =
        Io { var: &var, ask: &mut ask, out: &mut out, dialer: make_ref, wait: Duration::from_secs(20), step: Duration::from_millis(100) };
    let result = tool::run(&args, &mut io).map_err(|e| e.to_string());
    let out = String::from_utf8(out).unwrap();
    if let Some(w) = wallet {
        no_words(&out, &w.words);
        if let Err(e) = &result {
            no_words(e, &w.words);
        }
    }
    Run { result, out, asked }
}

/// Neither the words nor any three of them in a row appear.
fn no_words(text: &str, words: &str) {
    assert!(!text.contains(words), "the words leaked");
    let w: Vec<&str> = words.split(' ').collect();
    for three in w.windows(3) {
        assert!(!text.contains(&three.join(" ")), "three words in a row leaked");
    }
}

fn until(mut f: impl FnMut() -> bool) -> bool {
    let end = std::time::Instant::now() + Duration::from_secs(20);
    while std::time::Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    f()
}

#[test]
fn mainnet_needs_the_confirmation_and_words_stay_out_of_sight() {
    let w = Wallet::new();
    let r = run(None, Some(&w), "--network mainnet addresses", true);
    let e = r.result.unwrap_err();
    assert!(e.contains(MAINNET_CONFIRM), "{e}");
    let args = vec![
        "--network".into(),
        "mainnet".into(),
        "--mainnet-confirm".into(),
        MAINNET_CONFIRM.into(),
        "addresses".into(),
        "--count".into(),
        "2".into(),
    ];
    let r = run_args(None, Some(&w), args, true);
    r.result.unwrap();
    assert!(r.out.contains("MAINNET · real XCH"), "{}", r.out);
    assert!(r.out.contains("xch1"), "{}", r.out);
    assert!(!r.out.contains("txch1"), "{}", r.out);
    // testnet11 by default.
    let r = run(None, Some(&w), "addresses", true);
    assert!(r.out.contains("testnet11 · test coins") && r.out.contains("txch1"), "{}", r.out);
    // No words anywhere: refused, naming where to put them.
    let r = run(None, None, "addresses", true);
    let e = r.result.unwrap_err();
    assert!(e.contains(tool::WORDS_FILE_VAR), "{e}");
    // Words as an argument are refused outright.
    let r = run(None, None, "--words abandon addresses", true);
    assert!(r.result.unwrap_err().contains("never an argument"));
    // A words file others can read is warned about; its words never printed.
    let dir = scratch("words");
    let file = dir.join("words.txt");
    std::fs::write(&file, &w.words).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
    }
    let args = vec!["--words-file".into(), file.display().to_string(), "addresses".into()];
    let r = run_args(None, None, args, true);
    r.result.unwrap();
    no_words(&r.out, &w.words);
    #[cfg(unix)]
    assert!(r.out.contains("chmod 600"), "{}", r.out);
}

#[test]
fn create_did_says_what_it_spends_asks_and_keeps_to_the_caps() {
    let sim = Sim::start().unwrap();
    let w = Wallet::new();
    sim.pay(w.keys.primary(), 1_000_000).unwrap();
    // Above the cap: refused before anything is asked.
    let r = run(Some(&sim), Some(&w), "--max-did-fee 5 create-did --fee 10", true);
    let e = r.result.unwrap_err();
    assert!(e.contains("max-did-fee"), "{e}");
    assert_eq!(r.asked, 0);
    // Said no: nothing spent.
    let r = run(Some(&sim), Some(&w), "create-did --fee 10", false);
    assert_eq!(r.result.unwrap_err(), "cancelled: nothing was spent");
    assert_eq!(r.asked, 1);
    assert!(r.out.contains("This step will:"), "{}", r.out);
    assert!(r.out.contains("11 mojos"), "{}", r.out);
    assert!(r.out.contains("testnet11"), "{}", r.out);
    assert_eq!(sim.balance(w.keys.primary()), 1_000_000);
    // --yes: made, confirmed, held by this wallet.
    let r = run(Some(&sim), Some(&w), "create-did --fee 10 --yes", false);
    r.result.unwrap();
    assert_eq!(r.asked, 0);
    assert!(r.out.contains("Confirmed at height"), "{}", r.out);
    assert!(r.out.contains("(this wallet)"), "{}", r.out);
    let did = r
        .out
        .split_whitespace()
        .find(|t| t.starts_with("did:chia:1"))
        .unwrap()
        .trim_end_matches(',')
        .to_owned();
    // Read only: `did` needs no words.
    let r = run(Some(&sim), None, &format!("did {did}"), true);
    r.result.unwrap();
    assert!(r.out.contains("held by txch1"), "{}", r.out);
    // `balance` lists the unspent coins only: the change, not the spent coin nor the DID's.
    let r = run(Some(&sim), Some(&w), "balance", true);
    r.result.unwrap();
    let listed: Vec<&str> = r.out.lines().filter(|l| l.contains("(coin ")).collect();
    assert_eq!(listed.len(), 1, "{}", r.out);
    assert!(listed[0].contains("999989 mojos"), "{}", r.out);
    assert!(r.out.contains("Balance: 999989 mojos"), "{}", r.out);
    assert!(r.out.contains("in 1 coin(s)"), "{}", r.out);
}

#[test]
fn an_offer_made_by_one_wallet_is_taken_by_another() {
    let sim = Sim::start().unwrap();
    let maker = Wallet::new();
    let taker = Wallet::new();
    sim.pay(maker.keys.primary(), 10).unwrap();
    sim.pay(taker.keys.primary(), 1_000_000).unwrap();
    let dir = scratch("offer");
    let file = dir.join("admission.offer");
    let r = run(Some(&sim), Some(&maker), &format!("make-offer --price 5000 --minutes 30 --out {} --yes", file.display()), true);
    r.result.unwrap();
    assert!(r.out.contains("gives 1 mojos"), "{}", r.out);
    assert!(r.out.contains("asks 5000 mojos"), "{}", r.out);
    assert!(file.exists());
    // The taker sees the price and the fee before paying.
    let r = run(Some(&sim), Some(&taker), &format!("take-offer {} --fee 7", file.display()), true);
    r.result.unwrap();
    assert!(r.out.contains("pay 5007 mojos"), "{}", r.out);
    assert!(r.out.contains("receive 1 mojos"), "{}", r.out);
    assert!(until(|| sim.balance(maker.keys.primary()) == 10 - 1 + 5000), "{}", sim.balance(maker.keys.primary()));
    // A cap on taking: refused with the cap named.
    let r = run(Some(&sim), Some(&taker), &format!("--max-take 4999 take-offer {}", file.display()), true);
    assert!(r.result.unwrap_err().contains("max-take"));
}

#[test]
fn mainnet_steps_are_capped_and_signed_for_mainnet() {
    let sim = Sim::start_on(ChainNetwork::Mainnet).unwrap();
    let w = Wallet::new();
    sim.pay(w.keys.primary(), 1_000_000_000).unwrap();
    let base = |rest: &str| {
        let mut a = vec!["--network".to_owned(), "mainnet".into(), "--mainnet-confirm".into(), MAINNET_CONFIRM.into()];
        a.extend(rest.split_whitespace().map(str::to_owned));
        a.splice(0..0, ["--data".to_owned(), scratch("mainnet").display().to_string()]);
        a
    };
    // mainnet's default caps.
    let r = run_args(Some(&sim), Some(&w), base("make-offer --price 10000000001 --yes"), true);
    assert!(r.result.unwrap_err().contains("max-offer-price"));
    let r = run_args(Some(&sim), Some(&w), base("create-did --fee 100000001 --yes"), true);
    assert!(r.result.unwrap_err().contains("max-did-fee"));
    let r = run_args(Some(&sim), Some(&w), base("caps"), true);
    assert!(r.out.contains("max-fee = 100000000 mojos (0.0001 XCH)"), "{}", r.out);
    // Under the caps the plan names mainnet and real XCH; the spend is signed for mainnet, which
    // this testnet11-checking simulator refuses.
    let r = run_args(Some(&sim), Some(&w), base("create-did --fee 1000"), true);
    assert_eq!(r.asked, 1);
    assert!(r.out.contains("MAINNET · real XCH"), "{}", r.out);
    assert!(r.out.contains("REAL XCH"), "{}", r.out);
    assert!(r.out.contains("1001 mojos (0.000000001001 XCH)"), "{}", r.out);
    assert!(r.out.contains("xch1"), "{}", r.out);
    let e = r.result.unwrap_err();
    assert!(e.contains("the chain refused the spend"), "{e}");
    assert_eq!(sim.balance(w.keys.primary()), 1_000_000_000);
}

#[test]
fn a_sponsor_request_spends_nothing_and_the_did_arrives() {
    let sim = Sim::start().unwrap();
    let w = Wallet::new();
    // A stand-in sponsor: it checks the request's signature and launches the DID on the
    // simulator, as horizon-sponsor does (that service has its own tests).
    let launcher = sim.sponsor_did(w.keys.primary()).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let served = std::thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let mut buf = vec![0u8; 16 * 1024];
        let mut got = Vec::new();
        loop {
            let n = s.read(&mut buf).unwrap();
            got.extend_from_slice(&buf[..n]);
            let text = String::from_utf8_lossy(&got).into_owned();
            if let Some((head, body)) = text.split_once("\r\n\r\n") {
                let len: usize = head
                    .lines()
                    .find_map(|l| l.strip_prefix("Content-Length: "))
                    .and_then(|v| v.trim().parse().ok())
                    .unwrap_or(0);
                if body.len() >= len {
                    break;
                }
            }
            if n == 0 {
                break;
            }
        }
        let text = String::from_utf8_lossy(&got).into_owned();
        let body = text.split_once("\r\n\r\n").unwrap().1.to_owned();
        let req: chiacraft_wallet::sponsor::Request = serde_json::from_str(&body).unwrap();
        assert!(req.verify());
        let did = chiacraft_wallet::chain::did_string(launcher);
        let reply = format!(
            "{{\"did\":\"{did}\",\"launcher\":\"{}\",\"coin\":\"\",\"status\":\"pending\"}}",
            chiacraft_wallet::chain::hex32(launcher)
        );
        write!(
            s,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply}",
            reply.len()
        )
        .unwrap();
        req
    });
    let r = run(Some(&sim), Some(&w), &format!("sponsor-request http://127.0.0.1:{port} --wait --yes"), true);
    r.result.unwrap();
    let req = served.join().unwrap();
    assert_eq!(req.network, "testnet11");
    assert!(req.address.starts_with("txch1"));
    assert!(r.out.contains("spend nothing from this wallet"), "{}", r.out);
    assert!(r.out.contains("held by this wallet's first address"), "{}", r.out);
    // https isn't spoken (the game's own sponsor call is).
    let r = run(Some(&sim), Some(&w), "sponsor-request https://example.org --yes", true);
    assert!(r.result.unwrap_err().contains("http://"));
}
