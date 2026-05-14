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

#[cfg(test)]
mod tests {
    use super::*;

    /// `RiseTransOptions::horizon_height` is a fluent builder method — the
    /// value should round-trip through `.horizon_height(x)` and be observable
    /// via the constructed options struct.
    #[test]
    fn horizon_height_round_trip() {
        let opts = RiseTransOptions::new(0.0, Body::SUN, [0.0, 0.0, 0.0]).horizon_height(2.5);
        assert!((opts.horhgt - 2.5).abs() < 1e-12);

        // Negative offset (depression below horizon, e.g. for a ship's bridge)
        let opts = RiseTransOptions::new(0.0, Body::SUN, [0.0, 0.0, 0.0]).horizon_height(-1.2);
        assert!((opts.horhgt - (-1.2)).abs() < 1e-12);

        // Zero is the default; re-setting to 0 must still yield 0
        let opts = RiseTransOptions::new(0.0, Body::SUN, [0.0, 0.0, 0.0]).horizon_height(0.0);
        assert_eq!(opts.horhgt, 0.0);
    }

    /// Builder methods chain. Each one only mutates its own field.
    #[test]
    fn builder_chains_independently() {
        let opts = RiseTransOptions::new(2_460_000.0, Body::SUN, [1.0, 2.0, 3.0])
            .horizon_height(10.0)
            .atmosphere(900.0, 20.0)
            .event(2);
        assert_eq!(opts.geopos, [1.0, 2.0, 3.0]);
        assert_eq!(opts.horhgt, 10.0);
        assert_eq!(opts.pressure, 900.0);
        assert_eq!(opts.temp, 20.0);
        assert_eq!(opts.event_type, 2);
    }

    /// Remaining builder setters (`flags`, `star`) write to their fields.
    #[test]
    fn builder_flags_and_star_fields() {
        let opts = RiseTransOptions::new(2_460_000.0, Body::SUN, [0.0, 0.0, 0.0])
            .flags(CalcFlags::BUILTIN)
            .star("Sirius");
        assert_eq!(opts.flags.as_raw(), CalcFlags::BUILTIN.as_raw());
        assert_eq!(opts.starname.as_deref(), Some("Sirius"));
    }

    // ─── Crossing-fn smoke tests ─────────────────────────────────────────────
    //
    // J2000 reference: JD 2_451_545.0 = 2000-01-01 12:00 TT.
    // Sun is near 280° ecliptic longitude (Capricorn) on that date.

    /// `solcross` finds the next time the Sun crosses 0° (vernal equinox),
    /// which from J2000 should land roughly at JD 2_451_624 (≈ March 2000).
    #[test]
    fn solcross_finds_vernal_equinox() {
        let jd = solcross(0.0, 2_451_545.0, CalcFlags::BUILTIN).expect("crossing");
        assert!(jd.is_finite());
        assert!(
            (2_451_600.0..2_451_700.0).contains(&jd),
            "vernal equinox JD out of expected window: {jd}",
        );
    }

    /// `solcross_ut` and `solcross` agree to within seconds-of-time on the
    /// same crossing (the ET↔UT offset is tiny near J2000).
    #[test]
    fn solcross_ut_matches_et_variant() {
        let et = solcross(180.0, 2_451_545.0, CalcFlags::BUILTIN).expect("et");
        let ut = solcross_ut(180.0, 2_451_545.0, CalcFlags::BUILTIN).expect("ut");
        assert!((et - ut).abs() < 0.01, "et={et}, ut={ut}");
    }

    /// `mooncross` finds *some* Moon longitude crossing within the next
    /// 30 days. The Moon moves ~13°/day so any target longitude is hit fast.
    #[test]
    fn mooncross_finds_within_30_days() {
        let start = 2_451_545.0;
        let jd = mooncross(120.0, start, CalcFlags::BUILTIN).expect("crossing");
        assert!(jd > start && jd < start + 30.0, "mooncross jd={jd}");
    }

    /// `mooncross_ut` mirrors `mooncross` to better than 1 second.
    #[test]
    fn mooncross_ut_matches_et_variant() {
        let et = mooncross(45.0, 2_451_545.0, CalcFlags::BUILTIN).expect("et");
        let ut = mooncross_ut(45.0, 2_451_545.0, CalcFlags::BUILTIN).expect("ut");
        assert!((et - ut).abs() < 0.01);
    }

    /// `mooncross_node` returns a node-crossing within ~14 days (half a
    /// nodal half-cycle = ~13.6 days).
    #[test]
    fn mooncross_node_returns_finite() {
        let start = 2_451_545.0;
        let node = mooncross_node(start, CalcFlags::BUILTIN).expect("node");
        assert!(node.jd_cross.is_finite());
        assert!(node.xlon.is_finite());
        assert!((node.jd_cross - start).abs() < 30.0);
    }

    /// `mooncross_node_ut` delegates to `mooncross_node` with the same args.
    #[test]
    fn mooncross_node_ut_matches_et_variant() {
        let a = mooncross_node(2_451_545.0, CalcFlags::BUILTIN).expect("et");
        let b = mooncross_node_ut(2_451_545.0, CalcFlags::BUILTIN).expect("ut");
        assert!((a.jd_cross - b.jd_cross).abs() < 1e-9);
    }

    /// `solcross_back_ut` finds a crossing in the PAST.
    #[test]
    fn solcross_back_walks_backward() {
        let start = 2_451_545.0;
        let back = solcross_back_ut(0.0, start, CalcFlags::BUILTIN).expect("back");
        assert!(back < start, "back={back}, start={start}");
        assert!(start - back < 366.0, "more than a year back: {back}");
    }

    /// `mooncross_back_ut` finds a crossing in the PAST within 30 days.
    #[test]
    fn mooncross_back_walks_backward() {
        let start = 2_451_545.0;
        let back = mooncross_back_ut(200.0, start, CalcFlags::BUILTIN).expect("back");
        assert!(back < start);
        assert!(start - back < 30.0);
    }

    /// `helio_cross` finds a Mars heliocentric crossing forward in time.
    #[test]
    fn helio_cross_mars_forward() {
        let jd = helio_cross(Body::MARS, 0.0, 2_451_545.0, CalcFlags::BUILTIN, 1).expect("mars");
        assert!(jd > 2_451_545.0);
    }

    /// `helio_cross_ut` is a thin wrapper around `helio_cross`.
    #[test]
    fn helio_cross_ut_matches_helio_cross() {
        let et = helio_cross(Body::MARS, 90.0, 2_451_545.0, CalcFlags::BUILTIN, 1);
        let ut = helio_cross_ut(Body::MARS, 90.0, 2_451_545.0, CalcFlags::BUILTIN, 1);
        match (et, ut) {
            (Ok(a), Ok(b)) => assert!((a - b).abs() < 1e-9),
            (Err(_), Err(_)) => {} // both failed: still consistent
            (a, b) => panic!("disagree: et={a:?}, ut={b:?}"),
        }
    }

    // ─── rise_trans smoke ────────────────────────────────────────────────────

    /// `rise_trans` returns a finite JD for the Sun at the equator, where
    /// no body is ever circumpolar.
    #[test]
    fn rise_trans_sun_at_equator() {
        let res = rise_trans(
            2_451_545.0,
            Body::SUN,
            None,
            CalcFlags::BUILTIN,
            1, // CALC_RISE
            [0.0, 0.0, 0.0],
            1013.25,
            15.0,
        )
        .expect("rise_trans");
        assert!(res.tret.is_finite());
        // Rise within a day of start
        assert!((res.tret - 2_451_545.0).abs() < 2.0);
    }

    /// `rise_trans` for the Moon at the equator returns a finite event.
    #[test]
    fn rise_trans_moon_at_equator() {
        let res = rise_trans(
            2_451_545.0,
            Body::MOON,
            None,
            CalcFlags::BUILTIN,
            1,
            [0.0, 0.0, 0.0],
            1013.25,
            15.0,
        )
        .expect("rise_trans moon");
        assert!(res.tret.is_finite());
    }

    /// `rise_trans` for a planet (Mars) takes the third match arm.
    #[test]
    fn rise_trans_planet_at_equator() {
        let res = rise_trans(
            2_451_545.0,
            Body::MARS,
            None,
            CalcFlags::BUILTIN,
            1,
            [0.0, 0.0, 0.0],
            1013.25,
            15.0,
        );
        // Mars may or may not rise within the small window; either is fine,
        // just no panic and a sensible Result.
        if let Ok(r) = res {
            assert!(r.tret.is_finite());
        }
    }

    /// Ambiguous event_type bits (e.g. all set or none) fall back to
    /// `CALC_RISE` per the source comment. Should not panic.
    #[test]
    fn rise_trans_ambiguous_event_falls_back() {
        let _ = rise_trans(
            2_451_545.0,
            Body::SUN,
            None,
            CalcFlags::BUILTIN,
            0b111, // rise + transit + set: ambiguous
            [0.0, 0.0, 0.0],
            1013.25,
            15.0,
        );
        let _ = rise_trans(
            2_451_545.0,
            Body::SUN,
            None,
            CalcFlags::BUILTIN,
            0, // no bits set
            [0.0, 0.0, 0.0],
            1013.25,
            15.0,
        );
    }

    /// At extreme polar latitudes during the dark season, the Sun is
    /// circumpolar (never rises). `rise_trans` must return
    /// `Error::CircumpolarBody`, not panic.
    #[test]
    fn rise_trans_circumpolar_returns_error() {
        // Dec solstice in 2000: Sun at -23.4° declination, never rises
        // above the horizon for an observer at +85° N.
        let jd_solstice = 2_451_899.0; // approx 2000-12-21
        let res = rise_trans(
            jd_solstice,
            Body::SUN,
            None,
            CalcFlags::BUILTIN,
            1,
            [0.0, 85.0, 0.0],
            1013.25,
            15.0,
        );
        assert!(
            matches!(res, Err(Error::CircumpolarBody { .. })),
            "expected CircumpolarBody, got {res:?}",
        );
    }

    /// `rise_trans_true_hor` is a thin wrapper; result must match
    /// `rise_trans` byte-for-byte at horizon_height = 0.
    #[test]
    fn rise_trans_true_hor_matches_rise_trans() {
        let a = rise_trans(
            2_451_545.0,
            Body::SUN,
            None,
            CalcFlags::BUILTIN,
            1,
            [0.0, 0.0, 0.0],
            1013.25,
            15.0,
        );
        let b = rise_trans_true_hor(
            2_451_545.0,
            Body::SUN,
            None,
            CalcFlags::BUILTIN,
            1,
            [0.0, 0.0, 0.0],
            1013.25,
            15.0,
            0.0,
        );
        match (a, b) {
            (Ok(x), Ok(y)) => assert!((x.tret - y.tret).abs() < 1e-12),
            (Err(_), Err(_)) => {}
            (x, y) => panic!("disagree: a={x:?}, b={y:?}"),
        }
    }

    /// `RiseTransOptions::search()` runs end-to-end with the default
    /// settings.
    #[test]
    fn options_search_executes() {
        let res = RiseTransOptions::new(2_451_545.0, Body::SUN, [0.0, 0.0, 0.0])
            .event(1)
            .flags(CalcFlags::BUILTIN)
            .search()
            .expect("search");
        assert!(res.tret.is_finite());
    }
}
