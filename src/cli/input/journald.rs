use crate::cli::input::LogInput;
use crate::cli::Result;
use anyhow::anyhow;
use std::io::{BufRead, BufReader};
use std::process::{ChildStdout, Command, Stdio};

/// Journald log input using journalctl
pub struct JournaldInput {
    reader: Option<BufReader<ChildStdout>>,
    source_desc: String,
    unit_filter: Option<String>,
    follow: bool,
    lines: Option<usize>,
}

impl JournaldInput {
    /// Create a new journald input reader
    pub fn new(unit: Option<String>, follow: bool, lines: Option<usize>) -> Result<Self> {
        let source_desc = match (&unit, follow, lines) {
            (Some(u), true, _) => format!("journald(unit={},follow)", u),
            (Some(u), false, Some(n)) => format!("journald(unit={},lines={})", u, n),
            (Some(u), false, None) => format!("journald(unit={})", u),
            (None, true, _) => "journald(follow)".to_string(),
            (None, false, Some(n)) => format!("journald(lines={})", n),
            (None, false, None) => "journald".to_string(),
        };

        Ok(Self {
            reader: None,
            source_desc,
            unit_filter: unit,
            follow,
            lines,
        })
    }

    /// Initialize the journalctl process if not already started
    fn ensure_reader(&mut self) -> Result<()> {
        if self.reader.is_some() {
            return Ok(());
        }

        let mut cmd = Command::new("journalctl");

        // Add output format
        cmd.arg("--output=json");

        // Add unit filter if specified
        if let Some(ref unit) = self.unit_filter {
            cmd.arg("--unit").arg(unit);
        }

        // Add follow mode
        if self.follow {
            cmd.arg("--follow");
        }

        // Add lines limit
        if let Some(lines) = self.lines {
            cmd.arg("--lines").arg(lines.to_string());
        }

        // Don't page output
        cmd.arg("--no-pager");

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                anyhow!(
                    "Failed to start journalctl: {}. Make sure systemd-journald is available.",
                    e
                )
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to capture journalctl stdout"))?;

        self.reader = Some(BufReader::new(stdout));

        Ok(())
    }
}

impl LogInput for JournaldInput {
    fn read_entry(&mut self) -> Result<Option<String>> {
        self.ensure_reader()?;

        let reader = self
            .reader
            .as_mut()
            .ok_or_else(|| anyhow!("Journald reader not initialized"))?;

        let mut line = String::new();
        match reader.read_line(&mut line)? {
            0 => Ok(None), // EOF
            _ => {
                // Remove trailing newline
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }

                // Parse JSON to extract the MESSAGE field
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                    if let Some(message) = json.get("MESSAGE").and_then(|m| m.as_str()) {
                        Ok(Some(message.to_string()))
                    } else {
                        // If no MESSAGE field, return the raw line
                        Ok(Some(line))
                    }
                } else {
                    // If not valid JSON, return the raw line
                    Ok(Some(line))
                }
            }
        }
    }

    fn source_name(&self) -> &str {
        &self.source_desc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journald_source_description() {
        let input1 = JournaldInput::new(None, false, None).unwrap();
        assert_eq!(input1.source_name(), "journald");

        let input2 = JournaldInput::new(Some("sshd".to_string()), false, None).unwrap();
        assert_eq!(input2.source_name(), "journald(unit=sshd)");

        let input3 = JournaldInput::new(None, true, None).unwrap();
        assert_eq!(input3.source_name(), "journald(follow)");

        let input4 = JournaldInput::new(Some("sshd".to_string()), true, None).unwrap();
        assert_eq!(input4.source_name(), "journald(unit=sshd,follow)");

        let input5 = JournaldInput::new(None, false, Some(50)).unwrap();
        assert_eq!(input5.source_name(), "journald(lines=50)");
    }
}
