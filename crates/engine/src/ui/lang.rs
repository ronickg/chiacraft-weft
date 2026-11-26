//! Translations. Only the selected language and the English fallback are ever loaded; the
//! language list reads just the first line of each file.
//!
//! File format (`lang/<code>.lang` in any mod or pack): `key = value` per line, `#` comments, the
//! first entry should be `language.name`. An indented line continues the previous value on a new
//! line. Values are templates, see `text.rs`.

use std::cell::RefCell;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::assets::Assets;

pub const FALLBACK: &str = "en_us";
/// The pseudo-locale: every English string 40% longer, accented and bracketed (`[Ŝàvé ŵôŕłð ~~]`),
/// so the text-fit check (`ui::audit`) sees how screens hold a long language and which text
/// isn't a translation (it shows unbracketed). Not listed with the languages; set it by
/// `core:language`.
pub const PSEUDO: &str = "en_xa";

pub struct Lang {
    pub code: String,
    entries: FxHashMap<String, String>,
    fallback: FxHashMap<String, String>,
    pub dev: bool,
    reported: RefCell<FxHashSet<String>>,
}

fn parse(src: &str, into: &mut FxHashMap<String, String>) {
    for (_, k, v) in crate::kv::long_pairs(src) {
        into.insert(k.to_string(), v.replace("\\n", "\n"));
    }
}

/// Every mod's file for `code`, then the packs' on top: a pack's file changes only the keys it has.
fn load_code(assets: &Assets, code: &str) -> FxHashMap<String, String> {
    let mut map = FxHashMap::default();
    for (name, path) in assets.layers("lang", "lang") {
        if name.ends_with(&format!(":{code}")) {
            if let Ok(src) = std::fs::read_to_string(&path) {
                parse(&src, &mut map);
            }
        }
    }
    map
}

/// A pseudo-locale value: letters accented, each line bracketed and 40% longer, template tags
/// (`{0}`, `{#rrggbb}`, `{key:...}`) and literal braces kept as they are. Colors and the
/// language's name stay English.
fn pseudo(key: &str, v: &str) -> String {
    if key.starts_with("color.") || key == "language.name" || v.trim().is_empty() {
        return v.to_string();
    }
    const FROM: &str = "aceinouyAEIOUCNSZswrdl";
    const TO: [char; 22] = ['à', 'ç', 'é', 'î', 'ñ', 'ô', 'ü', 'ÿ', 'À', 'É', 'Î', 'Ô', 'Ü', 'Ç', 'Ñ', 'Ŝ', 'Ž', 'ŝ', 'ŵ', 'ŕ', 'ð', 'ł'];
    let line = |l: &str| {
        let (mut out, mut letters, mut chars) = (String::from("["), 0usize, l.chars().peekable());
        while let Some(c) = chars.next() {
            if c == '{' {
                out.push(c);
                if chars.peek() == Some(&'{') {
                    out.push(chars.next().unwrap());
                    continue;
                }
                for t in chars.by_ref() {
                    out.push(t);
                    if t == '}' {
                        break;
                    }
                }
                continue;
            }
            if c.is_alphabetic() {
                letters += 1;
            }
            out.push(FROM.find(c).map_or(c, |i| TO[i]));
        }
        let pad = (letters * 2).div_ceil(5);
        if pad > 0 {
            out.push(' ');
            out.extend(std::iter::repeat_n('~', pad));
        }
        out.push(']');
        out
    };
    v.split('\n').map(line).collect::<Vec<_>>().join("\n")
}

impl Lang {
    pub fn load(assets: &Assets, code: &str, dev: bool) -> Lang {
        let fallback = load_code(assets, FALLBACK);
        let entries = match code {
            FALLBACK => FxHashMap::default(),
            PSEUDO => fallback
                .iter()
                .map(|(k, v)| (k.clone(), pseudo(k, v)))
                .collect(),
            _ => load_code(assets, code),
        };
        Lang { code: code.to_string(), entries, fallback, dev, reported: RefCell::new(FxHashSet::default()) }
    }

    /// Template for a key. A missing key falls back to English; if that is missing too, the key
    /// itself shows (and is logged once).
    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        // No key: no text (a bound text that set nothing this frame), not a missing one.
        if key.is_empty() {
            return "";
        }
        if let Some(v) = self.entries.get(key).or_else(|| self.fallback.get(key)) {
            return v;
        }
        if self.reported.borrow_mut().insert(key.to_string()) {
            log::warn!("missing translation: {key}");
        }
        key
    }

    /// A color a key holds (`color.button = {#d8dceb}`; `#d8dceb` or eight digits with alpha
    /// too): the engine's and mods' named colors, which packs change like any translation.
    /// `0xRRGGBBAA`, `None` when missing or not a color.
    pub fn color(&self, key: &str) -> Option<u32> {
        let v = self.entries.get(key).or_else(|| self.fallback.get(key))?;
        let h = v
            .trim()
            .trim_start_matches('{')
            .trim_end_matches('}')
            .strip_prefix('#')?;
        let c = u32::from_str_radix(h, 16).ok()?;
        match h.len() {
            6 => Some(c << 8 | 0xFF),
            8 => Some(c),
            _ => None,
        }
    }

    /// Every `color.<name>` key: (name, color) and the names set to `none`.
    pub fn colors(&self) -> (FxHashMap<String, u32>, FxHashSet<String>) {
        let (mut colors, mut nones) = (FxHashMap::default(), FxHashSet::default());
        for k in self.fallback.keys().chain(self.entries.keys()) {
            let Some(name) = k.strip_prefix("color.") else { continue };
            match self.color(k) {
                Some(c) => {
                    colors.insert(name.to_string(), c);
                }
                None if self.get(k).trim() == "none" => {
                    nones.insert(name.to_string());
                }
                None => {}
            }
        }
        (colors, nones)
    }

    pub fn has(&self, key: &str) -> bool {
        self.entries.contains_key(key) || self.fallback.contains_key(key)
    }

    /// (code, display name) for every language any mod ships, reading only the first line.
    pub fn available(assets: &Assets) -> Vec<(String, String)> {
        let mut out: FxHashMap<String, String> = FxHashMap::default();
        for (name, path) in assets.list("lang", "lang") {
            let code = name.rsplit(':').next().unwrap_or("").to_string();
            if out.get(&code).is_some_and(|n| n != &code) {
                continue;
            }
            let first = std::fs::read_to_string(&path).ok().and_then(|s| {
                s.lines()
                    .find(|l| l.trim_start().starts_with("language.name"))
                    .and_then(|l| l.split_once('='))
                    .map(|(_, v)| v.trim().to_string())
            });
            out.insert(code.clone(), first.unwrap_or(code));
        }
        let mut v: Vec<_> = out.into_iter().collect();
        v.sort();
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indented_lines_continue_the_value() {
        let mut m = FxHashMap::default();
        parse("# c\na = one\n    two\n\n\tthree\nb = x\\ny\n", &mut m);
        assert_eq!(m["a"], "one\ntwo\nthree");
        assert_eq!(m["b"], "x\ny");
    }
}
