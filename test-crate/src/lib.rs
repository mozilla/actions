//! Minimal test crate for exercising mozilla-actions CI workflows.
//!
//! This crate provides simple utility functions with enough structure
//! to exercise clippy, rustfmt, cargo-deny, cargo-machete, cargo-mutants,
//! and sanitizer workflows.

#[cfg(feature = "encoding")]
mod encode;
#[cfg(feature = "hashing")]
mod hash;

#[cfg(feature = "hashing")]
pub use hash::crc32;

#[cfg(feature = "encoding")]
pub use encode::encode_base64;

/// Categorize a value into a named bucket.
///
/// Returns `"negative"` for values below zero, `"zero"` for zero,
/// `"small"` for 1..=100, and `"large"` for everything above 100.
pub fn categorize(value: i64) -> &'static str {
    if value < 0 {
        "negative"
    } else if value == 0 {
        "zero"
    } else if value <= 100 {
        "small"
    } else {
        "large"
    }
}

/// Compute the greatest common divisor of two non-negative integers.
///
/// # Panics
///
/// Panics if both `a` and `b` are zero.
pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    assert!(a != 0 || b != 0, "gcd(0, 0) is undefined");
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// A trait for objects that can validate themselves.
pub trait Validate {
    /// Returns `true` if the object is in a valid state.
    fn is_valid(&self) -> bool;
}

/// A value clamped to lie within `[min, max]`.
pub struct Bounded {
    /// The current value.
    pub value: i64,
    /// The minimum allowed value.
    pub min: i64,
    /// The maximum allowed value.
    pub max: i64,
}

impl Bounded {
    /// Create a new [`Bounded`] value, clamping to the given range.
    pub fn new(value: i64, min: i64, max: i64) -> Self {
        let clamped = value.clamp(min, max);
        Self {
            value: clamped,
            min,
            max,
        }
    }
}

impl Validate for Bounded {
    fn is_valid(&self) -> bool {
        self.value >= self.min && self.value <= self.max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize() {
        assert_eq!(categorize(-1), "negative");
        assert_eq!(categorize(0), "zero");
        assert_eq!(categorize(1), "small");
        assert_eq!(categorize(100), "small");
        assert_eq!(categorize(101), "large");
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(7, 1), 1);
        assert_eq!(gcd(0, 5), 5);
        assert_eq!(gcd(100, 0), 100);
    }

    #[test]
    #[should_panic(expected = "gcd(0, 0) is undefined")]
    fn test_gcd_zero_zero() {
        gcd(0, 0);
    }

    #[test]
    fn test_bounded_clamping() {
        let b = Bounded::new(150, 0, 100);
        assert_eq!(b.value, 100);
        let b = Bounded::new(-10, 0, 100);
        assert_eq!(b.value, 0);
        let b = Bounded::new(50, 0, 100);
        assert_eq!(b.value, 50);
    }

    #[test]
    fn test_bounded_valid() {
        let b = Bounded::new(50, 0, 100);
        assert!(b.is_valid());
        // Directly-constructed Bounded with out-of-range value is invalid.
        let invalid = Bounded {
            value: 150,
            min: 0,
            max: 100,
        };
        assert!(!invalid.is_valid());
    }

    // Exercises heap allocation (relevant for address sanitizer).
    #[test]
    fn test_heap_allocation() {
        let data: Vec<i64> = (-50..50).collect();
        let categories: Vec<_> = data.iter().map(|&v| categorize(v)).collect();
        assert_eq!(categories[0], "negative");
        assert_eq!(categories[50], "zero");
        assert_eq!(categories[51], "small");
    }

    // Exercises threading (relevant for thread sanitizer).
    #[test]
    fn test_threading() {
        use std::sync::Arc;
        use std::thread;
        let handles: Vec<_> = (1u64..=4)
            .map(|i| {
                let n = Arc::new(i * 25);
                thread::spawn(move || gcd(*n, 100))
            })
            .collect();
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert_eq!(results, vec![25, 50, 25, 100]);
    }
}
