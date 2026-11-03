//! Links between worlds (chiacraft plan, phase 3): one world asks another to link them (portals
//! come next); the other's owner accepts, or has said to always accept that owner; the link is
//! one record both worlds' keys sign over the same bytes, kept by both; either side ends it alone.
//!
//! Requests and answers go between the owners' account endpoints (`chiacraft/links/1`), not the
//! worlds, because a world is offline most of the time while its owner's game runs: the owner's
//! game is the inbox for all of their worlds. What can't be delivered yet waits in an outbox and is
//! tried again. Declining says nothing to the asker. Everything lives in `links.json` in the run
//! folder.
//!
//! The record, signed by each world's key after the label `chiacraft link v1\n`:
//!
//! ```text
//! chiacraft link 1
//! from <asking world's key>
//! to <asked world's key>
//! created <seconds since 1970>
//! nonce <16 hex digits>
//! note <a line from the asker>
//! ```
//!
//! Its id is the BLAKE3 hash of that text. Ending a link is a tombstone, `chiacraft link revoke
//! v1\nlink <id>\nby <world key>\ncreated <secs>`, signed by the world ending it; it can't be
//! undone, and a link ended can't come back.

use std::path::{Path, PathBuf};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

const SIGNED_LABEL: &str = "chiacraft link v1\n";
const REVOKE_LABEL: &str = "chiacraft link revoke v1\n";
/// Requests waiting for an answer, at most; older ones go first when more come.
const MAX_INCOMING: usize = 100;
/// A request or answer older than this (or this far ahead) is refused; queued ones give up.
const MAX_AGE: u64 = 14 * 86_400;
const MAX_NOTE: usize = 200;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Body {
    pub from: String,
    pub to: String,
    pub created: u64,
    pub nonce: String,
    pub note: String,
}

impl Body {
    pub fn text(&self) -> String {
        format!(
            "chiacraft link 1\nfrom {}\nto {}\ncreated {}\nnonce {}\nnote {}\n",
            self.from, self.to, self.created, self.nonce, self.note
        )
    }

    pub fn id(&self) -> String {
        chiacraft_packages::identity::Hash::of(self.text().as_bytes()).hex()
    }

    fn signed(&self) -> Vec<u8> {
        format!("{SIGNED_LABEL}{}", self.text()).into_bytes()
    }

    fn check(&self) -> Result<(), String> {
        let now = crate::saves::now_secs();
        let fine = self.note.chars().count() <= MAX_NOTE
            && !self.note.contains(['\n', '\r'])
            && self.nonce.len() == 16
            && self.nonce.chars().all(|c| c.is_ascii_hexdigit())
            && self.created + MAX_AGE > now
            && self.created < now + 86_400
            && self.from != self.to;
        if fine { Ok(()) } else { Err("a link this game can't take".into()) }
    }
}

/// An accepted link, as each side keeps it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Link {
    pub body: Body,
    pub from_sig: String,
    pub to_sig: String,
    pub from_name: String,
    pub to_name: String,
    /// The owners' identities (hex), from the worlds' certificates.
    pub from_owner: String,
    pub to_owner: String,
    /// The other owner's account endpoint, where a revocation goes.
    pub peer_inbox: String,
}

impl Link {
    pub fn id(&self) -> String {
        self.body.id()
    }
}

/// A request that came in, waiting for its owner's answer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Incoming {
    pub request: Request,
    /// The asking world's owner (identity hex), checked.
    pub owner: String,
    /// The name of the asked world (this player's).
    pub to_name: String,
}

/// A request this player sent, until it is accepted (or forgotten).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Outgoing {
    pub request: Request,
    pub to_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub body: Body,
    pub from_sig: String,
    pub from_cert: String,
    pub from_name: String,
    /// The asking owner's account endpoint, where the answer goes, and their identity's vouching
    /// for it (`<identity hex>:<signature hex>`).
    pub reply_to: String,
    pub reply_vouch: String,
    /// An invite from the asked world's owner (`invite`): the request is accepted without asking.
    #[serde(default)]
    pub invite: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Accept {
    pub body: Body,
    pub from_sig: String,
    pub to_sig: String,
    pub to_cert: String,
    pub to_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Revoke {
    pub link: String,
    pub by: String,
    pub created: u64,
    pub sig: String,
}

impl Revoke {
    fn signed(&self) -> Vec<u8> {
        format!("{REVOKE_LABEL}link {}\nby {}\ncreated {}", self.link, self.by, self.created).into_bytes()
    }
}

/// What goes between account endpoints.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Message {
    Request(Request),
    Accept(Accept),
    Revoke(Revoke),
}

/// A message waiting to be delivered: to an account endpoint (`inbox`), or, while that is not
/// known yet, to whoever owns `world` (asked of the world itself).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Queued {
    pub inbox: String,
    pub world: String,
    /// A hint for finding `world` (its address as the player gave it).
    pub address: String,
    pub message: Message,
    pub queued: u64,
    pub tries: u32,
    pub next_try: u64,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Store {
    pub links: Vec<Link>,
    pub incoming: Vec<Incoming>,
    pub outgoing: Vec<Outgoing>,
    pub outbox: Vec<Queued>,
    /// Ids of links ended, which can't come back.
    pub revoked: Vec<String>,
    /// Owners (identity hex) whose requests are accepted without asking.
    pub allow: Vec<String>,
    /// Owners and worlds whose requests are dropped without a word.
    pub blocked: Vec<String>,
    /// Invites used (their nonces): each is good once.
    #[serde(default)]
    pub invites_used: Vec<String>,
}

struct State {
    path: PathBuf,
    saves: PathBuf,
    store: Store,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

/// Requests that came in since the player last looked (for a notice).
pub static UNSEEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Reads `links.json` in `run_dir`; this player's worlds are in `saves`.
pub fn init(run_dir: &Path, saves: &Path) {
    let path = run_dir.join("links.json");
    let store = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    *STATE.lock() = Some(State { path, saves: saves.to_path_buf(), store });
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> Option<R> {
    STATE.lock().as_mut().map(f)
}

/// Counts changes, so screens showing links know to show them again.
pub static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn save(s: &State) {
    GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    match serde_json::to_string_pretty(&s.store) {
        Ok(t) => {
            if let Err(e) = std::fs::write(&s.path, t) {
                log::warn!("saving links: {e}");
            }
        }
        Err(e) => log::warn!("saving links: {e}"),
    }
}

/// A copy of what is kept, for screens.
pub fn snapshot() -> Store {
    with(|s| serde_json::from_str(&serde_json::to_string(&s.store).unwrap_or_default()).unwrap_or_default()).unwrap_or_default()
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn unhex<const N: usize>(s: &str) -> Option<[u8; N]> {
    if s.len() != N * 2 {
        return None;
    }
    let mut out = [0u8; N];
    for (i, o) in out.iter_mut().enumerate() {
        *o = u8::from_str_radix(s.get(i * 2..i * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}

fn verify(key: &str, msg: &[u8], sig: &str) -> bool {
    let (Ok(key), Some(sig)) = (key.parse::<iroh::PublicKey>(), unhex::<64>(sig)) else { return false };
    key.verify(msg, &iroh::Signature::from_bytes(&sig)).is_ok()
}

/// This player's world with key `key`: its key pair and name.
fn my_world(saves: &Path, key: &str) -> Option<(crate::world_address::WorldIdentity, String)> {
    crate::saves::list(saves)
        .into_iter()
        .filter(|m| m.key_index.is_some())
        .find_map(|mut m| {
            let id = crate::world_address::for_world(&mut m).ok()?;
            (id.key() == key).then(|| (id, m.name.clone()))
        })
}

fn nonce() -> String {
    use std::hash::{BuildHasher, Hasher};
    format!(
        "{:016x}",
        std::collections::hash_map::RandomState::new()
            .build_hasher()
            .finish()
    )
}

fn queue(s: &mut State, inbox: &str, world: &str, address: &str, message: Message) {
    let now = crate::saves::now_secs();
    s.store.outbox.push(Queued {
        inbox: inbox.into(),
        world: world.into(),
        address: address.into(),
        message,
        queued: now,
        tries: 0,
        next_try: now,
    });
}

/// The player's account endpoint and their identity's vouching for it: where answers come back.
fn reply_address(account_key: &iroh::SecretKey) -> Result<(String, String), String> {
    let session = chiacraft_account::current().ok_or("no account is open")?;
    let me = account_key.public().to_string();
    let vouch = session
        .vouch_line(&me)
        .ok_or("the account can't vouch for its network key")?;
    Ok((me, vouch))
}

/// Asks the world at `address` to link with this player's world `from` (named `from_name`).
pub fn request(
    account_key: &iroh::SecretKey,
    from: &crate::world_address::WorldIdentity,
    from_name: &str,
    address: &str,
    note: &str,
) -> Result<(), String> {
    let target = crate::world_address::parse(address).ok_or("that is not a world address")?;
    if target.key.parse::<iroh::PublicKey>().is_err() {
        return Err("that is not a world address".into());
    }
    let (reply_to, reply_vouch) = reply_address(account_key)?;
    let note: String = note
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_NOTE)
        .collect();
    let body = Body { from: from.key(), to: target.key.clone(), created: crate::saves::now_secs(), nonce: nonce(), note };
    let request = Request {
        from_sig: hex(&from.secret.sign(&body.signed()).to_bytes()),
        body,
        from_cert: from.cert.text(),
        from_name: from_name.to_string(),
        reply_to,
        reply_vouch,
        invite: target.invite.clone(),
    };
    with(|s| {
        if s.store.links.iter().any(|l| {
            (l.body.from == request.body.from && l.body.to == request.body.to)
                || (l.body.from == request.body.to && l.body.to == request.body.from)
        }) {
            return Err("these worlds are linked already".to_string());
        }
        s.store
            .outgoing
            .retain(|o| !(o.request.body.from == request.body.from && o.request.body.to == request.body.to));
        s.store
            .outbox
            .retain(|q| !matches!(&q.message, Message::Request(r) if r.body.from == request.body.from && r.body.to == request.body.to));
        queue(s, "", &target.key, address, Message::Request(request.clone()));
        s.store
            .outgoing
            .push(Outgoing { request, to_name: target.name });
        save(s);
        Ok(())
    })
    .unwrap_or_else(|| Err("links are not ready".into()))?;
    crate::net::p2p::wake_outbox();
    Ok(())
}

/// Accepts waiting request `id` (a link id); `always` also accepts this owner from now on.
pub fn accept(id: &str, always: bool) -> Result<(), String> {
    with(|s| {
        let i = s
            .store
            .incoming
            .iter()
            .position(|r| r.request.body.id() == id)
            .ok_or_else(|| "no such request".to_string())?;
        let inc = s.store.incoming.remove(i);
        if always && !s.store.allow.contains(&inc.owner) {
            s.store.allow.push(inc.owner.clone());
        }
        let r = accept_request(s, &inc.request, &inc.owner);
        save(s);
        r
    })
    .unwrap_or_else(|| Err("links are not ready".into()))?;
    crate::net::p2p::wake_outbox();
    Ok(())
}

fn accept_request(s: &mut State, r: &Request, owner: &str) -> Result<(), String> {
    let (world, name) = my_world(&s.saves, &r.body.to).ok_or("that world is not on this computer")?;
    let to_sig = hex(&world.secret.sign(&r.body.signed()).to_bytes());
    let to_owner = world.cert.owner_identity().to_string();
    s.store.links.push(Link {
        body: r.body.clone(),
        from_sig: r.from_sig.clone(),
        to_sig: to_sig.clone(),
        from_name: r.from_name.clone(),
        to_name: name.clone(),
        from_owner: owner.to_string(),
        to_owner,
        peer_inbox: r.reply_to.clone(),
    });
    let a = Accept { body: r.body.clone(), from_sig: r.from_sig.clone(), to_sig, to_cert: world.cert.text(), to_name: name.clone() };
    queue(s, &r.reply_to, &r.body.from, "", Message::Accept(a));
    log::info!("linked '{}' with '{}'", name, r.from_name);
    Ok(())
}

/// Declines waiting request `id`: the asker is not told. `block` also drops that owner's requests
/// from now on.
pub fn decline(id: &str, block: bool) {
    with(|s| {
        if let Some(i) = s
            .store
            .incoming
            .iter()
            .position(|r| r.request.body.id() == id)
        {
            let inc = s.store.incoming.remove(i);
            if block && !s.store.blocked.contains(&inc.owner) {
                s.store.blocked.push(inc.owner);
            }
            save(s);
        }
    });
}

/// Ends link `id`: for good, on both sides (the other side is told when it can be reached).
pub fn revoke(id: &str) -> Result<(), String> {
    with(|s| {
        let i = s
            .store
            .links
            .iter()
            .position(|l| l.id() == id)
            .ok_or("no such link")?;
        let link = s.store.links[i].clone();
        let mine = [&link.body.from, &link.body.to]
            .into_iter()
            .find_map(|k| my_world(&s.saves, k))
            .ok_or("neither world is on this computer")?;
        let mut r = Revoke { link: id.to_string(), by: mine.0.key(), created: crate::saves::now_secs(), sig: String::new() };
        r.sig = hex(&mine.0.secret.sign(&r.signed()).to_bytes());
        s.store.links.remove(i);
        s.store.revoked.push(id.to_string());
        let other = if link.body.from == r.by { &link.body.to } else { &link.body.from };
        queue(s, &link.peer_inbox, other, "", Message::Revoke(r));
        save(s);
        log::info!("ended the link of '{}' and '{}'", link.from_name, link.to_name);
        Ok::<(), String>(())
    })
    .unwrap_or_else(|| Err("links are not ready".into()))?;
    crate::net::p2p::wake_outbox();
    Ok(())
}

/// Forgets a request this player sent (the other side may still answer it; that is then ignored).
pub fn withdraw(id: &str) {
    with(|s| {
        s.store.outgoing.retain(|o| o.request.body.id() != id);
        s.store
            .outbox
            .retain(|q| !matches!(&q.message, Message::Request(r) if r.body.id() == id));
        save(s);
    });
}

/// A message from the account endpoint `peer`. Errors are for the log; refusals of strangers are
/// quiet (`Ok`), so the asker learns nothing.
pub fn receive(peer: &str, message: Message) -> Result<(), String> {
    let r = with(|s| {
        let r = match message {
            Message::Request(r) => receive_request(s, peer, r),
            Message::Accept(a) => receive_accept(s, peer, a),
            Message::Revoke(r) => receive_revoke(s, r),
        };
        save(s);
        r
    })
    .unwrap_or_else(|| Err("links are not ready".into()));
    crate::net::p2p::wake_outbox();
    r
}

fn receive_request(s: &mut State, peer: &str, r: Request) -> Result<(), String> {
    r.body.check()?;
    if r.reply_to != peer {
        return Err("a request relayed by someone else".into());
    }
    if !verify(&r.body.from, &r.body.signed(), &r.from_sig) {
        return Err("a request its world didn't sign".into());
    }
    let cert = crate::world_address::WorldCert::parse(&r.from_cert)?;
    let owner = (cert.world == r.body.from)
        .then(|| cert.verify())
        .flatten()
        .ok_or("a request whose world's certificate doesn't check out")?;
    if chiacraft_account::check_vouch(&r.reply_vouch, &r.reply_to).as_deref() != Some(owner.as_str()) {
        return Err("a request whose answer would go to someone other than its owner".into());
    }
    let id = r.body.id();
    if s.store
        .blocked
        .iter()
        .any(|b| *b == owner || *b == r.body.from)
        || s.store.revoked.contains(&id)
        || s.store.links.iter().any(|l| l.id() == id)
    {
        return Ok(());
    }
    let Some((_, to_name)) = my_world(&s.saves, &r.body.to) else {
        return Ok(());
    };
    if s.store.allow.contains(&owner) {
        log::info!("link request from '{}' for '{to_name}': accepted (its owner is always accepted)", r.from_name);
        return accept_request(s, &r, &owner);
    }
    if !r.invite.is_empty() {
        match check_invite(&r.invite, &r.body.to) {
            Ok(nonce) if !s.store.invites_used.contains(&nonce) => {
                log::info!("link request from '{}' for '{to_name}': accepted (it came with an invite)", r.from_name);
                s.store.invites_used.push(nonce);
                return accept_request(s, &r, &owner);
            }
            Ok(_) => log::info!("link request from '{}': its invite was used already", r.from_name),
            Err(e) => log::info!("link request from '{}': {e}", r.from_name),
        }
    }
    // One waiting request per pair of worlds; the oldest go when there are too many.
    s.store
        .incoming
        .retain(|i| !(i.request.body.from == r.body.from && i.request.body.to == r.body.to));
    while s.store.incoming.len() >= MAX_INCOMING {
        s.store.incoming.remove(0);
    }
    log::info!("link request from '{}' (owner {}) for '{to_name}'", r.from_name, &owner[..12.min(owner.len())]);
    s.store
        .incoming
        .push(Incoming { request: r, owner, to_name });
    UNSEEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

fn receive_accept(s: &mut State, peer: &str, a: Accept) -> Result<(), String> {
    let id = a.body.id();
    let Some(i) = s
        .store
        .outgoing
        .iter()
        .position(|o| o.request.body.id() == id)
    else {
        return Ok(());
    };
    let o = s.store.outgoing[i].clone();
    if o.request.from_sig != a.from_sig || !verify(&a.body.to, &a.body.signed(), &a.to_sig) {
        return Err("an answer the asked world didn't sign".into());
    }
    let cert = crate::world_address::WorldCert::parse(&a.to_cert)?;
    let to_owner = (cert.world == a.body.to)
        .then(|| cert.verify())
        .flatten()
        .ok_or("an answer whose world's certificate doesn't check out")?;
    let from_owner = crate::world_address::WorldCert::parse(&o.request.from_cert)
        .map(|c| c.owner_identity().to_string())
        .unwrap_or_default();
    s.store.outgoing.remove(i);
    s.store
        .outbox
        .retain(|q| !matches!(&q.message, Message::Request(r) if r.body.id() == id));
    log::info!("'{}' accepted the link with '{}'", a.to_name, o.request.from_name);
    s.store.links.push(Link {
        body: a.body,
        from_sig: a.from_sig,
        to_sig: a.to_sig,
        from_name: o.request.from_name,
        to_name: a.to_name,
        from_owner,
        to_owner,
        peer_inbox: peer.to_string(),
    });
    Ok(())
}

fn receive_revoke(s: &mut State, r: Revoke) -> Result<(), String> {
    let Some(i) = s.store.links.iter().position(|l| l.id() == r.link) else {
        if !s.store.revoked.contains(&r.link) {
            // It may arrive before the accept it ends: remember it all the same.
            s.store.revoked.push(r.link);
        }
        return Ok(());
    };
    let l = &s.store.links[i];
    if (r.by != l.body.from && r.by != l.body.to) || !verify(&r.by, &r.signed(), &r.sig) {
        return Err("an ending neither world signed".into());
    }
    log::info!("the link of '{}' and '{}' was ended", l.from_name, l.to_name);
    s.store.links.remove(i);
    s.store.revoked.push(r.link);
    Ok(())
}

const INVITE_HEADER: &str = "chiacraft-invite-1";
const INVITE_LABEL: &str = "chiacraft link invite v1\n";
/// How long an invite is good for.
const INVITE_LIFETIME: u64 = 7 * 86_400;

/// An invite to link with world `world` (its address, with the invite as a hint): whoever asks
/// with it is linked without being asked about, once, within a week. Signed by the world's key.
pub fn invite(world: &crate::world_address::WorldIdentity, name: &str) -> String {
    let body = format!("{INVITE_HEADER};world={};expires={};nonce={}", world.key(), crate::saves::now_secs() + INVITE_LIFETIME, nonce());
    let sig = hex(&world
        .secret
        .sign(format!("{INVITE_LABEL}{body}").as_bytes())
        .to_bytes());
    let address = crate::world_address::address(&world.key(), name, None, &[]);
    crate::world_address::with_hint(&address, "i", &format!("{body};sig={sig}"))
}

/// The invite's nonce if it is world `world`'s, signed by it, and not expired.
fn check_invite(text: &str, world: &str) -> Result<String, String> {
    let (body, sig) = text
        .rsplit_once(";sig=")
        .ok_or("an invite this game can't read")?;
    let mut f = body.split(';');
    let field = |f: &mut std::str::Split<'_, char>, name: &str| {
        f.next()
            .and_then(|p| p.strip_prefix(name))
            .and_then(|p| p.strip_prefix('='))
            .map(str::to_string)
    };
    if f.next() != Some(INVITE_HEADER) {
        return Err("an invite this game can't read".into());
    }
    let (Some(w), Some(expires), Some(nonce)) = (field(&mut f, "world"), field(&mut f, "expires"), field(&mut f, "nonce")) else {
        return Err("an invite this game can't read".into());
    };
    if w != world {
        return Err("an invite for another world".into());
    }
    if expires.parse::<u64>().unwrap_or(0) < crate::saves::now_secs() {
        return Err("an expired invite".into());
    }
    if !verify(world, format!("{INVITE_LABEL}{body}").as_bytes(), sig) {
        return Err("an invite its world didn't sign".into());
    }
    Ok(nonce)
}

/// The invite made last, for the Links screen to show.
pub static LAST_INVITE: Mutex<String> = Mutex::new(String::new());

/// Messages due for delivery now (a copy, with their place in the outbox).
pub fn due() -> Vec<(usize, Queued)> {
    let now = crate::saves::now_secs();
    with(|s| {
        s.store.outbox.retain(|q| q.queued + MAX_AGE > now);
        s.store
            .outbox
            .iter()
            .cloned()
            .enumerate()
            .filter(|(_, q)| q.next_try <= now)
            .collect()
    })
    .unwrap_or_default()
}

/// The result of trying to deliver `q`: delivered, or tried again later (the inbox, if it was
/// learned meanwhile, is kept).
pub fn tried(q: &Queued, inbox: &str, delivered: bool) {
    with(|s| {
        let Some(slot) = s
            .store
            .outbox
            .iter_mut()
            .position(|x| x.queued == q.queued && x.world == q.world && same(&x.message, &q.message))
        else {
            return;
        };
        if delivered {
            s.store.outbox.remove(slot);
        } else {
            let x = &mut s.store.outbox[slot];
            x.inbox = inbox.to_string();
            x.tries += 1;
            x.next_try = crate::saves::now_secs() + (30u64 << x.tries.min(7)).min(3600);
        }
        save(s);
    });
}

fn same(a: &Message, b: &Message) -> bool {
    serde_json::to_string(a).ok() == serde_json::to_string(b).ok()
}

/// The links of world `key` (for its portals, and screens).
pub fn of_world(key: &str) -> Vec<Link> {
    with(|s| {
        s.store
            .links
            .iter()
            .filter(|l| l.body.from == key || l.body.to == key)
            .cloned()
            .collect()
    })
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_text_and_checks() {
        let now = crate::saves::now_secs();
        let b = Body { from: "a".into(), to: "b".into(), created: now, nonce: "0123456789abcdef".into(), note: "hi".into() };
        assert_eq!(b.text(), format!("chiacraft link 1\nfrom a\nto b\ncreated {now}\nnonce 0123456789abcdef\nnote hi\n"));
        assert_eq!(b.id().len(), 64);
        assert!(b.check().is_ok());
        assert!(Body { note: "a\nb".into(), ..b.clone() }.check().is_err());
        assert!(
            Body { created: now - MAX_AGE - 1, ..b.clone() }
                .check()
                .is_err()
        );
        assert!(Body { to: "a".into(), ..b.clone() }.check().is_err());
    }

    #[test]
    fn invites_are_checked() {
        let k = iroh::SecretKey::from_bytes(&[3; 32]);
        let world = k.public().to_string();
        let body = format!("{INVITE_HEADER};world={world};expires={};nonce=abc", crate::saves::now_secs() + 60);
        let sig = hex(&k
            .sign(format!("{INVITE_LABEL}{body}").as_bytes())
            .to_bytes());
        assert_eq!(check_invite(&format!("{body};sig={sig}"), &world).unwrap(), "abc");
        assert!(check_invite(&format!("{body};sig={sig}"), "another").is_err());
        assert!(check_invite(&format!("{};sig={sig}", body.replace("nonce=abc", "nonce=abd")), &world).is_err());
        let old = format!("{INVITE_HEADER};world={world};expires=1;nonce=abc");
        let sig = hex(&k.sign(format!("{INVITE_LABEL}{old}").as_bytes()).to_bytes());
        assert!(check_invite(&format!("{old};sig={sig}"), &world).is_err());
    }

    #[test]
    fn signatures_bind_the_body() {
        let k = iroh::SecretKey::from_bytes(&[7; 32]);
        let b = Body { from: k.public().to_string(), to: "b".into(), created: 1, nonce: "0123456789abcdef".into(), note: String::new() };
        let sig = hex(&k.sign(&b.signed()).to_bytes());
        assert!(verify(&b.from, &b.signed(), &sig));
        assert!(!verify(&b.from, &Body { note: "x".into(), ..b.clone() }.signed(), &sig));
        let r = Revoke { link: b.id(), by: b.from.clone(), created: 2, sig: String::new() };
        let rs = hex(&k.sign(&r.signed()).to_bytes());
        assert!(verify(&r.by, &r.signed(), &rs) && !verify(&r.by, &b.signed(), &rs));
    }
}
