//! Time and date conversion functions.

use crate::body::{CalcFlags, Calendar};

use crate::error::Result;

// ─── Return types ─────────────────────────────────────────────────────────────

/// A calendar date broken down into components.
#[derive(Debug, Clone, PartialEq)]
pub struct CalDate {
    /// Year.
    pub year: i32,
    /// Month (1–12).
    pub month: i32,
    /// Day (1–31).
    pub day: i32,
    /// Hour including fractional part.
    pub hour: f64,
}

/// A pair of Julian day numbers (ET and UT1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JdPair {
    /// Julian day (Terrestrial Time).
    pub et: f64,
    /// Julian day (UT1).
    pub ut1: f64,
}

/// A UTC date-time structure used by [`utc_to_jd`] and related functions.
#[derive(Debug, Clone, PartialEq)]
pub struct UtcDate {
    /// Year.
    pub year: i32,
    /// Month.
    pub month: i32,
    /// Day.
    pub day: i32,
    /// Hour.
    pub hour: i32,
    /// Minute.
    pub minute: i32,
    /// Second (including fractional part).
    pub second: f64,
}

// ─── Julian day ───────────────────────────────────────────────────────────────

/// Convert a calendar date to a Julian day number.
///
/// `calendar` — use [`crate::GREG_CAL`] or [`crate::JUL_CAL`].
#[must_use]
pub fn julday(year: i32, month: i32, day: i32, hour: f64, calendar: Calendar) -> f64 {
    // Pure-Rust Meeus algorithm (same regardless of feature flag)
    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let a = (y as f64 / 100.0).floor() as i64;
    let b = if calendar == Calendar::Gregorian {
        2 - a + (a / 4)
    } else {
        0
    };
    (365.25 * (y as f64 + 4716.0)).floor()
        + (30.6001 * (m as f64 + 1.0)).floor()
        + day as f64
        + hour / 24.0
        + b as f64
        - 1524.5
}

/// Convert a Julian day number to a calendar date.
#[must_use]
pub fn revjul(jd: f64, calendar: Calendar) -> CalDate {
    // Defensive: extreme f64 values (NaN, ±Inf, ±f64::MAX) cause the
    // floor()/as-i64 chain below to wrap or saturate at i64::MAX, which then
    // overflows on the next add. Map any non-finite or astronomically-absurd
    // input to a sentinel epoch to keep the function panic-free for fuzzers.
    if !jd.is_finite() || jd.abs() > 1.0e10 {
        return CalDate {
            year: 0,
            month: 1,
            day: 1,
            hour: 0.0,
        };
    }

    let jd = jd + 0.5;
    let z = jd.floor() as i64;
    let f = jd.fract();
    let a = if calendar == Calendar::Gregorian {
        let alpha = ((z as f64 - 1867216.25) / 36524.25).floor() as i64;
        z + 1 + alpha - alpha / 4
    } else {
        z
    };
    let b = a + 1524;
    let c = ((b as f64 - 122.1) / 365.25).floor() as i64;
    let d = (365.25 * c as f64).floor() as i64;
    let e = ((b - d) as f64 / 30.6001).floor() as i64;
    let day = (b - d) as i32 - (30.6001 * e as f64).floor() as i32;
    let month = if e < 14 {
        (e - 1) as i32
    } else {
        (e - 13) as i32
    };
    let year = if month > 2 {
        (c - 4716) as i32
    } else {
        (c - 4715) as i32
    };
    let hour = f * 24.0;
    CalDate {
        year,
        month,
        day,
        hour,
    }
}

/// Convert a calendar date to a Julian day using character calendar flag.
///
/// `cal` — `b'g'` for Gregorian, `b'j'` for Julian.
pub fn date_conversion(year: i32, month: i32, day: i32, hour: f64, cal: u8) -> Result<f64> {
    let calendar = if cal == b'g' {
        Calendar::Gregorian
    } else {
        Calendar::Julian
    };
    Ok(julday(year, month, day, hour, calendar))
}

/// Convert UTC date/time to Julian day (returns both ET and UT1).
pub fn utc_to_jd(date: &UtcDate, calendar: Calendar) -> Result<JdPair> {
    // Pure: simple conversion without leap-second tables
    let hour = date.hour as f64 + date.minute as f64 / 60.0 + date.second / 3600.0;
    let ut1 = julday(date.year, date.month, date.day, hour, calendar);
    let dt = crate::astronomy::deltat(ut1) / 86_400.0;
    Ok(JdPair { et: ut1 + dt, ut1 })
}

/// Convert ET Julian day to UTC components.
#[must_use]
pub fn jd_et_to_utc(jd_et: f64, calendar: Calendar) -> UtcDate {
    let dt = crate::astronomy::deltat(jd_et) / 86_400.0;
    let jd_ut = jd_et - dt;
    jd_ut_to_utcdate(jd_ut, calendar)
}

/// Convert UT1 Julian day to UTC components.
#[must_use]
pub fn jd_ut_to_utc(jd_ut: f64, calendar: Calendar) -> UtcDate {
    jd_ut_to_utcdate(jd_ut, calendar)
}

/// Convert a Julian day (UT) to a UTC date.
fn jd_ut_to_utcdate(jd: f64, calendar: Calendar) -> UtcDate {
    let cd = revjul(jd, calendar);
    let hour_f = cd.hour;
    let h = hour_f.floor() as i32;
    let min_f = (hour_f - h as f64) * 60.0;
    let m = min_f.floor() as i32;
    let s = (min_f - m as f64) * 60.0;
    UtcDate {
        year: cd.year,
        month: cd.month,
        day: cd.day,
        hour: h,
        minute: m,
        second: s,
    }
}

/// Apply a timezone offset to a UTC date.
#[must_use]
pub fn utc_time_zone(date: &UtcDate, d_timezone: f64) -> UtcDate {
    // Pure implementation: add offset in whole minutes to avoid float rounding
    let total_minutes = date.hour * 60 + date.minute + (d_timezone * 60.0).round() as i32;
    let total_minutes = total_minutes + date.day * 1440 + 10 * 1440; // ensure positive
    let day_offset = total_minutes.div_euclid(1440) - date.day - 10;
    let new_minutes = total_minutes.rem_euclid(1440);
    let new_hour = new_minutes / 60;
    let new_min = new_minutes % 60;
    let base_jd = julday(
        date.year,
        date.month,
        date.day + day_offset,
        0.0,
        Calendar::Gregorian,
    );
    let cd = revjul(base_jd, Calendar::Gregorian);
    UtcDate {
        year: cd.year,
        month: cd.month,
        day: cd.day,
        hour: new_hour,
        minute: new_min,
        second: date.second,
    }
}

// ─── Delta T and sidereal time ────────────────────────────────────────────────

/// Compute ΔT for a given Julian day (UT), returned in **days**.
///
/// ΔT is the accumulated difference between Terrestrial Time (TT) and
/// Universal Time (UT): ΔT = TT − UT. It arises from the irregular rotation
/// of the Earth and accumulates ~1 ms per century on top of long-period
/// variations from tidal braking and core-mantle coupling.
///
/// At J2000.0 (JD 2451545.0), ΔT ≈ 63.83 seconds ≈ 0.000739 days.
///
/// # Units — important
/// This function returns ΔT in **days** to match the Swiss Ephemeris `swe_deltat`
/// convention. If you want seconds, either multiply by 86400 or use
/// [`deltat_ex`] which returns seconds directly.
///
/// # Overrides
/// Returns the user-defined value (set via `set_delta_t_userdef`) when one
/// is installed; otherwise evaluates the built-in Espenak/Meeus polynomial
/// fit (see [`astronomy::delta_t_for_year`]).
#[must_use]
pub fn deltat(tjd: f64) -> f64 {
    if let Some(dt) = crate::functions::config::user_delta_t() {
        return dt;
    }
    crate::astronomy::deltat(tjd) / 86_400.0
}

/// Compute ΔT for a given Julian day (UT), returned in **seconds**.
///
/// Companion to [`deltat`] that preserves the native seconds unit instead of
/// converting to days. Matches Swiss Ephemeris `swe_deltat_ex` semantics.
///
/// At J2000.0, returns ≈ 63.83 s.
pub fn deltat_ex(jd: f64, _flags: CalcFlags) -> Result<f64> {
    if let Some(dt) = crate::functions::config::user_delta_t() {
        return Ok(dt * 86_400.0);
    }
    Ok(crate::astronomy::deltat(jd))
}

/// Greenwich Mean Sidereal Time (GMST) in decimal hours — without the
/// equation of the equinoxes. Use [`sidtime`] for apparent sidereal time (GAST).
#[must_use]
pub fn mean_sidtime(jd_ut: f64) -> f64 {
    crate::astronomy::houses::mean_sidereal_time_deg(jd_ut) / 15.0
}

/// Compute apparent sidereal time for a UT Julian day.
#[must_use]
pub fn sidtime(jd_ut: f64) -> f64 {
    crate::astronomy::houses::sidereal_time_deg(jd_ut) / 15.0
}

/// Compute apparent sidereal time with obliquity and nutation.
#[must_use]
pub fn sidtime0(jd_ut: f64, eps: f64, nut: f64) -> f64 {
    let gmst = crate::astronomy::houses::sidereal_time_deg(jd_ut);
    nut.mul_add(eps.to_radians().cos(), gmst) / 15.0
}

/// Compute the day of week (0 = Monday, …, 6 = Sunday).
#[must_use]
pub fn day_of_week(jd: f64) -> i32 {
    // Zeller / JD mod 7: JD 0 = Monday
    ((jd + 0.5).floor() as i64).rem_euclid(7) as i32
}

/// Compute the equation of time.
/// Equation of time at Julian Day (UT) — the difference between
/// apparent solar time and mean solar time, in hours.
///
/// Positive when the sundial is ahead of the clock.
/// Range: roughly −14 to +16 minutes (±0.27 hours).
///
/// Algorithm: Meeus, "Astronomical Algorithms" ch. 27.
pub fn time_equ(jd_ut: f64) -> Result<f64> {
    let jde = crate::astronomy::delta_t::ut_to_tt(jd_ut);
    // Julian centuries from J2000.0
    let t = (jde - 2_451_545.0) / 36525.0;

    // Geometric mean longitude of Sun (degrees)
    let l0 = 36_000.770_1_f64.mul_add(t, 280.460_6).rem_euclid(360.0);
    // Mean anomaly of Sun (degrees)
    let m = 35_999.050_3_f64.mul_add(t, 357.528_3).rem_euclid(360.0);
    let m_r = m.to_radians();

    // Equation of centre (degrees)
    let c = 1.9146_f64
        .mul_add(m_r.sin(), 0.020 * (2.0 * m_r).sin())
        + 0.0003 * (3.0 * m_r).sin();

    // Sun's true longitude (degrees)
    let sun_lon = (l0 + c).rem_euclid(360.0);

    // Apparent right ascension of the Sun (degrees)
    // Obliquity of ecliptic
    let eps = 23.439_2_f64 - 0.013_0 * t;
    let eps_r = eps.to_radians();
    let sun_r = sun_lon.to_radians();
    let (sin_sun, cos_sun) = sun_r.sin_cos();
    let ra = (eps_r.cos() * sin_sun).atan2(cos_sun).to_degrees();
    let ra = ra.rem_euclid(360.0);

    // RA of mean Sun = mean longitude L0 (to good approximation)
    let ra_mean = l0;

    // Equation of time = apparent solar time - mean solar time (positive = sundial ahead)
    // Convention: E = (RA_apparent - RA_mean) / 15
    let e_deg = crate::functions::utils::wrap_signed_180(ra - ra_mean);
    Ok(e_deg / 15.0) // hours
}

/// Convert local apparent time to local mean time.
pub fn lat_to_lmt(tjd_lat: f64, geolon: f64) -> Result<f64> {
    // Pure: simple longitude correction
    Ok(tjd_lat - geolon / 360.0)
}

/// Convert local mean time to local apparent time.
pub fn lmt_to_lat(tjd_lmt: f64, geolon: f64) -> Result<f64> {
    Ok(tjd_lmt + geolon / 360.0)
}

// ─── Date/time display & parsing helpers (ex datetime.rs) ───────────────────────

/// Current Julian Day (UT) derived from the system clock.
#[must_use]
pub fn jdnow() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64());
    // Unix epoch = JD 2440587.5
    2_440_587.5 + secs / 86_400.0
}

/// Decompose a Julian day into `[year, month, day, hour, minute, second]`.
#[must_use]
pub fn revjul_hms(jd: f64, calendar: Calendar) -> [i32; 6] {
    let d = crate::revjul(jd, calendar);
    let frac = d.hour; // decimal hours
    let h = frac as i32;
    let min_f = (frac - h as f64) * 60.0;
    let m = min_f as i32;
    let s = ((min_f - m as f64) * 60.0).round() as i32;
    let s = s.min(59);
    [d.year, d.month, d.day, h, m, s]
}

/// Parse an ISO-8601-style datetime string into `[year, month, day, hour, min, sec]`.
///
/// Accepts any separator between fields; leading `-` counts for year only.
/// Returns `None` if invalid.
pub fn parse_datetime(s: &str) -> Option<[i32; 6]> {
    let s = s.trim();
    let negative = s.starts_with('-');
    // Strip everything that isn't a digit, replacing with space
    let clean: String = s
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c == '-' && i == 0 {
                ' '
            } else if c.is_ascii_digit() {
                c
            } else {
                ' '
            }
        })
        .collect();
    let parts: Vec<i32> = clean
        .split_whitespace()
        .filter_map(|t| t.parse().ok())
        .collect();
    if parts.len() < 3 {
        return None;
    }
    let year = if negative { -parts[0] } else { parts[0] };
    let month = parts[1];
    let day = parts[2];
    let hour = parts.get(3).copied().unwrap_or(0);
    let min = parts.get(4).copied().unwrap_or(0);
    let sec = parts.get(5).copied().unwrap_or(0);
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&min)
        || !(0..=59).contains(&sec)
    {
        return None;
    }
    Some([year, month, day, hour, min, sec])
}

/// Parse a time string into `[hour, min, sec]`.
pub fn parse_time(s: &str) -> Option<[i32; 3]> {
    let clean: String = s
        .chars()
        .map(|c| if c.is_ascii_digit() { c } else { ' ' })
        .collect();
    let parts: Vec<i32> = clean
        .split_whitespace()
        .filter_map(|t| t.parse().ok())
        .collect();
    if parts.is_empty() {
        return None;
    }
    let h = parts[0];
    let m = parts.get(1).copied().unwrap_or(0);
    let s = parts.get(2).copied().unwrap_or(0);
    if !(0..=23).contains(&h) || !(0..=59).contains(&m) || !(0..=59).contains(&s) {
        return None;
    }
    Some([h, m, s])
}

/// Duration between two Julian days → `[days, hours, minutes, seconds]`.
#[must_use]
pub fn jd_duration(jd_start: f64, jd_end: f64) -> [i32; 4] {
    let mut span = (jd_end - jd_start).abs();
    let days = span as i32;
    span -= days as f64;
    let hours = (span * 24.0) as i32;
    span -= hours as f64 / 24.0;
    let minutes = (span * 1440.0) as i32;
    span -= minutes as f64 / 1440.0;
    let seconds = (span * 86400.0) as i32;
    [days, hours, minutes, seconds]
}

/// Format a Julian day as an ISO-8601 string `"YYYY-MM-DD HH:MM:SS UTC"`.
#[must_use]
pub fn jd_to_iso_string(jd: f64, calendar: Calendar) -> String {
    let [y, mo, d, h, mi, s] = revjul_hms(jd, calendar);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02} UTC")
}

// ─── Obliquity & nutation ─────────────────────────────────────────────────────

/// Mean obliquity of the ecliptic in degrees at a Julian Ephemeris Day.
#[must_use]
pub fn mean_obliquity(jde: f64) -> f64 {
    crate::astronomy::obliquity(jde)
}

/// True (apparent) obliquity of the ecliptic in degrees (includes nutation).
#[must_use]
pub fn true_obliquity(jde: f64) -> f64 {
    crate::astronomy::obliquity_true(jde)
}

/// Nutation in longitude and obliquity (degrees) at a Julian Ephemeris Day.
///
/// Returns `(nutation_longitude_deg, nutation_obliquity_deg)`.
#[must_use]
pub fn nutation(jde: f64) -> (f64, f64) {
    let (nl, no) = crate::astronomy::get_nutation(jde);
    (nl / 3600.0, no / 3600.0)
}

/// TT (Terrestrial Time) to UT1 — returns the UT1 Julian day.
#[allow(dead_code)]
#[must_use]
pub fn tt_to_ut(jde: f64) -> f64 {
    crate::astronomy::delta_t::tt_to_ut(jde)
}

// ═══════════════════════════════════════════════════════════════════════════
// ISO 8601 week number
// ═══════════════════════════════════════════════════════════════════════════

/// Day-of-year (1-366) for a Gregorian date.
#[must_use]
pub fn day_of_year(year: i32, month: u32, day: u32) -> u32 {
    let jd_jan1 = julday(year, 1, 1, 12.0, Calendar::Gregorian);
    let jd_d = julday(year, month as i32, day as i32, 12.0, Calendar::Gregorian);
    ((jd_d - jd_jan1) as u32) + 1
}

/// Number of ISO weeks in a given Gregorian year (52 or 53).
///
/// An ISO year has 53 weeks iff Jan 1 or Dec 31 falls on a Thursday.
#[must_use]
pub fn weeks_in_iso_year(year: i32) -> u32 {
    let jan1 = day_of_week(julday(year, 1, 1, 12.0, Calendar::Gregorian));
    let dec31 = day_of_week(julday(year, 12, 31, 12.0, Calendar::Gregorian));
    // day_of_week returns 0=Monday..6=Sunday — Thursday = 3
    if jan1 == 3 || dec31 == 3 {
        53
    } else {
        52
    }
}

/// ISO 8601 week number as `(iso_year, week_number)`.
///
/// The ISO year can differ from the calendar year near Jan 1 / Dec 31:
/// early January dates may belong to the previous ISO year, late December
/// dates may belong to the next ISO year. Week 1 is the week containing
/// the first Thursday of the year.
#[must_use]
pub fn iso_week(jd: f64) -> (i32, u32) {
    let d = revjul(jd, Calendar::Gregorian);
    let ordinal = day_of_year(d.year, d.month as u32, d.day as u32) as i32;
    // day_of_week: 0=Monday..6=Sunday; ISO weekday: 1=Monday..7=Sunday
    let iso_wd = day_of_week(jd) + 1;
    let week = (ordinal - iso_wd + 10) / 7;
    if week < 1 {
        let prev_year = d.year - 1;
        (prev_year, weeks_in_iso_year(prev_year))
    } else if week > weeks_in_iso_year(d.year) as i32 {
        (d.year + 1, 1)
    } else {
        (d.year, week as u32)
    }
}

#[cfg(test)]
mod iso_week_tests {
    use super::*;

    #[test]
    fn iso_week_mon_jan1() {
        // 2024-01-01 Monday → (2024, 1)
        let jd = julday(2024, 1, 1, 12.0, Calendar::Gregorian);
        assert_eq!(iso_week(jd), (2024, 1));
    }

    #[test]
    fn iso_week_sun_jan1_belongs_prev_year() {
        // 2023-01-01 Sunday → last week of 2022
        let jd = julday(2023, 1, 1, 12.0, Calendar::Gregorian);
        assert_eq!(iso_week(jd), (2022, 52));
    }

    #[test]
    fn iso_53_week_year() {
        // 2020-12-31 Thursday → (2020, 53)
        let jd = julday(2020, 12, 31, 12.0, Calendar::Gregorian);
        assert_eq!(iso_week(jd), (2020, 53));
    }

    #[test]
    fn day_of_year_basic() {
        assert_eq!(day_of_year(2024, 1, 1), 1);
        assert_eq!(day_of_year(2024, 12, 31), 366); // leap year
        assert_eq!(day_of_year(2023, 12, 31), 365);
    }

    #[test]
    fn weeks_in_year() {
        assert_eq!(weeks_in_iso_year(2020), 53);
        assert_eq!(weeks_in_iso_year(2021), 52);
        assert_eq!(weeks_in_iso_year(2026), 53);
    }
}
