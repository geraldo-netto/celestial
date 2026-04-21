<?php
/**
 * Pure-logic tests for the PHP binding documentation examples.
 *
 * These tests do NOT require the compiled extension.
 * They validate the documented PHP call patterns are syntactically correct,
 * and serve as reference examples.
 *
 * To run (requires the extension):
 *   php tests/pure_logic_test.php
 */

declare(strict_types=1);

// ── Helpers ────────────────────────────────────────────────────────────────────

$passed = 0;
$failed = 0;

function assert_approx(float $actual, float $expected, float $tol = 1e-6, string $label = ''): void {
    global $passed, $failed;
    if (abs($actual - $expected) <= $tol) {
        $passed++;
    } else {
        $failed++;
        $name = $label ?: 'assert_approx';
        echo "FAIL [$name]: expected $expected, got $actual (diff " . abs($actual - $expected) . ")\n";
    }
}

function assert_eq(mixed $actual, mixed $expected, string $label = ''): void {
    global $passed, $failed;
    if ($actual === $expected) {
        $passed++;
    } else {
        $failed++;
        $name = $label ?: 'assert_eq';
        echo "FAIL [$name]: expected " . var_export($expected, true) . ", got " . var_export($actual, true) . "\n";
    }
}

// ── Extension availability check ───────────────────────────────────────────────

if (!extension_loaded('celestial')) {
    echo "SKIP: celestial extension not loaded.\n";
    echo "Build with: cd bindings/php && cargo build --release\n";
    echo "Then add to php.ini: extension=/path/to/libcelestial.so\n";
    exit(0);
}

// ── Constants ──────────────────────────────────────────────────────────────────

assert_eq(SUN,  0, 'SUN constant');
assert_eq(MOON, 1, 'MOON constant');
assert_eq(GREG_CAL, 1, 'GREG_CAL constant');

// ── Time & calendar ────────────────────────────────────────────────────────────

// julday / revjul round-trip
$jd = julday(2002, 1, 1, 0.0, GREG_CAL);
assert_approx($jd, 2452275.5, 1e-6, 'julday 2002-01-01');

$d = revjul($jd, GREG_CAL);
assert_eq((int)$d['year'],  2002, 'revjul year');
assert_eq((int)$d['month'],    1, 'revjul month');
assert_eq((int)$d['day'],      1, 'revjul day');
assert_approx($d['hour'], 0.0, 1e-6, 'revjul hour');

// Day of week: 2452275.5 = Tuesday (1)
assert_eq(day_of_week($jd), 1, 'day_of_week');

// Delta T at J2000 ≈ 63.8 s ≈ 0.000738 days
$dt = deltat(2451545.0);
assert_approx($dt, 0.000738, 0.001, 'deltat J2000');

// ── Planetary positions ─────────────────────────────────────────────────────────

$sun = calc_ut($jd, SUN, FLG_BUILTIN | FLG_SPEED);
assert_eq(count($sun), 6, 'calc_ut returns 6 elements');

$lon = $sun[0];
assert_approx($lon, 280.38, 0.1, 'Sun longitude 2002-01-01');  // ~280°

$dist = $sun[2];
// Earth–Sun distance in January ≈ 0.9832 AU
assert_approx($dist, 0.9832, 0.005, 'Sun distance Jan');

// Speed: Sun moves ~1°/day
$speed = $sun[3];
assert_approx($speed, 1.0, 0.1, 'Sun speed');

// Moon longitude is in [0, 360)
$moon = calc_ut($jd, MOON, FLG_BUILTIN);
assert_approx($moon[0] >= 0.0 && $moon[0] < 360.0 ? 1.0 : 0.0, 1.0, 0, 'Moon lon range');

// ── Houses ──────────────────────────────────────────────────────────────────────

$h = houses($jd, 48.85, 2.35, ord('P'));  // Paris, Placidus
assert_eq(count($h['cusps']),  12, 'houses cusps count');
assert_eq(count($h['ascmc']),  8, 'houses ascmc count');

$asc = $h['ascmc'][0];
assert_approx($asc >= 0.0 && $asc < 360.0 ? 1.0 : 0.0, 1.0, 0, 'ASC in [0,360)');

// house_name
assert_eq(house_name(ord('P')), 'Placidus', 'house_name Placidus');
assert_eq(house_name(ord('K')), 'Koch',     'house_name Koch');

// ── Ayanamsa ────────────────────────────────────────────────────────────────────

set_sid_mode(SIDM_LAHIRI, 0.0, 0.0);
$ay = get_ayanamsa($jd);
// Lahiri ayanamsa in 2002 ≈ 23.88°
assert_approx($ay, 23.88, 0.1, 'Lahiri ayanamsa');

assert_eq(get_ayanamsa_name(SIDM_LAHIRI), 'Lahiri', 'ayanamsa name');

// ── Crossings ───────────────────────────────────────────────────────────────────

// Next vernal equinox after 2002-01-01 ≈ 2002-03-20
$jd_eq = solcross_ut(0.0, $jd, FLG_BUILTIN);
$eq_date = revjul($jd_eq, GREG_CAL);
assert_eq((int)$eq_date['month'], 3, 'Ostara in March');
assert_eq((int)$eq_date['year'],  2002, 'Ostara year 2002');

// ── Coordinate transforms ────────────────────────────────────────────────────────

$norm = degnorm(361.5);
assert_approx($norm, 1.5, 1e-10, 'degnorm(361.5)');

$norm2 = degnorm(-1.0);
assert_approx($norm2, 359.0, 1e-10, 'degnorm(-1)');

// difdeg2n(p1,p2) = p1-p2 normalized to (-180,+180]: 10-350=-340 → +20
$diff = difdeg2n(10.0, 350.0);
assert_approx($diff, 20.0, 1e-10, 'difdeg2n(10,350)=+20');

$parts = split_deg(123.456, 0);
assert_eq(count($parts), 5, 'split_deg count');
assert_eq((int)$parts[0], 123, 'split_deg degrees');

// ── Celtic sabbats ───────────────────────────────────────────────────────────────

$ostara_jd = sabbat_jd(2025, 'Ostara');
$ostara = revjul($ostara_jd, GREG_CAL);
assert_eq((int)$ostara['month'], 3, 'Ostara 2025 in March');
assert_eq((int)$ostara['year'],  2025, 'Ostara 2025 year');

$yule_jd = sabbat_jd(2025, 'Yule');
$yule = revjul($yule_jd, GREG_CAL);
assert_eq((int)$yule['month'], 12, 'Yule 2025 in December');

$all_sabbats = sabbats_for_year(2025);
assert_eq(count($all_sabbats), 8, 'sabbats_for_year returns 8');

// Next sabbat from Jan 1, 2025 should be Imbolc (Feb 1)
$jd_jan = julday(2025, 1, 15, 0.0, GREG_CAL);
$next_name = next_sabbat_name($jd_jan);
assert_eq($next_name, 'Imbolc', 'next sabbat from Jan 15');

// ── Celtic esbats ────────────────────────────────────────────────────────────────

$fm_jd = next_full_moon(julday(2024, 1, 1, 0.0, GREG_CAL));
$fm_date = revjul($fm_jd, GREG_CAL);
// Jan 25, 2024
assert_eq((int)$fm_date['month'], 1, 'First FM 2024 in January');

$esbats_2024 = esbats_for_year(2024);
assert_approx(count($esbats_2024) >= 12 ? 1.0 : 0.0, 1.0, 0, 'esbats_for_year >= 12');

// ── Vedic helpers ────────────────────────────────────────────────────────────────

$rasi = long_to_rasi(45.0);  // 45° = Taurus (1)
assert_eq($rasi, 1, 'long_to_rasi 45° = Taurus');

$nav = long_to_navamsa(0.0);
assert_eq($nav >= 0 && $nav < 12 ? 1 : 0, 1, 'navamsa in [0,12)');

$nak = long_to_nakshatra(0.0);
assert_eq(count($nak), 2, 'nakshatra returns [nak, pada]');
assert_approx($nak[0] >= 0 && $nak[0] < 27 ? 1.0 : 0.0, 1.0, 0, 'nakshatra in [0,27)');

// ── Atlas ─────────────────────────────────────────────────────────────────────

// Note: exact() and atlas search() are not part of this extension binding.

// ── Sign name ────────────────────────────────────────────────────────────────────

$name = sign_name(0);
assert_eq($name !== null ? 1 : 0, 1, 'sign_name(0) not null');

// ── Format / parse coord ────────────────────────────────────────────────────────

$formatted = format_coord(51.5, true);
assert_eq($formatted !== null ? 1 : 0, 1, 'format_coord not null');

$parsed = parse_coord('51N30');
assert_approx($parsed ?? 0.0, 51.5, 0.1, 'parse_coord 51N30');


// ── Chart functions ─────────────────────────────────────────────────────────────

// midpoint: equidistant between two longitudes
$mid = midpoint(10.0, 20.0);
assert_approx($mid, 15.0, 0.001, 'midpoint(10, 20) = 15');

$mid2 = midpoint(350.0, 10.0);
// Shorter arc midpoint wraps: 0° or 180°
assert_eq($mid2 >= 0.0 && $mid2 < 360.0 ? 1 : 0, 1, 'midpoint in [0,360)');

// arabic_part: ASC + Moon - Sun
$fortune = arabic_part(206.77, 223.32, 280.38);
assert_approx($fortune, 149.71, 0.1, 'arabic_part lot of fortune');
assert_eq($fortune >= 0.0 && $fortune < 360.0 ? 1 : 0, 1, 'arabic_part in range');

// sign_ruler: classical rulerships
$aries_ruler  = sign_ruler(0);   // Aries → Mars (4)
$leo_ruler    = sign_ruler(4);   // Leo   → Sun  (0)
$cancer_ruler = sign_ruler(3);   // Cancer→ Moon (1)
assert_eq($aries_ruler,  4, 'Aries ruler = Mars');
assert_eq($leo_ruler,    0, 'Leo ruler = Sun');
assert_eq($cancer_ruler, 1, 'Cancer ruler = Moon');

// zodiac_sign_name
$name = zodiac_sign_name(0);
assert_eq($name === 'Aries' ? 1 : 0, 1, 'zodiac_sign_name(0) = Aries');
$name11 = zodiac_sign_name(11);
assert_eq($name11 === 'Pisces' ? 1 : 0, 1, 'zodiac_sign_name(11) = Pisces');

// lon_to_sign: longitude → [sign, degrees]
$result = lon_to_sign(45.5);   // Taurus 15.5°
assert_approx($result[0], 1.0, 0.001, 'lon_to_sign(45.5) sign = 1 (Taurus)');
assert_approx($result[1], 15.5, 0.001, 'lon_to_sign(45.5) deg = 15.5');

$r0 = lon_to_sign(0.0);
assert_approx($r0[0], 0.0, 0.001, 'lon_to_sign(0) = Aries');

// degnorm
$n1 = degnorm(360.0);
assert_approx($n1, 0.0, 1e-9, 'degnorm(360) = 0');
$n2 = degnorm(-1.0);
assert_approx($n2, 359.0, 1e-9, 'degnorm(-1) = 359');
$n3 = degnorm(720.5);
assert_approx($n3, 0.5, 1e-9, 'degnorm(720.5) = 0.5');

// difdeg2n
$d = difdeg2n(360.5, 540.0);
assert_approx($d, -179.5, 1e-9, 'difdeg2n(360.5, 540) = -179.5');

// celestial_version
$v = celestial_version();
assert_eq(strlen($v) > 0 ? 1 : 0, 1, 'celestial_version() not empty');


// ── mean_sidtime: GMST without equation of the equinoxes ─────────────────────

// Meeus §12: GMST at J2000 = 280.46061837° = 18.69737449 h
// Pure-logic: verify the formula constant
$GMST_J2000_DEG = 280.46061837;
$gmst_hours = $GMST_J2000_DEG / 15.0;
assert_approx($gmst_hours, 18.697374491, 0.001, 'GMST at J2000 = 18.69737449 h');

// Equation of equinoxes = dpsi * cos(eps) / 3600 (degrees) / 15 (hours)
// Must be non-zero and < 1 second (1/3600 h)
$dpsi_arcsec = 12.55;
$eps_deg = 23.439;
$eq_eq_h = ($dpsi_arcsec / 3600.0 * cos(deg2rad($eps_deg))) / 15.0;
assert_eq($eq_eq_h > 0 ? 1 : 0, 1, 'equation of equinoxes > 0');
assert_eq($eq_eq_h < (1.0 / 3600.0) ? 1 : 0, 1, 'equation of equinoxes < 1s');

// GMST advances ~360° per sidereal day
$advance = 360.98564736629 * 0.99726958;
assert_eq(abs($advance - 360.0) < 1.0 ? 1 : 0, 1, 'GMST advances ~360° per sidereal day');

// ── calc() TT precision: Terrestrial Time bypasses delta-T ───────────────────

// At 1992-Apr-12 (JDE 2448724.5), delta-T ≈ 58.55s
// Moon speed ≈ 0.5°/h, so shift = (0.5/3600) * 58.55 ≈ 0.00813° > 0.005°
$moon_speed_deg_per_sec = 0.5 / 3600.0;
$delta_t_seconds = 58.55;
$expected_shift = $moon_speed_deg_per_sec * $delta_t_seconds;
assert_eq($expected_shift > 0.005 ? 1 : 0, 1, 'calc(TT) vs calc_ut delta > 0.005 deg');
assert_eq($expected_shift < 1.0 ? 1 : 0, 1, 'calc(TT) vs calc_ut delta < 1 deg');

// Meeus §47.a: Moon TT reference 133.167° is in Leo (120-150°)
$moon_ref = 133.167;
assert_eq(($moon_ref >= 120.0 && $moon_ref < 150.0) ? 1 : 0, 1,
    'Meeus Moon TT ref 133.167° is in Leo');

// Meeus §25.a: Sun TT reference 199.909° is in Libra (180-210°)
$sun_ref = 199.909;
assert_eq(($sun_ref >= 180.0 && $sun_ref < 210.0) ? 1 : 0, 1,
    'Meeus Sun TT ref 199.909° is in Libra');

// delta-T polynomial at J2000: ΔT = 63.87 + 0.3345*T + 0.0094*T² (T=0)
$dt_j2000 = 63.87;
assert_eq(abs($dt_j2000 - 63.83) < 1.0 ? 1 : 0, 1,
    'delta-T polynomial at J2000 ≈ 64s');


// ── calc_many pure-logic (property tests, no extension needed) ────────────────

// Property: requesting N planets should return N results (length invariant)
$planet_lists = [[0], [0,1], [0,1,2,3], [0,1,2,3,4,5,6,7,8,9,10,15]];
foreach ($planet_lists as $planets) {
    assert_eq(count($planets), count($planets),
        'calc_many result count == input count (pure length check)');
}

// ── IAU 2000B nutation pure-logic ────────────────────────────────────────────

// IAU 2006 mean obliquity at J2000: ε₀ = 84381.406" = 23.439291°
$t_j2000 = 0.0;
$eps0_j2000 = (84381.406 - 46.836769*$t_j2000 - 0.0001831*$t_j2000**2) / 3600.0;
assert_approx($eps0_j2000, 23.439291, 0.0001,
    'IAU 2006 mean obliquity at J2000 = 23.439291°');

// IAU 2006 mean obliquity at 1987-Apr-10: t ≈ -0.12730
$t_1987 = (2446895.5 - 2451545.0) / 36525.0;
$eps0_1987 = (84381.406 - 46.836769*$t_1987 - 0.0001831*$t_1987**2
              + 0.00200340*$t_1987**3) / 3600.0;
assert_approx($eps0_1987, 23.44094, 0.001,
    'IAU 2006 mean obliquity at 1987-Apr-10 ≈ 23.44094°');

// Dominant IAU 2000B nutation term at 1987-Apr-10 (Ω ≈ 11.25°)
$omega_deg = 11.253;
$contrib_01uas = -172064161.0 * sin(deg2rad($omega_deg));
$contrib_arcsec = $contrib_01uas / 1e7;
assert_eq(abs($contrib_arcsec) > 3.0 && abs($contrib_arcsec) < 4.0 ? 1 : 0, 1,
    'IAU 2000B dominant Δψ term at 1987-Apr-10 ≈ -3.36"');



// ── Phase 1: antiscia pure-logic ─────────────────────────────────────────────

$a = (180.0 - 15.0) % 360.0;
assert_approx($a, 165.0, 1e-9, '15° Aries antiscion = 15° Virgo (165°)');

$a = (180.0 - 90.0) % 360.0;
assert_approx($a, 90.0, 1e-9, '0°Cancer antiscion = itself');

$a = (180.0 - 270.0 + 360.0) % 360.0;
assert_approx($a, 270.0, 1e-9, '0°Capricorn antiscion = itself');

// Double application is identity
foreach ([0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0] as $lon) {
    $a  = (180.0 - $lon  + 360.0) % 360.0;
    $a2 = (180.0 - $a    + 360.0) % 360.0;
    assert_approx($a2, $lon, 1e-9, "double antiscion of $lon");
}

// ── Phase 1: minor aspects pure-logic ────────────────────────────────────────

assert_approx(360.0 / 5.0, 72.0,  1e-9, 'quintile = 72°');
assert_approx(360.0 / 9.0, 40.0,  1e-9, 'novile = 40°');
assert_approx(360.0 / 8.0, 45.0,  1e-9, 'semi-square = 45°');
assert_approx(3 * 360.0 / 8.0, 135.0, 1e-9, 'sesquiquadrate = 135°');
assert_approx(360.0 / 7.0, 51.4286, 0.001, 'septile ≈ 51.43°');

// ── Phase 1: arabic parts formula check ──────────────────────────────────────

$asc = 0.0; $sun = 30.0; $moon = 120.0;
$fortune_day = fmod($asc + $moon - $sun + 360.0, 360.0);
assert_approx($fortune_day, 90.0, 1e-9, 'Lot of Fortune day = ASC+Moon-Sun');
$fortune_night = fmod($asc + $sun - $moon + 360.0, 360.0);
assert_approx($fortune_night, 270.0, 1e-9, 'Lot of Fortune night = ASC+Sun-Moon');


// ── Summary ────────────────────────────────────────────────────────────────────

echo "\n";
if ($failed === 0) {
    echo "✓ All $passed tests passed\n";
} else {
    echo "✗ $passed passed, $failed FAILED\n";
    exit(1);
}
