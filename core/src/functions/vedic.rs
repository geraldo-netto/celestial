//! Vedic/Jyotish helpers.

use crate::body::{Body, CalcFlags};
use crate::units::{Degrees, JulianDay, Longitude};
use crate::{diff_deg_signed, norm_deg};

#[inline]
fn norm360(d: f64) -> f64 {
    norm_deg(d)
}

fn opposite(d: f64) -> f64 {
    norm360(d + 180.0)
}

fn saturn_star_index(sat: f64, mut stars: [f64; 4]) -> f64 {
    stars.sort_by(f64::total_cmp);
    let i = stars
        .binary_search_by(|star| star.total_cmp(&sat))
        .unwrap_or_else(std::convert::identity);
    let s0 = stars[(i + stars.len() - 1) % stars.len()];
    let s1 = stars[i % stars.len()];
    let midp = norm360(crate::midpoint_deg(s0, s1));
    let dist0 = diff_deg_signed(sat, midp).abs();
    let scale = diff_deg_signed(midp, s0).abs();
    dist0 / (scale / 100.0)
}

/// Positions of the four stars used in Vedic Saturn-related calculations
/// (`Pushya`, `Revati`, `Hasta`, `Chitra`) at a given Julian day.
pub fn saturn_4_stars(jd: JulianDay, flags: CalcFlags) -> crate::Result<[f64; 6]> {
    let jd: f64 = jd.into();
    let sat = crate::calc_ut(JulianDay::new(jd), Body::SATURN, flags)?.lon;
    let ald = crate::functions::calc::fixstar("Aldebaran", JulianDay::new(jd), flags)?.xx[0];
    let reg = crate::functions::calc::fixstar("Regulus", JulianDay::new(jd), flags)?.xx[0];
    let ant = crate::functions::calc::fixstar("Antares", JulianDay::new(jd), flags)?.xx[0];
    let fom = crate::functions::calc::fixstar("Fomalhaut", JulianDay::new(jd), flags)?.xx[0];

    let index = saturn_star_index(sat, [ald, reg, ant, fom]);

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
#[must_use]
pub fn raman_houses(asc: Degrees, mc: Degrees, sandhi: bool) -> [f64; 12] {
    let asc: f64 = asc.into();
    let mc: f64 = mc.into();
    let mut ret = [0.0f64; 12];
    if !sandhi {
        ret[0] = norm360(asc);
        ret[9] = norm360(mc);
    } else {
        let arc = diff_deg_signed(asc, mc).abs() / 6.0;
        ret[0] = norm360(asc - arc);
        ret[9] = norm360(mc - arc);
    }
    ret[6] = opposite(ret[0]);
    ret[3] = opposite(ret[9]);
    let arc1 = diff_deg_signed(ret[0], ret[9]).abs() / 3.0;
    ret[11] = norm360(ret[0] - arc1);
    ret[10] = norm360(ret[9] + arc1);
    ret[4] = opposite(ret[10]);
    ret[5] = opposite(ret[11]);
    let arc2 = diff_deg_signed(ret[0], ret[3]).abs() / 3.0;
    ret[1] = norm360(ret[0] + arc2);
    ret[2] = norm360(ret[3] - arc2);
    ret[7] = opposite(ret[1]);
    ret[8] = opposite(ret[2]);
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
#[must_use]
pub fn long_to_rasi(lon: Longitude) -> i32 {
    let lon: f64 = lon.into();
    (norm360(lon) / 30.0) as i32
}

/// Navamsa (0–11) from ecliptic longitude.
#[inline]
/// Convert ecliptic longitude (degrees) to a navamsa division number (0–35).
#[must_use]
pub fn long_to_navamsa(lon: Longitude) -> i32 {
    let lon: f64 = lon.into();
    ((norm360(lon) / (10.0 / 3.0)) as i32) % 12
}

/// Nakshatra (0–26) and Pada (0–3) from ecliptic longitude.
#[must_use]
pub fn long_to_nakshatra(lon: Longitude) -> (i32, i32) {
    let lon: f64 = lon.into();
    let lon = norm360(lon);
    let nak = (lon / (40.0 / 3.0)) as i32;
    let pada = ((-(nak as f64)).mul_add(40.0 / 3.0, lon) / (10.0 / 3.0)) as i32;
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
#[must_use]
pub fn rasi_norm(r: i32) -> i32 {
    r.rem_euclid(12)
}

/// Forward distance in rasi from `r1` to `r2` (0–11).
#[must_use]
pub fn rasi_diff(r1: i32, r2: i32) -> i32 {
    let r1 = rasi_norm(r1);
    let r2 = rasi_norm(r2);
    match r1.cmp(&r2) {
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Less => 12 - (r2 - r1),
        std::cmp::Ordering::Greater => r1 - r2,
    }
}

/// Signed rasi difference from `r1` to `r2` (−5 to +6).
#[must_use]
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
#[must_use]
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
/// Strength contribution within a single bhava `[c1, c2]`. `None` means the
/// graha lies outside this house; `Some(s)` returns the residential strength.
fn residential_within(graha: f64, c1: f64, c2: f64) -> Option<f64> {
    if graha == c1 || graha == c2 {
        return Some(0.0);
    }
    let arc1 = diff_deg_signed(c1, graha);
    let arc2 = diff_deg_signed(c2, graha);
    let inside = (arc1 >= 0.0) != (arc2 >= 0.0) && arc1.abs() + arc2.abs() < 180.0;
    if !inside {
        return None;
    }
    let midp = norm360(crate::midpoint_deg(c1, c2));
    if graha == midp {
        return Some(1.0);
    }
    let a1 = arc1.abs();
    let a2 = arc2.abs();
    let strength = a1.min(a2) / diff_deg_signed(midp, c1).abs();
    Some(strength)
}

/// Residential strength (bhava-bala) of a graha given the 12 bhavamadhya
/// longitudes. Returns the in-house strength in [0, 1], or `None` if the graha
/// falls in no house.
pub fn residential_strength(graha: f64, bm: &[f64; 12]) -> Option<f64> {
    let wrap = |i: usize| if i >= 12 { 0 } else { i };
    for i in 0..12 {
        if let Some(s) = residential_within(graha, bm[i], bm[wrap(i + 1)]) {
            return Some(s);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-12;
    const HOUSES: [f64; 12] = [
        0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0, 210.0, 240.0, 270.0, 300.0, 330.0,
    ];

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < EPSILON,
            "{actual} != {expected}"
        );
    }

    fn assert_houses(actual: [f64; 12], expected: [f64; 12]) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert_close(actual, expected);
        }
    }

    #[test]
    fn saturn_index_covers_each_bracket_path() {
        let stars = [300.0, 10.0, 200.0, 100.0];
        let cases = [
            (10.0, 100.0),
            (50.0, 100.0 / 9.0),
            (90.0, 700.0 / 9.0),
            (300.0, 100.0),
            (350.0, 300.0 / 7.0),
        ];
        for (sat, expected) in cases {
            assert_close(saturn_star_index(sat, stars), expected);
        }
    }

    #[test]
    fn saturn_four_stars_regression() {
        let actual = saturn_4_stars(JulianDay::new(2_452_275.5), CalcFlags::BUILTIN).unwrap();
        let expected = [
            68.97365443719134,
            69.81695132105718,
            149.85705791063336,
            249.79030935491775,
            333.8886761406571,
            98.2418178951306,
        ];
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert_close(actual, expected);
        }
    }

    #[test]
    fn raman_house_modes_are_exact() {
        assert_houses(
            raman_houses(Degrees::new(15.0), Degrees::new(275.0), false),
            [
                15.0,
                125.0 / 3.0,
                205.0 / 3.0,
                95.0,
                385.0 / 3.0,
                485.0 / 3.0,
                195.0,
                665.0 / 3.0,
                745.0 / 3.0,
                275.0,
                925.0 / 3.0,
                1025.0 / 3.0,
            ],
        );
        assert_houses(
            raman_houses(Degrees::new(15.0), Degrees::new(275.0), true),
            [
                1075.0 / 3.0,
                25.0,
                155.0 / 3.0,
                235.0 / 3.0,
                335.0 / 3.0,
                145.0,
                535.0 / 3.0,
                205.0,
                695.0 / 3.0,
                775.0 / 3.0,
                875.0 / 3.0,
                325.0,
            ],
        );
    }

    #[test]
    fn sign_and_lunar_divisions_cover_boundaries() {
        let lords = [4, 3, 2, 1, 0, 2, 3, 4, 5, 6, 6, 5];
        for (sign, lord) in lords.into_iter().enumerate() {
            assert_eq!(sign_lord(sign as i32), Some(lord));
        }
        assert_eq!(sign_lord(-1), None);
        assert_eq!(sign_lord(12), None);
        assert_eq!(long_to_rasi(Longitude::new(-1.0)), 11);
        assert_eq!(long_to_rasi(Longitude::new(45.0)), 1);
        assert_eq!(long_to_navamsa(Longitude::new(3.5)), 1);
        assert_eq!(long_to_navamsa(Longitude::new(40.0)), 0);
        assert_eq!(long_to_nakshatra(Longitude::new(17.0)), (1, 1));
        assert_eq!(long_to_nakshatra(Longitude::new(-1.0)), (26, 3));
    }

    #[test]
    fn nakshatra_names_cover_valid_and_invalid_indices() {
        assert_eq!(nakshatra_name(0), Some("Aswini"));
        assert_eq!(nakshatra_name(13), Some("Chitta"));
        assert_eq!(nakshatra_name(26), Some("Revathi"));
        assert_eq!(nakshatra_name(-1), None);
        assert_eq!(nakshatra_name(27), None);
    }

    #[test]
    fn rasi_relations_cover_wrap_and_thresholds() {
        assert_eq!(rasi_norm(-1), 11);
        assert_eq!(rasi_norm(13), 1);
        assert_eq!(rasi_diff(0, 0), 0);
        assert_eq!(rasi_diff(1, 4), 9);
        assert_eq!(rasi_diff(4, 1), 3);
        let signed = [0, 1, 2, 3, 4, 5, 6, -5, -4, -3, -2, -1];
        for (rasi, expected) in signed.into_iter().enumerate() {
            assert_eq!(rasi_diff2(rasi as i32, 0), expected);
        }
        assert_eq!(tatkalika_relation(4, 1), 1);
        assert_eq!(tatkalika_relation(5, 1), -1);
    }

    #[test]
    fn naisargika_matrix_and_validation_are_exact() {
        let expected = [
            [0, 1, 0, -1, 1, 1, -1],
            [1, 0, 1, 0, 0, 0, 0],
            [1, -1, 0, 1, 0, 0, 0],
            [-1, -1, 1, 0, 0, 0, 1],
            [1, 1, -1, 0, 0, 1, 0],
            [1, 1, -1, -1, 1, 0, 0],
            [-1, -1, 1, 1, -1, 0, 0],
        ];
        for (first, row) in expected.into_iter().enumerate() {
            for (second, relation) in row.into_iter().enumerate() {
                assert_eq!(
                    naisargika_relation(first as i32, second as i32),
                    Some(relation)
                );
            }
        }
        for pair in [(-1, 0), (0, -1), (7, 0), (0, 7)] {
            assert_eq!(naisargika_relation(pair.0, pair.1), None);
        }
    }

    #[test]
    fn residential_strength_covers_boundaries_and_wrap() {
        assert_eq!(residential_within(0.0, 0.0, 30.0), Some(0.0));
        assert_eq!(residential_within(30.0, 0.0, 30.0), Some(0.0));
        assert_eq!(residential_within(15.0, 0.0, 30.0), Some(1.0));
        assert_close(residential_within(5.0, 0.0, 30.0).unwrap(), 1.0 / 3.0);
        assert_close(residential_within(25.0, 0.0, 30.0).unwrap(), 1.0 / 3.0);
        assert_eq!(residential_within(45.0, 0.0, 30.0), None);
        assert_eq!(residential_within(90.0, 0.0, 180.0), None);
        assert_eq!(residential_within(100.0, 0.0, 200.0), None);
        assert_close(residential_strength(5.0, &HOUSES).unwrap(), 1.0 / 3.0);
        assert_eq!(residential_strength(10.0, &[0.0; 12]), None);
    }

    #[test]
    fn ochchabala_covers_every_planet_and_scaling() {
        let exaltations = [190.0, 213.0, 345.0, 177.0, 118.0, 275.0, 20.0];
        for (graha, exaltation) in exaltations.into_iter().enumerate() {
            assert_eq!(ochchabala(graha as i32, exaltation), Some(0.0));
        }
        assert_eq!(ochchabala(0, 193.0), Some(1.0));
        assert_eq!(ochchabala(-1, 0.0), None);
        assert_eq!(ochchabala(7, 0.0), None);
    }
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
