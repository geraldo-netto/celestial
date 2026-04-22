<?php
/**
 * PHPStan stubs for the celestial PHP extension.
 *
 * Declares every function exposed by the compiled libcelestial.so so that
 * phpstan can analyse the test files without the extension being loaded.
 * Return types mirror the Rust binding signatures in bindings/php/src/lib.rs.
 */
declare(strict_types=1);

// ── Constants ────────────────────────────────────────────────────────────────
const GREG_CAL = 1;
const JUL_CAL  = 0;
const SE_GREG_CAL = 1;
const SE_JUL_CAL  = 0;

const SIDM_LAHIRI        = 1;
const SIDM_FAGAN_BRADLEY = 0;
const SIDM_RAMAN         = 3;
const SIDM_KRISHNAMURTI  = 5;
const SIDM_TRUE_CITRA    = 27;

// planets
const SUN  = 0; const MOON = 1; const MERCURY = 2; const VENUS  = 3;
const MARS = 4; const JUPITER = 5; const SATURN = 6; const URANUS = 7;
const NEPTUNE = 8; const PLUTO = 9; const MEAN_NODE = 10; const TRUE_NODE = 11;
const CHIRON = 15;

const FLG_BUILTIN = 2;
const FLG_SPEED   = 256;
const FLG_SIDEREAL = 64;
const FLG_TOPOCTR  = 32768;
const FLG_NOABERR  = 512;
const FLG_NOGDEFL  = 1024;

// ── Time ─────────────────────────────────────────────────────────────────────
function julday(int $year, int $month, int $day, float $hour, int $calendar = 1): float {}
/** @return array{year: float, month: float, day: float, hour: float} */
function revjul(float $jd, int $calendar = 1): array {}
function day_of_week(float $jd): int {}
function deltat(float $jd): float {}
function sidtime(float $jd_ut): float {}
function mean_sidtime(float $jd_ut): float {}
function set_delta_t_userdef(float $dt): void {}

// ── Calculation ───────────────────────────────────────────────────────────────
/** @return array<float> [lon, lat, dist, speed_lon, speed_lat, speed_dist] */
function calc_ut(float $tjdut, int $planet, int $flags = 2): array {}
/** @return array<float> */
function calc(float $tjdet, int $planet, int $flags = 2): array {}
/** @return array<float> */
function calc_pctr(float $tjdet, int $planet, int $center, int $flags = 2): array {}

// ── Houses ────────────────────────────────────────────────────────────────────
/** @return array{cusps: array<float>, ascmc: array<float>} */
function houses(float $tjdut, float $lat, float $lon, int $hsys = 80): array {}
/** @return array{cusps: array<float>, ascmc: array<float>} */
function houses_ex(float $tjdut, float $lat, float $lon, int $hsys = 80, int $flags = 0): array {}
function house_name(int $hsys): string {}

// ── Sidereal ─────────────────────────────────────────────────────────────────
function set_sid_mode(int $sid_mode, float $t0 = 0.0, float $ayan_t0 = 0.0): void {}
function get_ayanamsa(float $tjdet): float {}
function get_ayanamsa_ut(float $tjdut): float {}
function get_ayanamsa_name(int $sid_mode): string {}

// ── Math helpers ──────────────────────────────────────────────────────────────
function norm_deg(float $d): float {}
function norm_cs(int $p): int {}
function degnorm(float $d): float {}
function difdeg2n(float $p1, float $p2): float {}
function diff_deg_signed(float $p1, float $p2): float {}
/** @return array<float> [d, m, s, fraction, sign] */
function split_deg(float $deg, int $round_flag = 0): array {}
function midpoint_deg(float $x1, float $x0): float {}
function midpoint(float $x1, float $x0): float {}
/** @return array<float> */
function coord_transform(array $coords, float $eps): array {}

// ── Crossings ─────────────────────────────────────────────────────────────────
function solcross_ut(float $x2cross, float $jd_ut, int $flags = 2): float {}
function mooncross_ut(float $x2cross, float $jd_ut, int $flags = 2): float {}

// ── Zodiac ────────────────────────────────────────────────────────────────────
/** @return array<float> [sign, degrees] */
function lon_to_sign(float $lon): array {}
function zodiac_sign_name(int $sign): string {}
function sign_name(int $sign): string {}
function sign_ruler(int $sign): int {}

// ── Vedic ─────────────────────────────────────────────────────────────────────
/** @return int[] [nakshatra_index (0-26), pada (1-4)] */
function long_to_nakshatra(float $lon): array {}
function long_to_navamsa(float $lon): int {}
function long_to_rasi(float $lon): int {}

// ── Chart helpers ─────────────────────────────────────────────────────────────
function arabic_part(float $asc, float $body2, float $body1): float {}
function parse_coord(string $coord): ?float {}

// ── Moon phases ───────────────────────────────────────────────────────────────
function moon_phase(float $jd): string {}
function moon_illumination(float $jd): float {}
function moon_elongation(float $jd): float {}
function next_new_moon(float $jd_from): float {}
function next_full_moon(float $jd_from): float {}
function next_full_moon_phase(float $jd_from): float {}
function moon_phases_for_month(int $year, int $month): array {}

// ── Sabbats / esbats ──────────────────────────────────────────────────────────
function sabbat_jd(int $year, string $kind): float {}
function next_sabbat_name(float $jd): string {}

// ── Calendars ─────────────────────────────────────────────────────────────────
/** @return array{year: string, month: string, day: string} */
function hijri_from_jd(float $jd): array {}
function hijri_to_jd(int $year, int $month, int $day): float {}
/** @return array<array<string, string>> */
function jewish_holidays(int $hebrew_year): array {}
/** @return array{month: string, day: string} */
function easter_gregorian(int $year): array {}
function easter_jd(int $year): float {}
function panchanga(float $jd): array {}
function nowruz_jd(int $year): float {}
function vesak_jd(int $year): float {}

// ── Misc ──────────────────────────────────────────────────────────────────────
function celestial_version(): string {}
function set_ephe_path(string $path): void {}
function close(): void {}
function get_planet_name(int $planet): int {}

// ── Sabbats / esbats (full year) ─────────────────────────────────────────────
/** @return array<array<string, mixed>> list of {name, jd, date} for all 8 sabbats */
function sabbats_for_year(int $year): array {}
/** @return array<array<string, mixed>> list of full moons for the year */
function esbats_for_year(int $year): array {}

// ── Coordinate formatting ─────────────────────────────────────────────────────
function format_coord(float $coord, bool $is_latitude): ?string {}

// ── Parallel multi-body calculation ─────────────────────────────────────────

/** @return array<array<float>> [[lon,lat,dist,speed_lon,speed_lat,speed_dist], ...] */
function calc_many(float $tjdet, array $planets, int $flags = 2): array {}
/** @return array<array<float>> */
function calc_ut_many(float $tjdut, array $planets, int $flags = 2): array {}

// ── Nutation & obliquity ─────────────────────────────────────────────────────

/** @return float[] [dpsi_degrees, deps_degrees] — IAU 2000B, ~1 mas accuracy */
function nutation(float $jde): array {}
/** Mean obliquity of the ecliptic in degrees (IAU 2006). */
function mean_obliquity(float $jde): float {}
/** True (apparent) obliquity of the ecliptic in degrees. */
function true_obliquity(float $jde): float {}

// Phase 5–8 functions
function celestial_egyptian_terms_ruler(float $lon): int {}
function celestial_decan_ruler(float $lon): int {}
function celestial_full_dignity(int $body, float $lon, bool $is_day): array {}
function celestial_almuten(float $lon, bool $is_day): array {}
function celestial_four_pillars(float $jd, float $hour, float $sun_lon): array {}
function celestial_solar_term_position(float $sun_lon): array {}
function celestial_tonalpohualli(float $jd): array {}
function celestial_tzolkin(float $jd): array {}
function celestial_haab(float $jd): array {}
function celestial_medicine_wheel_totem(float $sun_lon): array {}
function celestial_egyptian_decan(float $lon): array {}
function celestial_fixstar_ut(string $star, float $jd, int $flags): ?array {}
function celestial_firdaria(float $jd_birth, bool $is_day, float $span): array {}
function celestial_is_day_chart(float $sun_lon, array $cusps): bool {}

/** @return float */
function celestial_ic_transit_ut(int $planet, float $jd_natal, float $jd_start,
    float $lat, float $lon, int $hsys, int $flags, bool $backward): float {}

/** @return float */
function celestial_asc_transit_ut(int $planet, float $jd_natal, float $jd_start,
    float $lat, float $lon, int $hsys, int $flags, bool $backward): float {}

/** @return float */
function celestial_dsc_transit_ut(int $planet, float $jd_natal, float $jd_start,
    float $lat, float $lon, int $hsys, int $flags, bool $backward): float {}

/** @return float */
function celestial_julday(int $year, int $onth, int $day, float $hour, int $calendar): float {}

/** @return array */
function celestial_revjul(float $jd, int $calendar): array {}

/** @return int */
function celestial_day_of_week(float $jd): int {}

/** @return float */
function celestial_deltat(float $jd): float {}

/** @return float */
function celestial_sidtime(float $jd_ut): float {}

/** @return void */
function celestial_set_delta_t_userdef(float $dt): void {}

/** @return void */
function celestial_set_ephe_path(string $path): void {}

/** @return void */
function celestial_close(): void {}

/** @return void */
function celestial_set_sid_mode(int $sid_mode, float $0, float $ayan_t0): void {}

/** @return void */
function celestial_set_topo(float $geolon, float $geolat, float $geoalt): void {}

/** @return string */
function celestial_get_planet_name(int $planet): string {}

/** @return float[] */
function celestial_calc_ut(float $jdut, int $planet, int $flags): float[] {}

/** @return float[] */
function celestial_calc(float $jdet, int $planet, int $flags): float[] {}

/** @return float[] */
function celestial_nutation(float $jde): float[] {}

/** @return float */
function celestial_mean_obliquity(float $jde): float {}

/** @return float */
function celestial_true_obliquity(float $jde): float {}

/** @return array */
function celestial_calc_many(float $jdet, int[] $planets, int $flags): array {}

/** @return array */
function celestial_calc_ut_many(float $jdut, int[] $planets, int $flags): array {}

/** @return array */
function celestial_fixstar(string $star, float $jdet, int $flags): array {}

/** @return float */
function celestial_fixstar_mag(string $star): float {}

/** @return array */
function celestial_nod_aps(float $jdet, int $planet, int $flags, mixed $ethod): array {}

/** @return array */
function celestial_houses(float $jdut, float $geolat, float $geolon, mixed $hsys): array {}

/** @return array */
function celestial_houses_ex(float $jdut, int $flags, float $geolat, float $geolon, mixed $hsys): array {}

/** @return string */
function celestial_house_name(int $hsys): string {}

/** @return float */
function celestial_house_pos(float $armc, float $geolat, float $eps, int $hsys, float $lon, mixed $lat_body): float {}

/** @return float */
function celestial_get_ayanamsa(float $jdet): float {}

/** @return float */
function celestial_get_ayanamsa_ut(float $jdut): float {}

/** @return string */
function celestial_get_ayanamsa_name(int $sid_mode): string {}

/** @return array */
function celestial_sol_eclipse_when_glob(float $jd_start, int $flags, int $ecl_type, mixed $backwards): array {}

/** @return array */
function celestial_lun_eclipse_when(float $jd_start, int $flags, int $ecl_type, mixed $backwards): array {}

/** @return array */
function celestial_rise_trans(float $jdut, int $planet, int $flags, int $event_type, float[] $geopos, float $pressure_mb, mixed $emp_c): array {}

/** @return float */
function celestial_solcross_ut(float $x2cross, float $jd_ut, int $flags): float {}

/** @return float */
function celestial_mooncross_ut(float $x2cross, float $jd_ut, int $flags): float {}

/** @return float */
function celestial_helio_cross_ut(int $planet, float $x2cross, float $jd_ut, int $flags, mixed $dir): float {}

/** @return float */
function celestial_norm_deg(float $d): float {}

/** @return int */
function celestial_norm_cs(int $p): int {}

/** @return float */
function celestial_diff_deg_signed(float $p1, float $p2): float {}

/** @return float[] */
function celestial_split_deg(float $deg, int $round_flag): float[] {}

/** @return float */
function celestial_midpoint_deg(float $x1, float $x0): float {}

/** @return float[] */
function celestial_coord_transform(float[] $coords, float $eps): float[] {}

/** @return array */
function celestial_azalt(float $jdut, int $calc_flag, float[] $geopos, float $pressure_mb, float $emp_c, array $xin): array {}

/** @return float[] */
function celestial_azalt_rev(float $jdut, int $calc_flag, float[] $geopos, float $az, mixed $alt): float[] {}

/** @return float */
function celestial_refrac(float $altitude, float $pressure_mb, float $emp_c, int $calc_flag): float {}

/** @return array */
function celestial_sabbats_for_year(int $year): array {}

/** @return float */
function celestial_next_sabbat_jd(float $jd_from): float {}

/** @return string */
function celestial_next_sabbat_name(float $jd_from): string {}

/** @return float */
function celestial_sabbat_jd(int $year, string $kind): float {}

/** @return float */
function celestial_next_full_moon(float $jd_from): float {}

/** @return array */
function celestial_esbats_for_year(int $year): array {}

/** @return string */
function celestial_next_esbat_name(float $jd_from): string {}

/** @return float */
function celestial_next_esbat_jd(float $jd_from): float {}

/** @return array */
function celestial_match_aspect(float $pos0, float $speed0, float $pos1, float $speed1, float $aspect, mixed $orb): array {}

/** @return float|null */
function celestial_next_retro(int $planet, float $jd_start, bool $backward, float $stop_days, mixed $flags): float|null {}

/** @return int */
function celestial_long_to_rasi(float $lon): int {}

/** @return int */
function celestial_long_to_navamsa(float $lon): int {}

/** @return int[] */
function celestial_long_to_nakshatra(float $lon): int[] {}

/** @return string|null */
function celestial_nakshatra_name(int $nak): string|null {}

/** @return float[] */
function celestial_raman_houses(float $asc, float $c, bool $sandhi): float[] {}

/** @return string|null */
function celestial_sign_name(int $sign): string|null {}

/** @return string|null */
function celestial_format_coord(float $coord, bool $is_latitude): string|null {}

/** @return float|null */
function celestial_parse_coord(string $s): float|null {}

/** @return float */
function celestial_jdnow(): float {}

/** @return string */
function celestial_jd_to_iso_string(float $jd, int $calendar): string {}

/** @return float[] */
function celestial_sign_ingress_ut(int $planet, float $jd, int $flags, bool $backward): float[] {}

/** @return float */
function celestial_arabic_part(float $asc, float $body2, float $body1): float {}

/** @return float */
function celestial_transit_to_degree(int $planet, float $arget_lon, float $jd, int $flags, mixed $backward): float {}

/** @return float */
function celestial_mc_transit_ut(int $planet, float $jd_natal, float $jd_start, float $lat, float $lon, int $hsys, int $flags, mixed $backward): float {}

/** @return float */
function celestial_solar_return_jd(float $jd_natal, int $return_year, int $flags): float {}

/** @return float */
function celestial_lunar_return_jd(float $jd_natal, float $jd_start, int $flags): float {}

/** @return float */
function celestial_midpoint(float $lon1, float $lon2): float {}

/** @return int */
function celestial_sign_ruler(int $sign): int {}

/** @return mixed */
function celestial_zodiac_sign_name(int $sign): mixed {}

/** @return float[] */
function celestial_lon_to_sign(float $lon): float[] {}

/** @return float */
function celestial_local_apparent_solar_time(float $jd_ut, float $geolon_deg): float {}

/** @return float[] */
function celestial_annual_profection(float[] $cusps, int $age): float[] {}

/** @return array */
function celestial_vimshottari_dasha(float $jd_birth, float $oon_lon_sidereal, float $years_ahead): array {}

/** @return mixed */
function celestial_celestial_version(): mixed {}

/** @return array|null */
function celestial_omer_from_jd(float $jd): array|null {}

/** @return float|null */
function celestial_omer_day_jd(int $hebrew_year, int $day): float|null {}

/** @return float */
function celestial_omer_start_jd(int $hebrew_year): float {}

/** @return array */
function celestial_omer_days(int $hebrew_year): array {}

/** @return string */
function celestial_omer_declaration(int $day): string {}

/** @return array */
function celestial_omer_period(float $jd): array {}

/** @return array */
function celestial_jewish_holidays(int $hebrew_year): array {}

/** @return float|null */
function celestial_jewish_holiday_jd(int $hebrew_year, string $name): float|null {}

/** @return int */
function celestial_hebrew_year_from_jd(float $jd): int {}

/** @return array */
function celestial_jd_to_hebrew_date(float $jd): array {}

/** @return array */
function celestial_easter_gregorian(int $year): array {}

/** @return array */
function celestial_easter_orthodox(int $year): array {}

/** @return float */
function celestial_easter_jd(int $year): float {}

/** @return float */
function celestial_easter_orthodox_jd(int $year): float {}

/** @return array */
function celestial_christian_feasts(int $year): array {}

/** @return array */
function celestial_christian_fixed_feasts(int $year): array {}

/** @return array */
function celestial_hijri_from_jd(float $jd): array {}

/** @return float */
function celestial_hijri_to_jd(int $year, int $onth, int $day): float {}

/** @return string */
function celestial_hijri_month_name(int $onth): string {}

/** @return array */
function celestial_islamic_observances(int $hijri_year): array {}

/** @return array */
function celestial_panchanga(float $jd): array {}

/** @return array */
function celestial_hindu_festivals(int $gregorian_year): array {}

/** @return float */
function celestial_vesak_jd(int $year): float {}

/** @return array */
function celestial_uposatha_days(int $year): array {}

/** @return float */
function celestial_nowruz_jd(int $year): float {}

/** @return int */
function celestial_gregorian_to_solar_hijri(int $year): int {}

/** @return float */
function celestial_naw_ruz_jd(int $bahai_year): float {}

/** @return array */
function celestial_jd_to_bahai(float $jd): array {}

/** @return array */
function celestial_bahai_holy_days(int $bahai_year): array {}

/** @return string */
function celestial_moon_phase(float $jd): string {}

/** @return float */
function celestial_moon_illumination(float $jd): float {}

/** @return float */
function celestial_moon_elongation(float $jd): float {}

/** @return float */
function celestial_moon_phase_angle(float $jd): float {}

/** @return float */
function celestial_next_new_moon(float $jd_from): float {}

/** @return float */
function celestial_next_first_quarter(float $jd_from): float {}

/** @return float */
function celestial_next_full_moon_phase(float $jd_from): float {}

/** @return float */
function celestial_next_last_quarter(float $jd_from): float {}

/** @return array */
function celestial_moon_phases_for_month(int $year, int $onth): array {}

/** @return array */
function celestial_moon_phase_info(float $jd): array {}

/** @return int */
function celestial_celestial_egyptian_terms_ruler(float $lon): int {}

/** @return int */
function celestial_celestial_decan_ruler(float $lon): int {}

/** @return string[] */
function celestial_celestial_full_dignity(int $body_raw, float $lon, bool $is_day): string[] {}

/** @return int[] */
function celestial_celestial_almuten(float $lon, bool $is_day): int[] {}

/** @return array */
function celestial_celestial_four_pillars(float $jd_ut, float $hour_ut, float $sun_lon): array {}

/** @return float[] */
function celestial_celestial_solar_term_position(float $sun_lon): float[] {}

/** @return string[] */
function celestial_celestial_tonalpohualli(float $jd): string[] {}

/** @return string[] */
function celestial_celestial_tzolkin(float $jd): string[] {}

/** @return string[] */
function celestial_celestial_haab(float $jd): string[] {}

/** @return string[] */
function celestial_celestial_medicine_wheel_totem(float $sun_lon): string[] {}

/** @return string[] */
function celestial_celestial_egyptian_decan(float $lon): string[] {}

/** @return float[]|null */
function celestial_celestial_fixstar_ut(string $star, float $jdut, int $flags): float[]|null {}

/** @return array */
function celestial_celestial_firdaria(float $jd_birth, bool $is_day, float $span_years): array {}

/** @return bool */
function celestial_celestial_is_day_chart(float $sun_lon, float[] $cusps): bool {}

/** @return int[] */
function celestial_triplicity_rulers(float $lon): int[] {}

/** @return string[] */
function celestial_sexagenary_name(int $cycle_index): string[] {}

/** @return array */
function celestial_secondary_progressions(float $jd_natal, float $years, int[] $bodies, float $lat, float $lon, int $hsys, mixed $flags): array {}

/** @return float[] */
function celestial_solar_arc_directions(float $jd_natal, float $years, array $natal_positions, float $lon, ...]     natal_mc, mixed $flags): float[] {}

/** @return array */
function celestial_midpoint_table(array $positions, mixed $lon, ...]     orb): array {}

/** @return array */
function celestial_calc_chart_aspects(array $positions, float[] $speed, ...]     aspects, mixed $orb): array {}

/** @return array */
function celestial_calc_chart_aspects_auto(array $positions, array $speed, ...]     aspects): array {}

/** @return float[] */
function celestial_monthly_profection(float[] $cusps, int $age_years, int $age_months): float[] {}

/** @return float */
function celestial_next_esbat(float $jd_from): float {}

/** @return float[]|null */
function celestial_next_aspect_cusp(int $body, float $aspect, int $cusp, float $jd_start, float $lat, float $lon, int $hsys, bool $backward, mixed $flags): float[]|null {}

/** @return float[] */
function celestial_antiscion(float[] $pos, float $axis): float[] {}

/** @return float */
function celestial_ayanamsa(float $jd_et): float {}

/** @return float */
function celestial_ayanamsa_ut(float $jd_ut): float {}

/** @return string */
function celestial_ayanamsa_name(int $sid_mode): string {}

/** @return float[] */
function celestial_calc_pctr(float $jdet, int $planet, int $center, int $flags): float[] {}

/** @return string[] */
function celestial_calendar_round(float $jd): string[] {}

/** @return int[] */
function celestial_degsplit(float $pos): int[] {}

/** @return float[] */
function celestial_fixstar2(string $star, float $jdet, int $flags): float[] {}

/** @return float */
function celestial_fixstar2_mag(string $star): float {}

/** @return float[] */
function celestial_fixstar2_ut(string $star, float $jdut, int $flags): float[] {}

/** @return int[] */
function celestial_gregorian_to_hijri_years(int $gregorian_year): int[] {}

/** @return string */
function celestial_house_name_str(int $hsys): string {}

/** @return float[] */
function celestial_houses_ex2(float $jdut, float $lat, float $lon, int $hsys, int $flags): float[] {}

/** @return int[] */
function celestial_jd_duration(float $jd_start, float $jd_end): int[] {}

/** @return float[] */
function celestial_lun_eclipse_how(float $jd_ut, int $flags): float[] {}

/** @return float[] */
function celestial_lun_eclipse_when_loc(float $jd_start, float[] $geopos, int $flags, bool $backwards): float[] {}

/** @return float[] */
function celestial_match_aspect2(float $pos0, float $speed0, float $pos1, float $speed1, float $aspect, float $orb): float[] {}

/** @return float[] */
function celestial_match_aspect3(float $pos0, float $speed0, float $pos1, float $speed1, float $aspect, float $app_orb, float $sep_orb): float[] {}

/** @return float[] */
function celestial_match_aspect4(float $pos0, float $speed0, float $pos1, float $speed1, float $aspect, float $app_orb, float $sep_orb): float[] {}

/** @return float */
function celestial_mean_sidtime(float $jd): float {}

/** @return float[] */
function celestial_mooncross_node(float $jd_et, int $flags): float[] {}

/** @return float[] */
function celestial_mooncross_node_ut(float $jd_ut, int $flags): float[] {}

/** @return int */
function celestial_naisargika_relation(int $gr1, int $gr2): int {}

/** @return float[]|null */
function celestial_next_aspect(int $planet, float $aspect, float $fixed_pt, float $jd_start, bool $backward, float $stop_days, int $flags): float[]|null {}

/** @return float[]|null */
function celestial_next_aspect_cusp2(int $body, float $aspect, int $cusp, float $jd_start, float $lat, float $lon, int $hsys, bool $backward, int $flags): float[]|null {}

/** @return float[]|null */
function celestial_next_aspect_with(int $planet, float $aspect, int $other, float $jd_start, bool $backward, float $stop_days, int $flags): float[]|null {}

/** @return float[] */
function celestial_next_sabbat(float $jd_from): float[] {}

/** @return float */
function celestial_ochchabala(int $graha, float $sputha): float {}

/** @return int[]|null */
function celestial_parse_datetime(string $s): int[]|null {}

/** @return string */
function celestial_planet_name(int $planet): string {}

/** @return float[] */
function celestial_refrac_extended(float $altitude, float $geoalt, float $pressure_mb, float $emp_c, float $lapse_rate, int $calc_flag): float[] {}

/** @return float */
function celestial_residential_strength(float $graha, float[] $bm): float {}

/** @return float[] */
function celestial_retrograde_station_ut(int $planet, float $jd, int $flags): float[] {}

/** @return int[] */
function celestial_revjul_hms(float $jd, int $calendar): int[] {}

/** @return float[] */
function celestial_saturn_4_stars(float $jd, int $flags): float[] {}

/** @return void */
function celestial_set_jpl_file(string $fname): void {}

/** @return int */
function celestial_sign_ruler_modern(int $sign): int {}

/** @return float[] */
function celestial_sol_eclipse_how(float $jd_ut, float[] $geopos, int $flags): float[] {}

/** @return float[] */
function celestial_sol_eclipse_when_loc(float $jd_start, float[] $geopos, int $flags, bool $backwards): float[] {}

/** @return float[] */
function celestial_sol_eclipse_where(float $jd, int $flags): float[] {}

/** @return float[] */
function celestial_utc_to_jd(int $year, int $onth, int $day, int $hour, int $inute, float $second, int $calendar): float[] {}

/** @return string[] */
function celestial_xiuhpohualli(float $jd): string[] {}

