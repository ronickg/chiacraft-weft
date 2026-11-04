//! `sponsor.toml`: what the sponsor serves, where it keeps its data, its budgets and limits. The
//! funding wallet's words are never in it: they come from `CHIACRAFT_SPONSOR_WORDS` or a file named
//! by `CHIACRAFT_SPONSOR_WORDS_FILE` (or `words_file` here; the old `HORIZON_SPONSOR_*` names are
//! read too), are read once at start, and are never logged ([`load_words`]).
//!
//! **Mainnet** (I12) is an explicit opt-in: `network = "mainnet"` **and** `allow_mainnet = true`;
//! without the second line the sponsor refuses to start. On mainnet the budget fields default to
//! small values ([`MAINNET_DEFAULTS`]) and `fee_per_did` must stay within `max_fee_per_did`.
use chiacraft_wallet::{
    Mnemonic, Secret,
    chain::ChainNetwork,
    spending::{self, MainnetConsent, SpendCaps},
};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// The budget fields' defaults per network: fee per DID, daily budget, DIDs per asking endpoint
/// per day, DIDs per bundle, the cap on the fee per DID.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkDefaults {
    pub fee_per_did: u64,
    pub daily_budget_mojos: u64,
    pub per_endpoint_per_day: u32,
    pub max_batch: usize,
    pub max_fee_per_did: u64,
}

/// testnet11: test coins; a fee that gets through its busy mempool.
pub const TESTNET11_DEFAULTS: NetworkDefaults = NetworkDefaults {
    fee_per_did: 5_000_000,
    daily_budget_mojos: 1_000_000_000,
    per_endpoint_per_day: 3,
    max_batch: 25,
    max_fee_per_did: u64::MAX,
};

/// Mainnet: real XCH, so small: 0.000001 XCH fee per DID, 0.00005 XCH a day (about 49 DIDs),
/// one DID per endpoint a day, 5 per bundle; a fee per DID above 0.0001 XCH is refused.
pub const MAINNET_DEFAULTS: NetworkDefaults = NetworkDefaults {
    fee_per_did: 1_000_000,
    daily_budget_mojos: 50_000_000,
    per_endpoint_per_day: 1,
    max_batch: 5,
    max_fee_per_did: 100_000_000,
};

/// The service's settings. Every field has a default, so an empty file is a testnet11 sponsor
/// keeping its data in `sponsor-data`.
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "Raw")]
pub struct Config {
    /// `testnet11` (the default) or `mainnet` (needs `allow_mainnet = true`).
    pub network: String,
    /// The second setting mainnet needs: the operator knows the sponsor spends real XCH.
    pub allow_mainnet: bool,
    /// The request log (`sponsor-log.jsonl`), the chain cache and the endpoint's key
    /// (`sponsor.key`) live here.
    pub data_dir: PathBuf,
    /// A file holding the funding wallet's words (or `CHIACRAFT_SPONSOR_WORDS[_FILE]`).
    pub words_file: Option<PathBuf>,
    /// How many of the funding wallet's addresses the reader follows (its coins can be at any).
    pub addresses: u32,
    /// Full nodes to use instead of the DNS introducers (`host:port`; `ws://127.0.0.1:port` for a
    /// local simulator). `CHIACRAFT_CHAIN_PEERS` (or `HORIZON_CHAIN_PEERS`; comma-separated) is
    /// added to these.
    pub peers: Vec<String>,
    /// The fee paid per DID, in mojos (a batch of n pays n times this). testnet11's mempool is
    /// busy with dust, and a spend with no fee can wait most of an hour.
    pub fee_per_did: u64,
    /// The most the sponsor spends in one UTC day, in mojos: a mojo per DID plus its fee.
    pub daily_budget_mojos: u64,
    /// DIDs per asking endpoint (iroh endpoint id) per UTC day. Endpoint ids cost nothing to
    /// make, so this only slows a careless client: the identity limit and the daily budget are
    /// what hold.
    pub per_endpoint_per_day: u32,
    /// How far a request's `time` may be from the sponsor's clock, in seconds.
    pub max_skew_seconds: u64,
    /// Requests arriving within this many milliseconds of the first one go in one bundle.
    pub batch_window_ms: u64,
    /// The most DIDs in one bundle.
    pub max_batch: usize,
    /// The most requests waiting at once (more are refused with 429).
    pub max_queue: usize,
    /// How long a request waits for a free coin (all of the sponsor's coins are in unconfirmed
    /// spends) before it is refused with 503.
    pub max_wait_seconds: u64,
    /// A launch not confirmed after this many seconds is dropped: its identity may ask again and
    /// its cost goes back to the day's budget.
    pub drop_after_seconds: u64,
    /// Zero bits of proof of work a request must carry (`chiacraft_wallet::sponsor::work_text`):
    /// about 2^bits hashes of the asker's CPU per DID. 0: none.
    pub work_bits: u32,
    /// The day's budget is released hour by hour (by the end of hour h of the UTC day, (h + 1) /
    /// 24 of it), so a burst at midnight can't spend the whole day's.
    pub hourly_release: bool,
    /// The most `fee_per_did` may be (a hard cap: a higher fee refuses to start, and the chain
    /// reader refuses a bundle paying more per DID). Unlimited on testnet11 by default.
    pub max_fee_per_did: u64,
}

/// The file as written: the budget fields left out take their network's defaults.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Raw {
    network: String,
    allow_mainnet: bool,
    data_dir: PathBuf,
    words_file: Option<PathBuf>,
    addresses: u32,
    peers: Vec<String>,
    fee_per_did: Option<u64>,
    daily_budget_mojos: Option<u64>,
    /// Horizon's name for it is read too.
    #[serde(alias = "per_ip_per_day")]
    per_endpoint_per_day: Option<u32>,
    max_skew_seconds: u64,
    batch_window_ms: u64,
    max_batch: Option<usize>,
    max_queue: usize,
    max_wait_seconds: u64,
    drop_after_seconds: u64,
    max_fee_per_did: Option<u64>,
    work_bits: u32,
    hourly_release: bool,
}

impl Default for Raw {
    fn default() -> Self {
        Self {
            network: "testnet11".into(),
            allow_mainnet: false,
            data_dir: PathBuf::from("sponsor-data"),
            words_file: None,
            addresses: 20,
            peers: Vec::new(),
            fee_per_did: None,
            daily_budget_mojos: None,
            per_endpoint_per_day: None,
            max_skew_seconds: 300,
            batch_window_ms: 2_000,
            max_batch: None,
            max_queue: 200,
            max_wait_seconds: 120,
            work_bits: 22,
            hourly_release: true,
            drop_after_seconds: 3_600,
            max_fee_per_did: None,
        }
    }
}

impl TryFrom<Raw> for Config {
    type Error = String;
    fn try_from(r: Raw) -> Result<Self, String> {
        let d = match ChainNetwork::parse(&r.network) {
            Some(ChainNetwork::Mainnet) => MAINNET_DEFAULTS,
            _ => TESTNET11_DEFAULTS,
        };
        let config = Self {
            network: r.network,
            allow_mainnet: r.allow_mainnet,
            data_dir: r.data_dir,
            words_file: r.words_file,
            addresses: r.addresses,
            peers: r.peers,
            fee_per_did: r.fee_per_did.unwrap_or(d.fee_per_did),
            daily_budget_mojos: r.daily_budget_mojos.unwrap_or(d.daily_budget_mojos),
            per_endpoint_per_day: r.per_endpoint_per_day.unwrap_or(d.per_endpoint_per_day),
            max_skew_seconds: r.max_skew_seconds,
            batch_window_ms: r.batch_window_ms,
            max_batch: r.max_batch.unwrap_or(d.max_batch),
            max_queue: r.max_queue,
            max_wait_seconds: r.max_wait_seconds,
            drop_after_seconds: r.drop_after_seconds,
            max_fee_per_did: r.max_fee_per_did.unwrap_or(d.max_fee_per_did),
            work_bits: r.work_bits.min(32),
            hourly_release: r.hourly_release,
        };
        config.check()?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::try_from(Raw::default()).expect("the testnet11 defaults are valid")
    }
}

impl Config {
    /// Read and check a config file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Parse and check a config text.
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|e| e.message().trim().to_owned())
    }

    /// Everything that refuses to start: the network and its confirmation, the fee within its
    /// cap, the batch size, the addresses.
    pub fn check(&self) -> Result<(), String> {
        let network = self.chain_network()?;
        if self.max_batch == 0 || self.max_batch > chiacraft_wallet::chain::sponsor::MAX_BATCH {
            return Err(format!("max_batch must be 1 to {}", chiacraft_wallet::chain::sponsor::MAX_BATCH));
        }
        if self.addresses == 0 {
            return Err("addresses must be at least 1".into());
        }
        if self.fee_per_did > self.max_fee_per_did {
            return Err(format!(
                "fee_per_did = {} is above max_fee_per_did = {} on {network}: refused",
                spending::mojos_and_amount(self.fee_per_did, network.address_network()),
                spending::mojos_and_amount(self.max_fee_per_did, network.address_network())
            ));
        }
        Ok(())
    }

    /// The network served: testnet11 (the default), or mainnet with `allow_mainnet = true`.
    pub fn chain_network(&self) -> Result<ChainNetwork, String> {
        match ChainNetwork::parse(&self.network) {
            Some(ChainNetwork::Testnet11) => Ok(ChainNetwork::Testnet11),
            Some(ChainNetwork::Mainnet) if self.allow_mainnet => Ok(ChainNetwork::Mainnet),
            Some(ChainNetwork::Mainnet) => {
                Err("network = \"mainnet\" spends real XCH: it also needs allow_mainnet = true; refusing to start".into())
            }
            None => Err(format!("network {:?}: \"testnet11\" (the default) or \"mainnet\"", self.network)),
        }
    }

    /// The chain reader's mainnet consent (`allow_mainnet = true` on mainnet), else none.
    pub fn consent(&self) -> Option<MainnetConsent> {
        (self.chain_network().ok() == Some(ChainNetwork::Mainnet))
            .then(|| MainnetConsent::confirm(spending::MAINNET_CONFIRM, "allow_mainnet").ok())
            .flatten()
    }

    /// The caps the sponsor's chain reader holds every spend to: the network's, with the DID fee
    /// capped at `max_fee_per_did`.
    pub fn caps(&self) -> Result<SpendCaps, String> {
        let network = self.chain_network()?;
        let mut caps = SpendCaps::for_network(network.address_network());
        caps.max_did_fee = self.max_fee_per_did;
        Ok(caps)
    }

    /// The cost of one DID to the budget: a mojo plus its fee.
    pub fn cost_per_did(&self) -> u64 {
        self.fee_per_did.saturating_add(1)
    }

    /// The peers to dial: the config's, then `CHIACRAFT_CHAIN_PEERS`' (else `HORIZON_CHAIN_PEERS`').
    pub fn all_peers(&self) -> Vec<String> {
        let mut out = self.peers.clone();
        if let Some(env) = env_var("CHAIN_PEERS").and_then(|v| v.into_string().ok()) {
            out.extend(
                env.split(',')
                    .map(str::trim)
                    .filter(|p| !p.is_empty())
                    .map(str::to_owned),
            );
        }
        out
    }
}

/// An environment variable by its chiacraft name (`CHIACRAFT_<name>`), else by Horizon's
/// (`HORIZON_<name>`), with the name it was found under.
pub fn env_named(name: &str) -> Option<(String, std::ffi::OsString)> {
    ["CHIACRAFT_", "HORIZON_"].iter().find_map(|prefix| {
        let var = format!("{prefix}{name}");
        std::env::var_os(&var).map(|value| (var, value))
    })
}

/// [`env_named`]'s value.
pub fn env_var(name: &str) -> Option<std::ffi::OsString> {
    env_named(name).map(|(_, value)| value)
}

/// The funding wallet's words: `CHIACRAFT_SPONSOR_WORDS`, else the file named by
/// `CHIACRAFT_SPONSOR_WORDS_FILE`, else `config.words_file` (`HORIZON_SPONSOR_WORDS[_FILE]` are
/// read when the chiacraft names aren't set). The text is held in [`Secret`] and dropped once
/// parsed; errors say where the words were looked for, never what they are.
pub fn load_words(config: &Config) -> Result<Mnemonic, String> {
    let (text, from): (Secret<String>, String) = if let Some((var, words)) = env_named("SPONSOR_WORDS") {
        (Secret::new(words.into_string().unwrap_or_default()), var)
    } else {
        let path = env_var("SPONSOR_WORDS_FILE")
            .map(PathBuf::from)
            .or_else(|| config.words_file.clone())
            .ok_or_else(|| "no funding wallet: set CHIACRAFT_SPONSOR_WORDS, CHIACRAFT_SPONSOR_WORDS_FILE or words_file".to_owned())?;
        let text = std::fs::read_to_string(&path).map_err(|e| format!("the words file {}: {e}", path.display()))?;
        (Secret::new(text), path.display().to_string())
    };
    Mnemonic::parse(text.expose()).map_err(|problem| format!("the words in {from}: {problem}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_refusals() {
        let c = Config::parse("").unwrap();
        assert_eq!(c.chain_network().unwrap(), ChainNetwork::Testnet11);
        assert_eq!(c.cost_per_did(), 5_000_001);
        assert!(c.consent().is_none());
        let e = Config::parse("network = \"mainnet\"").unwrap_err();
        assert!(e.contains("allow_mainnet = true"), "{e}");
        assert!(Config::parse("network = \"mainnet\"\nallow_mainnet = false").is_err());
        assert!(Config::parse("network = \"testnet10\"").is_err());
        assert!(Config::parse("max_batch = 0").is_err());
        assert!(Config::parse("nonsense = 1").is_err());
        let c = Config::parse("fee_per_did = 0\nper_ip_per_day = 9").unwrap();
        assert_eq!((c.cost_per_did(), c.per_endpoint_per_day), (1, 9));
        // Horizon's HTTP settings are gone.
        assert!(Config::parse("listen = \"127.0.0.1:8080\"").is_err());
    }

    #[test]
    fn mainnet_needs_allow_mainnet_and_takes_small_defaults() {
        let c = Config::parse("network = \"mainnet\"\nallow_mainnet = true").unwrap();
        assert_eq!(c.chain_network().unwrap(), ChainNetwork::Mainnet);
        assert!(c.consent().is_some());
        assert_eq!(
            (c.fee_per_did, c.daily_budget_mojos, c.per_endpoint_per_day, c.max_batch, c.max_fee_per_did),
            (1_000_000, 50_000_000, 1, 5, 100_000_000)
        );
        assert_eq!(c.caps().unwrap().max_did_fee, 100_000_000);
        // Set explicitly, a field keeps its value; a fee above its cap refuses to start.
        let c = Config::parse("network = \"mainnet\"\nallow_mainnet = true\ndaily_budget_mojos = 7").unwrap();
        assert_eq!(c.daily_budget_mojos, 7);
        let e = Config::parse("network = \"mainnet\"\nallow_mainnet = true\nfee_per_did = 100000001").unwrap_err();
        assert!(e.contains("max_fee_per_did"), "{e}");
        assert!(e.contains("0.0001 XCH"), "{e}");
        let e = Config::parse("fee_per_did = 11\nmax_fee_per_did = 10").unwrap_err();
        assert!(e.contains("max_fee_per_did"), "{e}");
    }

    #[test]
    fn the_example_file_is_the_defaults() {
        let example = Config::parse(include_str!("../sponsor.example.toml")).unwrap();
        assert_eq!(format!("{example:?}"), format!("{:?}", Config::default()));
    }
}
