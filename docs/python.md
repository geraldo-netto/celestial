# celestial — Python binding

**Package:** `celestial-py` (PyO3)  
**Import:** `import celestial_py as celestial`

## Quick start

```bash
cd bindings/python && pip install maturin && maturin develop
```

See **[docs/python.md](docs/python.md)** for installation, all function signatures,
return types, constants, and Phase 5–8 examples.

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

## Phase 5 — Hellenistic / Persian

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

## Phase 6 — Chinese astrology (Ba Zi)

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

## Phase 7 — Mesoamerican calendars

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

## Phase 8 — Indigenous / Egyptian

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

## Type stubs (`.pyi`)

A full `celestial_py.pyi` stub file ships with the binding at
`bindings/python/python/celestial_py/celestial_py.pyi`.

It covers all 114 exported functions and the following named types:

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
