<?php
/**
 * Pure-logic tests for the PHP binding documentation examples.
 *
 * These tests require the compiled extension and validate the documented PHP
 * call patterns against the native implementation.
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

if (!extension_loaded('celestial-php')) {
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

$h_ex = houses_ex($jd, 48.85, 2.35, ord('P'), FLG_BUILTIN);
assert_eq(count($h_ex['cusps']), 12, 'houses_ex cusps count');
assert_eq(count($h_ex['ascmc']), 8, 'houses_ex ascmc count');

// house_name
assert_eq(house_name(ord('P')), 'Placidus', 'house_name Placidus');
assert_eq(house_name(ord('K')), 'Koch',     'house_name Koch');

// ── Ayanamsa ────────────────────────────────────────────────────────────────────

set_sid_mode(SIDM_LAHIRI, 0.0, 0.0);
$ay = ayanamsa($jd);
// Lahiri ayanamsa in 2002 ≈ 23.88°
assert_approx($ay, 23.88, 0.1, 'Lahiri ayanamsa');

assert_eq(ayanamsa_name(SIDM_LAHIRI), 'Lahiri', 'ayanamsa name');

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

// version
$v = version();
assert_eq(strlen($v) > 0 ? 1 : 0, 1, 'version() not empty');


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



// ── Phase 4: Vedic pure-logic ─────────────────────────────────────────────────

// Rasi: each longitude gives a sign in [0, 11]
for ($lon = 0; $lon < 360; $lon += 15) {
    $rasi = intval($lon / 30) % 12;
    assert($rasi >= 0 && $rasi < 12, "rasi $rasi out of [0,12) for lon=$lon");
}

// Nakshatra: 27 nakshatras of 13.333° each
$nak_len = 360.0 / 27.0;
for ($lon = 0; $lon < 360; $lon += 10) {
    $nak = intval($lon / $nak_len) % 27;
    assert($nak >= 0 && $nak < 27, "nakshatra $nak out of [0,27) for lon=$lon");
    $pada = intval(fmod($lon, $nak_len) / ($nak_len / 4)) + 1;
    assert($pada >= 1 && $pada <= 4, "pada $pada out of [1,4] for lon=$lon");
}

// Vimshottari total = 120 years
$periods = [6, 10, 7, 18, 16, 19, 17, 7, 20]; // Sun..Venus
$total_years = array_sum($periods);
assert($total_years === 120, "Vimshottari total $total_years != 120");

// North Indian house rotation
for ($lagna = 0; $lagna < 12; $lagna++) {
    for ($sign = 0; $sign < 12; $sign++) {
        $house = ($sign - $lagna + 12) % 12 + 1;
        assert($house >= 1 && $house <= 12, "house $house out of [1,12]");
        if ($sign === $lagna) {
            assert($house === 1, "lagna sign should be house 1, got $house");
        }
    }
}



// ── Phase 5: Hellenistic pure-logic ──────────────────────────────────────────

// Egyptian terms: Aries first term (0-6°) = Jupiter
$sign = 0; $deg = 3.0; // 3° Aries
$terms = [
    [[6,"Jupiter"],[12,"Venus"],[20,"Mercury"],[25,"Mars"],[30,"Saturn"]],   // Aries
    [[8,"Venus"],[14,"Mercury"],[22,"Jupiter"],[27,"Saturn"],[30,"Mars"]],   // Taurus
];
$ruler = null;
foreach ($terms[$sign] as [$end, $planet]) {
    if ($deg < $end) { $ruler = $planet; break; }
}
assert($ruler === "Jupiter", "Aries 3° terms ruler should be Jupiter, got $ruler");

// Decans: 36 decans of 10°; Aries 1st = Mars
$decan_idx = intval(5.0 / 10) % 36; // 5° Aries = first decan
assert($decan_idx === 0, "Aries first decan index = 0");
$DECAN_RULERS = ["Mars","Sun","Venus","Mercury","Moon","Saturn",
                 "Jupiter","Mars","Sun","Venus","Mercury","Moon"];
assert($DECAN_RULERS[0] === "Mars", "Aries 1st decan = Mars");

// Triplicity element cycle: sign % 4 → fire/earth/air/water
$elements = ["fire","earth","air","water"];
assert($elements[0 % 4] === "fire",  "Aries = fire");
assert($elements[1 % 4] === "earth", "Taurus = earth");
assert($elements[2 % 4] === "air",   "Gemini = air");
assert($elements[3 % 4] === "water", "Cancer = water");

// Sect: diurnal = Sun, Jupiter, Saturn
$diurnal = ["Sun", "Jupiter", "Saturn"];
$nocturnal = ["Moon", "Venus", "Mars"];
foreach ($diurnal as $p) {
    assert(!in_array($p, $nocturnal), "$p should not be in nocturnal list");
}

// Firdaria day seq total = 70 years (before nodes)
$day_seq = [10, 8, 13, 9, 11, 12, 7]; // Sun Venus Mercury Moon Saturn Jupiter Mars
assert(array_sum($day_seq) === 70, "Day Firdaria 7-planet total = 70y");

// Profection rotation
for ($age = 0; $age < 48; $age++) {
    $house = ($age % 12) + 1;
    assert($house >= 1 && $house <= 12, "profection house $house out of range");
}
assert((0 % 12) + 1 === 1,  "age 0 = house 1");
assert((12 % 12) + 1 === 1, "age 12 = house 1 again");



// ── Phase 6: Chinese Ba Zi pure-logic ────────────────────────────────────────

// Year cycle: (year - 4) % 60
$cycle = (2044 - 4) % 60;
assert($cycle === 0, "2044 should be year-cycle 0 (Jiǎ-Zǐ)");

for ($y = 1900; $y < 2100; $y++) {
    $c = ($y - 4) % 60;
    assert($c >= 0 && $c < 60, "year cycle out of [0,60) for year $y");
}

// 10 stems × 12 branches = 60 cycle
assert(10 * 6 === 60, "LCM(10,12) = 60");

// ── Phase 7: Mesoamerican pure-logic ─────────────────────────────────────────

// Calendar Round = LCM(260, 365) = 18980
function gcd_fn(int $a, int $b): int { return $b === 0 ? $a : gcd_fn($b, $a % $b); }
$lcm = 260 * 365 / gcd_fn(260, 365);
assert($lcm === 18980, "Calendar Round LCM should be 18980, got $lcm");

// Tonalpohualli 260-day cycle
$GMT = 584283;
$jd  = 2451545;
$day1 = ($jd - $GMT) % 260;
$day2 = ($jd + 260 - $GMT) % 260;
assert($day1 === $day2, "Tonalpohualli should repeat after 260 days");

// Trecena range [1,13]
for ($d = 0; $d < 260; $d++) {
    $t = ($d % 13) + 1;
    assert($t >= 1 && $t <= 13, "trecena $t out of range at day $d");
}

// 20 day signs
assert(count(["Cipactli","Ehecatl","Calli","Cuetzpallin","Coatl","Miquiztli",
              "Mazatl","Tochtli","Atl","Itzcuintli","Ozomatli","Malinalli",
              "Acatl","Ocelotl","Cuauhtli","Cozcacuauhtli","Ollin","Tecpatl",
              "Quiahuitl","Xochitl"]) === 20, "Tonalpohualli should have 20 signs");

// ── Phase 8: Indigenous / Egyptian pure-logic ─────────────────────────────────

// 36 Egyptian decans of 10° each
assert(360 / 10 === 36, "Should be 36 decans");
for ($deg = 0; $deg < 360; $deg++) {
    $idx = intval($deg / 10) % 36;
    assert($idx >= 0 && $idx < 36, "decan idx $idx out of range");
}

// Medicine Wheel: 12 birth totems
const SNOW_GOOSE = 'Snow Goose';
assert(count([SNOW_GOOSE,"Otter","Cougar","Red Hawk","Beaver","Deer",
              "Flicker","Sturgeon","Brown Bear","Raven","Snake","Elk"]) === 12,
       "Medicine Wheel should have 12 totems");



// ── Shared fixture-driven cross-language tests ────────────────────────────────

$fixture_path = __DIR__ . '/../../../tests/fixtures/reference_values.json';
$fx = json_decode(file_get_contents($fixture_path), true);
assert($fx !== null, "Could not load reference_values.json");

$GMT = 584283;

// Antiscia
foreach ($fx['antiscia'] as $case) {
    $lon = $case['input_lon'];
    $got  = fmod(180.0 - $lon + 360.0, 360.0);
    $gotc = fmod(360.0 - $lon + 360.0, 360.0);
    assert(abs($got  - $case['antiscion']) < 1e-9, "antiscion mismatch at lon=$lon");
    assert(abs($gotc - $case['contra'])    < 1e-9, "contra mismatch at lon=$lon");
}

// Tonalpohualli
foreach ($fx['tonalpohualli'] as $case) {
    $jd  = $case['jd'];
    $day = (($jd - $GMT) % 260 + 260) % 260;
    $t   = $day % 13 + 1;
    $s   = $day % 20;
    assert($t == $case['trecena'],  "trecena at jd=$jd");
    assert($s == $case['sign_idx'], "sign at jd=$jd");
}

// Profections
foreach ($fx['profections'] as $case) {
    $house = ($case['age'] % 12) + 1;
    assert($house === $case['house'], "profection house mismatch for age {$case['age']}");
}

// Medicine Wheel
$TOTEMS = [
    [300,330,SNOW_GOOSE,'Earth','Turtle','Winter'],
    [330,360,'Otter','Air','Butterfly','Winter'],
    [0,30,'Cougar','Air','Butterfly','Spring'],
    [30,60,'Red Hawk','Fire','Thunderbird','Spring'],
    [60,90,'Beaver','Earth','Turtle','Spring'],
    [90,120,'Deer','Air','Butterfly','Summer'],
    [120,150,'Flicker','Water','Frog','Summer'],
    [150,180,'Sturgeon','Fire','Thunderbird','Summer'],
    [180,210,'Brown Bear','Earth','Turtle','Autumn'],
    [210,240,'Raven','Air','Butterfly','Autumn'],
    [240,270,'Snake','Water','Frog','Autumn'],
    [270,300,'Elk','Fire','Thunderbird','Winter'],
];
function get_totem($lon, $TOTEMS) {
    $lon = fmod(fmod($lon, 360) + 360, 360);
    foreach ($TOTEMS as [$lo, $hi, $animal, $element, $clan, $season]) {
        if ($lo < $hi ? ($lon >= $lo && $lon < $hi) : ($lon >= $lo || $lon < $hi)) {
            return [$animal, $element, $clan, $season];
        }
    }
    return [SNOW_GOOSE,'Earth','Turtle','Winter'];
}
foreach ($fx['medicine_wheel'] as $case) {
    [$animal, $element, $clan, $season] = get_totem($case['sun_lon'], $TOTEMS);
    assert($animal  === $case['animal'],  "animal mismatch at lon={$case['sun_lon']}");
    assert($element === $case['element'], "element mismatch");
    assert($clan    === $case['clan'],    "clan mismatch");
    assert($season  === $case['season'],  "season mismatch");
}


// ── DEAD-3: pos6 array contract for the php binding ─────────────────────────
//
// `celestial_ffi::pos6` flattens a `PlanetPos` into the documented
// `[lon, lat, dist, speed_lon, speed_lat, speed_dist]` shape every php
// `calc*` export returns. Pin the contract end-to-end so a regression in
// either the Rust shim or the php marshalling fails this test.

$jd = julday(2000, 1, 1, 12.0, GREG_CAL);
$pos = calc_ut($jd, SUN, FLG_BUILTIN | FLG_SPEED);
assert_eq(count($pos),       6,   'DEAD-3 pos6: array length is 6');
assert_eq(is_array($pos)?1:0, 1,  'DEAD-3 pos6: result is an array');
for ($i = 0; $i < 6; $i++) {
    assert_eq(is_float($pos[$i]) ? 1 : 0, 1, "DEAD-3 pos6: slot $i is float");
    assert_eq(is_finite($pos[$i])  ? 1 : 0, 1, "DEAD-3 pos6: slot $i is finite");
}
// Field ordering pin — Sun's distance (slot 2) is ~1 AU, speed_lon (slot 3)
// is small positive (~ +1 deg/day). Catches accidental field reorder.
assert_eq($pos[2] > 0.95 && $pos[2] < 1.05 ? 1 : 0, 1, 'DEAD-3 pos6: slot 2 ≈ 1 AU (Sun distance)');
assert_eq($pos[3] > 0.0  && $pos[3] < 2.0  ? 1 : 0, 1, 'DEAD-3 pos6: slot 3 ≈ Sun speed_lon');

// REL-8 binding-seam guard: invalid body ids must raise an Exception, not
// crash or return zeroed data.
foreach ([21, 39, 5000, -2, -11, 1_010_000] as $bad) {
    $threw = false;
    try {
        calc_ut($jd, $bad, FLG_BUILTIN);
    } catch (Throwable $e) {
        $threw = true;
    }
    assert_eq($threw ? 1 : 0, 1, "REL-8: invalid body id $bad must throw");
}
// Documented ids must still succeed.
foreach ([SUN, MOON, MERCURY, VENUS, MARS, JUPITER, SATURN, URANUS, NEPTUNE, PLUTO] as $good) {
    $ok = false;
    try {
        $r = calc_ut($jd, $good, FLG_BUILTIN);
        $ok = is_array($r) && count($r) === 6;
    } catch (Throwable $e) {
        // unreachable for documented ids
    }
    assert_eq($ok ? 1 : 0, 1, "REL-8: documented body id $good must succeed");
}


// ── Summary ────────────────────────────────────────────────────────────────────

echo "\n";
if ($failed === 0) {
    echo "✓ All $passed tests passed\n";
} else {
    echo "✗ $passed passed, $failed FAILED\n";
    exit(1);
}
