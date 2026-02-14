use crate::cli::Result;
use crate::cli::args::ShowMode;
use crate::cli::output::{FilteredEntry, LogOutput};
use serde_json;
use std::io::{self, Write};

/// JSON-based output formatter
pub struct JsonOutput {
    show_mode: ShowMode,
    writer: Box<dyn Write>,
}

impl JsonOutput {
    /// Create a new JSON output formatter that writes to stdout
    pub fn new(show_mode: ShowMode) -> Self {
        Self {
            show_mode,
            writer: Box::new(io::stdout()),
        }
    }

    /// Create a new JSON output formatter with custom writer
    pub fn with_writer(show_mode: ShowMode, writer: Box<dyn Write>) -> Self {
        Self { show_mode, writer }
    }
}

impl LogOutput for JsonOutput {
    fn write_entry(&mut self, entry: &FilteredEntry) -> Result<()> {
        if !entry.should_show(&self.show_mode) {
            return Ok(());
        }

        // Create a JSON representation of the filtered entry
        let json_entry = serde_json::json!({
            "message": entry.message,
            "matched": entry.matched,
            "category": entry.category.as_ref().map(|c| format!("{:?}", c)),
            "rule_type": match &entry.category {
                Some(cat) => match cat {
                    crate::rules::RuleCategory::Cracking => "cracking",
                    crate::rules::RuleCategory::CrackingIgnore => "cracking_ignore",
                    crate::rules::RuleCategory::Violations => "violations",
                    crate::rules::RuleCategory::ViolationsIgnore => "violations_ignore",
                    crate::rules::RuleCategory::SystemEvents => "ignore",
                    crate::rules::RuleCategory::Workstation => "ignore",
                    crate::rules::RuleCategory::Server => "ignore",
                    crate::rules::RuleCategory::Local => "ignore",
                },
                None => "unmatched",
            }
        });

        writeln!(self.writer, "{}", serde_json::to_string(&json_entry)?)?;
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::RuleCategory;

    #[test]
    fn test_json_output_creation() {
        let output = JsonOutput::new(ShowMode::All);
        assert!(matches!(output.show_mode, ShowMode::All));

        let violations_output = JsonOutput::new(ShowMode::Violations);
        assert!(matches!(violations_output.show_mode, ShowMode::Violations));
    }

    #[test]
    fn test_json_formatting() -> Result<()> {
        let mut output = JsonOutput::new(ShowMode::All);

        let entry = FilteredEntry::new("test message".to_string(), Some(RuleCategory::Violations));

        // This should not panic and should produce valid JSON
        output.write_entry(&entry)?;

        let unmatched = FilteredEntry::new("unmatched".to_string(), None);
        output.write_entry(&unmatched)?;

        Ok(())
    }
}
