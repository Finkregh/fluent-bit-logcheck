use std::env;
use std::fs;
use std::path::Path;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Logcheck-based log filtering tool
#[derive(Parser)]
#[command(name = "logcheck-filter")]
#[command(about = "Filter logs using logcheck rules")]
#[command(version)]
pub struct Cli {
    /// Path to logcheck rules directory
    #[arg(long, required = true, help = "Path to logcheck rules directory")]
    pub rules: PathBuf,

    /// Output format
    #[arg(long, value_enum, default_value = "text", help = "Output format")]
    pub format: OutputFormat,

    /// Show mode
    #[arg(long, value_enum, default_value = "all", help = "What entries to show")]
    pub show: ShowMode,

    /// Show statistics after processing
    #[arg(long, help = "Show processing statistics")]
    pub stats: bool,

    /// Enable colored output
    #[arg(long, help = "Enable colored output")]
    pub color: bool,

    /// Write filtered logs to file (informational logs go to stdout)
    #[arg(long, help = "Write filtered logs to file")]
    pub output_file: Option<PathBuf>,

    /// Input source
    #[command(subcommand)]
    pub input: InputSource,
}

#[derive(Subcommand)]
pub enum InputSource {
    /// Read from a file
    File {
        /// Path to log file
        path: PathBuf,
    },
    /// Read from standard input
    Stdin,
    /// Read from systemd journal
    #[cfg(target_os = "linux")]
    Journald {
        /// Systemd unit to filter
        #[arg(long, help = "Filter by systemd unit")]
        unit: Option<String>,
        /// Follow mode (like tail -f)
        #[arg(long, help = "Follow new journal entries")]
        follow: bool,
        /// Number of lines to show from end
        #[arg(long, help = "Show last N entries")]
        lines: Option<usize>,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    /// Human-readable text format
    Text,
    /// JSON format
    Json,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ShowMode {
    /// Show all log entries
    All,
    /// Show only violations (cracking/violations)
    Violations,
    /// Show only unmatched entries
    Unmatched,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Skip generation when building on docs.rs
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=Skipping CLI documentation generation on docs.rs");
        return Ok(());
    }

    println!("cargo:rerun-if-changed=src/cli/args.rs");

    // Create docs directory
    let docs_dir = Path::new("docs");
    if !docs_dir.exists() {
        fs::create_dir_all(docs_dir)?;
        println!("cargo:warning=Created docs/ directory");
    }

    // Generate the CLI command
    let cmd = Cli::command();

    // Generate markdown CLI reference
    generate_markdown_docs(&cmd, docs_dir)?;

    // Generate man pages
    generate_man_pages(&cmd, docs_dir)?;

    Ok(())
}

fn generate_markdown_docs(
    _cmd: &clap::Command,
    docs_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let options = clap_markdown::MarkdownOptions::default()
        .title("logcheck-filter CLI Reference".to_string())
        .show_table_of_contents(true)
        .show_footer(true);

    let markdown_content = clap_markdown::help_markdown_custom::<Cli>(&options);

    let markdown_path = docs_dir.join("cli-reference.md");
    fs::write(&markdown_path, markdown_content)?;

    println!(
        "cargo:warning=Generated markdown CLI reference at {}",
        markdown_path.display()
    );

    Ok(())
}

fn generate_man_pages(
    cmd: &clap::Command,
    docs_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let man_dir = docs_dir.join("man");
    if !man_dir.exists() {
        fs::create_dir_all(&man_dir)?;
    }

    let man = clap_mangen::Man::new(cmd.clone());
    let mut buffer: Vec<u8> = Vec::new();
    man.render(&mut buffer)?;

    let man_path = man_dir.join("logcheck-filter.1");
    fs::write(&man_path, buffer)?;

    println!("cargo:warning=Generated man page at {}", man_path.display());

    Ok(())
}
