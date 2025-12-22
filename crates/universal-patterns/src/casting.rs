//! Safe type casting utilities
//!
//! This module provides safe casting helpers that prevent overflow
//! and other undefined behavior.

use std::fmt;

/// Error that occurs during casting operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CastError {
    /// Value would overflow the target type
    Overflow {
        /// Source type name
        from_type: &'static str,
        /// Target type name
        to_type: &'static str,
        /// String representation of the value
        value: String,
    },
    /// Value would underflow the target type
    Underflow {
        /// Source type name
        from_type: &'static str,
        /// Target type name
        to_type: &'static str,
        /// String representation of the value
        value: String,
    },
}

impl fmt::Display for CastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CastError::Overflow {
                from_type,
                to_type,
                value,
            } => {
                write!(
                    f,
                    "Overflow casting {} to {}: value {}",
                    from_type, to_type, value
                )
            }
            CastError::Underflow {
                from_type,
                to_type,
                value,
            } => {
                write!(
                    f,
                    "Underflow casting {} to {}: value {}",
                    from_type, to_type, value
                )
            }
        }
    }
}

impl std::error::Error for CastError {}

/// Safely cast u64 to i64, checking for overflow
pub fn u64_to_i64(value: u64) -> Result<i64, CastError> {
    i64::try_from(value).map_err(|_| CastError::Overflow {
        from_type: "u64",
        to_type: "i64",
        value: value.to_string(),
    })
}

/// Safely cast i64 to u64, checking for underflow
pub fn i64_to_u64(value: i64) -> Result<u64, CastError> {
    u64::try_from(value).map_err(|_| CastError::Underflow {
        from_type: "i64",
        to_type: "u64",
        value: value.to_string(),
    })
}

/// Safely cast usize to u64
pub fn usize_to_u64(value: usize) -> Result<u64, CastError> {
    u64::try_from(value).map_err(|_| CastError::Overflow {
        from_type: "usize",
        to_type: "u64",
        value: value.to_string(),
    })
}

/// Safely cast u64 to usize
pub fn u64_to_usize(value: u64) -> Result<usize, CastError> {
    usize::try_from(value).map_err(|_| CastError::Overflow {
        from_type: "u64",
        to_type: "usize",
        value: value.to_string(),
    })
}

/// Safely cast i64 to i32
pub fn i64_to_i32(value: i64) -> Result<i32, CastError> {
    i32::try_from(value).map_err(|_| CastError::Overflow {
        from_type: "i64",
        to_type: "i32",
        value: value.to_string(),
    })
}

/// Safely cast u32 to i64
pub fn u32_to_i64(value: u32) -> i64 {
    // u32 always fits in i64
    value as i64
}

/// Saturating cast from u64 to i64 (clamps at i64::MAX)
pub fn u64_to_i64_saturating(value: u64) -> i64 {
    if value > i64::MAX as u64 {
        i64::MAX
    } else {
        value as i64
    }
}

/// Saturating cast from i64 to u64 (clamps at 0)
pub fn i64_to_u64_saturating(value: i64) -> u64 {
    if value < 0 {
        0
    } else {
        value as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_to_i64_success() {
        assert_eq!(u64_to_i64(0).unwrap(), 0);
        assert_eq!(u64_to_i64(100).unwrap(), 100);
        assert_eq!(u64_to_i64(i64::MAX as u64).unwrap(), i64::MAX);
    }

    #[test]
    fn test_u64_to_i64_overflow() {
        let result = u64_to_i64(u64::MAX);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CastError::Overflow { .. }));
    }

    #[test]
    fn test_i64_to_u64_success() {
        assert_eq!(i64_to_u64(0).unwrap(), 0);
        assert_eq!(i64_to_u64(100).unwrap(), 100);
        assert_eq!(i64_to_u64(i64::MAX).unwrap(), i64::MAX as u64);
    }

    #[test]
    fn test_i64_to_u64_underflow() {
        let result = i64_to_u64(-1);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CastError::Underflow { .. }));
    }

    #[test]
    fn test_saturating_casts() {
        // u64 to i64 saturating
        assert_eq!(u64_to_i64_saturating(0), 0);
        assert_eq!(u64_to_i64_saturating(100), 100);
        assert_eq!(u64_to_i64_saturating(u64::MAX), i64::MAX);

        // i64 to u64 saturating
        assert_eq!(i64_to_u64_saturating(0), 0);
        assert_eq!(i64_to_u64_saturating(100), 100);
        assert_eq!(i64_to_u64_saturating(-1), 0);
        assert_eq!(i64_to_u64_saturating(i64::MIN), 0);
    }

    #[test]
    fn test_u32_to_i64() {
        // u32 always fits in i64
        assert_eq!(u32_to_i64(0), 0);
        assert_eq!(u32_to_i64(u32::MAX), u32::MAX as i64);
    }
}
