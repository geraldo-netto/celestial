# celestial — Rust API

`celestial-core` is the pure-Rust computation engine with no C dependencies.

## Quick start

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
pub enum Error {
    Calc(String),      // planetary calculation failure
    Houses(String),    // house system failure
    Eclipse(String),   // eclipse search failure
    RiseTrans(String), // rise/transit failure
    Date(String),      // date conversion failure
}
pub type Result<T> = std::result::Result<T, Error>;

// Pattern match on errors
match calc_ut(jd, SUN, FLG_BUILTIN) {
    Ok(pos)               => println!("lon={:.4}", pos.lon),
    Err(Error::Calc(msg)) => eprintln!("calculation failed: {msg}"),
    Err(e)                => eprintln!("error: {e}"),
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
