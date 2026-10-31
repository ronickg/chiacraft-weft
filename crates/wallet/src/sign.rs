//! Signing, behind a narrow interface (identity-research.md 1.6, decision 4): the game holds an
//! identity key's secret only inside an [`IdentitySigner`], which signs nothing but chiacraft's
//! own texts, each as a Chia signed message (CHIP-0002: the tree hash of
//! `("Chia Signed Message" . message)`, AugSchemeMPL), so a signature can never be replayed as a
//! coin spend, and an external Sage could make the same signature through `signMessage`.
//! Ported from `@chia/core` `dapp.ts` (`signMessage`). A helper process can implement [`Signer`]
//! later without the game changing.
use crate::{Error, Result, keys::IdentityKey, secret::Secret};
use chia_bls::{PublicKey, SecretKey, Signature};
use clvm_utils::{tree_hash_atom, tree_hash_pair};

/// Every text an identity key signs starts with this.
pub const PREFIX: &str = "chiacraft ";

/// The 32 bytes a Chia wallet signs for `message` (CHIP-0002).
pub fn message_hash(message: &[u8]) -> [u8; 32] {
    tree_hash_pair(
        tree_hash_atom(b"Chia Signed Message"),
        tree_hash_atom(message),
    )
    .to_bytes()
}

/// Whether `signature` is `key`'s signature of `message` as a Chia signed message.
pub fn verify(key: &IdentityKey, message: &[u8], signature: &[u8]) -> bool {
    let Ok(signature): std::result::Result<[u8; 96], _> = signature.try_into() else {
        return false;
    };
    let (Ok(public), Ok(signature)) = (
        PublicKey::from_bytes(&key.0),
        Signature::from_bytes(&signature),
    ) else {
        return false;
    };
    chia_bls::verify(&signature, &public, message_hash(message))
}

/// What the game may ask of a key: its public half and a signature of a chiacraft text.
pub trait Signer: Send + Sync {
    fn identity(&self) -> IdentityKey;
    /// Sign `message` as a Chia signed message; refused unless it starts with [`PREFIX`].
    fn sign(&self, message: &str) -> Result<[u8; 96]>;
}

/// One identity key's secret, in memory while the account is unlocked.
pub struct IdentitySigner {
    index: u32,
    public: IdentityKey,
    secret: Secret<[u8; 32]>,
}
impl std::fmt::Debug for IdentitySigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IdentitySigner({}, {:?})", self.index, self.public)
    }
}
impl IdentitySigner {
    pub(crate) fn new(master: &SecretKey, index: u32) -> Self {
        let secret = crate::keys::identity_secret(master, index);
        Self {
            index,
            public: IdentityKey(secret.public_key().to_bytes()),
            secret: Secret::new(secret.to_bytes()),
        }
    }
    /// A signer for identity key `index` of `mnemonic`.
    pub fn from_mnemonic(mnemonic: &crate::Mnemonic, index: u32) -> Self {
        Self::new(&crate::keys::master(&mnemonic.seed()), index)
    }
    pub fn index(&self) -> u32 {
        self.index
    }
}
impl Signer for IdentitySigner {
    fn identity(&self) -> IdentityKey {
        self.public
    }
    fn sign(&self, message: &str) -> Result<[u8; 96]> {
        if !message.starts_with(PREFIX) {
            return Err(Error::new(
                "the identity key signs only chiacraft's own texts",
            ));
        }
        let secret = SecretKey::from_bytes(self.secret.expose())
            .map_err(|_| Error::new("identity key damaged in memory"))?;
        Ok(chia_bls::sign(&secret, message_hash(message.as_bytes())).to_bytes())
    }
}

/// A wallet address's key, signing as the owner of what that address holds (a DID, an NFT, CAT
/// coins) for admission proofs (identity row I7): the **synthetic** secret key of wallet key
/// `m/12381/8444/2/index` (unhardened, or hardened as Sage also derives), as Sage's
/// `signMessageByAddress` signs. Its public half is the key whose standard puzzle hash is the
/// address, so a server checks ownership by comparing that puzzle hash with the asset's owner.
/// Like [`IdentitySigner`] it signs only chiacraft's own texts, as Chia signed messages: never a
/// spend.
pub struct OwnerSigner {
    index: u32,
    hardened: bool,
    public: [u8; 48],
    secret: Secret<[u8; 32]>,
}
impl std::fmt::Debug for OwnerSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OwnerSigner({}{}, [redacted])",
            self.index,
            if self.hardened { "h" } else { "" }
        )
    }
}
impl OwnerSigner {
    /// The owner key of wallet address `index` of `mnemonic`.
    pub fn from_mnemonic(mnemonic: &crate::Mnemonic, index: u32, hardened: bool) -> Self {
        Self::from_master(&crate::keys::master(&mnemonic.seed()), index, hardened)
    }
    fn from_master(master: &SecretKey, index: u32, hardened: bool) -> Self {
        use chia_bls::DerivableKey;
        use chia_puzzle_types::DeriveSynthetic;
        let master = master.clone();
        let path = [
            crate::keys::CHIA_ROOT[0],
            crate::keys::CHIA_ROOT[1],
            crate::keys::WALLET_PURPOSE,
            index,
        ];
        let key = if hardened {
            crate::keys::derive_hardened(&master, &path)
        } else {
            path.iter().fold(master, |k, i| k.derive_unhardened(*i))
        };
        let synthetic = key.derive_synthetic();
        Self {
            index,
            hardened,
            public: synthetic.public_key().to_bytes(),
            secret: Secret::new(synthetic.to_bytes()),
        }
    }
    /// The wallet address of `mnemonic` whose standard puzzle hash is `owner`, among its first
    /// `count` unhardened and hardened addresses (linking a DID: which key owns it).
    pub fn find(mnemonic: &crate::Mnemonic, owner: [u8; 32], count: u32) -> Option<Self> {
        let master = crate::keys::master(&mnemonic.seed());
        (0..count).find_map(|i| {
            [false, true]
                .into_iter()
                .map(|h| Self::from_master(&master, i, h))
                .find(|s| s.puzzle_hash() == owner)
        })
    }
    pub fn index(&self) -> u32 {
        self.index
    }
    pub fn hardened(&self) -> bool {
        self.hardened
    }
    /// The synthetic public key (what a proof carries).
    pub fn public(&self) -> [u8; 48] {
        self.public
    }
    /// The address's standard puzzle hash (the owner a DID or NFT held there names).
    pub fn puzzle_hash(&self) -> [u8; 32] {
        crate::keys::p2_of_synthetic(&self.public).unwrap_or([0; 32])
    }
    /// Sign `message` as a Chia signed message; refused unless it starts with [`PREFIX`].
    pub fn sign(&self, message: &str) -> Result<[u8; 96]> {
        if !message.starts_with(PREFIX) {
            return Err(Error::new(
                "a wallet key signs only chiacraft's own texts here",
            ));
        }
        let secret = SecretKey::from_bytes(self.secret.expose())
            .map_err(|_| Error::new("wallet key damaged in memory"))?;
        Ok(chia_bls::sign(&secret, message_hash(message.as_bytes())).to_bytes())
    }
}

/// Whether `signature` is the G1 key `key`'s signature of `message` (a Chia signed message): an
/// owner key's proof, as [`verify`] checks an identity key's.
pub fn verify_owner(key: &[u8; 48], message: &[u8], signature: &[u8]) -> bool {
    verify(&IdentityKey(*key), message, signature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Mnemonic;

    #[test]
    fn owner_keys_sign_as_their_address() {
        let m = Mnemonic::generate().unwrap();
        let o = OwnerSigner::from_mnemonic(&m, 0, false);
        let facts = crate::keys::facts_of(&m);
        assert_eq!(
            o.puzzle_hash(),
            crate::keys::wallet_puzzle_hash(&facts.master, 0).unwrap()
        );
        let text = "chiacraft sign-in v1\nsession: 00";
        let sig = o.sign(text).unwrap();
        assert!(verify_owner(&o.public(), text.as_bytes(), &sig));
        assert!(!verify_owner(&o.public(), b"chiacraft other", &sig));
        assert!(o.sign("a coin spend").is_err());
        let h = OwnerSigner::from_mnemonic(&m, 0, true);
        assert_ne!(h.puzzle_hash(), o.puzzle_hash());
        assert!(format!("{h:?}").contains("[redacted]"));
        assert!(!format!("{o:?}").contains(&crate::keys::hex(o.secret.expose())));
    }

    #[test]
    fn signs_and_verifies_only_chiacraft_texts() {
        let m = Mnemonic::generate().unwrap();
        let s = IdentitySigner::from_mnemonic(&m, 0);
        let text = "chiacraft sign-in v2\nsession: 00";
        let sig = s.sign(text).unwrap();
        assert!(verify(&s.identity(), text.as_bytes(), &sig));
        assert!(!verify(
            &s.identity(),
            b"chiacraft sign-in v2\nsession: 01",
            &sig
        ));
        let other = IdentitySigner::from_mnemonic(&Mnemonic::generate().unwrap(), 0);
        assert!(!verify(&other.identity(), text.as_bytes(), &sig));
        assert!(!verify(&s.identity(), text.as_bytes(), &sig[..95]));
        assert!(s.sign("a coin spend").is_err());
        assert!(!format!("{s:?}").contains(&keys_hex(&s)));
    }

    fn keys_hex(s: &IdentitySigner) -> String {
        crate::keys::hex(s.secret.expose())
    }

    #[test]
    fn message_hash_is_the_clvm_tree_hash() {
        // sha256(2, sha256(1, "Chia Signed Message"), sha256(1, msg)) by hand.
        use sha2::{Digest, Sha256};
        let atom = |b: &[u8]| -> [u8; 32] {
            let mut h = Sha256::new();
            h.update([1u8]);
            h.update(b);
            h.finalize().into()
        };
        let mut h = Sha256::new();
        h.update([2u8]);
        h.update(atom(b"Chia Signed Message"));
        h.update(atom(b"hello"));
        let want: [u8; 32] = h.finalize().into();
        assert_eq!(message_hash(b"hello"), want);
    }
}
