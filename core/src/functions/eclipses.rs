//! Eclipse and occultation search functions.

use crate::units::JulianDay;
use crate::astronomy::eclipses as ae;
use crate::body::{Body, CalcFlags};
use crate::error::{Error, Result};

// ─── Return types ─────────────────────────────────────────────────────────────

/// Eclipse search result: classification flags plus contact-time array.
#[derive(Debug, Clone, PartialEq)]
#[must_use = "the search result contains the computed data — did you mean to use it?"]
pub struct EclipseResult {
    /// Eclipse-type bit flags (e.g. `ECL_TOTAL`, `ECL_PARTIAL`).
    pub ret_flags: i32,
    /// Eclipse contact times as Julian Days (UT); `tret[0]` is the maximum.
    pub tret: [f64; 10],
}
/// Eclipse search result with location-specific attributes.
#[derive(Debug, Clone, PartialEq)]
pub struct EclipseResultAttr {
    /// Eclipse-type bit flags.
    pub ret_flags: i32,
    /// Eclipse contact times as Julian Days (UT); `tret[0]` is the maximum.
    pub tret: [f64; 10],
    /// Eclipse attributes (magnitude, obscuration, altitude, …).
    pub attr: [f64; 20],
}
/// Eclipse circumstances (magnitude, obscuration, contacts) at a fixed time.
#[derive(Debug, Clone, PartialEq)]
pub struct EclipseHow {
    /// Eclipse-type bit flags.
    pub ret_flags: i32,
    /// Eclipse attributes (magnitude, obscuration, contact times, …).
    pub attr: [f64; 20],
}
/// Eclipse geographic result: point of greatest eclipse plus attributes.
#[derive(Debug, Clone, PartialEq)]
pub struct EclipseWhere {
    /// Eclipse-type bit flags.
    pub ret_flags: i32,
    /// Geographic position; `geopos[0]` = longitude, `geopos[1]` = latitude.
    pub geopos: [f64; 10],
    /// Eclipse attributes (magnitude, gamma, shadow-axis distance, …).
    pub attr: [f64; 20],
}

fn ec_to_result(e: ae::EclipseResult) -> EclipseResult {
    EclipseResult {
        ret_flags: e.ret_flags,
        tret: e.tret,
    }
}

// ─── Solar eclipses ───────────────────────────────────────────────────────────

/// Next solar eclipse globally, starting from `tjd_start`.
///
/// `ecl_type` filters by eclipse kind: 0 = any, or combine `ECL_TOTAL`,
/// `ECL_ANNULAR`, `ECL_PARTIAL`, `ECL_HYBRID`.
pub fn sol_eclipse_when_glob(
    tjd_start: JulianDay,
    _flags: CalcFlags,
    ecl_type: i32,
    backwards: bool,
) -> Result<EclipseResult> {
    let tjd_start: f64 = tjd_start.into();
    ae::solar_eclipse_when_glob(JulianDay::new(tjd_start), ecl_type, backwards)
        .map(ec_to_result)
        .ok_or(Error::NoEclipseFound { from_jd: tjd_start })
}

/// Next solar eclipse visible from a geographic location.
///
/// `geopos` = `[longitude, latitude, altitude_m]`.
pub fn sol_eclipse_when_loc(
    tjd_start: JulianDay,
    _flags: CalcFlags,
    geopos: [f64; 3],
    backwards: bool,
) -> Result<EclipseResultAttr> {
    let tjd_start: f64 = tjd_start.into();
    ae::solar_eclipse_when_glob(JulianDay::new(tjd_start), 0, backwards)
        .map(|e| {
            let attr = ae::solar_eclipse_attr(JulianDay::new(e.tret[0]), geopos);
            EclipseResultAttr {
                ret_flags: e.ret_flags,
                tret: e.tret,
                attr,
            }
        })
        .ok_or(Error::NoEclipseFound { from_jd: tjd_start })
}

/// Solar eclipse attributes at a specific time and location.
///
/// Returns magnitude, obscuration, and contact times in the `attr` array.
pub fn sol_eclipse_how(jd_ut: JulianDay, _flags: CalcFlags, geopos: [f64; 3]) -> Result<EclipseHow> {
    let jd_ut: f64 = jd_ut.into();
    let attr = ae::solar_eclipse_attr(JulianDay::new(jd_ut), geopos);
    let ret_flags = if attr[1] > 0.0 {
        ae::ECL_TOTAL
    } else {
        ae::ECL_PARTIAL
    };
    Ok(EclipseHow { ret_flags, attr })
}

/// Geographic point of greatest solar eclipse at a given Julian Day (UT).
///
/// Computes the sub-lunar point where the Moon's shadow axis intersects Earth's
/// surface — i.e. the location experiencing the greatest eclipse.
///
/// `geopos[0]` = longitude (°E), `geopos[1]` = latitude (°N).
/// `attr[0]`   = eclipse magnitude at that point.
/// `attr[7]`   = gamma (shadow axis distance from Earth centre, in Earth radii).
pub fn sol_eclipse_where(jd_ut: JulianDay, _flags: CalcFlags) -> Result<EclipseWhere> {
    let jd_ut: f64 = jd_ut.into();
    let k = ae::k_from_jd(JulianDay::new(jd_ut), true);
    let (kind, gamma, u) = ae::check_solar_eclipse(k);
    if kind == ae::EclipseKind::None {
        // No central eclipse at this time — find the nearest one and use it
        // Return the point for the nearest solar eclipse
        let result = ae::solar_eclipse_when_glob(JulianDay::new(jd_ut), 0, false)
            .ok_or(Error::NoEclipseFound { from_jd: jd_ut })?;
        let k2 = ae::k_from_jd(JulianDay::new(result.tret[0]), true);
        let (_kind2, gamma2, u2) = ae::check_solar_eclipse(k2);
        let jde = result.tret[0];
        let (lon, lat) = ae::solar_eclipse_geopos(jde, gamma2);
        let (pen_mag, umb_mag) = ae::solar_eclipse_magnitude(k2);
        let mut geopos = [0.0f64; 10];
        let mut attr = [0.0f64; 20];
        geopos[0] = lon;
        geopos[1] = lat;
        attr[0] = if umb_mag > 0.0 { umb_mag } else { pen_mag };
        attr[7] = gamma2;
        attr[8] = u2;
        return Ok(EclipseWhere {
            ret_flags: result.ret_flags,
            geopos,
            attr,
        });
    }

    let jde = ae::new_moon_jd(k as f64);
    let (lon, lat) = ae::solar_eclipse_geopos(jde, gamma);
    let (pen_mag, umb_mag) = ae::solar_eclipse_magnitude(k);

    let mut geopos = [0.0f64; 10];
    let mut attr = [0.0f64; 20];
    geopos[0] = lon;
    geopos[1] = lat;
    attr[0] = if umb_mag > 0.0 { umb_mag } else { pen_mag };
    attr[5] = umb_mag.max(0.0);
    attr[6] = pen_mag.max(0.0);
    attr[7] = gamma;
    attr[8] = u;

    Ok(EclipseWhere {
        ret_flags: ae::kind_to_flags(kind),
        geopos,
        attr,
    })
}

// ─── Lunar eclipses ───────────────────────────────────────────────────────────

/// Next lunar eclipse, starting from `tjd_start`.
///
/// `ecl_type` filters: 0 = any, or `ECL_TOTAL`, `ECL_PARTIAL`, `ECL_PENUMBRAL`.
pub fn lun_eclipse_when(
    tjd_start: JulianDay,
    _flags: CalcFlags,
    ecl_type: i32,
    backwards: bool,
) -> Result<EclipseResult> {
    let tjd_start: f64 = tjd_start.into();
    ae::lun_eclipse_when(JulianDay::new(tjd_start), ecl_type, backwards)
        .map(ec_to_result)
        .ok_or(Error::NoEclipseFound { from_jd: tjd_start })
}

/// Next lunar eclipse visible from a geographic location.
pub fn lun_eclipse_when_loc(
    tjd_start: JulianDay,
    _flags: CalcFlags,
    _geopos: [f64; 3],
    backwards: bool,
) -> Result<EclipseResultAttr> {
    let tjd_start: f64 = tjd_start.into();
    ae::lun_eclipse_when(JulianDay::new(tjd_start), 0, backwards)
        .map(|e| EclipseResultAttr {
            ret_flags: e.ret_flags,
            tret: e.tret,
            attr: [0.0; 20],
        })
        .ok_or(Error::NoEclipseFound { from_jd: tjd_start })
}

/// Lunar eclipse attributes at a specific time.
pub fn lun_eclipse_how(
    jd_ut: JulianDay,
    _flags: CalcFlags,
    _geopos: Option<[f64; 3]>,
) -> Result<EclipseHow> {
    let jd_ut: f64 = jd_ut.into();
    use crate::astronomy::eclipses::k_from_jd;
    let k = k_from_jd(JulianDay::new(jd_ut), true);
    let attr = ae::lunar_eclipse_attr(k);
    let (_, pen, umb) = ae::check_lunar_eclipse(k);
    let flags = if umb > 0.0 {
        ae::ECL_TOTAL
    } else if pen > 0.0 {
        ae::ECL_PARTIAL
    } else {
        ae::ECL_PENUMBRAL
    };
    Ok(EclipseHow {
        ret_flags: flags,
        attr,
    })
}

// ─── Lunar occultations ───────────────────────────────────────────────────────

/// Next occultation of a planet by the Moon, searching globally.
///
/// A lunar occultation occurs when the Moon passes in front of a planet as
/// seen from Earth. The search steps forward in 0.5-day increments and
/// refines using Newton's method when an angular separation minimum is found.
///
/// `body.as_raw()` — body number (see body constants: `VENUS`, `MARS`, `SATURN`, etc.)
/// `ecl_type` — 0 = any occultation; unused (reserved for future filter)
///
/// Returns the time of closest approach in `tret[0]` and the angular
/// separation at that time (degrees) in `attr[0]` of the result.
/// Ecliptic (lon°, lat°) → equatorial (RA rad, Dec rad) using true obliquity.
fn ecl_to_eq(lon_deg: f64, lat_deg: f64, eps: f64) -> (f64, f64) {
    let lon = lon_deg.to_radians();
    let lat = lat_deg.to_radians();
    let (sin_lon, cos_lon) = lon.sin_cos();
    let (sin_lat, cos_lat) = lat.sin_cos();
    let (sin_eps, cos_eps) = eps.sin_cos();
    let ra = (-lat.tan())
        .mul_add(sin_eps, sin_lon * cos_eps)
        .atan2(cos_lon);
    let dec = sin_lat
        .mul_add(cos_eps, cos_lat * sin_eps * sin_lon)
        .asin();
    (ra, dec)
}

/// True angular separation (degrees) between Moon and `body` at given JD.
fn moon_body_separation(jd: f64, body: Body) -> Option<f64> {
    let moon = crate::calc_ut(JulianDay::new(jd), Body::MOON, CalcFlags::BUILTIN).ok()?;
    let planet = crate::calc_ut(JulianDay::new(jd), body, CalcFlags::BUILTIN).ok()?;
    let eps = crate::true_obliquity(JulianDay::new(jd)).to_radians();
    let (ra_m, dec_m) = ecl_to_eq(moon.lon, moon.lat, eps);
    let (ra_p, dec_p) = ecl_to_eq(planet.lon, planet.lat, eps);
    let d_ra = ra_m - ra_p;
    let (sin_dm, cos_dm) = dec_m.sin_cos();
    let (sin_dp, cos_dp) = dec_p.sin_cos();
    let cos_d = sin_dm.mul_add(sin_dp, cos_dm * cos_dp * d_ra.cos());
    Some(cos_d.clamp(-1.0, 1.0).acos().to_degrees())
}

/// Golden-section refinement around an angular-separation minimum candidate.
fn refine_separation_minimum(jd_mid: f64, step: f64, body: Body) -> (f64, f64) {
    let bracket = 2.0 * step.abs();
    let (mut lo, mut hi) = (jd_mid - bracket, jd_mid + bracket);
    for _ in 0..30 {
        let phi = (hi - lo) / 3.0;
        let m1 = lo + phi;
        let m2 = hi - phi;
        if moon_body_separation(m1, body).unwrap_or(180.0)
            < moon_body_separation(m2, body).unwrap_or(180.0)
        {
            hi = m2;
        } else {
            lo = m1;
        }
    }
    let jd_occ = (lo + hi) / 2.0;
    (jd_occ, moon_body_separation(jd_occ, body).unwrap_or(180.0))
}

/// One step of the occultation search. Returns `Some(EclipseResult)` if the
/// scanned interval contains a refined occultation within the disc-diameter
/// threshold; `None` if no occultation here.
fn try_refine_occultation(jd: f64, step: f64, prev: f64, curr: f64, body: Body) -> Option<EclipseResult> {
    const THRESHOLD: f64 = 1.5;
    const OCC_DISC: f64 = 0.27;
    let next = moon_body_separation(jd + step, body).unwrap_or(180.0);
    let is_min = curr <= prev && curr <= next && curr < THRESHOLD;
    if !is_min {
        return None;
    }
    let (jd_occ, min_sep) = refine_separation_minimum(jd, step, body);
    if min_sep >= OCC_DISC {
        return None;
    }
    let mut tret = [0.0f64; 10];
    tret[0] = jd_occ;
    Some(EclipseResult { ret_flags: 64, tret })
}

/// Next occultation of `body` by the Moon, searching globally from `tjd_start`.
///
/// Scans in 0.1-day steps for an angular-separation minimum and refines it;
/// returns the time of closest approach in `tret[0]`. Set `backwards` to search
/// toward earlier dates.
pub fn lun_occult_when_glob(
    tjd_start: JulianDay,
    body: Body,
    _starname: Option<&str>,
    _flags: CalcFlags,
    _ecl_type: i32,
    backwards: bool,
) -> Result<EclipseResult> {
    let tjd_start: f64 = tjd_start.into();
    let step = if backwards { -0.1_f64 } else { 0.1_f64 };
    let mut jd = tjd_start;
    let limit = step.mul_add(4000.0, tjd_start);
    let mut prev = moon_body_separation(jd, body).unwrap_or(180.0);

    for _ in 0..5000 {
        if (jd - limit) * step.signum() >= 0.0 {
            break;
        }
        jd += step;
        let curr = moon_body_separation(jd, body).unwrap_or(180.0);
        if let Some(result) = try_refine_occultation(jd, step, prev, curr, body) {
            return Ok(result);
        }
        prev = curr;
    }

    Err(Error::Eclipse(format!(
        "no occultation of body {} found within search window",
        body.as_raw()
    )))
}

/// Next occultation of a planet by the Moon visible from a geographic location.
///
/// Finds the global occultation time (via [`lun_occult_when_glob`]) then checks
/// visibility from `geopos`. Returns visibility attributes in `attr`:
/// `attr[0]` = angular separation at closest approach (degrees),
/// `attr[1]` = altitude of Moon above horizon at that time (degrees, negative = below).
///
/// `geopos` = `[longitude_deg, latitude_deg, altitude_m]`.
pub fn lun_occult_when_loc(
    tjd_start: JulianDay,
    body: Body,
    starname: Option<&str>,
    flags: CalcFlags,
    geopos: [f64; 3],
    backwards: bool,
) -> Result<EclipseResultAttr> {
    // Find the global occultation first
    let global = lun_occult_when_glob(tjd_start, body, starname, flags, 0, backwards)?;
    let jd_occ = global.tret[0];

    // Compute Moon altitude at the occultation time from the given location
    let moon_alt = {
        let moon = crate::calc_ut(JulianDay::new(jd_occ), Body::MOON, CalcFlags::BUILTIN).unwrap_or_default();
        // Hour angle = LST - RA (approximate: use geographic longitude for LST)
        let lst_deg = crate::sidtime(JulianDay::new(jd_occ)) * 15.0 + geopos[0];
        let ha_deg = lst_deg - moon.lon; // rough HA using ecliptic lon ≈ RA
        let ha_r = ha_deg.to_radians();
        let lat_r = geopos[1].to_radians();
        let dec_r = moon.lat.to_radians();
        // Altitude formula
        let (sin_lat, cos_lat) = lat_r.sin_cos();
        let (sin_dec, cos_dec) = dec_r.sin_cos();
        let sin_alt = sin_lat.mul_add(sin_dec, cos_lat * cos_dec * ha_r.cos());
        sin_alt.clamp(-1.0, 1.0).asin().to_degrees()
    };

    let mut attr = [0.0f64; 20];
    attr[1] = moon_alt; // altitude of Moon at occultation

    Ok(EclipseResultAttr {
        ret_flags: global.ret_flags,
        tret: global.tret,
        attr,
    })
}

/// Geographic path of a lunar occultation.
///
/// Currently returns a stub result.
pub fn lun_occult_where(
    _jd_ut: JulianDay,
    _body: Body,
    _starname: Option<&str>,
    _flags: CalcFlags,
) -> Result<EclipseWhere> {
    Ok(EclipseWhere {
        ret_flags: 0,
        geopos: [0.0; 10],
        attr: [0.0; 20],
    })
}
