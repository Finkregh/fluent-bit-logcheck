use anyhow::Result;
use logcheck_fluent_bit_filter::cli::{
    args::Cli, input::create_input, output::create_output, processor::LogProcessor,
};
use logcheck_fluent_bit_filter::rules::LogcheckDatabase;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse_args();

    // When --output-file is specified, info goes to stdout; otherwise stderr
    let info_to_stdout = cli.output_file.is_some();

    // Load logcheck database
    if info_to_stdout {
        println!("Loading logcheck rules from: {}", cli.rules.display());
    } else {
        eprintln!("Loading logcheck rules from: {}", cli.rules.display());
    }

    let database = LogcheckDatabase::load_from_directory(&cli.rules)
        .map_err(|e| anyhow::anyhow!("Failed to load logcheck rules: {:?}", e))?;

    // Print database statistics
    let stats = database.get_stats();
    if info_to_stdout {
        println!(
            "Loaded {} rules across {} categories",
            stats.get("total_rules").unwrap_or(&0),
            stats.len() - 1
        );
    } else {
        eprintln!(
            "Loaded {} rules across {} categories",
            stats.get("total_rules").unwrap_or(&0),
            stats.len() - 1
        );
    }

    // Create input source
    let mut input = create_input(&cli.input)?;
    if info_to_stdout {
        println!("Reading from: {}", input.source_name());
    } else {
        eprintln!("Reading from: {}", input.source_name());
    }

    // Create output formatter
    let mut output = create_output(&cli.format, cli.show, cli.color, cli.output_file)?;

    // Create processor and run
    let processor = LogProcessor::new(database);
    let processing_stats = processor.process(input.as_mut(), output.as_mut())?;

    // Show statistics if requested
    if cli.stats {
        processing_stats.print_summary_to(input.source_name(), info_to_stdout);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_rules_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Create violations.d directory with a simple rule
        let violations_dir = temp_dir.path().join("violations.d");
        fs::create_dir_all(&violations_dir).unwrap();
        fs::write(violations_dir.join("test"), ".*test violation.*\n").unwrap();

        temp_dir
    }

    #[test]
    fn test_database_loading() {
        let temp_dir = create_test_rules_dir();
        let database = LogcheckDatabase::load_from_directory(temp_dir.path()).unwrap();
        let stats = database.get_stats();

        assert!(stats.get("total_rules").unwrap_or(&0) > &0);
        assert!(stats.get("violations_rules").unwrap_or(&0) > &0);
    }
}
