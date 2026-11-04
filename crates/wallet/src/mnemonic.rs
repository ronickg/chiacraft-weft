//! The account's 24 words (BIP-39, English, 256 bits of entropy), as Sage and Ronald's wallet
//! make them (`@chia/core` `wallet.ts`: `mnemonicGenerate(true)`, `toSeed('')`).
use crate::{Error, Result, secret::Secret};
use bip39::Language;

/// Words in a new account's mnemonic.
pub const WORDS: usize = 24;

/// A mnemonic. Its `Debug` is redacted and it wipes itself on drop (bip39's `zeroize`).
pub struct Mnemonic(bip39::Mnemonic);

impl std::fmt::Debug for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mnemonic({} words, [redacted])", self.0.word_count())
    }
}

/// Why typed words aren't a mnemonic, for the restore screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WordsProblem {
    /// Not 12, 15, 18, 21 or 24 words (the count found).
    Count(usize),
    /// The word at this position (1-based) isn't in the English list.
    Unknown(usize),
    /// Every word is known but the checksum doesn't match (a word is wrong or out of order).
    Checksum,
}
impl std::fmt::Display for WordsProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Count(n) => write!(f, "{n} words: an account has 24 (12 to 24 accepted)"),
            Self::Unknown(i) => write!(f, "word {i} isn't in the word list"),
            Self::Checksum => f.write_str("a word is wrong or out of order (the checksum fails)"),
        }
    }
}

/// Lower-case words separated by single spaces.
fn normalise(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| w.to_ascii_lowercase())
        .collect()
}

impl Mnemonic {
    /// A fresh 24-word mnemonic from the system's randomness.
    pub fn generate() -> Result<Self> {
        let mut entropy = Secret::new([0u8; 32]);
        getrandom::fill(entropy.expose_mut()).map_err(|e| Error::new(format!("randomness: {e}")))?;
        Self::from_entropy(entropy.expose())
    }
    /// The mnemonic of stored entropy (16 to 32 bytes).
    pub fn from_entropy(entropy: &[u8]) -> Result<Self> {
        bip39::Mnemonic::from_entropy_in(Language::English, entropy)
            .map(Self)
            .map_err(|_| Error::new("stored entropy has the wrong length"))
    }
    /// Words a player typed: any case, any spacing.
    pub fn parse(text: &str) -> std::result::Result<Self, WordsProblem> {
        let words = normalise(text);
        if !matches!(words.len(), 12 | 15 | 18 | 21 | 24) {
            return Err(WordsProblem::Count(words.len()));
        }
        if let Some(i) = words
            .iter()
            .position(|w| Language::English.find_word(w).is_none())
        {
            return Err(WordsProblem::Unknown(i + 1));
        }
        let joined = Secret::new(words.join(" "));
        let mut words = words;
        zeroize::Zeroize::zeroize(&mut words);
        bip39::Mnemonic::parse_in_normalized(Language::English, joined.expose())
            .map(Self)
            .map_err(|_| WordsProblem::Checksum)
    }
    /// Whether one word is in the list (the restore screen marks unknown words as they're typed).
    pub fn known_word(word: &str) -> bool {
        Language::English
            .find_word(&word.to_ascii_lowercase())
            .is_some()
    }
    pub fn word_count(&self) -> usize {
        self.0.word_count()
    }
    /// The words, for the backup screen only.
    pub fn words(&self) -> Secret<Vec<String>> {
        Secret::new(self.0.words().map(str::to_owned).collect())
    }
    /// Writes the words to a new file only its owner can read (mode 600 on unix): the live test
    /// tool's `new-wallet`. Refuses a file that exists; the words never pass through a log or a
    /// terminal.
    pub fn write_new_file(&self, path: &std::path::Path) -> Result<()> {
        use std::io::Write;
        let text = Secret::new(self.0.words().collect::<Vec<_>>().join(" ") + "\n");
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(path)
            .map_err(|e| Error::new(format!("{}: {e}", path.display())))?;
        file.write_all(text.expose().as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|e| Error::new(format!("{}: {e}", path.display())))
    }
    /// The entropy: what the keystore keeps.
    pub fn entropy(&self) -> Secret<Vec<u8>> {
        Secret::new(self.0.to_entropy())
    }
    /// The BIP-39 seed with an empty passphrase, as Chia wallets use it.
    pub fn seed(&self) -> Secret<[u8; 64]> {
        Secret::new(self.0.to_seed_normalized(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const TEST_WORDS: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    #[test]
    fn generates_24_words_that_parse_back() {
        let m = Mnemonic::generate().unwrap();
        assert_eq!(m.word_count(), WORDS);
        let words = m.words();
        let again = Mnemonic::parse(&words.expose().join("  ").to_uppercase()).unwrap();
        assert_eq!(again.entropy().expose(), m.entropy().expose());
        assert!(!format!("{m:?}").contains(&words.expose()[0]));
    }

    #[test]
    fn the_bip39_vector_gives_its_seed() {
        // The test vector Ronald's wallet checks (chia-wallet-app/__tests__/keys.harness.ts).
        let m = Mnemonic::parse(TEST_WORDS).unwrap();
        let seed = m.seed();
        let hex: String = seed.expose()[..16]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(hex, "5eb00bbddcf069084889a8ab91555681");
    }

    #[test]
    fn says_what_is_wrong() {
        assert_eq!(Mnemonic::parse("abandon abandon").unwrap_err(), WordsProblem::Count(2));
        let mut words: Vec<&str> = TEST_WORDS.split(' ').collect();
        words[4] = "notaword";
        assert_eq!(Mnemonic::parse(&words.join(" ")).unwrap_err(), WordsProblem::Unknown(5));
        words[4] = "abandon";
        words[11] = "abandon";
        assert_eq!(Mnemonic::parse(&words.join(" ")).unwrap_err(), WordsProblem::Checksum);
        assert!(Mnemonic::known_word("About"));
        assert!(!Mnemonic::known_word("aboutt"));
    }
}
