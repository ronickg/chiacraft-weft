//! The sponsor's memory: one append-only JSON-lines file (`sponsor-log.jsonl`) of what it did:
//! each DID launched (for which identity and address, its launcher and coin, its cost), each
//! bundle pushed (its transaction id, the coins it spent, the fee), confirmations and drops. The
//! state (identities served, today's spending, launches per asking endpoint) is rebuilt from it at
//! start, so a restart forgets nothing. Each line is flushed and synced before the request is
//! answered.
//!
//! The askers' endpoint ids are not kept: only a hash of the day and the id, enough to count an
//! endpoint's launches that day.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Seconds in a day: budgets and endpoint limits count per UTC day (`unix seconds / 86400`).
pub const DAY: u64 = 86_400;

/// One line of the log.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    /// A DID launched for a request (pushed; not yet confirmed).
    Sponsored {
        time: u64,
        identity: String,
        address: String,
        launcher: String,
        coin: String,
        cost: u64,
        /// [`client_hash`] of the asker (`ip` in Horizon's log, a hash of its IP address).
        #[serde(alias = "ip")]
        client: String,
        txid: String,
    },
    /// A bundle pushed.
    Bundle { time: u64, txid: String, dids: usize, fee: u64, change: u64, spent: Vec<String> },
    /// The DID's eve coin is on chain.
    Confirmed { time: u64, launcher: String, height: u32 },
    /// Never confirmed: the identity may ask again; the cost goes back to that day's budget.
    Dropped { time: u64, launcher: String, reason: String },
}

/// Where a launch is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase", tag = "status")]
pub enum Status {
    Pending,
    Confirmed { height: u32 },
    Dropped,
}

/// One DID the sponsor launched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Launch {
    pub identity: String,
    pub address: String,
    pub launcher: String,
    pub coin: String,
    pub cost: u64,
    pub client: String,
    pub txid: String,
    pub time: u64,
    pub status: Status,
}

/// The log and the state it adds up to.
pub struct Store {
    path: Option<PathBuf>,
    file: Option<std::fs::File>,
    launches: BTreeMap<String, Launch>,
    /// Identity → its launch that isn't dropped.
    by_identity: HashMap<String, String>,
    /// Day → mojos spent (dropped launches refunded).
    spent: BTreeMap<u64, u64>,
    /// (day, asker hash) → launches.
    per_client: HashMap<(u64, String), u32>,
    bundles: usize,
}

impl Store {
    /// A store in memory only (tests).
    pub fn memory() -> Self {
        Self {
            path: None,
            file: None,
            launches: BTreeMap::new(),
            by_identity: HashMap::new(),
            spent: BTreeMap::new(),
            per_client: HashMap::new(),
            bundles: 0,
        }
    }

    /// Open (or create) the log at `path` and replay it. A line that doesn't parse (a crash in
    /// the middle of a write) is skipped with a warning.
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut store = Self::memory();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        let mut torn = false;
        if let Ok(text) = std::fs::read_to_string(path) {
            torn = !text.is_empty() && !text.ends_with('\n');
            for (i, line) in text.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                match serde_json::from_str::<Event>(line) {
                    Ok(e) => store.apply(&e),
                    Err(e) => log::warn!("{}:{}: skipped: {e}", path.display(), i + 1),
                }
            }
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if torn {
            // End the torn line, so the next event starts a line of its own.
            file.write_all(b"\n")
                .map_err(|e| format!("{}: {e}", path.display()))?;
        }
        store.path = Some(path.to_owned());
        store.file = Some(file);
        Ok(store)
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Write an event (synced to disk) and apply it.
    pub fn record(&mut self, event: Event) -> Result<(), String> {
        if let Some(file) = &mut self.file {
            let mut line = serde_json::to_string(&event).map_err(|e| e.to_string())?;
            line.push('\n');
            file.write_all(line.as_bytes())
                .and_then(|()| file.sync_data())
                .map_err(|e| format!("the sponsor log: {e}"))?;
        }
        self.apply(&event);
        Ok(())
    }

    fn apply(&mut self, event: &Event) {
        match event {
            Event::Sponsored { time, identity, address, launcher, coin, cost, client, txid } => {
                let day = time / DAY;
                *self.spent.entry(day).or_default() += cost;
                *self.per_client.entry((day, client.clone())).or_default() += 1;
                self.by_identity.insert(identity.clone(), launcher.clone());
                self.launches.insert(
                    launcher.clone(),
                    Launch {
                        identity: identity.clone(),
                        address: address.clone(),
                        launcher: launcher.clone(),
                        coin: coin.clone(),
                        cost: *cost,
                        client: client.clone(),
                        txid: txid.clone(),
                        time: *time,
                        status: Status::Pending,
                    },
                );
            }
            Event::Bundle { .. } => self.bundles += 1,
            Event::Confirmed { launcher, height, .. } => {
                if let Some(l) = self.launches.get_mut(launcher) {
                    l.status = Status::Confirmed { height: *height };
                }
            }
            Event::Dropped { launcher, .. } => {
                if let Some(l) = self.launches.get_mut(launcher)
                    && l.status == Status::Pending
                {
                    l.status = Status::Dropped;
                    let day = l.time / DAY;
                    if let Some(s) = self.spent.get_mut(&day) {
                        *s = s.saturating_sub(l.cost);
                    }
                    if let Some(n) = self.per_client.get_mut(&(day, l.client.clone())) {
                        *n = n.saturating_sub(1);
                    }
                    if self.by_identity.get(&l.identity) == Some(launcher) {
                        self.by_identity.remove(&l.identity);
                    }
                }
            }
        }
    }

    /// The identity's launch, unless it was dropped.
    pub fn by_identity(&self, identity: &str) -> Option<&Launch> {
        self.by_identity
            .get(identity)
            .and_then(|l| self.launches.get(l))
    }

    pub fn launch(&self, launcher: &str) -> Option<&Launch> {
        self.launches.get(launcher)
    }

    /// Launches still waiting for the chain.
    pub fn pending(&self) -> Vec<Launch> {
        self.launches
            .values()
            .filter(|l| l.status == Status::Pending)
            .cloned()
            .collect()
    }

    /// Mojos spent on `day`.
    pub fn spent_on(&self, day: u64) -> u64 {
        self.spent.get(&day).copied().unwrap_or(0)
    }

    /// Launches for an asker's hash on `day`.
    pub fn client_count(&self, day: u64, client: &str) -> u32 {
        self.per_client
            .get(&(day, client.to_owned()))
            .copied()
            .unwrap_or(0)
    }

    /// Launches on `day` (not dropped).
    pub fn launched_on(&self, day: u64) -> usize {
        self.launches
            .values()
            .filter(|l| l.time / DAY == day && l.status != Status::Dropped)
            .count()
    }

    pub fn bundles(&self) -> usize {
        self.bundles
    }
}

/// What the store keeps of who asked (the asking endpoint's id): a hash of the day and the id (not
/// reversible without trying ids; changes every day).
pub fn client_hash(day: u64, client: &str) -> String {
    let mut h = Sha256::new();
    h.update(format!("chiacraft sponsor endpoint v1 {day} {client}").as_bytes());
    h.finalize()[..16]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sponsored(time: u64, identity: &str, launcher: &str) -> Event {
        Event::Sponsored {
            time,
            identity: identity.into(),
            address: "txch1x".into(),
            launcher: launcher.into(),
            coin: "c".into(),
            cost: 10,
            client: "ip".into(),
            txid: "t".into(),
        }
    }

    #[test]
    fn replays_and_refunds_drops() {
        let dir = std::env::temp_dir().join(format!("chiacraft-sponsor-store-{}", std::process::id()));
        let path = dir.join("sponsor-log.jsonl");
        let _ = std::fs::remove_dir_all(&dir);
        let day = 20_000;
        {
            let mut s = Store::open(&path).unwrap();
            s.record(sponsored(day * DAY + 5, "a", "L1")).unwrap();
            s.record(sponsored(day * DAY + 6, "b", "L2")).unwrap();
            s.record(Event::Confirmed { time: 0, launcher: "L1".into(), height: 7 })
                .unwrap();
            s.record(Event::Dropped { time: 0, launcher: "L2".into(), reason: "x".into() })
                .unwrap();
        }
        // A torn last line is skipped.
        std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"{\"event\":\"spons")
            .unwrap();
        let s = Store::open(&path).unwrap();
        assert_eq!(s.by_identity("a").unwrap().status, Status::Confirmed { height: 7 });
        assert!(s.by_identity("b").is_none());
        assert_eq!(s.spent_on(day), 10);
        assert_eq!(s.client_count(day, "ip"), 1);
        assert_eq!(s.launched_on(day), 1);
        assert!(s.pending().is_empty());
        // The next event after a torn line is kept.
        let mut s = s;
        s.record(sponsored(day * DAY + 7, "c", "L3")).unwrap();
        drop(s);
        let s = Store::open(&path).unwrap();
        assert_eq!(s.by_identity("c").unwrap().launcher, "L3");
        assert_eq!(s.pending().len(), 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn client_hashes_change_daily() {
        let id = "ae58ff8833241ac82d6ff7611046ed67b5072d142c588d0063e942d9a75502b6";
        assert_ne!(client_hash(1, id), client_hash(2, id));
        assert_eq!(client_hash(1, id).len(), 32);
        assert!(!client_hash(1, id).contains(&id[..8]));
    }

    #[test]
    fn horizons_log_lines_replay() {
        let line =
            r#"{"event":"sponsored","time":5,"identity":"a","address":"txch1x","launcher":"L","coin":"c","cost":3,"ip":"h","txid":"t"}"#;
        let e: Event = serde_json::from_str(line).unwrap();
        assert!(matches!(e, Event::Sponsored { ref client, .. } if client == "h"), "{e:?}");
    }
}
