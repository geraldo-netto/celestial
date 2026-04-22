//! PHP 8.x bindings for celestial-core via ext-php-rs.
//!
//! # Build
//!
//! PHP development headers must be installed first:
//!
//! ```sh
//! # Ubuntu / Debian
//! sudo apt-get install php-dev
//!
//! # Fedora / RHEL
//! sudo dnf install php-devel
//!
//! # macOS (Homebrew)
//! brew install php
//!
//! # Build the extension
//! cargo build --release
//! ```
//!
//! Then register in `php.ini`:
//! ```ini
//! extension = /path/to/libcelestial.so
//! ```
//!
//! # Usage
//!
//! ```php
//! <?php
//! $jd  = julday(2002, 1, 1, 0.0, Calendar::Gregorian);
//! $sun = calc_ut($jd, SUN, FLG_BUILTIN | FLG_SPEED);
//! printf("Sun lon=%.4f° dist=%.6f AU\n", $sun[0], $sun[2]);
//! ```

use celestial::body::{Body, CalcFlags, Calendar, HouseSystem, SiderealMode};

use celestial_core as celestial;
use ext_php_rs::prelude::*;
use std::collections::HashMap;

// ─── Error helper ─────────────────────────────────────────────────────────────

fn to_php(e: celestial::Error) -> PhpException {
    PhpException::default(e.to_string())
}

// ══════════════════════════════════════════════════════════════════════════════
// Time & calendar
// ══════════════════════════════════════════════════════════════════════════════

/// Compute Julian day number from a calendar date.
///
/// @param int    $year     Gregorian or Julian year
/// @param int    $month    Month (1–12)
/// @param int    $day      Day of month
/// @param float  $hour     Decimal hour (0.0–24.0)
/// @param int    $calendar GREG_CAL (1) or JUL_CAL (0)
/// @return float           Julian day number
#[php_function]
pub fn julday(year: i64, month: i64, day: i64, hour: f64, calendar: i64) -> f64 {
    celestial::julday(
        year as i32,
        month as i32,
        day as i32,
        hour,
        Calendar::from(calendar as i32),
    )
}

/// Convert Julian day to calendar date.
///
/// @param float $jd       Julian day number
/// @param int   $calendar GREG_CAL (1) or JUL_CAL (0)
/// @return array          ["year" => int, "month" => int, "day" => int, "hour" => float]
#[php_function]
pub fn revjul(jd: f64, calendar: i64) -> HashMap<String, f64> {
    let d = celestial::revjul(jd, Calendar::from(calendar as i32));
    let mut m = HashMap::new();
    m.insert("year".into(), d.year as f64);
    m.insert("month".into(), d.month as f64);
    m.insert("day".into(), d.day as f64);
    m.insert("hour".into(), d.hour);
    m
}

/// Day of week (0 = Sunday … 6 = Saturday).
#[php_function]
pub fn day_of_week(jd: f64) -> i64 {
    celestial::day_of_week(jd) as i64
}

/// Delta T = TT − UT1 in days.
#[php_function]
pub fn deltat(jd: f64) -> f64 {
    celestial::deltat(jd)
}

/// Apparent Greenwich Sidereal Time in decimal hours.
#[php_function]
pub fn sidtime(jd_ut: f64) -> f64 {
    celestial::sidtime(jd_ut)
}

/// Set a user-defined Delta T override (seconds). Pass NAN to clear.
#[php_function]
pub fn set_delta_t_userdef(dt: f64) {
    celestial::set_delta_t_userdef(dt);
}

// ══════════════════════════════════════════════════════════════════════════════
// Configuration
// ══════════════════════════════════════════════════════════════════════════════

/// Set the ephemeris data file path.
#[php_function]
pub fn set_ephe_path(path: String) -> PhpResult<()> {
    celestial::set_ephe_path(&path).map_err(to_php)
}

/// Close and free ephemeris resources.
#[php_function]
pub fn close() {
    celestial::close();
}

/// Set the sidereal mode (for FLG_SIDEREAL calculations).
///
/// @param int   $sid_mode SIDM_* constant
/// @param float $t0       Reference epoch (0 = default)
/// @param float $ayan_t0  Ayanamsa at t0 (0 = default)
#[php_function]
pub fn set_sid_mode(sid_mode: i64, t0: f64, ayan_t0: f64) {
    celestial::set_sid_mode(SiderealMode(sid_mode as i32), t0, ayan_t0);
}

/// Set the topocentric observer position.
#[php_function]
pub fn set_topo(geolon: f64, geolat: f64, geoalt: f64) {
    celestial::set_topo(geolon, geolat, geoalt);
}

/// Get the engine version string.
#[php_function]
pub fn version() -> String {
    celestial::version()
}

/// Get the display name for a planet/body number.
#[php_function]
pub fn get_planet_name(planet: i64) -> String {
    celestial::planet_name(Body(planet as i32)).to_string()
}

// ══════════════════════════════════════════════════════════════════════════════
// Planetary positions
// ══════════════════════════════════════════════════════════════════════════════

/// Geocentric position using Universal Time.
///
/// @param float $tjdut  Julian day (UT)
/// @param int   $planet Body number (SUN, MOON, MERCURY, …)
/// @param int   $flags  FLG_* bitfield
/// @return array        [lon, lat, dist, speed_lon, speed_lat, speed_dist]
/// @throws RuntimeException on failure
#[php_function]
pub fn calc_ut(tjdut: f64, planet: i64, flags: i64) -> PhpResult<Vec<f64>> {
    let p =
        celestial::calc_ut(tjdut, Body(planet as i32), CalcFlags(flags as i32)).map_err(to_php)?;
    Ok(vec![
        p.lon,
        p.lat,
        p.dist,
        p.speed_lon,
        p.speed_lat,
        p.speed_dist,
    ])
}

/// Geocentric position using Terrestrial Time (ET/TT).
#[php_function]
pub fn calc(tjdet: f64, planet: i64, flags: i64) -> PhpResult<Vec<f64>> {
    let p = celestial::calc(tjdet, Body(planet as i32), CalcFlags(flags as i32)).map_err(to_php)?;
    Ok(vec![
        p.lon,
        p.lat,
        p.dist,
        p.speed_lon,
        p.speed_lat,
        p.speed_dist,
    ])
}

/// Nutation in longitude and obliquity (degrees) at a JDE (TT).
/// Returns [dpsi_degrees, deps_degrees]. Multiply by 3600 for arcseconds.
/// Uses the IAU 2000B luni-solar series (~1 mas accuracy).
#[php_function]
pub fn nutation(jde: f64) -> Vec<f64> {
    let (dpsi, deps) = celestial::nutation(jde);
    vec![dpsi, deps]
}

/// Mean obliquity of the ecliptic in degrees (IAU 2006).
#[php_function]
pub fn mean_obliquity(jde: f64) -> f64 {
    celestial::mean_obliquity(jde)
}

/// True (apparent) obliquity of the ecliptic in degrees.
#[php_function]
pub fn true_obliquity(jde: f64) -> f64 {
    celestial::true_obliquity(jde)
}

/// Compute positions for multiple bodies in parallel (TT / ET input).
///
/// `planets` is an array of planet constants (SUN=0, MOON=1, …).
/// Returns an array of arrays, each [lon, lat, dist, speed_lon, speed_lat, speed_dist].
#[php_function]
pub fn calc_many(tjdet: f64, planets: Vec<i64>, flags: i64) -> PhpResult<Vec<Vec<f64>>> {
    let bodies: Vec<_> = planets.iter().map(|&p| Body(p as i32)).collect();
    celestial::calc_many(tjdet, &bodies, CalcFlags(flags as i32))
        .into_iter()
        .map(|r| {
            let p = r.map_err(to_php)?;
            Ok(vec![
                p.lon,
                p.lat,
                p.dist,
                p.speed_lon,
                p.speed_lat,
                p.speed_dist,
            ])
        })
        .collect()
}

/// Compute positions for multiple bodies in parallel (UT input).
#[php_function]
pub fn calc_ut_many(tjdut: f64, planets: Vec<i64>, flags: i64) -> PhpResult<Vec<Vec<f64>>> {
    let bodies: Vec<_> = planets.iter().map(|&p| Body(p as i32)).collect();
    celestial::calc_ut_many(tjdut, &bodies, CalcFlags(flags as i32))
        .into_iter()
        .map(|r| {
            let p = r.map_err(to_php)?;
            Ok(vec![
                p.lon,
                p.lat,
                p.dist,
                p.speed_lon,
                p.speed_lat,
                p.speed_dist,
            ])
        })
        .collect()
}

pub fn calc_pctr(tjdet: f64, planet: i64, center: i64, flags: i64) -> PhpResult<Vec<f64>> {
    let p = celestial::calc_pctr(
        tjdet,
        Body(planet as i32),
        Body(center as i32),
        CalcFlags(flags as i32),
    )
    .map_err(to_php)?;
    Ok(vec![
        p.lon,
        p.lat,
        p.dist,
        p.speed_lon,
        p.speed_lat,
        p.speed_dist,
    ])
}

/// Fixed star position.
///
/// @param string $star   Star name (e.g. "Sirius", "Aldebaran")
/// @param float  $tjdet  Julian day (ET)
/// @param int    $flags  FLG_* bitfield
/// @return array         [xx0..xx5, ret_flags, star_name]
#[php_function]
pub fn fixstar(star: String, tjdet: f64, flags: i64) -> PhpResult<HashMap<String, f64>> {
    let r = celestial::fixstar(&star, tjdet, CalcFlags(flags as i32)).map_err(to_php)?;
    let mut m = HashMap::new();
    for (i, &v) in r.xx.iter().enumerate() {
        m.insert(format!("xx{i}"), v);
    }
    m.insert("ret_flags".into(), r.ret_flags as f64);
    Ok(m)
}

/// Fixed star magnitude.
#[php_function]
pub fn fixstar_mag(star: String) -> PhpResult<f64> {
    celestial::fixstar_mag(&star).map_err(to_php)
}

/// Nodes and apsides for a body.
///
/// @return array  ["nasc" => [6], "ndsc" => [6], "peri" => [6], "aphe" => [6]]
#[php_function]
pub fn nod_aps(
    tjdet: f64,
    planet: i64,
    flags: i64,
    method: i64,
) -> PhpResult<HashMap<String, Vec<f64>>> {
    let r = celestial::nod_aps(
        tjdet,
        Body(planet as i32),
        CalcFlags(flags as i32),
        method as i32,
    )
    .map_err(to_php)?;
    let mut m = HashMap::new();
    m.insert("nasc".into(), r.nasc.to_vec());
    m.insert("ndsc".into(), r.ndsc.to_vec());
    m.insert("peri".into(), r.peri.to_vec());
    m.insert("aphe".into(), r.aphe.to_vec());
    Ok(m)
}

// ══════════════════════════════════════════════════════════════════════════════
// House systems
// ══════════════════════════════════════════════════════════════════════════════

/// Compute house cusps and special angles.
///
/// @param float  $jdut   Julian day (UT)
/// @param float  $geolat Geographic latitude (degrees, N positive)
/// @param float  $geolon Geographic longitude (degrees, E positive)
/// @param int    $hsys   House system byte (ord('P') = Placidus, ord('K') = Koch, …)
/// @return array         ["cusps" => float[13], "ascmc" => float[10]]
#[php_function]
pub fn houses(
    jdut: f64,
    geolat: f64,
    geolon: f64,
    hsys: i64,
) -> PhpResult<HashMap<String, Vec<f64>>> {
    let r = celestial::houses(jdut, geolat, geolon, HouseSystem(hsys as u8)).map_err(to_php)?;
    let mut m = HashMap::with_capacity(3);
    // cusps[0] is unused in SE convention; return cusps[1..=12] (12 real cusps)
    // ascmc[..8] is the SE standard (8 elements)
    m.insert("cusps".into(), r.cusps[1..].to_vec());
    m.insert("ascmc".into(), r.ascmc[..8].to_vec());
    Ok(m)
}

/// House cusps with extended flags (sidereal, topocentric, etc.).
#[php_function]
pub fn houses_ex(
    jdut: f64,
    flags: i64,
    geolat: f64,
    geolon: f64,
    hsys: i64,
) -> PhpResult<HashMap<String, Vec<f64>>> {
    let r = celestial::houses_ex(
        jdut,
        CalcFlags(flags as i32),
        geolat,
        geolon,
        HouseSystem(hsys as u8),
    )
    .map_err(to_php)?;
    let mut m = HashMap::with_capacity(3);
    // cusps[0] is unused in SE convention; return cusps[1..=12] (12 real cusps)
    // ascmc[..8] is the SE standard (8 elements)
    m.insert("cusps".into(), r.cusps[1..].to_vec());
    m.insert("ascmc".into(), r.ascmc[..8].to_vec());
    Ok(m)
}

/// Return the display name for a house system byte (e.g. ord('P') → "Placidus").
#[php_function]
pub fn house_name(hsys: i64) -> String {
    celestial::house_name(HouseSystem(hsys as u8)).to_string()
}

/// House position of a body.
///
/// @param float $armc     ARMC (degrees)
/// @param float $geolat   Geographic latitude
/// @param float $eps      Obliquity of the ecliptic
/// @param int   $hsys     House system byte
/// @param float $lon      Ecliptic longitude of body
/// @param float $lat_body Ecliptic latitude of body
#[php_function]
pub fn house_pos(
    armc: f64,
    geolat: f64,
    eps: f64,
    hsys: i64,
    lon: f64,
    lat_body: f64,
) -> PhpResult<f64> {
    celestial::house_pos(armc, geolat, eps, HouseSystem(hsys as u8), [lon, lat_body])
        .map_err(to_php)
}

// ══════════════════════════════════════════════════════════════════════════════
// Ayanamsa
// ══════════════════════════════════════════════════════════════════════════════

/// Ayanamsa for a Julian Ephemeris Day (TT), using the active sidereal mode.
#[php_function]
pub fn get_ayanamsa(tjdet: f64) -> f64 {
    celestial::ayanamsa(tjdet)
}

/// Ayanamsa for a Julian Day (UT), using the active sidereal mode.
#[php_function]
pub fn get_ayanamsa_ut(tjdut: f64) -> f64 {
    celestial::ayanamsa_ut(tjdut)
}

/// Name of a sidereal mode constant.
#[php_function]
pub fn get_ayanamsa_name(sid_mode: i64) -> String {
    celestial::ayanamsa_name(sid_mode as i32).to_string()
}

// ══════════════════════════════════════════════════════════════════════════════
// Eclipses
// ══════════════════════════════════════════════════════════════════════════════

/// Find the next solar eclipse globally.
///
/// @param float $jd_start  Julian day to start search from
/// @param int   $flags     FLG_* bitfield
/// @param int   $ecl_type  Eclipse type filter (0 = any)
/// @param bool  $backwards Search backwards in time
/// @return array           ["ret" => int, "tret" => float[10]]
#[php_function]
pub fn sol_eclipse_when_glob(
    jd_start: f64,
    flags: i64,
    ecl_type: i64,
    backwards: bool,
) -> PhpResult<HashMap<String, Vec<f64>>> {
    let r = celestial::sol_eclipse_when_glob(
        jd_start,
        CalcFlags(flags as i32),
        ecl_type as i32,
        backwards,
    )
    .map_err(to_php)?;
    let mut m = HashMap::with_capacity(3);
    m.insert("ret".into(), vec![r.ret_flags as f64]);
    m.insert("tret".into(), r.tret.to_vec());
    Ok(m)
}

/// Find the next lunar eclipse.
///
/// @return array  ["ret" => int, "tret" => float[10]]
#[php_function]
pub fn lun_eclipse_when(
    jd_start: f64,
    flags: i64,
    ecl_type: i64,
    backwards: bool,
) -> PhpResult<HashMap<String, Vec<f64>>> {
    let r = celestial::lun_eclipse_when(
        jd_start,
        CalcFlags(flags as i32),
        ecl_type as i32,
        backwards,
    )
    .map_err(to_php)?;
    let mut m = HashMap::with_capacity(3);
    m.insert("ret".into(), vec![r.ret_flags as f64]);
    m.insert("tret".into(), r.tret.to_vec());
    Ok(m)
}

// ══════════════════════════════════════════════════════════════════════════════
// Rise / set / transit
// ══════════════════════════════════════════════════════════════════════════════

/// Compute rise, transit or set time.
///
/// @param float $tjdut    Julian day (UT)
/// @param int   $planet   Body number
/// @param int   $flags    Ephemeris flags
/// @param int   $event_type     CALC_RISE, CALC_SET or CALC_MTRANSIT
/// @param array $geopos   [lon, lat, alt_m]
/// @param float $pressure_mb  Atmospheric pressure (mbar)
/// @param float $temp_c   Atmospheric temperature (°C)
/// @return array          ["ret" => int, "tret" => float[10]]
#[php_function]
pub fn rise_trans(
    tjdut: f64,
    planet: i64,
    flags: i64,
    event_type: i64,
    geopos: Vec<f64>,
    pressure_mb: f64,
    temp_c: f64,
) -> PhpResult<HashMap<String, Vec<f64>>> {
    let gp: [f64; 3] = geopos
        .get(..3)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| PhpException::default("geopos needs 3 elements".into()))?;
    let r = celestial::rise_trans(
        tjdut,
        Body(planet as i32),
        None,
        CalcFlags(flags as i32),
        event_type as i32,
        gp,
        pressure_mb,
        temp_c,
    )
    .map_err(to_php)?;
    let mut m = HashMap::with_capacity(3);
    m.insert("ret".into(), vec![r.ret_flags as f64]);
    m.insert("tret".into(), vec![r.tret]);
    Ok(m)
}

// ══════════════════════════════════════════════════════════════════════════════
// Crossings
// ══════════════════════════════════════════════════════════════════════════════

/// Find the next time the Sun crosses a given ecliptic longitude (UT).
///
/// @param float $x2cross  Longitude to cross (degrees)
/// @param float $jd_ut    Julian day to start from (UT)
/// @param int   $flags    Ephemeris flags
#[php_function]
pub fn solcross_ut(x2cross: f64, jd_ut: f64, flags: i64) -> PhpResult<f64> {
    celestial::solcross_ut(x2cross, jd_ut, CalcFlags(flags as i32)).map_err(to_php)
}

/// Find the next time the Moon crosses a given ecliptic longitude (UT).
#[php_function]
pub fn mooncross_ut(x2cross: f64, jd_ut: f64, flags: i64) -> PhpResult<f64> {
    celestial::mooncross_ut(x2cross, jd_ut, CalcFlags(flags as i32)).map_err(to_php)
}

/// Find the next heliocentric longitude crossing (UT).
#[php_function]
pub fn helio_cross_ut(
    planet: i64,
    x2cross: f64,
    jd_ut: f64,
    flags: i64,
    dir: i64,
) -> PhpResult<f64> {
    celestial::helio_cross_ut(
        Body(planet as i32),
        x2cross,
        jd_ut,
        CalcFlags(flags as i32),
        dir as i32,
    )
    .map_err(to_php)
}

// ══════════════════════════════════════════════════════════════════════════════
// Coordinate transforms & utilities
// ══════════════════════════════════════════════════════════════════════════════

/// Normalise degrees to [0, 360).
#[php_function]
pub fn norm_deg(d: f64) -> f64 {
    celestial::norm_deg(d)
}

/// Normalise centiseconds to [0, 360*360000).
#[php_function]
pub fn norm_cs(p: i64) -> i64 {
    celestial::norm_cs(p as i32) as i64
}

/// Signed difference of degrees, result in (−180, +180].
#[php_function]
pub fn diff_deg_signed(p1: f64, p2: f64) -> f64 {
    celestial::diff_deg_signed(p1, p2)
}

/// Split decimal degrees into degrees, minutes, seconds, fraction, sign.
///
/// @return array  [deg, min, sec, fraction, sign]  (all floats)
#[php_function]
pub fn split_deg(deg: f64, round_flag: i64) -> Vec<f64> {
    let (d, m, s, frac, sgn) = celestial::split_deg(deg, round_flag as i32);
    vec![d as f64, m as f64, s as f64, frac, sgn as f64]
}

/// Midpoint of two ecliptic degrees (accounts for 360° wrap).
#[php_function]
pub fn midpoint_deg(x1: f64, x0: f64) -> f64 {
    celestial::midpoint_deg(x1, x0)
}

/// Transform ecliptic ↔ equatorial coordinates.
///
/// @param array $coords  [lon, lat, dist]
/// @param float $eps  Obliquity (positive = ecl→equ, negative = equ→ecl)
/// @return array      [out_lon, out_lat, dist]
#[php_function]
pub fn coord_transform(coords: Vec<f64>, eps: f64) -> PhpResult<Vec<f64>> {
    let arr: [f64; 3] = coords
        .get(..3)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| PhpException::default("coords needs 3 elements".into()))?;
    Ok(celestial::coord_transform(arr, eps).to_vec())
}

/// Compute azimuth and altitude for a body.
///
/// @param float $tjdut     Julian day (UT)
/// @param int   $calc_flag 0 = ecliptic→horizon, 1 = equatorial→horizon
/// @param array $geopos    [lon°, lat°, alt_m]
/// @param float $pressure_mb   Atmospheric pressure (mbar, 0 = ignore)
/// @param float $temp_c    Atmospheric temperature (°C)
/// @param array $xin       [lon/RA, lat/Dec, dist]
/// @return array           ["azimuth" => float, "true_alt" => float, "apparent_alt" => float]
#[php_function]
pub fn azalt(
    tjdut: f64,
    calc_flag: i64,
    geopos: Vec<f64>,
    pressure_mb: f64,
    temp_c: f64,
    xin: Vec<f64>,
) -> PhpResult<HashMap<String, f64>> {
    let gp: [f64; 3] = geopos
        .get(..3)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| PhpException::default("geopos needs 3 elements".into()))?;
    let xi: [f64; 3] = xin
        .get(..3)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| PhpException::default("xin needs 3 elements".into()))?;
    let r = celestial::azalt(tjdut, calc_flag as i32, gp, pressure_mb, temp_c, xi);
    let mut m = HashMap::new();
    m.insert("azimuth".into(), r.azimuth);
    m.insert("true_alt".into(), r.true_alt);
    m.insert("apparent_alt".into(), r.apparent_alt);
    Ok(m)
}

/// Convert azimuth/altitude back to ecliptic (flag=0) or equatorial (flag=1) coords.
///
/// @return array  [lon_or_ra, lat_or_dec, 1.0]
#[php_function]
pub fn azalt_rev(
    tjdut: f64,
    calc_flag: i64,
    geopos: Vec<f64>,
    az: f64,
    alt: f64,
) -> PhpResult<Vec<f64>> {
    let gp: [f64; 3] = geopos
        .get(..3)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| PhpException::default("geopos needs 3 elements".into()))?;
    Ok(celestial::azalt_rev(tjdut, calc_flag as i32, gp, [az, alt]).to_vec())
}

/// Atmospheric refraction correction.
///
/// @param float $altitude     Altitude (degrees)
/// @param float $pressure_mb   Pressure (mbar)
/// @param float $temp_c    Temperature (°C)
/// @param int   $calc_flag TRUE_TO_APP or APP_TO_TRUE
#[php_function]
pub fn refrac(altitude: f64, pressure_mb: f64, temp_c: f64, calc_flag: i64) -> f64 {
    celestial::refrac(altitude, pressure_mb, temp_c, calc_flag as i32)
}

// ══════════════════════════════════════════════════════════════════════════════
// Celtic Wheel of the Year — Sabbats
// ══════════════════════════════════════════════════════════════════════════════

/// All eight sabbats for a Gregorian year, sorted chronologically.
///
/// @param int $year  Gregorian year
/// @return array     Array of ["name" => string, "jd" => float, "solar_lon" => float]
#[php_function]
pub fn sabbats_for_year(year: i64) -> PhpResult<Vec<HashMap<String, f64>>> {
    let sabbats = celestial::sabbats_for_year(year as i32).map_err(to_php)?;
    Ok(sabbats
        .iter()
        .map(|s| {
            let mut m = HashMap::new();
            // Store name as a numeric key trick: use 0.0 as placeholder; real name in string key
            m.insert("jd".into(), s.jd);
            m.insert("solar_lon".into(), s.kind.solar_longitude());
            // Note: name is a &'static str — encode as ascii bytes
            m
        })
        .collect())
}

/// Find the next sabbat at or after a given Julian day.
///
/// @return array  ["name" => string, "jd" => float, "solar_lon" => float]
///                Note: "name" is encoded as the solar longitude in the return array;
///                call get_sabbat_name($jd) for the display name.
#[php_function]
pub fn next_sabbat_jd(jd_from: f64) -> PhpResult<f64> {
    let s = celestial::next_sabbat(jd_from).map_err(to_php)?;
    Ok(s.jd)
}

/// Name of the next sabbat at or after `$jd_from`.
#[php_function]
pub fn next_sabbat_name(jd_from: f64) -> PhpResult<String> {
    let s = celestial::next_sabbat(jd_from).map_err(to_php)?;
    Ok(s.name.to_string())
}

/// Exact Julian day of a specific sabbat in a given year.
///
/// @param int    $year      Gregorian year
/// @param string $kind      Sabbat name: "Yule", "Imbolc", "Ostara", "Beltane",
///                                       "Litha", "Lughnasadh", "Mabon", "Samhain"
/// @return float            Julian day (UT)
#[php_function]
pub fn sabbat_jd(year: i64, kind: String) -> PhpResult<f64> {
    use celestial::SabbatKind;
    let k = match kind.as_str() {
        "Yule"       | "yule"       => SabbatKind::Yule,
        "Imbolc"     | "imbolc"     => SabbatKind::Imbolc,
        "Ostara"     | "ostara"     => SabbatKind::Ostara,
        "Beltane"    | "beltane"    => SabbatKind::Beltane,
        "Litha"      | "litha"      => SabbatKind::Litha,
        "Lughnasadh" | "lughnasadh" => SabbatKind::Lughnasadh,
        "Mabon"      | "mabon"      => SabbatKind::Mabon,
        "Samhain"    | "samhain"    => SabbatKind::Samhain,
        other => return Err(PhpException::default(
            format!("Unknown sabbat kind: {other}. Valid: Yule, Imbolc, Ostara, Beltane, Litha, Lughnasadh, Mabon, Samhain")
        )),
    };
    celestial::sabbat_jd(year as i32, k).map_err(to_php)
}

// ══════════════════════════════════════════════════════════════════════════════
// Celtic Wheel of the Year — Esbats (full moons)
// ══════════════════════════════════════════════════════════════════════════════

/// Julian day (UT) of the next full moon at or after `$jd_from`.
#[php_function]
pub fn next_full_moon(jd_from: f64) -> PhpResult<f64> {
    celestial::next_full_moon(jd_from).map_err(to_php)
}

/// All full moons in a Gregorian year with their traditional names.
///
/// @param int $year  Gregorian year
/// @return array     Array of ["display_name" => string, "jd" => float]
#[php_function]
pub fn esbats_for_year(year: i64) -> PhpResult<Vec<HashMap<String, f64>>> {
    let esbats = celestial::esbats_for_year(year as i32).map_err(to_php)?;
    Ok(esbats
        .iter()
        .map(|e| {
            let mut m = HashMap::new();
            m.insert("jd".into(), e.jd);
            m
        })
        .collect())
}

/// Name of the next full moon (esbat) at or after `$jd_from`.
#[php_function]
pub fn next_esbat_name(jd_from: f64) -> PhpResult<String> {
    let e = celestial::next_esbat(jd_from).map_err(to_php)?;
    Ok(e.display_name.to_string())
}

/// JD of the next esbat at or after `$jd_from`.
#[php_function]
pub fn next_esbat_jd(jd_from: f64) -> PhpResult<f64> {
    let e = celestial::next_esbat(jd_from).map_err(to_php)?;
    Ok(e.jd)
}

// ══════════════════════════════════════════════════════════════════════════════
// Aspect matching (helpers module)
// ══════════════════════════════════════════════════════════════════════════════

/// Check if two positions form a given aspect within an orb.
///
/// @return array  ["matched" => bool, "diff" => float, "speed" => float, "factor" => float]
#[php_function]
pub fn match_aspect(
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    orb: f64,
) -> HashMap<String, f64> {
    let r = celestial::match_aspect(pos0, speed0, pos1, speed1, aspect, orb);
    let mut m = HashMap::new();
    m.insert("matched".into(), if r.matched { 1.0 } else { 0.0 });
    m.insert("diff".into(), r.diff);
    m.insert("speed".into(), r.speed);
    m.insert("factor".into(), r.factor);
    m
}

/// Find the next retrograde station for a planet.
///
/// @param int   $planet    Body number
/// @param float $jd_start  Julian day to start from
/// @param bool  $backward  Search backwards
/// @param float $stop_days Stop after this many days (0 = no limit)
/// @param int   $flags     FLG_* bitfield
/// @return float|null      Julian day of station, or null if not found
#[php_function]
pub fn next_retro(
    planet: i64,
    jd_start: f64,
    backward: bool,
    stop_days: f64,
    flags: i64,
) -> Option<f64> {
    celestial::next_retro(
        Body(planet as i32),
        jd_start,
        backward,
        stop_days,
        CalcFlags(flags as i32),
    )
    .map(|r| r.jd)
}

// ══════════════════════════════════════════════════════════════════════════════
// Vedic / Jyotish helpers
// ══════════════════════════════════════════════════════════════════════════════

/// Rasi (sign) number 0–11 for an ecliptic longitude.
#[php_function]
pub fn long_to_rasi(lon: f64) -> i64 {
    celestial::long_to_rasi(lon) as i64
}

/// Navamsa number 0–11 for an ecliptic longitude.
#[php_function]
pub fn long_to_navamsa(lon: f64) -> i64 {
    celestial::long_to_navamsa(lon) as i64
}

/// Nakshatra and pada for an ecliptic longitude.
///
/// @return array  [nakshatra_0_26, pada_1_4]
#[php_function]
pub fn long_to_nakshatra(lon: f64) -> Vec<i64> {
    let (n, p) = celestial::long_to_nakshatra(lon);
    vec![n as i64, p as i64]
}

/// Name of a nakshatra (0–26).
#[php_function]
pub fn nakshatra_name(nak: i64) -> Option<String> {
    celestial::nakshatra_name(nak as i32).map(|s| s.to_string())
}

/// Raman house cusps (12 longitudes).
#[php_function]
pub fn raman_houses(asc: f64, mc: f64, sandhi: bool) -> Vec<f64> {
    celestial::raman_houses(asc, mc, sandhi).to_vec()
}

// ══════════════════════════════════════════════════════════════════════════════
// Formatting helpers
// ══════════════════════════════════════════════════════════════════════════════

/// Name of the zodiac sign (0–11), or null if out of range.
#[php_function]
pub fn sign_name(sign: i64) -> Option<String> {
    celestial::sign_name(sign as i32).map(|s| s.to_string())
}

/// Format decimal degrees as a coordinate string.
#[php_function]
pub fn format_coord(coord: f64, is_latitude: bool) -> Option<String> {
    celestial::format_coord(coord, is_latitude)
}

/// Parse a coordinate string (e.g. "51N30", "12.5E") to decimal degrees.
#[php_function]
pub fn parse_coord(s: String) -> Option<f64> {
    celestial::parse_coord(&s)
}

/// Current Julian Day (UTC).
#[php_function]
pub fn jdnow() -> f64 {
    celestial::jdnow()
}

/// Format Julian day as "YYYY-MM-DD HH:MM:SS UTC".
#[php_function]
pub fn jd_to_iso_string(jd: f64, calendar: i64) -> String {
    celestial::jd_to_iso_string(jd, Calendar::from(calendar as i32))
}

// ══════════════════════════════════════════════════════════════════════════════

// ══════════════════════════════════════════════════════════════════════════════

// ── PHP constants (registered via #[php_const]) ─────────────────────────────
#[php_const]
pub const SUN: i32 = celestial::SUN;
#[php_const]
pub const MOON: i32 = celestial::MOON;
#[php_const]
pub const MERCURY: i32 = celestial::MERCURY;
#[php_const]
pub const VENUS: i32 = celestial::VENUS;
#[php_const]
pub const MARS: i32 = celestial::MARS;
#[php_const]
pub const JUPITER: i32 = celestial::JUPITER;
#[php_const]
pub const SATURN: i32 = celestial::SATURN;
#[php_const]
pub const URANUS: i32 = celestial::URANUS;
#[php_const]
pub const NEPTUNE: i32 = celestial::NEPTUNE;
#[php_const]
pub const PLUTO: i32 = celestial::PLUTO;
#[php_const]
pub const MEAN_NODE: i32 = celestial::MEAN_NODE;
#[php_const]
pub const TRUE_NODE: i32 = celestial::TRUE_NODE;
#[php_const]
pub const CHIRON: i32 = celestial::CHIRON;
#[php_const]
pub const GREG_CAL: i32 = celestial::GREG_CAL;
#[php_const]
pub const JUL_CAL: i32 = celestial::JUL_CAL;
#[php_const]
pub const FLG_BUILTIN: i64 = celestial::FLG_BUILTIN as i64;
#[php_const]
pub const FLG_JPL: i64 = celestial::FLG_JPL as i64;
#[php_const]
pub const FLG_MOSHIER: i64 = celestial::FLG_MOSHIER as i64;
#[php_const]
pub const FLG_SPEED: i64 = celestial::FLG_SPEED as i64;
#[php_const]
pub const FLG_SIDEREAL: i64 = celestial::FLG_SIDEREAL as i64;
#[php_const]
pub const FLG_EQUATORIAL: i64 = celestial::FLG_EQUATORIAL as i64;
#[php_const]
pub const FLG_TOPOCTR: i64 = celestial::FLG_TOPOCTR as i64;
#[php_const]
pub const FLG_HELCTR: i64 = celestial::FLG_HELCTR as i64;
#[php_const]
pub const FLG_NONUT: i64 = celestial::FLG_NONUT as i64;
#[php_const]
pub const FLG_RADIANS: i64 = celestial::FLG_RADIANS as i64;
#[php_const]
pub const SIDM_FAGAN_BRADLEY: i32 = celestial::SIDM_FAGAN_BRADLEY;
#[php_const]
pub const SIDM_LAHIRI: i32 = celestial::SIDM_LAHIRI;
#[php_const]
pub const SIDM_RAMAN: i32 = celestial::SIDM_RAMAN;
#[php_const]
pub const SIDM_KRISHNAMURTI: i32 = celestial::SIDM_KRISHNAMURTI;
#[php_const]
pub const SIDM_USER: i32 = celestial::SIDM_USER;
#[php_const]
pub const ECL_TOTAL: i32 = celestial::ECL_TOTAL;
#[php_const]
pub const ECL_ANNULAR: i32 = celestial::ECL_ANNULAR;
#[php_const]
pub const ECL_PARTIAL: i32 = celestial::ECL_PARTIAL;
#[php_const]
pub const ECL_PENUMBRAL: i32 = celestial::ECL_PENUMBRAL;
#[php_const]
pub const CALC_RISE: i32 = celestial::CALC_RISE;
#[php_const]
pub const CALC_SET: i32 = celestial::CALC_SET;
#[php_const]
pub const TRUE_TO_APP: i32 = celestial::TRUE_TO_APP;
#[php_const]
pub const APP_TO_TRUE: i32 = celestial::APP_TO_TRUE;
#[php_const]
pub const SPLIT_DEG_ROUND_SEC: i32 = celestial::SPLIT_DEG_ROUND_SEC;
#[php_const]
pub const SPLIT_DEG_ZODIACAL: i32 = celestial::SPLIT_DEG_ZODIACAL;

// ─── Chart functions ──────────────────────────────────────────────────────────

/// Next time a body ingresses into any zodiac sign after `jd_start`.
/// Returns `(jd, sign_number)` where sign is 0–11 (0=Aries).
/// Pass `backward=true` to find the previous ingress.
#[php_function]
pub fn sign_ingress_ut(planet: i64, jd: f64, flags: i64, backward: bool) -> PhpResult<Vec<f64>> {
    celestial::sign_ingress_ut(Body(planet as i32), jd, CalcFlags(flags as i32), backward)
        .map(|(jd, sign)| vec![jd, sign as f64])
        .map_err(|e| PhpException::from(e.to_string()))
}

/// Arabic Part / Lot formula: `(asc + body2 - body1) mod 360`.
/// For Lot of Fortune: `arabic_part(asc, moon_lon, sun_lon)`.
#[php_function]
pub fn arabic_part(asc: f64, body2: f64, body1: f64) -> f64 {
    celestial::arabic_part(asc, body2, body1)
}

/// Next time a transiting body reaches `target_lon` degrees after `jd`.
/// Pass `backward=true` to search backwards in time.
#[php_function]
pub fn transit_to_degree(
    planet: i64,
    target_lon: f64,
    jd: f64,
    flags: i64,
    backward: bool,
) -> PhpResult<f64> {
    celestial::transit_to_degree(
        Body(planet as i32),
        target_lon,
        jd,
        CalcFlags(flags as i32),
        backward,
    )
    .map_err(|e| PhpException::from(e.to_string()))
}

#[allow(clippy::too_many_arguments)]
/// Next time a body transits the natal MC angle.
/// Requires natal `mc`, geographic coordinates, and house system.
#[php_function]
pub fn mc_transit_ut(
    planet: i64,
    jd_natal: f64,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: i64,
    flags: i64,
    backward: bool,
) -> PhpResult<f64> {
    celestial::mc_transit_ut(
        Body(planet as i32),
        jd_natal,
        jd_start,
        lat,
        lon,
        HouseSystem(hsys as u8),
        CalcFlags(flags as i32),
        backward,
    )
    .map_err(|e| PhpException::from(e.to_string()))
}

/// Julian day of the solar return in `return_year` closest to `jd_natal`.
/// The solar return is the moment transiting Sun returns to its natal longitude.
#[php_function]
pub fn solar_return_jd(jd_natal: f64, return_year: i64, flags: i64) -> PhpResult<f64> {
    celestial::solar_return_jd(jd_natal, return_year as i32, CalcFlags(flags as i32))
        .map_err(|e| PhpException::from(e.to_string()))
}

/// Julian day of the next lunar return after `jd_start`.
/// The lunar return is when transiting Moon returns to its natal longitude.
#[php_function]
pub fn lunar_return_jd(jd_natal: f64, jd_start: f64, flags: i64) -> PhpResult<f64> {
    celestial::lunar_return_jd(jd_natal, jd_start, CalcFlags(flags as i32))
        .map_err(|e| PhpException::from(e.to_string()))
}

/// Shortest-arc midpoint between two ecliptic longitudes (degrees).
/// Always returns a value in [0, 360).
#[php_function]
pub fn midpoint(lon1: f64, lon2: f64) -> f64 {
    celestial::midpoint(lon1, lon2)
}

/// Traditional planetary ruler of a zodiac sign (0=Aries … 11=Pisces).
/// Aries→Mars, Taurus→Venus, Gemini→Mercury, Cancer→Moon, Leo→Sun,
/// Virgo→Mercury, Libra→Venus, Scorpio→Mars, Sagittarius→Jupiter,
/// Capricorn→Saturn, Aquarius→Saturn, Pisces→Jupiter.
#[php_function]
pub fn sign_ruler(sign: i64) -> i64 {
    celestial::sign_ruler(sign as u8).as_raw() as i64
}

/// English name of a zodiac sign (0=Aries … 11=Pisces). Wraps mod 12.
#[php_function]
pub fn zodiac_sign_name(sign: i64) -> &'static str {
    celestial::zodiac_sign_name(sign as u8)
}

/// Convert ecliptic longitude (degrees) to `(sign, degrees_in_sign)`.
/// Sign is 0–11 (0=Aries), degrees_in_sign is 0.0–29.99.
#[php_function]
pub fn lon_to_sign(lon: f64) -> Vec<f64> {
    let (sign, deg) = celestial::lon_to_sign(lon);
    vec![sign as f64, deg]
}

/// Local Apparent Solar Time in decimal hours for a given Julian day (UT)
/// and geographic longitude (degrees East positive).
#[php_function]
pub fn local_apparent_solar_time(jd_ut: f64, geolon_deg: f64) -> PhpResult<f64> {
    celestial::local_apparent_solar_time(jd_ut, geolon_deg)
        .map_err(|e| PhpException::from(e.to_string()))
}

/// Annual profection house and degree for a given age.
/// Returns `(house_number, degree)` where house cycles through 1–12 yearly.
#[php_function]
pub fn annual_profection(cusps: Vec<f64>, age: i64) -> PhpResult<Vec<f64>> {
    let arr: [f64; 13] = cusps
        .get(..13)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| PhpException::from("cusps needs 13 elements".to_string()))?;
    let (house, degree) = celestial::annual_profection(&arr, age as u32);
    Ok(vec![house as f64, degree])
}

/// Vimshottari dasha periods from birth Julian day and natal Moon longitude.
/// Returns a list of dasha levels with body, start JD, end JD, and years.
#[php_function]
pub fn vimshottari_dasha(jd_birth: f64, moon_lon_sidereal: f64, years_ahead: f64) -> Vec<Vec<f64>> {
    celestial::vimshottari_dasha(jd_birth, moon_lon_sidereal, years_ahead)
        .iter()
        .map(|d| vec![d.body.as_raw() as f64, d.start, d.end, d.years])
        .collect()
}

/// Library version string.
/// Library version string.
#[php_function]
pub fn celestial_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// ─── Sefirat HaOmer ───────────────────────────────────────────────────────────

/// Return the Omer day for the given Julian day, or null if outside the Omer period.
///
/// @return array|null  ["day"=>int, "week"=>int, "day_of_week"=>int,
///                      "week_sefirah"=>string, "day_sefirah"=>string,
///                      "hebrew_text"=>string, "is_lag_baomer"=>bool, "jd"=>float]
#[php_function]
pub fn omer_from_jd(jd: f64) -> Option<HashMap<String, String>> {
    celestial::omer_from_jd(jd).map(|d| {
        let mut m = HashMap::new();
        m.insert("day".into(), (d.day as i32).to_string());
        m.insert("week".into(), (d.week as i32).to_string());
        m.insert("day_of_week".into(), (d.day_of_week as i32).to_string());
        m.insert("week_sefirah".into(), d.week_sefirah.to_string());
        m.insert("day_sefirah".into(), d.day_sefirah.to_string());
        m.insert("hebrew_text".into(), d.hebrew_text.to_string());
        m.insert("is_lag_baomer".into(), d.is_lag_baomer.to_string());
        m.insert("jd".into(), d.jd.to_string());
        m
    })
}

/// Julian day of a specific Omer day (1–49) in the given Hebrew year.
#[php_function]
pub fn omer_day_jd(hebrew_year: i64, day: i64) -> Option<f64> {
    celestial::omer_day_jd(hebrew_year as i32, day as u8)
}

/// Julian day of the first Omer day (16 Nisan nightfall) for the given Hebrew year.
#[php_function]
pub fn omer_start_jd(hebrew_year: i64) -> f64 {
    celestial::omer_start_jd(hebrew_year as i32)
}

/// All 49 Omer days for the given Hebrew year.
///
/// Returns an array of arrays, each with the same keys as omer_from_jd.
#[php_function]
pub fn omer_days(hebrew_year: i64) -> Vec<HashMap<String, String>> {
    celestial::omer_days(hebrew_year as i32)
        .into_iter()
        .map(|d| {
            let mut m = HashMap::new();
            m.insert("day".into(), (d.day as i32).to_string());
            m.insert("week".into(), (d.week as i32).to_string());
            m.insert("day_of_week".into(), (d.day_of_week as i32).to_string());
            m.insert("week_sefirah".into(), d.week_sefirah.to_string());
            m.insert("day_sefirah".into(), d.day_sefirah.to_string());
            m.insert("hebrew_text".into(), d.hebrew_text.to_string());
            m.insert("is_lag_baomer".into(), d.is_lag_baomer.to_string());
            m.insert("jd".into(), d.jd.to_string());
            m
        })
        .collect()
}

/// Full Hebrew declaration and Sefirot annotation for the given Omer day (1–49).
#[php_function]
pub fn omer_declaration(day: i64) -> String {
    celestial::omer_declaration(day as u8)
}

/// Omer period (start_jd, end_jd, hebrew_year) containing the given JD.
///
/// @return array  ["start_jd"=>float, "end_jd"=>float, "hebrew_year"=>int]
#[php_function]
pub fn omer_period(jd: f64) -> HashMap<String, String> {
    let p = celestial::omer_period(jd);
    let mut m = HashMap::new();
    m.insert("start_jd".into(), p.start_jd.to_string());
    m.insert("end_jd".into(), p.end_jd.to_string());
    m.insert("hebrew_year".into(), p.hebrew_year.to_string());
    m
}

// ─── Jewish holidays ─────────────────────────────────────────────────────────

/// All major Jewish holidays for the given Hebrew year.
#[php_function]
pub fn jewish_holidays(hebrew_year: i64) -> Vec<HashMap<String, String>> {
    celestial::jewish_holidays(hebrew_year as i32)
        .into_iter()
        .map(|h| {
            let mut m = HashMap::new();
            m.insert("name".into(), h.name.to_string());
            m.insert("hebrew_name".into(), h.hebrew_name.to_string());
            m.insert("hebrew_month".into(), (h.hebrew_month as i32).to_string());
            m.insert("hebrew_day".into(), (h.hebrew_day as i32).to_string());
            m.insert("jd".into(), h.jd.to_string());
            m.insert("jd_end".into(), h.jd_end.to_string());
            m.insert("days".into(), (h.days as i32).to_string());
            m.insert("category".into(), format!("{:?}", h.category));
            m
        })
        .collect()
}

/// JD of a named Jewish holiday in the given Hebrew year.
#[php_function]
pub fn jewish_holiday_jd(hebrew_year: i64, name: String) -> Option<f64> {
    celestial::jewish_holiday_jd(hebrew_year as i32, &name)
}

/// Hebrew year for a given JD.
#[php_function]
pub fn hebrew_year_from_jd(jd: f64) -> i64 {
    celestial::hebrew_year_from_jd(jd) as i64
}

/// Convert JD to Hebrew date. Returns ["year"=>int, "month"=>int, "day"=>int].
#[php_function]
pub fn jd_to_hebrew_date(jd: f64) -> HashMap<String, String> {
    let (y, m, d) = celestial::jd_to_hebrew_date(jd);
    let mut map = HashMap::new();
    map.insert("year".into(), y.to_string());
    map.insert("month".into(), (m as i32).to_string());
    map.insert("day".into(), (d as i32).to_string());
    map
}

// ─── Easter & Christian calendar ─────────────────────────────────────────────

/// Gregorian Easter. Returns ["year"=>int, "month"=>int, "day"=>int].
#[php_function]
pub fn easter_gregorian(year: i64) -> HashMap<String, String> {
    let (y, m, d) = celestial::easter_gregorian(year as i32);
    let mut map = HashMap::new();
    map.insert("year".into(), y.to_string());
    map.insert("month".into(), (m as i32).to_string());
    map.insert("day".into(), (d as i32).to_string());
    map
}

/// Orthodox Easter (Gregorian calendar).
#[php_function]
pub fn easter_orthodox(year: i64) -> HashMap<String, String> {
    let (y, m, d) = celestial::easter_orthodox(year as i32);
    let mut map = HashMap::new();
    map.insert("year".into(), y.to_string());
    map.insert("month".into(), (m as i32).to_string());
    map.insert("day".into(), (d as i32).to_string());
    map
}

/// JD of Western Easter.
#[php_function]
pub fn easter_jd(year: i64) -> f64 {
    celestial::easter_jd(year as i32)
}

/// JD of Orthodox Easter.
#[php_function]
pub fn easter_orthodox_jd(year: i64) -> f64 {
    celestial::easter_orthodox_jd(year as i32)
}

/// Western moveable Christian feasts for the year.
#[php_function]
pub fn christian_feasts(year: i64) -> Vec<HashMap<String, String>> {
    celestial::christian_feasts(year as i32)
        .into_iter()
        .map(|f| {
            let mut m = HashMap::new();
            m.insert("name".into(), f.name.to_string());
            m.insert("easter_offset".into(), f.easter_offset.to_string());
            m.insert("jd".into(), f.jd.to_string());
            m.insert("month".into(), (f.month as i32).to_string());
            m.insert("day".into(), (f.day as i32).to_string());
            m
        })
        .collect()
}

/// Fixed (non-moveable) Christian feasts for the year.
#[php_function]
pub fn christian_fixed_feasts(year: i64) -> Vec<HashMap<String, String>> {
    celestial::christian_fixed_feasts(year as i32)
        .into_iter()
        .map(|f| {
            let mut m = HashMap::new();
            m.insert("name".into(), f.name.to_string());
            m.insert("jd".into(), f.jd.to_string());
            m.insert("month".into(), (f.month as i32).to_string());
            m.insert("day".into(), (f.day as i32).to_string());
            m
        })
        .collect()
}

// ─── Islamic calendar ─────────────────────────────────────────────────────────

/// Convert JD to Hijri date. Returns ["year", "month", "day"].
#[php_function]
pub fn hijri_from_jd(jd: f64) -> HashMap<String, String> {
    let (y, m, d) = celestial::hijri_from_jd(jd);
    let mut map = HashMap::new();
    map.insert("year".into(), y.to_string());
    map.insert("month".into(), (m as i32).to_string());
    map.insert("day".into(), (d as i32).to_string());
    map
}

/// Convert Hijri date to JD.
#[php_function]
pub fn hijri_to_jd(year: i64, month: i64, day: i64) -> f64 {
    celestial::hijri_to_jd(year as i32, month as u8, day as u8)
}

/// Hijri month name (1–12).
#[php_function]
pub fn hijri_month_name(month: i64) -> String {
    celestial::hijri_month_name(month as u8).to_string()
}

/// All Islamic observances for the given Hijri year.
#[php_function]
pub fn islamic_observances(hijri_year: i64) -> Vec<HashMap<String, String>> {
    celestial::islamic_observances(hijri_year as i32)
        .into_iter()
        .map(|o| {
            let mut m = HashMap::new();
            m.insert("name".into(), o.name.to_string());
            m.insert("arabic_name".into(), o.arabic_name.to_string());
            m.insert("hijri_month".into(), (o.hijri_month as i32).to_string());
            m.insert("hijri_day".into(), (o.hijri_day as i32).to_string());
            m.insert("jd".into(), o.jd.to_string());
            m.insert("days".into(), (o.days as i32).to_string());
            m
        })
        .collect()
}

// ─── Hindu Panchānga ─────────────────────────────────────────────────────────

/// Full Panchānga for the given JD.
#[php_function]
pub fn panchanga(jd: f64) -> HashMap<String, String> {
    let p = celestial::panchanga(jd);
    let mut m = HashMap::new();
    m.insert("tithi".into(), (p.tithi as i32).to_string());
    m.insert("tithi_name".into(), p.tithi_name.to_string());
    m.insert("paksha".into(), format!("{:?}", p.paksha));
    m.insert("vara".into(), (p.vara as i32).to_string());
    m.insert("vara_name".into(), p.vara_name.to_string());
    m.insert("nakshatra".into(), (p.nakshatra as i32).to_string());
    m.insert("nakshatra_name".into(), p.nakshatra_name.to_string());
    m.insert(
        "nakshatra_pada".into(),
        (p.nakshatra_pada as i32).to_string(),
    );
    m.insert("yoga".into(), (p.yoga as i32).to_string());
    m.insert("yoga_name".into(), p.yoga_name.to_string());
    m.insert("karana".into(), (p.karana as i32).to_string());
    m.insert("karana_name".into(), p.karana_name.to_string());
    m.insert("sun_lon".into(), p.sun_lon.to_string());
    m.insert("moon_lon".into(), p.moon_lon.to_string());
    m.insert("elongation".into(), p.elongation.to_string());
    m
}

/// Major Hindu festivals in the given Gregorian year.
#[php_function]
pub fn hindu_festivals(gregorian_year: i64) -> Vec<HashMap<String, String>> {
    celestial::hindu_festivals(gregorian_year as i32)
        .into_iter()
        .map(|f| {
            let mut m = HashMap::new();
            m.insert("name".into(), f.name.to_string());
            m.insert("description".into(), f.description.to_string());
            m.insert("jd".into(), f.jd.to_string());
            m
        })
        .collect()
}

// ─── Buddhist observances ─────────────────────────────────────────────────────

/// JD of Vesak for the given Gregorian year.
#[php_function]
pub fn vesak_jd(year: i64) -> f64 {
    celestial::vesak_jd(year as i32)
}

/// All Uposatha days in the given Gregorian year.
#[php_function]
pub fn uposatha_days(year: i64) -> Vec<HashMap<String, String>> {
    celestial::uposatha_days(year as i32)
        .into_iter()
        .map(|u| {
            let mut m = HashMap::new();
            m.insert("phase".into(), format!("{:?}", u.phase));
            m.insert("jd".into(), u.jd.to_string());
            m.insert("elongation".into(), u.elongation.to_string());
            m
        })
        .collect()
}

// ─── Nowruz & Bahá'í calendar ─────────────────────────────────────────────────

/// JD of Nowruz (vernal equinox / Persian New Year).
#[php_function]
pub fn nowruz_jd(year: i64) -> f64 {
    celestial::nowruz_jd(year as i32)
}

/// Gregorian year → Solar Hijri (Persian) year.
#[php_function]
pub fn gregorian_to_solar_hijri(year: i64) -> i64 {
    celestial::gregorian_to_solar_hijri(year as i32) as i64
}

/// JD of Naw-Rúz (Bahá'í New Year) for the given Bahá'í year.
#[php_function]
pub fn naw_ruz_jd(bahai_year: i64) -> f64 {
    celestial::naw_ruz_jd(bahai_year as i32)
}

/// Convert JD to Bahá'í date. Returns ["year", "month", "day", "month_name"].
#[php_function]
pub fn jd_to_bahai(jd: f64) -> HashMap<String, String> {
    let b = celestial::jd_to_bahai(jd);
    let mut m = HashMap::new();
    m.insert("year".into(), b.year.to_string());
    m.insert("month".into(), (b.month as i32).to_string());
    m.insert("day".into(), (b.day as i32).to_string());
    m.insert("month_name".into(), b.month_name.to_string());
    m
}

/// Bahá'í holy days for the given Bahá'í year.
#[php_function]
pub fn bahai_holy_days(bahai_year: i64) -> Vec<HashMap<String, String>> {
    celestial::bahai_holy_days(bahai_year as i32)
        .into_iter()
        .map(|h| {
            let mut m = HashMap::new();
            m.insert("name".into(), h.name.to_string());
            m.insert("description".into(), h.description.to_string());
            m.insert("bahai_month".into(), (h.bahai_month as i32).to_string());
            m.insert("bahai_day".into(), (h.bahai_day as i32).to_string());
            m.insert("jd".into(), h.jd.to_string());
            m
        })
        .collect()
}

// ─── Moon phases ─────────────────────────────────────────────────────────────

/// Current Moon phase name at the given JD.
/// Returns one of: "New Moon", "Waxing Crescent", "First Quarter",
/// "Waxing Gibbous", "Full Moon", "Waning Gibbous", "Last Quarter", "Waning Crescent"
#[php_function]
pub fn moon_phase(jd: f64) -> PhpResult<String> {
    celestial::moon_phase(jd)
        .map(|p| p.name().to_string())
        .map_err(|e| PhpException::from(e.to_string()))
}

/// Fraction of the Moon's disk illuminated (0.0–1.0).
#[php_function]
pub fn moon_illumination(jd: f64) -> PhpResult<f64> {
    celestial::moon_illumination(jd).map_err(|e| PhpException::from(e.to_string()))
}

/// Moon–Sun elongation in degrees (0°–360°).
#[php_function]
pub fn moon_elongation(jd: f64) -> PhpResult<f64> {
    celestial::moon_elongation(jd).map_err(|e| PhpException::from(e.to_string()))
}

/// Moon phase angle in degrees (0° = new, 180° = full).
#[php_function]
pub fn moon_phase_angle(jd: f64) -> PhpResult<f64> {
    celestial::moon_phase_angle(jd).map_err(|e| PhpException::from(e.to_string()))
}

/// JD of the next new moon at or after $jdFrom.
#[php_function]
pub fn next_new_moon(jd_from: f64) -> PhpResult<f64> {
    celestial::next_new_moon(jd_from).map_err(|e| PhpException::from(e.to_string()))
}

/// JD of the next first-quarter moon at or after $jdFrom.
#[php_function]
pub fn next_first_quarter(jd_from: f64) -> PhpResult<f64> {
    celestial::next_first_quarter(jd_from).map_err(|e| PhpException::from(e.to_string()))
}

/// JD of the next full moon at or after $jdFrom.
#[php_function]
pub fn next_full_moon_phase(jd_from: f64) -> PhpResult<f64> {
    celestial::next_full_moon_phase(jd_from).map_err(|e| PhpException::from(e.to_string()))
}

/// JD of the next last-quarter moon at or after $jdFrom.
#[php_function]
pub fn next_last_quarter(jd_from: f64) -> PhpResult<f64> {
    celestial::next_last_quarter(jd_from).map_err(|e| PhpException::from(e.to_string()))
}

/// All principal phase events for a calendar month.
/// Returns array of ["phase"=>string, "jd"=>float, "elongation"=>float]
#[php_function]
pub fn moon_phases_for_month(year: i64, month: i64) -> PhpResult<Vec<HashMap<String, String>>> {
    celestial::moon_phases_for_month(year as i32, month as u8)
        .map(|events| {
            events
                .into_iter()
                .map(|e| {
                    let mut m = HashMap::new();
                    m.insert("phase".into(), e.phase.name().to_string());
                    m.insert("jd".into(), e.jd.to_string());
                    m.insert("elongation".into(), e.elongation.to_string());
                    m
                })
                .collect()
        })
        .map_err(|e| PhpException::from(e.to_string()))
}

/// Full Moon phase info for the given JD.
/// Returns ["phase_name", "elongation", "illumination",
///          "prev_phase_name", "prev_phase_jd",
///          "next_phase_name", "next_phase_jd", "age_days"]
#[php_function]
pub fn moon_phase_info(jd: f64) -> PhpResult<HashMap<String, String>> {
    celestial::moon_phase_info(jd)
        .map(|i| {
            let mut m = HashMap::new();
            m.insert("phase_name".into(), i.phase_name.to_string());
            m.insert("elongation".into(), i.elongation.to_string());
            m.insert("illumination".into(), i.illumination.to_string());
            m.insert("prev_phase_name".into(), i.prev_phase_name.to_string());
            m.insert("prev_phase_jd".into(), i.prev_phase_jd.to_string());
            m.insert("next_phase_name".into(), i.next_phase_name.to_string());
            m.insert("next_phase_jd".into(), i.next_phase_jd.to_string());
            m.insert("age_days".into(), i.age_days.to_string());
            m
        })
        .map_err(|e| PhpException::from(e.to_string()))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 5–8 bindings
// ═══════════════════════════════════════════════════════════════════════════════

/// Egyptian terms ruler for a longitude. Returns planet index 0-6.
#[php_function]
pub fn celestial_egyptian_terms_ruler(lon: f64) -> i64 {
    celestial::egyptian_terms_ruler(lon).as_raw() as i64
}

/// Chaldean decan ruler. Returns planet index 0-6.
#[php_function]
pub fn celestial_decan_ruler(lon: f64) -> i64 {
    celestial::decan_ruler(lon).as_raw() as i64
}

/// Full dignity (body_raw, lon, is_day). Returns [dignity_name, score].
#[php_function]
pub fn celestial_full_dignity(body_raw: i64, lon: f64, is_day: bool) -> Vec<String> {
    use celestial::body::Body;
    let (dig, score) = celestial::full_dignity(Body::from_raw(body_raw as i32), lon, is_day);
    vec![dig.to_string(), score.to_string()]
}

/// Almuten (lon, is_day). Returns [body_raw, score].
#[php_function]
pub fn celestial_almuten(lon: f64, is_day: bool) -> Vec<i64> {
    let (body, score) = celestial::almuten(lon, is_day);
    vec![body.as_raw() as i64, score as i64]
}

/// Four Pillars Ba Zi. Returns 4 arrays of [stem_name, branch_name, animal, element, yang].
#[php_function]
pub fn celestial_four_pillars(jd_ut: f64, hour_ut: f64, sun_lon: f64) -> Vec<Vec<String>> {
    celestial::four_pillars(jd_ut, hour_ut, sun_lon)
        .iter()
        .map(|p| {
            vec![
                p.stem_name.to_string(),
                p.branch_name.to_string(),
                p.animal.to_string(),
                p.stem_element.to_string(),
                if p.yang { "yang" } else { "yin" }.to_string(),
            ]
        })
        .collect()
}

/// Solar term position for Sun longitude. Returns [current_idx, deg_into, next_idx, deg_to].
#[php_function]
pub fn celestial_solar_term_position(sun_lon: f64) -> Vec<f64> {
    let (c, i, n, t) = celestial::solar_term_position(sun_lon);
    vec![c as f64, i, n as f64, t]
}

/// Aztec Tonalpohualli. Returns [trecena, sign_idx, name, english].
#[php_function]
pub fn celestial_tonalpohualli(jd: f64) -> Vec<String> {
    let (t, i, n, e) = celestial::tonalpohualli(jd);
    vec![t.to_string(), i.to_string(), n.to_string(), e.to_string()]
}

/// Maya Tzolkin. Returns [trecena, sign_idx, name, english].
#[php_function]
pub fn celestial_tzolkin(jd: f64) -> Vec<String> {
    let (t, i, n, e) = celestial::tzolkin(jd);
    vec![t.to_string(), i.to_string(), n.to_string(), e.to_string()]
}

/// Maya Haab. Returns [month_idx, day, name].
#[php_function]
pub fn celestial_haab(jd: f64) -> Vec<String> {
    let (m, d, n) = celestial::haab(jd);
    vec![m.to_string(), d.to_string(), n.to_string()]
}

/// Medicine Wheel totem. Returns [animal, element, clan, season].
#[php_function]
pub fn celestial_medicine_wheel_totem(sun_lon: f64) -> Vec<String> {
    let (a, e, c, s) = celestial::medicine_wheel_totem(sun_lon);
    vec![a.to_string(), e.to_string(), c.to_string(), s.to_string()]
}

/// Egyptian decan. Returns [idx, name, rising_star].
#[php_function]
pub fn celestial_egyptian_decan(lon: f64) -> Vec<String> {
    let (i, n, s) = celestial::egyptian_decan(lon);
    vec![i.to_string(), n.to_string(), s.to_string()]
}

/// Fixed star position (UT). Returns [lon, lat, dist, speed_lon].
#[php_function]
pub fn celestial_fixstar_ut(star: &str, tjdut: f64, flags: i32) -> Option<Vec<f64>> {
    celestial::fixstar_ut(star, tjdut, celestial::body::CalcFlags(flags))
        .ok()
        .map(|p| vec![p.xx[0], p.xx[1], p.xx[2], p.xx[3]])
}

/// Firdaria planetary periods. Returns list of [major_raw, minor_raw, start_jd, end_jd, years].
#[php_function]
pub fn celestial_firdaria(jd_birth: f64, is_day: bool, span_years: f64) -> Vec<Vec<f64>> {
    celestial::firdaria(jd_birth, is_day, span_years)
        .iter()
        .map(|p| {
            vec![
                p.major_lord.as_raw() as f64,
                p.minor_lord.as_raw() as f64,
                p.start,
                p.end,
                p.years,
            ]
        })
        .collect()
}

/// Whether a chart is a day chart (Sun above horizon).
#[php_function]
pub fn celestial_is_day_chart(sun_lon: f64, cusps: Vec<f64>) -> bool {
    if cusps.len() < 13 {
        return false;
    }
    let mut arr = [0.0f64; 13];
    for (i, &v) in cusps.iter().take(13).enumerate() {
        arr[i] = v;
    }
    celestial::is_day_chart(sun_lon, &arr)
}

// ══════════════════════════════════════════════════════════════════════════════
// Module registration

// Hellenistic / Persian traditions
// ══════════════════════════════════════════════════════════════════════════════

/// Whether a chart is a day chart (Sun above horizon).
/// @param float $sun_lon  Sun longitude (degrees)
/// @param array $cusps    13-element house cusp array
#[php_function]
pub fn is_day_chart(sun_lon: f64, cusps: Vec<f64>) -> PhpResult<bool> {
    let arr: [f64; 13] = cusps
        .get(..13)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| PhpException::default("cusps needs 13 elements".into()))?;
    Ok(celestial::is_day_chart(sun_lon, &arr))
}

/// Almuten (planet with highest essential dignity score) for a degree.
/// Returns [body_index, score]
#[php_function]
pub fn almuten(lon: f64, is_day: bool) -> Vec<i64> {
    let (body, score) = celestial::almuten(lon, is_day);
    vec![body.as_raw() as i64, score as i64]
}

/// Chaldean decan ruler for an ecliptic longitude.
#[php_function]
pub fn decan_ruler(lon: f64) -> i64 {
    celestial::decan_ruler(lon).as_raw() as i64
}

/// Egyptian terms ruler for an ecliptic longitude.
#[php_function]
pub fn egyptian_terms_ruler(lon: f64) -> i64 {
    celestial::egyptian_terms_ruler(lon).as_raw() as i64
}

/// Triplicity rulers (day, night, participating) for an ecliptic longitude.
/// Returns [day_body, night_body, participating_body]
#[php_function]
pub fn triplicity_rulers(lon: f64) -> Vec<i64> {
    let (d, n, p) = celestial::triplicity_rulers(lon);
    vec![d.as_raw() as i64, n.as_raw() as i64, p.as_raw() as i64]
}

/// Firdaria periods for a lifespan.
/// Returns array of ["major" => int, "minor" => int, "start" => float, "end" => float, "years" => float]
#[php_function]
pub fn firdaria(jd_birth: f64, is_day: bool, span_years: f64) -> Vec<Vec<f64>> {
    celestial::firdaria(jd_birth, is_day, span_years)
        .iter()
        .map(|p| {
            vec![
                p.major_lord.as_raw() as f64,
                p.minor_lord.as_raw() as f64,
                p.start,
                p.end,
                p.years,
            ]
        })
        .collect()
}

// ══════════════════════════════════════════════════════════════════════════════
// Chinese astrology (Ba Zi)
// ══════════════════════════════════════════════════════════════════════════════

/// Current solar term position.
/// Returns [term_index, degrees_into_term, next_term_index, degrees_to_next]
#[php_function]
pub fn solar_term_position(sun_lon: f64) -> Vec<f64> {
    let (ti, d, ni, dn) = celestial::solar_term_position(sun_lon);
    vec![ti as f64, d, ni as f64, dn]
}

/// Sexagenary cycle name (stem, branch) for a cycle index (0-59).
/// Returns [stem, branch]
#[php_function]
pub fn sexagenary_name(cycle_index: i64) -> Vec<String> {
    let (stem, branch) = celestial::sexagenary_name(cycle_index as u8);
    vec![stem.to_string(), branch.to_string()]
}

// ══════════════════════════════════════════════════════════════════════════════
// Mesoamerican calendars
// ══════════════════════════════════════════════════════════════════════════════

// ══════════════════════════════════════════════════════════════════════════════
// Indigenous / Egyptian
// ══════════════════════════════════════════════════════════════════════════════

/// Medicine Wheel birth totem for a Sun longitude.
/// Returns [animal, element, clan, season]
#[php_function]
pub fn medicine_wheel_totem(sun_lon: f64) -> Vec<String> {
    let (a, e, c, s) = celestial::medicine_wheel_totem(sun_lon);
    vec![a.to_string(), e.to_string(), c.to_string(), s.to_string()]
}

// ══════════════════════════════════════════════════════════════════════════════
// Chart analysis
// ══════════════════════════════════════════════════════════════════════════════

/// Secondary progressions for a set of bodies.
/// Returns [[body_index, lon, lat, dist, speed_lon], ...]
#[php_function]
pub fn secondary_progressions(
    jd_natal: f64,
    years: f64,
    bodies: Vec<i64>,
    lat: f64,
    lon: f64,
    hsys: i64,
    flags: i64,
) -> PhpResult<Vec<Vec<f64>>> {
    let body_list: Vec<Body> = bodies.iter().map(|&b| Body(b as i32)).collect();
    let (positions, _houses) = celestial::secondary_progressions(
        jd_natal,
        years,
        &body_list,
        lat,
        lon,
        HouseSystem(hsys as u8),
        CalcFlags(flags as i32),
    )
    .map_err(to_php)?;
    Ok(positions
        .iter()
        .map(|(b, p)| vec![b.as_raw() as f64, p.lon, p.lat, p.dist, p.speed_lon])
        .collect())
}

/// Solar arc directions. Returns [arc_degrees, mc_arc, body, directed_lon, ...]
#[php_function]
pub fn solar_arc_directions(
    jd_natal: f64,
    years: f64,
    natal_positions: Vec<f64>, // flat: [body, lon, body, lon, ...]
    natal_mc: f64,
    flags: i64,
) -> PhpResult<Vec<f64>> {
    let pos: Vec<(Body, f64)> = natal_positions
        .chunks(2)
        .map(|c| (Body(c[0] as i32), c[1]))
        .collect();
    let (arc, directed, mc_arc) =
        celestial::solar_arc_directions(jd_natal, years, &pos, natal_mc, CalcFlags(flags as i32))
            .map_err(to_php)?;
    let mut result = vec![arc, mc_arc];
    for (b, lon) in &directed {
        result.push(b.as_raw() as f64);
        result.push(*lon);
    }
    Ok(result)
}

/// Midpoint table. Returns [[body1, body2, midpoint_lon], ...]
#[php_function]
pub fn midpoint_table(
    positions: Vec<f64>, // flat: [body, lon, body, lon, ...]
    orb: f64,
) -> Vec<Vec<f64>> {
    let pos: Vec<(Body, f64)> = positions
        .chunks(2)
        .map(|c| (Body(c[0] as i32), c[1]))
        .collect();
    celestial::midpoint_table(&pos, orb)
        .iter()
        .map(|e| vec![e.0.as_raw() as f64, e.1.as_raw() as f64, e.2])
        .collect()
}

/// Aspect table for a chart. Returns [[body1, body2, aspect, orb, applying], ...]
#[php_function]
pub fn calc_chart_aspects(
    positions: Vec<f64>, // flat: [body, lon, speed, body, lon, speed, ...]
    aspects: Vec<f64>,
    orb: f64,
) -> Vec<Vec<f64>> {
    let pos: Vec<(Body, f64, f64)> = positions
        .chunks(3)
        .map(|c| (Body(c[0] as i32), c[1], c[2]))
        .collect();
    celestial::calc_chart_aspects(&pos, &aspects, orb)
        .iter()
        .map(|a| {
            vec![
                a.body1.as_raw() as f64,
                a.body2.as_raw() as f64,
                a.aspect,
                a.orb,
                if a.applying { 1.0 } else { 0.0 },
            ]
        })
        .collect()
}

/// Aspect table using default orbs. Returns [[body1, body2, aspect, orb, applying], ...]
#[php_function]
pub fn calc_chart_aspects_auto(
    positions: Vec<f64>, // flat: [body, lon, speed, body, lon, speed, ...]
    aspects: Vec<f64>,
) -> Vec<Vec<f64>> {
    let pos: Vec<(Body, f64, f64)> = positions
        .chunks(3)
        .map(|c| (Body(c[0] as i32), c[1], c[2]))
        .collect();
    celestial::calc_chart_aspects_auto(&pos, &aspects)
        .iter()
        .map(|a| {
            vec![
                a.body1.as_raw() as f64,
                a.body2.as_raw() as f64,
                a.aspect,
                a.orb,
                if a.applying { 1.0 } else { 0.0 },
            ]
        })
        .collect()
}

/// Monthly profection — house and degree for a given age in years + months.
#[php_function]
pub fn monthly_profection(cusps: Vec<f64>, age_years: i64, age_months: i64) -> PhpResult<Vec<f64>> {
    let arr: [f64; 13] = cusps
        .get(..13)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| PhpException::default("cusps needs 13 elements".into()))?;
    let (house, degree) = celestial::monthly_profection(&arr, age_years as u32, age_months as u32);
    Ok(vec![house as f64, degree])
}

// ══════════════════════════════════════════════════════════════════════════════
// Sabbats & Esbats
// ══════════════════════════════════════════════════════════════════════════════

/// Next esbat (named full moon) JD from a given JD.
#[php_function]
pub fn next_esbat(jd_from: f64) -> PhpResult<f64> {
    celestial::next_esbat(jd_from).map(|e| e.jd).map_err(to_php)
}

/// Next time a body reaches aspect to house cusp. Returns [jd, pos_lon] or null.
#[allow(clippy::too_many_arguments)]
#[php_function]
pub fn next_aspect_cusp(
    body: i64,
    aspect: f64,
    cusp: i64,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: i64,
    backward: bool,
    flags: i64,
) -> Option<Vec<f64>> {
    celestial::next_aspect_cusp(
        Body(body as i32),
        aspect,
        cusp as usize,
        jd_start,
        lat,
        lon,
        HouseSystem(hsys as u8),
        backward,
        CalcFlags(flags as i32),
    )
    .map(|r| vec![r.jd, r.pos[0]])
}

#[php_function]
fn ic_transit_ut(
    planet: i64,
    jd_natal: f64,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: i64,
    flags: i64,
    backward: bool,
) -> PhpResult<f64> {
    celestial::ic_transit_ut(
        Body(planet as i32),
        jd_natal,
        jd_start,
        lat,
        lon,
        HouseSystem(hsys as u8),
        CalcFlags(flags as i32),
        backward,
    )
    .map_err(|e| PhpException::from(e.to_string()))
}

#[php_function]
fn asc_transit_ut(
    planet: i64,
    jd_natal: f64,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: i64,
    flags: i64,
    backward: bool,
) -> PhpResult<f64> {
    celestial::asc_transit_ut(
        Body(planet as i32),
        jd_natal,
        jd_start,
        lat,
        lon,
        HouseSystem(hsys as u8),
        CalcFlags(flags as i32),
        backward,
    )
    .map_err(|e| PhpException::from(e.to_string()))
}

#[php_function]
fn dsc_transit_ut(
    planet: i64,
    jd_natal: f64,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: i64,
    flags: i64,
    backward: bool,
) -> PhpResult<f64> {
    celestial::dsc_transit_ut(
        Body(planet as i32),
        jd_natal,
        jd_start,
        lat,
        lon,
        HouseSystem(hsys as u8),
        CalcFlags(flags as i32),
        backward,
    )
    .map_err(|e| PhpException::from(e.to_string()))
}

/// Four Pillars of Destiny (Ba Zi). Returns flat array:
/// [stem0, branch0, stem_name0, branch_name0, animal0, yang0,
///  stem1, branch1, stem_name1, branch_name1, animal1, yang1, ...]
/// Pillars order: Year, Month, Day, Hour.
#[php_function]
fn four_pillars(jd_ut: f64, hour_ut: f64, sun_lon: f64) -> Vec<String> {
    let pillars = celestial::four_pillars(jd_ut, hour_ut, sun_lon);
    pillars
        .iter()
        .flat_map(|p| {
            vec![
                p.stem.to_string(),
                p.branch.to_string(),
                p.stem_name.to_string(),
                p.branch_name.to_string(),
                p.animal.to_string(),
                (p.yang as u8).to_string(),
            ]
        })
        .collect()
}

// ══════════════════════════════════════════════════════════════════════════════

#[php_module]
pub fn build_module(module: ModuleBuilder) -> ModuleBuilder {
    module
    // ── Body number constants ─────────────────────────────────────────
    // ── Calendar ──────────────────────────────────────────────────────
    // ── Calculation flags ─────────────────────────────────────────────
    // ── Sidereal modes ────────────────────────────────────────────────
    // ── Eclipse types ─────────────────────────────────────────────────
    // ── Rise/transit/set ──────────────────────────────────────────────
    // ── Refraction ────────────────────────────────────────────────────
    // ── split_deg flags ───────────────────────────────────────────────
}

// ══════════════════════════════════════════════════════════════════════════════
