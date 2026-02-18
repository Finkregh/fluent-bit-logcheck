/// Pattern grouping and regex generation using grex
///
/// This module groups similar log messages and generates
/// regular expressions that match each group.
use crate::cli::Result;
use crate::regex_conversion;
use anyhow::Context;
use grex::RegExpBuilder;
use regex::Regex;
use std::collections::HashMap;

/// A group of similar log messages with a generated regex
#[derive(Debug, Clone)]
pub struct PatternGroup {
    /// The generated regex pattern in POSIX format (for logcheck)
    pub regex: String,
    /// Indices of messages that match this pattern
    pub matching_indices: Vec<usize>,
    /// Number of matches
    pub match_count: usize,
}

impl PatternGroup {
    /// Create a compiled regex from this pattern
    pub fn compile(&self) -> Result<Regex> {
        // Convert POSIX to Rust format for compilation
        let rust_pattern = regex_conversion::posix_to_rust(&self.regex);
        Regex::new(&rust_pattern)
            .with_context(|| format!("Failed to compile regex: {}", self.regex))
    }

    /// Get a display string showing the pattern and match count
    pub fn display(&self) -> String {
        format!("[{} matches] {}", self.match_count, self.regex)
    }

    /// Get the regex in modern format (for display purposes)
    pub fn modern_format(&self) -> String {
        regex_conversion::posix_to_modern(&self.regex)
    }
}

/// Group similar log messages and generate regex patterns
///
/// # Arguments
/// * `entries` - Log messages to analyze
/// * `min_group_size` - Minimum number of messages to form a group
///
/// # Returns
/// Vector of pattern groups, sorted by match count (descending)
pub fn group_and_generate(entries: &[String], min_group_size: usize) -> Result<Vec<PatternGroup>> {
    if entries.is_empty() {
        return Ok(Vec::new());
    }

    // First pass: Group by exact prefix (first 50 chars or first word)
    let mut prefix_groups: HashMap<String, Vec<usize>> = HashMap::new();

    for (idx, entry) in entries.iter().enumerate() {
        let prefix = extract_prefix(entry);
        prefix_groups.entry(prefix).or_default().push(idx);
    }

    // Second pass: Generate regex for each group
    let mut pattern_groups = Vec::new();

    for (_prefix, indices) in prefix_groups {
        // Skip small groups
        if indices.len() < min_group_size {
            continue;
        }

        // Get the actual log messages for this group
        let group_messages: Vec<&str> = indices.iter().map(|&idx| entries[idx].as_str()).collect();

        // Generate regex using grex (produces modern format with \d, \w, etc.)
        let modern_regex = RegExpBuilder::from(&group_messages)
            .with_conversion_of_digits()
            .with_conversion_of_words()
            .with_conversion_of_repetitions()
            .build();

        // Convert to POSIX format for logcheck compatibility
        let posix_regex = regex_conversion::modern_to_posix(&modern_regex);

        // Verify the regex matches all group members
        // We need to convert back to Rust format for validation
        let rust_regex = regex_conversion::posix_to_rust(&posix_regex);
        if let Ok(compiled) = Regex::new(&rust_regex) {
            let valid_indices: Vec<usize> = indices
                .into_iter()
                .filter(|&idx| compiled.is_match(&entries[idx]))
                .collect();

            if valid_indices.len() >= min_group_size {
                pattern_groups.push(PatternGroup {
                    regex: posix_regex,
                    matching_indices: valid_indices.clone(),
                    match_count: valid_indices.len(),
                });
            }
        }
    }

    // Sort by match count (descending)
    pattern_groups.sort_by(|a, b| b.match_count.cmp(&a.match_count));

    Ok(pattern_groups)
}

/// Extract a representative prefix from a log message for grouping
fn extract_prefix(entry: &str) -> String {
    // Try to extract the first meaningful part
    // This helps group similar messages together

    // Strategy 1: Find the last colon in the first 80 chars (likely the service/program separator)
    let search_limit = entry.len().min(80);
    if let Some(colon_pos) = entry[..search_limit].rfind(':') {
        return entry[..=colon_pos].to_string();
    }

    // Strategy 2: Take first N characters (up to 50)
    let prefix_len = entry.len().min(50);
    entry[..prefix_len].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use grex::RegExpBuilder;

    #[test]
    fn test_extract_prefix() {
        assert_eq!(
            extract_prefix("pam_unix(sudo:session): session opened"),
            "pam_unix(sudo:session):"
        );

        assert_eq!(extract_prefix("systemd[1]: Started service"), "systemd[1]:");

        assert_eq!(extract_prefix("short message"), "short message");
    }

    #[test]
    fn test_group_and_generate() -> Result<()> {
        let entries = vec![
            "pam_unix(sudo:session): session opened for user root".to_string(),
            "pam_unix(sudo:session): session closed for user root".to_string(),
            "pam_unix(sudo:session): session opened for user alice".to_string(),
            "systemd[1]: Started service foo".to_string(),
            "systemd[1]: Started service bar".to_string(),
        ];

        let groups = group_and_generate(&entries, 2)?;

        // Should have 2 groups: pam_unix (3 matches) and systemd (2 matches)
        assert_eq!(groups.len(), 2, "Expected 2 groups, got {}", groups.len());

        // First group should have most matches
        assert_eq!(groups[0].match_count, 3);
        // The regex is in POSIX format with word character classes
        assert!(
            groups[0].regex.contains("[[:alnum:]_]"),
            "Expected POSIX word character class in: {}",
            groups[0].regex
        );

        // Second group
        assert_eq!(groups[1].match_count, 2);
        assert!(
            groups[1].regex.contains("[[:alnum:]_]") || groups[1].regex.contains("[[:digit:]]"),
            "Expected character classes in: {}",
            groups[1].regex
        );

        // Test that patterns can be compiled and match the original entries
        let compiled1 = groups[0].compile()?;
        assert!(compiled1.is_match(&entries[0]));
        assert!(compiled1.is_match(&entries[1]));
        assert!(compiled1.is_match(&entries[2]));

        let compiled2 = groups[1].compile()?;
        assert!(compiled2.is_match(&entries[3]));
        assert!(compiled2.is_match(&entries[4]));

        Ok(())
    }

    #[test]
    fn test_pattern_group_compile() -> Result<()> {
        // Test with POSIX format (what we store)
        let group = PatternGroup {
            regex: "test [[:digit:]]+".to_string(),
            matching_indices: vec![0, 1],
            match_count: 2,
        };

        let compiled = group.compile()?;
        assert!(compiled.is_match("test 123"));
        assert!(!compiled.is_match("test abc"));

        Ok(())
    }

    #[test]
    fn test_group_generation_anchors_and_matches() -> Result<()> {
        let entries = vec![
            "app: User alice logged in id=123".to_string(),
            "app: User bob logged in id=456".to_string(),
            "app: User charlie logged in id=789".to_string(),
        ];

        let groups = group_and_generate(&entries, 2)?;
        assert_eq!(groups.len(), 1);

        let group = &groups[0];
        assert!(group.regex.contains("[[:digit:]]"));
        assert!(group.regex.contains("[[:alnum:]_]"));

        let compiled = group.compile()?;
        for entry in entries {
            assert!(compiled.is_match(&entry));
        }

        Ok(())
    }

    #[test]
    fn test_modern_format_conversion() {
        let group = PatternGroup {
            regex: "^pam_unix\\(sudo:session\\): session [[:alnum:]_]+ for user [[:alnum:]_]+$"
                .to_string(),
            matching_indices: vec![0, 1, 2],
            match_count: 3,
        };

        let modern = group.modern_format();
        assert!(modern.contains(r"\w+"));
        assert!(!modern.contains("[[:alnum:]_]"));
    }

    #[test]
    fn test_grex_output_for_sample_lines() -> Result<()> {
        let entries = vec![
            "pam_unix(sudo:session): session opened for user root(uid=0) by root(uid=1000)"
                .to_string(),
            "pam_unix(sudo:session): session closed for user root".to_string(),
        ];

        let groups = group_and_generate(&entries, 2)?;
        assert_eq!(groups.len(), 1);

        let group = &groups[0];
        println!("Generated regex: {}", group.regex);
        Ok(())
    }

    #[test]
    fn test_grex_replacements_for_words() {
        let samples = vec!["opened", "closed", "root"];

        let default_regex = RegExpBuilder::from(&samples).build();
        println!("grex default: {}", default_regex);

        let no_anchors = RegExpBuilder::from(&samples).without_anchors().build();
        println!("grex no anchors: {}", no_anchors);

        let words_to_class = RegExpBuilder::from(&samples)
            .with_conversion_of_words()
            .build();
        println!("grex words->\\w: {}", words_to_class);

        let digits_to_class = RegExpBuilder::from(&samples)
            .with_conversion_of_digits()
            .build();
        println!("grex digits->\\d: {}", digits_to_class);

        let repetitions = RegExpBuilder::from(&samples)
            .with_conversion_of_repetitions()
            .build();
        println!("grex repetitions: {}", repetitions);
    }
}
