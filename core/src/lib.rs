//! # celestial-core
//!
//! Pure-Rust astronomical engine — no C compiler, no data files, no external dependencies.
//!
//! ## Quick start
//!
//! ```no_run
//! use celestial_core::body::{Body, CalcFlags};
//! use celestial_core::time::jdnow;
//! use celestial_core::position::calc_ut;
//! use celestial_core::JulianDay;
//!
//! let sun = calc_ut(JulianDay::new(jdnow()), Body::SUN, CalcFlags::BUILTIN | CalcFlags::SPEED).unwrap();
//! println!("Sun lon = {:.4}°", sun.lon);
//! ```
//!
//! ## Module layout
//!
//! | Module | Contents |
//! |---|---|
//! | [`body`] | [`Body`], [`CalcFlags`], [`HouseSystem`], [`SiderealMode`], [`Calendar`] |
//! | [`constants`] | Numeric constants (body indices, flags, sidereal modes) |
//! | [`position`] | `calc_ut`, `calc_many`, `CalcOptions`, fixed stars, ayanamsa |
//! | [`time`] | Julian day, UTC, calendar conversion |
//! | [`houses`](mod@houses) | House cusp systems |
//! | [`motion`] | Crossings, rise/set/transit, eclipses, `RiseTransOptions`, `SearchOptions` |
//! | [`moon`] | Phase, illumination, principal phases, esbats, sabbats |
//! | [`chart`] | Aspects, `AspectOrbs`, progressions, returns, Arabic parts, traditions |
//! | [`vedic`] | Jyotish helpers, Panchānga |
//! | [`geo`] | Coordinate formatting, timezones |
//! | [`calendar`] | Hebrew, Christian, Islamic, Hindu, Buddhist, Persian, Celtic |
//!
//! All public symbols are re-exported at the crate root, so
//! `use celestial_core::calc_ut` continues to work alongside
//! the preferred `use celestial_core::position::calc_ut`.
//!
//! A [`prelude`] module re-exports the most-used items for convenience.

// DOC-1: every public item carries a doc comment; this lint keeps it that
// way (CI runs the feature matrix under `-D warnings`, so a regression fails).
#![warn(missing_docs)]

// ── Crate-private physics engine (unchanged) ──────────────────────────────────
pub(crate) mod astronomy;
pub mod constants; // raw i32 constants kept for internal use

// ── Core types ─────────────────────────────────────────────────────────────────
pub mod error;
pub(crate) mod types;

// ── New domain modules ─────────────────────────────────────────────────────────
pub mod body;
#[cfg(feature = "calendar-traditions")]
pub mod calendar;
pub mod chart;
pub mod geo;
pub mod houses;
pub mod moon;
pub mod motion;
pub mod position;
pub mod solar;
pub mod time;
pub mod units;
pub mod vedic;

// ── Flat re-exports from domain modules ─────────────────────────────────────────
// Everything accessible as `celestial_core::calc_ut` etc. for backward compatibility
// and for the language bindings. Prefer using domain modules directly:
//   use celestial_core::position::calc_ut;
//   use celestial_core::moon::moon_phase;
pub use body::{Body, BodyError, CalcFlags, Calendar, HouseSystem, SiderealMode};
#[cfg(feature = "calendar-traditions")]
pub use calendar::{
    approx_hebrew_year, bahai_holy_days, buddhist, celtic, christian, christian_feasts,
    christian_fixed_feasts, coptic_month_days, coptic_to_jd, days_in_hebrew_year, easter_gregorian,
    easter_jd, easter_julian, easter_orthodox, easter_orthodox_jd, elapsed_days, ethiopic_to_jd,
    fasli_nowruz_jd, gregorian_to_hijri_years, gregorian_to_solar_hijri, hebrew, hebrew_month_days,
    hebrew_month_start_jd, hebrew_new_year_jd, hebrew_year_from_jd, hijri_from_jd,
    hijri_month_days, hijri_month_name, hijri_month_start_jd, hijri_new_year_jd, hijri_to_jd,
    hindu, is_bahai_leap_year, is_coptic_leap_year, is_hebrew_leap_year, is_hijri_leap_year,
    islamic, islamic_observances, islamic_observances_for_jd, jd_to_bahai, jd_to_coptic,
    jd_to_ethiopic, jd_to_fasli, jd_to_hebrew_date, jewish_holiday_jd, jewish_holidays, losar_jd,
    months_in_hebrew_year, naw_ruz_jd, nowruz_jd, omer_day_jd, omer_days, omer_declaration,
    omer_from_jd, omer_period, omer_start_jd, persian, solar_hijri_to_gregorian, tibetan_year_name,
    BahaiDate, BahaiHolyDay, ChristianFeast, IslamicObservance, JewishHoliday, OmerDay, OmerPeriod,
    COPTIC_EPOCH_JD, COPTIC_MONTHS, ETHIOPIC_EPOCH_JD, ETHIOPIC_MONTHS, FASLI_MONTHS, GATHA_DAYS,
    LHASA_TZ_OFFSET_HOURS,
};
pub use chart::{
    almuten, annual_profection, antiscion, arabic_part, arabic_parts_seven, asc_transit_ut,
    aspect_angles, calc_chart_aspects, calc_chart_aspects_auto, calendar_round, decan_ruler,
    default_orb, distance_to_mc, dsc_transit_ut, egyptian_decan, egyptian_terms_ruler, firdaria,
    four_pillars, full_dignity, haab, ic_transit_ut, is_applying, is_day_chart,
    local_apparent_solar_time, lon_to_sign, lower_meridian_transit_ut, lunar_return_jd,
    make_pillar, match_aspect, match_aspect2, match_aspect3, match_aspect4, maya_long_count,
    maya_long_count_str, mc_transit_ut, medicine_wheel_totem, meridian_transit_ut, midpoint,
    midpoint_table, monthly_profection, nakshatras, next_aspect, next_aspect2, next_aspect_cusp,
    next_aspect_cusp2, next_aspect_with, next_aspect_with2, next_retro, parallactic_angle,
    planet_conjunct_mc, planet_house_number, planet_on_midpoint, retrograde_station_ut, same_sect,
    secondary_progressions, sexagenary_name, sign_exaltation, sign_ingress_ut, sign_ruler,
    sign_ruler_modern, signs, solar_arc_directions, solar_return_jd, solar_term_position,
    tonalpohualli, transit_to_degree, triplicity_rulers, tzolkin,
    vietnamese_chinese_boundary_differs, vietnamese_month_start_jd, vimshottari_dasha,
    xiuhpohualli, years_diff, zodiac_sign_name, Antiscion, ArabicPart, AspectCuspResult,
    AspectMatch, AspectOrbs, AspectResult, BaZiPillar, ChartAspect, DashaLevel, Dignity,
    FirdariaPeriod, MidpointEntry, RetroResult, SearchOptions, SolarArcResult, Stations,
    ALL_ASPECTS, CHINA_TZ_OFFSET_HOURS, DASHA_SEQUENCE, EARTHLY_BRANCHES, GMT_CORRELATION,
    HEAVENLY_STEMS, MAJOR_ASPECTS, SOLAR_TERMS, TONALPOHUALLI_SIGNS, TZOLKIN_SIGNS,
    VIETNAM_TZ_OFFSET_HOURS, XIUHPOHUALLI_MONTHS,
};
pub use constants::{
    ACRONYCHAL_RISING, ACRONYCHAL_SETTING, ADMETOS, APOLLON, APP_TO_TRUE, ARCSEC_PER_RAD, ARMC,
    ASC, ASTNAMFILE, AST_OFFSET, AU_KM, BIT_ASTRO_TWILIGHT, BIT_CIVIL_TWILIGHT, BIT_DISC_BOTTOM,
    BIT_DISC_CENTER, BIT_FIXED_DISC_SIZE, BIT_FORCE_SLOW_METHOD, BIT_GEOCTR_NO_ECL_LAT,
    BIT_HINDU_RISING, BIT_NAUTIC_TWILIGHT, BIT_NO_REFRACTION, CALC_AZALT, CALC_AZALT_REV,
    CALC_ITRANSIT, CALC_MTRANSIT, CALC_RISE, CALC_SET, CERES, CHIRON, COASC1, COASC2, COMET_OFFSET,
    COSMICAL_RISING, COSMICAL_SETTING, CUPIDO, EARTH, EARTH_RADIUS_KM, ECL_1ST_VISIBLE,
    ECL_2ND_VISIBLE, ECL_3RD_VISIBLE, ECL_4TH_VISIBLE, ECL_ALLTYPES_LUNAR, ECL_ALLTYPES_SOLAR,
    ECL_ANNULAR, ECL_ANNULAR_TOTAL, ECL_CENTRAL, ECL_HYBRID, ECL_MAX_VISIBLE, ECL_NONCENTRAL,
    ECL_NUT, ECL_OCCULTATION, ECL_ONE_TRY, ECL_PARTBEG_VISIBLE, ECL_PARTEND_VISIBLE, ECL_PARTIAL,
    ECL_PENUMBRAL, ECL_TOTAL, ECL_TOTBEG_VISIBLE, ECL_TOTEND_VISIBLE, ECL_VISIBLE, EPHE_PATH,
    EQUASC, EVENING_FIRST, EVENING_RISING, FICTFILE, FICT_MAX, FICT_OFFSET, FICT_OFFSET_1, FIXSTAR,
    FLG_ASTROMETRIC, FLG_BARYCTR, FLG_BUILTIN, FLG_CENTER_BODY, FLG_DPSIDEPS_1980, FLG_EQUATORIAL,
    FLG_HELCTR, FLG_ICRS, FLG_J2000, FLG_JPL, FLG_JPLHOR, FLG_JPLHOR_APPROX, FLG_MOSHIER,
    FLG_NOABERR, FLG_NOGDEFL, FLG_NONUT, FLG_ORBEL_AA, FLG_RADIANS, FLG_SIDEREAL, FLG_SPEED,
    FLG_SPEED3, FLG_TEST_PLMOON, FLG_TOPOCTR, FLG_TROPICAL, FLG_TRUEPOS, FLG_XYZ, FNAME_DE200,
    FNAME_DE403, FNAME_DE404, FNAME_DE405, FNAME_DE406, FNAME_DE431, FNAME_DFT, FNAME_DFT2,
    GREG_CAL, HADES, HARRINGTON, HELFLAG_AV, HELFLAG_AVKIND_MIN7, HELFLAG_AVKIND_MIN9,
    HELFLAG_AVKIND_PTO, HELFLAG_AVKIND_VR, HELFLAG_HIGH_PRECISION, HELFLAG_LONG_SEARCH,
    HELFLAG_NO_DETAILS, HELFLAG_OPTICAL_PARAMS, HELFLAG_SEARCH_1_PERIOD, HELFLAG_VISLIM_DARK,
    HELFLAG_VISLIM_NOMOON, HELFLAG_VISLIM_PHOTOPIC, HELFLAG_VISLIM_SCOTOPIC, HELIACAL_RISING,
    HELIACAL_SETTING, INTP_APOG, INTP_PERG, ISIS, JUL_CAL, JUNO, JUPITER, KRONOS, MARS, MC,
    MEAN_APOG, MEAN_NODE, MERCURY, MOON, MORNING_LAST, MORNING_SETTING, NALL_NAT_POINTS, NASCMC,
    NEPTUNE, NEPTUNE_ADAMS, NEPTUNE_LEVERRIER, NFICT_ELEM, NIBIRU, NODBIT_FOPOINT, NODBIT_MEAN,
    NODBIT_OSCU, NODBIT_OSCU_BAR, NPLANETS, OBLIQUITY_J2000, OSCU_APOG, PALLAS, PHOLUS,
    PLMOON_OFFSET, PLUTO, PLUTO_LOWELL, PLUTO_PICKERING, POLASC, POSEIDON, PROSERPINA,
    RAD_PER_ARCSEC, SATURN, SIDBITS, SIDBIT_ECL_DATE, SIDBIT_ECL_T0, SIDBIT_NO_PREC_OFFSET,
    SIDBIT_PREC_ORIG, SIDBIT_SSY_PLANE, SIDBIT_USER_UT, SIDM_ALDEBARAN_15TAU, SIDM_ARYABHATA,
    SIDM_ARYABHATA_522, SIDM_ARYABHATA_MSUN, SIDM_B1950, SIDM_BABYL_BRITTON, SIDM_BABYL_ETPSC,
    SIDM_BABYL_HUBER, SIDM_BABYL_KUGLER1, SIDM_BABYL_KUGLER2, SIDM_BABYL_KUGLER3, SIDM_DELUCE,
    SIDM_DJWHAL_KHUL, SIDM_FAGAN_BRADLEY, SIDM_GALALIGN_MARDYKS, SIDM_GALCENT_0SAG,
    SIDM_GALCENT_COCHRANE, SIDM_GALCENT_MULA_WILHELM, SIDM_GALCENT_RGILBRAND, SIDM_GALEQU_FIORENZA,
    SIDM_GALEQU_IAU1958, SIDM_GALEQU_MULA, SIDM_GALEQU_TRUE, SIDM_HIPPARCHOS, SIDM_J1900,
    SIDM_J2000, SIDM_JN_BHASIN, SIDM_KRISHNAMURTI, SIDM_KRISHNAMURTI_VP291, SIDM_LAHIRI,
    SIDM_LAHIRI_1940, SIDM_LAHIRI_ICRC, SIDM_LAHIRI_VP285, SIDM_RAMAN, SIDM_SASSANIAN,
    SIDM_SS_CITRA, SIDM_SS_REVATI, SIDM_SURYASIDDHANTA, SIDM_SURYASIDDHANTA_MSUN, SIDM_TRUE_CITRA,
    SIDM_TRUE_MULA, SIDM_TRUE_PUSHYA, SIDM_TRUE_REVATI, SIDM_TRUE_SHEORAN, SIDM_USER,
    SIDM_USHASHASHI, SIDM_VALENS_MOON, SIDM_YUKTESHWAR, SOLAR_PARALLAX, SPLIT_DEG_KEEP_DEG,
    SPLIT_DEG_KEEP_SIGN, SPLIT_DEG_NAKSHATRA, SPLIT_DEG_ROUND_DEG, SPLIT_DEG_ROUND_MIN,
    SPLIT_DEG_ROUND_SEC, SPLIT_DEG_ZODIACAL, STARFILE, STARFILE_OLD, SUN, TIDAL_26,
    TIDAL_AUTOMATIC, TIDAL_DE200, TIDAL_DE403, TIDAL_DE404, TIDAL_DE405, TIDAL_DE406, TIDAL_DE421,
    TIDAL_DE422, TIDAL_DE430, TIDAL_DE431, TIDAL_DEFAULT, TIDAL_JPLEPH_DE406, TIDAL_MOSEPH,
    TIDAL_STEPHENSON_2016, TIDAL_SWIEPH, TRUE_NODE, TRUE_TO_APP, URANUS, VARUNA, VENUS, VERTEX,
    VESTA, VULCAN, VULKANUS, WALDEMATH, WHITE_MOON, ZEUS,
};
pub use geo::{
    azalt, azalt_rev, centisec_to_deg_str, centisec_to_lonlat_str, centisec_to_time_str,
    coord_transform, coord_transform_with_speed, cs_round_sec, deg_to_cs, degsplit, diff_cs,
    diff_cs_signed, diff_deg, diff_deg_signed, diff_rad_signed, format_coord, geo_to_dms,
    house_system_char, house_system_id, midpoint_deg, midpoint_rad, norm_cs, norm_deg, norm_rad,
    parse_coord, refrac, refrac_extended, sidereal_mode_flag, sidereal_mode_id, sign_name,
    split_deg, wrap_signed_180, AzAlt,
};
#[cfg(feature = "timezone")]
pub use geo::{tz_abbr_find, TzAbbr, TZ_TABLE};
pub use houses::{
    house_name, house_pos, houses, houses_armc, houses_armc_ex2, houses_ex, houses_ex2,
    houses_from_armc, mean_sidereal_time_deg, sidereal_time_deg, HouseResult, HouseResultEx2,
};
#[cfg(feature = "calendar-traditions")]
pub use moon::{
    esbats_for_year, next_esbat, next_full_moon, next_full_moon_after, next_new_moon_after,
    next_sabbat, sabbat_jd, sabbats_for_year, uposatha_days, vesak_jd, Esbat, EsbatName, Sabbat,
    SabbatKind, Uposatha, UposathaPhase,
};
pub use moon::{
    moon_elongation, moon_illumination, moon_phase, moon_phase_angle, moon_phase_info,
    moon_phases_for_month, next_first_quarter, next_full_moon_phase, next_last_quarter,
    next_new_moon, next_principal_phase, MoonPhase, MoonPhaseInfo, PhaseEvent, PrincipalPhase,
    SYNODIC_MONTH,
};
pub use motion::{
    helio_cross, helio_cross_ut, lun_eclipse_how, lun_eclipse_when, lun_eclipse_when_loc,
    lun_occult_when_glob, lun_occult_when_loc, lun_occult_where, mooncross, mooncross_back_ut,
    mooncross_node, mooncross_node_ut, mooncross_ut, rise_trans, rise_trans_true_hor,
    sol_eclipse_how, sol_eclipse_when_glob, sol_eclipse_when_loc, sol_eclipse_where, solcross,
    solcross_back_ut, solcross_ut, EclipseHow, EclipseResult, EclipseResultAttr, EclipseWhere,
    MoonCrossNode, RiseTransOptions, RiseTransResult,
};
pub use position::{
    ayanamsa, ayanamsa_ex, ayanamsa_ex_ut, ayanamsa_name, ayanamsa_ut, best_time_method, calc,
    calc_many, calc_pctr, calc_ut, calc_ut_many, close, current_file_data, fixstar, fixstar2,
    fixstar2_mag, fixstar2_ut, fixstar_mag, fixstar_ut, gauquelin_sector, get_orbital_elements,
    heliacal_pheno_ut, heliacal_ut, library_path, nod_aps, nod_aps_ut, orbit_max_min_true_distance,
    pheno, pheno_ut, planet_name, set_delta_t_userdef, set_ephe_path, set_jpl_file, set_lapse_rate,
    set_sid_mode, set_tid_acc, set_topo, tid_acc, version, vis_limit_mag, yallop_q, CalcOptions,
    CalcStrategy, CurrentFileData,
};
pub use solar::{
    cycle_nickname, grand_solar_epoch, solar_cycle, GrandSolarEpoch, SolarCycleInfo,
    SolarCyclePhase,
};
pub use time::{
    date_conversion, day_of_week, day_of_year, deltat, deltat_ex, iso_week, jd_duration,
    jd_et_to_utc, jd_to_iso_string, jd_ut_to_utc, jdnow, julday, lat_to_lmt, lmt_to_lat,
    mean_obliquity, mean_sidtime, nutation, parse_datetime, parse_time, revjul, revjul_hms,
    sidtime, sidtime0, time_equ, true_obliquity, tt_to_ut, utc_time_zone, utc_to_jd,
    weeks_in_iso_year, CalDate, JdPair, UtcDate,
};
pub use vedic::{
    hindu_festivals, karana_name, long_to_nakshatra, long_to_navamsa, long_to_rasi,
    naisargika_relation, nakshatra_name, ochchabala, panchanga, raman_houses, rasi_diff,
    rasi_diff2, rasi_norm, residential_strength, saturn_4_stars, sign_lord, tatkalika_relation,
    HinduFestival, Paksha, Panchanga, KARANA_NAMES, NAKSHATRA_NAMES, TITHI_NAMES, VARA_NAMES,
    YOGA_NAMES,
};

// ── Public types from types.rs ─────────────────────────────────────────────────
pub use types::{FixStarPos, NodAps, OrbitalDistances, OrbitalElements, PlanetPos};

// ── Unit newtypes (DP-2) ───────────────────────────────────────────────────────
pub use units::{Degrees, JulianDay, Latitude, Longitude};

// ── Error ──────────────────────────────────────────────────────────────────────
pub use error::{Error, Result};

// ── Prelude ────────────────────────────────────────────────────────────────────
pub mod prelude {
    //! The most commonly used items, all in one place.
    //!
    //! ```no_run
    //! use celestial_core::prelude::*;
    //!
    //! let jd  = JulianDay::new(julday(2025, 3, 20, 9.0, Calendar::Gregorian));
    //! let sun = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN).unwrap();
    //! println!("Sun: {:.4}°", sun.lon);
    //! ```
    pub use crate::body::{Body, BodyError, CalcFlags, Calendar, HouseSystem, SiderealMode};
    pub use crate::chart::AspectOrbs;
    pub use crate::chart::{
        calc_chart_aspects, calc_chart_aspects_auto, lunar_return_jd, solar_return_jd,
    };
    pub use crate::error::{Error, Result};
    pub use crate::houses::houses;
    pub use crate::moon::{moon_illumination, moon_phase, moon_phases_for_month, MoonPhase};
    pub use crate::motion::{
        mooncross_ut, rise_trans, solcross_ut, RiseTransOptions, SearchOptions,
    };
    pub use crate::position::{
        ayanamsa, calc_many, calc_ut, calc_ut_many, planet_name, set_sid_mode, CalcOptions,
        CalcStrategy, PlanetPos,
    };
    pub use crate::time::{jdnow, julday, revjul, CalDate};
    pub use crate::units::{Degrees, JulianDay, Latitude, Longitude};
}

// ── Implementation detail — not part of the public API ──────────────────────────
pub(crate) mod functions;
