use crate::cli::input::LogInput;
use crate::cli::Result;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;

/// File-based log input
pub struct FileInput {
    reader: Lines<BufReader<File>>,
    file_path: String,
}

impl FileInput {
    /// Create a new file input reader
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        Ok(Self {
            reader: reader.lines(),
            file_path: path.to_string_lossy().to_string(),
        })
    }
}

impl LogInput for FileInput {
    fn read_entry(&mut self) -> Result<Option<String>> {
        match self.reader.next() {
            Some(Ok(line)) => Ok(Some(line)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    fn source_name(&self) -> &str {
        &self.file_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_input() -> Result<()> {
        // Create a temporary file with test content
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "First log line")?;
        writeln!(temp_file, "Second log line")?;
        temp_file.flush()?;

        let mut input = FileInput::new(temp_file.path())?;

        // Test reading entries
        assert_eq!(input.read_entry()?, Some("First log line".to_string()));
        assert_eq!(input.read_entry()?, Some("Second log line".to_string()));
        assert_eq!(input.read_entry()?, None);

        // Test source name
        assert!(input.source_name().contains("tmp"));

        Ok(())
    }
}
