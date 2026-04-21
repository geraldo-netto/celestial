//! All safe wrapper sub-modules.

pub mod calc;
pub mod config;
pub mod eclipses;
pub mod esbats;
pub mod houses;
pub mod motion; // crossings + rise/set/transit
pub mod sabbats;
pub mod time; // calendar, JD, UTC, display helpers
pub mod utils;

pub mod phenomena; // pheno, heliacal, gauquelin sector

// ─── Helper library (ex-swephelp) ─────────────────────────────────────────────
pub mod aspects;
pub mod chart;
pub mod chinese; // Phase 6: Ba Zi, solar terms
pub mod easter;
pub mod geoformat;
pub mod hellenistic; // Phase 5: terms, decans, dignity, almuten, firdaria
pub mod indigenous; // Phase 8: Medicine Wheel, Egyptian decans
pub mod islamic;
pub mod jewish;
pub mod mesoamerican; // Phase 7: Tonalpohualli, Tzolkin, Haab, Calendar Round
pub mod moon_phases;
pub mod nowruz;
pub mod omer;
pub mod panchanga;
pub mod searches;
pub mod timezone;
pub mod vedic; // complete chart utilities
pub mod vesak;
