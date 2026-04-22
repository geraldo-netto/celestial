//! Planetary positions, fixed stars, ayanamsa, and configuration.
//!
//! # Examples
//!
//! ```
//! use celestial_core::position::{calc_ut, CalcOptions, CalcStrategy};
//! use celestial_core::body::{Body, CalcFlags};
//!
//! // Single body
//! let sun = calc_ut(2_451_545.0, Body::SUN, CalcFlags::BUILTIN).unwrap();
//! println!("Sun lon = {:.4}°", sun.lon);
//!
//! // Multiple bodies with auto parallel strategy
//! let results = CalcOptions::ut(2_451_545.0, CalcFlags::BUILTIN)
//!     .bodies(&[Body::SUN, Body::MOON, Body::MERCURY])
//!     .get_many();
//! ```

pub use crate::functions::calc::{
    calc, calc_many, calc_pctr, calc_ut, calc_ut_many, close, fixstar, fixstar2, fixstar2_mag,
    fixstar2_ut, fixstar_mag, fixstar_ut, get_orbital_elements, nod_aps, nod_aps_ut,
    orbit_max_min_true_distance, CalcOptions, CalcStrategy, FixStarPos, NodAps, OrbitalDistances,
    OrbitalElements, PlanetPos,
};
pub use crate::functions::config::{
    ayanamsa, ayanamsa_ex, ayanamsa_ex_ut, ayanamsa_name, ayanamsa_ut, current_file_data,
    library_path, planet_name, set_delta_t_userdef, set_ephe_path, set_jpl_file, set_lapse_rate,
    set_sid_mode, set_tid_acc, set_topo, tid_acc, version, CurrentFileData,
};
pub use crate::functions::phenomena::{
    gauquelin_sector, heliacal_pheno_ut, heliacal_ut, pheno, pheno_ut, vis_limit_mag,
};
