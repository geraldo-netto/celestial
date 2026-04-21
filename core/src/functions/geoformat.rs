//! Geographic coordinate and format helpers (port of swhformat.c + swhgeo.c).

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn norm360(d: f64) -> f64 {
    d.rem_euclid(360.0)
}
#[allow(dead_code)]
fn diff_deg_signed(p1: f64, p2: f64) -> f64 {
    let d = (p2 - p1).rem_euclid(360.0);
    if d > 180.0 {
        d - 360.0
    } else {
        d
    }
}
#[allow(dead_code)]
fn diff_deg(p1: f64, p2: f64) -> f64 {
    (p2 - p1).rem_euclid(360.0)
}

/// Split a decimal degree value into `[degrees, minutes, seconds, centiseconds]`.
pub fn degsplit(pos: f64) -> [i32; 4] {
    let mut p = norm360(pos);
    let sign = (p / 30.0) as i32;
    p -= sign as f64 * 30.0;
    let deg = p as i32;
    p -= deg as f64;
    let min = (p * 60.0) as i32;
    p -= min as f64 / 60.0;
    let sec = (p * 3600.0) as i32;
    [deg, sign, min, sec]
}

/// English name of a zodiac sign (0 = Aries … 11 = Pisces).
pub fn sign_name(sign: i32) -> Option<&'static str> {
    match sign {
        0 => Some("Aries"),
        1 => Some("Taurus"),
        2 => Some("Gemini"),
        3 => Some("Cancer"),
        4 => Some("Leo"),
        5 => Some("Virgo"),
        6 => Some("Libra"),
        7 => Some("Scorpio"),
        8 => Some("Sagittarius"),
        9 => Some("Capricorn"),
        10 => Some("Aquarius"),
        11 => Some("Pisces"),
        _ => None,
    }
}

/// Integer id for a house-system char.
/// Returns `None` for unknown characters.
pub fn house_system_id(hsys: u8) -> Option<i32> {
    match hsys {
        b'P' => Some(0),
        b'K' => Some(1),
        b'R' => Some(2),
        b'C' => Some(3),
        b'B' => Some(4),
        b'M' => Some(5),
        b'O' => Some(6),
        b'A' | b'E' => Some(7),
        b'H' => Some(8),
        b'V' => Some(9),
        b'X' => Some(10),
        b'G' => Some(11),
        b'T' => Some(12),
        b'U' => Some(13),
        b'W' => Some(14),
        b'Y' => Some(15),
        _ => None,
    }
}

/// House-system char from integer id.
pub fn house_system_char(id: i32) -> Option<u8> {
    match id {
        0 => Some(b'P'),
        1 => Some(b'K'),
        2 => Some(b'R'),
        3 => Some(b'C'),
        4 => Some(b'B'),
        5 => Some(b'M'),
        6 => Some(b'O'),
        7 => Some(b'A'),
        8 => Some(b'H'),
        9 => Some(b'V'),
        10 => Some(b'X'),
        11 => Some(b'G'),
        12 => Some(b'T'),
        13 => Some(b'U'),
        14 => Some(b'W'),
        15 => Some(b'Y'),
        _ => None,
    }
}

/// Map sidereal-mode index (1–21 = SE modes) to celestial flag value.
/// Index 0 = tropical (returns 256), index 22 = user-defined (returns 255).
pub fn sidereal_mode_flag(sidmode: i32) -> Option<i32> {
    match sidmode {
        0 => Some(256),
        22 => Some(255),
        1..=21 => Some(sidmode - 1),
        _ => None,
    }
}

/// Reverse of `sidereal_mode_flag`.
pub fn sidereal_mode_id(flag: i32) -> Option<i32> {
    match flag {
        256 => Some(0),
        255 => Some(22),
        0..=20 => Some(flag + 1),
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Geographic coordinate parsing & formatting
// ═══════════════════════════════════════════════════════════════════════════

/// Parse a geographic coordinate string to a decimal degree value.
///
/// Accepted formats (D=degrees, M=minutes, S=seconds, x=N/S/E/W):
/// `DMSx`, `DxMS`, `DMx`, `DxM`, `DMS`, `Dx`, `DM`, `D`
/// Decorators like `°'":/,` are treated as separators.
pub fn parse_coord(s: &str) -> Option<f64> {
    let s = s.trim();

    // Detect leading negative sign (before any digit or direction)
    let (s, leading_neg) = if let Some(stripped) = s.strip_prefix('-') {
        (stripped, true)
    } else {
        (s, false)
    };

    // Detect direction letter anywhere in the string
    let dir: Option<char> = s
        .chars()
        .find(|c| matches!(c, 'N' | 'n' | 'S' | 's' | 'E' | 'e' | 'W' | 'w'))
        .map(|c| c.to_ascii_lowercase());

    // Strip non-numeric, non-decimal chars → collect digit/dot groups
    let cleaned: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_digit() || c == '.' {
                c
            } else {
                ' '
            }
        })
        .collect();
    let nums: Vec<f64> = cleaned
        .split_whitespace()
        .filter_map(|t| t.parse::<f64>().ok())
        .collect();
    if nums.is_empty() {
        return None;
    }

    let deg = nums[0];
    let min = nums.get(1).copied().unwrap_or(0.0);
    let sec = nums.get(2).copied().unwrap_or(0.0);

    // Validate ranges
    let max_deg = match dir {
        Some('n') | Some('s') => 90.0,
        _ => 180.0,
    };
    if deg > max_deg || min >= 60.0 || sec >= 60.0 {
        return None;
    }

    let val = deg + min / 60.0 + sec / 3600.0;
    let negative = match dir {
        Some('s') | Some('w') => true,
        None => leading_neg,
        _ => false,
    };
    Some(if negative { -val } else { val })
}

/// Decompose a geographic coordinate into `[degrees, minutes, seconds]` (all positive).
pub fn geo_to_dms(coord: f64) -> [i32; 3] {
    let c = coord.abs();
    let deg = c as i32;
    let rem = c - deg as f64;
    let min = (rem * 60.0).round() as i32;
    let rem2 = rem - min as f64 / 60.0;
    let sec = (rem2 * 3600.0).round() as i32;
    [deg, min, sec.max(0)]
}

/// Format a geographic coordinate as `"DD:N|S:MM:SS"` (latitude) or `"DDD:E|W:MM:SS"` (longitude).
pub fn format_coord(coord: f64, is_latitude: bool) -> Option<String> {
    let max = if is_latitude { 90.0 } else { 180.0 };
    if coord.abs() > max {
        return None;
    }
    let [d, m, s] = geo_to_dms(coord);
    let dir = if is_latitude {
        if coord >= 0.0 {
            "N"
        } else {
            "S"
        }
    } else {
        if coord >= 0.0 {
            "E"
        } else {
            "W"
        }
    };
    Some(if is_latitude {
        format!("{:02}:{}:{:02}:{:02}", d, dir, m, s)
    } else {
        format!("{:03}:{}:{:02}:{:02}", d, dir, m, s)
    })
}

// ─── Centisecond-based string formatters (C API compat) ─────────────────────────

/// Format centiseconds as a degree string.
pub fn centisec_to_deg_str(t: i32) -> String {
    let d = t / 360_000;
    let rest = t.abs() % 360_000;
    let m = rest / 6_000;
    let s = (rest % 6_000) / 100;
    format!("{d:3}°{m:02}'{s:02}\"")
}

/// Format centiseconds as a longitude/latitude string.
pub fn centisec_to_lonlat_str(t: i32, pos_char: char, neg_char: char) -> String {
    let sign = if t >= 0 { pos_char } else { neg_char };
    let abs = t.unsigned_abs();
    let d = abs / 360_000;
    let m = (abs % 360_000) / 6_000;
    let s = (abs % 6_000) / 100;
    format!("{d:3}{sign}{m:02}'{s:02}\"")
}

/// Format centiseconds as a time string.
pub fn centisec_to_time_str(t: i32, sep: char, suppress_zero: bool) -> String {
    let abs = t.unsigned_abs();
    let h = abs / 360_000;
    let m = (abs % 360_000) / 6_000;
    let s = (abs % 6_000) / 100;
    if suppress_zero && h == 0 {
        format!("{m:02}{sep}{s:02}")
    } else {
        format!("{h:02}{sep}{m:02}{sep}{s:02}")
    }
}
