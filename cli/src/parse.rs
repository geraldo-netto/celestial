//! Shared argument parsers used by all subcommands.

use celestial_core::body::{Body, Calendar};
use celestial_core::{jdnow, julday, revjul};
use celestial_core::{
    CHIRON, JUPITER, MARS, MEAN_NODE, MERCURY, MOON, NEPTUNE, PLUTO, SATURN, SUN, TRUE_NODE,
    URANUS, VENUS,
};

// ─── Date / JD ────────────────────────────────────────────────────────────────

/// Parse a date string into a Julian day number (UT).
///
/// Accepted formats:
/// - `now`                     — current UTC time
/// - `YYYY-MM-DD`              — midnight UT
/// - `YYYY-MM-DD HH:MM`        — that time UT
/// - `YYYY-MM-DD HH:MM:SS`     — that time UT
/// - a bare float              — Julian day number
pub fn parse_date(s: &str) -> Result<f64, String> {
    let s = s.trim();

    if s.eq_ignore_ascii_case("now") {
        return Ok(jdnow());
    }

    // Try bare float (JD)
    if let Ok(jd) = s.parse::<f64>() {
        return Ok(jd);
    }

    // Split date and optional time
    let (date_s, time_s) = match s.split_once(' ') {
        Some((d, t)) => (d, t),
        None => (s, "00:00:00"),
    };

    // Parse YYYY-MM-DD
    let dp: Vec<&str> = date_s.split('-').collect();
    if dp.len() != 3 {
        return Err(format!("expected YYYY-MM-DD, got: {date_s}"));
    }
    let year: i32 = dp[0].parse().map_err(|_| format!("bad year: {}", dp[0]))?;
    let month: i32 = dp[1].parse().map_err(|_| format!("bad month: {}", dp[1]))?;
    let day: i32 = dp[2].parse().map_err(|_| format!("bad day: {}", dp[2]))?;

    // Parse HH: MM[:SS]
    let tp: Vec<&str> = time_s.split(':').collect();
    let hh: f64 = tp
        .first()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0.0);
    let mm: f64 = tp.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(0.0);
    let ss: f64 = tp.get(2).and_then(|s| s.trim().parse().ok()).unwrap_or(0.0);
    let hour = hh + mm / 60.0 + ss / 3600.0;

    Ok(julday(year, month, day, hour, Calendar::Gregorian))
}

/// Format a Julian day as a `YYYY-MM-DD HH: MM UT` string.
pub fn jd_to_str(jd: f64) -> String {
    let d = revjul(jd, Calendar::Gregorian);
    let total_sec = (d.hour * 3600.0).round() as i32; // round to nearest second first
    let total_min = total_sec / 60; // truncate seconds from display
    let h = total_min / 60;
    let m = total_min % 60;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02} UT",
        d.year, d.month, d.day, h, m
    )
}

// ─── Celestial bodies ─────────────────────────────────────────────────────────

/// Parse a body name or number to a body constant.
pub fn parse_body(s: &str) -> Result<i32, String> {
    Ok(match s.to_lowercase().as_str() {
        "sun" => SUN,
        "moon" => MOON,
        "mercury" => MERCURY,
        "venus" => VENUS,
        "mars" => MARS,
        "jupiter" => JUPITER,
        "saturn" => SATURN,
        "uranus" => URANUS,
        "neptune" => NEPTUNE,
        "pluto" => PLUTO,
        "node" | "meannode" | "mean_node" => MEAN_NODE,
        "truenode" | "true_node" => TRUE_NODE,
        "chiron" => CHIRON,
        _ => s.parse::<i32>().map_err(|_| format!("unknown body: {s}"))?,
    })
}

/// The default set of bodies for `calc`.
pub fn default_bodies() -> Vec<i32> {
    vec![
        SUN, MOON, MERCURY, VENUS, MARS, JUPITER, SATURN, URANUS, NEPTUNE,
    ]
}

/// Body number → display name.
pub fn body_name(body: Body) -> &'static str {
    match body.as_raw() {
        0 => "Sun",
        1 => "Moon",
        2 => "Mercury",
        3 => "Venus",
        4 => "Mars",
        5 => "Jupiter",
        6 => "Saturn",
        7 => "Uranus",
        8 => "Neptune",
        9 => "Pluto",
        10 => "Mean Node",
        11 => "True Node",
        15 => "Chiron",
        n => {
            let _ = n;
            "Unknown"
        }
    }
}

// ─── House systems ────────────────────────────────────────────────────────────

/// Parse a house system name or letter to its byte code.
pub fn parse_hsys(s: &str) -> Result<u8, String> {
    Ok(match s.to_lowercase().as_str() {
        "placidus" | "p" => b'P',
        "koch" | "k" => b'K',
        "equal" | "e" => b'E',
        "whole" | "w" => b'W',
        "porphyry" | "o" => b'O',
        "regio" | "regiomontanus" | "r" => b'R',
        "campanus" | "c" => b'C',
        "morinus" | "m" => b'M',
        "alcabitus" | "b" => b'B',
        "axial" | "x" => b'X',
        s if s.len() == 1 => s.as_bytes()[0].to_ascii_uppercase(),
        _ => {
            return Err(format!(
                "unknown house system: {s} \
            (use: placidus, koch, equal, whole, porphyry, regio, campanus, morinus)"
            ))
        }
    })
}

/// House system byte → display name.
pub fn hsys_name(hsys: u8) -> &'static str {
    match hsys {
        b'P' => "Placidus",
        b'K' => "Koch",
        b'E' => "Equal",
        b'W' => "Whole-Sign",
        b'O' => "Porphyry",
        b'R' => "Regiomontanus",
        b'C' => "Campanus",
        b'M' => "Morinus",
        b'B' => "Alcabitus",
        b'X' => "Axial Rotation",
        b'A' | b'G' => "Gauquelin",
        _ => "Unknown",
    }
}

// ─── Sidereal modes ───────────────────────────────────────────────────────────

/// Parse a sidereal mode name to its integer code.
pub fn parse_sid_mode(s: &str) -> Result<i32, String> {
    Ok(match s.to_lowercase().as_str() {
        "fagan" | "fagan-bradley" | "fagan_bradley" => 0,
        "lahiri" => 1,
        "deluce" | "de-luce" => 2,
        "raman" => 3,
        "krishnamurti" => 5,
        "sassanian" => 11,
        s => s
            .parse::<i32>()
            .map_err(|_| format!("unknown sidereal mode: {s}"))?,
    })
}
