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
    assert_eq!(progression::checks().len(), 17);
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

#[test]
fn gem_ceiling() {
    check("grade and gems at most +60% over a plain item");
}

#[test]
fn stagger() {
    check("stagger: same-age fights break every part, knockdowns at most a third of a fight");
}

#[test]
fn ranged() {
    check("bows on the curve: full draw a shot a second, a reference bow's arrows the reference's damage");
}

#[test]
fn saddles() {
    check("saddles lift tames by their fitting, as the tame model does, tooltip in step");
}

#[test]
fn loot_per_hour() {
    check(
        "loot per hour: kill drops priced from their cheapest kill, no kill paying over its time (+30 %, +50 % more for what fights back)",
    );
}

#[test]
fn path_parity() {
    check("path parity: each age's key by each route within ×1.15 of its cheapest, in EM");
}
