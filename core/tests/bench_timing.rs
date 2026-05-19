//! Timing tests — run with `cargo test --release bench_ -- --nocapture`
//! These are NOT statistical benchmarks; they give ballpark numbers
//! and catch gross performance regressions.

use crate::body::Calendar;

use celestial_core::body::{Body, CalcFlags, HouseSystem, SiderealMode};
use celestial_core::*;
use std::time::Instant;

const J2000: f64 = 2_451_545.0;
const N: u32 = 10_000;

fn bench<F: Fn()>(name: &str, n: u32, f: F) {
    // Warmup
    for _ in 0..10 {
        f();
    }
    let t = Instant::now();
    for _ in 0..n {
        f();
    }
    let elapsed = t.elapsed();
    let per_call = elapsed / n;
    println!(
        "  {name:<40} {:>8.2} µs/call  ({} calls)",
        per_call.as_secs_f64() * 1e6,
        n
    );
}

#[test]
fn bench_planet_positions() {
    println!("\n=== Planet positions (geocentric, CalcFlags::BUILTIN) ===");
    let bodies = [
        (Body::SUN, "Sun"),
        (Body::MOON, "Moon"),
        (Body::MERCURY, "Mercury"),
        (Body::VENUS, "Venus"),
        (Body::MARS, "Mars"),
        (Body::JUPITER, "Jupiter"),
        (Body::SATURN, "Saturn"),
    ];
    for (body, name) in bodies {
        bench(name, N, || {
            let _ = calc_ut(JulianDay::new(J2000), body, CalcFlags::BUILTIN);
        });
    }
    bench("Mars + CalcFlags::SPEED", N, || {
        let _ = calc_ut(JulianDay::new(J2000), Body::MARS, CalcFlags::BUILTIN | CalcFlags::SPEED);
    });
    bench("Mars + CalcFlags::HELIOCENTRIC", N, || {
        let _ = calc_ut(
            JulianDay::new(J2000),
            Body::MARS,
            CalcFlags::BUILTIN | CalcFlags::HELIOCENTRIC,
        );
    });
    set_sid_mode(SiderealMode::LAHIRI, 0.0, 0.0);
    bench("Moon + CalcFlags::SIDEREAL", N, || {
        let _ = calc_ut(JulianDay::new(J2000), Body::MOON, CalcFlags::BUILTIN | CalcFlags::SIDEREAL);
    });
}

#[test]
fn bench_full_chart() {
    println!("\n=== Full chart (10 bodies) ===");
    let bodies = [
        Body::SUN,
        Body::MOON,
        Body::MERCURY,
        Body::VENUS,
        Body::MARS,
        Body::JUPITER,
        Body::SATURN,
        Body::URANUS,
        Body::NEPTUNE,
        Body::MEAN_NODE,
    ];
    bench("10 bodies at once", 1000, || {
        for &b in &bodies {
            let _ = calc_ut(JulianDay::new(J2000), b, CalcFlags::BUILTIN);
        }
    });
}

#[test]
fn bench_houses() {
    println!("\n=== House calculations ===");
    let systems: &[(HouseSystem, &str)] = &[
        (HouseSystem::PLACIDUS, "Placidus"),
        (HouseSystem::KOCH, "Koch"),
        (HouseSystem::EQUAL, "Equal"),
        (HouseSystem::WHOLE_SIGN, "Whole Sign"),
        (HouseSystem::PORPHYRY, "Porphyrius"),
        (HouseSystem::REGIOMONTANUS, "Regiomontanus"),
    ];
    for &(sys, name) in systems {
        bench(name, N, || {
            let _ = houses(JulianDay::new(J2000), Latitude::new(48.85), Longitude::new(2.35), sys);
        });
    }
}

#[test]
fn bench_calendar_and_time() {
    println!("\n=== Calendar & time functions ===");
    bench("julday", N, || {
        let _ = julday(2025, 3, 20, 12.0, Calendar::Gregorian);
    });
    bench("revjul", N, || {
        let _ = revjul(J2000, Calendar::Gregorian);
    });
    bench("deltat", N, || {
        let _ = deltat(J2000);
    });
    bench("sidtime", N, || {
        let _ = sidtime(J2000);
    });
    bench("nutation", N, || {
        let _ = nutation(J2000);
    });
    bench("mean_obliquity", N, || {
        let _ = mean_obliquity(J2000);
    });
    set_sid_mode(SiderealMode::LAHIRI, 0.0, 0.0);
    bench("ayanamsa (Lahiri)", N, || {
        let _ = ayanamsa(J2000);
    });
}

#[test]
fn bench_rise_set() {
    println!("\n=== Rise / set / transit (Paris) ===");
    let geo = [2.35_f64, 48.85, 0.0];
    bench("Sun rise", 500, || {
        let _ = rise_trans(
            J2000,
            Body::SUN,
            None,
            CalcFlags::BUILTIN,
            CALC_RISE,
            geo,
            0.0,
            0.0,
        );
    });
    bench("Moon rise", 500, || {
        let _ = rise_trans(
            J2000,
            Body::MOON,
            None,
            CalcFlags::BUILTIN,
            CALC_RISE,
            geo,
            0.0,
            0.0,
        );
    });
    bench("Sun transit", 500, || {
        let _ = rise_trans(
            J2000,
            Body::SUN,
            None,
            CalcFlags::BUILTIN,
            CALC_MTRANSIT,
            geo,
            0.0,
            0.0,
        );
    });
}

#[cfg(feature = "calendar-traditions")]
#[test]
fn bench_sabbats_esbats() {
    println!("\n=== High-level: Sabbats & Esbats ===");
    bench("sabbats_for_year(2025)", 200, || {
        let _ = sabbats_for_year(2025);
    });
    bench("esbats_for_year(2025)", 200, || {
        let _ = esbats_for_year(2025);
    });
}

#[test]
fn bench_eclipses() {
    println!("\n=== Eclipse search (slow — 50 calls) ===");
    bench("next solar eclipse", 50, || {
        let _ = sol_eclipse_when_glob(J2000, CalcFlags::BUILTIN, 0, false);
    });
    bench("next lunar eclipse", 50, || {
        let _ = lun_eclipse_when(J2000, CalcFlags::BUILTIN, 0, false);
    });
}
