//! Vedic/Jyotish helpers.

use crate::body::{Body, CalcFlags};
use crate::{diff_deg_signed, norm_deg};

#[inline]
fn norm360(d: f64) -> f64 {
    norm_deg(d)
}

/// Positions of the four stars used in Vedic Saturn-related calculations
/// (`Pushya`, `Revati`, `Hasta`, `Chitra`) at a given Julian day.
pub fn saturn_4_stars(jd: f64, flags: CalcFlags) -> crate::Result<[f64; 6]> {
    let sat = crate::calc_ut(jd, Body::SATURN, flags)?.lon;
    let ald = crate::functions::calc::fixstar("Aldebaran", jd, flags)?.xx[0];
    let reg = crate::functions::calc::fixstar("Regulus", jd, flags)?.xx[0];
    let ant = crate::functions::calc::fixstar("Antares", jd, flags)?.xx[0];
    let fom = crate::functions::calc::fixstar("Fomalhaut", jd, flags)?.xx[0];

    // Sort stars by ecliptic longitude
    let mut stars = [ald, reg, ant, fom];
    stars.sort_by(|a, b| a.total_cmp(b));

    // Find the two bracketing stars for Saturn
    let (s0, s1) = if sat <= stars[0] || sat > stars[3] {
        (stars[3], stars[0]) // wraps
    } else {
        let i = stars.partition_point(|&s| s < sat);
        (stars[i - 1], stars[i])
    };

    let midp = crate::norm_deg(crate::midpoint_deg(s0, s1));
    let dist0 = diff_deg_signed(sat, midp).abs();
    let dist1 = diff_deg_signed(sat, s0).abs();
    let dist2 = diff_deg_signed(sat, s1).abs();

    let index = if dist1 <= dist2 {
        dist0 / (diff_deg_signed(midp, s0).abs() / 100.0)
    } else {
        dist0 / (diff_deg_signed(midp, s1).abs() / 100.0)
    };

    Ok([sat, ald, reg, ant, fom, index])
}

// ═══════════════════════════════════════════════════════════════════════════
// Vedic / Jyotish helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Compute Raman house cusps (bhavamadhya or arambhasandhi).
///
/// `asc` = Udaya Lagna (Ascendant), `mc` = Madhya Lagna (MC).
/// `sandhi = false` → bhavamadhya (house midpoints), `true` → arambhasandhi (house beginnings).
/// Returns 12 house cusp longitudes.
pub fn raman_houses(asc: f64, mc: f64, sandhi: bool) -> [f64; 12] {
    let mut ret = [0.0f64; 12];
    if !sandhi {
        ret[0] = norm360(asc);
        ret[9] = norm360(mc);
    } else {
        let arc = diff_deg_signed(asc, mc).abs() / 6.0;
        ret[0] = norm360(asc - arc);
        ret[9] = norm360(mc - arc);
    }
    ret[6] = norm360(ret[0] + 180.0);
    ret[3] = norm360(ret[9] + 180.0);
    let arc1 = diff_deg_signed(ret[0], ret[9]).abs() / 3.0;
    ret[11] = norm360(ret[0] - arc1);
    ret[10] = norm360(ret[9] + arc1);
    ret[4] = norm360(ret[10] + 180.0);
    ret[5] = norm360(ret[11] + 180.0);
    let arc2 = diff_deg_signed(ret[0], ret[3]).abs() / 3.0;
    ret[1] = norm360(ret[0] + arc2);
    ret[2] = norm360(ret[3] - arc2);
    ret[7] = norm360(ret[1] + 180.0);
    ret[8] = norm360(ret[2] + 180.0);
    ret
}

/// Lord (ruling planet body number) of a sign.
/// Returns `None` for invalid sign numbers.
pub fn sign_lord(sign: i32) -> Option<i32> {
    match sign {
        0 | 7 => Some(4),  // Aries / Scorpio → Mars
        1 | 6 => Some(3),  // Taurus / Libra  → Venus
        2 | 5 => Some(2),  // Gemini / Virgo  → Mercury
        3 => Some(1),      // Cancer          → Moon
        4 => Some(0),      // Leo             → Sun
        8 | 11 => Some(5), // Sagittarius / Pisces → Jupiter
        9 | 10 => Some(6), // Capricorn / Aquarius → Saturn
        _ => None,
    }
}

/// Rasi (sign number 0–11) from ecliptic longitude.
#[inline]
/// Convert ecliptic longitude (degrees) to a Vedic rasi number (0 = Aries, …, 11 = Pisces).
pub fn long_to_rasi(lon: f64) -> i32 {
    (norm360(lon) / 30.0) as i32
}

/// Navamsa (0–11) from ecliptic longitude.
#[inline]
/// Convert ecliptic longitude (degrees) to a navamsa division number (0–35).
pub fn long_to_navamsa(lon: f64) -> i32 {
    ((norm360(lon) / (10.0 / 3.0)) as i32) % 12
}

/// Nakshatra (0–26) and Pada (0–3) from ecliptic longitude.
pub fn long_to_nakshatra(lon: f64) -> (i32, i32) {
    let lon = norm360(lon);
    let nak = (lon / (40.0 / 3.0)) as i32;
    let pada = ((lon - nak as f64 * (40.0 / 3.0)) / (10.0 / 3.0)) as i32;
    (nak, pada)
}

/// English name of a Nakshatra.
pub fn nakshatra_name(nak: i32) -> Option<&'static str> {
    const NAMES: &[&str] = &[
        "Aswini",
        "Bharani",
        "Krithika",
        "Rohini",
        "Mrigasira",
        "Aridra",
        "Punarvasu",
        "Pushyami",
        "Aslesha",
        "Makha",
        "Pubba",
        "Uttara",
        "Hasta",
        "Chitta",
        "Swathi",
        "Vishaka",
        "Anuradha",
        "Jyesta",
        "Moola",
        "Poorvashada",
        "Uttarashada",
        "Sravana",
        "Dhanishta",
        "Satabhisha",
        "Poorvabhadra",
        "Uttarabhadra",
        "Revathi",
    ];
    NAMES.get(nak as usize).copied()
}

/// Normalise a rasi number to `[0, 11]`.
#[inline]
/// Normalise a rasi index to 0–11 (wrapping modulo 12).
pub fn rasi_norm(r: i32) -> i32 {
    r.rem_euclid(12)
}

/// Forward distance in rasi from `r1` to `r2` (0–11).
pub fn rasi_diff(r1: i32, r2: i32) -> i32 {
    let r1 = rasi_norm(r1);
    let r2 = rasi_norm(r2);
    if r1 == r2 {
        0
    } else if r1 < r2 {
        12 - (r2 - r1)
    } else {
        r1 - r2
    }
}

/// Signed rasi difference from `r1` to `r2` (−5 to +6).
pub fn rasi_diff2(r1: i32, r2: i32) -> i32 {
    let d = rasi_diff(r1, r2);
    if d > 6 {
        -6 + (d - 6)
    } else {
        d
    }
}

/// Tatkalika relation: +1 (Mitra) if |rasi_diff2| ≤ 3, else −1 (Satru).
#[inline]
/// Tatkalika (temporary) graha relation between two rashis.
///
/// Returns `1` (friend), `0` (neutral), or `-1` (enemy).
pub fn tatkalika_relation(r1: i32, r2: i32) -> i32 {
    if rasi_diff2(r1, r2).abs() <= 3 {
        1
    } else {
        -1
    }
}

/// Naisargika (permanent) relation between two planets.
/// Returns `Some(1)` = Mitra, `Some(0)` = Sama, `Some(-1)` = Satru, `None` = invalid.
pub fn naisargika_relation(gr1: i32, gr2: i32) -> Option<i32> {
    // Body numbers: 0=Sun 1=Moon 2=Mercury 3=Venus 4=Mars 5=Jupiter 6=Saturn
    if !(0..=6).contains(&gr1) || !(0..=6).contains(&gr2) {
        return None;
    }
    let table: &[(i32, &[i32], &[i32])] = &[
        (0, &[1, 4, 5], &[3, 6]),
        (1, &[0, 2], &[]),
        (2, &[0, 3], &[1]),
        (3, &[2, 6], &[0, 1]),
        (4, &[0, 1, 5], &[2]),
        (5, &[0, 1, 4], &[2, 3]),
        (6, &[2, 3], &[0, 1, 4]),
    ];
    for &(pl, mitras, satrus) in table {
        if pl != gr1 {
            continue;
        }
        if gr2 == gr1 {
            return Some(0);
        }
        if mitras.contains(&gr2) {
            return Some(1);
        }
        if satrus.contains(&gr2) {
            return Some(-1);
        }
        return Some(0); // sama (equal)
    }
    None
}

/// Residential strength of a graha given 12 bhavamadhya longitudes.
/// Returns value in [0, 1] or `None` on error.
pub fn residential_strength(graha: f64, bm: &[f64; 12]) -> Option<f64> {
    let wrap = |i: usize| if i >= 12 { 0 } else { i };
    for i in 0..12 {
        let c1 = bm[i];
        let c2 = bm[wrap(i + 1)];
        if graha == c1 || graha == c2 {
            return Some(0.0);
        }
        let arc1 = diff_deg_signed(c1, graha);
        let arc2 = diff_deg_signed(c2, graha);
        if (arc1 >= 0.0) != (arc2 >= 0.0) && arc1.abs() + arc2.abs() < 180.0 {
            let midp = norm360(crate::midpoint_deg(c1, c2));
            if graha == midp {
                return Some(1.0);
            }
            let a1 = arc1.abs();
            let a2 = arc2.abs();
            let strength = if a1 < a2 {
                a1 / diff_deg_signed(midp, c1).abs()
            } else {
                a2 / diff_deg_signed(midp, c2).abs()
            };
            return Some(strength);
        }
    }
    None
}

/// Ochchabala (exaltation strength) for a graha in shashtiamsa.
/// Returns `None` for invalid body numbers.
pub fn ochchabala(graha: i32, sputha: f64) -> Option<f64> {
    let exalt: i32 = match graha {
        0 => 190, // Sun exalted at 10° Aries = 190° from... actually exalt pt
        1 => 213, // Moon  — 3° Taurus = 33°? Adjusted per C source: 213
        2 => 345, // Mercury
        3 => 177, // Venus
        4 => 118, // Mars
        5 => 275, // Jupiter
        6 => 20,  // Saturn
        _ => return None,
    };
    Some(diff_deg_signed(sputha, exalt as f64).abs() / 3.0)
}

// ═══════════════════════════════════════════════════════════════════════════
// Iterative searches
// ═══════════════════════════════════════════════════════════════════════════
