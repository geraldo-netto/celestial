//! Library configuration and information functions.

use crate::body::{Body, CalcFlags, SiderealMode};
use crate::error::Result;

use std::cell::Cell;

thread_local! {
    static CURRENT_SID_MODE: Cell<i32> = const { Cell::new(0) }; // default: Fagan-Bradley
}

/// Returns the currently active sidereal mode (for pure-mode ayanamsa).
pub(crate) fn current_sid_mode() -> i32 {
    CURRENT_SID_MODE.with(|m| m.get())
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

thread_local! {
    /// Observer position for topocentric calculations (lon°, lat°, alt_m).
    static TOPO_POS: std::cell::Cell<(f64, f64, f64)> =
        const { std::cell::Cell::new((0.0, 0.0, 0.0)) };
}

/// Returns the currently stored topocentric observer position `(lon°, lat°, alt_m)`.
#[allow(dead_code)]
pub(crate) fn current_topo() -> (f64, f64, f64) {
    TOPO_POS.with(|p| p.get())
}

/// Set the topocentric observer position.
///
/// Stored and used by `azalt` / `azalt_rev` when no explicit geopos is given.
/// Has no effect on `calc_ut` (which computes geocentric positions only).
pub fn set_topo(geolon: f64, geolat: f64, geoalt: f64) {
    TOPO_POS.with(|p| p.set((geolon, geolat, geoalt)));
}

/// Set the sidereal mode for ayanamsa calculations.
pub fn set_sid_mode(sid_mode: SiderealMode, _t0: f64, _ayan_t0: f64) {
    CURRENT_SID_MODE.with(|m| m.set(sid_mode.as_raw()));
}

thread_local! {
    static DELTA_T_USERDEF: Cell<Option<f64>> = const { Cell::new(None) };
}

/// Returns the user-defined delta T override (days), if set.
pub(crate) fn user_delta_t() -> Option<f64> {
    DELTA_T_USERDEF.with(|d| d.get())
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
    DELTA_T_USERDEF.with(|d| d.set(val));
}

/// Set the tidal acceleration.
pub fn set_tid_acc(_t_acc: f64) {}

/// Get the current tidal acceleration.
pub fn tid_acc() -> f64 {
    0.0
}

/// Set the atmospheric lapse rate.
pub fn set_lapse_rate(_lapse_rate: f64) {}

// ─── Ayanamsa ─────────────────────────────────────────────────────────────────

/// Get the ayanamsa for a JDE (TT).
pub fn ayanamsa(jd_et: f64) -> f64 {
    crate::astronomy::get_ayanamsa(jd_et, current_sid_mode())
}

/// Get the ayanamsa for a JD (UT).
pub fn ayanamsa_ut(jd_ut: f64) -> f64 {
    crate::astronomy::get_ayanamsa(jd_ut, current_sid_mode())
}

/// Get ayanamsa with extended flags (TT).
pub fn ayanamsa_ex(jd_et: f64, _flags: CalcFlags) -> Result<f64> {
    Ok(crate::astronomy::get_ayanamsa(jd_et, current_sid_mode()))
}

/// Get ayanamsa with extended flags (UT).
pub fn ayanamsa_ex_ut(jd_ut: f64, _flags: CalcFlags) -> Result<f64> {
    Ok(crate::astronomy::get_ayanamsa(jd_ut, current_sid_mode()))
}

/// Get the name of a sidereal mode.
pub fn ayanamsa_name(isidmode: i32) -> &'static str {
    crate::astronomy::get_ayanamsa_name(isidmode)
}

// ─── Info ─────────────────────────────────────────────────────────────────────

/// Get the Swiss Ephemeris version string.
pub fn version() -> &'static str {
    crate::astronomy::engine_version()
}

/// Get the library path.
pub fn library_path() -> String {
    String::from("(pure-Rust engine, no library path)")
}

/// Get the planet name for a body number.
pub fn planet_name(body: Body) -> &'static str {
    crate::astronomy::planet_name(body.as_raw())
}

/// Get current ephemeris file data.
pub fn current_file_data(_ifno: i32) -> Option<CurrentFileData> {
    None
}
