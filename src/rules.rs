use regex::{Regex, RegexSet};
use serde;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RuleCategory {
    Cracking,
    CrackingIgnore,
    Violations,
    ViolationsIgnore,
    SystemEvents,
    Workstation,
    Server,
    Local,
}

/// A rule set that can be split into multiple chunks to avoid regex size limits
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RuleSet {
    pub category: RuleCategory,
    pub patterns: Vec<String>,
    pub compiled: Option<CompiledRules>,
    pub source_files: Vec<PathBuf>,
}

/// Compiled regex rules that may be split into multiple chunks
#[derive(Debug, Clone)]
pub enum CompiledRules {
    /// Single RegexSet (optimized, when all patterns fit)
    Single(RegexSet),
    /// Multiple RegexSets (used when patterns exceed size limit)
    Chunked(Vec<RegexSet>),
}

#[derive(Debug, Clone)]
pub struct LogcheckDatabase {
    pub cracking_rules: RuleSet,
    pub cracking_ignore: RuleSet,
    pub violations_rules: RuleSet,
    pub violations_ignore: RuleSet,
    pub system_events: RuleSet,
    pub workstation: RuleSet,
    pub server: RuleSet,
    pub local: RuleSet,
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum RuleError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("Invalid rule format: {0}")]
    InvalidRuleFormat(String),
    #[error("Directory not found: {0}")]
    DirectoryNotFound(PathBuf),
}

impl Default for LogcheckDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl LogcheckDatabase {
    /// Create a new empty logcheck database
    pub fn new() -> Self {
        Self {
            cracking_rules: RuleSet::new(RuleCategory::Cracking),
            cracking_ignore: RuleSet::new(RuleCategory::CrackingIgnore),
            violations_rules: RuleSet::new(RuleCategory::Violations),
            violations_ignore: RuleSet::new(RuleCategory::ViolationsIgnore),
            system_events: RuleSet::new(RuleCategory::SystemEvents),
            workstation: RuleSet::new(RuleCategory::Workstation),
            server: RuleSet::new(RuleCategory::Server),
            local: RuleSet::new(RuleCategory::Local),
        }
    }

    /// Convert POSIX character classes to Rust regex equivalents
    /// Logcheck rules use POSIX classes like [[:alnum:]], [[:digit:]], etc.
    /// which are not supported by Rust's regex crate
    fn convert_posix_classes(pattern: &str) -> String {
        let mut result = pattern.to_string();

        // Replace POSIX character classes with Rust equivalents
        // Note: Order matters for nested replacements
        let replacements = vec![
            ("[[:alnum:]]", "a-zA-Z0-9"),
            ("[[:alpha:]]", "a-zA-Z"),
            ("[[:digit:]]", "0-9"),
            ("[[:xdigit:]]", "0-9a-fA-F"),
            ("[[:lower:]]", "a-z"),
            ("[[:upper:]]", "A-Z"),
            ("[[:space:]]", "\\s"),
            ("[[:blank:]]", " \\t"),
            ("[[:punct:]]", "!\"#$%&'()*+,\\-./:;<=>?@\\[\\\\\\]^_`{|}~"),
            ("[[:print:]]", "\\x20-\\x7E"),
            ("[[:graph:]]", "!-~"),
            ("[[:cntrl:]]", "\\x00-\\x1F\\x7F"),
        ];

        for (posix_class, rust_equiv) in replacements {
            // When POSIX class appears inside a character class, just replace the contents
            // e.g., [._[:alnum:]-] becomes [._a-zA-Z0-9-]
            result = result.replace(posix_class, rust_equiv);
        }

        // Escape unescaped curly braces that aren't part of quantifiers
        // This is a simplified approach - just escape standalone braces
        result = result.replace(" { ", " \\{ ");
        result = result.replace(" } ", " \\} ");

        result
    }

    /// Load logcheck database from traditional directory structure
    #[allow(dead_code)]
    pub fn load_from_directory<P: AsRef<Path>>(base_path: P) -> Result<Self, RuleError> {
        let base_path = base_path.as_ref();

        if !base_path.exists() {
            return Err(RuleError::DirectoryNotFound(base_path.to_path_buf()));
        }

        let mut database = Self::new();

        // Load rule categories in order
        let categories = [
            ("cracking.d", &mut database.cracking_rules),
            ("cracking.ignore.d", &mut database.cracking_ignore),
            ("violations.d", &mut database.violations_rules),
            ("violations.ignore.d", &mut database.violations_ignore),
            ("ignore.d.paranoid", &mut database.system_events),
            ("ignore.d.workstation", &mut database.workstation),
            ("ignore.d.server", &mut database.server),
            ("local.d", &mut database.local),
        ];

        for (dir_name, rule_set) in categories {
            let dir_path = base_path.join(dir_name);
            if dir_path.exists() && dir_path.is_dir() {
                Self::load_rule_directory(&dir_path, rule_set)?;
            }
        }

        // Compile all rule sets
        database.compile_all()?;

        Ok(database)
    }

    /// Load all rule files from a directory
    #[allow(dead_code)]
    fn load_rule_directory(dir_path: &Path, rule_set: &mut RuleSet) -> Result<(), RuleError> {
        let entries = fs::read_dir(dir_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Only process regular files
            if path.is_file() {
                Self::load_rule_file(&path, rule_set)?;
            }
        }

        Ok(())
    }

    /// Load a single rule file
    #[allow(dead_code)]
    fn load_rule_file(file_path: &Path, rule_set: &mut RuleSet) -> Result<(), RuleError> {
        let content = fs::read_to_string(file_path)?;
        let mut skipped = 0;

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Skip patterns with backreferences (not supported in Rust regex)
            if line.contains("\\1") || line.contains("\\2") || line.contains("\\3") {
                skipped += 1;
                continue;
            }

            // Convert POSIX character classes to Rust regex equivalents
            let converted_pattern = Self::convert_posix_classes(line);

            // Validate regex pattern
            if let Err(e) = Regex::new(&converted_pattern) {
                eprintln!(
                    "Warning: Skipping invalid regex in {}:{}: '{}'",
                    file_path.display(),
                    line_num + 1,
                    line
                );
                eprintln!("  Error: {}", e);
                skipped += 1;
                continue; // Skip invalid rules instead of failing
            }

            rule_set.patterns.push(converted_pattern);
        }

        if skipped > 0 {
            eprintln!(
                "ℹ️  Skipped {} incompatible rules in {}",
                skipped,
                file_path.display()
            );
        }

        rule_set.source_files.push(file_path.to_path_buf());
        Ok(())
    }

    /// Compile all rule sets for efficient matching
    pub fn compile_all(&mut self) -> Result<(), RuleError> {
        self.cracking_rules.compile()?;
        self.cracking_ignore.compile()?;
        self.violations_rules.compile()?;
        self.violations_ignore.compile()?;
        self.system_events.compile()?;
        self.workstation.compile()?;
        self.server.compile()?;
        self.local.compile()?;
        Ok(())
    }

    /// Match a log message against logcheck rules
    /// Returns the rule category if matched, following logcheck precedence
    pub fn match_message(&self, message: &str) -> Option<RuleCategory> {
        // Logcheck processing order: cracking -> violations -> ignore rules

        // 1. Check for cracking attempts (highest priority)
        if self.cracking_rules.matches(message) && !self.cracking_ignore.matches(message) {
            return Some(RuleCategory::Cracking);
        }

        // 2. Check for violations (security events)
        if self.violations_rules.matches(message) && !self.violations_ignore.matches(message) {
            return Some(RuleCategory::Violations);
        }

        // 3. Check ignore rules (system events) - these are "normal" events to ignore
        if self.system_events.matches(message)
            || self.server.matches(message)
            || self.workstation.matches(message)
            || self.local.matches(message)
        {
            return Some(RuleCategory::SystemEvents);
        }

        // No match found - this is an unclassified/new event
        None
    }

    /// Get statistics about loaded rules
    #[allow(dead_code)]
    pub fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();

        stats.insert(
            "cracking_rules".to_string(),
            self.cracking_rules.patterns.len(),
        );
        stats.insert(
            "cracking_ignore".to_string(),
            self.cracking_ignore.patterns.len(),
        );
        stats.insert(
            "violations_rules".to_string(),
            self.violations_rules.patterns.len(),
        );
        stats.insert(
            "violations_ignore".to_string(),
            self.violations_ignore.patterns.len(),
        );
        stats.insert(
            "system_events".to_string(),
            self.system_events.patterns.len(),
        );
        stats.insert("workstation".to_string(), self.workstation.patterns.len());
        stats.insert("server".to_string(), self.server.patterns.len());
        stats.insert("local".to_string(), self.local.patterns.len());

        let total = stats.values().sum();
        stats.insert("total_rules".to_string(), total);

        stats
    }
}

impl RuleSet {
    pub fn new(category: RuleCategory) -> Self {
        Self {
            category,
            patterns: Vec::new(),
            compiled: None,
            source_files: Vec::new(),
        }
    }

    /// Compile patterns into RegexSet(s), automatically chunking if size limit is exceeded
    pub fn compile(&mut self) -> Result<(), RuleError> {
        if self.patterns.is_empty() {
            return Ok(());
        }

        // First, try to compile all patterns in a single RegexSet
        match RegexSet::new(&self.patterns) {
            Ok(regex_set) => {
                // Success! Use single optimized RegexSet
                self.compiled = Some(CompiledRules::Single(regex_set));
                Ok(())
            }
            Err(e) => {
                // Check if error is due to compiled size limit
                let error_msg = e.to_string();

                if error_msg.contains("size limit") || error_msg.contains("CompiledTooBig") {
                    // Automatically chunk the patterns
                    eprintln!(
                        "⚠️  Category {:?} ({} patterns) exceeds regex size limit, splitting into chunks...",
                        self.category,
                        self.patterns.len()
                    );
                    self.compile_chunked()
                } else {
                    // Other regex error, propagate it
                    Err(RuleError::RegexError(e))
                }
            }
        }
    }

    /// Compile patterns into multiple chunks using adaptive sizing
    fn compile_chunked(&mut self) -> Result<(), RuleError> {
        let total_patterns = self.patterns.len();

        // Start with an initial chunk size estimate
        // We use binary search to find the optimal chunk size
        let mut chunk_size = self.find_optimal_chunk_size()?;

        eprintln!(
            "   Splitting into {} chunks of ~{} patterns each",
            total_patterns.div_ceil(chunk_size),
            chunk_size
        );

        let mut chunks = Vec::new();
        let mut retry_count = 0;
        let max_retries = 3;

        loop {
            chunks.clear();
            let mut failed = false;

            for (chunk_idx, pattern_chunk) in self.patterns.chunks(chunk_size).enumerate() {
                match RegexSet::new(pattern_chunk) {
                    Ok(regex_set) => {
                        chunks.push(regex_set);
                    }
                    Err(e) => {
                        if e.to_string().contains("size limit") && retry_count < max_retries {
                            // Some chunks are more complex than others
                            // Reduce chunk size and retry
                            chunk_size = (chunk_size * 3) / 4; // Reduce by 25%
                            if chunk_size < 5 {
                                chunk_size = 5;
                            }
                            eprintln!(
                                "   Chunk {} too large, reducing chunk size to {} and retrying...",
                                chunk_idx, chunk_size
                            );
                            retry_count += 1;
                            failed = true;
                            break;
                        } else {
                            eprintln!(
                                "   Failed to compile chunk {} (size {}): {}",
                                chunk_idx,
                                pattern_chunk.len(),
                                e
                            );
                            return Err(RuleError::RegexError(e));
                        }
                    }
                }
            }

            if !failed {
                // All chunks compiled successfully
                break;
            }
        }

        eprintln!(
            "✅ Successfully compiled {:?} category into {} chunks",
            self.category,
            chunks.len()
        );

        self.compiled = Some(CompiledRules::Chunked(chunks));
        Ok(())
    }

    /// Find optimal chunk size using binary search
    /// Returns the largest chunk size that compiles successfully
    fn find_optimal_chunk_size(&self) -> Result<usize, RuleError> {
        let total = self.patterns.len();

        // Start with a conservative estimate
        // Typical safe size is around 50-100 patterns for normal rules
        // For very complex patterns, we may need smaller chunks
        let mut low = 10; // Minimum chunk size (very conservative)
        let mut high = total.min(200); // Maximum chunk size to try
        let mut best_size = low;

        // Binary search for optimal chunk size
        while low <= high {
            let mid = (low + high) / 2;

            // Try compiling a chunk of this size
            let test_chunk = &self.patterns[0..mid.min(total)];

            match RegexSet::new(test_chunk) {
                Ok(_) => {
                    // This size works, try larger
                    best_size = mid;
                    low = mid + 1;
                }
                Err(e) => {
                    if e.to_string().contains("size limit")
                        || e.to_string().contains("CompiledTooBig")
                    {
                        // Too large, try smaller
                        high = mid - 1;
                    } else {
                        // Other error, this might be a regex syntax issue
                        // Try smaller chunks anyway
                        high = mid - 1;
                    }
                }
            }
        }

        // Ensure we have a valid chunk size
        if best_size < 5 {
            best_size = 5; // Absolute minimum fallback
        }

        Ok(best_size)
    }

    /// Check if message matches any pattern in this rule set
    pub fn matches(&self, message: &str) -> bool {
        match &self.compiled {
            Some(CompiledRules::Single(regex_set)) => regex_set.is_match(message),
            Some(CompiledRules::Chunked(chunks)) => {
                // Check all chunks, return true if any chunk matches
                chunks.iter().any(|chunk| chunk.is_match(message))
            }
            None => false,
        }
    }

    pub fn add_pattern(&mut self, pattern: String) -> Result<(), RuleError> {
        // Validate regex before adding
        Regex::new(&pattern)?;
        self.patterns.push(pattern);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_rules_directory() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Create violations.d directory with some rules
        let violations_dir = temp_dir.path().join("violations.d");
        fs::create_dir_all(&violations_dir).unwrap();

        let mut violations_file = fs::File::create(violations_dir.join("security")).unwrap();
        writeln!(violations_file, "# Security violation rules").unwrap();
        writeln!(violations_file, "^.*authentication failure.*$").unwrap();
        writeln!(violations_file, "^.*failed password.*$").unwrap();

        // Create ignore.d.server directory
        let server_dir = temp_dir.path().join("ignore.d.server");
        fs::create_dir_all(&server_dir).unwrap();

        let mut server_file = fs::File::create(server_dir.join("systemd")).unwrap();
        writeln!(server_file, "# SystemD ignore rules").unwrap();
        writeln!(server_file, "^.*systemd.*: Started Session.*$").unwrap();
        writeln!(server_file, "^.*systemd.*: Stopped Session.*$").unwrap();

        temp_dir
    }

    #[test]
    fn test_load_from_directory() {
        let temp_dir = create_test_rules_directory();
        let database = LogcheckDatabase::load_from_directory(temp_dir.path()).unwrap();

        assert_eq!(database.violations_rules.patterns.len(), 2);
        assert_eq!(database.server.patterns.len(), 2);

        let stats = database.get_stats();
        assert_eq!(stats["violations_rules"], 2);
        assert_eq!(stats["server"], 2);
        assert_eq!(stats["total_rules"], 4);
    }

    #[test]
    fn test_rule_matching() {
        let temp_dir = create_test_rules_directory();
        let database = LogcheckDatabase::load_from_directory(temp_dir.path()).unwrap();

        // Test violation match
        let violation_msg = "Jan 01 12:00:00 host sshd[1234]: authentication failure for user";
        assert_eq!(
            database.match_message(violation_msg),
            Some(RuleCategory::Violations)
        );

        // Test system event match (ignore)
        let system_msg = "Jan 01 12:00:00 host systemd[1]: Started Session 123 of user alice";
        assert_eq!(
            database.match_message(system_msg),
            Some(RuleCategory::SystemEvents)
        );

        // Test no match
        let unknown_msg = "Jan 01 12:00:00 host myapp[999]: Some custom message";
        assert_eq!(database.match_message(unknown_msg), None);
    }

    #[test]
    fn test_rule_validation() {
        let mut rule_set = RuleSet::new(RuleCategory::Local);

        // Valid regex should work
        assert!(rule_set.add_pattern("^.*valid.*$".to_string()).is_ok());

        // Invalid regex should fail
        assert!(rule_set.add_pattern("[invalid regex(".to_string()).is_err());
    }
}
