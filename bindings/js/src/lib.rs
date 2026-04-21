//! Node.js/TypeScript bindings for celestial-core via napi-rs.
//!
//! Build with: `napi build --platform --release`
//! TypeScript declarations are auto-generated into `index.d.ts`.

#![allow(clippy::too_many_arguments)]

use celestial::body::{Body, CalcFlags, Calendar, HouseSystem, SiderealMode};
use celestial_core as celestial;
use napi_derive::napi;

fn to_napi(e: celestial::Error) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

// ─── Returned object shapes ───────────────────────────────────────────────────

/// Planetary position vector returned by `calc` / `calcUt`.
#[napi(object)]
pub struct PlanetPos {
    /// Ecliptic longitude (degrees)
    pub lon: f64,
    /// Ecliptic latitude (degrees)
    pub lat: f64,
    /// Distance in AU
    pub dist: f64,
    /// Speed in longitude (deg/day)
    pub speed_lon: f64,
    /// Speed in latitude (deg/day)
    pub speed_lat: f64,
    /// Speed in distance (AU/day)
    pub speed_dist: f64,
    /// Return flags from the library
    pub ret_flags: i32,
}

/// Fixed star position.
#[napi(object)]
pub struct StarPos {
    /// 6-element position/speed array
    pub xx: Vec<f64>,
    /// Canonical star name (e.g. `"Sirius,alCMa"`)
    pub star_name: String,
    /// Return flags
    pub ret_flags: i32,
}

/// House calculation result.
#[napi(object)]
pub struct HouseResult {
    /// Cusp positions (12 or 36 values depending on house system)
    pub cusps: Vec<f64>,
    /// Additional points: ASC, MC, ARMC, Vertex, EquatorialASC, CoASC1, CoASC2, PolarASC
    pub ascmc: Vec<f64>,
}

/// House result with cusp speeds.
#[napi(object)]
pub struct HouseResultEx2 {
    pub cusps: Vec<f64>,
    pub ascmc: Vec<f64>,
    pub cusp_speeds: Vec<f64>,
    pub ascmc_speeds: Vec<f64>,
}

/// Eclipse or rise/transit result.
#[napi(object)]
pub struct EclipseResult {
    pub ret_flags: i32,
    /// Time array (10 values)
    pub tret: Vec<f64>,
}

/// Eclipse result with attribute array.
#[napi(object)]
pub struct EclipseResultAttr {
    pub ret_flags: i32,
    pub tret: Vec<f64>,
    /// Attribute array (20 values)
    pub attr: Vec<f64>,
}

/// Geographic position of eclipse centrality.
#[napi(object)]
pub struct EclipseWhere {
    pub ret_flags: i32,
    pub geopos: Vec<f64>,
    pub attr: Vec<f64>,
}

/// Eclipse attributes only.
#[napi(object)]
pub struct EclipseHow {
    pub ret_flags: i32,
    pub attr: Vec<f64>,
}

/// Rise/transit result.
#[napi(object)]
pub struct RiseTrans {
    pub ret_flags: i32,
    /// Julian day of the event
    pub tret: f64,
}

/// Azimuth / altitude.
#[napi(object)]
pub struct AzAlt {
    pub azimuth: f64,
    pub true_alt: f64,
    pub apparent_alt: f64,
}

/// `julday` / `revjul` calendar date.
#[napi(object)]
pub struct CalDate {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: f64,
}

/// UTC date with integer H:M:S.
#[napi(object)]
pub struct UtcDate {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: f64,
}

/// `utcToJd` result.
#[napi(object)]
pub struct JdPair {
    pub et: f64,
    pub ut1: f64,
}

/// Refraction extended result.
#[napi(object)]
pub struct RefracExt {
    pub result: f64,
    pub dret: Vec<f64>,
}

// ─── Configuration ────────────────────────────────────────────────────────────

/// Set the path to Swiss Ephemeris data files.
#[napi(js_name = "setEphePath")]
pub fn set_ephe_path(path: String) -> napi::Result<()> {
    celestial::set_ephe_path(&path).map_err(to_napi)
}

/// Set the JPL ephemeris file.
#[napi(js_name = "setJplFile")]
pub fn set_jpl_file(fname: String) -> napi::Result<()> {
    celestial::set_jpl_file(&fname).map_err(to_napi)
}

/// Set sidereal mode. `sidMode` is one of the `SIDM_*` constants.
#[napi(js_name = "setSidMode")]
pub fn set_sid_mode(sid_mode: i32, t0: Option<f64>, ayan_t0: Option<f64>) {
    celestial::set_sid_mode(
        SiderealMode(sid_mode),
        t0.unwrap_or(0.0),
        ayan_t0.unwrap_or(0.0),
    );
}

/// Set topocentric observer position.
#[napi(js_name = "setTopo")]
pub fn set_topo(geolon: f64, geolat: f64, geoalt: f64) {
    celestial::set_topo(geolon, geolat, geoalt);
}

/// Release all library resources.
#[napi]
pub fn close() {
    celestial::close();
}

// ─── Calculations ─────────────────────────────────────────────────────────────

/// Calculate planetary positions (Ephemeris Time).
#[napi]
pub fn calc(tjdet: f64, planet: i32, flags: i32) -> napi::Result<PlanetPos> {
    celestial::calc(tjdet, Body::from_raw(planet), CalcFlags(flags))
        .map(|p| PlanetPos {
            lon: p.lon,
            lat: p.lat,
            dist: p.dist,
            speed_lon: p.speed_lon,
            speed_lat: p.speed_lat,
            speed_dist: p.speed_dist,
            ret_flags: p.ret_flags,
        })
        .map_err(to_napi)
}

/// Calculate planetary positions (Universal Time).
#[napi(js_name = "calcUt")]
pub fn calc_ut(tjdut: f64, planet: i32, flags: i32) -> napi::Result<PlanetPos> {
    celestial::calc_ut(tjdut, Body::from_raw(planet), CalcFlags(flags))
        .map(|p| PlanetPos {
            lon: p.lon,
            lat: p.lat,
            dist: p.dist,
            speed_lon: p.speed_lon,
            speed_lat: p.speed_lat,
            speed_dist: p.speed_dist,
            ret_flags: p.ret_flags,
        })
        .map_err(to_napi)
}

/// Planetocentric positions (ET).
#[napi(js_name = "calcPctr")]
pub fn calc_pctr(tjdet: f64, planet: i32, center: i32, flags: i32) -> napi::Result<PlanetPos> {
    celestial::calc_pctr(
        tjdet,
        Body::from_raw(planet),
        Body::from_raw(center),
        CalcFlags(flags),
    )
    .map(|p| PlanetPos {
        lon: p.lon,
        lat: p.lat,
        dist: p.dist,
        speed_lon: p.speed_lon,
        speed_lat: p.speed_lat,
        speed_dist: p.speed_dist,
        ret_flags: p.ret_flags,
    })
    .map_err(to_napi)
}

/// Fixed star position (ET).
#[napi]
pub fn fixstar(star: String, tjdet: f64, flags: i32) -> napi::Result<StarPos> {
    celestial::fixstar(&star, tjdet, CalcFlags(flags))
        .map(|r| StarPos {
            xx: r.xx.to_vec(),
            star_name: r.star_name,
            ret_flags: r.ret_flags,
        })
        .map_err(to_napi)
}

/// Fixed star position (UT).
#[napi(js_name = "fixstarUt")]
pub fn fixstar_ut(star: String, tjdut: f64, flags: i32) -> napi::Result<StarPos> {
    celestial::fixstar_ut(&star, tjdut, CalcFlags(flags))
        .map(|r| StarPos {
            xx: r.xx.to_vec(),
            star_name: r.star_name,
            ret_flags: r.ret_flags,
        })
        .map_err(to_napi)
}

/// Fixed star magnitude.
#[napi(js_name = "fixstarMag")]
pub fn fixstar_mag(star: String) -> napi::Result<f64> {
    celestial::fixstar_mag(&star).map_err(to_napi)
}

// ─── Houses ───────────────────────────────────────────────────────────────────

/// Calculate house cusps (UT). `hsys` is the ASCII code of the house letter.
#[napi]
pub fn houses(tjdut: f64, lat: f64, lon: f64, hsys: u32) -> napi::Result<HouseResult> {
    celestial::houses(tjdut, lat, lon, HouseSystem(hsys as u8))
        .map(|r| {
            // SE stores cusps[0..=12]; index 0 is unused (ASC is in ascmc[0]).
            // Return only cusps[1..=12] (12 real house cusps).
            let n = r
                .cusps
                .iter()
                .skip(1)
                .filter(|&&v| v != 0.0)
                .count()
                .max(12);
            HouseResult {
                cusps: r.cusps[1..=(n.min(r.cusps.len() - 1))].to_vec(),
                ascmc: r.ascmc[..8].to_vec(),
            }
        })
        .map_err(to_napi)
}

/// Extended house cusps with optional flags.
#[napi(js_name = "housesEx")]
pub fn houses_ex(
    tjdut: f64,
    lat: f64,
    lon: f64,
    hsys: u32,
    flags: Option<i32>,
) -> napi::Result<HouseResult> {
    celestial::houses_ex(
        tjdut,
        CalcFlags(flags.unwrap_or(0)),
        lat,
        lon,
        HouseSystem(hsys as u8),
    )
    .map(|r| HouseResult {
        cusps: r.cusps.to_vec(),
        ascmc: r.ascmc[..8].to_vec(),
    })
    .map_err(to_napi)
}

/// Houses with cusp speeds.
#[napi(js_name = "housesEx2")]
pub fn houses_ex2(
    tjdut: f64,
    lat: f64,
    lon: f64,
    hsys: u32,
    flags: Option<i32>,
) -> napi::Result<HouseResultEx2> {
    celestial::houses_ex2(
        tjdut,
        CalcFlags(flags.unwrap_or(0)),
        lat,
        lon,
        HouseSystem(hsys as u8),
    )
    .map(|r| HouseResultEx2 {
        cusps: r.cusps.to_vec(),
        ascmc: r.ascmc[..8].to_vec(),
        cusp_speeds: r.cusp_speeds.to_vec(),
        ascmc_speeds: r.ascmc_speeds[..8].to_vec(),
    })
    .map_err(to_napi)
}

/// Name of a house system.
#[napi(js_name = "houseName")]
pub fn house_name(hsys: u32) -> String {
    celestial::house_name(HouseSystem(hsys as u8))
}

// ─── Eclipses ─────────────────────────────────────────────────────────────────

/// Next solar eclipse globally (UT).
#[napi(js_name = "solEclipseWhenGlob")]
pub fn sol_eclipse_when_glob(
    tjd_start: f64,
    flags: i32,
    ecl_type: Option<i32>,
    backwards: Option<bool>,
) -> napi::Result<EclipseResult> {
    celestial::sol_eclipse_when_glob(
        tjd_start,
        CalcFlags(flags),
        ecl_type.unwrap_or(0),
        backwards.unwrap_or(false),
    )
    .map(|r| EclipseResult {
        ret_flags: r.ret_flags,
        tret: r.tret.to_vec(),
    })
    .map_err(to_napi)
}

/// Next solar eclipse from a location (UT).
#[napi(js_name = "solEclipseWhenLoc")]
pub fn sol_eclipse_when_loc(
    tjd_start: f64,
    geopos: Vec<f64>,
    flags: i32,
    backwards: Option<bool>,
) -> napi::Result<EclipseResultAttr> {
    let gp: [f64; 3] = geopos
        .try_into()
        .map_err(|_| napi::Error::from_reason("geopos must have 3 elements"))?;
    celestial::sol_eclipse_when_loc(tjd_start, CalcFlags(flags), gp, backwards.unwrap_or(false))
        .map(|r| EclipseResultAttr {
            ret_flags: r.ret_flags,
            tret: r.tret.to_vec(),
            attr: r.attr.to_vec(),
        })
        .map_err(to_napi)
}

/// Solar eclipse attributes at a location.
#[napi(js_name = "solEclipseHow")]
pub fn sol_eclipse_how(jd_ut: f64, geopos: Vec<f64>, flags: i32) -> napi::Result<EclipseHow> {
    let gp: [f64; 3] = geopos
        .try_into()
        .map_err(|_| napi::Error::from_reason("geopos must have 3 elements"))?;
    celestial::sol_eclipse_how(jd_ut, CalcFlags(flags), gp)
        .map(|r| EclipseHow {
            ret_flags: r.ret_flags,
            attr: r.attr.to_vec(),
        })
        .map_err(to_napi)
}

/// Where a solar eclipse is central.
#[napi(js_name = "solEclipseWhere")]
pub fn sol_eclipse_where(tjd: f64, flags: i32) -> napi::Result<EclipseWhere> {
    celestial::sol_eclipse_where(tjd, CalcFlags(flags))
        .map(|r| EclipseWhere {
            ret_flags: r.ret_flags,
            geopos: r.geopos.to_vec(),
            attr: r.attr.to_vec(),
        })
        .map_err(to_napi)
}

/// Next lunar eclipse globally (UT).
#[napi(js_name = "lunEclipseWhen")]
pub fn lun_eclipse_when(
    tjd_start: f64,
    flags: i32,
    ecl_type: Option<i32>,
    backwards: Option<bool>,
) -> napi::Result<EclipseResult> {
    celestial::lun_eclipse_when(
        tjd_start,
        CalcFlags(flags),
        ecl_type.unwrap_or(0),
        backwards.unwrap_or(false),
    )
    .map(|r| EclipseResult {
        ret_flags: r.ret_flags,
        tret: r.tret.to_vec(),
    })
    .map_err(to_napi)
}

/// Lunar eclipse attributes.
#[napi(js_name = "lunEclipseHow")]
pub fn lun_eclipse_how(
    jd_ut: f64,
    flags: i32,
    geopos: Option<Vec<f64>>,
) -> napi::Result<EclipseHow> {
    let gp = geopos
        .map(|v| -> napi::Result<[f64; 3]> {
            v.try_into()
                .map_err(|_| napi::Error::from_reason("geopos must have 3 elements"))
        })
        .transpose()?;
    celestial::lun_eclipse_how(jd_ut, CalcFlags(flags), gp)
        .map(|r| EclipseHow {
            ret_flags: r.ret_flags,
            attr: r.attr.to_vec(),
        })
        .map_err(to_napi)
}

// ─── Rise / transit ───────────────────────────────────────────────────────────

/// Rise, set, or transit calculation.
#[napi(js_name = "riseTrans")]
pub fn rise_trans(
    tjdut: f64,
    planet: i32,
    ephe_flags: i32,
    event_type: i32,
    geopos: Vec<f64>,
    pressure_mb: Option<f64>,
    temp_c: Option<f64>,
) -> napi::Result<RiseTrans> {
    let gp: [f64; 3] = geopos
        .try_into()
        .map_err(|_| napi::Error::from_reason("geopos must have 3 elements"))?;
    celestial::rise_trans(
        tjdut,
        Body::from_raw(planet),
        None,
        CalcFlags(ephe_flags),
        event_type,
        gp,
        pressure_mb.unwrap_or(0.0),
        temp_c.unwrap_or(0.0),
    )
    .map(|r| RiseTrans {
        ret_flags: r.ret_flags,
        tret: r.tret,
    })
    .map_err(to_napi)
}

// ─── Time ─────────────────────────────────────────────────────────────────────

/// Convert a calendar date to a Julian day number.
#[napi]
pub fn julday(year: i32, month: i32, day: i32, hour: Option<f64>, calendar: Option<i32>) -> f64 {
    celestial::julday(
        year,
        month,
        day,
        hour.unwrap_or(0.0),
        Calendar::from(calendar.unwrap_or(1)),
    )
}

/// Reverse a Julian day to a calendar date.
#[napi]
pub fn revjul(jd: f64, calendar: Option<i32>) -> CalDate {
    let d = celestial::revjul(jd, Calendar::from(calendar.unwrap_or(1)));
    CalDate {
        year: d.year,
        month: d.month,
        day: d.day,
        hour: d.hour,
    }
}

/// Day of week (0 = Monday, …, 6 = Sunday).
#[napi(js_name = "dayOfWeek")]
pub fn day_of_week(jd: f64) -> i32 {
    celestial::day_of_week(jd)
}

/// Delta-T (TT − UT).
#[napi]
pub fn deltat(tjd: f64) -> f64 {
    celestial::deltat(tjd)
}

/// Sidereal time.
#[napi]
pub fn sidtime(jd_ut: f64) -> f64 {
    celestial::sidtime(jd_ut)
}

/// Convert UTC to Julian day numbers.
#[napi(js_name = "utcToJd")]
pub fn utc_to_jd(date: UtcDate, calendar: Option<i32>) -> napi::Result<JdPair> {
    let d = celestial::UtcDate {
        year: date.year,
        month: date.month,
        day: date.day,
        hour: date.hour,
        minute: date.minute,
        second: date.second,
    };
    celestial::utc_to_jd(&d, Calendar::from(calendar.unwrap_or(1)))
        .map(|p| JdPair {
            et: p.et,
            ut1: p.ut1,
        })
        .map_err(to_napi)
}

// ─── Ayanamsa ─────────────────────────────────────────────────────────────────

/// Ayanamsa value at Julian Ephemeris Day (TT).
#[napi]
pub fn ayanamsa(jd_et: f64) -> f64 {
    celestial::ayanamsa(jd_et)
}
/// Ayanamsa value at Julian Day (UT).
#[napi]
pub fn ayanamsa_ut(jd_ut: f64) -> f64 {
    celestial::ayanamsa_ut(jd_ut)
}
/// Name of a sidereal mode.
#[napi]
pub fn ayanamsa_name(sid_mode: i32) -> &'static str {
    celestial::ayanamsa_name(sid_mode)
}

// ─── Math utilities ───────────────────────────────────────────────────────────

/// Normalise degrees to 0…360.
/// Degree midpoint (360° wrap aware).
#[napi(js_name = "degMidp")]
pub fn midpoint_deg(x1: f64, x0: f64) -> f64 {
    celestial::midpoint_deg(x1, x0)
}
/// Signed difference between two degree values.
/// Normalise centiseconds.
/// Split a degree value into components.
/// Returns `[deg, min, sec, secFraction, sign]`.
#[napi(js_name = "splitDeg")]
pub fn split_deg(deg: f64, round_flag: i32) -> Vec<f64> {
    let (d, m, s, frac, sgn) = celestial::split_deg(deg, round_flag);
    vec![d as f64, m as f64, s as f64, frac, sgn as f64]
}

/// Coordinate transform (ecliptic ↔ equatorial).
#[napi(js_name = "coordTransform")]
pub fn coord_transform(coord: Vec<f64>, eps: f64) -> napi::Result<Vec<f64>> {
    let c: [f64; 3] = coord
        .try_into()
        .map_err(|_| napi::Error::from_reason("coord must have 3 elements"))?;
    Ok(celestial::coord_transform(c, eps).to_vec())
}

/// Azimuth and altitude from ecliptic/equatorial coordinates.
#[napi]
pub fn azalt(
    tjdut: f64,
    calc_flag: i32,
    geopos: Vec<f64>,
    pressure_mb: f64,
    temp_c: f64,
    xin: Vec<f64>,
) -> napi::Result<AzAlt> {
    let gp: [f64; 3] = geopos
        .try_into()
        .map_err(|_| napi::Error::from_reason("geopos must have 3 elements"))?;
    let xi: [f64; 3] = xin
        .try_into()
        .map_err(|_| napi::Error::from_reason("xin must have 3 elements"))?;
    let r = celestial::azalt(tjdut, calc_flag, gp, pressure_mb, temp_c, xi);
    Ok(AzAlt {
        azimuth: r.azimuth,
        true_alt: r.true_alt,
        apparent_alt: r.apparent_alt,
    })
}

/// Atmospheric refraction.
#[napi]
pub fn refrac(altitude: f64, pressure_mb: f64, temp_c: f64, calc_flag: i32) -> f64 {
    celestial::refrac(altitude, pressure_mb, temp_c, calc_flag)
}

/// Extended atmospheric refraction.
#[napi(js_name = "refracExtended")]
pub fn refrac_extended(
    altitude: f64,
    geoalt: f64,
    pressure_mb: f64,
    temp_c: f64,
    lapse_rate: f64,
    calc_flag: i32,
) -> RefracExt {
    let (result, dret) =
        celestial::refrac_extended(altitude, geoalt, pressure_mb, temp_c, lapse_rate, calc_flag);
    RefracExt {
        result,
        dret: dret.to_vec(),
    }
}

/// Moon node crossing result.
#[napi(object)]
pub struct MoonCrossNodeResult {
    pub jd_cross: f64,
    pub xlon: f64,
}

// ─── Info ─────────────────────────────────────────────────────────────────────

/// Library version string.
/// Name of a planet / body.
/// Name of a house system.
#[napi]
pub fn planet_name(planet: i32) -> &'static str {
    celestial::planet_name(Body::from_raw(planet))
}

/// Full name of a house system from its one-letter code (e.g. `P` → `"Placidus"`).
#[napi(js_name = "houseNameStr")]
pub fn house_name_str(hsys: u32) -> String {
    celestial::house_name(HouseSystem(hsys as u8))
}

/// Duration between two JDs → [days, hours, minutes, seconds].
#[napi(js_name = "jdDuration")]
pub fn jd_duration(jd_start: f64, jd_end: f64) -> Vec<i32> {
    let d = celestial::jd_duration(jd_start, jd_end);
    d.to_vec()
}

/// Format Julian day as ISO string "YYYY-MM-DD HH:MM:SS UTC".
#[napi(js_name = "jdToIsoString")]
pub fn jd_to_iso_string(jd: f64, calendar: i32) -> String {
    celestial::jd_to_iso_string(jd, Calendar::from(calendar))
}

/// Moon node crossing (ET).
#[napi(js_name = "mooncrossNode")]
pub fn mooncross_node(jd_et: f64, flags: i32) -> napi::Result<MoonCrossNodeResult> {
    celestial::mooncross_node(jd_et, CalcFlags(flags))
        .map(|n| MoonCrossNodeResult {
            jd_cross: n.jd_cross,
            xlon: n.xlon,
        })
        .map_err(to_napi)
}

/// Moon node crossing (UT).
#[napi(js_name = "mooncrossNodeUt")]
pub fn mooncross_node_ut(jd_ut: f64, flags: i32) -> napi::Result<MoonCrossNodeResult> {
    celestial::mooncross_node_ut(jd_ut, CalcFlags(flags))
        .map(|n| MoonCrossNodeResult {
            jd_cross: n.jd_cross,
            xlon: n.xlon,
        })
        .map_err(to_napi)
}

// ─── Constants ────────────────────────────────────────────────────────────────

#[napi]
pub const GREG_CAL: i32 = celestial::GREG_CAL;
#[napi]
pub const JUL_CAL: i32 = celestial::JUL_CAL;

#[napi]
pub const SUN: i32 = celestial::SUN;
#[napi]
pub const MOON: i32 = celestial::MOON;
#[napi]
pub const MERCURY: i32 = celestial::MERCURY;
#[napi]
pub const VENUS: i32 = celestial::VENUS;
#[napi]
pub const MARS: i32 = celestial::MARS;
#[napi]
pub const JUPITER: i32 = celestial::JUPITER;
#[napi]
pub const SATURN: i32 = celestial::SATURN;
#[napi]
pub const URANUS: i32 = celestial::URANUS;
#[napi]
pub const NEPTUNE: i32 = celestial::NEPTUNE;
#[napi]
pub const PLUTO: i32 = celestial::PLUTO;
#[napi]
pub const MEAN_NODE: i32 = celestial::MEAN_NODE;
#[napi]
pub const TRUE_NODE: i32 = celestial::TRUE_NODE;
#[napi]
pub const CHIRON: i32 = celestial::CHIRON;
#[napi]
pub const FLG_BUILTIN: i64 = celestial::FLG_BUILTIN as i64;
#[napi]
pub const FLG_SPEED: i64 = celestial::FLG_SPEED as i64;

#[napi]
pub const EARTH: i32 = Body::EARTH.as_raw();

#[napi]
pub const FLG_JPL: i64 = celestial::FLG_JPL as i64;
#[napi]
pub const FLG_MOSHIER: i64 = celestial::FLG_MOSHIER as i64;
#[napi]
#[napi]
pub const FLG_EQUATORIAL: i64 = celestial::FLG_EQUATORIAL as i64;
#[napi]
pub const FLG_TOPOCTR: i64 = celestial::FLG_TOPOCTR as i64;
#[napi]
pub const FLG_SIDEREAL: i64 = celestial::FLG_SIDEREAL as i64;
#[napi]
pub const FLG_HELCTR: i64 = celestial::FLG_HELCTR as i64;
#[napi]
pub const FLG_XYZ: i64 = 4096i32 as i64;
#[napi]
pub const FLG_RADIANS: i64 = celestial::FLG_RADIANS as i64;
#[napi]
pub const FLG_NONUT: i64 = celestial::FLG_NONUT as i64;

#[napi]
pub const SIDM_FAGAN_BRADLEY: i32 = celestial::SIDM_FAGAN_BRADLEY;
#[napi]
pub const SIDM_LAHIRI: i32 = celestial::SIDM_LAHIRI;
#[napi]
pub const SIDM_RAMAN: i32 = celestial::SIDM_RAMAN;
#[napi]
pub const SIDM_USER: i32 = 255i32;

#[napi]
pub const ECL_TOTAL: i32 = celestial::ECL_TOTAL;
#[napi]
pub const ECL_ANNULAR: i32 = celestial::ECL_ANNULAR;
#[napi]
pub const ECL_PARTIAL: i32 = celestial::ECL_PARTIAL;
#[napi]
pub const ECL_PENUMBRAL: i32 = celestial::ECL_PENUMBRAL;

#[napi]
pub const CALC_RISE: i32 = celestial::CALC_RISE;
#[napi]
pub const CALC_SET: i32 = celestial::CALC_SET;

#[napi]
pub const TRUE_TO_APP: i32 = celestial::TRUE_TO_APP;
#[napi]
pub const APP_TO_TRUE: i32 = celestial::APP_TO_TRUE;

#[napi]
pub const SPLIT_DEG_ROUND_SEC: i32 = celestial::SPLIT_DEG_ROUND_SEC;
#[napi]
pub const SPLIT_DEG_ZODIACAL: i32 = celestial::SPLIT_DEG_ZODIACAL;

// ─── Functions present in Python binding, added here for API symmetry ────────

/// Set a user-defined delta-T value (seconds). Pass NaN to reset to automatic.
#[napi(js_name = "setDeltaTUserdef")]
pub fn set_delta_t_userdef(dt: f64) {
    celestial::set_delta_t_userdef(dt);
}

/// Revjul with hours/minutes/seconds breakdown. Returns [year,month,day,hour,min,sec].
#[napi(js_name = "revjulHms")]
pub fn revjul_hms(jd: f64, calendar: i32) -> Vec<i32> {
    celestial::revjul_hms(jd, Calendar::from(calendar)).to_vec()
}

/// Parse an ISO datetime string into [year,month,day,hour,min,sec].
#[napi(js_name = "parseDatetime")]
pub fn parse_datetime(s: String) -> Option<Vec<i32>> {
    celestial::parse_datetime(&s).map(|a| a.to_vec())
}

/// Split ecliptic longitude into [degrees_in_sign, sign_number, minutes, seconds].
#[napi(js_name = "degsplit")]
pub fn degsplit(pos: f64) -> Vec<i32> {
    celestial::degsplit(pos).to_vec()
}

/// Antiscion of a planet. Returns [lon, lat, dist, speed_lon, speed_lat, speed_dist].
#[napi(js_name = "antiscion")]
pub fn antiscion(pos: Vec<f64>, axis: f64) -> napi::Result<Vec<f64>> {
    if pos.len() < 6 {
        return Err(napi::Error::from_reason("pos must have 6 elements"));
    }
    let arr: [f64; 6] = pos[..6].try_into().unwrap();
    let r = celestial::antiscion(arr, axis);
    Ok(r.antiscion.to_vec())
}

/// Match aspect (checking for exact/applying/separating). Returns (match_flag, app_flag, t_exact).
#[napi(js_name = "matchAspect")]
pub fn match_aspect(
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    orb: f64,
) -> Vec<f64> {
    let r = celestial::match_aspect(pos0, speed0, pos1, speed1, aspect, orb);
    vec![if r.matched { 1.0 } else { 0.0 }, r.diff, r.speed, r.factor]
}

/// match_aspect with separate applying/separating orbs.
#[napi(js_name = "matchAspect2")]
pub fn match_aspect2(
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    orb: f64,
) -> Vec<f64> {
    let r = celestial::match_aspect2(pos0, speed0, pos1, speed1, aspect, orb);
    vec![if r.matched { 1.0 } else { 0.0 }, r.diff, r.speed, r.factor]
}

/// match_aspect with separate app/sep orbs (version 3).
#[napi(js_name = "matchAspect3")]
pub fn match_aspect3(
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    app_orb: f64,
    sep_orb: f64,
) -> Vec<f64> {
    let r = celestial::match_aspect3(
        pos0,
        speed0,
        pos1,
        speed1,
        aspect,
        app_orb,
        sep_orb,
        app_orb.max(sep_orb),
    );
    vec![if r.matched { 1.0 } else { 0.0 }, r.diff, r.speed, r.factor]
}

/// match_aspect with separate app/sep orbs (version 4 — fastest).
#[napi(js_name = "matchAspect4")]
pub fn match_aspect4(
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    app_orb: f64,
    sep_orb: f64,
) -> Vec<f64> {
    let r = celestial::match_aspect4(
        pos0,
        speed0,
        pos1,
        speed1,
        aspect,
        app_orb,
        sep_orb,
        app_orb.max(sep_orb),
    );
    vec![if r.matched { 1.0 } else { 0.0 }, r.diff, r.speed, r.factor]
}

/// Find next retrograde station. Returns [jd_station, lon_station] or null.
#[napi(js_name = "nextRetro")]
pub fn next_retro(
    planet: i32,
    jd_start: f64,
    backward: bool,
    stop_days: f64,
    flags: i32,
) -> Option<Vec<f64>> {
    celestial::next_retro(
        Body::from_raw(planet),
        jd_start,
        backward,
        stop_days,
        CalcFlags(flags),
    )
    .map(|r| [vec![r.jd], r.pos.to_vec()].concat())
}

/// Find next aspect from a planet to a fixed point. Returns [jd, lon] or null.
#[napi(js_name = "nextAspect")]
pub fn next_aspect(
    planet: i32,
    aspect: f64,
    fixed_pt: f64,
    jd_start: f64,
    backward: bool,
    stop_days: f64,
    flags: i32,
) -> Option<Vec<f64>> {
    celestial::next_aspect(
        Body::from_raw(planet),
        aspect,
        fixed_pt,
        jd_start,
        backward,
        stop_days,
        CalcFlags(flags),
    )
    .map(|r| [vec![r.jd], r.pos1.to_vec(), r.pos2.to_vec()].concat())
}

/// Find next aspect between two planets. Returns [jd, p1_lon, p2_lon] or null.
#[napi(js_name = "nextAspectWith")]
pub fn next_aspect_with(
    planet: i32,
    aspect: f64,
    other: i32,
    jd_start: f64,
    backward: bool,
    stop_days: f64,
    flags: i32,
) -> Option<Vec<f64>> {
    celestial::next_aspect_with(
        Body::from_raw(planet),
        aspect,
        Body::from_raw(other),
        jd_start,
        backward,
        stop_days,
        CalcFlags(flags),
    )
    .map(|r| [vec![r.jd], r.pos1.to_vec(), r.pos2.to_vec()].concat())
}

/// House position of a body. Returns house number (1-12).
#[napi(js_name = "housePos")]
pub fn house_pos(
    armc: f64,
    lat: f64,
    eps: f64,
    hsys: u32,
    lon: f64,
    lat_body: f64,
) -> napi::Result<f64> {
    let r = celestial::house_pos(armc, lat, eps, HouseSystem(hsys as u8), [lon, lat_body])
        .map_err(to_napi)?;
    Ok(r)
}

/// Find next lunar eclipse visible from a location.
#[napi(js_name = "lunEclipseWhenLoc")]
pub fn lun_eclipse_when_loc(
    tjd_start: f64,
    geopos: Vec<f64>,
    flags: i32,
    backwards: bool,
) -> napi::Result<EclipseResultAttr> {
    let gp: [f64; 3] = geopos
        .get(..3)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| napi::Error::from_reason("geopos needs 3 elements"))?;
    celestial::lun_eclipse_when_loc(tjd_start, CalcFlags(flags), gp, backwards)
        .map(|r| EclipseResultAttr {
            ret_flags: r.ret_flags,
            tret: r.tret.to_vec(),
            attr: r.attr.to_vec(),
        })
        .map_err(to_napi)
}

/// Fixed star position (ET) — alias for fixstar.
#[napi(js_name = "fixstar2")]
pub fn fixstar2(star: String, tjdet: f64, flags: i32) -> napi::Result<StarPos> {
    celestial::fixstar2(&star, tjdet, CalcFlags(flags))
        .map(|r| StarPos {
            xx: r.xx.to_vec(),
            star_name: r.star_name,
            ret_flags: r.ret_flags,
        })
        .map_err(to_napi)
}

/// Fixed star position (UT) — alias for fixstar_ut.
#[napi(js_name = "fixstar2Ut")]
pub fn fixstar2_ut(star: String, tjdut: f64, flags: i32) -> napi::Result<StarPos> {
    celestial::fixstar2_ut(&star, tjdut, CalcFlags(flags))
        .map(|r| StarPos {
            xx: r.xx.to_vec(),
            star_name: r.star_name,
            ret_flags: r.ret_flags,
        })
        .map_err(to_napi)
}

/// Fixed star magnitude — alias for fixstar_mag.
#[napi(js_name = "fixstar2Mag")]
pub fn fixstar2_mag(star: String) -> napi::Result<f64> {
    celestial::fixstar2_mag(&star).map_err(to_napi)
}

/// Convert (az, alt) back to ecliptic (flag=0) or equatorial (flag=1) coords.
/// Returns [lon_or_ra, lat_or_dec, 1.0].
#[napi(js_name = "azaltRev")]
pub fn azalt_rev(
    tjdut: f64,
    calc_flag: i32,
    geopos: Vec<f64>,
    az: f64,
    alt: f64,
) -> napi::Result<Vec<f64>> {
    let gp: [f64; 3] = geopos
        .get(..3)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| napi::Error::from_reason("geopos needs 3 elements"))?;
    Ok(celestial::azalt_rev(tjdut, calc_flag, gp, [az, alt]).to_vec())
}

// ─── Functions present in Python binding but previously missing from JS ────────

/// English name of a zodiac sign (0=Aries … 11=Pisces).
#[napi(js_name = "signName")]
pub fn sign_name(sign: i32) -> Option<String> {
    celestial::sign_name(sign).map(|s| s.to_string())
}

/// Current Julian Day number (UT) from the system clock.
#[napi(js_name = "jdnow")]
pub fn jdnow() -> f64 {
    celestial::jdnow()
}

/// Parse a geographic coordinate string to decimal degrees.
/// Accepts formats like `48°52'N`, `48:52:N`, `48.87N`, `2.35E`.
#[napi(js_name = "parseCoord")]
pub fn parse_coord(s: String) -> Option<f64> {
    celestial::parse_coord(&s)
}

/// Format a geographic coordinate as `"DD:N|S:MM:SS"` or `"DDD:E|W:MM:SS"`.
#[napi(js_name = "formatCoord")]
pub fn format_coord(coord: f64, is_latitude: bool) -> Option<String> {
    celestial::format_coord(coord, is_latitude)
}

/// Vedic rasi (sign number 0–11) from ecliptic longitude.
#[napi(js_name = "longToRasi")]
pub fn long_to_rasi(lon: f64) -> i32 {
    celestial::long_to_rasi(lon)
}

/// Navamsa division number (0–35) from ecliptic longitude.
#[napi(js_name = "longToNavamsa")]
pub fn long_to_navamsa(lon: f64) -> i32 {
    celestial::long_to_navamsa(lon)
}

/// Nakshatra (0–26) and pada (0–3) from ecliptic longitude.
#[napi(js_name = "longToNakshatra")]
pub fn long_to_nakshatra(lon: f64) -> Vec<i32> {
    let (nak, pada) = celestial::long_to_nakshatra(lon);
    vec![nak, pada]
}

/// English name of a nakshatra (0–26).
#[napi(js_name = "nakshatraName")]
pub fn nakshatra_name(nak: i32) -> Option<String> {
    celestial::nakshatra_name(nak).map(|s| s.to_string())
}

/// Raman house cusps from Ascendant and MC.
/// `sandhi=false` gives bhavamadhya (midpoints), `true` gives arambhasandhi.
#[napi(js_name = "ramanHouses")]
pub fn raman_houses(asc: f64, mc: f64, sandhi: bool) -> Vec<f64> {
    celestial::raman_houses(asc, mc, sandhi).to_vec()
}

/// Residential strength of a graha given 12 bhavamadhya longitudes.
/// Returns a value in [0, 1].
#[napi(js_name = "residentialStrength")]
pub fn residential_strength(graha: f64, bm: Vec<f64>) -> napi::Result<f64> {
    let arr: [f64; 12] = bm
        .get(..12)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| napi::Error::from_reason("bm needs 12 elements"))?;
    celestial::residential_strength(graha, &arr)
        .ok_or_else(|| napi::Error::from_reason("graha out of range"))
}

/// Ochchabala (exaltation strength) for a graha in shashtiamsa.
#[napi(js_name = "ochchabala")]
pub fn ochchabala(graha: i32, sputha: f64) -> napi::Result<f64> {
    celestial::ochchabala(graha, sputha)
        .ok_or_else(|| napi::Error::from_reason("graha out of range"))
}

/// Naisargika (permanent) planetary relation.
/// Returns 1 (friend), 0 (neutral), or -1 (enemy).
#[napi(js_name = "naisargakaRelation")]
pub fn naisargika_relation(gr1: i32, gr2: i32) -> napi::Result<i32> {
    celestial::naisargika_relation(gr1, gr2)
        .ok_or_else(|| napi::Error::from_reason("invalid graha"))
}

/// Positions of Pushya, Revati, Hasta, and Chitra at a given Julian day.
#[napi(js_name = "saturnFourStars")]
pub fn saturn_4_stars(jd: f64, flags: i32) -> napi::Result<Vec<f64>> {
    celestial::saturn_4_stars(jd, CalcFlags(flags))
        .map(|a| a.to_vec())
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Library version string (Cargo package version).
#[napi(js_name = "version")]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Normalise degrees to [0, 360).
#[napi(js_name = "normDeg")]
pub fn norm_deg(x: f64) -> f64 {
    celestial::norm_deg(x)
}

/// Signed difference of degrees, result in (−180, +180].
#[napi(js_name = "diffDegSigned")]
pub fn diff_deg_signed(p1: f64, p2: f64) -> f64 {
    celestial::diff_deg_signed(p1, p2)
}

/// Normalise centiseconds to [0, 360×360000).
#[napi(js_name = "normCs")]
pub fn norm_cs(p: i32) -> i32 {
    celestial::norm_cs(p) as i32
}

// ─── Chart functions (added) ──────────────────────────────────────────────────

#[napi(object)]
pub struct IngressResult {
    pub jd: f64,
    pub sign: u32,
}

/// Next time a body ingresses into any zodiac sign after `jd_start`.
/// Returns `(jd, sign_number)` where sign is 0–11.
#[napi(js_name = "signIngressUt")]
pub fn sign_ingress_ut(
    planet: i32,
    jd: f64,
    flags: i32,
    backward: bool,
) -> napi::Result<IngressResult> {
    celestial::sign_ingress_ut(Body::from_raw(planet), jd, CalcFlags(flags), backward)
        .map(|(jd, sign)| IngressResult {
            jd,
            sign: sign as u32,
        })
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[napi(object)]
pub struct Stations {
    pub retrograde: f64,
    pub direct: f64,
}

/// Find the next retrograde and direct stations for a body after `jd_start`.
#[napi(js_name = "retrogradeStationUt")]
pub fn retrograde_station_ut(planet: i32, jd: f64, flags: i32) -> napi::Result<Stations> {
    celestial::retrograde_station_ut(Body::from_raw(planet), jd, CalcFlags(flags))
        .map(|s| Stations {
            retrograde: s.retrograde,
            direct: s.direct,
        })
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Arabic Part formula: `(asc + body2 - body1) mod 360`.
/// For Lot of Fortune: `arabicPart(asc, moonLon, sunLon)`.
#[napi(js_name = "arabicPart")]
pub fn arabic_part(asc: f64, body2: f64, body1: f64) -> f64 {
    celestial::arabic_part(asc, body2, body1)
}

/// Next time a transiting body reaches `target_lon` degrees after `jd`.
#[napi(js_name = "transitToDegree")]
pub fn transit_to_degree(
    planet: i32,
    target_lon: f64,
    jd: f64,
    flags: i32,
    backward: bool,
) -> napi::Result<f64> {
    celestial::transit_to_degree(
        Body::from_raw(planet),
        target_lon,
        jd,
        CalcFlags(flags),
        backward,
    )
    .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Next time a body transits the natal MC angle.
#[napi(js_name = "mcTransitUt")]
pub fn mc_transit_ut(
    planet: i32,
    jd_natal: f64,
    jd_start: f64,
    lat: f64,
    lon: f64,
    hsys: u32,
    flags: i32,
    backward: bool,
) -> napi::Result<f64> {
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
    .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Julian day of the solar return in `returnYear` closest to `jdNatal`.
#[napi(js_name = "solarReturnJd")]
pub fn solar_return_jd(jd_natal: f64, return_year: i32, flags: i32) -> napi::Result<f64> {
    celestial::solar_return_jd(jd_natal, return_year, CalcFlags(flags))
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[napi(js_name = "lunarReturnJd")]
pub fn lunar_return_jd(jd_natal: f64, jd_start: f64, flags: i32) -> napi::Result<f64> {
    celestial::lunar_return_jd(jd_natal, jd_start, CalcFlags(flags))
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[napi(js_name = "midpoint")]
pub fn midpoint(lon1: f64, lon2: f64) -> f64 {
    celestial::midpoint(lon1, lon2)
}

#[napi(js_name = "signRuler")]
pub fn sign_ruler(sign: u32) -> i32 {
    celestial::sign_ruler(sign as u8).as_raw()
}

#[napi(js_name = "signRulerModern")]
pub fn sign_ruler_modern(sign: u32) -> i32 {
    celestial::sign_ruler_modern(sign as u8).as_raw()
}

#[napi(js_name = "zodiacSignName")]
pub fn zodiac_sign_name(sign: u32) -> String {
    celestial::zodiac_sign_name(sign as u8).to_string()
}

#[napi(js_name = "lonToSign")]
pub fn lon_to_sign(lon: f64) -> Vec<f64> {
    let (sign, deg) = celestial::lon_to_sign(lon);
    vec![sign as f64, deg]
}

#[napi(js_name = "localApparentSolarTime")]
pub fn local_apparent_solar_time(jd_ut: f64, geolon_deg: f64) -> napi::Result<f64> {
    celestial::local_apparent_solar_time(jd_ut, geolon_deg)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[napi(js_name = "annualProfection")]
pub fn annual_profection(cusps: Vec<f64>, age: u32) -> napi::Result<Vec<f64>> {
    let arr: [f64; 13] = cusps
        .get(..13)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| napi::Error::from_reason("cusps needs 13 elements"))?;
    let (house, degree) = celestial::annual_profection(&arr, age);
    Ok(vec![house as f64, degree])
}

#[napi(js_name = "vimshottariDasha")]
pub fn vimshottari_dasha(jd_birth: f64, moon_lon_sidereal: f64, years_ahead: f64) -> Vec<Vec<f64>> {
    celestial::vimshottari_dasha(jd_birth, moon_lon_sidereal, years_ahead)
        .iter()
        .map(|d| vec![d.body.as_raw() as f64, d.start, d.end, d.years])
        .collect()
}

// ─── Sefirat HaOmer ───────────────────────────────────────────────────────────

#[napi(object)]
pub struct OmerDay {
    /// Day number in the Omer (1–49).
    pub day: u32,
    /// Week number (1–7).
    pub week: u32,
    /// Day within the week (1–7).
    pub day_of_week: u32,
    /// Sefirah of the week (e.g. "Chesed").
    pub week_sefirah: String,
    /// Sefirah of the day (e.g. "Tiferet").
    pub day_sefirah: String,
    /// Full Hebrew declaration text.
    pub hebrew_text: String,
    /// True if this is Lag Ba'Omer (day 33).
    pub is_lag_baomer: bool,
    /// Julian day of the start of this Omer day.
    pub jd: f64,
}

/// Return the Omer day for a given Julian day, or null if outside the Omer period.
#[napi(js_name = "omerFromJd")]
pub fn omer_from_jd(jd: f64) -> Option<OmerDay> {
    celestial::omer_from_jd(jd).map(|d| OmerDay {
        day: d.day as u32,
        week: d.week as u32,
        day_of_week: d.day_of_week as u32,
        week_sefirah: d.week_sefirah.to_string(),
        day_sefirah: d.day_sefirah.to_string(),
        hebrew_text: d.hebrew_text.to_string(),
        is_lag_baomer: d.is_lag_baomer,
        jd: d.jd,
    })
}

/// Julian day of a specific Omer day (1–49) in the given Hebrew year.
#[napi(js_name = "omerDayJd")]
pub fn omer_day_jd(hebrew_year: i32, day: u32) -> Option<f64> {
    celestial::omer_day_jd(hebrew_year, day as u8)
}

/// Julian day of the first day of the Omer for the given Hebrew year.
#[napi(js_name = "omerStartJd")]
pub fn omer_start_jd(hebrew_year: i32) -> f64 {
    celestial::omer_start_jd(hebrew_year)
}

/// All 49 Omer days for the given Hebrew year.
#[napi(js_name = "omerDays")]
pub fn omer_days(hebrew_year: i32) -> Vec<OmerDay> {
    celestial::omer_days(hebrew_year)
        .into_iter()
        .map(|d| OmerDay {
            day: d.day as u32,
            week: d.week as u32,
            day_of_week: d.day_of_week as u32,
            week_sefirah: d.week_sefirah.to_string(),
            day_sefirah: d.day_sefirah.to_string(),
            hebrew_text: d.hebrew_text.to_string(),
            is_lag_baomer: d.is_lag_baomer,
            jd: d.jd,
        })
        .collect()
}

/// Omer period (start and end Julian days, Hebrew year) containing the given JD.
#[napi(object)]
pub struct OmerPeriod {
    pub start_jd: f64,
    pub end_jd: f64,
    pub hebrew_year: i32,
}

#[napi(js_name = "omerPeriod")]
pub fn omer_period(jd: f64) -> OmerPeriod {
    let p = celestial::omer_period(jd);
    OmerPeriod {
        start_jd: p.start_jd,
        end_jd: p.end_jd,
        hebrew_year: p.hebrew_year,
    }
}

/// Full declaration string for the given Omer day (1–49).
#[napi(js_name = "omerDeclaration")]
pub fn omer_declaration(day: u32) -> String {
    celestial::omer_declaration(day as u8)
}

// ─── Jewish holidays ──────────────────────────────────────────────────────────

#[napi(object)]
pub struct JewishHoliday {
    pub name: String,
    pub hebrew_name: String,
    pub hebrew_month: u32,
    pub hebrew_day: u32,
    pub jd: f64,
    pub jd_end: f64,
    pub days: u32,
    pub category: String,
}

#[napi(js_name = "jewishHolidays")]
pub fn jewish_holidays(hebrew_year: i32) -> Vec<JewishHoliday> {
    celestial::jewish_holidays(hebrew_year)
        .into_iter()
        .map(|h| JewishHoliday {
            name: h.name.to_string(),
            hebrew_name: h.hebrew_name.to_string(),
            hebrew_month: h.hebrew_month as u32,
            hebrew_day: h.hebrew_day as u32,
            jd: h.jd,
            jd_end: h.jd_end,
            days: h.days as u32,
            category: format!("{:?}", h.category),
        })
        .collect()
}

#[napi(js_name = "jewishHolidayJd")]
pub fn jewish_holiday_jd(hebrew_year: i32, name: String) -> Option<f64> {
    celestial::jewish_holiday_jd(hebrew_year, &name)
}

#[napi(js_name = "hebrewYearFromJd")]
pub fn hebrew_year_from_jd(jd: f64) -> i32 {
    celestial::hebrew_year_from_jd(jd)
}

#[napi(object)]
pub struct HebrewDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[napi(js_name = "jdToHebrewDate")]
pub fn jd_to_hebrew_date(jd: f64) -> HebrewDate {
    let (y, m, d) = celestial::jd_to_hebrew_date(jd);
    HebrewDate {
        year: y,
        month: m as u32,
        day: d as u32,
    }
}

// ─── Easter & Christian calendar ──────────────────────────────────────────────

#[napi(object)]
pub struct CalendarDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[napi(js_name = "easterGregorian")]
pub fn easter_gregorian(year: i32) -> CalendarDate {
    let (y, m, d) = celestial::easter_gregorian(year);
    CalendarDate {
        year: y,
        month: m as u32,
        day: d as u32,
    }
}

#[napi(js_name = "easterOrthodox")]
pub fn easter_orthodox(year: i32) -> CalendarDate {
    let (y, m, d) = celestial::easter_orthodox(year);
    CalendarDate {
        year: y,
        month: m as u32,
        day: d as u32,
    }
}

#[napi(js_name = "easterJd")]
pub fn easter_jd(year: i32) -> f64 {
    celestial::easter_jd(year)
}

#[napi(js_name = "easterOrthodoxJd")]
pub fn easter_orthodox_jd(year: i32) -> f64 {
    celestial::easter_orthodox_jd(year)
}

#[napi(object)]
pub struct ChristianFeast {
    pub name: String,
    pub easter_offset: i32,
    pub jd: f64,
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[napi(js_name = "christianFeasts")]
pub fn christian_feasts(year: i32) -> Vec<ChristianFeast> {
    celestial::christian_feasts(year)
        .into_iter()
        .map(|f| ChristianFeast {
            name: f.name.to_string(),
            easter_offset: f.easter_offset,
            jd: f.jd,
            year: f.year,
            month: f.month as u32,
            day: f.day as u32,
        })
        .collect()
}

#[napi(js_name = "christianFixedFeasts")]
pub fn christian_fixed_feasts(year: i32) -> Vec<ChristianFeast> {
    celestial::christian_fixed_feasts(year)
        .into_iter()
        .map(|f| ChristianFeast {
            name: f.name.to_string(),
            easter_offset: 0,
            jd: f.jd,
            year: f.year,
            month: f.month as u32,
            day: f.day as u32,
        })
        .collect()
}

// ─── Islamic calendar ─────────────────────────────────────────────────────────

#[napi(object)]
pub struct HijriDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

#[napi(js_name = "hijriFromJd")]
pub fn hijri_from_jd(jd: f64) -> HijriDate {
    let (y, m, d) = celestial::hijri_from_jd(jd);
    HijriDate {
        year: y,
        month: m as u32,
        day: d as u32,
    }
}

#[napi(js_name = "hijriToJd")]
pub fn hijri_to_jd(year: i32, month: u32, day: u32) -> f64 {
    celestial::hijri_to_jd(year, month as u8, day as u8)
}

#[napi(js_name = "hijriMonthName")]
pub fn hijri_month_name(month: u32) -> String {
    celestial::hijri_month_name(month as u8).to_string()
}

#[napi(object)]
pub struct IslamicObservance {
    pub name: String,
    pub arabic_name: String,
    pub hijri_month: u32,
    pub hijri_day: u32,
    pub jd: f64,
    pub days: u32,
}

#[napi(js_name = "islamicObservances")]
pub fn islamic_observances(hijri_year: i32) -> Vec<IslamicObservance> {
    celestial::islamic_observances(hijri_year)
        .into_iter()
        .map(|o| IslamicObservance {
            name: o.name.to_string(),
            arabic_name: o.arabic_name.to_string(),
            hijri_month: o.hijri_month as u32,
            hijri_day: o.hijri_day as u32,
            jd: o.jd,
            days: o.days as u32,
        })
        .collect()
}

#[napi(object)]
pub struct HijriYears {
    pub year1: i32,
    pub year2: i32,
}

#[napi(js_name = "gregorianToHijriYears")]
pub fn gregorian_to_hijri_years(gregorian_year: i32) -> HijriYears {
    let (y1, y2) = celestial::gregorian_to_hijri_years(gregorian_year);
    HijriYears {
        year1: y1,
        year2: y2,
    }
}

// ─── Hindu Panchānga ──────────────────────────────────────────────────────────

#[napi(object)]
pub struct Panchanga {
    pub tithi: u32,
    pub tithi_name: String,
    pub paksha: String,
    pub vara: u32,
    pub vara_name: String,
    pub nakshatra: u32,
    pub nakshatra_name: String,
    pub nakshatra_pada: u32,
    pub yoga: u32,
    pub yoga_name: String,
    pub karana: u32,
    pub karana_name: String,
    pub sun_lon: f64,
    pub moon_lon: f64,
    pub elongation: f64,
}

#[napi(js_name = "panchanga")]
pub fn panchanga(jd: f64) -> Panchanga {
    let p = celestial::panchanga(jd);
    Panchanga {
        tithi: p.tithi as u32,
        tithi_name: p.tithi_name.to_string(),
        paksha: format!("{:?}", p.paksha),
        vara: p.vara as u32,
        vara_name: p.vara_name.to_string(),
        nakshatra: p.nakshatra as u32,
        nakshatra_name: p.nakshatra_name.to_string(),
        nakshatra_pada: p.nakshatra_pada as u32,
        yoga: p.yoga as u32,
        yoga_name: p.yoga_name.to_string(),
        karana: p.karana as u32,
        karana_name: p.karana_name.to_string(),
        sun_lon: p.sun_lon,
        moon_lon: p.moon_lon,
        elongation: p.elongation,
    }
}

#[napi(object)]
pub struct HinduFestival {
    pub name: String,
    pub description: String,
    pub jd: f64,
}

#[napi(js_name = "hinduFestivals")]
pub fn hindu_festivals(gregorian_year: i32) -> Vec<HinduFestival> {
    celestial::hindu_festivals(gregorian_year)
        .into_iter()
        .map(|f| HinduFestival {
            name: f.name.to_string(),
            description: f.description.to_string(),
            jd: f.jd,
        })
        .collect()
}

// ─── Buddhist observances ─────────────────────────────────────────────────────

#[napi(js_name = "vesakJd")]
pub fn vesak_jd(year: i32) -> f64 {
    celestial::vesak_jd(year)
}

#[napi(object)]
pub struct Uposatha {
    pub phase: String,
    pub jd: f64,
    pub elongation: f64,
}

#[napi(js_name = "uposathaDays")]
pub fn uposatha_days(year: i32) -> Vec<Uposatha> {
    celestial::uposatha_days(year)
        .into_iter()
        .map(|u| Uposatha {
            phase: format!("{:?}", u.phase),
            jd: u.jd,
            elongation: u.elongation,
        })
        .collect()
}

// ─── Nowruz & Bahá'í calendar ─────────────────────────────────────────────────

#[napi(js_name = "nowruzJd")]
pub fn nowruz_jd(year: i32) -> f64 {
    celestial::nowruz_jd(year)
}

#[napi(js_name = "gregorianToSolarHijri")]
pub fn gregorian_to_solar_hijri(year: i32) -> i32 {
    celestial::gregorian_to_solar_hijri(year)
}

#[napi(js_name = "nawRuzJd")]
pub fn naw_ruz_jd(bahai_year: i32) -> f64 {
    celestial::naw_ruz_jd(bahai_year)
}

#[napi(object)]
pub struct BahaiDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub month_name: String,
}

#[napi(js_name = "jdToBahai")]
pub fn jd_to_bahai(jd: f64) -> BahaiDate {
    let b = celestial::jd_to_bahai(jd);
    BahaiDate {
        year: b.year,
        month: b.month as u32,
        day: b.day as u32,
        month_name: b.month_name.to_string(),
    }
}

#[napi(object)]
pub struct BahaiHolyDay {
    pub name: String,
    pub description: String,
    pub bahai_month: u32,
    pub bahai_day: u32,
    pub jd: f64,
}

#[napi(js_name = "bahaiHolyDays")]
pub fn bahai_holy_days(bahai_year: i32) -> Vec<BahaiHolyDay> {
    celestial::bahai_holy_days(bahai_year)
        .into_iter()
        .map(|h| BahaiHolyDay {
            name: h.name.to_string(),
            description: h.description.to_string(),
            bahai_month: h.bahai_month as u32,
            bahai_day: h.bahai_day as u32,
            jd: h.jd,
        })
        .collect()
}

// ─── Moon phases ──────────────────────────────────────────────────────────────

/// Current Moon phase name at the given JD.
#[napi(js_name = "moonPhase")]
pub fn moon_phase(jd: f64) -> napi::Result<String> {
    celestial::moon_phase(jd)
        .map(|p| p.name().to_string())
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Fraction of the Moon's disk illuminated (0.0–1.0).
#[napi(js_name = "moonIllumination")]
pub fn moon_illumination(jd: f64) -> napi::Result<f64> {
    celestial::moon_illumination(jd).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Moon–Sun elongation in degrees (0°–360°).
#[napi(js_name = "moonElongation")]
pub fn moon_elongation(jd: f64) -> napi::Result<f64> {
    celestial::moon_elongation(jd).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Moon phase angle in degrees (0° = new, 180° = full).
#[napi(js_name = "moonPhaseAngle")]
pub fn moon_phase_angle(jd: f64) -> napi::Result<f64> {
    celestial::moon_phase_angle(jd).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// JD of the next new moon at or after jdFrom.
#[napi(js_name = "nextNewMoon")]
pub fn next_new_moon(jd_from: f64) -> napi::Result<f64> {
    celestial::next_new_moon(jd_from).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// JD of the next first-quarter moon at or after jdFrom.
#[napi(js_name = "nextFirstQuarter")]
pub fn next_first_quarter(jd_from: f64) -> napi::Result<f64> {
    celestial::next_first_quarter(jd_from).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// JD of the next full moon at or after jdFrom.
#[napi(js_name = "nextFullMoonPhase")]
pub fn next_full_moon_phase(jd_from: f64) -> napi::Result<f64> {
    celestial::next_full_moon_phase(jd_from).map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// JD of the next last-quarter moon at or after jdFrom.
#[napi(js_name = "nextLastQuarter")]
pub fn next_last_quarter(jd_from: f64) -> napi::Result<f64> {
    celestial::next_last_quarter(jd_from).map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[napi(object)]
pub struct PhaseEvent {
    pub phase: String,
    pub jd: f64,
    pub elongation: f64,
}

/// All principal phase events (New, FQ, Full, LQ) in a calendar month.
#[napi(js_name = "moonPhasesForMonth")]
pub fn moon_phases_for_month(year: i32, month: u32) -> napi::Result<Vec<PhaseEvent>> {
    celestial::moon_phases_for_month(year, month as u8)
        .map(|events| {
            events
                .into_iter()
                .map(|e| PhaseEvent {
                    phase: e.phase.name().to_string(),
                    jd: e.jd,
                    elongation: e.elongation,
                })
                .collect()
        })
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[napi(object)]
pub struct MoonPhaseInfo {
    pub phase_name: String,
    pub elongation: f64,
    pub illumination: f64,
    pub prev_phase_name: String,
    pub prev_phase_jd: f64,
    pub next_phase_name: String,
    pub next_phase_jd: f64,
    pub age_days: f64,
}

/// Full Moon phase info for the given JD.
#[napi(js_name = "moonPhaseInfo")]
pub fn moon_phase_info(jd: f64) -> napi::Result<MoonPhaseInfo> {
    celestial::moon_phase_info(jd)
        .map(|i| MoonPhaseInfo {
            phase_name: i.phase_name.to_string(),
            elongation: i.elongation,
            illumination: i.illumination,
            prev_phase_name: i.prev_phase_name.to_string(),
            prev_phase_jd: i.prev_phase_jd,
            next_phase_name: i.next_phase_name.to_string(),
            next_phase_jd: i.next_phase_jd,
            age_days: i.age_days,
        })
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}
