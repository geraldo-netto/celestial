# celestial

[![celestial-core](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-core.yml/badge.svg)](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-core.yml)
[![celestial-cli](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-cli.yml/badge.svg)](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-cli.yml)
[![celestial-python](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-python.yml/badge.svg)](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-python.yml)
[![celestial-js](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-js.yml/badge.svg)](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-js.yml)
[![celestial-php](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-php.yml/badge.svg)](https://github.com/geraldo-netto/celestial/actions/workflows/celestial-php.yml)

A pure-Rust astronomical engine — no C compiler, no data files, **no external dependencies**.

Covers the Swiss Ephemeris API surface: planetary positions, house cusps, eclipses, crossings, rise/set, aspects, retrogrades, Vedic helpers, Celtic Wheel of the Year, and eight astrological traditions across 24 chart types.

---

## Table of Contents

- [Workspace layout](#workspace-layout)
- [CLI](#cli)
  - [Commands](#commands)
  - [Examples](#examples)
  - [Chart rendering](#chart-rendering----celestial-render)
  - [Phase 1 — Wheel enhancements](#phase-1--wheel-enhancements)
  - [Phase 2 — New Western chart types](#phase-2--new-western-chart-types)
  - [Phase 3 — Specialist Western charts](#phase-3--specialist-western-charts)
  - [Phase 4 — Vedic / Jyotish charts](#phase-4--vedic--jyotish-charts)
  - [Phase 5 — Hellenistic / Persian](#phase-5--hellenistic--persian)
  - [Phase 6 — Chinese astrology](#phase-6--chinese-astrology)
  - [Phase 7 — Mesoamerican calendars](#phase-7--mesoamerican-calendars)
  - [Phase 8 — Indigenous / other traditions](#phase-8--indigenous--other-traditions)
  - [Plugin architecture](#plugin-architecture)
- [Rust API](#rust-api)
- [Python](#python)
- [JavaScript / TypeScript](#javascript--typescript)
- [PHP](#php)
- [Accuracy](#accuracy)
- [Building from source](#building-from-source)
- [Benchmarks](#benchmarks)
- [API reference](#api-reference)
- [Documentation](#documentation)
- [CI](#ci)
- [License](#license)

---

## Workspace layout

```
celestial-workspace/
├── core/                        celestial-core — pure-Rust engine, zero dependencies
│   └── src/
│       ├── astronomy/           IAU 2000B nutation, VSOP87, 3-pass light-time, rise/set
│       └── functions/
│           ├── chart.rs         Phases 1–4 (Western + Vedic helpers)
│           ├── hellenistic.rs   Phase 5: dignity, almuten, firdaria
│           ├── chinese.rs       Phase 6: Ba Zi, solar terms
│           ├── mesoamerican.rs  Phase 7: Tonalpohualli, Tzolkin, Haab
│           ├── indigenous.rs    Phase 8: Medicine Wheel, Egyptian decans
│           ├── vedic.rs         Jyotish utilities
│           ├── calc.rs          calc_ut, calc_many (parallel), nutation
│           └── ...              aspects, eclipses, crossings, calendars
│   └── tests/
│       ├── new_features_test.rs Phases 2–4 core tests
│       ├── phase1_test.rs       Phase 1: minor aspects, antiscia, dignities
│       ├── phase5_test.rs       Phase 5: Hellenistic / Persian
│       ├── phase6_test.rs       Phase 6: Chinese
│       ├── phase7_test.rs       Phase 7: Mesoamerican
│       └── phase8_test.rs       Phase 8: Indigenous / Egyptian
├── cli/                         celestial — 11 sub-commands + plugin system
│   └── src/cmd/render/
│       ├── mod.rs               Dispatch, shared helpers (~3200 lines)
│       ├── western.rs           Natal, cosmogram, returns, progressions, bi-wheel
│       ├── specialist.rs        Dial, composite, tri-wheel, ephemeris, local-space
│       ├── vedic.rs             Rasi, navamsa, dasha, ashtakavarga, shadbala
│       ├── hellenistic.rs       Hellenistic overlay, firdaria, profection
│       ├── chinese.rs           Ba Zi (Four Pillars)
│       ├── mesoamerican.rs      Aztec + Maya calendar chart
│       └── indigenous.rs        Medicine Wheel + Egyptian decans
├── bindings/
│   ├── python/                  PyO3 — 152 exported functions
│   ├── js/                      napi-rs — 155 exported functions + index.d.ts
│   └── php/                     ext-php-rs — 130 exported functions
├── fuzz/                        53 property-test suites (cargo run)
├── tests/fixtures/              reference_values.json — cross-language canonical values
└── docs/                        Extended documentation (see Documentation section)
```

The public API is a **single flat namespace** — `use celestial_core::*` gives you everything.

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
celestial render    Template-driven SVG chart rendering (24 chart types)
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

# Full astrological chart
celestial chart \
  --date "1985-07-14 14:30" \
  --lat 48.8566 \
  --lon 2.3522 \
  --name "Bastille Day 1985" \
  --svg chart.svg

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
celestial calendar jewish --year 2025
celestial calendar easter --year 2025
celestial calendar islamic --year 2025
celestial calendar islamic --convert 2025-04-20
celestial calendar panchanga --date 2025-03-20
celestial calendar panchanga --festivals --year 2025
celestial calendar vesak --year 2025
celestial calendar vesak --uposatha --year 2025
celestial calendar nowruz --year 2025
celestial calendar nowruz --bahai --year 2025
```

All subcommands accept `--json` for machine-readable output.

---

### Chart rendering — `celestial render`

Produces SVG charts for 24 astrological chart types across eight traditions.

![Celestial Chart — Paris J2000.0](docs/example_chart.svg)

```bash
# Built-in SVG chart (dark theme, full legend)
celestial render --date 2025-03-20 --lat 48.85 --lon 2.35 --out chart.svg

# Override palette and title
celestial render --date now --lat 48.85 --lon 2.35 \
  --var "title=My Chart" --var "bg_color=#1a1a2e" --out chart.svg

# TOML config file
celestial render --config chart.toml --out chart.svg

# Bootstrap a custom template
celestial render --print-template > my_chart.tmpl

# Inspect all available template variables as JSON
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
```

Templates use [TinyTemplate](https://github.com/bheisler/TinyTemplate) syntax. All wheel geometry (planet x/y, cusp lines, aspect endpoints) is pre-computed in Rust — templates need no math, just `{planet.x}`, `{planet.y}`, etc.

| Variable | Type | Description |
|---|---|---|
| `{date}` | string | ISO date |
| `{jd}` | float | Julian Day |
| `{asc}` / `{mc}` / `{ic}` / `{dsc}` | float | Angle longitudes |
| `{asc_dms}` … | string | DMS formatted angles |
| `{planets}` | list | 12 bodies with `.lon .lat .x .y .glyph .dms .retro` |
| `{signs}` | list | 12 sign sectors with `.spoke_x1 .spoke_y1 .glyph_x .glyph_y` |
| `{houses}` | list | 12 cusps with `.x1 .y1 .x2 .y2 .num_x .num_y .dms` |
| `{aspects}` | list | Active aspects with `.x1 .y1 .x2 .y2 .orb .applying .is_hard` |
| `{moon_phase_name}` | string | Current lunar phase |
| `{moon_illumination}` | float | Illumination 0–100% |
| `{vars.key}` | string | Any `--var key=value` or `[vars] key = "value"` |

---

### Phase 1 — Wheel enhancements

The natal wheel renders these additional layers automatically:

**Minor aspects** — seven new aspects as dashed lines at reduced opacity:

| Aspect | Angle | Orb |
|---|---|---|
| Semi-sextile | 30° | 2° |
| Semi-square | 45° | 2° |
| Quintile | 72° | 1.5° |
| Sesquiquadrate | 135° | 2° |
| Biquintile | 144° | 1.5° |
| Septile | 51.43° | 1° |
| Novile | 40° | 1° |

**Station markers** — planets with `|speed| < 0.05°/day` receive a small `S` badge.

**Antiscia and contra-antiscia** — each planet's antiscion (mirror over the 0°Cancer–0°Capricorn solstice axis) is shown as a faint glyph at the house ring:

```
antiscion(lon)         = (180° − lon) mod 360°
contra_antiscion(lon)  = (360° − lon) mod 360°
```

**Arabic Parts / Lots** — all seven traditional Arabic Parts placed on the wheel (Fortune, Spirit, Love, Necessity, Courage, Victory, Nemesis). Day/night reversal of Fortune and Spirit is applied automatically based on whether the Sun is above the horizon.

**Fixed stars** — 15 significant fixed stars (Algol, Aldebaran, Sirius, Regulus, Spica, Antares, etc.) plotted as magnitude-proportional dots on the sign band.

**Essential dignities table**:

| Status | Description |
|---|---|
| **domicile** | Planet rules the sign it occupies |
| **exaltation** | Planet is in its exaltation sign |
| **detriment** | Planet is in the sign opposite its domicile |
| **fall** | Planet is in the sign opposite its exaltation |
| peregrine | None of the above |

---

### Phase 2 — New Western chart types

```bash
# Cosmogram (wheel without houses)
celestial render --date 2000-01-01 --lat 48.85 --lon 2.35 --type cosmogram

# Solar Return (specify the return year)
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 \
  --type solar-return --return-year 2025

# Lunar Return (search from --date2)
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 \
  --type lunar-return --date2 2025-01-01

# Secondary Progressions
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 \
  --type progressed --years 39.5

# Solar Arc Directions
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 \
  --type solar-arc --years 39.5

# Bi-wheel (synastry / transit overlay)
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 \
  --type biwheel --date2 2025-03-20
```

The bi-wheel draws natal planets as the inner ring and second-date planets on the outer ring (rendered in green). Cross-aspects between rings are shown as dashed lines.

---

### Phase 3 — Specialist Western charts

```bash
# 90° Midpoint Dial (Uranian/Hamburg)
celestial render --date 2000-01-01 --lat 48.85 --lon 2.35 --type dial

# Composite chart (midpoint of two nativities)
celestial render --date 1985-07-15 --date2 1990-03-20 \
  --lat 48.85 --lon 2.35 --type composite

# Tri-wheel (natal + progressed + transits)
celestial render --date 1985-07-15 --date2 2010-01-01 --date3 2025-03-20 \
  --lat 48.85 --lon 2.35 --type triwheel

# Graphic Ephemeris (planetary motion over time)
celestial render --date 2025-01-01 --date2 2025-12-31 --type ephemeris

# Local Space chart (azimuth-based compass)
celestial render --date 2000-01-01 --lat 48.85 --lon 2.35 --type local-space
```

The **90° dial** compresses all four zodiacal quadrants onto a single circle. Midpoints triggered by a planet within 1.5° are shown as tick marks and listed in the legend.

The **graphic ephemeris** plots each planet's ecliptic longitude against time. Retrograde arcs and sign ingresses are immediately visible as changes in line direction.

---

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

# Shadbala planetary strength
celestial render --date 1990-05-15 --lat 13.08 --lon 80.27 --type shadbala
```

**Ashtakavarga** — 8-source bindu system. 7 planet rows × 12 sign columns (0–8 bindus each) plus a Sarvashtakavarga totals row (0–56). Green = strong (≥ 5 / ≥ 28), red = weak (≤ 2 / ≤ 18).

**Shadbala** — three strength components: Ochchabala (exaltation distance, max 60), Saptavargaja (sign placement), Chesta bala (motional strength). Total > 100 shashtiamsas = strong planet.

---

### Phase 5 — Hellenistic / Persian

```bash
# Hellenistic natal chart with full dignity overlay
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 --type hellenistic

# Persian Firdaria timeline (75-year period chart)
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 --type firdaria

# Annual profection wheel (specify age with --years)
celestial render --date 1985-07-15 --lat 48.85 --lon 2.35 \
  --type profection --years 39
```

The **hellenistic** overlay adds a dignity table: dignity name, score, Egyptian term lord, Chaldean decan lord, triplicity rulers, and sect status for each planet.

**Firdaria** (Abū Maʿshar) — 75-year horizontal bar chart. Day charts start with Sun; night charts with Moon. Period lengths: Sun 10y · Venus 8y · Mercury 13y · Moon 9y · Saturn 11y · Jupiter 12y · Mars 7y · North Node 3y · South Node 2y.

**Profection** — the ASC advances one house per year of age. At age 35 the profected ASC is in house 12 (35 mod 12 + 1). The profection lord is highlighted in the legend.

Core Hellenistic API functions:

| Function | Description |
|---|---|
| `egyptian_terms_ruler(lon)` | Egyptian bounds planet for a longitude |
| `decan_ruler(lon)` | Chaldean decan (face) ruler |
| `triplicity_rulers(lon)` | `(day, night, participating)` triplicity rulers |
| `full_dignity(body, lon, is_day)` | Returns `(Dignity, score)` |
| `almuten(lon, is_day)` | Planet with highest dignity score at a degree |
| `is_day_chart(sun_lon, cusps)` | True if Sun is above the horizon |
| `same_sect(body, is_day)` | True if planet matches the chart's sect |
| `firdaria(jd, is_day, span_years)` | Vec of `FirdariaPeriod` |
| `annual_profection(cusps, age)` | `(house_number, profected_lon)` |
| `monthly_profection(cusps, age_years, months)` | Sub-annual profection |

---

### Phase 6 — Chinese astrology

```bash
# Four Pillars of Destiny (Ba Zi)
celestial render --date 1985-07-15 --type bazi
```

The chart shows four pillars (Year, Month, Day, Hour), each with Heavenly Stem (天干), Earthly Branch (地支), element, and Yin/Yang polarity. An element balance bar chart shows Wood/Fire/Earth/Metal/Water distribution. The current solar term (节气) is displayed with degrees remaining until the next term.

| Function | Returns |
|---|---|
| `four_pillars(jd, hour_ut, sun_lon)` | `[BaZiPillar; 4]` — year, month, day, hour |
| `solar_term_position(sun_lon)` | `(current_idx, deg_into, next_idx, deg_to_next)` |
| `sexagenary_name(cycle_idx)` | `(stem_name, animal_name)` |
| `SOLAR_TERMS` | 24-entry array of `(longitude°, pinyin, english)` |
| `HEAVENLY_STEMS` | 10-entry array of `(name, element, yang)` |
| `EARTHLY_BRANCHES` | 12-entry array of `(name, animal, element, yang)` |

---

### Phase 7 — Mesoamerican calendars

```bash
# Aztec + Maya calendar positions for any date
celestial render --date 2000-01-01 --type mesoamerican
```

| Calendar | Cycle | Description |
|---|---|---|
| Tonalpohualli | 260 days | Aztec ritual calendar: 20 day signs × 13 trecena numbers |
| Xiuhpohualli | 365 days | Aztec solar year: 18 months of 20 days + 5 Nemontemi |
| Tzolkin | 260 days | Maya sacred calendar (same cycle as Tonalpohualli) |
| Haab | 365 days | Maya vague year: 18 months + 5-day Wayeb |

The **Calendar Round** (52-year cycle) is LCM(260, 365) = 18,980 days. All calculations use the **GMT correlation** (constant 584,283).

| Function | Returns |
|---|---|
| `tonalpohualli(jd)` | `(trecena 1–13, sign_idx 0–19, nahuatl_name, english)` |
| `xiuhpohualli(jd)` | `(month_idx, day, month_name, english)` |
| `tzolkin(jd)` | `(trecena, sign_idx, mayan_name, english)` |
| `haab(jd)` | `(month_idx, day, month_name)` |
| `calendar_round(jd)` | `(tzolkin_trecena, tzolkin_sign, haab_day, haab_month)` |
| `GMT_CORRELATION` | `584_283i64` |

---

### Phase 8 — Indigenous / other traditions

```bash
# Medicine Wheel + Egyptian decans
celestial render --date 2000-01-01 --lat 48.85 --lon 2.35 --type medicine-wheel
```

**Medicine Wheel** — compass-rose wheel using the Sun Bear / Wabun Wind synthesis (1980). 12 birth totems correspond to ~30° Sun longitude segments: Snow Goose · Otter · Cougar · Red Hawk · Beaver · Deer · Flicker · Sturgeon · Brown Bear · Raven · Snake · Elk. Each totem belongs to a clan (Turtle/Earth, Butterfly/Air, Thunderbird/Fire, Frog/Water) and a season.

**Egyptian decans** — each of 36 ten-degree ecliptic sections corresponds to a traditional decan name (Firmicus Maternus / Ptolemy) and its heliacal rising star.

| Function | Returns |
|---|---|
| `medicine_wheel_totem(sun_lon)` | `(animal, element, clan, season)` |
| `egyptian_decan(lon)` | `(decan_idx 0–35, decan_name, rising_star)` |

> **Note:** The Medicine Wheel system presented here is the Sun Bear synthesis from *The Medicine Wheel* (1980). Traditional indigenous astronomical knowledge varies enormously by nation and is generally observational and seasonal, not a natal chart system. This implementation is labelled accordingly in the SVG output.

---

### Plugin architecture

Any executable named `celestial-<n>` on `$PATH` becomes a first-class subcommand:

```bash
celestial --list-plugins              # discover all installed plugins
celestial synastry --date 1985-01-01  # runs celestial-synastry if on PATH
```

---

## Rust API

```toml
[dependencies]
celestial-core = { path = "core" }
```

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

    // Parallel multi-body calculation
    let planets = [SUN, MOON, MERCURY, VENUS, MARS, JUPITER, SATURN];
    let results = calc_many(jd, &planets, FLG_BUILTIN | FLG_SPEED)?;

    // Placidus house cusps — Paris
    let h = houses(jd, 48.85, 2.35, b'P')?;
    println!("ASC={:.2}°  MC={:.2}°", h.ascmc[0], h.ascmc[1]);

    // Sidereal (Lahiri) position of Mars
    set_sid_mode(SIDM_LAHIRI, 0.0, 0.0);
    let mars = calc_ut(jd, MARS, FLG_BUILTIN | FLG_SIDEREAL)?;

    // Ba Zi (Four Pillars)
    let pillars = four_pillars(jd, 14.0, sun.lon);
    println!("Year: {} {}", pillars[0].stem_name, pillars[0].branch_name);

    // Tonalpohualli
    let (trecena, sign, name, _) = tonalpohualli(jd);
    println!("Aztec day: {trecena} {name}");

    // Medicine Wheel
    let (animal, element, clan, season) = medicine_wheel_totem(sun.lon);
    println!("Totem: {animal} ({element}, {clan}, {season})");

    // Hellenistic dignities
    let (dignity, score) = full_dignity(SUN, sun.lon, is_day_chart(sun.lon, &h.cusps))?;
    println!("Sun dignity: {dignity:?} (score {score})");

    Ok(())
}
```

For extended Rust examples see [docs/rust.md](docs/rust.md).

---

## Python

```bash
cd bindings/python && pip install maturin && maturin develop
```

```python
import celestial_py as celestial

jd  = celestial.julday(2025, 3, 20, 9.0, celestial.GREG_CAL)
pos = celestial.calc_ut(jd, celestial.SUN, celestial.FLG_BUILTIN | celestial.FLG_SPEED)
print(f"Sun  lon={pos.lon:.4f}°  dist={pos.dist:.6f} AU")

h = celestial.houses_ex(jd, 0, 48.85, 2.35, ord("P"))
print(f"ASC={h.ascmc[0]:.2f}°  MC={h.ascmc[1]:.2f}°")

# Parallel multi-body
results = celestial.calc_many(jd, [0,1,2,3,4,5,6], celestial.FLG_BUILTIN)

# Sidereal (Lahiri)
celestial.set_sid_mode(celestial.SIDM_LAHIRI, 0.0, 0.0)
moon = celestial.calc_ut(jd, celestial.MOON, celestial.FLG_BUILTIN | celestial.FLG_SIDEREAL)

# Phase 5 — Hellenistic
is_day   = celestial.is_day_chart(pos.lon, h.cusps)
dignity, score = celestial.full_dignity(celestial.SUN, pos.lon, is_day)

# Phase 6 — Ba Zi
pillars = celestial.four_pillars(jd, 9.0, pos.lon)

# Phase 7 — Mesoamerican
trecena, sign_idx, name, english = celestial.tonalpohualli(jd)

# Phase 8 — Medicine Wheel
animal, element, clan, season = celestial.medicine_wheel_totem(pos.lon)

# Moon phase
phase      = celestial.moon_phase(jd)
illumination = celestial.moon_illumination(jd)
```

For the full Python API reference see [docs/python.md](docs/python.md).

---

## JavaScript / TypeScript

```bash
cd bindings/js && npm install && npm run build
```

```typescript
import * as celestial from "celestial-js";

const jd  = celestial.julday(2025, 3, 20, 9.0, celestial.GREG_CAL);
const sun = celestial.calc_ut(jd, 0, celestial.FLG_BUILTIN | celestial.FLG_SPEED);
console.log(`Sun  lon=${sun.lon.toFixed(4)}°`);

const h = celestial.houses_ex(jd, 0, 48.85, 2.35, "P".charCodeAt(0));
console.log(`ASC=${h.ascmc[0].toFixed(2)}°  MC=${h.ascmc[1].toFixed(2)}°`);

// Parallel multi-body
const results = celestial.calc_many(jd, [0,1,2,3,4,5,6], celestial.FLG_BUILTIN);

// Phase 5 — Hellenistic
const isDay = celestial.is_day_chart(sun.lon, h.cusps);
const [dignityName, score] = celestial.full_dignity(0, sun.lon, isDay);

// Phase 6 — Ba Zi
const pillars = celestial.four_pillars(jd, 9.0, sun.lon);

// Phase 7 — Mesoamerican
const [trecena, signIdx, name, english] = celestial.tonalpohualli(jd);

// Phase 8 — Medicine Wheel
const [animal, element, clan, season] = celestial.medicine_wheel_totem(sun.lon);
```

Full TypeScript types in `bindings/js/index.d.ts`. For the full JS API reference see [docs/javascript.md](docs/javascript.md).

---

## PHP

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

## Accuracy

| Body | Longitude | Latitude | Distance |
|---|---|---|---|
| Sun | ~1″ | ~0.1″ | ~0.00001 AU |
| Moon | ~10″ | ~4″ | ~4 km |
| Inner planets | ~1″–30″ | ~1″–10″ | ~0.001 AU |
| Outer planets | ~1″–60″ | ~1″–30″ | ~0.01 AU |

Accuracy degrades beyond ±3000 years from J2000. Nutation: **IAU 2000B** luni-solar series (77 terms, ~1 mas accuracy). Obliquity: **IAU 2006** formula.

Precision vs Meeus *Astronomical Algorithms* 2nd ed. using `calc()` (TT input):

| Body | Error | Model |
|---|---|---|
| Sun 1992-Oct-13 | 3.2″ | VSOP87 + IAU 2000B nutation |
| Moon 1992-Apr-12 | 0.7″ | ELP2000-82 + IAU 2000B nutation |
| julday J2000 | exact | — |

---

## Building from source

```bash
# Full workspace build
cargo build

# Tests — 627 unit/integration + 53 property-test suites
cargo test --package celestial-core -- --test-threads=1
cargo test --package celestial-cli
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
# Rust micro-benchmarks
cargo bench --package celestial-core

# Precision + performance comparison vs pyephem and astropy
pip install astropy pyephem
python3 benches/precision_comparison.py
```

Precision vs Meeus benchmarks confirmed at ~3.2″ Sun / ~0.7″ Moon. `SYNODIC_MONTH = 29.530_588_853` days.

---

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
| `calc_many(jd, bodies, flags)` | Parallel multi-body → `Vec<PlanetPos>` |
| `calc_ut_many(jd, bodies, flags)` | Parallel multi-body (UT) |
| `calc_pctr(jd, body, center, flags)` | Position relative to center body |
| `fixstar_ut(name, jd, flags)` | Fixed star position → `FixStarPos` |
| `fixstar_mag(name)` | Fixed star visual magnitude |
| `nutation(jd, flags)` | IAU 2000B nutation → `NutationResult` |
| `nod_aps(jd, body, flags, method)` | Nodes and apsides → `NodAps` |

**`PlanetPos` fields:** `lon` `lat` `dist` `speed_lon` `speed_lat` `speed_dist` `ret_flags`

**Body constants:** `SUN=0` `MOON=1` `MERCURY=2` `VENUS=3` `MARS=4` `JUPITER=5` `SATURN=6` `URANUS=7` `NEPTUNE=8` `PLUTO=9` `MEAN_NODE=10` `TRUE_NODE=11` `CHIRON=15`

**Flag constants:** `FLG_BUILTIN` `FLG_SPEED` `FLG_SIDEREAL` `FLG_EQUATORIAL` `FLG_HELCTR` `FLG_TOPOCTR` `FLG_NONUT` `FLG_RADIANS` `FLG_XYZ`

### Configuration

| Function | Description |
|---|---|
| `planet_name(body)` | Body number → display name |
| `set_sid_mode(mode, t0, ayan_t0)` | Activate a sidereal mode |
| `set_topo(lon, lat, alt_m)` | Set topocentric observer |
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

**`HouseResult`:** `cusps[12]` (index 0 skipped), `ascmc[8]` (0=ASC, 1=MC, 2=ARMC, 3=Vertex, 4=Equatorial ASC)

**House systems:** `b'P'` Placidus · `b'K'` Koch · `b'E'` Equal · `b'W'` Whole-Sign · `b'O'` Porphyry · `b'R'` Regiomontanus · `b'C'` Campanus · `b'M'` Morinus · `b'B'` Alcabitus · `b'X'` Axial Rotation

### Moon phases

| Function | Description |
|---|---|
| `moon_phase(jd)` | Named phase → `MoonPhase` (8 variants) |
| `moon_illumination(jd)` | Fraction illuminated (0.0–1.0) |
| `moon_elongation(jd)` | Moon–Sun elongation (0°–360°) |
| `next_new_moon(jd)` | JD of next new moon |
| `next_first_quarter(jd)` | JD of next first quarter |
| `next_full_moon_phase(jd)` | JD of next full moon |
| `next_last_quarter(jd)` | JD of next last quarter |
| `moon_phases_for_month(year, month)` | All phases in a calendar month |
| `moon_phase_info(jd)` | Rich `MoonPhaseInfo` with prev/next phase |

**`MoonPhase` variants:** `NewMoon` · `WaxingCrescent` · `FirstQuarter` · `WaxingGibbous` · `FullMoon` · `WaningGibbous` · `LastQuarter` · `WaningCrescent`

**`MoonPhaseInfo` fields:** `phase` · `illumination` · `elongation` · `age_days` · `prev_phase_jd` · `prev_phase_name` · `next_phase_jd` · `next_phase_name`

### Calendars & religious observances

| Function | Description |
|---|---|
| `sabbats_for_year(year)` | All 8 Celtic sabbats → `Vec<Sabbat>` |
| `esbats_for_year(year)` | All named full moons → `Vec<Esbat>` |
| `omer_from_jd(jd)` | Tonight's Omer day (if in period) |
| `omer_days(hebrew_year)` | Full 49-day schedule |
| `jewish_holidays(hebrew_year)` | All major Jewish holidays |
| `easter_gregorian(year)` | Western Easter → `(y, m, d)` |
| `easter_orthodox(year)` | Orthodox Easter → `(y, m, d)` |
| `christian_feasts(year)` | All moveable feasts (Ash Wednesday → Corpus Christi) |
| `hijri_from_jd(jd)` | JD → Hijri date |
| `islamic_observances(hijri_year)` | Major Islamic observances |
| `panchanga(jd)` | Hindu Panchānga → `PanchangaResult` |
| `hindu_festivals(year)` | Major Hindu festivals |
| `vesak_jd(year)` | Vesak (Buddha Day) JD |
| `uposatha_days(year)` | All four Uposatha phases |
| `nowruz_jd(year)` | Nowruz JD (exact vernal equinox) |
| `jd_to_bahai(jd)` | JD → `BahaiDate` |
| `bahai_holy_days(bahai_year)` | 13 Bahá'í holy days |

### Crossings, rise/set & eclipses

| Function | Description |
|---|---|
| `solcross_ut(lon, jd, flags)` | Next solar ecliptic longitude crossing |
| `mooncross_ut(lon, jd, flags)` | Next lunar ecliptic longitude crossing |
| `rise_trans(jd, body, star, flags, rsmi, geo, press, temp)` | Rise / transit / set |
| `sol_eclipse_when_glob(jd, flags, type, back)` | Next solar eclipse |
| `lun_eclipse_when(jd, flags, type, back)` | Next lunar eclipse |

### Aspects & searches

| Function | Description |
|---|---|
| `calc_chart_aspects(positions, aspects, orb)` | All aspects in a chart |
| `calc_chart_aspects_auto(positions, orbs)` | Aspects with per-planet orb table |
| `match_aspect(p0, s0, p1, s1, aspect, orb)` | Test if aspect is within orb |
| `next_retro(body, jd, back, days, flags)` | Next retrograde station |
| `next_aspect_with(body, aspect, other, jd, …)` | Aspect between two bodies |
| `next_aspect_cusp(body, aspect, cusp, jd, …)` | Aspect to a house cusp |
| `sign_ingress_ut(body, jd, flags, back)` | Next sign ingress |
| `retrograde_station_ut(body, jd, flags)` | Retrograde and direct station JDs |

### Vedic / Jyotish

| Function | Description |
|---|---|
| `long_to_rasi(lon)` | Sign 0–11 |
| `long_to_navamsa(lon)` | Navamsa 0–11 |
| `long_to_nakshatra(lon)` | `(nakshatra 0–26, pada 0–3)` |
| `nakshatra_name(n)` | Nakshatra name |
| `raman_houses(asc, mc, sandhi)` | 12 Raman house cusps |
| `vimshottari_dasha(jd, moon_lon, span)` | Dasha period list |
| `ochchabala(graha, lon)` | Exaltation strength (0–60) |
| `tatkalika_relation(g1, g2)` | Temporary relationship (−1 / 0 / 1) |
| `naisargika_relation(g1, g2)` | Natural relationship |
| `residential_strength(lon, cusps)` | Bhava bala |

### Hellenistic / Persian (Phase 5)

| Function | Description |
|---|---|
| `egyptian_terms_ruler(lon)` | Egyptian bounds planet |
| `decan_ruler(lon)` | Chaldean decan ruler |
| `triplicity_rulers(lon)` | Day / night / participating rulers |
| `full_dignity(body, lon, is_day)` | `(Dignity, score)` |
| `almuten(lon, is_day)` | Planet with highest dignity at a degree |
| `is_day_chart(sun_lon, cusps)` | True if Sun is above horizon |
| `same_sect(body, is_day)` | Sect membership |
| `firdaria(jd, is_day, span_years)` | Firdaria period list |
| `annual_profection(cusps, age)` | `(house_number, profected_lon)` |
| `monthly_profection(cusps, age_years, months)` | Sub-annual profection |

### Chinese (Phase 6)

| Function | Description |
|---|---|
| `four_pillars(jd, hour_ut, sun_lon)` | `[BaZiPillar; 4]` |
| `solar_term_position(sun_lon)` | Current/next solar term info |
| `sexagenary_name(idx)` | Stem + animal name |
| `SOLAR_TERMS` | 24-term array (longitude, pinyin, english) |
| `HEAVENLY_STEMS` | 10 stems (name, element, yang) |
| `EARTHLY_BRANCHES` | 12 branches (name, animal, element, yang) |

### Mesoamerican (Phase 7)

| Function | Description |
|---|---|
| `tonalpohualli(jd)` | Aztec 260-day day position |
| `xiuhpohualli(jd)` | Aztec 365-day solar year position |
| `tzolkin(jd)` | Maya 260-day sacred calendar |
| `haab(jd)` | Maya 365-day vague year |
| `calendar_round(jd)` | Combined Tzolkin + Haab position |
| `GMT_CORRELATION` | 584,283 — the correlation constant |

### Indigenous / Egyptian (Phase 8)

| Function | Description |
|---|---|
| `medicine_wheel_totem(sun_lon)` | `(animal, element, clan, season)` |
| `egyptian_decan(lon)` | `(decan_idx, decan_name, rising_star)` |

---

## Documentation

Extended per-topic documentation lives in the `docs/` directory:

| Document | Contents |
|---|---|
| [docs/index.md](docs/index.md) | Overview, quick-start for all languages, architecture |
| [docs/api_reference.md](docs/api_reference.md) | Complete function reference — all 8 phases and return types |
| [docs/rust.md](docs/rust.md) | Full Rust crate API with extended code examples |
| [docs/python.md](docs/python.md) | Python (PyO3) binding — installation, all functions, constants |
| [docs/javascript.md](docs/javascript.md) | JavaScript / TypeScript (napi-rs) — typed API, full guide |
| [docs/php.md](docs/php.md) | PHP (ext-php-rs) — installation, phpstan integration, all functions |

---

## CI

Five independent pipelines, each triggered on changes to its crate or `core/`:

| Pipeline | Jobs |
|---|---|
| **celestial-core** | `lint` (fmt + clippy) → `test` (627 unit tests) ‖ `fuzz` (53 suites) |
| **celestial-cli** | `lint` (clippy) → `test` (81 tests) → `build` (3 OS) |
| **celestial-python** | `lint-rs` ‖ `lint-py` (black + ruff) → `test` (245 pure-logic) → `build` (maturin wheel) |
| **celestial-js** | `lint-rs` ‖ `lint-ts` (eslint + tsc) → `test` (162 pure-logic) → `build` (napi-rs addon) |
| **celestial-php** | `lint-rs` → `build` (ext-php-rs + pure-logic tests, PHP 8.1) |

`lint-rs` and `lint-py`/`lint-ts` always run in parallel with strict scope.

---

## License

Released under **AGPL-3.0**, matching the Swiss Ephemeris it emulates.

The helper functions (`aspects.rs`, `searches.rs`, `vedic.rs`, `geoformat.rs`, `timezone.rs`, `datetime.rs`) are a Rust port of [swephelp](https://github.com/astrorigin/swephelp) by Stanislas Marquis (© 2007–2020, GPL-2).
