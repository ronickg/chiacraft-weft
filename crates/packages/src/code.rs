//! A package's sandboxed code, as far as the game needs it without running it: which version of
//! the mod API (`chiacraft:mods@<major>.<minor>`, `docs/sandboxed-mods.md`) a component was built
//! for, read from its imports. The host puts it in its offers, so a joining game refuses a mod
//! that needs a newer API before downloading anything; the sandbox checks again when it loads.

/// The mod API this game runs: a component built for 1.x with x up to this loads.
pub const API: (u32, u32) = (1, 1);

/// The newest `chiacraft:mods` version among a component's imports, as `(major, minor)`; `None`
/// if it imports none of the API. An import outside the API, a version that doesn't parse, or
/// bytes that aren't a component are errors.
pub fn api_of(component: &[u8]) -> Result<Option<(u32, u32)>, String> {
    use wasmparser::{Parser, Payload};
    let mut depth = 0u32;
    let mut newest: Option<(u32, u32)> = None;
    let mut is_component = false;
    for payload in Parser::new(0).parse_all(component) {
        match payload.map_err(|e| format!("not a component: {e}"))? {
            Payload::Version { encoding, .. } if depth == 0 => {
                is_component = encoding == wasmparser::Encoding::Component;
                if !is_component {
                    return Err("a core module, not a component".into());
                }
            }
            Payload::ModuleSection { .. } | Payload::ComponentSection { .. } => depth += 1,
            Payload::End(_) => depth = depth.saturating_sub(1),
            Payload::ComponentImportSection(reader) if depth == 0 => {
                for import in reader {
                    let name = import.map_err(|e| format!("not a component: {e}"))?.name.0;
                    let v = api_version(name)?;
                    newest = newest.max(Some(v));
                }
            }
            _ => {}
        }
    }
    if !is_component {
        return Err("not a component".into());
    }
    Ok(newest)
}

/// The version of one import, `chiacraft:mods/<interface>@<major>.<minor>.<patch>`.
fn api_version(name: &str) -> Result<(u32, u32), String> {
    let shown: String = name.chars().take(80).collect();
    let Some(rest) = name.strip_prefix("chiacraft:mods/") else {
        return Err(format!("it imports {shown}, which no sandboxed mod gets (files, network, clocks)"));
    };
    let version = rest.split_once('@').map(|(_, v)| v).unwrap_or("");
    let mut parts = version.split('.').map(|p| p.parse::<u32>().ok());
    match (parts.next().flatten(), parts.next().flatten()) {
        (Some(major), Some(minor)) => Ok((major, minor)),
        _ => Err(format!("it imports {shown}, without a version this game reads")),
    }
}

/// Whether this game runs a component built for API `v`.
pub fn runs(v: (u32, u32)) -> bool {
    v.0 == API.0 && v.1 <= API.1
}

/// `"1.1"` for an offer; empty for no code.
pub fn text(v: Option<(u32, u32)>) -> String {
    v.map_or(String::new(), |(a, b)| format!("{a}.{b}"))
}

/// An offer's API text back: `Some(None)` for no code, `None` if it doesn't parse.
pub fn parse(text: &str) -> Option<Option<(u32, u32)>> {
    if text.is_empty() {
        return Some(None);
    }
    let (a, b) = text.split_once('.')?;
    if a.len() > 4 || b.len() > 4 {
        return None;
    }
    Some(Some((a.parse().ok()?, b.parse().ok()?)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_by_name() {
        assert_eq!(api_version("chiacraft:mods/world@1.1.0"), Ok((1, 1)));
        assert_eq!(api_version("chiacraft:mods/log@1.0.2"), Ok((1, 0)));
        assert!(api_version("wasi:filesystem/types@0.2.0").is_err());
        assert!(api_version("chiacraft:mods/world").is_err());
        assert!(runs((1, 0)) && runs(API) && !runs((1, API.1 + 1)) && !runs((2, 0)));
        assert_eq!(parse(&text(Some((1, 1)))), Some(Some((1, 1))));
        assert_eq!(parse(""), Some(None));
        assert_eq!(parse("1"), None);
        assert_eq!(parse("99999.1"), None);
    }

    #[test]
    fn not_components() {
        assert!(api_of(b"").is_err());
        assert!(api_of(b"\0asm\x01\0\0\0").is_err());
        assert!(api_of(b"garbage").is_err());
    }

    #[test]
    fn reads_the_clans_component() {
        let wasm = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/packages/clans/code/server.wasm")).unwrap();
        assert_eq!(api_of(&wasm), Ok(Some((1, 0))));
    }
}
