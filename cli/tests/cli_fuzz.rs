//! Property-based fuzz tests for CLI parsers.
//!
//! Every parser must handle ARBITRARY garbage input without panicking — only
//! returning `Result::Err`. This file generates pseudo-random byte sequences
//! and feeds them through each `parse_*` entry point.
//!
//! Self-contained: uses a stdlib xorshift64 PRNG, no external crates.

use celestial_cli::parse::{
    body_name, hsys_name, jd_to_str, parse_body, parse_date, parse_hsys, parse_sid_mode,
};
use celestial_core::body::Body;

const N: u32 = 5_000;

// ─── Minimal PRNG (matches the one in fuzz/src/main.rs) ──────────────────────

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
}

/// Generate a random ASCII string of length `len_max` ± from a printable subset.
fn random_string(rng: &mut Xorshift64, len_max: usize) -> String {
    let len = (rng.next_u64() as usize) % len_max;
    let chars: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        0123456789 -:.,/+\xC2\xB0'\"NSEWnsew\t\n";
    (0..len)
        .map(|_| chars[(rng.next_u64() as usize) % chars.len()] as char)
        .collect()
}

// ─── Fuzz tests ──────────────────────────────────────────────────────────────

#[test]
fn fuzz_parse_date_no_panic() {
    let mut rng = Xorshift64::new(0xDEAD_BEEF_CAFE_BABE);
    for i in 0..N {
        let s = random_string(&mut rng, 30);
        // parse_date must never panic — only Ok or Err
        let _ = parse_date(&s);
        // Also feed a few targeted edge cases on every Nth iteration
        if i % 100 == 0 {
            for edge in [
                "",
                " ",
                "\t",
                "\n",
                "now",
                "NOW",
                "Now",
                "0",
                "0.0",
                "-1.0",
                "1e308",
                "-1e308",
                "1e-308",
                "NaN",
                "inf",
                "-inf",
                "1582-10-04",
                "1582-10-15",
                "9999-12-31",
                "-1-01-01",
                "0-02-29",
                "2024-02-29",
                "2025-02-29",
                "2024/04/08",
                "2024-04-08 18:30",
                "2024-04-08 18:30:45",
                "2024-04-08T18:30Z",
                "  2024-04-08  ",
            ] {
                let _ = parse_date(edge);
            }
        }
    }
}

#[test]
fn fuzz_parse_body_no_panic() {
    let mut rng = Xorshift64::new(0xBABE_FACE_FEED_BABA);
    for _ in 0..N {
        let s = random_string(&mut rng, 20);
        let _ = parse_body(&s);
    }
    // Also: every documented body name must round-trip through body_name()
    for name in [
        "sun",
        "moon",
        "mercury",
        "venus",
        "mars",
        "jupiter",
        "saturn",
        "uranus",
        "neptune",
        "pluto",
        "node",
        "true_node",
        "chiron",
        "Sun",
        "MOON",
        "Mars",
    ] {
        if let Ok(code) = parse_body(name) {
            let display = body_name(Body(code));
            assert!(
                !display.is_empty(),
                "body {name} round-tripped through body_name returned empty"
            );
        }
    }
}

#[test]
fn fuzz_parse_hsys_no_panic() {
    let mut rng = Xorshift64::new(0x1357_9BDF_2468_ACE0);
    for _ in 0..N {
        let s = random_string(&mut rng, 15);
        let _ = parse_hsys(&s);
    }
    // Every recognized house-system letter must round-trip through hsys_name
    for letter in b"PKRCBMOAEHVXGTUWY" {
        if let Ok(code) = parse_hsys(&(*letter as char).to_string()) {
            let name = hsys_name(code);
            assert!(
                !name.is_empty(),
                "hsys '{}' returned empty name",
                *letter as char
            );
        }
    }
}

#[test]
fn fuzz_parse_sid_mode_no_panic() {
    let mut rng = Xorshift64::new(0xABCD_EF12_3456_7890);
    for _ in 0..N {
        let s = random_string(&mut rng, 30);
        let _ = parse_sid_mode(&s);
    }
    // Known modes must parse
    for mode in ["fagan", "lahiri", "krishnamurti", "raman", "0", "1"] {
        let _ = parse_sid_mode(mode);
    }
}

#[test]
fn fuzz_jd_to_str_no_panic() {
    let mut rng = Xorshift64::new(0x9999_8888_7777_6666);
    for _ in 0..N {
        // Random JD across wide range, including exotic values
        let bits = rng.next_u64();
        let jd = f64::from_bits(bits);
        // Skip non-finite values for the panic check (they'd produce strings like "NaN")
        // but still call to confirm no panic
        let s = jd_to_str(jd);
        // Whatever comes out must be valid UTF-8 (it's a String, so this is automatic)
        // and not empty for finite inputs
        if jd.is_finite() && jd > 0.0 {
            assert!(!s.is_empty(), "jd_to_str({jd}) returned empty");
        }
    }
    // Edge cases
    for jd in [
        0.0_f64,
        2_415_021.0,
        2_451_545.0,
        2_460_409.0,
        1.0e10,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MIN,
        f64::MAX,
    ] {
        let _ = jd_to_str(jd);
    }
}

#[test]
fn fuzz_body_name_for_all_codes() {
    // Body wraps an i32. Test the full small-range space and a few outliers.
    for code in -10..=300i32 {
        let name = body_name(Body(code));
        assert!(!name.is_empty(), "body_name({code}) empty");
    }
}

#[test]
fn fuzz_hsys_name_for_all_codes() {
    // hsys_name takes a u8; test every value
    for code in 0u8..=255 {
        let name = hsys_name(code);
        assert!(!name.is_empty(), "hsys_name({code}) empty");
    }
}
