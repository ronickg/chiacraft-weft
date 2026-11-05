//! The progression and balance checks (`progression::checks`): each fails with what it found.

fn check(name: &str) {
    let all = progression::checks();
    let (_, problems) = all
        .iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("no check named {name}"));
    assert!(problems.is_empty(), "{name}:\n{}", problems.join("\n"));
}

#[test]
fn every_check_runs() {
    assert_eq!(progression::checks().len(), 11);
}

#[test]
fn tagging() {
    check("every item and block tagged, every made one with a recipe");
}

#[test]
fn recipes_declared() {
    check("every recipe in the mods' sources declared");
}

#[test]
fn reachable() {
    check("everything reachable from the start");
}

#[test]
fn tier_order() {
    check("nothing at tier N needs anything above N");
}

#[test]
fn age_keys() {
    check("each age's key by each declared route");
}

#[test]
fn materials() {
    check("every tool-part material tagged, made from its tier or below");
}

#[test]
fn making_budget() {
    check("each age's making inside its budget");
}

#[test]
fn sizes_and_roles() {
    check("every size and role inside its bands, floors hold");
}

#[test]
fn species() {
    check("every species inside its bands at every level, floors hold");
}

#[test]
fn tames() {
    check("tames capped by their saddle and ceiling");
}

#[test]
fn tweaks() {
    check("tweaks inside ×0.75-×1.25 with a reason");
}
