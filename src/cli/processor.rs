use crate::cli::analyzer::UnmatchedCollector;
use crate::cli::input::LogInput;
use crate::cli::output::{FilteredEntry, LogOutput};
use crate::cli::Result;
use crate::rules::{LogcheckDatabase, RuleCategory};
use std::collections::HashMap;

/// Main log processor that orchestrates filtering
pub struct LogProcessor {
    database: LogcheckDatabase,
}

/// Statistics about the processing run
#[derive(Debug, Default)]
pub struct ProcessingStats {
    /// Total number of entries processed
    pub total: usize,
    /// Number of entries that matched rules
    pub matched: usize,
    /// Number of entries that didn't match any rules
    pub unmatched: usize,
    /// Breakdown by rule category
    pub by_category: HashMap<RuleCategory, usize>,
}

impl LogProcessor {
    /// Create a new log processor with the given database
    pub fn new(database: LogcheckDatabase) -> Self {
        Self { database }
    }

    /// Process log entries from input and write filtered results to output
    /// Optionally collect unmatched entries for analysis
    pub fn process(
        &self,
        input: &mut dyn LogInput,
        output: &mut dyn LogOutput,
    ) -> Result<ProcessingStats> {
        self.process_with_collector(input, output, None)
    }

    /// Process log entries with optional unmatched entry collection
    pub fn process_with_collector(
        &self,
        input: &mut dyn LogInput,
        output: &mut dyn LogOutput,
        mut collector: Option<&mut UnmatchedCollector>,
    ) -> Result<ProcessingStats> {
        let mut stats = ProcessingStats::default();

        // Process each log entry
        while let Some(message) = input.read_entry()? {
            stats.total += 1;

            // Apply logcheck rules to determine category
            let category = self.database.match_message(&message);

            // Update statistics
            if let Some(ref cat) = category {
                stats.matched += 1;
                *stats.by_category.entry(cat.clone()).or_insert(0) += 1;
            } else {
                stats.unmatched += 1;

                // Collect unmatched entry if collector is provided
                if let Some(ref mut collector) = collector {
                    collector.add_entry(message.clone());
                }
            }

            // Create filtered entry and write to output
            let filtered_entry = FilteredEntry::new(message, category);
            output.write_entry(&filtered_entry)?;
        }

        // Finish output processing
        output.finish()?;

        Ok(stats)
    }
}

impl ProcessingStats {
    /// Create a new empty statistics struct
    pub fn new() -> Self {
        Self::default()
    }

    /// Print statistics in a human-readable format
    pub fn print_summary(&self, source_name: &str) {
        self.print_summary_to(source_name, false);
    }

    /// Print statistics to stdout or stderr based on flag
    pub fn print_summary_to(&self, source_name: &str, to_stdout: bool) {
        if to_stdout {
            println!("\n📊 Processing Summary for {}", source_name);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Total entries processed: {}", self.total);
            println!("Matched entries: {}", self.matched);
            println!("Unmatched entries: {}", self.unmatched);

            if !self.by_category.is_empty() {
                println!("\nBreakdown by category:");
                let mut categories: Vec<_> = self.by_category.iter().collect();
                categories.sort_by_key(|(_, count)| *count);
                categories.reverse();

                for (category, count) in categories {
                    let category_name = match category {
                        RuleCategory::Cracking => "🔴 Cracking attempts",
                        RuleCategory::CrackingIgnore => "🟠 Cracking (ignored)",
                        RuleCategory::Violations => "🟡 Security violations",
                        RuleCategory::ViolationsIgnore => "🟠 Violations (ignored)",
                        RuleCategory::SystemEvents => "🟢 System events",
                        RuleCategory::Workstation => "🔵 Workstation events",
                        RuleCategory::Server => "🟣 Server events",
                        RuleCategory::Local => "⚪ Local rules",
                    };
                    println!("  {}: {}", category_name, count);
                }
            }

            if self.total > 0 {
                let match_rate = (self.matched as f64 / self.total as f64) * 100.0;
                println!("\nMatch rate: {:.1}%", match_rate);
            }
        } else {
            eprintln!("\n📊 Processing Summary for {}", source_name);
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("Total entries processed: {}", self.total);
            eprintln!("Matched entries: {}", self.matched);
            eprintln!("Unmatched entries: {}", self.unmatched);

            if !self.by_category.is_empty() {
                eprintln!("\nBreakdown by category:");
                let mut categories: Vec<_> = self.by_category.iter().collect();
                categories.sort_by_key(|(_, count)| *count);
                categories.reverse();

                for (category, count) in categories {
                    let category_name = match category {
                        RuleCategory::Cracking => "🔴 Cracking attempts",
                        RuleCategory::CrackingIgnore => "🟠 Cracking (ignored)",
                        RuleCategory::Violations => "🟡 Security violations",
                        RuleCategory::ViolationsIgnore => "🟠 Violations (ignored)",
                        RuleCategory::SystemEvents => "🟢 System events",
                        RuleCategory::Workstation => "🔵 Workstation events",
                        RuleCategory::Server => "🟣 Server events",
                        RuleCategory::Local => "⚪ Local rules",
                    };
                    eprintln!("  {}: {}", category_name, count);
                }
            }

            if self.total > 0 {
                let match_rate = (self.matched as f64 / self.total as f64) * 100.0;
                eprintln!("\nMatch rate: {:.1}%", match_rate);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args::{OutputFormat, ShowMode};
    use crate::cli::output::create_output;

    // Mock input for testing
    struct MockInput {
        messages: Vec<String>,
        current: usize,
    }

    impl MockInput {
        fn new(messages: Vec<String>) -> Self {
            Self {
                messages,
                current: 0,
            }
        }
    }

    impl LogInput for MockInput {
        fn read_entry(&mut self) -> Result<Option<String>> {
            if self.current < self.messages.len() {
                let msg = self.messages[self.current].clone();
                self.current += 1;
                Ok(Some(msg))
            } else {
                Ok(None)
            }
        }

        fn source_name(&self) -> &str {
            "mock"
        }
    }

    #[test]
    fn test_processor_with_mock_data() -> Result<()> {
        // Create a simple database with test rules
        let mut db = LogcheckDatabase::new();
        db.violations_rules.add_pattern(".*failed.*".to_string())?;
        db.server.add_pattern(".*session started.*".to_string())?;
        db.compile_all().unwrap();

        let processor = LogProcessor::new(db);

        // Create mock input
        let mut input = MockInput::new(vec![
            "authentication failed for user alice".to_string(),
            "session started for user bob".to_string(),
            "some random message".to_string(),
        ]);

        // Create output
        let mut output = create_output(&OutputFormat::Json, ShowMode::All, false, None)?;

        // Process
        let stats = processor.process(&mut input, output.as_mut())?;

        // Verify statistics
        assert_eq!(stats.total, 3);
        assert_eq!(stats.matched, 2);
        assert_eq!(stats.unmatched, 1);
        assert_eq!(stats.by_category.len(), 2);

        Ok(())
    }

    #[test]
    fn test_processing_stats() {
        let stats = ProcessingStats::new();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.matched, 0);
        assert_eq!(stats.unmatched, 0);
        assert!(stats.by_category.is_empty());

        // Test stats summary (should not panic)
        stats.print_summary("test");
    }
}
