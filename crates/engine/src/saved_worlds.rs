//! Worlds this player has joined, to join again (chiacraft plan, phase 3): kept in
//! `worlds.txt` in the run folder, one world per line, and asked for their status (online,
//! players, owner) while the list is shown.
//!
//! A line is `<world key>\t<address>\t<name>\t<owner identity>\t<last seen>\t<owner's inbox>`: the
//! address keeps the hints last used (relay, direct addresses), the name is the one the world gives
//! itself (not checked by anyone), the owner is the identity whose certificate for the world's key
//! checked out, and the inbox is the owner's account endpoint. A world that stops answering is
//! looked up there: if its owner gave it a new address (`friends`), the entry follows it.

use std::path::{Path, PathBuf};

use parking_lot::Mutex;
use rustc_hash::FxHashMap;

use crate::net::p2p::Status;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedWorld {
    pub key: String,
    pub address: String,
    pub name: String,
    pub owner: String,
    /// Seconds since 1970 when it last answered, or 0.
    pub last_seen: u64,
    /// The owner's account endpoint, from the world's status (vouched for by the owner).
    pub inbox: String,
}

fn path(run_dir: &Path) -> PathBuf {
    run_dir.join("worlds.txt")
}

fn clean(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).take(256).collect()
}

pub fn load(run_dir: &Path) -> Vec<SavedWorld> {
    std::fs::read_to_string(path(run_dir))
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            let mut f = l.split('\t');
            let key = f.next()?.to_string();
            let address = f.next()?.to_string();
            let name = f.next().unwrap_or("").to_string();
            let owner = f.next().unwrap_or("").to_string();
            let last_seen = f.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            let inbox = f.next().unwrap_or("").to_string();
            (!key.is_empty()).then_some(SavedWorld { key, address, name, owner, last_seen, inbox })
        })
        .collect()
}

fn save(run_dir: &Path, worlds: &[SavedWorld]) {
    let mut s = String::from("# Worlds this player has joined (chiacraft): key, address, name, owner, last seen, owner's inbox.\n");
    for w in worlds {
        s.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\n", clean(&w.key), clean(&w.address), clean(&w.name), clean(&w.owner), w.last_seen, clean(&w.inbox)));
    }
    if let Err(e) = std::fs::write(path(run_dir), s) {
        log::warn!("saving the worlds list: {e}");
    }
}

/// Remembers the world at `address` (just joined), first in the list; keeps what was known of it.
pub fn remember(run_dir: &Path, address: &str) {
    let Some(p) = crate::world_address::parse(address) else { return };
    let mut worlds = load(run_dir);
    let old = worlds.iter().position(|w| w.key == p.key).map(|i| worlds.remove(i));
    let mut w = old.unwrap_or(SavedWorld { key: p.key.clone(), address: String::new(), name: String::new(), owner: String::new(), last_seen: 0, inbox: String::new() });
    w.address = address.trim().to_string();
    if !p.name.is_empty() {
        w.name = p.name;
    }
    w.last_seen = crate::saves::now_secs();
    worlds.insert(0, w);
    save(run_dir, &worlds);
    // Just joined, so it answers now: learn its owner and their inbox while it does, so the entry
    // can follow the world if it moves.
    if let Some(settings) = SETTINGS.lock().clone() {
        let (rx, dir, key) = (crate::net::p2p::ping(address, &settings), run_dir.to_path_buf(), p.key);
        std::thread::spawn(move || {
            if let Ok(Ok(st)) = rx.recv_timeout(std::time::Duration::from_secs(15)) {
                let mut worlds = load(&dir);
                if let Some(w) = worlds.iter_mut().find(|w| w.key == key) {
                    w.name = st.name;
                    if !st.owner.is_empty() {
                        w.owner = st.owner;
                    }
                    if !st.inbox.is_empty() {
                        w.inbox = st.inbox;
                    }
                }
                save(&dir, &worlds);
            }
        });
    }
}

/// The settings the background questions go out with (relays); set when the game starts.
pub fn set_settings(settings: &crate::settings::Settings) {
    *SETTINGS.lock() = Some(settings.clone());
}

pub fn forget(run_dir: &Path, key: &str) {
    let mut worlds = load(run_dir);
    worlds.retain(|w| w.key != key);
    save(run_dir, &worlds);
    PINGS.lock().remove(key);
}

/// What is known of a world's status while the list is shown.
pub enum Ping {
    Asking(crossbeam_channel::Receiver<Result<Status, String>>, std::time::Instant),
    Online(Status, std::time::Instant),
    Offline(std::time::Instant),
    /// Not answering: asking its owner whether it moved.
    Following(crossbeam_channel::Receiver<Result<crate::friends::Answer, String>>, std::time::Instant),
}

/// The settings pings go out with (relays), kept for the follow-up questions.
static SETTINGS: Mutex<Option<crate::settings::Settings>> = Mutex::new(None);

static PINGS: Mutex<FxHashMap<String, Ping>> = Mutex::new(FxHashMap::with_hasher(rustc_hash::FxBuildHasher));

/// The world's name and owner as shown: what the list had, then what the world just answered.
pub fn title_line(w: &SavedWorld) -> (String, Vec<crate::ui::text::Arg>) {
    use crate::ui::text::Arg;
    let (mut name, mut owner) = (w.name.clone(), w.owner.clone());
    if let Some(Ping::Online(st, _)) = PINGS.lock().get(&w.key) {
        name = st.name.clone();
        if !st.owner.is_empty() {
            owner = st.owner.clone();
        }
    }
    let short = |s: &str| s.chars().take(12).collect::<String>();
    let name = if name.is_empty() { short(&w.key) } else { name };
    let owner = if owner.is_empty() { "?".to_string() } else { short(&owner) };
    ("multiplayer.saved_world".into(), vec![Arg::Text(name), Arg::Text(owner)])
}

/// Asks every saved world for its status, unless it was asked in the last few seconds.
pub fn ping_all(run_dir: &Path, settings: &crate::settings::Settings) {
    *SETTINGS.lock() = Some(settings.clone());
    let mut pings = PINGS.lock();
    for w in load(run_dir) {
        let fresh = match pings.get(&w.key) {
            Some(Ping::Asking(..) | Ping::Following(..)) => true,
            Some(Ping::Online(_, t) | Ping::Offline(t)) => t.elapsed().as_secs() < 10,
            None => false,
        };
        if !fresh {
            pings.insert(w.key.clone(), Ping::Asking(crate::net::p2p::ping(&w.address, settings), std::time::Instant::now()));
        }
    }
}

/// The status line of saved world `key`, as a lang key and its arguments; an answer that just
/// arrived updates the list (name, owner, last seen).
pub fn status_line(run_dir: &Path, key: &str) -> (String, Vec<crate::ui::text::Arg>) {
    use crate::ui::text::Arg;
    let mut pings = PINGS.lock();
    if let Some(Ping::Asking(rx, _)) = pings.get(key)
        && let Ok(answer) = rx.try_recv()
    {
        let now = std::time::Instant::now();
        match answer {
            Ok(st) => {
                let mut worlds = load(run_dir);
                if let Some(w) = worlds.iter_mut().find(|w| w.key == key) {
                    w.name = st.name.clone();
                    if !st.owner.is_empty() {
                        w.owner = st.owner.clone();
                    }
                    if !st.inbox.is_empty() {
                        w.inbox = st.inbox.clone();
                    }
                    w.last_seen = crate::saves::now_secs();
                }
                save(run_dir, &worlds);
                pings.insert(key.to_string(), Ping::Online(st, now));
            }
            Err(e) => {
                log::debug!("world {}: {e}", &key[..key.len().min(12)]);
                // Its owner may have given it a new address: ask them, if we know where.
                let w = load(run_dir).into_iter().find(|w| w.key == key);
                let settings = SETTINGS.lock().clone();
                match (w, settings) {
                    (Some(w), Some(settings)) if !w.inbox.is_empty() && !w.owner.is_empty() => {
                        pings.insert(key.to_string(), Ping::Following(crate::net::p2p::ask_worlds(&w.inbox, &w.owner, &settings), now));
                    }
                    _ => {
                        pings.insert(key.to_string(), Ping::Offline(now));
                    }
                }
            }
        }
    }
    if let Some(Ping::Following(rx, _)) = pings.get(key)
        && let Ok(answer) = rx.try_recv()
    {
        let mut worlds = load(run_dir);
        let moved_to = worlds.iter().find(|w| w.key == key).and_then(|w| answer.ok().and_then(|a| crate::friends::follow(&a.moved, key, &w.owner)));
        match moved_to {
            Some(to) => {
                if let Some(w) = worlds.iter_mut().find(|w| w.key == key) {
                    log::info!("'{}' moved to a new address; following it", w.name);
                    w.address = crate::world_address::address(&to, &w.name, None, &[]);
                    w.key = to;
                }
                save(run_dir, &worlds);
                pings.remove(key);
            }
            None => {
                pings.insert(key.to_string(), Ping::Offline(std::time::Instant::now()));
            }
        }
    }
    match pings.get(key) {
        Some(Ping::Online(st, _)) => ("multiplayer.status.online".into(), vec![Arg::Text(st.players.to_string()), Arg::Text(st.packages.to_string())]),
        Some(Ping::Offline(_)) => {
            let seen = load(run_dir).into_iter().find(|w| w.key == key).map_or(0, |w| w.last_seen);
            if seen == 0 {
                ("multiplayer.status.offline".into(), Vec::new())
            } else {
                ("multiplayer.status.last_seen".into(), vec![Arg::Text(ago(crate::saves::now_secs().saturating_sub(seen)))])
            }
        }
        Some(Ping::Following(..)) => ("multiplayer.status.following".into(), Vec::new()),
        _ => ("multiplayer.status.asking".into(), Vec::new()),
    }
}

fn ago(secs: u64) -> String {
    match secs {
        0..120 => "just now".into(),
        120..7200 => format!("{} minutes ago", secs / 60),
        7200..172_800 => format!("{} hours ago", secs / 3600),
        _ => format!("{} days ago", secs / 86_400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remember_and_forget() {
        let dir = std::env::temp_dir().join(format!("chiacraft-saved-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        remember(&dir, "chiacraft://w/aaa?n=Ruby%20Hills");
        remember(&dir, "bbb;https://relay.example/;1.2.3.4:5");
        remember(&dir, "chiacraft://w/aaa");
        let w = load(&dir);
        assert_eq!(w.iter().map(|w| w.key.as_str()).collect::<Vec<_>>(), ["aaa", "bbb"]);
        assert_eq!(w[0].name, "Ruby Hills");
        forget(&dir, "aaa");
        assert_eq!(load(&dir).len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
