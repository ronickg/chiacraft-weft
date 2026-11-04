//! Whether the numbers hold together (`notes/SURVIVAL.md`), as the animals' `balance.rs` does for
//! species: every food fits a new player's stomach and digests in a sensible time, no single food
//! makes one well fed but a varied meal does, the level curve is as long as intended, and the
//! temperatures feel as they should. `problems()` lists what's out of line; the tests ask for
//! none, so a new food or a changed number that throws it off is caught.

use crate::effects::{self, Effect};
use crate::food::{self, FOODS, Freshness, Meal};
use crate::levels;
use crate::stamina;
use crate::stats;
use crate::stomach::{self, Stomach};
use crate::temperature::{self, Body, Feeling, Place};

pub fn problems() -> Vec<String> {
    let mut out = Vec::new();
    let capacity = stats::PLAYER_BASE[stats::Stat::Food.index()];

    for f in FOODS {
        let p = f.per100;
        if p.water + p.protein + p.carbs + p.fat + p.fiber > 101.0 {
            out.push(format!("{}: more than 100 g in 100 g", f.name));
        }
        if f.units() > capacity {
            out.push(format!("{}: {} units, more than a new player's stomach ({capacity})", f.name, f.units()));
        }
        let minutes = f.digest_minutes();
        if !(2.0..=60.0).contains(&minutes) {
            out.push(format!("{}: digests in {minutes:.0} minutes (2 to 60 wanted)", f.name));
        }
        // One food alone, as much as fits, must not make one well fed: variety does.
        let mut s = Stomach::default();
        let i = food::find(f.name).unwrap_or(0);
        while s.eat(i, 1.0, food::HOT_FOR, capacity).is_ok() {}
        if s.well_fed(capacity) {
            out.push(format!("{}: alone makes one well fed", f.name));
        }
    }

    if best_meal(capacity).is_none() {
        out.push("no meal of cooked and fresh food makes a new player well fed".into());
    }

    if levels::to_next(1) != 20 {
        out.push(format!("level 1 to 2 asks {} XP (20 wanted)", levels::to_next(1)));
    }
    let all = levels::total_to(levels::MAX_LEVEL);
    if !(150_000..=400_000).contains(&all) {
        out.push(format!("{all} XP to level {} (150 000 to 400 000 wanted)", levels::MAX_LEVEL));
    }

    // A new player, well fed, sprints ten to twenty-five seconds.
    let sprint = 110.0 / stamina::SPRINT;
    if !(10.0..=25.0).contains(&sprint) {
        out.push(format!("a new player sprints {sprint:.0} s (10 to 25 wanted)"));
    }

    let person = Body { comfort: temperature::PERSON, ..Body::default() };
    let noon = |biome_c: f32| Place {
        biome_c,
        humidity: 0.5,
        altitude: 0.0,
        hour: 13.0,
        open_sky: true,
        raining: false,
        snowing: false,
        in_water: false,
        fire: 0.0,
        season: 0.0,
        underground: false,
        wind: 0.0,
    };
    let feels = |b: &Body, p: &Place| temperature::feeling(b.target(temperature::air(p)));
    if feels(&person, &noon(18.0)) != Feeling::Comfortable {
        out.push("unclothed at 18 °C is not comfortable".into());
    }
    if feels(&person, &noon(-10.0)) != Feeling::Freezing {
        out.push("unclothed at -10 °C doesn't freeze".into());
    }
    let leather = Body { cold_insulation: temperature::set_insulation("leather").0, ..person };
    let by_fire = Place { fire: temperature::fire_heat(3.0), hour: 3.0, ..noon(0.0) };
    if matches!(feels(&leather, &by_fire), Feeling::Cold | Feeling::Freezing) {
        out.push("in leather by a fire at 0 °C at night is still cold".into());
    }
    if !matches!(feels(&person, &Place { humidity: 0.1, ..noon(38.0) }), Feeling::Hot | Feeling::Heatstroke) {
        out.push("noon in a 38 °C desert isn't hot".into());
    }
    let wet = Body { wet: true, ..leather };
    if feels(&wet, &Place { hour: 20.0, ..noon(8.0) }) == Feeling::Comfortable {
        out.push("wet in leather on an 8 °C evening feels comfortable".into());
    }

    // Freezing takes a full new player in two to five minutes at normal difficulty.
    let freeze = 1.0 / Effect::Freezing.modifiers().health_loss;
    if !(120.0..=300.0).contains(&freeze) {
        out.push(format!("freezing kills in {freeze:.0} s (120 to 300 wanted)"));
    }
    if effects::combine([Effect::Rested, Effect::WellFed]).stamina_regen <= 1.0 {
        out.push("rested and well fed don't raise stamina regeneration".into());
    }
    out
}

/// The smallest meal (up to six portions of fresh, cooked or safe foods) that makes a new player
/// well fed, as food names; `None` if there is none.
pub fn best_meal(capacity: f32) -> Option<Vec<&'static str>> {
    let safe: Vec<usize> = (0..FOODS.len())
        .filter(|&i| !FOODS[i].raw && FOODS[i].poison == 0.0 && FOODS[i].meal != Meal::Plain)
        .collect();
    let mut best: Option<Vec<usize>> = None;
    let mut pick = Vec::new();
    search(&safe, 0, &mut pick, capacity, &mut best);
    best.map(|b| b.iter().map(|&i| FOODS[i].name).collect())
}

fn search(safe: &[usize], from: usize, pick: &mut Vec<usize>, capacity: f32, best: &mut Option<Vec<usize>>) {
    let mut s = Stomach::default();
    let fits = pick
        .iter()
        .all(|&i| s.eat(i, Freshness::Fresh.quality(), 0.0, capacity).is_ok());
    if !fits {
        return;
    }
    if !pick.is_empty() && s.well_fed(capacity) {
        if best.as_ref().is_none_or(|b| pick.len() < b.len()) {
            *best = Some(pick.clone());
        }
        return;
    }
    if pick.len() >= 6 {
        return;
    }
    for k in from..safe.len() {
        pick.push(safe[k]);
        search(safe, k, pick, capacity, best);
        pick.pop();
    }
}

/// Max health and stamina of a new player after `meal` (food names), for the notes and checks.
pub fn after(meal: &[&str]) -> (f32, f32) {
    let capacity = stats::PLAYER_BASE[stats::Stat::Food.index()];
    let mut s = Stomach::default();
    for name in meal {
        if let Some(i) = food::find(name) {
            let _ = s.eat(i, 1.0, food::HOT_FOR, capacity);
        }
    }
    stomach::max_health_stamina(stats::PLAYER_BASE[0], stats::PLAYER_BASE[1], &s, capacity)
}

#[cfg(test)]
mod tests {
    #[test]
    fn in_balance() {
        let problems = super::problems();
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }

    #[test]
    fn levels_round_trip() {
        for xp in [0, 19, 20, 1000, 50_000, 10_000_000] {
            let (level, into) = crate::levels::level_of(xp);
            assert_eq!(crate::levels::total_to(level) + into, xp);
        }
        assert_eq!(crate::levels::level_of(20).0, 2);
        assert_eq!(crate::levels::after_death(crate::levels::total_to(5) + 100, 0.25), crate::levels::total_to(5) + 75);
    }

    #[test]
    fn the_calendar_turns() {
        use crate::calendar::{self, Season};
        assert_eq!(calendar::date(0).season, Season::Spring);
        assert_eq!(calendar::date(0).day_of_season, 1);
        let summer = calendar::date(14 * calendar::DAY_TICKS);
        assert_eq!((summer.season, summer.day_of_season), (Season::Summer, 1));
        let next = calendar::date(56 * calendar::DAY_TICKS + 3000);
        assert_eq!((next.year, next.season), (1, Season::Spring));
        // Midsummer is the warmest, midwinter the coldest.
        let at = |day: f32| calendar::date((day * calendar::DAY_TICKS as f32) as u64).from_midwinter();
        assert!((at(calendar::MIDSUMMER) - 0.5).abs() < 0.01);
        assert!(crate::temperature::season(10.0, at(calendar::MIDSUMMER)) > 9.9);
        assert_eq!(calendar::date(calendar::on_day(3 * calendar::DAY_TICKS + 500, 42)).season, Season::Winter);
    }

    #[test]
    fn a_stomach_digests_and_fades() {
        let apple = crate::food::find("apple").unwrap();
        let mut s = crate::stomach::Stomach::default();
        s.eat(apple, 1.0, 0.0, 100.0).unwrap();
        let before = s.fullness(100.0)[crate::food::WATER];
        s.digest(60.0, 1.0);
        assert!(s.fullness(100.0)[crate::food::WATER] < before);
        s.digest(3600.0, 1.0);
        assert_eq!(s.used(), 0.0);
    }
}
