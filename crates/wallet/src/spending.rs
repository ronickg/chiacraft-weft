//! Which network the wallet spends on, and how much it may spend (identity row I12).
//!
//! **testnet11 is the default.** Mainnet is an explicit opt-in: the network setting says
//! `mainnet` **and** a second setting carries the exact sentence [`MAINNET_CONFIRM`]. Only then
//! is a [`MainnetConsent`] made, and the chain reader refuses to start on mainnet without one
//! (`chain::ChainConfig::mainnet`). Every caller (the dedicated server, the game's wallet, the
//! sponsor, the live test tool) goes through [`choose_network`].
//!
//! **Hard caps** ([`SpendCaps`]) on every spend the wallet signs: the fee of one transaction, the
//! price of an offer it makes, what taking an offer pays, a DID's fee, the anchor's fee and a
//! split's coins. testnet11's defaults are unlimited (test coins); mainnet's are small. A spend
//! above a cap is refused before anything is built or signed, and the refusal names the cap.
//!
//! No chain feature needed: the game shows the labels and checks the settings without it.
use crate::{Error, Network, Result};

/// The sentence that confirms mainnet, word for word.
pub const MAINNET_CONFIRM: &str = "I understand this spends real XCH";

/// Mojos per XCH.
pub const MOJOS_PER_XCH: u64 = 1_000_000_000_000;

/// Proof that whoever configured this process typed [`MAINNET_CONFIRM`]: the chain reader needs
/// one to start on mainnet. Made only by [`MainnetConsent::confirm`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MainnetConsent(());

impl MainnetConsent {
    /// The consent, when `text` is the confirmation sentence (surrounding spaces and quotes
    /// ignored); `setting` names where it is set, for the refusal.
    pub fn confirm(text: &str, setting: &str) -> Result<Self> {
        let t = text.trim().trim_matches('"').trim();
        if t == MAINNET_CONFIRM {
            Ok(Self(()))
        } else {
            Err(Error::new(format!("mainnet spends real XCH: it also needs {setting} = \"{MAINNET_CONFIRM}\" (word for word)")))
        }
    }
}

/// The network a setting names (`testnet11`, the default when empty, or `mainnet`), and for
/// mainnet the consent the confirmation setting gives. `setting`/`confirm_setting` name the two
/// settings, for the refusals.
pub fn choose_network(network: &str, confirm: &str, setting: &str, confirm_setting: &str) -> Result<(Network, Option<MainnetConsent>)> {
    match network.trim() {
        "" | "testnet11" => Ok((Network::Testnet11, None)),
        "mainnet" => {
            let consent = MainnetConsent::confirm(confirm, confirm_setting)?;
            Ok((Network::Mainnet, Some(consent)))
        }
        other => Err(Error::new(format!("{setting} = {other}: unknown (testnet11, the default, or mainnet)"))),
    }
}

/// The network's name as the settings spell it.
pub fn network_name(network: Network) -> &'static str {
    match network {
        Network::Testnet11 => "testnet11",
        Network::Mainnet => "mainnet",
    }
}

/// The short label screens show beside every amount: "testnet11 · test coins" or
/// "MAINNET · real XCH".
pub fn network_label(network: Network) -> &'static str {
    match network {
        Network::Testnet11 => "testnet11 · test coins",
        Network::Mainnet => "MAINNET · real XCH",
    }
}

/// The warning logged when something starts on mainnet.
pub fn mainnet_warning(what: &str) -> String {
    format!(
        "{what} runs on MAINNET: payments, offers, DIDs and fees spend REAL XCH from this wallet. Use a throwaway wallet holding a small amount; spends above the caps are refused"
    )
}

/// The coin's unit on a network: `XCH` or `TXCH`.
pub fn unit(network: Network) -> &'static str {
    match network {
        Network::Testnet11 => "TXCH",
        Network::Mainnet => "XCH",
    }
}

/// Mojos as XCH (TXCH on testnet11), trimmed: `0.0001 XCH`.
pub fn amount(mojos: u64, network: Network) -> String {
    let whole = mojos / MOJOS_PER_XCH;
    let frac = mojos % MOJOS_PER_XCH;
    let unit = unit(network);
    if frac == 0 {
        return format!("{whole} {unit}");
    }
    let f = format!("{frac:012}");
    format!("{whole}.{} {unit}", f.trim_end_matches('0'))
}

/// Mojos with the XCH amount beside them: `100000000 mojos (0.0001 XCH)`.
pub fn mojos_and_amount(mojos: u64, network: Network) -> String {
    format!("{mojos} mojos ({})", amount(mojos, network))
}

/// The most the wallet signs for, per kind of spend, in mojos. `u64::MAX`: no cap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpendCaps {
    /// The network the caps were made for (amounts in refusals are shown in its unit).
    pub network: Network,
    /// The fee of one transaction (`max-fee`).
    pub max_fee: u64,
    /// The price an offer this wallet makes asks (`max-offer-price`).
    pub max_offer_price: u64,
    /// What taking an offer pays, besides the fee: the XCH it requests plus royalties
    /// (`max-take`).
    pub max_take: u64,
    /// The fee of one DID this wallet makes or sponsors (`max-did-fee`).
    pub max_did_fee: u64,
    /// The achievements anchor's fee (`max-anchor-fee`).
    pub max_anchor_fee: u64,
    /// The mojos a split moves into new coins (`max-split`).
    pub max_split: u64,
    /// The mojos one plain payment sends to another address (`max-send`).
    pub max_send: u64,
}

/// The cap names, as settings and flags spell them after a prefix (`chain-max-fee`, `--max-fee`).
pub const CAP_NAMES: [&str; 7] = ["max-fee", "max-offer-price", "max-take", "max-did-fee", "max-anchor-fee", "max-split", "max-send"];

impl SpendCaps {
    /// testnet11: no caps (test coins, worth nothing).
    pub fn testnet11() -> Self {
        Self {
            network: Network::Testnet11,
            max_fee: u64::MAX,
            max_offer_price: u64::MAX,
            max_take: u64::MAX,
            max_did_fee: u64::MAX,
            max_anchor_fee: u64::MAX,
            max_split: u64::MAX,
            max_send: u64::MAX,
        }
    }

    /// Mainnet: small, for a throwaway wallet holding a little XCH.
    pub fn mainnet() -> Self {
        Self {
            network: Network::Mainnet,
            // 0.0001 XCH: well above what a busy mempool asks for a simple spend.
            max_fee: 100_000_000,
            // 0.01 XCH: an admission price for a test.
            max_offer_price: 10_000_000_000,
            max_take: 10_000_000_000,
            // 0.0001 XCH per DID.
            max_did_fee: 100_000_000,
            max_anchor_fee: 100_000_000,
            // A pool of 1-mojo coins is a few mojos; a million is plenty.
            max_split: 1_000_000,
            // 0.01 XCH: moving test amounts between a test's own wallets.
            max_send: 10_000_000_000,
        }
    }

    /// The defaults for a network.
    pub fn for_network(network: Network) -> Self {
        match network {
            Network::Testnet11 => Self::testnet11(),
            Network::Mainnet => Self::mainnet(),
        }
    }

    /// Change caps from settings: `lookup(name)` gives the value set for each of [`CAP_NAMES`]
    /// (None or empty: keep the default). `prefix` is how the settings spell them
    /// (`chain-`), for errors.
    pub fn with_overrides(mut self, prefix: &str, lookup: impl Fn(&str) -> Option<String>) -> Result<Self> {
        for name in CAP_NAMES {
            let Some(text) = lookup(name).filter(|t| !t.trim().is_empty()) else {
                continue;
            };
            let value: u64 = text
                .trim()
                .parse()
                .map_err(|_| Error::new(format!("{prefix}{name} = {}: not a number of mojos", text.trim())))?;
            *self.slot(name) = value;
        }
        Ok(self)
    }

    fn slot(&mut self, name: &str) -> &mut u64 {
        match name {
            "max-fee" => &mut self.max_fee,
            "max-offer-price" => &mut self.max_offer_price,
            "max-take" => &mut self.max_take,
            "max-did-fee" => &mut self.max_did_fee,
            "max-anchor-fee" => &mut self.max_anchor_fee,
            "max-send" => &mut self.max_send,
            _ => &mut self.max_split,
        }
    }

    /// The value of a cap by name.
    pub fn get(&self, name: &str) -> u64 {
        let mut copy = *self;
        *copy.slot(name)
    }

    /// Refuse `mojos` of `what` above the cap `name`.
    pub fn check(&self, name: &str, what: &str, mojos: u64) -> Result<()> {
        let cap = self.get(name);
        if mojos > cap {
            return Err(Error::new(format!(
                "refused: {what} of {} is above the cap {name} = {} on {}",
                mojos_and_amount(mojos, self.network),
                mojos_and_amount(cap, self.network),
                network_name(self.network)
            )));
        }
        Ok(())
    }

    /// One transaction's fee.
    pub fn check_fee(&self, fee: u64) -> Result<()> {
        self.check("max-fee", "the fee", fee)
    }

    /// The caps as `name = value` lines (for a tool's `--help` and the logs).
    pub fn describe(&self) -> String {
        CAP_NAMES
            .iter()
            .map(|n| {
                let v = self.get(n);
                if v == u64::MAX { format!("{n} = none") } else { format!("{n} = {}", mojos_and_amount(v, self.network)) }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mainnet_needs_both_settings() {
        assert_eq!(choose_network("", "", "chain", "chain-mainnet-confirm").unwrap(), (Network::Testnet11, None));
        assert_eq!(choose_network("testnet11", "", "chain", "c").unwrap().0, Network::Testnet11);
        let e = choose_network("mainnet", "", "chain", "chain-mainnet-confirm").unwrap_err();
        assert!(e.to_string().contains("chain-mainnet-confirm"), "{e}");
        assert!(e.to_string().contains(MAINNET_CONFIRM), "{e}");
        assert!(choose_network("mainnet", "i understand this spends real xch", "a", "b").is_err());
        assert!(choose_network("mainnet", "yes", "a", "b").is_err());
        let (n, consent) = choose_network("mainnet", "  \"I understand this spends real XCH\" ", "chain", "chain-mainnet-confirm").unwrap();
        assert_eq!(n, Network::Mainnet);
        assert!(consent.is_some());
        assert!(choose_network("testnet10", "", "chain", "c").is_err());
    }

    #[test]
    fn caps_refuse_above_and_name_the_cap() {
        let m = SpendCaps::mainnet();
        assert!(m.check_fee(100_000_000).is_ok());
        let e = m.check_fee(100_000_001).unwrap_err().to_string();
        assert!(e.contains("max-fee"), "{e}");
        assert!(e.contains("0.0001 XCH"), "{e}");
        assert!(e.contains("mainnet"), "{e}");
        assert!(
            m.check("max-offer-price", "the price", 10_000_000_001)
                .is_err()
        );
        let t = SpendCaps::testnet11();
        assert!(t.check_fee(u64::MAX).is_ok());
        let lowered = m
            .with_overrides("chain-", |n| (n == "max-fee").then(|| "5".to_owned()))
            .unwrap();
        assert_eq!(lowered.max_fee, 5);
        assert_eq!(lowered.max_take, m.max_take);
        assert!(
            m.with_overrides("chain-", |_| Some("lots".into()))
                .unwrap_err()
                .to_string()
                .contains("chain-max-fee")
        );
        assert!(m.describe().contains("max-did-fee = 100000000 mojos"));
        assert!(t.describe().contains("max-fee = none"));
    }

    #[test]
    fn amounts_and_labels() {
        assert_eq!(amount(2_000_000_000_000, Network::Mainnet), "2 XCH");
        assert_eq!(amount(300_000_000_000, Network::Testnet11), "0.3 TXCH");
        assert_eq!(amount(1, Network::Mainnet), "0.000000000001 XCH");
        assert!(network_label(Network::Mainnet).contains("MAINNET"));
        assert!(network_label(Network::Testnet11).contains("test coins"));
    }
}
