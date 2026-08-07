pub(crate) fn f64_fingerprint(values: impl IntoIterator<Item = f64>) -> u64 {
    values
        .into_iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, value| {
            (hash ^ value.to_bits()).wrapping_mul(0x100_0000_01b3)
        })
}

#[test]
fn fingerprint_is_order_and_bit_sensitive() {
    assert_eq!(f64_fingerprint([1.0, -0.0]), 0x22c2_8807_b4eb_6fed);
    assert_ne!(f64_fingerprint([1.0, 0.0]), f64_fingerprint([1.0, -0.0]));
    assert_ne!(f64_fingerprint([-0.0, 1.0]), f64_fingerprint([1.0, -0.0]));
}
