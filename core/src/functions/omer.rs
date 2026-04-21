//! Sefirat HaOmer — Counting of the Omer.
//!
//! The Omer is a 49-day period in the Hebrew calendar that runs from
//! the second night of Passover (16 Nisan) through the eve of Shavuot
//! (5 Sivan evening, completing on 6 Sivan).
//!
//! Each day is associated with a pair of Kabbalistic Sefirot that form
//! the spiritual theme of that day. The 49 combinations are derived from
//! the 7 lower Sefirot: Chesed, Gevurah, Tiferet, Netzach, Hod, Yesod, Malkhut.
//!
//! # Examples
//!
//! ```
//! # use celestial_core::body::Calendar;
//! use celestial_core::{omer_from_jd, julday, GREG_CAL};
//!
//! // 16 Nisan 5785 begins at nightfall April 13, 2025
//! let jd = julday(2025, 4, 13, 20.0, Calendar::Gregorian);
//! if let Some(day) = omer_from_jd(jd) {
//!     assert_eq!(day.day, 1);
//!     assert_eq!(day.week, 1);
//!     assert_eq!(day.day_of_week, 1);
//!     assert_eq!(day.week_sefirah, "Chesed");
//!     assert_eq!(day.day_sefirah, "Chesed");
//! }
//! ```

// ─── Sefirot ──────────────────────────────────────────────────────────────────

/// The seven lower Sefirot used in the Omer count.
pub const SEFIROT: [&str; 7] = [
    "Chesed",  // 1 — loving-kindness
    "Gevurah", // 2 — strength / discipline
    "Tiferet", // 3 — beauty / harmony
    "Netzach", // 4 — eternity / victory
    "Hod",     // 5 — splendor / gratitude
    "Yesod",   // 6 — foundation
    "Malkhut", // 7 — sovereignty / presence
];

/// Hebrew names for Omer day numbers (1–49).
pub const OMER_DAY_NAMES: [&str; 49] = [
    "Yom echad la'Omer",                                                       // 1
    "Shnei yamim la'Omer",                                                     // 2
    "Shlosha yamim la'Omer",                                                   // 3
    "Arba'a yamim la'Omer",                                                    // 4
    "Chamisha yamim la'Omer",                                                  // 5
    "Shisha yamim la'Omer",                                                    // 6
    "Shiv'a yamim she'hem shavua echad la'Omer",                               // 7
    "Shmona yamim she'hem shavua echad v'yom echad la'Omer",                   // 8
    "Tish'a yamim she'hem shavua echad u'shnei yamim la'Omer",                 // 9
    "Asara yamim she'hem shavua echad u'shlosha yamim la'Omer",                // 10
    "Achad asar yom she'hem shavua echad v'arba'a yamim la'Omer",              // 11
    "Shnem asar yom she'hem shavua echad v'chamisha yamim la'Omer",            // 12
    "Shlosha asar yom she'hem shavua echad v'shisha yamim la'Omer",            // 13
    "Arba'a asar yom she'hem shnei shavuot la'Omer",                           // 14
    "Chamisha asar yom she'hem shnei shavuot v'yom echad la'Omer",             // 15
    "Shisha asar yom she'hem shnei shavuot u'shnei yamim la'Omer",             // 16
    "Shiv'a asar yom she'hem shnei shavuot u'shlosha yamim la'Omer",           // 17
    "Shmona asar yom she'hem shnei shavuot v'arba'a yamim la'Omer",            // 18
    "Tish'a asar yom she'hem shnei shavuot v'chamisha yamim la'Omer",          // 19
    "Esrim yom she'hem shnei shavuot v'shisha yamim la'Omer",                  // 20
    "Echad v'esrim yom she'hem shlosha shavuot la'Omer",                       // 21
    "Shnayim v'esrim yom she'hem shlosha shavuot v'yom echad la'Omer",         // 22
    "Shlosha v'esrim yom she'hem shlosha shavuot u'shnei yamim la'Omer",       // 23
    "Arba'a v'esrim yom she'hem shlosha shavuot u'shlosha yamim la'Omer",      // 24
    "Chamisha v'esrim yom she'hem shlosha shavuot v'arba'a yamim la'Omer",     // 25
    "Shisha v'esrim yom she'hem shlosha shavuot v'chamisha yamim la'Omer",     // 26
    "Shiv'a v'esrim yom she'hem shlosha shavuot v'shisha yamim la'Omer",       // 27
    "Shmona v'esrim yom she'hem arba'a shavuot la'Omer",                       // 28
    "Tish'a v'esrim yom she'hem arba'a shavuot v'yom echad la'Omer",           // 29
    "Shloshim yom she'hem arba'a shavuot u'shnei yamim la'Omer",               // 30
    "Echad u'shloshim yom she'hem arba'a shavuot u'shlosha yamim la'Omer",     // 31
    "Shnayim u'shloshim yom she'hem arba'a shavuot v'arba'a yamim la'Omer",    // 32
    "Shlosha u'shloshim yom she'hem arba'a shavuot v'chamisha yamim la'Omer",  // 33 (Lag Ba'Omer)
    "Arba'a u'shloshim yom she'hem arba'a shavuot v'shisha yamim la'Omer",     // 34
    "Chamisha u'shloshim yom she'hem chamisha shavuot la'Omer",                // 35
    "Shisha u'shloshim yom she'hem chamisha shavuot v'yom echad la'Omer",      // 36
    "Shiv'a u'shloshim yom she'hem chamisha shavuot u'shnei yamim la'Omer",    // 37
    "Shmonim u'shloshim yom she'hem chamisha shavuot u'shlosha yamim la'Omer", // 38
    "Tish'a u'shloshim yom she'hem chamisha shavuot v'arba'a yamim la'Omer",   // 39
    "Arba'im yom she'hem chamisha shavuot v'chamisha yamim la'Omer",           // 40
    "Echad v'arba'im yom she'hem chamisha shavuot v'shisha yamim la'Omer",     // 41
    "Shnayim v'arba'im yom she'hem shisha shavuot la'Omer",                    // 42
    "Shlosha v'arba'im yom she'hem shisha shavuot v'yom echad la'Omer",        // 43
    "Arba'a v'arba'im yom she'hem shisha shavuot u'shnei yamim la'Omer",       // 44
    "Chamisha v'arba'im yom she'hem shisha shavuot u'shlosha yamim la'Omer",   // 45
    "Shisha v'arba'im yom she'hem shisha shavuot v'arba'a yamim la'Omer",      // 46
    "Shiv'a v'arba'im yom she'hem shisha shavuot v'chamisha yamim la'Omer",    // 47
    "Shmonim v'arba'im yom she'hem shisha shavuot v'shisha yamim la'Omer",     // 48
    "Tish'a v'arba'im yom she'hem shiv'a shavuot la'Omer",                     // 49
];

// ─── Data types ───────────────────────────────────────────────────────────────

/// A single day of the Omer count with its full Kabbalistic annotation.
#[derive(Debug, Clone, PartialEq)]
pub struct OmerDay {
    /// Day number in the Omer count (1–49).
    pub day: u8,
    /// Week number (1–7).
    pub week: u8,
    /// Day within the current week (1–7).
    pub day_of_week: u8,
    /// Sefirah of the week (outer quality).
    pub week_sefirah: &'static str,
    /// Sefirah of the day (inner quality).
    pub day_sefirah: &'static str,
    /// Full Hebrew declaration (e.g. "Yom echad la'Omer").
    pub hebrew_text: &'static str,
    /// Whether this is Lag Ba'Omer (day 33).
    pub is_lag_baomer: bool,
    /// Julian day of the start of this Omer day (at nightfall, ~18:00 local time).
    pub jd: f64,
}

/// Result of the Omer period for a Hebrew year.
#[derive(Debug, Clone)]
pub struct OmerPeriod {
    /// Julian day of the first day of the Omer (16 Nisan, nightfall).
    pub start_jd: f64,
    /// Julian day of the last day of the Omer (49th day, 5 Sivan, nightfall).
    pub end_jd: f64,
    /// Hebrew year.
    pub hebrew_year: i32,
}

// ─── Hebrew calendar internals ────────────────────────────────────────────────

/// Is the given Hebrew year a leap year (13 months)?
pub fn is_hebrew_leap_year(year: i32) -> bool {
    (7 * year + 1) % 19 < 7
}

/// Number of months in the Hebrew year.
pub fn months_in_hebrew_year(year: i32) -> i32 {
    if is_hebrew_leap_year(year) {
        13
    } else {
        12
    }
}

/// Elapsed days from Hebrew epoch to 1 Tishrei of the given year.
pub fn elapsed_days(year: i32) -> i64 {
    let months = 235 * ((year - 1) / 19) as i64
        + 12 * ((year - 1) % 19) as i64
        + ((7 * ((year - 1) % 19) + 1) / 19) as i64;
    let parts = 204 + 793 * (months % 1080);
    let hours = 5 + 12 * months + 793 * (months / 1080) + parts / 1080;
    let day = 1 + 29 * months + hours / 24;
    let parts = 1080 * (hours % 24) + parts % 1080;

    // Postponement rules (dechiyot)
    let alt = if parts >= 19440
        || (day % 7 == 2 && parts >= 9924 && !is_hebrew_leap_year(year))
        || (day % 7 == 1 && parts >= 16789 && is_hebrew_leap_year(year - 1))
    {
        day + 1
    } else {
        day
    };
    if alt % 7 == 0 || alt % 7 == 3 || alt % 7 == 5 {
        alt + 1
    } else {
        alt
    }
}

/// Julian day of 1 Tishrei for the given Hebrew year.
pub fn hebrew_new_year_jd(year: i32) -> i64 {
    347_996 + elapsed_days(year)
}

/// Days in the Hebrew year.
pub fn days_in_hebrew_year(year: i32) -> i64 {
    hebrew_new_year_jd(year + 1) - hebrew_new_year_jd(year)
}

/// Days in the given Hebrew month.
pub fn hebrew_month_days(year: i32, month: i32) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 11 => 30,
        2 | 4 | 6 | 10 | 13 => 29,
        8 if days_in_hebrew_year(year) % 10 == 5 => 30,
        8 => 29,
        9 => {
            if days_in_hebrew_year(year) % 10 == 3 {
                29
            } else {
                30
            }
        }
        12 if is_hebrew_leap_year(year) => 30,
        12 => 29,
        _ => 29,
    }
}

/// Julian day of the first day of the given Hebrew month.
pub fn hebrew_month_start_jd(year: i32, month: i32) -> i64 {
    // Walk from Tishrei (7) forward, then Nisan (1) forward
    let months_order: Vec<i32> = (7..=months_in_hebrew_year(year)).chain(1..7).collect();
    let mut jd = hebrew_new_year_jd(year);
    for &m in &months_order {
        if m == month {
            break;
        }
        jd += hebrew_month_days(year, m);
    }
    jd
}

/// Approximate Hebrew year from a Julian day number.
pub fn approx_hebrew_year(jd: f64) -> i32 {
    // Average Hebrew year ≈ 365.25 days; epoch = 347997
    ((jd - 347_997.0) * 98_496.0 / 35_975_351.0) as i32 + 1
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Julian day of 16 Nisan (start of the Omer count) for the given Hebrew year.
///
/// The Omer begins at nightfall on 15 Nisan (the end of the first Passover
/// seder). We return the JD at approximately 18:00 local mean time.
pub fn omer_start_jd(hebrew_year: i32) -> f64 {
    // 16 Nisan = first day of Omer count
    let nisan_1 = hebrew_month_start_jd(hebrew_year, 1);
    (nisan_1 + 15) as f64 + 0.25 // day 16 at ~18:00 (0.25 day = 6 hours after midnight)
}

/// Julian day of a specific Omer day (1–49) in the given Hebrew year.
///
/// Returns `None` if `day` is outside the range 1–49.
pub fn omer_day_jd(hebrew_year: i32, day: u8) -> Option<f64> {
    if !(1..=49).contains(&day) {
        return None;
    }
    Some(omer_start_jd(hebrew_year) + (day - 1) as f64)
}

/// Return the [`OmerDay`] for a given Julian day, or `None` if the JD falls
/// outside the Omer period.
///
/// The Hebrew day starts at nightfall (~18:00). This function uses the
/// astronomical convention that the JD advances at noon, so an evening time
/// (e.g. 20:00 = JD + 0.33) is treated as the beginning of the next Hebrew day.
pub fn omer_from_jd(jd: f64) -> Option<OmerDay> {
    // Find candidate Hebrew year
    let mut year = approx_hebrew_year(jd);
    // Refine (may be off by 1)
    while (omer_start_jd(year + 1)) <= jd {
        year += 1;
    }
    while year > 1 && omer_start_jd(year) > jd + 1.0 {
        year -= 1;
    }

    let start = omer_start_jd(year);
    let elapsed = (jd - start + 0.5).floor() as i64; // +0.5 to handle nightfall

    if !(0..=48).contains(&elapsed) {
        // Try the next year
        let start_next = omer_start_jd(year + 1);
        let elapsed_next = (jd - start_next + 0.5).floor() as i64;
        if !(0..=48).contains(&elapsed_next) {
            return None;
        }
        return omer_from_day(elapsed_next as u8 + 1, start_next + elapsed_next as f64);
    }

    omer_from_day(elapsed as u8 + 1, start + elapsed as f64)
}

/// Build an [`OmerDay`] from a day number (1–49) and its Julian day.
fn omer_from_day(day: u8, jd: f64) -> Option<OmerDay> {
    if !(1..=49).contains(&day) {
        return None;
    }
    let week = (day - 1) / 7 + 1;
    let day_of_week = (day - 1) % 7 + 1;
    let week_sefirah = SEFIROT[(week - 1) as usize];
    let day_sefirah = SEFIROT[(day_of_week - 1) as usize];
    let hebrew_text = OMER_DAY_NAMES[(day - 1) as usize];

    Some(OmerDay {
        day,
        week,
        day_of_week,
        week_sefirah,
        day_sefirah,
        hebrew_text,
        is_lag_baomer: day == 33,
        jd,
    })
}

/// Return the [`OmerPeriod`] (start JD, end JD, Hebrew year) for the Hebrew
/// year that contains the given Julian day.
///
/// If `jd` is not within any Omer period, returns the Omer period of the
/// nearest upcoming Hebrew year.
pub fn omer_period(jd: f64) -> OmerPeriod {
    let mut year = approx_hebrew_year(jd).max(1);
    // Find the year whose Omer period contains or follows jd
    while omer_start_jd(year) + 48.0 < jd {
        year += 1;
    }
    let start = omer_start_jd(year);
    OmerPeriod {
        start_jd: start,
        end_jd: start + 48.0,
        hebrew_year: year,
    }
}

/// Return all 49 [`OmerDay`]s for the given Hebrew year.
pub fn omer_days(hebrew_year: i32) -> Vec<OmerDay> {
    let start = omer_start_jd(hebrew_year);
    (1u8..=49)
        .filter_map(|d| omer_from_day(d, start + (d - 1) as f64))
        .collect()
}

/// Human-readable declaration for the given Omer day.
///
/// Returns the full Hebrew declaration and the Sefirot annotation.
/// ```
/// use celestial_core::omer_declaration;
/// let s = omer_declaration(33);
/// assert!(s.contains("Lag Ba'Omer"));
/// assert!(s.contains("Hod"));
/// ```
pub fn omer_declaration(day: u8) -> String {
    let jd = 0.0; // placeholder — declaration is day-only
    if let Some(d) = omer_from_day(day, jd) {
        let lag = if d.is_lag_baomer {
            " (Lag Ba'Omer)"
        } else {
            ""
        };
        format!(
            "{}{} — {} sheb'{}'",
            d.hebrew_text, lag, d.week_sefirah, d.day_sefirah
        )
    } else {
        String::from("Invalid Omer day")
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn gregorian_to_jd(y: i32, m: i32, d: i32, h: f64) -> f64 {
        let a = (14 - m) / 12;
        let y2 = y + 4800 - a;
        let m2 = m + 12 * a - 3;
        let jd = d + (153 * m2 + 2) / 5 + 365 * y2 + y2 / 4 - y2 / 100 + y2 / 400 - 32045;
        jd as f64 - 0.5 + h / 24.0
    }

    #[test]
    fn omer_day_1_5785() {
        // 16 Nisan 5785 begins at nightfall April 13, 2025
        // (Hebrew day starts at sunset; 15 Nisan seder = April 12/13, so 16 Nisan = April 13 eve)
        let jd = gregorian_to_jd(2025, 4, 13, 20.0);
        let day = omer_from_jd(jd).expect("should be Omer day 1");
        assert_eq!(day.day, 1);
        assert_eq!(day.week, 1);
        assert_eq!(day.day_of_week, 1);
        assert_eq!(day.week_sefirah, "Chesed");
        assert_eq!(day.day_sefirah, "Chesed");
        assert!(!day.is_lag_baomer);
    }

    #[test]
    fn lag_baomer_5785() {
        // Lag Ba'Omer 5785 = day 33 = nightfall May 15, 2025
        let jd = gregorian_to_jd(2025, 5, 15, 20.0);
        let day = omer_from_jd(jd).expect("should be Omer day 33");
        assert_eq!(day.day, 33);
        assert!(day.is_lag_baomer);
        assert_eq!(day.week_sefirah, "Hod");
        assert_eq!(day.day_sefirah, "Hod");
    }

    #[test]
    fn omer_day_49_5785() {
        // Day 49 = 5 Sivan 5785 = nightfall May 31, 2025 (eve of Shavuot June 1)
        let jd = gregorian_to_jd(2025, 5, 31, 20.0);
        let day = omer_from_jd(jd).expect("should be Omer day 49");
        assert_eq!(day.day, 49);
        assert_eq!(day.week, 7);
        assert_eq!(day.day_of_week, 7);
        assert_eq!(day.week_sefirah, "Malkhut");
        assert_eq!(day.day_sefirah, "Malkhut");
    }

    #[test]
    fn outside_omer_returns_none() {
        // Random date in October — not in Omer
        let jd = gregorian_to_jd(2025, 10, 1, 12.0);
        assert!(omer_from_jd(jd).is_none());
    }

    #[test]
    fn all_49_days_present() {
        let days = omer_days(5785);
        assert_eq!(days.len(), 49);
        for (i, d) in days.iter().enumerate() {
            assert_eq!(d.day as usize, i + 1);
        }
    }

    #[test]
    fn sefirot_cycle_correct() {
        let days = omer_days(5785);
        // Day 7 = Malkhut sheb'Chesed (end of week 1)
        assert_eq!(days[6].week_sefirah, "Chesed");
        assert_eq!(days[6].day_sefirah, "Malkhut");
        // Day 8 = Chesed sheb'Gevurah (start of week 2)
        assert_eq!(days[7].week_sefirah, "Gevurah");
        assert_eq!(days[7].day_sefirah, "Chesed");
    }

    #[test]
    fn declaration_contains_lag_baomer() {
        let s = omer_declaration(33);
        assert!(s.contains("Lag Ba'Omer"), "got: {s}");
        assert!(s.contains("Hod"), "got: {s}");
    }

    #[test]
    fn omer_period_5785() {
        let jd = gregorian_to_jd(2025, 5, 1, 12.0); // Mid-Omer
        let p = omer_period(jd);
        assert_eq!(p.hebrew_year, 5785);
        assert!(p.start_jd < jd && jd < p.end_jd);
    }
}
