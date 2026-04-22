//! Aspect matching and antiscion helpers (port of swhaspect.c).

// ─── Angle helpers ────────────────────────────────────────────────────────────

use crate::{diff_deg, diff_deg_signed, norm_deg};

#[inline]
fn norm360(d: f64) -> f64 {
    norm_deg(d)
}

#[allow(dead_code)]
pub mod aspect_angles {
    pub const CONJUNCTION: f64 = 0.0;
    pub const SQUISEXTILE: f64 = 15.0;
    pub const SEMINOVILE: f64 = 20.0;
    pub const SQUISQUARE: f64 = 22.5;
    pub const UNDECILE: f64 = 360.0 / 11.0;
    pub const SEMISEXTILE: f64 = 30.0;
    pub const SEMIQUINTILE: f64 = 36.0;
    pub const NOVILE: f64 = 40.0;
    pub const SEMISQUARE: f64 = 45.0;
    pub const SEPTILE: f64 = 360.0 / 7.0;
    pub const SEXTILE: f64 = 60.0;
    pub const BIUNDECILE: f64 = 360.0 / 11.0 * 2.0;
    pub const QUINTILE: f64 = 72.0;
    pub const BINOVILE: f64 = 80.0;
    pub const SQUARE: f64 = 90.0;
    pub const TRIUNDECILE: f64 = 360.0 / 11.0 * 3.0;
    pub const BISEPTILE: f64 = 360.0 / 7.0 * 2.0;
    pub const TRINE: f64 = 120.0;
    pub const QUADUNDECILE: f64 = 360.0 / 11.0 * 4.0;
    pub const SESQUISQUARE: f64 = 135.0;
    pub const BIQUINTILE: f64 = 144.0;
    pub const QUINCUNX: f64 = 150.0;
    pub const INCONJUNCT: f64 = 150.0;
    pub const TRISEPTILE: f64 = 360.0 / 7.0 * 3.0;
    pub const QUATRONOVILE: f64 = 160.0;
    pub const QUINUNDECILE: f64 = 360.0 / 11.0 * 5.0;
    pub const OPPOSITION: f64 = 180.0;
}

/// Sign numbers.
#[allow(dead_code)]
pub mod signs {
    pub const ARIES: i32 = 0;
    pub const MESHA: i32 = 0;
    pub const TAURUS: i32 = 1;
    pub const VRISHABA: i32 = 1;
    pub const GEMINI: i32 = 2;
    pub const MITHUNA: i32 = 2;
    pub const CANCER: i32 = 3;
    pub const KATAKA: i32 = 3;
    pub const LEO: i32 = 4;
    pub const SIMHA: i32 = 4;
    pub const VIRGO: i32 = 5;
    pub const KANYA: i32 = 5;
    pub const LIBRA: i32 = 6;
    pub const THULA: i32 = 6;
    pub const SCORPIO: i32 = 7;
    pub const VRISHIKA: i32 = 7;
    pub const SAGITTARIUS: i32 = 8;
    pub const DHANUS: i32 = 8;
    pub const CAPRICORN: i32 = 9;
    pub const MAKARA: i32 = 9;
    pub const AQUARIUS: i32 = 10;
    pub const KUMBHA: i32 = 10;
    pub const PISCES: i32 = 11;
    pub const MEENA: i32 = 11;
}

/// Nakshatra numbers.
#[allow(dead_code)]
pub mod nakshatras {
    pub const ASWINI: i32 = 0;
    pub const BHARANI: i32 = 1;
    pub const KRITHIKA: i32 = 2;
    pub const ROHINI: i32 = 3;
    pub const MRIGASIRA: i32 = 4;
    pub const ARIDRA: i32 = 5;
    pub const PUNARVASU: i32 = 6;
    pub const PUSHYAMI: i32 = 7;
    pub const ASLESHA: i32 = 8;
    pub const MAKHA: i32 = 9;
    pub const PUBBA: i32 = 10;
    pub const UTTARA: i32 = 11;
    pub const HASTA: i32 = 12;
    pub const CHITTA: i32 = 13;
    pub const SWATHI: i32 = 14;
    pub const VISHAKA: i32 = 15;
    pub const ANURADHA: i32 = 16;
    pub const JYESTA: i32 = 17;
    pub const MOOLA: i32 = 18;
    pub const POORVASHADA: i32 = 19;
    pub const UTTARASHADA: i32 = 20;
    pub const SRAVANA: i32 = 21;
    pub const DHANISHTA: i32 = 22;
    pub const SATABISHA: i32 = 23;
    pub const POORVABHADRA: i32 = 24;
    pub const UTTARABHADRA: i32 = 25;
    pub const REVATHI: i32 = 26;
}

// ═══════════════════════════════════════════════════════════════════════════
// Aspect matching & antiscion
// ═══════════════════════════════════════════════════════════════════════════

/// Result of an aspect match check.
#[derive(Debug, Clone, Copy)]
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

/// Check whether two longitudes make a given aspect within the given orb.
/// `aspect` must be in `[0, 360)`.
pub fn match_aspect(
    pos0: f64,
    speed0: f64,
    pos1: f64,
    speed1: f64,
    aspect: f64,
    orb: f64,
) -> AspectMatch {
    let aspect = norm360(aspect);
    let orb = orb.abs();
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
    let speed = if diff > 0.0 {
        speed1 - speed0
    } else {
        speed0 - speed1
    };
    let factor = diff / orb;
    let matched = aspect - orb <= diff0 && diff0 <= aspect + orb;
    AspectMatch {
        diff,
        speed,
        factor,
        matched,
    }
}

/// Like `match_aspect` but `aspect` in `[0, 180]` — tests both ±aspect.
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
    if asp == 0.0 || asp == 180.0 {
        return a0;
    }
    let a1 = match_aspect(pos0, speed0, pos1, speed1, -asp, orb);
    // Pick the one closer to exact
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

/// Like `match_aspect` with separate applying / separating / stationary orbs.
#[allow(clippy::too_many_arguments)]
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
    let aspect = norm360(aspect);
    let diff0 = diff_deg(pos1, pos0);
    if diff0 == aspect {
        return AspectMatch {
            diff: 0.0,
            speed: 0.0,
            factor: 0.0,
            matched: true,
        };
    }
    let diff = diff0 - aspect;
    let speed = if diff > 0.0 {
        speed1 - speed0
    } else {
        speed0 - speed1
    };
    let orb = if speed < 0.0 {
        app_orb.abs()
    } else if speed > 0.0 {
        sep_orb.abs()
    } else {
        def_orb.abs()
    };
    let factor = diff / orb;
    let matched = aspect - orb <= diff0 && diff0 <= aspect + orb;
    AspectMatch {
        diff,
        speed,
        factor,
        matched,
    }
}

/// Like `match_aspect2` with separate applying / separating / stationary orbs.
#[allow(clippy::too_many_arguments)]
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
    if asp == 0.0 || asp == 180.0 {
        return a0;
    }
    let a1 = match_aspect3(pos0, speed0, pos1, speed1, -asp, app_orb, sep_orb, def_orb);
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
/// Replaces `match_aspect3` (separate app/sep orbs) and `match_aspect4`
/// (separate app/sep/def orbs) with a fluent API.
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
