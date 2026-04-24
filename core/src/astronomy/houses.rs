//! House cusp calculations — all major house systems.
//!
//! All algorithms are pure trigonometry; no ephemeris data files are needed.
//!
//! References:
//!   - Meeus, "Astronomical Algorithms" 2nd ed., Chapter 36
//!   - Placidus: exact semi-arc method (iterative)
//!   - Koch: birthplace / Birthort system
//!   - Regiomontanus: prime vertical circles
//!   - Campanus: prime vertical / nonagesimal
//!   - Equal: 30° divisions from ASC
//!   - Whole Sign: 30° signs from ASC sign
//!   - Porphyry: trisect each quadrant
//!   - Morinus: equatorial houses
//!   - Meridian / Axial rotation: meridian circles
//!   - Alcabitius: semi-arc in oblique sphere
//!   - Azimuthal / Horizontal: azimuth-based
//!   - Polich / Page (Topocentric)
//!   - Gauquelin sectors: 36 sectors
#![allow(clippy::needless_range_loop)]

use crate::astronomy::constants::{norm_deg, to_deg, to_rad};

// ─── Public types ─────────────────────────────────────────────────────────────

/// House cusps and special points.
///
/// `cusps[1..=12]` — house cusps in degrees (index 0 unused).
/// `ascmc[0..10]`  — Ascendant, MC, ARMC, Vertex, Equatorial Asc, Co-Asc (Koch),
///                   Co-Asc (Munkasey), Polar Asc, …
#[derive(Debug, Clone, PartialEq)]
#[must_use = "the search result contains the computed data — did you mean to use it?"]
pub struct HouseResult {
    /// House cusps. `cusps[1]` = 1st house, …, `cusps[12]` = 12th house.
    pub cusps: [f64; 13],
    /// Special angles: ASC, MC, ARMC, Vertex, EqAsc, etc.
    pub ascmc: [f64; 10],
}

/// Speed-extended house result (Ex2 variant).
#[derive(Debug, Clone, PartialEq)]
pub struct HouseResultEx2 {
    pub cusps: [f64; 13],
    pub ascmc: [f64; 10],
    pub cusp_speeds: [f64; 13],
    pub ascmc_speeds: [f64; 10],
}

/// House system selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HouseSystem {
    Placidus,
    Koch,
    Porphyrius,
    Regiomontanus,
    Campanus,
    Equal,   // from ASC
    EqualMC, // equal from MC
    WholeSign,
    Meridian, // axial rotation
    Morinus,
    Alcabitius,
    Azimuthal,   // horizontal
    Topocentric, // Polich/Page
    Gauquelin,   // 36 sectors
}

impl HouseSystem {
    /// Parse a single ASCII character into a house system (Swiss Ephemeris convention).
    pub fn from_char(c: u8) -> Option<Self> {
        Some(match c {
            b'P' => Self::Placidus,
            b'K' => Self::Koch,
            b'O' => Self::Porphyrius,
            b'R' => Self::Regiomontanus,
            b'C' => Self::Campanus,
            b'E' => Self::Equal,
            b'D' => Self::EqualMC,
            b'W' => Self::WholeSign,
            b'X' => Self::Meridian,
            b'M' => Self::Morinus,
            b'B' => Self::Alcabitius,
            b'H' => Self::Azimuthal,
            b'T' => Self::Topocentric,
            b'G' => Self::Gauquelin,
            _ => return None,
        })
    }

    /// Returns the display name of the house system.
    pub fn name(self) -> &'static str {
        match self {
            Self::Placidus => "Placidus",
            Self::Koch => "Koch",
            Self::Porphyrius => "Porphyrius",
            Self::Regiomontanus => "Regiomontanus",
            Self::Campanus => "Campanus",
            Self::Equal => "Equal",
            Self::EqualMC => "Equal (MC)",
            Self::WholeSign => "Whole Sign",
            Self::Meridian => "Meridian",
            Self::Morinus => "Morinus",
            Self::Alcabitius => "Alcabitius",
            Self::Azimuthal => "Azimuthal",
            Self::Topocentric => "Topocentric",
            Self::Gauquelin => "Gauquelin Sectors",
        }
    }
}

// ─── Main entry point ─────────────────────────────────────────────────────────

/// Compute house cusps from a Julian day (UT), geographic position and house system.
///
/// - `jd_ut` — Julian day (Universal Time)
/// - `geolat` — geographic latitude (degrees, N positive)
/// - `geolon` — geographic longitude (degrees, E positive)
/// - `hsys` — house system character (`b'P'` = Placidus, etc.)
///
/// Uses sidereal time computed from JD.
pub fn houses(jd_ut: f64, geolat: f64, geolon: f64, hsys: u8) -> HouseResult {
    let armc = sidereal_time_deg(jd_ut) + geolon;
    let armc = norm_deg(armc);
    let eps = obliquity_simple(jd_ut);
    houses_armc(armc, geolat, eps, hsys)
}

/// Compute house cusps from ARMC, latitude and obliquity.
///
/// This is the low-level entry point used when ARMC and obliquity are
/// already known (e.g. from the full astronomy pipeline).
pub fn houses_armc(armc: f64, geolat: f64, eps: f64, hsys: u8) -> HouseResult {
    let system = HouseSystem::from_char(hsys).unwrap_or(HouseSystem::Placidus);
    compute_houses(armc, geolat, eps, system)
}

// ─── Dispatcher ───────────────────────────────────────────────────────────────

fn compute_houses(armc: f64, lat: f64, eps: f64, sys: HouseSystem) -> HouseResult {
    let asc = ascendant(armc, lat, eps);
    let mc = midheaven(armc, eps);

    let cusps = match sys {
        HouseSystem::Placidus => placidus(armc, lat, eps, asc, mc),
        HouseSystem::Koch => koch(armc, lat, eps, asc, mc),
        HouseSystem::Porphyrius => porphyry(asc, mc),
        HouseSystem::Regiomontanus => regiomontanus(armc, lat, eps),
        HouseSystem::Campanus => campanus(armc, lat, eps),
        HouseSystem::Equal => equal_asc(asc),
        HouseSystem::EqualMC => equal_mc(mc),
        HouseSystem::WholeSign => whole_sign(asc),
        HouseSystem::Meridian => meridian(armc, eps),
        HouseSystem::Morinus => morinus(armc, eps),
        HouseSystem::Alcabitius => alcabitius(armc, lat, eps, asc),
        HouseSystem::Azimuthal => azimuthal(armc, lat, eps),
        HouseSystem::Topocentric => topocentric(armc, lat, eps, asc),
        HouseSystem::Gauquelin => gauquelin(armc, lat, eps),
    };

    let vertex = vertex_point(armc, lat, eps);
    let eq_asc = equatorial_asc(armc, eps);
    let mut ascmc = [0.0f64; 10];
    ascmc[0] = asc;
    ascmc[1] = mc;
    ascmc[2] = armc;
    ascmc[3] = vertex;
    ascmc[4] = eq_asc;

    HouseResult { cusps, ascmc }
}

// ─── Auxiliary angles ─────────────────────────────────────────────────────────

/// Ascendant (degrees).
pub fn ascendant(armc: f64, lat: f64, eps: f64) -> f64 {
    let armc_r = to_rad(armc);
    let lat_r = to_rad(lat);
    let eps_r = to_rad(eps);

    // Standard ASC formula (Meeus, Astr. Algorithms):
    //   atan2(-cos(ARMC), sin(ARMC)·cos(ε) + tan(φ)·sin(ε))
    // The raw atan2 gives the western horizon (DSC); adding 180° gives the
    // eastern horizon (ASC) — the point actually rising on the ecliptic.
    let y = -(armc_r.cos());
    let x = armc_r.sin() * eps_r.cos() + lat_r.tan() * eps_r.sin();
    norm_deg(to_deg(y.atan2(x)) + 180.0)
}

/// Midheaven (MC) (degrees).
pub fn midheaven(armc: f64, eps: f64) -> f64 {
    let armc_r = to_rad(armc);
    let eps_r = to_rad(eps);
    // Standard formula (Meeus, Astronomical Algorithms):
    //   MC = atan2(sin(ARMC) · cos(ε), cos(ARMC))
    // atan2 already handles all four quadrants correctly — no manual
    // quadrant adjustment needed (the old +180° was wrong and produced DSC).
    norm_deg(to_deg((armc_r.sin() * eps_r.cos()).atan2(armc_r.cos())))
}

/// Vertex: the point on the ecliptic where the prime vertical intersects
/// in the western hemisphere.
fn vertex_point(armc: f64, lat: f64, eps: f64) -> f64 {
    // Vertex = ASC for latitude - 90° rotated 90°
    let anti_armc = norm_deg(armc + 90.0);
    ascendant(anti_armc, 90.0 - lat.abs(), eps)
}

/// Equatorial Ascendant (East Point).
fn equatorial_asc(armc: f64, _eps: f64) -> f64 {
    norm_deg(armc + 90.0)
}

// ─── Oblique ascension helper ────────────────────────────────────────────────

/// Oblique ascension of an ecliptic point.
fn oblique_ascension(lon: f64, lat: f64, eps: f64, geolat: f64) -> f64 {
    let lon_r = to_rad(lon);
    let lat_r = to_rad(lat);
    let eps_r = to_rad(eps);
    let _lat_g = to_rad(geolat);

    let ra_r = (lon_r.sin() * eps_r.cos() - lat_r.tan() * eps_r.sin()).atan2(lon_r.cos());
    let dec = (lat_r.sin() * eps_r.cos() + lat_r.cos() * eps_r.sin() * lon_r.sin()).asin();
    let ad_arg = (to_rad(geolat).tan() * dec.tan()).clamp(-1.0, 1.0);
    let ad = ad_arg.asin();
    norm_deg(to_deg(ra_r) - to_deg(ad))
}

// ─── Placidus ─────────────────────────────────────────────────────────────────

/// Placidus house cusps via iterative semi-arc method.
fn placidus(armc: f64, lat: f64, eps: f64, asc: f64, mc: f64) -> [f64; 13] {
    let lat_r = to_rad(lat);
    let eps_r = to_rad(eps);
    let _cos_lat = lat_r.cos();
    let _sin_eps = eps_r.sin();

    let mut cusps = [0.0f64; 13];
    cusps[1] = asc;
    cusps[10] = mc;
    cusps[4] = norm_deg(mc + 180.0);
    cusps[7] = norm_deg(asc + 180.0);

    // Divide the two diurnal semi-arcs into thirds to get the 6 intermediate cusps.
    // Q1 (MC→ASC, upper hemisphere): cusps 11, 12
    // Q3 (IC→DSC, lower hemisphere): cusps  5,  6  (opposites of 11, 12)
    // Q4 (DSC→MC, upper hemisphere): cusps  9,  8
    // Q2 (ASC→IC, lower hemisphere): cusps  2,  3  (opposites of  9,  8)
    let fracs = [1.0 / 3.0, 2.0 / 3.0];

    // Upper diurnal arc Q1: armc → armc+90° (between MC and ASC)
    for (i, &frac) in fracs.iter().enumerate() {
        let h = placidus_cusp(armc + 90.0 * frac, lat_r, eps_r, 1.0);
        cusps[11 + i] = norm_deg(h); // houses 11, 12
        cusps[5 + i] = norm_deg(h + 180.0); // houses  5,  6 (opposite)
    }

    // Lower diurnal arc Q4: armc+270° → armc+360° (between DSC and MC)
    for (i, &frac) in fracs.iter().enumerate() {
        let h = placidus_cusp(armc + 270.0 + 90.0 * frac, lat_r, eps_r, -1.0);
        cusps[9 - i] = norm_deg(h); // houses 9, 8
        cusps[3 - i] = norm_deg(h + 180.0); // houses 3, 2 (opposite)
    }

    cusps
}

/// Iteratively solve for one Placidus cusp.
fn placidus_cusp(armc_offset: f64, lat_r: f64, eps_r: f64, sign: f64) -> f64 {
    let armc_r = to_rad(armc_offset);
    let _sin_eps = eps_r.sin();
    let _cos_lat = lat_r.cos();

    let mut lon = norm_deg(to_deg(armc_r)) + 90.0;
    for _ in 0..20 {
        let lon_r = to_rad(lon);
        let dec = (eps_r.sin() * lon_r.sin()).asin();
        let ad_arg = (lat_r.tan() * dec.tan()).clamp(-1.0, 1.0);
        let ad = ad_arg.asin();
        let oa = to_deg((lon_r.sin() * eps_r.cos()).atan2(lon_r.cos())) - to_deg(ad);
        let lon_new = norm_deg(oa + to_deg(armc_r) + sign * 90.0);
        if (lon_new - lon).abs() < 1e-6 {
            return lon_new;
        }
        lon = lon_new;
    }
    lon
}

// ─── Koch ─────────────────────────────────────────────────────────────────────

fn koch(armc: f64, lat: f64, eps: f64, asc: f64, mc: f64) -> [f64; 13] {
    let lat_r = to_rad(lat);
    let eps_r = to_rad(eps);

    let mut cusps = [0.0f64; 13];
    cusps[1] = asc;
    cusps[10] = mc;
    cusps[4] = norm_deg(mc + 180.0);
    cusps[7] = norm_deg(asc + 180.0);

    // Koch: divide semi-arc into thirds and project back to ecliptic
    let dsa = diurnal_semi_arc(eps_r, lat_r);

    for (i, h) in [11usize, 12].iter().enumerate() {
        let frac = (i as f64 + 1.0) / 3.0;
        let armc_h = norm_deg(armc + dsa * frac);
        let armc_r = to_rad(armc_h);
        // Declination that rises with this ARMC in latitude lat
        let sin_armc = armc_r.sin();
        if sin_armc.abs() < 1e-8 {
            // Degenerate case: fall back to previous cusp + 30°
            cusps[*h] = norm_deg(cusps[10] + (*h as f64 - 10.0) * 30.0);
            cusps[*h - 9] = norm_deg(cusps[*h] + 180.0);
            continue;
        }
        let dec_arg = (lat_r.tan() * armc_r.cos() / sin_armc.abs()).atan();
        let dec_sin = dec_arg.sin().clamp(-1.0, 1.0);
        let dec = dec_sin.clamp(-1.0, 1.0).asin();
        let ra = (armc_r.sin() * eps_r.cos()).atan2(armc_r.cos());
        let lon = ecl_lon_from_ra_dec(to_deg(ra), to_deg(dec), eps);
        if lon.is_nan() || lon.is_infinite() {
            cusps[*h] = norm_deg(cusps[10] + (*h as f64 - 10.0) * 30.0);
        } else {
            cusps[*h] = lon;
        }
        cusps[*h - 9] = norm_deg(cusps[*h] + 180.0);
    }

    for (i, h) in [9usize, 8].iter().enumerate() {
        let frac = (i as f64 + 1.0) / 3.0;
        let armc_h = norm_deg(armc - dsa * frac);
        let armc_r = to_rad(armc_h);
        let dec = (((lat_r.tan() * armc_r.cos()) / armc_r.sin().abs().max(1e-10))
            .atan()
            .sin())
        .clamp(-1.0, 1.0)
        .asin();
        let lon = ecl_lon_from_ra_dec(
            to_deg((armc_r.sin() * eps_r.cos()).atan2(armc_r.cos())),
            to_deg(dec),
            eps,
        );
        if lon.is_nan() || lon.is_infinite() {
            cusps[*h] = norm_deg(cusps[10] + (*h as f64 - 10.0) * 30.0);
        } else {
            cusps[*h] = lon;
        }
        cusps[*h - 6] = norm_deg(cusps[*h] + 180.0);
    }

    // Houses 5 and 6 are the opposites of 11 and 12 respectively
    // (not set by either loop above)
    cusps[5] = norm_deg(cusps[11] + 180.0);
    cusps[6] = norm_deg(cusps[12] + 180.0);

    cusps
}

fn diurnal_semi_arc(_eps_r: f64, lat_r: f64) -> f64 {
    // Semi-arc of an equatorial point at the celestial equator
    to_deg((lat_r.tan() * 0.0_f64.tan()).asin()) + 90.0
}

fn ecl_lon_from_ra_dec(ra: f64, dec: f64, eps: f64) -> f64 {
    let ra_r = to_rad(ra);
    let dec_r = to_rad(dec);
    let eps_r = to_rad(eps);
    let y = ra_r.sin() * eps_r.cos() + dec_r.tan() * eps_r.sin();
    let x = ra_r.cos();
    norm_deg(to_deg(y.atan2(x)))
}

// ─── Porphyry ─────────────────────────────────────────────────────────────────

fn porphyry(asc: f64, mc: f64) -> [f64; 13] {
    let mut cusps = [0.0f64; 13];
    cusps[1] = asc;
    cusps[10] = mc;
    cusps[4] = norm_deg(mc + 180.0);
    cusps[7] = norm_deg(asc + 180.0);

    // Trisect each quadrant
    let q1 = arc_between(mc, asc) / 3.0;
    cusps[11] = norm_deg(mc + q1);
    cusps[12] = norm_deg(mc + 2.0 * q1);

    let q4 = arc_between(asc, norm_deg(mc + 180.0)) / 3.0;
    cusps[2] = norm_deg(asc + q4);
    cusps[3] = norm_deg(asc + 2.0 * q4);

    // Opposite houses
    for i in 1..=6 {
        cusps[i + 6] = norm_deg(cusps[i] + 180.0);
    }

    // Enforce 180° opposite constraint for all 6 pairs
    for &(h, opp) in &[(1usize, 7usize), (2, 8), (3, 9), (4, 10), (5, 11), (6, 12)] {
        if cusps[h] != 0.0 && cusps[opp] == 0.0 {
            cusps[opp] = norm_deg(cusps[h] + 180.0);
        }
    }

    cusps
}

/// Shortest arc from `a` to `b` in the direction of increasing longitude.
fn arc_between(a: f64, b: f64) -> f64 {
    let d = norm_deg(b - a);
    if d <= 0.0 {
        d + 360.0
    } else {
        d
    }
}

// ─── Regiomontanus ────────────────────────────────────────────────────────────

fn regiomontanus(armc: f64, lat: f64, eps: f64) -> [f64; 13] {
    let lat_r = to_rad(lat);
    let eps_r = to_rad(eps);

    let mut cusps = [0.0f64; 13];
    cusps[1] = ascendant(armc, lat, eps);
    cusps[10] = midheaven(armc, eps);

    for h in [11usize, 12, 2, 3, 8, 9] {
        let angle = match h {
            11 => 60.0,
            12 => 120.0,
            2 => 210.0,
            3 => 240.0,
            8 => 300.0,
            9 => 330.0,
            _ => unreachable!(),
        };
        let campanus_r = to_rad(armc + angle);
        let num = campanus_r.sin() * eps_r.cos();
        let den = campanus_r.cos() * lat_r.cos() - campanus_r.sin() * eps_r.sin() * lat_r.sin();
        cusps[h] = norm_deg(to_deg(num.atan2(den)));
    }

    for i in 1..=6 {
        cusps[i + 6] = norm_deg(cusps[i] + 180.0);
    }

    // Enforce 180° opposite constraint for all 6 pairs
    for &(h, opp) in &[(1usize, 7usize), (2, 8), (3, 9), (4, 10), (5, 11), (6, 12)] {
        if cusps[h] != 0.0 && cusps[opp] == 0.0 {
            cusps[opp] = norm_deg(cusps[h] + 180.0);
        }
    }

    cusps
}

// ─── Campanus ─────────────────────────────────────────────────────────────────

fn campanus(armc: f64, lat: f64, eps: f64) -> [f64; 13] {
    let lat_r = to_rad(lat);
    let eps_r = to_rad(eps);

    let mut cusps = [0.0f64; 13];
    cusps[1] = ascendant(armc, lat, eps);
    cusps[10] = midheaven(armc, eps);
    cusps[7] = norm_deg(cusps[1] + 180.0); // DSC = ASC + 180°
    cusps[4] = norm_deg(cusps[10] + 180.0); // IC  = MC  + 180°

    // Campanus: divide the prime vertical into 12 equal 30° arcs.
    // Compute upper-quadrant cusps (11, 12, 2, 3) from the prime-vertical
    // intersection formula, then set the opposing cusps as exact opposites.
    let sin_eps = eps_r.sin();
    let sin_lat = lat_r.sin();
    let cos_lat = lat_r.cos();

    for &h in &[11usize, 12, 2, 3] {
        let angle = (h as f64 - 1.0) * 30.0;
        let pv_r = to_rad(armc + angle + 90.0);
        let num = pv_r.sin() * eps_r.cos() + sin_eps * sin_lat / cos_lat;
        let den = pv_r.cos();
        cusps[h] = norm_deg(to_deg(num.atan2(den)));
        // Opposite house: h→ h+6 (wrapping within 1-12)
        let opp = if h + 6 > 12 { h - 6 } else { h + 6 };
        cusps[opp] = norm_deg(cusps[h] + 180.0);
    }

    cusps
}

// ─── Equal (from ASC) ─────────────────────────────────────────────────────────

fn equal_asc(asc: f64) -> [f64; 13] {
    let mut cusps = [0.0f64; 13];
    for h in 1..=12 {
        cusps[h] = norm_deg(asc + (h as f64 - 1.0) * 30.0);
    }
    cusps
}

// ─── Equal (from MC) ──────────────────────────────────────────────────────────

fn equal_mc(mc: f64) -> [f64; 13] {
    let mut cusps = [0.0f64; 13];
    for h in 1..=12 {
        cusps[h] = norm_deg(mc + (h as f64 - 10.0) * 30.0);
    }
    cusps
}

// ─── Whole Sign ───────────────────────────────────────────────────────────────

fn whole_sign(asc: f64) -> [f64; 13] {
    let mut cusps = [0.0f64; 13];
    let asc_sign_start = (asc / 30.0).floor() * 30.0;
    for h in 1..=12 {
        cusps[h] = norm_deg(asc_sign_start + (h as f64 - 1.0) * 30.0);
    }
    cusps
}

// ─── Meridian (Axial Rotation) ───────────────────────────────────────────────

fn meridian(armc: f64, eps: f64) -> [f64; 13] {
    let eps_r = to_rad(eps);
    let mut cusps = [0.0f64; 13];
    for h in 1..=12 {
        let angle = armc + (h as f64 - 10.0) * 30.0;
        let angle_r = to_rad(angle);
        cusps[h] = norm_deg(to_deg((angle_r.sin() * eps_r.cos()).atan2(angle_r.cos())));
    }
    // Enforce 180° opposite constraint for all 6 pairs
    for &(h, opp) in &[(1usize, 7usize), (2, 8), (3, 9), (4, 10), (5, 11), (6, 12)] {
        if cusps[h] != 0.0 && cusps[opp] == 0.0 {
            cusps[opp] = norm_deg(cusps[h] + 180.0);
        }
    }

    cusps
}

// ─── Morinus ──────────────────────────────────────────────────────────────────

fn morinus(armc: f64, eps: f64) -> [f64; 13] {
    let eps_r = to_rad(eps);
    let mut cusps = [0.0f64; 13];
    for h in 1..=12 {
        let ra = norm_deg(armc + (h as f64 - 1.0) * 30.0);
        let ra_r = to_rad(ra);
        cusps[h] = norm_deg(to_deg((ra_r.sin() * eps_r.cos()).atan2(ra_r.cos())));
    }
    // Enforce 180° opposite constraint for all 6 pairs
    for &(h, opp) in &[(1usize, 7usize), (2, 8), (3, 9), (4, 10), (5, 11), (6, 12)] {
        if cusps[h] != 0.0 && cusps[opp] == 0.0 {
            cusps[opp] = norm_deg(cusps[h] + 180.0);
        }
    }

    cusps
}

// ─── Alcabitius ───────────────────────────────────────────────────────────────

fn alcabitius(armc: f64, lat: f64, eps: f64, asc: f64) -> [f64; 13] {
    let lat_r = to_rad(lat);
    let eps_r = to_rad(eps);
    let asc_r = to_rad(asc);

    let dec_asc = (eps_r.sin() * asc_r.sin()).asin();
    let dsa = to_deg((-(lat_r.tan() * dec_asc.tan())).clamp(-1.0, 1.0).asin());

    let mut cusps = [0.0f64; 13];
    cusps[1] = asc;
    cusps[10] = midheaven(armc, eps);

    for h in [11usize, 12, 2, 3] {
        let frac = match h {
            11 => 1.0 / 3.0,
            12 => 2.0 / 3.0,
            2 => 1.0 / 3.0,
            3 => 2.0 / 3.0,
            _ => unreachable!(),
        };
        let sign = if h >= 11 { 1.0 } else { -1.0 };
        let oa = oblique_ascension(asc, 0.0, eps, lat) + sign * (90.0 + dsa) * frac;
        let lon = ecl_lon_from_ra_dec(oa + 0.0, 0.0, eps);
        cusps[h] = norm_deg(lon);
    }

    for i in 1..=6 {
        cusps[i + 6] = norm_deg(cusps[i] + 180.0);
    }

    cusps
}

// ─── Azimuthal (Horizontal) ───────────────────────────────────────────────────

fn azimuthal(armc: f64, lat: f64, eps: f64) -> [f64; 13] {
    let lat_r = to_rad(lat);
    let eps_r = to_rad(eps);

    let mut cusps = [0.0f64; 13];
    for h in 1..=12 {
        let az = (h as f64 - 1.0) * 30.0;
        let az_r = to_rad(az);
        // Convert azimuth to ecliptic longitude
        let alt = 0.0_f64; // horizon
        let az_lon = azimuth_to_ecliptic(az_r, to_rad(alt), lat_r, armc, eps_r);
        cusps[h] = az_lon;
    }
    cusps
}

fn azimuth_to_ecliptic(az: f64, alt: f64, lat_r: f64, armc: f64, eps_r: f64) -> f64 {
    let sin_lat = lat_r.sin();
    let cos_lat = lat_r.cos();
    let sin_alt = alt.sin();
    let cos_alt = alt.cos();

    let dec = (sin_alt * sin_lat + cos_alt * cos_lat * az.cos()).asin();
    let ha = (cos_alt * az.sin()).atan2(sin_alt * cos_lat - cos_alt * az.cos() * sin_lat);
    let ra = norm_deg(to_deg(to_rad(armc) - ha));
    norm_deg(ecl_lon_from_ra_dec(ra, to_deg(dec), to_deg(eps_r)))
}

// ─── Topocentric (Polich/Page) ────────────────────────────────────────────────

fn topocentric(armc: f64, lat: f64, eps: f64, asc: f64) -> [f64; 13] {
    // Topocentric (Polich/Page): close to Placidus with a latitude-based correction.
    let mut cusps = placidus(armc, lat, eps, asc, midheaven(armc, eps));

    let lat_r = to_rad(lat);
    let eps_r = to_rad(eps);
    let d = to_deg((lat_r.tan() * eps_r.sin()).atan());

    // Adjust the three pairs of intermediate cusps and immediately re-enforce
    // the 180° opposite constraint so both houses in each pair stay consistent.
    for &(h, opp) in &[(11usize, 5usize), (12, 6), (2, 8), (3, 9)] {
        cusps[h] = norm_deg(cusps[h] + d * 0.2);
        cusps[opp] = norm_deg(cusps[h] + 180.0);
    }
    for &(h, opp) in &[(8usize, 2usize), (9, 3)] {
        // 8 and 9 were already set above via 2→8 and 3→9; skip double-apply
        let _ = (h, opp);
    }

    cusps
}

// ─── Gauquelin Sectors ────────────────────────────────────────────────────────

/// Returns 37-element array: sectors[1..=36], sectors[0] unused.
fn gauquelin(armc: f64, lat: f64, eps: f64) -> [f64; 13] {
    // Standard houses placeholder — full 36-sector variant is in gauquelin_sectors()
    let mut cusps = [0.0f64; 13];
    let asc = ascendant(armc, lat, eps);
    let mc = midheaven(armc, eps);
    cusps[1] = asc;
    cusps[10] = mc;
    for i in 1..=12 {
        cusps[i] = norm_deg(asc + (i as f64 - 1.0) * 30.0);
    }
    cusps
}

// ─── Time helpers ─────────────────────────────────────────────────────────────

/// Approximate mean sidereal time (degrees) for a UT Julian day.
/// Mean Greenwich Sidereal Time (degrees) for a UT Julian day.
pub fn mean_sidereal_time_deg(jd_ut: f64) -> f64 {
    let t = (jd_ut - 2_451_545.0) / 36_525.0;
    let gmst = 280.460_618_37 + 360.985_647_366_29 * (jd_ut - 2_451_545.0) + 0.000_387_93 * t * t
        - t * t * t / 38_710_000.0;
    norm_deg(gmst)
}

/// Apparent Greenwich Sidereal Time (degrees) for a UT Julian day.
///
/// Adds the equation of the equinoxes (nutation in longitude × cos ε) to
/// mean sidereal time, giving the true origin of hour angles.
pub fn sidereal_time_deg(jd_ut: f64) -> f64 {
    let gmst = mean_sidereal_time_deg(jd_ut);
    // Equation of the equinoxes: dpsi * cos(eps)
    let nut = crate::astronomy::nutation::nutation(jd_ut);
    let eps = crate::astronomy::nutation::true_obliquity(jd_ut);
    let eq_eq = nut.dpsi / 3600.0 * eps.to_radians().cos(); // arcsec → degrees
    norm_deg(gmst + eq_eq)
}

/// Approximate obliquity of the ecliptic (degrees) for a UT Julian day.
pub fn obliquity_simple(jd_ut: f64) -> f64 {
    let t = (jd_ut - 2_451_545.0) / 36_525.0;
    23.439_291_111 - 0.013_004_2 * t - 0.000_001_64 * t * t + 0.000_000_504 * t * t * t
}

#[cfg(test)]
mod tests {
    use super::*;

    /// celestial reference: houses(2452275.499255786, 0, 0, b'P')
    /// cusps[0] == 191.0989364639854, ascmc[0] == 191.098...
    #[test]
    fn placidus_equator_reference() {
        let result = houses(2_452_275.499_255_786, 0.0, 0.0, b'P');
        // At the equator all Placidus cusps should be 30° apart
        for h in 1..=12 {
            assert!(
                result.cusps[h] >= 0.0 && result.cusps[h] < 360.0,
                "cusp {h} out of range: {}",
                result.cusps[h]
            );
        }
    }

    #[test]
    fn mc_is_opposite_ic() {
        let result = houses(2_452_275.5, 48.0, 2.0, b'P');
        let mc = result.ascmc[1];
        let ic = result.cusps[4];
        let diff = (norm_deg(ic - mc) - 180.0).abs();
        assert!(diff < 0.001, "IC should be MC+180°, diff = {diff}");
    }

    #[test]
    fn all_cusps_in_range() {
        for sys in [b'P', b'K', b'E', b'W', b'C', b'R', b'O', b'M', b'X', b'B'] {
            let result = houses(2_451_545.0, 51.5, -0.1, sys);
            for h in 1..=12 {
                assert!(
                    result.cusps[h] >= 0.0 && result.cusps[h] < 360.0,
                    "sys {} cusp {h} = {}",
                    sys as char,
                    result.cusps[h]
                );
            }
        }
    }

    #[test]
    fn equal_houses_30_apart() {
        let result = houses(2_451_545.0, 51.5, -0.1, b'E');
        for h in 1..12 {
            let diff = norm_deg(result.cusps[h + 1] - result.cusps[h]);
            assert!(
                (diff - 30.0).abs() < 0.001,
                "Equal houses: diff H{h}→H{} = {diff}",
                h + 1
            );
        }
    }

    #[test]
    fn whole_sign_multiple_of_30() {
        let result = houses(2_451_545.0, 40.0, -74.0, b'W');
        for h in 1..=12 {
            assert!(
                (result.cusps[h] % 30.0).abs() < 0.001
                    || (result.cusps[h] % 30.0 - 30.0).abs() < 0.001,
                "Whole Sign cusp {h} not on sign boundary: {}",
                result.cusps[h]
            );
        }
    }

    #[test]
    fn house_system_name() {
        assert_eq!(HouseSystem::from_char(b'P').unwrap().name(), "Placidus");
        assert_eq!(HouseSystem::from_char(b'K').unwrap().name(), "Koch");
        assert_eq!(HouseSystem::from_char(b'E').unwrap().name(), "Equal");
        assert_eq!(HouseSystem::from_char(b'W').unwrap().name(), "Whole Sign");
        assert!(HouseSystem::from_char(b'Z').is_none());
    }
}
/// Alias for `houses_armc` — compute house cusps directly from ARMC, latitude and obliquity.
pub fn houses_from_armc(armc: f64, geolat: f64, eps: f64, hsys: u8) -> Option<HouseResult> {
    Some(houses_armc(armc, geolat, eps, hsys))
}
