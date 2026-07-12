//! Constant-time utility functions for side-channel attack resistance.
//!
//! All functions in this module execute in time independent of the input data,
//! preventing timing side-channel attacks that could leak secret weight values
//! or authentication tokens.

/// Constant-time equality comparison of two byte slices.
///
/// Returns `true` if and only if `a` and `b` have the same length and
/// identical contents.  The comparison always examines every byte of both
/// slices regardless of where the first difference is, preventing
/// timing-based inference of partial matches.
#[inline]
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Constant-time conditional selection between two `i32` values.
///
/// If `condition` is `true`, returns `a`; otherwise returns `b`.
/// Both branches are always evaluated and the result is selected
/// via bitwise masking to avoid data-dependent branches.
#[inline]
pub fn ct_select_i32(condition: bool, a: i32, b: i32) -> i32 {
    // Convert bool to an all-ones or all-zeros mask without branching.
    // `condition as i32` gives 0 or 1; negating gives 0 or -1 (0xFFFFFFFF).
    let mask = -(condition as i32); // 0x00000000 or 0xFFFFFFFF
    (mask & a) | (!mask & b)
}

/// Constant-time conditional mask: returns `value` if `condition` is true,
/// otherwise 0.
#[inline]
#[allow(dead_code)]
pub fn ct_mask_i32(condition: bool, value: i32) -> i32 {
    let mask = -(condition as i32);
    mask & value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_eq_identical() {
        let a = [1u8, 2, 3, 4, 5];
        let b = [1u8, 2, 3, 4, 5];
        assert!(ct_eq(&a, &b));
    }

    #[test]
    fn test_ct_eq_different() {
        let a = [1u8, 2, 3, 4, 5];
        let b = [1u8, 2, 3, 4, 6];
        assert!(!ct_eq(&a, &b));
    }

    #[test]
    fn test_ct_eq_different_lengths() {
        let a = [1u8, 2, 3];
        let b = [1u8, 2, 3, 4];
        assert!(!ct_eq(&a, &b));
    }

    #[test]
    fn test_ct_select_true() {
        assert_eq!(ct_select_i32(true, 42, 99), 42);
    }

    #[test]
    fn test_ct_select_false() {
        assert_eq!(ct_select_i32(false, 42, 99), 99);
    }

    #[test]
    fn test_ct_mask_true() {
        assert_eq!(ct_mask_i32(true, 77), 77);
    }

    #[test]
    fn test_ct_mask_false() {
        assert_eq!(ct_mask_i32(false, 77), 0);
    }
}
