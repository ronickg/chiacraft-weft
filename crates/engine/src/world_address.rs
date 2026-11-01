//! A world's own address (chiacraft plan, phase 3): every world has an Ed25519 key of its own,
//! derived from the owner's words with a random index the world keeps, and its public half is
//! both the world's address and the id of the endpoint that hosts it. The owner's identity key
//! certifies it, so whoever reaches the world learns who owns it.
//!
//! An address is `chiacraft://w/<world key>?n=<name>&r=<relay>&a=<ip:port,...>`: the key alone is
//! enough (it is looked up), the rest are hints. The older `<key>;<relay>;<ip:port,...>` ticket
//! still works.
//!
//! The certificate, served by the world (`chiacraft/status/1`) and kept with saved worlds:
//!
//! ```text
//! chiacraft world certificate 1
//! world <world key>
//! generation <n>
//! created <seconds since 1970>
//! owner <identity hex>:<the identity key's signature of `world_cert_text`, hex>
//! ```

use std::hash::{BuildHasher, Hasher};

use crate::saves::WorldMeta;

const CERT_HEADER: &str = "chiacraft world certificate 1";

/// A world's key and the owner's certificate for it.
pub struct WorldIdentity {
    pub secret: iroh::SecretKey,
    pub cert: WorldCert,
}

impl WorldIdentity {
    /// The world's address: its key, as the network writes it.
    pub fn key(&self) -> String {
        self.secret.public().to_string()
    }
}

/// The owner's certificate for a world, checked or to be checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldCert {
    pub world: String,
    pub generation: u32,
    pub created: u64,
    /// `<identity hex>:<signature hex>`.
    pub owner: String,
}

impl WorldCert {
    pub fn text(&self) -> String {
        format!("{CERT_HEADER}\nworld {}\ngeneration {}\ncreated {}\nowner {}\n", self.world, self.generation, self.created, self.owner)
    }

    pub fn parse(text: &str) -> Result<WorldCert, String> {
        if text.len() > 1024 {
            return Err("certificate too big".into());
        }
        let mut lines = text.lines();
        if lines.next() != Some(CERT_HEADER) {
            return Err("not a world certificate".into());
        }
        let mut field = |name: &str| lines.next().and_then(|l| l.strip_prefix(name)).and_then(|r| r.strip_prefix(' ')).map(str::to_string).ok_or_else(|| format!("no {name} line"));
        let world = field("world")?;
        let generation = field("generation")?.parse().map_err(|_| "bad generation")?;
        let created = field("created")?.parse().map_err(|_| "bad created")?;
        let owner = field("owner")?;
        Ok(WorldCert { world, generation, created, owner })
    }

    /// The owner's identity (hex) if the certificate checks out.
    pub fn verify(&self) -> Option<String> {
        chiacraft_account::check_world_cert(&self.owner, &self.world, self.generation, self.created)
    }

    /// The owner's identity as written, checked or not (the first 12 digits are what people see).
    pub fn owner_identity(&self) -> &str {
        self.owner.split(':').next().unwrap_or("")
    }
}

/// A random index for a new world's key: worlds made on two computers never share one.
pub fn random_index() -> u32 {
    let n = std::collections::hash_map::RandomState::new().build_hasher().finish();
    (n as u32) & 0x7FFF_FFFF
}

/// The world's key and certificate, from the open account. A world without a key index yet (made
/// before worlds had keys) gets one, saved with it.
pub fn for_world(meta: &mut WorldMeta) -> Result<WorldIdentity, String> {
    let session = chiacraft_account::current().ok_or("no account is open")?;
    if meta.key_index.is_none() {
        meta.key_index = Some(random_index());
        if meta.created == 0 {
            meta.created = meta.last_played.max(1);
        }
        meta.save();
    }
    let index = meta.key_index.unwrap_or(0);
    let secret = iroh::SecretKey::from_bytes(&session.world_seed(index, meta.key_generation));
    let world = secret.public().to_string();
    let owner = session.world_cert_line(&world, meta.key_generation, meta.created)?;
    Ok(WorldIdentity { secret, cert: WorldCert { world, generation: meta.key_generation, created: meta.created, owner } })
}

/// The address to share: the key, and as hints the world's name, relay and direct addresses.
pub fn address(key: &str, name: &str, relay: Option<&str>, addrs: &[String]) -> String {
    let mut s = format!("chiacraft://w/{key}");
    let mut sep = '?';
    let mut hint = |k: &str, v: &str| {
        if !v.is_empty() {
            s.push_str(&format!("{sep}{k}={}", encode(v)));
            sep = '&';
        }
    };
    hint("n", name);
    hint("r", relay.unwrap_or(""));
    hint("a", &addrs.join(","));
    s
}

/// What an address says: the key, and the hints it carries (name, relay, direct addresses).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Parsed {
    pub key: String,
    pub name: String,
    pub relay: String,
    pub addrs: Vec<String>,
}

/// Reads an address, or the older `<key>;<relay>;<ip:port,...>` ticket.
pub fn parse(text: &str) -> Option<Parsed> {
    let text = text.trim();
    if let Some(rest) = text.strip_prefix("chiacraft://w/") {
        let (key, query) = rest.split_once('?').unwrap_or((rest, ""));
        let mut p = Parsed { key: key.trim_end_matches('/').to_string(), ..Default::default() };
        for pair in query.split('&').filter(|s| !s.is_empty()) {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            let v = decode(v)?;
            match k {
                "n" => p.name = v.chars().take(64).collect(),
                "r" => p.relay = v,
                "a" => p.addrs = v.split(',').filter(|s| !s.is_empty()).map(str::to_string).collect(),
                _ => {}
            }
        }
        return (!p.key.is_empty()).then_some(p);
    }
    let mut parts = text.split(';');
    let key = parts.next()?.to_string();
    let relay = parts.next().unwrap_or("").to_string();
    let addrs = parts.next().unwrap_or("").split(',').filter(|s| !s.is_empty()).map(str::to_string).collect();
    (!key.is_empty()).then_some(Parsed { key, name: String::new(), relay, addrs })
}

fn encode(v: &str) -> String {
    let mut out = String::new();
    for b in v.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~:/,".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn decode(v: &str) -> Option<String> {
    let b = v.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' => {
                out.push(u8::from_str_radix(v.get(i + 1..i + 3)?, 16).ok()?);
                i += 3;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_round_trip_and_old_tickets() {
        let a = address("abc123", "Ruby Hills & co", Some("https://relay.example/"), &["1.2.3.4:5".into(), "[::1]:6".into()]);
        assert!(a.starts_with("chiacraft://w/abc123?n=Ruby%20Hills%20%26%20co&r=https://relay.example/&a="));
        let p = parse(&a).unwrap();
        assert_eq!(p, Parsed { key: "abc123".into(), name: "Ruby Hills & co".into(), relay: "https://relay.example/".into(), addrs: vec!["1.2.3.4:5".into(), "[::1]:6".into()] });
        assert_eq!(parse("chiacraft://w/abc123").unwrap().key, "abc123");
        let old = parse("abc123;https://relay.example/;1.2.3.4:5").unwrap();
        assert_eq!((old.key.as_str(), old.relay.as_str(), old.addrs.len()), ("abc123", "https://relay.example/", 1));
        assert!(parse("").is_none() && parse("chiacraft://w/").is_none());
    }

    #[test]
    fn certificate_text_round_trip() {
        let c = WorldCert { world: "abc".into(), generation: 2, created: 1_700_000_000, owner: "aa:bb".into() };
        assert_eq!(WorldCert::parse(&c.text()).unwrap(), c);
        assert!(WorldCert::parse("hello").is_err());
        assert_eq!(c.owner_identity(), "aa");
    }
}
