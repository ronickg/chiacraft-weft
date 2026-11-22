//! Friends and their worlds (chiacraft plan, phase 3): a friend is another player's account (the
//! endpoint their game is reached at, and the identity that vouches for it), added from a world
//! of theirs you joined or a link between your worlds. Asked over `chiacraft/worlds/1`, a game
//! tells its friends which worlds it has and which are open now, and tells anyone where its worlds
//! moved: a world given a new key (a new generation) leaves a notice its owner's identity signs,
//! so saved addresses can follow it. Kept in `friends.json` in the run folder.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::app::{Ctx, Request};
use crate::screens::{BuildCx, Screen, button, frame, label};
use crate::ui::text::Arg;
use crate::ui::{Style, Ui};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Friend {
    /// Their account endpoint (where their game answers).
    pub account: String,
    /// Their identity (hex), which vouches for `account`.
    pub identity: String,
    /// Where they were met: a world of theirs.
    pub met: String,
}

/// A world that moved to a new key, signed by its owner's identity.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Moved {
    pub from: String,
    pub to: String,
    pub generation: u32,
    pub sig: String,
}

#[derive(Default, Serialize, Deserialize)]
struct Store {
    friends: Vec<Friend>,
    /// This player's own worlds' moves.
    moved: Vec<Moved>,
}

struct State {
    path: PathBuf,
    saves: PathBuf,
    store: Store,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

pub fn init(run_dir: &Path, saves: &Path) {
    let path = run_dir.join("friends.json");
    let store = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    *STATE.lock() = Some(State { path, saves: saves.to_path_buf(), store });
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> Option<R> {
    STATE.lock().as_mut().map(f)
}

fn save(s: &State) {
    if let Ok(t) = serde_json::to_string_pretty(&s.store)
        && let Err(e) = std::fs::write(&s.path, t)
    {
        log::warn!("saving friends: {e}");
    }
}

pub fn list() -> Vec<Friend> {
    with(|s| s.store.friends.clone()).unwrap_or_default()
}

pub fn is_friend(account: &str) -> bool {
    with(|s| s.store.friends.iter().any(|f| f.account == account)).unwrap_or(false)
}

/// Whether the player with identity `identity` (hex) is on this player's Friends list.
pub fn is_friend_identity(identity: &str) -> bool {
    !identity.is_empty() && with(|s| s.store.friends.iter().any(|f| f.identity == identity)).unwrap_or(false)
}

/// Friends' worlds as known now (each friend is asked again when the last answer is old): the
/// world's key and name, and the friend's account endpoint (where a link request for it goes).
pub fn known_worlds() -> Vec<(String, String, String)> {
    if let Some(settings) = crate::saved_worlds::settings() {
        ask_all(&settings);
    }
    list()
        .into_iter()
        .filter_map(|f| match worlds_of(&f.account)? {
            Ok(a) => Some((f, a)),
            Err(_) => None,
        })
        .flat_map(|(f, a)| {
            a.worlds
                .into_iter()
                .map(move |w| (w.key, w.name, f.account.clone()))
        })
        .collect()
}

/// Adds a friend: their account endpoint, the identity that vouches for it, and where you met.
pub fn add(account: &str, identity: &str, met: &str) {
    with(|s| {
        if account.is_empty() || identity.is_empty() || s.store.friends.iter().any(|f| f.account == account) {
            return;
        }
        s.store
            .friends
            .push(Friend { account: account.into(), identity: identity.into(), met: met.chars().take(64).collect() });
        save(s);
        log::info!("added a friend (identity {}), met through '{met}'", &identity[..12.min(identity.len())]);
    });
}

pub fn remove(account: &str) {
    with(|s| {
        s.store.friends.retain(|f| f.account != account);
        save(s);
    });
    if let Some(q) = QUERIES.lock().as_mut() {
        q.remove(account);
    }
}

/// A world of this player's, as their friends see it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldEntry {
    pub key: String,
    pub name: String,
    pub cert: String,
    /// Open now: this game hosts it.
    pub open: bool,
}

/// What `chiacraft/worlds/1` answers.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Answer {
    pub worlds: Vec<WorldEntry>,
    pub moved: Vec<Moved>,
}

/// The answer for the account endpoint `peer`: this player's worlds if `peer` is a friend (with
/// `hosted`, the key of the world this game hosts now, marked open), and where worlds moved.
pub fn answer_for(peer: &str, hosted: Option<String>) -> Answer {
    let Some((saves, moved)) = with(|s| (s.saves.clone(), s.store.moved.clone())) else { return Answer::default() };
    let worlds = if is_friend(peer) {
        crate::saves::list(&saves)
            .into_iter()
            .filter(|m| m.key_index.is_some())
            .filter_map(|mut m| {
                let id = crate::world_address::for_world(&mut m).ok()?;
                let key = id.key();
                Some(WorldEntry { open: hosted.as_deref() == Some(key.as_str()), key, name: m.name.clone(), cert: id.cert.text() })
            })
            .collect()
    } else {
        Vec::new()
    };
    Answer { worlds, moved }
}

/// Gives the world in folder `meta` a new key: its address changes, saved addresses follow the
/// notice this player's identity signs, and links to the old key end. Only for a world not open.
pub fn new_address(meta: &mut crate::saves::WorldMeta) -> Result<String, String> {
    let session = chiacraft_account::current().ok_or("no account is open")?;
    let old = crate::world_address::for_world(meta)?.key();
    meta.key_generation += 1;
    meta.save();
    let new = crate::world_address::for_world(meta)?.key();
    let sig = session.world_moved_sig(&old, &new, meta.key_generation)?;
    with(|s| {
        s.store
            .moved
            .push(Moved { from: old.clone(), to: new.clone(), generation: meta.key_generation, sig });
        save(s);
    });
    log::info!("'{}' has a new address: {new} (was {old})", meta.name);
    Ok(new)
}

/// Where world `key` moved, if `owner` (identity hex) signed a notice for it in `moved`.
pub fn follow(moved: &[Moved], key: &str, owner: &str) -> Option<String> {
    let mut at = key.to_string();
    let mut seen = 0;
    while let Some(m) = moved
        .iter()
        .find(|m| m.from == at && chiacraft_account::check_world_moved(owner, &m.from, &m.to, m.generation, &m.sig))
    {
        at = m.to.clone();
        seen += 1;
        if seen > 16 {
            break;
        }
    }
    (at != key).then_some(at)
}

// ------------------------------------------------------------------ asking friends

/// What each friend answered (or is answering), while the Friends screen shows.
enum Query {
    Asking(crossbeam_channel::Receiver<Result<Answer, String>>),
    Answered(Result<Answer, String>, std::time::Instant),
}

static QUERIES: Mutex<Option<rustc_hash::FxHashMap<String, Query>>> = Mutex::new(None);

/// Asks every friend for their worlds, unless asked in the last few seconds.
fn ask_all(settings: &crate::settings::Settings) {
    let mut q = QUERIES.lock();
    let q = q.get_or_insert_with(Default::default);
    for f in list() {
        let fresh = match q.get(&f.account) {
            Some(Query::Asking(_)) => true,
            Some(Query::Answered(_, t)) => t.elapsed().as_secs() < 10,
            None => false,
        };
        if !fresh {
            q.insert(f.account.clone(), Query::Asking(crate::net::p2p::ask_worlds(&f.account, &f.identity, settings)));
        }
    }
}

/// A friend's worlds as known now: `None` while asking.
fn worlds_of(account: &str) -> Option<Result<Answer, String>> {
    let mut q = QUERIES.lock();
    let q = q.as_mut()?;
    if let Some(Query::Asking(rx)) = q.get(account)
        && let Ok(a) = rx.try_recv()
    {
        q.insert(account.to_string(), Query::Answered(a, std::time::Instant::now()));
    }
    match q.get(account)? {
        Query::Answered(a, _) => Some(a.clone()),
        Query::Asking(_) => None,
    }
}

/// Friends' answers that arrived since the screen was built (it shows them again).
pub static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Friends and the worlds they have, to join the open ones.
pub struct Friends;

impl Screen for Friends {
    fn name(&self) -> &str {
        "core:friends"
    }

    fn build(&mut self, ui: &mut Ui, cx: &BuildCx) {
        ask_all(cx.settings);
        let friends = list();
        frame(ui, cx, "friends.title", |ui| {
            let body = ui.container(
                Style::default()
                    .gap(10.0)
                    .w(400.0)
                    .h(0.0)
                    .grow()
                    .min_h(60.0)
                    .scroll(),
                |ui| {
                    if friends.is_empty() {
                        ui.panel("panel", Style::default().pad(8.0).stretch(), |ui| {
                            label(ui, cx, "friends.none", &[], "dim");
                        });
                    }
                    for (i, f) in friends.iter().enumerate() {
                        ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch(), |ui| {
                            ui.container(Style::row().gap(6.0).stretch(), |ui| {
                                label(
                                    ui,
                                    cx,
                                    "friends.friend",
                                    &[Arg::Text(f.identity.chars().take(12).collect()), Arg::Text(f.met.clone())],
                                    "default",
                                );
                                let account = f.account.clone();
                                button(ui, cx, "friends.remove", &[], 80.0, move |c| {
                                    remove(&account);
                                    c.requests.push(Request::Rebuild);
                                });
                            });
                            match worlds_of(&f.account) {
                                None => {
                                    label(ui, cx, "friends.asking", &[], "dim");
                                }
                                Some(Err(_)) => {
                                    label(ui, cx, "friends.away", &[], "dim");
                                }
                                Some(Ok(a)) if a.worlds.is_empty() => {
                                    label(ui, cx, "friends.not_mutual", &[], "dim");
                                }
                                Some(Ok(a)) => {
                                    for (j, w) in a.worlds.iter().enumerate() {
                                        ui.container(Style::row().gap(6.0).stretch(), |ui| {
                                            label(
                                                ui,
                                                cx,
                                                if w.open { "friends.world_open" } else { "friends.world_closed" },
                                                &[Arg::Text(w.name.clone())],
                                                "default",
                                            );
                                            if w.open {
                                                let address = crate::world_address::address(&w.key, &w.name, None, &[]);
                                                let id = ui.button(
                                                    "button",
                                                    Style::default().min_w(60.0).min_h(20.0).pad_xy(6.0, 4.0),
                                                    Rc::new(move |c: &mut Ctx| c.requests.push(Request::Join(address.clone()))),
                                                    |ui| {
                                                        ui.text(cx.t("multiplayer.join", &[]), "button");
                                                    },
                                                );
                                                ui.set_id(id, &format!("join{i}_{j}"));
                                            }
                                        });
                                    }
                                }
                            }
                        });
                    }
                },
            );
            ui.set_id(body, "friends_list");
            button(ui, cx, "menu.back", &[], 200.0, |c| c.requests.push(Request::Close));
        });
    }
}

/// Asks before giving a world a new address.
pub struct ConfirmNewAddress {
    pub world: crate::saves::WorldMeta,
}

impl Screen for ConfirmNewAddress {
    fn name(&self) -> &str {
        "core:confirm_new_address"
    }

    fn build(&mut self, ui: &mut Ui, cx: &BuildCx) {
        let world = self.world.clone();
        frame(ui, cx, "friends.new_address_title", |ui| {
            ui.panel("panel", Style::default().pad(8.0).gap(6.0).w(360.0), |ui| {
                label(ui, cx, "friends.new_address_text", &[Arg::Text(world.name.clone())], "default");
                ui.container(Style::row().gap(6.0), |ui| {
                    let id = button(ui, cx, "friends.new_address_yes", &[], 170.0, move |c| {
                        let mut meta = world.clone();
                        if let Err(e) = new_address(&mut meta) {
                            log::warn!("{e}");
                        }
                        c.requests.push(Request::Close);
                        c.requests.push(Request::Rebuild);
                    });
                    ui.set_id(id, "new_address_yes");
                    button(ui, cx, "menu.back", &[], 170.0, |c| c.requests.push(Request::Close));
                });
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follow_needs_a_signed_notice() {
        let fake = Moved { from: "a".into(), to: "b".into(), generation: 1, sig: "00".into() };
        assert_eq!(follow(&[fake], "a", "ff"), None);
    }
}
