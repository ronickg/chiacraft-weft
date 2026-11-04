//! Who made a data package, and which packages and creators the player trusts (chiacraft plan,
//! phase 2). A package's creator signs it with the content key of their account
//! (`chiacraft_packages::signature`); the account's identity key vouches for that key. When a
//! world needs packages this game lacks, the player is asked once (the join sheet); what they
//! accept is remembered here, per package and, if they say so, per creator.

use std::path::{Path, PathBuf};

use chiacraft_packages::identity::FileManifest;
pub use chiacraft_packages::signature::fingerprint;
use chiacraft_packages::signature::{Envelope, SIG_FILE, signed_text};

/// A package's creator, checked: their content key and the account identity behind it (hex).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Creator {
    pub key: String,
    pub identity: String,
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

/// Checks a signature envelope for package `package_hex`: the content key's signature of the
/// package, and the identity key's vouching for the content key.
pub fn verify(envelope: &str, package_hex: &str) -> Result<Creator, String> {
    let e = Envelope::parse(envelope)?;
    if e.package != package_hex {
        return Err("the signature is for another package".into());
    }
    let key = iroh::PublicKey::from_bytes(&unhex::<32>(&e.creator).ok_or("a bad creator key")?).map_err(|_| "a bad creator key")?;
    let sig = iroh::Signature::from_bytes(&unhex::<64>(&e.signature).ok_or("a bad signature")?);
    key.verify(signed_text(package_hex).as_bytes(), &sig)
        .map_err(|_| "the signature doesn't check out")?;
    let identity = chiacraft_account::check_content_vouch(&e.vouch, &e.creator)
        .ok_or("the creator's account doesn't vouch for the key that signed")?;
    Ok(Creator { key: e.creator, identity })
}

/// Signs the package in folder `dir` with the open account's content key: writes `package.sig`
/// there. Gives the package hash and the creator.
pub fn sign_dir(dir: &Path) -> Result<(String, Creator), String> {
    let session = chiacraft_account::current().ok_or("no account is open")?;
    let hash = FileManifest::of_dir(dir)?.hash().hex();
    let secret = iroh::SecretKey::from_bytes(&session.content_seed());
    let key = hex(secret.public().as_bytes());
    let envelope = Envelope {
        package: hash.clone(),
        creator: key.clone(),
        vouch: session.content_vouch_line(&key)?,
        signature: hex(&secret.sign(signed_text(&hash).as_bytes()).to_bytes()),
    };
    let creator = verify(&envelope.text(), &hash)?;
    std::fs::write(dir.join(SIG_FILE), envelope.text()).map_err(|e| e.to_string())?;
    Ok((hash, creator))
}

/// `weft --sign-package <dir>`: opens the account of `run_dir` and signs the package in `dir`.
pub fn sign_with_account(run_dir: &Path, dir: &Path) -> Result<(String, Creator), String> {
    chiacraft_account::open(run_dir)?;
    sign_dir(dir)
}

/// What the player trusts: packages (by hash) and creators (by content key). Kept in
/// `trust.txt` in the run folder (`test-trust.txt` for automated runs, so they never change the
/// player's).
pub struct Trust {
    path: PathBuf,
    pub creators: Vec<Creator>,
    pub packages: Vec<String>,
}

impl Trust {
    pub fn load(run_dir: &Path) -> Trust {
        let name = if crate::config::automated() { "test-trust.txt" } else { "trust.txt" };
        let path = run_dir.join(name);
        let mut t = Trust { path, creators: Vec::new(), packages: Vec::new() };
        for line in std::fs::read_to_string(&t.path).unwrap_or_default().lines() {
            let mut parts = line.split_whitespace();
            match (parts.next(), parts.next(), parts.next()) {
                (Some("creator"), Some(key), Some(identity)) => t
                    .creators
                    .push(Creator { key: key.into(), identity: identity.into() }),
                (Some("package"), Some(hash), None) => t.packages.push(hash.into()),
                _ => {}
            }
        }
        t
    }

    pub fn save(&self) {
        let mut s = String::from("# Data packages and creators this player trusts (chiacraft): not asked about again.\n");
        for c in &self.creators {
            s.push_str(&format!("creator {} {}\n", c.key, c.identity));
        }
        for p in &self.packages {
            s.push_str(&format!("package {p}\n"));
        }
        if let Err(e) = std::fs::write(&self.path, s) {
            log::warn!("saving {}: {e}", self.path.display());
        }
    }

    pub fn trusts_creator(&self, c: &Creator) -> bool {
        self.creators.iter().any(|t| t == c)
    }

    /// Trusted as this exact package, or by its (checked) creator.
    pub fn trusts(&self, package_hex: &str, creator: Option<&Creator>) -> bool {
        self.packages.iter().any(|p| p == package_hex) || creator.is_some_and(|c| self.trusts_creator(c))
    }

    pub fn add_package(&mut self, package_hex: &str) {
        if !self.packages.iter().any(|p| p == package_hex) {
            self.packages.push(package_hex.to_string());
        }
    }

    pub fn add_creator(&mut self, c: &Creator) {
        if !self.trusts_creator(c) {
            self.creators.push(c.clone());
        }
    }
}
