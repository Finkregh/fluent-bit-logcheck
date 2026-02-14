use crate::cli::Result;
use crate::cli::input::LogInput;
use std::io::{self, BufRead};

/// Standard input log reader
pub struct StdinInput {
    stdin: io::StdinLock<'static>,
}

impl Default for StdinInput {
    fn default() -> Self {
        Self::new()
    }
}

impl StdinInput {
    /// Create a new stdin input reader
    pub fn new() -> Self {
        Self {
            stdin: io::stdin().lock(),
        }
    }
}

impl LogInput for StdinInput {
    fn read_entry(&mut self) -> Result<Option<String>> {
        let mut line = String::new();
        match self.stdin.read_line(&mut line)? {
            0 => Ok(None), // EOF
            _ => {
                // Remove trailing newline
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                Ok(Some(line))
            }
        }
    }

    fn source_name(&self) -> &str {
        "stdin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdin_source_name() {
        let input = StdinInput::new();
        assert_eq!(input.source_name(), "stdin");
    }
}
