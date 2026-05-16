//! Shared argument parsers used by all subcommands.
//!
//! Each parseable concept is a small newtype with a `FromStr`
//! implementation whose `Err` is the single [`ParseError`]. The free
//! functions (`parse_date`, `parse_tz_offset`, …) are thin wrappers that
//! return the inner primitive, kept for call-site ergonomics. `?` lifts
//! a [`ParseError`] into [`crate::error::CliError`] via `From`.

use celestial_core::body::{Body, Calendar, HouseSystem};
use celestial_core::{jdnow, julday, revjul};
use celestial_core::{
    CHIRON, JUPITER, MARS, MEAN_NODE, MERCURY, MOON, NEPTUNE, PLUTO, SATURN, SUN, TRUE_NODE,
    URANUS, VENUS,
};
use std::str::FromStr;

/// The single error type for every argument parser.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    /// Malformed date / Julian-day specifier.
    #[error("{0}")]
    Date(String),
    /// Unknown / ambiguous / malformed timezone or numeric offset.
    #[error("{0}")]
    Tz(String),
    /// Unknown celestial body name or number.
    #[error("{0}")]
    Body(String),
    /// Unknown house-system name or letter.
    #[error("{0}")]
    HouseSys(String),
    /// Unknown sidereal-mode name or number.
    #[error("{0}")]
    SidMode(String),
    /// A natal/derived chart was given a date with no time-of-day.
    #[error("{0}")]
    NeedsTime(String),
}

// ─── Date / JD ────────────────────────────────────────────────────────────────

/// A Julian day (UT) parsed from a date specifier.
///
/// Accepted formats:
/// - `now`                     — current UTC time
/// - `YYYY-MM-DD`              — midnight UT
/// - `YYYY-MM-DD HH:MM`        — that time UT
/// - `YYYY-MM-DD HH:MM:SS`     — that time UT
/// - a bare float              — Julian day number
#[derive(Debug, Clone, Copy)]
pub struct DateJd(pub f64);

impl FromStr for DateJd {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();

        if s.eq_ignore_ascii_case("now") {
            return Ok(DateJd(jdnow()));
        }

        // Try bare float (JD)
        if let Ok(jd) = s.parse::<f64>() {
            return Ok(DateJd(jd));
        }

        // Split date and optional time
        let (date_s, time_s) = match s.split_once(' ') {
            Some((d, t)) => (d, t),
            None => (s, "00:00:00"),
        };

        // Parse YYYY-MM-DD
        let dp: Vec<&str> = date_s.split('-').collect();
        if dp.len() != 3 {
            return Err(ParseError::Date(format!("expected YYYY-MM-DD, got: {date_s}")));
        }
        let year: i32 = dp[0]
            .parse()
            .map_err(|_| ParseError::Date(format!("bad year: {}", dp[0])))?;
        let month: i32 = dp[1]
            .parse()
            .map_err(|_| ParseError::Date(format!("bad month: {}", dp[1])))?;
        let day: i32 = dp[2]
            .parse()
            .map_err(|_| ParseError::Date(format!("bad day: {}", dp[2])))?;

        // Parse HH:MM[:SS]
        let tp: Vec<&str> = time_s.split(':').collect();
        let hh: f64 = tp
            .first()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0.0);
        let mm: f64 = tp.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(0.0);
        let ss: f64 = tp.get(2).and_then(|s| s.trim().parse().ok()).unwrap_or(0.0);
        let hour = hh + mm / 60.0 + ss / 3600.0;

        Ok(DateJd(julday(year, month, day, hour, Calendar::Gregorian)))
    }
}

/// Parse a date string into a Julian day number (UT). See [`DateJd`].
pub fn parse_date(s: &str) -> Result<f64, ParseError> {
    DateJd::from_str(s).map(|d| d.0)
}

/// A UTC offset in **hours, east-positive**.
///
/// Accepted forms (case-insensitive):
/// - `UTC`, `GMT`, `Z`                         — offset 0
/// - `+HH`, `-HH`, `+HH:MM`, `-HHMM`           — explicit numeric offset
/// - `UTC+HH:MM`, `GMT-HH`                      — same, with prefix
/// - a timezone abbreviation (`BRT`, `EST`, …) — resolved via the built-in
///   203-entry table. Abbreviations that map to more than one offset
///   (e.g. `CST`, `IST`, `AST`) are rejected — pass a numeric offset instead.
///
/// `Local = UTC + offset`, so the caller converts a local civil time to UT
/// with `jd_utc = jd_local - offset / 24.0`.
#[derive(Debug, Clone, Copy)]
pub struct Tz(pub f64);

impl FromStr for Tz {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        let t = s.trim();
        if t.is_empty() {
            return Err(ParseError::Tz("empty timezone".to_owned()));
        }
        let up = t.to_ascii_uppercase();

        // Bare zero-offset spellings.
        if up == "UTC" || up == "GMT" || up == "Z" || up == "UT" {
            return Ok(Tz(0.0));
        }

        // Numeric offset, optionally prefixed with UTC/GMT.
        let numeric = up
            .strip_prefix("UTC")
            .or_else(|| up.strip_prefix("GMT"))
            .unwrap_or(&up);
        if numeric.starts_with('+') || numeric.starts_with('-') {
            return parse_numeric_offset(numeric).map(Tz);
        }

        // Otherwise treat as an abbreviation; look it up in the built-in table.
        let matches: Vec<_> = celestial_core::geo::TZ_TABLE
            .iter()
            .filter(|z| z.name.eq_ignore_ascii_case(t))
            .collect();
        match matches.as_slice() {
            [] => Err(ParseError::Tz(format!(
                "unknown timezone `{s}` — use a numeric offset like `-03:00`, \
                 `UTC`, or a known abbreviation (see README timezone table)"
            ))),
            _ => {
                let offsets: Vec<f64> = matches
                    .iter()
                    .map(|z| {
                        parse_numeric_offset(&z.offset.to_ascii_uppercase().replace("UTC", ""))
                            .unwrap_or(
                                z.hours as f64
                                    + (z.minutes as f64) / 60.0
                                        * z.hours.signum().max(1) as f64,
                            )
                    })
                    .collect();
                // SEC-7: this arm only runs for a non-empty `matches`
                // and `offsets` maps 1:1, so `[0]` is safe today — but
                // make the non-empty requirement explicit rather than
                // a latent panic if that invariant ever changes.
                let Some(&first) = offsets.first() else {
                    return Err(ParseError::Tz(format!("unknown timezone `{s}`")));
                };
                if offsets.iter().any(|o| (o - first).abs() > 1e-9) {
                    let opts: Vec<String> = matches
                        .iter()
                        .map(|z| format!("{} = {} ({})", z.name, z.offset, z.desc))
                        .collect();
                    return Err(ParseError::Tz(format!(
                        "ambiguous timezone `{s}` maps to multiple offsets:\n  {}\n\
                         pass an explicit numeric offset instead, e.g. `--timezone -03:00`",
                        opts.join("\n  ")
                    )));
                }
                Ok(Tz(first))
            }
        }
    }
}

/// Parse a timezone specifier into a UTC offset in hours. See [`Tz`].
pub fn parse_tz_offset(s: &str) -> Result<f64, ParseError> {
    Tz::from_str(s).map(|t| t.0)
}

/// Parse a signed numeric offset: `+HH`, `-HH`, `+HH:MM`, `-HHMM`.
fn parse_numeric_offset(s: &str) -> Result<f64, ParseError> {
    let s = s.trim();
    let (sign, rest) = match s.as_bytes().first() {
        Some(b'+') => (1.0, &s[1..]),
        Some(b'-') => (-1.0, &s[1..]),
        _ => {
            return Err(ParseError::Tz(format!(
                "offset must start with + or -, got `{s}`"
            )))
        }
    };
    if rest.is_empty() {
        return Err(ParseError::Tz(format!("empty numeric offset `{s}`")));
    }
    let (hh, mm) = if let Some((h, m)) = rest.split_once(':') {
        (h, m)
    } else if rest.len() == 4 {
        (&rest[..2], &rest[2..]) // HHMM
    } else {
        (rest, "0")
    };
    let h: f64 = hh
        .parse()
        .map_err(|_| ParseError::Tz(format!("bad offset hours `{hh}`")))?;
    let m: f64 = mm
        .parse()
        .map_err(|_| ParseError::Tz(format!("bad offset minutes `{mm}`")))?;
    if !(0.0..=14.0).contains(&h) || !(0.0..60.0).contains(&m) {
        return Err(ParseError::Tz(format!("offset out of range `{s}`")));
    }
    Ok(sign * (h + m / 60.0))
}

/// Format a UTC offset in hours (east-positive) as `UTC`, `UTC-03:00`, …
pub fn fmt_utc_offset(offset_hours: f64) -> String {
    if offset_hours.abs() < 1e-9 {
        return "UTC".to_string();
    }
    let sign = if offset_hours < 0.0 { '-' } else { '+' };
    let total_min = (offset_hours.abs() * 60.0).round() as i32;
    format!("UTC{sign}{:02}:{:02}", total_min / 60, total_min % 60)
}

/// Ensure a date string carries an explicit time-of-day component.
/// Returns `Ok` for `now` and bare JD floats (already unambiguous in UT).
pub fn require_datetime(date_str: &str) -> Result<(), ParseError> {
    let s = date_str.trim();
    if s.eq_ignore_ascii_case("now") || s.parse::<f64>().is_ok() {
        return Ok(());
    }
    match s.split_once(' ') {
        Some((_, t)) if t.trim().contains(':') => Ok(()),
        _ => Err(ParseError::NeedsTime(format!(
            "date `{date_str}` has no time-of-day — a natal chart needs the \
             exact birth time. Pass it as `--date \"YYYY-MM-DD HH:MM\"` (or \
             add `--time HH:MM`)"
        ))),
    }
}

/// Format a Julian day as a `YYYY-MM-DD HH:MM UT` string.
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

/// A body constant parsed from a name or number.
#[derive(Debug, Clone, Copy)]
pub struct BodyId(pub i32);

impl FromStr for BodyId {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        Ok(BodyId(match s.to_lowercase().as_str() {
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
            _ => s
                .parse::<i32>()
                .map_err(|_| ParseError::Body(format!("unknown body: {s}")))?,
        }))
    }
}

/// Parse a body name or number to a body constant. See [`BodyId`].
pub fn parse_body(s: &str) -> Result<i32, ParseError> {
    BodyId::from_str(s).map(|b| b.0)
}

/// The default set of bodies for `calc`.
pub fn default_bodies() -> Vec<i32> {
    vec![
        SUN, MOON, MERCURY, VENUS, MARS, JUPITER, SATURN, URANUS, NEPTUNE,
    ]
}

/// Human-readable body name.
pub fn body_name(body: Body) -> &'static str {
    body.name()
}

// ─── House systems ────────────────────────────────────────────────────────────

/// A house-system byte code parsed from a name or letter.
#[derive(Debug, Clone, Copy)]
pub struct HouseSys(pub u8);

impl FromStr for HouseSys {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        Ok(HouseSys(match s.to_lowercase().as_str() {
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
                return Err(ParseError::HouseSys(format!(
                    "unknown house system: {s} \
            (use: placidus, koch, equal, whole, porphyry, regio, campanus, morinus)"
                )))
            }
        }))
    }
}

/// Parse a house system name or letter to its byte code. See [`HouseSys`].
pub fn parse_hsys(s: &str) -> Result<u8, ParseError> {
    HouseSys::from_str(s).map(|h| h.0)
}

/// House system byte → display name. Delegates to the canonical table on
/// [`celestial_core::body::HouseSystem`]; only the CLI-specific `'A'` alias
/// for Gauquelin is handled here (core maps `'G'` only).
pub fn hsys_name(hsys: u8) -> &'static str {
    match hsys {
        b'A' => "Gauquelin",
        b => HouseSystem(b).name(),
    }
}

// ─── Sidereal modes ───────────────────────────────────────────────────────────

/// A sidereal-mode integer code parsed from a name or number.
#[derive(Debug, Clone, Copy)]
pub struct SidMode(pub i32);

impl FromStr for SidMode {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        Ok(SidMode(match s.to_lowercase().as_str() {
            "fagan" | "fagan-bradley" | "fagan_bradley" => 0,
            "lahiri" => 1,
            "deluce" | "de-luce" => 2,
            "raman" => 3,
            "krishnamurti" => 5,
            "sassanian" => 11,
            s => s
                .parse::<i32>()
                .map_err(|_| ParseError::SidMode(format!("unknown sidereal mode: {s}")))?,
        }))
    }
}

/// Parse a sidereal mode name to its integer code. See [`SidMode`].
pub fn parse_sid_mode(s: &str) -> Result<i32, ParseError> {
    SidMode::from_str(s).map(|m| m.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_date_forms() {
        assert!(parse_date("now").is_ok());
        assert_eq!(parse_date("2451545.0").unwrap(), 2451545.0);
        assert!(parse_date("1986-05-30").is_ok());
        assert!(parse_date("1986-05-30 09:00").is_ok());
        assert!(parse_date("1986-05-30 09:00:30").is_ok());
        assert!(matches!(parse_date("not-a-date"), Err(ParseError::Date(_))));
        assert!(parse_date("1986-13-99 09:00").is_ok()); // julday is lenient; no panic
    }

    #[test]
    fn parse_tz_forms() {
        assert_eq!(parse_tz_offset("UTC").unwrap(), 0.0);
        assert_eq!(parse_tz_offset("Z").unwrap(), 0.0);
        assert_eq!(parse_tz_offset("+05:30").unwrap(), 5.5);
        assert_eq!(parse_tz_offset("-0300").unwrap(), -3.0);
        assert_eq!(parse_tz_offset("UTC+01").unwrap(), 1.0);
        assert!(parse_tz_offset("BRT").is_ok()); // known abbrev
        assert!(matches!(parse_tz_offset("ZZZ"), Err(ParseError::Tz(_))));
        assert!(parse_tz_offset("").is_err());
        assert!(parse_tz_offset("+99:99").is_err()); // out of range
        assert!(parse_tz_offset("x05").is_err()); // bad sign
    }

    #[test]
    fn fmt_utc_offset_forms() {
        assert_eq!(fmt_utc_offset(0.0), "UTC");
        assert_eq!(fmt_utc_offset(-3.0), "UTC-03:00");
        assert_eq!(fmt_utc_offset(5.5), "UTC+05:30");
    }

    #[test]
    fn require_datetime_rules() {
        assert!(require_datetime("now").is_ok());
        assert!(require_datetime("2451545.0").is_ok());
        assert!(require_datetime("1986-05-30 09:00").is_ok());
        assert!(matches!(
            require_datetime("1986-05-30"),
            Err(ParseError::NeedsTime(_))
        ));
    }

    #[test]
    fn jd_to_str_roundtrip_shape() {
        let s = jd_to_str(2446581.0);
        assert!(s.ends_with(" UT") && s.contains('-') && s.contains(':'));
    }

    #[test]
    fn parse_body_names_and_numbers() {
        for n in [
            "sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn", "uranus", "neptune",
            "pluto", "node", "mean_node", "true_node", "chiron",
        ] {
            assert!(parse_body(n).is_ok(), "{n}");
        }
        assert_eq!(parse_body("7").unwrap(), 7);
        assert!(matches!(parse_body("nope"), Err(ParseError::Body(_))));
        assert!(!default_bodies().is_empty());
        assert_eq!(body_name(celestial_core::body::Body::SUN), "Sun");
    }

    #[test]
    fn parse_hsys_names_letters_bad() {
        for n in [
            "placidus", "koch", "equal", "whole", "porphyry", "regio", "campanus", "morinus",
            "alcabitus", "axial",
        ] {
            assert!(parse_hsys(n).is_ok(), "{n}");
        }
        assert_eq!(parse_hsys("P").unwrap(), b'P');
        assert!(matches!(
            parse_hsys("unknownsystem"),
            Err(ParseError::HouseSys(_))
        ));
        assert_eq!(hsys_name(b'A'), "Gauquelin");
        assert!(!hsys_name(b'P').is_empty());
    }

    #[test]
    fn parse_sid_mode_names_num_bad() {
        for (n, v) in [
            ("fagan", 0),
            ("lahiri", 1),
            ("deluce", 2),
            ("raman", 3),
            ("krishnamurti", 5),
            ("sassanian", 11),
        ] {
            assert_eq!(parse_sid_mode(n).unwrap(), v, "{n}");
        }
        assert_eq!(parse_sid_mode("9").unwrap(), 9);
        assert!(matches!(
            parse_sid_mode("bogus"),
            Err(ParseError::SidMode(_))
        ));
    }
}
