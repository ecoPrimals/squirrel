// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Safe numeric cast helpers — no silent truncation or precision loss.
//!
//! Absorbed from groundSpring V114 / airSpring V0.8.9. Provides checked
//! conversions that return `Option` or saturate instead of panicking or
//! silently wrapping.
//!
//! These functions intentionally use `as` casts after guarding against
//! overflow/truncation, so we allow the cast lints at the module level.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "Checked conversions after guards; intentional safe casts"
)]

/// Cast `usize` to `u32`, returning `None` if the value overflows.
#[must_use]
#[inline]
pub const fn usize_to_u32(value: usize) -> Option<u32> {
    if value > u32::MAX as usize {
        None
    } else {
        Some(value as u32)
    }
}

/// Cast `usize` to `u32`, saturating at `u32::MAX`.
#[must_use]
#[inline]
pub const fn usize_to_u32_saturating(value: usize) -> u32 {
    if value > u32::MAX as usize {
        u32::MAX
    } else {
        value as u32
    }
}

/// Cast `f64` to `f32`, returning `None` if the value is out of `f32` range.
#[must_use]
#[inline]
pub const fn f64_to_f32(value: f64) -> Option<f32> {
    if value.is_nan() {
        return Some(f32::NAN);
    }
    if value.is_infinite() {
        return Some(if value.is_sign_positive() {
            f32::INFINITY
        } else {
            f32::NEG_INFINITY
        });
    }
    let result = value as f32;
    if result.is_infinite() {
        None
    } else {
        Some(result)
    }
}

/// Cast `u64` to `usize`, returning `None` on 32-bit platforms if value overflows.
#[must_use]
#[inline]
pub const fn u64_to_usize(value: u64) -> Option<usize> {
    if value > usize::MAX as u64 {
        None
    } else {
        Some(value as usize)
    }
}

/// Cast `i64` to `usize`, returning `None` if negative or overflows.
#[must_use]
#[inline]
pub const fn i64_to_usize(value: i64) -> Option<usize> {
    if value < 0 || value as u64 > usize::MAX as u64 {
        None
    } else {
        Some(value as usize)
    }
}

/// Cast `f64` to `u64`, clamping to `[0, u64::MAX]` and rounding.
#[must_use]
#[inline]
pub fn f64_to_u64_clamped(value: f64) -> u64 {
    if value.is_nan() || value <= 0.0 {
        0
    } else if value >= u64::MAX as f64 {
        u64::MAX
    } else {
        value.round() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usize_u32_within_range() {
        assert_eq!(usize_to_u32(42), Some(42));
        assert_eq!(usize_to_u32(0), Some(0));
        assert_eq!(usize_to_u32(u32::MAX as usize), Some(u32::MAX));
    }

    #[test]
    fn usize_u32_overflow() {
        if std::mem::size_of::<usize>() > 4 {
            assert_eq!(usize_to_u32(u32::MAX as usize + 1), None);
        }
    }

    #[test]
    fn usize_u32_saturating() {
        assert_eq!(usize_to_u32_saturating(42), 42);
        if std::mem::size_of::<usize>() > 4 {
            assert_eq!(usize_to_u32_saturating(u32::MAX as usize + 1), u32::MAX);
        }
    }

    #[test]
    fn f64_f32_normal() {
        assert_eq!(f64_to_f32(1.0), Some(1.0));
        assert_eq!(f64_to_f32(0.0), Some(0.0));
        assert_eq!(f64_to_f32(-1.5), Some(-1.5));
    }

    #[test]
    fn f64_f32_special() {
        assert!(f64_to_f32(f64::NAN).expect("should succeed").is_nan());
        assert_eq!(f64_to_f32(f64::INFINITY), Some(f32::INFINITY));
        assert_eq!(f64_to_f32(f64::NEG_INFINITY), Some(f32::NEG_INFINITY));
    }

    #[test]
    fn f64_f32_overflow() {
        assert_eq!(f64_to_f32(f64::MAX), None);
    }

    #[test]
    fn u64_usize_valid() {
        assert_eq!(u64_to_usize(42), Some(42));
        assert_eq!(u64_to_usize(0), Some(0));
    }

    #[test]
    fn i64_usize_valid() {
        assert_eq!(i64_to_usize(42), Some(42));
        assert_eq!(i64_to_usize(0), Some(0));
    }

    #[test]
    fn i64_usize_negative() {
        assert_eq!(i64_to_usize(-1), None);
    }

    #[test]
    fn f64_u64_clamped() {
        assert_eq!(f64_to_u64_clamped(42.7), 43);
        assert_eq!(f64_to_u64_clamped(0.0), 0);
        assert_eq!(f64_to_u64_clamped(-5.0), 0);
        assert_eq!(f64_to_u64_clamped(f64::NAN), 0);
        assert_eq!(f64_to_u64_clamped(f64::MAX), u64::MAX);
    }
}
