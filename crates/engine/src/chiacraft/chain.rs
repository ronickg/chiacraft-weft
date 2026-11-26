//! The chain in the game (`notes/PLAN.md`, phase 4, slice 4): one reader of the Chia network the
//! setting `core:chain` names, started the first time something needs it (getting, linking or
//! checking a DID) and kept while the game runs. Never in the game loop: callers wait on it on a
//! thread of their own.
//!
//! `off` turns it off; `testnet11` is the default while we test; `mainnet` is refused in this
//! build (real XCH: only after Ronald's yes, and then with the player's confirmation). The cache
//! (DIDs read, the account's coins) is `<run dir>/chain/<network>.json`, so answers survive a
//! restart and a world can check a player it has seen while no peer answers.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chiacraft_wallet::chain::{self, Bytes32, ChainConfig, ChainNetwork, ChainReader, DidRecord};

static READER: parking_lot::Mutex<Option<(ChainNetwork, Arc<ChainReader>)>> = parking_lot::Mutex::new(None);

/// Where and how to read the chain, set at start: the reader starts the first time [`get`] is
/// asked (a world checking a joining player's DID, the Account screen).
static CONFIG: parking_lot::Mutex<Option<(std::path::PathBuf, String, Vec<String>)>> = parking_lot::Mutex::new(None);

pub fn configure(run_dir: &Path, choice: &str, peers: Vec<String>) {
    *CONFIG.lock() = Some((run_dir.to_path_buf(), choice.to_string(), peers));
}

/// The reader as configured, started now if need be (never from an async task).
pub fn get() -> Result<Arc<ChainReader>, String> {
    let (dir, choice, peers) = CONFIG
        .lock()
        .clone()
        .ok_or("the chain isn't set up in this process")?;
    start(&dir, &choice, peers)
}

/// Blocks a day on Chia (one every 18.75 s on average).
pub const BLOCKS_A_DAY: u32 = 4608;

/// How long a DID lookup may take before the cache answers instead.
pub const LOOKUP: Duration = Duration::from_secs(20);

/// The network the setting names, or why there is none.
pub fn network(choice: &str) -> Result<ChainNetwork, String> {
    match choice.trim() {
        "" | "off" => Err("the chain is off (setting core:chain)".into()),
        "mainnet" => Err("mainnet is off in this build: DIDs are on testnet11 while we test".into()),
        other => ChainNetwork::parse(other).ok_or_else(|| format!("no chain called '{other}' (core:chain: off or testnet11)")),
    }
}

/// Whether the game may spend coins on `network` (the daily anchor, a world DID's listing):
/// testnet11 only. Real XCH needs Ronald's own yes; until then this refuses mainnet even if a
/// reader were somehow on it (the setting and the wallet's own mainnet gate refuse first).
pub fn may_spend(network: ChainNetwork) -> Result<(), String> {
    match network {
        ChainNetwork::Testnet11 => Ok(()),
        other => Err(format!("this build spends coins only on testnet11, not {other}")),
    }
}

/// The game's reader, started now if it isn't running (or runs on another network). Blocks
/// briefly; never call it from an async task (the reader runs its own runtime).
pub fn reader(settings: &crate::settings::Settings) -> Result<Arc<ChainReader>, String> {
    start(&settings.run_dir, &crate::settings::chain(), crate::settings::chain_peers())
}

/// [`reader`] from its parts (a dedicated server, tests).
pub fn start(run_dir: &Path, choice: &str, peers: Vec<String>) -> Result<Arc<ChainReader>, String> {
    if tokio::runtime::Handle::try_current().is_ok() {
        return Err("the chain reader was asked for from an async task".into());
    }
    let network = network(choice)?;
    let mut held = READER.lock();
    if let Some((n, r)) = held.as_ref()
        && *n == network
    {
        return Ok(r.clone());
    }
    let mut config = ChainConfig::new(network);
    config.peers = peers;
    config.cache_file = Some(
        run_dir
            .join("chain")
            .join(format!("{}.json", network.name())),
    );
    // The account's first addresses, both branches: where its DIDs are held (a sponsor launches
    // to the first).
    if let Some(s) = chiacraft_account::current() {
        config.puzzle_hashes = s.puzzle_hashes(20).into_iter().map(Bytes32::from).collect();
    }
    if let Some(dir) = config.cache_file.as_ref().and_then(|f| f.parent()) {
        let _ = std::fs::create_dir_all(dir);
    }
    let reader = Arc::new(ChainReader::start(config).map_err(|e| e.to_string())?);
    log::info!("reading the {network} chain");
    *held = Some((network, reader.clone()));
    Ok(reader)
}

/// The running reader, if one was started (no starting).
pub fn running() -> Option<Arc<ChainReader>> {
    READER.lock().as_ref().map(|(_, r)| r.clone())
}

/// A DID read now (continuing from the cache), or the cache's record while no peer answers:
/// the record and whether it was read live.
pub fn did(reader: &ChainReader, did: &str) -> Result<(DidRecord, bool), String> {
    let launcher = chain::parse_did(did).map_err(|e| e.to_string())?;
    let found = reader
        .resolve_did(launcher, LOOKUP)
        .map_err(|e| e.to_string())?;
    Ok((found.record, found.live))
}

/// What the reader is doing, for the Account screen: (network, phase, peak height, peers).
pub fn status() -> Option<(String, String, Option<u32>, usize)> {
    let r = running()?;
    let s = r.status();
    Some((r.network().name().to_string(), format!("{:?}", s.phase).to_lowercase(), s.peak, s.peers.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_setting_names_the_network() {
        assert_eq!(network("testnet11"), Ok(ChainNetwork::Testnet11));
        assert_eq!(network(" testnet11 "), Ok(ChainNetwork::Testnet11));
        assert!(network("off").is_err());
        assert!(network("").is_err());
        assert!(network("mainnet").unwrap_err().contains("mainnet is off"));
        assert!(network("devnet").is_err());
        assert!(may_spend(ChainNetwork::Testnet11).is_ok());
        assert!(may_spend(ChainNetwork::Mainnet).is_err());
    }
}
