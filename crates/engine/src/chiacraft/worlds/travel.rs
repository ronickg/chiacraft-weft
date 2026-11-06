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

/// Where a player arriving through a portal stands: the spot beside the world's portal for that
/// link, the portal's dimension (by name; `None` in spots saved before it was kept: the spawn's),
/// and the way to face (yaw in radians), away from the portal.
#[derive(Clone, Debug, PartialEq)]
pub struct Arrival {
    pub pos: [f64; 3],
    pub dim: Option<String>,
    pub yaw: Option<f32>,
}

/// Where player `key` arrives in the world in `dir`, if they came through a portal just now.
pub fn arrival(dir: &Path, key: &str) -> Option<Arrival> {
    let (link, when) = ARRIVING.lock().as_mut()?.remove(key)?;
    if when.elapsed() > Duration::from_secs(120) {
        return None;
    }
    let spot = spots(dir).into_iter().find(|s| s.link == link);
    if spot.is_none() {
        log::info!("{key} came through a portal, but this world has no portal for that link: placed as usual");
    }
    spot.map(|s| Arrival { pos: [s.pos[0] as f64 + 0.5, s.pos[1] as f64, s.pos[2] as f64 + 0.5], dim: s.dim, yaw: s.yaw })
}

/// A portal's arrival spot as `portals.txt` keeps it, a line each:
/// `link x y z [dimension yaw]` (the last two since 11-05; older lines have neither).
#[derive(Clone, Debug, PartialEq)]
pub struct Spot {
    pub link: String,
    pub pos: [i32; 3],
    pub dim: Option<String>,
    pub yaw: Option<f32>,
}

fn spots(dir: &Path) -> Vec<Spot> {
    std::fs::read_to_string(dir.join("portals.txt"))
        .unwrap_or_default()
        .lines()
        .filter_map(parse_spot)
        .collect()
}

fn parse_spot(line: &str) -> Option<Spot> {
    let mut f = line.split_whitespace();
    let link = f.next()?.to_string();
    let p: Vec<i32> = f.by_ref().take(3).filter_map(|n| n.parse().ok()).collect();
    let [x, y, z] = p[..] else { return None };
    let dim = f.next().map(str::to_string);
    let yaw = f
        .next()
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|v| v.is_finite());
    Some(Spot { link, pos: [x, y, z], dim, yaw })
}

/// Sets (or, with `None`, forgets) where arrivals through `link` stand in the world in `dir`.
pub fn set_spot(dir: &Path, link: &str, spot: Option<([i32; 3], Option<String>, Option<f32>)>) {
    let mut all = spots(dir);
    all.retain(|s| s.link != link);
    if let Some((pos, dim, yaw)) = spot {
        all.push(Spot { link: link.to_string(), pos, dim, yaw });
    }
    let text: String = all
        .iter()
        .map(|s| {
            let mut l = format!("{} {} {} {}", s.link, s.pos[0], s.pos[1], s.pos[2]);
            if let Some(d) = &s.dim {
                l.push_str(&format!(" {d} {:.4}", s.yaw.unwrap_or(0.0)));
            }
            l + "\n"
        })
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
        set_spot(&dir, "a", Some(([1, 2, 3], None, None)));
        set_spot(&dir, "b", Some(([4, 5, 6], Some("base:overworld".into()), Some(1.5))));
        set_spot(&dir, "a", Some(([7, 8, 9], Some("base:nether".into()), Some(-1.5708))));
        assert_eq!(
            spots(&dir),
            vec![
                Spot { link: "b".into(), pos: [4, 5, 6], dim: Some("base:overworld".into()), yaw: Some(1.5) },
                Spot { link: "a".into(), pos: [7, 8, 9], dim: Some("base:nether".into()), yaw: Some(-1.5708) },
            ]
        );
        set_spot(&dir, "b", None);
        assert_eq!(spots(&dir).len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn old_spot_lines_still_read() {
        assert_eq!(parse_spot("l 1 2 3"), Some(Spot { link: "l".into(), pos: [1, 2, 3], dim: None, yaw: None }));
        assert_eq!(parse_spot("l 1 2"), None);
    }
}
