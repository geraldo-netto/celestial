# celestial — Rust API

`celestial-core` is the pure-Rust computation engine with no C dependencies.

---

## Quick start

```toml
[dependencies]
celestial-core = { path = "core" }
```

See **[docs/rust.md](docs/rust.md)** for the full API reference, extended examples,
and a complete chart calculation walkthrough.

---

## Add as dependency

```toml
[dependencies]
celestial-core = { path = "../core" }
```

The public API is a single flat namespace:

```rust
use celestial_core::*;
```

---

## Error handling

```rust
// Error is #[non_exhaustive] — always include _ in match arms
pub enum Error {
    // Structured variants (carry typed fields)
    BodyNotImplemented { body: i32 },
    StarNotFound       { name: String },
    PhaseNotFound      { phase: String, from_jd: f64 },
    NoEclipseFound     { from_jd: f64 },
    CircumpolarBody    { body: i32, lat: f64 },
    HouseSystemFailed  { system: u8, lat: f64 },
    // Legacy string variants (backward-compatible)
    Calc(String), Houses(String), Eclipse(String), RiseTrans(String), Date(String),
}
pub type Result<T> = std::result::Result<T, Error>;

// All variants impl Display — e.to_string() always works
match calc_ut(jd, Body::SUN, CalcFlags::BUILTIN) {
    Ok(pos) => println!("lon={:.4}", pos.lon),
    Err(Error::StarNotFound { name }) => eprintln!("star '{name}' not found"),
    Err(Error::BodyNotImplemented { body }) => eprintln!("body {body} unsupported"),
    Err(e) => eprintln!("error: {e}"),  // _ catch-all required by #[non_exhaustive]
}
```

---

## Planetary positions

```rust
use celestial_core::*;

let jd = julday(2025, 3, 20, 9.0, GREG_CAL);

// Single body (Universal Time input)
let sun = calc_ut(jd, SUN, FLG_BUILTIN | FLG_SPEED)?;
println!("Sun  lon={:.4}°  dist={:.6} AU  speed={:.4}°/d",
    sun.lon, sun.dist, sun.speed_lon);

// All main bodies
for body in [SUN, MOON, MERCURY, VENUS, MARS, JUPITER, SATURN, URANUS, NEPTUNE] {
    let p = calc_ut(jd, body, FLG_BUILTIN)?;
    println!("  {:<10}  {:>10.4}°", planet_name(body), p.lon);
}

// Parallel multi-body (uses rayon, scales with CPU cores)
let planets = [SUN, MOON, MERCURY, VENUS, MARS, JUPITER, SATURN, URANUS, NEPTUNE, PLUTO, MEAN_NODE, CHIRON];
let results = calc_many(jd, &planets, FLG_BUILTIN | FLG_SPEED)?;
// results[i] == calc(jd, planets[i], flags) for all i

// Fixed stars
let aldebaran = fixstar_ut("Aldebaran", jd, FLG_BUILTIN)?;
let mag        = fixstar_mag("Aldebaran")?;
println!("Aldebaran  lon={:.4}°  mag={:.2}", aldebaran.lon, mag);

// Nutation (IAU 2000B, 77 terms)
let nut = nutation(jd, FLG_BUILTIN)?;
println!("Δψ={:.6}°  Δε={:.6}°  ε_true={:.6}°", nut.dpsi, nut.deps, nut.eps_true);
```

---

## Houses

```rust
// Placidus house cusps
let h = houses(jd, 48.85, 2.35, b'P')?;
println!("ASC={:.2}°  MC={:.2}°", h.ascmc[0], h.ascmc[1]);
// h.cusps[1..=12] are the twelve house cusps
// h.ascmc: [ASC, MC, ARMC, Vertex, Equatorial ASC, …, Polar ASC]

// With flags (e.g. sidereal, topocentric)
let h_sid = houses_ex(jd, FLG_SIDEREAL, 48.85, 2.35, b'P')?;

// House position of a body
let body_house = house_pos(h.ascmc[2], 48.85, nut.eps_true, b'P', sun.lon)?;
```

**House systems:** `b'P'` Placidus · `b'K'` Koch · `b'E'` Equal · `b'W'` Whole-Sign · `b'O'` Porphyry · `b'R'` Regiomontanus · `b'C'` Campanus · `b'M'` Morinus

---

## Sidereal positions

```rust
// Activate Lahiri ayanamsa
set_sid_mode(SIDM_LAHIRI, 0.0, 0.0);

// Calculate sidereal position
let mars = calc_ut(jd, MARS, FLG_BUILTIN | FLG_SIDEREAL)?;
println!("Mars (Lahiri) = {:.4}°", mars.lon);

// Current ayanamsa value
let ayan = ayanamsa_ut(jd);
println!("Lahiri ayanamsa = {:.4}°", ayan);
```

---

## Builder API

### `CalcOptions` — unified calculation

```rust
use celestial_core::*;

// Single body
let sun = CalcOptions::ut(jd, CalcFlags::BUILTIN | CalcFlags::SPEED)
    .body(Body::SUN)
    .get()?;

// Multiple bodies — Auto strategy (≤2 sequential, >2 parallel)
let results = CalcOptions::ut(jd, CalcFlags::BUILTIN)
    .bodies(&[Body::SUN, Body::MOON, Body::MERCURY, Body::VENUS, Body::MARS])
    .get_many();

// Explicit strategy
let results = CalcOptions::ut(jd, CalcFlags::BUILTIN)
    .strategy(CalcStrategy::Parallel)
    .bodies(&[Body::SUN, Body::MOON, Body::MERCURY])
    .get_many();

// TT (Terrestrial Time) input — for Meeus examples
let moon_tt = CalcOptions::tt(jde, CalcFlags::BUILTIN).body(Body::MOON).get()?;
```

**`CalcStrategy` variants:** `Sequential` · `Parallel` · `Auto` (default)

### `RiseTransOptions`

```rust
// Replaces the 8-argument rise_trans() free function
let rise = RiseTransOptions::new(jd, Body::MOON, [2.35, 48.85, 35.0])
    .event(1)                    // 1 = CALC_RISE
    .atmosphere(1013.25, 15.0)
    .search()?;
println!("Moon rises at JD {:.4}", rise.tret);

// For a fixed star
let aldebaran = RiseTransOptions::new(jd, Body::SUN, [2.35, 48.85, 35.0])
    .star("Aldebaran")
    .event(1)
    .search()?;
```

### `SearchOptions`

```rust
// Aspect to house cusp
let hit = SearchOptions::new(Body::SATURN, jd)
    .aspect(90.0)
    .cusp(10, lat, lon, HouseSystem::PLACIDUS)
    .search_cusp();

// Natal angle transits
let jd_mc = SearchOptions::new(Body::SATURN, jd_start)
    .natal_chart(jd_natal, lat, lon, HouseSystem::PLACIDUS)
    .search_mc_transit()?;
let jd_asc = SearchOptions::new(Body::SATURN, jd_start)
    .natal_chart(jd_natal, lat, lon, HouseSystem::PLACIDUS)
    .search_asc_transit()?;

// Backward search
let jd_past = SearchOptions::new(Body::JUPITER, jd)
    .natal_chart(jd_natal, lat, lon, HouseSystem::PLACIDUS)
    .backward(true)
    .search_mc_transit()?;
```

### `AspectOrbs`

```rust
// Replaces match_aspect3 / match_aspect4
let m = AspectOrbs::new(2.0, 1.5)  // applying_orb, separating_orb
    .check(pos0, speed0, pos1, speed1, 120.0);  // trine

if m.matched {
    println!("Trine  orb={:.2}°  {}", m.diff.abs(),
        if m.diff < 0.0 { "applying" } else { "separating" });
}
```

---

## Celtic calendar

```rust
// All 8 sabbats for 2025
for s in sabbats_for_year(2025)? {
    let d = revjul(s.jd, GREG_CAL);
    println!("{:<14}  {:04}-{:02}-{:02}  {:.0}°",
        s.name, d.year, d.month, d.day, s.kind.solar_longitude());
}

// Next sabbat from a date
let next = next_sabbat(julday(2025, 6, 1, 0.0, GREG_CAL))?;
println!("Next: {}", next.name);

// Named full moons
for m in esbats_for_year(2025)? {
    let d = revjul(m.jd, GREG_CAL);
    println!("{:<18}  {:04}-{:02}-{:02}", m.display_name, d.year, d.month, d.day);
}
```

---

## Moon phases

```rust
let jd = jdnow();

let phase = moon_phase(jd);                // → MoonPhase variant
let illum = moon_illumination(jd);         // 0.0–1.0
let elong = moon_elongation(jd);           // 0°–360°

let info  = moon_phase_info(jd);
println!("Phase: {}  Illumination: {:.1}%  Age: {:.1} days",
    info.phase_name, info.illumination * 100.0, info.age_days);
println!("Next: {} at JD {:.2}", info.next_phase_name, info.next_phase_jd);

// Next principal phases
let new_moon  = next_new_moon(jd);
let full_moon = next_full_moon_phase(jd);

// All phases in a month
let phases = moon_phases_for_month(2025, 4);
for p in &phases {
    let d = revjul(p.jd, GREG_CAL);
    println!("{:<18}  {:04}-{:02}-{:02}", p.name, d.year, d.month, d.day);
}
```

---

## Sefirat HaOmer

```rust
let jd = jdnow();

if let Some(day) = omer_from_jd(jd) {
    println!("Tonight is Omer day {} — {}", day.day, day.hebrew_text);
    if day.is_lag_baomer { println!("  Lag Ba'Omer!"); }
}

// Full 49-day schedule
for d in omer_days(5785) {
    println!("Day {:2}: {} sheb'{}", d.day, d.week_sefirah, d.day_sefirah);
}
```

---

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

    // Phase 5 — Hellenistic dignities
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

## Crossings & eclipses

```rust
let jd = julday(2025, 1, 1, 0.0, GREG_CAL);

// Next vernal equinox
let ostara = solcross_ut(0.0, jd, FLG_BUILTIN)?;
let d = revjul(ostara, GREG_CAL);
println!("Ostara {:04}-{:02}-{:02}", d.year, d.month, d.day);

// Next solar eclipse
let ecl = sol_eclipse_when_glob(jd, FLG_BUILTIN, 0, false)?;
let d   = revjul(ecl.tret[0], GREG_CAL);
println!("Solar eclipse {:04}-{:02}-{:02}", d.year, d.month, d.day);

// Retrograde/direct stations
let stations = retrograde_station_ut(MARS, jd, FLG_BUILTIN)?;
println!("Mars retrograde: JD {:.2}", stations.retrograde);
println!("Mars direct:     JD {:.2}", stations.direct);

// Sign ingress
let (jd_ingress, sign) = sign_ingress_ut(SATURN, jd, FLG_BUILTIN, false)?;
println!("Saturn enters {}: JD {jd_ingress:.2}", zodiac_sign_name(sign));
```

---

## Serializable output types

`ChartAspect`, `Stations`, `ArabicPart`, and `DashaLevel` derive
`serde::Serialize` and `serde::Deserialize`:

```rust
use serde_json;

let aspects = calc_chart_aspects(&positions, MAJOR_ASPECTS, 8.0);
let json    = serde_json::to_string(&aspects)?;  // works directly
```

Add `serde` to your `Cargo.toml` to use this:

```toml
[dependencies]
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
```
