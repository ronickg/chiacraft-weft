//! chiacraft (MIT): the server's side of admission rules and bans (`crate::admission`): who is
//! trusted, probation while playing, and the owner's commands.

use super::builtin::Ctx;
use super::*;
use crate::admission::{self as adm, Gate, Kind, Probation, Who};
use crate::ui::text::{Arg, Msg};

/// The world's gate. A dedicated server's old `whitelist.txt` becomes the world's allow list the
/// first time a world starts without `admission.json`.
pub(super) fn gate(cfg: &ServerConfig, dir: Option<std::path::PathBuf>) -> Gate {
    if let (Some(dir), Some(list)) = (&dir, &cfg.whitelist)
        && !dir.join(adm::RULES_FILE).exists()
        && let Ok(text) = std::fs::read_to_string(list)
    {
        let listed: Vec<&str> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        if !listed.is_empty() {
            let rules = adm::Rules {
                only_listed: true,
                allow: listed
                    .iter()
                    .map(|k| adm::Allowed {
                        kind: adm::kind_of(k),
                        value: k.to_string(),
                        name: String::new(),
                        added: crate::saves::now_secs(),
                    })
                    .collect(),
                ..Default::default()
            };
            match adm::save_rules(dir, &rules) {
                Ok(()) => {
                    log::info!("{} players of {} are this world's allow list now ({})", listed.len(), list.display(), adm::RULES_FILE)
                }
                Err(e) => log::warn!("{e}"),
            }
        }
    }
    Gate::new(dir)
}

fn yes_no(on: bool) -> Arg {
    Arg::Key(if on { "admission.yes" } else { "admission.no" }.into())
}

impl Player {
    /// Seconds played in this world, this visit included.
    pub(super) fn played_now(&self) -> u64 {
        self.played + self.since.map_or(0, |t| t.elapsed().as_secs())
    }

    /// Seconds of probation left.
    pub(super) fn probation_left(&self, p: &Probation) -> u64 {
        ((p.hours * 3600.0) as u64).saturating_sub(self.played_now())
    }

    fn who(&self) -> Who {
        Who {
            name: self.name.clone(),
            key: self.key.clone(),
            identity: self.identity.clone(),
            did: self.did.clone(),
            did_days: self.did_days,
            achievements: self.achievements.clone(),
        }
    }
}

/// A joining player's DID, as the chain answered (in the background).
pub(super) struct DidChecked {
    /// The player's login key.
    key: String,
    did: String,
    /// The key that signed still holds the DID.
    held: bool,
    days: Option<u32>,
    /// The chain answered from its cache, or not at all.
    why: Option<String>,
}

/// How long a join waits for its DID check before the rules decide without it.
const DID_WAIT: std::time::Duration = std::time::Duration::from_secs(25);

/// What probation may keep a player from doing.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Act {
    Build,
    Containers,
}

impl Server {
    /// The world's owner (its certificate's identity), or anyone with a permission level (an
    /// operator; on a world hosted from a game, its player): never refused, never on probation.
    pub(super) fn trusted(&self, pi: usize, key: &str) -> bool {
        let p = &self.players[pi];
        // The player on the machine that runs the world, when the world has no certificate to
        // say who its owner is. (Not by connection order alone: once the owner's own
        // connection is gone, a guest's may come first.)
        if pi == 0 && !self.cfg.dedicated && self.cfg.identity.is_none() {
            return true;
        }
        if p.joined && self.permission_of(pi) >= abi::permission::MODERATOR {
            return true;
        }
        let owner = self
            .cfg
            .identity
            .as_ref()
            .and_then(|i| i.cert.owner.split_once(':').map(|(id, _)| id.to_string()));
        if owner.is_some() && owner == p.identity {
            return true;
        }
        self.storage.as_ref().is_some_and(|s| {
            s.lock()
                .meta
                .ops
                .iter()
                .any(|o| o.0 == key && o.1 >= abi::permission::MODERATOR)
        })
    }

    /// The probation player `pi` is on now, if any.
    pub(super) fn probation_of(&mut self, pi: usize) -> Option<Probation> {
        let key = self.players[pi].key.clone();
        let trusted = self.trusted(pi, &key);
        let p = &self.players[pi];
        let (who, played, lifted) = (p.who(), std::time::Duration::from_secs(p.played_now()), p.lifted);
        self.gate.probation(&who, trusted, played, lifted)
    }

    /// The probation that keeps player `pi` from `act` now, if any (nothing said to them).
    pub(super) fn barred(&mut self, pi: usize, act: Act) -> Option<Probation> {
        self.probation_of(pi).filter(|p| match act {
            Act::Build => p.build,
            Act::Containers => p.containers,
        })
    }

    /// Tells player `pi` why they can't do `act` yet (at most every few seconds).
    pub(super) fn tell_barred(&mut self, pi: usize, act: Act, p: &Probation) {
        let player = &mut self.players[pi];
        if player.told.is_some_and(|t| t.elapsed().as_secs() < 3) {
            return;
        }
        player.told = Some(Instant::now());
        log::info!("{} is on probation: not yet ({})", player.name, if act == Act::Build { "building" } else { "containers" });
        let key = match act {
            Act::Build => "admission.probation_build",
            Act::Containers => "admission.probation_containers",
        };
        let minutes = player.probation_left(p).div_ceil(60) as i64;
        player
            .conn
            .send(ToClient::Show(crate::net::Shown::chat(Msg::new(key, vec![Arg::Int(minutes)]))));
    }

    /// Whether player `pi` may do `act`; when not, they are told why.
    pub(super) fn may(&mut self, pi: usize, act: Act) -> bool {
        match self.barred(pi, act) {
            Some(p) => {
                self.tell_barred(pi, act, &p);
                false
            }
            None => true,
        }
    }

    /// Whom `what` names: a player here by name, a player who was here (by the name in their
    /// file), or a login key, identity or DID as given.
    fn whom(&self, what: &str) -> Option<(Who, Option<usize>)> {
        if let Some(pi) = self
            .players
            .iter()
            .position(|p| p.joined && p.name.eq_ignore_ascii_case(what))
        {
            return Some((self.players[pi].who(), Some(pi)));
        }
        if let Some(s) = &self.storage {
            let dir = s.lock().meta.dir.join("players");
            for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
                let text = std::fs::read_to_string(entry.path()).unwrap_or_default();
                if crate::kv::get(&text, "name").is_some_and(|n| n.eq_ignore_ascii_case(what)) {
                    let key = entry
                        .path()
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    let identity = crate::kv::get(&text, "identity").map(str::to_string);
                    let did = crate::kv::get(&text, "did").map(str::to_string);
                    return Some((Who { name: what.to_string(), key, identity, did, ..Default::default() }, None));
                }
            }
        }
        if what.len() < 8 {
            return None;
        }
        let mut who = Who { name: String::new(), ..Default::default() };
        match adm::kind_of(what) {
            Kind::Key => who.key = what.to_string(),
            Kind::Identity => who.identity = Some(what.to_string()),
            Kind::Did => who.did = Some(what.to_string()),
        }
        // Someone here under that key, identity or DID.
        let here = self
            .players
            .iter()
            .position(|p| p.joined && (p.key == what || p.identity.as_deref() == Some(what)));
        Some((who, here))
    }

    /// Starts checking the DID player `pi` (login key `key`) claims, if `proof` is one their
    /// identity's DID's owner key signed: whether it was started.
    pub(super) fn check_did(&mut self, pi: usize, key: &str, proof: &str) -> bool {
        let p = &self.players[pi];
        let Some(identity) = p.identity.clone() else { return false };
        if proof.is_empty() {
            return false;
        }
        let Some((launcher, owner)) = chiacraft_account::check_did_proof(proof, &identity) else {
            log::info!("player {key}: a DID proof that doesn't check out; joined without a DID");
            return false;
        };
        let Some(did) = chiacraft_wallet::keys::encode_did(&launcher) else { return false };
        let tx = self.did_checks.0.clone();
        let key = key.to_string();
        std::thread::Builder::new()
            .name("did check".into())
            .spawn(move || {
                let answer = crate::chiacraft::chain::get().and_then(|reader| {
                    let (record, live) = crate::chiacraft::chain::did(&reader, &did)?;
                    let peak = reader.status().peak.unwrap_or(0).max(record.checked_height);
                    let days = record
                        .launched()
                        .map(|h| peak.saturating_sub(h) / crate::chiacraft::chain::BLOCKS_A_DAY);
                    Ok((record.owned_by(owner.into()), days, live))
                });
                let checked = match answer {
                    Ok((held, days, live)) => DidChecked { key, did, held, days, why: (!live).then(|| "from the cache".into()) },
                    Err(e) => DidChecked { key, did, held: false, days: None, why: Some(e) },
                };
                let _ = tx.send(checked);
            })
            .is_ok()
    }

    /// DID checks that came back: the player's DID is known (or not), a waiting join goes on,
    /// and a joined player whose DID is banned leaves. Joins that waited too long go on without.
    pub(super) fn did_answers(&mut self, c: &'static Content) {
        let mut banned_check = false;
        while let Ok(d) = self.did_checks.1.try_recv() {
            let Some(pi) = self
                .players
                .iter()
                .position(|p| p.key == d.key || p.waiting.as_ref().is_some_and(|w| w.1 == d.key))
            else {
                continue;
            };
            let p = &mut self.players[pi];
            if d.held {
                log::info!(
                    "player {} proved {} ({} days old){}",
                    d.key,
                    d.did,
                    d.days.map_or("?".into(), |x| x.to_string()),
                    d.why.as_deref().map_or(String::new(), |w| format!(", {w}"))
                );
                p.did = Some(d.did);
                p.did_days = d.days;
            } else {
                log::info!(
                    "player {}: {} isn't theirs on chain{}",
                    d.key,
                    d.did,
                    d.why
                        .as_deref()
                        .map_or(String::new(), |w| format!(" ({w})"))
                );
            }
            p.did_pending = false;
            if p.waiting.is_none() {
                banned_check = true;
            } else if !p.proofs_pending
                && let Some((name, key, _)) = p.waiting.take()
            {
                self.join(c, pi, name, key);
            }
        }
        for pi in 0..self.players.len() {
            if self.players[pi]
                .waiting
                .as_ref()
                .is_some_and(|w| w.2.elapsed() > DID_WAIT)
                && let Some((name, key, _)) = self.players[pi].waiting.take()
            {
                log::info!("{name}: no answer about their DID or achievements in time; the rules decide without them");
                self.players[pi].did_pending = false;
                self.players[pi].proofs_pending = false;
                self.join(c, pi, name, key);
            }
        }
        if banned_check {
            self.send_banned_away(c);
        }
    }

    /// Players here who are banned now (a ban or a followed list changed): sent away.
    pub(super) fn send_banned_away(&mut self, c: &'static Content) {
        for pi in 0..self.players.len() {
            if !self.players[pi].joined {
                continue;
            }
            let key = self.players[pi].key.clone();
            let trusted = self.trusted(pi, &key);
            let who = self.players[pi].who();
            // Only bans: a player already in stays in when the allow list changes.
            let mut rules = self.gate.rules.clone();
            rules.only_listed = false;
            let v = adm::verdict(&rules, &self.gate.bans, &self.gate.lists, &who, trusted, crate::saves::now_secs());
            if let adm::Verdict::Refuse(why) = v {
                log::info!("{} is sent away: {}", who.name, why.key);
                self.leave(c, pi, ToClient::Refused(why));
            }
        }
    }
}

fn say(key: &str, args: Vec<Arg>) -> Vec<Msg> {
    vec![Msg::new(&format!("admission.{key}"), args)]
}

fn text(s: &str) -> Arg {
    Arg::Text(s.to_string())
}

fn failed(e: String) -> Vec<Msg> {
    say("failed", vec![text(&e)])
}

fn short(v: &str) -> String {
    if v.chars().count() > 20 { format!("{}…", v.chars().take(16).collect::<String>()) } else { v.to_string() }
}

/// `/ban <who> [reason]`.
pub(super) fn ban(s: &mut Server, x: &Ctx) -> Vec<Msg> {
    ban_for(s, x, None)
}

/// `/tempban <who> <time> [reason]`.
pub(super) fn tempban(s: &mut Server, x: &Ctx) -> Vec<Msg> {
    let Some(secs) = x.a.text("time").and_then(adm::parse_duration) else {
        return say("bad_time", vec![text(x.a.text("time").unwrap_or(""))]);
    };
    ban_for(s, x, Some(crate::saves::now_secs() + secs))
}

fn ban_for(s: &mut Server, x: &Ctx, until: Option<u64>) -> Vec<Msg> {
    let what = x.a.text("who").unwrap_or("");
    let Some((who, here)) = s.whom(what) else {
        return say("unknown_player", vec![text(what)]);
    };
    if here.is_some_and(|pi| s.trusted(pi, &s.players[pi].key.clone())) {
        return say("not_the_trusted", vec![text(&who.name)]);
    }
    let reason = x.a.text("reason").unwrap_or("").to_string();
    let by = s.players[x.pi].name.clone();
    if let Err(e) = s
        .gate
        .edit_bans(|b| adm::ban_all(b, &who, &reason, &by, until))
    {
        return failed(e);
    }
    let name = if who.name.is_empty() { short(what) } else { who.name.clone() };
    log::info!("{by} banned {name}{}", until.map_or(String::new(), |u| format!(" until {u}")));
    s.send_banned_away(x.c);
    match until {
        Some(u) => say("banned_for_done", vec![text(&name), Arg::Int(u.saturating_sub(crate::saves::now_secs()).div_ceil(3600) as i64)]),
        None => say("banned_done", vec![text(&name)]),
    }
}

/// `/unban <who>`.
pub(super) fn unban(s: &mut Server, x: &Ctx) -> Vec<Msg> {
    let what = x.a.text("who").unwrap_or("").to_string();
    let mut n = 0;
    if let Err(e) = s.gate.edit_bans(|b| n = adm::unban(b, &what)) {
        return failed(e);
    }
    log::info!("unban {what}: {n} lifted");
    if n == 0 { say("not_banned", vec![text(&what)]) } else { say("unbanned", vec![text(&what), Arg::Int(n as i64)]) }
}

/// `/bans`: who is banned here.
pub(super) fn bans(s: &mut Server, _x: &Ctx) -> Vec<Msg> {
    s.gate.refresh(true);
    let now = crate::saves::now_secs();
    let list: Vec<_> = s.gate.bans.bans.iter().filter(|b| b.active(now)).collect();
    if list.is_empty() {
        return say("no_bans", Vec::new());
    }
    list.iter()
        .map(|b| {
            let name = if b.name.is_empty() { short(&b.value) } else { b.name.clone() };
            let reason = if b.reason.is_empty() { Arg::Key("admission.no_reason".into()) } else { text(&b.reason) };
            Msg::new("admission.ban_line", vec![text(&name), Arg::Key(format!("admission.kind.{}", b.kind.name())), reason])
        })
        .collect()
}

/// `/allow add <who> | remove <who> | list | only | everyone`.
pub(super) fn allow(s: &mut Server, x: &Ctx) -> Vec<Msg> {
    let a = x.a;
    if a.get("only").is_some() || a.get("everyone").is_some() {
        let only = a.get("only").is_some();
        if let Err(e) = s.gate.edit_rules(|r| r.only_listed = only) {
            return failed(e);
        }
        return say(if only { "only_listed_on" } else { "only_listed_off" }, Vec::new());
    }
    if a.get("list").is_some() {
        s.gate.refresh(true);
        let r = &s.gate.rules;
        let mut lines = say(if r.only_listed { "allow_list_only" } else { "allow_list_open" }, vec![Arg::Int(r.allow.len() as i64)]);
        for e in &r.allow {
            let name = if e.name.is_empty() { short(&e.value) } else { e.name.clone() };
            lines.push(Msg::new("admission.allow_line", vec![text(&name), Arg::Key(format!("admission.kind.{}", e.kind.name()))]));
        }
        return lines;
    }
    let what = a.text("who").unwrap_or("").to_string();
    if a.get("remove").is_some() {
        let mut n = 0;
        let r = s.gate.edit_rules(|r| {
            let before = r.allow.len();
            r.allow
                .retain(|e| !(e.value == what || (!e.name.is_empty() && e.name.eq_ignore_ascii_case(&what))));
            n = before - r.allow.len();
        });
        if let Err(e) = r {
            return failed(e);
        }
        return if n == 0 { say("not_listed_here", vec![text(&what)]) } else { say("allow_removed", vec![text(&what)]) };
    }
    let Some((who, _)) = s.whom(&what) else {
        return say("unknown_player", vec![text(&what)]);
    };
    // The identity when known (any login key of that account), else what was given.
    let (kind, value) = match (&who.identity, &who.did) {
        (Some(i), _) => (Kind::Identity, i.clone()),
        (None, Some(d)) => (Kind::Did, d.clone()),
        _ => (Kind::Key, who.key.clone()),
    };
    let name = who.name.clone();
    let r = s.gate.edit_rules(|r| {
        if !r.allow.iter().any(|e| e.kind == kind && e.value == value) {
            r.allow
                .push(adm::Allowed { kind, value, name: name.clone(), added: crate::saves::now_secs() });
        }
    });
    if let Err(e) = r {
        return failed(e);
    }
    say("allow_added", vec![text(if name.is_empty() { &what } else { &name })])
}

/// `/probation show | hours <n> | build <on|off> | containers <on|off> | end <who>`.
pub(super) fn probation(s: &mut Server, x: &Ctx) -> Vec<Msg> {
    let a = x.a;
    let on = |v: Option<&str>| v == Some("on");
    let changed = if let Some(h) = a.int("hours") {
        Some(
            s.gate
                .edit_rules(|r| r.probation.hours = h.clamp(0, 10_000) as f32),
        )
    } else if a.get("build").is_some() {
        Some(s.gate.edit_rules(|r| r.probation.build = on(a.text("on"))))
    } else if a.get("containers").is_some() {
        Some(
            s.gate
                .edit_rules(|r| r.probation.containers = on(a.text("on"))),
        )
    } else {
        None
    };
    if let Some(Err(e)) = changed {
        return failed(e);
    }
    if a.get("end").is_some() {
        let what = a.text("who").unwrap_or("");
        return match s.whom(what) {
            Some((_, Some(pi))) => {
                s.players[pi].lifted = true;
                log::info!("{} is off probation", s.players[pi].name);
                s.players[pi]
                    .conn
                    .send(ToClient::Show(crate::net::Shown::chat(Msg::new("admission.probation_lifted_you", Vec::new()))));
                say("probation_lifted", vec![text(&s.players[pi].name.clone())])
            }
            Some((who, None)) => match &s.storage {
                // Someone not here: their file says so for next time.
                Some(st) => {
                    let path = st.lock().player_path(&who.key);
                    match std::fs::read_to_string(&path) {
                        Ok(t) => {
                            let t: String = t
                                .lines()
                                .filter(|l| !l.starts_with("probation"))
                                .map(|l| format!("{l}\n"))
                                .collect();
                            match std::fs::write(&path, format!("{t}probation = lifted\n")) {
                                Ok(()) => say("probation_lifted", vec![text(what)]),
                                Err(e) => failed(e.to_string()),
                            }
                        }
                        Err(_) => say("unknown_player", vec![text(what)]),
                    }
                }
                None => say("unknown_player", vec![text(what)]),
            },
            None => say("unknown_player", vec![text(what)]),
        };
    }
    s.gate.refresh(true);
    let p = &s.gate.rules.probation;
    say("probation_show", vec![Arg::Int(p.hours.ceil() as i64), yes_no(p.build), yes_no(p.containers)])
}

/// `/banlist list | follow <world address> [maintainer] | unfollow <maintainer, address or name>
/// | publish [name]`. Without a maintainer, a followed address means its world's owner's list.
pub(super) fn banlist(s: &mut Server, x: &Ctx) -> Vec<Msg> {
    let a = x.a;
    if a.get("follow").is_some() {
        let from = a.text("from").unwrap_or("").to_string();
        let maintainer = a.text("maintainer").unwrap_or("").to_string();
        if adm::world_key(&from).is_none() {
            return say("not_an_address", vec![text(&from)]);
        }
        if !maintainer.is_empty() && adm::kind_of(&maintainer) != Kind::Identity {
            return say("not_an_identity", vec![text(&maintainer)]);
        }
        let mut full = false;
        let r = s.gate.edit_rules(|r| {
            let same = |f: &adm::Following| f.maintainer == maintainer && (!maintainer.is_empty() || f.from.first() == Some(&from));
            if let Some(f) = r.lists.iter_mut().find(|f| same(f)) {
                if !f.from.contains(&from) {
                    f.from.push(from.clone());
                }
            } else if r.lists.len() < adm::MAX_LISTS {
                r.lists
                    .push(adm::Following { maintainer: maintainer.clone(), from: vec![from.clone()], name: String::new() });
            } else {
                full = true;
            }
        });
        if let Err(e) = r {
            return failed(e);
        }
        if full {
            return say("too_many_lists", vec![Arg::Int(adm::MAX_LISTS as i64)]);
        }
        if let Some(dir) = s.gate.dir() {
            crate::net::fetch_ban_lists(dir.to_path_buf());
        }
        log::info!("following the ban list of {} from {}", if maintainer.is_empty() { "its owner" } else { &maintainer }, short(&from));
        return say("following", vec![text(&short(if maintainer.is_empty() { &from } else { &maintainer }))]);
    }
    if a.get("unfollow").is_some() {
        let what = a.text("what").unwrap_or("").to_string();
        let named: Vec<String> = s
            .gate
            .lists
            .iter()
            .filter(|l| l.name.eq_ignore_ascii_case(&what))
            .map(|l| l.maintainer.clone())
            .collect();
        let owners = s.gate.dir().map(adm::owners).unwrap_or_default();
        let mut n = 0;
        let r = s.gate.edit_rules(|r| {
            let before = r.lists.len();
            r.lists.retain(|f| {
                let m = adm::maintainer_of(f, &owners).unwrap_or_default();
                !(f.maintainer == what || m == what || f.from.contains(&what) || named.contains(&m))
            });
            n = before - r.lists.len();
        });
        if let Err(e) = r {
            return failed(e);
        }
        return if n == 0 { say("not_following", vec![text(&short(&what))]) } else { say("unfollowed", vec![text(&short(&what))]) };
    }
    if a.get("publish").is_some() {
        let Some(dir) = s.gate.dir().map(|d| d.to_path_buf()) else {
            return failed("this world isn't saved".into());
        };
        let world = s
            .storage
            .as_ref()
            .map(|st| st.lock().meta.name.clone())
            .unwrap_or_default();
        let name = a
            .text("name")
            .map(str::to_string)
            .unwrap_or_else(|| world.clone());
        return match adm::publish(&dir, &name) {
            Ok(list) => {
                log::info!(
                    "published ban list '{}' version {} ({} entries), maintainer {}",
                    list.name,
                    list.seq,
                    list.entries.len(),
                    list.maintainer
                );
                say(
                    "published",
                    vec![text(&list.name), Arg::Int(list.entries.len() as i64), Arg::Int(list.seq as i64), text(&list.maintainer)],
                )
            }
            Err(e) => failed(e),
        };
    }
    s.gate.refresh(true);
    if s.gate.rules.lists.is_empty() {
        return say("no_lists", Vec::new());
    }
    let lists = s.gate.lists.clone();
    let owners = s.gate.dir().map(adm::owners).unwrap_or_default();
    s.gate
        .rules
        .lists
        .iter()
        .map(|f| {
            let m = adm::maintainer_of(f, &owners);
            match lists.iter().find(|l| Some(&l.maintainer) == m.as_ref()) {
                Some(l) => Msg::new(
                    "admission.list_line",
                    vec![text(&l.name), text(&short(&l.maintainer)), Arg::Int(l.entries.len() as i64), Arg::Int(l.seq as i64)],
                ),
                None => Msg::new("admission.list_waiting", vec![text(&short(f.from.first().unwrap_or(&f.maintainer)))]),
            }
        })
        .collect()
}

/// `/didrule show | required <on|off> | age <days>`: whether players must prove a DID, and how
/// old it must be (phase 4, slice 6).
pub(super) fn didrule(s: &mut Server, x: &Ctx) -> Vec<Msg> {
    let a = x.a;
    let changed = if a.get("required").is_some() {
        let on = a.text("on") == Some("on");
        Some(s.gate.edit_rules(|r| r.did_required = on))
    } else if let Some(days) = a.int("days") {
        Some(
            s.gate
                .edit_rules(|r| r.did_min_days = days.clamp(0, 36_500) as u32),
        )
    } else {
        None
    };
    if let Some(Err(e)) = changed {
        return failed(e);
    }
    s.gate.refresh(true);
    let r = &s.gate.rules;
    say("did_rule_show", vec![yes_no(r.needs_did()), Arg::Int(r.did_min_days as i64)])
}

/// `/achievementrule <id> [issuer keys…] | off | show`: only players who earned achievement `id`
/// (`base:workbench`) get in, as a credential signed by this world's owner or by one of the
/// identity keys given (phase 4).
pub(super) fn achievementrule(s: &mut Server, x: &Ctx) -> Vec<Msg> {
    let a = x.a;
    if a.get("off").is_some() {
        if let Err(e) = s.gate.edit_rules(|r| r.achievement = None) {
            return failed(e);
        }
        return say("achievement_rule_off", Vec::new());
    }
    if let Some(id) = a.text("id") {
        let id = id.trim().to_string();
        let owner = s
            .cfg
            .identity
            .as_ref()
            .and_then(|i| i.cert.owner.split_once(':').map(|(o, _)| o.to_string()))
            .or_else(|| chiacraft_account::current().map(|s| s.identity_hex()));
        let mut from: Vec<String> = owner.into_iter().collect();
        for k in a.text("from").unwrap_or("").split_whitespace() {
            if adm::kind_of(k) != Kind::Identity {
                return say("not_an_identity", vec![text(k)]);
            }
            if !from.iter().any(|f| f == k) {
                from.push(k.to_string());
            }
        }
        // The base game's titles are `advancement.<namespace>.<name>`.
        let title = id
            .split_once(':')
            .map_or_else(|| id.clone(), |(ns, n)| format!("advancement.{ns}.{n}"));
        let (rule_id, trusted) = (id.clone(), from.len());
        let r = s
            .gate
            .edit_rules(|r| r.achievement = Some(adm::AchievementRule { id: rule_id, title, from }));
        if let Err(e) = r {
            return failed(e);
        }
        log::info!("only players with the achievement {id} get in now ({trusted} issuers trusted)");
        return say("achievement_rule_set", vec![text(&id), Arg::Int(trusted as i64)]);
    }
    s.gate.refresh(true);
    match &s.gate.rules.achievement {
        Some(r) => say("achievement_rule_set", vec![text(&r.id), Arg::Int(r.from.len() as i64)]),
        None => say("achievement_rule_off", Vec::new()),
    }
}

/// `/revoke <who> <achievement id | *>`: withdraws what this world signed for that player (phase
/// 4): its issuer's revocation list grows, every player here gets it, and it no longer counts at
/// the door.
pub(super) fn revoke(s: &mut Server, x: &Ctx) -> Vec<Msg> {
    let what = x.a.text("who").unwrap_or("").to_string();
    let id = x.a.text("id").unwrap_or("").to_string();
    let Some((who, _)) = s.whom(&what) else {
        return say("unknown_player", vec![text(&what)]);
    };
    let Some(identity) = who.identity else {
        return say("no_identity", vec![text(&what)]);
    };
    match s.revoke_achievements(&identity, &id) {
        Ok(0) => say("nothing_to_revoke", vec![text(&what), text(&id)]),
        Ok(n) => say("revoked", vec![text(&what), text(&id), Arg::Int(n as i64)]),
        Err(e) => failed(e),
    }
}
