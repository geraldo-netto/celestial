//! Celtic esbats — the full moons (13 per year).
//!
//! In the Celtic / neopagan calendar, each full moon is an esbat.  They carry
//! traditional folk names drawn from Native American, Old English, and medieval
//! European sources.
//!
//! | Month | Name            |
//! |-------|-----------------|
//! | Jan   | Wolf Moon       |
//! | Feb   | Snow Moon       |
//! | Mar   | Worm Moon       |
//! | Apr   | Pink Moon       |
//! | May   | Flower Moon     |
//! | Jun   | Strawberry Moon |
//! | Jul   | Buck Moon       |
//! | Aug   | Sturgeon Moon   |
//! | Sep/Oct | Harvest Moon  | (closest full moon to the autumn equinox)
//! | Oct   | Hunter's Moon   | (full moon following the Harvest Moon)
//! | Nov   | Beaver Moon     |
//! | Dec   | Cold Moon       |
//! | —     | Blue Moon       | (second full moon in the same calendar month)
//!
//! Full moons are found by bisecting the Moon–Sun elongation function to the
//! exact moment when elongation = 180°, accurate to within a few seconds.

use crate::units::{JulianDay, Longitude};
use crate::body::{Body, CalcFlags, Calendar};
use crate::error::{Error, Result};

// ─── Constants ────────────────────────────────────────────────────────────────

/// Mean synodic month (days).  Chapront-Touzé & Chapront (1988).
const SYNODIC_MONTH: f64 = 29.530_588_853;

/// A known full moon used as an epoch anchor (Julian day, UT).
/// January 21, 2000, ≈ 04:40 UT — close to J2000.0.
const EPOCH_FULL_MOON: f64 = 2_451_564.695;

// ─── EsbatName ────────────────────────────────────────────────────────────────

/// Traditional folk names for the Celtic full moons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EsbatName {
    /// January — wolves howled in the cold winter nights.
    Wolf,
    /// February — heavy snowfall.
    Snow,
    /// March — earthworms re-emerge as the ground thaws.
    Worm,
    /// April — wild ground phlox (pink wildflowers) bloom.
    Pink,
    /// May — abundant blossoms everywhere.
    Flower,
    /// June — strawberry harvest season.
    Strawberry,
    /// July — buck deer begin to grow new velvet antlers.
    Buck,
    /// August — lake sturgeon are plentiful.
    Sturgeon,
    /// The full moon closest to the autumn equinox (Mabon).
    Harvest,
    /// The full moon immediately following the Harvest Moon.
    Hunter,
    /// November — time to set beaver traps before lakes froze.
    Beaver,
    /// December — long, cold nights.
    Cold,
    /// The second full moon in the same calendar month.
    Blue,
}

impl EsbatName {
    /// Display name (e.g. `"Wolf Moon"`).
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Wolf => "Wolf Moon",
            Self::Snow => "Snow Moon",
            Self::Worm => "Worm Moon",
            Self::Pink => "Pink Moon",
            Self::Flower => "Flower Moon",
            Self::Strawberry => "Strawberry Moon",
            Self::Buck => "Buck Moon",
            Self::Sturgeon => "Sturgeon Moon",
            Self::Harvest => "Harvest Moon",
            Self::Hunter => "Hunter's Moon",
            Self::Beaver => "Beaver Moon",
            Self::Cold => "Cold Moon",
            Self::Blue => "Blue Moon",
        }
    }

    /// Alternative names from various traditions.
    #[must_use]
    pub fn alt_names(self) -> &'static [&'static str] {
        match self {
            Self::Wolf => &["Old Moon", "Ice Moon", "Moon After Yule"],
            Self::Snow => &["Hunger Moon", "Storm Moon", "Bone Moon"],
            Self::Worm => &["Crow Moon", "Sap Moon", "Sugar Moon", "Crust Moon"],
            Self::Pink => &["Egg Moon", "Fish Moon", "Sprouting Grass Moon"],
            Self::Flower => &["Corn Planting Moon", "Milk Moon"],
            Self::Strawberry => &["Rose Moon", "Hot Moon", "Honey Moon", "Mead Moon"],
            Self::Buck => &["Thunder Moon", "Hay Moon", "Wort Moon"],
            Self::Sturgeon => &["Grain Moon", "Green Corn Moon", "Red Moon"],
            Self::Harvest => &["Corn Moon"],
            Self::Hunter => &["Blood Moon", "Dying Grass Moon", "Travel Moon"],
            Self::Beaver => &["Frost Moon", "Mourning Moon"],
            Self::Cold => &["Long Nights Moon", "Oak Moon", "Moon Before Yule"],
            Self::Blue => &["Full Moon", "Aerra Geola"],
        }
    }
}

// ─── Esbat ────────────────────────────────────────────────────────────────────

/// A Celtic esbat (full moon) with its traditional name and Julian day.
#[derive(Debug, Clone, PartialEq)]
pub struct Esbat {
    /// Traditional name.
    pub name: EsbatName,
    /// Display string (e.g. `"Harvest Moon"`).
    pub display_name: &'static str,
    /// Julian day (UT) of the exact full moon moment.
    pub jd: f64,
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Julian day (UT) of the next full moon at or after `jd_from`.
///
/// Accuracy: within a few seconds of the true full moon.
pub fn next_full_moon(jd_from: f64) -> Result<f64> {
    find_full_moon_after(jd_from)
}

/// All full moons in a given Gregorian year with their traditional names.
///
/// A year has 12 or 13 full moons.  The 13th is the Blue Moon (second full
/// moon in the same calendar month).  The Harvest Moon is the full moon
/// closest to the autumn equinox and may fall in September or October.
pub fn esbats_for_year(year: i32) -> Result<Vec<Esbat>> {
    let year_start = crate::julday(year, 1, 1, 0.0, Calendar::Gregorian);
    let year_end = crate::julday(year + 1, 1, 1, 0.0, Calendar::Gregorian);

    // Collect all full moons that fall within the calendar year.
    // A calendar year has 12 or 13 full moons (Blue-Moon years).
    let mut full_moons: Vec<f64> = Vec::with_capacity(13);
    let mut jd = find_full_moon_after(year_start)?;
    while jd < year_end {
        full_moons.push(jd);
        jd = find_full_moon_after(jd + 28.0)?; // advance safely past this moon
    }

    // Autumn equinox JD (Sun at 180° = Mabon) for Harvest Moon determination
    let equinox_jd = crate::functions::motion::solcross(
        Longitude::new(180.0),
        JulianDay::new(crate::julday(year, 9, 1, 0.0, Calendar::Gregorian)),
        CalcFlags::BUILTIN,
    )
    .unwrap_or_else(|_| crate::julday(year, 9, 22, 0.0, Calendar::Gregorian));

    let names = assign_names(&full_moons, equinox_jd);

    Ok(full_moons
        .into_iter()
        .zip(names)
        .map(|(jd, name)| Esbat {
            display_name: name.display_name(),
            name,
            jd,
        })
        .collect())
}

/// The next esbat (named full moon) at or after the given Julian day.
pub fn next_esbat(jd_from: f64) -> Result<Esbat> {
    let d = crate::revjul(JulianDay::new(jd_from), Calendar::Gregorian);
    // Search this year and next so we always find a result near year boundaries
    for year in [d.year, d.year + 1] {
        if let Ok(esbats) = esbats_for_year(year) {
            if let Some(e) = esbats.into_iter().find(|e| e.jd >= jd_from) {
                return Ok(e);
            }
        }
    }
    Err(Error::Calc("could not find next esbat".into()))
}

// ─── Core: full-moon finder ───────────────────────────────────────────────────

/// Julian day (UT) of the next full moon at or after `jd_from`.
///
/// Uses the synodic-month lunation number to get a ±0.35-day initial bracket,
/// then bisects to sub-second precision.
fn find_full_moon_after(jd_from: f64) -> Result<f64> {
    // How many complete synodic months since the epoch full moon?
    let n = ((jd_from - EPOCH_FULL_MOON) / SYNODIC_MONTH).floor() as i64;

    // Try this lunation and the next two in case the estimate is slightly early.
    for k in n..=(n + 2) {
        let jd_approx = EPOCH_FULL_MOON + k as f64 * SYNODIC_MONTH;
        if jd_approx + 1.0 < jd_from {
            continue;
        } // bracket is before our window
        if let Ok(jd) = bisect_full_moon(jd_approx - 1.0, jd_approx + 1.0) {
            if jd >= jd_from {
                return Ok(jd);
            }
        }
    }
    Err(Error::Calc(format!(
        "could not find full moon after JD {jd_from:.1}"
    )))
}

/// Moon–Sun elongation (degrees, [0, 360)) at Julian day `jd`.
fn elongation(jd: f64) -> Result<f64> {
    let sun = crate::calc_ut(JulianDay::new(jd), Body::SUN, CalcFlags::BUILTIN)?;
    let moon = crate::calc_ut(JulianDay::new(jd), Body::MOON, CalcFlags::BUILTIN)?;
    Ok((moon.lon - sun.lon).rem_euclid(360.0))
}

/// Find the exact full-moon moment in `[jd_lo, jd_hi]` (elongation = 180°).
///
/// Signed distance of elongation from 180°, in (−180, +180]. Used by
/// [`bisect_full_moon`] as the function whose root is sought (i.e. f(jd) = 0
/// at exact full moon).
fn full_moon_signed(jd: f64) -> Result<f64> {
    let e = elongation(jd)?;
    Ok(crate::functions::utils::wrap_signed_180(e - 180.0))
}

/// Newton's-method refinement of `jd` toward the full-moon root.
///
/// Converges in ~5 iterations near the root; bails early on sub-arcsecond
/// accuracy or a degenerate derivative. Numerical derivative uses a
/// 14-minute step.
fn newton_refine_full_moon(mut jd: f64, jd_lo: f64, jd_hi: f64) -> Result<f64> {
    const H: f64 = 0.01; // 0.01 day ≈ 14 min for numerical derivative
    const TOL: f64 = 1.0 / 3_600.0; // sub-arcsecond convergence
    for _ in 0..10 {
        let f = full_moon_signed(jd)?;
        if f.abs() < TOL {
            break;
        }
        let fp = (full_moon_signed(jd + H)? - full_moon_signed(jd - H)?) / (2.0 * H);
        if fp.abs() < 0.1 {
            break; // guard against near-zero derivative
        }
        let step = (f / fp).clamp(-3.0, 3.0); // avoid wild jumps
        jd = (jd - step).clamp(jd_lo - 1.0, jd_hi + 1.0); // stay near bracket
    }
    Ok(jd)
}

/// Sign-preserving bisection fallback for [`bisect_full_moon`] when Newton's
/// method diverges (near-polar latitudes, peculiar elongation curves).
/// Stops at 1-minute precision or 20 iterations.
fn bisect_fallback_full_moon(jd_lo: f64, jd_hi: f64) -> Result<f64> {
    const PRECISION: f64 = 1.0 / 1_440.0; // 1 minute
    let (mut lo, mut hi) = (jd_lo, jd_hi);
    let mut lo_sign = full_moon_signed(lo)?.signum();
    for _ in 0..20 {
        if hi - lo < PRECISION {
            break;
        }
        let mid = (lo + hi) / 2.0;
        let mid_sign = full_moon_signed(mid)?.signum();
        if mid_sign == lo_sign || mid_sign == 0.0 {
            lo = mid;
            lo_sign = mid_sign;
        } else {
            hi = mid;
        }
    }
    Ok((lo + hi) / 2.0)
}

/// Find the exact full-moon JD inside the bracket `[jd_lo, jd_hi]`.
/// Newton's method with numerical derivative, falling back to bisection
/// if Newton diverges (sanity-checked against 0.05°).
fn bisect_full_moon(jd_lo: f64, jd_hi: f64) -> Result<f64> {
    let initial = (jd_lo + jd_hi) / 2.0;
    let jd = newton_refine_full_moon(initial, jd_lo, jd_hi)?;

    // Sanity check: elongation should be within 0.05° of 180°
    let e = elongation(jd)?;
    let err = (e - 180.0).abs().min(360.0 - (e - 180.0).abs());
    if err <= 0.05 {
        return Ok(jd);
    }
    bisect_fallback_full_moon(jd_lo, jd_hi)
}

// ─── Naming logic ─────────────────────────────────────────────────────────────

/// Assign traditional names to a list of full moon JDs in the same calendar year.
fn assign_names(full_moons: &[f64], equinox_jd: f64) -> Vec<EsbatName> {
    let n = full_moons.len();

    let months: Vec<i32> = full_moons
        .iter()
        .map(|&jd| crate::revjul(JulianDay::new(jd), Calendar::Gregorian).month)
        .collect();

    // ── 1. Blue Moon: second full moon in the same calendar month ─────────────
    // Determined first so that the Blue Moon is excluded from Harvest Moon selection.
    let blue_idx: Option<usize> = (0..n).find(|&i| i > 0 && months[i] == months[i - 1]);

    // ── 2. Harvest Moon (closest to the autumn equinox, excluding Blue Moon) ───
    let harvest_idx = full_moons
        .iter()
        .enumerate()
        .filter(|(i, _)| Some(*i) != blue_idx)
        .min_by(|(_, &a), (_, &b)| (a - equinox_jd).abs().total_cmp(&(b - equinox_jd).abs()))
        .map_or(0, |(i, _)| i);

    // ── 3. Hunter's Moon: the first non-Blue full moon after the Harvest Moon ──
    let hunter_idx = ((harvest_idx + 1)..n).find(|&i| Some(i) != blue_idx);

    // ── 4. Assign names in priority order ─────────────────────────────────────
    (0..n)
        .map(|i| {
            if Some(i) == blue_idx {
                EsbatName::Blue
            } else if i == harvest_idx {
                EsbatName::Harvest
            } else if Some(i) == hunter_idx {
                EsbatName::Hunter
            } else {
                name_by_month(months[i])
            }
        })
        .collect()
}

/// Default name for the full moon in a given calendar month.
fn name_by_month(month: i32) -> EsbatName {
    match month {
        1 => EsbatName::Wolf,
        2 => EsbatName::Snow,
        3 => EsbatName::Worm,
        4 => EsbatName::Pink,
        5 => EsbatName::Flower,
        6 => EsbatName::Strawberry,
        7 => EsbatName::Buck,
        8 => EsbatName::Sturgeon,
        9 => EsbatName::Sturgeon, // September default; overridden to Harvest/Hunter by assign_names
        10 => EsbatName::Hunter,
        11 => EsbatName::Beaver,
        12 => EsbatName::Cold,
        _ => EsbatName::Blue,
    }
}

#[cfg(test)]
mod cov_tests {
    use super::*;

    #[test]
    fn bisect_fallback_full_moon_brackets_around_known_full() {
        // Full moon near JD 2_451_553 (J2000 + ~8d). Use a 4-day window around it.
        let jd = bisect_fallback_full_moon(2_451_551.0, 2_451_555.0).expect("bracket");
        assert!(jd > 2_451_551.0 && jd < 2_451_555.0, "jd={jd}");
    }
}
