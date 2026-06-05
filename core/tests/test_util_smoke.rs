//! Smoke tests for the shared `celestial-test-util` helpers (DUP-6).
//!
//! Asserts every public surface is reachable from a downstream crate and
//! that the PRNG / edge vectors satisfy their documented contracts.

use celestial_test_util::{
    random_bytes, random_string, repeat_byte, Xorshift64, EDGE_F64, EDGE_I32, EDGE_I64,
    EDGE_STRINGS, EDGE_STR_LENS, STR_LEN_MAX,
};

#[test]
fn xorshift_is_deterministic_and_non_trivial() {
    let mut a = Xorshift64::new(42);
    let mut b = Xorshift64::new(42);
    for _ in 0..1_000 {
        assert_eq!(
            a.next_u64(),
            b.next_u64(),
            "PRNG not deterministic for seed=42"
        );
    }
    // Zero seed must not lock the state.
    let mut z = Xorshift64::new(0);
    assert_ne!(z.next_u64(), 0);
    assert_ne!(z.next_u64(), 0);
}

#[test]
fn xorshift_f64_in_unit_interval() {
    let mut rng = Xorshift64::new(0xC0FFEE);
    for _ in 0..10_000 {
        let x = rng.next_f64();
        assert!((0.0..1.0).contains(&x), "out of [0,1): {x}");
    }
}

#[test]
fn range_helpers_respect_bounds() {
    let mut rng = Xorshift64::new(1);
    for _ in 0..1_000 {
        let f = rng.range_f64(-10.0, 10.0);
        assert!((-10.0..10.0).contains(&f), "f={f}");
        let i = rng.range_i32(-5, 5);
        assert!((-5..5).contains(&i), "i={i}");
        let l = rng.range_i64(i64::MIN, i64::MIN + 1_000_000);
        assert!((i64::MIN..i64::MIN + 1_000_000).contains(&l), "l={l}");
    }
}

#[test]
fn random_string_bounded_by_len_max() {
    let mut rng = Xorshift64::new(2);
    for cap in [0usize, 1, 10, 1024, STR_LEN_MAX] {
        let s = random_string(&mut rng, cap.max(1)); // len_max=0 explicitly returns ""
        assert!(s.len() <= cap.max(1), "len {} > {}", s.len(), cap);
    }
    // The `len_max == 0` short-circuit.
    let mut empty_rng = Xorshift64::new(3);
    assert!(random_string(&mut empty_rng, 0).is_empty());
}

#[test]
fn random_bytes_bounded_and_any_byte() {
    let mut rng = Xorshift64::new(4);
    let mut seen = [false; 256];
    for _ in 0..1_000 {
        for b in random_bytes(&mut rng, 32) {
            seen[b as usize] = true;
        }
    }
    let coverage = seen.iter().filter(|&&x| x).count();
    // Loose bound — we shouldn't fail flakily, just want broad byte coverage.
    assert!(coverage > 200, "byte coverage only {coverage}");
    assert!(random_bytes(&mut rng, 0).is_empty());
}

#[test]
fn repeat_byte_produces_ascii() {
    assert_eq!(repeat_byte(0, b'x'), "");
    assert_eq!(repeat_byte(5, b'a'), "aaaaa");
    // Non-ASCII gets coerced to 'x' so the result is valid utf-8.
    assert!(repeat_byte(8, 0xFF).chars().all(|c| c == 'x'));
}

#[test]
fn edge_vectors_are_non_empty() {
    // Table-driven so CC stays at 2 even as the vector list grows.
    let lens: [(&str, usize); 5] = [
        ("EDGE_STRINGS", EDGE_STRINGS.len()),
        ("EDGE_STR_LENS", EDGE_STR_LENS.len()),
        ("EDGE_I32", EDGE_I32.len()),
        ("EDGE_I64", EDGE_I64.len()),
        ("EDGE_F64", EDGE_F64.len()),
    ];
    for (name, n) in lens {
        assert!(n > 0, "{name} empty");
    }
}

#[test]
fn edge_i32_contains_required_anchors() {
    let required = [-1, 0, 1, i32::MIN, i32::MAX];
    for n in required {
        assert!(EDGE_I32.contains(&n), "EDGE_I32 missing anchor {n}");
    }
}

#[test]
fn edge_str_lens_and_f64_anchors_present() {
    assert!(EDGE_STR_LENS.contains(&0), "EDGE_STR_LENS missing 0");
    assert!(EDGE_STR_LENS.contains(&STR_LEN_MAX), "missing STR_LEN_MAX");
    assert!(EDGE_F64.iter().any(|f| f.is_nan()), "EDGE_F64 missing NaN");
    assert!(
        EDGE_F64.iter().any(|f| f.is_infinite()),
        "EDGE_F64 missing Inf"
    );
}
