//! Library configuration and information functions.

use crate::body::{Body, CalcFlags, SiderealMode};
use crate::error::Result;
use crate::units::{JulianDay, Latitude, Longitude};

use std::cell::Cell;

/// All thread-local engine configuration in one typed record (DP-9):
/// replaces the three separate `thread_local! { Cell<…> }` globals so a
/// new setting is one field + one accessor, audited in one place.
/// Values/defaults are byte-identical to the previous globals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct EngineConfig {
    /// Sidereal mode for pure-mode ayanamsa (default 0 = Fagan-Bradley).
    sid_mode: i32,
    /// Topocentric observer position `(lon°, lat°, alt_m)`.
    topo: (f64, f64, f64),
    /// User delta-T override in **days**, if set.
    delta_t: Option<f64>,
}

const DEFAULT_CONFIG: EngineConfig = EngineConfig {
    sid_mode: 0,
    topo: (0.0, 0.0, 0.0),
    delta_t: None,
};

thread_local! {
    static CONFIG: Cell<EngineConfig> = const { Cell::new(DEFAULT_CONFIG) };
}

/// Read the thread-local config.
fn cfg() -> EngineConfig {
    CONFIG.with(Cell::get)
}

/// Copy all thread-local engine settings from the current thread.
pub(crate) fn current_config() -> EngineConfig {
    cfg()
}

/// Install a full engine-config snapshot in the current thread.
pub(crate) fn set_thread_config(config: EngineConfig) {
    CONFIG.with(|c| c.set(config));
}

/// Mutate one field of the thread-local config.
fn cfg_update(f: impl FnOnce(&mut EngineConfig)) {
    CONFIG.with(|c| {
        let mut v = c.get();
        f(&mut v);
        c.set(v);
    });
}

/// Returns the currently active sidereal mode (for pure-mode ayanamsa).
pub(crate) fn current_sid_mode() -> i32 {
    cfg().sid_mode
}

/// Metadata about a currently open ephemeris file.
#[derive(Debug, Clone, PartialEq)]
pub struct CurrentFileData {
    /// File path.
    pub path: String,
    /// Start date (JDE).
    pub tfstart: f64,
    /// End date (JDE).
    pub tfend: f64,
    /// DE number (e.g. 430).
    pub denum: i32,
}

// ─── Configuration setters ────────────────────────────────────────────────────

/// Set the path for Swiss Ephemeris data files.
pub fn set_ephe_path(_path: &str) -> Result<()> {
    Ok(())
}

/// Set the JPL file to use.
pub fn set_jpl_file(_fname: &str) -> Result<()> {
    Ok(())
}

/// Returns the currently stored topocentric observer position `(lon°, lat°, alt_m)`.
#[allow(dead_code)]
pub(crate) fn current_topo() -> (f64, f64, f64) {
    cfg().topo
}

/// Set the topocentric observer position.
///
/// Stored and used by `FLG_TOPOCTR` calculations and by `azalt` / `azalt_rev`
/// when no explicit geopos is given.
pub fn set_topo(geolon: Longitude, geolat: Latitude, geoalt: f64) {
    let geolon: f64 = geolon.into();
    let geolat: f64 = geolat.into();
    cfg_update(|c| c.topo = (geolon, geolat, geoalt));
}

/// Set the sidereal mode for ayanamsa calculations.
pub fn set_sid_mode(sid_mode: SiderealMode, _t0: f64, _ayan_t0: f64) {
    cfg_update(|c| c.sid_mode = sid_mode.as_raw());
}

/// Returns the user-defined delta T override (days), if set.
pub(crate) fn user_delta_t() -> Option<f64> {
    cfg().delta_t
}

/// Override the automatic delta T calculation with a fixed value.
///
/// `dt` is in **seconds**; stored as days.  Pass `f64::NAN` to clear.
pub fn set_delta_t_userdef(dt: f64) {
    let val = if dt.is_finite() {
        Some(dt / 86400.0)
    } else {
        None
    };
    cfg_update(|c| c.delta_t = val);
}

/// Set the tidal acceleration.
pub fn set_tid_acc(_t_acc: f64) {}

/// Get the current tidal acceleration.
#[must_use]
pub fn tid_acc() -> f64 {
    0.0
}

/// Set the atmospheric lapse rate.
pub fn set_lapse_rate(_lapse_rate: f64) {}

// ─── Ayanamsa ─────────────────────────────────────────────────────────────────

/// Get the ayanamsa for a JDE (TT).
#[must_use]
pub fn ayanamsa(jd_et: JulianDay) -> f64 {
    let jd_et: f64 = jd_et.into();
    crate::astronomy::get_ayanamsa(jd_et, current_sid_mode())
}

/// Get the ayanamsa for a JD (UT).
#[must_use]
pub fn ayanamsa_ut(jd_ut: JulianDay) -> f64 {
    let jd_ut: f64 = jd_ut.into();
    crate::astronomy::get_ayanamsa(jd_ut, current_sid_mode())
}

/// Get ayanamsa with extended flags (TT).
pub fn ayanamsa_ex(jd_et: JulianDay, _flags: CalcFlags) -> Result<f64> {
    let jd_et: f64 = jd_et.into();
    Ok(crate::astronomy::get_ayanamsa(jd_et, current_sid_mode()))
}

/// Get ayanamsa with extended flags (UT).
pub fn ayanamsa_ex_ut(jd_ut: JulianDay, _flags: CalcFlags) -> Result<f64> {
    let jd_ut: f64 = jd_ut.into();
    Ok(crate::astronomy::get_ayanamsa(jd_ut, current_sid_mode()))
}

/// Get the name of a sidereal mode.
#[must_use]
pub fn ayanamsa_name(isidmode: i32) -> &'static str {
    crate::astronomy::get_ayanamsa_name(isidmode)
}

// ─── Info ─────────────────────────────────────────────────────────────────────

/// Get the Swiss Ephemeris version string.
#[must_use]
pub fn version() -> &'static str {
    crate::astronomy::engine_version()
}

/// Get the library path.
#[must_use]
pub fn library_path() -> String {
    String::from("(pure-Rust engine, no library path)")
}

/// Get the planet name for a body number.
#[must_use]
pub fn planet_name(body: Body) -> &'static str {
    crate::astronomy::planet_name(body.as_raw())
}

/// Get current ephemeris file data.
pub fn current_file_data(_ifno: i32) -> Option<CurrentFileData> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_defaults_match_legacy() {
        // Fresh thread → defaults identical to the old per-global ones.
        assert_eq!(current_sid_mode(), 0);
        assert_eq!(current_topo(), (0.0, 0.0, 0.0));
        assert_eq!(user_delta_t(), None);
    }

    #[test]
    fn config_set_get_roundtrip_isolated_fields() {
        // Each setter touches only its field; others keep prior values.
        set_sid_mode(SiderealMode::LAHIRI, 0.0, 0.0);
        assert_eq!(current_sid_mode(), 1);
        assert_eq!(current_topo(), (0.0, 0.0, 0.0));

        set_topo(Longitude::new(12.5), Latitude::new(-7.25), 100.0);
        assert_eq!(current_topo(), (12.5, -7.25, 100.0));
        assert_eq!(current_sid_mode(), 1); // unchanged

        set_delta_t_userdef(86400.0); // 1 day in seconds → 1.0 day
        assert_eq!(user_delta_t(), Some(1.0));

        set_delta_t_userdef(f64::NAN); // clears
        assert_eq!(user_delta_t(), None);
        assert_eq!(current_topo(), (12.5, -7.25, 100.0)); // still set
    }
}
