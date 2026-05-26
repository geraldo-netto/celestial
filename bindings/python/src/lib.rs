//! Python bindings for celestial-core via PyO3.
//!
//! Build with `maturin develop` (dev install) or `maturin build` (wheel).
//! The module is exposed as `celestial_py`.

use celestial::Longitude;
use celestial::Latitude;
use celestial::JulianDay;
use celestial::Degrees;
use celestial::body::{Body, CalcFlags, Calendar, HouseSystem, SiderealMode};
use celestial_ffi as celestial;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
// pyo3 0.24 deprecated the `IntoPy::into_py(py) -> PyObject` conversion in
// favour of `IntoPyObject` + `IntoPyObjectExt::into_py_any`. The latter is
// importable from `IntoPyObjectExt` and returns `PyResult<PyObject>`.
// Conversions used in this crate (primitives, tuples of primitives, Vec,
// Option, &str, etc.) are all infallible so `.unwrap()` never panics.
use pyo3::IntoPyObjectExt;

// ─── Error mapping ────────────────────────────────────────────────────────────

fn to_py(e: celestial::Error) -> PyErr {
    PyRuntimeError::new_err(celestial_ffi::FfiError::from(e).message())
}

/// Validate-then-construct a [`Body`] at the FFI seam (REL-8 guard).
///
/// Rejects ids outside every documented range so bad input fails fast with
/// a clean `ValueError`-shaped message instead of an internal calc failure.
fn body_of(n: i32) -> PyResult<Body> {
    Body::try_from_raw(n).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
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
    celestial::set_topo(Longitude::new(geolon), Latitude::new(geolat), geoalt);
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
    let pos = celestial::calc(JulianDay::new(tjdet), body_of(planet)?, CalcFlags(flags)).map_err(to_py)?;
    let xx = celestial_ffi::pos6_tuple(&pos);
    Ok((xx, pos.ret_flags).into_py_any(py).unwrap())
}

/// Calculate planetary positions (UT).
#[pyfunction]
#[pyo3(signature = (tjdut, planet, flags = 258))]
fn calc_ut(py: Python<'_>, tjdut: f64, planet: i32, flags: i32) -> PyResult<PyObject> {
    let pos = celestial::calc_ut(JulianDay::new(tjdut), body_of(planet)?, CalcFlags(flags)).map_err(to_py)?;
    let xx = celestial_ffi::pos6_tuple(&pos);
    Ok((xx, pos.ret_flags).into_py_any(py).unwrap())
}

/// Nutation in longitude and obliquity at a JDE (TT).
///
/// Returns `(dpsi_degrees, deps_degrees)` — both in degrees.
/// Multiply by 3600 to convert to arcseconds.
/// Uses the IAU 2000B luni-solar series (77 terms, ~1 mas accuracy).
#[pyfunction]
#[pyo3(signature = (jde))]
fn nutation(jde: f64) -> (f64, f64) {
    celestial::nutation(JulianDay::new(jde))
}

/// Mean obliquity of the ecliptic in degrees (IAU 2006 formula).
#[pyfunction]
#[pyo3(signature = (jde))]
fn mean_obliquity(jde: f64) -> f64 {
    celestial::mean_obliquity(JulianDay::new(jde))
}

/// True (apparent) obliquity of the ecliptic in degrees.
/// Equals mean obliquity + nutation in obliquity.
#[pyfunction]
#[pyo3(signature = (jde))]
fn true_obliquity(jde: f64) -> f64 {
    celestial::true_obliquity(JulianDay::new(jde))
}

/// Calculate positions for a list of bodies in parallel (ET / TT input).
///
/// Returns a list of ((lon, lat, dist, speed_lon, speed_lat, speed_dist), ret_flags)
/// tuples in the same order as `planets`.
#[pyfunction]
#[pyo3(signature = (tjdet, planets, flags = 258))]
fn calc_many(py: Python<'_>, tjdet: f64, planets: Vec<i32>, flags: i32) -> PyResult<PyObject> {
    let bodies: Vec<Body> = planets
        .iter()
        .map(|&p| body_of(p))
        .collect::<PyResult<Vec<_>>>()?;
    let results = celestial::calc_many(JulianDay::new(tjdet), &bodies, CalcFlags(flags));
    let list: Vec<PyObject> = results
        .into_iter()
        .map(|r| {
            let pos = r.map_err(to_py)?;
            let xx = celestial_ffi::pos6_tuple(&pos);
            Ok::<PyObject, pyo3::PyErr>((xx, pos.ret_flags).into_py_any(py).unwrap())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(list.into_py_any(py).unwrap())
}

/// Calculate positions for a list of bodies in parallel (UT input).
#[pyfunction]
#[pyo3(signature = (tjdut, planets, flags = 258))]
fn calc_ut_many(py: Python<'_>, tjdut: f64, planets: Vec<i32>, flags: i32) -> PyResult<PyObject> {
    let bodies: Vec<Body> = planets
        .iter()
        .map(|&p| body_of(p))
        .collect::<PyResult<Vec<_>>>()?;
    let results = celestial::calc_ut_many(JulianDay::new(tjdut), &bodies, CalcFlags(flags));
    let list: Vec<PyObject> = results
        .into_iter()
        .map(|r| {
            let pos = r.map_err(to_py)?;
            let xx = celestial_ffi::pos6_tuple(&pos);
            Ok::<PyObject, pyo3::PyErr>((xx, pos.ret_flags).into_py_any(py).unwrap())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(list.into_py_any(py).unwrap())
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
        JulianDay::new(tjdet),
        body_of(planet)?,
        body_of(center)?,
        CalcFlags(flags),
    )
    .map_err(to_py)?;
    let xx = celestial_ffi::pos6_tuple(&pos);
    Ok((xx, pos.ret_flags).into_py_any(py).unwrap())
}

/// Calculate a fixed star position (ET). Returns `((xx), name, ret_flags)`.
#[pyfunction]
#[pyo3(signature = (star, tjdet, flags = 2))]
fn fixstar(py: Python<'_>, star: &str, tjdet: f64, flags: i32) -> PyResult<PyObject> {
    let r = celestial::fixstar(star, JulianDay::new(tjdet), CalcFlags(flags)).map_err(to_py)?;
    let xx = PyObject::from(pyo3::types::PyTuple::new(py, r.xx).unwrap());
    Ok((xx, r.star_name, r.ret_flags).into_py_any(py).unwrap())
}

/// Fixed star position (UT).
#[pyfunction]
#[pyo3(signature = (star, tjdut, flags = 2))]
fn fixstar_ut(py: Python<'_>, star: &str, tjdut: f64, flags: i32) -> PyResult<PyObject> {
    let r = celestial::fixstar_ut(star, JulianDay::new(tjdut), CalcFlags(flags)).map_err(to_py)?;
    let xx = PyObject::from(pyo3::types::PyTuple::new(py, r.xx).unwrap());
    Ok((xx, r.star_name, r.ret_flags).into_py_any(py).unwrap())
}

/// Faster fixed star position (ET).
#[pyfunction]
#[pyo3(signature = (star, tjdet, flags = 2))]
fn fixstar2(py: Python<'_>, star: &str, tjdet: f64, flags: i32) -> PyResult<PyObject> {
    let r = celestial::fixstar2(star, JulianDay::new(tjdet), CalcFlags(flags)).map_err(to_py)?;
    let xx = PyObject::from(pyo3::types::PyTuple::new(py, r.xx).unwrap());
    Ok((xx, r.star_name, r.ret_flags).into_py_any(py).unwrap())
}

/// Faster fixed star position (UT).
#[pyfunction]
#[pyo3(signature = (star, tjdut, flags = 2))]
fn fixstar2_ut(py: Python<'_>, star: &str, tjdut: f64, flags: i32) -> PyResult<PyObject> {
    let r = celestial::fixstar2_ut(star, JulianDay::new(tjdut), CalcFlags(flags)).map_err(to_py)?;
    let xx = PyObject::from(pyo3::types::PyTuple::new(py, r.xx).unwrap());
    Ok((xx, r.star_name, r.ret_flags).into_py_any(py).unwrap())
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
    let r = celestial::houses(JulianDay::new(tjdut), Latitude::new(lat), Longitude::new(lon), HouseSystem(hsys)).map_err(to_py)?;
    // cusps[0] is unused in SE convention; return cusps[1..=12] (12 real cusps)
    // ascmc[0..8] is the SE standard (skip our extra IC/DSC at [8],[9])
    let cusps: Vec<f64> = r.cusps[1..].to_vec();
    let ascmc: Vec<f64> = r.ascmc[..8].to_vec();
    Ok((cusps, ascmc).into_py_any(py).unwrap())
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
    let r = celestial::houses_ex(JulianDay::new(tjdut), CalcFlags(flags), Latitude::new(lat), Longitude::new(lon), HouseSystem(hsys))
        .map_err(to_py)?;
    let cusps: Vec<f64> = r.cusps[1..].to_vec();
    let ascmc: Vec<f64> = r.ascmc[..8].to_vec();
    Ok((cusps, ascmc).into_py_any(py).unwrap())
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
    let r = celestial::houses_ex2(JulianDay::new(tjdut), CalcFlags(flags), Latitude::new(lat), Longitude::new(lon), HouseSystem(hsys))
        .map_err(to_py)?;
    Ok((
        r.cusps[1..].to_vec(),
        r.ascmc[..8].to_vec(),
        r.cusp_speeds[1..].to_vec(),
        r.ascmc_speeds[..8].to_vec(),
    )
        .into_py_any(py).unwrap())
}

/// Calculate house position of a point.
#[pyfunction]
#[pyo3(signature = (armc, lat, eps, hsys = b'P', lon = 0.0, lat_body = 0.0))]
fn house_pos(armc: f64, lat: f64, eps: f64, hsys: u8, lon: f64, lat_body: f64) -> PyResult<f64> {
    celestial::house_pos(Degrees::new(armc), Latitude::new(lat), Degrees::new(eps), HouseSystem(hsys), [lon, lat_body]).map_err(to_py)
}

/// Name of a house system given its byte code.
#[pyfunction]
fn house_name(hsys: u8) -> &'static str {
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
    let r = celestial::sol_eclipse_when_glob(JulianDay::new(tjd_start), CalcFlags(flags), ecl_type, backwards)
        .map_err(to_py)?;
    Ok((r.ret_flags, r.tret.to_vec()).into_py_any(py).unwrap())
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
    let r = celestial::sol_eclipse_when_loc(JulianDay::new(tjd_start), CalcFlags(flags), geopos, backwards)
        .map_err(to_py)?;
    Ok((r.ret_flags, r.tret.to_vec(), r.attr.to_vec()).into_py_any(py).unwrap())
}

/// How a solar eclipse looks from a location. Returns `(ret_flags, (attr...))`.
#[pyfunction]
#[pyo3(signature = (jd_ut, geopos, flags = 2))]
fn sol_eclipse_how(py: Python<'_>, jd_ut: f64, geopos: [f64; 3], flags: i32) -> PyResult<PyObject> {
    let r = celestial::sol_eclipse_how(JulianDay::new(jd_ut), CalcFlags(flags), geopos).map_err(to_py)?;
    Ok((r.ret_flags, r.attr.to_vec()).into_py_any(py).unwrap())
}

/// Where a solar eclipse is central. Returns `(ret_flags, (geopos...), (attr...))`.
#[pyfunction]
#[pyo3(signature = (tjd, flags = 2))]
fn sol_eclipse_where(py: Python<'_>, tjd: f64, flags: i32) -> PyResult<PyObject> {
    let r = celestial::sol_eclipse_where(JulianDay::new(tjd), CalcFlags(flags)).map_err(to_py)?;
    Ok((r.ret_flags, r.geopos.to_vec(), r.attr.to_vec()).into_py_any(py).unwrap())
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
    let r = celestial::lun_eclipse_when(JulianDay::new(tjd_start), CalcFlags(flags), ecl_type, backwards)
        .map_err(to_py)?;
    Ok((r.ret_flags, r.tret.to_vec()).into_py_any(py).unwrap())
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
    let r = celestial::lun_eclipse_when_loc(JulianDay::new(tjd_start), CalcFlags(flags), geopos, backwards)
        .map_err(to_py)?;
    Ok((r.ret_flags, r.tret.to_vec(), r.attr.to_vec()).into_py_any(py).unwrap())
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
    let r = celestial::lun_eclipse_how(JulianDay::new(jd_ut), CalcFlags(flags), geopos).map_err(to_py)?;
    Ok((r.ret_flags, r.attr.to_vec()).into_py_any(py).unwrap())
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
        JulianDay::new(tjdut),
        body_of(planet)?,
        None,
        CalcFlags(flags),
        event_type,
        geopos,
        pressure_mb,
        temp_c,
    )
    .map_err(to_py)?;
    Ok((r.ret_flags, r.tret).into_py_any(py).unwrap())
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
    let d = celestial::revjul(JulianDay::new(jd), Calendar::from(calendar));
    (d.year, d.month, d.day, d.hour).into_py_any(py).unwrap()
}

/// Day of week (0 = Monday, …, 6 = Sunday).
#[pyfunction]
fn day_of_week(jd: f64) -> i32 {
    celestial::day_of_week(JulianDay::new(jd))
}

/// Delta-T (TT − UT) for a Julian day.
#[pyfunction]
fn deltat(tjd: f64) -> f64 {
    celestial::deltat(JulianDay::new(tjd))
}

/// Sidereal time for a UT Julian day.
#[pyfunction]
fn sidtime(jd_ut: f64) -> f64 {
    celestial::sidtime(JulianDay::new(jd_ut))
}

/// Greenwich Mean Sidereal Time (GMST) in decimal hours — without the
/// equation of the equinoxes. Use `sidtime()` for apparent sidereal time (GAST).
#[pyfunction]
fn mean_sidtime(jd_ut: f64) -> f64 {
    celestial::mean_sidtime(JulianDay::new(jd_ut))
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
    Ok((pair.et, pair.ut1).into_py_any(py).unwrap())
}

// ─── Ayanamsa ─────────────────────────────────────────────────────────────────

/// Ayanamsa for a Julian day (ET).
#[pyfunction]
fn ayanamsa(jd_et: f64) -> f64 {
    celestial::ayanamsa(JulianDay::new(jd_et))
}

/// Ayanamsa for a Julian day (UT).
#[pyfunction]
fn ayanamsa_ut(jd_ut: f64) -> f64 {
    celestial::ayanamsa_ut(JulianDay::new(jd_ut))
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
    (d, m, s, frac, sgn).into_py_any(py).unwrap()
}

/// Coordinate transform ecliptic ↔ equatorial.
#[pyfunction]
fn coord_transform(py: Python<'_>, coord: [f64; 3], eps: f64) -> PyObject {
    celestial::coord_transform(coord, Degrees::new(eps)).into_py_any(py).unwrap()
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
    let r = celestial::azalt(JulianDay::new(tjdut), calc_flag, geopos, pressure_mb, temp_c, xin);
    (r.azimuth, r.true_alt, r.apparent_alt).into_py_any(py).unwrap()
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
    let out = celestial::azalt_rev(JulianDay::new(tjdut), calc_flag, geopos, [az, alt]);
    (out[0], out[1]).into_py_any(py).unwrap()
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
    (r, d.to_vec()).into_py_any(py).unwrap()
}

// ─── Info ─────────────────────────────────────────────────────────────────────

/// Swiss Ephemeris library version string.
#[pyfunction]
fn version() -> &'static str {
    celestial::version()
}

/// Name of a planet / body.
#[pyfunction]
fn planet_name(planet: i32) -> &'static str {
    // Lenient: returns "Unknown" for out-of-range ids (no Result channel here).
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
    (r.diff, r.speed, r.factor, r.matched as i32).into_py_any(py).unwrap()
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
    (r.diff, r.speed, r.factor, r.matched as i32).into_py_any(py).unwrap()
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
    (r.diff, r.speed, r.factor, r.matched as i32).into_py_any(py).unwrap()
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
    (r.diff, r.speed, r.factor, r.matched as i32).into_py_any(py).unwrap()
}

/// Compute antiscion and contrantiscion. Returns `(antiscion[6], contrantiscion[6])`.
#[pyfunction]
fn antiscion(py: Python<'_>, pos: [f64; 6], axis: f64) -> PyObject {
    let a = celestial::antiscion(pos, axis);
    (a.antiscion.to_vec(), a.contrantiscion.to_vec()).into_py_any(py).unwrap()
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
    celestial::revjul_hms(JulianDay::new(jd), Calendar::from(calendar))
        .to_vec()
        .into_py_any(py).unwrap()
}

/// Parse ISO datetime string to [y,mo,d,h,mi,s].
#[pyfunction]
fn parse_datetime(py: Python<'_>, s: &str) -> PyObject {
    match celestial::parse_datetime(s) {
        Some(dt) => dt.to_vec().into_py_any(py).unwrap(),
        None => py.None(),
    }
}

/// Duration between two JDs → [days, hours, min, sec].
#[pyfunction]
fn jd_duration(py: Python<'_>, jd_start: f64, jd_end: f64) -> PyObject {
    celestial::jd_duration(JulianDay::new(jd_start), JulianDay::new(jd_end))
        .to_vec()
        .into_py_any(py).unwrap()
}

/// Format JD as ISO string "YYYY-MM-DD HH:MM:SS UTC".
#[pyfunction]
fn jd_to_iso_string(jd: f64, calendar: i32) -> String {
    celestial::jd_to_iso_string(JulianDay::new(jd), Calendar::from(calendar))
}

// ─── Formatting ──────────────────────────────────────────────────────────────

/// Split ecliptic longitude → [deg_in_sign, sign_num, minutes, seconds].
#[pyfunction]
fn degsplit(py: Python<'_>, pos: f64) -> PyObject {
    celestial::degsplit(pos).to_vec().into_py_any(py).unwrap()
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
    celestial::long_to_rasi(Longitude::new(lon))
}
/// Navamsa from ecliptic longitude.
#[pyfunction]
fn long_to_navamsa(lon: f64) -> i32 {
    celestial::long_to_navamsa(Longitude::new(lon))
}
/// Nakshatra and Pada from ecliptic longitude. Returns (nakshatra, pada).
#[pyfunction]
fn long_to_nakshatra(py: Python<'_>, lon: f64) -> PyObject {
    let (n, p) = celestial::long_to_nakshatra(Longitude::new(lon));
    (n, p).into_py_any(py).unwrap()
}
/// Nakshatra name from index.
#[pyfunction]
fn nakshatra_name(n: i32) -> Option<&'static str> {
    celestial::nakshatra_name(n)
}
/// Raman house cusps. Returns 12 longitude values.
#[pyfunction]
fn raman_houses(py: Python<'_>, asc: f64, mc: f64, sandhi: bool) -> PyObject {
    celestial::raman_houses(Degrees::new(asc), Degrees::new(mc), sandhi)
        .to_vec()
        .into_py_any(py).unwrap()
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
    celestial::residential_strength(graha, &bm).into_py_any(py).unwrap()
}
/// Saturn 4-Stars index (Halbronn). Returns [sat, ald, reg, ant, fom, index].
#[pyfunction]
fn saturn_4_stars(py: Python<'_>, jd: f64, flags: i32) -> PyResult<PyObject> {
    celestial::saturn_4_stars(JulianDay::new(jd), CalcFlags(flags))
        .map(|r| r.to_vec().into_py_any(py).unwrap())
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
    // Lenient: PyObject return — invalid ids surface as Python None via downstream.
    match celestial::next_retro(
        Body::from_raw(planet),
        jd_start,
        backward,
        stop_days,
        CalcFlags(flags),
    ) {
        Some(r) => (r.jd, r.pos.to_vec()).into_py_any(py).unwrap(),
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
    // Lenient: PyObject return — invalid ids surface as Python None via downstream.
    match celestial::next_aspect(
        Body::from_raw(planet),
        aspect,
        fixed_pt,
        jd_start,
        backward,
        stop_days,
        CalcFlags(flags),
    ) {
        Some(r) => (r.jd, r.pos1.to_vec()).into_py_any(py).unwrap(),
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
    // Lenient: PyObject return — invalid ids surface as Python None via downstream.
    match celestial::next_aspect_with(
        Body::from_raw(planet),
        aspect,
        Body(other),
        jd_start,
        backward,
        stop_days,
        CalcFlags(flags),
    ) {
        Some(r) => (r.jd, r.pos1.to_vec(), r.pos2.to_vec()).into_py_any(py).unwrap(),
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
    celestial::sign_ingress_ut(body_of(planet)?, JulianDay::new(jd), CalcFlags(flags), backward)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Find the next retrograde and direct stations for a body after `jd_start`.
/// Returns `(retrograde_jd, direct_jd)`.
#[pyfunction]
#[pyo3(signature = (planet, jd, flags))]
fn retrograde_station_ut(py: Python<'_>, planet: i32, jd: f64, flags: i32) -> PyResult<PyObject> {
    celestial::retrograde_station_ut(body_of(planet)?, JulianDay::new(jd), CalcFlags(flags))
        .map(|s| (s.retrograde, s.direct).into_py_any(py).unwrap())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Arabic Part / Lot formula: `(asc + body2 - body1) mod 360`.
/// For Lot of Fortune: `arabic_part(asc, moon_lon, sun_lon)`.
#[pyfunction]
fn arabic_part(asc: f64, body2: f64, body1: f64) -> f64 {
    celestial::arabic_part(Degrees::new(asc), Longitude::new(body2), Longitude::new(body1))
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
        body_of(planet)?,
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
        body_of(planet)?,
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
    celestial::solar_return_jd(JulianDay::new(jd_natal), return_year, CalcFlags(flags))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Julian day of the next lunar return after `jd_start`.
/// The lunar return is when transiting Moon returns to its natal longitude.
#[pyfunction]
fn lunar_return_jd(jd_natal: f64, jd_start: f64, flags: i32) -> PyResult<f64> {
    celestial::lunar_return_jd(JulianDay::new(jd_natal), JulianDay::new(jd_start), CalcFlags(flags))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Shortest-arc midpoint between two ecliptic longitudes (degrees).
/// Always returns a value in [0, 360).
#[pyfunction]
fn midpoint(lon1: f64, lon2: f64) -> f64 {
    celestial::midpoint(Longitude::new(lon1), Longitude::new(lon2))
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
    celestial::local_apparent_solar_time(JulianDay::new(jd_ut), Longitude::new(geolon_deg))
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
    Ok((house, degree).into_py_any(py).unwrap())
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
    let dashas = celestial::vimshottari_dasha(JulianDay::new(jd_birth), Longitude::new(moon_lon_sidereal), years_ahead);
    let result: Vec<PyObject> = dashas
        .iter()
        .map(|d| (d.body.as_raw(), d.start, d.end, d.years).into_py_any(py).unwrap())
        .collect();
    result.into_py_any(py).unwrap()
}

// ─── Sefirat HaOmer ───────────────────────────────────────────────────────────

/// Return the Omer day for the given Julian day, or None if not in the Omer period.
///
/// Returns (day, week, day_of_week, week_sefirah, day_sefirah, hebrew_text, is_lag_baomer, jd)
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn omer_from_jd(py: Python<'_>, jd: f64) -> PyObject {
    match celestial::omer_from_jd(JulianDay::new(jd)) {
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
            .into_py_any(py).unwrap(),
        None => py.None(),
    }
}

/// Return the Julian day of a specific Omer day (1–49) in the given Hebrew year.
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn omer_day_jd(hebrew_year: i32, day: i32) -> Option<f64> {
    celestial::omer_day_jd(hebrew_year, day as u8)
}

/// Return the Julian day of the first day of the Omer for the given Hebrew year.
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn omer_start_jd(hebrew_year: i32) -> f64 {
    celestial::omer_start_jd(hebrew_year)
}

/// Return all 49 Omer days for the given Hebrew year.
///
/// Each element is (day, week, day_of_week, week_sefirah, day_sefirah, hebrew_text, is_lag_baomer, jd)
#[cfg(feature = "calendar-traditions")]
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
                .into_py_any(py).unwrap()
        })
        .collect();
    result.into_py_any(py).unwrap()
}

/// Return the Omer period (start_jd, end_jd, hebrew_year) containing the given JD.
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn omer_period(py: Python<'_>, jd: f64) -> PyObject {
    let p = celestial::omer_period(JulianDay::new(jd));
    (p.start_jd, p.end_jd, p.hebrew_year).into_py_any(py).unwrap()
}

/// Return the full declaration string for the given Omer day (1–49).
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn omer_declaration(day: i32) -> String {
    celestial::omer_declaration(day as u8)
}

// ─── Module registration ──────────────────────────────────────────────────────

// Register the `celestial_py` Python extension module.

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 5–8 bindings: Hellenistic, Persian, Chinese, Mesoamerican, Indigenous
// ═══════════════════════════════════════════════════════════════════════════════

/// Egyptian terms (bounds) ruler for an ecliptic longitude.
/// Returns the planet index (0=Sun…6=Saturn).
#[pyfunction]
fn egyptian_terms_ruler(lon: f64) -> i32 {
    celestial::egyptian_terms_ruler(Longitude::new(lon)).as_raw()
}

/// Chaldean decan (face) ruler for an ecliptic longitude.
/// Returns the planet index.
#[pyfunction]
fn decan_ruler(lon: f64) -> i32 {
    celestial::decan_ruler(Longitude::new(lon)).as_raw()
}

/// Triplicity rulers for an ecliptic longitude.
/// Returns (day_ruler, night_ruler, participating_ruler) as planet indices.
#[pyfunction]
fn triplicity_rulers(py: Python<'_>, lon: f64) -> PyObject {
    let (d, n, p) = celestial::triplicity_rulers(Longitude::new(lon));
    (d.as_raw(), n.as_raw(), p.as_raw()).into_py_any(py).unwrap()
}

/// Full dignity for a planet at a longitude.
/// Returns (dignity_name: str, score: i8).
#[pyfunction]
fn full_dignity(py: Python<'_>, body_raw: i32, lon: f64, is_day: bool) -> PyObject {
    use celestial::body::Body;
    // Lenient: full_dignity tolerates unknown ids by returning ("None", 0).
    let body = Body::from_raw(body_raw);
    let (dig, score) = celestial::full_dignity(body, Longitude::new(lon), is_day);
    (dig.to_string(), score).into_py_any(py).unwrap()
}

/// Almuten (planet with highest dignity score) at a longitude.
/// Returns (body_raw: i32, score: i8).
#[pyfunction]
fn almuten(py: Python<'_>, lon: f64, is_day: bool) -> PyObject {
    let (body, score) = celestial::almuten(Longitude::new(lon), is_day);
    (body.as_raw(), score).into_py_any(py).unwrap()
}

/// Whether a chart is a day chart (Sun above horizon).
#[pyfunction]
fn is_day_chart(sun_lon: f64, cusps: Vec<f64>) -> bool {
    if cusps.len() < 13 {
        return false;
    }
    let mut arr = [0.0f64; 13];
    for (i, &v) in cusps.iter().take(13).enumerate() {
        arr[i] = v;
    }
    celestial::is_day_chart(Longitude::new(sun_lon), &arr)
}

/// Firdaria planetary period timeline.
/// Returns list of (major_lord, minor_lord, start_jd, end_jd, years).
#[pyfunction]
fn firdaria(py: Python<'_>, jd_birth: f64, is_day: bool, span_years: f64) -> PyObject {
    let periods = celestial::firdaria(JulianDay::new(jd_birth), is_day, span_years);
    let result: Vec<PyObject> = periods
        .iter()
        .map(|p| {
            (
                p.major_lord.as_raw(),
                p.minor_lord.as_raw(),
                p.start,
                p.end,
                p.years,
            )
                .into_py_any(py).unwrap()
        })
        .collect();
    result.into_py_any(py).unwrap()
}

/// Four Pillars of Destiny (Ba Zi).
/// Returns list of 4 dicts: year, month, day, hour pillars.
#[pyfunction]
fn four_pillars(py: Python<'_>, jd_ut: f64, hour_ut: f64, sun_lon: f64) -> PyObject {
    let pillars = celestial::four_pillars(JulianDay::new(jd_ut), hour_ut, Longitude::new(sun_lon));
    let result: Vec<PyObject> = pillars
        .iter()
        .map(|p| {
            pyo3::types::PyDict::new(py)
                .tap(|d| {
                    let _ = d.set_item("stem", p.stem);
                    let _ = d.set_item("branch", p.branch);
                    let _ = d.set_item("stem_name", p.stem_name);
                    let _ = d.set_item("branch_name", p.branch_name);
                    let _ = d.set_item("animal", p.animal);
                    let _ = d.set_item("stem_element", p.stem_element);
                    let _ = d.set_item("branch_element", p.branch_element);
                    let _ = d.set_item("yang", p.yang);
                })
                .into_py_any(py).unwrap()
        })
        .collect();
    result.into_py_any(py).unwrap()
}

/// Current solar term position.
/// Returns (current_idx, degrees_into, next_idx, degrees_to_next).
#[pyfunction]
fn solar_term_position(py: Python<'_>, sun_lon: f64) -> PyObject {
    let (cur, into, next, to) = celestial::solar_term_position(Longitude::new(sun_lon));
    (cur, into, next, to).into_py_any(py).unwrap()
}

/// Aztec Tonalpohualli (260-day) position.
/// Returns (trecena 1-13, sign_idx 0-19, nahuatl_name, english).
#[pyfunction]
fn tonalpohualli(py: Python<'_>, jd: f64) -> PyObject {
    let (t, i, n, e) = celestial::tonalpohualli(JulianDay::new(jd));
    (t, i, n, e).into_py_any(py).unwrap()
}

/// Aztec Xiuhpohualli (365-day) position.
/// Returns (month_idx, day, month_name, english).
#[pyfunction]
fn xiuhpohualli(py: Python<'_>, jd: f64) -> PyObject {
    let (m, d, n, e) = celestial::xiuhpohualli(JulianDay::new(jd));
    (m, d, n, e).into_py_any(py).unwrap()
}

/// Maya Tzolkin (260-day) position.
/// Returns (trecena, sign_idx, mayan_name, english).
#[pyfunction]
fn tzolkin(py: Python<'_>, jd: f64) -> PyObject {
    let (t, i, n, e) = celestial::tzolkin(JulianDay::new(jd));
    (t, i, n, e).into_py_any(py).unwrap()
}

/// Maya Haab (365-day) position.
/// Returns (month_idx, day, month_name).
#[pyfunction]
fn haab(py: Python<'_>, jd: f64) -> PyObject {
    let (m, d, n) = celestial::haab(JulianDay::new(jd));
    (m, d, n).into_py_any(py).unwrap()
}

/// Maya Calendar Round (52-year cycle).
/// Returns (tzolkin_trecena, tzolkin_sign, haab_day, haab_month).
#[pyfunction]
fn calendar_round(py: Python<'_>, jd: f64) -> PyObject {
    let (t, s, d, m) = celestial::calendar_round(JulianDay::new(jd));
    (t, s, d, m).into_py_any(py).unwrap()
}

/// Medicine Wheel birth totem (Sun Bear synthesis).
/// Returns (animal, element, clan, season) for a Sun longitude.
#[pyfunction]
fn medicine_wheel_totem(py: Python<'_>, sun_lon: f64) -> PyObject {
    let (a, e, c, s) = celestial::medicine_wheel_totem(Longitude::new(sun_lon));
    (a, e, c, s).into_py_any(py).unwrap()
}

/// Egyptian decan (face) for an ecliptic longitude.
/// Returns (decan_idx 0-35, decan_name, rising_star).
#[pyfunction]
fn egyptian_decan(py: Python<'_>, lon: f64) -> PyObject {
    let (i, n, s) = celestial::egyptian_decan(Longitude::new(lon));
    (i, n, s).into_py_any(py).unwrap()
}

/// Schwabe solar-cycle info at the given Julian Day.
/// Returns `None` for non-finite input or for dates outside cycles 1..=25
/// (i.e. before ~1755 or after ~2030). Tuple layout:
/// `(cycle_num, phase, phase_name, min_jd, max_jd, next_min_jd,
///   years_since_min, nickname_or_empty, grand_epoch_or_empty)`.
#[pyfunction]
fn solar_cycle(py: Python<'_>, jd: f64) -> Option<PyObject> {
    let info = celestial::solar_cycle(JulianDay::new(jd))?;
    Some(
        (
            info.cycle_num,
            info.phase,
            info.phase_name.name(),
            info.min_jd,
            info.max_jd,
            info.next_min_jd,
            info.years_since_min,
            info.nickname.unwrap_or(""),
            info.grand_epoch.map_or("", |g| g.name()),
        )
            .into_py_any(py).unwrap(),
    )
}

/// Grand solar epoch label for the given JD, or empty string outside any
/// named long-term envelope (Spörer / Maunder / Dalton / Modern Maximum).
#[pyfunction]
fn grand_solar_epoch(jd: f64) -> &'static str {
    celestial::grand_solar_epoch(JulianDay::new(jd)).map_or("", |g| g.name())
}

/// Informal name for a Schwabe cycle (e.g. cycle 19 = "the Great Cycle").
/// Returns an empty string for cycles without a nickname.
#[pyfunction]
fn cycle_nickname(n: u8) -> &'static str {
    celestial::cycle_nickname(n).unwrap_or("")
}

// Helper trait for PyDict tap pattern
trait Tap: Sized {
    fn tap(self, f: impl FnOnce(&Self)) -> Self {
        f(&self);
        self
    }
}
impl Tap for pyo3::Bound<'_, pyo3::types::PyDict> {}

#[pyfunction]
fn house_name_str(hsys: u8) -> &'static str {
    celestial::house_name(HouseSystem(hsys))
}

#[pyfunction]
fn mooncross_node(py: Python<'_>, jd_et: f64, flags: i32) -> PyResult<PyObject> {
    let n = celestial::mooncross_node(JulianDay::new(jd_et), CalcFlags(flags)).map_err(to_py)?;
    Ok((n.jd_cross, n.xlon).into_py_any(py).unwrap())
}

#[pyfunction]
fn mooncross_node_ut(py: Python<'_>, jd_ut: f64, flags: i32) -> PyResult<PyObject> {
    let n = celestial::mooncross_node_ut(JulianDay::new(jd_ut), CalcFlags(flags)).map_err(to_py)?;
    Ok((n.jd_cross, n.xlon).into_py_any(py).unwrap())
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
fn next_aspect_cusp2(
    py: Python<'_>,
    body: i32,
    aspect: f64,
    cusp: usize,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: u32,
    backward: bool,
    flags: i32,
) -> PyObject {
    // Lenient: PyObject return — invalid ids fall through to a None Python value via downstream.
    match celestial::next_aspect_cusp2(
        Body::from_raw(body),
        aspect,
        cusp,
        jd_start,
        lat,
        lon,
        HouseSystem(hsys as u8),
        backward,
        CalcFlags(flags),
    ) {
        Some(r) => r.jd.into_py_any(py).unwrap(),
        None => py.None(),
    }
}

// ── Legacy aliases (parity with PHP binding) ──────────────────────────────────

#[pyfunction]
fn degnorm(d: f64) -> f64 {
    celestial::norm_deg(d)
}

#[pyfunction]
fn difdeg2n(p1: f64, p2: f64) -> f64 {
    celestial::diff_deg_signed(p1, p2)
}

#[pyfunction]
fn get_ayanamsa(jd_et: f64) -> f64 {
    celestial::ayanamsa(JulianDay::new(jd_et))
}

#[pyfunction]
fn get_ayanamsa_name(sid_mode: i32) -> String {
    celestial::ayanamsa_name(sid_mode).to_string()
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn next_full_moon(jd_start: f64) -> f64 {
    celestial::next_full_moon_after(JulianDay::new(jd_start))
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn next_sabbat_name(jd_from: f64) -> PyResult<String> {
    celestial::next_sabbat(jd_from)
        .map(|s| s.name.to_string())
        .map_err(to_py)
}

#[pyfunction]
fn solcross_ut(x2cross: f64, jd_ut: f64, flags: i32) -> PyResult<f64> {
    celestial::solcross_ut(Longitude::new(x2cross), JulianDay::new(jd_ut), CalcFlags(flags)).map_err(to_py)
}

// ═══════════════════════════════════════════════════════════════════════════
// Additions: ISO week, Maya Long Count, Yallop, Coptic, Zoroastrian, Tibetan,
// Vietnamese
// ═══════════════════════════════════════════════════════════════════════════

#[pyfunction]
fn iso_week(py: Python<'_>, jd: f64) -> PyObject {
    celestial::iso_week(JulianDay::new(jd)).into_py_any(py).unwrap()
}

#[pyfunction]
fn day_of_year(year: i32, month: u32, day: u32) -> u32 {
    celestial::day_of_year(year, month, day)
}

#[pyfunction]
fn weeks_in_iso_year(year: i32) -> u32 {
    celestial::weeks_in_iso_year(year)
}

#[pyfunction]
fn maya_long_count(py: Python<'_>, jd: f64) -> PyObject {
    celestial::maya_long_count(JulianDay::new(jd)).into_py_any(py).unwrap()
}

#[pyfunction]
fn maya_long_count_str(jd: f64) -> String {
    celestial::maya_long_count_str(JulianDay::new(jd))
}

#[pyfunction]
fn yallop_q(py: Python<'_>, arcv_deg: f64, arcl_deg: f64, sd_arcmin: f64) -> PyObject {
    let (q, c) = celestial::yallop_q(arcv_deg, arcl_deg, sd_arcmin);
    (q, c.to_string()).into_py_any(py).unwrap()
}

#[pyfunction]
fn best_time_method(jd_sunset: f64, jd_moonset: f64) -> f64 {
    celestial::best_time_method(JulianDay::new(jd_sunset), JulianDay::new(jd_moonset))
}

#[pyfunction]
fn vietnamese_month_start_jd(jd_ut: f64) -> Option<f64> {
    celestial::vietnamese_month_start_jd(JulianDay::new(jd_ut))
}

#[pyfunction]
fn vietnamese_chinese_boundary_differs(jd_ut: f64) -> bool {
    celestial::vietnamese_chinese_boundary_differs(JulianDay::new(jd_ut))
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn coptic_to_jd(year: i32, month: u32, day: u32) -> f64 {
    celestial::coptic_to_jd(year, month, day)
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn jd_to_coptic(py: Python<'_>, jd: f64) -> PyObject {
    celestial::jd_to_coptic(JulianDay::new(jd)).into_py_any(py).unwrap()
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn ethiopic_to_jd(year: i32, month: u32, day: u32) -> f64 {
    celestial::ethiopic_to_jd(year, month, day)
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn jd_to_ethiopic(py: Python<'_>, jd: f64) -> PyObject {
    celestial::jd_to_ethiopic(JulianDay::new(jd)).into_py_any(py).unwrap()
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn is_coptic_leap_year(year: i32) -> bool {
    celestial::is_coptic_leap_year(year)
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn coptic_month_days(year: i32, month: u32) -> u32 {
    celestial::coptic_month_days(year, month)
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn fasli_nowruz_jd(year: i32) -> Option<f64> {
    celestial::fasli_nowruz_jd(year)
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn jd_to_fasli(py: Python<'_>, jd: f64) -> PyObject {
    match celestial::jd_to_fasli(JulianDay::new(jd)) {
        Some(t) => t.into_py_any(py).unwrap(),
        None => py.None(),
    }
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn losar_jd(year: i32) -> Option<f64> {
    celestial::losar_jd(year)
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn tibetan_year_name(py: Python<'_>, year: i32) -> PyObject {
    celestial::tibetan_year_name(year).into_py_any(py).unwrap()
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
fn ic_transit_ut(
    planet: i32,
    jd_natal: f64,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: u32,
    flags: i32,
    backward: bool,
) -> PyResult<f64> {
    celestial::ic_transit_ut(
        body_of(planet)?,
        jd_natal,
        jd_start,
        lat,
        lon,
        HouseSystem(hsys as u8),
        CalcFlags(flags),
        backward,
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
fn asc_transit_ut(
    planet: i32,
    jd_natal: f64,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: u32,
    flags: i32,
    backward: bool,
) -> PyResult<f64> {
    celestial::asc_transit_ut(
        body_of(planet)?,
        jd_natal,
        jd_start,
        lat,
        lon,
        HouseSystem(hsys as u8),
        CalcFlags(flags),
        backward,
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
fn dsc_transit_ut(
    planet: i32,
    jd_natal: f64,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: u32,
    flags: i32,
    backward: bool,
) -> PyResult<f64> {
    celestial::dsc_transit_ut(
        body_of(planet)?,
        jd_natal,
        jd_start,
        lat,
        lon,
        HouseSystem(hsys as u8),
        CalcFlags(flags),
        backward,
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
fn next_aspect_cusp(
    body: i32,
    aspect: f64,
    cusp: usize,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: u32,
    backward: bool,
    flags: i32,
) -> Option<(f64, f64)> {
    celestial::next_aspect_cusp(
        body_of(body).ok()?,
        aspect,
        cusp,
        jd_start,
        lat,
        lon,
        HouseSystem(hsys as u8),
        backward,
        CalcFlags(flags),
    )
    .map(|r| (r.jd, r.pos[0]))
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn sabbats_for_year(py: Python<'_>, year: i32) -> PyResult<PyObject> {
    let sabbats =
        celestial::sabbats_for_year(year).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let result: Vec<PyObject> = sabbats.iter().map(|s| (s.name, s.jd).into_py_any(py).unwrap()).collect();
    Ok(result.into_py_any(py).unwrap())
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn next_sabbat(jd_from: f64) -> PyResult<(String, f64)> {
    let s = celestial::next_sabbat(jd_from).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok((s.name.to_string(), s.jd))
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn sabbat_jd(year: i32, kind: u8) -> PyResult<f64> {
    use celestial::SabbatKind;
    let kinds = [
        SabbatKind::Samhain,
        SabbatKind::Yule,
        SabbatKind::Imbolc,
        SabbatKind::Ostara,
        SabbatKind::Beltane,
        SabbatKind::Litha,
        SabbatKind::Lughnasadh,
        SabbatKind::Mabon,
    ];
    let k = kinds
        .get(kind as usize)
        .ok_or_else(|| PyRuntimeError::new_err("invalid sabbat kind (0-7)"))?;
    celestial::sabbat_jd(year, *k).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn esbats_for_year(py: Python<'_>, year: i32) -> PyResult<PyObject> {
    let esbats =
        celestial::esbats_for_year(year).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let result: Vec<PyObject> = esbats
        .iter()
        .map(|e| (e.display_name, e.jd).into_py_any(py).unwrap())
        .collect();
    Ok(result.into_py_any(py).unwrap())
}

#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn next_esbat(jd_from: f64) -> PyResult<(String, f64)> {
    let e = celestial::next_esbat(jd_from).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok((e.display_name.to_string(), e.jd))
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
fn secondary_progressions(
    py: Python<'_>,
    jd_natal: f64,
    years: f64,
    bodies: Vec<i32>,
    lat: f64,
    lon: f64,
    hsys: u8,
    flags: i32,
) -> PyResult<PyObject> {
    let body_list: Vec<celestial::Body> = bodies.iter().map(|&b| celestial::Body(b)).collect();
    let (positions, houses) = celestial::secondary_progressions(
        JulianDay::new(jd_natal),
        years,
        &body_list,
        Latitude::new(lat),
        Longitude::new(lon),
        celestial::HouseSystem(hsys),
        celestial::CalcFlags(flags),
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let pos_list: Vec<PyObject> = positions
        .iter()
        .map(|(b, p)| (b.as_raw(), p.lon, p.lat, p.dist, p.speed_lon).into_py_any(py).unwrap())
        .collect();
    let cusps: Vec<f64> = houses.cusps.to_vec();
    Ok((pos_list, cusps).into_py_any(py).unwrap())
}

#[pyfunction]
fn solar_arc_directions(
    py: Python<'_>,
    jd_natal: f64,
    years: f64,
    natal_positions: Vec<(i32, f64)>,
    natal_mc: f64,
    flags: i32,
) -> PyResult<PyObject> {
    let pos: Vec<(celestial::Body, f64)> = natal_positions
        .iter()
        .map(|&(b, lon)| (celestial::Body(b), lon))
        .collect();
    let (arc, directed, mc_arc) = celestial::solar_arc_directions(
        JulianDay::new(jd_natal),
        years,
        &pos,
        Degrees::new(natal_mc),
        celestial::CalcFlags(flags),
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let directed_list: Vec<PyObject> = directed
        .iter()
        .map(|(b, lon)| (b.as_raw(), *lon).into_py_any(py).unwrap())
        .collect();
    Ok((arc, directed_list, mc_arc).into_py_any(py).unwrap())
}

#[pyfunction]
fn midpoint_table(py: Python<'_>, positions: Vec<(i32, f64)>, orb: f64) -> PyObject {
    let pos: Vec<(celestial::Body, f64)> = positions
        .iter()
        .map(|&(b, lon)| (celestial::Body(b), lon))
        .collect();
    let table = celestial::midpoint_table(&pos, orb);
    let result: Vec<PyObject> = table
        .iter()
        .map(|e| (e.0.as_raw(), e.1.as_raw(), e.2).into_py_any(py).unwrap())
        .collect();
    result.into_py_any(py).unwrap()
}

#[pyfunction]
fn calc_chart_aspects(
    py: Python<'_>,
    positions: Vec<(i32, f64, f64)>,
    aspects: Vec<f64>,
    orb: f64,
) -> PyObject {
    let pos: Vec<(celestial::Body, f64, f64)> = positions
        .iter()
        .map(|&(b, lon, spd)| (celestial::Body(b), lon, spd))
        .collect();
    let result: Vec<PyObject> = celestial::calc_chart_aspects(&pos, &aspects, orb)
        .iter()
        .map(|a| {
            (
                a.body1.as_raw(),
                a.body2.as_raw(),
                a.aspect,
                a.orb,
                a.applying,
            )
                .into_py_any(py).unwrap()
        })
        .collect();
    result.into_py_any(py).unwrap()
}

#[pyfunction]
fn calc_chart_aspects_auto(
    py: Python<'_>,
    positions: Vec<(i32, f64, f64)>,
    aspects: Vec<f64>,
) -> PyObject {
    let pos: Vec<(celestial::Body, f64, f64)> = positions
        .iter()
        .map(|&(b, lon, spd)| (celestial::Body(b), lon, spd))
        .collect();
    let result: Vec<PyObject> = celestial::calc_chart_aspects_auto(&pos, &aspects)
        .iter()
        .map(|a| {
            (
                a.body1.as_raw(),
                a.body2.as_raw(),
                a.aspect,
                a.orb,
                a.applying,
            )
                .into_py_any(py).unwrap()
        })
        .collect();
    result.into_py_any(py).unwrap()
}

#[pyfunction]
fn sexagenary_name(cycle_index: u8) -> (String, String) {
    let (stem, branch) = celestial::sexagenary_name(cycle_index);
    (stem.to_string(), branch.to_string())
}

#[pyfunction]
fn monthly_profection(cusps: Vec<f64>, age_years: u32, age_months: u32) -> PyResult<(u8, f64)> {
    let arr: [f64; 13] = cusps
        .get(..13)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| PyRuntimeError::new_err("cusps needs 13 elements"))?;
    Ok(celestial::monthly_profection(&arr, age_years, age_months))
}

/// Current Moon phase at the given JD.
/// Returns the phase name string.
#[pyfunction]
fn moon_phase(jd: f64) -> PyResult<String> {
    celestial::moon_phase(JulianDay::new(jd))
        .map(|p| p.name().to_string())
        .map_err(to_py)
}

/// Fraction of the Moon's disk illuminated (0.0–1.0).
#[pyfunction]
fn moon_illumination(jd: f64) -> PyResult<f64> {
    celestial::moon_illumination(JulianDay::new(jd)).map_err(to_py)
}

/// Moon–Sun elongation in degrees (0°–360°).
#[pyfunction]
fn moon_elongation(jd: f64) -> PyResult<f64> {
    celestial::moon_elongation(JulianDay::new(jd)).map_err(to_py)
}

/// Phase angle in degrees (0° = new, 180° = full).
#[pyfunction]
fn moon_phase_angle(jd: f64) -> PyResult<f64> {
    celestial::moon_phase_angle(JulianDay::new(jd)).map_err(to_py)
}

/// JD of the next new moon at or after `jd_from`.
#[pyfunction]
fn next_new_moon(jd_from: f64) -> PyResult<f64> {
    celestial::next_new_moon(JulianDay::new(jd_from)).map_err(to_py)
}

/// JD of the next first-quarter moon at or after `jd_from`.
#[pyfunction]
fn next_first_quarter(jd_from: f64) -> PyResult<f64> {
    celestial::next_first_quarter(JulianDay::new(jd_from)).map_err(to_py)
}

/// JD of the next full moon at or after `jd_from`.
#[pyfunction]
fn next_full_moon_phase(jd_from: f64) -> PyResult<f64> {
    celestial::next_full_moon_phase(JulianDay::new(jd_from)).map_err(to_py)
}

/// JD of the next last-quarter moon at or after `jd_from`.
#[pyfunction]
fn next_last_quarter(jd_from: f64) -> PyResult<f64> {
    celestial::next_last_quarter(JulianDay::new(jd_from)).map_err(to_py)
}

/// All 4 principal phase events for a calendar month.
/// Each item: (phase_name, jd, elongation)
#[pyfunction]
fn moon_phases_for_month(py: Python<'_>, year: i32, month: i32) -> PyResult<PyObject> {
    let events = celestial::moon_phases_for_month(year, month as u8).map_err(to_py)?;
    let result: Vec<PyObject> = events
        .into_iter()
        .map(|e| (e.phase.name(), e.jd, e.elongation).into_py_any(py).unwrap())
        .collect();
    Ok(result.into_py_any(py).unwrap())
}

/// Full Moon phase info: phase, illumination, prev/next principal phase.
/// Returns (phase_name, elongation, illumination,
///          prev_phase_name, prev_phase_jd,
///          next_phase_name, next_phase_jd, age_days)
#[pyfunction]
fn moon_phase_info(py: Python<'_>, jd: f64) -> PyResult<PyObject> {
    let info = celestial::moon_phase_info(JulianDay::new(jd)).map_err(to_py)?;
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
        .into_py_any(py).unwrap())
}

fn register_setup_fns(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(set_ephe_path, m)?)?;
    m.add_function(wrap_pyfunction!(set_jpl_file, m)?)?;
    m.add_function(wrap_pyfunction!(set_sid_mode, m)?)?;
    m.add_function(wrap_pyfunction!(set_topo, m)?)?;
    m.add_function(wrap_pyfunction!(set_delta_t_userdef, m)?)?;
    m.add_function(wrap_pyfunction!(close, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}

fn register_calc_fns(m: &Bound<'_, PyModule>) -> PyResult<()> {
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
    m.add_function(wrap_pyfunction!(ayanamsa, m)?)?;
    m.add_function(wrap_pyfunction!(ayanamsa_ut, m)?)?;
    m.add_function(wrap_pyfunction!(ayanamsa_name, m)?)?;
    m.add_function(wrap_pyfunction!(get_ayanamsa, m)?)?;
    m.add_function(wrap_pyfunction!(get_ayanamsa_name, m)?)?;
    m.add_function(wrap_pyfunction!(planet_name, m)?)?;
    Ok(())
}

fn register_houses_eclipses(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(houses, m)?)?;
    m.add_function(wrap_pyfunction!(houses_ex, m)?)?;
    m.add_function(wrap_pyfunction!(houses_ex2, m)?)?;
    m.add_function(wrap_pyfunction!(house_pos, m)?)?;
    m.add_function(wrap_pyfunction!(house_name, m)?)?;
    m.add_function(wrap_pyfunction!(house_name_str, m)?)?;
    m.add_function(wrap_pyfunction!(sol_eclipse_when_glob, m)?)?;
    m.add_function(wrap_pyfunction!(sol_eclipse_when_loc, m)?)?;
    m.add_function(wrap_pyfunction!(sol_eclipse_how, m)?)?;
    m.add_function(wrap_pyfunction!(sol_eclipse_where, m)?)?;
    m.add_function(wrap_pyfunction!(lun_eclipse_when, m)?)?;
    m.add_function(wrap_pyfunction!(lun_eclipse_when_loc, m)?)?;
    m.add_function(wrap_pyfunction!(lun_eclipse_how, m)?)?;
    m.add_function(wrap_pyfunction!(rise_trans, m)?)?;
    Ok(())
}

fn register_time_fns(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(julday, m)?)?;
    m.add_function(wrap_pyfunction!(revjul, m)?)?;
    m.add_function(wrap_pyfunction!(day_of_week, m)?)?;
    m.add_function(wrap_pyfunction!(deltat, m)?)?;
    m.add_function(wrap_pyfunction!(sidtime, m)?)?;
    m.add_function(wrap_pyfunction!(mean_sidtime, m)?)?;
    m.add_function(wrap_pyfunction!(utc_to_jd, m)?)?;
    m.add_function(wrap_pyfunction!(jdnow, m)?)?;
    m.add_function(wrap_pyfunction!(revjul_hms, m)?)?;
    m.add_function(wrap_pyfunction!(parse_datetime, m)?)?;
    m.add_function(wrap_pyfunction!(jd_duration, m)?)?;
    m.add_function(wrap_pyfunction!(jd_to_iso_string, m)?)?;
    Ok(())
}

fn register_coord_helpers(m: &Bound<'_, PyModule>) -> PyResult<()> {
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
    m.add_function(wrap_pyfunction!(degsplit, m)?)?;
    m.add_function(wrap_pyfunction!(degnorm, m)?)?;
    m.add_function(wrap_pyfunction!(difdeg2n, m)?)?;
    m.add_function(wrap_pyfunction!(parse_coord, m)?)?;
    m.add_function(wrap_pyfunction!(format_coord, m)?)?;
    Ok(())
}

#[pymodule]
fn _celestial_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    register_setup_fns(m)?;
    register_calc_fns(m)?;
    register_houses_eclipses(m)?;
    register_time_fns(m)?;
    register_coord_helpers(m)?;
    register_constants(m)?;
    register_chart_fns(m)?;
    register_calendar_fns(m)?;
    register_moon_fns(m)?;
    register_traditions_fns(m)?;
    m.add_function(wrap_pyfunction!(next_aspect_cusp2, m)?)?;
    m.add_function(wrap_pyfunction!(mooncross_node_ut, m)?)?;
    m.add_function(wrap_pyfunction!(mooncross_node, m)?)?;

    Ok(())
}

fn register_constants(m: &Bound<'_, PyModule>) -> PyResult<()> {
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
    Ok(())
}

fn register_chart_fns(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(match_aspect, m)?)?;
    m.add_function(wrap_pyfunction!(match_aspect2, m)?)?;
    m.add_function(wrap_pyfunction!(match_aspect3, m)?)?;
    m.add_function(wrap_pyfunction!(match_aspect4, m)?)?;
    m.add_function(wrap_pyfunction!(antiscion, m)?)?;
    m.add_function(wrap_pyfunction!(sign_name, m)?)?;
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
    m.add_function(wrap_pyfunction!(ic_transit_ut, m)?)?;
    m.add_function(wrap_pyfunction!(asc_transit_ut, m)?)?;
    m.add_function(wrap_pyfunction!(dsc_transit_ut, m)?)?;
    m.add_function(wrap_pyfunction!(next_aspect_cusp, m)?)?;
    m.add_function(wrap_pyfunction!(secondary_progressions, m)?)?;
    m.add_function(wrap_pyfunction!(solar_arc_directions, m)?)?;
    m.add_function(wrap_pyfunction!(midpoint_table, m)?)?;
    m.add_function(wrap_pyfunction!(calc_chart_aspects, m)?)?;
    m.add_function(wrap_pyfunction!(calc_chart_aspects_auto, m)?)?;
    m.add_function(wrap_pyfunction!(sexagenary_name, m)?)?;
    m.add_function(wrap_pyfunction!(monthly_profection, m)?)?;
    m.add_function(wrap_pyfunction!(egyptian_terms_ruler, m)?)?;
    m.add_function(wrap_pyfunction!(decan_ruler, m)?)?;
    m.add_function(wrap_pyfunction!(triplicity_rulers, m)?)?;
    m.add_function(wrap_pyfunction!(full_dignity, m)?)?;
    m.add_function(wrap_pyfunction!(almuten, m)?)?;
    m.add_function(wrap_pyfunction!(is_day_chart, m)?)?;
    m.add_function(wrap_pyfunction!(firdaria, m)?)?;
    m.add_function(wrap_pyfunction!(four_pillars, m)?)?;
    m.add_function(wrap_pyfunction!(solar_term_position, m)?)?;
    Ok(())
}

fn register_calendar_fns(m: &Bound<'_, PyModule>) -> PyResult<()> {
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(omer_from_jd, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(omer_day_jd, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(omer_start_jd, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(omer_days, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(omer_period, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(omer_declaration, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(jewish_holidays, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(jewish_holiday_jd, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(hebrew_year_from_jd, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(jd_to_hebrew_date, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(easter_gregorian, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(easter_orthodox, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(easter_jd, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(easter_orthodox_jd, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(christian_feasts, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(christian_fixed_feasts, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(hijri_from_jd, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(hijri_to_jd, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(hijri_month_name, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(islamic_observances, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(gregorian_to_hijri_years, m)?)?;
    m.add_function(wrap_pyfunction!(panchanga, m)?)?;
    m.add_function(wrap_pyfunction!(hindu_festivals, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(vesak_jd, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(uposatha_days, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(nowruz_jd, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(gregorian_to_solar_hijri, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(naw_ruz_jd, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(jd_to_bahai, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(bahai_holy_days, m)?)?;
    m.add_function(wrap_pyfunction!(iso_week, m)?)?;
    m.add_function(wrap_pyfunction!(day_of_year, m)?)?;
    m.add_function(wrap_pyfunction!(weeks_in_iso_year, m)?)?;
    m.add_function(wrap_pyfunction!(maya_long_count, m)?)?;
    m.add_function(wrap_pyfunction!(maya_long_count_str, m)?)?;
    m.add_function(wrap_pyfunction!(yallop_q, m)?)?;
    m.add_function(wrap_pyfunction!(best_time_method, m)?)?;
    m.add_function(wrap_pyfunction!(vietnamese_month_start_jd, m)?)?;
    m.add_function(wrap_pyfunction!(vietnamese_chinese_boundary_differs, m)?)?;
    m.add_function(wrap_pyfunction!(tonalpohualli, m)?)?;
    m.add_function(wrap_pyfunction!(xiuhpohualli, m)?)?;
    m.add_function(wrap_pyfunction!(tzolkin, m)?)?;
    m.add_function(wrap_pyfunction!(haab, m)?)?;
    m.add_function(wrap_pyfunction!(calendar_round, m)?)?;
    m.add_function(wrap_pyfunction!(medicine_wheel_totem, m)?)?;
    m.add_function(wrap_pyfunction!(egyptian_decan, m)?)?;
    m.add_function(wrap_pyfunction!(solar_cycle, m)?)?;
    m.add_function(wrap_pyfunction!(grand_solar_epoch, m)?)?;
    m.add_function(wrap_pyfunction!(cycle_nickname, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(sabbats_for_year, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(next_sabbat, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(sabbat_jd, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(esbats_for_year, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(next_esbat, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(next_full_moon, m)?)?;
    #[cfg(feature = "calendar-traditions")]
    m.add_function(wrap_pyfunction!(next_sabbat_name, m)?)?;
    m.add_function(wrap_pyfunction!(solcross_ut, m)?)?;
    Ok(())
}

fn register_moon_fns(m: &Bound<'_, PyModule>) -> PyResult<()> {
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

fn register_traditions_fns(m: &Bound<'_, PyModule>) -> PyResult<()> {
    #[cfg(feature = "calendar-traditions")]
    {
        m.add_function(wrap_pyfunction!(coptic_to_jd, m)?)?;
        m.add_function(wrap_pyfunction!(jd_to_coptic, m)?)?;
        m.add_function(wrap_pyfunction!(ethiopic_to_jd, m)?)?;
        m.add_function(wrap_pyfunction!(jd_to_ethiopic, m)?)?;
        m.add_function(wrap_pyfunction!(is_coptic_leap_year, m)?)?;
        m.add_function(wrap_pyfunction!(coptic_month_days, m)?)?;
        m.add_function(wrap_pyfunction!(fasli_nowruz_jd, m)?)?;
        m.add_function(wrap_pyfunction!(jd_to_fasli, m)?)?;
        m.add_function(wrap_pyfunction!(losar_jd, m)?)?;
        m.add_function(wrap_pyfunction!(tibetan_year_name, m)?)?;
    }
    let _ = m;
    Ok(())
}

// ─── Jewish holidays ──────────────────────────────────────────────────────────

/// All major Jewish holidays for the given Hebrew year.
/// Each item: (name, hebrew_name, hebrew_month, hebrew_day, jd, jd_end, days, category)
#[cfg(feature = "calendar-traditions")]
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
                .into_py_any(py).unwrap()
        })
        .collect();
    result.into_py_any(py).unwrap()
}

/// JD of a specific Jewish holiday by name in the given Hebrew year.
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn jewish_holiday_jd(hebrew_year: i32, name: &str) -> Option<f64> {
    celestial::jewish_holiday_jd(hebrew_year, name)
}

/// Hebrew year for a given Julian day.
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn hebrew_year_from_jd(jd: f64) -> i32 {
    celestial::hebrew_year_from_jd(JulianDay::new(jd))
}

/// Convert JD to Hebrew date → (year, month, day).
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn jd_to_hebrew_date(py: Python<'_>, jd: f64) -> PyObject {
    let (y, m, d) = celestial::jd_to_hebrew_date(JulianDay::new(jd));
    (y, m as i32, d as i32).into_py_any(py).unwrap()
}

// ─── Easter & Christian calendar ─────────────────────────────────────────────

/// Gregorian (Western) Easter → (year, month, day).
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn easter_gregorian(py: Python<'_>, year: i32) -> PyObject {
    let (y, m, d) = celestial::easter_gregorian(year);
    (y, m as i32, d as i32).into_py_any(py).unwrap()
}

/// Orthodox Easter in Gregorian calendar → (year, month, day).
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn easter_orthodox(py: Python<'_>, year: i32) -> PyObject {
    let (y, m, d) = celestial::easter_orthodox(year);
    (y, m as i32, d as i32).into_py_any(py).unwrap()
}

/// JD of Western Easter.
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn easter_jd(year: i32) -> f64 {
    celestial::easter_jd(year)
}

/// JD of Orthodox Easter.
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn easter_orthodox_jd(year: i32) -> f64 {
    celestial::easter_orthodox_jd(year)
}

/// All Western Christian moveable feasts for the year.
/// Each item: (name, easter_offset, jd, year, month, day)
#[cfg(feature = "calendar-traditions")]
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
                .into_py_any(py).unwrap()
        })
        .collect();
    result.into_py_any(py).unwrap()
}

/// Fixed (non-moveable) Christian feasts for the year.
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn christian_fixed_feasts(py: Python<'_>, year: i32) -> PyObject {
    let result: Vec<PyObject> = celestial::christian_fixed_feasts(year)
        .into_iter()
        .map(|f| (f.name, f.jd, f.year, f.month as i32, f.day as i32).into_py_any(py).unwrap())
        .collect();
    result.into_py_any(py).unwrap()
}

// ─── Islamic calendar ─────────────────────────────────────────────────────────

/// Convert JD to Hijri date → (year, month, day).
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn hijri_from_jd(py: Python<'_>, jd: f64) -> PyObject {
    let (y, m, d) = celestial::hijri_from_jd(JulianDay::new(jd));
    (y, m as i32, d as i32).into_py_any(py).unwrap()
}

/// Convert Hijri date to JD.
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn hijri_to_jd(year: i32, month: i32, day: i32) -> f64 {
    celestial::hijri_to_jd(year, month as u8, day as u8)
}

/// Hijri month name (1–12).
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn hijri_month_name(month: i32) -> &'static str {
    celestial::hijri_month_name(month as u8)
}

/// All major Islamic observances for the given Hijri year.
/// Each item: (name, arabic_name, hijri_month, hijri_day, jd, days)
#[cfg(feature = "calendar-traditions")]
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
                .into_py_any(py).unwrap()
        })
        .collect();
    result.into_py_any(py).unwrap()
}

/// Gregorian year → overlapping Hijri years → (year1, year2).
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn gregorian_to_hijri_years(py: Python<'_>, gregorian_year: i32) -> PyObject {
    let (y1, y2) = celestial::gregorian_to_hijri_years(gregorian_year);
    (y1, y2).into_py_any(py).unwrap()
}

// ─── Hindu Panchānga ─────────────────────────────────────────────────────────

/// Full Panchānga for a Julian day.
/// Returns (tithi, tithi_name, paksha, vara, vara_name,
///          nakshatra, nakshatra_name, nakshatra_pada,
///          yoga, yoga_name, karana, karana_name,
///          sun_lon, moon_lon, elongation)
#[pyfunction]
fn panchanga(py: Python<'_>, jd: f64) -> PyObject {
    let p = celestial::panchanga(JulianDay::new(jd));
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
    (part1, part2).into_py_any(py).unwrap()
}

/// Major Hindu festivals in the given Gregorian year.
/// Each item: (name, description, jd)
#[pyfunction]
fn hindu_festivals(py: Python<'_>, gregorian_year: i32) -> PyObject {
    let result: Vec<PyObject> = celestial::hindu_festivals(gregorian_year)
        .into_iter()
        .map(|f| (f.name, f.description, f.jd).into_py_any(py).unwrap())
        .collect();
    result.into_py_any(py).unwrap()
}

// ─── Buddhist observances ─────────────────────────────────────────────────────

/// JD of Vesak for the given Gregorian year.
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn vesak_jd(year: i32) -> f64 {
    celestial::vesak_jd(year)
}

/// All Uposatha days in the given Gregorian year.
/// Each item: (phase, jd, elongation)
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn uposatha_days(py: Python<'_>, year: i32) -> PyObject {
    let result: Vec<PyObject> = celestial::uposatha_days(year)
        .into_iter()
        .map(|u| {
            let phase = format!("{:?}", u.phase);
            (phase, u.jd, u.elongation).into_py_any(py).unwrap()
        })
        .collect();
    result.into_py_any(py).unwrap()
}

// ─── Nowruz & Bahá'í calendar ─────────────────────────────────────────────────

/// JD of Nowruz (vernal equinox / Persian New Year) for the given Gregorian year.
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn nowruz_jd(year: i32) -> f64 {
    celestial::nowruz_jd(year)
}

/// Convert Gregorian year to Solar Hijri (Persian) year.
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn gregorian_to_solar_hijri(year: i32) -> i32 {
    celestial::gregorian_to_solar_hijri(year)
}

/// JD of Naw-Rúz (Bahá'í New Year) for the given Bahá'í year.
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn naw_ruz_jd(bahai_year: i32) -> f64 {
    celestial::naw_ruz_jd(bahai_year)
}

/// Convert JD to Bahá'í date → (year, month, day, month_name).
#[cfg(feature = "calendar-traditions")]
#[pyfunction]
fn jd_to_bahai(py: Python<'_>, jd: f64) -> PyObject {
    let b = celestial::jd_to_bahai(JulianDay::new(jd));
    (b.year, b.month as i32, b.day as i32, b.month_name).into_py_any(py).unwrap()
}

/// Bahá'í holy days for the given Bahá'í year.
/// Each item: (name, description, bahai_month, bahai_day, jd)
#[cfg(feature = "calendar-traditions")]
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
                .into_py_any(py).unwrap()
        })
        .collect();
    result.into_py_any(py).unwrap()
}
