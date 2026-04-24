//! Pure-Rust implementation layer — all `pub(crate)` to the outside world.
//!
//! Domain modules in `src/` re-export from here; callers should import
//! via those modules rather than `functions::` directly.

// ── Core ephemeris ────────────────────────────────────────────────────────────
/// Planetary position calculations (`calc_ut`, `calc_many`, `CalcOptions`).
pub mod calc;
/// Engine configuration: ephemeris path, sidereal mode, topocentric origin.
pub mod config;
/// Solar and lunar eclipse searches, occultation windows.
pub mod eclipses;
/// Esbats (full-moon sabbats) and named lunations.
#[cfg(feature = "calendar-traditions")]
pub mod esbats;
/// House cusp systems (Placidus, Koch, Equal, Whole-sign …).
pub mod houses;
/// Crossings, rise/set/transit, `RiseTransOptions`, heliacal phenomena.
pub mod motion;
/// Wheel of the Year sabbats and neo-pagan seasonal festivals.
#[cfg(feature = "calendar-traditions")]
pub mod sabbats;
/// Julian Day conversion, UTC, calendar arithmetic, `CalDate`.
pub mod time;
/// Degree/radian normalisation, coordinate transforms, centisecond helpers.
pub mod utils;

// ── Observational ─────────────────────────────────────────────────────────────
/// Pheno, heliacal rising/setting, Gauquelin sector, limiting magnitude.
pub mod phenomena;

// ── Chart analysis ────────────────────────────────────────────────────────────
/// Aspect matching, `AspectOrbs`, applying/separating detection.
pub mod aspects;
/// Progressions, returns, solar arc, midpoints, Arabic parts, `AspectOrbs`.
pub mod chart;
/// Aspect and angle transit searches, `SearchOptions`.
pub mod searches;

// ── Traditional astrology ─────────────────────────────────────────────────────
/// Ba Zi four pillars, solar terms, sexagenary cycle.
pub mod chinese;
/// Essential dignities, triplicity, almuten, firdaria, annual profection.
pub mod hellenistic;
/// Medicine Wheel birth totems, Egyptian decans.
pub mod indigenous;
/// Tonalpohualli, Xiuhpohualli, Tzolkin, Haab, Calendar Round.
pub mod mesoamerican;

// ── Vedic ─────────────────────────────────────────────────────────────────────
/// Panchānga (tithi, nakshatra, yoga, karana, vara).
pub mod panchanga;
/// Jyotish chart utilities: dignities, vargas, yogas, Shadbala.
pub mod vedic;

// ── Calendars & observances ───────────────────────────────────────────────────
#[cfg(feature = "calendar-traditions")]
pub mod coptic;
/// Easter (Gregorian + Orthodox), moveable and fixed Christian feasts.
#[cfg(feature = "calendar-traditions")]
pub mod easter;
/// Geographic coordinate formatting and DMS/centisecond display helpers.
pub mod geoformat;
/// Hijri conversion, Islamic observances (Ramadan, Eid, …).
#[cfg(feature = "calendar-traditions")]
pub mod islamic;
/// Hebrew calendar, Jewish holidays, Sefirat HaOmer.
#[cfg(feature = "calendar-traditions")]
pub mod jewish;
/// Moon phases, phase info, illumination, `moon_phases_for_month`.
pub mod moon_phases;
/// Nowruz, Solar Hijri, Bahá'í calendar and holy days.
#[cfg(feature = "calendar-traditions")]
pub mod nowruz;
/// Sefirat HaOmer daily count, periods, and declaration text.
#[cfg(feature = "calendar-traditions")]
pub mod omer;
#[cfg(feature = "calendar-traditions")]
pub mod tibetan;
/// Timezone lookup and JD ↔ local time conversion.
#[cfg(feature = "timezone")]
pub mod timezone;
/// Vesak and Uposatha day calculations.
#[cfg(feature = "calendar-traditions")]
pub mod vesak;
#[cfg(feature = "calendar-traditions")]
pub mod zoroastrian;
