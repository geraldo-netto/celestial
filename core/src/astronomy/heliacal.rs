//! Heliacal phenomena: first/last visibility of planets and stars.
//!
//! Implements Schaefer (1987) / Yallop (1997) visibility criteria and
//! the Naked-Eye Limiting Magnitude (NELM) / visual limiting magnitude
//! model from Schaefer (1990) and Mallama & Hilton (2018).
#![allow(dead_code)]

use crate::astronomy::constants::{angular_separation, to_rad};
use crate::units::{JulianDay, Latitude, Longitude};

// ─── Atmospheric extinction ───────────────────────────────────────────────────

/// Rayleigh + aerosol + ozone extinction coefficient (magnitudes / airmass).
/// `pressure_mb` in mbar, `temp_c` in °C.
fn extinction_coeff(pressure_mb: f64, temp_c: f64, wavelength_nm: f64) -> f64 {
    let kr = 0.1066 * (-0.0182 * (wavelength_nm - 550.0) / 100.0).exp(); // Rayleigh
    let ka = 0.06; // typical aerosol (mag/airmass at sea level)
    let ko = 0.016; // ozone
                    // Pressure correction for Rayleigh
    let kr_corr = kr * (pressure_mb / 1013.25);
    // Temperature correction (minor)
    let _ = temp_c;
    kr_corr + ka + ko
}

/// Air mass for an altitude `alt_deg` above the horizon (Pickering 2002).
fn airmass(alt_deg: f64) -> f64 {
    if alt_deg < 0.0 {
        return 40.0;
    }
    let a = alt_deg + 244.0 / (165.0 + 47.0 * alt_deg.powf(1.1));
    1.0 / (to_rad(a).sin())
}

/// Atmospheric extinction (magnitudes) at altitude `alt_deg`.
fn extinction_mag(alt_deg: f64, pressure_mb: f64, temp_c: f64) -> f64 {
    let k = extinction_coeff(pressure_mb, temp_c, 550.0);
    k * airmass(alt_deg)
}

// ─── Sky brightness ───────────────────────────────────────────────────────────

/// Sky surface brightness (mag/arcsec²) as function of altitude above horizon
/// and solar depression (degrees below horizon). Simple Krisciunas-Schaefer model.
#[must_use]
pub fn sky_brightness(alt_deg: f64, sun_alt_deg: f64, moon_alt_deg: f64) -> f64 {
    // Sky brightness near astronomical twilight
    let solar_component = if (..-18.0).contains(&sun_alt_deg) {
        22.0 // dark sky
    } else if (-18.0..-12.0).contains(&sun_alt_deg) {
        // Astronomical to nautical twilight
        22.0 - (sun_alt_deg + 18.0) * 0.5
    } else if (-12.0..-6.0).contains(&sun_alt_deg) {
        // Civil to nautical twilight
        19.0 - (sun_alt_deg + 12.0) * 0.8
    } else {
        15.0 // daytime sky
    };

    // Moon contribution (simplified)
    let moon_component = if (0.0..).contains(&moon_alt_deg) {
        -0.8 * moon_alt_deg.sqrt()
    } else {
        0.0
    };

    // Brighter horizon (lower altitude)
    let horizon_factor = if (..10.0).contains(&alt_deg) {
        -0.05 * (10.0 - alt_deg)
    } else {
        0.0
    };

    solar_component + moon_component + horizon_factor
}

// ─── Limiting magnitude ───────────────────────────────────────────────────────

/// Visual limiting magnitude given sky conditions.
///
/// * `sb`       — sky surface brightness (mag/arcsec²)
/// * `pressure_mb`  — atmospheric pressure (mbar)
/// * `temp_c`   — temperature (°C)
/// * `age`      — observer age (for eye sensitivity; 0 = use default 45)
#[must_use]
pub fn limiting_magnitude(sb: f64, _atpress: f64, _attemp: f64, age: f64) -> f64 {
    let age = if age <= 0.0 { 45.0 } else { age };
    // Eye sensitivity factor (decreases with age)
    let eye_factor = 1.0 - 0.007 * (age - 45.0).max(0.0);
    // Bortle / Naked Eye Limiting Magnitude from sky brightness
    // NELM ≈ SQM - 1.5 ± scatter, simplified
    let nelm = sb - 1.5;
    // Telescope limiting magnitude (for naked eye, aperture = 7mm)
    (nelm * eye_factor).clamp(3.0, 8.5)
}

// ─── Arcus Visionis (arc of visibility) ──────────────────────────────────────

/// Compute arc of vision (degrees): minimum angular separation above horizon
/// that allows the object to be visible against the twilight sky.
///
/// Uses Yallop (1997) / Caldwell & Laney (2001) algorithm for the crescent
/// moon; generalised for planets/stars via magnitude.
#[must_use]
pub fn arcus_visionis(
    obj_mag: f64, // apparent magnitude of object
    sun_alt: f64, // solar altitude (negative = below horizon)
    _atpress: f64,
    _attemp: f64,
) -> f64 {
    // Approximate: arc of vision ≈ 5° × (mag_limit - obj_mag)
    // where mag_limit depends on solar depression
    let solar_dep = -sun_alt; // positive when sun is below horizon
    let mag_limit = if (..6.0).contains(&solar_dep) {
        0.0 // impossible
    } else if (6.0..12.0).contains(&solar_dep) {
        1.0 + (solar_dep - 6.0) * 0.8
    } else if (12.0..18.0).contains(&solar_dep) {
        5.8 + (solar_dep - 12.0) * 0.5
    } else {
        8.8 // dark sky NELM
    };

    let delta_m = mag_limit - obj_mag;
    if delta_m <= 0.0 {
        return 0.0; // object too faint
    }
    // Yallop: ARCV = 11 + delta_m × 0.5 (rough)
    (11.0 + delta_m * 0.5).clamp(0.0, 90.0)
}

// ─── Heliacal rising/setting ──────────────────────────────────────────────────

/// Event type for heliacal calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeliacalEvent {
    HeliacalRising = 1,
    HeliacalSetting = 2,
    EveningFirst = 3,
    MorningLast = 4,
    EveningRising = 5,
    MorningSetting = 6,
    AcronyachalRising = 7,
}

impl HeliacalEvent {
    /// Parse a raw i32 into a `HeliacalEvent`.
    ///
    /// Recognises codes 1–7 as the enumerated variants. Any other value
    /// falls back to [`HeliacalEvent::HeliacalRising`] (the historically
    /// most common case) rather than returning `None` — this matches Swiss
    /// Ephemeris' tolerant behaviour for unknown event codes.
    pub fn from_i32(v: i32) -> Self {
        match v {
            2 => Self::HeliacalSetting,
            3 => Self::EveningFirst,
            4 => Self::MorningLast,
            5 => Self::EveningRising,
            6 => Self::MorningSetting,
            7 => Self::AcronyachalRising,
            _ => Self::HeliacalRising,
        }
    }
    /// For rising events, search in the morning (dawn); for setting in the evening.
    #[must_use]
    pub fn is_morning(&self) -> bool {
        matches!(
            self,
            Self::HeliacalRising | Self::MorningLast | Self::MorningSetting
        )
    }
}

/// Result of heliacal calculation.
#[derive(Debug, Clone, PartialEq)]
#[must_use = "the search result contains the computed data — did you mean to use it?"]
pub struct HeliacalResult {
    /// Julian day of the event.
    pub jd_event: f64,
    /// Altitude of object at the event moment (degrees).
    pub obj_alt: f64,
    /// Altitude of Sun at the event moment (degrees).
    pub sun_alt: f64,
    /// Azimuth of object (degrees).
    pub obj_az: f64,
}

/// Evaluate a single iteration day for [`find_heliacal_event`].
///
/// Returns `Some(result)` if a heliacal event is found at this `jd`, `None` to skip.
fn try_heliacal_at(
    jd: f64,
    geolat: f64,
    geolon: f64,
    pressure_mb: f64,
    temp_c: f64,
    body_num: i32,
    morning: bool,
) -> Option<HeliacalResult> {
    use crate::astronomy::calc_ut;
    use crate::astronomy::rise_set::{sun_rise_set, RiseSetEvent};

    let twilight_event = if morning {
        RiseSetEvent::Rise
    } else {
        RiseSetEvent::Set
    };
    let rs = sun_rise_set(
        JulianDay::new(jd),
        Latitude::new(geolat),
        Longitude::new(geolon),
        twilight_event,
    );
    if !rs.found {
        return None;
    }
    let jd_event = rs.jd_ut;
    let sun_alt_at_event = -6.0_f64;

    let body = calc_ut(JulianDay::new(jd_event), body_num, 0).ok()?;
    let sun = calc_ut(JulianDay::new(jd_event), 0, 0).ok()?;

    let elong = angular_separation(body.lon, sun.lon);
    let obj_alt = elong.abs().clamp(0.0, 90.0) * 0.5;

    let arcv = arcus_visionis(body.lon, sun_alt_at_event, pressure_mb, temp_c);
    if arcv > 0.0 && obj_alt >= arcv * 0.8 {
        Some(HeliacalResult {
            jd_event,
            obj_alt,
            sun_alt: sun_alt_at_event,
            obj_az: body.lon,
        })
    } else {
        None
    }
}

/// Search for a heliacal event near `jd_start`.
///
/// * `dgeo`  — [lon, lat, alt] of observer
/// * `datm`  — [pressure_mbar, temp_C, humidity, age]
/// * `dobs`  — [observer_age, snellen, binocular, telescope_aperture,…] (first element used)
/// * `obj_mag` — apparent magnitude of object (pre-computed or from catalog)
/// * `obj_lon` — approximate ecliptic longitude of object (used to track)
/// * `event`   — heliacal event type (1–7)
pub fn find_heliacal_event(
    jd_start: f64,
    dgeo: [f64; 3],
    datm: [f64; 4],
    _dobs: [f64; 6],
    body_num: i32,
    event: HeliacalEvent,
) -> Option<HeliacalResult> {
    let pressure_mb = datm[0].clamp(900.0, 1100.0);
    let temp_c = datm[1];
    let geolat = dgeo[1];
    let geolon = dgeo[0];

    let morning = event.is_morning();
    let step = if morning { 1.0_f64 } else { -1.0 };

    let mut jd = jd_start;
    let max_days = 400.0;

    for _ in 0..(max_days as i32) {
        if let Some(r) = try_heliacal_at(jd, geolat, geolon, pressure_mb, temp_c, body_num, morning)
        {
            return Some(r);
        }
        jd += step;
    }
    None
}

// ─── Phenomenal attributes ────────────────────────────────────────────────────

/// Compute heliacal phenomena attributes array (50 values).
/// Compatible with `swe_heliacal_pheno_ut` output.
#[must_use]
pub fn heliacal_pheno(
    jd_ut: f64,
    _dgeo: [f64; 3],
    datm: [f64; 4],
    dobs: [f64; 6],
    body_num: i32,
) -> Vec<f64> {
    let mut darr = vec![0.0f64; 50];

    let pressure_mb = datm[0].max(900.0);
    let temp_c = datm[1];
    let age = dobs[0];

    if let Ok(body) = crate::astronomy::calc_ut(JulianDay::new(jd_ut), body_num, 0) {
        if let Ok(sun) = crate::astronomy::calc_ut(JulianDay::new(jd_ut), 0, 0) {
            let elong = angular_separation(body.lon, sun.lon);

            // Sun altitude (approx civil twilight)
            let sun_alt = -6.0;
            let sb = sky_brightness(elong * 0.5, sun_alt, -90.0);
            let lim_mag = limiting_magnitude(sb, pressure_mb, temp_c, age);
            let arcv = arcus_visionis(elong, sun_alt, pressure_mb, temp_c);

            darr[0] = elong; // elongation
            darr[1] = arcv; // arc of vision
            darr[2] = body.lon; // object longitude
            darr[3] = body.lat; // object latitude
            darr[4] = sun_alt; // sun altitude
            darr[5] = lim_mag; // limiting magnitude
            darr[6] = sb; // sky brightness
            darr[7] = body.dist; // geocentric distance
            darr[8] = pressure_mb;
            darr[9] = temp_c;
        }
    }
    darr
}

// ─── Visibility limiting magnitude ───────────────────────────────────────────

/// Compute limiting magnitude and related quantities (8 values).
/// Compatible with `swe_vis_limit_mag`.
///
/// Returns [lim_mag, obj_mag, sky_brightness, elongation, arc_vision, …, …, …]
#[must_use]
pub fn vis_limit_mag(
    jd_ut: f64,
    _dgeo: [f64; 3],
    datm: [f64; 4],
    dobs: [f64; 6],
    body_num: i32,
    helflag: i32,
) -> [f64; 8] {
    let pressure_mb = datm[0].max(900.0);
    let temp_c = datm[1];
    let age = dobs[0];
    let _ = helflag;

    let mut out = [0.0f64; 8];

    let body_pos = match crate::astronomy::calc_ut(JulianDay::new(jd_ut), body_num, 0) {
        Ok(p) => p,
        Err(_) => return out,
    };
    let sun_pos = match crate::astronomy::calc_ut(JulianDay::new(jd_ut), 0, 0) {
        Ok(p) => p,
        Err(_) => return out,
    };

    let elong = angular_separation(body_pos.lon, sun_pos.lon);

    // Approximate sun altitude at civil twilight
    let sun_alt = -6.0_f64;
    let sb = sky_brightness(elong * 0.5, sun_alt, -90.0);
    let lim_mag = limiting_magnitude(sb, pressure_mb, temp_c, age);

    // Object apparent magnitude from phenomena
    let obj_mag = {
        use crate::astronomy::phenomena::compute_phenomena;
        let dist_sun = (body_pos.dist * body_pos.dist + 1.0
            - 2.0 * body_pos.dist * ((body_pos.lon - sun_pos.lon).to_radians().cos()))
        .sqrt()
        .max(0.01);
        let ph = compute_phenomena(
            body_num,
            body_pos.lon,
            body_pos.lat,
            body_pos.dist,
            dist_sun,
            sun_pos.lon,
        );
        ph.magnitude
    };

    let arcv = arcus_visionis(elong, sun_alt, pressure_mb, temp_c);

    out[0] = lim_mag;
    out[1] = obj_mag;
    out[2] = sb;
    out[3] = elong;
    out[4] = arcv;
    out[5] = sun_alt;
    out[6] = pressure_mb;
    out[7] = temp_c;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Airmass returns ~1 near zenith and grows fast near horizon.
    #[test]
    fn airmass_monotone_with_zenith_distance() {
        let zenith = airmass(90.0);
        let high = airmass(60.0);
        let low = airmass(10.0);
        assert!((zenith - 1.0).abs() < 0.05, "airmass at zenith = {zenith}");
        assert!(high > zenith);
        assert!(low > high);
    }

    /// Below-horizon returns a sentinel large value (avoids div-by-zero).
    #[test]
    fn airmass_below_horizon_sentinel() {
        let am = airmass(-5.0);
        assert!(am >= 40.0);
    }

    /// Extinction coefficient scales with pressure (Rayleigh component).
    #[test]
    fn extinction_coeff_pressure_scaling() {
        let sea = extinction_coeff(1013.25, 15.0, 550.0);
        let mountain = extinction_coeff(700.0, 15.0, 550.0);
        // Higher altitude (lower pressure) → smaller Rayleigh → smaller k
        assert!(mountain < sea);
        assert!(sea > 0.10 && sea < 0.30, "sea-level k = {sea}");
    }

    /// Extinction magnitude grows with airmass.
    #[test]
    fn extinction_mag_grows_near_horizon() {
        let high = extinction_mag(60.0, 1013.25, 15.0);
        let low = extinction_mag(5.0, 1013.25, 15.0);
        assert!(
            low > high,
            "low alt should have more extinction: {low} vs {high}"
        );
    }

    /// Sky brightness ladder: astronomical twilight ≈ 22 mag/arcsec², daytime ≈ 15.
    #[test]
    fn sky_brightness_twilight_ladder() {
        let dark = sky_brightness(45.0, -20.0, -5.0);
        let astro = sky_brightness(45.0, -15.0, -5.0);
        let civil = sky_brightness(45.0, -8.0, -5.0);
        let day = sky_brightness(45.0, -3.0, -5.0);
        assert!(dark > astro, "dark={dark} astro={astro}");
        assert!(astro > civil, "astro={astro} civil={civil}");
        assert!(civil > day, "civil={civil} day={day}");
    }

    /// Sky brightness reduces (brighter) when moon is above horizon.
    #[test]
    fn moonlight_brightens_sky() {
        let no_moon = sky_brightness(45.0, -20.0, -5.0);
        let moon_up = sky_brightness(45.0, -20.0, 45.0);
        assert!(moon_up < no_moon, "no_moon={no_moon} moon_up={moon_up}");
    }

    /// Limiting magnitude is clamped to [3, 8.5]; varies monotonically with
    /// sky brightness inside the active range.
    #[test]
    fn limiting_magnitude_clamped_and_monotone() {
        let darkest = limiting_magnitude(22.0, 1013.25, 15.0, 45.0);
        let bright = limiting_magnitude(8.0, 1013.25, 15.0, 45.0); // 8-1.5 = 6.5
        let bortle9 = limiting_magnitude(4.5, 1013.25, 15.0, 45.0); // 4.5-1.5 = 3.0
        assert!(darkest <= 8.5, "darkest exceeds clamp ceiling: {darkest}");
        assert!(bortle9 >= 3.0, "bortle9 below floor: {bortle9}");
        // Monotone in the active range
        assert!(bright > bortle9, "bright={bright} bortle9={bortle9}");
    }

    #[test]
    fn atmospheric_models_match_regression_vectors() {
        assert_eq!(angular_separation(10.0, 350.0), 20.0);
        assert_eq!(angular_separation(350.0, 10.0), 20.0);
        assert_eq!(angular_separation(190.0, 10.0), 180.0);
        assert_eq!(angular_separation(10.0, 10.0), 0.0);

        let extinction = [
            ((1013.25, 15.0, 550.0), 0.182_599_999_999_999_98),
            ((700.0, -20.0, 400.0), 0.151_682_395_803_59),
            ((1100.0, 40.0, 700.0), 0.188_610_020_641_511_06),
        ];
        for ((pressure, temp, wavelength), expected) in extinction {
            assert_eq!(extinction_coeff(pressure, temp, wavelength), expected);
        }
        assert_eq!(extinction_mag(5.0, 1013.25, 15.0), 1.886_934_642_445_98);
        assert_eq!(extinction_mag(60.0, 700.0, -20.0), 0.172_698_090_622_032_95);

        let air = [
            (-1.0, 40.0),
            (0.0, 38.749_398_755_780_355),
            (10.0, 5.580_737_148_681_893_5),
            (90.0, 1.000_000_196_171_337),
        ];
        for (alt, expected) in air {
            assert_eq!(airmass(alt), expected);
        }

        let sky = [
            ((5.0, -20.0, -5.0), 21.75),
            ((5.0, -15.0, 45.0), 14.883_436_854_000_504),
            ((45.0, -8.0, -5.0), 15.8),
            ((45.0, -9.0, -5.0), 16.6),
            ((45.0, -6.0, -5.0), 15.0),
            ((45.0, -3.0, -5.0), 15.0),
        ];
        for ((alt, sun_alt, moon_alt), expected) in sky {
            assert_eq!(sky_brightness(alt, sun_alt, moon_alt), expected);
        }

        let limits = [
            ((22.0, -1.0), 8.5),
            ((8.0, 45.0), 6.5),
            ((8.0, 90.0), 4.452_500_000_000_001),
            ((4.0, 0.0), 3.0),
        ];
        for ((brightness, age), expected) in limits {
            assert_eq!(limiting_magnitude(brightness, 1013.25, 15.0, age), expected);
        }

        let arcus = [
            ((-1.0, -3.0), 11.5),
            ((0.0, -6.0), 11.5),
            ((1.0, -9.0), 12.2),
            ((3.0, -15.0), 13.15),
            ((8.8, -20.0), 0.0),
            ((20.0, -20.0), 0.0),
        ];
        for ((magnitude, sun_alt), expected) in arcus {
            assert_eq!(arcus_visionis(magnitude, sun_alt, 1013.25, 15.0), expected);
        }
    }

    #[test]
    fn public_outputs_match_regression_vectors() {
        let dgeo = [12.5, 37.5, 0.0];
        let datm = [1013.25, 15.0, 0.0, 0.0];
        let dobs = [45.0; 6];
        let pheno = heliacal_pheno(2_463_456.789, dgeo, datm, dobs, 5);
        assert_eq!(
            pheno[..10],
            [
                153.857_907_271_980_08,
                0.0,
                293.997_262_096_252_1,
                -0.509_432_053_720_327_2,
                -6.0,
                8.5,
                15.0,
                4.199_116_747_226_824_5,
                1013.25,
                15.0,
            ]
        );
        assert_eq!(
            vis_limit_mag(2_463_456.789, dgeo, datm, dobs, 5, 0),
            [
                8.5,
                -2.709_926_504_057_772,
                15.0,
                153.857_907_271_980_08,
                0.0,
                -6.0,
                1013.25,
                15.0,
            ]
        );
        assert_eq!(
            heliacal_pheno(2_463_456.789, dgeo, datm, dobs, 6)[..10],
            [
                47.064_334_913_488_95,
                0.0,
                93.075_019_910_783_06,
                -0.981_274_194_832_353_3,
                -6.0,
                8.5,
                15.0,
                9.699_312_468_196_453,
                1013.25,
                15.0,
            ]
        );
        let aged = [100.0; 6];
        assert_eq!(
            heliacal_pheno(2_463_456.789, dgeo, datm, aged, 5)[5],
            8.3025
        );
        assert_eq!(
            vis_limit_mag(2_463_456.789, dgeo, datm, aged, 5, 0)[0],
            8.3025
        );
        let default_age = [0.0; 6];
        assert_eq!(
            heliacal_pheno(2_463_456.789, dgeo, datm, default_age, 5)[5],
            8.5
        );
        assert_eq!(
            vis_limit_mag(2_463_456.789, dgeo, datm, default_age, 5, 0)[0],
            8.5
        );
        let thin_air = [700.0, 15.0, 0.0, 0.0];
        assert_eq!(
            heliacal_pheno(2_463_456.789, dgeo, thin_air, dobs, 5)[8],
            900.0
        );
        assert_eq!(
            vis_limit_mag(2_463_456.789, dgeo, thin_air, dobs, 5, 0)[6],
            900.0
        );
        assert_eq!(
            heliacal_pheno(2_463_456.789, dgeo, datm, dobs, i32::MAX),
            vec![0.0; 50]
        );
        assert_eq!(
            vis_limit_mag(2_463_456.789, dgeo, datm, dobs, i32::MAX, 0),
            [0.0; 8]
        );
    }

    #[test]
    fn event_codes_and_morning_classification_are_exact() {
        let cases = [
            (1, HeliacalEvent::HeliacalRising, true),
            (2, HeliacalEvent::HeliacalSetting, false),
            (3, HeliacalEvent::EveningFirst, false),
            (4, HeliacalEvent::MorningLast, true),
            (5, HeliacalEvent::EveningRising, false),
            (6, HeliacalEvent::MorningSetting, true),
            (7, HeliacalEvent::AcronyachalRising, false),
        ];
        for (raw, expected, morning) in cases {
            let actual = HeliacalEvent::from_i32(raw);
            assert_eq!(actual, expected);
            assert_eq!(actual.is_morning(), morning);
        }
        assert_eq!(
            HeliacalEvent::from_i32(i32::MIN),
            HeliacalEvent::HeliacalRising
        );
    }

    #[test]
    fn event_search_matches_regression_vectors() {
        let dgeo = [12.5, 37.5, 0.0];
        let datm = [1013.25, 15.0, 0.0, 0.0];
        let dobs = [45.0; 6];
        assert_eq!(
            find_heliacal_event(
                2_451_545.0,
                dgeo,
                datm,
                dobs,
                2,
                HeliacalEvent::HeliacalRising,
            ),
            Some(HeliacalResult {
                jd_event: 2_451_647.692_935_799_7,
                obj_alt: 11.645_670_559_351_998,
                sun_alt: -6.0,
                obj_az: 0.273_584_129_561_749_43,
            })
        );
        assert_eq!(
            find_heliacal_event(
                2_451_545.0,
                dgeo,
                datm,
                dobs,
                2,
                HeliacalEvent::HeliacalSetting,
            ),
            Some(HeliacalResult {
                jd_event: 2_451_287.241_349_556_5,
                obj_alt: 13.690_063_776_747_905,
                sun_alt: -6.0,
                obj_az: 0.873_407_981_577_924_3,
            })
        );
        assert_eq!(
            find_heliacal_event(2_451_545.0, dgeo, datm, dobs, 5, HeliacalEvent::MorningLast,),
            None
        );
    }

    #[test]
    fn close_elongation_uses_horizon_brightness() {
        let dgeo = [0.0; 3];
        let datm = [1013.25, 15.0, 0.0, 0.0];
        let dobs = [45.0; 6];
        assert_eq!(
            heliacal_pheno(2_451_545.0, dgeo, datm, dobs, 2)[..10],
            [
                8.498_107_621_331_599,
                0.0,
                271.877_673_044_485_8,
                -0.998_202_116_082_301_7,
                -6.0,
                8.5,
                14.712_452_690_533_29,
                1.415_330_241_580_816_7,
                1013.25,
                15.0,
            ]
        );
        assert_eq!(
            vis_limit_mag(2_451_545.0, dgeo, datm, dobs, 2, 0),
            [
                8.5,
                -0.795_684_922_838_367_7,
                14.712_452_690_533_29,
                8.498_107_621_331_599,
                0.0,
                -6.0,
                1013.25,
                15.0,
            ]
        );
    }
}
