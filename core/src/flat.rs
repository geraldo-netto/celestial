//! Flat re-exports — backward compatibility layer.
//!
//! These are all the symbols that were previously exported at the crate root.
//! Prefer the namespaced forms (`celestial_core::position::calc_ut`) in new code.

pub use crate::functions::aspects::*;
pub use crate::functions::calc::*;
pub use crate::functions::chart::*;
pub use crate::functions::config::*;
pub use crate::functions::easter::*;
pub use crate::functions::eclipses::*;
pub use crate::functions::esbats::*;
pub use crate::functions::geoformat::*;
pub use crate::functions::houses::*;
pub use crate::functions::islamic::*;
pub use crate::functions::jewish::*;
pub use crate::functions::moon_phases::*;
pub use crate::functions::motion::*;
pub use crate::functions::nowruz::*;
pub use crate::functions::omer::*;
pub use crate::functions::panchanga::*;
pub use crate::functions::phenomena::*;
pub use crate::functions::sabbats::*;
pub use crate::functions::searches::*;
pub use crate::functions::time::*;
pub use crate::functions::timezone::*;
pub use crate::functions::utils::*;
pub use crate::functions::vedic::*;
pub use crate::functions::vesak::*;

// Body type shortcuts at crate root for convenience
pub use crate::body::{Body, CalcFlags, Calendar, HouseSystem, SiderealMode};

// Raw i32 backward-compat constants (delegate to Body/CalcFlags)
// These exist so existing code that does `use celestial_core::SUN` still compiles.
pub const SUN: i32 = 0;
pub const MOON: i32 = 1;
pub const MERCURY: i32 = 2;
pub const VENUS: i32 = 3;
pub const MARS: i32 = 4;
pub const JUPITER: i32 = 5;
pub const SATURN: i32 = 6;
pub const URANUS: i32 = 7;
pub const NEPTUNE: i32 = 8;
pub const PLUTO: i32 = 9;
pub const MEAN_NODE: i32 = 10;
pub const TRUE_NODE: i32 = 11;
pub const CHIRON: i32 = 15;
pub const GREG_CAL: i32 = 1;
pub const JUL_CAL: i32 = 0;
pub const FLG_BUILTIN: i32 = 2;
pub const FLG_SPEED: i32 = 256;
pub const FLG_SIDEREAL: i32 = 65536;
pub const FLG_EQUATORIAL: i32 = 2048;
pub const FLG_HELCTR: i32 = 8;
pub const FLG_TOPOCTR: i32 = 32768;
pub const FLG_NONUT: i32 = 64;
pub const FLG_RADIANS: i32 = 8192;
pub const FLG_JPL: i32 = 1;
pub const FLG_MOSHIER: i32 = 4;
pub const SIDM_FAGAN_BRADLEY: i32 = 0;
pub const SIDM_LAHIRI: i32 = 1;
pub const SIDM_DELUCE: i32 = 2;
pub const SIDM_RAMAN: i32 = 3;
pub const SIDM_KRISHNAMURTI: i32 = 5;
pub const CALC_RISE: i32 = 1;
pub const CALC_SET: i32 = 2;
pub const CALC_MTRANSIT: i32 = 4;
pub const CALC_ITRANSIT: i32 = 8;
pub const ASC: i32 = 0;
pub const MC: i32 = 1;

// Eclipse type constants
pub const ECL_TOTAL: i32 = 4;
pub const ECL_ANNULAR: i32 = 8;
pub const ECL_PARTIAL: i32 = 16;
pub const ECL_HYBRID: i32 = 32;
pub const ECL_PENUMBRAL: i32 = 64;
pub const ECL_OCCULTATION: i32 = 64;

// Refraction constants
pub const TRUE_TO_APP: i32 = 0;
pub const APP_TO_TRUE: i32 = 1;

// split_deg flags
pub const SPLIT_DEG_ROUND_SEC: i32 = 1;
pub const SPLIT_DEG_ZODIACAL: i32 = 8;

// Additional body constants
pub const EARTH_BODY: i32 = 14; // EARTH conflicts with Body::EARTH

// Additional flag constants
pub const FLG_NOABERR: i32 = 1024;
pub const FLG_NOGDEFL: i32 = 512;
pub const FLG_XYZ: i32 = 4096;
pub const FLG_TRUEPOS: i32 = 16;
pub const FLG_J2000: i32 = 32;
pub const FLG_SPEED3: i32 = 128;

// Eclipse constants
pub const ECL_CENTRAL: i32 = 1;
pub const ECL_NONCENTRAL: i32 = 2;
pub const ECL_ANNULAR_TOTAL: i32 = 32;

// split_deg additional flags
pub const SPLIT_DEG_ROUND_MIN: i32 = 2;
pub const SPLIT_DEG_ROUND_DEG: i32 = 4;
pub const SPLIT_DEG_NAKSHATRA: i32 = 1024;
pub const SPLIT_DEG_KEEP_SIGN: i32 = 256;
pub const SPLIT_DEG_KEEP_DEG: i32 = 512;

// Sidereal mode constants
pub const SIDM_USER: i32 = 255;
pub const SIDM_SASSANIAN: i32 = 11;
pub const SIDM_GALCENT_RGILBR: i32 = 16;
pub const SIDM_GALEQU_IAU1958: i32 = 17;
pub const SIDM_GALEQU_TRUE: i32 = 18;
pub const SIDM_GALEQU_MULA: i32 = 19;
pub const SIDM_GALALIGN_MARDYKS: i32 = 20;
pub const SIDM_TEO_NINOS: i32 = 21;
pub const SIDM_NEWCOMB: i32 = 22;

// Body constants missing from flat.rs
pub const ECL_NUT: i32 = -1;
pub const MEAN_APOG: i32 = 12;
pub const OSCU_APOG: i32 = 13;

// ── All chart, vedic, utils, phenomena, and tradition functions are already
// ── exported via the pub use wildcards above (chart::*, vedic::*, utils::*, etc.).
// ── No explicit re-exports are needed here.

// ── Deprecated backward-compat aliases for renamed functions ─────────────────
// These will emit a compiler warning when used; prefer the new names.

#[deprecated(since = "0.2.0", note = "use `norm_deg` instead")]
pub fn degnorm(x: f64) -> f64 {
    crate::functions::utils::norm_deg(x)
}

#[deprecated(since = "0.2.0", note = "use `norm_rad` instead")]
pub fn radnorm(x: f64) -> f64 {
    crate::functions::utils::norm_rad(x)
}

#[deprecated(since = "0.2.0", note = "use `midpoint_deg` instead")]
pub fn deg_midp(x1: f64, x0: f64) -> f64 {
    crate::functions::utils::midpoint_deg(x1, x0)
}

#[deprecated(since = "0.2.0", note = "use `midpoint_rad` instead")]
pub fn rad_midp(x1: f64, x0: f64) -> f64 {
    crate::functions::utils::midpoint_rad(x1, x0)
}

#[deprecated(since = "0.2.0", note = "use `diff_deg_signed` instead")]
pub fn difdeg2n(p1: f64, p2: f64) -> f64 {
    crate::functions::utils::diff_deg_signed(p1, p2)
}

#[deprecated(since = "0.2.0", note = "use `diff_deg` instead")]
pub fn difdegn(p1: f64, p2: f64) -> f64 {
    crate::functions::utils::diff_deg(p1, p2)
}

#[deprecated(since = "0.2.0", note = "use `diff_rad_signed` instead")]
pub fn difrad2n(p1: f64, p2: f64) -> f64 {
    crate::functions::utils::diff_rad_signed(p1, p2)
}

#[deprecated(since = "0.2.0", note = "use `norm_cs` instead")]
pub fn csnorm(p: i32) -> i64 {
    crate::functions::utils::norm_cs(p)
}

#[deprecated(since = "0.2.0", note = "use `cs_round_sec` instead")]
pub fn csroundsec(x: i32) -> i64 {
    crate::functions::utils::cs_round_sec(x)
}

#[deprecated(since = "0.2.0", note = "use `deg_to_cs` instead")]
pub fn d2l(x: f64) -> i64 {
    crate::functions::utils::deg_to_cs(x)
}

#[deprecated(since = "0.2.0", note = "use `coord_transform` instead")]
pub fn cotrans(coords: [f64; 3], obliquity: f64) -> [f64; 3] {
    crate::functions::utils::coord_transform(coords, obliquity)
}

#[deprecated(since = "0.2.0", note = "use `coord_transform_with_speed` instead")]
pub fn cotrans_sp(coords: [f64; 6], obliquity: f64) -> [f64; 6] {
    crate::functions::utils::coord_transform_with_speed(coords, obliquity)
}

#[deprecated(since = "0.2.0", note = "use `centisec_to_deg_str` instead")]
pub fn cs2degstr(cs: i32) -> String {
    crate::functions::geoformat::centisec_to_deg_str(cs)
}

#[deprecated(since = "0.2.0", note = "use `centisec_to_lonlat_str` instead")]
pub fn cs2lonlatstr(cs: i32, pos_char: char, neg_char: char) -> String {
    crate::functions::geoformat::centisec_to_lonlat_str(cs, pos_char, neg_char)
}

#[deprecated(since = "0.2.0", note = "use `centisec_to_time_str` instead")]
pub fn cs2timestr(cs: i32, sep: char, suppress_zero: bool) -> String {
    crate::functions::geoformat::centisec_to_time_str(cs, sep, suppress_zero)
}

#[deprecated(since = "0.2.0", note = "use `jd_et_to_utc` instead")]
pub fn jdet_to_utc(jd_et: f64, calendar: i32) -> crate::functions::time::UtcDate {
    crate::functions::time::jd_et_to_utc(jd_et, Calendar::from(calendar))
}

#[deprecated(since = "0.2.0", note = "use `jd_ut_to_utc` instead")]
pub fn jdut1_to_utc(jd_ut: f64, calendar: i32) -> crate::functions::time::UtcDate {
    crate::functions::time::jd_ut_to_utc(jd_ut, Calendar::from(calendar))
}

pub use crate::functions::aspects::AspectOrbs;
pub use crate::functions::calc::{CalcOptions, CalcStrategy};
pub use crate::functions::motion::RiseTransOptions;
pub use crate::functions::searches::SearchOptions;
