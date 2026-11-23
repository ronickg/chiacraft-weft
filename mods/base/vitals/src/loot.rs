//! Loot per hour (`reports/Balance model.md`, 5.2 and 5.5): what a kill earns against the time it
//! takes, both in effort minutes (`curve::Em`). An effort minute is a minute of an average player's
//! play, so an hour's hunting is fairly paid when its loot is worth about an hour: a rate of 1 EM a
//! minute, within `BAND`, with up to `RISK_PREMIUM` more for what fights back. The checks
//! (`mods/base/progression`, `loot`) price each kill's drops at the progression graph's costs and
//! its time as finding one, fighting it at its home age and taking what it leaves.

/// Minutes to find a common animal where it lives (a herd on the next hill, a fox in the wood): a
/// rarer one takes as much longer as its spawn chance is smaller (`animals_core`'s
/// `Rarity::find_minutes`).
pub const COMMON_FIND_MINUTES: f32 = 1.5;

/// Minutes for a hostile mob to find the player at night (they come to you).
pub const NIGHT_FIND_MINUTES: f32 = 0.5;

/// Minutes to walk to what fell and take it, a kill.
pub const HANDLING_MINUTES: f32 = 0.25;

/// Minutes of work a hide or a fur takes before it's used (scraping, tanning or curing, drying), on
/// top of its share of the kill (Planner 11-23, for Ronald: option A made hides and fur about ten
/// times cheaper and the Stone Age's kit 18 % of its budget; this brings it back near the old
/// 30-40 %). 0 drops it.
pub const PELT_MINUTES: f32 = 2.0;

/// What `PELT_MINUTES` counts for.
pub const PELTS: &[&str] = &["base:hide", "base:fur"];

/// Handling minutes for one piece of `item` (0 for all but pelts).
pub fn handling(item: &str) -> f32 {
    if PELTS.contains(&item) { PELT_MINUTES } else { 0.0 }
}

/// How far a kill's rate (EM earned a minute) may sit from 1: ±30 %, the balance model's band for
/// an activity's value per hour.
pub const BAND: f32 = 0.3;

/// What fights back may pay this much more (the balance model's risk premium, up to +50 %): the
/// time lost when it downs the player.
pub const RISK_PREMIUM: f32 = 0.5;

/// What kills drop, each with the price it had by hand (effort minutes a piece, the hunt included),
/// now a weight: a kill's minutes are shared among its drops by count × weight, and each drop
/// costs what its cheapest kill makes it (Planner 11-23, option A: the hand prices made kills pay
/// 2-31× their time). Pricing these directly again is option B, kept here to come back to if the
/// game feels off. Boss loot and found things keep their hand prices (`progression::content`).
pub const KILL_DROPS: &[(&str, f32)] = &[
    ("animals:carcass", 4.0),
    ("animals:venison", 3.0),
    ("animals:hare", 3.0),
    ("animals:trout", 3.0),
    ("animals:pacu", 3.0),
    ("animals:claw", 10.0),
    ("animals:scute", 10.0),
    ("base:hide", 2.0),
    ("base:fur", 2.0),
    ("base:feather", 2.0),
    ("base:bone", 2.0),
    ("base:arrow", 2.0),
    ("base:gunpowder", 2.0),
    ("base:spider_eye", 2.0),
    ("base:rotten_flesh", 2.0),
];

/// A kill drop's weight (`KILL_DROPS`).
pub fn weight(item: &str) -> Option<f32> {
    KILL_DROPS.iter().find(|d| d.0 == item).map(|d| d.1)
}

/// What each of a kill's `drops` (item, count) costs a piece, from its `minutes`: the minutes
/// shared by count × weight (an item without a weight counts 1).
pub fn shares(minutes: f32, drops: &[(String, f32)]) -> impl Iterator<Item = (&str, f32)> {
    let w = |id: &str| weight(id).unwrap_or(1.0);
    let total: f32 = drops.iter().map(|(id, n)| n * w(id)).sum();
    drops
        .iter()
        .map(move |(id, _)| (id.as_str(), minutes * w(id) / total.max(1e-6)))
}

/// The band of a kill's rate, by whether it fights back.
pub fn band(fights_back: bool) -> (f32, f32) {
    let top = 1.0 + BAND;
    (1.0 - BAND, if fights_back { top * (1.0 + RISK_PREMIUM) } else { top })
}

/// Minutes a kill takes: `find` to find a group, shared by the `kills` in it, the fight's seconds,
/// the handling.
pub fn kill_minutes(find: f32, kills: f32, fight_secs: f32) -> f32 {
    find / kills.max(1.0) + fight_secs / 60.0 + HANDLING_MINUTES
}
