//! Clans: the animals mod's tribes (`mods/animals/src/commands.rs`, `tribe_command`) ported to a
//! sandboxed mod, the first of ours (chiacraft plan, phase 6).
//!
//! What changed in the port, and why:
//! - The tribe was a per-player component (`TRIBE`, `INVITE`, saved and kept on respawn). The
//!   sandbox has no components yet, so each player's clan and invite live in the mod's storage,
//!   kept by the player's key (the same in every world and visit).
//! - The commands are `/clan` and `/claninvite`, so the native mod's `/tribe` keeps its name.
//! - New: the invited player is told, the clan hears of a new member, and a returning member is
//!   greeted, through `World::tell` (lines in the package's own language files).
//!
//! The rest is the original code with its `use weft_sdk::*;` (the package alias in Cargo.toml).

use std::collections::BTreeMap;

use weft_sdk::*;

declare_mod!(deps: [], init: init);

/// A clan: a random id (shared by its members) and a name.
#[derive(Clone, Default, PartialEq)]
struct Tribe {
    id: String,
    name: String,
}

impl Tribe {
    fn is_some(&self) -> bool {
        !self.id.is_empty()
    }
}

/// A player's clan and the one they are invited to.
#[derive(Clone, Default)]
struct Member {
    tribe: Tribe,
    invite: Tribe,
}

/// Every player's, by key: one storage value, a line each (`key\tid\tname\tinvite id\tinvite name`).
const TABLE: &str = "members";

fn clean(s: &str) -> String {
    s.replace(['\t', '\n'], " ").chars().take(24).collect()
}

fn load() -> BTreeMap<String, Member> {
    let text = storage::get_text(TABLE).unwrap_or_default();
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            let [key, id, name, iid, iname] = f[..] else { return None };
            let t = |id: &str, name: &str| Tribe { id: id.into(), name: name.into() };
            Some((key.to_string(), Member { tribe: t(id, name), invite: t(iid, iname) }))
        })
        .collect()
}

fn save(all: &BTreeMap<String, Member>) {
    let text: String = all
        .iter()
        .filter(|(_, m)| m.tribe.is_some() || m.invite.is_some())
        .map(|(k, m)| format!("{k}\t{}\t{}\t{}\t{}\n", m.tribe.id, m.tribe.name, m.invite.id, m.invite.name))
        .collect();
    if let Err(e) = storage::set(TABLE, text.as_bytes()) {
        error!("saving the clans: {e}");
    }
}

fn init(r: &mut Registry) {
    r.command("clan", "[action:word(create,join,leave,info)] [name:text]", tribe_command)
        .permission(permission::EVERYONE);
    r.command("claninvite", "<player:targets>", |cmd| {
        let (w, player) = (cmd.world(), cmd.player());
        let mut all = load();
        let Some(t) = all
            .get(&w.player_key(player))
            .map(|m| m.tribe.clone())
            .filter(Tribe::is_some)
        else {
            cmd.reply("command.clans.none", &[]);
            return;
        };
        for other in cmd.targets("player") {
            let key = w.player_key(other);
            if key.is_empty() || other == player {
                continue;
            }
            all.entry(key).or_default().invite = t.clone();
            w.tell(other, "clans.invite", &[Arg::Text(&t.name)]);
        }
        save(&all);
        cmd.reply("command.clans.invited", &[Arg::Text(&t.name)]);
    })
    .permission(permission::EVERYONE);
    r.on(&PLAYER_JOINED, |b| {
        let w = b.world();
        let all = load();
        for e in b.iter() {
            if let Some(m) = all
                .get(&w.player_key(e.player))
                .filter(|m| m.tribe.is_some())
            {
                w.tell(e.player, "clans.welcome", &[Arg::Text(&m.tribe.name)]);
            }
        }
    });
}

fn tribe_command(cmd: &mut ServerCommand) {
    let (w, player) = (cmd.world(), cmd.player());
    let key = w.player_key(player);
    let mut all = load();
    let mine = all.get(&key).cloned().unwrap_or_default();
    match cmd.text("action").unwrap_or("info") {
        "create" => {
            let name = clean(cmd.text("name").unwrap_or("").trim());
            if name.is_empty() {
                cmd.reply("command.clans.name", &[]);
                return;
            }
            let id: String = (0..16)
                .map(|_| format!("{:02x}", w.random_below(256)))
                .collect();
            let t = Tribe { id, name };
            all.entry(key).or_default().tribe = t.clone();
            save(&all);
            info!("clan {} founded by player {player}", t.name);
            cmd.reply("command.clans.created", &[Arg::Text(&t.name)]);
        }
        "join" => {
            if !mine.invite.is_some() {
                cmd.reply("command.clans.no_invite", &[]);
                return;
            }
            let invite = mine.invite.clone();
            let me = all.entry(key.clone()).or_default();
            me.tribe = invite.clone();
            me.invite = Tribe::default();
            save(&all);
            // Members here hear of it.
            for p in w.players() {
                let k = w.player_key(p);
                if k != key && all.get(&k).is_some_and(|m| m.tribe == invite) {
                    w.tell(p, "clans.member_joined", &[Arg::Text(&invite.name)]);
                }
            }
            info!("player {player} joined clan {}", invite.name);
            cmd.reply("command.clans.joined", &[Arg::Text(&invite.name)]);
        }
        "leave" => {
            if let Some(m) = all.get_mut(&key) {
                m.tribe = Tribe::default();
            }
            save(&all);
            cmd.reply("command.clans.left", &[Arg::Text(&mine.tribe.name)]);
        }
        _ => {
            if mine.tribe.is_some() {
                let members = all.values().filter(|m| m.tribe == mine.tribe).count();
                cmd.reply("command.clans.info", &[Arg::Text(&mine.tribe.name), Arg::Int(members as i64)]);
            } else {
                cmd.reply("command.clans.none", &[]);
            }
        }
    }
}
