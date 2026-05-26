//! Shared helpers for the workspace's fuzz / property tests.
//!
//! Replaces the per-crate `Xorshift64` copies (was DUP-6) and exposes the
//! edge-case vectors that every parser/FFI seam is expected to survive:
//! null/empty/oversize strings, numeric extremes, NaN/Inf, surrogate code
//! points, and signed/unsigned overflow boundaries.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

/// Maximum string length the fuzzers will emit. Larger than any documented
/// CLI input field (longest known: `--ephe-path`, OS-bounded ~ 4 kiB).
pub const STR_LEN_MAX: usize = 8 * 1024;

/// Deterministic 64-bit xorshift PRNG.
///
/// Tiny, allocation-free, reproducible. Quality is "good enough for fuzzing
/// dumb parsers"; do not use for cryptography or statistics.
pub struct Xorshift64(u64);

impl Xorshift64 {
    /// Seeded ctor. Seed `0` would lock the state at zero → forced to odd.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    /// Next raw 64-bit word.
    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    /// Next `f64` in `[0, 1)` using the high 53 bits.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform `f64` in `[lo, hi)`.
    pub fn range_f64(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.next_f64() * (hi - lo)
    }

    /// Uniform `i32` in `[lo, hi)`. `lo < hi` required.
    pub fn range_i32(&mut self, lo: i32, hi: i32) -> i32 {
        let span = (hi as i64 - lo as i64) as u64;
        lo + (self.next_u64() % span) as i32
    }

    /// Uniform `i64` in `[lo, hi)`. `lo < hi` required.
    pub fn range_i64(&mut self, lo: i64, hi: i64) -> i64 {
        let span = hi.wrapping_sub(lo) as u64;
        lo + (self.next_u64() % span) as i64
    }
}

/// Random printable ASCII string of length `0..len_max`.
#[must_use]
pub fn random_string(rng: &mut Xorshift64, len_max: usize) -> String {
    if len_max == 0 {
        return String::new();
    }
    let len = (rng.next_u64() as usize) % len_max;
    let chars: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        0123456789 -:.,/+\xC2\xB0'\"NSEWnsew\t\n";
    (0..len)
        .map(|_| chars[(rng.next_u64() as usize) % chars.len()] as char)
        .collect()
}

/// Random byte buffer of length `0..len_max` — may contain any byte 0x00..=0xFF.
/// Useful to exercise utf-8 validation paths and null-byte handling.
#[must_use]
pub fn random_bytes(rng: &mut Xorshift64, len_max: usize) -> Vec<u8> {
    if len_max == 0 {
        return Vec::new();
    }
    let len = (rng.next_u64() as usize) % len_max;
    (0..len).map(|_| (rng.next_u64() & 0xFF) as u8).collect()
}

/// Edge-case strings every text parser must survive without panic.
///
/// Order is stable: tests are allowed to reference the row count.
pub const EDGE_STRINGS: &[&str] = &[
    "",                                                // null/empty
    " ",                                               // whitespace only
    "\t\n\r",                                          // control whitespace
    "\0",                                              // embedded null
    "abc\0def",                                        // null in the middle
    "\u{FEFF}",                                        // BOM
    "\u{200B}",                                        // zero-width space
    "ümlaut",                                          // non-ASCII
    "日本語",                                          // CJK
    "🌑🌒🌓🌔🌕",                                      // emoji (moon phases)
    "-1",                                              // negative
    "0",                                               // zero
    "1",                                               // one
    "9223372036854775807",                             // i64::MAX
    "-9223372036854775808",                            // i64::MIN
    "18446744073709551615",                            // u64::MAX
    "999999999999999999999999999999",                  // > u64
    "1e308",                                           // near f64::MAX
    "-1e308",                                          // near f64::MIN
    "1e-323",                                          // subnormal
    "NaN",                                             // NaN literal
    "inf",                                             // ±Inf
    "-inf",
    "0.0/0.0",                                         // div-by-zero phrasing
    "0x7fffffff",                                      // hex i32::MAX
    "-2147483648",                                     // i32::MIN
    "2147483647",                                      // i32::MAX
    "2147483648",                                      // i32::MAX + 1 (wraps)
];

/// Length tiers covering tiny / typical / max / over-max strings.
pub const EDGE_STR_LENS: &[usize] = &[0, 1, 2, 64, 1024, STR_LEN_MAX, STR_LEN_MAX + 1];

/// Numeric extremes every `parse_*` / `from_raw` must handle.
pub const EDGE_I32: &[i32] = &[
    i32::MIN,
    i32::MIN + 1,
    -1_000_000,
    -1,
    0,
    1,
    1_000_000,
    i32::MAX - 1,
    i32::MAX,
];

/// 64-bit extremes for fields that originate from JSON / FFI.
pub const EDGE_I64: &[i64] = &[i64::MIN, -1, 0, 1, i32::MAX as i64 + 1, i64::MAX];

/// `f64` extremes covering NaN, ±Inf, subnormals, and the ±360 boundary used
/// by every angular routine.
pub const EDGE_F64: &[f64] = &[
    f64::NAN,
    f64::INFINITY,
    f64::NEG_INFINITY,
    f64::MIN,
    f64::MIN_POSITIVE,
    -0.0,
    0.0,
    1.0,
    -1.0,
    f64::MAX,
    360.0,
    359.999_999_999_999_94,
    360.000_000_000_000_06,
    -360.0,
    720.0,
];

/// Build a string `n` bytes long. Used to probe the "string bigger than max
/// length" branch of every parser without burning RNG state.
#[must_use]
pub fn repeat_byte(n: usize, b: u8) -> String {
    let mut v = vec![b; n];
    // Make sure utf-8-valid: only ASCII bytes are passed.
    if b >= 0x80 {
        v.iter_mut().for_each(|c| *c = b'x');
    }
    String::from_utf8(v).expect("ascii is utf-8")
}
