//! Easter and the Christian liturgical year.
//!
//! Implements both the **Gregorian computus** (Western Easter, used since 1583)
//! and the **Julian computus** (Eastern Orthodox Easter, "Old Calendar").
//! All derived moveable feasts (Ash Wednesday, Palm Sunday, Ascension,
//! Pentecost, etc.) are computed from Easter Sunday.
//!
//! # Examples
//! ```
//! use celestial_core::easter_gregorian;
//! let (y, m, d) = easter_gregorian(2025);
//! assert_eq!((y, m, d), (2025, 4, 20));
//! ```

/// Compute Western (Gregorian) Easter Sunday for the given year.
///
/// Uses the Anonymous Gregorian algorithm (also known as the "Meeus/Jones/Butcher" method).
/// Returns `(year, month, day)`.
///
/// Valid for all years 1583–4099.
#[must_use]
pub fn easter_gregorian(year: i32) -> (i32, u8, u8) {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = ((h + l - 7 * m + 114) % 31) + 1;
    (year, month as u8, day as u8)
}

/// Compute Eastern Orthodox Easter Sunday (Julian calendar date) for the given year.
///
/// Uses the Meeus Julian algorithm.
/// Returns `(year, month, day)` in the **Julian calendar**.
#[must_use]
pub fn easter_julian(year: i32) -> (i32, u8, u8) {
    let a = year % 4;
    let b = year.rem_euclid(7);
    let c = year % 19;
    let d = (19 * c + 15) % 30;
    let e = (2 * a + 4 * b - d + 34) % 7;
    let month = (d + e + 114) / 31;
    let day = ((d + e + 114) % 31) + 1;
    (year, month as u8, day as u8)
}

/// Compute Eastern Orthodox Easter in the **Gregorian calendar** for the given year.
///
/// Converts the Julian date to Gregorian by adding the appropriate century correction.
#[must_use]
pub fn easter_orthodox(year: i32) -> (i32, u8, u8) {
    let (y, m, d) = easter_julian(year);
    let correction = julian_to_gregorian_offset(year);

    // Add correction days
    let mut month = m as i32;
    let mut day = d as i32 + correction;

    let month_days = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    while day > month_days[month as usize] {
        day -= month_days[month as usize];
        month += 1;
    }
    (y, month as u8, day as u8)
}

fn julian_to_gregorian_offset(year: i32) -> i32 {
    year / 100 - year / 400 - 2
}

fn gregorian_to_jd(y: i32, m: i32, d: i32) -> f64 {
    let a = (14 - m) / 12;
    let y2 = y + 4800 - a;
    let m2 = m + 12 * a - 3;
    let jd = d + (153 * m2 + 2) / 5 + 365 * y2 + y2 / 4 - y2 / 100 + y2 / 400 - 32045;
    jd as f64 - 0.5
}

/// A Christian feast day with its date and offset from Easter.
#[derive(Debug, Clone)]
pub struct ChristianFeast {
    /// Name of the feast.
    pub name: &'static str,
    /// Days offset from Easter Sunday (negative = before Easter).
    pub easter_offset: i32,
    /// Julian day of the feast.
    pub jd: f64,
    /// Gregorian year, month, day.
    pub year: i32,
    /// Gregorian month of the feast (1–12).
    pub month: u8,
    /// Gregorian day of the feast (1–31).
    pub day: u8,
}

/// Moveable feasts relative to Easter Sunday.
const MOVEABLE_FEASTS: &[(&str, i32)] = &[
    ("Septuagesima Sunday", -63),
    ("Sexagesima Sunday", -56),
    ("Quinquagesima Sunday", -49),
    ("Shrove Tuesday (Mardi Gras)", -47),
    ("Ash Wednesday", -46),
    ("Palm Sunday", -7),
    ("Holy (Maundy) Thursday", -3),
    ("Good Friday", -2),
    ("Holy Saturday", -1),
    ("Easter Sunday", 0),
    ("Easter Monday", 1),
    ("Ascension Thursday", 39),
    ("Pentecost (Whit Sunday)", 49),
    ("Whit Monday", 50),
    ("Trinity Sunday", 56),
    ("Corpus Christi", 60),
];

/// Compute all Western Christian moveable feasts for the given year.
#[must_use]
pub fn christian_feasts(year: i32) -> Vec<ChristianFeast> {
    let (ey, em, ed) = easter_gregorian(year);
    let easter_jd = gregorian_to_jd(ey, em as i32, ed as i32);

    MOVEABLE_FEASTS
        .iter()
        .map(|&(name, offset)| {
            let jd = easter_jd + offset as f64;
            // JD to Gregorian
            let jd_n = (jd + 0.5) as i64;
            let l = jd_n + 68569;
            let n = (4 * l) / 146097;
            let l = l - (146097 * n + 3) / 4;
            let i = (4000 * (l + 1)) / 1461001;
            let l = l - (1461 * i) / 4 + 31;
            let j = (80 * l) / 2447;
            let d = l - (2447 * j) / 80;
            let l = j / 11;
            let m = j + 2 - 12 * l;
            let y = 100 * (n - 49) + i + l;
            ChristianFeast {
                name,
                easter_offset: offset,
                jd,
                year: y as i32,
                month: m as u8,
                day: d as u8,
            }
        })
        .collect()
}

/// Fixed Christian feasts (non-moveable) for the given year.
#[must_use]
pub fn christian_fixed_feasts(year: i32) -> Vec<ChristianFeast> {
    let fixed: &[(&str, i32, i32)] = &[
        ("Epiphany", 1, 6),
        ("Candlemas", 2, 2),
        ("Annunciation", 3, 25),
        ("Nativity of John the Baptist", 6, 24),
        ("Assumption of Mary", 8, 15),
        ("All Saints' Day", 11, 1),
        ("All Souls' Day", 11, 2),
        ("Feast of the Immaculate Conception", 12, 8),
        ("Christmas Day", 12, 25),
        ("St. Stephen's Day", 12, 26),
    ];

    fixed
        .iter()
        .map(|&(name, m, d)| {
            let jd = gregorian_to_jd(year, m, d);
            ChristianFeast {
                name,
                easter_offset: 0,
                jd,
                year,
                month: m as u8,
                day: d as u8,
            }
        })
        .collect()
}

/// Julian day of Western (Gregorian) Easter for the given year.
#[must_use]
pub fn easter_jd(year: i32) -> f64 {
    let (y, m, d) = easter_gregorian(year);
    gregorian_to_jd(y, m as i32, d as i32)
}

/// Julian day of Eastern Orthodox Easter (in Gregorian calendar) for the given year.
#[must_use]
pub fn easter_orthodox_jd(year: i32) -> f64 {
    let (y, m, d) = easter_orthodox(year);
    gregorian_to_jd(y, m as i32, d as i32)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easter_2025_gregorian() {
        assert_eq!(easter_gregorian(2025), (2025, 4, 20));
    }

    #[test]
    fn easter_2024_gregorian() {
        assert_eq!(easter_gregorian(2024), (2024, 3, 31));
    }

    #[test]
    fn easter_2019_gregorian() {
        assert_eq!(easter_gregorian(2019), (2019, 4, 21));
    }

    #[test]
    fn easter_orthodox_2025() {
        // Orthodox Easter 2025 = April 20 (same as Gregorian in 2025)
        let (y, m, _d) = easter_orthodox(2025);
        assert_eq!(y, 2025);
        assert!(m == 4 || m == 5); // April or May range
    }

    #[test]
    fn easter_orthodox_1600_uses_historical_offset() {
        assert_eq!(easter_orthodox(1600), (1600, 4, 2));
    }

    #[test]
    fn christian_feasts_2025_count() {
        let feasts = christian_feasts(2025);
        assert_eq!(feasts.len(), MOVEABLE_FEASTS.len());
    }

    #[test]
    fn ash_wednesday_2025() {
        // Easter 2025 = April 20; Ash Wednesday = 46 days before = March 5
        let feasts = christian_feasts(2025);
        let aw = feasts.iter().find(|f| f.name == "Ash Wednesday").unwrap();
        assert_eq!((aw.month, aw.day), (3, 5));
    }

    #[test]
    fn good_friday_2025() {
        let feasts = christian_feasts(2025);
        let gf = feasts.iter().find(|f| f.name == "Good Friday").unwrap();
        assert_eq!((gf.month, gf.day), (4, 18));
    }

    #[test]
    fn pentecost_2025() {
        // Easter April 20 + 49 = June 8
        let feasts = christian_feasts(2025);
        let pent = feasts
            .iter()
            .find(|f| f.name.contains("Pentecost"))
            .unwrap();
        assert_eq!((pent.month, pent.day), (6, 8));
    }

    fn mix(hash: u64, value: u64) -> u64 {
        hash.wrapping_mul(1_099_511_628_211) ^ value
    }

    fn mix_date(hash: u64, date: (i32, u8, u8)) -> u64 {
        let hash = mix(hash, date.0 as u64);
        let hash = mix(hash, date.1 as u64);
        mix(hash, date.2 as u64)
    }

    fn mix_name(mut hash: u64, name: &str) -> u64 {
        for byte in name.bytes() {
            hash = mix(hash, byte as u64);
        }
        hash
    }

    fn mix_feast(hash: u64, feast: &ChristianFeast) -> u64 {
        let hash = mix_name(hash, feast.name);
        let hash = mix(hash, feast.easter_offset as u64);
        let hash = mix(hash, feast.jd.to_bits());
        mix_date(hash, (feast.year, feast.month, feast.day))
    }

    #[test]
    fn computus_algorithms_match_full_range_fingerprint() {
        let mut hash = 14_695_981_039_346_656_037;
        for year in 1583..=4099 {
            hash = mix_date(hash, easter_gregorian(year));
            hash = mix_date(hash, easter_julian(year));
            hash = mix_date(hash, easter_orthodox(year));
            hash = mix(hash, easter_jd(year).to_bits());
            hash = mix(hash, easter_orthodox_jd(year).to_bits());
            hash = mix(hash, julian_to_gregorian_offset(year) as u64);
        }
        assert_eq!(hash, 29_964_202_374_967_097);
    }

    #[test]
    fn christian_feasts_match_full_range_fingerprint() {
        let mut hash = 14_695_981_039_346_656_037;
        for year in 1583..=4099 {
            for feast in christian_feasts(year) {
                hash = mix_feast(hash, &feast);
            }
            for feast in christian_fixed_feasts(year) {
                hash = mix_feast(hash, &feast);
            }
        }
        assert_eq!(hash, 90_213_548_656_065_479);
    }

    #[test]
    fn gregorian_jd_matches_calendar_fingerprint() {
        let mut hash = 14_695_981_039_346_656_037;
        for year in 1583..=4099 {
            for month in 1..=12 {
                hash = mix(hash, gregorian_to_jd(year, month, 1).to_bits());
                hash = mix(hash, gregorian_to_jd(year, month, 28).to_bits());
            }
        }
        assert_eq!(hash, 5_480_982_162_503_307_397);
    }
}
