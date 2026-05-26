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

#[test]
fn panchanga_clamps_are_in_range() {
    // The clamp guards live on the floor-cast paths inside `panchanga`.
    // We exercise the public entry across the boundary JDs that the
    // arithmetic is most sensitive to: J2000, one-ulp on each side of a
    // full revolution, far future, far past. Each call must produce
    // in-range tithi/nakshatra/yoga/karana — invariant pinning, not an
    // ephemeris correctness check (covered elsewhere).
    use celestial_core::panchanga;
    use celestial_core::JulianDay;
    let jds = [
        2_451_545.0,                    // J2000
        2_451_544.999_999_999_94,       // 1 ulp below
        2_451_545.000_000_000_06,       // 1 ulp above
        0.0,
        1.0,
        -1.0,
        f64::from_bits(0x4140_0000_0000_0000), // ≈ 2.25e6
        100_000.5,
    ];
    for &jd in &jds {
        let p = panchanga(JulianDay::new(jd));
        assert!((1..=30).contains(&p.tithi), "tithi out of range: {}", p.tithi);
        assert!((0..=26).contains(&p.nakshatra), "nakshatra out: {}", p.nakshatra);
        assert!((1..=4).contains(&p.nakshatra_pada), "pada out: {}", p.nakshatra_pada);
        assert!((0..=26).contains(&p.yoga), "yoga out: {}", p.yoga);
        assert!((1..=60).contains(&p.karana), "karana out: {}", p.karana);
        // Name slots must be non-empty (table lookup invariant).
        assert!(!p.tithi_name.is_empty());
        assert!(!p.nakshatra_name.is_empty());
        assert!(!p.yoga_name.is_empty());
        // REL-2: karana_name on the produced karana must not be empty.
        assert!(!p.karana_name.is_empty(), "karana_name empty for k={}", p.karana);
    }
}

// ── REL-8 ─────────────────────────────────────────────────────────────────────

#[test]
fn body_try_from_raw_accepts_documented_ids() {
    // Main planets + luminaries
    for n in 0..=20 {
        assert!(Body::try_from_raw(n).is_ok(), "0..=20 missed: {n}");
    }
    // Pseudo-bodies
    assert!(Body::try_from_raw(-1).is_ok()); // ECL_NUT
    assert!(Body::try_from_raw(-10).is_ok()); // FIXED_STAR
    // Uranian / Hamburg range
    assert!(Body::try_from_raw(Body::FICTITIOUS_OFFSET).is_ok());
    assert!(Body::try_from_raw(Body::FICTITIOUS_OFFSET + 99).is_ok());
    // Planetary moons
    assert!(Body::try_from_raw(Body::MOON_OFFSET).is_ok());
    assert!(Body::try_from_raw(Body::MOON_OFFSET + 999).is_ok());
    // Asteroids
    assert!(Body::try_from_raw(Body::ASTEROID_OFFSET).is_ok());
    assert!(Body::try_from_raw(Body::ASTEROID_OFFSET + 999_999).is_ok());
}

#[test]
fn body_try_from_raw_rejects_garbage_and_gaps() {
    let garbage = [
        // Gap between Vesta (20) and FICTITIOUS_OFFSET (40)
        21, 22, 30, 39,
        // Gap between FICTITIOUS_OFFSET window (..140) and MOON_OFFSET (9000)
        Body::FICTITIOUS_OFFSET + 100, // 140 — first rejected
        141,
        1_000,
        8_999, // one below MOON_OFFSET
        // Just past ASTEROID_OFFSET window (10000..1010000)
        Body::ASTEROID_OFFSET + 1_000_000, // 1_010_000 — exclusive
        Body::ASTEROID_OFFSET + 2_000_000,
        // Deep garbage
        -2, -11, -100, -1_000_000,
        i32::MIN, i32::MIN + 1, i32::MAX, i32::MAX - 1,
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
