//! Auto-split from chart.rs — do not edit section headers.

use crate::units::JulianDay;

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 7 — Mesoamerican calendars (Aztec/Nahuatl + Mayan)
// ═══════════════════════════════════════════════════════════════════════════════

/// GMT correlation constant (Goodman–Martínez–Thompson).
/// Converts Julian Day to the Maya/Aztec Long Count.
/// JD 584283 = Maya Day 0 (correlation adopted by most scholars).
pub const GMT_CORRELATION: i64 = 584_283;

/// The 20 Nahuatl day signs (Tonalpohualli) in traditional order.
pub const TONALPOHUALLI_SIGNS: &[(&str, &str)] = &[
    ("Cipactli", "Crocodile"),
    ("Ehecatl", "Wind"),
    ("Calli", "House"),
    ("Cuetzpallin", "Lizard"),
    ("Coatl", "Serpent"),
    ("Miquiztli", "Death"),
    ("Mazatl", "Deer"),
    ("Tochtli", "Rabbit"),
    ("Atl", "Water"),
    ("Itzcuintli", "Dog"),
    ("Ozomatli", "Monkey"),
    ("Malinalli", "Grass"),
    ("Acatl", "Reed"),
    ("Ocelotl", "Ocelot"),
    ("Cuauhtli", "Eagle"),
    ("Cozcacuauhtli", "Vulture"),
    ("Ollin", "Earthquake"),
    ("Tecpatl", "Flint"),
    ("Quiahuitl", "Rain"),
    ("Xochitl", "Flower"),
];

/// The 20 Maya day signs (Tzolkin) in traditional order.
pub const TZOLKIN_SIGNS: &[(&str, &str)] = &[
    ("Imix", "Water Lily"),
    ("Ik", "Wind"),
    ("Akbal", "Night"),
    ("Kan", "Corn"),
    ("Chicchan", "Serpent"),
    ("Cimi", "Death"),
    ("Manik", "Deer"),
    ("Lamat", "Star"),
    ("Muluc", "Rain"),
    ("Oc", "Dog"),
    ("Chuen", "Monkey"),
    ("Eb", "Road"),
    ("Ben", "Corn Stalk"),
    ("Ix", "Jaguar"),
    ("Men", "Eagle"),
    ("Cib", "Vulture"),
    ("Caban", "Earth"),
    ("Etznab", "Flint"),
    ("Cauac", "Storm"),
    ("Ahau", "Sun"),
];

/// The 18 Aztec months of the Xiuhpohualli (365-day solar year).
pub const XIUHPOHUALLI_MONTHS: &[(&str, &str)] = &[
    ("Izcalli", "Sprouting"),
    ("Atlcahualo", "Ceasing of Water"),
    ("Tlacaxipehualiztli", "Flaying of Men"),
    ("Tozoztontli", "Small Vigil"),
    ("Huei Tozoztli", "Great Vigil"),
    ("Toxcatl", "Drought"),
    ("Etzalqualiztli", "Eating of Maize"),
    ("Tecuilhuitontli", "Small Feast of Lords"),
    ("Huei Tecuilhuitl", "Great Feast of Lords"),
    ("Tlaxochimaco", "Offering of Flowers"),
    ("Xocotl Huetzi", "Falling Fruit"),
    ("Ochpaniztli", "Sweeping"),
    ("Teotleco", "Return of the Gods"),
    ("Tepeilhuitl", "Mountain Feast"),
    ("Quecholli", "Flamingo"),
    ("Panquetzaliztli", "Raising of Banners"),
    ("Atemoztli", "Falling Water"),
    ("Tititl", "Stretching"),
];

/// Aztec Tonalpohualli position for a given Julian Day.
///
/// Returns `(trecena_number, day_sign_index, day_sign_name, day_sign_english)`.
/// The trecena (13-day week) number is 1–13; the day sign index is 0–19.
///
/// Uses the GMT correlation. The Aztec calendar system is in continuous
/// synchrony with the Maya Tzolkin.
#[must_use]
pub fn tonalpohualli(jd: JulianDay) -> (u8, usize, &'static str, &'static str) {
    let jd: f64 = jd.into();
    // Aztec day number from the base correlation. JD 584283 (Maya
    // Long Count epoch) is the canonical Maya "4 Ahau" / Aztec "4 Xochitl"
    // anchor: trecena 4, sign index 19. The +3 / +19 offsets shift
    // a raw 0-based day_num onto that anchor.
    let day_num = (jd as i64 - GMT_CORRELATION).rem_euclid(260) as usize;
    let trecena = ((day_num + 3) % 13 + 1) as u8; // 1-13, anchored at 4
    let sign_idx = (day_num + 19) % 20; // 0-19, anchored at 19 (Ahau)
    (
        trecena,
        sign_idx,
        TONALPOHUALLI_SIGNS[sign_idx].0,
        TONALPOHUALLI_SIGNS[sign_idx].1,
    )
}

/// Aztec Xiuhpohualli (365-day solar year) position for a Julian Day.
///
/// Returns `(month_index, day_in_month, month_name, month_english)`.
/// Month 18 (index 18) is the 5-day "Nemontemi" (unlucky days).
#[must_use]
pub fn xiuhpohualli(jd: JulianDay) -> (usize, u8, &'static str, &'static str) {
    let jd: f64 = jd.into();
    let day_num = (jd as i64 - GMT_CORRELATION).rem_euclid(365) as usize;
    let month_idx = (day_num / 20).min(18);
    let day_in_month = (day_num % 20 + 1) as u8;
    if month_idx < 18 {
        (
            month_idx,
            day_in_month,
            XIUHPOHUALLI_MONTHS[month_idx].0,
            XIUHPOHUALLI_MONTHS[month_idx].1,
        )
    } else {
        // Nemontemi: 5 unlucky days after the 18 months
        (18, (day_num - 360 + 1) as u8, "Nemontemi", "Unlucky Days")
    }
}

/// Maya Tzolkin (260-day sacred calendar) position for a Julian Day.
///
/// Returns `(trecena_number, day_sign_index, day_sign_name, day_sign_english)`.
#[must_use]
pub fn tzolkin(jd: JulianDay) -> (u8, usize, &'static str, &'static str) {
    let jd: f64 = jd.into();
    // Per Maya GMT correlation, JD 584283 = 4 Ahau (trecena 4,
    // sign 19). The +3 / +19 offsets shift the raw 0-based day_num
    // onto that canonical anchor.
    let day_num = (jd as i64 - GMT_CORRELATION).rem_euclid(260) as usize;
    let trecena = ((day_num + 3) % 13 + 1) as u8;
    let sign_idx = (day_num + 19) % 20;
    (
        trecena,
        sign_idx,
        TZOLKIN_SIGNS[sign_idx].0,
        TZOLKIN_SIGNS[sign_idx].1,
    )
}

/// Maya Haab (365-day vague year) position for a Julian Day.
///
/// Returns `(month_index, day_in_month, month_name)`.
/// Months 0–17 each have 20 days; month 18 (Wayeb) has 5.
#[must_use]
pub fn haab(jd: JulianDay) -> (usize, u8, &'static str) {
    let jd: f64 = jd.into();
    const HAAB_MONTHS: &[&str] = &[
        "Pop", "Wo", "Sip", "Sotz", "Sek", "Xul", "Yaxkin", "Mol", "Ch'en", "Yax", "Sak", "Keh",
        "Mak", "Kankin", "Muwan", "Pax", "Kayab", "Kumku", "Wayeb",
    ];
    // Per Maya GMT correlation, JD 584283 = "8 Kumku" — i.e. day 8 in
    // the 18th Haab month (Kumku, index 17). Cumulative position in
    // the 365-day Haab cycle: 17·20 + 8 = 348.
    let day_num = (jd as i64 - GMT_CORRELATION + 348).rem_euclid(365) as usize;
    let month_idx = (day_num / 20).min(18);
    let day = (day_num % 20) as u8;
    let name = HAAB_MONTHS[month_idx];
    (month_idx, day, name)
}

/// Maya Calendar Round: the 52-year cycle combining Tzolkin + Haab.
/// Returns `(tzolkin_trecena, tzolkin_sign, haab_day, haab_month)`.
#[must_use]
pub fn calendar_round(jd: JulianDay) -> (u8, &'static str, u8, &'static str) {
    let jd: f64 = jd.into();
    let (trecena, _, sign_name, _) = tzolkin(JulianDay::new(jd));
    let (_, haab_day, haab_month) = haab(JulianDay::new(jd));
    (trecena, sign_name, haab_day, haab_month)
}

// ═══════════════════════════════════════════════════════════════════════════
// Maya Long Count
// ═══════════════════════════════════════════════════════════════════════════

/// Convert a Julian Day to the Maya Long Count: `(baktun, katun, tun, uinal, kin)`.
///
/// Position values (Maya positional base-20, except tun = 18 uinal):
/// - 1 kin    = 1 day
/// - 1 uinal  = 20 kin
/// - 1 tun    = 18 uinal   (≈ 1 year, = 360 days)
/// - 1 katun  = 20 tun     (≈ 19.7 years)
/// - 1 baktun = 20 katun   (≈ 394 years)
///
/// Uses the GMT correlation (JD 584 283 = Maya Day 0 = 0.0.0.0.0 4 Ajaw 8 Kumk'u).
#[must_use]
pub fn maya_long_count(jd: JulianDay) -> (u32, u32, u32, u32, u32) {
    let jd: f64 = jd.into();
    let mut days = (jd.floor() as i64 - GMT_CORRELATION).max(0);
    let kin = (days % 20) as u32;
    days /= 20;
    let uinal = (days % 18) as u32;
    days /= 18;
    let tun = (days % 20) as u32;
    days /= 20;
    let katun = (days % 20) as u32;
    days /= 20;
    let baktun = days as u32;
    (baktun, katun, tun, uinal, kin)
}

/// Long Count in canonical dotted notation, e.g. `"13.0.0.0.0"`.
#[must_use]
pub fn maya_long_count_str(jd: JulianDay) -> String {
    let jd: f64 = jd.into();
    let (b, k, t, u, ki) = maya_long_count(JulianDay::new(jd));
    format!("{b}.{k}.{t}.{u}.{ki}")
}

#[cfg(test)]
mod long_count_tests {
    use super::*;

    const EPOCH: f64 = GMT_CORRELATION as f64;

    fn epoch_day(offset: i64) -> JulianDay {
        JulianDay::new(EPOCH + offset as f64)
    }

    #[test]
    fn tonalpohualli_anchor_and_rollover() {
        assert_eq!(tonalpohualli(epoch_day(0)), (4, 19, "Xochitl", "Flower"));
        assert_eq!(tonalpohualli(epoch_day(1)), (5, 0, "Cipactli", "Crocodile"));
    }

    #[test]
    fn xiuhpohualli_month_and_nemontemi_boundaries() {
        assert_eq!(xiuhpohualli(epoch_day(0)), (0, 1, "Izcalli", "Sprouting"));
        assert_eq!(
            xiuhpohualli(epoch_day(20)),
            (1, 1, "Atlcahualo", "Ceasing of Water")
        );
        assert_eq!(
            xiuhpohualli(epoch_day(360)),
            (18, 1, "Nemontemi", "Unlucky Days")
        );
    }

    #[test]
    fn tzolkin_anchor_and_rollover() {
        assert_eq!(tzolkin(epoch_day(0)), (4, 19, "Ahau", "Sun"));
        assert_eq!(tzolkin(epoch_day(1)), (5, 0, "Imix", "Water Lily"));
    }

    #[test]
    fn haab_anchor_wayeb_and_cycle_boundaries() {
        assert_eq!(haab(epoch_day(0)), (17, 8, "Kumku"));
        assert_eq!(haab(epoch_day(12)), (18, 0, "Wayeb"));
        assert_eq!(haab(epoch_day(17)), (0, 0, "Pop"));
    }

    #[test]
    fn calendar_round_combines_exact_components() {
        assert_eq!(calendar_round(epoch_day(0)), (4, "Ahau", 8, "Kumku"));
    }

    #[test]
    fn long_count_place_boundaries() {
        let cases = [
            (19, (0, 0, 0, 0, 19)),
            (20, (0, 0, 0, 1, 0)),
            (359, (0, 0, 0, 17, 19)),
            (360, (0, 0, 1, 0, 0)),
            (7_199, (0, 0, 19, 17, 19)),
            (7_200, (0, 1, 0, 0, 0)),
            (143_999, (0, 19, 19, 17, 19)),
            (144_000, (1, 0, 0, 0, 0)),
        ];

        for (offset, expected) in cases {
            assert_eq!(maya_long_count(epoch_day(offset)), expected);
        }
    }

    #[test]
    fn long_count_clamps_before_epoch() {
        assert_eq!(maya_long_count(epoch_day(-1)), (0, 0, 0, 0, 0));
    }

    #[test]
    fn long_count_2012_bak13() {
        // 2012-12-21 = 13.0.0.0.0 (end of 13th baktun, popular "Mayan prophecy")
        // JD 2456283.0 (2012-12-21 12:00 UT)
        let (b, k, t, u, ki) = maya_long_count(JulianDay::new(2_456_283.0));
        assert_eq!((b, k, t, u, ki), (13, 0, 0, 0, 0));
    }

    #[test]
    fn long_count_j2000() {
        // J2000 = 2000-01-01 12:00 UT → known: 12.19.6.15.2
        let (b, k, t, u, ki) = maya_long_count(JulianDay::new(2_451_545.0));
        assert_eq!((b, k, t, u, ki), (12, 19, 6, 15, 2));
    }

    #[test]
    fn long_count_str_format() {
        assert_eq!(
            maya_long_count_str(JulianDay::new(2_456_283.0)),
            "13.0.0.0.0"
        );
    }

    #[test]
    fn long_count_epoch() {
        // JD 584283 = 0.0.0.0.0 (start of current creation cycle)
        let (b, k, t, u, ki) = maya_long_count(JulianDay::new(584_283.0));
        assert_eq!((b, k, t, u, ki), (0, 0, 0, 0, 0));
    }
}
