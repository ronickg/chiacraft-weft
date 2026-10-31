//! [`Secret`]: the one type wallet secrets travel in. It redacts itself in `Debug`, has no
//! `Display`, no `Serialize` and no `Clone` (a copy is made on purpose with
//! [`Secret::duplicate`]), and wipes its memory when dropped. So a secret can't end up in a log
//! line, a telemetry event, a crash report or a JSON file by accident: formatting it prints
//! `[redacted]`, and serialising it doesn't compile. `tests/no_secret_leaks.rs` checks the rest
//! (no log macro or telemetry call in this crate takes a secret's contents).
use zeroize::Zeroize;

/// A wallet secret (entropy, seed, words, a passphrase, a private key's bytes).
pub struct Secret<T: Zeroize>(T);

impl<T: Zeroize> Secret<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
    /// The secret itself, for the code that must use it (deriving keys, the backup screen).
    /// Never format, log or send what this returns.
    pub fn expose(&self) -> &T {
        &self.0
    }
    pub fn expose_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
impl<T: Zeroize + Clone> Secret<T> {
    /// A second copy, made on purpose (both are wiped when dropped).
    pub fn duplicate(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}
impl<T: Zeroize> std::fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[redacted]")
    }
}
impl<T: Zeroize> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// A passphrase (empty: the file store keeps the account unprotected, as Sage's own file does).
pub type Passphrase = Secret<String>;

impl Secret<String> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_is_redacted() {
        let s = Secret::new(String::from("abandon ability able"));
        assert_eq!(format!("{s:?}"), "[redacted]");
        assert_eq!(format!("{:?}", Some(&s)), "Some([redacted])");
        assert_eq!(s.expose(), "abandon ability able");
    }
}
