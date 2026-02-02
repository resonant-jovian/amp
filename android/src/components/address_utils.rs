//! Address fuzzy matching utilities
//!
//! Provides helper functions for fuzzy string matching and normalization
//! to improve address lookup accuracy.
//!
//! # Examples
//! ```
//! use amp_android::address_utils::{fuzzy_match_address, normalize_string};
//!
//! let normalized = normalize_string("  STORGATAN  ");
//! assert_eq!(normalized, "storgatan");
//!
//! let is_match = fuzzy_match_address("Storgatan", "storgatan");
//! assert!(is_match);
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
/// Fuzzy match two address strings
///
/// Case-insensitive comparison with whitespace normalization.
///
/// # Arguments
/// * `a` - First address string
/// * `b` - Second address string
///
/// # Returns
/// true if strings match after normalization
///
/// # Examples
/// ```
/// use amp_android::address_utils::fuzzy_match_address;
///
/// assert!(fuzzy_match_address("Storgatan", "storgatan"));
/// assert!(fuzzy_match_address("  STORGATAN  ", "storgatan"));
/// assert!(fuzzy_match_address("Test Street", "TEST STREET"));
/// assert!(!fuzzy_match_address("Storgatan", "Änggården"));
/// ```
pub fn fuzzy_match_address(a: &str, b: &str) -> bool {
    normalize_string(a) == normalize_string(b)
}
/// Calculate simple string similarity score (0.0 to 1.0)
///
/// Uses basic character overlap calculation.
/// More sophisticated algorithms (Levenshtein, Jaro-Winkler) can be added later.
///
/// # Arguments
/// * `a` - First string
/// * `b` - Second string
///
/// # Returns
/// Similarity score between 0.0 (no match) and 1.0 (exact match)
///
/// # Examples
/// ```
/// use amp_android::address_utils::similarity_score;
///
/// let score = similarity_score("Storgatan", "Storgatan");
/// assert_eq!(score, 1.0);
///
/// let score = similarity_score("Storgatan", "Storatan");
/// assert!(score > 0.7 && score < 1.0);
///
/// let score = similarity_score("Storgatan", "Xyz");
/// assert!(score < 0.5);
/// ```
pub fn similarity_score(a: &str, b: &str) -> f64 {
    let a_norm = normalize_string(a);
    let b_norm = normalize_string(b);
    if a_norm == b_norm {
        return 1.0;
    }
    if a_norm.is_empty() || b_norm.is_empty() {
        return 0.0;
    }
    let mut matches = 0;
    let a_chars: Vec<char> = a_norm.chars().collect();
    let b_chars: Vec<char> = b_norm.chars().collect();
    for (i, &a_char) in a_chars.iter().enumerate() {
        if i < b_chars.len() && b_chars[i] == a_char {
            matches += 1;
        }
    }
    let max_len = a_chars.len().max(b_chars.len()) as f64;
    matches as f64 / max_len
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
    #[test]
    fn test_fuzzy_match_address() {
        assert!(fuzzy_match_address("Storgatan", "storgatan"));
        assert!(fuzzy_match_address("  STORGATAN  ", "storgatan"));
        assert!(fuzzy_match_address("Test Street", "TEST STREET"));
        assert!(!fuzzy_match_address("Storgatan", "Änggården"));
    }
    #[test]
    fn test_similarity_score_exact() {
        let score = similarity_score("Storgatan", "Storgatan");
        assert_eq!(score, 1.0);
    }
    #[test]
    fn test_similarity_score_similar() {
        let score = similarity_score("Storgatan", "Storatan");
        assert!(score > 0.7 && score < 1.0);
    }
    #[test]
    fn test_similarity_score_different() {
        let score = similarity_score("Storgatan", "Xyz");
        assert!(score < 0.5);
    }
    #[test]
    fn test_similarity_score_empty() {
        assert_eq!(similarity_score("", ""), 0.0);
        assert_eq!(similarity_score("Test", ""), 0.0);
        assert_eq!(similarity_score("", "Test"), 0.0);
    }
}
