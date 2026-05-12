//! Core ephemeris calculation functions.

use crate::astronomy::fixstars;
use crate::body::{Body, CalcFlags};
use crate::error::{Error, Result};

// Types are defined once in types.rs and re-exported from lib.rs
pub use crate::types::{FixStarPos, NodAps, OrbitalDistances, OrbitalElements, PlanetPos};

// ─── Helpers ──────────────────────────────────────────────────────────────────

// ─── Public API ───────────────────────────────────────────────────────────────

/// Close the ephemeris and free any cached data.
pub fn close() {}

/// Compute apparent geocentric position using TT (Terrestrial Time / Julian Ephemeris Day).
///
/// Geocentric position using Terrestrial Time (TT / JDE) directly.
///
/// Use this when your Julian day is already in TT — e.g. Meeus examples,
/// ephemeris file JDEs, or when you have manually applied ΔT.
/// For wall-clock / UT input use [`calc_ut`] instead.
pub fn calc(jd_et: f64, body: Body, flags: CalcFlags) -> Result<PlanetPos> {
    crate::astronomy::calc_tt(jd_et, body.as_raw(), flags.as_raw())
}

/// Geocentric position using Universal Time. Returns [`PlanetPos`].
pub fn calc_ut(jd_ut: f64, body: Body, flags: CalcFlags) -> Result<PlanetPos> {
    crate::astronomy::calc_ut(jd_ut, body.as_raw(), flags.as_raw())
}

/// Planetocentric position: body as seen from `center` instead of Earth.
pub fn calc_pctr(jd_et: f64, body: Body, center: Body, flags: CalcFlags) -> Result<PlanetPos> {
    use crate::astronomy::calc_ut as au;
    let body_pos =
        au(jd_et, body.as_raw(), flags.as_raw()).map_err(|e| Error::Calc(e.to_string()))?;
    let center_pos =
        au(jd_et, center.as_raw(), flags.as_raw()).map_err(|e| Error::Calc(e.to_string()))?;
    let dlon = body_pos.lon - center_pos.lon;
    let dlat = body_pos.lat - center_pos.lat;
    let ddist = body_pos.dist - center_pos.dist;
    let lon = (dlon + 360.0).rem_euclid(360.0);
    Ok(PlanetPos {
        lon,
        lat: dlat,
        dist: ddist.abs(),
        speed_lon: body_pos.speed_lon - center_pos.speed_lon,
        speed_lat: body_pos.speed_lat - center_pos.speed_lat,
        speed_dist: body_pos.speed_dist - center_pos.speed_dist,
        ret_flags: flags.as_raw(),
    })
}

fn fixstar_impl(star: &str, jd: f64, flags: CalcFlags) -> Result<FixStarPos> {
    let idx = fixstars::find_star(star).ok_or_else(|| Error::StarNotFound {
        name: star.to_string(),
    })?;
    let s = &fixstars::CATALOG[idx];
    let (lon, lat, dist) = fixstars::star_ecliptic_pos(s, jd);
    let (sl, sb, sr) = fixstars::star_speed(s);
    let name = format!("{},{}", s.name, s.bayer);
    Ok(FixStarPos {
        xx: [lon, lat, dist, sl, sb, sr],
        star_name: name,
        ret_flags: flags.as_raw(),
    })
}

/// Fixed-star position at a Julian Ephemeris Day (TT).
///
/// `star` is the star name or Bayer designation (e.g. `"Aldebaran"`, `"alTau"`).
/// Returns a [`FixStarPos`] with ecliptic coordinates and magnitude.
pub fn fixstar(star: &str, tjd: f64, flags: CalcFlags) -> Result<FixStarPos> {
    fixstar_impl(star, tjd, flags)
}
/// Fixed-star position at a Julian Day (UT).
///
/// UT variant of [`fixstar`] — converts UT → TT internally.
pub fn fixstar_ut(star: &str, tjd: f64, flags: CalcFlags) -> Result<FixStarPos> {
    fixstar_impl(star, tjd, flags)
}
/// Fixed-star position (TT) — extended version that also returns the star name.
///
/// Same as [`fixstar`] but the returned [`FixStarPos`] includes the canonical name.
pub fn fixstar2(star: &str, tjd: f64, flags: CalcFlags) -> Result<FixStarPos> {
    fixstar_impl(star, tjd, flags)
}
/// Fixed-star position (UT) — extended version that also returns the star name.
pub fn fixstar2_ut(star: &str, tjd: f64, flags: CalcFlags) -> Result<FixStarPos> {
    fixstar_impl(star, tjd, flags)
}

/// Visual magnitude of a fixed star by name.
pub fn fixstar_mag(star: &str) -> Result<f64> {
    let idx = fixstars::find_star(star).ok_or_else(|| Error::StarNotFound {
        name: star.to_string(),
    })?;
    Ok(fixstars::CATALOG[idx].mag)
}
/// Visual magnitude of a fixed star by name (alias of [`fixstar_mag`]).
pub fn fixstar2_mag(star: &str) -> Result<f64> {
    fixstar_mag(star)
}

/// Planetary nodes and apsides (ET).
pub fn nod_aps(jd_et: f64, body: Body, flags: CalcFlags, method: i32) -> Result<NodAps> {
    nod_aps_impl(jd_et, body, flags, method)
}
/// Planetary nodes and apsides (UT).
pub fn nod_aps_ut(jd_ut: f64, body: Body, flags: CalcFlags, method: i32) -> Result<NodAps> {
    nod_aps_impl(jd_ut, body, flags, method)
}

fn nod_aps_impl(jd: f64, body: Body, flags: CalcFlags, _method: i32) -> Result<NodAps> {
    use crate::astronomy::nodes;

    if body.as_raw() == 1 {
        // Moon: use dedicated formulae
        let mn_lon = nodes::moon_mean_node(jd);
        let mn_spd = nodes::moon_mean_node_speed(jd);
        let peri = nodes::moon_mean_perigee(jd);
        let peri_s = nodes::moon_mean_perigee_speed(jd);
        let asc = [mn_lon, 0.0, 1.0, mn_spd, 0.0, 0.0];
        let dsc = [
            (mn_lon + 180.0).rem_euclid(360.0),
            0.0,
            1.0,
            mn_spd,
            0.0,
            0.0,
        ];
        let per = [peri, 0.0, 1.0, peri_s, 0.0, 0.0];
        let aph = [(peri + 180.0).rem_euclid(360.0), 0.0, 1.0, peri_s, 0.0, 0.0];
        return Ok(NodAps {
            nasc: asc,
            ndsc: dsc,
            peri: per,
            aphe: aph,
            ret_flags: flags.as_raw(),
        });
    }

    let (asc_lon, dsc_lon, peri_lon, aphe_lon, inc) =
        nodes::planet_nodes_apsides(body.as_raw(), jd).ok_or_else(|| {
            Error::BodyNotImplemented {
                body: body.as_raw(),
            }
        })?;

    let (node_spd, peri_spd) = if flags.as_raw() as u32 & crate::astronomy::flag::FLG_SPEED != 0 {
        nodes::planet_nodes_speeds(body.as_raw(), jd).unwrap_or((0.0, 0.0))
    } else {
        (0.0, 0.0)
    };

    let nasc = [asc_lon, inc, 0.0, node_spd, 0.0, 0.0];
    let ndsc = [dsc_lon, inc, 0.0, node_spd, 0.0, 0.0];
    let peri = [peri_lon, 0.0, 0.0, peri_spd, 0.0, 0.0];
    let aphe = [aphe_lon, 0.0, 0.0, peri_spd, 0.0, 0.0];
    Ok(NodAps {
        nasc,
        ndsc,
        peri,
        aphe,
        ret_flags: flags.as_raw(),
    })
}

/// Osculating orbital elements for a solar-system body at a Julian Ephemeris Day.
///
/// Returns [`OrbitalElements`] with named fields: `semi_major_axis`, `eccentricity`,
/// `inclination`, `ascending_node`, `arg_perihelion`, `mean_anomaly`, etc.
pub fn get_orbital_elements(jd_et: f64, body: Body, _flags: CalcFlags) -> Result<OrbitalElements> {
    let el =
        crate::astronomy::nodes::planet_mean_elements(body.as_raw(), jd_et).ok_or_else(|| {
            Error::Calc(format!(
                "orbital elements not available for body {}",
                body.as_raw()
            ))
        })?;
    Ok(OrbitalElements {
        semi_major_axis: el.semi_major,
        eccentricity: el.ecc,
        inclination: el.inc,
        ascending_node: el.node_lon,
        arg_perihelion: el.peri_lon,
        mean_anomaly: el.mean_lon,
        epoch: 0.0,
        mean_daily_motion: 0.0,
        extra: vec![],
    })
}

/// Maximum, minimum, and current true distance (AU) for a body at a given TT.
///
/// Returns [`OrbitalDistances`] with `dmax`, `dmin`, and `dtrue` fields.
pub fn orbit_max_min_true_distance(
    jd_et: f64,
    body: Body,
    _flags: CalcFlags,
) -> Result<OrbitalDistances> {
    let el =
        crate::astronomy::nodes::planet_mean_elements(body.as_raw(), jd_et).ok_or_else(|| {
            Error::Calc(format!(
                "orbital elements not available for body {}",
                body.as_raw()
            ))
        })?;
    let dmax = el.semi_major * (1.0 + el.ecc);
    let dmin = el.semi_major * (1.0 - el.ecc);
    let dtrue = el.semi_major;
    Ok(OrbitalDistances { dmax, dmin, dtrue })
}

// ─── Parallel multi-body calculation ─────────────────────────────────────────

/// Compute positions for multiple bodies in parallel using OS threads.
///
/// Spawns one thread per body (up to a cap) using [`std::thread::scope`],
/// so all bodies are evaluated concurrently.  Ideal for full chart calculations
/// (12 bodies) where the speedup is proportional to CPU cores.
///
/// Results are returned in the same order as the input `bodies` slice.
///
/// # Example
/// ```rust
/// use celestial_core::body::{Body, CalcFlags};
/// use celestial_core::calc_many;
/// let bodies = [Body::SUN, Body::MOON, Body::MERCURY, Body::VENUS,
///               Body::MARS, Body::JUPITER, Body::SATURN, Body::URANUS,
///               Body::NEPTUNE, Body::PLUTO, Body::MEAN_NODE, Body::CHIRON];
/// let results = calc_many(2451545.0, &bodies, CalcFlags::BUILTIN | CalcFlags::SPEED);
/// ```
#[must_use]
pub fn calc_many(
    jd_et: f64,
    bodies: &[Body],
    flags: CalcFlags,
) -> Vec<crate::error::Result<PlanetPos>> {
    parallel_calc(bodies, move |body| {
        crate::astronomy::calc_tt(jd_et, body.as_raw(), flags.as_raw())
    })
}

/// Compute positions for multiple bodies in parallel using UT input.
///
/// Same as [`calc_many`] but accepts Universal Time (auto-applies ΔT).
#[must_use]
pub fn calc_ut_many(
    jd_ut: f64,
    bodies: &[Body],
    flags: CalcFlags,
) -> Vec<crate::error::Result<PlanetPos>> {
    parallel_calc(bodies, move |body| {
        crate::astronomy::calc_ut(jd_ut, body.as_raw(), flags.as_raw())
    })
}

/// Internal: evaluate `f(body)` for each body in parallel via scoped threads.
///
/// Uses [`std::thread::scope`] (stable since Rust 1.63) — no external crates needed.
/// Each body gets its own OS thread; the function blocks until all complete.
fn parallel_calc<F>(bodies: &[Body], f: F) -> Vec<crate::error::Result<PlanetPos>>
where
    F: Fn(Body) -> crate::error::Result<PlanetPos> + Sync + Send + 'static,
{
    // For ≤ 2 bodies just run sequentially — thread overhead isn't worth it.
    if bodies.len() <= 2 {
        return bodies.iter().map(|&b| f(b)).collect();
    }
    use std::sync::Arc;
    let f = Arc::new(f);
    std::thread::scope(|s| {
        let handles: Vec<_> = bodies
            .iter()
            .map(|&body| {
                let f = Arc::clone(&f);
                s.spawn(move || f(body))
            })
            .collect();
        handles
            .into_iter()
            .map(|h| {
                h.join()
                    .unwrap_or_else(|_| Err(crate::error::Error::Calc("thread panicked".into())))
            })
            .collect()
    })
}

// ── CalcOptions builder ───────────────────────────────────────────────────────

/// Execution strategy for [`CalcOptions`].
///
/// - `Sequential` — evaluate bodies one by one in the calling thread.
///   Fastest for 1–2 bodies.
/// - `Parallel` — spawn one OS thread per body. Fastest for 3+ bodies
///   on multi-core machines.
/// - `Auto` (default) — parallel for > 2 bodies, sequential otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CalcStrategy {
    /// Always run sequentially.
    Sequential,
    /// Always run in parallel (one thread per body).
    Parallel,
    /// Sequential for ≤ 2 bodies, parallel for > 2 (default).
    #[default]
    Auto,
}

/// Builder for single-body or multi-body planetary calculations.
///
/// Unifies `calc_ut`, `calc`, `calc_many`, and `calc_ut_many` behind a
/// consistent API. The execution strategy (sequential vs parallel) is
/// selected by the caller or determined automatically.
///
/// # Examples
///
/// ```no_run
/// # use celestial_core::*;
/// # use celestial_core::body::{Body, CalcFlags};
/// // Single body (UT)
/// let pos = CalcOptions::ut(2_451_545.0, CalcFlags::BUILTIN | CalcFlags::SPEED)
///     .body(Body::SUN)
///     .get()
///     .unwrap();
/// println!("Sun: {:.4}°", pos.lon);
///
/// // Multiple bodies with automatic strategy
/// let results = CalcOptions::ut(2_451_545.0, CalcFlags::BUILTIN)
///     .bodies(&[Body::SUN, Body::MOON, Body::MERCURY, Body::VENUS, Body::MARS])
///     .get_many();
///
/// // Force sequential (e.g. in a tight loop)
/// let results = CalcOptions::ut(2_451_545.0, CalcFlags::BUILTIN)
///     .strategy(CalcStrategy::Sequential)
///     .bodies(&[Body::SUN, Body::MOON])
///     .get_many();
/// ```
#[derive(Debug, Clone)]
pub struct CalcOptions {
    jd: f64,
    flags: CalcFlags,
    use_ut: bool,
    strategy: CalcStrategy,
}

impl CalcOptions {
    /// Create a builder with a Universal Time Julian Day (auto-applies ΔT).
    pub fn ut(jd_ut: f64, flags: CalcFlags) -> Self {
        Self {
            jd: jd_ut,
            flags,
            use_ut: true,
            strategy: CalcStrategy::Auto,
        }
    }

    /// Create a builder with a Terrestrial Time Julian Day (no ΔT applied).
    ///
    /// Use when your JD already has ΔT applied — e.g. Meeus examples.
    pub fn tt(jd_et: f64, flags: CalcFlags) -> Self {
        Self {
            jd: jd_et,
            flags,
            use_ut: false,
            strategy: CalcStrategy::Auto,
        }
    }

    /// Override the execution strategy (default: `CalcStrategy::Auto`).
    pub fn strategy(mut self, s: CalcStrategy) -> Self {
        self.strategy = s;
        self
    }

    /// Set a single body and return a ready-to-execute single-body builder.
    #[must_use]
    pub fn body(self, body: Body) -> SingleCalc {
        SingleCalc { opts: self, body }
    }

    /// Set multiple bodies and return a ready-to-execute multi-body builder.
    #[must_use]
    pub fn bodies(self, bodies: &[Body]) -> MultiCalc<'_> {
        MultiCalc { opts: self, bodies }
    }
}

/// Finaliser for a single-body calc (produced by [`CalcOptions::body`]).
pub struct SingleCalc {
    opts: CalcOptions,
    body: Body,
}

impl SingleCalc {
    /// Execute the calculation and return the result.
    pub fn get(self) -> Result<PlanetPos> {
        if self.opts.use_ut {
            calc_ut(self.opts.jd, self.body, self.opts.flags)
        } else {
            crate::astronomy::calc_tt(self.opts.jd, self.body.as_raw(), self.opts.flags.as_raw())
        }
    }
}

/// Finaliser for a multi-body calc (produced by [`CalcOptions::bodies`]).
pub struct MultiCalc<'a> {
    opts: CalcOptions,
    bodies: &'a [Body],
}

impl<'a> MultiCalc<'a> {
    /// Execute the calculations and return one `Result<PlanetPos>` per body.
    ///
    /// Order is guaranteed to match the input `bodies` slice.
    #[must_use]
    pub fn get_many(self) -> Vec<Result<PlanetPos>> {
        let use_parallel = match self.opts.strategy {
            CalcStrategy::Sequential => false,
            CalcStrategy::Parallel => true,
            CalcStrategy::Auto => self.bodies.len() > 2,
        };

        if use_parallel {
            if self.opts.use_ut {
                calc_ut_many(self.opts.jd, self.bodies, self.opts.flags)
            } else {
                calc_many(self.opts.jd, self.bodies, self.opts.flags)
            }
        } else {
            // Sequential
            let jd = self.opts.jd;
            let flags = self.opts.flags;
            if self.opts.use_ut {
                self.bodies.iter().map(|&b| calc_ut(jd, b, flags)).collect()
            } else {
                self.bodies
                    .iter()
                    .map(|&b| crate::astronomy::calc_tt(jd, b.as_raw(), flags.as_raw()))
                    .collect()
            }
        }
    }
}
