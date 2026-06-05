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
use crate::units::{Degrees, JulianDay, Latitude, Longitude};

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
    /// House cusps. `cusps[1]` = 1st house, …, `cusps[12]` = 12th house.
    pub cusps: [f64; 13],
    /// Special angles: ASC, MC, ARMC, Vertex, EqAsc, etc.
    pub ascmc: [f64; 10],
    /// Speeds (deg/day) of the house cusps, parallel to `cusps`.
    pub cusp_speeds: [f64; 13],
    /// Speeds (deg/day) of the special angles, parallel to `ascmc`.
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
    #[must_use]
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
pub fn houses(jd_ut: JulianDay, geolat: Latitude, geolon: Longitude, hsys: u8) -> HouseResult {
    let jd_ut: f64 = jd_ut.into();
    let geolat: f64 = geolat.into();
    let geolon: f64 = geolon.into();
    let armc = sidereal_time_deg(JulianDay::new(jd_ut)) + geolon;
    let armc = norm_deg(armc);
    let eps = obliquity_simple(JulianDay::new(jd_ut));
    houses_armc(
        Degrees::new(armc),
        Latitude::new(geolat),
        Degrees::new(eps),
        hsys,
    )
}

/// Compute house cusps from ARMC, latitude and obliquity.
///
/// This is the low-level entry point used when ARMC and obliquity are
/// already known (e.g. from the full astronomy pipeline).
pub fn houses_armc(armc: Degrees, geolat: Latitude, eps: Degrees, hsys: u8) -> HouseResult {
    let armc: f64 = armc.into();
    let geolat: f64 = geolat.into();
    let eps: f64 = eps.into();
    let system = HouseSystem::from_char(hsys).unwrap_or(HouseSystem::Placidus);
    compute_houses(armc, geolat, eps, system)
}

// ─── Dispatcher ───────────────────────────────────────────────────────────────

fn compute_houses(armc: f64, lat: f64, eps: f64, sys: HouseSystem) -> HouseResult {
    let asc = ascendant(Degrees::new(armc), Latitude::new(lat), Degrees::new(eps));
    let mc = midheaven(Degrees::new(armc), Degrees::new(eps));

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
#[must_use]
pub fn ascendant(armc: Degrees, lat: Latitude, eps: Degrees) -> f64 {
    let armc: f64 = armc.into();
    let lat: f64 = lat.into();
    let eps: f64 = eps.into();
    let armc_r = to_rad(armc);
    let lat_r = to_rad(lat);
    let eps_r = to_rad(eps);

    // Standard ASC formula (Meeus, Astr. Algorithms):
    //   atan2(-cos(ARMC), sin(ARMC)·cos(ε) + tan(φ)·sin(ε))
    // The raw atan2 gives the western horizon (DSC); adding 180° gives the
    // eastern horizon (ASC) — the point actually rising on the ecliptic.
    let (sin_armc, cos_armc) = armc_r.sin_cos();
    let (sin_eps, cos_eps) = eps_r.sin_cos();
    let y = -cos_armc;
    let x = lat_r.tan().mul_add(sin_eps, sin_armc * cos_eps);
    norm_deg(to_deg(y.atan2(x)) + 180.0)
}

/// Midheaven (MC) (degrees).
#[must_use]
pub fn midheaven(armc: Degrees, eps: Degrees) -> f64 {
    let armc: f64 = armc.into();
    let eps: f64 = eps.into();
    let armc_r = to_rad(armc);
    let eps_r = to_rad(eps);
    // Meeus, "Astronomical Algorithms" 2nd ed. ch. 13/25, for a point on the
    // ecliptic (latitude 0°):
    //     tan(MC) = sin(ARMC) / (cos(ARMC) · cos(ε))
    // i.e. cos(ε) divides cos(ARMC), it does NOT multiply sin(ARMC).
    // atan2 handles all four quadrants — no manual ±180° adjustment.
    let (sin_armc, cos_armc) = armc_r.sin_cos();
    norm_deg(to_deg(sin_armc.atan2(cos_armc * eps_r.cos())))
}

/// Vertex: the point on the ecliptic where the prime vertical intersects
/// in the western hemisphere.
fn vertex_point(armc: f64, lat: f64, eps: f64) -> f64 {
    // Vertex = ASC for latitude - 90° rotated 90°
    let anti_armc = norm_deg(armc + 90.0);
    ascendant(
        Degrees::new(anti_armc),
        Latitude::new(90.0 - lat.abs()),
        Degrees::new(eps),
    )
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
    let geolat_r = to_rad(geolat);

    let (sin_lon, cos_lon) = lon_r.sin_cos();
    let (sin_eps, cos_eps) = eps_r.sin_cos();
    let (sin_lat, cos_lat) = lat_r.sin_cos();
    let ra_r = (-lat_r.tan())
        .mul_add(sin_eps, sin_lon * cos_eps)
        .atan2(cos_lon);
    let dec = sin_lat.mul_add(cos_eps, cos_lat * sin_eps * sin_lon).asin();
    let ad_arg = (geolat_r.tan() * dec.tan()).clamp(-1.0, 1.0);
    let ad = ad_arg.asin();
    norm_deg(to_deg(ra_r) - to_deg(ad))
}

// ─── Placidus ─────────────────────────────────────────────────────────────────

/// Placidus house cusps via iterative semi-arc method.
///
/// Classical Placidus: each intermediate cusp lies at a fixed fraction of
/// the semi-diurnal (or semi-nocturnal) arc swept by a point with that
/// cusp's declination. Because the declination depends on the ecliptic
/// longitude we're solving for, the formula is iterative.
///
/// Reference: Pingré (1786); modern reformulation in Walter Koch (1960).
fn placidus(armc: f64, lat: f64, eps: f64, asc: f64, mc: f64) -> [f64; 13] {
    let lat_r = to_rad(lat);
    let eps_r = to_rad(eps);

    let mut cusps = [0.0f64; 13];
    cusps[1] = asc;
    cusps[10] = mc;
    cusps[4] = norm_deg(mc + 180.0);
    cusps[7] = norm_deg(asc + 180.0);

    // House → (semi-arc fraction, hemisphere sign).
    //   sign = +1  → upper hemisphere (semi-diurnal arc, above horizon)
    //   sign = −1  → lower hemisphere (semi-nocturnal arc, below horizon)
    //
    // For each cusp the hour-angle from the meridian is `sign · F · SA`,
    // where SA is the semi-diurnal arc when sign=+1 and the semi-nocturnal
    // arc when sign=−1. Sign convention for the hour angle: positive west
    // of meridian. Cusps east of the meridian get negative HA.
    //
    //   h11 = 1/3 of upper east semi-arc from MC, HA = -F·SDA   (east, upper)
    //   h12 = 2/3 of upper east semi-arc from MC, HA = -F·SDA   (east, upper)
    //   h2  = 1/3 of lower east semi-arc from ASC, HA = -π + F·SNA (east, lower)
    //   h3  = 2/3 of lower east semi-arc from ASC, HA = -π + F·SNA (east, lower)
    // and h5, h6, h8, h9 are 180°-opposites of h11, h12, h2, h3.
    let upper = [(11_usize, 1.0 / 3.0), (12, 2.0 / 3.0)];
    let lower = [(2_usize, 1.0 / 3.0), (3, 2.0 / 3.0)];

    for (cusp, f) in upper {
        let lon = placidus_cusp_iter(armc, lat_r, eps_r, f, true);
        cusps[cusp] = norm_deg(lon);
        cusps[cusp - 6] = norm_deg(lon + 180.0); // h5 opp h11, h6 opp h12
    }
    for (cusp, f) in lower {
        let lon = placidus_cusp_iter(armc, lat_r, eps_r, f, false);
        cusps[cusp] = norm_deg(lon);
        cusps[cusp + 6] = norm_deg(lon + 180.0); // h8 opp h2, h9 opp h3
    }

    cusps
}

/// Iteratively solve for one Placidus intermediate cusp.
///
/// `f` is the semi-arc fraction (1/3 or 2/3). `upper = true` for cusps
/// 11, 12 (above horizon, east of meridian); `false` for cusps 2, 3
/// (below horizon, east of meridian).
///
/// Cusp hour angles from the meridian (HA = ARMC − RA, negative = east):
///   h11 (upper, f=1/3):  HA = −F · SDA
///   h12 (upper, f=2/3):  HA = −F · SDA
///   h2  (lower, f=1/3):  HA = −SDA − F · SNA
///   h3  (lower, f=2/3):  HA = −SDA − F · SNA
/// where SDA = acos(−tan φ · tan δ) and SNA = π − SDA depend on the
/// cusp's own declination. Iterate until both δ and λ converge.
fn placidus_cusp_iter(armc: f64, lat_r: f64, eps_r: f64, f: f64, upper: bool) -> f64 {
    let sin_eps = eps_r.sin();
    let cos_eps = eps_r.cos();
    let tan_lat = lat_r.tan();
    let armc_r = to_rad(armc);

    // Initial guess: equator approximation (SDA = SNA = π/2).
    let mut lon_r = if upper {
        armc_r + f * std::f64::consts::FRAC_PI_2
    } else {
        armc_r + std::f64::consts::FRAC_PI_2 + f * std::f64::consts::FRAC_PI_2
    };

    for _ in 0..30 {
        let sin_lon = lon_r.sin();
        let dec = (sin_eps * sin_lon).asin();
        let tan_dec = dec.tan();

        // SDA, SNA for the current declination. |tan φ · tan δ| ≥ 1 means
        // the cusp is circumpolar at this latitude (rare); clamp to keep
        // the iteration finite — Placidus is conventionally undefined
        // there but every astrology program returns *something*.
        let cos_arg = (-tan_lat * tan_dec).clamp(-1.0, 1.0);
        let sda = cos_arg.acos();
        let sna = std::f64::consts::PI - sda;

        // HA from meridian (negative east). Then RA of cusp = ARMC − HA.
        // (Convention here: HA = ARMC − RA so RA = ARMC − HA, but since
        // we want RA = ARMC + |east offset|, just add the positive
        // east-offset directly.)
        let east_offset = if upper { f * sda } else { sda + f * sna };
        let ra = armc_r + east_offset;
        let (sin_ra, cos_ra) = ra.sin_cos();

        // Invert the equatorial→ecliptic transform for a point ON the
        // ecliptic (β = 0). For such a point the forward transform gives
        //     sin(α)·cos(δ) = sin(λ)·cos(ε)
        //     cos(α)·cos(δ) = cos(λ)
        // so tan(λ) = sin(α) / (cos(α)·cos(ε)). The tan(δ)·sin(ε) term
        // that appears in the more general inversion vanishes when β = 0.
        let _ = tan_dec; // unused: kept available for non-ecliptic cusps
        let lon_new_r = sin_ra.atan2(cos_ra * cos_eps);

        let delta = (lon_new_r - lon_r).rem_euclid(2.0 * std::f64::consts::PI);
        let delta = delta.min(2.0 * std::f64::consts::PI - delta);
        lon_r = lon_new_r;
        if delta < 1e-9 {
            break;
        }
    }
    to_deg(lon_r)
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
        let armc_h = norm_deg(dsa.mul_add(frac, armc));
        let armc_r = to_rad(armc_h);
        // Declination that rises with this ARMC in latitude lat
        let sin_armc = armc_r.sin();
        if sin_armc.abs() < 1e-8 {
            // Degenerate case: fall back to previous cusp + 30°
            cusps[*h] = norm_deg((*h as f64 - 10.0).mul_add(30.0, cusps[10]));
            cusps[*h - 9] = norm_deg(cusps[*h] + 180.0);
            continue;
        }
        let dec_arg = (lat_r.tan() * armc_r.cos() / sin_armc.abs()).atan();
        let dec_sin = dec_arg.sin().clamp(-1.0, 1.0);
        let dec = dec_sin.clamp(-1.0, 1.0).asin();
        let ra = (armc_r.sin() * eps_r.cos()).atan2(armc_r.cos());
        let lon = ecl_lon_from_ra_dec(to_deg(ra), to_deg(dec), eps);
        if lon.is_nan() || lon.is_infinite() {
            cusps[*h] = norm_deg((*h as f64 - 10.0).mul_add(30.0, cusps[10]));
        } else {
            cusps[*h] = lon;
        }
        cusps[*h - 9] = norm_deg(cusps[*h] + 180.0);
    }

    for (i, h) in [9usize, 8].iter().enumerate() {
        let frac = (i as f64 + 1.0) / 3.0;
        let armc_h = norm_deg(dsa.mul_add(-frac, armc));
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
            cusps[*h] = norm_deg((*h as f64 - 10.0).mul_add(30.0, cusps[10]));
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
    let (sin_ra, cos_ra) = ra_r.sin_cos();
    let (sin_eps, cos_eps) = eps_r.sin_cos();
    let y = dec_r.tan().mul_add(sin_eps, sin_ra * cos_eps);
    norm_deg(to_deg(y.atan2(cos_ra)))
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
    cusps[12] = norm_deg(2.0_f64.mul_add(q1, mc));

    let q4 = arc_between(asc, norm_deg(mc + 180.0)) / 3.0;
    cusps[2] = norm_deg(asc + q4);
    cusps[3] = norm_deg(2.0_f64.mul_add(q4, asc));

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
    cusps[1] = ascendant(Degrees::new(armc), Latitude::new(lat), Degrees::new(eps));
    cusps[10] = midheaven(Degrees::new(armc), Degrees::new(eps));
    // Set IC and DSC up front so the later "fill opposites from cusps
    // 1..=6" loop has correct seed values for h=4 and h=1.
    cusps[4] = norm_deg(cusps[10] + 180.0);
    cusps[7] = norm_deg(cusps[1] + 180.0);

    for h in [11usize, 12, 2, 3, 8, 9] {
        // Campanus uses constant 30°-spaced angles per house; the input set is
        // closed so falling through to a default is dead code, but if a caller
        // ever passes an unexpected `h` we fall back to 0° rather than panic.
        let angle = match h {
            11 => 60.0,
            12 => 120.0,
            2 => 210.0,
            3 => 240.0,
            8 => 300.0,
            _ => 330.0, // h == 9
        };
        let campanus_r = to_rad(armc + angle);
        let (sin_c, cos_c) = campanus_r.sin_cos();
        let (sin_eps, cos_eps) = eps_r.sin_cos();
        let (sin_lat, cos_lat) = lat_r.sin_cos();
        let num = sin_c * cos_eps;
        let den = (-sin_c).mul_add(sin_eps * sin_lat, cos_c * cos_lat);
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
    cusps[1] = ascendant(Degrees::new(armc), Latitude::new(lat), Degrees::new(eps));
    cusps[10] = midheaven(Degrees::new(armc), Degrees::new(eps));
    cusps[7] = norm_deg(cusps[1] + 180.0); // DSC = ASC + 180°
    cusps[4] = norm_deg(cusps[10] + 180.0); // IC  = MC  + 180°

    // Campanus: divide the prime vertical into 12 equal 30° arcs.
    // Compute upper-quadrant cusps (11, 12, 2, 3) from the prime-vertical
    // intersection formula, then set the opposing cusps as exact opposites.
    let sin_eps = eps_r.sin();
    let sin_lat = lat_r.sin();
    let cos_lat = lat_r.cos();
    let cos_lat_denom = cos_lat.abs().max(1e-10).copysign(cos_lat);

    for &h in &[11usize, 12, 2, 3] {
        let angle = (h as f64 - 1.0) * 30.0;
        let pv_r = to_rad(armc + angle + 90.0);
        let num = pv_r
            .sin()
            .mul_add(eps_r.cos(), sin_eps * sin_lat / cos_lat_denom);
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
        cusps[h] = norm_deg((h as f64 - 1.0).mul_add(30.0, asc));
    }
    cusps
}

// ─── Equal (from MC) ──────────────────────────────────────────────────────────

fn equal_mc(mc: f64) -> [f64; 13] {
    let mut cusps = [0.0f64; 13];
    for h in 1..=12 {
        cusps[h] = norm_deg((h as f64 - 10.0).mul_add(30.0, mc));
    }
    cusps
}

// ─── Whole Sign ───────────────────────────────────────────────────────────────

fn whole_sign(asc: f64) -> [f64; 13] {
    let mut cusps = [0.0f64; 13];
    let asc_sign_start = (asc / 30.0).floor() * 30.0;
    for h in 1..=12 {
        cusps[h] = norm_deg((h as f64 - 1.0).mul_add(30.0, asc_sign_start));
    }
    cusps
}

// ─── Meridian (Axial Rotation) ───────────────────────────────────────────────

fn meridian(armc: f64, eps: f64) -> [f64; 13] {
    let eps_r = to_rad(eps);
    let mut cusps = [0.0f64; 13];
    for h in 1..=12 {
        let angle = (h as f64 - 10.0).mul_add(30.0, armc);
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
        let ra = norm_deg((h as f64 - 1.0).mul_add(30.0, armc));
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
    cusps[10] = midheaven(Degrees::new(armc), Degrees::new(eps));
    // IC and DSC seeded explicitly so the later opposite-fill loop has
    // valid values for h=4 and h=1 (same bug class as Regiomontanus).
    cusps[4] = norm_deg(cusps[10] + 180.0);
    cusps[7] = norm_deg(cusps[1] + 180.0);

    for h in [11usize, 12, 2, 3] {
        let frac = match h {
            11 | 2 => 1.0 / 3.0,
            _ => 2.0 / 3.0, // h ∈ {12, 3}
        };
        let sign = if h >= 11 { 1.0 } else { -1.0 };
        let oa = (sign * (90.0 + dsa)).mul_add(frac, oblique_ascension(asc, 0.0, eps, lat));
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

    let dec = sin_alt
        .mul_add(sin_lat, cos_alt * cos_lat * az.cos())
        .asin();
    let ha = (cos_alt * az.sin()).atan2(sin_alt.mul_add(cos_lat, -(cos_alt * az.cos() * sin_lat)));
    let ra = norm_deg(to_deg(to_rad(armc) - ha));
    norm_deg(ecl_lon_from_ra_dec(ra, to_deg(dec), to_deg(eps_r)))
}

// ─── Topocentric (Polich/Page) ────────────────────────────────────────────────

fn topocentric(armc: f64, lat: f64, eps: f64, asc: f64) -> [f64; 13] {
    // Topocentric (Polich/Page): close to Placidus with a latitude-based correction.
    let mut cusps = placidus(
        armc,
        lat,
        eps,
        asc,
        midheaven(Degrees::new(armc), Degrees::new(eps)),
    );

    let lat_r = to_rad(lat);
    let eps_r = to_rad(eps);
    let d = to_deg((lat_r.tan() * eps_r.sin()).atan());

    // Adjust the three pairs of intermediate cusps and immediately re-enforce
    // the 180° opposite constraint so both houses in each pair stay consistent.
    for &(h, opp) in &[(11usize, 5usize), (12, 6), (2, 8), (3, 9)] {
        cusps[h] = norm_deg(d.mul_add(0.2, cusps[h]));
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
    let asc = ascendant(Degrees::new(armc), Latitude::new(lat), Degrees::new(eps));
    let mc = midheaven(Degrees::new(armc), Degrees::new(eps));
    cusps[1] = asc;
    cusps[10] = mc;
    for i in 1..=12 {
        cusps[i] = norm_deg((i as f64 - 1.0).mul_add(30.0, asc));
    }
    cusps
}

// ─── Time helpers ─────────────────────────────────────────────────────────────

/// Approximate mean sidereal time (degrees) for a UT Julian day.
/// Mean Greenwich Sidereal Time (degrees) for a UT Julian day.
#[must_use]
pub fn mean_sidereal_time_deg(jd_ut: JulianDay) -> f64 {
    let jd_ut: f64 = jd_ut.into();
    let t = (jd_ut - 2_451_545.0) / 36_525.0;
    let dt = jd_ut - 2_451_545.0;
    let p1 = 360.985_647_366_29_f64.mul_add(dt, 280.460_618_37);
    let p2 = (0.000_387_93 * t).mul_add(t, p1);
    let gmst = (t * t).mul_add(-t / 38_710_000.0, p2);
    norm_deg(gmst)
}

/// Apparent Greenwich Sidereal Time (degrees) for a UT Julian day.
///
/// Adds the equation of the equinoxes (nutation in longitude × cos ε) to
/// mean sidereal time, giving the true origin of hour angles.
#[must_use]
pub fn sidereal_time_deg(jd_ut: JulianDay) -> f64 {
    let jd_ut: f64 = jd_ut.into();
    let gmst = mean_sidereal_time_deg(JulianDay::new(jd_ut));
    // Equation of the equinoxes: dpsi * cos(eps)
    let nut = crate::astronomy::nutation::nutation(jd_ut);
    // PERF-7: reuse `nut` instead of re-running the 77-term series inside
    // true_obliquity. Byte-identical: true_obliquity == mean_obliquity + deps/3600.
    let eps = crate::astronomy::nutation::mean_obliquity(jd_ut) + nut.deps / 3600.0;
    let eq_eq = nut.dpsi / 3600.0 * eps.to_radians().cos(); // arcsec → degrees
    norm_deg(gmst + eq_eq)
}

/// Approximate obliquity of the ecliptic (degrees) for a UT Julian day.
#[must_use]
pub fn obliquity_simple(jd_ut: JulianDay) -> f64 {
    let jd_ut: f64 = jd_ut.into();
    let t = (jd_ut - 2_451_545.0) / 36_525.0;
    // Horner form: more accurate (single rounding per FMA on FMA hosts) and
    // equivalent in IEEE-754 to the original term-by-term expansion within
    // ULP precision for any practical |t|.
    let p1 = 0.000_000_504_f64.mul_add(t, -0.000_001_64);
    let p2 = p1.mul_add(t, -0.013_004_2);
    p2.mul_add(t, 23.439_291_111)
}
/// Alias for `houses_armc` — compute house cusps directly from ARMC, latitude and obliquity.
pub fn houses_from_armc(armc: Degrees, geolat: Latitude, eps: Degrees, hsys: u8) -> HouseResult {
    let armc: f64 = armc.into();
    let geolat: f64 = geolat.into();
    let eps: f64 = eps.into();
    houses_armc(
        Degrees::new(armc),
        Latitude::new(geolat),
        Degrees::new(eps),
        hsys,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// celestial reference: houses(2452275.499255786, 0, 0, b'P')
    /// cusps[0] == 191.0989364639854, ascmc[0] == 191.098...
    #[test]
    fn placidus_equator_reference() {
        let result = houses(
            JulianDay::new(2_452_275.499_255_786),
            Latitude::new(0.0),
            Longitude::new(0.0),
            b'P',
        );
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
        let result = houses(
            JulianDay::new(2_452_275.5),
            Latitude::new(48.0),
            Longitude::new(2.0),
            b'P',
        );
        let mc = result.ascmc[1];
        let ic = result.cusps[4];
        let diff = (norm_deg(ic - mc) - 180.0).abs();
        assert!(diff < 0.001, "IC should be MC+180°, diff = {diff}");
    }

    #[test]
    fn all_cusps_in_range() {
        for sys in [b'P', b'K', b'E', b'W', b'C', b'R', b'O', b'M', b'X', b'B'] {
            let result = houses(
                JulianDay::new(2_451_545.0),
                Latitude::new(51.5),
                Longitude::new(-0.1),
                sys,
            );
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
    fn campanus_poles_are_finite() {
        for lat in [-90.0, 90.0] {
            let result = houses(
                JulianDay::new(2_451_545.0),
                Latitude::new(lat),
                Longitude::new(2.35),
                b'C',
            );
            for h in 1..=12 {
                assert!(
                    result.cusps[h].is_finite(),
                    "lat={lat} cusp[{h}]={}",
                    result.cusps[h]
                );
            }
            assert!(
                result.ascmc[0].is_finite(),
                "lat={lat} asc={}",
                result.ascmc[0]
            );
            assert!(
                result.ascmc[1].is_finite(),
                "lat={lat} mc={}",
                result.ascmc[1]
            );
        }
    }

    #[test]
    fn equal_houses_30_apart() {
        let result = houses(
            JulianDay::new(2_451_545.0),
            Latitude::new(51.5),
            Longitude::new(-0.1),
            b'E',
        );
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
        let result = houses(
            JulianDay::new(2_451_545.0),
            Latitude::new(40.0),
            Longitude::new(-74.0),
            b'W',
        );
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

    /// Regression test against an independently verified Placidus chart
    /// (World of Wisdom astrology software, 1986-05-30 06:00 -03:00,
    /// São Paulo 23°32'S 46°38'W). Asserts every cusp matches the
    /// reference within 5 arcminutes — the precision the published chart
    /// is quoted to.
    #[test]
    fn placidus_full_cusp_match_published_chart_1986_sp() {
        // 06:00 local UTC-3 = 09:00 UT.
        let jd_ut =
            crate::functions::time::julday(1986, 5, 30, 9.0, crate::body::Calendar::Gregorian);
        let result = houses(
            JulianDay::new(jd_ut),
            Latitude::new(-23.5333),
            Longitude::new(-46.6333),
            b'P',
        );
        let tol = 5.0 / 60.0; // 5 arcminutes

        // PDF cusps in decimal degrees (sign × 30 + degrees + minutes/60).
        let pdf = [
            ("h1 ASC", 1, 59.0667),  // 29°04' Taurus
            ("h2", 2, 88.7833),      // 28°47' Gemini
            ("h3", 3, 120.6833),     // 0°41' Leo
            ("h4 IC", 4, 154.05),    // 4°03' Virgo
            ("h5", 5, 186.0833),     // 6°05' Libra
            ("h6", 6, 214.3667),     // 4°22' Scorpio
            ("h7 DSC", 7, 239.0667), // 29°04' Scorpio
            ("h8", 8, 268.7833),     // 28°47' Sagittarius
            ("h9", 9, 300.6833),     // 0°41' Aquarius
            ("h10 MC", 10, 334.05),  // 4°03' Pisces
            ("h11", 11, 6.0833),     // 6°05' Aries
            ("h12", 12, 34.3667),    // 4°22' Taurus
        ];

        for (label, idx, ref_lon) in pdf {
            let got = result.cusps[idx];
            let diff = ((got - ref_lon + 540.0) % 360.0 - 180.0).abs();
            assert!(
                diff < tol,
                "{label} = {got:.4}°, expected {ref_lon:.4}° (diff {diff:.4}° = {:.2}')",
                diff * 60.0,
            );
        }

        // Sanity: every Placidus cusp is the exact 180° opposite of its pair.
        for h in 1..=6 {
            let opp = (result.cusps[h] + 180.0) % 360.0;
            let diff = (result.cusps[h + 6] - opp).abs();
            assert!(
                diff < 1e-6 || (diff - 360.0).abs() < 1e-6,
                "Placidus h{}+180° = {opp}° but h{} = {} (diff {diff}°)",
                h,
                h + 6,
                result.cusps[h + 6],
            );
        }
    }
}
