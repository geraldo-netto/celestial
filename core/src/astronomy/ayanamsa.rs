//! Ayanamsa — the offset between tropical and sidereal zodiac.
//!
//! An ayanamsa is the arc between the vernal equinox and the fixed sidereal
//! reference point used by a particular sidereal zodiac system.
//!
//! The value returned is subtracted from tropical longitude to obtain sidereal
//! longitude:  λ_sid = λ_trop − ayanamsa.

use crate::astronomy::constants::julian_centuries;

/// Sidereal mode constants, matching the Swiss Ephemeris `SIDM_*` values.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SidMode {
    FaganBradley = 0,
    Lahiri = 1,
    Deluce = 2,
    Raman = 3,
    Ushashashi = 4,
    Krishnamurti = 5,
    DjwhalKhul = 6,
    Yukteshwar = 7,
    JnBhasin = 8,
    BabylKugler1 = 9,
    BabylKugler2 = 10,
    BabylKugler3 = 11,
    BabylHuber = 12,
    BabylEtpsc = 13,
    Aldebaran15Tau = 14,
    Hipparchos = 15,
    Sassanian = 16,
    GalactCtr0Sag = 17,
    J2000 = 18,
    J1900 = 19,
    B1950 = 20,
    SuryaSiddhanta = 21,
    SuryaSiddhantaMsun = 22,
    Aryabhata = 23,
    AryabhataMsun = 24,
    SsRevati = 25,
    SsCitra = 26,
    TrueCitra = 27,
    TrueRevati = 28,
    TruePushya = 29,
    GalacticCtrBrand = 30,
    GalacticEqMidMula = 31,
    SkydramMula = 32,
    TrueMula = 33,
    GalacticCtrOHara = 34,
    Galactic0Aries = 35,
    User = 255,
}

/// Lookup table for [`SidMode`]: raw code, variant, and display name.
///
/// A single source of truth for both [`SidMode::from_i32`] and [`SidMode::name`].
/// Adding a new mode requires exactly one new row here.
const SID_MODE_TABLE: &[(i32, SidMode, &str)] = &[
    (0, SidMode::FaganBradley, "Fagan-Bradley"),
    (1, SidMode::Lahiri, "Lahiri"),
    (2, SidMode::Deluce, "De Luce"),
    (3, SidMode::Raman, "Raman"),
    (4, SidMode::Ushashashi, "Usha-Shashi"),
    (5, SidMode::Krishnamurti, "Krishnamurti"),
    (6, SidMode::DjwhalKhul, "Djwhal Khul"),
    (7, SidMode::Yukteshwar, "Yukteshwar"),
    (8, SidMode::JnBhasin, "J.N.Bhasin"),
    (9, SidMode::BabylKugler1, "Babylonian/Kugler 1"),
    (10, SidMode::BabylKugler2, "Babylonian/Kugler 2"),
    (11, SidMode::BabylKugler3, "Babylonian/Kugler 3"),
    (12, SidMode::BabylHuber, "Babylonian/Huber"),
    (13, SidMode::BabylEtpsc, "Babylonian/ETPSC"),
    (14, SidMode::Aldebaran15Tau, "Aldebaran at 15 Tau"),
    (15, SidMode::Hipparchos, "Hipparchos"),
    (16, SidMode::Sassanian, "Sassanian"),
    (17, SidMode::GalactCtr0Sag, "Galactic Center at 0 Sag"),
    (18, SidMode::J2000, "J2000"),
    (19, SidMode::J1900, "J1900"),
    (20, SidMode::B1950, "B1950"),
    (21, SidMode::SuryaSiddhanta, "Surya Siddhanta"),
    (22, SidMode::SuryaSiddhantaMsun, "Surya Siddhanta, mean Sun"),
    (23, SidMode::Aryabhata, "Aryabhata"),
    (24, SidMode::AryabhataMsun, "Aryabhata, mean Sun"),
    (25, SidMode::SsRevati, "SS, Revati/zeta Psc"),
    (26, SidMode::SsCitra, "SS, Citra/Spica"),
    (27, SidMode::TrueCitra, "True Citra"),
    (28, SidMode::TrueRevati, "True Revati"),
    (29, SidMode::TruePushya, "True Pushya"),
    (30, SidMode::GalacticCtrBrand, "Galactic Center (Brand)"),
    (31, SidMode::GalacticEqMidMula, "Galactic Equator mid-Mula"),
    (32, SidMode::SkydramMula, "Skydram (Mula)"),
    (33, SidMode::TrueMula, "True Mula"),
    (34, SidMode::GalacticCtrOHara, "Galactic Center (O'Hara)"),
    (
        35,
        SidMode::Galactic0Aries,
        "Galactic Equator IAU (0 Aries)",
    ),
];

impl SidMode {
    /// Parse a raw i32 into a `SidMode`, returning `None` for unknown values.
    pub fn from_i32(n: i32) -> Option<Self> {
        SID_MODE_TABLE
            .iter()
            .find(|(code, _, _)| *code == n)
            .map(|(_, m, _)| *m)
    }

    /// Display name for the ayanamsa.
    #[must_use]
    pub fn name(self) -> &'static str {
        // User-defined is the only variant not in the table
        if matches!(self, Self::User) {
            return "User-defined";
        }
        SID_MODE_TABLE
            .iter()
            .find(|(_, m, _)| *m == self)
            .map_or("Unknown", |(_, _, n)| *n)
    }
}

// ─── Reference epoch offsets (degrees at J2000.0) ─────────────────────────────
//
// Each entry is (mode, offset_at_j2000_deg).
// The ayanamsa grows with time due to precession (~50.3"/year ≈ 0.01397°/year).
// ayanamsa(jde) ≈ offset_j2000 + precession_rate * T
// where T = Julian centuries from J2000.0.
//
// More precisely: ayanamsa = offset_j2000 + (precession * T + second-order terms)
//
// Source: Swiss Ephemeris documentation + Meeus Chapter 21.

const PRECESSION_RATE: f64 = 1.396_971_278; // degrees per Julian century

/// Ayanamsa reference data: (mode, value_at_J2000 in degrees).
/// Value = precession amount subtracted from tropical to give sidereal.
static AYANAMSA_J2000: &[(i32, f64)] = &[
    (0, 24.740_920),  // Fagan-Bradley       (Spica at 29°06' Virgo)
    (1, 23.853_570),  // Lahiri              (Citrā/Spica on cusp 0° Libra)
    (2, 29.659_500),  // De Luce             (0° Aries at 221 CE)
    (3, 22.460_148),  // Raman               (sundry tradition)
    (4, 20.916_667),  // Usha-Shashi         (Moon as reference)
    (5, 23.785_278),  // Krishnamurti (KP)  = 23°47'07" per K.S. Krishnamurti
    (6, 23.333_333),  // Djwhal Khul
    (7, 22.460_148),  // Yukteshwar          (same as Raman tradition)
    (8, 22.460_148),  // J.N.Bhasin
    (9, 4.060_000),   // Babylonian/Kugler 1
    (10, 5.560_000),  // Babylonian/Kugler 2
    (11, 9.280_000),  // Babylonian/Kugler 3
    (12, 4.280_000),  // Babylonian/Huber
    (13, 4.990_000),  // Babylonian/ETPSC
    (14, 15.000_000), // Aldebaran at 15 Tau (Aldebaran = 15° Taurus)
    (15, 9.250_000),  // Hipparchos
    (16, 19.775_000), // Sassanian
    (17, 0.000_000),  // Galactic Center = 0 Sag (self-referential)
    (18, 0.000_000),  // J2000 (no ayanamsa)
    (19, 0.833_333),  // J1900 (1° precession over 71.6 years)
    (20, 0.361_667),  // B1950
    (21, 23.970_000), // Surya Siddhanta     (Mesha at 0°, traditional)
    (22, 23.970_000), // Surya Siddhanta mean
    (23, 23.966_667), // Aryabhata
    (24, 23.966_667), // Aryabhata mean
    (25, 29.720_000), // SS Revati/zeta Psc
    (26, 23.830_000), // SS Citra
    (27, 23.800_000), // True Citra          (Spica at 0° Libra exactly)
    (28, 29.700_000), // True Revati
    (29, 25.470_000), // True Pushya
    (30, 2.110_000),  // Galactic Center (Brand)
    (31, 0.000_000),  // Galactic Equator mid-Mula
    (32, 6.666_667),  // Skydram Mula
    (33, 6.666_667),  // True Mula
    (34, 2.110_000),  // Galactic Center (O'Hara)
    (35, 0.000_000),  // Galactic Equator IAU
];

/// Compute the ayanamsa in degrees for a given Julian Ephemeris Day (TT).
///
/// Returns the tropical–sidereal offset in degrees; subtract from tropical
/// longitude to obtain sidereal longitude.
#[must_use]
pub fn ayanamsa(jde: f64, mode: SidMode) -> f64 {
    if mode == SidMode::User {
        return 0.0; // user must supply their own offset externally
    }
    let t = julian_centuries(jde);
    let base = ayanamsa_j2000(mode);
    // Linear precession: ayanamsa grows with time (~50.3"/year, positive).
    // (The previous (−PRECESSION_RATE) had the sign flipped — sent the
    // ayanamsa backwards in time, giving 25.25° at 1900 instead of the
    // tabulated 22.47° per Indian Ephemeris and Nautical Almanac.)
    PRECESSION_RATE.mul_add(t, base + precession_correction(t))
}

/// Look up the J2000 base value for a given mode.
fn ayanamsa_j2000(mode: SidMode) -> f64 {
    let id = mode as i32;
    AYANAMSA_J2000
        .iter()
        .find(|&&(m, _)| m == id)
        .map_or(0.0, |&(_, v)| v)
}

/// Second-order precession correction (degrees) — Lieske et al. 1977.
///
/// This corrects for the non-linearity of the precession over long time spans.
fn precession_correction(t: f64) -> f64 {
    // Accumulated precession (Lieske formula, degrees)
    // ψ_A = 5029.097" * t + 1.558" * t² + ...
    // We account for the second-order term here
    0.000_214 * t * t // ~0.78"/century² converted to degrees
}

/// Get ayanamsa name string for a mode integer.
#[must_use]
pub fn ayanamsa_name(mode_id: i32) -> &'static str {
    SidMode::from_i32(mode_id).map_or("Unknown", SidMode::name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lahiri_j2000() {
        // Lahiri ayanamsa at J2000 ≈ 23.853°
        let ay = ayanamsa(2_451_545.0, SidMode::Lahiri);
        assert!((ay - 23.853).abs() < 0.1, "Lahiri at J2000 = {ay}");
    }

    #[test]
    fn fagan_bradley_j2000() {
        let ay = ayanamsa(2_451_545.0, SidMode::FaganBradley);
        assert!((ay - 24.740).abs() < 0.1, "Fagan-Bradley at J2000 = {ay}");
    }

    #[test]
    fn ayanamsa_increases_with_time() {
        // Precession of the equinoxes drifts the vernal point WESTWARD
        // along the ecliptic at ~50.3" per year, so the sidereal-tropical
        // offset (ayanamsa) GROWS in the positive direction with time.
        // Reference (Indian Ephemeris & Nautical Almanac, Lahiri):
        //   J2000      → 23°51'11"
        //   J2000 + 1y → 23°52'01"
        let ay1 = ayanamsa(2_451_545.0, SidMode::Lahiri);
        let ay2 = ayanamsa(2_451_545.0 + 365.25, SidMode::Lahiri);
        let diff = ay2 - ay1;
        assert!(
            diff > 0.0 && diff < 0.02,
            "Ayanamsa should grow by ~50\"/year (~0.014°) — got Δ = {diff}",
        );
    }

    #[test]
    fn j2000_mode_is_zero() {
        let ay = ayanamsa(2_451_545.0, SidMode::J2000);
        assert!(ay.abs() < 0.001, "J2000 ayanamsa = {ay}");
    }

    #[test]
    fn all_modes_return_finite() {
        let jde = 2_451_545.0;
        for mode_id in 0..=35 {
            if let Some(mode) = SidMode::from_i32(mode_id) {
                let ay = ayanamsa(jde, mode);
                assert!(
                    ay.is_finite(),
                    "Mode {mode_id} returned non-finite ayanamsa"
                );
                assert!(
                    ay < 40.0 && ay > -5.0,
                    "Mode {mode_id} ayanamsa out of range: {ay}"
                );
            }
        }
    }
}
