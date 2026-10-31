//! A local Chia chain for the checks (`tools/chiacraft/admission_check.py`): chia-sdk-test's peer
//! simulator in this process, reachable at the `ws://127.0.0.1:<port>` it prints (servers and
//! games take it as `HORIZON_CHAIN_PEERS`). Never a real network, never real funds.
//!
//!     cargo run -p chiacraft-wallet --features sim --example chain-sim
//!
//! Prints `SIM ws://127.0.0.1:<port>`, then answers one line per command on standard input:
//!
//! - `fund <txch1… address | 64 hex puzzle hash> <mojos>` → `OK <coin id>` (paid in a transaction,
//!   so readers already following the address see it)
//! - `did <public test words…>` → `OK <launcher id> <did:chia:1…>` (a DID owned by the words'
//!   first address, funded from nothing)
//! - `nft <public test words…>` → `OK <launcher id>`
//! - `balance <address>` → `OK <mojos>`
//! - `sponsor <address>` → `OK <launcher id> <did:chia:1…>` (what a DID sponsor does: an eve DID
//!   launched straight to the address, identity row I8)
//! - `quit`
//!
//! Errors answer `ERR <why>`.
use chiacraft_wallet::{
    Mnemonic,
    chain::{ChainNetwork, WalletKeys, did_string, hex32, parse_address, sim::Sim},
};
use std::io::{BufRead, Write};

fn keys_of(words: &[&str]) -> Result<WalletKeys, String> {
    let m = Mnemonic::parse(&words.join(" ")).map_err(|e| format!("{e:?}"))?;
    Ok(WalletKeys::from_mnemonic(&m, ChainNetwork::Testnet11, 1))
}

fn run(sim: &Sim, line: &str) -> Result<String, String> {
    let w: Vec<&str> = line.split_whitespace().collect();
    match w.as_slice() {
        ["fund", addr, mojos] => {
            let ph = parse_address(addr).map_err(|e| e.to_string())?;
            let amount: u64 = mojos.parse().map_err(|_| "not a number".to_owned())?;
            let coin = sim.pay(ph, amount).map_err(|e| e.to_string())?;
            Ok(hex32(coin.coin_id()))
        }
        ["balance", addr] => {
            let ph = parse_address(addr).map_err(|e| e.to_string())?;
            Ok(sim.balance(ph).to_string())
        }
        ["sponsor", addr] => {
            let ph = parse_address(addr).map_err(|e| e.to_string())?;
            let id = sim.sponsor_did(ph).map_err(|e| e.to_string())?;
            Ok(format!("{} {}", hex32(id), did_string(id)))
        }
        ["did", words @ ..] => {
            let keys = keys_of(words)?;
            sim.fund(keys.primary(), 10);
            let id = sim.create_did(&keys).map_err(|e| e.to_string())?;
            Ok(format!("{} {}", hex32(id), did_string(id)))
        }
        ["nft", words @ ..] => {
            let keys = keys_of(words)?;
            sim.fund(keys.primary(), 10);
            let id = sim.mint_nft(&keys).map_err(|e| e.to_string())?;
            Ok(hex32(id))
        }
        _ => Err(format!("unknown command {line:?}")),
    }
}

fn main() {
    let sim = Sim::start().expect("the simulator starts");
    let address = sim.address().expect("the simulator's address");
    println!("SIM {address}");
    let _ = std::io::stdout().flush();
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let line = line.trim();
        if line == "quit" {
            break;
        }
        if line.is_empty() {
            continue;
        }
        match run(&sim, line) {
            Ok(out) => println!("OK {out}"),
            Err(e) => println!("ERR {e}"),
        }
        let _ = std::io::stdout().flush();
    }
}
