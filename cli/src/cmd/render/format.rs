//! Display formatting helpers (DMS longitude, moon phase, JD → date).
//!
//! Extracted from the render god-file (ARCH-8); re-exported by `mod.rs`
//! so call sites keep the short `super::fmt_lon_dms` names.

use celestial_core::body::Calendar;
use celestial_core::JulianDay;
use celestial_core::{lon_to_sign, moon_phase, revjul, MoonPhase};

pub(crate) fn fmt_lon_dms(lon: f64) -> String {
    let (sign_idx, deg_in_sign) = lon_to_sign(lon);
    let d = deg_in_sign as u32;
    let m = ((deg_in_sign - d as f64) * 60.0) as u32;
    let s = (((deg_in_sign - d as f64) * 3600.0) - m as f64 * 60.0).round() as u32;
    // Trailing `\u{FE0E}` forces the text-presentation form of each
    // zodiac glyph — without it SVG renderers fall back to colour-emoji
    // bitmap glyphs (Noto Color Emoji et al.) which look blocky next
    // to the surrounding crisp serif/sans digits.
    let glyphs = [
        "\u{2648}\u{FE0E}",
        "\u{2649}\u{FE0E}",
        "\u{264A}\u{FE0E}",
        "\u{264B}\u{FE0E}",
        "\u{264C}\u{FE0E}",
        "\u{264D}\u{FE0E}",
        "\u{264E}\u{FE0E}",
        "\u{264F}\u{FE0E}",
        "\u{2650}\u{FE0E}",
        "\u{2651}\u{FE0E}",
        "\u{2652}\u{FE0E}",
        "\u{2653}\u{FE0E}",
    ];
    format!(
        "{d:02}\u{00B0}{m:02}\u{2032}{s:02}\u{2033}{}",
        glyphs[sign_idx as usize % 12]
    )
}

pub(crate) fn moon_phase_str(jd: f64) -> &'static str {
    match moon_phase(JulianDay::new(jd)).unwrap_or(MoonPhase::NewMoon) {
        MoonPhase::NewMoon => "New Moon",
        MoonPhase::WaxingCrescent => "Waxing Crescent",
        MoonPhase::FirstQuarter => "First Quarter",
        MoonPhase::WaxingGibbous => "Waxing Gibbous",
        MoonPhase::FullMoon => "Full Moon",
        MoonPhase::WaningGibbous => "Waning Gibbous",
        MoonPhase::LastQuarter => "Last Quarter",
        MoonPhase::WaningCrescent => "Waning Crescent",
    }
}

/// Format a Julian Day as a date string, including HH:MM when the time is not midnight.
pub(crate) fn jd_to_date_str(jd: f64) -> String {
    let d = revjul(JulianDay::new(jd), Calendar::Gregorian);
    let total_sec = (d.hour * 3600.0).round() as i32; // round to nearest second first
    let total_min = total_sec / 60; // truncate seconds from display
    let h = total_min / 60;
    let m = total_min % 60;
    if h == 0 && m == 0 {
        format!(
            "{:04}-{:02}-{:02}",
            d.year as i32, d.month as u32, d.day as u32
        )
    } else {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02} UT",
            d.year as i32, d.month as u32, d.day as u32, h, m
        )
    }
}
