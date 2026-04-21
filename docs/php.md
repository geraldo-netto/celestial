# celestial — PHP binding

**Extension:** `celestial` (ext-php-rs)  
**PHP version:** 8.1+  
**All functions** are in the global namespace with the `celestial_` prefix.

## Quick start

```bash
sudo apt-get install php-dev   # Ubuntu/Debian
cd bindings/php && cargo build --release
# Add to php.ini:  extension=/path/to/libcelestial.so
```

```php
<?php
$jd  = celestial_julday(2025, 3, 20, 9.0, GREG_CAL);
$sun = celestial_calc_ut($jd, SE_SUN, FLG_BUILTIN | FLG_SPEED);
printf("Sun  lon=%.4f°\n", $sun[0]);

$h = celestial_houses_ex($jd, 0, 48.85, 2.35, ord('P'));
printf("ASC=%.2f°  MC=%.2f°\n", $h['ascmc'][0], $h['ascmc'][1]);

// Parallel multi-body
$results = celestial_calc_many($jd, [SE_SUN, SE_MOON, SE_MERCURY], FLG_BUILTIN);

// Phase 5 — Hellenistic
$isDay = celestial_is_day_chart($sun[0], $h['cusps']);
[$dignityName, $score] = celestial_full_dignity(SE_SUN, $sun[0], $isDay);

// Phase 6 — Ba Zi
$pillars = celestial_four_pillars($jd, 9.0, $sun[0]);

// Phase 7 — Mesoamerican
$tonal = celestial_tonalpohualli($jd);

// Phase 8 — Medicine Wheel
$totem = celestial_medicine_wheel_totem($sun[0]);
```

All functions are prefixed `celestial_`. phpstan stubs at `bindings/php/phpstan-stubs.php`. For the full PHP API reference see [docs/php.md](docs/php.md).

---

---

## Installation

### From source

```bash
# Install PHP development headers
sudo apt-get install php-dev   # Ubuntu / Debian
sudo dnf install php-devel     # Fedora / RHEL
brew install php               # macOS

# Build the extension
cd bindings/php
cargo build --release

# Add to php.ini
extension=/path/to/target/release/libcelestial.so   # Linux
extension=/path/to/target/release/libcelestial.dylib  # macOS
```

### phpstan integration

```neon
# phpstan.neon
parameters:
    bootstrapFiles:
        - vendor/path/to/bindings/php/phpstan-stubs.php
```

Stubs file: `bindings/php/phpstan-stubs.php`

---

## Constants

### Body indices

```php
define('SE_SUN',       0);
define('SE_MOON',      1);
define('SE_MERCURY',   2);
define('SE_VENUS',     3);
define('SE_MARS',      4);
define('SE_JUPITER',   5);
define('SE_SATURN',    6);
define('SE_URANUS',    7);
define('SE_NEPTUNE',   8);
define('SE_PLUTO',     9);
define('SE_MEAN_NODE', 10);
define('SE_TRUE_NODE', 11);
define('SE_CHIRON',    15);
```

### Calculation flags

```php
define('FLG_BUILTIN',    2);
define('FLG_SPEED',      256);
define('FLG_SIDEREAL',   64);
define('FLG_EQUATORIAL', 2048);
define('FLG_HELCTR',     8);
```

### Sidereal modes

```php
define('SIDM_FAGAN_BRADLEY', 0);
define('SIDM_LAHIRI',        1);
define('SIDM_RAMAN',         3);
define('SIDM_KRISHNAMURTI',  5);
```

---

## Core functions

### Time

```php
// Calendar → Julian Day
$jd = celestial_julday(2025, 3, 20, 9.0, GREG_CAL);

// Julian Day → calendar date
$date = celestial_revjul($jd, GREG_CAL);
// $date['year'], $date['month'], $date['day'], $date['hour']

// Current Julian Day
$now = celestial_jdnow();
```

### Planetary positions

```php
// Single body
// Returns [lon, lat, dist, speed_lon, speed_lat, speed_dist, ret_flags]
$sun = celestial_calc_ut($jd, SE_SUN, FLG_BUILTIN | FLG_SPEED);
printf("Sun lon=%.4f°  dist=%.6f AU\n", $sun[0], $sun[2]);

// Multiple bodies in parallel
// Returns array of [lon, lat, dist, speed_lon, ...] for each body
$bodies  = [SE_SUN, SE_MOON, SE_MERCURY, SE_VENUS, SE_MARS,
            SE_JUPITER, SE_SATURN, SE_URANUS, SE_NEPTUNE,
            SE_PLUTO, SE_MEAN_NODE, SE_CHIRON];
$results = celestial_calc_many($jd, $bodies, FLG_BUILTIN | FLG_SPEED);
// $results[$i] corresponds to $bodies[$i]

// Nutation (IAU 2000B)
// Returns ['dpsi' => float, 'deps' => float, 'eps_true' => float]
$nut = celestial_nutation($jd, FLG_BUILTIN);
printf("dpsi=%.6f°  eps_true=%.4f°\n", $nut['dpsi'], $nut['eps_true']);

// Fixed star
$star = celestial_fixstar_ut('Aldebaran', $jd, FLG_BUILTIN);
$mag  = celestial_fixstar_mag('Aldebaran');
```

### Houses

```php
// Returns ['cusps' => float[13], 'ascmc' => float[8]]
// cusps[1..12] are the house cusps; cusps[0] is unused
// ascmc[0]=ASC, ascmc[1]=MC, ascmc[2]=ARMC, ascmc[3]=Vertex
$h = celestial_houses_ex($jd, 0, 48.85, 2.35, ord('P'));
printf("ASC=%.2f°  MC=%.2f°\n", $h['ascmc'][0], $h['ascmc'][1]);

// With flags (e.g. sidereal)
$hSid = celestial_houses_ex($jd, FLG_SIDEREAL, 48.85, 2.35, ord('P'));
```

**House system codes:** `ord('P')` Placidus · `ord('K')` Koch · `ord('E')` Equal · `ord('W')` Whole-Sign · `ord('O')` Porphyry · `ord('R')` Regiomontanus

### Sidereal positions

```php
celestial_set_sid_mode(SIDM_LAHIRI, 0.0, 0.0);
$moon = celestial_calc_ut($jd, SE_MOON, FLG_BUILTIN | FLG_SIDEREAL);
printf("Moon (Lahiri) = %.4f°\n", $moon[0]);

$ayan = celestial_ayanamsa_ut($jd);
printf("Lahiri ayanamsa = %.4f°\n", $ayan);
```

---

## Moon phases

```php
$phase = celestial_moon_phase($jd);          // int (phase variant)
$illum = celestial_moon_illumination($jd);   // float 0.0–1.0
$elong = celestial_moon_elongation($jd);     // float 0°–360°

$info  = celestial_moon_phase_info($jd);
// $info['phase_name'], $info['illumination'], $info['age_days']
// $info['next_phase_name'], $info['next_phase_jd']
printf("%s  %.1f%%\n", $info['phase_name'], $info['illumination'] * 100);

$newMoon  = celestial_next_new_moon($jd);
$fullMoon = celestial_next_full_moon_phase($jd);
```

---

## Celtic calendar

```php
$sabbats = celestial_sabbats_for_year(2025);
foreach ($sabbats as $s) {
    $d = celestial_revjul($s['jd'], GREG_CAL);
    printf("%-14s  %04d-%02d-%02d\n", $s['name'], $d['year'], $d['month'], $d['day']);
}

$esbats = celestial_esbats_for_year(2025);
foreach ($esbats as $m) {
    $d = celestial_revjul($m['jd'], GREG_CAL);
    printf("%-18s  %04d-%02d-%02d\n", $m['display_name'], $d['year'], $d['month'], $d['day']);
}
```

---

## Vedic / Jyotish

```php
celestial_set_sid_mode(SIDM_LAHIRI, 0.0, 0.0);
$moon = celestial_calc_ut($jd, SE_MOON, FLG_BUILTIN | FLG_SIDEREAL);

[$nak, $pada] = celestial_long_to_nakshatra($moon[0]);
printf("Nakshatra: %s, pada %d\n", celestial_nakshatra_name($nak), $pada + 1);

$rasi    = celestial_long_to_rasi($moon[0]);     // 0–11
$navamsa = celestial_long_to_navamsa($moon[0]);  // 0–11

$dashas  = celestial_vimshottari_dasha($jd, $moon[0], 120.0);
foreach (array_slice($dashas, 0, 3) as $d) {
    printf("%s dasha: %.1f years\n", $d['planet_name'], $d['years']);
}
```

---

## Phase 5 — Hellenistic / Persian

```php
$sun = celestial_calc_ut($jd, SE_SUN, FLG_BUILTIN);
$h   = celestial_houses_ex($jd, 0, 48.85, 2.35, ord('P'));

// Day or night chart
$isDay = celestial_is_day_chart($sun[0], $h['cusps']);

// Egyptian terms ruler (planet index 0–6)
$ruler = celestial_egyptian_terms_ruler($sun[0]);

// Chaldean decan ruler
$decan = celestial_decan_ruler($sun[0]);

// Full dignity: [dignity_name, score]
[$dignityName, $score] = celestial_full_dignity(SE_SUN, $sun[0], $isDay);
printf("Sun dignity: %s (score %d)\n", $dignityName, $score);

// Almuten: [body_raw, score]
[$almutenBody, $almutenScore] = celestial_almuten($sun[0], $isDay);

// Firdaria periods: array of ['major_lord', 'minor_lord', 'start_jd', 'end_jd', 'years']
$periods = celestial_firdaria($jd, $isDay, 75.0);
foreach (array_slice($periods, 0, 3) as $p) {
    printf("Major: %d  Minor: %d  Start JD: %.1f\n",
        $p['major_lord'], $p['minor_lord'], $p['start_jd']);
}

// Annual profection: [house_number, profected_lon]
[$houseNum, $profLon] = celestial_annual_profection($h['cusps'], 35);
printf("Age 35 → House %d (%.2f°)\n", $houseNum, $profLon);
```

---

## Phase 6 — Chinese astrology (Ba Zi)

```php
$sun     = celestial_calc_ut($jd, SE_SUN, FLG_BUILTIN);
$pillars = celestial_four_pillars($jd, 9.0, $sun[0]);

$labels = ['Year', 'Month', 'Day', 'Hour'];
foreach ($pillars as $i => $pillar) {
    printf("%s: %s %s (%s)\n",
        $labels[$i], $pillar['stem_name'], $pillar['branch_name'], $pillar['animal']);
}

// Solar term: [current_idx, deg_into, next_idx, deg_to_next]
[$curIdx, $degInto, $nextIdx, $degToNext] = celestial_solar_term_position($sun[0]);
```

---

## Phase 7 — Mesoamerican calendars

```php
// Aztec Tonalpohualli: [trecena, sign_idx, nahuatl_name, english]
$tonal = celestial_tonalpohualli($jd);
printf("Tonalpohualli: %d %s (%s)\n", $tonal[0], $tonal[2], $tonal[3]);

// Aztec Xiuhpohualli: [month_idx, day, name, english]
$xiu = celestial_xiuhpohualli($jd);

// Maya Tzolkin: [trecena, sign_idx, mayan_name, english]
$tzolkin = celestial_tzolkin($jd);

// Maya Haab: [month_idx, day, name]
$haab = celestial_haab($jd);

// Calendar Round: [tz_trecena, tz_sign, haab_day, haab_month]
$cr = celestial_calendar_round($jd);
```

---

## Phase 8 — Indigenous / Egyptian

```php
$sun = celestial_calc_ut($jd, SE_SUN, FLG_BUILTIN);

// Medicine Wheel: [animal, element, clan, season]
$totem = celestial_medicine_wheel_totem($sun[0]);
printf("Totem: %s — %s element, %s clan, %s\n",
    $totem[0], $totem[1], $totem[2], $totem[3]);

// Egyptian decan: [idx, decan_name, rising_star]
$decan = celestial_egyptian_decan($sun[0]);
printf("Decan %d: %s (rising star: %s)\n", $decan[0] + 1, $decan[1], $decan[2]);
```

---

## Error handling

Functions return `null` on failure:

```php
$pos = celestial_calc_ut($jd, SE_SUN, FLG_BUILTIN);
if ($pos === null) {
    error_log('celestial_calc_ut failed');
    return;
}
```

---

## Return type reference

| Function | Returns |
|---|---|
| `celestial_calc_ut` | `[lon, lat, dist, speed_lon, speed_lat, speed_dist, ret]` |
| `celestial_calc_many` | `array` of the above arrays |
| `celestial_houses_ex` | `['cusps' => float[13], 'ascmc' => float[8]]` |
| `celestial_nutation` | `['dpsi' => float, 'deps' => float, 'eps_true' => float]` |
| `celestial_revjul` | `['year' => int, 'month' => int, 'day' => int, 'hour' => float]` |
| `celestial_moon_phase_info` | `['phase_name', 'illumination', 'age_days', 'next_phase_name', …]` |
| `celestial_four_pillars` | `array[4]` of `['stem_name', 'branch_name', 'animal', …]` |
| `celestial_firdaria` | `array` of `['major_lord', 'minor_lord', 'start_jd', 'end_jd', 'years']` |
| `celestial_full_dignity` | `[string $name, int $score]` |
| `celestial_almuten` | `[int $body, int $score]` |
| `celestial_is_day_chart` | `bool` |
| `celestial_tonalpohualli` | `[int $trecena, int $sign_idx, string $name, string $english]` |
| `celestial_medicine_wheel_totem` | `[string $animal, string $element, string $clan, string $season]` |
| `celestial_egyptian_decan` | `[int $idx, string $name, string $rising_star]` |
