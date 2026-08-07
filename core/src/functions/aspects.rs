//! Aspect matching and antiscion helpers (port of swhaspect.c).

// ─── Angle helpers ────────────────────────────────────────────────────────────

use crate::{diff_deg, diff_deg_signed, norm_deg};

#[inline]
fn norm360(d: f64) -> f64 {
    norm_deg(d)
}

/// Aspect angles in degrees.
#[allow(dead_code)]
pub mod aspect_angles {
    /// Conjunction (0°).
    pub const CONJUNCTION: f64 = 0.0;
    /// Squisextile (15°).
    pub const SQUISEXTILE: f64 = 15.0;
    /// Seminovile (20°).
    pub const SEMINOVILE: f64 = 20.0;
    /// Squisquare (22.5°).
    pub const SQUISQUARE: f64 = 22.5;
    /// Undecile (360°/11 ≈ 32.7°).
    pub const UNDECILE: f64 = 360.0 / 11.0;
    /// Semisextile (30°).
    pub const SEMISEXTILE: f64 = 30.0;
    /// Semiquintile (36°).
    pub const SEMIQUINTILE: f64 = 36.0;
    /// Novile (40°).
    pub const NOVILE: f64 = 40.0;
    /// Semisquare (45°).
    pub const SEMISQUARE: f64 = 45.0;
    /// Septile (360°/7 ≈ 51.4°).
    pub const SEPTILE: f64 = 360.0 / 7.0;
    /// Sextile (60°).
    pub const SEXTILE: f64 = 60.0;
    /// Biundecile (2 × 360°/11 ≈ 65.5°).
    pub const BIUNDECILE: f64 = 360.0 / 11.0 * 2.0;
    /// Quintile (72°).
    pub const QUINTILE: f64 = 72.0;
    /// Binovile (80°).
    pub const BINOVILE: f64 = 80.0;
    /// Square (90°).
    pub const SQUARE: f64 = 90.0;
    /// Triundecile (3 × 360°/11 ≈ 98.2°).
    pub const TRIUNDECILE: f64 = 360.0 / 11.0 * 3.0;
    /// Biseptile (2 × 360°/7 ≈ 102.9°).
    pub const BISEPTILE: f64 = 360.0 / 7.0 * 2.0;
    /// Trine (120°).
    pub const TRINE: f64 = 120.0;
    /// Quadundecile (4 × 360°/11 ≈ 130.9°).
    pub const QUADUNDECILE: f64 = 360.0 / 11.0 * 4.0;
    /// Sesquisquare (135°).
    pub const SESQUISQUARE: f64 = 135.0;
    /// Biquintile (144°).
    pub const BIQUINTILE: f64 = 144.0;
    /// Quincunx (150°).
    pub const QUINCUNX: f64 = 150.0;
    /// Inconjunct (150°).
    pub const INCONJUNCT: f64 = 150.0;
    /// Triseptile (3 × 360°/7 ≈ 154.3°).
    pub const TRISEPTILE: f64 = 360.0 / 7.0 * 3.0;
    /// Quatronovile (160°).
    pub const QUATRONOVILE: f64 = 160.0;
    /// Quinundecile (5 × 360°/11 ≈ 163.6°).
    pub const QUINUNDECILE: f64 = 360.0 / 11.0 * 5.0;
    /// Opposition (180°).
    pub const OPPOSITION: f64 = 180.0;
}

/// Sign numbers.
#[allow(dead_code)]
pub mod signs {
    /// Aries (sign 0).
    pub const ARIES: i32 = 0;
    /// Mesha — Aries (sign 0).
    pub const MESHA: i32 = 0;
    /// Taurus (sign 1).
    pub const TAURUS: i32 = 1;
    /// Vrishaba — Taurus (sign 1).
    pub const VRISHABA: i32 = 1;
    /// Gemini (sign 2).
    pub const GEMINI: i32 = 2;
    /// Mithuna — Gemini (sign 2).
    pub const MITHUNA: i32 = 2;
    /// Cancer (sign 3).
    pub const CANCER: i32 = 3;
    /// Kataka — Cancer (sign 3).
    pub const KATAKA: i32 = 3;
    /// Leo (sign 4).
    pub const LEO: i32 = 4;
    /// Simha — Leo (sign 4).
    pub const SIMHA: i32 = 4;
    /// Virgo (sign 5).
    pub const VIRGO: i32 = 5;
    /// Kanya — Virgo (sign 5).
    pub const KANYA: i32 = 5;
    /// Libra (sign 6).
    pub const LIBRA: i32 = 6;
    /// Thula — Libra (sign 6).
    pub const THULA: i32 = 6;
    /// Scorpio (sign 7).
    pub const SCORPIO: i32 = 7;
    /// Vrishika — Scorpio (sign 7).
    pub const VRISHIKA: i32 = 7;
    /// Sagittarius (sign 8).
    pub const SAGITTARIUS: i32 = 8;
    /// Dhanus — Sagittarius (sign 8).
    pub const DHANUS: i32 = 8;
    /// Capricorn (sign 9).
    pub const CAPRICORN: i32 = 9;
    /// Makara — Capricorn (sign 9).
    pub const MAKARA: i32 = 9;
    /// Aquarius (sign 10).
    pub const AQUARIUS: i32 = 10;
    /// Kumbha — Aquarius (sign 10).
    pub const KUMBHA: i32 = 10;
    /// Pisces (sign 11).
    pub const PISCES: i32 = 11;
    /// Meena — Pisces (sign 11).
    pub const MEENA: i32 = 11;
}

/// Nakshatra numbers.
#[allow(dead_code)]
pub mod nakshatras {
    /// Aswini (nakshatra 0).
    pub const ASWINI: i32 = 0;
    /// Bharani (nakshatra 1).
    pub const BHARANI: i32 = 1;
    /// Krithika (nakshatra 2).
    pub const KRITHIKA: i32 = 2;
    /// Rohini (nakshatra 3).
    pub const ROHINI: i32 = 3;
    /// Mrigasira (nakshatra 4).
    pub const MRIGASIRA: i32 = 4;
    /// Aridra (nakshatra 5).
    pub const ARIDRA: i32 = 5;
    /// Punarvasu (nakshatra 6).
    pub const PUNARVASU: i32 = 6;
    /// Pushyami (nakshatra 7).
    pub const PUSHYAMI: i32 = 7;
    /// Aslesha (nakshatra 8).
    pub const ASLESHA: i32 = 8;
    /// Makha (nakshatra 9).
    pub const MAKHA: i32 = 9;
    /// Pubba (nakshatra 10).
    pub const PUBBA: i32 = 10;
    /// Uttara (nakshatra 11).
    pub const UTTARA: i32 = 11;
    /// Hasta (nakshatra 12).
    pub const HASTA: i32 = 12;
    /// Chitta (nakshatra 13).
    pub const CHITTA: i32 = 13;
    /// Swathi (nakshatra 14).
    pub const SWATHI: i32 = 14;
    /// Vishaka (nakshatra 15).
    pub const VISHAKA: i32 = 15;
    /// Anuradha (nakshatra 16).
    pub const ANURADHA: i32 = 16;
    /// Jyesta (nakshatra 17).
    pub const JYESTA: i32 = 17;
    /// Moola (nakshatra 18).
    pub const MOOLA: i32 = 18;
    /// Poorvashada (nakshatra 19).
    pub const POORVASHADA: i32 = 19;
    /// Uttarashada (nakshatra 20).
    pub const UTTARASHADA: i32 = 20;
    /// Sravana (nakshatra 21).
    pub const SRAVANA: i32 = 21;
    /// Dhanishta (nakshatra 22).
    pub const DHANISHTA: i32 = 22;
    /// Satabisha (nakshatra 23).
    pub const SATABISHA: i32 = 23;
    /// Poorvabhadra (nakshatra 24).
    pub const POORVABHADRA: i32 = 24;
    /// Uttarabhadra (nakshatra 25).
    pub const UTTARABHADRA: i32 = 25;
    /// Revathi (nakshatra 26).
    pub const REVATHI: i32 = 26;
}

// ═══════════════════════════════════════════════════════════════════════════
// Aspect matching & antiscion
// ═══════════════════════════════════════════════════════════════════════════

/// Result of an aspect match check.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AspectMatch {
    /// Difference between aspect and objects distance (pos1 + asp + diff = pos2).
    pub diff: f64,
    /// Differential speed of the aspect.  Negative → applying.
    pub speed: f64,
    /// Difference expressed in orb units.
    pub factor: f64,
    /// `true` if within orb.
    pub matched: bool,
}

/// Shared aspect-match core (DP-7/DUP-6). `select_orb(speed)` returns
/// the orb given the differential speed — constant for `match_aspect`,
/// speed-dependent for `match_aspect3`. Arithmetic + op order are
/// byte-identical to the four former hand-written copies.
fn match_core(
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    select_orb: impl FnOnce(f64) -> f64,
) -> AspectMatch {
    let aspect = norm360(aspect);
    let diff0 = diff_deg(pos1, pos0); // unsigned [0,360) = pos1-pos0
    if diff0 == aspect {
        return AspectMatch {
            diff: 0.0,
            speed: 0.0,
            factor: 0.0,
            matched: true,
        };
    }
    let diff = diff0 - aspect;
    let speed = differential_speed(diff, speed0, speed1);
    let orb = select_orb(speed).abs();
    let factor = diff / orb;
    let matched = aspect - orb <= diff0 && diff0 <= aspect + orb;
    AspectMatch {
        diff,
        speed,
        factor,
        matched,
    }
}

fn differential_speed(diff: f64, speed0: f64, speed1: f64) -> f64 {
    match diff.partial_cmp(&0.0) {
        Some(std::cmp::Ordering::Greater) => speed1 - speed0,
        Some(std::cmp::Ordering::Less) => speed0 - speed1,
        _ => 0.0,
    }
}

/// Pick the better ±aspect candidate: smaller |diff|, then (tie) the
/// more-applying (smaller `speed`). Byte-identical to the former
/// duplicated `*2`/`*4` tail.
fn pick_closer(a0: AspectMatch, a1: AspectMatch) -> AspectMatch {
    if a1.diff.abs() < a0.diff.abs() {
        a1
    } else if a0.diff.abs() < a1.diff.abs() {
        a0
    } else if a1.speed < a0.speed {
        a1
    } else {
        a0
    }
}

/// Check whether two longitudes make a given aspect within the given orb.
/// `aspect` must be in `[0, 360)`.
#[must_use]
pub fn match_aspect(
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    orb: f64,
) -> AspectMatch {
    // former: `let orb = orb.abs()` then constant orb — match_core
    // applies `.abs()` to the closure result, so this is identical.
    match_core(pos0, speed0, pos1, speed1, aspect, |_| orb)
}

/// Like `match_aspect` but `aspect` in `[0, 180]` — tests both ±aspect.
#[must_use]
pub fn match_aspect2(
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    orb: f64,
) -> AspectMatch {
    let asp = diff_deg_signed(0.0, aspect).abs(); // normalise to [0,180]
    let a0 = match_aspect(pos0, speed0, pos1, speed1, asp, orb);
    let a1 = match_aspect(pos0, speed0, pos1, speed1, -asp, orb);
    pick_closer(a0, a1)
}

/// Like `match_aspect` with separate applying / separating / stationary orbs.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn match_aspect3(
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    app_orb: f64,
    sep_orb: f64,
    def_orb: f64,
) -> AspectMatch {
    // former: orb chosen from speed sign, each branch `.abs()`-ed —
    // match_core `.abs()`-es the closure result, so identical.
    match_core(pos0, speed0, pos1, speed1, aspect, |speed| {
        if speed < 0.0 {
            app_orb
        } else if speed > 0.0 {
            sep_orb
        } else {
            def_orb
        }
    })
}

/// Like `match_aspect2` with separate applying / separating / stationary orbs.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn match_aspect4(
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    app_orb: f64,
    sep_orb: f64,
    def_orb: f64,
) -> AspectMatch {
    let asp = diff_deg_signed(0.0, aspect).abs();
    let a0 = match_aspect3(pos0, speed0, pos1, speed1, asp, app_orb, sep_orb, def_orb);
    let a1 = match_aspect3(pos0, speed0, pos1, speed1, -asp, app_orb, sep_orb, def_orb);
    pick_closer(a0, a1)
}

/// Antiscion result.
#[derive(Debug, Clone, Copy)]
pub struct Antiscion {
    /// Antiscion position [lon, lat, dist, speed_lon, speed_lat, speed_dist].
    pub antiscion: [f64; 6],
    /// Contrantiscion position.
    pub contrantiscion: [f64; 6],
}

/// Compute antiscion and contrantiscion of a body around a given axis.
/// `axis` is the degree of the reflection axis (e.g. 90.0 for 0°Cancer/Capricorn).
#[must_use]
pub fn antiscion(pos: [f64; 6], axis: f64) -> Antiscion {
    // Reflect pos around axis: antiscion = 2*axis - pos (mod 360)
    let anti_lon = norm360(2.0 * axis - pos[0]);
    let anti = [anti_lon, pos[1], pos[2], -pos[3], pos[4], pos[5]];
    let cont = [
        norm360(anti_lon + 180.0),
        -pos[1],
        pos[2],
        -pos[3],
        -pos[4],
        pos[5],
    ];
    Antiscion {
        antiscion: anti,
        contrantiscion: cont,
    }
}

// ── AspectOrbs builder ────────────────────────────────────────────────────────

/// Builder for fine-grained aspect matching with separate applying/separating orbs.
///
/// # Example
/// ```
/// # use celestial_core::AspectOrbs;
/// let m = AspectOrbs::new(1.5, 1.0)
///     .def_orb(2.0)
///     .check(120.0, 0.5, 0.0, -0.4, 120.0);  // trine
/// assert!(m.matched);
/// ```
#[derive(Debug, Clone)]
pub struct AspectOrbs {
    app_orb: f64,
    sep_orb: f64,
    def_orb: f64,
}

impl AspectOrbs {
    /// Create a new builder with applying and separating orbs in degrees.
    pub fn new(app_orb: f64, sep_orb: f64) -> Self {
        Self {
            app_orb,
            sep_orb,
            def_orb: app_orb.max(sep_orb),
        }
    }

    /// Set an additional default orb (used in `match_aspect4`).
    ///
    /// Defaults to `max(app_orb, sep_orb)` when not set.
    pub fn def_orb(mut self, orb: f64) -> Self {
        self.def_orb = orb;
        self
    }

    /// Test if two bodies are in aspect, using all three orbs.
    ///
    /// `pos0`/`speed0` — longitude and daily speed of body 1.
    /// `pos1`/`speed1` — longitude and daily speed of body 2.
    /// `aspect` — target aspect angle (0°, 60°, 90°, 120°, 180°…).
    #[must_use]
    pub fn check(
        &self,
        pos0: f64,
        speed0: f64,
        pos1: f64,
        speed1: f64,
        aspect: f64,
    ) -> AspectMatch {
        match_aspect4(
            pos0,
            speed0,
            pos1,
            speed1,
            aspect,
            self.app_orb,
            self.sep_orb,
            self.def_orb,
        )
    }

    /// Test using only applying and separating orbs (no default orb).
    #[must_use]
    pub fn check_simple(
        &self,
        pos0: f64,
        speed0: f64,
        pos1: f64,
        speed1: f64,
        aspect: f64,
    ) -> AspectMatch {
        match_aspect3(
            pos0,
            speed0,
            pos1,
            speed1,
            aspect,
            self.app_orb,
            self.sep_orb,
            self.def_orb,
        )
    }
}

#[cfg(test)]
mod mutation_tests {
    use super::*;

    fn aspect(diff: f64, speed: f64, factor: f64, matched: bool) -> AspectMatch {
        AspectMatch {
            diff,
            speed,
            factor,
            matched,
        }
    }

    #[test]
    fn derived_aspect_angles_are_exact() {
        use aspect_angles::*;
        assert_eq!(UNDECILE, 32.727_272_727_272_73);
        assert_eq!(SEPTILE, 51.428_571_428_571_43);
        assert_eq!(BIUNDECILE, 65.454_545_454_545_45);
        assert_eq!(TRIUNDECILE, 98.181_818_181_818_19);
        assert_eq!(BISEPTILE, 102.857_142_857_142_86);
        assert_eq!(QUADUNDECILE, 130.909_090_909_090_9);
        assert_eq!(TRISEPTILE, 154.285_714_285_714_28);
        assert_eq!(QUINUNDECILE, 163.636_363_636_363_63);
    }

    #[test]
    fn aspect_match_paths_are_exact() {
        assert_eq!(
            match_aspect(10.0, 1.0, 70.0, 0.5, 60.0, 2.0),
            aspect(0.0, 0.0, 0.0, true)
        );
        assert_eq!(
            match_aspect(10.0, 1.0, 75.0, 0.5, 60.0, 2.0),
            aspect(5.0, -0.5, 2.5, false)
        );
        assert_eq!(
            match_aspect(10.0, 1.0, 68.0, 0.5, 60.0, 4.0),
            aspect(-2.0, 0.5, -0.5, true)
        );
    }

    #[test]
    fn orb_boundaries_are_inclusive() {
        assert!(match_aspect(10.0, 1.0, 68.0, 0.0, 60.0, 2.0).matched);
        assert!(match_aspect(10.0, 1.0, 72.0, 0.0, 60.0, 2.0).matched);
        assert!(!match_aspect(10.0, 1.0, 67.999, 0.0, 60.0, 2.0).matched);
        assert!(!match_aspect(10.0, 1.0, 72.001, 0.0, 60.0, 2.0).matched);
    }

    #[test]
    fn closer_candidate_order_is_exact() {
        let base = aspect(2.0, 0.5, 1.0, true);
        let closer = aspect(1.0, 2.0, 3.0, false);
        assert_eq!(pick_closer(base, closer), closer);
        assert_eq!(pick_closer(closer, base), closer);

        let applying = aspect(-2.0, -0.5, 4.0, false);
        assert_eq!(pick_closer(base, applying), applying);
        let tied = aspect(-2.0, 0.5, 9.0, false);
        assert_eq!(pick_closer(base, tied), base);
    }

    #[test]
    fn signed_aspect_candidates_are_exact() {
        assert_eq!(
            match_aspect2(10.0, 1.0, 308.0, 0.5, 60.0, 3.0),
            aspect(-2.0, 0.5, -0.666_666_666_666_666_6, true)
        );
        assert_eq!(
            match_aspect2(10.0, 1.0, 190.0, 0.5, 180.0, 3.0),
            aspect(0.0, 0.0, 0.0, true)
        );
        assert_eq!(
            match_aspect4(10.0, 1.0, 308.0, 0.5, 60.0, 4.0, 3.0, 2.0),
            aspect(-2.0, 0.5, -0.666_666_666_666_666_6, true)
        );
    }

    #[test]
    fn speed_selects_the_orb_exactly() {
        assert_eq!(
            match_aspect3(10.0, 1.0, 72.0, 0.5, 60.0, 4.0, 3.0, 2.0),
            aspect(2.0, -0.5, 0.5, true)
        );
        assert_eq!(
            match_aspect3(10.0, 0.5, 72.0, 1.0, 60.0, 4.0, 3.0, 2.0),
            aspect(2.0, 0.5, 0.666_666_666_666_666_6, true)
        );
        assert_eq!(
            match_aspect3(10.0, 1.0, 72.0, 1.0, 60.0, 4.0, 3.0, 2.0),
            aspect(2.0, 0.0, 1.0, true)
        );
    }

    #[test]
    fn antiscion_signs_are_exact() {
        let result = antiscion([123.0, -4.0, 2.0, 0.5, -0.25, 0.01], 90.0);
        assert_eq!(result.antiscion, [57.0, -4.0, 2.0, -0.5, -0.25, 0.01]);
        assert_eq!(result.contrantiscion, [237.0, 4.0, 2.0, -0.5, 0.25, 0.01]);
    }

    #[test]
    fn aspect_orbs_builder_preserves_configuration() {
        let orbs = AspectOrbs::new(4.0, 3.0).def_orb(2.0);
        assert_eq!(
            orbs.check(10.0, 1.0, 72.0, 1.0, 60.0),
            aspect(2.0, 0.0, 1.0, true)
        );
        assert_eq!(
            orbs.check_simple(10.0, 1.0, 72.0, 0.5, 60.0),
            aspect(2.0, -0.5, 0.5, true)
        );
    }
}
