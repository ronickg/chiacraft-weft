//! Travel between linked worlds (chiacraft plan, phase 3): a portal's world signs a short-lived
//! travel ticket for the player who steps in; the player's game carries it into the join; the
//! linked world checks it and places the player beside its own portal for that link instead of at
//! its spawn.
//!
//! A ticket, one line, signed by the origin world's key after the label `chiacraft travel v1\n`:
//!
//! ```text
//! chiacraft travel 1;player=<player key>;from=<origin world key>;to=<destination world key>;link=<link id>;expires=<secs>;nonce=<16 hex>;sig=<hex>
//! ```
//!
//! The destination takes it only for the player whose connection it is, for a link it has with
//! the origin, before it expires, and once. Where each world's portals for its links are (the
//! arrival spots) is kept in `portals.txt` in the world's folder: a line per link,
//! `<link id> <x> <y> <z>`, set when the owner binds a portal.

use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

const LABEL: &str = "chiacraft travel v1\n";
const HEADER: &str = "chiacraft travel 1";
/// How long a ticket is good for.
const LIFETIME: u64 = 60;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ticket {
    pub player: String,
    pub from: String,
    pub to: String,
    pub link: String,
    pub expires: u64,
    pub nonce: String,
    pub sig: String,
}

impl Ticket {
    fn body(&self) -> String {
        format!(
            "{HEADER};player={};from={};to={};link={};expires={};nonce={}",
            self.player, self.from, self.to, self.link, self.expires, self.nonce
        )
    }

    fn signed(&self) -> Vec<u8> {
        format!("{LABEL}{}", self.body()).into_bytes()
    }

    pub fn text(&self) -> String {
        format!("{};sig={}", self.body(), self.sig)
    }

    pub fn parse(text: &str) -> Option<Ticket> {
        if text.len() > 1024 {
            return None;
        }
        let mut parts = text.split(';');
        if parts.next()? != HEADER {
            return None;
        }
        let mut field = |name: &str| {
            parts
                .next()?
                .strip_prefix(name)?
                .strip_prefix('=')
                .map(str::to_string)
        };
        Some(Ticket {
            player: field("player")?,
            from: field("from")?,
            to: field("to")?,
            link: field("link")?,
            expires: field("expires")?.parse().ok()?,
            nonce: field("nonce")?,
            sig: field("sig")?,
        })
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// The origin world's ticket for `player` (their key) to go through `link` to the world `to`.
pub fn issue(world: &crate::world_address::WorldIdentity, player: &str, link: &str, to: &str) -> String {
    use std::hash::{BuildHasher, Hasher};
    let nonce = format!(
        "{:016x}",
        std::collections::hash_map::RandomState::new()
            .build_hasher()
            .finish()
    );
    let mut t = Ticket {
        player: player.into(),
        from: world.key(),
        to: to.into(),
        link: link.into(),
        expires: crate::saves::now_secs() + LIFETIME,
        nonce,
        sig: String::new(),
    };
    t.sig = hex(&world.secret.sign(&t.signed()).to_bytes());
    t.text()
}

/// Tickets taken, by nonce, until they expire: each is good once.
static USED: Mutex<Option<HashMap<String, u64>>> = Mutex::new(None);
/// Players let in with a ticket, waiting to be placed: their key, the link, and when.
static ARRIVING: Mutex<Option<HashMap<String, (String, Instant)>>> = Mutex::new(None);

/// Checks a ticket a joining player brought to world `me` (its key), on connection `peer` (the
/// player's key). On success the player is placed at the link's portal when they join.
pub fn admit(text: &str, peer: &str, me: &str) -> Result<(), String> {
    let t = Ticket::parse(text).ok_or("a travel ticket this game can't read")?;
    let now = crate::saves::now_secs();
    if t.player != peer || t.to != me {
        return Err("a travel ticket for someone else".into());
    }
    if t.expires < now || t.expires > now + 2 * LIFETIME {
        return Err("an expired travel ticket".into());
    }
    let (Ok(from), Some(sig)) = (t.from.parse::<iroh::PublicKey>(), unhex64(&t.sig)) else { return Err("a bad travel ticket".into()) };
    if from
        .verify(&t.signed(), &iroh::Signature::from_bytes(&sig))
        .is_err()
    {
        return Err("a travel ticket its world didn't sign".into());
    }
    let linked = crate::links::of_world(me)
        .iter()
        .any(|l| l.id() == t.link && (l.body.from == t.from || l.body.to == t.from));
    if !linked {
        return Err("a travel ticket for a link this world doesn't have".into());
    }
    let mut used = USED.lock();
    let used = used.get_or_insert_with(HashMap::new);
    used.retain(|_, exp| *exp >= now);
    if used.insert(t.nonce.clone(), t.expires).is_some() {
        return Err("a travel ticket used already".into());
    }
    ARRIVING
        .lock()
        .get_or_insert_with(HashMap::new)
        .insert(t.player, (t.link, Instant::now()));
    Ok(())
}

/// Coming home through a portal (to the world this game runs): the player's own world needs no
/// ticket checked; they are placed at its portal for `link`.
pub fn expect(player: &str, link: &str) {
    ARRIVING
        .lock()
        .get_or_insert_with(HashMap::new)
        .insert(player.to_string(), (link.to_string(), Instant::now()));
}

fn unhex64(s: &str) -> Option<[u8; 64]> {
    if s.len() != 128 {
        return None;
    }
    let mut out = [0u8; 64];
    for (i, o) in out.iter_mut().enumerate() {
        *o = u8::from_str_radix(s.get(i * 2..i * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}

/// Where player `key` arrives in the world in `dir`, if they came through a portal just now: the
/// spot beside the world's portal for that link.
pub fn arrival(dir: &Path, key: &str) -> Option<[f64; 3]> {
    let (link, when) = ARRIVING.lock().as_mut()?.remove(key)?;
    if when.elapsed() > Duration::from_secs(120) {
        return None;
    }
    let spot = spots(dir)
        .into_iter()
        .find(|(l, _)| *l == link)
        .map(|(_, p)| p);
    if spot.is_none() {
        log::info!("{key} came through a portal, but this world has no portal for that link: placed as usual");
    }
    spot.map(|p| [p[0] as f64 + 0.5, p[1] as f64, p[2] as f64 + 0.5])
}

fn spots(dir: &Path) -> Vec<(String, [i32; 3])> {
    std::fs::read_to_string(dir.join("portals.txt"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let mut f = l.split_whitespace();
            let link = f.next()?.to_string();
            let p: Vec<i32> = f.filter_map(|n| n.parse().ok()).collect();
            let [x, y, z] = p[..] else { return None };
            Some((link, [x, y, z]))
        })
        .collect()
}

/// Sets (or, with `None`, forgets) where arrivals through `link` stand in the world in `dir`.
pub fn set_spot(dir: &Path, link: &str, pos: Option<[i32; 3]>) {
    let mut all = spots(dir);
    all.retain(|(l, _)| l != link);
    if let Some(p) = pos {
        all.push((link.to_string(), p));
    }
    let text: String = all
        .iter()
        .map(|(l, p)| format!("{l} {} {} {}\n", p[0], p[1], p[2]))
        .collect();
    if let Err(e) = std::fs::write(dir.join("portals.txt"), text) {
        log::warn!("saving portal spots: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticket_round_trip_and_signature() {
        let k = iroh::SecretKey::from_bytes(&[9; 32]);
        let mut t = Ticket {
            player: "p".into(),
            from: k.public().to_string(),
            to: "w".into(),
            link: "l".into(),
            expires: 5,
            nonce: "0011223344556677".into(),
            sig: String::new(),
        };
        t.sig = hex(&k.sign(&t.signed()).to_bytes());
        let back = Ticket::parse(&t.text()).unwrap();
        assert_eq!(back, t);
        let pk: iroh::PublicKey = back.from.parse().unwrap();
        assert!(
            pk.verify(&back.signed(), &iroh::Signature::from_bytes(&unhex64(&back.sig).unwrap()))
                .is_ok()
        );
        let forged = Ticket { player: "q".into(), ..back };
        assert!(
            pk.verify(&forged.signed(), &iroh::Signature::from_bytes(&unhex64(&forged.sig).unwrap()))
                .is_err()
        );
        assert!(Ticket::parse("hello").is_none());
    }

    #[test]
    fn spots_are_kept_per_link() {
        let dir = std::env::temp_dir().join(format!("chiacraft-spots-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        set_spot(&dir, "a", Some([1, 2, 3]));
        set_spot(&dir, "b", Some([4, 5, 6]));
        set_spot(&dir, "a", Some([7, 8, 9]));
        set_spot(&dir, "b", None);
        assert_eq!(spots(&dir), vec![("a".to_string(), [7, 8, 9])]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
