//! Tibetan calendar (Phugpa system).
//!
//! Provides:
//! - **Losar** (New Year): the second new moon after the preceding winter
//!   solstice, evaluated in Lhasa local time (UTC+6).
//! - **Rabjung cycle year name**: animal × element × yin/yang from the
//!   Rabjung 60-year cycle (first Rabjung began AD 1027).
//!
//! Full tithi- / month-number resolution (Kalachakra correction tables) is
//! **not** provided; consult published Tibetan almanacs for liturgical use.

use crate::body::CalcFlags;
use crate::functions::moon_phases::next_new_moon;
use crate::functions::motion::solcross_ut;
use crate::functions::time::julday;

/// Lhasa / Tibetan civil timezone offset from UTC (hours).
pub const LHASA_TZ_OFFSET_HOURS: f64 = 6.0;

/// Julian Day of Tibetan Losar (New Year) for a given Gregorian year.
///
/// Losar is defined as the civil day (Lhasa UTC+6) of the second new moon
/// after the preceding winter solstice. Returns `None` if the underlying
/// astronomical searches fail.
pub fn losar_jd(gregorian_year: i32) -> Option<f64> {
    // Winter solstice of the *previous* year (Sun at 270° ecliptic lon)
    let jd_dec = julday(
        gregorian_year - 1,
        12,
        21,
        0.0,
        crate::body::Calendar::Gregorian,
    );
    let sols = solcross_ut(270.0, jd_dec - 5.0, CalcFlags::BUILTIN).ok()?;

    // First new moon after the solstice
    let nm1 = next_new_moon(sols).ok()?;
    // Second new moon (search from ≥2 days after the first)
    let nm2 = next_new_moon(nm1 + 2.0).ok()?;

    // Civil day in Lhasa timezone
    let local = nm2 + LHASA_TZ_OFFSET_HOURS / 24.0;
    Some(local.floor() + 0.5 - LHASA_TZ_OFFSET_HOURS / 24.0)
}

/// Tibetan year attributes: `(rabjung_cycle, year_in_cycle, element, gender, animal)`.
///
/// The Rabjung 60-year cycle combines 12 animals × 5 elements × 2 genders
/// (yin/yang, tied to element parity). The first Rabjung cycle began in
/// AD 1027.
pub fn tibetan_year_name(
    gregorian_year: i32,
) -> (u32, u32, &'static str, &'static str, &'static str) {
    const RABJUNG_EPOCH: i32 = 1027;
    const ANIMALS: [&str; 12] = [
        "Mouse", "Ox", "Tiger", "Rabbit", "Dragon", "Snake", "Horse", "Sheep", "Monkey", "Bird",
        "Dog", "Pig",
    ];
    const ELEMENTS: [&str; 5] = ["Wood", "Fire", "Earth", "Iron", "Water"];
    const GENDERS: [&str; 2] = ["Male", "Female"];

    // Rabjung cycle number uses 1027 epoch; element/animal/gender mapping
    // uses the Chinese-aligned 1984 anchor (Wood Mouse = year 1 of 60-cycle).
    // Compute in i64 to safely handle years before the epoch (negative offset).
    let offset_cycle = (gregorian_year as i64) - (RABJUNG_EPOCH as i64);
    let offset_anim = (gregorian_year - 1984).rem_euclid(60) as usize;

    // Clamp to non-negative for the display cycle number. Years before 1027
    // yield cycle = 0 ("pre-Rabjung"); callers should interpret that accordingly.
    let cycle_i64 = offset_cycle.div_euclid(60) + 1;
    let cycle = cycle_i64.max(0) as u32;
    let year_in_cycle = offset_cycle.rem_euclid(60) as u32 + 1;

    let animal = ANIMALS[offset_anim % 12];
    let element = ELEMENTS[(offset_anim / 2) % 5];
    let gender = GENDERS[offset_anim % 2];

    (cycle, year_in_cycle, element, gender, animal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rabjung_2024_wood_dragon_male() {
        let (cycle, yic, element, gender, animal) = tibetan_year_name(2024);
        assert_eq!(cycle, 17);
        assert_eq!(yic, 38);
        assert_eq!(element, "Wood");
        assert_eq!(gender, "Male");
        assert_eq!(animal, "Dragon");
    }

    #[test]
    fn rabjung_1027_first_cycle() {
        let (c, yic, _, _, _) = tibetan_year_name(1027);
        assert_eq!(c, 1);
        assert_eq!(yic, 1);
    }

    #[test]
    fn rabjung_cycle_rollover() {
        let (c1, _, _, _, _) = tibetan_year_name(1086); // end of cycle 1
        let (c2, _, _, _, _) = tibetan_year_name(1087); // start of cycle 2
        assert_eq!(c1, 1);
        assert_eq!(c2, 2);
    }

    #[test]
    fn losar_tz_constant() {
        assert!((LHASA_TZ_OFFSET_HOURS - 6.0).abs() < 1e-9);
    }
}
