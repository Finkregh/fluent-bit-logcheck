pub mod file;
#[cfg(target_os = "linux")]
pub mod journald;
pub mod stdin;

use crate::cli::args::InputSource;
use crate::cli::Result;

/// Trait for reading log entries from various sources
pub trait LogInput {
    /// Read the next log entry
    /// Returns Ok(Some(entry)) for a new entry, Ok(None) for EOF
    fn read_entry(&mut self) -> Result<Option<String>>;

    /// Get a descriptive name for this input source
    fn source_name(&self) -> &str;
}

/// Factory function to create appropriate LogInput implementation
pub fn create_input(source: &InputSource) -> Result<Box<dyn LogInput>> {
    match source {
        InputSource::File { path } => Ok(Box::new(file::FileInput::new(path)?)),
        InputSource::Stdin => Ok(Box::new(stdin::StdinInput::new())),
        #[cfg(target_os = "linux")]
        InputSource::Journald {
            unit,
            follow,
            lines,
            mode: _,
        } => Ok(Box::new(journald::JournaldInput::new(
            unit.clone(),
            *follow,
            *lines,
        )?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args::InputSource;

    #[test]
    fn test_create_stdin_input() {
        let source = InputSource::Stdin;
        let input = create_input(&source).unwrap();
        assert_eq!(input.source_name(), "stdin");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_create_journald_input() {
        let source = InputSource::Journald {
            unit: Some("sshd".to_string()),
            follow: false,
            lines: Some(10),
            mode: None,
        };
        let input = create_input(&source).unwrap();
        assert!(input.source_name().contains("journald"));
    }
}
