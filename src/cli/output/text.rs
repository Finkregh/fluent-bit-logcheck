use crate::cli::Result;
use crate::cli::args::ShowMode;
use crate::cli::output::{FilteredEntry, LogOutput};
use crate::rules::RuleCategory;
use colored::{ColoredString, Colorize};
use std::io::{self, Write};

/// Text-based output formatter
pub struct TextOutput {
    show_mode: ShowMode,
    use_color: bool,
    writer: Box<dyn Write>,
}

impl TextOutput {
    /// Create a new text output formatter that writes to stdout
    pub fn new(show_mode: ShowMode, use_color: bool) -> Self {
        Self {
            show_mode,
            use_color,
            writer: Box::new(io::stdout()),
        }
    }

    /// Create a new text output formatter with custom writer
    pub fn with_writer(show_mode: ShowMode, use_color: bool, writer: Box<dyn Write>) -> Self {
        Self {
            show_mode,
            use_color,
            writer,
        }
    }

    /// Format a rule category with appropriate color
    fn format_category(&self, category: &RuleCategory) -> ColoredString {
        let category_str = match category {
            RuleCategory::Cracking => "[CRACKING]",
            RuleCategory::CrackingIgnore => "[CRACKING-IGNORE]",
            RuleCategory::Violations => "[VIOLATION]",
            RuleCategory::ViolationsIgnore => "[VIOLATION-IGNORE]",
            RuleCategory::SystemEvents => "[SYSTEM]",
            RuleCategory::Workstation => "[WORKSTATION]",
            RuleCategory::Server => "[SERVER]",
            RuleCategory::Local => "[LOCAL]",
        };

        if self.use_color {
            match category {
                RuleCategory::Cracking => category_str.red().bold(),
                RuleCategory::Violations => category_str.yellow().bold(),
                RuleCategory::SystemEvents | RuleCategory::Server | RuleCategory::Workstation => {
                    category_str.green()
                }
                RuleCategory::Local => category_str.blue(),
                _ => category_str.normal(),
            }
        } else {
            category_str.normal()
        }
    }

    /// Format a message with appropriate color
    fn format_message(&self, message: &str, category: &Option<RuleCategory>) -> ColoredString {
        if self.use_color {
            match category {
                Some(RuleCategory::Cracking) => message.red(),
                Some(RuleCategory::Violations) => message.yellow(),
                Some(_) => message.normal(),
                None => message.cyan(), // Unmatched entries
            }
        } else {
            message.normal()
        }
    }
}

impl LogOutput for TextOutput {
    fn write_entry(&mut self, entry: &FilteredEntry) -> Result<()> {
        if !entry.should_show(&self.show_mode) {
            return Ok(());
        }

        match &entry.category {
            Some(category) => {
                let category_tag = self.format_category(category);
                let formatted_message = self.format_message(&entry.message, &entry.category);
                writeln!(self.writer, "{} {}", category_tag, formatted_message)?;
            }
            None => {
                let unmatched_tag = if self.use_color {
                    "[UNMATCHED]".cyan().bold()
                } else {
                    "[UNMATCHED]".normal()
                };
                let formatted_message = self.format_message(&entry.message, &entry.category);
                writeln!(self.writer, "{} {}", unmatched_tag, formatted_message)?;
            }
        }

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

    #[test]
    fn test_text_output_creation() {
        let output = TextOutput::new(ShowMode::All, false);
        assert!(!output.use_color);
        assert!(matches!(output.show_mode, ShowMode::All));

        let colored_output = TextOutput::new(ShowMode::Violations, true);
        assert!(colored_output.use_color);
        assert!(matches!(colored_output.show_mode, ShowMode::Violations));
    }

    #[test]
    fn test_category_formatting() {
        let output = TextOutput::new(ShowMode::All, false);

        let cracking_format = output.format_category(&RuleCategory::Cracking);
        assert_eq!(cracking_format.to_string(), "[CRACKING]");

        let violation_format = output.format_category(&RuleCategory::Violations);
        assert_eq!(violation_format.to_string(), "[VIOLATION]");

        let system_format = output.format_category(&RuleCategory::SystemEvents);
        assert_eq!(system_format.to_string(), "[SYSTEM]");
    }
}
