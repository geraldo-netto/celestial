# celestial — Python binding

**Package:** `celestial-py` (PyO3)  
**Import:** `import celestial_py as celestial`

## Quick start

```bash
cd bindings/python && pip install maturin && maturin develop
```

This guide covers installation, all function signatures, return types, constants,
and tradition-specific examples.

---

> **Note on Rust builder structs:** `CalcOptions`, `RiseTransOptions`,
> `SearchOptions`, and `AspectOrbs` are Rust-only builder structs and are
> not currently exposed in the Python binding. The Python binding provides
> the underlying functions directly: `calc_ut`, `calc_many`, `rise_trans`,
> `next_aspect_cusp`, `match_aspect3`, `match_aspect4`, etc.

## Installation

```bash
cd bindings/python
pip install maturin
maturin develop          # development build
maturin build --release  # production wheel
pip install target/wheels/celestial_py-*.whl
```

---

## Constants

### Body indices

| Constant | Value |
|---|---|
| `celestial.SUN` | 0 |
| `celestial.MOON` | 1 |
| `celestial.MERCURY` | 2 |
| `celestial.VENUS` | 3 |
| `celestial.MARS` | 4 |
| `celestial.JUPITER` | 5 |
| `celestial.SATURN` | 6 |
| `celestial.URANUS` | 7 |
| `celestial.NEPTUNE` | 8 |
| `celestial.PLUTO` | 9 |
| `celestial.MEAN_NODE` | 10 |
| `celestial.TRUE_NODE` | 11 |
| `celestial.CHIRON` | 15 |

### Calculation flags

| Constant | Description |
|---|---|
| `celestial.FLG_BUILTIN` | Use built-in ephemeris (always include) |
| `celestial.FLG_SPEED` | Include speed in result |
| `celestial.FLG_SIDEREAL` | Sidereal positions |
| `celestial.FLG_EQUATORIAL` | Equatorial coordinates |
| `celestial.FLG_HELCTR` | Heliocentric |

### Sidereal modes

| Constant | Mode |
|---|---|
| `celestial.SIDM_LAHIRI` | Lahiri (official Indian) |
| `celestial.SIDM_FAGAN_BRADLEY` | Fagan-Bradley |
| `celestial.SIDM_RAMAN` | B.V. Raman |
| `celestial.SIDM_KRISHNAMURTI` | Krishnamurti |

---

## Core functions

### Time

```python
# Calendar → Julian Day
jd = celestial.julday(2025, 3, 20, 9.0, celestial.GREG_CAL)

# Julian Day → calendar date
date = celestial.revjul(jd, celestial.GREG_CAL)
print(f"{date.year}-{date.month:02d}-{date.day:02d}")

# Current JD
now = celestial.jdnow()
```

### Planetary positions

```python
# Single planet
pos = celestial.calc_ut(jd, celestial.SUN, celestial.FLG_BUILTIN | celestial.FLG_SPEED)
print(f"lon={pos.lon:.4f}°  lat={pos.lat:.4f}°  dist={pos.dist:.6f} AU")
print(f"speed={pos.speed_lon:.4f}°/day")

# Multiple planets in parallel (returns list in same order as input)
planets = [celestial.SUN, celestial.MOON, celestial.MERCURY, celestial.VENUS,
           celestial.MARS, celestial.JUPITER, celestial.SATURN,
           celestial.URANUS, celestial.NEPTUNE, celestial.PLUTO,
           celestial.MEAN_NODE, celestial.CHIRON]
results = celestial.calc_many(jd, planets, celestial.FLG_BUILTIN | celestial.FLG_SPEED)
# results[i] corresponds to planets[i]

# Nutation (IAU 2000B)
nut = celestial.nutation(jd, celestial.FLG_BUILTIN)
print(f"dpsi={nut.dpsi:.6f}°  deps={nut.deps:.6f}°  eps_true={nut.eps_true:.6f}°")

# Fixed star
star = celestial.fixstar_ut("Aldebaran", jd, celestial.FLG_BUILTIN)
mag  = celestial.fixstar_mag("Aldebaran")
```

### Houses

```python
# Standard house calculation
h = celestial.houses_ex(jd, 0, 48.85, 2.35, ord('P'))
print(f"ASC={h.ascmc[0]:.2f}°  MC={h.ascmc[1]:.2f}°")
# h.cusps[0..12], h.ascmc[0..8]

# With sidereal flag
h_sid = celestial.houses_ex(jd, celestial.FLG_SIDEREAL, 48.85, 2.35, ord('P'))
```

**House system codes:** `ord('P')` Placidus · `ord('K')` Koch · `ord('E')` Equal · `ord('W')` Whole-Sign · `ord('O')` Porphyry · `ord('R')` Regiomontanus

### Sidereal positions

```python
celestial.set_sid_mode(celestial.SIDM_LAHIRI, 0.0, 0.0)
moon = celestial.calc_ut(jd, celestial.MOON, celestial.FLG_BUILTIN | celestial.FLG_SIDEREAL)
print(f"Moon (Lahiri) = {moon.lon:.4f}°")

ayan = celestial.ayanamsa_ut(jd)
print(f"Lahiri ayanamsa = {ayan:.4f}°")
```

---

## Moon phases

```python
phase      = celestial.moon_phase(jd)           # MoonPhase object
illumination = celestial.moon_illumination(jd)  # 0.0–1.0
elongation   = celestial.moon_elongation(jd)    # 0°–360°

info = celestial.moon_phase_info(jd)
print(f"{info.phase_name}  {info.illumination*100:.1f}%  age={info.age_days:.1f}d")
print(f"Next: {info.next_phase_name}")

# Principal phases
new_moon  = celestial.next_new_moon(jd)
full_moon = celestial.next_full_moon_phase(jd)

# All phases in April 2025
phases = celestial.moon_phases_for_month(2025, 4)
for p in phases:
    d = celestial.revjul(p.jd, celestial.GREG_CAL)
    print(f"{p.name:<18} {d.year}-{d.month:02d}-{d.day:02d}")
```

---

## Celtic calendar

```python
for s in celestial.sabbats_for_year(2025):
    d = celestial.revjul(s['jd'], celestial.GREG_CAL)
    print(f"{s['name']:<14}  {d.year}-{d.month:02d}-{d.day:02d}")

for m in celestial.esbats_for_year(2025):
    d = celestial.revjul(m['jd'], celestial.GREG_CAL)
    print(f"{m['display_name']:<18}  {d.year}-{d.month:02d}-{d.day:02d}")
```

---

## Vedic / Jyotish

```python
celestial.set_sid_mode(celestial.SIDM_LAHIRI, 0.0, 0.0)
moon = celestial.calc_ut(jd, celestial.MOON, celestial.FLG_BUILTIN | celestial.FLG_SIDEREAL)

nak, pada = celestial.long_to_nakshatra(moon.lon)
print(f"Nakshatra: {celestial.nakshatra_name(nak)}, pada {pada+1}")

rasi    = celestial.long_to_rasi(moon.lon)       # 0–11
navamsa = celestial.long_to_navamsa(moon.lon)    # 0–11

dashas = celestial.vimshottari_dasha(jd, moon.lon, 120.0)
for d in dashas[:3]:
    print(f"{d['planet_name']} dasha: {d['years']:.1f} years")

och = celestial.ochchabala(celestial.SUN, pos.lon)  # 0–60
```

---

## Hellenistic / Persian

```python
pos = celestial.calc_ut(jd, celestial.SUN, celestial.FLG_BUILTIN)
h   = celestial.houses_ex(jd, 0, 48.85, 2.35, ord('P'))

# Day or night chart
is_day = celestial.is_day_chart(pos.lon, list(h.cusps))

# Egyptian terms ruler (planet index)
ruler = celestial.egyptian_terms_ruler(pos.lon)

# Chaldean decan ruler
decan = celestial.decan_ruler(pos.lon)

# Triplicity rulers
day_r, night_r, part_r = celestial.triplicity_rulers(pos.lon)

# Full dignity
dignity, score = celestial.full_dignity(celestial.SUN, pos.lon, is_day)
print(f"Sun: {dignity} (score {score})")

# Almuten
body, alm_score = celestial.almuten(pos.lon, is_day)

# Firdaria periods
periods = celestial.firdaria(jd, is_day, 75.0)
for p in periods[:3]:
    print(f"Major: {p['major_lord']}  Minor: {p['minor_lord']}  "
          f"Start: JD {p['start_jd']:.1f}")

# Annual profection (age 35)
house_num, prof_lon = celestial.annual_profection(list(h.cusps), 35)
print(f"Age 35 → House {house_num} ({prof_lon:.2f}°)")
```

---

## Chinese astrology (Ba Zi)

```python
pos = celestial.calc_ut(jd, celestial.SUN, celestial.FLG_BUILTIN)
pillars = celestial.four_pillars(jd, 9.0, pos.lon)  # 9.0 = 9:00 UT

for i, pillar in enumerate(pillars):
    label = ["Year", "Month", "Day", "Hour"][i]
    print(f"{label}: {pillar['stem_name']} {pillar['branch_name']} "
          f"({pillar['animal']}, {pillar['stem_element']})")

# Solar term
cur_idx, deg_into, next_idx, deg_to_next = celestial.solar_term_position(pos.lon)
terms = celestial.SOLAR_TERMS  # list of (longitude, pinyin, english)
print(f"Current term: {terms[cur_idx][1]} ({deg_into:.1f}° in)")
print(f"Next term: {terms[next_idx][1]} in {deg_to_next:.1f}°")
```

---

## Mesoamerican calendars

```python
# Aztec Tonalpohualli (260-day)
trecena, sign_idx, nahuatl, english = celestial.tonalpohualli(jd)
print(f"Tonalpohualli: {trecena} {nahuatl} ({english})")

# Aztec Xiuhpohualli (365-day solar year)
month_idx, day, month_name, month_en = celestial.xiuhpohualli(jd)
print(f"Xiuhpohualli: {day} {month_name}")

# Maya Tzolkin (260-day)
trecena, sign_idx, mayan, english = celestial.tzolkin(jd)
print(f"Tzolkin: {trecena} {mayan} ({english})")

# Maya Haab (365-day)
month_idx, day, month_name = celestial.haab(jd)
print(f"Haab: {day} {month_name}")

# Calendar Round
tz_t, tz_s, haab_d, haab_m = celestial.calendar_round(jd)

# GMT correlation constant
print(celestial.GMT_CORRELATION)  # 584283
```

---

## Indigenous / Egyptian

```python
pos = celestial.calc_ut(jd, celestial.SUN, celestial.FLG_BUILTIN)

# Medicine Wheel birth totem (Sun Bear / Wabun Wind synthesis)
animal, element, clan, season = celestial.medicine_wheel_totem(pos.lon)
print(f"Totem: {animal} — {element} element, {clan} clan, {season}")

# Egyptian decan
decan_idx, decan_name, rising_star = celestial.egyptian_decan(pos.lon)
print(f"Decan {decan_idx+1}: {decan_name} (rising star: {rising_star})")
```


---

## Sabbats & esbats

```python
# All eight sabbats for a year
sabbats = celestial.sabbats_for_year(2025)
for name, jd in sabbats:
    print(f"{name}: JD {jd:.4f}")

# Next sabbat from a given JD
name, jd = celestial.next_sabbat(2_451_545.0)

# Specific sabbat JD (kind 0-7: Samhain, Yule, Imbolc, Ostara,
#                              Beltane, Litha, Lughnasadh, Mabon)
jd_yule = celestial.sabbat_jd(2025, 1)

# All esbats (named full moons) for a year
esbats = celestial.esbats_for_year(2025)
for name, jd in esbats:
    print(f"{name}: JD {jd:.4f}")

# Next esbat from a given JD
name, jd = celestial.next_esbat(2_451_545.0)
```

---

## Angle transits

```python
# Transit to natal angles (ic / asc / dsc added alongside existing mc_transit_ut)
jd_mc  = celestial.mc_transit_ut(planet, jd_natal, jd_start, lat, lon, hsys, flags, False)
jd_ic  = celestial.ic_transit_ut(planet, jd_natal, jd_start, lat, lon, hsys, flags, False)
jd_asc = celestial.asc_transit_ut(planet, jd_natal, jd_start, lat, lon, hsys, flags, False)
jd_dsc = celestial.dsc_transit_ut(planet, jd_natal, jd_start, lat, lon, hsys, flags, False)

# Aspect to a house cusp — returns (jd, orb) or None
hit = celestial.next_aspect_cusp(planet, 90.0, 10, jd_start,
                                   lat, lon, hsys, False, flags)
```

---

## Chart analysis

```python
# All aspects in a chart with auto orbs
positions = [(celestial.SUN, sun_lon, sun_speed), (celestial.MOON, moon_lon, moon_speed), ...]
aspects = celestial.calc_chart_aspects_auto(positions)
for body1, body2, aspect, orb, applying in aspects:
    print(f"{body1}/{body2}: {aspect:.0f}° orb {orb:.2f}° {'app' if applying else 'sep'}")

# Custom aspect list and orb
aspects = celestial.calc_chart_aspects(positions, [0.0, 60.0, 90.0, 120.0, 180.0], 8.0)

# Midpoint table — returns [(body1, body2, midpoint_lon), ...]
positions_2d = [(celestial.SUN, sun_lon), (celestial.MOON, moon_lon), ...]
table = celestial.midpoint_table(positions_2d, 2.0)

# Secondary progressions
pos_list, cusps = celestial.secondary_progressions(
    jd_natal, years=35.0, bodies=[celestial.SUN, celestial.MOON],
    lat=48.85, lon=2.35, hsys=ord('P'), flags=celestial.FLG_BUILTIN
)

# Solar arc directions — returns (arc_degrees, directed_positions, mc_arc)
arc, directed, mc_arc = celestial.solar_arc_directions(
    jd_natal, years=35.0,
    natal_positions=[(celestial.SUN, sun_lon), (celestial.MOON, moon_lon)],
    natal_mc=mc, flags=celestial.FLG_BUILTIN
)

# Monthly profection (age in years + months)
house, degree = celestial.monthly_profection(cusps, age_years=35, age_months=6)
```


---

## Type stubs (`.pyi`)

A full `celestial_py.pyi` stub file ships with the binding at
`bindings/python/python/celestial_py/celestial_py.pyi`.

It covers all 198 exported functions and the following named types:

| Class | Fields |
|---|---|
| `PlanetPos` | `lon`, `lat`, `dist`, `speed_lon`, `speed_lat`, `speed_dist`, `ret_flags` |
| `HouseResult` | `cusps: list[float]` (13 elements), `ascmc: list[float]` (8 elements) |
| `NutationResult` | `dpsi`, `deps`, `eps_true` |
| `RiseTransResult` | `tret`, `trise`, `tset`, `ttransit` |
| `BaZiPillar` | `stem_name`, `branch_name`, `animal`, `stem_element`, `branch_element`, `yang` |
| `FirdariaPeriod` | `major_lord`, `minor_lord`, `start`, `end`, `years` |
| `MoonPhaseInfo` | `phase_name`, `illumination`, `elongation`, `age_days`, `next_phase_name`, … |
| `JewishHoliday` | `name`, `jd`, `days` |

IDEs (VS Code, PyCharm, etc.) will pick this up automatically when the package is installed.
You can also use it directly for type-checking:

```bash
# mypy
mypy --ignore-missing-imports your_script.py

# pyright
pyright your_script.py
```

---

## Parallel calculation

The Python binding exposes `calc_many` for parallel multi-body computation:

```python
import celestial_py as celestial

planets = [
    celestial.SUN, celestial.MOON, celestial.MERCURY, celestial.VENUS,
    celestial.MARS, celestial.JUPITER, celestial.SATURN,
    celestial.URANUS, celestial.NEPTUNE, celestial.PLUTO,
    celestial.MEAN_NODE, celestial.CHIRON,
]
# All 12 bodies computed in parallel — order preserved
results = celestial.calc_many(jd, planets, celestial.FLG_BUILTIN | celestial.FLG_SPEED)
sun = results[0]   # corresponds to planets[0] = SUN
moon = results[1]  # corresponds to planets[1] = MOON
```

---

## Error handling

All functions raise `Exception` with a descriptive message on failure:

```python
try:
    pos = celestial.calc_ut(jd, celestial.SUN, celestial.FLG_BUILTIN)
except Exception as e:
    print(f"Calculation failed: {e}")
```

---

## Type reference

| Function | Return type |
|---|---|
| `calc_ut` / `calc_many` | `PlanetPos` with fields: `lon lat dist speed_lon speed_lat speed_dist` |
| `houses_ex` | `HouseResult` with fields: `cusps` (list[13]) `ascmc` (list[8]) |
| `nutation` | `NutationResult` with fields: `dpsi deps eps_true` |
| `moon_phase_info` | `MoonPhaseInfo` — see Moon phases section |
| `four_pillars` | `list[dict]` — `stem_name branch_name animal stem_element branch_element yang` |
| `firdaria` | `list[dict]` — `major_lord minor_lord start_jd end_jd years` |
| `full_dignity` | `(str, int)` — dignity name, score |
| `almuten` | `(int, int)` — body index, score |
| `is_day_chart` | `bool` |
| `tonalpohualli` | `(int, int, str, str)` — trecena, sign_idx, nahuatl name, english |
| `medicine_wheel_totem` | `(str, str, str, str)` — animal, element, clan, season |
| `egyptian_decan` | `(int, str, str)` — index, name, rising star |

---

## Legacy / compatibility aliases

These SwissEph-compatible names are available alongside their canonical equivalents:

```python
from celestial_py import (
    degnorm,          # → norm_deg()
    difdeg2n,         # → diff_deg_signed()
    get_ayanamsa,     # → ayanamsa()
    get_ayanamsa_name,# → ayanamsa_name()
    next_sabbat_name, # → next_sabbat() — returns name only
    next_full_moon,   # → next_full_moon_after()
    solcross_ut,      # finds when Sun crosses a given degree
)
```

## Recent additions — 19 new functions

### ISO 8601 week

```python
from celestial_py import iso_week, day_of_year, weeks_in_iso_year

year, week = iso_week(2456293.0)        # → (2012, 52) for 2012-12-31
dow  = day_of_year(2024, 3, 15)          # → 75
wks  = weeks_in_iso_year(2020)           # → 53
```

### Maya Long Count

```python
from celestial_py import maya_long_count, maya_long_count_str

bktn, ktn, tun, unl, kin = maya_long_count(2456283.0)   # 2012-12-21 → (13,0,0,0,0)
s = maya_long_count_str(2451545.0)                       # J2000 → "12.19.6.15.2"
```

### Yallop crescent visibility

```python
from celestial_py import yallop_q, best_time_method

# Compute q-value and visibility class at the best time:
jd_best = best_time_method(jd_sunset=2460000.0, jd_moonset=2460000.1)
q, cls  = yallop_q(arcv_deg=10.0, arcl_deg=15.0, sd_arcmin=15.5)
# cls ∈ {'A','B','C','D','E','F'}; 'A' = easily visible, 'F' = not visible
```

### Coptic / Ethiopic (feature: `calendar-traditions`)

```python
from celestial_py import coptic_to_jd, jd_to_coptic, ethiopic_to_jd, jd_to_ethiopic
from celestial_py import is_coptic_leap_year, coptic_month_days

jd         = coptic_to_jd(1740, 1, 1)             # 1 Thout AM 1740
y, m, d    = jd_to_coptic(jd)
is_leap    = is_coptic_leap_year(1739)            # True (1739 mod 4 == 3)
days       = coptic_month_days(1739, 13)          # 6 (leap year epagomenal)

# Ethiopic has the same structure, 276 years offset:
eth_y, eth_m, eth_d = jd_to_ethiopic(jd)
```

### Zoroastrian Fasli (feature: `calendar-traditions`)

```python
from celestial_py import fasli_nowruz_jd, jd_to_fasli

jd_nowruz  = fasli_nowruz_jd(2024)   # JD of astronomical vernal equinox 2024
date       = jd_to_fasli(2460400.0)  # → (fasli_year, month_index, day) or None
# month_index 13 = the 5 Gatha (epagomenal) days
```

### Tibetan Phugpa (feature: `calendar-traditions`)

```python
from celestial_py import losar_jd, tibetan_year_name

jd_losar             = losar_jd(2024)                # 2nd new moon after winter solstice
cycle, yic, el, g, a = tibetan_year_name(2024)
# → (17, 38, "Wood", "Male", "Dragon") — 17th Rabjung cycle, year 38
```

### Vietnamese Âm Lịch

```python
from celestial_py import vietnamese_month_start_jd, vietnamese_chinese_boundary_differs

jd_month_start = vietnamese_month_start_jd(2460000.0)
diverges       = vietnamese_chinese_boundary_differs(2460000.0)
# True on roughly 4% of JDs — when UTC+7 and UTC+8 fall on different civil days
```
