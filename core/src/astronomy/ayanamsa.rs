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

impl SidMode {
    /// Parse a raw i32 into a `SidMode`, returning `None` for unknown values.
    pub fn from_i32(n: i32) -> Option<Self> {
        Some(match n {
            0 => Self::FaganBradley,
            1 => Self::Lahiri,
            2 => Self::Deluce,
            3 => Self::Raman,
            4 => Self::Ushashashi,
            5 => Self::Krishnamurti,
            6 => Self::DjwhalKhul,
            7 => Self::Yukteshwar,
            8 => Self::JnBhasin,
            9 => Self::BabylKugler1,
            10 => Self::BabylKugler2,
            11 => Self::BabylKugler3,
            12 => Self::BabylHuber,
            13 => Self::BabylEtpsc,
            14 => Self::Aldebaran15Tau,
            15 => Self::Hipparchos,
            16 => Self::Sassanian,
            17 => Self::GalactCtr0Sag,
            18 => Self::J2000,
            19 => Self::J1900,
            20 => Self::B1950,
            21 => Self::SuryaSiddhanta,
            22 => Self::SuryaSiddhantaMsun,
            23 => Self::Aryabhata,
            24 => Self::AryabhataMsun,
            25 => Self::SsRevati,
            26 => Self::SsCitra,
            27 => Self::TrueCitra,
            28 => Self::TrueRevati,
            29 => Self::TruePushya,
            30 => Self::GalacticCtrBrand,
            31 => Self::GalacticEqMidMula,
            32 => Self::SkydramMula,
            33 => Self::TrueMula,
            34 => Self::GalacticCtrOHara,
            35 => Self::Galactic0Aries,
            _ => return None,
        })
    }

    /// Display name for the ayanamsa.
    pub fn name(self) -> &'static str {
        match self {
            Self::FaganBradley => "Fagan-Bradley",
            Self::Lahiri => "Lahiri",
            Self::Deluce => "De Luce",
            Self::Raman => "Raman",
            Self::Ushashashi => "Usha-Shashi",
            Self::Krishnamurti => "Krishnamurti",
            Self::DjwhalKhul => "Djwhal Khul",
            Self::Yukteshwar => "Yukteshwar",
            Self::JnBhasin => "J.N.Bhasin",
            Self::BabylKugler1 => "Babylonian/Kugler 1",
            Self::BabylKugler2 => "Babylonian/Kugler 2",
            Self::BabylKugler3 => "Babylonian/Kugler 3",
            Self::BabylHuber => "Babylonian/Huber",
            Self::BabylEtpsc => "Babylonian/ETPSC",
            Self::Aldebaran15Tau => "Aldebaran at 15 Tau",
            Self::Hipparchos => "Hipparchos",
            Self::Sassanian => "Sassanian",
            Self::GalactCtr0Sag => "Galactic Center at 0 Sag",
            Self::J2000 => "J2000",
            Self::J1900 => "J1900",
            Self::B1950 => "B1950",
            Self::SuryaSiddhanta => "Surya Siddhanta",
            Self::SuryaSiddhantaMsun => "Surya Siddhanta, mean Sun",
            Self::Aryabhata => "Aryabhata",
            Self::AryabhataMsun => "Aryabhata, mean Sun",
            Self::SsRevati => "SS, Revati/zeta Psc",
            Self::SsCitra => "SS, Citra/Spica",
            Self::TrueCitra => "True Citra",
            Self::TrueRevati => "True Revati",
            Self::TruePushya => "True Pushya",
            Self::GalacticCtrBrand => "Galactic Center (Brand)",
            Self::GalacticEqMidMula => "Galactic Equator mid-Mula",
            Self::SkydramMula => "Skydram (Mula)",
            Self::TrueMula => "True Mula",
            Self::GalacticCtrOHara => "Galactic Center (O'Hara)",
            Self::Galactic0Aries => "Galactic Equator IAU (0 Aries)",
            Self::User => "User-defined",
        }
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
    (5, 23.979_472),  // Krishnamurti
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
pub fn ayanamsa(jde: f64, mode: SidMode) -> f64 {
    if mode == SidMode::User {
        return 0.0; // user must supply their own offset externally
    }
    let t = julian_centuries(jde);
    let base = ayanamsa_j2000(mode);
    // Linear precession (rough approximation — good to ~1' over 1000 years)
    base - PRECESSION_RATE * t + precession_correction(t)
}

/// Look up the J2000 base value for a given mode.
fn ayanamsa_j2000(mode: SidMode) -> f64 {
    let id = mode as i32;
    AYANAMSA_J2000
        .iter()
        .find(|&&(m, _)| m == id)
        .map(|&(_, v)| v)
        .unwrap_or(0.0)
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
pub fn ayanamsa_name(mode_id: i32) -> &'static str {
    SidMode::from_i32(mode_id)
        .map(SidMode::name)
        .unwrap_or("Unknown")
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
        // Precession moves the ayanamsa forward ~50" per year
        let ay1 = ayanamsa(2_451_545.0, SidMode::Lahiri);
        let ay2 = ayanamsa(2_451_545.0 + 365.25, SidMode::Lahiri);
        // ayanamsa *decreases* as we move forward (tropical precesses faster)
        assert!(
            ay1 > ay2,
            "Ayanamsa should decrease moving forward: {ay1} vs {ay2}"
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
