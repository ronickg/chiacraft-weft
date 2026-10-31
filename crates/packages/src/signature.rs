//! Who made a package: a signature envelope beside it, outside the files its hash covers, so
//! reviewers can add their own later without changing the package (or any world's pin).
//!
//! `package.sig` in a package folder (and beside the package in the store) holds:
//!
//! ```text
//! chiacraft package signature 1
//! package <package hash, hex>
//! creator <the creator's content key: Ed25519 public key, hex>
//! vouch <identity hex>:<the identity key's signature of the content key, hex>
//! signature <the content key's Ed25519 signature of `signed_text(package hash)`, hex>
//! ```
//!
//! The content key is derived from the creator's account words on its own label and vouched for
//! by the account's identity key (later linked to a DID), so trusting a creator means trusting an
//! identity, not a name anyone can copy. This module reads and writes the envelope; checking the
//! signatures is the game's (it has the keys' code).

/// The envelope's file name in a package folder; never part of the package's files.
pub const SIG_FILE: &str = "package.sig";

const HEADER: &str = "chiacraft package signature 1";

/// What the content key signs for a package: labelled, so a package signature can never be
/// mistaken for any other signature the key or the account makes.
pub fn signed_text(package_hex: &str) -> String {
    format!("chiacraft package v1\npackage: {package_hex}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub package: String,
    pub creator: String,
    pub vouch: String,
    pub signature: String,
}

impl Envelope {
    pub fn text(&self) -> String {
        format!("{HEADER}\npackage {}\ncreator {}\nvouch {}\nsignature {}\n", self.package, self.creator, self.vouch, self.signature)
    }

    pub fn parse(text: &str) -> Result<Envelope, String> {
        if text.len() > 4096 {
            return Err("signature file too big".into());
        }
        let mut lines = text.lines();
        if lines.next() != Some(HEADER) {
            return Err("not a package signature".into());
        }
        let mut field = |name: &str| -> Result<String, String> {
            let line = lines.next().ok_or_else(|| format!("no {name} line"))?;
            let v = line.strip_prefix(name).and_then(|r| r.strip_prefix(' ')).ok_or_else(|| format!("expected a {name} line"))?;
            if v.is_empty() || !v.chars().all(|c| c.is_ascii_hexdigit() || c == ':') {
                return Err(format!("bad {name}"));
            }
            Ok(v.to_string())
        };
        let e = Envelope { package: field("package")?, creator: field("creator")?, vouch: field("vouch")?, signature: field("signature")? };
        Ok(e)
    }
}

/// A content key as people see it: its first 8 hex digits in two groups, `ab12 cd34`.
pub fn fingerprint(key_hex: &str) -> String {
    let k = key_hex.get(..8).unwrap_or(key_hex);
    format!("{} {}", &k[..k.len().min(4)], k.get(4..).unwrap_or(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_refusals() {
        let e = Envelope { package: "ab".repeat(32), creator: "cd".repeat(32), vouch: format!("{}:{}", "ef".repeat(48), "01".repeat(96)), signature: "23".repeat(64) };
        assert_eq!(Envelope::parse(&e.text()).unwrap(), e);
        assert!(Envelope::parse(&e.text().replace("creator", "maker")).is_err());
        assert!(Envelope::parse("hello").is_err());
        assert_eq!(fingerprint("ab12cd34ef"), "ab12 cd34");
        assert!(signed_text("ff").starts_with("chiacraft package v1\n"));
    }
}
