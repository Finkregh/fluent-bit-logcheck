/// Bidirectional conversion between modern regex syntax and logcheck POSIX format
///
/// This module handles conversion between:
/// - Modern regex shorthand (`\d`, `\w`, `\s`) used by grex
/// - POSIX character classes (`[[:digit:]]`, `[[:alnum:]]`, `[[:space:]]`) used by logcheck
use once_cell::sync::Lazy;

/// Conversion mapping between modern regex and POSIX character classes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexConversion {
    /// Modern regex pattern (e.g., `\d`)
    pub modern: &'static str,
    /// POSIX character class (e.g., `[[:digit:]]`)
    pub posix: &'static str,
    /// Description of what it matches
    pub description: &'static str,
}

/// Complete list of regex conversions
pub static REGEX_CONVERSIONS: Lazy<Vec<RegexConversion>> = Lazy::new(|| {
    vec![
        RegexConversion {
            modern: r"\d",
            posix: "[[:digit:]]",
            description: "Digits 0-9",
        },
        RegexConversion {
            modern: r"\D",
            posix: "[^[:digit:]]",
            description: "Non-digits",
        },
        RegexConversion {
            modern: r"\s",
            posix: "[[:space:]]",
            description: "Whitespace characters",
        },
        RegexConversion {
            modern: r"\S",
            posix: "[^[:space:]]",
            description: "Non-whitespace characters",
        },
        // Note: \w is approximately [[:alnum:]_] but not exact due to Unicode
        RegexConversion {
            modern: r"\w",
            posix: "[[:alnum:]_]",
            description: "Word characters (letters, digits, underscore)",
        },
        RegexConversion {
            modern: r"\W",
            posix: "[^[:alnum:]_]",
            description: "Non-word characters",
        },
    ]
});

/// Additional POSIX classes that don't have direct modern equivalents
pub static POSIX_ONLY_CONVERSIONS: Lazy<Vec<(&'static str, &'static str)>> = Lazy::new(|| {
    vec![
        ("[[:alpha:]]", "a-zA-Z"),
        ("[[:alnum:]]", "a-zA-Z0-9"),
        ("[[:xdigit:]]", "0-9a-fA-F"),
        ("[[:lower:]]", "a-z"),
        ("[[:upper:]]", "A-Z"),
        ("[[:blank:]]", " \\t"),
        ("[[:punct:]]", "!\"#$%&'()*+,\\-./:;<=>?@\\[\\\\\\]^_`{|}~"),
        ("[[:print:]]", "\\x20-\\x7E"),
        ("[[:graph:]]", "!-~"),
        ("[[:cntrl:]]", "\\x00-\\x1F\\x7F"),
    ]
});

/// Convert modern regex syntax (from grex) to POSIX format (for logcheck)
///
/// Converts shorthand like `\d`, `\w`, `\s` to POSIX classes like `[[:digit:]]`
///
/// # Examples
/// ```
/// use logcheck_fluent_bit_filter::regex_conversion::modern_to_posix;
///
/// assert_eq!(modern_to_posix(r"test\d\d\d"), "test[[:digit:]][[:digit:]][[:digit:]]");
/// assert_eq!(modern_to_posix(r"\w+"), "[[:alnum:]_]+");
/// ```
pub fn modern_to_posix(pattern: &str) -> String {
    let mut result = pattern.to_string();

    // Apply conversions in order (longer patterns first to avoid partial replacements)
    for conversion in REGEX_CONVERSIONS.iter() {
        result = result.replace(conversion.modern, conversion.posix);
    }

    result
}

/// Convert POSIX format (from logcheck) to modern regex syntax
///
/// Converts POSIX classes like `[[:digit:]]` to shorthand like `\d`
///
/// # Examples
/// ```
/// use logcheck_fluent_bit_filter::regex_conversion::posix_to_modern;
///
/// assert_eq!(posix_to_modern("test[[:digit:]][[:digit:]][[:digit:]]"), r"test\d\d\d");
/// assert_eq!(posix_to_modern("[[:alnum:]_]+"), r"\w+");
/// ```
pub fn posix_to_modern(pattern: &str) -> String {
    let mut result = pattern.to_string();

    // Apply conversions in reverse order
    for conversion in REGEX_CONVERSIONS.iter() {
        result = result.replace(conversion.posix, conversion.modern);
    }

    // Also convert POSIX-only classes to their regex equivalents
    for (posix_class, rust_equiv) in POSIX_ONLY_CONVERSIONS.iter() {
        result = result.replace(posix_class, rust_equiv);
    }

    result
}

/// Convert POSIX format to Rust regex syntax (for internal processing)
///
/// This is similar to `posix_to_modern` but also handles POSIX-only classes
///
/// # Examples
/// ```
/// use logcheck_fluent_bit_filter::regex_conversion::posix_to_rust;
///
/// assert_eq!(posix_to_rust("[[:digit:]]"), r"\d");
/// assert_eq!(posix_to_rust("[[:alpha:]]"), "a-zA-Z");
/// ```
pub fn posix_to_rust(pattern: &str) -> String {
    let mut result = pattern.to_string();

    // First apply direct conversions
    for conversion in REGEX_CONVERSIONS.iter() {
        result = result.replace(conversion.posix, conversion.modern);
    }

    // Then handle POSIX-only classes
    for (posix_class, rust_equiv) in POSIX_ONLY_CONVERSIONS.iter() {
        result = result.replace(posix_class, rust_equiv);
    }

    // Escape unescaped curly braces that aren't part of quantifiers
    result = result.replace(" { ", " \\{ ");
    result = result.replace(" } ", " \\} ");

    result
}

/// Remove anchors from a regex pattern
///
/// grex always adds `^` and `$` anchors, but logcheck patterns may not need them
pub fn remove_anchors(pattern: &str) -> String {
    let mut result = pattern.to_string();

    // Remove start anchor
    if result.starts_with('^') {
        result = result[1..].to_string();
    }

    // Remove end anchor
    if result.ends_with('$') {
        result = result[..result.len() - 1].to_string();
    }

    result
}

/// Add anchors to a regex pattern if not present
pub fn ensure_anchors(pattern: &str) -> String {
    let mut result = pattern.to_string();

    if !result.starts_with('^') {
        result = format!("^{}", result);
    }

    if !result.ends_with('$') {
        result = format!("{}$", result);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modern_to_posix() {
        assert_eq!(modern_to_posix(r"\d"), "[[:digit:]]");
        assert_eq!(
            modern_to_posix(r"\d\d\d"),
            "[[:digit:]][[:digit:]][[:digit:]]"
        );
        assert_eq!(modern_to_posix(r"\w+"), "[[:alnum:]_]+");
        assert_eq!(modern_to_posix(r"\s"), "[[:space:]]");
        assert_eq!(modern_to_posix(r"\D"), "[^[:digit:]]");
        assert_eq!(modern_to_posix(r"\W"), "[^[:alnum:]_]");
        assert_eq!(modern_to_posix(r"\S"), "[^[:space:]]");
    }

    #[test]
    fn test_posix_to_modern() {
        assert_eq!(posix_to_modern("[[:digit:]]"), r"\d");
        assert_eq!(
            posix_to_modern("[[:digit:]][[:digit:]][[:digit:]]"),
            r"\d\d\d"
        );
        assert_eq!(posix_to_modern("[[:alnum:]_]+"), r"\w+");
        assert_eq!(posix_to_modern("[[:space:]]"), r"\s");
        assert_eq!(posix_to_modern("[^[:digit:]]"), r"\D");
    }

    #[test]
    fn test_posix_to_rust() {
        assert_eq!(posix_to_rust("[[:digit:]]"), r"\d");
        assert_eq!(posix_to_rust("[[:alpha:]]"), "a-zA-Z");
        assert_eq!(posix_to_rust("[[:alnum:]]"), "a-zA-Z0-9");
        assert_eq!(posix_to_rust("[[:space:]]"), r"\s");
    }

    #[test]
    fn test_roundtrip() {
        let modern = r"\d+\s\w+";
        let posix = modern_to_posix(modern);
        assert_eq!(posix, "[[:digit:]]+[[:space:]][[:alnum:]_]+");
        let back_to_modern = posix_to_modern(&posix);
        assert_eq!(back_to_modern, modern);
    }

    #[test]
    fn test_remove_anchors() {
        assert_eq!(remove_anchors("^test$"), "test");
        assert_eq!(remove_anchors("^test"), "test");
        assert_eq!(remove_anchors("test$"), "test");
        assert_eq!(remove_anchors("test"), "test");
    }

    #[test]
    fn test_ensure_anchors() {
        assert_eq!(ensure_anchors("test"), "^test$");
        assert_eq!(ensure_anchors("^test"), "^test$");
        assert_eq!(ensure_anchors("test$"), "^test$");
        assert_eq!(ensure_anchors("^test$"), "^test$");
    }

    #[test]
    fn test_complex_pattern() {
        let grex_output = r"^pam_unix\(sudo:session\): session \w+ for user \w+$";
        let posix = modern_to_posix(grex_output);
        assert_eq!(
            posix,
            "^pam_unix\\(sudo:session\\): session [[:alnum:]_]+ for user [[:alnum:]_]+$"
        );
    }
}
