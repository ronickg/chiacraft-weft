//! Shared by the sponsor's tests: chia-sdk-test's peer simulator (never a real network), a
//! throwaway funding wallet, throwaway players, a scratch data folder, and iroh endpoints on
//! loopback only (no relay, no address lookup: nothing leaves the machine).
#![allow(dead_code)]

use chiacraft_sponsor::{Config, Sponsor, net, service::now_seconds};
use chiacraft_wallet::{
    IdentitySigner, Mnemonic,
    chain::{self, Bytes32, ChainNetwork, WalletKeys, sim::Sim},
    sponsor::Request,
};
use iroh::endpoint::presets;
use iroh::{Endpoint, EndpointAddr, RelayMode, SecretKey};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub const WAIT: Duration = Duration::from_secs(20);

/// A scratch folder for one test's data (removed first, so a rerun starts clean).
pub fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("chiacraft-sponsor-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Fast settings for tests: a short batch window, a small fee.
pub fn config(dir: &std::path::Path) -> Config {
    let mut c = Config::parse("").unwrap();
    c.data_dir = dir.to_owned();
    c.fee_per_did = 1_000;
    // A world DID's gift: twice the fee, as the default is (it was computed from the parsed fee).
    c.gift_mojos = 2 * c.fee_per_did;
    c.batch_window_ms = 200;
    c.max_wait_seconds = 2;
    c.per_endpoint_per_day = 100;
    // The proof of work and the hourly release have tests of their own.
    c.work_bits = 0;
    c.hourly_release = false;
    c
}

/// A sponsor on the simulator, funded with `mojos` (0: unfunded), synced.
pub fn sponsor(sim: &Sim, config: Config, words: &Mnemonic, mojos: u64) -> Sponsor {
    if mojos > 0 {
        let keys = WalletKeys::from_mnemonic(words, ChainNetwork::Testnet11, 1);
        sim.pay(keys.primary(), mojos).unwrap();
    }
    let s = Sponsor::start_with(config, words, sim.dialer()).unwrap();
    assert!(s.wait_synced(WAIT), "{:?}", s.status());
    s
}

/// A throwaway player: its identity signer, its first address and that address's puzzle hash.
pub struct Player {
    pub words: Mnemonic,
    pub signer: IdentitySigner,
    pub keys: WalletKeys,
    pub address: String,
}

impl Player {
    pub fn new() -> Self {
        let words = Mnemonic::generate().unwrap();
        let signer = IdentitySigner::from_mnemonic(&words, 0);
        let keys = WalletKeys::from_mnemonic(&words, ChainNetwork::Testnet11, 1);
        let address = chain::address(keys.primary(), ChainNetwork::Testnet11);
        Self { words, signer, keys, address }
    }
    /// The identity key, as the sponsor keeps it (hex).
    pub fn identity(&self) -> String {
        use chiacraft_wallet::Signer;
        self.signer.identity().hex()
    }
    pub fn p2(&self) -> Bytes32 {
        self.keys.primary()
    }
    pub fn request(&self) -> Request {
        self.request_at(now_seconds())
    }
    pub fn request_at(&self, time: u64) -> Request {
        Request::sign(&self.signer, ChainNetwork::Testnet11.name(), &self.address, time).unwrap()
    }
}

/// An asker, as the sponsor sees it: an endpoint id's text.
pub fn client(n: u8) -> String {
    SecretKey::from_bytes(&[n; 32]).public().to_string()
}

/// Wait until `f` holds (or the timeout).
pub fn until(mut f: impl FnMut() -> bool) -> bool {
    let end = std::time::Instant::now() + WAIT;
    while std::time::Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    f()
}

/// An endpoint on loopback only: no relay, no address lookup.
pub async fn local_endpoint(key: SecretKey, alpns: Vec<Vec<u8>>) -> Endpoint {
    Endpoint::builder(presets::Minimal)
        .relay_mode(RelayMode::Disabled)
        .secret_key(key)
        .alpns(alpns)
        .bind_addr("127.0.0.1:0")
        .unwrap()
        .bind()
        .await
        .unwrap()
}

/// A sponsor served over iroh on loopback (its key from its data folder, as the binary's), and a
/// runtime to ask it from.
pub struct Served {
    pub rt: tokio::runtime::Runtime,
    pub sponsor: Arc<Sponsor>,
    pub server: Endpoint,
    /// The sponsor's address with its loopback paths (no lookup to find it by id).
    pub addr: EndpointAddr,
}

impl Served {
    pub fn start(sponsor: Sponsor) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        let sponsor = Arc::new(sponsor);
        let key = net::load_key(&sponsor.config().data_dir).unwrap();
        let server = rt.block_on(local_endpoint(key, vec![chiacraft_sponsor::ALPN.to_vec()]));
        rt.spawn(net::serve(sponsor.clone(), server.clone()));
        let addr = server.addr();
        Self { rt, sponsor, server, addr }
    }

    /// A fresh asker (its own endpoint, as a game's).
    pub fn asker(&self) -> Endpoint {
        self.rt
            .block_on(local_endpoint(SecretKey::generate(), Vec::new()))
    }

    /// Ask for a DID from `ep`: the answer's status and body.
    pub fn ask(&self, ep: &Endpoint, request: &Request) -> (u16, serde_json::Value) {
        let (status, body) = self
            .rt
            .block_on(chiacraft_sponsor::ask(ep, self.addr.clone(), request))
            .unwrap();
        (status, serde_json::from_str(&body).unwrap())
    }

    /// Send `bytes` as they are from `ep`.
    pub fn send(&self, ep: &Endpoint, bytes: &[u8]) -> Result<(u16, serde_json::Value), String> {
        let (status, body) = self
            .rt
            .block_on(chiacraft_sponsor::client::exchange(ep, self.addr.clone(), bytes, WAIT))?;
        Ok((status, serde_json::from_str(&body).unwrap()))
    }

    pub fn status(&self, ep: &Endpoint) -> serde_json::Value {
        let (status, body) = self
            .rt
            .block_on(chiacraft_sponsor::client::status(ep, self.addr.clone()))
            .unwrap();
        assert_eq!(status, 200, "{body}");
        serde_json::from_str(&body).unwrap()
    }
}

impl Drop for Served {
    fn drop(&mut self) {
        let server = self.server.clone();
        self.rt.block_on(async move { server.close().await });
    }
}
