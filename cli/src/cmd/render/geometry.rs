//! Wheel geometry + planet-label collision avoidance.
//!
//! Pure math, no rendering or context types. Extracted from the render
//! god-file (ARCH-8); re-exported by `mod.rs` so call sites keep the
//! short `super::wx` / `super::spread_labels` names.

use celestial_core::wrap_signed_180;

/// SVG x coordinate for an ecliptic longitude on the wheel.
/// ASC is placed at the 9-o'clock position (leftmost), per astrological convention.
pub(crate) fn wx(cx: f64, r: f64, lon: f64, asc: f64) -> f64 {
    cx + r * wheel_angle(lon, asc).to_radians().cos()
}

/// SVG y coordinate for an ecliptic longitude on the wheel.
pub(crate) fn wy(cy: f64, r: f64, lon: f64, asc: f64) -> f64 {
    cy - r * wheel_angle(lon, asc).to_radians().sin()
}

/// Convert an ecliptic longitude (degrees) to a wheel position in
/// standard math coordinates (0° = east / 3 o'clock, increasing CCW).
/// The chart is rotated so the Ascendant sits at 9 o'clock (180°) and
/// zodiac longitudes increase CCW around the wheel — i.e. moving
/// 30° past the ASC takes us **below** the horizon (lower-left of
/// the wheel) which puts House 1 at the bottom-left, IC at the
/// bottom, DSC on the right and MC at the top, matching the
/// World-of-Wisdom PDF and the AstroDienst / traditional layout.
#[must_use]
pub(crate) fn wheel_angle(lon: f64, asc: f64) -> f64 {
    (180.0 + (lon - asc)).rem_euclid(360.0)
}

/// Clamp a label so it does not drift more than `max_drift` degrees from its
/// "natural" angle.  Uses signed wrap so e.g. natural = 350° and placed = 5°
/// is treated as a +15° drift, not +355°.
fn clamp_drift(placed: &mut f64, natural: f64, max_drift: f64) {
    let drift = wrap_signed_180(*placed - natural);
    if drift.abs() > max_drift {
        *placed = natural + drift.signum() * max_drift;
    }
}

/// One repulsion pass between labels `i` and `j`. Returns `true` if the pair
/// was crowded and got pushed apart.
fn repel_pair(
    placed: &mut [f64],
    natural: &[f64],
    i: usize,
    j: usize,
    min_sep: f64,
    max_drift: f64,
) -> bool {
    let d = wrap_signed_180(placed[j] - placed[i]);
    if d.abs() >= min_sep {
        return false;
    }
    // Push symmetrically; slightly more than half ensures convergence
    let push = (min_sep - d.abs()) * 0.55 + 0.05;
    if d >= 0.0 {
        placed[j] += push;
        placed[i] -= push;
    } else {
        placed[i] += push;
        placed[j] -= push;
    }
    clamp_drift(&mut placed[i], natural[i], max_drift);
    clamp_drift(&mut placed[j], natural[j], max_drift);
    true
}

/// Compute non-overlapping wheel angles for planet degree labels.
///
/// Starts each label at its natural angular position (derived from the planet
/// longitude) and iteratively separates overlapping pairs symmetrically along
/// the arc, keeping each label within `MAX_DRIFT` degrees of its planet.
pub(crate) fn spread_labels(lons: &[f64], asc: f64) -> Vec<f64> {
    // Approximate angular half-width of a degree label (e.g. "29°Gem") at
    // the label ring radius.  At r = RP + 22 ≈ 234px, 10° of arc ≈ 41px,
    // which comfortably brackets a ~36px label.
    const HALF_DEG: f64 = 5.5; // half-width in degrees
    const MIN_SEP: f64 = HALF_DEG * 2.0 + 1.5; // 12.5° minimum centre-to-centre
    const MAX_DRIFT: f64 = 28.0; // max degrees a label may wander from its planet
    const MAX_ITER: usize = 300;

    let n = lons.len();
    let natural: Vec<f64> = lons.iter().map(|&l| wheel_angle(l, asc)).collect();
    let mut placed = natural.clone();

    // Iterative pairwise repulsion until no pair is crowded (or MAX_ITER).
    for _iter in 0..MAX_ITER {
        let mut any = false;
        for i in 0..n {
            for j in (i + 1)..n {
                if repel_pair(&mut placed, &natural, i, j, MIN_SEP, MAX_DRIFT) {
                    any = true;
                }
            }
        }
        if !any {
            break;
        }
    }
    placed.iter().map(|a| a.rem_euclid(360.0)).collect()
}
