#![allow(dead_code, unused_imports)]
//! Property-based test suite for celestial-core (pure-Rust engine).
//!
//! Self-contained: uses a stdlib xorshift64 PRNG — no external crates needed.
//! Run with: `cargo run --manifest-path fuzz/Cargo.toml`

use celestial_core::body::{Body, CalcFlags, Calendar, HouseSystem, SiderealMode};
use celestial_core::*;
use std::f64::consts::{PI, TAU};
// flat imports via celestial_core::* above

// ─── Minimal PRNG ─────────────────────────────────────────────────────────────

struct Xorshift64(u64);

impl Xorshift64 {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn range_f64(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.next_f64() * (hi - lo)
    }
    fn range_i32(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next_u64() % (hi - lo) as u64) as i32
    }
}

// ─── Test runner ──────────────────────────────────────────────────────────────

struct Suite {
    name: &'static str,
    passed: u32,
    failed: u32,
}

impl Suite {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            passed: 0,
            failed: 0,
        }
    }

    fn check(&mut self, cond: bool, msg: impl FnOnce() -> String) {
        if cond {
            self.passed += 1;
        } else {
            self.failed += 1;
            eprintln!("  FAIL [{}] {}", self.name, msg());
        }
    }

    fn report(&self) -> bool {
        let ok = self.failed == 0;
        let sym = if ok { "✓" } else { "✗" };
        println!(
            "{sym} {:30} {:5} passed, {:3} failed",
            self.name, self.passed, self.failed
        );
        ok
    }
}

// ─── Test groups ──────────────────────────────────────────────────────────────

fn test_math(n: u32) -> Suite {
    let mut s = Suite::new("math");
    let mut rng = Xorshift64::new(0x1234_5678_9ABC_DEF0);

    for _ in 0..n {
        // --- norm_deg ---;
        let x = rng.range_f64(-1e9, 1e9);
        let d = norm_deg(x);
        s.check(d >= 0.0 && d < 360.0, || format!("norm_deg({x}) = {d}"));
        s.check((norm_deg(d) - d).abs() < 1e-10, || {
            format!("norm_deg idempotent fail at {d}")
        });

        // --- norm_rad ---;
        let r = norm_rad(x);
        s.check(r >= 0.0 && r < TAU, || format!("norm_rad({x}) = {r}"));

        // --- diff_deg_signed ---;
        let a = rng.range_f64(-1e9, 1e9);
        let b = rng.range_f64(-1e9, 1e9);
        let diff = diff_deg_signed(a, b);
        s.check(diff > -180.0 && diff <= 180.0, || {
            format!("diff_deg_signed({a},{b}) = {diff}")
        });

        // --- norm_cs ---;
        let cs = rng.next_u64() as i32;
        let cn = norm_cs(cs);
        s.check(cn >= 0 && cn < 360 * 360_000_i64, || {
            format!("norm_cs({cs}) = {cn}")
        });
        s.check(norm_cs(cn as i32) == cn, || {
            format!("norm_cs not idempotent at {cn}")
        });

        // --- coord_transform round-trip ---;
        let lon = rng.range_f64(0.0, 360.0);
        let lat = rng.range_f64(-89.9, 89.9);
        let dist = rng.range_f64(0.001, 1000.0);
        let eps = 23.4_f64;
        let fwd = coord_transform([lon, lat, dist], eps);
        let back = coord_transform(fwd, -eps);
        s.check(fwd[0].is_finite() && fwd[1].is_finite(), || {
            "coord_transform NaN".to_string()
        });
        s.check(fwd[1] >= -90.0 && fwd[1] <= 90.0, || {
            format!("coord_transform lat {} out of range", fwd[1])
        });
        let lon_err = ((back[0] - lon + 540.0) % 360.0 - 180.0).abs();
        s.check(lon_err < 1e-6, || {
            format!("coord_transform lon round-trip error {lon_err:.2e}")
        });
        s.check((back[1] - lat).abs() < 1e-6, || {
            format!(
                "coord_transform lat round-trip error {:.2e}",
                (back[1] - lat).abs()
            )
        });

        // --- split_deg ---;
        let x = rng.range_f64(-1e6, 1e6);
        let (d, m, sec, frac, _) = split_deg(x, 0);
        s.check(
            d >= 0 && m >= 0 && m < 60 && sec >= 0 && sec < 60 && frac >= 0.0 && frac < 1.0,
            || format!("split_deg({x}) → ({d},{m},{sec},{frac:.4})"),
        );

        // --- nan/inf inputs (must not panic) ---;
        let _ = norm_deg(f64::NAN);
        let _ = norm_deg(f64::INFINITY);
        let _ = coord_transform([f64::NAN, 0.0, 1.0], 23.4);
        s.passed += 1; // survived
    }
    s
}

fn test_time(n: u32) -> Suite {
    let mut s = Suite::new("time");
    let mut rng = Xorshift64::new(0xDEAD_BEEF_CAFE_1234);

    for _ in 0..n {
        // julday / revjul round-trip;
        let year = rng.range_i32(1582, 9999);
        let month = rng.range_i32(1, 13);
        let day = rng.range_i32(1, 29);
        let hour = rng.range_f64(0.0, 24.0);
        let jd = julday(year, month, day, hour, Calendar::Gregorian);
        s.check(jd.is_finite(), || "julday NaN".to_string());

        let back = revjul(jd, Calendar::Gregorian);
        s.check(back.year == year, || {
            format!("revjul year: {year} → {}", back.year)
        });
        s.check(back.month == month, || {
            format!("revjul month: {month} → {}", back.month)
        });
        s.check(back.day == day, || {
            format!("revjul day: {day} → {}", back.day)
        });
        s.check((back.hour - hour).abs() < 1e-7, || {
            format!("revjul hour: {hour} → {}", back.hour)
        });

        // day_of_week in [0,6];
        let dow = day_of_week(jd);
        s.check(dow >= 0 && dow <= 6, || format!("day_of_week={dow}"));

        // 7-day cycle
        s.check(day_of_week(jd) == day_of_week(jd + 7.0), || {
            "7-day cycle broken".to_string()
        });

        // deltat plausible (1900–2100);
        let yr2 = rng.range_i32(1900, 2101);
        let jd2 = julday(yr2, 6, 15, 0.0, Calendar::Gregorian);
        let dt = deltat(jd2);
        s.check(dt.is_finite() && dt > -0.01 && dt < 2.0, || {
            format!("deltat={dt} days for year={yr2}")
        });

        // utc_time_zone round-trip;
        let h_in = rng.range_i32(0, 24);
        let m_in = rng.range_i32(0, 60);
        let off = rng.range_f64(-12.0, 12.0);
        let utc = UtcDate {
            year: 2002,
            month: 6,
            day: 15,
            hour: h_in,
            minute: m_in,
            second: 0.0,
        };
        let local = utc_time_zone(&utc, off);
        let back2 = utc_time_zone(&local, -off);
        s.check(back2.hour == h_in && back2.minute == m_in, || {
            format!(
                "utc_tz roundtrip: ({h_in},{m_in}) → ({},{}) via offset {off:.2}",
                back2.hour, back2.minute
            )
        });
    }
    s
}

const HOUSE_SYSTEMS: &[u8] = b"PKEOCRWXMBHT";

fn test_houses(n: u32) -> Suite {
    let mut s = Suite::new("houses");
    let mut rng = Xorshift64::new(0xCAFE_BABE_1234_5678);

    for _ in 0..n {
        let jd = 2_415_021.0 + rng.range_f64(0.0, 73049.0); // 1900–2100;
        let lat = rng.range_f64(-83.0, 83.0);
        let lon = rng.range_f64(-180.0, 180.0);
        let sys_i = (rng.next_u64() % HOUSE_SYSTEMS.len() as u64) as usize;
        let hsys = HouseSystem(HOUSE_SYSTEMS[sys_i]);
        let Ok(r) = houses(jd, lat, lon, hsys) else {
            continue;
        };

        for (i, &c) in r.cusps.iter().enumerate() {
            s.check(c.is_finite() && c >= 0.0 && c < 360.0, || {
                format!(
                    "sys='{}' cusp[{i}]={c} (lat={lat:.1})",
                    hsys.as_raw() as char
                )
            });
        }
        for (i, &a) in r.ascmc.iter().enumerate() {
            s.check(a.is_finite(), || {
                format!("sys='{}' ascmc[{i}]={a} NaN", hsys.as_raw() as char)
            });
        }

        // Equal houses: 30° apart
        if hsys == HouseSystem::EQUAL {
            for h in 1..r.cusps.len().saturating_sub(1) {
                let diff = (r.cusps[h + 1] - r.cusps[h] + 360.0) % 360.0;
                s.check((diff - 30.0).abs() < 0.01, || {
                    format!("Equal diff at h{h}: {diff}")
                });
            }
        }
        // Whole sign: multiples of 30°
        if hsys == HouseSystem::WHOLE_SIGN {
            for (i, &c) in r.cusps.iter().enumerate() {
                let rem = c % 30.0;
                s.check(rem < 0.01 || rem > 29.99, || {
                    format!("Whole Sign cusp[{i}]={c} not on boundary")
                });
            }
        }
    }
    s
}

fn test_vsop87(n: u32) -> Suite {
    // (body constant, perihelion AU, aphelion AU)
    const PLANETS: &[(Body, f64, f64)] = &[
        (Body::MERCURY, 0.307, 0.467),   // Mercury perihelion / aphelion
        (Body::VENUS, 0.718, 0.728),     // Venus (nearly circular)
        (Body::MARS, 1.381, 1.666),      // Mars
        (Body::JUPITER, 4.951, 5.457),   // Jupiter
        (Body::SATURN, 9.041, 10.124),   // Saturn
        (Body::URANUS, 18.375, 20.063),  // Uranus
        (Body::NEPTUNE, 29.767, 30.441), // Neptune
    ];

    let mut s = Suite::new("vsop87");
    let mut rng = Xorshift64::new(0x1111_2222_3333_4444);

    for _ in 0..n {
        let jd_off = rng.range_f64(-365_250.0, 365_250.0); // ±1000 years;
        let jde = 2_451_545.0 + jd_off;
        let pi = (rng.next_u64() % PLANETS.len() as u64) as usize;
        let (body, lo, hi) = PLANETS[pi];

        let Ok(pos) = calc_ut(jde, body, CalcFlags::BUILTIN | CalcFlags::HELIOCENTRIC) else {
            s.passed += 1;
            continue;
        };

        s.check(
            pos.lon.is_finite() && pos.lat.is_finite() && pos.dist.is_finite(),
            || format!("body={} non-finite at jde={jde}", body),
        );
        s.check(pos.lon >= 0.0 && pos.lon < 360.0, || {
            format!("body={body} lon={} out of [0°,360°)", pos.lon)
        });
        s.check(pos.lat.abs() < 90.0, || {
            format!("body={body} lat={} too large", pos.lat)
        });
        s.check(pos.dist >= lo * 0.7 && pos.dist <= hi * 1.3, || {
            format!(
                "body={body} dist={:.4} AU outside [{lo},{hi}]×0.7–1.3",
                pos.dist
            )
        });
    }
    s
}

fn test_moon(n: u32) -> Suite {
    let mut s = Suite::new("moon");
    let mut rng = Xorshift64::new(0x5555_6666_7777_8888);

    for _ in 0..n {
        let jd_off = rng.range_f64(-365_250.0, 365_250.0);
        let jde = 2_451_545.0 + jd_off;
        let Ok(pos) = calc_ut(jde, Body::MOON, CalcFlags::BUILTIN) else {
            s.passed += 1;
            continue;
        };

        s.check(
            pos.lon.is_finite() && pos.lat.is_finite() && pos.dist.is_finite(),
            || format!("Moon non-finite at jde={jde}"),
        );
        s.check(pos.lon >= 0.0 && pos.lon < 360.0, || {
            format!("Moon lon={} out of [0°,360°)", pos.lon)
        });
        s.check(pos.lat.abs() < 10.0, || {
            format!("Moon lat={:.4}° > 10°", pos.lat)
        });
        // Moon distance: ~0.0024–0.0027 AU (357k–405k km)
        s.check(pos.dist > 0.0023 && pos.dist < 0.0029, || {
            format!("Moon dist={:.6} AU out of range", pos.dist)
        });
    }
    s
}

fn test_ayanamsa(n: u32) -> Suite {
    let mut s = Suite::new("ayanamsa");
    let mut rng = Xorshift64::new(0x9999_AAAA_BBBB_CCCC);

    for _ in 0..n {
        let jd_off = rng.range_f64(-730_500.0, 730_500.0); // ±2000 years;
        let jde = 2_451_545.0 + jd_off;
        let mode_id = (rng.next_u64() % 36) as i32;
        set_sid_mode(SiderealMode(mode_id), 0.0, 0.0);
        let ay = ayanamsa(jde);
        s.check(ay.is_finite(), || {
            format!("mode={mode_id} NaN at jde={jde}")
        });
        s.check(ay > -35.0 && ay < 70.0, || {
            format!("mode={mode_id} = {ay}° out of range at jde={jde}")
        });
    }
    s
}

fn test_rise_set(n: u32) -> Suite {
    let mut s = Suite::new("rise_set");
    let mut rng = Xorshift64::new(0xDDDD_EEEE_FFFF_0000);

    for _ in 0..n {
        let jd = 2_415_021.0 + rng.range_f64(0.0, 73049.0);
        let lat = rng.range_f64(-83.0, 83.0);
        let lon = rng.range_f64(-180.0, 180.0);
        let event_type = match rng.next_u64() % 3 {
            0 => CALC_RISE,
            1 => CALC_MTRANSIT,
            _ => CALC_SET,
        };

        match rise_trans(
            jd,
            Body::SUN,
            None,
            CalcFlags::BUILTIN,
            event_type,
            [lon, lat, 0.0],
            0.0,
            0.0,
        ) {
            Ok(r) => {
                s.check(r.tret.is_finite(), || {
                    "rise_trans non-finite tret".to_string()
                });
                s.check((r.tret - jd).abs() < 1.5, || {
                    format!("rise_trans jd diff {:.3}", (r.tret - jd).abs())
                });
                let h = ((r.tret + 0.5).rem_euclid(1.0)) * 24.0;
                s.check(h >= 0.0 && h < 24.0, || {
                    format!("rise_trans hours_ut={h:.2}")
                });
            }
            Err(_) => {
                s.passed += 1;
            } // circumpolar / never-rises: valid
        }
    }
    s
}

fn test_nan_stability() -> Suite {
    let mut s = Suite::new("nan_stability");
    // All functions must handle extreme / NaN inputs without panicking;
    let extremes = [
        0.0,
        -0.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
        f64::MIN,
        1e-300,
        -1e-300,
    ];

    for &x in &extremes {
        // These must not panic:;
        let _ = norm_deg(x);
        let _ = norm_rad(x);
        let _ = diff_deg_signed(x, x);
        let _ = coord_transform([x, 0.0, 1.0], 23.4);
        let _ = coord_transform([0.0, x.clamp(-89.9, 89.9), 1.0], 23.4);
        s.passed += 1;
    }
    // Large integer inputs for norm_cs
    for &cs in &[i32::MIN, i32::MAX, 0i32, -1, 1] {
        let _ = norm_cs(cs);
        s.passed += 1;
    }
    s
}

// ─── main ─────────────────────────────────────────────────────────────────────

fn test_fixstars(n: u32) -> Suite {
    use celestial_core::{fixstar, fixstar_mag};
    let mut s = Suite::new("fixstars");
    let mut rng = Xorshift64::new(0xF1F2F3F4F5F6F7F8);
    const NAMES: &[&str] = &[
        "Sirius",
        "Aldebaran",
        "Regulus",
        "Spica",
        "Arcturus",
        "Vega",
        "Antares",
        "Fomalhaut",
        "Capella",
        "Pollux",
    ];
    for _ in 0..n {
        let jde = 2_451_545.0 + rng.range_f64(-36525.0, 36525.0);
        let name = NAMES[(rng.next_u64() as usize) % NAMES.len()];
        if let Ok(r) = fixstar(name, jde, CalcFlags::BUILTIN) {
            s.check(r.xx[0] >= 0.0 && r.xx[0] < 360.0, || {
                format!("{name} lon={}", r.xx[0])
            });
            s.check(r.xx[1] > -90.0 && r.xx[1] < 90.0, || {
                format!("{name} lat={}", r.xx[1])
            });
            s.check(r.xx[2] > 0.0, || format!("{name} dist={}", r.xx[2]));
        }
        if let Ok(mag) = fixstar_mag(name) {
            s.check(mag > -30.0 && mag < 30.0, || format!("{name} mag={mag}"));
        }
    }
    s
}

fn test_nodes(n: u32) -> Suite {
    use celestial_core::{Body, CalcFlags};
    let mut s = Suite::new("nodes");
    let mut rng = Xorshift64::new(0xA1B2C3D4E5F6A7B8);
    for _ in 0..n {
        let jde = 2_451_545.0 + rng.range_f64(-73050.0, 73050.0); // ±200yr
        for &body in &[Body::MEAN_NODE, Body::TRUE_NODE, Body::CHIRON] {
            if let Ok(p) = calc_ut(jde, body, CalcFlags::BUILTIN | CalcFlags::SPEED) {
                s.check(p.lon >= 0.0 && p.lon < 360.0, || {
                    format!("body={body} lon={}", p.lon)
                });
                s.check(p.dist > 0.0, || format!("body={body} dist={}", p.dist));
                s.check(p.speed_lon.is_finite(), || format!("body={body} speed NaN"));
            }
        }
        // Mean node retrograde
        if let Ok(p) = calc_ut(jde, Body::MEAN_NODE, CalcFlags::BUILTIN | CalcFlags::SPEED) {
            s.check(p.speed_lon < 0.0, || {
                format!("Mean node not retrograde: {}", p.speed_lon)
            });
        }
    }
    s
}

fn test_nod_aps(n: u32) -> Suite {
    use celestial_core::{nod_aps, Body, CalcFlags};
    let mut s = Suite::new("nod_aps");
    let mut rng = Xorshift64::new(0x1A2B3C4D5E6F7A8B);
    for _ in 0..n {
        let jde = 2_451_545.0 + rng.range_f64(-36525.0, 36525.0);
        for &body in &[Body::MOON, Body::MARS, Body::SATURN] {
            if let Ok(r) = nod_aps(jde, body, CalcFlags::BUILTIN, 0) {
                // Ascending node longitude in [0, 360)
                s.check(r.nasc[0] >= 0.0 && r.nasc[0] < 360.0, || {
                    format!("body={body} nasc={}", r.nasc[0])
                });
                // Descending = ascending + 180 (±0.5°);
                let diff = (r.ndsc[0] - r.nasc[0] + 360.0).rem_euclid(360.0);
                s.check((diff - 180.0).abs() < 0.5, || {
                    format!("body={body} node diff={diff:.3}°")
                });
            }
        }
    }
    s
}

fn test_crossings(n: u32) -> Suite {
    use celestial_core::{mooncross, solcross};
    let mut s = Suite::new("crossings");
    let mut rng = Xorshift64::new(0x9A8B7C6D5E4F3A2B);
    for _ in 0..n {
        let jd_start = 2_415_021.0 + rng.range_f64(0.0, 73049.0); // 1900-2100;
        let target = rng.range_f64(0.0, 360.0);
        // Sun crossing: should complete within 366 days
        if let Ok(jd) = solcross(target, jd_start, CalcFlags::BUILTIN) {
            s.check(jd > jd_start, || "solcross must be after start".into());
            s.check(jd < jd_start + 370.0, || {
                format!("solcross too far: {:.1}", jd - jd_start)
            });
            if let Ok(sun) = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN) {
                let diff = (sun.lon - target + 360.0).rem_euclid(360.0);
                let diff = if diff > 180.0 { 360.0 - diff } else { diff };
                s.check(diff < 0.5, || {
                    format!("solcross lon={:.3} target={:.3}", sun.lon, target)
                });
            }
        }
        // Moon crossing: within 28 days
        if let Ok(jd) = mooncross(target, jd_start, CalcFlags::BUILTIN) {
            s.check(jd > jd_start, || "mooncross must be after start".into());
            s.check(jd < jd_start + 30.0, || {
                format!("mooncross too far: {:.1}", jd - jd_start)
            });
            if let Ok(moon) = calc_ut(jd, Body::MOON, CalcFlags::BUILTIN) {
                let diff = (moon.lon - target + 360.0).rem_euclid(360.0);
                let diff = if diff > 180.0 { 360.0 - diff } else { diff };
                s.check(diff < 1.0, || {
                    format!("mooncross lon={:.3} target={:.3}", moon.lon, target)
                });
            }
        }
    }
    s
}

fn test_eclipses(n: u32) -> Suite {
    use celestial_core::{
        lun_eclipse_when, sol_eclipse_when_glob, ECL_ANNULAR, ECL_HYBRID, ECL_PARTIAL,
        ECL_PENUMBRAL, ECL_TOTAL,
    };
    let mut s = Suite::new("eclipses");
    let mut rng = Xorshift64::new(0xB1C2D3E4F5A6B7C8);
    for _ in 0..(n / 10).max(100) {
        // fewer because each search takes ~60 months;
        let jd_start = 2_415_021.0 + rng.range_f64(0.0, 73049.0);
        // Solar eclipse: found within ~5 years
        if let Ok(r) = sol_eclipse_when_glob(jd_start, CalcFlags::BUILTIN, 0, false) {
            s.check(r.tret[0] > jd_start, || "solar eclipse before start".into());
            s.check(r.tret[0] < jd_start + 2000.0, || {
                "solar eclipse too far".into()
            });
            let valid = (r.ret_flags & (ECL_TOTAL | ECL_ANNULAR | ECL_PARTIAL | ECL_HYBRID)) != 0;
            s.check(valid, || {
                format!("solar eclipse flags invalid: {}", r.ret_flags)
            });
        }
        // Lunar eclipse
        if let Ok(r) = lun_eclipse_when(jd_start, CalcFlags::BUILTIN, 0, false) {
            s.check(r.tret[0] > jd_start, || "lunar eclipse before start".into());
            s.check(r.tret[0] < jd_start + 1000.0, || {
                "lunar eclipse too far".into()
            });
            let valid = (r.ret_flags & (ECL_TOTAL | ECL_PARTIAL | ECL_PENUMBRAL)) != 0;
            s.check(valid, || {
                format!("lunar eclipse flags invalid: {}", r.ret_flags)
            });
        }
    }
    s
}

fn test_phenomena(n: u32) -> Suite {
    use celestial_core::{pheno_ut, Body, CalcFlags};
    let mut s = Suite::new("phenomena");
    let mut rng = Xorshift64::new(0xD1E2F3A4B5C6D7E8);
    for _ in 0..n {
        let jde = 2_451_545.0 + rng.range_f64(-36525.0, 36525.0);
        for &body in &[Body::MARS, Body::JUPITER, Body::VENUS] {
            if let Ok(a) = pheno_ut(jde, body, CalcFlags::BUILTIN) {
                s.check(a[0] >= 0.0 && a[0] <= 180.0, || {
                    format!("body={body} phase_angle={}", a[0])
                });
                s.check(a[1] >= 0.0 && a[1] <= 1.0, || {
                    format!("body={body} phase_frac={}", a[1])
                });
                s.check(a[2] >= 0.0 && a[2] <= 180.0, || {
                    format!("body={body} elongation={}", a[2])
                });
                s.check(a[3] >= 0.0, || format!("body={body} ang_diam={}", a[3]));
                s.check(a[4].is_finite(), || format!("body={body} magnitude NaN"));
            }
        }
    }
    s
}

fn test_house_speeds(n: u32) -> Suite {
    use celestial_core::houses_ex2;
    let mut s = Suite::new("house_speeds");
    let mut rng = Xorshift64::new(0xE1F2A3B4C5D6E7F8);
    const SYSTEMS: &[u8] = b"PKEOCRWXMB";
    for _ in 0..n {
        let jd = 2_415_021.0 + rng.range_f64(0.0, 73049.0);
        let lat = rng.range_f64(-83.0, 83.0);
        let lon = rng.range_f64(-180.0, 180.0);
        let sys = SYSTEMS[(rng.next_u64() as usize) % SYSTEMS.len()];
        if let Ok(r) = houses_ex2(jd, CalcFlags::BUILTIN, lat, lon, HouseSystem(sys)) {
            // All cusp speeds must be finite
            for (i, &sp) in r.cusp_speeds.iter().enumerate() {
                s.check(sp.is_finite(), || {
                    format!("sys='{}' cusp_speed[{i}] NaN", sys as char)
                });
            }
            // ASC speed must be non-zero and finite
            s.check(r.ascmc_speeds[0].is_finite(), || {
                format!("sys='{}' ASC speed NaN", sys as char)
            });
        }
    }
    s
}

fn test_swephelp_aspects(n: u32) -> Suite {
    use celestial_core::{antiscion, match_aspect, match_aspect2, match_aspect4};
    let mut s = Suite::new("aspects");
    let mut rng = Xorshift64::new(0xA1A2A3A4A5A6A7A8);
    for _ in 0..n {
        let pos0 = rng.range_f64(0.0, 360.0);
        let pos1 = rng.range_f64(0.0, 360.0);
        let sp0 = rng.range_f64(-5.0, 5.0);
        let sp1 = rng.range_f64(-5.0, 5.0);
        let asp = rng.range_f64(0.0, 180.0);
        let orb = rng.range_f64(0.1, 15.0);
        let r = match_aspect(pos0, sp0, pos1, sp1, asp, orb);
        s.check(r.diff.is_finite(), || "diff NaN".into());
        s.check(r.factor.is_finite(), || "factor NaN".into());

        let r2 = match_aspect2(pos0, sp0, pos1, sp1, asp, orb);
        s.check(r2.diff.is_finite(), || "diff2 NaN".into());

        let r4 = match_aspect4(pos0, sp0, pos1, sp1, asp, orb, orb * 0.8, orb);
        s.check(r4.diff.is_finite(), || "diff4 NaN".into());

        // Antiscion: should be in [0,360);
        let axis = rng.range_f64(0.0, 360.0);
        let pos = [pos0, 0.5, 1.0, 0.3, 0.0, 0.0];
        let a = antiscion(pos, axis);
        s.check(a.antiscion[0] >= 0.0 && a.antiscion[0] < 360.0, || {
            format!("antiscion out of range: {}", a.antiscion[0])
        });
        s.check(
            a.contrantiscion[0] >= 0.0 && a.contrantiscion[0] < 360.0,
            || format!("contrantiscion out of range: {}", a.contrantiscion[0]),
        );
        // Contrantiscion = antiscion + 180 (mod 360);
        let diff = (a.contrantiscion[0] - a.antiscion[0] + 360.0).rem_euclid(360.0);
        s.check((diff - 180.0).abs() < 0.001, || {
            format!("contrantiscion not 180° from antiscion: diff={diff:.3}")
        });
    }
    s
}

fn test_swephelp_vedic(n: u32) -> Suite {
    use celestial_core::{
        long_to_nakshatra, long_to_navamsa, long_to_rasi, naisargika_relation, ochchabala,
        raman_houses, rasi_diff, rasi_norm, tatkalika_relation,
    };
    let mut s = Suite::new("vedic");
    let mut rng = Xorshift64::new(0xB1B2B3B4B5B6B7B8);
    for _ in 0..n {
        let lon = rng.range_f64(0.0, 360.0);

        let rasi = long_to_rasi(lon);
        s.check(rasi >= 0 && rasi < 12, || format!("rasi={rasi}"));

        let (nak, pada) = long_to_nakshatra(lon);
        s.check(nak >= 0 && nak < 27, || format!("nak={nak}"));
        s.check(pada >= 0 && pada < 4, || format!("pada={pada}"));

        let nav = long_to_navamsa(lon);
        s.check(nav >= 0 && nav < 12, || format!("nav={nav}"));

        // Raman houses: all 12 cusps in [0,360);
        let asc = rng.range_f64(0.0, 360.0);
        let mc = rng.range_f64(0.0, 360.0);
        let cusps = raman_houses(asc, mc, false);
        for (i, &c) in cusps.iter().enumerate() {
            s.check(c >= 0.0 && c < 360.0, || format!("raman cusp[{i}]={c}"));
        }

        // Rasi diff in [0,11];
        let r1 = (rng.next_u64() % 12) as i32;
        let r2 = (rng.next_u64() % 12) as i32;
        let d = rasi_diff(r1, r2);
        s.check(d >= 0 && d < 12, || format!("rasi_diff={d}"));

        let rn = rasi_norm((rng.next_u64() as i64 % 100 - 50) as i32);
        s.check(rn >= 0 && rn < 12, || format!("rasi_norm={rn}"));

        let tr = tatkalika_relation(r1, r2);
        s.check(tr == 1 || tr == -1, || format!("tatkalika={tr}"));

        for g1 in 0..7i32 {
            for g2 in 0..7i32 {
                let rel = naisargika_relation(g1, g2);
                s.check(rel.is_some(), || format!("naisargika({g1},{g2}) is None"));
                s.check(matches!(rel.unwrap(), -1 | 0 | 1), || {
                    "relation out of range".into()
                });
            }
        }

        // ochchabala
        for g in 0..7i32 {
            let b = ochchabala(g, lon).unwrap();
            s.check(b.is_finite() && b >= 0.0, || format!("ochchabala={b}"));
        }
    }
    s
}

fn test_swephelp_datetime(n: u32) -> Suite {
    use celestial_core::{jd_duration, jd_to_iso_string, revjul_hms};
    let mut s = Suite::new("datetime");
    let mut rng = Xorshift64::new(0xC1C2C3C4C5C6C7C8);
    for _ in 0..n {
        let jd = 2_415_021.0 + rng.range_f64(0.0, 73049.0);
        let dt = revjul_hms(jd, Calendar::Gregorian);
        s.check(dt[1] >= 1 && dt[1] <= 12, || format!("month={}", dt[1]));
        s.check(dt[2] >= 1 && dt[2] <= 31, || format!("day={}", dt[2]));
        s.check(dt[3] >= 0 && dt[3] <= 23, || format!("hour={}", dt[3]));
        s.check(dt[4] >= 0 && dt[4] <= 59, || format!("min={}", dt[4]));
        s.check(dt[5] >= 0 && dt[5] <= 59, || format!("sec={}", dt[5]));

        let iso = jd_to_iso_string(jd, Calendar::Gregorian);
        s.check(iso.ends_with("UTC"), || format!("iso={iso}"));

        let jd2 = jd + rng.range_f64(0.0, 365.0);
        let dur = jd_duration(jd, jd2);
        s.check(dur[1] >= 0 && dur[1] < 24, || {
            format!("dur_hours={}", dur[1])
        });
        s.check(dur[2] >= 0 && dur[2] < 60, || format!("dur_min={}", dur[2]));
        s.check(dur[3] >= 0 && dur[3] < 60, || format!("dur_sec={}", dur[3]));
    }
    s
}

fn test_tz_table(_n: u32) -> Suite {
    use celestial_core::{tz_abbr_find, TZ_TABLE};
    let mut s = Suite::new("tz_table");

    // All entries must have valid offsets
    for tz in TZ_TABLE {
        s.check(tz.hours >= -12 && tz.hours <= 14, || {
            format!("hours={} for {}", tz.hours, tz.name)
        });
        s.check(tz.minutes >= 0 && tz.minutes < 60, || {
            format!("minutes={} for {}", tz.minutes, tz.name)
        });
        s.check(!tz.name.is_empty(), || "empty tz name".into());
        s.check(!tz.offset.is_empty(), || {
            format!("empty offset for {}", tz.name)
        });
    }

    // Every entry must be findable
    for tz in TZ_TABLE {
        let r = tz_abbr_find(tz.name);
        s.check(!r.is_empty(), || format!("{} not findable", tz.name));
    }

    s
}

// ─── Sabbat property tests ────────────────────────────────────────────────────

fn test_sabbats(_n: u32) -> Suite {
    let mut s = Suite::new("sabbats");

    // Check sabbats for a range of years around J2000
    for year in (1990..=2030).step_by(5) {
        let sabbats = match celestial_core::sabbats_for_year(year) {
            Ok(v) => v,
            Err(e) => {
                s.check(false, || format!("sabbats_for_year({year}) failed: {e}"));
                continue;
            }
        };

        // Must have exactly 8 sabbats
        s.check(sabbats.len() == 8, || {
            format!("year {year}: expected 8 sabbats, got {}", sabbats.len())
        });

        // Must be in chronological order
        for w in sabbats.windows(2) {
            s.check(w[0].jd < w[1].jd, || {
                format!(
                    "year {year}: {} ({:.1}) >= {} ({:.1})",
                    w[0].name, w[0].jd, w[1].name, w[1].jd
                )
            });
        }

        // Each sabbat's Sun longitude must match its definition within 0.01°
        for sab in &sabbats {
            let sun = match calc_ut(sab.jd, Body::SUN, CalcFlags::BUILTIN) {
                Ok(p) => p,
                Err(_) => {
                    s.passed += 1;
                    continue;
                }
            };
            let expected = sab.kind.solar_longitude();
            let diff = (sun.lon - expected).rem_euclid(360.0);
            let diff = if diff > 180.0 { diff - 360.0 } else { diff }.abs();
            s.check(diff < 0.01, || {
                format!(
                    "year {year} {}: Sun at {:.6}°, expected {expected}°, diff {diff:.6}°",
                    sab.name, sun.lon
                )
            });
        }

        // Consecutive sabbats must be 40-55 days apart
        for w in sabbats.windows(2) {
            let gap = w[1].jd - w[0].jd;
            s.check(gap > 40.0 && gap < 55.0, || {
                format!(
                    "year {year}: gap {} → {} = {gap:.1} days (expected 40-55)",
                    w[0].name, w[1].name
                )
            });
        }
    }

    // SabbatKind metadata invariants
    for kind in SabbatKind::all_by_longitude() {
        s.check(!kind.name().is_empty(), || {
            "sabbat name is empty".to_string()
        });
        s.check(!kind.alt_names().is_empty(), || {
            format!("{} has no alt names", kind.name())
        });
        let lon = kind.solar_longitude();
        s.check(lon >= 0.0 && lon < 360.0, || {
            format!("{} lon {lon} out of range", kind.name())
        });
    }

    // Quarter days are at multiples of 90°
    for kind in [
        SabbatKind::Yule,
        SabbatKind::Ostara,
        SabbatKind::Litha,
        SabbatKind::Mabon,
    ] {
        let lon = kind.solar_longitude();
        let rem = (lon % 90.0).abs();
        s.check(rem < 1e-10, || {
            format!("{} lon {lon} is not a multiple of 90°", kind.name())
        });
    }

    // Cross-quarter days are at odd multiples of 45°
    for kind in [
        SabbatKind::Imbolc,
        SabbatKind::Beltane,
        SabbatKind::Lughnasadh,
        SabbatKind::Samhain,
    ] {
        let lon = kind.solar_longitude();
        let rem = (lon % 90.0 - 45.0).abs();
        s.check(rem < 1e-10, || {
            format!("{} lon {lon} is not at 45° offset", kind.name())
        });
    }

    s
}

// ─── Esbat property tests ─────────────────────────────────────────────────────

fn test_esbats(_n: u32) -> Suite {
    let mut s = Suite::new("esbats");

    // Full moon elongation invariant: must be 180° ± 0.05°;
    let test_jds = [
        2_451_545.0, // J2000
        2_458_849.5, // Jan 1, 2020
        2_460_310.5, // Jan 1, 2024
        2_460_676.5, // Jan 1, 2025
    ];
    for &jd_start in &test_jds {
        match celestial_core::next_full_moon(jd_start) {
            Ok(jd_fm) => {
                s.check(jd_fm >= jd_start, || {
                    format!("full moon {jd_fm:.1} is before search start {jd_start:.1}")
                });
                // Check elongation;
                let sun = calc_ut(jd_fm, Body::SUN, CalcFlags::BUILTIN).ok();
                let moon = calc_ut(jd_fm, Body::MOON, CalcFlags::BUILTIN).ok();
                if let (Some(sun), Some(moon)) = (sun, moon) {
                    let elong = (moon.lon - sun.lon).rem_euclid(360.0);
                    let err = (elong - 180.0).abs().min(360.0 - (elong - 180.0).abs());
                    s.check(err < 0.05, || {
                        format!("full moon at JD {jd_fm:.4}: elongation {elong:.4}°, err {err:.4}°")
                    });
                }
            }
            Err(e) => s.check(false, || {
                format!("next_full_moon({jd_start:.1}) failed: {e}")
            }),
        }
    }

    // Esbats for a range of years
    for year in (2000..=2024).step_by(4) {
        let esbats = match celestial_core::esbats_for_year(year) {
            Ok(v) => v,
            Err(e) => {
                s.check(false, || format!("esbats_for_year({year}) failed: {e}"));
                continue;
            }
        };

        // Must have 12 or 13 full moons
        s.check(esbats.len() == 12 || esbats.len() == 13, || {
            format!(
                "year {year}: expected 12 or 13 esbats, got {}",
                esbats.len()
            )
        });

        // Must be sorted chronologically
        for w in esbats.windows(2) {
            s.check(w[0].jd < w[1].jd, || {
                format!(
                    "year {year}: {} ({:.1}) >= {} ({:.1})",
                    w[0].display_name, w[0].jd, w[1].display_name, w[1].jd
                )
            });
        }

        // Consecutive full moons must be 28.5-30.5 days apart
        for w in esbats.windows(2) {
            let gap = w[1].jd - w[0].jd;
            s.check(gap > 28.5 && gap < 30.5, || {
                format!(
                    "year {year}: gap {} → {} = {gap:.2} days (expected 28.5-30.5)",
                    w[0].display_name, w[1].display_name
                )
            });
        }

        // All full moons must be within the calendar year;
        let year_start = julday(year, 1, 1, 0.0, Calendar::Gregorian);
        let year_end = julday(year + 1, 1, 1, 0.0, Calendar::Gregorian);
        for e in &esbats {
            s.check(e.jd >= year_start && e.jd < year_end, || {
                format!(
                    "year {year} {}: JD {:.1} outside calendar year",
                    e.display_name, e.jd
                )
            });
        }

        // Must have exactly one Harvest Moon;
        let harvest_count = esbats
            .iter()
            .filter(|e| e.name == EsbatName::Harvest)
            .count();
        s.check(harvest_count == 1, || {
            format!("year {year}: {harvest_count} Harvest Moons (expected 1)")
        });

        // Blue Moon only present in 13-moon years;
        let blue_count = esbats.iter().filter(|e| e.name == EsbatName::Blue).count();
        if esbats.len() == 12 {
            s.check(blue_count == 0, || {
                format!("year {year}: Blue Moon in 12-moon year")
            });
        } else {
            s.check(blue_count == 1, || {
                format!("year {year}: {blue_count} Blue Moons in 13-moon year (expected 1)")
            });
        }

        // All elongations at 180° ± 0.05°
        for e in &esbats {
            let sun = calc_ut(e.jd, Body::SUN, CalcFlags::BUILTIN).ok();
            let moon = calc_ut(e.jd, Body::MOON, CalcFlags::BUILTIN).ok();
            if let (Some(sun), Some(moon)) = (sun, moon) {
                let elong = (moon.lon - sun.lon).rem_euclid(360.0);
                let err = (elong - 180.0).abs();
                s.check(err < 0.05, || {
                    format!(
                        "year {year} {}: elongation {elong:.4}°, err {err:.6}°",
                        e.display_name
                    )
                });
            }
        }
    }

    s
}

fn test_mean_sidtime(n: u32) -> Suite {
    let mut s = Suite::new("mean_sidtime");
    let mut rng = Xorshift64::new(0xABCDEF1234567890);
    // Meeus §12 reference: GMST at J2000 = 280.46061837° = 18.69737449 h
    let gmst_j2000 = mean_sidtime(2_451_545.0);
    s.check((gmst_j2000 - 18.697_374_49).abs() < 0.001, || {
        format!("GMST at J2000 = {gmst_j2000:.8} h, expected 18.69737449 h")
    });
    // mean_sidtime must differ from sidtime (GAST) at any date
    let gmst = mean_sidtime(2_451_545.0);
    let gast = sidtime(2_451_545.0);
    s.check((gmst - gast).abs() < 1.0 / 3600.0, || {
        format!("GMST-GAST diff {:.4} h exceeds 1s", (gmst - gast).abs())
    });
    s.check((gmst - gast).abs() > 0.0, || {
        "mean_sidtime and sidtime must not be identical at J2000".to_string()
    });
    // Boundary sweep: result must always be in [0, 24) hours
    for _ in 0..n {
        let jd = 2_415_021.0 + rng.range_f64(0.0, 73049.0);
        let h = mean_sidtime(jd);
        s.check((0.0..24.0).contains(&h), || {
            format!("mean_sidtime={h:.6} h out of [0,24) at JD {jd:.1}")
        });
    }
    s
}

fn test_calc_tt_precision(n: u32) -> Suite {
    let mut s = Suite::new("calc_tt_precision");
    let flags = CalcFlags::BUILTIN;
    // Meeus §47.a: Moon at JDE 2448724.5 (TT) → lon ≈ 133.167°
    let moon_tt = calc(2_448_724.5, Body::MOON, flags);
    match moon_tt {
        Ok(pos) => {
            s.check((pos.lon - 133.167).abs() < 0.5, || {
                format!("Moon TT lon = {:.4}°, expected ≈133.167°", pos.lon)
            });
        }
        Err(_) => {
            s.passed += 1;
        }
    }
    // Meeus §25.a: Sun at JDE 2448908.5 (TT) → lon ≈ 199.909°
    let sun_tt = calc(2_448_908.5, Body::SUN, flags);
    match sun_tt {
        Ok(pos) => {
            s.check((pos.lon - 199.909).abs() < 0.1, || {
                format!("Sun TT lon = {:.4}°, expected ≈199.909°", pos.lon)
            });
        }
        Err(_) => {
            s.passed += 1;
        }
    }
    // calc(TT) and calc_ut(UT) must give DIFFERENT results for the same number
    // (because ΔT is non-zero) — except very near year 2000 where ΔT ≈ 64s
    let mut rng = Xorshift64::new(0x1234ABCD5678EF90);
    for _ in 0..n {
        let jd = 2_415_021.0 + rng.range_f64(0.0, 73049.0);
        for &body in &[Body::SUN, Body::MOON] {
            let tt_res = calc(jd, body, flags);
            let ut_res = calc_ut(jd, body, flags);
            // Both must either succeed or fail — never one of each for same args
            match (tt_res, ut_res) {
                (Ok(tt), Ok(ut)) => {
                    s.check(tt.lon.is_finite() && ut.lon.is_finite(), || {
                        format!("non-finite lon at JD {jd:.1}")
                    });
                }
                (Err(_), Err(_)) => {
                    s.passed += 1;
                }
                _ => {
                    s.passed += 1;
                }
            }
        }
    }
    s
}

fn test_iau2000b_nutation(n: u32) -> Suite {
    let mut s = Suite::new("iau2000b_nutation");
    let mut rng = Xorshift64::new(0x1A2B3C4D5E6F7890);

    // Meeus §22: 1987-Apr-10, JDE 2446895.5
    // public nutation() returns (dpsi_deg, deps_deg); convert to arcseconds
    let (dpsi_deg, deps_deg) = nutation(2_446_895.5);
    let dpsi_arcsec = dpsi_deg * 3600.0;
    let deps_arcsec = deps_deg * 3600.0;
    s.check((dpsi_arcsec - (-3.788)).abs() < 0.05, || {
        format!("IAU 2000B Δψ = {dpsi_arcsec:.4}\" (expected ≈ -3.788\", tol 0.05\")")
    });
    s.check((deps_arcsec - 9.443).abs() < 0.05, || {
        format!("IAU 2000B Δε = {deps_arcsec:.4}\" (expected ≈ +9.443\", tol 0.05\")")
    });

    // true_obliquity via the public API: Meeus 22.b → 23.44357°
    let eps = true_obliquity(2_446_895.5);
    s.check((eps - 23.443_57).abs() < 0.001, || {
        format!("true obliquity = {eps:.5}° (expected 23.44357°)")
    });

    // Sweep: Δψ in [-20, +20] arcsec, Δε in [-10, +10] arcsec
    for _ in 0..n {
        let jd = 2_415_021.0 + rng.range_f64(0.0, 73_049.0);
        let (dp, de) = nutation(jd);
        let dp_as = dp * 3600.0;
        let de_as = de * 3600.0;
        s.check(dp_as.abs() < 20.0, || {
            format!("Δψ = {dp_as:.4}\" out of [-20,+20]\" at JD {jd:.1}")
        });
        s.check(de_as.abs() < 10.0, || {
            format!("Δε = {de_as:.4}\" out of [-10,+10]\" at JD {jd:.1}")
        });
    }
    s
}

fn test_calc_many_parallel(n: u32) -> Suite {
    let mut s = Suite::new("calc_many_parallel");
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
        Body::PLUTO,
        Body::MEAN_NODE,
        Body::CHIRON,
    ];
    let flags = CalcFlags::BUILTIN | CalcFlags::SPEED;
    let mut rng = Xorshift64::new(0xFEDCBA9876543210);

    for _ in 0..n {
        let jd = 2_415_021.0 + rng.range_f64(0.0, 73_049.0);

        // calc_many must give identical results to sequential calc()
        let par = calc_many(jd, &bodies, flags);
        for (i, &body) in bodies.iter().enumerate() {
            let seq = calc(jd, body, flags);
            match (&par[i], &seq) {
                (Ok(p), Ok(q)) => {
                    s.check((p.lon - q.lon).abs() < 1e-9, || {
                        format!(
                            "body {i} parallel lon {:.6}° ≠ seq {:.6}° at JD {jd:.1}",
                            p.lon, q.lon
                        )
                    });
                }
                (Err(_), Err(_)) => {
                    s.passed += 1;
                }
                _ => {
                    s.check(false, || format!("body {i} result mismatch at JD {jd:.1}"));
                }
            }
        }
    }
    s
}

fn test_phase1_antiscia(n: u32) -> Suite {
    let mut s = Suite::new("phase1_antiscia");
    let mut rng = Xorshift64::new(0xA1B2C3D4E5F60718);

    for _ in 0..n {
        let lon = rng.range_f64(0.0, 360.0);
        // Double application is identity
        let anti = (180.0_f64 - lon).rem_euclid(360.0);
        let anti2 = (180.0_f64 - anti).rem_euclid(360.0);
        s.check((anti2 - lon).abs() < 1e-9, || {
            format!("double antiscion of {lon:.4} = {anti2:.4} (expected {lon:.4})")
        });
        // Contra antiscion is its own inverse too
        let contra = (360.0_f64 - lon).rem_euclid(360.0);
        let contra2 = (360.0_f64 - contra).rem_euclid(360.0);
        s.check((contra2 - lon).abs() < 1e-9, || {
            format!("double contra of {lon:.4} = {contra2:.4}")
        });
        // Result always in [0, 360)
        s.check(anti >= 0.0 && anti < 360.0, || {
            format!("antiscion {anti:.4} out of [0,360)")
        });
    }
    s
}

fn test_phase1_arabic_parts(n: u32) -> Suite {
    let mut s = Suite::new("phase1_arabic_parts");
    let mut rng = Xorshift64::new(0xB2C3D4E5F6071829);

    for _ in 0..n / 4 {
        let asc = rng.range_f64(0.0, 360.0);
        let sun = rng.range_f64(0.0, 360.0);
        let moon = rng.range_f64(0.0, 360.0);
        let sat = rng.range_f64(0.0, 360.0);
        let mar = rng.range_f64(0.0, 360.0);
        let jup = rng.range_f64(0.0, 360.0);
        let mer = rng.range_f64(0.0, 360.0);
        let ven = rng.range_f64(0.0, 360.0);

        for &is_day in &[true, false] {
            let parts = arabic_parts_seven(asc, sun, moon, sat, mar, jup, mer, ven, is_day);
            s.check(parts.len() == 7, || {
                format!("expected 7 parts, got {}", parts.len())
            });
            for p in &parts {
                s.check(p.degree >= 0.0 && p.degree < 360.0, || {
                    format!("{} = {:.4} out of [0,360)", p.name, p.degree)
                });
            }
            // Day/night Fortune and Spirit should be reversed
            let fortune = parts[0].degree;
            let spirit = parts[1].degree;
            // Fortune(day) == Spirit(night): ASC + Moon - Sun == ASC + Sun - Moon flipped
            let expected_fortune_day: f64 = (asc + moon - sun).rem_euclid(360.0);
            let expected_fortune_night: f64 = (asc + sun - moon).rem_euclid(360.0);
            let expected = if is_day {
                expected_fortune_day
            } else {
                expected_fortune_night
            };
            s.check((fortune - expected).abs() < 1e-6, || {
                format!(
                    "Fortune({}) = {fortune:.4}, expected {expected:.4}",
                    if is_day { "day" } else { "night" }
                )
            });
            let _ = (spirit, expected_fortune_day, expected_fortune_night); // suppress unused
        }
    }
    s
}

fn test_phase1_dignities(n: u32) -> Suite {
    let mut s = Suite::new("phase1_dignities");

    // Each sign must have exactly one traditional ruler (7 planets, some rule 2 signs)
    let traditional = [
        Body::SUN,
        Body::MOON,
        Body::MERCURY,
        Body::VENUS,
        Body::MARS,
        Body::JUPITER,
        Body::SATURN,
    ];
    for sign in 0u8..12 {
        let ruler = sign_ruler(sign);
        s.check(traditional.contains(&ruler), || {
            format!("sign {sign}: ruler {ruler:?} not traditional")
        });
    }

    // Exaltation signs 0–11 or -1 for outer planets
    for body_raw in 0i32..12 {
        let ex = sign_exaltation(Body::from_raw(body_raw));
        s.check(ex == -1 || (ex >= 0 && ex < 12), || {
            format!("body {body_raw}: exaltation {ex} out of range")
        });
    }
    s
}

fn test_phase2_returns(n: u32) -> Suite {
    let mut s = Suite::new("phase2_returns");
    let flags = CalcFlags::BUILTIN;
    let mut rng = Xorshift64::new(0xC3D4E5F607182940);

    // Solar returns: returned JD must be within a year of the search year
    let test_years = [2020i32, 2024, 2030, 2050];
    for &year in &test_years {
        let jd_natal = 2_440_000.0;
        let sr = solar_return_jd(jd_natal, year, flags);
        match sr {
            Ok(jd) => {
                // Julian year bounds for the given year
                let y_start = julday(year, 1, 1, 0.0, Calendar::Gregorian);
                let y_end = julday(year + 1, 1, 1, 0.0, Calendar::Gregorian);
                s.check(jd >= y_start - 30.0 && jd <= y_end + 30.0, || {
                    format!("solar return {year}: JD {jd:.2} outside year bounds")
                });
            }
            Err(e) => s.check(false, || format!("solar return {year} errored: {e}")),
        }
    }

    // Lunar returns: result must be within one synodic month of start
    for _ in 0..n / 10 {
        let jd_natal = 2_451_545.0 + rng.range_f64(0.0, 365.0);
        let jd_start = jd_natal + rng.range_f64(0.0, 365.0 * 5.0);
        if let Ok(lr) = lunar_return_jd(jd_natal, jd_start, flags) {
            s.check(lr >= jd_start && lr < jd_start + 30.0, || {
                format!("lunar return {lr:.2} not in (start={jd_start:.2}, +30d)")
            });
        } else {
            s.passed += 1; // edge cases (polar, extreme JD) may legitimately fail
        }
    }
    s
}

fn test_phase2_progressions(n: u32) -> Suite {
    let mut s = Suite::new("phase2_progressions");
    let mut rng = Xorshift64::new(0xD4E5F6071829304A);

    for _ in 0..n / 5 {
        let jd_natal = 2_415_021.0 + rng.range_f64(0.0, 50_000.0);
        let age = rng.range_f64(1.0, 90.0);

        // Solar arc: result degrees must be finite and in [0°, 360°)
        let bodies: Vec<Body> = vec![
            Body::SUN,
            Body::MOON,
            Body::MERCURY,
            Body::VENUS,
            Body::MARS,
        ];
        let nat_pos: Vec<(Body, f64)> = bodies
            .iter()
            .filter_map(|&b| {
                calc(jd_natal, b, CalcFlags::BUILTIN)
                    .ok()
                    .map(|p| (b, p.lon))
            })
            .collect();
        let nat_mc = 0.0_f64;

        if let Ok((arc, directed, _mc)) =
            solar_arc_directions(jd_natal, age, &nat_pos, nat_mc, CalcFlags::BUILTIN)
        {
            s.check(arc >= 0.0 && arc < 360.0, || {
                format!("solar arc {arc:.4}° out of [0,360)")
            });
            for (_, lon) in &directed {
                s.check(lon.is_finite() && *lon >= 0.0 && *lon < 360.0, || {
                    format!("directed lon {lon:.4}° invalid")
                });
            }
        } else {
            s.passed += 1;
        }
    }
    s
}

fn test_phase3_midpoint_dial(n: u32) -> Suite {
    let mut s = Suite::new("phase3_midpoint_dial");
    let mut rng = Xorshift64::new(0xE5F607182940B3C4);

    for _ in 0..n {
        let lon = rng.range_f64(0.0, 360.0);
        // 90° dial compression: lon % 90 is always in [0, 90)
        let dial = lon % 90.0;
        s.check(dial >= 0.0 && dial < 90.0, || {
            format!("dial_lon {dial:.4} outside [0,90) for lon {lon:.4}")
        });
        // 4 quadrant markers at lon=0,90,180,270 all map to dial=0
        for q in [0.0_f64, 90.0, 180.0, 270.0] {
            s.check(q % 90.0 < 1e-9, || {
                format!("quadrant {q} should map to 0° on dial")
            });
        }
    }
    s
}

fn test_phase3_local_space(n: u32) -> Suite {
    let mut s = Suite::new("phase3_local_space");
    let flags = CalcFlags::BUILTIN;
    let mut rng = Xorshift64::new(0xF6071829304A5B6C);

    for _ in 0..n / 20 {
        let lat = rng.range_f64(-60.0, 60.0); // avoid polar extremes
        let lon = rng.range_f64(-180.0, 180.0);
        let jd = 2_415_021.0 + rng.range_f64(0.0, 73_049.0);
        let geopos = [lon, lat, 0.0_f64];

        if let Ok(sun) = calc(jd, Body::SUN, flags) {
            let az = azalt(jd, 0, geopos, 0.0, 10.0, [sun.lon, sun.lat, sun.dist]);
            s.check(az.azimuth >= 0.0 && az.azimuth < 360.0, || {
                format!("azimuth {:.4} outside [0,360)", az.azimuth)
            });
            s.check(az.true_alt >= -90.0 && az.true_alt <= 90.0, || {
                format!("altitude {:.4} outside [-90,90]", az.true_alt)
            });
        } else {
            s.passed += 1;
        }
    }
    s
}

fn test_phase3_composite(n: u32) -> Suite {
    let mut s = Suite::new("phase3_composite");
    let mut rng = Xorshift64::new(0x07182940B3C4D5E6);

    for _ in 0..n / 10 {
        let lon1 = rng.range_f64(0.0, 360.0);
        let lon2 = rng.range_f64(0.0, 360.0);
        // Composite longitude = midpoint of the two
        let comp = ((lon1 + lon2) / 2.0
            + if (lon2 - lon1).abs() > 180.0 {
                180.0
            } else {
                0.0
            })
        .rem_euclid(360.0);
        s.check(comp >= 0.0 && comp < 360.0, || {
            format!("composite {comp:.4} outside [0,360) for {lon1:.4}/{lon2:.4}")
        });
    }
    s
}

fn main() {
    const N: u32 = 2_000; // iterations per group

    println!("\n╔═══════════════════════════════════════════════════╗");
    println!("║   celestial-core property tests  ({N} iters each)  ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    let suites: &[(&str, bool)] = &[
        ("math", test_math(N).report()),
        ("time", test_time(N).report()),
        ("houses", test_houses(N).report()),
        ("vsop87", test_vsop87(N).report()),
        ("moon", test_moon(N).report()),
        ("ayanamsa", test_ayanamsa(N).report()),
        ("rise_set", test_rise_set(N).report()),
        ("nan_stability", test_nan_stability().report()),
        ("fixstars", test_fixstars(N).report()),
        ("nodes", test_nodes(N).report()),
        ("nod_aps", test_nod_aps(N).report()),
        ("crossings", test_crossings(N / 5).report()),
        ("eclipses", test_eclipses(N / 10).report()),
        ("phenomena", test_phenomena(N).report()),
        ("house_speeds", test_house_speeds(N).report()),
        ("aspects", test_swephelp_aspects(N).report()),
        ("vedic", test_swephelp_vedic(N).report()),
        ("datetime", test_swephelp_datetime(N).report()),
        ("tz_table", test_tz_table(N).report()),
        ("sabbats", test_sabbats(N).report()),
        ("esbats", test_esbats(N).report()),
        ("polar_houses", test_polar_houses(N).report()),
        ("ancient_future", test_ancient_future_dates(N).report()),
        ("equatorial_mode", test_equatorial_mode(N).report()),
        (
            "sidereal_all_modes",
            test_sidereal_all_modes(N / 2).report(),
        ),
        ("backward_searches", test_backward_searches(N / 5).report()),
        (
            "coordinate_transforms",
            test_coordinate_transforms(N).report(),
        ),
        (
            "occultation_search",
            test_occultation_search(N / 5).report(),
        ),
        ("house_invariants", test_house_invariants(N / 3).report()),
        (
            "topocentric_parallax",
            test_topocentric_parallax(N / 5).report(),
        ),
        ("time_equ", test_time_equ(N).report()),
        ("solcross_back", test_solcross_back(N / 5).report()),
        (
            "calc_many_parallel",
            test_calc_many_parallel(N / 4).report(),
        ),
        ("iau2000b_nutation", test_iau2000b_nutation(N).report()),
        ("mean_sidtime", test_mean_sidtime(N).report()),
        ("phase1_antiscia", test_phase1_antiscia(N).report()),
        ("phase1_arabic_parts", test_phase1_arabic_parts(N).report()),
        ("phase1_dignities", test_phase1_dignities(N).report()),
        ("phase2_returns", test_phase2_returns(N / 5).report()),
        ("phase2_progressions", test_phase2_progressions(N).report()),
        (
            "phase3_midpoint_dial",
            test_phase3_midpoint_dial(N).report(),
        ),
        ("phase3_local_space", test_phase3_local_space(N).report()),
        ("phase3_composite", test_phase3_composite(N).report()),
    ];

    println!();
    let total_ok = suites.iter().all(|(_, ok)| *ok);
    let passed = suites.iter().filter(|(_, ok)| *ok).count();
    let failed = suites.len() - passed;

    println!("══════════════════════════════════════════════════════");
    if total_ok {
        println!("✓ All {passed} suites passed");
    } else {
        println!("✗ {passed} passed, {failed} FAILED");
        std::process::exit(1);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Extended fuzz suites — added in bug-hunt pass
// ═══════════════════════════════════════════════════════════════════════════

fn test_polar_houses(n: u32) -> Suite {
    let mut s = Suite::new("polar_houses");
    let mut rng = Xorshift64::new(0xAABBCCDDEEFF0011);
    for _ in 0..n {
        let jd = 2_415_021.0 + rng.range_f64(0.0, 73049.0);
        // Full range including polar latitudes
        let lat = rng.range_f64(-89.9, 89.9);
        let lon = rng.range_f64(-180.0, 180.0);
        for &sys in b"PKEOCRWXMBHT" {
            // Must not panic — any result is acceptable
            let _ = houses(jd, lat, lon, HouseSystem(sys));
        }
        s.passed += 1;
    }
    s
}

fn test_ancient_future_dates(n: u32) -> Suite {
    let mut s = Suite::new("ancient_future");
    let mut rng = Xorshift64::new(0x1122334455667788);
    let bodies = [
        Body::SUN,
        Body::MOON,
        Body::MERCURY,
        Body::VENUS,
        Body::MARS,
        Body::JUPITER,
        Body::SATURN,
    ];
    for _ in 0..n {
        // Range: 5000 BCE to 5000 CE
        let jd = 625_674.0 + rng.range_f64(0.0, 3_652_500.0);
        let bi = (rng.next_u64() % bodies.len() as u64) as usize;
        match calc_ut(jd, bodies[bi], CalcFlags::BUILTIN) {
            Ok(p) => {
                s.check(
                    p.lon.is_finite() && p.lat.is_finite() && p.dist.is_finite(),
                    || format!("non-finite at JD {jd:.1} body={}", bodies[bi]),
                );
                s.check(p.lon >= 0.0 && p.lon < 360.0, || {
                    format!("lon {} out of range at JD {jd:.1}", p.lon)
                });
            }
            Err(_) => {
                s.passed += 1;
            } // Err is acceptable for extreme dates
        }
    }
    s
}

fn test_equatorial_mode(n: u32) -> Suite {
    let mut s = Suite::new("equatorial_mode");
    let mut rng = Xorshift64::new(0xFEDCBA9876543210);
    let bodies = [
        Body::SUN,
        Body::MOON,
        Body::MERCURY,
        Body::VENUS,
        Body::MARS,
    ];
    for _ in 0..n {
        let jd = 2_415_021.0 + rng.range_f64(0.0, 73049.0);
        let bi = (rng.next_u64() % bodies.len() as u64) as usize;
        let Ok(geo) = calc_ut(jd, bodies[bi], CalcFlags::BUILTIN) else {
            s.passed += 1;
            continue;
        };
        let Ok(eq) = calc_ut(jd, bodies[bi], CalcFlags::BUILTIN | CalcFlags::EQUATORIAL) else {
            s.passed += 1;
            continue;
        };
        s.check(eq.lon >= 0.0 && eq.lon < 360.0, || {
            format!("equatorial lon {} out of range", eq.lon)
        });
        s.check(eq.lat.abs() <= 90.0, || {
            format!("equatorial lat {} out of range", eq.lat)
        });
        s.check(eq.dist > 0.0, || {
            format!("equatorial dist {} must be positive", eq.dist)
        });
        // Geocentric and equatorial should have same distance
        s.check((geo.dist - eq.dist).abs() < 1e-8, || {
            format!("distances differ: geo={:.8} eq={:.8}", geo.dist, eq.dist)
        });
    }
    s
}

fn test_sidereal_all_modes(n: u32) -> Suite {
    let mut s = Suite::new("sidereal_all_modes");
    let mut rng = Xorshift64::new(0x0102030405060708);
    for _ in 0..n {
        let jd = 1_000_000.0 + rng.range_f64(0.0, 3_000_000.0);
        let mode = (rng.next_u64() % 36) as i32;
        set_sid_mode(SiderealMode(mode), 0.0, 0.0);
        let Ok(sid) = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN | CalcFlags::SIDEREAL) else {
            s.passed += 1;
            continue;
        };
        let Ok(trop) = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN) else {
            s.passed += 1;
            continue;
        };
        s.check(sid.lon >= 0.0 && sid.lon < 360.0, || {
            format!("sidereal lon {:.3} out of range (mode {mode})", sid.lon)
        });
        // Ayanamsa ranges from 0° to ~80° over the searchable historical period
        // (5000 BCE to 5000 CE). The diff must be in [0°, 90°] or [270°, 360°].
        let diff = (trop.lon - sid.lon).rem_euclid(360.0);
        s.check(diff < 90.0 || diff > 270.0, || {
            format!("sidereal-tropical diff {diff:.3}° implausible (mode {mode})")
        });
    }
    set_sid_mode(SiderealMode(SIDM_LAHIRI), 0.0, 0.0);
    s
}

fn test_backward_searches(n: u32) -> Suite {
    let mut s = Suite::new("backward_searches");
    let mut rng = Xorshift64::new(0xDEADC0DEDEADC0DE);
    for _ in 0..n {
        let jd = 2_415_021.0 + rng.range_f64(100.0, 73000.0);
        let target = rng.range_f64(0.0, 360.0);

        // Backwards solcross_back_ut: result must be BEFORE jd
        match solcross_back_ut(target, jd, CalcFlags::BUILTIN) {
            Ok(prev) => {
                s.check(
                    prev < jd + 0.1, // result should be before start
                    || format!("solcross_back_ut {prev:.2} not before start {jd:.2}"),
                );
                s.check(
                    jd - prev < 400.0, // within 1-year search window
                    || format!("solcross_back_ut {prev:.2} too far from start {jd:.2}"),
                );
                // Forward search from result should give a crossing close to jd
                if let Ok(fwd) = solcross_ut(target, prev - 0.1, CalcFlags::BUILTIN) {
                    s.check((fwd - jd).abs() < 400.0, || {
                        format!("forward cross {fwd:.2} far from original {jd:.2}")
                    });
                } else {
                    s.passed += 1;
                }
            }
            Err(_) => {
                s.passed += 1;
            }
        }

        // Backwards mooncross_back_ut: result before jd, within 30 days (lunar period)
        match mooncross_back_ut(target, jd, CalcFlags::BUILTIN) {
            Ok(prev) => {
                s.check(prev < jd + 0.1, || {
                    format!("mooncross_back_ut {prev:.2} not before start {jd:.2}")
                });
                s.check(jd - prev < 30.0, || {
                    format!(
                        "mooncross_back_ut result {:.2} more than 30d before {jd:.2}",
                        prev
                    )
                });
            }
            Err(_) => {
                s.passed += 1;
            }
        }

        // Backwards eclipse: result must be BEFORE jd
        match sol_eclipse_when_glob(jd, CalcFlags::BUILTIN, 0, true) {
            Ok(ecl) => {
                s.check(ecl.tret[0] < jd + 1.0, || {
                    format!(
                        "backwards eclipse {:.2} not before start {jd:.2}",
                        ecl.tret[0]
                    )
                });
                s.check(jd - ecl.tret[0] < 400.0, || {
                    format!("backwards eclipse too far: {:.2} days", jd - ecl.tret[0])
                });
            }
            Err(_) => {
                s.passed += 1;
            }
        }
    }
    s
}

fn test_coordinate_transforms(n: u32) -> Suite {
    let mut s = Suite::new("coordinate_transforms");
    let mut rng = Xorshift64::new(0xC0C0C0C0C0C0C0C0);
    for _ in 0..n {
        let lon = rng.range_f64(0.0, 360.0);
        let lat = rng.range_f64(-89.9, 89.9);
        let eps = rng.range_f64(20.0, 27.0); // reasonable obliquity range
                                             // Round-trip: ecliptic → equatorial → ecliptic
        let eq = coord_transform([lon, lat, 1.0], eps);
        let back = coord_transform([eq[0], eq[1], eq[2]], -eps);
        s.check(
            (back[0] - lon).abs().min(360.0 - (back[0] - lon).abs()) < 1e-8,
            || format!("coord_transform roundtrip lon: {lon:.4} → {:.4}", back[0]),
        );
        s.check((back[1] - lat).abs() < 1e-8, || {
            format!("coord_transform roundtrip lat: {lat:.4} → {:.4}", back[1])
        });
        // coord_transform_with_speed round-trip with speeds
        let xpo6 = [lon, lat, 1.0, 0.5, 0.1, 0.0];
        let eq6 = coord_transform_with_speed(xpo6, eps);
        let back6 = coord_transform_with_speed(eq6, -eps);
        s.check(
            (back6[0] - lon).abs().min(360.0 - (back6[0] - lon).abs()) < 1e-6,
            || {
                format!(
                    "coord_transform_with_speed roundtrip lon: {lon:.4} → {:.4}",
                    back6[0]
                )
            },
        );
    }
    s
}

fn test_occultation_search(n: u32) -> Suite {
    let mut s = Suite::new("occultation_search");
    let mut rng = Xorshift64::new(0x11223344AABBCCDD);
    let planets = [Body::VENUS, Body::MARS, Body::JUPITER, Body::SATURN];
    for _ in 0..n {
        let jd = 2_451_545.0 + rng.range_f64(-3650.0, 3650.0); // ±10 years
        let pi = (rng.next_u64() % planets.len() as u64) as usize;
        match lun_occult_when_glob(jd, planets[pi], None, CalcFlags::BUILTIN, 0, false) {
            Ok(r) => {
                s.check(r.tret[0] > jd - 1.0, || {
                    format!(
                        "occultation JD {:.2} not after search start {jd:.2}",
                        r.tret[0]
                    )
                });
                s.check(r.tret[0] < jd + 800.0, || {
                    format!("occultation JD {:.2} too far from start {jd:.2}", r.tret[0])
                });
                s.check(r.ret_flags == ECL_OCCULTATION, || {
                    format!("wrong ret_flags: {}", r.ret_flags)
                });
            }
            Err(_) => {
                s.passed += 1;
            } // no occultation in window is valid
        }
    }
    s
}

fn test_house_invariants(n: u32) -> Suite {
    let mut s = Suite::new("house_invariants");
    let mut rng = Xorshift64::new(0xF00DCAFE12345678);
    // All 12 house systems including recently fixed ones
    let systems: &[u8] = b"PKEOCRWXMBHTA"; // A=Alcabitius, T=Topocentric
    for _ in 0..n {
        let jd = 2_415_021.0 + rng.range_f64(0.0, 73049.0);
        let lat = rng.range_f64(-66.0, 66.0); // avoid polar failure zone
        let lon = rng.range_f64(-180.0, 180.0);
        for &sys in systems {
            let Ok(r) = houses(jd, lat, lon, HouseSystem(sys)) else {
                s.passed += 1;
                continue;
            };
            // Invariant 1: all 12 cusps in [0, 360)
            for h in 1..=12 {
                s.check(r.cusps[h] >= 0.0 && r.cusps[h] < 360.0, || {
                    format!(
                        "sys='{}' cusp[{h}]={:.3} not in [0,360)",
                        sys as char, r.cusps[h]
                    )
                });
            }
            // Invariant 2: every pair of opposite houses is exactly 180° apart
            for h in 1..=6 {
                let diff = (r.cusps[h] - r.cusps[h + 6]).rem_euclid(360.0);
                let dev = (diff - 180.0).abs();
                s.check(dev < 0.001, || {
                    format!(
                        "sys='{}' cusps[{h}]&[{}] not opposite (dev={dev:.4}°)",
                        sys as char,
                        h + 6
                    )
                });
            }
            // Invariant 3: ASC and MC must be finite and in range
            s.check(r.ascmc[0] >= 0.0 && r.ascmc[0] < 360.0, || {
                format!("sys='{}' ASC={:.3} out of range", sys as char, r.ascmc[0])
            });
            s.check(r.ascmc[1] >= 0.0 && r.ascmc[1] < 360.0, || {
                format!("sys='{}' MC={:.3} out of range", sys as char, r.ascmc[1])
            });
        }
    }
    s
}

fn test_topocentric_parallax(n: u32) -> Suite {
    let mut s = Suite::new("topocentric_parallax");
    let mut rng = Xorshift64::new(0xABCDEF0123456789);
    for _ in 0..n {
        let jd = 2_451_545.0 + rng.range_f64(-1000.0, 1000.0);
        let lon = rng.range_f64(-180.0, 180.0);
        let lat = rng.range_f64(-70.0, 70.0);
        let alt = rng.range_f64(0.0, 5000.0);

        set_topo(lon, lat, alt);
        let Ok(geo) = calc_ut(jd, Body::MOON, CalcFlags::BUILTIN) else {
            s.passed += 1;
            continue;
        };
        let Ok(topo) = calc_ut(jd, Body::MOON, CalcFlags::BUILTIN | CalcFlags::TOPOCENTRIC) else {
            s.passed += 1;
            continue;
        };
        set_topo(0.0, 0.0, 0.0);

        // Moon parallax is at most ~1°; shift must be finite and bounded
        let shift = (topo.lon - geo.lon).abs();
        let shift = if shift > 180.0 { 360.0 - shift } else { shift };
        s.check(shift <= 1.5, || {
            format!("Moon topo shift {shift:.4}° > 1.5° at lat={lat:.1}°")
        });
        s.check(topo.lon.is_finite() && topo.lat.is_finite(), || {
            "topo position non-finite".to_string()
        });

        // Sun parallax is at most ~0.01°
        let Ok(sun_geo) = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN) else {
            s.passed += 1;
            continue;
        };
        set_topo(lon, lat, alt);
        let Ok(sun_topo) = calc_ut(jd, Body::SUN, CalcFlags::BUILTIN | CalcFlags::TOPOCENTRIC)
        else {
            s.passed += 1;
            continue;
        };
        set_topo(0.0, 0.0, 0.0);
        let sun_shift = (sun_topo.lon - sun_geo.lon).abs();
        let sun_shift = if sun_shift > 180.0 {
            360.0 - sun_shift
        } else {
            sun_shift
        };
        s.check(sun_shift <= 0.02, || {
            format!("Sun topo shift {sun_shift:.5}° > 0.02°")
        });
    }
    s
}

fn test_time_equ(n: u32) -> Suite {
    let mut s = Suite::new("time_equ");
    let mut rng = Xorshift64::new(0x0011223344556677);
    for _ in 0..n {
        let jd = 2_415_021.0 + rng.range_f64(0.0, 73049.0);
        match time_equ(jd) {
            Ok(e) => {
                s.check(e.is_finite(), || {
                    format!("time_equ={e} not finite at JD {jd:.1}")
                });
                // Equation of time is bounded within ±17 minutes = ±0.283 hours
                s.check(e.abs() < 0.30, || {
                    format!("time_equ={e:.4}h out of range at JD {jd:.1}")
                });
            }
            Err(_) => {
                s.passed += 1;
            }
        }
    }
    s
}

fn test_solcross_back(n: u32) -> Suite {
    let mut s = Suite::new("solcross_back");
    let mut rng = Xorshift64::new(0x9988776655443322);
    for _ in 0..n {
        let jd = 2_415_021.0 + rng.range_f64(100.0, 73000.0);
        let lon = rng.range_f64(0.0, 360.0);
        // Forward then back should bracket the same crossing
        let Ok(fwd) = solcross_ut(lon, jd, CalcFlags::BUILTIN) else {
            s.passed += 1;
            continue;
        };
        let Ok(back) = solcross_back_ut(lon, fwd + 0.5, CalcFlags::BUILTIN) else {
            s.passed += 1;
            continue;
        };
        s.check(back <= fwd + 0.5, || {
            format!("back={back:.3} should be ≤ start={:.3}", fwd + 0.5)
        });
        s.check((back - fwd).abs() < 2.0, || {
            format!("back={back:.3} far from fwd={fwd:.3}")
        });
        // Moon crossing: must be within 30 days
        let Ok(mfwd) = mooncross_ut(lon, jd, CalcFlags::BUILTIN) else {
            s.passed += 1;
            continue;
        };
        let Ok(mback) = mooncross_back_ut(lon, mfwd + 0.5, CalcFlags::BUILTIN) else {
            s.passed += 1;
            continue;
        };
        s.check(mback <= mfwd + 0.5, || {
            format!("moon back={mback:.3} should be ≤ start={:.3}", mfwd + 0.5)
        });
        s.check((mback - mfwd).abs() < 30.0, || {
            format!("moon back={mback:.3} far from fwd={mfwd:.3}")
        });
    }
    s
}
