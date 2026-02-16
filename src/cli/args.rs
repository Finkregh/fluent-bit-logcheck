use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

/// Logcheck-based log filtering tool
#[derive(Parser)]
#[command(name = "logcheck-filter")]
#[command(about = "Filter logs using logcheck rules")]
#[command(version)]
pub struct Cli {
    /// Path to logcheck rules directory (defaults to /etc/logcheck)
    #[arg(
        long,
        default_value = "/etc/logcheck",
        help = "Path to logcheck rules directory"
    )]
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

    /// Generate CLI documentation in Markdown format
    #[arg(
        long,
        hide = true,
        help = "Generate CLI documentation in Markdown format"
    )]
    pub generate_docs: bool,

    /// Generate shell completion scripts
    #[arg(
        long,
        value_enum,
        hide = true,
        help = "Generate shell completion scripts"
    )]
    pub generate_completion: Option<Shell>,

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

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Test basic file input with explicit rules path
        let args = vec![
            "logcheck-filter",
            "--rules",
            "/etc/logcheck",
            "file",
            "/var/log/syslog",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.rules, PathBuf::from("/etc/logcheck"));
        assert!(matches!(cli.format, OutputFormat::Text));
        assert!(matches!(cli.show, ShowMode::All));
        assert!(!cli.stats);
        assert!(!cli.color);
        assert!(matches!(cli.input, InputSource::File { .. }));
    }

    #[test]
    fn test_cli_default_rules() {
        // Test that rules defaults to /etc/logcheck when not specified
        let args = vec!["logcheck-filter", "stdin"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.rules, PathBuf::from("/etc/logcheck"));
    }

    #[test]
    fn test_cli_with_options() {
        let args = vec![
            "logcheck-filter",
            "--rules",
            "/etc/logcheck",
            "--format",
            "json",
            "--show",
            "violations",
            "--stats",
            "--color",
            "stdin",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(matches!(cli.format, OutputFormat::Json));
        assert!(matches!(cli.show, ShowMode::Violations));
        assert!(cli.stats);
        assert!(cli.color);
        assert!(matches!(cli.input, InputSource::Stdin));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_journald_options() {
        let args = vec![
            "logcheck-filter",
            "--rules",
            "/etc/logcheck",
            "journald",
            "--unit",
            "sshd",
            "--follow",
            "--lines",
            "100",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        if let InputSource::Journald {
            unit,
            follow,
            lines,
        } = cli.input
        {
            assert_eq!(unit, Some("sshd".to_string()));
            assert!(follow);
            assert_eq!(lines, Some(100));
        } else {
            panic!("Expected Journald input source");
        }
    }
}
