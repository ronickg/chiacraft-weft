//! The ten grades of a part (`notes/CRAFTING.md`, "The ten grades"): what a part's material, the
//! work and the station add up to, with a little luck, and what each grade multiplies.

/// One grade: its name's key, what it multiplies the part's main stat and durability by, and the
/// modifier slots it gives an item whose working part has it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grade {
    pub n: u8,
    pub key: &'static str,
    pub mult: f32,
    pub slots: u8,
}

/// Crude to Legendary. Five colour bands of two, the second of each pair marked (UI).
pub const GRADES: [Grade; 10] = [
    Grade { n: 1, key: "grade.base.crude", mult: 0.70, slots: 0 },
    Grade { n: 2, key: "grade.base.rough", mult: 0.80, slots: 0 },
    Grade { n: 3, key: "grade.base.plain", mult: 0.90, slots: 0 },
    Grade { n: 4, key: "grade.base.sound", mult: 1.00, slots: 1 },
    Grade { n: 5, key: "grade.base.good", mult: 1.06, slots: 1 },
    Grade { n: 6, key: "grade.base.fine", mult: 1.12, slots: 1 },
    Grade { n: 7, key: "grade.base.superior", mult: 1.20, slots: 2 },
    Grade { n: 8, key: "grade.base.exquisite", mult: 1.28, slots: 2 },
    Grade { n: 9, key: "grade.base.masterwork", mult: 1.38, slots: 3 },
    Grade { n: 10, key: "grade.base.legendary", mult: 1.50, slots: 3 },
];

/// Grade `n` (1-10, clamped).
pub fn grade(n: u8) -> &'static Grade {
    &GRADES[(n.clamp(1, 10) - 1) as usize]
}

/// Most points from each share.
pub const MATERIAL_MAX: u32 = 40;
pub const WORK_MAX: u32 = 40;
pub const STATION_MAX: u32 = 20;
/// The most the work scores from the book (from memory): only hands-on work goes higher.
pub const MEMORY_MAX: u32 = 30;
/// What a machine's work scores (automation later): about Good with good materials.
pub const MACHINE_WORK: u32 = 22;
/// Points grade 10 needs, past what work from memory can reach (40 + 30 + 20): only hands-on work
/// makes a Legendary part, though luck can still lift a 92-point part from 9.
pub const LEGENDARY_POINTS: u32 = 92;

/// What a part was made with: its material's share, the work's and the station's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Points {
    pub material: u32,
    pub work: u32,
    pub station: u32,
}

impl Points {
    pub fn total(&self) -> u32 {
        self.material.min(MATERIAL_MAX) + self.work.min(WORK_MAX) + self.station.min(STATION_MAX)
    }
}

/// The work's share from the book, by the maker's skill in that craft (0-10): 10 points and 2 a
/// level, never past `MEMORY_MAX`.
pub fn work_from_memory(skill: u32) -> u32 {
    (10 + 2 * skill).min(MEMORY_MAX)
}

/// Luck's steps and their chances, in order: one down, none, one up, two up.
pub const LUCK: [(i8, f32); 4] = [(-1, 0.10), (0, 0.70), (1, 0.17), (2, 0.03)];

/// Luck's step for a roll `u` in 0..1.
pub fn luck(u: f32) -> i8 {
    let mut acc = 0.0;
    for (step, chance) in LUCK {
        acc += chance;
        if u < acc {
            return step;
        }
    }
    LUCK[LUCK.len() - 1].0
}

/// The grade `points` make with roll `u` (0..1): the points in tens, then luck. Grade 10 only
/// from `LEGENDARY_POINTS`.
pub fn roll(points: Points, u: f32) -> u8 {
    let total = points.total();
    let base = (total / 10).clamp(1, 10) as i8;
    let g = (base + luck(u)).clamp(1, 10) as u8;
    if g == 10 && total < LEGENDARY_POINTS { 9 } else { g }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grades_rise() {
        for w in GRADES.windows(2) {
            assert!(w[1].mult > w[0].mult && w[1].slots >= w[0].slots, "{w:?}");
            assert_eq!(w[1].n, w[0].n + 1);
        }
        assert_eq!(grade(4).mult, 1.0, "Sound is the plain number");
    }

    #[test]
    fn luck_adds_up() {
        let sum: f32 = LUCK.iter().map(|l| l.1).sum();
        assert!((sum - 1.0).abs() < 1e-6);
        assert_eq!(luck(0.05), -1);
        assert_eq!(luck(0.5), 0);
        assert_eq!(luck(0.9), 1);
        assert_eq!(luck(0.99), 2);
    }

    #[test]
    fn points_in_tens() {
        let p = Points { material: 30, work: 30, station: 12 };
        assert_eq!(p.total(), 72);
        assert_eq!(roll(p, 0.5), 7, "the mock-up's Superior");
        assert_eq!(roll(Points::default(), 0.0), 1, "never below Crude");
    }

    #[test]
    fn legendary_needs_hands() {
        // From memory at the best station with the best material: 90, and luck stops at 9.
        let memory = Points { material: 40, work: work_from_memory(10), station: 20 };
        assert_eq!(memory.total(), 90);
        assert_eq!(roll(memory, 0.999), 9);
        // A machine never reaches it, whatever luck does.
        let machine = Points { material: 40, work: MACHINE_WORK, station: 20 };
        assert!(roll(machine, 0.999) < 10);
        // Hands-on work past 32 can roll up to it.
        assert_eq!(roll(Points { material: 40, work: 38, station: 18 }, 0.999), 10);
    }
}
