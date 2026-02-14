pub mod json;
pub mod text;

use crate::cli::args::{OutputFormat, ShowMode};
use crate::cli::Result;
use crate::rules::RuleCategory;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

/// Represents a log entry after filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredEntry {
    /// The original log message
    pub message: String,
    /// The matched rule category, if any
    pub category: Option<RuleCategory>,
    /// Whether the entry matched any rule
    pub matched: bool,
}

impl FilteredEntry {
    /// Create a new filtered entry
    pub fn new(message: String, category: Option<RuleCategory>) -> Self {
        let matched = category.is_some();
        Self {
            message,
            category,
            matched,
        }
    }

    /// Check if this entry should be shown based on the show mode
    pub fn should_show(&self, show_mode: &ShowMode) -> bool {
        match show_mode {
            ShowMode::All => true,
            ShowMode::Violations => {
                matches!(
                    self.category,
                    Some(RuleCategory::Cracking) | Some(RuleCategory::Violations)
                )
            }
            ShowMode::Unmatched => !self.matched,
        }
    }
}

/// Trait for writing filtered log entries to various outputs
pub trait LogOutput {
    /// Write a single filtered entry
    fn write_entry(&mut self, entry: &FilteredEntry) -> Result<()>;

    /// Called when all entries have been processed
    fn finish(&mut self) -> Result<()>;
}

/// Factory function to create appropriate LogOutput implementation
pub fn create_output(
    format: &OutputFormat,
    show_mode: ShowMode,
    use_color: bool,
    output_file: Option<PathBuf>,
) -> Result<Box<dyn LogOutput>> {
    if let Some(path) = output_file {
        // Write to file
        let file = File::create(&path)
            .map_err(|e| anyhow::anyhow!("Failed to create output file {:?}: {}", path, e))?;
        let writer = Box::new(BufWriter::new(file)) as Box<dyn Write>;

        match format {
            OutputFormat::Text => Ok(Box::new(text::TextOutput::with_writer(
                show_mode, use_color, writer,
            ))),
            OutputFormat::Json => Ok(Box::new(json::JsonOutput::with_writer(show_mode, writer))),
        }
    } else {
        // Write to stdout (default)
        match format {
            OutputFormat::Text => Ok(Box::new(text::TextOutput::new(show_mode, use_color))),
            OutputFormat::Json => Ok(Box::new(json::JsonOutput::new(show_mode))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filtered_entry_creation() {
        let entry1 = FilteredEntry::new("test message".to_string(), Some(RuleCategory::Violations));
        assert_eq!(entry1.message, "test message");
        assert_eq!(entry1.category, Some(RuleCategory::Violations));
        assert!(entry1.matched);

        let entry2 = FilteredEntry::new("unmatched".to_string(), None);
        assert_eq!(entry2.message, "unmatched");
        assert_eq!(entry2.category, None);
        assert!(!entry2.matched);
    }

    #[test]
    fn test_should_show() {
        let violation = FilteredEntry::new("violation".to_string(), Some(RuleCategory::Violations));
        let system_event =
            FilteredEntry::new("system".to_string(), Some(RuleCategory::SystemEvents));
        let unmatched = FilteredEntry::new("unmatched".to_string(), None);

        // Test ShowMode::All
        assert!(violation.should_show(&ShowMode::All));
        assert!(system_event.should_show(&ShowMode::All));
        assert!(unmatched.should_show(&ShowMode::All));

        // Test ShowMode::Violations
        assert!(violation.should_show(&ShowMode::Violations));
        assert!(!system_event.should_show(&ShowMode::Violations));
        assert!(!unmatched.should_show(&ShowMode::Violations));

        // Test ShowMode::Unmatched
        assert!(!violation.should_show(&ShowMode::Unmatched));
        assert!(!system_event.should_show(&ShowMode::Unmatched));
        assert!(unmatched.should_show(&ShowMode::Unmatched));
    }
}
