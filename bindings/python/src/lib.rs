//! Python bindings for celestial-core via PyO3.
//!
//! Build with `maturin develop` (dev install) or `maturin build` (wheel).
//! The module is exposed as `celestial_py`.

use celestial::body::{Body, CalcFlags, Calendar, HouseSystem, SiderealMode};
use celestial_core as celestial;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

// ─── Error mapping ────────────────────────────────────────────────────────────

fn to_py(e: celestial::Error) -> PyErr {
    PyRuntimeError::new_err(e.to_string())
}

// ─── Configuration ────────────────────────────────────────────────────────────

/// Set the path to Swiss Ephemeris data files.
#[pyfunction]
#[pyo3(signature = (path = ""))]
fn set_ephe_path(path: &str) -> PyResult<()> {
    celestial::set_ephe_path(path).map_err(to_py)
}

/// Set the JPL ephemeris file name.
#[pyfunction]
fn set_jpl_file(fname: &str) -> PyResult<()> {
    celestial::set_jpl_file(fname).map_err(to_py)
}

/// Set the sidereal mode. `sid_mode` is one of the `SIDM_*` constants.
#[pyfunction]
#[pyo3(signature = (sid_mode, t0 = 0.0, ayan_t0 = 0.0))]
fn set_sid_mode(sid_mode: i32, t0: f64, ayan_t0: f64) {
    celestial::set_sid_mode(SiderealMode(sid_mode), t0, ayan_t0);
}

/// Set the topocentric observer position.
#[pyfunction]
fn set_topo(geolon: f64, geolat: f64, geoalt: f64) {
    celestial::set_topo(geolon, geolat, geoalt);
}

/// Override the ΔT value. Pass `f64::MAX` to reset to automatic.
#[pyfunction]
fn set_delta_t_userdef(dt: f64) {
    celestial::set_delta_t_userdef(dt);
}

/// Close the Swiss Ephemeris and free all resources.
#[pyfunction]
fn close() {
    celestial::close();
}

// ─── Calculations ─────────────────────────────────────────────────────────────

/// Calculate planetary positions (ET).
///
/// Returns `((lon, lat, dist, speed_lon, speed_lat, speed_dist), ret_flags)`.
#[pyfunction]
#[pyo3(signature = (tjdet, planet, flags = 258))]
fn calc(py: Python<'_>, tjdet: f64, planet: i32, flags: i32) -> PyResult<PyObject> {
    let pos = celestial::calc(tjdet, Body::from_raw(planet), CalcFlags(flags)).map_err(to_py)?;
    let xx = (
        pos.lon,
        pos.lat,
        pos.dist,
        pos.speed_lon,
        pos.speed_lat,
        pos.speed_dist,
    );
    Ok((xx, pos.ret_flags).into_py(py))
}

/// Calculate planetary positions (UT).
#[pyfunction]
#[pyo3(signature = (tjdut, planet, flags = 258))]
fn calc_ut(py: Python<'_>, tjdut: f64, planet: i32, flags: i32) -> PyResult<PyObject> {
    let pos = celestial::calc_ut(tjdut, Body::from_raw(planet), CalcFlags(flags)).map_err(to_py)?;
    let xx = (
        pos.lon,
        pos.lat,
        pos.dist,
        pos.speed_lon,
        pos.speed_lat,
        pos.speed_dist,
    );
    Ok((xx, pos.ret_flags).into_py(py))
}

/// Nutation in longitude and obliquity at a JDE (TT).
///
/// Returns `(dpsi_degrees, deps_degrees)` — both in degrees.
/// Multiply by 3600 to convert to arcseconds.
/// Uses the IAU 2000B luni-solar series (77 terms, ~1 mas accuracy).
#[pyfunction]
#[pyo3(signature = (jde))]
fn nutation(jde: f64) -> (f64, f64) {
    celestial::nutation(jde)
}

/// Mean obliquity of the ecliptic in degrees (IAU 2006 formula).
#[pyfunction]
#[pyo3(signature = (jde))]
fn mean_obliquity(jde: f64) -> f64 {
    celestial::mean_obliquity(jde)
}

/// True (apparent) obliquity of the ecliptic in degrees.
/// Equals mean obliquity + nutation in obliquity.
#[pyfunction]
#[pyo3(signature = (jde))]
fn true_obliquity(jde: f64) -> f64 {
    celestial::true_obliquity(jde)
}

/// Calculate positions for a list of bodies in parallel (ET / TT input).
///
/// Returns a list of ((lon, lat, dist, speed_lon, speed_lat, speed_dist), ret_flags)
/// tuples in the same order as `planets`.
#[pyfunction]
#[pyo3(signature = (tjdet, planets, flags = 258))]
fn calc_many(py: Python<'_>, tjdet: f64, planets: Vec<i32>, flags: i32) -> PyResult<PyObject> {
    let bodies: Vec<_> = planets.iter().map(|&p| Body::from_raw(p)).collect();
    let results = celestial::calc_many(tjdet, &bodies, CalcFlags(flags));
    let list: Vec<PyObject> = results
        .into_iter()
        .map(|r| {
            let pos = r.map_err(to_py)?;
            let xx = (
                pos.lon,
                pos.lat,
                pos.dist,
                pos.speed_lon,
                pos.speed_lat,
                pos.speed_dist,
            );
            Ok::<PyObject, pyo3::PyErr>((xx, pos.ret_flags).into_py(py))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(list.into_py(py))
}

/// Calculate positions for a list of bodies in parallel (UT input).
#[pyfunction]
#[pyo3(signature = (tjdut, planets, flags = 258))]
fn calc_ut_many(py: Python<'_>, tjdut: f64, planets: Vec<i32>, flags: i32) -> PyResult<PyObject> {
    let bodies: Vec<_> = planets.iter().map(|&p| Body::from_raw(p)).collect();
    let results = celestial::calc_ut_many(tjdut, &bodies, CalcFlags(flags));
    let list: Vec<PyObject> = results
        .into_iter()
        .map(|r| {
            let pos = r.map_err(to_py)?;
            let xx = (
                pos.lon,
                pos.lat,
                pos.dist,
                pos.speed_lon,
                pos.speed_lat,
                pos.speed_dist,
            );
            Ok::<PyObject, pyo3::PyErr>((xx, pos.ret_flags).into_py(py))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(list.into_py(py))
}

/// Calculate planetocentric positions (ET).
#[pyfunction]
#[pyo3(signature = (tjdet, planet, center, flags = 258))]
fn calc_pctr(
    py: Python<'_>,
    tjdet: f64,
    planet: i32,
    center: i32,
    flags: i32,
) -> PyResult<PyObject> {
    let pos = celestial::calc_pctr(
        tjdet,
        Body::from_raw(planet),
        Body::from_raw(center),
        CalcFlags(flags),
    )
    .map_err(to_py)?;
    let xx = (
        pos.lon,
        pos.lat,
        pos.dist,
        pos.speed_lon,
        pos.speed_lat,
        pos.speed_dist,
    );
    Ok((xx, pos.ret_flags).into_py(py))
}

/// Calculate a fixed star position (ET). Returns `((xx), name, ret_flags)`.
#[pyfunction]
#[pyo3(signature = (star, tjdet, flags = 2))]
fn fixstar(py: Python<'_>, star: &str, tjdet: f64, flags: i32) -> PyResult<PyObject> {
    let r = celestial::fixstar(star, tjdet, CalcFlags(flags)).map_err(to_py)?;
    let xx = PyObject::from(pyo3::types::PyTuple::new_bound(py, r.xx));
    Ok((xx, r.star_name, r.ret_flags).into_py(py))
}

/// Fixed star position (UT).
#[pyfunction]
#[pyo3(signature = (star, tjdut, flags = 2))]
fn fixstar_ut(py: Python<'_>, star: &str, tjdut: f64, flags: i32) -> PyResult<PyObject> {
    let r = celestial::fixstar_ut(star, tjdut, CalcFlags(flags)).map_err(to_py)?;
    let xx = PyObject::from(pyo3::types::PyTuple::new_bound(py, r.xx));
    Ok((xx, r.star_name, r.ret_flags).into_py(py))
}

/// Faster fixed star position (ET).
#[pyfunction]
#[pyo3(signature = (star, tjdet, flags = 2))]
fn fixstar2(py: Python<'_>, star: &str, tjdet: f64, flags: i32) -> PyResult<PyObject> {
    let r = celestial::fixstar2(star, tjdet, CalcFlags(flags)).map_err(to_py)?;
    let xx = PyObject::from(pyo3::types::PyTuple::new_bound(py, r.xx));
    Ok((xx, r.star_name, r.ret_flags).into_py(py))
}

/// Faster fixed star position (UT).
#[pyfunction]
#[pyo3(signature = (star, tjdut, flags = 2))]
fn fixstar2_ut(py: Python<'_>, star: &str, tjdut: f64, flags: i32) -> PyResult<PyObject> {
    let r = celestial::fixstar2_ut(star, tjdut, CalcFlags(flags)).map_err(to_py)?;
    let xx = PyObject::from(pyo3::types::PyTuple::new_bound(py, r.xx));
    Ok((xx, r.star_name, r.ret_flags).into_py(py))
}

/// Get fixed star magnitude.
#[pyfunction]
fn fixstar_mag(star: &str) -> PyResult<f64> {
    celestial::fixstar_mag(star).map_err(to_py)
}

/// Faster fixed star magnitude.
#[pyfunction]
fn fixstar2_mag(star: &str) -> PyResult<f64> {
    celestial::fixstar2_mag(star).map_err(to_py)
}

// ─── Houses ───────────────────────────────────────────────────────────────────

/// Calculate house cusps (UT).
///
/// Returns `((cusps...), (ascmc...))`.
#[pyfunction]
#[pyo3(signature = (tjdut, lat, lon, hsys = b'P'))]
fn houses(py: Python<'_>, tjdut: f64, lat: f64, lon: f64, hsys: u8) -> PyResult<PyObject> {
    let r = celestial::houses(tjdut, lat, lon, HouseSystem(hsys)).map_err(to_py)?;
    // cusps[0] is unused in SE convention; return cusps[1..=12] (12 real cusps)
    // ascmc[0..8] is the SE standard (skip our extra IC/DSC at [8],[9])
    let cusps: Vec<f64> = r.cusps[1..].to_vec();
    let ascmc: Vec<f64> = r.ascmc[..8].to_vec();
    Ok((cusps, ascmc).into_py(py))
}

/// Extended house cusps with flags.
#[pyfunction]
#[pyo3(signature = (tjdut, lat, lon, hsys = b'P', flags = 0))]
fn houses_ex(
    py: Python<'_>,
    tjdut: f64,
    lat: f64,
    lon: f64,
    hsys: u8,
    flags: i32,
) -> PyResult<PyObject> {
    let r = celestial::houses_ex(tjdut, CalcFlags(flags), lat, lon, HouseSystem(hsys))
        .map_err(to_py)?;
    let cusps: Vec<f64> = r.cusps[1..].to_vec();
    let ascmc: Vec<f64> = r.ascmc[..8].to_vec();
    Ok((cusps, ascmc).into_py(py))
}

/// Houses with cusp speeds.
#[pyfunction]
#[pyo3(signature = (tjdut, lat, lon, hsys = b'P', flags = 0))]
fn houses_ex2(
    py: Python<'_>,
    tjdut: f64,
    lat: f64,
    lon: f64,
    hsys: u8,
    flags: i32,
) -> PyResult<PyObject> {
    let r = celestial::houses_ex2(tjdut, CalcFlags(flags), lat, lon, HouseSystem(hsys))
        .map_err(to_py)?;
    Ok((
        r.cusps[1..].to_vec(),
        r.ascmc[..8].to_vec(),
        r.cusp_speeds[1..].to_vec(),
        r.ascmc_speeds[..8].to_vec(),
    )
        .into_py(py))
}

/// Calculate house position of a point.
#[pyfunction]
#[pyo3(signature = (armc, lat, eps, hsys = b'P', lon = 0.0, lat_body = 0.0))]
fn house_pos(armc: f64, lat: f64, eps: f64, hsys: u8, lon: f64, lat_body: f64) -> PyResult<f64> {
    celestial::house_pos(armc, lat, eps, HouseSystem(hsys), [lon, lat_body]).map_err(to_py)
}

/// Name of a house system given its byte code.
#[pyfunction]
fn house_name(hsys: u8) -> String {
    celestial::house_name(HouseSystem(hsys))
}

// ─── Eclipses ─────────────────────────────────────────────────────────────────

/// Find next solar eclipse globally (UT). Returns `(ret_flags, (tret...))`.
#[pyfunction]
#[pyo3(signature = (tjd_start, flags = 2, ecl_type = 0, backwards = false))]
fn sol_eclipse_when_glob(
    py: Python<'_>,
    tjd_start: f64,
    flags: i32,
    ecl_type: i32,
    backwards: bool,
) -> PyResult<PyObject> {
    let r = celestial::sol_eclipse_when_glob(tjd_start, CalcFlags(flags), ecl_type, backwards)
        .map_err(to_py)?;
    Ok((r.ret_flags, r.tret.to_vec()).into_py(py))
}

/// Find next solar eclipse from a location (UT). Returns `(ret_flags, (tret...), (attr...))`.
#[pyfunction]
#[pyo3(signature = (tjd_start, geopos, flags = 2, backwards = false))]
fn sol_eclipse_when_loc(
    py: Python<'_>,
    tjd_start: f64,
    geopos: [f64; 3],
    flags: i32,
    backwards: bool,
) -> PyResult<PyObject> {
    let r = celestial::sol_eclipse_when_loc(tjd_start, CalcFlags(flags), geopos, backwards)
        .map_err(to_py)?;
    Ok((r.ret_flags, r.tret.to_vec(), r.attr.to_vec()).into_py(py))
}

/// How a solar eclipse looks from a location. Returns `(ret_flags, (attr...))`.
#[pyfunction]
#[pyo3(signature = (jd_ut, geopos, flags = 2))]
fn sol_eclipse_how(py: Python<'_>, jd_ut: f64, geopos: [f64; 3], flags: i32) -> PyResult<PyObject> {
    let r = celestial::sol_eclipse_how(jd_ut, CalcFlags(flags), geopos).map_err(to_py)?;
    Ok((r.ret_flags, r.attr.to_vec()).into_py(py))
}

/// Where a solar eclipse is central. Returns `(ret_flags, (geopos...), (attr...))`.
#[pyfunction]
#[pyo3(signature = (tjd, flags = 2))]
fn sol_eclipse_where(py: Python<'_>, tjd: f64, flags: i32) -> PyResult<PyObject> {
    let r = celestial::sol_eclipse_where(tjd, CalcFlags(flags)).map_err(to_py)?;
    Ok((r.ret_flags, r.geopos.to_vec(), r.attr.to_vec()).into_py(py))
}

/// Find next lunar eclipse (UT). Returns `(ret_flags, (tret...))`.
#[pyfunction]
#[pyo3(signature = (tjd_start, flags = 2, ecl_type = 0, backwards = false))]
fn lun_eclipse_when(
    py: Python<'_>,
    tjd_start: f64,
    flags: i32,
    ecl_type: i32,
    backwards: bool,
) -> PyResult<PyObject> {
    let r = celestial::lun_eclipse_when(tjd_start, CalcFlags(flags), ecl_type, backwards)
        .map_err(to_py)?;
    Ok((r.ret_flags, r.tret.to_vec()).into_py(py))
}

/// Find next lunar eclipse from a location. Returns `(ret_flags, (tret...), (attr...))`.
#[pyfunction]
#[pyo3(signature = (tjd_start, geopos, flags = 2, backwards = false))]
fn lun_eclipse_when_loc(
    py: Python<'_>,
    tjd_start: f64,
    geopos: [f64; 3],
    flags: i32,
    backwards: bool,
) -> PyResult<PyObject> {
    let r = celestial::lun_eclipse_when_loc(tjd_start, CalcFlags(flags), geopos, backwards)
        .map_err(to_py)?;
    Ok((r.ret_flags, r.tret.to_vec(), r.attr.to_vec()).into_py(py))
}

/// Lunar eclipse attributes. Returns `(ret_flags, (attr...))`.
#[pyfunction]
#[pyo3(signature = (jd_ut, geopos = None, flags = 2))]
fn lun_eclipse_how(
    py: Python<'_>,
    jd_ut: f64,
    geopos: Option<[f64; 3]>,
    flags: i32,
) -> PyResult<PyObject> {
    let r = celestial::lun_eclipse_how(jd_ut, CalcFlags(flags), geopos).map_err(to_py)?;
    Ok((r.ret_flags, r.attr.to_vec()).into_py(py))
}

// ─── Rise / transit ───────────────────────────────────────────────────────────

/// Rise/set/transit calculation. Returns `(ret_flags, tret)`.
#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (tjdut, planet, event_type, geopos, pressure_mb = 0.0, temp_c = 0.0, flags = 0))]
fn rise_trans(
    py: Python<'_>,
    tjdut: f64,
    planet: i32,
    event_type: i32,
    geopos: [f64; 3],
    pressure_mb: f64,
    temp_c: f64,
    flags: i32,
) -> PyResult<PyObject> {
    let r = celestial::rise_trans(
        tjdut,
        Body::from_raw(planet),
        None,
        CalcFlags(flags),
        event_type,
        geopos,
        pressure_mb,
        temp_c,
    )
    .map_err(to_py)?;
    Ok((r.ret_flags, r.tret).into_py(py))
}

// ─── Time ─────────────────────────────────────────────────────────────────────

/// Convert a calendar date to a Julian day number.
#[pyfunction]
#[pyo3(signature = (year, month, day, hour = 0.0, calendar = celestial::GREG_CAL))]
fn julday(year: i32, month: i32, day: i32, hour: f64, calendar: i32) -> f64 {
    celestial::julday(year, month, day, hour, Calendar::from(calendar))
}

/// Convert a Julian day number to a calendar date. Returns `(year, month, day, hour)`.
#[pyfunction]
fn revjul(py: Python<'_>, jd: f64, calendar: i32) -> PyObject {
    let d = celestial::revjul(jd, Calendar::from(calendar));
    (d.year, d.month, d.day, d.hour).into_py(py)
}

/// Day of week (0 = Monday, …, 6 = Sunday).
#[pyfunction]
fn day_of_week(jd: f64) -> i32 {
    celestial::day_of_week(jd)
}

/// Delta-T (TT − UT) for a Julian day.
#[pyfunction]
fn deltat(tjd: f64) -> f64 {
    celestial::deltat(tjd)
}

/// Sidereal time for a UT Julian day.
#[pyfunction]
fn sidtime(jd_ut: f64) -> f64 {
    celestial::sidtime(jd_ut)
}

/// Greenwich Mean Sidereal Time (GMST) in decimal hours — without the
/// equation of the equinoxes. Use `sidtime()` for apparent sidereal time (GAST).
#[pyfunction]
fn mean_sidtime(jd_ut: f64) -> f64 {
    celestial::mean_sidtime(jd_ut)
}

/// Convert UTC to Julian day numbers. Returns `(jdet, jdut1)`.
#[allow(clippy::too_many_arguments)]
#[pyfunction]
fn utc_to_jd(
    py: Python<'_>,
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: f64,
    calendar: i32,
) -> PyResult<PyObject> {
    let date = celestial::UtcDate {
        year,
        month,
        day,
        hour,
        minute,
        second,
    };
    let pair = celestial::utc_to_jd(&date, Calendar::from(calendar)).map_err(to_py)?;
    Ok((pair.et, pair.ut1).into_py(py))
}

// ─── Ayanamsa ─────────────────────────────────────────────────────────────────

/// Ayanamsa for a Julian day (ET).
#[pyfunction]
fn ayanamsa(jd_et: f64) -> f64 {
    celestial::ayanamsa(jd_et)
}

/// Ayanamsa for a Julian day (UT).
#[pyfunction]
fn ayanamsa_ut(jd_ut: f64) -> f64 {
    celestial::ayanamsa_ut(jd_ut)
}

/// Name of a sidereal mode.
#[pyfunction]
fn ayanamsa_name(sid_mode: i32) -> &'static str {
    celestial::ayanamsa_name(sid_mode)
}

// ─── Math utilities ───────────────────────────────────────────────────────────

/// Normalise degrees to 0…360.
#[pyfunction]
fn norm_deg(x: f64) -> f64 {
    celestial::norm_deg(x)
}

/// Midpoint of two degree values (360° wrap aware).
#[pyfunction]
fn midpoint_deg(x1: f64, x0: f64) -> f64 {
    celestial::midpoint_deg(x1, x0)
}

/// Signed difference between two degree values (result in −180…+180).
#[pyfunction]
fn diff_deg_signed(p1: f64, p2: f64) -> f64 {
    celestial::diff_deg_signed(p1, p2)
}

/// Normalise centiseconds to 0…360°.
#[pyfunction]
fn norm_cs(p: i32) -> i64 {
    celestial::norm_cs(p)
}

/// Split a degree value to degrees, minutes, seconds, fraction, sign.
#[pyfunction]
fn split_deg(py: Python<'_>, deg: f64, round_flag: i32) -> PyObject {
    let (d, m, s, frac, sgn) = celestial::split_deg(deg, round_flag);
    (d, m, s, frac, sgn).into_py(py)
}

/// Coordinate transform ecliptic ↔ equatorial.
#[pyfunction]
fn coord_transform(py: Python<'_>, coord: [f64; 3], eps: f64) -> PyObject {
    celestial::coord_transform(coord, eps).into_py(py)
}

/// Azimuth and altitude from ecliptic/equatorial coordinates.
#[pyfunction]
fn azalt(
    py: Python<'_>,
    tjdut: f64,
    calc_flag: i32,
    geopos: [f64; 3],
    pressure_mb: f64,
    temp_c: f64,
    xin: [f64; 3],
) -> PyObject {
    let r = celestial::azalt(tjdut, calc_flag, geopos, pressure_mb, temp_c, xin);
    (r.azimuth, r.true_alt, r.apparent_alt).into_py(py)
}

/// Reverse azimuth/altitude to coordinates. Returns `(lon, lat)`.
#[pyfunction]
fn azalt_rev(
    py: Python<'_>,
    tjdut: f64,
    calc_flag: i32,
    geopos: [f64; 3],
    az: f64,
    alt: f64,
) -> PyObject {
    let out = celestial::azalt_rev(tjdut, calc_flag, geopos, [az, alt]);
    (out[0], out[1]).into_py(py)
}

/// Atmospheric refraction.
#[pyfunction]
fn refrac(altitude: f64, pressure_mb: f64, temp_c: f64, calc_flag: i32) -> f64 {
    celestial::refrac(altitude, pressure_mb, temp_c, calc_flag)
}

/// Extended atmospheric refraction. Returns `(result, (dret[4]))`.
#[pyfunction]
fn refrac_extended(
    py: Python<'_>,
    altitude: f64,
    geoalt: f64,
    pressure_mb: f64,
    temp_c: f64,
    lapse_rate: f64,
    calc_flag: i32,
) -> PyObject {
    let (r, d) =
        celestial::refrac_extended(altitude, geoalt, pressure_mb, temp_c, lapse_rate, calc_flag);
    (r, d.to_vec()).into_py(py)
}

// ─── Info ─────────────────────────────────────────────────────────────────────

/// Swiss Ephemeris library version string.
#[pyfunction]
fn version() -> String {
    celestial::version()
}

/// Name of a planet / body.
#[pyfunction]
fn planet_name(planet: i32) -> &'static str {
    celestial::planet_name(Body::from_raw(planet))
}

// ─── Aspect matching ─────────────────────────────────────────────────────────

/// Check if two planets form an aspect. Returns `(diff, speed, factor, matched)`.
#[pyfunction]
#[pyo3(signature = (pos0, speed0, pos1, speed1, aspect, orb))]
fn match_aspect(
    py: Python<'_>,
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    orb: f64,
) -> PyObject {
    let r = celestial::match_aspect(pos0, speed0, pos1, speed1, aspect, orb);
    (r.diff, r.speed, r.factor, r.matched as i32).into_py(py)
}

/// Like match_aspect but aspect in [0,180].
#[pyfunction]
#[pyo3(signature = (pos0, speed0, pos1, speed1, aspect, orb))]
fn match_aspect2(
    py: Python<'_>,
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    orb: f64,
) -> PyObject {
    let r = celestial::match_aspect2(pos0, speed0, pos1, speed1, aspect, orb);
    (r.diff, r.speed, r.factor, r.matched as i32).into_py(py)
}

/// Aspect with applying/separating/stationary orbs. Returns `(diff, speed, factor, matched)`.
#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (pos0, speed0, pos1, speed1, aspect, app_orb, sep_orb, def_orb))]
fn match_aspect3(
    py: Python<'_>,
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    app_orb: f64,
    sep_orb: f64,
    def_orb: f64,
) -> PyObject {
    let r = celestial::match_aspect3(
        pos0, speed0, pos1, speed1, aspect, app_orb, sep_orb, def_orb,
    );
    (r.diff, r.speed, r.factor, r.matched as i32).into_py(py)
}

/// Like match_aspect3 but aspect in [0,180].
#[allow(clippy::too_many_arguments)]
#[pyfunction]
fn match_aspect4(
    py: Python<'_>,
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    app_orb: f64,
    sep_orb: f64,
    def_orb: f64,
) -> PyObject {
    let r = celestial::match_aspect4(
        pos0, speed0, pos1, speed1, aspect, app_orb, sep_orb, def_orb,
    );
    (r.diff, r.speed, r.factor, r.matched as i32).into_py(py)
}

/// Compute antiscion and contrantiscion. Returns `(antiscion[6], contrantiscion[6])`.
#[pyfunction]
fn antiscion(py: Python<'_>, pos: [f64; 6], axis: f64) -> PyObject {
    let a = celestial::antiscion(pos, axis);
    (a.antiscion.to_vec(), a.contrantiscion.to_vec()).into_py(py)
}

// ─── Datetime helpers ────────────────────────────────────────────────────────

/// Current Julian day (UTC).
#[pyfunction]
fn jdnow() -> f64 {
    celestial::jdnow()
}

/// Decompose JD to [year, month, day, hour, min, sec].
#[pyfunction]
fn revjul_hms(py: Python<'_>, jd: f64, calendar: i32) -> PyObject {
    celestial::revjul_hms(jd, Calendar::from(calendar))
        .to_vec()
        .into_py(py)
}

/// Parse ISO datetime string to [y,mo,d,h,mi,s].
#[pyfunction]
fn parse_datetime(py: Python<'_>, s: &str) -> PyObject {
    match celestial::parse_datetime(s) {
        Some(dt) => dt.to_vec().into_py(py),
        None => py.None(),
    }
}

/// Duration between two JDs → [days, hours, min, sec].
#[pyfunction]
fn jd_duration(py: Python<'_>, jd_start: f64, jd_end: f64) -> PyObject {
    celestial::jd_duration(jd_start, jd_end)
        .to_vec()
        .into_py(py)
}

/// Format JD as ISO string "YYYY-MM-DD HH:MM:SS UTC".
#[pyfunction]
fn jd_to_iso_string(jd: f64, calendar: i32) -> String {
    celestial::jd_to_iso_string(jd, Calendar::from(calendar))
}

// ─── Formatting ──────────────────────────────────────────────────────────────

/// Split ecliptic longitude → [deg_in_sign, sign_num, minutes, seconds].
#[pyfunction]
fn degsplit(py: Python<'_>, pos: f64) -> PyObject {
    celestial::degsplit(pos).to_vec().into_py(py)
}

/// English name of zodiac sign (0–11).
#[pyfunction]
fn sign_name(sign: i32) -> Option<&'static str> {
    celestial::sign_name(sign)
}

/// Parse geographic coordinate string to decimal degrees.
#[pyfunction]
fn parse_coord(s: &str) -> Option<f64> {
    celestial::parse_coord(s)
}

/// Format geographic coordinate as string.
#[pyfunction]
fn format_coord(coord: f64, is_latitude: bool) -> Option<String> {
    celestial::format_coord(coord, is_latitude)
}

// ─── Vedic helpers ───────────────────────────────────────────────────────────

/// Rasi (sign) number from ecliptic longitude.
#[pyfunction]
fn long_to_rasi(lon: f64) -> i32 {
    celestial::long_to_rasi(lon)
}
/// Navamsa from ecliptic longitude.
#[pyfunction]
fn long_to_navamsa(lon: f64) -> i32 {
    celestial::long_to_navamsa(lon)
}
/// Nakshatra and Pada from ecliptic longitude. Returns (nakshatra, pada).
#[pyfunction]
fn long_to_nakshatra(py: Python<'_>, lon: f64) -> PyObject {
    let (n, p) = celestial::long_to_nakshatra(lon);
    (n, p).into_py(py)
}
/// Nakshatra name from index.
#[pyfunction]
fn nakshatra_name(n: i32) -> Option<&'static str> {
    celestial::nakshatra_name(n)
}
/// Raman house cusps. Returns 12 longitude values.
#[pyfunction]
fn raman_houses(py: Python<'_>, asc: f64, mc: f64, sandhi: bool) -> PyObject {
    celestial::raman_houses(asc, mc, sandhi)
        .to_vec()
        .into_py(py)
}
/// Naisargika (permanent) relation between two planets. Returns 1/0/-1.
#[pyfunction]
fn naisargika_relation(gr1: i32, gr2: i32) -> Option<i32> {
    celestial::naisargika_relation(gr1, gr2)
}
/// Ochchabala (exaltation strength) in shashtiamsa.
#[pyfunction]
fn ochchabala(graha: i32, sputha: f64) -> Option<f64> {
    celestial::ochchabala(graha, sputha)
}
/// Residential strength in [0,1].
#[pyfunction]
fn residential_strength(py: Python<'_>, graha: f64, bm: [f64; 12]) -> PyObject {
    celestial::residential_strength(graha, &bm).into_py(py)
}
/// Saturn 4-Stars index (Halbronn). Returns [sat, ald, reg, ant, fom, index].
#[pyfunction]
fn saturn_4_stars(py: Python<'_>, jd: f64, flags: i32) -> PyResult<PyObject> {
    celestial::saturn_4_stars(jd, CalcFlags(flags))
        .map(|r| r.to_vec().into_py(py))
        .map_err(to_py)
}

// ─── Search functions ────────────────────────────────────────────────────────

/// Find next retrograde/direct station. Returns `(jd, pos[6])` or None.
#[pyfunction]
#[pyo3(signature = (planet, jd_start, backward=false, stop_days=0.0, flags=2|256))]
fn next_retro(
    py: Python<'_>,
    planet: i32,
    jd_start: f64,
    backward: bool,
    stop_days: f64,
    flags: i32,
) -> PyObject {
    match celestial::next_retro(
        Body::from_raw(planet),
        jd_start,
        backward,
        stop_days,
        CalcFlags(flags),
    ) {
        Some(r) => (r.jd, r.pos.to_vec()).into_py(py),
        None => py.None(),
    }
}

/// Find next exact aspect to a fixed point. Returns `(jd, pos[6])` or None.
#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (planet, aspect, fixed_pt, jd_start, backward=false, stop_days=0.0, flags=2))]
fn next_aspect(
    py: Python<'_>,
    planet: i32,
    aspect: f64,
    fixed_pt: f64,
    jd_start: f64,
    backward: bool,
    stop_days: f64,
    flags: i32,
) -> PyObject {
    match celestial::next_aspect(
        Body::from_raw(planet),
        aspect,
        fixed_pt,
        jd_start,
        backward,
        stop_days,
        CalcFlags(flags),
    ) {
        Some(r) => (r.jd, r.pos1.to_vec()).into_py(py),
        None => py.None(),
    }
}

/// Find next aspect between two moving planets. Returns `(jd, pos1[6], pos2[6])` or None.
#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (planet, aspect, other, jd_start, backward=false, stop_days=0.0, flags=2))]
fn next_aspect_with(
    py: Python<'_>,
    planet: i32,
    aspect: f64,
    other: i32,
    jd_start: f64,
    backward: bool,
    stop_days: f64,
    flags: i32,
) -> PyObject {
    match celestial::next_aspect_with(
        Body::from_raw(planet),
        aspect,
        Body(other),
        jd_start,
        backward,
        stop_days,
        CalcFlags(flags),
    ) {
        Some(r) => (r.jd, r.pos1.to_vec(), r.pos2.to_vec()).into_py(py),
        None => py.None(),
    }
}

// ─── Chart functions (added) ──────────────────────────────────────────────────

/// Next time a body ingresses into any zodiac sign after `jd_start`.
/// Returns `(jd, sign_number)` where sign is 0–11 (0=Aries).
/// Pass `backward=true` to find the previous ingress.
#[pyfunction]
#[pyo3(signature = (planet, jd, flags, backward=false))]
fn sign_ingress_ut(planet: i32, jd: f64, flags: i32, backward: bool) -> PyResult<(f64, u8)> {
    celestial::sign_ingress_ut(Body::from_raw(planet), jd, CalcFlags(flags), backward)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Find the next retrograde and direct stations for a body after `jd_start`.
/// Returns `(retrograde_jd, direct_jd)`.
#[pyfunction]
#[pyo3(signature = (planet, jd, flags))]
fn retrograde_station_ut(py: Python<'_>, planet: i32, jd: f64, flags: i32) -> PyResult<PyObject> {
    celestial::retrograde_station_ut(Body::from_raw(planet), jd, CalcFlags(flags))
        .map(|s| (s.retrograde, s.direct).into_py(py))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Arabic Part / Lot formula: `(asc + body2 - body1) mod 360`.
/// For Lot of Fortune: `arabic_part(asc, moon_lon, sun_lon)`.
#[pyfunction]
fn arabic_part(asc: f64, body2: f64, body1: f64) -> f64 {
    celestial::arabic_part(asc, body2, body1)
}

/// Next time a transiting body reaches `target_lon` degrees after `jd`.
/// Pass `backward=true` to search backwards in time.
#[pyfunction]
#[pyo3(signature = (planet, target_lon, jd, flags, backward=false))]
fn transit_to_degree(
    planet: i32,
    target_lon: f64,
    jd: f64,
    flags: i32,
    backward: bool,
) -> PyResult<f64> {
    celestial::transit_to_degree(
        Body::from_raw(planet),
        target_lon,
        jd,
        CalcFlags(flags),
        backward,
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

#[allow(clippy::too_many_arguments)]
/// Next time a body transits the natal MC angle.
/// Requires natal `mc`, geographic coordinates, and house system.
#[pyfunction]
#[pyo3(signature = (planet, jd_natal, jd_start, lat, lon, hsys, flags, backward=false))]
fn mc_transit_ut(
    planet: i32,
    jd_natal: f64,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: u32,
    flags: i32,
    backward: bool,
) -> PyResult<f64> {
    celestial::mc_transit_ut(
        Body::from_raw(planet),
        jd_natal,
        jd_start,
        lat,
        lon,
        HouseSystem(hsys as u8),
        CalcFlags(flags),
        backward,
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Julian day of the solar return in `return_year` closest to `jd_natal`.
/// The solar return is the moment transiting Sun returns to its natal longitude.
#[pyfunction]
fn solar_return_jd(jd_natal: f64, return_year: i32, flags: i32) -> PyResult<f64> {
    celestial::solar_return_jd(jd_natal, return_year, CalcFlags(flags))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Julian day of the next lunar return after `jd_start`.
/// The lunar return is when transiting Moon returns to its natal longitude.
#[pyfunction]
fn lunar_return_jd(jd_natal: f64, jd_start: f64, flags: i32) -> PyResult<f64> {
    celestial::lunar_return_jd(jd_natal, jd_start, CalcFlags(flags))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Shortest-arc midpoint between two ecliptic longitudes (degrees).
/// Always returns a value in [0, 360).
#[pyfunction]
fn midpoint(lon1: f64, lon2: f64) -> f64 {
    celestial::midpoint(lon1, lon2)
}

/// Traditional planetary ruler of a zodiac sign (0=Aries … 11=Pisces).
/// Aries→Mars, Taurus→Venus, Gemini→Mercury, Cancer→Moon, Leo→Sun,
/// Virgo→Mercury, Libra→Venus, Scorpio→Mars, Sagittarius→Jupiter,
/// Capricorn→Saturn, Aquarius→Saturn, Pisces→Jupiter.
#[pyfunction]
fn sign_ruler(sign: u8) -> i32 {
    celestial::sign_ruler(sign).as_raw()
}

/// Modern planetary ruler (Uranus→Aquarius, Neptune→Pisces, Pluto→Scorpio).
/// Falls back to traditional ruler for other signs.
#[pyfunction]
fn sign_ruler_modern(sign: u8) -> i32 {
    celestial::sign_ruler_modern(sign).as_raw()
}

/// English name of a zodiac sign (0=Aries … 11=Pisces). Wraps mod 12.
#[pyfunction]
fn zodiac_sign_name(sign: u8) -> &'static str {
    celestial::zodiac_sign_name(sign)
}

/// Convert ecliptic longitude (degrees) to `(sign, degrees_in_sign)`.
/// Sign is 0–11 (0=Aries), degrees_in_sign is 0.0–29.99.
#[pyfunction]
fn lon_to_sign(lon: f64) -> (u8, f64) {
    celestial::lon_to_sign(lon)
}

/// Local Apparent Solar Time in decimal hours for a given Julian day (UT)
/// and geographic longitude (degrees East positive).
#[pyfunction]
fn local_apparent_solar_time(jd_ut: f64, geolon_deg: f64) -> PyResult<f64> {
    celestial::local_apparent_solar_time(jd_ut, geolon_deg)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Annual profection house and degree for a given age.
/// Returns `(house_number, degree)` where house cycles through 1–12 yearly.
#[pyfunction]
fn annual_profection(py: Python<'_>, cusps: Vec<f64>, age: u32) -> PyResult<PyObject> {
    let arr: [f64; 13] = cusps
        .get(..13)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("cusps needs 13 elements")
        })?;
    let (house, degree) = celestial::annual_profection(&arr, age);
    Ok((house, degree).into_py(py))
}

/// Vimshottari dasha periods from birth Julian day and natal Moon longitude.
/// Returns a list of dasha levels with body, start JD, end JD, and years.
#[pyfunction]
#[pyo3(signature = (jd_birth, moon_lon_sidereal, years_ahead=120.0))]
fn vimshottari_dasha(
    py: Python<'_>,
    jd_birth: f64,
    moon_lon_sidereal: f64,
    years_ahead: f64,
) -> PyObject {
    let dashas = celestial::vimshottari_dasha(jd_birth, moon_lon_sidereal, years_ahead);
    let result: Vec<PyObject> = dashas
        .iter()
        .map(|d| (d.body.as_raw(), d.start, d.end, d.years).into_py(py))
        .collect();
    result.into_py(py)
}

// ─── Sefirat HaOmer ───────────────────────────────────────────────────────────

/// Return the Omer day for the given Julian day, or None if not in the Omer period.
///
/// Returns (day, week, day_of_week, week_sefirah, day_sefirah, hebrew_text, is_lag_baomer, jd)
#[pyfunction]
fn omer_from_jd(py: Python<'_>, jd: f64) -> PyObject {
    match celestial::omer_from_jd(jd) {
        Some(d) => (
            d.day as i32,
            d.week as i32,
            d.day_of_week as i32,
            d.week_sefirah,
            d.day_sefirah,
            d.hebrew_text,
            d.is_lag_baomer,
            d.jd,
        )
            .into_py(py),
        None => py.None(),
    }
}

/// Return the Julian day of a specific Omer day (1–49) in the given Hebrew year.
#[pyfunction]
fn omer_day_jd(hebrew_year: i32, day: i32) -> Option<f64> {
    celestial::omer_day_jd(hebrew_year, day as u8)
}

/// Return the Julian day of the first day of the Omer for the given Hebrew year.
#[pyfunction]
fn omer_start_jd(hebrew_year: i32) -> f64 {
    celestial::omer_start_jd(hebrew_year)
}

/// Return all 49 Omer days for the given Hebrew year.
///
/// Each element is (day, week, day_of_week, week_sefirah, day_sefirah, hebrew_text, is_lag_baomer, jd)
#[pyfunction]
fn omer_days(py: Python<'_>, hebrew_year: i32) -> PyObject {
    let result: Vec<PyObject> = celestial::omer_days(hebrew_year)
        .into_iter()
        .map(|d| {
            (
                d.day as i32,
                d.week as i32,
                d.day_of_week as i32,
                d.week_sefirah,
                d.day_sefirah,
                d.hebrew_text,
                d.is_lag_baomer,
                d.jd,
            )
                .into_py(py)
        })
        .collect();
    result.into_py(py)
}

/// Return the Omer period (start_jd, end_jd, hebrew_year) containing the given JD.
#[pyfunction]
fn omer_period(py: Python<'_>, jd: f64) -> PyObject {
    let p = celestial::omer_period(jd);
    (p.start_jd, p.end_jd, p.hebrew_year).into_py(py)
}

/// Return the full declaration string for the given Omer day (1–49).
#[pyfunction]
fn omer_declaration(day: i32) -> String {
    celestial::omer_declaration(day as u8)
}

// ─── Module registration ──────────────────────────────────────────────────────

/// Register the `celestial_py` Python extension module.
#[pymodule]
fn _celestial_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // ── functions ──────────────────────────────────────────────────────────
    m.add_function(wrap_pyfunction!(set_ephe_path, m)?)?;
    m.add_function(wrap_pyfunction!(set_jpl_file, m)?)?;
    m.add_function(wrap_pyfunction!(set_sid_mode, m)?)?;
    m.add_function(wrap_pyfunction!(set_topo, m)?)?;
    m.add_function(wrap_pyfunction!(set_delta_t_userdef, m)?)?;
    m.add_function(wrap_pyfunction!(close, m)?)?;

    m.add_function(wrap_pyfunction!(calc, m)?)?;
    m.add_function(wrap_pyfunction!(calc_ut, m)?)?;
    m.add_function(wrap_pyfunction!(nutation, m)?)?;
    m.add_function(wrap_pyfunction!(mean_obliquity, m)?)?;
    m.add_function(wrap_pyfunction!(true_obliquity, m)?)?;
    m.add_function(wrap_pyfunction!(calc_many, m)?)?;
    m.add_function(wrap_pyfunction!(calc_ut_many, m)?)?;
    m.add_function(wrap_pyfunction!(calc_pctr, m)?)?;
    m.add_function(wrap_pyfunction!(fixstar, m)?)?;
    m.add_function(wrap_pyfunction!(fixstar_ut, m)?)?;
    m.add_function(wrap_pyfunction!(fixstar2, m)?)?;
    m.add_function(wrap_pyfunction!(fixstar2_ut, m)?)?;
    m.add_function(wrap_pyfunction!(fixstar_mag, m)?)?;
    m.add_function(wrap_pyfunction!(fixstar2_mag, m)?)?;

    m.add_function(wrap_pyfunction!(houses, m)?)?;
    m.add_function(wrap_pyfunction!(houses_ex, m)?)?;
    m.add_function(wrap_pyfunction!(houses_ex2, m)?)?;
    m.add_function(wrap_pyfunction!(house_pos, m)?)?;
    m.add_function(wrap_pyfunction!(house_name, m)?)?;

    m.add_function(wrap_pyfunction!(sol_eclipse_when_glob, m)?)?;
    m.add_function(wrap_pyfunction!(sol_eclipse_when_loc, m)?)?;
    m.add_function(wrap_pyfunction!(sol_eclipse_how, m)?)?;
    m.add_function(wrap_pyfunction!(sol_eclipse_where, m)?)?;
    m.add_function(wrap_pyfunction!(lun_eclipse_when, m)?)?;
    m.add_function(wrap_pyfunction!(lun_eclipse_when_loc, m)?)?;
    m.add_function(wrap_pyfunction!(lun_eclipse_how, m)?)?;

    m.add_function(wrap_pyfunction!(rise_trans, m)?)?;

    m.add_function(wrap_pyfunction!(julday, m)?)?;
    m.add_function(wrap_pyfunction!(revjul, m)?)?;
    m.add_function(wrap_pyfunction!(day_of_week, m)?)?;
    m.add_function(wrap_pyfunction!(deltat, m)?)?;
    m.add_function(wrap_pyfunction!(sidtime, m)?)?;
    m.add_function(wrap_pyfunction!(mean_sidtime, m)?)?;
    m.add_function(wrap_pyfunction!(utc_to_jd, m)?)?;

    m.add_function(wrap_pyfunction!(ayanamsa, m)?)?;
    m.add_function(wrap_pyfunction!(ayanamsa_ut, m)?)?;
    m.add_function(wrap_pyfunction!(ayanamsa_name, m)?)?;

    m.add_function(wrap_pyfunction!(norm_deg, m)?)?;
    m.add_function(wrap_pyfunction!(midpoint_deg, m)?)?;
    m.add_function(wrap_pyfunction!(diff_deg_signed, m)?)?;
    m.add_function(wrap_pyfunction!(norm_cs, m)?)?;
    m.add_function(wrap_pyfunction!(split_deg, m)?)?;
    m.add_function(wrap_pyfunction!(coord_transform, m)?)?;
    m.add_function(wrap_pyfunction!(azalt, m)?)?;
    m.add_function(wrap_pyfunction!(azalt_rev, m)?)?;
    m.add_function(wrap_pyfunction!(refrac, m)?)?;
    m.add_function(wrap_pyfunction!(refrac_extended, m)?)?;

    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(planet_name, m)?)?;

    // ── constants ──────────────────────────────────────────────────────────
    m.add("GREG_CAL", celestial::GREG_CAL)?;
    m.add("JUL_CAL", celestial::JUL_CAL)?;
    m.add("SUN", celestial::SUN)?;
    m.add("MOON", celestial::MOON)?;
    m.add("MERCURY", celestial::MERCURY)?;
    m.add("VENUS", celestial::VENUS)?;
    m.add("MARS", celestial::MARS)?;
    m.add("JUPITER", celestial::JUPITER)?;
    m.add("SATURN", celestial::SATURN)?;
    m.add("URANUS", celestial::URANUS)?;
    m.add("NEPTUNE", celestial::NEPTUNE)?;
    m.add("PLUTO", celestial::PLUTO)?;
    m.add("MEAN_NODE", celestial::MEAN_NODE)?;
    m.add("TRUE_NODE", celestial::TRUE_NODE)?;
    m.add("CHIRON", celestial::CHIRON)?;
    m.add("EARTH", Body::EARTH.as_raw())?;

    m.add("FLG_BUILTIN", celestial::FLG_BUILTIN)?;
    m.add("FLG_JPL", celestial::FLG_JPL)?;
    m.add("FLG_MOSHIER", celestial::FLG_MOSHIER)?;
    m.add("FLG_SPEED", celestial::FLG_SPEED)?;
    m.add("FLG_EQUATORIAL", celestial::FLG_EQUATORIAL)?;
    m.add("FLG_TOPOCTR", celestial::FLG_TOPOCTR)?;
    m.add("FLG_SIDEREAL", celestial::FLG_SIDEREAL)?;
    m.add("FLG_HELCTR", celestial::FLG_HELCTR)?;
    m.add("FLG_XYZ", 4096i32)?;
    m.add("FLG_RADIANS", celestial::FLG_RADIANS)?;
    m.add("FLG_NONUT", celestial::FLG_NONUT)?;
    m.add("FLG_NOABERR", celestial::FLG_NOABERR)?;
    m.add("FLG_NOGDEFL", celestial::FLG_NOGDEFL)?;

    m.add("SIDM_FAGAN_BRADLEY", celestial::SIDM_FAGAN_BRADLEY)?;
    m.add("SIDM_LAHIRI", celestial::SIDM_LAHIRI)?;
    m.add("SIDM_RAMAN", celestial::SIDM_RAMAN)?;
    m.add("SIDM_USER", 255i32)?;

    m.add("ECL_TOTAL", celestial::ECL_TOTAL)?;
    m.add("ECL_ANNULAR", celestial::ECL_ANNULAR)?;
    m.add("ECL_PARTIAL", celestial::ECL_PARTIAL)?;
    m.add("ECL_CENTRAL", celestial::ECL_CENTRAL)?;
    m.add("ECL_PENUMBRAL", celestial::ECL_PENUMBRAL)?;

    m.add("CALC_RISE", celestial::CALC_RISE)?;
    m.add("CALC_SET", celestial::CALC_SET)?;

    m.add("TRUE_TO_APP", celestial::TRUE_TO_APP)?;
    m.add("APP_TO_TRUE", celestial::APP_TO_TRUE)?;

    m.add("SPLIT_DEG_ROUND_SEC", celestial::SPLIT_DEG_ROUND_SEC)?;
    m.add("SPLIT_DEG_ROUND_MIN", celestial::SPLIT_DEG_ROUND_MIN)?;
    m.add("SPLIT_DEG_ROUND_DEG", celestial::SPLIT_DEG_ROUND_DEG)?;
    m.add("SPLIT_DEG_ZODIACAL", celestial::SPLIT_DEG_ZODIACAL)?;
    m.add("SPLIT_DEG_NAKSHATRA", celestial::SPLIT_DEG_NAKSHATRA)?;

    m.add_function(wrap_pyfunction!(match_aspect, m)?)?;
    m.add_function(wrap_pyfunction!(match_aspect2, m)?)?;
    m.add_function(wrap_pyfunction!(match_aspect3, m)?)?;
    m.add_function(wrap_pyfunction!(match_aspect4, m)?)?;
    m.add_function(wrap_pyfunction!(antiscion, m)?)?;
    m.add_function(wrap_pyfunction!(jdnow, m)?)?;
    m.add_function(wrap_pyfunction!(revjul_hms, m)?)?;
    m.add_function(wrap_pyfunction!(parse_datetime, m)?)?;
    m.add_function(wrap_pyfunction!(jd_duration, m)?)?;
    m.add_function(wrap_pyfunction!(jd_to_iso_string, m)?)?;
    m.add_function(wrap_pyfunction!(degsplit, m)?)?;
    m.add_function(wrap_pyfunction!(sign_name, m)?)?;
    m.add_function(wrap_pyfunction!(parse_coord, m)?)?;
    m.add_function(wrap_pyfunction!(format_coord, m)?)?;
    m.add_function(wrap_pyfunction!(long_to_rasi, m)?)?;
    m.add_function(wrap_pyfunction!(long_to_navamsa, m)?)?;
    m.add_function(wrap_pyfunction!(long_to_nakshatra, m)?)?;
    m.add_function(wrap_pyfunction!(nakshatra_name, m)?)?;
    m.add_function(wrap_pyfunction!(raman_houses, m)?)?;
    m.add_function(wrap_pyfunction!(naisargika_relation, m)?)?;
    m.add_function(wrap_pyfunction!(ochchabala, m)?)?;
    m.add_function(wrap_pyfunction!(residential_strength, m)?)?;
    m.add_function(wrap_pyfunction!(saturn_4_stars, m)?)?;
    m.add_function(wrap_pyfunction!(next_retro, m)?)?;
    m.add_function(wrap_pyfunction!(next_aspect, m)?)?;
    m.add_function(wrap_pyfunction!(next_aspect_with, m)?)?;
    m.add_function(wrap_pyfunction!(sign_ingress_ut, m)?)?;
    m.add_function(wrap_pyfunction!(retrograde_station_ut, m)?)?;
    m.add_function(wrap_pyfunction!(arabic_part, m)?)?;
    m.add_function(wrap_pyfunction!(transit_to_degree, m)?)?;
    m.add_function(wrap_pyfunction!(mc_transit_ut, m)?)?;
    m.add_function(wrap_pyfunction!(solar_return_jd, m)?)?;
    m.add_function(wrap_pyfunction!(lunar_return_jd, m)?)?;
    m.add_function(wrap_pyfunction!(midpoint, m)?)?;
    m.add_function(wrap_pyfunction!(sign_ruler, m)?)?;
    m.add_function(wrap_pyfunction!(sign_ruler_modern, m)?)?;
    m.add_function(wrap_pyfunction!(zodiac_sign_name, m)?)?;
    m.add_function(wrap_pyfunction!(lon_to_sign, m)?)?;
    m.add_function(wrap_pyfunction!(local_apparent_solar_time, m)?)?;
    m.add_function(wrap_pyfunction!(annual_profection, m)?)?;
    m.add_function(wrap_pyfunction!(vimshottari_dasha, m)?)?;
    m.add_function(wrap_pyfunction!(omer_from_jd, m)?)?;
    m.add_function(wrap_pyfunction!(omer_day_jd, m)?)?;
    m.add_function(wrap_pyfunction!(omer_start_jd, m)?)?;
    m.add_function(wrap_pyfunction!(omer_days, m)?)?;
    m.add_function(wrap_pyfunction!(omer_period, m)?)?;
    m.add_function(wrap_pyfunction!(omer_declaration, m)?)?;
    m.add_function(wrap_pyfunction!(jewish_holidays, m)?)?;
    m.add_function(wrap_pyfunction!(jewish_holiday_jd, m)?)?;
    m.add_function(wrap_pyfunction!(hebrew_year_from_jd, m)?)?;
    m.add_function(wrap_pyfunction!(jd_to_hebrew_date, m)?)?;
    m.add_function(wrap_pyfunction!(easter_gregorian, m)?)?;
    m.add_function(wrap_pyfunction!(easter_orthodox, m)?)?;
    m.add_function(wrap_pyfunction!(easter_jd, m)?)?;
    m.add_function(wrap_pyfunction!(easter_orthodox_jd, m)?)?;
    m.add_function(wrap_pyfunction!(christian_feasts, m)?)?;
    m.add_function(wrap_pyfunction!(christian_fixed_feasts, m)?)?;
    m.add_function(wrap_pyfunction!(hijri_from_jd, m)?)?;
    m.add_function(wrap_pyfunction!(hijri_to_jd, m)?)?;
    m.add_function(wrap_pyfunction!(hijri_month_name, m)?)?;
    m.add_function(wrap_pyfunction!(islamic_observances, m)?)?;
    m.add_function(wrap_pyfunction!(gregorian_to_hijri_years, m)?)?;
    m.add_function(wrap_pyfunction!(panchanga, m)?)?;
    m.add_function(wrap_pyfunction!(hindu_festivals, m)?)?;
    m.add_function(wrap_pyfunction!(vesak_jd, m)?)?;
    m.add_function(wrap_pyfunction!(uposatha_days, m)?)?;
    m.add_function(wrap_pyfunction!(nowruz_jd, m)?)?;
    m.add_function(wrap_pyfunction!(gregorian_to_solar_hijri, m)?)?;
    m.add_function(wrap_pyfunction!(naw_ruz_jd, m)?)?;
    m.add_function(wrap_pyfunction!(jd_to_bahai, m)?)?;
    m.add_function(wrap_pyfunction!(bahai_holy_days, m)?)?;

    // ─── Moon phases ──────────────────────────────────────────────────────────────

    /// Current Moon phase at the given JD.
    /// Returns the phase name string.
    #[pyfunction]
    fn moon_phase(jd: f64) -> PyResult<String> {
        celestial::moon_phase(jd)
            .map(|p| p.name().to_string())
            .map_err(to_py)
    }

    /// Fraction of the Moon's disk illuminated (0.0–1.0).
    #[pyfunction]
    fn moon_illumination(jd: f64) -> PyResult<f64> {
        celestial::moon_illumination(jd).map_err(to_py)
    }

    /// Moon–Sun elongation in degrees (0°–360°).
    #[pyfunction]
    fn moon_elongation(jd: f64) -> PyResult<f64> {
        celestial::moon_elongation(jd).map_err(to_py)
    }

    /// Phase angle in degrees (0° = new, 180° = full).
    #[pyfunction]
    fn moon_phase_angle(jd: f64) -> PyResult<f64> {
        celestial::moon_phase_angle(jd).map_err(to_py)
    }

    /// JD of the next new moon at or after `jd_from`.
    #[pyfunction]
    fn next_new_moon(jd_from: f64) -> PyResult<f64> {
        celestial::next_new_moon(jd_from).map_err(to_py)
    }

    /// JD of the next first-quarter moon at or after `jd_from`.
    #[pyfunction]
    fn next_first_quarter(jd_from: f64) -> PyResult<f64> {
        celestial::next_first_quarter(jd_from).map_err(to_py)
    }

    /// JD of the next full moon at or after `jd_from`.
    #[pyfunction]
    fn next_full_moon_phase(jd_from: f64) -> PyResult<f64> {
        celestial::next_full_moon_phase(jd_from).map_err(to_py)
    }

    /// JD of the next last-quarter moon at or after `jd_from`.
    #[pyfunction]
    fn next_last_quarter(jd_from: f64) -> PyResult<f64> {
        celestial::next_last_quarter(jd_from).map_err(to_py)
    }

    /// All 4 principal phase events for a calendar month.
    /// Each item: (phase_name, jd, elongation)
    #[pyfunction]
    fn moon_phases_for_month(py: Python<'_>, year: i32, month: i32) -> PyResult<PyObject> {
        let events = celestial::moon_phases_for_month(year, month as u8).map_err(to_py)?;
        let result: Vec<PyObject> = events
            .into_iter()
            .map(|e| (e.phase.name(), e.jd, e.elongation).into_py(py))
            .collect();
        Ok(result.into_py(py))
    }

    /// Full Moon phase info: phase, illumination, prev/next principal phase.
    /// Returns (phase_name, elongation, illumination,
    ///          prev_phase_name, prev_phase_jd,
    ///          next_phase_name, next_phase_jd, age_days)
    #[pyfunction]
    fn moon_phase_info(py: Python<'_>, jd: f64) -> PyResult<PyObject> {
        let info = celestial::moon_phase_info(jd).map_err(to_py)?;
        Ok((
            info.phase_name,
            info.elongation,
            info.illumination,
            info.prev_phase_name,
            info.prev_phase_jd,
            info.next_phase_name,
            info.next_phase_jd,
            info.age_days,
        )
            .into_py(py))
    }

    m.add_function(wrap_pyfunction!(moon_phase, m)?)?;
    m.add_function(wrap_pyfunction!(moon_illumination, m)?)?;
    m.add_function(wrap_pyfunction!(moon_elongation, m)?)?;
    m.add_function(wrap_pyfunction!(moon_phase_angle, m)?)?;
    m.add_function(wrap_pyfunction!(next_new_moon, m)?)?;
    m.add_function(wrap_pyfunction!(next_first_quarter, m)?)?;
    m.add_function(wrap_pyfunction!(next_full_moon_phase, m)?)?;
    m.add_function(wrap_pyfunction!(next_last_quarter, m)?)?;
    m.add_function(wrap_pyfunction!(moon_phases_for_month, m)?)?;
    m.add_function(wrap_pyfunction!(moon_phase_info, m)?)?;
    Ok(())
}

// ─── Jewish holidays ──────────────────────────────────────────────────────────

/// All major Jewish holidays for the given Hebrew year.
/// Each item: (name, hebrew_name, hebrew_month, hebrew_day, jd, jd_end, days, category)
#[pyfunction]
fn jewish_holidays(py: Python<'_>, hebrew_year: i32) -> PyObject {
    let result: Vec<PyObject> = celestial::jewish_holidays(hebrew_year)
        .into_iter()
        .map(|h| {
            let cat = format!("{:?}", h.category);
            (
                h.name,
                h.hebrew_name,
                h.hebrew_month as i32,
                h.hebrew_day as i32,
                h.jd,
                h.jd_end,
                h.days as i32,
                cat,
            )
                .into_py(py)
        })
        .collect();
    result.into_py(py)
}

/// JD of a specific Jewish holiday by name in the given Hebrew year.
#[pyfunction]
fn jewish_holiday_jd(hebrew_year: i32, name: &str) -> Option<f64> {
    celestial::jewish_holiday_jd(hebrew_year, name)
}

/// Hebrew year for a given Julian day.
#[pyfunction]
fn hebrew_year_from_jd(jd: f64) -> i32 {
    celestial::hebrew_year_from_jd(jd)
}

/// Convert JD to Hebrew date → (year, month, day).
#[pyfunction]
fn jd_to_hebrew_date(py: Python<'_>, jd: f64) -> PyObject {
    let (y, m, d) = celestial::jd_to_hebrew_date(jd);
    (y, m as i32, d as i32).into_py(py)
}

// ─── Easter & Christian calendar ─────────────────────────────────────────────

/// Gregorian (Western) Easter → (year, month, day).
#[pyfunction]
fn easter_gregorian(py: Python<'_>, year: i32) -> PyObject {
    let (y, m, d) = celestial::easter_gregorian(year);
    (y, m as i32, d as i32).into_py(py)
}

/// Orthodox Easter in Gregorian calendar → (year, month, day).
#[pyfunction]
fn easter_orthodox(py: Python<'_>, year: i32) -> PyObject {
    let (y, m, d) = celestial::easter_orthodox(year);
    (y, m as i32, d as i32).into_py(py)
}

/// JD of Western Easter.
#[pyfunction]
fn easter_jd(year: i32) -> f64 {
    celestial::easter_jd(year)
}

/// JD of Orthodox Easter.
#[pyfunction]
fn easter_orthodox_jd(year: i32) -> f64 {
    celestial::easter_orthodox_jd(year)
}

/// All Western Christian moveable feasts for the year.
/// Each item: (name, easter_offset, jd, year, month, day)
#[pyfunction]
fn christian_feasts(py: Python<'_>, year: i32) -> PyObject {
    let result: Vec<PyObject> = celestial::christian_feasts(year)
        .into_iter()
        .map(|f| {
            (
                f.name,
                f.easter_offset,
                f.jd,
                f.year,
                f.month as i32,
                f.day as i32,
            )
                .into_py(py)
        })
        .collect();
    result.into_py(py)
}

/// Fixed (non-moveable) Christian feasts for the year.
#[pyfunction]
fn christian_fixed_feasts(py: Python<'_>, year: i32) -> PyObject {
    let result: Vec<PyObject> = celestial::christian_fixed_feasts(year)
        .into_iter()
        .map(|f| (f.name, f.jd, f.year, f.month as i32, f.day as i32).into_py(py))
        .collect();
    result.into_py(py)
}

// ─── Islamic calendar ─────────────────────────────────────────────────────────

/// Convert JD to Hijri date → (year, month, day).
#[pyfunction]
fn hijri_from_jd(py: Python<'_>, jd: f64) -> PyObject {
    let (y, m, d) = celestial::hijri_from_jd(jd);
    (y, m as i32, d as i32).into_py(py)
}

/// Convert Hijri date to JD.
#[pyfunction]
fn hijri_to_jd(year: i32, month: i32, day: i32) -> f64 {
    celestial::hijri_to_jd(year, month as u8, day as u8)
}

/// Hijri month name (1–12).
#[pyfunction]
fn hijri_month_name(month: i32) -> &'static str {
    celestial::hijri_month_name(month as u8)
}

/// All major Islamic observances for the given Hijri year.
/// Each item: (name, arabic_name, hijri_month, hijri_day, jd, days)
#[pyfunction]
fn islamic_observances(py: Python<'_>, hijri_year: i32) -> PyObject {
    let result: Vec<PyObject> = celestial::islamic_observances(hijri_year)
        .into_iter()
        .map(|o| {
            (
                o.name,
                o.arabic_name,
                o.hijri_month as i32,
                o.hijri_day as i32,
                o.jd,
                o.days as i32,
            )
                .into_py(py)
        })
        .collect();
    result.into_py(py)
}

/// Gregorian year → overlapping Hijri years → (year1, year2).
#[pyfunction]
fn gregorian_to_hijri_years(py: Python<'_>, gregorian_year: i32) -> PyObject {
    let (y1, y2) = celestial::gregorian_to_hijri_years(gregorian_year);
    (y1, y2).into_py(py)
}

// ─── Hindu Panchānga ─────────────────────────────────────────────────────────

/// Full Panchānga for a Julian day.
/// Returns (tithi, tithi_name, paksha, vara, vara_name,
///          nakshatra, nakshatra_name, nakshatra_pada,
///          yoga, yoga_name, karana, karana_name,
///          sun_lon, moon_lon, elongation)
#[pyfunction]
fn panchanga(py: Python<'_>, jd: f64) -> PyObject {
    let p = celestial::panchanga(jd);
    let paksha = format!("{:?}", p.paksha);
    // Split into two tuples to avoid PyO3 15-element limit
    let part1 = (
        p.tithi as i32,
        p.tithi_name,
        paksha,
        p.vara as i32,
        p.vara_name,
        p.nakshatra as i32,
        p.nakshatra_name,
        p.nakshatra_pada as i32,
    );
    let part2 = (
        p.yoga as i32,
        p.yoga_name,
        p.karana as i32,
        p.karana_name,
        p.sun_lon,
        p.moon_lon,
        p.elongation,
    );
    (part1, part2).into_py(py)
}

/// Major Hindu festivals in the given Gregorian year.
/// Each item: (name, description, jd)
#[pyfunction]
fn hindu_festivals(py: Python<'_>, gregorian_year: i32) -> PyObject {
    let result: Vec<PyObject> = celestial::hindu_festivals(gregorian_year)
        .into_iter()
        .map(|f| (f.name, f.description, f.jd).into_py(py))
        .collect();
    result.into_py(py)
}

// ─── Buddhist observances ─────────────────────────────────────────────────────

/// JD of Vesak for the given Gregorian year.
#[pyfunction]
fn vesak_jd(year: i32) -> f64 {
    celestial::vesak_jd(year)
}

/// All Uposatha days in the given Gregorian year.
/// Each item: (phase, jd, elongation)
#[pyfunction]
fn uposatha_days(py: Python<'_>, year: i32) -> PyObject {
    let result: Vec<PyObject> = celestial::uposatha_days(year)
        .into_iter()
        .map(|u| {
            let phase = format!("{:?}", u.phase);
            (phase, u.jd, u.elongation).into_py(py)
        })
        .collect();
    result.into_py(py)
}

// ─── Nowruz & Bahá'í calendar ─────────────────────────────────────────────────

/// JD of Nowruz (vernal equinox / Persian New Year) for the given Gregorian year.
#[pyfunction]
fn nowruz_jd(year: i32) -> f64 {
    celestial::nowruz_jd(year)
}

/// Convert Gregorian year to Solar Hijri (Persian) year.
#[pyfunction]
fn gregorian_to_solar_hijri(year: i32) -> i32 {
    celestial::gregorian_to_solar_hijri(year)
}

/// JD of Naw-Rúz (Bahá'í New Year) for the given Bahá'í year.
#[pyfunction]
fn naw_ruz_jd(bahai_year: i32) -> f64 {
    celestial::naw_ruz_jd(bahai_year)
}

/// Convert JD to Bahá'í date → (year, month, day, month_name).
#[pyfunction]
fn jd_to_bahai(py: Python<'_>, jd: f64) -> PyObject {
    let b = celestial::jd_to_bahai(jd);
    (b.year, b.month as i32, b.day as i32, b.month_name).into_py(py)
}

/// Bahá'í holy days for the given Bahá'í year.
/// Each item: (name, description, bahai_month, bahai_day, jd)
#[pyfunction]
fn bahai_holy_days(py: Python<'_>, bahai_year: i32) -> PyObject {
    let result: Vec<PyObject> = celestial::bahai_holy_days(bahai_year)
        .into_iter()
        .map(|h| {
            (
                h.name,
                h.description,
                h.bahai_month as i32,
                h.bahai_day as i32,
                h.jd,
            )
                .into_py(py)
        })
        .collect();
    result.into_py(py)
}
