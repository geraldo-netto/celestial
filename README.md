# celestial

[![celestial-core](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-core.yml/badge.svg)](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-core.yml)
[![celestial-cli](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-cli.yml/badge.svg)](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-cli.yml)
[![celestial-python](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-python.yml/badge.svg)](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-python.yml)
[![celestial-js](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-js.yml/badge.svg)](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-js.yml)
[![celestial-php](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-php.yml/badge.svg)](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-php.yml)

A pure-Rust astronomical engine — no C compiler, no data files, **no external dependencies**.

Covers the Swiss Ephemeris API surface: planetary positions, house cusps, eclipses, crossings, rise/set times, aspects, retrogrades, Vedic helpers, the Celtic Wheel of the Year.

---

## Workspace layout

```
celestial-workspace/
├── core/          celestial-core    — pure-Rust engine, zero dependencies
│   └── src/functions/
│       ├── calc.rs, config.rs, houses.rs, eclipses.rs
│       ├── motion.rs  (crossings + rise/set/transit)
│       ├── time.rs    (calendar, JD, UTC, display helpers)
│       ├── utils.rs   (math, coordinates, refraction, phenomena)
│       ├── aspects.rs, searches.rs, vedic.rs
│       ├── sabbats.rs, esbats.rs, omer.rs
│       ├── jewish.rs, easter.rs, islamic.rs
│       ├── panchanga.rs, vesak.rs, nowruz.rs
│       ├── moon_phases.rs
│       └── geoformat.rs, timezone.rs
├── cli/           celestial-cli     — command-line tool  (`celestial` binary)
├── bindings/
│   ├── python/    celestial-py      — Python bindings via PyO3 / maturin
│   ├── js/        celestial-js      — Node.js bindings via napi-rs
│   └── php/       celestial-php     — PHP 8.x bindings via ext-php-rs
└── fuzz/          celestial-fuzz    — 32 property-test suites
```

The public API is a **single flat namespace** — `use celestial_core::*` gives you everything.

---

## License

Released under **AGPL-3.0**, matching the Swiss Ephemeris it emulates.

The helper functions (`aspects.rs`, `searches.rs`, `vedic.rs`, `geoformat.rs`, `timezone.rs`, `datetime.rs`) are a Rust port of [swephelp](https://github.com/astrorigin/swephelp) by Stanislas Marquis (© 2007-2020, GPL-2).

---

## CLI

Install the `celestial` binary:

```bash
cargo install --path cli
```

### Commands

```
celestial calc      Geocentric planetary positions
celestial houses    House cusps and special angles
celestial sabbats   Celtic Wheel of the Year (8 sabbats)
celestial esbats    Named full moons
celestial jd        Julian day ↔ calendar date
celestial crossing  Next ecliptic longitude crossing
celestial eclipse   Solar and lunar eclipses
celestial chart     Full astrological chart — JSON output + SVG wheel
celestial moon      Moon phase, illumination, and next principal phases
celestial omer      Sefirat HaOmer — 49-day Omer count
celestial calendar  Multi-tradition religious calendars
                   jewish | easter | islamic | panchanga | vesak | nowruz
```

### Examples

```bash
# All nine planets, table output
celestial calc --date 2025-03-20

# Sidereal (Lahiri), selected bodies, JSON
celestial calc --date 2025-03-20 --body sun,moon --sidereal --mode lahiri --json

# Placidus cusps for Paris
celestial houses --date 2025-03-20 --lat 48.85 --lon 2.35

# Koch cusps for London
celestial houses --date 2025-03-20 --lat 51.5 --lon -0.12 --system koch

# Wheel of the Year
celestial sabbats --year 2025
celestial sabbats --next

# Full moons
celestial esbats --year 2025 --json
celestial esbats --next

# Date ↔ JD
celestial jd 2025-03-20
celestial jd --from-jd 2460754.5

# Next vernal equinox (Sun at 0°)
celestial crossing --body sun --lon 0 --from 2025-01-01

# Eclipses
celestial eclipse --type solar --from 2025-01-01
celestial eclipse --type lunar

# Moon phases
celestial moon                          # current phase
celestial moon --date 2025-04-13        # phase at specific date
celestial moon --full                   # next full moon
celestial moon --month 2025-04          # all phases in April 2025
celestial moon --month 2025-04 --json

# Sefirat HaOmer
celestial omer                          # tonight's Omer day (if in Omer period)
celestial omer --date 2025-05-16        # Lag Ba'Omer
celestial omer --all --year 5785        # all 49 days
celestial omer --json

# Religious calendars
celestial calendar jewish --year 2025                # Jewish holidays
celestial calendar easter --year 2025                # Easter + all feasts with --all
celestial calendar islamic --year 2025               # Islamic observances
celestial calendar islamic --convert 2025-04-20      # Gregorian → Hijri
celestial calendar panchanga --date 2025-03-20       # Hindu Panchānga
celestial calendar panchanga --festivals --year 2025 # Hindu festivals
celestial calendar vesak --year 2025                 # Vesak (Buddha Day)
celestial calendar vesak --uposatha --year 2025      # all Uposatha days
celestial calendar nowruz --year 2025                # Nowruz + Persian calendar
celestial calendar nowruz --bahai --year 2025        # Bahá'í holy days
```

All subcommands accept `--json` for machine-readable output.

---

## Accuracy

| Body | Longitude | Latitude | Distance |
|---|---|---|---|
| Sun | ~1″ | ~0.1″ | ~0.00001 AU |
| Moon | ~10″ | ~4″ | ~4 km |
| Inner planets | ~1″–30″ | ~1″–10″ | ~0.001 AU |
| Outer planets | ~1″–60″ | ~1″–30″ | ~0.01 AU |

Accuracy degrades beyond ±3000 years from J2000.

---

## Rust

### Installation

```toml
[dependencies]
celestial-core = { path = "core" }
```

### Planetary positions

```rust
use celestial_core::*;

fn main() -> Result<()> {
    let jd = julday(2025, 3, 20, 9.0, GREG_CAL);

    // Sun — geocentric (Universal Time)
    let sun = calc_ut(jd, SUN, FLG_BUILTIN | FLG_SPEED)?;
    println!("Sun  lon={:.4}°  dist={:.6} AU  speed={:.4}°/d",
        sun.lon, sun.dist, sun.speed_lon);

    // All main planets
    for body in [SUN, MOON, MERCURY, VENUS, MARS, JUPITER, SATURN, URANUS, NEPTUNE] {
        let p = calc_ut(jd, body, FLG_BUILTIN)?;
        println!("  {:<10}  {:>10.4}°", planet_name(body), p.lon);
    }

    // Placidus house cusps — Paris
    let h = houses(jd, 48.85, 2.35, b'P')?;
    println!("ASC={:.2}°  MC={:.2}°", h.ascmc[0], h.ascmc[1]);

    // Sidereal (Lahiri) position of Mars
    set_sid_mode(SIDM_LAHIRI, 0.0, 0.0);
    let mars = calc_ut(jd, MARS, FLG_BUILTIN | FLG_SIDEREAL)?;
    println!("Mars sidereal lon={:.4}°", mars.lon);

    Ok(())
}
```

### Error handling

```rust
use celestial_core::{calc_ut, SUN, FLG_BUILTIN, Error};

match calc_ut(jd, SUN, FLG_BUILTIN) {
    Ok(pos)               => println!("lon={:.4}", pos.lon),
    Err(Error::Calc(msg)) => eprintln!("calculation failed: {msg}"),
    Err(e)                => eprintln!("error: {e}"),
}
```

### Celtic sabbats & esbats

```rust
use celestial_core::*;

fn main() -> Result<()> {
    for s in sabbats_for_year(2025)? {
        let d = revjul(s.jd, GREG_CAL);
        println!("{:<14}  {:04}-{:02}-{:02}  {:.0}°",
            s.name, d.year, d.month, d.day, s.kind.solar_longitude());
    }

    let next = next_sabbat(julday(2025, 6, 1, 0.0, GREG_CAL))?;
    println!("Next: {}", next.name);

    for m in esbats_for_year(2025)? {
        let d = revjul(m.jd, GREG_CAL);
        println!("{:<18}  {:04}-{:02}-{:02}", m.display_name, d.year, d.month, d.day);
    }

    Ok(())
}
```

### Moon Phases

| Function | Description |
|---|---|
| `moon_phase(jd)` | Named phase → `MoonPhase` (8 variants) |
| `moon_illumination(jd)` | Fraction illuminated (0.0–1.0) |
| `moon_elongation(jd)` | Moon–Sun elongation (0°–360°) |
| `moon_phase_angle(jd)` | Alias for `moon_elongation` |
| `next_new_moon(jd)` | JD of next new moon (elongation = 0°) |
| `next_first_quarter(jd)` | JD of next first quarter (elongation = 90°) |
| `next_full_moon_phase(jd)` | JD of next full moon (elongation = 180°) |
| `next_last_quarter(jd)` | JD of next last quarter (elongation = 270°) |
| `moon_phases_for_month(year, month)` | All 4–5 `PhaseEvent`s in a calendar month |
| `moon_phase_info(jd)` | Rich `MoonPhaseInfo`: phase, illumination, age, prev/next |

**`MoonPhase` variants:** `NewMoon` · `WaxingCrescent` · `FirstQuarter` · `WaxingGibbous` · `FullMoon` · `WaningGibbous` · `LastQuarter` · `WaningCrescent`

**`MoonPhaseInfo` fields:** `phase` · `phase_name` · `elongation` · `illumination` · `prev_phase_jd` · `prev_phase_name` · `next_phase_jd` · `next_phase_name` · `age_days`

All phase timings use the same Newton + bisection engine as `next_full_moon`, accurate to within a few seconds. `SYNODIC_MONTH = 29.530_588_853` days is exported as a constant.


### Sefirat HaOmer

```rust
use celestial_core::*;

fn main() {
    // Is tonight an Omer night?
    let jd = jdnow();
    if let Some(day) = omer_from_jd(jd) {
        println!("Tonight is Omer day {} — {}", day.day, day.hebrew_text);
        println!("  {} sheb'{}'", day.week_sefirah, day.day_sefirah);
        if day.is_lag_baomer {
            println!("  🔥 Lag Ba'Omer!");
        }
    } else {
        println!("Not currently in the Omer period.");
    }

    // Full 49-day schedule for Hebrew year 5785
    let days = omer_days(5785);
    for d in &days {
        println!(
            "Day {:2}: {} sheb'{}{}",
            d.day,
            d.week_sefirah,
            d.day_sefirah,
            if d.is_lag_baomer { " (Lag Ba'Omer)" } else { "" }
        );
    }

    // JD of a specific day
    let lag_jd = omer_day_jd(5785, 33).unwrap();
    let d = revjul(lag_jd, GREG_CAL);
    println!("Lag Ba'Omer 5785: {:04}-{:02}-{:02}", d.year, d.month, d.day);

    // Full declaration
    println!("{}", omer_declaration(33));
    // → "Tish'a u'shloshim yom she'hem arba'a shavuot v'chamisha yamim la'Omer
    //    (Lag Ba'Omer) — Hod sheb'Hod"

    // Period for a given JD
    let period = omer_period(jdnow());
    println!("Hebrew year {} Omer: JD {:.0}–{:.0}",
        period.hebrew_year, period.start_jd, period.end_jd);
}
```


### Jewish Holidays

```rust
use celestial_core::*;

let holidays = jewish_holidays(5785); // Hebrew year 5785 = 2024/2025
for h in &holidays {
    let d = revjul(h.jd, GREG_CAL);
    println!("{:<30}  {:04}-{:02}-{:02}  ({} days)",
        h.name, d.year, d.month, d.day, h.days);
}

// Lookup a specific holiday
let yom_kippur = jewish_holiday_jd(5785, "Yom Kippur").unwrap();

// Convert any JD to Hebrew date
let (year, month, day) = jd_to_hebrew_date(jdnow());
println!("Today in Hebrew: {} {}/{}", year, month, day);
```

### Easter & Christian Calendar

```rust
use celestial_core::*;

// Western Easter
let (y, m, d) = easter_gregorian(2025);
assert_eq!((y, m, d), (2025, 4, 20));

// Orthodox Easter
let (y, m, d) = easter_orthodox(2025);

// All moveable feasts (Ash Wednesday → Corpus Christi)
for feast in christian_feasts(2025) {
    println!("{:<30}  {:02}/{:02}  (Easter {:+} days)",
        feast.name, feast.month, feast.day, feast.easter_offset);
}

// Fixed feasts (Christmas, Epiphany, etc.)
for feast in christian_fixed_feasts(2025) {
    println!("{:<30}  {:02}/{:02}", feast.name, feast.month, feast.day);
}
```

### Islamic (Hijri) Calendar

```rust
use celestial_core::*;

// Convert today to Hijri
let (year, month, day) = hijri_from_jd(jdnow());
println!("Hijri: {} {} {}", day, hijri_month_name(month), year);

// All major observances for Hijri year 1446
for obs in islamic_observances(1446) {
    let d = revjul(obs.jd, GREG_CAL);
    println!("{:<35}  {:04}-{:02}-{:02}", obs.name, d.year, d.month, d.day);
}

// Which Hijri years overlap Gregorian 2025?
let (y1, y2) = gregorian_to_hijri_years(2025);
// → (1446, 1447)
```

### Hindu Panchānga

```rust
use celestial_core::*;

let jd = julday(2025, 3, 20, 6.0, GREG_CAL);
let p = panchanga(jd);

println!("Tithi:     {} ({:?})", p.tithi_name, p.paksha);
println!("Vara:      {}", p.vara_name);
println!("Nakshatra: {} pada {}", p.nakshatra_name, p.nakshatra_pada);
println!("Yoga:      {}", p.yoga_name);
println!("Karana:    {}", p.karana_name);

// Major Hindu festivals in a Gregorian year
for f in hindu_festivals(2025) {
    let d = revjul(f.jd, GREG_CAL);
    println!("{:<35}  {:04}-{:02}-{:02}", f.name, d.year, d.month, d.day);
}
```

### Buddhist Observances

```rust
use celestial_core::*;

// Vesak (Buddha Day) — full moon in Vaisakha
let vesak = vesak_jd(2025);
let d = revjul(vesak, GREG_CAL);
println!("Vesak 2025: {:04}-{:02}-{:02}", d.year, d.month, d.day);

// All four Uposatha phases for the year
let days = uposatha_days(2025);
for u in days.iter().filter(|u| u.phase == UposathaPhase::FullMoon).take(3) {
    let d = revjul(u.jd, GREG_CAL);
    println!("Full Moon Uposatha: {:04}-{:02}-{:02}", d.year, d.month, d.day);
}
```

### Nowruz, Persian Calendar & Bahá'í Calendar

```rust
use celestial_core::*;

// Nowruz = exact vernal equinox
let jd = nowruz_jd(2025);
let d = revjul(jd, GREG_CAL);
println!("Nowruz 2025: {:04}-{:02}-{:02} {:04.1}h UT",
    d.year, d.month, d.day, d.hour);

// Solar Hijri (Persian) year
println!("Solar Hijri: {}", gregorian_to_solar_hijri(2025)); // → 1404

// Bahá'í date
let bd = jd_to_bahai(jdnow());
println!("Bahá'í: {} ({}) {}, year {}",
    bd.day, bd.month_name, bd.year, bd.year);

// Bahá'í holy days
for h in bahai_holy_days(182) { // 182 BE = 2025/2026 CE
    let d = revjul(h.jd, GREG_CAL);
    println!("{:<35}  {:04}-{:02}-{:02}", h.name, d.year, d.month, d.day);
}
```


### Crossings, rise/set & eclipses

```rust
use celestial_core::*;

fn main() -> Result<()> {
    let jd = julday(2025, 1, 1, 0.0, GREG_CAL);

    // Next vernal equinox
    let ostara = solcross_ut(0.0, jd, FLG_BUILTIN)?;
    let d = revjul(ostara, GREG_CAL);
    println!("Ostara {:04}-{:02}-{:02}", d.year, d.month, d.day);

    // Next solar eclipse
    let ecl = sol_eclipse_when_glob(jd, FLG_BUILTIN, 0, false)?;
    let d = revjul(ecl.tret[0], GREG_CAL);
    println!("Solar eclipse {:04}-{:02}-{:02}", d.year, d.month, d.day);

    // Moon rise in London
    let rise = rise_trans(jd, MOON, None, FLG_BUILTIN, CALC_RISE,
        [-0.12, 51.5, 10.0], 0.0, 0.0)?;
    println!("Moon rises JD {:.4}", rise.tret);  // tret is f64 for rise_trans

    Ok(())
}
```

### Complete chart calculations

```rust
use celestial_core::*;

fn main() -> Result<()> {
    let jd   = julday(1985, 7, 14, 12.0, GREG_CAL);
    let lat  = 48.85;
    let lon  = 2.35;

    // ── Positions & aspects ───────────────────────────────────────
    let bodies = [SUN, MOON, MERCURY, VENUS, MARS, JUPITER, SATURN];
    let positions: Vec<(i32, f64, f64)> = bodies.iter().map(|&b| {
        let p = calc_ut(jd, b, FLG_BUILTIN | FLG_SPEED).unwrap();
        (b, p.lon, p.speed_lon)
    }).collect();

    let aspects = calc_chart_aspects(&positions, MAJOR_ASPECTS, 8.0);
    for a in &aspects {
        println!("{} {:.0}° {} orb={:.2}°",
            planet_name(a.body1), a.aspect, planet_name(a.body2), a.orb);
    }

    // ── Sign ingress & retrograde stations ────────────────────────
    let (jd_ingress, sign) = sign_ingress_ut(SATURN, jd, FLG_BUILTIN, false)?;
    println!("Saturn enters {}: JD {jd_ingress:.2}", zodiac_sign_name(sign));

    let stations = retrograde_station_ut(MARS, jd, FLG_BUILTIN)?;
    println!("Mars retrograde: JD {:.2}", stations.retrograde);
    println!("Mars direct:     JD {:.2}", stations.direct);

    // ── Transits to natal chart ───────────────────────────────────
    let jd_now = jdnow();
    let jd_sr  = solar_return_jd(jd, 2025, FLG_BUILTIN)?;
    let d = revjul(jd_sr, GREG_CAL);
    println!("Solar return 2025: {:04}-{:02}-{:02}", d.year, d.month, d.day);

    let jd_sat_mc = mc_transit_ut(SATURN, jd, jd_now, lat, lon, b'P', FLG_BUILTIN, false)?;
    println!("Saturn conjunct natal MC: JD {jd_sat_mc:.2}");

    // ── Arabic Parts ──────────────────────────────────────────────
    let chart = houses(jd, lat, lon, b'P')?;
    let sun  = calc_ut(jd, SUN,  FLG_BUILTIN)?;
    let moon = calc_ut(jd, MOON, FLG_BUILTIN)?;
    let fortune = arabic_part(chart.ascmc[0], moon.lon, sun.lon);
    let (sign, deg) = lon_to_sign(fortune);
    println!("Lot of Fortune: {:.2}° ({} {:.2}°)", fortune, zodiac_sign_name(sign), deg);

    // ── Vedic Vimshottari dasha ───────────────────────────────────
    set_sid_mode(SIDM_LAHIRI, 0.0, 0.0);
    let moon_sid = calc_ut(jd, MOON, FLG_BUILTIN | FLG_SIDEREAL)?;
    let dashas = vimshottari_dasha(jd, moon_sid.lon, 120.0);
    for d in dashas.iter().take(3) {
        println!("{} dasha: {:.1} years", planet_name(d.planet), d.years);
    }

    // ── Progressions & midpoints ──────────────────────────────────
    let (prog_pos, _) = secondary_progressions(jd, 35.0, &bodies, lat, lon, b'P', FLG_BUILTIN)?;
    let prog_sun  = prog_pos[0].1.lon; // progressed Sun longitude
    let prog_moon = prog_pos[1].1.lon; // progressed Moon longitude
    let mid = midpoint(prog_sun, prog_moon);
    println!("Progressed Sun-Moon midpoint: {mid:.2}°");

    Ok(())
}
```

### Chart command

Computes a full astrological chart for a given date and location. JSON is written
to stdout (pipe-friendly); the text table goes to stderr; an SVG wheel is written
to a file with `--svg`.

```bash
# Full chart for a birth date in Paris
celestial chart \
  --date "1985-07-14 14:30" \
  --lat 48.8566 \
  --lon 2.3522 \
  --name "Bastille Day 1985" \
  --svg chart.svg

# Pipe JSON to jq
celestial chart --date "2000-01-01 12:00" --lat 40.71 --lon -74.01 2>/dev/null \
  | jq '.angles.asc'

# Different house system
celestial chart --date now --lat 51.5 --lon -0.1 --system whole
```

The JSON output includes:

```json
{
  "date": "2000-01-01 12:00 UT",
  "angles": { "asc": { "lon": 206.78, "sign": "Libra", "deg": 26.78 }, ... },
  "planets": [ { "body": "Sun", "sign": "Capricorn", "sign_deg": "10.38", ... } ],
  "houses":  [ { "house": 1, "cusp": 206.78, "sign": "Libra" }, ... ],
  "aspects": [ { "body1": "Sun", "body2": "Chiron", "aspect": "Conjunction", "orb": "0.03" } ]
}
```


---

## Python

### Installation

```bash
cd bindings/python && pip install maturin && maturin develop
```

### Usage

```python
import celestial_py as celestial

jd = celestial.julday(2025, 3, 20, 9.0, celestial.GREG_CAL)

# calc_ut returns ((lon, lat, dist, speed_lon, speed_lat, speed_dist), ret_flags)
(lon, lat, dist, speed_lon, *_), _ = celestial.calc_ut(jd, celestial.SUN, celestial.FLG_BUILTIN | celestial.FLG_SPEED)
print(f"Sun  lon={lon:.4f}°  dist={dist:.6f} AU")

# House cusps
cusps, ascmc = celestial.houses(jd, 48.85, 2.35, ord("P"))
print(f"ASC={ascmc[0]:.2f}°  MC={ascmc[1]:.2f}°")

# Sidereal
celestial.set_sid_mode(celestial.SIDM_LAHIRI)
(moon_lon, *_), _ = celestial.calc_ut(jd, celestial.MOON, celestial.FLG_BUILTIN | celestial.FLG_SIDEREAL)
print(f"Moon (Lahiri) = {moon_lon:.4f}°")

# Celtic calendar
for s in celestial.sabbats_for_year(2025):
    y, mo, d, _ = celestial.revjul(s["jd"], celestial.GREG_CAL)
    print(f"{s['name']:<14}  {y}-{mo:02d}-{d:02d}")

# Vedic
celestial.set_sid_mode(celestial.SIDM_LAHIRI)
(moon_lon, *_), _ = celestial.calc_ut(jd, celestial.MOON, celestial.FLG_BUILTIN | celestial.FLG_SIDEREAL)
nak, pada = celestial.long_to_nakshatra(moon_lon)
print(f"Nakshatra: {celestial.nakshatra_name(nak)}, pada {pada}")
```

### Moon Phases

| Function | Description |
|---|---|
| `moon_phase(jd)` | Named phase → `MoonPhase` (8 variants) |
| `moon_illumination(jd)` | Fraction illuminated (0.0–1.0) |
| `moon_elongation(jd)` | Moon–Sun elongation (0°–360°) |
| `moon_phase_angle(jd)` | Alias for `moon_elongation` |
| `next_new_moon(jd)` | JD of next new moon (elongation = 0°) |
| `next_first_quarter(jd)` | JD of next first quarter (elongation = 90°) |
| `next_full_moon_phase(jd)` | JD of next full moon (elongation = 180°) |
| `next_last_quarter(jd)` | JD of next last quarter (elongation = 270°) |
| `moon_phases_for_month(year, month)` | All 4–5 `PhaseEvent`s in a calendar month |
| `moon_phase_info(jd)` | Rich `MoonPhaseInfo`: phase, illumination, age, prev/next |

**`MoonPhase` variants:** `NewMoon` · `WaxingCrescent` · `FirstQuarter` · `WaxingGibbous` · `FullMoon` · `WaningGibbous` · `LastQuarter` · `WaningCrescent`

**`MoonPhaseInfo` fields:** `phase` · `phase_name` · `elongation` · `illumination` · `prev_phase_jd` · `prev_phase_name` · `next_phase_jd` · `next_phase_name` · `age_days`

All phase timings use the same Newton + bisection engine as `next_full_moon`, accurate to within a few seconds. `SYNODIC_MONTH = 29.530_588_853` days is exported as a constant.



---

## JavaScript / TypeScript

### Installation

```bash
cd bindings/js && npm install && npm run build
```

### Usage

```typescript
import * as celestial from "./index";

const jd  = celestial.julday(2025, 3, 20, 9.0, celestial.GREG_CAL);
// PlanetPos fields are camelCase (napi-rs convention)
const sun = celestial.calcUt(jd, celestial.SUN, celestial.FLG_BUILTIN | celestial.FLG_SPEED);
console.log(`Sun  lon=${sun.lon.toFixed(4)}°  dist=${sun.dist.toFixed(6)} AU`);

const h = celestial.houses(jd, 48.85, 2.35, "P".charCodeAt(0));
console.log(`ASC=${h.ascmc[0].toFixed(2)}°  MC=${h.ascmc[1].toFixed(2)}°`);

celestial.setSidMode(celestial.SIDM_LAHIRI, 0, 0);
const moon = celestial.calcUt(jd, celestial.MOON, celestial.FLG_BUILTIN | celestial.FLG_SIDEREAL);
console.log(`Moon (Lahiri) = ${moon.lon.toFixed(4)}°`);

// Celtic calendar
const wheel = celestial.sabbatsForYear(2025);
for (const s of wheel) {
  const d = celestial.revjul(s.jd, celestial.GREG_CAL);
  console.log(`${s.name.padEnd(14)} ${d.year}-${String(d.month).padStart(2,"0")}-${String(d.day).padStart(2,"0")}`);
}
```

### Moon Phases

| Function | Description |
|---|---|
| `moon_phase(jd)` | Named phase → `MoonPhase` (8 variants) |
| `moon_illumination(jd)` | Fraction illuminated (0.0–1.0) |
| `moon_elongation(jd)` | Moon–Sun elongation (0°–360°) |
| `moon_phase_angle(jd)` | Alias for `moon_elongation` |
| `next_new_moon(jd)` | JD of next new moon (elongation = 0°) |
| `next_first_quarter(jd)` | JD of next first quarter (elongation = 90°) |
| `next_full_moon_phase(jd)` | JD of next full moon (elongation = 180°) |
| `next_last_quarter(jd)` | JD of next last quarter (elongation = 270°) |
| `moon_phases_for_month(year, month)` | All 4–5 `PhaseEvent`s in a calendar month |
| `moon_phase_info(jd)` | Rich `MoonPhaseInfo`: phase, illumination, age, prev/next |

**`MoonPhase` variants:** `NewMoon` · `WaxingCrescent` · `FirstQuarter` · `WaxingGibbous` · `FullMoon` · `WaningGibbous` · `LastQuarter` · `WaningCrescent`

**`MoonPhaseInfo` fields:** `phase` · `phase_name` · `elongation` · `illumination` · `prev_phase_jd` · `prev_phase_name` · `next_phase_jd` · `next_phase_name` · `age_days`

All phase timings use the same Newton + bisection engine as `next_full_moon`, accurate to within a few seconds. `SYNODIC_MONTH = 29.530_588_853` days is exported as a constant.



---

## PHP

Supports PHP 8.1 via [`ext-php-rs`](https://github.com/davidcole1340/ext-php-rs).

### Installation

```bash
sudo apt-get install php-dev   # Ubuntu/Debian
sudo dnf install php-devel     # Fedora/RHEL
brew install php               # macOS

cd bindings/php && cargo build --release
# Add to php.ini:  extension=/path/to/libcelestial.so
```

### Usage

```php
<?php
$jd  = julday(2025, 3, 20, 9.0, GREG_CAL);
$sun = calc_ut($jd, SUN, FLG_BUILTIN | FLG_SPEED);
printf("Sun  lon=%.4f°  dist=%.6f AU\n", $sun[0], $sun[2]);

$h = houses($jd, 48.85, 2.35, ord('P'));
printf("ASC=%.2f°  MC=%.2f°\n", $h['ascmc'][0], $h['ascmc'][1]);

set_sid_mode(SIDM_LAHIRI, 0.0, 0.0);
$moon = calc_ut($jd, MOON, FLG_BUILTIN | FLG_SIDEREAL);
printf("Moon (Lahiri) = %.4f°\n", $moon[0]);

// Celtic calendar
foreach (sabbats_for_year(2025) as $s) {
    $name = next_sabbat_name($s['jd'] - 0.001);
    $d    = revjul($s['jd'], GREG_CAL);
    printf("%-14s  %04d-%02d-%02d\n", $name, $d['year'], $d['month'], $d['day']);
}
```

### Moon Phases

| Function | Description |
|---|---|
| `moon_phase(jd)` | Named phase → `MoonPhase` (8 variants) |
| `moon_illumination(jd)` | Fraction illuminated (0.0–1.0) |
| `moon_elongation(jd)` | Moon–Sun elongation (0°–360°) |
| `moon_phase_angle(jd)` | Alias for `moon_elongation` |
| `next_new_moon(jd)` | JD of next new moon (elongation = 0°) |
| `next_first_quarter(jd)` | JD of next first quarter (elongation = 90°) |
| `next_full_moon_phase(jd)` | JD of next full moon (elongation = 180°) |
| `next_last_quarter(jd)` | JD of next last quarter (elongation = 270°) |
| `moon_phases_for_month(year, month)` | All 4–5 `PhaseEvent`s in a calendar month |
| `moon_phase_info(jd)` | Rich `MoonPhaseInfo`: phase, illumination, age, prev/next |

**`MoonPhase` variants:** `NewMoon` · `WaxingCrescent` · `FirstQuarter` · `WaxingGibbous` · `FullMoon` · `WaningGibbous` · `LastQuarter` · `WaningCrescent`

**`MoonPhaseInfo` fields:** `phase` · `phase_name` · `elongation` · `illumination` · `prev_phase_jd` · `prev_phase_name` · `next_phase_jd` · `next_phase_name` · `age_days`

All phase timings use the same Newton + bisection engine as `next_full_moon`, accurate to within a few seconds. `SYNODIC_MONTH = 29.530_588_853` days is exported as a constant.



---


### Parallel multi-body calculation — `calc_many` / `calc_ut_many`

Compute positions for multiple bodies concurrently. Useful for full chart
calculations (12 bodies) where the speedup scales with CPU cores.

```python
# Python — compute all 12 chart bodies at once
import celestial_py as c
planets = [c.SUN, c.MOON, c.MERCURY, c.VENUS, c.MARS, c.JUPITER,
           c.SATURN, c.URANUS, c.NEPTUNE, c.PLUTO, c.MEAN_NODE, c.CHIRON]
results = c.calc_many(2451545.0, planets, c.FLG_BUILTIN | c.FLG_SPEED)
# results[i] == c.calc(jd, planets[i], flags) for all i
```

```javascript
// JavaScript
const { calcMany, SUN, MOON, MERCURY, FLG_BUILTIN } = require('./index');
const results = calcMany(2451545.0, [SUN, MOON, MERCURY], FLG_BUILTIN);
```

```php
// PHP
$results = calc_many(2451545.0, [SUN, MOON, MERCURY], FLG_BUILTIN);
```

### Nutation and obliquity

```python
# IAU 2000B nutation (77 terms, ~1 mas accuracy)
dpsi_deg, deps_deg = c.nutation(2451545.0)    # degrees
dpsi_arcsec = dpsi_deg * 3600                  # → arcseconds

eps_mean = c.mean_obliquity(2451545.0)         # IAU 2006 formula
eps_true = c.true_obliquity(2451545.0)         # mean + Δε
```


### Phase 1 — wheel enhancements

The `render` command now includes the following additional layers on the natal chart:

#### Minor aspects
Seven new aspects rendered as dashed lines at reduced opacity:

| Aspect | Angle | Orb |
|---|---|---|
| Semi-sextile | 30° | 2° |
| Semi-square | 45° | 2° |
| Quintile | 72° | 1.5° |
| Sesquiquadrate | 135° | 2° |
| Biquintile | 144° | 1.5° |
| Septile | 51.43° | 1° |
| Novile | 40° | 1° |

Major aspects are solid lines; minor aspects are dashed and lighter.

#### Station markers
Planets with `|speed| < 0.05°/day` receive a small `S` badge — they are
stationary or very near a retrograde/direct station.

#### Antiscia and contra-antiscia
Each planet's antiscion (mirror over the 0°Cancer–0°Capricorn solstice axis)
is shown as a faint glyph at the house ring.

```
antiscion(lon) = (180° − lon) mod 360°
contra_antiscion(lon) = (360° − lon) mod 360°
```

#### Arabic Parts / Lots
All seven traditional Arabic Parts are computed and placed on the wheel
(initials: Fo=Fortune, Sp=Spirit, Lo=Love, Ne=Necessity, Co=Courage,
Vi=Victory, Nm=Nemesis). Day/night reversal of Fortune and Spirit is applied
automatically based on whether the Sun is above the horizon.

A dedicated "Arabic Parts" legend section lists all seven with their degree and sign.

#### Fixed stars
The 15 most astrologically significant fixed stars (Algol, Aldebaran, Sirius,
Regulus, Spica, Antares, etc.) are plotted as dots on the sign band with
magnitude-proportional sizing.

#### Essential dignities table
A new legend section shows each planet's essential dignity:

| Status | Description |
|---|---|
| **domicile** | Planet rules the sign it occupies |
| **exaltation** | Planet is in its exaltation sign |
| **detriment** | Planet is in the sign opposite its domicile |
| **fall** | Planet is in the sign opposite its exaltation |
| peregrine | None of the above |


### Plugin architecture

Any executable named `celestial-<name>` on `$PATH` becomes a first-class subcommand:

```bash
celestial --list-plugins           # discover all installed plugins
celestial synastry --date 1985-01-01 # runs celestial-synastry if on PATH
```

### `render` — template-driven charts


### Example output

![Celestial Chart — Paris J2000.0](docs/example_chart.svg)

*Built-in SVG chart for Paris, 2000-01-01. Wheel shows zodiac sectors, house cusps,
planet glyphs with degree labels, aspect lines (blue = soft, red = hard), and a
three-column legend: Planets · Angles & Houses · Aspects.*

```bash
# Built-in SVG chart (dark theme, full legend):
celestial render --date 2025-03-20 --lat 48.85 --lon 2.35 --out chart.svg

# Override palette and title:
celestial render --date now --lat 48.85 --lon 2.35 \
  --var "title=My Chart" --var "bg_color=#1a1a2e" --out chart.svg

# TOML config file:
celestial render --config chart.toml --out chart.svg

# Bootstrap a custom template:
celestial render --print-template > my_chart.tmpl

# Inspect all available template variables as JSON:
celestial render --date 2025-03-20 --print-context
```

`chart.toml` example:
```toml
[render]
date  = "2025-03-20"
lat   = 48.8566
lon   = 2.3522
hsys  = "P"

[vars]
title        = "Spring Equinox 2025"
bg_color     = "#0d1117"
ring_color   = "#58a6ff"
my_custom_var = "hello"
```

Templates use [TinyTemplate](https://github.com/bheisler/TinyTemplate) syntax.
All wheel geometry (planet x/y, cusp lines, aspect endpoints) is **pre-computed**
in Rust — templates need no math, just `{planet.x}`, `{planet.y}`, etc.

| Variable | Type | Description |
|---|---|---|
| `{date}` | string | ISO date |
| `{jd}` | float | Julian Day |
| `{asc}` / `{mc}` / `{ic}` / `{dsc}` | float | Angle longitudes |
| `{asc_dms}` … | string | DMS formatted angles |
| `{planets}` | list | 12 bodies with `.lon .lat .x .y .glyph .dms .retro` … |
| `{signs}` | list | 12 sign sectors with `.spoke_x1 .spoke_y1 .glyph_x .glyph_y` |
| `{houses}` | list | 12 cusps with `.x1 .y1 .x2 .y2 .num_x .num_y .dms` |
| `{aspects}` | list | Active aspects with `.x1 .y1 .x2 .y2 .orb .applying .is_hard` |
| `{moon_phase_name}` | string | Current lunar phase |
| `{moon_illumination}` | float | Illumination 0–100% |
| `{vars.key}` | string | Any `--var key=value` or `[vars] key = "value"` |


## Building from source

```bash
# Full workspace build
cargo build

# Tests — 549 unit/integration + 32 property-test suites
cargo test --package celestial-core -- --test-threads=1
cargo run  --manifest-path fuzz/Cargo.toml

# Install CLI
cargo install --path cli

# Python wheel
cd bindings/python && maturin build --release

# Node.js addon
cd bindings/js && npm install && npm run build

# PHP extension
cd bindings/php && cargo build --release
```


> **Upgrading from an older checkout?** Delete these files if they still exist locally:
> ```bash
> rm core/src/functions/crossings.rs   # merged into motion.rs
> rm core/src/functions/rise_trans.rs  # merged into motion.rs
> rm core/src/functions/datetime.rs    # merged into time.rs
> ```

---

## Benchmarks

```bash
# Rust micro-benchmarks (44 functions, std::time harness)
cargo bench --package celestial-core

# Precision + performance comparison vs pyephem and astropy
pip install astropy pyephem
python3 benches/precision_comparison.py
```

Precision vs Meeus *Astronomical Algorithms* 2nd ed. using `calc()` (TT input):

| Body | Error | Model |
|---|---|---|
| Sun 1992-Oct-13 | 3.2″ | VSOP87 + IAU 2000B nutation |
| Moon 1992-Apr-12 | 0.7″ | ELP2000-82 + IAU 2000B nutation |
| julday J2000 | exact | — |

Nutation: **IAU 2000B** luni-solar series (77 terms, ~1 mas = 0.001″ accuracy).
Replaces the former IAU 1980 model (64 terms, ~500 mas). Obliquity uses the
**IAU 2006** formula (Capitaine et al. 2003).


## API reference

### Error types

```rust
pub enum Error {
    Calc(String),      // planetary calculation failure
    Houses(String),    // house system failure
    Eclipse(String),   // eclipse search failure
    RiseTrans(String), // rise/transit failure
    Date(String),      // date conversion failure
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Time & calendar

| Function | Description |
|---|---|
| `julday(y, m, d, h, cal)` | Calendar date → Julian day number |
| `revjul(jd, cal)` | Julian day → `CalDate` |
| `utc_to_jd(date, cal)` | UTC → `JdPair` (jd_et, jd_ut) |
| `deltat(jd)` | ΔT = TT − UT1 in days |
| `sidtime(jd_ut)` | Greenwich Apparent Sidereal Time (hours) |
| `day_of_week(jd)` | 0 = Sunday … 6 = Saturday |
| `jdnow()` | Current Julian day (UTC) |
| `jd_to_iso_string(jd, cal)` | Format JD as `YYYY-MM-DD HH:MM:SS UTC` |

### Planetary positions

| Function | Description |
|---|---|
| `calc_ut(jd, body, flags)` | Geocentric position (UT) → `PlanetPos` |
| `calc(jd, body, flags)` | Geocentric position (TT/ET) → `PlanetPos` |
| `calc_pctr(jd, body, center, flags)` | Position relative to center body |
| `fixstar(name, jd, flags)` | Fixed star position → `FixStarPos` |
| `fixstar_mag(name)` | Fixed star visual magnitude |
| `nod_aps(jd, body, flags, method)` | Nodes and apsides → `NodAps` |
| `get_orbital_elements(jd, body, flags)` | Osculating orbital elements |

**`PlanetPos` fields:** `lon` `lat` `dist` `speed_lon` `speed_lat` `speed_dist` `ret_flags`

**Body constants:** `SUN=0` `MOON=1` `MERCURY=2` `VENUS=3` `MARS=4` `JUPITER=5` `SATURN=6` `URANUS=7` `NEPTUNE=8` `PLUTO=9` `MEAN_NODE=10` `TRUE_NODE=11` `CHIRON=15`

**Flag constants:** `FLG_BUILTIN` `FLG_SPEED` `FLG_SIDEREAL` `FLG_EQUATORIAL` `FLG_HELCTR` `FLG_TOPOCTR` `FLG_NONUT` `FLG_RADIANS` `FLG_XYZ`

### Configuration

| Function | Description |
|---|---|
| `planet_name(body)` | Body number → display name |
| `set_sid_mode(mode, t0, ayan_t0)` | Activate a sidereal mode |
| `set_topo(lon, lat, alt_m)` | Set topocentric observer |
| `set_ephe_path(path)` | Ephemeris data directory |
| `ayanamsa(jd_et)` | Ayanamsa for the active mode (TT) |
| `ayanamsa_ut(jd_ut)` | Ayanamsa for the active mode (UT) |
| `ayanamsa_name(mode)` | Name of a sidereal mode constant |

**Sidereal modes:** `SIDM_FAGAN_BRADLEY=0` · `SIDM_LAHIRI=1` · `SIDM_DELUCE=2` · `SIDM_RAMAN=3` · `SIDM_KRISHNAMURTI=5` · `SIDM_SASSANIAN=11` · `SIDM_USER=255`

### Houses

| Function | Description |
|---|---|
| `houses(jd, lat, lon, sys)` | Cusps + angles → `HouseResult` |
| `houses_ex(jd, flags, lat, lon, sys)` | With sidereal / topocentric flags |
| `house_pos(armc, lat, eps, sys, pos)` | House number for a body |
| `house_name(sys)` | System byte → name |

**`HouseResult`:** `cusps[12]` (12 house cusps, index 0 skipped), `ascmc[8]` (0=ASC, 1=MC, 2=ARMC, 3=Vertex, …7=Polar Asc)

**Systems:** `b'P'` Placidus · `b'K'` Koch · `b'E'` Equal · `b'W'` Whole-Sign · `b'O'` Porphyry · `b'R'` Regiomontanus · `b'C'` Campanus · `b'M'` Morinus · `b'B'` Alcabitus · `b'X'` Axial Rotation

### Moon Phases

| Function | Description |
|---|---|
| `moon_phase(jd)` | Named phase → `MoonPhase` (8 variants) |
| `moon_illumination(jd)` | Fraction illuminated (0.0–1.0) |
| `moon_elongation(jd)` | Moon–Sun elongation (0°–360°) |
| `moon_phase_angle(jd)` | Alias for `moon_elongation` |
| `next_new_moon(jd)` | JD of next new moon (elongation = 0°) |
| `next_first_quarter(jd)` | JD of next first quarter (elongation = 90°) |
| `next_full_moon_phase(jd)` | JD of next full moon (elongation = 180°) |
| `next_last_quarter(jd)` | JD of next last quarter (elongation = 270°) |
| `moon_phases_for_month(year, month)` | All 4–5 `PhaseEvent`s in a calendar month |
| `moon_phase_info(jd)` | Rich `MoonPhaseInfo`: phase, illumination, age, prev/next |

**`MoonPhase` variants:** `NewMoon` · `WaxingCrescent` · `FirstQuarter` · `WaxingGibbous` · `FullMoon` · `WaningGibbous` · `LastQuarter` · `WaningCrescent`

**`MoonPhaseInfo` fields:** `phase` · `phase_name` · `elongation` · `illumination` · `prev_phase_jd` · `prev_phase_name` · `next_phase_jd` · `next_phase_name` · `age_days`

All phase timings use the same Newton + bisection engine as `next_full_moon`, accurate to within a few seconds. `SYNODIC_MONTH = 29.530_588_853` days is exported as a constant.


### Sefirat HaOmer

```rust
use celestial_core::*;

fn main() {
    // Is tonight an Omer night?
    let jd = jdnow();
    if let Some(day) = omer_from_jd(jd) {
        println!("Tonight is Omer day {} — {}", day.day, day.hebrew_text);
        println!("  {} sheb'{}'", day.week_sefirah, day.day_sefirah);
        if day.is_lag_baomer {
            println!("  🔥 Lag Ba'Omer!");
        }
    } else {
        println!("Not currently in the Omer period.");
    }

    // Full 49-day schedule for Hebrew year 5785
    let days = omer_days(5785);
    for d in &days {
        println!(
            "Day {:2}: {} sheb'{}{}",
            d.day,
            d.week_sefirah,
            d.day_sefirah,
            if d.is_lag_baomer { " (Lag Ba'Omer)" } else { "" }
        );
    }

    // JD of a specific day
    let lag_jd = omer_day_jd(5785, 33).unwrap();
    let d = revjul(lag_jd, GREG_CAL);
    println!("Lag Ba'Omer 5785: {:04}-{:02}-{:02}", d.year, d.month, d.day);

    // Full declaration
    println!("{}", omer_declaration(33));
    // → "Tish'a u'shloshim yom she'hem arba'a shavuot v'chamisha yamim la'Omer
    //    (Lag Ba'Omer) — Hod sheb'Hod"

    // Period for a given JD
    let period = omer_period(jdnow());
    println!("Hebrew year {} Omer: JD {:.0}–{:.0}",
        period.hebrew_year, period.start_jd, period.end_jd);
}
```


### Jewish Holidays

```rust
use celestial_core::*;

let holidays = jewish_holidays(5785); // Hebrew year 5785 = 2024/2025
for h in &holidays {
    let d = revjul(h.jd, GREG_CAL);
    println!("{:<30}  {:04}-{:02}-{:02}  ({} days)",
        h.name, d.year, d.month, d.day, h.days);
}

// Lookup a specific holiday
let yom_kippur = jewish_holiday_jd(5785, "Yom Kippur").unwrap();

// Convert any JD to Hebrew date
let (year, month, day) = jd_to_hebrew_date(jdnow());
println!("Today in Hebrew: {} {}/{}", year, month, day);
```

### Easter & Christian Calendar

```rust
use celestial_core::*;

// Western Easter
let (y, m, d) = easter_gregorian(2025);
assert_eq!((y, m, d), (2025, 4, 20));

// Orthodox Easter
let (y, m, d) = easter_orthodox(2025);

// All moveable feasts (Ash Wednesday → Corpus Christi)
for feast in christian_feasts(2025) {
    println!("{:<30}  {:02}/{:02}  (Easter {:+} days)",
        feast.name, feast.month, feast.day, feast.easter_offset);
}

// Fixed feasts (Christmas, Epiphany, etc.)
for feast in christian_fixed_feasts(2025) {
    println!("{:<30}  {:02}/{:02}", feast.name, feast.month, feast.day);
}
```

### Islamic (Hijri) Calendar

```rust
use celestial_core::*;

// Convert today to Hijri
let (year, month, day) = hijri_from_jd(jdnow());
println!("Hijri: {} {} {}", day, hijri_month_name(month), year);

// All major observances for Hijri year 1446
for obs in islamic_observances(1446) {
    let d = revjul(obs.jd, GREG_CAL);
    println!("{:<35}  {:04}-{:02}-{:02}", obs.name, d.year, d.month, d.day);
}

// Which Hijri years overlap Gregorian 2025?
let (y1, y2) = gregorian_to_hijri_years(2025);
// → (1446, 1447)
```

### Hindu Panchānga

```rust
use celestial_core::*;

let jd = julday(2025, 3, 20, 6.0, GREG_CAL);
let p = panchanga(jd);

println!("Tithi:     {} ({:?})", p.tithi_name, p.paksha);
println!("Vara:      {}", p.vara_name);
println!("Nakshatra: {} pada {}", p.nakshatra_name, p.nakshatra_pada);
println!("Yoga:      {}", p.yoga_name);
println!("Karana:    {}", p.karana_name);

// Major Hindu festivals in a Gregorian year
for f in hindu_festivals(2025) {
    let d = revjul(f.jd, GREG_CAL);
    println!("{:<35}  {:04}-{:02}-{:02}", f.name, d.year, d.month, d.day);
}
```

### Buddhist Observances

```rust
use celestial_core::*;

// Vesak (Buddha Day) — full moon in Vaisakha
let vesak = vesak_jd(2025);
let d = revjul(vesak, GREG_CAL);
println!("Vesak 2025: {:04}-{:02}-{:02}", d.year, d.month, d.day);

// All four Uposatha phases for the year
let days = uposatha_days(2025);
for u in days.iter().filter(|u| u.phase == UposathaPhase::FullMoon).take(3) {
    let d = revjul(u.jd, GREG_CAL);
    println!("Full Moon Uposatha: {:04}-{:02}-{:02}", d.year, d.month, d.day);
}
```

### Nowruz, Persian Calendar & Bahá'í Calendar

```rust
use celestial_core::*;

// Nowruz = exact vernal equinox
let jd = nowruz_jd(2025);
let d = revjul(jd, GREG_CAL);
println!("Nowruz 2025: {:04}-{:02}-{:02} {:04.1}h UT",
    d.year, d.month, d.day, d.hour);

// Solar Hijri (Persian) year
println!("Solar Hijri: {}", gregorian_to_solar_hijri(2025)); // → 1404

// Bahá'í date
let bd = jd_to_bahai(jdnow());
println!("Bahá'í: {} ({}) {}, year {}",
    bd.day, bd.month_name, bd.year, bd.year);

// Bahá'í holy days
for h in bahai_holy_days(182) { // 182 BE = 2025/2026 CE
    let d = revjul(h.jd, GREG_CAL);
    println!("{:<35}  {:04}-{:02}-{:02}", h.name, d.year, d.month, d.day);
}
```


### Crossings, rise/set & eclipses

| Function | Description |
|---|---|
| `solcross_ut(lon, jd, flags)` | Next solar ecliptic longitude crossing |
| `mooncross_ut(lon, jd, flags)` | Next lunar ecliptic longitude crossing |
| `helio_cross_ut(body, lon, jd, flags, dir)` | Heliocentric crossing |
| `mooncross_node(jd, flags)` | Next Moon/node crossing |
| `rise_trans(jd, body, star, flags, rsmi, geo, press, temp)` | Rise / transit / set → `RiseTransResult` |
| `sol_eclipse_when_glob(jd, flags, type, back)` | Next solar eclipse → `EclipseResult` |
| `lun_eclipse_when(jd, flags, type, back)` | Next lunar eclipse → `EclipseResult` |

### Aspects & searches

| Function | Description |
|---|---|
| `match_aspect(p0, s0, p1, s1, aspect, orb)` | Test if aspect is within orb |
| `next_retro(body, jd, back, days, flags)` | Next retrograde station → `Option<RetroResult>` |
| `next_aspect(body, aspect, fixed, jd, …)` | Aspect to a fixed point |
| `next_aspect_with(body, aspect, other, jd, …)` | Aspect between two bodies |
| `next_aspect_cusp(body, aspect, cusp, jd, …)` | Aspect to a house cusp |
| `antiscion(pos, axis)` | Antiscion + contrantiscion → `Antiscion` |

### Celtic calendar

| Function | Description |
|---|---|
| `sabbat_jd(year, kind)` | Exact JD of a specific sabbat |
| `sabbats_for_year(year)` | All 8 sabbats sorted chronologically |
| `next_sabbat(jd)` | Next `Sabbat` at or after `jd` |
| `next_full_moon(jd)` | Next full moon JD (bisection, sub-second) |
| `esbats_for_year(year)` | All named full moons → `Vec<Esbat>` |
| `next_esbat(jd)` | Next `Esbat` at or after `jd` |

**`SabbatKind`:** `Yule(270°)` · `Imbolc(315°)` · `Ostara(0°)` · `Beltane(45°)` · `Litha(90°)` · `Lughnasadh(135°)` · `Mabon(180°)` · `Samhain(225°)`

**`EsbatName`:** Wolf · Snow · Worm · Pink · Flower · Strawberry · Buck · Sturgeon · Harvest · Hunter · Beaver · Cold · Blue

### Nowruz, Persian & Bahá'í Calendar

| Function | Description |
|---|---|
| `nowruz_jd(year)` | Exact JD of Nowruz (vernal equinox) |
| `gregorian_to_solar_hijri(year)` | Gregorian → Solar Hijri (Persian) year |
| `naw_ruz_jd(bahai_year)` | JD of Naw-Rúz (Bahá'í New Year) |
| `jd_to_bahai(jd)` | JD → `BahaiDate` (year, month 1–19/0, day) |
| `bahai_holy_days(bahai_year)` | 13 Bahá'í holy days for the year |

**`BahaiDate` fields:** `year` (BE) · `month` (1–19, 0=Ayyám-i-Há) · `day` · `month_name`

The Bahá'í calendar has 19 months of 19 days (361 days) + 4–5 intercalary days (Ayyám-i-Há) before the 19th month ('Alá', the month of fasting). Year 1 BE = 1844 CE.


### Vedic / Jyotish

| Function | Description |
|---|---|
| `long_to_rasi(lon)` | Sign 0–11 |
| `long_to_navamsa(lon)` | Navamsa 0–11 |
| `long_to_nakshatra(lon)` | `(nakshatra 0–26, pada 0–3)` |
| `nakshatra_name(n)` | Nakshatra name |
| `sign_lord(sign)` | Traditional sign ruler |
| `raman_houses(asc, mc, sandhi)` | 12 Raman house cusps |
| `tatkalika_relation(g1, g2)` | Temporary relationship (-1 / 0 / 1) |
| `naisargika_relation(g1, g2)` | Natural relationship |
| `residential_strength(lon, cusps)` | Bhava bala |
| `ochchabala(graha, lon)` | Exaltation strength |

---

## CI

Five independent pipelines, each triggered on changes to its own crate or `core/`:

| Pipeline | Jobs |
|---|---|
| **celestial-core** | `lint` (fmt + clippy) → `test` (549 unit tests) ‖ `fuzz` (32 property suites) |
| **celestial-cli** | `lint` (clippy) → `test` (cargo test) → `build` (3 OS) |
| **celestial-python** | `lint-rs` (clippy) ‖ `lint-py` (black + isort + ruff) → `test` (pure-logic pytest) ‖ `fuzz` (boundary + randomised) → `build` (maturin wheel, Python 3.12, 3 OS) |
| **celestial-js** | `lint-rs` (clippy) ‖ `lint-ts` (prettier + tsc) → `test` (pure-logic Node 22) ‖ `fuzz` (boundary × 5 runs) → `build` (napi-rs addon, Node 22, 3 OS) |
| **celestial-php** | `lint-rs` (clippy) → `build` (ext-php-rs + pure-logic tests, PHP 8.1) ‖ `fuzz` (boundary + 3× suite) |

`lint-rs` and `lint-py`/`lint-ts` always run in parallel with strict scope: Rust clippy never touches `.py`/`.ts` files and Python/JS linters never touch `.rs` files.

---

### Phase 2 — new Western chart types

All accessible via `celestial render --type <name>`:

```bash
# Cosmogram: wheel without houses
celestial render --date 2000-01-01 --lat 48.85 --lon 2.35 --type cosmogram

# Solar Return (specify the return year)
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 --type solar-return --return-year 2025

# Lunar Return (search from --date2)
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 --type lunar-return --date2 2025-01-01

# Secondary Progressions
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 --type progressed --years 39.5

# Solar Arc Directions
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 --type solar-arc --years 39.5

# Bi-wheel (synastry / transit overlay)
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 --type biwheel --date2 2025-03-20
```

The bi-wheel draws the natal chart as the inner ring and the second date's planets
on the outer ring (rendered in green). Cross-aspects between rings are shown as dashed
lines.

### Phase 3 — specialist Western charts

```bash
# 90° Midpoint Dial (Uranian/Hamburg)
celestial render --date 2000-01-01 --lat 48.85 --lon 2.35 --type dial

# Composite chart (midpoint of two nativities)
celestial render --date 1985-07-15 --date2 1990-03-20 --lat 48.85 --lon 2.35 --type composite

# Tri-wheel (natal + progressed + transits)
celestial render --date 1985-07-15 --date2 2010-01-01 --date3 2025-03-20 \
  --lat 48.85 --lon 2.35 --type triwheel

# Graphic Ephemeris (planetary motion over time)
celestial render --date 2025-01-01 --date2 2025-12-31 --type ephemeris

# Local Space chart (azimuth-based compass)
celestial render --date 2000-01-01 --lat 48.85 --lon 2.35 --type local-space
```

The **90° dial** compresses all four zodiacal quadrants onto a single circle.
Midpoints that are triggered by a planet within 1.5° are shown as short tick marks
and listed in the legend.

The **graphic ephemeris** plots each planet's ecliptic longitude against time. Retrograde
arcs and sign ingresses are immediately visible as changes in line direction.

### Phase 4 — Vedic / Jyotish charts

All Vedic charts use sidereal (Lahiri ayanamsa) positions via `--type`:

```bash
# South Indian Rasi chart (fixed-sign 4×4 grid)
celestial render --date 1990-05-15 --lat 13.08 --lon 80.27 --type rasi

# North Indian chart (rotating diamond layout, lagna = ASC sign)
celestial render --date 1990-05-15 --lat 13.08 --lon 80.27 --type north-indian

# Navamsa D9 divisional chart
celestial render --date 1990-05-15 --lat 13.08 --lon 80.27 --type navamsa

# Vimshottari dasha timeline (120-year bar chart)
celestial render --date 1990-05-15 --lat 13.08 --lon 80.27 --type dasha

# Ashtakavarga (7×12 bindu table + Sarvashtakavarga totals)
celestial render --date 1990-05-15 --lat 13.08 --lon 80.27 --type ashtakavarga

# Shadbala planetary strength (Ochchabala, Saptavargaja, Chesta, Dig bala)
celestial render --date 1990-05-15 --lat 13.08 --lon 80.27 --type shadbala
```

#### South Indian chart
The classic 4×4 grid with fixed sign positions. Pisces occupies the top-left cell;
signs proceed clockwise. The Vimshottari dasha schedule is shown below the grid.

#### North Indian diamond chart
A 12-cell diamond layout where house 1 always shows the ASC sign. House numbers
rotate clockwise from the lagna. Each triangular cell shows the house number,
rasi glyph, and planets.

#### Navamsa (D9) chart
Same grid layout as the South Indian chart but with each planet placed in its
D9 navamsa sign — one ninth of each rasi (3°20′ each).

#### Ashtakavarga
Classical 8-source bindu system. Each of 7 planets plus the ASC contributes
benefic bindus to signs by rule. The table shows:
- 7 planet rows × 12 sign columns (individual bindus 0–8)
- A Sarvashtakavarga totals row (sum across all 7 planets, 0–56 per sign)
- Color-coded: green = strong (≥ 5 / ≥ 28), red = weak (≤ 2 / ≤ 18)

#### Shadbala
Three of the six classical strength components (Shadbala means "six strengths"):

| Component | Calculation | Max |
|---|---|---|
| Ochchabala | Distance from exaltation point | 60 |
| Saptavargaja | Sign placement (D1 + D9 relationship) | 60 |
| Chesta bala | Motional strength (speed vs mean) | 60 |
| Dig bala | Directional strength (placeholder) | 60 |

Total > 100 shashtiamsas = planet considered strong.

### Phase 5 — Hellenistic / Persian

```bash
# Hellenistic natal chart with full dignity overlay
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 --type hellenistic

# Persian Firdaria timeline (75-year period chart)
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 --type firdaria

# Annual profection wheel (specify age with --years)
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 --type profection --years 39
```

#### Hellenistic dignities overlay (`--type hellenistic`)
The standard natal wheel with an extended dignities table below it showing:

| Column | Description |
|---|---|
| Dignity | Domicile (5), Exaltation (4), Triplicity (3), Term (2), Decan (1), Peregrine (0), Detriment (−5), Fall (−4) |
| Score | Numeric dignity score |
| Term lord | Egyptian bounds ruler (Ptolemy/Tetrabiblos) |
| Decan lord | Face ruler in Chaldean sequence |
| Triplicity | Day ruler / Night ruler |
| Sect | "in sect" or "out of sect" for day/night chart |

#### Firdaria (`--type firdaria`)
Persian planetary period system (Abū Maʿshar). A horizontal bar chart covering
75 years from birth. Day charts start with the Sun; night charts with the Moon.
Each major period (Sun=10y, Venus=8y, Mercury=13y…) is subdivided into 7
sub-periods (minor lords) following the same sequence.

Sequence for **day charts**: Sun 10 · Venus 8 · Mercury 13 · Moon 9 · Saturn 11 · Jupiter 12 · Mars 7 · North Node 3 · South Node 2 (= 75 years).

**Night chart** sequence starts with Moon.

#### Profection wheel (`--type profection --years AGE`)
Annual profection: the ASC advances one house per year of age. At age 35 the
profected ASC is in house 12 (35 mod 12 + 1). The chart renders the natal wheel
with a gold marker at the profected house. The profection lord (ruler of the
profected sign) is highlighted in the legend.

#### Core Hellenistic functions (accessible via Rust/Python/JS/PHP APIs)

| Function | Description |
|---|---|
| `egyptian_terms_ruler(lon)` | Returns the Egyptian bounds planet for a longitude |
| `decan_ruler(lon)` | Chaldean decan (face) ruler |
| `triplicity_rulers(lon)` | `(day, night, participating)` triplicity rulers |
| `full_dignity(body, lon, is_day)` | Returns `(Dignity, score)` |
| `almuten(lon, is_day)` | Planet with highest dignity score at a degree |
| `same_sect(body, is_day)` | True if planet is of the same sect as the chart |
| `firdaria(jd, is_day, span_years)` | Vec of `FirdariaPeriod` |
| `annual_profection(cusps, age)` | `(house_number, profected_lon)` |
| `monthly_profection(cusps, age_years, age_months)` | Same for sub-annual |

### Phase 6 — Chinese astrology

```bash
# Four Pillars of Destiny (Ba Zi)
celestial render --date 1985-07-15 --type bazi
```

The chart shows the four pillars (Year, Month, Day, Hour), each with:
- **Heavenly Stem** (天干): Jiǎ, Yǐ, Bǐng, Dīng, Wù, Jǐ, Gēng, Xīn, Rén, Guǐ
- **Earthly Branch** (地支): Rat, Ox, Tiger, Rabbit, Dragon, Snake, Horse, Goat, Monkey, Rooster, Dog, Pig
- Element and Yin/Yang polarity for both

An element balance bar chart shows the distribution of Wood, Fire, Earth, Metal and Water
across all eight stem+branch positions.

The current **solar term** (节气) is computed from the Sun's ecliptic longitude and
displayed with the degrees remaining until the next term.

#### Available via API

| Function | Returns |
|---|---|
| `four_pillars(jd, hour_ut, sun_lon)` | `[BaZiPillar; 4]` — year, month, day, hour |
| `solar_term_position(sun_lon)` | `(current_idx, deg_into, next_idx, deg_to_next)` |
| `sexagenary_name(cycle_idx)` | `(stem_name, animal_name)` |
| `SOLAR_TERMS` | 24-entry array of `(longitude°, pinyin, english)` |
| `HEAVENLY_STEMS` | 10-entry array of `(name, element, yang)` |
| `EARTHLY_BRANCHES` | 12-entry array of `(name, animal, element, yang)` |

### Phase 7 — Mesoamerican calendars

```bash
# Aztec + Maya calendar positions for any date
celestial render --date 2000-01-01 --type mesoamerican
# Also accepts: --type aztec, --type maya
```

The chart shows all four calendar systems side by side:

| Calendar | Cycle | Description |
|---|---|---|
| Tonalpohualli | 260 days | Aztec ritual calendar: 20 day signs × 13 trecena numbers |
| Xiuhpohualli | 365 days | Aztec solar year: 18 months of 20 days + 5 Nemontemi |
| Tzolkin | 260 days | Maya sacred calendar (same cycle as Tonalpohualli) |
| Haab | 365 days | Maya vague year: 18 months + 5-day Wayeb |

The **Calendar Round** (52-year cycle) is the LCM(260, 365) = 18,980-day combination.

All calculations use the **GMT correlation** (constant 584,283).

#### Available via API

| Function | Returns |
|---|---|
| `tonalpohualli(jd)` | `(trecena 1–13, sign_idx 0–19, nahuatl_name, english)` |
| `xiuhpohualli(jd)` | `(month_idx, day, month_name, english)` |
| `tzolkin(jd)` | `(trecena, sign_idx, mayan_name, english)` |
| `haab(jd)` | `(month_idx, day, month_name)` |
| `calendar_round(jd)` | `(tzolkin_trecena, tzolkin_sign, haab_day, haab_month)` |
| `GMT_CORRELATION` | `584_283i64` — the correlation constant |

### Phase 8 — Indigenous / other traditions

```bash
# Medicine Wheel + Egyptian decans
celestial render --date 2000-01-01 --lat 48.85 --lon 2.35 --type medicine-wheel
# Also accepts: --type indigenous, --type egyptian-decans
```

#### Medicine Wheel (modern synthesis)

The chart shows a compass-rose wheel with the Sun's current position. Birth totem
and element are derived from the **Sun Bear / Wabun Wind system** (1980 — a modern
New Age synthesis, not a traditional single-nation indigenous system).

12 birth totems correspond to roughly 30° Sun longitude segments:

Snow Goose · Otter · Cougar · Red Hawk · Beaver · Deer · Flicker · Sturgeon ·
Brown Bear · Raven · Snake · Elk

Each totem belongs to a clan (Turtle/Earth, Butterfly/Air, Thunderbird/Fire, Frog/Water)
and a season.

#### Egyptian Decans

Each of the 36 ten-degree sections of the ecliptic corresponds to a traditional Egyptian
decan name (from Firmicus Maternus / Ptolemy) and its associated heliacal rising star.

| Function | Returns |
|---|---|
| `medicine_wheel_totem(sun_lon)` | `(animal, element, clan, season)` |
| `egyptian_decan(lon)` | `(decan_idx 0–35, decan_name, rising_star)` |

> **Note:** The Medicine Wheel system presented here is the Sun Bear synthesis from
> *The Medicine Wheel* (1980). Traditional indigenous astronomical knowledge varies
> enormously by nation and is generally observational and seasonal, not a natal chart
> system. This implementation is labelled accordingly in the SVG output.
