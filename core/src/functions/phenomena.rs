//! Astronomical phenomena, heliacal events, and Gauquelin sector calculations.

use crate::body::{Body, CalcFlags};
use crate::error::{Error, Result};

// ─── Phenomena ────────────────────────────────────────────────────────────────

/// Compute planetary phenomena.
/// Returns attr[20]: [phase_angle, phase_frac, elongation, ang_diam, magnitude, …]
pub fn pheno(jd_et: f64, body: Body, flags: CalcFlags) -> Result<[f64; 20]> {
    pheno_impl(jd_et, body, flags)
}

/// Compute planetary phenomena (UT).
pub fn pheno_ut(jd_ut: f64, body: Body, flags: CalcFlags) -> Result<[f64; 20]> {
    pheno_impl(jd_ut, body, flags)
}

// ─── Gauquelin sector ─────────────────────────────────────────────────────────

/// Compute Gauquelin sector (1–36) for a body.
///
/// The 36-sector Gauquelin system divides the diurnal arc into 18 parts above
/// and 18 below the horizon, numbered from the ascendant.
#[allow(clippy::too_many_arguments)]
pub fn gauquelin_sector(
    jd_ut: f64,
    body: Body,
    _starname: Option<&str>,
    flags: CalcFlags,
    _imeth: i32,
    geopos: [f64; 3],
    _atpress: f64,
    _attemp: f64,
) -> Result<f64> {
    use crate::functions::calc::calc_ut;
    let pos = calc_ut(jd_ut, body, flags)?;
    let lon_body = pos.lon;
    let lat_geo = geopos[1];
    let lon_geo = geopos[0];
    // Compute houses to get ASC and MC
    use crate::body::HouseSystem;
    let r = crate::functions::houses::houses(jd_ut, lat_geo, lon_geo, HouseSystem::PLACIDUS)?;
    let asc = r.ascmc[0];
    let _mc = r.ascmc[1];
    // Angular distance from ASC in the diurnal direction
    let d = (lon_body - asc + 360.0).rem_euclid(360.0);
    // Map to 36 sectors
    let sector = (d / 10.0).floor() + 1.0;
    Ok(sector.clamp(1.0, 36.0))
}

// ─── Heliacal ─────────────────────────────────────────────────────────────────

/// Find heliacal rising / setting / first-visibility event.
///
/// * `tjd_start` — Julian day to start searching (UT)
/// * `dgeo`      — `[longitude, latitude, altitude_m]` of observer
/// * `datm`      — `[pressure_mbar, temp_C, humidity_0_1, unused]`
/// * `dobs`      — `[age, snellen_ratio, ...]`
/// * `objectname`— planet name ("venus", "mars", …) or star name from catalog
/// * `type_event` — 1=heliacal rising, 2=heliacal setting, 3=evening first,
///   4=morning last, 5=evening rising, 6=morning setting
/// * `helflag`   — flags (currently unused)
///
/// Returns a vector of 50 f64 values where `[0]` is the event JD.
pub fn heliacal_ut(
    jd_start: f64,
    dgeo: [f64; 3],
    datm: [f64; 4],
    dobs: [f64; 6],
    objectname: &str,
    type_event: i32,
    _flags: CalcFlags,
) -> Result<Vec<f64>> {
    use crate::astronomy::heliacal::{find_heliacal_event, HeliacalEvent};
    let body_num = body_name_to_num(objectname);
    let event = HeliacalEvent::from_i32(type_event);
    let result = find_heliacal_event(jd_start, dgeo, datm, dobs, body_num, event)
        .ok_or_else(|| Error::Calc("heliacal_ut: no event found within search window".into()))?;
    let mut dret = vec![0.0f64; 50];
    dret[0] = result.jd_event;
    dret[1] = result.obj_alt;
    dret[2] = result.sun_alt;
    dret[3] = result.obj_az;
    Ok(dret)
}

/// Compute heliacal phenomena attributes for a body at a given JD.
///
/// Returns a 50-element array with elongation, arc of vision, sky brightness,
/// limiting magnitude, and related quantities.
pub fn heliacal_pheno_ut(
    jd_ut: f64,
    dgeo: [f64; 3],
    datm: [f64; 4],
    dobs: [f64; 6],
    objectname: &str,
    _type_event: i32,
    _flags: CalcFlags,
) -> Result<Vec<f64>> {
    use crate::astronomy::heliacal::heliacal_pheno;
    let body_num = body_name_to_num(objectname);
    Ok(heliacal_pheno(jd_ut, dgeo, datm, dobs, body_num))
}

fn pheno_impl(jd: f64, body: Body, _flags: CalcFlags) -> Result<[f64; 20]> {
    let mut attr = [0.0f64; 20];
    use std::f64::consts::PI;
    let to_rad = |d: f64| d * PI / 180.0;
    use crate::astronomy::phenomena;
    use crate::functions::calc::calc_ut;
    // Geocentric position of the body
    let pos = calc_ut(jd, body, CalcFlags::BUILTIN)?;
    // Geocentric position of the Sun
    let sun = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN)?;
    // Heliocentric distance of the body ≈ distance from Sun
    // (simplified: use geocentric dist for outer planets, 1 AU for Sun)
    let dist_sun = if body == Body::SUN {
        pos.dist
    } else {
        // approximate heliocentric distance
        (pos.dist * pos.dist + 1.0 - 2.0 * pos.dist * (to_rad(pos.lon - sun.lon).cos()))
            .sqrt()
            .max(0.01)
    };
    let ph =
        phenomena::compute_phenomena(body.as_raw(), pos.lon, pos.lat, pos.dist, dist_sun, sun.lon);
    attr[0] = ph.phase_angle;
    attr[1] = ph.phase_frac;
    attr[2] = ph.elongation;
    attr[3] = ph.ang_diameter;
    attr[4] = ph.magnitude;
    Ok(attr)
}

/// Compute visibility limiting magnitude and related quantities.
///
/// Returns `[lim_mag, obj_mag, sky_brightness, elongation, arc_vision, sun_alt, pressure_mb, temp_c]`
pub fn vis_limit_mag(
    jd_ut: f64,
    dgeo: [f64; 3],
    datm: [f64; 4],
    dobs: [f64; 6],
    objectname: &str,
    helflag: i32,
) -> Result<[f64; 8]> {
    use crate::astronomy::heliacal::vis_limit_mag as vlm;
    let body_num: i32 = match objectname.trim().to_ascii_lowercase().as_str() {
        "" | "sun" => 0,
        "moon" => 1,
        "mercury" => 2,
        "venus" => 3,
        "mars" => 4,
        "jupiter" => 5,
        "saturn" => 6,
        "uranus" => 7,
        "neptune" => 8,
        "pluto" => 9,
        _ => 2, // default to Mercury for unknown
    };
    Ok(vlm(jd_ut, dgeo, datm, dobs, body_num, helflag))
}

/// Map a body name string to a SWE body number.
fn body_name_to_num(name: &str) -> i32 {
    match name.trim().to_ascii_lowercase().as_str() {
        "" | "sun" => 0,
        "moon" => 1,
        "mercury" => 2,
        "venus" => 3,
        "mars" => 4,
        "jupiter" => 5,
        "saturn" => 6,
        "uranus" => 7,
        "neptune" => 8,
        "pluto" => 9,
        _ => {
            // Try fixed-star catalog — fall back to Venus as representative
            if crate::astronomy::fixstars::find_star(name).is_some() {
                3
            } else {
                2
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Yallop crescent visibility (Yallop 1998)
// ═══════════════════════════════════════════════════════════════════════════

/// Yallop's q-value and visibility classification for a lunar crescent.
///
/// Based on Yallop, B.D. (1998), "A method for predicting the first sighting
/// of the new crescent moon", NAO Technical Note No. 69.
///
/// # Inputs
/// - `arcv_deg`:  arc of vision = topocentric altitude of Moon − altitude of Sun (°)
/// - `arcl_deg`:  arc of light  = topocentric elongation of Moon from Sun (°)
/// - `sd_arcmin`: topocentric semi-diameter of the Moon (arcminutes)
///
/// All inputs should be evaluated at the "best time" =
/// sunset + 4/9 × (moonset − sunset). See [`best_time_method`].
///
/// # Returns
/// `(q, code)` where `code` is one of:
/// - `'A'`: easily visible (naked eye)
/// - `'B'`: visible under perfect conditions (naked eye)
/// - `'C'`: may need optical aid to find first, then naked-eye visible
/// - `'D'`: will need optical aid (binoculars or telescope)
/// - `'E'`: barely visible even with telescope
/// - `'F'`: not visible (below Danjon limit)
pub fn yallop_q(arcv_deg: f64, arcl_deg: f64, sd_arcmin: f64) -> (f64, char) {
    // Crescent width W (arcminutes) = SD · (1 − cos(ARCL))
    let w = sd_arcmin * (1.0 - arcl_deg.to_radians().cos());
    // Yallop's q: (ARCV − polynomial(W)) / 10
    let poly = 11.8371 - 6.3226 * w + 0.7319 * w.powi(2) - 0.1018 * w.powi(3);
    let q = (arcv_deg - poly) / 10.0;

    let code = if q > 0.216 {
        'A'
    } else if q > -0.014 {
        'B'
    } else if q > -0.160 {
        'C'
    } else if q > -0.232 {
        'D'
    } else if q > -0.293 {
        'E'
    } else {
        'F'
    };
    (q, code)
}

/// Best time for crescent visibility evaluation:
/// sunset + 4/9 × (moonset − sunset).
///
/// Inputs are Julian Days (UT). This is the standard epoch for evaluating
/// [`yallop_q`] and related crescent-visibility criteria.
pub fn best_time_method(jd_sunset: f64, jd_moonset: f64) -> f64 {
    jd_sunset + (4.0 / 9.0) * (jd_moonset - jd_sunset)
}

#[cfg(test)]
mod yallop_tests {
    use super::*;

    #[test]
    fn yallop_easily_visible() {
        // Wide crescent (20° elongation) with 10° ARCV → class A
        let (_, c) = yallop_q(10.0, 20.0, 15.0);
        assert_eq!(c, 'A');
    }

    #[test]
    fn yallop_below_danjon() {
        // Sun and moon very close, moon below sun → not visible
        let (_, c) = yallop_q(-3.0, 5.0, 15.0);
        assert_eq!(c, 'F');
    }

    #[test]
    fn yallop_marginal_boundaries() {
        // ARCV of 10.5° with 10° elongation → around class B/C boundary
        let (q, _) = yallop_q(10.5, 10.0, 15.0);
        // q should be near zero — boundary between classes B and C
        assert!(q.abs() < 0.2, "q = {q}");
    }

    #[test]
    fn best_time_midway() {
        // If moonset is 2 hours after sunset, best time is 4/9 × 2h after sunset
        let jd_ss = 2_451_545.0;
        let jd_ms = jd_ss + 2.0 / 24.0;
        let bt = best_time_method(jd_ss, jd_ms);
        let expected = jd_ss + (4.0 / 9.0) * 2.0 / 24.0;
        assert!((bt - expected).abs() < 1e-9);
    }
}
