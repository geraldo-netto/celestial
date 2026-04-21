//! Self-contained micro-benchmarks for celestial-core.
//!
//! Uses std::time — no external dependencies, works on Rust 1.75+.
//! Run:  cargo bench --package celestial-core

#![allow(unused_must_use)]
use celestial_core::body::{Body, CalcFlags, Calendar, HouseSystem};
use std::hint::black_box;
use std::time::Instant;

const J2000: f64 = 2451545.0;
const JD_RECENT: f64 = 2460000.0;

struct R {
    name: String,
    ns: f64,
    sd: f64,
}

fn bench<F: FnMut()>(name: &str, n: u32, mut f: F) -> R {
    for _ in 0..n / 10 {
        f();
    }
    let mut s = Vec::with_capacity(n as usize);
    for _ in 0..n {
        let t0 = Instant::now();
        f();
        s.push(t0.elapsed().as_nanos() as f64);
    }
    let mean = s.iter().sum::<f64>() / s.len() as f64;
    let sd = (s.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / s.len() as f64).sqrt();
    R {
        name: name.into(),
        ns: mean,
        sd,
    }
}

fn pr(r: &R) {
    println!(
        "test {:55} ... bench: {:>10.0} ns/iter (+/- {:.0})",
        r.name, r.ns, r.sd
    );
}

fn hdr(g: &str) {
    println!("\n── {g}");
}

fn main() {
    let f = CalcFlags::BUILTIN | CalcFlags::SPEED;
    let fb = CalcFlags::BUILTIN;
    let mut all: Vec<R> = Vec::new();

    hdr("time");
    all.push(bench("time::julday_gregorian", 1000, || {
        black_box(celestial_core::julday(
            2025,
            3,
            20,
            12.0,
            Calendar::Gregorian,
        ));
    }));
    all.push(bench("time::revjul_gregorian", 1000, || {
        black_box(celestial_core::revjul(
            black_box(J2000),
            Calendar::Gregorian,
        ));
    }));
    all.push(bench("time::jd_et_to_utc", 1000, || {
        black_box(celestial_core::jd_et_to_utc(
            black_box(J2000),
            Calendar::Gregorian,
        ));
    }));
    all.push(bench("time::deltat", 1000, || {
        black_box(celestial_core::deltat(black_box(J2000)));
    }));
    all.push(bench("time::sidtime", 1000, || {
        black_box(celestial_core::sidtime(black_box(J2000)));
    }));
    all.push(bench("time::jd_to_iso_string", 1000, || {
        black_box(celestial_core::jd_to_iso_string(
            black_box(J2000),
            Calendar::Gregorian,
        ));
    }));

    hdr("math");
    all.push(bench("math::norm_deg", 5000, || {
        black_box(celestial_core::norm_deg(black_box(725.5)));
    }));
    all.push(bench("math::diff_deg_signed", 5000, || {
        black_box(celestial_core::diff_deg_signed(
            black_box(350.0),
            black_box(10.0),
        ));
    }));
    all.push(bench("math::midpoint_deg", 5000, || {
        black_box(celestial_core::midpoint_deg(
            black_box(350.0),
            black_box(10.0),
        ));
    }));
    all.push(bench("math::split_deg", 5000, || {
        black_box(celestial_core::split_deg(black_box(123.456), black_box(0)));
    }));
    all.push(bench("math::coord_transform", 5000, || {
        black_box(celestial_core::coord_transform(
            black_box([123.456, 43.5, 1.0]),
            black_box(23.4),
        ));
    }));
    all.push(bench("math::lon_to_sign", 5000, || {
        black_box(celestial_core::lon_to_sign(black_box(123.456)));
    }));

    hdr("calc");
    for (name, body) in [
        ("sun", Body::SUN),
        ("moon", Body::MOON),
        ("mercury", Body::MERCURY),
        ("venus", Body::VENUS),
        ("mars", Body::MARS),
        ("jupiter", Body::JUPITER),
        ("saturn", Body::SATURN),
        ("uranus", Body::URANUS),
        ("neptune", Body::NEPTUNE),
        ("pluto", Body::PLUTO),
        ("chiron", Body::CHIRON),
    ] {
        all.push(bench(&format!("calc::calc_ut_{name}"), 500, || {
            black_box(celestial_core::calc_ut(
                black_box(J2000),
                black_box(body),
                black_box(f),
            ));
        }));
    }
    all.push(bench("calc::calc_all_10_planets", 200, || {
        for body in [
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
        ] {
            black_box(celestial_core::calc_ut(
                black_box(J2000),
                body,
                black_box(f),
            ));
        }
    }));

    hdr("houses");
    let (lat, lon) = (48.85_f64, 2.35_f64);
    for (name, code) in [
        ("placidus", b'P'),
        ("koch", b'K'),
        ("equal", b'E'),
        ("whole_sign", b'W'),
        ("regiomontanus", b'R'),
        ("campanus", b'C'),
    ] {
        all.push(bench(&format!("houses::houses_ex_{name}"), 500, || {
            black_box(celestial_core::houses_ex(
                black_box(J2000),
                black_box(fb),
                black_box(lat),
                black_box(lon),
                black_box(HouseSystem(code)),
            ));
        }));
    }

    hdr("calendar");
    all.push(bench("calendar::easter_gregorian", 1000, || {
        black_box(celestial_core::easter_gregorian(2025));
    }));
    all.push(bench("calendar::easter_orthodox", 1000, || {
        black_box(celestial_core::easter_orthodox(2025));
    }));
    all.push(bench("calendar::jewish_holidays_5785", 1000, || {
        black_box(celestial_core::jewish_holidays(5785));
    }));
    all.push(bench("calendar::hijri_from_jd", 1000, || {
        black_box(celestial_core::hijri_from_jd(black_box(J2000)));
    }));

    hdr("vedic");
    all.push(bench("vedic::long_to_nakshatra", 5000, || {
        black_box(celestial_core::long_to_nakshatra(black_box(123.456)));
    }));
    all.push(bench("vedic::long_to_navamsa", 5000, || {
        black_box(celestial_core::long_to_navamsa(black_box(123.456)));
    }));
    all.push(bench("vedic::panchanga", 200, || {
        black_box(celestial_core::panchanga(black_box(J2000)));
    }));
    all.push(bench("vedic::vimshottari_dasha", 500, || {
        black_box(celestial_core::vimshottari_dasha(
            black_box(J2000),
            black_box(210.0),
            black_box(120.0),
        ));
    }));

    hdr("searches (10 iters each)");
    all.push(bench("searches::solcross_ut_0deg", 10, || {
        black_box(celestial_core::solcross_ut(
            black_box(0.0),
            black_box(JD_RECENT),
            black_box(fb),
        ));
    }));
    all.push(bench("searches::mooncross_ut_0deg", 10, || {
        black_box(celestial_core::mooncross_ut(
            black_box(0.0),
            black_box(JD_RECENT),
            black_box(fb),
        ));
    }));
    all.push(bench("searches::sol_eclipse_when_glob", 10, || {
        black_box(celestial_core::sol_eclipse_when_glob(
            black_box(JD_RECENT),
            black_box(fb),
            0,
            false,
        ));
    }));
    all.push(bench("searches::lun_eclipse_when", 10, || {
        black_box(celestial_core::lun_eclipse_when(
            black_box(JD_RECENT),
            black_box(fb),
            0,
            false,
        ));
    }));
    all.push(bench("searches::sign_ingress_saturn", 10, || {
        black_box(celestial_core::sign_ingress_ut(
            black_box(Body::SATURN),
            black_box(JD_RECENT),
            black_box(fb),
            false,
        ));
    }));
    all.push(bench("searches::solar_return_2025", 10, || {
        black_box(celestial_core::solar_return_jd(
            black_box(J2000),
            2025,
            black_box(fb),
        ));
    }));

    println!("\n{:═<78}", "");
    println!("RESULTS ({} benchmarks)", all.len());
    println!("{:═<78}", "");
    for r in &all {
        pr(r);
    }

    let fastest = all
        .iter()
        .min_by(|a, b| a.ns.partial_cmp(&b.ns).unwrap())
        .unwrap();
    let slowest = all
        .iter()
        .max_by(|a, b| a.ns.partial_cmp(&b.ns).unwrap())
        .unwrap();
    let total: f64 = all.iter().map(|r| r.ns).sum();
    println!("\n── Summary");
    println!("  Fastest : {} ({:.0} ns)", fastest.name, fastest.ns);
    println!(
        "  Slowest : {} ({:.0} ns / {:.2} ms)",
        slowest.name,
        slowest.ns,
        slowest.ns / 1e6
    );
    println!("  Total   : {:.1} µs per full sweep", total / 1000.0);
}
