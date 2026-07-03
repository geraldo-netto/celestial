//! Core ephemeris calculation functions.

use crate::astronomy::constants::{norm_deg, to_deg, to_rad};
use crate::astronomy::fixstars;
use crate::body::{Body, CalcFlags};
use crate::error::{Error, Result};
use crate::units::JulianDay;

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
pub fn calc(jd_et: JulianDay, body: Body, flags: CalcFlags) -> Result<PlanetPos> {
    crate::astronomy::calc_tt(jd_et.get(), body.as_raw(), flags.as_raw())
}

/// Geocentric position using Universal Time. Returns [`PlanetPos`].
pub fn calc_ut(jd_ut: JulianDay, body: Body, flags: CalcFlags) -> Result<PlanetPos> {
    crate::astronomy::calc_ut(jd_ut, body.as_raw(), flags.as_raw())
}

/// Planetocentric position: body as seen from `center` instead of Earth.
pub fn calc_pctr(
    jd_et: JulianDay,
    body: Body,
    center: Body,
    flags: CalcFlags,
) -> Result<PlanetPos> {
    let jde = jd_et.get();
    let flags_pos = flags & !(CalcFlags::SPEED | CalcFlags::SPEED3);
    let (lon, lat, dist) = planetocentric_coords(jde, body, center, flags_pos)?;
    let (speed_lon, speed_lat, speed_dist) = planetocentric_speed(jde, body, center, flags)?;
    Ok(PlanetPos {
        lon,
        lat,
        dist,
        speed_lon,
        speed_lat,
        speed_dist,
        ret_flags: flags.as_raw(),
    })
}

fn planetocentric_coords(
    jde: f64,
    body: Body,
    center: Body,
    flags: CalcFlags,
) -> Result<(f64, f64, f64)> {
    let body_pos = crate::astronomy::calc_tt(jde, body.as_raw(), flags.as_raw())?;
    let center_pos = crate::astronomy::calc_tt(jde, center.as_raw(), flags.as_raw())?;
    let rel = subtract_vec(to_cartesian(&body_pos), to_cartesian(&center_pos));
    Ok(to_spherical(rel))
}

fn planetocentric_speed(
    jde: f64,
    body: Body,
    center: Body,
    flags: CalcFlags,
) -> Result<(f64, f64, f64)> {
    if !flags.is_speed() && (flags & CalcFlags::SPEED3).as_raw() == 0 {
        return Ok((0.0, 0.0, 0.0));
    }
    let flags_pos = flags & !(CalcFlags::SPEED | CalcFlags::SPEED3);
    let plus = planetocentric_coords(jde + 0.5, body, center, flags_pos)?;
    let minus = planetocentric_coords(jde - 0.5, body, center, flags_pos)?;
    Ok((
        angle_delta(plus.0, minus.0),
        plus.1 - minus.1,
        plus.2 - minus.2,
    ))
}

fn to_cartesian(pos: &PlanetPos) -> [f64; 3] {
    let lon = to_rad(pos.lon);
    let lat = to_rad(pos.lat);
    let (sin_lon, cos_lon) = lon.sin_cos();
    let (sin_lat, cos_lat) = lat.sin_cos();
    [
        pos.dist * cos_lat * cos_lon,
        pos.dist * cos_lat * sin_lon,
        pos.dist * sin_lat,
    ]
}

fn subtract_vec(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn to_spherical(v: [f64; 3]) -> (f64, f64, f64) {
    let xy = v[0].hypot(v[1]);
    let dist = xy.hypot(v[2]);
    if dist == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    (
        norm_deg(to_deg(v[1].atan2(v[0]))),
        to_deg(v[2].atan2(xy)),
        dist,
    )
}

fn angle_delta(plus: f64, minus: f64) -> f64 {
    (plus - minus + 540.0).rem_euclid(360.0) - 180.0
}

fn fixstar_impl(star: &str, jd: f64, flags: CalcFlags) -> Result<FixStarPos> {
    let idx = fixstars::find_star(star).ok_or_else(|| Error::StarNotFound {
        name: star.to_string(),
    })?;
    let s = &fixstars::CATALOG[idx];
    let (lon, lat, dist) = fixstars::star_ecliptic_pos(s, JulianDay::new(jd));
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
pub fn fixstar(star: &str, tjd: JulianDay, flags: CalcFlags) -> Result<FixStarPos> {
    let tjd: f64 = tjd.into();
    fixstar_impl(star, tjd, flags)
}
/// Fixed-star position at a Julian Day (UT).
///
/// UT variant of [`fixstar`] — converts UT → TT internally.
pub fn fixstar_ut(star: &str, tjd: JulianDay, flags: CalcFlags) -> Result<FixStarPos> {
    let tjd: f64 = tjd.into();
    fixstar_impl(star, tjd, flags)
}
/// Fixed-star position (TT) — extended version that also returns the star name.
///
/// Same as [`fixstar`] but the returned [`FixStarPos`] includes the canonical name.
pub fn fixstar2(star: &str, tjd: JulianDay, flags: CalcFlags) -> Result<FixStarPos> {
    let tjd: f64 = tjd.into();
    fixstar_impl(star, tjd, flags)
}
/// Fixed-star position (UT) — extended version that also returns the star name.
pub fn fixstar2_ut(star: &str, tjd: JulianDay, flags: CalcFlags) -> Result<FixStarPos> {
    let tjd: f64 = tjd.into();
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
pub fn nod_aps(jd_et: JulianDay, body: Body, flags: CalcFlags, method: i32) -> Result<NodAps> {
    let jd_et: f64 = jd_et.into();
    nod_aps_impl(jd_et, body, flags, method)
}
/// Planetary nodes and apsides (UT).
pub fn nod_aps_ut(jd_ut: JulianDay, body: Body, flags: CalcFlags, method: i32) -> Result<NodAps> {
    let jd_ut: f64 = jd_ut.into();
    nod_aps_impl(jd_ut, body, flags, method)
}

fn nod_aps_impl(jd: f64, body: Body, flags: CalcFlags, _method: i32) -> Result<NodAps> {
    use crate::astronomy::nodes;

    if body.as_raw() == 1 {
        // Moon: use dedicated formulae
        let mn_lon = nodes::moon_mean_node(JulianDay::new(jd));
        let mn_spd = nodes::moon_mean_node_speed(JulianDay::new(jd));
        let peri = nodes::moon_mean_perigee(JulianDay::new(jd));
        let peri_s = nodes::moon_mean_perigee_speed(JulianDay::new(jd));
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
        nodes::planet_nodes_apsides(body.as_raw(), JulianDay::new(jd)).ok_or_else(|| {
            Error::BodyNotImplemented {
                body: body.as_raw(),
            }
        })?;

    let (node_spd, peri_spd) = if flags.as_raw() as u32 & crate::astronomy::flag::FLG_SPEED != 0 {
        nodes::planet_nodes_speeds(body.as_raw(), JulianDay::new(jd)).unwrap_or((0.0, 0.0))
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
pub fn get_orbital_elements(
    jd_et: JulianDay,
    body: Body,
    _flags: CalcFlags,
) -> Result<OrbitalElements> {
    let jd_et: f64 = jd_et.into();
    let el = crate::astronomy::nodes::planet_mean_elements(body.as_raw(), JulianDay::new(jd_et))
        .ok_or_else(|| {
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
    jd_et: JulianDay,
    body: Body,
    _flags: CalcFlags,
) -> Result<OrbitalDistances> {
    let jd_et: f64 = jd_et.into();
    let el = crate::astronomy::nodes::planet_mean_elements(body.as_raw(), JulianDay::new(jd_et))
        .ok_or_else(|| {
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
/// Spawns up to [`std::thread::available_parallelism`] workers using
/// [`std::thread::scope`], then chunks bodies across those workers. Ideal for
/// full chart calculations where the speedup is proportional to CPU cores.
///
/// Results are returned in the same order as the input `bodies` slice.
///
/// # Example
/// ```rust
/// use celestial_core::body::{Body, CalcFlags};
/// use celestial_core::{calc_many, JulianDay};
/// let bodies = [Body::SUN, Body::MOON, Body::MERCURY, Body::VENUS,
///               Body::MARS, Body::JUPITER, Body::SATURN, Body::URANUS,
///               Body::NEPTUNE, Body::PLUTO, Body::MEAN_NODE, Body::CHIRON];
/// let results = calc_many(JulianDay::new(2451545.0), &bodies, CalcFlags::BUILTIN | CalcFlags::SPEED);
/// ```
#[must_use]
pub fn calc_many(
    jd_et: JulianDay,
    bodies: &[Body],
    flags: CalcFlags,
) -> Vec<crate::error::Result<PlanetPos>> {
    let jd_et = jd_et.get();
    parallel_calc(bodies, move |body| {
        crate::astronomy::calc_tt(jd_et, body.as_raw(), flags.as_raw())
    })
}

/// Compute positions for multiple bodies in parallel using UT input.
///
/// Same as [`calc_many`] but accepts Universal Time (auto-applies ΔT).
#[must_use]
pub fn calc_ut_many(
    jd_ut: JulianDay,
    bodies: &[Body],
    flags: CalcFlags,
) -> Vec<crate::error::Result<PlanetPos>> {
    let jd_ut = jd_ut.get();
    parallel_calc(bodies, move |body| {
        crate::astronomy::calc_ut(JulianDay::new(jd_ut), body.as_raw(), flags.as_raw())
    })
}

/// Internal: evaluate `f(body)` for each body in parallel via scoped threads.
///
/// Uses [`std::thread::scope`] (stable since Rust 1.63) — no external crates needed.
/// Bodies are chunked across a capped worker set; the function blocks until all
/// chunks complete.
fn parallel_calc<F>(bodies: &[Body], f: F) -> Vec<crate::error::Result<PlanetPos>>
where
    F: Fn(Body) -> crate::error::Result<PlanetPos> + Sync + Send + 'static,
{
    // For ≤ 2 bodies just run sequentially — thread overhead isn't worth it.
    if bodies.len() <= 2 {
        return bodies.iter().map(|&b| f(b)).collect();
    }
    use std::sync::Arc;
    let config = crate::functions::config::current_config();
    let chunk_size = parallel_chunk_size(bodies.len());
    let f = Arc::new(f);
    std::thread::scope(|s| {
        let handles: Vec<_> = bodies
            .chunks(chunk_size)
            .enumerate()
            .map(|(chunk_idx, chunk)| {
                let f = Arc::clone(&f);
                let offset = chunk_idx * chunk_size;
                let count = chunk.len();
                let handle = s.spawn(move || {
                    crate::functions::config::set_thread_config(config);
                    chunk
                        .iter()
                        .enumerate()
                        .map(|(i, &body)| (offset + i, f(body)))
                        .collect::<Vec<_>>()
                });
                (offset, count, handle)
            })
            .collect();
        let mut indexed = Vec::with_capacity(bodies.len());
        for (offset, count, handle) in handles {
            match handle.join() {
                Ok(results) => indexed.extend(results),
                Err(_) => indexed.extend(thread_panic_results(offset, count)),
            }
        }
        sort_indexed_results(indexed)
    })
}

fn parallel_worker_count(body_count: usize) -> usize {
    if body_count == 0 {
        return 0;
    }
    std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .clamp(1, body_count)
}

fn parallel_chunk_size(body_count: usize) -> usize {
    let workers = parallel_worker_count(body_count).max(1);
    body_count.div_ceil(workers)
}

fn thread_panic_results(
    offset: usize,
    count: usize,
) -> Vec<(usize, crate::error::Result<PlanetPos>)> {
    (0..count)
        .map(|i| {
            (
                offset + i,
                Err(crate::error::Error::Calc("thread panicked".into())),
            )
        })
        .collect()
}

fn sort_indexed_results(
    mut indexed: Vec<(usize, crate::error::Result<PlanetPos>)>,
) -> Vec<crate::error::Result<PlanetPos>> {
    indexed.sort_by_key(|(i, _)| *i);
    indexed.into_iter().map(|(_, result)| result).collect()
}

// ── CalcOptions builder ───────────────────────────────────────────────────────

/// Execution strategy for [`CalcOptions`].
///
/// - `Sequential` — evaluate bodies one by one in the calling thread.
///   Fastest for 1–2 bodies.
/// - `Parallel` — chunk bodies across a capped worker set. Fastest for 3+
///   bodies on multi-core machines.
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
/// let pos = CalcOptions::ut(JulianDay::new(2_451_545.0), CalcFlags::BUILTIN | CalcFlags::SPEED)
///     .body(Body::SUN)
///     .get()
///     .unwrap();
/// println!("Sun: {:.4}°", pos.lon);
///
/// // Multiple bodies with automatic strategy
/// let results = CalcOptions::ut(JulianDay::new(2_451_545.0), CalcFlags::BUILTIN)
///     .bodies(&[Body::SUN, Body::MOON, Body::MERCURY, Body::VENUS, Body::MARS])
///     .get_many();
///
/// // Force sequential (e.g. in a tight loop)
/// let results = CalcOptions::ut(JulianDay::new(2_451_545.0), CalcFlags::BUILTIN)
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
    pub fn ut(jd_ut: JulianDay, flags: CalcFlags) -> Self {
        let jd_ut: f64 = jd_ut.into();
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
    pub fn tt(jd_et: JulianDay, flags: CalcFlags) -> Self {
        let jd_et: f64 = jd_et.into();
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
            calc_ut(JulianDay::new(self.opts.jd), self.body, self.opts.flags)
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
                calc_ut_many(JulianDay::new(self.opts.jd), self.bodies, self.opts.flags)
            } else {
                calc_many(JulianDay::new(self.opts.jd), self.bodies, self.opts.flags)
            }
        } else {
            // Sequential
            let jd = self.opts.jd;
            let flags = self.opts.flags;
            if self.opts.use_ut {
                self.bodies
                    .iter()
                    .map(|&b| calc_ut(JulianDay::new(jd), b, flags))
                    .collect()
            } else {
                self.bodies
                    .iter()
                    .map(|&b| crate::astronomy::calc_tt(jd, b.as_raw(), flags.as_raw()))
                    .collect()
            }
        }
    }
}

#[cfg(test)]
mod cov_tests {
    use super::*;
    use crate::body::SiderealMode;
    use crate::functions::config;
    use crate::units::{Latitude, Longitude};

    struct ConfigGuard(config::EngineConfig);

    impl ConfigGuard {
        fn new() -> Self {
            Self(config::current_config())
        }
    }

    impl Drop for ConfigGuard {
        fn drop(&mut self) {
            config::set_thread_config(self.0);
        }
    }

    fn lon_diff(a: f64, b: f64) -> f64 {
        (a - b + 180.0).rem_euclid(360.0) - 180.0
    }

    #[test]
    fn parallel_calc_short_input_sequential_path() {
        // ≤ 2 bodies trips the sequential fast-path.
        let bodies = [Body::SUN, Body::MOON];
        let out = parallel_calc(&bodies, |b| {
            calc_ut(JulianDay::new(2_451_545.0), b, CalcFlags::BUILTIN)
        });
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(Result::is_ok));
    }

    #[test]
    fn parallel_calc_threaded_path() {
        // > 2 bodies trips the scoped-thread path.
        let bodies = [
            Body::SUN,
            Body::MOON,
            Body::MERCURY,
            Body::VENUS,
            Body::MARS,
        ];
        let out = parallel_calc(&bodies, |b| {
            calc_ut(JulianDay::new(2_451_545.0), b, CalcFlags::BUILTIN)
        });
        assert_eq!(out.len(), 5);
        assert!(out.iter().all(Result::is_ok));
    }

    #[test]
    fn parallel_calc_caps_large_inputs_to_worker_count() {
        let body_count = 100_000;
        let workers = parallel_worker_count(body_count);
        let chunk_size = parallel_chunk_size(body_count);
        let chunk_count = body_count.div_ceil(chunk_size);
        let cap = std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1);
        assert!(workers <= cap);
        assert!(chunk_count <= workers);
    }

    #[test]
    fn parallel_calc_chunked_path_preserves_input_order() {
        let bodies: Vec<Body> = (0..64).rev().map(Body).collect();
        let out = parallel_calc(&bodies, |body| {
            Ok(PlanetPos {
                lon: body.as_raw() as f64,
                ..PlanetPos::default()
            })
        });
        let lons = out
            .into_iter()
            .map(|r| r.map(|pos| pos.lon))
            .collect::<Result<Vec<_>>>();
        let expected = bodies
            .iter()
            .map(|body| body.as_raw() as f64)
            .collect::<Vec<_>>();
        assert_eq!(lons.unwrap(), expected);
    }

    #[test]
    fn calc_ut_many_threaded_path_keeps_sidereal_mode() {
        let _guard = ConfigGuard::new();
        config::set_sid_mode(SiderealMode::RAMAN, 0.0, 0.0);
        let jd = JulianDay::new(2_451_545.0);
        let flags = CalcFlags::BUILTIN | CalcFlags::SIDEREAL;
        let short = [Body::SUN, Body::MOON];
        let long = [Body::SUN, Body::MOON, Body::MERCURY];
        let seq = calc_ut_many(jd, &short, flags)[0].as_ref().unwrap().lon;
        let par = calc_ut_many(jd, &long, flags)[0].as_ref().unwrap().lon;
        assert!(lon_diff(seq, par).abs() < 1e-10);
    }

    #[test]
    fn calc_ut_many_threaded_path_keeps_topocentric_origin() {
        let _guard = ConfigGuard::new();
        config::set_topo(Longitude::new(-75.0), Latitude::new(40.0), 120.0);
        let jd = JulianDay::new(2_451_545.0);
        let flags = CalcFlags::BUILTIN | CalcFlags::TOPOCENTRIC;
        let short = [Body::MOON, Body::SUN];
        let long = [Body::MOON, Body::SUN, Body::MERCURY];
        let seq = calc_ut_many(jd, &short, flags)[0].as_ref().unwrap().lon;
        let par = calc_ut_many(jd, &long, flags)[0].as_ref().unwrap().lon;
        assert!(lon_diff(seq, par).abs() < 1e-10);
    }

    #[test]
    fn calc_ut_many_threaded_path_keeps_delta_t_override() {
        let _guard = ConfigGuard::new();
        config::set_delta_t_userdef(86_400.0);
        let jd = JulianDay::new(2_451_545.0);
        let bodies = [Body::SUN, Body::MOON, Body::MERCURY];
        let many = calc_ut_many(jd, &bodies, CalcFlags::BUILTIN);
        let one = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN).unwrap();
        let got = many[0].as_ref().unwrap();
        assert!(lon_diff(got.lon, one.lon).abs() < 1e-10);
    }
}
