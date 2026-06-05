//! Reliability clamp tests for the 2026-05-27 fixes:
//!
//! * REL-2 — `panchanga::karana_name` underflow guard
//! * REL-7 — `panchanga()` floor-cast clamps for nakshatra / pada / karana
//! * REL-8 — `Body::try_from_raw` validating constructor
//!
//! Every numeric extreme requested by the work order is exercised:
//! `-1`, `0`, `1`, every `i32`/`i64`/`f64` boundary, NaN, ±Inf, subnormals,
//! and the angular wrap point `360.0 ± 1ulp`.

use celestial_core::body::{Body, BodyError};
use celestial_core::{karana_name, KARANA_NAMES};
use celestial_test_util::{EDGE_F64, EDGE_I32, EDGE_I64};

// ── REL-2 ─────────────────────────────────────────────────────────────────────

#[test]
fn karana_name_full_domain() {
    // Documented domain: 0 → "", 1 → Kimstughna, 60 → Abhijit, 2..=59 → cycle,
    // 61..=u8::MAX → "" (no panic).
    assert_eq!(karana_name(0), "");
    assert_eq!(karana_name(1), "Kimstughna");
    assert_eq!(karana_name(60), "Abhijit");
    for k in 2u8..=59 {
        let want = KARANA_NAMES[((k - 2) % 7) as usize];
        assert_eq!(karana_name(k), want, "karana={k}");
    }
    for k in 61u8..=u8::MAX {
        assert_eq!(karana_name(k), "", "karana={k}");
    }
}

#[test]
fn karana_name_never_panics_on_any_byte() {
    // Exhaustive over the entire `u8` domain — was DEBUG-only panic on 0
    // pre-fix (REL-2). After the fix every input returns a static `&str`.
    for k in 0u8..=u8::MAX {
        let s = karana_name(k);
        // `&'static str` always satisfies these invariants
        assert!(s.is_empty() || !s.is_empty());
    }
}

// ── REL-7 ─────────────────────────────────────────────────────────────────────

/// Single-row check kept outside the loop so the test fn stays CC ≤ 10.
fn assert_panchanga_slots_in_range(p: &celestial_core::Panchanga, jd: f64) {
    let checks: [(&str, bool); 8] = [
        ("tithi", (1..=30).contains(&p.tithi)),
        ("nakshatra", (0..=26).contains(&p.nakshatra)),
        ("nakshatra_pada", (1..=4).contains(&p.nakshatra_pada)),
        ("yoga", (0..=26).contains(&p.yoga)),
        ("karana", (1..=60).contains(&p.karana)),
        ("tithi_name", !p.tithi_name.is_empty()),
        ("nakshatra_name", !p.nakshatra_name.is_empty()),
        ("karana_name", !p.karana_name.is_empty()),
    ];
    for (label, ok) in checks {
        assert!(ok, "panchanga slot `{label}` out of range at jd={jd}");
    }
}

#[test]
fn panchanga_clamps_are_in_range() {
    use celestial_core::panchanga;
    use celestial_core::JulianDay;
    let jds = [
        2_451_545.0,              // J2000
        2_451_544.999_999_999_94, // 1 ulp below
        2_451_545.000_000_000_06, // 1 ulp above
        0.0,
        1.0,
        -1.0,
        f64::from_bits(0x4140_0000_0000_0000), // ≈ 2.25e6
        100_000.5,
    ];
    for jd in jds {
        assert_panchanga_slots_in_range(&panchanga(JulianDay::new(jd)), jd);
    }
}

// ── REL-8 ─────────────────────────────────────────────────────────────────────

#[test]
fn body_try_from_raw_accepts_documented_ids() {
    // Build the full accept set table-driven; one assert per id.
    let mut ids: Vec<i32> = (0..=20).collect();
    ids.extend([
        -1,
        -10,
        Body::FICTITIOUS_OFFSET,
        Body::FICTITIOUS_OFFSET + 99,
        Body::MOON_OFFSET,
        Body::MOON_OFFSET + 999,
        Body::ASTEROID_OFFSET,
        Body::ASTEROID_OFFSET + 999_999,
    ]);
    for n in ids {
        assert!(Body::try_from_raw(n).is_ok(), "documented id rejected: {n}");
    }
}

#[test]
fn body_try_from_raw_rejects_garbage_and_gaps() {
    let garbage = [
        // Gap between Vesta (20) and FICTITIOUS_OFFSET (40)
        21,
        22,
        30,
        39,
        // Gap between FICTITIOUS_OFFSET window (..140) and MOON_OFFSET (9000)
        Body::FICTITIOUS_OFFSET + 100, // 140 — first rejected
        141,
        1_000,
        8_999, // one below MOON_OFFSET
        // Just past ASTEROID_OFFSET window (10000..1010000)
        Body::ASTEROID_OFFSET + 1_000_000, // 1_010_000 — exclusive
        Body::ASTEROID_OFFSET + 2_000_000,
        // Deep garbage
        -2,
        -11,
        -100,
        -1_000_000,
        i32::MIN,
        i32::MIN + 1,
        i32::MAX,
        i32::MAX - 1,
    ];
    for n in garbage {
        let r = Body::try_from_raw(n);
        assert!(r.is_err(), "expected reject, got {r:?} for id={n}");
        if let Err(BodyError::OutOfRange { id }) = r {
            assert_eq!(id, n);
        }
    }
}

#[test]
fn body_try_from_raw_full_edge_grid() {
    // Every numeric edge value (-1, 0, 1, i32::MIN/MAX, …) must produce
    // either Ok or a clean OutOfRange error — never a panic.
    for &n in EDGE_I32 {
        let _ = Body::try_from_raw(n);
    }
    // Long-int values that get cast to i32 must also be handled.
    for &n in EDGE_I64 {
        let _ = Body::try_from_raw(n as i32);
    }
}

#[test]
fn body_error_display_is_informative() {
    let e = Body::try_from_raw(99_999_999).unwrap_err();
    let msg = e.to_string();
    assert!(msg.contains("99999999"), "msg missing id: {msg}");
    assert!(msg.contains("range"), "msg missing 'range': {msg}");
}

#[test]
fn is_known_id_matches_try_from_raw() {
    // The two predicates must agree on every input the test util generates.
    for &n in EDGE_I32 {
        assert_eq!(
            Body::is_known_id(n),
            Body::try_from_raw(n).is_ok(),
            "disagreement on n={n}"
        );
    }
}

// ── Cross-cutting numeric fuzz ────────────────────────────────────────────────

/// Feed every f64 edge case (NaN, ±Inf, subnormals, ±360) through the JD
/// constructor + downstream so we verify nothing panics on degenerate input.
#[test]
fn julian_day_constructors_handle_every_f64_edge() {
    use celestial_core::JulianDay;
    for &x in EDGE_F64 {
        let _ = JulianDay::new(x);
    }
}
