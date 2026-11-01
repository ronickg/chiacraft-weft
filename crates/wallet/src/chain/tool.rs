//! `chiacraft-wallet-test`: the live test tool Ronald runs himself (identity row I12), one step
//! at a time, against testnet11 (the default) or mainnet (opt-in), with a throwaway wallet.
//!
//! Every step that spends first prints **what it will spend, from which address, on which
//! network**, then asks (`--yes` skips the question), and is held to the wallet's caps
//! ([`crate::spending::SpendCaps`]; `--max-fee` and the other cap flags change them). The words
//! come only from `HORIZON_TEST_WALLET_WORDS`, a file named by `HORIZON_TEST_WALLET_WORDS_FILE`,
//! or `--words-file`: never an argument, never printed, never logged. Mainnet needs `--network
//! mainnet` and `--mainnet-confirm "I understand this spends real XCH"` (or
//! `HORIZON_MAINNET_CONFIRM`), as every other place that reaches mainnet.
//!
//! The logic lives here (not in the binary) so the simulator tests run it: [`run`] takes its
//! environment, its question and its output as [`Io`].
use super::{
    Bytes32, ChainConfig, ChainNetwork, ChainReader, MakeOffer, OfferGive, OfferState, PushOutcome, WalletKeys, did_string, hex32, offer,
    parse_did, reader::Dialer,
};
use crate::{
    Error, IdentitySigner, Mnemonic, Result, Secret, keys,
    spending::{self, MainnetConsent, SpendCaps},
};
use std::{
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};

/// The wallet's words, in the environment.
pub const WORDS_VAR: &str = "HORIZON_TEST_WALLET_WORDS";
/// A file holding the wallet's words.
pub const WORDS_FILE_VAR: &str = "HORIZON_TEST_WALLET_WORDS_FILE";
/// The network (instead of `--network`).
pub const NETWORK_VAR: &str = "HORIZON_TEST_NETWORK";
/// Mainnet's confirmation (instead of `--mainnet-confirm`).
pub const CONFIRM_VAR: &str = "HORIZON_MAINNET_CONFIRM";

/// The default fee of a step that pays one: 0.00001 (T)XCH, under mainnet's caps.
pub const DEFAULT_FEE: u64 = 10_000_000;

pub const USAGE: &str = "\
chiacraft-wallet-test [options] <command>

Commands (each step that spends says what it spends and on which network, then asks):
  new-wallet --out FILE          fresh 24 words written to FILE (made with mode 600, never printed; refused
                                 if FILE exists or sits inside a git checkout); prints the address to fund
  addresses [--count N]          the wallet's first addresses (unhardened, then hardened); no network
  balance                        sync and show the coins and the balance
  caps                           the spending caps on this network
  create-did [--fee MOJOS]       make a DID paid by this wallet (1 mojo + the fee), wait for it
  make-offer --price MOJOS [--give MOJOS] [--minutes M] [--fee MOJOS] [--out FILE] [--wait]
                                 offer --give mojos (1) for --price; write the offer file
  take-offer FILE [--fee MOJOS]  take an offer from a file (or the text itself)
  did ID                         look a DID up (did:chia:1... or its launcher id); read only
  send ADDRESS --amount MOJOS [--fee MOJOS]
                                 pay ADDRESS from this wallet (capped by max-send on mainnet)
  sponsor-request URL [--wait]   ask a DID sponsor (http://host:port) for a DID at your first address

Options:
  --network testnet11|mainnet    default testnet11 (or HORIZON_TEST_NETWORK)
  --mainnet-confirm TEXT         \"I understand this spends real XCH\" (or HORIZON_MAINNET_CONFIRM)
  --words-file PATH              the wallet's words (or HORIZON_TEST_WALLET_WORDS[_FILE]); never an argument
  --data DIR                     the chain cache's folder (default chiacraft-wallet-test)
  --peers HOST:PORT,...          own full nodes (or HORIZON_CHAIN_PEERS); default: the DNS introducers
  --addresses N                  addresses followed of each kind (default 20)
  --yes                          don't ask before spending
  --max-fee, --max-offer-price, --max-take, --max-did-fee, --max-anchor-fee, --max-split MOJOS
                                 change a spending cap (mainnet's are small by default)
";

/// What [`run`] reads, asks and writes.
pub struct Io<'a> {
    /// The environment (`std::env::var` in the binary).
    pub var: &'a dyn Fn(&str) -> Option<String>,
    /// Ask a yes/no question; true: go ahead.
    pub ask: &'a mut dyn FnMut(&str) -> bool,
    pub out: &'a mut dyn Write,
    /// How peers are reached (tests: the simulator); None: TLS to real peers.
    pub dialer: Option<&'a dyn Fn() -> Box<dyn Dialer>>,
    /// How long to wait for a sync, a DID or an offer.
    pub wait: Duration,
    /// How often to look again while waiting.
    pub step: Duration,
}

impl Dialer for Box<dyn Dialer> {
    fn candidates(&self) -> super::reader::BoxFuture<'_, Vec<String>> {
        (**self).candidates()
    }
    fn dial(&self, address: String) -> super::reader::BoxFuture<'_, Result<super::reader::Connection>> {
        (**self).dial(address)
    }
}

#[derive(Default)]
struct Opts {
    command: String,
    positional: Vec<String>,
    network: Option<String>,
    confirm: Option<String>,
    words_file: Option<PathBuf>,
    data: Option<PathBuf>,
    peers: Option<String>,
    addresses: u32,
    yes: bool,
    wait: bool,
    caps: Vec<(String, String)>,
    fee: Option<u64>,
    price: Option<u64>,
    give: Option<u64>,
    minutes: Option<u64>,
    count: Option<u32>,
    out: Option<PathBuf>,
    amount: Option<u64>,
}

fn number<T: std::str::FromStr>(flag: &str, value: Option<String>) -> Result<T> {
    let v = value.ok_or_else(|| Error::new(format!("{flag} needs a value")))?;
    v.trim()
        .parse()
        .map_err(|_| Error::new(format!("{flag} {v}: not a number")))
}

fn parse_args(args: &[String]) -> Result<Opts> {
    let mut o = Opts { addresses: super::wallet::DEFAULT_ADDRESSES, ..Opts::default() };
    let mut it = args.iter().cloned();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--network" => o.network = it.next(),
            "--mainnet-confirm" => o.confirm = it.next(),
            "--words-file" => o.words_file = it.next().map(PathBuf::from),
            "--data" => o.data = it.next().map(PathBuf::from),
            "--peers" => o.peers = it.next(),
            "--addresses" => o.addresses = number(&a, it.next())?,
            "--yes" | "-y" => o.yes = true,
            "--help" | "-h" => o.command = "help".to_owned(),
            "--wait" => o.wait = true,
            "--fee" => o.fee = Some(number(&a, it.next())?),
            "--price" => o.price = Some(number(&a, it.next())?),
            "--give" => o.give = Some(number(&a, it.next())?),
            "--minutes" => o.minutes = Some(number(&a, it.next())?),
            "--count" => o.count = Some(number(&a, it.next())?),
            "--out" => o.out = it.next().map(PathBuf::from),
            "--amount" => o.amount = Some(number(&a, it.next())?),
            "--words" => {
                return Err(Error::new(
                    "words are never an argument (they would land in the shell's history): use HORIZON_TEST_WALLET_WORDS_FILE or --words-file",
                ));
            }
            flag if flag.starts_with("--max-") => {
                let name = &flag[2..];
                if !spending::CAP_NAMES.contains(&name) {
                    return Err(Error::new(format!("unknown cap {flag}")));
                }
                let v = it
                    .next()
                    .ok_or_else(|| Error::new(format!("{flag} needs a value")))?;
                o.caps.push((name.to_owned(), v));
            }
            flag if flag.starts_with('-') => {
                return Err(Error::new(format!("unknown option {flag} (try --help)")));
            }
            _ if o.command.is_empty() => o.command = a,
            _ => o.positional.push(a),
        }
    }
    Ok(o)
}

/// Run the tool with `args` (without the program name).
pub fn run(args: &[String], io: &mut Io<'_>) -> Result<()> {
    let o = parse_args(args)?;
    if o.command.is_empty() || matches!(o.command.as_str(), "help" | "-h" | "--help") {
        write!(io.out, "{USAGE}").ok();
        return Ok(());
    }
    if o.command == "new-wallet" {
        return new_wallet(&o, io);
    }
    let mut tool = Tool::new(o, io)?;
    let command = tool.o.command.clone();
    match command.as_str() {
        "addresses" => tool.addresses(),
        "caps" => tool.caps_command(),
        "balance" => tool.balance(),
        "create-did" => tool.create_did(),
        "make-offer" => tool.make_offer(),
        "take-offer" => tool.take_offer(),
        "did" => tool.did(),
        "send" => tool.send(),
        "sponsor-request" => tool.sponsor_request(),
        other => Err(Error::new(format!("unknown command {other:?} (try --help)"))),
    }
}

/// `new-wallet --out FILE`: fresh words straight into a new file only its owner can read; only the
/// fingerprint and the first address (to fund) are printed. Spends nothing, needs no network.
fn new_wallet(o: &Opts, io: &mut Io<'_>) -> Result<()> {
    let path = o
        .out
        .clone()
        .ok_or_else(|| Error::new("new-wallet needs --out FILE (where the words go)"))?;
    if path.exists() {
        return Err(Error::new(format!("{} exists already: new-wallet never overwrites a file", path.display())));
    }
    let dir = std::path::absolute(&path).map_err(|e| Error::new(format!("{}: {e}", path.display())))?;
    if dir.ancestors().skip(1).any(|d| d.join(".git").exists()) {
        return Err(Error::new(format!("{} is inside a git checkout: keep wallet words outside any repository", path.display())));
    }
    let network = match o
        .network
        .clone()
        .or_else(|| (io.var)(NETWORK_VAR))
        .unwrap_or_default()
        .as_str()
    {
        "mainnet" => ChainNetwork::Mainnet,
        _ => ChainNetwork::Testnet11,
    };
    let words = Mnemonic::generate()?;
    words.write_new_file(&path)?;
    let fingerprint = keys::facts_of(&words).fingerprint;
    let first = WalletKeys::from_mnemonic(&words, network, 1).puzzle_hashes()[0];
    drop(words);
    let line = format!(
        "New wallet: fingerprint {fingerprint}, words written to {} (mode 600, not printed).\nFund this address ({}): {}",
        path.display(),
        network.name(),
        super::address(first, network)
    );
    writeln!(io.out, "{line}").ok();
    Ok(())
}

struct Tool<'a, 'b> {
    o: Opts,
    io: &'a mut Io<'b>,
    network: ChainNetwork,
    consent: Option<MainnetConsent>,
    caps: SpendCaps,
}

impl<'a, 'b> Tool<'a, 'b> {
    fn new(o: Opts, io: &'a mut Io<'b>) -> Result<Self> {
        let network = o
            .network
            .clone()
            .or_else(|| (io.var)(NETWORK_VAR))
            .unwrap_or_default();
        let confirm = o
            .confirm
            .clone()
            .or_else(|| (io.var)(CONFIRM_VAR))
            .unwrap_or_default();
        let (net, consent) = spending::choose_network(&network, &confirm, "--network", "--mainnet-confirm (or HORIZON_MAINNET_CONFIRM)")?;
        let caps = SpendCaps::for_network(net).with_overrides("--", |name| {
            o.caps
                .iter()
                .rev()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
        })?;
        Ok(Self { o, io, network: net.into(), consent, caps })
    }

    fn net(&self) -> crate::Network {
        self.network.address_network()
    }

    fn say(&mut self, line: impl AsRef<str>) {
        writeln!(self.io.out, "{}", line.as_ref()).ok();
    }

    fn amount(&self, mojos: u64) -> String {
        spending::mojos_and_amount(mojos, self.net())
    }

    /// The network line every step starts with; mainnet loudly.
    fn banner(&mut self) {
        let line = format!("Network: {}", spending::network_label(self.net()));
        self.say(line);
        if self.network == ChainNetwork::Mainnet {
            let w = spending::mainnet_warning("This step");
            self.say(format!("!! {w}"));
        }
    }

    /// The wallet's words: the environment, else a file. Never printed.
    fn read_words(&mut self) -> Result<Mnemonic> {
        let (text, from) = if let Some(w) = (self.io.var)(WORDS_VAR).filter(|w| !w.trim().is_empty()) {
            (Secret::new(w), WORDS_VAR.to_owned())
        } else {
            let path = (self.io.var)(WORDS_FILE_VAR)
                .filter(|p| !p.trim().is_empty())
                .map(PathBuf::from)
                .or_else(|| self.o.words_file.clone())
                .ok_or_else(|| {
                    Error::new(format!(
                        "no wallet words: set {WORDS_VAR}, {WORDS_FILE_VAR} or --words-file (a throwaway wallet's words, never your savings)"
                    ))
                })?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = std::fs::metadata(&path)
                    && meta.permissions().mode() & 0o077 != 0
                {
                    let line = format!("warning: {} can be read by other users (chmod 600 it)", path.display());
                    self.say(line);
                }
            }
            let text = std::fs::read_to_string(&path).map_err(|e| Error::new(format!("the words file {}: {e}", path.display())))?;
            (Secret::new(text), path.display().to_string())
        };
        Mnemonic::parse(text.expose()).map_err(|problem| Error::new(format!("the words in {from}: {problem}")))
    }

    fn keys(&self, words: &Mnemonic) -> WalletKeys {
        WalletKeys::with_hardened(words, self.network, self.o.addresses, self.o.addresses)
    }

    fn address(&self, ph: Bytes32) -> String {
        super::address(ph, self.network)
    }

    /// A reader for `puzzle_hashes` (none: DID lookups only), synced.
    fn reader(&mut self, fingerprint: Option<u32>, puzzle_hashes: Vec<Bytes32>) -> Result<ChainReader> {
        let mut config = ChainConfig::chosen(self.net(), self.consent);
        config.caps = self.caps;
        config.puzzle_hashes = puzzle_hashes;
        config.peers = self
            .o
            .peers
            .clone()
            .or_else(|| (self.io.var)("HORIZON_CHAIN_PEERS"))
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(str::to_owned)
            .collect();
        let data = self
            .o
            .data
            .clone()
            .unwrap_or_else(|| PathBuf::from("chiacraft-wallet-test"));
        if let Some(f) = fingerprint {
            std::fs::create_dir_all(&data)?;
            config.cache_file = Some(data.join(format!("chain-{}-{f}.json", self.network)));
        }
        self.say(format!("Connecting to {} peers…", self.network));
        let reader = match self.io.dialer {
            Some(make) => ChainReader::start_with(config, make())?,
            None => ChainReader::start(config)?,
        };
        if !reader.wait_synced(self.io.wait) {
            let why = reader
                .status()
                .last_error
                .unwrap_or_else(|| "no peer answered".into());
            return Err(Error::new(format!("not synced with {} within {} s ({why})", self.network, self.io.wait.as_secs())));
        }
        let s = reader.status();
        self.say(format!("Synced at height {} with {} peer(s)", s.synced_height.unwrap_or(0), s.peers.len()));
        Ok(reader)
    }

    /// Show the plan and ask (unless `--yes`). No: nothing was spent.
    fn confirm(&mut self, plan: &[String]) -> Result<()> {
        self.say("This step will:");
        for line in plan {
            self.say(format!("  - {line}"));
        }
        if self.o.yes {
            self.say("(--yes: going ahead)");
            return Ok(());
        }
        let question = format!("Go ahead on {}? [y/N] ", self.network.name().to_uppercase());
        if (self.io.ask)(&question) { Ok(()) } else { Err(Error::new("cancelled: nothing was spent")) }
    }

    fn pushed(&mut self, push: &PushOutcome) -> Result<String> {
        match push {
            PushOutcome::Accepted { txid } | PushOutcome::Pending { txid } => Ok(hex32(*txid)),
            PushOutcome::Rejected { error, .. } => Err(Error::new(format!("the chain refused the spend: {error}"))),
        }
    }

    fn addresses(&mut self) -> Result<()> {
        let words = self.read_words()?;
        let count = self.o.count.unwrap_or(5).max(1);
        let facts = keys::facts_of(&words);
        let unhardened = WalletKeys::from_mnemonic(&words, self.network, count);
        let hardened: Vec<Bytes32> = keys::hardened_puzzle_hashes(&words, count)
            .into_iter()
            .map(Bytes32::from)
            .collect();
        drop(words);
        self.banner();
        self.say(format!("Account fingerprint {}", facts.fingerprint));
        for (i, ph) in unhardened.puzzle_hashes().iter().enumerate() {
            let a = self.address(*ph);
            self.say(format!("  {i:>2}  {a}{}", if i == 0 { "  (primary: fund this one)" } else { "" }));
        }
        for (i, ph) in hardened.iter().enumerate() {
            let a = self.address(*ph);
            self.say(format!("  {i:>2}' {a}  (hardened)"));
        }
        Ok(())
    }

    fn caps_command(&mut self) -> Result<()> {
        self.banner();
        let d = self.caps.describe();
        self.say(format!("Caps: {d}"));
        Ok(())
    }

    fn balance(&mut self) -> Result<()> {
        let words = self.read_words()?;
        let keys = self.keys(&words);
        let fingerprint = keys::facts_of(&words).fingerprint;
        drop(words);
        self.banner();
        let r = self.reader(Some(fingerprint), keys.puzzle_hashes())?;
        // The unspent XCH coins only: the cache also keeps spent ones (a send's parent) and
        // hinted ones (a DID), which the balance doesn't count either.
        let mut coins = r.cache().wallet.unspent_xch();
        coins.sort_by_key(|c| std::cmp::Reverse(c.amount));
        for c in &coins {
            let a = self.address(c.puzzle_hash);
            let amount = self.amount(c.amount);
            self.say(format!("  {amount} at {a} (coin {})", hex32(c.coin_id())));
        }
        let s = r.status();
        let total = self.amount(s.balance);
        let spendable = self.amount(r.spendable_balance());
        let primary = self.address(keys.primary());
        self.say(format!("Balance: {total} in {} coin(s); spendable {spendable}; primary address {primary}", s.coins));
        Ok(())
    }

    fn create_did(&mut self) -> Result<()> {
        let fee = self.o.fee.unwrap_or(DEFAULT_FEE);
        self.caps.check("max-did-fee", "the DID's fee", fee)?;
        let words = self.read_words()?;
        let keys = self.keys(&words);
        let fingerprint = keys::facts_of(&words).fingerprint;
        drop(words);
        self.banner();
        let r = self.reader(Some(fingerprint), keys.puzzle_hashes())?;
        let total = fee.saturating_add(1);
        let plan = vec![
            format!(
                "spend {} from this wallet ({}): 1 mojo becomes the DID, {} is the fee",
                self.amount(total),
                self.address(keys.primary()),
                self.amount(fee)
            ),
            format!("the wallet has {} spendable", self.amount(r.spendable_balance())),
        ];
        self.confirm(&plan)?;
        let made = r.create_did(&keys, fee, self.io.wait)?;
        let tx = self.pushed(&made.push)?;
        let did = did_string(made.launcher_id);
        self.say(format!("Pushed transaction {tx}: {did}"));
        self.say("Waiting for the chain to confirm it…");
        let record = r.wait_did(made.launcher_id, self.io.step, self.io.wait)?;
        let owner = self.address(made.owner);
        self.say(format!(
            "Confirmed at height {}: {did}, held by {owner} ({})",
            record.launched().unwrap_or(0),
            if keys.puzzle_hashes().iter().any(|p| record.owned_by(*p)) { "this wallet" } else { "NOT this wallet" }
        ));
        Ok(())
    }

    fn send(&mut self) -> Result<()> {
        let to_text = self
            .o
            .positional
            .first()
            .cloned()
            .ok_or_else(|| Error::new("send needs an ADDRESS"))?;
        let amount = self
            .o
            .amount
            .ok_or_else(|| Error::new("send needs --amount MOJOS"))?;
        let fee = self.o.fee.unwrap_or(DEFAULT_FEE);
        let prefix = match self.network {
            ChainNetwork::Mainnet => "xch1",
            ChainNetwork::Testnet11 => "txch1",
        };
        if !to_text.trim().starts_with(prefix) {
            return Err(Error::new(format!("{to_text} isn't a {} address (it should start with {prefix})", self.network.name())));
        }
        let to = super::parse_address(&to_text)?;
        self.caps.check_fee(fee)?;
        self.caps.check("max-send", "the payment", amount)?;
        let words = self.read_words()?;
        let keys = self.keys(&words);
        let fingerprint = keys::facts_of(&words).fingerprint;
        drop(words);
        self.banner();
        let r = self.reader(Some(fingerprint), keys.puzzle_hashes())?;
        let plan = vec![
            format!(
                "send {} from this wallet ({}) to {}, plus {} fee",
                self.amount(amount),
                self.address(keys.primary()),
                to_text.trim(),
                self.amount(fee)
            ),
            format!("the wallet has {} spendable", self.amount(r.spendable_balance())),
        ];
        self.confirm(&plan)?;
        let push = r.send(&keys, to, amount, fee, self.io.wait)?;
        let tx = self.pushed(&push)?;
        self.say(format!("Pushed transaction {tx}: {} to {}", self.amount(amount), to_text.trim()));
        Ok(())
    }

    fn make_offer(&mut self) -> Result<()> {
        let price = self
            .o
            .price
            .ok_or_else(|| Error::new("make-offer needs --price MOJOS"))?;
        let give = self.o.give.unwrap_or(1);
        let fee = self.o.fee.unwrap_or(0);
        let minutes = self.o.minutes.unwrap_or(60);
        self.caps.check_fee(fee)?;
        self.caps
            .check("max-offer-price", "the offer's price", price)?;
        self.caps
            .check("max-offer-price", "what the offer gives", give)?;
        let words = self.read_words()?;
        let keys = self.keys(&words);
        let fingerprint = keys::facts_of(&words).fingerprint;
        drop(words);
        self.banner();
        let r = self.reader(Some(fingerprint), keys.puzzle_hashes())?;
        let expires = offer::now_seconds() + minutes * 60;
        let plan = vec![
            format!(
                "make an offer that gives {} and asks {}, paid to {}",
                self.amount(give),
                self.amount(price),
                self.address(keys.primary())
            ),
            format!("lock the coin it gives until it is taken or expires in {minutes} minutes"),
            format!("pay {} fee now (nothing moves until someone takes it)", self.amount(fee)),
        ];
        self.confirm(&plan)?;
        let made = r.make_offer(MakeOffer { give: OfferGive::Xch(give), price, expires_at: Some(expires), fee }, &keys, self.io.wait)?;
        let path = self
            .o
            .out
            .clone()
            .unwrap_or_else(|| PathBuf::from(format!("offer-{}.offer", offer::now_seconds())));
        std::fs::write(&path, &made.text)?;
        self.say(format!(
            "Offer written to {} (coin {}); anyone can take it until it expires",
            path.display(),
            made.offered_coins.join(", ")
        ));
        if !self.o.wait {
            return Ok(());
        }
        self.say("Waiting for it to be taken and paid (Ctrl-C stops waiting; the offer stays open)…");
        let end = Instant::now() + Duration::from_secs(minutes * 60);
        loop {
            match r.offer_state(&made) {
                OfferState::Taken { height, paid: true } => {
                    let a = self.amount(price);
                    self.say(format!("Taken and paid at height {height}: {a} received"));
                    return Ok(());
                }
                OfferState::Expired => {
                    self.say("Expired untaken: nothing moved");
                    return Ok(());
                }
                _ if Instant::now() >= end => {
                    self.say("Still open; stopped waiting");
                    return Ok(());
                }
                _ => std::thread::sleep(self.io.step),
            }
        }
    }

    fn take_offer(&mut self) -> Result<()> {
        let source = self
            .o
            .positional
            .first()
            .cloned()
            .ok_or_else(|| Error::new("take-offer needs the offer's file (or its offer1… text)"))?;
        let text = if source.trim().starts_with("offer1") {
            source
        } else {
            std::fs::read_to_string(&source).map_err(|e| Error::new(format!("{source}: {e}")))?
        };
        let summary = offer::summarize(&text)?;
        let fee = self.o.fee.unwrap_or(0);
        let pays = summary.requested_xch.saturating_add(summary.royalties_xch);
        self.caps.check_fee(fee)?;
        self.caps
            .check("max-take", "taking the offer (its price and royalties)", pays)?;
        if !summary.requested_cats.is_empty() || !summary.requested_nfts.is_empty() {
            return Err(Error::new("the offer asks for CATs or NFTs: this wallet pays XCH only"));
        }
        let words = self.read_words()?;
        let keys = self.keys(&words);
        let fingerprint = keys::facts_of(&words).fingerprint;
        drop(words);
        self.banner();
        let r = self.reader(Some(fingerprint), keys.puzzle_hashes())?;
        let mut gets = vec![];
        if summary.offered_xch > 0 {
            gets.push(self.amount(summary.offered_xch));
        }
        gets.extend(summary.offered_nfts.iter().map(|n| format!("the NFT {n}")));
        gets.extend(
            summary
                .offered_cats
                .iter()
                .map(|(a, n)| format!("{n} of the CAT {a}")),
        );
        let plan = vec![
            format!(
                "pay {} from this wallet ({}): {} price{} + {} fee",
                self.amount(pays.saturating_add(fee)),
                self.address(keys.primary()),
                self.amount(summary.requested_xch),
                if summary.royalties_xch > 0 { format!(" + {} royalties", self.amount(summary.royalties_xch)) } else { String::new() },
                self.amount(fee)
            ),
            format!("receive {}", if gets.is_empty() { "nothing".to_owned() } else { gets.join(", ") }),
            "a payment can't be undone".into(),
        ];
        self.confirm(&plan)?;
        let taken = r.take_offer(&text, &keys, fee, self.io.wait)?;
        let tx = self.pushed(&taken.push)?;
        self.say(format!("Took the offer: transaction {tx}; it confirms in the next blocks (about a minute)"));
        Ok(())
    }

    fn did(&mut self) -> Result<()> {
        let id = self
            .o
            .positional
            .first()
            .cloned()
            .ok_or_else(|| Error::new("did needs a DID (did:chia:1… or its launcher id)"))?;
        let launcher = parse_did(&id)?;
        self.banner();
        let r = self.reader(None, Vec::new())?;
        let found = r.resolve_did(launcher, self.io.wait)?;
        let rec = &found.record;
        let owner = rec.owner().map(|p| self.address(p));
        self.say(format!(
            "{}: {}; launched at height {}, {} coin(s) in its history{}",
            rec.did(),
            if rec.melted {
                "melted".to_owned()
            } else if rec.is_eve() {
                "still eve (never spent since launch: a sponsored DID shows its owner after its first spend)".to_owned()
            } else {
                format!("held by {}", owner.unwrap_or_else(|| "an unknown owner".into()))
            },
            rec.launched().map_or("-".into(), |h| h.to_string()),
            rec.history.len(),
            if found.live { "" } else { " (cached: no peer)" }
        ));
        Ok(())
    }

    fn sponsor_request(&mut self) -> Result<()> {
        let url = self
            .o
            .positional
            .first()
            .cloned()
            .ok_or_else(|| Error::new("sponsor-request needs the sponsor's address (http://127.0.0.1:8080)"))?;
        let words = self.read_words()?;
        let keys = self.keys(&words);
        let fingerprint = keys::facts_of(&words).fingerprint;
        let signer = IdentitySigner::from_mnemonic(&words, 0);
        drop(words);
        self.banner();
        let address = self.address(keys.primary());
        let plan = vec![
            format!("send {url} your first address {address}, your identity key (public) and its signature of the request"),
            "spend nothing from this wallet: the sponsor pays the DID's mojo and fee".into(),
        ];
        self.confirm(&plan)?;
        let request = crate::sponsor::Request::sign(&signer, self.network.name(), &address, offer::now_seconds())?;
        drop(signer);
        let (status, body) = http_post(&url, crate::sponsor::PATH, &request.to_json())?;
        let reply = crate::sponsor::read_answer(status, &body)?;
        let launcher = Bytes32::from(reply.launcher_id()?);
        let did = did_string(launcher);
        if reply.status == crate::sponsor::EXISTING {
            self.say(format!("The sponsor already launched {did} for this identity"));
        } else {
            self.say(format!("The sponsor launched {did} ({})", reply.status));
        }
        if !self.o.wait {
            self.say("Check it with `did <id>` (or rerun with --wait)");
            return Ok(());
        }
        let r = self.reader(Some(fingerprint), keys.puzzle_hashes())?;
        self.say("Waiting for the chain to confirm it…");
        let record = r.wait_did(launcher, self.io.step, self.io.wait)?;
        let ours = keys.puzzle_hashes().iter().any(|p| record.owned_by(*p));
        self.say(format!(
            "On chain at height {}: {did}, {}",
            record.launched().unwrap_or(0),
            if ours { "held by this wallet's first address" } else { "NOT held by this wallet" }
        ));
        Ok(())
    }
}

/// `POST {base}{path}` with a JSON body over plain HTTP (a sponsor run on this computer or the
/// LAN; `https://` sponsors: use the game's own "Get a DID from the sponsor"). Status and body.
pub fn http_post(base: &str, path: &str, body: &str) -> Result<(u16, String)> {
    use std::io::Read;
    let base = base.trim().trim_end_matches('/');
    let rest = base
        .strip_prefix("http://")
        .ok_or_else(|| Error::new("the tool speaks plain http:// only (a sponsor on this computer: http://127.0.0.1:8080)"))?;
    let (host, prefix) = match rest.split_once('/') {
        Some((h, p)) => (h, format!("/{p}")),
        None => (rest, String::new()),
    };
    let addr = if host.contains(':') { host.to_owned() } else { format!("{host}:80") };
    let socket = std::net::ToSocketAddrs::to_socket_addrs(&addr)
        .map_err(|e| Error::new(format!("{addr}: {e}")))?
        .next()
        .ok_or_else(|| Error::new(format!("{addr}: no address")))?;
    let mut stream =
        std::net::TcpStream::connect_timeout(&socket, Duration::from_secs(10)).map_err(|e| Error::new(format!("{addr}: {e}")))?;
    stream.set_read_timeout(Some(Duration::from_secs(150))).ok();
    let request = format!(
        "POST {prefix}{path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(request.as_bytes())?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)?;
    let text = String::from_utf8_lossy(&raw).into_owned();
    let (head, payload) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| Error::new("the sponsor's answer has no body"))?;
    let status: u16 = head
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Error::new("the sponsor's answer has no status"))?;
    let chunked = head
        .lines()
        .any(|l| l.to_ascii_lowercase().starts_with("transfer-encoding:") && l.to_ascii_lowercase().contains("chunked"));
    let body = if chunked { dechunk(payload) } else { payload.to_owned() };
    Ok((status, body))
}

fn dechunk(mut s: &str) -> String {
    let mut out = String::new();
    while let Some((size, rest)) = s.split_once("\r\n") {
        let Ok(n) = usize::from_str_radix(size.trim(), 16) else {
            break;
        };
        if n == 0 || rest.len() < n {
            break;
        }
        out.push_str(&rest[..n]);
        s = rest[n..].trim_start_matches("\r\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments() {
        let a = |s: &str| s.split_whitespace().map(str::to_owned).collect::<Vec<_>>();
        let o = parse_args(&a("--network mainnet --max-fee 5 make-offer --price 7 --yes")).unwrap();
        assert_eq!(o.command, "make-offer");
        assert_eq!(o.price, Some(7));
        assert!(o.yes);
        assert_eq!(o.caps, vec![("max-fee".to_owned(), "5".to_owned())]);
        assert!(parse_args(&a("--words abandon balance")).is_err());
        assert!(parse_args(&a("--max-everything 5 balance")).is_err());
        assert!(parse_args(&a("--price x make-offer")).is_err());
        assert_eq!(dechunk("5\r\nhello\r\n3\r\n!!!\r\n0\r\n\r\n"), "hello!!!");
    }
}
