use anyhow::{Context, Result};
use grex::RegExpBuilder;
use std::fs;
use std::path::Path;

pub fn generate_patterns_from_logs(log_file: &Path, output_file: &Path) -> Result<()> {
    let log_content = fs::read_to_string(log_file)
        .with_context(|| format!("Failed to read log file {}", log_file.display()))?;
    let log_lines: Vec<&str> = log_content
        .lines()
        .filter(|line| !line.is_empty())
        .collect();

    if log_lines.is_empty() {
        anyhow::bail!("Log file contained no lines to process");
    }

    let regex = RegExpBuilder::from(&log_lines)
        .with_conversion_of_digits()
        .with_conversion_of_words()
        .with_conversion_of_repetitions()
        .build();

    let logcheck_rule = format!("# Generated from {}\n^{}$\n", log_file.display(), regex);

    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    fs::write(output_file, logcheck_rule)
        .with_context(|| format!("Failed to write rule file {}", output_file.display()))?;

    println!("Generated logcheck rule: {}", output_file.display());
    Ok(())
}
