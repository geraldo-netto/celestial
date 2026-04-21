//! Hebrew calendar arithmetic (shared by Omer and Jewish holidays).
//!
//! Based on the standard postponement algorithm (dechiyot).
//! All JD values are integral (integer day boundaries).

/// Is the given Hebrew year a leap year (13 months)?
pub fn is_leap_year(year: i32) -> bool {
    (7 * year + 1) % 19 < 7
}

/// Number of months in the Hebrew year.
pub fn months_in_year(year: i32) -> i32 {
    if is_leap_year(year) {
        13
    } else {
        12
    }
}

/// Elapsed days from the Hebrew epoch to 1 Tishrei of the given year.
pub fn elapsed_days(year: i32) -> i64 {
    let months = 235 * ((year - 1) / 19) as i64
        + 12 * ((year - 1) % 19) as i64
        + ((7 * ((year - 1) % 19) + 1) / 19) as i64;
    let parts = 204 + 793 * (months % 1080);
    let hours = 5 + 12 * months + 793 * (months / 1080) + parts / 1080;
    let day = 1 + 29 * months + hours / 24;
    let parts = 1080 * (hours % 24) + parts % 1080;

    let alt = if parts >= 19440
        || (day % 7 == 2 && parts >= 9924 && !is_leap_year(year))
        || (day % 7 == 1 && parts >= 16789 && is_leap_year(year - 1))
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

/// Julian day of 1 Tishrei (New Year) for the given Hebrew year.
pub fn new_year_jd(year: i32) -> i64 {
    347_996 + elapsed_days(year)
}

/// Days in the Hebrew year.
pub fn days_in_year(year: i32) -> i64 {
    new_year_jd(year + 1) - new_year_jd(year)
}

/// Days in the given Hebrew month.
pub fn month_days(year: i32, month: i32) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 11 => 30,
        2 | 4 | 6 | 10 | 13 => 29,
        8 if days_in_year(year) % 10 == 5 => 30,
        8 => 29,
        9 if days_in_year(year) % 10 == 3 => 29,
        9 => 30,
        12 if is_leap_year(year) => 30,
        12 => 29,
        _ => 29,
    }
}

/// Julian day of the 1st day of the given Hebrew month.
pub fn month_start_jd(year: i32, month: i32) -> i64 {
    let months_order: Vec<i32> = (7..=months_in_year(year)).chain(1..7).collect();
    let mut jd = new_year_jd(year);
    for &m in &months_order {
        if m == month {
            break;
        }
        jd += month_days(year, m);
    }
    jd
}

/// Approximate Hebrew year from a Julian day number.
pub fn approx_year(jd: f64) -> i32 {
    ((jd - 347_997.0) * 98_496.0 / 35_975_351.0) as i32 + 1
}
