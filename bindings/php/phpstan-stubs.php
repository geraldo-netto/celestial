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
function long_to_nakshatra(float $lon): int {}
function long_to_navamsa(float $lon): int {}
function long_to_rasi(float $lon): int {}

// ── Chart helpers ─────────────────────────────────────────────────────────────
function arabic_part(float $asc, float $body2, float $body1): float {}
function parse_coord(string $coord): float {}

// ── Moon phases ───────────────────────────────────────────────────────────────
function moon_phase(float $jd): string {}
function moon_illumination(float $jd): float {}
function moon_elongation(float $jd): float {}
function next_new_moon(float $jd_from): float {}
function next_full_moon(float $jd_from): float {}
function next_full_moon_phase(float $jd_from): float {}
function moon_phases_for_month(int $year, int $month): array {}

// ── Sabbats / esbats ──────────────────────────────────────────────────────────
function sabbat_jd(int $year, int $sabbat): float {}
function next_sabbat_name(float $jd): string {}
function search(float $jd_start, int $type): float {}

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

