# celestial — API & Rust Reference

This document is the unified reference for `celestial-core`.  
Language-specific guides: [python.md](python.md) · [javascript.md](javascript.md) · [php.md](php.md)

---

## Add as dependency

```toml
[dependencies]
celestial-core = { path = "../core" }
```

### Import styles

All symbols are available at the crate root (backward-compatible) and via their domain module (preferred):

```rust
// Preferred — explicit, IDE-friendly
use celestial_core::position::{calc_ut, CalcOptions};
use celestial_core::motion::{RiseTransOptions, SearchOptions};
use celestial_core::chart::{AspectOrbs, full_dignity, firdaria};
use celestial_core::moon::{moon_phase, sabbats_for_year};
use celestial_core::calendar::{jewish_holidays, easter_gregorian};
use celestial_core::body::{Body, CalcFlags, HouseSystem};

// Convenience — import everything at once
use celestial_core::prelude::*;

// Still works — crate root re-exports all domain modules
use celestial_core::calc_ut;
```

The public API is a single flat namespace:

```rust
use celestial_core::*;
```

---

## Error types

```rust
/// #[non_exhaustive] — match arms must include `_` wildcard.
pub enum Error {
    // Structured variants (carry typed fields for programmatic inspection)
    BodyNotImplemented { body: i32 },
    StarNotFound       { name: String },
    PhaseNotFound      { phase: String, from_jd: f64 },
    NoEclipseFound     { from_jd: f64 },
    CircumpolarBody    { body: i32, lat: f64 },
    HouseSystemFailed  { system: u8, lat: f64 },

    // Legacy string variants (kept for backward compatibility)
    Calc(String),
    Houses(String),
    Eclipse(String),
    RiseTrans(String),
    Date(String),
}
```

All variants implement `Display` with a human-readable message, so `e.to_string()` always works regardless of which variant is returned.

---

## Serializable output types

The following structs derive `serde::Serialize` and `serde::Deserialize` and can be used directly with `serde_json::to_value()` or any serde format:

| Struct | Fields |
|---|---|
| `ChartAspect` | `body1`, `body2`, `aspect`, `orb`, `applying` |
| `Stations` | `retrograde`, `direct` |
| `ArabicPart` | `name`, `formula`, `degree` |
| `DashaLevel` | `body`, `start`, `end`, `years` |

---

## Time & calendar

| Function | Signature | Description |
|---|---|---|
| `julday` | `(y, m, d, h, cal) → f64` | Calendar date → Julian day number |
| `revjul` | `(jd, cal) → CalDate` | Julian day → calendar date |
| `utc_to_jd` | `(date, cal) → JdPair` | UTC → `(jd_et, jd_ut)` |
| `deltat` | `(jd) → f64` | ΔT = TT − UT1 in days |
| `sidtime` | `(jd_ut) → f64` | Greenwich Apparent Sidereal Time (hours) |
| `mean_sidtime` | `(jd) → f64` | Mean sidereal time (degrees) |
| `day_of_week` | `(jd) → u8` | 0 = Sunday … 6 = Saturday |
| `jdnow` | `() → f64` | Current Julian day (UTC) |

**`CalDate` fields:** `year` `month` `day` `hour` `cal`

---

## Planetary positions

| Function | Signature | Description |
|---|---|---|
| `calc_ut` | `(jd, body, flags) → Result<PlanetPos>` | Geocentric position (UT input) |
| `calc` | `(jd, body, flags) → Result<PlanetPos>` | Geocentric position (TT/ET input) |
| `calc_many` | `(jd, &[body], flags) → Result<Vec<PlanetPos>>` | Parallel multi-body (TT) |
| `calc_ut_many` | `(jd, &[body], flags) → Result<Vec<PlanetPos>>` | Parallel multi-body (UT) |
| `calc_pctr` | `(jd, body, center, flags) → Result<PlanetPos>` | Position relative to center |
| `fixstar_ut` | `(name, jd, flags) → Result<FixStarPos>` | Fixed star (UT) |
| `fixstar_mag` | `(name) → Result<f64>` | Fixed star visual magnitude |
| `nutation` | `(jd, flags) → Result<NutationResult>` | IAU 2000B nutation + obliquity |
| `nod_aps` | `(jd, body, flags, method) → Result<NodAps>` | Nodes and apsides |

**`PlanetPos` fields:**

| Field | Type | Description |
|---|---|---|
| `lon` | `f64` | Ecliptic longitude (degrees) |
| `lat` | `f64` | Ecliptic latitude (degrees) |
| `dist` | `f64` | Distance (AU for planets, light-years for stars) |
| `speed_lon` | `f64` | Daily speed in longitude (°/day) |
| `speed_lat` | `f64` | Daily speed in latitude |
| `speed_dist` | `f64` | Daily change in distance |
| `ret_flags` | `i32` | Return flags from the calculation |

**Body constants:**

| Constant | Value | Body |
|---|---|---|
| `SUN` | 0 | Sun |
| `MOON` | 1 | Moon |
| `MERCURY` | 2 | Mercury |
| `VENUS` | 3 | Venus |
| `MARS` | 4 | Mars |
| `JUPITER` | 5 | Jupiter |
| `SATURN` | 6 | Saturn |
| `URANUS` | 7 | Uranus |
| `NEPTUNE` | 8 | Neptune |
| `PLUTO` | 9 | Pluto |
| `MEAN_NODE` | 10 | Mean Lunar Node |
| `TRUE_NODE` | 11 | True Lunar Node |
| `CHIRON` | 15 | Chiron |

**Calculation flags:**

| Flag | Description |
|---|---|
| `FLG_BUILTIN` | Use built-in ephemeris (required) |
| `FLG_SPEED` | Include daily speed in result |
| `FLG_SIDEREAL` | Sidereal positions (requires `set_sid_mode` first) |
| `FLG_EQUATORIAL` | Equatorial coordinates (RA/Dec) |
| `FLG_HELCTR` | Heliocentric positions |
| `FLG_TOPOCTR` | Topocentric (requires `set_topo` first) |
| `FLG_NONUT` | Suppress nutation correction |
| `FLG_XYZ` | Cartesian (x, y, z) output |

---

## Configuration

| Function | Signature | Description |
|---|---|---|
| `set_sid_mode` | `(mode, t0, ayan_t0)` | Activate a sidereal ayanamsa |
| `set_topo` | `(lon, lat, alt_m)` | Set topocentric observer position |
| `ayanamsa_ut` | `(jd_ut) → f64` | Ayanamsa for the active mode |
| `ayanamsa` | `(jd_et) → f64` | Ayanamsa (TT input) |
| `ayanamsa_name` | `(mode) → &str` | Name of a sidereal mode |
| `planet_name` | `(body) → &str` | Body number → display name |

**Sidereal modes:**

| Constant | Value | Mode |
|---|---|---|
| `SIDM_FAGAN_BRADLEY` | 0 | Fagan-Bradley |
| `SIDM_LAHIRI` | 1 | Lahiri (official Indian) |
| `SIDM_DELUCE` | 2 | DeLuce |
| `SIDM_RAMAN` | 3 | B.V. Raman |
| `SIDM_KRISHNAMURTI` | 5 | Krishnamurti |
| `SIDM_SASSANIAN` | 11 | Sassanian |
| `SIDM_USER` | 255 | User-defined |

---

## Houses

| Function | Signature | Description |
|---|---|---|
| `houses` | `(jd, lat, lon, sys) → Result<HouseResult>` | House cusps + angles |
| `houses_ex` | `(jd, flags, lat, lon, sys) → Result<HouseResult>` | With sidereal/topocentric flags |
| `house_pos` | `(armc, lat, eps, sys, pos) → Result<f64>` | House position of a body |
| `house_name` | `(sys) → &str` | System byte → name |

**`HouseResult` fields:**

| Field | Description |
|---|---|
| `cusps[12]` | House cusps; index 0 unused, 1–12 are the twelve house cusps |
| `ascmc[0]` | Ascendant (ASC) |
| `ascmc[1]` | Midheaven (MC) |
| `ascmc[2]` | ARMC (sidereal time × 15) |
| `ascmc[3]` | Vertex |
| `ascmc[4]` | Equatorial ASC |
| `ascmc[7]` | Polar ASC |

**House systems:**

| Byte | Name |
|---|---|
| `b'P'` | Placidus |
| `b'K'` | Koch |
| `b'E'` | Equal (from ASC) |
| `b'W'` | Whole-Sign |
| `b'O'` | Porphyry |
| `b'R'` | Regiomontanus |
| `b'C'` | Campanus |
| `b'M'` | Morinus |
| `b'B'` | Alcabitus |
| `b'X'` | Axial Rotation |
| `b'H'` | Azimuthal / Horizontal |

---

## Moon phases

| Function | Signature | Description |
|---|---|---|
| `moon_phase` | `(jd) → MoonPhase` | Named phase (8 variants) |
| `moon_illumination` | `(jd) → f64` | Fraction illuminated 0.0–1.0 |
| `moon_elongation` | `(jd) → f64` | Moon–Sun elongation 0°–360° |
| `next_new_moon` | `(jd) → f64` | JD of next new moon |
| `next_first_quarter` | `(jd) → f64` | JD of next first quarter |
| `next_full_moon_phase` | `(jd) → f64` | JD of next full moon |
| `next_last_quarter` | `(jd) → f64` | JD of next last quarter |
| `moon_phases_for_month` | `(year, month) → Vec<PhaseEvent>` | All phases in a month |
| `moon_phase_info` | `(jd) → MoonPhaseInfo` | Rich phase info with prev/next |

**`MoonPhase` variants:** `NewMoon` · `WaxingCrescent` · `FirstQuarter` · `WaxingGibbous` · `FullMoon` · `WaningGibbous` · `LastQuarter` · `WaningCrescent`

**`MoonPhaseInfo` fields:** `phase` · `phase_name` · `elongation` · `illumination` · `age_days` · `prev_phase_jd` · `prev_phase_name` · `next_phase_jd` · `next_phase_name`

`SYNODIC_MONTH = 29.530_588_853` days is exported as a constant.

---

## Calendars & religious observances

### Celtic

| Function | Description |
|---|---|
| `sabbat_jd(year, kind)` | Exact JD of a specific sabbat |
| `sabbats_for_year(year)` | All 8 sabbats sorted chronologically |
| `next_sabbat(jd)` | Next sabbat at or after `jd` |
| `esbats_for_year(year)` | All named full moons |
| `next_esbat(jd)` | Next named full moon |

**`SabbatKind`:** `Yule(270°)` · `Imbolc(315°)` · `Ostara(0°)` · `Beltane(45°)` · `Litha(90°)` · `Lughnasadh(135°)` · `Mabon(180°)` · `Samhain(225°)`

### Jewish

| Function | Description |
|---|---|
| `jewish_holidays(hebrew_year)` | All major holidays |
| `jewish_holiday_jd(year, name)` | JD of a specific holiday |
| `jd_to_hebrew_date(jd)` | JD → `(year, month, day)` |
| `omer_from_jd(jd)` | Tonight's Omer day |
| `omer_days(hebrew_year)` | Full 49-day schedule |
| `omer_day_jd(year, day)` | JD of a specific Omer day |
| `omer_declaration(day)` | Traditional Omer declaration text |

### Easter & Christian

| Function | Description |
|---|---|
| `easter_gregorian(year)` | Western Easter → `(y, m, d)` |
| `easter_orthodox(year)` | Orthodox Easter → `(y, m, d)` |
| `christian_feasts(year)` | All moveable feasts |
| `christian_fixed_feasts(year)` | Fixed feasts (Christmas, Epiphany…) |

### Islamic

| Function | Description |
|---|---|
| `hijri_from_jd(jd)` | JD → `(year, month, day)` |
| `hijri_month_name(month)` | Month number → Arabic name |
| `islamic_observances(hijri_year)` | Major observances |
| `gregorian_to_hijri_years(year)` | Overlapping Hijri years |

### Hindu

| Function | Description |
|---|---|
| `panchanga(jd)` | Full Panchānga for a JD |
| `hindu_festivals(year)` | Major festivals for a Gregorian year |

**`PanchangaResult` fields:** `tithi_name` · `paksha` · `vara_name` · `nakshatra_name` · `nakshatra_pada` · `yoga_name` · `karana_name`

### Buddhist

| Function | Description |
|---|---|
| `vesak_jd(year)` | Vesak (Buddha Day) JD |
| `uposatha_days(year)` | All four Uposatha phases |

### Nowruz, Persian & Bahá'í

| Function | Description |
|---|---|
| `nowruz_jd(year)` | Exact Nowruz JD (vernal equinox) |
| `gregorian_to_solar_hijri(year)` | Gregorian → Solar Hijri year |
| `naw_ruz_jd(bahai_year)` | Bahá'í Naw-Rúz JD |
| `jd_to_bahai(jd)` | JD → `BahaiDate` |
| `bahai_holy_days(bahai_year)` | 13 Bahá'í holy days |

**`BahaiDate`:** `year` (BE) · `month` (1–19, 0=Ayyám-i-Há) · `day` · `month_name`

---

## Builder structs

### `CalcOptions` — unified planetary calculation

Replaces `calc_ut`, `calc`, `calc_many`, `calc_ut_many` with a single discoverable API.

```rust
use celestial_core::{CalcOptions, CalcStrategy, Body, CalcFlags};

// Single body (UT)
let pos = CalcOptions::ut(jd, CalcFlags::BUILTIN | CalcFlags::SPEED)
    .body(Body::SUN)
    .get()?;

// Multiple bodies — Auto strategy (sequential ≤2, parallel >2)
let results = CalcOptions::ut(jd, CalcFlags::BUILTIN)
    .bodies(&[Body::SUN, Body::MOON, Body::MERCURY])
    .get_many();

// Force sequential
let results = CalcOptions::ut(jd, CalcFlags::BUILTIN)
    .strategy(CalcStrategy::Sequential)
    .bodies(&[Body::SUN, Body::MOON])
    .get_many();
```

**`CalcStrategy` variants:** `Sequential` · `Parallel` · `Auto` (default)

### `RiseTransOptions` — rise/transit/set

```rust
use celestial_core::{RiseTransOptions, Body, CalcFlags};

let result = RiseTransOptions::new(jd, Body::MOON, [lon, lat, alt_m])
    .event(1)                      // 1=rise, 2=set, 4=upper transit
    .atmosphere(1013.25, 15.0)     // pressure mb, temperature °C
    .flags(CalcFlags::BUILTIN)
    .search()?;
println!("Rises at JD {}", result.tret);
```

### `SearchOptions` — cusp-aspect and angle-transit searches

```rust
use celestial_core::{SearchOptions, Body, CalcFlags, HouseSystem};

// Aspect to house cusp
let hit = SearchOptions::new(Body::SATURN, jd_start)
    .aspect(90.0)
    .cusp(10, lat, lon, HouseSystem::PLACIDUS)
    .search_cusp();

// Natal angle transits
let jd = SearchOptions::new(Body::SATURN, jd_start)
    .natal_chart(jd_natal, lat, lon, HouseSystem::PLACIDUS)
    .search_mc_transit()?;
// Also: .search_ic_transit() · .search_asc_transit() · .search_dsc_transit()
```

### `AspectOrbs` — fine-grained aspect matching

```rust
use celestial_core::AspectOrbs;

let m = AspectOrbs::new(2.0, 1.5)   // applying_orb, separating_orb
    .def_orb(2.0)
    .check(pos0, speed0, pos1, speed1, 120.0);  // trine

assert!(m.matched);
println!("Orb: {:.2}°  Applying: {}", m.diff.abs(), m.diff < 0.0);
```

---

## Crossings, rise/set & eclipses

| Function | Description |
|---|---|
| `solcross_ut(lon, jd, flags)` | Next solar ecliptic longitude crossing |
| `mooncross_ut(lon, jd, flags)` | Next lunar ecliptic longitude crossing |
| `helio_cross_ut(body, lon, jd, flags, dir)` | Heliocentric crossing |
| `mooncross_node(jd, flags)` | Next Moon/node crossing |
| `rise_trans(jd, body, star, flags, rsmi, geo, press, temp)` | Rise / transit / set |
| `sol_eclipse_when_glob(jd, flags, type, back)` | Next solar eclipse |
| `lun_eclipse_when(jd, flags, type, back)` | Next lunar eclipse |

## Complete chart calculation

```rust
use celestial_core::*;

fn main() -> Result<()> {
    let jd  = julday(1985, 7, 14, 12.0, GREG_CAL);
    let lat = 48.85;
    let lon = 2.35;

    // Positions
    let bodies = [SUN, MOON, MERCURY, VENUS, MARS, JUPITER, SATURN,
                  URANUS, NEPTUNE, PLUTO, MEAN_NODE, CHIRON];
    let positions = calc_many(jd, &bodies, FLG_BUILTIN | FLG_SPEED)?;

    // Aspects
    let pos_pairs: Vec<(i32, f64, f64)> = positions.iter().zip(bodies.iter())
        .map(|(p, &b)| (b, p.lon, p.speed_lon)).collect();
    let aspects = calc_chart_aspects(&pos_pairs, MAJOR_ASPECTS, 8.0);
    for a in &aspects {
        println!("{} {} {} orb={:.2}°",
            planet_name(a.body1), a.aspect, planet_name(a.body2), a.orb);
    }

    // Solar return
    let jd_sr = solar_return_jd(jd, 2025, FLG_BUILTIN)?;
    let d     = revjul(jd_sr, GREG_CAL);
    println!("Solar return 2025: {:04}-{:02}-{:02}", d.year, d.month, d.day);

    // Secondary progressions (35 years)
    let (prog, _) = secondary_progressions(jd, 35.0, &bodies, lat, lon, b'P', FLG_BUILTIN)?;
    let prog_sun  = prog[0].1.lon;
    let prog_moon = prog[1].1.lon;
    println!("Prog Sun={:.2}°  Prog Moon={:.2}°", prog_sun, prog_moon);

    // Midpoint
    let mid = midpoint_deg(prog_sun, prog_moon);
    println!("Sun-Moon midpoint: {:.2}°", mid);

    // Vedic — Vimshottari dasha
    set_sid_mode(SIDM_LAHIRI, 0.0, 0.0);
    let moon_sid = calc_ut(jd, MOON, FLG_BUILTIN | FLG_SIDEREAL)?;
    let dashas   = vimshottari_dasha(jd, moon_sid.lon, 120.0);
    for d in dashas.iter().take(3) {
        println!("{} dasha: {:.1} years", planet_name(d.planet), d.years);
    }

    // Hellenistic — dignity
    let h      = houses(jd, lat, lon, b'P')?;
    let sun    = positions[0];
    let is_day = is_day_chart(sun.lon, &h.cusps);
    let (dig, score) = full_dignity(SUN, sun.lon, is_day)?;
    println!("Sun dignity: {dig:?} (score {score})");

    // Ba Zi
    let pillars = four_pillars(jd, 12.0, sun.lon);
    for (i, p) in pillars.iter().enumerate() {
        let label = ["Year", "Month", "Day", "Hour"][i];
        println!("{label}: {} {}", p.stem_name, p.branch_name);
    }

    // Tonalpohualli
    let (trecena, _, name, _) = tonalpohualli(jd);
    println!("Aztec day: {trecena} {name}");

    // Medicine Wheel
    let (animal, element, clan, season) = medicine_wheel_totem(sun.lon);
    println!("Totem: {animal} ({element}, {clan}, {season})");

    // Hellenistic dignities
    let (dig, score) = full_dignity(Body::SUN, sun.lon, is_day_chart(sun.lon, &h.cusps))?;
    println!("Sun dignity: {dig:?} (score {score})");

    let periods = firdaria(jd_natal, is_day_chart(sun.lon, &h.cusps), 75.0);
    println!("First firdaria lord: {}", planet_name(periods[0].major_lord));

    let (house, _) = annual_profection(&h.cusps, 35);
    println!("Age 35 profection: house {house}");

    Ok(())
}
```

---

## Aspects & searches

| Function | Description |
|---|---|
| `calc_chart_aspects(positions, aspects, orb)` | All aspects in a chart |
| `calc_chart_aspects_auto(positions, orbs)` | Aspects with per-planet orb table |
| `match_aspect(p0, s0, p1, s1, aspect, orb)` | Test if aspect is within orb |
| `next_retro(body, jd, back, days, flags)` | Next retrograde station |
| `next_aspect(body, aspect, fixed, jd, …)` | Aspect to a fixed point |
| `next_aspect_with(body, aspect, other, jd, …)` | Aspect between two bodies |
| `next_aspect_cusp(body, aspect, cusp, jd, …)` | Aspect to a house cusp |
| `sign_ingress_ut(body, jd, flags, back)` | Next sign ingress |
| `retrograde_station_ut(body, jd, flags)` | Retrograde and direct station JDs |

---

## Vedic / Jyotish

| Function | Description |
|---|---|
| `long_to_rasi(lon)` | Ecliptic longitude → rasi (sign) 0–11 |
| `long_to_navamsa(lon)` | Longitude → navamsa sign 0–11 |
| `long_to_nakshatra(lon)` | Longitude → `(nakshatra 0–26, pada 0–3)` |
| `nakshatra_name(n)` | Nakshatra number → name string |
| `raman_houses(asc, mc, sandhi)` | 12 Raman house cusps |
| `vimshottari_dasha(jd, moon_lon, span)` | Dasha period list |
| `ochchabala(graha, lon)` | Exaltation strength 0–60 |
| `tatkalika_relation(g1, g2)` | Temporary relationship −1/0/1 |
| `naisargika_relation(g1, g2)` | Natural relationship −1/0/1 |
| `residential_strength(lon, cusps)` | Bhava bala |

---

## Hellenistic / Persian

| Function | Signature | Description |
|---|---|---|
| `egyptian_terms_ruler` | `(lon) → Body` | Egyptian bounds (Ptolemy/Tetrabiblos) |
| `decan_ruler` | `(lon) → Body` | Chaldean decan (face) ruler |
| `triplicity_rulers` | `(lon) → (Body, Body, Body)` | Day / night / participating |
| `full_dignity` | `(body, lon, is_day) → (Dignity, i32)` | Dignity name + score |
| `almuten` | `(lon, is_day) → (Body, i32)` | Highest-scoring planet |
| `is_day_chart` | `(sun_lon, cusps) → bool` | Sun above horizon |
| `same_sect` | `(body, is_day) → bool` | Sect membership |
| `firdaria` | `(jd, is_day, span) → Vec<FirdariaPeriod>` | 75-year period list |
| `annual_profection` | `(cusps, age) → (u8, f64)` | House number + lon |
| `monthly_profection` | `(cusps, age_years, months) → (u8, f64)` | Sub-annual |

**`Dignity` variants:** `Domicile` · `Exaltation` · `Triplicity` · `Term` · `Face` · `Peregrine` · `Detriment` · `Fall`

**`FirdariaPeriod` fields:** `major_lord` · `minor_lord` · `start_jd` · `end_jd` · `years`

---

## Chinese astrology

| Function | Signature | Description |
|---|---|---|
| `four_pillars` | `(jd, hour_ut, sun_lon) → [BaZiPillar; 4]` | Year/Month/Day/Hour pillars |
| `solar_term_position` | `(sun_lon) → (usize, f64, usize, f64)` | Current/next solar term |
| `sexagenary_name` | `(idx) → (String, String)` | Stem + animal names |

**`BaZiPillar` fields:** `stem_name` · `branch_name` · `animal` · `stem_element` · `branch_element` · `yang`

**Constants:** `SOLAR_TERMS[24]` · `HEAVENLY_STEMS[10]` · `EARTHLY_BRANCHES[12]`

---

## Mesoamerican calendars

| Function | Signature | Description |
|---|---|---|
| `tonalpohualli` | `(jd) → (u8, u8, String, String)` | Aztec 260-day: trecena, sign, names |
| `xiuhpohualli` | `(jd) → (u8, u8, String, String)` | Aztec 365-day: month, day, names |
| `tzolkin` | `(jd) → (u8, u8, String, String)` | Maya 260-day: trecena, sign, names |
| `haab` | `(jd) → (u8, u8, String)` | Maya 365-day: month, day, name |
| `calendar_round` | `(jd) → (u8, u8, u8, u8)` | Tzolkin + Haab combined position |

**Constants:** `GMT_CORRELATION = 584_283i64` · `TONALPOHUALLI_SIGNS[20]` · `TZOLKIN_SIGNS[20]` · `XIUHPOHUALLI_MONTHS[18]`

---

## Indigenous / Egyptian

| Function | Signature | Description |
|---|---|---|
| `medicine_wheel_totem` | `(sun_lon) → (String, String, String, String)` | Animal, element, clan, season |
| `egyptian_decan` | `(lon) → (u8, String, String)` | Index 0–35, decan name, rising star |

---

## Chart render types

All types are passed via `celestial render --type <n>`:

| Type | Tradition | Phase |
|---|---|---|
| `natal` | Western | 1 |
| `cosmogram` | Western | 2 |
| `solar-return` | Western | 2 |
| `lunar-return` | Western | 2 |
| `progressed` | Western | 2 |
| `solar-arc` | Western | 2 |
| `biwheel` | Western | 2 |
| `composite` | Western | 3 |
| `triwheel` | Western | 3 |
| `dial` | Western | 3 |
| `ephemeris` | Western | 3 |
| `local-space` | Western | 3 |
| `rasi` | Vedic | 4 |
| `north-indian` | Vedic | 4 |
| `navamsa` | Vedic | 4 |
| `dasha` | Vedic | 4 |
| `ashtakavarga` | Vedic | 4 |
| `shadbala` | Vedic | 4 |
| `hellenistic` | Hellenistic | 5 |
| `firdaria` | Persian | 5 |
| `profection` | Hellenistic | 5 |
| `bazi` | Chinese | 6 |
| `mesoamerican` | Mesoamerican | 7 |
| `medicine-wheel` | Indigenous | 8 |

---

## Legacy / Compatibility Aliases

These thin wrappers exist for SwissEph API compatibility and are available
in Python, JavaScript, and PHP.

| Alias | Canonical | Notes |
|-------|-----------|-------|
| `degnorm(d)` | `norm_deg(d)` | Normalise degrees to [0°, 360°) |
| `difdeg2n(p1, p2)` | `diff_deg_signed(p1, p2)` | Signed diff in (−180°, +180°] |
| `get_ayanamsa(jd_et)` | `ayanamsa(jd_et)` | SwissEph `get_` prefix |
| `get_ayanamsa_name(mode)` | `ayanamsa_name(mode)` | SwissEph `get_` prefix |
| `next_sabbat_name(jd)` | `next_sabbat(jd)` | Returns name string only |
| `next_full_moon(jd)` | `next_full_moon_after(jd)` | Short name variant |

---

## `solcross_ut(x2cross, jd_ut, flags)` → `float`

Returns the Julian Day (UT) when the Sun next crosses ecliptic longitude
`x2cross` degrees. Use this to find equinoxes, solstices, or any solar
degree transit.

```python
# Python — vernal equinox (Sun crosses 0° Aries)
jd = julday(2025, 1, 1, 0.0, GREG_CAL)
equinox_jd = solcross_ut(0.0, jd, FLG_BUILTIN)
```

```php
// PHP
$equinox_jd = solcross_ut(0.0, $jd, FLG_BUILTIN);
```
