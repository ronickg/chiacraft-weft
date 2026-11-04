//! Who may enter a world, and what a newcomer may do there (`notes/PLAN.md`, phase 4, slices 1-3):
//! the owner's rules, bans, and ban lists other people keep. Nothing here needs the chain; DID
//! rules come with the DID slices. Every gate is the owner's choice: a new world is open, with no
//! probation (rule 2).
//!
//! In the world's folder:
//! - `admission.json`: [`Rules`]: open or allow list only, the allow list, probation, and the
//!   ban lists the world follows;
//! - `bans.json`: [`Bans`], by login key, identity or DID;
//! - `banlists/<maintainer>.json`: the ban lists it follows, as fetched (each signed by its
//!   maintainer, checked on every read), and its own list once the owner publishes it.
//!
//! Both files are read again when they change, so an edit takes effect at the next join with no
//! restart. The server keeps a [`Gate`] over them.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};

use crate::ui::text::{Arg, Msg};

/// Raised whenever a world's rules, bans or lists are saved: the Admission screen shows them anew.
pub static GENERATION: AtomicU64 = AtomicU64::new(0);

/// The world this game runs (its folder), for the Admission screen.
static LOCAL_WORLD: parking_lot::Mutex<Option<PathBuf>> = parking_lot::Mutex::new(None);

pub fn set_local_world(dir: Option<PathBuf>) {
    *LOCAL_WORLD.lock() = dir;
}

pub fn local_world() -> Option<PathBuf> {
    LOCAL_WORLD.lock().clone()
}

pub const RULES_FILE: &str = "admission.json";
pub const BANS_FILE: &str = "bans.json";
pub const LISTS_DIR: &str = "banlists";
/// The most entries a ban list may hold, and the most bytes its file may have.
pub const MAX_LIST_ENTRIES: usize = 10_000;
pub const MAX_LIST_BYTES: usize = 2 << 20;
/// The most ban lists a world follows.
pub const MAX_LISTS: usize = 16;

/// What an entry names a player by.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// The login key: the network key the connection proved.
    Key,
    /// The account's identity key (hex), which vouched for the login key.
    Identity,
    /// A DID (`did:chia:1…`) the player proved on joining.
    Did,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Key => "key",
            Kind::Identity => "identity",
            Kind::Did => "did",
        }
    }
}

/// Whom a value names: a DID by its prefix, an identity key by its length (48 bytes, hex), any
/// other word a login key.
pub fn kind_of(value: &str) -> Kind {
    if value.starts_with("did:chia:1") {
        Kind::Did
    } else if value.len() == 96 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
        Kind::Identity
    } else {
        Kind::Key
    }
}

/// A player on the allow list.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Allowed {
    pub kind: Kind,
    pub value: String,
    /// The name they had when added (for the screen; never checked).
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub added: u64,
}

/// What a newcomer may not do, and for how long.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Probation {
    /// Hours of play in this world before it ends; 0: no probation.
    #[serde(default)]
    pub hours: f32,
    /// No breaking or placing blocks.
    #[serde(default = "yes")]
    pub build: bool,
    /// No opening containers (chests, furnaces: anything a block keeps).
    #[serde(default = "yes")]
    pub containers: bool,
}

fn yes() -> bool {
    true
}

impl Default for Probation {
    fn default() -> Self {
        Self { hours: 0.0, build: true, containers: true }
    }
}

impl Probation {
    pub fn on(&self) -> bool {
        self.hours > 0.0 && (self.build || self.containers)
    }
}

/// A ban list the world follows: whose, and where to get it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Following {
    /// The maintainer's identity key, hex: only lists it signed count. Empty: the owner of the
    /// first world in `from`, as that world's certificate says (learned when fetching, kept in
    /// `banlists/owners.json`).
    #[serde(default)]
    pub maintainer: String,
    /// Where it is served: world addresses (`chiacraft://w/…` or a bare world key).
    #[serde(default)]
    pub from: Vec<String>,
    /// Its name as last fetched.
    #[serde(default)]
    pub name: String,
}

/// The owner's rules (`admission.json`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rules {
    #[serde(default)]
    pub v: u32,
    /// Only players on the allow list get in.
    #[serde(default)]
    pub only_listed: bool,
    #[serde(default)]
    pub allow: Vec<Allowed>,
    #[serde(default)]
    pub probation: Probation,
    #[serde(default)]
    pub lists: Vec<Following>,
    /// Only players who prove a DID on chain get in (phase 4, slice 6).
    #[serde(default)]
    pub did_required: bool,
    /// ... and only with a DID at least this many days old (0: any age). Implies a DID.
    #[serde(default)]
    pub did_min_days: u32,
}

impl Rules {
    /// Whether a join must wait for its DID to be checked on chain.
    pub fn needs_did(&self) -> bool {
        self.did_required || self.did_min_days > 0
    }
}

/// One ban.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ban {
    pub kind: Kind,
    pub value: String,
    /// The name the player had (for the screen and the refusal; never checked).
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub reason: String,
    /// Who banned (a name).
    #[serde(default)]
    pub by: String,
    /// Unix seconds.
    #[serde(default)]
    pub at: u64,
    /// Unix seconds it ends at; none: for good.
    #[serde(default)]
    pub until: Option<u64>,
}

impl Ban {
    pub fn active(&self, now: u64) -> bool {
        self.until.is_none_or(|u| now < u)
    }
}

/// The world's bans (`bans.json`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Bans {
    #[serde(default)]
    pub v: u32,
    #[serde(default)]
    pub bans: Vec<Ban>,
}

/// An entry of a shared ban list.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListEntry {
    pub kind: Kind,
    pub value: String,
    #[serde(default)]
    pub reason: String,
}

/// A ban list someone keeps for others to follow, signed by their identity key. Any copy is as
/// good as the original: the signature covers everything, and a higher `seq` replaces a lower.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BanList {
    pub v: u32,
    /// The maintainer's identity key, hex.
    pub maintainer: String,
    pub name: String,
    /// Raised by one with every new version.
    pub seq: u64,
    /// Unix seconds.
    pub updated: u64,
    pub entries: Vec<ListEntry>,
    /// The maintainer's signature of [`BanList::text`], hex.
    pub signature: String,
}

impl BanList {
    /// What the maintainer signs: every field but the signature, one line each, the entries as
    /// JSON so no reason can break a line.
    pub fn text(&self) -> String {
        let mut t = format!(
            "chiacraft ban list v1\nmaintainer {}\nname {}\nseq {}\nupdated {}\n",
            self.maintainer,
            serde_json::to_string(&self.name).unwrap_or_default(),
            self.seq,
            self.updated
        );
        for e in &self.entries {
            t.push_str(&serde_json::to_string(&(e.kind, &e.value, &e.reason)).unwrap_or_default());
            t.push('\n');
        }
        t
    }

    /// The list signed by the open account, which becomes its maintainer.
    pub fn sign(name: &str, seq: u64, entries: Vec<ListEntry>) -> Result<BanList, String> {
        let session = chiacraft_account::current().ok_or("no account is open")?;
        let mut list = BanList {
            v: 1,
            maintainer: session.identity_hex(),
            name: name.chars().take(64).collect(),
            seq,
            updated: crate::saves::now_secs(),
            entries,
            signature: String::new(),
        };
        list.signature = session.sign_text(&list.text())?;
        Ok(list)
    }

    /// Whether it is well formed and its maintainer signed it.
    pub fn check(&self) -> Result<(), String> {
        if self.v != 1 {
            return Err(format!("a ban list of version {}, which this game can't read", self.v));
        }
        if self.entries.len() > MAX_LIST_ENTRIES {
            return Err("a ban list with too many entries".into());
        }
        if !chiacraft_account::check_text(&self.maintainer, &self.text(), &self.signature) {
            return Err("a ban list its maintainer didn't sign".into());
        }
        Ok(())
    }

    /// Read and check a list from JSON.
    pub fn parse(bytes: &[u8]) -> Result<BanList, String> {
        if bytes.len() > MAX_LIST_BYTES {
            return Err("a ban list too big".into());
        }
        let list: BanList = serde_json::from_slice(bytes).map_err(|e| format!("a ban list this game can't read: {e}"))?;
        list.check()?;
        Ok(list)
    }
}

/// Who is joining, as the connection and the account proved it.
#[derive(Clone, Debug, Default)]
pub struct Who {
    pub name: String,
    /// The login key.
    pub key: String,
    /// The identity that vouched for the login key.
    pub identity: Option<String>,
    /// A DID the player proved, checked on chain (`did:chia:1…`).
    pub did: Option<String>,
    /// That DID's age in days, as the chain says.
    pub did_days: Option<u32>,
}

impl Who {
    pub fn is(&self, kind: Kind, value: &str) -> bool {
        match kind {
            Kind::Key => self.key == value,
            Kind::Identity => self.identity.as_deref() == Some(value),
            Kind::Did => self.did.as_deref() == Some(value),
        }
    }
}

/// The answer to a join.
#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    Admit,
    /// Refused, with the reason to show the player.
    Refuse(Msg),
}

fn reason_arg(reason: &str) -> Arg {
    if reason.is_empty() { Arg::Key("admission.no_reason".into()) } else { Arg::Text(reason.to_string()) }
}

/// The verdict on `who`. `trusted`: the owner or an operator, who always get in. `lists`: the
/// ban lists the world follows ([`Gate`] holds those). Bans come first, the world's own and then
/// the lists it follows; a player on the allow list is never refused by
/// a list the world follows (its own word wins), nor by "only the allow list".
pub fn verdict(rules: &Rules, bans: &Bans, lists: &[BanList], who: &Who, trusted: bool, now: u64) -> Verdict {
    if trusted {
        return Verdict::Admit;
    }
    if let Some(b) = bans
        .bans
        .iter()
        .find(|b| b.active(now) && who.is(b.kind, &b.value))
    {
        return Verdict::Refuse(match b.until {
            Some(until) => {
                let hours = until.saturating_sub(now).div_ceil(3600).max(1);
                Msg::new("admission.banned_for", vec![Arg::Int(hours as i64), reason_arg(&b.reason)])
            }
            None => Msg::new("admission.banned", vec![reason_arg(&b.reason)]),
        });
    }
    let listed = rules.allow.iter().any(|a| who.is(a.kind, &a.value));
    if listed {
        return Verdict::Admit;
    }
    for list in lists {
        if let Some(e) = list.entries.iter().find(|e| who.is(e.kind, &e.value)) {
            return Verdict::Refuse(Msg::new("admission.banned_by_list", vec![Arg::Text(list.name.clone()), reason_arg(&e.reason)]));
        }
    }
    if rules.only_listed {
        return Verdict::Refuse(Msg::new("admission.not_listed", Vec::new()));
    }
    if rules.needs_did() && who.did.is_none() {
        let key = if rules.did_min_days > 0 { "admission.did_required_days" } else { "admission.did_required" };
        return Verdict::Refuse(Msg::new(key, vec![Arg::Int(rules.did_min_days as i64)]));
    }
    if rules.did_min_days > 0 && who.did_days.unwrap_or(0) < rules.did_min_days {
        return Verdict::Refuse(Msg::new(
            "admission.did_too_young",
            vec![Arg::Int(rules.did_min_days as i64), Arg::Int(who.did_days.unwrap_or(0) as i64)],
        ));
    }
    Verdict::Admit
}

/// The probation `who` is on, if any: none for the owner, operators, players on the allow list,
/// players the owner let off, or once they have played `hours` in the world.
pub fn probation_for(rules: &Rules, who: &Who, trusted: bool, played: Duration, lifted: bool) -> Option<Probation> {
    let p = &rules.probation;
    if trusted || lifted || !p.on() || rules.allow.iter().any(|a| who.is(a.kind, &a.value)) {
        return None;
    }
    (played.as_secs_f32() < p.hours * 3600.0).then(|| p.clone())
}

/// `30m`, `12h`, `7d`, `2w`: a duration in seconds.
pub fn parse_duration(text: &str) -> Option<u64> {
    let text = text.trim();
    let (n, unit) = text.split_at(text.find(|c: char| !c.is_ascii_digit())?);
    let n: u64 = n.parse().ok()?;
    let unit = match unit {
        "m" | "min" => 60,
        "h" => 3600,
        "d" => 86_400,
        "w" => 7 * 86_400,
        _ => return None,
    };
    n.checked_mul(unit).filter(|s| *s > 0)
}

/// A short summary of the rules for the world's status answer (lang keys, shown before joining):
/// `admission.summary.listed`, `admission.summary.probation` (hours).
pub fn summary(rules: &Rules) -> Vec<(String, i64)> {
    let mut out = Vec::new();
    if rules.only_listed {
        out.push(("admission.summary.listed".to_string(), 0));
    }
    if rules.did_min_days > 0 {
        out.push(("admission.summary.did_days".to_string(), rules.did_min_days as i64));
    } else if rules.did_required {
        out.push(("admission.summary.did".to_string(), 0));
    }
    if rules.probation.on() {
        out.push(("admission.summary.probation".to_string(), rules.probation.hours.ceil() as i64));
    }
    out
}

fn read_json<T: for<'a> Deserialize<'a> + Default>(path: &Path) -> T {
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|e| {
            log::warn!("{} can't be read, so it counts as empty: {e}", path.display());
            T::default()
        }),
        Err(_) => T::default(),
    }
}

/// Whole or not at all.
fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("new");
    std::fs::write(&tmp, text)
        .and_then(|_| std::fs::rename(&tmp, path))
        .map_err(|e| format!("saving {}: {e}", path.display()))?;
    GENERATION.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

pub fn load_rules(world: &Path) -> Rules {
    read_json(&world.join(RULES_FILE))
}

pub fn save_rules(world: &Path, rules: &Rules) -> Result<(), String> {
    write_json(&world.join(RULES_FILE), rules)
}

pub fn load_bans(world: &Path) -> Bans {
    read_json(&world.join(BANS_FILE))
}

pub fn save_bans(world: &Path, bans: &Bans) -> Result<(), String> {
    write_json(&world.join(BANS_FILE), bans)
}

fn list_path(world: &Path, maintainer: &str) -> Option<PathBuf> {
    // A maintainer is an identity key, hex: nothing else becomes a file name.
    (kind_of(maintainer) == Kind::Identity).then(|| world.join(LISTS_DIR).join(format!("{maintainer}.json")))
}

const OWNERS_FILE: &str = "owners.json";

/// The world key in an address (`chiacraft://w/…` or a bare key).
pub fn world_key(address: &str) -> Option<String> {
    crate::world_address::parse(address).map(|p| p.key)
}

/// The owners of the worlds whose lists are followed by address: world key → identity, as each
/// world's certificate said when its list was fetched.
pub fn owners(world: &Path) -> std::collections::BTreeMap<String, String> {
    read_json(&world.join(LISTS_DIR).join(OWNERS_FILE))
}

/// Remembers that the world with key `key` is owned by `identity` (its certificate checked).
pub fn set_owner(world: &Path, key: &str, identity: &str) -> Result<(), String> {
    let mut o = owners(world);
    if o.get(key).map(String::as_str) == Some(identity) {
        return Ok(());
    }
    o.insert(key.to_string(), identity.to_string());
    write_json(&world.join(LISTS_DIR).join(OWNERS_FILE), &o)
}

/// Whose list `f` is: its maintainer, or the owner of its world as last learned.
pub fn maintainer_of(f: &Following, owners: &std::collections::BTreeMap<String, String>) -> Option<String> {
    if !f.maintainer.is_empty() {
        return Some(f.maintainer.clone());
    }
    let key = world_key(f.from.first()?)?;
    owners.get(&key).cloned()
}

/// The copy of `maintainer`'s list this world holds, checked.
pub fn load_list(world: &Path, maintainer: &str) -> Option<BanList> {
    let bytes = std::fs::read(list_path(world, maintainer)?).ok()?;
    match BanList::parse(&bytes) {
        Ok(l) if l.maintainer == maintainer => Some(l),
        Ok(_) => None,
        Err(e) => {
            log::warn!("the ban list of {}: {e}", &maintainer[..12]);
            None
        }
    }
}

/// Keeps `list` if it checks out and is newer than the copy held: whether it was kept.
pub fn store_list(world: &Path, list: &BanList) -> Result<bool, String> {
    list.check()?;
    let path = list_path(world, &list.maintainer).ok_or("a ban list whose maintainer isn't an identity")?;
    if load_list(world, &list.maintainer).is_some_and(|held| held.seq >= list.seq) {
        return Ok(false);
    }
    write_json(&path, list)?;
    Ok(true)
}

/// The lists this world holds copies of (its own published one too), raw, for serving.
pub fn held_list(world: &Path, maintainer: &str) -> Option<Vec<u8>> {
    let bytes = std::fs::read(list_path(world, maintainer)?).ok()?;
    (bytes.len() <= MAX_LIST_BYTES).then_some(bytes)
}

/// The bans that keep the login key `key` out of the world in `world` (its own and the lists it
/// follows), checked at the door: before the world serves the joining game anything, when only
/// the connection's key is known. The rest of the rules wait for the join, which proves the
/// identity and the DID.
pub fn at_the_door(world: &Path, key: &str) -> Option<Msg> {
    let gate = Gate::new(Some(world.to_path_buf()));
    let rules = Rules { only_listed: false, did_required: false, did_min_days: 0, ..gate.rules.clone() };
    let who = Who { key: key.to_string(), ..Default::default() };
    match verdict(&rules, &gate.bans, &gate.lists, &who, false, crate::saves::now_secs()) {
        Verdict::Refuse(m) => Some(m),
        Verdict::Admit => None,
    }
}

/// A refusal as one line of text for a reply that carries only text (the hello's): the lang key
/// and its text arguments, separated by U+001F. [`refusal_from_text`] reads it back.
pub fn refusal_text(m: &Msg) -> String {
    let mut parts = vec![m.key.clone()];
    parts.extend(m.args.iter().map(|a| match a {
        Arg::Text(t) => t.replace('\u{1f}', " "),
        Arg::Int(n) => n.to_string(),
        Arg::Key(k) => format!("@{k}"),
        _ => String::new(),
    }));
    parts.join("\u{1f}")
}

/// A refusal written by [`refusal_text`], as a message to show; none for any other text. Only
/// `admission.` keys, so a world can't make the game say anything else.
pub fn refusal_from_text(text: &str) -> Option<Msg> {
    let mut parts = text.split('\u{1f}');
    let key = parts.next()?;
    if !key.starts_with("admission.")
        || !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
    {
        return None;
    }
    let args = parts
        .map(|p| match (p.strip_prefix('@'), p.parse::<i64>()) {
            (Some(k), _) if k.starts_with("admission.") => Arg::Key(k.to_string()),
            (_, Ok(n)) => Arg::Int(n),
            _ => Arg::Text(p.chars().take(200).collect()),
        })
        .collect();
    Some(Msg::new(key, args))
}

/// Whether the world follows a list it holds no copy of yet (fetched again soon, not daily).
pub fn lists_missing(world: &Path) -> bool {
    let rules = load_rules(world);
    let owners = owners(world);
    rules
        .lists
        .iter()
        .take(MAX_LISTS)
        .any(|f| maintainer_of(f, &owners).is_none_or(|m| load_list(world, &m).is_none()))
}

/// The world's own bans as a list its owner signs: the bans for good (a timed ban isn't for
/// others to copy), a sequence number one above the last published.
pub fn publish(world: &Path, name: &str) -> Result<BanList, String> {
    let bans = load_bans(world);
    let entries: Vec<ListEntry> = bans
        .bans
        .iter()
        .filter(|b| b.until.is_none())
        .map(|b| ListEntry { kind: b.kind, value: b.value.clone(), reason: b.reason.clone() })
        .collect();
    let me = chiacraft_account::current()
        .ok_or("no account is open")?
        .identity_hex();
    let seq = load_list(world, &me).map_or(1, |l| l.seq + 1);
    let list = BanList::sign(name, seq, entries)?;
    store_list(world, &list)?;
    Ok(list)
}

/// The rules, bans and followed lists of one world, read again when their files change (looked
/// at no more than every two seconds). The server's view of `admission.json` and `bans.json`.
pub struct Gate {
    dir: Option<PathBuf>,
    pub rules: Rules,
    pub bans: Bans,
    pub lists: Vec<BanList>,
    stamps: Vec<Option<SystemTime>>,
    looked: Option<Instant>,
    /// Read again since [`Gate::take_changed`] last asked.
    changed: bool,
}

fn stamp(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

impl Gate {
    /// The gate of the world in `dir`; none: a world that isn't saved, open to all.
    pub fn new(dir: Option<PathBuf>) -> Gate {
        let mut g = Gate {
            dir,
            rules: Rules::default(),
            bans: Bans::default(),
            lists: Vec::new(),
            stamps: Vec::new(),
            looked: None,
            changed: false,
        };
        g.refresh(true);
        g
    }

    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    fn files(&self, dir: &Path) -> Vec<PathBuf> {
        let mut files = vec![dir.join(RULES_FILE), dir.join(BANS_FILE), dir.join(LISTS_DIR).join(OWNERS_FILE)];
        let owners = owners(dir);
        files.extend(
            self.rules
                .lists
                .iter()
                .filter_map(|f| maintainer_of(f, &owners))
                .filter_map(|m| list_path(dir, &m)),
        );
        files
    }

    /// Read the files again if they changed (`force`: now): whether they did.
    pub fn refresh(&mut self, force: bool) -> bool {
        if !force
            && self
                .looked
                .is_some_and(|t| t.elapsed() < Duration::from_secs(2))
        {
            return false;
        }
        self.looked = Some(Instant::now());
        let Some(dir) = self.dir.clone() else { return false };
        let stamps: Vec<_> = self.files(&dir).iter().map(|f| stamp(f)).collect();
        if !force && stamps == self.stamps {
            return false;
        }
        self.rules = load_rules(&dir);
        self.bans = load_bans(&dir);
        let owners = owners(&dir);
        self.lists = self
            .rules
            .lists
            .iter()
            .take(MAX_LISTS)
            .filter_map(|f| maintainer_of(f, &owners))
            .filter_map(|m| load_list(&dir, &m))
            .collect();
        // The rules may name other lists now.
        self.stamps = self.files(&dir).iter().map(|f| stamp(f)).collect();
        self.changed = true;
        true
    }

    /// Whether the files were read again since the last time this was asked (looking at them
    /// first): whoever is banned now must leave.
    pub fn take_changed(&mut self) -> bool {
        self.refresh(false);
        std::mem::take(&mut self.changed)
    }

    pub fn verdict(&mut self, who: &Who, trusted: bool) -> Verdict {
        self.refresh(false);
        verdict(&self.rules, &self.bans, &self.lists, who, trusted, crate::saves::now_secs())
    }

    pub fn probation(&mut self, who: &Who, trusted: bool, played: Duration, lifted: bool) -> Option<Probation> {
        self.refresh(false);
        probation_for(&self.rules, who, trusted, played, lifted)
    }

    /// Change the rules and save them.
    pub fn edit_rules(&mut self, f: impl FnOnce(&mut Rules)) -> Result<(), String> {
        let dir = self.dir.clone().ok_or("this world isn't saved")?;
        self.refresh(true);
        f(&mut self.rules);
        save_rules(&dir, &self.rules)?;
        self.refresh(true);
        Ok(())
    }

    /// Change the bans and save them.
    pub fn edit_bans(&mut self, f: impl FnOnce(&mut Bans)) -> Result<(), String> {
        let dir = self.dir.clone().ok_or("this world isn't saved")?;
        self.refresh(true);
        f(&mut self.bans);
        save_bans(&dir, &self.bans)?;
        self.refresh(true);
        Ok(())
    }
}

/// Ban `who` (every way it is known: login key, identity, DID), replacing their earlier bans.
pub fn ban_all(bans: &mut Bans, who: &Who, reason: &str, by: &str, until: Option<u64>) {
    let now = crate::saves::now_secs();
    let mut ways = vec![(Kind::Key, who.key.clone())];
    ways.extend(who.identity.clone().map(|i| (Kind::Identity, i)));
    ways.extend(who.did.clone().map(|d| (Kind::Did, d)));
    bans.bans
        .retain(|b| !ways.iter().any(|(k, v)| b.kind == *k && &b.value == v));
    for (kind, value) in ways.into_iter().filter(|(_, v)| !v.is_empty()) {
        bans.bans.push(Ban {
            kind,
            value,
            name: who.name.clone(),
            reason: reason.chars().take(200).collect(),
            by: by.to_string(),
            at: now,
            until,
        });
    }
}

/// Lift every ban matching `what`: a value (key, identity, DID), or the name a player was banned
/// under (every way they were banned). How many were lifted.
pub fn unban(bans: &mut Bans, what: &str) -> usize {
    let before = bans.bans.len();
    bans.bans
        .retain(|b| !(b.value == what || (!b.name.is_empty() && b.name.eq_ignore_ascii_case(what))));
    before - bans.bans.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn who(key: &str, identity: Option<&str>) -> Who {
        Who { name: "bob".into(), key: key.into(), identity: identity.map(str::to_string), did: None, did_days: None }
    }

    #[test]
    fn open_by_default_and_the_allow_list_closes_it() {
        let mut rules = Rules::default();
        let bans = Bans::default();
        let bob = who("k1", Some("id1"));
        assert_eq!(verdict(&rules, &bans, &[], &bob, false, 100), Verdict::Admit);
        rules.only_listed = true;
        assert_eq!(verdict(&rules, &bans, &[], &bob, false, 100), Verdict::Refuse(Msg::new("admission.not_listed", Vec::new())));
        // The owner and operators always get in.
        assert_eq!(verdict(&rules, &bans, &[], &bob, true, 100), Verdict::Admit);
        rules
            .allow
            .push(Allowed { kind: Kind::Identity, value: "id1".into(), name: "bob".into(), added: 0 });
        assert_eq!(verdict(&rules, &bans, &[], &bob, false, 100), Verdict::Admit);
        // Another login key of the same identity is the same player.
        assert_eq!(verdict(&rules, &bans, &[], &who("k2", Some("id1")), false, 100), Verdict::Admit);
        assert!(matches!(verdict(&rules, &bans, &[], &who("k2", None), false, 100), Verdict::Refuse(_)));
    }

    #[test]
    fn bans_by_any_way_a_player_is_known_and_they_end() {
        let rules = Rules::default();
        let mut bans = Bans::default();
        let bob = who("k1", Some("id1"));
        ban_all(&mut bans, &bob, "griefing", "alice", None);
        assert_eq!(bans.bans.len(), 2);
        let Verdict::Refuse(m) = verdict(&rules, &bans, &[], &bob, false, 100) else { panic!("admitted") };
        assert_eq!(m.key, "admission.banned");
        // A new login key with the same identity: still banned.
        assert!(matches!(verdict(&rules, &bans, &[], &who("k9", Some("id1")), false, 100), Verdict::Refuse(_)));
        // Banning again replaces, never doubles.
        ban_all(&mut bans, &bob, "", "alice", Some(1000 + 7200));
        assert_eq!(bans.bans.len(), 2);
        let Verdict::Refuse(m) = verdict(&rules, &bans, &[], &bob, false, 1000) else { panic!("admitted") };
        assert_eq!((m.key.as_str(), &m.args[0]), ("admission.banned_for", &Arg::Int(2)));
        assert_eq!(m.args[1], Arg::Key("admission.no_reason".into()));
        assert_eq!(verdict(&rules, &bans, &[], &bob, false, 1000 + 7200), Verdict::Admit);
        // A ban beats the allow list; the owner is never banned out.
        let mut listed = Rules::default();
        listed
            .allow
            .push(Allowed { kind: Kind::Key, value: "k1".into(), name: String::new(), added: 0 });
        assert!(matches!(verdict(&listed, &bans, &[], &bob, false, 1000), Verdict::Refuse(_)));
        assert_eq!(verdict(&listed, &bans, &[], &bob, true, 1000), Verdict::Admit);
        // Lifted by name or by value.
        assert_eq!(unban(&mut bans, "BOB"), 2);
        assert!(bans.bans.is_empty());
        ban_all(&mut bans, &bob, "", "alice", None);
        assert_eq!(unban(&mut bans, "id1"), 1);
        assert_eq!(unban(&mut bans, "nobody"), 0);
    }

    #[test]
    fn did_rules_ask_for_a_proven_did_of_some_age() {
        let mut rules = Rules { did_required: true, ..Default::default() };
        let mut bob = who("k1", Some("id1"));
        let Verdict::Refuse(m) = verdict(&rules, &Bans::default(), &[], &bob, false, 0) else { panic!("admitted") };
        assert_eq!(m.key, "admission.did_required");
        bob.did = Some("did:chia:1abc".into());
        bob.did_days = Some(3);
        assert_eq!(verdict(&rules, &Bans::default(), &[], &bob, false, 0), Verdict::Admit);
        rules.did_min_days = 30;
        let Verdict::Refuse(m) = verdict(&rules, &Bans::default(), &[], &bob, false, 0) else { panic!("admitted") };
        assert_eq!((m.key.as_str(), &m.args[1]), ("admission.did_too_young", &Arg::Int(3)));
        bob.did_days = Some(31);
        assert_eq!(verdict(&rules, &Bans::default(), &[], &bob, false, 0), Verdict::Admit);
        // The age rule alone asks for a DID too; the allow list and the owner skip both.
        let young = Rules { did_min_days: 7, ..Default::default() };
        assert!(young.needs_did());
        assert!(matches!(verdict(&young, &Bans::default(), &[], &who("k2", None), false, 0), Verdict::Refuse(_)));
        assert_eq!(verdict(&young, &Bans::default(), &[], &who("k2", None), true, 0), Verdict::Admit);
        // A banned DID refuses whoever proves it.
        let mut bans = Bans::default();
        ban_all(&mut bans, &Who { did: Some("did:chia:1abc".into()), ..Default::default() }, "", "alice", None);
        let other = Who { key: "k9".into(), did: Some("did:chia:1abc".into()), did_days: Some(99), ..Default::default() };
        assert!(matches!(verdict(&Rules::default(), &bans, &[], &other, false, 0), Verdict::Refuse(_)));
    }

    #[test]
    fn probation_counts_play_and_spares_the_trusted() {
        let mut rules = Rules::default();
        let bob = who("k1", Some("id1"));
        assert!(probation_for(&rules, &bob, false, Duration::ZERO, false).is_none());
        rules.probation.hours = 2.0;
        assert!(probation_for(&rules, &bob, false, Duration::from_secs(3600), false).is_some());
        assert!(probation_for(&rules, &bob, false, Duration::from_secs(7200), false).is_none());
        assert!(probation_for(&rules, &bob, true, Duration::ZERO, false).is_none());
        assert!(probation_for(&rules, &bob, false, Duration::ZERO, true).is_none());
        rules.probation.build = false;
        rules.probation.containers = false;
        assert!(probation_for(&rules, &bob, false, Duration::ZERO, false).is_none());
        rules.probation.containers = true;
        rules
            .allow
            .push(Allowed { kind: Kind::Key, value: "k1".into(), name: String::new(), added: 0 });
        assert!(probation_for(&rules, &bob, false, Duration::ZERO, false).is_none());
    }

    #[test]
    fn refusals_survive_a_text_reply() {
        let m = Msg::new("admission.banned_for", vec![Arg::Int(3), Arg::Text("grief\u{1f}ing".into())]);
        let back = refusal_from_text(&refusal_text(&m)).unwrap();
        assert_eq!(back, Msg::new("admission.banned_for", vec![Arg::Int(3), Arg::Text("grief ing".into())]));
        let m = Msg::new("admission.banned", vec![Arg::Key("admission.no_reason".into())]);
        assert_eq!(refusal_from_text(&refusal_text(&m)), Some(m));
        assert_eq!(refusal_from_text("protocol mismatch: server 7, client 6"), None);
        assert_eq!(refusal_from_text("menu.quit\u{1f}x"), None);
        let at = Msg::new("admission.banned", vec![Arg::Key("menu.quit".into())]);
        assert!(matches!(refusal_from_text(&refusal_text(&at)).unwrap().args[0], Arg::Text(_)));
    }

    #[test]
    fn durations_and_kinds() {
        assert_eq!(parse_duration("30m"), Some(1800));
        assert_eq!(parse_duration("12h"), Some(43_200));
        assert_eq!(parse_duration("7d"), Some(604_800));
        assert_eq!(parse_duration("2w"), Some(1_209_600));
        assert_eq!(parse_duration("0d"), None);
        assert_eq!(parse_duration("d"), None);
        assert_eq!(parse_duration("5y"), None);
        assert_eq!(parse_duration("griefing"), None);
        assert_eq!(kind_of("did:chia:1abc"), Kind::Did);
        assert_eq!(kind_of(&"ab".repeat(48)), Kind::Identity);
        assert_eq!(kind_of("5f3c0d9e1a"), Kind::Key);
    }

    #[test]
    fn rules_files_read_old_and_partial_text() {
        let r: Rules = serde_json::from_str(r#"{"only_listed":true,"probation":{"hours":1}}"#).unwrap();
        assert!(r.only_listed && r.probation.build && r.probation.containers);
        assert_eq!(summary(&r), vec![("admission.summary.listed".into(), 0), ("admission.summary.probation".into(), 1)]);
        let b: Bans = serde_json::from_str(r#"{"bans":[{"kind":"did","value":"did:chia:1x"}]}"#).unwrap();
        assert!(b.bans[0].active(u64::MAX));
    }

    #[test]
    fn shared_lists_are_signed_and_the_allow_list_wins() {
        with_account();
        let entries = vec![ListEntry { kind: Kind::Key, value: "k1".into(), reason: "spam\nbot".into() }];
        let list = BanList::sign("Ronald's list", 1, entries).unwrap();
        list.check().unwrap();
        let bytes = serde_json::to_vec(&list).unwrap();
        assert_eq!(BanList::parse(&bytes).unwrap(), list);
        // Any change breaks it.
        let mut changed = list.clone();
        changed.entries[0].value = "k2".into();
        assert!(changed.check().is_err());
        let mut renamed = list.clone();
        renamed.seq = 2;
        assert!(renamed.check().is_err());

        let bob = who("k1", None);
        let mut rules = Rules::default();
        assert_eq!(verdict(&rules, &Bans::default(), &[], &bob, false, 0), Verdict::Admit);
        let Verdict::Refuse(m) = verdict(&rules, &Bans::default(), std::slice::from_ref(&list), &bob, false, 0) else { panic!("admitted") };
        assert_eq!(m.key, "admission.banned_by_list");
        rules
            .allow
            .push(Allowed { kind: Kind::Key, value: "k1".into(), name: String::new(), added: 0 });
        assert_eq!(verdict(&rules, &Bans::default(), std::slice::from_ref(&list), &bob, false, 0), Verdict::Admit);

        // Kept on disk only when newer.
        let dir = std::env::temp_dir().join(format!("chiacraft-banlists-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(store_list(&dir, &list).unwrap());
        assert!(!store_list(&dir, &list).unwrap());
        assert_eq!(load_list(&dir, &list.maintainer), Some(list.clone()));
        assert!(load_list(&dir, "../../etc").is_none());
        let newer = BanList::sign("Ronald's list", 2, Vec::new()).unwrap();
        assert!(store_list(&dir, &newer).unwrap());
        assert_eq!(load_list(&dir, &list.maintainer).unwrap().seq, 2);
        // Followed by a world's address: its owner's list, once the owner is known.
        let mut f = Following { maintainer: String::new(), from: vec!["chiacraft://w/abcdef0123?n=x".into()], name: String::new() };
        assert_eq!(maintainer_of(&f, &owners(&dir)), None);
        set_owner(&dir, "abcdef0123", &list.maintainer).unwrap();
        assert_eq!(maintainer_of(&f, &owners(&dir)), Some(list.maintainer.clone()));
        f.maintainer = "ab".repeat(48);
        assert_eq!(maintainer_of(&f, &owners(&dir)), Some("ab".repeat(48)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// An account for the tests that sign, in a folder of their own.
    pub(super) fn with_account() {
        let dir = std::env::temp_dir().join(format!("chiacraft-admission-account-{}", std::process::id()));
        chiacraft_account::use_file_store();
        chiacraft_account::open(&dir).expect("a test account");
    }
}
