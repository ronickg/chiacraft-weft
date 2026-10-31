//! Achievements that outlive a server (identity row I10; identity-research.md 4.3,
//! docs/identity.md "Achievements"): a small record that names the player, the server and the
//! achievement, signed by the server's identity key and kept by the player. Free (no chain), and
//! anyone verifies it offline: one BLS signature against the issuer's key, which the record
//! carries. Each verifier decides whose achievements it trusts.
//!
//! - **The record** ([`Credential`], format 1) is JSON, the same text in the player's
//!   `achievements.json`, on the wire and in an exported file. Its signature covers a
//!   **domain-separated text** built from the fields ([`Credential::signing_text`]: first line
//!   `chiacraft achievement v1`, one `name: value` line per field, text fields as JSON strings), not
//!   the JSON itself, so key order and whitespace don't matter and the signature can't stand for any
//!   other chiacraft text. It is signed as a Chia signed message (CHIP-0002) like every identity
//!   signature ([`crate::sign`]).
//! - **Self-signed**: in single player the world's "server" is the player's own account, so issuer
//!   and subject are the same key; [`Credential::self_signed`] says so and screens label it.
//! - **The issuer's DID** (optional): a server with `server-did` adds the DID and a proof
//!   ([`IssuerDid`]): the DID owner key's signature of [`issuer_did_text`], binding the DID to the
//!   issuer key. Checked offline; whether the key still owns the DID is a chain question.
//! - **Revocation** ([`Revocations`]): each issuer keeps one signed list of revoked nonces, a newer
//!   `seq` replacing an older one. Players get it from the server, other servers ask the address in
//!   the credential's `status` when online.
//! - **The daily anchor** (optional, [`Anchor`]): a server may commit one Merkle root of a day's
//!   credentials on chain; a credential can carry its inclusion proof ([`merkle_root`],
//!   [`merkle_path`], [`Anchor::check`]). Not part of the signed text: it is added after the day.
use crate::{
    Error, Result,
    keys::{IdentityKey, from_hex, hex},
    sign::{self, Signer},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The credential format this build writes and reads.
pub const FORMAT: u32 = 1;
/// The first line of the signed text.
pub const SIGNED_LINE: &str = "chiacraft achievement v1";
/// Field limits (characters).
pub const MAX_ID: usize = 128;
pub const MAX_TITLE: usize = 64;
pub const MAX_DESCRIPTION: usize = 256;
pub const MAX_NAME: usize = 64;
pub const MAX_ICON: usize = 128;
/// A mod's or command's optional data, in bytes.
pub const MAX_DATA: usize = 1024;
pub const MAX_STATUS: usize = 256;
/// Nonces one revocation list holds at most.
pub const MAX_REVOKED: usize = 8192;
/// The memo the daily anchor's coin carries before the root.
pub const ANCHOR_MEMO: &[u8] = b"chiacraft achievements v1";

/// Who signed: the server's identity key (hex, 48 bytes), its name as it called itself, and
/// optionally its DID with the owner key's proof.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issuer {
    pub key: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub did: Option<IssuerDid>,
}

/// An issuer's DID (`did:chia:1…`) and its owner key's signature of [`issuer_did_text`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssuerDid {
    pub did: String,
    /// The DID owner's synthetic public key (hex, 48 bytes).
    pub owner: String,
    /// Hex, 96 bytes.
    pub sig: String,
}

/// Who earned it: the player's identity key (hex, 48 bytes) and their player id v2.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    pub key: String,
    /// `8-4-4-4-12` hex: the first 16 bytes of SHA-256(`cc player id v2\0account\0` + key).
    pub player: String,
}

/// What was earned (from the server's content, a command or a mod).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub title: String,
    pub description: String,
    /// An item id screens may draw beside it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// A day's inclusion proof (the optional daily anchor): the credential's leaf is in the Merkle
/// tree whose root the server committed on chain that day.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anchor {
    /// Days since 1970-01-01 (UTC) the root covers.
    pub day: u64,
    /// The root (hex, 32 bytes).
    pub root: String,
    /// The leaf's place and the siblings from the leaf up: `l:<hex>` (sibling on the left) or
    /// `r:<hex>`.
    pub index: u32,
    pub path: Vec<String>,
    /// `testnet11` (or `sim` for checks).
    pub network: String,
    /// The coin whose creation carries the root as a memo (hex, 32 bytes), once confirmed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coin: Option<String>,
}

/// One achievement credential (format [`FORMAT`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credential {
    pub v: u32,
    pub issuer: Issuer,
    pub subject: Subject,
    pub achievement: Achievement,
    /// Unix seconds.
    pub issued: u64,
    /// 16 random bytes (hex): what revocation names.
    pub nonce: String,
    /// Data the granting mod or command attached (at most [`MAX_DATA`] bytes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Where the issuer serves its revocation list (`host:port`), when it has a public address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// The issuer key's signature of [`Credential::signing_text`] (hex, 96 bytes).
    pub sig: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<Anchor>,
}

/// What [`Credential::verify`] found besides a good signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checked {
    /// Issuer and subject are the same key (single player).
    pub self_signed: bool,
    /// The issuer's DID, its owner proof checked (not whether it still owns it: a chain question).
    pub did: Option<String>,
    /// The day the anchor proof covers, when it carries one that leads to its root.
    pub anchored: Option<u64>,
}

/// A player id v2 of an identity key (horizon-protocol's `player_id_from_account`, which a test
/// in the game keeps equal).
pub fn player_id(identity: &[u8; 48]) -> u128 {
    let mut h = Sha256::new();
    h.update(b"cc player id v2\0account\0");
    h.update(identity);
    let d: [u8; 32] = h.finalize().into();
    u128::from_be_bytes(d[..16].try_into().expect("16 bytes"))
}

/// A player id as `8-4-4-4-12` hex.
pub fn id_string(id: u128) -> String {
    let h = format!("{id:032x}");
    format!(
        "{}-{}-{}-{}-{}",
        &h[0..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..32]
    )
}

/// An identity key's fingerprint (32 hex digits; horizon-protocol's `identity_fingerprint_hex`).
pub fn fingerprint(identity: &[u8; 48]) -> String {
    let mut h = Sha256::new();
    h.update(b"cc identity v1\0");
    h.update(identity);
    let d: [u8; 32] = h.finalize().into();
    hex(&d[..16])
}

/// The text a DID's owner key signs to vouch for an issuer key: its own first line, the DID's
/// launcher id and the issuer key, so it stands for nothing else.
pub fn issuer_did_text(launcher: &[u8; 32], issuer: &[u8; 48]) -> String {
    format!(
        "chiacraft achievement issuer v1\ndid: {}\nissuer: {}",
        hex(launcher),
        hex(issuer)
    )
}

/// 16 fresh random bytes as hex.
pub fn new_nonce() -> Result<String> {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).map_err(|e| Error::new(format!("no randomness: {e}")))?;
    Ok(hex(&b))
}

fn quoted(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}
fn opt_quoted(s: Option<&str>) -> String {
    s.map_or_else(|| "none".to_owned(), quoted)
}

fn key48(text: &str, what: &str) -> Result<[u8; 48]> {
    IdentityKey::from_hex(text)
        .map(|k| k.0)
        .ok_or_else(|| Error::new(format!("{what} isn't a key")))
}

fn bytes<const N: usize>(text: &str, what: &str) -> Result<[u8; N]> {
    from_hex(text)
        .and_then(|b| b.try_into().ok())
        .ok_or_else(|| Error::new(format!("{what} isn't {N} bytes of hex")))
}

fn sha256(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

fn too_long(what: &str, s: &str, max: usize) -> Result<()> {
    if s.chars().count() > max {
        return Err(Error::new(format!("{what} is longer than {max}")));
    }
    Ok(())
}

impl Credential {
    /// An unsigned credential; [`Credential::sign`] completes it.
    #[allow(clippy::too_many_arguments)]
    pub fn unsigned(
        issuer: Issuer,
        subject_key: &[u8; 48],
        achievement: Achievement,
        issued: u64,
        nonce: String,
        data: Option<String>,
        status: Option<String>,
    ) -> Self {
        Self {
            v: FORMAT,
            issuer,
            subject: Subject {
                key: hex(subject_key),
                player: id_string(player_id(subject_key)),
            },
            achievement,
            issued,
            nonce,
            data,
            status,
            sig: String::new(),
            anchor: None,
        }
    }

    /// The text the issuer signs: domain-separated, one field per line, text fields as JSON
    /// strings (so no field can add a line).
    pub fn signing_text(&self) -> String {
        format!(
            "{SIGNED_LINE}\nissuer: {}\nissuer name: {}\nissuer did: {}\nsubject: {}\nplayer: {}\nid: {}\ntitle: {}\ndescription: {}\nicon: {}\nissued: {}\nnonce: {}\ndata: {}\nstatus: {}",
            self.issuer.key,
            quoted(&self.issuer.name),
            self.issuer.did.as_ref().map_or("none", |d| d.did.as_str()),
            self.subject.key,
            self.subject.player,
            quoted(&self.achievement.id),
            quoted(&self.achievement.title),
            quoted(&self.achievement.description),
            opt_quoted(self.achievement.icon.as_deref()),
            self.issued,
            self.nonce,
            opt_quoted(self.data.as_deref()),
            opt_quoted(self.status.as_deref()),
        )
    }

    /// SHA-256 of the signed text: the credential's id and its Merkle leaf.
    pub fn hash(&self) -> [u8; 32] {
        sha256(&[self.signing_text().as_bytes()])
    }

    /// Signs with `signer`, which must hold the issuer key. The key never leaves the signer.
    pub fn sign(mut self, signer: &dyn Signer) -> Result<Self> {
        if signer.identity().hex() != self.issuer.key {
            return Err(Error::new("the signer doesn't hold the issuer key"));
        }
        self.check_fields()?;
        self.sig = hex(&signer.sign(&self.signing_text())?);
        Ok(self)
    }

    /// Issuer and subject are the same key: single player's own record.
    pub fn self_signed(&self) -> bool {
        self.issuer.key == self.subject.key
    }

    /// The issuer key's fingerprint (32 hex digits), as servers and screens show identities.
    pub fn issuer_fingerprint(&self) -> String {
        key48(&self.issuer.key, "the issuer")
            .map(|k| fingerprint(&k))
            .unwrap_or_default()
    }

    fn check_fields(&self) -> Result<()> {
        if self.v != FORMAT {
            return Err(Error::new(format!("unknown credential format {}", self.v)));
        }
        too_long("the issuer name", &self.issuer.name, MAX_NAME)?;
        too_long("the id", &self.achievement.id, MAX_ID)?;
        too_long("the title", &self.achievement.title, MAX_TITLE)?;
        too_long(
            "the description",
            &self.achievement.description,
            MAX_DESCRIPTION,
        )?;
        if let Some(i) = &self.achievement.icon {
            too_long("the icon", i, MAX_ICON)?;
        }
        if self.data.as_ref().is_some_and(|d| d.len() > MAX_DATA) {
            return Err(Error::new(format!(
                "the data is longer than {MAX_DATA} bytes"
            )));
        }
        if let Some(s) = &self.status {
            too_long("the status address", s, MAX_STATUS)?;
        }
        if self.achievement.id.is_empty() || self.achievement.title.trim().is_empty() {
            return Err(Error::new("an achievement needs an id and a title"));
        }
        bytes::<16>(&self.nonce, "the nonce")?;
        let subject = key48(&self.subject.key, "the subject")?;
        if self.subject.player != id_string(player_id(&subject)) {
            return Err(Error::new("the player id isn't the subject key's"));
        }
        key48(&self.issuer.key, "the issuer")?;
        Ok(())
    }

    /// Checks everything that can be checked offline: the format, the fields, the player id, the
    /// issuer's signature, the DID proof and the anchor's path to its root.
    pub fn verify(&self) -> Result<Checked> {
        self.check_fields()?;
        let issuer = key48(&self.issuer.key, "the issuer")?;
        let sig = from_hex(&self.sig).unwrap_or_default();
        if !sign::verify(&IdentityKey(issuer), self.signing_text().as_bytes(), &sig) {
            return Err(Error::new("the issuer's signature doesn't check"));
        }
        let did = match &self.issuer.did {
            None => None,
            Some(d) => {
                let launcher = crate::keys::decode_did(&d.did)
                    .ok_or_else(|| Error::new("the issuer DID isn't a did:chia"))?;
                let owner = key48(&d.owner, "the DID owner")?;
                let sig = from_hex(&d.sig).unwrap_or_default();
                if !sign::verify_owner(&owner, issuer_did_text(&launcher, &issuer).as_bytes(), &sig)
                {
                    return Err(Error::new("the issuer's DID proof doesn't check"));
                }
                Some(d.did.clone())
            }
        };
        let anchored = match &self.anchor {
            None => None,
            Some(a) if a.check(self.hash()) => Some(a.day),
            Some(_) => return Err(Error::new("the anchor proof doesn't lead to its root")),
        };
        Ok(Checked {
            self_signed: self.self_signed(),
            did,
            anchored,
        })
    }

    pub fn from_json(text: &str) -> Result<Self> {
        serde_json::from_str(text).map_err(|e| Error::new(format!("not a credential: {e}")))
    }
    /// One line (the wire, the store).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// An issuer's signed list of revoked nonces. A higher `seq` replaces a lower one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revocations {
    pub v: u32,
    pub issuer: String,
    pub seq: u64,
    /// Unix seconds.
    pub updated: u64,
    /// Nonces (hex), sorted.
    pub revoked: Vec<String>,
    pub sig: String,
}

impl Revocations {
    /// The text the issuer signs.
    pub fn signing_text(&self) -> String {
        format!(
            "chiacraft revocations v1\nissuer: {}\nseq: {}\nupdated: {}\nrevoked: {}",
            self.issuer,
            self.seq,
            self.updated,
            self.revoked.join(",")
        )
    }

    /// A signed list (`revoked` sorted and deduplicated here).
    pub fn signed(
        signer: &dyn Signer,
        seq: u64,
        updated: u64,
        mut revoked: Vec<String>,
    ) -> Result<Self> {
        revoked.sort();
        revoked.dedup();
        if revoked.len() > MAX_REVOKED {
            return Err(Error::new(format!(
                "a revocation list holds at most {MAX_REVOKED} nonces"
            )));
        }
        let mut r = Self {
            v: FORMAT,
            issuer: signer.identity().hex(),
            seq,
            updated,
            revoked,
            sig: String::new(),
        };
        r.sig = hex(&signer.sign(&r.signing_text())?);
        Ok(r)
    }

    pub fn verify(&self) -> Result<()> {
        if self.v != FORMAT {
            return Err(Error::new("unknown revocation list format"));
        }
        if self.revoked.len() > MAX_REVOKED || self.revoked.windows(2).any(|w| w[0] >= w[1]) {
            return Err(Error::new(
                "the revocation list isn't sorted or is too long",
            ));
        }
        let issuer = key48(&self.issuer, "the issuer")?;
        let sig = from_hex(&self.sig).unwrap_or_default();
        if !sign::verify(&IdentityKey(issuer), self.signing_text().as_bytes(), &sig) {
            return Err(Error::new("the revocation list's signature doesn't check"));
        }
        Ok(())
    }

    /// Whether this (verified) list revokes `c`.
    pub fn revokes(&self, c: &Credential) -> bool {
        self.issuer == c.issuer.key && self.revoked.binary_search(&c.nonce).is_ok()
    }

    pub fn from_json(text: &str) -> Result<Self> {
        serde_json::from_str(text).map_err(|e| Error::new(format!("not a revocation list: {e}")))
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// A credential hash as a Merkle leaf (domain byte 0).
pub fn leaf(hash: [u8; 32]) -> [u8; 32] {
    sha256(&[&[0], &hash])
}
fn node(l: &[u8; 32], r: &[u8; 32]) -> [u8; 32] {
    sha256(&[&[1], l, r])
}

/// The root of a day's leaves ([`leaf`] of each credential hash, in issue order); a level's odd
/// last node goes up unpaired. All zeros for none.
pub fn merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    if leaves.is_empty() {
        return [0; 32];
    }
    let mut level = leaves.to_vec();
    while level.len() > 1 {
        level = level
            .chunks(2)
            .map(|p| {
                if p.len() == 2 {
                    node(&p[0], &p[1])
                } else {
                    p[0]
                }
            })
            .collect();
    }
    level[0]
}

/// Leaf `index`'s path: each sibling up to the root, with whether it is on the left.
pub fn merkle_path(leaves: &[[u8; 32]], mut index: usize) -> Vec<(bool, [u8; 32])> {
    let mut out = Vec::new();
    let mut level = leaves.to_vec();
    while level.len() > 1 {
        let sib = index ^ 1;
        if sib < level.len() {
            out.push((sib < index, level[sib]));
        }
        level = level
            .chunks(2)
            .map(|p| {
                if p.len() == 2 {
                    node(&p[0], &p[1])
                } else {
                    p[0]
                }
            })
            .collect();
        index /= 2;
    }
    out
}

/// Whether `leaf` reaches `root` along `path`.
pub fn merkle_check(leaf: [u8; 32], path: &[(bool, [u8; 32])], root: [u8; 32]) -> bool {
    path.iter().fold(
        leaf,
        |h, (left, sib)| {
            if *left { node(sib, &h) } else { node(&h, sib) }
        },
    ) == root
}

impl Anchor {
    /// The anchor for leaf `index` of `leaves` (credential hashes, before [`leaf`]).
    pub fn of(day: u64, hashes: &[[u8; 32]], index: usize, network: &str) -> Self {
        let leaves: Vec<[u8; 32]> = hashes.iter().copied().map(leaf).collect();
        Self {
            day,
            root: hex(&merkle_root(&leaves)),
            index: index as u32,
            path: merkle_path(&leaves, index)
                .into_iter()
                .map(|(l, h)| format!("{}:{}", if l { "l" } else { "r" }, hex(&h)))
                .collect(),
            network: network.to_owned(),
            coin: None,
        }
    }

    /// Whether the credential with `hash` is in the tree this anchor names.
    pub fn check(&self, hash: [u8; 32]) -> bool {
        let Ok(root) = bytes::<32>(&self.root, "the root") else {
            return false;
        };
        let mut path = Vec::new();
        for p in &self.path {
            let Some((side, h)) = p.split_once(':') else {
                return false;
            };
            let Ok(h) = bytes::<32>(h, "a sibling") else {
                return false;
            };
            path.push((side == "l", h));
        }
        merkle_check(leaf(hash), &path, root)
    }
}

/// Days since 1970-01-01 of a Unix time.
pub fn day_of(unix: u64) -> u64 {
    unix / 86_400
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IdentitySigner, Mnemonic, OwnerSigner};

    fn signer() -> IdentitySigner {
        IdentitySigner::from_mnemonic(&Mnemonic::generate().unwrap(), 0)
    }

    fn credential(issuer: &IdentitySigner, subject: &[u8; 48]) -> Credential {
        Credential::unsigned(
            Issuer {
                key: issuer.identity().hex(),
                name: "Stormhold\nfake: line".into(),
                did: None,
            },
            subject,
            Achievement {
                id: "trophies:miner".into(),
                title: "Miner".into(),
                description: "Mined 100 blocks.".into(),
                icon: Some("chiacraft:stone_pickaxe".into()),
            },
            1_790_000_000,
            new_nonce().unwrap(),
            Some("{\"blocks\":100}".into()),
            Some("play.example.net:25565".into()),
        )
    }

    #[test]
    fn signs_verifies_and_refuses_tampering() {
        let server = signer();
        let player = signer();
        let c = credential(&server, &player.identity().0)
            .sign(&server)
            .unwrap();
        let checked = c.verify().unwrap();
        assert!(!checked.self_signed && checked.did.is_none() && checked.anchored.is_none());
        // The text: domain-separated, a field can't add a line.
        let text = c.signing_text();
        assert!(text.starts_with("chiacraft achievement v1\n"));
        assert_eq!(text.lines().count(), 14);
        assert!(text.contains("issuer name: \"Stormhold\\nfake: line\""));
        // Round trip through JSON.
        let back = Credential::from_json(&c.to_json()).unwrap();
        assert_eq!(back, c);
        assert!(back.verify().is_ok());
        // Every signed field matters.
        let mut t = c.clone();
        t.achievement.title = "Master miner".into();
        assert!(t.verify().is_err());
        let mut t = c.clone();
        t.issued += 1;
        assert!(t.verify().is_err());
        let mut t = c.clone();
        t.data = None;
        assert!(t.verify().is_err());
        // Another subject: the player id no longer matches, and with it fixed the signature fails.
        let other = signer();
        let mut t = c.clone();
        t.subject.key = other.identity().hex();
        assert!(t.verify().unwrap_err().to_string().contains("player id"));
        t.subject.player = id_string(player_id(&other.identity().0));
        assert!(t.verify().unwrap_err().to_string().contains("signature"));
        // The wrong signer is refused before signing.
        assert!(
            credential(&server, &player.identity().0)
                .sign(&player)
                .is_err()
        );
        // Limits.
        let mut big = credential(&server, &player.identity().0);
        big.data = Some("x".repeat(MAX_DATA + 1));
        assert!(big.sign(&server).is_err());
    }

    #[test]
    fn self_signed_and_issuer_did() {
        let me = signer();
        let c = credential(&me, &me.identity().0).sign(&me).unwrap();
        assert!(c.self_signed() && c.verify().unwrap().self_signed);
        // A DID proof by an owner key binds the DID to the issuer key.
        let server = signer();
        let owner = OwnerSigner::from_mnemonic(&Mnemonic::generate().unwrap(), 0, false);
        let launcher = [7u8; 32];
        let did = crate::keys::encode_did(&launcher).unwrap();
        let proof = owner
            .sign(&issuer_did_text(&launcher, &server.identity().0))
            .unwrap();
        let mut u = credential(&server, &me.identity().0);
        u.issuer.did = Some(IssuerDid {
            did: did.clone(),
            owner: hex(&owner.public()),
            sig: hex(&proof),
        });
        let c = u.sign(&server).unwrap();
        assert_eq!(c.verify().unwrap().did.as_deref(), Some(did.as_str()));
        // The same proof for another issuer key fails.
        let other = signer();
        let mut u = credential(&other, &me.identity().0);
        u.issuer.did = c.issuer.did.clone();
        let c2 = u.sign(&other).unwrap();
        assert!(c2.verify().unwrap_err().to_string().contains("DID proof"));
    }

    #[test]
    fn revocation_lists() {
        let server = signer();
        let player = signer();
        let a = credential(&server, &player.identity().0)
            .sign(&server)
            .unwrap();
        let b = credential(&server, &player.identity().0)
            .sign(&server)
            .unwrap();
        let r = Revocations::signed(&server, 2, 5, vec![b.nonce.clone(), b.nonce.clone()]).unwrap();
        assert!(r.verify().is_ok());
        assert_eq!(r.revoked.len(), 1);
        assert!(r.revokes(&b) && !r.revokes(&a));
        let back = Revocations::from_json(&r.to_json()).unwrap();
        assert!(back.verify().is_ok());
        let mut t = r.clone();
        t.revoked.clear();
        assert!(t.verify().is_err());
        // Another issuer's list revokes nothing of this one's.
        let other = signer();
        let r2 = Revocations::signed(&other, 1, 5, vec![b.nonce.clone()]).unwrap();
        assert!(!r2.revokes(&b));
    }

    #[test]
    fn merkle_proofs_for_every_leaf() {
        for n in [1usize, 2, 3, 5, 8, 13] {
            let hashes: Vec<[u8; 32]> = (0..n).map(|i| sha256(&[&[i as u8]])).collect();
            let leaves: Vec<[u8; 32]> = hashes.iter().copied().map(leaf).collect();
            let root = merkle_root(&leaves);
            for i in 0..n {
                let a = Anchor::of(20_000, &hashes, i, "sim");
                assert_eq!(a.root, hex(&root));
                assert!(a.check(hashes[i]), "n {n} leaf {i}");
                assert!(!a.check(sha256(&[b"other"])));
            }
        }
        let server = signer();
        let player = signer();
        let cs: Vec<Credential> = (0..3)
            .map(|_| {
                credential(&server, &player.identity().0)
                    .sign(&server)
                    .unwrap()
            })
            .collect();
        let hashes: Vec<[u8; 32]> = cs.iter().map(Credential::hash).collect();
        let mut c = cs[1].clone();
        c.anchor = Some(Anchor::of(day_of(c.issued), &hashes, 1, "sim"));
        assert_eq!(c.verify().unwrap().anchored, Some(day_of(c.issued)));
        // The anchor isn't signed: a wrong one is refused, not hidden.
        c.anchor.as_mut().unwrap().index = 0;
        c.anchor.as_mut().unwrap().path[0] = format!("r:{}", hex(&[0; 32]));
        assert!(c.verify().is_err());
    }
}
