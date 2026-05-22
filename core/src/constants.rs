#![allow(dead_code)]
//! Constants exported by the Swiss Ephemeris library.
//!
//! These mirror the `#define` values from `swephexp.h` and the constants
//! registered in `pycelestial.c` via `PyModule_AddIntConstant`.

// ─── Calendar flags ──────────────────────────────────────────────────────────

/// Julian calendar flag — pass to `julday`, `revjul`, etc.
pub const JUL_CAL: i32 = 0;
/// Gregorian calendar flag — pass to `julday`, `revjul`, etc.
pub const GREG_CAL: i32 = 1;

// ─── Body numbers ────────────────────────────────────────────────────────────

/// Ecliptic obliquity and nutation (special body index).
pub const ECL_NUT: i32 = -1;
/// The Sun.
pub const SUN: i32 = 0;
/// The Moon.
pub const MOON: i32 = 1;
/// Mercury.
pub const MERCURY: i32 = 2;
/// Venus.
pub const VENUS: i32 = 3;
/// Mars.
pub const MARS: i32 = 4;
/// Jupiter.
pub const JUPITER: i32 = 5;
/// Saturn.
pub const SATURN: i32 = 6;
/// Uranus.
pub const URANUS: i32 = 7;
/// Neptune.
pub const NEPTUNE: i32 = 8;
/// Pluto.
pub const PLUTO: i32 = 9;
/// Mean lunar node (Rahu / Ketu axis, mean position).
pub const MEAN_NODE: i32 = 10;
/// True (osculating) lunar node.
pub const TRUE_NODE: i32 = 11;
/// Mean lunar apogee (Black Moon Lilith, mean).
pub const MEAN_APOG: i32 = 12;
/// Osculating lunar apogee (True Lilith).
pub const OSCU_APOG: i32 = 13;
/// Earth (heliocentric calculations).
pub const EARTH: i32 = 14;
/// Chiron.
pub const CHIRON: i32 = 15;
/// Pholus.
pub const PHOLUS: i32 = 16;
/// Ceres.
pub const CERES: i32 = 17;
/// Pallas.
pub const PALLAS: i32 = 18;
/// Juno.
pub const JUNO: i32 = 19;
/// Vesta.
pub const VESTA: i32 = 20;
/// Interpolated lunar apogee.
pub const INTP_APOG: i32 = 21;
/// Interpolated lunar perigee.
pub const INTP_PERG: i32 = 22;

/// Number of main planetary body numbers (0..=22).
pub const NPLANETS: i32 = 23;

/// Body-number offset for planetary moons.
pub const PLMOON_OFFSET: i32 = 9000;
/// Body-number offset for numbered asteroids.
pub const AST_OFFSET: i32 = 10000;
/// Varuna (asteroid 20000), as a body number.
pub const VARUNA: i32 = AST_OFFSET + 20000;
/// Body-number offset for fictitious bodies.
pub const FICT_OFFSET: i32 = 40;
/// Body-number offset minus one for fictitious bodies.
pub const FICT_OFFSET_1: i32 = 39;
/// Maximum body number for fictitious bodies.
pub const FICT_MAX: i32 = 999;
/// Number of fictitious orbital elements.
pub const NFICT_ELEM: i32 = 15;
/// Body-number offset for comets.
pub const COMET_OFFSET: i32 = 1000;
/// Total count of natal points (planets plus fictitious elements).
pub const NALL_NAT_POINTS: i32 = NPLANETS + NFICT_ELEM;

// Uranian / fictitious planets
/// Cupido (Uranian / fictitious planet).
pub const CUPIDO: i32 = 40;
/// Hades (Uranian / fictitious planet).
pub const HADES: i32 = 41;
/// Zeus (Uranian / fictitious planet).
pub const ZEUS: i32 = 42;
/// Kronos (Uranian / fictitious planet).
pub const KRONOS: i32 = 43;
/// Apollon (Uranian / fictitious planet).
pub const APOLLON: i32 = 44;
/// Admetos (Uranian / fictitious planet).
pub const ADMETOS: i32 = 45;
/// Vulkanus (Uranian / fictitious planet).
pub const VULKANUS: i32 = 46;
/// Poseidon (Uranian / fictitious planet).
pub const POSEIDON: i32 = 47;
/// Isis (fictitious planet).
pub const ISIS: i32 = 48;
/// Nibiru (fictitious planet).
pub const NIBIRU: i32 = 49;
/// Harrington (fictitious planet).
pub const HARRINGTON: i32 = 50;
/// Neptune as predicted by Le Verrier (fictitious planet).
pub const NEPTUNE_LEVERRIER: i32 = 51;
/// Neptune as predicted by Adams (fictitious planet).
pub const NEPTUNE_ADAMS: i32 = 52;
/// Pluto as predicted by Lowell (fictitious planet).
pub const PLUTO_LOWELL: i32 = 53;
/// Pluto as predicted by Pickering (fictitious planet).
pub const PLUTO_PICKERING: i32 = 54;
/// Vulcan (fictitious planet).
pub const VULCAN: i32 = 55;
/// White Moon / Selena (fictitious body).
pub const WHITE_MOON: i32 = 56;
/// Proserpina (fictitious planet).
pub const PROSERPINA: i32 = 57;
/// Waldemath (fictitious second moon).
pub const WALDEMATH: i32 = 58;

/// Fixed-star body number (use with `fixstar` functions).
pub const FIXSTAR: i32 = -10;

// ─── House cusp points ───────────────────────────────────────────────────────

/// Ascendant (index into the `ascmc` array).
pub const ASC: i32 = 0;
/// Midheaven / Medium Coeli (index into the `ascmc` array).
pub const MC: i32 = 1;
/// Right ascension of the MC (index into the `ascmc` array).
pub const ARMC: i32 = 2;
/// Vertex (index into the `ascmc` array).
pub const VERTEX: i32 = 3;
/// Equatorial ascendant / East Point (index into the `ascmc` array).
pub const EQUASC: i32 = 4;
/// Co-ascendant (Walter Koch; index into the `ascmc` array).
pub const COASC1: i32 = 5;
/// Co-ascendant (Michael Munkasey; index into the `ascmc` array).
pub const COASC2: i32 = 6;
/// Polar ascendant (M. Munkasey; index into the `ascmc` array).
pub const POLASC: i32 = 7;
/// Number of entries in the `ascmc` array.
pub const NASCMC: i32 = 8;

// ─── Calculation flags (iflag) ───────────────────────────────────────────────

/// Use JPL ephemeris data (not yet implemented — treated as `FLG_BUILTIN`).
pub const FLG_JPL: i32 = 1;
/// Use the built-in Swiss Ephemeris data.
pub const FLG_BUILTIN: i32 = 2;
/// Use Moshier approximation (not yet implemented — treated as `FLG_BUILTIN`).
pub const FLG_MOSHIER: i32 = 4;

/// Heliocentric position (default: geocentric).
pub const FLG_HELCTR: i32 = 8;
/// True geometric position (no light-time correction).
pub const FLG_TRUEPOS: i32 = 16;
/// No precession to current epoch (J2000 frame).
pub const FLG_J2000: i32 = 32;
/// Suppress nutation correction (not yet implemented — nutation always applied).
/// No nutation.
pub const FLG_NONUT: i32 = 64;
/// High-precision speed via three-position differentiation.
pub const FLG_SPEED3: i32 = 128;
/// Speed of the body (`pos.speed_lon`, etc.) — use this flag.
pub const FLG_SPEED: i32 = 256;
/// No gravitational deflection.
pub const FLG_NOGDEFL: i32 = 512;
/// No annual aberration of light.
pub const FLG_NOABERR: i32 = 1024;
/// Astrometric position (no deflection, no aberration).
pub const FLG_ASTROMETRIC: i32 = FLG_NOABERR | FLG_NOGDEFL;
/// Equatorial output (right ascension + declination).
pub const FLG_EQUATORIAL: i32 = 2048;
/// Return Cartesian XYZ coordinates (not yet implemented).
pub const FLG_XYZ: i32 = 4096;
/// Return coordinates in radians (not yet implemented — degrees always returned).
/// Output in radians instead of degrees.
pub const FLG_RADIANS: i32 = 8192;
/// Barycentric position (relative to solar-system barycentre).
pub const FLG_BARYCTR: i32 = 16384;
/// Topocentric position (requires `set_topo` call).
pub const FLG_TOPOCTR: i32 = 32768;
/// Orbital elements using AA (Astronomical Almanac) method.
pub const FLG_ORBEL_AA: i32 = FLG_TOPOCTR;
/// Tropical zodiac position (default).
pub const FLG_TROPICAL: i32 = 0;
/// Sidereal position (requires `set_sid_mode` call).
pub const FLG_SIDEREAL: i32 = 65536;
/// ICRS frame (IERS reference frame).
pub const FLG_ICRS: i32 = 131072;
/// Use IAU 1980 nutation series (delta-psi / delta-epsilon).
pub const FLG_DPSIDEPS_1980: i32 = 262144;
/// JPL Horizons mode (exact, using daily IERS corrections).
pub const FLG_JPLHOR: i32 = 524288;
/// Approximate JPL Horizons mode.
pub const FLG_JPLHOR_APPROX: i32 = 1048576;
/// Position relative to a central body other than the Sun (e.g. a planet centre).
pub const FLG_CENTER_BODY: i32 = 2097152;
/// Test mode for planetary moons.
pub const FLG_TEST_PLMOON: i32 = 0x80000000u32 as i32;

// ─── Sidereal mode bits ──────────────────────────────────────────────────────

/// Mask for the sidereal-mode option bits.
pub const SIDBITS: i32 = 256;
/// Project onto the ecliptic of the reference date t0.
pub const SIDBIT_ECL_T0: i32 = 256;
/// Project onto the mean plane of the solar system.
pub const SIDBIT_SSY_PLANE: i32 = 512;
/// Interpret a user-defined ayanamsa value as given in UT.
pub const SIDBIT_USER_UT: i32 = 1024;
/// Project onto the ecliptic of date.
pub const SIDBIT_ECL_DATE: i32 = 2048;
/// Do not apply the precession offset to the ayanamsa.
pub const SIDBIT_NO_PREC_OFFSET: i32 = 4096;
/// Use the original (pre-correction) precession model.
pub const SIDBIT_PREC_ORIG: i32 = 8192;

// ─── Sidereal modes ──────────────────────────────────────────────────────────

/// Fagan/Bradley ayanamsa.
pub const SIDM_FAGAN_BRADLEY: i32 = 0;
/// Lahiri (Chitrapaksha) ayanamsa.
pub const SIDM_LAHIRI: i32 = 1;
/// De Luce ayanamsa.
pub const SIDM_DELUCE: i32 = 2;
/// Raman ayanamsa.
pub const SIDM_RAMAN: i32 = 3;
/// Usha/Shashi ayanamsa.
pub const SIDM_USHASHASHI: i32 = 4;
/// Krishnamurti (KP) ayanamsa.
pub const SIDM_KRISHNAMURTI: i32 = 5;
/// Djwhal Khul ayanamsa.
pub const SIDM_DJWHAL_KHUL: i32 = 6;
/// Yukteshwar ayanamsa.
pub const SIDM_YUKTESHWAR: i32 = 7;
/// J. N. Bhasin ayanamsa.
pub const SIDM_JN_BHASIN: i32 = 8;
/// Babylonian (Kugler 1) ayanamsa.
pub const SIDM_BABYL_KUGLER1: i32 = 9;
/// Babylonian (Kugler 2) ayanamsa.
pub const SIDM_BABYL_KUGLER2: i32 = 10;
/// Babylonian (Kugler 3) ayanamsa.
pub const SIDM_BABYL_KUGLER3: i32 = 11;
/// Babylonian (Huber) ayanamsa.
pub const SIDM_BABYL_HUBER: i32 = 12;
/// Babylonian (eta Piscium / Mercury) ayanamsa.
pub const SIDM_BABYL_ETPSC: i32 = 13;
/// Aldebaran at 15° Taurus ayanamsa.
pub const SIDM_ALDEBARAN_15TAU: i32 = 14;
/// Hipparchos ayanamsa.
pub const SIDM_HIPPARCHOS: i32 = 15;
/// Sassanian ayanamsa.
pub const SIDM_SASSANIAN: i32 = 16;
/// Galactic centre at 0° Sagittarius ayanamsa.
pub const SIDM_GALCENT_0SAG: i32 = 17;
/// J2000 reference-frame ayanamsa.
pub const SIDM_J2000: i32 = 18;
/// J1900 reference-frame ayanamsa.
pub const SIDM_J1900: i32 = 19;
/// B1950 reference-frame ayanamsa.
pub const SIDM_B1950: i32 = 20;
/// Suryasiddhanta ayanamsa.
pub const SIDM_SURYASIDDHANTA: i32 = 21;
/// Suryasiddhanta (mean Sun) ayanamsa.
pub const SIDM_SURYASIDDHANTA_MSUN: i32 = 22;
/// Aryabhata ayanamsa.
pub const SIDM_ARYABHATA: i32 = 23;
/// Aryabhata (mean Sun) ayanamsa.
pub const SIDM_ARYABHATA_MSUN: i32 = 24;
/// SS Revati ayanamsa.
pub const SIDM_SS_REVATI: i32 = 25;
/// SS Citra ayanamsa.
pub const SIDM_SS_CITRA: i32 = 26;
/// True Chitra (Spica fixed at 0° Libra) ayanamsa.
pub const SIDM_TRUE_CITRA: i32 = 27;
/// True Revati ayanamsa.
pub const SIDM_TRUE_REVATI: i32 = 28;
/// True Pushya (delta Cancri) ayanamsa.
pub const SIDM_TRUE_PUSHYA: i32 = 29;
/// Galactic centre (Gil Brand) ayanamsa.
pub const SIDM_GALCENT_RGILBRAND: i32 = 30;
/// Galactic equator (IAU 1958) ayanamsa.
pub const SIDM_GALEQU_IAU1958: i32 = 31;
/// Galactic equator (true) ayanamsa.
pub const SIDM_GALEQU_TRUE: i32 = 32;
/// Galactic equator at Mula ayanamsa.
pub const SIDM_GALEQU_MULA: i32 = 33;
/// Galactic alignment (Mardyks) ayanamsa.
pub const SIDM_GALALIGN_MARDYKS: i32 = 34;
/// True Mula (Wilhelm) ayanamsa.
pub const SIDM_TRUE_MULA: i32 = 35;
/// Galactic centre at Mula (Wilhelm) ayanamsa.
pub const SIDM_GALCENT_MULA_WILHELM: i32 = 36;
/// Aryabhata 522 ayanamsa.
pub const SIDM_ARYABHATA_522: i32 = 37;
/// Babylonian (Britton) ayanamsa.
pub const SIDM_BABYL_BRITTON: i32 = 38;
/// True Sheoran ayanamsa.
pub const SIDM_TRUE_SHEORAN: i32 = 39;
/// Galactic centre (Cochrane) ayanamsa.
pub const SIDM_GALCENT_COCHRANE: i32 = 40;
/// Galactic equator (Fiorenza) ayanamsa.
pub const SIDM_GALEQU_FIORENZA: i32 = 41;
/// Vettius Valens (Moon) ayanamsa.
pub const SIDM_VALENS_MOON: i32 = 42;
/// Lahiri 1940 ayanamsa.
pub const SIDM_LAHIRI_1940: i32 = 43;
/// Lahiri (VP285) ayanamsa.
pub const SIDM_LAHIRI_VP285: i32 = 44;
/// Krishnamurti (VP291) ayanamsa.
pub const SIDM_KRISHNAMURTI_VP291: i32 = 45;
/// Lahiri (ICRC) ayanamsa.
pub const SIDM_LAHIRI_ICRC: i32 = 46;
/// User-defined ayanamsa (set via `set_sid_mode`).
pub const SIDM_USER: i32 = 255;

// ─── Eclipse / occultation types ─────────────────────────────────────────────

/// Central eclipse.
pub const ECL_CENTRAL: i32 = 1;
/// Non-central eclipse.
pub const ECL_NONCENTRAL: i32 = 2;
/// Total eclipse.
pub const ECL_TOTAL: i32 = 4;
/// Annular eclipse.
pub const ECL_ANNULAR: i32 = 8;
/// Partial eclipse.
pub const ECL_PARTIAL: i32 = 16;
/// Annular-total (hybrid) eclipse.
pub const ECL_ANNULAR_TOTAL: i32 = 32;
/// Hybrid (annular-total) eclipse.
pub const ECL_HYBRID: i32 = 32;
/// Lunar occultation of a planet (Moon passes in front of a planet).
pub const ECL_OCCULTATION: i32 = 64;
/// Penumbral lunar eclipse.
pub const ECL_PENUMBRAL: i32 = 64;
/// Mask matching all solar-eclipse types.
pub const ECL_ALLTYPES_SOLAR: i32 =
    ECL_CENTRAL | ECL_NONCENTRAL | ECL_TOTAL | ECL_ANNULAR | ECL_PARTIAL | ECL_ANNULAR_TOTAL;
/// Mask matching all lunar-eclipse types.
pub const ECL_ALLTYPES_LUNAR: i32 = ECL_TOTAL | ECL_PARTIAL | ECL_PENUMBRAL;
/// Eclipse is visible from the given location.
pub const ECL_VISIBLE: i32 = 128;
/// Time of maximum eclipse is visible.
pub const ECL_MAX_VISIBLE: i32 = 256;
/// First contact is visible.
pub const ECL_1ST_VISIBLE: i32 = 512;
/// Beginning of partial phase is visible.
pub const ECL_PARTBEG_VISIBLE: i32 = 512;
/// Second contact is visible.
pub const ECL_2ND_VISIBLE: i32 = 1024;
/// Beginning of total phase is visible.
pub const ECL_TOTBEG_VISIBLE: i32 = 1024;
/// Third contact is visible.
pub const ECL_3RD_VISIBLE: i32 = 2048;
/// End of total phase is visible.
pub const ECL_TOTEND_VISIBLE: i32 = 2048;
/// Fourth contact is visible.
pub const ECL_4TH_VISIBLE: i32 = 4096;
/// End of partial phase is visible.
pub const ECL_PARTEND_VISIBLE: i32 = 4096;
/// Search only a single period for the eclipse.
pub const ECL_ONE_TRY: i32 = 32 * 1024;

// ─── Rise/transit flags ──────────────────────────────────────────────────────

/// Event flag: compute rise time.
pub const CALC_RISE: i32 = 1;
/// Event flag: compute set time.
pub const CALC_SET: i32 = 2;
/// Event flag: upper (meridian) transit.
pub const CALC_MTRANSIT: i32 = 4;
/// Event flag: lower (anti-meridian) transit.
pub const CALC_ITRANSIT: i32 = 8;
/// Rise/set for centre of disc (not limb).
pub const BIT_DISC_CENTER: i32 = 256;
/// Rise/set for bottom edge of disc.
pub const BIT_DISC_BOTTOM: i32 = 8192;
/// Use geocentric position, ignoring the body's ecliptic latitude.
pub const BIT_GEOCTR_NO_ECL_LAT: i32 = 128;
/// No atmospheric refraction.
pub const BIT_NO_REFRACTION: i32 = 512;
/// Civil twilight (sun 6° below horizon).
pub const BIT_CIVIL_TWILIGHT: i32 = 1024;
/// Nautical twilight (sun 12° below horizon).
pub const BIT_NAUTIC_TWILIGHT: i32 = 2048;
/// Astronomical twilight (sun 18° below horizon).
pub const BIT_ASTRO_TWILIGHT: i32 = 4096;
/// Use a fixed disc size.
pub const BIT_FIXED_DISC_SIZE: i32 = 16384;
/// Force the slow (iterative) rise/set computation method.
pub const BIT_FORCE_SLOW_METHOD: i32 = 32768;
/// Hindu rising method.
pub const BIT_HINDU_RISING: i32 = BIT_DISC_CENTER | BIT_NO_REFRACTION | BIT_GEOCTR_NO_ECL_LAT;

// ─── Nutation / apsis method ─────────────────────────────────────────────────

/// Compute mean nodes and apsides.
pub const NODBIT_MEAN: i32 = 1;
/// Compute osculating nodes and apsides.
pub const NODBIT_OSCU: i32 = 2;
/// Compute osculating nodes and apsides, barycentric.
pub const NODBIT_OSCU_BAR: i32 = 4;
/// Return the second focal point instead of the aphelion.
pub const NODBIT_FOPOINT: i32 = 256;

// ─── Azimuth/altitude calc flags ─────────────────────────────────────────────

/// Convert ecliptic/equatorial coordinates to azimuth and altitude.
pub const CALC_AZALT: i32 = 0;
/// Convert azimuth and altitude back to ecliptic/equatorial coordinates.
pub const CALC_AZALT_REV: i32 = 1;

// ─── Refraction flags ────────────────────────────────────────────────────────

/// Convert true altitude to apparent altitude (apply refraction).
pub const TRUE_TO_APP: i32 = 0;
/// Convert apparent altitude to true altitude (remove refraction).
pub const APP_TO_TRUE: i32 = 1;

// ─── Split-deg flags ─────────────────────────────────────────────────────────

/// Round the split position to the nearest arcsecond.
pub const SPLIT_DEG_ROUND_SEC: i32 = 1;
/// Round the split position to the nearest arcminute.
pub const SPLIT_DEG_ROUND_MIN: i32 = 2;
/// Round the split position to the nearest degree.
pub const SPLIT_DEG_ROUND_DEG: i32 = 4;
/// Split into zodiac sign plus degrees within the sign.
pub const SPLIT_DEG_ZODIACAL: i32 = 8;
/// Split into nakshatra plus degrees within the nakshatra.
pub const SPLIT_DEG_NAKSHATRA: i32 = 1024;
/// Do not roll the sign forward when rounding up.
pub const SPLIT_DEG_KEEP_SIGN: i32 = 16;
/// Do not roll the degree forward when rounding up.
pub const SPLIT_DEG_KEEP_DEG: i32 = 32;

// ─── Heliacal event types ────────────────────────────────────────────────────

/// Heliacal rising (morning first visibility).
pub const HELIACAL_RISING: i32 = 1;
/// Heliacal setting (evening last visibility).
pub const HELIACAL_SETTING: i32 = 2;
/// Evening first (first evening visibility of an inner planet).
pub const EVENING_FIRST: i32 = 3;
/// Morning last (last morning visibility of an inner planet).
pub const MORNING_LAST: i32 = 4;
/// Evening rising (object rises at sunset).
pub const EVENING_RISING: i32 = 5;
/// Morning setting (object sets at sunrise).
pub const MORNING_SETTING: i32 = 6;
/// Acronychal rising (object rises as the Sun sets).
pub const ACRONYCHAL_RISING: i32 = 7;
/// Cosmical setting (object sets as the Sun rises).
pub const COSMICAL_SETTING: i32 = 7;
/// Acronychal setting (object sets as the Sun sets).
pub const ACRONYCHAL_SETTING: i32 = 8;
/// Cosmical rising (object rises as the Sun rises).
pub const COSMICAL_RISING: i32 = 8;

// ─── Heliacal flags ──────────────────────────────────────────────────────────

/// Extend the search range for the heliacal event.
pub const HELFLAG_LONG_SEARCH: i32 = 128;
/// Use high-precision computation for heliacal events.
pub const HELFLAG_HIGH_PRECISION: i32 = 256;
/// Use the supplied optical instrument parameters.
pub const HELFLAG_OPTICAL_PARAMS: i32 = 512;
/// Skip computation of detailed result fields.
pub const HELFLAG_NO_DETAILS: i32 = 1024;
/// Search only a single synodic period.
pub const HELFLAG_SEARCH_1_PERIOD: i32 = 1 << 11;
/// Visibility limit assuming dark sky.
pub const HELFLAG_VISLIM_DARK: i32 = 1 << 12;
/// Visibility limit ignoring moonlight.
pub const HELFLAG_VISLIM_NOMOON: i32 = 1 << 13;
/// Visibility limit using photopic (daylight) vision.
pub const HELFLAG_VISLIM_PHOTOPIC: i32 = 1 << 14;
/// Visibility limit using scotopic (night) vision.
pub const HELFLAG_VISLIM_SCOTOPIC: i32 = 1 << 15;
/// Return the arcus visionis (AV) value.
pub const HELFLAG_AV: i32 = 1 << 16;
/// Arcus visionis kind: VR (visual range).
pub const HELFLAG_AVKIND_VR: i32 = 1 << 16;
/// Arcus visionis kind: Ptolemy.
pub const HELFLAG_AVKIND_PTO: i32 = 1 << 17;
/// Arcus visionis kind: minimum-7 method.
pub const HELFLAG_AVKIND_MIN7: i32 = 1 << 18;
/// Arcus visionis kind: minimum-9 method.
pub const HELFLAG_AVKIND_MIN9: i32 = 1 << 19;

// ─── Tidal acceleration constants ────────────────────────────────────────────

/// Tidal acceleration of the Moon for the DE200 ephemeris ("/cy²).
pub const TIDAL_DE200: f64 = -23.8946;
/// Tidal acceleration of the Moon for the DE403 ephemeris ("/cy²).
pub const TIDAL_DE403: f64 = -25.580;
/// Tidal acceleration of the Moon for the DE404 ephemeris ("/cy²).
pub const TIDAL_DE404: f64 = -25.580;
/// Tidal acceleration of the Moon for the DE405 ephemeris ("/cy²).
pub const TIDAL_DE405: f64 = -25.826;
/// Tidal acceleration of the Moon for the DE406 ephemeris ("/cy²).
pub const TIDAL_DE406: f64 = -25.826;
/// Tidal acceleration of the Moon for the DE421 ephemeris ("/cy²).
pub const TIDAL_DE421: f64 = -25.85;
/// Tidal acceleration of the Moon for the DE422 ephemeris ("/cy²).
pub const TIDAL_DE422: f64 = -25.85;
/// Tidal acceleration of the Moon for the DE430 ephemeris ("/cy²).
pub const TIDAL_DE430: f64 = -25.82;
/// Tidal acceleration of the Moon for the DE431 ephemeris ("/cy²).
pub const TIDAL_DE431: f64 = -25.82;
/// Tidal acceleration of -26.0 "/cy².
pub const TIDAL_26: f64 = -26.0;
/// Tidal acceleration of the Moon for the Stephenson 2016 model ("/cy²).
pub const TIDAL_STEPHENSON_2016: f64 = -25.85;
/// Default tidal acceleration (DE431).
pub const TIDAL_DEFAULT: f64 = TIDAL_DE431;
/// Sentinel selecting tidal acceleration automatically from the ephemeris in use.
pub const TIDAL_AUTOMATIC: f64 = 999999.0;
/// Tidal acceleration used with the Moshier ephemeris.
pub const TIDAL_MOSEPH: f64 = TIDAL_DE404;
/// Tidal acceleration used with the Swiss Ephemeris.
pub const TIDAL_SWIEPH: f64 = TIDAL_DEFAULT;
/// Tidal acceleration used with the JPL DE406 ephemeris.
pub const TIDAL_JPLEPH_DE406: f64 = TIDAL_DE406;

// ─── File name strings ───────────────────────────────────────────────────────

/// File name of the JPL DE200 ephemeris.
pub const FNAME_DE200: &str = "de200.eph";
/// File name of the JPL DE403 ephemeris.
pub const FNAME_DE403: &str = "de403.eph";
/// File name of the JPL DE404 ephemeris.
pub const FNAME_DE404: &str = "de404.eph";
/// File name of the JPL DE405 ephemeris.
pub const FNAME_DE405: &str = "de405.eph";
/// File name of the JPL DE406 ephemeris.
pub const FNAME_DE406: &str = "de406.eph";
/// File name of the JPL DE431 ephemeris.
pub const FNAME_DE431: &str = "de431.eph";
/// Default JPL ephemeris file name.
pub const FNAME_DFT: &str = "de431.eph";
/// Secondary default JPL ephemeris file name.
pub const FNAME_DFT2: &str = "de406.eph";
/// Legacy fixed-star catalogue file name.
pub const STARFILE_OLD: &str = "fixstars.cat";
/// Fixed-star catalogue file name.
pub const STARFILE: &str = "sefstars.txt";
/// Asteroid-name file name.
pub const ASTNAMFILE: &str = "seasnam.txt";
/// Fictitious-bodies orbital-element file name.
pub const FICTFILE: &str = "seorbel.txt";

/// Default search path for ephemeris data files.
pub const EPHE_PATH: &str = "/usr/share/celestial:/usr/local/share/celestial";

// ─── Physical constants ───────────────────────────────────────────────────────

/// Astronomical Unit in kilometres.
pub const AU_KM: f64 = 149_597_870.7;
/// Earth's equatorial radius in kilometres.
pub const EARTH_RADIUS_KM: f64 = 6_378.137;
/// Solar parallax in arcseconds.
pub const SOLAR_PARALLAX: f64 = 8.794_148;
/// Mean obliquity of the ecliptic at J2000 (degrees).
pub const OBLIQUITY_J2000: f64 = 23.439_291_111;
/// Arcseconds per radian.
pub const ARCSEC_PER_RAD: f64 = 206_264.806;
/// Radians per arcsecond.
pub const RAD_PER_ARCSEC: f64 = 1.0 / 206_264.806;
