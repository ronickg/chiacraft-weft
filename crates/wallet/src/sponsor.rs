//! The DID sponsor contract (identity row I8 is the client, I9 the service; research 2.2 (c)): a
//! player with no coins asks a sponsor to launch a DID straight to the player's own address,
//! which the sponsor then never controls. No network here: this module builds and checks the
//! request and reads the answer, so the game and the sponsor service agree on one text.
//!
//! ```text
//! POST {sponsor}/v1/did
//! { "network": "testnet11", "address": "txch1…", "identity": "<identity key hex>",
//!   "signature": "<identity key's signature of the text>", "time": <unix seconds> }
//! → 200 { "did": "did:chia:1…", "launcher": "<hex>", "coin": "<hex>", "status": "pending" }
//! → 400/403/429 { "error": "…" }
//! ```
//!
//! A request also carries `work`, a nonce that makes the SHA-256 of [`work_text`] start with as
//! many zero bits as the sponsor asks for (its status says how many): identities cost nothing to
//! make, so each DID asked for costs some seconds of CPU instead (research round 4, P7).
//!
//! The signed text is `chiacraft sponsor v1 <address> <unix seconds>`, signed by the account's
//! identity key as a Chia signed message (CHIP-0002, [`crate::sign`]). The identity key names who
//! asked (a sponsor's daily budget per identity); the address is where the DID goes. Nothing in a
//! request is secret.
//!
//! **World DIDs** (plan phase 4, slice 7): a request may carry `world` ([`WorldAsk`]: the owner's
//! DID, the world's key and its generation). The sponsor then launches a world DID to the address
//! (the owner's), its metadata naming the owner's DID and the world's key and generation, after
//! checking on chain that the owner's DID is held by that address. The identity key then signs
//! [`world_text`], `chiacraft sponsor world v1 <address> <time> <owner_did> <key> <generation>`,
//! instead of [`text`]; the proof of work is the same. A request without `world` is a player's,
//! exactly as before.
use crate::{Error, Result, keys::IdentityKey, sign::Signer};
use serde::{Deserialize, Serialize};

/// The path under the sponsor's base address.
pub const PATH: &str = "/v1/did";
/// How far a request's time may be from the sponsor's clock (seconds) before it is refused.
pub const MAX_SKEW: u64 = 300;

/// The text the identity key signs.
pub fn text(address: &str, time: u64) -> String {
    format!("chiacraft sponsor v1 {address} {time}")
}

/// The text the identity key signs for a world DID: the player's text's fields, then the world's
/// (each exactly as sent).
pub fn world_text(address: &str, time: u64, world: &WorldAsk) -> String {
    format!("chiacraft sponsor world v1 {address} {time} {} {} {}", world.owner_did, world.key, world.generation)
}

/// What the proof of work hashes: the request's address, time and identity, and the nonce.
pub fn work_text(address: &str, time: u64, identity: &str, nonce: u64) -> String {
    format!("chiacraft sponsor work v1 {address} {time} {identity} {nonce}")
}

/// The zero bits a SHA-256 starts with.
fn zero_bits(hash: &[u8]) -> u32 {
    let mut n = 0;
    for b in hash {
        if *b == 0 {
            n += 8;
        } else {
            return n + b.leading_zeros();
        }
    }
    n
}

/// What a world DID's request asks for: the metadata the sponsor launches it with.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldAsk {
    /// The owner's DID: its launcher id, 64 hex digits. It must be held by the request's address.
    pub owner_did: String,
    /// The world's Ed25519 public key as iroh writes it: its 32 bytes in hex (64 digits).
    pub key: String,
    /// The key's generation (0 for a world's first key, one more per rotation).
    pub generation: u32,
}

/// The request's JSON body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub network: String,
    pub address: String,
    pub identity: String,
    pub signature: String,
    pub time: u64,
    /// The proof of work's nonce ([`work_text`]).
    #[serde(default)]
    pub work: u64,
    /// A world DID's ask; None (left out of the JSON): a player's DID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world: Option<WorldAsk>,
}

impl Request {
    /// A request for a DID at `address` on `network`, signed by the account's identity key.
    pub fn sign(signer: &dyn Signer, network: &str, address: &str, time: u64) -> Result<Self> {
        let signature = signer.sign(&text(address, time))?;
        Ok(Self {
            network: network.to_owned(),
            address: address.to_owned(),
            identity: signer.identity().hex(),
            signature: crate::keys::hex(&signature),
            time,
            work: 0,
            world: None,
        })
    }
    /// A request for a world DID at `address` (the owner's) on `network`, signed by the account's
    /// identity key over [`world_text`].
    pub fn sign_world(signer: &dyn Signer, network: &str, address: &str, time: u64, world: WorldAsk) -> Result<Self> {
        let signature = signer.sign(&world_text(address, time, &world))?;
        Ok(Self {
            network: network.to_owned(),
            address: address.to_owned(),
            identity: signer.identity().hex(),
            signature: crate::keys::hex(&signature),
            time,
            work: 0,
            world: Some(world),
        })
    }
    /// The text the identity key signed: [`world_text`] for a world DID, else [`text`].
    pub fn signed_text(&self) -> String {
        match &self.world {
            Some(world) => world_text(&self.address, self.time, world),
            None => text(&self.address, self.time),
        }
    }
    /// The zero bits this request's proof of work reaches.
    pub fn work_bits(&self) -> u32 {
        use sha2::{Digest, Sha256};
        zero_bits(&Sha256::digest(work_text(&self.address, self.time, &self.identity, self.work).as_bytes()))
    }
    /// Find a nonce reaching `bits` zero bits (about 2^bits hashes: 22 bits is a second or two).
    /// Bits above 40 are refused, so a sponsor can't make a client work forever.
    pub fn do_work(&mut self, bits: u32) -> Result<()> {
        if bits > 40 {
            return Err(Error::new(format!("the sponsor asks for {bits} bits of work, too many")));
        }
        self.work = 0;
        while self.work_bits() < bits {
            self.work += 1;
        }
        Ok(())
    }
    /// Whether the signature is the identity key's over the text ([`Self::signed_text`]; the
    /// sponsor's first check).
    pub fn verify(&self) -> bool {
        let Some(key) = IdentityKey::from_hex(&self.identity) else {
            return false;
        };
        let Some(signature) = crate::keys::from_hex(&self.signature) else {
            return false;
        };
        crate::sign::verify(&key, self.signed_text().as_bytes(), &signature)
    }
    /// Whether `time` is within [`MAX_SKEW`] of `now`.
    pub fn fresh(&self, now: u64) -> bool {
        self.time.abs_diff(now) <= MAX_SKEW
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// A sponsor's yes: the DID it launched (not yet confirmed).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reply {
    pub did: String,
    pub launcher: String,
    pub coin: String,
    pub status: String,
}

impl Reply {
    /// The launcher id, checked against the `did:chia:1…` text when both are given.
    pub fn launcher_id(&self) -> Result<[u8; 32]> {
        let from_hex =
            crate::keys::from_hex(self.launcher.trim().trim_start_matches("0x")).and_then(|b| <[u8; 32]>::try_from(b.as_slice()).ok());
        let from_did = crate::keys::decode_did(&self.did);
        match (from_hex, from_did) {
            (Some(a), Some(b)) if a != b => Err(Error::new("the sponsor's answer names two different DIDs")),
            (Some(a), _) | (None, Some(a)) => Ok(a),
            (None, None) => Err(Error::new("the sponsor's answer names no DID")),
        }
    }
}

/// A sponsor's no.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Refusal {
    pub error: String,
}

/// The `status` of a reply rebuilt from a 403 that names the DID the sponsor already launched
/// for this identity: the player's game watches and links that one (after checking on chain
/// that it is held by the player's address, as for a new one).
pub const EXISTING: &str = "existing";

/// A 403's body when the identity already has a DID from this sponsor (I9 as built).
#[derive(Deserialize)]
struct Existing {
    #[serde(default)]
    did: String,
    #[serde(default)]
    launcher: String,
}

/// Read a sponsor's answer: its HTTP status and body. A refusal's text is the sponsor's own words
/// (shown to the player), clipped. A 403 naming a DID the sponsor already launched for this
/// identity reads as a [`Reply`] with status [`EXISTING`].
pub fn read_answer(status: u16, body: &str) -> Result<Reply> {
    if status == 200 {
        let reply: Reply = serde_json::from_str(body).map_err(|e| Error::new(format!("the sponsor's answer can't be read: {e}")))?;
        reply.launcher_id()?;
        return Ok(reply);
    }
    if status == 403
        && let Ok(e) = serde_json::from_str::<Existing>(body)
    {
        let reply = Reply { did: e.did, launcher: e.launcher, coin: String::new(), status: EXISTING.into() };
        if reply.launcher_id().is_ok() {
            return Ok(reply);
        }
    }
    let why = serde_json::from_str::<Refusal>(body)
        .map(|r| r.error)
        .unwrap_or_else(|_| format!("HTTP {status}"));
    let why: String = why.chars().take(200).collect();
    Err(Error::new(match status {
        429 => format!("the sponsor is busy or its budget is used up for now: {why}"),
        403 => format!("the sponsor refused: {why}"),
        503 => format!("the sponsor can't launch DIDs right now: {why}"),
        _ => format!("the sponsor said no ({status}): {why}"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IdentitySigner, Mnemonic};

    #[test]
    fn requests_sign_and_verify() {
        let s = IdentitySigner::from_mnemonic(&Mnemonic::generate().unwrap(), 0);
        let r = Request::sign(&s, "testnet11", "txch1abc", 1_700_000_000).unwrap();
        assert!(r.verify());
        assert!(r.fresh(1_700_000_100));
        assert!(!r.fresh(1_700_001_000));
        let mut moved = r.clone();
        moved.address = "txch1other".into();
        assert!(!moved.verify());
        let mut late = r.clone();
        late.time += 1;
        assert!(!late.verify());
        // The proof of work: found, checked, and broken by another nonce.
        let mut worked = r.clone();
        worked.do_work(8).unwrap();
        assert!(worked.work_bits() >= 8 && worked.verify());
        assert!(worked.clone().do_work(41).is_err());
        assert_eq!(zero_bits(&[0, 0x1f]), 11);
        let json = r.to_json();
        assert!(json.contains("\"network\":\"testnet11\""), "{json}");
        let back: Request = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn world_requests_sign_and_verify() {
        let s = IdentitySigner::from_mnemonic(&Mnemonic::generate().unwrap(), 0);
        let world = WorldAsk { owner_did: "ab".repeat(32), key: "cd".repeat(32), generation: 3 };
        let r = Request::sign_world(&s, "testnet11", "txch1abc", 1_700_000_000, world.clone()).unwrap();
        assert!(r.verify());
        assert_eq!(
            world_text("txch1abc", 1_700_000_000, &world),
            format!("chiacraft sponsor world v1 txch1abc 1700000000 {} {} 3", "ab".repeat(32), "cd".repeat(32))
        );
        assert_eq!(r.signed_text(), world_text("txch1abc", 1_700_000_000, &world));
        // Tampering with any field of the world (or the rest) breaks it.
        let tampered = |f: &dyn Fn(&mut Request)| {
            let mut t = r.clone();
            f(&mut t);
            t.verify()
        };
        assert!(!tampered(&|t| t.world.as_mut().unwrap().owner_did = "ac".repeat(32)));
        assert!(!tampered(&|t| t.world.as_mut().unwrap().key = "ce".repeat(32)));
        assert!(!tampered(&|t| t.world.as_mut().unwrap().generation = 4));
        assert!(!tampered(&|t| t.address = "txch1other".into()));
        assert!(!tampered(&|t| t.time += 1));
        // Not a player's request: dropping the world, or signing the player's text, fails.
        assert!(!tampered(&|t| t.world = None));
        let player = Request::sign(&s, "testnet11", "txch1abc", 1_700_000_000).unwrap();
        let mut grafted = player.clone();
        grafted.world = Some(world.clone());
        assert!(!grafted.verify());
        assert_ne!(player.signature, r.signature);
        // The proof of work is the same text and covers it.
        let mut worked = r.clone();
        worked.do_work(8).unwrap();
        assert!(worked.work_bits() >= 8 && worked.verify());
        // The JSON carries the world; a player's leaves it out, and JSON without it still parses.
        let json = r.to_json();
        assert!(json.contains("\"world\":{\"owner_did\""), "{json}");
        assert_eq!(serde_json::from_str::<Request>(&json).unwrap(), r);
        assert!(!player.to_json().contains("world"));
        let old = r#"{"network":"testnet11","address":"txch1abc","identity":"00","signature":"00","time":5}"#;
        let back: Request = serde_json::from_str(old).unwrap();
        assert_eq!((back.world, back.work), (None, 0));
    }

    #[test]
    fn answers_read() {
        let id = [9u8; 32];
        let did = crate::keys::encode_did(&id).unwrap();
        let body = format!(r#"{{"did":"{did}","launcher":"{}","coin":"00","status":"pending"}}"#, crate::keys::hex(&id));
        assert_eq!(read_answer(200, &body).unwrap().launcher_id().unwrap(), id);
        let wrong = body.replace(&crate::keys::hex(&id), &crate::keys::hex(&[8u8; 32]));
        assert!(read_answer(200, &wrong).is_err());
        let e = read_answer(429, r#"{"error":"daily budget"}"#).unwrap_err();
        assert!(e.to_string().contains("daily budget"), "{e}");
        // The identity already has one: that DID, to watch and link.
        let had = format!(r#"{{"error":"this identity already has a DID","did":"{did}","launcher":"{}"}}"#, crate::keys::hex(&id));
        let r = read_answer(403, &had).unwrap();
        assert_eq!((r.status.as_str(), r.launcher_id().unwrap()), (EXISTING, id));
        let e = read_answer(403, r#"{"error":"network not served"}"#).unwrap_err();
        assert!(e.to_string().contains("not served"), "{e}");
        assert!(read_answer(503, r#"{"error":"not synced"}"#).is_err());
        assert!(
            read_answer(500, "<html>")
                .unwrap_err()
                .to_string()
                .contains("500")
        );
    }
}
