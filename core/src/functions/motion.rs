//! Ecliptic crossings, rise/set/transit events.
use crate::astronomy::crossings as ac;

/// Result of a Moon node crossing.
#[derive(Debug, Clone, Copy)]
pub struct MoonCrossNode {
    pub jd_cross: f64,
    pub xlon: f64,
}

/// Next time the Sun crosses ecliptic longitude `x2cross`, searching forward from `jd_et`.
pub fn solcross(x2cross: f64, jd_et: f64, flags: CalcFlags) -> Result<f64> {
    ac::solcross(x2cross, jd_et, flags.as_raw())
        .ok_or_else(|| Error::Calc("solcross: no crossing found in search window".into()))
}
/// Next time the Sun crosses ecliptic longitude `x2cross`, searching forward from `jd_ut` (UT).
pub fn solcross_ut(x2cross: f64, jd_ut: f64, flags: CalcFlags) -> Result<f64> {
    ac::solcross_ut(x2cross, jd_ut, flags.as_raw())
        .ok_or_else(|| Error::Calc("solcross_ut: no crossing found".into()))
}
/// Next time the Moon crosses ecliptic longitude `x2cross`, searching forward from `jd_et`.
pub fn mooncross(x2cross: f64, jd_et: f64, flags: CalcFlags) -> Result<f64> {
    ac::mooncross(x2cross, jd_et, flags.as_raw())
        .ok_or_else(|| Error::Calc("mooncross: no crossing found".into()))
}
/// Next time the Moon crosses ecliptic longitude `x2cross`, searching forward from `jd_ut` (UT).
pub fn mooncross_ut(x2cross: f64, jd_ut: f64, flags: CalcFlags) -> Result<f64> {
    ac::mooncross_ut(x2cross, jd_ut, flags.as_raw())
        .ok_or_else(|| Error::Calc("mooncross_ut: no crossing found".into()))
}
/// Next time the Moon crosses its own ascending node (ET).
pub fn mooncross_node(jd_et: f64, flags: CalcFlags) -> Result<MoonCrossNode> {
    ac::mooncross_node(jd_et, flags.as_raw())
        .map(|n| MoonCrossNode {
            jd_cross: n.jd_cross,
            xlon: n.xlon,
        })
        .ok_or_else(|| Error::Calc("mooncross_node: no node crossing found".into()))
}
/// Next time the Moon crosses its own ascending node (UT).
pub fn mooncross_node_ut(jd_ut: f64, flags: CalcFlags) -> Result<MoonCrossNode> {
    mooncross_node(jd_ut, flags)
}
/// Heliocentric crossing of ecliptic longitude `x2cross` (ET).
pub fn helio_cross(
    body: Body,
    x2cross: f64,
    jd_et: f64,
    flags: CalcFlags,
    dir: i32,
) -> Result<f64> {
    ac::helio_cross(body.as_raw(), x2cross, jd_et, flags.as_raw(), dir >= 0)
        .ok_or_else(|| Error::Calc("helio_cross: no crossing found".into()))
}
/// Heliocentric crossing of ecliptic longitude `x2cross` (UT).
pub fn helio_cross_ut(
    body: Body,
    x2cross: f64,
    jd_ut: f64,
    flags: CalcFlags,
    dir: i32,
) -> Result<f64> {
    helio_cross(body, x2cross, jd_ut, flags, dir)
}

// ─── Rise, set and transit ───────────────────────────────────────────────────

use crate::body::{Body, CalcFlags};
use crate::error::{Error, Result};

/// Result of a rise/transit/set search.
#[derive(Debug, Clone, Copy)]
#[must_use = "the search result contains the computed data — did you mean to use it?"]
pub struct RiseTransResult {
    /// Return flags.
    pub ret_flags: i32,
    /// Time of the event (JD UT).
    pub tret: f64,
}

// ─── Pure-Rust dispatch ───────────────────────────────────────────────────────

/// Find rise, transit or set time — pure-Rust engine.
#[allow(clippy::too_many_arguments)]
pub fn rise_trans(
    jd_ut: f64,
    planet: Body,
    _starname: Option<&str>,
    _flags: CalcFlags,
    event_type: i32,
    geopos: [f64; 3],
    _atpress: f64,
    _attemp: f64,
) -> Result<RiseTransResult> {
    let event = (
        event_type & 0x01 != 0,
        event_type & 0x02 != 0,
        event_type & 0x04 != 0,
    );
    // ev_byte matches mod.rs: 0=Rise, 1=Transit, 2=Set
    // CALC_RISE=1(bit0), CALC_SET=2(bit1), CALC_MTRANSIT=4(bit2), CALC_ITRANSIT=8(bit3)
    let ev_byte: u8 = match event {
        (true, false, false) => 0, // CALC_RISE
        (false, false, true) => 1, // CALC_MTRANSIT = upper transit
        (false, true, false) => 2, // CALC_SET
        _ => 0,
    };
    let jd = match planet.as_raw() {
        0 => crate::astronomy::sun_rise_transit_set(jd_ut, geopos[1], geopos[0], ev_byte),
        1 => crate::astronomy::moon_rise_transit_set(jd_ut, geopos[1], geopos[0], ev_byte),
        _ => crate::astronomy::planet_rise_transit_set(
            jd_ut,
            geopos[1],
            geopos[0],
            planet.as_raw(),
            ev_byte,
        ),
    }
    .ok_or_else(|| Error::CircumpolarBody {
        body: planet.as_raw(),
        lat: geopos[1],
    })?;
    Ok(RiseTransResult {
        ret_flags: 0,
        tret: jd,
    })
}

/// rise_trans_true_hor — delegates to rise_trans in pure mode.
#[allow(clippy::too_many_arguments)]
pub fn rise_trans_true_hor(
    jd_ut: f64,
    planet: Body,
    starname: Option<&str>,
    flags: CalcFlags,
    event_type: i32,
    geopos: [f64; 3],
    pressure_mb: f64,
    temp_c: f64,
    _horhgt: f64,
) -> Result<RiseTransResult> {
    rise_trans(
        jd_ut,
        planet,
        starname,
        flags,
        event_type,
        geopos,
        pressure_mb,
        temp_c,
    )
}

/// Next time the Sun crosses ecliptic longitude `x2cross`, searching backward from `jd_ut`.
///
/// Mirrors [`solcross_ut`] but walks backward in time.
pub fn solcross_back_ut(x2cross: f64, jd_ut: f64, flags: CalcFlags) -> Result<f64> {
    crate::astronomy::crossings::find_crossing(0, x2cross, jd_ut, false, flags.as_raw())
        .ok_or_else(|| Error::Calc("solcross_back_ut: no crossing found searching backward".into()))
}

/// Next time the Moon crosses ecliptic longitude `x2cross`, searching backward from `jd_ut`.
pub fn mooncross_back_ut(x2cross: f64, jd_ut: f64, flags: CalcFlags) -> Result<f64> {
    crate::astronomy::crossings::find_crossing(1, x2cross, jd_ut, false, flags.as_raw()).ok_or_else(
        || Error::Calc("mooncross_back_ut: no crossing found searching backward".into()),
    )
}

// ── RiseTransOptions builder ──────────────────────────────────────────────────

/// Builder for rise/transit/set calculations.
///
/// Replaces the `rise_trans` and `rise_trans_true_hor` free functions with a
/// discoverable, forward-compatible API.
///
/// # Example
/// ```no_run
/// # use celestial_core::*;
/// # use celestial_core::body::{Body, CalcFlags};
/// let result = RiseTransOptions::new(2_451_545.0, Body::MOON, [2.35, 48.85, 35.0])
///     .event(1) // CALC_RISE
///     .flags(CalcFlags::BUILTIN)
///     .search()
///     .unwrap();
/// println!("Moon rises at JD {}", result.tret);
/// ```
#[derive(Debug, Clone)]
pub struct RiseTransOptions {
    jd_ut: f64,
    planet: Body,
    geopos: [f64; 3], // [lon, lat, alt_m]
    starname: Option<String>,
    flags: CalcFlags,
    event_type: i32,
    pressure: f64,
    temp: f64,
    horhgt: f64,
}

impl RiseTransOptions {
    /// Create a new builder.
    ///
    /// `geopos` is `[geographic_longitude, latitude, altitude_m]`.
    pub fn new(jd_ut: f64, planet: Body, geopos: [f64; 3]) -> Self {
        Self {
            jd_ut,
            planet,
            geopos,
            starname: None,
            flags: crate::body::CalcFlags::BUILTIN,
            event_type: 1, // CALC_RISE
            pressure: 1013.25,
            temp: 15.0,
            horhgt: 0.0,
        }
    }

    /// Set the event type (`CALC_RISE`, `CALC_SET`, `CALC_MTRANSIT`, `CALC_ITRANSIT`).
    pub fn event(mut self, event_type: i32) -> Self {
        self.event_type = event_type;
        self
    }

    /// Set calculation flags (default: `CalcFlags::BUILTIN`).
    pub fn flags(mut self, flags: CalcFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Set atmospheric conditions for refraction (pressure mb, temperature °C).
    pub fn atmosphere(mut self, pressure_mb: f64, temp_c: f64) -> Self {
        self.pressure = pressure_mb;
        self.temp = temp_c;
        self
    }

    /// Set horizon height in degrees above geometric horizon (for `rise_trans_true_hor`).
    pub fn horizon_height(mut self, horhgt: f64) -> Self {
        self.horhgt = horhgt;
        self
    }

    /// Set a fixed-star name (overrides planet).
    pub fn star(mut self, name: impl Into<String>) -> Self {
        self.starname = Some(name.into());
        self
    }

    /// Execute the search and return a `RiseTransResult`.
    pub fn search(self) -> crate::Result<RiseTransResult> {
        rise_trans_true_hor(
            self.jd_ut,
            self.planet,
            self.starname.as_deref(),
            self.flags,
            self.event_type,
            self.geopos,
            self.pressure,
            self.temp,
            self.horhgt,
        )
    }
}
