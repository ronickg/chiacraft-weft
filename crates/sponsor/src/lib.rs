//! chiacraft's DID sponsor: a small service that launches a DID straight to a new player's
//! address, paid from the sponsor's own wallet, so a player with no coins can have one. The player
//! signs nothing but the request (with the game's identity key); the sponsor never holds the DID:
//! from its launch, its puzzle names the player's address as the owner. testnet11 by default
//! (mainnet only opted in), with a daily budget; anyone (a server for its own players) can run one.
//! It is reached over iroh, by its endpoint id, with the ALPN [`ALPN`] (`chiacraft/sponsor/1`).
//!
//! Always built (the game's side, light: iroh and the wallet without `chain`):
//! - [`client`]: [`ask`] a sponsor for a DID, or for its [`status`](client::status).
//!
//! With the feature `service` (the sponsor itself and the `chiacraft-sponsor` binary):
//! - `config`: `sponsor.toml` and the funding wallet's words (an env var or a file).
//! - `store`: the request log, replayed at start.
//! - `service`: checks, the queue, batching, launching, confirmations.
//! - `net`: the iroh endpoint, its key kept in the data folder, and the protocol's serving side.
//!
//! Keys, coin selection, the spend and the chain reader are `chiacraft-wallet`'s (`chain`
//! feature; the eve-DID bundle is `chiacraft_wallet::chain::sponsor`); the request and the answer
//! are its `sponsor` module, so the game and the sponsor agree on one text. The only wallet key
//! the sponsor holds is its own funding wallet's, inside `chiacraft_wallet::Secret`; nothing logs
//! it.
#![deny(unsafe_code)]
pub mod client;
#[cfg(feature = "service")]
pub mod config;
#[cfg(feature = "service")]
pub mod net;
#[cfg(feature = "service")]
pub mod service;
#[cfg(feature = "service")]
pub mod store;

pub use client::{ALPN, STATUS, ask};
#[cfg(feature = "service")]
pub use config::{Config, load_words};
#[cfg(feature = "service")]
pub use service::{DidReply, DidState, Refusal, Sponsor, StatusReply};
