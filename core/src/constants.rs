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
pub const MOON: i32 = 1;
/// Mercury.
pub const MERCURY: i32 = 2;
pub const VENUS: i32 = 3;
/// Mars.
pub const MARS: i32 = 4;
pub const JUPITER: i32 = 5;
/// Saturn.
pub const SATURN: i32 = 6;
pub const URANUS: i32 = 7;
/// Neptune.
pub const NEPTUNE: i32 = 8;
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
pub const PHOLUS: i32 = 16;
/// Ceres.
pub const CERES: i32 = 17;
pub const PALLAS: i32 = 18;
/// Juno.
pub const JUNO: i32 = 19;
pub const VESTA: i32 = 20;
pub const INTP_APOG: i32 = 21;
pub const INTP_PERG: i32 = 22;

pub const NPLANETS: i32 = 23;

pub const PLMOON_OFFSET: i32 = 9000;
pub const AST_OFFSET: i32 = 10000;
pub const VARUNA: i32 = AST_OFFSET + 20000;
pub const FICT_OFFSET: i32 = 40;
pub const FICT_OFFSET_1: i32 = 39;
pub const FICT_MAX: i32 = 999;
pub const NFICT_ELEM: i32 = 15;
pub const COMET_OFFSET: i32 = 1000;
pub const NALL_NAT_POINTS: i32 = NPLANETS + NFICT_ELEM;

// Uranian / fictitious planets
pub const CUPIDO: i32 = 40;
pub const HADES: i32 = 41;
pub const ZEUS: i32 = 42;
pub const KRONOS: i32 = 43;
pub const APOLLON: i32 = 44;
pub const ADMETOS: i32 = 45;
pub const VULKANUS: i32 = 46;
pub const POSEIDON: i32 = 47;
pub const ISIS: i32 = 48;
pub const NIBIRU: i32 = 49;
pub const HARRINGTON: i32 = 50;
pub const NEPTUNE_LEVERRIER: i32 = 51;
pub const NEPTUNE_ADAMS: i32 = 52;
pub const PLUTO_LOWELL: i32 = 53;
pub const PLUTO_PICKERING: i32 = 54;
pub const VULCAN: i32 = 55;
pub const WHITE_MOON: i32 = 56;
pub const PROSERPINA: i32 = 57;
pub const WALDEMATH: i32 = 58;

pub const FIXSTAR: i32 = -10;

// ─── House cusp points ───────────────────────────────────────────────────────

pub const ASC: i32 = 0;
pub const MC: i32 = 1;
pub const ARMC: i32 = 2;
pub const VERTEX: i32 = 3;
pub const EQUASC: i32 = 4;
pub const COASC1: i32 = 5;
pub const COASC2: i32 = 6;
pub const POLASC: i32 = 7;
pub const NASCMC: i32 = 8;

// ─── Calculation flags (iflag) ───────────────────────────────────────────────

/// Use JPL ephemeris data (not yet implemented — treated as `FLG_BUILTIN`).
pub const FLG_JPL: i32 = 1;
pub const FLG_BUILTIN: i32 = 2;
/// Use Moshier approximation (not yet implemented — treated as `FLG_BUILTIN`).
pub const FLG_MOSHIER: i32 = 4;

/// Heliocentric position (default: geocentric).
pub const FLG_HELCTR: i32 = 8;
pub const FLG_TRUEPOS: i32 = 16;
/// No precession to current epoch (J2000 frame).
pub const FLG_J2000: i32 = 32;
/// Suppress nutation correction (not yet implemented — nutation always applied).
/// No nutation.
pub const FLG_NONUT: i32 = 64;
pub const FLG_SPEED3: i32 = 128;
/// Speed of the body (`pos.speed_lon`, etc.) — use this flag.
pub const FLG_SPEED: i32 = 256;
/// No gravitational deflection.
pub const FLG_NOGDEFL: i32 = 512;
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
pub const FLG_BARYCTR: i32 = 16384;
/// Topocentric position (requires `set_topo` call).
pub const FLG_TOPOCTR: i32 = 32768;
/// Orbital elements using AA (Astronomical Almanac) method.
pub const FLG_ORBEL_AA: i32 = FLG_TOPOCTR;
pub const FLG_TROPICAL: i32 = 0;
/// Sidereal position (requires `set_sid_mode` call).
pub const FLG_SIDEREAL: i32 = 65536;
/// ICRS frame (IERS reference frame).
pub const FLG_ICRS: i32 = 131072;
pub const FLG_DPSIDEPS_1980: i32 = 262144;
pub const FLG_JPLHOR: i32 = 524288;
/// Approximate JPL Horizons mode.
pub const FLG_JPLHOR_APPROX: i32 = 1048576;
pub const FLG_CENTER_BODY: i32 = 2097152;
/// Test mode for planetary moons.
pub const FLG_TEST_PLMOON: i32 = 0x80000000u32 as i32;

// ─── Sidereal mode bits ──────────────────────────────────────────────────────

pub const SIDBITS: i32 = 256;
pub const SIDBIT_ECL_T0: i32 = 256;
pub const SIDBIT_SSY_PLANE: i32 = 512;
pub const SIDBIT_USER_UT: i32 = 1024;
pub const SIDBIT_ECL_DATE: i32 = 2048;
pub const SIDBIT_NO_PREC_OFFSET: i32 = 4096;
pub const SIDBIT_PREC_ORIG: i32 = 8192;

// ─── Sidereal modes ──────────────────────────────────────────────────────────

pub const SIDM_FAGAN_BRADLEY: i32 = 0;
pub const SIDM_LAHIRI: i32 = 1;
pub const SIDM_DELUCE: i32 = 2;
pub const SIDM_RAMAN: i32 = 3;
pub const SIDM_USHASHASHI: i32 = 4;
pub const SIDM_KRISHNAMURTI: i32 = 5;
pub const SIDM_DJWHAL_KHUL: i32 = 6;
pub const SIDM_YUKTESHWAR: i32 = 7;
pub const SIDM_JN_BHASIN: i32 = 8;
pub const SIDM_BABYL_KUGLER1: i32 = 9;
pub const SIDM_BABYL_KUGLER2: i32 = 10;
pub const SIDM_BABYL_KUGLER3: i32 = 11;
pub const SIDM_BABYL_HUBER: i32 = 12;
pub const SIDM_BABYL_ETPSC: i32 = 13;
pub const SIDM_ALDEBARAN_15TAU: i32 = 14;
pub const SIDM_HIPPARCHOS: i32 = 15;
pub const SIDM_SASSANIAN: i32 = 16;
pub const SIDM_GALCENT_0SAG: i32 = 17;
pub const SIDM_J2000: i32 = 18;
pub const SIDM_J1900: i32 = 19;
pub const SIDM_B1950: i32 = 20;
pub const SIDM_SURYASIDDHANTA: i32 = 21;
pub const SIDM_SURYASIDDHANTA_MSUN: i32 = 22;
pub const SIDM_ARYABHATA: i32 = 23;
pub const SIDM_ARYABHATA_MSUN: i32 = 24;
pub const SIDM_SS_REVATI: i32 = 25;
pub const SIDM_SS_CITRA: i32 = 26;
pub const SIDM_TRUE_CITRA: i32 = 27;
pub const SIDM_TRUE_REVATI: i32 = 28;
pub const SIDM_TRUE_PUSHYA: i32 = 29;
pub const SIDM_GALCENT_RGILBRAND: i32 = 30;
pub const SIDM_GALEQU_IAU1958: i32 = 31;
pub const SIDM_GALEQU_TRUE: i32 = 32;
pub const SIDM_GALEQU_MULA: i32 = 33;
pub const SIDM_GALALIGN_MARDYKS: i32 = 34;
pub const SIDM_TRUE_MULA: i32 = 35;
pub const SIDM_GALCENT_MULA_WILHELM: i32 = 36;
pub const SIDM_ARYABHATA_522: i32 = 37;
pub const SIDM_BABYL_BRITTON: i32 = 38;
pub const SIDM_TRUE_SHEORAN: i32 = 39;
pub const SIDM_GALCENT_COCHRANE: i32 = 40;
pub const SIDM_GALEQU_FIORENZA: i32 = 41;
pub const SIDM_VALENS_MOON: i32 = 42;
pub const SIDM_LAHIRI_1940: i32 = 43;
pub const SIDM_LAHIRI_VP285: i32 = 44;
pub const SIDM_KRISHNAMURTI_VP291: i32 = 45;
pub const SIDM_LAHIRI_ICRC: i32 = 46;
pub const SIDM_USER: i32 = 255;

// ─── Eclipse / occultation types ─────────────────────────────────────────────

pub const ECL_CENTRAL: i32 = 1;
pub const ECL_NONCENTRAL: i32 = 2;
pub const ECL_TOTAL: i32 = 4;
pub const ECL_ANNULAR: i32 = 8;
pub const ECL_PARTIAL: i32 = 16;
pub const ECL_ANNULAR_TOTAL: i32 = 32;
pub const ECL_HYBRID: i32 = 32;
/// Lunar occultation of a planet (Moon passes in front of a planet).
pub const ECL_OCCULTATION: i32 = 64;
pub const ECL_PENUMBRAL: i32 = 64;
pub const ECL_ALLTYPES_SOLAR: i32 =
    ECL_CENTRAL | ECL_NONCENTRAL | ECL_TOTAL | ECL_ANNULAR | ECL_PARTIAL | ECL_ANNULAR_TOTAL;
pub const ECL_ALLTYPES_LUNAR: i32 = ECL_TOTAL | ECL_PARTIAL | ECL_PENUMBRAL;
pub const ECL_VISIBLE: i32 = 128;
pub const ECL_MAX_VISIBLE: i32 = 256;
pub const ECL_1ST_VISIBLE: i32 = 512;
pub const ECL_PARTBEG_VISIBLE: i32 = 512;
pub const ECL_2ND_VISIBLE: i32 = 1024;
pub const ECL_TOTBEG_VISIBLE: i32 = 1024;
pub const ECL_3RD_VISIBLE: i32 = 2048;
pub const ECL_TOTEND_VISIBLE: i32 = 2048;
pub const ECL_4TH_VISIBLE: i32 = 4096;
pub const ECL_PARTEND_VISIBLE: i32 = 4096;
pub const ECL_ONE_TRY: i32 = 32 * 1024;

// ─── Rise/transit flags ──────────────────────────────────────────────────────

/// Event flag: compute rise time.
pub const CALC_RISE: i32 = 1;
pub const CALC_SET: i32 = 2;
/// Event flag: upper (meridian) transit.
pub const CALC_MTRANSIT: i32 = 4;
pub const CALC_ITRANSIT: i32 = 8;
/// Rise/set for centre of disc (not limb).
pub const BIT_DISC_CENTER: i32 = 256;
/// Rise/set for bottom edge of disc.
pub const BIT_DISC_BOTTOM: i32 = 8192;
pub const BIT_GEOCTR_NO_ECL_LAT: i32 = 128;
/// No atmospheric refraction.
pub const BIT_NO_REFRACTION: i32 = 512;
pub const BIT_CIVIL_TWILIGHT: i32 = 1024;
/// Nautical twilight (sun 12° below horizon).
pub const BIT_NAUTIC_TWILIGHT: i32 = 2048;
/// Astronomical twilight (sun 18° below horizon).
pub const BIT_ASTRO_TWILIGHT: i32 = 4096;
/// Use a fixed disc size.
pub const BIT_FIXED_DISC_SIZE: i32 = 16384;
pub const BIT_FORCE_SLOW_METHOD: i32 = 32768;
/// Hindu rising method.
pub const BIT_HINDU_RISING: i32 = BIT_DISC_CENTER | BIT_NO_REFRACTION | BIT_GEOCTR_NO_ECL_LAT;

// ─── Nutation / apsis method ─────────────────────────────────────────────────

pub const NODBIT_MEAN: i32 = 1;
pub const NODBIT_OSCU: i32 = 2;
pub const NODBIT_OSCU_BAR: i32 = 4;
pub const NODBIT_FOPOINT: i32 = 256;

// ─── Azimuth/altitude calc flags ─────────────────────────────────────────────

pub const CALC_AZALT: i32 = 0;
pub const CALC_AZALT_REV: i32 = 1;

// ─── Refraction flags ────────────────────────────────────────────────────────

pub const TRUE_TO_APP: i32 = 0;
pub const APP_TO_TRUE: i32 = 1;

// ─── Split-deg flags ─────────────────────────────────────────────────────────

pub const SPLIT_DEG_ROUND_SEC: i32 = 1;
pub const SPLIT_DEG_ROUND_MIN: i32 = 2;
pub const SPLIT_DEG_ROUND_DEG: i32 = 4;
pub const SPLIT_DEG_ZODIACAL: i32 = 8;
pub const SPLIT_DEG_NAKSHATRA: i32 = 1024;
pub const SPLIT_DEG_KEEP_SIGN: i32 = 16;
pub const SPLIT_DEG_KEEP_DEG: i32 = 32;

// ─── Heliacal event types ────────────────────────────────────────────────────

pub const HELIACAL_RISING: i32 = 1;
pub const HELIACAL_SETTING: i32 = 2;
pub const EVENING_FIRST: i32 = 3;
pub const MORNING_LAST: i32 = 4;
pub const EVENING_RISING: i32 = 5;
pub const MORNING_SETTING: i32 = 6;
pub const ACRONYCHAL_RISING: i32 = 7;
pub const COSMICAL_SETTING: i32 = 7;
pub const ACRONYCHAL_SETTING: i32 = 8;
pub const COSMICAL_RISING: i32 = 8;

// ─── Heliacal flags ──────────────────────────────────────────────────────────

pub const HELFLAG_LONG_SEARCH: i32 = 128;
pub const HELFLAG_HIGH_PRECISION: i32 = 256;
pub const HELFLAG_OPTICAL_PARAMS: i32 = 512;
pub const HELFLAG_NO_DETAILS: i32 = 1024;
pub const HELFLAG_SEARCH_1_PERIOD: i32 = 1 << 11;
pub const HELFLAG_VISLIM_DARK: i32 = 1 << 12;
pub const HELFLAG_VISLIM_NOMOON: i32 = 1 << 13;
pub const HELFLAG_VISLIM_PHOTOPIC: i32 = 1 << 14;
pub const HELFLAG_VISLIM_SCOTOPIC: i32 = 1 << 15;
pub const HELFLAG_AV: i32 = 1 << 16;
pub const HELFLAG_AVKIND_VR: i32 = 1 << 16;
pub const HELFLAG_AVKIND_PTO: i32 = 1 << 17;
pub const HELFLAG_AVKIND_MIN7: i32 = 1 << 18;
pub const HELFLAG_AVKIND_MIN9: i32 = 1 << 19;

// ─── Tidal acceleration constants ────────────────────────────────────────────

pub const TIDAL_DE200: f64 = -23.8946;
pub const TIDAL_DE403: f64 = -25.580;
pub const TIDAL_DE404: f64 = -25.580;
pub const TIDAL_DE405: f64 = -25.826;
pub const TIDAL_DE406: f64 = -25.826;
pub const TIDAL_DE421: f64 = -25.85;
pub const TIDAL_DE422: f64 = -25.85;
pub const TIDAL_DE430: f64 = -25.82;
pub const TIDAL_DE431: f64 = -25.82;
pub const TIDAL_26: f64 = -26.0;
pub const TIDAL_STEPHENSON_2016: f64 = -25.85;
pub const TIDAL_DEFAULT: f64 = TIDAL_DE431;
pub const TIDAL_AUTOMATIC: f64 = 999999.0;
pub const TIDAL_MOSEPH: f64 = TIDAL_DE404;
pub const TIDAL_SWIEPH: f64 = TIDAL_DEFAULT;
pub const TIDAL_JPLEPH_DE406: f64 = TIDAL_DE406;

// ─── File name strings ───────────────────────────────────────────────────────

pub const FNAME_DE200: &str = "de200.eph";
pub const FNAME_DE403: &str = "de403.eph";
pub const FNAME_DE404: &str = "de404.eph";
pub const FNAME_DE405: &str = "de405.eph";
pub const FNAME_DE406: &str = "de406.eph";
pub const FNAME_DE431: &str = "de431.eph";
pub const FNAME_DFT: &str = "de431.eph";
pub const FNAME_DFT2: &str = "de406.eph";
pub const STARFILE_OLD: &str = "fixstars.cat";
pub const STARFILE: &str = "sefstars.txt";
pub const ASTNAMFILE: &str = "seasnam.txt";
pub const FICTFILE: &str = "seorbel.txt";

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
