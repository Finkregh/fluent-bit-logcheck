/// Interactive analyzer for unmatched log entries
///
/// This module provides pattern analysis and regex generation
/// for unmatched log entries via a TUI interface.
pub mod pattern_grouper;
pub mod rule_writer;
pub mod tui;

use crate::cli::Result;

/// Collector for unmatched log entries during processing
#[derive(Debug, Default)]
pub struct UnmatchedCollector {
    /// Unmatched log messages
    entries: Vec<String>,
}

impl UnmatchedCollector {
    /// Create a new empty collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an unmatched entry
    pub fn add_entry(&mut self, message: String) {
        self.entries.push(message);
    }

    /// Get all collected entries
    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    /// Get the number of collected entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if collector is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Launch the interactive analyzer TUI
    pub fn analyze(&self, min_group_size: usize) -> Result<()> {
        if self.is_empty() {
            eprintln!("No unmatched entries to analyze");
            return Ok(());
        }

        // Group similar entries and generate patterns
        let pattern_groups = pattern_grouper::group_and_generate(self.entries(), min_group_size)?;

        if pattern_groups.is_empty() {
            eprintln!("No patterns could be generated (try lowering --min-group-size)");
            return Ok(());
        }

        // Launch TUI
        tui::run_analyzer(pattern_groups, self.entries())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector() {
        let mut collector = UnmatchedCollector::new();
        assert!(collector.is_empty());
        assert_eq!(collector.len(), 0);

        collector.add_entry("test entry 1".to_string());
        collector.add_entry("test entry 2".to_string());

        assert!(!collector.is_empty());
        assert_eq!(collector.len(), 2);
        assert_eq!(collector.entries()[0], "test entry 1");
        assert_eq!(collector.entries()[1], "test entry 2");
    }
}
