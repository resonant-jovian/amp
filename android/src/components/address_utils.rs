//! Address fuzzy matching utilities
//!
//! Provides helper functions for fuzzy string matching and normalization
//! to improve address lookup accuracy.
//!
//! # Examples
//! ```
//! use amp_android::address_utils::normalize_string;
//!
//! let normalized = normalize_string("  STORGATAN  ");
//! assert_eq!(normalized, "storgatan");
//! ```
/// Normalize string for comparison
///
/// Converts to lowercase and trims whitespace.
///
/// # Arguments
/// * `s` - String to normalize
///
/// # Returns
/// Normalized string (lowercase, trimmed)
///
/// # Examples
/// ```
/// use amp_android::address_utils::normalize_string;
///
/// assert_eq!(normalize_string("  STORGATAN  "), "storgatan");
/// assert_eq!(normalize_string("Test Street"), "test street");
/// assert_eq!(normalize_string("  "), "");
/// ```
pub fn normalize_string(s: &str) -> String {
    s.trim().to_lowercase()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_normalize_string() {
        assert_eq!(normalize_string("Storgatan"), "storgatan");
        assert_eq!(normalize_string("  STORGATAN  "), "storgatan");
        assert_eq!(normalize_string("Test Street"), "test street");
        assert_eq!(normalize_string(""), "");
        assert_eq!(normalize_string("  "), "");
    }
}
